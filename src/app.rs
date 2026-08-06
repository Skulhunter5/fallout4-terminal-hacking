use std::io;

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm::{
        self,
        event::{Event, KeyCode, KeyEvent},
    },
    layout::{Constraint, Offset, Position, Rect, Size},
    text::Text,
    widgets::{Block, BorderType, Widget},
};

use crate::{main_widget::MainWidget, right_widget::RightWidget, top_widget::TopWidget};

#[derive(Debug, Default)]
pub struct App {
    should_exit: bool,
    top_widget: TopWidget,
    right_widget: RightWidget,
    main_widget: MainWidget,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
        // TODO: add logic
        // > move cursor (mouse (+ arrow keys?))
        // > click on element
        if let Event::Key(key_event) = crossterm::event::read()? {
            self.handle_key_event(key_event);
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => self.exit(),
            _ => (),
        }
    }

    fn exit(&mut self) {
        self.should_exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const TOP_WIDTH: u16 = TERMINAL_WIDTH;
        const TOP_HEIGHT: u16 = 4;
        const RIGHT_WIDTH: u16 = 1 + "Entry denied.".len() as u16;
        const RIGHT_HEIGHT: u16 = MainWidget::HEIGHT;
        const TERMINAL_WIDTH: u16 = MainWidget::WIDTH + 1 + RIGHT_WIDTH;
        const TERMINAL_HEIGHT: u16 = MainWidget::HEIGHT + 1 + TOP_HEIGHT;
        const SPACING: u16 = 1;

        const TOP_POS: Position = Position::new(0, 0);
        const TOP_SIZE: Size = Size::new(TOP_WIDTH, TOP_HEIGHT);
        const MAIN_POS: Position = Position {
            x: TOP_POS.x,
            y: TOP_POS.y + TOP_HEIGHT + SPACING,
        };
        const RIGHT_POS: Position = Position {
            x: MAIN_POS.x + MainWidget::WIDTH + SPACING,
            y: MAIN_POS.y,
        };
        const RIGHT_SIZE: Size = Size::new(RIGHT_WIDTH, RIGHT_HEIGHT);

        const fn p2o(position: Position) -> Offset {
            Offset::new(position.x as i32, position.y as i32)
        }

        if area.width < TERMINAL_WIDTH || area.height < TERMINAL_HEIGHT {
            let warning_text = Text::raw(format!(
                "terminal too small\n(is {}x{}, needs {}x{})",
                area.width, area.height, TERMINAL_WIDTH, TERMINAL_HEIGHT
            ))
            .centered();
            let warning_area =
                area.centered_vertically(Constraint::Length(warning_text.height() as u16));
            warning_text.render(warning_area, buf);
            return;
        }

        let terminal_area =
            if area.width >= TERMINAL_WIDTH + 4 && area.height >= TERMINAL_HEIGHT + 2 {
                let border_area = area.centered(
                    Constraint::Length(TERMINAL_WIDTH + 4),
                    Constraint::Length(TERMINAL_HEIGHT + 2),
                );
                let block = Block::bordered().border_type(BorderType::Rounded);
                let terminal_area = block.inner(border_area);
                block.render(border_area, buf);
                terminal_area.offset(Offset::new(1, 0))
            } else {
                area.centered(
                    Constraint::Length(TERMINAL_WIDTH),
                    Constraint::Length(TERMINAL_HEIGHT),
                )
            };

        let top_area = terminal_area.resize(TOP_SIZE).offset(p2o(TOP_POS));
        let main_area = terminal_area.resize(MainWidget::SIZE).offset(p2o(MAIN_POS));
        let right_area = terminal_area.resize(RIGHT_SIZE).offset(p2o(RIGHT_POS));
        assert_eq!(top_area.as_size(), TOP_SIZE);
        assert_eq!(main_area.as_size(), MainWidget::SIZE);
        assert_eq!(right_area.as_size(), RIGHT_SIZE);

        self.top_widget.render(top_area, buf);
        self.main_widget.render(main_area, buf);
        self.right_widget.render(right_area, buf);
    }
}
