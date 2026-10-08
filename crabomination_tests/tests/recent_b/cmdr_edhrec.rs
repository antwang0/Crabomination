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
