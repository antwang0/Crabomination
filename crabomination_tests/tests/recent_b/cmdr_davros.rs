//! Commander: the Masters of Evil precon (WHO, Davros, Dalek Creator,
//! `decks::cmdr_davros`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..seats {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: x })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    cast_x(g, id, target, None)
}

fn run(g: &mut GameState, effect: Effect, source: CardId) {
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(source);
    let ev = g.resolve_effect(&effect, &ctx).expect("effect");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// A face-down 2/2 Cyberman artifact creature.
fn is_cyberman(g: &GameState, id: CardId) -> bool {
    let Some(c) = g.battlefield_find(id) else { return false };
    let cp = g.computed_permanent(id).expect("computed");
    c.face_down
        && (cp.power, cp.toughness) == (2, 2)
        && cp.card_types().contains(&CardType::Artifact)
        && cp.card_types().contains(&CardType::Creature)
        && c.definition.subtypes.creature_types.contains(&CreatureType::Cyberman)
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

/// CR 708.2 — Cyber Conversion turns a creature face down as a 2/2 Cyberman
/// artifact creature under its controller.
#[test]
fn cr_708_2_cyber_conversion_makes_a_cyberman() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::cyber_conversion());
    cast(&mut g, spell, Some(Target::Permanent(giant))).expect("Cyber Conversion");
    assert!(is_cyberman(&g, giant));
    assert_eq!(g.battlefield_find(giant).unwrap().controller, 1, "it stays with its controller");
}

/// CR 708.2 — Missy: another nonartifact creature dying returns face down,
/// tapped, under your control as a 2/2 Cyberman; an artifact creature doesn't.
#[test]
fn cr_708_2_missy_converts_the_dead() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::missy());
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![giant]) }, giant);
    assert!(is_cyberman(&g, giant), "the Giant returns as a Cyberman");
    let c = g.battlefield_find(giant).unwrap();
    assert_eq!(c.controller, 0);
    assert!(c.tapped);
    let droid = g.add_card_to_battlefield(1, catalog::clockwork_droid());
    run(&mut g, Effect::Destroy { what: Selector::ExactObjects(vec![droid]) }, droid);
    assert!(g.battlefield_find(droid).is_none(), "an artifact creature stays dead");
}

/// CR 708.2 — Cybership's combat damage puts the top two cards of that
/// player's library onto your battlefield as face-down Cybermen.
#[test]
fn cr_708_2_cybership_harvests_the_top_two() {
    let mut g = main_phase(2);
    let ship = g.add_card_to_battlefield(0, catalog::cybership());
    let before = g.players[1].library.len();
    g.clear_sickness(ship);
    // Treat it as crewed for the attack.
    run(
        &mut g,
        Effect::BecomeCreature {
            what: Selector::ExactObjects(vec![ship]),
            power: crabomination::card::Value::Const(8),
            toughness: crabomination::card::Value::Const(8),
            creature_types: vec![],
            keywords: vec![],
            duration: crabomination::effect::Duration::EndOfTurn,
        },
        ship,
    );
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ship, target: AttackTarget::Player(1) }]))
        .expect("attack");
    for _ in 0..8 {
        if g.step == TurnStep::PostCombatMain {
            break;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].library.len(), before - 2);
    let cybermen = g.battlefield.iter().filter(|c| c.controller == 0 && c.face_down).count();
    assert_eq!(cybermen, 2);
}

/// CR 701.55 — Davros: an opponent who lost 3 life faces the villainous
/// choice; with The Valeyard they face it twice.
#[test]
fn cr_701_55_the_valeyard_doubles_davros_choice() {
    for (valeyard, expected) in [(false, 1), (true, 2)] {
        let mut g = main_phase(2);
        g.add_card_to_battlefield(0, catalog::davros_dalek_creator());
        if valeyard {
            g.add_card_to_battlefield(0, catalog::the_valeyard());
        }
        for _ in 0..4 {
            g.add_card_to_hand(1, catalog::forest());
        }
        let (hand0, hand1) = (g.players[0].hand.len(), g.players[1].hand.len());
        g.players[1].life -= 3;
        g.players[1].life_lost_this_turn = 3;
        end_step(&mut g);
        assert_eq!(named(&g, 0, "Dalek").len(), 1, "an opponent bled 3: a Dalek");
        let drawn = g.players[0].hand.len() - hand0;
        let discarded = hand1 - g.players[1].hand.len();
        assert_eq!(drawn + discarded, expected, "Valeyard {valeyard}");
    }
}

/// CR 702.153 — Ashad: the first nonlegendary artifact spell each turn has
/// casualty 2; a legendary one, or the second, doesn't.
#[test]
fn cr_702_153_ashad_grants_casualty_to_the_first_artifact() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::ashad_the_lone_cyberman());
    let droid = catalog::clockwork_droid();
    assert_eq!(g.casualty_for(0, &droid), Some(2));
    assert_eq!(g.casualty_for(0, &catalog::cult_of_skaro()), None, "legendary");
    assert_eq!(g.casualty_for(0, &catalog::grizzly_bears()), None, "not an artifact");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let first = g.add_card_to_hand(0, catalog::clockwork_droid());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpellCasualty {
        card_id: first,
        sacrifice: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("casualty cast");
    drain_stack(&mut g);
    assert_eq!(named(&g, 0, "Clockwork Droid").len(), 2, "the copy is a token");
    let ashad = named(&g, 0, "Ashad, the Lone Cyberman")[0];
    assert_eq!(g.battlefield_find(ashad).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.casualty_for(0, &droid), None, "only the first each turn");
}

/// CR 707.2 — The Master, Formed Anew: casting it exiles a creature of yours
/// with a takeover counter, and it enters as a copy of that card.
#[test]
fn cr_707_2_the_master_formed_anew_takes_over() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    let master = g.add_card_to_hand(0, catalog::the_master_formed_anew());
    cast(&mut g, master, Some(Target::Permanent(giant))).expect("The Master");
    let exiled = g.exile.iter().find(|c| c.id == giant).expect("the Giant is exiled");
    assert_eq!(exiled.counter_count(CounterType::Takeover), 1);
    let m = g.battlefield_find(master).expect("The Master");
    assert_eq!(m.definition.name, "Hill Giant", "it entered as a copy");
}

/// CR 614.1 — Don't Blink: this turn a creature card entering from exile is
/// shuffled into its owner's library instead.
#[test]
fn cr_614_1_dont_blink_sends_exiled_creatures_home() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    run(&mut g, Effect::Move { what: Selector::ExactObjects(vec![bear]), to: crabomination::effect::ZoneDest::Exile }, bear);
    let spell = g.add_card_to_hand(0, catalog::dont_blink());
    cast(&mut g, spell, None).expect("Don't Blink");
    run(
        &mut g,
        Effect::Move {
            what: Selector::ExactObjects(vec![bear]),
            to: crabomination::effect::ZoneDest::Battlefield {
                controller: crabomination::effect::PlayerRef::Seat(1),
                tapped: false,
            },
        },
        bear,
    );
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.players[1].library.iter().any(|c| c.id == bear), "shuffled into its owner's library");
}

/// CR 608.2 — Dalek Drone destroys a creature and its controller loses 3.
#[test]
fn cr_608_2_dalek_drone_exterminates() {
    let mut g = main_phase(2);
    let giant = g.add_card_to_battlefield(1, catalog::hill_giant());
    let life = g.players[1].life;
    let drone = g.add_card_to_hand(0, catalog::dalek_drone());
    cast(&mut g, drone, None).expect("Dalek Drone");
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(g.players[1].life, life - 3);
}

/// CR 107.3 — Delete deals X to each nonartifact creature and each player.
#[test]
fn cr_107_3_delete_spares_artifact_creatures() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let droid = g.add_card_to_battlefield(1, catalog::cybermat());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let spell = g.add_card_to_hand(0, catalog::delete());
    cast_x(&mut g, spell, None, Some(2)).expect("Delete");
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(droid).is_some(), "an artifact creature is spared");
    assert_eq!((g.players[0].life, g.players[1].life), (l0 - 2, l1 - 2));
}

/// CR 119.3 — Wound Reflection: at each end step each opponent loses the
/// life they lost this turn again.
#[test]
fn cr_119_3_wound_reflection_doubles_the_bleeding() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::wound_reflection());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    cast(&mut g, bolt, Some(Target::Player(1))).expect("bolt");
    end_step(&mut g);
    assert_eq!(g.players[1].life, life - 6);
}

/// CR 122.1 — Genesis of the Daleks: a Dalek per lore counter (1, 2, 3).
#[test]
fn cr_122_1_genesis_of_the_daleks_musters_per_lore_counter() {
    let mut g = main_phase(2);
    let saga = g.add_card_to_hand(0, catalog::genesis_of_the_daleks());
    cast(&mut g, saga, None).expect("Genesis");
    assert_eq!(named(&g, 0, "Dalek").len(), 1);
}
