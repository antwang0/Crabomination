//! Functionality tests for Journey into Nyx (JOU) — `catalog::sets::jou`.

use crabomination::card::{CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(
    g: &mut GameState,
    def: crabomination::card::CardDefinition,
    target: Option<Target>,
    colorless: u32,
    colors: &[(Color, u32)],
) -> CardId {
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(colorless);
    for (c, n) in colors {
        g.players[0].mana_pool.add(*c, *n);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
    id
}

/// Stat / keyword lines for the JOU bodies.
#[test]
fn jou_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::oreskos_swiftclaw, 3, 1, &[]),
        (catalog::pensive_minotaur, 2, 3, &[]),
        (catalog::eagle_of_the_watch, 2, 1, &[Keyword::Flying, Keyword::Vigilance]),
        (catalog::bassara_tower_archer, 2, 1, &[Keyword::Hexproof, Keyword::Reach]),
        (catalog::cloaked_siren, 3, 2, &[Keyword::Flash, Keyword::Flying]),
        (catalog::gold_forged_sentinel, 4, 4, &[Keyword::Flying]),
        (catalog::golden_hind, 2, 1, &[]),
        (catalog::lagonna_band_trailblazer, 0, 4, &[]),
        (catalog::dawnbringer_charioteers, 2, 4, &[Keyword::Flying, Keyword::Lifelink]),
        (catalog::felhide_petrifier, 2, 3, &[Keyword::Deathtouch]),
        (catalog::hydra_broodmaster, 7, 7, &[]),
        (catalog::mogiss_warhound, 2, 2, &[Keyword::MustAttack]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

/// Heroic on Akroan Line Breaker pumps and grants intimidate.
#[test]
fn akroan_line_breaker_heroic_pumps_and_sneaks() {
    let mut g = main_phase();
    let breaker = g.add_card_to_battlefield(0, catalog::akroan_line_breaker());
    cast(
        &mut g,
        catalog::pin_to_the_earth(),
        Some(Target::Permanent(breaker)),
        1,
        &[(Color::Blue, 1)],
    );
    let cp = g.computed_permanent(breaker).unwrap();
    assert_eq!(cp.power, 2 + 2 - 6, "heroic +2/+0 on top of the Aura's -6/-0");
    assert!(cp.keywords.contains(&Keyword::Intimidate));
}

/// The Centaur lord pumps and keywords its band, not itself twice.
#[test]
fn pheres_band_warchief_leads_the_centaurs() {
    let mut g = main_phase();
    let chief = g.add_card_to_battlefield(0, catalog::pheres_band_warchief());
    let other = g.add_card_to_battlefield(0, catalog::pheres_band_thunderhoof());
    assert_eq!(g.computed_permanent(chief).map(|c| (c.power, c.toughness)), Some((3, 3)));
    assert_eq!(g.computed_permanent(other).map(|c| (c.power, c.toughness)), Some((4, 5)));
    assert!(g.computed_permanent(other).unwrap().keywords.contains(&Keyword::Trample));
}

/// Felhide Petrifier hands deathtouch to your other Minotaurs.
#[test]
fn felhide_petrifier_arms_the_minotaurs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::felhide_petrifier());
    let mino = g.add_card_to_battlefield(0, catalog::pensive_minotaur());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(mino).unwrap().keywords.contains(&Keyword::Deathtouch));
    assert!(!g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Cyclops of Eternal Fury hastes the team.
#[test]
fn cyclops_of_eternal_fury_hastes_the_team() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::cyclops_of_eternal_fury());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Extinguish All Hope spares enchantment creatures.
#[test]
fn extinguish_all_hope_spares_enchantment_creatures() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let nyx = g.add_card_to_battlefield(1, catalog::cyclops_of_eternal_fury());
    cast(&mut g, catalog::extinguish_all_hope(), None, 4, &[(Color::Black, 2)]);
    assert!(g.battlefield_find(bear).is_none());
    assert!(g.battlefield_find(nyx).is_some(), "enchantment creature survived");
}

/// Feast of Dreams kills an enchanted creature and an enchantment creature.
#[test]
fn feast_of_dreams_hits_enchanted_and_enchantment_creatures() {
    let mut g = main_phase();
    let nyx = g.add_card_to_battlefield(1, catalog::cyclops_of_eternal_fury());
    cast(&mut g, catalog::feast_of_dreams(), Some(Target::Permanent(nyx)), 1, &[(Color::Black, 1)]);
    assert!(g.battlefield_find(nyx).is_none());

    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, catalog::aspect_of_gorgon(), Some(Target::Permanent(bear)), 2, &[(Color::Black, 1)]);
    cast(&mut g, catalog::feast_of_dreams(), Some(Target::Permanent(bear)), 1, &[(Color::Black, 1)]);
    assert!(g.battlefield_find(bear).is_none(), "the enchanted Bears died too");
}

/// Nyx Infusion swings either way depending on the host's types.
#[test]
fn nyx_infusion_reads_the_host() {
    let mut g = main_phase();
    let sentinel = g.add_card_to_battlefield(0, catalog::gold_forged_sentinel()); // 4/4
    cast(&mut g, catalog::nyx_infusion(), Some(Target::Permanent(sentinel)), 2, &[(Color::Black, 1)]);
    assert_eq!(g.computed_permanent(sentinel).map(|c| (c.power, c.toughness)), Some((2, 2)));

    let nyx = g.add_card_to_battlefield(0, catalog::cyclops_of_eternal_fury());
    cast(&mut g, catalog::nyx_infusion(), Some(Target::Permanent(nyx)), 2, &[(Color::Black, 1)]);
    assert_eq!(g.computed_permanent(nyx).map(|c| (c.power, c.toughness)), Some((7, 5)));
}

/// Hubris bounces the creature and everything hanging off it.
#[test]
fn hubris_bounces_the_whole_package() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = cast(
        &mut g,
        catalog::aspect_of_gorgon(),
        Some(Target::Permanent(bear)),
        2,
        &[(Color::Black, 1)],
    );
    cast(&mut g, catalog::hubris(), Some(Target::Permanent(bear)), 1, &[(Color::Blue, 1)]);
    assert!(g.battlefield_find(bear).is_none(), "the creature bounced");
    assert!(g.battlefield_find(aura).is_none(), "so did its Aura");
}

/// Hall of Triumph anthems only the chosen color.
#[test]
fn hall_of_triumph_anthems_the_chosen_color() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Green)]));
    cast(&mut g, catalog::hall_of_triumph(), None, 3, &[]);
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let white = g.add_card_to_battlefield(0, catalog::oreskos_swiftclaw());
    assert_eq!(g.computed_permanent(green).map(|c| (c.power, c.toughness)), Some((3, 3)));
    assert_eq!(g.computed_permanent(white).map(|c| (c.power, c.toughness)), Some((3, 1)));
}

/// Hydra Broodmaster's monstrosity X mints X X/X Hydras.
#[test]
fn hydra_broodmaster_mints_a_brood() {
    let mut g = main_phase();
    let hydra = g.add_card_to_battlefield(0, catalog::hydra_broodmaster());
    g.clear_sickness(hydra);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hydra,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
    })
    .expect("monstrosity 2");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2
    );
    let brood: Vec<_> = g.battlefield.iter().filter(|c| c.definition.name == "Hydra").collect();
    assert_eq!(brood.len(), 2, "two Hydras");
    assert_eq!((brood[0].definition.power, brood[0].definition.toughness), (2, 2));
}

/// The Fonts trade themselves in for their payoff.
#[test]
fn jou_fonts_cash_out() {
    let mut g = main_phase();
    let font = cast(&mut g, catalog::font_of_vigor(), None, 1, &[(Color::White, 1)]);
    let life = g.players[0].life;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: font,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    })
    .expect("cash it in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 7);
    assert!(g.battlefield_find(font).is_none(), "sacrificed");
}

/// King Macar turns a creature into a Gold token.
#[test]
fn king_macar_gilds_a_creature() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let king = g.add_card_to_battlefield(0, catalog::king_macar_the_gold_cursed());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        king,
        0,
        Some(Target::Permanent(victim)),
    );
    g.resolve_effect(
        &catalog::king_macar_the_gold_cursed().triggered_abilities[0].effect,
        &ctx,
    )
    .expect("inspired body");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "exiled");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Gold"));
}
