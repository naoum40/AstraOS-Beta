// weather.rs - AstraOS Weather widget.
//
// Floating glassmorphism panel that shows the configured city's
// current temperature, an emoji condition icon (☀️ 🌧️ ⛅ ❄️) and a
// short description. Data is fetched from `https://wttr.in/{city}?format=j1`
// (a free, no-key weather API) every 30 minutes using `reqwest::blocking`
// on a worker thread; results are cached so the UI never blocks.
//
// Pipeline:
//   1. Build the window + three labels: city, temp (large), condition.
//   2. Spawn a worker thread that performs the first fetch immediately,
//      then sleeps 30 minutes between refreshes. Each fetch result is
//      marshalled back to the GTK main thread via `glib::idle_add_local`
//      with a `SendWeakRef<Label>` for each label (safe across threads).
//   3. On error (no network / wttr.in unreachable) the widget shows
//      `--` placeholders and a "offline" hint; it retries on the next
//      30-min tick.
//
// JSON shape (subset we parse):
//   {
//     "current_condition": [{
//       "temp_C": "18",
//       "weatherCode": "113",       // 113=clear, 116=partly cloudy, ...
//       "weatherDesc": [{"value": "Sunny"}]
//     }]
//   }

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use gtk4::glib::object::SendWeakRef;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};
use serde::Deserialize;

use crate::widgets::attach_drag;

/// Refresh interval — 30 minutes between wttr.in polls.
const REFRESH_INTERVAL: Duration = Duration::from_secs(30 * 60);

/// Default city shown on first boot (configurable later via the
/// settings panel; hardcoded for ISO 4).
const DEFAULT_CITY: &str = "Paris";

/// The weather widget — owns its `ApplicationWindow`.
pub struct WeatherWidget {
    #[allow(dead_code)] // held for the lifetime of the app
    window: ApplicationWindow,
}

impl WeatherWidget {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Astra Weather")
            .default_width(220)
            .default_height(140)
            .decorated(false)
            .build();
        window.add_css_class("astra-widget");
        window.add_css_class("weather");

        let container = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .halign(gtk4::Align::Center)
            .valign(gtk4::Align::Center)
            .margin_start(16)
            .margin_end(16)
            .margin_top(14)
            .margin_bottom(14)
            .build();
        window.set_child(Some(&container));

        let city_label = Label::builder()
            .label(DEFAULT_CITY)
            .css_classes(["weather-city"])
            .build();
        container.append(&city_label);

        let temp_label = Label::builder()
            .label("--°")
            .css_classes(["weather-temp"])
            .build();
        container.append(&temp_label);

        let cond_label = Label::builder()
            .label("…")
            .css_classes(["weather-cond"])
            .build();
        container.append(&cond_label);

        // Restore saved position.
        // NOTE: gtk4-rs 0.9 removed `Window::move_()` (deprecated C API
        // in GTK 4.10) — the saved position is honored by the WM via
        // layer-shell rules (post-ISO 4); ISO 4 lets the WM place the
        // window with its default size.
        attach_drag(&window, "weather");

        // Cross-thread cache — written by the worker thread, read by
        // anyone needing the latest snapshot (e.g. for future UI hooks).
        let cache: Arc<Mutex<Option<WeatherSnapshot>>> = Arc::new(Mutex::new(None));

        // SendWeakRef<Label> is `Send + Sync` even though `Label` is
        // neither — the worker thread can safely hold it and only
        // `upgrade()` it back into a strong `Label` while on the GTK
        // main thread (inside `glib::idle_add_local`).
        let city_w: SendWeakRef<Label> = city_label.downgrade().into();
        let temp_w: SendWeakRef<Label> = temp_label.downgrade().into();
        let cond_w: SendWeakRef<Label> = cond_label.downgrade().into();
        let cache_for_thread = cache.clone();
        let city = DEFAULT_CITY.to_string();

        thread::spawn(move || loop {
            match fetch_weather(&city) {
                Ok(snap) => {
                    log::info!("[weather] {}: {}°, {}", city, snap.temp_c, snap.desc);
                    {
                        let mut g = cache_for_thread.lock().expect("cache lock");
                        *g = Some(snap.clone());
                    }
                    let city_w = city_w.clone();
                    let temp_w = temp_w.clone();
                    let cond_w = cond_w.clone();
                    let snap = snap;
                    glib::idle_add_once(move || {
                        if let (Some(c), Some(t), Some(d)) =
                            (city_w.upgrade(), temp_w.upgrade(), cond_w.upgrade())
                        {
                            c.set_label(&snap.city);
                            t.set_label(&format!("{}°C", snap.temp_c));
                            d.set_label(&format!("{} {}", snap.emoji, snap.desc));
                        }
                    });
                }
                Err(e) => {
                    log::warn!("[weather] fetch failed for {}: {}", city, e);
                    let temp_w = temp_w.clone();
                    let cond_w = cond_w.clone();
                    glib::idle_add_once(move || {
                        if let Some(t) = temp_w.upgrade() {
                            t.set_label("--°");
                        }
                        if let Some(c) = cond_w.upgrade() {
                            c.set_label("offline");
                        }
                    });
                }
            }
            thread::sleep(REFRESH_INTERVAL);
        });

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

/// In-memory snapshot of the current weather state — what the worker
/// thread writes after each successful fetch.
#[derive(Debug, Clone)]
struct WeatherSnapshot {
    city: String,
    temp_c: i32,
    desc: String,
    emoji: &'static str,
}

/// Subset of the wttr.in JSON response we care about.
/// `weatherDesc` is an array of objects with a `value` field — we model
/// only the first entry's `value`.
#[derive(Debug, Deserialize)]
struct WttrResponse {
    current_condition: Vec<WttrCurrent>,
}

#[derive(Debug, Deserialize)]
struct WttrCurrent {
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "weatherCode")]
    weather_code: String,
    #[serde(rename = "weatherDesc")]
    weather_desc: Vec<WttrDesc>,
}

#[derive(Debug, Deserialize)]
struct WttrDesc {
    value: String,
}

/// Perform a blocking HTTP GET to `https://wttr.in/{city}?format=j1`,
/// parse the JSON body, and return a `WeatherSnapshot`.
///
/// On any failure (network error, malformed JSON, missing fields) the
/// error is bubbled up to the caller — the widget renders an "offline"
/// placeholder for that tick.
fn fetch_weather(city: &str) -> Result<WeatherSnapshot, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://wttr.in/{}?format=j1", city);
    log::debug!("[weather] GET {}", url);
    let body = reqwest::blocking::get(&url)?.text()?;
    let parsed: WttrResponse = serde_json::from_str(&body)?;
    let cur = parsed
        .current_condition
        .into_iter()
        .next()
        .ok_or("no current_condition in response")?;
    let temp_c: i32 = cur.temp_c.parse().map_err(|_| "temp_C not a number")?;
    let desc = cur
        .weather_desc
        .into_iter()
        .next()
        .map(|d| d.value)
        .unwrap_or_else(|| "Unknown".to_string());
    let emoji = pick_emoji(&cur.weather_code, &desc);
    Ok(WeatherSnapshot {
        city: city.to_string(),
        temp_c,
        desc,
        emoji,
    })
}

/// Map a wttr.in weather code + description into one of the four emoji
/// the spec calls for (☀️ 🌧️ ⛅ ❄️). Falls back to ⛅ when the code is
/// unknown — wttr.in returns ~30 distinct codes; we bucket them.
///
/// Reference: https://wttr.in/:help (weather code table)
fn pick_emoji(code: &str, desc: &str) -> &'static str {
    match code {
        "113" => "☀️",
        "116" => "⛅",
        "119" | "122" => "☁️",
        "143" | "248" | "260" => "🌫️",
        "176" | "263" | "266" | "281" | "284" | "293" | "296" | "299" | "302" | "305" | "308"
        | "311" | "314" | "317" | "350" | "353" | "356" | "359" | "362" | "365" | "368" | "371"
        | "374" | "377" | "386" | "389" | "392" | "395" => "🌧️",
        "179" | "182" | "185" | "227" | "230" | "320" | "323" | "326" | "329" | "332" | "335"
        | "338" => "❄️",
        "200" => "⛈️",
        _ => {
            // Fall back to keyword match on the description.
            let d = desc.to_lowercase();
            if d.contains("snow") || d.contains("ice") {
                "❄️"
            } else if d.contains("rain") || d.contains("drizzle") || d.contains("shower") {
                "🌧️"
            } else if d.contains("cloud") || d.contains("overcast") {
                "☁️"
            } else if d.contains("clear") || d.contains("sunny") {
                "☀️"
            } else {
                "⛅"
            }
        }
    }
}
