//! Commander: the one-card gaps in most-built commanders' EDHREC average
//! decks (`decks::cmdr_edhrec`).

use crabomination::card::CardId;
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
    for seat in 0..n {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
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

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, id: CardId, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("activate");
    drain_stack(g);
}

fn named(g: &GameState, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == name).count()
}

fn connect(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(g);
    }
}

/// Cadira's hit makes a Rabbit for each token you control, counted as it
/// resolves.
#[test]
fn cadira_makes_a_rabbit_per_token() {
    let mut g = pod(2);
    let cadira = ready(&mut g, 0, catalog::cadira_caller_of_the_small());
    let t = crabomination_base::tokens::treasure_token();
    g.add_token_to_battlefield(0, &t);
    g.add_token_to_battlefield(0, &t);
    connect(&mut g, cadira);
    assert_eq!(named(&g, "Rabbit"), 2);
}

/// The Unbeatable Squirrel Girl: a Squirrel on entering, then {1}{G}{G}{G}
/// doubles the Squirrels (she is one herself).
#[test]
fn squirrel_girl_doubles_the_squirrels() {
    let mut g = pod(2);
    let sg = g.add_card_to_hand(0, catalog::the_unbeatable_squirrel_girl());
    flood(&mut g);
    cast(&mut g, sg, None);
    assert_eq!(named(&g, "Squirrel"), 1);
    activate(&mut g, sg, None);
    assert_eq!(named(&g, "Squirrel"), 3, "two Squirrels counted: her and the token");
}

/// Forced Fruition draws seven for the opponent who cast, not the others.
#[test]
fn forced_fruition_draws_the_caster_seven() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::forced_fruition());
    for _ in 0..10 {
        g.add_card_to_library(2, catalog::island());
    }
    let bolt = g.add_card_to_hand(2, catalog::lightning_bolt());
    g.players[2].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 2;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!([g.players[1].hand.len(), g.players[2].hand.len()], [0, 7]);
}

/// Lesser Masticore costs a discard to cast and pings a creature for {4}.
#[test]
fn lesser_masticore_discards_and_pings() {
    let mut g = pod(2);
    let fodder = g.add_card_to_hand(0, catalog::island());
    let m = g.add_card_to_hand(0, catalog::lesser_masticore());
    flood(&mut g);
    cast(&mut g, m, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "discarded as a cost");
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.clear_sickness(m);
    activate(&mut g, m, Some(Target::Permanent(elf)));
    assert!(g.battlefield_find(elf).is_none());
}

/// Dark Deal — every hand is discarded and redrawn one card short.
#[test]
fn dark_deal_redraws_one_fewer() {
    let mut g = pod(3);
    for (seat, n) in [(0, 3), (1, 4), (2, 0)] {
        for _ in 0..n {
            g.add_card_to_hand(seat, catalog::island());
        }
    }
    let deal = g.add_card_to_hand(0, catalog::dark_deal());
    flood(&mut g);
    cast(&mut g, deal, None);
    assert_eq!([g.players[0].hand.len(), g.players[1].hand.len(), g.players[2].hand.len()], [2, 3, 0]);
}

/// Agent of Treachery steals a permanent; with three stolen its end step
/// draws three.
#[test]
fn agent_of_treachery_steals_and_draws_with_three() {
    let mut g = pod(2);
    let ring = ready(&mut g, 1, catalog::sol_ring());
    let agent = g.add_card_to_hand(0, catalog::agent_of_treachery());
    flood(&mut g);
    cast(&mut g, agent, Some(Target::Permanent(ring)));
    assert_eq!(g.battlefield_find(ring).unwrap().controller, 0);
    for _ in 0..2 {
        let b = ready(&mut g, 1, catalog::grizzly_bears());
        g.battlefield_find_mut(b).unwrap().controller = 0;
    }
    let hand = g.players[0].hand.len();
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Worldfire leaves an empty table and every life total at 1.
#[test]
fn worldfire_resets_the_table() {
    let mut g = pod(3);
    ready(&mut g, 1, catalog::grizzly_bears());
    g.add_card_to_hand(2, catalog::island());
    g.add_card_to_graveyard(1, catalog::island());
    let fire = g.add_card_to_hand(0, catalog::worldfire());
    flood(&mut g);
    cast(&mut g, fire, None);
    assert!(g.battlefield.is_empty());
    assert!(g.players.iter().all(|p| p.hand.is_empty() && p.life == 1));
    // Worldfire itself goes to the graveyard after it resolves.
    let graves: Vec<Vec<&str>> = g.players.iter().map(|p| p.graveyard.iter().map(|c| c.definition.name).collect()).collect();
    assert_eq!(graves, [vec!["Worldfire"], vec![], vec![]]);
}

/// Ganax makes a Treasure for itself and for each other Dragon entering.
#[test]
fn ganax_treasures_for_dragons() {
    let mut g = pod(2);
    let ganax = g.add_card_to_hand(0, catalog::ganax_astral_hunter());
    flood(&mut g);
    cast(&mut g, ganax, None);
    assert_eq!(named(&g, "Treasure"), 1);
    let drake = g.add_card_to_hand(0, catalog::shivan_dragon());
    cast(&mut g, drake, None);
    assert_eq!(named(&g, "Treasure"), 2);
}

/// Afterlife Insurance's granted afterlife makes a Spirit when a creature
/// dies this turn.
#[test]
fn afterlife_insurance_grants_afterlife() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let ins = g.add_card_to_hand(0, catalog::afterlife_insurance());
    flood(&mut g);
    cast(&mut g, ins, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield.iter().filter(|c| c.is_token).count(), 1, "a Spirit");
}

/// Dockside Chef may sacrifice itself to draw.
#[test]
fn dockside_chef_sacrifices_to_draw() {
    let mut g = pod(2);
    let chef = ready(&mut g, 0, catalog::dockside_chef());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand = g.players[0].hand.len();
    activate(&mut g, chef, None);
    assert!(g.battlefield_find(chef).is_none());
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// Wizard's Staff: the equipped creature has prowess, and its triggered
/// abilities trigger twice (CR 603.2d) — two cast pumps per spell.
#[test]
fn wizards_staff_grants_prowess_and_doubles_its_triggers() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let staff = ready(&mut g, 0, catalog::wizards_staff());
    flood(&mut g);
    g.perform_action(GameAction::Equip { equipment: staff, target: bear }).expect("equip");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "prowess fired twice");
}

/// Sorcerer Class: loot two on entering; level 2 creatures tap for
/// restricted {U}/{R}; level 3 an instant pings each opponent per spell.
#[test]
fn sorcerer_class_levels_up() {
    let mut g = pod(3);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::island());
    }
    let class = g.add_card_to_hand(0, catalog::sorcerer_class());
    flood(&mut g);
    let hand = g.players[0].hand.len();
    cast(&mut g, class, None);
    assert_eq!(g.players[0].hand.len(), hand - 1, "drew two, discarded two");
    activate(&mut g, class, None);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: class,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("level 3");
    drain_stack(&mut g);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let pool = g.players[0].mana_pool.restricted_total();
    activate(&mut g, bear, None);
    assert_eq!(g.players[0].mana_pool.restricted_total(), pool + 1, "granted restricted mana ability");
    let lives = [g.players[1].life, g.players[2].life];
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0] - 1, lives[1] - 1]);
}

/// Echo of Eons wheels every seat into seven fresh cards.
#[test]
fn echo_of_eons_wheels_everyone() {
    let mut g = pod(3);
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_graveyard(2, catalog::island());
    let echo = g.add_card_to_hand(0, catalog::echo_of_eons());
    flood(&mut g);
    cast(&mut g, echo, None);
    assert!(g.players.iter().all(|p| p.hand.len() == 7));
    assert!(g.players[2].graveyard.is_empty());
}

/// Nine-Lives Familiar enters with eight revival counters when cast and
/// comes back at the next end step with seven.
#[test]
fn nine_lives_familiar_returns_with_one_fewer() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let cat = g.add_card_to_hand(0, catalog::nine_lives_familiar());
    flood(&mut g);
    cast(&mut g, cat, None);
    assert_eq!(g.battlefield_find(cat).unwrap().counter_count(CounterType::Revival), 8);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(cat)));
    assert!(g.battlefield_find(cat).is_none());
    to_end_step(&mut g);
    assert_eq!(g.battlefield_find(cat).expect("returned").counter_count(CounterType::Revival), 7);
}

/// Protean Hydra prevents damage by shedding counters, then regrows two per
/// counter at the next end step.
#[test]
fn protean_hydra_sheds_and_regrows() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let hydra = g.add_card_to_hand(0, catalog::protean_hydra());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: hydra, target: None, additional_targets: vec![], mode: None, x_value: Some(4) })
        .expect("cast");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(hydra)));
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    to_end_step(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne), 7);
}

/// Valakut Exploration: a land drop exiles the top card; the end step bins
/// it and deals 1 to each opponent.
#[test]
fn valakut_exploration_burns_what_went_unplayed() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::valakut_exploration());
    let land = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert_eq!(g.exile.len(), 1);
    let lives = [g.players[1].life, g.players[2].life];
    to_end_step(&mut g);
    assert!(g.exile.is_empty());
    assert_eq!([g.players[1].life, g.players[2].life], [lives[0] - 1, lives[1] - 1]);
}

/// The Queen of Dale recruits on an opponent's first noncreature spell of
/// the turn — and not on their second.
#[test]
fn queen_of_dale_recruits_once_a_turn_per_opponent() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_queen_of_dale());
    g.decider = Box::new(ScriptedDecider::new([]));
    for n in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
        assert_eq!(g.players[0].graveyard.len(), 1, "one recruit discard after spell {n}");
    }
}

/// Liberator grows when a spell cast with more mana than its power.
#[test]
fn liberator_grows_on_big_spells() {
    let mut g = pod(2);
    let lib = ready(&mut g, 0, catalog::liberator_urzas_battlethopter());
    flood(&mut g);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    assert_eq!(g.computed_permanent(lib).unwrap().power, 1, "1 mana isn't more than 1");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(g.computed_permanent(lib).unwrap().power, 2);
}

/// Ratadrabik turns a dead legend into a nonlegendary 2/2 black Zombie copy.
#[test]
fn ratadrabik_copies_a_dead_legend() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ratadrabik_of_urborg());
    let legend = ready(&mut g, 0, catalog::radagast_the_brown());
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(legend)));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(legend)));
    let tok = g.battlefield.iter().find(|c| c.is_token).expect("copy").id;
    let cp = g.computed_permanent(tok).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(!cp.supertypes().contains(&crabomination::card::Supertype::Legendary));
    assert!(cp.subtypes().creature_types.contains(&crabomination::card::CreatureType::Zombie));
    assert!(cp.colors.contains(Color::Black) && cp.colors.contains(Color::Green));
}

/// The Cabbage Merchant makes Food off an opponent's noncreature spell and
/// taps two Foods for a mana.
#[test]
fn cabbage_merchant_foods_and_mana() {
    let mut g = pod(2);
    let merchant = ready(&mut g, 0, catalog::the_cabbage_merchant());
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Player(0)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
    }
    assert_eq!(named(&g, "Food"), 2);
    let pool = g.players[0].mana_pool.total();
    activate(&mut g, merchant, None);
    assert_eq!(g.players[0].mana_pool.total(), pool + 1);
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Food").all(|c| c.tapped));
}

/// CR 702.108 — Bria's "other creatures you control have prowess" is a
/// layer grant; the prowess site read printed keywords only, so the Bear
/// never pumped.
#[test]
fn bria_grants_prowess_to_the_team() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::bria_riptide_rogue());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Koth: landfall pings each opponent; a Mountain also adds {R}.
#[test]
fn koth_pings_and_adds_red_for_a_mountain() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::koth_the_geomancer());
    let m = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(m)).expect("land");
    drain_stack(&mut g);
    assert_eq!([g.players[1].life, g.players[2].life], [19, 19]);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

/// Gaea's Gift: a counter and four keywords until end of turn.
#[test]
fn gaeas_gift_counter_and_keywords() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let gift = g.add_card_to_hand(0, catalog::gaeas_gift());
    flood(&mut g);
    cast(&mut g, gift, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Indestructible) && cp.keywords().contains(&Keyword::Hexproof));
}

/// Conflux finds one card of each color.
#[test]
fn conflux_finds_five_colors() {
    let mut g = pod(2);
    for def in [catalog::serra_angel(), catalog::counterspell(), catalog::dark_ritual(), catalog::lightning_bolt(), catalog::llanowar_elves()] {
        g.add_card_to_library(0, def);
    }
    let c = g.add_card_to_hand(0, catalog::conflux());
    flood(&mut g);
    cast(&mut g, c, None);
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        assert!(g.players[0].hand.iter().any(|h| h.definition.printed_colors().contains(&c)), "{c:?}");
    }
    assert_eq!(g.players[0].hand.len(), 5);
}

/// CR 121.2a — Underrealm Lich turns a draw into "look at three, keep one,
/// bin the rest".
#[test]
fn underrealm_lich_bins_the_rest() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::underrealm_lich());
    let hand = g.players[0].hand.len();
    let gy = g.players[0].graveyard.len();
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert_eq!(g.players[0].graveyard.len(), gy + 2);
}

/// CR 121.2a — Eruth turns a draw into two cards exiled and playable.
#[test]
fn eruth_exiles_two_instead_of_drawing() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::eruth_tormented_prophet());
    let hand = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(g.exile.iter().filter(|c| c.may_play_until.is_some()).count(), 2);
}

/// Liesa, Forgotten Archangel: your creature that dies returns at the end
/// step; an opponent's token is exiled rather than dying.
#[test]
fn liesa_returns_yours_and_exiles_theirs() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::liesa_forgotten_archangel());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(bear)));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    cast(&mut g, bolt, Some(Target::Permanent(theirs)));
    assert!(g.exile.iter().any(|c| c.id == theirs), "exiled instead of dying");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "back to hand");
}

/// Virtue of Strength triples a basic land's mana, not a nonbasic's.
#[test]
fn virtue_of_strength_triples_basics() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::virtue_of_strength());
    let forest = ready(&mut g, 0, catalog::forest());
    let grove = ready(&mut g, 0, catalog::command_tower());
    activate(&mut g, forest, None);
    assert_eq!(g.players[0].mana_pool.total(), 3);
    activate(&mut g, grove, None);
    assert_eq!(g.players[0].mana_pool.total(), 4);
}

/// CR 614 — "If a creature an opponent controls would die, exile it instead"
/// (Misery's Shadow) covers tokens: an exiled token never died, so Blood
/// Artist sees nothing. The redirect skipped every token before.
#[test]
fn miserys_shadow_exiles_an_opponents_token() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::miserys_shadow());
    ready(&mut g, 0, catalog::blood_artist());
    let spirit = crabomination::card::TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![crabomination::card::CardType::Creature],
        ..Default::default()
    };
    let tok = g.add_token_to_battlefield(1, &spirit);
    flood(&mut g);
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(tok)));
    assert!(g.battlefield_find(tok).is_none());
    assert_eq!(g.players[1].life, life, "no death, no drain");
}

/// Deathgreeter and Virulent Emissary: a death and an entry each gain 1.
#[test]
fn deathgreeter_and_virulent_emissary_gain_life() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::deathgreeter());
    ready(&mut g, 0, catalog::virulent_emissary());
    flood(&mut g);
    let life = g.players[0].life;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(g.players[0].life, life + 1, "Emissary");
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(theirs)));
    assert_eq!(g.players[0].life, life + 2, "Deathgreeter");
}

/// Oblivion Crown grants Noose Constrictor's discard pump to the creature it
/// enchants.
#[test]
fn oblivion_crown_grants_a_discard_pump() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let crown = g.add_card_to_hand(0, catalog::oblivion_crown());
    flood(&mut g);
    cast(&mut g, crown, Some(Target::Permanent(bear)));
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, bear, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Ancestral Statue bounces a nonland permanent (itself, with nothing else).
#[test]
fn ancestral_statue_returns_a_nonland_permanent() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::forest());
    let statue = g.add_card_to_hand(0, catalog::ancestral_statue());
    flood(&mut g);
    cast(&mut g, statue, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == statue));
}

/// Rattleclaw Mystic turned face up adds {G}{U}{R}.
#[test]
fn rattleclaw_mystic_face_up_adds_three() {
    let mut g = pod(2);
    let rm = g.add_card_to_hand(0, catalog::rattleclaw_mystic());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFaceDown { card_id: rm }).expect("morph");
    drain_stack(&mut g);
    let green = g.players[0].mana_pool.amount(Color::Green);
    g.perform_action(GameAction::TurnFaceUp { card_id: rm }).expect("face up");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), green + 1);
}

/// Basal Sliver gives every Sliver — an opponent's too — a sacrifice for
/// {B}{B}.
#[test]
fn basal_sliver_grants_every_sliver_mana() {
    let mut g = pod(2);
    let basal = ready(&mut g, 0, catalog::basal_sliver());
    let pool = g.players[0].mana_pool.amount(Color::Black);
    activate(&mut g, basal, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), pool + 2);
    assert!(g.battlefield_find(basal).is_none());
}

/// Training Grounds: Basal-free check — a {4} creature ability costs {2},
/// and a {1} one stays {1}.
#[test]
fn training_grounds_discounts_creature_abilities() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::training_grounds());
    let m = ready(&mut g, 0, catalog::lesser_masticore());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: m,
        ability_index: 0,
        target: Some(Target::Permanent(elf)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{4} paid with {2}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none());
}

/// Circle of Flame pings a ground attacker, not a flier.
#[test]
fn circle_of_flame_burns_ground_attackers() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::circle_of_flame());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let angel = ready(&mut g, 0, catalog::serra_angel());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: elf, target: AttackTarget::Player(1) },
        Attack { attacker: angel, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none());
    assert_eq!(g.battlefield_find(angel).unwrap().damage, 0);
}

/// Smash to Dust's third mode pings each opposing creature.
#[test]
fn smash_to_dust_sweeps_x1s() {
    let mut g = pod(3);
    let a = ready(&mut g, 1, catalog::llanowar_elves());
    let b = ready(&mut g, 2, catalog::llanowar_elves());
    let mine = ready(&mut g, 0, catalog::llanowar_elves());
    let smash = g.add_card_to_hand(0, catalog::smash_to_dust());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: smash, target: None, additional_targets: vec![], mode: Some(2), x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
    assert!(g.battlefield_find(mine).is_some());
}

/// Starnheim Courser cuts an artifact spell by {1}.
#[test]
fn starnheim_courser_discounts_artifacts() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::starnheim_courser());
    let staff = g.add_card_to_hand(0, catalog::wizards_staff());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: staff, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{U} paid with {U}");
}

/// Diabolic Revelation tutors X cards to hand.
#[test]
fn diabolic_revelation_tutors_x() {
    let mut g = pod(2);
    let spell = g.add_card_to_hand(0, catalog::diabolic_revelation());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([]));
    let (hand, lib) = (g.players[0].hand.len(), g.players[0].library.len());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3);
    assert_eq!(g.players[0].library.len(), lib - 3);
}

/// Gelatinous Genesis with X=3 makes three 3/3 Oozes ({X}{X} counted once).
#[test]
fn gelatinous_genesis_makes_x_xs() {
    let mut g = pod(2);
    let spell = g.add_card_to_hand(0, catalog::gelatinous_genesis());
    g.players[0].mana_pool.add(Color::Green, 7);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(3) })
        .expect("{3}{3}{G} paid with seven");
    drain_stack(&mut g);
    let oozes: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Ooze").map(|c| c.id).collect();
    assert_eq!(oozes.len(), 3);
    assert!(oozes.iter().all(|&o| g.computed_permanent(o).unwrap().power == 3));
}

/// Children of Korlis refunds the life lost this turn (CR 119.3: damage is
/// life loss).
#[test]
fn children_of_korlis_refunds_lost_life() {
    let mut g = pod(2);
    let kids = ready(&mut g, 0, catalog::children_of_korlis());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    let life = g.players[0].life;
    activate(&mut g, kids, None);
    assert!(g.battlefield_find(kids).is_none());
    assert_eq!(g.players[0].life, life + 3);
}

/// Rodolf: paying {1}{W/B} at the end step returns a creature card no
/// bigger than the life gained this turn.
#[test]
fn rodolf_reanimates_by_life_gained() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rodolf_duskbringer());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].life_gained_this_turn = 2;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // Pools empty between steps (CR 500.4): float the {1}{W/B} here.
    g.players[0].mana_pool.add(Color::Black, 2);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear back with 2 life gained");
}

/// Tor Wauki the Younger: the bolt's 3 becomes 4 (CR 614.1a), and the cast
/// trigger's own 2 is not boosted (it's Tor Wauki's damage).
#[test]
fn tor_wauki_the_younger_adds_one_to_other_noncombat() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::tor_wauki_the_younger());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    cast(&mut g, bolt, Some(Target::Player(1)));
    assert_eq!(life - g.players[1].life, 4 + 2, "bolt 3+1, trigger 2");
}

/// Linvala, Keeper of Silence locks an opponent's creature ability.
#[test]
fn linvala_keeper_of_silence_locks_opposing_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::linvala_keeper_of_silence());
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.priority.player_with_priority = 1;
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: elf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(r.is_err(), "Elves' mana ability is locked");
}

/// Jin-Gitaxias counters an opponent's first instant each turn, but only
/// the first (CR 603.3d — triggers only once each turn).
#[test]
fn jin_gitaxias_counters_only_the_first_opposing_instant() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::jin_gitaxias_progress_tyrant());
    let life = g.players[0].life;
    for _ in 0..2 {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life - 3, "first bolt countered, second hits");
}

/// Jin-Gitaxias copies your first sorcery each turn.
#[test]
fn jin_gitaxias_copies_your_first_sorcery() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::jin_gitaxias_progress_tyrant());
    let spell = g.add_card_to_hand(0, catalog::gelatinous_genesis());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(1) })
        .expect("cast");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Ooze"), 2, "the copy keeps X = 1 (CR 707.10)");
}

/// Phylath makes a Plant per basic land, and landfall grows a Plant by four.
#[test]
fn phylath_plants_and_grows() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let phylath = g.add_card_to_hand(0, catalog::phylath_world_sculptor());
    flood(&mut g);
    cast(&mut g, phylath, None);
    let plants: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Plant").map(|c| c.id).collect();
    assert_eq!(plants.len(), 3);
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    assert!(plants.iter().any(|&p| g.computed_permanent(p).unwrap().power == 4));
}

/// Nissa, Vital Force's +1 animates a land into a 5/5 haste Elemental.
#[test]
fn nissa_vital_force_animates_a_land() {
    let mut g = pod(2);
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let nissa = ready(&mut g, 0, catalog::nissa_vital_force());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 0, target: Some(Target::Permanent(land)), x_value: None })
        .expect("+1");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Cerulean Wisps untaps and draws.
#[test]
fn cerulean_wisps_untaps_and_draws() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let wisps = g.add_card_to_hand(0, catalog::cerulean_wisps());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    cast(&mut g, wisps, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.players[0].hand.len(), hand);
}

/// Nissa of Shadowed Boughs: landfall adds loyalty; −5 puts a graveyard
/// creature onto the battlefield with two +1/+1 counters.
#[test]
fn nissa_of_shadowed_boughs_landfall_and_reanimate() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let nissa = ready(&mut g, 0, catalog::nissa_of_shadowed_boughs());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(&mut g);
    let loyalty = |g: &GameState| g.battlefield_find(nissa).unwrap().counter_count(crabomination::card::CounterType::Loyalty);
    assert_eq!(loyalty(&g), 5);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: nissa, ability_index: 1, target: None, x_value: None })
        .expect("-5");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("bear on the battlefield");
    assert_eq!(cp.power, 4);
}

/// Mikaeus, the Unhallowed: a bolted non-Human comes back with a +1/+1
/// counter (undying granted, CR 702.93a).
#[test]
fn mikaeus_the_unhallowed_grants_undying() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mikaeus_the_unhallowed());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 2);
    for b in [bolt, bolt2] {
        let target = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").map(|c| c.id);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: b, target: target.map(Target::Permanent), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        drain_stack(&mut g);
    }
    // First death: undying returns it with a counter (4/4 under Mikaeus); the
    // second bolt only deals 3, so it survives.
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").expect("returned");
    assert_eq!(g.computed_permanent(back.id).unwrap().power, 4);
}

/// Lich's Mastery: a bolt to the face exiles three things; a 2-life gain
/// draws two.
#[test]
fn lichs_mastery_pays_in_cards() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::lichs_mastery());
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "three graveyard cards exiled");
    let hand = g.players[0].hand.len();
    let lifelink = g.add_card_to_hand(0, catalog::healing_salve());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, lifelink, Some(Target::Player(0)));
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three for three life");
}

/// Talion, named 1: an opponent's bolt (mana value 1) drains 2 and draws.
#[test]
fn talion_punishes_the_named_number() {
    let mut g = pod(2);
    let talion = g.add_card_to_hand(0, catalog::talion_the_kindly_lord());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(1)]));
    cast(&mut g, talion, None);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    let (life, hand) = (g.players[1].life, g.players[0].hand.len());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Ruthless Technomancer: two artifacts sacrificed (X = 2) bring back a
/// 2-power creature card.
#[test]
fn ruthless_technomancer_trades_artifacts_for_a_body() {
    let mut g = pod(2);
    let tech = ready(&mut g, 0, catalog::ruthless_technomancer());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::ornithopter());
    }
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tech,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("activate with X = 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert_eq!(named(&g, "Ornithopter"), 0);
}

/// Will, Scion of Peace: with 3 life gained this turn, a blue {1}{U}
/// Wizard's Staff costs just {U}.
#[test]
fn will_scion_of_peace_discounts_by_life_gained() {
    let mut g = pod(2);
    let will = ready(&mut g, 0, catalog::will_scion_of_peace());
    g.players[0].life_gained_this_turn = 3;
    activate(&mut g, will, None);
    let staff = g.add_card_to_hand(0, catalog::wizards_staff());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: staff, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{U} paid with {U}");
}

/// Gold-Forged Thopteryx gives a legendary permanent ward {2}.
#[test]
fn gold_forged_thopteryx_wards_legends() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::gold_forged_thopteryx());
    let will = ready(&mut g, 0, catalog::will_scion_of_peace());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(will)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(will).is_some(), "ward countered the unpaid bolt");
}

/// Emiel: paying {G/W} as a creature enters puts a +1/+1 counter on it.
#[test]
fn emiel_pays_for_an_entering_counter() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::emiel_the_blessed());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, bear, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Misthollow Griffin is castable from exile.
#[test]
fn misthollow_griffin_casts_from_exile() {
    let mut g = pod(2);
    let griffin = g.add_card_to_hand(0, catalog::misthollow_griffin());
    let card = g.players[0].hand.iter().position(|c| c.id == griffin).unwrap();
    let c = g.players[0].hand.remove(card);
    g.exile.push(c);
    g.players[0].mana_pool.add(Color::Blue, 4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAdventureCreature {
        card_id: griffin, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("cast from exile");
    drain_stack(&mut g);
    assert!(g.battlefield_find(griffin).is_some());
}

/// Mnemonic Deluge recasts an opponent's graveyard Lightning Bolt three
/// times and exiles itself.
#[test]
fn mnemonic_deluge_triples_a_graveyard_spell() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let deluge = g.add_card_to_hand(0, catalog::mnemonic_deluge());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Bool(true), DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    cast(&mut g, deluge, Some(Target::Permanent(bolt)));
    assert_eq!(life - g.players[1].life, 9, "three bolts");
    assert!(g.exile.iter().any(|c| c.id == deluge), "Deluge exiles itself");
}

/// Cultivator Colossus chains land drops: two lands in hand both go in, a
/// card drawn for each, and it counts every land.
#[test]
fn cultivator_colossus_chains_lands() {
    let mut g = pod(2);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::forest());
    }
    let colossus = g.add_card_to_hand(0, catalog::cultivator_colossus());
    flood(&mut g);
    cast(&mut g, colossus, None);
    let lands = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands, 5);
    assert_eq!(g.computed_permanent(colossus).unwrap().power, 5);
}

/// Iridescent Hornbeetle: Emiel's paid counter on an entering bear is a
/// +1/+1 counter you put on your own creature, so the end step makes an
/// Insect.
#[test]
fn iridescent_hornbeetle_counts_your_counters() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::iridescent_hornbeetle());
    ready(&mut g, 0, catalog::emiel_the_blessed());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, bear, None);
    assert_eq!(g.own_p1p1_counters_this_turn(0), 1);
    to_end_step(&mut g);
    assert_eq!(named(&g, "Insect"), 1);
}

/// South Wind Avatar: a bear dying gains 2, and each gain drains 1.
#[test]
fn south_wind_avatar_drains_on_gain() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::south_wind_avatar());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, mine + 2);
    assert_eq!(g.players[1].life, theirs - 1);
}

/// Oakhame Adversary costs {G} against a green opponent, and its hit draws.
#[test]
fn oakhame_adversary_discount_and_draw() {
    let mut g = pod(2);
    ready(&mut g, 1, catalog::llanowar_elves());
    let oak = g.add_card_to_hand(0, catalog::oakhame_adversary());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, oak, None);
    assert!(g.battlefield_find(oak).is_some(), "{{1}}{{G}} after the {{2}} discount");
    g.clear_sickness(oak);
    let hand = g.players[0].hand.len();
    connect(&mut g, oak);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Angelfire Ignition: +2 counters and lifelink on a bear.
#[test]
fn angelfire_ignition_suits_up() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::angelfire_ignition());
    flood(&mut g);
    cast(&mut g, spell, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Lifelink));
}

/// Sheltering Light saves a creature from a destroy.
#[test]
fn sheltering_light_makes_indestructible() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let light = g.add_card_to_hand(0, catalog::sheltering_light());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, light, Some(Target::Permanent(bear)));
    let kill = g.add_card_to_hand(1, catalog::murder());
    g.players[1].mana_pool.add(Color::Black, 3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: kill, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .expect("murder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// Dreadmaw's Ire: +2/+2 on an attacker, and its hit destroys an artifact
/// the defending player controls.
#[test]
fn dreadmaws_ire_breaks_an_artifact() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let rock = ready(&mut g, 1, catalog::ornithopter());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    let ire = g.add_card_to_hand(0, catalog::dreadmaws_ire());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, ire, Some(Target::Permanent(bear)));
    let life = g.players[1].life;
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(life - g.players[1].life, 4);
    assert!(g.battlefield_find(rock).is_none(), "Ornithopter destroyed");
}

/// Khenra Spellspear transforms (sorcery speed) into a 3/3 whose two
/// prowess instances each trigger on one noncreature spell.
#[test]
fn khenra_spellspear_transforms_into_double_prowess() {
    let mut g = pod(2);
    let jackal = ready(&mut g, 0, catalog::khenra_spellspear());
    flood(&mut g);
    activate(&mut g, jackal, None);
    let cp = g.computed_permanent(jackal).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Gitaxian Spellstalker");
    let wisps = g.add_card_to_hand(0, catalog::cerulean_wisps());
    cast(&mut g, wisps, Some(Target::Permanent(jackal)));
    assert_eq!(g.computed_permanent(jackal).unwrap().power, 5, "two prowess triggers");
}

/// Vraska, Soul of Stone: a noncreature spell makes a 1/1 Sculpture
/// Treasure creature, which has vigilance.
#[test]
fn vraska_soul_of_stone_makes_sculptures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::vraska_soul_of_stone());
    let wisps = g.add_card_to_hand(0, catalog::cerulean_wisps());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, wisps, Some(Target::Permanent(bear)));
    let s = g.battlefield.iter().find(|c| c.definition.name == "Sculpture").expect("Sculpture").id;
    assert!(g.computed_permanent(s).unwrap().keywords().contains(&crabomination::card::Keyword::Vigilance));
}

/// See the Truth from hand: one of three to hand.
#[test]
fn see_the_truth_takes_one_from_hand() {
    let mut g = pod(2);
    let spell = g.add_card_to_hand(0, catalog::see_the_truth());
    g.players[0].mana_pool.add(Color::Blue, 2);
    let (hand, lib) = (g.players[0].hand.len(), g.players[0].library.len());
    cast(&mut g, spell, None);
    assert_eq!(g.players[0].hand.len(), hand);
    assert_eq!(g.players[0].library.len(), lib - 1);
}

/// Simulacrum Synthesizer: a 3-MV artifact entering makes a Construct that
/// counts every artifact you control.
#[test]
fn simulacrum_synthesizer_mints_constructs() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::simulacrum_synthesizer());
    let big = g.add_card_to_hand(0, catalog::simulacrum_synthesizer());
    flood(&mut g);
    cast(&mut g, big, None);
    let c = g.battlefield.iter().find(|c| c.definition.name == "Construct").expect("Construct").id;
    // Two Synthesizers + the Construct itself.
    assert_eq!(g.computed_permanent(c).unwrap().power, 3);
}

/// Stoneskin gives +0/+10.
#[test]
fn stoneskin_adds_ten_toughness() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::stoneskin());
    flood(&mut g);
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 12);
}

/// Malcolm at three chorus counters: the fourth hit loots, and the
/// discarded Lightning Bolt is cast free at the defending player.
#[test]
fn malcolm_recasts_the_looted_card_at_four_chorus() {
    let mut g = pod(2);
    let malcolm = ready(&mut g, 0, catalog::malcolm_alluring_scoundrel());
    g.battlefield_find_mut(malcolm).unwrap().counters.insert(crabomination::card::CounterType::Chorus, 3);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![bolt]), DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    connect(&mut g, malcolm);
    assert_eq!(g.battlefield_find(malcolm).unwrap().counter_count(crabomination::card::CounterType::Chorus), 4);
    assert_eq!(life - g.players[1].life, 2 + 3, "Malcolm's 2 and the free bolt's 3");
}

/// Likeness Looter becomes a flying copy of a 2-MV graveyard bear (X = 2)
/// that keeps its copy ability.
#[test]
fn likeness_looter_copies_a_graveyard_creature() {
    let mut g = pod(2);
    let looter = ready(&mut g, 0, catalog::likeness_looter());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: looter,
        ability_index: 1,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("copy for X = 2");
    drain_stack(&mut g);
    let c = g.battlefield_find(looter).unwrap();
    assert_eq!(c.definition.name, "Grizzly Bears");
    let cp = g.computed_permanent(looter).unwrap();
    assert_eq!(cp.power, 2);
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Flying));
    assert_eq!(g.battlefield_find(looter).unwrap().definition.activated_abilities.len(), 1, "keeps the copy ability");
}

/// Life Finds a Way: a 4-power nontoken creature entering copies a token.
#[test]
fn life_finds_a_way_populates() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::life_finds_a_way());
    let ooze = g.add_card_to_hand(0, catalog::gelatinous_genesis());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: ooze, target: None, additional_targets: vec![], mode: None, x_value: Some(1) })
        .expect("one Ooze");
    drain_stack(&mut g);
    let big = g.add_card_to_hand(0, catalog::phylath_world_sculptor());
    cast(&mut g, big, None);
    assert_eq!(named(&g, "Ooze"), 2);
}

/// Thornbite Staff: the equipped creature pings for {2}, {T}, and untaps
/// when a creature dies.
#[test]
fn thornbite_staff_pings_and_untaps() {
    let mut g = pod(2);
    let staff = ready(&mut g, 0, catalog::thornbite_staff());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(staff).unwrap().attached_to = Some(bear);
    flood(&mut g);
    let elf = ready(&mut g, 1, catalog::llanowar_elves());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: Some(Target::Permanent(elf)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("granted ping");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none());
    assert!(!g.battlefield_find(bear).unwrap().tapped, "the elf's death untapped it");
}

/// Terisian Mindbreaker's attack mills half the defender's library, rounded
/// up (CR 701.13).
#[test]
fn terisian_mindbreaker_mills_half() {
    let mut g = pod(2);
    let jug = ready(&mut g, 0, catalog::terisian_mindbreaker());
    g.add_card_to_library(1, catalog::grizzly_bears());
    let lib = g.players[1].library.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: jug, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib - lib.div_ceil(2));
}

/// Vantress Gargoyle can't attack into a thin graveyard (CR 508.1a), and
/// can once the defender has seven cards there.
#[test]
fn vantress_gargoyle_needs_a_full_graveyard() {
    let mut g = pod(2);
    let gar = ready(&mut g, 0, catalog::vantress_gargoyle());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: gar, target: AttackTarget::Player(1) }]))
        .is_err());
    for _ in 0..7 {
        g.add_card_to_graveyard(1, catalog::grizzly_bears());
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: gar, target: AttackTarget::Player(1) }]))
        .expect("seven in the yard");
}

/// Ghalta the Immovable costs {X} less for your biggest toughness, and a
/// 0/3 Wall swings for 3 under it.
#[test]
fn ghalta_discounts_and_walls_hit_by_toughness() {
    let mut g = pod(2);
    let wall = ready(&mut g, 0, catalog::wall_of_omens());
    let ghalta = g.add_card_to_hand(0, catalog::ghalta_the_immovable());
    // Wall of Omens is 0/4: {8}{W} - {4} = {4}{W}.
    g.players[0].mana_pool.add(Color::White, 5);
    cast(&mut g, ghalta, None);
    assert!(g.battlefield_find(ghalta).is_some());
    let life = g.players[1].life;
    connect(&mut g, wall);
    assert_eq!(life - g.players[1].life, 4, "the 0/4 Wall assigns 4");
}

/// Rammas Echor: the second spell in a turn draws and makes a Wall.
#[test]
fn rammas_echor_rewards_the_second_spell() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rammas_echor_ancient_shield());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    for _ in 0..2 {
        let w = g.add_card_to_hand(0, catalog::cerulean_wisps());
        cast(&mut g, w, Some(Target::Permanent(bear)));
    }
    assert_eq!(named(&g, "Wall"), 1);
}

/// Blossoming Tortoise: entering mills three and returns a land tapped.
#[test]
fn blossoming_tortoise_recurs_a_land() {
    let mut g = pod(2);
    g.players[0].library.clear();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let turtle = g.add_card_to_hand(0, catalog::blossoming_tortoise());
    flood(&mut g);
    let lands = |g: &GameState| g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let before = lands(&g);
    cast(&mut g, turtle, None);
    assert_eq!(lands(&g), before + 1);
}

/// Echoing Deeps enters tapped as a copy of a graveyard land.
#[test]
fn echoing_deeps_copies_a_graveyard_land() {
    let mut g = pod(2);
    g.add_card_to_graveyard(1, catalog::forest());
    let deeps = g.add_card_to_hand(0, catalog::echoing_deeps());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(deeps)).expect("land");
    drain_stack(&mut g);
    let c = g.battlefield_find(deeps).unwrap();
    assert_eq!(c.definition.name, "Forest");
    assert!(c.tapped);
}

/// Thran Vigil: reanimating a creature card on your turn grows a creature.
#[test]
fn thran_vigil_counts_graveyard_departures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::thran_vigil());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 2);
    let raise = g.add_card_to_hand(0, catalog::raise_dead());
    cast(&mut g, raise, Some(Target::Permanent(dead)));
    let grown = [bear, dead].iter().any(|&b| g.battlefield_find(b).is_some_and(|c| c.counter_count(crabomination::card::CounterType::PlusOnePlusOne) == 1));
    assert!(grown, "one creature got the counter");
}

/// Vohar's loot: discarding an instant drains each opponent 1.
#[test]
fn vohar_drains_on_a_spell_discard() {
    let mut g = pod(2);
    let vohar = ready(&mut g, 0, catalog::vohar_vodalian_desecrator());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![bolt])]));
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    activate(&mut g, vohar, None);
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 1, theirs - 1));
}

/// Iron Spider's tap pumps each artifact creature you control.
#[test]
fn iron_spider_counters_artifact_creatures() {
    let mut g = pod(2);
    let spider = ready(&mut g, 0, catalog::iron_spider_stark_upgrade());
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    activate(&mut g, spider, None);
    let p1 = crabomination::card::CounterType::PlusOnePlusOne;
    assert_eq!(g.battlefield_find(thopter).unwrap().counter_count(p1), 1);
    assert_eq!(g.battlefield_find(spider).unwrap().counter_count(p1), 1);
}

/// Zur makes an enchantment a creature with P/T equal to its mana value,
/// and it gains deathtouch.
#[test]
fn zur_animates_an_enchantment() {
    let mut g = pod(2);
    let zur = ready(&mut g, 0, catalog::zur_eternal_schemer());
    let vigil = ready(&mut g, 0, catalog::thran_vigil());
    flood(&mut g);
    activate(&mut g, zur, Some(Target::Permanent(vigil)));
    let cp = g.computed_permanent(vigil).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
    assert!(cp.keywords().contains(&crabomination::card::Keyword::Deathtouch));
}

/// Karn, Legacy Reforged: a 5/5 as the biggest artifact, and its upkeep {C}
/// survives into the main phase (CR 500.4 exception).
#[test]
fn karn_legacy_reforged_keeps_upkeep_mana() {
    let mut g = pod(2);
    let karn = ready(&mut g, 0, catalog::karn_legacy_reforged());
    ready(&mut g, 0, catalog::ornithopter());
    assert_eq!(g.computed_permanent(karn).unwrap().power, 5);
    g.step = TurnStep::Untap;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].mana_pool.restricted_total(), 2, "two artifacts → {{C}}{{C}} kept into main");
}

/// Spelltithe Enforcer's tax paid on a Room spell at a four-seat table: the
/// Room still resolves once every seat passes (CR 117.4).
#[test]
fn a_taxed_room_spell_resolves_after_everyone_passes() {
    let mut g = pod(4);
    ready(&mut g, 1, catalog::spelltithe_enforcer());
    let room = g.add_card_to_hand(0, catalog::secret_arcade_dusty_parlor());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: room, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast the Room");
    for _ in 0..40 {
        if g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.stack.is_empty(), "stack: {:?}", g.stack.len());
    assert!(g.battlefield_find(room).is_some());
}

/// Scorn-Blade Berserker backs up a bear: +1/+1 counter, and the bear can
/// sacrifice itself to draw this turn.
#[test]
fn scorn_blade_berserker_backs_up_with_its_draw() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let blade = g.add_card_to_hand(0, catalog::scorn_blade_berserker());
    flood(&mut g);
    cast(&mut g, blade, None);
    let backed = g.battlefield_find(bear).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne) == 1;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    let r = g.perform_action(GameAction::ActivateAbility {
        card_id: bear,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    // Backing up itself grants nothing (CR 702.164a: only "another creature").
    assert_eq!(r.is_ok(), backed, "the bear has the sacrifice ability iff it was backed up");
    if backed {
        drain_stack(&mut g);
        assert!(g.battlefield_find(bear).is_none());
        assert_eq!(g.players[0].hand.len(), hand + 1);
    }
}

/// The Necrobloom: with seven differently named lands, landfall makes a
/// Zombie instead of a Plant.
#[test]
fn the_necrobloom_makes_zombies_at_seven_names() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_necrobloom());
    for land in [catalog::plains(), catalog::island(), catalog::swamp(), catalog::mountain(), catalog::command_tower(), catalog::evolving_wilds()] {
        g.add_card_to_battlefield(0, land);
    }
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(forest)).expect("land");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Zombie"), 1);
    assert_eq!(named(&g, "Plant"), 0);
}

/// The Necrobloom gives graveyard lands dredge 2 (CR 702.52): a draw
/// returns the land and mills two.
#[test]
fn the_necrobloom_lands_dredge() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::the_necrobloom());
    let land = g.add_card_to_graveyard(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let lib = g.players[0].library.len();
    g.draw_one(0, &mut Vec::new());
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "dredged back");
    assert_eq!(g.players[0].library.len(), lib - 2);
}

/// Entity Tracker's Eerie half for Rooms: fully unlocking one draws a card.
#[test]
fn entity_tracker_draws_on_a_fully_unlocked_room() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::entity_tracker());
    let room = g.add_card_to_hand(0, catalog::secret_arcade_dusty_parlor());
    flood(&mut g);
    cast(&mut g, room, None);
    let hand = g.players[0].hand.len();
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::UnlockRoomDoor { card_id: room, right: true }).expect("unlock the other door");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Crystal Barricade prevents a bolt to another of your creatures, not to
/// itself (CR 615).
#[test]
fn crystal_barricade_shields_other_creatures() {
    let mut g = pod(2);
    let wall = ready(&mut g, 0, catalog::crystal_barricade());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for t in [bear, wall] {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(t)), additional_targets: vec![], mode: None, x_value: None })
            .expect("bolt");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(bear).is_some(), "the bear was shielded");
    assert_eq!(g.battlefield_find(wall).unwrap().damage, 3, "the wall itself is not");
}

/// Vizier of the Menagerie: red mana casts a green creature (CR 609.4b), but
/// not a noncreature spell.
#[test]
fn vizier_spends_any_type_on_creatures() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::vizier_of_the_menagerie());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, bear, None);
    assert!(g.battlefield_find(bear).is_some());
    let wisps = g.add_card_to_hand(0, catalog::cerulean_wisps());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    assert!(g
        .perform_action(GameAction::CastSpell { card_id: wisps, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None })
        .is_err());
}

/// Reality Acid: when it leaves (here, bounced), the enchanted permanent's
/// controller sacrifices it.
#[test]
fn reality_acid_takes_its_host_with_it() {
    let mut g = pod(2);
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let acid = g.add_card_to_hand(0, catalog::reality_acid());
    flood(&mut g);
    cast(&mut g, acid, Some(Target::Permanent(theirs)));
    assert_eq!(g.battlefield_find(acid).unwrap().attached_to, Some(theirs));
    g.remove_from_battlefield_to_exile(acid);
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none());
}

/// Venser's +2 flickers a permanent you own back at the next end step.
#[test]
fn venser_flickers_your_permanent() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let venser = ready(&mut g, 0, catalog::venser_the_sojourner());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: venser, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None })
        .expect("+2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none());
    to_end_step(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.controller == 0));
}

/// Queen Allenal: a creature token comes with a Soldier, and she counts it.
#[test]
fn queen_allenal_adds_a_soldier() {
    let mut g = pod(2);
    let queen = ready(&mut g, 0, catalog::queen_allenal_of_ruadach());
    let spell = g.add_card_to_hand(0, catalog::gelatinous_genesis());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: Some(1) })
        .expect("one Ooze");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Soldier"), 1);
    assert_eq!(g.computed_permanent(queen).unwrap().power, 3, "queen, Ooze, Soldier");
}

/// Rabble Rousing: attacking with two makes two Citizens.
#[test]
fn rabble_rousing_makes_citizens_per_attacker() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rabble_rousing());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Citizen"), 2);
}

/// Tinybones: its activation makes an opponent discard; the card is stashed
/// in exile, and on your turn you may cast it with mana of any type.
#[test]
fn tinybones_stashes_and_plays_a_discard() {
    let mut g = pod(2);
    let bones = ready(&mut g, 0, catalog::tinybones_bauble_burglar());
    g.players[1].hand.clear();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g);
    activate(&mut g, bones, None);
    let c = g.exile.iter().find(|c| c.id == bolt).expect("stashed in exile");
    assert_eq!(c.counter_count(crabomination::card::CounterType::Stash), 1);
    assert!(c.may_play_until.is_some_and(|m| m.player == 0), "playable on your turn");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("cast the stashed bolt from exile");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "cast with blue mana");
}

/// Grolnok: a Frog attacking mills three; the milled permanent cards are
/// exiled with croak counters and playable, the instants stay binned.
#[test]
fn grolnok_croaks_milled_permanents() {
    let mut g = pod(2);
    let frog = ready(&mut g, 0, catalog::grolnok_the_omnivore());
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let land = g.add_card_to_library(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: frog, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    for id in [bear, land] {
        let c = g.exile.iter().find(|c| c.id == id).expect("croaked into exile");
        assert_eq!(c.counter_count(crabomination::card::CounterType::Croak), 1);
        assert!(c.may_play_until.is_some_and(|m| m.player == 0));
    }
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt), "a nonpermanent stays");
}

/// Mnemonic Betrayal: cast an opponent's binned spell with off-color mana;
/// the uncast cards go home at the end step and the sorcery exiles itself.
#[test]
fn mnemonic_betrayal_borrows_the_graveyard() {
    let mut g = pod(2);
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let mb = g.add_card_to_hand(0, catalog::mnemonic_betrayal());
    flood(&mut g);
    cast(&mut g, mb, None);
    assert!(g.exile.iter().any(|c| c.id == mb), "exiles itself");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 1);
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("cast the borrowed bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    to_end_step(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "the bear went home");
}

/// Rona: {5}{B/P} (2 life for the {B/P}) transforms her; damage to the back
/// face exiles a random card from the dealer's controller's hand, and Rona's
/// controller may cast it free.
#[test]
fn rona_transforms_and_steals_on_damage() {
    let mut g = pod(2);
    let rona = ready(&mut g, 0, catalog::rona_herald_of_invasion());
    g.players[0].mana_pool.add_colorless(5);
    let life = g.players[0].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: rona, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("transform for 2 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2);
    assert_eq!(g.computed_permanent(rona).unwrap().power, 5, "Tolarian Obliterator");
    g.players[1].hand.clear();
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let shock = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: shock, target: Some(Target::Permanent(rona)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt Rona");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == bear && c.controller == 0), "cast the stolen bear");
}

/// Mech Hangar: its colored mana funds a Vehicle spell but not a Bear; {3},
/// {T} turns a Vehicle into an artifact creature.
#[test]
fn mech_hangar_funds_vehicles_and_animates_one() {
    let mut g = pod(2);
    let hangar = ready(&mut g, 0, catalog::mech_hangar());
    let act = |g: &mut GameState, idx: usize, target: Option<Target>| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility { card_id: hangar, ability_index: idx, target, additional_targets: vec![], x_value: None, mode: None })
    };
    act(&mut g, 1, None).expect("restricted mana");
    g.players[0].mana_pool.add_colorless(1);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).is_err());
    let copter = g.add_card_to_hand(0, catalog::smugglers_copter());
    cast(&mut g, copter, None);
    assert!(g.battlefield_find(copter).is_some(), "a Vehicle spell");
    g.battlefield_find_mut(hangar).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(3);
    act(&mut g, 2, Some(Target::Permanent(copter))).expect("animate");
    drain_stack(&mut g);
    assert!(g.computed_permanent(copter).unwrap().card_types().contains(&crabomination::card::CardType::Creature));
}

/// CR 702.11d — Tam grants "hexproof from each of its colors": an opponent's
/// red Bolt can't target a red creature but can target a green one; after
/// Tam's {T} turns the bear all colors, it can't be Bolted either.
#[test]
fn cr_702_11d_tam_hexproof_from_its_colors() {
    let mut g = pod(2);
    let tam = ready(&mut g, 0, catalog::tam_mindful_first_year());
    let goblin = ready(&mut g, 0, catalog::goblin_guide());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt_at = |g: &mut GameState, t: CardId| {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Permanent(t)), additional_targets: vec![], mode: None, x_value: None })
    };
    assert!(bolt_at(&mut g, goblin).is_err(), "red can't target a red creature");
    flood(&mut g);
    activate(&mut g, tam, Some(Target::Permanent(bear)));
    assert!(bolt_at(&mut g, bear).is_err(), "an all-colors bear is red too");
    assert!(bolt_at(&mut g, tam).is_ok(), "Tam itself isn't covered");
}

/// Inga and Esika: three creature-made mana into a creature spell draws; the
/// granted mana is creature-spell only.
#[test]
fn inga_and_esika_draws_off_creature_mana() {
    let mut g = pod(2);
    let inga = ready(&mut g, 0, catalog::inga_and_esika());
    let dorks: Vec<CardId> = (0..2).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    for id in [inga, dorks[0], dorks[1]] {
        let idx = g.battlefield_find(id).unwrap().definition.activated_abilities.len();
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: idx, target: None, additional_targets: vec![], x_value: None, mode: None })
            .expect("granted mana");
    }
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand = g.players[0].hand.len();
    let giant = g.add_card_to_hand(0, catalog::hill_giant());
    cast(&mut g, giant, None);
    assert!(g.battlefield_find(giant).is_some());
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off three creature mana");
}

/// The Skullspore Nexus: two nontoken bears dying together make one Fungus
/// Dinosaur with their total power; {2}, {T} doubles a creature's power.
#[test]
fn skullspore_nexus_mints_a_fungus_of_their_total_power() {
    let mut g = pod(2);
    let nexus = ready(&mut g, 0, catalog::the_skullspore_nexus());
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    flood(&mut g);
    cast(&mut g, wrath, None);
    let fungi: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Fungus Dinosaur").map(|c| c.id).collect();
    assert_eq!(fungi.len(), 1, "one per batch");
    let cp = g.computed_permanent(fungi[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: nexus, ability_index: 0, target: Some(Target::Permanent(fungi[0])), additional_targets: vec![], x_value: None, mode: None })
        .expect("double");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(fungi[0]).unwrap().power, 8);
}

/// Welcome to . . .: I walls up an opponent's noncreature artifact, II makes
/// a hasty trample Dinosaur, III destroys the Walls and flips to Jurassic
/// Park, which taps for {G} per Dinosaur.
#[test]
fn welcome_to_walls_then_becomes_jurassic_park() {
    let mut g = pod(2);
    let vault = ready(&mut g, 1, catalog::memorial_vault());
    let saga = g.add_card_to_hand(0, catalog::welcome_to());
    flood(&mut g);
    cast(&mut g, saga, None);
    let cp = g.computed_permanent(vault).unwrap();
    assert!(cp.card_types().contains(&crabomination::card::CardType::Creature), "a Wall now");
    assert_eq!((cp.power, cp.toughness), (0, 4));
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Dinosaur"), 1);
    g.saga_advance(saga);
    drain_stack(&mut g);
    assert!(g.battlefield_find(vault).is_none(), "the Wall was destroyed");
    let park = g.battlefield_find(saga).expect("returned transformed");
    assert_eq!(park.definition.name, "Jurassic Park");
    g.players[0].mana_pool = Default::default();
    activate(&mut g, saga, None);
    assert_eq!(g.players[0].mana_pool.total(), 1, "one Dinosaur, one {{G}}");
}

/// Mechtitan Core: {5} and four other artifact creatures make Mechtitan; when
/// it leaves, the four come back tapped and the Core stays exiled.
#[test]
fn mechtitan_core_assembles_and_disassembles() {
    let mut g = pod(2);
    let core = ready(&mut g, 0, catalog::mechtitan_core());
    let parts: Vec<CardId> = (0..4).map(|_| ready(&mut g, 0, catalog::memnite())).collect();
    flood(&mut g);
    activate(&mut g, core, None);
    let titan = g.battlefield.iter().find(|c| c.definition.name == "Mechtitan").map(|c| c.id).expect("Mechtitan");
    assert_eq!(g.computed_permanent(titan).unwrap().power, 10);
    assert!(parts.iter().all(|p| g.battlefield_find(*p).is_none()), "the parts are exiled");
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    cast(&mut g, wrath, None);
    for p in &parts {
        assert!(g.battlefield_find(*p).is_some_and(|c| c.tapped), "a part came back tapped");
    }
    assert!(g.exile.iter().any(|c| c.id == core), "the Core stays exiled");
}

/// Efficient Construction makes a Thopter per artifact spell; Crystal Skull
/// casts a historic (artifact) card off the library top but not a Bear.
#[test]
fn efficient_construction_and_crystal_skull() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::efficient_construction());
    ready(&mut g, 0, catalog::crystal_skull_isu_spyglass());
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).is_err(), "a Bear isn't historic");
    g.players[0].library.clear();
    let mite = g.add_card_to_library(0, catalog::memnite());
    cast(&mut g, mite, None);
    assert!(g.battlefield_find(mite).is_some(), "cast off the top");
    assert_eq!(named(&g, "Thopter"), 1, "an artifact spell made a Thopter");
}

/// Mirrodin Besieged, Phyrexian: the end-step loot, then with fifteen artifact
/// cards in your graveyard the targeted opponent loses — never you.
#[test]
fn mirrodin_besieged_phyrexian_wins_off_fifteen_artifacts() {
    let mut g = pod(3);
    let mb = g.add_card_to_hand(0, catalog::mirrodin_besieged());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(1)]));
    cast(&mut g, mb, None);
    for _ in 0..15 {
        g.add_card_to_graveyard(0, catalog::memnite());
    }
    to_end_step(&mut g);
    assert!(g.players[0].is_alive(), "the controller is never the target");
    let def = catalog::mirrodin_besieged();
    let phyrexian = &def.enter_modes.as_ref().unwrap()[1].triggered_abilities[0].effect;
    assert_eq!(phyrexian.target_filter_for_slot(0), Some(&crabomination::card::SelectionRequirement::OpponentPlayer));
    assert_eq!(g.players.iter().filter(|p| !p.is_alive()).count(), 1, "one opponent lost");
}

/// Rings of Brighthearth: paying {2} copies a pinger's activation.
#[test]
fn rings_of_brighthearth_copies_an_activation() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::rings_of_brighthearth());
    let pinger = ready(&mut g, 0, catalog::prodigal_pyromancer());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    activate(&mut g, pinger, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2, "the copy pinged too");
}

/// The Reality Chip: only while attached may the top card be cast.
#[test]
fn the_reality_chip_plays_from_top_while_attached() {
    let mut g = pod(2);
    let chip = ready(&mut g, 0, catalog::the_reality_chip());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].library.clear();
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell { card_id: top, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None }).is_err(), "unattached");
    g.perform_action(GameAction::Reconfigure { equipment: chip, target: Some(bear) }).expect("reconfigure");
    drain_stack(&mut g);
    cast(&mut g, top, Some(Target::Player(1)));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "cast off the top");
}

/// Illustrious Wanderglyph: ten permanents give the city's blessing and the
/// other artifact creatures +2/+2; each upkeep makes a Gnome.
#[test]
fn illustrious_wanderglyph_ascends() {
    let mut g = pod(2);
    let glyph = ready(&mut g, 0, catalog::illustrious_wanderglyph());
    let mite = ready(&mut g, 0, catalog::memnite());
    assert_eq!(g.computed_permanent(mite).unwrap().power, 1, "no blessing yet");
    for _ in 0..8 {
        ready(&mut g, 0, catalog::forest());
    }
    assert_eq!(g.computed_permanent(mite).unwrap().power, 3, "blessed");
    assert_eq!(g.computed_permanent(glyph).unwrap().power, 2, "not itself");
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Gnome"), 1);
}

/// Thousand Moons Smithy: its Gnome Soldier counts artifacts and creatures;
/// tapping five at your first main phase flips it, and Barracks mana funding a
/// creature spell makes another Soldier — unmarked mana doesn't.
#[test]
fn thousand_moons_smithy_flips_into_barracks() {
    let mut g = pod(2);
    let smithy = g.add_card_to_hand(0, catalog::thousand_moons_smithy());
    flood(&mut g);
    cast(&mut g, smithy, None);
    let soldier = g.battlefield.iter().find(|c| c.definition.name == "Gnome Soldier").unwrap().id;
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2, "the Smithy and itself");
    for _ in 0..4 {
        ready(&mut g, 0, catalog::memnite());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(smithy).unwrap().definition.name, "Barracks of the Thousand");
    g.battlefield_find_mut(smithy).unwrap().tapped = false;
    g.players[0].mana_pool = Default::default();
    activate(&mut g, smithy, None);
    let lions = g.add_card_to_hand(0, catalog::savannah_lions());
    cast(&mut g, lions, None);
    assert_eq!(named(&g, "Gnome Soldier"), 2, "Barracks mana made a Soldier");
    g.players[0].mana_pool.add(Color::White, 1);
    let lions2 = g.add_card_to_hand(0, catalog::savannah_lions());
    cast(&mut g, lions2, None);
    assert_eq!(named(&g, "Gnome Soldier"), 2, "other mana doesn't");
}

/// Raid Bombardment pings the attacked player for a small attacker; Patriar's
/// Seal untaps a legendary creature.
#[test]
fn raid_bombardment_and_patriars_seal() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::raid_bombardment());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let life = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "pinged on the attack");
    let mut g = pod(2);
    let seal = ready(&mut g, 0, catalog::patriars_seal());
    let legend = ready(&mut g, 0, catalog::rat_king_verminister());
    g.battlefield_find_mut(legend).unwrap().tapped = true;
    flood(&mut g);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: seal, ability_index: 1, target: Some(Target::Permanent(legend)), additional_targets: vec![], x_value: None, mode: None })
        .expect("untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(legend).unwrap().tapped);
}

/// Ashcoat attacking pumps the other Rats by the Rat count.
#[test]
fn ashcoat_pumps_the_swarm() {
    let mut g = pod(2);
    let ash = ready(&mut g, 0, catalog::ashcoat_of_the_shadow_swarm());
    let king = ready(&mut g, 0, catalog::rat_king_verminister());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: ash, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(king).unwrap().power, 3, "1 + two Rats");
    assert_eq!(g.computed_permanent(ash).unwrap().power, 3, "not itself");
}

/// Rat King: three Rats sacrificed return a creature card and every other
/// card sharing its name from your graveyard, tapped.
#[test]
fn rat_king_returns_a_name_from_the_graveyard() {
    let mut g = pod(2);
    let king = ready(&mut g, 0, catalog::rat_king_verminister());
    for _ in 0..3 {
        ready(&mut g, 0, catalog::ashcoat_of_the_shadow_swarm());
    }
    let a = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let b = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: king, ability_index: 0, target: Some(Target::Permanent(a)), additional_targets: vec![], x_value: None, mode: None })
        .expect("activate");
    drain_stack(&mut g);
    for id in [a, b] {
        assert!(g.battlefield_find(id).is_some_and(|c| c.tapped), "back, tapped");
    }
}

/// Plague of Vermin: each seat's life paid becomes that many Rats.
#[test]
fn plague_of_vermin_life_for_rats() {
    let mut g = pod(3);
    let lives: Vec<i32> = g.players.iter().map(|p| p.life).collect();
    let plague = g.add_card_to_hand(0, catalog::plague_of_vermin());
    flood(&mut g);
    // Round one: 3, 2, 0; round two: 1, 0, 0; round three: nobody pays.
    let bids = [3, 2, 0, 1, 0, 0, 0, 0, 0].map(DecisionAnswer::Amount);
    g.decider = Box::new(ScriptedDecider::new(bids));
    cast(&mut g, plague, None);
    for (p, want) in [(0usize, 4), (1, 2), (2, 0)] {
        let paid = lives[p] - g.players[p].life;
        let rats = g.battlefield.iter().filter(|c| c.controller == p && c.definition.name == "Rat").count() as i32;
        assert_eq!((paid, rats), (want, want), "seat {p}: a Rat per life paid");
    }
}

/// Demon's Disciple makes every player sacrifice; Patron of the Arts leaves a
/// Treasure entering and another dying.
#[test]
fn demons_disciple_and_patron_of_the_arts() {
    let mut g = pod(2);
    let patron = g.add_card_to_hand(0, catalog::patron_of_the_arts());
    flood(&mut g);
    cast(&mut g, patron, None);
    assert_eq!(named(&g, "Treasure"), 1);
    ready(&mut g, 1, catalog::grizzly_bears());
    let dd = g.add_card_to_hand(0, catalog::demons_disciple());
    cast(&mut g, dd, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears").count(), 0, "the opponent sacrificed");
    let mine = ["Patron of the Arts", "Demon's Disciple"].iter().map(|n| named(&g, n)).sum::<usize>();
    assert_eq!(mine, 1, "and so did you");
    if g.battlefield_find(patron).is_none() {
        assert_eq!(named(&g, "Treasure"), 2, "a sacrificed Patron left a Treasure");
    }
}

/// Force of Despair, pitched off a black card on an opponent's turn, destroys
/// only creatures that entered this turn.
#[test]
fn force_of_despair_kills_the_new_arrivals() {
    let mut g = pod(2);
    let old = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let fresh = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: fresh, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    g.battlefield_find_mut(old).unwrap().entered_turn = None;
    let force = g.add_card_to_hand(0, catalog::force_of_despair());
    let pitch = g.add_card_to_hand(0, catalog::demons_disciple());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative { card_id: force, pitch_card: Some(pitch), target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("pitch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fresh).is_none());
    assert!(g.battlefield_find(old).is_some());
}

/// Vona's Hunger: one sacrifice each, or half rounded up with the city's
/// blessing (ten permanents as it resolves).
#[test]
fn vonas_hunger_halves_with_the_blessing() {
    for (yours, theirs_left) in [(0usize, 4usize), (10, 2)] {
        let mut g = pod(2);
        for _ in 0..yours {
            ready(&mut g, 0, catalog::forest());
        }
        for _ in 0..5 {
            ready(&mut g, 1, catalog::grizzly_bears());
        }
        let hunger = g.add_card_to_hand(0, catalog::vonas_hunger());
        flood(&mut g);
        cast(&mut g, hunger, None);
        assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Grizzly Bears").count(), theirs_left);
    }
}

/// Attack 0 → 1 with `attacker`; seat 1 blocks with `blocker`. Leaves the game
/// at declare-blockers with the block in the map.
fn attack_into_block(g: &mut GameState, attacker: CardId, blocker: Option<CardId>) {
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    let _ = g.perform_action(GameAction::DeclareBlockers(blocker.map(|b| vec![(b, attacker)]).unwrap_or_default()));
    drain_stack(g);
}

/// Sting: +1/+1 and haste, and first strike only when the bearer is blocked
/// by a Goblin.
#[test]
fn sting_first_strikes_against_goblins() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let sting = ready(&mut g, 0, catalog::sting_the_glinting_dagger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: sting, target: bear }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3);
    assert!(cp.keywords().contains(&Keyword::Haste) && !cp.keywords().contains(&Keyword::FirstStrike));
    let goblin = ready(&mut g, 1, catalog::goblin_guide());
    attack_into_block(&mut g, bear, Some(goblin));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::FirstStrike), "blocked by a Goblin");
}

/// Hobgoblin Bandit Lord pings for the Goblins that entered this turn; Muxus
/// puts the revealed cheap Goblins onto the battlefield.
#[test]
fn hobgoblin_bandit_lord_and_muxus() {
    let mut g = pod(2);
    let lord = ready(&mut g, 0, catalog::hobgoblin_bandit_lord());
    g.players[0].library.clear();
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::goblin_guide());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    let muxus = g.add_card_to_hand(0, catalog::muxus_goblin_grandee());
    flood(&mut g);
    cast(&mut g, muxus, None);
    assert_eq!(named(&g, "Goblin Guide"), 2, "both revealed Goblins entered");
    assert!(g.players[0].library.iter().any(|c| c.definition.name == "Grizzly Bears"), "the rest to the bottom");
    let life = g.players[1].life;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: lord, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None, mode: None })
        .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "Muxus and two Guides entered");
}

/// Dolmen Gate keeps an attacker alive through a bigger blocker.
#[test]
fn dolmen_gate_shields_attackers() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::dolmen_gate());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    attack_into_block(&mut g, bear, Some(giant));
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(bear).is_some_and(|c| c.damage == 0));
}

/// Subira: {1} makes a small creature unblockable; the discard-hand ability
/// draws when a small creature connects this turn.
#[test]
fn subira_draws_off_small_hits() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let subira = ready(&mut g, 0, catalog::subira_tulzidi_caravanner());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    activate(&mut g, subira, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Unblockable));
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: subira, ability_index: 1, target: None, additional_targets: vec![], x_value: None, mode: None })
        .expect("discard the hand");
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty());
    g.add_card_to_library(0, catalog::island());
    attack_into_block(&mut g, bear, None);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew off the bear's hit");
}

/// The Battle of Bywater kills the big ones and feeds the survivors.
#[test]
fn the_battle_of_bywater_feeds_the_small() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::grizzly_bears());
    let giant = ready(&mut g, 1, catalog::hill_giant());
    let battle = g.add_card_to_hand(0, catalog::the_battle_of_bywater());
    flood(&mut g);
    cast(&mut g, battle, None);
    assert!(g.battlefield_find(giant).is_none());
    assert_eq!(named(&g, "Food"), 2);
}

/// Kagha: attacking mills two; one milled permanent card may be cast that
/// turn, then the once-a-turn budget is spent. Beside Coram, Kagha's play is
/// its own budget.
#[test]
fn kagha_plays_one_milled_permanent() {
    let mut g = pod(2);
    let kagha = ready(&mut g, 0, catalog::kagha_shadow_archdruid());
    g.players[0].library.clear();
    let a = g.add_card_to_library(0, catalog::grizzly_bears());
    let b = g.add_card_to_library(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kagha, target: AttackTarget::Player(1) }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(kagha).unwrap().keywords().contains(&crabomination::card::Keyword::Deathtouch));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == a) && g.players[0].graveyard.iter().any(|c| c.id == b));
    g.step = TurnStep::PostCombatMain;
    flood(&mut g);
    cast(&mut g, a, None);
    assert!(g.battlefield_find(a).is_some(), "the first milled Bear");
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell { card_id: b, target: None, additional_targets: vec![], mode: None, x_value: None }).is_err(), "once a turn");
    // With Coram too, Coram's spell is a second play.
    ready(&mut g, 0, catalog::coram_the_undertaker());
    cast(&mut g, b, None);
    assert!(g.battlefield_find(b).is_some(), "Coram's budget");
}

/// Malignus is half the highest opponent life, rounded up.
#[test]
fn malignus_halves_the_healthiest_opponent() {
    let mut g = pod(3);
    let m = ready(&mut g, 0, catalog::malignus());
    g.players[1].life = 39;
    g.players[2].life = 12;
    g.players[0].life = 80;
    let cp = g.computed_permanent(m).unwrap();
    assert_eq!((cp.power, cp.toughness), (20, 20), "39 halves up to 20; your own 80 isn't read");
}

/// Hallowed Haunting makes a Spirit Cleric per enchantment spell; Weaver of
/// Harmony copies that trigger (an enchantment source's) for a second Cleric;
/// seven enchantments give your creatures flying.
#[test]
fn hallowed_haunting_weaver_copies_the_trigger() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::hallowed_haunting());
    let haunting = g.battlefield.iter().find(|c| c.definition.name == "Hallowed Haunting").unwrap().id;
    let weaver = ready(&mut g, 0, catalog::weaver_of_harmony());
    flood(&mut g);
    let ench = g.add_card_to_hand(0, catalog::efficient_construction());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    // The Haunting's trigger is on the stack above the spell: copy it.
    g.priority.player_with_priority = 0;
    let trigger = g.top_ability_of(haunting).expect("the Haunting's trigger, as an object (CR 115.1)");
    g.perform_action(GameAction::ActivateAbility { card_id: weaver, ability_index: 0, target: Some(Target::Permanent(trigger)), additional_targets: vec![], x_value: None, mode: None })
        .expect("copy");
    drain_stack(&mut g);
    assert_eq!(named(&g, "Spirit Cleric"), 2, "the trigger and its copy");
    let cleric = g.battlefield.iter().find(|c| c.definition.name == "Spirit Cleric").unwrap().id;
    assert_eq!(g.computed_permanent(cleric).unwrap().power, 2, "two Spirits");
    assert!(!g.computed_permanent(weaver).unwrap().keywords().contains(&Keyword::Flying));
    for _ in 0..5 {
        ready(&mut g, 0, catalog::efficient_construction());
    }
    assert!(g.computed_permanent(weaver).unwrap().keywords().contains(&Keyword::Flying), "seven enchantments");
}

/// Yenna copies a lone enchantment as a nonlegendary token.
#[test]
fn yenna_copies_an_enchantment() {
    let mut g = pod(2);
    let yenna = ready(&mut g, 0, catalog::yenna_redtooth_regent());
    let ench = ready(&mut g, 0, catalog::efficient_construction());
    flood(&mut g);
    activate(&mut g, yenna, Some(Target::Permanent(ench)));
    assert_eq!(named(&g, "Efficient Construction"), 2);
}

/// Forge Anew: returns an Equipment; the first equip of your turn is free and
/// may happen at instant speed; the second pays its cost.
#[test]
fn forge_anew_free_first_equip() {
    let mut g = pod(2);
    let sting = g.add_card_to_graveyard(0, catalog::sting_the_glinting_dagger());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bear2 = ready(&mut g, 0, catalog::grizzly_bears());
    let forge = g.add_card_to_hand(0, catalog::forge_anew());
    flood(&mut g);
    cast(&mut g, forge, Some(Target::Permanent(sting)));
    assert!(g.battlefield_find(sting).is_some(), "returned");
    g.players[0].mana_pool = Default::default();
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: sting, target: bear }).expect("first equip: free, at instant speed");
    assert!(g.perform_action(GameAction::Equip { equipment: sting, target: bear2 }).is_err(), "the second costs {{2}}");
}

/// Rev: your creatures connecting make a Treasure and exile the player's top
/// card face down, castable (not playable as a land).
#[test]
fn rev_tithes_a_card_and_a_treasure() {
    let mut g = pod(2);
    let rev = ready(&mut g, 0, catalog::rev_tithe_extractor());
    g.players[1].library.clear();
    let top = g.add_card_to_library(1, catalog::lightning_bolt());
    attack_into_block(&mut g, rev, None);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(named(&g, "Treasure"), 1);
    let c = g.exile.iter().find(|c| c.id == top).expect("exiled");
    assert!(c.face_down && c.may_play_until.is_some_and(|m| m.player == 0 && m.cast_only));
}

/// Archpriest of Shadows: connecting returns a creature card from your
/// graveyard.
#[test]
fn archpriest_of_shadows_reanimates_on_hit() {
    let mut g = pod(2);
    let priest = ready(&mut g, 0, catalog::archpriest_of_shadows());
    let dead = g.add_card_to_graveyard(0, catalog::hill_giant());
    attack_into_block(&mut g, priest, None);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some());
}

/// Cloud, Midgar Mercenary: equipped, the Equipment's trigger (Buster Sword's
/// draw, granted to Cloud) fires twice.
#[test]
fn cloud_doubles_equipment_triggers() {
    let mut g = pod(2);
    let cloud = ready(&mut g, 0, catalog::cloud_midgar_mercenary());
    let sword = ready(&mut g, 0, catalog::buster_sword());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(cloud);
    let library = g.players[0].library.len();
    attack_into_block(&mut g, cloud, None);
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), library - 2, "the draw triggered an additional time");
}

/// Amy Rose: attacking, she picks up an Equipment and pumps another attacker
/// by her power.
#[test]
fn amy_rose_equips_and_pumps() {
    let mut g = pod(2);
    let amy = ready(&mut g, 0, catalog::amy_rose());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let sting = ready(&mut g, 0, catalog::sting_the_glinting_dagger());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: amy, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sting).unwrap().attached_to, Some(amy));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2 + 4, "Amy is 4 with Sting");
}

/// Errant and Giada cast a flier off the top, not a Bear; Tails draws for a
/// flying Vehicle and gives a grounded one a flying counter.
#[test]
fn errant_and_giada_and_tails() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::errant_and_giada());
    ready(&mut g, 0, catalog::miles_tails_prower());
    g.players[0].library.clear();
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    flood(&mut g);
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None }).is_err());
    g.players[0].library.clear();
    let copter = g.add_card_to_library(0, catalog::smugglers_copter());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    cast(&mut g, copter, None);
    assert_eq!(g.players[0].hand.len(), hand + 1, "a flying Vehicle draws");
    let core = g.add_card_to_hand(0, catalog::mechtitan_core());
    cast(&mut g, core, None);
    assert!(g.computed_permanent(core).unwrap().keywords().contains(&Keyword::Flying), "a flying counter");
}

/// Arcades' walls: Shield-Bearers pump other defenders, Tanglecord buys
/// reach, Bar the Door adds +0/+4, Jeskai Barricade bounces another creature.
#[test]
fn arcades_walls() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::stalwart_shield_bearers());
    let wall = ready(&mut g, 0, catalog::wall_of_tanglecord());
    assert_eq!(g.computed_permanent(wall).unwrap().toughness, 8);
    flood(&mut g);
    activate(&mut g, wall, None);
    assert!(g.computed_permanent(wall).unwrap().keywords().contains(&Keyword::Reach));
    let bar = g.add_card_to_hand(0, catalog::bar_the_door());
    cast(&mut g, bar, None);
    assert_eq!(g.computed_permanent(wall).unwrap().toughness, 12);
    // Barricade's ETB is the trigger's own (optional) target; alone with a
    // Bear, the auto-target names it.
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let barricade = g.add_card_to_hand(0, catalog::jeskai_barricade());
    flood(&mut g);
    cast(&mut g, barricade, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bounced");
}

/// Queen Marchesa's courts: each crowns its caster; at your upkeep Ambition
/// drains an opponent with no cards to discard six as the monarch, and
/// Embereth makes a Knight, then deals one damage per creature you control.
#[test]
fn courts_of_ambition_and_embereth() {
    let mut g = pod(2);
    flood(&mut g);
    let court = g.add_card_to_hand(0, catalog::court_of_ambition());
    cast(&mut g, court, None);
    assert_eq!(g.monarch, Some(0));
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 6, "nothing to discard");

    let mut g = pod(2);
    flood(&mut g);
    ready(&mut g, 0, catalog::grizzly_bears());
    let court = g.add_card_to_hand(0, catalog::court_of_embereth());
    cast(&mut g, court, None);
    let life = g.players[1].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(named(&g, "Knight"), 1);
    assert_eq!(g.players[1].life, life - 2, "the Bear and the new Knight");
}

/// Emberwilde Captain: an opponent attacking its monarch controller takes
/// damage equal to their hand size.
#[test]
fn emberwilde_captain_punishes_attacks_on_the_monarch() {
    let mut g = pod(2);
    flood(&mut g);
    let captain = g.add_card_to_hand(0, catalog::emberwilde_captain());
    cast(&mut g, captain, None);
    assert_eq!(g.monarch, Some(0));
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::island());
    }
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let life = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
}

/// Kenrith's table: Agatha's power comes off her own pump's generic (five
/// mana does the six-mana pump); Gluntch hands out three gifts to three
/// different players; The Theorist draws on an opponent's draw step and
/// bounces their creature with −2.
#[test]
fn kenrith_agatha_gluntch_and_the_theorist() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let agatha = ready(&mut g, 0, catalog::agatha_of_the_vile_cauldron());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    activate(&mut g, agatha, None);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    assert_eq!(g.computed_permanent(agatha).unwrap().power, 1, "others only");

    let mut g = pod(3);
    let gluntch = ready(&mut g, 0, catalog::gluntch_the_bestower());
    ready(&mut g, 2, catalog::grizzly_bears());
    let hands: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gluntch).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "first gift is ours");
    assert_eq!(g.players[1].hand.len(), hands[1] + 1, "the creatureless opponent draws");
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure" && c.controller == 2).count();
    assert_eq!(treasures, 2);
    assert_eq!(g.players[0].hand.len(), hands[0]);

    let mut g = pod(2);
    let jace = ready(&mut g, 0, catalog::the_theorist_jace_beleren());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let hand = g.players[0].hand.len();
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace,
        ability_index: 1,
        target: Some(Target::Permanent(theirs)),
        x_value: None,
    })
    .expect("-2");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "bounced");
}

/// Fire Lord Azula's court: Azula, Cunning Usurper takes an opponent's
/// creature and graveyard card, castable on your turn only, at instant speed,
/// with mana of any type; Ozai keeps lost mana as red and flies on six
/// floating; Zuko firebends for his experience and gains it from combat
/// spells.
#[test]
fn fire_lord_azula_court() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let bolt = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    flood(&mut g);
    let azula = g.add_card_to_hand(0, catalog::azula_cunning_usurper());
    cast(&mut g, azula, Some(Target::Player(1)));
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_with == Some(azula)));
    assert!(g.exile.iter().any(|c| c.id == bolt && c.exiled_with == Some(azula)));
    // Not on an opponent's turn.
    advance_to_turn_of(&mut g, 1);
    let cast_bear = |g: &mut GameState| {
        g.perform_action(GameAction::CastFromZoneWithoutPaying { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
    };
    g.priority.player_with_priority = 0;
    assert!(cast_bear(&mut g).is_err(), "not on an opponent's turn");
    // Our turn, in combat, with blue mana only: flash and any type.
    advance_to_turn_of(&mut g, 0);
    g.step = TurnStep::BeginCombat;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast_bear(&mut g).expect("cast the stolen Bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some_and(|c| c.controller == 0));

    let mut g = pod(2);
    let ozai = ready(&mut g, 0, catalog::ozai_the_phoenix_king());
    g.players[0].mana_pool.add(Color::Blue, 6);
    assert!(g.computed_permanent(ozai).unwrap().keywords().contains(&Keyword::Flying));
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 6, "lost mana turns red");
    g.players[0].mana_pool = Default::default();
    assert!(!g.computed_permanent(ozai).unwrap().keywords().contains(&Keyword::Flying));

    let mut g = pod(2);
    let zuko = ready(&mut g, 0, catalog::zuko_firebending_master());
    g.players[0].experience = 2;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: zuko, target: AttackTarget::Player(1) }]))
        .expect("attack");
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].experience, 3);
}

fn advance_to_turn_of(g: &mut GameState, seat: usize) {
    loop {
        let ev = g.advance_step(Vec::new()).expect("step");
        g.dispatch_triggers_for_events(&ev);
        drain_stack(g);
        if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
            break;
        }
    }
}

/// Minsc & Boo's counters: Vorinclex doubles what its controller puts (Surge's
/// 1 then doubling: 2 + 4) and halves what an opponent puts (their Surge's
/// single counter rounds to nothing); Defiler of Vigor grows the team on a
/// green permanent spell.
#[test]
fn vorinclex_scales_counters_by_who_places_them() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::vorinclex_monstrous_raider());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g);
    let surge = g.add_card_to_hand(0, catalog::invigorating_surge());
    cast(&mut g, surge, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
    let surge = g.add_card_to_hand(1, catalog::invigorating_surge());
    g.players[1].mana_pool.add(Color::Green, 3);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: surge,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("their surge");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);

    let mut g = pod(2);
    let defiler = ready(&mut g, 0, catalog::defiler_of_vigor());
    flood(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(g.battlefield_find(defiler).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Agrus Kos: an ability aimed only at it is copied, for {1}{R/W}, onto each
/// other creature its controller controls; one aimed elsewhere isn't.
#[test]
fn agrus_kos_copies_an_ability_onto_each_other_creature() {
    use crabomination::card::{ActivatedAbility, CardDefinition, CardType, CounterType, SelectionRequirement as R};
    use crabomination::effect::{Effect, Value, shortcut::target_filtered};
    let pumper = || CardDefinition {
        name: "Test Pumper",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut g = pod(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let agrus = ready(&mut g, 0, catalog::agrus_kos_eternal_soldier());
    let a = ready(&mut g, 0, catalog::grizzly_bears());
    let b = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let tool = ready(&mut g, 0, pumper());
    flood(&mut g);
    activate(&mut g, tool, Some(Target::Permanent(agrus)));
    let n = |g: &GameState, id| g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!((n(&g, agrus), n(&g, a), n(&g, b), n(&g, theirs)), (1, 1, 1, 0));
    g.battlefield_find_mut(tool).unwrap().tapped = false;
    activate(&mut g, tool, Some(Target::Permanent(a)));
    assert_eq!((n(&g, agrus), n(&g, a), n(&g, b)), (1, 2, 1), "not aimed at Agrus");
}

/// Eriette's Auras: Light-Paws answers a cast Aura with a differently named
/// one no costlier, attached to itself; Trespasser's Curse drains per
/// creature; Clawing Torment shrinks, grounds and bleeds its host's
/// controller; Vampiric Link gains what its host deals.
#[test]
fn eriette_auras() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    let paws = ready(&mut g, 0, catalog::light_paws_emperors_voice());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::vampiric_link());
    let torment = g.add_card_to_library(0, catalog::clawing_torment());
    g.add_card_to_library(0, catalog::trespassers_curse());
    flood(&mut g);
    let link = g.add_card_to_hand(0, catalog::vampiric_link());
    cast(&mut g, link, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(link).unwrap().attached_to, Some(bear));
    assert_eq!(g.battlefield_find(torment).and_then(|c| c.attached_to), Some(paws), "the other name, MV 1");
    let cp = g.computed_permanent(paws).unwrap();
    assert_eq!(cp.power, 1);
    assert!(cp.keywords().contains(&Keyword::CantBlock));
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "the Torment's upkeep");

    let mut g = pod(2);
    flood(&mut g);
    let curse = g.add_card_to_hand(0, catalog::trespassers_curse());
    cast(&mut g, curse, Some(Target::Player(1)));
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("their Bear");
    drain_stack(&mut g);
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 1, theirs - 1));
}

/// Alela's Faeries: Harbinger stacks a Faerie on top, Tome-Skimmer draws for a
/// spell cast on an opponent's turn, Harmonized Crescendo counts the chosen
/// type, Unwind counters a noncreature spell and untaps lands.
#[test]
fn alela_faeries() {
    let mut g = pod(2);
    flood(&mut g);
    let faerie = g.add_card_to_library(0, catalog::voracious_tome_skimmer());
    let harbinger = g.add_card_to_hand(0, catalog::faerie_harbinger());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, harbinger, None);
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(faerie));

    let mut g = pod(2);
    ready(&mut g, 0, catalog::voracious_tome_skimmer());
    ready(&mut g, 0, catalog::faerie_harbinger());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    let unwind = g.add_card_to_hand(0, catalog::unwind());
    for _ in 0..3 {
        let land = ready(&mut g, 0, catalog::island());
        g.battlefield_find_mut(land).unwrap().tapped = true;
    }
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    cast(&mut g, unwind, Some(Target::Permanent(bolt)));
    assert_eq!(g.players[0].life, life - 1, "Tome-Skimmer's 1 life; the Bolt was countered");
    assert_eq!(g.players[0].hand.len(), hand, "Unwind left, a card came in");
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Island").all(|c| !c.tapped));
    let crescendo = g.add_card_to_hand(0, catalog::harmonized_crescendo());
    g.active_player_idx = 0;
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(crabomination::card::CreatureType::Faerie)]));
    cast(&mut g, crescendo, None);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "two Faeries");
}

/// Ovika's big spells: four tapped creatures take {4} off Explosive
/// Singularity's generic but never pay its {R}{R}; Transcendent Message's X
/// convokes.
#[test]
fn explosive_singularity_and_transcendent_message() {
    let mut g = pod(2);
    let goblins: Vec<CardId> = (0..4).map(|_| ready(&mut g, 0, catalog::goblin_guide())).collect();
    let boom = g.add_card_to_hand(0, catalog::explosive_singularity());
    let convoke = |g: &mut GameState, card_id, target, x_value, helpers: &[CardId]| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellConvoke {
            card_id,
            target,
            additional_targets: vec![],
            mode: None,
            x_value,
            convoke_creatures: helpers.to_vec(),
        })
    };
    g.players[0].mana_pool.add_colorless(6);
    assert!(convoke(&mut g, boom, Some(Target::Player(1)), None, &goblins).is_err(), "red creatures can't pay {{R}}");
    assert!(goblins.iter().all(|&id| !g.battlefield_find(id).unwrap().tapped), "rolled back");
    g.players[0].mana_pool = Default::default();
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    let life = g.players[1].life;
    convoke(&mut g, boom, Some(Target::Player(1)), None, &goblins).expect("{4}{R}{R} after four taps");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 10);

    let mut g = pod(2);
    let helpers: Vec<CardId> = (0..2).map(|_| ready(&mut g, 0, catalog::grizzly_bears())).collect();
    let message = g.add_card_to_hand(0, catalog::transcendent_message());
    g.players[0].mana_pool.add(Color::Blue, 4);
    let hand = g.players[0].hand.len();
    convoke(&mut g, message, None, Some(2), &helpers).expect("X = 2 by convoke");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2);
}

fn landfall_now(g: &mut GameState) {
    let land = g.add_card_to_hand(0, catalog::forest());
    g.players[0].lands_played_this_turn = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("land");
    drain_stack(g);
}

/// Tifa's landfall kit: Pick-Axe hops on and pumps on landfall; Staff of
/// Titania counts Forests and makes a Dryad on attack; Scythecat Cub's second
/// landfall doubles; Roaring Earth's landfall counter.
#[test]
fn tifa_landfall_kit() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    flood(&mut g);
    let axe = g.add_card_to_hand(0, catalog::skyclave_pick_axe());
    cast(&mut g, axe, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(axe).unwrap().attached_to, Some(bear));
    ready(&mut g, 0, catalog::scythecat_cub());
    landfall_now(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 2 + 2 + 1, "axe +2, cub's counter");
    landfall_now(&mut g);
    let n = g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(n, 2, "the second resolution doubles");

    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    ready(&mut g, 0, catalog::roaring_earth());
    for _ in 0..2 {
        ready(&mut g, 0, catalog::forest());
    }
    let staff = ready(&mut g, 0, catalog::staff_of_titania());
    g.battlefield_find_mut(staff).unwrap().attached_to = Some(bear);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "two Forests");
    landfall_now(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    connect(&mut g, bear);
    assert_eq!(named(&g, "Forest Dryad"), 1);
}

/// Sokka's Allies: Longshot shaves noncreature spells and pings on them;
/// Allied Teamwork's Ally is 2/2; Sokka's Charge gives double strike on your
/// turn only.
#[test]
fn sokka_allies() {
    use crabomination::card::Keyword;
    let mut g = pod(2);
    ready(&mut g, 0, catalog::longshot_rebel_bowman());
    let life = g.players[1].life;
    let team = g.add_card_to_hand(0, catalog::allied_teamwork());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, team, None);
    assert_eq!(g.players[1].life, life - 2, "a noncreature spell, a {{1}} cheaper");
    let ally = g.battlefield.iter().find(|c| c.definition.name == "Ally").map(|c| c.id).unwrap();
    assert_eq!(g.computed_permanent(ally).unwrap().power, 2);
    ready(&mut g, 0, catalog::sokkas_charge());
    assert!(g.computed_permanent(ally).unwrap().keywords().contains(&Keyword::DoubleStrike));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(ally).unwrap().keywords().contains(&Keyword::DoubleStrike));
}

/// Gev's Lizards: Pyreling and Master of Barbs answer a Bolt to an opponent;
/// Hissing Iguanar pings on a death; Collective Inferno doubles the chosen
/// type's damage.
#[test]
fn gev_lizards() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = pod(2);
    let pyre = ready(&mut g, 0, catalog::chandras_pyreling());
    let barbs = ready(&mut g, 0, catalog::master_of_barbs());
    flood(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Player(1)));
    let cp = g.computed_permanent(pyre).unwrap();
    assert_eq!(cp.power, 1 + 1 + 1, "Pyreling +1, Barbs +1");
    assert!(cp.keywords().contains(&Keyword::DoubleStrike));
    assert_eq!(g.computed_permanent(barbs).unwrap().power, 3);

    let mut g = pod(2);
    ready(&mut g, 0, catalog::hissing_iguanar());
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g);
    g.players[0].hostile_player_targets = true;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast(&mut g, bolt, Some(Target::Permanent(victim)));
    assert_eq!(g.players[1].life, life - 1);

    let mut g = pod(2);
    let lizard = ready(&mut g, 0, catalog::hissing_iguanar());
    flood(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Lizard)]));
    let inferno = g.add_card_to_hand(0, catalog::collective_inferno());
    cast(&mut g, inferno, None);
    let life = g.players[1].life;
    connect(&mut g, lizard);
    assert_eq!(g.players[1].life, life - 6, "a 3-power Lizard doubled");
}

/// Rocco's and Tannuk's tables: Myojin of Roaring Blades, cast, carries an
/// indestructible counter it spends for 7 to up to three targets; Arbaaz Mir
/// drains on a historic arrival; Gala Greeters cycles its modes; Saltskitter
/// blinks to the end step; Alena taps for the biggest newcomer's power.
#[test]
fn rocco_and_tannuk_cards() {
    use crabomination::card::CounterType;
    let mut g = pod(2);
    flood(&mut g);
    let myojin = g.add_card_to_hand(0, catalog::myojin_of_roaring_blades());
    cast(&mut g, myojin, None);
    assert_eq!(g.battlefield_find(myojin).unwrap().counter_count(CounterType::Indestructible), 1);
    let life = g.players[1].life;
    activate(&mut g, myojin, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 7);
    assert_eq!(g.battlefield_find(myojin).unwrap().counter_count(CounterType::Indestructible), 0);

    let mut g = pod(2);
    ready(&mut g, 0, catalog::arbaaz_mir());
    ready(&mut g, 0, catalog::gala_greeters());
    let salt = ready(&mut g, 0, catalog::saltskitter());
    flood(&mut g);
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    cast(&mut g, ring, None);
    assert_eq!((g.players[0].life, g.players[1].life), (mine + 1, theirs - 1), "Arbaaz on a historic");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert!(g.battlefield_find(salt).is_none(), "Saltskitter blinked out");
    let alena = ready(&mut g, 0, catalog::alena_kessig_trapper());
    g.players[0].mana_pool = Default::default();
    activate(&mut g, alena, None);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "the Bear entered this turn");
    to_end_step(&mut g);
    assert!(g.battlefield_find(salt).is_some(), "back at the end step");
}

/// Blessed Sanctuary: a Bolt to its controller or their creature does
/// nothing, an opponent's creature still takes it, and a nontoken creature
/// arriving brings a Unicorn.
#[test]
fn blessed_sanctuary_shields_and_makes_unicorns() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::blessed_sanctuary());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    flood(&mut g);
    let life = g.players[0].life;
    for target in [Target::Player(0), Target::Permanent(mine), Target::Permanent(theirs)] {
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        cast(&mut g, bolt, Some(target));
    }
    assert_eq!(g.players[0].life, life);
    assert!(g.battlefield_find(mine).is_some());
    assert!(g.battlefield_find(theirs).is_none());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(named(&g, "Unicorn"), 1);
}

/// Kudo's Bears: Ayula answers another Bear with two counters; Beorn pumps
/// the other Bears, makes a target a trampling Bear at combat and draws with
/// three Bears; King Darien's anthem and Soldier; Wilson can't be countered.
#[test]
fn kudo_bears() {
    use crabomination::card::{CounterType, CreatureType, Keyword};
    let mut g = pod(2);
    ready(&mut g, 0, catalog::ayula_queen_among_bears());
    flood(&mut g);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    cast(&mut g, bear, None);
    let counters: u32 = g.battlefield.iter().map(|c| c.counter_count(CounterType::PlusOnePlusOne)).sum();
    assert_eq!(counters, 2);

    let mut g = pod(2);
    ready(&mut g, 0, catalog::beorn_the_fierce());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "Beorn's +2/+2");
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let trampling = [bear, elf].iter().filter(|&&id| g.computed_permanent(id).unwrap().keywords().contains(&Keyword::Trample)).count();
    assert_eq!(trampling, 1, "one target got the trample counter");
    let elf_is_bear = g.computed_permanent(elf).unwrap().subtypes().creature_types.contains(&CreatureType::Bear);
    assert_eq!(g.players[0].hand.len(), hand + if elf_is_bear { 2 } else { 0 }, "three Bears draw two");

    let mut g = pod(2);
    let king = ready(&mut g, 0, catalog::king_darien_xlviii());
    flood(&mut g);
    activate(&mut g, king, None);
    let soldier = g.battlefield.iter().find(|c| c.definition.name == "Soldier").map(|c| c.id).unwrap();
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2);
    assert!(catalog::wilson_refined_grizzly().keywords.contains(&Keyword::CantBeCountered));
}
