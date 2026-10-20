//! Commander: the 2026-10-20 (`01DDeUtc`) closers (`decks::cmdr_01dd_*`).

use crabomination::card::{CardId, CardType, CreatureType, Subtypes, TokenDefinition};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 3);
    }
    g.players[seat].mana_pool.add_colorless(8);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), GameError> {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })?;
    drain_stack(g);
    Ok(())
}

fn beast(power: i32) -> TokenDefinition {
    TokenDefinition {
        name: "Beast".into(),
        power,
        toughness: power,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        ..Default::default()
    }
}

fn treasures(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Treasure").count()
}

/// CR 702.110a-b — Henry Wu grants exploit to a Human entering; exploiting a
/// non-Human with power 3 draws a card and makes a Treasure.
#[test]
fn cr_702_110b_henry_wu_pays_for_a_big_non_human() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::henry_wu_ingen_geneticist());
    let token = g.add_token_to_battlefield(0, &beast(3));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, vanguard, None).expect("cast");
    assert!(g.battlefield_find(token).is_none(), "the Beast was exploited");
    assert_eq!(g.players[0].hand.len(), hand, "cast one, drew one");
    assert_eq!(treasures(&g, 0), 1, "power 3 or greater");
}

/// CR 702.110b — a non-Human with power under 3 draws without a Treasure.
#[test]
fn henry_wu_small_non_human_draws_only() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::henry_wu_ingen_geneticist());
    g.add_token_to_battlefield(0, &beast(2));
    let vanguard = g.add_card_to_hand(0, catalog::elite_vanguard());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, vanguard, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(treasures(&g, 0), 0);
}

/// CR 702.110b — Skull Skaab's Zombie needs a nontoken exploit: a token
/// exploited makes nothing, a Grizzly Bears exploited makes a 2/2 Zombie.
#[test]
fn cr_702_110b_skull_skaab_needs_a_nontoken_creature() {
    let zombies = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Zombie" && c.is_token).count();
    let mut g = pod(2);
    g.add_token_to_battlefield(0, &beast(1));
    let skaab = g.add_card_to_hand(0, catalog::skull_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, skaab, None).expect("cast");
    assert_eq!(zombies(&g), 0, "a token exploited");

    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let skaab = g.add_card_to_hand(0, catalog::skull_skaab());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, skaab, None).expect("cast");
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(zombies(&g), 1, "a nontoken creature exploited");
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker,
        target: crabomination::game::types::AttackTarget::Player(defender),
    }]))
    .expect("attack");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Harpoon Sniper's X counts every Merfolk you control, itself included:
/// with two others it deals 3 to a blocker-less attacker.
#[test]
fn harpoon_sniper_counts_your_merfolk() {
    let mut g = pod(2);
    let sniper = ready(&mut g, 1, catalog::harpoon_sniper());
    ready(&mut g, 1, catalog::judge_of_currents());
    ready(&mut g, 1, catalog::harpoon_sniper());
    let giant = ready(&mut g, 0, catalog::hill_giant());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: giant,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sniper,
        ability_index: 0,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_none(), "3 damage to a 3/3");
}

/// Judge of Currents fires for a Merfolk of yours becoming tapped (an attack
/// taps it) and not for an opponent's.
#[test]
fn judge_of_currents_gains_on_your_merfolk_tapping() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::judge_of_currents());
    let sniper = ready(&mut g, 0, catalog::harpoon_sniper());
    let theirs = ready(&mut g, 1, catalog::harpoon_sniper());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    attack(&mut g, sniper, 1);
    assert_eq!(g.players[0].life, life + 1);
    g.battlefield_find_mut(theirs).unwrap().tapped = false;
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: theirs, actor: None, as_attacker: false }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "an opponent's Merfolk");
}

/// Sygg, Wanderwine Wisdom's entry grants "deals combat damage to a player,
/// draw a card" to its target; Sygg can't be blocked and draws on the hit.
#[test]
fn sygg_wanderwine_wisdom_grants_a_draw_on_combat_damage() {
    let mut g = pod(2);
    let sygg = g.add_card_to_hand(0, catalog::sygg_wanderwine_wisdom());
    flood(&mut g, 0);
    cast(&mut g, sygg, None).expect("cast");
    g.clear_sickness(sygg);
    let hand = g.players[0].hand.len();
    attack(&mut g, sygg, 1);
    assert_eq!(g.players[1].life, 18, "unblocked");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off the granted trigger");
}

/// Sygg transforms at your first main phase when you pay {W}; entering the
/// back face gives a creature you control protection from each color until
/// your next turn.
#[test]
fn sygg_pays_w_to_transform_and_shield() {
    let mut g = pod(2);
    let sygg = ready(&mut g, 0, catalog::sygg_wanderwine_wisdom());
    ready(&mut g, 0, catalog::plains());
    g.step = TurnStep::Upkeep;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sygg).unwrap().definition.name, "Sygg, Wanderbrine Shield");
    assert!(g.computed_permanent(sygg).unwrap().keywords().contains(&crabomination::card::Keyword::Protection(Color::Red)));
}

fn cast_no_drain(g: &mut GameState, id: CardId, target: Option<Target>, mode: Option<usize>) {
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
        .expect("cast");
}

/// Walk seat 0 into its upkeep from seat 1's end step and resolve what fired.
fn into_upkeep(g: &mut GameState) {
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::End;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep) {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// CR 702.24a — Thought Lash's upkeep is paid by exiling the top card per
/// age counter; with too few cards it goes unpaid, it's sacrificed, and its
/// trigger exiles the rest of the library.
#[test]
fn cr_702_24a_thought_lash_exiles_the_top_or_the_whole_library() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let lash = g.add_card_to_battlefield(0, catalog::thought_lash());
    into_upkeep(&mut g);
    assert!(g.battlefield_find(lash).is_some(), "paid");
    assert_eq!(g.players[0].library.len(), 9, "one card for one age counter");

    let mut g = pod(2);
    let lash = g.add_card_to_battlefield(0, catalog::thought_lash());
    g.battlefield_find_mut(lash).unwrap().counters.insert(CounterType::Age, 11);
    into_upkeep(&mut g);
    assert!(g.battlefield_find(lash).is_none(), "twelve cards owed, ten in the library");
    assert!(g.players[0].library.is_empty(), "the unpaid trigger exiled the library");
}

/// Mirror of Fate chooses before it exiles: the chosen exiled card ends up
/// as the whole library, and the old library is in exile.
#[test]
fn mirror_of_fate_rebuilds_the_library_from_exile() {
    let mut g = pod(2);
    let mirror = ready(&mut g, 0, catalog::mirror_of_fate());
    let bolt = g.add_card_to_exile(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bolt])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: mirror,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    let lib: Vec<CardId> = g.players[0].library.iter().map(|c| c.id).collect();
    assert_eq!(lib, vec![bolt]);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 10);
}

/// Sea Gate Stormcaller copies the next cheap instant or sorcery once, or
/// twice kicked.
#[test]
fn sea_gate_stormcaller_copies_the_next_cheap_spell() {
    for (kicked, life) in [(false, 14), (true, 11)] {
        let mut g = pod(2);
        let sgs = g.add_card_to_hand(0, catalog::sea_gate_stormcaller());
        flood(&mut g, 0);
        if kicked {
            g.perform_action(GameAction::CastSpellKicked { card_id: sgs, target: None, additional_targets: vec![], mode: None, x_value: None })
                .expect("kick");
            drain_stack(&mut g);
        } else {
            cast(&mut g, sgs, None).expect("cast");
        }
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
        assert_eq!(g.players[1].life, life, "kicked {kicked}");
    }
}

/// Insidious Will's third mode copies a spell.
#[test]
fn insidious_will_copies_an_instant() {
    let mut g = pod(2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let will = g.add_card_to_hand(0, catalog::insidious_will());
    flood(&mut g, 0);
    cast_no_drain(&mut g, bolt, Some(Target::Player(1)), None);
    cast_no_drain(&mut g, will, Some(Target::Permanent(bolt)), Some(2));
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14);
}

/// CR 613.1b — Illusory Gains' steal follows the Aura: a creature entering
/// under an opponent takes the Aura, and the first host goes home.
#[test]
fn illusory_gains_moves_to_the_newest_opposing_creature() {
    let mut g = pod(3);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let gains = g.add_card_to_hand(0, catalog::illusory_gains());
    flood(&mut g, 0);
    cast(&mut g, gains, Some(Target::Permanent(bears))).expect("cast");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0, "stolen");
    let giant = g.add_card_to_hand(2, catalog::hill_giant());
    flood(&mut g, 2);
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    cast(&mut g, giant, None).expect("cast giant");
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 0, "the newcomer is stolen");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 1, "the old host went home");
}

/// Crown of the Ages moves an Aura between creatures: Drake Umbra's +3/+3
/// leaves the first creature for the second.
#[test]
fn crown_of_the_ages_moves_an_aura() {
    let mut g = pod(2);
    let crown = ready(&mut g, 0, catalog::crown_of_the_ages());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let vanguard = ready(&mut g, 0, catalog::elite_vanguard());
    let umbra = g.add_card_to_hand(0, catalog::drake_umbra());
    flood(&mut g, 0);
    cast(&mut g, umbra, Some(Target::Permanent(bears))).expect("cast");
    assert_eq!(g.computed_permanent(bears).unwrap().power, 5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crown,
        ability_index: 0,
        target: Some(Target::Permanent(umbra)),
        additional_targets: vec![Target::Permanent(vanguard)],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 2);
    assert_eq!(g.computed_permanent(vanguard).unwrap().power, 5);
    assert!(g.computed_permanent(vanguard).unwrap().keywords().contains(&crabomination::card::Keyword::Flying));
}

/// CR 702.143 (threshold-style cast restriction) — Proft can't be cast with
/// six cards in your graveyard and can with seven.
#[test]
fn proft_needs_seven_cards_in_your_graveyard() {
    let mut g = pod(2);
    let proft = g.add_card_to_hand(0, catalog::proft_sinister_mastermind());
    for _ in 0..6 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    flood(&mut g, 0);
    assert!(cast(&mut g, proft, None).is_err(), "six cards");
    g.add_card_to_graveyard(0, catalog::island());
    cast(&mut g, proft, None).expect("seven cards");
    assert!(g.battlefield_find(proft).is_some());
}

/// Tyrannical Pitlord's chosen creature gets +3/+3 and flying, and is
/// sacrificed when the Pitlord leaves.
#[test]
fn tyrannical_pitlord_links_the_chosen_creature() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let pitlord = g.add_card_to_hand(0, catalog::tyrannical_pitlord());
    flood(&mut g, 0);
    cast(&mut g, pitlord, None).expect("cast");
    let cp = g.computed_permanent(bears).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));
    let murder = g.add_card_to_hand(0, catalog::murder());
    flood(&mut g, 0);
    cast(&mut g, murder, Some(Target::Permanent(pitlord))).expect("murder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pitlord).is_none());
    assert!(g.battlefield_find(bears).is_none(), "the chosen creature was sacrificed");
}

/// Circle of Affliction drains when a source of the chosen color damages you.
#[test]
fn circle_of_affliction_drains_off_chosen_color_damage() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    let circle = g.add_card_to_hand(0, catalog::circle_of_affliction());
    flood(&mut g, 0);
    cast(&mut g, circle, None).expect("cast");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.players[0].life, 18, "3 damage, then gained 1");
    assert_eq!(g.players[1].life, 19, "drained");
}

/// Grave Strength counts creature cards in your graveyard after its mill.
#[test]
fn grave_strength_counts_creatures_after_milling() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let gs = g.add_card_to_hand(0, catalog::grave_strength());
    flood(&mut g, 0);
    cast(&mut g, gs, Some(Target::Permanent(bears))).expect("cast");
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 4);
}
