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
    while g.active_player_idx != seat || g.step != step {
        let evs = g.advance_step(Vec::new()).unwrap();
        g.dispatch_triggers_for_events(&evs);
        drain_stack(g);
    }
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
