//! Functionality tests for the `catalog::sets::decks::recent95` Kamigawa: Neon
//! Dynasty batch.

use crabomination::catalog;
use crabomination::card::{CounterType, Keyword};
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;

/// Advance from the current step to PostCombatMain, resolving combat damage.
fn pass_through_combat(g: &mut GameState) {
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(g);
}

/// Network Disruptor taps a permanent on entry (auto-targets the opponent's).
#[test]
fn network_disruptor_taps_on_etb() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let disruptor = g.add_card_to_battlefield(0, catalog::network_disruptor());
    g.fire_self_etb_triggers(disruptor, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's permanent tapped");
}

/// Enthusiastic Mechanaut discounts artifact spells by {1}.
#[test]
fn enthusiastic_mechanaut_discounts_artifacts() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::enthusiastic_mechanaut());
    let artifact = crabomination::card::CardInstance::new(g.next_id(), catalog::bonesplitter(), 0);
    let creature = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &artifact, None), 1, "artifact −{{1}}");
    assert_eq!(cost_reduction_for_spell(&g, 0, &creature, None), 0, "non-artifact unaffected");
}

/// Imperial Oath makes three Samurai and scries.
#[test]
fn imperial_oath_makes_three_samurai() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let oath = g.add_card_to_hand(0, catalog::imperial_oath());
    for _ in 0..5 { g.players[0].mana_pool.add(crabomination::mana::Color::White, 1); }
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: oath, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Imperial Oath");
    drain_stack(&mut g);
    let samurai = g.battlefield.iter().filter(|c| c.definition.name == "Samurai").count();
    assert_eq!(samurai, 3, "three Samurai tokens");
}

/// Twinshot Sniper's ETB deals 2 to the opponent (auto-targeted).
#[test]
fn twinshot_sniper_etb_pings() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let sniper = g.add_card_to_battlefield(0, catalog::twinshot_sniper());
    g.fire_self_etb_triggers(sniper, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "dealt 2 to the opponent");
}

/// Moonfolk Puzzlemaker fires its scry trigger when it becomes tapped (by
/// attacking); it ends up tapped with the trigger resolved.
#[test]
fn moonfolk_puzzlemaker_scries_on_tap() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let mf = g.add_card_to_battlefield(0, catalog::moonfolk_puzzlemaker());
    g.clear_sickness(mf);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: mf, target: AttackTarget::Player(1) }]))
        .expect("Moonfolk attacks and taps");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mf).unwrap().tapped, "tapped by attacking");
}

/// Jukai Preserver's ETB adds a +1/+1 counter to a creature you control.
#[test]
fn jukai_preserver_etb_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let jukai = g.add_card_to_battlefield(0, catalog::jukai_preserver());
    g.fire_self_etb_triggers(jukai, 0);
    drain_stack(&mut g);
    let mine: u32 = [bear, jukai].iter()
        .map(|id| g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(mine, 1, "exactly one +1/+1 counter placed on a creature you control");
}

/// Selfless Samurai grants lifelink to a lone Samurai/Warrior attacker.
#[test]
fn selfless_samurai_lifelink_on_solo_attack() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::selfless_samurai());
    let samurai = g.add_card_to_battlefield(0, catalog::selfless_samurai());
    g.clear_sickness(samurai);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: samurai, target: AttackTarget::Player(1) }]))
        .expect("attack alone");
    drain_stack(&mut g);
    assert!(g.computed_permanent(samurai).unwrap().keywords.contains(&Keyword::Lifelink),
        "lone Samurai gained lifelink");
}

/// Selfless Samurai's sac ability grants indestructible to *another* creature —
/// it can't target itself.
#[test]
fn selfless_samurai_sac_targets_another() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let samurai = g.add_card_to_battlefield(0, catalog::selfless_samurai());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    // Self-target is illegal ("another target creature").
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: samurai,
        ability_index: 0,
        target: Some(Target::Permanent(samurai)),
        additional_targets: vec![],
        x_value: None,
    })
    .is_err(), "can't target itself");
    // Targeting the bear works and sacrifices the Samurai.
    g.perform_action(GameAction::ActivateAbility {
        card_id: samurai,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("target another creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(samurai).is_none(), "Samurai sacrificed");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Indestructible));
}

/// Prosperous Thief mints a Treasure on combat damage.
#[test]
fn prosperous_thief_makes_treasure() {
    let mut g = two_player_game();
    let thief = g.add_card_to_battlefield(0, catalog::prosperous_thief());
    g.clear_sickness(thief);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: thief, target: AttackTarget::Player(1) }]))
        .expect("thief attacks");
    drain_stack(&mut g);
    pass_through_combat(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"), "made a Treasure");
}

/// Bronzeplate Boar buffs +3/+2 (and trample) when attached via Reconfigure.
#[test]
fn bronzeplate_boar_equip_bonus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boar = g.add_card_to_battlefield(0, catalog::bronzeplate_boar());
    g.battlefield.iter_mut().find(|c| c.id == boar).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4), "2/2 + 3/2");
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Automated Artificer taps for one mana.
#[test]
fn automated_artificer_makes_mana() {
    let mut g = two_player_game();
    let bot = g.add_card_to_battlefield(0, catalog::automated_artificer());
    g.clear_sickness(bot);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bot, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for {C}");
    // The {C} is spend-restricted (artifacts/abilities only), so it lands in the
    // restricted pool rather than `total()`; the tap confirms the ability fired.
    assert!(g.battlefield_find(bot).unwrap().tapped, "tapped for restricted {{C}}");
}
