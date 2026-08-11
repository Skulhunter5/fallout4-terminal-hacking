use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect, Size},
    widgets::Widget,
};

// TODO: add history
#[derive(Debug, Default)]
pub struct RightWidget {
    selected_string: String,
}

impl RightWidget {
    pub fn set_selected_string(&mut self, string: String) {
        self.selected_string = string;
    }
}

impl Widget for &RightWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: render history
        let cmd_area = area
            .resize(Size::new(area.width, 1))
            .offset(Offset::new(0, area.height as i32 - 1));
        format!(">{}", self.selected_string).render(cmd_area, buf);
    }
}
