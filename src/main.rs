use std::io::stdout;

use crate::{app::App, game::GameResult};
use anyhow::Result;
use ratatui::crossterm::{
    ExecutableCommand as _,
    event::{DisableMouseCapture, EnableMouseCapture},
};

mod app;
mod game;
mod main_widget;
mod menu;
mod right_widget;
mod top_widget;
mod wordlists;

// TODO: big refactor
// - refactor modules into a hierarchy (e.g. game::top_widget)
// - move smaller structs and enums (e.g. Difficulty) into their own modules

// TODO: Refactor headline to be global to the terminal and reused across menu and game.

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    stdout().execute(EnableMouseCapture)?;

    let app_result = App::default().run(&mut terminal);

    stdout().execute(DisableMouseCapture)?;
    ratatui::restore();

    let game_result = app_result?;
    match game_result {
        GameResult::Terminated => (),
        GameResult::LockedOut => println!("You've been locked out of the terminal."),
        GameResult::Hacked => println!("You successfully hacked the terminal."),
    }

    Ok(())
}

impl Drop for App {
    fn drop(&mut self) {
        stdout().execute(DisableMouseCapture).unwrap();
    }
}
