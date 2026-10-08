//! Commander: most-built commanders seated from their EDHREC average decks
//! (`decks::cmdr_most_built`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn activate(g: &mut GameState, id: CardId, index: usize) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// CR 603.2c — Ob Nixilis: three opponents each losing 1 life at once (Mount
/// Doom's ping) is ONE batch, so one trigger: one counter, one card exiled.
#[test]
fn ob_nixilis_triggers_once_for_a_batch_of_single_losses() {
    let mut g = pod(4);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let doom = ready(&mut g, 0, catalog::mount_doom());
    let library = g.players[0].library.len();
    activate(&mut g, doom, 1);
    assert_eq!([g.players[1].life, g.players[2].life, g.players[3].life], [19, 19, 19]);
    assert_eq!(plus_ones(&g, ob), 1);
    assert_eq!(g.players[0].library.len(), library - 1, "one impulse-exiled card");
}

/// Ruling 2023-05-12 — combat damage to a player is read in total: two 1/1s
/// connecting with one opponent is a loss of 2, so no trigger; one 1/1 on
/// each of two opponents is a loss of exactly 1 each, one trigger.
#[test]
fn ob_nixilis_reads_a_players_total_combat_loss() {
    let mut g = pod(3);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let a = ready(&mut g, 0, catalog::memnite());
    let b = ready(&mut g, 0, catalog::memnite());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(plus_ones(&g, ob), 0, "a loss of 2 is not exactly 1");

    let mut g = pod(3);
    let ob = ready(&mut g, 0, catalog::ob_nixilis_captive_kingpin());
    let a = ready(&mut g, 0, catalog::memnite());
    let b = ready(&mut g, 0, catalog::memnite());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(2) },
    ]))
    .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
    assert_eq!(plus_ones(&g, ob), 1, "both opponents lost exactly 1 in one batch");
}

/// Mount Doom's last ability: the chosen two survive (Ob Nixilis and the
/// costliest by the headless pick), every other creature is destroyed, and
/// the land and a legendary artifact are the cost.
#[test]
fn mount_doom_keeps_up_to_two_creatures_and_destroys_the_rest() {
    let mut g = pod(3);
    let doom = ready(&mut g, 0, catalog::mount_doom());
    let mox = ready(&mut g, 0, catalog::mox_amber());
    for seat in 0..3 {
        ready(&mut g, seat, catalog::grizzly_bears());
        ready(&mut g, seat, catalog::grizzly_bears());
    }
    activate(&mut g, doom, 2);
    let creatures = g.battlefield.iter().filter(|c| c.definition.is_creature()).count();
    assert_eq!(creatures, 2);
    assert!(g.battlefield_find(doom).is_none() && g.battlefield_find(mox).is_none());
}

/// Manabarbs — each land tapped for mana is its own 1 damage to the player
/// who tapped it (2009-10-01 rulings), whoever controls the enchantment.
#[test]
fn manabarbs_burns_whoever_taps_a_land() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::manabarbs());
    let m1 = ready(&mut g, 1, catalog::mountain());
    let m2 = ready(&mut g, 1, catalog::mountain());
    for land in [m1, m2] {
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("tap for mana");
    }
    g.priority.player_with_priority = 1;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 20);
}

/// Shadow of the Goblin — a land PLAYED from the graveyard (Crucible of
/// Worlds) pings each opponent; one played from hand does not.
#[test]
fn shadow_of_the_goblin_pings_for_a_land_played_from_elsewhere() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::shadow_of_the_goblin());
    ready(&mut g, 0, catalog::crucible_of_worlds());
    let from_hand = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(from_hand)).expect("play from hand");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [20, 20]);
    g.players[0].lands_played_this_turn = 0;
    let from_yard = g.add_card_to_graveyard(0, catalog::mountain());
    g.perform_action(GameAction::PlayLandFromGraveyard(from_yard)).expect("play from graveyard");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<crabomination::game::types::Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// CR 702.40 — Storm's combat damage gives the NEXT instant or sorcery this
/// turn storm: a Bolt after one earlier spell is copied once; the one after
/// it is not (the grant is spent).
#[test]
fn storm_gives_the_next_instant_or_sorcery_storm() {
    use crabomination::game::types::Target;
    let mut g = pod(3);
    let storm = ready(&mut g, 0, catalog::storm_force_of_nature());
    flood(&mut g);
    let opt = g.add_card_to_hand(0, catalog::opt());
    cast_at(&mut g, opt, None);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: storm, target: AttackTarget::Player(2) }]))
        .expect("attack");
    advance_to(&mut g, TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, 17);
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 14, "the Bolt and one storm copy");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 11, "the grant was spent");
}

/// Ashling's Magecraft: the second resolution this turn deals 2 to each
/// opponent and each creature they control; the third adds {R}{R}{R}{R}.
#[test]
fn ashling_escalates_on_the_second_and_third_spell() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::ashling_flame_dancer());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    for i in 0..3 {
        for _ in 0..2 {
            g.add_card_to_hand(0, catalog::island());
        }
        let opt = g.add_card_to_hand(0, catalog::opt());
        flood(&mut g);
        let red = g.players[0].mana_pool.amount(Color::Red);
        cast_at(&mut g, opt, None);
        match i {
            0 => assert_eq!(g.players[1].life, 20),
            1 => {
                assert_eq!([g.players[1].life, g.players[2].life], [18, 18]);
                assert!(g.battlefield_find(bear).is_none(), "their creature took 2");
                assert!(g.battlefield_find(mine).is_some(), "yours did not");
            }
            _ => assert_eq!(g.players[0].mana_pool.amount(Color::Red), red + 4),
        }
    }
}

/// Electro: an instant or sorcery cast adds {R}; leaving the battlefield it
/// may pay {X} for a reflexive X damage to target player (2025-09-19 ruling).
#[test]
fn electro_adds_red_and_burns_as_it_leaves() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let electro = ready(&mut g, 0, catalog::electro_assaulting_battery());
    let opt = g.add_card_to_hand(0, catalog::opt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast_at(&mut g, opt, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "Electro's red mana");
    g.players[0].mana_pool.add_colorless(3);
    // The bots' profile: a hostile player slot names an opponent first.
    g.players[0].hostile_player_targets = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(Target::Permanent(electro)));
    assert!(g.battlefield_find(electro).is_none());
    assert_eq!(g.players[1].life, 17, "paid {{X}} = 3 for 3 damage");
}

fn shock_it(g: &mut GameState, id: CardId) {
    flood(g);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(g, shock, Some(crabomination::game::types::Target::Permanent(id)));
}

/// CR 702.54 — Indoraptor's bloodthirst X counts all damage dealt to your
/// opponents this turn (2023-11-10 ruling), summed across opponents.
#[test]
fn indoraptor_enters_with_the_turns_damage_to_opponents() {
    let mut g = pod(3);
    g.players[1].damage_taken_this_turn = 2;
    g.players[2].damage_taken_this_turn = 3;
    let raptor = g.add_card_to_hand(0, catalog::indoraptor_the_perfect_hybrid());
    flood(&mut g);
    cast_at(&mut g, raptor, None);
    assert_eq!(plus_ones(&g, raptor), 5);
}

/// Indoraptor's enrage: the random opponent with no nontoken creature to
/// sacrifice is dealt damage equal to its power.
#[test]
fn indoraptor_enrage_punishes_a_random_opponent() {
    let mut g = pod(2);
    let raptor = ready(&mut g, 0, catalog::indoraptor_the_perfect_hybrid());
    g.battlefield_find_mut(raptor).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    shock_it(&mut g, raptor);
    assert!(g.battlefield_find(raptor).is_some(), "a 6/4 survives the Shock");
    assert_eq!(g.players[1].life, 14, "6 damage to the only opponent");
}

/// Polyraptor's enrage copies it; Silverclad Ferocidons' makes each opponent
/// sacrifice a permanent.
#[test]
fn polyraptor_copies_itself_and_ferocidons_eats_a_permanent() {
    let mut g = pod(3);
    let poly = ready(&mut g, 0, catalog::polyraptor());
    shock_it(&mut g, poly);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Polyraptor").count(), 2);

    let mut g = pod(3);
    let fero = ready(&mut g, 0, catalog::silverclad_ferocidons());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::mountain());
    shock_it(&mut g, fero);
    assert!(g.battlefield.iter().all(|c| c.controller == 0), "each opponent lost its one permanent");
}

/// Forerunner of the Empire: a Dinosaur of yours entering may have it deal 1
/// damage to each creature.
#[test]
fn forerunner_of_the_empire_pings_each_creature_when_a_dinosaur_enters() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::forerunner_of_the_empire());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let dino = g.add_card_to_hand(0, catalog::polyraptor());
    flood(&mut g);
    cast_at(&mut g, dino, None);
    assert!(g.battlefield_find(elf).is_none(), "the 1/1 took 1");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Polyraptor").count(), 2, "and Polyraptor copied itself");
}

/// Shadow the Hedgehog — a creature of yours with haste dying draws a card;
/// one without flash or haste does not.
#[test]
fn shadow_draws_when_a_hasty_creature_dies() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::shadow_the_hedgehog());
    let hasty = ready(&mut g, 0, catalog::raging_goblin());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    shock_it(&mut g, bear);
    assert_eq!(g.players[0].hand.len(), hand, "no draw for the bear");
    shock_it(&mut g, hasty);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a draw for the haste creature");
}

/// CR 702.61 — Chaos Control: a spell cast with Sol Ring's mana has split
/// second, so the opponent can't respond; the same spell paid from lands
/// does not.
#[test]
fn shadow_gives_artifact_paid_spells_split_second() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::shadow_the_hedgehog());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    g.perform_action(GameAction::ActivateAbility {
        card_id: ring, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("tap Sol Ring");
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.perform_action(GameAction::CastSpell { card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast with artifact mana");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let respond = GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(respond.clone()).is_err(), "split second locks the response");
    drain_stack(&mut g);

    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast from the pool");
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(respond).is_ok(), "no artifact mana, no split second");
}

/// Knuckles — double strike is two combat-damage steps, each a Treasure.
#[test]
fn knuckles_makes_a_treasure_per_combat_damage_step() {
    let mut g = pod(2);
    let k = ready(&mut g, 0, catalog::knuckles_the_echidna());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: k, target: AttackTarget::Player(1) }]))
        .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 2);
}

/// Smaug enters with a tapped Treasure per artifact the opponents control.
#[test]
fn smaug_counts_opponents_artifacts() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::sol_ring());
    ready(&mut g, 2, catalog::mind_stone());
    ready(&mut g, 2, catalog::sol_ring());
    let smaug = g.add_card_to_hand(0, catalog::smaug_wicked_worm());
    flood(&mut g);
    cast_at(&mut g, smaug, None);
    let mine: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Treasure").collect();
    assert_eq!(mine.len(), 3);
    assert!(mine.iter().all(|c| c.tapped));
}

/// Super State — the enchanted 9/9's combat damage to one opponent is dealt
/// again to each other opponent.
#[test]
fn super_state_splashes_combat_damage() {
    use crabomination::game::types::Target;
    let mut g = pod(3);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::super_state());
    flood(&mut g);
    cast_at(&mut g, aura, Some(Target::Permanent(bear)));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    advance_to(&mut g, TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [11, 11]);
}

fn shock_player(g: &mut GameState, seat: usize) {
    flood(g);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(g, shock, Some(crabomination::game::types::Target::Player(seat)));
}

/// CR 614.1a — Ojer Axonil: a red source of yours dealing an opponent less
/// noncombat damage than its power deals its power instead; damage to a
/// creature is untouched.
#[test]
fn ojer_axonil_raises_red_noncombat_damage_to_its_power() {
    use crabomination::game::types::Target;
    let mut g = pod(3);
    ready(&mut g, 0, catalog::ojer_axonil_deepest_might());
    shock_player(&mut g, 1);
    assert_eq!(g.players[1].life, 16, "Shock's 2 became 4");
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    flood(&mut g);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast_at(&mut g, shock, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[2].life, 20);
}

/// Ojer Axonil dies into Temple of Power, tapped; the Temple transforms back
/// only once red sources you controlled dealt 4+ noncombat damage this turn.
#[test]
fn ojer_axonil_returns_as_a_temple_that_needs_four_red_damage() {
    let mut g = pod(2);
    let ojer = ready(&mut g, 0, catalog::ojer_axonil_deepest_might());
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, bolt, Some(crabomination::game::types::Target::Permanent(ojer)));
    cast_at(&mut g, bolt2, Some(crabomination::game::types::Target::Permanent(ojer)));
    let temple = g.battlefield.iter().find(|c| c.definition.name == "Temple of Power").expect("returned transformed");
    assert!(temple.tapped);
    let temple = temple.id;
    g.battlefield_find_mut(temple).unwrap().tapped = false;
    flood(&mut g);
    g.priority.player_with_priority = 0;
    let transform = GameAction::ActivateAbility {
        card_id: temple, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    };
    assert!(g.perform_action(transform.clone()).is_ok(), "6 red noncombat damage this turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(temple).unwrap().definition.name, "Ojer Axonil, Deepest Might");
}

/// Chandra's Incinerator — {X} less per noncombat damage to opponents this
/// turn, and a source of yours burning an opponent burns one of their
/// creatures for the same amount.
#[test]
fn chandras_incinerator_discounts_and_redirects_burn() {
    let mut g = pod(3);
    shock_player(&mut g, 1);
    shock_player(&mut g, 2);
    let inc = g.add_card_to_hand(0, catalog::chandras_incinerator());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Red, 2);
    cast_at(&mut g, inc, None);
    assert!(g.battlefield_find(inc).is_some(), "{{5}}{{R}} less 4 is {{1}}{{R}}");
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    shock_player(&mut g, 1);
    assert!(g.battlefield_find(bear).is_none(), "2 more to the bear");
}

/// Defiler of Instinct — a red permanent spell cast pings any target.
#[test]
fn defiler_of_instinct_pings_on_red_permanent_spells() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    ready(&mut g, 0, catalog::defiler_of_instinct());
    let goblin = g.add_card_to_hand(0, catalog::raging_goblin());
    flood(&mut g);
    cast_at(&mut g, goblin, None);
    assert_eq!(g.players[1].life, 19);
}

/// Urabrask — each instant or sorcery pings target opponent and adds {R};
/// after three, {R} flips it into The Great Work, whose chapter I deals 3 to
/// target opponent and each creature they control.
#[test]
fn urabrask_flips_into_the_great_work() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    let ura = ready(&mut g, 0, catalog::urabrask());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    for _ in 0..3 {
        let opt = g.add_card_to_hand(0, catalog::opt());
        flood(&mut g);
        cast_at(&mut g, opt, None);
    }
    assert_eq!(g.players[1].life, 17);
    activate(&mut g, ura, 0);
    assert_eq!(g.battlefield_find(ura).map(|c| c.definition.name), Some("The Great Work"));
    assert!(g.battlefield_find(bear).is_none(), "chapter I");
    assert_eq!(g.players[1].life, 14);
}

/// Burning Earth burns a nonbasic land's tapper, not a basic's.
#[test]
fn burning_earth_burns_nonbasic_taps_only() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::burning_earth());
    let tower = ready(&mut g, 1, catalog::command_tower());
    let mountain = ready(&mut g, 1, catalog::mountain());
    for land in [tower, mountain] {
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
        })
        .expect("tap");
    }
    g.priority.player_with_priority = 1;
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
}

/// Virtue of Courage — burning an opponent for 2 may exile your top two to
/// play this turn.
#[test]
fn virtue_of_courage_impulses_the_damage_dealt() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::virtue_of_courage());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    shock_player(&mut g, 1);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0 && c.may_play_until.is_some()).count(), 2);
}

/// Rowan — X is the life you lost this turn, fixed as the ability resolves;
/// only black and/or red spells get the discount, and only on generic mana.
#[test]
fn rowan_discounts_black_and_red_spells_by_life_lost() {
    let mut g = pod(2);
    let rowan = ready(&mut g, 0, catalog::rowan_scion_of_war());
    g.adjust_life(0, -3);
    g.players[0].life_lost_this_turn = 3;
    activate(&mut g, rowan, 0);
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let tink = g.add_card_to_hand(0, catalog::inspired_tinkering());
    cast_at(&mut g, tink, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count(), 3, "{{4}}{{R}} less 3");
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::CastSpell { card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None })
        .is_err(), "a colorless spell keeps its {{2}}");
}

/// Vilis — losing 3 life draws 3.
#[test]
fn vilis_draws_for_life_lost() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::vilis_broker_of_blood());
    let hand = g.players[0].hand.len();
    shock_player_from(&mut g, 1, 0);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

fn shock_player_from(g: &mut GameState, caster: usize, seat: usize) {
    let shock = g.add_card_to_hand(caster, catalog::shock());
    g.players[caster].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = caster;
    g.perform_action(GameAction::CastSpell {
        card_id: shock,
        target: Some(crabomination::game::types::Target::Player(seat)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("shock");
    drain_stack(g);
}

/// Erebos's Intervention's second mode exiles up to twice X TARGET graveyard
/// cards (the printed targets; the catalog read it as a resolution pick).
#[test]
fn erebos_s_intervention_exiles_twice_x_cards() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::erebos_s_intervention());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: Some(1),
        x_value: Some(1),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b));
}

/// Peer into the Abyss — half the library and half the life, rounded up.
#[test]
fn peer_into_the_abyss_rounds_up() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    g.players[0].life = 7;
    let peer = g.add_card_to_hand(0, catalog::peer_into_the_abyss());
    flood(&mut g);
    let hand = g.players[0].hand.len();
    cast_at(&mut g, peer, Some(Target::Player(0)));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 5, "10 cards: draw 5");
    assert_eq!(g.players[0].life, 3, "7 life: lose 4");
}


/// Bug fix (pod seed 31701 game 90): Blood Celebrant's mana ability costs {B}
/// and has no {T}, so the auto-tapper paid its {B} with the Celebrant again,
/// forever, until the thread's stack overflowed. A source mid-activation is
/// never re-entered: with no other black source the cast is simply refused,
/// and with one it pays.
#[test]
fn a_costly_mana_source_does_not_pay_for_itself() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::blood_celebrant());
    let ritual = g.add_card_to_hand(0, catalog::dark_ritual());
    g.priority.player_with_priority = 0;
    let cast = GameAction::CastSpell { card_id: ritual, target: None, additional_targets: vec![], mode: None, x_value: None };
    assert!(g.perform_action(cast.clone()).is_err());
    ready(&mut g, 0, catalog::swamp());
    assert!(g.perform_action(cast).is_ok(), "the Swamp pays for the Celebrant, which pays for the Ritual");
}

/// CR 612 / 400.7 — Deadpool exchanges text boxes with an opponent's Llanowar
/// Elves: Deadpool taps for {G}, the Elves carry the upkeep life loss and the
/// sacrifice ability; Deadpool leaving the battlefield reverts only Deadpool.
#[test]
fn deadpool_exchanges_text_boxes() {
    let mut g = pod(2);
    let elves = ready(&mut g, 1, catalog::llanowar_elves());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![elves])]));
    let dp = g.add_card_to_hand(0, catalog::deadpool_trading_card());
    flood(&mut g);
    cast_at(&mut g, dp, None);
    let elves_def = &g.battlefield_find(elves).unwrap().definition;
    assert_eq!(elves_def.name, "Llanowar Elves");
    assert!(elves_def.activated_abilities.iter().any(|a| a.sac_cost), "the Elves have Deadpool's sacrifice");
    assert_eq!(elves_def.triggered_abilities.len(), 1, "and its upkeep trigger");
    let dp_def = &g.battlefield_find(dp).unwrap().definition;
    assert!(dp_def.triggered_abilities.is_empty() && !dp_def.activated_abilities.iter().any(|a| a.sac_cost));
    assert_eq!((dp_def.power, dp_def.toughness), (5, 3), "P/T stays");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    cast_at(&mut g, bolt, Some(crabomination::game::types::Target::Permanent(dp)));
    let back = g.players[0].graveyard.iter().chain(g.players[0].command.iter()).find(|c| c.id == dp).expect("left");
    assert!(back.definition.activated_abilities.iter().any(|a| a.sac_cost), "Deadpool's card reverts");
    assert!(g.battlefield_find(elves).unwrap().definition.activated_abilities.iter().any(|a| a.sac_cost));
}

/// Elturel Survivors gets +X/+0 while attacking, X the defending player's
/// lands; myriad copies it at each other opponent.
#[test]
fn elturel_survivors_counts_the_defenders_lands() {
    let mut g = pod(3);
    let surv = ready(&mut g, 0, catalog::elturel_survivors());
    for _ in 0..3 {
        ready(&mut g, 1, catalog::mountain());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert_eq!(g.computed_permanent(surv).unwrap().power, 0);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: surv, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(surv).unwrap().power, 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elturel Survivors").count(), 2, "myriad");
}

fn unblocked_attack(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
}

/// CR 702.49 — Satoru grants ninjutsu {2}{U}{B} to a creature card in hand,
/// and activating it (an activated ability, CR 702.49a) takes one of the top
/// three into hand.
#[test]
fn satoru_grants_ninjutsu_and_digs_on_activation() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::satoru_umezawa());
    let sneaker = ready(&mut g, 0, catalog::memnite());
    let dragon = g.add_card_to_hand(0, catalog::ancient_silver_dragon());
    flood(&mut g);
    unblocked_attack(&mut g, sneaker);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::Ninjutsu { ninja: dragon, returning: sneaker }).expect("granted ninjutsu");
    drain_stack(&mut g);
    assert!(g.attacking.iter().any(|a| a.attacker == dragon));
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1 + 1, "the dragon left, the Memnite came back, one card dug");
}

/// Thousand-Faced Shadow ninjutsu'd in copies another attacking creature,
/// tapped and attacking.
#[test]
fn thousand_faced_shadow_copies_an_attacker() {
    let mut g = pod(2);
    let sneaker = ready(&mut g, 0, catalog::memnite());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let shadow = g.add_card_to_hand(0, catalog::thousand_faced_shadow());
    flood(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: sneaker, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Ninjutsu { ninja: shadow, returning: sneaker }).expect("ninjutsu");
    drain_stack(&mut g);
    let copies: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Grizzly Bears").map(|c| c.id).collect();
    assert_eq!(copies.len(), 1);
    assert!(g.attacking.iter().any(|a| a.attacker == copies[0]) && g.battlefield_find(copies[0]).unwrap().tapped);
}

/// Cunning Evasion returns a blocked creature to its owner's hand.
#[test]
fn cunning_evasion_bounces_a_blocked_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::cunning_evasion());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bear)])).expect("block");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Kaito Shizuki — −2 makes an unblockable 1/1 Ninja; +1 draws and, with no
/// attack this turn, discards.
#[test]
fn kaito_shizuki_ninja_and_loot() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let kaito = ready(&mut g, 0, catalog::kaito_shizuki());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: kaito, ability_index: 1, target: None, x_value: None })
        .expect("-2");
    drain_stack(&mut g);
    let ninja = g.battlefield.iter().find(|c| c.is_token).expect("ninja");
    assert!(ninja.definition.keywords.contains(&Keyword::Unblockable));
    g.battlefield_find_mut(kaito).unwrap().loyalty_uses_this_turn = 0;
    g.add_card_to_hand(0, catalog::mountain());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: kaito, ability_index: 0, target: None, x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew one, discarded one");
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

/// Volo, Guide to Monsters copies a creature spell of a new type (CR 707.10:
/// the copy resolves as a token) — not one sharing a type with a creature on
/// the battlefield or a creature card in the graveyard.
#[test]
fn volo_copies_only_creature_spells_of_an_unseen_type() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::volo_guide_to_monsters());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::raging_goblin());
    flood(&mut g);
    for (def, name, want) in [
        (catalog::llanowar_elves(), "Llanowar Elves", 2),
        (catalog::goblin_piker(), "Goblin Piker", 1),
        (catalog::grizzly_bears(), "Grizzly Bears", 2),
    ] {
        let id = g.add_card_to_hand(0, def);
        cast_at(&mut g, id, None);
        assert_eq!(named(&g, name), want, "{name}");
    }
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "only the Elves were copied");
}

/// Volo's Journal notes one new creature type per creature spell; Volo's
/// {2},{T} draws a card for each noted type.
#[test]
fn volos_journal_notes_types_and_volo_draws_for_them() {
    use crabomination::game::types::Target;
    let mut g = pod(2);
    let volo = g.add_card_to_hand(0, catalog::volo_itinerant_scholar());
    flood(&mut g);
    cast_at(&mut g, volo, None);
    g.clear_sickness(volo);
    let journal = g.battlefield.iter().find(|c| c.definition.name == "Volo's Journal").expect("journal").id;
    // Elf Druid twice notes both types; a Bear adds a third, a second Bear none.
    for def in [catalog::llanowar_elves(), catalog::llanowar_elves(), catalog::grizzly_bears(), catalog::grizzly_bears()] {
        let id = g.add_card_to_hand(0, def);
        cast_at(&mut g, id, None);
    }
    assert_eq!(g.battlefield_find(journal).unwrap().noted_creature_types.len(), 3);
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: volo,
        ability_index: 0,
        target: Some(Target::Permanent(journal)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Radagast digs the entering creature's mana value deep for a creature card
/// sharing no type with a creature you control: past the Bear, to the Elves.
#[test]
fn radagast_digs_for_a_creature_of_a_new_type() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::llanowar_elves());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    ready(&mut g, 0, catalog::radagast_the_brown());
    flood(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, None);
    let names: Vec<_> = g.players[0].hand.iter().map(|c| c.definition.name).collect();
    assert_eq!(names, ["Llanowar Elves"]);
    assert_eq!(g.players[0].library.len(), 6, "the Bear went to the bottom");
}

/// Silverback Elder triggers on each creature spell and resolves one mode.
#[test]
fn silverback_elder_triggers_on_a_creature_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::silverback_elder());
    let ring = ready(&mut g, 1, catalog::sol_ring());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    flood(&mut g);
    let lands = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_at(&mut g, bear, None);
    let fired = g.battlefield_find(ring).is_none()
        || g.players[0].life == 24
        || g.battlefield.iter().filter(|c| c.definition.is_land()).count() == lands + 1;
    assert!(fired);
}

/// Dutiful Replicator — pay {1} as it enters to copy a token you control.
#[test]
fn dutiful_replicator_copies_a_token_for_one() {
    let mut g = pod(2);
    let treasure = crabomination_base::tokens::treasure_token();
    g.add_token_to_battlefield(0, &treasure);
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let rep = g.add_card_to_hand(0, catalog::dutiful_replicator());
    cast_at(&mut g, rep, None);
    assert_eq!(named(&g, "Treasure"), 2);
}

/// The bot spends Volo, Itinerant Scholar's draw on its Journal: the
/// draw-sink generator activated untargeted draws only, so a census never saw
/// it fire.
#[test]
fn bot_activates_volos_targeted_draw() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = pod(2);
    let volo = g.add_card_to_hand(0, catalog::volo_itinerant_scholar());
    flood(&mut g);
    cast_at(&mut g, volo, None);
    g.clear_sickness(volo);
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast_at(&mut g, elf, None);
    g.players[0].mana_pool = Default::default();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let a = HeuristicBot::new().next_action(&g, 0);
    assert!(matches!(a, Some(GameAction::ActivateAbility { card_id, target: Some(_), .. }) if card_id == volo), "{a:?}");
}
