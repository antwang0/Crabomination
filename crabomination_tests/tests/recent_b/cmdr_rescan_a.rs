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
    // CR 611.2c — the gained "{T}: Add {U}" ends with the land-ness.
    assert!(g.battlefield_find(hydro).unwrap().granted_activated_abilities.is_empty(), "the gained ability is gone");
}

/// Hydro-Man has no mana ability as a creature: it is gained with the land-ness.
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
    assert_eq!(angel.definition.cost.cmc(), 5, "a {{3}}{{W}}{{W}} token (CR 111.3)");
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

/// CR 508.1 — under Tomik, a planeswalker takes one attacker a combat.
#[test]
fn tomik_caps_attackers_per_planeswalker() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::tomik_orzhov_lawmage());
    let walker = ready(&mut g, 1, catalog::ajani_outland_chaperone());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let at_pw = |id| Attack { attacker: id, target: AttackTarget::Planeswalker(walker) };
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![at_pw(a), at_pw(b)])).is_err());
    g.perform_action(GameAction::DeclareAttackers(vec![at_pw(a), Attack { attacker: b, target: AttackTarget::Player(1) }]))
        .expect("one at the planeswalker");
}

/// Hew the Entwood: sacrifice N lands, reveal N, and the chosen artifact and
/// land cards enter — lands tapped, nonland artifacts untapped (CR 701.21).
#[test]
fn hew_the_entwood_trades_lands_for_revealed_artifacts() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::forest());
    }
    let citadel = g.add_card_to_library(0, catalog::darksteel_citadel());
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    let hew = g.add_card_to_hand(0, catalog::hew_the_entwood());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(3),
        DecisionAnswer::Cards(vec![citadel, ring]),
    ]));
    cast_with(&mut g, hew, None, vec![], None);
    let forests = g.battlefield.iter().filter(|c| c.definition.name == "Forest").count();
    assert_eq!(forests, 0);
    assert!(g.battlefield_find(citadel).is_some_and(|c| c.tapped));
    assert!(g.battlefield_find(ring).is_some_and(|c| !c.tapped));
    assert!(g.battlefield_find(bolt).is_none() && !g.players[0].hand.iter().any(|c| c.id == bolt));
}

/// Hawkeye: becoming tapped lets you pay up to {3} and choose that many modes
/// (CR 700.2); paying {2} picks Explosive and Boomerang here. In a pod the
/// damage mode's auto-target is an opponent.
#[test]
fn hawkeye_trick_arrows_pay_two_for_two_modes() {
    let mut g = pod(3);
    let hawk = ready(&mut g, 0, catalog::hawkeye_master_marksman());
    let filler = g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    flood(&mut g);
    let pool = g.players[0].mana_pool.total();
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2), DecisionAnswer::Modes(vec![1, 2])]));
    declare(&mut g, vec![(hawk, 1)]);
    assert_eq!(g.players[0].mana_pool.total(), pool - 2);
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == filler));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Bug fix: the up-to-{3} taps lands — nothing floats, two Islands pay {2}
/// (it read the floating pool only, so a Commander bot never paid).
#[test]
fn hawkeye_pays_from_untapped_lands() {
    let mut g = pod(3);
    let hawk = ready(&mut g, 0, catalog::hawkeye_master_marksman());
    let islands: Vec<CardId> = (0..2).map(|_| ready(&mut g, 0, catalog::island())).collect();
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let life = g.players[1].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2), DecisionAnswer::Modes(vec![1, 2])]));
    declare(&mut g, vec![(hawk, 1)]);
    assert!(islands.iter().all(|&i| g.battlefield_find(i).is_some_and(|c| c.tapped)));
    assert_eq!(g.players[1].life, life - 2);
}

/// Zareth San swaps in for an unblocked attacking Rogue (tapped and attacking,
/// CR 508.4), then connects to steal a permanent card from their graveyard.
#[test]
fn zareth_san_swaps_for_an_unblocked_rogue_and_steals() {
    let mut g = pod(2);
    let rogue = ready(&mut g, 0, catalog::nephalia_smuggler());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let zareth = g.add_card_to_hand(0, catalog::zareth_san_the_trickster());
    let ring = g.add_card_to_graveyard(1, catalog::sol_ring());
    declare(&mut g, vec![(rogue, 1), (bear, 1)]);
    // CR 509.1h — before blocks the Rogue is not yet unblocked.
    flood(&mut g);
    let early = g.perform_action(GameAction::ActivateAbility {
        card_id: zareth,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(early.is_err(), "no unblocked Rogue in the declare attackers step");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    activate(&mut g, zareth, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == rogue));
    let z = g.battlefield.iter().find(|c| c.definition.name == "Zareth San, the Trickster").expect("in play");
    assert!(z.tapped);
    let z = z.id;
    assert!(g.attacking.iter().any(|a| a.attacker == z));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
    }
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring" && c.controller == 0), "{ring:?}");
}

/// CR 602.5b / 107.3 — Fabrication Foundry's X is the total mana value it
/// exiles: a target the board can't cover is rejected, and a coverable one
/// costs the cheapest cover (one Arcane Signet for a Mind Stone).
#[test]
fn fabrication_foundry_exiles_enough_mana_value_to_cover_the_target() {
    let mut g = pod(2);
    let foundry = ready(&mut g, 0, catalog::fabrication_foundry());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    let signet = ready(&mut g, 0, catalog::arcane_signet());
    let solemn = g.add_card_to_graveyard(0, catalog::solemn_simulacrum());
    let stone = g.add_card_to_graveyard(0, catalog::mind_stone());
    flood(&mut g);
    let act = |g: &mut GameState, t: CardId| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: foundry,
            ability_index: 1,
            target: Some(Target::Permanent(t)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(act(&mut g, solemn).is_err(), "3 mana value on board can't cover a 4");
    act(&mut g, stone).expect("a 2 is covered");
    drain_stack(&mut g);
    assert!(g.battlefield_find(stone).is_some());
    assert!(g.battlefield_find(signet).is_none());
    assert!(g.battlefield_find(ring).is_some() && g.battlefield_find(thopter).is_some());
}

/// CR 702.142 / 707.12 — Zemo's boast needs fifteen black pips of exiled
/// cards (twelve fall short); paid, it copies all four Obliterators and casts
/// three of the copies, leaving the one-pip Duress in the graveyard. The
/// copies are cast from exile, so Zemo's from-hand connive stays quiet (with
/// an empty library it would have drawn him out of the game).
#[test]
fn baron_zemo_boast_counts_black_pips_and_casts_three_copies() {
    let mut g = pod(2);
    let zemo = ready(&mut g, 0, catalog::baron_helmut_zemo());
    let duress = g.add_card_to_graveyard(0, catalog::duress());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::phyrexian_obliterator());
    }
    declare(&mut g, vec![(zemo, 1)]);
    let boast = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility {
            card_id: zemo,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(boast(&mut g).is_err(), "13 black pips");
    g.add_card_to_graveyard(0, catalog::phyrexian_obliterator());
    boast(&mut g).expect("16 black pips");
    drain_stack(&mut g);
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Obliterator").count();
    assert_eq!(copies, 3);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == duress));
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert!(g.game_over.is_none());
}

/// "You may cast up to three of the copies" is the controller's pick (it was
/// always the priciest three): here one of the four.
#[test]
fn baron_zemo_casts_only_the_copies_his_controller_names() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let zemo = ready(&mut g, 0, catalog::baron_helmut_zemo());
    let obliterators: Vec<_> = (0..4).map(|_| g.add_card_to_graveyard(0, catalog::phyrexian_obliterator())).collect();
    declare(&mut g, vec![(zemo, 1)]);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![obliterators[2]])]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: zemo,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("16 black pips");
    drain_stack(&mut g);
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian Obliterator").count();
    assert_eq!(copies, 1);
}

/// Zemo connives when you cast a black spell from your hand, not a red one.
#[test]
fn baron_zemo_connives_on_black_spells_from_hand() {
    let mut g = pod(2);
    let zemo = ready(&mut g, 0, catalog::baron_helmut_zemo());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[0].hand.len(), 0);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let duress = g.add_card_to_hand(0, catalog::duress());
    cast_with(&mut g, duress, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name != "Lightning Bolt").count(), 2);
    assert_eq!(plus(&g, zemo), 1, "a nonland discard");
}

/// CR 701.28 / 702.43a — Blaster cast converted enters back face up with its
/// modular 3 counters; moving them all away converts it back to the Robot.
#[test]
fn blaster_cast_converted_enters_with_modular_counters_and_spends_them() {
    let mut g = pod(2);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let blaster = g.add_card_to_hand(0, catalog::blaster_combat_dj());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: blaster,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("More Than Meets the Eye");
    drain_stack(&mut g);
    let b = g.battlefield_find(blaster).expect("in play");
    assert_eq!(b.definition.name, "Blaster, Morale Booster");
    assert_eq!(plus(&g, blaster), 3);
    let shift = |g: &mut GameState, x: u32| {
        flood(g);
        g.battlefield_find_mut(blaster).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: blaster,
            ability_index: 0,
            target: Some(Target::Permanent(ring)),
            additional_targets: vec![],
            x_value: Some(x),
            mode: None,
        })
        .expect("move counters");
        drain_stack(g);
    };
    shift(&mut g, 2);
    assert_eq!(plus(&g, ring), 2);
    assert!(g.permanent_has_keyword(ring, &Keyword::Haste));
    assert_eq!(g.battlefield_find(blaster).unwrap().definition.name, "Blaster, Morale Booster");
    shift(&mut g, 1);
    assert_eq!(plus(&g, ring), 3);
    assert_eq!(g.battlefield_find(blaster).unwrap().definition.name, "Blaster, Combat DJ");
}

/// Blaster's front grants modular 1 to your other nontoken artifact
/// creatures (CR 702.43a), and a +1/+1 counter put on Blaster converts it.
#[test]
fn blaster_grants_modular_and_converts_on_counters() {
    let mut g = pod(2);
    let blaster = ready(&mut g, 0, catalog::blaster_combat_dj());
    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    cast_with(&mut g, thopter, None, vec![], None);
    assert_eq!(plus(&g, thopter), 1);
    assert!(g.permanent_has_keyword(thopter, &Keyword::Modular(1)));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(plus(&g, bear), 0, "not an artifact");
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::AddCounter {
                what: crabomination::effect::Selector::EachPermanent(crabomination::card::SelectionRequirement::Artifact.and(crabomination::card::SelectionRequirement::Creature)),
                kind: CounterType::PlusOnePlusOne,
                amount: crabomination::card::Value::ONE,
            },
            &ctx,
        )
        .expect("counters");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(blaster).unwrap().definition.name, "Blaster, Morale Booster");
}

/// CR 702.43a — the modular Blaster grants is "put into a graveyard from the
/// battlefield": an uncrewed Vehicle (not a creature) moves its counter too.
#[test]
fn cr_702_43a_blaster_modular_moves_an_uncrewed_vehicles_counter() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let blaster = ready(&mut g, 0, catalog::blaster_combat_dj());
    let copter = g.add_card_to_hand(0, catalog::smugglers_copter());
    cast_with(&mut g, copter, None, vec![], None);
    assert_eq!(plus(&g, copter), 1, "entered with modular 1's counter");
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    let before = plus(&g, thopter) + plus(&g, blaster);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_ability(copter, 0, Some(Target::Permanent(copter)));
    let evs = g
        .resolve_effect(&crabomination::effect::Effect::Destroy { what: crabomination::effect::Selector::Target(0) }, &ctx)
        .expect("destroy");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(copter).is_none());
    assert_eq!(plus(&g, thopter) + plus(&g, blaster), before + 1, "the Vehicle's counter moved");
}

/// CR 702.15 — Firesong gives red instants and sorceries lifelink; a red
/// spell's lifelink gain isn't a white spell's, so the 3-damage trigger stays
/// quiet.
#[test]
fn firesong_lends_lifelink_to_red_spells_only() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::firesong_and_sunspeaker());
    let (me, p1, p2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[0].life, me + 3);
    assert_eq!(g.players[1].life + g.players[2].life, p1 + p2 - 3);
}

/// A white instant that gains you life sets off Firesong's 3 damage.
#[test]
fn firesong_burns_when_a_white_spell_gains_you_life() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::firesong_and_sunspeaker());
    let (me, p1, p2) = (g.players[0].life, g.players[1].life, g.players[2].life);
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    cast_with(&mut g, salve, Some(Target::Player(0)), vec![], Some(0));
    assert_eq!(g.players[0].life, me + 3);
    assert_eq!(g.players[1].life + g.players[2].life, p1 + p2 - 3);
}

/// Koh exiles a dying creature and wears it: paying 1 life lends the chosen
/// exiled card's activated abilities, and a new choice replaces the old one
/// (only the last chosen card lends).
#[test]
fn koh_wears_the_last_chosen_face() {
    let mut g = pod(2);
    let koh = ready(&mut g, 0, catalog::koh_the_face_stealer());
    let elves = ready(&mut g, 1, catalog::llanowar_elves());
    let sorcerer = ready(&mut g, 1, catalog::prodigal_sorcerer());
    for victim in [elves, sorcerer] {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast_with(&mut g, bolt, Some(Target::Permanent(victim)), vec![], None);
        assert!(g.exile.iter().any(|c| c.id == victim && c.exiled_with == Some(koh)));
    }
    let own = g.battlefield_find(koh).unwrap().definition.activated_abilities.len();
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![elves])]));
    activate(&mut g, koh, 0, None);
    assert_eq!(g.players[0].life, life - 1);
    let names = |g: &GameState| g.battlefield_find(koh).unwrap().definition.activated_abilities.len();
    assert_eq!(names(&g), own + 1);
    g.players[0].mana_pool.empty();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: koh,
        ability_index: own,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the Elves' mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![sorcerer])]));
    activate(&mut g, koh, 0, None);
    assert_eq!(names(&g), own + 1, "the Sorcerer replaces the Elves");
    let borrowed = &g.battlefield_find(koh).unwrap().definition.activated_abilities[own];
    assert!(!matches!(borrowed.effect, crabomination::effect::Effect::AddMana { .. }));
}

/// CR 601.2c / 603.3d — Koh's enter trigger exiles "up to one" other creature:
/// with no opposing creature it declines rather than exiling its controller's.
#[test]
fn koh_enters_without_exiling_its_own_creature() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let koh = g.add_card_to_hand(0, catalog::koh_the_face_stealer());
    cast_with(&mut g, koh, None, vec![], None);
    assert!(g.battlefield_find(koh).is_some());
    assert!(g.battlefield_find(bear).is_some(), "the bear stays");
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let koh = g.add_card_to_hand(0, catalog::koh_the_face_stealer());
    cast_with(&mut g, koh, None, vec![], None);
    assert!(g.exile.iter().any(|c| c.id == foe), "an opposing creature is still taken");
}

/// Koh's new choice drops only the old borrow: another ability baked onto Koh
/// (here a granted trigger) survives the swap.
#[test]
fn koh_new_face_keeps_other_grants() {
    let mut g = pod(2);
    let koh = ready(&mut g, 0, catalog::koh_the_face_stealer());
    let elves = ready(&mut g, 1, catalog::llanowar_elves());
    let sorcerer = ready(&mut g, 1, catalog::prodigal_sorcerer());
    for victim in [elves, sorcerer] {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast_with(&mut g, bolt, Some(Target::Permanent(victim)), vec![], None);
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![elves])]));
    activate(&mut g, koh, 0, None);
    let granted = catalog::blood_artist().triggered_abilities[0].clone();
    g.battlefield_find_mut(koh).unwrap().bake_grant().triggered_abilities.push(granted.clone());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![sorcerer])]));
    activate(&mut g, koh, 0, None);
    let def = &g.battlefield_find(koh).unwrap().definition;
    assert!(def.triggered_abilities.contains(&granted), "the other grant stays");
    assert!(!def.activated_abilities.iter().any(|a| matches!(a.effect, crabomination::effect::Effect::AddMana { .. })));
}

/// CR 700.9 — Lethal Exploit's shrink grows by one per modified creature you
/// control: two +1/+1-countered bears take a Serra Angel to -4/-4.
#[test]
fn lethal_exploit_counts_your_modified_creatures() {
    let mut g = pod(2);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::lethal_exploit());
    cast_with(&mut g, spell, Some(Target::Permanent(angel)), vec![], None);
    assert_eq!(pt(&g, angel), (2, 2), "no modified creatures: -2/-2");
    for _ in 0..2 {
        let bear = ready(&mut g, 0, catalog::grizzly_bears());
        g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let other = ready(&mut g, 1, catalog::serra_angel());
    let spell = g.add_card_to_hand(0, catalog::lethal_exploit());
    cast_with(&mut g, spell, Some(Target::Permanent(other)), vec![], None);
    assert!(g.battlefield_find(other).is_none());
}

/// CR 702.11 — after Detection Tower's ability, an opponent's hexproof
/// creature and the hexproof opponent themself are legal targets for you this
/// turn, and only this turn.
#[test]
fn detection_tower_strips_opponents_hexproof_for_the_turn() {
    let mut g = pod(2);
    let tower = ready(&mut g, 0, catalog::detection_tower());
    let bogle = ready(&mut g, 1, catalog::slippery_bogle());
    ready(&mut g, 1, catalog::leyline_of_sanctity());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    let cast = |g: &mut GameState, t: Target| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(t), additional_targets: vec![], mode: None, x_value: None })
    };
    assert!(cast(&mut g, Target::Permanent(bogle)).is_err());
    assert!(cast(&mut g, Target::Player(1)).is_err());
    activate(&mut g, tower, 1, None);
    cast(&mut g, Target::Player(1)).expect("hexproof player is fair game");
    drain_stack(&mut g);
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2,
        target: Some(Target::Permanent(bogle)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("hexproof creature is fair game");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bogle).is_none());
    g.turn_number += 1;
    let bolt3 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let _ = bolt3;
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt3,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the grant ends with the turn"
    );
}

fn lore(g: &mut GameState, saga: CardId) {
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let evs = g
        .resolve_effect(
            &crabomination::effect::Effect::AddCounter {
                what: crabomination::effect::Selector::ExactObjects(vec![saga]),
                kind: CounterType::Lore,
                amount: crabomination::card::Value::ONE,
            },
            &ctx,
        )
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

/// CR 714.2 — Fall of the Impostor's chapter III exiles the target
/// opponent's greatest-power creature.
#[test]
fn fall_of_the_impostor_exiles_the_biggest() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::fall_of_the_impostor());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    lore(&mut g, saga);
    assert!(g.battlefield_find(angel).is_none());
    assert!(g.battlefield_find(bear).is_some());
}

/// CR 714.2 — The Phasing of Zhalfir's III wipes creatures and pays each
/// controller a 2/2 Phyrexian per creature of theirs destroyed.
#[test]
fn phasing_of_zhalfir_replaces_the_dead_with_phyrexians() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_phasing_of_zhalfir());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::serra_angel());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    lore(&mut g, saga);
    let phyrexians = |seat: usize| {
        g.battlefield.iter().filter(|c| c.definition.name == "Phyrexian" && c.controller == seat).count()
    };
    assert_eq!((phyrexians(0), phyrexians(1)), (1, 2));
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// CR 702.26 — "can't phase in for as long as **you control** this Saga":
/// the phased-out permanent comes back when its controller loses control of
/// the Saga, not only when the Saga leaves.
#[test]
fn phasing_of_zhalfir_releases_when_you_lose_control_of_the_saga() {
    let mut g = pod(2);
    let saga = ready(&mut g, 0, catalog::the_phasing_of_zhalfir());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let chapter = g.battlefield_find(saga).unwrap().definition.saga_chapters[0].1.clone();
    let ctx = crabomination::game::effects::EffectContext::for_ability(saga, 0, Some(Target::Permanent(bear)));
    g.resolve_effect(&chapter, &ctx).expect("chapter I");
    assert!(g.phased_out.iter().any(|c| c.id == bear), "phased out, held by the Saga");
    // Losing control of the Saga ends the hold (CR 702.26); the Saga leaving
    // already did.
    let steal = crabomination::effect::Effect::GainControl {
        what: crabomination::effect::Selector::ExactObjects(vec![saga]),
        to: Some(crabomination::effect::PlayerRef::Seat(1)),
        duration: crabomination::effect::Duration::Permanent,
    };
    let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    let evs = g.resolve_effect(&steal, &ctx).expect("steal the Saga");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(saga).map(|c| c.controller), Some(1));
    assert!(g.battlefield_find(bear).is_some(), "phased in when control changed");
}

/// CR 702.188 — Spiders-Man cast with web-slinging gains 3 and makes two
/// Spiders; cast normally it does neither.
#[test]
fn spiders_man_rewards_web_slinging() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let horde = g.add_card_to_hand(0, catalog::spiders_man_heroic_horde());
    flood(&mut g);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: horde,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("web-slinging");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert_eq!(g.players[0].life, life + 3);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spider").count(), 2);
    // The web-slinging kick is no granted offspring: no 1/1 copy of a legend
    // to send the original to the graveyard.
    assert!(g.battlefield_find(horde).is_some());
}

/// Grim Reaper's Scythe returns a creature card with a finality counter, and
/// the card leaving the graveyard mints a Zombie (CR 603.2c, once per batch).
#[test]
fn grim_reapers_scythe_reanimates_and_mints_a_zombie() {
    let mut g = pod(2);
    let scythe = ready(&mut g, 0, catalog::grim_reapers_scythe());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    activate(&mut g, scythe, 0, Some(Target::Permanent(giant)));
    let back = g.battlefield_find(giant).expect("returned");
    assert_eq!(back.counter_count(CounterType::Finality), 1);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Zombie").count(), 1);
}

/// Devout Invocation makes an Angel per creature actually tapped.
#[test]
fn devout_invocation_angels_per_creature_tapped() {
    let mut g = pod(2);
    let bears: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    let spell = g.add_card_to_hand(0, catalog::devout_invocation());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bears[0], bears[2]])]));
    cast_with(&mut g, spell, None, vec![], None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Angel").count(), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears" && c.tapped).count(), 2);
}

/// Spry and Mighty: X is the difference in power — a 4/4 and a 2/2 draw two
/// and each get +2/+2 and trample.
#[test]
fn spry_and_mighty_reads_the_power_gap() {
    let mut g = pod(2);
    let angel = ready(&mut g, 0, catalog::serra_angel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::spry_and_mighty());
    cast_with(&mut g, spell, None, vec![], None);
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(pt(&g, angel), (6, 6));
    assert_eq!(pt(&g, bear), (4, 4));
    assert!(g.permanent_has_keyword(bear, &Keyword::Trample));
}

/// Knowledge Exploitation casts an instant out of the target opponent's
/// library for free (CR 601.2, 118.9).
#[test]
fn knowledge_exploitation_casts_from_their_library() {
    let mut g = pod(2);
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::knowledge_exploitation());
    let lives = g.players[0].life + g.players[1].life;
    cast_with(&mut g, spell, Some(Target::Player(1)), vec![], None);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "the Bolt resolved into its owner's graveyard");
    assert_eq!(g.players[1].library.len(), 1);
    assert_eq!(g.players[0].life + g.players[1].life, lives - 3);
}

/// CR 702.78 — Raiding Schemes gives your noncreature spells conspire: two
/// red creatures tapped copy a Bolt from hand.
#[test]
fn raiding_schemes_conspires_noncreature_spells() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::raiding_schemes());
    let a = ready(&mut g, 0, catalog::hill_giant());
    let b = ready(&mut g, 0, catalog::hill_giant());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    flood(&mut g);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellConspire {
        card_id: bolt,
        conspire_creatures: [a, b],
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("conspired Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 6);
}

/// CR 101.2 / 500 — Azor's ETB locks each opponent out of instants and
/// sorceries during that player's next turn, and only that turn.
#[test]
fn azor_silences_each_opponents_next_turn() {
    let mut g = pod(3);
    for seat in 0..3 {
        for _ in 0..4 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    let azor = g.add_card_to_hand(0, catalog::azor_the_lawbringer());
    cast_with(&mut g, azor, None, vec![], None);
    let try_bolt = |g: &mut GameState| {
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
    };
    step_to(&mut g, 1, TurnStep::PreCombatMain);
    assert!(try_bolt(&mut g).is_err(), "their next turn is locked");
    step_to(&mut g, 2, TurnStep::PreCombatMain);
    try_bolt(&mut g).expect("the lock lasted one turn");
}

/// Azor's attack payment {X}{W}{U}{U} taps lands: nothing floats, five lands
/// pay X = 2 — two life, two cards.
#[test]
fn azor_pays_its_attack_x_from_lands() {
    let mut g = pod(2);
    let azor = ready(&mut g, 0, catalog::azor_the_lawbringer());
    ready(&mut g, 0, catalog::plains());
    for _ in 0..4 {
        ready(&mut g, 0, catalog::island());
    }
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    declare(&mut g, vec![(azor, 1)]);
    assert_eq!((g.players[0].life, g.players[0].hand.len()), (life + 2, hand + 2));
}

/// CR 702.30a — paying Shah of Naar Isle's echo {0} lets each opponent draw
/// up to three cards.
#[test]
fn shah_of_naar_isle_echo_paid_feeds_opponents() {
    let mut g = pod(2);
    let shah = ready(&mut g, 0, catalog::shah_of_naar_isle());
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    let evs = g.process_echo();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(shah).is_some_and(|c| c.echo_paid));
    assert_eq!(g.players[1].hand.len(), 3);
}

/// CR 702.33d / 700.2 — Go Nuts! takes both modes only when teamwork 3 is
/// paid: the Hill Giant taps to cover it, the bear gets the counter, and the
/// Giant fights their bear.
#[test]
fn go_nuts_teamwork_unlocks_both_modes() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::go_nuts());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: spell,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(giant), Target::Permanent(theirs)],
        x_value: None,
    })
    .expect("teamwork pays for both");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).unwrap().tapped);
    assert_eq!(plus(&g, bear), 1);
    assert!(g.battlefield_find(theirs).is_none());

    // Without 3 power to tap, both modes are out of reach.
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::go_nuts());
    flood(&mut g);
    assert!(g
        .perform_action(GameAction::CastSpellSpree {
            card_id: spell,
            spree_modes: vec![0, 1],
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![Target::Permanent(bear), Target::Permanent(theirs)],
            x_value: None,
        })
        .is_err());
}

/// CR 613.1 / 702.6 — a legend entering lets The Irencrag become Everflame:
/// a legendary Equipment with equip {3} and +3/+3, without its mana ability.
#[test]
fn the_irencrag_becomes_everflame() {
    let mut g = pod(2);
    let crag = ready(&mut g, 0, catalog::the_irencrag());
    let tomik = g.add_card_to_hand(0, catalog::tomik_orzhov_lawmage());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast_with(&mut g, tomik, None, vec![], None);
    let c = g.battlefield_find(crag).unwrap();
    assert_eq!(c.definition.name, "Everflame, Heroes' Legacy");
    assert!(c.definition.activated_abilities.is_empty(), "the {{T}}: Add {{C}} is gone");
    assert!(c.definition.keywords.iter().any(|k| matches!(k, Keyword::Equip(_))));
    g.clear_sickness(tomik);
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: crag, target: tomik }).expect("equip {3}");
    drain_stack(&mut g);
    assert_eq!(pt(&g, tomik), (5, 4));
}

/// CR 715.3 / 601.2b — Allure of Power costs a creature: with none to
/// sacrifice the Adventure can't be cast; with one it goes and two are drawn.
/// The card's main-half cast pays no sacrifice.
#[test]
fn my_precious_allure_of_power_sacrifices_as_a_cost() {
    let mut g = pod(2);
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let precious = g.add_card_to_hand(0, catalog::my_precious());
    flood(&mut g);
    let adventure = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastAdventure {
            card_id: precious,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(adventure(&mut g).is_err(), "no creature to sacrifice");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    adventure(&mut g).expect("Allure of Power");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.players[0].hand.len(), 2);
    assert!(g.exile.iter().any(|c| c.id == precious));

    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let precious = g.add_card_to_hand(0, catalog::my_precious());
    cast_with(&mut g, precious, None, vec![], None);
    assert!(g.battlefield_find(precious).is_some() && g.battlefield_find(bear).is_some());
}

/// Diaochan destroys your pick and then an opponent's pick, but only on your
/// turn before attackers (CR 602.5b timing restriction).
#[test]
fn diaochan_destroys_two_before_attackers() {
    let mut g = pod(2);
    let diaochan = ready(&mut g, 0, catalog::diaochan_artful_beauty());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::ActivateAbility {
            card_id: diaochan,
            ability_index: 0,
            target: Some(Target::Permanent(angel)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(), "after combat");
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, diaochan, 0, Some(Target::Permanent(angel)));
    assert!(g.battlefield_find(angel).is_none());
    let creatures = g.battlefield.iter().filter(|c| c.definition.card_types.contains(&crabomination::card::CardType::Creature)).count();
    assert_eq!(creatures, 1, "the opponent's pick died too");
}
