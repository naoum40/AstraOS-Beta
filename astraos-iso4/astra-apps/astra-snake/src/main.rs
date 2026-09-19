// main.rs - AstraOS Snake.
//
// Classic Snake game built with GTK4-rs + cairo. The board is a
// 20x20 grid (each cell 18px → 360x360 canvas) drawn on a
// `gtk4::DrawingArea` via a cairo draw function.
//
// Controls:
//   * Arrow keys (or WASD) to steer the snake.
//   * Reset button restarts the game at any time.
//   * Spacebar also restarts after a game over.
//
// Game state lives in an `Rc<RefCell<GameState>>` shared between:
//   * the draw function closure,
//   * the EventControllerKey closure (arrow keys),
//   * the per-tick `glib::timeout_add_local` closure (120ms interval),
//   * the reset button closure.
//
// Colors:
//   * Snake head: #0b6ad8 (dark blue)
//   * Snake body: #4aa3ff (lighter blue)
//   * Apple:      #e24a4a (red)
//   * Background: deep navy, 1px grid lines

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gio::ApplicationFlags;
use glib::ControlFlow;
use gtk4::cairo;
use gtk4::gdk;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, DrawingArea,
    EventControllerKey, Label, Orientation,
};

/// GApplication ID.
const APP_ID: &str = "org.astraos.Snake";

/// Grid dimensions: 20 columns × 20 rows.
const GRID_SIZE: usize = 20;

/// Pixel size of each cell. 20 × 18 = 360px canvas.
const CELL_PX: f64 = 18.0;

/// Tick interval: 120ms between snake movements.
const TICK_MS: u64 = 120;

/// System path for the installed stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/snake.css";

/// Minimal embedded stylesheet (used when the system theme is missing).
const FALLBACK_CSS: &str = r#"
window.astra-snake {
    background: rgba(11, 13, 23, 0.96);
    color: #f1f5f9;
    font-family: 'Outfit', 'Inter', 'Cantarell', sans-serif;
}
.snake-score-bar {
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    padding: 8px 14px;
    margin: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
}
.snake-score {
    font-size: 13pt;
    font-weight: 700;
    color: #818cf8;
}
.snake-reset {
    font-weight: 700;
    border-radius: 8px;
    padding: 6px 14px;
    background: rgba(129, 140, 248, 0.20);
    border: 1px solid rgba(129, 140, 248, 0.35);
    color: #f1f5f9;
}
.snake-reset:hover {
    background: rgba(129, 140, 248, 0.35);
}
.snake-canvas {
    border-radius: 8px;
    margin: 0 8px 8px;
}
"#;

// =============================================================================
// RNG: xorshift64 seeded from SystemTime (no external `rand` dep)
// =============================================================================

fn xorshift_next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

// =============================================================================
// Direction + game state
// =============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Cell offset (dx, dy) for one move in this direction.
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// True if `other` is the opposite direction of `self` (a 180° turn
    /// which would cause instant self-collision).
    fn is_opposite(self, other: Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}

struct GameState {
    /// Snake segments, head first.
    snake: Vec<(i32, i32)>,
    /// Current committed direction (drives the next tick).
    direction: Direction,
    /// Queued direction (committed at the start of the next tick).
    /// Prevents the player from queuing an instant 180° turn.
    next_direction: Direction,
    /// Apple cell.
    apple: (i32, i32),
    /// Player score (= number of apples eaten).
    score: u32,
    /// True when the snake has hit a wall or itself.
    game_over: bool,
    /// xorshift64 state for apple placement.
    rng_state: u64,
}

impl GameState {
    fn new() -> Self {
        let mut s = Self {
            snake: vec![(10, 10), (9, 10), (8, 10)],
            direction: Direction::Right,
            next_direction: Direction::Right,
            apple: (15, 10),
            score: 0,
            game_over: false,
            rng_state: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0xC0FFEE),
        };
        s.place_apple();
        s
    }

    /// Queue a direction change (ignored if it would be a 180° turn
    /// or if the game is already over).
    fn set_direction(&mut self, dir: Direction) {
        if self.game_over {
            return;
        }
        if self.direction.is_opposite(dir) {
            return;
        }
        self.next_direction = dir;
    }

    /// Advance the game by one tick. Commits `next_direction`, computes
    /// the new head position, checks wall + self collision, and either
    /// eats the apple (grow + new apple) or trims the tail (move).
    fn tick(&mut self) {
        if self.game_over {
            return;
        }
        self.direction = self.next_direction;
        let (dx, dy) = self.direction.delta();
        let (head_x, head_y) = self.snake[0];
        let new_head = (head_x + dx, head_y + dy);

        // Wall collision.
        if new_head.0 < 0
            || new_head.0 >= GRID_SIZE as i32
            || new_head.1 < 0
            || new_head.1 >= GRID_SIZE as i32
        {
            self.game_over = true;
            return;
        }

        // Self collision. If the snake is about to eat the apple, it
        // will grow — so the tail does not move this tick and every
        // body cell counts as a collision target. Otherwise the tail
        // moves out of the way, so the last cell is a free slot.
        let eating = new_head == self.apple;
        let body_check_end = if eating {
            self.snake.len()
        } else {
            self.snake.len() - 1
        };
        if self.snake[..body_check_end].contains(&new_head) {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);
        if eating {
            self.score += 1;
            self.place_apple();
        } else {
            self.snake.pop();
        }
    }

    /// Place a new apple on a random cell not currently occupied by
    /// the snake. Always terminates because the snake can never fill
    /// the entire board (max length 20*20=400, but we always have at
    /// least one free cell while there's room).
    fn place_apple(&mut self) {
        let max_attempts = GRID_SIZE * GRID_SIZE * 4;
        for _ in 0..max_attempts {
            let x = (xorshift_next(&mut self.rng_state) as usize) % GRID_SIZE;
            let y = (xorshift_next(&mut self.rng_state) as usize) % GRID_SIZE;
            let pos = (x as i32, y as i32);
            if !self.snake.contains(&pos) {
                self.apple = pos;
                return;
            }
        }
        // Board is essentially full — leave apple where it is.
    }
}

// =============================================================================
// Drawing
// =============================================================================

/// Draw the snake, apple, grid, and (if game over) overlay.
fn draw_game(state: &GameState, cr: &cairo::Context) {
    // Background fill — deep navy.
    let _ = cr.set_source_rgba(0.043, 0.051, 0.091, 1.0);
    let _ = cr.paint();

    // Subtle grid lines.
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.04);
    cr.set_line_width(1.0);
    for i in 0..=GRID_SIZE {
        let p = i as f64 * CELL_PX;
        let _ = cr.move_to(p, 0.0);
        let _ = cr.line_to(p, GRID_SIZE as f64 * CELL_PX);
        let _ = cr.move_to(0.0, p);
        let _ = cr.line_to(GRID_SIZE as f64 * CELL_PX, p);
    }
    let _ = cr.stroke();

    // Apple — red.
    cr.set_source_rgb(0.886, 0.290, 0.290); // #e24a4a
    cr.rectangle(
        state.apple.0 as f64 * CELL_PX + 1.0,
        state.apple.1 as f64 * CELL_PX + 1.0,
        CELL_PX - 2.0,
        CELL_PX - 2.0,
    );
    let _ = cr.fill();

    // Snake — head dark blue, body lighter.
    for (i, (x, y)) in state.snake.iter().enumerate() {
        if i == 0 {
            cr.set_source_rgb(0.043, 0.416, 0.847); // #0b6ad8 (head)
        } else {
            cr.set_source_rgb(0.290, 0.639, 1.0); // #4aa3ff (body)
        }
        cr.rectangle(
            *x as f64 * CELL_PX + 1.0,
            *y as f64 * CELL_PX + 1.0,
            CELL_PX - 2.0,
            CELL_PX - 2.0,
        );
        let _ = cr.fill();
    }

    // Game-over overlay — translucent dark + text.
    if state.game_over {
        let _ = cr.set_source_rgba(0.0, 0.0, 0.0, 0.65);
        let _ = cr.paint();

        cr.set_source_rgb(1.0, 1.0, 1.0);
        cr.set_font_size(28.0);
        let _ = cr.move_to(
            (GRID_SIZE as f64 * CELL_PX) / 2.0 - 80.0,
            (GRID_SIZE as f64 * CELL_PX) / 2.0 - 10.0,
        );
        let _ = cr.show_text("Game Over");
        cr.set_font_size(14.0);
        let _ = cr.move_to(
            (GRID_SIZE as f64 * CELL_PX) / 2.0 - 100.0,
            (GRID_SIZE as f64 * CELL_PX) / 2.0 + 18.0,
        );
        let _ = cr.show_text("Press Reset to play again");
    }
}

// =============================================================================
// Theme
// =============================================================================

fn load_theme() {
    let provider = CssProvider::new();
    let css = match std::fs::read_to_string(THEME_PATH) {
        Ok(c) => {
            log::info!("Loaded snake theme from {}", THEME_PATH);
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
        .title("Astra Snake")
        .default_width(420)
        .default_height(460)
        .build();
    window.add_css_class("astra-snake");

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    window.set_child(Some(&root));

    // --- Score bar -------------------------------------------------------
    let score_bar = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .css_classes(["snake-score-bar"])
        .build();

    let score_label = Label::new(Some("Score: 0"));
    score_label.add_css_class("snake-score");

    let reset_button = Button::with_label("Reset");
    reset_button.add_css_class("snake-reset");

    let hint_label = Label::new(Some("Arrows or WASD to steer"));
    hint_label.add_css_class("snake-score");

    score_bar.append(&score_label);
    score_bar.append(&reset_button);
    score_bar.append(&hint_label);
    root.append(&score_bar);

    // --- Canvas (DrawingArea with cairo draw function) -------------------
    let canvas = DrawingArea::builder()
        .css_classes(["snake-canvas"])
        .content_width((GRID_SIZE as f64 * CELL_PX) as i32)
        .content_height((GRID_SIZE as f64 * CELL_PX) as i32)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .can_focus(true)
        .focusable(true)
        .build();
    root.append(&canvas);

    // Shared game state.
    let state: Rc<RefCell<GameState>> = Rc::new(RefCell::new(GameState::new()));

    // Draw function closure — clones the Rc and borrows the state on each
    // paint. Reads only, so a plain `borrow()` is safe.
    let state_for_draw = state.clone();
    canvas.set_draw_func(move |_area, cr, _w, _h| {
        let s = state_for_draw.borrow();
        draw_game(&s, cr);
    });

    // Keyboard control via EventControllerKey attached to the canvas.
    let state_for_keys = state.clone();
    let canvas_for_focus = canvas.clone();
    let key_controller = EventControllerKey::new();
    canvas.add_controller(key_controller.clone());
    key_controller.connect_key_pressed(move |_, keyval, _keycode, _state| {
        // Map arrow keys + WASD to directions. Spacebar resets after game over.
        let dir: Option<Direction> =
            if keyval == gdk::Key::Up || keyval == gdk::Key::w || keyval == gdk::Key::W {
                Some(Direction::Up)
            } else if keyval == gdk::Key::Down || keyval == gdk::Key::s || keyval == gdk::Key::S {
                Some(Direction::Down)
            } else if keyval == gdk::Key::Left || keyval == gdk::Key::a || keyval == gdk::Key::A {
                Some(Direction::Left)
            } else if keyval == gdk::Key::Right || keyval == gdk::Key::d || keyval == gdk::Key::D {
                Some(Direction::Right)
            } else {
                None
            };

        if let Some(d) = dir {
            state_for_keys.borrow_mut().set_direction(d);
            glib::Propagation::Stop
        } else if keyval == gdk::Key::space {
            // Spacebar restarts the game (useful after game over).
            let mut s = state_for_keys.borrow_mut();
            *s = GameState::new();
            let _ = &canvas_for_focus; // keep clone alive in this closure
            canvas_for_focus.queue_draw();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });

    // --- Game tick: 120ms timeout ----------------------------------------
    let state_for_tick = state.clone();
    let canvas_for_tick = canvas.clone();
    let score_label_for_tick = score_label.clone();
    glib::timeout_add_local(Duration::from_millis(TICK_MS), move || {
        {
            let mut s = state_for_tick.borrow_mut();
            s.tick();
        }
        // Update score label + redraw.
        let s = state_for_tick.borrow();
        score_label_for_tick.set_label(&format!("Score: {}", s.score));
        drop(s);
        canvas_for_tick.queue_draw();
        ControlFlow::Continue
    });

    // --- Reset button ----------------------------------------------------
    {
        let state = state.clone();
        let canvas = canvas.clone();
        let score_label = score_label.clone();
        reset_button.connect_clicked(move |_| {
            *state.borrow_mut() = GameState::new();
            score_label.set_label("Score: 0");
            canvas.queue_draw();
            // Refocus canvas so arrow keys keep working after click.
            let _ = canvas.grab_focus();
        });
    }

    window.present();
    // Steal keyboard focus into the canvas so arrow keys work immediately.
    let _ = canvas.grab_focus();
}

fn main() {
    env_logger::init();
    log::info!("Astra Snake v0.4.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);
    app.run();
}
