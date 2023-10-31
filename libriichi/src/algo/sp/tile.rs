use crate::tile::Tile;

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct DiscardTile {
    pub(super) tile: Tile,
    pub(super) shanten_diff: i8,
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct DrawTile {
    pub(super) tile: Tile,
    pub(super) count: u8,
    pub(super) shanten_diff: i8,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RequiredTile {
    pub tile: Tile,
    pub count: u8,
}
