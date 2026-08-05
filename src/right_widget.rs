use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[derive(Debug)]
pub struct RightWidget;

impl Default for RightWidget {
    fn default() -> Self {
        Self
    }
}

impl Widget for &RightWidget {
    fn render(self, _area: Rect, _buf: &mut Buffer) {
        // TODO
    }
}
