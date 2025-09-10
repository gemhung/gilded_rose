![CI – Rust Build & Test](https://github.com/gemhung/gilded_rose/actions/workflows/build.yml/badge.svg)

## ⚡ Quickstart
- **Working folder**: `cd rust`
- **Toolchain:** Rust 1.89 (pinned via `rust-toolchain.toml`)
- **Build:** `cargo build --all --locked`
- **Lint:** `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
- **Test:** `cargo test --all --locked -q`


# 📝 Thinking Process 

## ✅ Functional requirements: 
1. Item and Item property definition remain unchanged
2. Once the sell-by date has passed, `Quality` degrades twice as fast
3. The `Quality` of an item is never negative
4. `Aged Brie` actually increases in Quality the older it gets
5. The `Quality` of an item is never more than 50
6. `Sulfuras`, being a legendary item, never has to be sold or decreases in Quality
7. `Backstage passes`, like `Aged Brie`, increase in `Quality` as its `SellIn` value approaches;
   - `Quality` increases by 2 when there are 10 days or less and by 3 when there are 5 days or less but
   - `Quality` drops to 0 after the concert
8. `Conjured items`: `Conjured` items degrade in Quality twice as fast as normal items. Please note that `Conjured` is a property of the item and not a type, and as such any item can be conjured (eg, `Conjured Aged Brie`, `Conjured Backstage passes`).
9. `Sulfuras` is a legendary item and as such its Quality is 80 and it never alters

## ⚠️ Preconditions and assumptions
### 📌 Inclusive thresholds & day-0 semantics
According to `Requirement #7`, `Backstage` thresholds are inclusive:
`days or less` means ≤10 and ≤5.

The original implementation used `< 10` and `< 5`, which contradicts the spec.

This change aligns the code with the Requirements by changing:
- `< 10` → `<= 10`
- `< 5`  → `<= 5`

Rationale: `Requirements-first`. The written spec says `days or less`, so thresholds must be `inclusive`.

### 📌 Assume input is always valid
The original code didn't validate inputs. The changes here focus on refactoring and aligning behavior with the Requirements, not on adding global input normalization. `We assume valid inputs` and only guard each operation with `inc_to_cap` / `dec_to_floor`. 
For development safety, debug-only preconditions are enforced:
  - Non-legendary items: 0 ≤ quality ≤ 50
  - Sulfuras (Legendary): quality == 80 (and never changes)
### 📌 Conjured name parsing (strict) & bare "Conjured" policy
* Conjured is a property recognized only when the name starts with `Conjured ` (with a trailing space) followed by a non-empty base name.
* A bare `Conjured` (no base name) is treated as non-conjured Normal.
### 📌 Conjured interaction policy (degrade-only)
We adopt a **degrade-only** interpretation for Conjured:

- `Conjured` doubles **negative** quality changes only (degradation).
- It does **not** amplify positive gains (e.g., Aged Brie’s +1/+2) nor override drop-to-zero rules (Backstage).
- Legendary items (Sulfuras) remain immutable.

**Rationale:** The kata wording says `Conjured items **degrade** twice as fast,` which we interpret narrowly as *decrease*-only. Special items (Brie/Backstage) keep their own positive/zeroing behaviors.

### 📌 Non-goals
* No input normalization (e.g., no auto-trim, no case-insensitive matching, no global clamping).
* No cross-thread concurrency semantics or locking.
* No micro-optimizations or alternative data structures.
* No introduction of `lib.rs` (kept as a small, single-file kata for easier review).
* No introduction of `tracing/logging` or `telemetry`.

## 🧩 Modeling
**Intent:** minimize diffs, maximize reviewability; keep rules explicit and edge cases locked by tests.
### 📌 What is modeled
- **Classification:** Item behavior is driven by a **kind** derived from its name:
  - `Kind = { AgedBrie, BackstagePass, Legendary, Normal }`
  - `Conjured` is a **property** on top of the base name (not a separate type).
- **Parsing policy:** `split_conjured(name) -> (is_conjured, base_name)` treats `Conjured ` (with a trailing space) as the only valid prefix. The `base_name` then maps to `Kind` via an exhaustive conversion.
- **State & update pipeline:** Rules are a pure function of `(Kind, is_conjured, sell_in, quality)` with a fixed order:
  1) apply the day’s **quality rule**  
  2) `sell_in = sell_in.saturating_sub(1)`  
  3) apply the **expiry rule** (if now expired)
- **Complexity:** `O(n)` where `n` = number of items.
- **Deterministic:** no randomness, wall-clock, or timezone dependencies.

### 📌 Why not traits here
- **Exhaustiveness:** An `enum` plus **exhaustive `match`** gives compile-time coverage for all kinds; adding a new kind forces code & tests to update.
- **Local reasoning:** Keeps logic explicit and in one place (no dynamic dispatch or indirection).
- **YAGNI for this challenge:** Polymorphic traits would add complexity without clear benefit; see `Coding Style: Use traits only when necessary`.

### 📌 Guard-rails (invariants)
- **Legendary:** no changes to `sell_in` or `quality`.
- **Boundaries:** quality kept within `[0, 50]` by operation-level helpers (`inc_to_cap`, `dec_to_floor`); no global clamp.
- **Overflow-safe:** `sell_in` uses `saturating_sub(1)`; `i32::MIN` covered by tests.
- **Debug-only preconditions:** assert non-legendary `quality ∈ [0,50]`; legendary `quality == 80` (original kata rule).
- **Inclusive thresholds:** Backstage uses ≤10 and ≤5 (`days or less`).

### 📌 Known deviations
- `sell_in` **saturates at `i32::MIN`** instead of underflowing.
- Runtime **does not force** Sulfuras quality to 80 (keeps whatever input is provided); the 80 check is only enforced in **debug assertions** to reflect the original kata rule.

### 📌 Minimal API sketch (for reviewers)
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
enum Kind { AgedBrie, BackstagePass, Legendary, Normal }

// Update pipeline: quality rule → sell_in.saturating_sub(1) → expiry rule
// Classification is done inline at the callsite:
let (is_conjured, base_name) = split_conjured(item.name.as_str());
let kind: Kind = base_name.into();
```
**Why no `Quality`/`SellIn`/`ItemName` newtypes?**  
The kata requires the original `Item` shape to remain unchanged. To keep diffs small and reviewable, I keep `i32` fields and enforce invariants via small operation-level helpers. (Future-ready: internal newtypes could be added behind a feature without changing the public shape.)

### ➕ How to add a new Kind
1. Add a new variant to `enum Kind` in `spec.rs`.
2. Extend `From<&str> for Kind` (classification) and update `split_conjured` behavior if needed.
3. Add daily and expiry rules in `update_one_item`’s `match` arms.
4. Add tests that cover: pre-expiry, expiry transition, post-expiry, caps/floors, and Conjured interaction (if applicable).
5. Run `cargo fmt && cargo clippy -D warnings && cargo test`.

### 📌 Rule matrix (per tick)

| Kind \ Sell-in band | >10 | 6..=10 | 1..=5 | ≤0 (expired) |
|---|---:|---:|---:|---:|
| **Normal** | −1 | −1 | −1 | −2 |
| **Conjured Normal** | **−2** | **−2** | **−2** | **−4** |
| **Aged Brie** | +1 | +1 | +1 | +1 (cap 50) |
| **Backstage** | +1 | +2 | +3 | → 0 |
| **Legendary (Sulfuras)** | 0 | 0 | 0 | 0 |

### ⚖️ Rule precedence & conflicts
When rules appear to conflict, special-case rules take precedence over generic ones:

1. **Legendary (Sulfuras)**: immutable — no `sell_in` or `quality` change.
2. **Backstage**: banded positive increments; **after the concert it becomes `0`**.
3. **Aged Brie**: always increases; **after expiry it effectively gains +2/day** (day increment + expiry pass), capped at 50.
4. **Generic/Normal**: −1/day; **after expiry −2/day**.
5. **Conjured (property)**: see [Conjured interaction policy](#📌-conjured-interaction-policy-(degrade-only))

### 🧪 Edge behaviors (explicit)
- **Order of operations per day**: (1) apply daily rule → (2) `sell_in = sell_in.saturating_sub(1)` → (3) apply expiry rule if `sell_in < 0`.
- **Backstage “after concert”**: means the post-decrement `sell_in` is `< 0`; then `quality = 0`.
- **Saturating sell_in**: `sell_in` uses `saturating_sub(1)`; at `i32::MIN` it stays `i32::MIN`.
- **Bounds**: all quality updates use `inc_to_cap` / `dec_to_floor`; no global clamp is applied elsewhere.


### 📎 Requirement traceability

| Req | Summary | Code anchor | Key tests |
|---|---|---|---|
| #1 | Item shape unchanged | `rust/src/gilded_rose.rs` (structs/signatures) | `empty_inventory_no_panic`, `multiple_items_update_independently` |
| #2 | After expiry, normal degrades ×2 | `update_one_item` match: `Kind::Normal` + expiry pass | `normal_after_expiry_degrades_by_2`, `normal_expiry_transition_exact` |
| #3 | Quality never negative | `dec_to_floor` | `invariant_quality_never_negative_for_non_sulfuras`, `normal_quality_never_negative_even_after_expiry` |
| #4 | Aged Brie increases | `Kind::AgedBrie` branch + expiry pass | `brie_before_expiry_increases_by_1`, `brie_after_expiry_increases_by_2`, `brie_caps_at_50`, `brie_stays_50_even_after_expiry`, `brie_expired_from_49_hits_cap_and_stays` |
| #5 | Quality ≤ 50 | `inc_to_cap` | `invariant_quality_never_exceeds_50_for_non_sulfuras`, `backstage_caps_at_50_when_incrementing` |
| #6 | Sulfuras immutable | early return on `Kind::Legendary` | `sulfuras_does_not_change_sellin_or_quality`, `sulfuras_with_negative_sell_in_unchanged` |
| #7 | Backstage bands & drop to 0 | `Kind::BackstagePass` bands + expiry drop | `backstage_between_6_and_10_days_plus_2_edges`, `backstage_between_1_and_5_days_plus_3_edges`, `backstage_after_concert_drops_to_zero`, `backstage_exact_transition_points`, `backstage_monotonic_until_concert_then_zero` |
| #8 | Conjured degrades ×2 (degrade-only) | `split_conjured` + doubled decrement | `conjured_normal_degrades_by_2_before_expiry`, `conjured_normal_degrades_by_4_after_expiry`, `conjured_normal_respects_quality_floor`, `conjured_aged_brie_behaves_like_regular_brie`, `conjured_backstage_behaves_like_regular_backstage_and_drops_to_zero`, `conjured_backstage_thresholds_are_unchanged` |
| #9 | Sulfuras quality is 80 | debug assertions in `assert_preconditions` | covered implicitly by Sulfuras tests |


## 🗂️ File Layout
In the scope of this challenge, a **`flat file structure`** is used for simplicity and ease of review.  
This keeps all relevant logic and tests close together, so reviewers can focus on the rules rather than navigation.  
In a production setting, files could be reorganized into modules, but here a flat structure keeps the focus on business rules.
```bash
rust/
├─ src/
│  ├─ gilded_rose.rs   
│  ├─ spec.rs          
│  ├─ unit_tests.rs    
│  └─ main.rs          
├─ Cargo.toml
├─ rust-toolchain.toml
└─ rustfmt.toml
```

- 📑 **`src/spec.rs`**
  Shared helpers and constants that define the business rules. Contains utility functions like `inc_to_cap` / `dec_to_floor` and constants for quality boundaries (`QUALITY_MIN`, `QUALITY_MAX`).
- 📑 **`src/unit_tests.rs`**
  Centralized test suite for the kata. Placed under `src/` (instead of `tests/`) so reviewers can see all rules and edge cases in one file. Runs with `cargo test`.
- 📑 **`src/main.rs`**
  Minimal binary entry point, mainly a placeholder for manual runs or debugging.
  *Note: Left unchanged so reviewers can focus only on the rule refactoring.*
- 📑 **`src/gilded_rose.rs`**
  Core implementation of the kata. Defines the `Item` struct, the `GildedRose` container, and the `update_quality` logic.
  *Note: This file is kept as close to the original as possible to make diffs easier for reviewers.*


## 🦀 My Rust Coding Style for this challenge
- **Prefer `match` over long `if/else` chains** in large or complex functions; use pattern/range matches and keep arms explicit.
- **Prefer exhaustive `match` arms** instead of a catch-all `_ => { ... }` when matching on known enums or types.
- **Prefer `if/else`** for small, local checks to avoid unnecessary ceremony.
- **Prefer early returns** to reduce nesting and highlight the happy path.
- **Prefer std traits** (e.g., `AsRef`, `From/Into`, `Borrow`, `Deref`) before adding third-party trait crates.
- **Prefer targeted comments** over relying on `self-documenting code`; explain *why*, not the obvious *what*.
- **Prefer toolchain lockdown** to avoid breaking code.
- **Decompose large functions with care**. Break them down only when it improves clarity; avoid over-fragmentation and preserve readability.
- **Use `traits` only when necessary**. Introduce traits only when they clearly reduce duplication or enable clear polymorphism; avoid premature abstraction and overly generic designs, especially in application code (favor concrete types by default).
- **Balance functional combinators and imperative code**—pick whichever is clearest for the reader.
- **Leverage the type system** to encode invariants and prevent semantic misuse.
- **Keep `crate dependencies` minimal** and explicit.
- **Avoid `unwrap()`/`expect()` in non-test code.**


## 🗒️ Author’s Notes
Most of my time on this challenge went into aligning the code with the written requirements and clarifying scope—what to implement and what to leave out. Once that was settled, the coding was straightforward.

Since this isn’t a PR, I moved the usual PR context into this README: preconditions, assumptions, design decisions, known deviations, and test commitments. The goal is to keep the reviewer and me aligned without back-and-forth.

Future improvements could include `profiling`, `concurrency support`, `statelessness`, `observability`, `coverage`, `CI enhancements`, and `packaging as a library`. For this challenge, I focused on the core kata requirements and kept things simple and clear.

Throughout, I optimized for reviewability over cleverness: small diffs, a flat structure, explicit rules, and edge cases locked down by tests.

## 🙏 Thanks
Thanks for taking the time to read through the code and README. Much appreciated.
If you’d like to discuss the challenge, trade-offs, or tests, I’d be glad to connect.  
Please open an issue/PR on this repo or reach me at **gembright.stone.hung@gmail.com**.

