use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    layout::{Offset, Position, Rect, Size},
    style::Style,
    widgets::Widget,
};

use crate::app::{App, Scene};

const HEAD_HEIGHT: u16 = 1;
const SPACING: u16 = 1;
const HEAD_POS: Position = Position::new(0, 0);
const OPTIONS_POS: Position = Position::new(HEAD_POS.x, HEAD_POS.y + HEAD_HEIGHT + SPACING);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuResult {
    Exit,
    Selected(String),
}

#[derive(Debug)]
pub struct Menu {
    head_line: String,
    result: Option<MenuResult>,
    selected: usize,
    options: Vec<String>,
    clickable: bool,
}

impl Menu {
    pub const WIDTH: u16 = App::TERMINAL_WIDTH;
    pub const HEIGHT: u16 = App::TERMINAL_HEIGHT;

    pub fn new() -> Self {
        let options = ["Novice", "Advanced", "Expert", "Master"]
            .into_iter()
            .map(|option| option.to_owned())
            .collect();

        Self {
            head_line: "Welcome to ROBCO Industries (TM) Termlink".to_owned(),
            result: None,
            selected: 0,
            options,
            clickable: false,
        }
    }

    fn exit(&mut self, menu_result: MenuResult) {
        self.result = Some(menu_result);
    }

    pub fn should_exit(&self) -> Option<MenuResult> {
        self.result.clone()
    }

    fn set_selected(&mut self, selected: usize) -> bool {
        if self.selected == selected {
            return false;
        }
        self.selected = selected;
        true
    }

    fn submit_selection(&mut self) {
        self.exit(MenuResult::Selected(self.options[self.selected].clone()))
    }
}

impl Scene for Menu {
    fn tick(&mut self) -> bool {
        false
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Tab {
            self.exit(MenuResult::Exit);
            return;
        }

        if key_event.code == KeyCode::Char('e') {
            self.submit_selection();
            return;
        }

        // TODO: Does `clickable` depend on movement input or actual movement? I.e. does it
        // matter whether the user is trying to move up above the first option or down below the
        // last option?
        // -> There's actually seems to be no limitation for mouse clicking in menus. You can freely
        //    move the cursor up and down and still click with the mouse. It still selects the
        //    cursor though. But you do have to be within the area of the options; going above or
        //    below before clicking doesn't work.
        //    While the mouse is not over any option, the mouse can't be clicked
        match key_event.code {
            KeyCode::Up | KeyCode::Char('w') => {
                let selection_changed = self.set_selected(self.selected.saturating_sub(1));
                if selection_changed {
                    self.clickable = false;
                }
            }
            KeyCode::Down | KeyCode::Char('s') => {
                let selection_changed =
                    self.set_selected(self.options.len().min(self.selected + 1));
                if selection_changed {
                    self.clickable = false;
                }
            }
            _ => (),
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Moved => {
                let area = Rect::new(0, 0, Self::WIDTH, Self::HEIGHT);
                let options_area = area
                    .resize(Size::new(area.width, area.height - HEAD_HEIGHT))
                    .offset(Offset::new(OPTIONS_POS.x as i32, OPTIONS_POS.y as i32));

                if options_area.contains(Position::new(mouse_event.column, mouse_event.row)) {
                    let column = (mouse_event.column - options_area.x) as usize;
                    let row = (mouse_event.row - options_area.y) as usize;
                    // TODO: Is this the correct behavior or does the whole row count as
                    // selectable by the mouse?
                    // -> Yes the whole row is selectable for each option
                    if let Some(option) = self.options.get(row) {
                        let option_width = option.chars().count() + "[]".chars().count();
                        if column < option_width {
                            self.selected = row;
                            self.clickable = true;
                            return;
                        }
                    }
                }
                self.clickable = false;
            }
            MouseEventKind::Down(mouse_button) => match mouse_button {
                MouseButton::Left | MouseButton::Right => {
                    if self.clickable {
                        self.submit_selection();
                    }
                }
                MouseButton::Middle => (),
            },
            _ => (),
        }
    }
}

impl Widget for &Menu {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: improve headline
        // - include the line "Initializing..." (maybe that's for holotapes)
        // - Another example ingame: "Museum of Freedom Operations Terminal"
        //   - Maybe something like "Fallout 4 Hacking Minigame"
        let head_area = area
            .resize(Size::new(area.width, HEAD_HEIGHT))
            .offset(Offset::new(HEAD_POS.x as i32, HEAD_POS.y as i32));
        // TODO: Maybe add text line like "Choose a difficulty:" right above the options
        let options_area = area
            .resize(Size::new(area.width, area.height - head_area.height))
            .offset(Offset::new(OPTIONS_POS.x as i32, OPTIONS_POS.y as i32));

        self.head_line.as_str().render(head_area, buf);

        self.options
            .iter()
            .zip(options_area.rows())
            .enumerate()
            .for_each(|(i, (option, row_area))| {
                // TODO: highlight the whole selected row, not just the width of the text
                let line = format!("[{}]", option);
                let line_width = line.chars().count() as u16;
                if i == self.selected {
                    buf.set_style(
                        row_area.resize(Size::new(line_width, 1)),
                        Style::new().reversed(),
                    );
                }
                line.render(row_area, buf);
            });
    }
}
