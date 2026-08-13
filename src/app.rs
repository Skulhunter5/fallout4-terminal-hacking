use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent, MouseEvent},
    },
    layout::{Constraint, Offset, Rect, Size},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Widget},
};

use crate::{
    game::{self, Game, GameResult},
    menu::{Menu, MenuResult},
};

pub trait Scene
where
    for<'a> &'a Self: Widget,
{
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let _ = key_event;
    }
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        let _ = mouse_event;
    }
}

#[derive(Debug)]
enum ActiveScene {
    Menu(Menu),
    Game(Game),
}

impl Scene for ActiveScene {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self {
            Self::Menu(menu) => menu.handle_key_event(key_event),
            Self::Game(game) => game.handle_key_event(key_event),
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match self {
            Self::Menu(menu) => menu.handle_mouse_event(mouse_event),
            Self::Game(game) => game.handle_mouse_event(mouse_event),
        }
    }
}

impl Widget for &ActiveScene {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            ActiveScene::Menu(menu) => menu.render(area, buf),
            ActiveScene::Game(game) => game.render(area, buf),
        }
    }
}

#[derive(Debug)]
pub struct App {
    should_exit: bool,
    active_scene: ActiveScene,
    areas: Option<(Option<Rect>, Rect)>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub const TERMINAL_WIDTH: u16 = game::TOTAL_WIDTH;
    pub const TERMINAL_HEIGHT: u16 = game::TOTAL_HEIGHT;

    pub fn new() -> Self {
        let menu = Menu::new();

        Self {
            should_exit: false,
            active_scene: ActiveScene::Menu(menu),
            areas: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<GameResult> {
        let Size {
            width: columns,
            height: rows,
        } = terminal.size()?;
        self.resize(columns, rows);

        'main_loop: while !self.should_exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event(crossterm::event::read()?);

            match &self.active_scene {
                ActiveScene::Menu(menu) => {
                    if let Some(menu_result) = menu.should_exit() {
                        match menu_result {
                            MenuResult::Exit => self.exit(),
                            MenuResult::Selected(_selected_option) => {
                                // TODO: start game with the corresponding difficulty
                                self.active_scene = ActiveScene::Game(Game::default());
                                continue 'main_loop;
                            }
                        }
                    }
                }
                ActiveScene::Game(game) => {
                    if let Some(game_result) = game.should_exit() {
                        match game_result {
                            GameResult::Terminated => {
                                self.active_scene = ActiveScene::Menu(Menu::new());
                                continue 'main_loop;
                            }
                            GameResult::Hacked | GameResult::LockedOut => return Ok(game_result),
                        }
                    }
                }
            }
        }
        Ok(GameResult::Terminated)
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn resize(&mut self, columns: u16, rows: u16) {
        let area = Rect {
            x: 0,
            y: 0,
            width: columns,
            height: rows,
        };

        if area.width < Self::TERMINAL_WIDTH || area.height < Self::TERMINAL_HEIGHT {
            self.areas = None;
            return;
        }

        let (border_area, terminal_area) =
            if area.width >= Self::TERMINAL_WIDTH + 4 && area.height >= Self::TERMINAL_HEIGHT + 2 {
                let border_area = area.centered(
                    Constraint::Length(Self::TERMINAL_WIDTH + 4),
                    Constraint::Length(Self::TERMINAL_HEIGHT + 2),
                );
                let block = Block::bordered().border_type(BorderType::Rounded);
                let inner_area = block.inner(border_area);
                let terminal_area = inner_area
                    .resize(Size::new(inner_area.width - 2, inner_area.height))
                    .offset(Offset::new(1, 0));
                (Some(border_area), terminal_area)
            } else {
                (
                    None,
                    area.centered(
                        Constraint::Length(Self::TERMINAL_WIDTH),
                        Constraint::Length(Self::TERMINAL_HEIGHT),
                    ),
                )
            };

        self.areas = Some((border_area, terminal_area));
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    self.exit();
                    return;
                }
                _ => (),
            },
            Event::Resize(columns, rows) => {
                self.resize(columns, rows);
                return;
            }
            _ => (),
        }

        let Some((_, terminal_area)) = &self.areas else {
            return;
        };
        match event {
            Event::Key(key_event) => self.active_scene.handle_key_event(key_event),
            Event::Mouse(mut mouse_event) => {
                // TODO: translate mouse events to correct area
                let col_start = terminal_area.x;
                let col_end = terminal_area.x + terminal_area.width;
                let row_start = terminal_area.y;
                let row_end = terminal_area.y + terminal_area.height;
                if (col_start..col_end).contains(&mouse_event.column)
                    && (row_start..row_end).contains(&mouse_event.row)
                {
                    mouse_event.column -= col_start;
                    mouse_event.row -= row_start;
                    self.active_scene.handle_mouse_event(mouse_event);
                }
            }
            _ => (),
        }
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::new().fg(Color::Green));

        let Some((border_area, terminal_area)) = self.areas else {
            let warning_text = Text::raw(format!(
                "terminal too small\n(is {}x{}, needs {}x{})",
                area.width,
                area.height,
                App::TERMINAL_WIDTH,
                App::TERMINAL_HEIGHT
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

        self.active_scene.render(terminal_area, buf);
    }
}
