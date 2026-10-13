//! Commander: Cynette's, Elda's and Ekthi's last gaps (`decks::cmdr_edhrec37`).

use crabomination::card::{CardId, CounterType, Keyword};
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
    g
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn named(g: &GameState, seat: usize, name: &str) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).map(|c| c.id).collect()
}

/// Naban doubles a Wizard's enter trigger: Cynette entering makes two
/// Jellyfish, each 2/2 under Cynette's flier anthem.
#[test]
fn naban_doubles_cynettes_jellyfish() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::naban_dean_of_iteration());
    let cynette = g.add_card_to_hand(0, catalog::cynette_jelly_drover());
    cast(&mut g, cynette, None).expect("cynette");
    let jellies = named(&g, 0, "Jellyfish");
    assert_eq!(jellies.len(), 2);
    assert_eq!(g.computed_permanent(jellies[0]).unwrap().power, 2);
}

/// Elda gives the first nonlegendary creature spell each turn demonstrate:
/// you and a chosen opponent each get a token copy.
#[test]
fn elda_demonstrates_the_first_creature_spell() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::elda_conjurer_of_spectacle());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "the bear and its copy");
    assert_eq!(named(&g, 1, "Grizzly Bears").len(), 1, "the opponent's copy");
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, second, None).expect("second bear");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 3, "only the first each turn");
}

/// Fungus Sliver: a Sliver that survives damage grows; Reflex Sliver gives
/// every Sliver haste, an opponent's included.
#[test]
fn slivers_share_growth_and_haste() {
    let mut g = pod(2);
    let big = g.add_card_to_battlefield(0, catalog::fungus_sliver());
    g.add_card_to_battlefield(1, catalog::reflex_sliver());
    assert!(g.computed_permanent(big).unwrap().keywords().contains(&Keyword::Haste));
    g.battlefield_find_mut(big).unwrap().counters.insert(CounterType::PlusOnePlusOne, 2);
    let shock = g.add_card_to_hand(0, catalog::shock());
    cast(&mut g, shock, Some(Target::Permanent(big))).expect("shock the 4/4");
    assert_eq!(g.battlefield_find(big).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Ekthi gives each Equipment living weapon (CR 702.92a); Swordsman's Steel
/// scales with your Equipment count.
#[test]
fn cr_702_92a_ekthi_arms_every_equipment() {
    let mut g = pod(2);
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::ekthi_contaminator_priest());
    let lances = g.add_card_to_hand(0, catalog::long_lost_lances());
    cast(&mut g, lances, None).expect("lances");
    let germ = named(&g, 0, "Phyrexian Germ");
    assert_eq!(germ.len(), 1);
    assert_eq!(g.battlefield_find(lances).unwrap().attached_to, Some(germ[0]));
    let cp = g.computed_permanent(germ[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (0 + 2 + 1, 0 + 1), "+2/+0 Lances, +1/+1 Ekthi");
    assert!(cp.keywords().contains(&Keyword::FirstStrike), "your turn");
    let steel = g.add_card_to_hand(0, catalog::swordsmans_steel());
    let hand = g.players[0].hand.len();
    cast(&mut g, steel, None).expect("steel");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "two Equipment: two cards");
    let germs = named(&g, 0, "Phyrexian Germ");
    let steel_germ = g.battlefield_find(steel).unwrap().attached_to.expect("attached");
    assert!(germs.contains(&steel_germ));
    assert_eq!(g.computed_permanent(steel_germ).unwrap().power, 4 + 1);
}
