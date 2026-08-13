use std::io::stdout;

use crate::app::App;
use anyhow::Result;
use ratatui::crossterm::{
    ExecutableCommand as _,
    event::{DisableMouseCapture, EnableMouseCapture},
};

mod app;
mod main_widget;
mod right_widget;
mod top_widget;
#[allow(unused)]
mod wordlists;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    stdout().execute(EnableMouseCapture)?;
    let app_result = App::default().run(&mut terminal);
    stdout().execute(DisableMouseCapture)?;
    ratatui::restore();
    app_result.map_err(|e| e.into())
}

impl Drop for App {
    fn drop(&mut self) {
        stdout().execute(DisableMouseCapture).unwrap();
    }
}
