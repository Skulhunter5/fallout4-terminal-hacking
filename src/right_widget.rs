use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect, Size},
    widgets::Widget,
};

use crate::main_widget::{ClickResult, ClickResultKind};

#[derive(Debug, Default)]
pub struct RightWidget {
    selected_string: String,
    history: Vec<String>,
}

impl RightWidget {
    pub fn set_selected_string(&mut self, string: String) {
        self.selected_string = string;
        self.selected_string.insert(0, '>');
    }

    pub fn add_to_history(&mut self, click_result: &ClickResult) {
        self.history.push(format!(">{}", click_result.string));
        match click_result.kind {
            ClickResultKind::Char => self.history.push(">Error".to_owned()),
            ClickResultKind::Word { likeness } => {
                self.history.push(">Entry denied.".to_owned());
                self.history.push(format!(">Likeness={}", likeness));
            }
            ClickResultKind::Solution => todo!(),
        };
    }
}

impl Widget for &RightWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cmd_area = area
            .resize(Size::new(area.width, 1))
            .offset(Offset::new(0, area.height as i32 - 1));
        self.selected_string.as_str().render(cmd_area, buf);
        // TODO: render blinking cursor after selected string line
        // - block cursor. same size as for remaining attempts
        // - speed unclear. somewhere between 2 and 4 times per second

        let history_area = area.resize(Size::new(area.width, area.height - 1));
        if history_area.height > 0 && !self.history.is_empty() {
            let mut i = self.history.len() - 1;
            let mut remaining_line: &str = &self.history[i];
            for row in history_area.rows().rev() {
                let line_length = match remaining_line.len() % row.width as usize {
                    0 => row.width as usize,
                    rem => rem,
                };
                let (left, right) = remaining_line.split_at(remaining_line.len() - line_length);
                remaining_line = left;
                right.render(row, buf);

                if remaining_line.is_empty() {
                    if i == 0 {
                        break;
                    }
                    i -= 1;
                    remaining_line = &self.history[i];
                }
            }
        }
    }
}
