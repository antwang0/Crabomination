//! Functionality tests for `catalog::sets::decks::recent68`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::effect::{Effect, EventKind, Selector};
use crabomination::game::two_player_game;
use crabomination::game::*;

fn resolve_spell(g: &mut GameState, def: crabomination::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    g.resolve_effect(&def.effect, &ctx).unwrap();
}

fn activate(g: &mut GameState, id: CardId, idx: usize, target: Option<Target>) {
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: idx, target, additional_targets: Vec::new(), x_value: None,
    }).expect("ability activates");
    drain_stack(g);
}

#[test]
fn chrome_steed_metalcraft_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::chrome_steed());
    assert_eq!(g.computed_permanent(id).unwrap().power, 2, "steed is one artifact → no metalcraft");
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::ornithopter());
    }
    assert_eq!(g.computed_permanent(id).unwrap().power, 4, "three artifacts → +2/+2");
}

#[test]
fn vulshok_replica_sac_burns() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::vulshok_replica());
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, id, 0, Some(Target::Permanent(foe)));
    assert!(g.battlefield_find(id).is_none(), "sacrificed as a cost");
    assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 3/3");
}

#[test]
fn bloodhall_ooze_has_two_color_gated_upkeep_growers() {
    use crabomination::card::{Predicate, SelectionRequirement as R};
    use crabomination::mana::Color;
    let d = catalog::bloodhall_ooze();
    assert_eq!(d.triggered_abilities.len(), 2, "one per color");
    for (color, ab) in [Color::Black, Color::Green].iter().zip(&d.triggered_abilities) {
        assert!(matches!(ab.event.kind, EventKind::StepBegins(TurnStep::Upkeep)));
        let want = Predicate::SelectorCountAtLeast {
            sel: Selector::EachPermanent(R::HasColor(*color).and(R::ControlledByYou)),
            n: crabomination::card::Value::Const(1),
        };
        assert_eq!(ab.event.filter.as_ref(), Some(&want), "gated on controlling that color");
        assert!(matches!(ab.effect, Effect::MayDo { .. }), "may add a +1/+1 counter");
    }
}

#[test]
fn sylvan_might_pumps_and_grants_trample() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::sylvan_might(), vec![Target::Permanent(mine)]);
    let cp = g.computed_permanent(mine).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(catalog::sylvan_might().keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
}

#[test]
fn nimble_innovator_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island()); // the card to draw
    let id = g.add_card_to_hand(0, catalog::nimble_innovator());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nimble Innovator");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "resolved onto the battlefield");
    assert_eq!(g.players[0].hand.len(), 1, "ETB drew the island");
}

#[test]
fn barrage_ogre_sacs_artifact_to_burn() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::barrage_ogre());
    let art = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.clear_sickness(ogre);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    activate(&mut g, ogre, 0, Some(Target::Permanent(foe)));
    assert!(g.battlefield_find(art).is_none(), "sacrificed the artifact");
    assert!(g.battlefield_find(foe).is_none(), "2 damage kills the 2/2");
}

#[test]
fn reckless_imp_flies_and_cant_block() {
    let imp = catalog::reckless_imp();
    assert!(imp.keywords.contains(&Keyword::Flying));
    assert!(imp.keywords.contains(&Keyword::CantBlock));
    assert!(imp.alternative_cost.is_some(), "has Dash");
}

#[test]
fn colossodon_yearling_is_a_beast() {
    let c = catalog::colossodon_yearling();
    assert_eq!((c.power, c.toughness), (2, 4));
    assert!(c.subtypes.creature_types.contains(&crabomination::card::CreatureType::Beast));
}
