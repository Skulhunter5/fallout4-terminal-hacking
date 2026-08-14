use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    layout::{Offset, Position, Rect, Size},
    style::Style,
    text::Text,
    widgets::Widget,
};

use crate::app::{App, Scene};

const HEAD_HEIGHT: u16 = 2;
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
    text: String,
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
            head_line: "Welcome to ROBCO Industries (TM) Termlink\nFallout 4 Hacking Minigame"
                .to_owned(),
            result: None,
            text: "Choose a difficulty:".to_owned(),
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

        match key_event.code {
            KeyCode::Up | KeyCode::Char('w') => self.selected = self.selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('s') => {
                self.selected = self.options.len().min(self.selected + 1)
            }
            _ => (),
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Moved => {
                let area = Rect::new(0, 0, Self::WIDTH, Self::HEIGHT);
                let options_area = area
                    .resize(Size::new(area.width, area.height - HEAD_HEIGHT - SPACING))
                    .offset(Offset::new(OPTIONS_POS.x as i32, OPTIONS_POS.y as i32));
                let text_height = self.text.lines().count();
                let options_area = options_area
                    .resize(Size::new(
                        options_area.width,
                        options_area.height - text_height as u16,
                    ))
                    .offset(Offset::new(0, text_height as i32));

                if options_area.contains(Position::new(mouse_event.column, mouse_event.row)) {
                    let row = (mouse_event.row - options_area.y) as usize;
                    if self.options.get(row).is_some() {
                        self.selected = row;
                        self.clickable = true;
                        return;
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
        let head_area = area
            .resize(Size::new(area.width, HEAD_HEIGHT))
            .offset(Offset::new(HEAD_POS.x as i32, HEAD_POS.y as i32));
        let options_area = area
            .resize(Size::new(area.width, area.height - head_area.height))
            .offset(Offset::new(OPTIONS_POS.x as i32, OPTIONS_POS.y as i32));

        Text::from(self.head_line.as_str()).render(head_area, buf);

        let text_height = self.text.lines().count();
        options_area
            .rows()
            .zip(self.text.lines())
            .for_each(|(row_area, text_line)| text_line.render(row_area, buf));
        options_area
            .rows()
            .skip(text_height)
            .zip(&self.options)
            .enumerate()
            .for_each(|(i, (row_area, option))| {
                let line = format!("[{}]", option);
                if i == self.selected {
                    buf.set_style(row_area, Style::new().reversed());
                }
                line.render(row_area, buf);
            });
    }
}
