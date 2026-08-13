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
#[allow(unused)]
mod wordlists;

// TODO: add menu for selecting difficulty

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    stdout().execute(EnableMouseCapture)?;

    let app_result = App::default().run(&mut terminal);
    // let app_result = Game::default().run(&mut terminal);

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
