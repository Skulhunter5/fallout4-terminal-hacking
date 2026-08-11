use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

#[derive(Debug)]
pub struct TopWidget {
    head_line: String,
    remaining_attempts: usize,
}

impl TopWidget {
    pub fn remove_attempt(&mut self) -> bool {
        if self.remaining_attempts > 0 {
            self.remaining_attempts -= 1;
        }
        self.remaining_attempts == 0
    }
}

impl Default for TopWidget {
    fn default() -> Self {
        Self {
            head_line: "Welcome to ROBCO Industries (TM) Termlink".to_owned(),
            remaining_attempts: 4,
        }
    }
}

impl Widget for &TopWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = area.rows().collect::<Vec<Rect>>();
        assert_eq!(lines.len(), 4);

        self.head_line.as_str().render(lines[0], buf);
        "Password Required".render(lines[1], buf);
        let attempts_string = std::iter::repeat_n("■", self.remaining_attempts)
            .collect::<Vec<_>>()
            .join(" ");
        format!("Attempts Remaining: {}", attempts_string).render(lines[3], buf);
    }
}
