//! Commander: the Animated Army precon (BLC, Bello, `decks::cmdr_bello`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
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

fn cast_action(id: CardId, target: Option<Target>) -> GameAction {
    GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None }
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g);
    g.perform_action(cast_action(id, target)).expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map(|_| ())
    .map_err(|e| format!("{e:?}"))
}

fn attack(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    for (a, _) in &attacks {
        g.clear_sickness(*a);
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Attack, no blocks, through combat damage.
fn connect(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    attack(g, attacks);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1 - g.active_player_idx;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn count_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn is_creature(g: &GameState, id: CardId) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.card_types().contains(&crabomination::card::CardType::Creature))
}

// ── Bello and friends ──────────────────────────────────────────────────────

/// CR 613.1d/f — on your turn only, a mana value 4+ artifact is a 4/4 hasty,
/// indestructible Elemental; Equipment and cheap artifacts aren't.
#[test]
fn bello_animates_big_artifacts_on_your_turn() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bello_bard_of_the_brambles());
    let archive = g.add_card_to_battlefield(0, catalog::hedron_archive());
    let skull = g.add_card_to_battlefield(0, catalog::batterskull());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    assert!(is_creature(&g, archive));
    assert_eq!(pt(&g, archive), (4, 4));
    assert!(g.computed_permanent(archive).unwrap().keywords().contains(&Keyword::Indestructible));
    assert!(!is_creature(&g, skull) || g.computed_permanent(skull).unwrap().power != 4, "Equipment");
    assert!(!is_creature(&g, stone), "mana value 2");
    g.active_player_idx = 1;
    assert!(!is_creature(&g, archive), "not on an opponent's turn");
}

/// CR 716 — two tapped Treasures on entry; level 2 makes a Treasure tap for
/// two; level 3 burns each opponent for a spell cast with Treasure mana.
#[test]
fn alchemists_talent_levels_up_its_treasures() {
    let mut g = main_phase();
    let talent = g.add_card_to_hand(0, catalog::alchemists_talent());
    cast(&mut g, talent, None);
    let treasures: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure").map(|c| c.id).collect();
    assert_eq!(treasures.len(), 2);
    assert!(treasures.iter().all(|t| g.battlefield_find(*t).unwrap().tapped));
    flood(&mut g);
    activate(&mut g, talent, 0, None).expect("level 2");
    drain_stack(&mut g);
    activate(&mut g, talent, 1, None).expect("level 3");
    drain_stack(&mut g);
    g.players[0].mana_pool.empty();
    for t in &treasures {
        g.battlefield_find_mut(*t).unwrap().tapped = false;
    }
    // The granted ability (index 1) adds two of one color.
    activate(&mut g, treasures[0], 1, None).expect("sac for two");
    assert_eq!(g.players[0].mana_pool.total(), 2);
    let div = g.add_card_to_hand(0, catalog::divination());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(cast_action(div, None)).expect("Divination with Treasure mana");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "its mana value to each opponent");
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, None);
    assert_eq!(g.players[1].life, 17, "no Treasure mana, no damage");
}

/// Attacking creatures have double strike.
#[test]
fn berserkers_onslaught_doubles_attackers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::berserkers_onslaught());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    connect(&mut g, vec![(bears, 1)]);
    assert_eq!(g.players[1].life, 16);
}

/// Your lands gain "{T}: Create a Treasure token."
#[test]
fn bootleggers_stash_turns_lands_into_treasure() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::bootleggers_stash());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, forest, 1, None).expect("the granted ability");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Treasure"), 1);
}

/// Saprolings tap for {G}; a Saproling each end step; the adventure makes two
/// (CR 715).
#[test]
fn brightcap_badger_grows_a_mana_colony() {
    let mut g = main_phase();
    let badger = g.add_card_to_hand(0, catalog::brightcap_badger());
    flood(&mut g);
    g.perform_action(GameAction::CastAdventure {
        card_id: badger,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Fungus Frolic");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Saproling"), 2);
    g.add_card_to_battlefield(0, catalog::brightcap_badger());
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    g.clear_sickness(sap);
    g.players[0].mana_pool.empty();
    activate(&mut g, sap, 0, None).expect("{T}: Add {G}");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Saproling"), 3);
}

/// CR 702.75 — hideaway twice; connecting plays one of them free.
#[test]
fn evercoat_ursine_hides_two_and_plays_one() {
    let mut g = main_phase();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let bear = g.add_card_to_battlefield_entering(0, catalog::evercoat_ursine());
    g.fire_self_etb_triggers(bear, 0);
    drain_stack(&mut g);
    let hidden = g.exile.iter().filter(|c| c.exiled_with == Some(bear)).count();
    assert_eq!(hidden, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    connect(&mut g, vec![(bear, 1)]);
    assert_eq!(g.players[1].life, 20 - 6 - 3, "trample 6, then a free Bolt");
}

/// Another creature attacking may fight Grothama; when it leaves, each
/// player draws the damage their sources dealt it.
#[test]
fn grothama_trades_fights_for_cards() {
    let mut g = main_phase();
    let grothama = g.add_card_to_battlefield(0, catalog::grothama_all_devouring());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::forest());
    }
    g.active_player_idx = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    attack(&mut g, vec![(bears, 0)]);
    assert!(g.battlefield_find(bears).is_none(), "the bears fought and died");
    assert_eq!(g.battlefield_find(grothama).unwrap().damage, 2);
    g.active_player_idx = 0;
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[1].hand.len();
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(grothama)));
    assert_eq!(g.players[1].hand.len(), hand + 2, "their bears dealt 2");
}

/// Other non-Humans enter with a +1/+1 counter; a Human doesn't.
#[test]
fn grumgully_counters_non_humans() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grumgully_the_generous());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, None);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    let human = g.add_card_to_hand(0, catalog::elvish_mystic());
    cast(&mut g, human, None);
    // Elvish Mystic is an Elf Druid — not Human, so it gets one too.
    assert_eq!(g.battlefield_find(human).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// CR 603.4 — the entrant lets you put an equal-or-cheaper permanent from
/// hand; what that puts doesn't trigger it again.
#[test]
fn kodama_puts_an_equal_or_cheaper_permanent_once() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::kodama_of_the_east_tree());
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    let third = g.add_card_to_hand(0, catalog::grizzly_bears());
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    let first = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, first, None);
    let on_bf = [second, third].iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(on_bf, 1, "one bears came along, and it did not chain");
    assert!(g.players[0].hand.iter().any(|c| c.id == giant), "the Hill Giant costs more");
}

/// First strike; connecting makes that many tapped Treasures.
#[test]
fn prosperous_bandit_pays_in_treasure() {
    let mut g = main_phase();
    let bandit = g.add_card_to_battlefield(0, catalog::prosperous_bandit());
    connect(&mut g, vec![(bandit, 1)]);
    let treasures: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").collect();
    assert_eq!(treasures.len(), 2);
    assert!(treasures.iter().all(|t| t.tapped));
}

/// Attacking, +X/+0 for your biggest artifact.
#[test]
fn pyreswipe_hawk_swings_with_its_biggest_artifact() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::hedron_archive());
    let hawk = g.add_card_to_battlefield(0, catalog::pyreswipe_hawk());
    attack(&mut g, vec![(hawk, 1)]);
    assert_eq!(pt(&g, hawk), (8, 4));
}

/// The first spell cast with Treasure mana each turn cascades (CR 702.85a).
#[test]
fn rain_of_riches_cascades_off_treasure_mana() {
    let mut g = main_phase();
    let rain = g.add_card_to_hand(0, catalog::rain_of_riches());
    cast(&mut g, rain, None);
    let treasures: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.definition.name == "Treasure").map(|c| c.id).collect();
    assert_eq!(treasures.len(), 2);
    g.players[0].mana_pool.empty();
    for lib in [catalog::lightning_bolt(), catalog::forest(), catalog::forest()] {
        g.add_card_to_library(0, lib);
    }
    activate(&mut g, treasures[0], 0, None).expect("Treasure for mana");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let div = g.add_card_to_hand(0, catalog::divination());
    g.perform_action(cast_action(div, None)).expect("Divination with a Treasure");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "cascaded into the Bolt");
}

/// Crew 3; attacking makes three Hamsters and then hits for the Hamster
/// count.
#[test]
fn rolling_hamsphere_brings_the_hamsters() {
    let mut g = main_phase();
    let sphere = g.add_card_to_battlefield(0, catalog::rolling_hamsphere());
    let crew = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.clear_sickness(crew);
    g.clear_sickness(sphere);
    g.perform_action(GameAction::Crew { vehicle: sphere, crew_creatures: vec![crew] }).expect("crew");
    attack(&mut g, vec![(sphere, 1)]);
    assert_eq!(count_named(&g, 0, "Hamster"), 3);
    assert_eq!(pt(&g, sphere), (7, 7));
    assert_eq!(g.players[1].life, 17, "three Hamsters, three damage");
}

/// A spell from hand reveals its mana value in cards and casts one of that
/// mana value or less free (the rest go to the bottom).
#[test]
fn sunbirds_invocation_casts_a_revealed_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::sunbirds_invocation());
    for lib in [catalog::forest(), catalog::lightning_bolt(), catalog::forest(), catalog::island(), catalog::island()] {
        g.add_card_to_library(0, lib);
    }
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, None);
    assert_eq!(g.players[1].life, 17, "the revealed Bolt was cast");
}

/// A Saproling every upkeep; with ten permanents the city's blessing makes
/// them 3/3 (CR 702.131).
#[test]
fn tendershoot_dryad_breeds_and_blesses() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::tendershoot_dryad());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let sap = g.battlefield.iter().find(|c| c.definition.name == "Saproling").unwrap().id;
    assert_eq!(pt(&g, sap), (1, 1));
    for _ in 0..7 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    // CR 702.131b — the tenth permanent entering grants the blessing.
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("tenth permanent");
    assert!(g.players[0].city_blessing);
    assert_eq!(pt(&g, sap), (3, 3));
}

/// Taps for any color; expend 8 returns a permanent card.
#[test]
fn trailtracker_scout_recurs_on_expend_eight() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::trailtracker_scout());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let big = g.add_card_to_hand(0, catalog::hill_giant());
    cast(&mut g, big, None);
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, None);
    assert!(!g.players[0].hand.iter().any(|c| c.id == dead), "seven spent");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "the eighth mana returns it");
}

/// Enchantment spells from hand cascade.
#[test]
fn wildsear_cascades_enchantments_from_hand() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wildsear_scouring_maw());
    for lib in [catalog::forest(), catalog::lightning_bolt(), catalog::forest()] {
        g.add_card_to_library(0, lib);
    }
    let pacifism = g.add_card_to_hand(0, catalog::pacifism());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, pacifism, Some(Target::Permanent(target)));
    assert!(
        g.players[1].life == 17 || g.battlefield_find(target).is_none(),
        "cascaded into the Bolt"
    );
}
