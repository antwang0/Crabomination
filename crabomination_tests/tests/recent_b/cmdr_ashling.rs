//! Commander: the Dance of the Elements precon (ECC, Ashling, the
//! Limitless, `decks::cmdr_ashling`).

use crabomination::card::{CardId, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase(seats: usize) -> GameState {
    let mut g = if seats == 2 { two_player_game() } else { multi_player_game(seats) };
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

fn act(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_at(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
}

fn evoke(g: &mut GameState, id: CardId) -> Result<(), String> {
    act(g, GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

fn in_graveyard(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].graveyard.iter().any(|c| c.id == id)
}

/// Advance to `step`, dispatching each step's events the way a priority
/// pass does.
fn to_step(g: &mut GameState, step: TurnStep) {
    for _ in 0..16 {
        if g.step == step {
            return;
        }
        if let Ok(events) = g.advance_step(Vec::new()) {
            g.dispatch_triggers_for_events(&events);
        }
        drain_stack(g);
    }
}

/// CR 702.74 — Ashling gives an Elemental permanent spell cast from hand
/// evoke {4} (a non-Elemental gets none); the evoked Belonging is
/// sacrificed, Ashling mints a hasty token copy (whose ETB makes three more
/// Shapeshifters), and at the next end step the unpaid-for token is
/// sacrificed.
#[test]
fn cr_702_74_ashling_evokes_and_copies_the_sacrificed_elemental() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::ashling_the_limitless());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(evoke(&mut g, bears).is_err(), "a Bear gains no evoke");
    let belonging = g.add_card_to_hand(0, catalog::belonging());
    evoke(&mut g, belonging).expect("evoke {4}");
    assert!(in_graveyard(&g, 0, belonging), "the evoked card is sacrificed");
    let copy = named(&g, 0, "Belonging");
    assert_eq!(copy.len(), 1, "one token copy");
    assert!(g.battlefield_find(copy[0]).unwrap().is_token);
    assert!(g.computed_permanent(copy[0]).unwrap().keywords().contains(&Keyword::Haste));
    assert_eq!(named(&g, 0, "Shapeshifter").len(), 6, "both ETBs made three");
    // The token itself is a token: sacrificing it copies nothing.
    g.players[0].mana_pool.empty();
    to_step(&mut g, TurnStep::End);
    assert!(named(&g, 0, "Belonging").is_empty(), "unpaid, the token is sacrificed");
}

/// CR 702.143 — Haunting Voyage returns up to two creature cards of the
/// chosen type; cast for its foretell cost, all of them.
#[test]
fn cr_702_143_haunting_voyage_foretold_returns_all() {
    for foretold in [false, true] {
        let mut g = main_phase(2);
        let dead: Vec<CardId> = [catalog::jubilation(), catalog::lamentation(), catalog::shimmercreep()]
            .into_iter()
            .map(|c| g.add_card_to_graveyard(0, c))
            .collect();
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let voyage = g.add_card_to_hand(0, catalog::haunting_voyage());
        g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elemental)]));
        if foretold {
            flood(&mut g, 0);
            g.perform_action(GameAction::Foretell { card_id: voyage }).expect("foretell");
            g.foretold_this_turn.clear();
            act(&mut g, GameAction::CastForetold {
                card_id: voyage,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            })
            .expect("cast foretold");
        } else {
            cast_at(&mut g, voyage, None).expect("cast");
        }
        let back = dead.iter().filter(|id| g.battlefield_find(**id).is_some()).count();
        assert_eq!(back, if foretold { 3 } else { 2 }, "foretold: {foretold}");
        assert!(named(&g, 0, "Grizzly Bears").is_empty(), "only the chosen type");
    }
}

/// Cavalier of Thorns puts the land from the top five onto the battlefield
/// and the rest into the graveyard.
#[test]
fn cavalier_of_thorns_reveals_five() {
    let mut g = main_phase(2);
    let forest = g.add_card_to_library(0, catalog::forest());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let cav = g.add_card_to_hand(0, catalog::cavalier_of_thorns());
    cast_at(&mut g, cav, None).expect("cast");
    assert!(g.battlefield_find(forest).is_some());
    assert_eq!(g.players[0].graveyard.len(), 4);
}

/// Vivid counts the colors among your permanents: Shimmercreep drains two
/// with a red and a black permanent, Elemental Spectacle makes three 5/5s
/// (black Shimmercreep, red Soulstoke, green Cavalier).
#[test]
fn vivid_counts_colors_among_your_permanents() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::incandescent_soulstoke());
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    let creep = g.add_card_to_hand(0, catalog::shimmercreep());
    cast_at(&mut g, creep, None).expect("cast");
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 2, theirs - 2));
    g.add_card_to_battlefield(0, catalog::cavalier_of_thorns());
    let spectacle = g.add_card_to_hand(0, catalog::elemental_spectacle());
    let life = g.players[0].life;
    cast_at(&mut g, spectacle, None).expect("cast");
    assert_eq!(named(&g, 0, "Elemental").len(), 3);
    assert_eq!(g.players[0].life, life + 6, "a life per creature you control");
}

/// Slithermuse, evoked, draws up to the fullest opponent's hand as it
/// leaves.
#[test]
fn slithermuse_draws_the_difference() {
    let mut g = main_phase(3);
    for _ in 0..5 {
        g.add_card_to_hand(2, catalog::forest());
        g.add_card_to_library(0, catalog::forest());
    }
    let muse = g.add_card_to_hand(0, catalog::slithermuse());
    evoke(&mut g, muse).expect("evoke");
    assert!(in_graveyard(&g, 0, muse));
    assert_eq!(g.players[0].hand.len(), 5);
}

/// Incandescent Soulstoke puts an Elemental from hand into play with
/// haste (pumped by its anthem) and sacrifices it at the end step.
#[test]
fn incandescent_soulstoke_cheats_an_elemental() {
    let mut g = main_phase(2);
    let stoke = g.add_card_to_battlefield(0, catalog::incandescent_soulstoke());
    g.battlefield.iter_mut().for_each(|c| c.summoning_sick = false);
    let sovereign = g.add_card_to_hand(0, catalog::vernal_sovereign());
    act(&mut g, GameAction::ActivateAbility {
        card_id: stoke,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    let cp = g.computed_permanent(sovereign).expect("in play");
    assert!(cp.keywords().contains(&Keyword::Haste));
    assert_eq!((cp.power, cp.toughness), (5, 5));
    // Vernal Sovereign's own ETB token counts every creature you control.
    let token = named(&g, 0, "Elemental")[0];
    let tp = g.computed_permanent(token).unwrap();
    assert_eq!((tp.power, tp.toughness), (4, 4), "three creatures, +1/+1 from Soulstoke");
    to_step(&mut g, TurnStep::End);
    assert!(in_graveyard(&g, 0, sovereign));
}

/// Horde of Notions casts an Elemental card from your graveyard for free.
#[test]
fn horde_of_notions_recasts_an_elemental() {
    let mut g = main_phase(2);
    let horde = g.add_card_to_battlefield(0, catalog::horde_of_notions());
    let belonging = g.add_card_to_graveyard(0, catalog::belonging());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    act(&mut g, GameAction::ActivateAbility {
        card_id: horde,
        ability_index: 0,
        target: Some(Target::Permanent(belonging)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    assert!(g.battlefield_find(belonging).is_some());
    assert_eq!(named(&g, 0, "Shapeshifter").len(), 3);
}

/// CR 614.1c — "enters tapped" replacements: Primal Beyond enters untapped only
/// with an Elemental card in hand; Timeless Lotus always enters tapped.
#[test]
fn primal_beyond_and_timeless_lotus_enter_tapped() {
    for with_elemental in [false, true] {
        let mut g = main_phase(2);
        if with_elemental {
            g.add_card_to_hand(0, catalog::smokebraider());
        }
        let land = g.add_card_to_hand(0, catalog::primal_beyond());
        g.perform_action(GameAction::PlayLand(land)).expect("play");
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(land).unwrap().tapped, !with_elemental);
    }
    let mut g = main_phase(2);
    let lotus = g.add_card_to_hand(0, catalog::timeless_lotus());
    cast_at(&mut g, lotus, None).expect("cast");
    assert!(g.battlefield_find(lotus).unwrap().tapped);
}
