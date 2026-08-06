//! CR conformance for this run:
//! - CR 502.3 — an untap-step cap ("players can't untap more than two
//!   permanents") applies to every seat and lifts when its gate closes.
//! - CR 508.1d — an "if one attacks, all attack if able" requirement is
//!   checked against the declared batch, so it costs nothing when none of the
//!   group attacks.
//! - CR 713.4 — a double-faced card is its front face for all game purposes
//!   while the back stays inert.
//! - CR 203.1 / 213.1 / 200.2 — printed-only parts of a card (illustration,
//!   collector data) are not characteristics and have no gameplay effect.
//! - CR 900.2 — the casual-variant supplemental zones start empty.

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

/// CR 502.3 — the cap is global: the non-active player's own untap step is
/// capped too, and tapping the Orb lifts it.
#[test]
fn cr_502_3_untap_cap_applies_to_every_seat() {
    let mut g = two_player_game();
    let orb = g.add_card_to_battlefield(0, catalog::static_orb());
    let lands: Vec<CardId> =
        (0..3).map(|_| g.add_card_to_battlefield(1, catalog::forest())).collect();
    let tap_all = |g: &mut GameState, ids: &[CardId]| {
        for id in ids {
            g.battlefield_find_mut(*id).unwrap().tapped = true;
        }
    };
    let untapped = |g: &GameState, ids: &[CardId]| {
        ids.iter().filter(|id| !g.battlefield_find(**id).unwrap().tapped).count()
    };

    tap_all(&mut g, &lands);
    g.active_player_idx = 1;
    g.do_untap();
    assert_eq!(untapped(&g, &lands), 2, "the opponent's untap step is capped too");

    tap_all(&mut g, &lands);
    g.battlefield_find_mut(orb).unwrap().tapped = true;
    g.do_untap();
    assert_eq!(untapped(&g, &lands), 3, "a tapped Orb imposes nothing");
}

/// CR 508.1d — the group requirement only bites once one of the group is in
/// the declared batch.
#[test]
fn cr_508_1d_group_attack_requirement_is_batch_relative() {
    let mut g = two_player_game();
    let web = ready(&mut g, 0, catalog::magnetic_web());
    let magnet = ready(&mut g, 0, catalog::grizzly_bears());
    let free = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: web,
        ability_index: 0,
        target: Some(Target::Permanent(magnet)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("magnetize");
    drain_stack(&mut g);

    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 0;
    // Attacking with only the unmagnetized creature is legal — no member of
    // the group attacked, so the requirement never applies.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: free,
        target: AttackTarget::Player(1),
    }]))
    .expect("the group stays home");
}

/// CR 713.4 / 203.1 — a DFC is the card its front face names for every game
/// purpose; the back face contributes no characteristics until it transforms.
#[test]
fn cr_713_4_dfc_is_its_front_face_until_it_transforms() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::birgi_god_of_storytelling());
    let live = g.battlefield_find(id).expect("on the battlefield");
    assert_eq!(live.definition.name, "Birgi, God of Storytelling");
    assert!(live.definition.back_face.is_some(), "the back rides along on the card");
    let computed = g.computed_permanent(id).expect("computed");
    assert_eq!((computed.power, computed.toughness), (3, 3), "front-face P/T");
    assert!(
        !computed.keywords.contains(&Keyword::Haste),
        "the back face's text is inert while the front is up"
    );
}

/// CR 213.1 — information printed below the text box (collector data, artist)
/// is not part of a card's characteristics, so two copies of one card are
/// indistinguishable to every rules query.
#[test]
fn cr_213_1_printed_only_information_does_not_affect_play() {
    let a = catalog::grizzly_bears();
    let b = catalog::grizzly_bears();
    assert_eq!(a.name, b.name);
    assert_eq!(a.cost, b.cost);
    assert_eq!((a.power, a.toughness), (b.power, b.toughness));
    assert_eq!(a.card_types, b.card_types);
    assert_eq!(a.keywords, b.keywords);

    let mut g = two_player_game();
    let one = g.add_card_to_battlefield(0, a);
    let two = g.add_card_to_battlefield(0, b);
    let pt = |g: &GameState, id| {
        let c = g.computed_permanent(id).unwrap();
        (c.power, c.toughness)
    };
    assert_eq!(pt(&g, one), pt(&g, two));
}

/// CR 203.1 — the illustration has no effect on play: a creature depicted
/// flying flies only if its rules text says so.
#[test]
fn cr_203_1_illustration_grants_nothing() {
    let mut g = two_player_game();
    // Cloud Elemental is a flier by rules text; Grizzly Bears is not, however
    // either is depicted.
    let flier = g.add_card_to_battlefield(0, catalog::cloud_elemental());
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::Flying));
    assert!(!g.computed_permanent(ground).unwrap().keywords.contains(&Keyword::Flying));
}

/// CR 200.2 / 109.3 — only some parts of a card are characteristics of the
/// object. A permanent's computed characteristics are exactly the rules-visible
/// ones, and they survive a round-trip through the object's live definition.
#[test]
fn cr_200_2_only_characteristics_reach_the_object() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::serra_angel());
    let c = g.computed_permanent(id).expect("computed");
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords.contains(&Keyword::Flying) && c.keywords.contains(&Keyword::Vigilance));
    let def = &g.battlefield_find(id).unwrap().definition;
    assert_eq!(def.name, "Serra Angel");
    assert_eq!(def.card_types, vec![crabomination::card::CardType::Creature]);
}

/// CR 900.2 — the casual-variant supplemental zones exist alongside the
/// ordinary ones and start empty in a traditional game.
#[test]
fn cr_900_2_supplemental_zones_are_empty_in_a_normal_game() {
    let g = two_player_game();
    assert!(
        g.players.iter().all(|p| p.command.is_empty()),
        "no commanders, planes, schemes or emblems"
    );
    assert!(g.exile.is_empty());
}
