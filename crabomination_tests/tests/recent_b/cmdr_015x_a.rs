//! Commander: the 2026-10-18 (`015XaGkQ`) one-deck closers
//! (`decks::cmdr_015x_a`).

use crabomination::card::{CardId, CounterType};
use crabomination::catalog;
use crabomination::effect::{Effect, Selector, Value};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::forest());
        }
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn flood(g: &mut GameState) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 10);
    }
}

fn activate(g: &mut GameState, card_id: CardId, ability_index: usize, x_value: Option<u32>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index,
        target: None,
        additional_targets: vec![],
        x_value,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn cast(g: &mut GameState, card_id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

/// April O'Neil draws one card per card type among the spells cast this
/// turn: an instant and an artifact are two, however many spells.
#[test]
fn april_oneil_draws_per_card_type_cast() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::april_oneil_hacktivist());
    flood(&mut g);
    for def in [catalog::lightning_bolt(), catalog::shock()] {
        let id = g.add_card_to_hand(0, def);
        cast(&mut g, id, Some(Target::Player(1)));
    }
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    let hand = g.players[0].hand.len();
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2);
}

/// Bruce Banner draws X for {X}{X} and transforms into the Hulk, whose
/// enrage while attacking untaps him and banks another combat (CR 500.8).
#[test]
fn bruce_banner_draws_and_hulks_out() {
    let mut g = pod(2);
    let bruce = ready(&mut g, 0, catalog::bruce_banner());
    flood(&mut g);
    activate(&mut g, bruce, 0, Some(2));
    assert_eq!(g.players[0].hand.len(), 2);
    activate(&mut g, bruce, 1, None);
    assert_eq!(g.battlefield_find(bruce).unwrap().definition.name, "The Incredible Hulk");
    g.battlefield_find_mut(bruce).unwrap().tapped = false;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bruce, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bruce).unwrap().tapped);
    let bolt = Effect::DealDamage { to: Selector::ExactObjects(vec![bruce]), amount: Value::Const(3) };
    let evs = g.resolve_effect(&bolt, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let hulk = g.battlefield_find(bruce).unwrap();
    assert!(!hulk.tapped, "untapped");
    assert_eq!(hulk.counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.additional_combat_phases, 1);
}

/// CR 702.188 — Amazing Spider-Man gives a colored legendary spell
/// web-slinging {G}{W}{U}: it is cast for that, returning a tapped creature.
#[test]
fn amazing_spider_man_grants_web_slinging_to_legendary_spells() {
    let mut g = pod(2);
    let peter = ready(&mut g, 0, catalog::peter_parker());
    flood(&mut g);
    activate(&mut g, peter, 0, None);
    assert_eq!(g.battlefield_find(peter).unwrap().definition.name, "Amazing Spider-Man");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.players[0].mana_pool = Default::default();
    for c in [Color::Green, Color::White, Color::Blue] {
        g.players[0].mana_pool.add(c, 1);
    }
    let legend = g.add_card_to_hand(0, catalog::isamaru_hound_of_konda());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: legend,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("web-slinging");
    drain_stack(&mut g);
    assert!(g.battlefield_find(legend).is_some());
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "the tapped creature went home");
}

/// Peter Parker's entry makes a 2/1 reach Spider.
#[test]
fn peter_parker_enters_with_a_spider() {
    let mut g = pod(2);
    flood(&mut g);
    let peter = g.add_card_to_hand(0, catalog::peter_parker());
    cast(&mut g, peter, None);
    let spider = g.battlefield.iter().find(|c| c.definition.name == "Spider").expect("a Spider");
    assert_eq!((spider.definition.power, spider.definition.toughness), (2, 1));
}

/// CR 508.1d — Public Enemy on seat 1's creature: seat 2's creatures must
/// attack seat 1 if able — not seat 0, and not stay home. The enchanted
/// creature dying draws the Aura's controller a card.
#[test]
fn public_enemy_makes_everyone_attack_its_hosts_controller() {
    let mut g = pod(3);
    let host = ready(&mut g, 1, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::public_enemy());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    let bear = ready(&mut g, 2, catalog::grizzly_bears());
    g.active_player_idx = 2;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 2;
    let declare = |g: &mut GameState, to: Option<usize>| {
        let attacks = to.map(|p| Attack { attacker: bear, target: AttackTarget::Player(p) }).into_iter().collect();
        g.clone().perform_action(GameAction::DeclareAttackers(attacks)).is_ok()
    };
    assert!(!declare(&mut g, None), "it must attack");
    assert!(!declare(&mut g, Some(0)), "and it must attack the host's controller");
    assert!(declare(&mut g, Some(1)));
    let hand = g.players[0].hand.len();
    let kill = Effect::Destroy { what: Selector::ExactObjects(vec![host]) };
    let evs = g.resolve_effect(&kill, &EffectContext::for_spell(2, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Maraxus of Keld counts your untapped artifacts, creatures and lands —
/// itself included — and a tap shrinks it (CR 604.3).
#[test]
fn maraxus_of_keld_counts_untapped_artifacts_creatures_and_lands() {
    let mut g = pod(2);
    let maraxus = ready(&mut g, 0, catalog::maraxus_of_keld());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    g.add_card_to_battlefield(1, catalog::forest());
    let pt = |g: &GameState| g.computed_permanent(maraxus).map(|cp| (cp.power, cp.toughness));
    assert_eq!(pt(&g), Some((3, 3)));
    g.battlefield_find_mut(land).unwrap().tapped = true;
    assert_eq!(pt(&g), Some((2, 2)));
}

/// Codie copies a prepared spell (Peer Review: two Cadets, not one) and
/// re-prepares an unprepared Fateseer for five colors and a tap.
#[test]
fn codie_copies_prepared_spells_and_prepares_the_team() {
    let mut g = pod(2);
    let codie = ready(&mut g, 0, catalog::codie_ravenous_codex());
    let seer = ready(&mut g, 0, catalog::prudent_fateseer());
    g.battlefield_find_mut(seer).unwrap().add_counters(CounterType::Prepared, 1);
    flood(&mut g);
    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: seer,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Peer Review");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Cadet").count(), 2);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Prepared), 0);
    activate(&mut g, codie, 0, None);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Prepared), 1);
    // An ordinary spell is no prepared spell.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17, "one Bolt, no copy");
}

/// Prowl exiles a tapped creature as it attacks; that card's owner plays it
/// from exile, and Prowl's controller draws while Prowl converts.
#[test]
fn prowl_exiles_a_tapped_creature_and_converts_when_it_is_played() {
    let mut g = pod(2);
    let prowl = ready(&mut g, 0, catalog::prowl_stoic_strategist());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: prowl, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the tapped bear is exiled");
    g.step = TurnStep::PostCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Green, 2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("its owner casts it from exile");
    assert_eq!(g.players[1].mana_pool.total(), 0, "paying its own cost");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.battlefield_find(prowl).unwrap().definition.name, "Prowl, Pursuit Vehicle");
}

/// Nick Fury's power-up: two counters, then a Hero off the top seven onto
/// the battlefield; the rest go to the bottom. Only once (CR 702.193).
#[test]
fn nick_fury_powers_up_into_a_hero() {
    let mut g = pod(2);
    let fury = ready(&mut g, 0, catalog::nick_fury_agent_of_shield());
    g.players[0].library.clear();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::island());
    }
    let hero = g.add_card_to_library(0, catalog::peter_parker());
    flood(&mut g);
    activate(&mut g, fury, 0, None);
    assert_eq!(g.battlefield_find(fury).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert!(g.battlefield_find(hero).is_some(), "Peter Parker is a Hero");
    assert_eq!(g.players[0].library.len(), 6);
    flood(&mut g);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: fury,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "a power-up activates once"
    );
}

/// Miles Morales counters two creatures as he enters; as Ultimate
/// Spider-Man, attacking doubles the counters on Spiders and legends (not
/// on the plain bear), and Camouflage leaves him colorless and hexproof.
#[test]
fn miles_morales_counters_then_doubles_them_on_attack() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let isamaru = ready(&mut g, 0, catalog::isamaru_hound_of_konda());
    flood(&mut g);
    let miles = g.add_card_to_hand(0, catalog::miles_morales());
    g.perform_action(GameAction::CastSpell { card_id: miles, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let plus = |g: &GameState, id: CardId| g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    assert_eq!(plus(&g, bear) + plus(&g, isamaru), 2, "two counters on up to two creatures");
    g.battlefield_find_mut(bear).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.battlefield_find_mut(isamaru).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    let (b0, i0) = (plus(&g, bear), plus(&g, isamaru));
    flood(&mut g);
    activate(&mut g, miles, 0, None);
    assert_eq!(g.battlefield_find(miles).unwrap().definition.name, "Ultimate Spider-Man");
    activate(&mut g, miles, 0, None);
    let cp = g.computed_permanent(miles).unwrap();
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Hexproof));
    assert!(cp.colors.is_empty(), "colorless until end of turn");
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: miles, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(plus(&g, isamaru), 2 * i0, "a legend's counters double");
    assert_eq!(plus(&g, bear), b0, "a plain bear's don't");
    assert_eq!(plus(&g, miles), 2, "nor do his own fail to");
}

/// Multiversal Passage enters as the chosen basic type and taps for it.
#[test]
fn multiversal_passage_is_the_chosen_type() {
    let mut g = pod(2);
    let land = g.add_card_to_hand(0, catalog::multiversal_passage());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    g.battlefield_find_mut(land).unwrap().tapped = false;
    assert_eq!(g.computed_permanent(land).unwrap().subtypes().land_types.len(), 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Falco Spara casts the library top by removing a counter from a creature
/// you control (its own shield counter here); with no counter left, it can't.
#[test]
fn falco_spara_casts_from_the_top_for_a_counter() {
    let mut g = pod(2);
    let falco = ready(&mut g, 0, catalog::falco_spara_pactweaver());
    g.battlefield_find_mut(falco).unwrap().add_counters(CounterType::Shield, 1);
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].library.rotate_right(1);
    assert_eq!(g.players[0].library[0].id, bear);
    flood(&mut g);
    cast(&mut g, bear, None);
    assert!(g.battlefield_find(bear).is_some(), "cast off the top");
    assert_eq!(g.battlefield_find(falco).unwrap().counter_count(CounterType::Shield), 0, "the counter paid for it");
    let next = g.players[0].library[0].id;
    let bear2 = g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].library.rotate_right(1);
    assert_ne!(next, bear2);
    assert!(
        g.perform_action(GameAction::CastSpell { card_id: bear2, target: None, additional_targets: vec![], mode: None, x_value: None })
            .is_err(),
        "no counter, no cast"
    );
}

/// Evelyn exiles each player's top card with a collection counter; her
/// controller may cast one of them a turn, with mana of any color.
#[test]
fn evelyn_collects_each_top_card_and_casts_one_a_turn() {
    let mut g = pod(2);
    for seat in 0..2 {
        let bear = g.add_card_to_library(seat, catalog::grizzly_bears());
        g.players[seat].library.rotate_right(1);
        assert_eq!(g.players[seat].library[0].id, bear);
    }
    let theirs = g.players[1].library[0].id;
    let mine = g.players[0].library[0].id;
    flood(&mut g);
    let evelyn = g.add_card_to_hand(0, catalog::evelyn_the_covetous());
    cast(&mut g, evelyn, None);
    for id in [theirs, mine] {
        let c = g.exile.iter().find(|c| c.id == id).expect("exiled");
        assert_eq!(c.counter_count(CounterType::Collection), 1);
    }
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: theirs,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("an opponent's bear, paid with blue");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).map(|c| c.controller), Some(0));
    assert!(
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: mine,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "once each turn"
    );
}

/// Garza Zol grows when a creature it damaged this turn dies.
#[test]
fn garza_zol_grows_off_its_kills() {
    let mut g = pod(2);
    let garza = ready(&mut g, 0, catalog::garza_zol_plague_queen());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let hit = Effect::DealDamage { to: Selector::ExactObjects(vec![bear]), amount: Value::Const(5) };
    let mut ctx = EffectContext::for_spell(0, None, 0, 0);
    ctx.source = Some(garza);
    let evs = g.resolve_effect(&hit, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    assert_eq!(g.battlefield_find(garza).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

fn attack(g: &mut GameState, attackers: &[CardId]) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

/// Optimus Prime, Hero bolsters 1 each end step and comes back converted
/// when it dies; the Autobot Leader bolsters 2 as you attack, and that
/// creature's combat damage converts Optimus back.
#[test]
fn optimus_prime_bolsters_and_converts() {
    let mut g = pod(2);
    let optimus = ready(&mut g, 0, catalog::optimus_prime_hero());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    let plus = |g: &GameState, id: CardId| g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne));
    assert_eq!(plus(&g, bear), 1, "the bear has the least toughness");
    let kill = Effect::Destroy { what: Selector::ExactObjects(vec![optimus]) };
    let evs = g.resolve_effect(&kill, &EffectContext::for_spell(1, None, 0, 0)).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let leader = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Optimus Prime, Autobot Leader")
        .map(|c| c.id)
        .expect("back, converted");
    g.clear_sickness(leader);
    g.active_player_idx = 0;
    attack(&mut g, &[bear]);
    assert_eq!(plus(&g, bear), 3, "bolster 2");
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&crabomination::card::Keyword::Trample));
    g.step = TurnStep::CombatDamage;
    let evs = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(leader).unwrap().definition.name, "Optimus Prime, Hero");
}

/// Ultra Magnus puts an artifact creature from hand in attacking and
/// converts at end of combat; the Armored Carrier's Formidable makes the
/// attackers indestructible and converts it back at 8 total power.
#[test]
fn ultra_magnus_deploys_then_converts() {
    let mut g = pod(2);
    let magnus = ready(&mut g, 0, catalog::ultra_magnus_tactician());
    let thopter = g.add_card_to_hand(0, catalog::ornithopter());
    attack(&mut g, &[magnus]);
    assert!(g.attacking.iter().any(|a| a.attacker == thopter), "deployed attacking");
    g.step = TurnStep::CombatDamage;
    while g.step != TurnStep::PostCombatMain && g.game_over.is_none() {
        g.perform_action(GameAction::PassPriority).ok();
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(magnus).unwrap().definition.name, "Ultra Magnus, Armored Carrier");
    g.battlefield_find_mut(magnus).unwrap().tapped = false;
    let big = ready(&mut g, 0, catalog::craw_wurm());
    g.attacking.clear();
    attack(&mut g, &[magnus, big]);
    assert!(g.computed_permanent(big).unwrap().keywords().contains(&crabomination::card::Keyword::Indestructible));
    assert_eq!(g.battlefield_find(magnus).unwrap().definition.name, "Ultra Magnus, Tactician", "4 + 6 power converts it");
}

/// Rootpath Purifier — a search for a basic land card finds Command Tower in
/// your library (its ruling), and a nonbasic land you control stops being
/// nonbasic; an opponent's library and lands are untouched.
#[test]
fn rootpath_purifier_makes_your_lands_basic() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::{PlayerRef, ZoneDest};
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rootpath_purifier());
    let tower = g.add_card_to_battlefield(0, catalog::command_tower());
    let theirs = g.add_card_to_battlefield(1, catalog::command_tower());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    assert!(!g.evaluate_requirement_static(&R::IsNonbasicLand, &Target::Permanent(tower), 0, None));
    assert!(g.evaluate_requirement_static(&R::IsNonbasicLand, &Target::Permanent(theirs), 0, None));
    g.players[0].library.clear();
    let card = g.add_card_to_library(0, catalog::command_tower());
    let search = Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) };
    g.resolve_effect(&search, &ctx).expect("search");
    assert!(g.players[0].hand.iter().any(|c| c.id == card), "Command Tower counts as basic");
}

/// Boiling Rock Rioter taps itself (an Ally) to exile an Ally card from your
/// graveyard, then casts it from exile as it attacks, paying its cost.
#[test]
fn boiling_rock_rioter_recasts_an_exiled_ally() {
    let mut g = pod(2);
    let rioter = ready(&mut g, 0, catalog::boiling_rock_rioter());
    let ally = g.add_card_to_graveyard(0, catalog::boiling_rock_rioter());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rioter,
        ability_index: 0,
        target: Some(Target::Permanent(ally)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap an Ally: exile");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == ally && c.exiled_with == Some(rioter)));
    g.battlefield_find_mut(rioter).unwrap().tapped = false;
    flood(&mut g);
    attack(&mut g, &[rioter]);
    assert!(g.battlefield_find(ally).is_some(), "cast from exile as it attacked");
}

/// Nahiri's Wrath: discard X = 2 (Hill Giant and Grizzly Bears, total mana
/// value 6) to deal 6 to each of two targets.
#[test]
fn nahiris_wrath_deals_the_discarded_total() {
    let mut g = pod(2);
    g.players[0].hand.clear();
    let wurm = ready(&mut g, 1, catalog::craw_wurm());
    let other = ready(&mut g, 1, catalog::serra_angel());
    g.add_card_to_hand(0, catalog::hill_giant());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::nahiris_wrath());
    flood(&mut g);
    g.perform_action(GameAction::CastSpell {
        card_id: wrath,
        target: Some(Target::Permanent(wurm)),
        additional_targets: vec![Target::Permanent(other)],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "both discards and the Wrath");
    assert!(g.battlefield_find(wurm).is_none(), "6 damage kills the 6/4");
    assert!(g.battlefield_find(other).is_none(), "and the 4/4");
}

/// Nathan Drake exiles each top card as he attacks and casts the one he
/// doesn't own, paying with mana of the wrong color.
#[test]
fn nathan_drake_casts_an_opponents_exiled_spell() {
    let mut g = pod(2);
    let drake = ready(&mut g, 0, catalog::nathan_drake_treasure_hunter());
    let ox = g.add_card_to_library(1, catalog::grizzly_bears());
    g.players[1].library.rotate_right(1);
    let mine = g.players[0].library[0].id;
    g.players[0].mana_pool.add(Color::Blue, 2);
    attack(&mut g, &[drake]);
    assert!(g.exile.iter().any(|c| c.id == mine), "my top card is exiled too");
    assert_eq!(g.battlefield_find(ox).map(|c| c.controller), Some(0), "their bear, cast with blue");
}

/// Kotose exiles a card from an opponent's graveyard and its namesakes from
/// their hand and library; its controller may play one of them, with mana of
/// any color.
#[test]
fn kotose_steals_a_name_and_plays_one() {
    let mut g = pod(2);
    let anchor = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let in_hand = g.add_card_to_hand(1, catalog::grizzly_bears());
    let in_lib = g.add_card_to_library(1, catalog::grizzly_bears());
    flood(&mut g);
    let kotose = g.add_card_to_hand(0, catalog::kotose_the_silent_spider());
    cast(&mut g, kotose, None);
    for id in [anchor, in_hand, in_lib] {
        assert!(g.exile.iter().any(|c| c.id == id), "{id:?} swept");
    }
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: in_hand,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("their bear, with blue");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(in_hand).map(|c| c.controller), Some(0));
}

/// Flamewar converts off a sacrificed artifact; the Operative's combat damage
/// banks that many face-down intel cards and converts it back; the Veteran
/// trades its hand for them.
#[test]
fn flamewar_banks_intel_and_cashes_it_in() {
    let mut g = pod(2);
    let flamewar = ready(&mut g, 0, catalog::flamewar_brash_veteran());
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: flamewar,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice an artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none());
    assert_eq!(g.battlefield_find(flamewar).unwrap().definition.name, "Flamewar, Streetwise Operative");
    let library = g.players[0].library.len();
    attack(&mut g, &[flamewar]);
    g.step = TurnStep::CombatDamage;
    let evs = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let power = 3; // 2/1 plus the +1/+1 counter
    assert_eq!(g.players[0].library.len(), library - power);
    let intel = g.exile.iter().filter(|c| c.counter_count(CounterType::Intel) == 1 && c.face_down).count();
    assert_eq!(intel, power);
    assert_eq!(g.battlefield_find(flamewar).unwrap().definition.name, "Flamewar, Brash Veteran");
    g.step = TurnStep::PostCombatMain;
    flood(&mut g);
    activate(&mut g, flamewar, 1, None);
    assert_eq!(g.players[0].hand.len(), power, "the hand went, the intel came home");
}

/// Soundwave: a token's 2 combat damage exiles a mana value 2 instant from
/// the damaged player's graveyard and casts a copy (Raise the Alarm), which
/// converts it; the Captain converts back on an odd spell and makes Ravage.
#[test]
fn soundwave_copies_a_graveyard_spell_then_makes_ravage() {
    let mut g = pod(2);
    let wave = ready(&mut g, 0, catalog::soundwave_sonic_spy());
    let token = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(token).unwrap().is_token = true;
    let alarm = g.add_card_to_graveyard(1, catalog::raise_the_alarm());
    attack(&mut g, &[token]);
    g.step = TurnStep::CombatDamage;
    let evs = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == alarm), "exiled from their graveyard");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier" && c.controller == 0).count(), 2);
    assert_eq!(g.battlefield_find(wave).unwrap().definition.name, "Soundwave, Superior Captain");
    g.step = TurnStep::PostCombatMain;
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.battlefield_find(wave).unwrap().definition.name, "Soundwave, Sonic Spy");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ravage" && c.controller == 0));
}
