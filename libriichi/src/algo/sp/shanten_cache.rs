use ahash::AHashMap;

/// Just to mention here, you can't reuse the cache for `Values` like this,
/// because `Values` depends on `tiles_seen` which changes rapidly in a game,
/// but shanten is only determined by the tehai. And very often, the shanten
/// cache will have a high hit rate throughout a kyoku since all hands are
/// either slightly derived from or the same as its previous one.
#[derive(Default)]
pub struct ShantenCache {
    pub at_3n1: AHashMap<[u8; 34], i8>,
    pub at_3n2: AHashMap<[u8; 34], i8>,
}

impl ShantenCache {
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.at_3n1.clear();
        self.at_3n2.clear();
    }
}
