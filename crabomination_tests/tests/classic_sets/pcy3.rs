//! Prophecy (PCY), third wave — the Rhystic cycle.

use crabomination::card::{CardDefinition, CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// The printed-keyword bodies.
#[test]
fn pcy3_keyword_bodies_carry_their_printed_keywords() {
    let cases: &[(fn() -> CardDefinition, &[Keyword])] = &[
        (catalog::rib_cage_spider, &[Keyword::Reach]),
        (catalog::spitting_spider, &[Keyword::Reach]),
        (catalog::spiketail_drake, &[Keyword::Flying]),
        (catalog::ribbon_snake, &[Keyword::Flying]),
        (catalog::mercenary_informer, &[Keyword::HexproofFromColor(Color::Black)]),
        (catalog::rebel_informer, &[Keyword::HexproofFromColor(Color::White)]),
    ];
    for (factory, expected) in cases {
        let def = factory();
        for kw in *expected {
            assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// Rethink counters at the spell's own mana value.
#[test]
fn rethink_counters_an_unpaid_spell() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.players[1].mana_pool.empty();
    let rethink = g.add_card_to_hand(0, catalog::rethink());
    cast(&mut g, 0, rethink, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// Spiketail Drake trades itself for a Mana Leak.
#[test]
fn spiketail_drake_counters_by_sacrificing_itself() {
    let mut g = main_phase();
    let drake = g.add_card_to_battlefield(0, catalog::spiketail_drake());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    g.players[1].mana_pool.empty();
    activate(&mut g, 0, drake, 0, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, 20, "the Bolt was countered");
    assert!(g.battlefield_find(drake).is_none(), "the Drake paid for it");
}

/// Rhystic Cave fixes any colour when nobody pays the toll.
#[test]
fn rhystic_cave_makes_a_colour_when_unpaid() {
    let mut g = main_phase();
    let cave = g.add_card_to_battlefield(0, catalog::rhystic_cave());
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cave,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Rhystic Syphon drains for five when the toll goes unpaid.
#[test]
fn rhystic_syphon_drains_when_unpaid() {
    let mut g = main_phase();
    let syphon = g.add_card_to_hand(0, catalog::rhystic_syphon());
    cast(&mut g, 0, syphon, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 15);
    assert_eq!(g.players[0].life, 25);
}

/// Rhystic Lightning's second half lands when nobody pays.
#[test]
fn rhystic_lightning_deals_four_when_unpaid() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(0, catalog::rhystic_lightning());
    cast(&mut g, 0, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 16);
}

/// Rhystic Shield's bonus half lands when nobody pays.
#[test]
fn rhystic_shield_gives_the_full_bonus_when_unpaid() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let shield = g.add_card_to_hand(0, catalog::rhystic_shield());
    cast(&mut g, 0, shield, None);
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 5), "+0/+1 then +0/+2");
}

/// Rhystic Tutor finds anything when nobody pays.
#[test]
fn rhystic_tutor_finds_a_card_when_unpaid() {
    let mut g = main_phase();
    let prize = g.add_card_to_library(0, catalog::shivan_dragon());
    let tutor = g.add_card_to_hand(0, catalog::rhystic_tutor());
    // The tax's yes/no comes first (declined), then the tutor's pick.
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(false),
        crabomination::decision::DecisionAnswer::Search(Some(prize)),
    ]));
    cast(&mut g, 0, tutor, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == prize));
}

/// Rhystic Deluge taps a creature its controller won't pay for.
#[test]
fn rhystic_deluge_taps_when_unpaid() {
    let mut g = main_phase();
    let deluge = g.add_card_to_battlefield(0, catalog::rhystic_deluge());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, deluge, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).unwrap().tapped);
}

/// Scoria Cat is a 6/6 while you're tapped out.
#[test]
fn scoria_cat_grows_when_you_are_tapped_out() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::scoria_cat());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    assert_eq!(g.computed_permanent(cat).unwrap().power, 3);
    g.battlefield_find_mut(land).unwrap().tapped = true;
    assert_eq!(g.computed_permanent(cat).unwrap().power, 6);
}

/// Root Cage keeps Mercenaries tapped.
#[test]
fn root_cage_locks_mercenaries_down() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::root_cage());
    let merc = g.add_card_to_battlefield(1, catalog::bog_glider());
    g.battlefield_find_mut(merc).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.step = TurnStep::Untap;
    g.do_untap();
    assert!(g.battlefield_find(merc).unwrap().tapped);
}

/// Silt Crawler costs you the rest of your turn's mana.
#[test]
fn silt_crawler_taps_your_lands_on_entry() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let crawler = g.add_card_to_hand(0, catalog::silt_crawler());
    cast(&mut g, 0, crawler, None);
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// Ribbon Snake can be grounded by the player it's flying over.
#[test]
fn ribbon_snake_can_be_grounded_by_any_player() {
    let mut g = main_phase();
    let snake = g.add_card_to_battlefield(0, catalog::ribbon_snake());
    assert!(g.computed_permanent(snake).unwrap().keywords.contains(&Keyword::Flying));
    activate(&mut g, 1, snake, 0, None);
    assert!(!g.computed_permanent(snake).unwrap().keywords.contains(&Keyword::Flying));
}

/// Shrouded Serpent goes unblockable when the defender won't pay.
#[test]
fn shrouded_serpent_is_unblockable_when_unpaid() {
    let mut g = two_player_game();
    let serpent = g.add_card_to_battlefield(0, catalog::shrouded_serpent());
    g.clear_sickness(serpent);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: serpent, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(serpent).unwrap().keywords.contains(&Keyword::Unblockable),
        "they declined the toll"
    );
}

/// Soul Charmer's bite gains you life unless they pay.
#[test]
fn soul_charmer_gains_life_on_an_unpaid_bite() {
    let mut g = two_player_game();
    let charmer = g.add_card_to_battlefield(0, catalog::soul_charmer());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(charmer);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: charmer, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, charmer)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, 22);
}

/// The Informers bottom the opposing faction.
#[test]
fn mercenary_informer_bottoms_a_mercenary() {
    let mut g = main_phase();
    let informer = g.add_card_to_battlefield(0, catalog::mercenary_informer());
    let merc = g.add_card_to_battlefield(1, catalog::bog_glider());
    activate(&mut g, 0, informer, 0, Some(Target::Permanent(merc)));
    assert!(g.battlefield_find(merc).is_none());
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(merc), "bottom of their library");
}

/// Snag blanks the whole unblocked attack; a Forest can pay for it.
#[test]
fn snag_fogs_the_unblocked_attackers() {
    let mut g = two_player_game();
    let forest = g.add_card_to_hand(1, catalog::forest());
    let snag = g.add_card_to_hand(1, catalog::snag());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: snag,
        target: None,
        additional_targets: vec![],
        pitch_card: None,
        mode: None,
        x_value: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == forest), "the Forest paid");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[1].life, 20, "no combat damage got through");
}

/// Reveille Squad untaps your team when they swing at you.
#[test]
fn reveille_squad_untaps_your_board_on_their_attack() {
    let mut g = two_player_game();
    let squad = g.add_card_to_battlefield(0, catalog::reveille_squad());
    let tapped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(tapped).unwrap().tapped, "the squad woke everyone up");
    assert!(g.battlefield_find(squad).is_some());
}
