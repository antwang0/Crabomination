//! Invasion (INV) gap wave 4 — the Dragon Legends, the last kicker cards and
//! the enchantment shell.

use crabomination::card::{CounterType, Keyword, LandType};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
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

/// Swing `attacker` at seat 1 and resolve combat damage.
fn connect(g: &mut GameState, attacker: CardId) {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    mana(g, 0);
    g.resolve_combat().expect("combat damage");
    drain_stack(g);
}

// ── The Dragon Legends ──────────────────────────────────────────────────────

/// All five Dragons are 6/6 legendary fliers with one combat-damage trigger.
#[test]
fn inv_dragon_legends_share_their_shape() {
    for f in [
        catalog::crosis_the_purger as fn() -> _,
        catalog::dromar_the_banisher,
        catalog::treva_the_renewer,
        catalog::rith_the_awakener,
        catalog::darigaaz_the_igniter,
    ] {
        let def = f();
        assert_eq!((def.power, def.toughness), (6, 6), "{}", def.name);
        assert!(def.keywords.contains(&Keyword::Flying), "{}", def.name);
        assert_eq!(def.triggered_abilities.len(), 1, "{}", def.name);
    }
}

/// Treva gains life for each permanent of the colour it picks.
#[test]
fn treva_gains_life_per_permanent_of_a_color() {
    let mut g = main_phase();
    let treva = g.add_card_to_battlefield(0, catalog::treva_the_renewer());
    g.clear_sickness(treva);
    // Three green permanents on the board (the Leech, a Tree, a Forest).
    g.add_card_to_battlefield(1, catalog::jade_leech());
    g.add_card_to_battlefield(1, catalog::utopia_tree());
    g.add_card_to_battlefield(0, catalog::forest());
    let before = g.players[0].life;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    connect(&mut g, treva);
    // The auto-decider takes the first legal colour; any pick is a gain of at
    // least one, and the trigger must have resolved.
    assert!(g.players[0].life > before, "Treva's trigger paid out");
}

/// Rith mints a Saproling for each permanent of the colour it picks.
#[test]
fn rith_mints_saprolings() {
    let mut g = main_phase();
    let rith = g.add_card_to_battlefield(0, catalog::rith_the_awakener());
    g.clear_sickness(rith);
    g.add_card_to_battlefield(1, catalog::jade_leech());
    g.add_card_to_battlefield(1, catalog::utopia_tree());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    connect(&mut g, rith);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Saproling"),
        "at least one Saproling arrived"
    );
}

// ── Kicker ──────────────────────────────────────────────────────────────────

/// Skizzik sticks around only when it was kicked.
#[test]
fn skizzik_sacrifices_itself_unless_kicked() {
    let mut g = main_phase();
    let plain = g.add_card_to_hand(0, catalog::skizzik());
    cast(&mut g, 0, plain, None);
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != plain), "unkicked Skizzik left");

    let mut g = main_phase();
    let kicked = g.add_card_to_hand(0, catalog::skizzik());
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
    g.step = TurnStep::End;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == kicked), "kicked Skizzik stayed");
}

/// Verdeloth's kicked X becomes X Saprolings, and its lord pumps them.
#[test]
fn verdeloth_kicked_makes_pumped_saprolings() {
    let mut g = main_phase();
    let verdeloth = g.add_card_to_hand(0, catalog::verdeloth_the_ancient());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: verdeloth,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    let saps: Vec<_> =
        g.battlefield.iter().filter(|c| c.definition.name == "Saproling").map(|c| c.id).collect();
    assert_eq!(saps.len(), 3);
    let cp = g.computed_permanent(saps[0]).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "the Treefolk lord pumps Saprolings");
}

/// Kangee's kicked X becomes counters that pump other Birds.
#[test]
fn kangee_pumps_other_birds_per_counter() {
    let mut g = main_phase();
    let kangee = g.add_card_to_hand(0, catalog::kangee_aerie_keeper());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: kangee,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kangee).unwrap().counter_count(CounterType::Feather), 2);
    let bird = g.add_card_to_battlefield(0, catalog::rainbow_crow());
    let cp = g.computed_permanent(bird).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
    // Kangee doesn't pump itself.
    let cp = g.computed_permanent(kangee).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2));
}

/// Vigorous Charge's kicker adds lifelink for the turn.
#[test]
fn vigorous_charge_kicked_grants_lifelink() {
    let mut g = main_phase();
    let target = g.add_card_to_battlefield(0, catalog::noble_panther());
    let spell = g.add_card_to_hand(0, catalog::vigorous_charge());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: spell,
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(target).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample));
    assert!(cp.keywords.contains(&Keyword::Lifelink));
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Devouring Strossus eats a creature at every upkeep.
#[test]
fn devouring_strossus_eats_a_creature_each_upkeep() {
    let mut g = main_phase();
    let strossus = g.add_card_to_battlefield(0, catalog::devouring_strossus());
    let snack = g.add_card_to_battlefield(0, catalog::noble_panther());
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let alive = [strossus, snack].iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(alive, 1, "exactly one creature was sacrificed");
}

/// Phyrexian Infiltrator trades itself for something across the table.
#[test]
fn phyrexian_infiltrator_swaps_control() {
    let mut g = main_phase();
    let infiltrator = g.add_card_to_battlefield(0, catalog::phyrexian_infiltrator());
    let prize = g.add_card_to_battlefield(1, catalog::noble_panther());
    activate(&mut g, 0, infiltrator, 0, Some(Target::Permanent(prize)));
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(infiltrator).unwrap().controller, 1);
}

/// Slimy Kavu turns a land into a Swamp for the turn.
#[test]
fn slimy_kavu_makes_a_swamp() {
    let mut g = main_phase();
    let kavu = g.add_card_to_battlefield(0, catalog::slimy_kavu());
    g.clear_sickness(kavu);
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    activate(&mut g, 0, kavu, 0, Some(Target::Permanent(land)));
    assert!(g.computed_permanent(land).unwrap().subtypes.land_types.contains(&LandType::Swamp));
}

/// Tek reads the basic land types you control.
#[test]
fn tek_scales_with_your_basic_types() {
    let mut g = main_phase();
    let tek = g.add_card_to_battlefield(0, catalog::tek());
    let cp = g.computed_permanent(tek).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "no basics, no bonuses");
    g.add_card_to_battlefield(0, catalog::swamp());
    g.add_card_to_battlefield(0, catalog::island());
    let cp = g.computed_permanent(tek).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "Swamp is +2/+0");
    assert!(cp.keywords.contains(&Keyword::Flying), "Island grants flying");
}

/// Pyre Zombie goes out shooting for {1}{R}{R}.
#[test]
fn pyre_zombie_sacrifices_for_two_damage() {
    let mut g = main_phase();
    let zombie = g.add_card_to_battlefield(0, catalog::pyre_zombie());
    activate(&mut g, 0, zombie, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    assert!(g.battlefield.iter().all(|c| c.id != zombie));
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Teferi's Moat keeps the chosen colour's ground creatures at home.
#[test]
fn teferis_moat_walls_off_a_color() {
    let mut g = two_player_game();
    let moat = g.add_card_to_hand(1, catalog::teferis_moat());
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    cast(&mut g, 1, moat, None);
    let chosen = g.battlefield_find(moat).unwrap().chosen_color.expect("a colour was chosen");

    // A ground creature of the chosen colour can't attack the Moat's controller.
    let factory = match chosen {
        Color::White => catalog::ruham_djinn as fn() -> _,
        Color::Blue => catalog::zanam_djinn,
        Color::Black => catalog::goham_djinn,
        Color::Red => catalog::halam_djinn,
        Color::Green => catalog::sulam_djinn,
    };
    let attacker = g.add_card_to_battlefield(0, factory());
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]));
    // Zanam Djinn flies, so it is exempt; every other pick is walled off.
    if chosen == Color::Blue {
        assert!(res.is_ok(), "the flier ignores the Moat");
    } else {
        assert!(res.is_err(), "a ground creature of the chosen colour can't attack");
    }
}

/// Spirit of Resistance blanks damage to you once all five colours are out.
#[test]
fn spirit_of_resistance_needs_all_five_colors() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::spirit_of_resistance());
    let mut events = Vec::new();
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut events);
    assert_eq!(g.players[0].life, 17, "four colours short — damage lands");

    for f in [
        catalog::alabaster_leech as fn() -> _,
        catalog::sapphire_leech,
        catalog::andradite_leech,
        catalog::ruby_leech,
        catalog::jade_leech,
    ] {
        g.add_card_to_battlefield(0, f());
    }
    let life = g.players[0].life;
    g.deal_damage_to_from(crabomination::game::effects::EntityRef::Player(0), 3, None, &mut events);
    assert_eq!(g.players[0].life, life, "all five colours — damage is prevented");
}

/// Tectonic Instability taps out whoever plays a land.
#[test]
fn tectonic_instability_taps_the_lands_out() {
    let mut g = main_phase();
    g.add_card_to_battlefield(1, catalog::tectonic_instability());
    let old = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield.iter_mut().find(|c| c.id == old).unwrap().tapped = false;
    let drop = g.add_card_to_hand(0, catalog::mountain());
    g.perform_action(GameAction::PlayLand(drop)).expect("land drop");
    drain_stack(&mut g);
    assert!(g.battlefield_find(old).unwrap().tapped, "the trigger tapped the whole board");
    assert!(g.battlefield_find(drop).unwrap().tapped);
}

/// Saproling Infestation pays out whenever anyone kicks a spell.
#[test]
fn saproling_infestation_reads_every_kick() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::saproling_infestation());
    let kicked = g.add_card_to_hand(1, catalog::kavu_titan());
    g.active_player_idx = 1;
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: kicked,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Saproling" && c.controller == 0),
        "the enchantment's controller got the token"
    );
}

/// Saproling Symbiosis doubles your board.
#[test]
fn saproling_symbiosis_matches_your_creature_count() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::noble_panther());
    }
    let spell = g.add_card_to_hand(0, catalog::saproling_symbiosis());
    cast(&mut g, 0, spell, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(),
        3
    );
}

/// Teferi's Care eats an enchantment to destroy one.
#[test]
fn teferis_care_trades_enchantments() {
    let mut g = main_phase();
    let care = g.add_card_to_battlefield(0, catalog::teferis_care());
    let fodder = g.add_card_to_battlefield(0, catalog::dueling_grounds());
    let victim = g.add_card_to_battlefield(1, catalog::saproling_infestation());
    activate(&mut g, 0, care, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield.iter().all(|c| c.id != victim));
    assert!(g.battlefield.iter().all(|c| c.id != fodder), "an enchantment was the cost");
}

// ── CR 601.2b — the "pay {2} more for flash" rider ──────────────────────────

/// Ghitu Fire costs its printed {X}{R} at sorcery speed and {2} more when it
/// is cast outside it.
#[test]
fn ghitu_fire_pays_two_more_for_flash() {
    let mut g = main_phase();
    let card = g.add_card_to_hand(0, catalog::ghitu_fire());
    let inst = g.players[0].hand.iter().find(|c| c.id == card).unwrap();
    assert!(g.flash_surcharge_for(0, inst).is_none(), "main phase pays the printed cost");

    // Off sorcery timing the surcharge applies — and the cast is now legal.
    g.step = TurnStep::DeclareBlockers;
    let inst = g.players[0].hand.iter().find(|c| c.id == card).unwrap();
    assert_eq!(g.flash_surcharge_for(0, inst).map(|c| c.cmc()), Some(2));
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: card,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("flash cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17);
}

/// Breaking Wave flips every creature's tapped state.
#[test]
fn breaking_wave_swaps_tapped_and_untapped() {
    let mut g = main_phase();
    let tapped = g.add_card_to_battlefield(0, catalog::noble_panther());
    let untapped = g.add_card_to_battlefield(1, catalog::noble_panther());
    g.battlefield.iter_mut().find(|c| c.id == tapped).unwrap().tapped = true;
    g.battlefield.iter_mut().find(|c| c.id == untapped).unwrap().tapped = false;
    let spell = g.add_card_to_hand(0, catalog::breaking_wave());
    cast(&mut g, 0, spell, None);
    assert!(!g.battlefield_find(tapped).unwrap().tapped);
    assert!(g.battlefield_find(untapped).unwrap().tapped);
}

/// Void wipes the named mana value off the board and out of the target's hand.
#[test]
fn void_clears_one_mana_value_everywhere() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    // Noble Panther is mana value 3; Jade Leech is 4.
    let hit = g.add_card_to_battlefield(1, catalog::noble_panther());
    let miss = g.add_card_to_battlefield(1, catalog::jade_leech());
    let in_hand = g.add_card_to_hand(1, catalog::noble_panther());
    let safe_land = g.add_card_to_hand(1, catalog::mountain());
    let spell = g.add_card_to_hand(0, catalog::void());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(3)]));
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert!(g.battlefield.iter().all(|c| c.id != hit), "the 3-drop died");
    assert!(g.battlefield.iter().any(|c| c.id == miss), "the 4-drop survived");
    assert!(g.players[1].hand.iter().all(|c| c.id != in_hand), "the 3-drop was discarded");
    assert!(g.players[1].hand.iter().any(|c| c.id == safe_land), "lands are exempt");
}

/// Prison Barricade's kicked half grants both the counter and the defender
/// bypass; the unkicked copy stays home.
#[test]
fn prison_barricade_kicked_can_attack() {
    let mut g = main_phase();
    let plain = g.add_card_to_hand(0, catalog::prison_barricade());
    cast(&mut g, 0, plain, None);
    g.battlefield_find_mut(plain).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: plain,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "an unkicked Wall still has defender"
    );

    let mut g = main_phase();
    let kicked = g.add_card_to_hand(0, catalog::prison_barricade());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: kicked,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kicked).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    g.battlefield_find_mut(kicked).unwrap().summoning_sick = false;
    connect(&mut g, kicked);
    assert_eq!(g.players[1].life, 18, "the kicked 2/4 Wall connected");
}
