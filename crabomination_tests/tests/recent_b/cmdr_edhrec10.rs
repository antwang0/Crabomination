//! Commander: one-card EDHREC gaps of EDHREC's top-1000 commanders
//! (`decks::cmdr_edhrec10`).

use crabomination::card::{CardId, CounterType, Keyword};
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
    let lili = walker(&mut g, catalog::liliana_of_the_dark_realms(), 15);
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
    // Ruling: the sign is chosen as the ability resolves, not by whose
    // creature it is — the controller may shrink their own.
    reset(&mut g);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(1)]));
    walk(&mut g, lili, 1, Some(perm(mine))).expect("-3 shrinking mine");
    assert_eq!(g.computed_permanent(mine).unwrap().power, 2, "+2 then -2 this turn");
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

/// CR 702.11d — Eradicator Valkyrie has hexproof from planeswalkers: an
/// opponent's Liliana can't aim her −3 at it, but can at a Grizzly Bears.
#[test]
fn eradicator_valkyrie_has_hexproof_from_planeswalkers() {
    let mut g = pod(2);
    let lili = walker(&mut g, catalog::liliana_of_the_dark_realms(), 15);
    let valkyrie = ready(&mut g, 1, catalog::eradicator_valkyrie());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    assert!(walk(&mut g, lili, 1, Some(perm(valkyrie))).is_err(), "a loyalty ability can't target it");
    walk(&mut g, lili, 1, Some(perm(bears))).expect("the Bears are fair game");
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

/// Teferi, Temporal Pilgrim's −12: the target opponent chooses a permanent
/// they control to return to hand (their choice, not a target — CR 115.10),
/// then every other nonland permanent of theirs is shuffled away.
#[test]
fn teferi_temporal_pilgrim_ultimate_saves_one_permanent() {
    let mut g = pod(2);
    let t = walker(&mut g, catalog::teferi_temporal_pilgrim(), 12);
    let angel = ready(&mut g, 1, catalog::serra_angel());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let land = ready(&mut g, 1, catalog::forest());
    walk(&mut g, t, 2, Some(Target::Player(1))).expect("-12");
    let in_hand = |id| g.players[1].hand.iter().any(|c| c.id == id);
    assert_eq!([angel, bear].iter().filter(|id| in_hand(**id)).count(), 1, "one permanent back to hand");
    assert!(g.battlefield_find(angel).is_none() && g.battlefield_find(bear).is_none(), "the rest shuffled");
    assert!(g.battlefield_find(land).is_some() || in_hand(land), "lands are not shuffled");
}

/// Treefolk Umbra: the host deals combat damage by toughness (0/4 Wall of
/// Omens + 0/2 hits for 6 — but walls can't attack, so a 2/2 Bears hits for 4).
#[test]
fn treefolk_umbra_hits_by_toughness() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let u = g.add_card_to_hand(0, catalog::treefolk_umbra());
    cast(&mut g, u, &[perm(bears)]).expect("cast");
    assert_eq!(g.computed_permanent(bears).unwrap().toughness, 4);
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    for _ in 0..12 {
        if g.players[1].life != life {
            break;
        }
        let p = g.priority.player_with_priority;
        let _ = p;
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(g.players[1].life, life - 4);
}

/// Sigarda's Summons: a creature with a +1/+1 counter is a 4/4 flying Angel
/// (plus its counter), one without isn't.
#[test]
fn sigardas_summons_angels_the_countered() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sigardas_summons());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::savannah_lions());
    g.battlefield_find_mut(a).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    let cp = g.computed_permanent(a).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(has_kw(&g, a, Keyword::Flying));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Angel));
    assert!(!has_kw(&g, b, Keyword::Flying));
}

/// Kuldotha Forgemaster: sacrifice three artifacts (itself among them) to
/// put an artifact from the library onto the battlefield.
#[test]
fn kuldotha_forgemaster_trades_three_for_one() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::sol_ring());
    let k = ready(&mut g, 0, catalog::kuldotha_forgemaster());
    let a = ready(&mut g, 0, catalog::ornithopter());
    assert!(activate(&mut g, k, 0, None).is_err(), "two artifacts aren't three");
    let b = ready(&mut g, 0, catalog::ornithopter());
    activate(&mut g, k, 0, None).expect("activate");
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none() && g.battlefield_find(k).is_none());
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Sol Ring"));
}

/// Outlaws' Merriment makes one hasty red-and-white Human each upkeep.
#[test]
fn outlaws_merriment_makes_a_random_human() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::outlaws_merriment());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let t: Vec<_> = g.battlefield.iter().filter(|c| c.is_token).collect();
    assert_eq!(t.len(), 1);
    assert!(t[0].definition.name.starts_with("Human"));
    let id = t[0].id;
    assert!(has_kw(&g, id, Keyword::Haste));
}

/// Bard, King of Dale doubles tokens you create.
#[test]
fn bard_king_of_dale_doubles_tokens() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bard_king_of_dale());
    let p = g.add_card_to_hand(0, catalog::pirates_pillage());
    g.add_card_to_hand(0, catalog::island());
    library(&mut g, 0, 6);
    cast(&mut g, p, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Treasure"), 4);
}

/// Defiler of Faith: a white permanent spell makes a Soldier; it may pay
/// 2 life for its {W}.
#[test]
fn defiler_of_faith_makes_soldiers() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::defiler_of_faith());
    let lions = g.add_card_to_hand(0, catalog::savannah_lions());
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: lions, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("Lions for 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(tokens_of(&g, 0, "Soldier"), 1);
}

/// Grimgrin: sacrificing another creature untaps it with a counter.
#[test]
fn grimgrin_eats_to_untap() {
    let mut g = pod(2);
    let gr = ready(&mut g, 0, catalog::grimgrin_corpse_born());
    g.battlefield_find_mut(gr).unwrap().tapped = true;
    let food = ready(&mut g, 0, catalog::grizzly_bears());
    activate(&mut g, gr, 0, None).expect("sacrifice");
    assert!(g.battlefield_find(food).is_none());
    let c = g.battlefield_find(gr).unwrap();
    assert!(!c.tapped);
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Inferno of the Star Mounts: the activation that brings it to exactly 20
/// power deals 20 damage.
#[test]
fn inferno_of_the_star_mounts_fires_at_twenty() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    let inf = ready(&mut g, 0, catalog::inferno_of_the_star_mounts());
    for _ in 0..13 {
        activate(&mut g, inf, 0, None).expect("pump");
    }
    assert_eq!(g.computed_permanent(inf).unwrap().power, 19);
    let life = g.players[1].life;
    activate(&mut g, inf, 0, None).expect("pump to 20");
    assert_eq!(g.players[1].life, life - 20);
}

/// Guiding Hydra enters with X counters and may shift one onto each other
/// creature at your beginning of combat.
#[test]
fn guiding_hydra_shares_its_counters() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let h = g.add_card_to_hand(0, catalog::guiding_hydra());
    cast_x(&mut g, h, &[], Some(3)).expect("cast");
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(h).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Greasefang returns a Vehicle from your graveyard with haste, back to hand
/// at the next end step.
#[test]
fn greasefang_borrows_a_vehicle_for_the_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::greasefang_okiba_boss());
    let v = g.add_card_to_graveyard(0, catalog::smugglers_copter());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(g.battlefield_find(v).is_some(), "Copter returned");
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == v), "back to hand");
}

/// Psemilla: only the turn's first enchantment spell makes a Nymph; five
/// enchantments pump her at a combat.
#[test]
fn psemilla_nymphs_and_grows() {
    let mut g = pod(2);
    let p = ready(&mut g, 0, catalog::psemilla_meletian_poet());
    let a = g.add_card_to_hand(0, catalog::sultai_ascendancy());
    let b = g.add_card_to_hand(0, catalog::polluted_bonds());
    cast(&mut g, a, &[]).expect("first");
    cast(&mut g, b, &[]).expect("second");
    assert_eq!(tokens_of(&g, 0, "Nymph"), 1, "the first enchantment spell only");
    for _ in 0..3 {
        ready(&mut g, 0, catalog::sigardas_summons());
    }
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(p).unwrap().power, 5);
    assert!(has_kw(&g, p, Keyword::Lifelink));
}

/// Feast of the Victorious Dead: an end step after two deaths gains 2 and
/// spreads two counters.
#[test]
fn feast_of_the_victorious_dead_counts_the_dead() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::feast_of_the_victorious_dead());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    for s in [0, 1] {
        let v = ready(&mut g, s, catalog::savannah_lions());
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, &[perm(v)]).expect("bolt");
    }
    let life = g.players[0].life;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2);
    assert_eq!(g.battlefield_find(mine).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Anje: a Vampire entering makes one Blood a turn; sacrificing it drains
/// each opponent for 2.
#[test]
fn anje_bleeds_the_table() {
    let mut g = pod(3);
    let anje = ready(&mut g, 0, catalog::anje_maid_of_dishonor());
    let v = g.add_card_to_hand(0, catalog::vampire_nighthawk());
    cast(&mut g, v, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Blood"), 1);
    let (a, b, me) = (g.players[1].life, g.players[2].life, g.players[0].life);
    activate(&mut g, anje, 0, None).expect("drain");
    assert_eq!((g.players[1].life, g.players[2].life, g.players[0].life), (a - 2, b - 2, me + 2), "you gain 2, not 2 per opponent");
}

/// Archghoul of Thraben: a Zombie dying puts a Zombie card on top into hand.
#[test]
fn archghoul_digs_for_zombies() {
    let mut g = pod(2);
    let ghoul = ready(&mut g, 0, catalog::archghoul_of_thraben());
    let top = g.add_card_to_library(0, catalog::archghoul_of_thraben());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(ghoul)]).expect("bolt");
    assert!(g.players[0].hand.iter().any(|c| c.id == top));
}

/// Emissary Green: in a two-seat pod both votes land somewhere — two
/// Treasures per profit vote, a counter per security vote, four "units" in
/// total.
#[test]
fn emissary_green_pays_out_each_vote() {
    let mut g = pod(2);
    let e = ready(&mut g, 0, catalog::emissary_green());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: e, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let treasures = tokens_of(&g, 0, "Treasure");
    let counters = g.battlefield_find(e).unwrap().counter_count(CounterType::PlusOnePlusOne) as usize;
    assert_eq!(treasures / 2 + counters, 2, "two votes");
}

/// The Millennium Calendar: a counter per permanent untapped in your untap
/// step; doubling to 1,000 sacrifices it and each opponent loses 1,000.
#[test]
fn millennium_calendar_counts_to_a_thousand() {
    let mut g = pod(3);
    let cal = ready(&mut g, 0, catalog::the_millennium_calendar());
    for _ in 0..2 {
        let l = ready(&mut g, 0, catalog::forest());
        g.battlefield_find_mut(l).unwrap().tapped = true;
    }
    g.step = TurnStep::Untap;
    g.do_untap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cal).unwrap().counter_count(CounterType::Time), 2);
    g.step = TurnStep::PreCombatMain;
    g.battlefield_find_mut(cal).unwrap().counters.insert(CounterType::Time, 500);
    activate(&mut g, cal, 0, None).expect("double");
    assert!(g.battlefield_find(cal).is_none(), "sacrificed at 1,000");
    assert!(g.players[1].life <= 0 && g.players[2].life <= 0);
}

/// Confounding Conundrum: an opponent's second land in a turn sends one home.
/// Bug fix: `lands_entered_this_turn` counted only land TOKENS ("played or
/// otherwise" per its doc), so a played land plus a fetched one read as zero
/// and this — like Lavaball Trap and Nelly Borca's draw — never fired.
#[test]
fn confounding_conundrum_bounces_the_second_land() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::confounding_conundrum());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let a = g.add_card_to_hand(1, catalog::forest());
    g.perform_action(GameAction::PlayLand(a)).expect("land 1");
    drain_stack(&mut g);
    let lands = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_land()).count();
    assert_eq!(lands(&g), 1);
    g.priority.player_with_priority = 1;
    let r = g.add_card_to_hand(1, catalog::rampant_growth());
    g.add_card_to_library(1, catalog::forest());
    flood(&mut g, 1);
    g.perform_action(GameAction::CastSpell { card_id: r, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("ramp");
    drain_stack(&mut g);
    assert_eq!(lands(&g), 1, "the second land bounced one");
}

fn cast_from(g: &mut GameState, id: CardId, target: Option<Target>) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Lurrus of the Dream-Den: once each of your turns, a permanent spell with
/// mana value 2 or less from your graveyard — not a third one, nor a 3-drop.
/// Bug fix: it rode Muldrotha's static (one per permanent type, any mana
/// value), so a Lurrus seat recast a five-drop from its graveyard.
#[test]
fn lurrus_casts_one_cheap_permanent_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::lurrus_of_the_dream_den());
    let big = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let lions = g.add_card_to_graveyard(0, catalog::savannah_lions());
    flood(&mut g, 0);
    assert!(cast_from(&mut g, big, None).is_err(), "mana value 6");
    cast_from(&mut g, bears, None).expect("a 2-drop");
    assert!(g.battlefield_find(bears).is_some());
    assert!(cast_from(&mut g, lions, None).is_err(), "once a turn");
}

/// Danitha, New Benalia's Light: an Aura from the graveyard, once a turn.
#[test]
fn danitha_recasts_an_aura() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::danitha_new_benalias_light());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let p = g.add_card_to_graveyard(0, catalog::pacifism());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    assert!(cast_from(&mut g, bears, None).is_err(), "not an Aura or Equipment");
    cast_from(&mut g, p, Some(perm(foe))).expect("Pacifism from the graveyard");
    assert_eq!(g.battlefield_find(p).and_then(|c| c.attached_to), Some(foe));
}

/// Astor, Bearer of Blades: equip {1} beats Loxodon Warhammer's {3}; the
/// ETB digs an Equipment out of the top seven.
#[test]
fn astor_equips_for_one() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let hammer_card = g.add_card_to_library(0, catalog::bonesplitter());
    let a = g.add_card_to_hand(0, catalog::astor_bearer_of_blades());
    cast(&mut g, a, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == hammer_card), "dug the Equipment");
    let hammer = ready(&mut g, 0, catalog::loxodon_warhammer());
    g.clear_sickness(a);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: hammer, target: a }).expect("equip {1}");
    assert_eq!(g.battlefield_find(hammer).and_then(|c| c.attached_to), Some(a));
}

/// Claim // Fame: Claim reanimates a 2-drop; Fame is cast from the graveyard
/// (aftermath) for +2/+0 and haste.
#[test]
fn claim_then_fame() {
    let mut g = pod(2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::claim_fame());
    cast(&mut g, c, &[Target::Permanent(bears)]).expect("Claim");
    assert!(g.battlefield_find(bears).is_some());
    assert!(g.players[0].graveyard.iter().any(|x| x.id == c));
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAftermath {
        card_id: c,
        target: Some(perm(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Fame");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
    assert!(has_kw(&g, bears, Keyword::Haste));
}

/// Evil Twin copies a creature and can destroy one with the same name.
#[test]
fn evil_twin_kills_its_original() {
    let mut g = pod(2);
    let orig = ready(&mut g, 1, catalog::craw_wurm());
    let t = g.add_card_to_hand(0, catalog::evil_twin());
    cast(&mut g, t, &[]).expect("cast");
    assert_eq!(g.computed_permanent(t).unwrap().power, 6, "a Craw Wurm");
    g.clear_sickness(t);
    let n = g.battlefield_find(t).unwrap().definition.activated_abilities.len();
    activate(&mut g, t, n.saturating_sub(1), Some(perm(orig))).expect("destroy the original");
    assert!(g.battlefield_find(orig).is_none());
}

/// The Mind Stone: harnessed, it blinks another nonland permanent at your end
/// step (the blinked creature is a new object, CR 400.7).
#[test]
fn the_mind_stone_blinks_once_harnessed() {
    let mut g = pod(2);
    let stone = ready(&mut g, 0, catalog::the_mind_stone());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().counters.insert(CounterType::PlusOnePlusOne, 1);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "not harnessed");
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, stone, 1, None).expect("harness");
    assert!(g.battlefield_find(stone).unwrap().harnessed);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 0, "a new object");
}

/// Claim the Kingdom: four landfalls grow a creature four times, then the
/// Plan is sacrificed for an indestructible counter.
#[test]
fn claim_the_kingdom_pays_off_on_the_fourth_land() {
    let mut g = pod(2);
    let ck = ready(&mut g, 0, catalog::claim_the_kingdom());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..4 {
        let f = g.add_card_to_hand(0, catalog::forest());
        g.players[0].lands_played_this_turn = 0;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(f)).expect("land");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(ck).is_none(), "sacrificed");
    let b = g.battlefield_find(bears).unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 4);
    assert_eq!(b.counter_count(CounterType::Indestructible), 1);
}

/// Lady Octopus: the first two draws a turn add ingenuity counters; her tap
/// casts an artifact up to that mana value free.
#[test]
fn lady_octopus_builds_free_artifacts() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let lo = ready(&mut g, 0, catalog::lady_octopus_inspired_inventor());
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, &[]).expect("draw two");
    assert_eq!(g.battlefield_find(lo).unwrap().counter_count(CounterType::Ingenuity), 2);
    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    let vault = g.add_card_to_hand(0, catalog::loxodon_warhammer());
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: lo, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(thopter).is_some(), "Ornithopter cast free");
    assert!(g.battlefield_find(vault).is_none());
}

/// Lara Croft: attacking exiles a legendary artifact from a graveyard with a
/// discovery counter, playable this turn; raid makes a Treasure at end of
/// combat.
#[test]
fn lara_croft_raids_tombs() {
    let mut g = pod(2);
    let lara = ready(&mut g, 0, catalog::lara_croft_tomb_raider());
    let relic = g.add_card_to_graveyard(1, catalog::the_millennium_calendar());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: lara, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let c = g.exile.iter().find(|c| c.id == relic).expect("exiled");
    assert_eq!(c.counter_count(CounterType::Discovery), 1);
    assert!(c.may_play_until.is_some_and(|p| p.player == 0));
    g.step = TurnStep::EndCombat;
    g.fire_step_triggers(TurnStep::EndCombat);
    drain_stack(&mut g);
    assert_eq!(tokens_of(&g, 0, "Treasure"), 1);
}

/// Lara Croft's "you may play A card from exile with a discovery counter":
/// the grants share one play — a land drop through one spends the other.
#[test]
fn lara_croft_plays_one_discovery_card_not_all() {
    let mut g = pod(2);
    let lara = ready(&mut g, 0, catalog::lara_croft_tomb_raider());
    let relic = g.add_card_to_graveyard(1, catalog::the_millennium_calendar());
    let forest = g.add_card_to_exile(0, catalog::forest());
    g.exile.iter_mut().find(|c| c.id == forest).unwrap().counters.insert(CounterType::Discovery, 1);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: lara, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count() >= 2, "both playable");
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(forest)).expect("play the Forest from exile");
    let c = g.exile.iter().find(|c| c.id == relic).expect("still exiled");
    assert!(c.may_play_until.is_none(), "the one play is spent");
}

/// Widespread Thieving: a multicolored spell makes a Treasure.
#[test]
fn widespread_thieving_pays_for_gold_spells() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let w = g.add_card_to_hand(0, catalog::widespread_thieving());
    cast(&mut g, w, &[]).expect("cast");
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(w)).count(), 1, "hideaway 5");
    let gold = g.add_card_to_hand(0, catalog::anje_maid_of_dishonor());
    cast(&mut g, gold, &[]).expect("a gold spell");
    assert_eq!(tokens_of(&g, 0, "Treasure"), 1);
}

/// The Watcher in the Water enters tapped with nine stun counters; a draw on
/// an opponent's turn makes a Tentacle, not one on yours.
#[test]
fn the_watcher_in_the_water_grows_tentacles() {
    let mut g = pod(2);
    library(&mut g, 0, 4);
    let w = g.add_card_to_hand(0, catalog::the_watcher_in_the_water());
    cast(&mut g, w, &[]).expect("cast");
    let c = g.battlefield_find(w).unwrap();
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::Stun), 9);
    let div = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, div, &[]).expect("my turn");
    assert_eq!(tokens_of(&g, 0, "Tentacle"), 0);
    g.active_player_idx = 1;
    let opt = g.add_card_to_hand(0, catalog::union_of_the_third_path());
    cast(&mut g, opt, &[]).expect("their turn");
    assert_eq!(tokens_of(&g, 0, "Tentacle"), 1);
}

/// Corporeal Projection: the granted myriad copies an attacker at each other
/// opponent (three seats: one copy).
#[test]
fn corporeal_projection_grants_myriad() {
    let mut g = pod(3);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let cp = g.add_card_to_hand(0, catalog::corporeal_projection());
    cast(&mut g, cp, &[perm(bears)]).expect("cast");
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true); 4]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let copies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Grizzly Bears").count();
    assert_eq!(copies, 1);
}

/// Ruthless Ripper: the morph cost is revealing a black card (none in hand,
/// no turn-up); turned up, it drains a target player for 2.
#[test]
fn ruthless_ripper_reveals_to_flip() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    let r = g.add_card_to_hand(0, catalog::ruthless_ripper());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFaceDown { card_id: r }).expect("face down for {3}");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::TurnFaceUp { card_id: r }).is_err(), "no black card to reveal");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(g.perform_action(GameAction::TurnFaceUp { card_id: r }).is_err(), "a red card doesn't do");
    let _ = bolt;
    let black = g.add_card_to_hand(0, catalog::archghoul_of_thraben());
    let life = g.players[1].life;
    g.perform_action(GameAction::TurnFaceUp { card_id: r }).expect("reveal Archghoul");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == black), "revealed, not discarded");
    assert_eq!(g.players[1].life, life - 2);
}

/// Edge of Autumn ramps at four or fewer lands; its cycling sacrifices a land
/// (CR 702.29) and can't be paid without one.
#[test]
fn edge_of_autumn_ramps_or_cycles() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let e = g.add_card_to_hand(0, catalog::edge_of_autumn());
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::Cycle { card_id: e, x_value: None }).is_err(), "no land to sacrifice");
    cast(&mut g, e, &[]).expect("cast");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count(), 1);
    let e2 = g.add_card_to_hand(0, catalog::edge_of_autumn());
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Cycle { card_id: e2, x_value: None }).expect("cycle");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count(), 0, "the land paid");
    assert_eq!(g.players[0].hand.len(), hand, "discarded one, drew one");
}

/// Trail of Evidence: an instant or sorcery investigates.
#[test]
fn trail_of_evidence_investigates() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::trail_of_evidence());
    library(&mut g, 0, 3);
    let u = g.add_card_to_hand(0, catalog::union_of_the_third_path());
    cast(&mut g, u, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Clue"), 1);
}

/// Virulent Plague shrinks every creature token, not cards.
#[test]
fn virulent_plague_kills_small_tokens() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let e = g.add_card_to_hand(1, catalog::eiganjo_uprising());
    let _ = e;
    let vp = g.add_card_to_hand(0, catalog::virulent_plague());
    let up = g.add_card_to_hand(0, catalog::eiganjo_uprising());
    cast_x(&mut g, up, &[], Some(2)).expect("tokens");
    cast(&mut g, vp, &[]).expect("plague");
    assert_eq!(tokens_of(&g, 0, "Samurai") + tokens_of(&g, 1, "Samurai"), 0, "2/2 tokens die");
    assert!(g.battlefield_find(bears).is_some());
}

/// Rescuer Chwinga returns another permanent you control.
#[test]
fn rescuer_chwinga_rescues() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let c = g.add_card_to_hand(0, catalog::rescuer_chwinga());
    cast(&mut g, c, &[]).expect("cast");
    assert!(g.battlefield_find(c).is_some(), "not itself");
    let _ = bears;
}

/// Thieves' Tools: a Treasure, and the equipped 2/2 can't be blocked; a
/// pump past 3 power makes it blockable again.
#[test]
fn thieves_tools_sneaks_small_creatures() {
    let mut g = pod(2);
    let t = g.add_card_to_hand(0, catalog::thieves_tools());
    cast(&mut g, t, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Treasure"), 1);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: t, target: bears }).expect("equip");
    assert!(has_kw(&g, bears, Keyword::UnblockableWhilePowerAtMost(3)));
}

/// Eradicator Valkyrie's boast (after attacking): each opponent sacrifices.
#[test]
fn eradicator_valkyrie_boasts() {
    let mut g = pod(3);
    let v = ready(&mut g, 0, catalog::eradicator_valkyrie());
    let fodder = ready(&mut g, 0, catalog::grizzly_bears());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    assert!(activate(&mut g, v, 0, None).is_err(), "it hasn't attacked");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: v, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    activate(&mut g, v, 0, None).expect("boast");
    assert!(g.battlefield_find(fodder).is_none());
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Sentinel Tower: the second instant of your turn deals 2.
#[test]
fn sentinel_tower_escalates() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    ready(&mut g, 0, catalog::sentinel_tower());
    library(&mut g, 0, 4);
    let life = g.players[1].life;
    for _ in 0..2 {
        let u = g.add_card_to_hand(0, catalog::union_of_the_third_path());
        cast(&mut g, u, &[]).expect("cast");
    }
    assert_eq!(g.players[1].life, life - 3, "1 then 2");
}

/// Sentinel Tower counts every instant and sorcery cast before the trigger's
/// spell this turn, whoever cast it — an opponent's instant on your turn raises
/// the next shot (its oracle: "spells cast before that spell this turn").
#[test]
fn sentinel_tower_counts_an_opponents_instant() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    ready(&mut g, 0, catalog::sentinel_tower());
    library(&mut g, 0, 4);
    library(&mut g, 1, 4);
    let theirs = g.add_card_to_hand(1, catalog::union_of_the_third_path());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: theirs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("their instant");
    drain_stack(&mut g);
    let life = g.players[1].life;
    let u = g.add_card_to_hand(0, catalog::union_of_the_third_path());
    cast(&mut g, u, &[]).expect("cast");
    assert_eq!(g.players[1].life, life - 2, "1 plus their earlier instant");
}

/// Mishra's Self-Replicator copies itself off a historic spell for {1}.
#[test]
fn mishras_self_replicator_copies() {
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    ready(&mut g, 0, catalog::mishras_self_replicator());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Mishra's Self-Replicator"), 1);
}

/// Joraga Treespeaker: level 1 taps for {G}{G}; at level 5 its Elves do.
#[test]
fn joraga_treespeaker_levels_into_a_mana_engine() {
    let mut g = pod(2);
    let j = ready(&mut g, 0, catalog::joraga_treespeaker());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    assert!(activate(&mut g, j, 1, None).is_err(), "level 0");
    activate(&mut g, j, 0, None).expect("level up");
    assert_eq!(g.computed_permanent(j).unwrap().toughness, 2);
    g.players[0].mana_pool = Default::default();
    activate(&mut g, j, 1, None).expect("tap for GG");
    g.battlefield_find_mut(j).unwrap().counters.insert(CounterType::Level, 5);
    assert_eq!(g.computed_permanent(j).unwrap().toughness, 4);
    let n = g.battlefield_find(elf).unwrap().definition.activated_abilities.len();
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: elf, ability_index: n, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("the granted {T}: Add {G}{G}");
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Teferi, Master of Time activates on an opponent's turn.
#[test]
fn teferi_master_of_time_works_on_their_turn() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let t = walker(&mut g, catalog::teferi_master_of_time(), 3);
    g.add_card_to_hand(0, catalog::island());
    g.active_player_idx = 1;
    g.step = TurnStep::End;
    walk(&mut g, t, 0, None).expect("+1 at instant speed");
    assert_eq!(g.battlefield_find(t).unwrap().counter_count(CounterType::Loyalty), 4);
}

/// Lupine Prototype attacks only while some player's hand is empty.
#[test]
fn lupine_prototype_needs_an_empty_hand() {
    let mut g = pod(3);
    let w = ready(&mut g, 0, catalog::lupine_prototype());
    for s in 0..3 {
        g.add_card_to_hand(s, catalog::island());
    }
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: w, target: AttackTarget::Player(1) }])).is_err());
    g.players[2].hand.clear();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: w, target: AttackTarget::Player(1) }]))
        .expect("seat 2's hand is empty");
}

/// Majestic Genesis reveals as many cards as your commander's mana value and
/// deploys the permanents among them (CR 903.3 — the commander designation).
#[test]
fn majestic_genesis_reads_the_commander() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let a = g.add_card_to_library(0, catalog::savannah_lions());
    let b = g.add_card_to_library(0, catalog::union_of_the_third_path());
    let c = g.add_card_to_library(0, catalog::forest());
    let m = g.add_card_to_hand(0, catalog::majestic_genesis());
    cast(&mut g, m, &[]).expect("cast");
    assert!(g.battlefield_find(a).is_some(), "MV 2: the top two are revealed");
    assert!(g.battlefield_find(b).is_none() && g.battlefield_find(c).is_none());
}

/// Tetzimoc marks from hand on your turn, then its entry destroys the marked.
#[test]
fn tetzimoc_hunts_its_prey() {
    let mut g = pod(2);
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let t = g.add_card_to_hand(0, catalog::tetzimoc_primal_death());
    activate(&mut g, t, 0, Some(perm(foe))).expect("mark from hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == t), "revealed, still in hand");
    assert_eq!(g.battlefield_find(foe).unwrap().counter_count(CounterType::Prey), 1);
    cast(&mut g, t, &[]).expect("cast");
    assert!(g.battlefield_find(foe).is_none());
}

/// Sword of Wealth and Power: a hit makes a Treasure and copies the next
/// instant cast this turn.
#[test]
fn sword_of_wealth_and_power_copies_the_next_instant() {
    let mut g = pod(2);
    library(&mut g, 0, 6);
    let sword = ready(&mut g, 0, catalog::sword_of_wealth_and_power());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: sword, target: bears }).expect("equip");
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    for _ in 0..12 {
        if tokens_of(&g, 0, "Treasure") > 0 {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert_eq!(tokens_of(&g, 0, "Treasure"), 1);
    drain_stack(&mut g);
    g.step = TurnStep::PostCombatMain;
    let hand = g.players[0].hand.len();
    let u = g.add_card_to_hand(0, catalog::union_of_the_third_path());
    cast(&mut g, u, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 2, "the copy drew too");
}

/// Relentless Dead: dying, {B} returns it to hand and {X} returns another
/// Zombie with mana value X.
#[test]
fn relentless_dead_keeps_coming_back() {
    let mut g = pod(2);
    let rd = ready(&mut g, 0, catalog::relentless_dead());
    let ghoul = g.add_card_to_graveyard(0, catalog::archghoul_of_thraben());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(3), DecisionAnswer::Bool(true)]));
    // The {X} trigger was put on the stack last, so it resolves (and asks)
    // first.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(rd)]).expect("bolt");
    assert!(g.players[0].hand.iter().any(|c| c.id == rd), "back to hand");
    assert!(g.battlefield_find(ghoul).is_some(), "the X=3 Zombie returned");
}

/// Chiss-Goria: affinity for artifacts; attacking exiles the top five and
/// one artifact among them may be cast this turn — with affinity for
/// artifacts (CR 702.41a; it used to pay full price).
#[test]
fn chiss_goria_forges_from_the_top() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::ornithopter());
    }
    let cg = g.add_card_to_hand(0, catalog::chiss_goria_forge_tyrant());
    g.players[0].mana_pool.add(Color::Red, 6);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: cg, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{6}{R}{R}{R} less three");
    drain_stack(&mut g);
    let ring = g.add_card_to_library(0, catalog::sol_ring());
    let ring2 = g.add_card_to_library(0, catalog::mind_stone());
    library(&mut g, 0, 4);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: cg, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let granted = |id| g.exile.iter().any(|c| c.id == id && c.may_play_until.is_some());
    assert!(granted(ring) && granted(ring2));
    assert_eq!(g.exile.iter().filter(|c| c.exiled_with == Some(cg)).count(), 5);
    // Three Ornithopters pay Mind Stone's {2} with no mana up.
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: ring2,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Mind Stone with affinity");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring2).is_some());
}

/// The Ghoul, Gunslinger: a nontoken Zombie of yours dying gives a target
/// player two rad counters (a Treasure when it's you).
#[test]
fn the_ghoul_irradiates() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_ghoul_gunslinger());
    let z = ready(&mut g, 0, catalog::relentless_dead());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].hostile_player_targets = true;
    cast(&mut g, bolt, &[perm(z)]).expect("bolt");
    assert_eq!(g.players[1].rad_counters + g.players[0].rad_counters, 2);
    if g.players[0].rad_counters == 2 {
        assert_eq!(tokens_of(&g, 0, "Treasure"), 1);
    }
}

// ── The two-card gaps (ranks 601-1000) ──────────────────────────────────────

/// Ghoulcaller's Bell mills every player one.
#[test]
fn ghoulcallers_bell_mills_the_table() {
    let mut g = pod(3);
    for s in 0..3 {
        library(&mut g, s, 2);
    }
    let bell = ready(&mut g, 0, catalog::ghoulcallers_bell());
    activate(&mut g, bell, 0, None).expect("tap");
    assert!((0..3).all(|s| g.players[s].graveyard.len() == 1));
}

/// Hand of the Praetors pumps other infect creatures and poisons on an
/// infect creature spell.
#[test]
fn hand_of_the_praetors_spreads_poison() {
    let mut g = pod(2);
    g.players[0].hostile_player_targets = true;
    ready(&mut g, 0, catalog::hand_of_the_praetors());
    let cur = g.add_card_to_hand(0, catalog::corpse_cur());
    cast(&mut g, cur, &[]).expect("cast");
    assert_eq!(g.computed_permanent(cur).unwrap().power, 3, "a lord for infect");
    assert_eq!(g.players[1].poison_counters, 1);
}

/// Deathbloom Ritualist counts creature cards in your graveyard.
#[test]
fn graveyard_mana_counts_creature_cards() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.add_card_to_graveyard(0, catalog::island());
    let dr = ready(&mut g, 0, catalog::deathbloom_ritualist());
    g.players[0].mana_pool = Default::default();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: dr, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 3);
}

/// Electrostatic Infantry grows per instant or sorcery.
#[test]
fn electrostatic_infantry_grows() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let ei = ready(&mut g, 0, catalog::electrostatic_infantry());
    let u = g.add_card_to_hand(0, catalog::union_of_the_third_path());
    cast(&mut g, u, &[]).expect("cast");
    assert_eq!(g.battlefield_find(ei).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Super Shredder grows when another permanent leaves the battlefield.
#[test]
fn super_shredder_feeds_on_departures() {
    let mut g = pod(2);
    let ss = ready(&mut g, 0, catalog::super_shredder());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(foe)]).expect("bolt");
    assert_eq!(g.battlefield_find(ss).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Shredder's Technique: an enchantment destroyed costs 2 life, a creature
/// doesn't.
#[test]
fn shredders_technique_taxes_enchantments() {
    let mut g = pod(2);
    let ench = ready(&mut g, 1, catalog::polluted_bonds());
    let st = g.add_card_to_hand(0, catalog::shredders_technique());
    let life = g.players[0].life;
    cast(&mut g, st, &[perm(ench)]).expect("cast");
    assert!(g.battlefield_find(ench).is_none());
    assert_eq!(g.players[0].life, life - 2);
    // "If an enchantment was destroyed this way": an indestructible god
    // survives, so nothing is paid.
    let god = ready(&mut g, 1, catalog::thassa_god_of_the_sea());
    let st = g.add_card_to_hand(0, catalog::shredders_technique());
    cast(&mut g, st, &[perm(god)]).expect("cast at the god");
    assert!(g.battlefield_find(god).is_some());
    assert_eq!(g.players[0].life, life - 2);
}

/// Snapback can be cast by exiling a blue card.
#[test]
fn snapback_pitches_a_blue_card() {
    let mut g = pod(2);
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let blue = g.add_card_to_hand(0, catalog::dragon_turtle());
    let sb = g.add_card_to_hand(0, catalog::snapback());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative { card_id: sb, pitch_card: Some(blue), target: Some(perm(foe)), additional_targets: vec![], mode: None, x_value: None })
        .expect("pitch");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == foe));
    assert!(g.exile.iter().any(|c| c.id == blue));
}

/// Wargate fetches a permanent with mana value X or less onto the
/// battlefield.
#[test]
fn wargate_deploys_x_or_less() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::grizzly_bears());
    let w = g.add_card_to_hand(0, catalog::wargate());
    cast_x(&mut g, w, &[], Some(2)).expect("cast");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Grizzly Bears"));
}

/// Liliana the Faultless gains a life per other creature entering.
#[test]
fn liliana_the_faultless_gains_on_entries() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::liliana_the_faultless());
    let life = g.players[0].life;
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bears, &[]).expect("cast");
    assert_eq!(g.players[0].life, life + 1);
}

/// Ajani Resolute: gaining life adds loyalty; −4 makes Ajani's Pridemate.
#[test]
fn ajani_resolute_grows_with_life() {
    let mut g = pod(2);
    let a = walker(&mut g, catalog::ajani_resolute(), 2);
    walk(&mut g, a, 0, None).expect("0: gain 1");
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::Loyalty), 3, "the life gain added one");
    g.battlefield_find_mut(a).unwrap().counters.insert(CounterType::Loyalty, 6);
    g.battlefield_find_mut(a).unwrap().loyalty_uses_this_turn = 0;
    walk(&mut g, a, 1, None).expect("-4");
    assert_eq!(tokens_of(&g, 0, "Ajani's Pridemate"), 1);
}

/// Feasting Troll King cast from hand brings three Foods, and three Foods
/// bring it back from the graveyard on your turn.
#[test]
fn feasting_troll_king_eats_its_way_back() {
    let mut g = pod(2);
    let k = g.add_card_to_hand(0, catalog::feasting_troll_king());
    cast(&mut g, k, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Food"), 3);
    let bolt = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, bolt, &[perm(k)]).expect("kill");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == k));
    activate(&mut g, k, 0, None).expect("three Foods");
    assert!(g.battlefield_find(k).is_some());
    assert_eq!(tokens_of(&g, 0, "Food"), 0);
}

/// Giant Opportunity: three Foods without two to spare.
#[test]
fn giant_opportunity_makes_food_or_a_giant() {
    let mut g = pod(2);
    let go = g.add_card_to_hand(0, catalog::giant_opportunity());
    cast(&mut g, go, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Food"), 3);
}

/// Slash the Ranks spares commanders (CR 903.3).
#[test]
fn slash_the_ranks_spares_commanders() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::slash_the_ranks());
    cast(&mut g, s, &[]).expect("cast");
    assert!(g.battlefield_find(cmdr).is_some());
    assert!(g.battlefield_find(other).is_none());
}

/// Stimulus Package turns a Treasure into a Citizen.
#[test]
fn stimulus_package_spends_treasure() {
    let mut g = pod(2);
    let sp = g.add_card_to_hand(0, catalog::stimulus_package());
    cast(&mut g, sp, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Treasure"), 2);
    activate(&mut g, sp, 0, None).expect("sac a Treasure");
    assert_eq!((tokens_of(&g, 0, "Treasure"), tokens_of(&g, 0, "Citizen")), (1, 1));
}

/// Morbid Curiosity draws the sacrificed permanent's mana value.
#[test]
fn morbid_curiosity_draws_by_mana_value() {
    let mut g = pod(2);
    library(&mut g, 0, 8);
    ready(&mut g, 0, catalog::craw_wurm());
    let mc = g.add_card_to_hand(0, catalog::morbid_curiosity());
    let hand = g.players[0].hand.len();
    cast(&mut g, mc, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 6);
}

/// Ares, God of War returns your attacking creatures that die to hand.
#[test]
fn ares_returns_fallen_attackers() {
    let mut g = pod(2);
    let ares = ready(&mut g, 0, catalog::ares_god_of_war());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ares, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, bolt, &[perm(ares)]).expect("kill the attacker");
    assert!(g.players[0].hand.iter().any(|c| c.id == ares));
}

// ── The one-card gaps left in the 601-1000 claim ────────────────────────────

/// Cloudsteel Kirin reconfigured onto a creature: CR 104.3d — its controller
/// can't lose at 0 life.
#[test]
fn cloudsteel_kirin_keeps_its_bearer_alive() {
    let mut g = pod(2);
    let kirin = ready(&mut g, 0, catalog::cloudsteel_kirin());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::Reconfigure { equipment: kirin, target: Some(bears) }).expect("reconfigure");
    assert!(has_kw(&g, bears, Keyword::Flying));
    g.players[0].life = 0;
    g.check_state_based_actions();
    assert!(g.players[0].is_alive());
}

/// Fell Beast's Shriek: each opponent's chosen creature is tapped and goaded
/// (CR 701.15). The pick is a choice, not a target: a hexproof creature is
/// chosen too (CR 702.11b).
#[test]
fn fell_beasts_shriek_taps_and_goads_one_each() {
    let mut g = pod(3);
    let bears = ready(&mut g, 1, catalog::slippery_bogle());
    let lions = ready(&mut g, 2, catalog::savannah_lions());
    let s = g.add_card_to_hand(0, catalog::fell_beasts_shriek());
    cast(&mut g, s, &[]).expect("cast");
    for id in [bears, lions] {
        let c = g.battlefield_find(id).unwrap();
        assert!(c.tapped);
        assert!(g.goaded_by_player(c, 0));
    }
}

/// CR 702.47a — Fell Beast's Shriek splices onto Lightning Bolt: the Bolt
/// hits and the opponent's creature is tapped and goaded; the Shriek stays
/// in hand.
#[test]
fn cr_702_47a_fell_beasts_shriek_splices_onto_a_bolt() {
    let mut g = pod(2);
    let lions = ready(&mut g, 1, catalog::savannah_lions());
    let shriek = g.add_card_to_hand(0, catalog::fell_beasts_shriek());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 2);
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpellSpliced {
        card_id: bolt,
        splice_cards: vec![shriek],
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Bolt + Shriek");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    let c = g.battlefield_find(lions).unwrap();
    assert!(c.tapped && g.goaded_by_player(c, 0));
    assert!(g.players[0].hand.iter().any(|c| c.id == shriek));
}

/// Scheming Symmetry: both target players search (CR 701.23).
#[test]
fn scheming_symmetry_tutors_two_players() {
    let mut g = pod(3);
    library(&mut g, 0, 3);
    library(&mut g, 2, 3);
    let s = g.add_card_to_hand(0, catalog::scheming_symmetry());
    cast(&mut g, s, &[Target::Player(0), Target::Player(2)]).expect("cast");
    assert!(g.players[0].searched_library_this_turn);
    assert!(g.players[2].searched_library_this_turn);
    assert!(!g.players[1].searched_library_this_turn, "not targeted");
    assert_eq!(g.players[2].library.len(), 3);
}

/// Mindclaw Shaman casts a sorcery from the opponent's hand for free, never a
/// creature card.
#[test]
fn mindclaw_shaman_steals_a_sorcery() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    g.add_card_to_hand(1, catalog::divination());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let m = g.add_card_to_hand(0, catalog::mindclaw_shaman());
    let hand = g.players[0].hand.len();
    cast(&mut g, m, &[]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand + 1, "Shaman left, two drawn");
    assert_eq!(g.players[1].hand.len(), 1);
    assert_eq!(g.players[1].hand[0].definition.name, "Grizzly Bears");
}

/// CR 603.7 — Swashbuckler: sacrifice two Treasures as you attack, and the
/// reflexive trigger gives up to two target creatures double strike — any
/// creatures, a non-attacker included, never more than the count.
#[test]
fn swashbuckler_trades_treasure_for_double_strike() {
    let mut g = pod(2);
    let s = g.add_card_to_hand(0, catalog::swashbuckler_extraordinaire());
    cast(&mut g, s, &[]).expect("cast");
    assert_eq!(tokens_of(&g, 0, "Treasure"), 1);
    g.add_token_to_battlefield(0, &crabomination::game::effects::treasure_token());
    g.clear_sickness(s);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let home = ready(&mut g, 0, catalog::hill_giant());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(2)]));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: s, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(tokens_of(&g, 0, "Treasure"), 0);
    let ds = [s, bears, home].iter().filter(|&&id| has_kw(&g, id, Keyword::DoubleStrike)).count();
    assert_eq!(ds, 2, "two Treasures, up to two targets");
}

/// Toggo: landfall makes a Rock; the equipped creature flings it for 2.
#[test]
fn toggo_throws_rocks() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::toggo_goblin_weaponsmith());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("play");
    drain_stack(&mut g);
    let rock = g.battlefield.iter().find(|c| c.is_token && c.definition.name == "Rock").map(|c| c.id).expect("a Rock");
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: rock, target: bears }).expect("equip {1}");
    drain_stack(&mut g);
    let life = g.players[1].life;
    activate(&mut g, bears, 0, Some(Target::Player(1))).expect("throw");
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.battlefield_find(rock).is_none(), "the Rock is sacrificed");
}

/// Ayara, Widow of the Realm drains by the sacrificed permanent's mana value,
/// and transforms at sorcery speed (CR 701.28).
#[test]
fn ayara_drains_then_transforms() {
    let mut g = pod(2);
    let ayara = ready(&mut g, 0, catalog::ayara_widow_of_the_realm());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    activate(&mut g, ayara, 0, Some(Target::Player(1))).expect("sacrifice the Bears");
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 2, theirs - 2));
    activate(&mut g, ayara, 1, None).expect("transform");
    assert!(g.battlefield_find(ayara).unwrap().transformed);
    assert_eq!(g.computed_permanent(ayara).unwrap().power, 4);
}

// ── Two-card gaps in the 601-1000 claim ─────────────────────────────────────

/// Pir + Toothy: each draw puts 1 + 1 counters on Toothy (CR 614.16); Toothy
/// leaving draws one per counter (last known information, CR 608.2h).
#[test]
fn pir_and_toothy_grow_and_cash_in() {
    let mut g = pod(2);
    library(&mut g, 0, 10);
    ready(&mut g, 0, catalog::pir_imaginative_rascal());
    let toothy = ready(&mut g, 0, catalog::toothy_imaginary_friend());
    let d = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, d, &[]).expect("draw two");
    assert_eq!(g.battlefield_find(toothy).unwrap().counter_count(CounterType::PlusOnePlusOne), 4);
    let hand = g.players[0].hand.len();
    let bounce = g.add_card_to_hand(0, catalog::unsummon());
    cast(&mut g, bounce, &[perm(toothy)]).expect("unsummon");
    assert_eq!(g.players[0].hand.len(), hand + 1 + 4, "Toothy back plus four draws");
}

/// Pir's "a permanent your team controls" reaches a Two-Headed Giant
/// teammate's creature (CR 810.2) and not an opponent's.
#[test]
fn pir_adds_a_counter_on_a_teammates_permanent() {
    let mut g = pod(4);
    g.assign_teams(vec![vec![0, 2], vec![1, 3]]).expect("teams");
    flood(&mut g, 0);
    ready(&mut g, 0, catalog::pir_imaginative_rascal());
    let mate = ready(&mut g, 2, catalog::grizzly_bears());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    for t in [mate, foe] {
        let b = g.add_card_to_hand(0, catalog::battlegrowth());
        cast(&mut g, b, &[perm(t)]).expect("battlegrowth");
    }
    let n = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!((n(&g, mate), n(&g, foe)), (2, 1));
}

/// Sunderflock costs {4} less beside Twinflame Travelers and bounces only
/// non-Elementals; Twinflame doubles its enters trigger (CR 603.2d).
#[test]
fn sunderflock_with_twinflame_travelers() {
    let mut g = pod(2);
    let tt = ready(&mut g, 0, catalog::twinflame_travelers());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::sunderflock());
    g.players[0].mana_pool.add(Color::Blue, 5);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: s, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{7}{U}{U} less 4");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(tt).is_some() && g.battlefield_find(s).is_some());
}

/// Edge of the Divinity pumps a white creature +1/+2 and leaves a green one.
#[test]
fn edge_of_the_divinity_reads_the_hosts_color() {
    let mut g = pod(2);
    let lions = ready(&mut g, 0, catalog::savannah_lions());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let e1 = g.add_card_to_hand(0, catalog::edge_of_the_divinity());
    cast(&mut g, e1, &[perm(lions)]).expect("on the Lions");
    let e2 = g.add_card_to_hand(0, catalog::edge_of_the_divinity());
    cast(&mut g, e2, &[perm(bears)]).expect("on the Bears");
    let l = g.computed_permanent(lions).unwrap();
    assert_eq!((l.power, l.toughness), (3, 3));
    let b = g.computed_permanent(bears).unwrap();
    assert_eq!((b.power, b.toughness), (2, 2));
}

/// Dark Dabbling shields the target, draws, and with spell mastery shields
/// the rest of your creatures (CR 701.19).
#[test]
fn dark_dabbling_regenerates_the_team_with_mastery() {
    let mut g = pod(2);
    library(&mut g, 0, 2);
    g.add_card_to_graveyard(0, catalog::divination());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::savannah_lions());
    let foe = ready(&mut g, 1, catalog::hill_giant());
    let d = g.add_card_to_hand(0, catalog::dark_dabbling());
    let hand = g.players[0].hand.len();
    cast(&mut g, d, &[perm(a)]).expect("cast");
    assert_eq!(g.players[0].hand.len(), hand);
    for id in [a, b] {
        assert_eq!(g.battlefield_find(id).unwrap().regeneration_shields, 1);
    }
    assert_eq!(g.battlefield_find(foe).unwrap().regeneration_shields, 0);
}

/// Shay Cormac: a targeted opposing creature gets a bounty counter, and its
/// death grows Shay by two.
#[test]
fn shay_cormac_collects_bounties() {
    let mut g = pod(2);
    let shay = ready(&mut g, 0, catalog::shay_cormac());
    let foe = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(foe)]).expect("bolt");
    assert!(g.battlefield_find(foe).is_none());
    assert_eq!(g.battlefield_find(shay).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Termination Facilitator: a bountied opposing creature dealt any damage is
/// destroyed.
#[test]
fn termination_facilitator_finishes_marked_targets() {
    let mut g = pod(2);
    let tf = ready(&mut g, 0, catalog::termination_facilitator());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    activate(&mut g, tf, 0, Some(perm(wurm))).expect("mark");
    assert_eq!(g.battlefield_find(wurm).unwrap().counter_count(CounterType::Bounty), 1);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[perm(wurm)]).expect("bolt");
    assert!(g.battlefield_find(wurm).is_none(), "3 damage to a 6/4 still kills it");
}

/// Scrapyard Recombiner enters with two counters (CR 702.43a) and can feed
/// itself to fetch a Construct.
#[test]
fn scrapyard_recombiner_fetches_a_construct() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::scrapyard_recombiner());
    let s = g.add_card_to_hand(0, catalog::scrapyard_recombiner());
    cast(&mut g, s, &[]).expect("cast");
    assert_eq!(g.battlefield_find(s).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    g.clear_sickness(s);
    activate(&mut g, s, 0, None).expect("sacrifice itself");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Scrapyard Recombiner"));
}

/// Pia's Revolution: a destroyed nontoken artifact comes back to hand unless
/// the opponent takes 3.
#[test]
fn pias_revolution_punishes_artifact_removal() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::pias_revolution());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let life = g.players[1].life;
    let sh = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, sh, &[perm(ring)]).expect("shatter");
    let back = g.players[0].hand.iter().any(|c| c.id == ring);
    assert!(back != (g.players[1].life == life - 3), "exactly one: hand or 3 damage");
}

/// Goldnight Castigator doubles damage to its controller and to itself
/// (CR 614.5), not to other players.
#[test]
fn goldnight_castigator_doubles_damage_to_you_and_it() {
    let mut g = pod(2);
    let gc = ready(&mut g, 0, catalog::goldnight_castigator());
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    for t in [Target::Player(0), Target::Player(1), perm(gc)] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, &[t]).expect("bolt");
    }
    assert_eq!(g.players[0].life, mine - 6);
    assert_eq!(g.players[1].life, theirs - 3);
    assert_eq!(g.battlefield_find(gc).unwrap().damage, 6);
}

/// Nine Lives: each damage event to you is prevented for one incarnation
/// counter; the ninth exiles it and its leaving loses you the game
/// (CR 104.3e).
#[test]
fn nine_lives_counts_down_to_a_loss() {
    let mut g = pod(3);
    let nl = ready(&mut g, 0, catalog::nine_lives());
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(0)]).expect("bolt");
    assert_eq!(g.players[0].life, life, "prevented");
    assert_eq!(g.battlefield_find(nl).unwrap().counter_count(CounterType::Incarnation), 1);
    g.battlefield_find_mut(nl).unwrap().counters.insert(CounterType::Incarnation, 8);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(0)]).expect("bolt");
    g.check_state_based_actions();
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(nl).is_none(), "nine counters exile it");
    assert!(!g.players[0].is_alive());
}

/// CR 615.12 — Nine Lives *prevents* (CR 615), so under "damage can't be
/// prevented" the bolt is dealt and no counter goes on (it was a replacement).
#[test]
fn nine_lives_cannot_prevent_unpreventable_damage() {
    let mut g = pod(2);
    let nl = ready(&mut g, 0, catalog::nine_lives());
    g.damage_cant_be_prevented_this_turn = true;
    let life = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, &[Target::Player(0)]).expect("bolt");
    assert_eq!(g.players[0].life, life - 3);
    assert_eq!(g.battlefield_find(nl).unwrap().counter_count(CounterType::Incarnation), 0);
}

/// Bag of Holding: a discarded card is exiled with it, and sacrificing it
/// returns that card to hand.
#[test]
fn bag_of_holding_keeps_discards() {
    let mut g = pod(2);
    library(&mut g, 0, 3);
    let bag = ready(&mut g, 0, catalog::bag_of_holding());
    activate(&mut g, bag, 0, None).expect("loot");
    assert!(g.players[0].graveyard.is_empty(), "the discard is exiled");
    assert_eq!(g.exile.len(), 1);
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(bag).unwrap().tapped = false;
    activate(&mut g, bag, 1, None).expect("open the bag");
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(g.exile.is_empty());
}

/// Pair o' Dice Lost: a 2 and a 3 (CR 706) return up to 5 total mana value —
/// asking for the Giant (4) and the Wurm (6) gets only the Giant.
#[test]
fn pair_o_dice_lost_returns_by_total_mana_value() {
    let mut g = pod(2);
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let giant = g.add_card_to_graveyard(0, catalog::hill_giant());
    let p = g.add_card_to_hand(0, catalog::pair_o_dice_lost());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::DieRoll(2),
        DecisionAnswer::DieRoll(3),
        DecisionAnswer::Cards(vec![giant, wurm]),
    ]));
    cast(&mut g, p, &[]).expect("cast");
    assert!(g.players[0].hand.iter().any(|c| c.id == giant));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == wurm), "6 is over the 5 rolled");
    assert!(g.exile.iter().any(|c| c.id == p));
}

/// Malik, Grim Manipulator: you name their best creature, they name their
/// worst, and both are sacrificed (CR 701.21a); each sacrifice by an opponent
/// makes a Treasure.
#[test]
fn malik_takes_two_and_profits() {
    let mut g = pod(2);
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let m = g.add_card_to_hand(0, catalog::malik_grim_manipulator());
    cast(&mut g, m, &[]).expect("cast");
    assert!(g.battlefield_find(bears).is_none() && g.battlefield_find(wurm).is_none());
    assert_eq!(tokens_of(&g, 0, "Treasure"), 2);
}

/// Gisa's Favorite Shovel: the equipped attacker makes the defender sacrifice
/// a creature, and that sacrifice digs up a Walker.
#[test]
fn gisas_favorite_shovel_digs_a_walker() {
    let mut g = pod(2);
    let shovel = ready(&mut g, 0, catalog::gisas_favorite_shovel());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let foe = ready(&mut g, 1, catalog::savannah_lions());
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: shovel, target: bears }).expect("equip {4}");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bears).unwrap().power, 4);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bears, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none());
    assert_eq!(tokens_of(&g, 0, "Walker"), 1);
}

/// Wernog, Rider's Chaplain — "each opponent may investigate. Each opponent who
/// doesn't loses 1 life. You investigate X times, where X is one plus the
/// number of opponents who investigated this way." (CR 101.4: every choice in
/// APNAP order, then the actions.)
#[test]
fn wernog_decliners_lose_life_and_acceptors_pay_you_a_clue() {
    let mut g = pod(4);
    let wernog = g.add_card_to_hand(0, catalog::wernog_riders_chaplain());
    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    // Seat 1 accepts, seats 2 and 3 decline.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(false),
    ]));
    cast_x(&mut g, wernog, &[], None).unwrap();
    drain_stack(&mut g);
    let clues = |g: &GameState, p: usize| {
        g.battlefield.iter().filter(|c| c.controller == p && c.definition.name == "Clue").count()
    };
    assert_eq!((clues(&g, 0), clues(&g, 1), clues(&g, 2)), (2, 1, 0));
    assert_eq!(g.players[1].life, life[1], "the acceptor loses nothing");
    assert_eq!((g.players[2].life, g.players[3].life), (life[2] - 1, life[3] - 1));
    assert_eq!(g.players[0].life, life[0]);
}
