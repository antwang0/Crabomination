//! CR conformance for rules exercised by the recent301–304 Dissension batches:
//! CR 305.7 (a land that "becomes a basic type" loses its other types and its
//! intrinsic mana ability follows the new type — Terraformer), CR 202.2b
//! (hybrid mana pips make a card multicolored — the Eidolon recur trigger), and
//! CR 701.15 (a regeneration shield replaces destruction).

use crabomination::card::LandType;
use crabomination::catalog;
use crabomination::effect::{Effect, Selector};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::Target;
use crabomination::game::{drain_stack, two_player_game, GameAction};
use crabomination::mana::Color;

/// CR 305.7 — Terraformer turns each of your lands into a chosen basic type
/// until end of turn: the land loses its other land types (and the printed
/// abilities tied to them), keeping only the chosen basic type.
#[test]
fn cr_305_7_terraformer_relands_your_lands() {
    let mut g = two_player_game();
    let tf = g.add_card_to_battlefield(0, catalog::terraformer());
    g.clear_sickness(tf);
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Color(Color::Blue),
    ]));
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: tf, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("activate Terraformer choosing Island");
    drain_stack(&mut g);
    let cp = g.computed_permanent(mtn).unwrap();
    assert_eq!(cp.subtypes.land_types, vec![LandType::Island], "Mountain fully replaced by Island");
    assert!(!cp.subtypes.land_types.contains(&LandType::Mountain), "lost its old basic type");
}

/// CR 202.2b — a card with hybrid mana symbols is every color those symbols
/// reference; Boros Guildmage ({R/W}{R/W}) is both red and white.
#[test]
fn cr_202_2b_hybrid_card_is_multicolored() {
    let mut g = two_player_game();
    let bg = g.add_card_to_battlefield(0, catalog::boros_guildmage());
    let colors = g.computed_permanent(bg).unwrap().colors.clone();
    assert!(colors.contains(&Color::Red) && colors.contains(&Color::White),
        "hybrid pips make it both colors");
    assert_eq!(colors.len(), 2, "exactly two colors → multicolored");
}

/// CR 701.15 — a regeneration shield replaces the next destruction this turn:
/// the creature is tapped and stays on the battlefield instead of dying.
#[test]
fn cr_701_15_regeneration_shield_replaces_destruction() {
    let mut g = two_player_game();
    let ape = g.add_card_to_battlefield(0, catalog::gorilla_chieftain());
    g.clear_sickness(ape);
    // Stamp a shield via its "{1}{G}: Regenerate this creature." ability.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ape, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ape).unwrap().regeneration_shields, 1);
    // Destroy it — the shield is spent instead of the creature dying.
    let ctx = EffectContext::for_ability(ape, 0, Some(Target::Permanent(ape)));
    let evs = g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let c = g.battlefield_find(ape).expect("survived destruction");
    assert!(c.tapped, "regeneration taps it");
    assert_eq!(c.regeneration_shields, 0, "shield consumed");
}
