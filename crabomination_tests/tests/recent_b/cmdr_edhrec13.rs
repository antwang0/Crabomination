//! Commander: primitive-blocked one-card EDHREC gaps (`decks::cmdr_edhrec13`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn tap_for_mana(g: &mut GameState, id: CardId) {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("mana ability");
    drain_stack(g);
}

fn plus_ones(g: &GameState, id: CardId) -> u32 {
    g.battlefield_find(id).map_or(0, |c| c.counter_count(CounterType::PlusOnePlusOne))
}

/// Tyvar the Bellicose (CR 605.1a, 603.2): the granted trigger counts the
/// mana the creature's own ability made — three Elves make Elvish Archdruid
/// tap for three — and per its "only once each turn" (CR 603.2h? ruling
/// 2023-05-12) a second tap the same turn adds nothing.
#[test]
fn tyvar_counts_the_mana_each_creature_made_once_a_turn() {
    let mut g = pod(4);
    ready(&mut g, 0, catalog::tyvar_the_bellicose());
    let druid = ready(&mut g, 0, catalog::elvish_archdruid());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let foe = ready(&mut g, 1, catalog::llanowar_elves());
    tap_for_mana(&mut g, druid);
    // Archdruid's lord bonus doesn't touch counters; Tyvar + druid + Llanowar = 3 Elves.
    assert_eq!(plus_ones(&g, druid), 3, "three Elves → three green → three counters");
    tap_for_mana(&mut g, elf);
    assert_eq!(plus_ones(&g, elf), 1);
    g.battlefield_find_mut(elf).unwrap().tapped = false;
    tap_for_mana(&mut g, elf);
    assert_eq!(plus_ones(&g, elf), 1, "the granted trigger fires once a turn per creature");
    assert_eq!(plus_ones(&g, foe), 0, "an opponent's creature doesn't have the ability");
}

/// Tyvar the Bellicose: "Whenever one or more Elves you control attack, they
/// gain deathtouch" — the attacking Elves, not a non-Elf beside them.
#[test]
fn tyvar_gives_attacking_elves_deathtouch() {
    let mut g = pod(3);
    let tyvar = ready(&mut g, 0, catalog::tyvar_the_bellicose());
    let elf = ready(&mut g, 0, catalog::llanowar_elves());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        crabomination::game::types::Attack { attacker: elf, target: crabomination::game::types::AttackTarget::Player(1) },
        crabomination::game::types::Attack { attacker: bear, target: crabomination::game::types::AttackTarget::Player(2) },
    ]))
    .expect("declare");
    drain_stack(&mut g);
    assert!(g.computed_permanent(elf).unwrap().keywords().contains(&Keyword::Deathtouch));
    assert!(!g.computed_permanent(bear).unwrap().keywords().contains(&Keyword::Deathtouch));
    assert!(!g.computed_permanent(tyvar).unwrap().keywords().contains(&Keyword::Deathtouch), "Tyvar stayed home");
}

fn cast_from_hand(g: &mut GameState, def: crabomination::card::CardDefinition) -> Result<(), String> {
    cast_at(g, def, None)
}

fn cast_at(g: &mut GameState, def: crabomination::card::CardDefinition, target: Option<Target>) -> Result<(), String> {
    let id = g.add_card_to_hand(0, def);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 119.4b + Angel of Jubilation's rulings (2012-05-01): no creature may be
/// sacrificed to pay a spell's or an ability's cost — an artifact creature
/// can't pay "sacrifice an artifact" — but a noncreature artifact still can.
#[test]
fn cr_119_4b_angel_of_jubilation_bars_creature_sacrifices_as_costs() {
    let mut g = pod(4);
    ready(&mut g, 1, catalog::angel_of_jubilation());
    ready(&mut g, 0, catalog::grizzly_bears());
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 10);
    g.players[0].mana_pool.add_colorless(10);
    assert!(cast_from_hand(&mut g, catalog::village_rites()).is_err(), "Village Rites can't sacrifice a creature");
    assert!(cast_from_hand(&mut g, catalog::deadly_dispute()).is_err(), "an artifact creature can't pay it either");
    let seer = ready(&mut g, 0, catalog::viscera_seer());
    let act = g.perform_action(GameAction::ActivateAbility {
        card_id: seer,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    });
    assert!(act.is_err(), "Viscera Seer's sacrifice-a-creature cost is barred");
    let ring = ready(&mut g, 0, catalog::sol_ring());
    for _ in 0..2 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    cast_from_hand(&mut g, catalog::deadly_dispute()).expect("a noncreature artifact pays Deadly Dispute");
    assert!(g.battlefield_find(ring).is_none(), "Sol Ring went");
    assert!(g.battlefield_find(thopter).is_some(), "the Ornithopter stayed");
}

/// CR 119.4b (Angel of Jubilation) and CR 119.8 (Platinum Emperion's "can't
/// lose life"): a Phyrexian pip can't be paid with life, so Gitaxian Probe
/// needs its blue — and with the blue it's cast without the life.
#[test]
fn cr_119_8_phyrexian_pips_take_mana_when_life_cant_be_paid() {
    for barrier in [catalog::angel_of_jubilation as fn() -> _, catalog::platinum_emperion] {
        let mut g = pod(2);
        ready(&mut g, 0, barrier());
        assert!(cast_at(&mut g, catalog::gitaxian_probe(), Some(Target::Player(1))).is_err(), "no life, no blue: no Probe");
        g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
        let life = g.players[0].life;
        cast_at(&mut g, catalog::gitaxian_probe(), Some(Target::Player(1))).expect("blue pays the pip");
        assert_eq!(g.players[0].life, life);
    }
    // Control: with neither out, the pip is paid with 2 life.
    let mut g = pod(2);
    let life = g.players[0].life;
    cast_at(&mut g, catalog::gitaxian_probe(), Some(Target::Player(1))).expect("life pays the pip");
    assert_eq!(g.players[0].life, life - 2);
}

/// Emrakul, the Exigent Doom: {3} and exiling it from hand gives a land
/// "{T}: Add {C}{C}" until the card is cast from exile (CR 611.2b), and lets
/// it be cast from exile; casting it untaps your lands (its cast trigger).
#[test]
fn emrakul_grants_a_land_two_colorless_until_cast_from_exile() {
    let mut g = pod(4);
    let land = ready(&mut g, 0, catalog::wastes());
    let emrakul = g.add_card_to_hand(0, catalog::emrakul_the_exigent_doom());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: emrakul,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("hand ability");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == emrakul && c.may_play_until.is_some()), "exiled, castable");
    assert_eq!(g.battlefield_find(land).unwrap().granted_activated_abilities.len(), 1);
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: emrakul,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from exile");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the cast paid Emrakul's own {{10}}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(emrakul).is_some(), "Emrakul resolved");
    let l = g.battlefield_find(land).unwrap();
    assert!(l.granted_activated_abilities.is_empty(), "the grant ended as Emrakul was cast");
    assert!(!l.tapped, "the cast trigger untapped the land");
}

/// CR 400.7 — Emrakul's land grant lasts "until this card is cast from
/// exile": once it has left exile uncast, a cast from hand leaves the land its
/// {T}: Add {C}{C}.
#[test]
fn emrakul_grant_outlives_a_cast_from_hand() {
    let mut g = pod(4);
    let land = ready(&mut g, 0, catalog::wastes());
    let emrakul = g.add_card_to_hand(0, catalog::emrakul_the_exigent_doom());
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: emrakul,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("hand ability");
    drain_stack(&mut g);
    let back = crabomination::effect::Effect::Move {
        what: crabomination::effect::Selector::ExactObjects(vec![emrakul]),
        to: crabomination::effect::ZoneDest::Hand(crabomination::effect::PlayerRef::You),
    };
    g.resolve_effect(&back, &crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0)).unwrap();
    assert!(g.players[0].hand.iter().any(|c| c.id == emrakul), "back in hand, uncast");
    g.players[0].mana_pool.add_colorless(10);
    g.perform_action(GameAction::CastSpell { card_id: emrakul, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast from hand");
    drain_stack(&mut g);
    assert!(g.battlefield_find(emrakul).is_some());
    assert_eq!(g.battlefield_find(land).unwrap().granted_activated_abilities.len(), 1, "the grant stays");
}

/// Mystic Reflection (CR 707.2, 614.12; rulings 2021-02-05): every creature in
/// the next batch of entries enters as a copy of the chosen creature — both
/// Raise the Alarm tokens become Grizzly Bears — and a later entry doesn't.
#[test]
fn mystic_reflection_copies_the_next_batch_of_entries_only() {
    let mut g = pod(4);
    let bears = ready(&mut g, 0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, catalog::mystic_reflection(), Some(Target::Permanent(bears))).expect("reflection");
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_from_hand(&mut g, catalog::raise_the_alarm()).expect("tokens");
    let copies = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Grizzly Bears").count();
    assert_eq!(copies, 2, "both tokens of the batch entered as Grizzly Bears");
    cast_from_hand(&mut g, catalog::ornithopter()).expect("ornithopter");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Ornithopter"), "the batch after is itself");
}

/// Mystic Reflection's ruling: a model that left the battlefield is copied
/// as it last existed there (CR 608.2h) — a dead token model still works.
#[test]
fn mystic_reflection_copies_a_model_that_left_as_it_last_existed() {
    use crabomination::mana::Color;
    let mut g = pod(4);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_from_hand(&mut g, catalog::raise_the_alarm()).expect("tokens");
    let soldier = g.battlefield.iter().find(|c| c.is_token).unwrap().id;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, catalog::mystic_reflection(), Some(Target::Permanent(soldier))).expect("reflection");
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, catalog::lightning_bolt(), Some(Target::Permanent(soldier))).expect("bolt");
    assert!(g.battlefield_find(soldier).is_none());
    cast_from_hand(&mut g, catalog::ornithopter()).expect("ornithopter");
    let soldiers = g.battlefield.iter().filter(|c| !c.is_token && c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 1, "the Ornithopter entered as the dead Soldier token");
}

/// Painter's Servant (CR 105.3; rulings 2026-09-01): the chosen color is added
/// to permanents (any controller's, colorless ones too) and to cards in hand
/// and graveyard, beside their own colors; it goes when the Servant does.
#[test]
fn painters_servant_paints_every_zone_in_addition() {
    use crabomination::mana::Color;
    let mut g = pod(3);
    let servant = ready(&mut g, 0, catalog::painters_servant());
    g.battlefield_find_mut(servant).unwrap().chosen_color = Some(Color::Blue);
    let bear = ready(&mut g, 1, catalog::grizzly_bears());
    let hand = g.add_card_to_hand(2, catalog::lightning_bolt());
    let colors = |g: &GameState, id| g.card_colors_anywhere(id);
    assert!(colors(&g, bear).contains(&Color::Blue) && colors(&g, bear).contains(&Color::Green));
    assert_eq!(colors(&g, servant), vec![Color::Blue], "colorless becomes just the color");
    assert!(colors(&g, hand).contains(&Color::Blue) && colors(&g, hand).contains(&Color::Red));
    g.remove_from_battlefield_to_graveyard_raw(servant);
    assert!(!colors(&g, hand).contains(&Color::Blue), "gone with the Servant");
    assert!(!colors(&g, bear).contains(&Color::Blue));
}
