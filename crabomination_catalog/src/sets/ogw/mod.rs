//! Oath of the Gatewatch (OGW) — 2016

pub use super::no_abilities;

mod creatures;
mod instants;

pub use creatures::*;
pub use instants::*;

/// Every OGW/Eldrazi factory, for the snapshot name→factory registry and the
/// client art prefetch.
pub fn all_ogw_card_factories() -> &'static [crate::CardFactory] {
    &[
        stormchaser_mage,
        mist_intruder,
        breaker_of_armies,
        eldrazi_devastator,
        warden_of_geometries,
        cultivator_drone,
        salvage_drone,
        skitterskin,
        mindmelter,
        deepfathom_skulker,
        culling_drone,
        benthic_infiltrator,
        maw_of_kozilek,
        voracious_null,
        vile_aggregate,
        dread_drone,
        slaughter_drone,
        kozileks_channeler,
        scion_summoner,
        brood_monitor,
        eldrazi_skyspawner,
        incubator_drone,
        eyeless_watcher,
        blisterpod,
        catacomb_sifter,
        ulamog_the_infinite_gyre,
        kozilek_butcher_of_truth,
        pathrazer_of_ulamog,
        ulamogs_crusher,
        artisan_of_kozilek,
        desolation_twin,
        hand_of_emrakul,
        bane_of_bala_ged,
        birthing_hulk,
        drowner_of_hope,
        kozileks_shrieker,
        sifter_of_skulls,
        pawn_of_ulamog,
        vestige_of_emrakul,
        stalking_drone,
        nettle_drone,
        ruination_guide,
        dominator_drone,
        blinding_drone,
        kozileks_translator,
        flayer_drone,
        kozileks_sentinel,
        spawnsire_of_ulamog,
        matter_reshaper,
        wasteland_strangler,
        mind_raker,
        blight_herder,
        sludge_crawler,
        murderous_compulsion,
        sweep_away,
        warping_wail,
        tar_snare,
        witness_the_end,
        scour_from_existence,
        kozileks_return,
        oblivion_strike,
        complete_disregard,
        spatial_contortion,
        unnatural_endurance,
        call_the_scions,
        reality_hemorrhage,
        touch_of_the_void,
    ]
}
