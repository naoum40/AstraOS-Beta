// main.rs - AstraOS Tic-Tac-Toe.
//
// Classic 3x3 Tic-Tac-Toe for two players (X and O), built with
// GTK4-rs. X always starts; turns alternate. Win detection covers
// all 8 winning lines (3 rows + 3 cols + 2 diagonals). Draw is
// detected when every cell is filled with no winner.
//
// UI:
//   * Header row: X wins label, reset button, O wins label, draws label
//   * 3x3 grid of 80x80 buttons
//   * Each click on an empty cell places the current player's mark
//   * Reset button clears the board (keeps scores across rounds)
//   * X marks are blue (#0b6ad8), O marks are red (#ff2d55)
//
// State (board, current player, scores) is held in `Rc<RefCell<App>>`
// and shared across button callbacks.

use std::cell::RefCell;
use std::rc::Rc;

use gio::ApplicationFlags;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Box as GtkBox, Button, CssProvider, Grid, Label, Orientation,
};

/// GApplication ID.
const APP_ID: &str = "org.astraos.TicTacToe";

/// System path for the installed stylesheet.
const THEME_PATH: &str = "/usr/share/astraos/themes/tictactoe.css";

/// Minimal embedded stylesheet (used when the system theme is missing).
const FALLBACK_CSS: &str = r#"
window.astra-tictactoe {
    background: rgba(11, 13, 23, 0.96);
    color: #f1f5f9;
    font-family: 'Outfit', 'Inter', 'Cantarell', sans-serif;
}
.ttt-header {
    background: rgba(255, 255, 255, 0.04);
    border-radius: 10px;
    padding: 8px 14px;
    margin: 8px;
    border: 1px solid rgba(255, 255, 255, 0.06);
}
.ttt-score {
    font-size: 13pt;
    font-weight: 700;
    min-width: 70px;
}
.ttt-score.x { color: #0b6ad8; }
.ttt-score.o { color: #ff2d55; }
.ttt-score.draw { color: #94a3b8; }
.ttt-reset {
    font-weight: 700;
    border-radius: 8px;
    padding: 6px 14px;
    background: rgba(129, 140, 248, 0.20);
    border: 1px solid rgba(129, 140, 248, 0.35);
}
.ttt-reset:hover {
    background: rgba(129, 140, 248, 0.35);
}
.ttt-cell {
    min-width: 80px;
    min-height: 80px;
    font-size: 24pt;
    font-weight: 700;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    color: #f1f5f9;
    padding: 0;
}
.ttt-cell:hover {
    background: rgba(129, 140, 248, 0.18);
}
.ttt-cell.x { color: #0b6ad8; }
.ttt-cell.o { color: #ff2d55; }
.ttt-cell.winner {
    background: rgba(129, 140, 248, 0.35);
    border-color: #818cf8;
}
.ttt-status {
    color: #cbd5e1;
    font-size: 11pt;
    padding: 4px 0 8px;
}
"#;

// =============================================================================
// Game state
// =============================================================================

#[derive(Clone, Copy, PartialEq, Debug)]
enum Player {
    X,
    O,
}

impl Player {
    fn label(self) -> &'static str {
        match self {
            Player::X => "X",
            Player::O => "O",
        }
    }

    fn css(self) -> &'static str {
        match self {
            Player::X => "x",
            Player::O => "o",
        }
    }

    fn other(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

struct AppState {
    /// 3x3 board. `None` = empty.
    board: [[Option<Player>; 3]; 3],
    /// Whose turn it is (X starts).
    current: Player,
    /// Winning line: `Some([(r, c); 3])` when game is won.
    winner_line: Option<[(usize, usize); 3]>,
    /// True when game is over (win or draw).
    game_over: bool,
    /// X win count.
    score_x: u32,
    /// O win count.
    score_o: u32,
    /// Draw count.
    score_draw: u32,
}

impl AppState {
    fn new() -> Self {
        Self {
            board: [[None; 3]; 3],
            current: Player::X,
            winner_line: None,
            game_over: false,
            score_x: 0,
            score_o: 0,
            score_draw: 0,
        }
    }

    /// Reset the board to empty but keep scores. X always starts.
    fn reset_board(&mut self) {
        self.board = [[None; 3]; 3];
        self.current = Player::X;
        self.winner_line = None;
        self.game_over = false;
    }

    /// Place `current` at `(r, c)` if empty, then check win/draw and
    /// flip turn (if game still going).
    fn play(&mut self, r: usize, c: usize) {
        if self.game_over {
            return;
        }
        if r >= 3 || c >= 3 {
            return;
        }
        if self.board[r][c].is_some() {
            return;
        }
        self.board[r][c] = Some(self.current);

        if let Some(line) = self.check_winner() {
            self.winner_line = Some(line);
            self.game_over = true;
            match self.current {
                Player::X => self.score_x += 1,
                Player::O => self.score_o += 1,
            }
            return;
        }

        // Draw = no winner and all 9 cells filled.
        if self.board.iter().flatten().all(|c| c.is_some()) {
            self.game_over = true;
            self.score_draw += 1;
            return;
        }

        self.current = self.current.other();
    }

    /// Walk the 8 winning lines; return the first one fully owned by a
    /// single player (or None).
    fn check_winner(&self) -> Option<[(usize, usize); 3]> {
        const LINES: &[[(usize, usize); 3]] = &[
            // Rows
            [(0, 0), (0, 1), (0, 2)],
            [(1, 0), (1, 1), (1, 2)],
            [(2, 0), (2, 1), (2, 2)],
            // Columns
            [(0, 0), (1, 0), (2, 0)],
            [(0, 1), (1, 1), (2, 1)],
            [(0, 2), (1, 2), (2, 2)],
            // Diagonals
            [(0, 0), (1, 1), (2, 2)],
            [(0, 2), (1, 1), (2, 0)],
        ];
        for line in LINES {
            let a = self.board[line[0].0][line[0].1];
            let b = self.board[line[1].0][line[1].1];
            let c = self.board[line[2].0][line[2].1];
            if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                if a == b && b == c {
                    return Some(*line);
                }
            }
        }
        None
    }

    /// Status string shown at the bottom of the window.
    fn status(&self) -> String {
        if let Some(line) = self.winner_line {
            let winner = self.board[line[0].0][line[0].1].unwrap();
            return format!("{} wins! Click Reset for a new round.", winner.label());
        }
        if self.game_over {
            return "Draw! Click Reset for a new round.".to_string();
        }
        format!("{}'s turn", self.current.label())
    }
}

// =============================================================================
// App (game + UI references)
// =============================================================================

struct App {
    state: AppState,
    cells: [[Button; 3]; 3],
    score_x_label: Label,
    score_o_label: Label,
    score_draw_label: Label,
    status_label: Label,
}

impl App {
    /// Re-render every cell + header from the current game state.
    fn render(&self) {
        for r in 0..3 {
            for c in 0..3 {
                let btn = &self.cells[r][c];
                btn.remove_css_class("x");
                btn.remove_css_class("o");
                btn.remove_css_class("winner");
                btn.set_label("");
                btn.set_sensitive(true);

                if let Some(player) = self.state.board[r][c] {
                    btn.set_label(player.label());
                    btn.add_css_class(player.css());
                    // Disable filled cells.
                    btn.set_sensitive(false);
                }

                // Highlight winning line.
                if let Some(line) = self.state.winner_line {
                    if line.contains(&(r, c)) {
                        btn.add_css_class("winner");
                    }
                }

                // When game is over, lock all cells.
                if self.state.game_over {
                    btn.set_sensitive(false);
                }
            }
        }

        self.score_x_label
            .set_label(&format!("X: {}", self.state.score_x));
        self.score_o_label
            .set_label(&format!("O: {}", self.state.score_o));
        self.score_draw_label
            .set_label(&format!("Draws: {}", self.state.score_draw));
        self.status_label.set_label(&self.state.status());
    }
}

// =============================================================================
// Theme
// =============================================================================

fn load_theme() {
    let provider = CssProvider::new();
    let css = match std::fs::read_to_string(THEME_PATH) {
        Ok(c) => {
            log::info!("Loaded tictactoe theme from {}", THEME_PATH);
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
        .title("Astra Tic-Tac-Toe")
        .default_width(360)
        .default_height(420)
        .build();
    window.add_css_class("astra-tictactoe");

    let root = GtkBox::builder()
        .orientation(Orientation::Vertical)
        .spacing(0)
        .build();
    window.set_child(Some(&root));

    // --- Header: X score | reset | O score | draws -----------------------
    let header = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .halign(gtk4::Align::Center)
        .css_classes(["ttt-header"])
        .build();

    let score_x_label = Label::new(Some("X: 0"));
    score_x_label.add_css_class("ttt-score");
    score_x_label.add_css_class("x");

    let reset_button = Button::with_label("Reset");
    reset_button.add_css_class("ttt-reset");

    let score_o_label = Label::new(Some("O: 0"));
    score_o_label.add_css_class("ttt-score");
    score_o_label.add_css_class("o");

    let score_draw_label = Label::new(Some("Draws: 0"));
    score_draw_label.add_css_class("ttt-score");
    score_draw_label.add_css_class("draw");

    header.append(&score_x_label);
    header.append(&reset_button);
    header.append(&score_o_label);
    header.append(&score_draw_label);
    root.append(&header);

    // --- 3x3 grid of 80x80 buttons ---------------------------------------
    let grid = Grid::builder()
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .row_spacing(6)
        .column_spacing(6)
        .margin_top(8)
        .margin_bottom(8)
        .build();
    root.append(&grid);

    // Build cells.
    let cells: [[Button; 3]; 3] = [
        [Button::new(), Button::new(), Button::new()],
        [Button::new(), Button::new(), Button::new()],
        [Button::new(), Button::new(), Button::new()],
    ];
    for r in 0..3 {
        for c in 0..3 {
            let btn = cells[r][c].clone();
            btn.add_css_class("ttt-cell");
            grid.attach(&btn, c as i32, r as i32, 1, 1);
        }
    }

    let status_label = Label::new(Some("X's turn"));
    status_label.add_css_class("ttt-status");
    root.append(&status_label);

    let state = Rc::new(RefCell::new(App {
        state: AppState::new(),
        cells,
        score_x_label,
        score_o_label,
        score_draw_label,
        status_label,
    }));

    // Wire cell click handlers.
    for r in 0..3 {
        for c in 0..3 {
            let btn = state.borrow().cells[r][c].clone();
            let st = state.clone();
            btn.connect_clicked(move |_| {
                let mut app = st.borrow_mut();
                app.state.play(r, c);
                app.render();
            });
        }
    }

    // Reset button → reset board (keep scores).
    {
        let st = state.clone();
        reset_button.connect_clicked(move |_| {
            let mut app = st.borrow_mut();
            app.state.reset_board();
            app.render();
        });
    }

    state.borrow().render();
    window.present();
}

fn main() {
    env_logger::init();
    log::info!("Astra Tic-Tac-Toe v0.4.0 starting...");

    gtk4::init().expect("Failed to initialize GTK");

    let app = Application::builder()
        .application_id(APP_ID)
        .flags(ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(build_ui);
    app.run();
}
