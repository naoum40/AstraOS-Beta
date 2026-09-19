// main.rs - AstraOS Minesweeper.
//
// Classic 9x9 / 10-mines Minesweeper game built with GTK4-rs.
//
// Layout:
//   * Header row (mine count label, smiley reset button, timer label)
//   * 9x9 grid of `Button`s
//
// Game flow:
//   * Mines are placed on the FIRST click (so the first click is always
//     safe — its cell + neighbors are excluded from mine placement).
//   * Left-click on a cell reveals it. Revealing a 0 cell triggers a
//     recursive flood-fill that reveals every adjacent 0 cell and
//     their direct neighbors (so the borders of the flood region are
//     numbered cells).
//   * Right-click (via a `GestureClick` with `button = 3`) toggles a
//     flag on an unrevealed cell.
//   * Stepping on a mine ends the game (lose). Revealing every non-mine
//     cell wins.
//   * The smiley reset button starts a fresh game: cleared grid, new
//     mine placement on next click, timer reset.
//
// RNG: a tiny xorshift64 seeded from `SystemTime` (no `rand` dep), which
// is more than enough for a 9x9 board.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gio::ApplicationFlags;
use glib::ControlFlow;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, GestureClick, Grid, Label,
    Orientation,
};

/// GApplication ID.
const APP_ID: &str = "org.astraos.Minesweeper";

/// 9x9 grid.
const GRID: usize = 9;

/// 10 mines (Beginner difficulty).
const MINES: usize = 10;

/// System path for the installed stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/minesweeper.css";

/// Minimal embedded stylesheet (used when the system theme is missing).
const FALLBACK_CSS: &str = r#"
window.astra-minesweeper {
    background: rgba(11, 13, 23, 0.96);
    color: #f1f5f9;
    font-family: 'Outfit', 'Inter', 'Cantarell', sans-serif;
}
.mw-header {
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    padding: 8px 12px;
    margin: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
}
.mw-counter {
    font-size: 14pt;
    font-weight: 700;
    color: #818cf8;
    min-width: 70px;
}
.mw-reset {
    font-size: 20pt;
    min-width: 44px;
    min-height: 44px;
    border-radius: 10px;
    background: rgba(129, 140, 248, 0.20);
    border: 1px solid rgba(129, 140, 248, 0.35);
}
.mw-reset:hover {
    background: rgba(129, 140, 248, 0.35);
}
.mw-cell {
    min-width: 32px;
    min-height: 32px;
    font-weight: 700;
    font-size: 12pt;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #f1f5f9;
    padding: 0;
}
.mw-cell:hover {
    background: rgba(255, 255, 255, 0.16);
}
.mw-cell.revealed {
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.03);
}
.mw-cell.revealed:hover {
    background: rgba(255, 255, 255, 0.02);
}
.mw-cell.mine {
    background: rgba(255, 45, 85, 0.55);
    color: #ffffff;
}
.mw-cell.flagged {
    color: #fbbf24;
}
.mw-cell.num-1 { color: #00c6ff; }
.mw-cell.num-2 { color: #00ff00; }
.mw-cell.num-3 { color: #ff2d55; }
.mw-cell.num-4 { color: #ffb900; }
.mw-cell.num-5 { color: #ff007f; }
.mw-cell.num-6 { color: #00b7c3; }
.mw-cell.num-7 { color: #ffffff; }
.mw-cell.num-8 { color: #94a3b8; }
"#;

// =============================================================================
// RNG: xorshift64 seeded from SystemTime (no external `rand` dep)
// =============================================================================

/// Xorshift64 step. Mutates `state` in place and returns the next u64.
fn xorshift_next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Uniform-ish usize in `[0, max)` from `state`.
fn rand_range(state: &mut u64, max: usize) -> usize {
    if max == 0 {
        return 0;
    }
    (xorshift_next(state) as usize) % max
}

// =============================================================================
// Game state
// =============================================================================

/// One cell on the board. `Empty(n)` carries the adjacent-mine count
/// (0..=8); `Mine` is a mine.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Mine,
    Empty(u8),
}

struct GameState {
    grid: Vec<Vec<Cell>>,
    revealed: Vec<Vec<bool>>,
    flagged: Vec<Vec<bool>>,
    game_over: bool,
    won: bool,
    started: bool,
    elapsed: u32,
    flags_placed: usize,
}

impl GameState {
    fn new() -> Self {
        Self {
            grid: vec![vec![Cell::Empty(0); GRID]; GRID],
            revealed: vec![vec![false; GRID]; GRID],
            flagged: vec![vec![false; GRID]; GRID],
            game_over: false,
            won: false,
            started: false,
            elapsed: 0,
            flags_placed: 0,
        }
    }

    /// Place `MINES` mines randomly, excluding `(exclude_r, exclude_c)`
    /// and its 8 neighbors so the first click is always safe + opens a
    /// region (not a single cell).
    fn place_mines(&mut self, rng: &mut u64, exclude_r: usize, exclude_c: usize) {
        // Mark the excluded (first-click + neighbors) cells.
        let mut forbidden = vec![vec![false; GRID]; GRID];
        for dr in -1i32..=1 {
            for dc in -1i32..=1 {
                let nr = exclude_r as i32 + dr;
                let nc = exclude_c as i32 + dc;
                if nr >= 0 && (nr as usize) < GRID && nc >= 0 && (nc as usize) < GRID {
                    forbidden[nr as usize][nc as usize] = true;
                }
            }
        }

        let mut placed: usize = 0;
        while placed < MINES {
            let r = rand_range(rng, GRID);
            let c = rand_range(rng, GRID);
            if forbidden[r][c] {
                continue;
            }
            if matches!(self.grid[r][c], Cell::Mine) {
                continue;
            }
            self.grid[r][c] = Cell::Mine;
            placed += 1;
        }

        // Compute adjacent counts for non-mine cells.
        for r in 0..GRID {
            for c in 0..GRID {
                if matches!(self.grid[r][c], Cell::Mine) {
                    continue;
                }
                let mut count: u8 = 0;
                for dr in -1i32..=1 {
                    for dc in -1i32..=1 {
                        if dr == 0 && dc == 0 {
                            continue;
                        }
                        let nr = r as i32 + dr;
                        let nc = c as i32 + dc;
                        if nr >= 0
                            && (nr as usize) < GRID
                            && nc >= 0
                            && (nc as usize) < GRID
                            && matches!(self.grid[nr as usize][nc as usize], Cell::Mine)
                        {
                            count += 1;
                        }
                    }
                }
                self.grid[r][c] = Cell::Empty(count);
            }
        }
    }

    /// Reveal `(r, c)`. If empty (0 mines adjacent), recursively reveal
    /// all 8 neighbors. Hits on mine → game over.
    fn reveal(&mut self, r: usize, c: usize) {
        if self.game_over || self.won {
            return;
        }
        if self.flagged[r][c] || self.revealed[r][c] {
            return;
        }
        self.revealed[r][c] = true;
        if matches!(self.grid[r][c], Cell::Mine) {
            self.game_over = true;
            return;
        }
        if matches!(self.grid[r][c], Cell::Empty(0)) {
            for dr in -1i32..=1 {
                for dc in -1i32..=1 {
                    if dr == 0 && dc == 0 {
                        continue;
                    }
                    let nr = r as i32 + dr;
                    let nc = c as i32 + dc;
                    if nr >= 0 && (nr as usize) < GRID && nc >= 0 && (nc as usize) < GRID {
                        self.reveal(nr as usize, nc as usize);
                    }
                }
            }
        }
        self.check_win();
    }

    /// Toggle the flag on an unrevealed cell.
    fn toggle_flag(&mut self, r: usize, c: usize) {
        if self.game_over || self.won {
            return;
        }
        if self.revealed[r][c] {
            return;
        }
        if self.flagged[r][c] {
            self.flagged[r][c] = false;
            self.flags_placed = self.flags_placed.saturating_sub(1);
        } else {
            self.flagged[r][c] = true;
            self.flags_placed += 1;
        }
    }

    /// Win = every non-mine cell is revealed.
    fn check_win(&mut self) {
        for r in 0..GRID {
            for c in 0..GRID {
                if !matches!(self.grid[r][c], Cell::Mine) && !self.revealed[r][c] {
                    return;
                }
            }
        }
        self.won = true;
    }
}

// =============================================================================
// App state (game + UI references)
// =============================================================================

struct App {
    game: GameState,
    cells: Vec<Vec<Button>>,
    mine_label: Label,
    timer_label: Label,
    reset_button: Button,
    timer_id: Option<glib::SourceId>,
    rng_state: u64,
}

impl App {
    /// (Re)start the game: clear state, regenerate cell buttons, reset
    /// timer + mine counter.
    fn restart(&mut self) {
        // Stop existing timer, if any.
        if let Some(id) = self.timer_id.take() {
            id.remove();
        }
        self.game = GameState::new();
        self.render();
    }

    /// Start the per-second timer (idempotent — only starts once per game).
    fn ensure_timer_started(&mut self) {
        if self.timer_id.is_some() {
            return;
        }
        let timer_label = self.timer_label.clone();
        // We can't share `&mut self` into the closure, so we keep the
        // elapsed counter in a shared `Cell<u32>` incremented by the
        // timeout callback. The per-second label update is all we need
        // for a 9x9 game.
        let elapsed = std::rc::Rc::new(std::cell::Cell::new(0u32));
        let elapsed_clone = elapsed.clone();
        let id = glib::timeout_add_local(Duration::from_secs(1), move || {
            let next = elapsed_clone.get() + 1;
            elapsed_clone.set(next);
            timer_label.set_label(&format!("⏱ {:03}", next));
            ControlFlow::Continue
        });
        self.timer_id = Some(id);
    }

    /// Re-render every cell + header from the current game state.
    fn render(&self) {
        let all_classes: [&str; 9] = [
            "revealed", "flagged", "mine", "num-1", "num-2", "num-3", "num-4", "num-5", "num-6",
        ];
        let _extra_classes: [&str; 2] = ["num-7", "num-8"];
        for r in 0..GRID {
            for c in 0..GRID {
                let btn = &self.cells[r][c];
                for cls in all_classes.iter().chain(["num-7", "num-8"].iter()) {
                    btn.remove_css_class(cls);
                }
                btn.set_label("");
                btn.set_sensitive(true);

                let is_revealed = self.game.revealed[r][c];
                let is_flagged = self.game.flagged[r][c];
                let is_mine = matches!(self.game.grid[r][c], Cell::Mine);
                let is_game_over = self.game.game_over;

                if is_flagged {
                    btn.set_label("🚩");
                    btn.add_css_class("flagged");
                    if is_game_over && is_mine {
                        // Keep flag
                    }
                }
                if is_revealed || (is_game_over && is_mine) {
                    if is_mine {
                        btn.set_label("💣");
                        btn.add_css_class("mine");
                        btn.add_css_class("revealed");
                        btn.set_sensitive(false);
                    } else if let Cell::Empty(n) = self.game.grid[r][c] {
                        btn.add_css_class("revealed");
                        if n > 0 {
                            btn.set_label(&n.to_string());
                            btn.add_css_class(&format!("num-{}", n));
                        }
                        btn.set_sensitive(false);
                    }
                } else if is_game_over || self.game.won {
                    btn.set_sensitive(false);
                }
            }
        }

        // Header counters.
        let mine_count = MINES as i32 - self.game.flags_placed as i32;
        self.mine_label
            .set_label(&format!("💣 {:03}", mine_count.max(0)));
        self.timer_label
            .set_label(&format!("⏱ {:03}", self.game.elapsed));

        // Reset button face.
        if self.game.won {
            self.reset_button.set_label("😎");
        } else if self.game.game_over {
            self.reset_button.set_label("😵");
        } else {
            self.reset_button.set_label("😊");
        }
    }
}

// =============================================================================
// Theme
// =============================================================================

fn load_theme() {
    let provider = CssProvider::new();
    let css = match std::fs::read_to_string(THEME_PATH) {
        Ok(c) => {
            log::info!("Loaded minesweeper theme from {}", THEME_PATH);
            c
        }
        Err(_) => {
            log::warn!(
                "Theme not found at {} — using embedded fallback CSS",
                THEME_PATH
            );
            FALLBACK_CSS.to_string()
        }
    };
    provider.load_from_data(&css);
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

// =============================================================================
// UI build
// =============================================================================

fn build_ui(app: &Application) {
    load_theme();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Astra Minesweeper")
        .default_width(340)
        .default_height(480)
        .build();
    window.add_css_class("astra-minesweeper");

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    window.set_child(Some(&root));

    // --- Header (mine count + smiley + timer) ----------------------------
    let header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(gtk4::Align::Center)
        .css_classes(["mw-header"])
        .build();

    let mine_label = Label::new(Some("💣 010"));
    mine_label.add_css_class("mw-counter");

    let reset_button = Button::with_label("😊");
    reset_button.add_css_class("mw-reset");

    let timer_label = Label::new(Some("⏱ 000"));
    timer_label.add_css_class("mw-counter");

    header.append(&mine_label);
    header.append(&reset_button);
    header.append(&timer_label);
    root.append(&header);

    // --- Grid of cells ---------------------------------------------------
    let grid = Grid::builder()
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .row_spacing(2)
        .column_spacing(2)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    root.append(&grid);

    // Build cells + wire signals. The shared state is `Rc<RefCell<App>>`.
    let mut cells: Vec<Vec<Button>> = Vec::with_capacity(GRID);
    for r in 0..GRID {
        let mut row: Vec<Button> = Vec::with_capacity(GRID);
        for c in 0..GRID {
            let btn = Button::new();
            btn.add_css_class("mw-cell");
            btn.set_label("");
            grid.attach(&btn, c as i32, r as i32, 1, 1);
            row.push(btn);
        }
        cells.push(row);
    }

    let state = Rc::new(RefCell::new(App {
        game: GameState::new(),
        cells,
        mine_label: mine_label.clone(),
        timer_label: timer_label.clone(),
        reset_button: reset_button.clone(),
        timer_id: None,
        rng_state: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xC0FFEE),
    }));

    // Wire each cell's click + right-click signals.
    for r in 0..GRID {
        for c in 0..GRID {
            let btn = state.borrow().cells[r][c].clone();

            // Left-click → reveal
            {
                let state = state.clone();
                btn.connect_clicked(move |_| {
                    let mut app = state.borrow_mut();
                    if app.game.game_over || app.game.won {
                        return;
                    }
                    if !app.game.started {
                        // Place mines, excluding this cell + neighbors.
                        app.game.started = true;
                        let mut rng = app.rng_state;
                        // Stir the rng a bit with cell coords so two
                        // back-to-back "new games" with the same time
                        // seed still get different layouts.
                        rng = rng
                            .wrapping_add((r as u64).wrapping_mul(0x9E3779B97F4A7C15))
                            .wrapping_add((c as u64).wrapping_mul(0xC2B2AE3D27D4EB4F));
                        app.rng_state = rng;
                        app.game.place_mines(&mut rng, r, c);
                        app.ensure_timer_started();
                    }
                    app.game.reveal(r, c);
                    app.render();
                });
            }

            // Right-click → toggle flag
            {
                let state = state.clone();
                let gesture = GestureClick::new();
                gesture.set_button(3);
                btn.add_controller(gesture.clone());
                gesture.connect_pressed(move |_, _, _, _| {
                    let mut app = state.borrow_mut();
                    if app.game.game_over || app.game.won {
                        return;
                    }
                    app.game.toggle_flag(r, c);
                    app.render();
                });
            }
        }
    }

    // Reset button
    {
        let state = state.clone();
        reset_button.connect_clicked(move |_| {
            let mut app = state.borrow_mut();
            app.restart();
        });
    }

    // Initial render.
    state.borrow().render();
    window.present();
}

fn main() {
    env_logger::init();
    log::info!("Astra Minesweeper v0.4.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);
    app.run();
}
