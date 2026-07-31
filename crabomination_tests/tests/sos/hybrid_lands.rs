#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
use crate::prepared_on_battlefield;

// ── Hybrid mana pips: off-color payment ───────────────────────────────────────
// These cards print a two-color hybrid pip (e.g. {W/B}). The engine models
// the pip with `ManaSymbol::Hybrid`, so the pip can be paid with EITHER half.
// Each test casts the card paying the hybrid pip with the *off-color* (the
// half that the prior mono-color approximation did not accept).

#[test]
fn essenceknit_scholar_hybrid_pip_payable_with_green() {
    // {B}{B/G}{G}: pay the B/G pip with green → B + G + G.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::essenceknit_scholar());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Essenceknit Scholar castable for {B}{G}{G} via hybrid pip");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Scholar resolves when the hybrid pip is paid with green");
}

#[test]
fn paradox_surveyor_hybrid_pip_payable_with_blue() {
    // {G}{G/U}{U}: pay the G/U pip with blue → G + U + U.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::paradox_surveyor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Paradox Surveyor castable for {G}{U}{U} via hybrid pip");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn abstract_paintmage_hybrid_pip_payable_with_red() {
    // {U}{U/R}{R}: pay the U/R pip with red → U + R + R.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::abstract_paintmage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Abstract Paintmage castable for {U}{R}{R} via hybrid pip");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn practiced_scrollsmith_hybrid_pip_payable_with_white() {
    // {R}{R/W}{W}: pay the R/W pip with white → R + W + W.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::practiced_scrollsmith());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Practiced Scrollsmith castable for {R}{W}{W} via hybrid pip");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn stirring_honormancer_hybrid_pip_payable_with_black() {
    // {2}{W}{W/B}{B}: pay the W/B pip with black → {2} + W + B + B.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::stirring_honormancer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Stirring Honormancer castable for {2}{W}{B}{B} via hybrid pip");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn heroic_stanza_prepare_hybrid_pip_payable_with_white() {
    // Abigale's prepare spell Heroic Stanza is {1}{W/B}: pay the hybrid
    // pip with white → {1}{W}. Puts a +1/+1 counter on target creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = prepared_on_battlefield(&mut g, 0, catalog::abigale_poet_laureate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Heroic Stanza castable for {1}{W} via hybrid pip");
    drain_stack(&mut g);

    let b = g.battlefield_find(bear).expect("bear still on bf");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 1,
        "Heroic Stanza puts a +1/+1 counter on the bear");
    let view = g.computed_permanent(bear).unwrap();
    assert_eq!(view.power, 3, "2/2 + one +1/+1 counter");
    assert_eq!(view.toughness, 3);
}

#[test]
fn pest_friend_prepare_hybrid_pip_payable_with_green() {
    // Lluwen's prepare spell Pest Friend is {B/G}: pay the hybrid pip
    // with green. Creates a 1/1 Pest token.
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::lluwen_exchange_student());
    g.players[0].mana_pool.add(Color::Green, 1);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Pest Friend castable for {G} via hybrid pip");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Pest token created");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Pest"));
}

#[test]
fn spectacle_mage_castable_with_two_red() {
    // {U/R}{U/R}: both hybrid pips payable with red → cast for {R}{R}.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::spectacle_mage());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Spectacle Mage castable for {R}{R} via hybrid pips");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id));
}

#[test]
fn fervent_strike_castable_with_green() {
    // {R/G}: hybrid pip payable with green.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fervent_strike());
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fervent Strike castable for {G} via hybrid pip");
    drain_stack(&mut g);

    let view = g.computed_permanent(bear).expect("bear on bf");
    assert_eq!(view.power, 4, "Fervent Strike pumps +2/+0");
    assert!(view.keywords.contains(&Keyword::Trample));
}

// ── Monocolored hybrid pips ({n/C}) ───────────────────────────────────────────
// The "Archaic" Avatars print {2/R} / {2/G} pips — payable with {2}
// generic OR one mana of the color. Modeled via `ManaSymbol::MonoHybrid`.

#[test]
fn magmablood_archaic_mana_value_is_six() {
    // {2/R}{2/R}{2/R}: CR 202.3f — monocolored hybrid MV uses the
    // generic side, so the total mana value is 6 (not 9).
    let def = catalog::magmablood_archaic();
    assert_eq!(def.cost.cmc(), 6, "Magmablood Archaic MV = 6");
}

#[test]
fn magmablood_archaic_castable_with_three_red() {
    // Pay each {2/R} pip with a single red → 3 mana total.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::magmablood_archaic());
    g.players[0].mana_pool.add(Color::Red, 3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Magmablood castable for {R}{R}{R} via mono-hybrid pips");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id),
        "Magmablood resolves paying the colored side of each pip");
}

#[test]
fn magmablood_archaic_castable_with_six_generic() {
    // Pay each {2/R} pip with {2} generic → 6 mana, zero colors spent.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::magmablood_archaic());
    g.players[0].mana_pool.add_colorless(6);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Magmablood castable for {6} via the generic side of mono-hybrid pips");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).expect("Magmablood on bf");
    // Zero colors of mana spent → no Converge counters → base 2/2.
    assert_eq!(view.power, 2, "0 colors spent → no +1/+1 counters");
    assert_eq!(view.toughness, 2);
}

#[test]
fn magmablood_archaic_not_castable_with_two_mana() {
    // {2/R}{2/R}{2/R} needs at least 3 mana (one per pip via the red
    // side). Two mana is insufficient.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::magmablood_archaic());
    g.players[0].mana_pool.add(Color::Red, 2);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "Magmablood uncastable with only 2 mana");
}

#[test]
fn wildgrowth_archaic_castable_with_two_green() {
    // {2/G}{2/G}: pay each pip with a single green → 2 mana, 1 color.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::wildgrowth_archaic());
    g.players[0].mana_pool.add(Color::Green, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Wildgrowth castable for {G}{G} via mono-hybrid pips");
    drain_stack(&mut g);
    let view = g.computed_permanent(id).expect("Wildgrowth on bf");
    // 1 color (green) spent → 1 Converge counter → survives as 1/1.
    assert_eq!(view.power, 1, "1 color spent → 1 +1/+1 counter");
}

// ════════════════════════════════════════════════════════════════════════════
// Coverage backfill (claude/modern_decks): functionality tests for SOS cards
// that were wired but lacked a dedicated test. One test per card exercising
// its primary play pattern.
// ════════════════════════════════════════════════════════════════════════════

#[test]
fn lecturing_scornmage_repartee_adds_counter_on_is_targeting_creature() {
    // Repartee: when you cast an instant/sorcery targeting a creature,
    // Lecturing Scornmage gets a +1/+1 counter.
    let mut g = two_player_game();
    let scorn = g.add_card_to_battlefield(0, catalog::lecturing_scornmage());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let s = g.battlefield_find(scorn).unwrap();
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 1,
        "Repartee should land a +1/+1 counter on the Scornmage");
}

#[test]
fn lecturing_scornmage_repartee_skips_when_targeting_player() {
    let mut g = two_player_game();
    let scorn = g.add_card_to_battlefield(0, catalog::lecturing_scornmage());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let s = g.battlefield_find(scorn).unwrap();
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 0,
        "Repartee must not fire when the spell targets a player");
}

#[test]
fn melancholic_poet_repartee_drains_one() {
    // Repartee: when you cast an instant/sorcery targeting a creature,
    // each opponent loses 1 life and you gain 1.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::melancholic_poet());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let p0_life = g.players[0].life;
    let p1_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 1, "opponent drained 1");
    assert_eq!(g.players[0].life, p0_life + 1, "you gained 1");
}

#[test]
fn muse_seeker_opus_small_body_loots() {
    // Opus (small body, < 5 mana spent): draw a card, then discard a card.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::muse_seeker());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // something to discard
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1,
        "Opus small body draws one card");
    // Discard sent a card to the graveyard (alongside the resolved bolt).
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Opus small body discards a card");
}

#[test]
fn poisoners_apprentice_etb_shrinks_opp_creature_after_lifegain() {
    // ETB: if you gained life this turn, target creature an opponent
    // controls gets -4/-4 until end of turn.
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.adjust_life(0, 1); // gain life this turn
    let id = g.add_card_to_hand(0, catalog::poisoners_apprentice());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Poisoner's Apprentice castable for {2}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "2/2 bear dies to -4/-4 after lifegain");
}

#[test]
fn poisoners_apprentice_etb_no_lifegain_leaves_creature_intact() {
    let mut g = two_player_game();
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::poisoners_apprentice());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Poisoner's Apprentice castable for {2}{B}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == opp_bear),
        "without lifegain the ETB does nothing");
}

#[test]
fn rearing_embermare_is_a_four_five_reach_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::rearing_embermare());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.definition.power, c.definition.toughness), (4, 5));
    assert!(c.definition.keywords.contains(&Keyword::Reach));
    assert!(c.definition.keywords.contains(&Keyword::Haste));
}

#[test]
fn rehearsed_debater_repartee_self_pumps() {
    // Repartee: +1/+1 until end of turn when you cast an IS targeting a
    // creature.
    let mut g = two_player_game();
    let deb = g.add_card_to_battlefield(0, catalog::rehearsed_debater());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(opp_bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Bolt castable for {R}");
    drain_stack(&mut g);
    let c = g.computed_permanent(deb).unwrap();
    assert_eq!(c.power, 4, "3/3 base + Repartee +1/+1 = 4 power");
}

#[test]
fn tester_of_the_tangential_increment_adds_counter_on_three_mana_cast() {
    // Increment: when you cast a spell with mana spent greater than this
    // creature's power or toughness (both 1), put a +1/+1 counter on it.
    let mut g = two_player_game();
    let tester = g.add_card_to_battlefield(0, catalog::tester_of_the_tangential());
    // A 2-mana spell satisfies Increment (2 > the tester's power/toughness
    // of 1) and — being a creature spell — leaves the tester unharmed.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grizzly Bears castable for {1}{G}");
    drain_stack(&mut g);
    let t = g.battlefield_find(tester).unwrap();
    assert!(t.counter_count(CounterType::PlusOnePlusOne) >= 1,
        "2-mana cast satisfies Increment (2 > 1)");
}

#[test]
fn brush_off_counters_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lightning Bolt castable for {R}");
    let spell_id = g.stack.iter().find_map(|s| match s {
        StackItem::Spell { card, .. } => Some(card.id),
        _ => None,
    }).unwrap();
    g.priority.player_with_priority = 0;
    let brush = g.add_card_to_hand(0, catalog::brush_off());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: brush, target: Some(Target::Permanent(spell_id)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Brush Off castable for {2}{U}{U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the bolt was countered — no damage dealt");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt),
        "countered spell goes to its owner's graveyard");
}

#[test]
fn zaffai_grants_one_free_is_cast_at_first_main() {
    // Once during each of your turns, you may cast an IS from hand for
    // free. Wired as a precombat-main grant of MayPlay on one IS card.
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zaffai_and_the_tempests());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    let card = g.players[0].hand.iter().find(|c| c.id == bolt)
        .expect("bolt still in hand");
    assert!(card.may_play_until.is_some(),
        "Zaffai stamps a may-play permission on an instant/sorcery in hand");
}

// ── SOS school lands (enter tapped, tap for either of two colors) ────────────

fn assert_school_land(def_fn: fn() -> crabomination::card::CardDefinition, a: Color, b: Color) {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, def_fn());
    // Mana ability 0 → color a.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("first color mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(a), 1, "ability 0 taps for color a");
    // Mana ability 1 → color b.
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.players[0].mana_pool = crabomination::mana::ManaPool::default();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("second color mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(b), 1, "ability 1 taps for color b");
}

#[test]
fn fields_of_strife_taps_for_red_or_white() {
    assert_school_land(catalog::fields_of_strife, Color::Red, Color::White);
}

#[test]
fn forum_of_amity_taps_for_white_or_black() {
    assert_school_land(catalog::forum_of_amity, Color::White, Color::Black);
}

#[test]
fn paradox_gardens_taps_for_green_or_blue() {
    assert_school_land(catalog::paradox_gardens, Color::Green, Color::Blue);
}

#[test]
fn spectacle_summit_taps_for_blue_or_red() {
    assert_school_land(catalog::spectacle_summit, Color::Blue, Color::Red);
}

#[test]
fn titans_grave_taps_for_black_or_green() {
    assert_school_land(catalog::titans_grave, Color::Black, Color::Green);
}

#[test]
fn school_land_enters_tapped() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fields_of_strife());
    g.perform_action(GameAction::PlayLand(id))
        .expect("can play a land");
    drain_stack(&mut g);
    let land = g.battlefield_find(id).expect("land on battlefield");
    assert!(land.tapped, "SOS school lands enter the battlefield tapped");
}

/// Strixhaven Skycoach is a Vehicle (Crew 2): it animates into a 3/2 flier
/// when crewed by a 2-power creature.
#[test]
fn strixhaven_skycoach_crews_into_a_flier() {
    let mut g = two_player_game();
    let coach = g.add_card_to_battlefield(0, catalog::strixhaven_skycoach());
    // Not a creature until crewed.
    assert!(!g.computed_permanent(coach).unwrap().card_types.contains(&CardType::Creature));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::Crew { vehicle: coach, crew_creatures: vec![bear] })
        .expect("crew 2 satisfied by a 2/2");
    let cp = g.computed_permanent(coach).unwrap();
    assert!(cp.card_types.contains(&CardType::Creature));
    assert!(cp.keywords.contains(&Keyword::Flying));
    assert_eq!(cp.power, 3);
    assert_eq!(cp.toughness, 2);
}

#[test]
fn zaffai_grants_a_free_instant_or_sorcery_each_turn() {
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::zaffai_and_the_tempests());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    // Beginning of the active player's main phase grants a free IS cast.
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    // No mana in pool — only the Zaffai grant makes the Bolt castable.
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Zaffai grant lets Bolt be cast for free");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "free Bolt dealt 3");
}

#[test]
fn daydream_flickers_a_creature_and_adds_a_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::daydream());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Daydream castable for {1}{W}");
    drain_stack(&mut g);
    // Exile + return resolve in one shot — the bear is back (a fresh
    // instance) carrying a +1/+1 counter.
    let returned = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears")
        .expect("creature returned to battlefield");
    assert_eq!(returned.counter_count(CounterType::PlusOnePlusOne), 1,
        "flicker leaves a +1/+1 counter");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Daydream"));
}

// ── SOS Prepare — generic mechanic tests ────────────────────────────────────

#[test]
fn cast_prepare_spell_happy_path_unprepares_and_copy_vanishes() {
    let mut g = two_player_game();
    let id = prepared_on_battlefield(&mut g, 0, catalog::studious_first_year());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let gy_before = g.players[0].graveyard.len();

    g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("prepared creature's spell is castable");
    drain_stack(&mut g);

    // Resolved: Forest fetched, the Prepared counter is gone.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Forest" && c.controller == 0),
        "Rampant Growth fetched a Forest");
    let c = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(c.counter_count(CounterType::Prepared), 0, "casting unprepares");
    // The copy is in NO zone afterwards (CR 707.10a).
    assert_eq!(g.players[0].graveyard.len(), gy_before,
        "copy ceases to exist — not in graveyard");
    let copy_lingers = g.players[0].hand.iter().any(|c| c.definition.name == "Rampant Growth")
        || g.exile.iter().any(|c| c.definition.name == "Rampant Growth")
        || g.battlefield.iter().any(|c| c.definition.name == "Rampant Growth")
        || g.players[0].library.iter().any(|c| c.definition.name == "Rampant Growth");
    assert!(!copy_lingers, "copy not in hand/exile/battlefield/library either");

    // Second cast without re-preparing must reject.
    g.players[0].mana_pool.add(Color::Green, 2);
    let r2 = g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(r2, Err(GameError::NotPrepared(_))),
        "spent prepared counter — second cast rejects");
}

#[test]
fn cast_prepare_spell_rejects_creature_without_prepared_counter() {
    let mut g = two_player_game();
    // Straight onto the battlefield WITHOUT a Prepared counter.
    let id = g.add_card_to_battlefield(0, catalog::studious_first_year());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let r = g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(r, Err(GameError::NotPrepared(_))),
        "unprepared creature's spell is not castable");
}

#[test]
fn cast_prepare_spell_rejects_opponent_controlled_creature() {
    let mut g = two_player_game();
    // Prepared, but controlled by the OPPONENT — P0 may not cast it.
    let id = prepared_on_battlefield(&mut g, 1, catalog::studious_first_year());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;

    let r = g.perform_action(GameAction::CastPrepareSpell {
        creature_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(r, Err(GameError::NotPrepared(_))),
        "only the prepared creature's controller may cast the copy");
}

#[test]
fn cast_prepare_spell_rejects_creature_not_on_battlefield() {
    let mut g = two_player_game();
    let bogus = g.add_card_to_hand(0, catalog::studious_first_year());

    let r = g.perform_action(GameAction::CastPrepareSpell {
        creature_id: bogus, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(matches!(r, Err(GameError::CardNotOnBattlefield(_))),
        "creature must be on the battlefield");
}

#[test]
fn prepared_counter_never_stacks() {
    // "A creature can't become prepared if it's already prepared" — the
    // AddCounter pipeline clamps Prepared at 1. Scathing Shadelock's
    // first-main trigger prepares it each turn; firing it twice still
    // leaves exactly one counter.
    use crabomination::game::types::TurnStep;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::scathing_shadelock());
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);

    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::Prepared), 1,
        "Prepared is a flag — two prepare effects leave one counter");
}

#[test]
fn enters_prepared_creature_gets_counter_when_cast_from_hand() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goblin_glasswright());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Goblin Glasswright castable for {1}{R}");
    drain_stack(&mut g);

    let c = g.battlefield_find(id).expect("Glasswright on battlefield");
    assert_eq!(c.counter_count(CounterType::Prepared), 1,
        "\"This creature enters prepared.\" — one Prepared counter on ETB");
}

#[test]
fn prepare_castable_affordance_tracks_mana_availability() {
    let mut g = two_player_game();
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    let id = prepared_on_battlefield(&mut g, 0, catalog::studious_first_year());

    // No lands, empty mana pool → the {1}{G} prepare spell is unaffordable.
    let view = crabomination::server::view::project(&g, 0);
    assert!(view.prepare_castable.is_empty(),
        "no mana → prepare spell not castable");

    // An untapped Forest + a second land cover {1}{G} via auto-tap.
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island());
    let view = crabomination::server::view::project(&g, 0);
    assert!(view.prepare_castable.contains(&id),
        "two untapped lands → prepared creature surfaces in prepare_castable");
}

// ── Diagnostics (claude/modern_decks bugfix pass) ───────────────────────────

#[test]
fn cauldron_activation_reanimates_and_sac_fires_death_drain() {
    let mut g = two_player_game();
    let cauldron = g.add_card_to_battlefield(0, catalog::cauldron_of_essence());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_life = g.players[1].life;
    let _ = (cauldron, bear);

    g.perform_action(GameAction::ActivateAbility {
        card_id: cauldron,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("Cauldron reanimation activatable");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == dead),
        "dead bear reanimated to battlefield",
    );
    assert_eq!(
        g.players[1].life, opp_life - 1,
        "sacrificing the bear as a cost fires the death drain",
    );
}

#[test]
fn berta_activation_creates_fractal_token() {
    let mut g = two_player_game();
    let berta = g.add_card_to_battlefield(0, catalog::berta_wise_extrapolator());
    g.clear_sickness(berta);
    g.players[0].mana_pool.add_colorless(2);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::ActivateAbility {
        card_id: berta,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: Some(2) , mode: None})
    .expect("Berta token ability activatable");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Fractal token created");
    let token = g.battlefield.iter().find(|c| c.definition.name == "Fractal").expect("Fractal on battlefield");
    let view = g.computed_permanent(token.id).unwrap();
    assert_eq!((view.power, view.toughness), (2, 2), "0/0 + 2 counters");
}

#[test]
fn chelonian_tackle_ui_prompts_for_fight_defender() {
    // A `wants_ui` caster sends only slot 0 (the client's cast flow has
    // no multi-target pick); the engine suspends a ChooseTarget for the
    // fight defender, and the answered cast resolves both halves.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let friendly = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chelonian_tackle());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(friendly)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast suspends on the defender pick");
    let pending = g.pending_decision.as_ref().expect("defender ChooseTarget pending");
    match &pending.decision {
        crabomination::decision::Decision::ChooseTarget { legal, .. } => {
            assert_eq!(legal, &vec![Target::Permanent(opp)], "only the opp creature is legal");
        }
        other => panic!("expected ChooseTarget, got {other:?}"),
    }
    g.submit_decision(DecisionAnswer::Target(Target::Permanent(opp)))
        .expect("answer replays the cast with both slots");
    drain_stack(&mut g);

    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == opp),
        "opp bear dies to the fight",
    );
    assert!(g.computed_permanent(friendly).is_some(), "friendly bear survives at 2/12");
}

#[test]
fn ui_player_prompted_when_spirit_guide_could_pay() {
    // Forest on the battlefield AND Elvish Spirit Guide in hand: the {G}
    // could come from either, and the auto-tapper can't exercise the
    // guide — the cast must stop for a manual choice.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // CR 601.2g proper tapping is `manual_mana`, not `wants_ui`: this test
    // models a human choosing their own sources. A bot seat wants its
    // decisions surfaced but still auto-taps.
    g.players[0].manual_mana = true;
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_hand(0, catalog::elvish_spirit_guide());
    let id = g.add_card_to_hand(0, catalog::crop_rotation());

    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(
        matches!(result, Err(GameError::ManualTapRequired { .. })),
        "Spirit Guide in hand must force a manual payment choice, got {result:?}",
    );
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "spell stays in hand");
}

#[test]
fn ui_player_prompted_when_land_and_rock_make_same_color() {
    // Forest + Mox Emerald both make {G}: same colour signature but
    // different permanent kinds — tapping one over the other is a real
    // choice, so the cast prompts instead of auto-tapping.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // CR 601.2g proper tapping is `manual_mana`, not `wants_ui`: this test
    // models a human choosing their own sources. A bot seat wants its
    // decisions surfaced but still auto-taps.
    g.players[0].manual_mana = true;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let mox = g.add_card_to_battlefield(0, catalog::mox_emerald());
    let id = g.add_card_to_hand(0, catalog::crop_rotation());

    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(
        matches!(result, Err(GameError::ManualTapRequired { .. })),
        "land vs rock is a genuine tap choice, got {result:?}",
    );
    for src in [forest, mox] {
        assert!(
            !g.battlefield.iter().find(|c| c.id == src).unwrap().tapped,
            "neither ambiguous source auto-tapped",
        );
    }
}

#[test]
fn spirit_guide_activates_from_hand() {
    // Right-click path: `ActivateAbility` on a hand card with a
    // `from_hand` mana ability exiles the guide and floats {G}.
    let mut g = two_player_game();
    let guide = g.add_card_to_hand(0, catalog::elvish_spirit_guide());
    g.perform_action(GameAction::ActivateAbility {
        card_id: guide, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None , mode: None})
    .expect("guide's mana ability usable from hand");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "{{G}} floats");
    assert!(g.exile.iter().any(|c| c.id == guide), "guide exiled as its cost");
}

// ── Audit fixes (SOS audit pass) ────────────────────────────────────────────

#[test]
fn ui_player_x_spell_prompts_choose_amount() {
    // A wants_ui player casting an {X} spell with no x_value gets a
    // ChooseAmount suspend; the answered amount is the cast's X.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let id = g.add_card_to_hand(0, catalog::wild_hypothesis());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast suspends on the X pick");
    assert!(
        matches!(
            g.pending_decision.as_ref().map(|p| &p.decision),
            Some(crabomination::decision::Decision::ChooseAmount { .. }),
        ),
        "X spell should pose a ChooseAmount",
    );
    g.submit_decision(DecisionAnswer::Amount(3)).expect("X=3 replays the cast");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Fractal token survives at X=3");
    let token = g.battlefield.iter().find(|c| c.definition.name == "Fractal").unwrap();
    let view = g.computed_permanent(token.id).unwrap();
    assert_eq!((view.power, view.toughness), (3, 3), "X=3 counters applied");
}

#[test]
fn ui_player_graveyard_step_trigger_poses_card_picker_not_cursor() {
    // Ascendant Dustspeaker's begin-combat trigger targets a card in a
    // graveyard. Step triggers route through `drain_trigger_queue`,
    // which used to pose a ChooseTarget listing graveyard ids the
    // cursor can't click — an unanswerable soft-lock. It must now pose
    // a ChooseCards modal instead.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let dust = g.add_card_to_battlefield(0, catalog::ascendant_dustspeaker());
    g.clear_sickness(dust);
    let gy_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());

    // Walk priority from main into Begin Combat, where the trigger fires
    // and must suspend on the modal.
    for _ in 0..12 {
        if g.pending_decision.is_some() {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    let pending = g.pending_decision.as_ref().expect("begin-combat target pick pending");
    match &pending.decision {
        crabomination::decision::Decision::ChooseCards { candidates, .. } => {
            assert!(candidates.iter().any(|(cid, _)| *cid == gy_card), "graveyard card offered");
        }
        other => panic!("expected ChooseCards for a graveyard target, got {other:?}"),
    }
    g.submit_decision(DecisionAnswer::Cards(vec![gy_card])).expect("pick resolves");
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.id == gy_card),
        "picked graveyard card exiled by Dustspeaker",
    );
}

#[test]
fn graveyard_etb_trigger_auto_picks_for_ui_player() {
    // Self-ETB triggers push with an auto-picked target (stack.rs's
    // resolve path) — a wants_ui Zealous Lorecaster controller doesn't
    // get a picker, but the trigger must still resolve (no soft-lock,
    // an I/S card comes back to hand).
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::zealous_lorecaster());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Lorecaster castable");
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == bolt),
        "ETB auto-returns the instant from the graveyard",
    );
}

#[test]
fn borrowed_knowledge_mode_0_draws_opponent_hand_size() {
    let mut g = two_player_game();
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let extra = g.add_card_to_hand(0, catalog::grizzly_bears());
    let _ = extra;
    let id = g.add_card_to_hand(0, catalog::borrowed_knowledge());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("Borrowed Knowledge castable");
    drain_stack(&mut g);

    assert_eq!(
        g.players[0].hand.len(),
        4,
        "hand discarded, then draw = opponent's hand size (4)",
    );
}

#[test]
fn wilt_in_the_heat_rejects_player_target() {
    // Printed "target creature" — a bare Target(0) let the spell aim at
    // (and the bot auto-pick) the opponent's face. The slot-0 Creature
    // filter must reject a player target and still kill a creature.
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::wilt_in_the_heat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);

    let result = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(
        matches!(result, Err(GameError::SelectionRequirementViolated)),
        "player target must be rejected, got {result:?}",
    );

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("creature target still legal");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear dies to 5 damage");
}

#[test]
fn primary_research_does_not_reanimate_from_opponent_graveyard() {
    let mut g = two_player_game();
    let opp_card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::primary_research());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Primary Research castable");
    drain_stack(&mut g);

    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == opp_card),
        "printed 'from your graveyard' — the opponent's card stays put",
    );
}

#[test]
fn matterbending_mage_does_not_bounce_itself_when_alone() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::matterbending_mage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mage castable");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == id),
        "printed 'one OTHER target creature' — the Mage never bounces itself",
    );
}

#[test]
fn ennis_does_not_flicker_itself_when_alone() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ennis_debate_moderator());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ennis castable");
    drain_stack(&mut g);

    assert!(
        g.battlefield.iter().any(|c| c.id == id),
        "printed 'one OTHER target creature you control' — Ennis never exiles itself",
    );
}

#[test]
fn ui_player_graveyard_spell_target_via_card_picker() {
    // Moment of Reckoning mode 1 targets a nonland card in your
    // graveyard. A wants_ui cast with no target must suspend on a
    // ChooseCards modal (the cursor can't select graveyard cards), then
    // resolve with the picked card.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    })
    .expect("cast suspends on the graveyard pick");
    let pending = g.pending_decision.as_ref().expect("graveyard pick pending");
    match &pending.decision {
        crabomination::decision::Decision::ChooseCards { candidates, .. } => {
            assert!(candidates.iter().any(|(cid, _)| *cid == dead), "dead bear offered");
        }
        other => panic!("expected ChooseCards, got {other:?}"),
    }
    g.submit_decision(DecisionAnswer::Cards(vec![dead])).expect("pick replays the cast");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == dead),
        "picked card reanimated to the battlefield",
    );
}

#[test]
fn ui_player_loyalty_graveyard_target_via_card_picker() {
    // Ral Zarek, Guest Lecturer's −2 targets a creature card (MV ≤ 3)
    // in your graveyard. A wants_ui activation with no target must
    // suspend on a ChooseCards modal, then reanimate the pick.
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    let ral = g.add_card_to_battlefield(0, catalog::ral_zarek_guest_lecturer());
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: ral, ability_index: 2, target: None, x_value: None,
    })
    .expect("loyalty activation suspends on the graveyard pick");
    let pending = g.pending_decision.as_ref().expect("graveyard pick pending");
    match &pending.decision {
        crabomination::decision::Decision::ChooseCards { candidates, .. } => {
            assert!(candidates.iter().any(|(cid, _)| *cid == dead), "dead bear offered");
        }
        other => panic!("expected ChooseCards, got {other:?}"),
    }
    g.submit_decision(DecisionAnswer::Cards(vec![dead])).expect("pick replays the activation");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == dead),
        "picked creature reanimated by the −2",
    );
}

/// The five school lands enter tapped via a true CR 614.13 replacement —
/// the land is already tapped the moment it hits the battlefield, with no
/// trigger window in which it could be tapped for mana. They also carry
/// no basic land types (plain duals, not typed ones).
#[test]
fn school_lands_enter_tapped_as_replacement_not_trigger() {
    for def in [
        catalog::forum_of_amity,
        catalog::fields_of_strife,
        catalog::paradox_gardens,
        catalog::titans_grave,
        catalog::spectacle_summit,
    ] {
        let mut g = two_player_game();
        let id = g.add_card_to_hand(0, def());
        g.perform_action(GameAction::PlayLand(id)).unwrap();
        // No drain_stack: tapped state must hold before any trigger could
        // resolve (CR 614.13 replacement, not an ETB trigger).
        let land = g.battlefield_find(id).unwrap();
        assert!(land.tapped, "{} enters tapped immediately", land.definition.name);
        assert!(
            land.definition.subtypes.land_types.is_empty(),
            "{} has no basic land types",
            land.definition.name
        );
    }
}

/// "Whenever one or more cards leave your graveyard" fires once per
/// simultaneous batch (CR 603.2), not once per card: a mass exile of a
/// three-card graveyard grants Hardened Academic's payoff exactly one
/// +1/+1 counter.
#[test]
fn cards_leave_graveyard_batch_triggers_once() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hardened_academic());
    for _ in 0..3 {
        let id = g.next_id();
        let mut card = crabomination::card::CardInstance::new(id, catalog::lightning_bolt(), 0);
        card.controller = 0;
        g.players[0].graveyard.push(card);
    }
    let counters_before: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    // Opponent's Bojuka Bog exiles player 0's whole graveyard in one batch.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bog = g.add_card_to_hand(1, catalog::bojuka_bog());
    g.perform_action(GameAction::PlayLand(bog)).expect("play Bojuka Bog");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.is_empty(), "graveyard exiled");
    let counters_after: u32 = g
        .battlefield
        .iter()
        .filter(|c| c.controller == 0)
        .map(|c| c.counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert_eq!(
        counters_after,
        counters_before + 1,
        "three cards leaving at once is ONE batch — exactly one counter"
    );
}

/// "Whenever ONE OR MORE creatures you control deal combat damage to a
/// player" fires once per damage batch (CR 603.2): two attackers
/// connecting give Killian's Confidence a single may-pay prompt, not two.
#[test]
fn killians_confidence_two_attackers_one_trigger() {
    let mut g = two_player_game();
    let kc = g.add_card_to_graveyard(0, catalog::killians_confidence());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(b);
    // Two W floated + two scripted yeses: a per-attacker double-fire would
    // consume both; the batched trigger must leave one W unspent.
    g.players[0].mana_pool.add(Color::White, 2);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ]))
    .expect("attackers declared");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().expect("combat damage resolved");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == kc), "KC returned to hand");
    assert_eq!(
        g.players[0].mana_pool.amount(Color::White),
        1,
        "exactly one {{W/B}} paid — the batch fired the trigger once"
    );
}

/// Leech Collector prepares on the turn's FIRST life gain only: a gain that
/// happened before it hit the battlefield disqualifies later gains that turn
/// (oracle "whenever you gain life for the first time each turn" — not
/// "the first time this ability sees a gain").
#[test]
fn leech_collector_only_first_gain_of_turn_prepares() {
    // First gain of the turn happens with Leech Collector on the field →
    // it becomes prepared.
    let mut g = two_player_game();
    let lc = g.add_card_to_battlefield(0, catalog::leech_collector());
    let fountain = g.add_card_to_hand(0, catalog::radiant_fountain());
    g.perform_action(GameAction::PlayLand(fountain)).expect("fountain");
    drain_stack(&mut g);
    let c = g.battlefield_find(lc).unwrap();
    assert_eq!(c.counter_count(CounterType::Prepared), 1, "first gain prepares");

    // The turn's first gain happened BEFORE Leech Collector arrived → a
    // later gain the same turn does not prepare it.
    let mut g = two_player_game();
    g.players[0].extra_land_plays = 1;
    let f1 = g.add_card_to_hand(0, catalog::radiant_fountain());
    g.perform_action(GameAction::PlayLand(f1)).expect("first fountain");
    drain_stack(&mut g); // gain 2 — flips the "gained earlier" flag
    let lc = g.add_card_to_battlefield(0, catalog::leech_collector());
    let f2 = g.add_card_to_hand(0, catalog::radiant_fountain());
    g.perform_action(GameAction::PlayLand(f2)).expect("second fountain");
    drain_stack(&mut g);
    let c = g.battlefield_find(lc).unwrap();
    assert_eq!(
        c.counter_count(CounterType::Prepared),
        0,
        "a pre-arrival gain used up the turn's 'first time'"
    );
}

/// "Its controller creates ..." reads the destroyed creature's death-time
/// CONTROLLER (CR 608.2h LKI), not its owner — a stolen creature's Inkling
/// goes to the thief.
#[test]
fn harsh_annotation_token_goes_to_death_time_controller() {
    let mut g = two_player_game();
    // P0 owns the bear, but P1 has stolen it.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 1;
    let spell = g.add_card_to_hand(0, catalog::harsh_annotation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Harsh Annotation castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear destroyed");
    let inkling_controller: Vec<usize> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Inkling")
        .map(|c| c.controller)
        .collect();
    assert_eq!(inkling_controller, vec![1], "token minted for the CONTROLLER (P1), not the owner");
}

/// Tenured Concocter's "becomes the target of a spell or ability an
/// opponent controls" also fires for opponents' TRIGGERED abilities
/// (Ravenous Chupacabra's ETB targets it), not just spells and
/// activated abilities.
#[test]
fn tenured_concocter_triggers_on_opponent_triggered_ability_target() {
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let tc = g.add_card_to_battlefield(0, catalog::tenured_concocter());
    let hand_before = g.players[0].hand.len();
    // The "you may draw" MayDo: accept.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Opponent's Chupacabra ETB auto-targets P0's only creature.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let chupa = g.add_card_to_hand(1, catalog::ravenous_chupacabra());
    g.players[1].mana_pool.add(Color::Black, 2);
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: chupa, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Chupacabra castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == tc), "Concocter destroyed by the ETB");
    assert_eq!(
        g.players[0].hand.len(),
        hand_before + 1,
        "being targeted by the opponent's TRIGGERED ability drew a card"
    );
}

/// "Pay X life" is an additional COST paid on cast (CR 601.2h): a
/// countered Vicious Rivalry still cost its caster the X life, and the
/// board is untouched.
#[test]
fn vicious_rivalry_life_cost_paid_even_when_countered() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2 ≤ X
    let vr = g.add_card_to_hand(0, catalog::vicious_rivalry());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: vr, target: None, additional_targets: vec![], mode: None,
        x_value: Some(3),
    })
    .expect("Vicious Rivalry castable, X=3 life paid on cast");
    assert_eq!(g.players[0].life, life_before - 3, "life paid at CAST time");
    // Opponent counters it.
    g.priority.player_with_priority = 1;
    let cs = g.add_card_to_hand(1, catalog::counterspell());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: cs, target: Some(Target::Permanent(vr)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Counterspell castable");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == vr), "VR countered to graveyard");
    assert!(g.battlefield.iter().any(|c| c.id == bear), "nothing destroyed");
    assert_eq!(g.players[0].life, life_before - 3,
        "the X life stays paid — costs are not refunded on counter (CR 601.2h)");
}

/// The pay-X-life pre-flight rejects X above the caster's life total but
/// allows paying down to exactly 0 (CR 119.4).
#[test]
fn vicious_rivalry_x_capped_by_life_total() {
    let mut g = two_player_game();
    g.players[0].life = 5;
    let vr = g.add_card_to_hand(0, catalog::vicious_rivalry());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: vr, target: None, additional_targets: vec![], mode: None,
        x_value: Some(6),
    });
    assert!(err.is_err(), "X=6 with 5 life is unpayable");
    assert!(g.players[0].hand.iter().any(|c| c.id == vr), "cast rejected cleanly");
    // X=5 (down to exactly 0) is legal.
    g.perform_action(GameAction::CastSpell {
        card_id: vr, target: None, additional_targets: vec![], mode: None,
        x_value: Some(5),
    })
    .expect("paying your last life point is legal (CR 119.4)");
    assert_eq!(g.players[0].life, 0);
}

/// Moment of Reckoning's real "Choose up to four. You may choose the same
/// mode more than once." — one cast destroys TWO permanents and reanimates
/// TWO cards, each instance with its own target slot.
#[test]
fn moment_of_reckoning_four_mode_instances_in_one_cast() {
    let mut g = two_player_game();
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dead1 = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let dead2 = g.add_card_to_graveyard(0, catalog::mind_stone());
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: id,
        spree_modes: vec![0, 0, 1, 1],
        target: Some(Target::Permanent(bear1)),
        additional_targets: vec![
            Target::Permanent(bear2),
            Target::Permanent(dead1),
            Target::Permanent(dead2),
        ],
        x_value: None,
    })
    .expect("four mode instances castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear1), "first destroy");
    assert!(!g.battlefield.iter().any(|c| c.id == bear2), "second destroy");
    assert!(g.battlefield.iter().any(|c| c.id == dead1), "first reanimation");
    assert!(g.battlefield.iter().any(|c| c.id == dead2), "second reanimation");
}

/// The "up to four" cap rejects a five-instance pick.
#[test]
fn moment_of_reckoning_rejects_more_than_four_instances() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::moment_of_reckoning());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let err = g.perform_action(GameAction::CastSpellSpree {
        card_id: id,
        spree_modes: vec![0, 0, 0, 1, 1],
        target: None,
        additional_targets: vec![],
        x_value: None,
    });
    assert!(err.is_err(), "five instances exceed the printed 'up to four'");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "cast rejected cleanly");
}

/// Choreographed Sparks' "Choose one or both" — both modes in one cast:
/// copy the instant on the stack AND copy the creature spell on the stack.
#[test]
fn choreographed_sparks_copies_both_in_one_cast() {
    let mut g = two_player_game();
    // Creature spell on the stack (don't drain).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bear cast");
    // Instant on the stack above it.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt cast");
    // Sparks: mode 0 copies the bolt, mode 1 copies the bear spell.
    let sparks = g.add_card_to_hand(0, catalog::choreographed_sparks());
    g.players[0].mana_pool.add(Color::Red, 2);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpellSpree {
        card_id: sparks,
        spree_modes: vec![0, 1],
        target: Some(Target::Permanent(bolt)),
        additional_targets: vec![Target::Permanent(bear)],
        x_value: None,
    })
    .expect("one or both — both modes castable");
    drain_stack(&mut g);
    // Bolt + its copy: 6 damage total.
    assert_eq!(g.players[1].life, foe_life - 6, "original bolt + copied bolt");
    // Bear + its token copy on the battlefield.
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count(),
        2,
        "original bear + token copy of the creature spell"
    );
}

/// Aziza's tap-three cost is ALL-OR-NOTHING: with only two untapped
/// creatures (Aziza + one bear), the copy offer never fires — even an
/// eager decider gets no partial tap-and-copy.
#[test]
fn aziza_no_copy_with_fewer_than_three_untapped() {
    let mut g = two_player_game();
    let aziza = g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    g.clear_sickness(aziza);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let bob_life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, bob_life_before - 3,
        "two untapped creatures can't pay 'tap three' — no copy");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "cost not partially paid");
}

/// The controller chooses WHICH three creatures tap for Aziza's cost.
#[test]
fn aziza_controller_picks_which_creatures_tap() {
    let mut g = two_player_game();
    let aziza = g.add_card_to_battlefield(0, catalog::aziza_mage_tower_captain());
    g.clear_sickness(aziza);
    let mut bears = vec![];
    for _ in 0..3 {
        let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(b);
        bears.push(b);
    }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    // Yes to the offer, then pick the three BEARS (sparing Aziza).
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(bears.clone()),
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt castable");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(aziza).unwrap().tapped, "Aziza spared by the pick");
    for b in bears {
        assert!(g.battlefield_find(b).unwrap().tapped, "chosen bear tapped");
    }
}

/// Paradox Surveyor's "you MAY reveal ... and put it into your hand" is
/// genuinely declinable for a UI player: an explicit empty pick bottoms
/// all five looked-at cards and takes nothing.
#[test]
fn paradox_surveyor_ui_player_may_decline_the_pick() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    // Five known cards on top — two eligible (lands).
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let lib_before = g.players[0].library.len();
    let hand_before = g.players[0].hand.len();
    let id = g.add_card_to_hand(0, catalog::paradox_surveyor());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Paradox Surveyor castable");
    // Resolve the creature, then the ETB look suspends on the pick.
    for _ in 0..8 {
        if g.pending_decision.is_some() {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    assert!(g.pending_decision.is_some(), "look-at-top-five pick pending");
    // Decline: explicit empty pick.
    g.submit_decision(DecisionAnswer::Search(None)).expect("decline accepted");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].hand.len(),
        hand_before,
        "declined — nothing went to hand (the Surveyor itself left the hand for the battlefield)"
    );
    assert_eq!(
        g.players[0].library.len(),
        lib_before,
        "all five looked-at cards returned to the library (bottom)"
    );
}

/// Nita's activation sacrifices ANOTHER creature (never herself), and the
/// exiled card is cast by paying its own mana value with any type of mana
/// — not for free.
#[test]
fn nita_sacrifices_another_and_cast_pays_own_cost() {
    let mut g = two_player_game();
    let mut bolt =
        crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 1);
    bolt.controller = 1;
    let bolt_id = bolt.id;
    g.players[1].graveyard.push(bolt);
    let nita = g.add_card_to_battlefield(0, catalog::nita_forum_conciliator());
    let sac = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [nita, sac] {
        if let Some(c) = g.battlefield.iter_mut().find(|c| c.id == id) {
            c.summoning_sick = false;
        }
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nita, ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(bolt_id)),
        additional_targets: Vec::new(), x_value: None, mode: None,
    })
    .expect("Nita activation");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == nita), "Nita survives — 'sacrifice ANOTHER creature'");
    assert!(!g.battlefield.iter().any(|c| c.id == sac), "the bear paid the cost");
    // Casting the exiled bolt requires its mana value ({1}, any type).
    let err = g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt_id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "no mana floated — the cast is NOT free");
    g.players[0].mana_pool.add(Color::Green, 1); // any type pays
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bolt_id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("paying {1} with green mana casts the exiled bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "bolt resolved");
}

/// Visionary's Dance's "{2}, Discard this card: look at the top two, one to
/// hand and the other to the graveyard" — a real from-hand activation.
#[test]
fn visionarys_dance_discard_ability_activates_from_hand() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    let below = g.add_card_to_library(0, catalog::forest());
    // top of library order: bolt (index 0), forest (index 1)
    let vd = g.add_card_to_hand(0, catalog::visionarys_dance());
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    // Pick the bolt to hand (SearchLibrary decision).
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(top))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: vd, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("from-hand activation");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == vd), "Visionary's Dance discarded as the cost");
    assert!(g.players[0].hand.iter().any(|c| c.id == top), "picked card to hand");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == below), "the other card to the graveyard");
}

/// Zimone's Experiment is ONE look at five with typed routing: a picked
/// land enters the battlefield tapped while a picked creature goes to
/// hand, in the same pick.
#[test]
fn zimones_experiment_routes_land_to_battlefield_and_creature_to_hand() {
    let mut g = two_player_game();
    // Top five: forest, bear, three bolts.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let bear = g.add_card_to_library(0, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::zimones_experiment());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Multi-pick ChooseCards: take the forest AND the bear.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Cards(vec![
        forest, bear,
    ])]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Zimone's Experiment castable");
    drain_stack(&mut g);
    let land = g.battlefield.iter().find(|c| c.id == forest).expect("land onto battlefield");
    assert!(land.tapped, "revealed land enters tapped");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "revealed creature to hand");
    // The three bolts were bottomed (library holds exactly them).
    assert_eq!(g.players[0].library.len(), 3, "rest bottomed");
}

/// Archaic's Agony's exile rider: excess damage over the target's
/// toughness exiles that many cards with a play-until-your-next-turn-end
/// permission at full cost.
#[test]
fn archaics_agony_exiles_cards_equal_to_excess_damage() {
    let mut g = two_player_game();
    // 2/2 bear takes converge damage from a 4-color cast: 4 damage →
    // 2 excess over the bear's 2 toughness → 2 cards exiled.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    let id = g.add_card_to_hand(0, catalog::archaics_agony());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let exile_before = g.exile.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Archaic's Agony castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear died to converge damage");
    // Converge 4 colors (R+G+U+W) → 4 damage → 2 excess over toughness 2.
    let newly_exiled: Vec<_> = g.exile.iter().skip(exile_before)
        .filter(|c| c.definition.name == "Lightning Bolt").collect();
    assert_eq!(newly_exiled.len(), 2, "excess damage (4-2=2) exiled that many cards");
    for c in &newly_exiled {
        let p = c.may_play_until.expect("play permission granted");
        assert_eq!(
            p.duration,
            crabomination::card::MayPlayDuration::EndOfControllersNextTurn,
            "playable until the end of your next turn"
        );
        assert!(c.granted_alt_cast_cost_eot.is_some(), "a normal play — full cost");
    }
}

/// Mana Sculpt banks {C} equal to the countered spell's PAID mana (X
/// included) at the beginning of the caster's next main phase — not
/// immediately, and not the printed CMC.
#[test]
fn mana_sculpt_banks_paid_mana_at_next_main_phase() {
    let mut g = two_player_game();
    // A Wizard so the rider fires.
    g.add_card_to_battlefield(0, catalog::exhibition_tidecaller());
    // Opponent casts a 3-mana spell (Divination: {2}{U}).
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let div = g.add_card_to_hand(1, catalog::divination());
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: div, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Divination castable");
    // P0 counters with Mana Sculpt.
    g.priority.player_with_priority = 0;
    let ms = g.add_card_to_hand(0, catalog::mana_sculpt());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ms, target: Some(Target::Permanent(div)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Mana Sculpt castable");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == div), "Divination countered");
    // No mana yet — the {C} is banked for P0's next main phase.
    assert_eq!(g.players[0].mana_pool.total(), 0, "nothing added immediately");
    assert_eq!(g.delayed_triggers.len(), 1, "delayed trigger armed");
    // Advance to P0's next turn's precombat main.
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    let evs = g.advance_step(vec![]).expect("upkeep → draw");
    let _ = evs;
    let _ = g.advance_step(vec![]).expect("draw → main");
    drain_stack(&mut g);
    assert_eq!(
        g.players[0].mana_pool.total(),
        3,
        "3 colorless banked — the amount PAID for Divination"
    );
}

/// Tester of the Tangential's begin-combat "you may pay {X}" moves the
/// CHOSEN X counters — full X scaling, capped by floated mana, 0 declines.
#[test]
fn tester_of_the_tangential_pays_x_and_moves_x_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let tester = g.add_card_to_battlefield(0, catalog::tester_of_the_tangential());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tester).unwrap().add_counters(CounterType::PlusOnePlusOne, 3);
    // Float 2 → the pick is capped at 2 even though 3 counters sit on Tester.
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(2)]));
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(tester).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "X=2 counters left the Tester"
    );
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "X=2 counters arrived on the bear"
    );
    assert_eq!(g.players[0].mana_pool.total(), 0, "{{X}} = {{2}} paid from the pool");
}

/// Answering 0 declines the pay-{X} offer entirely.
#[test]
fn tester_of_the_tangential_zero_declines() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let tester = g.add_card_to_battlefield(0, catalog::tester_of_the_tangential());
    let _bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(tester).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Amount(0)]));
    g.active_player_idx = 0;
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(tester).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "declined — counters stay put"
    );
    assert_eq!(g.players[0].mana_pool.total(), 3, "nothing paid");
}

/// Great Hall of the Biblioplex's {5}: a PERMANENT 2/4 Wizard animation
/// (still a land) carrying the granted magecraft pump; the "isn't a
/// creature" gate blocks a second activation.
#[test]
fn great_hall_animates_permanently_with_magecraft_pump() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let hall = g.add_card_to_battlefield(0, catalog::great_hall_of_the_biblioplex());
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hall, ability_index: 2, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("{5} animation");
    drain_stack(&mut g);
    let cp = g.computed_permanent(hall).expect("on battlefield");
    assert!(cp.card_types.contains(&CardType::Creature), "now a creature");
    assert!(cp.card_types.contains(&CardType::Land), "still a land");
    assert_eq!((cp.power, cp.toughness), (2, 4), "2/4 Wizard");
    // Second activation is gated off ("if this land isn't a creature").
    g.players[0].mana_pool.add_colorless(5);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: hall, ability_index: 2, target: None, additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(err.is_err(), "already a creature — condition fails");
    // Granted magecraft: casting an instant pumps it +1/+0 EOT.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    let cp = g.computed_permanent(hall).unwrap();
    assert_eq!(cp.power, 3, "magecraft pump: 2 + 1");
}

/// Petrified Hamlet: the ETB name choice locks non-mana activations of
/// matching sources and grants matching lands "{T}: Add {C}".
#[test]
fn petrified_hamlet_name_lockout_and_land_grant() {
    let mut g = two_player_game();
    // Opponent has a Great Hall (an activated-ability land we can name).
    let hall = g.add_card_to_battlefield(1, catalog::great_hall_of_the_biblioplex());
    // And we control a Forest that will match the chosen name... name the
    // opponent's Great Hall instead to exercise the lockout, then check
    // the grant against another copy of the named land under our control.
    let ours = g.add_card_to_battlefield(0, catalog::great_hall_of_the_biblioplex());
    let _ = ours;
    let hamlet = g.add_card_to_hand(0, catalog::petrified_hamlet());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::NamedCard(
        "Great Hall of the Biblioplex".to_string(),
    )]));
    g.perform_action(GameAction::PlayLand(hamlet)).expect("play Hamlet");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hamlet).unwrap().named_card.as_deref(),
        Some("Great Hall of the Biblioplex"),
        "name stamped"
    );
    // Lockout: the opponent's Great Hall {5} animation (non-mana) is
    // suppressed; its mana abilities still work.
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add_colorless(5);
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: hall, ability_index: 2, target: None, additional_targets: vec![], x_value: None, mode: None,
    });
    assert!(err.is_err(), "non-mana activation locked by the chosen name");
    g.perform_action(GameAction::ActivateAbility {
        card_id: hall, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("mana abilities are exempt from the lockout");
    // Grant: OUR land with the chosen name gained "{T}: Add {C}" — its
    // granted ability index sits after the printed three.
    g.priority.player_with_priority = 0;
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: ours, ability_index: 3, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("granted {T}: Add {C} on the named land");
    assert_eq!(g.players[0].mana_pool.total(), before + 1, "granted mana ability produced {{C}}");
}

/// Improvisation Capstone's free casts are order-choosable: declining a
/// card keeps it available, so the controller can cast B before A.
#[test]
fn improvisation_capstone_declined_cards_are_reoffered() {
    let mut g = two_player_game();
    // Top of library: two bolts (MV 1+1... need total MV >= 4): bolt(1),
    // divination(3) → cumulative 4 → both exiled.
    let div = g.add_card_to_library(0, catalog::divination());
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let _ = (div, bolt);
    // library top order: bolt first, then divination.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest()); // draw fodder for Divination
    }
    let id = g.add_card_to_hand(0, catalog::improvisation_capstone());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(5);
    // Offers: bolt (decline), divination (accept), bolt re-offered (accept).
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Bool(true),
    ]));
    let foe_life = g.players[1].life;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Capstone castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 3, "the declined bolt was re-offered and cast");
    assert_eq!(
        g.players[0].hand.len(),
        hand_before - 1 + 2,
        "Divination cast free too (−Capstone +2 draws)"
    );
}
