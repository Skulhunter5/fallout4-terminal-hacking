use rand::{Rng, RngExt};
use ratatui::{
    buffer::Buffer,
    layout::{Offset, Rect, Size},
    widgets::Widget,
};

#[derive(Debug)]
pub struct MainWidget {
    first_offset: usize,
    // TODO: store content
}

impl Default for MainWidget {
    fn default() -> Self {
        Self::new_random()
    }
}

impl MainWidget {
    pub const SIZE: Size = Size::new(Self::WIDTH, Self::HEIGHT);
    pub const WIDTH: u16 = Self::BLOCK_WIDTH * 2 + 1;
    pub const HEIGHT: u16 = Self::ROWS as u16;
    const BLOCK_WIDTH: u16 = Self::OFFSET_WIDTH + 1 + Self::COLUMNS_PER_BLOCK as u16;
    const BLOCK_SIZE: Size = Size::new(Self::BLOCK_WIDTH, Self::ROWS as u16);
    const OFFSET_WIDTH: u16 = 6;

    pub const ROWS: usize = 17;
    pub const COLUMNS_PER_BLOCK: usize = 12;
    pub const BLOCKS: usize = 2;

    pub const MAX_OFFSET: usize = 0xFFFF;
    pub const POSSIBLE_OFFSET_COUNT: usize = Self::MAX_OFFSET / Self::COLUMNS_PER_BLOCK;
    pub const SHOWN_OFFSET_COUNT: usize = Self::ROWS * Self::BLOCKS;
    pub const TOTAL_OFFSET_LENGTH: usize = Self::ROWS * Self::COLUMNS_PER_BLOCK * Self::BLOCKS;
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

    pub fn new_random() -> Self {
        let mut rng = rand::rng();

        let first_offset = Self::random_first_offset(&mut rng);
        // TODO: generate content

        Self { first_offset }
    }
}

impl Widget for &MainWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        assert_eq!(area.as_size(), MainWidget::SIZE);

        let blocks: Vec<(usize, Rect)> = vec![
            (0, area.resize(MainWidget::BLOCK_SIZE)),
            (
                1,
                area.resize(MainWidget::BLOCK_SIZE)
                    .offset(Offset::new(MainWidget::BLOCK_SIZE.width as i32, 0)),
            ),
        ];
        for (_block_index, block_area) in blocks {
            let offset_area =
                block_area.resize(Size::new(MainWidget::OFFSET_WIDTH, block_area.height));
            (0..MainWidget::ROWS)
                .zip(offset_area.rows())
                .for_each(|(row_index, row_area)| {
                    let row_offset = self.first_offset + row_index * MainWidget::COLUMNS_PER_BLOCK;
                    format!("0x{:04x}", row_offset).render(row_area, buf);
                });
            // TODO: render content
        }
    }
}
