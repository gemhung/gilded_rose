pub const QUALITY_MIN: i32 = 0;
pub const QUALITY_MAX: i32 = 50;

#[must_use]
#[inline]
#[allow(clippy::manual_clamp)]
pub fn inc_to_cap(q: i32, n: i32) -> i32 {
    if q >= QUALITY_MAX {
        q
    } else {
        q.saturating_add(n).min(QUALITY_MAX)
    }
}

#[must_use]
#[inline]
#[allow(clippy::manual_clamp)]
pub fn dec_to_floor(q: i32, n: i32) -> i32 {
    if q <= QUALITY_MIN {
        q
    } else {
        q.saturating_sub(n).max(QUALITY_MIN)
    }
}

#[cfg(debug_assertions)]
pub fn assert_preconditions(kind: &Kind, q: i32) {
    // Legendary (Sulfuras): quality must be exactly 80.
    if matches!(kind, Kind::Legendary) {
        debug_assert!(
            q == 80,
            "Legendary (Sulfuras) must have quality 80, got {}",
            q
        );
        return;
    }
    // Non-legendary items: quality is expected to start within [0, 50].
    if !matches!(kind, Kind::Legendary) {
        debug_assert!(
            (QUALITY_MIN..=QUALITY_MAX).contains(&q),
            "quality out of range: {}",
            q
        );
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Kind {
    AgedBrie,
    BackstagePass,
    Legendary,
    Normal,
}

const NAME_BRIE: &str = "Aged Brie";
const PREFIX_BACKSTAGE: &str = "Backstage passes";
const NAME_SULFURAS: &str = "Sulfuras, Hand of Ragnaros";

impl From<&str> for Kind {
    #[inline]
    fn from(name: &str) -> Self {
        match name {
            NAME_BRIE => Kind::AgedBrie,
            NAME_SULFURAS => Kind::Legendary,
            s if s.starts_with(PREFIX_BACKSTAGE) => Kind::BackstagePass,
            _ => Kind::Normal,
        }
    }
}

// Detect "Conjured " as an item property (not a separate type).
const PREFIX_CONJURED: &str = "Conjured ";

#[must_use]
#[inline]
pub fn split_conjured(name: &str) -> (bool, &str) {
    // Assumes valid inputs: if it starts with "Conjured ", there is a non-empty base name after it.
    if let Some(rest) = name.strip_prefix(PREFIX_CONJURED) {
        (true, rest)
    } else {
        (false, name)
    }
}
