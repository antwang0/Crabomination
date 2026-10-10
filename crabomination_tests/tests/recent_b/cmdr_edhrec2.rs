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
