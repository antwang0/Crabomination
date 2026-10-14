//! Commander: one-card gaps of EDHREC ranks 2501-3500 (`decks::cmdr_rank3k_b`).

use crabomination::card::{CardId, CounterType};
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

fn act(g: &mut GameState, action: GameAction) {
    flood(g);
    g.priority.player_with_priority = 0;
    g.perform_action(action).expect("action");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, target: Option<Target>) {
    act(g, GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None });
}

fn activate(g: &mut GameState, id: CardId, index: usize, target: Option<Target>) {
    act(g, GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
}

fn named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.definition.name == name).count()
}

fn plus(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne)
}

fn declare(g: &mut GameState, attackers: &[CardId], defender: usize) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect(),
    ))
    .expect("attack");
    drain_stack(g);
}

fn finish_combat(g: &mut GameState) {
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Demonic Vigor returns the dead host to its owner's hand.
#[test]
fn demonic_vigor_returns_the_host() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let vigor = g.add_card_to_hand(0, catalog::demonic_vigor());
    cast(&mut g, vigor, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Stark's Ingenuity grows its host on each card you draw.
#[test]
fn starks_ingenuity_grows_on_draws() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let aura = g.add_card_to_hand(0, catalog::starks_ingenuity());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    let divination = g.add_card_to_hand(0, catalog::divination());
    cast(&mut g, divination, None);
    assert_eq!(plus(&g, bear), 2);
}

/// Seismic Tutelage starts at one counter and doubles on each attack.
#[test]
fn seismic_tutelage_doubles_on_attack() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::seismic_tutelage());
    cast(&mut g, aura, Some(Target::Permanent(bear)));
    assert_eq!(plus(&g, bear), 1);
    declare(&mut g, &[bear], 1);
    assert_eq!(plus(&g, bear), 2);
}

/// Xiahou Dun's sacrifice works only on your turn before attackers.
#[test]
fn xiahou_dun_regrows_a_black_card_before_combat() {
    let mut g = pod(2);
    let dun = ready(&mut g, 0, catalog::xiahou_dun_the_one_eyed());
    let murder = g.add_card_to_graveyard(0, catalog::murder());
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: dun,
            ability_index: 0,
            target: Some(Target::Permanent(murder)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "after attackers are declared"
    );
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, dun, 0, Some(Target::Permanent(murder)));
    assert!(g.players[0].hand.iter().any(|c| c.id == murder));
}

/// Arni untaps when another nontoken creature of yours enters.
#[test]
fn arni_untaps_on_a_nontoken_arrival() {
    let mut g = pod(2);
    let arni = ready(&mut g, 0, catalog::arni_humble_scribe());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    activate(&mut g, arni, 0, None);
    assert!(g.battlefield_find(arni).unwrap().tapped);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert!(!g.battlefield_find(arni).unwrap().tapped);
}

/// Imperial Subduer taps a blocker when a Samurai attacks alone; Peerless
/// Samurai's lone attack discounts the next spell.
#[test]
fn samurai_attacking_alone() {
    let mut g = pod(2);
    let subduer = ready(&mut g, 0, catalog::imperial_subduer());
    let blocker = ready(&mut g, 1, catalog::grizzly_bears());
    declare(&mut g, &[subduer], 1);
    assert!(g.battlefield_find(blocker).unwrap().tapped);

    let mut g = pod(2);
    let samurai = ready(&mut g, 0, catalog::peerless_samurai());
    declare(&mut g, &[samurai], 1);
    g.step = TurnStep::PostCombatMain;
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{1}{G} for {G}");
}

/// Campfire puts commanders from the command zone and graveyard into hand,
/// then shuffles the graveyard away.
#[test]
fn campfire_recalls_the_commanders() {
    let mut g = pod(2);
    let fire = ready(&mut g, 0, catalog::campfire());
    let cmdr = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].commanders.push(cmdr);
    g.add_card_to_graveyard(0, catalog::island());
    activate(&mut g, fire, 1, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == cmdr));
    assert!(g.players[0].graveyard.is_empty());
    assert!(g.battlefield_find(fire).is_none(), "exiled as a cost");
}

/// Vogar grows only on deaths during your turn and draws for its counters.
#[test]
fn vogar_grows_on_your_turn_and_cashes_in() {
    let mut g = pod(2);
    let vogar = ready(&mut g, 0, catalog::vogar_necropolis_tyrant());
    let a = ready(&mut g, 1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(a)));
    assert_eq!(plus(&g, vogar), 1);
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    let hand = g.players[0].hand.len();
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(vogar)));
    assert_eq!(g.players[0].hand.len(), hand + 1);
}

/// Herald of Ilharg grows on creature spells and burns with a big one.
#[test]
fn herald_of_ilharg_burns_on_a_big_creature() {
    let mut g = pod(3);
    let herald = ready(&mut g, 0, catalog::herald_of_ilharg());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, None);
    assert_eq!(plus(&g, herald), 2);
    assert_eq!(g.players[1].life, 20);
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    cast(&mut g, angel, None);
    assert_eq!(g.players[1].life, 16);
    assert_eq!(g.players[2].life, 16);
}

/// Force of Virtue can be pitched off a white card on an opponent's turn.
#[test]
fn force_of_virtue_pitches_off_turn() {
    let mut g = pod(2);
    g.active_player_idx = 1;
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let force = g.add_card_to_hand(0, catalog::force_of_virtue());
    let pitch = g.add_card_to_hand(0, catalog::serra_angel());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: force,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("pitch");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3);
}

/// Vision of Love sacrifices an artifact (or discards) to draw two.
#[test]
fn vision_of_love_pays_to_draw_two() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::sol_ring());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::island());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let vision = g.add_card_to_hand(0, catalog::vision_of_love());
    cast(&mut g, vision, None);
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(named(&g, 0, "Sol Ring"), 0);
}

/// Ultron makes a Robot on entry and drains when another artifact hits the
/// graveyard.
#[test]
fn ultron_drains_off_artifacts() {
    let mut g = pod(3);
    let ultron = g.add_card_to_hand(0, catalog::ultron_the_annihilator());
    cast(&mut g, ultron, None);
    assert_eq!(named(&g, 0, "Robot Villain"), 1);
    let ring = ready(&mut g, 0, catalog::sol_ring());
    let shatter = g.add_card_to_hand(0, catalog::shatter());
    cast(&mut g, shatter, Some(Target::Permanent(ring)));
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[2].life, 19);
}

/// Nadier grows when a token leaves and leaves Elves equal to its power.
#[test]
fn nadier_grows_and_leaves_elves() {
    let mut g = pod(2);
    let nadier = ready(&mut g, 0, catalog::nadier_agent_of_the_duskenel());
    let ultron = g.add_card_to_hand(0, catalog::ultron_the_annihilator());
    cast(&mut g, ultron, None);
    let robot = g.battlefield.iter().find(|c| c.definition.name == "Robot Villain").unwrap().id;
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(robot)));
    assert_eq!(plus(&g, nadier), 1);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(nadier)));
    assert_eq!(named(&g, 0, "Elf Warrior"), 4);
}

/// Molten Monstrosity costs less by the greatest power you control.
#[test]
fn molten_monstrosity_discounts_by_power() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::serra_angel());
    let m = g.add_card_to_hand(0, catalog::molten_monstrosity());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: m, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("{7}{R} less 4");
}

/// Leafkin Avenger taps for {G} per 4-power creature.
#[test]
fn leafkin_avenger_taps_for_big_creatures() {
    let mut g = pod(2);
    let leafkin = ready(&mut g, 0, catalog::leafkin_avenger());
    ready(&mut g, 0, catalog::serra_angel());
    ready(&mut g, 0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: leafkin,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2);
}

/// Guardian Naga takes no damage during its controller's turn only.
#[test]
fn guardian_naga_is_shielded_on_your_turn() {
    for (active, survives) in [(0, true), (1, false)] {
        let mut g = pod(2);
        let naga = ready(&mut g, 0, catalog::guardian_naga());
        g.active_player_idx = active;
        for _ in 0..2 {
            let bolt = g.add_card_to_hand(0, catalog::lava_axe());
            let _ = bolt;
        }
        let ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
        let hit = crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::ExactObjects(vec![naga]),
            amount: crabomination::effect::Value::Const(6),
        };
        g.resolve_effect(&hit, &ctx).unwrap();
        g.check_state_based_actions();
        assert_eq!(g.battlefield_find(naga).is_some(), survives, "active={active}");
    }
}

/// Dread Linnorm's adventure counters, untaps and shields a creature.
#[test]
fn dread_linnorm_scale_deflection() {
    let mut g = pod(2);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let linnorm = g.add_card_to_hand(0, catalog::dread_linnorm());
    act(&mut g, GameAction::CastAdventure {
        card_id: linnorm,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert_eq!(plus(&g, bear), 2);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Omnispell Adept casts an instant from hand for free.
#[test]
fn omnispell_adept_casts_free() {
    let mut g = pod(2);
    let adept = ready(&mut g, 0, catalog::omnispell_adept());
    g.add_card_to_hand(0, catalog::lava_axe());
    activate(&mut g, adept, 0, None);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lava Axe"));
}

/// Orca grows on deaths and splits its power when it dies.
#[test]
fn orca_divides_its_power_on_death() {
    let mut g = pod(2);
    let orca = ready(&mut g, 0, catalog::orca_siege_demon());
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(bear)));
    assert_eq!(plus(&g, orca), 1);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(orca)));
    let dealt = 20 - g.players[1].life + 20 - g.players[0].life;
    assert_eq!(dealt, 6, "six damage went somewhere");
}

/// Desperate Plea reanimates a creature no bigger than the sacrifice.
#[test]
fn desperate_plea_reanimates_by_power() {
    for (fodder, back) in [(catalog::serra_angel(), true), (catalog::llanowar_elves(), false)] {
        let mut g = pod(2);
        let victim = ready(&mut g, 0, fodder);
        let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
        let plea = g.add_card_to_hand(0, catalog::desperate_plea());
        let _ = victim;
        act(&mut g, GameAction::CastSpell {
            card_id: plea,
            target: Some(Target::Permanent(dead)),
            additional_targets: vec![],
            mode: Some(0),
            x_value: None,
        });
        assert_eq!(g.battlefield_find(dead).is_some(), back, "back={back}");
    }
}

/// Danitha grows when you cast a spell targeting an opponent's creature.
#[test]
fn danitha_grows_off_targeting_opponents() {
    let mut g = pod(2);
    let danitha = ready(&mut g, 0, catalog::danitha_spear_of_agony());
    let mine = ready(&mut g, 0, catalog::grizzly_bears());
    let theirs = ready(&mut g, 1, catalog::grizzly_bears());
    let giant = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, giant, Some(Target::Permanent(mine)));
    assert_eq!(plus(&g, danitha), 0);
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, murder, Some(Target::Permanent(theirs)));
    assert_eq!(plus(&g, danitha), 1);
}

/// Spider-Man deals damage equal to the cards discarded.
#[test]
fn spider_man_burns_for_discards() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::spider_man_new_champion());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    for _ in 0..2 {
        g.add_card_to_hand(0, catalog::island());
    }
    let discard = crabomination::effect::Effect::Discard {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::Const(2),
        random: false,
    };
    let evs = g.resolve_effect(&discard, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// The Destined White Mage counts three with a full party.
#[test]
fn destined_white_mage_rewards_a_full_party() {
    let mut g = pod(2);
    let mage = ready(&mut g, 0, catalog::the_destined_white_mage());
    let ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    let gain = crabomination::effect::Effect::GainLife {
        who: crabomination::effect::Selector::You,
        amount: crabomination::effect::Value::ONE,
    };
    let evs = g.resolve_effect(&gain, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(plus(&g, mage), 1);
}

/// Jet puts a cheap creature from the top five onto the battlefield
/// attacking.
#[test]
fn jet_rebel_leader_deploys_an_attacker() {
    let mut g = pod(2);
    let jet = ready(&mut g, 0, catalog::jet_rebel_leader());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.add_card_to_library(0, catalog::grizzly_bears());
    declare(&mut g, &[jet], 1);
    assert_eq!(named(&g, 0, "Grizzly Bears"), 1);
    finish_combat(&mut g);
    assert_eq!(g.players[1].life, 15);
}
