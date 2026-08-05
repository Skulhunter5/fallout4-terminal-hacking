use crate::app::App;
use anyhow::Result;

mod app;
mod main_widget;
mod right_widget;
mod top_widget;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result.map_err(|e| e.into())
}
