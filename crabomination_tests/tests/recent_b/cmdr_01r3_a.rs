//! Commander: the 2026-10-21 (`01R3xh4r`) one-gap pair closers
//! (`decks::cmdr_01r3_a`).

use crabomination::card::{CardId, CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, PlayerRef, Selector, Value};
use crabomination::game::effects::EffectContext;
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// CR 702.160 + 707.12 — Arcane Proxy's ETB reads its own power: cast as a
/// 2/1 prototype it can't take a mana value 3 sorcery, cast as a 4/3 it exiles
/// Divination and casts a copy (two cards drawn), the card staying in exile.
#[test]
fn cr_702_160_arcane_proxy_target_is_capped_by_its_power() {
    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let proxy = g.add_card_to_hand(0, catalog::arcane_proxy());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastPrototype {
        card_id: proxy,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("prototype cast");
    let hand = g.players[0].hand.len();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "a 2/1 can't reach mana value 3");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == div));

    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let proxy = g.add_card_to_hand(0, catalog::arcane_proxy());
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, proxy, None);
    assert_eq!(g.players[0].hand.len(), hand + 2, "the copy of Divination resolved");
    assert!(g.exile.iter().any(|c| c.id == div), "the card stays exiled");
}

/// Worldpurge: every permanent goes to its owner's hand, each seat keeps at
/// most seven (APNAP, CR 101.4) and shuffles the rest in, and the caster's
/// floating mana is gone (an effect, not the step-end empty — CR 500.4).
#[test]
fn worldpurge_bounces_then_trims_each_hand_to_seven() {
    let mut g = pod(3);
    let purge = g.add_card_to_hand(0, catalog::worldpurge());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..8 {
        g.add_card_to_hand(1, catalog::island());
    }
    g.add_card_to_hand(2, catalog::island());
    let lib1 = g.players[1].library.len();
    cast(&mut g, purge, None);
    assert!(g.battlefield.is_empty());
    assert_eq!(g.players[1].hand.len(), 7, "nine in hand, seven kept");
    assert_eq!(g.players[1].library.len(), lib1 + 2);
    assert!(g.players[1].hand.iter().any(|c| c.id == bears), "the headless seat keeps its spell");
    assert_eq!(g.players[2].hand.len(), 1, "a short hand keeps everything");
    assert_eq!(g.players[0].mana_pool.total(), 0);
}

fn activate(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) -> Result<Vec<GameEvent>, GameError> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target, additional_targets: vec![], x_value: None, mode: None })
}

/// CR 611.2c — Braided Net's lock lasts "for as long as it remains tapped":
/// the tapped Shivan Dragon can't pump, and once it untaps the lock is gone.
/// The net spends a counter per use.
#[test]
fn cr_611_2c_braided_net_locks_only_while_the_target_stays_tapped() {
    let mut g = pod(3);
    let net = g.add_card_to_battlefield(0, catalog::braided_net());
    g.battlefield_find_mut(net).unwrap().add_counters(CounterType::Net, 3);
    let dragon = g.add_card_to_battlefield(2, catalog::shivan_dragon());
    activate(&mut g, 0, net, Some(Target::Permanent(dragon))).expect("net");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).unwrap().tapped);
    assert_eq!(g.battlefield_find(net).unwrap().counter_count(CounterType::Net), 2);
    assert!(activate(&mut g, 2, dragon, None).is_err(), "locked while tapped");
    g.battlefield_find_mut(dragon).unwrap().tapped = false;
    g.check_state_based_actions();
    activate(&mut g, 2, dragon, None).expect("untapped, the lock has ended");
}

/// CR 702.26 — Disciple of Caelus Nin: each seat keeps five permanents
/// (turn order from the caster, CR 101.4), the rest phase out and stay out
/// while it is on the battlefield; once it leaves they phase in on their
/// controller's untap step.
#[test]
fn cr_702_26_disciple_of_caelus_nin_holds_the_board_phased_out() {
    let mut g = pod(3);
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..7 {
        g.add_card_to_battlefield(2, catalog::island());
    }
    let disciple = g.add_card_to_hand(0, catalog::disciple_of_caelus_nin());
    cast(&mut g, disciple, None);
    let count = |g: &GameState, seat| g.battlefield.iter().filter(|c| c.controller == seat).count();
    assert_eq!((count(&g, 0), count(&g, 1), count(&g, 2)), (6, 1, 5), "five kept and the Disciple");
    assert_eq!(g.phased_out.len(), 3);
    g.active_player_idx = 2;
    g.do_phasing();
    assert_eq!(g.phased_out.len(), 3, "permanents can't phase in");
    let pos = g.battlefield.iter().position(|c| c.id == disciple).unwrap();
    g.battlefield.remove(pos);
    g.do_phasing();
    assert_eq!(count(&g, 2), 7, "the lock is gone");
}

/// CR 702.29c — Deem Worthy cycled: the "may" trigger deals 2 to a creature.
#[test]
fn cr_702_29c_deem_worthy_cycling_deals_two() {
    let mut g = pod(2);
    let ogre = g.add_card_to_battlefield(1, catalog::hill_giant());
    let deem = g.add_card_to_hand(0, catalog::deem_worthy());
    flood(&mut g, 0);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::Cycle { card_id: deem, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ogre).unwrap().damage, 2);
}

/// Dire Wolves bands only while its controller has a Plains (CR 702.22).
#[test]
fn cr_702_22_dire_wolves_band_with_a_plains() {
    let mut g = pod(2);
    let wolves = g.add_card_to_battlefield(0, catalog::dire_wolves());
    let bands = |g: &GameState| g.computed_permanent(wolves).unwrap().keywords().contains(&Keyword::Banding);
    assert!(!bands(&g));
    g.add_card_to_battlefield(1, catalog::plains());
    assert!(!bands(&g), "an opponent's Plains doesn't count");
    g.add_card_to_battlefield(0, catalog::plains());
    assert!(bands(&g));
}

/// Geistflame Reservoir banks a charge counter per instant or sorcery cast,
/// then spends any number of them as damage (the X removed, CR 107.3).
#[test]
fn geistflame_reservoir_banks_spells_and_fires_x() {
    let mut g = pod(2);
    let res = g.add_card_to_battlefield(0, catalog::geistflame_reservoir());
    for _ in 0..3 {
        let opt = g.add_card_to_hand(0, catalog::opt());
        cast(&mut g, opt, None);
    }
    assert_eq!(g.battlefield_find(res).unwrap().counter_count(CounterType::Charge), 3);
    let life = g.players[1].life;
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: res,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("fire two");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.battlefield_find(res).unwrap().counter_count(CounterType::Charge), 1);
}

/// Invasion of Arcavios fetches an instant or sorcery from the graveyard as
/// well as the library (CR 903.11: nothing waits outside the game).
#[test]
fn invasion_of_arcavios_finds_a_graveyard_sorcery() {
    let mut g = pod(2);
    let div = g.add_card_to_graveyard(0, catalog::divination());
    let inv = g.add_card_to_hand(0, catalog::invasion_of_arcavios());
    cast(&mut g, inv, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == div));
}

/// Fungal Behemoth's P/T counts +1/+1 counters on every creature its
/// controller has — not an opponent's.
#[test]
fn fungal_behemoth_counts_counters_on_your_creatures() {
    let mut g = pod(2);
    let fb = g.add_card_to_battlefield(0, catalog::fungal_behemoth());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(theirs).unwrap().add_counters(CounterType::PlusOnePlusOne, 5);
    g.battlefield_find_mut(fb).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let cp = g.computed_permanent(fb).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "three counters, plus its own +1/+1");
}

/// Feral Encounter exiles a creature from the top five face up, castable this
/// turn; the other four go to the bottom.
#[test]
fn feral_encounter_exiles_a_creature_castable_this_turn() {
    let mut g = multi_player_game(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let fe = g.add_card_to_hand(0, catalog::feral_encounter());
    cast(&mut g, fe, None);
    let ex = g.exile.iter().find(|c| c.id == bears).expect("the creature is exiled");
    assert!(!ex.face_down && ex.may_play_until.is_some());
    assert_eq!(g.players[0].library.len(), 7);
}

/// CR 706.6 — Vedalken Squirrel-Whacker enters as the two rolls (5/6), and a
/// d6 its controller later rolls as a 1 is exchanged with the larger base stat:
/// the roll reads 6 and the creature becomes 5/1.
#[test]
fn cr_706_6_squirrel_whacker_rolls_its_body_and_trades_a_low_roll() {
    let mut g = pod(2);
    let whacker = g.add_card_to_hand(0, catalog::vedalken_squirrel_whacker());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(5), DecisionAnswer::DieRoll(6)]));
    cast(&mut g, whacker, None);
    let cp = g.computed_permanent(whacker).expect("a */* survives its as-enters roll");
    assert_eq!((cp.power, cp.toughness), (5, 6));
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1)]));
    let roll = Effect::RollDie {
        sides: 6,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results: vec![(6, 6, Effect::Draw { who: Selector::You, amount: Value::ONE })],
        on_doubles: None,
    };
    g.resolve_effect(&roll, &EffectContext::for_ability(whacker, 0, None)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "the 1 became a 6");
    let cp = g.computed_permanent(whacker).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 1));
}

/// CR 614.1a — Soul Echo: once the opponent chooses, damage to its controller
/// removes echo counters instead, until that player's next turn; life at 0
/// doesn't lose the game.
#[test]
fn cr_614_1a_soul_echo_turns_damage_into_counters_until_your_next_upkeep() {
    let mut g = pod(3);
    let echo = g.add_card_to_hand(0, catalog::soul_echo());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: echo, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast X=3");
    drain_stack(&mut g);
    let count = |g: &GameState| g.battlefield_find(echo).unwrap().counter_count(CounterType::Echo);
    assert_eq!(count(&g), 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let stamp = Effect::PlayerMayStampSourceTurn { who: Selector::Player(PlayerRef::Seat(1)) };
    g.resolve_effect(&stamp, &EffectContext::for_ability(echo, 0, None)).unwrap();
    let hit = |g: &mut GameState, n| {
        let dmg = Effect::DealDamage { to: Selector::Player(PlayerRef::Seat(0)), amount: Value::Const(n) };
        g.resolve_effect(&dmg, &EffectContext::for_ability(echo, 1, None)).unwrap();
    };
    let life = g.players[0].life;
    hit(&mut g, 2);
    assert_eq!((g.players[0].life, count(&g)), (life, 1));
    g.turn_number += 1;
    g.active_player_idx = 1;
    hit(&mut g, 5);
    assert_eq!((g.players[0].life, count(&g)), (life, 0), "still replaced, with nothing left to remove");
    g.turn_number += 1;
    g.active_player_idx = 0;
    hit(&mut g, life);
    assert_eq!(g.players[0].life, 0, "the controller's own turn ends the replacement");
    g.check_state_based_actions();
    assert!(g.players[0].is_alive(), "0 life doesn't lose");
}

/// CR 707.9b — crewed by a Shivan Dragon, Mindlink Mech becomes a 4/3 flying
/// Shivan Dragon that is a Vehicle artifact too, with the Dragon's firebreathing;
/// the copy ends at end of turn. A second crewing that turn doesn't trigger.
#[test]
fn cr_707_9b_mindlink_mech_copies_its_crew_as_a_four_three() {
    let mut g = pod(2);
    let mech = g.add_card_to_battlefield(0, catalog::mindlink_mech());
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    g.battlefield_find_mut(dragon).unwrap().summoning_sick = false;
    g.perform_action(GameAction::Crew { vehicle: mech, crew_creatures: vec![dragon] }).expect("crew");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mech).unwrap();
    assert_eq!(cp.def.name, "Shivan Dragon");
    assert_eq!((cp.power, cp.toughness), (4, 3));
    assert!(cp.def.is_artifact() && cp.keywords().contains(&Keyword::Flying));
    assert!(!cp.def.activated_abilities.is_empty(), "it has the Dragon's pump");
}

/// Encrust reaches an artifact as well as a creature: an enchanted Mind
/// Stone's activated abilities can't be activated (CR 602.5c).
#[test]
fn cr_602_5c_encrust_locks_an_artifact() {
    let mut g = pod(2);
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    let aura = g.add_card_to_hand(0, catalog::encrust());
    cast(&mut g, aura, Some(Target::Permanent(stone)));
    assert!(g.computed_permanent(stone).unwrap().keywords().contains(&Keyword::CantActivateAbilities));
}

/// Lifecraft Awakening animates a noncreature artifact as a 0/0 Construct
/// carrying the X counters; an artifact creature only gets the counters.
#[test]
fn lifecraft_awakening_animates_only_a_noncreature_artifact() {
    let mut g = pod(2);
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::lifecraft_awakening());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(stone)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast X=2");
    drain_stack(&mut g);
    let cp = g.computed_permanent(stone).expect("a 2/2, not a dead 0/0");
    assert!(cp.card_types().contains(&CardType::Creature) && cp.card_types().contains(&CardType::Artifact));
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// CR 702.173 — Monastery Raid exiles two, or X when its freerunning cost was
/// paid; the exiled cards stay playable.
#[test]
fn cr_702_173_monastery_raid_freerunning_exiles_x() {
    let mut g = pod(2);
    let raid = g.add_card_to_hand(0, catalog::monastery_raid());
    cast(&mut g, raid, None);
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 2);

    let mut g = pod(2);
    g.players[0].freerunning_unlocked_this_turn = true;
    let raid = g.add_card_to_hand(0, catalog::monastery_raid());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: raid,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(4),
    })
    .expect("freerunning X=4");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 4);
}

/// Slayer's Plate makes a Spirit only when its creature was a Human: Elite
/// Vanguard dying leaves one; Grizzly Bears dying leaves none.
#[test]
fn slayers_plate_spirit_needs_a_human() {
    for (def, spirits) in [(catalog::elite_vanguard(), 1), (catalog::grizzly_bears(), 0)] {
        let mut g = pod(2);
        let plate = g.add_card_to_battlefield(0, catalog::slayers_plate());
        let body = g.add_card_to_battlefield(0, def);
        g.battlefield_find_mut(plate).unwrap().attached_to = Some(body);
        let bolt = g.add_card_to_hand(0, catalog::murder());
        cast(&mut g, bolt, Some(Target::Permanent(body)));
        assert!(g.battlefield_find(body).is_none());
        let made = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
        assert_eq!(made, spirits);
    }
}

/// CR 614.9 — Kor Dirge sends the chosen source's damage to the second target
/// all turn: two hits on the Bears land on the Hill Giant.
#[test]
fn cr_614_9_kor_dirge_redirects_a_chosen_source_all_turn() {
    let mut g = pod(2);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // The headless chooser names the first hostile source.
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let dirge = g.add_card_to_hand(0, catalog::kor_dirge());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: dirge,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![Target::Permanent(giant)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    for _ in 0..2 {
        let hit = Effect::DealDamage { to: Selector::Target(0), amount: Value::ONE };
        g.resolve_effect(&hit, &EffectContext::for_ability(dragon, 1, Some(Target::Permanent(bears)))).unwrap();
    }
    assert_eq!(g.battlefield_find(bears).unwrap().damage, 0);
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 2);
}

/// Braids's Frightful Return chapter III: an opponent with nothing to
/// sacrifice loses 2 life and its controller draws (CR 714.2b — the chapter
/// triggers as the third lore counter lands).
#[test]
fn cr_714_2b_braidss_frightful_return_chapter_three_punishes() {
    let mut g = pod(2);
    let saga = g.add_card_to_battlefield(0, catalog::braidss_frightful_return());
    g.battlefield_find_mut(saga).unwrap().add_counters(CounterType::Lore, 2);
    let (life, hand) = (g.players[1].life, g.players[0].hand.len());
    let lore = Effect::AddCounter { what: Selector::This, kind: CounterType::Lore, amount: Value::ONE };
    g.resolve_effect(&lore, &EffectContext::for_ability(saga, 0, None)).unwrap();
    drain_stack(&mut g);
    assert_eq!((g.players[1].life, g.players[0].hand.len()), (life - 2, hand + 1));
}

/// Jester's Scepter exiles five face down, then counters a spell that shares a
/// name with one of them, binning that card.
#[test]
fn jesters_scepter_counters_a_spell_named_like_its_exile() {
    let mut g = pod(2);
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::lightning_bolt());
    }
    g.players[1].library.rotate_left(10);
    let scepter = g.add_card_to_hand(0, catalog::jesters_scepter());
    cast(&mut g, scepter, Some(Target::Player(1)));
    let exiled: Vec<_> = g.exile.iter().filter(|c| c.exiled_with == Some(scepter)).collect();
    assert_eq!(exiled.len(), 5);
    assert!(exiled.iter().all(|c| c.face_down && c.definition.name == "Lightning Bolt"));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let life = g.players[0].life;
    activate(&mut g, 0, scepter, Some(Target::Permanent(bolt))).expect("scepter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "the Bolt was countered");
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(scepter)).count(), 4, "one exiled Bolt binned");
}

/// Visions of Duplicity swaps two creatures neither of which its caster
/// controls (CR 701.12) — a three-seat swap between two opponents.
#[test]
fn cr_701_12_visions_of_duplicity_swaps_two_opponents_creatures() {
    let mut g = pod(3);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(2, catalog::hill_giant());
    let vis = g.add_card_to_hand(0, catalog::visions_of_duplicity());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: vis,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().controller, 2);
    assert_eq!(g.battlefield_find(b).unwrap().controller, 1);
}

/// CR 712 — Elusive Tormentor transforms for {1} and a discard into the
/// unblockable, indestructible Insidious Mist.
#[test]
fn cr_712_elusive_tormentor_becomes_insidious_mist() {
    let mut g = pod(2);
    let vamp = g.add_card_to_battlefield(0, catalog::elusive_tormentor());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, 0, vamp, None).expect("transform");
    drain_stack(&mut g);
    let cp = g.computed_permanent(vamp).unwrap();
    assert_eq!(cp.def.name, "Insidious Mist");
    assert!(cp.keywords().contains(&Keyword::Indestructible) && cp.keywords().contains(&Keyword::Unblockable));
}

/// CR 702.167a/c — Eye of Ojer Taq crafts with two graveyard sorceries; Apex
/// Observatory enters tapped naming Sorcery, and its grant casts the next
/// sorcery for nothing (spent by that cast).
#[test]
fn cr_702_167_eye_of_ojer_taq_crafts_on_a_shared_type() {
    let mut g = pod(2);
    let eye = g.add_card_to_battlefield(0, catalog::eye_of_ojer_taq());
    g.add_card_to_graveyard(0, catalog::divination());
    g.add_card_to_graveyard(0, catalog::divination());
    g.add_card_to_graveyard(0, catalog::opt());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility { card_id: eye, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("craft");
    drain_stack(&mut g);
    let apex = g.battlefield_find(eye).expect("returned transformed");
    assert_eq!(apex.definition.name, "Apex Observatory");
    assert!(apex.tapped, "it enters tapped");
    assert_eq!(apex.chosen_card_type, Some(CardType::Sorcery), "Opt's Instant is not shared");
    assert_eq!(g.players[0].graveyard.len(), 1, "two Divinations exiled, Opt kept");
    g.battlefield_find_mut(eye).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility { card_id: eye, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("grant");
    drain_stack(&mut g);
    g.players[0].mana_pool.empty();
    let div = g.add_card_to_hand(0, catalog::divination());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell { card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "cast for free and resolved");
    assert!(g.players[0].free_next_spells.is_empty(), "the grant is spent");
}

/// CR 706.6 — Monitor Monitor pays {1} to reroll a low die once a turn: the 1
/// becomes a 6; a second low roll that turn stands.
#[test]
fn cr_706_6_monitor_monitor_rerolls_once_a_turn() {
    let mut g = pod(2);
    let monitor = g.add_card_to_battlefield(0, catalog::monitor_monitor());
    g.players[0].mana_pool.add_colorless(2);
    let roll = Effect::RollDie {
        sides: 6,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results: vec![(6, 6, Effect::Draw { who: Selector::You, amount: Value::ONE })],
        on_doubles: None,
    };
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DieRoll(1), DecisionAnswer::DieRoll(6), DecisionAnswer::DieRoll(1)]));
    g.resolve_effect(&roll, &EffectContext::for_ability(monitor, 0, None)).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "rerolled into a 6");
    assert_eq!(g.players[0].mana_pool.total(), 1, "{{1}} paid");
    g.resolve_effect(&roll, &EffectContext::for_ability(monitor, 0, None)).unwrap();
    assert_eq!(g.players[0].mana_pool.total(), 1, "once each turn");
}

/// Command Performance's ticket mode: {TK}{TK} (CR 123.2); the Attraction
/// mode opens nothing from an empty Attraction deck.
#[test]
fn cr_123_2_command_performance_grants_two_tickets() {
    let mut g = pod(2);
    let cp = g.add_card_to_hand(0, catalog::command_performance());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellSpree { card_id: cp, spree_modes: vec![0, 2], target: None, additional_targets: vec![], x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].tickets, 2);
}

/// Feral Encounter's combat half: your creature deals damage equal to its
/// power to a creature you don't control — two targets, two slots (it read
/// one slot through both filters and could never resolve).
#[test]
fn feral_encounter_deals_damage_at_combat() {
    let mut g = pod(2);
    let ogre = g.add_card_to_battlefield(0, catalog::hill_giant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let Effect::Seq(steps) = catalog::feral_encounter().effect else { panic!() };
    let crabomination::effect::Effect::DelayUntil { body, .. } = &steps[1] else { panic!() };
    let mut ctx = EffectContext::for_spell(0, Some(Target::Permanent(ogre)), 0, 0);
    ctx.targets = vec![Target::Permanent(ogre), Target::Permanent(bear)];
    g.resolve_effect(body, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "3 damage kills the Bear");
}
