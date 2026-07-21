//! CR 702.55 — Haunt. Functionality tests for the Guildpact haunt cards in
//! `catalog::sets::gpt`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

fn resolve_spell(g: &mut GameState, def: crabomination::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// A haunt creature is exiled (not graveyard'd) when it dies, then its haunt
/// body fires when the haunted creature dies.
#[test]
fn shrieking_grotesque_haunts_then_payoff_on_death() {
    let mut g = two_player_game();
    let grotesque = g.add_card_to_battlefield(0, catalog::shrieking_grotesque());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, survives
    g.add_card_to_hand(1, catalog::grizzly_bears()); // the one card to discard

    // Kill the Grotesque → it's exiled haunting the opponent's creature.
    g.battlefield_find_mut(grotesque).unwrap().damage = 1; // lethal vs 2/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == grotesque), "exiled haunting");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == grotesque), "not in graveyard");
    assert_eq!(g.players[1].hand.len(), 1, "payoff not fired yet");

    // The haunted creature dies → opponent discards a card.
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "haunt payoff: opponent discarded");
}

/// Mourning Thrull's gain-2-and-draw trigger fires on entry.
#[test]
fn mourning_thrull_etb_gain_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::mourning_thrull());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew a card");
}

/// Mourning Thrull's haunt body (gain 2, draw 1) fires when the haunted
/// creature dies, even though the Thrull itself is in exile.
#[test]
fn mourning_thrull_haunt_payoff_on_haunted_death() {
    let mut g = two_player_game();
    let thrull = g.add_card_to_battlefield(0, catalog::mourning_thrull());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }

    g.battlefield_find_mut(thrull).unwrap().damage = 1; // lethal vs 1/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == thrull));

    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "haunt gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "haunt drew a card");
}

/// A haunt instant resolves its main effect, is exiled haunting a creature
/// (not graveyard'd), then fires its haunt body when that creature dies.
#[test]
fn douse_in_gloom_instant_haunts() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let douse = g.add_card_to_hand(0, catalog::douse_in_gloom());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);

    let life = g.players[0].life;
    cast_at(&mut g, douse, Target::Permanent(foe));
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "dealt 2");
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    assert!(g.exile.iter().any(|c| c.id == douse), "spell exiled haunting");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == douse), "not graveyard'd");

    // Kill the haunted creature → haunt body: 2 to the opponent, gain 2.
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "haunt dealt 2 to opponent");
    assert_eq!(g.players[0].life, p0_life + 2, "haunt gained 2");
}

/// Castigate exiles a nonland from the opponent's hand on cast and again when
/// the haunted creature dies.
#[test]
fn castigate_haunt_repeats_hand_exile() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let cast_id = g.add_card_to_hand(0, catalog::castigate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    cast_at(&mut g, cast_id, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 1, "cast exiled one nonland");

    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "haunt exiled the second nonland");
}

/// Absolver Thrull's enters-or-haunt trigger destroys an enchantment.
#[test]
fn absolver_thrull_etb_destroys_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::pacifism());
    g.move_card_to_battlefield_for_test(0, catalog::absolver_thrull());
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "ETB destroyed the enchantment");
}

// ── GPT gap cards (non-haunt + haunt reanimator) ─────────────────────────────

/// Giant Solifuge — 4/1 with trample, haste, shroud.
#[test]
fn giant_solifuge_stat_line() {
    let s = catalog::giant_solifuge();
    assert_eq!((s.power, s.toughness), (4, 1));
    for kw in [Keyword::Trample, Keyword::Haste, Keyword::Shroud] {
        assert!(s.keywords.contains(&kw), "has {kw:?}");
    }
}

/// Crystal Seer's activated ability bounces itself back to hand.
#[test]
fn crystal_seer_returns_itself() {
    let mut g = two_player_game();
    let seer = g.add_card_to_battlefield(0, catalog::crystal_seer());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(seer).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == seer), "returned to hand");
}

/// Izzet Chronarch's ETB returns an instant/sorcery from the graveyard.
#[test]
fn izzet_chronarch_recurs_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_helix());
    g.move_card_to_battlefield_for_test(0, catalog::izzet_chronarch());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "instant returned to hand");
}

/// Drowned Rusalka loots (discard then draw) by sacrificing a creature.
#[test]
fn drowned_rusalka_loots_on_sacrifice() {
    let mut g = two_player_game();
    let rusalka = g.add_card_to_battlefield(0, catalog::drowned_rusalka());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // the card to discard
    g.add_card_to_library(0, catalog::forest()); // the card to draw
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rusalka, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed as a cost");
    assert_eq!(g.players[0].hand.len(), hand0, "discarded one, drew one (net zero)");
}

/// Crash Landing strips flying and deals damage equal to Forests controlled.
#[test]
fn crash_landing_grounds_and_burns_by_forests() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    resolve_spell(&mut g, catalog::crash_landing(), vec![Target::Permanent(flyer)]);
    let s = g.battlefield_find(flyer).unwrap();
    assert_eq!(s.damage, 2, "2 damage = number of Forests");
    assert!(!g.compute_battlefield().iter().find(|c| c.id == flyer).unwrap()
        .keywords.contains(&Keyword::Flying), "lost flying this turn");
}

/// Hissing Miasma drains the attacking player when a creature attacks you.
#[test]
fn hissing_miasma_pings_the_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hissing_miasma());
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "attacking player lost 1 life");
}

/// Agent of Masks drains each opponent on your upkeep.
#[test]
fn agent_of_masks_upkeep_drain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::agent_of_masks());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, l0 + 1, "you gained that much");
}

/// Exhumer Thrull's ETB reanimates a creature card to hand.
#[test]
fn exhumer_thrull_recurs_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::exhumer_thrull());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

/// Benediction of Moons gains 1 life per player (2 in a 2-player game).
#[test]
fn benediction_of_moons_gains_per_player() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    resolve_spell(&mut g, catalog::benediction_of_moons(), vec![]);
    assert_eq!(g.players[0].life, life0 + 2, "1 life for each of the 2 players");
}
