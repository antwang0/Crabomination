//! Commander: EDHREC average-deck gaps, sixth batch (`decks::cmdr_edhrec6`):
//! Hylda of the Icy Crown's tappers, and the skip-next-untap rider they share.

use crabomination::card::{CardId, CounterType, CreatureType};
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

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], mode: Option<usize>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn tapped(g: &GameState, id: CardId) -> bool {
    g.battlefield_find(id).expect("on the battlefield").tapped
}

fn controller(g: &GameState, id: CardId) -> usize {
    g.battlefield_find(id).expect("on the battlefield").controller
}

/// Run `seat`'s untap step.
fn untap_step(g: &mut GameState, seat: usize) {
    g.active_player_idx = seat;
    g.do_untap();
}

/// `seat` casts a Grizzly Bears in its own main phase.
fn cast_on_own_turn(g: &mut GameState, seat: usize) -> CardId {
    let id = g.add_card_to_hand(seat, catalog::grizzly_bears());
    flood(g, seat);
    g.active_player_idx = seat;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
    g.active_player_idx = 0;
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

/// Hylda's Crown of Winter: the tap costs {1} less on your turn ({0}), full
/// price on another's; the sacrifice draws per tapped opposing creature.
#[test]
fn hyldas_crown_is_free_on_your_turn_and_counts_tapped_creatures() {
    let mut g = pod(3);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let crown = ready(&mut g, 0, catalog::hyldas_crown_of_winter());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::grizzly_bears());
    activate(&mut g, crown, 0, Some(perm(a))).expect("free on your turn");
    assert!(tapped(&g, a));
    g.battlefield_find_mut(crown).unwrap().tapped = false;
    g.active_player_idx = 1;
    assert!(activate(&mut g, crown, 0, Some(perm(b))).is_err(), "{{1}} on another's turn, and no mana");
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, crown, 0, Some(perm(b))).expect("one paid");
    flood(&mut g, 0);
    activate(&mut g, crown, 1, None).expect("sacrifice");
    assert_eq!(g.players[0].hand.len(), 2, "two tapped creatures across two opponents");
}

/// Willbreaker (CR 611.2c): targeting an opponent's creature with your own
/// ability steals it; your own creature or an opponent's targeting doesn't;
/// losing Willbreaker hands the creature back.
#[test]
fn willbreaker_steals_what_you_target_while_you_control_it() {
    let mut g = pod(3);
    let wb = ready(&mut g, 0, catalog::willbreaker());
    let crown = ready(&mut g, 0, catalog::hyldas_crown_of_winter());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 2, catalog::hill_giant());
    activate(&mut g, crown, 0, Some(perm(mine))).expect("tap mine");
    assert_eq!(controller(&g, mine), 0);
    g.battlefield_find_mut(crown).unwrap().tapped = false;
    activate(&mut g, crown, 0, Some(perm(theirs))).expect("tap theirs");
    assert_eq!(controller(&g, theirs), 0, "stolen as it became the target");
    g.battlefield_find_mut(wb).unwrap().controller = 1;
    g.check_state_based_actions();
    assert_eq!(controller(&g, theirs), 2, "back once you don't control Willbreaker");
}

/// Illithid Harvester (CR 708.2): tapped nontoken creatures turn face down as
/// 2/2 Horrors; a double-faced one can't be turned face down (CR 712.16).
#[test]
fn cr_712_16_illithid_harvester_skips_a_double_faced_creature() {
    let mut g = pod(2);
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let delver = ready(&mut g, 1, catalog::delver_of_secrets());
    for id in [giant, delver] {
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    let h = g.add_card_to_hand(0, catalog::illithid_harvester());
    cast(&mut g, h, &[perm(giant), perm(delver)]).expect("cast");
    let c = g.computed_permanent(giant).unwrap();
    assert!(g.battlefield_find(giant).unwrap().face_down);
    assert_eq!((c.power, c.toughness), (2, 2));
    assert!(c.subtypes().creature_types.contains(&CreatureType::Horror));
    assert!(!g.battlefield_find(delver).unwrap().face_down, "CR 712.16");
}

/// Plant Tadpoles: exactly X targets, tapped and held through their next untap.
#[test]
fn plant_tadpoles_taps_x_creatures_and_holds_them() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 1, catalog::hill_giant());
    let h = g.add_card_to_hand(0, catalog::illithid_harvester());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastAdventure {
        card_id: h,
        target: Some(perm(a)),
        additional_targets: vec![perm(b)],
        x_value: Some(2),
        mode: None,
    })
    .expect("cast Plant Tadpoles");
    drain_stack(&mut g);
    assert!(tapped(&g, a) && tapped(&g, b));
    untap_step(&mut g, 1);
    assert!(tapped(&g, a) && tapped(&g, b));
}

/// Cone of Cold (CR 706): a 20 also makes opposing creatures enter tapped
/// until your next turn (CR 614.1c); a 5 only taps.
#[test]
fn cone_of_cold_twenty_holds_the_door_until_your_next_turn() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let cone = g.add_card_to_hand(0, catalog::cone_of_cold());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(20)]));
    cast(&mut g, cone, &[]).expect("cast");
    assert!(tapped(&g, a));
    let mine = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, mine, &[]).expect("cast mine");
    assert!(!tapped(&g, mine), "yours doesn't enter tapped");
    let late = cast_on_own_turn(&mut g, 2);
    assert!(tapped(&g, late), "an opponent's creature does");
    untap_step(&mut g, 1);
    assert!(tapped(&g, a), "and the 20 holds the tapped ones too");
    untap_step(&mut g, 0);
    let after = cast_on_own_turn(&mut g, 2);
    assert!(!tapped(&g, after), "over once your turn begins");

    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let cone = g.add_card_to_hand(0, catalog::cone_of_cold());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5)]));
    cast(&mut g, cone, &[]).expect("cast");
    assert!(tapped(&g, a));
    untap_step(&mut g, 1);
    assert!(!tapped(&g, a), "a 5 doesn't hold");
}

/// Topplegeist: delirium taps a creature of the opponent whose upkeep it is.
#[test]
fn topplegeist_taps_on_each_opponents_upkeep_with_delirium() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::topplegeist());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!tapped(&g, a), "no delirium yet");
    for def in [catalog::island(), catalog::grizzly_bears(), catalog::shock(), catalog::opt()] {
        g.add_card_to_graveyard(0, def);
    }
    g.add_card_to_graveyard(0, catalog::hyldas_crown_of_winter());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(tapped(&g, a));
}

/// Hammers of Moradin (CR 601.2c): attacking at three seats it taps one
/// creature of each opponent, and myriad copies it at the other one.
#[test]
fn hammers_of_moradin_taps_one_per_opponent() {
    let mut g = pod(3);
    let h = ready(&mut g, 0, catalog::hammers_of_moradin());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: h, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(tapped(&g, a) && tapped(&g, b));
    let copies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Hammers of Moradin").count();
    assert_eq!(copies, 1, "myriad: a copy attacking seat 2");
}

/// Icy Blast: ferocious holds the tapped creatures; without it they untap.
#[test]
fn icy_blast_holds_only_with_ferocious() {
    for (ferocious, held) in [(false, false), (true, true)] {
        let mut g = pod(2);
        if ferocious {
            ready(&mut g, 0, catalog::illithid_harvester());
        }
        let a = ready(&mut g, 1, catalog::grizzly_bears());
        let blast = g.add_card_to_hand(0, catalog::icy_blast());
        cast_x(&mut g, blast, &[perm(a)], None, Some(1)).expect("cast");
        untap_step(&mut g, 1);
        assert_eq!(tapped(&g, a), held, "ferocious {ferocious}");
    }
}

/// Time of Ice: I locks a creature while you control the Saga; III bounces
/// every tapped creature.
#[test]
fn time_of_ice_locks_then_bounces_tapped_creatures() {
    let mut g = pod(2);
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let saga = g.add_card_to_hand(0, catalog::time_of_ice());
    cast(&mut g, saga, &[perm(a)]).expect("cast");
    assert!(tapped(&g, a));
    untap_step(&mut g, 1);
    assert!(tapped(&g, a), "locked while you control the Saga");
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    for _ in 0..2 {
        g.saga_advance(saga);
        drain_stack(&mut g);
        g.check_state_based_actions();
    }
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(mine).is_none());
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[0].hand.len(), 1);
}

// ── Mr. House, President and CEO ────────────────────────────────────────────

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

fn rolls(faces: &[u8]) -> Vec<DecisionAnswer> {
    faces.iter().map(|&f| DecisionAnswer::DieRoll(f)).collect()
}

fn attack_with(g: &mut GameState, attacks: &[(CardId, usize)]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn treasures(g: &GameState, seat: usize) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == "Treasure").count()
}

/// CR 706.2 — The Deck of Many Things subtracts your hand size: a natural 2
/// with two cards in hand is a 0, which discards the hand; a natural 20 with
/// none reanimates, and that creature's death costs its owner the game.
#[test]
fn cr_706_2_the_deck_of_many_things_reads_the_modified_result() {
    let mut g = pod(3);
    let deck = ready(&mut g, 0, catalog::the_deck_of_many_things());
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    script(&mut g, rolls(&[2]));
    flood(&mut g, 0);
    activate(&mut g, deck, 0, None).expect("roll");
    assert!(g.players[0].hand.is_empty(), "a 0 discards the hand");
    let giant = g.add_card_to_graveyard(2, catalog::hill_giant());
    g.battlefield_find_mut(deck).unwrap().tapped = false;
    script(&mut g, rolls(&[20]));
    flood(&mut g, 0);
    activate(&mut g, deck, 0, None).expect("roll");
    assert_eq!(controller(&g, giant), 0, "a 20 reanimates under your control");
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, &[perm(giant)]).expect("murder");
    g.check_state_based_actions();
    assert!(!g.players[2].is_alive(), "its owner loses the game");
    assert!(g.players[0].is_alive());
}

/// CR 706.2 — Night Shift: a 9 on Hoarding Ogre's d20 is moved to 10 for 1
/// life (two Treasures); the shift is once each turn.
#[test]
fn cr_706_2_night_shift_moves_a_result_once_each_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::night_shift_of_the_living_dead());
    let ogre = ready(&mut g, 0, catalog::hoarding_ogre());
    let life = g.players[0].life;
    let before = treasures(&g, 0);
    script(&mut g, rolls(&[9]));
    attack_with(&mut g, &[(ogre, 1)]);
    assert_eq!(treasures(&g, 0) - before, 2, "9 -> 10: two Treasures");
    assert!(g.players[0].life < life, "paid 1 life");
    let ogre3 = ready(&mut g, 0, catalog::hoarding_ogre());
    let before = treasures(&g, 0);
    script(&mut g, rolls(&[9]));
    attack_with(&mut g, &[(ogre3, 1)]);
    assert_eq!(treasures(&g, 0) - before, 1, "only once each turn");
}

/// CR 706.6 — Iron Mastiff rolls one d20 per attacked player and keeps the
/// highest: a 3 and a 15 is a 15, so the defending player takes 4.
#[test]
fn cr_706_6_iron_mastiff_keeps_the_highest_roll() {
    let mut g = pod(3);
    let dog = ready(&mut g, 0, catalog::iron_mastiff());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let (me, them) = (g.players[0].life, g.players[1].life);
    script(&mut g, rolls(&[3, 15]));
    attack_with(&mut g, &[(dog, 1), (bear, 2)]);
    assert_eq!(g.players[1].life, them - 4, "15: the defending player");
    assert_eq!(g.players[0].life, me, "the 3 was ignored");
}

/// Circuits Act — 1, 4, 4 is two different results: two Clown Robots
/// (2022-10-07 ruling).
#[test]
fn circuits_act_counts_different_results() {
    let mut g = pod(2);
    let act = g.add_card_to_hand(0, catalog::circuits_act());
    script(&mut g, rolls(&[1, 4, 4]));
    cast(&mut g, act, &[]).expect("cast");
    let robots = g.battlefield.iter().filter(|c| c.definition.name == "Clown Robot").count();
    assert_eq!(robots, 2);
}

/// CR 706.5 — Celebr-8000: doubles add double strike to each face's ability.
#[test]
fn cr_706_5_celebr_8000_doubles_grant_double_strike() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let c = ready(&mut g, 0, catalog::celebr_8000());
    script(&mut g, rolls(&[3, 3]));
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let kw = g.computed_permanent(c).unwrap().keywords().to_vec();
    assert!(kw.contains(&Keyword::Vigilance) && kw.contains(&Keyword::DoubleStrike));
}

/// CR 606.3 — Comet's 6 is +1 loyalty and two more activations this turn; a
/// 4 then deals his loyalty to a creature it kills, chosen without
/// targeting, and takes −2.
#[test]
fn cr_606_3_comet_rolls_for_more_activations_and_chosen_damage() {
    let mut g = pod(2);
    let comet = ready(&mut g, 0, catalog::comet_stellar_pup());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let loyal = |g: &GameState| g.battlefield_find(comet).map_or(0, |c| c.counter_count(CounterType::Loyalty));
    let use_comet = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: comet, ability_index: 0, target: None, x_value: None })
            .map(|_| drain_stack(g))
            .map_err(|e| format!("{e:?}"))
    };
    script(&mut g, rolls(&[6]));
    use_comet(&mut g).expect("first");
    assert_eq!(loyal(&g), 6);
    script(&mut g, rolls(&[4]));
    use_comet(&mut g).expect("a second activation");
    assert!(g.battlefield_find(wurm).is_none(), "6 damage kills the 6/4 Wurm");
    assert_eq!(loyal(&g), 4);
    script(&mut g, rolls(&[1]));
    use_comet(&mut g).expect("and a third");
    assert!(use_comet(&mut g).is_err(), "but not a fourth");
}

/// CR 706.6 — Wyll rolls an extra die and ignores the lowest (2 and 15 is a
/// 15: two Treasures off Hoarding Ogre), and grows once per roll.
#[test]
fn cr_706_6_wyll_rolls_an_extra_die_and_ignores_the_lowest() {
    let mut g = pod(2);
    let wyll = ready(&mut g, 0, catalog::wyll_blade_of_frontiers());
    let ogre = ready(&mut g, 0, catalog::hoarding_ogre());
    script(&mut g, rolls(&[2, 15]));
    attack_with(&mut g, &[(ogre, 1)]);
    assert_eq!(treasures(&g, 0), 2);
    assert_eq!(g.battlefield_find(wyll).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Contraband Livestock — the exiled creature's controller gets the token.
#[test]
fn contraband_livestock_pays_the_creatures_controller() {
    let mut g = pod(3);
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::contraband_livestock());
    script(&mut g, rolls(&[5]));
    cast(&mut g, spell, &[perm(bear)]).expect("cast");
    assert!(g.battlefield_find(bear).is_none());
    let ox = g.battlefield.iter().find(|c| c.definition.name == "Ox").expect("an Ox");
    assert_eq!(ox.controller, 2);
}

/// Myrkul's Edict's 20: each opponent sacrifices their greatest power.
#[test]
fn myrkuls_edict_twenty_takes_the_biggest() {
    let mut g = pod(3);
    let small = ready(&mut g, 1, catalog::grizzly_bears());
    let big = ready(&mut g, 1, catalog::craw_wurm());
    let other = ready(&mut g, 2, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::myrkuls_edict());
    script(&mut g, rolls(&[20]));
    cast(&mut g, spell, &[]).expect("cast");
    assert!(g.battlefield_find(big).is_none() && g.battlefield_find(small).is_some());
    assert!(g.battlefield_find(other).is_none());
}

/// Wand of Wonder: each opponent exiles to an instant or sorcery and shuffles
/// the rest back; on a 5 you cast one of them free (Opt draws you a card).
#[test]
fn wand_of_wonder_casts_an_opponents_spell_free() {
    let mut g = pod(2);
    let wand = ready(&mut g, 0, catalog::wand_of_wonder());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    for def in [catalog::island(), catalog::opt(), catalog::island(), catalog::island()] {
        g.add_card_to_library(1, def);
    }
    let lib = g.players[1].library.len();
    script(&mut g, vec![DecisionAnswer::DieRoll(5), DecisionAnswer::Bool(true)]);
    flood(&mut g, 0);
    activate(&mut g, wand, 0, None).expect("roll");
    assert_eq!(g.players[1].library.len(), lib - 1, "only Opt left the library");
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Opt"), "cast, then to its owner's graveyard");
    assert_eq!(g.players[0].hand.len(), 1, "Opt drew for its caster");
}

/// Thunderwave's 10-19 spares the creature you choose.
#[test]
fn thunderwave_spares_the_chosen_creature() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::thunderwave());
    script(&mut g, vec![DecisionAnswer::DieRoll(12), DecisionAnswer::Cards(vec![mine])]);
    cast(&mut g, spell, &[]).expect("cast");
    g.check_state_based_actions();
    assert!(g.battlefield_find(mine).is_some() && g.battlefield_find(theirs).is_none());
}

/// Wyll's Reversal adds your greatest power: a natural 13 with a 2-power
/// creature is a 15, so the Bolt is redirected and copied — both at its
/// caster (the headless retarget turns it on an opponent).
#[test]
fn wylls_reversal_copies_on_fifteen_or_more() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    let rev = g.add_card_to_hand(0, catalog::wylls_reversal());
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    script(&mut g, rolls(&[13]));
    cast(&mut g, rev, &[Target::Permanent(bolt)]).expect("reverse");
    assert_eq!((g.players[0].life, g.players[1].life), (mine, theirs - 6), "the Bolt and its copy");
}
