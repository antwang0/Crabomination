//! Card catalog — factory functions for every implemented Magic: The Gathering card.
//! Cards are grouped by the set in which they first appeared.
//!
//! This crate was split out of the monolithic `crabomination` crate, and is
//! itself now a facade over nine part crates (`parts/*`) that compile the card
//! files in parallel; `sets/mod.rs` has the layout and the measurements. It
//! depends solely on `crabomination_base` (the card/mana/effect data model) and
//! its parts. The name→factory registry used by snapshot deserialization lives in
//! the top-level `crabomination` crate, since it also aggregates the cube/demo
//! pools.

// Crate-level allows inherited from the original monolithic `crabomination`
// crate, where the card factories lived.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::large_enum_variant)]

// ── Crate-private compatibility shims ────────────────────────────────────────
// The card files spell `crate::card`, `crate::effect`, `crate::mana`,
// `crate::game::..` and `crate::catalog::..`. They are compiled by the parts
// (`parts/*`, see `sets/mod.rs`), which take these names from the core part;
// this crate keeps the same set so a file declared here still resolves them.
extern crate self as catalog;
#[allow(unused_imports)]
use crabomination_catalog_core::*;

pub mod sets;

// Re-export everything so callers use `catalog::some_card()`.
pub use sets::akh::*;
pub use sets::all::*;
pub use sets::all2::*;
pub use sets::ap::*;
pub use sets::arn::*;
pub use sets::bng::*;
pub use sets::bng2::*;
pub use sets::bng3::*;
pub use sets::apc::*;
pub use sets::apc2::*;
pub use sets::ody::*;
pub use sets::pls::*;
pub use sets::pls2::*;
pub use sets::bot::*;
pub use sets::bro::*;
pub use sets::c21::*;
pub use sets::cmdr::*;
pub use sets::chk::*;
pub use sets::chk2::*;
pub use sets::bok::*;
pub use sets::bok2::*;
pub use sets::sok::*;
pub use sets::sok2::*;
pub use sets::sok3::*;
pub use sets::ulg::*;
pub use sets::unf::*;
pub use sets::uds::*;
pub use sets::usg::*;
pub use sets::usg2::*;
pub use sets::usg3::*;
pub use sets::chk3::*;
pub use sets::mmq::*;
pub use sets::mmq2::*;
pub use sets::mmq3::*;
pub use sets::mmq4::*;
pub use sets::mmq5::*;
pub use sets::mmq6::*;
pub use sets::nms::*;
pub use sets::nms2::*;
pub use sets::nms3::*;
pub use sets::nms4::*;
pub use sets::pcy::*;
pub use sets::pip::*;
pub use sets::pcy2::*;
pub use sets::pcy3::*;
pub use sets::pcy4::*;
pub use sets::curses::*;
pub use sets::decks::*;
pub use sets::dgm::*;
pub use sets::dis::*;
pub use sets::eoe::*;
pub use sets::eoe2::*;
pub use sets::fem::*;
pub use sets::fin::*;
pub use sets::fin2::*;
pub use sets::gpt::*;
pub use sets::gtc::*;
pub use sets::gtc2::*;
pub use sets::gtc3::*;
pub use sets::gtc4::*;
pub use sets::gtc5::*;
pub use sets::gtc6::*;
pub use sets::gtc7::*;
pub use sets::gtc8::*;
pub use sets::gtc9::*;
pub use sets::gtc10::*;
pub use sets::gtc11::*;
pub use sets::gtc12::*;
pub use sets::gtc13::*;
pub use sets::gtc14::*;
pub use sets::gtc15::*;
pub use sets::gtc16::*;
pub use sets::gtc17::*;
pub use sets::ice::*;
pub use sets::inv::*;
pub use sets::jou::*;
pub use sets::jou2::*;
pub use sets::jou3::*;
pub use sets::khm::*;
pub use sets::kld::*;
pub use sets::ktk::*;
pub use sets::lci::*;
pub use sets::lea::*;
pub use sets::m11::*;
pub use sets::m15::*;
pub use sets::mh3::*;
pub use sets::mh3b::*;
pub use sets::mh3c::*;
pub use sets::mh3d::*;
pub use sets::mh3e::*;
pub use sets::mkm::*;
pub use sets::mkm2::*;
pub use sets::mod_set::*;
pub use sets::bfz::*;
pub use sets::zen2::*;
pub use sets::zen3::*;
pub use sets::wwk::*;
pub use sets::wwk2::*;
pub use sets::ogw::*;
pub use sets::one::*;
pub use sets::pc2::*;
pub use sets::por::*;
pub use sets::rav::*;
pub use sets::rna::*;
pub use sets::rna2::*;
pub use sets::rtr::*;
pub use sets::shm::*;
pub use sets::sos::*;
pub use sets::exo::*;
pub use sets::exo2::*;
pub use sets::mir::*;
pub use sets::mir2::*;
pub use sets::mir3::*;
pub use sets::mir4::*;
pub use sets::mir5::*;
pub use sets::vis::*;
pub use sets::csp::*;
pub use sets::vis2::*;
pub use sets::wth::*;
pub use sets::wth2::*;
pub use sets::sth::*;
pub use sets::stx::*;
pub use sets::thb::*;
pub use sets::jud2::*;
pub use sets::lgn::*;
pub use sets::arc::*;
pub use sets::ante::*;
pub use sets::leg::*;
pub use sets::leg2::*;
pub use sets::leg3::*;
pub use sets::leg4::*;
pub use sets::leg5::*;
pub use sets::leg6::*;
pub use sets::leg7::*;
pub use sets::atq::*;
pub use sets::clb::*;
pub use sets::cns::*;
pub use sets::cns2::*;
pub use sets::cn2::*;
pub use sets::cns3::*;
pub use sets::drk::*;
pub use sets::drk2::*;
pub use sets::ohop::*;
pub use sets::hml::*;
pub use sets::hml2::*;
pub use sets::hml3::*;
pub use sets::mbs::*;
pub use sets::nph::*;
pub use sets::scg::*;
pub use sets::scg2::*;
pub use sets::ons::*;
pub use sets::ons2::*;
pub use sets::ons3::*;
pub use sets::ons4::*;
pub use sets::vanguard::*;
pub use sets::jud::*;
pub use sets::tor::*;
pub use sets::tor2::*;
pub use sets::ths::*;
pub use sets::tmp::*;
pub use sets::war::*;
pub use sets::xtra::*;
pub use sets::zen::*;

/// A zero-arg factory that produces a fresh `CardDefinition`. The name→factory
/// registry (in the top-level `crabomination` crate) is built from these.
pub use crabomination_catalog_core::CardFactory;
