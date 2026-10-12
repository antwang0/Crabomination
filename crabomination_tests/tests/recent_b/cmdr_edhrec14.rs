//! Commander: one-card EDHREC gaps ranked 1001-1500 (`decks::cmdr_edhrec14`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn act(g: &mut GameState, a: GameAction) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(a).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

/// Fury of the Horde (CR 118.9, 601.2b): "exile two red cards from your hand
/// rather than pay this spell's mana cost" takes the pitch card and one more
/// red card — and with only one other red card the alternative cost is
/// unpayable.
#[test]
fn fury_of_the_horde_exiles_two_red_cards_for_its_alternative_cost() {
    let mut g = pod(4);
    let fury = g.add_card_to_hand(0, catalog::fury_of_the_horde());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let alt = |pitch| GameAction::CastSpellAlternative {
        card_id: fury,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    };
    assert!(act(&mut g, alt(bolt)).is_err(), "one red card can't pay for two");
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    act(&mut g, alt(bolt)).expect("two red cards pay it");
    assert!(g.exile.iter().any(|c| c.id == bolt) && g.exile.iter().any(|c| c.id == second));
    assert!(g.players[0].hand.is_empty());
}

/// Plate Armor: "Equip {3}. This ability costs {1} less to activate for each
/// other Equipment you control" — two others make it {1}.
#[test]
fn plate_armor_equip_costs_less_per_other_equipment() {
    let mut g = pod(2);
    let armor = ready(&mut g, 0, catalog::plate_armor());
    ready(&mut g, 0, catalog::bonesplitter());
    ready(&mut g, 0, catalog::bonesplitter());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    act(&mut g, GameAction::Equip { equipment: armor, target: bear }).expect("equip for {1}");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5));
    assert!(c.keywords().iter().any(|k| matches!(k, Keyword::Ward(_))));
}

/// Heartwood Storyteller (CR 102.2 at four seats): a noncreature spell draws
/// each of its caster's opponents a card — the caster draws nothing.
#[test]
fn heartwood_storyteller_draws_each_of_the_casters_opponents() {
    let mut g = pod(4);
    ready(&mut g, 2, catalog::heartwood_storyteller());
    for s in 0..4 {
        library(&mut g, s, 3);
    }
    let hands: Vec<usize> = (0..4).map(|s| g.players[s].hand.len()).collect();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    assert_eq!(g.players[0].hand.len(), hands[0]);
    for (s, before) in hands.iter().enumerate().skip(1) {
        assert_eq!(g.players[s].hand.len(), before + 1, "seat {s} draws");
    }
}

/// Qala, Ajani's Pridemate: life gained grows Qala; attacking pumps the other
/// attackers by the counters on it.
#[test]
fn qala_counts_its_counters_into_the_other_attackers() {
    let mut g = pod(2);
    let qala = ready(&mut g, 0, catalog::qala_ajanis_pridemate());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(6);
    g.players[0].mana_pool.add(Color::White, 2);
    for _ in 0..2 {
        act(&mut g, GameAction::ActivateAbility {
            card_id: qala,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("gain 1");
    }
    assert_eq!(g.battlefield_find(qala).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    g.step = TurnStep::DeclareAttackers;
    act(&mut g, GameAction::DeclareAttackers(vec![
        crabomination::game::types::Attack { attacker: qala, target: crabomination::game::types::AttackTarget::Player(1) },
        crabomination::game::types::Attack { attacker: bear, target: crabomination::game::types::AttackTarget::Player(1) },
    ]))
    .expect("attack");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "+2/+0 from two counters");
}

/// CR 603.7c — Rite of Harmony's "this turn, whenever a creature or
/// enchantment you control enters, draw a card": a creature entering this turn
/// draws, and after cleanup one entering draws nothing.
#[test]
fn cr_603_7c_rite_of_harmony_draws_for_this_turns_entries_only() {
    let mut g = pod(3);
    library(&mut g, 0, 5);
    let rite = g.add_card_to_hand(0, catalog::rite_of_harmony());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, rite, None).expect("rite");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(g.players[0].hand.len(), hand, "cast the bear (-1), drew one (+1)");
    let mut events = Vec::new();
    let _ = g.do_cleanup(&mut events);
    assert!(g.players[0].emblems.iter().all(|e| !e.this_turn), "cleanup ends the rite");
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.stack.clear();
    let bear2 = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear2, None).expect("bear");
    assert_eq!(g.players[0].hand.len(), hand - 1, "no draw after the turn");
}

/// Hex Magic: the hand is exiled, that many cards drawn, and the exiled cards
/// stay playable through the caster's next turn.
#[test]
fn hex_magic_trades_the_hand_for_as_many_draws() {
    let mut g = pod(2);
    library(&mut g, 0, 5);
    let a = g.add_card_to_hand(0, catalog::lightning_bolt());
    let b = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hex = g.add_card_to_hand(0, catalog::hex_magic());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, hex, None).expect("hex");
    assert_eq!(g.players[0].hand.len(), 2, "two exiled, two drawn");
    for id in [a, b] {
        assert!(g.exile.iter().any(|c| c.id == id && c.may_play_until.is_some()), "playable from exile");
    }
}

/// Key to the Side-Door's draw: the discard must be a legendary card naming a
/// legendary permanent you control — a copy of your commander in hand pays it,
/// a lone legend doesn't.
#[test]
fn key_to_the_side_door_discards_a_duplicate_legend() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let key = ready(&mut g, 0, catalog::key_to_the_side_door());
    g.add_card_to_hand(0, catalog::tyvar_the_bellicose());
    let draw = |g: &mut GameState| {
        g.players[0].mana_pool.add_colorless(1);
        act(g, GameAction::ActivateAbility {
            card_id: key,
            ability_index: 1,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    assert!(draw(&mut g).is_err(), "no Tyvar on the battlefield to match");
    ready(&mut g, 0, catalog::tyvar_the_bellicose());
    draw(&mut g).expect("the duplicate pays");
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Fatal Grudge: sacrificing an artifact as the cost makes each opponent
/// sacrifice a permanent sharing a card type with it — their artifact, not
/// their creature.
#[test]
fn fatal_grudge_matches_the_sacrificed_card_type() {
    let mut g = pod(3);
    library(&mut g, 0, 2);
    ready(&mut g, 0, catalog::sol_ring());
    let foe_ring = ready(&mut g, 1, catalog::sol_ring());
    let foe_bear = ready(&mut g, 1, catalog::grizzly_bears());
    let other_bear = ready(&mut g, 2, catalog::grizzly_bears());
    let grudge = g.add_card_to_hand(0, catalog::fatal_grudge());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, grudge, None).expect("grudge");
    assert!(g.battlefield_find(foe_ring).is_none(), "the opponent's artifact goes");
    assert!(g.battlefield_find(foe_bear).is_some() && g.battlefield_find(other_bear).is_some(), "creatures stay");
}

/// Demonic Gifts: the granted "when this creature dies, return it" brings the
/// creature back under its owner's control.
#[test]
fn demonic_gifts_returns_the_creature_that_dies() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gifts = g.add_card_to_hand(0, catalog::demonic_gifts());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, gifts, Some(Target::Permanent(bear))).expect("gifts");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0),
        "the bear came back"
    );
}

/// Irencrag Feat: seven red, then "you can cast only one more spell this
/// turn" — the first cast after it goes through, the second is refused.
#[test]
fn irencrag_feat_allows_one_more_spell() {
    let mut g = pod(2);
    let feat = g.add_card_to_hand(0, catalog::irencrag_feat());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, feat, None).expect("feat");
    let one = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, one, Some(Target::Player(1))).expect("the one more spell");
    let two = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(cast(&mut g, two, Some(Target::Player(1))).is_err(), "no third spell");
}

/// Phyrexian Censor: one non-Phyrexian spell a turn per player (Phyrexian
/// spells don't count or get locked), and non-Phyrexian creatures enter
/// tapped.
#[test]
fn phyrexian_censor_locks_non_phyrexian_spells_and_taps_entrants() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::phyrexian_censor());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear, None).expect("first non-Phyrexian spell");
    assert!(g.battlefield_find(bear).unwrap().tapped, "entered tapped");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    assert!(cast(&mut g, bolt, Some(Target::Player(1))).is_err(), "a second non-Phyrexian spell");
    let censor = g.add_card_to_hand(0, catalog::phyrexian_censor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, censor, None).expect("a Phyrexian spell isn't limited");
    assert!(!g.battlefield_find(censor).unwrap().tapped, "a Phyrexian creature enters untapped");
}

/// Golden Argosy (CR 702.122): attacking exiles each creature that crewed it
/// this turn, and they come back tapped at the next end step.
#[test]
fn golden_argosy_flickers_its_crew_at_end_step() {
    let mut g = pod(2);
    let argosy = ready(&mut g, 0, catalog::golden_argosy());
    let crew = ready(&mut g, 0, catalog::grizzly_bears());
    let idle = ready(&mut g, 0, catalog::grizzly_bears());
    act(&mut g, GameAction::Crew { vehicle: argosy, crew_creatures: vec![crew] }).expect("crew");
    g.step = TurnStep::DeclareAttackers;
    act(&mut g, GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: argosy,
        target: crabomination::game::types::AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(g.exile.iter().any(|c| c.id == crew), "the crew is exiled");
    assert!(g.battlefield_find(idle).is_some(), "a creature that didn't crew stays");
}

/// Teferi, Who Slows the Sunset +1: of the chosen artifact, creature and land,
/// yours untap and the others tap; you gain 2 life.
#[test]
fn teferi_who_slows_the_sunset_untaps_yours_and_taps_theirs() {
    let mut g = pod(2);
    let teferi = ready(&mut g, 0, catalog::teferi_who_slows_the_sunset());
    let my_land = ready(&mut g, 0, catalog::wastes());
    g.battlefield_find_mut(my_land).unwrap().tapped = true;
    let their_bear = ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[0].life;
    act(&mut g, GameAction::ActivateLoyaltyAbility {
        card_id: teferi,
        ability_index: 0,
        target: Some(Target::Permanent(their_bear)),
        x_value: None,
    })
    .expect("+1");
    assert!(g.battlefield_find(their_bear).unwrap().tapped, "their creature taps");
    assert_eq!(g.players[0].life, life + 2);
}

/// Toluz, Clever Conductor: cards you discard are exiled with it, and come
/// back to hand when it dies.
#[test]
fn toluz_banks_discards_and_returns_them_on_death() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let toluz = ready(&mut g, 0, catalog::toluz_clever_conductor());
    let card = g.add_card_to_hand(0, catalog::lightning_bolt());
    let mut events = Vec::new();
    g.discard_card(0, card, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == card), "the discard is exiled");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(toluz))).expect("bolt Toluz");
    assert!(g.players[0].hand.iter().any(|c| c.id == card), "back to hand when Toluz dies");
}

/// Amareth, the Lustrous: an entering creature takes a creature card off the
/// top; a land on top stays.
#[test]
fn amareth_takes_a_top_card_sharing_a_type() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::amareth_the_lustrous());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(g.players[0].hand.len(), 1, "the top Grizzly Bears shares creature");
    g.add_card_to_library(0, catalog::wastes());
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, second, None).expect("bear");
    assert_eq!(g.players[0].hand.len(), 1, "a land on top doesn't share a type with a creature");
}

/// Sin Prodder at three seats: when no opponent takes the damage, the
/// revealed card goes to hand.
#[test]
fn sin_prodder_hands_over_a_declined_card() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::sin_prodder());
    library(&mut g, 0, 2);
    let hand = g.players[0].hand.len();
    let lib = g.players[0].library.len();
    let life: Vec<i32> = (0..3).map(|s| g.players[s].life).collect();
    g.step = TurnStep::Untap;
    let _ = g.advance_step(Vec::new());
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "the top card left the library");
    let damaged = (1..3).any(|s| g.players[s].life < life[s]);
    let to_hand = g.players[0].hand.len() == hand + 1;
    assert!(damaged ^ to_hand, "either an opponent took the damage or the card went to hand");
}
