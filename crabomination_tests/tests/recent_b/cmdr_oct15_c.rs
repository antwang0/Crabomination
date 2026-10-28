//! Commander: short-text two- and three-card gaps of the 2026-10-15 evening
//! rescan (`decks::cmdr_oct15_c`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::{Color, SpendRestriction};

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    stock_libraries(&mut g, 10);
    g
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast_with(g: &mut GameState, id: CardId, target: Option<Target>, x_value: Option<u32>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value })
        .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId) {
    cast_with(g, id, None, None);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: index, target, additional_targets: vec![], x_value: None, mode: None })
        .expect("activate");
    drain_stack(g);
}

fn tap(g: &mut GameState, id: CardId) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let evs = g.resolve_effect(&Effect::Tap { what: Selector::ExactObjects(vec![id]) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn attack_unblocked(g: &mut GameState, attacks: Vec<(CardId, usize)>) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.into_iter().map(|(attacker, p)| Attack { attacker, target: AttackTarget::Player(p) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
}

fn step_to(g: &mut GameState, seat: usize, step: TurnStep) {
    for _ in 0..200 {
        if g.active_player_idx == seat && g.step == step {
            return;
        }
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
    panic!("never reached seat {seat} {step:?}");
}

fn named_count(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

/// CR 106.6 — Fíli and Kíli's mana funds Dwarf, Equipment and Saga spells and
/// nothing else.
#[test]
fn fili_and_kili_mana_funds_dwarves_equipment_and_sagas() {
    let r = SpendRestriction::DwarfEquipmentOrSagaSpells;
    for (def, ok) in [
        (catalog::fili_and_kili_joyous(), true),
        (catalog::bonesplitter(), true),
        (catalog::the_eldest_reborn(), true),
        (catalog::grizzly_bears(), false),
        (catalog::lightning_bolt(), false),
    ] {
        assert_eq!(r.allows(&def.spell_kind()), ok, "{}", def.name);
    }
}

/// CR 611.2b — Don't Move destroys the tapped creatures, then destroys any
/// creature that becomes tapped until its caster's next turn begins.
#[test]
fn dont_move_kills_tapped_creatures_until_your_next_turn() {
    let mut g = pod(2);
    let tapped = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    let later = ready(&mut g, 1, catalog::hill_giant());
    let survivor = ready(&mut g, 1, catalog::craw_wurm());
    let spell = g.add_card_to_hand(0, catalog::dont_move());
    cast(&mut g, spell);
    assert!(g.battlefield_find(tapped).is_none(), "a tapped creature is destroyed");
    assert!(g.battlefield_find(later).is_some(), "an untapped one survives the sweep");
    tap(&mut g, later);
    assert!(g.battlefield_find(later).is_none(), "becoming tapped later destroys it");
    step_to(&mut g, 0, TurnStep::Upkeep);
    tap(&mut g, survivor);
    assert!(g.battlefield_find(survivor).is_some(), "the trigger ended as your next turn began");
}

/// CR 614.1a — Charging Tuskodon's combat damage to a player is doubled.
#[test]
fn charging_tuskodon_doubles_its_combat_damage_to_a_player() {
    let mut g = pod(2);
    let dino = ready(&mut g, 0, catalog::charging_tuskodon());
    let life = g.players[1].life;
    attack_unblocked(&mut g, vec![(dino, 1)]);
    assert_eq!(g.players[1].life, life - 8);
}

/// Absorbing Man and Titania double a creature's damage, not a spell's.
#[test]
fn absorbing_man_doubles_creature_damage_only() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::absorbing_man_and_titania());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[1].life;
    attack_unblocked(&mut g, vec![(bears, 1)]);
    assert_eq!(g.players[1].life, life - 4, "the Bears deal 4");
    g.step = TurnStep::PostCombatMain;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Player(1)), None);
    assert_eq!(g.players[1].life, life - 7, "the Bolt deals 3");
}

/// CR 702.193 — Advancing the Spirit: the first power-up each of your turns
/// costs {0}; the second pays in full.
#[test]
fn advancing_the_spirit_waives_the_first_power_up_each_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::advancing_the_spirit());
    let a = ready(&mut g, 0, catalog::captain_marvel_earths_protector());
    let b = ready(&mut g, 0, catalog::captain_marvel_earths_protector());
    for id in [a, b] {
        g.battlefield_find_mut(id).unwrap().entered_turn = None;
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: a, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("the first power-up is free");
    drain_stack(&mut g);
    assert_eq!(plus(&g, a), 1);
    let second = g.perform_action(GameAction::ActivateAbility { card_id: b, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None });
    assert!(second.is_err(), "the second power-up needs its {{5}}{{W}}{{W}}");
}

/// Rally the Ancestors returns creature cards of mana value X or less, exiles
/// them at your next upkeep, and exiles itself.
#[test]
fn rally_the_ancestors_returns_then_exiles_at_your_upkeep() {
    let mut g = pod(2);
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let rally = g.add_card_to_hand(0, catalog::rally_the_ancestors());
    cast_with(&mut g, rally, None, Some(2));
    assert!(g.battlefield_find(bears).is_some(), "mana value 2 returns");
    assert!(g.battlefield_find(wurm).is_none(), "mana value 6 stays");
    assert!(g.exile.iter().any(|c| c.id == rally), "Rally exiles itself");
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(g.battlefield_find(bears).is_some(), "an opponent's upkeep isn't yours");
    step_to(&mut g, 0, TurnStep::Draw);
    assert!(g.exile.iter().any(|c| c.id == bears), "exiled at your next upkeep");
}

/// Picard puts two counters on another creature of yours as he enters and as
/// he attacks.
#[test]
fn picard_counters_another_creature_on_entry_and_attack() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let picard = g.add_card_to_hand(0, catalog::picard_steadfast_captain());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bears))]));
    cast(&mut g, picard);
    assert_eq!(plus(&g, bears), 2);
    g.clear_sickness(picard);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bears))]));
    attack_unblocked(&mut g, vec![(picard, 1)]);
    assert_eq!(plus(&g, bears), 4);
}

/// Bio-Asset Allocator's granted attack trigger drains each opponent — every
/// opponent at four seats — and only from artifact creatures.
#[test]
fn bio_asset_allocator_artifact_attackers_drain_each_opponent() {
    let mut g = pod(4);
    ready(&mut g, 0, catalog::bio_asset_allocator());
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[0].life;
    attack_unblocked(&mut g, vec![(thopter, 1), (bears, 2)]);
    assert_eq!(g.players[0].life, life + 1, "one trigger: you gain 1");
    assert_eq!(g.players[3].life, life - 1);
    assert_eq!(g.players[2].life, life - 2 - 1, "the Bears' combat damage plus the drain");
}

/// Badgey and Spireside Infiltrator trigger on becoming tapped.
#[test]
fn becoming_tapped_drains_and_pings() {
    let mut g = pod(3);
    let badgey = ready(&mut g, 0, catalog::badgey_malicious_glitch());
    let spire = ready(&mut g, 0, catalog::spireside_infiltrator());
    let life = g.players[1].life;
    tap(&mut g, badgey);
    tap(&mut g, spire);
    assert_eq!((g.players[0].life, g.players[1].life, g.players[2].life), (life + 1, life - 2, life - 2));
}

/// Hulk enters with a +1/+1 counter and doubles the counters on your Gamma
/// creatures each upkeep.
#[test]
fn hulk_doubles_gamma_counters_at_upkeep() {
    let mut g = pod(2);
    let hulk = g.add_card_to_hand(0, catalog::hulk_strongest_there_is());
    cast(&mut g, hulk);
    assert_eq!(plus(&g, hulk), 1);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!((plus(&g, hulk), plus(&g, bears)), (2, 1), "only Gamma creatures double");
}

/// Sporoloth Ancient grants its spore ability to your creatures; Sporesower
/// Thallid puts a spore counter on each Fungus at your upkeep.
#[test]
fn spore_counters_make_saprolings() {
    let mut g = pod(2);
    let sower = ready(&mut g, 0, catalog::sporesower_thallid());
    let ancient = ready(&mut g, 0, catalog::sporoloth_ancient());
    step_to(&mut g, 0, TurnStep::Draw);
    let spores = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::Spore);
    assert_eq!((spores(&g, sower), spores(&g, ancient)), (1, 2), "the Ancient gets both");
    g.step = TurnStep::PreCombatMain;
    let granted = g.battlefield_find(ancient).unwrap().definition.activated_abilities.len();
    activate(&mut g, ancient, granted, None);
    assert_eq!(named_count(&g, 0, "Saproling"), 1);
}

/// Saproling Migration makes four Saprolings when kicked.
#[test]
fn saproling_migration_kicked_makes_four() {
    let mut g = pod(2);
    let spell = g.add_card_to_hand(0, catalog::saproling_migration());
    flood(&mut g);
    g.perform_action(GameAction::CastSpellKicked { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("kicked");
    drain_stack(&mut g);
    assert_eq!(named_count(&g, 0, "Saproling"), 4);
}

/// Oviya's Construct is X/X for the creatures you control as it's made.
#[test]
fn oviya_construct_counts_your_creatures() {
    let mut g = pod(2);
    let oviya = ready(&mut g, 0, catalog::oviya_pashiri_sage_lifecrafter());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    activate(&mut g, oviya, 1, None);
    let construct = g.battlefield.iter().find(|c| c.definition.name == "Construct").unwrap().id;
    assert_eq!(pt(&g, construct), (2, 2));
}

/// Coordinated Barrage counts your permanents of the chosen type.
#[test]
fn coordinated_barrage_counts_the_chosen_type() {
    let mut g = pod(2);
    for _ in 0..3 {
        ready(&mut g, 0, catalog::llanowar_elves());
    }
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    g.attacking.push(Attack { attacker: wurm, target: AttackTarget::Player(0) });
    let spell = g.add_card_to_hand(0, catalog::coordinated_barrage());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(crabomination::card::CreatureType::Elf)]));
    cast_with(&mut g, spell, Some(Target::Permanent(wurm)), None);
    assert_eq!(g.battlefield_find(wurm).unwrap().damage, 3);
}

/// Hook Swords suits up its Ally token; the bonus is only on your turn.
#[test]
fn hook_swords_bonus_on_your_turn_only() {
    let mut g = pod(2);
    let swords = g.add_card_to_hand(0, catalog::hook_swords());
    cast(&mut g, swords);
    let ally = g.battlefield.iter().find(|c| c.definition.name == "Ally").unwrap().id;
    assert_eq!(g.battlefield_find(swords).unwrap().attached_to, Some(ally));
    assert_eq!(pt(&g, ally), (2, 2));
    assert!(g.permanent_has_keyword(ally, &Keyword::FirstStrike));
    g.active_player_idx = 1;
    assert_eq!(pt(&g, ally), (1, 1));
}

/// Patriot pumps your other creatures only while he is equipped.
#[test]
fn patriot_anthem_needs_equipment() {
    let mut g = pod(2);
    let patriot = ready(&mut g, 0, catalog::patriot_young_avenger());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, bears), (2, 2));
    let split = ready(&mut g, 0, catalog::bonesplitter());
    g.battlefield_find_mut(split).unwrap().attached_to = Some(patriot);
    assert_eq!(pt(&g, bears), (3, 2));
    assert_eq!(pt(&g, patriot), (5, 2), "Bonesplitter only, not his own anthem");
}

/// General Chang sacrifices a creature to grow every Klingon you control.
#[test]
fn general_chang_grows_klingons() {
    let mut g = pod(2);
    let chang = ready(&mut g, 0, catalog::general_chang_cold_warrior());
    let worf = ready(&mut g, 0, catalog::worf_chief_tactical_officer());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bears])]));
    activate(&mut g, chang, 0, None);
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!((plus(&g, chang), plus(&g, worf)), (1, 1));
}

/// Ent's Fury counters only a creature with power 4 or more, then fights.
#[test]
fn ents_fury_counters_big_creatures_then_fights() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ents_fury());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![Target::Permanent(bears)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(plus(&g, giant), 0, "a 3/3 gets no counter");
    assert!(g.battlefield_find(bears).is_none(), "the 4/4-for-the-turn Giant wins the fight");
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 2);
}

/// Perfected Theory's second mode sets base 4/5; Mordenkainen's Polymorph
/// makes a 4/4 flying Dragon.
#[test]
fn base_pt_spells_set_the_creature() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let theory = g.add_card_to_hand(0, catalog::perfected_theory());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell { card_id: theory, target: Some(Target::Permanent(bears)), additional_targets: vec![], mode: Some(1), x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(pt(&g, bears), (4, 5));
    let elves = ready(&mut g, 0, catalog::llanowar_elves());
    let poly = g.add_card_to_hand(0, catalog::mordenkainens_polymorph());
    cast_with(&mut g, poly, Some(Target::Permanent(elves)), None);
    assert_eq!(pt(&g, elves), (4, 4));
    assert!(g.permanent_has_keyword(elves, &Keyword::Flying));
}

/// Agent Maria Hill grows and draws when she's tapped for a teamwork cost.
#[test]
fn agent_maria_hill_triggers_on_teamwork_taps() {
    let mut g = pod(2);
    let hill = ready(&mut g, 0, catalog::agent_maria_hill());
    let elves = ready(&mut g, 0, catalog::llanowar_elves());
    g.add_card_to_library(0, catalog::island());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let nuts = g.add_card_to_hand(0, catalog::go_nuts());
    let hand = g.players[0].hand.len();
    flood(&mut g);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: nuts,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("teamwork paid");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elves).unwrap().tapped);
    assert!(g.battlefield_find(hill).unwrap().tapped, "Maria Hill tapped too");
    assert_eq!(plus(&g, hill), 1);
    assert_eq!(g.players[0].hand.len(), hand, "Go Nuts! left, a card came");
}

// ── Batch 2 ─────────────────────────────────────────────────────────────────

/// Stature becomes base 4/4 when you cast a noncreature spell, not a creature.
#[test]
fn stature_grows_on_noncreature_spells() {
    let mut g = pod(2);
    let stature = ready(&mut g, 0, catalog::stature_young_avenger());
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, elves);
    assert_eq!(pt(&g, stature), (2, 2));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Player(1)), None);
    assert_eq!(pt(&g, stature), (4, 4));
}

/// Markov Blademaster's double strike grows it between the two damage steps.
#[test]
fn markov_blademaster_grows_per_strike() {
    let mut g = pod(2);
    let blade = ready(&mut g, 0, catalog::markov_blademaster());
    let life = g.players[1].life;
    attack_unblocked(&mut g, vec![(blade, 1)]);
    assert_eq!(plus(&g, blade), 2);
    assert_eq!(g.players[1].life, life - 1 - 2, "1 first strike, then 2");
}

/// Talons of Wildwood and Gilded Assault Cart come back from the graveyard
/// to hand by their own costs.
#[test]
fn graveyard_returns_to_hand() {
    let mut g = pod(2);
    let talons = g.add_card_to_graveyard(0, catalog::talons_of_wildwood());
    activate(&mut g, talons, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == talons));
    let cart = g.add_card_to_graveyard(0, catalog::gilded_assault_cart());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let make = Effect::CreateToken {
        who: crabomination::effect::PlayerRef::You,
        count: crabomination::effect::Value::Const(2),
        definition: std::sync::Arc::new(crabomination_base::tokens::treasure_token()),
    };
    g.resolve_effect(&make, &ctx).unwrap();
    activate(&mut g, cart, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == cart));
    assert_eq!(named_count(&g, 0, "Treasure"), 0, "both Treasures paid");
}

/// Deadeye Brawler draws on combat damage only with the city's blessing.
#[test]
fn deadeye_brawler_draws_with_the_citys_blessing() {
    let mut g = pod(2);
    let brawler = ready(&mut g, 0, catalog::deadeye_brawler());
    let hand = g.players[0].hand.len();
    attack_unblocked(&mut g, vec![(brawler, 1)]);
    assert_eq!(g.players[0].hand.len(), hand, "no blessing, no card");
    g.players[0].city_blessing = true;
    g.step = TurnStep::PreCombatMain;
    g.battlefield_find_mut(brawler).unwrap().tapped = false;
    attack_unblocked(&mut g, vec![(brawler, 1)]);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Skyswimmer Koi loots when an artifact enters under your control.
#[test]
fn skyswimmer_koi_loots_on_artifacts() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::skyswimmer_koi());
    g.add_card_to_hand(0, catalog::island());
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let gy = g.players[0].graveyard.len();
    cast(&mut g, ring);
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "drew and discarded");
}

/// Eidolon of Astral Winds: an enchantment entering makes a creature you
/// control a 4/4 flier until end of turn.
#[test]
fn eidolon_constellation_makes_a_4_4_flier() {
    let mut g = pod(2);
    let elves = ready(&mut g, 0, catalog::llanowar_elves());
    let eidolon = g.add_card_to_hand(0, catalog::eidolon_of_astral_winds());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(elves))]));
    cast(&mut g, eidolon);
    assert_eq!(pt(&g, elves), (4, 4));
    assert!(g.permanent_has_keyword(elves, &Keyword::Flying));
}

/// Temporal Isolation gives shadow and stops its creature's damage; Triclopean
/// Sight untaps its creature and pumps it.
#[test]
fn isolation_and_sight_auras() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let iso = g.add_card_to_hand(0, catalog::temporal_isolation());
    cast_with(&mut g, iso, Some(Target::Permanent(wurm)), None);
    assert!(g.permanent_has_keyword(wurm, &Keyword::Shadow));
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bears).unwrap().tapped = true;
    let sight = g.add_card_to_hand(0, catalog::triclopean_sight());
    cast_with(&mut g, sight, Some(Target::Permanent(bears)), None);
    assert!(!g.battlefield_find(bears).unwrap().tapped);
    assert_eq!(pt(&g, bears), (3, 3));
}

/// Most Decrepit Old Bird's adventure mills four and takes an instant.
#[test]
fn old_bird_adventure_takes_an_instant() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let bird = g.add_card_to_hand(0, catalog::most_decrepit_old_bird());
    flood(&mut g);
    g.perform_action(GameAction::CastAdventure { card_id: bird, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("adventure");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt));
}

// ── Batch 3 ─────────────────────────────────────────────────────────────────

/// Wisecrack: a creature hits itself, and an attacker's controller takes 2.
#[test]
fn wisecrack_self_damage_and_attacker_tax() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    g.attacking.push(Attack { attacker: wurm, target: AttackTarget::Player(0) });
    let life = g.players[1].life;
    let spell = g.add_card_to_hand(0, catalog::wisecrack());
    cast_with(&mut g, spell, Some(Target::Permanent(wurm)), None);
    assert!(g.battlefield_find(wurm).is_none(), "6 damage to a 6/4");
    assert_eq!(g.players[1].life, life - 2);
}

/// Giantfall's first mode bites; its second destroys an artifact.
#[test]
fn giantfall_bites_or_breaks() {
    let mut g = pod(2);
    let giant = ready(&mut g, 0, catalog::hill_giant());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::giantfall());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(giant)),
        additional_targets: vec![Target::Permanent(bears)],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 0, "one-sided");
}

/// Shadowfax puts a lesser-power creature card from hand onto the battlefield
/// attacking; a bigger one stays in hand.
#[test]
fn shadowfax_deploys_a_smaller_creature_attacking() {
    let mut g = pod(2);
    let fax = ready(&mut g, 0, catalog::shadowfax_lord_of_horses());
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    attack_unblocked(&mut g, vec![(fax, 1)]);
    assert!(g.battlefield_find(bears).is_some(), "the 2-power Bears came in");
    assert!(g.players[0].hand.iter().any(|c| c.id == wurm), "the 6-power Wurm can't");
}

/// Vitaspore Thallid sacrifices a Saproling to give haste.
#[test]
fn vitaspore_thallid_hastes_with_a_saproling() {
    let mut g = pod(2);
    let vita = ready(&mut g, 0, catalog::vitaspore_thallid());
    g.battlefield_find_mut(vita).unwrap().add_counters(CounterType::Spore, 3);
    activate(&mut g, vita, 0, None);
    assert_eq!(named_count(&g, 0, "Saproling"), 1);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, vita, 1, Some(Target::Permanent(bears)));
    assert_eq!(named_count(&g, 0, "Saproling"), 0);
    assert!(g.permanent_has_keyword(bears, &Keyword::Haste));
}

/// Raven Clan War-Axe finds Eivor in the graveyard before the library.
#[test]
fn raven_clan_war_axe_finds_eivor() {
    let mut g = pod(2);
    let eivor = g.add_card_to_graveyard(0, catalog::eivor_battle_ready());
    let axe = g.add_card_to_hand(0, catalog::raven_clan_war_axe());
    cast(&mut g, axe);
    assert!(g.players[0].hand.iter().any(|c| c.id == eivor));
}

/// Settlement Blacksmith draws only with an Equipment out.
#[test]
fn settlement_blacksmith_needs_equipment() {
    let mut g = pod(2);
    let hand = g.players[0].hand.len();
    let a = g.add_card_to_hand(0, catalog::settlement_blacksmith());
    cast(&mut g, a);
    assert_eq!(g.players[0].hand.len(), hand);
    ready(&mut g, 0, catalog::bonesplitter());
    let b = g.add_card_to_hand(0, catalog::settlement_blacksmith());
    cast(&mut g, b);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Reaper's Talisman: attacking alone drains the defending player for 2 and
/// grants deathtouch.
#[test]
fn reapers_talisman_drains_a_lone_attacker() {
    let mut g = pod(3);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let tal = ready(&mut g, 0, catalog::reapers_talisman());
    g.battlefield_find_mut(tal).unwrap().attached_to = Some(bears);
    let life = g.players[2].life;
    attack_unblocked(&mut g, vec![(bears, 2)]);
    assert_eq!(g.players[2].life, life - 2 - 2);
    assert_eq!(g.players[1].life, life, "only the defending player");
    assert_eq!(g.players[0].life, life + 2);
}

/// Pain 101: the creature comes back tapped after dying this turn.
#[test]
fn pain_101_returns_the_creature_tapped() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::pain_101());
    cast_with(&mut g, spell, Some(Target::Permanent(bears)), None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(bears)), None);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert!(back.tapped);
}

// ── Batch 4 ─────────────────────────────────────────────────────────────────

/// Leonardo, Sewer Samurai: on your turn a 1-toughness creature card casts
/// from the graveyard and enters with a finality counter; a 2/2 can't.
#[test]
fn leonardo_casts_small_creatures_from_the_graveyard() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::leonardo_sewer_samurai());
    let elves = g.add_card_to_graveyard(0, catalog::llanowar_elves());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g);
    let no = g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None });
    assert!(no.is_err(), "a 2/2 isn't small enough");
    cast(&mut g, elves);
    let c = g.battlefield_find(elves).expect("cast from the graveyard");
    assert_eq!(c.counter_count(CounterType::Finality), 1);
}

/// Hawkeye grows when you cast a spell that targets a creature.
#[test]
fn hawkeye_grows_on_creature_targeting_spells() {
    let mut g = pod(2);
    let hawk = ready(&mut g, 0, catalog::hawkeye_bowslinger());
    let bears = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Player(1)), None);
    assert_eq!(plus(&g, hawk), 0, "a player target isn't a creature");
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(bears)), None);
    assert_eq!(plus(&g, hawk), 1);
}

/// Stunning Shot counters your creature and stuns an opposing one.
#[test]
fn stunning_shot_counters_and_stuns() {
    let mut g = pod(2);
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::stunning_shot());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(plus(&g, mine), 2);
    let t = g.battlefield_find(theirs).unwrap();
    assert!(t.tapped);
    assert_eq!(t.counter_count(CounterType::Stun), 1);
}

/// Angelic Cub grows only on the first targeting each turn.
#[test]
fn angelic_cub_grows_once_a_turn() {
    let mut g = pod(2);
    let cub = ready(&mut g, 0, catalog::angelic_cub());
    for _ in 0..2 {
        let shot = g.add_card_to_hand(0, catalog::stunning_shot());
        cast_with(&mut g, shot, Some(Target::Permanent(cub)), None);
    }
    assert_eq!(plus(&g, cub), 2 + 2 + 1, "two Shots, one Cub trigger");
    assert!(g.permanent_has_keyword(cub, &Keyword::Flying), "three or more counters");
}

/// Oathsworn Knight sheds a counter instead of taking damage.
#[test]
fn oathsworn_knight_prevents_with_counters() {
    let mut g = pod(2);
    let knight = g.add_card_to_hand(0, catalog::oathsworn_knight());
    cast(&mut g, knight);
    assert_eq!(plus(&g, knight), 4);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, bolt, Some(Target::Permanent(knight)), None);
    assert_eq!(plus(&g, knight), 3);
    assert_eq!(g.battlefield_find(knight).unwrap().damage, 0);
}

/// Mausoleum Secrets tutors a black card no bigger than your creature-card
/// count in the graveyard.
#[test]
fn mausoleum_secrets_reads_creature_cards_in_graveyard() {
    let mut g = pod(2);
    g.players[0].library.clear();
    let big = g.add_card_to_library(0, catalog::massacre_wurm());
    let small = g.add_card_to_library(0, catalog::dark_ritual());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::mausoleum_secrets());
    cast(&mut g, spell);
    assert!(g.players[0].hand.iter().any(|c| c.id == small));
    assert!(!g.players[0].hand.iter().any(|c| c.id == big));
}

/// Leonardo, Big Brother gets +1/+0 for each other creature you control.
#[test]
fn leonardo_big_brother_counts_the_team() {
    let mut g = pod(2);
    let leo = ready(&mut g, 0, catalog::leonardo_big_brother());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::llanowar_elves());
    ready(&mut g, 1, catalog::grizzly_bears());
    assert_eq!(pt(&g, leo), (3, 3));
}

// ── Batch 5 ─────────────────────────────────────────────────────────────────

/// Exalted stacks: three exalted creatures pump a lone attacker by three.
#[test]
fn exalted_cards_pump_a_lone_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::court_archers());
    ready(&mut g, 0, catalog::guardians_of_akrasa());
    ready(&mut g, 0, catalog::ethercaste_knight());
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[1].life;
    attack_unblocked(&mut g, vec![(bears, 1)]);
    assert_eq!(g.players[1].life, life - 5);
}

/// Griffin Rider is a 4/4 flier only while you control a Griffin.
#[test]
fn griffin_rider_needs_a_griffin() {
    let mut g = pod(2);
    let rider = ready(&mut g, 0, catalog::griffin_rider());
    assert_eq!(pt(&g, rider), (1, 1));
    ready(&mut g, 0, catalog::makindi_griffin());
    assert_eq!(pt(&g, rider), (4, 4));
    assert!(g.permanent_has_keyword(rider, &Keyword::Flying));
}

/// Ajani, Inspiring Leader's −3 exiles a creature and its controller gains 2.
#[test]
fn ajani_inspiring_leader_exiles_and_heals_its_controller() {
    let mut g = pod(2);
    let ajani = ready(&mut g, 0, catalog::ajani_inspiring_leader());
    g.battlefield_find_mut(ajani).unwrap().add_counters(CounterType::Loyalty, 5);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: ajani, ability_index: 1, target: Some(Target::Permanent(wurm)), x_value: None })
        .expect("loyalty");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == wurm));
    assert_eq!(g.players[1].life, life + 2);
}

/// Dwarven Priest counts your creatures, itself included.
#[test]
fn dwarven_priest_gains_per_creature() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    let life = g.players[0].life;
    let priest = g.add_card_to_hand(0, catalog::dwarven_priest());
    cast(&mut g, priest);
    assert_eq!(g.players[0].life, life + 2);
}

/// Criminal Past gives your commander menace and +X/+0 for creature cards in
/// your graveyard; a non-commander gets nothing.
#[test]
fn criminal_past_grows_your_commander() {
    let mut g = pod(2);
    let cmdr = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    let other = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::criminal_past());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    g.add_card_to_graveyard(0, catalog::craw_wurm());
    assert_eq!(pt(&g, cmdr), (4, 2));
    assert!(g.permanent_has_keyword(cmdr, &Keyword::Menace));
    assert_eq!(pt(&g, other), (2, 2));
}

/// Kruin Striker pumps when another creature enters; Riot Ringleader pumps
/// Humans as it attacks.
#[test]
fn kruin_striker_and_riot_ringleader() {
    let mut g = pod(2);
    let striker = ready(&mut g, 0, catalog::kruin_striker());
    let elves = g.add_card_to_hand(0, catalog::llanowar_elves());
    cast(&mut g, elves);
    assert_eq!(pt(&g, striker), (3, 1));
    assert!(g.permanent_has_keyword(striker, &Keyword::Trample));
}

/// Thatcher Revolt's hasty Humans are sacrificed at the next end step.
#[test]
fn thatcher_revolt_tokens_leave_at_end_step() {
    let mut g = pod(2);
    let spell = g.add_card_to_hand(0, catalog::thatcher_revolt());
    cast(&mut g, spell);
    assert_eq!(named_count(&g, 0, "Human"), 3);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert_eq!(named_count(&g, 0, "Human"), 0);
}

/// Gravewaker returns a creature card tapped.
#[test]
fn gravewaker_reanimates_tapped() {
    let mut g = pod(2);
    let waker = ready(&mut g, 0, catalog::gravewaker());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    activate(&mut g, waker, 0, Some(Target::Permanent(wurm)));
    assert!(g.battlefield_find(wurm).unwrap().tapped);
}

// ── Batch 6 ─────────────────────────────────────────────────────────────────

/// CR 702.80a — Puncture Blast's wither puts -1/-1 counters on a creature.
#[test]
fn puncture_blast_withers() {
    let mut g = pod(2);
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let blast = g.add_card_to_hand(0, catalog::puncture_blast());
    cast_with(&mut g, blast, Some(Target::Permanent(wurm)), None);
    let c = g.battlefield_find(wurm).unwrap();
    assert_eq!((c.counter_count(CounterType::MinusOneMinusOne), c.damage), (3, 0));
}

/// Arni's boast makes his base power 1 + your biggest other creature's.
#[test]
fn arni_boasts_to_the_biggest() {
    let mut g = pod(2);
    let arni = ready(&mut g, 0, catalog::arni_brokenbrow());
    ready(&mut g, 0, catalog::craw_wurm());
    g.battlefield_find_mut(arni).unwrap().attacked_this_turn = true;
    activate(&mut g, arni, 0, None);
    assert_eq!(pt(&g, arni), (7, 3));
}

/// Inferno Hellion shuffles itself away at the end step after it attacked.
#[test]
fn inferno_hellion_leaves_after_attacking() {
    let mut g = pod(2);
    let hellion = ready(&mut g, 0, catalog::inferno_hellion());
    attack_unblocked(&mut g, vec![(hellion, 1)]);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(g.battlefield_find(hellion).is_none());
    assert!(g.players[0].library.iter().any(|c| c.id == hellion));
}

/// Scourge of Geier Reach counts only opponents' creatures.
#[test]
fn scourge_of_geier_reach_counts_opponents() {
    let mut g = pod(3);
    let scourge = ready(&mut g, 0, catalog::scourge_of_geier_reach());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 2, catalog::grizzly_bears());
    assert_eq!(pt(&g, scourge), (5, 5));
}

/// Earthshaker Giant pumps your other creatures as it enters.
#[test]
fn earthshaker_giant_pumps_the_team() {
    let mut g = pod(2);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = g.add_card_to_hand(0, catalog::earthshaker_giant());
    cast(&mut g, giant);
    assert_eq!(pt(&g, bears), (5, 5));
    assert!(g.permanent_has_keyword(bears, &Keyword::Trample));
    assert_eq!(pt(&g, giant), (6, 6));
}

/// Saheeli makes one Thopter a turn however often you scry; Master Trinketeer
/// pumps it.
#[test]
fn saheeli_thopter_once_a_turn() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::saheeli_consul_of_oversight());
    ready(&mut g, 0, catalog::master_trinketeer());
    for _ in 0..2 {
        let shot = g.add_card_to_hand(0, catalog::skywhalers_shot());
        let wurm = ready(&mut g, 1, catalog::craw_wurm());
        cast_with(&mut g, shot, Some(Target::Permanent(wurm)), None);
        assert!(g.battlefield_find(wurm).is_none());
    }
    assert_eq!(named_count(&g, 0, "Thopter"), 1);
    let t = g.battlefield.iter().find(|c| c.definition.name == "Thopter").unwrap().id;
    assert_eq!(pt(&g, t), (2, 2));
}

/// Enchanted River's Grasp strips abilities and keeps the creature tapped.
#[test]
fn enchanted_rivers_grasp_locks_a_creature() {
    let mut g = pod(2);
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    let grasp = g.add_card_to_hand(0, catalog::enchanted_rivers_grasp());
    cast_with(&mut g, grasp, Some(Target::Permanent(elf)), None);
    assert!(g.battlefield_find(elf).unwrap().tapped);
    assert!(g.computed_permanent(elf).unwrap().lost_all_abilities);
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert!(g.battlefield_find(elf).unwrap().tapped, "doesn't untap");
}

/// Sound the Trumpets recruits only off a cheap spell.
#[test]
fn sound_the_trumpets_recruits_on_cheap_spells() {
    let mut g = pod(2);
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let trumpets = g.add_card_to_hand(0, catalog::sound_the_trumpets());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: trumpets, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None })
        .expect("counter");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "countered");
    assert_eq!(named_count(&g, 0, "Human Soldier"), 1, "discarded a nonland card");
}

/// Uninvited Geist transforms into an unblockable 3/3 after connecting.
#[test]
fn uninvited_geist_transforms() {
    let mut g = pod(2);
    let geist = ready(&mut g, 0, catalog::uninvited_geist());
    attack_unblocked(&mut g, vec![(geist, 1)]);
    assert_eq!(g.battlefield_find(geist).unwrap().definition.name, "Unimpeded Trespasser");
    assert_eq!(pt(&g, geist), (3, 3));
}

// ── Batch 7 ─────────────────────────────────────────────────────────────────

/// Pennon Blade scales with your creatures; Shambling Suit with your artifacts
/// and enchantments.
#[test]
fn pennon_blade_and_shambling_suit_scale() {
    let mut g = pod(2);
    let suit = ready(&mut g, 0, catalog::shambling_suit());
    let blade = ready(&mut g, 0, catalog::pennon_blade());
    assert_eq!(pt(&g, suit), (2, 3), "the Suit and the Blade");
    g.battlefield_find_mut(blade).unwrap().attached_to = Some(suit);
    ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(pt(&g, suit), (4, 5), "+1/+1 for each of two creatures");
}

/// Rhizome Lurcher enters with a counter per creature card in your graveyard;
/// Molderhulk costs {1} less per one.
#[test]
fn undergrowth_counts_creature_cards() {
    let mut g = pod(2);
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    let lurcher = g.add_card_to_hand(0, catalog::rhizome_lurcher());
    cast(&mut g, lurcher);
    assert_eq!(plus(&g, lurcher), 2);
    let forest = g.add_card_to_graveyard(0, catalog::forest());
    let hulk = g.add_card_to_hand(0, catalog::molderhulk());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(forest))]));
    cast(&mut g, hulk);
    assert!(g.battlefield_find(forest).is_some(), "the land came back");
}

/// Face Yourself copies each of a player's creatures with haste, and the copies
/// leave at the end step without a planeswalker.
#[test]
fn face_yourself_copies_then_sacrifices() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::grizzly_bears());
    ready(&mut g, 1, catalog::hill_giant());
    let spell = g.add_card_to_hand(0, catalog::face_yourself());
    cast_with(&mut g, spell, Some(Target::Player(1)), None);
    let mine: Vec<_> = g.battlefield.iter().filter(|c| c.controller == 0).map(|c| c.id).collect();
    assert_eq!(mine.len(), 2);
    assert!(mine.iter().all(|id| g.permanent_has_keyword(*id, &Keyword::Haste)));
    step_to(&mut g, 1, TurnStep::Upkeep);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0);
}

/// Stone-Giant of High Pass makes a Stone Boulder as it enters and sacrifices
/// an artifact for 4 damage.
#[test]
fn stone_giant_boulders_and_throws() {
    let mut g = pod(2);
    let giant = g.add_card_to_hand(0, catalog::stone_giant_of_high_pass());
    cast(&mut g, giant);
    assert_eq!(named_count(&g, 0, "Stone Boulder"), 1);
    let life = g.players[1].life;
    activate(&mut g, giant, 0, Some(Target::Player(1)));
    assert_eq!(named_count(&g, 0, "Stone Boulder"), 0);
    assert_eq!(g.players[1].life, life - 4);
}

/// Winter: sacrificing a creature makes each opponent sacrifice one.
#[test]
fn winter_trades_a_sacrifice_with_each_opponent() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::grizzly_bears());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let b = ready(&mut g, 2, catalog::grizzly_bears());
    let winter = g.add_card_to_hand(0, catalog::winter_tormented_loner());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, winter);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert_eq!(g.players[0].graveyard.len(), 1, "one creature of yours paid for it");
}

// ── Batch 8 ─────────────────────────────────────────────────────────────────

/// Wall Crawl makes a Spider, gains a life per Spider, and pumps Spiders.
#[test]
fn wall_crawl_spider_package() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::needlepeak_spider());
    let life = g.players[0].life;
    let crawl = g.add_card_to_hand(0, catalog::wall_crawl());
    cast(&mut g, crawl);
    assert_eq!(g.players[0].life, life + 2, "two Spiders");
    let token = g.battlefield.iter().find(|c| c.definition.name == "Spider").unwrap().id;
    assert_eq!(pt(&g, token), (3, 2));
}

/// Blood Operative returns from the graveyard when you surveil and pay 3 life.
#[test]
fn blood_operative_returns_on_surveil() {
    let mut g = pod(2);
    let op = g.add_card_to_graveyard(0, catalog::blood_operative());
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let fate = g.add_card_to_hand(0, catalog::your_fate_ends_here());
    let life = g.players[0].life;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] },
        DecisionAnswer::Bool(true),
    ]));
    cast_with(&mut g, fate, Some(Target::Permanent(wurm)), None);
    assert!(g.players[0].hand.iter().any(|c| c.id == op), "back in hand");
    assert_eq!(g.players[0].life, life - 3);
}

/// Gixian Skullflayer grows at upkeep with three creature cards in your
/// graveyard, not two.
#[test]
fn gixian_skullflayer_needs_three_creature_cards() {
    let mut g = pod(2);
    let flayer = ready(&mut g, 0, catalog::gixian_skullflayer());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::hill_giant());
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!(plus(&g, flayer), 0);
    g.add_card_to_graveyard(0, catalog::craw_wurm());
    step_to(&mut g, 1, TurnStep::Upkeep);
    step_to(&mut g, 0, TurnStep::Draw);
    assert_eq!(plus(&g, flayer), 1);
}

// ── Batch 9 ─────────────────────────────────────────────────────────────────

/// Silumgar's Scorn: with a Dragon on the battlefield it counters outright;
/// without one the spell's controller may pay {1}.
#[test]
fn silumgars_scorn_hard_counters_with_a_dragon() {
    let run = |dragon: bool| {
        let mut g = pod(2);
        if dragon {
            ready(&mut g, 0, catalog::emerald_dragon());
        }
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.players[1].mana_pool.add_colorless(1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        let scorn = g.add_card_to_hand(0, catalog::silumgars_scorn());
        flood(&mut g);
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpell { card_id: scorn, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None })
            .expect("scorn");
        drain_stack(&mut g);
        g.players[0].life
    };
    assert_eq!(run(true), 20, "countered with a Dragon out");
}

/// Fearsome Awakening gives a reanimated Dragon two counters.
#[test]
fn fearsome_awakening_boosts_dragons() {
    let mut g = pod(2);
    let dragon = g.add_card_to_graveyard(0, catalog::emerald_dragon());
    let spell = g.add_card_to_hand(0, catalog::fearsome_awakening());
    cast_with(&mut g, spell, Some(Target::Permanent(dragon)), None);
    assert_eq!(plus(&g, dragon), 2);
}

/// Renari lets you cast an artifact at instant speed.
#[test]
fn renari_gives_artifacts_flash() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::renari_merchant_of_marvels());
    g.step = TurnStep::BeginCombat;
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring);
    assert!(g.battlefield_find(ring).is_some());
}

/// Master Skald: the creature card exiled is the controller's pick (it was the
/// first in the graveyard); the artifact comes back.
#[test]
fn master_skald_exiles_the_chosen_creature_card() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_graveyard(0, catalog::craw_wurm());
    let ring = g.add_card_to_graveyard(0, catalog::sol_ring());
    let skald = g.add_card_to_hand(0, catalog::master_skald());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![wurm])]));
    cast_with(&mut g, skald, None, None);
    assert!(g.exile.iter().any(|c| c.id == wurm));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear));
    assert!(g.players[0].hand.iter().any(|c| c.id == ring));
}
