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

/// "The first nonlegendary creature spell you cast each turn" counts the spells
/// cast before Elda arrived: entering after one, she gives the next none.
#[test]
fn elda_skips_the_turn_when_a_creature_spell_came_first() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let first = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, first, None).expect("bear before Elda");
    g.add_card_to_battlefield(0, catalog::elda_conjurer_of_spectacle());
    let second = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, second, None).expect("second bear");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "no demonstrate copy");
    assert_eq!(named(&g, 1, "Grizzly Bears").len(), 0);
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
    assert_eq!((cp.power, cp.toughness), (3, 1), "a 0/0 Germ: +2/+0 Lances, +1/+1 Ekthi");
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

/// CR 111.1 / 707.10f — a demonstrated copy of a permanent spell becomes a
/// token as it resolves; it isn't "created", so a token doubler (Exalted
/// Sunborn) doesn't double it.
#[test]
fn cr_707_10f_a_resolved_spell_copy_is_not_a_created_token() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::elda_conjurer_of_spectacle());
    g.add_card_to_battlefield(0, catalog::exalted_sunborn());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None).expect("bear");
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2, "the bear and one copy");
}

/// CR 701.32 / 614.13 — populating a token doubler under three doublers
/// would add eight more; the default pick copies the other token instead.
#[test]
fn cr_701_32_populate_skips_a_runaway_doubler() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::parallel_lives());
    g.add_card_to_battlefield(0, catalog::parallel_lives());
    let sunborn = g.add_card_to_battlefield(0, catalog::exalted_sunborn());
    g.battlefield_find_mut(sunborn).unwrap().is_token = true;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().is_token = true;
    let rites = g.add_card_to_hand(0, catalog::rootborn_defenses());
    cast(&mut g, rites, None).expect("populate");
    assert_eq!(named(&g, 0, "Exalted Sunborn").len(), 1);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 1 + 8, "three doublings");
}

/// CR 701.32 — a token that populates (Nesting Dovehawk) copied each
/// combat doubles a turn; at ten copies the default populates the other
/// token instead (seed 922560 reached the action cap).
#[test]
fn cr_701_32_populate_skips_a_runaway_populater() {
    let mut g = pod(2);
    for _ in 0..10 {
        let hawk = g.add_card_to_battlefield(0, catalog::nesting_dovehawk());
        let c = g.battlefield_find_mut(hawk).unwrap();
        c.is_token = true;
        c.add_counters(CounterType::PlusOnePlusOne, 5);
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().is_token = true;
    let rites = g.add_card_to_hand(0, catalog::rootborn_defenses());
    cast(&mut g, rites, None).expect("populate");
    assert_eq!(named(&g, 0, "Nesting Dovehawk").len(), 10);
    assert_eq!(named(&g, 0, "Grizzly Bears").len(), 2);
}

/// CR 614.13 — a bot holds Second Harvest ("for each token you control, a
/// copy") when its tokens under its doublers would overflow the board: the
/// gate priced a `ForEach` spell at 0, and an Elda pod's Exalted Sunborn
/// tokens board-capped (seed 99300254). With two tokens it casts it.
#[test]
fn cr_614_13_bot_holds_a_board_bound_second_harvest() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let casts = |sunborns: usize| {
        let mut g = pod(2);
        stock_libraries(&mut g, 10);
        for _ in 0..sunborns {
            let s = g.add_card_to_battlefield(0, catalog::exalted_sunborn());
            g.battlefield_find_mut(s).unwrap().is_token = true;
        }
        for _ in 0..6 {
            g.add_card_to_battlefield(0, catalog::forest());
        }
        let harvest = g.add_card_to_hand(0, catalog::second_harvest());
        // An instant: the bot casts it in an opponent's end step.
        g.active_player_idx = 1;
        g.step = TurnStep::End;
        g.priority.player_with_priority = 0;
        let a = HeuristicBot::new().next_action(&g, 0);
        matches!(a, Some(GameAction::CastSpell { card_id, .. }) if card_id == harvest)
    };
    assert!(casts(2), "two tokens under two doublers: eight copies");
    assert!(!casts(9), "nine under nine: 4,608 copies");
}

/// CR 614.13 / 111.1 — Second Harvest makes its copies in one event: two
/// Exalted Sunborn tokens under their own two doublings become eight more,
/// not a doubling that compounds copy by copy (one resolution once minted
/// 766 Sunborns).
#[test]
fn cr_614_13_for_each_copies_share_one_doubling() {
    let mut g = pod(2);
    for _ in 0..2 {
        let s = g.add_card_to_battlefield(0, catalog::exalted_sunborn());
        g.battlefield_find_mut(s).unwrap().is_token = true;
    }
    let harvest = g.add_card_to_hand(0, catalog::second_harvest());
    cast(&mut g, harvest, None).expect("harvest");
    assert_eq!(named(&g, 0, "Exalted Sunborn").len(), 2 + 2 * 4);
}
