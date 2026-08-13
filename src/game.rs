use rand::{RngExt, rngs::ThreadRng};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    layout::{Offset, Position, Rect, Size},
    widgets::Widget,
};

use crate::{
    app::Scene,
    main_widget::{ClickResultKind, MainWidget},
    right_widget::{RightWidget, Submission, SubmissionKind},
    top_widget::TopWidget,
};

const _: () = {
    assert!(MainWidget::WIDTH + WIDGET_SPACING + RightWidget::WIDTH == TopWidget::WIDTH);
};
const WIDGET_SPACING: u16 = 1;
pub const TOTAL_WIDTH: u16 = MainWidget::WIDTH + WIDGET_SPACING + RightWidget::WIDTH;
pub const TOTAL_HEIGHT: u16 = MainWidget::HEIGHT + WIDGET_SPACING + TopWidget::HEIGHT;

const TOP_POS: Position = Position::new(0, 0);
const MAIN_POS: Position = Position {
    x: TOP_POS.x,
    y: TOP_POS.y + TopWidget::HEIGHT + WIDGET_SPACING,
};
const RIGHT_POS: Position = Position {
    x: MAIN_POS.x + MainWidget::WIDTH + WIDGET_SPACING,
    y: MAIN_POS.y,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    Terminated,
    LockedOut,
    Hacked,
}

// TODO: implement difficulty options
#[derive(Debug)]
pub struct Game {
    should_exit: bool,
    rng: ThreadRng,
    top_widget: TopWidget,
    right_widget: RightWidget,
    main_widget: MainWidget,
    main_widget_clickable: bool,
    result: GameResult,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    const SIZE: Size = Size::new(TOTAL_WIDTH, TOTAL_HEIGHT);

    pub fn new() -> Self {
        let mut rng = rand::rng();
        let top_widget = TopWidget::default();
        let main_widget = MainWidget::new_random(&mut rng);
        let mut right_widget = RightWidget::default();
        right_widget.set_selected_string(main_widget.get_highlighted_string());

        Self {
            should_exit: false,
            rng,
            top_widget,
            right_widget,
            main_widget,
            main_widget_clickable: false,
            result: GameResult::Terminated,
        }
    }

    fn submit_element_under_cursor(&mut self) {
        let click_result = self.main_widget.click();
        let submission = match click_result.kind {
            ClickResultKind::Error => Submission {
                string: click_result.string,
                kind: SubmissionKind::Error,
            },
            ClickResultKind::Word { likeness } => {
                let lockout = self.top_widget.remove_attempt();
                if lockout {
                    self.result = GameResult::LockedOut;
                }
                Submission {
                    string: click_result.string,
                    kind: SubmissionKind::Word { likeness, lockout },
                }
            }
            ClickResultKind::Pair => {
                // RESEARCH: What's the correct chance
                // RESET_TRIES_CHANCE = (numerator, denominator)
                const RESET_TRIES_CHANCE: (u32, u32) = (1, 10);
                if self
                    .rng
                    .random_ratio(RESET_TRIES_CHANCE.0, RESET_TRIES_CHANCE.1)
                {
                    self.top_widget.reset_attempts();
                    Submission {
                        kind: SubmissionKind::AttemptsReset,
                        string: click_result.string,
                    }
                } else {
                    self.main_widget.remove_dud(&mut self.rng);
                    Submission {
                        kind: SubmissionKind::DudRemoved,
                        string: click_result.string,
                    }
                }
            }
            ClickResultKind::Solution => {
                self.exit(GameResult::Hacked);
                return;
            }
        };
        self.right_widget.add_to_history(&submission);
    }

    fn exit(&mut self, game_result: GameResult) {
        self.result = game_result;
        self.should_exit = true;
    }

    pub fn should_exit(&self) -> Option<GameResult> {
        self.should_exit.then_some(self.result)
    }
}

impl Scene for Game {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Tab {
            self.exit(GameResult::Terminated);
            return;
        }

        if self.top_widget.locked_out() {
            return;
        }

        // Submit selected
        if KeyCode::Char('e') == key_event.code {
            self.submit_element_under_cursor();
            return;
        }

        // Cursor movement
        let cursor_moved = match key_event.code {
            KeyCode::Up | KeyCode::Char('w') => {
                self.main_widget.move_cursor_up();
                true
            }
            KeyCode::Down | KeyCode::Char('s') => {
                self.main_widget.move_cursor_down();
                true
            }
            KeyCode::Left | KeyCode::Char('a') => {
                self.main_widget.move_cursor_left();
                true
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.main_widget.move_cursor_right();
                true
            }
            _ => false,
        };
        if cursor_moved {
            self.main_widget_clickable = false;
            self.right_widget
                .set_selected_string(self.main_widget.get_highlighted_string());
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        if self.top_widget.locked_out() {
            return;
        }
        match mouse_event.kind {
            MouseEventKind::Moved => {
                let main_area = Rect::new(
                    MAIN_POS.x,
                    MAIN_POS.y,
                    MainWidget::WIDTH,
                    MainWidget::HEIGHT,
                );

                let column = mouse_event.column;
                let row = mouse_event.row;
                if !main_area.contains(Position::new(column, row)) {
                    self.main_widget_clickable = false;
                    return;
                }
                self.main_widget_clickable = self
                    .main_widget
                    .move_cursor(Position::new(column - main_area.x, row - main_area.y));
                self.right_widget
                    .set_selected_string(self.main_widget.get_highlighted_string());
            }
            MouseEventKind::Down(mouse_button) => match mouse_button {
                MouseButton::Left | MouseButton::Right => {
                    if self.main_widget_clickable {
                        self.submit_element_under_cursor();
                    }
                }
                MouseButton::Middle => (),
            },
            _ => (),
        }
    }
}

impl Widget for &Game {
    fn render(self, area: Rect, buf: &mut Buffer) {
        assert_eq!(area.as_size(), Game::SIZE);

        const fn p2o(position: Position) -> Offset {
            Offset::new(position.x as i32, position.y as i32)
        }
        let top_area = area.resize(TopWidget::SIZE).offset(p2o(TOP_POS));
        let main_area = area.resize(MainWidget::SIZE).offset(p2o(MAIN_POS));
        let right_area = area.resize(RightWidget::SIZE).offset(p2o(RIGHT_POS));

        self.top_widget.render(top_area, buf);
        self.main_widget.render(main_area, buf);
        self.right_widget.render(right_area, buf);
    }
}
