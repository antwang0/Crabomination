//! Commander: one-card gaps of the 2026-10-14 ranks 1-5500 rescan
//! (`decks::cmdr_rescan_a`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

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

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, more: Vec<Target>, mode: Option<usize>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: more,
        mode,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn declare(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks
            .into_iter()
            .map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) })
            .collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

/// Invigorated Rampage's second mode splits +2/+0 and trample over exactly
/// two creatures.
#[test]
fn invigorated_rampage_second_mode_pumps_two() {
    let mut g = pod(2);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::invigorated_rampage());
    cast_with(&mut g, spell, Some(Target::Permanent(a)), vec![Target::Permanent(b)], Some(1));
    for id in [a, b] {
        assert_eq!(pt(&g, id), (4, 2));
        assert!(g.permanent_has_keyword(id, &Keyword::Trample));
    }
}

/// Swarm of Bloodflies enters with two counters and grows when another
/// creature — anyone's — dies.
#[test]
fn swarm_of_bloodflies_grows_on_any_death() {
    let mut g = pod(2);
    let swarm = g.add_card_to_hand(0, catalog::swarm_of_bloodflies());
    cast_with(&mut g, swarm, None, vec![], None);
    assert_eq!(plus(&g, swarm), 2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(theirs)), vec![], None);
    assert_eq!(plus(&g, swarm), 3);
}

/// Bloodcrazed Paladin counts the creatures that died this turn as it enters.
#[test]
fn bloodcrazed_paladin_counts_this_turns_deaths() {
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(theirs)), vec![], None);
    let paladin = g.add_card_to_hand(0, catalog::bloodcrazed_paladin());
    cast_with(&mut g, paladin, None, vec![], None);
    assert_eq!(plus(&g, paladin), 1);
}

/// Homicidal Seclusion pumps a lone creature and switches off at two.
#[test]
fn homicidal_seclusion_needs_exactly_one_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::homicidal_seclusion());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, a), (5, 3));
    assert!(g.permanent_has_keyword(a, &Keyword::Lifelink));
    ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, a), (2, 2));
}

/// The Scarlet Witch's discount is her power, only on instants and sorceries
/// of mana value 4 or greater: Concentrate ({2}{U}{U}) casts off {U}{U}.
#[test]
fn scarlet_witch_discounts_big_instants_by_her_power() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_scarlet_witch());
    let spell = g.add_card_to_hand(0, catalog::concentrate());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{U}{U} pays for a discounted Concentrate");
}

/// Gnoll Hunter's pack tactics wants six total attacking power.
#[test]
fn gnoll_hunter_pack_tactics() {
    let mut g = pod(2);
    let gnoll = ready(&mut g, 0, catalog::gnoll_hunter());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    declare(&mut g, vec![(gnoll, 1), (angel, 1)]);
    assert_eq!(plus(&g, gnoll), 1, "2 + 4 = 6 power attacked");
}

/// Dragonscale General's bolster counts tapped creatures at your end step.
#[test]
fn dragonscale_general_bolsters_by_tapped_creatures() {
    let mut g = pod(2);
    let general = ready(&mut g, 0, catalog::dragonscale_general());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    g.battlefield_find_mut(general).unwrap().tapped = true;
    g.battlefield_find_mut(angel).unwrap().tapped = true;
    g.step = TurnStep::PostCombatMain;
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::End);
    assert_eq!(plus(&g, elf), 2, "two tapped creatures, onto the least toughness");
}

/// Mutant Chain Reaction kills a flyer and leaves a Mutagen behind.
#[test]
fn mutant_chain_reaction_destroys_and_makes_mutagen() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::mutant_chain_reaction());
    cast_with(&mut g, spell, Some(Target::Permanent(angel)), vec![], None);
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Mutagen"));
}

/// CR 903 pods: Gargantos's combat damage to one opponent is dealt to each
/// other opponent too.
#[test]
fn gargantos_splashes_combat_damage_to_other_opponents() {
    let mut g = pod(4);
    let garg = ready(&mut g, 0, catalog::gargantos_the_endbringer());
    g.battlefield_find_mut(garg).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    declare(&mut g, vec![(garg, 2)]);
    finish_combat(&mut g);
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    assert_eq!(life, vec![20, 17, 17, 17]);
}

/// Davvol drains you 1 and adds {B} per creature entering under your control.
#[test]
fn davvol_adds_black_mana_per_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::davvol_evincar_of_rath());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast_with(&mut g, bear, None, vec![], None);
    assert_eq!(g.players[0].life, 19);
}

/// Kinbinding's Kithkin enters at combat and pumps the team by one.
#[test]
fn kinbinding_mints_at_combat_and_counts_entries() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::kinbinding());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let evs = g.advance_step(Vec::new()).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::BeginCombat);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Kithkin"));
    // Only the Kithkin entered this turn: the setup bear skipped the tally.
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Gimli's Axe grants menace only to a legendary bearer.
#[test]
fn gimlis_axe_menace_needs_a_legend() {
    let mut g = pod(2);
    let axe = ready(&mut g, 0, catalog::gimlis_axe());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(bear);
    assert_eq!(pt(&g, bear), (5, 2));
    assert!(!g.permanent_has_keyword(bear, &Keyword::Menace));
    let witch = ready(&mut g, 0, catalog::the_scarlet_witch());
    g.battlefield_find_mut(axe).unwrap().attached_to = Some(witch);
    assert!(g.permanent_has_keyword(witch, &Keyword::Menace));
}

/// Weather Maker banks a charge counter on landfall.
#[test]
fn weather_maker_charges_on_landfall() {
    let mut g = pod(2);
    let wm = ready(&mut g, 0, catalog::weather_maker());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wm).unwrap().counter_count(CounterType::Charge), 1);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    while g.active_player_idx != seat || g.step != step {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn is_type(g: &GameState, id: CardId, t: crabomination::card::CardType) -> bool {
    g.computed_permanent(id).unwrap().card_types().contains(&t)
}

/// CR 205.1a / 611.2b — Hydro-Man's end-step trigger untaps him and makes him
/// a land that isn't a creature until his controller's next turn, when he is
/// a creature again.
#[test]
fn hydro_man_is_a_land_until_your_next_turn() {
    use crabomination::card::CardType;
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let hydro = ready(&mut g, 0, catalog::hydro_man_fluid_felon());
    g.battlefield_find_mut(hydro).unwrap().tapped = true;
    step_to(&mut g, 0, TurnStep::End);
    assert!(!g.battlefield_find(hydro).unwrap().tapped, "untapped");
    assert!(is_type(&g, hydro, CardType::Land));
    assert!(!is_type(&g, hydro, CardType::Creature));
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    assert!(is_type(&g, hydro, CardType::Land), "still a land on the opponent's turn");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: hydro,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for {U}");
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    assert!(is_type(&g, hydro, CardType::Creature));
    assert!(!is_type(&g, hydro, CardType::Land));
}

/// Hydro-Man's mana ability is dead while he is a creature.
#[test]
fn hydro_man_cannot_tap_for_mana_as_a_creature() {
    let mut g = pod(2);
    let hydro = ready(&mut g, 0, catalog::hydro_man_fluid_felon());
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: hydro,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(r.is_err());
}

/// CR 601.2f — Khalni Hydra's {G} reduction scales with green creatures:
/// three of them leave {G}{G}{G}{G}{G} to pay.
#[test]
fn khalni_hydra_costs_g_less_per_green_creature() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::llanowar_elves());
    }
    ready(&mut g, 0, catalog::serra_angel());
    let hydra = g.add_card_to_hand(0, catalog::khalni_hydra());
    g.players[0].mana_pool.add(Color::Green, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: hydra,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("GGGGG pays with three green creatures out");
}

/// CR 704.5j — Hall of Echoes copying your legendary creature keeps both:
/// the legend rule is off for your permanents this turn.
#[test]
fn hall_of_echoes_copies_a_legend_without_the_legend_rule() {
    let mut g = pod(2);
    let hall = ready(&mut g, 0, catalog::hall_of_echoes());
    let witch = ready(&mut g, 0, catalog::the_scarlet_witch());
    activate(&mut g, hall, 1, Some(Target::Permanent(witch)));
    assert_eq!(g.computed_permanent(hall).unwrap().def.name, "The Scarlet Witch");
    assert!(g.battlefield_find(hall).is_some() && g.battlefield_find(witch).is_some());
}

/// CR 603.7 / 400.7 — Xira's egg watch has no turn window: the egged
/// creature dying on a later turn still draws a card and makes an Insect.
#[test]
fn xira_egg_pays_off_on_a_later_turn() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let xira = ready(&mut g, 0, catalog::xira_the_golden_sting());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    declare(&mut g, vec![(xira, 1)]);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::Egg), 1);
    finish_combat(&mut g);
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    let hand = g.players[0].hand.len();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt the egg");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Insect"));
}

/// CR 614 / 104.3 — Stunning Reversal replaces the next loss this turn:
/// seven cards and life 1 instead of losing at 0 life.
#[test]
fn stunning_reversal_replaces_the_next_loss() {
    let mut g = pod(3);
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::stunning_reversal());
    cast_with(&mut g, spell, None, vec![], None);
    assert!(g.exile.iter().any(|c| c.id == spell), "exiles itself");
    g.players[0].life = 0;
    let _ = g.check_state_based_actions();
    assert!(!g.players[0].eliminated);
    assert_eq!(g.players[0].life, 1);
    assert_eq!(g.players[0].hand.len(), 7);
    // Spent: the next loss this turn is a loss.
    g.players[0].life = 0;
    let _ = g.check_state_based_actions();
    assert!(g.players[0].eliminated);
}

/// Marut's Treasure refund counts the Treasure mana spent on it.
#[test]
fn marut_refunds_treasure_mana() {
    let mut g = pod(2);
    g.priority.player_with_priority = 0;
    for _ in 0..3 {
        let t = g.add_token_to_battlefield(0, &crabomination_base::tokens::treasure_token());
        g.perform_action(GameAction::ActivateAbility {
            card_id: t,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("Treasure for mana");
    }
    let marut = g.add_card_to_hand(0, catalog::marut());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: marut,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("five from the pool, three from Treasures");
    drain_stack(&mut g);
    assert!(g.battlefield_find(marut).is_some());
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 3, "three spent, three made");
}

/// CR 702.146 — Faithbound Judge disturbed from the graveyard is Sinner's
/// Judgment on an opponent; the third judgment counter, on its controller's
/// third upkeep, makes that player lose (CR 800.4a: the pod plays on).
#[test]
fn sinners_judgment_counts_down_to_a_loss() {
    let mut g = pod(3);
    for seat in 0..3 {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let judge = g.add_card_to_graveyard(0, catalog::faithbound_judge());
    flood(&mut g);
    g.perform_action(GameAction::CastDisturb {
        card_id: judge,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
    })
    .expect("disturb");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(judge).unwrap().attached_to_player, Some(1));
    for _ in 0..2 {
        step_to(&mut g, 0, TurnStep::Draw);
        step_to(&mut g, 1, TurnStep::Upkeep);
    }
    assert!(!g.players[1].eliminated, "two counters");
    step_to(&mut g, 0, TurnStep::Draw);
    assert!(g.players[1].eliminated, "the third counter");
}

// ── decks::cmdr_rescan_b ─────────────────────────────────────────────────────

fn named_count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn loyalty(g: &mut GameState, pw: CardId, index: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: pw, ability_index: index, target, x_value: None })
        .expect("loyalty ability");
    drain_stack(g);
}

/// Meatsqueak's Food is once a turn however many creatures die.
#[test]
fn meatsqueak_makes_one_food_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::meatsqueak_hoard_lord());
    for _ in 0..2 {
        let bear = ready(&mut g, 1, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast_with(&mut g, bolt, Some(Target::Permanent(bear)), vec![], None);
    }
    assert_eq!(named_count(&g, 0, "Food"), 1);
}

/// Asari Captain pumps a lone Samurai/Warrior attacker per Samurai/Warrior.
#[test]
fn asari_captain_pumps_a_lone_attacker() {
    let mut g = pod(2);
    let captain = ready(&mut g, 0, catalog::asari_captain());
    ready(&mut g, 0, catalog::dragonscale_general());
    declare(&mut g, vec![(captain, 1)]);
    assert_eq!(pt(&g, captain).0, 6, "two Samurai/Warriors: +2/+0");
}

/// Thawing Glaciers fetches a tapped basic and goes back to hand at cleanup.
#[test]
fn thawing_glaciers_returns_to_hand_at_cleanup() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::forest());
    let glaciers = ready(&mut g, 0, catalog::thawing_glaciers());
    activate(&mut g, glaciers, 0, None);
    assert_eq!(named_count(&g, 0, "Forest"), 1);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(g.players[0].hand.iter().any(|c| c.id == glaciers));
}

/// CR 724 — Chief Magistrate crowns you; at your upkeep the monarch copies
/// each creature token, the fresh Goblin included.
#[test]
fn chief_magistrate_copies_tokens_while_monarch() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let chief = g.add_card_to_hand(0, catalog::chief_magistrate_of_mercadia());
    cast_with(&mut g, chief, None, vec![], None);
    assert_eq!(g.monarch, Some(0));
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!(named_count(&g, 0, "Goblin"), 2, "the Goblin and its copy");
}

/// Lightmine Field hits each attacker for the number of attackers.
#[test]
fn lightmine_field_punishes_attackers() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::lightmine_field());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(a, 1), (b, 1)]);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "2 damage each");
}

/// Unforgiving One returns a creature card up to its modified-creature count.
#[test]
fn unforgiving_one_reanimates_by_modified_count() {
    let mut g = pod(2);
    let one = ready(&mut g, 0, catalog::unforgiving_one());
    g.battlefield_find_mut(one).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    declare(&mut g, vec![(one, 1)]);
    assert!(g.battlefield_find(bear).is_some(), "two modified creatures: MV 2 returns");
}

/// Weatherlight Compleated becomes a creature at four phyresis counters.
#[test]
fn weatherlight_compleated_crews_itself_with_phyresis() {
    use crabomination::card::CardType;
    let mut g = pod(2);
    for _ in 0..8 {
        g.add_card_to_library(0, catalog::island());
    }
    let ship = ready(&mut g, 0, catalog::weatherlight_compleated());
    assert!(!is_type(&g, ship, CardType::Creature));
    for _ in 0..4 {
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast_with(&mut g, bolt, Some(Target::Permanent(bear)), vec![], None);
    }
    assert_eq!(g.battlefield_find(ship).unwrap().counter_count(CounterType::Phyresis), 4);
    assert!(is_type(&g, ship, CardType::Creature));
}

/// Thomil's 0 trades a creature for {B}{B}{B}.
#[test]
fn thomil_sacrifices_for_black_mana() {
    let mut g = pod(2);
    let thomil = ready(&mut g, 0, catalog::thomil_the_destroyer());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![bear])]));
    loyalty(&mut g, thomil, 1, None);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[0].mana_pool.total() >= 3);
}

/// CR 701.15 — Balefang hands an opponent a tapped, goaded Baneslayer and
/// shuts off all life gain.
#[test]
fn balefang_gives_a_goaded_baneslayer() {
    let mut g = pod(3);
    let fang = g.add_card_to_hand(0, catalog::balefang_the_unslayable());
    cast_with(&mut g, fang, None, vec![], None);
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Baneslayer Angel").expect("token");
    assert_ne!(angel.controller, 0);
    assert!(angel.tapped);
    assert!(!angel.goad_holds.is_empty(), "goaded for the game");
}

/// Kitsune's entry swaps two other creatures controlled by different players.
#[test]
fn kitsune_exchanges_two_creatures() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::llanowar_elves());
    let theirs = ready(&mut g, 1, catalog::serra_angel());
    let kitsune = g.add_card_to_hand(0, catalog::kitsune_dragons_daughter());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_with(&mut g, kitsune, None, vec![], None);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
}

/// Flying Drone loots for free once another flyer entered this turn.
#[test]
fn flying_drone_loots_free_after_a_flyer() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let drone = ready(&mut g, 0, catalog::flying_drone());
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: drone, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "no flyer entered yet");
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast_with(&mut g, angel, None, vec![], None);
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: drone, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("free loot");
}

/// Taeko grows when another creature you control leaves without dying.
#[test]
fn taeko_grows_on_a_bounce() {
    let mut g = pod(2);
    let taeko = ready(&mut g, 0, catalog::taeko_the_patient_avalanche());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let evs = g.remove_from_battlefield_to_hand(bear);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(plus(&g, taeko), 1);
}

/// You Cannot Pass! only hits a creature in a block with a legend this turn.
#[test]
fn you_cannot_pass_needs_a_legendary_combatant() {
    let mut g = pod(2);
    let witch = ready(&mut g, 0, catalog::the_scarlet_witch());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let wall = ready(&mut g, 1, catalog::serra_angel());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    declare(&mut g, vec![(witch, 1), (bear, 1)]);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, witch), (elf, bear)])).expect("blocks");
    drain_stack(&mut g);
    let spell = g.add_card_to_hand(0, catalog::you_cannot_pass());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    let not_legend = g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(elf)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(not_legend.is_err(), "the Elf blocked a non-legend");
    cast_with(&mut g, spell, Some(Target::Permanent(wall)), vec![], None);
    assert!(g.battlefield_find(wall).is_none());
}

/// CR 614 — Selenia's death curses an opponent: their life loss doubles.
#[test]
fn selenias_curse_doubles_the_enchanted_players_loss() {
    let mut g = pod(2);
    let selenia = ready(&mut g, 0, catalog::selenia_the_cursed_heart());
    g.players[0].life = 10;
    let _ = g.adjust_life(0, 3);
    assert_eq!(g.players[0].life, 16, "gain doubled");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.battlefield_find_mut(selenia).unwrap().damage = 4;
    cast_with(&mut g, bolt, Some(Target::Permanent(selenia)), vec![], None);
    let curse = g.battlefield.iter().find(|c| c.definition.name == "Selenia's Curse").expect("curse token");
    assert_eq!(curse.attached_to_player, Some(1));
    let life = g.players[1].life;
    cast_with(&mut g, bolt2, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[1].life, life - 6);
}

/// CR 508.1d — under an opponent's Trove of Temptation, a declaration that
/// attacks only another player is illegal while a creature could go at the
/// Trove's controller; one attacker aimed there satisfies it.
#[test]
fn trove_of_temptation_demands_an_attack_on_its_controller() {
    let mut g = pod(3);
    ready(&mut g, 2, catalog::trove_of_temptation());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let at = |id, p| Attack { attacker: id, target: AttackTarget::Player(p) };
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![at(a, 1)])).is_err(), "Trove's controller unattacked");
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![])).is_err(), "an empty attack");
    g.perform_action(GameAction::DeclareAttackers(vec![at(a, 1), at(b, 2)])).expect("one at the Trove");
}

/// CR 506.2 — Wardscale Dragon silences only the player it attacks.
#[test]
fn wardscale_dragon_silences_its_defender() {
    let mut g = pod(3);
    let dragon = ready(&mut g, 0, catalog::wardscale_dragon());
    declare(&mut g, vec![(dragon, 1)]);
    for seat in [1, 2] {
        g.players[seat].mana_pool.add(Color::Red, 1);
    }
    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(2, catalog::lightning_bolt());
    let cast = |g: &mut GameState, seat: usize, id: CardId| {
        g.priority.player_with_priority = seat;
        g.perform_action(GameAction::CastSpell {
            card_id: id, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
        })
    };
    assert!(cast(&mut g, 1, bolt1).is_err(), "the defending player");
    assert!(cast(&mut g, 2, bolt2).is_ok(), "a bystander");
}

/// Ashes of the Abhorrent stops a flashback cast and a graveyard activation.
#[test]
fn ashes_of_the_abhorrent_locks_graveyards() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::ashes_of_the_abhorrent());
    let spell = g.add_card_to_graveyard(0, catalog::think_twice());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastFlashback {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sage = g.add_card_to_graveyard(0, catalog::sage_of_the_fang());
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: sage, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None, mode: None,
    }).is_err(), "a graveyard activation");
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(bear)), vec![], None);
    assert_eq!(g.players[1].life, life + 1);
}

/// CR 614 — Ormos turns a draw from an empty library into five counters (no
/// CR 104.3c loss), and its discard wants three different names.
#[test]
fn ormos_replaces_empty_draws_and_discards_distinct_names() {
    let mut g = pod(2);
    let ormos = ready(&mut g, 0, catalog::ormos_archive_keeper());
    let mut evs = Vec::new();
    let _ = g.draw_one_or_deck(0, &mut evs);
    let _ = g.check_state_based_actions();
    assert!(!g.players[0].eliminated);
    assert_eq!(plus(&g, ormos), 5);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    flood(&mut g);
    g.priority.player_with_priority = 0;
    let act = GameAction::ActivateAbility {
        card_id: ormos, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    };
    assert!(g.perform_action(act.clone()).is_err(), "three Islands share a name");
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(act).expect("Island, Forest, Bolt");
}

/// CR 119.4 — Killing Wave for X=3: a player at 4 life keeps one creature
/// (paying 3) and loses the other; nobody can pay past their life.
#[test]
fn killing_wave_pay_or_sacrifice() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::serra_angel());
    let a = ready(&mut g, 1, catalog::serra_angel());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    g.players[1].life = 4;
    g.players[0].life = 2;
    let wave = g.add_card_to_hand(0, catalog::killing_wave());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: wave, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "2 life can't pay 3");
    assert_eq!(g.players[0].life, 2);
    assert_eq!(g.players[1].life, 1, "paid for one");
    assert_eq!([a, b].iter().filter(|id| g.battlefield_find(**id).is_some()).count(), 1);
    assert!(g.battlefield_find(a).is_some(), "kept the costlier Angel");
}

// ── decks::cmdr_rescan_c ─────────────────────────────────────────────────────

/// Torrent of Souls paid with {B} returns the graveyard creature and, with
/// no {R} spent, pumps nobody.
#[test]
fn torrent_of_souls_reads_the_mana_spent() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let spell = g.add_card_to_hand(0, catalog::torrent_of_souls());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Permanent(bear)],
        mode: None,
        x_value: None,
    })
    .expect("cast with {B}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "{{B}} spent: the Bears return");
    assert_eq!(pt(&g, elf), (1, 1), "no {{R}}: no pump");
}

/// Doom Foretold: a player with nothing to sacrifice feeds its controller,
/// and the enchantment goes.
#[test]
fn doom_foretold_pays_out_when_a_player_cannot_sacrifice() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..3 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let doom = ready(&mut g, 0, catalog::doom_foretold());
    let life = g.players[1].life;
    step_to(&mut g, 1, TurnStep::Draw);
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.battlefield_find(doom).is_none(), "sacrificed");
    assert_eq!(named_count(&g, 0, "Knight"), 1);
}

/// The Meep's sacrifice sets the team's base power and toughness to its MV.
#[test]
fn the_meep_sets_base_pt_to_the_sacrificed_mana_value() {
    let mut g = pod(2);
    let meep = ready(&mut g, 0, catalog::the_meep());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![angel])]));
    declare(&mut g, vec![(meep, 1)]);
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!(pt(&g, meep), (5, 5));
}

/// Nephilim Epochal: +1/+1 per other Nephilim, and monocolored creatures
/// can't block them.
#[test]
fn nephilim_epochal_pumps_other_nephilim() {
    let mut g = pod(2);
    let epochal = ready(&mut g, 0, catalog::nephilim_epochal());
    ready(&mut g, 0, catalog::nephilim_epochal());
    assert_eq!(pt(&g, epochal), (6, 6), "each Epochal's static counts the other");
    assert!(g.permanent_has_keyword(epochal, &Keyword::CantBeBlockedBy(Box::new(crabomination::card::SelectionRequirement::Monocolored))));
}

/// Getaway Barrel dying puts a creature from the top thirteen onto the
/// battlefield.
#[test]
fn getaway_barrel_finds_a_creature() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let barrel = ready(&mut g, 0, catalog::getaway_barrel());
    let mut evs = Vec::new();
    g.sacrifice_one(barrel, 0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// CR 509.1b — Nephilim Epochal's evasion: a monocolored creature can't
/// block a Nephilim; a colorless one can.
#[test]
fn nephilim_cant_be_blocked_by_monocolored() {
    let mut g = pod(2);
    let epochal = ready(&mut g, 0, catalog::nephilim_epochal());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let golem = ready(&mut g, 1, catalog::marut());
    declare(&mut g, vec![(epochal, 1)]);
    g.step = TurnStep::DeclareBlockers;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(bear, epochal)])).is_err());
    g.perform_action(GameAction::DeclareBlockers(vec![(golem, epochal)])).expect("colorless blocker");
}

/// Boros Strike-Captain's card is playable on a turn you attacked with three
/// or more — the battalion turn, and again on a later such turn, but not a
/// turn you attacked with fewer.
#[test]
fn boros_strike_captain_card_wakes_on_three_attacker_turns() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..6 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let captain = ready(&mut g, 0, catalog::boros_strike_captain());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    declare(&mut g, vec![(captain, 1), (a, 1), (b, 1)]);
    let first = g.exile.iter().find(|c| c.owner == 0).map(|c| c.id).expect("exiled");
    let live = |g: &GameState| {
        g.exile.iter().find(|c| c.id == first).and_then(|c| c.may_play_until).is_some_and(|p| p.player == 0)
    };
    assert!(live(&g), "playable on the battalion turn");
    finish_combat(&mut g);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(!live(&g), "parked between turns");
    step_to(&mut g, 0, TurnStep::PreCombatMain);
    for id in [captain, a, b] {
        g.battlefield_find_mut(id).unwrap().tapped = false;
    }
    declare(&mut g, vec![(a, 1)]);
    assert!(!live(&g), "one attacker isn't three");
    g.battlefield_find_mut(a).unwrap().tapped = false;
    g.attacking.clear();
    declare(&mut g, vec![(captain, 1), (a, 1), (b, 1)]);
    assert!(live(&g), "woken by a later three-attacker turn");
}

/// Ayesha Tanaka, Armorer deploys artifacts up to her power from the top four
/// — a 2-drop artifact comes, a 3-drop doesn't.
#[test]
fn ayesha_deploys_artifacts_up_to_her_power() {
    let mut g = pod(2);
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let mind = g.add_card_to_library(0, catalog::mind_stone());
    let big = g.add_card_to_library(0, catalog::marut());
    g.add_card_to_library(0, catalog::island());
    let ayesha = ready(&mut g, 0, catalog::ayesha_tanaka_armorer());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![ring, mind])]));
    declare(&mut g, vec![(ayesha, 1)]);
    assert!(g.battlefield_find(ring).is_some() && g.battlefield_find(mind).is_some());
    assert!(g.battlefield_find(big).is_none());
}

/// Maeve draws when the creature she goaded attacks one of her opponents,
/// not when it attacks her.
#[test]
fn maeve_draws_when_her_goaded_creature_hits_an_opponent() {
    let mut g = pod(3);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let maeve = ready(&mut g, 0, catalog::maeve_insidious_singer());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, maeve, 0, Some(Target::Permanent(bear)));
    let hand = g.players[0].hand.len();
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]))
        .expect("attack seat 2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// CR 615.12 — Volcano Hellion's entry damage goes through protection from
/// red, and its echo is your life total in generic mana.
#[test]
fn volcano_hellion_burns_through_prevention_and_echoes_your_life() {
    let mut g = pod(2);
    for seat in 0..2 {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let knight = ready(&mut g, 1, catalog::cho_manno_revolutionary());
    let hellion = g.add_card_to_hand(0, catalog::volcano_hellion());
    g.players[0].life = 10;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    cast_with(&mut g, hellion, None, vec![], None);
    assert!(g.battlefield_find(knight).is_none(), "unpreventable damage through Cho-Manno's prevention");
    assert_eq!(g.players[0].life, 8);
    // Next upkeep: 8 generic owed and nothing to pay with — sacrificed.
    g.players[0].mana_pool.empty();
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Draw);
    assert!(g.battlefield_find(hellion).is_none(), "echo unpaid");
}

/// CR 114.4 — Saheeli's −4 emblem pumps artifact creatures and discounts
/// artifact spells: Mind Stone ({2}) casts off {1}.
#[test]
fn saheeli_emblem_discounts_artifact_spells() {
    let mut g = pod(2);
    let saheeli = ready(&mut g, 0, catalog::saheeli_filigree_master());
    g.battlefield_find_mut(saheeli).unwrap().add_counters(CounterType::Loyalty, 2);
    loyalty(&mut g, saheeli, 2, None);
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{1} pays for Mind Stone under the emblem");
}

/// CR 603.2 — Dr. Beverly Crusher doubles the triggers her controller's life
/// gain causes, her own included: one gain, two counters.
#[test]
fn dr_crusher_doubles_lifegain_triggers() {
    let mut g = pod(2);
    let crusher = ready(&mut g, 0, catalog::dr_beverly_crusher());
    let spell = g.add_card_to_hand(0, catalog::healing_salve());
    cast_with(&mut g, spell, Some(Target::Player(0)), vec![], None);
    assert_eq!(plus(&g, crusher), 2);
}
