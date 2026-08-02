//! Invasion (INV) gap wave 2 — the Leech and Djinn cycles, the Domain payoffs,
//! the damage-threshold replacements and the utility shell.

use crabomination::card::{CounterType, Keyword, LandType};
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::game::{Attack, AttackTarget};
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Put `n` untapped basics of `kind` onto `seat`'s battlefield.
fn basics(g: &mut GameState, seat: usize, kind: LandType, n: usize) {
    let factory = match kind {
        LandType::Plains => catalog::plains as fn() -> crabomination::card::CardDefinition,
        LandType::Island => catalog::island,
        LandType::Swamp => catalog::swamp,
        LandType::Mountain => catalog::mountain,
        _ => catalog::forest,
    };
    for _ in 0..n {
        g.add_card_to_battlefield(seat, factory());
    }
}

// ── Table-driven shapes ─────────────────────────────────────────────────────

/// The three wave-2 taplands all arrive tapped and pay two colours.
#[test]
fn inv_taplands_enter_tapped() {
    for f in [catalog::salt_marsh, catalog::shivan_oasis, catalog::urborg_volcano] {
        let mut g = main_phase();
        let land = g.add_card_to_hand(0, f());
        g.perform_action(GameAction::PlayLand(land)).expect("land drop");
        assert!(g.battlefield_find(land).unwrap().tapped, "{} entered untapped", f().name);
    }
}

/// The sac-lands crack for their two off-colour pips and go to the graveyard.
#[test]
fn inv_sac_lands_crack_for_two() {
    for (f, want) in [
        (catalog::irrigation_ditch as fn() -> _, [Color::Green, Color::Blue]),
        (catalog::sulfur_vent, [Color::Blue, Color::Red]),
        (catalog::tinder_farm, [Color::Red, Color::White]),
    ] {
        let mut g = main_phase();
        let land = g.add_card_to_battlefield(0, f());
        g.battlefield.iter_mut().find(|c| c.id == land).unwrap().tapped = false;
        activate(&mut g, 0, land, 1, None);
        for c in want {
            assert!(g.players[0].mana_pool.amount(c) >= 1, "{} missing {c:?}", f().name);
        }
        assert!(g.battlefield.iter().all(|c| c.id != land), "sacrificed");
    }
}

/// The Cameos and Attendants pay out their guild / wedge colours.
#[test]
fn inv_cameos_and_attendants_fix_mana() {
    for (f, idx, want) in [
        (catalog::seashell_cameo as fn() -> _, 0, vec![Color::White, Color::Blue]),
        (catalog::tigereye_cameo, 0, vec![Color::Green, Color::White]),
        (catalog::troll_horn_cameo, 0, vec![Color::Red, Color::Green]),
        (catalog::riths_attendant, 0, vec![Color::Red, Color::Green, Color::White]),
        (catalog::trevas_attendant, 0, vec![Color::Green, Color::White, Color::Blue]),
    ] {
        let mut g = main_phase();
        let rock = g.add_card_to_battlefield(0, f());
        g.battlefield.iter_mut().find(|c| c.id == rock).unwrap().tapped = false;
        // A Cameo taps for ONE of its two colours; an Attendant pays all three.
        activate(&mut g, 0, rock, idx, None);
        let produced: u32 = want.iter().map(|c| g.players[0].mana_pool.amount(*c)).sum();
        assert!(produced >= 1, "{} produced nothing", f().name);
    }
}

// ── The Leech cycle (StaticEffect::ColoredSpellTax) ─────────────────────────

/// Alabaster Leech taxes its controller's white spells a real {W}.
#[test]
fn alabaster_leech_taxes_your_white_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::alabaster_leech());
    let spell = g.add_card_to_hand(0, catalog::reviving_dose());
    let taxed = crabomination::game::actions::colored_spell_tax_for_spell(
        &g,
        0,
        g.players[0].hand.iter().find(|c| c.id == spell).unwrap(),
    );
    assert_eq!(taxed.cmc(), 1);
    // The tax is colored, not generic: paying with only 3 colorless fails.
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "colorless mana should not cover the {{W}} surcharge"
    );
}

/// The tax is controller-scoped — an opponent's white spell is untaxed.
#[test]
fn leech_tax_does_not_touch_opponents() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::alabaster_leech());
    let spell = g.add_card_to_hand(1, catalog::reviving_dose());
    let taxed = crabomination::game::actions::colored_spell_tax_for_spell(
        &g,
        1,
        g.players[1].hand.iter().find(|c| c.id == spell).unwrap(),
    );
    assert_eq!(taxed.cmc(), 0);
}

/// Andradite Leech pumps itself for {B}.
#[test]
fn andradite_leech_pumps_for_b() {
    let mut g = main_phase();
    let leech = g.add_card_to_battlefield(0, catalog::andradite_leech());
    activate(&mut g, 0, leech, 0, None);
    let cp = g.computed_permanent(leech).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

// ── The Djinn cycle (Predicate::ColorIsMostCommonAmongPermanents) ───────────

/// Zanam Djinn shrinks while blue leads the board and recovers when it doesn't.
#[test]
fn zanam_djinn_shrinks_while_blue_leads() {
    let mut g = main_phase();
    let djinn = g.add_card_to_battlefield(0, catalog::zanam_djinn());
    // The Djinn itself is the only blue permanent → blue is most common.
    let cp = g.computed_permanent(djinn).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4));
    // Two red permanents outnumber it.
    for _ in 0..2 {
        g.add_card_to_battlefield(1, catalog::ruby_leech());
    }
    let cp = g.computed_permanent(djinn).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 6));
}

/// A tie for most common still shrinks the Djinn (printed "or is tied").
#[test]
fn djinn_shrinks_on_a_tie() {
    let mut g = main_phase();
    let djinn = g.add_card_to_battlefield(0, catalog::sulam_djinn());
    g.add_card_to_battlefield(1, catalog::ruby_leech());
    let cp = g.computed_permanent(djinn).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "green ties red at one each");
}

/// Barrin's Unmaking only targets a permanent of the leading colour.
#[test]
fn barrins_unmaking_targets_the_leading_color() {
    let mut g = main_phase();
    // Two green permanents lead; one blue trails.
    let green = g.add_card_to_battlefield(1, catalog::jade_leech());
    let _green2 = g.add_card_to_battlefield(1, catalog::utopia_tree());
    let blue = g.add_card_to_battlefield(1, catalog::sapphire_leech());
    let spell = g.add_card_to_hand(0, catalog::barrins_unmaking());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: Some(Target::Permanent(blue)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "blue is not the most common colour"
    );
    cast(&mut g, 0, spell, Some(Target::Permanent(green)));
    assert!(g.players[1].hand.iter().any(|c| c.id == green), "bounced to hand");
}

// ── Damage-threshold replacements ───────────────────────────────────────────

/// Callous Giant shrugs off a 3-damage hit but not a 4-damage one.
#[test]
fn callous_giant_prevents_small_damage() {
    let mut g = main_phase();
    let giant = g.add_card_to_battlefield(0, catalog::callous_giant());
    let mut events = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(giant),
        3,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 0);
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(giant),
        4,
        None,
        &mut events,
    );
    assert_eq!(g.battlefield_find(giant).unwrap().damage, 4);
}

/// Divine Presence replaces a big event with exactly 3 damage.
#[test]
fn divine_presence_caps_big_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::divine_presence());
    let ent = crabomination::game::effects::EntityRef::Player(1);
    assert_eq!(g.scale_damage_to(None, ent, 7), 3);
    assert_eq!(g.scale_damage_to(None, ent, 4), 3);
    // Below the threshold the event is untouched.
    assert_eq!(g.scale_damage_to(None, ent, 3), 3);
    assert_eq!(g.scale_damage_to(None, ent, 2), 2);
}

// ── Domain payoffs ──────────────────────────────────────────────────────────

/// Kavu Scout grows +1/+0 per basic land type, toughness unchanged.
#[test]
fn kavu_scout_scales_with_domain() {
    let mut g = main_phase();
    let scout = g.add_card_to_battlefield(0, catalog::kavu_scout());
    basics(&mut g, 0, LandType::Forest, 1);
    basics(&mut g, 0, LandType::Island, 1);
    basics(&mut g, 0, LandType::Swamp, 1);
    let cp = g.computed_permanent(scout).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2));
}

/// Wandering Stream pays 2 life per basic land type.
#[test]
fn wandering_stream_pays_two_per_type() {
    let mut g = main_phase();
    basics(&mut g, 0, LandType::Plains, 1);
    basics(&mut g, 0, LandType::Mountain, 1);
    let before = g.players[0].life;
    let spell = g.add_card_to_hand(0, catalog::wandering_stream());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].life, before + 4);
}

/// Ordered Migration mints one flying Bird per basic land type.
#[test]
fn ordered_migration_mints_a_bird_per_type() {
    let mut g = main_phase();
    basics(&mut g, 0, LandType::Plains, 1);
    basics(&mut g, 0, LandType::Island, 1);
    basics(&mut g, 0, LandType::Forest, 2);
    let spell = g.add_card_to_hand(0, catalog::ordered_migration());
    cast(&mut g, 0, spell, None);
    let birds: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Bird").collect();
    assert_eq!(birds.len(), 3);
    assert!(birds[0].definition.keywords.contains(&Keyword::Flying));
}

// ── Kicker commons ──────────────────────────────────────────────────────────

/// Kavu Titan is a 2/2 unkicked and a 5/5 trampler kicked.
#[test]
fn kavu_titan_kicked_is_a_trampling_five_five() {
    let mut g = main_phase();
    let plain = g.add_card_to_hand(0, catalog::kavu_titan());
    cast(&mut g, 0, plain, None);
    let cp = g.computed_permanent(plain).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));

    let kicked = g.add_card_to_hand(0, catalog::kavu_titan());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: kicked,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(kicked).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert_eq!(g.battlefield_find(kicked).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Prohibit's unkicked gate is mana value 2 — it counters Opt, not Jade Leech.
#[test]
fn prohibit_respects_its_mana_value_gate() {
    let mut g = main_phase();
    let cheap = g.add_card_to_hand(1, catalog::opt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: cheap,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Opt");
    let counter = g.add_card_to_hand(0, catalog::prohibit());
    cast(&mut g, 0, counter, Some(Target::Permanent(cheap)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == cheap), "Opt was countered");

    // A mana value 4 spell is out of range for the unkicked half.
    let mut g = main_phase();
    let big = g.add_card_to_hand(1, catalog::reviving_vapors());
    g.add_card_to_library(1, catalog::opt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: big,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Reviving Vapors");
    let counter = g.add_card_to_hand(0, catalog::prohibit());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: counter,
            target: Some(Target::Permanent(big)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "unkicked Prohibit can't target a mana value 4 spell"
    );
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Turf Wound locks the target out of land drops for the turn.
#[test]
fn turf_wound_locks_the_land_drop() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::turf_wound());
    g.add_card_to_library(0, catalog::opt());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert!(!g.can_player_play_land(1));
    assert!(g.can_player_play_land(0));
    assert_eq!(g.players[0].hand.len(), before, "spell left hand, cantrip refilled");
}

/// Phyrexian Reaper kills a green blocker but leaves an off-colour one alone.
#[test]
fn phyrexian_reaper_eats_green_blockers() {
    let mut g = two_player_game();
    let reaper = g.add_card_to_battlefield(0, catalog::phyrexian_reaper());
    g.clear_sickness(reaper);
    let green = g.add_card_to_battlefield(1, catalog::utopia_tree());
    let blue = g.add_card_to_battlefield(1, catalog::sapphire_leech());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: reaper,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(green, reaper), (blue, reaper)]))
        .expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != green), "the green blocker died");
    assert!(g.battlefield.iter().any(|c| c.id == blue), "the blue blocker survived");
}

/// Molimo's power and toughness track the lands you control.
#[test]
fn molimo_is_sized_by_your_lands() {
    let mut g = main_phase();
    let molimo = g.add_card_to_battlefield(0, catalog::molimo_maro_sorcerer());
    basics(&mut g, 0, LandType::Forest, 4);
    let cp = g.computed_permanent(molimo).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Yavimaya Kavu counts red creatures for power and green ones for toughness.
#[test]
fn yavimaya_kavu_counts_both_colors() {
    let mut g = main_phase();
    let kavu = g.add_card_to_battlefield(0, catalog::yavimaya_kavu());
    g.add_card_to_battlefield(0, catalog::ruby_leech());
    g.add_card_to_battlefield(1, catalog::utopia_tree());
    let cp = g.computed_permanent(kavu).unwrap();
    // The Kavu itself is red AND green.
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Kavu Monarch grants trample and grows on every other Kavu.
#[test]
fn kavu_monarch_grows_and_grants_trample() {
    let mut g = main_phase();
    let monarch = g.add_card_to_battlefield(0, catalog::kavu_monarch());
    let other = g.add_card_to_hand(0, catalog::kavu_scout());
    cast(&mut g, 0, other, None);
    assert_eq!(g.battlefield_find(monarch).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert!(g.computed_permanent(other).unwrap().keywords.contains(&Keyword::Trample));
}

/// Urza's Filter shaves {2} off a multicoloured spell.
#[test]
fn urzas_filter_discounts_gold_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::urzas_filter());
    let gold = g.add_card_to_hand(0, catalog::vicious_kavu());
    let card = g.players[0].hand.iter().find(|c| c.id == gold).unwrap();
    assert_eq!(crabomination::game::actions::cost_reduction_for_spell(&g, 0, card, None), 2);
}

/// Juntu Stakes keeps small creatures tapped through the untap step.
#[test]
fn juntu_stakes_locks_small_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::juntu_stakes());
    let small = g.add_card_to_battlefield(0, catalog::utopia_tree());
    let big = g.add_card_to_battlefield(0, catalog::jade_leech());
    for id in [small, big] {
        g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = true;
    }
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(small).unwrap().tapped, "0-power tree stayed tapped");
    assert!(!g.battlefield_find(big).unwrap().tapped, "5/5 untapped normally");
}

/// Crusading Knight swells with each Swamp the opponent controls.
#[test]
fn crusading_knight_feeds_on_opposing_swamps() {
    let mut g = main_phase();
    let knight = g.add_card_to_battlefield(0, catalog::crusading_knight());
    basics(&mut g, 1, LandType::Swamp, 3);
    basics(&mut g, 0, LandType::Swamp, 2); // your own Swamps don't count
    let cp = g.computed_permanent(knight).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Sparring Golem swells by the number of creatures blocking it.
#[test]
fn sparring_golem_grows_per_blocker() {
    let mut g = two_player_game();
    let golem = g.add_card_to_battlefield(0, catalog::sparring_golem());
    g.clear_sickness(golem);
    let a = g.add_card_to_battlefield(1, catalog::utopia_tree());
    let b = g.add_card_to_battlefield(1, catalog::jade_leech());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: golem,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(a, golem), (b, golem)]))
        .expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(golem).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}
