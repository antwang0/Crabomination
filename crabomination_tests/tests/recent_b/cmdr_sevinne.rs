//! Commander: the Mystic Intellect precon (C19, Sevinne, `decks::cmdr_sevinne`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_with(g: &mut GameState, action: GameAction) -> Result<(), String> {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(action).map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn cast(id: CardId, target: Option<Target>) -> GameAction {
    GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode: None, x_value: None }
}

fn flashback(id: CardId, target: Option<Target>) -> GameAction {
    GameAction::CastFlashback { card_id: id, target, additional_targets: vec![], mode: None, x_value: None }
}

fn in_exile(g: &GameState, id: CardId) -> bool {
    g.exile.iter().any(|c| c.id == id)
}

fn tokens_named(g: &GameState, seat: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == seat && c.is_token && c.definition.name == name).count()
}

/// Sevinne copies the first instant or sorcery cast from your graveyard each
/// turn, and only the first: two flashback Rolling Temblors on a
/// 6-toughness creature deal 2 + 2 (copied) + 2 (not copied).
#[test]
fn sevinne_copies_only_the_first_graveyard_instant_or_sorcery() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::sevinne_the_chronoclasm());
    let wall = g.add_card_to_battlefield(1, catalog::pramikon_sky_rampart());
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let t1 = g.add_card_to_graveyard(0, catalog::rolling_temblor());
    let t2 = g.add_card_to_graveyard(0, catalog::rolling_temblor());
    cast_with(&mut g, flashback(t1, None)).expect("flashback Temblor");
    assert_eq!(g.battlefield_find(big).map(|c| c.damage), Some(4), "the first graveyard cast is copied");
    assert!(g.battlefield_find(wall).is_some(), "Pramikon flies over the Temblor");
    cast_with(&mut g, flashback(t2, None)).expect("flashback Temblor again");
    assert!(g.battlefield_find(big).is_none(), "the second cast (not copied) finishes the Dreadmaw");
    assert!(in_exile(&g, t1) && in_exile(&g, t2), "CR 702.34a — flashback exiles");
}

/// Sevinne's printed "prevent all damage that would be dealt to Sevinne".
#[test]
fn sevinne_prevents_damage_to_itself() {
    let mut g = main_phase();
    let sevinne = g.add_card_to_battlefield(0, catalog::sevinne_the_chronoclasm());
    let temblor = g.add_card_to_hand(0, catalog::rolling_temblor());
    cast_with(&mut g, cast(temblor, None)).expect("Temblor");
    assert_eq!(g.battlefield_find(sevinne).map(|c| c.damage), Some(0));
}

/// The graveyard-cast payoffs (Burning Vengeance, Secrets of the Dead,
/// Thalia's Geistcaller) fire on a flashback cast and not on a cast from hand.
#[test]
fn graveyard_cast_payoffs_fire_only_for_graveyard_casts() {
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    g.add_card_to_battlefield(0, catalog::burning_vengeance());
    g.add_card_to_battlefield(0, catalog::secrets_of_the_dead());
    g.add_card_to_battlefield(0, catalog::thalias_geistcaller());
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let from_hand = g.add_card_to_hand(0, catalog::purify_the_grave());
    let hand0 = g.players[0].hand.len();
    let life1 = g.players[1].life;
    cast_with(&mut g, cast(from_hand, Some(Target::Permanent(dead)))).expect("Purify from hand");
    assert_eq!(g.players[0].hand.len(), hand0 - 1, "no draw off a hand cast");
    assert_eq!(tokens_named(&g, 0, "Spirit"), 0);
    let dead2 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    cast_with(&mut g, flashback(from_hand, Some(Target::Permanent(dead2)))).expect("Purify by flashback");
    assert_eq!(g.players[0].hand.len(), hand0, "Secrets of the Dead drew");
    assert_eq!(tokens_named(&g, 0, "Spirit"), 1, "Thalia's Geistcaller made a Spirit");
    assert_eq!(g.players[1].life, life1 - 2, "Burning Vengeance hit the opponent");
}

/// CR 724.2 — Mandate of Peace ends the combat phase: the attackers leave
/// combat without dealing damage, the game goes to the postcombat main, the
/// spell is exiled (724.2b), and opponents can't cast spells this turn.
#[test]
fn cr_724_2_mandate_of_peace_ends_the_combat_phase() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(0) }]))
        .expect("attack");
    let mandate = g.add_card_to_hand(0, catalog::mandate_of_peace());
    flood(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(cast(mandate, None)).expect("Mandate during combat");
    drain_stack(&mut g);
    assert_eq!(g.step, TurnStep::PostCombatMain);
    assert!(g.attacking.is_empty(), "CR 724.2d — everything left combat");
    assert_eq!(g.players[0].life, 20, "no combat damage");
    assert!(in_exile(&g, mandate), "CR 724.2b — the resolving spell is exiled");
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(g.perform_action(cast(bolt, Some(Target::Player(0)))).is_err(), "opponents are silenced");
}

/// "Cast this spell only during combat."
#[test]
fn mandate_of_peace_cannot_be_cast_outside_combat() {
    let mut g = main_phase();
    let mandate = g.add_card_to_hand(0, catalog::mandate_of_peace());
    assert!(cast_with(&mut g, cast(mandate, None)).is_err());
}

/// Elsha casts a noncreature spell from the top of the library at instant
/// speed; a sorcery in hand keeps sorcery timing.
#[test]
fn elsha_casts_the_top_card_with_flash() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::elsha_of_the_infinite());
    let top = g.add_card_to_library(0, catalog::rolling_temblor());
    let in_hand = g.add_card_to_hand(0, catalog::rolling_temblor());
    g.step = TurnStep::BeginCombat;
    assert!(cast_with(&mut g, cast(in_hand, None)).is_err(), "the hand copy is still a sorcery");
    cast_with(&mut g, cast(top, None)).expect("top of library, as though it had flash");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == top));
}

/// Elsha doesn't cast creature spells from the library.
#[test]
fn elsha_does_not_cast_creatures_from_the_top() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::elsha_of_the_infinite());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    assert!(cast_with(&mut g, cast(top, None)).is_err());
}

/// Wall of Stolen Identity copies a creature as a Wall with defender, and the
/// copied creature is tapped and stays tapped through its untap step.
#[test]
fn wall_of_stolen_identity_copies_and_locks() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let wall = g.add_card_to_hand(0, catalog::wall_of_stolen_identity());
    cast_with(&mut g, cast(wall, None)).expect("Wall");
    let copy = g.computed_permanent(wall).expect("the Wall survived as a copy");
    assert_eq!(copy.power, 6);
    assert!(copy.keywords().contains(&Keyword::Defender));
    assert!(g.battlefield_find(wurm).is_some_and(|c| c.tapped));
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(wurm).is_some_and(|c| c.tapped), "it doesn't untap while the Wall remains");
}

/// Gerrard returns the artifacts and creatures that died this turn, and is
/// exiled itself.
#[test]
fn gerrard_returns_this_turns_dead() {
    let mut g = main_phase();
    let gerrard = g.add_card_to_battlefield(0, catalog::gerrard_weatherlight_hero());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let old = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let temblor = g.add_card_to_hand(0, catalog::rolling_temblor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    cast_with(&mut g, cast(temblor, None)).expect("Temblor kills the bear");
    assert!(g.battlefield_find(bear).is_none());
    cast_with(&mut g, cast(bolt, Some(Target::Permanent(gerrard)))).expect("Bolt Gerrard");
    assert!(g.battlefield_find(bear).is_some(), "the bear came back");
    assert!(g.battlefield_find(old).is_none(), "a card that was already there stays");
    assert!(in_exile(&g, gerrard), "Gerrard exiles itself");
}

/// Backdraft Hellkite's attack gives a graveyard sorcery flashback at its
/// mana cost; it is exiled after the cast.
#[test]
fn backdraft_hellkite_grants_flashback() {
    let mut g = main_phase();
    let hellkite = g.add_card_to_battlefield(0, catalog::backdraft_hellkite());
    g.clear_sickness(hellkite);
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let fireball = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: hellkite, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    cast_with(
        &mut g,
        GameAction::CastFromZoneWithoutPaying {
            card_id: fireball,
            target: Some(Target::Permanent(bears)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    )
    .expect("cast the Bolt from the graveyard");
    assert!(g.battlefield_find(bears).is_none());
    assert!(in_exile(&g, fireball));
}

/// Runic Repetition returns an exiled card with flashback, not one without.
#[test]
fn runic_repetition_returns_an_exiled_flashback_card() {
    let mut g = main_phase();
    let temblor = g.add_card_to_exile(0, catalog::rolling_temblor());
    let bolt = g.add_card_to_exile(0, catalog::lightning_bolt());
    let rr = g.add_card_to_hand(0, catalog::runic_repetition());
    assert!(cast_with(&mut g, cast(rr, Some(Target::Permanent(bolt)))).is_err(), "no flashback");
    cast_with(&mut g, cast(rr, Some(Target::Permanent(temblor)))).expect("Runic Repetition");
    assert!(g.players[0].hand.iter().any(|c| c.id == temblor));
}

/// Mass Diminish sets the target player's creatures to base 1/1.
#[test]
fn mass_diminish_shrinks_one_players_creatures() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm());
    let mine = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let md = g.add_card_to_hand(0, catalog::mass_diminish());
    cast_with(&mut g, cast(md, Some(Target::Player(1)))).expect("Mass Diminish");
    assert_eq!(g.computed_permanent(wurm).map(|c| (c.power, c.toughness)), Some((1, 1)));
    assert_eq!(g.computed_permanent(mine).map(|c| c.power), Some(6));
}

/// Dockside Extortionist counts only opponents' artifacts and enchantments.
#[test]
fn dockside_counts_opponents_artifacts_and_enchantments() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(1, catalog::sol_ring());
    g.add_card_to_battlefield(2, catalog::burning_vengeance());
    g.add_card_to_battlefield(0, catalog::sol_ring());
    let dockside = g.add_card_to_hand(0, catalog::dockside_extortionist());
    flood(&mut g, 0);
    g.perform_action(cast(dockside, None)).expect("Dockside");
    while !g.stack.is_empty() {
        let p = g.priority.player_with_priority;
        g.perform_action(GameAction::PassPriority).unwrap_or_else(|e| panic!("seat {p}: {e:?}"));
    }
    assert_eq!(tokens_named(&g, 0, "Treasure"), 2);
}

/// Refuse deals damage to the target spell's controller equal to its mana
/// value; Cooperate is cast from the graveyard (aftermath) and copies.
#[test]
fn refuse_hits_the_spells_controller() {
    let mut g = main_phase();
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.active_player_idx = 1;
    g.perform_action(cast(wurm, None)).expect("Wurm");
    let refuse = g.add_card_to_hand(0, catalog::refuse_cooperate());
    cast_with(&mut g, cast(refuse, Some(Target::Permanent(wurm)))).expect("Refuse");
    assert_eq!(g.players[1].life, 14, "Craw Wurm's mana value is 6");
}

/// Pristine Skywise untaps whenever you cast a noncreature spell.
#[test]
fn pristine_skywise_untaps_on_noncreature_spells() {
    let mut g = main_phase();
    let skywise = g.add_card_to_battlefield(0, catalog::pristine_skywise());
    g.battlefield_find_mut(skywise).unwrap().tapped = true;
    let temblor = g.add_card_to_hand(0, catalog::rolling_temblor());
    cast_with(&mut g, cast(temblor, None)).expect("Temblor");
    assert!(g.battlefield_find(skywise).is_some_and(|c| !c.tapped));
}

fn lands(g: &mut GameState, seat: usize, n: usize) {
    for _ in 0..n {
        g.add_card_to_battlefield(seat, catalog::island());
        g.add_card_to_battlefield(seat, catalog::mountain());
    }
}

/// A 1,000-pod census never cast Runic Repetition: the bot casts it at an
/// exiled flashback card of its own.
#[test]
fn bot_casts_runic_repetition_at_an_exiled_flashback_card() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    lands(&mut g, 0, 3);
    let temblor = g.add_card_to_exile(0, catalog::rolling_temblor());
    let rr = g.add_card_to_hand(0, catalog::runic_repetition());
    let got = HeuristicBot::new().next_action(&g, 0);
    assert!(
        matches!(got, Some(GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }) if card_id == rr && t == temblor),
        "got {got:?}"
    );
}

/// CR 707.10 — with its own sorcery on the stack, the bot flashes back
/// Increasing Vengeance from the graveyard to copy it twice.
#[test]
fn bot_copies_its_own_spell_with_a_graveyard_increasing_vengeance() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    stock_libraries(&mut g, 10);
    lands(&mut g, 0, 5);
    let iv = g.add_card_to_graveyard(0, catalog::increasing_vengeance());
    let div = g.add_card_to_hand(0, catalog::divination());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.perform_action(cast(div, None)).expect("Divination");
    let got = HeuristicBot::new().next_action(&g, 0);
    assert!(
        matches!(got, Some(GameAction::CastFlashback { card_id, target: Some(Target::Permanent(t)), .. }) if card_id == iv && t == div),
        "got {got:?}"
    );
}

/// The bot answers an opponent's six-drop with Refuse for six.
#[test]
fn bot_refuses_an_opponents_big_spell() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    stock_libraries(&mut g, 5);
    lands(&mut g, 1, 3);
    g.active_player_idx = 0;
    let wurm = g.add_card_to_hand(0, catalog::craw_wurm());
    flood(&mut g, 0);
    g.perform_action(cast(wurm, None)).expect("Wurm");
    let refuse = g.add_card_to_hand(1, catalog::refuse_cooperate());
    g.priority.player_with_priority = 1;
    let got = HeuristicBot::new().next_action(&g, 1);
    assert!(
        matches!(got, Some(GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. }) if card_id == refuse && t == wurm),
        "got {got:?}"
    );
}

