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
