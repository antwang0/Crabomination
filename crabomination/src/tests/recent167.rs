//! Functionality tests for `catalog::sets::decks::recent167` — DFT Speed/Exhaust
//! staples plus the `Value::PlayerSpeed` and `EventKind::ExhaustAbilityActivated`
//! primitives.

use crate::card::{CounterType, Keyword};
use crate::catalog;
use crate::game::types::Target;
use crate::game::*;
use crate::mana::Color;

/// Loxodon Surveyor's max-speed graveyard ability exiles itself to draw, and is
/// gated behind speed 4.
#[test]
fn loxodon_surveyor_max_speed_draws_from_graveyard() {
    let mut g = two_player_game();
    let surveyor = g.add_card_to_graveyard(0, catalog::loxodon_surveyor());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    // Below max speed → activation rejected.
    g.players[0].speed = 3;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: surveyor, ability_index: 0, target: None,
            additional_targets: Vec::new(), x_value: None,
        }).is_err(),
        "not usable below max speed"
    );
    // At max speed → exile self, draw a card.
    g.players[0].speed = 4;
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: surveyor, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("max speed gy draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
    assert!(g.exile.iter().any(|c| c.id == surveyor), "exiled itself as a cost");
}

/// Leonin Surveyor has first strike only during its controller's turn.
#[test]
fn leonin_surveyor_first_strike_only_your_turn() {
    let mut g = two_player_game();
    let leonin = g.add_card_to_battlefield(0, catalog::leonin_surveyor());
    g.active_player_idx = 0;
    assert!(g.computed_permanent(leonin).unwrap().keywords.contains(&Keyword::FirstStrike),
        "first strike on your turn");
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(leonin).unwrap().keywords.contains(&Keyword::FirstStrike),
        "no first strike on opponent's turn");
}

/// Ooze Patrol mills two, then counts artifact/creature cards in the graveyard.
#[test]
fn ooze_patrol_grows_with_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // creature
    g.add_card_to_graveyard(0, catalog::sol_ring()); // artifact
    g.add_card_to_library(0, catalog::forest()); // milled — neither
    g.add_card_to_library(0, catalog::grizzly_bears()); // milled — creature
    let ooze = g.move_card_to_battlefield_for_test(0, catalog::ooze_patrol());
    drain_stack(&mut g);
    // Two starting + one milled creature = 3 art/creature cards in gy.
    assert_eq!(g.battlefield_find(ooze).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3);
}

/// Marketback Walker enters with X counters and draws that many on death.
#[test]
fn marketback_walker_enters_with_x_and_draws_on_death() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::forest()); }
    let walker = g.add_card_to_hand(0, catalog::marketback_walker());
    g.players[0].mana_pool.add_colorless(6); // {X}{X} with X=3
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.cast_spell(walker, None, vec![], None, Some(3)).expect("cast X=3");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(walker).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 3,
        "enters with 3 counters");
    let before = g.players[0].hand.len();
    g.battlefield_find_mut(walker).unwrap().damage = 3; // lethal vs its 3/3
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 3, "drew 3 on death (one per counter)");
}

/// Momentum Breaker's sac ability gains life equal to your speed.
#[test]
fn momentum_breaker_gains_life_equal_to_speed() {
    let mut g = two_player_game();
    let mb = g.add_card_to_battlefield(0, catalog::momentum_breaker());
    g.players[0].speed = 3;
    g.players[0].life = 20;
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mb, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("sac: gain life = speed");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3 life (speed)");
}

/// Momentum Breaker's ETB forces each opponent to sacrifice a creature.
#[test]
fn momentum_breaker_etb_edict() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::momentum_breaker());
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == victim), "opponent sacrificed its creature");
}

/// Adrenaline Jockey punishes off-turn spellcasting and grows on exhaust use.
#[test]
fn adrenaline_jockey_off_turn_burn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::adrenaline_jockey());
    // It's player 0's turn; player 1 casts a spell → 4 damage to player 1.
    g.active_player_idx = 0;
    g.players[1].life = 20;
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.cast_spell(bolt, Some(Target::Player(0)), vec![], None, None).expect("opp casts on your turn");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 16, "off-turn caster took 4");
}

/// Adrenaline Jockey grows when you activate an exhaust ability (Mutant Surveyor
/// has none, so use a known exhaust card in the same batch is unavailable; assert
/// the counter path directly by activating an exhaust ability on another source).
#[test]
fn adrenaline_jockey_grows_on_exhaust() {
    let mut g = two_player_game();
    let jockey = g.add_card_to_battlefield(0, catalog::adrenaline_jockey());
    // Jeong Jeong, the Deserter has an exhaust ability ({3}).
    let jj = g.add_card_to_battlefield(0, catalog::jeong_jeong_the_deserter());
    g.clear_sickness(jj);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: jj, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate exhaust ability");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jockey).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1,
        "gained a +1/+1 counter from the exhaust activation");
}

/// Hour of Victory makes a Zombie on ETB and, at max speed, tutors to hand.
#[test]
fn hour_of_victory_token_and_tutor() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let target = g.add_card_to_library(0, catalog::grizzly_bears());
    let hov = g.move_card_to_battlefield_for_test(0, catalog::hour_of_victory());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.is_token && c.definition.name == "Zombie"), "made a Zombie");
    g.players[0].speed = 4;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(target))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: hov, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("max speed sac: tutor");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == target), "tutored the card to hand");
}

/// Intimidation Tactics exiles an artifact/creature card from an opponent's hand.
#[test]
fn intimidation_tactics_exiles_from_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let creature = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::lightning_bolt()); // not a valid pick
    let spell = g.add_card_to_hand(0, catalog::intimidation_tactics());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Discard(vec![creature])]));
    g.cast_spell(spell, Some(Target::Player(1)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == creature), "exiled the creature card");
}

/// Muraganda Raceway taps for {C}, and doubles to {C}{C} at max speed.
#[test]
fn muraganda_raceway_max_speed_double_mana() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::muraganda_raceway());
    g.players[0].speed = 4;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("max speed: {T} add CC");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2, "added two colorless at max speed");
}

/// Avishkar Raceway's max-speed loot ability requires a discard and max speed.
#[test]
fn avishkar_raceway_max_speed_loots() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::avishkar_raceway());
    let pitch = g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].speed = 4;
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("max speed: discard, draw");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded a card");
    assert_eq!(g.players[0].hand.len(), hand_before, "net-zero hand (discard 1, draw 1)");
}

/// Night Market taps for the color chosen as it entered.
#[test]
fn night_market_taps_for_chosen_color() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    let land = g.move_card_to_battlefield_for_test(0, catalog::night_market());
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("{T}: add chosen color");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "added blue (the chosen color)");
}

/// Marshals' Pathcruiser tutors a basic land on ETB and animates via exhaust.
#[test]
fn marshals_pathcruiser_tutors_and_animates() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let basic = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    let vehicle = g.move_card_to_battlefield_for_test(0, catalog::marshals_pathcruiser());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == basic), "tutored a basic land to hand");
    assert!(!g.battlefield_find(vehicle).unwrap().definition.is_creature(), "not a creature before exhaust");
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vehicle, ability_index: 0, target: None,
        additional_targets: Vec::new(), x_value: None,
    })
    .expect("exhaust: become creature + 2 counters");
    drain_stack(&mut g);
    let v = g.battlefield_find(vehicle).unwrap();
    assert!(g.computed_permanent(vehicle).unwrap().card_types.contains(&crate::card::CardType::Creature), "became an artifact creature");
    assert_eq!(v.counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2, "two +1/+1 counters");
}

/// Boommobile's exhaust ability deals X damage and grows.
#[test]
fn boommobile_exhaust_burns() {
    let mut g = two_player_game();
    let boom = g.add_card_to_battlefield(0, catalog::boommobile());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4); // {X=2}{2}{R}
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: boom, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: Vec::new(), x_value: Some(2),
    })
    .expect("exhaust: X=2 damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "2 damage killed the 2/2");
    assert_eq!(g.battlefield_find(boom).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied().unwrap_or(0), 1);
}

/// Howlsquad Heavy grants other Goblins haste.
#[test]
fn howlsquad_heavy_goblin_haste() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::howlsquad_heavy());
    let goblin = g.add_card_to_battlefield(0, catalog::mogg_fanatic()); // a Goblin
    assert!(g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::Haste),
        "other Goblins gain haste");
}

/// Boosted Sloop loots (draw then discard) whenever you attack. The trigger is
/// controller-scoped, so any attacker you declare fires it.
#[test]
fn boosted_sloop_attack_loots() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::boosted_sloop());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }]))
    .unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "loot is net-zero (draw 1, discard 1)");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"), "discarded a card");
}

/// Howler's Heavy shrinks an opponent's creature by -3/-0 when cycled.
#[test]
fn howlers_heavy_cycle_debuff() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let card = g.add_card_to_hand(0, catalog::howlers_heavy());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Cycle { card_id: card, x_value: None })
        .expect("cycle Howler's Heavy");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(foe).unwrap().power, -1, "2/2 → -1/2 after -3/-0 (only opp creature auto-targeted)");
}

/// Wreckage Wickerfolk surveils 2 on entry (one card sent to the graveyard).
#[test]
fn wreckage_wickerfolk_surveils() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let next = g.add_card_to_library(0, catalog::forest());
    // Surveil 2: keep `next` on top, bin `top` to the graveyard.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: vec![next], bottom: vec![top] },
    ]));
    let wf = g.move_card_to_battlefield_for_test(0, catalog::wreckage_wickerfolk());
    drain_stack(&mut g);
    assert!(g.computed_permanent(wf).unwrap().keywords.contains(&Keyword::Flying), "has flying");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top), "surveiled a card to the graveyard");
}

/// Transit Mage tutors a mana-value-4 artifact to hand.
#[test]
fn transit_mage_tutors_artifact() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let rock = g.add_card_to_library(0, catalog::hedron_archive()); // MV 4 — eligible
    g.add_card_to_library(0, catalog::sol_ring()); // MV 1 — ineligible
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(rock))]));
    g.move_card_to_battlefield_for_test(0, catalog::transit_mage());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == rock), "tutored the MV-4 artifact to hand");
}

/// Veteran Beastrider untaps your creatures at your end step.
#[test]
fn veteran_beastrider_untaps_at_end_step() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::veteran_beastrider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "creature untapped at your end step");
}

/// Ticket Tortoise makes a Treasure when an opponent has more lands.
#[test]
fn ticket_tortoise_treasure_when_behind_on_lands() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_battlefield(1, catalog::forest()); }
    g.move_card_to_battlefield_for_test(0, catalog::ticket_tortoise());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "made a Treasure while behind on lands");
}

/// Haunt the Network makes two Thopters and drains for your artifact count.
#[test]
fn haunt_the_network_tokens_and_drain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sol_ring()); // 1 artifact before resolution
    let spell = g.add_card_to_hand(0, catalog::haunt_the_network());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.cast_spell(spell, Some(Target::Player(1)), vec![], None, None).expect("cast");
    drain_stack(&mut g);
    let thopters = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Thopter").count();
    assert_eq!(thopters, 2, "made two Thopters");
    // After the Thopters resolve, artifacts you control = Sol Ring + 2 Thopters = 3.
    assert_eq!(g.players[1].life, 17, "opponent lost 3 (artifact count)");
    assert_eq!(g.players[0].life, 23, "you gained 3");
}
