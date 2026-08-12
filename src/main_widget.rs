use rand::{Rng, RngExt};
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Position, Rect, Size},
    style::Style,
    widgets::Widget,
};

use crate::wordlists;

#[derive(Debug, Clone, Copy, Default)]
struct CursorPosition {
    block: usize,
    column: usize,
    row: usize,
}

impl CursorPosition {
    fn from_index(index: usize) -> Self {
        let block = index / MainWidget::CHARACTERS_PER_BLOCK;
        let block_rem = index % MainWidget::CHARACTERS_PER_BLOCK;
        let row = block_rem / MainWidget::CHARACTERS_PER_ROW;
        let row_rem = block_rem % MainWidget::CHARACTERS_PER_ROW;
        let column = row_rem;
        Self { block, column, row }
    }

    fn next_row(&self) -> Self {
        let mut next: Self = *self;

        next.column = 0;
        next.row += 1;
        if next.row >= MainWidget::ROWS_PER_BLOCK {
            next.row = 0;
            next.block += 1;
            if next.block >= MainWidget::BLOCKS {
                panic!("trying to advance CursorPosition beyond the last character");
            }
        }

        next
    }

    fn index(&self) -> usize {
        self.block * MainWidget::CHARACTERS_PER_BLOCK
            + self.row * MainWidget::CHARACTERS_PER_ROW
            + self.column
    }

    fn row_index(&self) -> usize {
        self.block * MainWidget::ROWS_PER_BLOCK + self.row
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CursorHighlight {
    Char(usize),
    Word { start_index: usize, length: usize },
}

#[derive(Debug)]
pub struct ClickResult {
    pub kind: ClickResultKind,
    pub string: String,
}

#[derive(Debug)]
pub enum ClickResultKind {
    Char,
    Word { likeness: usize },
    Solution,
}

#[derive(Debug)]
pub struct MainWidget {
    first_offset: usize,
    content: Vec<String>,
    cursor: CursorPosition,
    highlight: CursorHighlight,
    words: Vec<(String, usize)>,
    solution: String,
}

impl Default for MainWidget {
    fn default() -> Self {
        Self::new_random()
    }
}

impl MainWidget {
    pub const SIZE: Size = Size::new(Self::WIDTH, Self::HEIGHT);
    pub const WIDTH: u16 = Self::BLOCK_WIDTH * 2 + Self::SPACING;
    pub const HEIGHT: u16 = Self::ROWS_PER_BLOCK as u16;
    const BLOCK_WIDTH: u16 = Self::OFFSET_WIDTH + Self::SPACING + Self::COLUMNS_PER_BLOCK as u16;
    const BLOCK_SIZE: Size = Size::new(Self::BLOCK_WIDTH, Self::ROWS_PER_BLOCK as u16);
    const OFFSET_WIDTH: u16 = 6;

    const SPACING: u16 = 1;

    pub const ROWS_PER_BLOCK: usize = 17;
    pub const COLUMNS_PER_BLOCK: usize = 12;
    pub const BLOCKS: usize = 2;

    pub const MAX_OFFSET: usize = 0xFFFF;
    pub const POSSIBLE_OFFSET_COUNT: usize = Self::MAX_OFFSET / Self::COLUMNS_PER_BLOCK;
    pub const SHOWN_OFFSET_COUNT: usize = Self::ROWS_PER_BLOCK * Self::BLOCKS;
    pub const TOTAL_OFFSET_LENGTH: usize =
        Self::ROWS_PER_BLOCK * Self::COLUMNS_PER_BLOCK * Self::BLOCKS;
    pub const MAX_FIRST_OFFSET: usize = Self::POSSIBLE_OFFSET_COUNT - Self::SHOWN_OFFSET_COUNT;

    const CHARACTERS_PER_ROW: usize = Self::COLUMNS_PER_BLOCK;
    const CHARACTERS_PER_BLOCK: usize = Self::CHARACTERS_PER_ROW * Self::ROWS_PER_BLOCK;
    const CHARACTERS_TOTAL: usize = Self::CHARACTERS_PER_BLOCK * Self::BLOCKS;

    const _ASSERTION: () = const {
        assert!(
            (Self::MAX_FIRST_OFFSET * Self::COLUMNS_PER_BLOCK)
                + Self::SHOWN_OFFSET_COUNT * Self::COLUMNS_PER_BLOCK
                <= Self::MAX_OFFSET + 1
        );
    };

    fn random_first_offset(rng: &mut impl Rng) -> usize {
        rng.random_range(0..Self::MAX_FIRST_OFFSET) * Self::COLUMNS_PER_BLOCK
    }

    fn random_unique_words(rng: &mut impl Rng, word_length: usize, count: usize) -> Vec<String> {
        let mut words: Vec<&'static str> = Vec::with_capacity(count);
        let Some(wordlist) = wordlists::by_word_length(word_length) else {
            panic!("valid word length should've been ensured before calling this function");
        };
        while words.len() < count {
            let word = wordlist[rng.random_range(0..wordlist.len())];
            if !words.contains(&word) {
                words.push(word);
            }
        }

        words
            .iter()
            .map(|word| word.to_uppercase())
            .collect::<Vec<_>>()
    }

    fn random_content(rng: &mut impl Rng) -> (Vec<String>, Vec<(String, usize)>) {
        const WORD_LENGTH: usize = 4;
        const WORD_COUNT: usize = 10;

        let mut content = Vec::with_capacity(Self::ROWS_PER_BLOCK * Self::BLOCKS);
        for _ in 0..Self::BLOCKS {
            for _ in 0..Self::ROWS_PER_BLOCK {
                const BACKGROUND_CHARACTERS: &str = ";$@:!%+_?/,.\"'-\\#*<>()[]{}^=`";
                let mut row = String::new();
                for _ in 0..Self::COLUMNS_PER_BLOCK {
                    row.push(
                        BACKGROUND_CHARACTERS
                            .chars()
                            .nth(rng.random_range(0..BACKGROUND_CHARACTERS.len()))
                            .unwrap(),
                    );
                }
                content.push(row);
            }
        }

        let words = Self::random_unique_words(rng, WORD_LENGTH, WORD_COUNT);

        let mut word_positions = Vec::with_capacity(words.len());
        for _ in 0..words.len() {
            'outer: loop {
                let new_position =
                    rng.random_range(0..(MainWidget::CHARACTERS_TOTAL - WORD_LENGTH));
                'inner: for position in &word_positions {
                    if new_position + WORD_LENGTH < *position
                        || position + WORD_LENGTH < new_position
                    {
                        continue 'inner;
                    } else {
                        continue 'outer;
                    }
                }
                word_positions.push(new_position);
                break;
            }
        }

        let words = words.into_iter().zip(word_positions).collect::<Vec<_>>();

        for (word, position) in &words {
            let mut remaining_word = &word[..];
            let mut cursor = CursorPosition::from_index(*position);
            while remaining_word.len() > MainWidget::CHARACTERS_PER_ROW - cursor.column {
                let row = &mut content[cursor.row_index()];
                row.truncate(cursor.column);
                let (part, rem) =
                    remaining_word.split_at(MainWidget::CHARACTERS_PER_ROW - cursor.column);
                row.push_str(part);
                remaining_word = rem;

                cursor = cursor.next_row();
            }
            let row = &content[cursor.row_index()];
            let mut new_row = String::with_capacity(row.len());
            new_row.push_str(&row[..cursor.column]);
            new_row.push_str(remaining_word);
            new_row.push_str(&row[(cursor.column + remaining_word.len())..]);
            content[cursor.row_index()] = new_row;
        }

        (content, words)
    }

    pub fn new_random() -> Self {
        let mut rng = rand::rng();

        let first_offset = Self::random_first_offset(&mut rng);
        let (content, words) = Self::random_content(&mut rng);
        let solution = words[rng.random_range(0..words.len())].0.clone();

        let mut s = Self {
            first_offset,
            content,
            cursor: CursorPosition::default(),
            highlight: CursorHighlight::Char(0),
            words,
            solution,
        };

        s.fix_cursor_highlight();

        s
    }

    pub fn move_cursor(&mut self, position: Position) -> bool {
        let area = Rect::new(0, 0, Self::SIZE.width, Self::SIZE.height);

        let blocks = vec![
            area.resize(Self::BLOCK_SIZE),
            area.resize(Self::BLOCK_SIZE).offset(Offset::new(
                Self::BLOCK_SIZE.width as i32 + Self::SPACING as i32,
                0,
            )),
        ];
        for (block_index, block_area) in blocks.into_iter().enumerate() {
            let content_area = block_area
                .resize(Size::new(Self::COLUMNS_PER_BLOCK as u16, block_area.height))
                .offset(Offset::new(
                    Self::OFFSET_WIDTH as i32 + Self::SPACING as i32,
                    0,
                ));
            if !content_area.contains(position) {
                continue;
            }
            let projected_position = position.offset(Offset::new(
                -(content_area.x as i32),
                -(content_area.y as i32),
            ));
            self.set_cursor(CursorPosition {
                block: block_index,
                column: projected_position.x as usize,
                row: projected_position.y as usize,
            });
            return true;
        }

        false
    }

    fn set_cursor(&mut self, cursor: CursorPosition) {
        self.cursor = cursor;
        self.fix_cursor_highlight();
    }

    pub fn get_highlighted_string(&self) -> String {
        match self.highlight {
            CursorHighlight::Char(position) => self.content[self.cursor.row]
                .chars()
                .nth(position % Self::CHARACTERS_PER_ROW)
                .unwrap()
                .to_string(),
            CursorHighlight::Word {
                start_index,
                length,
            } => self
                .words
                .iter()
                .find_map(|(word, word_position)| {
                    (start_index >= *word_position && start_index < word_position + length)
                        .then_some(word)
                })
                .unwrap()
                .clone(),
        }
    }

    fn fix_cursor_highlight(&mut self) {
        let cursor_index = self.cursor.index();
        self.highlight = match self.words.iter().find(|(word, position)| {
            let end = position + word.len();
            cursor_index >= *position && cursor_index < end
        }) {
            Some((word, position)) => CursorHighlight::Word {
                start_index: *position,
                length: word.len(),
            },
            None => CursorHighlight::Char(cursor_index),
        };
    }

    fn likeness_score(&self, word: &str) -> usize {
        self.solution
            .chars()
            .zip(word.chars())
            .filter(|(c1, c2)| c1 == c2)
            .count()
    }

    pub fn click(&mut self) -> ClickResult {
        match self.highlight {
            CursorHighlight::Char(position) => {
                let cursor = CursorPosition::from_index(position);
                let c = self.content[cursor.row].chars().nth(cursor.column).unwrap();
                ClickResult {
                    kind: ClickResultKind::Char,
                    string: c.to_string(),
                }
            }
            CursorHighlight::Word { start_index, .. } => {
                let clicked_word = self
                    .words
                    .iter()
                    .find_map(|(word, word_position)| {
                        (*word_position == start_index).then_some(word)
                    })
                    .unwrap()
                    .clone();
                let kind = if clicked_word == self.solution {
                    ClickResultKind::Solution
                } else {
                    let likeness = self.likeness_score(&clicked_word);
                    ClickResultKind::Word { likeness }
                };
                ClickResult {
                    kind,
                    string: clicked_word,
                }
            }
        }
    }
}

impl Widget for &MainWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        assert_eq!(area.as_size(), MainWidget::SIZE);

        let blocks = vec![
            area.resize(MainWidget::BLOCK_SIZE),
            area.resize(MainWidget::BLOCK_SIZE).offset(Offset::new(
                MainWidget::BLOCK_SIZE.width as i32 + MainWidget::SPACING as i32,
                0,
            )),
        ];

        let highlight = match self.highlight {
            CursorHighlight::Char(index) => index..index + 1,
            CursorHighlight::Word {
                start_index,
                length,
            } => start_index..start_index + length,
        };

        for (block_index, block_area) in blocks.into_iter().enumerate() {
            let offset_area =
                block_area.resize(Size::new(MainWidget::OFFSET_WIDTH, block_area.height));
            for (row_index, row_area) in offset_area.rows().enumerate() {
                let row_offset = self.first_offset + row_index * MainWidget::COLUMNS_PER_BLOCK;
                format!("0x{:04X}", row_offset).render(row_area, buf);
            }

            let block = {
                let block_start = block_index * MainWidget::ROWS_PER_BLOCK;
                let block_end = block_start + MainWidget::ROWS_PER_BLOCK;
                &self.content[block_start..block_end]
            };
            let content_area = block_area
                .resize(Size::new(
                    MainWidget::COLUMNS_PER_BLOCK as u16,
                    block_area.height,
                ))
                .offset(Offset::new(
                    MainWidget::OFFSET_WIDTH as i32 + MainWidget::SPACING as i32,
                    0,
                ));
            for (row_index, row_area) in content_area.rows().enumerate() {
                assert_eq!(row_area.width, MainWidget::COLUMNS_PER_BLOCK as u16);
                assert_eq!(
                    block[row_index].chars().count(),
                    MainWidget::COLUMNS_PER_BLOCK
                );
                block[row_index].as_str().render(row_area, buf);
            }

            for index in highlight.clone() {
                let position = CursorPosition::from_index(index);
                if position.block != block_index {
                    continue;
                }
                buf.set_style(
                    content_area
                        .resize(Size::new(1, 1))
                        .offset(Offset::new(position.column as i32, position.row as i32)),
                    Style::new().reversed(),
                );
            }
        }
    }
}
