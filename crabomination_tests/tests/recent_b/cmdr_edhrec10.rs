//! Commander: one-card EDHREC gaps of EDHREC's top-1000 commanders
//! (`decks::cmdr_edhrec10`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
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

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x_value: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, id, targets, None)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    flood(g, 0);
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

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::island());
    }
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).is_some_and(|c| c.keywords().contains(&kw))
}

fn tokens_of(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.controller == seat && c.definition.name == name).count()
}

/// Thistledown Liege pumps other white and other blue creatures (a white-blue
/// one once per color, so +2/+2), not itself.
#[test]
fn thistledown_liege_lords_both_colors() {
    let mut g = pod(2);
    let liege = ready(&mut g, 0, catalog::thistledown_liege());
    let white = ready(&mut g, 0, catalog::savannah_lions());
    let blue = ready(&mut g, 0, catalog::merfolk_looter());
    let green = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(white).unwrap().power, 3);
    assert_eq!(g.computed_permanent(blue).unwrap().power, 2);
    assert_eq!(g.computed_permanent(green).unwrap().power, 2);
    assert_eq!(g.computed_permanent(liege).unwrap().power, 1, "not itself");
}

/// Geist of Saint Thalia: a noncreature spell costs {1} less, a creature
/// spell doesn't.
#[test]
fn geist_of_saint_thalia_discounts_noncreature_spells() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::geist_of_saint_thalia());
    library(&mut g, 0, 3);
    let opt = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell { card_id: opt, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Divination for {1}{U}");
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "a creature spell isn't discounted"
    );
}

/// Mass Calcify destroys nonwhite creatures only.
#[test]
fn mass_calcify_spares_white_creatures() {
    let mut g = pod(2);
    let lions = ready(&mut g, 1, catalog::savannah_lions());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::mass_calcify());
    cast(&mut g, m, &[]).expect("cast");
    assert!(g.battlefield_find(lions).is_some());
    assert!(g.battlefield_find(bears).is_none());
}

/// Union of the Third Path gains life equal to the hand after the draw.
#[test]
fn union_of_the_third_path_gains_per_card_in_hand() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::island());
    }
    let u = g.add_card_to_hand(0, catalog::union_of_the_third_path());
    let life = g.players[0].life;
    cast(&mut g, u, &[]).expect("cast");
    assert_eq!(g.players[0].life, life + 4, "three Islands plus the drawn card");
}

/// Pirate's Pillage: a discard as an extra cost, two cards, two Treasures.
#[test]
fn pirates_pillage_loots_into_treasure() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let p = g.add_card_to_hand(0, catalog::pirates_pillage());
    assert!(cast(&mut g, p, &[]).is_err(), "nothing to discard");
    g.add_card_to_hand(0, catalog::island());
    cast(&mut g, p, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(tokens_of(&g, 0, "Treasure"), 2);
}

/// Sheltering Word: hexproof and life equal to the target's toughness.
#[test]
fn sheltering_word_gains_the_toughness() {
    let mut g = pod(2);
    let wall = ready(&mut g, 0, catalog::wall_of_omens());
    let s = g.add_card_to_hand(0, catalog::sheltering_word());
    let life = g.players[0].life;
    cast(&mut g, s, &[perm(wall)]).expect("cast");
    assert!(has_kw(&g, wall, Keyword::Hexproof));
    assert_eq!(g.players[0].life, life + 4);
}

/// Polluted Bonds drains 2 for a land entering under an opponent's control,
/// not under its controller's.
#[test]
fn polluted_bonds_drains_opponents_lands() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::polluted_bonds());
    let (me, them) = (g.players[0].life, g.players[2].life);
    let land = g.add_card_to_hand(2, catalog::forest());
    g.active_player_idx = 2;
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    for _ in 0..6 {
        if g.stack.is_empty() {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!((g.players[0].life, g.players[2].life), (me + 2, them - 2));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(mine)).expect("play");
    assert!(g.stack.is_empty(), "not for your own lands");
}

/// Troop of Ponies: one basic onto the battlefield tapped, one to hand.
#[test]
fn troop_of_ponies_fetches_two_basics() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let t = ready(&mut g, 0, catalog::troop_of_ponies());
    let hand = g.players[0].hand.len();
    activate(&mut g, t, 0, None).expect("activate");
    assert!(g.battlefield_find(t).is_none(), "sacrificed");
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("a Forest");
    assert!(forest.tapped);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Dragon Turtle taps itself and an opponent's creature; neither untaps in
/// its controller's next untap step.
#[test]
fn dragon_turtle_drags_below() {
    let mut g = pod(2);
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::dragon_turtle());
    cast(&mut g, t, &[perm(foe)]).expect("cast");
    let turtle = g.battlefield_find(t).unwrap();
    assert!(turtle.tapped && turtle.skip_next_untap);
    let bear = g.battlefield_find(foe).unwrap();
    assert!(bear.tapped && bear.skip_next_untap);
}

/// Eiganjo Uprising: X hasty menace Samurai for you, X-1 for each opponent.
#[test]
fn eiganjo_uprising_arms_the_table() {
    let mut g = pod(3);
    let e = g.add_card_to_hand(0, catalog::eiganjo_uprising());
    cast_x(&mut g, e, &[], Some(3)).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Samurai"), 3);
    assert_eq!((tokens_of(&g, 1, "Samurai"), tokens_of(&g, 2, "Samurai")), (2, 2));
    let mine = g.battlefield.iter().find(|c| c.is_token && c.controller == 0).unwrap().id;
    assert!(has_kw(&g, mine, Keyword::Haste) && has_kw(&g, mine, Keyword::Menace));
    let theirs = g.battlefield.iter().find(|c| c.is_token && c.controller == 1).unwrap().id;
    assert!(!has_kw(&g, theirs, Keyword::Haste));
}

/// Magister Sphinx sets a target player's life total to 10.
#[test]
fn magister_sphinx_sets_life_to_ten() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::magister_sphinx());
    cast(&mut g, s, &[Target::Player(1)]).expect("cast");
    assert_eq!(g.players[1].life, 10);
}

/// Ravenous Robots: an artifact spell makes a Robot; its ability hastes
/// creature tokens only.
#[test]
fn ravenous_robots_build_and_rush() {
    let mut g = pod(2);
    let rr = ready(&mut g, 0, catalog::ravenous_robots());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Robot"), 1);
    let bot = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    activate(&mut g, rr, 0, None).expect("activate");
    assert!(has_kw(&g, bot, Keyword::Haste));
    assert!(!has_kw(&g, rr, Keyword::Haste), "not a token");
}

/// Selfless Savior: sacrifice to make another creature you control
/// indestructible.
#[test]
fn selfless_savior_shields_another() {
    let mut g = pod(2);
    let s = ready(&mut g, 0, catalog::selfless_savior());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, s, 0, Some(perm(bears))).expect("activate");
    assert!(g.battlefield_find(s).is_none());
    assert!(has_kw(&g, bears, Keyword::Indestructible));
}

fn walk(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: index, target, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn walker(g: &mut GameState, def: crabomination::card::CardDefinition, loyalty: u32) -> CardId {
    let id = ready(g, 0, def);
    g.battlefield_find_mut(id).unwrap().counters.insert(CounterType::Loyalty, loyalty);
    id
}

/// Liliana of the Dark Realms: +1 tutors a Swamp; −3 is +X/+X on your
/// creature and −X/−X on an opponent's (X = your Swamps); the emblem makes a
/// Swamp tap for {B}{B}{B}{B}.
#[test]
fn liliana_of_the_dark_realms_counts_swamps() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::swamp());
    let lili = walker(&mut g, catalog::liliana_of_the_dark_realms(), 12);
    walk(&mut g, lili, 0, None).expect("+1");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Swamp"));
    let swamp = ready(&mut g, 0, catalog::swamp());
    ready(&mut g, 0, catalog::swamp());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let reset = |g: &mut GameState| g.battlefield_find_mut(lili).unwrap().loyalty_uses_this_turn = 0;
    reset(&mut g);
    walk(&mut g, lili, 1, Some(perm(mine))).expect("-3 on mine");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4);
    reset(&mut g);
    walk(&mut g, lili, 1, Some(perm(theirs))).expect("-3 on theirs");
    assert!(g.battlefield_find(theirs).is_none(), "-2/-2 kills a 2/2");
    reset(&mut g);
    walk(&mut g, lili, 2, None).expect("-6");
    g.priority.player_with_priority = 0;
    let pool = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: swamp,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the emblem's granted ability");
    assert_eq!(g.players[0].mana_pool.total(), pool + 4);
}

/// Koth of the Hammer: +1 untaps and animates a Mountain into a 4/4 red
/// Elemental; −2 adds {R} per Mountain.
#[test]
fn koth_of_the_hammer_animates_and_ramps() {
    let mut g = pod(2);
    let koth = walker(&mut g, catalog::koth_of_the_hammer(), 10);
    let m = ready(&mut g, 0, catalog::mountain());
    ready(&mut g, 0, catalog::mountain());
    g.battlefield_find_mut(m).unwrap().tapped = true;
    walk(&mut g, koth, 0, Some(perm(m))).expect("+1");
    let cp = g.computed_permanent(m).unwrap();
    assert!(!g.battlefield_find(m).unwrap().tapped);
    assert_eq!((cp.power, cp.toughness), (4, 4));
    assert!(cp.colors.contains(Color::Red));
    g.battlefield_find_mut(koth).unwrap().loyalty_uses_this_turn = 0;
    let pool = g.players[0].mana_pool.total();
    walk(&mut g, koth, 1, None).expect("-2");
    assert_eq!(g.players[0].mana_pool.total(), pool + 2);
}

/// Basri Ket −2: each nontoken attacker this turn brings a tapped,
/// attacking Soldier.
#[test]
fn basri_ket_rallies_soldiers_to_the_attack() {
    let mut g = pod(2);
    let basri = walker(&mut g, catalog::basri_ket(), 3);
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::savannah_lions());
    walk(&mut g, basri, 1, None).expect("-2");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    let soldiers: Vec<_> = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Soldier").collect();
    assert_eq!(soldiers.len(), 2);
    assert!(soldiers.iter().all(|c| c.tapped));
}

/// Teferi, Temporal Pilgrim: drawing a card adds loyalty.
#[test]
fn teferi_temporal_pilgrim_grows_on_draws() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let t = walker(&mut g, catalog::teferi_temporal_pilgrim(), 4);
    walk(&mut g, t, 0, None).expect("0: draw");
    assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::Loyalty), 5);
}
