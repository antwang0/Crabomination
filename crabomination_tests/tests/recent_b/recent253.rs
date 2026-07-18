//! Functionality tests for `catalog::sets::decks::recent253` (Ravnica legends).

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;

/// Trostani, Three Whispers grants deathtouch to a target creature.
#[test]
fn trostani_grants_deathtouch() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let trostani = g.add_card_to_battlefield(0, catalog::trostani_three_whispers());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: trostani,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        x_value: None,
    })
    .expect("activate the deathtouch ability");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch),
        "bear gained deathtouch",
    );
}

/// Ezrim investigates twice on ETB and gains a chosen keyword by sacrificing an
/// artifact.
#[test]
fn ezrim_investigates_and_grants_chosen_keyword() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let ezrim = g.add_card_to_battlefield(0, catalog::ezrim_agency_chief());
    g.fire_self_etb_triggers(ezrim, 0);
    drain_stack(&mut g);
    let clues = g.battlefield.iter().filter(|c| c.definition.name == "Clue" && c.controller == 0).count();
    assert_eq!(clues, 2, "investigated twice");
    // The keyword-grant ability carries a "sacrifice an artifact" cost.
    let ability = &catalog::ezrim_agency_chief().activated_abilities[0];
    assert!(ability.sac_other_filter.is_some(), "sacrifice-an-artifact cost present");
    // Resolve the modal grant, choosing lifelink (mode 1).
    let ctx = EffectContext::for_trigger(ezrim, 0, None, 1);
    g.resolve_effect(&ability.effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(ezrim).unwrap().keywords.contains(&Keyword::Lifelink),
        "Ezrim gained the chosen keyword (lifelink)",
    );
}

/// Agrus Kos suspects a clean creature, then exiles it once it's suspected.
#[test]
fn agrus_kos_suspects_then_exiles() {
    let mut g = two_player_game();
    let agrus = g.add_card_to_battlefield(0, catalog::agrus_kos_spirit_of_justice());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let interrogate = catalog::agrus_kos_spirit_of_justice().triggered_abilities[0].effect.clone();
    // First interrogation: not suspected → suspect it.
    let ctx = EffectContext::for_trigger(agrus, 0, Some(Target::Permanent(foe)), 0);
    g.resolve_effect(&interrogate, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().suspected, "creature suspected");
    // Second interrogation: already suspected → exile it.
    let ctx = EffectContext::for_trigger(agrus, 0, Some(Target::Permanent(foe)), 0);
    g.resolve_effect(&interrogate, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "suspected creature exiled");
    assert!(g.exile.iter().any(|c| c.id == foe), "moved to exile");
}

/// Aurelia draws on a 3-creature attack and drains on a 5-creature attack.
#[test]
fn aurelia_law_above_attack_triggers() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let aurelia = g.add_card_to_battlefield(0, catalog::aurelia_the_law_above());
    let mut atk = vec![aurelia];
    for _ in 0..4 {
        atk.push(g.add_card_to_battlefield(0, catalog::grizzly_bears()));
    }
    for &id in &atk { g.clear_sickness(id); }
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let hand_before = g.players[0].hand.len();
    let foe_life = g.players[1].life;
    let my_life = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(atk.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect())
        .expect("declare five attackers");
    drain_stack(&mut g);
    // 5 attackers ≥ 3 (draw) and ≥ 5 (drain) both fire.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew from the 3+ attack trigger");
    assert_eq!(g.players[1].life, foe_life - 3, "opponent took 3 from the 5+ trigger");
    assert_eq!(g.players[0].life, my_life + 3, "gained 3 from the 5+ trigger");
}

/// Rakdos draws two when the opponent has nothing to sacrifice at your end step.
#[test]
fn rakdos_patron_draws_when_no_sacrifice() {
    use crabomination::game::TurnStep;
    let mut g = two_player_game();
    let rakdos = g.add_card_to_battlefield(0, catalog::rakdos_patron_of_chaos());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    // Opponent controls only a token (nontoken required) → can't pay.
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let tok = TokenDefinition {
        name: "Bird".into(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 1, toughness: 1, ..Default::default()
    };
    g.add_token_to_battlefield(1, &tok);
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_trigger(rakdos, 0, None, 0);
    let trig = catalog::rakdos_patron_of_chaos().triggered_abilities[0].effect.clone();
    g.step = TurnStep::End;
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew two (no sacrifice available)");
}

/// Voja pumps each creature by the Elf count and draws per Wolf on attack.
#[test]
fn voja_attack_counters_and_draw() {
    use crabomination::card::{CardType, CreatureType, Subtypes, TokenDefinition};
    let mut g = two_player_game();
    let voja = g.add_card_to_battlefield(0, catalog::voja_jaws_of_the_conclave()); // a Wolf
    let elf = TokenDefinition {
        name: "Elf".into(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 1, toughness: 1, ..Default::default()
    };
    let e1 = g.add_token_to_battlefield(0, &elf);
    g.add_token_to_battlefield(0, &elf);
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let hand_before = g.players[0].hand.len();
    let ctx = EffectContext::for_trigger(voja, 0, None, 0);
    let trig = catalog::voja_jaws_of_the_conclave().triggered_abilities[0].effect.clone();
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    // Two Elves → +2 counters on each creature; one Wolf (Voja) → draw 1.
    assert_eq!(g.battlefield_find(e1).unwrap().counters.get(&crabomination::card::CounterType::PlusOnePlusOne).copied().unwrap_or(0), 2, "each creature got 2 counters");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew one per Wolf");
}
