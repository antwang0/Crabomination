//! Commander: the Scions & Spellcraft precon (FIC, Y'shtola,
//! `decks::cmdr_yshtola`).

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    act(g, GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
}

fn cast_at(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn stock(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn tokens(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.is_token && c.definition.name == name).count()
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on the battlefield");
    (c.power, c.toughness)
}

fn has(g: &GameState, id: CardId, k: &Keyword) -> bool {
    g.computed_permanent(id).expect("on the battlefield").keywords().contains(k)
}

/// A cheap noncreature spell for the cast triggers.
fn opt(g: &mut GameState) {
    let s = g.add_card_to_hand(0, catalog::sol_ring());
    cast_at(g, s, &[]).expect("a noncreature spell");
}

/// Alisaie discounts the second spell; Alphinaud draws off it.
#[test]
fn the_leveilleurs_dualcast() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 3);
    g.add_card_to_battlefield(0, catalog::alphinaud_leveilleur());
    opt(&mut g);
    let hand = g.players[0].hand.len();
    opt(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "the second spell drew");
}

/// Ardbert grows your legends on white and black spells.
#[test]
fn ardbert_rallies_legends() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(0, catalog::ardbert_warrior_of_darkness());
    let t = g.add_card_to_hand(0, catalog::tataru_taru());
    g.add_card_to_library(0, catalog::island());
    cast_at(&mut g, t, &[Target::Player(1)]).expect("a white spell");
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(has(&g, a, &Keyword::Vigilance));
}

/// Job select: the Cane enters attached to a fresh Hero, a Wizard.
#[test]
fn blue_mages_cane_job_selects() {
    let mut g = main_phase(2);
    let c = g.add_card_to_hand(0, catalog::blue_mages_cane());
    cast_at(&mut g, c, &[]).expect("cast");
    let hero = g.battlefield.iter().find(|x| x.definition.name == "Hero").unwrap().id;
    assert_eq!(pt(&g, hero), (1, 3));
    assert!(g.computed_permanent(hero).unwrap().subtypes().creature_types.contains(&CreatureType::Wizard));
}

/// Champions from Beyond makes X Heroes.
#[test]
fn champions_from_beyond_assemble() {
    let mut g = main_phase(2);
    let c = g.add_card_to_hand(0, catalog::champions_from_beyond());
    cast_x(&mut g, c, &[], Some(3)).expect("cast");
    assert_eq!(tokens(&g, 0, "Hero"), 3);
}

/// Dancer's Chakrams: +2/+2 and lifelink on its Hero.
#[test]
fn dancers_chakrams_dance() {
    let mut g = main_phase(2);
    let c = g.add_card_to_hand(0, catalog::dancers_chakrams());
    cast_at(&mut g, c, &[]).expect("cast");
    let hero = g.battlefield.iter().find(|x| x.definition.name == "Hero").unwrap().id;
    assert_eq!(pt(&g, hero), (3, 3));
    assert!(has(&g, hero, &Keyword::Lifelink));
}

/// Dancer's Chakrams pumps *other* commanders: a commander wearing it gets
/// the equip bonus once, not the commander pump on top.
#[test]
fn dancers_chakrams_pumps_other_commanders() {
    let mut g = main_phase(2);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].commanders.push(a);
    g.players[0].commanders.push(b);
    let c = g.add_card_to_hand(0, catalog::dancers_chakrams());
    cast_at(&mut g, c, &[]).expect("cast");
    assert_eq!((pt(&g, a), pt(&g, b)), ((4, 4), (4, 4)), "on the Hero, both commanders are other");
    act(&mut g, GameAction::Equip { equipment: c, target: a }).expect("equip {3}");
    assert_eq!(pt(&g, a), (4, 4), "the wearer takes the equip bonus only");
    assert_eq!(pt(&g, b), (4, 4));
}

/// Estinien grows and flies on noncreature spells.
#[test]
fn estinien_takes_flight() {
    let mut g = main_phase(2);
    let e = g.add_card_to_battlefield(0, catalog::estinien_varlineau());
    opt(&mut g);
    assert_eq!(pt(&g, e), (4, 4));
    assert!(has(&g, e, &Keyword::Flying));
}

/// Estinien's second-main draw counts the opponents it or a Dragon dealt
/// combat damage — two of four here (a Bear hit the third in combat, the
/// Dragon the fourth outside it).
#[test]
fn estinien_counts_opponents_it_or_a_dragon_hit() {
    let mut g = main_phase(5);
    let e = g.add_card_to_battlefield(0, catalog::estinien_varlineau());
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for (seat, by) in [(1, e), (2, dragon), (3, bear)] {
        g.players[seat].creatures_that_damaged_me_this_turn.push(by);
        g.players[seat].creatures_that_combat_damaged_me_this_turn.push(by);
    }
    // A Dragon's NONcombat damage to a fourth seat doesn't count.
    g.players[4].creatures_that_damaged_me_this_turn.push(dragon);
    stock(&mut g, 0, 5);
    let (lib, life) = (g.players[0].library.len(), g.players[0].life);
    g.step = TurnStep::PostCombatMain;
    g.fire_step_triggers(TurnStep::PostCombatMain);
    drain_stack(&mut g);
    assert_eq!(lib - g.players[0].library.len(), 2);
    assert_eq!(life - g.players[0].life, 2);
}

/// Eye of Nidhogg makes a black 4/2 flying deathtouch Dragon.
#[test]
fn eye_of_nidhogg_dragonizes() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::eye_of_nidhogg());
    cast_at(&mut g, e, &[Target::Permanent(bear)]).expect("cast");
    assert_eq!(pt(&g, bear), (4, 2));
    assert!(has(&g, bear, &Keyword::Flying) && has(&g, bear, &Keyword::Deathtouch));
}

/// Fandaniel: an opponent without a creature pays life per spell card.
#[test]
fn fandaniel_punishes() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::fandaniel_telophoroi_ascian());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let life = g.players[1].life;
    for _ in 0..8 {
        if g.step == TurnStep::End {
            break;
        }
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
}

/// G'raha Tia pays life for a Hero with counters.
#[test]
fn graha_tia_throws_wide_the_gates() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::graha_tia_scion_reborn());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[0].life;
    opt(&mut g);
    assert_eq!(g.players[0].life, life - 1);
    let hero = g.battlefield.iter().find(|x| x.definition.name == "Hero").unwrap();
    assert_eq!(hero.counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Hermes makes a Bird per noncreature spell.
#[test]
fn hermes_hatches_birds() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::hermes_overseer_of_elpis());
    opt(&mut g);
    assert_eq!(tokens(&g, 0, "Bird"), 1);
}

/// Hildibrand pumps tokens; his Adventure makes a Zombie.
#[test]
fn hildibrand_rises_as_a_gentleman() {
    let mut g = main_phase(2);
    let h = g.add_card_to_hand(0, catalog::hildibrand_manderville());
    act(&mut g, GameAction::CastAdventure { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Gentleman's Rise");
    assert_eq!(tokens(&g, 0, "Zombie"), 1);
    act(&mut g, GameAction::CastAdventureCreature { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("then the creature from exile");
    let z = g.battlefield.iter().find(|x| x.definition.name == "Zombie").unwrap().id;
    assert_eq!(pt(&g, z), (3, 3));
}

/// Hraesvelgr's entry makes a creature unblockable.
#[test]
fn hraesvelgr_aids() {
    let mut g = main_phase(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let h = g.add_card_to_hand(0, catalog::hraesvelgr_of_the_first_brood());
    cast_at(&mut g, h, &[Target::Permanent(bear)]).expect("cast");
    assert!(has(&g, bear, &Keyword::Unblockable));
}

/// Idyllic Beachfront enters tapped.
#[test]
fn idyllic_beachfront_enters_tapped() {
    let mut g = main_phase(2);
    let l = g.add_card_to_hand(0, catalog::idyllic_beachfront());
    act(&mut g, GameAction::PlayLand(l)).expect("play");
    assert!(g.battlefield_find(l).unwrap().tapped);
}

/// Into the Story draws four.
#[test]
fn into_the_story_draws_four() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 5);
    let i = g.add_card_to_hand(0, catalog::into_the_story());
    cast_at(&mut g, i, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 4);
}

/// Into the Story's {3} discount reads opponents' graveyards only.
#[test]
fn into_the_story_discount_needs_an_opponents_graveyard() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 5);
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::plains());
    }
    let i = g.add_card_to_hand(0, catalog::into_the_story());
    let four = |g: &mut GameState| {
        g.players[0].mana_pool.empty();
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add_colorless(2);
    };
    four(&mut g);
    let cast = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpell { card_id: i, target: None, additional_targets: vec![], mode: None, x_value: None })
    };
    assert!(cast(&mut g).is_err(), "your own graveyard doesn't count");
    for _ in 0..7 {
        g.add_card_to_graveyard(1, catalog::plains());
    }
    four(&mut g);
    cast(&mut g).expect("{2}{U}{U} with an opponent at seven");
}

/// Krile returns a creature card of the spell's mana value.
#[test]
fn krile_traces_aether() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::krile_baldesion());
    let m = g.add_card_to_graveyard(0, catalog::merchant_raiders());
    let s = g.add_card_to_hand(0, catalog::observed_stasis());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast_at(&mut g, s, &[Target::Permanent(bear)]).expect("a four-drop");
    assert!(g.players[0].hand.iter().any(|c| c.id == m));
}

/// Lyse Hext double strikes after two noncreature spells.
#[test]
fn lyse_hext_strikes_twice() {
    let mut g = main_phase(2);
    let l = g.add_card_to_battlefield(0, catalog::lyse_hext());
    opt(&mut g);
    assert!(!has(&g, l, &Keyword::DoubleStrike));
    opt(&mut g);
    assert!(has(&g, l, &Keyword::DoubleStrike));
}

/// Observed Stasis shuts a creature down.
#[test]
fn observed_stasis_freezes() {
    let mut g = main_phase(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let s = g.add_card_to_hand(0, catalog::observed_stasis());
    cast_at(&mut g, s, &[Target::Permanent(angel)]).expect("cast");
    assert!(!has(&g, angel, &Keyword::Flying));
    assert!(has(&g, angel, &Keyword::CantAttack));
}

/// Papalymo drains 1 per noncreature spell.
#[test]
fn papalymo_drains() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::papalymo_totolymo());
    let life = g.players[1].life;
    opt(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// Reaper's Scythe collects souls and grows its bearer.
#[test]
fn reapers_scythe_reaps() {
    let mut g = main_phase(2);
    let s = g.add_card_to_hand(0, catalog::reapers_scythe());
    cast_at(&mut g, s, &[]).expect("cast");
    let hero = g.battlefield.iter().find(|x| x.definition.name == "Hero").unwrap().id;
    g.battlefield_find_mut(s).unwrap().add_counters(CounterType::Soul, 2);
    assert_eq!(pt(&g, hero), (3, 3));
}

/// Summon: Good King Mog XII's first chapter makes two Moogles.
#[test]
fn good_king_mog_summons_moogles() {
    let mut g = main_phase(2);
    let m = g.add_card_to_hand(0, catalog::summon_good_king_mog_xii());
    cast_at(&mut g, m, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Moogle"), 2);
}

/// Thancred gains indestructible on noncreature spells.
#[test]
fn thancred_stands_guard() {
    let mut g = main_phase(2);
    let t = g.add_card_to_battlefield(0, catalog::thancred_waters());
    opt(&mut g);
    assert!(has(&g, t, &Keyword::Indestructible));
}

/// Transpose from hand leaves a Wizard.
#[test]
fn transpose_leaves_a_wizard() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 2);
    let t = g.add_card_to_hand(0, catalog::transpose());
    cast_at(&mut g, t, &[]).expect("cast");
    assert_eq!(tokens(&g, 0, "Wizard"), 1);
}

/// Urianger exiles his top card and lets you play it.
#[test]
fn urianger_reads_the_arcana() {
    let mut g = main_phase(2);
    let top = g.add_card_to_library(0, catalog::sol_ring());
    let u = g.add_card_to_battlefield(0, catalog::urianger_augurelt());
    g.clear_sickness(u);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    act(&mut g, GameAction::ActivateAbility { card_id: u, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("draw arcanum");
    assert!(g.exile.iter().any(|c| c.id == top));
}

/// Tataru draws you a card and offers the opponent one.
#[test]
fn tataru_keeps_the_books() {
    let mut g = main_phase(2);
    stock(&mut g, 0, 2);
    stock(&mut g, 1, 2);
    let t = g.add_card_to_hand(0, catalog::tataru_taru());
    cast_at(&mut g, t, &[Target::Player(1)]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Emet-Selch: an opponent losing life recasts an instant from your
/// graveyard, exiled after.
#[test]
fn emet_selch_recasts_from_the_graveyard() {
    let mut g = main_phase(2);
    g.add_card_to_battlefield(0, catalog::emet_selch_of_the_third_seat());
    let old = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    let b = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_at(&mut g, b, &[Target::Player(1)]).expect("bolt");
    assert_eq!(g.players[1].life, life - 6);
    assert!(g.exile.iter().any(|c| c.id == old));
}

/// Bug fix: the bot read a 0/0 Aura granting any keyword as beneficial, so
/// Pacifism went on its own best creature and Observed Stasis ("enchant
/// creature an opponent controls") was never cast (0 casts in 1,000 pods).
/// Both now aim at the opponent's threat.
#[test]
fn bot_aims_a_lockdown_aura_at_the_opponent() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    for aura in [catalog::observed_stasis, catalog::pacifism] {
        let mut g = main_phase(2);
        g.step = TurnStep::PostCombatMain;
        stock(&mut g, 0, 5);
        for _ in 0..3 {
            g.add_card_to_battlefield(0, catalog::island());
            g.add_card_to_battlefield(0, catalog::plains());
        }
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
        let id = g.add_card_to_hand(0, aura());
        match HeuristicBot::new().next_action(&g, 0) {
            Some(GameAction::CastSpell { card_id, target, .. }) => {
                assert_eq!(card_id, id);
                assert_eq!(target, Some(Target::Permanent(wurm)));
            }
            other => panic!("expected the Aura cast, got {other:?}"),
        }
    }
}

/// Hildibrand Manderville, dying, may be cast from the graveyard as Gentleman's
/// Rise until the end of your next turn (CR 715.3 — the Adventure only, never
/// the creature); the bot takes it, and the grant lapses after that turn.
#[test]
fn hildibrand_dies_into_a_graveyard_adventure() {
    let mut g = main_phase(3);
    let h = g.add_card_to_battlefield(0, catalog::hildibrand_manderville());
    let kill = g.add_card_to_hand(0, catalog::doom_blade());
    act(&mut g, GameAction::CastSpell {
        card_id: kill, target: Some(Target::Permanent(h)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Doom Blade on Hildibrand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == h));
    let creature = GameAction::CastSpell { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None };
    assert!(act(&mut g.clone(), creature).is_err(), "not the creature");
    {
        use crabomination::server::bot::{Bot, HeuristicBot};
        let mut post = g.clone();
        post.priority.player_with_priority = 0;
        post.step = TurnStep::PostCombatMain;
        flood(&mut post, 0);
        let action = HeuristicBot::new().next_action(&post, 0);
        assert!(matches!(action, Some(GameAction::CastAdventure { card_id, .. }) if card_id == h), "{action:?}");
    }
    let mut lapsed = g.clone();
    act(&mut g, GameAction::CastAdventure { card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Gentleman's Rise from the graveyard");
    assert_eq!(tokens(&g, 0, "Zombie"), 1);
    assert!(g.exile.iter().any(|c| c.id == h), "on an adventure in exile");
    // Unused, the grant ends with seat 0's next turn.
    lapsed.expire_adventure_grants(0, lapsed.turn_number + 3);
    assert!(act(&mut lapsed, GameAction::CastAdventure {
        card_id: h, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err());
}
