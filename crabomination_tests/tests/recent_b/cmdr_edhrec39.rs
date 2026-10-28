//! Commander: the 1500 rescan's one- and two-card gaps for Kwia, Kyodai,
//! Eivor, Ashaya, The Mighty Thor, Azula, King Macar, Hurska, RMS Titanic and
//! Mu Yanling (`decks::cmdr_edhrec39`).

use crabomination::card::{CardId, CounterType, Keyword, Value};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer as A, ScriptedDecider};
use crabomination::effect::{Effect, Selector};
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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, targets: &[Target], x: Option<u32>) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_x(g, seat, id, targets, None)
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) -> Result<(), String> {
    cast_as(g, 0, id, targets)
}

fn activate_x(g: &mut GameState, id: CardId, index: usize, target: Option<Target>, x: Option<u32>) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: x,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) -> Result<(), String> {
    activate_x(g, id, index, target, None)
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn library(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

fn tokens_named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.is_token && c.definition.name == name).count()
}

fn declare(g: &mut GameState, attacks: &[(CardId, usize)]) -> Result<(), String> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attacks.iter().map(|&(a, d)| Attack { attacker: a, target: AttackTarget::Player(d) }).collect(),
    ))
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn combat_damage(g: &mut GameState) {
    g.step = TurnStep::CombatDamage;
    let ev = g.resolve_combat().expect("damage");
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).unwrap();
    (c.power, c.toughness)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

/// Add one lore counter to `saga` the way an effect would, firing its chapter.
fn lore(g: &mut GameState, saga: CardId) {
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let add = Effect::AddCounter { what: Selector::ExactObjects(vec![saga]), kind: CounterType::Lore, amount: Value::ONE };
    let evs = g.resolve_effect(&add, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(g);
}

fn in_hand(g: &GameState, seat: usize, id: CardId) -> bool {
    g.players[seat].hand.iter().any(|c| c.id == id)
}

/// Enlightened Confidant: with life gained this turn, its end-step surveil
/// returns a binned card whose mana value is at most that gain.
#[test]
fn enlightened_confidant_returns_a_cheap_surveiled_card() {
    let mut g = pod(2);
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    library(&mut g, 0, 3);
    ready(&mut g, 0, catalog::enlightened_confidant());
    g.players[0].life_gained_this_turn = 2;
    g.decider = Box::new(ScriptedDecider::new([A::ScryOrder { kept_top: vec![], bottom: vec![top] }]));
    g.step = TurnStep::PostCombatMain;
    advance_to(&mut g, TurnStep::End);
    assert!(in_hand(&g, 0, top), "Grizzly Bears (2) is at most the 2 gained");
}

/// Myojin of Blooming Dawn: cast, it keeps an indestructible counter;
/// removing it makes a Spirit per permanent you control.
#[test]
fn myojin_of_blooming_dawn_spends_its_counter_on_spirits() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::plains());
    ready(&mut g, 0, catalog::plains());
    let myojin = g.add_card_to_hand(0, catalog::myojin_of_blooming_dawn());
    cast(&mut g, myojin, &[]).expect("myojin");
    assert_eq!(g.battlefield_find(myojin).unwrap().counter_count(CounterType::Indestructible), 1);
    activate(&mut g, myojin, 0, None).expect("remove the counter");
    assert_eq!(tokens_named(&g, "Spirit"), 3, "Myojin and two Plains");
}

/// Eivor, Wolf-Kissed: seven combat damage mills seven; a Saga and a land
/// from among them enter.
#[test]
fn eivor_puts_a_saga_and_a_land_from_its_mill() {
    let mut g = pod(2);
    let saga = g.add_card_to_library(0, catalog::fall_of_the_first_civilization());
    let land = g.add_card_to_library(0, catalog::plains());
    library(&mut g, 0, 8);
    let eivor = ready(&mut g, 0, catalog::eivor_wolf_kissed());
    declare(&mut g, &[(eivor, 1)]).expect("attack");
    g.decider = Box::new(ScriptedDecider::new([A::Bool(true)]));
    combat_damage(&mut g);
    assert!(g.battlefield_find(saga).is_some() && g.battlefield_find(land).is_some());
    assert_eq!(g.players[0].graveyard.len(), 5, "the other five stay milled");
}

/// Fall of the First Civilization III: each player keeps three nonland
/// permanents; the rest are destroyed (an indestructible one survives).
#[test]
fn fall_of_the_first_civilization_keeps_three_each() {
    let mut g = pod(2);
    let fall = ready(&mut g, 0, catalog::fall_of_the_first_civilization());
    let theirs: Vec<CardId> = (0..4).map(|_| ready(&mut g, 1, catalog::grizzly_bears())).collect();
    let land = ready(&mut g, 1, catalog::plains());
    g.battlefield_find_mut(fall).unwrap().add_counters(CounterType::Lore, 2);
    lore(&mut g, fall);
    assert_eq!(theirs.iter().filter(|b| g.battlefield_find(**b).is_some()).count(), 3);
    assert!(g.battlefield_find(land).is_some(), "lands stay");
}

/// Ashaya's Enduring Bond: a legendary sorcery commander (CR 903.3a) that
/// discovers by the mana spent on it.
#[test]
fn ashayas_enduring_bond_discovers_by_mana_spent() {
    let mut g = pod(2);
    let def = catalog::ashayas_enduring_bond();
    assert!(def.can_be_commander);
    ready(&mut g, 0, catalog::nico_minoru_runaway());
    let three = g.add_card_to_library(0, catalog::serra_angel());
    let two = g.add_card_to_library(0, catalog::grizzly_bears());
    let bond = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bond, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("legendary sorcery with a legend out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(two).is_some() || in_hand(&g, 0, two), "Bears (2) is discovered past Serra (5)");
    assert!(!in_hand(&g, 0, three) && g.battlefield_find(three).is_none());
}

/// Nico Minoru: a spell cast from outside the hand pings each opponent;
/// its ability exiles to a nonland card and casts it free.
#[test]
fn nico_minoru_casts_from_the_top_and_pings() {
    let mut g = pod(3);
    let nico = ready(&mut g, 0, catalog::nico_minoru_runaway());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([A::Bool(true)]));
    activate(&mut g, nico, 0, None).expect("activate");
    assert!(g.battlefield_find(bears).is_some(), "cast free from exile");
    assert_eq!((g.players[1].life, g.players[2].life), (18, 18), "2 to each opponent");
}

/// Bespoke Bō bounces another nonland permanent and suits up its bearer;
/// Ironheart gives a noncreature spell improvise.
#[test]
fn bespoke_bo_and_ironheart() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let bo = g.add_card_to_hand(0, catalog::bespoke_bo());
    cast(&mut g, bo, &[perm(ring)]).expect("bo");
    assert!(in_hand(&g, 1, ring));
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    flood(&mut g, 0);
    g.perform_action(GameAction::Equip { equipment: bo, target: bear }).expect("equip");
    assert_eq!(pt(&g, bear), (4, 3));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Vigilance));
    let ironheart = ready(&mut g, 0, catalog::ironheart_clever_champion());
    let rings: Vec<CardId> = (0..2).map(|_| ready(&mut g, 0, catalog::sol_ring())).collect();
    library(&mut g, 0, 2);
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: div,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures: rings.clone(),
    })
    .expect("Divination improvised by two artifacts");
    drain_stack(&mut g);
    assert!(rings.iter().all(|r| g.battlefield_find(*r).unwrap().tapped));
    assert!(!g.battlefield_find(ironheart).unwrap().tapped);
}

/// Hama, the Bloodbender: the opponent mills three; a noncreature spell among
/// their graveyard is exiled and castable for its mana value in any mana.
#[test]
fn hama_exiles_a_spell_to_cast() {
    let mut g = pod(2);
    let bolt = g.add_card_to_library(1, catalog::lightning_bolt());
    for _ in 0..2 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let hama = g.add_card_to_hand(0, catalog::hama_the_bloodbender());
    g.decider = Box::new(ScriptedDecider::new([A::Bool(true)]));
    cast(&mut g, hama, &[Target::Player(1)]).expect("hama");
    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("the Bolt is exiled");
    assert!(exiled.may_play_until.is_some(), "castable from exile");
    // CR 611.2b — "during your turn", "for as long as you control Hama".
    g.players[0].mana_pool.add(Color::Red, 1);
    let shoot = |g: &mut GameState| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastFromZoneWithoutPaying {
            card_id: bolt,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    g.active_player_idx = 1;
    assert!(shoot(&mut g).is_err(), "not on an opponent's turn");
    g.active_player_idx = 0;
    let mut stolen = g.clone();
    stolen.battlefield_find_mut(hama).unwrap().controller = 1;
    stolen.dispatch_triggers_for_events(&[GameEvent::ControlChanged { card_id: hama, from: 0, to: 1 }]);
    assert!(stolen.exile.iter().any(|c| c.id == bolt && c.may_play_until.is_none()), "ends as Hama is stolen");
    shoot(&mut g).expect("on Hama's controller's turn");
}

/// Roll-Roll-Roll-Roll: each chapter blinks a creature of yours until the
/// next end step.
#[test]
fn roll_roll_blinks_until_the_end_step() {
    let mut g = pod(2);
    let saga = g.add_card_to_hand(0, catalog::roll_roll_roll_roll());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    cast(&mut g, saga, &[perm(bear)]).expect("saga");
    assert!(g.battlefield_find(bear).is_none(), "exiled");
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "back at the end step");
}

/// Mobile Garrison untaps another artifact of yours when it attacks;
/// Fortuitous Find returns an artifact and a creature.
#[test]
fn mobile_garrison_and_fortuitous_find() {
    let mut g = pod(2);
    let garrison = ready(&mut g, 0, catalog::mobile_garrison());
    let ring = ready(&mut g, 0, catalog::sol_ring());
    g.battlefield_find_mut(ring).unwrap().tapped = true;
    g.battlefield_find_mut(garrison).unwrap().definition_make_mut().card_types.push(crabomination::card::CardType::Creature);
    declare(&mut g, &[(garrison, 1)]).expect("attack");
    assert!(!g.battlefield_find(ring).unwrap().tapped, "untapped");
    let a = g.add_card_to_graveyard(0, catalog::sol_ring());
    let c = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let find = g.add_card_to_hand(0, catalog::fortuitous_find());
    flood(&mut g, 0);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellSpree {
        card_id: find,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(c)],
        x_value: None,
    })
        .expect("both modes");
    drain_stack(&mut g);
    assert!(in_hand(&g, 0, a) && in_hand(&g, 0, c));
}

/// Hurska Sweet-Tooth: attacking makes a Food; a life gain paid with {G/W}
/// pumps by the amount. Heroic Feast spreads that many counters.
#[test]
fn hurska_and_heroic_feast_feed_on_life_gain() {
    let mut g = pod(2);
    let hurska = ready(&mut g, 0, catalog::hurska_sweet_tooth());
    declare(&mut g, &[(hurska, 1)]).expect("attack");
    assert_eq!(tokens_named(&g, "Food"), 1);
    let feast = g.add_card_to_hand(0, catalog::heroic_feast());
    g.step = TurnStep::PostCombatMain;
    cast(&mut g, feast, &[]).expect("feast");
    assert_eq!(tokens_named(&g, "Food"), 2);
    let bears: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    let salve = g.add_card_to_hand(0, catalog::healing_salve());
    g.decider = Box::new(ScriptedDecider::new([A::Bool(true)]));
    cast(&mut g, salve, &[Target::Player(0)]).expect("gain 3");
    let countered = bears.iter().filter(|b| g.battlefield_find(**b).unwrap().counter_count(CounterType::PlusOnePlusOne) == 1).count();
    assert!(countered >= 2, "up to three counters among the bears, got {countered}");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && pt(&g, c.id).0 >= 5), "+3/+3 on a creature");
}

/// Dowsing Device: an artifact entering hastes a creature; the fourth
/// artifact flips it to Geode Grotto. Fatal Frenzy doubles power and the
/// creature is sacrificed at the end step.
#[test]
fn dowsing_device_flips_and_fatal_frenzy_doubles() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let device = g.add_card_to_hand(0, catalog::dowsing_device());
    cast(&mut g, device, &[perm(bear)]).expect("device");
    assert_eq!(pt(&g, bear), (3, 2));
    for _ in 0..3 {
        let ring = g.add_card_to_hand(0, catalog::sol_ring());
        cast(&mut g, ring, &[]).expect("ring");
    }
    assert_eq!(g.battlefield_find(device).unwrap().definition.name, "Geode Grotto");
    let frenzy = g.add_card_to_hand(0, catalog::fatal_frenzy());
    let ogre = ready(&mut g, 0, catalog::serra_angel());
    cast(&mut g, frenzy, &[perm(ogre)]).expect("frenzy");
    assert_eq!(pt(&g, ogre).0, 8);
    assert!(g.computed_permanent(ogre).unwrap().keywords().contains(&Keyword::Trample));
    advance_to(&mut g, TurnStep::End);
    assert!(g.battlefield_find(ogre).is_none(), "sacrificed");
}

/// Mu Yanling, Sky Dancer: +2 grounds a flier until your next turn; −3 makes
/// a 4/4 flier. Anchor to Reality trades a creature for a cheaper Equipment
/// and scries 2.
#[test]
fn mu_yanling_and_anchor_to_reality() {
    let mut g = pod(2);
    let mu = ready(&mut g, 0, catalog::mu_yanling_sky_dancer());
    let angel = ready(&mut g, 1, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: mu, ability_index: 0, target: Some(perm(angel)), x_value: None })
        .expect("+2");
    drain_stack(&mut g);
    let c = g.computed_permanent(angel).unwrap();
    assert_eq!(c.power, 2);
    assert!(!c.keywords().contains(&Keyword::Flying));
    let sword = g.add_card_to_library(0, catalog::bonesplitter());
    library(&mut g, 0, 3);
    let fodder = ready(&mut g, 0, catalog::serra_angel());
    let anchor = g.add_card_to_hand(0, catalog::anchor_to_reality());
    cast(&mut g, anchor, &[]).expect("anchor");
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed");
    assert!(g.battlefield_find(sword).is_some(), "Bonesplitter fetched");
}

/// Every Myojin "enters with a [divinity / indestructible] counter on it if
/// you cast it from your hand": cast from hand, it has one; put onto the
/// battlefield from a graveyard (not cast), it doesn't (CR 614.1c).
#[test]
fn myojins_count_only_a_cast_from_hand() {
    use crabomination::card::CounterType;
    use crabomination::effect::{Effect, Selector, ZoneDest};
    let myojins: [(fn() -> crabomination::card::CardDefinition, CounterType); 10] = [
        (catalog::myojin_of_cleansing_fire, CounterType::Divinity),
        (catalog::myojin_of_infinite_rage, CounterType::Divinity),
        (catalog::myojin_of_seeing_winds, CounterType::Divinity),
        (catalog::myojin_of_lifes_web, CounterType::Divinity),
        (catalog::myojin_of_cryptic_dreams, CounterType::Indestructible),
        (catalog::myojin_of_grim_betrayal, CounterType::Indestructible),
        (catalog::myojin_of_roaring_blades, CounterType::Indestructible),
        (catalog::myojin_of_towering_might, CounterType::Indestructible),
        (catalog::myojin_of_nights_reach, CounterType::Divinity),
        (catalog::myojin_of_blooming_dawn, CounterType::Indestructible),
    ];
    // An indestructible counter is a keyword counter (CR 122.1b); either
    // store counts.
    let has = |c: &crabomination::card::CardInstance, kind: CounterType| {
        c.counter_count(kind)
            + if kind == CounterType::Indestructible {
                c.keyword_counters.get(&Keyword::Indestructible).copied().unwrap_or(0)
            } else {
                0
            }
    };
    for (f, kind) in myojins {
        let name = f().name;
        let mut g = pod(2);
        let id = g.add_card_to_hand(0, f());
        cast(&mut g, id, &[]).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(g.battlefield_find(id).map(|c| has(c, kind)), Some(1), "{name} cast from hand");
        let mut g = pod(2);
        let id = g.add_card_to_graveyard(0, f());
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let to = ZoneDest::Battlefield { controller: crabomination::effect::PlayerRef::You, tapped: false };
        let evs = g.resolve_effect(&Effect::Move { what: Selector::ExactObjects(vec![id]), to }, &ctx).expect("reanimate");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).map(|c| has(c, kind)), Some(0), "{name} put onto the battlefield");
        // Cast, but from exile: still no counter.
        let mut g = pod(2);
        let id = g.add_card_to_exile(0, f());
        let ctx = EffectContext::for_spell(0, None, 0, 0);
        let cast_it = Effect::CastWithoutPayingImmediate {
            what: Selector::ExactObjects(vec![id]),
            source_zone: crabomination::card::Zone::Exile,
            exile_after: false,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: false,
        };
        g.decider = Box::new(ScriptedDecider::new([A::Bool(true)]));
        let evs = g.resolve_effect(&cast_it, &ctx).expect("cast from exile");
        g.dispatch_triggers_for_events(&evs);
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(id).map(|c| has(c, kind)), Some(0), "{name} cast from exile");
    }
}
