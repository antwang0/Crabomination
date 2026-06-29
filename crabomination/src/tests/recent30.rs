//! Functionality tests for `catalog::sets::decks::recent30` — Aetherdrift
//! (DFT) staples: Vehicles/Crew, Mounts/Saddle, Exhaust, Start your engines!,
//! reanimation, modal removal, and graveyard value.

use crate::catalog;
use crate::card::{CardId, CounterType, Keyword};
use crate::game::types::{Attack, AttackTarget, Target};
use crate::game::*;
use crate::TurnStep;

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield
        .iter()
        .filter(|c| c.controller == controller && c.definition.name == name)
        .count()
}

fn ready(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..10 {
        g.players[0].mana_pool.add_colorless(1);
    }
    for c in [crate::mana::Color::White, crate::mana::Color::Blue, crate::mana::Color::Black,
              crate::mana::Color::Red, crate::mana::Color::Green]
    {
        g.players[0].mana_pool.add(c, 4);
    }
}

fn etb_bf(g: &mut GameState, player: usize, def: crate::card::CardDefinition) -> CardId {
    let id = g.move_card_to_battlefield_for_test(player, def);
    drain_stack(g);
    id
}

/// Burner Rocket pumps a creature you control and grants trample on entry.
#[test]
fn burner_rocket_etb_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    etb_bf(&mut g, 0, catalog::burner_rocket());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "+2/+0");
    assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
}

/// Broadcast Rambler makes a Thopter on entry.
#[test]
fn broadcast_rambler_etb_thopter() {
    let mut g = two_player_game();
    etb_bf(&mut g, 0, catalog::broadcast_rambler());
    assert_eq!(count_named(&g, 0, "Thopter"), 1, "one 1/1 Thopter");
}

/// Carrion Cruiser mills two and returns a creature card from the graveyard.
#[test]
fn carrion_cruiser_etb_recursion() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    // The "return a creature/Vehicle" pick is offered as up-to-1, so script it.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bear])]));
    let hand = g.players[0].hand.len();
    etb_bf(&mut g, 0, catalog::carrion_cruiser());
    assert_eq!(g.players[0].hand.len(), hand + 1, "returned a milled creature to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the chosen creature");
}

/// Clamorous Ironclad is a Crew 3 Vehicle with menace.
#[test]
fn clamorous_ironclad_shape() {
    let mut g = two_player_game();
    let ci = g.add_card_to_battlefield(0, catalog::clamorous_ironclad());
    let cp = g.computed_permanent(ci).unwrap();
    assert!(cp.keywords.contains(&Keyword::Crew(3)));
    assert!(cp.keywords.contains(&Keyword::Menace));
}

/// Alacrian Jaguar pumps itself when it attacks while saddled.
#[test]
fn alacrian_jaguar_saddled_pump() {
    let mut g = two_player_game();
    let jag = g.add_card_to_battlefield(0, catalog::alacrian_jaguar());
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(jag);
    g.clear_sickness(helper);
    ready(&mut g);
    g.perform_action(GameAction::Saddle { mount: jag, creatures: vec![helper] }).expect("saddle");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: jag, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(jag).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "+2/+2 while saddled");
}

/// District Mascot enters as a 1/1 (a +1/+1 counter on a 0/0).
#[test]
fn district_mascot_enters_with_counter() {
    let mut g = two_player_game();
    let dm = etb_bf(&mut g, 0, catalog::district_mascot());
    let cp = g.computed_permanent(dm).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "0/0 + a +1/+1 counter");
}

/// Bulwark Ox's sacrifice gives your counter-bearing creatures indestructible.
#[test]
fn bulwark_ox_sac_protects_counter_creatures() {
    let mut g = two_player_game();
    let ox = g.add_card_to_battlefield(0, catalog::bulwark_ox());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ox, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac ability");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Indestructible), "counter-creature gained indestructible");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "and hexproof");
}

/// Autarch Mammoth makes an Elephant on entry.
#[test]
fn autarch_mammoth_etb_elephant() {
    let mut g = two_player_game();
    etb_bf(&mut g, 0, catalog::autarch_mammoth());
    assert_eq!(count_named(&g, 0, "Elephant"), 1, "a 3/3 Elephant");
}

/// Elvish Refueler's Exhaust ability grows it once; a second use is illegal.
#[test]
fn elvish_refueler_exhaust_once() {
    let mut g = two_player_game();
    let er = g.add_card_to_battlefield(0, catalog::elvish_refueler());
    ready(&mut g);
    g.perform_action(GameAction::ActivateAbility {
        card_id: er, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("first exhaust");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(er).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let second = g.perform_action(GameAction::ActivateAbility {
        card_id: er, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(second.is_err(), "exhaust can be used only once");
}

/// Endrider Catalyzer ships with Start your engines!; its mana ability is
/// gated behind max speed (illegal at speed 0).
#[test]
fn endrider_catalyzer_max_speed_gate() {
    let mut g = two_player_game();
    let ec = g.add_card_to_battlefield(0, catalog::endrider_catalyzer());
    g.clear_sickness(ec);
    assert!(g.computed_permanent(ec).unwrap().keywords.contains(&Keyword::StartYourEngines));
    ready(&mut g);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: ec, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    });
    assert!(res.is_err(), "max-speed ability is illegal below speed 4");
}

/// Collision Course deals damage equal to your creature/Vehicle count.
#[test]
fn collision_course_burns_for_board() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // three creatures → X=3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let cc = g.add_card_to_hand(0, catalog::collision_course());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: cc,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "3 damage killed the 2/2");
}

/// Back on Track reanimates a creature and leaves a Pilot behind.
#[test]
fn back_on_track_reanimates() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let bt = g.add_card_to_hand(0, catalog::back_on_track());
    ready(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bt,
        target: Some(Target::Permanent(dead)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some(), "creature reanimated");
    assert_eq!(count_named(&g, 0, "Pilot"), 1, "a 1/1 Pilot");
}

/// Dredger's Insight mills four on entry and lets you take a card.
#[test]
fn dredgers_insight_etb_mill() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let hand = g.players[0].hand.len();
    etb_bf(&mut g, 0, catalog::dredgers_insight());
    assert_eq!(g.players[0].hand.len(), hand + 1, "took a creature from the milled cards");
}

/// Aether Syphon taps for a card.
#[test]
fn aether_syphon_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let asy = g.add_card_to_battlefield(0, catalog::aether_syphon());
    ready(&mut g);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: asy, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("draw ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Alacrian Armory is an anthem: +0/+1 and vigilance for your creatures.
#[test]
fn alacrian_armory_anthem() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::alacrian_armory());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+0/+1");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "gained vigilance");
}

/// Dracosaur Auxiliary pings any target when it attacks while saddled.
#[test]
fn dracosaur_auxiliary_saddled_ping() {
    let mut g = two_player_game();
    let drac = g.add_card_to_battlefield(0, catalog::dracosaur_auxiliary());
    let helper1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let helper2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(drac);
    g.clear_sickness(helper1);
    g.clear_sickness(helper2);
    ready(&mut g);
    g.perform_action(GameAction::Saddle { mount: drac, creatures: vec![helper1, helper2] })
        .expect("saddle 3 via two 2-power bears");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.declare_attackers(vec![Attack { attacker: drac, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "dealt 2 to the opponent");
}

/// Detention Chariot exiles an opponent's creature until it leaves.
#[test]
fn detention_chariot_exiles_until_leaves() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dc = etb_bf(&mut g, 0, catalog::detention_chariot());
    assert!(g.battlefield_find(foe).is_none(), "opponent's creature exiled");
    g.remove_to_graveyard_with_triggers(dc);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 1),
        "returns when the Chariot leaves");
}

/// Endrider Spikespitter ships with reach and Start your engines!
#[test]
fn endrider_spikespitter_shape() {
    let mut g = two_player_game();
    let es = g.add_card_to_battlefield(0, catalog::endrider_spikespitter());
    let cp = g.computed_permanent(es).unwrap();
    assert!(cp.keywords.contains(&Keyword::Reach));
    assert!(cp.keywords.contains(&Keyword::StartYourEngines));
}
