use crate::spec::{Kind, assert_preconditions, dec_to_floor, inc_to_cap, split_conjured};
use std::fmt::{self, Display};

//  Requirements for the Gilded Rose system:
//    1. Item class definition and its method remain unchanged.
//
pub struct Item {
    pub name: String,
    pub sell_in: i32,
    pub quality: i32,
}

impl Item {
    pub fn new(name: impl Into<String>, sell_in: i32, quality: i32) -> Item {
        Item {
            name: name.into(),
            sell_in,
            quality,
        }
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}, {}", self.name, self.sell_in, self.quality)
    }
}

pub struct GildedRose {
    pub items: Vec<Item>,
}

//  Requirements for the Gilded Rose system:
//    2. Update method signature remains unchanged.
//
impl GildedRose {
    pub fn new(items: Vec<Item>) -> GildedRose {
        GildedRose { items }
    }

    pub fn update_quality(&mut self) {
        for it in &mut self.items {
            Self::update_one_item(it);
        }
    }

    fn update_one_item(it: &mut Item) {
        // Requirements: Any item can be conjured (eg, "Conjured Aged Brie", "Conjured Backstage passes").
        let (is_conjured, base_name) = split_conjured(it.name.as_str());
        // Determine item kind
        let kind: Kind = base_name.into();
        // Requirements: Legendary items do not change in quality or sell_in
        if matches!(kind, Kind::Legendary) {
            return;
        }
        // In debug mode, assert preconditions for non-legendary items (0-50 quality)
        #[cfg(debug_assertions)]
        assert_preconditions(&kind, it.quality);
        // Determine quality decrement delta (1 for normal, 2 for conjured)
        let dec_delta = if is_conjured { 2 } else { 1 };
        // Update quality based on kind and sell_in
        match (&kind, it.sell_in) {
            // Requirements: Legendary items do not change in quality or sell_in
            (Kind::Legendary, _) => {
                unreachable!("Legendary items should have been returned earlier")
            }
            // Requirements: Aged Brie increases in quality as it ages
            (Kind::AgedBrie, _) => it.quality = inc_to_cap(it.quality, 1),
            // Requirements: Backstage
            // 1. Like Aged Brie, quality increases by 1 when there are more than 10 days left
            // 2. Quality increases by 2 when there are 10 days or less
            // 3. Quality increases by 3 when there are 5 days or less
            // 4. Quality drops to 0 after the concert
            (Kind::BackstagePass, (11..)) => it.quality = inc_to_cap(it.quality, 1),
            (Kind::BackstagePass, (6..=10)) => it.quality = inc_to_cap(it.quality, 2),
            (Kind::BackstagePass, (1..=5)) => it.quality = inc_to_cap(it.quality, 3),
            (Kind::BackstagePass, (..=0)) => (), // Handled later in the expiry pass (after sell_in--)
            // Requirements: Normal items decrease in quality by 1 each day
            (Kind::Normal, _) => it.quality = dec_to_floor(it.quality, dec_delta),
        }
        // Decrease sell_in for all but legendary items
        it.sell_in = it.sell_in.saturating_sub(1);
        // Handle expired items
        if it.sell_in.is_negative() {
            match kind {
                // Requirements: Once the sell by date has passed, Quality degrades twice as fast
                Kind::Normal => it.quality = dec_to_floor(it.quality, dec_delta),
                Kind::AgedBrie => it.quality = inc_to_cap(it.quality, 1),
                // Requirements: Backstage quality drops to 0 after the concert
                Kind::BackstagePass => it.quality = 0,
                Kind::Legendary => {
                    unreachable!("Legendary items should have been returned earlier");
                }
            }
        }
    }
}
