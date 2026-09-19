//! Commander batch — Umbris, Fear Manifest and its EDHREC companions
//! (`decks::cmdr_umbris`).

use crabomination::card::{CardId, CardType, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{AutoDecider, DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn game(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    stock_libraries(&mut g, 20);
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 10);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>, extra: Vec<Target>, x: Option<u32>) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: extra,
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast_mode(g: &mut GameState, id: CardId, target: Option<Target>, mode: usize) {
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: Some(mode),
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })?;
    drain_stack(g);
    Ok(())
}

/// Put `card` on seat 0's battlefield and resolve its ETB triggers.
fn enter(g: &mut GameState, card: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, card);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

/// Seat 0 attacks seat `defender` with `attackers`, no blocks, through combat.
fn attack(g: &mut GameState, attackers: &[CardId], defender: usize) {
    for &a in attackers {
        g.clear_sickness(a);
    }
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(
        attackers
            .iter()
            .map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) })
            .collect(),
    )
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

fn end_step(g: &mut GameState) {
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(g);
}

fn upkeep(g: &mut GameState) {
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(g);
}

fn bolt(g: &mut GameState, seat: usize, target: Target) {
    let b = g.add_card_to_hand(seat, catalog::lightning_bolt());
    g.players[seat].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: b,
        target: Some(target),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(g);
    g.priority.player_with_priority = 0;
}

fn power(g: &GameState, id: CardId) -> i32 {
    g.computed_permanent(id).unwrap().power
}

fn keywords(g: &GameState, id: CardId) -> Vec<Keyword> {
    g.computed_permanent(id).unwrap().keywords().to_vec()
}

fn named_on_battlefield(g: &GameState, name: &str, controller: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name && c.controller == controller).count()
}

/// Seed `seat`'s library top with `cards` (top first), ahead of the stock.
fn stack_library(g: &mut GameState, seat: usize, cards: Vec<crabomination::card::CardDefinition>) {
    let stock: Vec<_> = std::mem::take(&mut *g.players[seat].library);
    for c in cards {
        g.add_card_to_library(seat, c);
    }
    g.players[seat].library.extend(stock);
}

/// The printed bodies whose static keywords are the whole point (the triggers
/// are tested per card below).
#[test]
fn cmdr_umbris_printed_bodies() {
    type Row = (fn() -> crabomination::card::CardDefinition, i32, i32, &'static [Keyword]);
    let rows: &[Row] = &[
        (catalog::aboleth_spawn, 2, 3, &[Keyword::Flash]),
        (catalog::vashta_nerada, 1, 1, &[Keyword::Indestructible, Keyword::Shadow]),
        (catalog::forgotten_creation, 3, 3, &[Keyword::Skulk]),
        (catalog::the_weaver_king, 3, 6, &[Keyword::Shadow]),
        (catalog::the_master_of_lake_town, 3, 2, &[Keyword::Deathtouch]),
        (catalog::elder_brain, 6, 6, &[Keyword::Menace]),
        (catalog::defiler_of_flesh, 4, 4, &[Keyword::Menace]),
        (catalog::gollum_the_abandoned, 2, 2, &[Keyword::CantBlock]),
        (catalog::wharf_infiltrator, 1, 1, &[Keyword::Skulk]),
        (catalog::brainstealer_dragon, 6, 6, &[Keyword::Flying]),
    ];
    for (f, p, t, kws) in rows {
        let mut g = game(2);
        let id = g.add_card_to_battlefield(0, f());
        let cp = g.computed_permanent(id).unwrap();
        let name = f().name;
        assert_eq!((cp.power, cp.toughness), (*p, *t), "{name} P/T");
        for k in *kws {
            assert!(cp.keywords().contains(k), "{name} has {k:?}");
        }
    }
    assert!(
        catalog::aboleth_spawn()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::Ward(_))),
        "Aboleth Spawn has ward"
    );
}

/// Umbris exiles until a land and grows with every card its opponents own in
/// exile.
#[test]
fn cmdr_umbris_exiles_until_land_and_grows() {
    let mut g = game(2);
    stack_library(
        &mut g,
        1,
        vec![catalog::grizzly_bears(), catalog::grizzly_bears(), catalog::island()],
    );
    flood(&mut g, 0);
    let umbris = g.add_card_to_hand(0, catalog::umbris_fear_manifest());
    cast(&mut g, umbris, None, vec![], None);
    let exiled = g.exile.iter().filter(|c| c.owner == 1).count();
    assert_eq!(exiled, 3, "two Bears then the Island");
    assert_eq!(power(&g, umbris), 4, "1 + three opponent-owned exiled cards");
}

/// Gollum, Riddle Master answers the chosen parity only, cycling its modes.
#[test]
fn cmdr_umbris_gollum_riddle_master_answers_odd_spells() {
    let mut g = game(2);
    flood(&mut g, 0);
    let gollum = g.add_card_to_hand(0, catalog::gollum_riddle_master());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    cast(&mut g, gollum, None, vec![], None);
    g.decider = Box::new(AutoDecider);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Lightning Bolt (MV 1, odd) triggers; Lightning Helix (MV 2) doesn't.
    bolt(&mut g, 1, Target::Permanent(bears));
    let counters = g.battlefield_find(gollum).unwrap().counter_count(CounterType::PlusOnePlusOne);
    let helix = g.add_card_to_hand(1, catalog::lightning_helix());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: helix,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("helix");
    drain_stack(&mut g);
    let life0 = g.players[0].life;
    assert_eq!(counters, 1, "the odd spell took the first unchosen mode");
    assert_eq!(life0, 17, "Helix (even) triggered nothing — only its 3 damage");
    bolt(&mut g, 1, Target::Player(0));
    assert_eq!(g.players[1].life, 20 + 3 - 2, "second odd spell drained each opponent");
    assert_eq!(g.players[0].life, 17 - 3 + 2);
}

/// Gollum the Abandoned hoses a graveyard card and drains; it rebuys itself
/// by sacrificing a creature.
#[test]
fn cmdr_umbris_gollum_the_abandoned_etb_and_rebuy() {
    let mut g = game(2);
    let victim = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let gollum = enter(&mut g, catalog::gollum_the_abandoned());
    assert!(g.exile.iter().any(|c| c.id == victim), "graveyard card exiled");
    assert_eq!(g.players[1].life, 18);
    // Back from the graveyard for {2} and a creature.
    g.battlefield.retain(|c| c.id != gollum);
    let dead = g.add_card_to_graveyard(0, catalog::gollum_the_abandoned());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    activate(&mut g, dead, 0, None).expect("rebuy");
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "back in hand");
    assert!(g.battlefield_find(fodder).is_none(), "the creature was sacrificed");
}

/// The Weaver King mills the damaged player and steals a creature card it
/// milled.
#[test]
fn cmdr_umbris_weaver_king_mills_and_steals() {
    let mut g = game(2);
    stack_library(
        &mut g,
        1,
        vec![catalog::island(), catalog::grizzly_bears(), catalog::island()],
    );
    let king = g.add_card_to_battlefield(0, catalog::the_weaver_king());
    attack(&mut g, &[king], 1);
    assert_eq!(g.players[1].life, 17);
    assert_eq!(named_on_battlefield(&g, "Grizzly Bears", 0), 1, "the milled Bears are yours");
}

/// The Master of Lake-town mills whoever loses life; on death it draws per
/// seven-card graveyard.
#[test]
fn cmdr_umbris_master_of_lake_town_mills_and_draws() {
    let mut g = game(2);
    let master = g.add_card_to_battlefield(0, catalog::the_master_of_lake_town());
    let lib = g.players[1].library.len();
    bolt(&mut g, 0, Target::Player(1));
    assert_eq!(g.players[1].library.len(), lib - 3, "lost 3 → milled 3");
    for _ in 0..8 {
        g.add_card_to_graveyard(0, catalog::island());
        g.add_card_to_graveyard(1, catalog::island());
    }
    let hand = g.players[0].hand.len();
    bolt(&mut g, 1, Target::Permanent(master));
    assert!(g.battlefield_find(master).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 2, "two graveyards hold seven or more");
}

/// Inside Information exiles X, and a spell cast from it costs life instead
/// of mana.
#[test]
fn cmdr_umbris_inside_information_casts_for_life() {
    let mut g = game(2);
    stack_library(&mut g, 1, vec![catalog::grizzly_bears(), catalog::island()]);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let spell = g.add_card_to_hand(0, catalog::inside_information());
    cast(&mut g, spell, Some(Target::Player(1)), vec![], Some(2));
    let bears = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    assert_eq!(g.players[0].mana_pool.total(), 0);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile for life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "paid the Bears' mana value in life");
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Captain N'ghathrod: Horrors get menace, a Horror's hit mills, and the end
/// step reanimates a milled creature under your control.
#[test]
fn cmdr_umbris_captain_nghathrod_mills_and_reanimates() {
    let mut g = game(2);
    stack_library(
        &mut g,
        1,
        vec![catalog::grizzly_bears(), catalog::island(), catalog::island()],
    );
    let captain = g.add_card_to_battlefield(0, catalog::captain_nghathrod());
    let other = g.add_card_to_battlefield(0, catalog::nemesis_of_reason());
    assert!(keywords(&g, other).contains(&Keyword::Menace), "Horrors you control have menace");
    attack(&mut g, &[captain], 1);
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"));
    end_step(&mut g);
    assert_eq!(named_on_battlefield(&g, "Grizzly Bears", 0), 1, "the milled Bears come to you");
}

/// Grazilaxx draws once for a whole batch of connecting attackers.
#[test]
fn cmdr_umbris_grazilaxx_draws_once_per_damage_batch() {
    let mut g = game(2);
    let graz = g.add_card_to_battlefield(0, catalog::grazilaxx_illithid_scholar());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    attack(&mut g, &[graz, bears], 1);
    assert_eq!(g.players[1].life, 15);
    assert_eq!(g.players[0].hand.len(), hand + 1, "one draw for the batch");
}

/// Ancient Cellarspawn discounts a Horror and bills the opponent the gap.
#[test]
fn cmdr_umbris_ancient_cellarspawn_discount_drains() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::ancient_cellarspawn());
    let nemesis = g.add_card_to_hand(0, catalog::nemesis_of_reason());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, nemesis, None, vec![], None);
    assert!(g.battlefield_find(nemesis).is_some(), "cast for four mana");
    assert_eq!(g.players[1].life, 19, "mana value 5, 4 spent: loses 1");
}

/// Sludge Monster's slime counter strips a non-Horror to a vanilla 2/2.
#[test]
fn cmdr_umbris_sludge_monster_slimes_a_non_horror() {
    let mut g = game(2);
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    enter(&mut g, catalog::sludge_monster());
    assert_eq!(g.battlefield_find(angel).unwrap().counter_count(CounterType::Slime), 1);
    let cp = g.computed_permanent(angel).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(!cp.keywords().contains(&Keyword::Flying), "lost all abilities");
    // A slimed Horror keeps everything.
    let brain = g.add_card_to_battlefield(1, catalog::elder_brain());
    g.battlefield_find_mut(brain).unwrap().add_counters(CounterType::Slime, 1);
    assert!(keywords(&g, brain).contains(&Keyword::Menace));
    assert_eq!(power(&g, brain), 6);
}

/// Endless Evil copies its host each upkeep and returns when a Horror host
/// dies.
#[test]
fn cmdr_umbris_endless_evil_copies_and_returns() {
    let mut g = game(2);
    flood(&mut g, 0);
    let host = g.add_card_to_battlefield(0, catalog::wharf_infiltrator());
    let aura = g.add_card_to_hand(0, catalog::endless_evil());
    cast(&mut g, aura, Some(Target::Permanent(host)), vec![], None);
    upkeep(&mut g);
    assert_eq!(named_on_battlefield(&g, "Wharf Infiltrator", 0), 2, "a token copy");
    bolt(&mut g, 1, Target::Permanent(host));
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "Endless Evil came back");
}

/// Zellix's mill ability makes one Horror for a creature-bearing mill.
#[test]
fn cmdr_umbris_zellix_mill_makes_a_horror() {
    let mut g = game(2);
    stack_library(
        &mut g,
        1,
        vec![catalog::grizzly_bears(), catalog::grizzly_bears(), catalog::island()],
    );
    let zellix = g.add_card_to_battlefield(0, catalog::zellix_sanity_flayer());
    g.clear_sickness(zellix);
    flood(&mut g, 0);
    activate(&mut g, zellix, 0, Some(Target::Player(1))).expect("mill 3");
    assert_eq!(g.players[1].graveyard.len(), 3);
    assert_eq!(named_on_battlefield(&g, "Horror", 0), 1, "one token for the batch");
}

/// Nemesis of Reason mills ten on the attack.
#[test]
fn cmdr_umbris_nemesis_of_reason_mills_ten() {
    let mut g = game(2);
    let nemesis = g.add_card_to_battlefield(0, catalog::nemesis_of_reason());
    attack(&mut g, &[nemesis], 1);
    assert_eq!(g.players[1].graveyard.len(), 10);
}

/// Wharf Infiltrator turns a discarded creature card into a 3/2.
#[test]
fn cmdr_umbris_wharf_infiltrator_discard_makes_eldrazi() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::wharf_infiltrator());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let rot = g.add_card_to_hand(0, catalog::mind_rot());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, rot, Some(Target::Player(0)), vec![], None);
    assert_eq!(named_on_battlefield(&g, "Eldrazi Horror", 0), 1);
}

/// Yarok's Fenlurker takes a card from every opponent's hand (4 players).
#[test]
fn cmdr_umbris_yaroks_fenlurker_hits_each_opponent() {
    let mut g = game(4);
    for seat in 1..4 {
        g.add_card_to_hand(seat, catalog::island());
    }
    enter(&mut g, catalog::yaroks_fenlurker());
    for seat in 1..4 {
        assert!(g.players[seat].hand.is_empty(), "seat {seat} exiled its card");
    }
}

/// Elder Brain exiles the defender's hand, refills it, and lets you play the
/// exiled cards.
#[test]
fn cmdr_umbris_elder_brain_swaps_the_hand() {
    let mut g = game(2);
    let a = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    let brain = g.add_card_to_battlefield(0, catalog::elder_brain());
    attack(&mut g, &[brain], 1);
    assert_eq!(g.players[1].hand.len(), 2, "drew that many");
    let exiled = g.exile.iter().find(|c| c.id == a).expect("Bears exiled");
    assert_eq!(exiled.may_play_until.map(|p| p.player), Some(0), "you may play it");
}

/// Mind Flayer steals a creature.
#[test]
fn cmdr_umbris_mind_flayer_steals() {
    let mut g = game(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    enter(&mut g, catalog::mind_flayer());
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Toxrill slimes the other side each end step, shrinking and killing.
#[test]
fn cmdr_umbris_toxrill_slimes_and_makes_slugs() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::toxrill_the_corrosive());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elves = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    end_step(&mut g);
    assert_eq!(power(&g, bears), 1, "2/2 with one slime counter");
    assert!(g.battlefield_find(elves).is_none(), "the 1/1 died");
    assert_eq!(named_on_battlefield(&g, "Slug", 0), 1);
    end_step(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "two counters kill the Bears");
}

/// Falthis gives your commander menace and deathtouch, and nothing else.
#[test]
fn cmdr_umbris_falthis_arms_commanders_only() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::falthis_shadowcat_familiar());
    let cmdr = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let k = keywords(&g, cmdr);
    assert!(k.contains(&Keyword::Menace) && k.contains(&Keyword::Deathtouch));
    assert!(!keywords(&g, plain).contains(&Keyword::Deathtouch));
}

/// Dread Presence draws off a Swamp drop.
#[test]
fn cmdr_umbris_dread_presence_swamp_draws() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::dread_presence());
    let swamp = g.add_card_to_hand(0, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::PlayLand(swamp)).expect("swamp");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "played one, drew one");
    assert_eq!(g.players[0].life, 19);
}

/// Brainstealer Dragon takes each opponent's top card; casting a stolen
/// permanent bills its owner.
#[test]
fn cmdr_umbris_brainstealer_dragon_steals_and_bills() {
    let mut g = game(3);
    stack_library(&mut g, 1, vec![catalog::grizzly_bears()]);
    stack_library(&mut g, 2, vec![catalog::island()]);
    g.add_card_to_battlefield(0, catalog::brainstealer_dragon());
    end_step(&mut g);
    let bears = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    assert!(g.exile.iter().any(|c| c.owner == 2), "seat 2's top card too");
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let bears_card = g.exile.iter().find(|c| c.id == bears).unwrap();
    assert_eq!(bears_card.may_play_until.map(|p| p.player), Some(0), "you may play it");
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the stolen Bears with blue mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
    assert_eq!(g.players[1].life, 18, "its owner lost its mana value");
}

/// Grell Philosopher borrows an opponent's artifact's activated abilities.
#[test]
fn cmdr_umbris_grell_philosopher_borrows_abilities() {
    let mut g = game(2);
    g.add_card_to_battlefield(1, catalog::mind_stone());
    let grell = enter(&mut g, catalog::grell_philosopher());
    assert!(!g.battlefield_find(grell).unwrap().granted_activated_eot.is_empty());
}

/// Intellect Devourer holds a card from each opponent that you may cast.
#[test]
fn cmdr_umbris_intellect_devourer_steals_a_hand_card() {
    let mut g = game(2);
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    enter(&mut g, catalog::intellect_devourer());
    assert!(g.exile.iter().any(|c| c.id == bears));
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast it with black mana");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Defiler of Flesh pumps a creature on each black permanent spell.
#[test]
fn cmdr_umbris_defiler_of_flesh_pumps() {
    let mut g = game(2);
    let defiler = g.add_card_to_battlefield(0, catalog::defiler_of_flesh());
    let lurker = g.add_card_to_hand(0, catalog::yaroks_fenlurker());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, lurker, None, vec![], None);
    let total: i32 = [defiler, lurker].iter().map(|&id| power(&g, id)).sum();
    assert_eq!(total, 4 + 1 + 1, "one creature got +1/+1");
}

/// Arvinox is a creature only while you control three permanents you don't
/// own; its end step takes the bottom of each opponent's library.
#[test]
fn cmdr_umbris_arvinox_animates_and_steals_bottoms() {
    let mut g = game(2);
    let arv = g.add_card_to_battlefield(0, catalog::arvinox_the_mind_flail());
    assert!(!g.computed_permanent(arv).unwrap().card_types().contains(&CardType::Creature));
    for _ in 0..3 {
        let id = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        g.battlefield_find_mut(id).unwrap().controller = 0;
    }
    assert!(g.computed_permanent(arv).unwrap().card_types().contains(&CardType::Creature));
    let bottom = g.players[1].library.last().unwrap().id;
    end_step(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bottom), "bottom card exiled");
}

/// Haunt of the Dead Marshes only climbs back under a legend.
#[test]
fn cmdr_umbris_haunt_needs_a_legend() {
    let mut g = game(2);
    let haunt = g.add_card_to_graveyard(0, catalog::haunt_of_the_dead_marshes());
    flood(&mut g, 0);
    assert!(activate(&mut g, haunt, 0, None).is_err(), "no legendary creature");
    g.add_card_to_battlefield(0, catalog::gollum_the_abandoned());
    activate(&mut g, haunt, 0, None).expect("with a legend");
    assert!(g.battlefield_find(haunt).unwrap().tapped, "returns tapped");
}

/// Grimdancer enters with the chosen pair of keyword counters.
#[test]
fn cmdr_umbris_grimdancer_picks_two_counters() {
    let mut g = game(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2)]));
    let id = enter(&mut g, catalog::grimdancer());
    let k = keywords(&g, id);
    assert!(k.contains(&Keyword::Deathtouch) && k.contains(&Keyword::Lifelink));
    assert!(!k.contains(&Keyword::Menace));
}

/// Braids: each opponent who can't match the sacrifice loses 2 and you draw.
#[test]
fn cmdr_umbris_braids_punishes_each_opponent() {
    let mut g = game(3);
    g.add_card_to_battlefield(0, catalog::braids_arisen_nightmare());
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    let hand = g.players[0].hand.len();
    end_step(&mut g);
    assert!(g.battlefield_find(stone).is_none(), "sacrificed the artifact");
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18));
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Abyssal Harvester copies a creature that died this turn as a Nightmare.
#[test]
fn cmdr_umbris_abyssal_harvester_copies_the_fallen() {
    let mut g = game(2);
    let harvester = g.add_card_to_battlefield(0, catalog::abyssal_harvester());
    g.clear_sickness(harvester);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    bolt(&mut g, 0, Target::Permanent(bears));
    activate(&mut g, harvester, 0, Some(Target::Permanent(bears))).expect("harvest");
    assert!(g.exile.iter().any(|c| c.id == bears));
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears" && c.controller == 0)
        .expect("token copy");
    assert!(token.definition.subtypes.creature_types.contains(&CreatureType::Nightmare));
}

/// Forgotten Creation wheels your hand on upkeep.
#[test]
fn cmdr_umbris_forgotten_creation_wheels() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::forgotten_creation());
    let a = g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    upkeep(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == a));
}

/// Vashta Nerada grows only on turns a creature died.
#[test]
fn cmdr_umbris_vashta_nerada_morbid() {
    let mut g = game(2);
    let vashta = g.add_card_to_battlefield(0, catalog::vashta_nerada());
    end_step(&mut g);
    assert_eq!(power(&g, vashta), 1, "nothing died");
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    bolt(&mut g, 0, Target::Permanent(bears));
    end_step(&mut g);
    assert_eq!(power(&g, vashta), 2);
}

/// The flicker instants: each returns its targets, fresh.
#[test]
fn cmdr_umbris_flicker_spells() {
    // Ghostly Flicker — two targets, both return.
    let mut g = game(2);
    flood(&mut g, 0);
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let s = g.add_card_to_hand(0, catalog::ghostly_flicker());
    cast(&mut g, s, Some(Target::Permanent(a)), vec![Target::Permanent(b)], None);
    assert!(g.battlefield_find(a).is_some() && g.battlefield_find(b).is_some());
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);

    // Displace — up to two creatures.
    let s = g.add_card_to_hand(0, catalog::displace());
    g.battlefield_find_mut(a).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    cast(&mut g, s, Some(Target::Permanent(a)), vec![], None);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);

    // Planar Incision — returns with a +1/+1 counter.
    let s = g.add_card_to_hand(0, catalog::planar_incision());
    cast(&mut g, s, Some(Target::Permanent(a)), vec![], None);
    assert_eq!(g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    // Essence Flux — a Spirit comes back with a counter.
    let sailor = g.add_card_to_battlefield(0, catalog::spectral_sailor());
    let s = g.add_card_to_hand(0, catalog::essence_flux());
    cast(&mut g, s, Some(Target::Permanent(sailor)), vec![], None);
    assert_eq!(g.battlefield_find(sailor).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);

    // Hide on the Ceiling — gone until the next end step.
    let s = g.add_card_to_hand(0, catalog::hide_on_the_ceiling());
    cast(&mut g, s, Some(Target::Permanent(a)), vec![], Some(1));
    assert!(g.battlefield_find(a).is_none(), "exiled");
    end_step(&mut g);
    assert!(g.battlefield_find(a).is_some(), "back at the end step");
}

/// The counterspells, each against a Lightning Bolt on the stack.
#[test]
fn cmdr_umbris_counterspells() {
    type Row = (&'static str, fn() -> crabomination::card::CardDefinition, i32, usize);
    // (name, card, bolt caster's life after, bolt caster's graveyard growth from mill)
    let rows: &[Row] = &[
        ("Miscast", catalog::miscast, 20, 0),
        ("Didn't Say Please", catalog::didnt_say_please, 20, 3),
        ("Countersquall", catalog::countersquall, 18, 0),
    ];
    for (name, card, life, milled) in rows {
        let mut g = game(2);
        let b = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: b,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        let counter = g.add_card_to_hand(0, card());
        g.players[0].mana_pool.add(Color::Blue, 2);
        g.players[0].mana_pool.add(Color::Black, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 0;
        let gy = g.players[1].graveyard.len();
        g.perform_action(GameAction::CastSpell {
            card_id: counter,
            target: Some(Target::Permanent(b)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect(name);
        drain_stack(&mut g);
        assert_eq!(g.players[0].life, 20, "{name}: the Bolt was countered");
        assert_eq!(g.players[1].life, *life, "{name}: controller's life");
        assert_eq!(g.players[1].graveyard.len(), gy + 1 + milled, "{name}: graveyard");
    }
}

/// Blot Out takes the opponent's most expensive creature.
#[test]
fn cmdr_umbris_blot_out_takes_the_biggest() {
    let mut g = game(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::blot_out());
    cast(&mut g, s, Some(Target::Player(1)), vec![], None);
    assert!(g.battlefield_find(angel).is_none() && g.battlefield_find(bears).is_some());
}

/// Spoils of Blood makes an X/X for the turn's deaths.
#[test]
fn cmdr_umbris_spoils_of_blood_counts_deaths() {
    let mut g = game(2);
    for _ in 0..2 {
        let v = g.add_card_to_battlefield(1, catalog::grizzly_bears());
        bolt(&mut g, 0, Target::Permanent(v));
    }
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::spoils_of_blood());
    cast(&mut g, s, None, vec![], None);
    let horror = g.battlefield.iter().find(|c| c.definition.name == "Horror").unwrap().id;
    assert_eq!(power(&g, horror), 2);
}

/// Drown in Dreams with a commander out runs both modes.
#[test]
fn cmdr_umbris_drown_in_dreams_both_modes_with_a_commander() {
    let mut g = game(2);
    let cmdr = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::drown_in_dreams());
    let hand = g.players[1].hand.len();
    cast(&mut g, s, Some(Target::Player(1)), vec![Target::Player(1)], Some(2));
    assert_eq!(g.players[1].hand.len(), hand + 2, "drew X");
    assert_eq!(g.players[1].graveyard.len(), 4, "milled twice X");
}

/// Szat's Will's graveyard mode makes Thrulls for the biggest exiled body.
#[test]
fn cmdr_umbris_szats_will_exiles_graveyards_for_thrulls() {
    let mut g = game(2);
    g.add_card_to_graveyard(1, catalog::serra_angel());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::szats_will());
    cast_mode(&mut g, s, None, 1);
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(named_on_battlefield(&g, "Thrull", 0), 4, "Serra Angel's power");
}

/// Essence Harvest drains by your biggest creature's power.
#[test]
fn cmdr_umbris_essence_harvest_drains_greatest_power() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::serra_angel());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::essence_harvest());
    cast(&mut g, s, Some(Target::Player(1)), vec![], None);
    assert_eq!((g.players[1].life, g.players[0].life), (16, 24));
}

/// Nightmare Unmaking's "greater than hand size" mode.
#[test]
fn cmdr_umbris_nightmare_unmaking_by_hand_size() {
    let mut g = game(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elves = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    g.add_card_to_hand(0, catalog::island());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::nightmare_unmaking());
    cast_mode(&mut g, s, None, 0);
    assert!(g.battlefield_find(bears).is_none(), "power 2 > one card in hand");
    assert!(g.battlefield_find(elves).is_some());
}

/// Dream Harvest exiles to five mana value and casts free.
#[test]
fn cmdr_umbris_dream_harvest_free_casts() {
    let mut g = game(2);
    stack_library(
        &mut g,
        1,
        vec![catalog::grizzly_bears(), catalog::divination(), catalog::island()],
    );
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::dream_harvest());
    cast(&mut g, s, None, vec![], None);
    let bears = g.exile.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    assert!(g.exile.iter().any(|c| c.definition.name == "Divination"));
    assert!(!g.exile.iter().any(|c| c.definition.name == "Island"), "stopped at 5");
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("free");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Final Act's chosen modes: wipe creatures and strip the opponent's counters.
#[test]
fn cmdr_umbris_final_act_modes() {
    let mut g = game(2);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[1].poison_counters = 4;
    g.players[1].energy = 3;
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::final_act());
    g.perform_action(GameAction::CastSpellSpree {
        card_id: s,
        spree_modes: vec![0, 4],
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("final act");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!((g.players[1].poison_counters, g.players[1].energy), (0, 0));
}

/// Mandate of Abaddon kills everything smaller than your chosen creature.
#[test]
fn cmdr_umbris_mandate_of_abaddon() {
    let mut g = game(2);
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::serra_angel());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::mandate_of_abaddon());
    cast(&mut g, s, Some(Target::Permanent(angel)), vec![], None);
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.battlefield_find(big).is_some() && g.battlefield_find(angel).is_some());
}

/// Rite of Consumption flings a sacrificed creature and gains the damage.
#[test]
fn cmdr_umbris_rite_of_consumption() {
    let mut g = game(2);
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::rite_of_consumption());
    cast(&mut g, s, Some(Target::Player(1)), vec![], None);
    assert!(g.battlefield_find(angel).is_none());
    assert_eq!((g.players[1].life, g.players[0].life), (16, 24));
}

/// Distant Melody draws per permanent of the named type.
#[test]
fn cmdr_umbris_distant_melody_counts_the_type() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::nemesis_of_reason());
    g.add_card_to_battlefield(0, catalog::elder_brain());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::distant_melody());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Horror)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, s, None, vec![], None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
}

/// Psionic Ritual recasts a graveyard sorcery and exiles itself.
#[test]
fn cmdr_umbris_psionic_ritual_copies_a_sorcery() {
    let mut g = game(2);
    let div = g.add_card_to_graveyard(1, catalog::divination());
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::psionic_ritual());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    cast(&mut g, s, Some(Target::Permanent(div)), vec![], None);
    assert!(g.exile.iter().any(|c| c.id == div));
    assert!(g.exile.iter().any(|c| c.id == s), "Psionic Ritual exiled itself");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "the copied Divination drew two");
}

/// Cut Your Losses mills half the target's library.
#[test]
fn cmdr_umbris_cut_your_losses_mills_half() {
    let mut g = game(2);
    flood(&mut g, 0);
    let lib = g.players[1].library.len();
    let s = g.add_card_to_hand(0, catalog::cut_your_losses());
    cast(&mut g, s, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[1].library.len(), lib - lib / 2);
}

/// Startled Awake mills thirteen, then climbs back as Persistent Nightmare.
#[test]
fn cmdr_umbris_startled_awake_mills_and_transforms() {
    let mut g = game(2);
    flood(&mut g, 0);
    let s = g.add_card_to_hand(0, catalog::startled_awake());
    cast(&mut g, s, Some(Target::Player(1)), vec![], None);
    assert_eq!(g.players[1].graveyard.len(), 13);
    flood(&mut g, 0);
    activate(&mut g, s, 0, None).expect("return transformed");
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Persistent Nightmare")
        .expect("Persistent Nightmare");
    assert!(back.definition.keywords.contains(&Keyword::Skulk));
}

/// Stone of Erech exiles the opponent's dying creatures and cashes in on a
/// graveyard.
#[test]
fn cmdr_umbris_stone_of_erech() {
    let mut g = game(2);
    let stone = g.add_card_to_battlefield(0, catalog::stone_of_erech());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    bolt(&mut g, 0, Target::Permanent(bears));
    assert!(g.exile.iter().any(|c| c.id == bears), "exiled instead of dying");
    g.add_card_to_graveyard(1, catalog::island());
    flood(&mut g, 0);
    let hand = g.players[0].hand.len();
    activate(&mut g, stone, 0, Some(Target::Player(1))).expect("crack");
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Panharmonicon doubles an entering creature's ETB trigger.
#[test]
fn cmdr_umbris_panharmonicon_doubles_etb() {
    let mut g = game(2);
    g.add_card_to_battlefield(0, catalog::panharmonicon());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::island());
    let lurker = g.add_card_to_hand(0, catalog::yaroks_fenlurker());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, lurker, None, vec![], None);
    assert!(g.players[1].hand.is_empty(), "the ETB fired twice");
}

/// Shape check: Startled Awake carries its Nightmare back face.
#[test]
fn cmdr_umbris_card_shapes() {
    let d = catalog::startled_awake();
    assert_eq!(d.back_face.as_ref().unwrap().name, "Persistent Nightmare");
}
