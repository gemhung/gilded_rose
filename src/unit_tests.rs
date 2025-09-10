use crate::gilded_rose::{GildedRose, Item};
use crate::spec::{QUALITY_MAX, QUALITY_MIN, dec_to_floor, inc_to_cap};

fn mk(name: &str, sell_in: i32, quality: i32) -> Item {
    Item::new(name, sell_in, quality)
}

fn rose_with(items: Vec<Item>) -> GildedRose {
    GildedRose::new(items)
}

fn tick(rose: &mut GildedRose) {
    rose.update_quality();
}

//
// Invariants & Common Rules
//

#[test]
fn invariant_quality_never_negative_for_non_sulfuras() {
    let mut r = rose_with(vec![mk("foo", 0, 0)]);
    tick(&mut r);
    assert_eq!(r.items[0].quality, 0);
}

#[test]
fn invariant_quality_never_exceeds_50_for_non_sulfuras() {
    let mut r = rose_with(vec![mk("Aged Brie", 1, 50)]);
    tick(&mut r);
    assert_eq!(r.items[0].quality, 50);
}

#[test]
fn invariant_sell_in_decrements_by_one_for_non_sulfuras() {
    let mut r = rose_with(vec![mk("foo", 10, 10)]);
    tick(&mut r);
    assert_eq!(r.items[0].sell_in, 9);
}

#[test]
fn sulfuras_does_not_change_sellin_or_quality() {
    let mut r = rose_with(vec![
        mk("Sulfuras, Hand of Ragnaros", 0, 80),
        mk("Sulfuras, Hand of Ragnaros", 10, 80),
    ]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 80));
    assert_eq!((r.items[1].sell_in, r.items[1].quality), (10, 80));
}

#[test]
fn invariant_quality_within_range_after_many_days() {
    let mut r = rose_with(vec![
        mk("foo", 5, 1),
        mk("Aged Brie", 2, 49),
        mk("Backstage passes to a TAFKAL80ETC concert", 3, 49),
    ]);
    for _ in 0..10 {
        tick(&mut r);
        for it in &r.items {
            if it.name != "Sulfuras, Hand of Ragnaros" {
                assert!((0..=50).contains(&it.quality));
            }
        }
    }
}

//
// Normal
//

#[test]
fn normal_before_expiry_degrades_by_1() {
    let mut r = rose_with(vec![mk("foo", 5, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (4, 9));
}

#[test]
fn normal_after_expiry_degrades_by_2() {
    let mut r = rose_with(vec![mk("foo", 0, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 8));
}

#[test]
fn normal_quality_never_negative_even_after_expiry() {
    let mut r = rose_with(vec![mk("foo", 0, 1)]);
    tick(&mut r);
    tick(&mut r);
    assert_eq!(r.items[0].quality, 0);
}

#[test]
fn normal_expiry_transition_exact() {
    let mut r = rose_with(vec![mk("foo", 1, 10)]);
    tick(&mut r); // -> 0, -1
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 9));
    tick(&mut r); // -> -1, -2
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 7));
}

//
// Aged Brie
//

#[test]
fn brie_before_expiry_increases_by_1() {
    let mut r = rose_with(vec![mk("Aged Brie", 2, 0)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (1, 1));
}

#[test]
fn brie_after_expiry_increases_by_2() {
    let mut r = rose_with(vec![mk("Aged Brie", 0, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 12));
}

#[test]
fn brie_caps_at_50() {
    let mut r = rose_with(vec![mk("Aged Brie", 1, 49)]);
    tick(&mut r);
    tick(&mut r);
    assert_eq!(r.items[0].quality, 50);
}

//
// Backstage
//

#[test]
fn backstage_more_than_10_days_plus_1() {
    let mut r = rose_with(vec![mk(
        "Backstage passes to a TAFKAL80ETC concert",
        15,
        20,
    )]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (14, 21));
}

#[test]
fn backstage_between_6_and_10_days_plus_2_edges() {
    for si in [10, 9, 6] {
        let mut r = rose_with(vec![mk(
            "Backstage passes to a TAFKAL80ETC concert",
            si,
            10,
        )]);
        tick(&mut r);
        assert_eq!(r.items[0].quality, 12, "start sell_in {}", si);
    }
}

#[test]
fn backstage_between_1_and_5_days_plus_3_edges() {
    for si in [5, 4, 1] {
        let mut r = rose_with(vec![mk(
            "Backstage passes to a TAFKAL80ETC concert",
            si,
            10,
        )]);
        tick(&mut r);
        assert_eq!(r.items[0].quality, 13, "start sell_in {}", si);
    }
}

#[test]
fn backstage_after_concert_drops_to_zero() {
    let mut r = rose_with(vec![mk("Backstage passes to a TAFKAL80ETC concert", 0, 30)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 0));
}

#[test]
fn backstage_caps_at_50_when_incrementing() {
    for si in [11, 10, 5, 1] {
        let mut r = rose_with(vec![mk(
            "Backstage passes to a TAFKAL80ETC concert",
            si,
            49,
        )]);
        tick(&mut r);
        assert_eq!(r.items[0].quality, 50, "start sell_in {}", si);
    }
}

#[test]
fn backstage_exact_transition_points() {
    let mut a = rose_with(vec![mk(
        "Backstage passes to a TAFKAL80ETC concert",
        11,
        10,
    )]);
    tick(&mut a); // 11→10 → +1
    assert_eq!((a.items[0].sell_in, a.items[0].quality), (10, 11));

    let mut b = rose_with(vec![mk("Backstage passes to a TAFKAL80ETC concert", 6, 10)]);
    tick(&mut b); // 6→5 → +2
    assert_eq!((b.items[0].sell_in, b.items[0].quality), (5, 12));
}

//
// Multi-day sanity
//

#[test]
fn multiple_days_normal_behaviour_consistent() {
    let mut r = rose_with(vec![mk("foo", 2, 5)]);
    tick(&mut r); // -> 1, 4
    tick(&mut r); // -> 0, 3
    tick(&mut r); // -> -1, 1（expired, -2）
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 1));
}

#[test]
fn backstage_two_day_transition_across_10_and_5() {
    // 11 -> 10：first day +1，second day +2
    let mut r = rose_with(vec![mk(
        "Backstage passes to a TAFKAL80ETC concert",
        11,
        10,
    )]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (10, 11));
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (9, 13));

    // 6 -> 5：first day +2，second day +3
    let mut r = rose_with(vec![mk("Backstage passes to a TAFKAL80ETC concert", 6, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (5, 12));
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (4, 15));
}

#[test]
fn brie_at_zero_day_caps_at_50() {
    let mut r = rose_with(vec![mk("Aged Brie", 0, 49)]);
    tick(&mut r); // +1 to 50，expired +1 → still 50
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 50));
}

#[test]
fn brie_stays_50_even_after_expiry() {
    let mut r = rose_with(vec![mk("Aged Brie", 0, 50)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 50));
}

#[test]
fn backstage_quality_50_at_concert_day_drops_to_zero() {
    let mut r = rose_with(vec![mk("Backstage passes to a TAFKAL80ETC concert", 0, 50)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 0));
}

#[test]
fn backstage_already_past_concert_stays_zero() {
    let mut r = rose_with(vec![mk(
        "Backstage passes to a TAFKAL80ETC concert",
        -1,
        10,
    )]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-2, 0));
}

#[test]
fn sulfuras_with_negative_sell_in_unchanged() {
    let mut r = rose_with(vec![mk("Sulfuras, Hand of Ragnaros", -5, 80)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-5, 80));
}

#[test]
fn sulfuras_is_case_sensitive_and_exact_match() {
    let mut r = rose_with(vec![mk("sulfuras, hand of ragnaros", 1, 10)]); // lowercase
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 9)); // as normal items -1
}

#[test]
fn empty_inventory_no_panic() {
    let mut r = rose_with(vec![]);
    tick(&mut r);
    assert!(r.items.is_empty());
}

#[test]
fn multiple_items_update_independently() {
    let mut r = rose_with(vec![
        mk("foo", 1, 10),       // normal
        mk("Aged Brie", 0, 49), // brie at cap boundary
        mk("Backstage passes to a TAFKAL80ETC concert", 5, 49),
        mk("Sulfuras, Hand of Ragnaros", 0, 80),
    ]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 9)); // normal -1
    assert_eq!((r.items[1].sell_in, r.items[1].quality), (-1, 50)); // brie capped
    assert_eq!((r.items[2].sell_in, r.items[2].quality), (4, 50)); // backstage capped (≤5 → +3)
    assert_eq!((r.items[3].sell_in, r.items[3].quality), (0, 80)); // sulfuras unchanged
}

#[test]
fn sell_in_keeps_decreasing_below_zero() {
    let mut r = rose_with(vec![mk("foo", 0, 10)]);
    for _ in 0..3 {
        tick(&mut r);
    }
    // start from 0，3 tick will get to -3
    assert_eq!(r.items[0].sell_in, -3);
}

#[test]
fn sell_in_saturates_at_min() {
    let mut r = rose_with(vec![mk("foo", i32::MIN, 10)]);
    tick(&mut r); // Shouldn't panic；It should remain MIN
    assert_eq!(r.items[0].sell_in, i32::MIN);
}

#[test]
fn sell_in_does_not_wrap_to_positive() {
    let mut r = rose_with(vec![mk("foo", -1, 10)]);
    for _ in 0..5 {
        tick(&mut r);
    }
    assert!(r.items[0].sell_in <= -6);
}

#[test]
fn sell_in_saturates_at_min_instead_of_wrapping() {
    let mut r = rose_with(vec![mk("foo", i32::MIN, 10)]);
    tick(&mut r);
    assert_eq!(r.items[0].sell_in, i32::MIN);
}

#[test]
fn sulfuras_name_must_match_exactly_no_trim() {
    let mut r = rose_with(vec![mk("Sulfuras, Hand of Ragnaros ", 1, 50)]);
    tick(&mut r);
    // Treated as normal item
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 49));
}

#[test]
fn backstage_monotonic_until_concert_then_zero() {
    let mut r = rose_with(vec![mk(
        "Backstage passes to a TAFKAL80ETC concert",
        12,
        10,
    )]);
    // 12->11->10 strictly increasing or stable (capped), then drops to 0 on concert day
    let mut last = r.items[0].quality;
    for _ in 0..=12 {
        tick(&mut r);
        if r.items[0].sell_in >= 0 {
            assert!(r.items[0].quality >= last);
        }
        last = r.items[0].quality;
    }
    assert_eq!(r.items[0].quality, 0);
}

#[test]
fn backstage_at_50_before_concert_stays_50() {
    for si in [11, 10, 6, 5, 1] {
        let mut r = rose_with(vec![mk(
            "Backstage passes to a TAFKAL80ETC concert",
            si,
            50,
        )]);
        tick(&mut r);
        assert_eq!(r.items[0].quality, 50, "start sell_in {}", si);
    }
}

#[test]
fn backstage_interval_increments_are_applied() {
    for (si, expected_inc) in [(12, 1), (10, 2), (5, 3)] {
        let mut r = rose_with(vec![mk("Backstage passes - hall", si, 10)]);
        tick(&mut r);
        assert_eq!(r.items[0].quality, 10 + expected_inc, "si={si}");
    }
}

#[test]
fn inc_basic_increment_stops_at_cap() {
    // Normal increase; must not exceed the cap.
    assert_eq!(inc_to_cap(49, 1), QUALITY_MAX); // 49 + 1 -> 50
    assert_eq!(inc_to_cap(49, 3), QUALITY_MAX); // 49 + 3 -> 50 (no overflow past cap)
}

#[test]
fn inc_at_cap_no_change_and_zero_step() {
    // Already at cap: no change. Zero step: no change.
    assert_eq!(inc_to_cap(QUALITY_MAX, 5), QUALITY_MAX);
    assert_eq!(inc_to_cap(20, 0), 20);
}

#[test]
fn inc_does_not_heal_when_above_cap() {
    // Route A behavior: do NOT "auto-heal" out-of-range inputs.
    assert_eq!(inc_to_cap(55, 10), 55);
    assert_eq!(inc_to_cap(i32::MAX, 1), i32::MAX);
}

#[test]
fn inc_handles_large_n_without_overflow() {
    // Large step should safely saturate at the cap.
    assert_eq!(inc_to_cap(49, i32::MAX), QUALITY_MAX);
}

#[test]
fn dec_basic_decrement_stops_at_floor() {
    // Normal decrease; must not go below the floor.
    assert_eq!(dec_to_floor(2, 1), 1);
    assert_eq!(dec_to_floor(1, 3), QUALITY_MIN); // 1 - 3 -> 0 (no negative)
}

#[test]
fn dec_at_floor_no_change_and_zero_step() {
    // Already at floor: no change. Zero step: no change.
    assert_eq!(dec_to_floor(QUALITY_MIN, 5), QUALITY_MIN);
    assert_eq!(dec_to_floor(20, 0), 20);
}

#[test]
fn dec_does_not_heal_when_below_floor() {
    // Route A behavior: do NOT "auto-heal" out-of-range inputs.
    assert_eq!(dec_to_floor(-3, 10), -3);
    assert_eq!(dec_to_floor(i32::MIN, 1), i32::MIN);
}

#[test]
fn dec_handles_large_n_without_overflow() {
    // Large step should safely saturate at the floor.
    assert_eq!(dec_to_floor(1, i32::MAX), QUALITY_MIN);
}

// --- Conjured-specific tests (degrade-only policy) ---

#[test]
fn conjured_normal_degrades_by_2_before_expiry() {
    // Normal degrades by -1; Conjured Normal doubles to -2.
    let mut r = rose_with(vec![mk("Conjured foo", 3, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (2, 8));
}

#[test]
fn conjured_normal_degrades_by_4_after_expiry() {
    // Normal expired degrades by -2; Conjured doubles the step to -4 total per day.
    let mut r = rose_with(vec![mk("Conjured foo", 0, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 6));
}

#[test]
fn conjured_normal_respects_quality_floor() {
    // Decrement should never push quality below 0.
    let mut r = rose_with(vec![mk("Conjured foo", 5, 1)]);
    tick(&mut r); // -2 but floored at 0
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (4, 0));

    // And stays at 0 with further ticks.
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (3, 0));
}

#[test]
fn conjured_aged_brie_behaves_like_regular_brie() {
    // Conjured doubles only *degrading* deltas. Brie increases, so it is unaffected.
    let mut r = rose_with(vec![mk("Conjured Aged Brie", 1, 49)]);
    tick(&mut r); // +1 to cap
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 50));
    tick(&mut r); // expired +1 but still capped at 50
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 50));
}

#[test]
fn conjured_backstage_behaves_like_regular_backstage_and_drops_to_zero() {
    // Conjured does not change the positive increments nor the drop-to-zero rule.
    let mut r = rose_with(vec![mk("Conjured Backstage passes - Hall", 6, 49)]);
    tick(&mut r); // 6..=10 => +2 (capped at 50)
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (5, 50));

    // Advance to concert day and then drop to 0 after the day ends.
    for _ in 0..5 {
        tick(&mut r);
    } // reach sell_in = 0
    tick(&mut r); // concert day completes -> quality becomes 0
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 0));
}

#[test]
fn conjured_legendary_is_still_immutable() {
    // Legendary items never change, with or without the Conjured prefix.
    let mut r = rose_with(vec![mk("Conjured Sulfuras, Hand of Ragnaros", 3, 80)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (3, 80));
}

#[test]
fn conjured_normal_with_min_sell_in_saturates_and_applies_double_degrade() {
    // sell_in uses saturating_sub(1), so i32::MIN remains i32::MIN; we still apply the rules.
    let mut r = rose_with(vec![mk("Conjured foo", i32::MIN, 3)]);
    tick(&mut r); // -2 (today) then -2 (expired pass) => floored at 0
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (i32::MIN, 0));
}

#[test]
fn conjured_backstage_thresholds_are_unchanged() {
    // Sanity on the three increment bands for Conjured Backstage.
    for (si, inc) in [(12, 1), (10, 2), (5, 3)] {
        let mut r = rose_with(vec![mk("Conjured Backstage passes - Arena", si, 10)]);
        tick(&mut r);
        assert_eq!(r.items[0].quality, 10 + inc, "sell_in={si}");
    }
}

#[test]
fn brie_expired_from_49_hits_cap_and_stays() {
    // Day 0: +1 to 50, then expired pass would add +1 but remains capped at 50.
    let mut r = rose_with(vec![mk("Aged Brie", 0, 49)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 50));
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-2, 50));
}

#[test]
fn conjured_normal_at_q2_before_expiry_floors_to_0() {
    // Conjured normal degrades by 2 before expiry.
    let mut r = rose_with(vec![mk("Conjured foo", 1, 2)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 0));
}

#[test]
fn conjured_normal_at_q4_on_expiry_day_drops_by_4_to_0() {
    // On expiry day: -2 (today) and -2 (expired pass) → floors at 0.
    let mut r = rose_with(vec![mk("Conjured foo", 0, 4)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 0));
}

#[test]
fn conjured_backstage_at_5_increments_but_caps_at_50() {
    // Backstage ≤5: +3, but capped; Conjured does not amplify positive increments.
    let mut r = rose_with(vec![mk("Conjured Backstage passes - Hall", 5, 48)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (4, 50));
}

#[test]
fn normal_at_quality_50_can_decrease() {
    // 50 is only an upper cap for increases; decreases are allowed.
    let mut r = rose_with(vec![mk("foo", 3, 50)]);
    tick(&mut r);
    assert_eq!(r.items[0].quality, 49);
}

#[test]
fn bare_conjured_is_treated_as_normal() {
    // Policy: a bare "Conjured" with no base name is treated as non-conjured Normal.
    let mut r = rose_with(vec![mk("Conjured", 2, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (1, 9));
}

#[test]
fn conjured_prefix_is_case_sensitive() {
    // Lowercase "conjured" is not recognized as the conjured prefix.
    let mut r = rose_with(vec![mk("conjured foo", 1, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 9));
}

#[test]
fn conjured_prefix_requires_trailing_space() {
    // "ConjuredAged Brie" (no space) does not match the "Conjured " prefix.
    let mut r = rose_with(vec![mk("ConjuredAged Brie", 1, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 9));
}

#[test]
fn normal_with_min_sell_in_degrades_by_2() {
    // With sell_in = i32::MIN (always expired): total -2 in one tick.
    let mut r = rose_with(vec![mk("foo", i32::MIN, 10)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (i32::MIN, 8));
}

#[test]
fn brie_with_min_sell_in_increases_by_2_but_caps() {
    // Aged Brie at i32::MIN: +1 (today) then +1 (expired pass), capped at 50.
    let mut r = rose_with(vec![mk("Aged Brie", i32::MIN, 48)]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (i32::MIN, 50));
}

#[test]
fn backstage_with_min_sell_in_drops_to_zero() {
    // Backstage at i32::MIN is always expired: quality becomes 0.
    let mut r = rose_with(vec![mk(
        "Backstage passes to a TAFKAL80ETC concert",
        i32::MIN,
        50,
    )]);
    tick(&mut r);
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (i32::MIN, 0));
}

#[test]
fn backstage_day_before_concert_caps_then_zero_next_day() {
    // Day 1 (sell_in=1): +3 up to cap 50; Day 2: drop to 0 after concert.
    let mut r = rose_with(vec![mk("Backstage passes to a TAFKAL80ETC concert", 1, 48)]);
    tick(&mut r); // -> (0, 50)
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (0, 50));
    tick(&mut r); // -> (-1, 0)
    assert_eq!((r.items[0].sell_in, r.items[0].quality), (-1, 0));
}
