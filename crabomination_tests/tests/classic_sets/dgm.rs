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

/// A Cluestone taps for one of its two colors and sacrifices for a card.
#[test]
fn cluestone_mana_and_sac_draw() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::izzet_cluestone());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Mana ability adds one blue-or-red.
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.total() >= 1, "produced a mana");
    // Sac ability ({U}{R}, {T}, Sacrifice: draw). Provide the two colored mana.
    let stone2 = g.add_card_to_battlefield(0, catalog::izzet_cluestone());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone2, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for card");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone2).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Maze Behemoth grants trample to your multicolored creatures (not mono ones).
#[test]
fn maze_behemoth_grants_trample_to_multicolored() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::maze_behemoth());
    // A gold creature gets trample; a mono creature doesn't.
    let gold = g.add_card_to_battlefield(0, catalog::spike_jester()); // {B}{R}
    let mono = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // {1}{G}
    assert!(g.computed_permanent(gold).unwrap().keywords.contains(&Keyword::Trample), "gold gains trample");
    assert!(!g.computed_permanent(mono).unwrap().keywords.contains(&Keyword::Trample), "mono unaffected");
}

/// Advent of the Wurm makes a 5/5 trampling Wurm.
#[test]
fn advent_of_the_wurm_token() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::advent_of_the_wurm());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Advent");
    drain_stack(&mut g);
    let wurm = g.battlefield.iter().find(|c| c.definition.name == "Wurm").expect("Wurm made");
    assert_eq!((wurm.definition.power, wurm.definition.toughness), (5, 5));
}

/// Renounce the Guilds makes each player sacrifice a multicolored permanent.
#[test]
fn renounce_the_guilds_sacs_gold() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::spike_jester()); // gold
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // mono, spared
    let theirs = g.add_card_to_battlefield(1, catalog::spike_jester()); // gold
    let spell = g.add_card_to_hand(0, catalog::renounce_the_guilds());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Renounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(), "both gold creatures sacrificed");
}

// ── DGM gap cards (guild legends, mythics, remaining commons) ───────────────

use crabomination::card::CounterType;

/// Sire of Insanity empties every hand at each end step.
#[test]
fn sire_of_insanity_empties_hands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sire_of_insanity());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
    for _ in 0..2 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty() && g.players[1].hand.is_empty(), "both hands discarded");
}

/// Savageborn Hydra enters with X +1/+1 counters and has double strike.
#[test]
fn savageborn_hydra_enters_with_x_counters() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::savageborn_hydra());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.cast_spell(spell, None, vec![], None, Some(3)).expect("cast Hydra X=3");
    drain_stack(&mut g);
    let hydra = g.battlefield.iter().find(|c| c.definition.name == "Savageborn Hydra").expect("hydra");
    assert_eq!(g.computed_permanent(hydra.id).map(|c| c.power), Some(3), "X=3 → 3/3");
    assert!(hydra.definition.keywords.contains(&Keyword::DoubleStrike));
}

/// Exava grants haste to other creatures you control with a +1/+1 counter.
#[test]
fn exava_grants_haste_to_counter_bearers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::exava_rakdos_blood_witch());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "no haste without a counter");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "counter-bearer gains haste");
}

/// Ruric Thar deals 6 to a player who casts a noncreature spell.
#[test]
fn ruric_thar_punishes_noncreature_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ruric_thar_the_unbowed());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::phytoburst());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.cast_spell(spell, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Phytoburst");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 6, "caster of a noncreature spell takes 6");
}

/// Lavinia detains opponents' low-cost nonland permanents on entry.
#[test]
fn lavinia_detains_cheap_permanents() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    g.move_card_to_battlefield_for_test(0, catalog::lavinia_of_the_tenth());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().detained_by.is_some(), "cheap opposing creature detained");
}

/// Blood Baron gets +6/+6 and flying only while you're at 30+ and an opp ≤10.
#[test]
fn blood_baron_conditional_buff() {
    let mut g = two_player_game();
    let baron = g.add_card_to_battlefield(0, catalog::blood_baron_of_vizkopa());
    g.players[0].life = 30;
    g.players[1].life = 10;
    let cp = g.computed_permanent(baron).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10), "buffed to 10/10");
    assert!(cp.keywords.contains(&Keyword::Flying), "gains flying");
    g.players[1].life = 11; // condition fails
    let cp = g.computed_permanent(baron).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "back to 4/4");
}

/// Mirko Vosk mills a player until four lands on combat damage.
#[test]
fn mirko_vosk_mills_until_four_lands() {
    let mut g = two_player_game();
    let mirko = g.add_card_to_battlefield(0, catalog::mirko_vosk_mind_drinker());
    g.clear_sickness(mirko);
    // Stack player 1's library: lands interspersed with nonlands.
    for _ in 0..6 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    for _ in 0..4 { g.add_card_to_library(1, catalog::azorius_guildgate()); }
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: mirko, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    let lands = g.players[1].graveyard.iter().filter(|c| c.definition.is_land()).count();
    assert!(lands >= 4, "milled until four lands ({lands} lands in graveyard)");
}

/// Tajic gets +5/+5 when it and two others attack (Battalion).
#[test]
fn tajic_battalion_pump() {
    let mut g = two_player_game();
    let tajic = g.add_card_to_battlefield(0, catalog::tajic_blade_of_the_legion());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for c in [tajic, a, b] { g.clear_sickness(c); }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: tajic, target: AttackTarget::Player(1) },
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])).expect("attack with three");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(tajic).map(|c| c.power), Some(7), "Battalion +5/+5");
}

/// Vorel doubles every kind of counter on the target.
#[test]
fn vorel_doubles_all_counters() {
    let mut g = two_player_game();
    let vorel = g.add_card_to_battlefield(0, catalog::vorel_of_the_hull_clade());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    g.clear_sickness(vorel);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: vorel, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Vorel");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 6, "3 → 6 counters");
}

/// Zhur-Taa Ancient doubles a land's mana output.
#[test]
fn zhur_taa_ancient_extra_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zhur_taa_ancient());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap Forest");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "Forest yields two green");
}

/// Smelt-Ward Gatekeepers steals a creature with two Gates.
#[test]
fn smelt_ward_gatekeepers_steal() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::azorius_guildgate());
    g.add_card_to_battlefield(0, catalog::boros_guildgate());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::smelt_ward_gatekeepers());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "stole the creature");
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Haste), "granted haste");
}

/// Scion of Vitu-Ghazi makes a Bird then populates it when cast.
#[test]
fn scion_of_vitu_ghazi_bird_and_populate() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::scion_of_vitu_ghazi());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.cast_spell(spell, None, vec![], None, None).expect("cast Scion");
    drain_stack(&mut g);
    let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird" && c.controller == 0).count();
    assert_eq!(birds, 2, "one minted Bird, then populate copies it");
}

/// Rot Farm Skeleton returns itself from the graveyard for a mill cost.
#[test]
fn rot_farm_skeleton_recurs_from_graveyard() {
    let mut g = two_player_game();
    let skel = g.add_card_to_graveyard(0, catalog::rot_farm_skeleton());
    for _ in 0..6 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate recursion");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Rot Farm Skeleton" && c.controller == 0), "returned to battlefield");
}

/// Gleam of Battle puts a +1/+1 counter on each attacker.
#[test]
fn gleam_of_battle_counters_attackers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gleam_of_battle());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "attacker got a counter");
}

/// Debt to the Deathless drains 2X and gains that much.
#[test]
fn debt_to_the_deathless_double_drain() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::debt_to_the_deathless());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let my_life = g.players[0].life;
    let opp_life = g.players[1].life;
    g.cast_spell(spell, None, vec![], None, Some(3)).expect("cast Debt X=3");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_life - 6, "opponent loses 2X = 6");
    assert_eq!(g.players[0].life, my_life + 6, "you gain the life lost");
}

/// Obzedat's Aid reanimates any permanent card from your graveyard.
#[test]
fn obzedats_aid_reanimates_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::obzedats_aid());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.cast_spell(spell, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Aid");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0), "bear reanimated");
}

/// Drown in Filth shrinks a creature by the lands in your graveyard.
#[test]
fn drown_in_filth_scales_with_lands() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_graveyard(0, catalog::forest()); }
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::drown_in_filth());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.cast_spell(spell, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Drown");
    drain_stack(&mut g);
    // Mills four (all lands here) → 6 lands in graveyard → -6/-6 kills the 2/2.
    assert!(g.battlefield_find(bear).is_none(), "creature died to -N/-N");
}

/// Blast of Genius deals damage equal to the discarded card's mana value.
#[test]
fn blast_of_genius_discard_burn() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::grizzly_bears()); } // MV 2 to draw & discard
    let spell = g.add_card_to_hand(0, catalog::blast_of_genius());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    let opp = g.players[1].life;
    g.cast_spell(spell, Some(Target::Player(1)), vec![], None, None).expect("cast Blast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "damage equals discarded card's MV (2)");
}

/// Maze's End's activated search fetches a Gate; the win check keys on ten
/// differently-named Gates.
#[test]
fn mazes_end_fetches_gate() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let end = g.add_card_to_battlefield(0, catalog::mazes_end());
    let gate = g.add_card_to_library(0, catalog::dimir_guildgate());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(gate))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: end, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Maze's End");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Dimir Guildgate"), "fetched a Gate");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Maze's End"), "returned to hand");
}



// ── DGM gap wave 2 (Aetherling, Dragonshift, Krasis, Fuse splits) ───────────

/// Aetherling's {1}: +1/-1 pump adjusts its stats.
#[test]
fn aetherling_pump_ability() {
    let mut g = two_player_game();
    let ae = g.add_card_to_battlefield(0, catalog::aetherling());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ae, ability_index: 2, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate +1/-1");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ae).map(|c| (c.power, c.toughness)), Some((5, 4)), "+1/-1");
}

/// Dragonshift turns your creature into a 4/4 flying Dragon with no abilities.
#[test]
fn dragonshift_animates_dragon() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dragonshift());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.cast_spell(spell, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Dragonshift");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "becomes 4/4");
    assert!(cp.keywords.contains(&Keyword::Flying), "gains flying");
}

/// Krasis Incubation locks the creature down, then bounces itself for two
/// counters.
#[test]
fn krasis_incubation_lock_and_bounce() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::krasis_incubation());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.cast_spell(aura, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Krasis");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantAttack), "creature locked");
    // Activate the release ability.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Krasis release");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "two counters");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Krasis Incubation"), "aura returned to hand");
}

/// Armed (left half) pumps +1/+1 and grants double strike.
#[test]
fn armed_left_half() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::armed_dangerous());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.cast_spell(spell, Some(Target::Permanent(bear)), vec![], None, None).expect("cast Armed");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    assert!(cp.keywords.contains(&Keyword::DoubleStrike), "gains double strike");
}

/// Serve (right half) gives -6/-0.
#[test]
fn serve_right_half() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::protect_serve());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSplitRight {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Serve");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, -4, "2 - 6 = -4 power");
}

/// Down (left half) makes a player discard two.
#[test]
fn down_left_half() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::down_dirty());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let before = g.players[1].hand.len();
    g.cast_spell(spell, Some(Target::Player(1)), vec![], None, None).expect("cast Down");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 2, "discarded two");
}

/// Progenitor Mimic enters as a copy of a creature and mints a token copy of
/// itself each upkeep.
#[test]
fn progenitor_mimic_copies_and_spawns() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // the copy target
    let mimic = g.add_card_to_hand(0, catalog::progenitor_mimic());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.cast_spell(mimic, None, vec![], None, None).expect("cast Progenitor Mimic");
    drain_stack(&mut g);
    // Entered as a Grizzly Bears copy: now two Bears-named permanents.
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).count(),
        2, "Mimic entered as a copy of the bear",
    );
    // Upkeep: mint a token copy of itself (a third Bears-named permanent).
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).count(),
        3, "upkeep minted a token copy",
    );
}

/// Showstopper grants your creatures a death-ping; a dying creature deals 2 to
/// an opponent's creature.
#[test]
fn showstopper_death_ping() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::showstopper());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.cast_spell(spell, None, vec![], None, None).expect("cast Showstopper");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).unwrap().definition.card_types.contains(&CardType::Creature));
    // Kill my creature; its death ping deals 2 to the opponent's bear (dies).
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(mine), 2, None, &mut evs);
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "opponent's bear destroyed by the death ping");
}

/// Teysa destroys a creature that deals combat damage to her controller and
/// makes a Spirit.
#[test]
fn teysa_destroys_attacker_and_makes_spirit() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teysa_envoy_of_ghosts());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }])).expect("attack player 0");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "attacker destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Spirit" && c.controller == 0),
        "a Spirit token was created",
    );
}

/// Scab-Clan Giant fights an opponent's creature on entry.
#[test]
fn scab_clan_giant_fights_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let giant = g.move_card_to_battlefield_for_test(0, catalog::scab_clan_giant());
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 bear died to the 4/5 giant");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 2, "giant took 2 back");
}

/// Breaking mills eight; Entering reanimates a creature with haste.
#[test]
fn breaking_entering_halves() {
    let mut g = two_player_game();
    for _ in 0..10 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let breaking = g.add_card_to_hand(0, catalog::breaking_entering());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.cast_spell(breaking, Some(Target::Player(1)), vec![], None, None).expect("cast Breaking");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 8, "milled eight");

    // Entering: reanimate a creature from a graveyard with haste.
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let entering = g.add_card_to_hand(0, catalog::breaking_entering());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSplitRight {
        card_id: entering,
        target: Some(Target::Permanent(corpse)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("cast Entering");
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(corpse).expect("creature on battlefield");
    assert_eq!(reanimated.controller, 0, "under my control");
    assert!(g.computed_permanent(corpse).unwrap().keywords.contains(&Keyword::Haste), "has haste");
}

/// Council of the Absolute's chosen-name cost reduction shaves {2} off matching
/// spells and nothing off others.
#[test]
fn council_named_spell_cost_reduction() {
    use crabomination::card::CardInstance;
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let council = g.add_card_to_battlefield(0, catalog::council_of_the_absolute());
    g.battlefield_find_mut(council).unwrap().named_card = Some("Punish the Enemy".into());
    let named = CardInstance::new(g.next_id(), catalog::punish_the_enemy(), 0);
    let other = CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &named, None), 2, "chosen name costs {{2}} less");
    assert_eq!(cost_reduction_for_spell(&g, 0, &other, None), 0, "other spells unaffected");
}

/// Blaze Commando mints two Soldiers when your instant/sorcery deals damage,
/// once per resolution.
#[test]
fn blaze_commando_spell_damage_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blaze_commando());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.cast_spell(bolt, Some(Target::Player(1)), vec![], None, None).expect("cast Bolt");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier" && c.controller == 0).count();
    assert_eq!(soldiers, 2, "two Soldier tokens from the spell's damage");
}

/// Deadbridge Chant mills ten on ETB, then each upkeep pulls a random graveyard
/// card back (creature → battlefield, else → hand).
#[test]
fn deadbridge_chant_mill_and_upkeep() {
    let mut g = two_player_game();
    for _ in 0..12 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.move_card_to_battlefield_for_test(0, catalog::deadbridge_chant());
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 10, "milled ten");
    // Upkeep: only creature cards are in the graveyard, so one enters play.
    let bf_before = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let bf_after = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bf_after, bf_before + 1, "a creature was reanimated");
    assert_eq!(g.players[0].graveyard.len(), 9, "one left the graveyard");
}

/// Ral Zarek's +1 taps its target and bumps loyalty to 5.
#[test]
fn ral_zarek_plus_taps() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let ral = g.add_card_to_battlefield(0, catalog::ral_zarek());
    let untapped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tapped = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ral, ability_index: 0, target: Some(Target::Permanent(untapped)), x_value: None,
    }).expect("Ral +1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(untapped).unwrap().tapped, "target got tapped");
    assert_eq!(g.battlefield_find(ral).unwrap().counter_count(CounterType::Loyalty), 5, "4→5");
}

/// Ral Zarek's −2 deals 3 damage to any target.
#[test]
fn ral_zarek_minus_two_burn() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let ral = g.add_card_to_battlefield(0, catalog::ral_zarek());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ral, ability_index: 1, target: Some(Target::Player(1)), x_value: None,
    }).expect("Ral -2");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "dealt 3");
    assert_eq!(g.battlefield_find(ral).unwrap().counter_count(CounterType::Loyalty), 2, "4→2");
}

/// Emmara Tandris prevents all damage to your creature tokens (combat and
/// noncombat), but not to your nontoken creatures.
#[test]
fn emmara_tandris_shields_tokens() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::emmara_tandris());
    // A token creature under player 0.
    let token_def = TokenDefinition {
        name: "Elf Warrior".into(), power: 1, toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        ..Default::default()
    };
    let tok = g.add_token_to_battlefield(0, &token_def);
    let nontoken = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Noncombat damage: prevented on the token, applied to the nontoken.
    let mut evs = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(tok), 5, None, &mut evs);
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Permanent(nontoken), 1, None, &mut evs);
    assert_eq!(g.battlefield_find(tok).unwrap().damage, 0, "token damage prevented");
    assert_eq!(g.battlefield_find(nontoken).unwrap().damage, 1, "nontoken took damage");
}

/// Beck draws when a creature enters this turn; Call makes four Birds.
#[test]
fn beck_call_halves() {
    // Beck: install the watcher, then a creature entering draws a card.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let beck = g.add_card_to_hand(0, catalog::beck_call());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.cast_spell(beck, None, vec![], None, None).expect("cast Beck");
    drain_stack(&mut g);
    // Opt into Beck's "you may draw".
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    // Cast a creature through the real entry path so the watcher fires.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len() - 1; // the bear leaves hand on cast
    g.cast_spell(bear, None, vec![], None, None).expect("cast Grizzly Bears");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew when a creature entered");

    // Call: four 1/1 white Bird tokens with flying.
    let mut g = two_player_game();
    let call = g.add_card_to_hand(0, catalog::beck_call());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSplitRight {
        card_id: call, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Call");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Bird" && c.controller == 0).count(),
        4, "four Bird tokens",
    );
}

// ── DGM gap wave 4 ──────────────────────────────────────────────────────────

/// Notion Thief redirects an opponent's non-draw-step draw to its controller.
#[test]
fn notion_thief_redirects_extra_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::notion_thief());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    let mut ev = Vec::new();
    g.draw_one(0, &mut ev); // an extra draw by the opponent
    assert_eq!(g.players[0].hand.len(), h0, "opponent's extra draw is skipped");
    assert_eq!(g.players[1].hand.len(), h1 + 1, "thief draws instead");
}

/// Boros Battleshaper's each-combat trigger grants "attacks/blocks if able" to
/// one target and "can't attack or block" to another distinct creature (both
/// same-filter "target creature" slots are auto-filled).
#[test]
fn boros_battleshaper_forces_and_forbids() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::boros_battleshaper());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    // One creature is forced to attack/block, a distinct one is forbidden.
    let forced: Vec<_> = g.battlefield.iter()
        .filter(|c| g.computed_permanent(c.id).unwrap().keywords.contains(&Keyword::MustAttack))
        .map(|c| c.id).collect();
    let forbidden: Vec<_> = g.battlefield.iter()
        .filter(|c| g.computed_permanent(c.id).unwrap().keywords.contains(&Keyword::CantAttack))
        .map(|c| c.id).collect();
    assert_eq!(forced.len(), 1, "exactly one creature must attack/block");
    assert_eq!(forbidden.len(), 1, "exactly one creature can't attack/block");
    assert_ne!(forced[0], forbidden[0], "the two slots pick distinct creatures");
    assert!(
        g.computed_permanent(forced[0]).unwrap().keywords.contains(&Keyword::MustBlock),
        "forced creature also gains MustBlock",
    );
    assert!(
        g.computed_permanent(forbidden[0]).unwrap().keywords.contains(&Keyword::CantBlock),
        "forbidden creature also gains CantBlock",
    );
}

/// Varolz grants scavenge (cost = mana cost) to creature cards in your
/// graveyard; a non-owner or a Varolz-less graveyard has no granted ability.
#[test]
fn varolz_grants_scavenge_to_graveyard_creatures() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Without Varolz, a vanilla graveyard creature has no index-0 ability.
    let orphan = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: orphan, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
        }).is_err(),
        "no scavenge without Varolz",
    );

    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::varolz_the_scar_striped());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // 2 power
    let boost = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Granted scavenge sits at index 0 (Grizzly Bears prints no ability).
    g.perform_action(GameAction::ActivateAbility {
        card_id: dead, ability_index: 0, target: Some(Target::Permanent(boost)),
        additional_targets: vec![], x_value: None,
    }).expect("activate granted scavenge");
    drain_stack(&mut g);
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == dead), "scavenged card exiled");
    assert_eq!(
        g.battlefield_find(boost).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "+1/+1 counters equal to scavenged power",
    );
}

/// The turn-based first draw of the opponent's draw step is exempt.
#[test]
fn notion_thief_exempts_draw_step() {
    let mut g = two_player_game();
    g.set_skip_first_draw(false);
    g.step = TurnStep::Upkeep; // rewind before player 0's draw step
    g.add_card_to_battlefield(1, catalog::notion_thief());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    while g.step != TurnStep::Draw {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].hand.len(), h0 + 1, "active player keeps the draw-step draw");
    assert_eq!(g.players[1].hand.len(), h1, "thief does not steal the first draw");
}

// ── gaps wave 5 ─────────────────────────────────────────────────────────────

/// Melek casts instants off the library top and copies what it casts from
/// there — but not the same spell cast from hand.
#[test]
fn melek_copies_only_library_casts() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::melek_izzet_paragon());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast off the library top");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "3 from the spell + 3 from the copy");

    let hand_bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: hand_bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from hand");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 11, "hand casts aren't copied");
}

/// Plasm Capture banks the countered spell's mana value as any-colour mana at
/// the caster's next main phase.
#[test]
fn plasm_capture_banks_the_spells_mana_value() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let spell = g.add_card_to_hand(1, catalog::divination()); // {2}{U} — MV 3
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    let capture = g.add_card_to_hand(0, catalog::plasm_capture());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: capture,
        target: Some(Target::Permanent(spell)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    let banked = g
        .delayed_triggers
        .iter()
        .any(|d| matches!(d.kind, crabomination::game::types::DelayedKind::YourNextMainPhase));
    assert!(banked, "the mana is banked for the next main phase");
}

/// Goblin Test Pilot picks its victim at random from every legal object.
#[test]
fn goblin_test_pilot_hits_something_at_random() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let pilot = g.add_card_to_battlefield(0, catalog::goblin_test_pilot());
    g.clear_sickness(pilot);
    let before: i32 = g.players.iter().map(|p| p.life).sum();
    g.perform_action(GameAction::ActivateAbility {
        card_id: pilot,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("tap for 2");
    drain_stack(&mut g);
    let after: i32 = g.players.iter().map(|p| p.life).sum();
    // The pool is the two players plus the Pilot itself, so "somewhere" is a
    // life total, marked damage, or the 0/2 dying to its own shot.
    let hit_itself =
        g.battlefield_find(pilot).is_none_or(|c| c.damage > 0);
    assert!(after == before - 2 || hit_itself, "the 2 damage landed somewhere");
}

/// Release edicts one permanent of each of the five types from every player.
#[test]
fn catch_release_edicts_five_types() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in [0, 1] {
        g.add_card_to_battlefield(seat, catalog::grizzly_bears());
        g.add_card_to_battlefield(seat, catalog::forest());
    }
    let release = g.add_card_to_hand(0, catalog::catch_release());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSplitRight {
        card_id: release,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Release");
    drain_stack(&mut g);
    for seat in [0, 1] {
        assert_eq!(
            g.battlefield.iter().filter(|c| c.controller == seat).count(),
            0,
            "both the creature and the land were sacrificed"
        );
    }
}
