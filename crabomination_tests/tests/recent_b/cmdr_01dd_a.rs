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

/// CR 614 / 119.5 — Enduring Angel replaces a reduction to 0 or less: the
/// life total becomes 3, never 0, and the Angel transforms into a */*
/// Angelic Enforcer sized by that life.
#[test]
fn cr_614_enduring_angel_catches_a_lethal_loss() {
    let mut g = pod(2);
    let angel = g.add_card_to_battlefield(0, catalog::enduring_angel());
    g.players[0].life = 2;
    assert_eq!(g.adjust_life_applied(0, -5), 1, "2 becomes 3: a gain of the difference");
    assert_eq!(g.players[0].life, 3);
    assert!(!g.players[0].eliminated);
    assert_eq!(g.battlefield_find(angel).unwrap().definition.name, "Angelic Enforcer");
    assert_eq!(g.computed_permanent(angel).unwrap().power, 3);
    // Transformed, the replacement is gone.
    g.adjust_life(0, -1);
    assert_eq!(g.players[0].life, 2);
}

/// CR 104.3 / 614 — Exquisite Archangel exiles itself instead of its
/// controller losing, and the life total resets to the starting total.
#[test]
fn cr_614_exquisite_archangel_resets_to_starting_life() {
    let mut g = pod(2);
    let arch = g.add_card_to_battlefield(0, catalog::exquisite_archangel());
    g.players[0].life = 1;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(0))).expect("bolt");
    assert!(!g.players[0].eliminated);
    assert!(g.battlefield_find(arch).is_none(), "exiled");
    assert_eq!(g.players[0].life, g.players[0].starting_life);
}

/// Oketra's Last Mercy resets your life and locks your lands for one untap.
#[test]
fn oketras_last_mercy_resets_life_and_skips_a_land_untap() {
    let mut g = pod(2);
    g.players[0].life = 4;
    let olm = g.add_card_to_hand(0, catalog::oketras_last_mercy());
    flood(&mut g, 0);
    cast(&mut g, olm, None).expect("cast");
    assert_eq!(g.players[0].life, g.players[0].starting_life);
    assert!(g.players[0].lands_dont_untap_next_untap > 0);
}

/// CR 700.2 — Sol'Kanar's "one that hasn't been chosen" walks its menu: four
/// end steps, four different modes; the last donates it to an opponent as a
/// new object, whose own record starts empty.
#[test]
fn cr_700_2_sol_kanar_walks_its_modes_then_leaves() {
    let mut g = pod(3);
    let sk = g.add_card_to_battlefield(0, catalog::sol_kanar_the_tainted());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    for pick in [0usize, 0, 0] {
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(pick)]));
        g.step = TurnStep::PostCombatMain;
        g.active_player_idx = 0;
        g.priority.player_with_priority = 0;
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(sk).unwrap().modes_chosen.len(), 3);
    assert!(g.battlefield_find(giant).is_none(), "the 3 damage mode");
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    let gone = g.battlefield_find(sk).expect("returned");
    assert_ne!(gone.controller, 0, "an opponent controls it");
    assert!(gone.modes_chosen.is_empty(), "a new object");
}

/// Greed's Gambit pays up front and charges it all back when it leaves.
#[test]
fn greeds_gambit_front_loads_and_back_charges() {
    let mut g = pod(2);
    let gambit = g.add_card_to_hand(0, catalog::greeds_gambit());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, gambit, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 2, "cast one, drew three");
    assert_eq!(g.players[0].life, 26);
    let bats = |g: &GameState| g.battlefield.iter().filter(|c| c.definition.name == "Bat").count();
    assert_eq!(bats(&g), 3);
    let shatter = g.add_card_to_hand(0, catalog::disenchant());
    flood(&mut g, 0);
    cast(&mut g, shatter, Some(Target::Permanent(gambit))).expect("disenchant");
    assert_eq!(g.players[0].life, 20);
    assert_eq!(bats(&g), 0, "three creatures sacrificed");
}

/// Blim hands a permanent to the player it hit; then everyone pays per
/// permanent they control but don't own — the damaged player one.
#[test]
fn blim_gives_a_gift_and_charges_for_it() {
    let mut g = pod(2);
    let blim = ready(&mut g, 0, catalog::blim_comedic_genius());
    let rock = ready(&mut g, 0, catalog::sol_ring());
    g.players[1].hand.clear();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    attack(&mut g, blim, 1);
    let rock = g.battlefield_find(rock).expect("still around");
    assert_eq!(rock.controller, 1);
    assert_eq!(g.players[1].life, 15, "4 combat damage, then 1 for the gift");
    assert_eq!(g.players[1].hand.len(), 1);
}

fn loyalty(g: &mut GameState, pw: CardId, index: usize, target: Option<Target>) {
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: pw, ability_index: index, target, x_value: None })
        .expect("loyalty");
    drain_stack(g);
}

/// CR 606 / 707.10 — Chandra's Regulator pays {1} to copy a Chandra's
/// loyalty ability: Flame's Fury's +1 hits twice.
#[test]
fn chandras_regulator_copies_a_chandra_ability() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::chandras_regulator());
    let fury = ready(&mut g, 0, catalog::chandra_flames_fury());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    loyalty(&mut g, fury, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 16, "2 damage, copied");
}

/// Chandra, Flame's Fury's −2 hits the creature and its controller.
#[test]
fn chandra_flames_fury_minus_two_hits_both() {
    let mut g = pod(3);
    let fury = ready(&mut g, 0, catalog::chandra_flames_fury());
    let giant = ready(&mut g, 2, catalog::hill_giant());
    loyalty(&mut g, fury, 1, Some(Target::Permanent(giant)));
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[2].life, 18);
    assert_eq!(g.players[1].life, 20);
}

/// Chandra, Flame's Catalyst's −8 refills the hand and makes it free for the
/// turn; the flag is gone at the next turn's start.
#[test]
fn chandra_flames_catalyst_ultimate_casts_for_free() {
    let mut g = pod(2);
    let cat = ready(&mut g, 0, catalog::chandra_flames_catalyst());
    g.battlefield_find_mut(cat).unwrap().counters.insert(crabomination::card::CounterType::Loyalty, 8);
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::hill_giant());
    }
    g.players[0].library.rotate_right(7);
    loyalty(&mut g, cat, 2, None);
    assert_eq!(g.players[0].hand.len(), 7);
    let giant = g.players[0].hand[0].id;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: giant,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free");
    drain_stack(&mut g);
    assert!(g.battlefield_find(giant).is_some());
}

/// Way of the Cryomancer empowers a Jace and gives it the copy ability.
#[test]
fn way_of_the_cryomancer_makes_a_jace_with_a_copy_ability() {
    let mut g = pod(2);
    let way = g.add_card_to_hand(0, catalog::way_of_the_cryomancer());
    flood(&mut g, 0);
    cast(&mut g, way, None).expect("cast");
    let jace = g.battlefield.iter().find(|c| c.definition.name.contains("Jace")).expect("a Jace token").id;
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(crabomination::card::CounterType::Loyalty), 5);
    // The granted −3 is the index past Jace's own two.
    loyalty(&mut g, jace, 2, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[1].life, 14, "the bolt was copied");
}

/// CR 603.4 — Phantasmal Mount's two links: killing the Mount sacrifices the
/// creature it boosted.
#[test]
fn phantasmal_mount_links_both_ways() {
    let mut g = pod(2);
    let mount = ready(&mut g, 0, catalog::phantasmal_mount());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mount,
        ability_index: 0,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 3);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Permanent(mount))).expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mount).is_none());
    assert!(g.battlefield_find(bears).is_none(), "sacrificed with the Mount");
}

/// Bill Ferny's second mode: blocked, it gives a Horse to the target
/// opponent, leaves combat, and makes three Treasures.
#[test]
fn bill_ferny_trades_a_horse_for_treasure() {
    let mut g = pod(2);
    let bill = ready(&mut g, 0, catalog::bill_ferny_bree_swindler());
    let horse = ready(&mut g, 0, catalog::phantasmal_mount());
    let wall = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bill,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(wall, bill)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(horse).unwrap().controller, 1);
    assert_eq!(treasures(&g, 0), 3);
    assert!(g.battlefield_find(bill).is_some(), "removed from combat, not killed");
}

/// Selective Snare returns X creatures sharing a creature type.
#[test]
fn selective_snare_bounces_x_of_one_type() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::grizzly_bears());
    let snare = g.add_card_to_hand(0, catalog::selective_snare());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: snare,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == a) && g.players[1].hand.iter().any(|c| c.id == b));
}

/// Wanderwine Farewell's tokens count what it returned, and only with a
/// Merfolk on your side once the returns are done.
#[test]
fn wanderwine_farewell_counts_returned_permanents() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::judge_of_currents());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::sol_ring());
    let wf = g.add_card_to_hand(0, catalog::wanderwine_farewell());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: wf,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.iter().filter(|c| c.id == a || c.id == b).count(), 2);
    let tokens = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Merfolk").count();
    assert_eq!(tokens, 2);
}

/// Vivid — Shinestriker draws one card per color among your permanents (it
/// counts itself: green, red and blue is three), and Wildvine Pummeler costs
/// {1} less per color.
#[test]
fn vivid_counts_colors_among_your_permanents() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::hill_giant());
    let ss = g.add_card_to_hand(0, catalog::shinestriker());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    cast(&mut g, ss, None).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
    let wp = g.add_card_to_hand(0, catalog::wildvine_pummeler());
    g.players[0].mana_pool.empty();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, wp, None).expect("{3}{G} with three colors");
    assert!(g.battlefield_find(wp).is_some());
}

/// Panglacial Wurm (2006-07-15 rulings) — cast from the library while a
/// search resolves, before anything is found, paying its own cost; the
/// search then finishes and the Wurm resolves.
#[test]
fn panglacial_wurm_is_cast_from_the_library_mid_search() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let wurm = g.add_card_to_library(0, catalog::panglacial_wurm());
    let forest = g.add_card_to_library(0, catalog::forest());
    let growth = g.add_card_to_hand(0, catalog::rampant_growth());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Search(Some(forest))]));
    cast(&mut g, growth, None).expect("cast");
    assert!(g.battlefield_find(wurm).is_some(), "cast and resolved");
    assert!(g.battlefield_find(forest).is_some(), "the search still found the Forest");
}

/// CR 701.19 — every search resolver offers Panglacial Wurm, not only the
/// generic one: Gaea's Balance's per-type search (`SearchEachBasicLandType`).
#[test]
fn panglacial_wurm_is_offered_by_every_search_resolver() {
    use crabomination::effect::{Effect, PlayerRef};
    use crabomination::game::effects::EffectContext;
    let mut g = pod(2);
    g.players[0].library.clear();
    let wurm = g.add_card_to_library(0, catalog::panglacial_wurm());
    let forest = g.add_card_to_library(0, catalog::forest());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let search = Effect::SearchEachBasicLandType { who: PlayerRef::You, tapped: false };
    g.resolve_effect(&search, &EffectContext::for_ability(src, 0, None)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(wurm).is_some(), "offered, cast and resolved");
    assert!(g.battlefield_find(forest).is_some(), "the search still found the Forest");
    assert!(g.players[0].searched_library_this_turn);
}

/// CR 614 — Elderscale Wurm holds a life total of 7 or more at 7 against
/// damage, and raises a lower one to 7 as it enters.
#[test]
fn elderscale_wurm_floors_damage_at_seven() {
    let mut g = pod(2);
    g.players[0].life = 3;
    let wurm = g.add_card_to_hand(0, catalog::elderscale_wurm());
    flood(&mut g, 0);
    cast(&mut g, wurm, None).expect("cast");
    assert_eq!(g.players[0].life, 7);
    g.players[0].life = 9;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    cast(&mut g, bolt, Some(Target::Player(0))).expect("bolt");
    assert_eq!(g.players[0].life, 7, "9 - 3 stops at 7");
}

/// Delirium — Soul Swallower grows at your upkeep with four card types in
/// your graveyard.
#[test]
fn soul_swallower_grows_with_delirium() {
    let mut g = pod(2);
    let ss = g.add_card_to_battlefield(0, catalog::soul_swallower());
    for c in [catalog::forest(), catalog::lightning_bolt(), catalog::grizzly_bears(), catalog::sol_ring()] {
        g.add_card_to_graveyard(0, c);
    }
    into_upkeep(&mut g);
    assert_eq!(g.battlefield_find(ss).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 3);
}
