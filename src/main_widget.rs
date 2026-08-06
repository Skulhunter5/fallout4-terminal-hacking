use rand::{Rng, RngExt};
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Position, Rect, Size},
    style::Style,
    widgets::Widget,
};

#[derive(Debug, Default)]
struct CursorPosition {
    block: usize,
    column: usize,
    row: usize,
}

#[derive(Debug)]
pub struct MainWidget {
    first_offset: usize,
    content: Vec<String>,
    cursor: CursorPosition,
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

    // TODO: generate some actual content
    fn random_content(rng: &mut impl Rng) -> Vec<String> {
        let mut content = Vec::with_capacity(Self::ROWS_PER_BLOCK * Self::BLOCKS);
        for _ in 0..Self::BLOCKS {
            for _ in 0..Self::ROWS_PER_BLOCK {
                const LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
                let mut row = String::new();
                for _ in 0..Self::COLUMNS_PER_BLOCK {
                    row.push(
                        LETTERS
                            .chars()
                            .nth(rng.random_range(0..LETTERS.len()))
                            .unwrap(),
                    );
                }
                content.push(row);
            }
        }

        content
    }

    pub fn new_random() -> Self {
        let mut rng = rand::rng();

        let first_offset = Self::random_first_offset(&mut rng);
        let content = Self::random_content(&mut rng);

        Self {
            first_offset,
            content,
            cursor: CursorPosition::default(),
        }
    }

    pub fn move_cursor(&mut self, position: Position) {
        let area = Rect::new(0, 0, Self::SIZE.width, Self::SIZE.height);

        let blocks = vec![
            area.resize(MainWidget::BLOCK_SIZE),
            area.resize(MainWidget::BLOCK_SIZE).offset(Offset::new(
                MainWidget::BLOCK_SIZE.width as i32 + MainWidget::SPACING as i32,
                0,
            )),
        ];
        for (block_index, block_area) in blocks.into_iter().enumerate() {
            let content_area = block_area
                .resize(Size::new(
                    MainWidget::COLUMNS_PER_BLOCK as u16,
                    block_area.height,
                ))
                .offset(Offset::new(
                    MainWidget::OFFSET_WIDTH as i32 + MainWidget::SPACING as i32,
                    0,
                ));
            if !content_area.contains(position) {
                continue;
            }
            let projected_position = position.offset(Offset::new(
                -(content_area.x as i32),
                -(content_area.y as i32),
            ));
            self.cursor = CursorPosition {
                block: block_index,
                column: projected_position.x as usize,
                row: projected_position.y as usize,
            };
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

            if self.cursor.block == block_index {
                buf.set_style(
                    content_area.resize(Size::new(1, 1)).offset(Offset::new(
                        self.cursor.column as i32,
                        self.cursor.row as i32,
                    )),
                    Style::new().reversed(),
                );
            }
        }
    }
}
