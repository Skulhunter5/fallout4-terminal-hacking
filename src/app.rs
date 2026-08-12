use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind},
    },
    layout::{Constraint, Offset, Position, Rect, Size},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Widget},
};

use crate::{
    main_widget::{ClickResultKind, MainWidget},
    right_widget::RightWidget,
    top_widget::TopWidget,
};

const _: () = {
    assert!(MainWidget::WIDTH + WIDGET_SPACING + RightWidget::WIDTH == TopWidget::WIDTH);
};
const WIDGET_SPACING: u16 = 1;
const TERMINAL_WIDTH: u16 = MainWidget::WIDTH + WIDGET_SPACING + RightWidget::WIDTH;
const TERMINAL_HEIGHT: u16 = MainWidget::HEIGHT + WIDGET_SPACING + TopWidget::HEIGHT;

const TOP_POS: Position = Position::new(0, 0);
const MAIN_POS: Position = Position {
    x: TOP_POS.x,
    y: TOP_POS.y + TopWidget::HEIGHT + WIDGET_SPACING,
};
const RIGHT_POS: Position = Position {
    x: MAIN_POS.x + MainWidget::WIDTH + WIDGET_SPACING,
    y: MAIN_POS.y,
};

const fn p2o(position: Position) -> Offset {
    Offset::new(position.x as i32, position.y as i32)
}

#[derive(Debug)]
pub struct App {
    should_exit: bool,
    top_widget: TopWidget,
    right_widget: RightWidget,
    main_widget: MainWidget,
    widget_areas: Option<(Option<Rect>, Rect, Rect, Rect)>,
    main_widget_clickable: bool,
}

impl Default for App {
    fn default() -> Self {
        let top_widget = TopWidget::default();
        let main_widget = MainWidget::default();
        let mut right_widget = RightWidget::default();
        right_widget.set_selected_string(main_widget.get_highlighted_string());

        Self {
            should_exit: false,
            top_widget,
            right_widget,
            main_widget,
            widget_areas: None,
            main_widget_clickable: false,
        }
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let Size {
            width: columns,
            height: rows,
        } = terminal.size()?;
        self.resize(columns, rows);

        while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match crossterm::event::read()? {
            Event::Key(key_event) => self.handle_key_event(key_event),
            Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event),
            Event::Resize(columns, rows) => self.resize(columns, rows),
            _ => (),
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.exit(),
            KeyCode::Char('e') => self.submit_element_under_cursor(),
            KeyCode::Up | KeyCode::Char('w') => self.main_widget.move_cursor_up(),
            KeyCode::Down | KeyCode::Char('s') => self.main_widget.move_cursor_down(),
            KeyCode::Left | KeyCode::Char('a') => self.main_widget.move_cursor_left(),
            KeyCode::Right | KeyCode::Char('d') => self.main_widget.move_cursor_right(),
            _ => (),
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match mouse_event.kind {
            MouseEventKind::Moved => {
                let Some((_, _, main_area, _)) = &self.widget_areas else {
                    return;
                };
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

    fn submit_element_under_cursor(&mut self) {
        let click_result = self.main_widget.click();
        match click_result.kind {
            ClickResultKind::Char => (),
            ClickResultKind::Word { .. } => {
                if self.top_widget.remove_attempt() {
                    // TODO: used up all attempts; init lockout
                    // Output (right widget) after selecting last wrong word is:
                    // >WORD
                    // >Entry denied.
                    // >Init Lockout
                }
            }
            ClickResultKind::Solution => todo!(),
        }
        self.right_widget.add_to_history(&click_result);
    }

    fn resize(&mut self, columns: u16, rows: u16) {
        let area = Rect {
            x: 0,
            y: 0,
            width: columns,
            height: rows,
        };

        if area.width < TERMINAL_WIDTH || area.height < TERMINAL_HEIGHT {
            self.widget_areas = None;
            return;
        }

        let (border_area, terminal_area) =
            if area.width >= TERMINAL_WIDTH + 4 && area.height >= TERMINAL_HEIGHT + 2 {
                let border_area = area.centered(
                    Constraint::Length(TERMINAL_WIDTH + 4),
                    Constraint::Length(TERMINAL_HEIGHT + 2),
                );
                let block = Block::bordered().border_type(BorderType::Rounded);
                let terminal_area = block.inner(border_area);
                (Some(border_area), terminal_area.offset(Offset::new(1, 0)))
            } else {
                (
                    None,
                    area.centered(
                        Constraint::Length(TERMINAL_WIDTH),
                        Constraint::Length(TERMINAL_HEIGHT),
                    ),
                )
            };

        let top_area = terminal_area.resize(TopWidget::SIZE).offset(p2o(TOP_POS));
        let main_area = terminal_area.resize(MainWidget::SIZE).offset(p2o(MAIN_POS));
        let right_area = terminal_area
            .resize(RightWidget::SIZE)
            .offset(p2o(RIGHT_POS));

        self.widget_areas = Some((border_area, top_area, main_area, right_area));
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::new().fg(Color::Green));

        let Some((border_area, top_area, main_area, right_area)) = self.widget_areas else {
            let warning_text = Text::raw(format!(
                "terminal too small\n(is {}x{}, needs {}x{})",
                area.width, area.height, TERMINAL_WIDTH, TERMINAL_HEIGHT
            ))
            .centered();
            let warning_area =
                area.centered_vertically(Constraint::Length(warning_text.height() as u16));
            warning_text.render(warning_area, buf);
            return;
        };

        if let Some(border_area) = border_area {
            let block = Block::bordered().border_type(BorderType::Rounded);
            block.render(border_area, buf);
        }

        self.top_widget.render(top_area, buf);
        self.main_widget.render(main_area, buf);
        self.right_widget.render(right_area, buf);
    }
}
