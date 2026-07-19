//! Functionality tests for the Modern-supplement card pack
//! (`catalog::sets::decks::modern`). One integration-test binary; each card gets
//! at least one test exercising its primary play pattern.

/// Card-factory fn pointer for parametrized case tables.
#[allow(unused)]
pub type Factory = fn() -> crabomination::card::CardDefinition;

mod cantrips_suspend;
mod creatures_madness_lands;
mod bloomburrow_tribal;
mod cube_rounds;
mod supplement_batches;
mod removal_bounce_tokens;
mod decks_08_10;
mod decks_11_13;
mod decks_14_16;
mod push_batches;
mod cube_expansion_singles;
mod singles_and_legends;
mod decks_16_17_misc;
mod lands_equipment_vehicles;
mod cascade_dredge_auras;
mod coverage_backfill;
mod devotion_theros;
mod explore_bestow_batches;
mod theros_fading_adventure;
mod maybeboard_soulbond;
mod painlands_statics;
mod altars_flips_artifacts;
mod aggro_allied_batches;
mod overrun_phasing_forecast;
mod tribal_value_miracle;
mod tribal_cheats_manifest;
mod channel_cipher_staples;
mod librarytop_rooms;
mod cycling_meld_eldrazi;
mod shadow_kicker_mill;
mod scam_znr_epic;
mod staples_2026;
mod slivers_faeries_cleave;
mod staples_june_mutate;
mod gift_tdm;
