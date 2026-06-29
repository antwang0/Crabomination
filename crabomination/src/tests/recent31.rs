//! Functionality tests for `catalog::sets::decks::recent31` — wedge/guild
//! modal charms & commands, the graveyard-CDA *goyf* family, and assorted
//! multicolor staples. Modal modes are exercised by resolving the chosen
//! `ChooseMode`/`ChooseN` sub-effect directly with a target-bearing context.

use crate::card::{CardType, CreatureType, Effect, Keyword};
use crate::catalog;
use crate::game::effects::EffectContext;
use crate::game::two_player_game;
use crate::game::*;

fn ctx0(_g: &GameState) -> EffectContext {
    EffectContext::for_ability(crate::card::CardId(0), 0, None)
}

fn modes(def: crate::card::CardDefinition) -> Vec<Effect> {
    match def.effect {
        Effect::ChooseMode(m) => m,
        Effect::ChooseN { modes, .. } => modes,
        other => panic!("not a modal card: {other:?}"),
    }
}

// ── Charms ───────────────────────────────────────────────────────────────────

#[test]
fn gruul_charm_burns_flyers() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let m = modes(catalog::gruul_charm());
    g.resolve_effect(&m[2], &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    // 3 damage marked on a 4-toughness flyer survives; assert the damage landed.
    assert_eq!(g.battlefield_find(flyer).unwrap().damage, 3, "3 damage to each flyer");
}

#[test]
fn dimir_charm_destroys_small_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&modes(catalog::dimir_charm())[1], &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "power-2 creature destroyed");
}

#[test]
fn orzhov_charm_destroy_loses_toughness_life() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(angel)];
    g.resolve_effect(&modes(catalog::orzhov_charm())[1], &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, 16, "lost life equal to its toughness (4)");
}

#[test]
fn naya_charm_burns_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&modes(catalog::naya_charm())[0], &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "3 damage kills a 2/2");
}

#[test]
fn naya_charm_taps_only_target_players_creatures() {
    // CR 107.3 / 508 — "target player controls": mode 3 taps the chosen
    // player's creatures and leaves everyone else's untapped.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&modes(catalog::naya_charm())[2], &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped, "target player's creature tapped");
    assert!(!g.battlefield_find(mine).unwrap().tapped, "our own creature untouched");
}

#[test]
fn jund_charm_adds_two_counters() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&modes(catalog::jund_charm())[2], &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two +1/+1 counters");
}

#[test]
fn grixis_charm_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&modes(catalog::grixis_charm())[1], &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "-4/-4 kills a 2/2");
}

// ── Commands (choose two) ────────────────────────────────────────────────────

#[test]
fn silumgars_command_minus_three() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&modes(catalog::silumgars_command())[2], &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "-3/-3 kills a 2/2");
}

#[test]
fn ojutais_command_gains_life() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.resolve_effect(&modes(catalog::ojutais_command())[1], &ctx0(&g)).unwrap();
    assert_eq!(g.players[0].life, 24);
}

#[test]
fn atarkas_command_burns_opponent() {
    let mut g = two_player_game();
    let before = g.players[1].life;
    g.resolve_effect(&modes(catalog::atarkas_command())[1], &ctx0(&g)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 3, "3 damage to each opponent");
}

// ── *goyf graveyard-CDA family ───────────────────────────────────────────────

#[test]
fn lhurgoyf_counts_all_graveyard_creatures() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::lhurgoyf());
    let cp = g.computed_permanent(id).unwrap();
    // 2 creature cards across all graveyards → power 2, toughness 2+1.
    assert_eq!((cp.power, cp.toughness), (2, 3));
}

#[test]
fn boneyard_wurm_counts_only_your_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears()); // opponent's — ignored
    let id = g.add_card_to_battlefield(0, catalog::boneyard_wurm());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "only your graveyard counts");
}

#[test]
fn splinterfright_is_a_trampling_goyf() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::splinterfright());
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

// ── Other staples ────────────────────────────────────────────────────────────

#[test]
fn disciple_of_bolas_sacrifices_for_value() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 sac fodder
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::disciple_of_bolas());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22, "gained life = sacrificed power (2)");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew = sacrificed power (2)");
}

#[test]
fn agony_warp_splits_minus_across_two_targets() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
    g.resolve_effect(&catalog::agony_warp().effect, &ctx).unwrap();
    g.check_state_based_actions();
    // a got -3/-0 (power 2→-1, survives as 0); b got -0/-3 (toughness 2→-1, dies).
    assert!(g.battlefield_find(b).is_none(), "-0/-3 kills the second target");
    assert!(g.battlefield_find(a).is_some(), "-3/-0 doesn't kill the first");
}

#[test]
fn savage_knuckleblade_pumps_itself() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::savage_knuckleblade());
    let mut ctx = ctx0(&g);
    ctx.source = Some(id);
    g.resolve_effect(&catalog::savage_knuckleblade().activated_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "the firebreathing ability pumps +2/+2");
}

#[test]
fn butcher_grants_chosen_keyword_for_a_sacrifice() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::butcher_of_the_horde());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    let mut ctx = ctx0(&g);
    ctx.source = Some(id);
    // Default decider picks mode 0 (vigilance).
    g.resolve_effect(&catalog::butcher_of_the_horde().activated_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Vigilance));
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 1,
        "sacrificed another creature");
}

#[test]
fn demonic_dread_has_cascade_and_grants_fear() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::demonic_dread().effect, &ctx).unwrap();
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Fear));
    // Cascade rides a printed cast trigger.
    assert!(catalog::demonic_dread().triggered_abilities.iter()
        .any(|t| matches!(t.effect, Effect::Cascade { .. })));
}

#[test]
fn glory_activates_from_graveyard() {
    let def = catalog::glory();
    assert!(def.keywords.contains(&Keyword::Flying));
    assert!(def.activated_abilities[0].from_graveyard, "protection grant is graveyard-only");
}

#[test]
fn foul_tongue_invocation_sacs_and_gains_with_dragon() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    let dragon = crate::card::CardDefinition {
        name: "Test Dragon",
        cost: crate::mana::cost(&[crate::mana::r()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Dragon], ..Default::default()
        },
        power: 4, toughness: 4,
        ..Default::default()
    };
    g.add_card_to_battlefield(0, dragon);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut ctx = ctx0(&g);
    ctx.targets = vec![Target::Player(1)];
    g.resolve_effect(&catalog::foul_tongue_invocation().effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target player sacrificed a creature");
    assert_eq!(g.players[0].life, 24, "gained 4 — you control a Dragon");
}

#[test]
fn first_sliver_grants_sliver_spells_cascade() {
    let def = catalog::the_first_sliver();
    // Its own printed cascade plus the battlefield "Sliver spells have cascade".
    let cascades = def.triggered_abilities.iter()
        .filter(|t| matches!(t.effect, Effect::Cascade { .. })).count();
    assert_eq!(cascades, 2);
    assert!(def.subtypes.creature_types.contains(&CreatureType::Sliver));
}

#[test]
fn mortivore_regenerates() {
    let def = catalog::mortivore();
    assert!(matches!(def.activated_abilities[0].effect, Effect::Regenerate { .. }));
}
