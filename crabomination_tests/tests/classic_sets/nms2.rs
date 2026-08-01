//! Nemesis (NMS), second wave.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

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

fn alt_cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
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

/// Seat 0's `attacker` attacks seat 1 and is blocked by seat 1's `blocker`,
/// stopping right after blockers are declared (so the trigger can be seen).
fn attack_and_block(g: &mut GameState, attacker: CardId, blocker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(g);
}

/// A blocked Laccolith trades its combat damage for a ping of its power.
#[test]
fn laccolith_trades_combat_damage_for_a_ping() {
    let mut g = two_player_game();
    let whelp = g.add_card_to_battlefield(0, catalog::laccolith_whelp()); // 1/1
    let blocker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    attack_and_block(&mut g, whelp, blocker);
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 1, "the ping landed");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 1, "and no combat damage on top");
}

/// Declining the trade leaves combat alone.
#[test]
fn laccolith_declined_deals_normal_combat_damage() {
    let mut g = two_player_game();
    let whelp = g.add_card_to_battlefield(0, catalog::laccolith_whelp());
    let blocker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    script(&mut g, vec![DecisionAnswer::Bool(false)]);
    attack_and_block(&mut g, whelp, blocker);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 1, "combat damage, not the ping");
}

/// Laccolith Rig bolts the same trigger onto any creature.
#[test]
fn laccolith_rig_grants_the_trigger() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let rig = g.add_card_to_hand(0, catalog::laccolith_rig());
    cast(&mut g, 0, rig, Some(Target::Permanent(host)));
    let blocker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    script(&mut g, vec![DecisionAnswer::Bool(true)]);
    attack_and_block(&mut g, host, blocker);
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 2, "the host pinged for its power");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 2, "and skipped combat damage");
}

/// Sneaky Homunculus ignores — and is ignored by — anything with 2 power.
#[test]
fn sneaky_homunculus_dodges_big_creatures() {
    let def = catalog::sneaky_homunculus();
    assert!(def.keywords.contains(&Keyword::CantBlockPowerAtLeast(2)));
    assert!(def.keywords.contains(&Keyword::CantBeBlockedByPowerAtLeast(2)));

    let mut g = two_player_game();
    let sneak = g.add_card_to_battlefield(0, catalog::sneaky_homunculus());
    let big = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(sneak);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: sneak, target: AttackTarget::Player(1) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(big, sneak)])).is_err(),
        "a 2-power blocker can't stop it"
    );
}

/// Animate Land turns a land into a 3/3 for the turn.
#[test]
fn animate_land_makes_a_three_three() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::animate_land());
    cast(&mut g, 0, spell, Some(Target::Permanent(forest)));
    let cp = g.computed_permanent(forest).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.card_types.contains(&crabomination::card::CardType::Land), "still a land");
}

/// Arc Mage splits two damage across two targets.
#[test]
fn arc_mage_splits_two_damage() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::arc_mage());
    g.clear_sickness(mage);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let a = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage,
        ability_index: 0,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().damage, 2);
}

/// Defender en-Vec turns fade counters into damage prevention.
#[test]
fn defender_en_vec_spends_fade_counters_on_shields() {
    let mut g = two_player_game();
    let cleric = g.move_card_to_battlefield_for_test(0, catalog::defender_en_vec());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cleric).unwrap().counter_count(CounterType::Fade), 4);
    activate(&mut g, 0, cleric, 0, Some(Target::Player(0)));
    assert_eq!(g.battlefield_find(cleric).unwrap().counter_count(CounterType::Fade), 3);
    let mut evs = vec![];
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut evs);
    assert_eq!(g.players[0].life, 19, "two of the three were soaked");
}

/// Fanatical Devotion trades a creature for a regeneration shield.
#[test]
fn fanatical_devotion_regenerates_off_a_sacrifice() {
    let mut g = two_player_game();
    let devotion = g.add_card_to_battlefield(0, catalog::fanatical_devotion());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let keeper = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw());
    activate(&mut g, 0, devotion, 0, Some(Target::Permanent(keeper)));
    assert!(g.battlefield_find(fodder).is_none(), "the fodder went");
    assert!(g.battlefield_find(keeper).unwrap().regeneration_shields > 0);
}

/// Massacre is free with a Swamp against a Plains and shrinks everything.
#[test]
fn massacre_is_free_and_sweeps() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(1, catalog::plains());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let massacre = g.add_card_to_hand(0, catalog::massacre());
    alt_cast(&mut g, 0, massacre, None);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
}

/// Mind Swords exiles two cards from every hand.
#[test]
fn mind_swords_exiles_two_from_each_hand() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::swamp());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::forest());
        g.add_card_to_hand(1, catalog::forest());
    }
    let swords = g.add_card_to_hand(0, catalog::mind_swords());
    alt_cast(&mut g, 0, swords, None);
    assert!(g.battlefield_find(fodder).is_none(), "paid with a creature");
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Nesting Wurm digs up its own siblings.
#[test]
fn nesting_wurm_finds_its_siblings() {
    let mut g = main_phase();
    let sibling = g.add_card_to_library(0, catalog::nesting_wurm());
    let wurm = g.add_card_to_hand(0, catalog::nesting_wurm());
    script(&mut g, vec![DecisionAnswer::Search(Some(sibling))]);
    cast(&mut g, 0, wurm, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == sibling));
}

/// Rathi Assassin only kills tapped nonblack creatures.
#[test]
fn rathi_assassin_kills_tapped_nonblack_creatures() {
    let mut g = two_player_game();
    let assassin = g.add_card_to_battlefield(0, catalog::rathi_assassin());
    g.clear_sickness(assassin);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: assassin,
            ability_index: 0,
            target: Some(Target::Permanent(victim)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "untapped is off limits"
    );
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    activate(&mut g, 0, assassin, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
}

/// Predator, Flagship grants flight, then shoots fliers down.
#[test]
fn predator_flagship_grants_then_kills_flying() {
    let mut g = two_player_game();
    let ship = g.add_card_to_battlefield(0, catalog::predator_flagship());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, ship, 0, Some(Target::Permanent(victim)));
    let cp = g.computed_permanent(victim).expect("computed");
    assert!(cp.keywords.contains(&Keyword::Flying));
    activate(&mut g, 0, ship, 1, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
}

/// Rupture sprays the sacrificed creature's power over the ground and both
/// players.
#[test]
fn rupture_sprays_the_sacrificed_power() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); // 6 power, no flying
    let flier = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5 flying
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rupture = g.add_card_to_hand(0, catalog::rupture());
    cast(&mut g, 0, rupture, None);
    assert!(g.battlefield_find(ground).is_none(), "the ground creature ate 6");
    assert!(g.battlefield_find(flier).is_some(), "fliers are spared");
    assert_eq!(g.players[0].life, 14);
    assert_eq!(g.players[1].life, 14);
}

/// Saproling Cluster is a discard outlet any player may use.
#[test]
fn saproling_cluster_is_open_to_everyone() {
    let mut g = two_player_game();
    let cluster = g.add_card_to_battlefield(0, catalog::saproling_cluster());
    g.add_card_to_hand(1, catalog::forest());
    activate(&mut g, 1, cluster, 0, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 1);
    assert!(g.players[1].hand.is_empty(), "the opponent paid the discard");
}

/// Spiritual Asylum shrouds your board until you attack.
#[test]
fn spiritual_asylum_shrouds_until_you_swing() {
    let mut g = main_phase();
    let asylum = g.add_card_to_battlefield(0, catalog::spiritual_asylum());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).expect("computed");
    assert!(cp.keywords.contains(&Keyword::Shroud));

    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(asylum).is_none(), "swinging sacrifices it");
}

/// Lin Sivvi fetches a Rebel within X and recycles one from the graveyard.
#[test]
fn lin_sivvi_fetches_and_recycles_rebels() {
    let mut g = two_player_game();
    let sivvi = g.add_card_to_battlefield(0, catalog::lin_sivvi_defiant_hero());
    g.clear_sickness(sivvi);
    let falcon = g.add_card_to_library(0, catalog::defiant_falcon()); // MV 2
    script(&mut g, vec![DecisionAnswer::Search(Some(falcon))]);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sivvi,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("tutor");
    drain_stack(&mut g);
    assert!(g.battlefield_find(falcon).is_some());

    let dead = g.add_card_to_graveyard(0, catalog::defiant_falcon());
    activate(&mut g, 0, sivvi, 1, Some(Target::Permanent(dead)));
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(dead), "bottom of the library");
}

/// Wild Mammoth defects to whoever has the most creatures.
#[test]
fn wild_mammoth_defects_to_the_biggest_board() {
    let mut g = two_player_game();
    let mammoth = g.add_card_to_battlefield(0, catalog::wild_mammoth());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mammoth).unwrap().controller, 1);
}

/// Oraxid can't be touched by red.
#[test]
fn oraxid_has_protection_from_red() {
    let mut g = main_phase();
    let oraxid = g.add_card_to_battlefield(0, catalog::oraxid());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(oraxid)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "protection from red blocks the target"
    );
}

