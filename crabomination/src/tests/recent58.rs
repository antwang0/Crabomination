//! Functionality tests for `catalog::sets::decks::recent58` — party (CR 700.18).
//! `Value::PartyCount` is exercised deterministically via Squad Commander's
//! "make a token for each creature in your party" ETB (no target ambiguity).

use crate::card::{CardType, CreatureType, Keyword, Subtypes};
use crate::catalog;
use crate::game::*;

/// A vanilla 1/1 creature of a chosen party role, for building a party.
fn role(ct: CreatureType) -> crate::card::CardDefinition {
    crate::card::CardDefinition {
        name: "Party Member",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    }
}

/// Squad Commander (itself a Warrior) mints one Kor Warrior per party member.
fn commander_tokens(setup: &[CreatureType]) -> usize {
    let mut g = two_player_game();
    for &ct in setup {
        g.add_card_to_battlefield(0, role(ct));
    }
    let cmd = g.add_card_to_battlefield(0, catalog::squad_commander());
    g.fire_self_etb_triggers(cmd, 0);
    drain_stack(&mut g);
    g.battlefield.iter().filter(|c| c.definition.name == "Kor Warrior").count()
}

#[test]
fn party_counts_each_distinct_role() {
    // Cleric + Rogue + Wizard + Squad (Warrior) = full party of 4.
    assert_eq!(commander_tokens(&[CreatureType::Cleric, CreatureType::Rogue, CreatureType::Wizard]), 4);
}

#[test]
fn party_duplicate_roles_fill_one_slot() {
    // Two extra Warriors + Squad (Warrior) — all Warriors → party 1.
    assert_eq!(commander_tokens(&[CreatureType::Warrior, CreatureType::Warrior]), 1);
}

#[test]
fn party_ignores_non_role_creatures() {
    // A Bear and a Cleric + Squad (Warrior) → only Cleric + Warrior count → 2.
    assert_eq!(commander_tokens(&[CreatureType::Bear, CreatureType::Cleric]), 2);
}

#[test]
fn tajuru_paragon_fills_only_one_party_slot() {
    // Tajuru is all four roles but fills only one slot (CR 700.18): with just
    // Squad (Warrior), the party is 2 — Squad→Warrior, Tajuru→one other.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tajuru_paragon());
    let cmd = g.add_card_to_battlefield(0, catalog::squad_commander());
    g.fire_self_etb_triggers(cmd, 0);
    drain_stack(&mut g);
    let tokens = g.battlefield.iter().filter(|c| c.definition.name == "Kor Warrior").count();
    assert_eq!(tokens, 2, "one creature fills at most one slot → party 2, not 4");
}

#[test]
fn squad_commander_full_party_buffs_team_at_combat() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, role(CreatureType::Cleric));
    g.add_card_to_battlefield(0, role(CreatureType::Rogue));
    g.add_card_to_battlefield(0, role(CreatureType::Wizard));
    let cmd = g.add_card_to_battlefield(0, catalog::squad_commander()); // Warrior → full party
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == cmd).unwrap();
    assert_eq!(c.power, 4, "+1/+0 from the full-party combat trigger");
    assert!(c.keywords.contains(&Keyword::Indestructible), "full party → indestructible");
}

#[test]
fn kabira_outrider_pumps_by_party_size() {
    // Cleric + Rogue + Wizard on board; Outrider (Warrior) enters → party 4.
    // The ETB pumps a creature by +4/+4; assert some creature grew by 4.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, role(CreatureType::Cleric));
    g.add_card_to_battlefield(0, role(CreatureType::Rogue));
    g.add_card_to_battlefield(0, role(CreatureType::Wizard));
    let out = g.add_card_to_battlefield(0, catalog::kabira_outrider());
    let total = |g: &GameState| g.compute_battlefield().iter()
        .filter(|c| c.controller == 0).map(|c| c.power).sum::<i32>();
    let before = total(&g);
    g.fire_self_etb_triggers(out, 0);
    drain_stack(&mut g);
    assert_eq!(total(&g), before + 4, "a creature got +4/+0..+4 for the full party of 4");
}
