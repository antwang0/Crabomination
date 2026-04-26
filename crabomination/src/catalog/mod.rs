//! Card catalog — factory functions for every implemented Magic: The Gathering card.
//! Cards are grouped by the set in which they first appeared.

mod sets;

// Re-export everything so callers use `catalog::some_card()`.
pub use sets::all::*;
pub use sets::ap::*;
pub use sets::arn::*;
pub use sets::dis::*;
pub use sets::fem::*;
pub use sets::gpt::*;
pub use sets::ice::*;
pub use sets::inv::*;
pub use sets::lea::*;
pub use sets::m11::*;
pub use sets::ogw::*;
pub use sets::pc2::*;
pub use sets::por::*;
pub use sets::rav::*;
pub use sets::rtr::*;
pub use sets::tmp::*;
pub use sets::zen::*;
pub use sets::ths::*;
pub use sets::decks::*;
