//! Functionality tests for `catalog::sets::decks::recent246` (MKM suspect
//! payoffs, graveyard-matters enchantments, Aura value).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::{drain_stack, two_player_game, GameEvent};

fn clues(g: &crabomination::game::GameState, who: usize) -> usize {
    g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == who).count()
}

/// Rune-Brand Juggler suspects a creature on ETB, then sacrifices a suspected
/// creature to shrink a target -5/-5.
#[test]
fn rune_brand_juggler_suspects_and_sacs() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let juggler = g.add_card_to_battlefield(0, catalog::rune_brand_juggler());
    let effect = catalog::rune_brand_juggler().triggered_abilities[0].effect.clone();
    let mut ctx = EffectContext::for_trigger(juggler, 0, None, 0);
    ctx.targets = vec![Target::Permanent(mine)];
    g.resolve_effect(&effect, &ctx).unwrap();
    assert!(g.battlefield_find(mine).unwrap().suspected, "ETB suspected our creature");
    // Now activate: sacrifice the suspected creature to shrink the opponent's.
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: juggler,
        ability_index: 0,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("sac the suspected creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "suspected creature sacrificed");
    assert!(g.battlefield_find(foe).is_none(), "-5/-5 killed the 3/3");
}

/// Chalk Outline mints a Detective and investigates when a creature card leaves
/// your graveyard.
#[test]
fn chalk_outline_detective_and_investigate() {
    let mut g = two_player_game();
    let _outline = g.add_card_to_battlefield(0, catalog::chalk_outline());
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: gy }]);
    drain_stack(&mut g);
    let detectives = g.battlefield.iter().filter(|c| c.definition.name == "Detective").count();
    assert_eq!(detectives, 1, "made a Detective");
    assert_eq!(clues(&g, 0), 1, "investigated");
}

/// Soul Enervation drains when a creature card leaves your graveyard.
#[test]
fn soul_enervation_drains_on_graveyard_leave() {
    let mut g = two_player_game();
    let _ener = g.add_card_to_battlefield(0, catalog::soul_enervation());
    let life = g.players[0].life;
    let opp = g.players[1].life;
    let gy = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::CardLeftGraveyard { player: 0, card_id: gy }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1");
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
}

/// Convenient Target suspects the creature it enchants and buffs it +1/+1.
#[test]
fn convenient_target_suspects_and_buffs() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::convenient_target());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    // Fire the ETB suspect trigger.
    g.fire_self_etb_triggers(aura, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().suspected, "enchanted creature suspected");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
}

/// Curious Inquiry buffs +1/+1 and grants a combat-damage investigate trigger.
#[test]
fn curious_inquiry_buffs_and_grants_investigate() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::curious_inquiry());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1");
    // The Aura grants a combat-damage investigate trigger to the creature.
    let bonus = catalog::curious_inquiry().equipped_bonus.unwrap();
    assert_eq!(bonus.triggered_abilities.len(), 1, "grants one triggered ability");
}

/// Due Diligence grants the enchanted creature +2/+2 and vigilance.
#[test]
fn due_diligence_buffs_enchanted() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::due_diligence());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Suspected filter: the {3}{B}{R} sac ability rejects when no suspected
/// creature is available.
#[test]
fn juggler_sac_needs_a_suspected_creature() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    let _plain = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not suspected
    let juggler = g.add_card_to_battlefield(0, catalog::rune_brand_juggler());
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: juggler,
        ability_index: 0,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        x_value: None,
    });
    assert!(res.is_err(), "no suspected creature to sacrifice → activation rejected");
}
