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
