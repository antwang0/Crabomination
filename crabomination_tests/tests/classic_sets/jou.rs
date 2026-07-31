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
        x_value: Some(2), mode: None,
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
        x_value: None, mode: None,
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

// ── Wave 2: Strive, constellation, and the rest of the core ──────────────────

/// Cast `def` with `extras` additional target slots filled.
fn cast_multi(
    g: &mut GameState,
    def: crabomination::card::CardDefinition,
    target: Target,
    extras: Vec<Target>,
    colorless: u32,
    colors: &[(Color, u32)],
) -> Result<(), crabomination::game::GameError> {
    let id = g.add_card_to_hand(0, def);
    g.players[0].mana_pool.add_colorless(colorless);
    for (c, n) in colors {
        g.players[0].mana_pool.add(*c, *n);
    }
    let r = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(target),
        additional_targets: extras,
        mode: None,
        x_value: None,
    });
    if r.is_ok() {
        drain_stack(g);
    }
    r.map(|_| ())
}

/// CR 702.122 Strive — each extra target charges the full colored rider, and
/// the spell pumps every target it paid for.
#[test]
fn strive_charges_a_colored_rider_per_extra_target() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::oreskos_swiftclaw());

    // Two targets on Ajani's Presence = {W} + {2}{W}. One white short fails.
    assert!(cast_multi(
        &mut g,
        catalog::ajanis_presence(),
        Target::Permanent(a),
        vec![Target::Permanent(b)],
        2,
        &[(Color::White, 1)],
    )
    .is_err());

    g.players[0].mana_pool.empty();
    cast_multi(
        &mut g,
        catalog::ajanis_presence(),
        Target::Permanent(a),
        vec![Target::Permanent(b)],
        2,
        &[(Color::White, 2)],
    )
    .expect("strive for two");
    for id in [a, b] {
        let cp = g.computed_permanent(id).unwrap();
        assert!(cp.keywords.contains(&Keyword::Indestructible), "both got the grant");
    }
    assert_eq!(g.computed_permanent(a).unwrap().power, 3);
}

/// A Strive spell cast on one target pays only its printed cost.
#[test]
fn strive_single_target_costs_the_printed_price() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast(
        &mut g,
        catalog::rouse_the_mob(),
        Some(Target::Permanent(bear)),
        0,
        &[(Color::Red, 1)],
    );
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Fireball's generic per-target rider still works after the Strive rewrite.
#[test]
fn fireball_still_charges_one_generic_per_extra_target() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // {X=4}{R} + {1} for the second target = 6 mana, 2 damage each.
    let id = g.add_card_to_hand(0, catalog::fireball());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: Some(4),
    })
    .expect("fireball two targets");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none());
}

/// Consign to Dust destroys artifacts and enchantments in one Strive.
#[test]
fn consign_to_dust_sweeps_both_types() {
    let mut g = main_phase();
    let ench = g.add_card_to_battlefield(1, catalog::skybind());
    let art = g.add_card_to_battlefield(1, catalog::hall_of_triumph());
    cast_multi(
        &mut g,
        catalog::consign_to_dust(),
        Target::Permanent(ench),
        vec![Target::Permanent(art)],
        4,
        &[(Color::Green, 2)],
    )
    .expect("strive for two");
    assert!(g.battlefield_find(ench).is_none() && g.battlefield_find(art).is_none());
}

/// Harness by Force steals, untaps, and hastes its targets.
#[test]
fn harness_by_force_steals_and_hastes() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    cast(
        &mut g,
        catalog::harness_by_force(),
        Some(Target::Permanent(bear)),
        1,
        &[(Color::Red, 2)],
    );
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 0);
    assert!(!c.tapped);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
}

/// Hour of Need exiles and hands the owner a 4/4 Sphinx.
#[test]
fn hour_of_need_trades_creatures_for_sphinxes() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast(&mut g, catalog::hour_of_need(), Some(Target::Permanent(bear)), 2, &[(Color::Blue, 1)]);
    assert!(g.battlefield_find(bear).is_none());
    let sphinx = g.battlefield.iter().find(|c| c.definition.name == "Sphinx").expect("Sphinx");
    assert_eq!(sphinx.controller, 1);
    assert_eq!((sphinx.definition.power, sphinx.definition.toughness), (4, 4));
}

/// Twinflame mints a hasty token copy that leaves at the next end step.
#[test]
fn twinflame_mints_a_transient_copy() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast(&mut g, catalog::twinflame(), Some(Target::Permanent(bear)), 1, &[(Color::Red, 1)]);
    let copies: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.is_token)
        .collect();
    assert_eq!(copies.len(), 1);
    assert!(g.computed_permanent(copies[0].id).unwrap().keywords.contains(&Keyword::Haste));
}

/// Solidarity of Heroes doubles the +1/+1 counters on its targets.
#[test]
fn solidarity_of_heroes_doubles_counters() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    cast(
        &mut g,
        catalog::solidarity_of_heroes(),
        Some(Target::Permanent(bear)),
        1,
        &[(Color::Green, 1)],
    );
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
}

/// Constellation counts the enchantment creature's own arrival.
#[test]
fn constellation_fires_on_its_own_entry() {
    let mut g = main_phase();
    let opp_life = g.players[1].life;
    cast(&mut g, catalog::thoughtrender_lamia(), None, 4, &[(Color::Black, 2)]);
    // Lamia's own ETB made the opponent discard once.
    assert_eq!(g.players[1].hand.len(), 0, "empty hand, nothing to discard");
    assert_eq!(g.players[1].life, opp_life);
    g.add_card_to_hand(1, catalog::grizzly_bears());
    // A second enchantment entering fires it again.
    cast(&mut g, catalog::skybind(), None, 3, &[(Color::White, 2)]);
    assert_eq!(g.players[1].hand.len(), 0, "the constellation trigger took it");
}

/// Oakheart Dryads pumps on each enchantment that enters.
#[test]
fn oakheart_dryads_pumps_on_each_enchantment() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::oakheart_dryads());
    cast(&mut g, catalog::strength_from_the_fallen(), None, 1, &[(Color::Green, 1)]);
    assert!(
        g.computed_permanent(bear).unwrap().power > 2,
        "the constellation trigger auto-targeted and pumped"
    );
}

/// Squelching Leeches's CDA reads Swamps you control.
#[test]
fn squelching_leeches_counts_swamps() {
    let mut g = main_phase();
    let leech = g.add_card_to_battlefield(0, catalog::squelching_leeches());
    assert_eq!(g.computed_permanent(leech).map(|c| (c.power, c.toughness)), Some((0, 0)));
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    g.add_card_to_battlefield(1, catalog::swamp());
    assert_eq!(
        g.computed_permanent(leech).map(|c| (c.power, c.toughness)),
        Some((3, 3)),
        "only your Swamps count"
    );
}

/// Oppressive Rays taxes attacking and every activation.
#[test]
fn oppressive_rays_taxes_attacks_and_abilities() {
    let mut g = main_phase();
    let starfish = g.add_card_to_battlefield(1, catalog::sigiled_starfish());
    g.clear_sickness(starfish);
    let aura = g.add_card_to_hand(0, catalog::oppressive_rays());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(starfish)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant");
    drain_stack(&mut g);
    // {T}: Scry 1 now costs {3}; with no mana the activation is rejected.
    g.priority.player_with_priority = 1;
    assert!(g
        .perform_action(GameAction::ActivateAbility {
            card_id: starfish,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None, mode: None,
        })
        .is_err());
    g.players[1].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: starfish,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("paid the {3} tax");
}

/// Market Festival's enchanted land adds two extra mana of any colors.
#[test]
fn market_festival_doubles_down_on_a_land() {
    let mut g = main_phase();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::market_festival());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura,
        target: Some(Target::Permanent(forest)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("enchant the land");
    drain_stack(&mut g);
    g.players[0].mana_pool.empty();
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 3, "{{G}} plus two more");
}

/// Thassa's Ire flips a creature's tap state either way.
#[test]
fn thassas_ire_taps_or_untaps() {
    let mut g = main_phase();
    let ire = g.add_card_to_battlefield(0, catalog::thassas_ire());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ire,
        ability_index: 0,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("untap mine");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mine).unwrap().tapped);

    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ire,
        ability_index: 0,
        target: Some(Target::Permanent(theirs)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap theirs");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Stonewise Fortifier's shield is scoped to one attacker.
#[test]
fn stonewise_fortifier_blanks_one_source() {
    let mut g = main_phase();
    let fort = g.add_card_to_battlefield(0, catalog::stonewise_fortifier());
    let big = g.add_card_to_battlefield(1, catalog::hydra_broodmaster()); // 7/7
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fort,
        ability_index: 0,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("shield up");
    drain_stack(&mut g);
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        big,
        1,
        Some(Target::Permanent(fort)),
    );
    g.resolve_effect(
        &crabomination::effect::Effect::DealDamage {
            to: crabomination::effect::Selector::Target(0),
            amount: crabomination::card::Value::Const(7),
        },
        &ctx,
    )
    .expect("swing damage");
    assert!(g.battlefield_find(fort).is_some(), "the shield soaked it");
}

/// Tormented Thoughts discards for the sacrificed creature's power.
#[test]
fn tormented_thoughts_discards_for_power() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::hydra_broodmaster()); // 7/7
    for _ in 0..5 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    cast(
        &mut g,
        catalog::tormented_thoughts(),
        Some(Target::Player(1)),
        2,
        &[(Color::Black, 1)],
    );
    assert_eq!(g.players[1].hand.len(), 0, "7 power emptied a 5-card hand");
}

/// Bearer of the Heavens wipes the board at the next end step, not on death.
#[test]
fn bearer_of_the_heavens_delays_the_wipe() {
    let mut g = main_phase();
    let bearer = g.add_card_to_battlefield(0, catalog::bearer_of_the_heavens());
    let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mut evs = Vec::new();
    g.destroy_permanent(bearer, false, &mut evs);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bystander).is_some(), "still alive during the main phase");
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bystander).is_none(), "the delayed wipe landed");
}

/// Wildfire Cerberus's monstrosity burns the opponent and their board.
#[test]
fn wildfire_cerberus_burns_on_monstrosity() {
    let mut g = main_phase();
    let dog = g.add_card_to_battlefield(0, catalog::wildfire_cerberus());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(dog);
    let life = g.players[1].life;
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dog,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("monstrosity 1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);
    assert!(g.battlefield_find(bear).is_none(), "2 damage killed the 2/2");
}

/// Swarmborn Giant gains reach only once monstrous.
#[test]
fn swarmborn_giant_reaches_when_monstrous() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::swarmborn_giant());
    g.clear_sickness(giant);
    assert!(!g.computed_permanent(giant).unwrap().keywords.contains(&Keyword::Reach));
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: giant,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("monstrosity 2");
    drain_stack(&mut g);
    assert!(g.computed_permanent(giant).unwrap().keywords.contains(&Keyword::Reach));
}

/// Starfall's rider only bites an enchantment creature's controller.
#[test]
fn starfall_punishes_enchantment_creatures() {
    let mut g = main_phase();
    let nyx = g.add_card_to_battlefield(1, catalog::whitewater_naiads()); // 4/4 enchantment
    let life = g.players[1].life;
    cast(&mut g, catalog::starfall(), Some(Target::Permanent(nyx)), 4, &[(Color::Red, 1)]);
    assert_eq!(g.players[1].life, life - 3, "the controller took 3 too");

    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    cast(&mut g, catalog::starfall(), Some(Target::Permanent(bear)), 4, &[(Color::Red, 1)]);
    assert_eq!(g.players[1].life, life, "a plain creature spares its controller");
}

/// Ritual of the Returned mints a Zombie with the exiled card's stats.
#[test]
fn ritual_of_the_returned_copies_stats() {
    let mut g = main_phase();
    let card = g.add_card_to_graveyard(0, catalog::hydra_broodmaster()); // 7/7
    cast(
        &mut g,
        catalog::ritual_of_the_returned(),
        Some(Target::Permanent(card)),
        3,
        &[(Color::Black, 1)],
    );
    let zombie = g.battlefield.iter().find(|c| c.definition.name == "Zombie").expect("Zombie");
    assert_eq!(g.computed_permanent(zombie.id).map(|c| (c.power, c.toughness)), Some((7, 7)));
}

/// Stat / keyword lines for the wave-2 bodies.
#[test]
fn jou2_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::agent_of_erebos, 2, 2, &[]),
        (catalog::dreadbringer_lampads, 4, 2, &[]),
        (catalog::forgeborn_oreads, 4, 2, &[]),
        (catalog::goldenhide_ox, 5, 4, &[]),
        (catalog::harvestguard_alseids, 2, 3, &[]),
        (catalog::humbler_of_mortals, 5, 5, &[]),
        (catalog::oakheart_dryads, 2, 3, &[]),
        (catalog::thassas_devourer, 2, 6, &[]),
        (catalog::thoughtrender_lamia, 5, 3, &[]),
        (catalog::whitewater_naiads, 4, 4, &[]),
        (catalog::riptide_chimera, 3, 4, &[Keyword::Flying]),
        (catalog::skyspear_cavalry, 2, 2, &[Keyword::Flying, Keyword::DoubleStrike]),
        (catalog::triton_shorestalker, 1, 1, &[Keyword::Unblockable]),
        (catalog::spawn_of_thraxes, 5, 5, &[Keyword::Flying]),
        (catalog::bearer_of_the_heavens, 10, 10, &[]),
        (catalog::war_wing_siren, 1, 3, &[Keyword::Flying]),
        (catalog::supply_line_cranes, 2, 4, &[Keyword::Flying]),
        (catalog::ravenous_leucrocota, 2, 4, &[Keyword::Vigilance]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
}

// ── Wave 3: bestow, Auras, and the rares ────────────────────────────────────

/// Battlefield Thaumaturge shaves {1} off an instant that targets a creature.
#[test]
fn battlefield_thaumaturge_discounts_creature_targeting_spells() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::battlefield_thaumaturge());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt = g.players[0].hand.iter().find(|c| c.id == bolt).unwrap().clone();
    assert_eq!(
        cost_reduction_for_spell(&g, 0, &bolt, Some(&Target::Permanent(bear))),
        1,
        "targets a creature"
    );
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, Some(&Target::Player(1))), 0);
}

/// Scourge of Fleets bounces exactly the creatures small enough for X.
#[test]
fn scourge_of_fleets_scales_with_islands() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::hydra_broodmaster()); // 7/7
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let kraken = g.add_card_to_battlefield(0, catalog::scourge_of_fleets());
    g.fire_self_etb_triggers(kraken, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(small).is_none(), "toughness 2 <= 3 Islands");
    assert!(g.battlefield_find(big).is_some(), "toughness 7 stays");
    assert!(g.battlefield_find(mine).is_some(), "only opponents' creatures");
}

/// Quarry Colossus buries a creature beneath the top X cards.
#[test]
fn quarry_colossus_buries_under_the_plains_count() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::plains());
    }
    for _ in 0..4 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let victim = g.add_card_to_battlefield(1, catalog::oreskos_swiftclaw());
    let colossus = g.add_card_to_battlefield(0, catalog::quarry_colossus());
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        colossus,
        0,
        Some(Target::Permanent(victim)),
    );
    g.resolve_effect(&catalog::quarry_colossus().triggered_abilities[0].effect, &ctx)
        .expect("bury it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(
        g.players[1].library.iter().position(|c| c.id == victim),
        Some(2),
        "third from the top"
    );
}

/// Sage of Hours cashes ten counters in for two extra turns.
#[test]
fn sage_of_hours_buys_a_turn_per_five_counters() {
    let mut g = main_phase();
    let sage = g.add_card_to_battlefield(0, catalog::sage_of_hours());
    g.clear_sickness(sage);
    g.battlefield_find_mut(sage).unwrap().counters.insert(CounterType::PlusOnePlusOne, 11);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sage,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].extra_turns, 2, "11 counters = two extra turns");
    assert_eq!(g.battlefield_find(sage).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Deserter's Quarters keeps its victim tapped while it stays tapped.
#[test]
fn deserters_quarters_locks_a_creature_down() {
    let mut g = main_phase();
    let quarters = g.add_card_to_battlefield(0, catalog::deserters_quarters());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(quarters);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: quarters,
        ability_index: 0,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("lock it");
    drain_stack(&mut g);
    let c = g.battlefield_find(victim).unwrap();
    assert!(c.tapped && c.untap_locked_by == Some(quarters));
}

/// Hypnotic Siren's bestow half steals the host.
#[test]
fn hypnotic_siren_bestow_steals_the_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let siren = g.add_card_to_hand(0, catalog::hypnotic_siren());
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastBestow {
        card_id: siren,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bestow");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Flying));
}

/// Armament of Nyx only arms an enchantment creature.
#[test]
fn armament_of_nyx_reads_the_host_types() {
    let mut g = main_phase();
    let nyx = g.add_card_to_battlefield(0, catalog::whitewater_naiads());
    let plain = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for host in [nyx, plain] {
        let aura = g.add_card_to_hand(0, catalog::armament_of_nyx());
        g.players[0].mana_pool.add_colorless(2);
        g.players[0].mana_pool.add(Color::White, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: aura,
            target: Some(Target::Permanent(host)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("enchant");
        drain_stack(&mut g);
    }
    assert!(g.computed_permanent(nyx).unwrap().keywords.contains(&Keyword::DoubleStrike));
    assert!(
        g.computed_permanent(plain).unwrap().keywords.contains(&Keyword::DealsNoCombatDamage),
        "a nonenchantment host deals no damage instead"
    );
}

/// Dictate of Karametra doubles every land tap, for every player.
#[test]
fn dictate_of_karametra_doubles_land_taps() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dictate_of_karametra());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None, mode: None,
    })
    .expect("tap");
    assert_eq!(g.players[1].mana_pool.total(), 2, "the opponent's land doubles too");
}

/// Stat / keyword lines for the wave-3 bodies.
#[test]
fn jou3_stat_lines() {
    let table: &[(fn() -> crabomination::card::CardDefinition, i32, i32, &[Keyword])] = &[
        (catalog::sightless_brawler, 3, 2, &[Keyword::CantAttackAlone]),
        (catalog::spirespine, 4, 1, &[Keyword::MustBlock]),
        (catalog::crystalline_nautilus, 4, 4, &[]),
        (catalog::hypnotic_siren, 1, 1, &[Keyword::Flying]),
        (catalog::battlefield_thaumaturge, 2, 1, &[]),
        (catalog::dakra_mystic, 1, 1, &[]),
        (catalog::daring_thief, 2, 3, &[]),
        (catalog::disciple_of_deceit, 1, 3, &[]),
        (catalog::nessian_game_warden, 4, 5, &[]),
        (
            catalog::prophetic_flamespeaker,
            1,
            3,
            &[Keyword::DoubleStrike, Keyword::Trample],
        ),
        (catalog::quarry_colossus, 5, 6, &[]),
        (catalog::sage_of_hours, 1, 1, &[]),
        (catalog::scourge_of_fleets, 6, 6, &[]),
        (catalog::stormchaser_chimera, 2, 3, &[Keyword::Flying]),
    ];
    for (f, p, t, kws) in table {
        let d = f();
        assert_eq!((d.power, d.toughness), (*p, *t), "{}", d.name);
        for kw in *kws {
            assert!(d.keywords.contains(kw), "{} lacks {:?}", d.name, kw);
        }
    }
    assert_eq!(catalog::ajani_mentor_of_heroes().base_loyalty, 4);
}
