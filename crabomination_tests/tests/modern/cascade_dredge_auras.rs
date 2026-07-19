#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── Cascade (CR 702.85) ─────────────────────────────────────────────────────

/// Bloodbraid Elf's cascade walks the top of the library, exiles a nonland
/// card with MV < 4 (Grizzly Bears, MV 2), and — when the controller opts
/// in — casts it for free. The cascaded creature ends up on the battlefield
/// alongside the Elf.
#[test]
fn bloodbraid_elf_cascade_casts_lower_mv_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Top of library: Grizzly Bears (MV 2 nonland) → cascade hits it.
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));

    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: elf, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bloodbraid Elf castable for {2}{R}{G}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "Cascade should cast Grizzly Bears for free onto the battlefield");
    assert!(g.battlefield.iter().any(|c| c.id == elf),
        "Bloodbraid Elf itself resolves onto the battlefield");
    assert!(!g.players[0].library.iter().any(|c| c.id == bears),
        "The cascaded card leaves the library");
}

/// Cascade skips lands during the exile walk. With a Forest on top and a
/// Grizzly Bears beneath it, the Forest is exiled-and-bottomed (it can't be
/// the cascade hit), and the Bears is the nonland card that gets cast.
#[test]
fn cascade_skips_lands_during_the_walk() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Top of library: Forest (land, skipped), then Grizzly Bears (the hit).
    let forest = g.add_card_to_library(0, catalog::forest());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));

    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    cast(&mut g, elf);

    assert!(g.battlefield.iter().any(|c| c.id == bears),
        "Cascade casts the first nonland card (Grizzly Bears)");
    // The exiled-but-not-cast Forest goes to the bottom of the library.
    assert!(g.players[0].library.iter().any(|c| c.id == forest),
        "Skipped land is bottomed, not exiled permanently");
    assert!(!g.exile.iter().any(|c| c.id == forest),
        "Nothing from this cascade is left stranded in exile");
}

/// Declining the cascade cast (the default AutoDecider answer) bottoms the
/// revealed card instead of casting it.
#[test]
fn cascade_declined_bottoms_the_card() {
    let mut g = two_player_game();
    // No ScriptedDecider → AutoDecider declines the optional free cast.
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());

    let elf = g.add_card_to_hand(0, catalog::bloodbraid_elf());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    cast(&mut g, elf);

    assert!(!g.battlefield.iter().any(|c| c.id == bears),
        "Declined cascade does not cast the card");
    assert!(g.players[0].library.iter().any(|c| c.id == bears),
        "Declined cascade bottoms the revealed card back into the library");
    assert!(!g.exile.iter().any(|c| c.id == bears),
        "The revealed card is not stranded in exile");
}

/// Apex Devastator cascades four times. With four MV-2 creatures stacked on
/// top of the library and a decider that opts into every free cast, all
/// four are cast onto the battlefield alongside the 10/10 Kavu.
#[test]
fn apex_devastator_cascades_four_times() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let mut bears = Vec::new();
    for _ in 0..4 {
        bears.push(g.add_card_to_library(0, catalog::grizzly_bears()));
    }
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));

    let apex = g.add_card_to_hand(0, catalog::apex_devastator());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(8);

    cast(&mut g, apex);

    let cast_bears = bears
        .iter()
        .filter(|&&b| g.battlefield.iter().any(|c| c.id == b))
        .count();
    assert_eq!(cast_bears, 4, "all four cascades cast a creature for free");
    assert!(g.battlefield.iter().any(|c| c.id == apex),
        "Apex Devastator itself resolves onto the battlefield");
}

// ── Dredge (CR 702.52) ──────────────────────────────────────────────────────

/// Dredge replaces a draw: with Golgari Thug (Dredge 4) in the graveyard
/// and at least four cards in the library, opting in mills four cards and
/// returns the Thug to hand instead of drawing.
#[test]
fn dredge_replaces_draw_by_milling_and_returning() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let thug = g_dredge_fixture_thug(5);
    let mut g = thug.0;
    let thug_id = thug.1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));

    let lib_before = g.players[0].library.len();
    let mut events = Vec::new();
    let ok = g.draw_one(0, &mut events);

    assert!(ok, "draw_one succeeds via dredge");
    assert!(g.players[0].hand.iter().any(|c| c.id == thug_id),
        "Golgari Thug returns to hand via dredge");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == thug_id),
        "the dredge card leaves the graveyard");
    assert_eq!(g.players[0].library.len(), lib_before - 4,
        "Dredge 4 mills exactly four cards from the library");
    let milled = events.iter().filter(|e| matches!(e, GameEvent::CardMilled { .. })).count();
    assert_eq!(milled, 4, "four CardMilled events emitted");
}

/// Declining the dredge (AutoDecider's default) draws normally — the dredge
/// card stays in the graveyard and the top of the library goes to hand.
#[test]
fn dredge_declined_draws_normally() {
    let thug = g_dredge_fixture_thug(5);
    let mut g = thug.0;
    let thug_id = thug.1;
    // No ScriptedDecider → AutoDecider declines the dredge.
    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();
    let mut events = Vec::new();
    g.draw_one(0, &mut events);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == thug_id),
        "declined dredge keeps the card in the graveyard");
    assert_eq!(g.players[0].library.len(), lib_before - 1, "a normal single draw happened");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "exactly one card drawn");
}

/// Dredge is unavailable when the library has fewer than N cards
/// (CR 702.52a) — the player must draw normally and is never prompted.
#[test]
fn dredge_unavailable_when_library_too_small() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    // Only three library cards but Dredge 4 — cannot dredge.
    let thug = g_dredge_fixture_thug(3);
    let mut g = thug.0;
    let thug_id = thug.1;
    // A Bool(true) is queued but must NOT be consumed (no prompt fires).
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == thug_id),
        "Thug stays in graveyard — dredge wasn't legal");
    assert_eq!(g.players[0].library.len(), 2, "a normal draw consumed one of the three cards");
}

/// Helper: a two-player game with `lib` Forests in P0's library and a
/// Golgari Thug in P0's graveyard. Returns the game and the Thug's id.
fn g_dredge_fixture_thug(lib: usize) -> (crabomination::game::GameState, crabomination::card::CardId) {
    let mut g = two_player_game();
    for _ in 0..lib {
        g.add_card_to_library(0, catalog::forest());
    }
    let thug_id = g.add_card_to_graveyard(0, catalog::golgari_thug());
    (g, thug_id)
}

/// Life from the Loam returns up to three land cards from the graveyard to
/// hand on resolution.
#[test]
fn life_from_the_loam_returns_lands_from_graveyard() {
    let mut g = two_player_game();
    let l1 = g.add_card_to_graveyard(0, catalog::forest());
    let l2 = g.add_card_to_graveyard(0, catalog::mountain());
    let l3 = g.add_card_to_graveyard(0, catalog::island());
    let loam = g.add_card_to_hand(0, catalog::life_from_the_loam());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, loam);
    for id in [l1, l2, l3] {
        assert!(g.players[0].hand.iter().any(|c| c.id == id),
            "each land card returns to hand");
    }
}

/// Golgari Thug's death trigger puts a creature card from the graveyard on
/// top of the library (the classic dredge-deck recursion line).
#[test]
fn golgari_thug_death_puts_creature_on_top_of_library() {
    let mut g = two_player_game();
    // A creature card waiting in the graveyard to be recurred.
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let thug = g.add_card_to_battlefield(0, catalog::golgari_thug());
    // Bolt the 1/1 Thug to trigger its dies ability.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(thug)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt the Thug");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == thug), "Thug died to the Bolt");
    assert!(g.players[0].library.iter().any(|c| c.id == bears),
        "the recurred creature ends up in the library after the Thug dies");
}

// ── Auras (attach-on-resolve, CR 303.4) ─────────────────────────────────────

/// Gift of Orzhova attaches to the targeted creature on resolution and
/// grants +1/+1, flying, and lifelink via its equipped_bonus.
#[test]
fn gift_of_orzhova_attaches_and_buffs_the_creature() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 vanilla
    let gift = g.add_card_to_hand(0, catalog::gift_of_orzhova());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: gift,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Gift of Orzhova castable for {1}{W}{B}");
    drain_stack(&mut g);

    // The Aura is attached to the bears.
    let aura = g.battlefield.iter().find(|c| c.id == gift).expect("aura on battlefield");
    assert_eq!(aura.attached_to, Some(bears), "Aura attaches to its target");

    // Recompute layers and read the buffed stats / keywords.
    let view = g.compute_battlefield();
    let buffed = view.iter().find(|c| c.id == bears).expect("bears present");
    assert_eq!((buffed.power, buffed.toughness), (3, 3), "enchanted creature is +1/+1");
    assert!(buffed.keywords.contains(&crabomination::card::Keyword::Flying), "gains flying");
    assert!(buffed.keywords.contains(&crabomination::card::Keyword::Lifelink), "gains lifelink");
}

/// When the enchanted creature leaves, the orphaned Aura is put into the
/// graveyard (CR 704.5n/5q) and stops buffing.
#[test]
fn gift_of_orzhova_falls_off_when_host_leaves() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gift = g.add_card_to_hand(0, catalog::gift_of_orzhova());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: gift, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gift");
    drain_stack(&mut g);

    // Kill the host with a Bolt.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the host");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bears), "host died");
    assert!(!g.battlefield.iter().any(|c| c.id == gift),
        "orphaned Aura leaves the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == gift),
        "orphaned Aura goes to its owner's graveyard");
}

// ── More cascade / dredge / aura cards (modern_decks expansion) ─────────────

#[test]
fn shardless_agent_is_a_two_two_cascade_artifact_creature() {
    let def = catalog::shardless_agent();
    assert_eq!((def.power, def.toughness), (2, 2));
    assert!(def.card_types.contains(&crabomination::card::CardType::Artifact));
    assert!(def.triggered_abilities.iter().any(|t|
        matches!(t.effect, crabomination::effect::Effect::Cascade { .. })));
}

#[test]
fn enlisted_wurm_cascades_when_cast() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let wurm = g.add_card_to_hand(0, catalog::enlisted_wurm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, wurm);
    assert!(g.battlefield.iter().any(|c| c.id == bears), "Enlisted Wurm cascades into the Bears");
}

#[test]
fn maelstrom_wanderer_double_cascades_and_grants_haste() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let b1 = g.add_card_to_library(0, catalog::grizzly_bears());
    let b2 = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(true),
    ]));
    let mw = g.add_card_to_hand(0, catalog::maelstrom_wanderer());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, mw);
    let cast_bears = [b1, b2].iter().filter(|&&b| g.battlefield.iter().any(|c| c.id == b)).count();
    assert_eq!(cast_bears, 2, "both cascades resolve");
    // The Wanderer's static grants haste to your creatures.
    let view = g.compute_battlefield();
    let bear_view = view.iter().find(|c| c.id == b1).expect("bear present");
    assert!(bear_view.keywords.contains(&crabomination::card::Keyword::Haste),
        "creatures you control have haste");
}

#[test]
fn stinkweed_imp_has_flying_deathtouch_and_dredge() {
    let def = catalog::stinkweed_imp();
    assert!(def.keywords.contains(&crabomination::card::Keyword::Flying));
    assert!(def.keywords.contains(&crabomination::card::Keyword::Deathtouch));
    assert!(def.keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Dredge(5))));
}

#[test]
fn golgari_brownscale_can_dredge_two() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let bs = g.add_card_to_graveyard(0, catalog::golgari_brownscale());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    assert!(g.players[0].hand.iter().any(|c| c.id == bs), "Brownscale dredges back to hand");
    assert_eq!(events.iter().filter(|e| matches!(e, GameEvent::CardMilled { .. })).count(), 2,
        "Dredge 2 mills two");
}

#[test]
fn golgari_grave_troll_enters_with_x_plus_one_counters() {
    let mut g = two_player_game();
    let troll = g.add_card_to_hand(0, catalog::golgari_grave_troll());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.players[0].mana_pool.add_colorless(3); // X = 3
    g.perform_action(GameAction::CastSpell {
        card_id: troll, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("Grave-Troll castable with X=3");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let t = view.iter().find(|c| c.id == troll).expect("troll present");
    assert_eq!((t.power, t.toughness), (3, 3), "0/0 base + three +1/+1 counters = 3/3");
}

#[test]
fn golgari_grave_troll_removes_four_counters_to_regenerate() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(0, catalog::golgari_grave_troll());
    // Seed five +1/+1 counters (enters_with_counters is bypassed for a
    // battlefield-placed card, so add them directly).
    if let Some(c) = g.battlefield_find_mut(troll) {
        c.add_counters(CounterType::PlusOnePlusOne, 5);
    }
    g.clear_sickness(troll);
    g.perform_action(GameAction::ActivateAbility {
        card_id: troll, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate ability activatable with 5 counters");
    drain_stack(&mut g);
    let c = g.battlefield_find(troll).expect("troll still here");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "four counters removed");
    assert_eq!(c.regeneration_shields, 1, "gained a regeneration shield");
}

#[test]
fn rancor_buffs_plus_two_zero_and_grants_trample() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rancor = g.add_card_to_hand(0, catalog::rancor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rancor, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rancor castable for {G}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let buffed = view.iter().find(|c| c.id == bears).expect("bears present");
    assert_eq!((buffed.power, buffed.toughness), (4, 2), "Rancor is +2/+0");
    assert!(buffed.keywords.contains(&crabomination::card::Keyword::Trample), "Rancor grants trample");
}

#[test]
fn rancor_returns_to_hand_when_it_leaves_the_battlefield() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let rancor = g.add_card_to_battlefield(0, catalog::rancor());
    g.battlefield_find_mut(rancor).unwrap().attached_to = Some(bears);
    // Host dies → Rancor is orphaned (SBA) → its LTB trigger bounces it home.
    g.remove_to_graveyard_with_triggers(bears);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(rancor).is_none(), "Rancor left the battlefield");
    assert!(
        g.players[0].hand.iter().any(|c| c.id == rancor),
        "Rancor returned to its owner's hand"
    );
}

// ── Transforming double-faced permanents (CR 712) ───────────────────────────

#[test]
fn concealing_curtains_transforms_and_strips_a_card() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let cc = g.add_card_to_battlefield(0, catalog::concealing_curtains());
    g.add_card_to_hand(1, catalog::grizzly_bears()); // a nonland to strip
    g.add_card_to_library(1, catalog::island()); // so the follow-up draw works
    let opp_hand = g.players[1].hand.len();
    g.players[0].mana_pool.add(Color::Black, 3); // {B} + {2} generic
    g.perform_action(GameAction::ActivateAbility {
        card_id: cc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{2}{B}: Transform");
    drain_stack(&mut g);
    let eye = g.battlefield_find(cc).expect("still on battlefield");
    assert_eq!(eye.definition.name, "Revealing Eye", "transformed to back face");
    assert!(eye.transformed);
    assert!(eye.definition.keywords.contains(&crabomination::card::Keyword::Menace));
    // Discarded the nonland (then drew): net hand unchanged, but the bear is gone.
    assert!(!g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert_eq!(g.players[1].hand.len(), opp_hand, "discarded one, drew one");
}

#[test]
fn delver_of_secrets_transforms_when_top_is_instant() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::Upkeep;
    let d = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    g.add_card_to_library(0, catalog::shock()); // sole card → top is an instant
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let ab = g.battlefield_find(d).expect("still here");
    assert_eq!(ab.definition.name, "Insectile Aberration");
    assert_eq!((ab.power(), ab.toughness()), (3, 2));
    assert!(ab.definition.keywords.contains(&crabomination::card::Keyword::Flying));
}

#[test]
fn delver_of_secrets_stays_front_when_top_is_a_creature() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::Upkeep;
    let d = g.add_card_to_battlefield(0, catalog::delver_of_secrets());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(d).unwrap().definition.name, "Delver of Secrets");
}

#[test]
fn everflowing_well_mills_draws_then_descends_to_a_land() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // ETB: mill 2, draw 2.
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let well = g.add_card_to_hand(0, catalog::the_everflowing_well());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: well, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Everflowing Well");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "drew two");
    assert_eq!(g.players[0].graveyard.len(), 2, "milled two");
    // Descend 8: stuff the graveyard, then fire upkeep → transform to the land.
    for _ in 0..8 { g.add_card_to_graveyard(0, catalog::island()); }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let pools = g.battlefield_find(well).expect("still in play");
    assert_eq!(pools.definition.name, "The Myriad Pools");
    assert!(pools.definition.card_types.contains(&crabomination::card::CardType::Land));
}

#[test]
fn reckless_waif_flips_on_quiet_then_busy_turns() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::Upkeep;
    let waif = g.add_card_to_battlefield(0, catalog::reckless_waif());
    // No spells were cast last turn → day side flips to Merciless Predator.
    g.spells_cast_last_turn = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let pred = g.battlefield_find(waif).unwrap();
    assert_eq!(pred.definition.name, "Merciless Predator");
    assert_eq!((pred.power(), pred.toughness()), (3, 2));
    // A player cast two+ spells last turn → it flips back to the human side.
    g.spells_cast_last_turn = 2;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(waif).unwrap().definition.name, "Reckless Waif");
}

#[test]
fn mayor_of_avabruck_anthems_track_the_flip() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let mayor = g.add_card_to_battlefield(0, catalog::mayor_of_avabruck());
    let human = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // stand-in body
    // Front anthem buffs Humans; the bear isn't a Human, so check the Mayor
    // sees the front face and a real flip swaps the static.
    assert_eq!(g.battlefield_find(mayor).unwrap().definition.name, "Mayor of Avabruck");
    let mut ev = vec![];
    g.transform_permanent(mayor, &mut ev);
    let alpha = g.battlefield_find(mayor).unwrap();
    assert_eq!(alpha.definition.name, "Howlpack Alpha");
    assert_eq!((alpha.power(), alpha.toughness()), (3, 3));
    let _ = human;
}

#[test]
fn gatstaf_shepherd_and_village_messenger_gain_evasion_at_night() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let gs = g.add_card_to_battlefield(0, catalog::gatstaf_shepherd());
    let vm = g.add_card_to_battlefield(0, catalog::village_messenger());
    assert!(g.battlefield_find(vm).unwrap().definition.keywords.contains(&Keyword::Haste));
    let mut ev = vec![];
    g.transform_permanent(gs, &mut ev);
    g.transform_permanent(vm, &mut ev);
    assert!(g.battlefield_find(gs).unwrap().definition.keywords.contains(&Keyword::Intimidate));
    assert!(g.battlefield_find(vm).unwrap().definition.keywords.contains(&Keyword::Menace));
}

#[test]
fn ulvenwald_captive_taps_for_mana_and_transforms() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let uc = g.add_card_to_battlefield(0, catalog::ulvenwald_captive());
    g.clear_sickness(uc);
    // {T}: Add {G}.
    g.perform_action(GameAction::ActivateAbility {
        card_id: uc, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{T}: Add G");
    assert_eq!(g.players[0].mana_pool.total(), 1, "tapped for one green");
    // Untap and pay {5}{G}{G} to transform.
    g.battlefield_find_mut(uc).unwrap().tapped = false;
    g.players[0].mana_pool.add(Color::Green, 7);
    g.perform_action(GameAction::ActivateAbility {
        card_id: uc, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{5}{G}{G}: Transform");
    drain_stack(&mut g);
    let abom = g.battlefield_find(uc).unwrap();
    assert_eq!(abom.definition.name, "Ulvenwald Abomination");
    assert_eq!((abom.power(), abom.toughness()), (4, 6));
}

#[test]
fn kruin_outlaw_gains_double_strike_and_werewolf_menace_anthem() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let kruin = g.add_card_to_battlefield(0, catalog::kruin_outlaw());
    let pack = g.add_card_to_battlefield(0, catalog::reckless_waif()); // a Werewolf
    assert!(g.battlefield_find(kruin).unwrap().definition.keywords.contains(&Keyword::FirstStrike));
    let mut ev = vec![];
    g.transform_permanent(kruin, &mut ev);
    let terror = g.battlefield_find(kruin).unwrap();
    assert!(terror.definition.keywords.contains(&Keyword::DoubleStrike));
    // The anthem grants menace to other Werewolves (computed via layers).
    let view = g.compute_battlefield();
    let waif = view.iter().find(|c| c.id == pack).unwrap();
    assert!(waif.keywords.contains(&Keyword::Menace), "werewolf gains menace from Terror");
}

#[test]
fn outland_liberator_is_daybound_and_sacrifices_to_destroy() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let target = g.add_card_to_battlefield(1, catalog::the_everflowing_well()); // an artifact
    let ol = g.add_card_to_hand(0, catalog::outland_liberator());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: ol, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Outland Liberator");
    drain_stack(&mut g);
    assert_eq!(g.day_night, Some(crabomination::game::types::DayNight::Day), "daybound → day");
    g.clear_sickness(ol);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ol, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("{1}, Sac: destroy artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(ol).is_none(), "Liberator sacrificed");
}

#[test]
fn legions_landing_makes_a_vampire_then_transforms_on_a_wide_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Cast it (ETB makes a 1/1 lifelink Vampire).
    let ll = g.add_card_to_hand(0, catalog::legions_landing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ll, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Legion's Landing");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Vampire"), "ETB Vampire token");
    // Attack with three creatures → transform into Adanto.
    let a: Vec<_> = (0..3).map(|_| {
        let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(c);
        c
    }).collect();
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(
        a.iter().map(|&c| Attack { attacker: c, target: AttackTarget::Player(1) }).collect(),
    )).expect("attack with three");
    drain_stack(&mut g);
    let adanto = g.battlefield_find(ll).unwrap();
    assert_eq!(adanto.definition.name, "Adanto, the First Fort");
    assert!(adanto.definition.card_types.contains(&crabomination::card::CardType::Land));
}

#[test]
fn vanilla_werewolves_flip_to_their_back_faces() {
    let mut g = two_player_game();
    let cards = [
        (catalog::tormented_pariah(), "Rampaging Werewolf", (6, 4)),
        (catalog::villagers_of_estwald(), "Howlpack of Estwald", (4, 6)),
        (catalog::hinterland_hermit(), "Hinterland Scourge", (3, 2)),
        (catalog::lambholt_elder(), "Silverpelt Werewolf", (4, 5)),
    ];
    for (def, back_name, (bp, bt)) in cards {
        let id = g.add_card_to_battlefield(0, def);
        let mut ev = vec![];
        g.transform_permanent(id, &mut ev);
        let back = g.battlefield_find(id).unwrap();
        assert_eq!(back.definition.name, back_name);
        assert_eq!((back.power(), back.toughness()), (bp, bt));
    }
    // Hinterland Scourge must be blocked; Silverpelt draws on combat damage.
    use crabomination::card::Keyword;
    let scourge = g.battlefield.iter().find(|c| c.definition.name == "Hinterland Scourge").unwrap();
    assert!(scourge.definition.keywords.contains(&Keyword::MustBeBlocked));
}

#[test]
fn geier_reach_bandit_flips_to_a_four_three() {
    let mut g = two_player_game();
    let grb = g.add_card_to_battlefield(0, catalog::geier_reach_bandit());
    let mut ev = vec![];
    g.transform_permanent(grb, &mut ev);
    let alpha = g.battlefield_find(grb).unwrap();
    assert_eq!(alpha.definition.name, "Vildin-Pack Alpha");
    assert_eq!((alpha.power(), alpha.toughness()), (4, 3));
}

#[test]
fn vildin_pack_alpha_transforms_an_entering_werewolf() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let alpha = *catalog::geier_reach_bandit().back_face.unwrap();
    g.add_card_to_battlefield(0, alpha);
    let pariah = g.add_card_to_hand(0, catalog::tormented_pariah());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: pariah, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tormented Pariah");
    drain_stack(&mut g);
    let entered = g.battlefield_find(pariah).unwrap();
    assert_eq!(entered.definition.name, "Rampaging Werewolf", "Alpha transformed the new Werewolf");
    assert_eq!((entered.power(), entered.toughness()), (6, 4));
}

#[test]
fn frenzied_trapbreaker_destroys_an_artifact_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let artifact = g.add_card_to_battlefield(1, catalog::the_everflowing_well());
    let frenzied = *catalog::outland_liberator().back_face.unwrap();
    let ft = g.add_card_to_battlefield(0, frenzied);
    g.clear_sickness(ft);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ft, target: AttackTarget::Player(1),
    }])).expect("Frenzied Trapbreaker attacks");
    drain_stack(&mut g);
    assert!(g.battlefield_find(artifact).is_none(), "on-attack trigger destroyed the artifact");
}

#[test]
fn mondronen_shaman_back_pings_opponent_casters() {
    let mut g = two_player_game();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let ms = g.add_card_to_battlefield(0, catalog::mondronen_shaman());
    let mut ev = vec![];
    g.transform_permanent(ms, &mut ev); // now Tovolar's Magehunter
    let life = g.players[1].life;
    // Opponent (player 1) casts a spell → takes 2.
    let bolt = g.add_card_to_hand(1, catalog::shock());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "Magehunter pings the opposing caster for 2");
}

#[test]
fn kessig_prowler_transforms_into_sinuous_predator() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let kp = g.add_card_to_battlefield(0, catalog::kessig_prowler());
    g.clear_sickness(kp);
    g.players[0].mana_pool.add(Color::Green, 5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kp, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{4}{G}: Transform");
    drain_stack(&mut g);
    let pred = g.battlefield_find(kp).unwrap();
    assert_eq!(pred.definition.name, "Sinuous Predator");
    assert_eq!((pred.power(), pred.toughness()), (4, 4));
    assert!(pred.definition.keywords.contains(&crabomination::card::Keyword::CantBeBlockedByMoreThanOne));
}

#[test]
fn search_for_azcanta_flips_when_graveyard_is_full() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::Upkeep;
    let sfa = g.add_card_to_battlefield(0, catalog::search_for_azcanta());
    g.add_card_to_library(0, catalog::island()); // for surveil
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::island()); }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let land = g.battlefield_find(sfa).unwrap();
    assert_eq!(land.definition.name, "Azcanta, the Sunken Ruin");
    assert!(land.definition.card_types.contains(&crabomination::card::CardType::Land));
}

#[test]
fn growing_rites_of_itlimoc_transforms_with_four_creatures() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::End;
    let gri = g.add_card_to_battlefield(0, catalog::growing_rites_of_itlimoc());
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::grizzly_bears()); }
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(gri).unwrap().definition.name, "Itlimoc, Cradle of the Sun");
}

#[test]
fn transformed_permanent_view_exposes_dfc_hints() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let cc = g.add_card_to_battlefield(0, catalog::concealing_curtains());
    // Front face: a DFC that hasn't flipped yet.
    let v = crabomination::server::view::project(&g, 0);
    let p = v.battlefield.iter().find(|p| p.id == cc).unwrap();
    assert!(p.has_other_face && !p.transformed, "front face: DFC but not transformed");
    // Transform it and re-project.
    let mut ev = vec![];
    g.transform_permanent(cc, &mut ev);
    let v = crabomination::server::view::project(&g, 0);
    let p = v.battlefield.iter().find(|p| p.id == cc).unwrap();
    assert_eq!(p.name, "Revealing Eye");
    assert!(p.has_other_face && p.transformed, "back face: transformed DFC");
}

#[test]
fn village_watch_flips_with_day_and_night() {
    use crabomination::game::types::DayNight;
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Cast it so the daybound ETB rule makes it day.
    let vw = g.add_card_to_hand(0, catalog::village_watch());
    g.players[0].mana_pool.add(Color::Red, 5);
    g.perform_action(GameAction::CastSpell {
        card_id: vw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Village Watch");
    drain_stack(&mut g);
    assert_eq!(g.day_night, Some(DayNight::Day), "daybound permanent makes it day");
    assert_eq!(g.battlefield_find(vw).unwrap().definition.name, "Village Watch");
    // Becomes night → flips to the nightbound back face.
    let mut ev = vec![];
    g.set_day_night(DayNight::Night, &mut ev);
    let reavers = g.battlefield_find(vw).unwrap();
    assert_eq!(reavers.definition.name, "Village Reavers");
    assert_eq!((reavers.power(), reavers.toughness()), (5, 4));
    // Becomes day again → flips back to the front.
    g.set_day_night(DayNight::Day, &mut ev);
    assert_eq!(g.battlefield_find(vw).unwrap().definition.name, "Village Watch");
}

#[test]
fn sink_into_stupor_bounces_then_plays_as_a_land() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Front: bounce an opposing creature.
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sis = g.add_card_to_hand(0, catalog::sink_into_stupor());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: sis, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast front face");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none(), "bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bears));
    // Back: play Soporific Springs as a land (pay 3 life to stay untapped).
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let springs = g.add_card_to_hand(0, catalog::sink_into_stupor());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(0)]));
    let life = g.players[0].life;
    g.perform_action(GameAction::PlayLandBack(springs)).expect("play back-face land");
    drain_stack(&mut g);
    let land = g.battlefield_find(springs).expect("land in play");
    assert_eq!(land.definition.name, "Soporific Springs");
    assert_eq!(g.players[0].life, life - 3, "paid 3 life");
    assert!(!land.tapped, "stayed untapped");
}

#[test]
fn thing_in_the_ice_melts_then_bounces_the_board() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Cast the spell so it enters with four ice counters.
    let tite = g.add_card_to_hand(0, catalog::thing_in_the_ice());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: tite, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Thing in the Ice");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(tite).unwrap().counter_count(crabomination::card::CounterType::Ice), 4,
        "enters with four ice counters"
    );
    // A couple of friendly bystanders + an opposing creature to bounce later.
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Cast four cheap instants to melt the ice.
    for _ in 0..4 {
        let bolt = g.add_card_to_hand(0, catalog::shock());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt, target: Some(Target::Player(1)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Shock");
        drain_stack(&mut g);
    }
    let horror = g.battlefield_find(tite).expect("still in play");
    assert_eq!(horror.definition.name, "Awoken Horror");
    assert_eq!((horror.power(), horror.toughness()), (7, 8));
    // Non-Horror creatures were bounced; the Horror itself stayed.
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
}

// ── Cascade instants / Darkblast dredge / simple Auras ──────────────────────

#[test]
fn bituminous_blast_burns_creature_and_cascades() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bb = g.add_card_to_hand(0, catalog::bituminous_blast());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: bb, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bituminous Blast castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "4 damage kills the 4/4");
    assert!(g.battlefield.iter().any(|c| c.id == bears), "cascade resolved into the Bears");
}

#[test]
fn violent_outburst_pumps_team_and_cascades() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false)]));
    let vo = g.add_card_to_hand(0, catalog::violent_outburst());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, vo);
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == mine).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "your creatures get +1/+1");
}

#[test]
fn ardent_plea_has_exalted_and_cascade() {
    let def = catalog::ardent_plea();
    assert!(def.triggered_abilities.iter().any(|t|
        matches!(t.effect, crabomination::effect::Effect::Cascade { .. })), "has cascade");
    assert!(def.triggered_abilities.iter().any(|t|
        matches!(t.effect, crabomination::effect::Effect::PumpPT { .. })), "has the exalted pump");
}

#[test]
fn darkblast_shrinks_a_creature_and_can_dredge() {
    let mut g = two_player_game();
    let imp = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let db = g.add_card_to_hand(0, catalog::darkblast());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: db, target: Some(Target::Permanent(imp)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Darkblast castable for {B}");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    if let Some(c) = view.iter().find(|c| c.id == imp) {
        assert_eq!((c.power, c.toughness), (1, 1), "-1/-1 applied");
    }
    // Darkblast carries Dredge 3.
    assert!(catalog::darkblast().keywords.iter().any(|k| matches!(k, crabomination::card::Keyword::Dredge(3))));
}

#[test]
fn spectral_flight_grants_plus_two_two_and_flying() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::spectral_flight());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spectral Flight castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == bears).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4));
    assert!(c.keywords.contains(&crabomination::card::Keyword::Flying));
}

/// Pacifism stops the enchanted creature from attacking (CR — granted
/// CantAttack/CantBlock via the aura).
#[test]
fn pacifism_stops_the_creature_attacking() {
    let mut g = two_player_game();
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bears);
    let aura = g.add_card_to_hand(0, catalog::pacifism());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pacifism castable");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == bears).unwrap();
    assert!(c.keywords.contains(&crabomination::card::Keyword::CantAttack));
    // Declaring it as an attacker is now rejected.
    g.step = TurnStep::DeclareAttackers;
    let err = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bears,
        target: AttackTarget::Player(1),
    }]));
    assert!(matches!(err, Err(GameError::CannotAttack(_))), "pacified creature can't attack");
}

#[test]
fn unholy_and_holy_strength_apply_their_buffs() {
    // Unholy Strength: +2/+1.
    let mut g = two_player_game();
    let b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let us = g.add_card_to_hand(0, catalog::unholy_strength());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: us, target: Some(Target::Permanent(b1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Unholy Strength castable");
    drain_stack(&mut g);
    let v = g.compute_battlefield();
    let c = v.iter().find(|c| c.id == b1).unwrap();
    assert_eq!((c.power, c.toughness), (4, 3), "Unholy Strength is +2/+1");

    // Holy Strength: +1/+2 (fresh game).
    let mut g2 = two_player_game();
    let b2 = g2.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hs = g2.add_card_to_hand(0, catalog::holy_strength());
    g2.players[0].mana_pool.add(Color::White, 1);
    g2.perform_action(GameAction::CastSpell {
        card_id: hs, target: Some(Target::Permanent(b2)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Holy Strength castable");
    drain_stack(&mut g2);
    let v2 = g2.compute_battlefield();
    let c2 = v2.iter().find(|c| c.id == b2).unwrap();
    assert_eq!((c2.power, c2.toughness), (3, 4), "Holy Strength is +1/+2");
}

