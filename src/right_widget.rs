use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect, Size},
    text::{Line, Text},
    widgets::{Paragraph, Widget, Wrap},
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

fn wrapped_line_count(text: &Text, width: u16) -> usize {
    if width == 0 {
        return 0;
    }

    let height = text
        .iter()
        .map(|line| line.width().max(1))
        .sum::<usize>()
        .min(u16::MAX as usize) as u16;
    let area = Rect::new(0, 0, width, height);
    let mut tmp = Buffer::empty(area);
    Paragraph::new(text.clone())
        .wrap(Wrap { trim: false })
        .render(area, &mut tmp);
    tmp.content
        .chunks(width as usize)
        .filter(|row| row.iter().any(|cell| cell.symbol() != " "))
        .count()
}

impl Widget for &RightWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let cmd_area = area
            .resize(Size::new(area.width, 1))
            .offset(Offset::new(0, area.height as i32 - 1));
        self.selected_string.as_str().render(cmd_area, buf);

        if area.height <= 1 {
            return;
        }
        let history_area = area.resize(Size::new(area.width, area.height - 1));

        let text = self
            .history
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect::<Text>();
        let total_lines = wrapped_line_count(&text, history_area.width);

        // Push short histories to the bottom so the newest entry sits right above the prompt.
        let padding = history_area.height.saturating_sub(total_lines as u16) as usize;
        let mut lines = vec![Line::default(); padding];
        lines.extend(self.history.iter().map(|line| Line::from(line.as_str())));
        let text = Text::from(lines);

        let scroll = total_lines.saturating_sub(history_area.height as usize);
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .scroll((scroll as u16, 0))
            .render(history_area, buf);
    }
}
