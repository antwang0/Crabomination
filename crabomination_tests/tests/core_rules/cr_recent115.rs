//! CR 508.1d — a creature-level "attacks each combat if able" requirement is
//! obeyed as far as the restrictions allow, not one creature at a time: two
//! Juggernauts under Silent Arbiter ("no more than one creature can attack")
//! had no legal declaration at all.

use crabomination::card::{CardDefinition, CardId};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn to_attacks(g: &mut GameState) {
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
}

fn declare(g: &mut GameState, attackers: &[CardId], defender: usize) -> Result<Vec<GameEvent>, GameError> {
    let attacks = attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(defender) }).collect();
    g.clone().perform_action(GameAction::DeclareAttackers(attacks))
}

/// CR 508.1d / 506.2 — with a cap of one, one Juggernaut attacking obeys the
/// most requirements possible; none attacking obeys fewer, and a non-required
/// attacker in the one slot obeys fewer too.
#[test]
fn cr_508_1d_requirements_are_maximized_under_an_attacker_cap() {
    let mut g = two_player_game();
    ready(&mut g, 1, catalog::silent_arbiter());
    let j1 = ready(&mut g, 0, catalog::juggernaut());
    let j2 = ready(&mut g, 0, catalog::juggernaut());
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    to_attacks(&mut g);
    declare(&mut g, &[j1], 1).expect("one Juggernaut fills the cap");
    declare(&mut g, &[j2], 1).expect("either one");
    assert!(declare(&mut g, &[], 1).is_err(), "no attack obeys none");
    assert!(declare(&mut g, &[bear], 1).is_err(), "the bear takes a required slot");
    assert!(declare(&mut g, &[j1, j2], 1).is_err(), "the cap still holds");
}

/// The same under Crawlspace's per-defender cap (CR 508.1d with a
/// restriction on the defender): two of three Juggernauts at the one opponent.
#[test]
fn cr_508_1d_requirements_are_maximized_under_a_defender_cap() {
    let mut g = two_player_game();
    ready(&mut g, 1, catalog::crawlspace());
    let js: Vec<CardId> = (0..3).map(|_| ready(&mut g, 0, catalog::juggernaut())).collect();
    to_attacks(&mut g);
    declare(&mut g, &js[..2], 1).expect("two fill the cap");
    assert!(declare(&mut g, &js[..1], 1).is_err(), "a second could still attack");
}

/// The bot's declaration under the cap is a legal one.
#[test]
fn bot_declares_a_legal_capped_attack() {
    use crabomination::server::bot::pick_attacks;
    let mut g = two_player_game();
    ready(&mut g, 1, catalog::silent_arbiter());
    let j1 = ready(&mut g, 0, catalog::juggernaut());
    let j2 = ready(&mut g, 0, catalog::juggernaut());
    to_attacks(&mut g);
    let attacks = pick_attacks(&g, 0);
    assert_eq!(attacks.len(), 1, "{attacks:?}");
    assert!([j1, j2].contains(&attacks[0].attacker));
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("legal");
}

fn goad(g: &mut GameState, goader: usize, id: CardId) {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::effect::{Effect, Selector};
    let ctx = EffectContext::for_spell(goader, None, 0, 0);
    let name = g.battlefield_find(id).unwrap().definition.name.to_string();
    g.resolve_effect(&Effect::Goad { what: Selector::EachPermanent(R::HasName(name)) }, &ctx).expect("goad");
}

/// CR 508.1d / 701.15b — a goaded creature that can't attack alone: with no
/// other creature able to join, attacking breaks a restriction, so staying
/// home is legal; with one, both attacking obeys the requirement, so the
/// empty declaration is not.
#[test]
fn cr_508_1d_a_goaded_creature_that_cant_attack_alone() {
    let mut g = multi_player_game(3);
    let kronch = ready(&mut g, 0, catalog::raging_kronch());
    goad(&mut g, 1, kronch);
    to_attacks(&mut g);
    declare(&mut g, &[], 2).expect("alone it can't attack, so it stays home");
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    assert!(declare(&mut g, &[], 2).is_err(), "the bear lets it attack");
    declare(&mut g, &[kronch, bear], 2).expect("both");
    use crabomination::server::bot::pick_attacks;
    let attacks = pick_attacks(&g, 0);
    assert!(attacks.iter().any(|a| a.attacker == kronch), "{attacks:?}");
    g.perform_action(GameAction::DeclareAttackers(attacks)).expect("the bot's declaration is legal");
}

/// CR 508.1d — a goaded Okk with no bigger creature able to attack can't
/// attack, so it stays home; with a bigger one, the pair must attack.
#[test]
fn cr_508_1d_a_goaded_okk() {
    let mut g = multi_player_game(3);
    let okk = ready(&mut g, 0, catalog::okk());
    goad(&mut g, 1, okk);
    ready(&mut g, 0, catalog::grizzly_bears());
    to_attacks(&mut g);
    declare(&mut g, &[], 2).expect("nothing bigger to attack with it");
    use crabomination::server::bot::pick_attacks;
    let attacks = pick_attacks(&g, 0);
    declare(&mut g, &attacks.iter().map(|a| a.attacker).collect::<Vec<_>>(), 2).expect("the bot's declaration");
    let wurm = ready(&mut g, 0, catalog::craw_wurm());
    assert!(declare(&mut g, &[], 2).is_err(), "the Wurm lets it attack");
    declare(&mut g, &[okk, wurm], 2).expect("both");
}

/// CR 509.1c — the blocking side: under Silent Arbiter ("no more than one
/// creature can block"), a Lure attacker gets one of the two able blockers;
/// both would break the cap and neither leaves a requirement unobeyed that
/// one could meet.
#[test]
fn cr_509_1c_lure_requirements_are_maximized_under_a_blocker_cap() {
    let mut g = two_player_game();
    ready(&mut g, 0, catalog::silent_arbiter());
    let lured = ready(&mut g, 0, catalog::grizzly_bears());
    let lure = g.add_card_to_battlefield(0, catalog::lure());
    g.battlefield_find_mut(lure).unwrap().attached_to = Some(lured);
    let b1 = ready(&mut g, 1, catalog::grizzly_bears());
    let b2 = ready(&mut g, 1, catalog::grizzly_bears());
    to_attacks(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: lured, target: AttackTarget::Player(1) }]))
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    let blocks = |g: &GameState, b: Vec<(CardId, CardId)>| g.clone().perform_action(GameAction::DeclareBlockers(b));
    blocks(&g, vec![(b1, lured)]).expect("one blocker fills the cap");
    assert!(blocks(&g, vec![]).is_err(), "one could block");
    assert!(blocks(&g, vec![(b1, lured), (b2, lured)]).is_err(), "the cap holds");
    let picked = crabomination::server::bot::pick_blocks_for_test(&g, 1);
    blocks(&g, picked).expect("the bot's blocks are legal");
}
