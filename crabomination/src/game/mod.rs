//! Core MTG game engine.
//!
//! # Design notes
//! - **Stack & priority**: a real LIFO `stack` of spells and abilities, drained
//!   by a multiplayer priority loop (`pass_priority`). Spells and abilities are
//!   pushed onto the stack and resolve only once all players pass in succession;
//!   players can respond while they hold priority.
//! - **Combat damage**: first-strike and regular combat damage are split into
//!   separate sub-steps (the `FirstStrikeDamage` step, skipped when no
//!   first/double-strike creatures are in combat). A DoubleStrike creature
//!   deals damage in both steps.
//! - **Hexproof/Shroud**: validated at targeting time.
//! - **Menace**: enforced in `declare_blockers` — an attacker with Menace must
//!   be blocked by ≥ 2 creatures or not blocked at all.
//! - **Dies triggers**: fired when a creature moves from battlefield to
//!   graveyard (via damage, destroy, or state-based actions).
//! - Actions are performed by whichever player currently holds priority (so a
//!   non-active player can cast instants / activate abilities in response);
//!   `declare_blockers` is called by whoever controls the defending creatures.

pub(crate) mod actions;
pub(crate) mod affordances;
pub(crate) mod combat;
pub(crate) mod effects;
pub mod layers;
pub(crate) mod stack;
#[cfg(test)]
#[path = "../tests/game.rs"]
mod tests;
#[cfg(test)]
#[path = "../tests/modern.rs"]
mod tests_modern;
#[cfg(test)]
#[path = "../tests/c21.rs"]
mod tests_c21;
#[cfg(test)]
#[path = "../tests/recent.rs"]
mod tests_recent;
#[cfg(test)]
#[path = "../tests/freerunning.rs"]
mod tests_freerunning;
#[cfg(test)]
#[path = "../tests/recent2.rs"]
mod tests_recent2;
#[cfg(test)]
#[path = "../tests/recent3.rs"]
mod tests_recent3;
#[cfg(test)]
#[path = "../tests/recent4.rs"]
mod tests_recent4;
#[cfg(test)]
#[path = "../tests/recent5.rs"]
mod tests_recent5;
#[cfg(test)]
#[path = "../tests/recent6.rs"]
mod tests_recent6;
#[cfg(test)]
#[path = "../tests/recent7.rs"]
mod tests_recent7;
#[cfg(test)]
#[path = "../tests/recent8.rs"]
mod tests_recent8;
#[cfg(test)]
#[path = "../tests/recent9.rs"]
mod tests_recent9;
#[cfg(test)]
#[path = "../tests/recent10.rs"]
mod tests_recent10;
#[cfg(test)]
#[path = "../tests/recent11.rs"]
mod tests_recent11;
#[cfg(test)]
#[path = "../tests/recent12.rs"]
mod tests_recent12;
#[cfg(test)]
#[path = "../tests/recent13.rs"]
mod tests_recent13;
#[cfg(test)]
#[path = "../tests/recent14.rs"]
mod tests_recent14;
#[cfg(test)]
#[path = "../tests/recent15.rs"]
mod tests_recent15;
#[cfg(test)]
#[path = "../tests/recent16.rs"]
mod tests_recent16;
#[cfg(test)]
#[path = "../tests/recent17.rs"]
mod tests_recent17;
#[cfg(test)]
#[path = "../tests/recent18.rs"]
mod tests_recent18;
#[cfg(test)]
#[path = "../tests/recent19.rs"]
mod tests_recent19;
#[cfg(test)]
#[path = "../tests/recent20.rs"]
mod tests_recent20;
#[cfg(test)]
#[path = "../tests/recent21.rs"]
mod tests_recent21;
#[cfg(test)]
#[path = "../tests/recent22.rs"]
mod tests_recent22;
#[cfg(test)]
#[path = "../tests/recent23.rs"]
mod tests_recent23;
#[cfg(test)]
#[path = "../tests/recent24.rs"]
mod tests_recent24;
#[cfg(test)]
#[path = "../tests/recent25.rs"]
mod tests_recent25;
#[cfg(test)]
#[path = "../tests/recent26.rs"]
mod tests_recent26;
#[cfg(test)]
#[path = "../tests/recent27.rs"]
mod tests_recent27;
#[cfg(test)]
#[path = "../tests/recent28.rs"]
mod tests_recent28;
#[cfg(test)]
#[path = "../tests/recent29.rs"]
mod tests_recent29;
#[cfg(test)]
#[path = "../tests/recent30.rs"]
mod tests_recent30;
#[cfg(test)]
#[path = "../tests/recent31.rs"]
mod tests_recent31;
#[cfg(test)]
#[path = "../tests/recent32.rs"]
mod tests_recent32;
#[cfg(test)]
#[path = "../tests/recent33.rs"]
mod tests_recent33;
#[cfg(test)]
#[path = "../tests/recent34.rs"]
mod tests_recent34;
#[cfg(test)]
#[path = "../tests/recent35.rs"]
mod tests_recent35;
#[cfg(test)]
#[path = "../tests/recent36.rs"]
mod tests_recent36;
#[cfg(test)]
#[path = "../tests/recent37.rs"]
mod tests_recent37;
#[cfg(test)]
#[path = "../tests/recent38.rs"]
mod tests_recent38;
#[cfg(test)]
#[path = "../tests/recent39.rs"]
mod tests_recent39;
#[cfg(test)]
#[path = "../tests/recent40.rs"]
mod tests_recent40;
#[cfg(test)]
#[path = "../tests/recent41.rs"]
mod tests_recent41;
#[cfg(test)]
#[path = "../tests/recent42.rs"]
mod tests_recent42;
#[cfg(test)]
#[path = "../tests/recent43.rs"]
mod tests_recent43;
#[cfg(test)]
#[path = "../tests/recent44.rs"]
mod tests_recent44;
#[cfg(test)]
#[path = "../tests/recent45.rs"]
mod tests_recent45;
#[cfg(test)]
#[path = "../tests/recent46.rs"]
mod tests_recent46;
#[cfg(test)]
#[path = "../tests/recent47.rs"]
mod tests_recent47;
#[cfg(test)]
#[path = "../tests/recent48.rs"]
mod tests_recent48;
#[cfg(test)]
#[path = "../tests/recent49.rs"]
mod tests_recent49;
#[cfg(test)]
#[path = "../tests/recent50.rs"]
mod tests_recent50;
#[cfg(test)]
#[path = "../tests/recent51.rs"]
mod tests_recent51;
#[cfg(test)]
#[path = "../tests/recent52.rs"]
mod tests_recent52;
#[cfg(test)]
#[path = "../tests/recent53.rs"]
mod tests_recent53;
#[cfg(test)]
#[path = "../tests/recent54.rs"]
mod tests_recent54;
#[cfg(test)]
#[path = "../tests/recent55.rs"]
mod tests_recent55;
#[cfg(test)]
#[path = "../tests/recent56.rs"]
mod tests_recent56;
#[cfg(test)]
#[path = "../tests/recent57.rs"]
mod tests_recent57;
#[cfg(test)]
#[path = "../tests/recent58.rs"]
mod tests_recent58;
#[cfg(test)]
#[path = "../tests/recent59.rs"]
mod tests_recent59;
#[cfg(test)]
#[path = "../tests/recent60.rs"]
mod tests_recent60;
#[cfg(test)]
#[path = "../tests/recent61.rs"]
mod tests_recent61;
#[cfg(test)]
#[path = "../tests/recent62.rs"]
mod tests_recent62;
#[cfg(test)]
#[path = "../tests/recent63.rs"]
mod tests_recent63;
#[cfg(test)]
#[path = "../tests/recent64.rs"]
mod tests_recent64;
#[cfg(test)]
#[path = "../tests/recent65.rs"]
mod tests_recent65;
#[cfg(test)]
#[path = "../tests/recent66.rs"]
mod tests_recent66;
#[cfg(test)]
#[path = "../tests/recent67.rs"]
mod tests_recent67;
#[cfg(test)]
#[path = "../tests/recent68.rs"]
mod tests_recent68;
#[cfg(test)]
#[path = "../tests/recent69.rs"]
mod tests_recent69;
#[cfg(test)]
#[path = "../tests/recent70.rs"]
mod tests_recent70;
#[cfg(test)]
#[path = "../tests/recent71.rs"]
mod tests_recent71;
#[cfg(test)]
#[path = "../tests/recent72.rs"]
mod tests_recent72;
#[cfg(test)]
#[path = "../tests/recent73.rs"]
mod tests_recent73;
#[cfg(test)]
#[path = "../tests/recent74.rs"]
mod tests_recent74;
#[cfg(test)]
#[path = "../tests/recent75.rs"]
mod tests_recent75;
#[cfg(test)]
#[path = "../tests/recent76.rs"]
mod tests_recent76;
#[cfg(test)]
#[path = "../tests/recent77.rs"]
mod tests_recent77;
#[cfg(test)]
#[path = "../tests/recent78.rs"]
mod tests_recent78;
#[cfg(test)]
#[path = "../tests/recent79.rs"]
mod tests_recent79;
#[cfg(test)]
#[path = "../tests/recent80.rs"]
mod tests_recent80;
#[cfg(test)]
#[path = "../tests/recent81.rs"]
mod tests_recent81;
#[cfg(test)]
#[path = "../tests/recent82.rs"]
mod tests_recent82;
#[cfg(test)]
#[path = "../tests/recent83.rs"]
mod tests_recent83;

#[cfg(test)]
#[path = "../tests/recent84.rs"]
mod tests_recent84;

#[cfg(test)]
#[path = "../tests/recent85.rs"]
mod tests_recent85;

#[cfg(test)]
#[path = "../tests/recent86.rs"]
mod tests_recent86;

#[cfg(test)]
#[path = "../tests/recent87.rs"]
mod tests_recent87;

#[cfg(test)]
#[path = "../tests/recent88.rs"]
mod tests_recent88;

#[cfg(test)]
#[path = "../tests/recent89.rs"]
mod tests_recent89;
#[cfg(test)]
#[path = "../tests/recent90.rs"]
mod tests_recent90;
#[cfg(test)]
#[path = "../tests/experience.rs"]
mod tests_experience;
#[cfg(test)]
#[path = "../tests/recent91.rs"]
mod tests_recent91;
#[cfg(test)]
#[path = "../tests/recent92.rs"]
mod tests_recent92;
#[cfg(test)]
#[path = "../tests/recent93.rs"]
mod tests_recent93;
#[cfg(test)]
#[path = "../tests/recent94.rs"]
mod tests_recent94;
#[cfg(test)]
#[path = "../tests/recent95.rs"]
mod tests_recent95;
#[cfg(test)]
#[path = "../tests/recent96.rs"]
mod tests_recent96;
#[cfg(test)]
#[path = "../tests/recent97.rs"]
mod tests_recent97;
#[cfg(test)]
#[path = "../tests/recent98.rs"]
mod tests_recent98;
#[cfg(test)]
#[path = "../tests/recent99.rs"]
mod tests_recent99;
#[cfg(test)]
#[path = "../tests/recent100.rs"]
mod tests_recent100;
#[cfg(test)]
#[path = "../tests/recent101.rs"]
mod tests_recent101;
#[cfg(test)]
#[path = "../tests/afr.rs"]
mod tests_afr;
#[cfg(test)]
#[path = "../tests/recent102.rs"]
mod tests_recent102;
#[cfg(test)]
#[path = "../tests/recent103.rs"]
mod tests_recent103;
#[cfg(test)]
#[path = "../tests/recent104.rs"]
mod tests_recent104;
#[cfg(test)]
#[path = "../tests/quests.rs"]
mod tests_quests;
#[cfg(test)]
#[path = "../tests/recent105.rs"]
mod tests_recent105;
#[cfg(test)]
#[path = "../tests/recent106.rs"]
mod tests_recent106;
#[cfg(test)]
#[path = "../tests/recent107.rs"]
mod tests_recent107;
#[cfg(test)]
#[path = "../tests/recent108.rs"]
mod tests_recent108;
#[cfg(test)]
#[path = "../tests/recent109.rs"]
mod tests_recent109;
#[cfg(test)]
#[path = "../tests/recent110.rs"]
mod tests_recent110;
#[cfg(test)]
#[path = "../tests/recent111.rs"]
mod tests_recent111;
#[cfg(test)]
#[path = "../tests/recent112.rs"]
mod tests_recent112;
#[cfg(test)]
#[path = "../tests/recent113.rs"]
mod tests_recent113;
#[cfg(test)]
#[path = "../tests/recent114.rs"]
mod tests_recent114;
#[cfg(test)]
#[path = "../tests/mh2b.rs"]
mod tests_mh2b;
#[cfg(test)]
#[path = "../tests/mh2c.rs"]
mod tests_mh2c;
#[cfg(test)]
#[path = "../tests/mh2d.rs"]
mod tests_mh2d;
#[cfg(test)]
#[path = "../tests/mh2e.rs"]
mod tests_mh2e;
#[cfg(test)]
#[path = "../tests/mh2f.rs"]
mod tests_mh2f;
#[cfg(test)]
#[path = "../tests/mh2g.rs"]
mod tests_mh2g;
#[cfg(test)]
#[path = "../tests/mh2h.rs"]
mod tests_mh2h;
#[cfg(test)]
#[path = "../tests/mh2i.rs"]
mod tests_mh2i;
#[cfg(test)]
#[path = "../tests/echo.rs"]
mod tests_echo;
#[cfg(test)]
#[path = "../tests/abilitywords.rs"]
mod tests_abilitywords;
#[cfg(test)]
#[path = "../tests/spree.rs"]
mod tests_spree;
#[cfg(test)]
#[path = "../tests/avatar_water.rs"]
mod tests_avatar_water;
#[cfg(test)]
#[path = "../tests/catalog_registration.rs"]
mod tests_catalog_registration;
#[cfg(test)]
#[path = "../tests/lci_craft.rs"]
mod tests_lci_craft;
#[cfg(test)]
#[path = "../tests/lci.rs"]
mod tests_lci;
#[cfg(test)]
#[path = "../tests/blight.rs"]
mod tests_blight;
#[cfg(test)]
#[path = "../tests/tla.rs"]
mod tests_tla;
#[cfg(test)]
#[path = "../tests/mom.rs"]
mod tests_mom;
#[cfg(test)]
#[path = "../tests/eoe.rs"]
mod tests_eoe;
#[cfg(test)]
#[path = "../tests/one.rs"]
mod tests_one;
#[cfg(test)]
#[path = "../tests/ltr.rs"]
mod tests_ltr;
#[cfg(test)]
#[path = "../tests/fin.rs"]
mod tests_fin;
#[cfg(test)]
#[path = "../tests/innistrad.rs"]
mod tests_innistrad;
#[cfg(test)]
#[path = "../tests/sos.rs"]
mod tests_sos;
#[cfg(test)]
#[path = "../tests/stx/mod.rs"]
mod tests_stx;
#[cfg(test)]
#[path = "../tests/multiplayer.rs"]
mod tests_multiplayer;
#[cfg(test)]
#[path = "../tests/xtra.rs"]
mod tests_xtra;
#[cfg(test)]
#[path = "../tests/combat_keywords.rs"]
mod tests_combat_keywords;
#[cfg(test)]
#[path = "../tests/classic.rs"]
mod tests_classic;
#[cfg(test)]
#[path = "../tests/counters.rs"]
mod tests_counters;
#[cfg(test)]
#[path = "../tests/energy.rs"]
mod tests_energy;
#[cfg(test)]
#[path = "../tests/ktk.rs"]
mod tests_ktk;
#[cfg(test)]
#[path = "../tests/akh.rs"]
mod tests_akh;
#[cfg(test)]
#[path = "../tests/mkm.rs"]
mod tests_mkm;
#[cfg(test)]
#[path = "../tests/mh3.rs"]
mod tests_mh3;
#[cfg(test)]
#[path = "../tests/mh3b.rs"]
mod tests_mh3b;
#[cfg(test)]
#[path = "../tests/mh3c.rs"]
mod tests_mh3c;
#[cfg(test)]
#[path = "../tests/mh3d.rs"]
mod tests_mh3d;
#[cfg(test)]
#[path = "../tests/mh3e.rs"]
mod tests_mh3e;
#[cfg(test)]
#[path = "../tests/recent115.rs"]
mod tests_recent115;
#[cfg(test)]
#[path = "../tests/recent116.rs"]
mod tests_recent116;
#[cfg(test)]
#[path = "../tests/recent117.rs"]
mod tests_recent117;
#[cfg(test)]
#[path = "../tests/recent118.rs"]
mod tests_recent118;
#[cfg(test)]
#[path = "../tests/recent119.rs"]
mod tests_recent119;
#[cfg(test)]
#[path = "../tests/recent120.rs"]
mod tests_recent120;
#[cfg(test)]
#[path = "../tests/recent121.rs"]
mod tests_recent121;
#[cfg(test)]
#[path = "../tests/recent122.rs"]
mod tests_recent122;
#[cfg(test)]
#[path = "../tests/recent123.rs"]
mod tests_recent123;
#[cfg(test)]
#[path = "../tests/recent124.rs"]
mod tests_recent124;
#[cfg(test)]
#[path = "../tests/recent125.rs"]
mod tests_recent125;
#[cfg(test)]
#[path = "../tests/recent126.rs"]
mod tests_recent126;
#[cfg(test)]
#[path = "../tests/recent127.rs"]
mod tests_recent127;
#[cfg(test)]
#[path = "../tests/recent128.rs"]
mod tests_recent128;
#[cfg(test)]
#[path = "../tests/recent129.rs"]
mod tests_recent129;
#[cfg(test)]
#[path = "../tests/recent130.rs"]
mod tests_recent130;

#[cfg(test)]
#[path = "../tests/recent131.rs"]
mod tests_recent131;

#[cfg(test)]
#[path = "../tests/recent132.rs"]
mod tests_recent132;

#[cfg(test)]
#[path = "../tests/recent133.rs"]
mod tests_recent133;

#[cfg(test)]
#[path = "../tests/recent134.rs"]
mod tests_recent134;
#[cfg(test)]
#[path = "../tests/recent135.rs"]
mod tests_recent135;
#[cfg(test)]
#[path = "../tests/recent136.rs"]
mod tests_recent136;
#[cfg(test)]
#[path = "../tests/recent137.rs"]
mod tests_recent137;
#[cfg(test)]
#[path = "../tests/recent138.rs"]
mod tests_recent138;
#[cfg(test)]
#[path = "../tests/recent139.rs"]
mod tests_recent139;
#[cfg(test)]
#[path = "../tests/recent140.rs"]
mod tests_recent140;
#[cfg(test)]
#[path = "../tests/recent141.rs"]
mod tests_recent141;
#[cfg(test)]
#[path = "../tests/recent142.rs"]
mod tests_recent142;
#[cfg(test)]
#[path = "../tests/recent143.rs"]
mod tests_recent143;
#[cfg(test)]
#[path = "../tests/recent144.rs"]
mod tests_recent144;
#[cfg(test)]
#[path = "../tests/recent145.rs"]
mod tests_recent145;
#[cfg(test)]
#[path = "../tests/recent146.rs"]
mod tests_recent146;
#[cfg(test)]
#[path = "../tests/recent147.rs"]
mod tests_recent147;
#[cfg(test)]
#[path = "../tests/recent148.rs"]
mod tests_recent148;
#[cfg(test)]
#[path = "../tests/recent149.rs"]
mod tests_recent149;
#[cfg(test)]
#[path = "../tests/recent150.rs"]
mod tests_recent150;
#[cfg(test)]
#[path = "../tests/recent151.rs"]
mod tests_recent151;
#[cfg(test)]
#[path = "../tests/recent152.rs"]
mod tests_recent152;
#[cfg(test)]
#[path = "../tests/recent153.rs"]
mod tests_recent153;
#[cfg(test)]
#[path = "../tests/recent154.rs"]
mod tests_recent154;
#[cfg(test)]
#[path = "../tests/recent155.rs"]
mod tests_recent155;
#[cfg(test)]
#[path = "../tests/recent156.rs"]
mod tests_recent156;
#[cfg(test)]
#[path = "../tests/recent157.rs"]
mod tests_recent157;
#[cfg(test)]
#[path = "../tests/recent158.rs"]
mod tests_recent158;
#[cfg(test)]
#[path = "../tests/recent159.rs"]
mod tests_recent159;
#[cfg(test)]
#[path = "../tests/recent160.rs"]
mod tests_recent160;
#[cfg(test)]
#[path = "../tests/recent161.rs"]
mod tests_recent161;
#[cfg(test)]
#[path = "../tests/recent165.rs"]
mod tests_recent165;
#[cfg(test)]
#[path = "../tests/recent166.rs"]
mod tests_recent166;
#[cfg(test)]
#[path = "../tests/spm.rs"]
mod tests_spm;
#[cfg(test)]
#[path = "../tests/recent167.rs"]
mod tests_recent167;
#[cfg(test)]
#[path = "../tests/recent168.rs"]
mod tests_recent168;
#[cfg(test)]
#[path = "../tests/recent169.rs"]
mod tests_recent169;
#[cfg(test)]
#[path = "../tests/recent170.rs"]
mod tests_recent170;
#[cfg(test)]
#[path = "../tests/recent171.rs"]
mod tests_recent171;
#[cfg(test)]
#[path = "../tests/recent172.rs"]
mod tests_recent172;
#[cfg(test)]
#[path = "../tests/recent173.rs"]
mod tests_recent173;
#[cfg(test)]
#[path = "../tests/recent174.rs"]
mod tests_recent174;
#[cfg(test)]
#[path = "../tests/recent175.rs"]
mod tests_recent175;
#[cfg(test)]
#[path = "../tests/recent164.rs"]
mod tests_recent164;
#[cfg(test)]
#[path = "../tests/recent163.rs"]
mod tests_recent163;
#[cfg(test)]
#[path = "../tests/recent162.rs"]
mod tests_recent162;
#[cfg(test)]
#[path = "../tests/ogw.rs"]
mod tests_ogw;
#[cfg(test)]
#[path = "../tests/cr_rules.rs"]
mod tests_cr_rules;
#[cfg(test)]
#[path = "../tests/thb.rs"]
mod tests_thb;
#[cfg(test)]
#[path = "../tests/chk.rs"]
mod tests_chk;
#[cfg(test)]
#[path = "../tests/bro.rs"]
mod tests_bro;
#[cfg(test)]
#[path = "../tests/gpt.rs"]
mod tests_gpt;
pub mod types;

#[cfg(test)]
pub(crate) fn two_player_game() -> GameState {
    multi_player_game(2)
}

/// `n`-player game (n ≥ 1), pre-advanced to the active player's pre-combat
/// main phase. Players are named "P0", "P1", …. Use for free-for-all
/// multiplayer tests; for format-specific life totals call
/// `game_with_format(format, n)`.
#[cfg(test)]
pub(crate) fn multi_player_game(n: usize) -> GameState {
    let players: Vec<_> = (0..n)
        .map(|i| crate::player::Player::new(i, format!("P{i}")))
        .collect();
    let mut g = GameState::new(players);
    g.step = TurnStep::PreCombatMain;
    g
}

/// `n`-player game with format-specific setup applied (starting life, draw-on-
/// turn-1 rule). Pre-advanced to the pre-combat main phase like
/// `two_player_game`.
#[cfg(test)]
pub(crate) fn game_with_format(format: crate::format::Format, n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.apply_format(format);
    g
}

/// Pass priority for both players until the stack is empty, returning all
/// events produced during resolution. Callers that don't care about events
/// can simply discard the return value.
#[cfg(test)]
pub(crate) fn drain_stack(g: &mut GameState) -> Vec<GameEvent> {
    let mut all_events = Vec::new();
    while !g.stack.is_empty() {
        all_events.extend(g.perform_action(GameAction::PassPriority).unwrap());
        all_events.extend(g.perform_action(GameAction::PassPriority).unwrap());
    }
    all_events
}

/// Cast a spell with no target and drain the stack. Returns resolve events.
/// Tests with non-default `mode`/`x_value`, the error path, or that need to
/// inspect cast-time events separately should use `GameAction::CastSpell`
/// directly.
#[cfg(test)]
pub(crate) fn cast(g: &mut GameState, id: CardId) -> Vec<GameEvent> {
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast spell");
    drain_stack(g)
}

/// Cast a spell at a specific target and drain the stack.
#[cfg(test)]
pub(crate) fn cast_at(g: &mut GameState, id: CardId, target: Target) -> Vec<GameEvent> {
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(target), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast spell at target");
    drain_stack(g)
}

pub use types::*;

use crate::card::{CardDefinition, CardId, CardInstance, CardType, Keyword, SelectionRequirement};
use crate::decision::{AutoDecider, Decider, DeciderKind, Decision, DecisionAnswer};
use crate::effect::Effect;
use crate::game::effects::EffectContext;
use crate::game::layers::{
    AffectedPermanents, ComputedPermanent, ContinuousEffect, EffectDuration, Layer, Modification,
    PtSublayer,
};
use crate::player::Player;
use std::collections::HashMap;

// ── Decider serde adapter ────────────────────────────────────────────────────
//
// `Box<dyn Decider>` can't directly derive serde, so we project it to
// `DeciderKind` (which IS serializable) on the wire and reconstitute on
// load. Custom deciders not modeled by the kind enum collapse to
// `AutoDecider` after a round-trip.

#[allow(clippy::borrowed_box)] // serde derive needs `&Box<T>` here
fn serialize_decider<S: serde::Serializer>(
    decider: &Box<dyn Decider + Send + Sync>,
    ser: S,
) -> Result<S::Ok, S::Error> {
    use serde::Serialize;
    decider.kind().serialize(ser)
}

fn deserialize_decider<'de, D: serde::Deserializer<'de>>(
    de: D,
) -> Result<Box<dyn Decider + Send + Sync>, D::Error> {
    use serde::Deserialize;
    let kind = DeciderKind::deserialize(de)?;
    Ok(kind.into_boxed())
}

// ── Game state ────────────────────────────────────────────────────────────────

/// Interior-mutable memo for [`GameState::with_frozen_layers`]: the gathered
/// continuous-effect set, shared via `Arc` so per-permanent layer passes
/// don't re-clone it. `Mutex` (not `RefCell`) keeps `GameState: Sync` for the
/// server's `Arc<GameState>` snapshot sink. Clones reset to unfrozen — a bot
/// dry-run clone taken inside a freeze scope mutates, so it must re-gather.
#[derive(Default)]
pub(crate) struct LayerFreeze(std::sync::Mutex<LayerFreezeState>);

#[derive(Default)]
struct LayerFreezeState {
    /// Nesting depth of active `with_frozen_layers` scopes; 0 = unfrozen.
    depth: u32,
    /// Lazily-gathered effect set, populated on the first computed read
    /// inside a scope and cleared when the outermost scope exits.
    memo: Option<std::sync::Arc<Vec<ContinuousEffect>>>,
}

impl LayerFreeze {
    fn lock(&self) -> std::sync::MutexGuard<'_, LayerFreezeState> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

impl Clone for LayerFreeze {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// Every from-hand affordance hint for one seat, produced in a single sweep
/// by [`GameState::compute_hand_affordances`]. Each field is the set of
/// CardIds the client should highlight for that affordance; the view layer
/// copies them straight into the matching `ClientView` fields. All-empty
/// when the seat doesn't currently hold priority.
#[derive(Debug, Clone, Default)]
pub struct HandAffordances {
    pub castable: Vec<CardId>,
    pub pitchable: Vec<CardId>,
    pub kickable: Vec<CardId>,
    pub buyback: Vec<CardId>,
    pub bestowable: Vec<CardId>,
    pub dashable: Vec<CardId>,
    pub blitzable: Vec<CardId>,
    /// EOE — hand cards castable for their Warp cost right now.
    pub warpable: Vec<CardId>,
    pub suspendable: Vec<CardId>,
    pub foretellable: Vec<CardId>,
    pub plottable: Vec<CardId>,
    pub adventurable: Vec<CardId>,
    /// CR 702.183 — hand cards with an Omen half castable right now.
    pub omenable: Vec<CardId>,
    /// CR 709 — split cards whose **right** half is castable right now.
    pub splittable_right: Vec<CardId>,
    /// CR 702.176 — hand cards with Bargain that are castable right now, so the
    /// client can offer a "sacrifice an artifact/enchantment/token?" toggle.
    pub bargainable: Vec<CardId>,
    /// CR 702.157 — hand cards with Squad castable paying the squad cost at
    /// least once, so the client can offer a "pay Squad N times?" stepper.
    pub squadable: Vec<CardId>,
    /// CR 702.172 — hand Spree cards castable choosing at least the cheapest
    /// mode right now, so the client can offer the per-mode cost picker.
    pub spreeable: Vec<CardId>,
    /// CR 702.107 — hand cards with Replicate castable paying the replicate
    /// cost at least once, so the client can offer a "replicate N times?" stepper.
    pub replicatable: Vec<CardId>,
    /// CR 702.78 — hand cards with Conspire castable right now while the seat
    /// controls two untapped creatures sharing a color with the spell, so the
    /// client can offer the "tap two to copy" toggle.
    pub conspirable: Vec<CardId>,
    /// CR 702.33c — hand cards with Multikicker castable paying the kicker
    /// cost at least once, so the client can offer a "kick N times?" stepper.
    pub multikickable: Vec<CardId>,
    /// CR 702.94 — hand cards with a live Miracle window (revealed as the
    /// turn's first draw): castable for the cheaper miracle cost via
    /// `GameAction::CastFromZoneWithoutPaying`.
    pub miracle: Vec<CardId>,
    pub activatable_permanents: Vec<CardId>,
    /// Hand cards carrying at least one `from_hand` activated ability (Talon
    /// Gates of Madara's `{4}: put this onto the battlefield`, the Spirit
    /// Guides' exile-for-mana). Surfaced so the client/bot can offer the
    /// from-hand activation; affordability is re-checked by `activate_ability`.
    pub hand_activatable: Vec<CardId>,
    /// CR 702.36 — hand cards with Morph/Megamorph/Disguise castable face down
    /// for {3} right now, so the client can offer the "cast face down" action.
    pub morphable: Vec<CardId>,
    /// CR 708.5 — face-down permanents the seat controls whose turn-up cost
    /// (Morph/Megamorph/Disguise cost, or a manifested/cloaked creature card's
    /// mana cost) is payable right now, so the client can offer "turn face up".
    pub turn_up_able: Vec<CardId>,
    /// CR 702.77 — hand cards with a Reinforce ability whose cost is payable
    /// right now (a legal creature target exists), so the client can offer the
    /// from-hand Reinforce activation.
    pub reinforceable: Vec<CardId>,
    /// Hand cards with a `discard_activated` ability whose cost is payable
    /// right now (Magma Opus), so the client can offer the from-hand
    /// "discard: …" activation.
    pub discard_activatable: Vec<CardId>,
    /// CR 709.5 — Room hand cards whose left/right door is castable right
    /// now (`(card, door)` — door 0 = left, 1 = right).
    pub room_castable: Vec<(CardId, u8)>,
    /// CR 709.5e — Room permanents the seat controls with a locked door
    /// whose unlock cost is payable right now (`(card, door)`).
    pub room_unlockable: Vec<(CardId, u8)>,
    /// SOS Prepare — prepared creatures the seat controls whose prepare
    /// spell is castable right now (`GameAction::CastPrepareSpell` would
    /// be accepted: cost payable, timing legal).
    pub prepare_castable: Vec<CardId>,
    /// MDFCs whose **back face** is castable right now via
    /// `GameAction::CastSpellBack` — from hand, plus any in the graveyard
    /// carrying the one-shot `may_cast_back_from_graveyard` permission
    /// (Pestilent Cauldron). Complements `castable` (which only probes the
    /// front face) so back-affordable MDFCs still highlight and hold open
    /// priority windows.
    pub back_castable: Vec<CardId>,
    /// CR 702.160 — hand cards with Prototype castable for the prototype
    /// cost right now, so the client can offer "cast for prototype".
    pub prototypable: Vec<CardId>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameState {
    pub players: Vec<Player>,
    /// Partition of seats into teams. Every seat appears in exactly one
    /// entry; free-for-all formats have one singleton team per seat,
    /// team formats (Two-Headed Giant) have multiple seats per team.
    /// Populated by `GameState::new`; reshape with `assign_teams`.
    /// Defaults to empty for snapshots predating the field — helpers
    /// (`team_of`, `teammates`, `opponents_of`) treat empty as "each
    /// seat is its own singleton team".
    #[serde(default)]
    pub teams: Vec<crate::team::Team>,
    /// All permanents currently in play.
    pub battlefield: Vec<CardInstance>,
    /// CR 702.26 — permanents that have phased out. They're treated as though
    /// they don't exist (every battlefield query iterates `battlefield`, so a
    /// phased-out permanent is invisible without per-site filtering), yet
    /// retain all state (counters, attachments, damage). They phase back in
    /// during their controller's untap step (`do_phasing`).
    #[serde(default)]
    pub phased_out: Vec<CardInstance>,
    /// Cards that have been exiled.
    pub exile: Vec<CardInstance>,
    /// The stack of spells and triggered abilities waiting to resolve (LIFO).
    pub stack: Vec<StackItem>,
    pub step: TurnStep,
    /// Index into `players` of the player whose turn it is.
    pub active_player_idx: usize,
    pub turn_number: u32,
    /// `None` while the game is ongoing; `Some(None)` for a draw;
    /// `Some(Some(i))` when player `i` has won.
    pub game_over: Option<Option<usize>>,
    /// Priority state — tracks who can act and when the stack resolves.
    pub priority: PriorityState,
    /// Active continuous effects from resolved spells, abilities, and static abilities.
    pub continuous_effects: Vec<ContinuousEffect>,
    pub(crate) next_effect_timestamp: u64,
    pub(crate) next_id: u32,
    /// Attackers declared this combat, each with the player or planeswalker
    /// it is attacking.
    pub(crate) attacking: Vec<Attack>,
    /// Blocker → attacker mapping for the current combat.
    pub(crate) block_map: HashMap<CardId, CardId>,
    /// CR 510.1c — the active player's chosen blocker order for each attacker
    /// that has multiple blockers, gathered (and cached) before combat damage
    /// is applied so the choice can suspend for a `wants_ui` player. Read by
    /// `resolve_combat_damage_with_filter`; reset per damage step and cleared
    /// at combat end. `#[serde(skip)]` — transient, like `pending_decision`'s
    /// resume context (a mid-combat snapshot can't resume anyway).
    #[serde(skip, default)]
    pub(crate) combat_damage_order: HashMap<CardId, Vec<CardId>>,
    /// CR 510.1c-d — the active player's chosen combat-damage assignment
    /// `(blocker, amount)` for each multi-blocker attacker. Gathered alongside
    /// `combat_damage_order`. `#[serde(skip)]` for the same reason.
    #[serde(skip, default)]
    pub(crate) combat_damage_assignment: HashMap<CardId, Vec<(CardId, u32)>>,
    /// Which damage step (`FirstStrikeDamage` / `CombatDamage`) the cached
    /// combat-damage choices above belong to. Lets the gather pass reset the
    /// caches once when moving from the first-strike step to the regular step,
    /// without wiping them on a mid-step decision resume.
    #[serde(skip, default)]
    pub(crate) combat_damage_plan_step: Option<TurnStep>,
    /// Set to true once `declare_blockers` has been called during the current DeclareBlockers step.
    pub(crate) blockers_declared: bool,
    /// Skip the draw on the very first turn (turn 1, first player).
    pub(crate) skip_first_draw: bool,
    /// Count of spells cast this turn (for Storm and related effects).
    pub spells_cast_this_turn: u32,
    /// CR 702.29 — per-game tally of how many times a card with each name has
    /// been cycled (Yidaro, Wandering Monster's "four or more times this game"
    /// recursion). Keyed by card name; never reset. `#[serde(default)]`.
    #[serde(default)]
    pub(crate) cycled_count_by_name: std::collections::HashMap<String, u32>,
    /// CR 700.14 — running total of mana the active player has spent to
    /// cast spells this turn (Expend). Bumped in `finalize_cast` by each
    /// spell's `mana_spent`; reset at cleanup. `#[serde(default)]`.
    #[serde(default)]
    pub(crate) mana_spent_on_spells_this_turn: u32,
    /// CR 700.14 — the spell-mana total *before* the cost that produced
    /// the `Expended` event currently being dispatched. Read by
    /// `Predicate::ExpendReached` to detect threshold crossings.
    /// Transient scratch — `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) expend_prev_total: u32,
    /// Total spells cast during the previous turn (snapshotted from
    /// `spells_cast_this_turn` at Cleanup). Drives the classic Innistrad
    /// werewolf transform check ("if no spells were cast last turn …").
    /// `#[serde(default)]` so older snapshots load as 0.
    #[serde(default)]
    pub spells_cast_last_turn: u32,
    /// CR 702.69 — count of permanents put into a graveyard from the
    /// battlefield this turn (any controller, any type). Drives Gravestorm
    /// copy counts; reset at each turn's untap step.
    #[serde(default)]
    pub permanents_to_graveyard_this_turn: u32,
    /// Cards put into a graveyard **from the battlefield** this turn (CR —
    /// Second Sunrise's restore set). Cleared at cleanup.
    #[serde(default)]
    pub graveyard_from_battlefield_this_turn: std::collections::HashSet<CardId>,
    /// Cards that entered the battlefield from a graveyard — or were cast
    /// from one — this turn. Stamped at the gy→battlefield move funnel and
    /// at every cast-from-graveyard site; read by
    /// `SelectionRequirement::EnteredFromGraveyardThisTurn` (Prized
    /// Amalgam's gate). Cleared at each turn's untap step.
    #[serde(default)]
    pub(crate) entered_from_graveyard_this_turn: std::collections::HashSet<CardId>,
    /// Permanents that entered the battlefield directly from exile (not via a
    /// cast) this turn. Set in the exile→battlefield move path; read by
    /// `Predicate::EnteredFromExile` (Fire Lord Zuko's "whenever a permanent
    /// you control enters from exile"). Cleared at each turn's untap step.
    #[serde(default)]
    pub(crate) entered_from_exile_this_turn: std::collections::HashSet<CardId>,
    /// Delayed triggered abilities registered by resolved spells/abilities
    /// (Pact upkeep cost, Goryo's exile-at-EOT, etc.). Fired by the step
    /// dispatcher when the matching event occurs.
    pub delayed_triggers: Vec<DelayedTrigger>,
    /// Tokens minted by `Effect::CreateTokenAttacking` with a non-`None`
    /// cleanup (Mobilize sacrifice / Myriad exile). Drained when the combat
    /// phase ends (CR 511.3).
    #[serde(default)]
    pub(crate) attacking_token_cleanup: Vec<(CardId, crate::effect::AttackingTokenCleanup)>,
    /// Transient: power of the most recently sacrificed creature within the
    /// current effect resolution. Set by `Effect::SacrificeAndRemember` and
    /// read by `Value::SacrificedPower` (e.g. Thud). Reset between
    /// independent spell/ability resolutions.
    pub(crate) sacrificed_power: Option<i32>,
    /// Transient: toughness of the most recently sacrificed creature within
    /// the current effect resolution. Set by `Effect::SacrificeAndRemember`
    /// alongside `sacrificed_power`; read by `Value::SacrificedToughness`
    /// (Tribute to Hunger). Reset between independent resolutions.
    #[serde(default)]
    pub(crate) sacrificed_toughness: Option<i32>,
    /// Transient: mana value of the most-recently-sacrificed creature within
    /// the current effect/cost resolution. Set alongside `sacrificed_power`
    /// (including the `sac_other_filter` activation-cost path); read by
    /// `SelectionRequirement::ManaValueEqualsSacrificedPlus` (Birthing Pod).
    /// Reset between independent resolutions.
    #[serde(default)]
    pub(crate) sacrificed_mana_value: Option<u32>,
    /// Transient: whether the most-recently-sacrificed cost permanent was an
    /// artifact ("if the sacrificed permanent was an artifact" — Foundry
    /// Helix's `Predicate::SacrificedWasArtifact`). Set on the additional-
    /// cast-cost sacrifice path; reset between resolutions.
    #[serde(default)]
    pub(crate) sacrificed_was_artifact: Option<bool>,
    /// Transient: whether the most-recently-sacrificed cost permanent was an
    /// outlaw (Assassin/Mercenary/Pirate/Rogue/Warlock) — Boneyard Desecrator's
    /// `Predicate::SacrificedWasOutlaw`. Set on the sacrifice-cost paths; reset
    /// between resolutions.
    #[serde(default)]
    pub(crate) sacrificed_was_outlaw: Option<bool>,
    /// Transient: whether the most-recently-sacrificed cost permanent was a
    /// Vehicle — Hellish Sideswipe's `Predicate::SacrificedWasVehicle`.
    #[serde(default)]
    pub(crate) sacrificed_was_vehicle: Option<bool>,
    /// Transient: card-type count of the most recently discarded card
    /// (`Value::LastDiscardedCardTypes` — Mount Velus Manticore). Stamped in
    /// `discard_card`.
    #[serde(default)]
    pub(crate) last_discarded_card_types: u32,
    /// Mana value of the last card discarded during the current resolution
    /// (Argentum Masticore's "MV ≤ the discarded card" reflexive gate).
    pub(crate) last_discarded_mana_value: Option<u32>,
    /// "Whenever a creature blocks this turn, its controller gets N poison
    /// counters" (Noxious Assault). Cleared at cleanup.
    pub(crate) block_poison_this_turn: u32,
    /// Transient: power of the creature tapped to pay a Station ability's cost
    /// (CR 702.184a). Stamped by `Effect::WithTappedPower` at resolution; read
    /// by `Value::TappedForCostPower`. Reset between independent resolutions.
    #[serde(default)]
    pub(crate) tapped_for_cost_power: Option<i32>,
    /// Transient: the firing event's amount for the trigger currently being
    /// targeted or resolved (stamped in `drain_trigger_queue` and
    /// `continue_trigger_resolution_with_source`). For died events this is
    /// the dying card's mana value, read by
    /// `SelectionRequirement::ManaValueLessThanEventAmount` (Scrap Trawler).
    #[serde(default)]
    pub(crate) trigger_event_amount_scratch: u32,
    /// Transient: id of the most-recently-created token within the current
    /// effect resolution. Set by `Effect::CreateToken` and read by
    /// `Selector::LastCreatedToken` so a follow-up `AddCounter` /
    /// `PumpPT` / etc. in the same `Effect::Seq` can target the freshly
    /// minted token (Fractal Anomaly, Applied Geometry). Reset between
    /// independent resolutions.
    #[serde(skip)]
    pub(crate) last_created_token: Option<CardId>,
    /// CR 706.4 — the result of the most recent die roll, read by
    /// `Value::LastDieRoll`. Set by the `Effect::RollDie` resolver.
    #[serde(skip)]
    pub(crate) last_die_roll: u8,
    /// Transient generic cost-reduction folded into the next spell cast
    /// (CR 601.2f). Set by `cast_spell_sacrifice_reduce` to "{N} less per
    /// creature sacrificed" before delegating to the normal cast path, read
    /// in `cost_reduction_for_spell`, and cleared immediately after the cast.
    #[serde(skip)]
    pub(crate) extra_cast_reduction: u32,
    /// Transient: the cast in flight paid with Cavern-of-Souls-style
    /// restricted mana whose rider makes the spell uncounterable
    /// (`SpendRestriction::CreatureOfTypeUncounterable`). Set right after
    /// payment, consumed by `finalize_cast`.
    #[serde(skip)]
    pub(crate) cast_paid_uncounterable: bool,
    /// Transient: ids of all tokens created within the current effect
    /// resolution. Set by `Effect::CreateToken`
    /// alongside `last_created_token` and read by
    /// `Selector::LastCreatedTokens` (plural) so a follow-up `AddCounter`
    /// in the same resolution can fan over every freshly-minted token
    /// (Fractal Spawning, Mascot Exhibition-style printed Oracles). Cleared
    /// at every resolution root start (see `reset_effect_scratch`).
    #[serde(skip)]
    pub(crate) last_created_tokens: Vec<CardId>,
    /// Transient: ids of every card moved within the current effect
    /// resolution. Populated by `Effect::Move` (and the mill/exile
    /// helpers) and read by `Selector::LastMoved` so a follow-up
    /// `GrantMayPlay` in the same `Effect::Seq` can target exactly the
    /// card(s) that were just lifted to exile/graveyard (Practiced
    /// Scrollsmith, Suspend Aggression, Tablet of Discovery, etc.).
    /// Cleared between resolutions.
    #[serde(skip)]
    pub(crate) last_moved_cards: Vec<CardId>,
    /// Transient: count of cards discarded within the current effect
    /// resolution. Bumped by every `GameEvent::CardDiscarded` emission
    /// inside `Effect::Discard` / `Effect::DiscardChosen` (random and
    /// player-chosen branches). Read by `Value::CardsDiscardedThisEffect`
    /// so a later step in the same `Effect::Seq` can draw N where N =
    /// "the number of cards discarded this way" (Borrowed Knowledge
    /// mode 1, Colossus of the Blood Age, etc.). Reset to 0 between
    /// independent resolutions.
    #[serde(skip)]
    pub(crate) cards_discarded_this_resolution: u32,
    /// Transient: amount of {E} paid by `Effect::PayAnyEnergy` within the
    /// current resolution. Read by `Value::EnergyPaidThisEffect` so a later
    /// step in the same `Effect::Seq` can scale off "each {E} paid this way"
    /// (Aether Spike's counter-unless-pay-{1}-per-{E}). Reset between
    /// independent resolutions.
    #[serde(skip)]
    pub(crate) energy_paid_this_resolution: u32,
    /// Transient: count of *creature* cards discarded within the current
    /// effect resolution. Bumped alongside `cards_discarded_this_resolution`
    /// when the discarded card carries `CardType::Creature`. Read by
    /// `Value::CreatureCardsDiscardedThisEffect` so a follow-up step in
    /// the same `Effect::Seq` can fire only when a creature was discarded
    /// (Plargg, Dean of Chaos's printed conditional 2-damage rider).
    /// Reset to 0 between independent resolutions.
    #[serde(skip)]
    pub(crate) creature_cards_discarded_this_resolution: u32,
    /// Transient: per-player count of cards discarded within the current
    /// effect resolution, indexed by player seat. Bumped alongside the
    /// flat `cards_discarded_this_resolution` whenever a discard event
    /// fires, so a follow-up step in the same `Effect::Seq` can read the
    /// *greatest* count across players. Used by Windfall's printed
    /// "draws cards equal to the greatest number of cards a player
    /// discarded this way" via `Value::MaxCardsDiscardedThisEffectByAnyPlayer`.
    /// Reset to empty between independent resolutions.
    #[serde(skip)]
    pub(crate) cards_discarded_per_player_this_resolution: std::collections::HashMap<usize, u32>,
    /// Transient: per-player count of *nonland* cards discarded within the
    /// current effect resolution. Read by `Predicate::DiscardedNonlandThisEffect`
    /// — Kroxa's "each opponent who didn't discard a nonland card this way
    /// loses 3 life." Reset to empty between independent resolutions.
    #[serde(skip)]
    pub(crate) nonland_cards_discarded_per_player_this_resolution:
        std::collections::HashMap<usize, u32>,
    /// Transient: set by `Effect::ShuffleSelfIntoLibrary` during spell
    /// resolution; the post-resolution routing reads it to send the
    /// resolving spell to its owner's library (shuffled) instead of the
    /// graveyard. Cleared once consumed. Beacon cycle.
    #[serde(skip)]
    pub(crate) shuffle_resolving_spell_into_library: bool,
    /// `Effect::ReturnResolvingSpellToHand` — same shape, hand-bound.
    #[serde(skip)]
    pub(crate) return_resolving_spell_to_hand: bool,
    /// `Effect::ExileResolvingSpell` — same shape, exile-bound.
    #[serde(skip)]
    pub(crate) exile_resolving_spell: bool,
    /// CR 728 — set by `Effect::EndTheTurn`; consumed after the current
    /// stack item finishes resolving (exile the stack, clear combat, jump
    /// to cleanup).
    #[serde(skip)]
    pub(crate) end_turn_requested: bool,
    /// CR 702.46 — Cipher. Set by `Effect::Cipher` to the creature the
    /// resolving spell should be exiled "encoded on"; the post-resolution
    /// routing consumes it to send the card to exile (with `encoded_on` stamped)
    /// instead of the graveyard. Cleared once consumed.
    #[serde(skip)]
    pub(crate) cipher_encode_pending: Option<CardId>,
    /// CR 702.55 — Haunt. Set by `Effect::HauntCreature` while an instant/
    /// sorcery resolves to the creature it should haunt plus the haunt body;
    /// the post-resolution routing exiles the spell card (instead of the
    /// graveyard) and registers the `WhenHauntedCreatureDies` delayed trigger.
    /// Cleared once consumed. (Creature haunt is handled inline since the card
    /// is already in the graveyard when its dies-trigger resolves.)
    #[serde(skip)]
    pub(crate) haunt_pending: Option<(CardId, crate::effect::Effect)>,
    /// Transient: the `CardId`s of cards discarded within the current
    /// effect resolution. Populated alongside the count fields above. Used
    /// by Mind Roots's "Put up to one land card discarded this way onto
    /// the battlefield tapped" rider — the engine walks this list at
    /// resolution time, finds the first Land card, and moves it onto the
    /// battlefield via `Effect::MoveDiscardedLandToBattlefield`. Reset
    /// to empty between independent resolutions.
    #[serde(skip)]
    pub(crate) discarded_card_ids_this_resolution: Vec<CardId>,
    /// Transient: the `CardId`s of cards put into exile within the current
    /// effect resolution (any source zone, via `place_card_in_dest`). Powers
    /// `Selector::ExiledThisResolution` — "if you exiled a [type] card this
    /// way" payoffs (Bonehoard Dracosaur). Reset between resolutions.
    #[serde(skip)]
    pub(crate) exiled_card_ids_this_resolution: Vec<CardId>,
    /// Transient: count of permanents destroyed by `Effect::Destroy` within
    /// the current resolution. Read by `Value::PermanentsDestroyedThisResolution`
    /// so a follow-up `Effect::Seq` step can scale off the kill count
    /// (Culling Ritual's "Add {B} or {G} for each permanent destroyed this
    /// way"). Counts only permanents that actually reach the graveyard —
    /// indestructible / shielded survivors don't bump it. Reset to 0
    /// between independent resolutions.
    #[serde(skip)]
    pub(crate) permanents_destroyed_this_resolution: u32,
    /// Transient: total excess damage (CR 120.10) dealt during the current
    /// resolution — for each creature/planeswalker/battle, damage beyond what
    /// would be lethal/its loyalty/its defense. Read by
    /// `Predicate::ExcessDamageDealtThisResolution` to gate "if excess damage
    /// was dealt this way" riders (Orbital Plunge). Reset between resolutions.
    #[serde(skip)]
    pub(crate) excess_damage_this_resolution: u32,
    /// Transient: seats that sacrificed at least one permanent during the
    /// current resolution. Read by `Predicate::PlayerSacrificedThisResolution`
    /// so a follow-up step can gate on "if you sacrificed a permanent this way"
    /// (Deadly Brew). Reset between independent resolutions.
    #[serde(skip)]
    pub(crate) players_sacrificed_this_resolution: std::collections::HashSet<usize>,
    /// Transient: the card name chosen by an `Effect::NameCard` within the
    /// current resolution. Read by `SelectionRequirement::NamedBySource` so a
    /// reveal-until-the-named-card chain (Spoils of the Vault) can match even
    /// when the naming source is a resolving spell held off to the side.
    /// Reset between independent resolutions.
    #[serde(skip)]
    pub(crate) named_card_this_resolution: Option<String>,
    /// Transient: which face / cast path the in-progress cast is using.
    /// Set by `cast_spell_back_face` (`Back`) and `cast_flashback`
    /// (`Flashback`); reset to `Front` after each emitted SpellCast
    /// event. Threaded into `GameEvent::SpellCast.face` so replays can
    /// distinguish a back-face MDFC cast from a normal hand cast.
    #[serde(skip, default)]
    pub(crate) pending_cast_face: CastFace,
    /// Transient: the permanents a `wants_ui` caster picked to satisfy a
    /// "sacrifice a permanent" additional cast cost (CR 601.2b). Set by
    /// `submit_decision`'s `CastSacrifice` resume just before it re-invokes
    /// the cast, and consumed (taken) by `pay_additional_costs` in lieu of
    /// the auto-pick. `None` for the auto-pick path (bots/tests, or a single
    /// legal choice). Never needs to survive a snapshot — it lives only
    /// across the synchronous resume → cast call.
    #[serde(skip, default)]
    pub(crate) pending_cast_sacrifices: Option<Vec<CardId>>,
    /// Transient sibling of [`pending_cast_sacrifices`] for a spell's
    /// "as an additional cost, discard a card" requirement
    /// (`AdditionalCastCost::Discard` — Big Score, Illuminate History). The
    /// cards a `wants_ui` caster picked to discard; consumed by
    /// `pay_additional_costs` in lieu of the first-N auto-pick. Never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_cast_discards: Option<Vec<CardId>>,
    /// Transient: the Spree mode indices chosen for the current cast (CR
    /// 702.172). Set by `cast_spell_spree` just before it invokes the shared
    /// cast path, consumed there to fold the chosen modes' mana into the cost
    /// and stamp them onto the resolving `CardInstance.spree_modes`. Never
    /// snapshots — it lives only across the synchronous cast call.
    #[serde(skip, default)]
    pub(crate) pending_spree_modes: Option<Vec<u8>>,
    /// Transient: the answer to a "spend your floating mana, or tap lands
    /// instead?" confirmation (CR 601.2g — the player chooses their mana
    /// sources). `Some(true)` = spend the pre-existing float; `Some(false)` =
    /// keep it and pay from freshly-tapped sources. Set by `submit_decision`'s
    /// `CastFloatConfirm` resume just before it replays the cast, taken at the
    /// top of the cast. Never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_cast_spend_float: Option<bool>,
    /// SOS Prepare — copies materialized by `cast_prepare_spell` whose cast
    /// hasn't finished yet, as `(copy_id, source_creature_id)`. Registered
    /// before the copy enters the cast pipeline and settled by
    /// `settle_prepare_copy_cast` once the copy reaches the stack (flag it
    /// `is_token`, unprepare the creature) or the cast fails
    /// (unmaterialize the copy). Unlike its transient `pending_cast_*`
    /// siblings this must survive a snapshot: a mid-cast suspension
    /// (float-spend confirm, additional-cost pick) parks the copy in the
    /// caster's hand across a client round-trip.
    #[serde(default)]
    pub(crate) pending_prepare_copies: Vec<(CardId, CardId)>,
    /// Transient: the library card a `wants_ui` cycler picked for a
    /// landcycling / typecycling fetch (CR 702.29e). Set by the
    /// `ActionSearchPick` resume just before it replays the Landcycle
    /// action; consumed by `landcycle_card` in lieu of the first-match
    /// auto-pick. Inner `None` = fail to find. Never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_landcycle_pick: Option<Option<crate::card::CardId>>,
    /// Transient: the permanent a `wants_ui` activator picked to satisfy an
    /// activated ability's "Sacrifice another …" cost (`sac_other_filter`).
    /// Set by `submit_decision`'s `ActivateAbilityChoice` resume just before it
    /// replays `activate_ability`, and consumed there in lieu of the auto-pick.
    /// `None` for the auto-pick path. Like `pending_cast_sacrifices`, it lives
    /// only across the synchronous resume → activate call and never snapshots.
    #[serde(skip, default)]
    pub(crate) pending_ability_sac_other: Option<CardId>,
    /// Transient sibling of [`pending_ability_sac_other`] for an activated
    /// ability's "Tap an untapped … you control" cost (`tap_other_filter`).
    /// Set by the `ActivateAbilityChoice` resume, consumed by `activate_ability`.
    #[serde(skip, default)]
    pub(crate) pending_ability_tap_other: Option<CardId>,
    /// Transient sibling of [`pending_ability_sac_other`] for an activated
    /// ability's "Exile N cards from your graveyard" cost (`exile_other_filter`).
    /// Carries the full chosen set (the cost can exile several — Grim
    /// Lavamancer exiles two). Set by the `ActivateAbilityChoice` resume,
    /// consumed by `activate_ability`.
    #[serde(skip, default)]
    pub(crate) pending_ability_exile_other: Option<Vec<CardId>>,
    /// Resolves player choices encountered during effect resolution. Used for
    /// *non-suspending* decisions (e.g. `AddManaAnyColor` auto-picks a color).
    /// Suspending decisions (currently Scry) surface through `pending_decision`
    /// instead; the UI/bot replies via `submit_decision`.
    ///
    /// Serialized via the `decider_kind` adapter — see `DeciderKind` —
    /// so the trait object round-trips through JSON.
    #[serde(serialize_with = "serialize_decider", deserialize_with = "deserialize_decider")]
    pub decider: Box<dyn Decider + Send + Sync>,
    /// Set when effect resolution needs player input. Check each frame in the
    /// client to render the appropriate decision modal; clear via
    /// `submit_decision`. While `Some`, no other game actions are permitted.
    pub pending_decision: Option<PendingDecision>,
    /// One-shot signal from `resolve_effect` to the enclosing resolver when an
    /// effect needs to suspend. Callers check this after each effect call, wrap
    /// it up in `pending_decision` with the full resume context, and return.
    /// `remaining` carries any sibling effects still queued behind the one that
    /// suspended (e.g. `Draw` after `Scry` in a Seq).
    pub(crate) suspend_signal: Option<(Decision, PendingEffectState, Effect)>,
    /// One-shot validated answer for a resolution-time choice whose suspend
    /// re-queues the originating effect as its continuation (`ChooseN`,
    /// `Escalate`, `MayDo`, `DealDamageDivided`, `ChooseAmount` payers, and
    /// deferred trigger `ChooseMode`s). `apply_pending_effect_answer` stashes
    /// the sanitised answer here; the re-run effect `take()`s it instead of
    /// asking the decider again. Always consumed within the same
    /// `submit_decision` call, so it never crosses a serialization boundary.
    #[serde(skip, default)]
    pub(crate) stashed_resolution_answer: Option<DecisionAnswer>,
    /// Replay log for multi-question resolution effects (Clash,
    /// `PlayersMayAccept`, `TemptingOffer`, `UnlessPlayerPays`, `MayPay`):
    /// each suspend re-queues the *originating effect*, whose re-run replays
    /// the logged answers in ask order (via a local cursor) before reaching
    /// the next unanswered question. `apply_pending_effect_answer` appends
    /// the validated answer; the effect clears the log on every completing
    /// path (`ask_bool_logged` / `clear_answer_log`). Serialized so a
    /// snapshot taken between two questions of the same effect round-trips.
    #[serde(default)]
    pub(crate) resolution_answer_log: Vec<DecisionAnswer>,
    /// Life-payment events (Phyrexian pips, "pay N life" costs) queued by
    /// `pay_receipt_life` mid-cast and drained into the action's event batch
    /// at the end of `perform_action`, so paid life fires life-loss triggers
    /// (CR 118.8 / 119.3c) after the cast completes (CR 601.3e).
    #[serde(skip, default)]
    pub(crate) pending_cost_events: Vec<GameEvent>,
    /// CR 700.4 — permanents that hit a graveyard from the battlefield since
    /// the last trigger dispatch: `(card_id, last_controller, is_creature,
    /// is_artifact)`. Populated at the single raw removal chokepoint
    /// (`remove_from_battlefield_to_graveyard_raw`); drained by
    /// `dispatch_triggers_for_events` into `GameEvent::PermanentDied` so
    /// "whenever a creature or artifact you control dies" triggers
    /// (Judge Magister Gabranth, G'raha Tia) fire on non-creature deaths that
    /// no `CreatureDied` event covers.
    #[serde(skip, default)]
    pub(crate) pending_permanent_deaths: Vec<(CardId, usize, bool, bool)>,
    /// True when an effect has flagged "prevent all combat damage this turn"
    /// (CR 615 — damage prevention as a replacement effect). Wired by
    /// Owlin Shieldmage's ETB trigger, Holy Day, Hallowed Burial-adjacent
    /// "fog" patterns. Cleared in `do_cleanup` alongside the other
    /// until-end-of-turn flags. Combat damage resolution (`resolve_combat_
    /// damage_with_filter`) consults this flag and skips dealing the
    /// damage half (lifelink, deathtouch, infect/wither, trigger emission
    /// for non-damage knock-ons all still resolve — only the damage
    /// number itself is set to 0 per CR 615.1).
    #[serde(default)]
    pub(crate) prevent_combat_damage_this_turn: bool,
    /// EOE Void — true once any nonland permanent has left the battlefield
    /// this turn. The left half of `Predicate::VoidActive`; reset at the turn
    /// boundary.
    #[serde(default)]
    pub(crate) nonland_permanent_left_bf_this_turn: bool,
    /// CR 615.1 fog with an exception (Inspire Awe). When `Some(filter)` and
    /// `prevent_combat_damage_this_turn` is set, a creature's combat damage is
    /// prevented unless the *dealer* matches `filter`. `None` = prevent all.
    #[serde(default)]
    pub(crate) prevent_combat_damage_except: Option<crate::card::SelectionRequirement>,
    /// CR 701.10f / 614.5 — transient mana-production multiplier for the
    /// mana ability currently resolving (Mana Reflection ×2, Nyxbloom
    /// Ancient ×3, composed). Set before a tapped-for-mana ability resolves
    /// and reset to 1 after; the `AddMana` resolver scales each pip count
    /// by it. 0 (serde default) reads as 1×.
    #[serde(default)]
    pub(crate) mana_production_multiplier: u32,
    /// Identity of the spell currently resolving — (card id, caster, printed
    /// colors). Stamped around `resolve_effect` in spell resolution so
    /// source-aware damage replacements (Torbran) can read the controller and
    /// colors of a card that's in no visible zone mid-resolution. Transient.
    #[serde(skip)]
    pub(crate) resolving_source: Option<(CardId, usize, Vec<crate::mana::Color>)>,
    /// Reentrancy guard: true while `gather_continuous_effects` runs, so
    /// layer-aware type filters (`evaluate_requirement_static`) fall back to
    /// printed types instead of recursing through `computed_permanent`.
    #[serde(skip)]
    pub(crate) in_layer_gather: std::sync::atomic::AtomicBool,
    /// Scoped memo of the gathered continuous-effect set — see
    /// [`GameState::with_frozen_layers`]. Always `None` outside a freeze
    /// scope; clones and serde restores start unfrozen.
    #[serde(skip)]
    pub(crate) layer_freeze: LayerFreeze,
    /// CR 505.1b — additional combat phases banked for the active player.
    /// `Effect::AdditionalCombatPhase` increments this; when the active
    /// player leaves the End of Combat step with it set, the turn loops back
    /// to Begin Combat (decrementing) instead of advancing to the postcombat
    /// main. Reset at cleanup so it can't bleed into the next turn.
    #[serde(default)]
    pub(crate) additional_combat_phases: u32,
    /// CR 505.1b — combat phases banked by `AdditionalCombatPhaseAfterMain`
    /// (Relentless Assault): when the active player leaves a main phase with
    /// one banked, the turn enters Begin Combat instead of the next phase
    /// (the follow-up main comes from the normal EndCombat → PostMain flow).
    /// Reset at cleanup.
    #[serde(default)]
    pub(crate) additional_post_main_combats: u32,
    /// How many Begin Combat steps have started this turn (1 during the first
    /// combat, 2 during an extra combat, …). Read by
    /// `Predicate::IsFirstCombatPhaseThisTurn` so "if it's the first combat
    /// phase of the turn" riders (Genji Glove) don't loop on the extra combat
    /// they grant. Reset at cleanup.
    #[serde(default)]
    pub(crate) combat_phases_this_turn: u32,
    /// CR 500.7 — additional end steps banked by `Effect::AdditionalEndStep`.
    /// When the active player leaves the End step with this set, the turn loops
    /// back to another End step (decrementing) instead of advancing to cleanup
    /// (Y'shtola Rhul). Reset at cleanup.
    #[serde(default)]
    pub(crate) additional_end_steps: u32,
    /// How many End steps have begun this turn (1 during the first, 2 during an
    /// extra, …). Read by `Predicate::IsFirstEndStepThisTurn` so "if it's the
    /// first end step" riders don't loop on the extra step they grant.
    #[serde(default)]
    pub(crate) end_steps_this_turn: u32,
    /// CR 500.9 — additional upkeep steps banked by
    /// `Effect::AdditionalUpkeepStep`. When the active player leaves the
    /// Upkeep step with this set, the turn loops back to another Upkeep
    /// (decrementing) instead of advancing to Draw (Paradox Haze). Reset at
    /// cleanup.
    #[serde(default)]
    pub(crate) additional_upkeep_steps: u32,
    /// How many Upkeep steps have begun this turn. Read by
    /// `Predicate::IsFirstUpkeepThisTurn` so Paradox Haze's "first upkeep
    /// step of your turn" gate doesn't loop on the extra step it grants.
    #[serde(default)]
    pub(crate) upkeep_steps_this_turn: u32,
    /// CR 614.9 / 615 — creatures whose combat damage is prevented in both
    /// directions for the rest of the turn (Maze of Ith: "prevent all combat
    /// damage that would be dealt to and dealt by that creature"). The combat
    /// resolver skips dealing *and* receiving combat damage for any creature
    /// in this set. Cleared at cleanup.
    #[serde(default)]
    pub(crate) combat_damage_prevented_creatures: Vec<CardId>,
    /// CR 615 — creatures that prevent all combat damage dealt *to* them this
    /// turn (incoming only; they still deal their own). The turn-scoped sibling
    /// of the `PreventAllCombatDamageToThis` static (Fog Bank), granted by a
    /// one-shot effect — Fleeting Flight's "prevent all combat damage that would
    /// be dealt to it this turn". Cleared at cleanup.
    #[serde(default)]
    pub(crate) combat_damage_prevented_to_this_turn: Vec<CardId>,
    /// CR 510.1c — attackers that became blocked this combat. An attacker
    /// stays blocked even if all its blockers leave combat (double-strike
    /// step-one kills, post-block removal): without trample it assigns no
    /// combat damage. Cleared when combat ends.
    #[serde(default)]
    pub(crate) blocked_attackers: Vec<CardId>,
    /// CR 614.13-style ETB-control replacement (Gather Specimens): seats
    /// whose opponents' creatures enter under their control instead this
    /// turn. Cleared at cleanup.
    #[serde(default)]
    pub(crate) creature_etb_steal_this_turn: Vec<usize>,
    /// Players who have paid the Leonin Arbiter search tax this turn
    /// (covers further searches until end of turn). Cleared at cleanup.
    #[serde(default)]
    pub(crate) search_tax_paid_this_turn: Vec<usize>,
    /// "Spells your opponents cast cost {N} more until your next turn"
    /// (Elspeth Conquers Death II). Each entry taxes matching spells cast by
    /// opponents of `controller`; cleared at `controller`'s untap.
    #[serde(default)]
    pub(crate) turn_scoped_spell_taxes: Vec<TurnScopedSpellTax>,
    /// CR 615.7 — sources whose damage is prevented entirely this turn
    /// (Burrenton Forge-Tender's chosen source). Cleared at cleanup.
    #[serde(default)]
    pub(crate) damage_prevented_sources: Vec<CardId>,
    /// Lightning, Army of One's Stagger — `(victim, registrant)` pairs: until
    /// the registrant's next turn, damage to the victim or a permanent they
    /// control is doubled (applied in `scale_damage_to`). Cleared as the
    /// registrant's turn begins.
    #[serde(default)]
    pub(crate) staggered_damage_players: Vec<(usize, usize)>,
    /// Per-pair "can't block" restrictions for the turn: `(blocker, attacker)`
    /// — the blocker can't block that specific attacker (Kozilek's Pathfinder's
    /// "{C}: Target creature can't block this creature this turn"). Cleared at
    /// cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) cant_block_pairs: Vec<(CardId, CardId)>,
    /// CR 508.1a — creatures granted "can attack this turn as though it didn't
    /// have defender" (Krotiq Nestguard's activated ability). Cleared at cleanup.
    pub(crate) attack_despite_defender_this_turn: Vec<CardId>,
    /// Active prevention shields (CR 615.1) around players/permanents.
    /// Created by `Effect::PreventNextDamage` / `PreventAllDamageThisTurn`;
    /// consulted by the non-combat damage path (`deal_damage_to_from`) and
    /// cleared at cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) prevention_shields: Vec<crate::game::types::PreventionShield>,
    /// CR 615.12 — "Damage can't be prevented this turn" (Skullcrack,
    /// Impractical Joke). While set, every prevention shield is ignored.
    /// Cleared at cleanup.
    #[serde(default)]
    pub(crate) damage_cant_be_prevented_this_turn: bool,
    /// Registered replacement effects (Phase H — Commander prerequisite).
    /// Walked by zone-change paths (`place_card_in_dest`,
    /// `remove_from_battlefield_to_*`) at placement time; a matching
    /// entry rewrites the destination zone.
    ///
    /// `#[serde(default)]` so snapshots written before this field
    /// existed deserialize cleanly as empty (no replacements active).
    #[serde(default)]
    pub replacement_effects: Vec<crate::replacement::ReplacementEffect>,
    /// Monotonic counter handing out `ReplacementId`s. Defaults to 0
    /// for snapshot back-compat.
    #[serde(default)]
    pub(crate) next_replacement_id: u32,
    /// Per-commander cast-from-command-zone counter (Phase L).
    /// Keyed by the commander's `CardId`; each entry tracks how many
    /// times that commander has been cast from the command zone this
    /// game. The commander tax is `{2}` × this value, added as
    /// generic mana on top of the printed cost (CR 903.8).
    ///
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub commander_cast_count: HashMap<CardId, u32>,
    /// 21-commander-damage tracker (Phase M / CR 704.5v). Keyed by
    /// `(victim_seat, commander_card_id)`; values are running totals
    /// of combat / direct damage dealt by that commander to that
    /// seat over the whole game. The SBA in
    /// `check_state_based_actions` eliminates a player when any of
    /// their entries crosses 21.
    ///
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub commander_damage: HashMap<(usize, CardId), u32>,
    /// Per-dying-card snapshot
    /// cache, populated at SBA emission time for every dying creature
    /// (token or non-token). Used by trigger-dispatch lookups
    /// (`game/effects/events.rs::event_matches_spec`,
    /// `evaluate_requirement_static` zone walk) so AnotherOfYours-
    /// scope triggers with creature-type filters (Witherbloom
    /// Pestmaster, Felisa, Fang of Silverquill) fire correctly when
    /// the dying subject is a token — CR 111.7c's "ceases to exist"
    /// SBA removes the token from every zone in the same sweep as
    /// the death event emission, so by the time
    /// `dispatch_triggers_for_events` runs the token is gone from
    /// both battlefield and graveyard. The cached `CardInstance`
    /// survives the SBA sweep, giving the dispatcher a reliable
    /// way to read both the controller AND the dying card's
    /// printed types / counters. Cleared after each dispatch pass.
    /// `#[serde(skip)]` because it's transient scratch — snapshots
    /// don't need to preserve mid-SBA state.
    #[serde(skip)]
    pub(crate) died_card_snapshots: HashMap<CardId, CardInstance>,
    /// Auras that lost their host this turn, keyed by the (now-gone) host's
    /// CardId → list of `(aura id, aura controller)`. Populated in the
    /// orphan-Aura SBA sweep before the Aura is sent to the graveyard, so
    /// "whenever an enchanted creature dies" payoffs (Hateful Eidolon,
    /// Dawn Evangel) can count the Auras you controlled that were on it at
    /// resolution time. Cleared in `do_cleanup`. `#[serde(skip)]` — transient.
    #[serde(skip)]
    pub(crate) auras_at_death: HashMap<CardId, Vec<(CardId, usize)>>,
    /// CR 603.10 / 608.2h — last-known-information snapshots for
    /// leaves-the-battlefield triggers that read the dying object's
    /// characteristics *as they last existed on the battlefield* (e.g.
    /// "when this dies, it deals damage equal to its power" — counters
    /// and pumps included). Keyed by the trigger's source CardId.
    /// Populated when such a trigger is pushed (`push_pending_trigger`)
    /// and removed once it resolves. `Value::PowerOf`/`ToughnessOf`
    /// consult it (priority over the graveyard's printed P/T) while
    /// `resolving_lki_source` names the trigger currently resolving.
    /// Transient scratch — `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) leaves_bf_lki: HashMap<CardId, CardInstance>,
    /// The source CardId of the leaves-battlefield trigger currently
    /// resolving, if it has a `leaves_bf_lki` snapshot. Scopes the LKI
    /// power/toughness read to that one resolution. `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) resolving_lki_source: Option<CardId>,
    /// CR 603.10 — the dead *subject* CardId of the leaves-battlefield trigger
    /// currently resolving (distinct from its source: Jenova's "whenever a
    /// Mutant you control dies, draw cards equal to its power" reads the dead
    /// Mutant's LKI power, not Jenova's). Scopes the same LKI read to that
    /// subject. `#[serde(skip)]`.
    #[serde(skip)]
    pub(crate) resolving_lki_subject: Option<CardId>,
    /// Set of permanent CardIds that gained one or more counters during
    /// the current turn. Bumped in `Effect::AddCounter`'s resolver
    /// whenever a permanent gains counters; reset to empty in
    /// `do_cleanup`. Powers Fractal Tender's end-step "if you put a
    /// counter on this creature this turn, mint a Fractal" rider via
    /// the new `Predicate::SourceGainedCounterThisTurn` predicate.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) permanents_gained_counter_this_turn: std::collections::HashSet<CardId>,
    /// Permanents whose `StaticEffect::CounterAmplifierOncePerTurn` extra
    /// +1/+1 counter has already been added this turn (Cursed Wombat). The
    /// granted ability "triggers only once each turn" per permanent; cleared at
    /// cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) permanents_amplified_counter_this_turn: std::collections::HashSet<CardId>,
    /// How many times each source's escalating ability has resolved this turn
    /// (CR 603.3-style "if this is the first/second/third time …" — Vito,
    /// Fanatic of Aclazotz). Keyed by source `CardId`; cleared at cleanup.
    #[serde(default)]
    pub(crate) ability_resolutions_this_turn: std::collections::HashMap<CardId, u32>,
    /// Per-permanent transient triggered abilities granted by spells /
    /// continuous effects (Rabid Attack, Root Manipulation: "creatures
    /// you control gain '…trigger…' until end of turn"). The dispatcher
    /// walks this map alongside each permanent's printed
    /// `triggered_abilities` and fires matching events. Cleared in
    /// `do_cleanup` (the "until end of turn" expiry). Other durations
    /// (Permanent) would need a separate map; only EOT grants are
    /// modeled today since that's what the printed catalog needs.
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) granted_triggers_eot:
        std::collections::HashMap<CardId, Vec<crate::card::TriggeredAbility>>,
    /// Permanents whose death is replaced by exile for the rest of the
    /// turn — "if that creature would die this turn, exile it instead"
    /// (Wilt in the Heat). Checked in `remove_from_battlefield_to_graveyard_raw`
    /// alongside the Finality-counter redirect; cleared at cleanup. The
    /// redirect lasts the whole turn, so it also catches deaths from later
    /// combat / removal, not just the spell's own damage. `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub(crate) dies_to_exile_eot: std::collections::HashSet<CardId>,
    /// CR 702.15 — the seat that should gain life from lifelink on damage dealt
    /// by the instant/sorcery spell currently resolving, if its controller has
    /// "your spells have lifelink" (Radiant Scrollwielder). Set around the
    /// spell's resolution and cleared after; transient, not serialized.
    #[serde(skip)]
    pub(crate) resolving_spell_lifelink_seat: Option<usize>,
    /// Reentrancy guard for the CR 121.2a draw-doubling replacement — the
    /// extra draws aren't themselves re-doubled (CR 614.5). Transient.
    #[serde(skip)]
    pub(crate) in_draw_double: bool,
    /// Reentrancy guard for CR 614.9 damage redirection (one redirect per
    /// damage event). Transient.
    #[serde(skip)]
    pub(crate) in_damage_redirect: bool,
    /// Reentrancy guard for token-mint replacements (Academy Manufactor) —
    /// the replacement's extra mints aren't re-replaced (CR 614.5). Transient.
    #[serde(skip)]
    pub(crate) in_token_replacement: bool,
    /// Temporary control changes awaiting reversion (Act of Treason /
    /// Threaten / Tempted by the Oriq). `Effect::GainControl` with a
    /// non-`Permanent` duration records the controller the permanent had
    /// immediately before the steal so control snaps back when the
    /// duration ends (CR 800.4 control-changing effects). `#[serde(default)]`
    /// for snapshot back-compat.
    #[serde(default)]
    pub(crate) temporary_control: Vec<TempControl>,
    /// Temporary "becomes a copy" definition swaps awaiting reversion
    /// (`Effect::BecomeCopyOfFor`, CR 707.2). Records the pre-copy
    /// definition; the swap snaps back when the duration ends, mirroring
    /// `temporary_control`. Entries whose card left the battlefield are
    /// dropped (a new object keeps nothing). `#[serde(default)]` for
    /// snapshot back-compat.
    #[serde(default)]
    pub(crate) temporary_copies: Vec<TempCopy>,
    /// CR 702.143b — cards foretold this turn can't be cast from exile until
    /// a later turn. Tracks the cards a player foretold during the current
    /// turn; cleared at cleanup. `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) foretold_this_turn: std::collections::HashSet<CardId>,
    /// CR 702.170 — cards currently plotted (exiled face-up, castable from
    /// exile without paying their mana cost on a later turn).
    /// `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub(crate) plotted_cards: std::collections::HashSet<CardId>,
    /// CR 702.170d — cards plotted *this* turn can't be cast until a later
    /// turn. Cleared at cleanup. `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub(crate) plotted_this_turn: std::collections::HashSet<CardId>,
    /// CR 603.3d — triggered abilities flagged `TriggeredAbility::once_per_turn`
    /// ("this ability triggers only once each turn") that have already fired
    /// this turn, keyed by (source card, trigger index). Cleared at cleanup.
    /// `#[serde(default)]` for snapshot back-compat. Powers Dramatic Finale.
    #[serde(default)]
    pub(crate) triggered_once_per_turn_used: std::collections::HashSet<(CardId, usize)>,
    /// `EventSpec::per_subject_cap` tallies: fires of a capped trigger this
    /// turn, keyed by (watcher, event subject). Cleared at cleanup.
    #[serde(default)]
    pub(crate) per_subject_trigger_uses: std::collections::HashMap<(CardId, CardId), u8>,
    /// CR 724 — the monarch (if any). The monarch draws a card at the
    /// beginning of their end step, and a creature dealing combat damage to
    /// the monarch makes its controller the new monarch. `#[serde(default)]`
    /// (None = no monarch) for snapshot back-compat.
    #[serde(default)]
    pub monarch: Option<usize>,
    /// CR 731 — the game's day/night designation (None = neither, the
    /// starting state). `#[serde(default)]` for snapshot back-compat.
    #[serde(default)]
    pub day_night: Option<crate::game::types::DayNight>,
    /// The active player of the turn that just ended — read by the CR 502.2
    /// day/night turn-based check (which consults the *previous* turn's
    /// active player's spell count). `#[serde(default)]` for back-compat.
    #[serde(default)]
    pub(crate) previous_turn_active: Option<usize>,
}

/// A pending control-reversion entry — see `GameState.temporary_control`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TempControl {
    pub(crate) card: CardId,
    pub(crate) original_controller: usize,
    pub(crate) duration: crate::effect::Duration,
    /// "For as long as [source] remains on the battlefield" steals (Sower of
    /// Temptation): control reverts when this permanent leaves, via
    /// `on_left_battlefield`. `duration` is `Permanent` so turn sweeps skip it.
    #[serde(default)]
    pub(crate) source: Option<CardId>,
}

/// A turn-scoped spell tax — see `GameState.turn_scoped_spell_taxes`.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct TurnScopedSpellTax {
    /// The effect's controller: their opponents pay, and the tax clears at
    /// their untap step.
    pub(crate) controller: usize,
    pub(crate) amount: u32,
    pub(crate) filter: crate::card::SelectionRequirement,
}

/// A pending copy-reversion entry — see `GameState.temporary_copies`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct TempCopy {
    pub(crate) card: CardId,
    /// Live handle to the pre-copy definition. Skipped in snapshots —
    /// recovered by name through the registry resolver on load (the same
    /// name-keyed round-trip `CardInstance` uses).
    #[serde(skip)]
    pub(crate) original: Option<std::sync::Arc<crate::card::CardDefinition>>,
    pub(crate) original_name: String,
    pub(crate) duration: crate::effect::Duration,
}

impl TempCopy {
    /// The pre-copy definition: the live Arc, or a registry lookup after a
    /// snapshot round-trip.
    fn original_def(&self) -> Option<std::sync::Arc<crate::card::CardDefinition>> {
        self.original.clone().or_else(|| {
            crabomination_base::registry::resolve_card(&self.original_name).map(std::sync::Arc::new)
        })
    }
}

/// Manual `Clone` impl so the bot can dry-run an action against a copy
/// of the state without committing it. `Box<dyn Decider>` blocks the
/// derive — we round-trip through `DeciderKind`. Custom deciders not
/// modeled by the kind enum collapse to `AutoDecider` on clone, which
/// is fine for the dry-run use case (we discard the clone immediately).
impl Clone for GameState {
    fn clone(&self) -> Self {
        Self {
            players: self.players.clone(),
            teams: self.teams.clone(),
            battlefield: self.battlefield.clone(),
            phased_out: self.phased_out.clone(),
            exile: self.exile.clone(),
            stack: self.stack.clone(),
            step: self.step,
            active_player_idx: self.active_player_idx,
            turn_number: self.turn_number,
            game_over: self.game_over,
            priority: self.priority.clone(),
            continuous_effects: self.continuous_effects.clone(),
            next_effect_timestamp: self.next_effect_timestamp,
            next_id: self.next_id,
            attacking: self.attacking.clone(),
            block_map: self.block_map.clone(),
            combat_damage_order: self.combat_damage_order.clone(),
            combat_damage_assignment: self.combat_damage_assignment.clone(),
            combat_damage_plan_step: self.combat_damage_plan_step,
            blockers_declared: self.blockers_declared,
            skip_first_draw: self.skip_first_draw,
            spells_cast_this_turn: self.spells_cast_this_turn,
            cycled_count_by_name: self.cycled_count_by_name.clone(),
            mana_spent_on_spells_this_turn: self.mana_spent_on_spells_this_turn,
            expend_prev_total: self.expend_prev_total,
            spells_cast_last_turn: self.spells_cast_last_turn,
            permanents_to_graveyard_this_turn: self.permanents_to_graveyard_this_turn,
            graveyard_from_battlefield_this_turn: self
                .graveyard_from_battlefield_this_turn
                .clone(),
            entered_from_graveyard_this_turn: self.entered_from_graveyard_this_turn.clone(),
            entered_from_exile_this_turn: self.entered_from_exile_this_turn.clone(),
            delayed_triggers: self.delayed_triggers.clone(),
            attacking_token_cleanup: self.attacking_token_cleanup.clone(),
            sacrificed_power: self.sacrificed_power,
            sacrificed_was_artifact: self.sacrificed_was_artifact,
            sacrificed_was_outlaw: self.sacrificed_was_outlaw,
            sacrificed_was_vehicle: self.sacrificed_was_vehicle,
            last_discarded_card_types: self.last_discarded_card_types,
            sacrificed_toughness: self.sacrificed_toughness,
            sacrificed_mana_value: self.sacrificed_mana_value,
            last_discarded_mana_value: self.last_discarded_mana_value,
            block_poison_this_turn: self.block_poison_this_turn,
            tapped_for_cost_power: self.tapped_for_cost_power,
            trigger_event_amount_scratch: self.trigger_event_amount_scratch,
            last_created_token: self.last_created_token,
            last_die_roll: self.last_die_roll,
            extra_cast_reduction: self.extra_cast_reduction,
            cast_paid_uncounterable: self.cast_paid_uncounterable,
            last_created_tokens: self.last_created_tokens.clone(),
            last_moved_cards: self.last_moved_cards.clone(),
            cards_discarded_this_resolution: self.cards_discarded_this_resolution,
            energy_paid_this_resolution: self.energy_paid_this_resolution,
            creature_cards_discarded_this_resolution: self.creature_cards_discarded_this_resolution,
            cards_discarded_per_player_this_resolution: self.cards_discarded_per_player_this_resolution.clone(),
            nonland_cards_discarded_per_player_this_resolution: self.nonland_cards_discarded_per_player_this_resolution.clone(),
            shuffle_resolving_spell_into_library: self.shuffle_resolving_spell_into_library,
            return_resolving_spell_to_hand: self.return_resolving_spell_to_hand,
            exile_resolving_spell: self.exile_resolving_spell,
            end_turn_requested: self.end_turn_requested,
            cipher_encode_pending: self.cipher_encode_pending,
            haunt_pending: self.haunt_pending.clone(),
            discarded_card_ids_this_resolution: self.discarded_card_ids_this_resolution.clone(),
            exiled_card_ids_this_resolution: self.exiled_card_ids_this_resolution.clone(),
            permanents_destroyed_this_resolution: self.permanents_destroyed_this_resolution,
            excess_damage_this_resolution: self.excess_damage_this_resolution,
            players_sacrificed_this_resolution: self.players_sacrificed_this_resolution.clone(),
            named_card_this_resolution: self.named_card_this_resolution.clone(),
            pending_cast_face: self.pending_cast_face,
            pending_cast_sacrifices: self.pending_cast_sacrifices.clone(),
            pending_cast_discards: self.pending_cast_discards.clone(),
            pending_spree_modes: self.pending_spree_modes.clone(),
            pending_cast_spend_float: self.pending_cast_spend_float,
            pending_prepare_copies: self.pending_prepare_copies.clone(),
            pending_landcycle_pick: self.pending_landcycle_pick,
            pending_ability_sac_other: self.pending_ability_sac_other,
            pending_ability_tap_other: self.pending_ability_tap_other,
            pending_ability_exile_other: self.pending_ability_exile_other.clone(),
            decider: self.decider.kind().into_boxed(),
            pending_decision: self.pending_decision.clone(),
            suspend_signal: self.suspend_signal.clone(),
            stashed_resolution_answer: self.stashed_resolution_answer.clone(),
            resolution_answer_log: self.resolution_answer_log.clone(),
            pending_cost_events: self.pending_cost_events.clone(),
            pending_permanent_deaths: self.pending_permanent_deaths.clone(),
            prevent_combat_damage_this_turn: self.prevent_combat_damage_this_turn,
            nonland_permanent_left_bf_this_turn: self.nonland_permanent_left_bf_this_turn,
            prevent_combat_damage_except: self.prevent_combat_damage_except.clone(),
            mana_production_multiplier: self.mana_production_multiplier,
            resolving_source: self.resolving_source.clone(),
            in_layer_gather: std::sync::atomic::AtomicBool::new(false),
            layer_freeze: LayerFreeze::default(),
            additional_combat_phases: self.additional_combat_phases,
            additional_post_main_combats: self.additional_post_main_combats,
            combat_phases_this_turn: self.combat_phases_this_turn,
            additional_end_steps: self.additional_end_steps,
            end_steps_this_turn: self.end_steps_this_turn,
            additional_upkeep_steps: self.additional_upkeep_steps,
            upkeep_steps_this_turn: self.upkeep_steps_this_turn,
            combat_damage_prevented_creatures: self.combat_damage_prevented_creatures.clone(),
            combat_damage_prevented_to_this_turn: self.combat_damage_prevented_to_this_turn.clone(),
            blocked_attackers: self.blocked_attackers.clone(),
            creature_etb_steal_this_turn: self.creature_etb_steal_this_turn.clone(),
            search_tax_paid_this_turn: self.search_tax_paid_this_turn.clone(),
            turn_scoped_spell_taxes: self.turn_scoped_spell_taxes.clone(),
            staggered_damage_players: self.staggered_damage_players.clone(),
            damage_prevented_sources: self.damage_prevented_sources.clone(),
            cant_block_pairs: self.cant_block_pairs.clone(),
            attack_despite_defender_this_turn: self.attack_despite_defender_this_turn.clone(),
            prevention_shields: self.prevention_shields.clone(),
            damage_cant_be_prevented_this_turn: self.damage_cant_be_prevented_this_turn,
            replacement_effects: self.replacement_effects.clone(),
            next_replacement_id: self.next_replacement_id,
            commander_cast_count: self.commander_cast_count.clone(),
            commander_damage: self.commander_damage.clone(),
            died_card_snapshots: self.died_card_snapshots.clone(),
            auras_at_death: self.auras_at_death.clone(),
            leaves_bf_lki: self.leaves_bf_lki.clone(),
            resolving_lki_source: self.resolving_lki_source,
            resolving_lki_subject: self.resolving_lki_subject,
            permanents_gained_counter_this_turn: self.permanents_gained_counter_this_turn.clone(),
            permanents_amplified_counter_this_turn: self.permanents_amplified_counter_this_turn.clone(),
            ability_resolutions_this_turn: self.ability_resolutions_this_turn.clone(),
            granted_triggers_eot: self.granted_triggers_eot.clone(),
            dies_to_exile_eot: self.dies_to_exile_eot.clone(),
            resolving_spell_lifelink_seat: self.resolving_spell_lifelink_seat,
            in_draw_double: self.in_draw_double,
            in_damage_redirect: self.in_damage_redirect,
            in_token_replacement: self.in_token_replacement,
            temporary_control: self.temporary_control.clone(),
            temporary_copies: self.temporary_copies.clone(),
            foretold_this_turn: self.foretold_this_turn.clone(),
            plotted_cards: self.plotted_cards.clone(),
            plotted_this_turn: self.plotted_this_turn.clone(),
            triggered_once_per_turn_used: self.triggered_once_per_turn_used.clone(),
            per_subject_trigger_uses: self.per_subject_trigger_uses.clone(),
            monarch: self.monarch,
            day_night: self.day_night,
            previous_turn_active: self.previous_turn_active,
        }
    }
}

impl GameState {
    /// Spend `amount` {E} from player `p`, clamped to what they have, and add
    /// it to `energy_spent_this_turn` (the tally behind "paid or lost N+ {E}
    /// this turn" gates). All energy-cost chokepoints route through here.
    pub(crate) fn spend_energy(&mut self, p: usize, amount: u32) {
        let amount = amount.min(self.players[p].energy);
        self.players[p].energy -= amount;
        self.players[p].energy_spent_this_turn =
            self.players[p].energy_spent_this_turn.saturating_add(amount);
    }

    /// Create a fresh game.  `players` must have at least 2 entries. Defaults
    /// to 20-life, 2-player rules; call [`apply_format`] (or set
    /// `skip_first_draw` / per-player `life` directly) to configure the game
    /// for a specific format or player count.
    pub fn new(players: Vec<Player>) -> Self {
        let n = players.len();
        // Default: one singleton team per seat (free-for-all semantics).
        // Team formats reshape this via `assign_teams`.
        let teams = (0..n)
            .map(|i| crate::team::Team {
                id: crate::team::TeamId(i),
                members: vec![i],
                shared_life: None,
            })
            .collect();
        Self {
            players,
            teams,
            battlefield: Vec::new(),
            phased_out: Vec::new(),
            exile: Vec::new(),
            stack: Vec::new(),
            step: TurnStep::Untap,
            active_player_idx: 0,
            turn_number: 1,
            game_over: None,
            priority: PriorityState::new(0),
            continuous_effects: Vec::new(),
            next_effect_timestamp: 1,
            next_id: 1,
            attacking: Vec::new(),
            block_map: HashMap::new(),
            combat_damage_order: HashMap::new(),
            combat_damage_assignment: HashMap::new(),
            combat_damage_plan_step: None,
            blockers_declared: false,
            // Multiplayer (3+) doesn't skip the first draw — only the 2-player
            // starting player does.
            skip_first_draw: n <= 2,
            spells_cast_this_turn: 0,
            cycled_count_by_name: std::collections::HashMap::new(),
            mana_spent_on_spells_this_turn: 0,
            expend_prev_total: 0,
            spells_cast_last_turn: 0,
            permanents_to_graveyard_this_turn: 0,
            graveyard_from_battlefield_this_turn: Default::default(),
            entered_from_graveyard_this_turn: std::collections::HashSet::new(),
            entered_from_exile_this_turn: std::collections::HashSet::new(),
            delayed_triggers: Vec::new(),
            attacking_token_cleanup: Vec::new(),
            sacrificed_power: None,
            sacrificed_was_artifact: None,
            sacrificed_was_outlaw: None,
            sacrificed_was_vehicle: None,
            last_discarded_card_types: 0,
            sacrificed_toughness: None,
            sacrificed_mana_value: None,
            last_discarded_mana_value: None,
            block_poison_this_turn: 0,
            tapped_for_cost_power: None,
            trigger_event_amount_scratch: 0,
            last_created_token: None,
            last_die_roll: 0,
            extra_cast_reduction: 0,
            cast_paid_uncounterable: false,
            last_created_tokens: Vec::new(),
            last_moved_cards: Vec::new(),
            cards_discarded_this_resolution: 0,
            energy_paid_this_resolution: 0,
            creature_cards_discarded_this_resolution: 0,
            cards_discarded_per_player_this_resolution: HashMap::new(),
            nonland_cards_discarded_per_player_this_resolution: HashMap::new(),
            shuffle_resolving_spell_into_library: false,
            return_resolving_spell_to_hand: false,
            exile_resolving_spell: false,
            end_turn_requested: false,
            cipher_encode_pending: None,
            haunt_pending: None,
            discarded_card_ids_this_resolution: Vec::new(),
            exiled_card_ids_this_resolution: Vec::new(),
            permanents_destroyed_this_resolution: 0,
            excess_damage_this_resolution: 0,
            players_sacrificed_this_resolution: std::collections::HashSet::new(),
            named_card_this_resolution: None,
            pending_cast_face: CastFace::Front,
            pending_cast_sacrifices: None,
            pending_cast_discards: None,
            pending_spree_modes: None,
            pending_cast_spend_float: None,
            pending_prepare_copies: Vec::new(),
            pending_landcycle_pick: None,
            pending_ability_sac_other: None,
            pending_ability_tap_other: None,
            pending_ability_exile_other: None,
            decider: Box::new(AutoDecider),
            pending_decision: None,
            suspend_signal: None,
            stashed_resolution_answer: None,
            resolution_answer_log: Vec::new(),
            pending_cost_events: Vec::new(),
            pending_permanent_deaths: Vec::new(),
            prevent_combat_damage_this_turn: false,
            nonland_permanent_left_bf_this_turn: false,
            prevent_combat_damage_except: None,
            mana_production_multiplier: 1,
            resolving_source: None,
            in_layer_gather: std::sync::atomic::AtomicBool::new(false),
            layer_freeze: LayerFreeze::default(),
            additional_combat_phases: 0,
            additional_post_main_combats: 0,
            combat_phases_this_turn: 0,
            additional_end_steps: 0,
            end_steps_this_turn: 0,
            additional_upkeep_steps: 0,
            upkeep_steps_this_turn: 0,
            combat_damage_prevented_creatures: Vec::new(),
            combat_damage_prevented_to_this_turn: Vec::new(),
            blocked_attackers: Vec::new(),
            creature_etb_steal_this_turn: Vec::new(),
            search_tax_paid_this_turn: Vec::new(),
            turn_scoped_spell_taxes: Vec::new(),
            staggered_damage_players: Vec::new(),
            damage_prevented_sources: Vec::new(),
            cant_block_pairs: Vec::new(),
            attack_despite_defender_this_turn: Vec::new(),
            prevention_shields: Vec::new(),
            damage_cant_be_prevented_this_turn: false,
            replacement_effects: Vec::new(),
            next_replacement_id: 1,
            commander_cast_count: HashMap::new(),
            commander_damage: HashMap::new(),
            died_card_snapshots: HashMap::new(),
            auras_at_death: HashMap::new(),
            leaves_bf_lki: HashMap::new(),
            resolving_lki_source: None,
            resolving_lki_subject: None,
            permanents_gained_counter_this_turn: std::collections::HashSet::new(),
            permanents_amplified_counter_this_turn: std::collections::HashSet::new(),
            ability_resolutions_this_turn: std::collections::HashMap::new(),
            granted_triggers_eot: std::collections::HashMap::new(),
            dies_to_exile_eot: std::collections::HashSet::new(),
            resolving_spell_lifelink_seat: None,
            in_draw_double: false,
            in_damage_redirect: false,
            in_token_replacement: false,
            temporary_control: Vec::new(),
            temporary_copies: Vec::new(),
            foretold_this_turn: std::collections::HashSet::new(),
            plotted_cards: std::collections::HashSet::new(),
            plotted_this_turn: std::collections::HashSet::new(),
            triggered_once_per_turn_used: std::collections::HashSet::new(),
            per_subject_trigger_uses: std::collections::HashMap::new(),
            monarch: None,
            day_night: None,
            previous_turn_active: None,
        }
    }

    /// CR 724 — make `player` the monarch. No-op if they already are; emits
    /// `MonarchChanged` on a real change.
    pub(crate) fn set_monarch(&mut self, player: usize, events: &mut Vec<GameEvent>) {
        if self.monarch == Some(player) {
            return;
        }
        self.monarch = Some(player);
        events.push(GameEvent::MonarchChanged { player });
        self.return_monarch_guarded_exiles(Some(player), events);
    }

    /// CR 701.54 — the Ring tempts `player`. Bumps their temptation level
    /// (capped at 4) and lets them designate a creature they control as
    /// Ring-bearer. Choice is auto-resolved to their best creature (highest
    /// power, then toughness) — per-player UI selection is a follow-up
    /// (TODO.md). If they control no creature the bearer is unchanged.
    pub(crate) fn ring_tempts(&mut self, player: usize, events: &mut Vec<GameEvent>) {
        self.players[player].ring_temptations =
            (self.players[player].ring_temptations + 1).min(4);
        let computed = self.compute_battlefield();
        let pick = self
            .battlefield
            .iter()
            .filter(|c| c.controller == player && c.definition.is_creature())
            .filter_map(|c| {
                computed
                    .iter()
                    .find(|cp| cp.id == c.id)
                    .map(|cp| (c.id, cp.power, cp.toughness))
            })
            .max_by_key(|(_, p, t)| (*p, *t))
            .map(|(id, ..)| id);
        if let Some(id) = pick {
            self.players[player].ring_bearer = Some(id);
        }
        events.push(GameEvent::RingTempted {
            player,
            level: self.players[player].ring_temptations,
            bearer: self.players[player].ring_bearer,
        });
    }

    /// CR 701.54a/b — `player`'s current Ring-bearer, validated: the stored
    /// designation only counts while that creature is on the battlefield and
    /// still controlled by `player` (a control change clears the designation).
    pub fn effective_ring_bearer(&self, player: usize) -> Option<CardId> {
        let id = self.players[player].ring_bearer?;
        self.battlefield
            .iter()
            .find(|c| c.id == id && c.controller == player && c.definition.is_creature())
            .map(|c| c.id)
    }

    /// CR 731 — set the game's day/night designation, emitting
    /// `DayNightChanged` on a real change.
    /// CR 712 — flip one DFC permanent to its other face in place. The object
    /// is unchanged (counters/tapped/attachments persist); fires `Transformed`.
    pub(crate) fn transform_permanent(&mut self, id: CardId, events: &mut Vec<GameEvent>) {
        let Some(c) = self.battlefield_find_mut(id) else { return };
        if !c.transformed {
            let Some(back) = c.definition.back_face.as_ref().map(|b| (**b).clone()) else { return };
            c.front_face = Some(c.definition.clone());
            c.definition = std::sync::Arc::new(back);
            c.transformed = true;
        } else {
            let Some(front) = c.front_face.take() else { return };
            c.definition = front;
            c.transformed = false;
        }
        events.push(GameEvent::Transformed { card_id: id });
    }

    /// CR 310.10 — a battle whose last defense counter is removed is defeated.
    /// For a Siege the printed defeat trigger is "exile it, then cast it
    /// transformed": we transform the permanent to its back face and flicker it
    /// (exile, then re-enter under its controller as a new object) so the
    /// back-face permanent enters with summoning sickness and ETB triggers.
    /// Modeled as a state-based flicker rather than a stack cast, so it isn't
    /// separately counterable.
    pub(crate) fn defeat_battle(&mut self, id: CardId, events: &mut Vec<GameEvent>) {
        let Some(c) = self.battlefield_find(id) else { return };
        let controller = c.controller;
        let has_back = c.definition.back_face.is_some();
        let ctx = crate::game::effects::EffectContext::for_ability(id, controller, None);
        if !has_back {
            self.move_card_to(id, &crate::effect::ZoneDest::Exile, &ctx, events);
            return;
        }
        // Transform to the back face, then flicker it onto the battlefield.
        self.transform_permanent(id, events);
        if let Some(c) = self.battlefield_find_mut(id) {
            c.protected_by = None;
        }
        self.move_card_to(id, &crate::effect::ZoneDest::Exile, &ctx, events);
        self.move_card_to(
            id,
            &crate::effect::ZoneDest::Battlefield {
                controller: crate::effect::PlayerRef::Seat(controller),
                tapped: false,
            },
            &ctx,
            events,
        );
    }

    /// CR 711.2 — flip one flip-card permanent to its flipped face in place.
    /// The object is unchanged (counters/tapped/attachments persist); fires
    /// `Flipped`. No-op if already flipped or it has no flip face.
    pub(crate) fn flip_permanent(&mut self, id: CardId, events: &mut Vec<GameEvent>) {
        let Some(c) = self.battlefield_find_mut(id) else { return };
        if c.flip().is_some() {
            events.push(GameEvent::Flipped { card_id: id });
        }
    }

    pub(crate) fn set_day_night(&mut self, dn: crate::game::types::DayNight, events: &mut Vec<GameEvent>) {
        use crate::game::types::DayNight;
        if self.day_night == Some(dn) {
            return;
        }
        let was_transition = self.day_night.is_some();
        self.day_night = Some(dn);
        events.push(GameEvent::DayNightChanged { day_night: dn, was_transition });
        // CR 702.146f/g — daybound/nightbound DFCs flip with the day/night
        // cycle: front (daybound) ↔ back (nightbound).
        let want = match dn {
            DayNight::Night => Keyword::Daybound,
            DayNight::Day => Keyword::Nightbound,
        };
        let to_flip: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.definition.keywords.contains(&want))
            .map(|c| c.id)
            .collect();
        for id in to_flip {
            self.transform_permanent(id, events);
        }
    }

    /// CR 502.2 — the day/night turn-based check run as each turn begins.
    /// If it's day and the previous turn's active player cast no spells, it
    /// becomes night; if it's night and they cast two or more, it becomes
    /// day. No effect while the game is neither day nor night.
    pub(crate) fn check_day_night_transition(&mut self, events: &mut Vec<GameEvent>) {
        use crate::game::types::DayNight;
        let Some(current) = self.day_night else { return };
        let Some(prev) = self.previous_turn_active else { return };
        let cast = self.players.get(prev).map(|p| p.spells_cast_this_turn).unwrap_or(0);
        match current {
            DayNight::Day if cast == 0 => self.set_day_night(DayNight::Night, events),
            DayNight::Night if cast >= 2 => self.set_day_night(DayNight::Day, events),
            _ => {}
        }
    }

    /// Transient triggers granted to a permanent until EOT (Root
    /// Manipulation, Rabid Attack-style "creatures gain '…' EOT").
    /// Returns an empty slice when no grant is active — call sites can
    /// `.iter().chain(self.granted_triggers(id))` against the printed
    /// abilities without cloning.
    pub(crate) fn granted_triggers(
        &self,
        id: CardId,
    ) -> &[crate::card::TriggeredAbility] {
        self.granted_triggers_eot
            .get(&id)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Triggered abilities granted to `card` by battlefield
    /// `StaticEffect::GrantTriggeredAbility` statics ("All artifacts have
    /// '…'" — Kataki, War's Wage). Each fires as though printed on the
    /// matching permanent (CR 702.6e-style source binding).
    pub(crate) fn statics_granted_triggers_for(
        &self,
        card: &CardInstance,
    ) -> Vec<crate::card::TriggeredAbility> {
        let mut out = Vec::new();
        for src in &self.battlefield {
            for sa in &src.definition.static_abilities {
                if let crate::effect::StaticEffect::GrantTriggeredAbility { filter, ability } =
                    &sa.effect
                    && self.evaluate_requirement_static(
                        filter,
                        &Target::Permanent(card.id),
                        src.controller,
                        Some(src.id),
                    )
                {
                    out.push((**ability).clone());
                }
            }
        }
        // CR 721.2a — a station card's own `{N+}` triggered-ability striations,
        // active only while it has at least `min` charge counters.
        if !card.definition.station.is_empty() {
            let charges = card.counter_count(crate::card::CounterType::Charge);
            for band in card.definition.station.iter().filter(|b| charges >= b.min) {
                out.extend(band.triggers.iter().cloned());
            }
        }
        out
    }

    /// Triggered abilities granted to `card` by Equipment attached to it
    /// (CR 702.6e). Only the `triggers_on_equipment == false` abilities are
    /// surfaced here — they fire as though printed on the equipped creature
    /// (Tarrian's Soulcleaver's "whenever another artifact/creature dies, put
    /// a +1/+1 counter on equipped creature"). The `triggers_on_equipment`
    /// (Jitte-style) abilities fire off the Equipment via the dedicated
    /// combat-damage hook, so they're excluded to avoid double-firing.
    pub(crate) fn equip_granted_triggers_for(
        &self,
        card: &CardInstance,
    ) -> Vec<crate::card::TriggeredAbility> {
        let mut out = Vec::new();
        for eq in &self.battlefield {
            if eq.attached_to != Some(card.id) {
                continue;
            }
            let Some(bonus) = &eq.definition.equipped_bonus else { continue };
            if bonus.triggers_on_equipment {
                continue;
            }
            out.extend(bonus.triggered_abilities.iter().cloned());
        }
        out
    }

    /// Apply format-specific setup: starting life total, turn-1 draw
    /// rule, and (for Two-Headed Giant) the team partition + shared
    /// life pool.
    pub fn apply_format(&mut self, format: crate::format::Format) {
        let rules = format.rules();
        let life = if self.players.len() > 2 {
            rules.multiplayer_starting_life.unwrap_or(rules.starting_life)
        } else {
            rules.starting_life
        };
        for p in &mut self.players {
            p.life = life;
            p.starting_life = life;
        }
        self.skip_first_draw = self.players.len() <= 2;

        // Two-Headed Giant — Phase F. Default seating partitions
        // consecutive seat pairs into teams (0+1, 2+3, …) per
        // CR 810.2a and seeds each team's shared pool to the format's
        // starting life. Callers wanting a different pairing can
        // call `assign_teams` afterwards; the shared-life seeding
        // happens here regardless. An odd seat count leaves the
        // trailing odd seat as a singleton (silly setup, but keeps
        // the helper total — the caller likely wants `assign_teams`).
        if matches!(format, crate::format::Format::TwoHeadedGiant) {
            let n = self.players.len();
            let mut partitions: Vec<Vec<usize>> = Vec::new();
            let mut i = 0;
            while i < n {
                if i + 1 < n {
                    partitions.push(vec![i, i + 1]);
                    i += 2;
                } else {
                    partitions.push(vec![i]);
                    i += 1;
                }
            }
            self.teams = partitions
                .into_iter()
                .enumerate()
                .map(|(idx, members)| crate::team::Team {
                    id: crate::team::TeamId(idx),
                    members,
                    shared_life: Some(life),
                })
                .collect();
        }
    }

    /// Number of players that have not been eliminated.
    pub fn alive_count(&self) -> usize {
        self.players.iter().filter(|p| p.is_alive()).count()
    }

    /// Next non-eliminated seat strictly after `from` (wrapping). Returns
    /// `from` if no other alive players remain.
    pub fn next_alive_seat(&self, from: usize) -> usize {
        let n = self.players.len();
        for step in 1..=n {
            let i = (from + step) % n;
            if self.players[i].is_alive() {
                return i;
            }
        }
        from
    }

    /// Sort `seats` into APNAP order — active player first, then each other
    /// seat in turn order (CR 101.4). Used when a single effect affects
    /// "each player" so simultaneous-ish fan-outs (draws, mills, sacrifices)
    /// resolve in the canonical order rather than raw seat index.
    pub(crate) fn apnap_sort(&self, mut seats: Vec<usize>) -> Vec<usize> {
        let n = self.players.len().max(1);
        let active = self.active_player_idx;
        let rank = |seat: usize| -> usize {
            if seat == active {
                return 0;
            }
            let mut s = active;
            for r in 1..=n {
                s = self.next_alive_seat(s);
                if s == seat {
                    return r;
                }
                if s == active {
                    break;
                }
            }
            n + seat // eliminated / unreachable: stable tail
        };
        seats.sort_by_key(|&s| rank(s));
        seats
    }

    // ── Team partitioning ─────────────────────────────────────────────────────

    /// Team that contains `seat`. Falls back to a virtual singleton
    /// `TeamId(seat)` when `teams` is empty (e.g. snapshots from before
    /// the field was added).
    pub fn team_of(&self, seat: usize) -> crate::team::TeamId {
        for t in &self.teams {
            if t.members.contains(&seat) {
                return t.id;
            }
        }
        crate::team::TeamId(seat)
    }

    /// Seats sharing a team with `seat`, excluding `seat` itself. Empty
    /// for singleton-team seats.
    pub fn teammates(&self, seat: usize) -> Vec<usize> {
        let my_team = self.team_of(seat);
        for t in &self.teams {
            if t.id == my_team {
                return t.members.iter().copied().filter(|&s| s != seat).collect();
            }
        }
        Vec::new()
    }

    /// Seats on every team other than `seat`'s. Includes eliminated
    /// players; callers that need a live-only list should filter on
    /// `players[s].is_alive()` themselves.
    pub fn opponents_of(&self, seat: usize) -> Vec<usize> {
        if self.teams.is_empty() {
            // No teams declared — treat every other seat as an opponent.
            return (0..self.players.len()).filter(|&s| s != seat).collect();
        }
        let my_team = self.team_of(seat);
        let mut out = Vec::new();
        for t in &self.teams {
            if t.id != my_team {
                out.extend(t.members.iter().copied());
            }
        }
        out
    }

    /// True when `a` and `b` are on the same team. A seat is always its
    /// own teammate (returns true for `a == b`).
    pub fn same_team(&self, a: usize, b: usize) -> bool {
        self.team_of(a) == self.team_of(b)
    }

    // ── Life total helpers (Phase F) ──────────────────────────────────────

    /// Effective life total visible to `seat`. In 2HG (`Team.shared_life
    /// == Some(n)`) every member of the team sees the same number; in
    /// solo-team formats (1v1 / FFA / Commander) this is just the
    /// player's own `life` field. Callers checking lethal damage,
    /// "if you have ≤ X life" predicates, "the most life total" etc.
    /// should consult this rather than `players[seat].life`.
    pub fn effective_life(&self, seat: usize) -> i32 {
        if let Some(t) = self.teams.iter().find(|t| t.members.contains(&seat))
            && let Some(shared) = t.shared_life
        {
            return shared;
        }
        self.players[seat].life
    }

    /// Number of Equipment currently attached to permanent `id` (CR 301.5).
    /// The single source of truth for equipped-state checks — `IsEquipped`,
    /// `EquippedByAtLeast`, `SourceIsEquipped`, and the per-Equipment CDA all
    /// route through here.
    pub(crate) fn attached_equipment_count(&self, id: CardId) -> usize {
        self.battlefield
            .iter()
            .filter(|c| c.attached_to == Some(id) && c.definition.is_equipment())
            .count()
    }

    /// Apply a life delta to `seat` — gain for `delta > 0`, loss for
    /// `delta < 0`. Routes through the team's shared pool when set
    /// (Phase F — 2HG), else mutates `players[seat].life` directly.
    /// Returns the post-mutation effective life total.
    ///
    /// Per-turn counters (`life_gained_this_turn`) are bumped on the
    /// *seat* receiving the change — they're a "you" payoff and the
    /// triggering side is still a specific player. For 2HG, CR 810.8
    /// also propagates the gain to teammates' "you gain life"
    /// triggers; that broader fan-out is handled at trigger-scope
    /// resolution time (`EventScope::YourControl` would need a
    /// team-aware variant), not here. This helper only owns the
    /// state-mutation half.
    pub fn adjust_life(&mut self, seat: usize, delta: i32) -> i32 {
        if delta == 0 {
            return self.effective_life(seat);
        }
        // CR 119.7: if `seat` can't gain life and the delta would
        // increase their life total, drop the gain on the floor. The
        // 119.10 rider — "If a player gains 0 life, no life gain event
        // would occur, and these effects won't apply" — is honored
        // implicitly: the gain never happens, no LifeGained event is
        // emitted, the `life_gained_this_turn` counter isn't bumped.
        //
        // The check ORs the directly-settable `Player.cannot_gain_life`
        // flag (set by emblems / once-per-game effects) with the
        // dynamic battlefield scan via `player_cannot_gain_life_now`
        // (consults `StaticEffect::PlayerCannotGainLife` statics on
        // the live battlefield).
        // CR 614 — Tainted Remedy-style replacement: a would-be life *gain*
        // becomes an equal life *loss* instead. Applied before the cannot-
        // gain drop (the gain is replaced, not prevented) and re-routed as a
        // negative delta so the loss honors cannot-lose-life / shared pools.
        if delta > 0 && self.life_gain_becomes_loss_now(seat) {
            return self.adjust_life(seat, -delta);
        }
        if delta > 0 && self.player_cannot_gain_life_now(seat) {
            return self.effective_life(seat);
        }
        // CR 119.10 — a genuine life *gain* is increased by any active
        // "you gain that much plus N" replacement (Honor Troll). Folded in
        // before the gain applies so the bonus counts toward
        // `life_gained_this_turn` and any downstream lifegain triggers.
        let delta = if delta > 0 {
            delta.saturating_mul(self.life_gain_multiplier_now(seat))
        } else {
            delta
        };
        let delta = if delta > 0 {
            delta.saturating_add(self.life_gain_bonus_now(seat))
        } else {
            delta
        };
        // CR 119.8: symmetric drop for negative deltas (lose-life).
        if delta < 0 && self.player_cannot_lose_life_now(seat) {
            return self.effective_life(seat);
        }
        // CR 614 — Bloodletter of Aclazotz: an opponent losing life during the
        // Bloodletter controller's turn loses twice that much instead. Applied
        // after the cannot-lose drop so a locked player still loses nothing.
        let delta = if delta < 0 && self.life_loss_doubled_now(seat) {
            delta.saturating_mul(2)
        } else {
            delta
        };
        let team_idx = self
            .teams
            .iter()
            .position(|t| t.members.contains(&seat));
        let writes_to_shared = team_idx
            .and_then(|i| self.teams[i].shared_life)
            .is_some();

        let new_total = if writes_to_shared {
            let t = team_idx.unwrap();
            let current = self.teams[t].shared_life.unwrap();
            let next = current.saturating_add(delta);
            self.teams[t].shared_life = Some(next);
            next
        } else {
            let p = &mut self.players[seat];
            p.life = p.life.saturating_add(delta);
            p.life
        };

        if delta > 0 {
            self.players[seat].life_gained_this_turn =
                self.players[seat].life_gained_this_turn.saturating_add(delta as u32);
        } else {
            // delta < 0 — this player lost life (CR 119.3). Powers Spectacle.
            self.players[seat].lost_life_this_turn = true;
            self.players[seat].life_lost_this_turn =
                self.players[seat].life_lost_this_turn.saturating_add((-delta) as u32);
            // CR 702.179 — the active player's speed increases by 1 (capped at
            // 4), once on their own turn, the first time an opponent loses life.
            let active = self.active_player_idx;
            if active < self.players.len()
                && self.players[active].speed >= 1
                && self.players[active].speed < 4
                && !self.players[active].speed_increased_this_turn
                && !self.same_team(seat, active)
            {
                self.players[active].speed += 1;
                self.players[active].speed_increased_this_turn = true;
            }
        }
        new_total
    }

    /// Like [`adjust_life`] but returns the *applied* delta — after the
    /// cannot-gain/lose drops, gain→loss replacement, and gain bonuses
    /// (CR 119.7/119.10/614). Callers that emit `LifeGained`/`LifeLost`
    /// must use this so triggers don't fire on gains that never happened.
    pub fn adjust_life_applied(&mut self, seat: usize, delta: i32) -> i32 {
        let before = self.effective_life(seat);
        let after = self.adjust_life(seat, delta);
        after - before
    }

    /// Overwrite the effective life total for `seat` (Effect::SetLife
    /// path). Routes through the shared pool when set, else writes
    /// `players[seat].life` directly. Does not bump
    /// `life_gained_this_turn` (set-to-N isn't a "gain").
    pub fn set_life(&mut self, seat: usize, new_total: i32) {
        if let Some(t) = self.teams.iter_mut().find(|t| t.members.contains(&seat))
            && t.shared_life.is_some()
        {
            t.shared_life = Some(new_total);
            return;
        }
        self.players[seat].life = new_total;
    }

    // ── Commander identity & damage (Phase J / M) ──────────────────────────

    /// True if `card_id` is a commander for any player. Used by the
    /// Phase M 21-damage accumulator and by Phase L's cast-from-CZ
    /// (a non-commander has no business hitting that path).
    pub fn is_commander(&self, card_id: crate::card::CardId) -> bool {
        self.players
            .iter()
            .any(|p| p.commanders.contains(&card_id))
    }

    /// Add `amount` to the commander-damage tally for
    /// `(victim_seat, source_card_id)`. Caller is responsible for
    /// checking `is_commander(source)` before invoking — invalid
    /// entries would otherwise pollute the SBA's view. Phase M's
    /// damage paths gate on this check.
    ///
    /// The SBA (`check_state_based_actions`) consults the table
    /// after every life mutation, so no immediate action is required
    /// here beyond bumping the counter.
    pub fn record_commander_damage(
        &mut self,
        victim_seat: usize,
        source_card_id: crate::card::CardId,
        amount: u32,
    ) {
        if amount == 0 {
            return;
        }
        let entry = self
            .commander_damage
            .entry((victim_seat, source_card_id))
            .or_insert(0);
        *entry = entry.saturating_add(amount);
    }

    // ── Commander seating (Phase J) ────────────────────────────────────────

    /// Place each card in `defs` into `seat`'s command zone as a new
    /// `CardInstance`, and register the Commander zone-change
    /// replacement effect for each — CR 903.9b's "if a commander
    /// would be put into a graveyard, exile, hand, or library from
    /// anywhere, its owner may put it into the command zone
    /// instead." Phase L's cast-from-CZ machinery + commander-cast
    /// counter consult the command zone contents; this helper sets
    /// up that initial state.
    ///
    /// Returns the `CardId`s of the seated commanders so callers
    /// can use them as `Selector::CardInZone(Command)` targets, or
    /// pass them to test helpers.
    ///
    /// The replacement is registered with `optional: true` — CR 903.9b
    /// says the redirect is "may", so the owner can elect to let the
    /// commander land in the original zone (e.g. when they want to
    /// reanimate it from the graveyard rather than re-pay tax).
    /// `AutoDecider` defaults to "yes redirect" so tournament-style
    /// play matches expectations; tests can script the opposite via
    /// `ScriptedDecider` answering `DecisionAnswer::Bool(false)` to
    /// the `Decision::CommanderRedirect` prompt.
    pub fn seat_commanders(
        &mut self,
        seat: usize,
        defs: Vec<crate::card::CardDefinition>,
    ) -> Vec<crate::card::CardId> {
        let mut ids = Vec::with_capacity(defs.len());
        for def in defs {
            let id = crate::card::CardId(self.next_id);
            self.next_id = self.next_id.saturating_add(1);
            let card = crate::card::CardInstance::new(id, def, seat);
            self.players[seat].command.push(card);
            self.players[seat].commanders.push(id);

            // CR 903.9b replacement — graveyard / exile / hand /
            // library from anywhere → command zone. `from: None`
            // matches any origin; the destination set is the four
            // zones the rule names.
            self.register_replacement(crate::replacement::ReplacementEffect {
                id: crate::replacement::ReplacementId(0), // overwritten
                source: crate::replacement::ReplacementSource::Card(id),
                from: None,
                to_zones: vec![
                    crate::card::Zone::Graveyard,
                    crate::card::Zone::Exile,
                    crate::card::Zone::Hand,
                    crate::card::Zone::Library,
                ],
                redirect_to: crate::card::Zone::Command,
                optional: true,
            });
            ids.push(id);
        }
        ids
    }

    // ── Replacement effects (Phase H) ─────────────────────────────────────

    /// Register `effect` with the engine. Returns the assigned id so the
    /// caller can `unregister_replacement` it later (e.g. when the
    /// originating permanent leaves play). The caller-supplied `id`
    /// field is ignored — the engine stamps a fresh monotonic id.
    pub fn register_replacement(
        &mut self,
        mut effect: crate::replacement::ReplacementEffect,
    ) -> crate::replacement::ReplacementId {
        let id = crate::replacement::ReplacementId(self.next_replacement_id);
        self.next_replacement_id = self.next_replacement_id.saturating_add(1);
        effect.id = id;
        self.replacement_effects.push(effect);
        id
    }

    /// Drop the replacement with `id` if present. Returns true on hit.
    pub fn unregister_replacement(&mut self, id: crate::replacement::ReplacementId) -> bool {
        if let Some(pos) = self
            .replacement_effects
            .iter()
            .position(|r| r.id == id)
        {
            self.replacement_effects.remove(pos);
            true
        } else {
            false
        }
    }

    /// Walk the replacement registry for a zone change. Returns the
    /// destination zone after applying any matching replacement. Loops
    /// up to [`crate::replacement::MAX_REPLACEMENT_ITERATIONS`] times
    /// so chained replacements (e.g. graveyard → exile → command) can
    /// fully resolve while pathological loops still terminate. When
    /// the cap is hit, the most-recent destination is returned and a
    /// debug-assert fires.
    ///
    /// For `optional: true` replacements the card's owner is consulted
    /// via the installed `Decider` (`Decision::CommanderRedirect`).
    /// `AutoDecider` answers "yes" (matching the typical "save my
    /// commander" play), tests can script the opposite via
    /// `ScriptedDecider`. A declined optional replacement still
    /// counts as "applied" for CR 614.5 purposes so the same prompt
    /// isn't surfaced twice in one resolution walk.
    ///
    /// `&mut self` because the decider call is mutable. CR 616
    /// ordering ("affected card's controller chooses") is
    /// approximated by registration order.
    pub fn resolve_zone_change(
        &mut self,
        card_id: crate::card::CardId,
        from: crate::card::Zone,
        mut to: crate::card::Zone,
    ) -> crate::card::Zone {
        use crate::replacement::{ReplacementSource, MAX_REPLACEMENT_ITERATIONS};
        // Note: CR 122.1h finality counter redirect is applied at the
        // call site (`remove_from_battlefield_to_graveyard_raw`) because by
        // the time we reach this resolver the card has already been
        // removed from the battlefield. The call site passes
        // `Zone::Exile` instead of `Zone::Graveyard` when finality is
        // present.
        let mut applied: Vec<crate::replacement::ReplacementId> = Vec::new();
        for _ in 0..MAX_REPLACEMENT_ITERATIONS {
            let mut fired = false;
            // Clone the small set of metadata we need so we can mutate
            // `self.decider` inside the loop without borrow-conflict
            // with `self.replacement_effects`.
            let candidates: Vec<_> = self
                .replacement_effects
                .iter()
                .map(|r| {
                    (
                        r.id,
                        r.source.clone(),
                        r.from,
                        r.to_zones.clone(),
                        r.redirect_to,
                        r.optional,
                    )
                })
                .collect();
            for (rid, source, r_from, to_zones, redirect_to, optional) in candidates {
                if applied.contains(&rid) {
                    // CR 614.5 — a replacement effect can apply at most
                    // once to a given event. Skip ones we've already
                    // used in this resolution.
                    continue;
                }
                match source {
                    ReplacementSource::Card(target) if target != card_id => continue,
                    ReplacementSource::Card(_) => {}
                }
                if let Some(f) = r_from
                    && f != from
                {
                    continue;
                }
                if !to_zones.contains(&to) {
                    continue;
                }
                // Optional replacement → consult the decider. Today
                // the only optional replacement we register is the
                // Commander redirect (CR 903.9b), so the
                // `CommanderRedirect` decision shape is the right
                // surface. If `optional` were used for some other
                // redirect later, this branch would need a generic
                // `OptionalReplacement` decision instead.
                if optional {
                    let answer = self.decider.decide(&crate::decision::Decision::CommanderRedirect {
                        commander: card_id,
                        would_be: to,
                    });
                    let say_yes = matches!(answer, crate::decision::DecisionAnswer::Bool(true));
                    applied.push(rid);
                    if !say_yes {
                        // Don't apply, but mark as asked so we don't
                        // re-prompt on this resolution.
                        continue;
                    }
                } else {
                    applied.push(rid);
                }
                to = redirect_to;
                fired = true;
                break;
            }
            if !fired {
                return to;
            }
        }
        debug_assert!(false, "replacement-effect resolution hit iteration cap");
        to
    }

    /// Number of `StaticEffect::DoubleTokens` permanents `seat` controls
    /// on the battlefield. Used by `Effect::CreateToken` to scale the
    /// token count by `2^n` — one Adrix and Nev, Twincasters in play
    /// means twice as many tokens are minted; two doublers means four
    /// times as many; etc. (CR 614.13: multiple replacement effects
    /// apply in any order chosen by the controller, but all functionally
    /// multiply rather than just add.)
    pub fn token_doublers_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoubleTokens))
                    .count() as u32
            })
            .sum()
    }

    /// Number of `StaticEffect::DoubleCounters` permanents `seat` controls
    /// on the battlefield. Used by `Effect::AddCounter` to scale the counter
    /// count by `2^n` per CR 614.16's "if one or more counters would be put
    /// on a permanent" replacement. One Doubling Season → 2×; one Doubling
    /// Season + one Hardened Scales → 4× (multiplicative, matching the
    /// printed Oracle).
    pub fn counter_doublers_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoubleCounters))
                    .count() as u32
            })
            .sum()
    }

    /// Cursed Wombat — if `cid`'s controller has a
    /// `StaticEffect::CounterAmplifierOncePerTurn` and this permanent hasn't
    /// been amplified yet this turn, add one extra +1/+1 counter (subject to the
    /// controller's counter doublers) and mark it. The extra placement does not
    /// re-trigger the amplifier ("only once each turn").
    pub(crate) fn amplify_counter_once_per_turn(
        &mut self,
        cid: crate::card::CardId,
        events: &mut Vec<GameEvent>,
    ) {
        use crate::card::CounterType;
        use crate::effect::StaticEffect;
        if self.permanents_amplified_counter_this_turn.contains(&cid) {
            return;
        }
        let Some(ctrl) = self.battlefield_find(cid).map(|c| c.controller) else { return };
        let has = self.battlefield.iter().any(|c| {
            c.controller == ctrl
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::CounterAmplifierOncePerTurn))
        });
        if !has {
            return;
        }
        self.permanents_amplified_counter_this_turn.insert(cid);
        let n = self.scaled_counter_count(ctrl, CounterType::PlusOnePlusOne, 1, true);
        if n == 0 {
            return;
        }
        if let Some(c) = self.battlefield_find_mut(cid) {
            c.add_counters(CounterType::PlusOnePlusOne, n);
            events.push(GameEvent::CounterAdded {
                card_id: cid,
                counter_type: CounterType::PlusOnePlusOne,
                count: n,
            });
        }
    }

    /// Number of `StaticEffect::ExtraPlusOneCounters` permanents `seat`
    /// controls — each adds one to a +1/+1 counter placement onto one of
    /// `seat`'s creatures (Hardened Scales). Applied additively before the
    /// `DoubleCounters` multiplier.
    pub fn plus_counter_adders_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::ExtraPlusOneCounters))
                    .count() as u32
            })
            .sum()
    }

    /// CR 614.16 counter-placement replacement chain for a `base`-count
    /// placement of `kind` onto a permanent controlled by `ctrl`. Applies, in
    /// order: Hardened Scales additive (+1/+1 only, creature only), Vizier of
    /// Remedies shave (-1/-1 only, creature only), then Doubling-Season-class
    /// doubling. Centralized so every counter-add site (AddCounter, Support,
    /// enters-with-counters, …) replaces consistently.
    pub fn scaled_counter_count(
        &self,
        ctrl: usize,
        kind: crate::card::CounterType,
        base: u32,
        is_creature: bool,
    ) -> u32 {
        use crate::card::CounterType;
        let mut n = base;
        if is_creature {
            // Winding Constrictor: +1 to any counter kind on a creature you control.
            n = n.saturating_add(self.extra_any_kind_adders_for(ctrl));
        }
        if is_creature && kind == CounterType::PlusOnePlusOne {
            n = n.saturating_add(self.plus_counter_adders_for(ctrl));
        }
        if is_creature && kind == CounterType::MinusOneMinusOne {
            n = n.saturating_sub(self.minus_counter_reduction_for(ctrl));
        }
        // +1/+1-only doublers (Branching Evolution, The Earth Crystal) compose
        // multiplicatively with the all-kinds Doubling-Season doublers.
        if is_creature && kind == CounterType::PlusOnePlusOne {
            for _ in 0..self.plus_counter_doublers_for(ctrl) {
                n = n.saturating_mul(2);
            }
        }
        for _ in 0..self.counter_doublers_for(ctrl) {
            n = n.saturating_mul(2);
        }
        n
    }

    /// CR 614.16 scaling for player-bound counters (poison): Winding
    /// Constrictor's "+1 to any counters you'd get" adder, then any all-kinds
    /// counter doublers. Players aren't creatures, so the `+1/+1`-only and
    /// creature-gated modifiers don't apply — this is the player analogue of
    /// [`scaled_counter_count`]. `base == 0` produces 0 (CR 119.10-style no-op).
    pub fn scaled_player_counter_count(&self, seat: usize, base: u32) -> u32 {
        if base == 0 {
            return 0;
        }
        let mut n = base.saturating_add(self.extra_any_kind_adders_for(seat));
        for _ in 0..self.counter_doublers_for(seat) {
            n = n.saturating_mul(2);
        }
        n
    }

    /// Central poison-placement funnel (CR 122 / 614.16): scales `base` by
    /// the poisoned player's counter modifiers (Winding Constrictor adder +
    /// doublers), applies Melira's "instead you get one and no more this
    /// turn" cap (CR 614), bumps `poison_counters`, and emits `PoisonAdded`.
    /// Every poison site routes here — `Effect::AddPoison`,
    /// `AddCounter(Player)`, proliferate, and infect/toxic combat damage.
    /// Returns the number of counters actually applied.
    pub(crate) fn add_poison(
        &mut self,
        seat: usize,
        base: u32,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        let mut n = self.scaled_player_counter_count(seat, base);
        // CR 614 — Melira, the Living Cure: "If you would get one or more
        // poison counters, instead you get one poison counter and you can't
        // get additional poison counters this turn."
        if n > 0 && self.player_has_static(seat, |se| {
            matches!(se, crate::effect::StaticEffect::PoisonCappedAtOnePerTurn)
        }) {
            n = if self.players[seat].poison_capped_this_turn { 0 } else { 1 };
            self.players[seat].poison_capped_this_turn = true;
        }
        // Melira, Sylvok Outcast — "You can't get poison counters."
        if n > 0 && self.player_has_static(seat, |se| {
            matches!(se, crate::effect::StaticEffect::PlayerCannotGetPoison)
        }) {
            n = 0;
        }
        if n == 0 {
            return 0;
        }
        self.players[seat].poison_counters += n;
        events.push(GameEvent::PoisonAdded { player: seat, amount: n });
        n
    }

    /// Whether `seat` controls a permanent printing a static ability matching
    /// `pred` — the shared query for player-scoped consult-at-a-funnel
    /// statics (draw doubling, proliferate doubling, poison caps, …).
    pub(crate) fn player_has_static(
        &self,
        seat: usize,
        pred: impl Fn(&crate::effect::StaticEffect) -> bool,
    ) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.static_abilities.iter().any(|sa| pred(&sa.effect))
        })
    }

    /// Number of `StaticEffect::DoublePlusOneCounters` permanents `seat`
    /// controls — each doubles a +1/+1 placement onto one of `seat`'s
    /// creatures (Branching Evolution / The Earth Crystal). Multiplicative,
    /// composing with the all-kinds `DoubleCounters` doublers.
    pub fn plus_counter_doublers_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoublePlusOneCounters))
                    .count() as u32
            })
            .sum()
    }

    /// Number of `StaticEffect::ExtraCounterAllKinds` permanents `seat`
    /// controls — each adds one to a placement of *any* counter kind onto one
    /// of `seat`'s creatures (Winding Constrictor). Additive, before doubling.
    pub fn extra_any_kind_adders_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::ExtraCounterAllKinds))
                    .count() as u32
            })
            .sum()
    }

    /// CR 614 — total energy-gain bonus for `seat` from `EnergyGainBonus`
    /// statics (Izzet Generatorium's "you get that many plus one {E} instead").
    pub fn energy_gain_bonus_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                StaticEffect::EnergyGainBonus { amount } => Some(amount),
                _ => None,
            })
            .sum()
    }

    /// CR 614.5 — how many -1/-1 counters to shave off a placement onto one
    /// of `seat`'s creatures (Vizier of Remedies; one per copy).
    pub fn minus_counter_reduction_for(&self, seat: usize) -> u32 {
        use crate::effect::StaticEffect;
        // Melira's full lock swallows any placement.
        if self.player_has_static(seat, |se| {
            matches!(se, StaticEffect::NoMinusCountersOnYourCreatures)
        }) {
            return u32::MAX;
        }
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat)
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::MinusCounterReduction))
                    .count() as u32
            })
            .sum()
    }

    /// Extra ETB counters granted by `StaticEffect::ChosenTypeEntersWithCounter`
    /// (Metallic Mimic). For a creature `entering` under `controller`, returns
    /// one counter spec per matching source: a *different* permanent the same
    /// player controls whose chosen creature type is among the entering
    /// creature's types. The entering card must already be on the battlefield.
    pub(crate) fn chosen_type_etb_counter_specs(
        &self,
        entering: CardId,
        controller: usize,
    ) -> Vec<(crate::card::CounterType, u32)> {
        use crate::effect::StaticEffect;
        let Some(ec) = self.battlefield.iter().find(|c| c.id == entering) else {
            return vec![];
        };
        if !ec.definition.is_creature() {
            return vec![];
        }
        let entering_types = ec.definition.subtypes.creature_types.clone();
        let mut specs = vec![];
        for src in &self.battlefield {
            if src.controller != controller || src.id == entering {
                continue;
            }
            let changeling = ec
                .definition
                .keywords
                .contains(&crate::card::Keyword::Changeling);
            for sa in &src.definition.static_abilities {
                match &sa.effect {
                    StaticEffect::ChosenTypeEntersWithCounter { kind }
                        if src
                            .chosen_creature_type
                            .is_some_and(|ct| entering_types.contains(&ct) || changeling)
                        => {
                            specs.push((*kind, 1));
                        }
                    StaticEffect::TypeEntersWithCounter { creature_type, kind }
                        if entering_types.contains(creature_type) || changeling =>
                    {
                        specs.push((*kind, 1));
                    }
                    // Giada — each other Angel enters with +1/+1 for each Angel
                    // you already control (the source counts, the entrant does
                    // not — it isn't yet on the battlefield at spec time here,
                    // but a token/reanimation entry already is, so exclude it).
                    StaticEffect::TypeEntersWithCountersPerControlled { creature_type, kind, per }
                        if entering_types.contains(creature_type) || changeling =>
                    {
                        let n = self
                            .battlefield
                            .iter()
                            .filter(|c| c.controller == controller && c.id != entering)
                            .filter(|c| {
                                self.evaluate_requirement_static(
                                    per,
                                    &crate::game::Target::Permanent(c.id),
                                    controller,
                                    Some(src.id),
                                )
                            })
                            .count() as u32;
                        if n > 0 {
                            specs.push((*kind, n));
                        }
                    }
                    // Master Biomancer — any other creature you control enters
                    // with additional counters equal to the source's live power.
                    StaticEffect::OtherCreaturesEnterWithCountersEqualToSourcePower { kind } => {
                        let n = src.power().max(0) as u32;
                        if n > 0 {
                            specs.push((*kind, n));
                        }
                    }
                    _ => {}
                }
            }
        }
        specs
    }

    /// CR 702.32 / 702.62 — a permanent with Fading N / Vanishing N enters
    /// with N fade / time counters. Called from both ETB paths after the
    /// permanent is on the battlefield.
    pub(crate) fn apply_fading_vanishing_etb(
        &mut self,
        cid: CardId,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        use crate::card::{CounterType, Keyword};
        let Some(card) = self.battlefield_find(cid) else { return };
        let spec = card.definition.keywords.iter().find_map(|k| match k {
            Keyword::Fading(n) => Some((CounterType::Fade, *n)),
            Keyword::Vanishing(n) => Some((CounterType::Time, *n)),
            _ => None,
        });
        let Some((kind, n)) = spec else { return };
        if n == 0 {
            return;
        }
        if let Some(card_mut) = self.battlefield_find_mut(cid) {
            card_mut.add_counters(kind, n);
        }
        events.push(crate::game::GameEvent::CounterAdded {
            card_id: cid,
            counter_type: kind,
            count: n,
        });
    }

    /// CR 702.183 — a permanent cast for its Impending cost enters with N
    /// time counters (stamped on `CardInstance.impending_counters` at cast).
    pub(crate) fn apply_impending_etb(
        &mut self,
        cid: CardId,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        use crate::card::CounterType;
        let Some(card) = self.battlefield_find_mut(cid) else { return };
        let n = card.impending_counters;
        if n == 0 {
            return;
        }
        card.impending_counters = 0;
        card.add_counters(CounterType::Time, n);
        events.push(crate::game::GameEvent::CounterAdded {
            card_id: cid,
            counter_type: CounterType::Time,
            count: n,
        });
    }

    /// CR 702.183 — at the beginning of the active player's end step, remove
    /// one time counter from each Impending permanent they control. Unlike
    /// Vanishing there's no sacrifice: when the last counter comes off the
    /// permanent simply stops being a non-creature (the layer effect reads
    /// the live counter count) and turns into a creature.
    pub(crate) fn process_impending(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        let affected: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.controller == active
                    && c.counter_count(CounterType::Time) > 0
                    && c.definition
                        .keywords
                        .iter()
                        .any(|k| matches!(k, Keyword::Impending(_)))
            })
            .map(|c| c.id)
            .collect();
        for id in affected {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.remove_counters(CounterType::Time, 1);
            }
            events.push(crate::game::GameEvent::CounterRemoved {
                card_id: id,
                counter_type: CounterType::Time,
                count: 1,
            });
        }
        events
    }

    /// CR 702.32 / 702.62 — at the beginning of the active player's upkeep,
    /// each Fading / Vanishing permanent they control removes a counter (and
    /// is sacrificed when it runs out). Processed as a turn-based action at
    /// upkeep before priority.
    pub(crate) fn process_fading_vanishing(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        // Snapshot the affected (id, fading?) pairs first to avoid borrow churn.
        let affected: Vec<(CardId, bool)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == active)
            .filter_map(|c| {
                c.definition.keywords.iter().find_map(|k| match k {
                    Keyword::Fading(_) => Some((c.id, true)),
                    Keyword::Vanishing(_) => Some((c.id, false)),
                    _ => None,
                })
            })
            .collect();
        for (id, is_fading) in affected {
            let kind = if is_fading { CounterType::Fade } else { CounterType::Time };
            let had = self.battlefield_find(id).map(|c| c.counter_count(kind)).unwrap_or(0);
            let sacrifice = if is_fading {
                // Fading: remove one; if none to remove, sacrifice.
                if had == 0 {
                    true
                } else {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.remove_counters(kind, 1);
                    }
                    events.push(crate::game::GameEvent::CounterRemoved {
                        card_id: id,
                        counter_type: kind,
                        count: 1,
                    });
                    false
                }
            } else {
                // Vanishing: remove one; sacrifice when the last is removed.
                if had > 0 {
                    if let Some(c) = self.battlefield_find_mut(id) {
                        c.remove_counters(kind, 1);
                    }
                    events.push(crate::game::GameEvent::CounterRemoved {
                        card_id: id,
                        counter_type: kind,
                        count: 1,
                    });
                }
                had <= 1
            };
            if sacrifice {
                // CR 700.4 — the shared sacrifice helper emits the full
                // event set (CreatureDied included) + die snapshot.
                self.sacrifice_one(id, active, &mut events);
            }
        }
        events
    }

    /// CR 702.24 — Cumulative upkeep. As a turn-based action at the active
    /// player's upkeep, each permanent they control with
    /// `Keyword::CumulativeUpkeep(cost)` gets one age counter; its controller
    /// then pays `cost` once per age counter on it (mana from the pool, life,
    /// or sacrificing matching permanents), or sacrifices the permanent.
    /// (Following `PayManaOrElse`, mana is auto-paid from the pool when
    /// affordable — an interactive pay prompt is a follow-up.)
    pub(crate) fn process_cumulative_upkeep(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, CumulativeUpkeepCost, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        let affected: Vec<(CardId, CumulativeUpkeepCost)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == active)
            .filter_map(|c| {
                c.definition.keywords.iter().find_map(|k| match k {
                    Keyword::CumulativeUpkeep(cost) => Some((c.id, cost.clone())),
                    _ => None,
                })
            })
            .collect();
        for (id, cost) in affected {
            if let Some(c) = self.battlefield_find_mut(id) {
                c.add_counters(CounterType::Age, 1);
            }
            events.push(crate::game::GameEvent::CounterAdded {
                card_id: id,
                counter_type: CounterType::Age,
                count: 1,
            });
            let n = self.battlefield_find(id).map(|c| c.counter_count(CounterType::Age)).unwrap_or(1);
            // A wants_ui controller gets a real pay-or-sacrifice trigger for
            // mana/life cumulative upkeeps (coin-flip and sacrifice kinds have
            // no meaningful decline and stay synchronous).
            if self.players.get(active).is_some_and(|p| p.wants_ui)
                && matches!(cost, CumulativeUpkeepCost::Mana(_) | CumulativeUpkeepCost::Life(_))
            {
                self.push_pending_trigger(
                    PendingTriggerPush {
                        source: id,
                        controller: active,
                        effect: Effect::CumulativeUpkeepPayOrSacrifice { cost: cost.clone() },
                        subject: None,
                        event_amount: 0,
                        mode: None,
                        intervening_if: None,
                    },
                    None,
                );
                continue;
            }
            let paid = match &cost {
                CumulativeUpkeepCost::Mana(mc) => {
                    // Total = cost × age counters (repeat the pip list N times).
                    let mut symbols = Vec::new();
                    for _ in 0..n {
                        symbols.extend(mc.symbols.iter().cloned());
                    }
                    self.players[active].mana_pool.pay(&crate::mana::ManaCost::new(symbols)).is_ok()
                }
                CumulativeUpkeepCost::Life(per) => {
                    let total = per * n;
                    // Auto-pay life only while it leaves the player alive.
                    if self.players[active].life > total as i32 {
                        let applied = self.adjust_life_applied(active, -(total as i32));
                        if applied < 0 {
                            events.push(crate::game::GameEvent::LifeLost { player: active, amount: (-applied) as u32 });
                        }
                        true
                    } else {
                        false
                    }
                }
                CumulativeUpkeepCost::FlipCoin => {
                    // Always payable — one flip per age counter; each fires
                    // the controller's win/lose-a-flip triggers (CR 705.1).
                    // Events are returned for the caller's single dispatch.
                    for _ in 0..n {
                        if self.flip_one_coin(active) {
                            events.push(crate::game::GameEvent::CoinFlipWon { player: active });
                        } else {
                            events.push(crate::game::GameEvent::CoinFlipLost { player: active });
                        }
                    }
                    true
                }
                CumulativeUpkeepCost::Sacrifice(filter) => {
                    // Need N matching permanents (other than the source) to pay.
                    let cands = self.sacrifice_candidates(active, filter, Some(id));
                    let cands: Vec<CardId> = cands.into_iter().filter(|&c| c != id).collect();
                    if cands.len() >= n as usize {
                        let pick = self.auto_pick_sacrifices(&cands, n as usize, Some(id), false, false);
                        for sid in pick {
                            self.sacrifice_one(sid, active, &mut events);
                        }
                        true
                    } else {
                        false
                    }
                }
            };
            if !paid {
                self.sacrifice_one(id, active, &mut events);
            }
        }
        events
    }

    /// CR 702.62d/e — remove one time counter from each suspended card the
    /// active player owns in exile; when the last counter comes off, cast
    /// the card without paying its mana cost (a creature so cast clears its
    /// summoning sickness — Suspend grants haste). Targets are auto-chosen,
    /// matching AutoDecider behavior for other free casts.
    pub(crate) fn process_suspend(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let active = self.active_player_idx;
        let mut events = Vec::new();
        // Snapshot suspended exiled cards (Suspend keyword + ≥1 time counter)
        // owned by the active player, so the borrow is released before casting.
        let suspended: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| {
                c.owner == active
                    && c.counter_count(CounterType::Time) > 0
                    && c.definition
                        .keywords
                        .iter()
                        .any(|k| matches!(k, Keyword::Suspend(..)))
            })
            .map(|c| c.id)
            .collect();
        for id in suspended {
            events.append(&mut self.remove_suspend_time_counter(id));
        }
        // CR 702.62e — cards that *gained* suspend (the card "Suspend")
        // tick on the same schedule even without the printed keyword.
        let granted: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| {
                c.owner == active && c.granted_suspend && c.counter_count(CounterType::Time) > 0
            })
            .map(|c| c.id)
            .collect();
        for id in granted {
            events.append(&mut self.remove_suspend_time_counter(id));
        }
        events
    }

    /// CR 702.29 — Echo. At the beginning of the controller's upkeep, each
    /// permanent they control with an unpaid echo (it came under their
    /// control since their last upkeep) is sacrificed unless its echo cost
    /// is paid: mana echoes auto-pay from the pool when affordable
    /// (matching `process_cumulative_upkeep`), `EchoDiscard` discards a
    /// card picked by the controller's decider. A `wants_ui` controller
    /// instead gets a real echo trigger on the stack whose resolution asks
    /// pay-or-sacrifice (`Effect::EchoPayOrSacrifice`).
    pub(crate) fn process_echo(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::Keyword;
        let active = self.active_player_idx;
        let mut events = Vec::new();
        let affected: Vec<(CardId, Option<crate::mana::ManaCost>)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == active && !c.echo_paid)
            .filter_map(|c| {
                c.definition.keywords.iter().find_map(|k| match k {
                    Keyword::Echo(cost) => Some((c.id, Some(cost.clone()))),
                    Keyword::EchoDiscard => Some((c.id, None)),
                    _ => None,
                })
            })
            .collect();
        for (id, cost) in affected {
            if self.players.get(active).is_some_and(|p| p.wants_ui) {
                self.push_pending_trigger(
                    PendingTriggerPush {
                        source: id,
                        controller: active,
                        effect: Effect::EchoPayOrSacrifice { mana_cost: cost },
                        subject: None,
                        event_amount: 0,
                        mode: None,
                        intervening_if: None,
                    },
                    None,
                );
                continue;
            }
            let paid = match &cost {
                // Auto-tap lands like a real payment — at upkeep the pool is
                // empty, so pool-only payment would always sacrifice.
                Some(mc) => match self.try_pay_with_auto_tap(active, mc) {
                    Ok(receipt) => {
                        events.extend(receipt.auto_events);
                        true
                    }
                    Err(_) => false,
                },
                None => {
                    // Echo—Discard a card: auto-discard the lowest-MV hand
                    // card if the hand isn't empty.
                    let pick = self.players[active]
                        .hand
                        .iter()
                        .min_by_key(|c| c.definition.cost.cmc())
                        .map(|c| c.id);
                    match pick {
                        Some(cid) => self.discard_card(active, cid, &mut events),
                        None => false,
                    }
                }
            };
            if paid {
                if let Some(c) = self.battlefield_find_mut(id) {
                    c.echo_paid = true;
                }
            } else {
                self.sacrifice_one(id, active, &mut events);
            }
        }
        events
    }

    /// Uvilda, Dean of Perfection — at the active player's upkeep, remove one
    /// hone counter from each instant/sorcery they own in exile with hone
    /// counters. When the last comes off, grant them permission to cast it
    /// from exile for {4} less (the printed "you may cast it" window).
    pub(crate) fn process_hone(&mut self) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, MayPlayDuration, MayPlayPermission};
        let active = self.active_player_idx;
        let turn = self.turn_number;
        let mut events = Vec::new();
        let honed: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| c.owner == active && c.counter_count(CounterType::Hone) > 0)
            .map(|c| c.id)
            .collect();
        for id in honed {
            let Some(card) = self.exile.iter_mut().find(|c| c.id == id) else { continue };
            card.remove_counters(CounterType::Hone, 1);
            events.push(crate::game::GameEvent::CounterRemoved {
                card_id: id,
                counter_type: CounterType::Hone,
                count: 1,
            });
            if card.counter_count(CounterType::Hone) > 0 {
                continue;
            }
            // Last hone counter removed — castable from exile for {4} less.
            let mut cost = card.definition.cost.clone();
            cost.reduce_generic(4);
            card.may_play_until = Some(MayPlayPermission {
                player: active,
                granted_turn: turn,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                exile_after: false,
            });
            card.granted_alt_cast_cost_eot = Some(cost);
        }
        events
    }

    /// Remove one time counter from a suspended card in exile; when the last
    /// is removed, free-cast it from exile (CR 702.62e–f). Shared by the
    /// upkeep tick (`process_suspend`) and accelerants (Deep-Sea Kraken).
    pub(crate) fn remove_suspend_time_counter(
        &mut self,
        id: CardId,
    ) -> Vec<crate::game::GameEvent> {
        use crate::card::CounterType;
        let mut events = Vec::new();
        let Some(card) = self.exile.iter_mut().find(|c| c.id == id) else { return events };
        if card.counter_count(CounterType::Time) == 0 {
            return events;
        }
        card.remove_counters(CounterType::Time, 1);
        events.push(crate::game::GameEvent::CounterRemoved {
            card_id: id,
            counter_type: CounterType::Time,
            count: 1,
        });
        if card.counter_count(CounterType::Time) > 0 {
            return events;
        }
        let owner = card.owner;
        // Last counter removed — cast it for free from exile. Compute an
        // auto-target against the card's effect (the owner chooses in real
        // play; we collapse to the AutoDecider's first-legal pick). Stamp the
        // suspend-cast flag so a creature gains haste on ETB (CR 702.62f).
        card.cast_from_suspend = true;
        let effect = card.definition.effect.clone();
        let auto_target = self.auto_target_for_effect_avoiding(&effect, owner, Some(id));
        // The suspending owner casts it; route priority so the cast helper
        // attributes it correctly.
        let saved_priority = self.priority.player_with_priority;
        self.priority.player_with_priority = owner;
        let cast = self.cast_card_for_free(
            owner,
            id,
            crate::card::Zone::Exile,
            auto_target,
            vec![],
            None,
            None,
            false,
        );
        self.priority.player_with_priority = saved_priority;
        // If it can't be cast (e.g. no legal target) CR 702.62e leaves it
        // exiled with 0 time counters.
        if let Ok(mut evs) = cast {
            events.append(&mut evs);
        }
        events
    }

    /// CR 702.62 accelerants — when `caster` casts a spell, tick a time
    /// counter off every opponent-owned suspended card that has
    /// `Keyword::SuspendAccelerant` (Deep-Sea Kraken).
    pub(crate) fn process_suspend_accelerants(
        &mut self,
        caster: usize,
    ) -> Vec<crate::game::GameEvent> {
        use crate::card::{CounterType, Keyword};
        let targets: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| {
                c.owner != caster
                    && c.counter_count(CounterType::Time) > 0
                    && c.definition.keywords.contains(&Keyword::SuspendAccelerant)
            })
            .map(|c| c.id)
            .collect();
        let mut events = Vec::new();
        for id in targets {
            events.append(&mut self.remove_suspend_time_counter(id));
        }
        events
    }

    /// CR 614.x — true if any active `StaticEffect::ExileNontokenCreaturesNotCast`
    /// (Containment Priest) is on the battlefield. Consulted by
    /// `place_card_in_dest` to reroute non-cast nontoken creatures to exile.
    pub fn nontoken_creature_etb_exile_active(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::ExileNontokenCreaturesNotCast))
        })
    }

    /// CR 614.2 — number of `StaticEffect::DoubleDamageDealt` permanents on
    /// the battlefield (controller-agnostic: Furnace of Rath doubles *all*
    /// damage). Damage is scaled by `2^n`; `n` doublers → `2^n×`.
    pub fn damage_doublers(&self) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::DoubleDamageDealt))
                    .count() as u32
            })
            .sum()
    }

    /// CR 614.5 — Solphim-style noncombat doublers that apply to this exact
    /// (source, target) pair: a `DoubleNoncombatDamageToOpponents` static
    /// whose controller also controls `source`, where `ent` is an opponent of
    /// that controller (a player or a permanent they control). Each match
    /// doubles the dealt amount. Only consulted on the noncombat funnel.
    pub fn noncombat_damage_doublers_for(
        &self,
        source: Option<CardId>,
        ent: crate::game::effects::EntityRef,
    ) -> u32 {
        use crate::effect::StaticEffect;
        use crate::game::effects::EntityRef;
        // Controller of the damage source (a battlefield permanent or the
        // resolving spell stamped by `resolve_spell`).
        let Some(src_ctrl) = source.and_then(|s| {
            self.computed_permanent(s).map(|cp| cp.controller).or_else(|| {
                match &self.resolving_source {
                    Some((id, caster, _)) if *id == s => Some(*caster),
                    _ => None,
                }
            })
        }) else {
            return 0;
        };
        // Affected player (a damaged player, or the controller of a damaged
        // permanent).
        let affected = match ent {
            EntityRef::Player(p) => Some(p),
            EntityRef::Permanent(c) => self.battlefield_find(c).map(|c| c.controller),
            EntityRef::Card(_) => None,
        };
        let Some(target_player) = affected else { return 0 };
        self.battlefield
            .iter()
            .map(|c| {
                if c.controller != src_ctrl || self.same_team(src_ctrl, target_player) {
                    return 0;
                }
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| {
                        matches!(sa.effect, StaticEffect::DoubleNoncombatDamageToOpponents)
                    })
                    .count() as u32
            })
            .sum()
    }

    /// Additive noncombat-damage bonus (Aether Revolt): sum of
    /// `NoncombatDamageToOpponentsBonus.amount` over battlefield permanents
    /// whose controller controls `source` and where `ent` is an opponent /
    /// their permanent. `while_revolt` statics only count while that
    /// controller has revolt (CR 702.139). Mirrors the doubler's scoping.
    pub fn noncombat_damage_bonus_for(
        &self,
        source: Option<CardId>,
        ent: crate::game::effects::EntityRef,
    ) -> u32 {
        use crate::effect::StaticEffect;
        use crate::game::effects::EntityRef;
        let Some(src_ctrl) = source.and_then(|s| {
            self.computed_permanent(s).map(|cp| cp.controller).or_else(|| {
                match &self.resolving_source {
                    Some((id, caster, _)) if *id == s => Some(*caster),
                    _ => None,
                }
            })
        }) else {
            return 0;
        };
        let affected = match ent {
            EntityRef::Player(p) => Some(p),
            EntityRef::Permanent(c) => self.battlefield_find(c).map(|c| c.controller),
            EntityRef::Card(_) => None,
        };
        let Some(target_player) = affected else { return 0 };
        self.battlefield
            .iter()
            .map(|c| {
                if c.controller != src_ctrl || self.same_team(src_ctrl, target_player) {
                    return 0;
                }
                c.definition
                    .static_abilities
                    .iter()
                    .filter_map(|sa| match sa.effect {
                        StaticEffect::NoncombatDamageToOpponentsBonus { amount, while_revolt } => {
                            if while_revolt
                                && !self.players[src_ctrl].permanent_left_battlefield_this_turn
                            {
                                None
                            } else {
                                Some(amount)
                            }
                        }
                        _ => None,
                    })
                    .sum::<u32>()
            })
            .sum()
    }

    /// CR 614.5 — number of `StaticEffect::HalveDamageDealt` permanents on
    /// the battlefield (Ghosts of the Innocent). Each halves the dealt
    /// amount, rounded down; applied after any doublers.
    pub fn damage_halvers(&self) -> u32 {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .map(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .filter(|sa| matches!(sa.effect, StaticEffect::HalveDamageDealt))
                    .count() as u32
            })
            .sum()
    }

    /// CR 615 — true when `tgt` has a `PreventAllCombatDamageToThis` self-static
    /// (Fog Bank, Guard Gomazoa), so the combat-damage resolver blanks damage
    /// marked on it.
    pub fn permanent_prevents_all_combat_damage_to_self(&self, tgt: crate::card::CardId) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield_find(tgt).is_some_and(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::PreventAllCombatDamageToThis))
        })
    }

    /// CR 615 — true when `tgt` prevents all combat damage to itself and
    /// prevention isn't switched off this turn. Consulted by the combat-damage
    /// resolver to zero damage marked on Fog Bank / Guard Gomazoa.
    pub fn combat_damage_prevented_to_self(&self, tgt: crate::card::CardId) -> bool {
        !self.damage_cant_be_prevented_this_turn
            && (self.permanent_prevents_all_combat_damage_to_self(tgt)
                || self.combat_damage_prevented_to_this_turn.contains(&tgt))
    }

    /// Scale a pending damage event by the global doubling/halving
    /// replacements (CR 614.2 / 614.5): every doubler ×2, then every
    /// halver ÷2 rounded down.
    pub fn scale_damage(&self, amount: u32) -> u32 {
        let d = self.damage_doublers().min(16);
        let h = self.damage_halvers().min(16);
        amount.saturating_mul(1 << d) >> h
    }

    /// Source- and target-aware damage scaling: the global doublers/halvers,
    /// the side-scoped ones (Gisela, Blade of Goldnight —
    /// `DoubleDamageToOpponents` doubles events hitting an opponent's side,
    /// `HalveDamageToYou` halves events hitting the controller's own side,
    /// CR 614.5), and the source-scoped additive bonus (Torbran —
    /// `AddDamageToOpponents`, applied before the multipliers).
    pub fn scale_damage_to(
        &self,
        source: Option<CardId>,
        ent: crate::game::effects::EntityRef,
        amount: u32,
    ) -> u32 {
        use crate::effect::StaticEffect;
        use crate::game::effects::EntityRef;
        let affected = match ent {
            EntityRef::Player(p) => Some(p),
            EntityRef::Permanent(c) => self.battlefield_find(c).map(|c| c.controller),
            EntityRef::Card(_) => None,
        };
        // Source identity: a battlefield permanent's computed colors +
        // controller, else the resolving spell stamped by `resolve_spell`.
        let source_info: Option<(usize, Vec<crate::mana::Color>)> = source.and_then(|s| {
            self.computed_permanent(s)
                .map(|cp| (cp.controller, cp.colors.clone()))
                .or_else(|| match &self.resolving_source {
                    Some((id, caster, colors)) if *id == s => {
                        Some((*caster, colors.clone()))
                    }
                    _ => None,
                })
        });
        let mut amount = amount;
        let mut d = self.damage_doublers();
        let mut h = self.damage_halvers();
        if let Some(p) = affected {
            // Stagger (Lightning, Army of One): damage to a staggered player
            // or their permanents is doubled until the registrant's next turn.
            d += self.staggered_damage_players.iter().filter(|(v, _)| *v == p).count() as u32;
            for c in &self.battlefield {
                for sa in &c.definition.static_abilities {
                    match &sa.effect {
                        StaticEffect::DoubleDamageToOpponents
                            if !self.same_team(c.controller, p) =>
                        {
                            d += 1;
                        }
                        StaticEffect::HalveDamageToYou if c.controller == p => h += 1,
                        StaticEffect::AddDamageToOpponents { source_color, amount: bonus }
                            if !self.same_team(c.controller, p) =>
                        {
                            // "+N if a [color] source you control" — needs a
                            // known source controlled by the static's owner.
                            if let Some((src_ctrl, src_colors)) = &source_info
                                && *src_ctrl == c.controller
                                && source_color.is_none_or(|sc| src_colors.contains(&sc))
                            {
                                amount = amount.saturating_add(*bonus);
                            }
                        }
                        StaticEffect::AddDamageToOpponentsPerCounter { kind }
                            if !self.same_team(c.controller, p) =>
                        {
                            if let Some((src_ctrl, _)) = &source_info
                                && *src_ctrl == c.controller
                            {
                                amount = amount.saturating_add(c.counter_count(*kind));
                            }
                        }
                        StaticEffect::AddDamageFromColorToPlayers { color, amount: bonus } => {
                            // Any source of the color, any player (CR 614.x).
                            if let Some((_, src_colors)) = &source_info
                                && src_colors.contains(color)
                            {
                                amount = amount.saturating_add(*bonus);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // Quest for Pure Flame — the source's controller doubles all damage
        // their sources deal this turn (CR 614.5, any permanent or player).
        if let Some((src_ctrl, _)) = &source_info
            && self.players[*src_ctrl].double_your_source_damage_this_turn
        {
            d += 1;
        }
        amount.saturating_mul(1 << d.min(16)) >> h.min(16)
    }

    /// CR 122.1 — true if any active `StaticEffect::CountersCantBePlaced`
    /// (Solemnity) is on the battlefield. While set, every counter-placement
    /// site drops the counters instead of adding them.
    pub fn counters_locked(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::CountersCantBePlaced))
        })
    }

    /// CR 614.6 — true if `card` bound for its owner's graveyard is exiled
    /// instead (Rest in Peace exiles everything; Leyline of the Void only
    /// an opponent's cards; Sanctifier en-Vec only black/red cards).
    /// Consulted by `route_to_graveyard` at every graveyard-placement site.
    pub fn graveyard_exiled_for(&self, card: &crate::card::CardInstance) -> bool {
        self.graveyard_exile_redirects(card).0
    }

    /// `(redirects, void_counter)` for `card`: whether some
    /// `ExileCardsBoundForGraveyard` static redirects it to exile, and
    /// whether any applicable redirect stamps a void counter on it
    /// (Dauthi Voidwalker).
    pub(crate) fn graveyard_exile_redirects(
        &self,
        card: &crate::card::CardInstance,
    ) -> (bool, bool) {
        use crate::effect::StaticEffect;
        let owner = card.owner;
        // Gaea's Will — the owner's graveyard-bound cards exile this turn.
        let mut redirects = self
            .players
            .get(owner)
            .is_some_and(|pl| pl.graveyard_bound_exiled_this_turn);
        let mut void = false;
        for c in &self.battlefield {
            for sa in &c.definition.static_abilities {
                if let StaticEffect::ExileCardsBoundForGraveyard {
                    opponents_only,
                    own_only,
                    colors,
                    card_types,
                    void_counter,
                } = &sa.effect
                {
                    let applies = (!opponents_only || c.controller != owner)
                        && (!own_only || c.controller == owner)
                        && colors.as_ref().is_none_or(|cs| {
                            card.definition.printed_colors().iter().any(|c| cs.contains(c))
                        })
                        && card_types.as_ref().is_none_or(|ts| {
                            card.definition.card_types.iter().any(|t| ts.contains(t))
                        });
                    if applies {
                        redirects = true;
                        void |= void_counter;
                    }
                }
            }
        }
        (redirects, void)
    }

    /// CR 614.5 — the actual mill count for `p` after doubling replacements
    /// (Bruvac the Grandiloquent: an opponent's mill is doubled, once per
    /// active static). 0 stays 0 (no event to replace).
    pub(crate) fn mill_count_for(&self, p: usize, n: usize) -> usize {
        use crate::effect::StaticEffect;
        if n == 0 {
            return 0;
        }
        let doublers = self
            .battlefield
            .iter()
            .filter(|c| {
                !self.same_team(c.controller, p)
                    && c.definition
                        .static_abilities
                        .iter()
                        .any(|sa| matches!(sa.effect, StaticEffect::OpponentMillDoubled))
            })
            .count()
            .min(16);
        n << doublers
    }

    /// CR 701.19c (Aven Mindcensor) — the number of cards from the top of
    /// the library `seat` may look at while searching, or `None` if
    /// unrestricted. The minimum across every opposing
    /// `OpponentsSearchTopN` static applies.
    pub(crate) fn search_top_limit_for(&self, seat: usize) -> Option<usize> {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .filter(|c| !self.same_team(c.controller, seat))
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                StaticEffect::OpponentsSearchTopN { count } => Some(count as usize),
                _ => None,
            })
            .min()
    }

    /// Leonin Arbiter — charge `seat` the search tax before a library
    /// search. Auto-pays {amount} per Arbiter from floating mana (the
    /// payment covers the rest of the turn); returns `false` when the tax
    /// is unpayable, in which case the search finds nothing.
    pub(crate) fn pay_search_tax(&mut self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        if self.search_tax_paid_this_turn.contains(&seat) {
            return true;
        }
        let tax: u32 = self
            .battlefield
            .iter()
            .flat_map(|c| c.definition.static_abilities.iter())
            .map(|sa| match sa.effect {
                StaticEffect::SearchTax { amount } => amount,
                _ => 0,
            })
            .sum();
        if tax == 0 {
            return true;
        }
        if self.players[seat].mana_pool.total() < tax {
            return false;
        }
        self.players[seat].mana_pool.spend_generic(tax);
        self.search_tax_paid_this_turn.push(seat);
        true
    }

    /// CR 614.10 — true when a battlefield static makes `player` skip
    /// `step` (Eon Hub's "players skip their upkeep steps", Stasis-style
    /// untap skipping).
    pub(crate) fn step_skipped_for(&self, player: usize, step: TurnStep) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition.static_abilities.iter().any(|sa| match &sa.effect {
                StaticEffect::SkipStep { step: s, all_players } if *s == step => {
                    *all_players || c.controller == player
                }
                _ => false,
            })
        })
    }

    /// Grafdigger's Cage — true while any battlefield permanent locks
    /// graveyards/libraries (no creature entries from them, no casts from
    /// them).
    pub(crate) fn graveyard_library_locked(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::GraveyardLibraryLockdown))
        })
    }

    /// True while graveyards specifically are locked — Grafdigger's Cage or
    /// the graveyard-only Kunoros lockdown.
    pub(crate) fn graveyard_locked(&self) -> bool {
        use crate::effect::StaticEffect;
        self.graveyard_library_locked()
            || self.battlefield.iter().any(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::GraveyardLockdown))
            })
    }

    /// Soulless Jailer — true while any battlefield permanent locks
    /// graveyard entries and graveyard/exile noncreature casts.
    pub(crate) fn graveyard_exile_locked(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::GraveyardExileLockdown))
        })
    }

    /// True when a static forbids `def` entering the battlefield from
    /// `zone` (Cage: creature cards from graveyards/libraries; Jailer:
    /// permanent cards from graveyards).
    pub(crate) fn battlefield_entry_from_zone_blocked(
        &self,
        def: &crate::card::CardDefinition,
        zone: crate::card::Zone,
    ) -> bool {
        use crate::card::Zone;
        (def.is_creature()
            && (matches!(zone, Zone::Graveyard) && self.graveyard_locked()
                || matches!(zone, Zone::Library) && self.graveyard_library_locked()))
            || (def.is_permanent()
                && zone == Zone::Graveyard
                && self.graveyard_exile_locked())
    }

    /// The escape cost `card` can be cast for from `p`'s graveyard: its
    /// printed Escape, else an Underworld-Breach grant (own mana cost +
    /// exile N) while `p` controls one.
    pub(crate) fn effective_escape(
        &self,
        card: &crate::card::CardInstance,
        p: usize,
    ) -> Option<(crate::mana::ManaCost, u32)> {
        use crate::effect::StaticEffect;
        if let Some((c, n)) = card.definition.has_escape() {
            return Some((c.clone(), n));
        }
        if card.definition.is_land() {
            return None;
        }
        self.battlefield.iter().find_map(|c| {
            if c.controller != p {
                return None;
            }
            c.definition.static_abilities.iter().find_map(|sa| match sa.effect {
                StaticEffect::GraveyardCardsHaveEscape { exile_count } => {
                    Some((card.definition.cost.clone(), exile_count))
                }
                _ => None,
            })
        })
    }

    /// True when `card` can be retraced from `p`'s graveyard: printed
    /// Retrace, else Six's "during your turn, nonland permanent cards in
    /// your graveyard have retrace" grant.
    pub(crate) fn effective_retrace(&self, card: &crate::card::CardInstance, p: usize) -> bool {
        use crate::effect::StaticEffect;
        if card.definition.has_retrace() {
            return true;
        }
        self.active_player_idx == p
            && !card.definition.is_land()
            && card.definition.is_permanent()
            && self.battlefield.iter().any(|c| {
                c.controller == p
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            StaticEffect::GraveyardPermanentsHaveRetraceDuringYourTurn
                        )
                    })
            })
    }

    /// The Ozolith — when a creature leaves the battlefield with counters,
    /// move them onto a `CollectsLeaverCounters` permanent its controller
    /// controls. Called at the leave funnels with the just-removed card.
    pub(crate) fn collect_leaver_counters(&mut self, card: &crate::card::CardInstance) {
        use crate::effect::StaticEffect;
        if !card.definition.is_creature() || card.counters.values().all(|&n| n == 0) {
            return;
        }
        let collector = self.battlefield.iter().find(|c| {
            c.controller == card.controller
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::CollectsLeaverCounters))
        });
        let Some(cid) = collector.map(|c| c.id) else { return };
        let counters = card.counters.clone();
        if let Some(c) = self.battlefield_find_mut(cid) {
            for (kind, n) in counters {
                if n > 0 {
                    c.add_counters(kind, n);
                }
            }
        }
    }

    /// True when a static forbids casting `def` from `zone` (Cage: any
    /// spell from graveyards/libraries; Jailer: noncreature spells from
    /// graveyards or exile).
    pub(crate) fn cast_from_zone_blocked(
        &self,
        caster: usize,
        def: &crate::card::CardDefinition,
        zone: crate::card::Zone,
    ) -> bool {
        use crate::card::Zone;
        (matches!(zone, Zone::Graveyard) && self.graveyard_locked())
            || (matches!(zone, Zone::Library) && self.graveyard_library_locked())
            || (!def.is_creature()
                && matches!(zone, Zone::Graveyard | Zone::Exile)
                && self.graveyard_exile_locked())
            // CR 601 — Drannith Magistrate: an opponent's permanent forbids
            // casting from any zone but the hand.
            || (!matches!(zone, Zone::Hand)
                && self.battlefield.iter().any(|c| {
                    !self.same_team(c.controller, caster)
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::OpponentsCantCastFromAnywhereButHand
                            )
                        })
                }))
    }

    /// CR 614 (Gather Specimens): if a creature would enter the battlefield
    /// under `intended`'s control while an opponent of theirs registered the
    /// steal-replacement this turn, it enters under that opponent instead.
    pub(crate) fn apply_etb_control_replacement(
        &self,
        card: &crate::card::CardInstance,
        intended: usize,
    ) -> usize {
        if !card.definition.is_creature() {
            return intended;
        }
        self.creature_etb_steal_this_turn
            .iter()
            .copied()
            .find(|b| !self.same_team(*b, intended))
            .unwrap_or(intended)
    }

    /// Mint a token onto the battlefield: applies the Gather Specimens ETB
    /// control replacement (CR 614), pushes the entry events, records
    /// `last_created_token(s)`, and fires self-source ETB triggers. The
    /// shared funnel for every token-creation site.
    pub(crate) fn mint_token_onto_battlefield(
        &mut self,
        def: CardDefinition,
        controller: usize,
        tapped: bool,
        events: &mut Vec<crate::game::GameEvent>,
    ) -> CardId {
        let id = self.next_id();
        let mut inst = crate::card::CardInstance::new_token(id, def, controller);
        // CR 111.2 — a token's owner is the player under whose control it
        // actually entered, so a stolen mint belongs to the thief.
        let ctrl = self.apply_etb_control_replacement(&inst, controller);
        inst.owner = ctrl;
        inst.controller = ctrl;
        inst.tapped = tapped;
        if inst.definition.is_creature() {
            self.players[ctrl].creatures_entered_this_turn.push(id);
        }
        if inst.definition.is_artifact() {
            self.players[ctrl].artifacts_entered_this_turn += 1;
        }
        if !inst.definition.is_land() {
            self.players[ctrl].nonland_permanents_entered_this_turn += 1;
        }
        if inst.definition.has_creature_type(crate::card::CreatureType::Mount)
            || inst.definition.is_vehicle()
        {
            self.players[ctrl].mounts_vehicles_entered_this_turn += 1;
        }
        self.battlefield.push(inst);
        // CR 707.2 — a token minted from a clone-y definition (Vizier of
        // Many Faces' embalm token) applies its `enters_as_copy` replacement
        // as it enters, before ETB triggers fire off the copied identity.
        // Mint-time type riders (embalm's "it's a Zombie in addition") are
        // re-layered on the copy: the delta vs the printed card survives.
        let minted_extra_types: Vec<crate::card::CreatureType> = {
            let c = self.battlefield.iter().find(|c| c.id == id).unwrap();
            if c.definition.enters_as_copy.is_some() {
                let printed = crabomination_base::registry::resolve_card(c.definition.name);
                c.definition
                    .subtypes
                    .creature_types
                    .iter()
                    .filter(|t| {
                        printed
                            .as_ref()
                            .is_none_or(|p| !p.subtypes.creature_types.contains(t))
                    })
                    .copied()
                    .collect()
            } else {
                vec![]
            }
        };
        if self.apply_enters_as_copy(id, ctrl, events) && !minted_extra_types.is_empty()
            && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == id)
        {
            let mut def = (*c.definition).clone();
            for t in minted_extra_types {
                if !def.subtypes.creature_types.contains(&t) {
                    def.subtypes.creature_types.push(t);
                }
            }
            c.definition = std::sync::Arc::new(def);
        }
        events.push(crate::game::GameEvent::TokenCreated { card_id: id });
        events.push(crate::game::GameEvent::PermanentEntered { card_id: id });
        self.last_created_token = Some(id);
        self.last_created_tokens.push(id);
        self.fire_self_etb_triggers(id, ctrl);
        // CR 614 — Academy Manufactor: a Clue/Food/Treasure mint becomes one
        // of each. The extra mints aren't re-replaced (CR 614.5).
        let minted_name = self
            .battlefield
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.definition.name)
            .unwrap_or_default();
        if !self.in_token_replacement
            && matches!(minted_name, "Clue" | "Food" | "Treasure")
            && self.battlefield.iter().any(|c| {
                c.controller == ctrl
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::ClueFoodTreasureMintsOneOfEach
                        )
                    })
            })
        {
            self.in_token_replacement = true;
            for other in [
                crabomination_base::tokens::clue_token(),
                crabomination_base::tokens::food_token(),
                crabomination_base::tokens::treasure_token(),
            ] {
                if other.name != minted_name {
                    self.mint_token_onto_battlefield(
                        crabomination_base::tokens::token_to_card_definition(&other),
                        ctrl,
                        tapped,
                        events,
                    );
                }
            }
            self.in_token_replacement = false;
        }
        id
    }

    /// Bookkeeping for a card leaving `p`'s graveyard: bumps the per-turn
    /// tally and emits `CardLeftGraveyard` so leaves-graveyard payoffs see
    /// mass exilers too (Rest in Peace, Go Blank, Surgical Extraction).
    pub(crate) fn note_left_graveyard(
        &mut self,
        p: usize,
        card_id: CardId,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        self.players[p].cards_left_graveyard_this_turn =
            self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
        events.push(crate::game::GameEvent::CardLeftGraveyard { player: p, card_id });
    }

    /// A spell removed from the stack by a counter / ward effect goes to
    /// its owner's graveyard — unless it's a copy (`is_token`), which
    /// simply ceases to exist (CR 707.10a): it never transits the
    /// graveyard, so no `CardPutIntoGraveyard` / descend bookkeeping may
    /// fire for it.
    pub(crate) fn countered_spell_off_stack(
        &mut self,
        card: crate::card::CardInstance,
        events: &mut Vec<crate::game::GameEvent>,
    ) {
        if card.is_token {
            return;
        }
        self.route_to_graveyard(card, events);
    }

    /// Place `card` into its owner's graveyard, or exile it instead when a
    /// graveyard-hate static (Rest in Peace / Leyline of the Void) is active
    /// for that owner. Pushes a `PermanentExiled` event and returns `true`
    /// when the card was redirected to exile, so callers can suppress their
    /// own graveyard-specific event (CardMilled, etc.).
    pub(crate) fn route_to_graveyard(
        &mut self,
        mut card: crate::card::CardInstance,
        events: &mut Vec<crate::game::GameEvent>,
    ) -> bool {
        // CR 702.183 — an Omen spell put into the graveyard from the stack
        // (countered, or fizzled on an illegal target) shuffles into its
        // owner's library instead.
        if card.omen_casting {
            use rand::seq::SliceRandom;
            let owner = card.owner;
            card.omen_casting = false;
            card.spliced_effects.clear();
            card.counters.clear();
            self.players[owner].library.push(card);
            self.players[owner].library.shuffle(&mut rand::rng());
            return false;
        }
        // CR 702.47e — splice changes are lost when the spell leaves the stack.
        card.spliced_effects.clear();
        // CR 122.2 — counters don't survive the zone change (replacement
        // riders below add to the new object afterward).
        card.counters.clear();
        card.keyword_counters.clear();
        // CR 712.16 — a melded shell dies as its two component cards.
        if !card.meld_parts.is_empty() {
            let mut card = card;
            let mut any_exiled = false;
            for part in std::mem::take(&mut card.meld_parts) {
                any_exiled |= self.route_to_graveyard(part, events);
            }
            return any_exiled;
        }
        // CR 702.140e — a merged (mutated) permanent dies as its components.
        if !card.mutate_stack.is_empty() {
            let mut card = card;
            let mut any_exiled = false;
            for part in std::mem::take(&mut card.mutate_stack) {
                any_exiled |= self.route_to_graveyard(part, events);
            }
            return any_exiled;
        }
        let owner = card.owner;
        // CR 614.6 — "shuffle into its owner's library instead" (Darksteel
        // Colossus). The card never touches the graveyard.
        if card.definition.shuffles_into_library_instead {
            use rand::seq::SliceRandom;
            self.players[owner].library.push(card);
            let mut rng = rand::rng();
            self.players[owner].library.shuffle(&mut rng);
            return false;
        }
        if self.graveyard_exiled_for(&card) || card.disturb_back_exiles() {
            let cid = card.id;
            let mut card = card;
            if self.graveyard_exile_redirects(&card).1 {
                card.add_counters(crate::card::CounterType::Void, 1);
            }
            self.exile.push(card);
            events.push(crate::game::GameEvent::PermanentExiled { card_id: cid });
            true
        } else {
            let cid = card.id;
            let is_land = card.definition.card_types.contains(&crate::card::CardType::Land);
            self.players[owner].send_to_graveyard(card);
            events.push(crate::game::GameEvent::CardPutIntoGraveyard {
                player: owner,
                card_id: cid,
                is_land,
            });
            false
        }
    }

    /// CR 702.66 — true if player `seat` controls a permanent granting
    /// "spells you cast have delve" (Teval, Arbiter of Virtue). Lets the
    /// cast path accept a delve-cards list on any spell, not just those
    /// printed with `Keyword::Delve`.
    pub fn controller_grants_spells_delve(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::SpellsYouCastHaveDelve))
        })
    }

    /// True if `seat` cannot gain life *right now*, per CR 119.7. ORs:
    /// 1. The directly-settable `Player.cannot_gain_life` flag (set by
    ///    emblems / once-per-game state — currently dormant; reserved for
    ///    permanent grants).
    /// 2. Any active `StaticEffect::PlayerCannotGainLife` on the
    ///    battlefield whose `target` resolves to include `seat`.
    ///
    /// Consulted by `GameState::adjust_life` to drop positive deltas
    /// targeting `seat` on the floor.
    pub fn player_cannot_gain_life_now(&self, seat: usize) -> bool {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        if self.players[seat].life_locked_this_turn {
            return true;
        }
        if self.players[seat].cannot_gain_life || self.players[seat].cannot_gain_life_this_turn {
            return true;
        }
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| {
                if let StaticEffect::PlayerCannotGainLife { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            })
        })
    }

    /// CR 615.12 — True if any active `StaticEffect::DamageCantBePrevented`
    /// is on the battlefield (Sulfuric Vortex, Sunspine Lynx). Consulted by
    /// `apply_prevention_shields` to bypass every shield.
    pub fn damage_cant_be_prevented_now(&self) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|src| {
            src.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::DamageCantBePrevented))
        })
    }

    /// CR 614 — True if `seat`'s life loss should be doubled right now: it's an
    /// opponent of a player who controls an `OpponentLifeLossDoubledDuringYourTurn`
    /// permanent and it's that controller's turn (Bloodletter of Aclazotz).
    pub fn life_loss_doubled_now(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|src| {
            src.controller == self.active_player_idx
                && !self.same_team(seat, src.controller)
                && src.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::OpponentLifeLossDoubledDuringYourTurn)
                })
        })
    }

    /// CR 119.8 — True if `seat` cannot lose life right now. Mirror of
    /// `player_cannot_gain_life_now`. Scans the battlefield for any
    /// `StaticEffect::PlayerCannotLoseLife` whose `target` resolves to
    /// include `seat`. Consulted by `adjust_life` (negative deltas) and
    /// by the lose-life paths (`Effect::LoseLife`, drain-target gates).
    pub fn player_cannot_lose_life_now(&self, seat: usize) -> bool {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        if self.players[seat].life_locked_this_turn {
            return true;
        }
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| {
                if let StaticEffect::PlayerCannotLoseLife { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            })
        })
    }

    /// CR 104.3d — true when `seat` can't lose the game right now: Angel's
    /// Grace this turn, a Platinum-Angel static they control, or an
    /// opponent's Abyssal-Persecutor static ("your opponents can't lose").
    /// Concession (CR 104.3a) bypasses this.
    pub fn player_cant_lose_game(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        if self.players[seat].cant_lose_this_turn {
            return true;
        }
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| match sa.effect {
                StaticEffect::ControllerCantLoseGame => src.controller == seat,
                StaticEffect::ControllerCantWinGame => !self.same_team(src.controller, seat),
                _ => false,
            })
        })
    }

    /// Phyrexian Unlife — true when `seat` controls a
    /// `ControllerDoesntLoseFromLife` static: the life-≤-0 loss SBA is
    /// skipped, and at ≤ 0 life all damage to them lands as poison.
    pub fn player_unlife_active(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|src| {
            src.controller == seat
                && src.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::ControllerDoesntLoseFromLife)
                })
        })
    }

    /// CR 104.3d — true when `seat` can't win the game right now: their own
    /// Abyssal-Persecutor static, an opponent's Platinum-Angel static, or an
    /// opponent under Angel's Grace this turn. Gates the win *effects*
    /// (`Effect::WinGame`, `WinInsteadOfDrawFromEmpty`); elimination-based
    /// endings are already blocked by the paired can't-lose checks.
    pub fn player_cant_win_game(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        if self
            .players
            .iter()
            .enumerate()
            .any(|(i, pl)| !self.same_team(i, seat) && pl.cant_lose_this_turn)
        {
            return true;
        }
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| match sa.effect {
                StaticEffect::ControllerCantWinGame => src.controller == seat,
                StaticEffect::ControllerCantLoseGame => !self.same_team(src.controller, seat),
                _ => false,
            })
        })
    }

    /// Angel's Grace / Worship — clamp a would-be damage life delta so it
    /// can't take `seat` below 1 life while a floor effect is active. The
    /// damage itself is still dealt (CR 614 — only the life reduction is
    /// replaced); a life total already ≤ 1 just doesn't move.
    pub(crate) fn clamp_damage_to_life_floor(&self, seat: usize, amount: u32) -> u32 {
        use crate::effect::StaticEffect;
        let floored = self.players[seat].damage_floor_this_turn
            || self.battlefield.iter().any(|src| {
                src.controller == seat
                    && src.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            StaticEffect::DamageWontReduceControllerLifeBelowOne { requires_creature }
                            if !requires_creature
                                || self.battlefield.iter().any(|c| {
                                    c.controller == seat && c.definition.is_creature()
                                })
                        )
                    })
            });
        if floored {
            amount.min(self.effective_life(seat).saturating_sub(1).max(0) as u32)
        } else {
            amount
        }
    }

    /// CR 705.3 — coin-flip advantage for `seat` from active
    /// `StaticEffect::CoinFlipAdvantage` permanents (Krark's Thumb). Summed
    /// so multiple sources stack; added to `Player.coin_flip_advantage` by
    /// the `Effect::FlipCoin` resolver.
    /// CR 705.1/705.3 — flip one coin for `player`, honoring Krark's-Thumb
    /// style advantage (replay + treat as heads if any replay is heads).
    /// Returns true for heads.
    pub(crate) fn flip_one_coin(&mut self, player: usize) -> bool {
        let advantage = self.players.get(player).map(|p| p.coin_flip_advantage).unwrap_or(0)
            + self.coin_flip_advantage_now(player);
        let mut heads = false;
        for _ in 0..(advantage as usize + 1) {
            let answer = self.decider.decide(&crate::decision::Decision::CoinFlip { player });
            if matches!(answer, crate::decision::DecisionAnswer::Bool(true)) {
                heads = true;
            }
        }
        heads
    }

    pub fn coin_flip_advantage_now(&self, seat: usize) -> u32 {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield.iter().map(|src| {
            src.definition.static_abilities.iter().filter(|sa| {
                if let StaticEffect::CoinFlipAdvantage { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            }).count() as u32
        }).sum()
    }

    /// CR 614 — True if a would-be life *gain* by `seat` should be replaced
    /// with an equal life *loss* (Tainted Remedy). Scans the battlefield for
    /// any active `StaticEffect::LifeGainBecomesLoss` whose `target` includes
    /// `seat`.
    pub fn life_gain_becomes_loss_now(&self, seat: usize) -> bool {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield.iter().any(|src| {
            src.definition.static_abilities.iter().any(|sa| {
                if let StaticEffect::LifeGainBecomesLoss { target } = &sa.effect {
                    match target {
                        PlayerStaticTarget::Controller => src.controller == seat,
                        PlayerStaticTarget::EachOpponent => src.controller != seat,
                        PlayerStaticTarget::EachPlayer => true,
                    }
                } else {
                    false
                }
            })
        })
    }

    /// CR 119.10 / 614 — total life-gain bonus currently applied to `seat`
    /// by `StaticEffect::LifeGainBonus` statics (Honor Troll's "+1 to each
    /// gain"). Bonuses from multiple sources stack additively.
    pub fn life_gain_bonus_now(&self, seat: usize) -> i32 {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield
            .iter()
            .flat_map(|src| {
                src.definition.static_abilities.iter().filter_map(move |sa| {
                    if let StaticEffect::LifeGainBonus { target, amount } = &sa.effect {
                        let hits = match target {
                            PlayerStaticTarget::Controller => src.controller == seat,
                            PlayerStaticTarget::EachOpponent => src.controller != seat,
                            PlayerStaticTarget::EachPlayer => true,
                        };
                        hits.then_some(*amount)
                    } else {
                        None
                    }
                })
            })
            .sum()
    }

    /// CR 614 — combined life-gain multiplier currently applied to `seat` by
    /// `StaticEffect::LifeGainMultiplier` statics (Rhox Faithmender's "twice").
    /// Multiple multipliers compound; returns 1 when none are active.
    pub fn life_gain_multiplier_now(&self, seat: usize) -> i32 {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield
            .iter()
            .flat_map(|src| {
                src.definition.static_abilities.iter().filter_map(move |sa| {
                    if let StaticEffect::LifeGainMultiplier { target, factor } = &sa.effect {
                        let hits = match target {
                            PlayerStaticTarget::Controller => src.controller == seat,
                            PlayerStaticTarget::EachOpponent => src.controller != seat,
                            PlayerStaticTarget::EachPlayer => true,
                        };
                        hits.then_some(*factor)
                    } else {
                        None
                    }
                })
            })
            .fold(1, |acc, f| acc.saturating_mul(f))
    }

    /// CR 121.2b — the smallest per-turn draw cap currently imposed on
    /// `seat` by any active `StaticEffect::CapDrawsPerTurn`, or `None` if
    /// the seat may draw freely. Multiple caps take the strictest (min).
    pub fn draw_cap_for(&self, seat: usize) -> Option<u32> {
        use crate::effect::{PlayerStaticTarget, StaticEffect};
        self.battlefield
            .iter()
            .flat_map(|src| {
                src.definition.static_abilities.iter().filter_map(move |sa| {
                    if let StaticEffect::CapDrawsPerTurn { target, max } = &sa.effect {
                        let hits = match target {
                            PlayerStaticTarget::Controller => src.controller == seat,
                            PlayerStaticTarget::EachOpponent => src.controller != seat,
                            PlayerStaticTarget::EachPlayer => true,
                        };
                        hits.then_some(*max)
                    } else {
                        None
                    }
                })
            })
            .min()
    }

    /// Replace the current team partition. Every seat must appear in
    /// exactly one entry; partitions must be non-empty. Used by team
    /// formats (2HG) after `new()` to group seats.
    pub fn assign_teams(
        &mut self,
        partitions: Vec<Vec<usize>>,
    ) -> Result<(), crate::team::TeamError> {
        let n = self.players.len();
        let mut seen = vec![false; n];
        for (i, part) in partitions.iter().enumerate() {
            if part.is_empty() {
                return Err(crate::team::TeamError::EmptyTeam(i));
            }
            for &seat in part {
                if seat >= n {
                    return Err(crate::team::TeamError::UnknownSeat {
                        seat,
                        num_players: n,
                    });
                }
                if seen[seat] {
                    return Err(crate::team::TeamError::DuplicateSeat(seat));
                }
                seen[seat] = true;
            }
        }
        for (seat, was_seen) in seen.iter().enumerate() {
            if !was_seen {
                return Err(crate::team::TeamError::MissingSeat(seat));
            }
        }
        self.teams = partitions
            .into_iter()
            .enumerate()
            .map(|(i, members)| crate::team::Team {
                id: crate::team::TeamId(i),
                members,
                shared_life: None,
            })
            .collect();
        Ok(())
    }

    /// The player who currently holds priority.
    pub fn player_with_priority(&self) -> usize {
        self.priority.player_with_priority
    }

    /// Give priority to the active player and reset consecutive passes.
    pub(crate) fn give_priority_to_active(&mut self) {
        self.priority.player_with_priority = self.active_player_idx;
        self.priority.consecutive_passes = 0;
    }

    // ── Layer system ──────────────────────────────────────────────────────────

    /// Compute the current derived state of all battlefield permanents after
    /// applying all active continuous effects in layer order.
    pub fn compute_battlefield(&self) -> Vec<ComputedPermanent> {
        if let Some(fx) = self.frozen_effects() {
            return crate::game::layers::apply_layers(&self.battlefield, &fx);
        }
        crate::game::layers::apply_layers(&self.battlefield, &self.gather_continuous_effects())
    }

    /// Run `f` with the gathered continuous-effect set memoized, so every
    /// `computed_permanent` / `compute_battlefield` call inside reuses one
    /// gather instead of rebuilding the full effect set per call. Sound by
    /// construction: `f` only receives `&GameState`, so none of the gather's
    /// inputs (battlefield, continuous effects, attachments…) can change
    /// while frozen. The memo fills lazily on the first computed read, so a
    /// scope that never needs the layer system costs nothing. Nested freezes
    /// reuse the outer memo. Use this around any read-only loop that filters
    /// or inspects many permanents.
    pub fn with_frozen_layers<R>(&self, f: impl FnOnce(&Self) -> R) -> R {
        self.layer_freeze.lock().depth += 1;
        // Decrement on drop (not after `f`) so a panicking assertion inside
        // a test closure can't leave a stale memo behind.
        struct Unfreeze<'a>(&'a GameState);
        impl Drop for Unfreeze<'_> {
            fn drop(&mut self) {
                let mut st = self.0.layer_freeze.lock();
                st.depth -= 1;
                if st.depth == 0 {
                    st.memo = None;
                }
            }
        }
        let _guard = Unfreeze(self);
        f(self)
    }

    /// Enter a freeze scope without a closure — pair with
    /// [`freeze_layers_pop`](Self::freeze_layers_pop). Prefer
    /// [`with_frozen_layers`](Self::with_frozen_layers) (panic-safe) except
    /// on recursion-hot paths where the closure+guard frame cost matters.
    pub(crate) fn freeze_layers_push(&self) {
        self.layer_freeze.lock().depth += 1;
    }

    /// Exit a freeze scope opened by [`freeze_layers_push`](Self::freeze_layers_push).
    pub(crate) fn freeze_layers_pop(&self) {
        let mut st = self.layer_freeze.lock();
        st.depth -= 1;
        if st.depth == 0 {
            st.memo = None;
        }
    }

    /// The memoized continuous-effect set when inside a
    /// [`with_frozen_layers`](Self::with_frozen_layers) scope (gathering and
    /// caching it on first use), else `None`.
    fn frozen_effects(&self) -> Option<std::sync::Arc<Vec<ContinuousEffect>>> {
        // Mid-gather reads see the printed-types fallback (`in_layer_gather`
        // reentrancy guard); don't serve or populate the memo from them.
        if self.in_layer_gather.load(std::sync::atomic::Ordering::Relaxed) {
            return None;
        }
        {
            let st = self.layer_freeze.lock();
            if st.depth == 0 {
                return None;
            }
            if let Some(fx) = &st.memo {
                return Some(fx.clone());
            }
        }
        // First computed read in this scope: gather outside the lock (the
        // gather itself re-enters `frozen_effects` via guarded eval paths).
        let fx = std::sync::Arc::new(self.gather_continuous_effects());
        let mut st = self.layer_freeze.lock();
        st.memo = Some(fx.clone());
        Some(fx)
    }

    /// Collect every continuous effect currently active in the game: the
    /// resolved-spell/ability effects in `continuous_effects`, plus the
    /// implicit effects derived each recompute from static abilities,
    /// equipment attachments, combat keyword grants, characteristic-
    /// defining P/T, life-gain pump/anthem tables, and graveyard-resident
    /// anthems. Shared by [`compute_battlefield`] (applies to every
    /// permanent) and [`computed_permanent`] (applies to just one).
    fn gather_continuous_effects(&self) -> Vec<ContinuousEffect> {
        // Reentrancy guard — see `in_layer_gather`. Passes below that
        // evaluate selection requirements must not recurse back into
        // `computed_permanent`.
        use std::sync::atomic::Ordering;
        self.in_layer_gather.store(true, Ordering::Relaxed);
        let out = self.gather_continuous_effects_inner();
        self.in_layer_gather.store(false, Ordering::Relaxed);
        out
    }

    fn gather_continuous_effects_inner(&self) -> Vec<ContinuousEffect> {
        // Include static-ability effects from permanents currently on the battlefield.
        let mut all_effects: Vec<ContinuousEffect> = self.continuous_effects.clone();
        for card in &self.battlefield {
            // CR 613.7a — static-ability effects carry the source object's
        // timestamp (entry-stamped; id-order fallback for unstamped objects).
        let ts = card.object_timestamp();
            let mut effects = static_ability_to_effects(card, ts);
            // Team-aware static abilities: `static_ability_to_effects` is a
            // free function with no GameState handle, so it can't fill in
            // `AllOpponents.friendly_seats` itself. Patch them now using
            // the source's actual team membership — in 1v1 / FFA this is
            // `[source_controller]` and behaves identically to the legacy
            // single-seat check; in team formats (2HG) it lists every
            // teammate so a Crackling Drake-style "creatures opponents
            // control" anthem doesn't accidentally buff the source's
            // partner.
            for e in &mut effects {
                if let AffectedPermanents::AllOpponents {
                    source_controller,
                    friendly_seats,
                    ..
                } = &mut e.affected
                    && friendly_seats.is_empty()
                {
                    let mut seats = self.teammates(*source_controller);
                    seats.push(*source_controller);
                    *friendly_seats = seats;
                }
            }
            all_effects.extend(effects);
        }
        // CR 114 — static-ability emblems (Vivien Reid's −8 anthem). Emblems
        // have no battlefield object, so synthesize a CardInstance per emblem
        // (controller = owner) and reuse `static_ability_to_effects`. The
        // source id sits in a high sentinel range so it can't collide with a
        // real card; the duration is remapped to `Indefinite` since emblems
        // never leave the command zone.
        for (seat, player) in self.players.iter().enumerate() {
            for (ei, emblem) in player.emblems.iter().enumerate() {
                if emblem.statics.is_empty() {
                    continue;
                }
                let synth_def = crate::card::CardDefinition {
                    name: "Emblem",
                    static_abilities: emblem.statics.clone(),
                    ..Default::default()
                };
                let sid = CardId(u32::MAX - (seat as u32 * 256 + ei as u32));
                let mut synth = CardInstance::new(sid, synth_def, seat);
                synth.controller = seat;
                for mut e in static_ability_to_effects(&synth, sid.0 as u64) {
                    e.duration = EffectDuration::Indefinite;
                    if let AffectedPermanents::AllOpponents {
                        source_controller,
                        friendly_seats,
                        ..
                    } = &mut e.affected
                        && friendly_seats.is_empty()
                    {
                        let mut seats = self.teammates(*source_controller);
                        seats.push(*source_controller);
                        *friendly_seats = seats;
                    }
                    all_effects.push(e);
                }
                // Live-conditional team anthems (`PumpTeamIf`) don't fold
                // through `static_ability_to_effects` — evaluate the gate
                // against the emblem's owner here (Ellywick's −7 emblem).
                for sa in &synth.definition.static_abilities {
                    let crate::effect::StaticEffect::PumpTeamIf {
                        condition,
                        applies_to,
                        power,
                        toughness,
                        keywords,
                    } = &sa.effect
                    else {
                        continue;
                    };
                    let ctx =
                        crate::game::effects::EffectContext::for_ability(sid, seat, None);
                    if !self.evaluate_predicate(condition, &ctx) {
                        continue;
                    }
                    let Some(affected) = selector_to_affected(applies_to, &synth) else {
                        continue;
                    };
                    if *power != 0 || *toughness != 0 {
                        all_effects.push(ContinuousEffect {
                            timestamp: synth.object_timestamp(),
                            source: sid,
                            affected: affected.clone(),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::Modify),
                            duration: EffectDuration::Indefinite,
                            modification: Modification::ModifyPowerToughness(*power, *toughness),
                        });
                    }
                    for kw in keywords {
                        all_effects.push(ContinuousEffect {
                            timestamp: synth.object_timestamp(),
                            source: sid,
                            affected: affected.clone(),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: EffectDuration::Indefinite,
                            modification: Modification::AddKeyword(kw.clone()),
                        });
                    }
                }
            }
        }
        // CR 702.6 — Equipment attachment statics. Each Equipment with a
        // live `attached_to` link and an `equipped_bonus` confers +P/+T
        // (layer 7c) and keyword grants (layer 6) on the creature it's
        // attached to, for as long as the Equipment stays on the battlefield.
        // The stale-link SBA in `stack.rs` clears `attached_to` when the
        // equipped creature leaves, so a dangling link can't leak a bonus.
        for card in &self.battlefield {
            let Some(bonus) = &card.definition.equipped_bonus else { continue };
            let Some(target) = card.attached_to else { continue };
            // Only apply while the target is still a creature on the bf.
            if !self.battlefield.iter().any(|c| c.id == target) {
                continue;
            }
            // Flat bonus plus optional board-scaled bonus (Nettlecyst: +1/+1
            // for each artifact/enchantment the Equipment's controller controls).
            let (mut bp, mut bt) = (bonus.power, bonus.toughness);
            if let Some(scale) = &bonus.scale {
                let n = match (
                    &scale.count_self_counters,
                    &scale.count_graveyard,
                    &scale.count_all_graveyards,
                ) {
                    (Some(kind), _, _) => card.counter_count(*kind) as i32,
                    (None, Some(gy_filter), _) => self.players[card.controller]
                        .graveyard
                        .iter()
                        .filter(|c| {
                            self.evaluate_requirement_on_card(gy_filter, c, card.controller)
                        })
                        .count() as i32,
                    (None, None, Some(all_gy_filter)) => self
                        .players
                        .iter()
                        .flat_map(|pl| pl.graveyard.iter())
                        .filter(|c| {
                            self.evaluate_requirement_on_card(all_gy_filter, c, card.controller)
                        })
                        .count() as i32,
                    (None, None, None) => self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == card.controller
                                && self.evaluate_requirement_on_card(
                                    &scale.filter,
                                    c,
                                    card.controller,
                                )
                        })
                        .count() as i32,
                };
                bp += n * scale.per_power;
                bt += n * scale.per_toughness;
            }
            if bp != 0 || bt != 0 {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(bp, bt),
                });
            }
            // "Loses all abilities AND has [keywords]" auras (Heliod's
            // Punishment): the removal must precede the aura's own keyword
            // grants so they survive (same timestamp → stable insertion
            // order; CR 613.7 grant-after-removal).
            if bonus.remove_abilities {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::RemoveAllAbilities,
                });
            }
            for kw in &bonus.keywords {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(kw.clone()),
                });
            }
            // "During your turn, equipped creature has [keyword]" (Dragoon's
            // Lance) — layer-6 grant gated on the source controller's turn.
            if self.active_player_idx == card.controller {
                for kw in &bonus.during_your_turn_keywords {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![target]),
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
            // Characteristic-overriding Auras (Ichthyomorphosis,
            // One with the Stars): set base P/T (7b), card/creature types,
            // and colors on the host while attached.
            let push_mod = |effects: &mut Vec<ContinuousEffect>, layer, sublayer, m| {
                effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![target]),
                    layer,
                    sublayer,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: m,
                });
            };
            if let Some((p, t)) = bonus.set_base_pt {
                push_mod(&mut all_effects, Layer::L7PowerTough, Some(PtSublayer::SetValue),
                    Modification::SetPowerToughness(p, t));
            }
            // Aettir and Priwen — base P/T X/X where X is the Equipment
            // controller's life total (layer 7b). Reads life directly (no
            // layer dependency), so it's reentrancy-safe.
            if bonus.set_base_pt_controller_life {
                let x = self.players[card.controller].life.max(0);
                push_mod(&mut all_effects, Layer::L7PowerTough, Some(PtSublayer::SetValue),
                    Modification::SetPowerToughness(x, x));
            }
            if let Some(types) = &bonus.set_card_types {
                push_mod(&mut all_effects, Layer::L4Type, None,
                    Modification::SetCardTypes(types.clone()));
            }
            if let Some(types) = &bonus.set_creature_types {
                push_mod(&mut all_effects, Layer::L4Type, None,
                    Modification::SetCreatureTypes(types.clone()));
            }
            for ct in &bonus.add_creature_types {
                push_mod(&mut all_effects, Layer::L4Type, None,
                    Modification::AddCreatureType(*ct));
            }
            if let Some(types) = &bonus.set_land_types {
                push_mod(&mut all_effects, Layer::L4Type, None,
                    Modification::SetLandTypes(types.clone()));
            }
            if let Some(colors) = &bonus.set_colors {
                push_mod(&mut all_effects, Layer::L5Color, None,
                    Modification::SetColors(colors.clone()));
            }
            // Host-conditional riders ("as long as enchanted creature is
            // green, …" — Shield of the Oversoul). Evaluated against the
            // host's pre-layer state, like `EquipScale` above.
            for cond in &bonus.conditional {
                let host_matches = self
                    .battlefield
                    .iter()
                    .find(|c| c.id == target)
                    .is_some_and(|host| {
                        self.evaluate_requirement_on_card(&cond.host_filter, host, card.controller)
                    });
                if !host_matches {
                    continue;
                }
                if let Some(pred) = &cond.condition {
                    let ctx = crate::game::effects::EffectContext::for_ability(
                        card.id,
                        card.controller,
                        None,
                    );
                    if !self.evaluate_predicate(pred, &ctx) {
                        continue;
                    }
                }
                if cond.power != 0 || cond.toughness != 0 {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![target]),
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(cond.power, cond.toughness),
                    });
                }
                for kw in &cond.keywords {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![target]),
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
        }
        // CR 702.95 — Soulbond. A creature carrying a `soulbond_bonus` that's
        // paired confers the bonus on BOTH itself and its partner (P/T layer
        // 7c, keywords layer 6), for as long as both stay on the battlefield.
        for card in &self.battlefield {
            let Some(bonus) = &card.definition.soulbond_bonus else { continue };
            let Some(partner) = card.soulbond_partner else { continue };
            if !self.battlefield.iter().any(|c| c.id == partner) {
                continue;
            }
            for &id in &[card.id, partner] {
                if bonus.power != 0 || bonus.toughness != 0 {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![id]),
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(
                            bonus.power,
                            bonus.toughness,
                        ),
                    });
                }
                for kw in &bonus.keywords {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![id]),
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
        }
        // CR 702.151c — a Reconfigure Equipment isn't a creature while it's
        // attached to a creature. Strip the Creature card type at layer 4
        // (the +1/+1 it confers still scales off its own counters; its equip
        // bonus and exile ability are unaffected). Lion Sash.
        for card in &self.battlefield {
            if card.attached_to.is_some() && card.definition.has_reconfigure().is_some() {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(vec![card.id]),
                    layer: Layer::L4Type,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::RemoveCardType(crate::card::CardType::Creature),
                });
            }
        }
        // "Attacking creatures you control have <keyword>" (Blade Historian).
        // Resolved here because `affects()` can't see combat state — we read
        // the live `attacking` list and scope the grant to the source's own
        // attackers. Layer-6 keyword addition, like the equipment grants.
        if !self.attacking.is_empty() {
            for card in &self.battlefield {
                for sa in &card.definition.static_abilities {
                    let crate::effect::StaticEffect::GrantKeywordToAttackers { keyword } =
                        &sa.effect
                    else {
                        continue;
                    };
                    let ids: Vec<CardId> = self
                        .attacking
                        .iter()
                        .map(|a| a.attacker)
                        .filter(|id| {
                            self.battlefield
                                .iter()
                                .any(|c| c.id == *id && c.controller == card.controller)
                        })
                        .collect();
                    if ids.is_empty() {
                        continue;
                    }
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(ids),
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    });
                }
            }
        }
        // CR 700.9 — "Modified creatures you control have <keyword>"
        // (Kodama of the West Tree) and "attacking [tokens] you control have
        // <keyword>" (Bone-Cairn Butcher). `IsModified` (attachments) and
        // `IsAttacking` (combat state) both need the live battlefield, so
        // filters mentioning them resolve here into a Specific id list per
        // recompute; `affected_from_requirement` drops them on the static
        // path, so there's no double application.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::GrantKeyword { applies_to, keyword } = &sa.effect
                else {
                    continue;
                };
                let crate::effect::Selector::EachPermanent(req) = applies_to else { continue };
                if !requirement_needs_live_resolution(req) {
                    continue;
                }
                let ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        self.evaluate_requirement_static(
                            req,
                            &Target::Permanent(c.id),
                            card.controller,
                            Some(card.id),
                        )
                    })
                    .map(|c| c.id)
                    .collect();
                if ids.is_empty() {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(ids),
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(keyword.clone()),
                });
            }
        }
        // CR 700.9 / combat anthems — "attacking creatures you control get
        // +X/+X" (Orcish Oriflamme) and modified-creature P/T lords. Mirrors
        // the GrantKeyword loop above: `IsAttacking`/`IsModified` need the live
        // battlefield, so `PumpPT` statics over them resolve here into a
        // Specific id list per recompute; `affected_from_requirement` drops
        // them on the static path, so there's no double application.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpPT { applies_to, power, toughness } =
                    &sa.effect
                else {
                    continue;
                };
                let crate::effect::Selector::EachPermanent(req) = applies_to else { continue };
                if !requirement_needs_live_resolution(req) {
                    continue;
                }
                let ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        self.evaluate_requirement_static(
                            req,
                            &Target::Permanent(c.id),
                            card.controller,
                            Some(card.id),
                        )
                    })
                    .map(|c| c.id)
                    .collect();
                if ids.is_empty() {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(ids),
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(*power, *toughness),
                });
            }
        }
        // CR 613 — "this creature has <keyword> as long as it matches
        // <condition>" (Kor Duelist's "double strike while equipped"). The
        // condition reads live board state, so it resolves here per recompute
        // into a layer-6 self keyword.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SelfHasKeywordWhile { keyword, condition } =
                    &sa.effect
                else {
                    continue;
                };
                if !self.evaluate_requirement_static(
                    condition,
                    &Target::Permanent(card.id),
                    card.controller,
                    Some(card.id),
                ) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(keyword.clone()),
                });
            }
        }
        // Predicate-gated sibling of the loop above — the condition reads live
        // board state relative to the source (e.g. "you control another Faerie").
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SelfHasKeywordWhilePredicate { keyword, condition } =
                    &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(card.id, 0, None);
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(keyword.clone()),
                });
            }
        }
        // CR 700.5 / Theros gods — "isn't a creature unless your devotion
        // to [colors] ≥ threshold." Emit a layer-4 RemoveCardType(Creature)
        // self-effect while the gate is unmet; reading devotion needs the
        // live GameState, so it can't route through static_ability_to_effects.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::NotCreatureWhileDevotionBelow {
                    colors,
                    threshold,
                } = &sa.effect
                else {
                    continue;
                };
                if (self.devotion_to(card.controller, colors) as u32) < *threshold {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::RemoveCardType(CardType::Creature),
                    });
                }
            }
        }
        // CR 613 — Opalescence / Starfield of Nyx: each other non-Aura
        // enchantment becomes an `MV/MV` creature. Starfield gates on the
        // controller holding five or more enchantments, so the set is gathered
        // state-aware rather than through `static_ability_to_effects`.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::NonAuraEnchantmentsAreCreatures {
                    yours_only,
                    requires_five,
                } = &sa.effect
                else {
                    continue;
                };
                if *requires_five
                    && self
                        .battlefield
                        .iter()
                        .filter(|c| {
                            c.controller == card.controller
                                && c.definition.card_types.contains(&CardType::Enchantment)
                        })
                        .count()
                        < 5
                {
                    continue;
                }
                use crate::card::{EnchantmentSubtype, SelectionRequirement as R};
                let mut req = R::Enchantment
                    .and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura))))
                    .and(R::OtherThanSource);
                if *yours_only {
                    req = req.and(R::ControlledByYou);
                }
                let affected = AffectedPermanents::CardMatch {
                    source_controller: card.controller,
                    requirement: Box::new(req),
                };
                let ts = card.object_timestamp();
                all_effects.push(ContinuousEffect {
                    timestamp: ts,
                    source: card.id,
                    affected: affected.clone(),
                    layer: Layer::L4Type,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddCardType(CardType::Creature),
                });
                all_effects.push(ContinuousEffect {
                    timestamp: ts,
                    source: card.id,
                    affected,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::SetValue),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::SetPowerToughnessToManaValue,
                });
            }
        }
        // Sliver Legion — "each [type] gets +P/+T for each OTHER [type]".
        // The bonus differs per affected permanent (it excludes itself), so
        // this is gathered state-aware: one Specific effect per matching
        // permanent, scaled by the live count minus one.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpPTPerOtherOfType {
                    creature_type,
                    power,
                    toughness,
                } = &sa.effect
                else {
                    continue;
                };
                let matching: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| c.definition.subtypes.creature_types.contains(creature_type))
                    .map(|c| c.id)
                    .collect();
                let others = matching.len().saturating_sub(1) as i32;
                if others == 0 {
                    continue;
                }
                for id in matching {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![id]),
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(
                            others * power,
                            others * toughness,
                        ),
                    });
                }
            }
        }
        // Coat of Arms — each creature gets +P/+T for each OTHER creature that
        // shares ≥1 creature type with it (Changeling shares all types). The
        // bonus is per-creature (shared-type count differs per subject), so it's
        // gathered state-aware like Sliver Legion above.
        {
            use crate::card::Keyword;
            let creatures: Vec<(CardId, &Vec<crate::card::CreatureType>, bool)> = self
                .battlefield
                .iter()
                .filter(|c| c.definition.is_creature())
                .map(|c| {
                    (c.id, &c.definition.subtypes.creature_types,
                     c.definition.keywords.contains(&Keyword::Changeling))
                })
                .collect();
            for src in &self.battlefield {
                for sa in &src.definition.static_abilities {
                    let crate::effect::StaticEffect::PumpPerSharedType { power, toughness } =
                        &sa.effect
                    else {
                        continue;
                    };
                    for (id, types, changeling) in &creatures {
                        let shared = creatures
                            .iter()
                            .filter(|(oid, otypes, ochange)| {
                                oid != id
                                    && (*changeling
                                        || *ochange
                                        || otypes.iter().any(|t| types.contains(t)))
                            })
                            .count() as i32;
                        if shared == 0 {
                            continue;
                        }
                        all_effects.push(ContinuousEffect {
                            timestamp: src.object_timestamp(),
                            source: src.id,
                            affected: AffectedPermanents::Specific(vec![*id]),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::Modify),
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::ModifyPowerToughness(
                                shared * power,
                                shared * toughness,
                            ),
                        });
                    }
                }
            }
        }
        // War Balloon — "as long as this has N+ [kind] counters, it's a
        // creature." Emit a layer-4 AddCardType(Creature) self-effect while
        // the count holds (printed P/T already carry the stats).
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SelfIsCreatureWhileCountersAtLeast { kind, n } =
                    &sa.effect
                else {
                    continue;
                };
                if card.counter_count(*kind) >= *n {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddCardType(CardType::Creature),
                    });
                }
            }
        }
        // Idol of False Gods — "as long as this has N+ [kind] counters, it has
        // [keyword]." Layer-6 keyword grant while the count holds.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SelfHasKeywordWhileCountersAtLeast { kind, n, keyword } =
                    &sa.effect
                else {
                    continue;
                };
                if card.counter_count(*kind) >= *n {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    });
                }
            }
        }
        // CR 702.183 — Impending: a permanent with the Impending keyword isn't
        // a creature while it has a time counter. Emit a layer-4
        // RemoveCardType(Creature) self-effect while counters remain.
        for card in &self.battlefield {
            let is_impending = card
                .definition
                .keywords
                .iter()
                .any(|k| matches!(k, crate::card::Keyword::Impending(_)));
            if is_impending && card.counter_count(crate::card::CounterType::Time) > 0 {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L4Type,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::RemoveCardType(CardType::Creature),
                });
            }
        }
        // Alpine Moon — opponents' lands matching the source's chosen name
        // lose all land types and abilities (the any-color mana grant rides
        // a separate `GrantActivatedAbility` over `NamedBySource`).
        for card in &self.battlefield {
            let has = card.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, crate::effect::StaticEffect::NamedLandsNeutralized)
            });
            let Some(name) = card.named_card.as_deref().filter(|_| has) else { continue };
            let hit: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.definition.is_land()
                        && c.definition.name == name
                        && !self.same_team(c.controller, card.controller)
                })
                .map(|c| c.id)
                .collect();
            if hit.is_empty() {
                continue;
            }
            for (layer, modification) in [
                (Layer::L4Type, Modification::SetLandTypes(vec![])),
                (Layer::L6Ability, Modification::RemoveAllAbilities),
            ] {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(hit.clone()),
                    layer,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification,
                });
            }
        }
        // Ultima — lands with a blight counter lose all land types and
        // abilities (the "{T}: Add {C}" half rides a `GrantActivatedAbility`
        // over `WithCounter(Blight)` lands).
        for card in &self.battlefield {
            let has = card.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, crate::effect::StaticEffect::BlightedLandsNeutralized)
            });
            if !has {
                continue;
            }
            let hit: Vec<CardId> = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.definition.is_land()
                        && c.counter_count(crate::card::CounterType::Blight) > 0
                })
                .map(|c| c.id)
                .collect();
            if hit.is_empty() {
                continue;
            }
            for (layer, modification) in [
                (Layer::L4Type, Modification::SetLandTypes(vec![])),
                (Layer::L6Ability, Modification::RemoveAllAbilities),
            ] {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(hit.clone()),
                    layer,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification,
                });
            }
        }
        // "This creature gets +X/+Y for each [filter] you control."
        // (`StaticEffect::PumpSelfByControlledPermanents`) — count the
        // controller's matching battlefield permanents live and emit a
        // layer-7 ModifyPowerToughness self-effect.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpSelfByControlledPermanents {
                    filter,
                    per_power,
                    per_toughness,
                } = &sa.effect
                else {
                    continue;
                };
                // Source-aware evaluation so an `OtherThanSource` filter
                // ("each *other* Rat you control" — Persistent Marshstalker)
                // excludes this permanent itself.
                let count = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == card.controller
                            && self.evaluate_requirement_static(
                                filter,
                                &crate::game::types::Target::Permanent(c.id),
                                card.controller,
                                Some(card.id),
                            )
                    })
                    .count() as i32;
                if count == 0 {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(
                        count * per_power,
                        count * per_toughness,
                    ),
                });
            }
        }
        // "[applies_to] you control get +P/+T for each [count_filter] you
        // control" (`StaticEffect::PumpTeamByControlledPermanents`) — count the
        // controller's matching permanents (plus graveyard cards when
        // `count_graveyard`), then emit a per-affected layer-7 pump. Warrior of
        // Light (legendary anthem) and Cid, Timeless Artificer (graveyard-aware
        // Artificer count) ride this.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpTeamByControlledPermanents {
                    applies_to,
                    count_filter,
                    per_power,
                    per_toughness,
                    count_graveyard,
                } = &sa.effect
                else {
                    continue;
                };
                let mut count = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        c.controller == card.controller
                            && self.evaluate_requirement_static(
                                count_filter,
                                &crate::game::types::Target::Permanent(c.id),
                                card.controller,
                                Some(card.id),
                            )
                    })
                    .count() as i32;
                if *count_graveyard {
                    count += self.players[card.controller]
                        .graveyard
                        .iter()
                        .filter(|c| {
                            self.evaluate_requirement_on_card(count_filter, c, card.controller)
                        })
                        .count() as i32;
                }
                if count == 0 {
                    continue;
                }
                for target in &self.battlefield {
                    if target.controller != card.controller
                        || !self.evaluate_requirement_static(
                            applies_to,
                            &crate::game::types::Target::Permanent(target.id),
                            card.controller,
                            Some(card.id),
                        )
                    {
                        continue;
                    }
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Specific(vec![target.id]),
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(
                            count * per_power,
                            count * per_toughness,
                        ),
                    });
                }
            }
        }
        // "As long as [condition], this creature gets +P/+T and has [keyword]."
        // (`StaticEffect::PumpSelfIf`) — evaluate the gating predicate live
        // against the source and, while it holds, emit a layer-7 pump plus an
        // optional keyword grant.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpSelfIf {
                    condition,
                    power,
                    toughness,
                    keywords,
                } = &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::Modify),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::ModifyPowerToughness(*power, *toughness),
                });
                for kw in keywords {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
            }
        }
        // "As long as [condition], this creature has base power and toughness
        // P/T." (`StaticEffect::SetBasePtIf`) — a live layer-7b set (Snowmelt
        // Stag). +N/+M and counters still stack on top per CR 613.7c/f.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SetBasePtIf { condition, power, toughness } =
                    &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L7PowerTough,
                    sublayer: Some(PtSublayer::SetValue),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::SetPowerToughness(*power, *toughness),
                });
            }
        }
        // "All [filter] have 'This gets +P/+T as long as [condition]'"
        // (`StaticEffect::GrantPumpSelfIf`) — Sedge Sliver. The condition is
        // evaluated per matching permanent with that permanent's controller
        // as "you".
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::GrantPumpSelfIf {
                    filter,
                    condition,
                    power,
                    toughness,
                    keywords,
                } = &sa.effect
                else {
                    continue;
                };
                for subject in &self.battlefield {
                    if !crate::game::layers::requirement_matches_card(
                        filter,
                        subject,
                        card.controller,
                    ) {
                        continue;
                    }
                    let ctx = crate::game::effects::EffectContext::for_ability(
                        subject.id,
                        subject.controller,
                        None,
                    );
                    if !self.evaluate_predicate(condition, &ctx) {
                        continue;
                    }
                    if *power != 0 || *toughness != 0 {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: AffectedPermanents::Specific(vec![subject.id]),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::Modify),
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::ModifyPowerToughness(*power, *toughness),
                        });
                    }
                    for kw in keywords {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: AffectedPermanents::Specific(vec![subject.id]),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::AddKeyword(kw.clone()),
                        });
                    }
                }
            }
        }
        // "As long as [condition], [creatures the selector picks] get +P/+T."
        // (`StaticEffect::PumpTeamIf`) — the conditional team anthem. Evaluate
        // the gate against the source; while it holds, emit a layer-7 pump for
        // every permanent the selector resolves to (e.g. Beastmaster Ascension
        // at 7+ quest counters → all your creatures +5/+5).
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::PumpTeamIf {
                    condition,
                    applies_to,
                    power,
                    toughness,
                    keywords,
                } = &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                if let Some(affected) = selector_to_affected(applies_to, card) {
                    if *power != 0 || *toughness != 0 {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: affected.clone(),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::Modify),
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::ModifyPowerToughness(*power, *toughness),
                        });
                    }
                    for kw in keywords {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: affected.clone(),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::AddKeyword(kw.clone()),
                        });
                    }
                }
            }
        }
        // Chosen-type tribal anthem (`StaticEffect::AnthemForChosenType`) —
        // pumps the controller's creatures of the type named at the source's
        // ETB (`CardInstance.chosen_creature_type`). Adaptive Automaton,
        // Patchwork Banner.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::AnthemForChosenType { power, toughness, exclude_source, opponents, per_counter } =
                    &sa.effect
                else {
                    continue;
                };
                let Some(ct) = card.chosen_creature_type else { continue };
                // Per-counter scaling (Door of Destinies): +P/+T for each
                // counter of `per_counter` on the source. No counters → no pump.
                let (power, toughness) = match per_counter {
                    Some(kind) => {
                        let n = card.counters.get(kind).copied().unwrap_or(0) as i32;
                        if n == 0 { continue }
                        (power * n, toughness * n)
                    }
                    None => (*power, *toughness),
                };
                // Whose creatures the modifier hits: the controller's (the
                // tribal-anthem default) or each opponent's (Plague Engineer).
                let seats: Vec<usize> = if *opponents {
                    self.opponents_of(card.controller)
                } else {
                    vec![card.controller]
                };
                for seat in seats {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::AllWithCreatureType {
                            controller: Some(seat),
                            creature_type: ct,
                            exclude_source: *exclude_source,
                        },
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(power, toughness),
                    });
                }
            }
        }
        // Chosen-type keyword grant (`StaticEffect::GrantKeywordToChosenType`) —
        // grants a keyword to the controller's (or each opponent's) creatures of
        // the type named at the source's ETB. Steely Resolve, Kindred Boon.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::GrantKeywordToChosenType { keyword, opponents } =
                    &sa.effect
                else {
                    continue;
                };
                let Some(ct) = card.chosen_creature_type else { continue };
                let seats: Vec<usize> = if *opponents {
                    self.opponents_of(card.controller)
                } else {
                    vec![card.controller]
                };
                for seat in seats {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::AllWithCreatureType {
                            controller: Some(seat),
                            creature_type: ct,
                            exclude_source: false,
                        },
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    });
                }
            }
        }
        // Fixed-filter team anthem (`StaticEffect::AnthemForFilter`) — pumps
        // and/or grants keywords to the controller's (or each opponent's)
        // permanents matching a printed filter. Balthier and Fran (Vehicles),
        // Ardyn, the Usurper (Demons).
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::AnthemForFilter {
                    filter, power, toughness, keywords, opponents, only_your_turn,
                    scale_by_counters_on_self,
                } = &sa.effect
                else {
                    continue;
                };
                // "During your turn" anthems switch off outside the controller's
                // turn (CR 611.2c live re-evaluation).
                if *only_your_turn && self.active_player_idx != card.controller {
                    continue;
                }
                // Chitterspitter — "+P/+T for each [kind] counter on this".
                let scale = match scale_by_counters_on_self {
                    Some(kind) => card.counter_count(*kind) as i32,
                    None => 1,
                };
                let (power, toughness) = (&(power * scale), &(toughness * scale));
                let seats: Vec<usize> = if *opponents {
                    self.opponents_of(card.controller)
                } else {
                    vec![card.controller]
                };
                for seat in seats {
                    // "[filter] you control" for `seat`. A printed-characteristics
                    // filter routes through the dynamic (GameState-blind) CardMatch
                    // path; a stateful filter (e.g. `IsEnchanted`, which must scan
                    // the battlefield for attached Auras — A Tale for the Ages) is
                    // resolved here against live state and pinned to those ids.
                    let affected = if crate::game::layers::requirement_is_card_only(filter) {
                        let req = crate::card::SelectionRequirement::And(
                            Box::new(filter.clone()),
                            Box::new(crate::card::SelectionRequirement::ControlledByYou),
                        );
                        AffectedPermanents::CardMatch {
                            source_controller: seat,
                            requirement: Box::new(req),
                        }
                    } else {
                        let ids: Vec<CardId> = self
                            .battlefield
                            .iter()
                            .filter(|c| c.controller == seat)
                            .filter(|c| {
                                self.evaluate_requirement_static(
                                    filter,
                                    &crate::game::types::Target::Permanent(c.id),
                                    seat,
                                    Some(card.id),
                                )
                            })
                            .map(|c| c.id)
                            .collect();
                        AffectedPermanents::Specific(ids)
                    };
                    if *power != 0 || *toughness != 0 {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: affected.clone(),
                            layer: Layer::L7PowerTough,
                            sublayer: Some(PtSublayer::Modify),
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::ModifyPowerToughness(*power, *toughness),
                        });
                    }
                    for kw in keywords {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: affected.clone(),
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::AddKeyword(kw.clone()),
                        });
                    }
                }
            }
        }
        // State-aware `SetBasePtForFilter` / `GrantKeyword` — when the
        // `applies_to` selector carries a *stateful* filter (e.g. `IsEnchanted`,
        // which must scan the battlefield for attached Auras — Archon of the
        // Wild Rose's "other creatures you control enchanted by Auras … have
        // base P/T 4/4 and flying"), `selector_to_affected` can't decompose it
        // and the per-static gather emits nothing. Resolve those live here and
        // pin the effect to the matching ids, mirroring the `AnthemForFilter`
        // stateful path above.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let (req, modification, layer, sublayer) = match &sa.effect {
                    crate::effect::StaticEffect::SetBasePtForFilter {
                        applies_to: crate::effect::Selector::EachPermanent(req),
                        power,
                        toughness,
                    } if !crate::game::layers::requirement_is_card_only(req) => (
                        req,
                        Modification::SetPowerToughness(*power, *toughness),
                        Layer::L7PowerTough,
                        Some(PtSublayer::SetValue),
                    ),
                    crate::effect::StaticEffect::GrantKeyword {
                        applies_to: crate::effect::Selector::EachPermanent(req),
                        keyword,
                    } if !crate::game::layers::requirement_is_card_only(req) => (
                        req,
                        Modification::AddKeyword(keyword.clone()),
                        Layer::L6Ability,
                        None,
                    ),
                    _ => continue,
                };
                let ids: Vec<CardId> = self
                    .battlefield
                    .iter()
                    .filter(|c| {
                        self.evaluate_requirement_static(
                            req,
                            &crate::game::types::Target::Permanent(c.id),
                            card.controller,
                            Some(card.id),
                        )
                    })
                    .map(|c| c.id)
                    .collect();
                if ids.is_empty() {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Specific(ids),
                    layer,
                    sublayer,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification,
                });
            }
        }
        // Predicate-gated self keyword (`StaticEffect::SelfHasKeywordIf`) —
        // Freya Crescent's "During your turn, Freya has flying".
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SelfHasKeywordIf { keyword, condition } =
                    &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(keyword.clone()),
                });
            }
        }
        // Predicate-gated self is-a-creature (`StaticEffect::SelfIsCreatureIf`) —
        // Midnight Mangler is an artifact creature during turns other than yours.
        for card in &self.battlefield {
            for sa in &card.definition.static_abilities {
                let crate::effect::StaticEffect::SelfIsCreatureIf { condition } = &sa.effect
                else {
                    continue;
                };
                let ctx = crate::game::effects::EffectContext::for_ability(
                    card.id,
                    card.controller,
                    None,
                );
                if !self.evaluate_predicate(condition, &ctx) {
                    continue;
                }
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L4Type,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddCardType(CardType::Creature),
                });
            }
        }
        // CR 604.3 — characteristic-defining dynamic P/T injection. The
        // formula lives on `CardDefinition.dynamic_pt`; we resolve it here
        // on every layer recompute and emit a layer-7 SetPT effect.
        let goyf_n = self.distinct_card_types_in_all_graveyards() as i32;
        let lands_in_gys: i32 = self.players.iter()
            .map(|p| p.graveyard.iter().filter(|c| c.definition.is_land()).count() as i32)
            .sum();
        let creatures_in_gys: i32 = self.players.iter()
            .map(|p| p.graveyard.iter().filter(|c| c.definition.is_creature()).count() as i32)
            .sum();
        for card in &self.battlefield {
            let Some(formula) = card.definition.dynamic_pt.clone() else { continue };
            let (power, toughness) = match formula {
                crate::card::DynamicPt::DistinctTypesInAllGraveyards => {
                    (goyf_n, goyf_n + 1)
                }
                crate::card::DynamicPt::DevotionTo { color, base_t } => {
                    let mut ctx = crate::game::effects::EffectContext::for_spell(
                        card.controller, None, 0, 0,
                    );
                    ctx.source = Some(card.id);
                    let n = self.evaluate_value(
                        &crate::effect::Value::DevotionTo(vec![color]),
                        &ctx,
                    );
                    (n, base_t)
                }
                crate::card::DynamicPt::DevotionToToughness { color, base_p } => {
                    let mut ctx = crate::game::effects::EffectContext::for_spell(
                        card.controller, None, 0, 0,
                    );
                    ctx.source = Some(card.id);
                    let n = self.evaluate_value(
                        &crate::effect::Value::DevotionTo(vec![color]),
                        &ctx,
                    );
                    (base_p, n)
                }
                crate::card::DynamicPt::ControllerGraveyardSize => {
                    let n = self.players[card.controller].graveyard.len() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::BasePlusUnspentColorMana { base_p, base_t, color } => {
                    let n = self.players[card.controller].mana_pool.amount(color) as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::BasePlusCreaturesInControllerGraveyard { base } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_creature()).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::PermanentCardsInControllerGraveyard { base_p, base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_permanent()).count() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::CardsYouOwnInExile { base_t } => {
                    let n = self.exile.iter()
                        .filter(|c| c.owner == card.controller).count() as i32;
                    (n, n + base_t)
                }
                crate::card::DynamicPt::CreaturesYouControlWithTypes { types } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.is_creature()
                            && c.definition.subtypes.creature_types.iter().any(|t| types.contains(t))
                    }).count() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::CreaturesYouControl { base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_creature()
                    }).count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::BasePlusOtherFlyersControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.id != card.id
                            && c.controller == card.controller
                            && c.definition.is_creature()
                            && c.definition.keywords.contains(&crate::card::Keyword::Flying)
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::BasePlusOpponentGraveyards { base, creatures_only } => {
                    let n: i32 = self
                        .opponents_of(card.controller)
                        .iter()
                        .map(|&o| {
                            self.players[o]
                                .graveyard
                                .iter()
                                .filter(|c| !creatures_only || c.definition.is_creature())
                                .count() as i32
                        })
                        .sum();
                    (base + n, base + n)
                }
                crate::card::DynamicPt::BasePlusLandsInAllGraveyards { base_p, base_t } => {
                    (base_p + lands_in_gys, base_t + lands_in_gys)
                }
                crate::card::DynamicPt::CreatureCardsInAllGraveyards { base_p, base_t } => {
                    (base_p + creatures_in_gys, base_t + creatures_in_gys)
                }
                crate::card::DynamicPt::CreatureCardsInAllGraveyardsPower { base_t } => {
                    (creatures_in_gys, base_t)
                }
                crate::card::DynamicPt::BasePlusLandsInControllerGraveyard { base_p, base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_land()).count() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::BaseMinusControllerLife { base_p, base_t } => {
                    let life = self.players[card.controller].life;
                    (base_p - life, base_t - life)
                }
                crate::card::DynamicPt::ColorlessCreaturesControlled { base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.is_creature()
                            && is_colorless_by_cost(&c.definition)
                    }).count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::CreaturesControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_creature()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::CreaturesControlledPower { base_p, base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_creature()
                    }).count() as i32;
                    (base_p + n, base_t)
                }
                crate::card::DynamicPt::PlusCountersOnLandsControlledPower { base_p, base_t } => {
                    let n: i32 = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_land()
                    }).map(|c| c.counter_count(crate::card::CounterType::PlusOnePlusOne) as i32).sum();
                    (base_p + n, base_t)
                }
                crate::card::DynamicPt::ControllerExperience { base_p, base_t } => {
                    let n = self.players[card.controller].experience as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::CreaturesOfTypeControlled { creature_type } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.is_creature()
                            && (c.definition.subtypes.creature_types.contains(&creature_type)
                                || c.has_keyword(&crate::card::Keyword::Changeling))
                    }).count() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::LandsControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_land()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::LandsControlledPower { base_p, base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_land()
                    }).count() as i32;
                    (base_p + n, base_t)
                }
                crate::card::DynamicPt::LandsControlledPlusLandsInControllerGraveyard { base } => {
                    let bf = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_land()
                    }).count() as i32;
                    let gy = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_land()).count() as i32;
                    (base + bf + gy, base + bf + gy)
                }
                crate::card::DynamicPt::CardTypesInOpponentsGraveyards { base_p, base_t } => {
                    let mut seen: std::collections::HashSet<crate::card::CardType> =
                        std::collections::HashSet::new();
                    for (i, player) in self.players.iter().enumerate() {
                        if i == card.controller { continue; }
                        for c in &player.graveyard {
                            for ct in &c.definition.card_types { seen.insert(ct.clone()); }
                        }
                    }
                    (base_p + seen.len() as i32, base_t)
                }
                crate::card::DynamicPt::CardTypesInControllerGraveyard { base_p, base_t } => {
                    let mut seen: std::collections::HashSet<crate::card::CardType> =
                        std::collections::HashSet::new();
                    for c in &self.players[card.controller].graveyard {
                        for ct in &c.definition.card_types { seen.insert(ct.clone()); }
                    }
                    let n = seen.len() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::BasePlusLandsOfTypeControlled { land_type, base_p, base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.subtypes.land_types.contains(&land_type)
                    }).count() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::PowerPlusLandsOfTypeControlled { land_type, base_p, base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.subtypes.land_types.contains(&land_type)
                    }).count() as i32;
                    (base_p + n, base_t)
                }
                crate::card::DynamicPt::BasePlusGreatestOtherArtifactMv { base_p, base_t } => {
                    let greatest = self.battlefield.iter().filter(|c| {
                        c.id != card.id
                            && c.controller == card.controller
                            && c.definition.is_artifact()
                    }).map(|c| c.definition.cost.cmc() as i32).max().unwrap_or(0);
                    (base_p + greatest, base_t)
                }
                crate::card::DynamicPt::BasePlusCountersOnSelf { counter_type, base_p, base_t, per_p, per_t } => {
                    let n = card.counter_count(counter_type) as i32;
                    (base_p + n * per_p, base_t + n * per_t)
                }
                crate::card::DynamicPt::ControllerHandSize => {
                    let n = self.players[card.controller].hand.len() as i32;
                    (n, n)
                }
                crate::card::DynamicPt::ControllerHandSizeTimes { factor } => {
                    let n = self.players[card.controller].hand.len() as i32 * factor;
                    (n, n)
                }
                crate::card::DynamicPt::MaxOpponentHandSize => {
                    let n = self
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != card.controller)
                        .map(|(_, p)| p.hand.len())
                        .max()
                        .unwrap_or(0) as i32;
                    (n, n)
                }
                crate::card::DynamicPt::BaseMinusOpponentsHandTotal { base_p, base_t } => {
                    let n = self
                        .players
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != card.controller && !self.same_team(*i, card.controller))
                        .map(|(_, p)| p.hand.len() as i32)
                        .sum::<i32>();
                    (base_p - n, base_t - n)
                }
                crate::card::DynamicPt::ArtifactsControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_artifact()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::ArtifactsControlledPower { base_p, base_t } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && c.definition.is_artifact()
                    }).count() as i32;
                    (base_p + n, base_t)
                }
                crate::card::DynamicPt::EnchantmentsInPlay { base_p, base_t } => {
                    let n = self.battlefield.iter()
                        .filter(|c| c.definition.is_enchantment())
                        .count() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::NonlandPermanentsControlled { base } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.controller == card.controller && !c.definition.is_land()
                    }).count() as i32;
                    (base + n, base + n)
                }
                crate::card::DynamicPt::ForestsInPlay { base_p } => {
                    let n = self.battlefield.iter()
                        .filter(|c| c.definition.subtypes.land_types.contains(&crate::card::LandType::Forest))
                        .count() as i32;
                    (base_p, n)
                }
                crate::card::DynamicPt::InstantsSorceriesInGraveyardAndExile { base_t } => {
                    let gy = &self.players[card.controller].graveyard;
                    let is_is = |c: &CardInstance| c.definition.is_instant() || c.definition.is_sorcery();
                    let n = gy.iter().filter(|c| is_is(c)).count() as i32
                        + self.exile.iter()
                            .filter(|c| c.owner == card.controller && is_is(c))
                            .count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::InstantsSorceriesInControllerGraveyard { base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| c.definition.is_instant() || c.definition.is_sorcery())
                        .count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::CardsDrawnThisTurnPower { base_t } => {
                    (self.players[card.controller].cards_drawn_this_turn as i32, base_t)
                }
                crate::card::DynamicPt::NoncreatureNonlandCardsInControllerGraveyard { base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| !c.definition.is_creature() && !c.definition.is_land())
                        .count() as i32;
                    (n, base_t)
                }
                crate::card::DynamicPt::BasePlusNoncreatureNonlandInControllerGraveyard { base_p, base_t } => {
                    let n = self.players[card.controller].graveyard.iter()
                        .filter(|c| !c.definition.is_creature() && !c.definition.is_land())
                        .count() as i32;
                    (base_p + n, base_t + n)
                }
                crate::card::DynamicPt::ColorsAmongAlliesControlledPower { base_p, base_t } => {
                    let mut colors: std::collections::HashSet<crate::mana::Color> =
                        std::collections::HashSet::new();
                    for c in self.battlefield.iter().filter(|c| {
                        c.controller == card.controller
                            && c.definition.is_creature()
                            && (c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Ally)
                                || c.has_keyword(&crate::card::Keyword::Changeling))
                    }) {
                        for col in c.definition.printed_colors() { colors.insert(col); }
                    }
                    (base_p + colors.len() as i32, base_t)
                }
                crate::card::DynamicPt::ExiledWithSourcePt { base_p, base_t } => self
                    .exile
                    .iter()
                    .find(|c| c.exiled_with == Some(card.id) && c.definition.is_creature())
                    .map(|c| (c.definition.base_power(), c.definition.base_toughness()))
                    .unwrap_or((base_p, base_t)),
                crate::card::DynamicPt::BasePlusPerAttachedAura { base_p, base_t, per } => {
                    let n = self.battlefield.iter().filter(|c| {
                        c.attached_to == Some(card.id) && c.definition.is_aura()
                    }).count() as i32;
                    (base_p + n * per, base_t + n * per)
                }
                crate::card::DynamicPt::BasePlusPerAttachedEquipment { base_p, base_t, per } => {
                    let n = self.attached_equipment_count(card.id) as i32;
                    (base_p + n * per, base_t + n * per)
                }
                crate::card::DynamicPt::BaseMinusHighestLife { base_p, base_t } => {
                    let hi = self.players.iter()
                        .filter(|p| !p.eliminated)
                        .map(|p| p.life)
                        .max()
                        .unwrap_or(0);
                    (base_p - hi, base_t - hi)
                }
            };
            all_effects.push(ContinuousEffect {
                timestamp: card.object_timestamp(),
                source: card.id,
                affected: AffectedPermanents::Source,
                layer: Layer::L7PowerTough,
                sublayer: Some(PtSublayer::CharDefining),
                duration: EffectDuration::WhileSourceOnBattlefield,
                modification: Modification::SetPowerToughness(power, toughness),
            });
        }
        // Ulamog, the Defiler — annihilator X where X = +1/+1 counters,
        // injected as a computed layer-6 keyword.
        for card in &self.battlefield {
            let has = card.definition.static_abilities.iter().any(|sa| {
                matches!(sa.effect, crate::effect::StaticEffect::AnnihilatorPerPlusOneCounter)
            });
            if !has {
                continue;
            }
            let n = card.counter_count(crate::card::CounterType::PlusOnePlusOne);
            if n > 0 {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(crate::card::Keyword::Annihilator(n)),
                });
            }
        }
        // CR 702.87 — Level up: the band matching the creature's level-counter
        // count sets its base P/T (layer 7a CDA) and grants its keywords.
        for card in &self.battlefield {
            if card.definition.level_bands.is_empty() {
                continue;
            }
            let lvl = card.counter_count(crate::card::CounterType::Level);
            let Some(band) = card
                .definition
                .level_bands
                .iter()
                .find(|b| lvl >= b.min && b.max.is_none_or(|m| lvl <= m))
            else {
                continue;
            };
            all_effects.push(ContinuousEffect {
                timestamp: card.object_timestamp(),
                source: card.id,
                affected: AffectedPermanents::Source,
                layer: Layer::L7PowerTough,
                sublayer: Some(PtSublayer::CharDefining),
                duration: EffectDuration::WhileSourceOnBattlefield,
                modification: Modification::SetPowerToughness(band.power, band.toughness),
            });
            for kw in &band.keywords {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(kw.clone()),
                });
            }
        }
        // CR 721.2 — Station symbols. Every band whose `{N+}` threshold is met
        // by the permanent's charge-counter count grants its abilities (layer
        // 6); a band with a `[P/T]` box also makes it a creature with that base
        // P/T (CR 721.2b — layers 4 + 7a).
        for card in &self.battlefield {
            if card.definition.station.is_empty() {
                continue;
            }
            let charges = card.counter_count(crate::card::CounterType::Charge);
            for band in card.definition.station.iter().filter(|b| charges >= b.min) {
                for kw in &band.keywords {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw.clone()),
                    });
                }
                if let Some((power, toughness)) = band.pt {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddCardType(crate::card::CardType::Creature),
                    });
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::CharDefining),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::SetPowerToughness(power, toughness),
                    });
                }
                // CR 721.2a — static abilities granted by the band.
                for se in &band.statics {
                    all_effects.extend(static_effect_to_effects(se, card, card.object_timestamp()));
                }
            }
        }
        for card in &self.battlefield {
            // CR 702.98 — Unleash's second static: a creature with the
            // Unleash keyword can't block while it has a +1/+1 counter.
            // Injected as a computed `CantBlock` so the existing block-
            // legality enforcement (`declare_blockers`) honors it.
            if card.definition.keywords.contains(&Keyword::Unleash)
                && card.counters.get(&crate::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0) > 0
            {
                all_effects.push(ContinuousEffect {
                    timestamp: card.object_timestamp(),
                    source: card.id,
                    affected: AffectedPermanents::Source,
                    layer: Layer::L6Ability,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddKeyword(Keyword::CantBlock),
                });
            }
            // CR 701.60 — a suspected creature has menace and can't block.
            // Injected as computed keywords so combat-legality enforcement
            // honors them.
            if card.suspected && card.definition.is_creature() {
                for kw in [Keyword::Menace, Keyword::CantBlock] {
                    all_effects.push(ContinuousEffect {
                        timestamp: card.object_timestamp(),
                        source: card.id,
                        affected: AffectedPermanents::Source,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(kw),
                    });
                }
            }
        }
        // Graveyard-resident static-ability injection — the Incarnation
        // cycle's `StaticEffect::GraveyardAnthem` ("As long as this card is
        // in your graveyard and you control a [Land subtype], creatures you
        // control have [keyword]"). Zone-special: read off graveyard cards'
        // printed statics, gated on the owner controlling a land of the
        // required subtype. The effect's `source` is the gy card's id, so
        // removing the gy card causes the effect to fall out.
        for player in &self.players {
            for card in &player.graveyard {
                for sa in &card.definition.static_abilities {
                    let crate::effect::StaticEffect::GraveyardAnthem { land_type, keyword } =
                        &sa.effect
                    else {
                        continue;
                    };
                    let (land_subtype, kw) = (*land_type, keyword.clone());
                    let controller_has_land = self.battlefield.iter().any(|c| {
                        c.controller == card.owner
                            && c.definition.subtypes.land_types.iter().any(|lt| lt == &land_subtype)
                    });
                    if controller_has_land {
                        all_effects.push(ContinuousEffect {
                            timestamp: card.object_timestamp(),
                            source: card.id,
                            affected: AffectedPermanents::All {
                                controller: Some(card.owner),
                                card_types: vec![CardType::Creature],
                                exclude_source: false,
                                color: None,
                                token: None,
                                colorless: false,
                            },
                            layer: Layer::L6Ability,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification: Modification::AddKeyword(kw),
                        });
                    }
                }
            }
        }
        // CR 701.54c — the Ring's level-1 emblem makes its controller's
        // Ring-bearer legendary (in addition to the can't-be-blocked rider,
        // which is enforced directly in `blocker_can_block_attacker`).
        for seat in 0..self.players.len() {
            if self.players[seat].ring_temptations >= 1
                && let Some(bearer) = self.effective_ring_bearer(seat)
            {
                all_effects.push(ContinuousEffect {
                    timestamp: 0,
                    source: bearer,
                    affected: AffectedPermanents::Specific(vec![bearer]),
                    layer: Layer::L4Type,
                    sublayer: None,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    modification: Modification::AddSupertype(crate::card::Supertype::Legendary),
                });
            }
        }
        all_effects
    }

    /// Count of distinct card types (Artifact, Creature, Enchantment,
    /// Instant, Land, Planeswalker, Sorcery, Battle, Tribal) across every
    /// player's graveyard. Used by Tarmogoyf-style dynamic P/T.
    pub fn distinct_card_types_in_all_graveyards(&self) -> usize {
        let mut seen: std::collections::HashSet<CardType> = std::collections::HashSet::new();
        for player in &self.players {
            for card in &player.graveyard {
                for ct in &card.definition.card_types {
                    seen.insert(ct.clone());
                }
            }
        }
        seen.len()
    }

    /// CR 700.4-ish — Delirium: `seat`'s graveyard holds four or more card
    /// types. Shared by the Delirium predicate and the combat-restriction
    /// keyword (Patchwork Beastie).
    pub fn delirium_active(&self, seat: usize) -> bool {
        self.distinct_card_types_in_graveyard(seat) >= 4
    }

    /// Distinct card types among cards in `seat`'s graveyard — the delirium
    /// count as a number (`Value::CardTypesInGraveyard`, Lucid Dreams).
    pub fn distinct_card_types_in_graveyard(&self, seat: usize) -> usize {
        let mut kinds: std::collections::HashSet<&CardType> = std::collections::HashSet::new();
        for c in &self.players[seat].graveyard {
            for t in &c.definition.card_types {
                kinds.insert(t);
            }
        }
        kinds.len()
    }

    /// CR 700.11 — the number of permanent cards in `seat`'s graveyard
    /// ("descend" count, the threshold for Descend N gates).
    pub fn descend_count(&self, seat: usize) -> usize {
        self.players[seat]
            .graveyard
            .iter()
            .filter(|c| c.definition.is_permanent())
            .count()
    }

    /// Get the computed state of a single permanent (or None if not on battlefield).
    ///
    /// Gathers the same continuous-effect set as `compute_battlefield` but
    /// applies the layer pass to only the one target card, instead of
    /// building a `ComputedPermanent` for every permanent and discarding
    /// all but one.
    pub fn computed_permanent(&self, id: CardId) -> Option<ComputedPermanent> {
        let card = self.battlefield.iter().find(|c| c.id == id)?;
        if let Some(fx) = self.frozen_effects() {
            return Some(crate::game::layers::apply_layers_one(card, &fx));
        }
        Some(crate::game::layers::apply_layers_one(
            card,
            &self.gather_continuous_effects(),
        ))
    }

    /// CR 603.10 — a last-known-information snapshot of a battlefield permanent
    /// that is about to leave (die/sacrifice). Clones the live `CardInstance`
    /// but stamps its *computed* creature types (layer 4) onto the definition
    /// so a "whenever a [type] you control dies" trigger reads types the
    /// creature gained from a continuous effect (Jenova's granted Mutant), not
    /// just its printed subtypes. Only pays the layer cost when a grant is
    /// actually present.
    /// CR 603.10 — the leaves-battlefield LKI snapshot for `cid` if it's the
    /// object currently being read as LKI (the resolving trigger's source or
    /// its dead subject) and it's no longer on the battlefield. Backs
    /// `Value::PowerOf`/`ToughnessOf` reads of a just-died creature.
    pub(crate) fn lki_snapshot(&self, cid: CardId) -> Option<&CardInstance> {
        if self.battlefield_find(cid).is_some() {
            return None;
        }
        if self.resolving_lki_source == Some(cid) || self.resolving_lki_subject == Some(cid) {
            self.leaves_bf_lki.get(&cid)
        } else {
            None
        }
    }

    pub(crate) fn dying_snapshot(&self, id: CardId) -> Option<CardInstance> {
        let mut snap = self.battlefield.iter().find(|c| c.id == id)?.clone();
        if let Some(cp) = self.computed_permanent(id) {
            let printed = &snap.definition.subtypes.creature_types;
            if cp.subtypes.creature_types.iter().any(|t| !printed.contains(t)) {
                std::sync::Arc::make_mut(&mut snap.definition).subtypes.creature_types =
                    cp.subtypes.creature_types.clone();
            }
        }
        Some(snap)
    }

    /// CR 702.16 — true if `target` has protection from any of `source`'s
    /// (computed) colors. Reads both sides through the layer system so granted
    /// protection / color-setting effects count. Backs damage prevention
    /// (702.16e) and equip/attach legality (702.16f).
    pub(crate) fn is_protected_from(&self, source: CardId, target: CardId) -> bool {
        self.damage_prevented_by_protection(source, target)
    }

    /// CR 702.16e — damage from a source is prevented if the target permanent
    /// has protection from any of that source's (computed) colors. Reads both
    /// sides through the layer system so granted protection / color-setting
    /// effects count.
    /// CR 106.4 override — empty every player's mana pool, except that a
    /// player with an `UnspentManaBecomesColorless` static (Kruphix) keeps
    /// the total as colorless mana.
    pub(crate) fn empty_mana_pools(&mut self) {
        use crate::effect::StaticEffect;
        let keepers: Vec<usize> = self
            .battlefield
            .iter()
            .filter(|c| {
                c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::UnspentManaBecomesColorless)
                })
            })
            .map(|c| c.controller)
            .collect();
        // CR 500.4 exception — Upwelling: no player loses unspent mana at all.
        let all_persist = self.battlefield.iter().any(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::ManaPoolsNeverEmpty))
        });
        // CR 106.4 exception — per-player kept colors (Omnath keeps green).
        let color_keepers: Vec<(usize, crate::mana::Color)> = self
            .battlefield
            .iter()
            .flat_map(|c| {
                c.definition.static_abilities.iter().filter_map(move |sa| {
                    if let StaticEffect::UnspentColorManaPersists(col) = sa.effect {
                        Some((c.controller, col))
                    } else {
                        None
                    }
                })
            })
            .collect();
        // CR 702.189a — Firebending mana survives until end of combat; the
        // end-of-combat-step empty is where it finally clears (no re-seed).
        let end_of_combat = self.step == crate::game::types::TurnStep::EndCombat;
        for (i, player) in self.players.iter_mut().enumerate() {
            if all_persist {
                // Pool survives intact; still handle firebending below.
            } else if keepers.contains(&i) {
                let total = player.mana_pool.total();
                player.mana_pool.empty();
                player.mana_pool.add_colorless(total);
            } else {
                // Preserve the amounts of any colors this player keeps.
                let kept: Vec<(crate::mana::Color, u32)> = color_keepers
                    .iter()
                    .filter(|(p, _)| *p == i)
                    .map(|(_, col)| (*col, player.mana_pool.amount(*col)))
                    .collect();
                player.mana_pool.empty();
                for (col, amt) in kept {
                    player.mana_pool.add(col, amt);
                }
            }
            if player.firebending_kept_red > 0 {
                if end_of_combat {
                    player.firebending_kept_red = 0;
                } else {
                    player.mana_pool.add(crate::mana::Color::Red, player.firebending_kept_red);
                }
            }
        }
    }

    /// CR 615 — true if `target` is an attacking creature whose controller
    /// has a `PreventDamageToYourAttackers` static in play (Iroas, God of
    /// Victory). Overridden by "damage can't be prevented" (CR 615.12).
    pub(crate) fn damage_to_attacker_prevented(&self, target: CardId) -> bool {
        if self.damage_cant_be_prevented_this_turn {
            return false;
        }
        if !self.attacking.iter().any(|a| a.attacker == target) {
            return false;
        }
        let Some(controller) = self.battlefield_find(target).map(|c| c.controller) else {
            return false;
        };
        self.battlefield.iter().any(|c| {
            c.controller == controller
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::PreventDamageToYourAttackers)
                })
        })
    }

    /// CR 615 — true if `player` controls a permanent with a blanket
    /// "prevent all damage that would be dealt to you" static (Glacial Chasm),
    /// unless prevention is shut off this turn (615.12).
    pub(crate) fn all_damage_to_player_prevented(&self, player: usize) -> bool {
        if self.damage_cant_be_prevented_this_turn {
            return false;
        }
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::PreventAllDamageToController)
                })
        })
    }

    /// CR 615 — true if `target` is a creature whose controller has a
    /// "prevent all noncombat damage to creatures you control" static (Mark of
    /// Asylum). Consulted only at the noncombat damage funnel.
    pub(crate) fn noncombat_damage_to_creature_prevented(&self, target: CardId) -> bool {
        if self.damage_cant_be_prevented_this_turn {
            return false;
        }
        let Some(tgt) = self.battlefield_find(target) else { return false };
        if !tgt.definition.is_creature() {
            return false;
        }
        let controller = tgt.controller;
        self.battlefield.iter().any(|c| {
            c.controller == controller
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::PreventNoncombatDamageToYourCreatures)
                })
        })
    }

    pub(crate) fn damage_prevented_by_protection(&self, source: CardId, target: CardId) -> bool {
        // Both sides read through the layer system — share one gather.
        self.with_frozen_layers(|g| g.damage_prevented_by_protection_inner(source, target))
    }

    fn damage_prevented_by_protection_inner(&self, source: CardId, target: CardId) -> bool {
        let Some(tgt) = self.computed_permanent(target) else { return false };
        let src_colors = self
            .computed_permanent(source)
            .map(|c| c.colors)
            .unwrap_or_else(|| {
                self.battlefield_find(source)
                    .map(|c| c.definition.cost.colors())
                    .unwrap_or_default()
            });
        // CR 702.16 — protection from creatures prevents all damage from a
        // creature source (Spirit Mantle).
        let src_is_creature = self
            .computed_permanent(source)
            .map(|c| c.card_types.contains(&crate::card::CardType::Creature))
            .unwrap_or_else(|| {
                self.battlefield_find(source)
                    .map(|c| c.definition.is_creature())
                    .unwrap_or(false)
            });
        if src_is_creature && tgt.keywords.contains(&Keyword::ProtectionFromCreatures) {
            return true;
        }
        // CR 702.16e — protection from a creature type prevents damage from a
        // source of that type.
        let src_creature_types = self
            .computed_permanent(source)
            .map(|c| c.subtypes.creature_types)
            .unwrap_or_else(|| {
                self.battlefield_find(source)
                    .map(|c| c.definition.subtypes.creature_types.clone())
                    .unwrap_or_default()
            });
        let src_mv = self
            .battlefield_find(source)
            .map(|c| c.definition.cost.cmc())
            .unwrap_or(0);
        let src_card_types = self
            .computed_permanent(source)
            .map(|c| c.card_types)
            .unwrap_or_else(|| {
                self.battlefield_find(source)
                    .map(|c| c.definition.card_types.clone())
                    .unwrap_or_default()
            });
        tgt.keywords.iter().any(|kw| match kw {
            Keyword::Protection(color) => src_colors.contains(color),
            Keyword::ProtectionFromCreatureType(ty) => src_creature_types.contains(ty),
            Keyword::ProtectionFromManaValueExcept(n) => src_mv != *n,
            Keyword::ProtectionFromManaValueParity { odd } => (src_mv % 2 == 1) == *odd,
            Keyword::ProtectionFromMulticolored => src_colors.len() >= 2,
            Keyword::ProtectionFromCardType(t) => src_card_types.contains(t),
            Keyword::ProtectionFromEverything => true,
            _ => false,
        })
    }

    /// CR 702.89 — Umbra armor: if the creature would be destroyed, instead
    /// remove all damage from it and destroy one umbra-armor Aura attached
    /// to it. Returns true when the destruction was replaced.
    pub(crate) fn apply_umbra_armor(
        &mut self,
        id: CardId,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let Some(aura_id) = self.battlefield.iter().find_map(|c| {
            (c.attached_to == Some(id)
                && c.definition.keywords.contains(&Keyword::UmbraArmor))
            .then_some(c.id)
        }) else {
            return false;
        };
        if let Some(c) = self.battlefield_find_mut(id) {
            c.damage = 0;
        }
        let mut evs = self.remove_to_graveyard_with_triggers(aura_id);
        events.append(&mut evs);
        true
    }

    /// Add a transient continuous effect (from a spell/ability resolution).
    pub fn add_continuous_effect(&mut self, effect: ContinuousEffect) {
        self.continuous_effects.push(effect);
    }

    /// Allocate a new monotonically-increasing timestamp.
    /// Grant `kw` to a battlefield permanent until end of turn, stamping
    /// the grant's layer timestamp (CR 613.7) so it orders correctly
    /// against RemoveKeyword / RemoveAllAbilities effects. Always records —
    /// re-granting a keyword the permanent already carries matters when an
    /// ability-loss effect sits between the two timestamps (Snakeform, then
    /// Jump: the later grant must survive). The layer walk dedups.
    pub(crate) fn grant_keyword_eot(&mut self, cid: CardId, kw: crate::card::Keyword) {
        let ts = self.next_timestamp();
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
            c.granted_keywords_eot.push(kw);
            c.granted_keywords_eot_ts.push(ts);
        }
    }

    pub(crate) fn next_timestamp(&mut self) -> u64 {
        let ts = self.next_effect_timestamp;
        self.next_effect_timestamp += 1;
        ts
    }

    /// Remove all continuous effects whose source is `id` (source left battlefield).
    pub(crate) fn remove_effects_from_source(&mut self, id: CardId) {
        self.continuous_effects.retain(|e| e.source != id);
    }

    /// Expire all `UntilEndOfTurn` continuous effects (called during Cleanup).
    /// Also sweeps `UntilEndOfCombat` for cards that registered combat-
    /// scoped effects during a turn that ended without an actual combat
    /// phase (defensive cleanup so they don't leak indefinitely).
    pub(crate) fn expire_end_of_turn_effects(&mut self) {
        self.continuous_effects.retain(|e| {
            e.duration != EffectDuration::UntilEndOfTurn
                && e.duration != EffectDuration::UntilEndOfCombat
        });
    }

    /// Revert temporary control changes (Act of Treason / Threaten) whose
    /// `Duration` is in `which`. The stolen permanent returns to whoever
    /// controlled it immediately before the steal. Entries whose card has
    /// since left the battlefield are dropped without effect (CR 800.4 —
    /// the control-changing effect simply ends).
    pub(crate) fn revert_temporary_control(&mut self, which: &[crate::effect::Duration]) {
        let mut kept = Vec::new();
        for tc in std::mem::take(&mut self.temporary_control) {
            let on_battlefield = self.battlefield.iter().any(|c| c.id == tc.card);
            if !on_battlefield {
                continue; // card left play — nothing to revert
            }
            if which.contains(&tc.duration) {
                self.change_control(tc.card, tc.original_controller);
            } else {
                kept.push(tc);
            }
        }
        self.temporary_control = kept;
    }

    /// CR 707 / 611.2c — a "becomes a copy" effect ends when the object
    /// leaves the battlefield: restore the pre-copy definition on the
    /// departing card and drop its pending revert entries. Called from the
    /// battlefield-leave funnels alongside `turn_face_up`.
    pub(crate) fn revert_copy_on_leave(&mut self, card: &mut crate::card::CardInstance) {
        // The oldest entry holds the original printed definition.
        if let Some(pos) = self.temporary_copies.iter().position(|tc| tc.card == card.id) {
            if let Some(def) = self.temporary_copies[pos].original_def() {
                card.definition = def;
            }
            self.temporary_copies.retain(|tc| tc.card != card.id);
        }
    }

    /// Revert temporary "becomes a copy" definition swaps
    /// (`Effect::BecomeCopyOfFor`) whose `Duration` is in `which`. Reverted
    /// in reverse order so stacked copies unwind to the oldest original.
    /// Entries whose card left the battlefield are dropped (the copy effect
    /// ended with the object).
    pub(crate) fn revert_temporary_copies(&mut self, which: &[crate::effect::Duration]) {
        let mut kept = Vec::new();
        for tc in std::mem::take(&mut self.temporary_copies).into_iter().rev() {
            if !self.battlefield.iter().any(|c| c.id == tc.card) {
                continue; // card left play — nothing to revert
            }
            if which.contains(&tc.duration) {
                if let Some(def) = tc.original_def()
                    && let Some(c) = self.battlefield.iter_mut().find(|c| c.id == tc.card)
                {
                    c.definition = def;
                }
            } else {
                kept.push(tc);
            }
        }
        kept.reverse();
        self.temporary_copies = kept;
    }

    /// Expire all `UntilEndOfCombat` continuous effects (CR 511.2 —
    /// "Effects that last 'until end of combat' expire at the end of the
    /// combat phase"). Invoked from `do_combat_end` once the end-of-
    /// combat step finishes.
    pub(crate) fn expire_end_of_combat_effects(&mut self) {
        self.continuous_effects
            .retain(|e| e.duration != EffectDuration::UntilEndOfCombat);
    }

    /// Sacrifice/exile Mobilize/Myriad tokens registered by
    /// `Effect::CreateTokenAttacking` as the combat phase ends (CR 511.3).
    pub(crate) fn process_attacking_token_cleanup(&mut self) -> Vec<GameEvent> {
        use crate::effect::AttackingTokenCleanup;
        let mut events = Vec::new();
        for (id, kind) in std::mem::take(&mut self.attacking_token_cleanup) {
            if !self.battlefield.iter().any(|c| c.id == id) {
                continue; // already gone (died in combat, bounced, etc.)
            }
            let who = self.battlefield_find(id).map(|c| c.controller).unwrap_or(0);
            match kind {
                AttackingTokenCleanup::SacrificeAtEndOfCombat => {
                    // Shared sacrifice funnel — die snapshot included.
                    self.sacrifice_one(id, who, &mut events);
                }
                AttackingTokenCleanup::ExileAtEndOfCombat => {
                    self.remove_from_battlefield_to_exile(id);
                }
                AttackingTokenCleanup::None => {}
            }
        }
        events.append(&mut self.check_state_based_actions());
        events
    }

    /// True if the stack is empty and it is `player`'s main phase — sorcery timing.
    pub fn can_cast_sorcery_speed(&self, player: usize) -> bool {
        self.stack.is_empty()
            && self.step.is_main_phase()
            && self.active_player_idx == player
            && self.priority.player_with_priority == player
    }

    pub(crate) fn next_id(&mut self) -> CardId {
        let id = CardId(self.next_id);
        self.next_id += 1;
        id
    }

    // ── Public setup helpers (useful in tests) ────────────────────────────────

    /// Add a card to a player's hand without going through library/draw mechanics.
    pub fn add_card_to_hand(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .hand
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Put a card directly onto the battlefield (enters with summoning sickness unless cleared).
    pub fn add_card_to_battlefield(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        let mut inst = CardInstance::new(id, def, player_idx);
        inst.battlefield_timestamp = self.next_timestamp();
        self.battlefield.push(inst);
        id
    }

    /// Drop a token onto the battlefield directly (test fixture). Mirrors
    /// `add_card_to_battlefield` but uses `CardInstance::new_token` so the
    /// `is_token` flag is set — required for SBA path 704.5d (tokens not on
    /// the battlefield cease to exist) and for filters that consult
    /// `c.is_token`. Used by tribal-anthem and aristocrats tests that need
    /// a token board state without round-tripping through a spell cast.
    pub fn add_token_to_battlefield(
        &mut self,
        player_idx: usize,
        token: &crate::card::TokenDefinition,
    ) -> CardId {
        let id = self.next_id();
        let def = crate::game::effects::token_to_card_definition(token);
        let mut inst = CardInstance::new_token(id, def, player_idx);
        inst.battlefield_timestamp = self.next_timestamp();
        self.battlefield.push(inst);
        id
    }

    /// Put a card onto the battlefield through the real ETB movement funnel
    /// (`move_card_to`), so enters-with-counters / chosen-type / fading
    /// replacements and self-source ETB triggers all fire — unlike
    /// `add_card_to_battlefield`, which pushes directly. Test fixture for
    /// exercising entry-replacement statics (Metallic Mimic).
    pub fn move_card_to_battlefield_for_test(
        &mut self,
        player_idx: usize,
        def: CardDefinition,
    ) -> CardId {
        let id = self.next_id();
        self.players[player_idx].graveyard.push(CardInstance::new(id, def, player_idx));
        let ctx = crate::game::effects::EffectContext::for_ability(id, player_idx, None);
        let mut events = Vec::new();
        self.move_card_to(
            id,
            &crate::effect::ZoneDest::Battlefield { controller: crate::effect::PlayerRef::You, tapped: false },
            &ctx,
            &mut events,
        );
        id
    }

    /// Add a card to the **bottom** of `player_idx`'s library — appends to
    /// the end of the `library` vec. Note: with an empty library the
    /// first call pushes to index 0 (the top of the deck), so test
    /// fixtures that call this once per card end up with the
    /// **first-pushed** card on top and successive pushes building down.
    /// For top-of-deck inserts use `Player::add_to_library_top` directly.
    pub fn add_card_to_library(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx].add_to_library_bottom(id, def);
        id
    }

    /// Put a card into `player_idx`'s Lessons sideboard ("outside the
    /// game"). A Learn ability may later reveal it into hand. Used by deck
    /// construction and test fixtures exercising the Learn mechanic.
    pub fn add_card_to_sideboard(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .sideboard
            .push(crate::card::CardInstance::new(id, def, player_idx));
        id
    }

    /// Put a card directly into `player_idx`'s graveyard. Useful for test
    /// fixtures that exercise flashback / reanimate / dredge paths without
    /// the bookkeeping of casting and resolving the spell first.
    pub fn add_card_to_graveyard(
        &mut self,
        player_idx: usize,
        def: CardDefinition,
    ) -> CardId {
        let id = self.next_id();
        self.players[player_idx]
            .graveyard
            .push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Put a card into the exile zone owned by `player_idx` (convenience for
    /// tests — e.g. seeding an opponent-owned card to be processed).
    pub fn add_card_to_exile(&mut self, player_idx: usize, def: CardDefinition) -> CardId {
        let id = self.next_id();
        self.exile.push(CardInstance::new(id, def, player_idx));
        id
    }

    /// Clear summoning sickness from a permanent (convenience for tests).
    pub fn clear_sickness(&mut self, id: CardId) {
        if let Some(c) = self.battlefield_find_mut(id) {
            c.summoning_sick = false;
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over.is_some()
    }

    /// Attackers declared in this combat step (with their chosen target).
    pub fn attacking(&self) -> &[Attack] {
        &self.attacking
    }

    // ── Snapshot accessors ────────────────────────────────────────────────────
    //
    // These are read/write helpers used by `crate::snapshot` to capture and
    // restore otherwise-private fields. They aren't intended for general
    // callers; the snapshot module guards round-trip correctness with tests.

    pub fn block_map(&self) -> &HashMap<CardId, CardId> {
        &self.block_map
    }

    pub fn blockers_declared(&self) -> bool {
        self.blockers_declared
    }
    pub fn skip_first_draw(&self) -> bool {
        self.skip_first_draw
    }
    pub fn peek_next_id(&self) -> u32 {
        self.next_id
    }
    pub fn set_next_id(&mut self, value: u32) {
        self.next_id = value;
    }
    pub fn set_attacking(&mut self, attacks: Vec<Attack>) {
        self.attacking = attacks;
    }
    pub fn set_block_map(&mut self, map: HashMap<CardId, CardId>) {
        self.block_map = map;
    }
    pub fn set_blockers_declared(&mut self, value: bool) {
        self.blockers_declared = value;
    }
    pub fn set_skip_first_draw(&mut self, value: bool) {
        self.skip_first_draw = value;
    }

    /// Convenience: just the IDs of all declared attackers.
    pub fn attacking_ids(&self) -> Vec<CardId> {
        self.attacking.iter().map(|a| a.attacker).collect()
    }

    /// Snapshot of the current blocker → attacker assignments. Lets the
    /// view layer expose blocks per-permanent without making `block_map`
    /// public.
    /// CR 510.1c — attackers that became blocked this combat (they stay
    /// blocked even if every blocker has since left combat).
    pub fn blocked_attackers(&self) -> &[CardId] {
        &self.blocked_attackers
    }

    pub fn block_map_snapshot(&self) -> Vec<(CardId, CardId)> {
        self.block_map.iter().map(|(b, a)| (*b, *a)).collect()
    }

    /// Look up the attack record for a given attacker id, if any.
    pub fn attack_for(&self, attacker: CardId) -> Option<&Attack> {
        self.attacking.iter().find(|a| a.attacker == attacker)
    }

    /// Resolve the defending player for a given attack target.
    pub fn defender_for(&self, target: AttackTarget) -> Option<usize> {
        match target {
            AttackTarget::Player(p) => Some(p),
            AttackTarget::Planeswalker(pw) => {
                self.battlefield_find(pw).map(|c| c.controller)
            }
            // CR 508.4 — the defending player for an attack on a battle is its
            // protector, who defends it with their creatures.
            AttackTarget::Battle(b) => {
                self.battlefield_find(b).and_then(|c| c.protected_by)
            }
        }
    }


    /// True if `blocker_id` can legally block at least one current attacker.
    /// Each declared attacker's live `CardInstance` paired with its computed
    /// view, gathered once. Hoisted out of the per-blocker scan so
    /// `legal_blockers` / the bot's block planner pay `apply_layers_one`
    /// per *attacker*, not per attacker × blocker (audit P2).
    pub(crate) fn computed_attackers(&self) -> Vec<(&CardInstance, ComputedPermanent)> {
        self.attacking
            .iter()
            .filter_map(|atk| {
                let card = self.battlefield.iter().find(|c| c.id == atk.attacker)?;
                let cp = self.computed_permanent(atk.attacker)?;
                Some((card, cp))
            })
            .collect()
    }

    /// True if `blocker_id` can legally block at least one of the prebuilt
    /// attacker views (build them with [`computed_attackers`]).
    ///
    /// [`computed_attackers`]: Self::computed_attackers
    pub(crate) fn can_block_any_computed_attacker(
        &self,
        blocker_id: CardId,
        attackers: &[(&CardInstance, ComputedPermanent)],
    ) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        // Only the blocker's and each attacker's computed views are needed —
        // avoid paying the whole-board `compute_battlefield` per candidate
        // (this runs per blocker in `legal_blockers` / the bot's block scan).
        let Some(blocker_cp) = self.computed_permanent(blocker_id) else {
            return false;
        };
        let blocker_cp = &blocker_cp;
        // CR 509.1a — creature-ness from the computed view (animated lands /
        // crewed Vehicles can block).
        if !blocker_cp.card_types.contains(&crate::card::CardType::Creature) || blocker.tapped {
            return false;
        }
        // Honor `Keyword::CantBlock` from the computed keyword set —
        // transient grants from pump spells (Duel Tactics) and static
        // restrictions (Postmortem Professor) both surface here.
        if blocker_cp.keywords.contains(&Keyword::CantBlock) {
            return false;
        }
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessEvenCounters)
            && blocker.counters.values().sum::<u32>() % 2 != 0
        {
            return false;
        }
        attackers.iter().any(|(_, atk_cp)| {
            can_block_attacker_computed(
                blocker,
                blocker_cp,
                atk_cp.keywords.as_slice(),
                atk_cp.colors.as_slice(),
                atk_cp.power,
            )
        })
    }

    /// True if `blocker_id` can legally block `attacker_id`.
    pub fn blocker_can_block_attacker(&self, blocker_id: CardId, attacker_id: CardId) -> bool {
        let Some(blocker) = self.battlefield.iter().find(|c| c.id == blocker_id) else {
            return false;
        };
        let Some(attacker) = self.battlefield.iter().find(|c| c.id == attacker_id) else {
            return false;
        };
        // Per-id computed views — see `can_block_any_attacker`.
        let Some(blocker_cp) = self.computed_permanent(blocker_id) else {
            return false;
        };
        let blocker_cp = &blocker_cp;
        if !blocker_cp.card_types.contains(&crate::card::CardType::Creature) || blocker.tapped {
            return false;
        }
        // CR 702.147 — Decayed creatures can't block (mirrors the
        // DeclareBlockers validation in `combat.rs`).
        if blocker_cp.keywords.contains(&Keyword::CantBlock)
            || blocker_cp.keywords.contains(&Keyword::Decayed)
        {
            return false;
        }
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessEvenCounters)
            && blocker.counters.values().sum::<u32>() % 2 != 0
        {
            return false;
        }
        // CR 509.1a — Hazoret-class: can't block unless hand is small.
        if blocker_cp.keywords.iter().any(|k| {
            matches!(k, Keyword::CantAttackOrBlockUnlessHandSizeAtMost(n)
                if self.players[blocker.controller].hand.len() as u32 > *n)
        }) {
            return false;
        }
        // CR 509.1a — Delirium gate (Patchwork Beastie).
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessDelirium)
            && !self.delirium_active(blocker.controller)
        {
            return false;
        }
        // CR 509.1a — "a creature died under your control this turn" gate (Bontu).
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn)
            && self.players[blocker.controller].creatures_died_this_turn == 0
        {
            return false;
        }
        // CR 509.1a — Descend N gate (The Ancient One).
        if blocker_cp.keywords.iter().any(|k| {
            matches!(k, Keyword::CantAttackOrBlockUnlessDescend(n)
                if self.descend_count(blocker.controller) < *n as usize)
        }) {
            return false;
        }
        // CR 509.1a — city's blessing gate (Wayward Swordtooth).
        if blocker_cp.keywords.contains(&Keyword::CantAttackOrBlockUnlessCityBlessing)
            && !self.players[blocker.controller].city_blessing
        {
            return false;
        }
        // "Can't block unless you control N+ [filter]" (Topiary Stomper).
        // Attack-only gates (Lambholt Pacifist) don't restrict blocking.
        if let Some((req, min, excl)) = blocker_cp.keywords.iter().find_map(|kw| match kw {
            Keyword::CantAttackOrBlockUnlessYouControlCount {
                filter, min, attack_only: false, exclude_self, ..
            } => Some((filter.clone(), *min, *exclude_self)),
            _ => None,
        }) {
            let owner = blocker.controller;
            let n = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == owner
                        && !(excl && c.id == blocker.id)
                        && self.evaluate_requirement_on_card(&req, c, owner)
                })
                .count();
            if (n as u32) < min {
                return false;
            }
        }
        let atk_cp = self.computed_permanent(attacker_id);
        let atk_kws = atk_cp.as_ref().map(|c| c.keywords.as_slice()).unwrap_or(&[]);
        let atk_colors = atk_cp.as_ref().map(|c| c.colors.as_slice()).unwrap_or(&[]);
        let atk_power = atk_cp.as_ref().map(|c| c.power).unwrap_or_else(|| attacker.power());
        // CR 701.54c (level 1+) — "Your Ring-bearer … can't be blocked by
        // creatures with greater power." Same shape as Skulk, but keyed on the
        // attacker being its controller's Ring-bearer.
        if self.effective_ring_bearer(attacker.controller) == Some(attacker_id)
            && self.players[attacker.controller].ring_temptations >= 1
            && blocker_cp.power > atk_power
        {
            return false;
        }
        can_block_attacker_computed(blocker, blocker_cp, atk_kws, atk_colors, atk_power)
    }

    // ── Main action dispatch ──────────────────────────────────────────────────

    pub fn perform_action(&mut self, action: GameAction) -> Result<Vec<GameEvent>, GameError> {
        if self.is_game_over() {
            return Err(GameError::GameAlreadyOver);
        }
        // Routing for decision answers is unconditional; everything else must
        // wait until the pending decision is resolved.
        if let GameAction::SubmitDecision(answer) = action {
            return self.submit_decision(answer);
        }
        if self.pending_decision.is_some() {
            return Err(GameError::DecisionPending);
        }
        // Revel in Silence-style lock: a silenced player can't cast spells
        // or activate loyalty abilities this turn. Gated here so every
        // Cast* action variant is covered at once.
        if action.is_cast_or_loyalty()
            && self.players[self.priority.player_with_priority].silenced_this_turn
        {
            return Err(GameError::SilencedThisTurn);
        }
        // CR 702.50b — a player can't cast spells once an epic spell they
        // control resolves (the per-upkeep copies are put on the stack by
        // the epic ability itself, not cast).
        if action.is_cast()
            && !self.players[self.priority.player_with_priority].epic_spells.is_empty()
        {
            return Err(GameError::EpicLocked);
        }
        // Rule of Law-style one-spell-per-turn locks — gated here so every
        // Cast* variant is covered at once. The plain `OneSpellPerTurn` lock
        // (Rule of Law) applies to any spell; `OneNoncreatureSpellPerTurn`
        // (Deafening Silence) and `OneNonartifactSpellPerTurn` (Ethersworn
        // Canonist) only count spells of the matching type.
        if action.is_cast() {
            use crate::effect::StaticEffect;
            let pl = &self.players[self.priority.player_with_priority];
            // The card types of the spell being cast (None for prepare spells,
            // which don't carry the cast card in a `card_id` field).
            let cast_types = action
                .cast_card_id()
                .and_then(|id| self.find_card_anywhere(id))
                .map(|c| c.definition.card_types.clone());
            let is_creature =
                cast_types.as_ref().is_some_and(|t| t.contains(&CardType::Creature));
            let is_artifact =
                cast_types.as_ref().is_some_and(|t| t.contains(&CardType::Artifact));
            let blocked = self.battlefield.iter().any(|c| {
                c.definition.static_abilities.iter().any(|sa| match sa.effect {
                    StaticEffect::OneSpellPerTurn => pl.spells_cast_this_game_turn >= 1,
                    StaticEffect::OneNoncreatureSpellPerTurn => {
                        !is_creature && pl.noncreature_spells_cast_this_game_turn >= 1
                    }
                    StaticEffect::OneNonartifactSpellPerTurn => {
                        !is_artifact && pl.nonartifact_spells_cast_this_game_turn >= 1
                    }
                    _ => false,
                })
            });
            if blocked {
                return Err(GameError::SpellLimitReached);
            }
        }
        // CR 702.61 — split second: while such a spell is on the stack no
        // player may cast spells or activate non-mana abilities. Special
        // actions (land drops, foretell, plot, turning face up, suspend…)
        // and triggered abilities are unaffected (702.61b).
        if self.stack_has_split_second() && self.split_second_blocks(&action) {
            return Err(GameError::SplitSecondLock);
        }
        // Iona, Shield of Emeria — opponents can't cast spells of the
        // chosen color (read off the printed cost's pips).
        if action.is_cast() {
            let caster = self.priority.player_with_priority;
            let cast_colors: Vec<crate::mana::Color> = action
                .cast_card_id()
                .and_then(|id| self.find_card_anywhere(id))
                .map(|c| c.definition.printed_colors())
                .unwrap_or_default();
            let locked = self.battlefield.iter().any(|c| {
                !self.same_team(c.controller, caster)
                    && c.chosen_color.is_some_and(|col| cast_colors.contains(&col))
                    && c.definition.static_abilities.iter().any(|sa| {
                        matches!(
                            sa.effect,
                            crate::effect::StaticEffect::OpponentsCantCastChosenColor
                        )
                    })
            });
            if locked {
                return Err(GameError::SilencedThisTurn);
            }
        }
        // Void Winnower — opponents can't cast spells with even mana values
        // (zero is even; read off the printed cost, X counts as 0).
        if action.is_cast() {
            let caster = self.priority.player_with_priority;
            let even_mv = action
                .cast_card_id()
                .and_then(|id| self.find_card_anywhere(id))
                .is_some_and(|c| c.definition.cost.cmc() % 2 == 0);
            if even_mv {
                let locked = self.battlefield.iter().any(|c| {
                    !self.same_team(c.controller, caster)
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::OpponentsCantCastEvenMv
                            )
                        })
                });
                if locked {
                    return Err(GameError::SilencedThisTurn);
                }
            }
        }
        // Voice of Victory — the active player's opponents can't cast spells
        // during that player's turn.
        if action.is_cast() {
            let caster = self.priority.player_with_priority;
            let active = self.active_player_idx;
            let locked = caster != active
                && !self.same_team(caster, active)
                && self.battlefield.iter().any(|c| {
                    c.controller == active
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::OpponentsCantCastDuringYourTurn
                                    | crate::effect::StaticEffect::OpponentsCantActDuringYourTurn
                            )
                        })
                });
            if locked {
                return Err(GameError::SilencedThisTurn);
            }
        }
        let events = match action {
            GameAction::PlayLand(id) => self.play_land(id),
            GameAction::PlayLandBack(id) => self.play_land_with_face(id, true),
            GameAction::PlayLandFromGraveyard(id) => self.play_land_from_graveyard(id),
            GameAction::CompanionToHand(card_id) => self.companion_to_hand(card_id),
            GameAction::CastSpell {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellKicked {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_kicked(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellBuyback {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_buyback(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellEntwine {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_entwine(card_id, target, additional_targets, mode, x_value),
            GameAction::CastBestow {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_bestow(card_id, target, additional_targets, mode, x_value),
            GameAction::CastRoomDoor { card_id, right } => self.cast_room_door(card_id, right),
            GameAction::UnlockRoomDoor { card_id, right } => self.unlock_room_door(card_id, right),
            GameAction::Suspend { card_id } => self.suspend_card(card_id),
            GameAction::Foretell { card_id } => self.foretell_card(card_id),
            GameAction::CastFaceDown { card_id } => self.cast_face_down(card_id),
            GameAction::TurnFaceUp { card_id } => self.turn_face_up_action(card_id),
            GameAction::CastForetold {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_foretold(card_id, target, additional_targets, mode, x_value),
            GameAction::CastAdventure {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_adventure(card_id, target, additional_targets, mode, x_value),
            GameAction::CastOmen {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_omen(card_id, target, additional_targets, mode, x_value),
            GameAction::CastGift {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_with_convoke(
                card_id, target, additional_targets, mode, x_value, &[], &[],
                crate::game::actions::CastFlags { gift: true, ..Default::default() },
            ),
            GameAction::CastAdventureCreature {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_adventure_creature(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSplitRight {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_split_half(card_id, target, additional_targets, mode, x_value, false),
            GameAction::CastSplitFused {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_split_half(card_id, target, additional_targets, mode, x_value, true),
            GameAction::CastAftermath {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_aftermath(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellCasualty {
                card_id,
                sacrifice,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_casualty(card_id, sacrifice, target, additional_targets, mode, x_value),
            GameAction::CastSpellSquad {
                card_id,
                times,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_squad(card_id, times, target, additional_targets, mode, x_value),
            GameAction::CastSpellMultikicked {
                card_id,
                times,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_multikicked(card_id, times, target, additional_targets, mode, x_value),
            GameAction::CastSpellReplicate {
                card_id,
                times,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_replicate(card_id, times, target, additional_targets, mode, x_value),
            GameAction::CastSpellConspire {
                card_id,
                conspire_creatures,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_conspire(card_id, conspire_creatures, target, additional_targets, mode, x_value),
            GameAction::CastSpellSacrificeReduce {
                card_id,
                sacrifices,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_sacrifice_reduce(
                card_id, sacrifices, target, additional_targets, mode, x_value,
            ),
            GameAction::CastSpellBargain {
                card_id,
                sacrifice,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_bargain(
                card_id, sacrifice, target, additional_targets, mode, x_value,
            ),
            GameAction::CastSpellSpree {
                card_id,
                spree_modes,
                target,
                additional_targets,
                x_value,
            } => self.cast_spell_spree(card_id, spree_modes, target, additional_targets, x_value),
            GameAction::Plot { card_id } => self.plot_card(card_id),
            GameAction::CastPlotted {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_plotted(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellSpliced {
                card_id,
                splice_cards,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_spliced(card_id, &splice_cards, target, additional_targets, mode, x_value),
            GameAction::CastSpellConvoke {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                convoke_creatures,
            } => self.cast_spell_with_convoke(card_id, target, additional_targets, mode, x_value, &convoke_creatures, &[], crate::game::actions::CastFlags::default()),
            GameAction::CastSpellWaterbend {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                helpers,
            } => self.cast_spell_waterbend(card_id, target, additional_targets, mode, x_value, &helpers),
            GameAction::CastSpellDelve {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                delve_cards,
            } => self.cast_spell_with_delve(card_id, target, additional_targets, mode, x_value, &delve_cards),
            GameAction::CastSpellAlternative {
                card_id,
                pitch_card,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_alternative(card_id, pitch_card, target, additional_targets, mode, x_value),
            GameAction::CastFlashback {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_flashback(card_id, target, additional_targets, mode, x_value),
            GameAction::CastMayhem {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_flashback(card_id, target, additional_targets, mode, x_value),
            GameAction::CastHarmonize {
                card_id,
                tap_creature,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_harmonize(card_id, tap_creature, target, additional_targets, mode, x_value),
            GameAction::CastDisturb { card_id, target, additional_targets } => {
                self.cast_disturb(card_id, target, additional_targets)
            }
            GameAction::CastRetrace {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_retrace(card_id, target, additional_targets, mode, x_value),
            GameAction::CastEscape {
                card_id,
                exile_cards,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_escape(card_id, &exile_cards, target, additional_targets, mode, x_value),
            GameAction::CastFlashbackTap {
                card_id,
                tap_creatures,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_flashback_tap(card_id, &tap_creatures, target, additional_targets, mode, x_value),
            GameAction::CastFromZoneWithoutPaying {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_from_zone_without_paying(
                card_id, target, additional_targets, mode, x_value,
            ),
            GameAction::CastFromCommandZone {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_from_command_zone(card_id, target, additional_targets, mode, x_value),
            GameAction::CastSpellBack {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_spell_back_face(card_id, target, additional_targets, mode, x_value),
            GameAction::CastPrototype {
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_prototype(card_id, target, additional_targets, mode, x_value),
            GameAction::CastMutate {
                card_id,
                target,
                on_top,
                x_value,
            } => self.cast_mutate(card_id, target, on_top, x_value),
            GameAction::CastPrepareSpell {
                creature_id,
                target,
                additional_targets,
                mode,
                x_value,
            } => self.cast_prepare_spell(creature_id, target, additional_targets, mode, x_value),
            GameAction::ActivateAbility {
                card_id,
                ability_index,
                target,
                additional_targets,
                x_value,
            } => self.activate_ability(card_id, ability_index, target, additional_targets, x_value),
            GameAction::ActivateAbilityWaterbend {
                card_id,
                ability_index,
                target,
                additional_targets,
                x_value,
                helpers,
            } => self.activate_ability_waterbend(
                card_id, ability_index, target, additional_targets, x_value, &helpers,
            ),
            GameAction::ActivateLoyaltyAbility {
                card_id,
                ability_index,
                target,
                x_value,
            } => self.activate_loyalty_ability(card_id, ability_index, target, x_value),
            GameAction::DeclareAttackers(ids) => self.declare_attackers(ids),
            GameAction::DeclareBlockers(assignments) => self.declare_blockers(assignments),
            GameAction::PassPriority => self.pass_priority(),
            GameAction::SubmitDecision(_) => unreachable!(),
            GameAction::Cycle { card_id, x_value } => self.cycle_card(card_id, x_value),
            GameAction::Reinforce { card_id, target } => self.reinforce_card(card_id, target),
            GameAction::ActivateDiscardAbility { card_id } => {
                self.activate_discard_ability(card_id)
            }
            GameAction::Landcycle { card_id } => self.landcycle_card(card_id),
            GameAction::Equip { equipment, target } => self.equip(equipment, target),
            GameAction::Reconfigure { equipment, target } => {
                self.reconfigure(equipment, target)
            }
            GameAction::Crew { vehicle, crew_creatures } => self.crew(vehicle, &crew_creatures),
            GameAction::Saddle { mount, creatures } => self.saddle(mount, &creatures),
            GameAction::Ninjutsu { ninja, returning } => self.ninjutsu(ninja, returning),
            // Fallback attribution for direct (non-networked) callers — the
            // server intercepts `Concede` in `handle_action` and routes it to
            // the *sending* seat via `concede`, bypassing this path entirely.
            GameAction::Concede => Ok(self.concede(self.active_player_idx)),
        }?;
        let mut events = events;
        // CR 119.3c — life paid as a cost (Phyrexian pips, life costs) is a
        // life-loss event; surface it after the action so loss triggers fire.
        events.extend(std::mem::take(&mut self.pending_cost_events));
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    /// CR 104.3a — `seat` concedes and leaves the game immediately. Legal at
    /// any time, regardless of priority, so this does *not* go through the
    /// priority-gated action path. Marks the player eliminated, removes the
    /// objects that leave with them (CR 800.4a), then runs state-based
    /// actions, which resolve the win/draw for the remaining team(s).
    ///
    /// No-ops (returns no events) if `seat` is out of range, already
    /// eliminated, or the game is already over.
    pub fn concede(&mut self, seat: usize) -> Vec<GameEvent> {
        if seat >= self.players.len()
            || self.players[seat].eliminated
            || self.game_over.is_some()
        {
            return Vec::new();
        }
        self.players[seat].eliminated = true;
        self.players[seat].loss_cause.get_or_insert(crate::player::LossCause::Other);
        let mut events = vec![GameEvent::PlayerConceded { player: seat }];
        // CR 800.4a — the conceding player's objects leave with them. SBAs
        // skip already-eliminated seats, so this won't fire for them there.
        self.objects_leave_with_player(seat);
        // Resolve the game-over / surviving-team determination.
        events.extend(self.check_state_based_actions());
        events
    }

    /// CR 701.8 / 702.35 — discard `card_id` from player `p`'s hand. This
    /// is the single hand-to-graveyard discard path; the random/chosen
    /// `Effect::Discard` branches both route through it so the discard
    /// bookkeeping and the Madness replacement live in one place.
    ///
    /// The discard itself always happens (`CardDiscarded` fires and the
    /// per-resolution discard-matters counters bump) regardless of where
    /// the card ends up. CR 702.35a: a discarded card with
    /// `Keyword::Madness` is exiled instead of going to the graveyard, then
    /// its owner is offered a cast for the madness cost (see
    /// `offer_madness_cast`); declining or being unable to pay sends it on
    /// to the graveyard (CR 702.35b). Returns `true` if the card was found
    /// and discarded.
    pub(crate) fn discard_card(
        &mut self,
        p: usize,
        card_id: crate::card::CardId,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let Some(card) = self.players[p].remove_from_hand(card_id) else {
            return false;
        };
        let was_creature = card
            .definition
            .card_types
            .contains(&crate::card::CardType::Creature);
        let was_nonland = !card.definition.card_types.contains(&crate::card::CardType::Land);
        let madness = card.definition.madness_cost().cloned();

        // The discard happens regardless of the destination zone (CR
        // 701.8b), so emit the event + bump the discard-matters counters
        // up front, before resolving the Madness replacement.
        events.push(GameEvent::CardDiscarded { player: p, card_id });
        self.players[p].cards_discarded_this_turn =
            self.players[p].cards_discarded_this_turn.saturating_add(1);
        self.players[p].discarded_this_turn.insert(card_id);
        self.cards_discarded_this_resolution += 1;
        self.last_discarded_mana_value = Some(card.definition.cost.cmc());
        self.last_discarded_card_types = card.definition.card_types.len() as u32;
        *self
            .cards_discarded_per_player_this_resolution
            .entry(p)
            .or_insert(0) += 1;
        self.discarded_card_ids_this_resolution.push(card_id);
        if was_creature {
            self.creature_cards_discarded_this_resolution += 1;
        }
        if was_nonland {
            *self
                .nonland_cards_discarded_per_player_this_resolution
                .entry(p)
                .or_insert(0) += 1;
        }

        let is_land = !was_nonland;
        match madness {
            None => {
                // CR 614.6 — through the graveyard funnel so Rest in Peace /
                // Leyline hate redirects the discard to exile.
                self.route_to_graveyard(card, events);
            }
            Some(cost) => {
                // CR 702.35a — exile instead of graveyard, then offer the
                // cast for the madness cost.
                self.exile.push(card);
                if !self.offer_madness_cast(p, card_id, &cost, events) {
                    // CR 702.35b — declined / unaffordable: the card goes
                    // from exile to its owner's graveyard.
                    if let Some(c) = Self::take_card(&mut self.exile, card_id) {
                        let owner = c.owner;
                        self.players[owner].send_to_graveyard(c);
                        events.push(GameEvent::CardPutIntoGraveyard {
                            player: owner, card_id, is_land,
                        });
                    }
                }
            }
        }
        true
    }

    /// CR 702.35b — offer the owner of an exiled Madness card a yes/no cast
    /// for `cost`, paid from their floated mana pool. Returns `true` if the
    /// spell was cast (it is now on the stack, sourced from exile). Mirrors
    /// the `Effect::MayPay` decision/payment shape; the `AutoDecider`
    /// declines by default so ordinary bot games never auto-cast.
    fn offer_madness_cast(
        &mut self,
        p: usize,
        card_id: crate::card::CardId,
        cost: &crate::mana::ManaCost,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let answer = self.decider.decide(&Decision::OptionalTrigger {
            source: card_id,
            description: "Cast for madness".to_string(),
        });
        if !matches!(answer, DecisionAnswer::Bool(true)) {
            return false;
        }
        // Pre-flight: try paying. On failure (unaffordable pool), decline.
        // Snapshot first — a failed cast must refund the madness payment
        // (CR 601.2h atomicity).
        let pool_before = self.players[p].mana_pool.clone();
        if self.players[p].mana_pool.pay(cost).is_err() {
            return false;
        }
        match self.cast_card_for_free(
            p,
            card_id,
            crate::card::Zone::Exile,
            None,
            vec![],
            None,
            None,
            false,
        ) {
            Ok(mut ev) => {
                events.append(&mut ev);
                true
            }
            Err(_) => {
                self.players[p].mana_pool = pool_before;
                false
            }
        }
    }

    /// CR 702.29a — Activate Cycling on `card_id` from the active
    /// player's hand. Pre-flight gates: card must be in someone's hand
    /// (we use the priority holder's hand), must carry
    /// `Keyword::Cycling(cost)`, and the controller must be able to
    /// pay the mana cost from their pool. On success: pays the cost,
    /// discards the card to the controller's graveyard, then draws a
    /// card. Per CR 702.29c, "When you cycle this card" triggers fire
    /// from the discarded zone (graveyard); the engine emits
    /// `GameEvent::CardDiscarded` from `discard_card_from_hand` so
    /// discard-matters triggers see the cycle.
    fn cycle_card(
        &mut self,
        card_id: crate::card::CardId,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        let seat = self.player_with_priority();
        // Locate the card in `seat`'s hand and clone the cycling cost —
        // mana (`Cycling`) or life ("Cycling—Pay 2 life", `CyclingLife`).
        let (cycling_cost, life_cost) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Cycling(mc) => Some((Some(mc.clone()), 0)),
                    Keyword::CyclingLife(n) => Some((None, *n)),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        if life_cost > 0 && self.players[seat].life < life_cost as i32 {
            return Err(GameError::InsufficientLife);
        }
        // Pay the cycling cost from the floated mana pool; an {X} in the
        // cost (Shark Typhoon's {X}{1}{U}) is paid as `x_value` generic.
        let x = x_value.unwrap_or(0);
        if let Some(mc) = &cycling_cost {
            let mc = if mc.has_x() { mc.with_x_value(x) } else { mc.clone() };
            self.players[seat].mana_pool.pay(&mc).map_err(GameError::Mana)?;
        }
        if life_cost > 0 {
            self.adjust_life(seat, -(life_cost as i32));
        }
        // Discard the card from hand via the centralized path (handles the
        // graveyard move, CardDiscarded, discard-matters counters, and the
        // Madness replacement, CR 702.35).
        let mut events = vec![];
        let cycled_name = self
            .find_card_anywhere(card_id)
            .map(|c| c.definition.name.to_string());
        if self.discard_card(seat, card_id, &mut events) {
            // CR 702.29c — emit the cycle-specific event in addition to
            // the discard event, so "When you cycle this card" triggers
            // distinguish cycle from a regular hand discard.
            if let Some(name) = cycled_name {
                *self.cycled_count_by_name.entry(name).or_insert(0) += 1;
            }
            events.push(GameEvent::CardCycled {
                player: seat,
                card_id,
                x,
            });
        }
        // Draw a card (Dredge can replace this draw, CR 702.52).
        self.draw_one(seat, &mut events);
        Ok(events)
    }

    /// CR 702.77 — Activate a Reinforce ability from the hand. Pays the cost,
    /// discards the card (firing discard triggers), then puts N +1/+1 counters
    /// on the targeted creature.
    fn reinforce_card(
        &mut self,
        card_id: crate::card::CardId,
        target: crate::game::types::Target,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::{CounterType, Keyword};
        let seat = self.player_with_priority();
        let (cost, n) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Reinforce(n, mc) => Some((mc.clone(), *n)),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Target must be a creature on the battlefield.
        let crate::game::types::Target::Permanent(tid) = target else {
            return Err(GameError::InvalidTarget);
        };
        if !self
            .battlefield
            .iter()
            .any(|c| c.id == tid && c.definition.is_creature())
        {
            return Err(GameError::InvalidTarget);
        }
        self.players[seat].mana_pool.pay(&cost).map_err(GameError::Mana)?;
        let mut events = vec![];
        self.discard_card(seat, card_id, &mut events);
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == tid) {
            c.add_counters(CounterType::PlusOnePlusOne, n);
        }
        Ok(events)
    }

    /// Activate a card's `discard_activated` ability from the hand: pay the
    /// cost, discard the card, then resolve its (targetless) effect.
    fn activate_discard_ability(
        &mut self,
        card_id: crate::card::CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        let seat = self.player_with_priority();
        let (cost, effect) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition
                    .discard_activated
                    .as_ref()
                    .map(|d| (d.cost.clone(), d.effect.clone()))
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        self.players[seat].mana_pool.pay(&cost).map_err(GameError::Mana)?;
        let mut events = vec![];
        self.discard_card(seat, card_id, &mut events);
        events.extend(self.continue_ability_resolution(card_id, seat, effect, None)?);
        Ok(events)
    }

    /// CR 702.29e — Activate a Landcycling ability. Pays the cost, discards the
    /// card (firing cycle/discard triggers, since typecycling *is* a cycling
    /// ability), then searches the library for a land of the keyword's land
    /// type and puts it into hand (shuffling after). The fetched land is the
    /// first matching card (a minor approximation for the rare multi-match
    /// case — usually a basic land).
    /// Typecycling granted to a hand card by a battlefield static
    /// (`GrantTypecyclingToHandCards` — Homing Sliver's slivercycling).
    /// Returns the cheapest matching grant's `(cost, search filter)`.
    pub fn granted_typecycling_for(
        &self,
        card: &crate::card::CardInstance,
    ) -> Option<(crate::mana::ManaCost, SelectionRequirement)> {
        self.battlefield
            .iter()
            .flat_map(|src| src.definition.static_abilities.iter().map(move |sa| (src, sa)))
            .filter_map(|(src, sa)| {
                let crate::effect::StaticEffect::GrantTypecyclingToHandCards {
                    filter,
                    cost,
                    search,
                } = &sa.effect
                else {
                    return None;
                };
                crate::game::layers::requirement_matches_card(filter, card, src.controller)
                    .then(|| (cost.clone(), search.clone()))
            })
            .min_by_key(|(c, _)| c.cmc())
    }

    fn landcycle_card(&mut self, card_id: crate::card::CardId) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        use rand::seq::SliceRandom;
        let seat = self.player_with_priority();
        let (cycling_cost, filter) = self.players[seat]
            .hand
            .iter()
            .find(|c| c.id == card_id)
            .and_then(|c| {
                c.definition
                    .keywords
                    .iter()
                    .find_map(|kw| match kw {
                        Keyword::Landcycling(mc, lt) => Some((
                            mc.clone(),
                            crate::card::SelectionRequirement::Land
                                .and(crate::card::SelectionRequirement::HasLandType(*lt)),
                        )),
                        Keyword::Typecycling(spec) => {
                            Some(((**spec).0.clone(), (**spec).1.clone()))
                        }
                        _ => None,
                    })
                    .or_else(|| self.granted_typecycling_for(c))
            })
            .ok_or(GameError::CardNotInHand(card_id))?;
        // Matching library cards, in library order.
        let matches: Vec<(crate::card::CardId, String)> = {
            let ids: Vec<(crate::card::CardId, String)> = self.players[seat]
                .library
                .iter()
                .map(|c| (c.id, c.definition.name.to_string()))
                .collect();
            ids.into_iter()
                .filter(|(id, _)| {
                    self.evaluate_requirement_static(
                        &filter,
                        &crate::game::types::Target::Permanent(*id),
                        seat,
                        None,
                    )
                })
                .collect()
        };
        // CR 702.29e — a `wants_ui` cycler with a real choice picks which
        // card to fetch. Suspend before any cost is paid; the resume replays
        // this action with the pick stashed.
        let stashed_pick = self.pending_landcycle_pick.take();
        if stashed_pick.is_none() && self.players[seat].wants_ui && matches.len() > 1 {
            let eligible: Vec<crate::card::CardId> = matches.iter().map(|(id, _)| *id).collect();
            self.pending_decision = Some(crate::game::types::PendingDecision {
                decision: crate::decision::Decision::SearchLibrary {
                    player: seat,
                    candidates: matches,
                    eligible: Some(eligible),
                },
                resume: crate::game::types::ResumeContext::ActionSearchPick {
                    actor: seat,
                    action: Box::new(GameAction::Landcycle { card_id }),
                },
            });
            return Ok(vec![]);
        }
        self.players[seat].mana_pool.pay(&cycling_cost).map_err(GameError::Mana)?;
        let mut events = vec![];
        if self.discard_card(seat, card_id, &mut events) {
            events.push(GameEvent::CardCycled { player: seat, card_id, x: 0 });
        }
        // Fetch the stashed pick (validated against the match set), else the
        // first match; reveal + to hand.
        let chosen = match stashed_pick {
            Some(pick) => pick.filter(|id| matches.iter().any(|(m, _)| m == id)),
            None => matches.first().map(|(id, _)| *id),
        };
        if let Some(fetched) =
            chosen.and_then(|id| Self::take_card(&mut self.players[seat].library, id))
        {
            self.place_card_in_dest(
                fetched,
                seat,
                &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(seat)),
                &mut events,
            );
        }
        self.players[seat].library.shuffle(&mut rand::rng());
        Ok(events)
    }

    /// CR 104.3c, with the 104.2 override — a failed draw from an empty
    /// library eliminates `p`, unless they control a "you win the game
    /// instead" static (Laboratory Maniac, Jace, Wielder of Mysteries):
    /// then every other player is eliminated and the SBA pass promotes the
    /// win.
    pub(crate) fn lose_to_empty_draw(&mut self, p: usize) {
        let wins = self.battlefield.iter().any(|c| {
            c.controller == p
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(
                        sa.effect,
                        crate::effect::StaticEffect::WinInsteadOfDrawFromEmpty
                    )
                })
        });
        use crate::player::LossCause;
        if wins {
            // CR 104.3d — the replacement still swallows the failed draw,
            // but a player who can't win isn't awarded the win, and
            // opponents who can't lose stay in the game.
            if self.player_cant_win_game(p) {
                return;
            }
            for idx in 0..self.players.len() {
                if idx != p && !self.player_cant_lose_game(idx) {
                    self.players[idx].eliminated = true;
                    self.players[idx].loss_cause.get_or_insert(LossCause::Other);
                }
            }
        } else if !self.player_cant_lose_game(p) {
            self.players[p].eliminated = true;
            self.players[p].loss_cause.get_or_insert(LossCause::Decked);
        }
    }

    /// Draw one card for `p`, first offering the Dredge replacement
    /// (CR 702.52). Returns `false` only when the draw couldn't be
    /// satisfied (empty library) and no dredge replacement applied — the
    /// caller is responsible for the resulting loss SBA. Pushes
    /// `CardDrawn` for a normal draw, or `CardMilled` ×N +
    /// `CardLeftGraveyard` for a dredge.
    pub(crate) fn draw_one(&mut self, p: usize, events: &mut Vec<GameEvent>) -> bool {
        if self.try_dredge_instead_of_draw(p, events) {
            return true;
        }
        let drew = match self.players[p].draw_top() {
            Some(id) => {
                events.push(GameEvent::CardDrawn { player: p, card_id: id });
                self.maybe_grant_miracle(p, id);
                true
            }
            None => false,
        };
        // CR 121.2a / 614 — "If you would draw a card, draw two instead"
        // (Thought Reflection). Each doubler applies once per draw event
        // (n doublers: 1 → 2^n); the replacement draws themselves aren't
        // re-doubled (CR 614.5), enforced by the reentrancy flag.
        if drew && !self.in_draw_double {
            let doublers = self
                .battlefield
                .iter()
                .filter(|c| {
                    c.controller == p
                        && c.definition.static_abilities.iter().any(|sa| {
                            matches!(sa.effect, crate::effect::StaticEffect::ControllerDrawsDoubled)
                        })
                })
                .count() as u32;
            if doublers > 0 {
                self.in_draw_double = true;
                for _ in 0..(1u32 << doublers.min(8)) - 1 {
                    self.draw_one(p, events);
                }
                self.in_draw_double = false;
            }
        }
        drew
    }

    /// CR 702.94 — Miracle. If `card_id` was the first card `p` drew this
    /// turn and it has a printed miracle cost, grant the miracle alt-cost
    /// until end of turn (the owner may then cast it for that cost via
    /// `GameAction::CastFromZoneWithoutPaying`). The reveal is treated as
    /// automatic — the grant only adds a cheaper *option*, so revealing is
    /// never a downside for the engine; a human simply declines to cast.
    pub(crate) fn maybe_grant_miracle(&mut self, p: usize, card_id: CardId) {
        if self.players[p].cards_drawn_this_turn != 1 {
            return;
        }
        if let Some(card) = self.players[p].hand.iter_mut().find(|c| c.id == card_id)
            && let Some(cost) = card.definition.miracle.clone()
        {
            card.may_play_until = Some(crate::card::MayPlayPermission {
                player: p,
                granted_turn: self.turn_number,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                exile_after: false,
            });
            card.granted_alt_cast_cost_eot = Some(cost);
        }
    }

    /// CR 702.52 — Dredge. If `p` has a card with `Keyword::Dredge(n)` in
    /// their graveyard and at least `n` cards in their library, the player
    /// may replace a draw by milling `n` cards and returning the dredge
    /// card to hand instead. Returns `true` when a dredge replacement was
    /// applied (caller skips the normal draw). The decision is surfaced as
    /// an `OptionalTrigger`, so the `AutoDecider` declines by default and
    /// ordinary games keep drawing normally.
    fn try_dredge_instead_of_draw(
        &mut self,
        p: usize,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        use crate::card::Keyword;
        use crate::decision::{Decision, DecisionAnswer};
        // First dredge card in the graveyard whose count the library can
        // satisfy (CR 702.52a — you can't dredge with fewer than N cards).
        let cand = self.players[p].graveyard.iter().find_map(|c| {
            c.definition.keywords.iter().find_map(|kw| match kw {
                Keyword::Dredge(n) if self.players[p].library.len() >= *n as usize => {
                    Some((c.id, *n))
                }
                _ => None,
            })
        });
        let Some((card_id, n)) = cand else { return false; };
        let answer = self.decider.decide(&Decision::OptionalTrigger {
            source: card_id,
            description: format!(
                "Dredge {n}: mill {n} card(s) and return this card to your hand instead of drawing?"
            ),
        });
        if !matches!(answer, DecisionAnswer::Bool(true)) {
            return false;
        }
        // Mill N from the top of the library.
        for _ in 0..n {
            if self.players[p].library.is_empty() {
                break;
            }
            let card = self.players[p].library.remove(0);
            let cid = card.id;
            self.players[p].send_to_graveyard(card);
            events.push(GameEvent::CardMilled { player: p, card_id: cid });
        }
        // Return the dredge card from the graveyard to its owner's hand.
        if let Some(card) = Self::take_card(&mut self.players[p].graveyard, card_id) {
            self.players[p].hand.push(card);
            self.players[p].cards_left_graveyard_this_turn = self.players[p]
                .cards_left_graveyard_this_turn
                .saturating_add(1);
            events.push(GameEvent::CardLeftGraveyard { player: p, card_id });
        }
        true
    }

    /// CR 702.6 — Activate an Equipment's equip ability, attaching it to a
    /// creature its controller controls. Equip is a special activated
    /// ability usable only at sorcery speed (CR 702.6e) and only targeting a
    /// creature you control (CR 702.6c). The equip cost (`Keyword::Equip`) is
    /// paid from the controller's floated mana pool; on success the
    /// Equipment's `attached_to` is repointed at `target`, and its
    /// `equipped_bonus` flows onto the equipped creature via the layer
    /// system (see `compute_battlefield`). Re-equipping a creature that's
    /// already wearing the Equipment is legal (it just re-pays the cost);
    /// moving from one creature to another silently detaches the old link.
    /// CR 702.6 — true if `player` controls a permanent granting "you may
    /// activate equip abilities any time you could cast an instant" (Leonin
    /// Shikari), lifting the equip sorcery-speed gate.
    fn controller_equips_at_instant_speed(&self, player: usize) -> bool {
        self.battlefield.iter().any(|c| {
            c.controller == player
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, crate::effect::StaticEffect::ControllerEquipAtInstantSpeed)
                })
        })
    }

    /// CR 702.6 — summed "equip costs you pay cost {N} less" reduction across
    /// the player's permanents (Auriok Steelshaper).
    fn equip_cost_reduction_for(&self, player: usize) -> u32 {
        self.battlefield
            .iter()
            .filter(|c| c.controller == player)
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                crate::effect::StaticEffect::EquipCostReduction { amount } => Some(amount),
                _ => None,
            })
            .sum()
    }

    fn equip(
        &mut self,
        equipment: crate::card::CardId,
        target: crate::card::CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        // Sorcery-speed gate (CR 702.6e) — unless the controller has a
        // "may activate equip abilities any time you could cast an instant"
        // static (Leonin Shikari).
        if !self.can_cast_sorcery_speed(p) && !self.controller_equips_at_instant_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        // Locate the Equipment (or Fortification — CR 702.71 fortify mirrors
        // equip with "land" for "creature"); it must be on the battlefield,
        // controlled by the activating player, with an equip/fortify cost.
        let equip_pos = self
            .battlefield
            .iter()
            .position(|c| c.id == equipment)
            .ok_or(GameError::CardNotOnBattlefield(equipment))?;
        if self.battlefield[equip_pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let fortify = self.battlefield[equip_pos].definition.has_fortify().cloned();
        if !self.battlefield[equip_pos].definition.is_equipment() && fortify.is_none() {
            return Err(GameError::NotEquipment(equipment));
        }
        let mut equip_cost = match &fortify {
            Some(c) => c.clone(),
            None => self.battlefield[equip_pos]
                .definition
                .has_equip()
                .cloned()
                .ok_or(GameError::NotEquipment(equipment))?,
        };
        // CR 702.6 — "Equip costs you pay cost {N} less" (Auriok Steelshaper).
        let reduction = self.equip_cost_reduction_for(p);
        if reduction > 0 {
            equip_cost.reduce_generic(reduction);
        }
        // The target must be a creature (equip, CR 702.6c) — or a land
        // (fortify, CR 702.71c) — the activating player controls. Use the
        // computed view so animated permanents are honored.
        let wanted = if fortify.is_some() {
            crate::card::CardType::Land
        } else {
            crate::card::CardType::Creature
        };
        let target_ok = self
            .computed_permanent(target)
            .is_some_and(|c| c.controller == p && c.card_types.contains(&wanted));
        if !target_ok {
            return Err(GameError::InvalidTarget);
        }
        // CR 702.16f — a creature can't be equipped by an Equipment whose
        // color it has protection from.
        if self.is_protected_from(equipment, target) {
            return Err(GameError::TargetHasProtection(target));
        }
        // "Equip—Pay {E}" (Inventor's Axe): an energy surcharge on top of the
        // mana cost. Gate before spending any mana so a failed pay is atomic.
        let energy_cost = self.battlefield[equip_pos].definition.equip_energy_cost;
        if energy_cost > 0 && self.players[p].energy < energy_cost {
            return Err(GameError::InsufficientEnergy);
        }
        // Pay the equip cost from the floated mana pool.
        self.players[p]
            .mana_pool
            .pay(&equip_cost)
            .map_err(GameError::Mana)?;
        self.spend_energy(p, energy_cost);
        // Attach.
        self.battlefield[equip_pos].attached_to = Some(target);
        Ok(vec![GameEvent::AttachmentMoved {
            attachment: equipment,
            attached_to: Some(target),
        }])
    }

    /// CR 702.151 — Reconfigure. Pay the reconfigure cost to attach the
    /// Equipment-creature to a creature you control (`Some`), or to unattach
    /// it (`None`). Attach reuses the equip-legality checks; unattach simply
    /// clears the link, restoring its creature-ness (the layer-4
    /// "not a creature while attached" strip keys on `attached_to`).
    /// Sorcery-speed only (CR 702.151c).
    fn reconfigure(
        &mut self,
        equipment: crate::card::CardId,
        target: Option<crate::card::CardId>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == equipment)
            .ok_or(GameError::CardNotOnBattlefield(equipment))?;
        if self.battlefield[pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let cost = self.battlefield[pos]
            .definition
            .has_reconfigure()
            .cloned()
            .ok_or(GameError::NotEquipment(equipment))?;
        match target {
            Some(t) => {
                // Attach: target must be a creature you control (and not the
                // Equipment itself), honoring protection (CR 702.16f).
                let target_ok = t != equipment
                    && self.computed_permanent(t).is_some_and(|c| {
                        c.controller == p
                            && c.card_types.contains(&crate::card::CardType::Creature)
                    });
                if !target_ok {
                    return Err(GameError::InvalidTarget);
                }
                if self.is_protected_from(equipment, t) {
                    return Err(GameError::TargetHasProtection(t));
                }
                self.players[p].mana_pool.pay(&cost).map_err(GameError::Mana)?;
                self.battlefield[pos].attached_to = Some(t);
                Ok(vec![GameEvent::AttachmentMoved {
                    attachment: equipment,
                    attached_to: Some(t),
                }])
            }
            None => {
                // Unattach: only meaningful if currently attached.
                if self.battlefield[pos].attached_to.is_none() {
                    return Err(GameError::InvalidTarget);
                }
                self.players[p].mana_pool.pay(&cost).map_err(GameError::Mana)?;
                self.battlefield[pos].attached_to = None;
                Ok(vec![GameEvent::AttachmentMoved {
                    attachment: equipment,
                    attached_to: None,
                }])
            }
        }
    }

    /// CR 702.122 — Crew a Vehicle. Taps the listed creatures (each an
    /// untapped creature the activator controls, none being the Vehicle
    /// itself) whose total power must meet or exceed the Vehicle's crew
    /// number. On success, registers an `UntilEndOfTurn` layer-4
    /// `AddCardType(Creature)` continuous effect so the Vehicle is an
    /// artifact creature for the rest of the turn (its printed P/T comes
    /// through the layer system via `base_power`/`base_toughness`). Crew is
    /// usable at instant speed (CR 702.122c), so there's no sorcery-speed
    /// gate. Re-crewing an already-crewed Vehicle is legal but pointless;
    /// the engine still taps the creatures and stacks a redundant effect.
    /// CR 702.122e / 702.171 — sum of "crews/saddles as though its power were
    /// N greater" bonuses applying to `cid` (Cloudspire Captain, Deathless
    /// Pilot). Folded into the crew / saddle power total, not real P/T.
    pub(crate) fn crew_saddle_power_bonus(&self, cid: crate::card::CardId) -> i32 {
        use crate::effect::StaticEffect;
        let Some(target) = self.battlefield.iter().find(|c| c.id == cid) else { return 0 };
        let mut bonus = 0;
        for src in &self.battlefield {
            for sa in &src.definition.static_abilities {
                if let StaticEffect::CrewSaddlePowerBonus { applies_to, amount } = &sa.effect
                    && let Some(affected) = selector_to_affected(applies_to, src)
                    && crate::game::layers::affected_includes(&affected, src.id, target)
                {
                    bonus += amount;
                }
            }
        }
        bonus
    }

    /// CR 702.122 / 702.171 — Interface Ace: a crewing/saddling creature that
    /// counts its toughness instead of its power (`SelfCrewsSaddlesWithToughness`).
    pub(crate) fn crew_saddle_uses_toughness(&self, cid: crate::card::CardId) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield
            .iter()
            .find(|c| c.id == cid)
            .is_some_and(|c| {
                c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::SelfCrewsSaddlesWithToughness))
            })
    }

    fn crew(
        &mut self,
        vehicle: crate::card::CardId,
        crew_creatures: &[crate::card::CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let veh_pos = self
            .battlefield
            .iter()
            .position(|c| c.id == vehicle)
            .ok_or(GameError::CardNotOnBattlefield(vehicle))?;
        if self.battlefield[veh_pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let crew_n = self.battlefield[veh_pos]
            .definition
            .crew_cost()
            .ok_or(GameError::InvalidTarget)?;
        // Validate the crew: distinct, controlled by p, untapped creatures,
        // none being the Vehicle itself. Sum their computed power.
        let computed = self.compute_battlefield();
        let mut seen = std::collections::HashSet::new();
        let mut total_power: i32 = 0;
        for &cid in crew_creatures {
            if cid == vehicle || !seen.insert(cid) {
                return Err(GameError::InvalidTarget);
            }
            let Some(cp) = computed.iter().find(|c| c.id == cid) else {
                return Err(GameError::CardNotOnBattlefield(cid));
            };
            if cp.controller != p || !cp.card_types.contains(&crate::card::CardType::Creature) {
                return Err(GameError::InvalidTarget);
            }
            let tapped = self
                .battlefield
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.tapped)
                .unwrap_or(true);
            if tapped {
                return Err(GameError::CardIsTapped(cid));
            }
            let base = if self.crew_saddle_uses_toughness(cid) { cp.toughness } else { cp.power };
            total_power += (base + self.crew_saddle_power_bonus(cid)).max(0);
        }
        if (total_power as u32) < crew_n {
            return Err(GameError::SelectionRequirementViolated);
        }
        // Tap the crew.
        let mut events = vec![];
        for &cid in crew_creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                c.tapped = true;
                events.push(GameEvent::PermanentTapped { card_id: cid, actor: None });
            }
        }
        // Animate the Vehicle until end of turn.
        let ts = self.next_timestamp();
        self.add_continuous_effect(crate::game::layers::ContinuousEffect {
            timestamp: ts,
            source: vehicle,
            affected: crate::game::layers::AffectedPermanents::Source,
            layer: crate::game::layers::Layer::L4Type,
            sublayer: None,
            duration: crate::game::layers::EffectDuration::UntilEndOfTurn,
            modification: crate::game::layers::Modification::AddCardType(
                crate::card::CardType::Creature,
            ),
        });
        events.push(GameEvent::VehicleCrewed { vehicle, crew: crew_creatures.to_vec() });
        Ok(events)
    }

    /// CR 702.171 — Saddle a Mount. Taps the listed other untapped creatures
    /// the activator controls (total power ≥ the Mount's saddle number) and
    /// marks the Mount saddled until end of turn. Sorcery speed (CR 702.171a).
    fn saddle(
        &mut self,
        mount: crate::card::CardId,
        creatures: &[crate::card::CardId],
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        if !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        let mount_pos = self
            .battlefield
            .iter()
            .position(|c| c.id == mount)
            .ok_or(GameError::CardNotOnBattlefield(mount))?;
        if self.battlefield[mount_pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        let saddle_n = self.battlefield[mount_pos]
            .definition
            .saddle_cost()
            .ok_or(GameError::InvalidTarget)?;
        let computed = self.compute_battlefield();
        let mut seen = std::collections::HashSet::new();
        let mut total_power: i32 = 0;
        for &cid in creatures {
            if cid == mount || !seen.insert(cid) {
                return Err(GameError::InvalidTarget);
            }
            let Some(cp) = computed.iter().find(|c| c.id == cid) else {
                return Err(GameError::CardNotOnBattlefield(cid));
            };
            if cp.controller != p || !cp.card_types.contains(&crate::card::CardType::Creature) {
                return Err(GameError::InvalidTarget);
            }
            let tapped = self
                .battlefield
                .iter()
                .find(|c| c.id == cid)
                .map(|c| c.tapped)
                .unwrap_or(true);
            if tapped {
                return Err(GameError::CardIsTapped(cid));
            }
            let base = if self.crew_saddle_uses_toughness(cid) { cp.toughness } else { cp.power };
            total_power += (base + self.crew_saddle_power_bonus(cid)).max(0);
        }
        if (total_power as u32) < saddle_n {
            return Err(GameError::SelectionRequirementViolated);
        }
        let mut events = vec![];
        for &cid in creatures {
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == cid) {
                c.tapped = true;
                events.push(GameEvent::PermanentTapped { card_id: cid, actor: None });
            }
        }
        if let Some(m) = self.battlefield.iter_mut().find(|c| c.id == mount) {
            m.saddled = true;
        }
        events.push(GameEvent::MountSaddled { mount, riders: creatures.to_vec() });
        Ok(events)
    }

    /// CR 702.49 — Ninjutsu. During the declare-blockers step, the active
    /// player returns an unblocked attacker (`returning`) to hand and puts
    /// `ninja` from hand onto the battlefield tapped and attacking the same
    /// defender, paying the ninjutsu cost.
    fn ninjutsu(
        &mut self,
        ninja: crate::card::CardId,
        returning: crate::card::CardId,
    ) -> Result<Vec<GameEvent>, GameError> {
        use crate::card::Keyword;
        if self.step != crate::TurnStep::DeclareBlockers {
            return Err(GameError::WrongStep { actual: self.step });
        }
        let p = self.player_with_priority();
        // The returning creature must be one of this player's unblocked
        // attackers (not a value in `block_map`).
        let Some(atk) = self.attack_for(returning).copied() else {
            return Err(GameError::InvalidTarget);
        };
        let returning_controller = self
            .battlefield
            .iter()
            .find(|c| c.id == returning)
            .map(|c| c.controller);
        if returning_controller != Some(p) {
            return Err(GameError::NotYourPriority);
        }
        // CR 702.49a — "unblocked attacker": once blocked it stays blocked
        // for the combat even if its blockers have since left (CR 510.1c).
        if self.block_map.values().any(|&a| a == returning)
            || self.blocked_attackers.contains(&returning)
        {
            return Err(GameError::InvalidTarget); // blocked — illegal
        }
        // The ninja must be in `p`'s hand and carry Ninjutsu; clone its cost.
        let cost = self.players[p]
            .hand
            .iter()
            .find(|c| c.id == ninja)
            .and_then(|c| {
                c.definition.keywords.iter().find_map(|kw| match kw {
                    Keyword::Ninjutsu(mc) => Some(mc.clone()),
                    _ => None,
                })
            })
            .ok_or(GameError::CardNotInHand(ninja))?;
        self.players[p].mana_pool.pay(&cost).map_err(GameError::Mana)?;

        let mut events = vec![];
        // Return the unblocked attacker to its owner's hand (this prunes it
        // from `attacking` via `remove_from_combat` inside `move_card_to`).
        let owner = self.find_card_owner(returning).unwrap_or(p);
        let ctx = crate::game::effects::EffectContext::for_trigger(returning, p, None, 0);
        self.move_card_to(
            returning,
            &crate::effect::ZoneDest::Hand(crate::effect::PlayerRef::Seat(owner)),
            &ctx,
            &mut events,
        );
        // Put the ninja onto the battlefield tapped (ETB fires here).
        let ninja_ctx = crate::game::effects::EffectContext::for_trigger(ninja, p, None, 0);
        self.move_card_to(
            ninja,
            &crate::effect::ZoneDest::Battlefield {
                controller: crate::effect::PlayerRef::Seat(p),
                tapped: true,
            },
            &ninja_ctx,
            &mut events,
        );
        // It enters attacking the same defender the returned creature was
        // attacking — bypassing the declare-attackers timing/sickness gates.
        if self.battlefield.iter().any(|c| c.id == ninja) {
            self.attacking.push(Attack { attacker: ninja, target: atk.target });
            if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == ninja) {
                c.attacked_this_turn = true;
            }
            events.push(GameEvent::AttackerDeclared(ninja));
        }
        Ok(events)
    }

    /// Walk the battlefield looking for triggered abilities whose `EventSpec`
    /// matches any of `events`, and push matching triggers onto the stack.
    ///
    /// Events already handled by hardcoded trigger sites (ETB, attackers,
    /// spell-cast, dies, step changes) are skipped here to avoid double-firing.
    /// Everything else (TurnBegins, CardDrawn, LandPlayed, LifeGained, etc.)
    /// gains trigger capability through this path.
    /// CR 702.95 — when `entered` (a creature) comes onto the battlefield,
    /// pair it with an eligible unpaired creature its controller controls. The
    /// "may" is auto-resolved (pairing is value-positive); the partner with
    /// the lowest CardId is chosen for determinism. A Soulbond creature can
    /// initiate the pair; a non-Soulbond creature only pairs if its controller
    /// already has an unpaired Soulbond creature waiting.
    pub(crate) fn apply_soulbond_pairing(&mut self, entered: CardId) {
        use crate::card::Keyword;
        let Some(card) = self.battlefield_find(entered) else { return };
        if !card.definition.is_creature() || card.soulbond_partner.is_some() {
            return;
        }
        let controller = card.controller;
        let entered_has_soulbond = card.definition.keywords.contains(&Keyword::Soulbond);
        let partner = self
            .battlefield
            .iter()
            .filter(|c| {
                c.id != entered
                    && c.controller == controller
                    && c.definition.is_creature()
                    && c.soulbond_partner.is_none()
                    && (entered_has_soulbond
                        || c.definition.keywords.contains(&Keyword::Soulbond))
            })
            .map(|c| c.id)
            .min_by_key(|id| id.0);
        let Some(p) = partner else { return };
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == entered) {
            c.soulbond_partner = Some(p);
        }
        if let Some(c) = self.battlefield.iter_mut().find(|c| c.id == p) {
            c.soulbond_partner = Some(entered);
        }
    }

    pub(crate) fn dispatch_triggers_for_events(&mut self, events: &[GameEvent]) {
        // Cost-payment events (paid life) queued since the last dispatch —
        // fold them in so resumed-decision paths that bypass
        // `perform_action`'s drain still fire their triggers.
        if !self.pending_cost_events.is_empty() {
            let pending = std::mem::take(&mut self.pending_cost_events);
            self.dispatch_triggers_for_events(&pending);
        }
        // CR 700.4 — fold in `PermanentDied` events synthesized from the deaths
        // recorded at the raw removal chokepoint since the last dispatch, so
        // non-creature deaths (which emit no `CreatureDied`) still reach
        // "creature or artifact you control dies" triggers.
        let deaths = std::mem::take(&mut self.pending_permanent_deaths);
        let synthesized: Vec<GameEvent> = deaths
            .into_iter()
            // CR 700.4 — a death redirected away from the graveyard (Rest in
            // Peace, void counters, Kalitas, Pulmonic Sliver) never happened;
            // skip it, mirroring the `CreatureDied` guard below.
            .filter(|(card_id, ..)| !self.death_was_replaced(*card_id))
            .map(|(card_id, controller, is_creature, is_artifact)| GameEvent::PermanentDied {
                card_id,
                controller,
                is_creature,
                is_artifact,
            })
            .collect();
        let folded: Vec<GameEvent>;
        let events: &[GameEvent] = if synthesized.is_empty() {
            events
        } else {
            folded = events.iter().cloned().chain(synthesized).collect();
            &folded
        };
        if events.is_empty() {
            return;
        }
        // CR 702.95 — Soulbond pairing. When a creature enters, attempt to pair
        // it (auto-resolved "may"). Done before trigger dispatch so a paired
        // creature's bonus is live for any subsequent ETB-trigger evaluation.
        // CR 603.4 — stamp the entry turn on every permanent that entered in
        // this batch, so `SelectionRequirement::EnteredThisTurn` (Shaile) can
        // compare against the current turn. Centralized here because every
        // battlefield-entry path emits a `PermanentEntered` event.
        let turn = self.turn_number;
        for e in events {
            match e {
                GameEvent::PermanentEntered { card_id } => {
                    // CR 613.7d — the new object's timestamp is its entry
                    // time, drawn from the same counter as resolved-effect
                    // timestamps so static-vs-spell ordering is coherent.
                    let ts = self.next_timestamp();
                    if let Some(c) = self.battlefield_find_mut(*card_id) {
                        c.entered_turn = Some(turn);
                        c.battlefield_timestamp = ts;
                    }
                    self.apply_soulbond_pairing(*card_id);
                }
                // CR 613.7e/f/g — attach, turn face up, and transform each
                // give the object a new timestamp.
                GameEvent::AttachmentMoved { attachment, attached_to: Some(_) } => {
                    let ts = self.next_timestamp();
                    if let Some(c) = self.battlefield_find_mut(*attachment) {
                        c.battlefield_timestamp = ts;
                    }
                }
                GameEvent::Transformed { card_id } | GameEvent::TurnedFaceUp { card_id } => {
                    let ts = self.next_timestamp();
                    if let Some(c) = self.battlefield_find_mut(*card_id) {
                        c.battlefield_timestamp = ts;
                    }
                }
                // Per-turn sacrifice tally — every sacrifice path funnels a
                // `PermanentSacrificed` through here exactly once, so this is
                // the one place to count "you sacrificed a permanent this turn".
                GameEvent::PermanentSacrificed { who, .. } => {
                    if let Some(pl) = self.players.get_mut(*who) {
                        pl.permanents_sacrificed_this_turn =
                            pl.permanents_sacrificed_this_turn.saturating_add(1);
                    }
                }
                _ => {}
            }
        }
        // Event-keyed delayed triggers ("when [card] dies this turn, …").
        // Fire any `WhenCardDies(cid)` whose watched card appears in a
        // `CreatureDied` event in this batch, with its captured target.
        // `PermanentDied` covers non-creature deaths (a watched artifact —
        // Melira's return rider); a creature death lists its id twice, which
        // is harmless since a fired watcher is removed.
        let died: Vec<CardId> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::CreatureDied { card_id }
                | GameEvent::PermanentDied { card_id, .. } => Some(*card_id),
                _ => None,
            })
            // CR 700.4 — a redirected death (exile / library-top) never
            // happened; "when [card] dies" watchers keep watching.
            .filter(|card_id| !self.death_was_replaced(*card_id))
            .collect();
        if !died.is_empty() {
            use crate::game::types::DelayedKind;
            let mut fire: Vec<crate::game::types::DelayedTrigger> = Vec::new();
            let mut watched: Vec<CardId> = Vec::new();
            self.delayed_triggers.retain(|dt| {
                let watched_id = match dt.kind {
                    // CR 702.55 — Haunt's death-watch fires any turn.
                    DelayedKind::WhenCardDies(cid)
                    | DelayedKind::WhenHauntedCreatureDies(cid) => Some(cid),
                    _ => None,
                };
                if let Some(cid) = watched_id
                    && died.contains(&cid)
                {
                    fire.push(dt.clone());
                    watched.push(cid);
                    false
                } else {
                    true
                }
            });
            for (dt, cid) in fire.into_iter().zip(watched) {
                // Expose the dead creature as the trigger's source so bodies
                // can reference it (e.g. "exile it") via `Selector::This` /
                // `TriggerSource`; `target` still carries its controller.
                // Carry the dead creature's mana value as the event amount so
                // `ManaValueLessThanEventAmount` filters (Rushed Rebirth's
                // "creature card with lesser mana value") read it at
                // resolution.
                let mv = self
                    .find_card_anywhere(cid)
                    .map(|c| c.definition.cost.cmc())
                    .unwrap_or(0);
                self.stack.push(
                    TriggerPush::new(dt.source, dt.controller, dt.effect)
                        .target(dt.target)
                        .trigger_source(Some(crate::game::effects::EntityRef::Card(cid)))
                        .event_amount(mv)
                        .build(),
                );
            }
        }
        // "Whenever a creature attacks you or a planeswalker you control"
        // floating triggers (Tamiyo +2). Fire once per qualifying attacker;
        // the attacker is the trigger source.
        let attackers: Vec<CardId> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::AttackerDeclared(id) => Some(*id),
                _ => None,
            })
            .collect();
        if !attackers.is_empty() {
            use crate::game::types::DelayedKind;
            let watchers: Vec<crate::game::types::DelayedTrigger> = self
                .delayed_triggers
                .iter()
                .filter(|dt| {
                    matches!(dt.kind, DelayedKind::CreatureAttacksYouUntilYourNextTurn)
                })
                .cloned()
                .collect();
            for dt in watchers {
                for &atk_id in &attackers {
                    let defender = self
                        .attack_for(atk_id)
                        .and_then(|a| self.defender_for(a.target));
                    if defender == Some(dt.controller) {
                        self.stack.push(
                            TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                                .trigger_source(Some(
                            crate::game::effects::EntityRef::Permanent(atk_id),
                        ))
                                .build(),
                        );
                    }
                }
            }
            // "Until end of turn, whenever a [filter] creature attacks, …"
            // floating triggers (Summon: Leviathan II/III). Any player's
            // qualifying attacker fires the registering controller's body.
            let matching_watchers: Vec<crate::game::types::DelayedTrigger> = self
                .delayed_triggers
                .iter()
                .filter(|dt| {
                    matches!(dt.kind, DelayedKind::MatchingCreatureAttacksThisTurn(_))
                })
                .cloned()
                .collect();
            for dt in matching_watchers {
                let DelayedKind::MatchingCreatureAttacksThisTurn(ref filt) = dt.kind else {
                    continue;
                };
                for &atk_id in &attackers {
                    let matches = self
                        .battlefield_find(atk_id)
                        .is_some_and(|c| self.evaluate_requirement_on_card(filt, c, dt.controller));
                    if matches {
                        self.stack.push(
                            TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                                .trigger_source(Some(
                                    crate::game::effects::EntityRef::Permanent(atk_id),
                                ))
                                .build(),
                        );
                    }
                }
            }
        }
        // Turn-scoped "whenever a creature you control enters this turn"
        // delayed triggers (CR 603.4 — First Day of Class). Fire once per
        // entering creature controlled by the trigger's controller; the
        // entering creature is the trigger source. These persist (not
        // fires_once) until cleanup.
        let entered_creatures: Vec<(CardId, usize)> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::PermanentEntered { card_id } => self
                    .battlefield_find(*card_id)
                    .filter(|c| c.definition.is_creature())
                    .map(|c| (*card_id, c.controller)),
                _ => None,
            })
            .collect();
        if !entered_creatures.is_empty() {
            use crate::game::types::DelayedKind;
            let watchers: Vec<crate::game::types::DelayedTrigger> = self
                .delayed_triggers
                .iter()
                .filter(|dt| {
                    matches!(dt.kind, DelayedKind::CreatureYouControlEntersThisTurn)
                })
                .cloned()
                .collect();
            for (cid, controller) in &entered_creatures {
                for dt in &watchers {
                    if dt.controller != *controller {
                        continue;
                    }
                    self.stack.push(
                        TriggerPush::new(dt.source, dt.controller, dt.effect.clone())
                            .trigger_source(Some(crate::game::effects::EntityRef::Permanent(*cid)))
                            .build(),
                    );
                }
            }
        }
        // Phase 1: collect candidate triggers while the borrow on
        // `self.battlefield` is shared. Phase 2 will mutate `self.stack`
        // and call `&self.evaluate_predicate` to gate each candidate by
        // the optional `EventSpec::filter`.
        let mut candidates: Vec<TriggerCandidate> = Vec::new();
        // Hushbringer (CR 614): suppress reaction creature-death triggers
        // ("whenever a creature dies") while a `SuppressCreatureEtbTriggers
        // { also_dies }` static is in play. (Self-death + SBA paths gate
        // separately in `stack.rs`.)
        let dies_suppressed = crate::game::actions::creature_dies_triggers_suppressed(self);
        // Resolve per-permanent layer state once so the dispatcher can
        // honour `Modification::RemoveAllAbilities` (Turn to Frog,
        // Mercurial Transformation, Lignify) — printed triggered abilities
        // are skipped while a strip-abilities effect is in scope per CR
        // 113.10b.
        let computed = self.compute_battlefield();
        // CR 603.3d — keys for `once_per_turn` triggers that fire in this
        // batch; merged into the turn-scoped set after the battlefield walk
        // (deferred so we don't mutate `self` mid-immutable-borrow).
        let mut once_fired_this_batch: std::collections::HashSet<(CardId, usize)> =
            std::collections::HashSet::new();
        // `EventSpec::per_subject_cap` fires in this batch (deferred merge).
        let mut capped_fired_this_batch: Vec<(CardId, CardId)> = Vec::new();
        for card in &self.battlefield {
            let stripped = computed
                .iter()
                .find(|c| c.id == card.id)
                .map(|c| c.lost_all_abilities)
                .unwrap_or(false);
            if stripped {
                continue;
            }
            // Walk printed triggered abilities AND any transient
            // granted_triggers_eot for this permanent (Root Manipulation,
            // Rabid Attack-style "creatures gain '…trigger…' EOT"). Printed
            // triggers carry their definition index so `once_per_turn`
            // (CR 603.3d) can be tracked per (source, index); granted
            // triggers are never once-per-turn and use a sentinel index.
            let n_printed = card.definition.triggered_abilities.len();
            let static_granted = self.statics_granted_triggers_for(card);
            let equip_granted = self.equip_granted_triggers_for(card);
            let all_triggers = card
                .definition
                .triggered_abilities
                .iter()
                .enumerate()
                .chain(self.granted_triggers(card.id).iter().map(|t| (usize::MAX, t)))
                .chain(static_granted.iter().map(|t| (usize::MAX, t)))
                .chain(equip_granted.iter().map(|t| (usize::MAX, t)));
            for (trig_idx, ta) in all_triggers {
                // A `FromYourGraveyard`-scoped trigger functions ONLY while
                // its card is in a graveyard (CR 603.3d zone-scoping —
                // Bloodghast, Voidwing Hybrid); the graveyard walk below
                // gathers those. Skip them on the battlefield.
                if matches!(ta.event.scope, crate::effect::EventScope::FromYourGraveyard) {
                    continue;
                }
                // CR 603.3d — "triggers only once each turn": skip if it has
                // already fired this turn or earlier in this same batch.
                let once_key = (card.id, trig_idx);
                if ta.event.once_per_turn
                    && trig_idx < n_printed
                    && (self.triggered_once_per_turn_used.contains(&once_key)
                        || once_fired_this_batch.contains(&once_key))
                {
                    continue;
                }
                // For batch-fanout-friendly event kinds (Attacks,
                // CreatureDied, CardDrawn, CardDiscarded, CardLeftGraveyard,
                // CounterAdded, BlockerDeclared, AttackerWentUnblocked,
                // CardMilled, LifeGained, LifeLost) the trigger fires
                // ONCE PER MATCHING EVENT — CR 603.6 "whenever X happens"
                // fan-out. For other event kinds (ETB, StepBegins, …) we
                // fire at most once per (source, trigger) pair because
                // they don't naturally produce duplicate events in a
                // single batch.
                let fanout = matches!(
                    ta.event.kind,
                    crate::effect::EventKind::Attacks
                        | crate::effect::EventKind::CreatureDied
                        | crate::effect::EventKind::CreatureOrArtifactDied
                        | crate::effect::EventKind::PermanentDied
                        | crate::effect::EventKind::CreatureSacrificed
                        | crate::effect::EventKind::PermanentSacrificed
                        | crate::effect::EventKind::PermanentLeavesBattlefield
                        | crate::effect::EventKind::CardDrawn
                        | crate::effect::EventKind::CardDiscarded
                        | crate::effect::EventKind::CardLeftGraveyard
                        | crate::effect::EventKind::CounterAdded(_)
                        | crate::effect::EventKind::AnyCounterAdded
                        | crate::effect::EventKind::Blocks
                        | crate::effect::EventKind::BecomesBlocked
                        | crate::effect::EventKind::AttacksAndIsntBlocked
                        | crate::effect::EventKind::LifeGained
                        | crate::effect::EventKind::LifeLost
                        | crate::effect::EventKind::EnergyGained
                        | crate::effect::EventKind::WonCoinFlip
                        | crate::effect::EventKind::LostCoinFlip
                        | crate::effect::EventKind::RolledDice
                        | crate::effect::EventKind::BecameTarget
                        // Enrage fires once per instance of damage
                        // (CR 702.130a) — fan out across the batch.
                        | crate::effect::EventKind::DealtDamage
                        // A Tekuthal-doubled proliferate emits two events in
                        // one batch; payoffs fire once per proliferation.
                        | crate::effect::EventKind::Proliferated
                );
                // "Only once each turn" overrides fan-out: a single batch of
                // simultaneous events mints one trigger, not one per event.
                let fanout = fanout && !ta.event.once_per_turn;
                for ev in events {
                    if is_event_hardcoded(ev, &ta.event) {
                        continue;
                    }
                    if dies_suppressed && matches!(ev, GameEvent::CreatureDied { .. }) {
                        continue;
                    }
                    // CR 700.4 — a creature whose death-placement was
                    // redirected away from the graveyard (Rest in Peace, void
                    // counters, Kalitas exile; Pulmonic Sliver library-top)
                    // never died; "whenever a creature dies" watchers don't
                    // fire. The redirected card sits in exile or a library at
                    // dispatch time.
                    if let GameEvent::CreatureDied { card_id } = ev
                        && self.death_was_replaced(*card_id)
                    {
                        continue;
                    }
                    if crate::game::effects::event_matches_spec(self, ev, &ta.event, card) {
                        let subject = crate::game::effects::event_subject(ev, &ta.event.kind);
                        // Evaluate the trigger's intervening filter here, before
                        // consuming any once-per-turn / per-subject budget: a
                        // candidate whose filter fails must not "use up" the
                        // once-per-turn slot (CR 603.4 — a trigger that doesn't
                        // meet its condition simply doesn't trigger). The same
                        // filter is re-checked in `drain_trigger_queue`; this
                        // pre-check just gates the budget bookkeeping. Powers
                        // Faerie Mastermind's "second card each turn" payoff.
                        if let Some(filter) = &ta.event.filter {
                            let ctx = crate::game::effects::EffectContext {
                                controller: card.controller,
                                source: Some(card.id),
                                targets: vec![],
                                trigger_source: subject,
                                mode: 0,
                                x_value: 0,
                                converged_value: 0,
                                mana_spent: 0,
                                mana_spent_by_color: Vec::new(),
                                source_name: None,
                                cast_from_hand: true,
                                event_amount: self.event_amount_for(ev),
                                kicked: false,
                                bargained: false,
                                cast_via_mayhem: false,
                                cast_via_waterbend: false,
                                entwined: false,
                                spree_modes: Vec::new(),
                            };
                            if !self.evaluate_predicate(filter, &ctx) {
                                if !fanout {
                                    break;
                                }
                                continue;
                            }
                        }
                        // Per-subject cap ("triggers only twice each turn"
                        // counted per creature — Nadu). Deferred bump (the
                        // battlefield is immutably borrowed here).
                        if let (Some(cap), Some(crate::game::effects::EntityRef::Permanent(sid))) =
                            (ta.event.per_subject_cap, subject)
                        {
                            let key = (card.id, sid);
                            let used = self.per_subject_trigger_uses.get(&key).copied().unwrap_or(0)
                                + capped_fired_this_batch.iter().filter(|k| **k == key).count() as u8;
                            if used >= cap {
                                continue;
                            }
                            capped_fired_this_batch.push(key);
                        }
                        candidates.push(TriggerCandidate {
                            source: card.id,
                            effect: ta.effect.clone(),
                            controller: card.controller,
                            filter: ta.event.filter.clone(),
                            subject,
                            event_amount: self.event_amount_for(ev),
                            triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                            triggered_by_death: matches!(
                                ev,
                                GameEvent::CreatureDied { .. }
                                    | GameEvent::CreatureSacrificed { .. }
                            ),
                        });
                        if ta.event.once_per_turn && trig_idx < n_printed {
                            once_fired_this_batch.insert(once_key);
                        }
                        if !fanout {
                            break;
                        }
                    }
                }
            }
        }
        for key in once_fired_this_batch.drain() {
            self.triggered_once_per_turn_used.insert(key);
        }
        for key in capped_fired_this_batch {
            *self.per_subject_trigger_uses.entry(key).or_insert(0) += 1;
        }
        // CR 702.130a / 603.10a — Enrage on lethal damage. A creature that
        // dies from the same damage that would trigger its "whenever this is
        // dealt damage" ability still triggers (the ability uses last-known
        // information). Such a creature is no longer on the battlefield by
        // dispatch time, so walk the just-died snapshots for SelfSource
        // `DealtDamage` triggers matching a `DamageDealt` event in this batch.
        // (Other SelfSource trigger kinds — die/leave — are handled via their
        // own dedicated paths, so this is scoped to DealtDamage only.)
        for snap in self.died_card_snapshots.values() {
            for ta in &snap.definition.triggered_abilities {
                // SelfSource `DealtDamage` (Enrage on lethal damage) and
                // `PermanentSacrificed` ("when you sacrifice this") both fire
                // from LKI — the source has left the battlefield by dispatch.
                let lki_self = matches!(
                    ta.event.kind,
                    crate::effect::EventKind::DealtDamage
                        | crate::effect::EventKind::PermanentSacrificed
                ) && ta.event.scope == crate::effect::EventScope::SelfSource;
                // "When enchanted creature dies" on a leaving Aura (Minion's
                // Return) — the snapshot is the orphaned Aura, scope keys on
                // the dead host being recorded in `auras_at_death`.
                let lki_enchanted = ta.event.kind == crate::effect::EventKind::CreatureDied
                    && ta.event.scope == crate::effect::EventScope::EnchantedBySource;
                if !(lki_self || lki_enchanted) {
                    continue;
                }
                for ev in events {
                    if crate::game::effects::event_matches_spec(self, ev, &ta.event, snap) {
                        candidates.push(TriggerCandidate {
                            source: snap.id,
                            effect: ta.effect.clone(),
                            controller: snap.controller,
                            filter: ta.event.filter.clone(),
                            subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                            event_amount: self.event_amount_for(ev),
                            triggered_by_etb: false,
                            triggered_by_death: false,
                        });
                    }
                }
            }
        }
        // Also walk every player's graveyard for triggers scoped
        // `FromYourGraveyard` — recursion creatures (Bloodghast,
        // Ichorid, Silversmote Ghoul) fire from there. The trigger's
        // effective controller is the card's owner. Per CR 702.29c,
        // SelfSource cycle triggers ("When you cycle this card") also
        // fire here — the cycled card is in graveyard at dispatch
        // time, and the trigger's source matches the cycled card by id.
        for player in &self.players {
            for card in &player.graveyard {
                for ta in &card.definition.triggered_abilities {
                    let from_gy_scope = matches!(
                        ta.event.scope,
                        crate::effect::EventScope::FromYourGraveyard
                    );
                    let self_scope = matches!(
                        ta.event.scope,
                        crate::effect::EventScope::SelfSource
                    );
                    // CR 702.29c cycle triggers and "when this card is
                    // milled" triggers both fire off the card in the
                    // graveyard.
                    let cycle_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::CardCycled
                    ) && self_scope;
                    let milled_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::CardMilled
                    ) && self_scope;
                    // "When this is put into a graveyard from anywhere"
                    // (Emrakul) — also fires off the card in the graveyard.
                    let putgy_self = matches!(
                        ta.event.kind,
                        crate::effect::EventKind::PutIntoGraveyard
                    ) && self_scope;
                    if !from_gy_scope && !cycle_self && !milled_self && !putgy_self {
                        continue;
                    }
                    for ev in events {
                        if is_event_hardcoded(ev, &ta.event) {
                            continue;
                        }
                        if crate::game::effects::event_matches_spec(self, ev, &ta.event, card) {
                            candidates.push(TriggerCandidate {
                                source: card.id,
                                effect: ta.effect.clone(),
                                controller: card.owner,
                                filter: ta.event.filter.clone(),
                                subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                                event_amount: self.event_amount_for(ev),
                                triggered_by_etb: matches!(ev, GameEvent::PermanentEntered { .. }),
                            triggered_by_death: matches!(
                                ev,
                                GameEvent::CreatureDied { .. }
                                    | GameEvent::CreatureSacrificed { .. }
                            ),
                            });
                            break;
                        }
                    }
                }
            }
        }
        // Player-level emblems (CR 114). Each player's emblems carry
        // triggered abilities that fire from the command zone alongside
        // battlefield permanents. Event-keyed emblem triggers are handled
        // here (step-keyed ones — "at the beginning of your upkeep" — fire
        // in `fire_step_triggers`). `event_amount` carries the magnitude
        // through to the body via `Value::TriggerEventAmount`. Professor
        // Dellian Fel's -6 emblem ("Whenever you gain life, each opponent
        // loses that much life") rides this path.
        for seat_idx in 0..self.players.len() {
            for em_idx in 0..self.players[seat_idx].emblems.len() {
                let triggers = self.players[seat_idx].emblems[em_idx].triggered.clone();
                for ta in &triggers {
                    if matches!(
                        ta.event.kind,
                        crate::effect::EventKind::StepBegins(_) | crate::effect::EventKind::TurnBegins
                    ) {
                        continue;
                    }
                    for ev in events {
                        if crate::game::effects::emblem_event_matches(self, ev, &ta.event, seat_idx) {
                            candidates.push(TriggerCandidate {
                                source: CardId(0),
                                effect: ta.effect.clone(),
                                controller: seat_idx,
                                filter: ta.event.filter.clone(),
                                subject: crate::game::effects::event_subject(ev, &ta.event.kind),
                                event_amount: self.event_amount_for(ev),
                                triggered_by_etb: false,
                            triggered_by_death: false,
                            });
                        }
                    }
                }
            }
        }
        // CR 603.3b — APNAP. When multiple abilities trigger off the same
        // batch of events, the active player puts their triggers on the
        // stack first (in any order they choose), then each non-active
        // player in turn order. Since the stack is LIFO, the active
        // player's triggers resolve LAST. Without this sort, candidates
        // were pushed in battlefield-iteration order, which produced
        // observable wrong orderings the moment more than one player
        // controlled a triggering permanent (acute for 4-player FFA, 2HG,
        // and Commander — invisible in 1v1 where there's only one
        // non-active player). Within a player's group we keep the
        // gathered order: stable sort means each player's
        // battlefield-iteration order is preserved as their chosen
        // order — fine for AutoDecider; a real UI player would pick.
        let n_players = self.players.len();
        let active = self.active_player_idx;
        let apnap_rank = |seat: usize| -> usize {
            if seat == active {
                return 0;
            }
            let mut s = active;
            for r in 1..=n_players {
                s = self.next_alive_seat(s);
                if s == seat {
                    return r;
                }
                if s == active {
                    break;
                }
            }
            // Eliminated / unknown controller: sort to the back so it
            // pushes last → resolves first. Triggers from a dead
            // permanent's owner shouldn't really hit this path, but
            // keep behavior deterministic if they do.
            n_players
        };
        candidates.sort_by_key(|c| apnap_rank(c.controller));

        // Prowess: inject +1/+1 EOT pump for each creature with the
        // Prowess keyword that does NOT already carry its own prowess()
        // triggered ability. Cards wired via shortcut::prowess() already
        // have a SpellCast trigger on their definition; we skip those to
        // avoid doubling the pump.
        for ev in events {
            if let GameEvent::SpellCast { player, card_id, .. } = ev {
                let is_creature_spell = self.stack.iter().any(|si| matches!(
                    si,
                    crate::game::types::StackItem::Spell { card, .. } if card.id == *card_id && card.definition.is_creature()
                ));
                if !is_creature_spell {
                    let prowess_ids: Vec<_> = self.battlefield.iter()
                        .filter(|c| {
                            c.controller == *player
                                && c.has_keyword(&Keyword::Prowess)
                                && !c.definition.triggered_abilities.iter().any(|ta| {
                                    matches!(ta.event.kind, crate::effect::EventKind::SpellCast)
                                })
                        })
                        .map(|c| c.id)
                        .collect();
                    for pid in prowess_ids {
                        candidates.push(TriggerCandidate {
                            source: pid,
                            effect: Effect::PumpPT {
                                what: crate::effect::Selector::This,
                                power: crate::effect::Value::Const(1),
                                toughness: crate::effect::Value::Const(1),
                                duration: crate::effect::Duration::EndOfTurn,
                            },
                            controller: *player,
                            filter: None,
                            subject: None,
                            event_amount: 0,
                            triggered_by_etb: false,
                            triggered_by_death: false,
                        });
                    }
                }
            }
        }

        // CR 701.54c — The Ring's bearer-keyed emblem abilities, injected as
        // triggers off the Ring-bearer (the emblem text is applied directly
        // from each player's `ring_temptations` level rather than synthesized
        // as a literal emblem). Level 2+: "Whenever your Ring-bearer attacks,
        // draw a card, then discard a card." Level 3+: "Whenever your
        // Ring-bearer becomes blocked by a creature, the blocking creature's
        // controller sacrifices it at end of combat." Level-4 combat-damage
        // drain rides the dedicated combat-damage path in `combat.rs`.
        let mut ring_blocked_done = vec![false; self.players.len()];
        for ev in events {
            match ev {
                GameEvent::AttackerDeclared(attacker) => {
                    for seat in 0..self.players.len() {
                        if self.players[seat].ring_temptations >= 2
                            && self.effective_ring_bearer(seat) == Some(*attacker)
                        {
                            candidates.push(TriggerCandidate {
                                source: *attacker,
                                effect: Effect::Seq(vec![
                                    Effect::Draw {
                                        who: crate::effect::Selector::You,
                                        amount: crate::effect::Value::Const(1),
                                    },
                                    Effect::Discard {
                                        who: crate::effect::Selector::You,
                                        amount: crate::effect::Value::Const(1),
                                        random: false,
                                    },
                                ]),
                                controller: seat,
                                filter: None,
                                subject: None,
                                event_amount: 0,
                                triggered_by_etb: false,
                            triggered_by_death: false,
                            });
                        }
                    }
                }
                GameEvent::BlockerDeclared { attacker, .. } => {
                    for (seat, done) in ring_blocked_done.iter_mut().enumerate() {
                        if !*done
                            && self.players[seat].ring_temptations >= 3
                            && self.effective_ring_bearer(seat) == Some(*attacker)
                        {
                            *done = true;
                            candidates.push(TriggerCandidate {
                                source: *attacker,
                                effect: Effect::SacrificeAtEndOfCombat {
                                    what: crate::effect::Selector::BlockingCreatures,
                                },
                                controller: seat,
                                filter: None,
                                subject: None,
                                event_amount: 0,
                                triggered_by_etb: false,
                            triggered_by_death: false,
                            });
                        }
                    }
                }
                _ => {}
            }
        }

        // CR 603.3b — let a `wants_ui` controller order their own
        // simultaneous triggers. After the APNAP regroup (stable so the
        // prowess pumps appended above rejoin their controller's run), we
        // walk each contiguous same-controller run of length ≥2 and ask
        // that player's decider for a stack-push order. Gated on `wants_ui`
        // so AutoDecider/bot games (and the bulk of the test suite) are
        // untouched; AutoDecider would keep the default order anyway.
        candidates.sort_by_key(|c| apnap_rank(c.controller));
        // May suspend on a networked controller's `OrderTriggers` pick
        // (CR 603.3b) — the resume path re-enters
        // `push_ordered_trigger_candidates` with the finished order.
        let Some(candidates) = self.continue_trigger_ordering(Vec::new(), candidates) else {
            return;
        };
        self.push_ordered_trigger_candidates(candidates);
    }

    /// Phase 2 of trigger dispatch: enforce each candidate's
    /// `EventSpec::filter`, expand ETB multipliers/taxes, and drain the
    /// resulting queue onto the stack. Split from
    /// `dispatch_triggers_for_events` so the `OrderTriggers` resume path
    /// can re-enter after a networked controller picks their order.
    pub(crate) fn push_ordered_trigger_candidates(
        &mut self,
        candidates: Vec<TriggerCandidate>,
    ) {
        // Phase 2: enforce the optional `EventSpec::filter` predicate now
        // that we're free to call `&self.evaluate_predicate`. The trigger's
        // source permanent is bound as `ctx.source`, and the event's
        // subject (cast spell, dying creature, attacker, etc.) is bound as
        // `Selector::TriggerSource` so filters can reference it.
        // Build the queue of triggers waiting to be pushed onto the
        // stack. `drain_trigger_queue` walks the queue and either
        // pushes each trigger with an auto-picked target, or — when
        // the controller has `wants_ui` and the effect needs a target
        // — suspends on `Decision::ChooseTarget` so the human can
        // pick. Remaining queue items are saved in
        // `ResumeContext::TriggerTargetPick` and drained on answer.
        let mut queue: Vec<PendingTriggerPush> = Vec::new();
        for candidate in candidates {
            let TriggerCandidate {
                source,
                effect,
                controller,
                filter,
                subject,
                event_amount,
                triggered_by_etb,
                triggered_by_death,
            } = candidate;
            if let Some(filter) = filter {
                let ctx = crate::game::effects::EffectContext {
                    controller,
                    source: Some(source),
                    targets: vec![],
                    trigger_source: subject,
                    mode: 0,
                    x_value: 0,
                    converged_value: 0,
                    mana_spent: 0,
                    mana_spent_by_color: Vec::new(),
                    source_name: None,
                    cast_from_hand: true,
                    event_amount,
                    kicked: false,
                    bargained: false,
                    cast_via_mayhem: false,
                    cast_via_waterbend: false,
                    entwined: false,
                    spree_modes: Vec::new(),
                };
                if !self.evaluate_predicate(&filter, &ctx) {
                    continue;
                }
            }
            // CR 700.2b — modal triggered ability mode pick at push-time.
            let mode = self.pick_trigger_mode(&effect, source, controller);
            if triggered_by_etb {
                // Yarok / Elesh Norn replacement (CR 614). A `wants`-side
                // ETB-trigger multiplier scales how many times this
                // reaction trigger fires (0 = suppressed by an opponent's
                // Spotlight, 1 normally, 2+ with a doubler). Self-source ETB
                // triggers go through the hardcoded path in `actions.rs`
                // (also multiplied), so they aren't double-counted here.
                let etb_mult = crate::game::actions::etb_trigger_multiplier(
                    self,
                    controller,
                    subject.as_ref().and_then(|s| s.as_permanent_id()),
                );
                // Katara, the Fearless: an Ally's triggered ability triggers an
                // additional time. Suppressed (etb_mult == 0) stays suppressed.
                let mult = if etb_mult == 0 {
                    0
                } else {
                    etb_mult + crate::game::actions::ally_trigger_extra_fires(self, controller, source)
                };
                for _ in 0..mult {
                    // Strict Proctor's CR 614 tax applies once per fire; a
                    // declined / unpayable tax sacrifices the source and
                    // halts the remaining fires.
                    if !crate::game::actions::apply_etb_trigger_tax(self, source, controller) {
                        break;
                    }
                    queue.push(PendingTriggerPush {
                        source,
                        controller,
                        effect: effect.clone(),
                        subject,
                        event_amount,
                        mode,
                        intervening_if: None,
                    });
                }
            } else {
                // Katara, the Fearless: a non-ETB Ally trigger fires an
                // additional time per Katara the controller controls.
                // Drivnod, Carnage Dominus: a death-caused trigger of a
                // permanent its controller controls fires an additional time
                // per Drivnod.
                let death_extra = if triggered_by_death {
                    self.battlefield
                        .iter()
                        .filter(|c| c.controller == controller)
                        .flat_map(|c| &c.definition.static_abilities)
                        .filter(|sa| {
                            matches!(
                                sa.effect,
                                crate::effect::StaticEffect::DoubleControllerDeathTriggers
                            )
                        })
                        .count()
                } else {
                    0
                };
                let fires = 1
                    + crate::game::actions::ally_trigger_extra_fires(self, controller, source)
                    + death_extra;
                for _ in 0..fires {
                    queue.push(PendingTriggerPush {
                        source,
                        controller,
                        effect: effect.clone(),
                        subject,
                        event_amount,
                        mode,
                        intervening_if: None,
                    });
                }
            }
        }
        self.drain_trigger_queue(queue);
        // Clear the per-die-event snapshot cache
        // after the dispatcher finishes with this event batch. Any
        // subsequent SBA cycle re-populates the entries it needs at
        // that cycle's die-time, so stale entries from prior batches
        // can't leak into later trigger resolution.
        self.died_card_snapshots.clear();
    }

    /// CR 603.3b — reorder each contiguous run of same-controller triggers
    /// per the controller's chosen stack-push order. Only consults the
    /// decider for a `wants_ui` controller whose run has ≥2 triggers; every
    /// other run is returned unchanged (AutoDecider keeps the default order
    /// regardless). The decider's `TriggerOrder(ids)` lists the desired
    /// push order; ids it omits keep their original relative order at the
    /// end, so a partial or empty answer is always legal.
    /// CR 603.3b — walk contiguous same-controller runs of simultaneous
    /// triggers, letting each `wants_ui` controller pick a stack-push order.
    /// Suspends on `Decision::OrderTriggers` (parking progress in
    /// `ResumeContext::TriggerOrder`) and returns `None`; `submit_decision`
    /// re-enters with the answered run applied. Returns the fully ordered
    /// list otherwise. AutoDecider/bot seats keep the default order.
    pub(crate) fn continue_trigger_ordering(
        &mut self,
        mut ordered: Vec<TriggerCandidate>,
        rest: Vec<TriggerCandidate>,
    ) -> Option<Vec<TriggerCandidate>> {
        let mut i = 0;
        while i < rest.len() {
            let ctrl = rest[i].controller;
            let mut j = i + 1;
            while j < rest.len() && rest[j].controller == ctrl {
                j += 1;
            }
            let run = &rest[i..j];
            // A decision already pending (e.g. a racing combat choice) has
            // nowhere to park this batch — keep the default order, matching
            // `drain_trigger_queue`'s behavior.
            if run.len() < 2
                || !self.players.get(ctrl).is_some_and(|p| p.wants_ui)
                || self.pending_decision.is_some()
            {
                ordered.extend_from_slice(run);
            } else {
                let labels: Vec<(CardId, String)> = run
                    .iter()
                    .map(|c| {
                        let name = self
                            .battlefield_find(c.source)
                            .map(|b| b.definition.name.to_string())
                            .unwrap_or_else(|| "Triggered ability".to_string());
                        (c.source, name)
                    })
                    .collect();
                self.pending_decision = Some(PendingDecision {
                    decision: crate::decision::Decision::OrderTriggers {
                        player: ctrl,
                        triggers: labels,
                    },
                    resume: ResumeContext::TriggerOrder {
                        ordered,
                        run: run.to_vec(),
                        rest: rest[j..].to_vec(),
                    },
                });
                return None;
            }
            i = j;
        }
        Some(ordered)
    }

    /// Apply a `TriggerOrder(ids)` answer to `run`: entries named in `ids`
    /// first (in that order), unnamed ones after in original order — a
    /// partial or empty answer is always legal.
    pub(crate) fn apply_trigger_order(
        ordered: &mut Vec<TriggerCandidate>,
        run: Vec<TriggerCandidate>,
        order: Vec<CardId>,
    ) {
        let mut remaining: Vec<Option<TriggerCandidate>> = run.into_iter().map(Some).collect();
        for id in order {
            if let Some(pos) =
                remaining.iter().position(|c| c.as_ref().is_some_and(|c| c.source == id))
            {
                ordered.push(remaining[pos].take().unwrap());
            }
        }
        for slot in remaining.into_iter().flatten() {
            ordered.push(slot);
        }
    }
    /// Walk a queue of pending triggers, pushing each onto the stack.
    /// Suspends on the first trigger whose controller has `wants_ui`
    /// and whose effect needs a target — emits
    /// `Decision::ChooseTarget` and parks the remaining queue in
    /// `ResumeContext::TriggerTargetPick`. The resume path
    /// (`submit_decision`) re-enters this function with the remaining
    /// queue once the user picks.
    pub(crate) fn drain_trigger_queue(&mut self, queue: Vec<PendingTriggerPush>) {
        // Don't stack up multiple pending decisions — if the engine
        // already suspended on something else we can't surface a target
        // picker, so the whole batch falls back to auto-targeting (the
        // triggers still hit the stack; they must not vanish).
        let force_auto = self.pending_decision.is_some();
        // Walk the queue in *forward* (APNAP) order so the active
        // player's triggers push first and resolve last, matching CR
        // 603.3b. Using an iterator lets us collect the unconsumed
        // tail into `remaining` when we suspend mid-batch.
        let mut iter = queue.into_iter();
        // Per-copy target choice for doubled triggers (CR 603.3d): track the
        // targets already auto-picked for each source in this batch so an
        // Elesh-Norn-doubled ETB aims its second copy at a fresh target
        // instead of duplicating (and later fizzling on) the first pick.
        let mut picked_this_batch: Vec<(CardId, CardId)> = Vec::new();
        while let Some(pending) = iter.next() {
            // Event-amount-relative target filters (Scrap Trawler's
            // "lesser mana value than that artifact") read this scratch
            // during legal-target enumeration below.
            self.trigger_event_amount_scratch = pending.event_amount;
            let needs = pending.effect.requires_target();
            let wants_ui = !force_auto
                && self
                    .players
                    .get(pending.controller)
                    .map(|p| p.wants_ui)
                    .unwrap_or(false);
            if needs && wants_ui {
                let legal = self.enumerate_legal_targets(&pending.effect, pending.controller);
                // No legal targets → fall back to auto (which returns
                // None) so the trigger still resolves CR-correctly as
                // a no-op rather than blocking the game on an
                // unanswerable picker.
                if legal.is_empty() {
                    self.push_pending_trigger(pending, None);
                    continue;
                }
                // The targeting cursor can only select players and
                // battlefield permanents. Off-board candidates (graveyard /
                // exile cards) must go through a `ChooseCards` modal —
                // leaving them in a `ChooseTarget` list soft-locks the game
                // on an unanswerable picker (Zealous Lorecaster's ETB).
                let (clickable, offboard): (Vec<Target>, Vec<Target>) =
                    legal.into_iter().partition(|t| match t {
                        Target::Player(_) => true,
                        Target::Permanent(id) => {
                            self.battlefield.iter().any(|c| c.id == *id)
                        }
                    });
                // Modal when the effect genuinely targets an off-board zone
                // ("… card from a graveyard") or nothing on the board is
                // clickable; otherwise cursor over the clickable set only
                // (dropping spurious off-board matches of board-shaped
                // filters like "target permanent").
                let zone_filter = pending
                    .effect
                    .primary_target_filter()
                    .is_some_and(|f| f.mentions_offboard_zone());
                let remaining: Vec<PendingTriggerPush> = iter.collect();
                let source_name = self
                    .find_card_anywhere(pending.source)
                    .map(|c| c.definition.name.to_string())
                    .unwrap_or_default();
                let description = pending.effect.effect_short_text();
                let decision = if !offboard.is_empty() && (zone_filter || clickable.is_empty()) {
                    let candidates: Vec<(CardId, String)> = offboard
                        .iter()
                        .filter_map(|t| match t {
                            Target::Permanent(id) => Some((
                                *id,
                                self.find_card_anywhere(*id)
                                    .map(|c| c.definition.name.to_string())
                                    .unwrap_or_default(),
                            )),
                            Target::Player(_) => None,
                        })
                        .collect();
                    Decision::ChooseCards {
                        source: pending.source,
                        prompt: format!("{source_name}: {description}"),
                        candidates,
                        min: 1,
                        max: 1,
                    }
                } else {
                    Decision::ChooseTarget {
                        source: pending.source,
                        legal: clickable,
                        source_name,
                        description,
                    }
                };
                self.pending_decision = Some(PendingDecision {
                    decision,
                    resume: ResumeContext::TriggerTargetPick {
                        pending,
                        remaining,
                    },
                });
                return;
            }
            // Prefer a non-source target: an "another target creature" trigger
            // (OtherThanSource) must not auto-pick its own source, and even a
            // plain "target creature" trigger reads better picking a different
            // permanent (a self-target trigger uses `Selector::This`, not a
            // target slot). Falls back to the source if it's the only legal pick.
            let mut avoid = vec![pending.source];
            avoid.extend(
                picked_this_batch
                    .iter()
                    .filter(|(src, _)| *src == pending.source)
                    .map(|(_, t)| *t),
            );
            let auto = self.auto_target_for_effect_avoiding_set(
                &pending.effect,
                pending.controller,
                &avoid,
            );
            if let Some(Target::Permanent(tid)) = &auto {
                picked_this_batch.push((pending.source, *tid));
            }
            self.push_pending_trigger(pending, auto);
        }
    }

    /// Push a `PendingTriggerPush` onto the stack with the given
    /// (already-chosen) target. Mirrors the original inline push at
    /// the trigger-dispatch site.
    pub(crate) fn push_pending_trigger(
        &mut self,
        pending: PendingTriggerPush,
        target: Option<Target>,
    ) {
        let PendingTriggerPush {
            source,
            controller,
            effect,
            subject,
            event_amount,
            mode,
            intervening_if,
        } = pending;
        // CR 603.10 — if this trigger's source just left the battlefield
        // (it's in the die-snapshot cache), stash its last-known instance
        // so a "deals damage equal to its power" body reads the
        // counter/pump-boosted P/T rather than the graveyard's printed
        // value. Removed when the trigger resolves (`resolve_stack_item`).
        if let Some(snap) = self.died_card_snapshots.get(&source) {
            self.leaves_bf_lki.insert(source, snap.clone());
        }
        // CR 603.10 — likewise stash the dead *subject* (Jenova's dying Mutant)
        // so "equal to its power" reads its LKI P/T at resolution. Scoped via
        // `resolving_lki_subject` in `resolve_stack_item`.
        if let Some(crate::game::effects::EntityRef::Card(sid))
        | Some(crate::game::effects::EntityRef::Permanent(sid)) = subject
            && sid != source
            && let Some(snap) = self.died_card_snapshots.get(&sid)
        {
            self.leaves_bf_lki.insert(sid, snap.clone());
        }
        // CR 115.1c — an engine-resolved "up to N target" triggered ability
        // (Gavony Silversmith) maximizes its targets: fill slots 1.. with
        // distinct legal picks the same way the cast path threads
        // `additional_targets`. Without this the auto-targeter under-filled to
        // a single target.
        let additional = self.auto_extra_targets_for(&effect, source, controller, target.clone());
        self.stack.push(
            TriggerPush::new(source, controller, effect)
                .target(target)
                .additional_targets(additional)
                .mode(mode)
                .trigger_source(subject)
                .event_amount(event_amount)
                .intervening_if(intervening_if)
                .build(),
        );
    }

    /// Pick the slot-1+ targets for an engine-resolved `Effect::ApplyToTargets`
    /// (an "up to N target" triggered ability). Returns up to `max_targets - 1`
    /// distinct legal targets beyond `primary`, preferring permanents other
    /// than the source. Empty for any other effect or for `max_targets <= 1`.
    pub(crate) fn auto_extra_targets_for(
        &self,
        eff: &Effect,
        source: CardId,
        controller: usize,
        primary: Option<Target>,
    ) -> Vec<Target> {
        let max = match eff {
            Effect::ApplyToTargets { max_targets, .. } => *max_targets as usize,
            // Effects whose slots carry *distinct* per-slot filters (Kor
            // Outfitter's ETB `Attach { what: target Equipment, to: target
            // creature }`) can't be filled by the same-filter loop below —
            // they walk each slot's own filter instead.
            _ => return self.auto_extra_distinct_slot_targets(eff, source, controller, primary),
        };
        if max <= 1 || primary.is_none() {
            return vec![];
        }
        let mut chosen: Vec<Target> = Vec::new();
        // First avoid entry doubles as the OtherThanSource avoid-source; keep
        // the trigger source there, then grow the set with each pick.
        let mut avoid: Vec<CardId> = vec![source];
        if let Some(Target::Permanent(c)) = primary {
            avoid.push(c);
        }
        while chosen.len() + 1 < max {
            match self.auto_target_for_effect_avoiding_set(eff, controller, &avoid) {
                Some(t @ Target::Permanent(cid)) if !avoid.contains(&cid) => {
                    avoid.push(cid);
                    chosen.push(t);
                }
                _ => break,
            }
        }
        chosen
    }

    /// Fill slots 1.. of a triggered ability whose effect surfaces a *distinct*
    /// target filter per slot (Kor Outfitter's `Attach`, where slot 0 = the
    /// Equipment and slot 1 = the creature). Each slot is matched against its
    /// own `target_filter_for_slot` filter, preferring the controller's own
    /// permanents and avoiding anything an earlier slot already claimed. Empty
    /// for single-slot effects (the common case).
    pub(crate) fn auto_extra_distinct_slot_targets(
        &self,
        eff: &Effect,
        source: CardId,
        controller: usize,
        primary: Option<Target>,
    ) -> Vec<Target> {
        let slot1 = match eff.target_filter_for_slot_in_mode_kicked(1, None, false) {
            Some(f) => f,
            None => return vec![],
        };
        // Only fill here when slot 1 carries a *distinct* filter from slot 0
        // (Kor Outfitter's Equipment→creature Attach). Same-filter "up to N"
        // effects (DealDamageDivided, DistributeCounters, ApplyToTargets) keep
        // their dedicated single-target auto behavior / resolution-time divide
        // pickers — fanning them out here would wrongly split the effect.
        if eff.target_filter_for_slot_in_mode_kicked(0, None, false) == Some(slot1) {
            return vec![];
        }
        let opp = self
            .opponents_of(controller)
            .first()
            .copied()
            .unwrap_or((controller + 1) % self.players.len());
        let mut avoid: Vec<CardId> = vec![source];
        if let Some(Target::Permanent(c)) = primary {
            avoid.push(c);
        }
        let mut chosen: Vec<Target> = Vec::new();
        let mut slot: u8 = 1;
        while slot < 16 {
            let req = match eff.target_filter_for_slot_in_mode_kicked(slot, None, false) {
                Some(r) => r.clone(),
                None => break,
            };
            let is_legal = |t: &Target| -> bool {
                self.evaluate_requirement_static(&req, t, controller, Some(source))
                    && self.check_target_legality(t, controller).is_ok()
            };
            // Player slots: controller first (your-side bias), then opponent.
            let mut pick = [Target::Player(controller), Target::Player(opp)]
                .into_iter()
                .find(|t| is_legal(t));
            // Then a not-yet-claimed permanent, your own preferred.
            if pick.is_none() {
                pick = self
                    .battlefield
                    .iter()
                    .filter(|c| !avoid.contains(&c.id) && c.controller == controller)
                    .chain(self.battlefield.iter().filter(|c| !avoid.contains(&c.id)))
                    .map(|c| Target::Permanent(c.id))
                    .find(|t| is_legal(t));
            }
            match pick {
                Some(t) => {
                    if let Target::Permanent(cid) = t {
                        avoid.push(cid);
                    }
                    chosen.push(t);
                }
                None => break,
            }
            slot += 1;
        }
        chosen
    }


    /// Activate a loyalty ability on a planeswalker (sorcery speed, once per turn).
    pub fn activate_loyalty_ability(
        &mut self,
        card_id: CardId,
        ability_index: usize,
        target: Option<Target>,
        x_value: Option<u32>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let p = self.priority.player_with_priority;
        let pos = self
            .battlefield
            .iter()
            .position(|c| c.id == card_id)
            .ok_or(GameError::CardNotOnBattlefield(card_id))?;
        if self.battlefield[pos].controller != p {
            return Err(GameError::NotYourPriority);
        }
        // CR 606.3 — loyalty is normally sorcery-speed. CR 606.3b exception:
        // The Wandering Emperor may activate at instant speed (any time you
        // could cast an instant — i.e. while holding priority) the turn it
        // entered. Having priority is implied by reaching this action.
        let flash_loyalty_window = self.battlefield[pos].definition.flash_loyalty
            && self.battlefield[pos].entered_turn == Some(self.turn_number);
        if !flash_loyalty_window && !self.can_cast_sorcery_speed(p) {
            return Err(GameError::SorcerySpeedOnly);
        }
        if !self.battlefield[pos].definition.is_planeswalker() {
            return Err(GameError::InvalidTarget);
        }
        // CR 701.35 — a detained planeswalker's loyalty abilities can't be
        // activated (same gate as the regular activation path).
        if self.battlefield[pos].detained_by.is_some() {
            return Err(GameError::InvalidTarget);
        }
        // CR 606.3 — once per turn, or twice with Urza, Planeswalker's
        // printed override.
        let allowed = if self.battlefield[pos].definition.loyalty_twice_each_turn
            || self.battlefield[pos].loyalty_twice_this_turn
        {
            2
        } else {
            1
        };
        if self.battlefield[pos].loyalty_uses_this_turn >= allowed {
            return Err(GameError::LoyaltyAbilityAlreadyUsed(card_id));
        }

        // Printed + statics-granted abilities (Kasmina's sharing static,
        // Ichormoon Gauntlet's fixed grants) at indices past the printed
        // count — same list the wire view surfaces.
        let abilities =
            effective_loyalty_abilities(&self.battlefield[pos], &self.battlefield);
        let ability = abilities
            .get(ability_index)
            .cloned()
            .ok_or(GameError::AbilityIndexOutOfBounds)?;

        // Off-board (graveyard / exile) targets — "return target creature
        // card from your graveyard" (Professor Dellian, Fel −2): the cursor
        // can't select those, so the client activates with no target. Bind
        // slot 0 via a `ChooseCards` modal (suspend + clean replay — the
        // loyalty cost hasn't been paid yet).
        if target.is_none()
            && self.players[p].wants_ui
            && let Some(filter) = ability
                .effect
                .target_filter_for_slot(0)
                .filter(|f| f.mentions_offboard_zone())
                .map(|f| f.resolve_x(x_value.unwrap_or(0)))
        {
            let candidates: Vec<(CardId, String)> = self
                .players
                .iter()
                .flat_map(|pl| pl.graveyard.iter())
                .chain(self.exile.iter())
                .filter(|c| {
                    self.evaluate_requirement_static(
                        &filter,
                        &Target::Permanent(c.id),
                        p,
                        Some(card_id),
                    )
                })
                .map(|c| (c.id, c.definition.name.to_string()))
                .collect();
            if candidates.is_empty() {
                return Err(GameError::SelectionRequirementViolated);
            }
            let source_name = self.battlefield[pos].definition.name.to_string();
            self.pending_decision = Some(PendingDecision {
                decision: Decision::ChooseCards {
                    source: card_id,
                    prompt: format!("{source_name}: choose a card to target"),
                    candidates,
                    min: 1,
                    max: 1,
                },
                resume: ResumeContext::CastSlot0TargetPick {
                    caster: p,
                    action: Box::new(GameAction::ActivateLoyaltyAbility {
                        card_id,
                        ability_index,
                        target: None,
                        x_value,
                    }),
                },
            });
            return Ok(vec![]);
        }

        // Validate target — both targeting legality (hexproof / shroud /
        // protection / Leyline-of-Sanctity) and the loyalty effect's
        // own selector requirement (Teferi -3's "nonland permanent
        // an opponent controls" filter, etc.). Spell casts and
        // activated-ability activations both gate on these; loyalty
        // abilities went unchecked and would happily aim a Teferi -3
        // at the controller's own permanent.
        if let Some(tgt) = &target {
            self.check_target_legality(tgt, p)?;
            if let Some(filter) = ability.effect.target_filter_for_slot(0)
                && !self.evaluate_requirement_static(filter, tgt, p, Some(card_id))
            {
                return Err(GameError::SelectionRequirementViolated);
            }
        }

        // CR 606 — opponents' loyalty-tax statics (Eidolon of Obstruction)
        // make this activation cost extra generic mana. Pay it before the
        // loyalty cost so an unpayable tax aborts cleanly.
        let opps = self.opponents_of(p);
        let loyalty_tax: u32 = self
            .battlefield
            .iter()
            .filter(|c| opps.contains(&c.controller))
            .flat_map(|c| c.definition.static_abilities.iter())
            .filter_map(|sa| match sa.effect {
                crate::effect::StaticEffect::OpponentLoyaltyActivationTax { amount } => Some(amount),
                _ => None,
            })
            .sum();
        if loyalty_tax > 0 {
            self.try_pay_with_auto_tap(p, &crate::mana::cost(&[crate::mana::generic(loyalty_tax)]))?;
        }

        // Apply loyalty cost. CR 606.5: a `-X` ability lets the player pick X
        // (0..=current loyalty); the cost paid is X loyalty and the body reads
        // X via `Value::XFromCost`. Fixed-cost abilities ignore `x_value`.
        let current_loyalty =
            self.battlefield[pos].counter_count(crate::card::CounterType::Loyalty) as i32;
        let x = if ability.x_cost {
            x_value.unwrap_or(0).min(current_loyalty.max(0) as u32)
        } else {
            0
        };
        let loyalty_change = if ability.x_cost { -(x as i32) } else { ability.loyalty_cost };
        // Carth the Lion — "loyalty abilities you activate cost an
        // additional [+1]": the cost shifts by +N (a −2 pays as −1).
        let loyalty_change = loyalty_change
            + self
                .battlefield
                .iter()
                .filter(|c| c.controller == p)
                .flat_map(|c| c.definition.static_abilities.iter())
                .filter_map(|sa| match sa.effect {
                    crate::effect::StaticEffect::LoyaltyAbilitiesCostExtra(n) => Some(n),
                    _ => None,
                })
                .sum::<i32>();
        let new_loyalty = current_loyalty + loyalty_change;
        if new_loyalty < 0 {
            return Err(GameError::NotEnoughLoyalty(card_id));
        }
        self.battlefield[pos]
            .counters
            .insert(crate::card::CounterType::Loyalty, new_loyalty as u32);
        self.battlefield[pos].loyalty_uses_this_turn =
            self.battlefield[pos].loyalty_uses_this_turn.saturating_add(1);
        let mut events = vec![
            GameEvent::LoyaltyAbilityActivated {
                planeswalker: card_id,
                loyalty_change,
            },
            GameEvent::LoyaltyChanged {
                card_id,
                new_loyalty,
            },
        ];

        // Push ability effects onto the stack.
        self.stack.push(
            TriggerPush::new(card_id, p, ability.effect)
                .target(target)
                .x_value(x)
                .build(),
        );
        self.give_priority_to_active();

        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        Ok(events)
    }

    /// Begin the pre-game London-mulligan phase. Deals 7 cards to each player
    /// and sets `pending_decision` for seat 0's opening-hand choice.
    /// Call this after constructing the `GameState` and before the first turn.
    pub fn start_mulligan_phase(&mut self) {
        let n = self.players.len();
        for i in 0..n {
            self.deal_to_hand(i, 7);
        }
        self.set_mulligan_decision(0, 0, if n > 1 { Some(1) } else { None });
    }

    fn deal_to_hand(&mut self, seat: usize, count: usize) {
        // Top of library is `library[0]` — `pop()` would deal from the
        // bottom, which produces the wrong opening hand for unshuffled
        // (test-fixture) decks. Drain the top `count` cards instead.
        for _ in 0..count {
            if self.players[seat].library.is_empty() {
                break;
            }
            let card = self.players[seat].library.remove(0);
            self.players[seat].hand.push(card);
        }
    }

    fn shuffle_hand_to_library(&mut self, seat: usize) {
        use rand::seq::SliceRandom;
        let hand = std::mem::take(&mut self.players[seat].hand);
        for card in hand {
            self.players[seat].library.push(card);
        }
        let mut rng = rand::rng();
        self.players[seat].library.shuffle(&mut rng);
    }

    fn set_mulligan_decision(&mut self, player: usize, mulligans_taken: usize, next_player: Option<usize>) {
        let hand: Vec<_> = self.players[player].hand
            .iter()
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        // Surface any in-hand Serum Powder–style mulligan helpers so the
        // decider can pick an alternative answer.
        let serum_powders: Vec<_> = self.players[player].hand
            .iter()
            .filter(|c| matches!(
                c.definition.opening_hand,
                Some(crate::effect::OpeningHandEffect::MulliganHelper),
            ))
            .map(|c| c.id)
            .collect();
        self.pending_decision = Some(PendingDecision {
            decision: Decision::Mulligan { player, hand, mulligans_taken, serum_powders },
            resume: ResumeContext::Mulligan { player, mulligans_taken, next_player },
        });
    }

    /// Submit an answer to the currently-pending decision and resume resolution.
    /// Fails if no decision is pending, or the answer shape doesn't match the
    /// decision kind.
    pub fn submit_decision(&mut self, answer: DecisionAnswer) -> Result<Vec<GameEvent>, GameError> {
        let pd = self
            .pending_decision
            .take()
            .ok_or(GameError::NoDecisionPending)?;
        let mut events = match pd.resume {
            ResumeContext::Spell {
                card,
                caster,
                target,
                additional_targets,
                mode,
                x_value,
                converged_value,
                mana_spent,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_spell_resolution(
                    *card,
                    caster,
                    target,
                    additional_targets,
                    mode,
                    x_value,
                    converged_value,
                    mana_spent,
                    Some(remaining),
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Trigger {
                source,
                controller,
                target,
                mode,
                in_progress,
                remaining,
                x_value,
                converged_value,
                mana_spent,
                trigger_source_ent,
                event_amount,
                additional_targets,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_trigger_resolution_with_source(
                    source, controller, remaining, target, mode, x_value, converged_value,
                    mana_spent, trigger_source_ent, event_amount, additional_targets,
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Ability {
                source,
                controller,
                target,
                in_progress,
                remaining,
            } => {
                let mut evs = self.apply_pending_effect_answer(in_progress, &answer)?;
                let mut more = self.continue_ability_resolution(
                    source, controller, remaining, target,
                )?;
                evs.append(&mut more);
                evs
            }
            ResumeContext::Mulligan { player, mulligans_taken, next_player } => {
                match answer {
                    DecisionAnswer::TakeMulligan => {
                        self.shuffle_hand_to_library(player);
                        self.deal_to_hand(player, 7);
                        self.set_mulligan_decision(player, mulligans_taken + 1, next_player);
                        return Ok(vec![]);
                    }
                    DecisionAnswer::Keep => {
                        if mulligans_taken > 0 {
                            let hand = self.players[player].hand
                                .iter()
                                .map(|c| (c.id, c.definition.name.to_string()))
                                .collect();
                            self.pending_decision = Some(PendingDecision {
                                decision: Decision::PutOnLibrary {
                                    player,
                                    count: mulligans_taken,
                                    hand,
                                },
                                // Carry the mulligan count forward so the
                                // PutOnLibrary handler below knows how many
                                // cards to bottom.
                                resume: ResumeContext::Mulligan { player, mulligans_taken, next_player },
                            });
                            return Ok(vec![]);
                        }
                        self.advance_mulligan(next_player);
                        return Ok(vec![]);
                    }
                    DecisionAnswer::PutOnLibrary(ids) => {
                        // London mulligan: chosen cards go to the BOTTOM of
                        // the library. CR 103.5a — exactly `mulligans_taken`
                        // cards must leave the hand; re-pose on a short or
                        // bogus answer so a hostile client can't keep a
                        // fresh seven.
                        let mut bottomed = 0usize;
                        for card_id in ids.iter().take(mulligans_taken) {
                            if let Some(card) = Self::take_card(&mut self.players[player].hand, *card_id) {
                                self.players[player].library.push(card);
                                bottomed += 1;
                            }
                        }
                        let still_owed = mulligans_taken - bottomed;
                        if still_owed > 0 && !self.players[player].hand.is_empty() {
                            let hand = self.players[player].hand
                                .iter()
                                .map(|c| (c.id, c.definition.name.to_string()))
                                .collect();
                            self.pending_decision = Some(PendingDecision {
                                decision: Decision::PutOnLibrary {
                                    player,
                                    count: still_owed,
                                    hand,
                                },
                                resume: ResumeContext::Mulligan {
                                    player,
                                    mulligans_taken: still_owed,
                                    next_player,
                                },
                            });
                            return Ok(vec![]);
                        }
                        self.advance_mulligan(next_player);
                        return Ok(vec![]);
                    }
                    DecisionAnswer::SerumPowder(powder_id) => {
                        // Serum Powder: exile the entire current hand (the
                        // powder card itself goes with it), then draw a new
                        // seven. Doesn't bump `mulligans_taken` — Serum
                        // Powder is intentionally separate from the London
                        // mulligan ladder (so multiple powders can stack
                        // without progressively shrinking the eventual hand).
                        // Reject if the named Serum Powder isn't actually in
                        // hand or doesn't carry the `MulliganHelper` flag.
                        let valid = self.players[player].hand.iter().any(|c| {
                            c.id == powder_id
                                && matches!(
                                    c.definition.opening_hand,
                                    Some(crate::effect::OpeningHandEffect::MulliganHelper),
                                )
                        });
                        if !valid {
                            return Err(GameError::DecisionAnswerMismatch);
                        }
                        let exiled: Vec<crate::card::CardInstance> =
                            std::mem::take(&mut self.players[player].hand);
                        for card in exiled {
                            self.exile.push(card);
                        }
                        self.deal_to_hand(player, 7);
                        self.set_mulligan_decision(player, mulligans_taken, next_player);
                        return Ok(vec![]);
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
            }
            ResumeContext::TriggerOrder { mut ordered, run, rest } => {
                // CR 603.3b — apply the controller's chosen order, then
                // continue the ordering walk (which may suspend again on a
                // later same-controller run) and finish the dispatch.
                let order = match answer {
                    DecisionAnswer::TriggerOrder(ids) => ids,
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                Self::apply_trigger_order(&mut ordered, run, order);
                if let Some(all) = self.continue_trigger_ordering(ordered, rest) {
                    self.push_ordered_trigger_candidates(all);
                }
                vec![]
            }
            ResumeContext::TriggerTargetPick { pending, remaining } => {
                // Apply the answered target to the trigger that was
                // waiting on it, then continue draining the queue
                // (which may suspend again on the next targeted
                // trigger in the same batch). Off-board (graveyard /
                // exile) picks arrive as `Cards` from the modal flow.
                let target = match answer {
                    DecisionAnswer::Target(t) => Some(t),
                    DecisionAnswer::Cards(ids) => {
                        let Some(id) = ids.first().copied() else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        Some(Target::Permanent(id))
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                self.push_pending_trigger(pending, target);
                self.drain_trigger_queue(remaining);
                vec![]
            }
            ResumeContext::CleanupDiscard { player } => {
                // CR 514.1 — apply the player's chosen discards, then resume
                // the rest of cleanup and the step advance.
                let ids = match &answer {
                    DecisionAnswer::Discard(ids) => ids.clone(),
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                let mut evs = Vec::new();
                for id in ids {
                    // CR 514.1 — discard down to, never past, the maximum.
                    // Once the hand is back at the limit, ignore any further
                    // ids so a buggy/hostile client can't force an
                    // over-discard with an oversized answer.
                    let over = self
                        .effective_max_hand_size(player)
                        .is_some_and(|max| self.players[player].hand.len() > max);
                    if !over {
                        break;
                    }
                    if self.players[player].hand.iter().any(|c| c.id == id) {
                        self.discard_card(player, id, &mut evs);
                    }
                }
                if !evs.is_empty() {
                    self.dispatch_triggers_for_events(&evs);
                }
                // Under-discard (the answer pitched too few): re-pose the
                // decision until the hand is back at the maximum.
                if let Some(max) = self.effective_max_hand_size(player)
                    && self.players[player].hand.len() > max
                {
                    let excess = (self.players[player].hand.len() - max) as u32;
                    self.set_cleanup_discard_decision(player, excess);
                    return Ok(evs);
                }
                return match self.finish_cleanup(&mut evs) {
                    crate::game::stack::CleanupOutcome::TurnOver => self.advance_step(evs),
                    _ => Ok(evs),
                };
            }
            ResumeContext::CombatDamage { player: _, attacker, kind } => {
                // CR 510.1c-d — cache the answered ordering/assignment choice,
                // then re-enter the current damage step. It re-runs the (now
                // cached) gather and either suspends on the next choice or
                // applies all combat damage. Mirrors the pass_priority combat
                // arms (give priority + dispatch triggers) on completion.
                self.apply_combat_decision_answer(attacker, kind, &answer);
                let evs = match self.step {
                    TurnStep::FirstStrikeDamage => self.resolve_first_strike_damage()?,
                    TurnStep::CombatDamage => self.resolve_combat()?,
                    _ => Vec::new(),
                };
                if self.pending_decision.is_none() {
                    self.give_priority_to_active();
                    self.dispatch_triggers_for_events(&evs);
                }
                return Ok(evs);
            }
            ResumeContext::CastAdditionalCost {
                caster,
                card_id,
                target,
                additional_targets,
                mode,
                x_value,
                kicked,
            } => {
                // CR 601.2b — the caster paid an additional cost choice. The
                // answer type says which: a permanent target (sacrifice) or a
                // discard list. Validate, stash it for `pay_additional_costs`,
                // and re-run the cast. The cast was suspended before any cost
                // was paid, so re-invoking from the top is a clean replay (no
                // double-spend / double-removal); it may suspend again for a
                // further additional cost.
                match &answer {
                    DecisionAnswer::Target(Target::Permanent(id))
                        if self.cast_sacrifice_choice_is_legal(caster, card_id, *id) =>
                    {
                        self.pending_cast_sacrifices = Some(vec![*id]);
                    }
                    DecisionAnswer::Discard(ids) => {
                        // Trust the option list that was posed (the caster's
                        // hand minus the card being cast); the apply path in
                        // `pay_additional_costs` re-checks each id is in hand.
                        self.pending_cast_discards = Some(ids.clone());
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
                // Priority is still the caster's (we never advanced it), so
                // the cast reads the right actor. Any cost failure (e.g.
                // mana shortfall) surfaces as a normal cast error. A kicked
                // suspend replays kicked (CR 702.33).
                let result = if kicked {
                    self.cast_spell_kicked(card_id, target, additional_targets, mode, x_value)
                } else {
                    self.cast_spell(card_id, target, additional_targets, mode, x_value)
                };
                // A prepare-spell copy that suspended here still needs its
                // token-flag/unprepare bookkeeping (no-op otherwise).
                return self.settle_prepare_after_cast(card_id, result);
            }
            ResumeContext::ActionFloatConfirm { actor, action } => {
                // CR 601.2g — the payer chose whether to spend floating mana.
                // Stash the choice and replay the exact originating action
                // (priority is still theirs, so it reads the right actor).
                let DecisionAnswer::Bool(spend) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let _ = actor;
                self.pending_cast_spend_float = Some(spend);
                // A prepare-spell copy replays as a plain `CastSpell` here —
                // settle its token-flag/unprepare bookkeeping afterwards
                // (no-op for anything that isn't a registered prepare copy).
                let cast_card = action.cast_card_id();
                let result = self.perform_action(*action);
                return match cast_card {
                    Some(id) => self.settle_prepare_after_cast(id, result),
                    None => result,
                };
            }
            ResumeContext::CastExtraTargetPick { caster, action } => {
                let DecisionAnswer::Target(t) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let _ = caster;
                let mut action = *action;
                let GameAction::CastSpell { additional_targets, .. } = &mut action else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                additional_targets.push(t);
                // Prepare-spell copies suspend/resume through plain CastSpell
                // replays — settle their bookkeeping wherever the cast lands.
                let cast_card = action.cast_card_id();
                let result = self.perform_action(action);
                return match cast_card {
                    Some(id) => self.settle_prepare_after_cast(id, result),
                    None => result,
                };
            }
            ResumeContext::CastXPick { caster, action } => {
                let DecisionAnswer::Amount(n) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let _ = caster;
                let mut action = *action;
                match &mut action {
                    GameAction::CastSpell { x_value, .. }
                    | GameAction::CastPrepareSpell { x_value, .. }
                    | GameAction::CastFlashback { x_value, .. } => *x_value = Some(n),
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
                let cast_card = action.cast_card_id();
                let result = self.perform_action(action);
                return match cast_card {
                    Some(id) => self.settle_prepare_after_cast(id, result),
                    None => result,
                };
            }
            ResumeContext::CastSlot0TargetPick { caster, action } => {
                let DecisionAnswer::Cards(ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let Some(id) = ids.first().copied() else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let _ = caster;
                let mut action = *action;
                match &mut action {
                    GameAction::CastSpell { target, .. }
                    | GameAction::CastPrepareSpell { target, .. }
                    | GameAction::ActivateLoyaltyAbility { target, .. } => {
                        *target = Some(Target::Permanent(id));
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
                let cast_card = action.cast_card_id();
                let result = self.perform_action(action);
                return match cast_card {
                    Some(id) => self.settle_prepare_after_cast(id, result),
                    None => result,
                };
            }
            ResumeContext::ActionSearchPick { actor, action } => {
                // CR 702.29e — the cycler picked which card to fetch. Stash
                // and replay the originating action.
                let DecisionAnswer::Search(pick) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let _ = actor;
                self.pending_landcycle_pick = Some(pick);
                let r = self.perform_action(*action);
                self.pending_landcycle_pick = None;
                return r;
            }
            ResumeContext::ActivateAbilityChoice {
                activator,
                card_id,
                ability_index,
                target,
                additional_targets,
                x_value,
                kind,
            } => {
                // CR 602.5 — the activator picked how to pay one of the
                // ability's "another …" costs. Stash it for the matching cost
                // and replay the activation from the top — nothing was paid
                // before the suspend, so this is a clean replay (which may
                // suspend again for a further choice). Sacrifice/tap come back
                // as a battlefield `Target`; the graveyard exile as `Cards`.
                use crate::game::types::AbilityCostChoice as K;
                match kind {
                    K::SacOther | K::TapOther => {
                        let DecisionAnswer::Target(Target::Permanent(id)) = answer else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        if id == card_id
                            || self.battlefield_find(id).is_none_or(|c| c.controller != activator)
                        {
                            return Err(GameError::DecisionAnswerMismatch);
                        }
                        if matches!(kind, K::SacOther) {
                            self.pending_ability_sac_other = Some(id);
                        } else {
                            self.pending_ability_tap_other = Some(id);
                        }
                    }
                    K::ExileOther => {
                        let DecisionAnswer::Cards(ids) = answer else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        // Trust the posed option list (the activator's graveyard
                        // minus the source); `activate_ability` re-checks each id
                        // is still in the graveyard and matches the filter.
                        self.pending_ability_exile_other = Some(ids);
                    }
                    K::XValue => {
                        let DecisionAnswer::Amount(n) = answer else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        return self.activate_ability(
                            card_id,
                            ability_index,
                            target,
                            additional_targets,
                            Some(n),
                        );
                    }
                    K::GraveyardTarget => {
                        // The pick is the activation's slot-0 target (a
                        // graveyard card); the replay re-validates it against
                        // the effect's target filter.
                        let DecisionAnswer::Cards(ids) = answer else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        let Some(id) = ids.first().copied() else {
                            return Err(GameError::DecisionAnswerMismatch);
                        };
                        return self.activate_ability(
                            card_id,
                            ability_index,
                            Some(Target::Permanent(id)),
                            additional_targets,
                            x_value,
                        );
                    }
                }
                return self.activate_ability(card_id, ability_index, target, additional_targets, x_value);
            }
        };
        let mut sba = self.check_state_based_actions();
        events.append(&mut sba);
        self.dispatch_triggers_for_events(&events);
        Ok(events)
    }

    fn advance_mulligan(&mut self, next_player: Option<usize>) {
        match next_player {
            Some(p) => {
                let after = (p + 1 < self.players.len()).then_some(p + 1);
                self.set_mulligan_decision(p, 0, after);
            }
            None => {
                // All players kept — apply opening-hand effects (Leyline of
                // Sanctity / Gemstone Caverns start in play; Chancellor reveals
                // schedule delayed triggers) and start the game with priority
                // on seat 0.
                self.apply_opening_hand_effects();
                self.pending_decision = None;
                self.give_priority_to_active();
            }
        }
    }

    /// Walk every player's opening hand and apply each card's
    /// `OpeningHandEffect`. The default `Decider` answers "yes" to every
    /// optional reveal — the `AutoDecider` and the bot benefit from these
    /// effects in the demo decks, and a future UI can deny the reveal by
    /// returning `Bool(false)` from an `OptionalTrigger` decision (not yet
    /// surfaced — opening-hand effects auto-fire today).
    /// Backwards-compat alias used by some tests — fires every player's
    /// opening-hand effects immediately. Equivalent to (and delegates to)
    /// `apply_opening_hand_effects`.
    pub fn fire_start_of_game_effects(&mut self) {
        self.apply_opening_hand_effects();
    }

    pub(crate) fn apply_opening_hand_effects(&mut self) {
        let n = self.players.len();
        for p in 0..n {
            // Snapshot ids first so we can iterate without aliasing the hand.
            let ids: Vec<crate::card::CardId> =
                self.players[p].hand.iter().map(|c| c.id).collect();
            for cid in ids {
                let oh = self.players[p]
                    .hand
                    .iter()
                    .find(|c| c.id == cid)
                    .and_then(|c| c.definition.opening_hand.clone());
                let Some(oh) = oh else { continue };
                match oh {
                    crate::effect::OpeningHandEffect::StartInPlay { tapped, extra } => {
                        // Pull the card out of hand and place it on the
                        // battlefield under its owner's control.
                        if let Some(mut card) = Self::take_card(&mut self.players[p].hand, cid) {
                            card.controller = p;
                            card.tapped = tapped;
                            card.summoning_sick = card.definition.is_creature();
                            self.battlefield.push(card);
                            // Run the optional follow-up effect (e.g. Gemstone
                            // Caverns wants a luck counter on its newly-entered
                            // self).
                            if !matches!(extra, crate::effect::Effect::Noop) {
                                let ctx = crate::game::effects::EffectContext::for_ability(
                                    cid, p, None,
                                );
                                let _ = self.resolve_effect(&extra, &ctx);
                            }
                            // Fire any self-source ETB triggers (the same hook
                            // play_land uses), so static-as-replaced abilities
                            // and "enters with N counters" still fire if the
                            // card uses that idiom in addition to `extra`.
                            self.fire_self_etb_triggers(cid, p);
                        }
                    }
                    crate::effect::OpeningHandEffect::RevealForDelayedTrigger { kind, body } => {
                        // Card stays in hand; register a delayed trigger that
                        // fires later (next upkeep / first main / end step).
                        use crate::game::types::DelayedTrigger;
                        let dk = crate::game::effects::delayed_kind_from_effect(kind);
                        self.delayed_triggers.push(DelayedTrigger {
                            controller: p,
                            source: cid,
                            kind: dk,
                            effect: body,
                            target: None,
                            bound_token: None,
                            fires_once: true,
                        });
                    }
                    crate::effect::OpeningHandEffect::MulliganHelper => {
                        // Surfaces during mulligan only; nothing to do here.
                    }
                }
            }
        }
    }

    /// Complete the suspended effect using the player's answer. Returns the
    /// events generated by the now-finished effect (e.g. `ScryPerformed`).
    pub(crate) fn apply_pending_effect_answer(
        &mut self,
        state: PendingEffectState,
        answer: &DecisionAnswer,
    ) -> Result<Vec<GameEvent>, GameError> {
        match state {
            PendingEffectState::ScryPeeked { count, player } => {
                let DecisionAnswer::ScryOrder { kept_top, bottom } = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(kept_top.len());
                for id in kept_top {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                let mut bottom_cards = Vec::with_capacity(bottom.len());
                for id in bottom {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        bottom_cards.push(remaining.remove(pos));
                    }
                }
                // Cards listed in neither bucket default to top (end of top).
                top_cards.extend(remaining);
                let bottomed = bottom_cards.len();
                let lib = &mut self.players[player].library;
                for c in bottom_cards {
                    lib.push(c);
                }
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![
                    GameEvent::ScryPerformed { player, looked_at: count, bottomed },
                    // CR 701.22 — fires "whenever you scry or surveil" payoffs
                    // (Matoya). Emitted here so both the synchronous and the
                    // suspended (human) resolution paths report it; RearrangeTop
                    // (Index/Spire Owl) is deliberately excluded.
                    GameEvent::ScriedOrSurveiled { player, surveil: false },
                ])
            }
            PendingEffectState::RearrangePeeked { count, player } => {
                // Index / Spire Owl — every peeked card returns to the top in
                // the chosen order; the `bottom` list is treated as "kept on
                // top, after kept_top" so nothing is ever bottomed.
                let DecisionAnswer::ScryOrder { kept_top, bottom } = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(count);
                for id in kept_top.iter().chain(bottom.iter()) {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                top_cards.extend(remaining);
                let lib = &mut self.players[player].library;
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![GameEvent::ScryPerformed { player, looked_at: count, bottomed: 0 }])
            }
            PendingEffectState::SurveilPeeked { count, player } => {
                // Surveil: player chooses which cards go to the graveyard; rest go to top.
                let DecisionAnswer::ScryOrder {
                    kept_top,
                    bottom: to_graveyard,
                } = answer
                else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut remaining: Vec<CardInstance> =
                    self.players[player].library.drain(..count).collect();
                let mut top_cards = Vec::with_capacity(kept_top.len());
                for id in kept_top {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        top_cards.push(remaining.remove(pos));
                    }
                }
                let mut graveyard_cards = Vec::with_capacity(to_graveyard.len());
                for id in to_graveyard {
                    if let Some(pos) = remaining.iter().position(|c| c.id == *id) {
                        graveyard_cards.push(remaining.remove(pos));
                    }
                }
                top_cards.extend(remaining);
                let graveyarded = graveyard_cards.len();
                for c in graveyard_cards {
                    self.players[player].send_to_graveyard(c);
                }
                let lib = &mut self.players[player].library;
                for c in top_cards.into_iter().rev() {
                    lib.insert(0, c);
                }
                Ok(vec![
                    GameEvent::SurveilPerformed { player, looked_at: count, graveyarded },
                    GameEvent::ScriedOrSurveiled { player, surveil: true },
                ])
            }
            PendingEffectState::LearnPending { player } => {
                let DecisionAnswer::Learn(choice) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                self.apply_learn_choice(player, choice.clone(), &mut events);
                Ok(events)
            }
            PendingEffectState::SearchPending { player, to, eligible, include_graveyard } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                if let Some(card_id) = chosen_id
                    && eligible.as_ref().is_none_or(|e| e.contains(card_id))
                {
                    // The pick may sit in the library, or (dual-zone search:
                    // Delivery Moogle) in the graveyard.
                    let from_zone = if self.players[player].library.iter().any(|c| c.id == *card_id)
                    {
                        Some(crate::card::Zone::Library)
                    } else if include_graveyard
                        && self.players[player].graveyard.iter().any(|c| c.id == *card_id)
                    {
                        Some(crate::card::Zone::Graveyard)
                    } else {
                        None
                    };
                    if let Some(from_zone) = from_zone {
                        let is_gy = from_zone == crate::card::Zone::Graveyard;
                        // Grafdigger's Cage — a locked card can't leave the
                        // library/graveyard for the battlefield while up.
                        let src = if is_gy {
                            &self.players[player].graveyard
                        } else {
                            &self.players[player].library
                        };
                        let def = src.iter().find(|c| c.id == *card_id).map(|c| c.definition.clone());
                        let blocked = matches!(to, crate::effect::ZoneDest::Battlefield { .. })
                            && def.as_ref().is_some_and(|d| {
                                self.battlefield_entry_from_zone_blocked(d, from_zone)
                            });
                        let taken = if blocked {
                            None
                        } else if is_gy {
                            Self::take_card(&mut self.players[player].graveyard, *card_id)
                        } else {
                            Self::take_card(&mut self.players[player].library, *card_id)
                        };
                        if let Some(card) = taken {
                            self.place_card_in_dest(card, player, &to, &mut events);
                            // Surface the found card so a downstream `Selector::LastMoved`
                            // can inspect its type (Oriq Loremage's "if instant/sorcery").
                            self.last_moved_cards.push(*card_id);
                        }
                    }
                }
                Ok(events)
            }
            PendingEffectState::ImpulsePending { player, revealed, rest_to_graveyard, eligible, take, to_battlefield, tapped, keep_on_top, gain_life_if_pick, gain_life_greatest_power_rest, rest_to_exile } => {
                // `None` eligible means "any revealed card" (no filter).
                let is_eligible = |id: &CardId| match &eligible {
                    None => true,
                    Some(v) => v.contains(id),
                };
                // A single-pick suspend answers `Search`; a take>1 suspend
                // answers `Cards` (Dig Through Time's real two-card pick).
                // Out-of-set ids are ignored; any shortfall auto-fills from
                // the remaining eligible revealed cards (AutoDecider /
                // empty pick keeps the top-down fill).
                let mut picks: Vec<CardId> = Vec::with_capacity(take);
                match answer {
                    DecisionAnswer::Search(chosen_id) => {
                        if let Some(id) = *chosen_id
                            && revealed.contains(&id)
                            && is_eligible(&id)
                        {
                            picks.push(id);
                        }
                    }
                    DecisionAnswer::Cards(chosen) => {
                        for id in chosen {
                            if picks.len() >= take {
                                break;
                            }
                            if revealed.contains(id) && is_eligible(id) && !picks.contains(id) {
                                picks.push(*id);
                            }
                        }
                    }
                    _ => return Err(GameError::DecisionAnswerMismatch),
                }
                for id in revealed.iter().copied() {
                    if picks.len() >= take {
                        break;
                    }
                    if is_eligible(&id) && !picks.contains(&id) {
                        picks.push(id);
                    }
                }
                let mut events = vec![];
                // Chrome Courier — check the picks against the rider filter
                // while they're still library objects.
                let pick_rider_life: Option<u32> = gain_life_if_pick.as_ref().and_then(|(f, n)| {
                    picks
                        .iter()
                        .any(|id| {
                            self.evaluate_requirement_static(
                                f,
                                &crate::game::types::Target::Permanent(*id),
                                player,
                                None,
                            )
                        })
                        .then_some(*n)
                });
                // Sage of Days: the pick stays on top of the library (it isn't
                // removed here), and the milling loop below clears the rest, so
                // the kept card rises to the top.
                for &pick in &picks {
                    if keep_on_top {
                        continue;
                    }
                    if let Some(card) = Self::take_card(&mut self.players[player].library, pick) {
                        if to_battlefield {
                            // Collected Company — picks enter the battlefield
                            // (ETBs fire through the shared placement funnel).
                            self.place_card_in_dest(
                                card,
                                player,
                                &crate::effect::ZoneDest::Battlefield {
                                    controller: crate::effect::PlayerRef::Seat(player),
                                    tapped,
                                },
                                &mut events,
                            );
                        } else {
                            // CR 121.5 — putting a card into hand this way
                            // is NOT a draw: no CardDrawn event, no
                            // draw-trigger fire (Sheoldred/Bowmasters).
                            self.players[player].hand.push(card);
                        }
                    }
                }
                // Move the rest of the revealed set to the bottom of the
                // library (or graveyard). They're still at the top of the
                // library after the picks were removed.
                let mut greatest_milled_power: Option<i32> = None;
                for rid in &revealed {
                    if picks.contains(rid) {
                        continue;
                    }
                    if let Some(card) = Self::take_card(&mut self.players[player].library, *rid) {
                        if rest_to_exile {
                            // Devourer of Destiny — the non-kept cards are
                            // exiled outright.
                            self.exile.push(card);
                        } else if rest_to_graveyard {
                            // Discerning Taste — track the greatest power
                            // among milled creature cards.
                            if gain_life_greatest_power_rest && card.definition.is_creature() {
                                greatest_milled_power =
                                    Some(greatest_milled_power.unwrap_or(0).max(card.definition.power));
                            }
                            // CR 614.6 — honor graveyard-hate redirects.
                            self.route_to_graveyard(card, &mut events);
                        } else {
                            self.players[player].library.push(card);
                        }
                    }
                }
                let rider_gain = pick_rider_life.map(|n| n as i32).unwrap_or(0)
                    + greatest_milled_power.unwrap_or(0).max(0);
                if rider_gain > 0 {
                    let applied = self.adjust_life_applied(player, rider_gain);
                    if applied > 0 {
                        events.push(GameEvent::LifeGained { player, amount: applied as u32 });
                    } else if applied < 0 {
                        events.push(GameEvent::LifeLost { player, amount: (-applied) as u32 });
                    }
                }
                Ok(events)
            }
            PendingEffectState::PayLifeLookPending { player, revealed } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // Default (AutoDecider / out-of-set): take the top revealed.
                let pick = chosen_id
                    .filter(|id| revealed.contains(id))
                    .or_else(|| revealed.first().copied());
                if let Some(pick) = pick
                    && let Some(card) = Self::take_card(&mut self.players[player].library, pick) {
                    // CR 121.5 — put into hand, not drawn: no CardDrawn.
                    self.players[player].hand.push(card);
                }
                // Exile the rest of the revealed set.
                for rid in &revealed {
                    if Some(*rid) == pick { continue; }
                    if let Some(card) = Self::take_card(&mut self.players[player].library, *rid) {
                        self.exile.push(card);
                    }
                }
                Ok(vec![])
            }
            PendingEffectState::TakeOnePerTypePending { player, revealed } => {
                let DecisionAnswer::Cards(chosen) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // "Up to one card of each card type" — walk the picks in
                // answer order, assigning each the first of its card types
                // not yet covered; a pick whose types are all covered (or
                // that wasn't revealed) is dropped rather than rejected.
                let mut covered: Vec<crate::card::CardType> = Vec::new();
                let mut taken: Vec<CardId> = Vec::new();
                for id in chosen {
                    if !revealed.contains(id) || taken.contains(id) {
                        continue;
                    }
                    let Some(card) = self.players[player].library.iter().find(|c| c.id == *id) else {
                        continue;
                    };
                    if let Some(ty) = card.definition.card_types.iter()
                        .find(|t| !covered.contains(t))
                        .cloned()
                    {
                        covered.push(ty);
                        taken.push(*id);
                    }
                }
                // Picks to hand (CR 121.5 — put, not drawn), rest to the
                // bottom in a random order (CR 401.4 hidden arrangement).
                for id in &taken {
                    if let Some(card) = Self::take_card(&mut self.players[player].library, *id) {
                        self.players[player].hand.push(card);
                    }
                }
                use rand::seq::SliceRandom;
                let mut rest: Vec<CardId> =
                    revealed.iter().copied().filter(|id| !taken.contains(id)).collect();
                rest.shuffle(&mut rand::rng());
                for id in rest {
                    if let Some(card) = Self::take_card(&mut self.players[player].library, id) {
                        self.players[player].library.push(card);
                    }
                }
                Ok(vec![])
            }
            PendingEffectState::PutOnLibraryPending { player, .. } => {
                let DecisionAnswer::PutOnLibrary(chosen) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                self.execute_put_on_library(player, chosen, &mut events);
                Ok(events)
            }
            PendingEffectState::AnyOneColorPending { player, count, restriction } => {
                let DecisionAnswer::Color(c) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    match restriction {
                        Some(r) => self.players[player].mana_pool.add_restricted(*c, 1, r),
                        None => self.players[player].mana_pool.add(*c, 1),
                    }
                    events.push(GameEvent::ManaAdded { player, color: *c, source: None });
                }
                Ok(events)
            }
            PendingEffectState::DevotionColorPending { player } => {
                let DecisionAnswer::Color(c) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let n = self.devotion_to(player, &[*c]).max(0) as u32;
                let mut events = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    self.players[player].mana_pool.add(*c, 1);
                    events.push(GameEvent::ManaAdded { player, color: *c, source: None });
                }
                Ok(events)
            }
            PendingEffectState::SacrificePending { player } => {
                // CR 701.16 — the player chose which permanent(s) to sacrifice.
                // A single sacrifice comes back as a `Target` (in-scene cursor);
                // a multi-sacrifice as `Cards` (the ChooseCards modal).
                let ids: Vec<CardId> = match answer {
                    DecisionAnswer::Target(Target::Permanent(id)) => vec![*id],
                    DecisionAnswer::Cards(ids) => ids.clone(),
                    _ => return Err(GameError::DecisionAnswerMismatch),
                };
                // Trust the option list that was posed (built from the legal
                // candidates), but guard against stale/hostile ids: each must
                // still be a permanent the sacrificing player controls.
                if ids.is_empty()
                    || !ids.iter().all(|id| {
                        self.battlefield_find(*id).is_some_and(|c| c.controller == player)
                    })
                {
                    return Err(GameError::DecisionAnswerMismatch);
                }
                let mut events = Vec::new();
                for id in ids {
                    self.sacrifice_one(id, player, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::DiscardChosenPending { target_player } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(card_ids.len());
                for cid in card_ids {
                    // The zone move + CardDiscarded + discard-matters
                    // counters + Madness replacement (CR 702.35) are all
                    // centralized in `discard_card`.
                    self.discard_card(target_player, *cid, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::BottomChosenFromHandAndDrawPending { target_player } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::new();
                for cid in card_ids {
                    // Move the chosen card from hand to the bottom of its
                    // owner's library, then draw a replacement (Vendilion
                    // Clique). Library index 0 = top, so `push` = bottom.
                    if let Some(card) = Self::take_card(&mut self.players[target_player].hand, *cid)
                    {
                        self.players[target_player].library.push(card);
                        self.draw_one(target_player, &mut events);
                    }
                }
                Ok(events)
            }
            PendingEffectState::ExileChosenUntilSourceLeavesPending {
                target_player,
                source,
                return_to,
            } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(card_ids.len());
                for cid in card_ids {
                    // Move the chosen card from hand to exile and link it to
                    // the source permanent.
                    if let Some(mut card) =
                        Self::take_card(&mut self.players[target_player].hand, *cid)
                    {
                        card.exiled_by = Some(crate::card::ExileLink {
                            source,
                            return_to,
                            monarch_guard: None,
                        });
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::ExileChosenFromHandPending { target_player } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::with_capacity(card_ids.len());
                for cid in card_ids {
                    if let Some(card) = Self::take_card(&mut self.players[target_player].hand, *cid)
                    {
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::HoneFromHandPending { target_player, count } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = Vec::new();
                for cid in card_ids {
                    if let Some(mut card) =
                        Self::take_card(&mut self.players[target_player].hand, *cid)
                    {
                        card.add_counters(crate::card::CounterType::Hone, count);
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::ExileFromHandTaxedPending { target_player, extra_cost } => {
                let DecisionAnswer::Discard(card_ids) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let turn = self.turn_number;
                let mut events = Vec::new();
                for cid in card_ids {
                    if let Some(mut card) =
                        Self::take_card(&mut self.players[target_player].hand, *cid)
                    {
                        let owner = card.owner;
                        // Owner may play it, taxed `extra_cost` more, while exiled.
                        let mut taxed = card.definition.cost.clone();
                        if extra_cost > 0 {
                            taxed.symbols.push(crate::mana::ManaSymbol::Generic(extra_cost));
                        }
                        card.may_play_until = Some(crate::card::MayPlayPermission {
                            player: owner,
                            granted_turn: turn,
                            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                            exile_after: false,
                        });
                        card.granted_alt_cast_cost_eot = Some(taxed);
                        self.exile.push(card);
                        events.push(GameEvent::PermanentExiled { card_id: *cid });
                    }
                }
                Ok(events)
            }
            PendingEffectState::ChooseCreatureTypePending { target_id } => {
                let DecisionAnswer::CreatureType(ct) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                if let Some(card) = self.battlefield_find_mut(target_id) {
                    card.chosen_creature_type = Some(*ct);
                }
                Ok(Vec::new())
            }
            PendingEffectState::PutFromZonesPending { player } => {
                let DecisionAnswer::Search(chosen_id) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                if let Some(cid) = chosen_id {
                    let card = if let Some(card) =
                        Self::take_card(&mut self.players[player].hand, *cid)
                    {
                        Some(card)
                    } else {
                        // Grafdigger's Cage / Soulless Jailer — locked
                        // graveyard cards can't enter the battlefield (hand
                        // picks are unaffected).
                        let gy_ok = self.players[player]
                            .graveyard
                            .iter()
                            .find(|c| c.id == *cid)
                            .is_some_and(|c| {
                                !self.battlefield_entry_from_zone_blocked(
                                    &c.definition,
                                    crate::card::Zone::Graveyard,
                                )
                            });
                        if gy_ok {
                            Self::take_card(&mut self.players[player].graveyard, *cid)
                        } else {
                            None
                        }
                    };
                    if let Some(card) = card {
                        let dest = crate::effect::ZoneDest::Battlefield {
                            controller: crate::effect::PlayerRef::Seat(player),
                            tapped: false,
                        };
                        self.place_card_in_dest(card, player, &dest, &mut events);
                    }
                }
                Ok(events)
            }
            PendingEffectState::NameDiscardMatchingPending { who } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                let matching: Vec<CardId> = self.players[who]
                    .hand
                    .iter()
                    .filter(|c| c.definition.name == name)
                    .map(|c| c.id)
                    .collect();
                for cid in matching {
                    self.discard_card(who, cid, &mut events);
                }
                Ok(events)
            }
            PendingEffectState::NameRevealTopPending { player, count } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let mut events = vec![];
                let revealed: Vec<CardId> = self.players[player]
                    .library
                    .iter()
                    .take(count)
                    .map(|c| c.id)
                    .collect();
                for id in revealed {
                    let Some(card) = Self::take_card(&mut self.players[player].library, id)
                    else {
                        continue;
                    };
                    let matches = card.definition.name == name;
                    if matches {
                        self.players[player].hand.push(card);
                    } else {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                Ok(events)
            }
            PendingEffectState::NameCardPending { target_id } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                if !name.is_empty() {
                    if let Some(card) = self.find_card_anywhere_mut(target_id) {
                        card.named_card = Some(name.clone());
                    }
                    // Also record on the per-resolution scratchpad so a
                    // following `NamedBySource` reveal can match even when the
                    // naming source is a resolving spell held off-zone.
                    self.named_card_this_resolution = Some(name.clone());
                }
                Ok(Vec::new())
            }
            PendingEffectState::OpponentNameLockPending { caster } => {
                let DecisionAnswer::NamedCard(name) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                if !name.is_empty()
                    && let Some(pl) = self.players.get_mut(caster)
                    && !pl.opponents_cant_cast_named.contains(name)
                {
                    pl.opponents_cant_cast_named.push(name.clone());
                }
                Ok(Vec::new())
            }
            // ── Stash-and-rerun answers ──────────────────────────────────
            // These five suspend with the *originating effect* re-queued as
            // the continuation; the apply step only validates/sanitises the
            // answer and stashes it for the re-run to consume (see
            // `GameState.stashed_resolution_answer`).
            PendingEffectState::ModesAnswerPending { num_modes } => {
                let DecisionAnswer::Modes(v) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let sane: Vec<u8> =
                    v.iter().copied().filter(|&i| (i as usize) < num_modes).collect();
                self.stashed_resolution_answer = Some(DecisionAnswer::Modes(sane));
                Ok(Vec::new())
            }
            PendingEffectState::ModeAnswerPending { num_modes } => {
                let DecisionAnswer::Mode(i) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                let sane = (*i).min(num_modes.saturating_sub(1));
                self.stashed_resolution_answer = Some(DecisionAnswer::Mode(sane));
                Ok(Vec::new())
            }
            PendingEffectState::AmountAnswerPending { max } => {
                let DecisionAnswer::Amount(n) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.stashed_resolution_answer = Some(DecisionAnswer::Amount((*n).min(max)));
                Ok(Vec::new())
            }
            PendingEffectState::MayDoAnswerPending => {
                let DecisionAnswer::Bool(b) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.stashed_resolution_answer = Some(DecisionAnswer::Bool(*b));
                Ok(Vec::new())
            }
            PendingEffectState::SeatBoolAnswerPending { .. } => {
                let DecisionAnswer::Bool(b) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.resolution_answer_log.push(DecisionAnswer::Bool(*b));
                Ok(Vec::new())
            }
            PendingEffectState::DivisionAnswerPending => {
                let DecisionAnswer::DamageDivision(v) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                // Raw stash — the re-run renormalises wrong length/sum.
                self.stashed_resolution_answer =
                    Some(DecisionAnswer::DamageDivision(v.clone()));
                Ok(Vec::new())
            }
            PendingEffectState::CreatureTypeAnswerPending => {
                let DecisionAnswer::CreatureType(ct) = answer else {
                    return Err(GameError::DecisionAnswerMismatch);
                };
                self.stashed_resolution_answer = Some(DecisionAnswer::CreatureType(*ct));
                Ok(Vec::new())
            }
        }
    }

    /// Heuristic candidates for a `ChooseCreatureType` decision, rendered by
    /// the client as pick buttons: every creature type on the battlefield,
    /// in any graveyard, or among `chooser`'s own hand and library (their
    /// deck is known to them; opponents' hidden zones are excluded so the
    /// suggestion list can't leak), most frequent first, padded with tribal
    /// staples and capped to keep the modal scannable.
    pub(crate) fn creature_type_suggestions(
        &self,
        chooser: usize,
    ) -> Vec<crate::card::CreatureType> {
        use crate::card::CreatureType;
        let mut counts: std::collections::HashMap<CreatureType, usize> =
            std::collections::HashMap::new();
        let public = self
            .battlefield
            .iter()
            .chain(self.players.iter().flat_map(|p| p.graveyard.iter()));
        let own = self
            .players
            .get(chooser)
            .into_iter()
            .flat_map(|p| p.hand.iter().chain(p.library.iter()));
        for c in public.chain(own) {
            for &ct in &c.definition.subtypes.creature_types {
                *counts.entry(ct).or_insert(0) += 1;
            }
        }
        let mut ranked: Vec<(CreatureType, usize)> = counts.into_iter().collect();
        ranked.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)))
        });
        let mut out: Vec<CreatureType> = ranked.into_iter().map(|(ct, _)| ct).collect();
        for ct in [
            CreatureType::Human,
            CreatureType::Elf,
            CreatureType::Goblin,
            CreatureType::Zombie,
            CreatureType::Merfolk,
            CreatureType::Dragon,
            CreatureType::Angel,
            CreatureType::Demon,
            CreatureType::Soldier,
            CreatureType::Wizard,
        ] {
            if !out.contains(&ct) {
                out.push(ct);
            }
        }
        out.truncate(24);
        out
    }

    /// Resolve a spell's effect tree. On suspension, installs a
    /// `pending_decision` and returns events accumulated so far. `override_effect`
    /// is used on resume to continue with whatever Seq tail was left after the
    /// suspending effect — pass `None` for the initial resolution and `Some(...)`
    /// when continuing from `submit_decision`.
    //
    // The argument list is wide because the spell-state quartet (target, mode,
    // x_value, converged_value) must be preserved across suspend/resume so the
    // spell can re-run its effect tree with the original cast-time choices.
    // The two callers (initial cast in `stack.rs` and resume in
    // `submit_decision`) both hand off these fields directly from a
    // `StackItem::Spell` / `ResumeContext::Spell`, so wrapping them in a
    // struct doesn't reduce coupling at the call sites.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn continue_spell_resolution(
        &mut self,
        card: CardInstance,
        caster: usize,
        target: Option<Target>,
        additional_targets: Vec<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        override_effect: Option<Effect>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let effect = override_effect.unwrap_or_else(|| {
            if let Some(half) = card.alt_spell_half() {
                // CR 715 / 702.183 — resolve the Adventure/Omen half's effect,
                // not the creature body.
                half.effect.clone()
            } else if card.gift_promised
                && let Some(gift) = card.definition.gift.as_ref()
            {
                // CR 702.165 — the gift was promised: resolve the enhanced
                // effect (which itself bestows the gift on the opponent).
                gift.gifted_effect.clone()
            } else if let (Some(half), Some(split)) =
                (card.split_cast, card.definition.split.as_ref())
            {
                // CR 709 — resolve the chosen half. Left (0) is the main body;
                // right (1) is `split.right`. Fused (2) resolves the left body
                // here, then the right half runs in a second pass below with
                // its own target (slot 0 of `additional_targets`).
                match half {
                    1 => split.right.effect.clone(),
                    _ => card.definition.effect.clone(),
                }
            } else {
                card.definition.effect.clone()
            }
        });
        // CR 608.2b — a spell whose single target is illegal as it tries to
        // resolve doesn't resolve. Applies when the primary target was a
        // battlefield permanent at cast time (zone-loose filters aimed at
        // graveyard cards are unaffected): the spell fizzles if the target
        // left the battlefield, stopped matching the spell's target filter,
        // or became illegal to target (granted Hexproof/Shroud). Token
        // copies additionally keep the bare filter re-check.
        // CR 702.172 — Spree spells validate each chosen mode's target at
        // resolution (per-mode), not through this single-`mode` fizzle path;
        // the chosen modes aren't encoded in `mode`, so a slot-0 filter here
        // could belong to an unchosen mode. Skip the blanket fizzle for Spree.
        let is_spree = matches!(effect, Effect::Spree { .. } | Effect::Tiered { .. });
        if !is_spree
            && additional_targets.is_empty()
            && let Some(t) = &target
        {
            let filter_fails = |g: &Self| {
                effect
                    .target_filter_for_slot_in_mode_kicked(0, Some(mode), card.kicked)
                    .is_some_and(|f| {
                        let f = f.resolve_x(x_value);
                        !g.evaluate_requirement_static(&f, t, caster, Some(card.id))
                    })
            };
            let fizzled = if card.cast_target_was_battlefield
                && let Target::Permanent(tid) = t
            {
                self.battlefield_find(*tid).is_none()
                    || filter_fails(self)
                    || self.check_target_legality_with_source(t, caster, Some(card.id)).is_err()
            } else {
                card.is_token && filter_fails(self)
            };
            if fizzled {
                // A fizzled token copy ceases to exist (already off the
                // stack); a real card is countered into its owner's
                // graveyard — except a flashbacked/aftermath cast, whose
                // CR 702.34d exile rider applies wherever it leaves the
                // stack, so a fizzle can't make it re-flashbackable.
                let mut events = Vec::new();
                if !card.is_token {
                    if card.cast_via_flashback {
                        self.exile.push(card);
                    } else {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                return Ok(events);
            }
        } else if !is_spree
            && card.cast_target_was_battlefield
            && let Some(t0) = &target
        {
            // CR 608.2b — a multi-target spell fizzles only if EVERY target
            // is illegal on resolution; effects already skip individual
            // missing targets. Scoped to battlefield-aimed casts (slot 0 was
            // a battlefield permanent at cast time) so zone-loose multi-
            // target spells (graveyard returns) are unaffected.
            let slot_illegal = |g: &Self, slot: u8, t: &Target| {
                let gone = matches!(t, Target::Permanent(tid)
                    if g.battlefield_find(*tid).is_none());
                let filter_fail = effect
                    .target_filter_for_slot_in_mode_kicked(slot, Some(mode), card.kicked)
                    .is_some_and(|f| {
                        let f = f.resolve_x(x_value);
                        !g.evaluate_requirement_static(&f, t, caster, Some(card.id))
                    });
                gone
                    || filter_fail
                    || g.check_target_legality_with_source(t, caster, Some(card.id)).is_err()
            };
            let all_illegal = slot_illegal(self, 0, t0)
                && additional_targets
                    .iter()
                    .enumerate()
                    .all(|(i, t)| slot_illegal(self, i as u8 + 1, t));
            if all_illegal {
                let mut events = Vec::new();
                if !card.is_token {
                    if card.cast_via_flashback {
                        self.exile.push(card); // CR 702.34d
                    } else {
                        self.route_to_graveyard(card, &mut events);
                    }
                }
                return Ok(events);
            }
        }
        let mut ctx = EffectContext::for_spell_with_source_and_origin(
            card.id,
            card.definition.name,
            caster,
            target.clone(),
            additional_targets.clone(),
            mode,
            x_value,
            converged_value,
            mana_spent,
            card.cast_from_hand,
        );
        ctx.kicked = card.kicked;
        ctx.bargained = card.bargained;
        ctx.cast_via_mayhem = card.cast_via_mayhem;
        ctx.cast_via_waterbend = card.cast_via_waterbend;
        ctx.entwined = card.entwined;
        ctx.spree_modes = card.spree_modes.clone();
        ctx.mana_spent_by_color = card.cast_mana_spent_by_color.clone();
        // Stamp the resolving spell's identity so source-aware damage
        // replacements (Torbran) can read its controller/colors while the
        // card is in no visible zone.
        let prev_src = self.resolving_source.replace((
            card.id,
            caster,
            card.definition.printed_colors(),
        ));
        let res = self.resolve_effect(&effect, &ctx);
        self.resolving_source = prev_src;
        let mut events = res?;
        // CR 709 / 702.102 — a fused split cast resolves its right half in a
        // second pass, reading its target from `additional_targets` slot 0
        // (the left half consumed `target`). Fusable halves are single-target.
        if card.split_cast == Some(2)
            && let Some(split) = card.definition.split.as_ref()
        {
            let right_effect = split.right.effect.clone();
            let right_ctx = EffectContext::for_spell_with_source_and_origin(
                card.id,
                card.definition.name,
                caster,
                additional_targets.first().cloned(),
                Vec::new(),
                mode,
                x_value,
                converged_value,
                mana_spent,
                card.cast_from_hand,
            );
            let mut right_events = self.resolve_effect(&right_effect, &right_ctx)?;
            events.append(&mut right_events);
        }
        // CR 702.47b — spliced rules text resolves after the main spell's
        // effect; spliced effect `i` reads its target from
        // `additional_targets[i]`.
        for (i, spliced) in card.spliced_effects.clone().into_iter().enumerate() {
            let splice_ctx = EffectContext::for_spell_with_source_and_origin(
                card.id,
                card.definition.name,
                caster,
                additional_targets.get(i).cloned(),
                Vec::new(),
                mode,
                x_value,
                converged_value,
                mana_spent,
                card.cast_from_hand,
            );
            let mut splice_events = self.resolve_effect(&spliced, &splice_ctx)?;
            events.append(&mut splice_events);
        }
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Spell {
                    card: Box::new(card),
                    caster,
                    target,
                    additional_targets,
                    mode,
                    x_value,
                    converged_value,
                    mana_spent,
                    in_progress,
                    remaining,
                },
            });
            return Ok(events);
        }
        // CR 702.50 — Epic: on resolution the caster snapshots the spell
        // (copied at each of their upkeeps) and can't cast spells for the
        // rest of the game. Copies don't carry the epic ability (702.50a).
        if !card.is_token && card.definition.keywords.contains(&crate::card::Keyword::Epic) {
            self.players[caster].epic_spells.push(crate::player::EpicSpell {
                name: card.definition.name.to_string(),
                target: target.clone(),
                additional_targets: additional_targets.clone(),
                mode: Some(mode),
                x_value,
            });
        }
        // Rebound: if this card has Keyword::Rebound and was cast from
        // hand, exile it instead of sending it to the graveyard, and
        // schedule a delayed trigger at the caster's next upkeep that
        // re-runs the spell's effect with a fresh auto-target.
        if card.cast_from_hand
            && card.definition.keywords.contains(&crate::card::Keyword::Rebound)
        {
            use crate::game::types::{DelayedKind, DelayedTrigger};
            let source = card.id;
            let body = card.definition.effect.clone();
            self.delayed_triggers.push(DelayedTrigger {
                controller: caster,
                source,
                kind: DelayedKind::YourNextUpkeep,
                effect: body,
                target: None, // re-pick at fire time
                bound_token: None,
                fires_once: true,
            });
            self.exile.push(card);
            return Ok(events);
        }
        // Flashback (CR 702.34d): a spell cast via its Flashback cost is
        // exiled on resolution instead of going to the graveyard.
        // `cast_flashback` sets `cast_via_flashback = true`; the
        // resolver consults that flag (it used to overload `kicked`,
        // which collided with cards that have both Kicker and Flashback).
        if card.cast_via_flashback {
            self.exile.push(card);
            return Ok(events);
        }
        // CR 701.x — "Then exile this spell" rider. Cards with
        // `exile_on_resolve = true` route to exile after resolution
        // instead of their owner's graveyard. Used by Awaken the Ages,
        // Divergent Equation, Settle the Score's printed rider.
        // Bump the owner's `cards_exiled_this_turn` so the Ennis-style
        // "cards put into exile this turn" payoffs see the exile.
        if card.definition.exile_on_resolve {
            self.players[caster].cards_exiled_this_turn =
                self.players[caster].cards_exiled_this_turn.saturating_add(1);
            self.exile.push(card);
            return Ok(events);
        }
        // CR 702.55 — Haunt. `Effect::HauntCreature` set `haunt_pending` to the
        // creature this resolving instant/sorcery should haunt. Exile the spell
        // card (not the graveyard) and register the death-watch delayed trigger.
        if let Some((haunted, body)) = self.haunt_pending.take() {
            use crate::game::types::{DelayedKind, DelayedTrigger};
            let src = card.id;
            self.exile.push(card);
            self.delayed_triggers.push(DelayedTrigger {
                controller: caster,
                source: src,
                kind: DelayedKind::WhenHauntedCreatureDies(haunted),
                effect: body,
                target: None,
                bound_token: None,
                fires_once: true,
            });
            return Ok(events);
        }
        // CR 702.46 — Cipher. `Effect::Cipher` set `cipher_encode_pending` to
        // the creature this spell should be encoded on. Route the card to exile
        // with `encoded_on` stamped instead of the graveyard.
        if let Some(creature) = self.cipher_encode_pending.take() {
            let mut card = card;
            card.encoded_on = Some(creature);
            self.exile.push(card);
            return Ok(events);
        }
        // Beacon cycle: "Shuffle this card into its owner's library."
        // `Effect::ShuffleSelfIntoLibrary` flagged the resolving spell — route
        // it to its owner's library and shuffle instead of the graveyard.
        if self.shuffle_resolving_spell_into_library {
            self.shuffle_resolving_spell_into_library = false;
            use rand::seq::SliceRandom;
            let owner = card.owner;
            self.players[owner].library.push(card);
            self.players[owner].library.shuffle(&mut rand::rng());
            return Ok(events);
        }
        // Revel in Silence's "exile this" rider.
        if self.exile_resolving_spell {
            self.exile_resolving_spell = false;
            self.exile.push(card);
            return Ok(events);
        }
        // Journey to the Oracle's "return this to its owner's hand" rider.
        if self.return_resolving_spell_to_hand {
            self.return_resolving_spell_to_hand = false;
            let owner = card.owner;
            self.players[owner].hand.push(card);
            return Ok(events);
        }
        // Buyback (CR 702.27e): a spell cast paying its buyback cost returns
        // to its owner's hand instead of the graveyard as it resolves.
        if card.bought_back {
            let owner = card.owner;
            self.players[owner].hand.push(card);
            return Ok(events);
        }
        // CR 702.127e — an Aftermath half (right half cast from the graveyard)
        // is exiled on resolution rather than returning to the graveyard.
        if card.split_cast == Some(1)
            && card.definition.split.as_ref().is_some_and(|s| s.aftermath)
        {
            self.exile.push(card);
            return Ok(events);
        }
        // CR 715 — an adventure spell goes to exile (not the graveyard) on
        // resolution, marked so its creature half can be cast from exile.
        if card.adventuring {
            let mut card = card;
            card.adventuring = false;
            card.on_adventure = true;
            self.exile.push(card);
            return Ok(events);
        }
        // CR 728.1a — a spell that ended the turn is exiled along with the
        // rest of the stack instead of going to the graveyard (Day's
        // Undoing). The flag stays set; `resolve_top_of_stack` consumes it.
        if self.end_turn_requested {
            self.exile.push(card);
            return Ok(events);
        }
        // CR 614.6 — an instant/sorcery bound for the graveyard is exiled
        // instead under Rest in Peace / Leyline of the Void.
        self.route_to_graveyard(card, &mut events);
        Ok(events)
    }

    /// Resolve a triggered ability's effect tree, carrying the
    /// trigger's "source entity" (the just-cast spell, the dying
    /// creature, etc.) into `ctx.trigger_source`. Used by spell-cast
    /// triggers whose body looks up the cast spell on the stack
    /// (e.g. Aziza's Magecraft copy, Conciliator's Duelist's Repartee
    /// exile-target). When `trigger_source_ent` is `None`, falls back
    /// to the legacy behavior (trigger_source = source permanent).
    ///
    /// `event_amount` carries the firing event's amount (life gained,
    /// life lost, damage dealt, …) so trigger bodies can read it via
    /// `Value::TriggerEventAmount` — used by Light of Promise's
    /// "Whenever you gain life, put that many +1/+1 counters …".
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn continue_trigger_resolution_with_source(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
        mode: usize,
        x_value: u32,
        converged_value: u32,
        mana_spent: u32,
        trigger_source_ent: Option<crate::game::effects::EntityRef>,
        event_amount: u32,
        additional_targets: Vec<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        // Event-amount-relative filters re-checked at resolution
        // (ManaValueLessThanEventAmount) read this scratch.
        self.trigger_event_amount_scratch = event_amount;
        // CR 608.2b — if the trigger's stored sole target is no longer legal
        // at resolution (left the zone, stopped matching the filter), the
        // ability doesn't resolve: none of its effects happen. It must NOT
        // re-aim at a fresh target.
        let resolved_target = match target.as_ref() {
            Some(t) => match effect.target_filter_for_slot(0).map(|f| f.resolve_x(x_value)) {
                Some(filter)
                    if !self.evaluate_requirement_static(&filter, t, controller, Some(source)) =>
                {
                    return Ok(vec![]);
                }
                _ => Some(t.clone()),
            },
            None => None,
        };
        let mut ctx =
            EffectContext::for_trigger(source, controller, resolved_target.clone(), mode);
        // Append slot-1+ targets (two-target activated abilities) after slot 0.
        ctx.targets.extend(additional_targets.iter().cloned());
        ctx.x_value = x_value;
        ctx.converged_value = converged_value;
        // CR 702.32 — an ETB/other trigger on a permanent reads the
        // source's `kicked` flag so "when ~ enters, if it was kicked, …"
        // riders (Goblin Bushwhacker) can branch on `SpellWasKicked`.
        if let Some(src) = self.battlefield.iter().find(|c| c.id == source) {
            ctx.kicked = src.kicked;
            ctx.bargained = src.bargained;
            // "Escapes with …" / cast-zone riders (Tizerus Charger) read
            // the source's cast zone through `Predicate::CastFromGraveyard`.
            ctx.cast_from_hand = src.cast_from_hand;
        }
        if let Some(ts) = trigger_source_ent {
            ctx.trigger_source = Some(ts);
        }
        ctx.mana_spent = mana_spent;
        ctx.event_amount = event_amount;
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Trigger {
                    source,
                    controller,
                    target,
                    mode,
                    in_progress,
                    remaining,
                    x_value,
                    converged_value,
                    mana_spent,
                    trigger_source_ent,
                    event_amount,
                    additional_targets,
                },
            });
        }
        Ok(events)
    }

    /// Resolve an activated ability's effect tree.
    pub(crate) fn continue_ability_resolution(
        &mut self,
        source: CardId,
        controller: usize,
        effect: crate::effect::Effect,
        target: Option<Target>,
    ) -> Result<Vec<GameEvent>, GameError> {
        let ctx = EffectContext::for_ability(source, controller, target.clone());
        let events = self.resolve_effect(&effect, &ctx)?;
        if let Some((decision, in_progress, remaining)) = self.suspend_signal.take() {
            self.pending_decision = Some(PendingDecision {
                decision,
                resume: ResumeContext::Ability {
                    source,
                    controller,
                    target,
                    in_progress,
                    remaining,
                },
            });
        }
        Ok(events)
    }

    /// Evaluate whether `target` satisfies `req` given the current game state.
    ///
    /// `controller` is the player who controls the spell or ability (used for
    /// `ControlledByYou` / `ControlledByOpponent` checks).
    pub fn evaluate_requirement(
        &self,
        req: &SelectionRequirement,
        target: &Target,
        controller: usize,
    ) -> bool {
        self.evaluate_requirement_static(req, target, controller, None)
    }

    pub(crate) fn battlefield_find(&self, id: CardId) -> Option<&CardInstance> {
        self.battlefield.iter().find(|c| c.id == id)
    }

    /// CR 700.4 — true when a death's graveyard placement was replaced away
    /// (Finality/void exile, Rest in Peace, Valentin, Pulmonic Sliver's
    /// library-top): the card never reached a graveyard, so it never "died"
    /// and dies-watchers must not fire. Checked at dispatch time, after the
    /// removal chokepoint has resolved the redirect.
    pub(crate) fn death_was_replaced(&self, card_id: CardId) -> bool {
        self.exile.iter().any(|c| c.id == card_id)
            || self
                .players
                .iter()
                .any(|p| p.library.iter().any(|c| c.id == card_id))
    }

    /// The firing event's magnitude for `Value::TriggerEventAmount` /
    /// `ManaValueLessThanEventAmount`. Mostly the event payload's `amount`;
    /// for died events it's the dying card's mana value (Scrap Trawler's
    /// "lesser mana value than that artifact"), read from the death
    /// snapshot cache (tokens are already gone from every zone).
    pub(crate) fn event_amount_for(&self, ev: &GameEvent) -> u32 {
        match ev {
            GameEvent::CreatureDied { card_id } => self
                .died_card_snapshots
                .get(card_id)
                .or_else(|| self.find_card_anywhere(*card_id))
                .map(|c| c.definition.cost.cmc())
                .unwrap_or(0),
            // "Where X is that spell's mana value" riders (Shark Typhoon).
            GameEvent::SpellCast { card_id, .. } => self
                .stack
                .iter()
                .find_map(|item| match item {
                    StackItem::Spell { card, .. } if card.id == *card_id => {
                        Some(card.definition.cost.cmc())
                    }
                    _ => None,
                })
                .or_else(|| self.find_card_anywhere(*card_id).map(|c| c.definition.cost.cmc()))
                .unwrap_or(0),
            GameEvent::CardCycled { x, .. } => *x,
            // Nicanzil: 1 when a land was explored, 0 for a nonland.
            GameEvent::Explored { explored_land, .. } => *explored_land as u32,
            _ => event_amount(ev),
        }
    }

    pub(crate) fn battlefield_find_mut(&mut self, id: CardId) -> Option<&mut CardInstance> {
        self.battlefield.iter_mut().find(|c| c.id == id)
    }

    /// Single funnel for on-battlefield control changes (steals, exchanges,
    /// duration reverts). Applies CR 302.6 summoning sickness and re-arms echo
    /// (CR 702.29b — echo is owed again once it "came under your control").
    /// Returns the previous controller when control actually changed.
    pub(crate) fn change_control(&mut self, id: CardId, new_ctrl: usize) -> Option<usize> {
        let c = self.battlefield_find_mut(id)?;
        if c.controller == new_ctrl {
            return None;
        }
        let prev = c.controller;
        c.controller = new_ctrl;
        c.summoning_sick = true;
        c.echo_paid = false;
        Some(prev)
    }

    /// Atomically scan-and-remove a card from a zone by id. Prefer this over
    /// the `position(..)` + `remove(pos)` two-step: any effect, trigger, or
    /// payment that mutates the zone between the scan and the remove silently
    /// invalidates a stored index (removing the wrong card or panicking),
    /// while re-locating by id at removal time cannot.
    pub(crate) fn take_card(zone: &mut Vec<CardInstance>, id: CardId) -> Option<CardInstance> {
        let pos = zone.iter().position(|c| c.id == id)?;
        Some(zone.remove(pos))
    }

    /// Look up a card instance by id across every visible zone in
    /// resolution order — battlefield → each player's graveyard / hand /
    /// library → exile → stack. General-purpose helper for predicates
    /// or effects that need to introspect a card regardless of where
    /// it currently lives.
    pub(crate) fn find_card_anywhere(&self, id: CardId) -> Option<&CardInstance> {
        if let Some(c) = self.battlefield_find(id) {
            return Some(c);
        }
        for p in &self.players {
            if let Some(c) = p.graveyard.iter().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.hand.iter().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.library.iter().find(|c| c.id == id) {
                return Some(c);
            }
        }
        if let Some(c) = self.exile.iter().find(|c| c.id == id) {
            return Some(c);
        }
        for si in &self.stack {
            if let crate::game::types::StackItem::Spell { card, .. } = si
                && card.id == id
            {
                return Some(card);
            }
        }
        None
    }

    /// Mutable variant of `find_card_anywhere` — walks battlefield,
    /// each player's hand/library/graveyard, and exile (in that order).
    /// Used by `Effect::GrantMayPlay` to stamp `may_play_until` on a
    /// card regardless of where the granting effect happens to find it.
    pub(crate) fn find_card_anywhere_mut(
        &mut self,
        id: CardId,
    ) -> Option<&mut CardInstance> {
        if self.battlefield.iter().any(|c| c.id == id) {
            return self.battlefield.iter_mut().find(|c| c.id == id);
        }
        for p in &mut self.players {
            if let Some(c) = p.hand.iter_mut().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.graveyard.iter_mut().find(|c| c.id == id) {
                return Some(c);
            }
            if let Some(c) = p.library.iter_mut().find(|c| c.id == id) {
                return Some(c);
            }
        }
        if let Some(c) = self.exile.iter_mut().find(|c| c.id == id) {
            return Some(c);
        }
        // A spell resolving its own effect (Spoils of the Vault's NameCard)
        // is still on the stack.
        for si in &mut self.stack {
            if let crate::game::types::StackItem::Spell { card, .. } = si
                && card.id == id
            {
                return Some(card);
            }
        }
        None
    }

    /// Look up which zone a card currently occupies. Returns `None` if
    /// the card isn't in any visible zone (battlefield, hand, library,
    /// graveyard, exile, stack). Used by the cast-from-zone path to
    /// confirm the card is still in the expected zone before lifting it.
    pub(crate) fn find_card_zone(&self, id: CardId) -> Option<crate::card::Zone> {
        use crate::card::Zone;
        if self.battlefield.iter().any(|c| c.id == id) {
            return Some(Zone::Battlefield);
        }
        for p in &self.players {
            if p.hand.iter().any(|c| c.id == id) {
                return Some(Zone::Hand);
            }
            if p.graveyard.iter().any(|c| c.id == id) {
                return Some(Zone::Graveyard);
            }
            if p.library.iter().any(|c| c.id == id) {
                return Some(Zone::Library);
            }
        }
        if self.exile.iter().any(|c| c.id == id) {
            return Some(Zone::Exile);
        }
        None
    }

    /// Look up the owner (seat index) of `id` across every public zone:
    /// battlefield, each player's graveyard, each player's hand, the
    /// stack, and exile. Returns `None` if no card with that id exists
    /// in any visible zone. Used by `PlayerRef::OwnerOf(...)` resolution
    /// to find the original owner of a target whose card has changed
    /// zones (e.g. destroyed and now in graveyard) by the time the
    /// owner-targeted effect resolves.
    pub(crate) fn find_card_owner(&self, id: CardId) -> Option<usize> {
        if let Some(c) = self.battlefield_find(id) {
            return Some(c.owner);
        }
        for (i, p) in self.players.iter().enumerate() {
            if p.graveyard.iter().any(|c| c.id == id)
                || p.hand.iter().any(|c| c.id == id)
                || p.library.iter().any(|c| c.id == id)
            {
                return Some(i);
            }
        }
        if self.exile.iter().any(|c| c.id == id) {
            return self.exile.iter().find(|c| c.id == id).map(|c| c.owner);
        }
        // Stack: a spell mid-resolution is on the stack but not yet in any
        // player's persistent zone. The spell's caster is its current
        // controller; `card.owner` is the printed owner (typically equal to
        // the caster, except for stolen spells like Wandering Archaic
        // copies). Cards on the stack via StackItem::Spell are findable here.
        for item in &self.stack {
            if let crate::game::types::StackItem::Spell { card, .. } = item
                && card.id == id
            {
                return Some(card.owner);
            }
        }
        None
    }

    /// Look up the caster (current controller) of a stack-resident spell
    /// by card id. Used by `PlayerRef::ControllerOf` to resolve "this
    /// spell's controller" — distinct from `find_card_owner`, which
    /// returns the printed `owner` even on the stack. Returns `None` if
    /// `id` is not currently a spell on the stack.
    pub(crate) fn stack_caster_for_card(&self, id: CardId) -> Option<usize> {
        for item in &self.stack {
            if let crate::game::types::StackItem::Spell { card, caster, .. } = item
                && card.id == id
            {
                return Some(*caster);
            }
        }
        None
    }

    /// Returns true if the permanent `id` has `kw` after all layer effects are applied.
    /// Falls back to `false` if the permanent is not on the battlefield.
    pub(crate) fn permanent_has_keyword(&self, id: CardId, kw: &Keyword) -> bool {
        self.computed_permanent(id)
            .is_some_and(|c| c.keywords.contains(kw))
    }
}

/// Whether `ev` is already handled by a hardcoded trigger site for the
/// given `spec.scope`. Dispatched triggers should skip events for which
/// the hardcoded site would already fire — but other scopes still need
/// the unified dispatcher.
///
/// Coverage of hardcoded sites:
/// - `EnterBattlefield` + `SelfSource` → `fire_self_etb_triggers`
/// - `Attacks` + `SelfSource` → `declare_attackers`
/// - `CreatureDied` + `SelfSource` → SBA-time hook in remove-to-graveyard
/// - `SpellCast` (any scope) → `collect_self_cast_triggers` (SelfSource)
///   plus `fire_spell_cast_triggers` (YourControl/AnyPlayer)
/// - `StepBegins` (any scope) → `fire_step_triggers`
///
/// Non-SelfSource scopes for ETB / Attacks / CreatureDied are NOT covered
/// by a hardcoded site and need the unified dispatcher (Temur Ascendancy's
/// "another creature you control enters" trigger, etc.).
fn is_event_hardcoded(ev: &GameEvent, spec: &crate::effect::EventSpec) -> bool {
    use crate::effect::EventScope;
    match ev {
        GameEvent::PermanentEntered { .. } => matches!(spec.scope, EventScope::SelfSource),
        // SelfSource mutate triggers are pushed inline by `resolve_top_of_stack`.
        GameEvent::Mutated { .. } => matches!(spec.scope, EventScope::SelfSource),
        GameEvent::AttackerDeclared(_) => matches!(spec.scope, EventScope::SelfSource),
        GameEvent::CreatureDied { .. } => matches!(spec.scope, EventScope::SelfSource),
        GameEvent::SpellCast { .. } => true,
        GameEvent::StepChanged(_) => true,
        _ => false,
    }
}

/// Extract the per-event scalar amount carried by `event` — the life
/// gained on a `LifeGained`, life lost on a `LifeLost`, the count of
/// cards milled / drawn, etc. Threaded into `EffectContext.event_amount`
/// via the trigger dispatcher so trigger bodies can read it via
/// `Value::TriggerEventAmount`. Returns 0 for events that don't carry
/// a scalar amount (CreatureDied, PermanentEntered, …).
fn event_amount(event: &GameEvent) -> u32 {
    match event {
        GameEvent::LifeGained { amount, .. }
        | GameEvent::LifeLost { amount, .. }
        | GameEvent::DamageDealt { amount, .. }
        | GameEvent::PoisonAdded { amount, .. }
        | GameEvent::EnergyGained { amount, .. } => *amount,
        GameEvent::DiscardedBatch { count, .. } => *count,
        GameEvent::CounterAdded { count, .. } => *count,
        GameEvent::Discovered { value, .. } => *value,
        GameEvent::Expended { total, .. } => *total,
        // CR 706.4 — the greatest result rolled, for "roll a 5 or higher"
        // result-gated triggers (`Predicate::DieResultAtLeast`).
        GameEvent::DiceRolled { high, .. } => *high as u32,
        _ => 0,
    }
}

/// True if a card definition is colorless from its printed characteristics:
/// it has Devoid (CR 702.114) or its mana cost carries no colored pips.
/// Used by the `ColorlessCreaturesControlled` dynamic-P/T formula; avoids the
/// layer-pass circularity of reading computed colors during the same recompute.
fn is_colorless_by_cost(def: &crate::card::CardDefinition) -> bool {
    use crate::mana::ManaSymbol;
    if def.keywords.contains(&crate::card::Keyword::Devoid) {
        return true;
    }
    !def.cost.symbols.iter().any(|s| {
        matches!(
            s,
            ManaSymbol::Colored(_)
                | ManaSymbol::Hybrid(_, _)
                | ManaSymbol::Phyrexian(_)
                | ManaSymbol::MonoHybrid(_, _)
        )
    })
}

// ── Static ability conversion ─────────────────────────────────────────────────

/// CR 606 — the loyalty abilities a planeswalker can actually activate:
/// printed ones, plus those granted by friendly statics (Kasmina, Enigma
/// Sage's sharing; Ichormoon Gauntlet's fixed "[0]: Proliferate" grants) at
/// indices past the printed count. Shared by `activate_loyalty_ability` and
/// the server view so networked seats see exactly what they can use.
pub(crate) fn effective_loyalty_abilities(
    card: &CardInstance,
    battlefield: &[CardInstance],
) -> Vec<crate::effect::LoyaltyAbility> {
    let mut abilities = card.definition.loyalty_abilities.clone();
    for c in battlefield {
        if c.id != card.id
            && c.controller == card.controller
            && c.definition.static_abilities.iter().any(|sa| {
                matches!(
                    sa.effect,
                    crate::effect::StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities
                )
            })
        {
            abilities.extend(c.definition.loyalty_abilities.iter().cloned());
        }
    }
    for c in battlefield {
        if c.controller != card.controller {
            continue;
        }
        for sa in &c.definition.static_abilities {
            if let crate::effect::StaticEffect::PlaneswalkersHaveLoyaltyAbilities {
                abilities: granted,
            } = &sa.effect
            {
                abilities.extend(granted.iter().cloned());
            }
        }
    }
    abilities
}

/// Convert a `StaticAbility` from a source permanent into `ContinuousEffect`s.
/// Takes the full `CardInstance` so Equipment/Aura abilities can use `attached_to`.
fn static_ability_to_effects(card: &CardInstance, timestamp: u64) -> Vec<ContinuousEffect> {
    card.definition
        .static_abilities
        .iter()
        .flat_map(|sa| static_effect_to_effects(&sa.effect, card, timestamp))
        .collect()
}

/// Convert a single `StaticEffect` from `card` into layer continuous effects.
/// Split out of `static_ability_to_effects` so charge-gated Station bands
/// (CR 721.2a) can reuse the same conversion.
fn static_effect_to_effects(
    effect: &crate::effect::StaticEffect,
    card: &CardInstance,
    timestamp: u64,
) -> Vec<ContinuousEffect> {
    use crate::effect::StaticEffect;
    let source = card.id;

    {
        match effect {
            StaticEffect::PumpPT { applies_to, power, toughness } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(*power, *toughness),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::PumpPTPerOwnCreatureType { applies_to, per_power, per_toughness, max } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPtPerOwnCreatureType(
                            *per_power, *per_toughness, *max,
                        ),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::PumpPTPerCounterOnSource { applies_to, kind, per_power, per_toughness } => {
                let n = card.counter_count(*kind) as i32;
                if n == 0 {
                    return vec![];
                }
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::Modify),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::ModifyPowerToughness(
                            n * per_power,
                            n * per_toughness,
                        ),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantKeyword { applies_to, keyword } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(keyword.clone()),
                    }],
                    None => vec![],
                }
            }
            // Ward Sliver — the granted protection color comes off the
            // source's ETB `chosen_color` stamp; inert until chosen.
            StaticEffect::GrantProtectionFromChosenColor { applies_to } => {
                match (card.chosen_color, selector_to_affected(applies_to, card)) {
                    (Some(color), Some(affected)) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(Keyword::Protection(color)),
                    }],
                    _ => vec![],
                }
            }
            StaticEffect::LoseKeyword { applies_to, keyword } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::RemoveKeyword(keyword.clone()),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::CantHaveKeyword { applies_to, keyword } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::CantHaveKeyword(keyword.clone()),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantAllBasicLandTypes { applies_to } => {
                use crate::card::LandType;
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::SetLandTypes(vec![
                            LandType::Plains,
                            LandType::Island,
                            LandType::Swamp,
                            LandType::Mountain,
                            LandType::Forest,
                        ]),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::GrantAllColors { applies_to } => {
                use crate::mana::Color;
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L5Color,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::SetColors(vec![
                            Color::White,
                            Color::Blue,
                            Color::Black,
                            Color::Red,
                            Color::Green,
                        ]),
                    }],
                    None => vec![],
                }
            }
            // Dress Down — every creature loses all abilities (layer 6).
            StaticEffect::CreaturesLoseAllAbilities => vec![ContinuousEffect {
                timestamp,
                source,
                affected: AffectedPermanents::All {
                    controller: None,
                    card_types: vec![crate::card::CardType::Creature],
                    exclude_source: false,
                    color: None,
                    token: None,
                    colorless: false,
                },
                layer: Layer::L6Ability,
                sublayer: None,
                duration: EffectDuration::WhileSourceOnBattlefield,
                modification: Modification::RemoveAllAbilities,
            }],
            StaticEffect::SetBasePtForFilter { applies_to, power, toughness } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L7PowerTough,
                        sublayer: Some(PtSublayer::SetValue),
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::SetPowerToughness(*power, *toughness),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::AddCreatureTypeToMatching { applies_to, creature_type } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddCreatureType(*creature_type),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::AddCardTypeToMatching { applies_to, card_type } => {
                match selector_to_affected(applies_to, card) {
                    Some(affected) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected,
                        layer: Layer::L4Type,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddCardType(card_type.clone()),
                    }],
                    None => vec![],
                }
            }
            StaticEffect::LandTypeChanger { applies_to, land_type, replace }
            | StaticEffect::LandTypeChangerWhileCounters {
                applies_to,
                land_type,
                replace,
                ..
            } => {
                // The counter-gated variant only materializes while the source
                // carries the threshold (Zhao's conqueror counter).
                if let StaticEffect::LandTypeChangerWhileCounters { kind, n, .. } = effect
                    && card.counter_count(*kind) < *n
                {
                    return vec![];
                }
                match selector_to_affected(applies_to, card) {
                    Some(affected) => {
                        let mk = |layer, modification| ContinuousEffect {
                            timestamp,
                            source,
                            affected: affected.clone(),
                            layer,
                            sublayer: None,
                            duration: EffectDuration::WhileSourceOnBattlefield,
                            modification,
                        };
                        if *replace {
                            // Blood Moon — lose other land types + abilities;
                            // the intrinsic mana ability follows the type.
                            vec![
                                mk(Layer::L4Type, Modification::SetLandTypes(vec![*land_type])),
                                mk(Layer::L6Ability, Modification::RemoveAllAbilities),
                            ]
                        } else {
                            // Urborg — the type in addition.
                            vec![mk(Layer::L4Type, Modification::AddLandType(*land_type))]
                        }
                    }
                    None => vec![],
                }
            }
            StaticEffect::EntersTapped { .. }
            | StaticEffect::EntersTappedUnless { .. }
            | StaticEffect::LandsEnterUntapped
            | StaticEffect::LethalDamageByPower { .. }
            | StaticEffect::ExtraLandPerTurn
            | StaticEffect::CostReduction { .. }
            | StaticEffect::CostReductionPerControllerExperience { .. }
            | StaticEffect::CostReductionBySourcePower { .. }
            | StaticEffect::CostReductionWhile { .. }
            | StaticEffect::GraveyardCastCostReduction { .. }
            | StaticEffect::CostReductionDuringOpponentsTurn { .. }
            | StaticEffect::CostReductionNthSpell { .. }
            | StaticEffect::CostReductionFirstCreatureSpell { .. }
            | StaticEffect::CostReductionTargetingFilter { .. }
            | StaticEffect::AdditionalCostAfterFirstSpell { .. }
            | StaticEffect::AdditionalCost { .. }
            | StaticEffect::ControllerHasHexproof
            | StaticEffect::LandsTapColorlessOnly
            // ArtifactActivatedAbilitiesLocked — consulted in
            // `activate_ability` (Collector Ouphe); no layer effect.
            | StaticEffect::ArtifactActivatedAbilitiesLocked
            // Teferi statics — handled at cast time via dedicated checks
            // (`player_locked_to_sorcery_timing` etc.); not modeled as
            // continuous-layer modifications here.
            | StaticEffect::OpponentsSorceryTimingOnly
            | StaticEffect::ControllerSorceriesAsFlash
            | StaticEffect::ControllerSpellsHaveFlash { .. }
            // DoubleTokens — read at `Effect::CreateToken` resolution time
            // via `GameState::token_doublers_for(seat)`; no layer effect.
            | StaticEffect::DoubleTokens
            // DoubleCounters / ExtraPlusOneCounters — read at counter-add
            // resolution via `GameState::scaled_counter_count`; no layer effect.
            | StaticEffect::DoubleCounters
            | StaticEffect::DoublePlusOneCounters
            | StaticEffect::ExtraPlusOneCounters
            | StaticEffect::ExtraCounterAllKinds
            // Energy-gain bonus — read at AddEnergy time via
            // `GameState::energy_gain_bonus_for`; no layer effect.
            | StaticEffect::EnergyGainBonus { .. }
            // Damage doubling/halving — read at damage time via
            // `GameState::damage_doublers` / `damage_halvers` /
            // `scale_damage_to`; no layer effect.
            | StaticEffect::DoubleDamageDealt
            | StaticEffect::HalveDamageDealt
            | StaticEffect::PreventAllCombatDamageToThis
            | StaticEffect::DoubleDamageToOpponents
            | StaticEffect::DoubleNoncombatDamageToOpponents
            | StaticEffect::NoncombatDamageToOpponentsBonus { .. }
            | StaticEffect::HalveDamageToYou
            | StaticEffect::AddDamageToOpponents { .. }
            | StaticEffect::AddDamageToOpponentsPerCounter { .. }
            | StaticEffect::AddDamageFromColorToPlayers { .. }
            | StaticEffect::OpponentMillDoubled
            // GrantAffinityToISSpells — read at cast time by
            // `cost_reduction_for_spell` directly; no layer effect.
            | StaticEffect::GrantAffinityToISSpells { .. }
            // ExtraEtbCountersForCreatureCasts — read at creature-spell
            // resolution time in `stack.rs::resolve_spell`; no layer effect.
            | StaticEffect::ExtraEtbCountersForCreatureCasts { .. }
            // EtbTriggerSpotlight / DoubleControllerEtbTriggers — read at ETB
            // trigger dispatch via `etb_trigger_multiplier`; no layer effect.
            | StaticEffect::EtbTriggerSpotlight
            | StaticEffect::DoubleControllerEtbTriggers
            // Katara / Harmonic Prodigy — read at trigger dispatch via
            // `ally_trigger_extra_fires`; no layer effect.
            | StaticEffect::DoubleControllerAllyTriggers
            | StaticEffect::DoubleControllerTriggersOfType { .. }
            | StaticEffect::DoubleControllerDeathTriggers
            // SuppressCreatureEtbTriggers — read at trigger dispatch via
            // `creature_etb_triggers_suppressed` / `creature_dies_triggers_suppressed`;
            // no layer effect (Torpor Orb, Tocatli Honor Guard, Hushbringer).
            | StaticEffect::SuppressCreatureEtbTriggers { .. }
            // OtherPlaneswalkersHaveSourceLoyaltyAbilities — read at loyalty
            // activation time in `activate_loyalty_ability`; no layer effect.
            | StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities
            | StaticEffect::PlaneswalkersHaveLoyaltyAbilities { .. }
            // PlayFromLibraryTop / TopOfLibraryRevealed — read by the play/
            // cast paths and the view projection; no layer effect.
            | StaticEffect::PlayFromLibraryTop { .. }
            | StaticEffect::PlayFromLibraryTopOncePerTurn { .. }
            | StaticEffect::TopOfLibraryRevealed
            | StaticEffect::AllLibraryTopsRevealed
            // NamedSpellCantBeCast — consulted in cast_spell_with_convoke
            // (Meddling Mage); no layer effect.
            | StaticEffect::NamedSpellCantBeCast
            // OpponentsCantCastNamed (Ashiok's Erasure) — cast-legality gate,
            // no layer effect.
            | StaticEffect::OpponentsCantCastNamed
            // SpellsYouCastHaveDelve (Teval) — read at cast time by
            // `controller_grants_spells_delve`; no layer effect.
            | StaticEffect::SpellsYouCastHaveDelve
            // EtbTriggerTax — read at ETB trigger push time by
            // `apply_etb_trigger_tax` (Strict Proctor); no layer effect.
            | StaticEffect::EtbTriggerTax { .. }
            // PlayerCannotGainLife — projected onto Player.cannot_gain_life
            // each recompute by apply_player_statics; no layer effect.
            | StaticEffect::PlayerCannotGainLife { .. }
            // PlayerCannotLoseLife — consulted dynamically by adjust_life /
            // damage paths via player_cannot_lose_life_now; no layer effect.
            | StaticEffect::PlayerCannotLoseLife { .. }
            // LifeGainBecomesLoss — consulted dynamically by adjust_life via
            // life_gain_becomes_loss_now (Tainted Remedy); no layer effect.
            | StaticEffect::LifeGainBecomesLoss { .. }
            // AttackTaxToController — consulted in declare_attackers; no layer.
            | StaticEffect::AttackTaxToController { .. }
            // BlockTaxToController — consulted in declare_blockers; no layer.
            | StaticEffect::BlockTaxToController { .. }
            // CapDrawsPerTurn — consulted at draw time via draw_cap_for; no
            // layer effect.
            | StaticEffect::CapDrawsPerTurn { .. }
            // CoinFlipAdvantage (Krark's Thumb) — consulted dynamically by
            // the FlipCoin resolver via coin_flip_advantage_now; no layer effect.
            | StaticEffect::CoinFlipAdvantage { .. }
            // PreventUntap — consulted by `do_untap` (CR 502.3); no layer
            // effect since it gates a turn-based action rather than a
            // characteristic.
            | StaticEffect::PreventUntap { .. }
            // SpellCostFloor (Trinisphere) — read at cast time by
            // `apply_spell_cost_floor`; no layer effect.
            | StaticEffect::SpellCostFloor { .. }
            // CastHandSpellsFree (Omniscience) — read by the free-cast
            // action via `player_casts_hand_spells_free`; no layer effect.
            | StaticEffect::CastHandSpellsFree
            // AnyoneCastsCheapCreaturesFree (Aluren) — read by the free-cast
            // action via `player_casts_cheap_creature_free`; no layer effect.
            | StaticEffect::AnyoneCastsCheapCreaturesFree { .. }
            // GrantKeywordToAttackers — needs live combat state, resolved in
            // `compute_battlefield` against `GameState.attacking`.
            | StaticEffect::GrantKeywordToAttackers { .. }
            // CrewSaddlePowerBonus — read directly by `crew` / `saddle` when
            // summing crew/saddle power; not a real P/T modification.
            | StaticEffect::CrewSaddlePowerBonus { .. }
            // SelfCrewsSaddlesWithToughness — read directly by `crew` / `saddle`.
            | StaticEffect::SelfCrewsSaddlesWithToughness
            // GrantTriggeredAbility — surfaced by `statics_granted_triggers_for`
            // in both trigger dispatchers; no layer effect.
            | StaticEffect::GrantTriggeredAbility { .. }
            // NamedLandsNeutralized — live-resolved in
            // `gather_continuous_effects_inner` (needs the source's named_card).
            | StaticEffect::NamedLandsNeutralized
            | StaticEffect::BlightedLandsNeutralized
            // TokenCreationAddsToken — consulted in the resolve_effect
            // epilogue (Quina's extra-Frog rider); not a layer effect.
            | StaticEffect::TokenCreationAddsToken { .. }
            | StaticEffect::TokenCreationAddsTokenPerToken { .. }
            // Consulted at the mint funnel (Academy Manufactor).
            | StaticEffect::ClueFoodTreasureMintsOneOfEach
            // GrantActivatedAbility — surfaced as a virtual activated ability
            // in `activate_ability`; not a characteristic layer effect.
            | StaticEffect::GrantActivatedAbility { .. }
            // Necrotic Ooze — surfaced via `granted_abilities_for`, not a layer.
            | StaticEffect::HasActivatedAbilitiesOfGraveyardCreatures
            | StaticEffect::HasActivatedAbilitiesOfGraveyardLands
            | StaticEffect::CostReductionPerCounterOnSource { .. }
            | StaticEffect::PreventDamageToThisRedirect
            | StaticEffect::HasActivatedAbilitiesOfLibraryTop { .. }
            | StaticEffect::CounteredCreaturesHaveAbilitiesOfExiledWithSource
            | StaticEffect::MayCastPermanentsFromGraveyard
            | StaticEffect::GraveyardCastWithLifeSurcharge { .. }
            | StaticEffect::ActivationCostReduction { .. }
            // Consulted directly in `activate_ability`, not a layer effect.
            | StaticEffect::OtherExhaustActivationCostReduction { .. }
            // Consulted directly in `equip()`, not a layer effect.
            | StaticEffect::ControllerEquipAtInstantSpeed
            | StaticEffect::EquipCostReduction { .. }
            // Recomputed live in `compute_battlefield`, not here.
            | StaticEffect::SelfHasKeywordWhile { .. }
            | StaticEffect::SelfHasKeywordWhilePredicate { .. }
            | StaticEffect::GraveyardLibraryLockdown
            | StaticEffect::GraveyardLockdown
            | StaticEffect::GraveyardExileLockdown
            | StaticEffect::GraveyardCardsHaveEscape { .. }
            | StaticEffect::GraveyardPermanentsHaveRetraceDuringYourTurn
            | StaticEffect::CollectsLeaverCounters
            | StaticEffect::OpponentsCantActivateArtifactAbilities
            // AnnihilatorPerPlusOneCounter — needs a live counter count,
            // injected in `gather_continuous_effects_inner`.
            | StaticEffect::AnnihilatorPerPlusOneCounter
            // SkipStep — consulted by `advance_step` (CR 614.10); no layer.
            | StaticEffect::SkipStep { .. }
            // AttackPowerCapByControllerHand — consulted in declare_attackers.
            | StaticEffect::AttackPowerCapByControllerHand
            // NotCreatureWhileDevotionBelow — needs live devotion count,
            // resolved in `gather_continuous_effects` against the GameState.
            | StaticEffect::NotCreatureWhileDevotionBelow { .. }
            // NonAuraEnchantmentsAreCreatures — Starfield's gate reads the live
            // enchantment count; resolved in `gather_continuous_effects`.
            | StaticEffect::NonAuraEnchantmentsAreCreatures { .. }
            // DevotionBonus — read directly by `devotion_to`, no continuous effect.
            | StaticEffect::DevotionBonus
            // PreventCombatDamageToSelfAndGrow — consulted at the combat damage
            // sites, not a continuous effect.
            | StaticEffect::PreventCombatDamageToSelfAndGrow
            // PumpSelfByControlledPermanents — needs a live battlefield
            // count; resolved in `gather_continuous_effects`.
            | StaticEffect::PumpSelfByControlledPermanents { .. }
            // PumpTeamByControlledPermanents — team anthem scaled by a live
            // controlled/graveyard count; resolved in `gather_continuous_effects`.
            | StaticEffect::PumpTeamByControlledPermanents { .. }
            // PumpPTPerOtherOfType — needs the live type count; resolved in
            // `gather_continuous_effects`.
            | StaticEffect::PumpPTPerOtherOfType { .. }
            // PumpPerSharedType (Coat of Arms) — per-creature shared-type count;
            // resolved in `gather_continuous_effects`.
            | StaticEffect::PumpPerSharedType { .. }
            // SelfIsCreatureWhileCountersAtLeast — live counter check; resolved
            // in `gather_continuous_effects`.
            | StaticEffect::SelfIsCreatureWhileCountersAtLeast { .. }
            | StaticEffect::SelfHasKeywordWhileCountersAtLeast { .. }
            // PumpSelfIf — needs live predicate evaluation; resolved in
            // `gather_continuous_effects`.
            | StaticEffect::PumpSelfIf { .. }
            // SetBasePtIf — live conditional base-P/T set, resolved in
            // `gather_continuous_effects`.
            | StaticEffect::SetBasePtIf { .. }
            // GrantPumpSelfIf — per-subject predicate, resolved in
            // `gather_continuous_effects`.
            | StaticEffect::GrantPumpSelfIf { .. }
            // PumpTeamIf — conditional team anthem, resolved in
            // `gather_continuous_effects` (needs live predicate eval).
            | StaticEffect::PumpTeamIf { .. }
            // AnthemForChosenType — reads the source's live chosen creature
            // type; resolved in `gather_continuous_effects`.
            | StaticEffect::AnthemForChosenType { .. }
            // AnthemForFilter / SelfHasKeywordIf — need live game state
            // (opponents / predicate eval); resolved in `gather_continuous_effects`.
            | StaticEffect::AnthemForFilter { .. }
            | StaticEffect::SelfHasKeywordIf { .. }
            | StaticEffect::SelfIsCreatureIf { .. }
            // GrantKeywordToChosenType — reads the source's live chosen type;
            // resolved in `gather_continuous_effects`.
            | StaticEffect::GrantKeywordToChosenType { .. }
            // ChosenTypeSpellCostReduction — read at cast time in
            // `cost_reduction_for_spell_zoned`; no continuous-layer effect.
            | StaticEffect::ChosenTypeSpellCostReduction { .. }
            // ChosenTypeEntersWithCounter — read at ETB-counter time via
            // `chosen_type_etb_counter_specs`; no continuous-layer effect.
            | StaticEffect::ChosenTypeEntersWithCounter { .. }
            // ExileNontokenCreaturesNotCast (Containment Priest) — read at
            // battlefield-entry time by `nontoken_creature_etb_exile_active`;
            // no layer effect.
            | StaticEffect::ExileNontokenCreaturesNotCast
            // NoMaximumHandSize / OpponentsMaxHandSizeReduced — consulted
            // at cleanup via `effective_max_hand_size`; no layer effect.
            | StaticEffect::NoMaximumHandSize
            // TappedCreaturesCanBlock — consulted at block declaration via
            // `tapped_creatures_can_block`; no layer effect.
            | StaticEffect::TappedCreaturesCanBlock
            | StaticEffect::OpponentsMaxHandSizeReduced(_)
            | StaticEffect::ControllerMaxHandSize(_)
            | StaticEffect::NamedSpellTax { .. }
            // MayPlayLandsFromGraveyard — consulted by the land-play paths
            // via `player_may_play_lands_from_graveyard`; no layer effect.
            | StaticEffect::MayPlayLandsFromGraveyard
            // MayReturnFromGraveyardInsteadOfLearn — consulted at the top of
            // `Effect::Learn` (Retriever Phoenix); no layer effect.
            | StaticEffect::MayReturnFromGraveyardInsteadOfLearn
            // LifeGainBonus — consulted in `adjust_life` via
            // `life_gain_bonus_now` (Honor Troll); no layer effect.
            | StaticEffect::LifeGainBonus { .. }
            // LifeGainMultiplier — consulted in `adjust_life` via
            // `life_gain_multiplier_now` (Rhox Faithmender); no layer effect.
            | StaticEffect::LifeGainMultiplier { .. }
            // DamageCantBePrevented — consulted in `apply_prevention_shields`
            // via `damage_cant_be_prevented_now` (Sulfuric Vortex); no layer.
            | StaticEffect::DamageCantBePrevented
            // Questing Beast — consulted directly in `apply_prevention_shields`;
            // no layer effect.
            | StaticEffect::ControllerCreaturesCombatDamageCantBePrevented
            // Frenzied Baloth — consulted in `apply_prevention_shields`; no layer.
            | StaticEffect::CombatDamageCantBePrevented
            // Bloodletter — consulted in `adjust_life` via `life_loss_doubled_now`;
            // no layer effect.
            | StaticEffect::OpponentLifeLossDoubledDuringYourTurn
            // ManaProductionDoubled / Tripled — consulted at mana-ability
            // resolution via `mana_production_multiplier_for`; no layer effect.
            | StaticEffect::ManaProductionDoubled
            | StaticEffect::ManaProductionTripled
            // PreventDamageByRemovingCounters (Polukranos, Unchained) —
            // consulted at both damage funnels; no layer effect.
            | StaticEffect::PreventDamageByRemovingCounters { .. }
            // CreatureActivatedAbilitiesLocked — consulted in
            // `activate_ability` (Cursed Totem); no layer effect.
            | StaticEffect::CreatureActivatedAbilitiesLocked
            // CountersCantBePlaced (Solemnity) — consulted at every
            // counter-placement site via `counters_locked`; no layer effect.
            | StaticEffect::CountersCantBePlaced
            // ExileCardsBoundForGraveyard (Rest in Peace / Leyline of the
            // Void) — consulted at graveyard-placement time via
            // `graveyard_exiled_for`; no layer effect.
            | StaticEffect::ExileCardsBoundForGraveyard { .. }
            // DiesToLibraryTopInstead (Pulmonic Sliver) — consulted in
            // `remove_from_battlefield_to_graveyard_raw`; no layer effect.
            | StaticEffect::DiesToLibraryTopInstead { .. }
            // OpponentsCantCastChosenColor (Iona) — gated at the cast
            // dispatch; no layer effect.
            | StaticEffect::OpponentsCantCastChosenColor
            // Melira's poison / -1/-1 locks — consulted at their funnels.
            | StaticEffect::PlayerCannotGetPoison
            | StaticEffect::NoMinusCountersOnYourCreatures
            // Search statics (Aven Mindcensor / Leonin Arbiter) — consulted
            // in `Effect::Search` via `search_top_limit_for` /
            // `pay_search_tax`; no layer effect.
            | StaticEffect::OpponentsSearchTopN { .. }
            | StaticEffect::SearchTax { .. }
            // ActivationTax (Suppression Field) — consulted in
            // `activate_ability`; no layer effect.
            | StaticEffect::ActivationTax { .. }
            // OpponentLoyaltyActivationTax (Eidolon of Obstruction) —
            // consulted in `activate_loyalty_ability`; no layer effect.
            | StaticEffect::OpponentLoyaltyActivationTax { .. }
            // UntapAllYoursEachUntapStep (Seedborn Muse) — consulted by
            // `do_untap`; no layer effect.
            | StaticEffect::UntapAllYoursEachUntapStep
            // ControllerCreatureAbilitiesAsThoughHaste (Tyvar) — consulted at
            // the CR 602.5g activation gate; no layer effect.
            | StaticEffect::ControllerCreatureAbilitiesAsThoughHaste
            // UntapSelfEachUntapStep (Thousand Moons Infantry) — consulted by
            // `do_untap`; no layer effect.
            | StaticEffect::UntapSelfEachUntapStep
            // MaxOneNonbasicLandUntap (Winter Moon) — consulted by `do_untap`;
            // no layer effect.
            | StaticEffect::MaxOneNonbasicLandUntap
            // CounterAmplifierOncePerTurn (Cursed Wombat) — consulted in the
            // `Effect::AddCounter` +1/+1 path; no layer effect.
            | StaticEffect::CounterAmplifierOncePerTurn
            // ExileDyingOpponentCreatures (Valentin) — consulted in
            // `remove_from_battlefield_to_graveyard_raw`; no layer effect.
            | StaticEffect::ExileDyingOpponentCreatures { .. }
            // YourInstantSorcerySpellsHaveLifelink (Radiant Scrollwielder) —
            // consulted in the non-combat damage path; no layer effect.
            | StaticEffect::YourInstantSorcerySpellsHaveLifelink
            // SelfCostReducedByGreatestPower (The Great Henge) — read by
            // `cost_reduction_for_spell` off the spell being cast; no layer.
            | StaticEffect::SelfCostReducedByGreatestPower
            // SelfCostReducedByTotalPower (Ghalta) — same, off the spell.
            | StaticEffect::SelfCostReducedByTotalPower
            // SelfCostReducedPerCreatureInGraveyard (Ghoultree) — same.
            | StaticEffect::SelfCostReducedPerCreatureInGraveyard
            // SelfCostReducedPerCardTypeInGraveyard (Emrakul) — same.
            | StaticEffect::SelfCostReducedPerCardTypeInGraveyard
            // SelfCostReducedByNoncreatureArtifactMv (Metalwork Colossus) — same.
            | StaticEffect::SelfCostReducedByNoncreatureArtifactMv
            // SelfCostReducedPerGraveyardCardMatching (Serpent of the Pass) —
            // read in `cost_reduction_for_spell`; no layer effect.
            | StaticEffect::SelfCostReducedPerGraveyardCardMatching { .. }
            // SelfCostReducedPerPermanentMatching (Allies at Last) — same.
            | StaticEffect::SelfCostReducedPerPermanentMatching { .. }
            // SelfFlashIf (Serpent of the Pass) — consulted at the cast-timing
            // gate; no continuous-layer effect.
            | StaticEffect::SelfFlashIf { .. }
            // SelfCostReducedIfCreatureDiedThisTurn (Bone Picker) — same.
            | StaticEffect::SelfCostReducedIfCreatureDiedThisTurn { .. }
            // SelfCostReducedByDomain (Leyline Binding) — same, off the spell.
            | StaticEffect::SelfCostReducedByDomain { .. }
            // SelfCostReducedByDistinctLandNames (Fungal Colossus) — same.
            | StaticEffect::SelfCostReducedByDistinctLandNames
            // SelfCostReducedDuringYourTurn (Mental Modulation) — same.
            | StaticEffect::SelfCostReducedDuringYourTurn { .. }
            // SelfCostReducedByDevotion (Daybreak Chimera) — same, off the spell.
            | StaticEffect::SelfCostReducedByDevotion { .. }
            // SacrificeCostReduction (Awaken the Blood Avatar) — an optional
            // additional cost consulted by `cast_spell_sacrifice_reduce`; no
            // continuous-layer effect.
            | StaticEffect::SacrificeCostReduction { .. }
            // BargainCostReduction — read by `cast_spell_bargain` at cast time.
            | StaticEffect::BargainCostReduction { .. }
            // OpponentsCantMakeYouSacrifice (Sigarda/Tamiyo) — consulted in
            // the `Effect::Sacrifice` resolver; no continuous-layer effect.
            | StaticEffect::OpponentsCantMakeYouSacrifice
            | StaticEffect::OpponentsCantMakeYouDiscard
            | StaticEffect::ControllerDrawsDoubled
            // ProliferateTwice / PoisonCappedAtOnePerTurn — consulted at the
            // proliferate resolver / `add_poison` funnel.
            | StaticEffect::ProliferateTwice
            | StaticEffect::PoisonCappedAtOnePerTurn
            | StaticEffect::RedirectDamageToSelf
            | StaticEffect::ControllerCantCastPermanentSpells
            | StaticEffect::NoncreatureSpellsCantBeCastIf { .. }
            | StaticEffect::NoncreatureSpellsWithChosenManaValueCantBeCast
            | StaticEffect::SelfCostReducedPerDiscardThisTurn { .. }
            | StaticEffect::SelfCostReducedPerSpellCastThisTurn { .. }
            | StaticEffect::SelfCostReducedPerCreatureAttackedThisTurn { .. }
            | StaticEffect::SelfCostReducedPerOpponent { .. }
            // SelfCostReducedIfControlEach (Of One Mind) — read off the spell.
            | StaticEffect::SelfCostReducedIfControlEach { .. }
            // SelfCostReducedIf (Gigastorm Titan) — read off the spell.
            | StaticEffect::SelfCostReducedIf { .. }
            | StaticEffect::WinInsteadOfDrawFromEmpty
            // CR 104.3d — consulted at the loss/win sites, no layer effect.
            | StaticEffect::ControllerCantLoseGame
            | StaticEffect::ControllerCantWinGame
            // Phyrexian Unlife — consulted at the loss SBA + damage funnels.
            | StaticEffect::ControllerDoesntLoseFromLife
            // Consulted at the damage-to-player life sites.
            | StaticEffect::DamageWontReduceControllerLifeBelowOne { .. }
            | StaticEffect::OneSpellPerTurn
            | StaticEffect::OneNoncreatureSpellPerTurn
            | StaticEffect::OneNonartifactSpellPerTurn
            | StaticEffect::SpellsCostMoreExceptOnControllerTurn { .. }
            | StaticEffect::PreventDamageToYourAttackers
            | StaticEffect::PreventAllDamageToController
            | StaticEffect::PreventNoncombatDamageToYourCreatures
            | StaticEffect::UnspentManaBecomesColorless
            // Consulted directly at the step/phase pool-empty sites.
            | StaticEffect::ManaPoolsNeverEmpty
            | StaticEffect::UnspentColorManaPersists(_)
            // GraveyardAnthem is zone-special: gathered from graveyards in
            // `gather_continuous_effects_inner`, never from the battlefield.
            | StaticEffect::GraveyardAnthem { .. }
            | StaticEffect::SpellsUncounterable { .. }
            | StaticEffect::MinusCounterReduction
            // Hand-zone grant, consulted by `landcycle_card` / the view.
            | StaticEffect::GrantTypecyclingToHandCards { .. }
            // CR 605.1b — resolved at the mana-ability fast path.
            | StaticEffect::ExtraManaOnLandTap { .. }
            // ETB-counter replacement, read at `chosen_type_etb_counter_specs`.
            | StaticEffect::TypeEntersWithCounter { .. }
            | StaticEffect::TypeEntersWithCountersPerControlled { .. }
            | StaticEffect::OtherCreaturesEnterWithCountersEqualToSourcePower { .. }
            // Target-tax, read at `extra_cost_for_spell` (Jubilant Skybonder).
            | StaticEffect::TaxOpponentSpellsTargeting { .. }
            | StaticEffect::OpponentsCantCastDuringYourTurn
            | StaticEffect::OpponentsCantActDuringYourTurn
            // Void Winnower — cast gate + block-legality gate; no layer effect.
            | StaticEffect::OpponentsCantCastEvenMv
            | StaticEffect::OpponentsCantBlockWithEvenMv
            // Attack-permission static, read in `ignores_defender_for_attack`.
            | StaticEffect::CanAttackIgnoringDefenderWhile { .. }
            // Drannith Magistrate — cast-legality gate in `cast_from_zone_blocked`.
            | StaticEffect::OpponentsCantCastFromAnywhereButHand
            // Lier — read by the flashback-cast path / graveyard view.
            | StaticEffect::GraveyardInstantsSorceriesHaveFlashback
            // Carth the Lion — read where loyalty activation costs apply.
            | StaticEffect::LoyaltyAbilitiesCostExtra(_)
            // Zabaz — read where the modular death trigger moves counters.
            | StaticEffect::ModularBonusCounters(_) => vec![],

            // Serra's Emissary — the creature half is a layer-6 keyword grant
            // keyed to the ETB-chosen card type; the player half is read at
            // the targeting/damage gates.
            StaticEffect::YouAndCreaturesProtectionFromChosenCardType => {
                match &card.chosen_card_type {
                    Some(t) => vec![ContinuousEffect {
                        timestamp,
                        source,
                        affected: AffectedPermanents::CardMatch {
                            source_controller: card.controller,
                            requirement: Box::new(SelectionRequirement::And(
                                Box::new(SelectionRequirement::Creature),
                                Box::new(SelectionRequirement::ControlledByYou),
                            )),
                        },
                        layer: Layer::L6Ability,
                        sublayer: None,
                        duration: EffectDuration::WhileSourceOnBattlefield,
                        modification: Modification::AddKeyword(
                            crate::card::Keyword::ProtectionFromCardType(t.clone()),
                        ),
                    }],
                    None => vec![],
                }
            }
        }
    }
}

/// Translate a selector into a `layers::AffectedPermanents` description for
/// those `StaticEffect` variants that express broad "lord-like" scope. Returns
/// `None` if the selector shape isn't representable in the layer system yet.
/// True if the filter tree contains `IsModified` (CR 700.9) — such filters
/// are resolved live in `gather_continuous_effects`, not through the static
/// `AffectedPermanents` decomposition.
fn requirement_mentions_modified(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::IsModified => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_modified(a) || requirement_mentions_modified(b)
        }
        R::Not(inner) => requirement_mentions_modified(inner),
        _ => false,
    }
}

fn requirement_mentions_equipped(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::IsEquipped | R::EquippedByAtLeast(_) => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_equipped(a) || requirement_mentions_equipped(b)
        }
        R::Not(inner) => requirement_mentions_equipped(inner),
        _ => false,
    }
}

/// Whether a `GrantKeyword` / `PumpPT` static's filter reads live battlefield
/// state (combat role, modified-ness, attachment counts) and must therefore be
/// resolved into a `Specific` id list on every layer recompute instead of
/// routing through the state-blind `CardMatch` path.
fn requirement_needs_live_resolution(req: &SelectionRequirement) -> bool {
    requirement_mentions_modified(req)
        || requirement_mentions_attacking(req)
        || requirement_mentions_equipped(req)
}

/// Whether `req` references the live combat state (`IsAttacking`), so a
/// `GrantKeyword` static over it must recompute its affected set per layer
/// pass rather than route through the printed-characteristics walker.
fn requirement_mentions_attacking(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::IsAttacking => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_attacking(a) || requirement_mentions_attacking(b)
        }
        R::Not(inner) => requirement_mentions_attacking(inner),
        _ => false,
    }
}

pub(crate) fn selector_to_affected(
    sel: &crate::effect::Selector,
    card: &CardInstance,
) -> Option<AffectedPermanents> {
    use crate::effect::Selector;
    let controller = card.controller;
    match sel {
        Selector::This => Some(AffectedPermanents::Source),
        Selector::AttachedTo(inner) => {
            if matches!(inner.as_ref(), Selector::This)
                && let Some(attached_id) = card.attached_to
            {
                Some(AffectedPermanents::Specific(vec![attached_id]))
            } else {
                None
            }
        }
        Selector::EachPermanent(req) => affected_from_requirement(req, controller),
        _ => None,
    }
}

/// Whether the conjunctive And-tree walker in `affected_from_requirement`
/// recognizes every leaf of `req`. Mirrors that walker's match arms; any
/// other leaf (or a disjunction) falls outside it.
fn simple_walker_can_handle(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::And(a, b) => simple_walker_can_handle(a) && simple_walker_can_handle(b),
        R::ControlledByYou | R::ControlledByOpponent | R::Creature | R::Artifact
        | R::Enchantment | R::Planeswalker | R::Land | R::HasCardType(_)
        | R::HasCreatureType(_) | R::WithCounter(_) | R::HasColor(_) | R::Colorless
        | R::IsToken | R::NotToken | R::OtherThanSource | R::Any | R::Permanent => true,
        _ => false,
    }
}

/// Split `PowerAtLeast(n)` leaves off a conjunctive requirement tree.
/// Returns `(max gate, residual tree)` — `None` when no power leaf exists or
/// the tree isn't a plain And-conjunction. An all-gate tree leaves
/// `SelectionRequirement::Any` as the residual.
fn extract_power_gate(
    req: &SelectionRequirement,
) -> Option<(i32, SelectionRequirement)> {
    use SelectionRequirement as R;
    fn walk(r: &R, gate: &mut Option<i32>) -> Option<Option<R>> {
        // Outer None = unsupported shape; inner None = leaf removed.
        match r {
            R::PowerAtLeast(n) => {
                *gate = Some(gate.map_or(*n, |g: i32| g.max(*n)));
                Some(None)
            }
            R::And(a, b) => {
                let ra = walk(a, gate)?;
                let rb = walk(b, gate)?;
                Some(match (ra, rb) {
                    (Some(a), Some(b)) => Some(R::And(Box::new(a), Box::new(b))),
                    (Some(x), None) | (None, Some(x)) => Some(x),
                    (None, None) => None,
                })
            }
            // Power leaves under Or/Not can't be hoisted into a single gate.
            R::Or(..) | R::Not(..) => {
                if requirement_mentions_power(r) { None } else { Some(Some(r.clone())) }
            }
            other => Some(Some(other.clone())),
        }
    }
    let mut gate = None;
    let residual = walk(req, &mut gate)?;
    gate.map(|g| (g, residual.unwrap_or(R::Any)))
}

fn requirement_mentions_power(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::PowerAtLeast(_) => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_power(a) || requirement_mentions_power(b)
        }
        R::Not(inner) => requirement_mentions_power(inner),
        _ => false,
    }
}

/// True when `req` contains an `AttachedToSource` leaf — the cost path uses
/// this to intersect candidates against the actual source id (Faunsbane Troll).
pub(crate) fn requirement_mentions_attached_to_source(req: &SelectionRequirement) -> bool {
    use SelectionRequirement as R;
    match req {
        R::AttachedToSource => true,
        R::And(a, b) | R::Or(a, b) => {
            requirement_mentions_attached_to_source(a)
                || requirement_mentions_attached_to_source(b)
        }
        R::Not(inner) => requirement_mentions_attached_to_source(inner),
        _ => false,
    }
}

fn affected_from_requirement(
    req: &SelectionRequirement,
    source_controller: usize,
) -> Option<AffectedPermanents> {
    use SelectionRequirement as R;
    // Disjunctive / nonbasic-land filters can't be flattened into the simple
    // controller+type decomposition below (`card_types` is conjunctive, and
    // there's no plain CardType for "nonbasic land"). When every leaf is
    // computable from a card's printed characteristics, route the whole filter
    // through the card-local matcher instead of dropping the static (CR 614.13
    // Thalia, Heretic Cathar). Only used when the simple walker can't.
    if !simple_walker_can_handle(req) && crate::game::layers::requirement_is_card_only(req) {
        return Some(AffectedPermanents::CardMatch {
            source_controller,
            requirement: Box::new(req.clone()),
        });
    }
    // Power-gated lord scope (CR 613.8 — Temur Ascendancy's "creatures you
    // control with power 4 or greater have haste"): split the PowerAtLeast
    // leaves off the And-tree; the residual must be card-only.
    if let Some((gate, residual)) = extract_power_gate(req)
        && crate::game::layers::requirement_is_card_only(&residual)
    {
        return Some(AffectedPermanents::CardMatchPowerGated {
            source_controller,
            requirement: Box::new(residual),
            power_at_least: gate,
        });
    }
    // Decompose And-trees to extract controller filter + card-type filter.
    let mut ctrl: Option<Option<usize>> = None; // Outer Some(None) = all players; Some(Some(n)) = specific player
    let mut types: Vec<CardType> = vec![];
    let mut creature_type: Option<crate::card::CreatureType> = None;
    let mut counter_filter: Option<crate::card::CounterType> = None;
    let mut color_filter: Option<crate::mana::Color> = None;
    let mut colorless_filter = false;
    let mut token_filter: Option<bool> = None;
    // CR-driven "other" exclusion (push XXXV). `SelectionRequirement::
    // OtherThanSource` flips this to true; the resulting AffectedPermanents
    // variant carries `exclude_source: true` so the layer-time `affects()`
    // check skips the source permanent itself — matching printed "**other**
    // [type] you control" wording.
    let mut other_than_source = false;
    let mut opponent = false;
    let mut walk = vec![req];
    while let Some(r) = walk.pop() {
        match r {
            R::And(a, b) => {
                walk.push(a);
                walk.push(b);
            }
            R::ControlledByYou => ctrl = Some(Some(source_controller)),
            // Accumulate a flag rather than returning early, so the opponent
            // filter composes with type filters regardless of And-tree order
            // (`ControlledByOpponent.and(Creature)` and the reverse both work).
            R::ControlledByOpponent => opponent = true,
            R::Creature => types.push(CardType::Creature),
            R::Artifact => types.push(CardType::Artifact),
            R::Enchantment => types.push(CardType::Enchantment),
            R::Planeswalker => types.push(CardType::Planeswalker),
            R::Land => types.push(CardType::Land),
            R::HasCardType(t) => types.push(t.clone()),
            R::HasCreatureType(ct) => creature_type = Some(*ct),
            R::WithCounter(ct) => counter_filter = Some(*ct),
            R::HasColor(c) => color_filter = Some(*c),
            R::Colorless => colorless_filter = true,
            R::IsToken => token_filter = Some(true),
            R::NotToken => token_filter = Some(false),
            R::OtherThanSource => other_than_source = true,
            R::Any | R::Permanent => {}
            _ => return None,
        }
    }
    if opponent {
        // `friendly_seats` is populated by `compute_battlefield` /
        // `apply_enters_tapped_replacement` once the source's team is known
        // (this helper has no GameState handle). Counter/creature-type filters
        // on the opponent path aren't decomposed yet (tracked in TODO.md).
        return Some(AffectedPermanents::AllOpponents {
            source_controller,
            card_types: types,
            friendly_seats: Vec::new(),
        });
    }
    if let Some(counter) = counter_filter {
        return Some(AffectedPermanents::AllWithCounter {
            controller: ctrl.flatten(),
            card_types: types,
            counter,
            at_least: 1,
        });
    }
    if let Some(ct) = creature_type {
        return Some(AffectedPermanents::AllWithCreatureType {
            controller: ctrl.flatten(),
            creature_type: ct,
            exclude_source: other_than_source,
        });
    }
    Some(AffectedPermanents::All {
        controller: ctrl.unwrap_or(None),
        card_types: types,
        exclude_source: other_than_source,
        color: color_filter,
        token: token_filter,
        colorless: colorless_filter,
    })
}


// ── Helpers ───────────────────────────────────────────────────────────────────

/// Returns true if `blocker` is legally allowed to block `attacker`.
/// Uses `blocker_kws` / `attacker_kws` as the effective keyword sets
/// (from `ComputedPermanent`) instead of the raw definition keywords.
pub(crate) fn can_block_attacker_computed(
    blocker: &CardInstance,
    blocker_computed: &ComputedPermanent,
    attacker_kws: &[Keyword],
    attacker_colors: &[crate::mana::Color],
    attacker_power: i32,
) -> bool {
    let blocker_kws = &blocker_computed.keywords;
    // Unblockable: can't be blocked at all.
    if attacker_kws.contains(&Keyword::Unblockable) {
        return false;
    }
    // Flying: can only be blocked by fliers or reach.
    if attacker_kws.contains(&Keyword::Flying)
        && !blocker_kws.contains(&Keyword::Flying)
        && !blocker_kws.contains(&Keyword::Reach)
    {
        return false;
    }
    // Wanderlight Spirit: this blocker can block only creatures with flying.
    if blocker_kws.contains(&Keyword::CanBlockOnlyFlying)
        && !attacker_kws.contains(&Keyword::Flying)
    {
        return false;
    }
    // Horsemanship: can only be blocked by other Horsemanship creatures.
    if attacker_kws.contains(&Keyword::Horsemanship)
        && !blocker_kws.contains(&Keyword::Horsemanship)
    {
        return false;
    }
    // Shadow: can only block/be blocked by other shadow creatures.
    if attacker_kws.contains(&Keyword::Shadow) && !blocker_kws.contains(&Keyword::Shadow) {
        return false;
    }
    if blocker_kws.contains(&Keyword::Shadow) && !attacker_kws.contains(&Keyword::Shadow) {
        return false;
    }
    // Skulk (CR 702.72a): can't be blocked by creatures with greater power.
    // Both sides use layer-computed power (an anthem-pumped Skulk attacker
    // dodges bigger blockers correctly).
    if attacker_kws.contains(&Keyword::Skulk) && blocker_computed.power > attacker_power {
        return false;
    }
    // Formation Breaker (CR 509.1b): creatures with power less than this
    // creature's power can't block it — the inverse of Skulk.
    if attacker_kws.contains(&Keyword::CantBeBlockedByPowerLess)
        && blocker_computed.power < attacker_power
    {
        return false;
    }
    // Questing Beast (CR 509.1b): can't be blocked by creatures with power N
    // or less — a fixed threshold, not relative to the attacker's power.
    if attacker_kws.iter().any(|k| {
        matches!(k, Keyword::CantBeBlockedByPowerAtMost(n) if blocker_computed.power <= *n as i32)
    }) {
        return false;
    }
    // Squeak By (CR 509.1b): can't be blocked by creatures with power N or
    // greater — the fixed-threshold mirror of `CantBeBlockedByPowerAtMost`.
    if attacker_kws.iter().any(|k| {
        matches!(k, Keyword::CantBeBlockedByPowerAtLeast(n) if blocker_computed.power >= *n as i32)
    }) {
        return false;
    }
    // Ironclaw Orcs (CR 509.1b): this blocker can't block creatures with power
    // N or greater — a restriction on the blocker keyed off the attacker.
    if blocker_kws.iter().any(|k| {
        matches!(k, Keyword::CantBlockPowerAtLeast(n) if attacker_power >= *n as i32)
    }) {
        return false;
    }
    // Fear (CR 702.36): can only be blocked by artifact creatures and/or
    // black creatures.
    if attacker_kws.contains(&Keyword::Fear) {
        let blocker_is_artifact = blocker.definition.is_artifact();
        let blocker_is_black = blocker_computed.colors.contains(&crate::mana::Color::Black);
        if !blocker_is_artifact && !blocker_is_black {
            return false;
        }
    }
    // Intimidate (CR 702.13): can only be blocked by artifact creatures
    // or creatures that share a color with the attacker. We compare the
    // attacker's *computed* colors (which include hybrid / mono-hybrid
    // pips and color-setting effects, via `ComputedPermanent.colors`)
    // against the blocker's computed colors — not raw `{C}` cost pips.
    if attacker_kws.contains(&Keyword::Intimidate) {
        let blocker_is_artifact = blocker.definition.is_artifact();
        let shares_color = blocker_computed
            .colors
            .iter()
            .any(|c| attacker_colors.contains(c));
        if !blocker_is_artifact && !shares_color {
            return false;
        }
    }
    // Protection from a color (CR 702.16e): the attacker can't be blocked
    // by a creature of a color it has protection from. Read the blocker's
    // computed colors so hybrid-pip and effect-granted colors count.
    for kw in attacker_kws {
        if let Keyword::Protection(color) = kw
            && blocker_computed.colors.contains(color)
        {
            return false;
        }
        // CR 702.16b — protection from creatures: can't be blocked at all.
        if matches!(kw, Keyword::ProtectionFromCreatures) {
            return false;
        }
        // CR 702.16e — protection from a creature type: can't be blocked by a
        // creature of that type.
        if let Keyword::ProtectionFromCreatureType(ty) = kw
            && blocker_computed.subtypes.creature_types.contains(ty)
        {
            return false;
        }
        // CR 702.16 — protection from each mana value other than N (Haktos):
        // can't be blocked by a creature whose mana value isn't N.
        if let Keyword::ProtectionFromManaValueExcept(n) = kw
            && blocker.definition.cost.cmc() != *n
        {
            return false;
        }
        // CR 702.16 — protection from each mana value of a parity: can't be
        // blocked by a creature whose mana value matches the chosen quality.
        if let Keyword::ProtectionFromManaValueParity { odd } = kw
            && (blocker.definition.cost.cmc() % 2 == 1) == *odd
        {
            return false;
        }
        // CR 702.16 — protection from multicolored: can't be blocked by a
        // creature that is two or more colors.
        if matches!(kw, Keyword::ProtectionFromMulticolored)
            && blocker_computed.colors.len() >= 2
        {
            return false;
        }
        // CR 702.16j — protection from a card type: can't be blocked by a
        // creature of that type (matters for artifact/enchantment creatures).
        if let Keyword::ProtectionFromCardType(t) = kw
            && blocker_computed.card_types.contains(t)
        {
            return false;
        }
        // CR 702.16 — protection from everything: can't be blocked at all.
        if matches!(kw, Keyword::ProtectionFromEverything) {
            return false;
        }
    }
    // CR 509.1b "can't be blocked except by [filter]" / "can't be blocked by
    // [filter]" — evaluate the blocker's computed characteristics against the
    // attacker's filter keywords.
    for kw in attacker_kws {
        match kw {
            Keyword::CantBeBlockedExceptBy(filter)
                if !blocker_matches_block_filter(blocker, blocker_computed, filter) => {
                    return false;
                }
            Keyword::CantBeBlockedBy(filter)
                if blocker_matches_block_filter(blocker, blocker_computed, filter) =>
            {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Lightweight evaluation of a block-restriction filter against a blocker's
/// *computed* characteristics. Covers the subset of `SelectionRequirement`
/// that "can't be blocked except by [filter]" cards actually use (type,
/// color, keyword, power/toughness thresholds). Unsupported variants resolve
/// to `false` (conservatively excluding the blocker).
fn blocker_matches_block_filter(
    blocker: &CardInstance,
    computed: &ComputedPermanent,
    req: &SelectionRequirement,
) -> bool {
    use SelectionRequirement as R;
    match req {
        R::Any | R::Permanent | R::Creature => true,
        R::Artifact => blocker.definition.is_artifact(),
        R::Enchantment => blocker.definition.is_enchantment(),
        R::Land => blocker.definition.is_land(),
        R::IsToken => blocker.is_token,
        R::NotToken => !blocker.is_token,
        R::HasColor(c) => computed.colors.contains(c),
        R::Colorless => computed.colors.is_empty(),
        R::HasKeyword(k) => computed.keywords.contains(k),
        R::HasToxic => computed.keywords.iter().any(|k| matches!(k, Keyword::Toxic(_))),
        R::HasMutate => blocker.definition.mutate.is_some(),
        R::HasCreatureType(t) => blocker.definition.subtypes.creature_types.contains(t)
            || computed.keywords.contains(&Keyword::Changeling),
        R::HasArtifactSubtype(a) => blocker.definition.subtypes.artifact_subtypes.contains(a),
        R::PowerAtMost(n) => computed.power <= *n,
        R::PowerAtLeast(n) => computed.power >= *n,
        R::ToughnessAtMost(n) => computed.toughness <= *n,
        R::ToughnessAtLeast(n) => computed.toughness >= *n,
        R::ToughnessGreaterThanPower => computed.toughness > computed.power,
        R::HasCardType(ct) => blocker.definition.card_types.contains(ct),
        R::And(a, b) => {
            blocker_matches_block_filter(blocker, computed, a)
                && blocker_matches_block_filter(blocker, computed, b)
        }
        R::Or(a, b) => {
            blocker_matches_block_filter(blocker, computed, a)
                || blocker_matches_block_filter(blocker, computed, b)
        }
        R::Not(inner) => !blocker_matches_block_filter(blocker, computed, inner),
        _ => false,
    }
}
