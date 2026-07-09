//! Functionality tests for `catalog::sets::decks::recent114` — the Enchantress
//! / enchantment-matters package, including the Opalescence / Starfield of Nyx
//! `NonAuraEnchantmentsAreCreatures` animate static.

use crate::catalog;
use crate::card::CardType;
use crate::game::types::{Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut guard = 0;
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
        guard += 1;
        assert!(guard < 80, "advance_to overran");
    }
}

// ── Opalescence / Starfield (NonAuraEnchantmentsAreCreatures) ────────────────

/// CR 613 — Opalescence makes each *other* non-Aura enchantment an `MV/MV`
/// creature, and leaves itself alone.
#[test]
fn opalescence_animates_other_non_aura_enchantments() {
    let mut g = two_player_game();
    let opal = g.add_card_to_battlefield(0, catalog::opalescence());
    // Enchantress's Presence is {2}{G} — mana value 3.
    let presence = g.add_card_to_battlefield(0, catalog::enchantresss_presence());

    let cp = g.computed_permanent(presence).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature), "non-Aura enchantment becomes a creature");
    assert_eq!((cp.power, cp.toughness), (3, 3), "base P/T = mana value");

    // Opalescence itself ("each *other*") is untouched.
    let self_cp = g.computed_permanent(opal).unwrap();
    assert!(!self_cp.card_types.contains(&CardType::Creature), "Opalescence isn't a creature");
}

/// An Aura enchantment is exempt from Opalescence (it names "non-Aura").
#[test]
fn opalescence_spares_auras() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::opalescence());
    // Wild Growth is an Aura enchantment.
    let aura = g.add_card_to_battlefield(0, catalog::wild_growth());
    assert!(
        !g.computed_permanent(aura).unwrap().card_types.contains(&CardType::Creature),
        "Aura enchantment stays a non-creature under Opalescence"
    );
}

/// Starfield of Nyx only animates while its controller has 5+ enchantments.
#[test]
fn starfield_gates_on_five_enchantments() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::starfield_of_nyx());
    let presence = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
    // Two enchantments total → gate unmet.
    assert!(
        !g.computed_permanent(presence).unwrap().card_types.contains(&CardType::Creature),
        "under five enchantments, nothing animates"
    );
    // Pad up to five enchantments.
    g.add_card_to_battlefield(0, catalog::sigil_of_the_empty_throne());
    g.add_card_to_battlefield(0, catalog::greater_auramancy());
    g.add_card_to_battlefield(0, catalog::nevermore());
    assert!(
        g.computed_permanent(presence).unwrap().card_types.contains(&CardType::Creature),
        "at five enchantments Starfield animates the others"
    );
}

// ── Enchantment-cast payoffs ─────────────────────────────────────────────────

/// Enchantress's Presence draws a card when you cast an enchantment spell.
#[test]
fn enchantresss_presence_draws_on_enchantment_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::enchantresss_presence());
    let spell = g.add_card_to_hand(0, catalog::sigil_of_the_empty_throne());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    let hand = g.players[0].hand.len();
    cast(&mut g, spell);
    // Cast one enchantment (-1 hand), the trigger drew one (+1) → net even.
    assert_eq!(g.players[0].hand.len(), hand - 1 + 1, "cast an enchantment, drew a card");
}

/// Herald of the Pantheon reduces enchantment spells by {1} and gains life.
#[test]
fn herald_of_the_pantheon_reduces_and_gains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::herald_of_the_pantheon());
    // Enchantress's Presence is {2}{G}; the {1} discount lets {1}{G} pay it.
    let ench = g.add_card_to_hand(0, catalog::enchantresss_presence());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    cast(&mut g, ench);
    assert!(g.battlefield_find(ench).is_some(), "discounted enchantment resolved");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life on the enchantment cast");
}

/// Sigil of the Empty Throne mints a 4/4 flying Angel per enchantment cast.
#[test]
fn sigil_of_the_empty_throne_makes_angels() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sigil_of_the_empty_throne());
    let ench = g.add_card_to_hand(0, catalog::greater_auramancy());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ench);
    let angels = g.battlefield.iter().filter(|c| c.definition.name == "Angel").count();
    assert_eq!(angels, 1, "one 4/4 Angel from the enchantment cast");
}

/// Ajani's Chosen mints a 2/2 Cat when an enchantment enters.
#[test]
fn ajanis_chosen_makes_cats() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ajanis_chosen());
    let ench = g.add_card_to_hand(0, catalog::nevermore());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, ench);
    let cats = g.battlefield.iter().filter(|c| c.definition.name == "Cat").count();
    assert_eq!(cats, 1, "one 2/2 Cat when the enchantment entered");
}

// ── Protection statics ───────────────────────────────────────────────────────

/// Privileged Position grants hexproof to your other permanents.
#[test]
fn privileged_position_grants_hexproof() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::privileged_position());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(
        g.computed_permanent(bear).unwrap().keywords.iter().any(|k| matches!(
            k,
            crate::card::Keyword::Hexproof
        )),
        "other permanent you control has hexproof"
    );
}

/// Greater Auramancy grants shroud to your other enchantments only.
#[test]
fn greater_auramancy_grants_shroud_to_enchantments() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::greater_auramancy());
    let other_ench = g.add_card_to_battlefield(0, catalog::enchantresss_presence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let has_shroud = |cp: &crate::game::layers::ComputedPermanent| {
        cp.keywords.iter().any(|k| matches!(k, crate::card::Keyword::Shroud))
    };
    assert!(has_shroud(&g.computed_permanent(other_ench).unwrap()), "other enchantment has shroud");
    assert!(!has_shroud(&g.computed_permanent(bear).unwrap()), "non-enchantment is unaffected");
}

/// Nevermore stops a named spell from being cast.
#[test]
fn nevermore_locks_named_spell() {
    let mut g = two_player_game();
    let nm = g.add_card_to_battlefield(0, catalog::nevermore());
    // Name Lightning Bolt on the enchantment.
    g.battlefield_find_mut(nm).unwrap().named_card = Some("Lightning Bolt".into());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "the named spell can't be cast");
}

// ── ETB / trigger utility ────────────────────────────────────────────────────

/// Grasp of Fate exiles an opponent's permanent and returns it when it leaves.
#[test]
fn grasp_of_fate_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let grasp = g.add_card_to_hand(0, catalog::grasp_of_fate());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: grasp, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast Grasp of Fate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "opponent's creature exiled");
    // Grasp leaves; the exiled card returns.
    let gid = g.battlefield.iter().find(|c| c.definition.name == "Grasp of Fate").unwrap().id;
    let evs = g.remove_to_graveyard_with_triggers(gid);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_some(), "returns when Grasp leaves");
}

/// Sigil of the New Dawn returns a dead creature to hand for {1}{W}.
#[test]
fn sigil_of_the_new_dawn_returns_dead_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sigil_of_the_new_dawn());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    // AutoDecider declines optional costs; script the {1}{W} payment.
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    g.battlefield_find_mut(bear).unwrap().damage = 2; // lethal → CreatureDied
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "paid {{1}}{{W}} to return the dead creature to hand"
    );
}

/// Calix, Guided by Fate puts a +1/+1 counter on a creature on constellation.
#[test]
fn calix_constellation_adds_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::calix_guided_by_fate());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ench = g.add_card_to_hand(0, catalog::nevermore());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast enchantment");
    drain_stack(&mut g);
    let _ = bear;
    let total: u32 = g
        .battlefield
        .iter()
        .map(|c| c.counter_count(crate::card::CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(total, 1, "constellation put one +1/+1 counter on a creature");
}

