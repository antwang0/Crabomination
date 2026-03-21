//! Card catalog — factory functions for every implemented Magic: The Gathering card.
//! Cards are grouped by the set in which they first appeared.

mod sets;

// Re-export everything so callers use `catalog::some_card()`.
pub use sets::ap::*;
pub use sets::dis::*;
pub use sets::gpt::*;
pub use sets::inv::*;
pub use sets::lea::*;
pub use sets::ogw::*;
pub use sets::pc2::*;
pub use sets::por::*;
pub use sets::rav::*;
pub use sets::rtr::*;
pub use sets::zen::*;
pub use sets::ths::*;
