//! Commander: EDHREC average-deck gaps, second batch (`decks::cmdr_edhrec2`):
//! Henzie "Toolbox" Torre's and Frodo + Sam's lists.

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

/// A pod with empty libraries (each test stacks its own) and seat 0 active.
fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(g: &mut GameState, id: CardId) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn connect(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(g);
    }
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Ancient Brass Dragon: the d20 sets the budget, CR 601.2c checks it across
/// every target, and either graveyard is open (CR 603.7 reflexive trigger).
#[test]
fn ancient_brass_dragon_reanimates_within_the_roll() {
    let mut g = pod(2);
    let dragon = ready(&mut g, 0, catalog::ancient_brass_dragon());
    let angel = g.add_card_to_graveyard(0, catalog::serra_angel());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let elf = g.add_card_to_graveyard(1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(3)]));
    connect(&mut g, dragon);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == angel), "mana value 5 is over a roll of 3");
    let back: Vec<CardId> = [bear, elf].into_iter().filter(|id| g.battlefield_find(*id).is_some()).collect();
    assert_eq!(back.len(), 2, "bear (2) + elf (1) fit a budget of 3");
    assert!(back.iter().all(|id| g.battlefield_find(*id).unwrap().controller == 0));
}

/// Ojer Kaslem's hit puts at most one creature and one land from the reveal
/// onto the battlefield; the rest go to the bottom.
#[test]
fn ojer_kaslem_takes_a_creature_and_a_land() {
    let mut g = pod(2);
    let ojer = ready(&mut g, 0, catalog::ojer_kaslem_deepest_growth());
    for f in [catalog::grizzly_bears, catalog::llanowar_elves, catalog::forest, catalog::forest, catalog::island, catalog::murder] {
        g.add_card_to_library(0, f());
    }
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    connect(&mut g, ojer);
    assert_eq!(named(&g, "Grizzly Bears") + named(&g, "Llanowar Elves"), 1, "one creature, not two");
    assert_eq!(named(&g, "Forest") + named(&g, "Island"), 1, "one land");
    assert_eq!(g.players[0].library.len(), 8);
    assert_eq!(g.players[0].library.iter().next().unwrap().definition.name, "Lightning Bolt", "the rest bottomed");
}

/// Ojer Kaslem dies and returns as Temple of Cultivation, tapped.
#[test]
fn ojer_kaslem_returns_as_the_temple() {
    let mut g = pod(2);
    let ojer = ready(&mut g, 0, catalog::ojer_kaslem_deepest_growth());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(ojer)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("murder");
    drain_stack(&mut g);
    let temple = g.battlefield.iter().find(|c| c.definition.name == "Temple of Cultivation").expect("returned");
    assert!(temple.tapped);
}

/// Primeval Herald fetches a basic as it enters and again as it attacks.
#[test]
fn primeval_herald_fetches_on_entering_and_attacking() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let herald = g.add_card_to_hand(0, catalog::primeval_herald());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, herald);
    assert_eq!(named(&g, "Forest"), 1);
    g.clear_sickness(herald);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    connect(&mut g, herald);
    assert_eq!(named(&g, "Forest"), 2);
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Forest").all(|c| c.tapped));
}

/// Seedguide Ash's death fetches up to three Forest cards, tapped.
#[test]
fn seedguide_ash_dies_into_three_forests() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let ash = ready(&mut g, 0, catalog::seedguide_ash());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(ash)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("murder");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Forest"), 3);
    assert_eq!(g.players[0].library.len(), 1);
}

/// Birthing Ritual: sacrificing a 2-drop lets a creature card of mana value
/// 3 or less from the top seven in; the angel (5) is out of reach.
#[test]
fn birthing_ritual_upgrades_by_one_mana_value() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::birthing_ritual());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for f in [catalog::serra_angel, catalog::island, catalog::llanowar_elves] {
        g.add_card_to_library(0, f());
    }
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] }, DecisionAnswer::Bool(true)]));
    to_end_step(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "sacrificed");
    assert_eq!(named(&g, "Llanowar Elves"), 1);
    assert_eq!(named(&g, "Serra Angel"), 0);
    assert_eq!(g.players[0].library.len(), 9);
    assert_eq!(g.players[0].library.iter().next().unwrap().definition.name, "Lightning Bolt", "the rest bottomed");
}

/// Declining the sacrifice still bottoms the seven looked at.
#[test]
fn birthing_ritual_declined_still_bottoms_the_seven() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::birthing_ritual());
    ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] }, DecisionAnswer::Bool(false)]));
    to_end_step(&mut g);
    assert_eq!(named(&g, "Grizzly Bears"), 1);
    assert_eq!(g.players[0].library.len(), 8);
    assert_eq!(g.players[0].library.iter().next().unwrap().definition.name, "Forest");
}

/// Bag End Banquet's three Foods enter at once: Belladonna Took resolves
/// three times (life, then a card, then counters), and Bilbo turns each Food
/// into a Food and a Treasure. The Banquet then taps for {C} per Food.
#[test]
fn bag_end_banquet_with_belladonna_and_bilbo() {
    let mut g = pod(2);
    let bella = ready(&mut g, 0, catalog::belladonna_took());
    ready(&mut g, 0, catalog::bilbo_fellow_conspirator());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let banquet = g.add_card_to_hand(0, catalog::bag_end_banquet());
    flood(&mut g);
    let life = g.players[0].life;
    cast(&mut g, banquet);
    assert_eq!([named(&g, "Food"), named(&g, "Treasure")], [3, 3]);
    assert_eq!(g.players[0].life, life + 1);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.battlefield_find(bella).unwrap().power(), 3, "the third resolution's counter");
    let before = g.players[0].mana_pool.colorless_amount();
    g.perform_action(GameAction::ActivateAbility {
        card_id: banquet,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), before + 3);
}

/// The Sackville-Bagginses: sacrificing a Treasure draws, makes a new
/// Treasure, and — a token was sacrificed — drains an opponent for 1.
#[test]
fn sackville_bagginses_cash_in_a_treasure() {
    let mut g = pod(2);
    let t = crabomination_base::tokens::treasure_token();
    g.add_token_to_battlefield(0, &t);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let sb = g.add_card_to_hand(0, catalog::the_sackville_bagginses());
    flood(&mut g);
    let opp = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, sb);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(named(&g, "Treasure"), 1, "one sacrificed, one made");
    assert_eq!(g.players[1].life, opp - 1);
}

fn activate(g: &mut GameState, id: CardId, ability_index: usize) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Asmodeus: CR 121.2a — each of the seven draws is replaced by a face-down
/// exile with Asmodeus; {B} returns them all and costs that much life.
#[test]
fn asmodeus_banks_draws_and_cashes_them_in() {
    let mut g = pod(2);
    let asmo = ready(&mut g, 0, catalog::asmodeus_the_archfiend());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    flood(&mut g);
    activate(&mut g, asmo, 0);
    assert_eq!(g.players[0].hand.len(), 0);
    assert_eq!(g.players[0].library.len(), 3);
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(asmo) && c.face_down).count(), 7);
    let life = g.players[0].life;
    activate(&mut g, asmo, 1);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[0].life, life - 7);
}

/// An empty library under Asmodeus replaces the draw with nothing: no
/// CR 104.3c loss.
#[test]
fn asmodeus_empty_library_draw_does_not_lose() {
    let mut g = pod(2);
    let asmo = ready(&mut g, 0, catalog::asmodeus_the_archfiend());
    flood(&mut g);
    activate(&mut g, asmo, 0);
    assert!(g.players[0].is_alive());
}

/// Hoarding Broodlord tutors a card into face-down exile, playable from there.
#[test]
fn hoarding_broodlord_tutors_into_exile_and_casts_it() {
    let mut g = pod(2);
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let lord = g.add_card_to_hand(0, catalog::hoarding_broodlord());
    flood(&mut g);
    cast(&mut g, lord);
    assert!(g.exile.iter().any(|c| c.id == bolt && c.face_down));
    let life = g.players[1].life;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt from exile");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
}

/// Razaketh: 2 life and another creature tutor a card to hand.
#[test]
fn razaketh_sacrifices_and_tutors() {
    let mut g = pod(2);
    let raz = ready(&mut g, 0, catalog::razaketh_the_foulblooded());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::lightning_bolt());
    let life = g.players[0].life;
    activate(&mut g, raz, 0);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Beseech the Queen: the cap is the land count (three), so the angel (5)
/// can't be found; {2/B} pips pay as {B} (CR 107.4e).
#[test]
fn beseech_the_queen_caps_by_lands() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::swamp());
    }
    g.add_card_to_library(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bq = g.add_card_to_hand(0, catalog::beseech_the_queen());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, bq);
    let hand: Vec<&str> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(hand, ["Grizzly Bears"]);
}

/// Dark Petition's spell mastery (CR 207.2c) adds {B}{B}{B} with two
/// instants/sorceries in the graveyard, and not with one.
#[test]
fn dark_petition_spell_mastery_refunds() {
    for (in_gy, refund) in [(2, 3), (1, 0)] {
        let mut g = pod(2);
        for _ in 0..in_gy {
            g.add_card_to_graveyard(0, catalog::lightning_bolt());
        }
        g.add_card_to_library(0, catalog::island());
        let dp = g.add_card_to_hand(0, catalog::dark_petition());
        g.players[0].mana_pool.add(Color::Black, 5);
        cast(&mut g, dp);
        assert_eq!(g.players[0].mana_pool.amount(Color::Black), refund, "{in_gy} in graveyard");
    }
}

fn to_upkeep(g: &mut GameState) {
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Bramblewood Paragon: another Warrior enters with a +1/+1 counter and so
/// has trample; the Paragon itself does not ("each other").
#[test]
fn bramblewood_paragon_counters_other_warriors() {
    let mut g = pod(2);
    let paragon = g.add_card_to_hand(0, catalog::bramblewood_paragon());
    let second = g.add_card_to_hand(0, catalog::bramblewood_paragon());
    flood(&mut g);
    cast(&mut g, paragon);
    assert_eq!(g.battlefield_find(paragon).unwrap().power(), 2);
    cast(&mut g, second);
    let c = g.battlefield_find(second).unwrap();
    assert_eq!(c.power(), 3);
    assert!(g.computed_permanent(second).unwrap().keywords().contains(&crabomination::card::Keyword::Trample));
    assert!(!g.computed_permanent(paragon).unwrap().keywords().contains(&crabomination::card::Keyword::Trample));
}

/// Cemetery Prowler exiles a creature card as it enters; a creature spell
/// then costs {1} less (one shared card type): Grizzly Bears for {G}.
#[test]
fn cemetery_prowler_discounts_by_shared_type() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let prowler = g.add_card_to_hand(0, catalog::cemetery_prowler());
    flood(&mut g);
    cast(&mut g, prowler);
    assert!(g.players[1].graveyard.is_empty(), "exiled from the opponent's graveyard");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Green, 1);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears);
    assert!(g.battlefield_find(bears).is_some());
}

/// Hollowhenge Overlord: one Wolf per Wolf/Werewolf at upkeep (itself counts).
#[test]
fn hollowhenge_overlord_doubles_the_pack() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hollowhenge_overlord());
    ready(&mut g, 0, catalog::cemetery_prowler());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    to_upkeep(&mut g);
    assert_eq!(named(&g, "Wolf"), 2);
}

/// Wolf-Skull Shaman's kinship: an Elf on top makes a Wolf; a land doesn't.
#[test]
fn wolf_skull_shaman_kinship() {
    for (top, wolves) in [(catalog::llanowar_elves as fn() -> _, 1), (catalog::forest, 0)] {
        let mut g = pod(2);
        ready(&mut g, 0, catalog::wolf_skull_shaman());
        g.add_card_to_library(0, top());
        g.add_card_to_library(0, catalog::forest());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        to_upkeep(&mut g);
        assert_eq!(named(&g, "Wolf"), wolves);
    }
}

/// Howling Moon: an opponent's second spell in a turn makes a Wolf; the
/// first does not.
#[test]
fn howling_moon_punishes_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::howling_moon());
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, "Wolf"), 1);
}

/// Chocobo Camp's mana rider: the next Bird creature spell this turn enters
/// with a +1/+1 counter (CR 603.7e); a non-Bird cast doesn't spend it.
#[test]
fn chocobo_camp_counters_the_next_bird() {
    let mut g = pod(2);
    let camp = ready(&mut g, 0, catalog::chocobo_camp());
    g.perform_action(GameAction::ActivateAbility {
        card_id: camp,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    flood(&mut g);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears);
    assert_eq!(g.battlefield_find(bears).unwrap().power(), 2);
    let watcher = g.add_card_to_hand(0, catalog::watcher_of_the_spheres());
    cast(&mut g, watcher);
    assert_eq!(g.battlefield_find(watcher).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Gwaihir costs {2} less after two draws this turn.
#[test]
fn gwaihir_discounted_after_two_draws() {
    let mut g = pod(2);
    let gw = g.add_card_to_hand(0, catalog::gwaihir_the_windlord());
    g.players[0].cards_drawn_this_turn = 2;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, gw);
    assert!(g.battlefield_find(gw).is_some());
}

/// Tawnos copies a Bird creature spell; the copy resolves into an artifact
/// creature token (CR 707.10f).
#[test]
fn tawnos_copies_a_bird_as_an_artifact() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::tawnos_the_toymaker());
    let watcher = g.add_card_to_hand(0, catalog::watcher_of_the_spheres());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, watcher);
    let copies: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Watcher of the Spheres").collect();
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().any(|c| c.is_token && c.definition.is_artifact()));
}

/// The Lord of the Eagles costs {X} less, X the total power of your fliers.
#[test]
fn lord_of_the_eagles_discounted_by_fliers() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serra_angel());
    let lord = g.add_card_to_hand(0, catalog::the_lord_of_the_eagles());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, lord);
    assert!(g.battlefield_find(lord).is_some(), "{{7}}{{U}}{{U}} less 4");
}

/// Flurry of Wings makes one Bird Soldier per attacking creature.
#[test]
fn flurry_of_wings_counts_attackers() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let fw = g.add_card_to_hand(0, catalog::flurry_of_wings());
    flood(&mut g);
    cast(&mut g, fw);
    assert_eq!(named(&g, "Bird Soldier"), 2);
}

fn enchant(g: &mut GameState, aura: crabomination::card::CardDefinition, host: CardId) -> CardId {
    let id = g.add_card_to_hand(0, aura);
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("aura");
    drain_stack(g);
    id
}

/// Armored Ascension counts Plains; Battle Mastery adds double strike; With
/// Great Power counts both Auras (+2/+2 each, itself included).
#[test]
fn light_paws_auras_stack_up() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::plains());
    }
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    enchant(&mut g, catalog::armored_ascension(), bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5);
    enchant(&mut g, catalog::battle_mastery(), bear);
    let kw = g.computed_permanent(bear).unwrap().keywords().to_vec();
    assert!(kw.contains(&crabomination::card::Keyword::Flying) && kw.contains(&crabomination::card::Keyword::DoubleStrike));
    enchant(&mut g, catalog::with_great_power(), bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 5 + 6, "three Auras attached, +2/+2 each");
}

/// With Great Power . . . sends damage dealt to you onto the enchanted
/// creature.
#[test]
fn with_great_power_redirects_your_damage() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    enchant(&mut g, catalog::with_great_power(), bear);
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert_eq!(g.players[0].life, life);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 3);
}

/// Helm of the Gods: +1/+1 per enchantment you control.
#[test]
fn helm_of_the_gods_counts_enchantments() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::howling_moon());
    ready(&mut g, 0, catalog::birthing_ritual());
    let helm = ready(&mut g, 0, catalog::helm_of_the_gods());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: helm, target: bear }).expect("equip");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// Benevolent Blessing (CR 702.16k): protection from white keeps your own
/// white Aura on the creature, but sheds an opponent's.
#[test]
fn benevolent_blessing_keeps_your_auras() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mine = enchant(&mut g, catalog::battle_mastery(), bear);
    let theirs = g.add_card_to_battlefield(1, catalog::battle_mastery());
    g.battlefield_find_mut(theirs).unwrap().attached_to = Some(bear);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::White)]));
    enchant(&mut g, catalog::benevolent_blessing(), bear);
    g.check_state_based_actions();
    assert!(g.battlefield_find(mine).is_some_and(|c| c.attached_to == Some(bear)));
    assert!(g.battlefield_find(theirs).is_none(), "an opponent's white Aura falls off");
}

/// Rebuff the Wicked counters a spell aimed at your permanent, and can't
/// target one aimed elsewhere.
#[test]
fn rebuff_the_wicked_guards_your_permanents() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let rebuff = g.add_card_to_hand(0, catalog::rebuff_the_wicked());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rebuff,
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("rebuff");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// Yurlok: everyone adds {B}{R}{G}; unspent, it burns each player for 3 as
/// the step ends — except a Horizon Stone controller, whose mana turns
/// colorless instead (CR 106.4 override) and so isn't lost.
#[test]
fn yurlok_burns_unspent_mana_and_horizon_stone_spares_it() {
    let mut g = pod(3);
    let yurlok = ready(&mut g, 0, catalog::yurlok_of_scorch_thrash());
    ready(&mut g, 2, catalog::horizon_stone());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: yurlok,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].mana_pool.total(), 3);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    for _ in 0..3 {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_ne!(g.step, TurnStep::PreCombatMain);
    assert_eq!(g.players[0].life, life[0] - 3);
    assert_eq!(g.players[1].life, life[1] - 3);
    assert_eq!(g.players[2].life, life[2], "Horizon Stone: the mana became colorless");
    assert_eq!(g.players[2].mana_pool.total(), 3);
}

/// Rug of Smothering: the third spell in a turn costs its caster 3 life.
#[test]
fn rug_of_smothering_taxes_each_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rug_of_smothering());
    let life = g.players[1].life;
    for _ in 0..3 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, life - 6, "1 + 2 + 3");
}

/// Power Surge counts the lands that were untapped as the turn began, not
/// the ones the untap step untapped.
#[test]
fn power_surge_reads_lands_untapped_at_turn_start() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::power_surge());
    let a = ready(&mut g, 0, catalog::forest());
    ready(&mut g, 0, catalog::forest());
    g.battlefield_find_mut(a).unwrap().tapped = true;
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    let life = g.players[0].life;
    // Seat 1's end step, so seat 0's turn begins (and untaps) for real.
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    g.priority.player_with_priority = 1;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1);
}

/// Belbe: in the postcombat main phase the active player adds {C}{C} per
/// opponent of Belbe's controller who lost life this turn.
#[test]
fn belbe_pays_out_for_opponents_who_lost_life() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::belbe_corrupted_observer());
    g.players[1].lost_life_this_turn = true;
    g.players[2].lost_life_this_turn = true;
    g.step = TurnStep::EndCombat;
    g.priority.player_with_priority = 0;
    for _ in 0..3 {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 4);
}

/// Umbral Mantle's granted {3}, {Q} pump: untap the equipped creature for
/// +2/+2.
#[test]
fn umbral_mantle_grants_an_untap_pump() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let mantle = ready(&mut g, 0, catalog::umbral_mantle());
    g.perform_action(GameAction::Equip { equipment: mantle, target: bear }).expect("equip");
    drain_stack(&mut g);
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert!(!b.tapped);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
}

/// Exile `def` for `seat` with a may-play permission this turn.
fn exiled_playable(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_exile(seat, def);
    let turn = g.turn_number;
    let c = g.exile.iter_mut().find(|c| c.id == id).unwrap();
    c.may_play_until = Some(crabomination::card::MayPlayPermission {
        player: seat,
        granted_turn: turn,
        duration: crabomination::card::MayPlayDuration::EndOfThisTurn,
        exile_after: false,
        miracle: false,
        pay_life: false,
        cast_only: false,
        locks_further_casts: false,
        one_cast_group: None,
        bottom_after: false,
        undaunted: false,
    });
    id
}

/// Rocco: at your end step every player exiles their top card and may play
/// it until Rocco's controller's next end step — an opponent's window is
/// bound to seat 0, not to their own end step.
#[test]
fn rocco_exiles_for_each_player_until_your_next_end_step() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::rocco_street_chef());
    for seat in 0..3 {
        g.add_card_to_library(seat, catalog::lightning_bolt());
    }
    to_end_step(&mut g);
    for seat in 0..3 {
        let c = g.exile.iter().find(|c| c.owner == seat).expect("exiled");
        let p = c.may_play_until.expect("playable");
        assert_eq!(p.player, seat);
        assert_eq!(p.duration, crabomination::card::MayPlayDuration::UntilSeatsNextEndStep { seat: 0 });
    }
}

/// Rocco's payoff: an opponent casting a spell from exile still feeds you a
/// +1/+1 counter and a Food.
#[test]
fn rocco_pays_off_an_opponents_cast_from_exile() {
    let mut g = pod(2);
    let rocco = ready(&mut g, 0, catalog::rocco_street_chef());
    let bolt = exiled_playable(&mut g, 1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt from exile");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Food"), 1);
    assert_eq!(g.battlefield_find(rocco).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Pia Nalaar: a land played from exile makes a Thopter; one from hand
/// doesn't.
#[test]
fn pia_nalaar_rewards_a_land_played_from_exile() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::pia_nalaar_consul_of_revival());
    let land = exiled_playable(&mut g, 0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land from exile");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Thopter"), 1);
    g.players[0].lands_played_this_turn = 0;
    let hand_land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(hand_land)).expect("land from hand");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Thopter"), 1);
}

/// Urabrask: an opponent's upkeep turns their draw-step draw into an exile
/// they may play this turn (CR 121.2a).
#[test]
fn urabrask_turns_an_opponents_draw_into_an_impulse() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::urabrask_heretic_praetor());
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.step = TurnStep::End;
    while !(g.active_player_idx == 1 && g.step == TurnStep::PreCombatMain) {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].hand.len(), 0, "the draw was replaced");
    let c = g.exile.iter().find(|c| c.owner == 1).expect("exiled instead");
    assert_eq!(c.may_play_until.unwrap().player, 1);
}

/// Avatar's Wrath spares its target, airbends the rest, and locks opponents
/// out of casting from outside their hands until your next turn.
#[test]
fn avatars_wrath_spares_one_and_locks_non_hand_casts() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::avatars_wrath());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("wrath");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some());
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.exile.iter().any(|c| c.id == wrath), "Exile Avatar's Wrath");
    let def = catalog::lightning_bolt();
    assert!(g.cast_from_zone_blocked(1, &def, crabomination::card::Zone::Exile));
    assert!(!g.cast_from_zone_blocked(0, &def, crabomination::card::Zone::Exile));
    assert!(!g.cast_from_zone_blocked(1, &def, crabomination::card::Zone::Hand));
}

/// Quintorius Kand: a spell cast from exile drains each opponent for 2.
#[test]
fn quintorius_kand_drains_on_casts_from_exile() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::quintorius_kand());
    let bolt = exiled_playable(&mut g, 0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt from exile");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life[0] + 2);
    assert_eq!(g.players[1].life, life[1] - 5);
    assert_eq!(g.players[2].life, life[2] - 2);
}

// ── Target-deck residuals fixed this run (pod_residuals named them) ──────────

/// Ossification enchants a basic land you control (CR 303.4a — the Aura
/// spell's target) and its trigger exiles an opponent's creature.
#[test]
fn ossification_enchants_your_basic_land() {
    let mut g = pod(2);
    let plains = ready(&mut g, 0, catalog::plains());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let oss = enchant(&mut g, catalog::ossification(), plains);
    assert_eq!(g.battlefield_find(oss).unwrap().attached_to, Some(plains));
    assert!(g.battlefield_find(theirs).is_none());
    let their_plains = ready(&mut g, 1, catalog::plains());
    let again = g.add_card_to_hand(0, catalog::ossification());
    flood(&mut g);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: again,
            target: Some(Target::Permanent(their_plains)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "not a land you control",
    );
}

/// Shardmage's Rescue: hexproof only during the turn it entered; the +1/+1
/// stays.
#[test]
fn shardmages_rescue_hexproof_lasts_its_entry_turn() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    enchant(&mut g, catalog::shardmages_rescue(), bear);
    let hexproof = |g: &GameState| g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Hexproof);
    assert!(hexproof(&g));
    g.turn_number += 1;
    assert!(!hexproof(&g));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// War's Toll (CR 508.1d): an opponent may attack with nothing, but once one
/// of their creatures attacks, all of them able to must.
#[test]
fn wars_toll_all_or_nothing_attacks() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::wars_toll());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let mut solo = g.clone();
    assert!(
        solo.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: a, target: AttackTarget::Player(0) }]))
            .is_err(),
        "one attacking drags the other in",
    );
    g.perform_action(GameAction::DeclareAttackers(vec![])).expect("no attack is fine");
}

/// Ragost: your artifacts are Foods with the sac-for-3-life ability.
#[test]
fn ragost_makes_your_artifacts_food() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ragost_deft_gastronaut());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let food = crabomination::card::ArtifactSubtype::Food;
    assert!(g.computed_permanent(ring).unwrap().subtypes().artifact_subtypes.contains(&food));
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("granted food ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.battlefield_find(ring).is_none());
}

// ── Tinybones, Bauble Burglar (seat 343) ─────────────────────────────────────

/// Aclazotz's attack: an opponent with an empty hand can't discard, so you
/// draw; a discarded land makes a flying Bat. Dying returns it as the Temple.
#[test]
fn aclazotz_punishes_empty_hands_and_returns_as_the_temple() {
    let mut g = pod(3);
    let acl = ready(&mut g, 0, catalog::aclazotz_deepest_betrayal());
    g.add_card_to_hand(1, catalog::forest());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    connect(&mut g, acl);
    assert_eq!(g.players[0].hand.len(), 1, "seat 2 couldn't discard");
    assert_eq!(named(&g, "Bat"), 1, "seat 1 discarded a land");
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    g.step = TurnStep::PostCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(acl)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("murder");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Temple of the Dead" && c.tapped));
}

/// Fell Specter: its own discard costs the opponent 2 life too.
#[test]
fn fell_specter_drains_on_each_opponent_discard() {
    let mut g = pod(2);
    g.add_card_to_hand(1, catalog::island());
    let spec = g.add_card_to_hand(0, catalog::fell_specter());
    flood(&mut g);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell { card_id: spec, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!(g.players[1].life, life - 2);
}

/// Tinybones, Pocket Nuisance (CR 603.2c): its entry makes both opponents
/// discard — one "one or more" event per player, so two triggers.
#[test]
fn tinybones_pocket_nuisance_pings_per_discard_batch() {
    let mut g = pod(3);
    for seat in 1..3 {
        g.add_card_to_hand(seat, catalog::island());
    }
    let tb = g.add_card_to_hand(0, catalog::tinybones_pocket_nuisance());
    flood(&mut g);
    let life = g.players[1].life;
    cast(&mut g, tb);
    assert_eq!(g.players[1].life, life - 2, "two discard batches, 1 each");
}

/// The Raven Man makes a Bird at the end step only after a discard.
#[test]
fn the_raven_man_needs_a_discard() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_raven_man());
    to_end_step(&mut g);
    assert_eq!(named(&g, "Bird"), 0);
    let mut g = pod(2);
    let raven = ready(&mut g, 0, catalog::the_raven_man());
    g.add_card_to_hand(1, catalog::island());
    flood(&mut g);
    activate(&mut g, raven, 0);
    to_end_step(&mut g);
    assert_eq!(named(&g, "Bird"), 1);
}

/// Tinybones, Trinket Thief: only opponents with empty hands lose 10.
#[test]
fn tinybones_trinket_thief_hits_empty_hands() {
    let mut g = pod(3);
    let tt = ready(&mut g, 0, catalog::tinybones_trinket_thief());
    g.add_card_to_hand(2, catalog::island());
    flood(&mut g);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    activate(&mut g, tt, 0);
    assert_eq!(g.players[1].life, life[1] - 10);
    assert_eq!(g.players[2].life, life[2]);
}

/// Mind Rake overloaded (CR 702.96): each player — you too — discards two.
#[test]
fn mind_rake_overload_hits_every_player() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_hand(seat, catalog::island());
        }
    }
    let rake = g.add_card_to_hand(0, catalog::mind_rake());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: rake,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("overload");
    drain_stack(&mut g);
    assert_eq!([g.players[0].hand.len(), g.players[1].hand.len()], [1, 1]);
}

/// Arterial Flow drains only with a Vampire.
#[test]
fn arterial_flow_drains_with_a_vampire() {
    for (vampire, loss) in [(true, 2), (false, 0)] {
        let mut g = pod(2);
        if vampire {
            ready(&mut g, 0, catalog::mirri_the_cursed());
        }
        let af = g.add_card_to_hand(0, catalog::arterial_flow());
        flood(&mut g);
        let life = g.players[1].life;
        cast(&mut g, af);
        assert_eq!(g.players[1].life, life - loss);
    }
}

// ── Indominus Rex, Alpha (seat 344) ──────────────────────────────────────────

/// Indominus Rex (CR 614.12, 122.1b): discarding Serra Angel (flying,
/// vigilance) and Grizzly Bears as it enters gives it two keyword counters,
/// and it draws two.
#[test]
fn indominus_rex_eats_keywords() {
    let mut g = pod(2);
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let rex = g.add_card_to_hand(0, catalog::indominus_rex_alpha());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![angel, bears])]));
    cast(&mut g, rex);
    let kw = g.computed_permanent(rex).unwrap().keywords().to_vec();
    assert!(kw.contains(&crabomination::card::Keyword::Flying) && kw.contains(&crabomination::card::Keyword::Vigilance));
    assert!(!kw.contains(&crabomination::card::Keyword::Trample));
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Mirri the Cursed grows from combat damage dealt to a creature.
#[test]
fn mirri_grows_on_creature_damage() {
    let mut g = pod(2);
    let mirri = ready(&mut g, 0, catalog::mirri_the_cursed());
    let bear = ready(&mut g, 1, catalog::giant_spider());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: mirri, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, mirri)])).expect("block");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(mirri).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// Morbius from the graveyard: exile it, look at three, keep one.
#[test]
fn morbius_digs_from_the_graveyard() {
    let mut g = pod(2);
    let morb = g.add_card_to_graveyard(0, catalog::morbius_the_living_vampire());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    flood(&mut g);
    activate(&mut g, morb, 0);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.exile.iter().any(|c| c.id == morb));
    assert_eq!(g.players[0].library.len(), 3);
}

/// Shadow of the Grave returns only the cards discarded this turn.
#[test]
fn shadow_of_the_grave_returns_this_turns_discards() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::island());
    let kept = g.add_card_to_hand(0, catalog::forest());
    // Seat 0 discards the Forest to its own Mind Rake.
    let rake = g.add_card_to_hand(0, catalog::mind_rake());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: rake,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("rake yourself");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == kept));
    let sog = g.add_card_to_hand(0, catalog::shadow_of_the_grave());
    flood(&mut g);
    cast(&mut g, sog);
    let hand: Vec<&str> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(hand, ["Forest"], "the old Island stays");
}

/// Luxior: equip planeswalker {1} makes it a creature, not a planeswalker,
/// with +1/+1 per loyalty counter.
#[test]
fn luxior_turns_a_planeswalker_into_a_creature() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_of_the_veil());
    let lux = ready(&mut g, 0, catalog::luxior_giadas_gift());
    let loyalty = g.battlefield_find(lili).unwrap().counter_count(crabomination::card::CounterType::Loyalty) as i32;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: lux, target: lili }).expect("equip planeswalker {1}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(lili).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature));
    assert!(!cp.card_types().contains(&crabomination::card::CardType::Planeswalker));
    assert_eq!(cp.power, loyalty);
}

/// Liliana of the Veil −6: you split the target player's permanents into two
/// piles; they sacrifice the pile they choose, and keep the other.
#[test]
fn liliana_of_the_veil_ultimate_splits_into_piles() {
    let mut g = pod(2);
    let lili = ready(&mut g, 0, catalog::liliana_of_the_veil());
    g.battlefield_find_mut(lili).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 3);
    for _ in 0..4 {
        ready(&mut g, 1, catalog::grizzly_bears());
    }
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: lili,
        ability_index: 2,
        target: Some(Target::Player(1)),
        x_value: None,
    })
    .expect("-6");
    drain_stack(&mut g);
    let left = g.battlefield.iter().filter(|c| c.controller == 1).count();
    assert!(left > 0 && left < 4, "one pile sacrificed, the other kept: {left} left");
}

/// Tinybones Joins Up: "any number of target players" — both opponents
/// discard.
#[test]
fn tinybones_joins_up_hits_any_number_of_players() {
    let mut g = pod(3);
    for seat in 1..3 {
        g.add_card_to_hand(seat, catalog::island());
    }
    let tju = g.add_card_to_hand(0, catalog::tinybones_joins_up());
    flood(&mut g);
    cast(&mut g, tju);
    assert_eq!([g.players[1].hand.len(), g.players[2].hand.len()], [0, 0]);
}

// ── Tergrid (345) and Athreos, God of Passage (346) ─────────────────────────

/// Liliana, Waker of the Dead +1: everyone discards; an opponent who can't
/// loses 3, and you don't lose for your own empty hand.
#[test]
fn liliana_waker_plus_one_taxes_empty_handed_opponents() {
    let mut g = pod(3);
    let lili = ready(&mut g, 0, catalog::liliana_waker_of_the_dead());
    g.battlefield_find_mut(lili).unwrap().add_counters(crabomination::card::CounterType::Loyalty, 4);
    g.add_card_to_hand(1, catalog::island());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: lili, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0);
    assert_eq!([g.players[0].life, g.players[1].life, g.players[2].life], [life[0], life[1], life[2] - 3]);
}

/// Pox: each step takes a third, rounded up — 20 life loses 7, three cards
/// lose one, two creatures lose one.
#[test]
fn pox_takes_a_third_rounded_up() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    let pox = g.add_card_to_hand(0, catalog::pox());
    flood(&mut g);
    let life = g.players[1].life;
    cast(&mut g, pox);
    assert_eq!(g.players[1].life, life - (life + 2) / 3);
    assert_eq!(g.players[1].hand.len(), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 1);
}

/// Shadowborn Apostle: six of them (CR 903.5b — a deck may hold any number)
/// sacrifice to fetch a Demon.
#[test]
fn shadowborn_apostles_summon_a_demon() {
    let mut g = pod(2);
    let first = ready(&mut g, 0, catalog::shadowborn_apostle());
    for _ in 0..5 {
        ready(&mut g, 0, catalog::shadowborn_apostle());
    }
    g.add_card_to_library(0, catalog::taborax_hopes_demise());
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, first, 0);
    assert_eq!(named(&g, "Shadowborn Apostle"), 0);
    assert_eq!(named(&g, "Taborax, Hope's Demise"), 1);
}

/// Taborax grows on a nontoken death, and a dying Cleric offers a card for 1
/// life.
#[test]
fn taborax_grows_and_draws_off_clerics() {
    let mut g = pod(2);
    let tab = ready(&mut g, 0, catalog::taborax_hopes_demise());
    let cleric = ready(&mut g, 0, catalog::shadowborn_apostle());
    g.add_card_to_library(0, catalog::island());
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: murder,
        target: Some(Target::Permanent(cleric)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("murder");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tab).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), 1);
    assert_eq!(g.players[0].life, life - 1);
}

/// Secret Salvage exiles a card from your graveyard and tutors every copy of
/// its name.
#[test]
fn secret_salvage_finds_every_copy() {
    let mut g = pod(2);
    let gy = g.add_card_to_graveyard(0, catalog::shadowborn_apostle());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::shadowborn_apostle());
    }
    g.add_card_to_library(0, catalog::island());
    let ss = g.add_card_to_hand(0, catalog::secret_salvage());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: ss,
        target: Some(Target::Permanent(gy)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("salvage");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gy));
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Shadowborn Apostle").count(), 3);
}

/// Seat 1 bolts seat 0.
fn bolt_player_zero(g: &mut GameState) {
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(g);
}

/// Saving Grace (Anti-Venom's list): CR 614.9 — the turn's damage to you goes
/// to the enchanted creature, which the Aura's +0/+3 keeps alive.
#[test]
fn saving_grace_takes_the_turns_damage_to_you() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let grace = g.add_card_to_hand(0, catalog::saving_grace());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: grace,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Saving Grace");
    drain_stack(&mut g);
    let life = g.players[0].life;
    bolt_player_zero(&mut g);
    assert_eq!(g.players[0].life, life, "the Bolt went to the bear");
    assert_eq!(g.battlefield_find(bear).expect("2/5 survives").damage, 3);
}

/// Martyrdom (Anti-Venom's list): the granted {0} ability moves the next 1
/// damage to you onto the creature (CR 614.9), once per activation.
#[test]
fn martyrdom_grants_a_one_damage_redirect() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let martyr = g.add_card_to_hand(0, catalog::martyrdom());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: martyr,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Martyrdom");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the granted {0} ability");
    drain_stack(&mut g);
    let life = g.players[0].life;
    bolt_player_zero(&mut g);
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.battlefield_find(bear).expect("1 damage on a 2/2").damage, 1);
}
