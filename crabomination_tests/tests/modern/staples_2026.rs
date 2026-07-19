#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Modern staples batch (2026-06-11) ───────────────────────────────────────

/// Absorb counters the spell and gains 3 life.
#[test]
fn absorb_counters_and_gains_three() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    let absorb = g.add_card_to_hand(0, catalog::absorb());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: absorb, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3, took no bolt");
}

/// Render Silent locks the countered spell's controller out of casting.
#[test]
fn render_silent_counters_and_silences() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    let rs = g.add_card_to_hand(0, catalog::render_silent());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rs, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt countered");
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect_err("silenced");
    assert!(matches!(err, GameError::SilencedThisTurn), "got {err:?}");
}

/// Sphinx's Revelation gains X and draws X.
#[test]
fn sphinxs_revelation_gains_and_draws_x() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let rev = g.add_card_to_hand(0, catalog::sphinxs_revelation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23);
    assert_eq!(g.players[0].hand.len(), 3);
}

/// Cryptic Serpent's cost shrinks by {1} per instant/sorcery in your graveyard.
#[test]
fn cryptic_serpent_graveyard_cost_reduction() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_graveyard(0, catalog::lightning_bolt()); }
    let serp = g.add_card_to_hand(0, catalog::cryptic_serpent());
    // {5}{U}{U} - 5 = {U}{U}
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: serp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("UU after 5 IS cards in the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(serp).is_some());
}

/// Timely Reinforcements: behind on life and creatures → 6 life + 3 Soldiers.
#[test]
fn timely_reinforcements_both_halves() {
    let mut g = two_player_game();
    g.players[0].life = 10;
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let tr = g.add_card_to_hand(0, catalog::timely_reinforcements());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 16, "gained 6");
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 3);
}

/// Timely Reinforcements: ahead on both → nothing happens.
#[test]
fn timely_reinforcements_ahead_does_nothing() {
    let mut g = two_player_game();
    g.players[0].life = 25;
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tr = g.add_card_to_hand(0, catalog::timely_reinforcements());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: tr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 25);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Soldier"));
}

/// Dwynen's Elite makes its token only with another Elf around.
#[test]
fn dwynens_elite_token_needs_another_elf() {
    let mut g = two_player_game();
    // No other elf: no token.
    let e1 = g.add_card_to_hand(0, catalog::dwynens_elite());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: e1, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Elf Warrior"));
    // Second copy sees the first: token.
    let e2 = g.add_card_to_hand(0, catalog::dwynens_elite());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: e2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elf Warrior"));
}

/// Cruel Ultimatum: full six-part sequence.
#[test]
fn cruel_ultimatum_sequence() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..4 { g.add_card_to_hand(1, catalog::island()); }
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let cu = g.add_card_to_hand(0, catalog::cruel_ultimatum());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: cu, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(opp_bear).is_none(), "opponent sacrificed");
    assert_eq!(g.players[1].hand.len(), 1, "discarded three of four");
    assert_eq!(g.players[1].life, 15);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned");
    assert_eq!(g.players[0].hand.len(), 4, "creature + three draws");
    assert_eq!(g.players[0].life, 25);
}

/// Khalni Heart Expedition charges on landfall and fetches two tapped basics.
#[test]
fn khalni_heart_expedition_quest_and_fetch() {
    let mut g = two_player_game();
    let f1 = g.add_card_to_library(0, catalog::forest());
    let f2 = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(true), DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(f1)), DecisionAnswer::Search(Some(f2)),
    ]));
    let khe = g.add_card_to_battlefield(0, catalog::khalni_heart_expedition());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..3 {
        let land = g.add_card_to_hand(0, catalog::mountain());
        g.players[0].lands_played_this_turn = 0;
        g.perform_action(GameAction::PlayLand(land)).unwrap();
        drain_stack(&mut g);
    }
    assert_eq!(
        g.battlefield_find(khe).unwrap().counter_count(crabomination::card::CounterType::Quest),
        3
    );
    g.perform_action(GameAction::ActivateAbility { card_id: khe, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(khe).is_none(), "sacrificed");
    let forests = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Forest").count();
    assert_eq!(forests, 2, "two basics fetched");
    assert!(g.battlefield.iter().filter(|c| c.definition.name == "Forest").all(|c| c.tapped));
}

/// Goblin Engineer tutors an artifact to the graveyard, then trades an
/// artifact for it on the battlefield.
#[test]
fn goblin_engineer_entomb_and_reanimate() {
    let mut g = two_player_game();
    let clamp = g.add_card_to_library(0, catalog::skullclamp());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(clamp)),
    ]));
    let eng = g.add_card_to_hand(0, catalog::goblin_engineer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: eng, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == clamp), "clamp entombed");
    // Activate: {R}, {T}, sac an artifact → return the clamp.
    let fodder = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.battlefield_find_mut(eng).unwrap().summoning_sick = false;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: eng, ability_index: 0, target: Some(Target::Permanent(clamp)), additional_targets: Vec::new(), x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(clamp).is_some(), "clamp returned to battlefield");
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
}

/// Kor Duelist has double strike only while equipped.
#[test]
fn kor_duelist_double_strike_while_equipped() {
    let mut g = two_player_game();
    let kor = g.add_card_to_battlefield(0, catalog::kor_duelist());
    let computed = g.computed_permanent(kor).unwrap();
    assert!(!computed.keywords.contains(&Keyword::DoubleStrike));
    let boner = g.add_card_to_battlefield(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Equip { equipment: boner, target: kor }).unwrap();
    let computed = g.computed_permanent(kor).unwrap();
    assert!(computed.keywords.contains(&Keyword::DoubleStrike));
}

/// Ancient Ziggurat mana casts creatures but not other spells.
#[test]
fn ancient_ziggurat_creature_only_mana() {
    let mut g = two_player_game();
    let zig = g.add_card_to_battlefield(0, catalog::ancient_ziggurat());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility { card_id: zig, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).unwrap();
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err(),
        "restricted mana can't fund a bolt"
    );
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature spell funded by Ziggurat mana");
}

/// Unclaimed Territory's colored mana only casts creatures of the chosen type.
#[test]
fn unclaimed_territory_chosen_type_mana() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::CreatureType(crabomination::card::CreatureType::Elf),
        DecisionAnswer::Color(Color::Green),
    ]));
    let land = g.add_card_to_hand(0, catalog::unclaimed_territory());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(land)).unwrap();
    drain_stack(&mut g);
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).unwrap();
    // Bear isn't an Elf — the restricted pip won't fund it.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        }).is_err()
    );
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Elf funded by chosen-type mana");
}

/// Castle Garenbrig enters untapped with a Forest, taps for six restricted
/// green that funds a creature spell.
#[test]
fn castle_garenbrig_six_green_for_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    let castle = g.add_card_to_hand(0, catalog::castle_garenbrig());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(castle)).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(castle).unwrap().tapped, "untapped with a Forest");
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility { card_id: castle, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).unwrap();
    // Six green restricted to creature spells: cast a 6-drop creature.
    let serp = g.add_card_to_hand(0, catalog::cryptic_serpent());
    // (graveyard empty — full {5}{U}{U}); use a green fatty instead.
    let _ = serp;
    let wurm = g.add_card_to_hand(0, catalog::elder_gargaroth());
    g.perform_action(GameAction::CastSpell {
        card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("six restricted green funds a creature");
}

/// Castle Garenbrig enters tapped without a Forest.
#[test]
fn castle_garenbrig_tapped_without_forest() {
    let mut g = two_player_game();
    let castle = g.add_card_to_hand(0, catalog::castle_garenbrig());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::PlayLand(castle)).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(castle).unwrap().tapped);
}

/// CR 702.89 — umbra armor replaces destruction: the Aura dies instead and
/// the creature's damage is removed.
#[test]
fn umbra_armor_saves_enchanted_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let umbra = g.add_card_to_battlefield(0, catalog::hyena_umbra());
    g.battlefield_find_mut(umbra).unwrap().attached_to = Some(bear);
    // Lethal damage (3 ≥ 2+1 toughness with the +1/+1 bonus).
    g.battlefield_find_mut(bear).unwrap().damage = 3;
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_some(), "creature saved");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "damage removed");
    assert!(g.battlefield_find(umbra).is_none(), "umbra destroyed instead");
}

/// Umbra armor also replaces Effect::Destroy.
#[test]
fn umbra_armor_replaces_destroy_effect() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let umbra = g.add_card_to_battlefield(1, catalog::spider_umbra());
    g.battlefield_find_mut(umbra).unwrap().attached_to = Some(bear);
    let blade = g.add_card_to_hand(0, catalog::doom_blade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: blade, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "creature saved");
    assert!(g.battlefield_find(umbra).is_none(), "umbra destroyed instead");
}

/// Spirit Mantle: protection from creatures — unblockable + no combat damage.
#[test]
fn spirit_mantle_protection_from_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mantle = g.add_card_to_battlefield(0, catalog::spirit_mantle());
    g.battlefield_find_mut(mantle).unwrap().attached_to = Some(bear);
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // The mantled bear can't be blocked.
    assert!(
        !g.blocker_can_block_attacker(opp, bear),
        "pro-creatures attacker can't be blocked"
    );
    // Damage from a creature source is prevented.
    assert!(g.damage_prevented_by_protection(opp, bear));
    // Non-creature damage still lands.
    let blade = g.add_card_to_battlefield(0, catalog::bonesplitter());
    assert!(!g.damage_prevented_by_protection(blade, bear));
}

/// Daybreak Coronet needs an already-enchanted creature at cast time.
#[test]
fn daybreak_coronet_requires_enchanted_target() {
    let mut g = two_player_game();
    let bare = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let coronet = g.add_card_to_hand(0, catalog::daybreak_coronet());
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: coronet, target: Some(Target::Permanent(bare)),
            additional_targets: vec![], mode: None, x_value: None,
        }).is_err(),
        "unenchanted creature is an illegal target"
    );
    // Enchant the bear first, then the Coronet sticks.
    let umbra = g.add_card_to_battlefield(0, catalog::hyena_umbra());
    g.battlefield_find_mut(umbra).unwrap().attached_to = Some(bare);
    g.perform_action(GameAction::CastSpell {
        card_id: coronet, target: Some(Target::Permanent(bare)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("now-enchanted creature is legal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(coronet).unwrap().attached_to, Some(bare));
}

/// Kor Spiritdancer grows +2/+2 per attached Aura and draws on Aura casts.
#[test]
fn kor_spiritdancer_scales_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let dancer = g.add_card_to_battlefield(0, catalog::kor_spiritdancer());
    let cp = g.computed_permanent(dancer).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 2));
    let umbra = g.add_card_to_hand(0, catalog::hyena_umbra());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: umbra, target: Some(Target::Permanent(dancer)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "cast the Aura, drew a card");
    let cp = g.computed_permanent(dancer).unwrap();
    // +2/+2 for the Aura itself, +1/+1 from Hyena Umbra's bonus.
    assert_eq!((cp.power, cp.toughness), (3, 5));
}

/// Counterbalance counters a spell whose MV matches the library top.
#[test]
fn counterbalance_counters_matching_mv() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::counterbalance());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Top of library: a 1-MV card.
    g.add_card_to_library(0, catalog::lightning_bolt());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "bolt countered by matching MV");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));
}

/// Counterbalance whiffs on a mismatched mana value.
#[test]
fn counterbalance_misses_wrong_mv() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::counterbalance());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 on top
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 17, "bolt resolved");
}

/// Whir of Invention fetches an artifact with MV ≤ X onto the battlefield.
#[test]
fn whir_of_invention_fetches_artifact() {
    let mut g = two_player_game();
    let clamp = g.add_card_to_library(0, catalog::skullclamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(clamp))]));
    let whir = g.add_card_to_hand(0, catalog::whir_of_invention());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: whir, target: None, additional_targets: vec![], mode: None, x_value: Some(1),
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(clamp).is_some(), "artifact onto the battlefield");
}

/// Nettle Sentinel skips its untap step but untaps on a green cast.
#[test]
fn nettle_sentinel_untap_cycle() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let sentinel = g.add_card_to_battlefield(0, catalog::nettle_sentinel());
    g.battlefield_find_mut(sentinel).unwrap().tapped = true;
    g.do_untap();
    assert!(g.battlefield_find(sentinel).unwrap().tapped, "skipped untap step");
    let elf = g.add_card_to_hand(0, catalog::llanowar_elves());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(!g.battlefield_find(sentinel).unwrap().tapped, "untapped on green cast");
}

/// Inventors' Fair tutors only with three artifacts on board.
#[test]
fn inventors_fair_metalcraft_gate() {
    let mut g = two_player_game();
    let fair = g.add_card_to_battlefield(0, catalog::inventors_fair());
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: fair, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
        }).is_err(),
        "needs three artifacts"
    );
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::bonesplitter()); }
    let clamp = g.add_card_to_library(0, catalog::skullclamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(clamp))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: fair, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("metalcraft satisfied");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == clamp), "artifact tutored to hand");
    assert!(g.battlefield_find(fair).is_none(), "Fair sacrificed");
}

/// Scourge of the Skyclaves: kicked cast halves each player's life; its
/// P/T track 20 minus the highest life total.
#[test]
fn scourge_of_the_skyclaves_kicked() {
    let mut g = two_player_game();
    let scourge = g.add_card_to_hand(0, catalog::scourge_of_the_skyclaves());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: scourge, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 10, "halved");
    assert_eq!(g.players[1].life, 10, "halved");
    let cp = g.computed_permanent(scourge).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10), "20 − highest life (10)");
}

/// Voice of Resurgence mints its scaling token when it dies.
#[test]
fn voice_of_resurgence_dies_token() {
    let mut g = two_player_game();
    let voice = g.add_card_to_battlefield(0, catalog::voice_of_resurgence());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_to_graveyard_with_triggers(voice);
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Elemental")
        .expect("token minted");
    let cp = g.computed_permanent(token.id).unwrap();
    // Bear + the token itself = 2 creatures.
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Voice of Resurgence triggers when an opponent casts during your turn.
#[test]
fn voice_of_resurgence_opponent_cast_on_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::voice_of_resurgence());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0; // your turn
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Elemental"));
}

/// Meddling Mage locks the named spell out of being cast.
#[test]
fn meddling_mage_locks_named_spell() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::meddling_mage());
    g.battlefield_find_mut(mage).unwrap().named_card = Some("Lightning Bolt".into());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect_err("named spell locked");
    assert!(matches!(err, GameError::SpellNameLocked), "got {err:?}");
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("other spells still castable");
}

/// Dress Down strips creature abilities while it's on the battlefield.
#[test]
fn dress_down_strips_abilities() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let cp = g.computed_permanent(angel).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying));
    let dd = g.add_card_to_battlefield(0, catalog::dress_down());
    let cp = g.computed_permanent(angel).unwrap();
    assert!(!cp.keywords.contains(&Keyword::Flying), "abilities stripped");
    g.remove_to_graveyard_with_triggers(dd);
    let cp = g.computed_permanent(angel).unwrap();
    assert!(cp.keywords.contains(&Keyword::Flying), "restored after Dress Down leaves");
}

/// Ox of Agonas: ETB dumps the hand and draws three; escaping adds a counter.
#[test]
fn ox_of_agonas_etb_and_escape_counter() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..2 { g.add_card_to_hand(0, catalog::island()); }
    // Escape it from the graveyard: 8 other cards to exile.
    let ox = g.add_card_to_graveyard(0, catalog::ox_of_agonas());
    let fodder: Vec<_> = (0..8)
        .map(|_| g.add_card_to_graveyard(0, catalog::mountain()))
        .collect();
    g.players[0].mana_pool.add(Color::Red, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastEscape {
        card_id: ox, exile_cards: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape");
    drain_stack(&mut g);
    let ox_bf = g.battlefield_find(ox).expect("Ox escaped onto the battlefield");
    assert_eq!(ox_bf.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), 3, "hand replaced with three fresh cards");
}

/// Fractured Identity exiles the permanent and gives each opponent a copy.
#[test]
fn fractured_identity_exiles_and_copies() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    let fi = g.add_card_to_hand(0, catalog::fractured_identity());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: fi, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "original exiled");
    let copy = g.battlefield.iter().find(|c| c.definition.name == "Serra Angel")
        .expect("token copy");
    assert_eq!(copy.controller, 0, "the other player (you) got the copy");
    assert!(copy.is_token);
}

/// Gifts Ungiven: opponent splits the four — two to graveyard, two to hand.
#[test]
fn gifts_ungiven_opponent_splits() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::lightning_bolt());
    let b = g.add_card_to_library(0, catalog::grizzly_bears());
    let c = g.add_card_to_library(0, catalog::island());
    let d = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![a, b, c, d]),
        DecisionAnswer::Cards(vec![a, b]), // opponent's picks → graveyard
    ]));
    let gifts = g.add_card_to_hand(0, catalog::gifts_ungiven());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: gifts, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|x| x.id == a), "bolt to graveyard");
    assert!(g.players[0].graveyard.iter().any(|x| x.id == b), "bears to graveyard");
    assert!(g.players[0].hand.iter().any(|x| x.id == c), "island to hand");
    assert!(g.players[0].hand.iter().any(|x| x.id == d), "forest to hand");
}

/// Gifts Ungiven rejects duplicate names in the searcher's pile.
#[test]
fn gifts_ungiven_distinct_names_enforced() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::lightning_bolt());
    let b = g.add_card_to_library(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![a, b]),
        DecisionAnswer::Cards(vec![a]),
    ]));
    let gifts = g.add_card_to_hand(0, catalog::gifts_ungiven());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: gifts, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    // Only the first bolt was picked (same name); a lone pick goes to the
    // graveyard (the opponent "chooses" all of an undersized pile).
    assert!(g.players[0].graveyard.iter().any(|x| x.id == a));
    assert!(g.players[0].library.iter().any(|x| x.id == b), "duplicate stayed");
}

/// Open the Armory tutors an Aura or Equipment to hand.
#[test]
fn open_the_armory_tutors() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let umbra = g.add_card_to_library(0, catalog::hyena_umbra());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(umbra))]));
    let ota = g.add_card_to_hand(0, catalog::open_the_armory());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: ota, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == umbra));
}

/// Scuttling Doom Engine: small creatures can't block it; dies-trigger burns.
#[test]
fn scuttling_doom_engine_block_gate_and_death_burn() {
    let mut g = two_player_game();
    let engine = g.add_card_to_battlefield(0, catalog::scuttling_doom_engine());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel());
    assert!(!g.blocker_can_block_attacker(bear, engine), "power 2 can't block");
    assert!(g.blocker_can_block_attacker(angel, engine), "power 4 can");
    g.remove_to_graveyard_with_triggers(engine);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "6 damage on death");
}

/// Spellskite redirects a spell's target to itself (paying {U/P} with life).
#[test]
fn spellskite_redirects_spell_target() {
    let mut g = two_player_game();
    let skite = g.add_card_to_battlefield(0, catalog::spellskite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap();
    // Respond: no blue mana — the Phyrexian pip is paid with 2 life.
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: skite, ability_index: 0,
        target: Some(Target::Permanent(bolt)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Spellskite");
    assert_eq!(g.players[0].life, 18, "paid 2 life for {{U/P}}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear untouched");
    let skite_card = g.battlefield_find(skite).expect("0/4 survives the bolt");
    assert_eq!(skite_card.damage, 3, "bolt redirected to Spellskite");
}

/// CR 115.7 — Spellskite also redirects a targeted *ability* on the stack
/// (Prodigal Sorcerer's ping).
#[test]
fn spellskite_redirects_ability_target() {
    let mut g = two_player_game();
    let skite = g.add_card_to_battlefield(0, catalog::spellskite());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tim = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(tim);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: tim, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("ping the bear");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: skite, ability_index: 0,
        target: Some(Target::Permanent(tim)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Spellskite at the ability");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "bear untouched");
    assert_eq!(g.battlefield_find(skite).unwrap().damage, 1, "ping redirected to Spellskite");
}

// ── P3K horsemanship + conditional auras + Porphyry Nodes + Ravenous Trap ───

/// CR 702.31 — a horsemanship attacker can't be blocked by a vanilla
/// creature, but can be blocked by another horsemanship creature.
#[test]
fn cr_702_31_horsemanship_blocks_only_horsemanship() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let liu_bei = g.add_card_to_battlefield(0, catalog::liu_bei_lord_of_shu());
    g.clear_sickness(liu_bei);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let zhang = g.add_card_to_battlefield(1, catalog::zhang_fei_fierce_warrior());
    g.attacking = vec![Attack { attacker: liu_bei, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareBlockers;
    g.active_player_idx = 0;
    assert!(g.declare_blockers(vec![(bear, liu_bei)]).is_err(),
        "vanilla bear can't block horsemanship");
    g.declare_blockers(vec![(zhang, liu_bei)]).expect("horsemanship blocks horsemanship");
}

/// Sun Quan grants horsemanship to your whole team (computed keywords).
#[test]
fn sun_quan_grants_team_horsemanship() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::sun_quan_lord_of_wu());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Horsemanship));
}

/// Liu Bei reads +2/+2 while Guan Yu is on your battlefield.
#[test]
fn liu_bei_pumps_beside_guan_yu() {
    let mut g = two_player_game();
    let liu = g.add_card_to_battlefield(0, catalog::liu_bei_lord_of_shu());
    assert_eq!(g.computed_permanent(liu).unwrap().power, 2);
    let guan = g.add_card_to_battlefield(0, catalog::guan_yu_sainted_warrior());
    assert_eq!(g.computed_permanent(liu).unwrap().power, 4, "+2/+2 beside Guan Yu");
    // An opponent's Guan Yu doesn't count ("you control").
    g.battlefield_find_mut(guan).unwrap().controller = 1;
    assert_eq!(g.computed_permanent(liu).unwrap().power, 2);
}

/// Guan Yu's dies trigger may shuffle him into his owner's library.
#[test]
fn guan_yu_dies_shuffles_into_library_on_yes() {
    let mut g = two_player_game();
    let guan = g.add_card_to_battlefield(0, catalog::guan_yu_sainted_warrior());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(&Effect::Destroy { what: crabomination::card::Selector::EachPermanent(
        crabomination::card::SelectionRequirement::Creature) }, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().all(|c| c.id != guan), "not in graveyard");
    assert!(g.players[0].library.iter().any(|c| c.id == guan), "shuffled into library");
}

/// Porphyry Nodes destroys the least-power creature (no regeneration) at
/// your upkeep, and sacrifices itself once no creature is left.
#[test]
fn porphyry_nodes_destroys_least_power_then_sacrifices_itself() {
    let mut g = two_player_game();
    let nodes = g.add_card_to_battlefield(0, catalog::porphyry_nodes());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pest = g.add_card_to_battlefield(1, catalog::memnite());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(pest).is_none(), "1/1 (least power) destroyed");
    assert!(g.battlefield_find(bear).is_some(), "bigger creature survives");
    // Clear the board; the next upkeep check sacrifices the Nodes.
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::Destroy { what: crabomination::card::Selector::EachPermanent(
        crabomination::card::SelectionRequirement::Creature) }, &ctx).unwrap();
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(nodes).is_none(), "no creatures → Nodes sacrificed");
}

/// Ravenous Trap casts for {0} once an opponent had 3+ cards put into their
/// graveyard this turn, and exiles the targeted graveyard.
#[test]
fn ravenous_trap_free_after_three_cards_hit_a_graveyard() {
    let mut g = two_player_game();
    let trap = g.add_card_to_hand(0, catalog::ravenous_trap());
    // Not yet: the {0} alternative is rejected with an empty tally.
    assert!(g.perform_action(GameAction::CastSpellAlternative {
        card_id: trap, pitch_card: None, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "no cards to the graveyard yet → no free cast");
    for _ in 0..3 { g.add_card_to_library(1, catalog::forest()); }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::Mill {
        who: crabomination::card::Selector::Player(crabomination::effect::PlayerRef::EachOpponent),
        amount: crabomination::card::Value::Const(3),
    }, &ctx).unwrap();
    assert_eq!(g.players[1].cards_to_graveyard_this_turn, 3, "tally stamped");
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: trap, pitch_card: None, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("free via trap condition");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "graveyard exiled");
}

/// Shield of the Oversoul reads the host's color: a green host gets +1/+1
/// and indestructible, a white host gets +1/+1 and flying, a red host nothing.
#[test]
fn shield_of_the_oversoul_tracks_host_color() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let aura = g.add_card_to_hand(0, catalog::shield_of_the_oversoul());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, aura, Target::Permanent(bear));
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "green host gets +1/+1");
    assert!(c.keywords.contains(&Keyword::Indestructible));
    assert!(!c.keywords.contains(&Keyword::Flying), "white clause off for a green host");
}

/// Steel of the Godhead on a white-and-blue host stacks both clauses
/// (+2/+2, lifelink, unblockable).
#[test]
fn steel_of_the_godhead_stacks_both_clauses_on_wu_host() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::mantis_rider()); // WUR
    let aura = g.add_card_to_hand(0, catalog::steel_of_the_godhead());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, aura, Target::Permanent(host));
    let c = g.computed_permanent(host).unwrap();
    assert_eq!((c.power, c.toughness), (3 + 2, 3 + 2), "both +1/+1 clauses apply");
    assert!(c.keywords.contains(&Keyword::Lifelink));
    assert!(c.keywords.contains(&Keyword::Unblockable));
}

// ── Split batch 2: Dusk//Dawn, Never//Return, Turn//Burn, Hide//Seek ────────

/// Dusk sweeps power-3+ creatures; Dawn (aftermath, from the graveyard)
/// returns the small dead to hand.
#[test]
fn dusk_sweeps_big_dawn_returns_small() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::cryptic_serpent());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::dusk_dawn());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dusk");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "power 3+ destroyed");
    assert!(g.battlefield_find(bear).is_some(), "power 2 survives");
    // Dawn from the graveyard: the bear (now killed) comes back to hand.
    let ctx = EffectContext::for_spell(1, None, 0, 0);
    g.resolve_effect(&Effect::Destroy { what: crabomination::card::Selector::EachPermanent(
        crabomination::card::SelectionRequirement::Creature) }, &ctx).unwrap();
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastAftermath {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Dawn from graveyard");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "bear back to hand");
    assert!(g.exile.iter().any(|c| c.id == id), "aftermath card exiled");
}

/// Never destroys a planeswalker; Return (aftermath) exiles a graveyard card
/// and mints a 2/2 Zombie.
#[test]
fn never_kills_planeswalker_return_exiles_and_mints_zombie() {
    let mut g = two_player_game();
    let pw = g.add_card_to_battlefield(1, catalog::teferi_hero_of_dominaria());
    let id = g.add_card_to_hand(0, catalog::never_return());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast_at(&mut g, id, Target::Permanent(pw));
    assert!(g.battlefield_find(pw).is_none(), "planeswalker destroyed");
    // Return from the graveyard, exiling the dead planeswalker card.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf = g.battlefield.len();
    g.perform_action(GameAction::CastAftermath {
        card_id: id, target: Some(Target::Permanent(pw)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Return from graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == pw), "graveyard card exiled");
    assert_eq!(g.battlefield.len(), bf + 1, "Zombie minted");
}

/// Turn resets the target to a red 0/1 Weird with no abilities until end of
/// turn.
#[test]
fn turn_resets_creature_to_red_0_1_weird() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::mantis_rider());
    let id = g.add_card_to_hand(0, catalog::turn_burn());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast_at(&mut g, id, Target::Permanent(flyer));
    let c = g.computed_permanent(flyer).unwrap();
    assert_eq!((c.power, c.toughness), (0, 1), "base 0/1");
    assert!(!c.keywords.contains(&Keyword::Flying), "abilities lost");
    assert_eq!(c.colors, vec![Color::Red], "became red");
    assert!(c.subtypes.creature_types.contains(&crabomination::card::CreatureType::Weird));
}

/// Hide bottoms an artifact; Seek (right half) exiles a card from the
/// opponent's library and gains its mana value in life.
#[test]
fn hide_bottoms_artifact_seek_exiles_and_gains_mv() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::hide_seek());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast_at(&mut g, id, Target::Permanent(rock));
    assert!(g.battlefield_find(rock).is_none(), "artifact off the battlefield");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(rock), "on the bottom");

    let seek = g.add_card_to_hand(0, catalog::hide_seek());
    let fatty = g.add_card_to_library(1, catalog::cryptic_serpent()); // MV 7 printed {5}{U}{U}
    let mv = catalog::cryptic_serpent().cost.cmc() as i32;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(fatty))]));
    let before = g.players[0].life;
    g.perform_action(GameAction::CastSplitRight {
        card_id: seek, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Seek at the opponent");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == fatty), "library card exiled");
    assert_eq!(g.players[0].life, before + mv, "gained MV life");
}

/// CR 701.19a — Seek's pick routes to the caster's seat, not the searched
/// library's owner.
#[test]
fn seek_pick_routes_to_the_caster() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let seek = g.add_card_to_hand(0, catalog::hide_seek());
    let fatty = g.add_card_to_library(1, catalog::cryptic_serpent());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSplitRight {
        card_id: seek, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Seek");
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("resolve");
    let pd = g.pending_decision.as_ref().expect("search pick is pending");
    assert_eq!(pd.acting_player(), 0, "the caster picks");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Search(Some(fatty))))
        .expect("submit the pick");
    assert!(g.exile.iter().any(|c| c.id == fatty), "picked card exiled from the opponent's library");
}

// ── Rhystic riders, Ad Nauseam, faithful Wrench Mind, MV≤X targeting ────────

/// Esper Sentinel: an opponent's first noncreature spell each turn draws you
/// a card unless they pay {X} = its power.
#[test]
fn esper_sentinel_taxes_first_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::esper_sentinel());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    g.players[1].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opponent casts Bolt");
    drain_stack(&mut g);
    // AutoDecider declines to pay → controller draws.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew off the rhystic tax");
}

/// The opponent paying the {X} tax denies the draw.
#[test]
fn esper_sentinel_paid_tax_denies_the_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::esper_sentinel());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    let mtn = g.add_card_to_battlefield(1, catalog::mountain());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("opponent casts Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "paid one generic — no draw");
    assert!(g.battlefield_find(mtn).unwrap().tapped, "auto-tapped the Mountain");
}

/// Mystic Remora draws off every opponent noncreature spell ({4} unpaid).
#[test]
fn mystic_remora_draws_off_noncreature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mystic_remora());
    g.add_card_to_library(0, catalog::forest());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::Green, 2);
    let before = g.players[0].hand.len();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("creature spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "creature spell — no Remora draw");
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("noncreature spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "noncreature spell — Remora draw");
}

/// Ad Nauseam keeps revealing while the caster says yes, charging MV in life.
#[test]
fn ad_nauseam_reveals_until_stopped() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt()); // MV 1, deeper
    let id = g.add_card_to_hand(0, catalog::ad_nauseam());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    // Yes (bears), yes (bolt is now top? order: add_to_library puts at ...)
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(false),
    ]));
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ad Nauseam");
    drain_stack(&mut g);
    let took = [bears, bolt].iter().filter(|c| g.players[0].hand.iter().any(|h| h.id == **c)).count();
    assert_eq!(took, 1, "took exactly one card before stopping");
    let mv = if g.players[0].hand.iter().any(|h| h.id == bears) { 2 } else { 1 };
    assert_eq!(g.players[0].life, life - mv, "lost the card's mana value in life");
}

/// Wrench Mind discards the artifact when the target has one; two cards
/// otherwise.
#[test]
fn wrench_mind_artifact_escape_hatch() {
    let mut g = two_player_game();
    let rock = g.add_card_to_hand(1, catalog::mind_stone());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::wrench_mind());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 2, "only the artifact was discarded");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == rock));

    let id2 = g.add_card_to_hand(0, catalog::wrench_mind());
    g.players[0].mana_pool.add(Color::Black, 2);
    cast_at(&mut g, id2, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 0, "no artifact → discards two");
}

/// Confront the Past mode 0 rejects a graveyard planeswalker with MV > X.
#[test]
fn confront_the_past_enforces_mv_at_most_x() {
    let mut g = two_player_game();
    let pw_id = {
        let id = g.add_card_to_hand(0, catalog::teferi_hero_of_dominaria()); // MV 5
        let card = g.players[0].remove_from_hand(id).unwrap();
        g.players[0].graveyard.push(card);
        id
    };
    let id = g.add_card_to_hand(0, catalog::confront_the_past());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(pw_id)), additional_targets: vec![],
        mode: Some(0), x_value: Some(3),
    }).is_err(), "MV 5 walker is not a legal X=3 target");
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(pw_id)), additional_targets: vec![],
        mode: Some(0), x_value: Some(5),
    }).expect("X=5 covers the MV-5 walker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(pw_id).is_some(), "reanimated");
}

// ── Kataki (granted-trigger static) + Alpine Moon (named-land hate) ─────────

/// Kataki grants every artifact an upkeep sac-tax: unpaid → sacrificed,
/// paid {1} (auto-tapped) → survives.
#[test]
fn kataki_taxes_artifacts_each_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::kataki_wars_wage());
    let rock = g.add_card_to_battlefield(0, catalog::mind_stone());
    g.active_player_idx = 0;
    // AutoDecider declines the {1} → the artifact is sacrificed.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "unpaid tax → sacrificed");
    // With a land and a willing payer it survives.
    let rock2 = g.add_card_to_battlefield(0, catalog::mind_stone());
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock2).is_some(), "paid tax → survives");
    assert!(
        g.battlefield_find(mtn).unwrap().tapped || g.battlefield_find(rock2).unwrap().tapped,
        "a mana source was auto-tapped for the tax"
    );
}

/// Kataki's tax doesn't touch non-artifacts.
#[test]
fn kataki_ignores_nonartifacts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::kataki_wars_wage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// Alpine Moon strips the named opponent land's abilities (its printed mana
/// ability is gone) and grants "{T}: any color" instead.
#[test]
fn alpine_moon_neutralizes_named_land() {
    let mut g = two_player_game();
    let post = g.add_card_to_battlefield(1, catalog::cloudpost());
    let moon = g.add_card_to_hand(0, catalog::alpine_moon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Cloudpost".into())]));
    g.perform_action(GameAction::CastSpell {
        card_id: moon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Alpine Moon");
    drain_stack(&mut g);
    let computed = g.computed_permanent(post).unwrap();
    assert!(computed.lost_all_abilities, "printed abilities stripped");
    assert!(computed.subtypes.land_types.is_empty(), "land types stripped");
    // The granted "{T}: any color" ability (index 1) is the real replacement;
    // Cloudpost's own "{C} per Locus" ability (index 0) now counts zero Loci
    // since its Locus type was stripped (CR 613.2, computed subtypes).
    g.priority.player_with_priority = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: post, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("granted any-color ability");
    assert_eq!(g.players[1].mana_pool.amount(Color::Green), 1, "made one mana of the chosen color");
}

// ── AKH embalm pair, Bring to Light, Conspicuous Snoop ──────────────────────

/// Heart-Piercer Manticore's ETB may sacrifice another creature to fling
/// its power at a target.
#[test]
fn heart_piercer_manticore_flings_sacrificed_power() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::cryptic_serpent());
    let power = g.battlefield_find(fodder).unwrap().power();
    let id = g.add_card_to_hand(0, catalog::heart_piercer_manticore());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Manticore at the opponent");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[1].life, before - power, "damage = sacrificed power");
}

/// Aven Wind Guide grants flying + vigilance to creature tokens only.
#[test]
fn aven_wind_guide_buffs_tokens_only() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aven_wind_guide());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::midnight_haunting());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("make Spirit tokens");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.is_token).expect("token");
    let computed = g.computed_permanent(token.id).unwrap();
    assert!(computed.keywords.contains(&Keyword::Vigilance), "token gains vigilance");
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance),
        "nontoken bear untouched");
}

/// Bring to Light's converge gate: the searchable MV tracks the number of
/// colors spent on the cast.
#[test]
fn bring_to_light_converge_caps_search_mv() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    let big = g.add_card_to_library(0, catalog::cryptic_serpent()); // MV 7
    let cheap = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let id = g.add_card_to_hand(0, catalog::bring_to_light());
    // Two colors spent (G,U + 3 generic from colorless) → converge 2.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(big))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bring to Light");
    drain_stack(&mut g);
    // The MV-7 pick was ineligible (converge 2); the bear is the only legal
    // find — auto-fallback or a scripted miss must not fetch the serpent.
    assert!(g.players[0].library.iter().any(|c| c.id == big), "MV 7 stays put");
    let _ = cheap;
}

/// Conspicuous Snoop borrows the top Goblin's activated ability and loses
/// it when the top card isn't a Goblin.
#[test]
fn conspicuous_snoop_shares_top_goblin_ability() {
    let mut g = two_player_game();
    g.players[0].library.clear();
    g.add_card_to_library(0, catalog::skirk_prospector());
    let snoop = g.add_card_to_battlefield(0, catalog::conspicuous_snoop());
    let fodder = g.add_card_to_battlefield(0, catalog::skirk_prospector());
    // Snoop index 0 = the borrowed "sac a Goblin: add {R}" (it has no
    // printed activated abilities).
    g.perform_action(GameAction::ActivateAbility {
        card_id: snoop, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("borrowed Prospector ability");
    assert_eq!(g.players[0].mana_pool.total(), 1, "made one red via the borrowed ability");
    assert!(g.battlefield_find(fodder).is_none(), "a Goblin was sacrificed as the cost");
    // Non-Goblin top → no borrowed ability.
    let fid = g.next_id();
    let forest = crabomination::card::CardInstance::new(fid, catalog::forest(), 0);
    g.players[0].library.insert(0, forest);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: snoop, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "no Goblin on top → no ability");
}

// ── Fanatic of Rhonas (faithful) + Cao Cao ──────────────────────────────────

/// Fanatic of Rhonas's Ferocious mana ability needs a power-4 creature.
#[test]
fn fanatic_of_rhonas_ferocious_gates_big_mana() {
    let mut g = two_player_game();
    let snake = g.add_card_to_battlefield(0, catalog::fanatic_of_rhonas());
    g.clear_sickness(snake);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: snake, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "no power-4 creature → Ferocious off");
    g.add_card_to_battlefield(0, catalog::cryptic_serpent());
    g.battlefield_find_mut(snake).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: snake, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Ferocious on with a 6-power creature");
    assert_eq!(g.players[0].mana_pool.total(), 4, "added four green");
}

/// Cao Cao taps for a two-card opponent discard, but only precombat on
/// your own turn.
#[test]
fn cao_cao_discards_two_precombat_only() {
    let mut g = two_player_game();
    let cao = g.add_card_to_battlefield(0, catalog::cao_cao_lord_of_wei());
    g.clear_sickness(cao);
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    g.step = TurnStep::PostCombatMain;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: cao, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).is_err(), "after combat → can't activate");
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cao, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("precombat on your turn");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "opponent discarded two");
}

/// `TokenDefinition.static_abilities` survives serde — a snapshotted Voice
/// of Resurgence still mints its CDA-scaled Elemental after restore.
#[test]
fn token_static_abilities_survive_serde() {
    use crabomination::effect::Effect;
    let def = catalog::voice_of_resurgence();
    let json = serde_json::to_string(&def).unwrap();
    let restored: crabomination::card::CardDefinition = serde_json::from_str(&json).unwrap();
    let token_static = |d: &crabomination::card::CardDefinition| {
        d.triggered_abilities.iter().any(|t| match &t.effect {
            Effect::CreateToken { definition, .. } => !definition.static_abilities.is_empty(),
            _ => false,
        })
    };
    assert!(token_static(&def), "factory carries the token static");
    assert!(token_static(&restored), "token static survives the wire");
}

// ── Split second batch (CR 702.61) ───────────────────────────────────────────

/// Sudden Death shrinks a creature -4/-4 for the turn.
#[test]
fn sudden_death_shrinks_for_turn() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::cryptic_serpent());
    let s = g.add_card_to_hand(0, catalog::sudden_death());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(big).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1), "6/5 at -4/-4");
}

/// Wipe Away bounces any permanent — lands included.
#[test]
fn wipe_away_bounces_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    let s = g.add_card_to_hand(0, catalog::wipe_away());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: s, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none());
    assert!(g.players[1].hand.iter().any(|c| c.id == land));
}

/// Trickbind counters an activated ability on the stack.
#[test]
fn trickbind_counters_activated_ability() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.clear_sickness(stone);
    g.players[1].mana_pool.add_colorless(1);
    g.add_card_to_library(1, catalog::island());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate draw ability");
    let ability_on_stack = matches!(g.stack.last(),
        Some(crabomination::game::types::StackItem::Trigger { source, .. }) if *source == stone);
    assert!(ability_on_stack, "draw ability uses the stack");
    g.priority.player_with_priority = 0;
    let t = g.add_card_to_hand(0, catalog::trickbind());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: t, target: Some(Target::Permanent(stone)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Trickbind");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before, "draw countered");
}

// ── LTR landcyclers + Typecycling (CR 702.29e) ───────────────────────────────

/// Swampcycling Troll of Khazad-dûm discards it and fetches a Swamp to hand.
#[test]
fn troll_of_khazad_dum_swampcycles() {
    let mut g = two_player_game();
    let troll = g.add_card_to_hand(0, catalog::troll_of_khazad_dum());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::swamp());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Landcycle { card_id: troll }).expect("swampcycle");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == troll), "discarded");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Swamp"), "fetched a Swamp");
}

/// Basic landcycling (Typecycling) on Ash Barrens fetches any basic, and the
/// land itself taps for {C} when played.
#[test]
fn ash_barrens_basic_landcycles() {
    let mut g = two_player_game();
    let barrens = g.add_card_to_hand(0, catalog::ash_barrens());
    g.add_card_to_library(0, catalog::watery_grave());
    g.add_card_to_library(0, catalog::mountain());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Landcycle { card_id: barrens }).expect("basic landcycle");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Mountain"),
        "fetched a basic, skipping the nonbasic dual");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == barrens));
}

/// Lorien Revealed draws three when cast normally.
#[test]
fn lorien_revealed_draws_three() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let s = g.add_card_to_hand(0, catalog::lorien_revealed());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, s);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3);
}

/// Eagles of the North pumps the team +1/+0 with first strike on ETB.
#[test]
fn eagles_of_the_north_etb_team_pump() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let eagles = g.add_card_to_hand(0, catalog::eagles_of_the_north());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, eagles);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

// ── Theros gods batch (CR 700.5 devotion) ────────────────────────────────────

/// Heliod grants other creatures vigilance and mints 2/1 Cleric tokens.
#[test]
fn heliod_vigilance_anthem_and_cleric_token() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_god_of_the_sun());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance));
    assert!(!g.computed_permanent(heliod).unwrap().keywords.contains(&Keyword::Vigilance),
        "\"other creatures\" excludes Heliod");
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: heliod, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mint Cleric");
    drain_stack(&mut g);
    let cleric = g.battlefield.iter().find(|c| c.definition.name == "Cleric").expect("token");
    assert_eq!((cleric.power(), cleric.toughness()), (2, 1));
    assert!(cleric.definition.card_types.contains(&CardType::Enchantment));
}

/// Heliod isn't a creature below five white devotion; is at five.
#[test]
fn heliod_devotion_gate() {
    let mut g = two_player_game();
    let heliod = g.add_card_to_battlefield(0, catalog::heliod_god_of_the_sun());
    assert!(!g.computed_permanent(heliod).unwrap().card_types.contains(&CardType::Creature));
    // Heliod itself is {3}{W} = 1 white pip; add four more.
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::yoked_ox()); }
    assert!(g.computed_permanent(heliod).unwrap().card_types.contains(&CardType::Creature));
}

/// Purphoros pings each opponent for 2 when another creature enters.
#[test]
fn purphoros_pings_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::purphoros_god_of_the_forge());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "2 damage on the bear's ETB");
}

/// Xenagos gives another creature haste and doubles its power at combat.
#[test]
fn xenagos_combat_trigger_doubles_power() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::xenagos_god_of_revels());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+X/+0 where X is its power");
    assert_eq!(cp.toughness, 2);
    assert!(cp.keywords.contains(&Keyword::Haste));
}

/// Phenax grants creatures a tap-to-mill-by-toughness ability.
#[test]
fn phenax_grants_mill_by_toughness() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::phenax_god_of_deception());
    let ox = g.add_card_to_battlefield(0, catalog::yoked_ox());
    g.clear_sickness(ox);
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let printed = catalog::yoked_ox().activated_abilities.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ox, ability_index: printed, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("granted mill ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 4, "milled X = Yoked Ox's toughness");
    assert!(g.battlefield_find(ox).unwrap().tapped);
}

/// Pharika exiles a graveyard creature and gives its owner a Snake.
#[test]
fn pharika_exiles_and_mints_snake_for_owner() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let pharika = g.add_card_to_battlefield(0, catalog::pharika_god_of_affliction());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pharika, ability_index: 0, target: Some(Target::Permanent(dead)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Pharika");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == dead), "creature card exiled");
    let snake = g.battlefield.iter().find(|c| c.definition.name == "Snake").expect("token");
    assert_eq!(snake.controller, 1, "the exiled card's owner gets the Snake");
    assert!(snake.definition.keywords.contains(&Keyword::Deathtouch));
}

/// Karametra fetches a Forest or Plains tapped when you cast a creature.
#[test]
fn karametra_fetches_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::karametra_god_of_harvests());
    g.add_card_to_library(0, catalog::island());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(forest)),
    ]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bear);
    drain_stack(&mut g);
    let forest = g.battlefield.iter().find(|c| c.definition.name == "Forest").expect("fetched");
    assert!(forest.tapped, "enters tapped");
}

/// Mogis makes each opponent sacrifice a creature (or take 2) at their upkeep.
#[test]
fn mogis_upkeep_punisher() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mogis_god_of_slaughter());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "sacrificed to Mogis");
    // With no creature, they take 2 instead.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Athreos returns your dying creatures unless an opponent pays 3 life.
#[test]
fn athreos_returns_creature_unless_opponent_pays() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::athreos_god_of_passage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Kill it with a burn spell so the death event dispatches the watcher;
    // AutoDecider declines to pay → the creature comes back.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "returned to hand");
    assert_eq!(g.players[1].life, 20, "opponent declined the 3-life payment");
}

/// Iroas grants menace and prevents all damage to your attacking creatures.
#[test]
fn iroas_menace_and_attacker_shield() {
    use crabomination::card::Keyword;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::iroas_god_of_victory());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Menace));
    g.attacking = vec![Attack { attacker: bear, target: AttackTarget::Player(1) }];
    // Spell damage to the attacking bear is prevented.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "damage prevented — bear lives");
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0);
}

/// Kruphix keeps unspent mana as colorless across steps.
#[test]
fn kruphix_unspent_mana_becomes_colorless() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kruphix_god_of_horizons());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[1].mana_pool.add(Color::Red, 1);
    g.empty_mana_pools();
    assert_eq!(g.players[0].mana_pool.total(), 2, "kept as colorless");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
    assert_eq!(g.players[1].mana_pool.total(), 0, "opponent's pool empties");
}

/// Ephara draws at upkeep only if another creature entered last turn.
#[test]
fn ephara_draws_after_a_creature_turn() {
    let mut g = two_player_game();
    let eph = g.add_card_to_battlefield(0, catalog::ephara_god_of_the_polis());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    // No creature entered last turn → no draw.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "no entry last turn — no draw");
    // A bear entered this turn; only Ephara herself doesn't count.
    g.players[0].creatures_entered_last_turn = vec![eph];
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "Ephara alone doesn't count");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].creatures_entered_last_turn = vec![bear];
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "draws after a creature turn");
}

/// Keranos: first draw on your turn — land draws a card, nonland bolts.
#[test]
fn keranos_first_draw_branches() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::keranos_god_of_storms());
    g.active_player_idx = 0;
    // Nonland first draw → 3 damage (auto-target picks the opponent).
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].cards_drawn_this_turn = 0;
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "nonland reveal bolts");
    // Land first draw → extra card.
    // `add_card_to_library` appends to the bottom — island first so it's on top.
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].cards_drawn_this_turn = 0;
    let hand = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.draw_one(0, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 2, "land reveal draws another");
    assert_eq!(g.players[1].life, 17, "the second (non-first) draw doesn't retrigger");
}

// ── Cast locks + channel discount + THB gods ─────────────────────────────────

/// Silence stops opponents (but not you) from casting this turn.
#[test]
fn silence_locks_opponents_only() {
    let mut g = two_player_game();
    let s = g.add_card_to_hand(0, catalog::silence());
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, s);
    drain_stack(&mut g);
    let opp = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: opp, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).unwrap_err();
    assert_eq!(err, GameError::SilencedThisTurn);
    let own = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, own, Target::Player(1));
}

/// Kicked Orim's Chant also stops creatures from attacking this turn.
#[test]
fn orims_chant_kicked_stops_attacks() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let chant = g.add_card_to_hand(0, catalog::orims_chant());
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: chant, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("kicked chant");
    drain_stack(&mut g);
    assert!(g.players[1].silenced_this_turn);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::DeclareAttackers;
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(0),
    }]));
    assert!(err.is_err(), "creatures can't attack under kicked Chant");
}

/// Channel abilities cost {1} less per legendary creature you control.
#[test]
fn channel_land_legendary_discount() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Two legendary creatures: Otawara's {3}{U} channel becomes {1}{U}.
    // (A devotion-gated god wouldn't count — it isn't a creature yet.)
    g.add_card_to_battlefield(0, catalog::ragavan_nimble_pilferer());
    g.add_card_to_battlefield(0, catalog::cao_cao_lord_of_wei());
    let ota = g.add_card_to_hand(0, catalog::otawara_soaring_city());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ota, ability_index: 1, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("discounted channel");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced");
}

/// Thassa, Deep-Dwelling flickers another creature at your end step.
#[test]
fn thassa_deep_dwelling_end_step_flicker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thassa_deep_dwelling());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).expect("returned");
    assert_eq!(b.damage, 0, "fresh object after the flicker");
}

/// Erebos, Bleak-Hearted converts dying creatures into cards for 2 life.
#[test]
fn erebos_bleak_hearted_pays_life_to_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::erebos_bleak_hearted());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::swamp());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "paid 2 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew");
}

/// Purphoros, Bronze-Blooded hastes the team and sneaks in a red creature.
#[test]
fn purphoros_bronze_blooded_sneak() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let pur = g.add_card_to_battlefield(0, catalog::purphoros_bronze_blooded());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![dragon]),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: pur, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sneak");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_some(), "dragon snuck in");
}

/// Nylea, Keen-Eyed discounts creatures and digs for them.
#[test]
fn nylea_keen_eyed_discount_and_dig() {
    let mut g = two_player_game();
    let nyl = g.add_card_to_battlefield(0, catalog::nylea_keen_eyed());
    // Discount: Grizzly Bears for just {G}.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, bear);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "cast for {{G}} with the discount");
    // Dig: reveal a creature on top → to hand.
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: nyl, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("dig");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "creature revealed → hand");
}

// ── AFR creature lands + Klothys/Emry/Claw/Bond/Swords ───────────────────────

/// Hall of Storm Giants animates into a 7/7 warded Giant; AFR lands enter
/// tapped only with two or more other lands.
#[test]
fn hall_of_storm_giants_animates() {
    use crabomination::card::{CreatureType, Keyword, WardCost};
    let mut g = two_player_game();
    let hall = g.add_card_to_battlefield(0, catalog::hall_of_storm_giants());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hall, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(hall).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7));
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(cp.card_types.contains(&CardType::Land), "still a land");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Giant));
    assert!(cp.keywords.contains(&Keyword::Ward(WardCost::generic(3))));
}

/// Lair of the Hydra animates into an X/X.
#[test]
fn lair_of_the_hydra_x_animate() {
    let mut g = two_player_game();
    let lair = g.add_card_to_battlefield(0, catalog::lair_of_the_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lair, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: Some(4),
    }).expect("animate for X=4");
    drain_stack(&mut g);
    let cp = g.computed_permanent(lair).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Klothys exiles a graveyard land for mana, a nonland for drain.
#[test]
fn klothys_main_phase_exile_branches() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::klothys_god_of_destiny());
    let land = g.add_card_to_graveyard(1, catalog::mountain());
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(land)),
    ]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == land));
    assert_eq!(g.players[0].mana_pool.total(), 1, "land branch adds a mana");
    // Nonland branch drains.
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22);
}

/// Emry mills four on ETB and lets you cast an artifact from your graveyard.
#[test]
fn emry_mills_and_recasts_artifacts() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let emry = g.add_card_to_hand(0, catalog::emry_lurker_of_the_loch());
    // Affinity: one artifact → {1}{U}.
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, emry);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 4, "milled four");
    let bauble = g.add_card_to_graveyard(0, catalog::mishras_bauble());
    g.clear_sickness(emry);
    g.perform_action(GameAction::ActivateAbility {
        card_id: emry, ability_index: 0, target: Some(Target::Permanent(bauble)), additional_targets: Vec::new(), x_value: None,
    }).expect("grant may-cast");
    drain_stack(&mut g);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bauble, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the Bauble from the graveyard");
}

/// Dragon's Claw offers a life on each red spell.
#[test]
fn dragons_claw_gains_on_red_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dragons_claw());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20 - 3 + 1, "took the bolt, gained 1");
}

/// Sanguine Bond converts your life gain into opponent life loss.
#[test]
fn sanguine_bond_drains_on_gain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sanguine_bond());
    let mut evs = Vec::new();
    let applied = g.adjust_life_applied(0, 3);
    evs.push(GameEvent::LifeGained { player: 0, amount: applied as u32 });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23);
    assert_eq!(g.players[1].life, 17, "opponent lost that much");
}

/// Sword of Sinew and Steel snipes an artifact on connect.
#[test]
fn sword_of_sinew_and_steel_destroys_artifact() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_sinew_and_steel());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(stone)),
    ]));
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear, target: crabomination::game::types::AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
        if g.step == TurnStep::PostCombatMain { break; }
    }
    assert!(g.battlefield_find(stone).is_none(), "artifact destroyed");
}

/// Sword of Hearth and Home flickers your creature and fetches a basic.
#[test]
fn sword_of_hearth_and_home_flicker_and_fetch() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_hearth_and_home());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let plains = g.add_card_to_library(0, catalog::plains());
    // The trigger auto-targets the only creature (the bear) for the flicker.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Search(Some(plains)),
    ]));
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![crabomination::game::types::Attack {
        attacker: bear, target: crabomination::game::types::AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
        if g.step == TurnStep::PostCombatMain { break; }
    }
    assert!(g.battlefield_find(bear).is_some(), "flickered back onto the battlefield");
    let fetched = g.battlefield_find(plains).expect("basic fetched");
    assert!(fetched.tapped);
}

/// Setessan Champion grows and draws on your enchantment ETBs.
#[test]
fn setessan_champion_constellation() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::setessan_champion());
    g.add_card_to_library(0, catalog::forest());
    let ench = g.add_card_to_hand(0, catalog::treacherous_blessing());
    for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, ench);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(champ).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1
    );
}

/// Woe Strider: Goat on ETB, sac-to-scry, and Escape from the graveyard.
#[test]
fn woe_strider_goat_and_escape() {
    let mut g = two_player_game();
    let strider = g.add_card_to_hand(0, catalog::woe_strider());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, strider);
    drain_stack(&mut g);
    let goat = g.battlefield.iter().find(|c| c.definition.name == "Goat").expect("token");
    assert_eq!((goat.power(), goat.toughness()), (0, 1));
    // Sac the goat to scry.
    let goat_id = goat.id;
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] },
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: strider, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac to scry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(goat_id).is_none(), "goat sacrificed");
    // Escape: bin it plus four other cards, recast from the graveyard.
    g.remove_to_graveyard_with_triggers(strider);
    drain_stack(&mut g);
    let fodder: Vec<_> = (0..4).map(|_| g.add_card_to_graveyard(0, catalog::forest())).collect();
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastEscape {
        card_id: strider, exile_cards: fodder,
        target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape");
    drain_stack(&mut g);
    assert!(g.battlefield_find(strider).is_some(), "escaped onto the battlefield");
}

/// Treacherous Blessing draws three, drains on casts, dies to targeting.
#[test]
fn treacherous_blessing_lifecycle() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::swamp()); }
    let tb = g.add_card_to_hand(0, catalog::treacherous_blessing());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len() - 1;
    cast(&mut g, tb);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 3, "drew three");
    // A later cast costs 1 life.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "lost 1 on the cast");
    // Targeting it makes it sacrifice itself.
    let disenchant = g.add_card_to_hand(1, catalog::disenchant());
    g.players[1].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: disenchant, target: Some(Target::Permanent(tb)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("target it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(tb).is_none());
}

/// Anax's power tracks red devotion, and dying nontoken creatures leave Satyrs.
#[test]
fn anax_devotion_power_and_satyrs() {
    let mut g = two_player_game();
    let anax = g.add_card_to_battlefield(0, catalog::anax_hardened_in_the_forge());
    // Anax alone: {1}{R}{R} = 2 red pips of devotion.
    assert_eq!(g.computed_permanent(anax).unwrap().power, 2);
    g.add_card_to_battlefield(0, catalog::purphoros_bronze_blooded());
    assert_eq!(g.computed_permanent(anax).unwrap().power, 3, "+1 from Purphoros' {{R}}");
    // A dying 6-power nontoken creature leaves two Satyrs.
    let dragon = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.battlefield_find_mut(dragon).unwrap().damage = 3;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(dragon)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("finish the dragon");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Satyr").count(),
        2,
        "power ≥ 4 → two Satyrs"
    );
}

/// Destiny Spinner protects your creature spells from counters and animates lands.
#[test]
fn destiny_spinner_uncounterable_and_animate() {
    let mut g = two_player_game();
    let spinner = g.add_card_to_battlefield(0, catalog::destiny_spinner());
    // Opponent's Counterspell fizzles against your creature spell.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    let cs = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: cs, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("counter attempt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "creature spell can't be countered");
    // Animate a land: X = enchantments you control (Spinner itself = 1).
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spinner, ability_index: 0, target: Some(Target::Permanent(forest)), additional_targets: Vec::new(), x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(forest).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    assert!(cp.card_types.contains(&CardType::Creature));
}

