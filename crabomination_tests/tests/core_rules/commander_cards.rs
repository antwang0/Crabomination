//! Cards whose printed text is *about* the Commander format (CR 903) and
//! which need a real commander on the board to do anything — the half of
//! `sets::cmdr` that `format.rs`'s deck validation cannot reach.

use crabomination::card::{
    CardDefinition, CardId, CardType, CreatureType, Keyword, Subtypes, Supertype,
};
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::catalog;
use crabomination::format::Format;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::{Color, b, cost, g, generic, r, u, w};

/// A free-to-name legendary Bear so a seat has a commander to control.
fn bear_commander() -> CardDefinition {
    CardDefinition {
        name: "Test Bear Commander",
        cost: cost(&[g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// A two-seat Commander game with seat 0 holding priority in its own
/// pre-combat main phase and its commander still in the command zone.
fn commander_game() -> GameState {
    let mut g = game_with_format(Format::Commander, 2);
    g.seat_commanders(0, vec![bear_commander()]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

/// Seat 0's commander, on the battlefield. CR 903.3 makes "is a commander" a
/// property of the `CardId`, not of the zone, so putting the instance on the
/// battlefield and registering that id is the same board the command-zone
/// cast path would produce — and this way the tests are about the two cards
/// under test rather than about casting.
fn commander_on_the_battlefield(g: &mut GameState) -> CardId {
    let id = g.add_card_to_battlefield(0, bear_commander());
    g.players[0].commanders.push(id);
    id
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let cp = g.computed_permanent(id).expect("on battlefield");
    (cp.power, cp.toughness)
}

fn has_kw(g: &GameState, id: CardId, kw: Keyword) -> bool {
    g.computed_permanent(id).unwrap().keywords().contains(&kw)
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── Bastion Protector ───────────────────────────────────────────────────────

/// CR 903.3 — "Commander creatures you control get +2/+2 and have
/// indestructible." The anthem is keyed on `IsCommander`, so it reaches the
/// commander and nothing else on the board.
#[test]
fn cr_903_3_bastion_protector_pumps_only_the_commander() {
    let mut g = commander_game();
    let cmd = commander_on_the_battlefield(&mut g);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let protector = g.add_card_to_battlefield(0, catalog::bastion_protector());

    assert_eq!(pt(&g, cmd), (4, 4), "2/2 commander under +2/+2");
    assert!(has_kw(&g, cmd, Keyword::Indestructible), "the commander is indestructible");

    assert_eq!(pt(&g, bears), (2, 2), "a non-commander gets nothing");
    assert!(!has_kw(&g, bears, Keyword::Indestructible));
    assert_eq!(pt(&g, protector), (3, 3), "and neither does the Protector");
    assert!(!has_kw(&g, protector, Keyword::Indestructible));
}

/// With the commander still in the command zone there is nothing on the
/// battlefield for the anthem to find, so the Protector is a vanilla 3/3.
#[test]
fn bastion_protector_finds_nothing_while_the_commander_is_in_the_zone() {
    let mut g = commander_game();
    let protector = g.add_card_to_battlefield(0, catalog::bastion_protector());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(pt(&g, protector), (3, 3));
    assert_eq!(pt(&g, bears), (2, 2));
    assert!(!has_kw(&g, bears, Keyword::Indestructible));
}

// ── Loyal Apprentice ────────────────────────────────────────────────────────

fn thopters(g: &GameState, seat: usize) -> Vec<CardId> {
    g.battlefield
        .iter()
        .filter(|c| c.definition.name == "Thopter" && c.controller == seat)
        .map(|c| c.id)
        .collect()
}

/// Lieutenant (CR 207.2c is the ability word; the gate is the printed "if you
/// control your commander"). With the commander on the board the beginning of
/// combat mints a Thopter, and it has haste the turn it arrives.
#[test]
fn lieutenant_loyal_apprentice_mints_a_hasty_thopter_with_its_commander_out() {
    let mut g = commander_game();
    commander_on_the_battlefield(&mut g);
    g.add_card_to_battlefield(0, catalog::loyal_apprentice());
    assert!(thopters(&g, 0).is_empty(), "nothing before combat");

    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);

    let made = thopters(&g, 0);
    assert_eq!(made.len(), 1, "one 1/1 Thopter");
    let t = g.battlefield_find(made[0]).unwrap();
    assert_eq!((t.definition.power, t.definition.toughness), (1, 1));
    assert!(t.definition.card_types.contains(&CardType::Artifact));
    assert!(has_kw(&g, made[0], Keyword::Flying), "printed on the token");
    assert!(
        has_kw(&g, made[0], Keyword::Haste),
        "granted to the token, not printed on it",
    );
}

/// Without the commander on the battlefield the Lieutenant clause does
/// nothing — the trigger still fires, its intervening "if" is what fails.
#[test]
fn loyal_apprentice_mints_nothing_without_its_commander() {
    let mut g = commander_game();
    g.add_card_to_battlefield(0, catalog::loyal_apprentice());
    advance_to(&mut g, TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert!(thopters(&g, 0).is_empty(), "no commander, no Thopter");
}

// ── War Room ────────────────────────────────────────────────────────────────

/// A two-colour legendary Bear, so the identity count is a number worth
/// asserting rather than 0 or 1.
fn azorius_commander() -> CardDefinition {
    CardDefinition {
        name: "Test Azorius Commander",
        cost: cost(&[w(), u()]),
        ..bear_commander()
    }
}

fn war_room_draw(g: &mut GameState, land: CardId) -> Result<Vec<GameEvent>, GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
}

/// CR 903.4 — the life paid is the number of colours in the seat's
/// commanders' colour identity, and it is a **cost**: paid on activation,
/// before anything resolves.
#[test]
fn cr_903_4_war_room_pays_one_life_per_colour_of_the_commanders_identity() {
    let mut g = game_with_format(Format::Commander, 2);
    g.seat_commanders(0, vec![azorius_commander()]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::war_room());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;

    war_room_draw(&mut g, land).expect("{3}, {T}, pay 2 life: draw");
    assert_eq!(g.players[0].life, life - 2, "two colours, two life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1, "drew a card");
    assert!(g.battlefield_find(land).unwrap().tapped);
}

/// The cost gate: a seat that cannot pay cannot activate, and it does not
/// lose the tap finding out.
#[test]
fn war_room_refuses_the_activation_when_the_life_is_not_there() {
    let mut g = game_with_format(Format::Commander, 2);
    g.seat_commanders(0, vec![azorius_commander()]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::war_room());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].life = 1;

    assert!(matches!(war_room_draw(&mut g, land), Err(GameError::InsufficientLife)));
    assert!(!g.battlefield_find(land).unwrap().tapped, "the tap is not burned");
}

/// Outside a Commander game the seat has no commander, so the count is zero
/// and the ability is a plain `{3}, {T}: Draw a card` — the same reading
/// `ManaPayload::AnyColorInCommanderIdentity` takes for a seat with no
/// commander, and what keeps the card playable in a cube.
#[test]
fn war_room_costs_no_life_with_no_commander() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::war_room());
    g.players[0].hand.clear();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[0].life;

    war_room_draw(&mut g, land).expect("no commander, no life");
    assert_eq!(g.players[0].life, life);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// And the colourless half is still just a land.
#[test]
fn war_room_taps_for_colourless() {
    let mut g = two_player_game();
    g.step = TurnStep::PreCombatMain;
    let land = g.add_card_to_battlefield(0, catalog::war_room());
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("{T}: Add {C}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1);
}

// ── Protection from each colour outside the commander identity ──────────────
//
// `Keyword::ProtectionFromColorsOutsideCommanderIdentity` (Commander's Plate).
// The protected set is the *complement* of a colour set the game state owns,
// which makes it the first protection keyword whose answer changes with the
// seat rather than with the permanent — and the engine asks that question at
// three separate gates (damage, targeting, blocking), so each gets a test.

/// A creature whose only colour comes from its cost, so the computed colour
/// is exactly what the pips say.
fn creature_costing(name: &'static str, c: crabomination::mana::ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    }
}

/// The bearer: a 2/2 carrying the keyword and nothing else.
fn plated_bear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromColorsOutsideCommanderIdentity],
        ..creature_costing("Test Plated Bear", cost(&[g()]))
    }
}

/// A seat-0 Commander game whose commander is the Azorius (W/U) Bear, so
/// "inside the identity" and "outside it" are both non-trivial.
fn azorius_commander_game() -> GameState {
    let mut g = game_with_format(Format::Commander, 2);
    g.seat_commanders(0, vec![azorius_commander()]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

/// CR 702.16 / CR 903.4 — the damage gate. "Protection from each color that's
/// not in your commander's color identity" is one protection per colour
/// outside the identity, so a red source is stopped by a W/U commander's
/// identity and a white one is not. A colourless source has no colour outside
/// the identity at all and always gets through (CR 702.16 — protection is
/// always *from a quality*, and colourless is not one of the five).
#[test]
fn cr_903_4_plate_protection_prevents_damage_only_from_off_identity_colours() {
    let mut g = azorius_commander_game();
    let bear = g.add_card_to_battlefield(0, plated_bear());
    let red = g.add_card_to_battlefield(1, creature_costing("Test Red", cost(&[r()])));
    let white = g.add_card_to_battlefield(1, creature_costing("Test White", cost(&[w()])));
    let grey = g.add_card_to_battlefield(1, creature_costing("Test Grey", cost(&[generic(2)])));

    assert!(g.damage_prevented_by_protection(red, bear), "red is outside W/U");
    assert!(!g.damage_prevented_by_protection(white, bear), "white is inside W/U");
    assert!(!g.damage_prevented_by_protection(grey, bear), "a colourless source has no colour");
}

/// CR 702.16c — the ability-targeting gate reads the same set.
#[test]
fn cr_702_16c_plate_protection_stops_an_off_identity_ability_from_targeting() {
    let mut g = azorius_commander_game();
    let bear = g.add_card_to_battlefield(0, plated_bear());
    let red = g.add_card_to_battlefield(1, creature_costing("Test Red", cost(&[r()])));
    let blue = g.add_card_to_battlefield(1, creature_costing("Test Blue", cost(&[u()])));

    assert!(g.ability_target_has_protection(&Target::Permanent(bear), red), "red is outside W/U");
    assert!(
        !g.ability_target_has_protection(&Target::Permanent(bear), blue),
        "blue is inside W/U",
    );
}

/// CR 702.16b — the blocking gate. The protected creature is the *attacker*
/// here, and the question is about the blocker's colours; this is the gate the
/// pure `can_block_attacker_computed` cannot answer, because the identity
/// lives on the attacker's controller rather than on either permanent.
#[test]
fn cr_702_16b_plate_protection_bars_a_block_by_an_off_identity_creature() {
    let mut g = azorius_commander_game();
    let attacker = g.add_card_to_battlefield(0, plated_bear());
    let red = g.add_card_to_battlefield(1, creature_costing("Test Red", cost(&[r()])));
    let white = g.add_card_to_battlefield(1, creature_costing("Test White", cost(&[w()])));

    assert!(!g.blocker_can_block_attacker(red, attacker), "red is outside W/U");
    assert!(g.blocker_can_block_attacker(white, attacker), "white is inside W/U");
}

/// CR 903.4 — a seat with **no commander** has an empty colour identity, so
/// every colour is outside it and the bearer has protection from all five.
/// ⚠ This is the reading `GameState::commander_identity_colors` deliberately
/// does *not* take (it answers all five so Command Tower stays a fixing land
/// in the two-player pool); read as a protection set that fallback would mean
/// protection from nothing at all. The raw `commander_identity_set` is what
/// this keyword asks, and it answers empty.
#[test]
fn cr_903_4_plate_protection_covers_every_colour_for_a_seat_with_no_commander() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, plated_bear());
    for (name, pip) in [
        ("Test W", w()),
        ("Test U", u()),
        ("Test B", b()),
        ("Test R", r()),
        ("Test G", crabomination::mana::g()),
    ] {
        let src = g.add_card_to_battlefield(1, creature_costing(name, cost(&[pip])));
        assert!(
            g.damage_prevented_by_protection(src, bear),
            "{name}: no commander means every colour is outside the identity",
        );
    }
    let grey = g.add_card_to_battlefield(1, creature_costing("Test Grey", cost(&[generic(2)])));
    assert!(
        !g.damage_prevented_by_protection(grey, bear),
        "and a colourless source still gets through — it has no colour to be protected from",
    );
}

/// CR 702.16 — and the fourth gate: a **spell** still in hand is checked
/// card-side, before it ever reaches the stack, so it needs its own arm and
/// its own test. Lightning Bolt is red and a W/U identity shuts it out; the
/// blue counterpart is inside the identity and is let through.
#[test]
fn cr_702_16_plate_protection_stops_an_off_identity_spell_at_cast_time() {
    let mut g = azorius_commander_game();
    let bear = g.add_card_to_battlefield(0, plated_bear());
    g.priority.player_with_priority = 1;

    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 1);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        matches!(err, Err(GameError::TargetHasProtection(_))),
        "a red spell is outside W/U, got {err:?}",
    );

    let bounce = g.add_card_to_hand(1, catalog::unsummon());
    g.players[1].mana_pool.add(crabomination::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bounce,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("a blue spell is inside W/U");
}

// ── Commander's Plate ───────────────────────────────────────────────────────

/// CR 903.4 — the card, on a real board: +3/+3 and protection from every
/// colour outside the W/U commander's identity.
#[test]
fn cr_903_4_commanders_plate_grants_the_pump_and_the_off_identity_protection() {
    let mut g = azorius_commander_game();
    let cmd = g.add_card_to_battlefield(0, azorius_commander());
    g.players[0].commanders.push(cmd);
    let plate = g.add_card_to_battlefield(0, catalog::commanders_plate());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: plate, target: cmd })
        .expect("Equip commander {3}");

    assert_eq!(pt(&g, cmd), (5, 5), "2/2 under +3/+3");
    let red = g.add_card_to_battlefield(1, creature_costing("Test Red", cost(&[r()])));
    let white = g.add_card_to_battlefield(1, creature_costing("Test White", cost(&[w()])));
    assert!(g.damage_prevented_by_protection(red, cmd), "red is outside W/U");
    assert!(!g.damage_prevented_by_protection(white, cmd), "white is inside W/U");
}

/// "Equip commander {3}. Equip {5}." — the restricted cost is the whole point
/// of the card and it is five mana cheaper than the printed one, so three
/// colourless equips the commander and refuses anything else.
#[test]
fn cr_903_3_commanders_plate_equips_a_commander_for_three_and_anything_else_for_five() {
    let mut g = azorius_commander_game();
    let cmd = g.add_card_to_battlefield(0, azorius_commander());
    g.players[0].commanders.push(cmd);
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let plate = g.add_card_to_battlefield(0, catalog::commanders_plate());

    g.players[0].mana_pool.add_colorless(3);
    assert!(
        g.perform_action(GameAction::Equip { equipment: plate, target: bears }).is_err(),
        "a non-commander pays the printed Equip {{5}}",
    );
    g.perform_action(GameAction::Equip { equipment: plate, target: cmd })
        .expect("Equip commander {3}");

    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::Equip { equipment: plate, target: bears })
        .expect("and five moves it to anything");
    assert_eq!(pt(&g, bears), (5, 5), "the Plate pumps whatever wears it");
}

// ── Nekusar, the Mindrazer ──────────────────────────────────────────────────
//
// Not a card *about* the format — it reads no command zone and no colour
// identity — but its whole text is about the table, so the only test that
// says anything is a multi-seat one. It lives here because this is where the
// branch's multi-seat card tests are.

/// "Whenever an opponent draws a card, Nekusar deals 1 damage to that player."
/// ⚠ "that player", not "each opponent": at four seats one opponent's draw
/// costs that opponent one life and the other two nothing.
#[test]
fn nekusar_pings_only_the_opponent_who_drew() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::nekusar_the_mindrazer());
    for seat in 0..4 {
        g.add_card_to_library(seat, catalog::grizzly_bears());
    }
    let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();

    let mut events = Vec::new();
    g.draw_one(2, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);

    assert_eq!(g.players[2].life, before[2] - 1, "the drawer takes 1");
    for seat in [0usize, 1, 3] {
        assert_eq!(g.players[seat].life, before[seat], "seat {seat} is untouched");
    }
}

/// "At the beginning of each player's draw step, that player draws an
/// additional card" — the symmetric half, which helps Nekusar's controller
/// too. One extra card for whoever's draw step it is, and nobody else.
#[test]
fn nekusar_draws_an_extra_card_for_whoevers_draw_step_it_is() {
    let mut g = multi_player_game(4);
    g.add_card_to_battlefield(0, catalog::nekusar_the_mindrazer());
    for seat in 0..4 {
        for _ in 0..5 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
        g.players[seat].hand.clear();
    }
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::Upkeep;

    let hands: Vec<usize> = g.players.iter().map(|p| p.hand.len()).collect();
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);

    assert_eq!(
        g.players[1].hand.len(),
        hands[1] + 2,
        "the active player's own draw plus Nekusar's extra",
    );
    for seat in [0usize, 2, 3] {
        assert_eq!(g.players[seat].hand.len(), hands[seat], "seat {seat} draws nothing");
    }
}

// ── Kenrith, the Returned King ──────────────────────────────────────────────

fn activate(g: &mut GameState, id: CardId, i: usize, target: Option<Target>) -> Result<Vec<GameEvent>, GameError> {
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: i,
        target,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
}

/// "{R}: **All** creatures gain trample and haste until end of turn" — every
/// seat's creatures, not the controller's. The one clause on Kenrith with no
/// target, and the one a two-seat test would half-answer.
#[test]
fn kenrith_grants_trample_and_haste_to_every_seats_creatures() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    let kenrith = g.add_card_to_battlefield(0, catalog::kenrith_the_returned_king());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Red, 1);

    activate(&mut g, kenrith, 0, None).expect("{R}");
    drain_stack(&mut g);
    for id in [mine, theirs] {
        assert!(has_kw(&g, id, Keyword::Trample), "trample");
        assert!(has_kw(&g, id, Keyword::Haste), "haste");
    }
}

/// "{2}{W}: **Target player** gains 5 life" and "{3}{U}: Target player draws a
/// card" — both can aim at an opponent, which is the whole point of the card.
#[test]
fn kenrith_can_aim_its_gifts_at_an_opponent() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    let kenrith = g.add_card_to_battlefield(0, catalog::kenrith_the_returned_king());
    g.add_card_to_library(2, catalog::grizzly_bears());
    g.players[2].hand.clear();
    let life = g.players[2].life;
    g.players[0].mana_pool.add_colorless(5);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);

    activate(&mut g, kenrith, 2, Some(Target::Player(2))).expect("{2}{W} at seat 2");
    drain_stack(&mut g);
    assert_eq!(g.players[2].life, life + 5, "the opponent gains the life");

    activate(&mut g, kenrith, 3, Some(Target::Player(2))).expect("{3}{U} at seat 2");
    drain_stack(&mut g);
    assert_eq!(g.players[2].hand.len(), 1, "and draws the card");
}

/// "{4}{B}: Put target creature card from **a** graveyard onto the
/// battlefield under **its owner's** control" — any graveyard, and the
/// creature lands under its owner rather than under Kenrith's controller.
#[test]
fn kenrith_reanimates_into_its_owners_control() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    let kenrith = g.add_card_to_battlefield(0, catalog::kenrith_the_returned_king());
    let corpse = g.add_card_to_graveyard(2, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Black, 1);

    activate(&mut g, kenrith, 4, Some(Target::Permanent(corpse))).expect("{4}{B}");
    drain_stack(&mut g);
    let back = g.battlefield_find(corpse).expect("on the battlefield");
    assert_eq!(back.controller, 2, "under its OWNER's control, not the activator's");
    assert!(g.players[2].graveyard.is_empty(), "and it left that graveyard");
}

// ── Zedruu the Greathearted ─────────────────────────────────────────────────

/// Zedruu's X is "permanents you **own** that your **opponents control**" —
/// a set that stays empty until something is given away, and the one clause
/// on any of these cards that needs ownership and control to be different
/// things.
#[test]
fn zedruu_counts_only_what_it_gave_away() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let zedruu = g.add_card_to_battlefield(0, catalog::zedruu_the_greathearted());
    let gift = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // A permanent seat 2 both owns and controls must not count for Zedruu.
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
    }
    g.players[0].hand.clear();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);

    g.perform_action(GameAction::ActivateAbility {
        card_id: zedruu,
        ability_index: 0,
        target: Some(Target::Player(2)),
        additional_targets: vec![Target::Permanent(gift)],
        x_value: None,
        mode: None,
    })
    .expect("{U}{R}{W}: hand the Bears to seat 2");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(gift).unwrap().controller,
        2,
        "seat 2 controls it now",
    );

    // ⚠ Stop IN the upkeep, not at the draw step: entering Draw takes the
    // turn-based draw too, and a hand of two would not say whose card was
    // whose.
    let life = g.players[0].life;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "X is one: the Bears it gave away");
    assert_eq!(g.players[0].hand.len(), 1, "and one card, before the turn's own draw");
}

// ── Phelddagrif ─────────────────────────────────────────────────────────────

/// Each of Phelddagrif's three abilities pairs a benefit for its controller
/// with a gift the OPPONENT receives, and the three gifts take three
/// different shapes: a token the opponent creates, life the opponent gains,
/// and a draw the opponent chooses.
#[test]
fn phelddagrif_gives_each_gift_to_the_targeted_opponent() {
    let hippos = |g: &GameState, seat: usize| {
        g.battlefield
            .iter()
            .filter(|c| c.definition.name == "Hippo" && c.controller == seat)
            .count()
    };
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    let hippo_dad = g.add_card_to_battlefield(0, catalog::phelddagrif());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    let life = g.players[2].life;

    activate(&mut g, hippo_dad, 0, Some(Target::Player(2))).expect("{G}");
    drain_stack(&mut g);
    assert!(has_kw(&g, hippo_dad, Keyword::Trample), "it gets the trample");
    assert_eq!(hippos(&g, 2), 1, "seat 2 creates the Hippo");
    assert_eq!(hippos(&g, 0), 0, "and seat 0 does not");

    activate(&mut g, hippo_dad, 1, Some(Target::Player(2))).expect("{W}");
    drain_stack(&mut g);
    assert!(has_kw(&g, hippo_dad, Keyword::Flying), "it gets the flying");
    assert_eq!(g.players[2].life, life + 2, "seat 2 gains the life");
}

/// "{U}: Return Phelddagrif to its owner's hand. Target opponent **may** draw
/// a card." ⚠ Two things worth pinning here. The "may" is the OPPONENT's, so
/// the ask is routed to them (`MayDoBy`, not `MayDo`) — and the engine's
/// `AutoDecider` **declines** every optional body, so the default path is a
/// bounce with no draw. The gift only lands when the asked seat says yes,
/// which is what the scripted half asserts.
#[test]
fn phelddagrif_bounces_itself_and_offers_the_opponent_the_draw() {
    let setup = || {
        let mut g = multi_player_game(4);
        g.step = TurnStep::PreCombatMain;
        let id = g.add_card_to_battlefield(0, catalog::phelddagrif());
        g.add_card_to_library(2, catalog::grizzly_bears());
        g.players[2].hand.clear();
        g.players[0].mana_pool.add(Color::Blue, 1);
        (g, id)
    };

    // Declined (the AutoDecider's answer): the bounce still happens.
    let (mut g, hippo_dad) = setup();
    activate(&mut g, hippo_dad, 2, Some(Target::Player(2))).expect("{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hippo_dad).is_none(), "it left the battlefield");
    assert_eq!(g.players[2].hand.len(), 0, "a declined 'may' draws nothing");

    // Accepted: the opponent takes the card, and it is the OPPONENT's draw.
    let (mut g, hippo_dad) = setup();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    activate(&mut g, hippo_dad, 2, Some(Target::Player(2))).expect("{U}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hippo_dad).is_none(), "it still left");
    assert_eq!(g.players[2].hand.len(), 1, "seat 2 drew");
    assert!(g.players[0].hand.iter().all(|c| c.definition.name != "Grizzly Bears"),
        "and seat 0 did not");
}

// ── Miirym, Sentinel Wyrm ───────────────────────────────────────────────────

fn dragons(g: &GameState, seat: usize) -> usize {
    g.battlefield
        .iter()
        .filter(|c| {
            c.controller == seat
                && c.definition.subtypes.creature_types.contains(&CreatureType::Dragon)
        })
        .count()
}

/// Miirym copies another **nontoken** Dragon you control as it enters, and
/// the copy is **not legendary**.
///
/// ⚠ Both riders are load-bearing and the test pins both. If the trigger
/// fired on tokens, the copy would itself be a Dragon entering under your
/// control and the engine would be right to fire again forever. If the copy
/// kept its legendary supertype, CR 704.5j would put one of the pair into the
/// graveyard as soon as state-based actions ran — the copy of a *legendary*
/// Dragon is exactly the case that matters.
#[test]
fn miirym_copies_a_nontoken_dragon_once_and_not_as_a_legend() {
    let mut g = multi_player_game(4);
    g.step = TurnStep::PreCombatMain;
    g.add_card_to_battlefield(0, catalog::miirym_sentinel_wyrm());
    assert_eq!(dragons(&g, 0), 1, "Miirym is the only Dragon so far");

    // A legendary Dragon: the copy must drop the supertype or CR 704.5j eats
    // one of them.
    let original = g.add_card_to_battlefield(0, catalog::shivan_dragon());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: original }]);
    drain_stack(&mut g);

    assert_eq!(dragons(&g, 0), 3, "Miirym, the Dragon, and exactly one copy");
    let copies: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.is_token && c.definition.name == "Shivan Dragon")
        .collect();
    assert_eq!(copies.len(), 1, "one token copy");
    assert!(!copies[0].definition.is_legendary(), "and it is not legendary");

    // The token's own entry must not have fired the trigger again.
    drain_stack(&mut g);
    assert_eq!(dragons(&g, 0), 3, "nontoken-only: the copy does not copy itself");
}

/// CR 903.8 / 601.2a — a spell cast from the command zone was not cast from a
/// graveyard (bug fix: `Predicate::CastFromGraveyard` read "not cast from
/// hand", so Ash Zealot dealt 3 to every player who cast their commander).
#[test]
fn a_command_zone_cast_is_not_a_graveyard_cast() {
    let mut g = commander_game();
    g.add_card_to_battlefield(1, catalog::ash_zealot());
    let cmd = g.players[0].command[0].id;
    g.players[0].mana_pool.add(Color::Green, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastFromCommandZone {
        card_id: cmd,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        alternative: false,
        pitch_card: None,
    })
    .expect("cast the commander");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cmd).is_some());
    assert_eq!(g.players[0].life, life, "Ash Zealot stays quiet");
}

// ── Primitives for Arcane Wizardry (C17, Inalla) ──────────────────────────

/// CR 707.2 — "You may have this creature enter as a copy of any creature
/// card in a graveyard" (Body Double): an `EntersAsCopy` with
/// `from_graveyards` copies a card in *any* graveyard, not a permanent.
#[test]
fn cr_707_2_enters_as_a_copy_of_a_creature_card_in_a_graveyard() {
    use crabomination::card::{EntersAsCopy, SelectionRequirement as R};
    let copier = CardDefinition {
        name: "Test Graveyard Copier",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            from_graveyards: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut g = commander_game();
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, copier);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("the copier survived as a 2/2");
    assert_eq!(c.definition.name, "Grizzly Bears");
    assert_eq!(pt(&g, id), (2, 2));
}

/// CR 702.34d's rider on a plain graveyard grant — "Once during each of your
/// turns, you may cast an instant or sorcery spell from your graveyard. If a
/// spell cast this way would be put into your graveyard, exile it instead"
/// (Kess, Dissident Mage). The second graveyard cast that turn is refused.
#[test]
fn a_once_per_turn_graveyard_cast_can_exile_the_spell() {
    use crabomination::card::{SelectionRequirement as R, StaticAbility};
    use crabomination::effect::StaticEffect;
    let mut kess = bear_commander();
    kess.name = "Test Graveyard Caster";
    kess.static_abilities = vec![StaticAbility {
        description: "Once during each of your turns, cast an instant or sorcery from your graveyard.",
        effect: StaticEffect::GraveyardCastOncePerTurn {
            mv_at_most_counters: None,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            exile_after: true,
        },
    }];
    let mut g = commander_game();
    g.add_card_to_battlefield(0, kess);
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast from the graveyard");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3);
    assert!(g.exile.iter().any(|c| c.id == bolt), "exiled instead of the graveyard");
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: bolt2,
            target: Some(Target::Player(1)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(), "once per turn");
}

/// The Abyss / Magus of the Abyss — "destroy target nonartifact creature that
/// player controls of their choice": ONE creature, picked by the player whose
/// upkeep it is (it used to destroy every one of them). An unprompted seat
/// gives up its weakest.
#[test]
fn the_abyss_destroys_one_creature_of_that_players_choice() {
    let mut g = commander_game();
    g.add_card_to_battlefield(1, catalog::the_abyss());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let giant = g.add_card_to_battlefield(0, catalog::hill_giant());
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the weakest went");
    assert!(g.battlefield_find(giant).is_some(), "one creature, not all");
}

/// CR 122.1 + Mairsil, the Pretender — a card exiled from the graveyard can
/// take a cage counter, and a permanent with "has all activated abilities of
/// all cards you own in exile with cage counters on them" activates the
/// caged card's ability as its own.
#[test]
fn cr_122_1_a_caged_card_in_exile_lends_its_activated_abilities() {
    use crabomination::card::{CounterType, SelectionRequirement as R, StaticAbility};
    use crabomination::effect::{Effect, PlayerRef, Selector, StaticEffect, Value};
    use crabomination::game::effects::EffectContext;
    let mut pretender = bear_commander();
    pretender.name = "Test Pretender";
    pretender.static_abilities = vec![StaticAbility {
        description: "Has the activated abilities of caged cards you own.",
        effect: StaticEffect::HasActivatedAbilitiesOfOwnedExiledWithCounter {
            counter: CounterType::Cage,
        },
    }];
    let mut g = commander_game();
    let me = g.add_card_to_battlefield(0, pretender);
    g.clear_sickness(me);
    let sorcerer = g.add_card_to_graveyard(0, catalog::prodigal_sorcerer());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::Seq(vec![
            Effect::ExileChosenFromHandOrGraveyard { who: PlayerRef::You, filter: R::Creature },
            Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Cage, amount: Value::ONE },
        ]),
        &ctx,
    )
    .expect("cage");
    let caged = g.exile.iter().find(|c| c.id == sorcerer).expect("exiled");
    assert_eq!(caged.counter_count(CounterType::Cage), 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: me,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the caged Sorcerer's ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1);
}

/// CR 115.1c + CR 700.2 — a death trigger whose modes each take "target
/// opponent" fills every slot, one opponent per mode ("each mode must target a
/// different player"): at four seats all three opponents are hit once; in a
/// duel only the first mode finds an opponent and the rest do nothing. Both
/// death funnels used to push the trigger with its first slot only, and the
/// slot filler only ever offered the first opponent.
#[test]
fn cr_700_2_a_death_trigger_spreads_its_modes_over_distinct_opponents() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::shortcut::{on_dies, target_filtered};
    use crabomination::effect::{Effect, Value};
    let lose = |n| Effect::LoseLife {
        who: target_filtered(R::Player.and(R::ControlledByOpponent)),
        amount: Value::Const(n),
    };
    let mut body = bear_commander();
    body.name = "Test Spiteful Corpse";
    body.supertypes.clear();
    body.triggered_abilities = vec![on_dies(Effect::ChooseN { picks: vec![0, 1, 2], modes: vec![lose(3), lose(2), lose(1)] })];
    for seats in [4usize, 2] {
        let mut g = multi_player_game(seats);
        g.active_player_idx = 0;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        let corpse = g.add_card_to_battlefield(0, body.clone());
        let before: Vec<i32> = g.players.iter().map(|p| p.life).collect();
        let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(corpse)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt it");
        drain_stack(&mut g);
        let mut losses: Vec<i32> = (1..seats).map(|s| before[s] - g.players[s].life).collect();
        losses.sort();
        let want: Vec<i32> = if seats == 4 { vec![1, 2, 3] } else { vec![3] };
        assert_eq!(losses, want, "{seats} seats");
    }
}

// ── Primitives for Urza's Iron Alliance (BRC, Urza) ───────────────────────

/// CR 702.141 — "its encore cost is equal to its mana cost" (Wire Surgeons):
/// Master of Etherium's encore costs {2}{U}, so three colorless won't pay it
/// where the mana-value form (Sliver Gravemother's {X}) would.
#[test]
fn cr_702_141_encore_at_the_cards_own_mana_cost() {
    use crabomination::card::{SelectionRequirement as R, StaticAbility};
    use crabomination::effect::StaticEffect;
    let mut surgeon = bear_commander();
    surgeon.name = "Test Encore Granter";
    surgeon.static_abilities = vec![StaticAbility {
        description: "Artifact creature cards in your graveyard have encore at their mana cost.",
        effect: StaticEffect::GraveyardCardsHaveEncore {
            filter: R::Artifact.and(R::Creature),
            mana_cost: true,
        },
    }];
    let mut g = commander_game();
    g.add_card_to_battlefield(0, surgeon);
    let master = g.add_card_to_graveyard(0, catalog::master_of_etherium());
    let encore = |g: &mut GameState| {
        g.perform_action(GameAction::ActivateAbility {
            card_id: master,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
    };
    g.players[0].mana_pool.add_colorless(3);
    assert!(encore(&mut g).is_err(), "{{2}}{{U}}, not three colorless");
    g.players[0].mana_pool.add(Color::Blue, 1);
    encore(&mut g).expect("paid with blue");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == master));
}

/// CR 122.1b — keyword counters are counters: "remove a counter from another
/// creature you control" (Hexavus) can take a flying counter when that's the
/// only counter there.
#[test]
fn cr_122_1b_a_remove_any_counter_cost_takes_a_keyword_counter() {
    use crabomination::card::{CounterType, SelectionRequirement as R};
    use crabomination::effect::{ActivatedAbility, Effect, Selector, Value};
    let mut pump = bear_commander();
    pump.name = "Test Counter Eater";
    pump.activated_abilities = vec![ActivatedAbility {
        remove_counter_among_filter: Some((None, 1, R::Creature.and(R::OtherThanSource))),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ..Default::default()
    }];
    let mut g = commander_game();
    let eater = g.add_card_to_battlefield(0, pump);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().keyword_counters.add(Keyword::Flying, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: eater,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("the flying counter pays");
    drain_stack(&mut g);
    assert!(!has_kw(&g, bear, Keyword::Flying));
    assert_eq!(pt(&g, eater), (3, 3));
}

/// CR 615 — "As long as [condition], prevent all damage that would be dealt
/// to [this]" (Sanwell, Avenger Ace): a `WhileCondition` gate on the self
/// prevention static is honoured. It used to read `WhileYourTurn` alone.
#[test]
fn cr_615_a_conditional_prevent_all_damage_to_this() {
    use crabomination::card::{SelectionRequirement as R, StaticAbility};
    use crabomination::effect::{Predicate, Selector, StaticEffect};
    let mut shy = bear_commander();
    shy.name = "Test Shy Bear";
    shy.static_abilities = vec![StaticAbility {
        description: "As long as you control an artifact, prevent all damage to this.",
        effect: StaticEffect::WhileCondition {
            condition: Predicate::SelectorExists(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
            inner: Box::new(StaticEffect::PreventAllDamageToThis),
        },
    }];
    let mut g = commander_game();
    let bear = g.add_card_to_battlefield(0, shy);
    let bolt = |g: &mut GameState| {
        let b = g.add_card_to_hand(0, catalog::lightning_bolt());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.perform_action(GameAction::CastSpell {
            card_id: b,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(g);
    };
    g.add_card_to_battlefield(0, catalog::sol_ring());
    bolt(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "prevented with an artifact out");
    let ring = g.battlefield.iter().find(|c| c.definition.name == "Sol Ring").unwrap().id;
    g.remove_to_graveyard_with_triggers(ring);
    bolt(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "not without one");
}

// ── Primitives for Exquisite Invention (C18, Saheeli) ─────────────────────

/// CR 903.8 — "cast your commander from the command zone without paying its
/// mana cost" still owes the tax, an additional cost: after one earlier cast,
/// {2} is due, and without it the commander stays in the command zone.
#[test]
fn cr_903_8_a_free_commander_cast_still_pays_the_tax() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    let id = g.players[0].commanders[0];
    g.commander_cast_count.insert(id, 1);
    g.step = TurnStep::DeclareBlockers;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::CastCommanderWithoutPaying, &ctx).expect("resolve");
    drain_stack(&mut g);
    assert!(g.players[0].command.iter().any(|c| c.id == id), "no mana for the {{2}} tax");
    g.players[0].mana_pool.add_colorless(2);
    g.resolve_effect(&Effect::CastCommanderWithoutPaying, &ctx).expect("resolve");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "cast mid-combat, the printed {{G}} unpaid");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the tax took both");
}

/// CR 611.2b — "until the end of your next turn" outlives the opponent's turn
/// and ends in the cleanup of its controller's next one.
#[test]
fn cr_611_2b_until_the_end_of_your_next_turn() {
    use crabomination::effect::{Duration, Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    for s in 0..2 {
        for _ in 0..6 {
            g.add_card_to_library(s, catalog::island());
        }
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let ctx = EffectContext { targets: vec![Target::Permanent(bear)], ..ctx };
    g.resolve_effect(
        &Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(3),
            toughness: Value::Const(0),
            duration: Duration::UntilEndOfYourNextTurn,
        },
        &ctx,
    )
    .expect("pump");
    let pass_until = |g: &mut GameState, seat: usize| {
        for _ in 0..400 {
            if g.active_player_idx == seat && g.step == TurnStep::PreCombatMain {
                return;
            }
            let _ = g.perform_action(GameAction::PassPriority);
        }
        panic!("never reached seat {seat}'s main phase");
    };
    pass_until(&mut g, 1);
    assert_eq!(pt(&g, bear).0, 5, "through the opponent's turn");
    pass_until(&mut g, 0);
    assert_eq!(pt(&g, bear).0, 5, "and our next turn");
    pass_until(&mut g, 1);
    assert_eq!(pt(&g, bear).0, 2, "gone after it");
}

// ── Primitives for Mishra's Burnished Banner (BRC, Mishra) ────────────────

/// CR 602.2 + 118.3 — "whenever you activate an ability … if one or more
/// permanents were sacrificed to pay its cost" (Ashnod the Uncaring): the
/// activation event says whether a sacrifice paid for it, so Viscera Seer's
/// sacrifice triggers and Prodigal Sorcerer's tap does not.
#[test]
fn cr_602_2_an_activation_knows_it_was_paid_by_sacrifice() {
    use crabomination::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::effect::{Effect, Selector, Value};
    let watcher = CardDefinition {
        name: "Test Sacrifice Watcher",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWithSacrifice, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    };
    let mut g = commander_game();
    g.add_card_to_battlefield(0, watcher);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let seer = g.add_card_to_battlefield(0, catalog::viscera_seer());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sorcerer = g.add_card_to_battlefield(0, catalog::prodigal_sorcerer());
    g.clear_sickness(sorcerer);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sorcerer,
        ability_index: 0,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "a tap is not a sacrifice");
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice the Bears");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "the sacrifice triggered");
}

/// CR 707.9b — "enter as a copy of any creature on the battlefield, except
/// it's an artifact and it isn't a creature" (Machine God's Effigy): the
/// exception is part of the copiable values, so the copy is a noncreature
/// artifact named for what it copied.
#[test]
fn cr_707_9b_a_copy_except_it_isnt_a_creature() {
    use crabomination::card::{EntersAsCopy, SelectionRequirement as R};
    let copier = CardDefinition {
        name: "Test Noncreature Copier",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_card_types: vec![CardType::Artifact],
            not_a_creature: true,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut g = commander_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, copier);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("a 0-toughness noncreature survives");
    assert_eq!(c.definition.name, "Grizzly Bears");
    assert!(c.definition.card_types.contains(&CardType::Artifact));
    assert!(!c.definition.card_types.contains(&CardType::Creature));
}

// ── Primitives for Tinker Time (MOC, Gimbal) ──────────────────────────────

/// CR 702.126a — "nonartifact spells you cast have improvise" (Inspiring
/// Statuary): artifacts tap for the generic part of a granted spell, and an
/// artifact spell gets no help.
#[test]
fn cr_702_126a_a_granted_improvise_taps_artifacts() {
    use crabomination::card::{SelectionRequirement as R, StaticAbility, StaticEffect};
    let statuary = CardDefinition {
        name: "Test Statuary",
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Nonartifact spells you cast have improvise.",
            effect: StaticEffect::GrantImproviseToSpells { filter: R::Not(Box::new(R::Artifact)) },
        }],
        ..Default::default()
    };
    let mut g = commander_game();
    let helper = g.add_card_to_battlefield(0, statuary);
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    // Grizzly Bears ({1}{G}): the Statuary pays the {1}.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    assert!(g.helper_tap_candidates(0, bear).contains(&helper));
    g.perform_action(GameAction::CastSpellConvoke {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        convoke_creatures: vec![helper],
    })
    .expect("improvised");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
    assert!(g.battlefield_find(helper).unwrap().tapped);
    // An artifact spell isn't granted it.
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    assert!(g.helper_tap_candidates(0, stone).is_empty(), "{thopter:?} can't help an artifact");
}

/// CR 107.3 + 406 — "exile the top card as many times as you choose; if the
/// total mana value is 13 or less, [then]" (Dance with Calamity): the exile
/// stops at the engine's stop point, and `then` runs only under the limit.
#[test]
fn exile_top_pushing_luck_runs_its_body_under_the_limit() {
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let gain = || Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE });
    let mut g = commander_game();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let life = g.players[0].life;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::ExileTopPushingLuck { stop_at: 3, limit: 4, then: gain() }, &ctx)
        .expect("resolve");
    assert_eq!(g.players[0].library.len(), 1, "two Bears reach the stop point");
    assert_eq!(g.players[0].life, life + 1, "four is within the limit");
    g.resolve_effect(&Effect::ExileTopPushingLuck { stop_at: 3, limit: 1, then: gain() }, &ctx)
        .expect("resolve");
    assert_eq!(g.players[0].life, life + 1, "over the limit");
}

/// CR 107.3 — a "cards exiled this way" filter that names X reads the
/// resolution's X (Rashmi and Ragavan's `WithX` "mana value less than"); it
/// used to see no X and match nothing.
#[test]
fn cr_107_3_an_exiled_this_way_filter_reads_x() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let exile_and_count = |x: i32| {
        Effect::Seq(vec![
            Effect::ExileTopOfLibrary {
                who: Selector::You,
                amount: Value::ONE,
                link_to_source: false,
                face_down: false,
            },
            Effect::WithX {
                x: Value::Const(x),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(Selector::ExiledThisResolution {
                        filter: R::ManaValueAtMostXFromCost,
                    })),
                }),
            },
        ])
    };
    let life = g.players[0].life;
    g.resolve_effect(&exile_and_count(2), &ctx).expect("resolve");
    assert_eq!(g.players[0].life, life + 1, "mana value 2 is at most X = 2");
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.resolve_effect(&exile_and_count(1), &ctx).expect("resolve");
    assert_eq!(g.players[0].life, life + 1, "but not at most X = 1");
}

// ── Primitives for Legends' Legacy (DMC, Dihada) ──────────────────────────

/// CR 602.2 + 119.4 — "if life was paid to activate it" (Verrak, Warped
/// Sengir): the activation event carries the life paid, and only a life-cost
/// activation triggers.
#[test]
fn cr_602_2_an_activation_knows_the_life_paid_for_it() {
    use crabomination::card::{ActivatedAbility, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crabomination::effect::{Effect, Selector, Value};
    let watcher = CardDefinition {
        name: "Test Life Watcher",
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWithLifePaid, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..Default::default()
    };
    let pay = |life: u32| CardDefinition {
        name: "Test Life Payer",
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            life_cost: life,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    };
    let mut g = commander_game();
    g.add_card_to_battlefield(0, watcher);
    let three = g.add_card_to_battlefield(0, pay(3));
    let free = g.add_card_to_battlefield(0, pay(0));
    let life = g.players[0].life;
    for id in [free, three] {
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].life, life, "paid 3, the watcher gave 3 back; the free one gave nothing");
}

/// CR 614.10 — "if a player would begin an extra turn, that player skips
/// that turn instead" (Gerrard's Hourglass Pendant) binds its own controller,
/// unlike Trouble in Pairs' opponents-only clause.
#[test]
fn cr_614_10_a_symmetric_extra_turn_skip_binds_its_controller() {
    use crabomination::card::{StaticAbility, StaticEffect};
    let pendant = |effect| CardDefinition {
        name: "Test Pendant",
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility { description: "", effect }],
        ..Default::default()
    };
    let mut g = commander_game();
    let id = g.add_card_to_battlefield(0, pendant(StaticEffect::OpponentsSkipExtraTurns));
    assert!(!g.extra_turn_denied_for(0) && g.extra_turn_denied_for(1));
    g.remove_from_battlefield_to_exile(id);
    g.add_card_to_battlefield(0, pendant(StaticEffect::PlayersSkipExtraTurns));
    assert!(g.extra_turn_denied_for(0) && g.extra_turn_denied_for(1));
}

/// CR 406 — "note the mana value of each card as it's put into exile" (Bell
/// Borca): the turn's greatest exiled mana value is noted from the library
/// and from the battlefield, and forgotten at the next turn.
#[test]
fn cr_406_the_greatest_exiled_mana_value_is_noted_this_turn() {
    use crabomination::effect::{Effect, Selector, Value};
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::island());
        g.add_card_to_library(1, catalog::island());
    }
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let note = |g: &mut GameState| {
        g.resolve_effect(
            &Effect::GainLife { who: Selector::You, amount: Value::GreatestManaValueExiledThisTurn },
            &ctx,
        )
        .expect("resolve");
    };
    g.resolve_effect(
        &Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::ONE, link_to_source: false, face_down: false },
        &ctx,
    )
    .expect("exile the Bears");
    let life = g.players[0].life;
    note(&mut g);
    assert_eq!(g.players[0].life, life + 2);
    let ring = g.add_card_to_battlefield(0, catalog::solemn_simulacrum());
    g.remove_from_battlefield_to_exile(ring);
    note(&mut g);
    assert_eq!(g.players[0].life, life + 2 + 4, "Solemn's four from the battlefield");
    for _ in 0..200 {
        if g.active_player_idx == 1 {
            break;
        }
        let _ = g.perform_action(GameAction::PassPriority);
    }
    let life = g.players[0].life;
    note(&mut g);
    assert_eq!(g.players[0].life, life, "a new turn forgets");
}

// ── Primitives for Planeswalker Party (CMM, Commodore Guff) ───────────────

/// A 5-loyalty test planeswalker whose +1 gains a life and whose type is `t`.
fn life_walker(t: crabomination::card::PlaneswalkerSubtype) -> CardDefinition {
    use crabomination::card::LoyaltyAbility;
    use crabomination::effect::{Effect, Selector, Value};
    CardDefinition {
        name: "Test Life Walker",
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![t], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: 1,
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn plus_one(g: &mut GameState, id: CardId) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility { card_id: id, ability_index: 0, target: None, x_value: None })
        .map(|_| ())
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// CR 606.3 — "you may activate the loyalty abilities of planeswalkers you
/// control twice each turn" (Oath of Teferi): a second activation is legal, a
/// third is not.
#[test]
fn cr_606_3_loyalty_abilities_twice_each_turn() {
    use crabomination::card::{PlaneswalkerSubtype, StaticAbility, StaticEffect};
    let mut g = commander_game();
    let pw = g.add_card_to_battlefield(0, life_walker(PlaneswalkerSubtype::Jace));
    plus_one(&mut g, pw).expect("first");
    assert!(plus_one(&mut g, pw).is_err(), "once without the Oath");
    g.add_card_to_battlefield(
        0,
        CardDefinition {
            name: "Test Oath",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility { description: "", effect: StaticEffect::LoyaltyAbilitiesTwiceEachTurn }],
            ..Default::default()
        },
    );
    plus_one(&mut g, pw).expect("second");
    assert!(plus_one(&mut g, pw).is_err(), "not a third");
}

/// CR 707.10 — "copy the next loyalty ability you activate this turn"
/// (Jaya's Phoenix): the copy resolves too, and the grant is spent.
#[test]
fn cr_707_10_the_next_loyalty_ability_is_copied_once() {
    use crabomination::card::PlaneswalkerSubtype;
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    let a = g.add_card_to_battlefield(0, life_walker(PlaneswalkerSubtype::Jace));
    let b = g.add_card_to_battlefield(0, life_walker(PlaneswalkerSubtype::Chandra));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::CopyNextLoyaltyAbility { copies: 1 }, &ctx).expect("grant");
    let life = g.players[0].life;
    plus_one(&mut g, a).expect("copied");
    assert_eq!(g.players[0].life, life + 2);
    plus_one(&mut g, b).expect("not copied");
    assert_eq!(g.players[0].life, life + 3, "the grant was spent");
}

/// CR 707.10 — Leori's type-scoped copies: every ability of a planeswalker
/// of the chosen type this turn, and none of another type.
#[test]
fn cr_707_10_copies_of_one_planeswalker_type() {
    use crabomination::card::PlaneswalkerSubtype;
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    let jace = g.add_card_to_battlefield(0, life_walker(PlaneswalkerSubtype::Jace));
    let jace2 = g.add_card_to_battlefield(0, life_walker(PlaneswalkerSubtype::Jace));
    let chandra = g.add_card_to_battlefield(0, life_walker(PlaneswalkerSubtype::Chandra));
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::CopyLoyaltyAbilitiesOfChosenTypeThisTurn, &ctx).expect("grant");
    let life = g.players[0].life;
    plus_one(&mut g, jace).expect("Jace");
    plus_one(&mut g, jace2).expect("Jace again");
    plus_one(&mut g, chandra).expect("Chandra");
    assert_eq!(g.players[0].life, life + 2 + 2 + 1, "both Jaces copied (the most common type)");
}

/// CR 508.1a — "until your next turn, each player may attack only the
/// nearest opponent in the last chosen direction" (Teyo's −2): with no
/// Barrier on the battlefield, seat 1 is the only legal defender for seat 0.
#[test]
fn cr_508_1a_a_temporary_attack_direction() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = game_with_format(Format::Commander, 4);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::ChooseAttackDirectionUntilYourNextTurn, &ctx).expect("left");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    let at = |seat| GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(seat) }]);
    assert!(!g.would_accept(at(2)), "seat 2 is not the nearest to the left");
    assert!(g.would_accept(at(1)));
}

/// CR 508.1g — "creatures can't attack planeswalkers you control unless
/// their controller pays {1} for each" (Onakke Oathkeeper): attacking the
/// player is free, the planeswalker costs {1}.
#[test]
fn cr_508_1g_a_tax_on_attacking_planeswalkers_only() {
    use crabomination::card::{PlaneswalkerSubtype, StaticAbility, StaticEffect};
    use crabomination::effect::Value;
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = commander_game();
    g.add_card_to_battlefield(
        1,
        CardDefinition {
            name: "Test Oathkeeper",
            card_types: vec![CardType::Enchantment],
            static_abilities: vec![StaticAbility {
                description: "",
                effect: StaticEffect::AttackTaxOnYourPlaneswalkers { amount: Value::ONE },
            }],
            ..Default::default()
        },
    );
    let pw = g.add_card_to_battlefield(1, life_walker(PlaneswalkerSubtype::Jace));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    let at = |t| GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: t }]);
    assert!(!g.would_accept(at(AttackTarget::Planeswalker(pw))), "no {{1}} in pool");
    assert!(g.would_accept(at(AttackTarget::Player(1))));
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.would_accept(at(AttackTarget::Planeswalker(pw))));
}

/// CR 701.24 — Guff Rewrites History's shuffle-in: the permanent goes into
/// its owner's library and its controller casts the top nonland card free.
#[test]
fn a_permanent_shuffled_in_is_replaced_by_a_free_cast_off_the_top() {
    use crabomination::effect::{Effect, Selector};
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    let ctx = EffectContext { targets: vec![Target::Permanent(ring)], ..ctx };
    g.resolve_effect(&Effect::ShuffleInThenCastFromTopFree { what: Selector::Target(0) }, &ctx).expect("resolve");
    drain_stack(&mut g);
    // The Ring is its library's only card, so it comes straight back.
    assert!(g.battlefield_find(ring).is_some_and(|c| c.controller == 1), "shuffled in and recast");
}

/// CR 106.7 — Gond Gate's "any color that a Gate you control could produce":
/// the Gates' own colors, nothing else — its ability makes {W} or {U} beside
/// an Azorius Guildgate, never {G} from a Forest.
#[test]
fn cr_106_7_colors_a_gate_could_produce() {
    use crabomination::mana::Color;
    let mut g = commander_game();
    g.add_card_to_battlefield(0, catalog::azorius_guildgate());
    let gond = g.add_card_to_battlefield(0, catalog::gond_gate());
    g.add_card_to_battlefield(0, catalog::forest());
    assert_eq!(g.colors_gates_could_produce(0), vec![Color::White, Color::Blue]);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: gond,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap");
    drain_stack(&mut g);
    let pool = &g.players[0].mana_pool;
    assert_eq!(pool.amount(Color::White) + pool.amount(Color::Blue), 1);
    assert_eq!(pool.amount(Color::Green), 0);
}

// ── Primitives for Eldrazi Incursion (M3C, Ulalek) ─────────────────────────

/// CR 105.2 — Selective Obliteration: a permanent survives only if it's
/// colorless or exactly its controller's chosen color (each seat's most
/// common one); a multicolored permanent never survives.
#[test]
fn each_player_keeps_one_color() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    let mut g = game_with_format(Format::Commander, 3);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt_guy = g.add_card_to_battlefield(1, catalog::goblin_guide());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::EachPlayerChoosesColorExileOthers, &ctx).expect("resolve");
    assert!(g.battlefield_find(bear).is_some() && g.battlefield_find(bear2).is_some(), "green, the chosen color");
    assert!(g.battlefield_find(bolt_guy).is_none(), "red isn't");
    assert!(g.battlefield_find(ring).is_some(), "colorless");
}

/// CR 707.10 — "copy all spells you control, then copy all other activated
/// and triggered abilities you control" (Ulalek): an opponent's spell on the
/// stack is left alone.
#[test]
fn cr_707_10_copy_every_spell_and_ability_you_control() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    use crabomination::game::types::StackItem;
    let mut g = game_with_format(Format::Commander, 3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    for (seat, card) in [(0, catalog::lightning_bolt()), (1, catalog::lightning_bolt())] {
        let id = g.add_card_to_hand(seat, card);
        g.players[seat].mana_pool.add(crabomination::mana::Color::Red, 1);
        g.priority.player_with_priority = seat;
        g.perform_action(GameAction::CastSpell {
            card_id: id,
            target: Some(Target::Player(2)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
    }
    let spells = |g: &GameState, seat| {
        g.stack.iter().filter(|s| matches!(s, StackItem::Spell { caster, .. } if *caster == seat)).count()
    };
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::CopyAllSpellsAndAbilitiesYouControl, &ctx).expect("copy");
    assert_eq!((spells(&g, 0), spells(&g, 1)), (2, 1));
}

/// CR 707.9 — a copy with exceptions: Benthic Anomaly's token copies one
/// chosen creature but takes the chosen creatures' summed power and
/// toughness and is colorless.
#[test]
fn cr_707_9_one_copy_with_summed_stats() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    let mut g = game_with_format(Format::Commander, 3);
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::goblin_guide());
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::CopyOnePerOpponentWithTotalStats, &ctx).expect("copy");
    let tok = g.battlefield.iter().find(|c| c.controller == 0 && c.is_token).expect("token").id;
    let cp = g.computed_permanent(tok).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "2/2 + 2/2");
    assert!(cp.colors.is_empty());
}

/// CR 107.4e — a colorless hybrid pip ({C/W}) is paid with one colorless
/// mana or one mana of its color, has mana value 1, and counts its color
/// toward color identity (CR 903.4).
#[test]
fn cr_107_4e_colorless_hybrid_pips() {
    use crabomination::mana::{colorless_hybrid, Color, ManaCost, ManaPool};
    let cost = ManaCost { symbols: vec![colorless_hybrid(Color::White), colorless_hybrid(Color::Blue)] };
    assert_eq!(cost.cmc(), 2);
    let mut pool = ManaPool::default();
    pool.add_colorless(1);
    pool.add(Color::Blue, 1);
    assert!(pool.clone().pay(&cost).is_ok(), "{{C}} for one, {{U}} for the other");
    let mut red = ManaPool::default();
    red.add(Color::Red, 2);
    assert!(red.pay(&cost).is_err(), "red pays neither");
    let ulalek = catalog::ulalek_fused_atrocity();
    assert_eq!(crabomination::color_identity::color_identity(&ulalek).len(), 5);
}

// ── Primitives for Eldrazi Unbound (CMM, Zhulodok) ─────────────────────────

/// CR 118.9 — "Once each turn, you may pay {0} rather than pay the mana
/// cost for a colorless spell you cast from your hand" (Darksteel Monolith):
/// the first colorless spell is free, the second isn't, and a colored one
/// never is.
#[test]
fn cr_118_9_a_zero_alternative_cost_once_each_turn() {
    let mut g = commander_game();
    g.add_card_to_battlefield(0, catalog::darksteel_monolith());
    let alt = |g: &mut GameState, id| {
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::CastSpellAlternative {
            card_id: id,
            pitch_card: None,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    assert!(alt(&mut g, bear).is_err(), "a green spell");
    let a = g.add_card_to_hand(0, catalog::sol_ring());
    let b = g.add_card_to_hand(0, catalog::arcane_signet());
    alt(&mut g, a).expect("the first colorless spell is free");
    drain_stack(&mut g);
    assert!(alt(&mut g, b).is_err(), "once each turn");
}

/// CR 115.10 — "target spell or ability that targets a permanent you
/// control" (Not of This World): a Bolt at your creature is legal, a Bolt at
/// a player isn't; and the {7} discount needs a power-7 creature targeted.
#[test]
fn a_spell_that_targets_a_permanent_you_control() {
    use crabomination::mana::Color;
    let mut g = game_with_format(Format::Commander, 3);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt the bear");
    let not = g.add_card_to_hand(0, catalog::not_of_this_world());
    g.priority.player_with_priority = 0;
    let cast = |g: &mut GameState| {
        g.perform_action(GameAction::CastSpell {
            card_id: not,
            target: Some(Target::Permanent(bolt)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
    };
    assert!(cast(&mut g).is_err(), "{{7}} with no mana — a 2/2 earns no discount");
    g.players[0].mana_pool.add_colorless(7);
    cast(&mut g).expect("the Bolt targets our Bears");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "countered");
}

/// CR 601.2c — a "for each opponent, target … that player controls" spell
/// (Desecrate Reality) whose targets all sit under one opponent: the bot
/// proposes one target, not two of the same controller the cast would
/// reject. Regression: the bot never cast Desecrate Reality in 1,000 pods.
#[test]
fn bot_picks_at_most_one_target_per_opponent() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = game_with_format(Format::Commander, 3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..8 {
        g.add_card_to_battlefield(0, catalog::wastes());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::desecrate_reality());
    let action = HeuristicBot::new().next_action(&g, 0).expect("the bot acts");
    assert!(
        matches!(&action, GameAction::CastSpell { card_id, additional_targets, .. }
            if *card_id == d && additional_targets.is_empty()),
        "expected a one-target Desecrate Reality, got {action:?}",
    );
}

/// CR 601.2c — on a prompting seat (every pod seat is `wants_ui`) an "up to
/// N targets" spell the bot under-fills suspends to ask for the next slot,
/// and the bot's dry run reads that as a rejection. Regression: Desecrate
/// Reality was never cast in 1,000 pods. The bot now names one target per
/// opponent up front, the cast completes, and the engine's extra-slot prompt
/// never offers a second target under a controller already named.
#[test]
fn a_prompting_bot_seat_fills_per_opponent_slots() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = game_with_format(Format::Commander, 4);
    for s in 0..4 {
        g.players[s].wants_ui = true;
    }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..8 {
        g.add_card_to_battlefield(0, catalog::wastes());
    }
    for s in 1..4 {
        g.add_card_to_battlefield(s, catalog::grizzly_bears());
    }
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d = g.add_card_to_hand(0, catalog::desecrate_reality());
    let action = HeuristicBot::new().next_action(&g, 0).expect("the bot acts");
    let GameAction::CastSpell { card_id, target, additional_targets, .. } = &action else {
        panic!("expected a cast, got {action:?}");
    };
    assert_eq!(*card_id, d);
    assert_eq!(additional_targets.len(), 2, "one target for each of three opponents");
    let controllers: Vec<usize> = target
        .iter()
        .chain(additional_targets.iter())
        .filter_map(|t| match t {
            Target::Permanent(id) => g.battlefield_find(*id).map(|c| c.controller),
            _ => None,
        })
        .collect();
    assert_eq!(controllers.len(), 3);
    assert!(controllers.iter().all(|c| controllers.iter().filter(|x| *x == c).count() == 1));
    assert!(g.would_accept(action));
}

// ── Primitives for Subjective Reality (C18, Aminatou) ─────────────────────

/// CR 800.4 / 110.2 — Aminatou's −6: each seat takes the nonland
/// permanents of the next seat in the chosen direction (left = the next
/// seat), except the source; lands stay put.
#[test]
fn nonland_permanents_rotate_one_seat() {
    use crabomination::effect::Effect;
    use crabomination::game::effects::EffectContext;
    let mut g = game_with_format(Format::Commander, 3);
    let src = g.add_card_to_battlefield(0, catalog::sol_ring());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let ctx = EffectContext { source: Some(src), ..EffectContext::for_spell(0, None, 0, 0) };
    g.resolve_effect(&Effect::RotateNonlandPermanents, &ctx).expect("rotate");
    let ctrl = |g: &GameState, id| g.battlefield_find(id).unwrap().controller;
    assert_eq!((ctrl(&g, a), ctrl(&g, b), ctrl(&g, c)), (2, 0, 1), "seat s takes seat s+1's");
    assert_eq!(ctrl(&g, land), 1);
    assert_eq!(ctrl(&g, src), 0, "not the source");
}

/// CR 120.3 — Sower of Discord's pair: damage to one chosen player makes the
/// other lose that much; anyone else is untouched.
#[test]
fn damage_to_one_chosen_player_drains_the_other() {
    let mut g = game_with_format(Format::Commander, 4);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[1].life = 30;
    g.players[2].life = 31;
    let sower = g.add_card_to_hand(0, catalog::sower_of_discord());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: sower, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    drain_stack(&mut g);
    let (l1, l2, l3) = (g.players[1].life, g.players[2].life, g.players[3].life);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None })
        .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 3);
    assert_eq!(g.players[2].life, l2 - 3, "the other of the two least-life opponents");
    assert_eq!(g.players[3].life, l3);
}

/// Answers a `ChooseOption` with the option carrying this label.
struct PickLabel(&'static str);
impl crabomination::decision::Decider for PickLabel {
    fn decide(&mut self, d: &crabomination::decision::Decision) -> crabomination::decision::DecisionAnswer {
        match d {
            crabomination::decision::Decision::ChooseOption { options, .. } => {
                let i = options.iter().position(|o| o == self.0).expect("the label is offered");
                crabomination::decision::DecisionAnswer::Amount(i as u32)
            }
            other => crabomination::decision::AutoDecider.decide(other),
        }
    }
    fn kind(&self) -> crabomination::decision::DeciderKind {
        crabomination::decision::DeciderKind::Scripted { answers: Vec::new(), asked: Vec::new() }
    }
}

/// CR 614.12 — Sower of Discord's two players are its controller's choice,
/// any two (itself included): here seats 1 and 3, not the least-life pair.
#[test]
fn sower_of_discord_pair_is_the_controllers_choice() {
    let mut g = game_with_format(Format::Commander, 4);
    g.decider = Box::new(PickLabel("Player 2 and Player 4"));
    let sower = g.move_card_to_battlefield_for_test(0, catalog::sower_of_discord());
    assert!(g.chosen_player_pairs.iter().any(|&(s, a, b)| s == sower && (a, b) == (1, 3)));
}

/// CR 601.2 — "for each nonland card type, you may cast a spell of that type
/// from among them without paying its mana cost": one card per type, and a
/// card of two types spends only one.
#[test]
fn one_free_cast_per_card_type() {
    use crabomination::effect::{Effect, Selector};
    use crabomination::game::effects::EffectContext;
    let mut g = commander_game();
    let ids: Vec<CardId> = [catalog::grizzly_bears(), catalog::hill_giant(), catalog::lightning_bolt(), catalog::sol_ring()]
        .into_iter()
        .map(|d| g.add_card_to_exile(0, d))
        .collect();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(&Effect::GrantFreeCastOnePerCardType { what: Selector::ExactObjects(ids.clone()) }, &ctx)
        .expect("grant");
    let free: Vec<bool> = ids.iter().map(|id| g.exile.iter().find(|c| c.id == *id).unwrap().may_play_until.is_some()).collect();
    assert_eq!(free, vec![false, true, true, true], "the bigger creature, the instant, the artifact");
}

/// Regression: a base-P/T-setting Equipment (Belt of Giant Strength, 10/10)
/// on one of two 11/11s *lowered* its host, so the bot's "equip the biggest"
/// moved it to the other every tick — 11,365 equips and an action-capped
/// 12-seat pod. Moving an Equipment now needs a strictly stronger new host.
#[test]
fn bot_does_not_shuffle_equipment_between_equal_hosts() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = commander_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for _ in 0..10 {
        g.add_card_to_battlefield(0, catalog::wastes());
    }
    let a = g.add_card_to_battlefield(0, catalog::it_that_betrays());
    g.add_card_to_battlefield(0, catalog::it_that_betrays());
    let belt = g.add_card_to_battlefield(0, catalog::belt_of_giant_strength());
    g.battlefield_find_mut(belt).unwrap().attached_to = Some(a);
    let action = HeuristicBot::new().next_action(&g, 0);
    assert!(!matches!(action, Some(GameAction::Equip { .. })), "got {action:?}");
}
