//! Functionality tests for Dragon's Maze (DGM) cards in `catalog::sets::dgm`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn two_gates(g: &mut GameState, p: usize) {
    g.add_card_to_battlefield(p, catalog::azorius_guildgate());
    g.add_card_to_battlefield(p, catalog::boros_guildgate());
}

/// Keyword vanillas carry their printed abilities.
#[test]
fn dgm_keyword_creatures() {
    let sky = catalog::skylasher();
    assert!(sky.keywords.contains(&Keyword::Flash));
    assert!(sky.keywords.contains(&Keyword::CantBeCountered));
    assert!(sky.keywords.contains(&Keyword::Reach));
    assert!(sky.keywords.contains(&Keyword::Protection(Color::Blue)));
    let law = catalog::ascended_lawmage();
    assert!(law.keywords.contains(&Keyword::Flying) && law.keywords.contains(&Keyword::Hexproof));
    let piker = catalog::riot_piker();
    assert!(piker.keywords.contains(&Keyword::FirstStrike) && piker.keywords.contains(&Keyword::MustAttack));
}

/// Sunspire Gatekeepers makes a Knight only with two or more Gates.
#[test]
fn sunspire_gatekeepers_gated_token() {
    let mut g = two_player_game();
    // Without gates: no token.
    let gk = g.move_card_to_battlefield_for_test(0, catalog::sunspire_gatekeepers());
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Knight"), "no token without gates");
    g.battlefield.retain(|c| c.id != gk);
    // With two gates: a Knight token appears.
    two_gates(&mut g, 0);
    g.move_card_to_battlefield_for_test(0, catalog::sunspire_gatekeepers());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Knight" && c.controller == 0), "Knight with two gates");
}

/// Saruli Gatekeepers gains 7 life with two or more Gates.
#[test]
fn saruli_gatekeepers_lifegain() {
    let mut g = two_player_game();
    two_gates(&mut g, 0);
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::saruli_gatekeepers());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 7);
}

/// Maw of the Obzedat pumps the team by sacrificing a creature.
#[test]
fn maw_of_the_obzedat_pumps_team() {
    let mut g = two_player_game();
    let maw = g.add_card_to_battlefield(0, catalog::maw_of_the_obzedat());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: maw, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Maw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none() || g.battlefield_find(bear).is_none(), "a creature was sacrificed");
    // The surviving pumped bear is 3/3.
    let survivor = if g.battlefield_find(bear).is_some() { bear } else { maw };
    assert_eq!(g.computed_permanent(survivor).map(|c| c.power), Some(if survivor == maw { 4 } else { 3 }));
}

/// Phytoburst gives +5/+5.
#[test]
fn phytoburst_pumps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::phytoburst());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Phytoburst");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).map(|c| (c.power, c.toughness)), Some((7, 7)));
}

/// Riot Control gains 1 life per opposing creature.
#[test]
fn riot_control_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::riot_control());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Riot Control");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "1 life per opposing creature");
}

/// Punish the Enemy deals 3 to a player and 3 to a creature.
#[test]
fn punish_the_enemy_split_burn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::punish_the_enemy());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![Target::Permanent(bear)],
        mode: None, x_value: None,
    }).expect("cast Punish the Enemy");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.battlefield_find(bear).is_none(), "the 2/2 died to 3 damage");
}

/// Morgue Burst reanimates to hand and burns for the returned card's power.
#[test]
fn morgue_burst_reanimate_and_burn() {
    let mut g = two_player_game();
    // A 3-power creature card in the graveyard.
    let dead = g.add_card_to_hand(0, catalog::hill_giant());
    g.players[0].hand.retain(|c| c.id != dead);
    let inst = crabomination::card::CardInstance::new(dead, catalog::hill_giant(), 0);
    g.players[0].graveyard.push(inst);
    let spell = g.add_card_to_hand(0, catalog::morgue_burst());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(dead)), additional_targets: vec![Target::Player(1)],
        mode: None, x_value: None,
    }).expect("cast Morgue Burst");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned to hand");
    assert_eq!(g.players[1].life, life - 3, "burn equal to returned power (3)");
}

/// Zhur-Taa Druid pings each opponent when tapped for mana.
#[test]
fn zhur_taa_druid_pings_on_tap() {
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::zhur_taa_druid());
    g.battlefield_find_mut(druid).unwrap().summoning_sick = false;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: druid, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "opponent pinged for 1");
    assert!(g.players[0].mana_pool.total() >= 1, "produced mana");
}

/// Trostani's Summoner makes three tokens on ETB.
#[test]
fn trostanis_summoner_tokens() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::trostanis_summoner());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Knight"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Centaur"));
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Rhino"));
}

/// Bronzebeak Moa pumps itself +3/+3 when another creature enters.
#[test]
fn bronzebeak_moa_pumps_on_etb() {
    let mut g = two_player_game();
    let moa = g.add_card_to_battlefield(0, catalog::bronzebeak_moa());
    // Cast a creature so the enters trigger dispatches to Moa.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(moa).map(|c| c.power), Some(5), "2/2 -> 5/5 until end of turn");
}

/// Fatal Fumes gives -4/-2, killing a 2/2.
#[test]
fn fatal_fumes_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fatal_fumes());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fatal Fumes");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 dies to -4/-2");
}

/// Runner's Bane taps the enchanted creature and keeps it from untapping.
#[test]
fn runners_bane_locks_down() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::runners_bane());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Runner's Bane");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "ETB taps the creature");
}

/// Sinister Possession drains the enchanted creature's controller when it attacks.
#[test]
fn sinister_possession_drains_on_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::sinister_possession());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    g.perform_action(GameAction::PassPriority).ok();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("declare attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "controller loses 2 on attack");
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut guard = 0;
    while g.step != step && guard < 40 {
        g.perform_action(GameAction::PassPriority).expect("pass");
        guard += 1;
    }
}

/// Blood Scrivener: drawing with an empty hand draws two and loses 1 life;
/// with a nonempty hand it draws normally.
#[test]
fn blood_scrivener_empty_hand_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blood_scrivener());
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.players[0].hand.clear();
    let life = g.players[0].life;
    // Empty hand: draw two, lose 1.
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    assert_eq!(g.players[0].hand.len(), 2, "empty-hand draw becomes two");
    assert_eq!(g.players[0].life, life - 1, "lose 1 life");
    // Nonempty hand: a normal single draw, no life loss.
    let life2 = g.players[0].life;
    g.draw_one(0, &mut ev);
    assert_eq!(g.players[0].hand.len(), 3, "normal single draw with cards in hand");
    assert_eq!(g.players[0].life, life2, "no life loss on a normal draw");
}

/// Pontiff of Blight grants extort to your other creatures: casting a spell
/// with one other creature out fires two extort triggers (drain 2 total).
#[test]
fn pontiff_of_blight_grants_extort() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pontiff_of_blight());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Mana for the spell ({1}{G}) plus two {W/B} extort payments.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    // Pay both extort triggers.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "two extort drains");
    assert_eq!(g.players[0].life, life0 + 2, "gained the drained life");
}
