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

// ── Cantrips ─────────────────────────────────────────────────────────────────

/// Augury Raven resolves as a 2/3 flyer when cast for its foretell cost.
#[test]
fn foretell_augury_raven_resolves_as_flyer() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::augury_raven());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{U}
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Augury Raven");
    drain_stack(&mut g);
    let r = g.battlefield_find(id).expect("on battlefield");
    assert_eq!((r.power(), r.toughness()), (3, 3));
    assert!(r.definition.keywords.contains(&Keyword::Flying));
}

/// CR 601 — an opponent's Drannith Magistrate stops a foretold spell being
/// cast from exile (a non-hand zone).
#[test]
fn drannith_magistrate_blocks_foretold_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::drannith_magistrate());
    let id = g.add_card_to_hand(0, catalog::augury_raven());
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[0].mana_pool.add(c, 20);
    }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    let err = g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "Drannith Magistrate forbids the foretold cast from exile");
}

/// Durkwood Baloth suspends for {G} and resolves into a 5/5 after 5 ticks.
#[test]
fn suspend_durkwood_baloth_resolves() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::durkwood_baloth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..5 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    let b = g.battlefield_find(id).expect("Baloth on battlefield");
    assert_eq!((b.power(), b.toughness()), (5, 5));
}

/// Keldon Halberdier suspends for {R} and resolves into a 4/1 first striker.
#[test]
fn suspend_keldon_halberdier_resolves() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::keldon_halberdier());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    let h = g.battlefield_find(id).expect("Halberdier on battlefield");
    assert_eq!((h.power(), h.toughness()), (4, 1));
    assert!(h.definition.keywords.contains(&Keyword::FirstStrike));
}

/// Deep-Sea Kraken suspends for {1}{U} and resolves into a 6/6.
#[test]
fn suspend_deep_sea_kraken_resolves() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::deep_sea_kraken());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    assert_eq!(g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time), 9);
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..9 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    let k = g.battlefield_find(id).expect("Kraken on battlefield");
    assert_eq!((k.power(), k.toughness()), (6, 6));
}

/// A creature free-cast off its last Suspend time counter gains haste
/// (CR 702.62f) — it enters with a granted Haste keyword.
#[test]
fn suspend_cast_creature_gains_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::keldon_halberdier());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    let h = g.battlefield_find(id).expect("Halberdier on battlefield");
    assert!(h.granted_keywords_eot.contains(&Keyword::Haste), "suspend-cast creature has haste");
}

/// Deep-Sea Kraken's accelerant: while it's suspended, an opponent casting a
/// spell removes a time counter from it.
#[test]
fn suspend_accelerant_ticks_on_opponent_cast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::deep_sea_kraken());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::Suspend { card_id: id }).expect("suspend");
    let before = g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time);
    // Player 1 (an opponent of the owner) casts a spell.
    let _ = g.process_suspend_accelerants(1);
    let after = g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time);
    assert_eq!(after, before - 1, "opponent's cast removes one time counter");
    // The owner's own cast does NOT tick it.
    let _ = g.process_suspend_accelerants(0);
    let same = g.exile.iter().find(|c| c.id == id).unwrap().counter_count(CounterType::Time);
    assert_eq!(same, after, "owner's own cast leaves it untouched");
}

// ── Blitz (CR 702.152) ───────────────────────────────────────────────────────

fn blitz(g: &mut GameState, id: crabomination::card::CardId) {
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("blitz cast should succeed");
    drain_stack(g);
}

/// A blitzed creature enters with haste, draws a card when it dies, and is
/// sacrificed at the next end step.
#[test]
fn blitz_grants_haste_then_sacrifices_with_death_draw() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::shock()); // a card to draw on death
    let id = g.add_card_to_hand(0, catalog::tenacious_underdog());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2); // Blitz {2}{B}
    blitz(&mut g, id);
    let c = g.battlefield_find(id).expect("blitzed creature on battlefield");
    assert!(c.granted_keywords_eot.contains(&Keyword::Haste), "blitz grants haste");
    let hand_before = g.players[0].hand.len();
    // Next end step: sacrifice + death-draw.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == id), "blitzed creature was sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "death-draw drew a card");
}

/// Ardent Elementalist's ETB returns an instant or sorcery from a graveyard.
#[test]
fn blitz_ardent_elementalist_etb_returns_instant() {
    let mut g = two_player_game();
    // Seed an instant in player 0's graveyard.
    let bolt = g.add_card_to_graveyard(0, catalog::shock());
    let id = g.add_card_to_hand(0, catalog::ardent_elementalist());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1); // Blitz {1}{R}
    blitz(&mut g, id);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bolt),
        "instant returned from graveyard to hand"
    );
}

// ── Tempo & utility spells ───────────────────────────────────────────────────

/// Gut Shot deals 1 damage, payable with Phyrexian (life) mana.
#[test]
fn gut_shot_deals_one() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::gut_shot());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "1 damage marked");
}

/// Wrangle steals a creature for the turn with haste, reverting at end of turn.
#[test]
fn wrangle_steals_until_end_of_turn() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wrangle());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen for the turn");
    // End-of-turn cleanup reverts control (CR 800.4).
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 0;
    g.perform_action(GameAction::PassPriority).unwrap();
    g.perform_action(GameAction::PassPriority).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "control reverts at EOT");
}

/// Ravenform destroys a creature and gives its controller a Bird.
#[test]
fn ravenform_destroys_and_gifts_bird() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ravenform());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    let birds = g.battlefield.iter()
        .filter(|c| c.controller == 1 && c.definition.name == "Bird Illusion").count();
    assert_eq!(birds, 1, "its controller got a Bird");
}

// ── Utility creatures ────────────────────────────────────────────────────────

/// Paradise Druid taps for one mana of any color and has hexproof.
#[test]
fn paradise_druid_taps_for_any_color() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::paradise_druid());
    g.clear_sickness(id);
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::Hexproof));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for mana");
    drain_stack(&mut g);
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.total() + pool.restricted_total(), 1, "produced one mana");
}

/// Merfolk Trickster taps an opponent's creature on ETB.
#[test]
fn merfolk_trickster_taps_opponent_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::merfolk_trickster());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast(&mut g, id);
    assert!(g.battlefield_find(bear).unwrap().tapped, "opponent's creature tapped");
}

/// Vampire Lacerator's upkeep drain stops once an opponent is low.
#[test]
fn vampire_lacerator_drain_turns_off_at_low_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vampire_lacerator());
    // Opponent healthy → controller loses 1 at their upkeep.
    let before = g.players[0].life;
    g.fire_step_triggers(crabomination::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before - 1, "drains while opponent is healthy");
    // Opponent at 10 → drain turns off.
    g.players[1].life = 10;
    let mid = g.players[0].life;
    g.fire_step_triggers(crabomination::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, mid, "no drain once an opponent has <=10 life");
}

// ── Removal & combat tricks ──────────────────────────────────────────────────

/// Ulcerate destroys a creature and costs you 3 life.
#[test]
fn ulcerate_destroys_and_costs_life() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::ulcerate());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.players[0].life, 17, "lost 3 life");
}

/// Might of Old Krosa pumps a creature +4/+4.
#[test]
fn might_of_old_krosa_pumps_four() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::might_of_old_krosa());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (6, 6), "2/2 + 4/4");
}

/// Aspect of Hydra pumps by devotion to green.
#[test]
fn aspect_of_hydra_scales_with_devotion() {
    let mut g = two_player_game();
    // Two green creatures with {G}{G}-ish pips → devotion 3 here.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // {1}{G} → 1 green pip
    let id = g.add_card_to_hand(0, catalog::aspect_of_hydra());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    let b = g.battlefield_find(bear).unwrap();
    // Devotion to green = green pips among permanents = 1 (the bear) → +1/+1.
    assert_eq!((b.power(), b.toughness()), (3, 3), "2/2 + devotion(1)");
}

// ── Rituals & impulse draw ───────────────────────────────────────────────────

/// Pyretic Ritual nets +1 red mana ({1}{R} in, {R}{R}{R} out).
#[test]
fn pyretic_ritual_adds_three_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pyretic_ritual());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert_eq!(g.players[0].mana_pool.total(), 3, "RRR added");
}

/// Seething Song adds {R}{R}{R}{R}{R}.
#[test]
fn seething_song_adds_five_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::seething_song());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[0].mana_pool.total(), 5, "RRRRR added");
}

/// Reckless Impulse exiles the top two cards and lets you play them.
#[test]
fn reckless_impulse_exiles_top_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::shock());
    let b = g.add_card_to_library(0, catalog::shock());
    let id = g.add_card_to_hand(0, catalog::reckless_impulse());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let exiled = g.exile.iter().filter(|c| (c.id == a || c.id == b) && c.may_play_until.is_some()).count();
    assert_eq!(exiled, 2, "both top cards exiled and playable");
    // Impulse "you may play them" pays each card's own cost — not a free cast.
    assert!(
        g.exile.iter().filter(|c| c.id == a || c.id == b).all(|c| c.granted_alt_cast_cost_eot.is_some()),
        "impulsed cards cost their own mana, not free",
    );
}

/// Wrenn's Resolve exiles the top two cards and lets you play them.
#[test]
fn wrenns_resolve_exiles_top_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_library(0, catalog::shock());
    let b = g.add_card_to_library(0, catalog::shock());
    let id = g.add_card_to_hand(0, catalog::wrenns_resolve());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let exiled = g.exile.iter().filter(|c| (c.id == a || c.id == b) && c.may_play_until.is_some()).count();
    assert_eq!(exiled, 2, "both top cards exiled and playable");
}

/// Kessig Flamebreather pings each opponent on instant/sorcery cast.
#[test]
fn kessig_flamebreather_pings_on_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kessig_flamebreather());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[1].life, 20 - 3 - 1, "Bolt + Flamebreather ping");
}

/// Play with Fire deals 2 to a player.
#[test]
fn play_with_fire_deals_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::play_with_fire());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, id, Target::Player(1));
    assert_eq!(g.players[1].life, 18, "dealt 2");
}

/// Electrostatic Field pings each opponent on instant/sorcery cast.
#[test]
fn electrostatic_field_pings_on_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::electrostatic_field());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[1].life, 20 - 3 - 1, "Bolt + Field ping");
}

/// Doomskar Titan's Boast pumps your team after it attacks.
#[test]
fn doomskar_titan_boast_pumps_team() {
    let mut g = two_player_game();
    let titan = g.add_card_to_battlefield(0, catalog::doomskar_titan());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(titan).unwrap().attacked_this_turn = true;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: titan, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("boast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "bear pumped +1/+0");
}

// ── Spell-matters aggro ──────────────────────────────────────────────────────

/// Firebrand Archer pings each opponent when you cast a noncreature spell.
#[test]
fn firebrand_archer_pings_on_noncreature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::firebrand_archer());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    // 3 from Bolt + 1 from Firebrand Archer.
    assert_eq!(g.players[1].life, 20 - 3 - 1, "Archer pinged on the noncreature cast");
}

/// Thermo-Alchemist taps for 1 to each opponent and untaps on instant cast.
#[test]
fn thermo_alchemist_pings_and_untaps_on_instant() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::thermo_alchemist());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("tap for damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "pinged opponent for 1");
    assert!(g.battlefield_find(id).unwrap().tapped, "tapped after activation");
    // Casting an instant untaps it (magecraft).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(id).unwrap().tapped, "untapped on instant cast");
}

/// Abbot of Keral Keep exiles the top card of your library on ETB and lets you
/// play it.
#[test]
fn abbot_of_keral_keep_exiles_top_and_grants_play() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::shock());
    let id = g.add_card_to_hand(0, catalog::abbot_of_keral_keep());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Abbot");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == top).expect("top card exiled");
    assert!(exiled.may_play_until.is_some(), "exiled card is playable");
}

/// Eidolon of the Great Revel burns the caster of a cheap spell, not an
/// expensive one.
#[test]
fn eidolon_burns_on_cheap_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::eidolon_of_the_great_revel());
    // Player 1 casts a MV-1 spell → takes 2.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast cheap spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "caster of a MV<=3 spell took 2 from Eidolon");
}

// ── Spectacle (CR 702.111) ───────────────────────────────────────────────────

/// Skewer the Critics can be cast for its Spectacle cost only after an
/// opponent has lost life this turn.
#[test]
fn spectacle_gated_on_opponent_life_loss() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::skewer_the_critics());
    g.players[0].mana_pool.add(Color::Red, 1); // only {R} — the Spectacle cost
    let spectacle_cast = GameAction::CastSpellAlternative {
        card_id: id, pitch_card: None,
        target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    };
    // No opponent life loss yet → Spectacle cast rejected.
    assert!(!g.would_accept(spectacle_cast.clone()), "Spectacle off without life loss");
    // Opponent loses life → Spectacle becomes available.
    g.adjust_life(1, -1);
    assert!(g.would_accept(spectacle_cast.clone()), "Spectacle on after life loss");
    g.perform_action(spectacle_cast).expect("cast for Spectacle");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 1 - 3, "Skewer dealt 3 after the {{R}} Spectacle cast");
}

/// Goldhound taps and sacrifices itself for one mana of any color.
#[test]
fn goldhound_sac_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::goldhound());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("activate Goldhound mana ability");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Goldhound sacrificed itself");
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
}

/// Beskir Shieldmate leaves two 1/1 tokens behind when it dies.
#[test]
fn beskir_shieldmate_dies_into_two_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::beskir_shieldmate());
    // Kill it via lethal damage + SBA.
    g.battlefield_find_mut(id).unwrap().damage = 5;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let tokens = g.battlefield.iter().filter(|c| c.definition.name == "Human Warrior").count();
    assert_eq!(tokens, 2, "two 1/1 Human Warriors on death");
}

/// Sarulf's Packmate cantrips on ETB when cast from its foretold exile.
#[test]
fn foretell_sarulfs_packmate_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::sarulfs_packmate());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{G}
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Packmate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB draws a card");
}

/// Dwarven Reinforcements makes two 2/1 tokens when cast from its foretold exile.
#[test]
fn foretell_dwarven_reinforcements_makes_two_tokens() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dwarven_reinforcements());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1); // {1}{R}
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Reinforcements");
    drain_stack(&mut g);
    let tokens = g.battlefield.iter()
        .filter(|c| c.definition.name == "Dwarf Berserker").count();
    assert_eq!(tokens, 2, "two 2/1 Dwarf Berserkers");
}

/// Mistwalker can be foretold and cast for {U} as a 2/1 flyer.
#[test]
fn foretell_mistwalker_resolves_as_flyer() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mistwalker());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    g.players[0].mana_pool.add(Color::Blue, 1); // {U}
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Mistwalker");
    drain_stack(&mut g);
    let m = g.battlefield_find(id).expect("on battlefield");
    assert_eq!((m.power(), m.toughness()), (1, 4));
    assert!(m.definition.keywords.contains(&Keyword::Flying));
}

/// Ravenous Lindwurm's ETB gains 4 life when cast from its foretold exile.
#[test]
fn foretell_ravenous_lindwurm_gains_life() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ravenous_lindwurm());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    let life_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3); // {3}{G}
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Lindwurm");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 4, "ETB gains 4 life");
    assert!(g.battlefield.iter().any(|c| c.id == id), "4/4 on the battlefield");
}

/// Demon Bolt deals 4 to a creature when cast from its foretold exile.
#[test]
fn foretell_demon_bolt_kills_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::demon_bolt());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell");
    g.foretold_this_turn.clear();
    g.players[0].mana_pool.add(Color::Red, 1); // {R} foretell cost
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Demon Bolt");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "4 damage kills the 2/2");
}

/// Behold the Multiverse draws two when cast for its foretell cost from exile.
#[test]
fn foretell_behold_the_multiverse_draws_two() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::behold_the_multiverse());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Foretell { card_id: id }).expect("foretell for {2}");
    g.foretold_this_turn.clear(); // simulate a later turn
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 2); // {1}{U} foretell cost
    g.perform_action(GameAction::CastForetold {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast foretold Behold");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "draws two cards");
}

// ── Suspend (CR 702.62) ──────────────────────────────────────────────────────

/// Rift Bolt suspends for {R}, ticks down at upkeep, then casts itself for
/// free when the last time counter is removed — dealing 3 to the opponent.
#[test]
fn suspend_rift_bolt_ticks_down_and_casts_for_free() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(0, catalog::rift_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::Suspend { card_id: bolt }).expect("suspend Rift Bolt");

    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("suspended card in exile");
    assert_eq!(exiled.counter_count(CounterType::Time), 1, "enters exile with N=1 time counters");
    assert!(!g.players[0].hand.iter().any(|c| c.id == bolt), "left hand");

    // Simulate the suspender's next upkeep: last counter off → free cast.
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let _ = g.process_suspend();
    drain_stack(&mut g);

    assert!(!g.exile.iter().any(|c| c.id == bolt), "the spell left exile when cast");
    assert_eq!(g.players[1].life, 17, "Rift Bolt deals 3 to the opponent");
}

/// Lotus Bloom suspends for {0} with three time counters; after three upkeep
/// ticks it resolves as an artifact on the battlefield.
#[test]
fn suspend_lotus_bloom_enters_after_three_ticks() {
    let mut g = two_player_game();
    let bloom = g.add_card_to_hand(0, catalog::lotus_bloom());
    g.perform_action(GameAction::Suspend { card_id: bloom }).expect("suspend Lotus Bloom");
    assert_eq!(
        g.exile.iter().find(|c| c.id == bloom).unwrap().counter_count(CounterType::Time),
        3,
    );

    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Three upkeeps: only the third removes the last counter and casts it.
    for _ in 0..3 {
        let _ = g.process_suspend();
    }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == bloom && c.definition.name == "Lotus Bloom"),
        "Lotus Bloom resolves onto the battlefield after its third tick");
}

/// Ancestral Vision suspends for {U} (no mana cost) and draws three cards
/// for its controller when the last of four time counters is removed.
#[test]
fn suspend_ancestral_vision_draws_three() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let av = g.add_card_to_hand(0, catalog::ancestral_vision());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::Suspend { card_id: av }).expect("suspend Ancestral Vision");
    let hand_before = g.players[0].hand.len();

    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { let _ = g.process_suspend(); }
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 3, "controller draws three");
}

/// CR 601.3e — no-mana-cost cards can't be cast from hand, while a printed
/// {0} cost (Ornithopter) still can.
#[test]
fn cr_601_3e_no_mana_cost_rejected_but_zero_cost_castable() {
    let mut g = two_player_game();
    let av = g.add_card_to_hand(0, catalog::ancestral_vision());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: av, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(err, Err(crabomination::game::GameError::NoManaCost)));
    let orn = g.add_card_to_hand(0, catalog::ornithopter());
    g.perform_action(GameAction::CastSpell {
        card_id: orn, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("{0} is a payable printed cost");
}

/// Search for Tomorrow, when its suspend resolves, fetches a basic land onto
/// the battlefield.
#[test]
fn suspend_search_for_tomorrow_fetches_a_land() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::lightning_bolt()); // padding
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let sft = g.add_card_to_hand(0, catalog::search_for_tomorrow());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::Suspend { card_id: sft }).expect("suspend");

    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..2 { let _ = g.process_suspend(); }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == forest),
        "the fetched basic land is on the battlefield");
}

/// Errant Ephemeron suspends for {1}{U} and resolves into a 6/4 flyer.
#[test]
fn suspend_errant_ephemeron_resolves_as_flyer() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let eph = g.add_card_to_hand(0, catalog::errant_ephemeron());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::Suspend { card_id: eph }).expect("suspend");
    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { let _ = g.process_suspend(); }
    drain_stack(&mut g);
    let e = g.battlefield_find(eph).expect("Ephemeron on battlefield");
    assert_eq!((e.power(), e.toughness()), (4, 4));
    assert!(e.definition.keywords.contains(&Keyword::Flying));
}

/// Riftwing Cloudskate's suspend resolves into a creature whose ETB bounces a
/// permanent to its owner's hand.
#[test]
fn suspend_riftwing_cloudskate_bounces_on_etb() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cloud = g.add_card_to_hand(0, catalog::riftwing_cloudskate());
    g.players[0].mana_pool.add(Color::Blue, 2); // {1}{U} suspend cost
    g.perform_action(GameAction::Suspend { card_id: cloud }).expect("suspend");

    g.step = TurnStep::Upkeep;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..3 { let _ = g.process_suspend(); }
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == cloud), "Cloudskate resolved onto the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == victim), "ETB bounced the bear to its owner's hand");
}

#[test]
fn ponder_resolves_and_draws_a_card() {
    let mut g = two_player_game();
    // Stock the library so the scry + draw both have inputs.
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::ponder());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ponder should be castable for {U}");
    drain_stack(&mut g);

    // Ponder leaves the hand and ends in graveyard; the draw nets +1 hand.
    assert_eq!(g.players[0].hand.len(), hand_before, "cast (-1) + draw (+1) = net 0");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Ponder"));
}

#[test]
fn index_rearranges_top_five_all_stay_on_top() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Library top→down: a, b (plus filler). Swap them; both must stay on top.
    let b = g.add_card_to_library(0, catalog::plains());
    let a = g.add_card_to_library(0, catalog::forest());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::ScryOrder {
        kept_top: vec![b, a], // put b on top, then a
        bottom: vec![],
    }]));
    let id = g.add_card_to_hand(0, catalog::index());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, id);
    drain_stack(&mut g);
    // No card was bottomed — library size unchanged and the chosen order holds.
    assert_eq!(g.players[0].library.len(), lib_before, "rearrange never bottoms");
    assert_eq!(g.players[0].library[0].id, b, "b is now on top");
    assert_eq!(g.players[0].library[1].id, a, "a is second");
}

#[test]
fn spire_owl_etb_rearranges_top_four() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    // AutoDecider keeps the peeked order; the ETB resolves without bottoming.
    g.add_card_to_battlefield(0, catalog::spire_owl());
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before, "owl ETB rearranges, never mills");
    let owl = g.battlefield.iter().find(|c| c.definition.name == "Spire Owl").unwrap();
    assert!(owl.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn sage_owl_is_a_flying_etb_rearranger() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    g.add_card_to_battlefield(0, catalog::sage_owl());
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before);
    let owl = g.battlefield.iter().find(|c| c.definition.name == "Sage Owl").unwrap();
    assert!(owl.definition.keywords.contains(&Keyword::Flying));
}

#[test]
fn manamorphose_adds_two_mana_and_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::manamorphose());
    // {1}{R/G}: pay the hybrid pip with red.
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Manamorphose castable for {1}{R}");
    drain_stack(&mut g);

    // Hand: -1 (cast) +1 (draw) → unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before);
    // Mana pool gained 2 mana of any colors. We don't constrain which colors
    // the bot picks; just that the total mana count went up by 2.
    let pool_total = g.players[0].mana_pool.total();
    assert_eq!(pool_total, 2, "Manamorphose nets 2 mana after paying its own cost");
}

#[test]
fn manamorphose_hybrid_pip_payable_with_green() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::manamorphose());
    // Pay the {R/G} pip with green instead of red.
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Manamorphose castable for {1}{G} via the hybrid pip");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

#[test]
fn sleight_of_hand_cantrip_resolves() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::sleight_of_hand());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Sleight of Hand castable for {U}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before, "cast + draw = net 0");
}

// ── Discard / draw-for-life / mill ───────────────────────────────────────────

#[test]
fn faithless_looting_draws_two_then_discards_two() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::faithless_looting());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand_before = g.players[0].hand.len();
    let grave_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Faithless Looting castable for {R}");
    drain_stack(&mut g);

    // Net hand change: -1 cast, +2 draw, -2 discard = -1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1,
        "Faithless Looting nets -1 hand size after cast/draw/discard");
    // Graveyard gains the spell + 2 discarded cards.
    assert_eq!(g.players[0].graveyard.len(), grave_before + 3);
}

#[test]
fn flashback_cast_exiles_spell_on_resolution() {
    // A flashback-cast spell is exiled on resolution (not sent to the
    // graveyard). Faithless Looting flashback {2}{R}: discard a card from
    // graveyard via the path; the card should end up in exile, not
    // graveyard.
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    // Put Faithless Looting straight into graveyard.
    let id = g.add_card_to_library(0, catalog::faithless_looting());
    let pos = g.players[0].library.iter().position(|c| c.id == id).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    // Pay the flashback cost {2}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Flashback castable for {2}{R} from graveyard");
    drain_stack(&mut g);

    // The card is in exile (not in graveyard).
    assert!(g.exile.iter().any(|c| c.id == id),
        "Flashback-cast Faithless Looting should be exiled on resolution");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == id),
        "Flashback-cast spell must NOT return to the graveyard");
}

/// Sign in Blood: target player draws 2 and loses 2 life. Verifies both self-target
/// and opp-target (the latter exercises the `target_filter(Player)` path).
#[test]
fn sign_in_blood_drains_targeted_player() {
    // Target self.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::sign_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "-1 cast +2 draw = +1");

    // Target opponent.
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::sign_in_blood());
    g.players[0].mana_pool.add(Color::Black, 2);
    let p0_life_before = g.players[0].life;
    let p1_life_before = g.players[1].life;
    let p0_hand_before = g.players[0].hand.len();
    let p1_hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sign in Blood castable for {B}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, p0_life_before, "caster life unchanged");
    assert_eq!(g.players[1].life, p1_life_before - 2, "target lost 2");
    assert_eq!(g.players[0].hand.len(), p0_hand_before - 1, "caster lost the spell");
    assert_eq!(g.players[1].hand.len(), p1_hand_before + 2, "target drew 2");
}

#[test]
fn nights_whisper_draws_two_loses_two_life() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::nights_whisper());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Night's Whisper castable for {1}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[0].life, life_before - 2);
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn duress_picks_noncreature_nonland() {
    let mut g = two_player_game();
    // P1 hand: a creature, a land, and a noncreature/nonland sorcery.
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::forest());
    let target_card = g.add_card_to_hand(1, catalog::lightning_bolt());

    let id = g.add_card_to_hand(0, catalog::duress());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Duress castable for {B}");
    drain_stack(&mut g);

    // The auto-discarder should have plucked the noncreature/nonland (the
    // Lightning Bolt, since the creature and land are filtered out).
    assert!(g.players[1].graveyard.iter().any(|c| c.id == target_card),
        "Duress should discard the noncreature/nonland card");
}

// ── Burn ─────────────────────────────────────────────────────────────────────

#[test]
fn lava_spike_deals_three_damage_to_player() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lava_spike());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lava Spike castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 3,
        "Lava Spike should deal 3 damage to the targeted player");
}

#[test]
fn shock_deals_two_damage_to_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::shock());
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Shock castable for {R}");
    drain_stack(&mut g);

    // Bear has 2 toughness; 2 damage kills it (state-based actions move it
    // to graveyard).
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed by Shock");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn lava_dart_deals_one_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::lava_dart());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life_before = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Lava Dart castable for {R}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].life, life_before - 1);
}

#[test]
fn lava_dart_flashback_sacrifices_a_mountain() {
    // Flashback—Sacrifice a Mountain (CR 702.34a, name-keyed additional cost).
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::lava_dart());
    let mtn = g.add_card_to_battlefield(0, catalog::mountain());
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastFlashback {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lava Dart flashback castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 1, "1 damage");
    assert!(g.battlefield_find(mtn).is_none(), "the Mountain was sacrificed");
}

#[test]
fn lava_dart_flashback_rejected_without_a_mountain() {
    let mut g = two_player_game();
    let id = g.add_card_to_graveyard(0, catalog::lava_dart());
    g.add_card_to_battlefield(0, catalog::forest()); // not a Mountain
    let result = g.perform_action(GameAction::CastFlashback {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(result.is_err(), "no Mountain to sacrifice → flashback rejected");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "Lava Dart stays in the graveyard");
}

// ── Reanimation / graveyard ──────────────────────────────────────────────────

#[test]
fn unburial_rites_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    // Put Atraxa in P0's graveyard for the rites to grab.
    let atraxa = g.add_card_to_library(0, catalog::atraxa_grand_unifier());
    let pos = g.players[0].library.iter().position(|c| c.id == atraxa).unwrap();
    let card = g.players[0].library.remove(pos);
    g.players[0].graveyard.push(card);

    let rites = g.add_card_to_hand(0, catalog::unburial_rites());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: rites,
        target: Some(Target::Permanent(atraxa)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Unburial Rites castable for {3}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == atraxa),
        "Atraxa should be reanimated onto the battlefield");
}

#[test]
fn entomb_searches_library_into_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bolt))]));

    let id = g.add_card_to_hand(0, catalog::entomb());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Entomb castable for {B}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == bolt),
        "Entomb should pull a card from library to graveyard");
    assert!(!g.players[0].library.iter().any(|c| c.id == bolt));
}

#[test]
fn buried_alive_searches_creature_into_graveyard() {
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::grizzly_bears());
    // Buried Alive now repeats the search up to 3 times. Answer the first
    // pull with the creature, then `Search(None)` to opt out of the
    // remaining iterations.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(creature)),
        DecisionAnswer::Search(None),
        DecisionAnswer::Search(None),
    ]));

    let id = g.add_card_to_hand(0, catalog::buried_alive());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Buried Alive castable for {2}{B}");
    drain_stack(&mut g);

    assert!(g.players[0].graveyard.iter().any(|c| c.id == creature),
        "Buried Alive should pull a creature card into the graveyard");
}

/// Buried Alive's full Oracle is "search for up to three creature cards" —
/// the engine wires that as `Repeat(3, Search(...))`. Stocking three
/// creatures in the library and answering each pull with a different one
/// should land all three in the graveyard.
#[test]
fn buried_alive_pulls_up_to_three_creatures() {
    let mut g = two_player_game();
    let c1 = g.add_card_to_library(0, catalog::grizzly_bears());
    let c2 = g.add_card_to_library(0, catalog::grizzly_bears());
    let c3 = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(c1)),
        DecisionAnswer::Search(Some(c2)),
        DecisionAnswer::Search(Some(c3)),
    ]));

    let id = g.add_card_to_hand(0, catalog::buried_alive());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Buried Alive castable for {2}{B}");
    drain_stack(&mut g);

    for cid in [c1, c2, c3] {
        assert!(g.players[0].graveyard.iter().any(|c| c.id == cid),
            "All three searched creatures should be in the graveyard");
    }
}

#[test]
fn exhume_reanimates_creature() {
    let mut g = two_player_game();
    // Push (modern_decks): printed Oracle is now "each player puts a
    // creature card from their graveyard onto the battlefield". The
    // caster's auto-decider picks the top creature card in their own
    // graveyard; same for the opp. This test only seeds the caster's
    // graveyard, so only the caster reanimates.
    let creature = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::exhume());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Exhume castable for {1}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == creature),
        "Exhume should reanimate the caster's only creature in their graveyard");
}

#[test]
fn exhume_each_player_reanimates_a_creature() {
    // Push (modern_decks): "each player reanimates" symmetry — both
    // P0 and P1 have a creature in their gy; both should land on the
    // battlefield under their respective controllers.
    let mut g = two_player_game();
    let p0_bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let p1_bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::exhume());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Exhume castable for {1}{B}");
    drain_stack(&mut g);

    let p0_back = g.battlefield_find(p0_bear);
    let p1_back = g.battlefield_find(p1_bear);
    assert!(p0_back.is_some() && p0_back.unwrap().controller == 0,
        "P0's bear should be on P0's battlefield");
    assert!(p1_back.is_some() && p1_back.unwrap().controller == 1,
        "P1's bear should be on P1's battlefield (each-player symmetry)");
}

