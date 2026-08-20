use std::{
    io,
    time::{Duration, Instant},
};

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
    game::{self, Difficulty, Game, GameResult},
    menu::{Menu, MenuResult},
};

pub trait Scene
where
    for<'a> &'a Self: Widget,
{
    fn tick(&mut self) -> bool;

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let _ = key_event;
    }
    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        let _ = mouse_event;
    }
}

#[derive(Debug)]
enum ActiveScene {
    MainMenu(Menu),
    WordLengthMenu(Menu),
    WordCountMenu(Menu),
    Game(Game),
}

impl Scene for ActiveScene {
    fn tick(&mut self) -> bool {
        match self {
            Self::MainMenu(menu) | Self::WordLengthMenu(menu) | Self::WordCountMenu(menu) => {
                menu.tick()
            }
            Self::Game(game) => game.tick(),
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self {
            Self::MainMenu(menu) | Self::WordLengthMenu(menu) | Self::WordCountMenu(menu) => {
                menu.handle_key_event(key_event)
            }
            Self::Game(game) => game.handle_key_event(key_event),
        }
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) {
        match self {
            Self::MainMenu(menu) | Self::WordLengthMenu(menu) | Self::WordCountMenu(menu) => {
                menu.handle_mouse_event(mouse_event)
            }
            Self::Game(game) => game.handle_mouse_event(mouse_event),
        }
    }
}

impl Widget for &ActiveScene {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self {
            ActiveScene::MainMenu(menu)
            | ActiveScene::WordLengthMenu(menu)
            | ActiveScene::WordCountMenu(menu) => menu.render(area, buf),
            ActiveScene::Game(game) => game.render(area, buf),
        }
    }
}

struct MainMenu;

impl MainMenu {
    const OPTIONS_TEXT: &str = "What do you want to do?";

    const OPTION_PLAY: &str = "Play";
    const OPTION_WORD_LENGTH: &str = "Change Difficulty: Word Length";
    const OPTION_WORD_COUNT: &str = "Change Difficulty: Word Count";
    const OPTIONS: &[&str] = &[
        Self::OPTION_PLAY,
        Self::OPTION_WORD_LENGTH,
        Self::OPTION_WORD_COUNT,
    ];

    fn create() -> ActiveScene {
        let options = Self::OPTIONS
            .iter()
            .map(|option| (*option).to_owned())
            .collect::<Vec<String>>();
        let preselected = 0;
        ActiveScene::MainMenu(Menu::new(options, Self::OPTIONS_TEXT, preselected))
    }
}

struct WordLengthMenu;

impl WordLengthMenu {
    const OPTIONS_TEXT: &str = "Choose a difficulty:";
    const OPTIONS: &[&str] = &["Novice (4)", "Advanced (6)", "Expert (8)", "Master (10)"];

    const MAP: &[(&str, usize)] = &[
        (Self::OPTIONS[0], 4),
        (Self::OPTIONS[1], 6),
        (Self::OPTIONS[2], 8),
        (Self::OPTIONS[3], 10),
    ];

    fn option_to_word_length(option: &str) -> Option<usize> {
        Self::MAP
            .iter()
            .find_map(|(opt, word_length)| (*opt == option).then_some(*word_length))
    }

    fn create(word_length: usize) -> ActiveScene {
        let options = Self::OPTIONS
            .iter()
            .map(|option| (*option).to_owned())
            .collect::<Vec<String>>();
        let preselected = Self::MAP
            .iter()
            .position(|(_option, wl)| *wl == word_length)
            .unwrap();
        ActiveScene::WordLengthMenu(Menu::new(options, Self::OPTIONS_TEXT, preselected))
    }
}

struct WordCountMenu;

impl WordCountMenu {
    const OPTIONS_TEXT: &str = "Choose the number of words:";
    const OPTIONS: &[&str] = &[
        "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
    ];

    fn option_to_word_count(option: &str) -> Option<usize> {
        option.parse::<usize>().ok()
    }

    fn create(word_length: usize) -> ActiveScene {
        let options = Self::OPTIONS
            .iter()
            .map(|option| (*option).to_owned())
            .collect::<Vec<String>>();
        let preselected = Self::OPTIONS
            .iter()
            .position(|option| Self::option_to_word_count(option).unwrap() == word_length)
            .unwrap();
        ActiveScene::WordCountMenu(Menu::new(options, Self::OPTIONS_TEXT, preselected))
    }
}

// TODO: Probably rework the menu system so that all menus can just be defined with all their transitions
// and created once, as such and then be switched between in an easier way. No creating the menu you
// want to go to on every transition.
// - "Going back" (pressing TAB) could be automated behavior
// - Define all options once. Create/Instatiate everything only once and then keep it, don't
// recreate on every transition.
// - Maybe a small callback function that is called when an option is selected
//   - This can, for example, apply the selected difficulty option
// - Define parent menu (or None)

#[derive(Debug)]
pub struct App {
    should_exit: bool,
    active_scene: ActiveScene,
    areas: Option<(Option<Rect>, Rect)>,
    difficulty: Difficulty,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub const TERMINAL_WIDTH: u16 = game::TOTAL_WIDTH;
    pub const TERMINAL_HEIGHT: u16 = game::TOTAL_HEIGHT;

    pub const TICK_TIME: Duration = Duration::from_millis(100);

    pub fn new() -> Self {
        Self {
            should_exit: false,
            active_scene: MainMenu::create(),
            areas: None,
            difficulty: Difficulty::NOVICE,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<GameResult> {
        let Size {
            width: columns,
            height: rows,
        } = terminal.size()?;
        self.resize(columns, rows);
        terminal.draw(|frame| self.draw(frame))?;

        let mut last_tick = Instant::now();
        'main_loop: loop {
            if last_tick.elapsed() >= Self::TICK_TIME {
                if self.active_scene.tick() {
                    terminal.draw(|frame| self.draw(frame))?;
                }
                last_tick = Instant::now();
            }

            if crossterm::event::poll(Self::TICK_TIME.saturating_sub(last_tick.elapsed()))? {
                self.handle_event(crossterm::event::read()?);
                if let Some(result) = self.handle_scene_transition() {
                    return Ok(result);
                }
                if self.should_exit {
                    break 'main_loop;
                }
                terminal.draw(|frame| self.draw(frame))?;
            }
        }
        Ok(GameResult::Terminated)
    }

    fn handle_scene_transition(&mut self) -> Option<GameResult> {
        match &self.active_scene {
            ActiveScene::MainMenu(menu) => {
                if let Some(menu_result) = menu.should_exit() {
                    match menu_result {
                        MenuResult::Exit => self.exit(),
                        MenuResult::Selected(selected_option) => match selected_option.as_str() {
                            MainMenu::OPTION_PLAY => {
                                self.active_scene = ActiveScene::Game(Game::new(self.difficulty))
                            }
                            MainMenu::OPTION_WORD_LENGTH => {
                                self.active_scene =
                                    WordLengthMenu::create(self.difficulty.word_length)
                            }
                            MainMenu::OPTION_WORD_COUNT => {
                                self.active_scene =
                                    WordCountMenu::create(self.difficulty.word_count)
                            }
                            _ => unreachable!(),
                        },
                    }
                }
            }
            ActiveScene::WordLengthMenu(menu) => {
                if let Some(menu_result) = menu.should_exit() {
                    match menu_result {
                        MenuResult::Exit => (),
                        MenuResult::Selected(selected_option) => {
                            self.difficulty.word_length =
                                WordLengthMenu::option_to_word_length(selected_option.as_str())
                                    .unwrap()
                        }
                    }
                    self.active_scene = MainMenu::create();
                }
            }
            ActiveScene::WordCountMenu(menu) => {
                if let Some(menu_result) = menu.should_exit() {
                    match menu_result {
                        MenuResult::Exit => (),
                        MenuResult::Selected(selected_option) => {
                            self.difficulty.word_count =
                                WordCountMenu::option_to_word_count(selected_option.as_str())
                                    .unwrap()
                        }
                    }
                    self.active_scene = MainMenu::create();
                }
            }
            ActiveScene::Game(game) => {
                if let Some(game_result) = game.should_exit() {
                    match game_result {
                        GameResult::Terminated => self.active_scene = MainMenu::create(),
                        GameResult::Hacked | GameResult::LockedOut => return Some(game_result),
                    }
                }
            }
        }
        None
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
