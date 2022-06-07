use crate::tile::Tile;

#[derive(Clone, Copy)]
pub enum ChiType {
    Low,
    Mid,
    High,
}

impl ChiType {
    #[must_use]
    pub fn new(consumed: [Tile; 2], tile: Tile) -> Self {
        let a = consumed[0].deaka().as_u8();
        let b = consumed[1].deaka().as_u8();
        let min = a.min(b);
        let max = a.max(b);
        let tile_id = tile.deaka().as_u8();
        if tile_id < min {
            Self::Low
        } else if tile_id < max {
            Self::Mid
        } else {
            Self::High
        }
    }
}
