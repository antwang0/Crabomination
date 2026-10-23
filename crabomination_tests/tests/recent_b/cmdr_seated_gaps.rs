//! Commander: seated commanders' printed gaps closed (INCOMPLETE_CARDS
//! "Seated commanders with a printed gap").

use crabomination::card::CardId;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = multi_player_game(n);
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

fn cast_as(g: &mut GameState, seat: usize, id: CardId, targets: &[Target]) -> Result<(), String> {
    flood(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn ready(g: &mut GameState, seat: usize, def: crabomination::card::CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn perm(id: CardId) -> Target {
    Target::Permanent(id)
}

/// Elenda, Saint of Dusk — CR 702.11d "hexproof from instants": an opponent's
/// instant can't target it, an opponent's sorcery and its controller's own
/// instant can.
#[test]
fn elenda_has_hexproof_from_instants_only() {
    let mut g = pod(3);
    let elenda = ready(&mut g, 0, catalog::elenda_saint_of_dusk());
    let shock = g.add_card_to_hand(1, catalog::shock());
    assert!(cast_as(&mut g.clone(), 1, shock, &[perm(elenda)]).is_err(), "an opponent's instant");
    let own = g.add_card_to_hand(0, catalog::shock());
    cast_as(&mut g, 0, own, &[perm(elenda)]).expect("its controller's instant");
    let slash = g.add_card_to_hand(2, catalog::flame_slash());
    g.active_player_idx = 2;
    cast_as(&mut g, 2, slash, &[perm(elenda)]).expect("an opponent's sorcery");
    assert!(g.battlefield_find(elenda).is_none(), "4 damage after the shock's 2");
}

/// Yosei, the Morning Star — CR 601.2c: the dies trigger targets a player
/// (slot 0), then up to five permanents *that player* controls. The bot names
/// an opponent; five of their seven lands tap, the other opponent's don't.
#[test]
fn yosei_taps_up_to_five_of_the_targeted_players_permanents() {
    let mut g = pod(3);
    let lands: Vec<Vec<CardId>> =
        (0..3).map(|s| (0..7).map(|_| g.add_card_to_battlefield(s, catalog::plains())).collect()).collect();
    let yosei = ready(&mut g, 0, catalog::yosei_the_morning_star());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_as(&mut g, 0, murder, &[perm(yosei)]).expect("murder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(yosei).is_none());
    let locked: Vec<usize> = (0..3).filter(|&p| g.players[p].skip_next_untap_step > 0).collect();
    assert_eq!(locked.len(), 1);
    let p = locked[0];
    assert_ne!(p, 0, "an opponent, not Yosei's controller");
    let tapped = |s: usize| lands[s].iter().filter(|&&id| g.battlefield_find(id).is_some_and(|c| c.tapped)).count();
    assert_eq!(tapped(p), 5, "up to five, not all seven");
    assert_eq!(tapped(3 - p), 0, "the other opponent's permanents aren't eligible");
}

fn gorm_attacks(g: &mut GameState, defenders: usize) -> (CardId, Vec<CardId>) {
    use crabomination::game::types::{Attack, AttackTarget};
    let gorm = ready(g, 0, catalog::gorm_the_great());
    let bears: Vec<CardId> = (0..defenders).map(|_| ready(g, 1, catalog::grizzly_bears())).collect();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: gorm, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    (gorm, bears)
}

/// Gorm the Great — CR 509.1c: "must be blocked if able, and by two or more
/// creatures if able". With three able defenders one blocker is illegal and
/// two are enough; the bot's plan declares a legal block.
#[test]
fn gorm_must_be_blocked_by_two_when_two_can() {
    let mut g = pod(3);
    let (gorm, bears) = gorm_attacks(&mut g, 3);
    let one = g.clone().perform_action(GameAction::DeclareBlockers(vec![(bears[0], gorm)]));
    assert!(matches!(one, Err(crabomination::game::types::GameError::MustBeBlockedIfAble(id)) if id == gorm));
    let plan = crabomination::server::bot::pick_blocks_for_test(&g, 1);
    assert!(plan.iter().filter(|(_, a)| *a == gorm).count() >= 2, "{plan:?}");
    g.clone().perform_action(GameAction::DeclareBlockers(plan)).expect("the bot's plan is legal");
    g.perform_action(GameAction::DeclareBlockers(vec![(bears[0], gorm), (bears[1], gorm)])).expect("two block");
}

/// Gorm the Great — CR 509.1c maximization: with one able defender the
/// two-or-more requirement can't be met, so one blocker satisfies "must be
/// blocked" and none is still illegal.
#[test]
fn gorm_with_one_able_defender_needs_only_one() {
    let mut g = pod(3);
    let (gorm, bears) = gorm_attacks(&mut g, 1);
    assert!(g.clone().perform_action(GameAction::DeclareBlockers(vec![])).is_err(), "must be blocked");
    g.perform_action(GameAction::DeclareBlockers(vec![(bears[0], gorm)])).expect("one is the most it can get");
}

/// Absolute Virtue — CR 702.16b/e "you have protection from each of your
/// opponents": an opponent's spell can't target you and their sources' damage
/// to you is prevented; your own Flame Rift still hits you.
#[test]
fn absolute_virtue_protects_its_controller_from_opponents() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::absolute_virtue());
    let shock = g.add_card_to_hand(1, catalog::shock());
    assert!(cast_as(&mut g.clone(), 1, shock, &[Target::Player(0)]).is_err(), "can't be targeted");
    let life = |g: &GameState| (0..3).map(|p| g.players[p].life).collect::<Vec<_>>();
    let before = life(&g);
    let rift = g.add_card_to_hand(1, catalog::flame_rift());
    g.active_player_idx = 1;
    cast_as(&mut g, 1, rift, &[]).expect("flame rift");
    assert_eq!(life(&g), vec![before[0], before[1] - 4, before[2] - 4], "an opponent's damage to you is prevented");
    let own = g.add_card_to_hand(0, catalog::flame_rift());
    g.active_player_idx = 0;
    cast_as(&mut g, 0, own, &[]).expect("own flame rift");
    assert_eq!(g.players[0].life, before[0] - 4, "not protected from itself");
}

/// Olivia, Crimson Bride — CR 603.8: the creature she returns attacking gains
/// "When you don't control a legendary Vampire, exile this creature." It
/// stays while Olivia does and is exiled once she's gone.
#[test]
fn olivias_returned_creature_is_exiled_without_a_legendary_vampire() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = pod(3);
    let olivia = ready(&mut g, 0, catalog::olivia_crimson_bride());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: olivia, target: AttackTarget::Player(1) }]))
        .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_some_and(|c| c.tapped), "returned tapped");
    assert!(g.attacking().iter().any(|a| a.attacker == bears), "and attacking");
    g.step = TurnStep::PostCombatMain;
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_as(&mut g, 0, murder, &[perm(olivia)]).expect("murder");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
    assert!(g.exile.iter().any(|c| c.id == bears), "exiled, not left on the battlefield");
}

/// Surrak, Elusive Hunter — "a creature spell you control" becoming an
/// opponent's target draws a card too (CR 115.1: a counterspell targets the
/// spell).
#[test]
fn surrak_draws_when_an_opponent_targets_a_creature_spell() {
    let mut g = pod(3);
    ready(&mut g, 0, catalog::surrak_elusive_hunter());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::island());
    }
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    flood(&mut g, 0);
    g.perform_action(GameAction::CastSpell { card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("bears");
    let hand = g.players[0].hand.len();
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    flood(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(perm(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counterspell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew off the creature spell being targeted");
}

/// Alpharael, Dreaming Acolyte — "discard two cards unless you discard an
/// artifact card": with an artifact drawn, one discard (the artifact) does it.
#[test]
fn alpharael_discards_one_artifact_instead_of_two() {
    let mut g = pod(3);
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::sol_ring());
    let a = g.add_card_to_hand(0, catalog::alpharael_dreaming_acolyte());
    let hand = g.players[0].hand.len();
    cast_as(&mut g, 0, a, &[]).expect("alpharael");
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2 - 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Sol Ring"));
}

/// Edgar, King of Figaro — Two-Headed Coin (CR 705): the first coin its
/// controller flips each turn comes up heads whatever the roll; the second
/// one is an ordinary flip.
#[test]
fn edgars_first_flip_each_turn_is_heads() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::edgar_king_of_figaro());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Seat 0's first flip never asks; seat 1's comes up tails.
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(false), DecisionAnswer::Cards(vec![theirs])]));
    let a = g.add_card_to_hand(0, catalog::goblin_assassin());
    cast_as(&mut g, 0, a, &[]).expect("assassin");
    assert!(g.battlefield_find(bears).is_some(), "heads: nothing sacrificed");
    assert!(g.battlefield_find(theirs).is_none());
    assert!(g.players[0].flipped_coin_this_turn);
    // The second flip this turn is the roll's: tails sacrifices. (The first
    // Assassin goes first, or it would trigger off the second too.)
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_as(&mut g, 0, murder, &[perm(a)]).expect("murder");
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Bool(false),
        DecisionAnswer::Bool(true),
        DecisionAnswer::Cards(vec![bears]),
    ]));
    let b = g.add_card_to_hand(0, catalog::goblin_assassin());
    cast_as(&mut g, 0, b, &[]).expect("second assassin");
    assert!(g.battlefield_find(bears).is_none(), "tails the second time");
}

/// Storrev, Devkarin Lich — CR 506: its combat-damage trigger can't return a
/// card put into the graveyard this combat; one that died before combat is
/// fair game.
#[test]
fn storrev_skips_cards_put_into_the_graveyard_this_combat() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = pod(3);
    let early = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let late = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let murder = g.add_card_to_hand(0, catalog::murder());
    cast_as(&mut g, 0, murder, &[perm(early)]).expect("murder before combat");
    while g.step != TurnStep::BeginCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let m2 = g.add_card_to_hand(0, catalog::murder());
    cast_as(&mut g, 0, m2, &[perm(late)]).expect("murder in combat");
    let combat = R::PutIntoGraveyardThisCombat;
    assert!(!g.evaluate_requirement_static(&combat, &perm(early), 0, None), "died before combat");
    assert!(g.evaluate_requirement_static(&combat, &perm(late), 0, None), "died this combat");
}

/// Tomik, Distinguished Advokist — its controller's opponents can't target any
/// land on the battlefield (another opponent's too) or a land card in a
/// graveyard, and can't play land cards from graveyards (Crucible of Worlds).
#[test]
fn tomik_locks_lands_and_graveyard_lands_for_opponents() {
    let mut g = pod(3);
    g.add_card_to_battlefield(0, catalog::tomik_distinguished_advokist());
    let theirs = g.add_card_to_battlefield(2, catalog::forest());
    let buried = g.add_card_to_graveyard(1, catalog::forest());
    assert!(g.check_target_legality(&perm(theirs), 1).is_err(), "another opponent's land");
    assert!(g.check_target_legality(&perm(buried), 1).is_err(), "a land card in a graveyard");
    assert!(g.check_target_legality(&perm(theirs), 0).is_ok(), "Tomik's controller may");
    g.add_card_to_battlefield(1, catalog::crucible_of_worlds());
    assert!(!g.player_may_play_land_from_graveyard(1, buried), "no land plays from graveyards");
}

fn activate(g: &mut GameState, seat: usize, id: CardId, index: usize) -> Result<(), String> {
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

fn to_next_turn(g: &mut GameState) {
    let started = g.turn_number;
    while g.turn_number == started {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
}

fn cast_plain(g: &mut GameState, id: CardId) -> Result<(), String> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None })
        .map_err(|e| format!("{e:?}"))?;
    drain_stack(g);
    Ok(())
}

/// Kang the Conqueror — CR 500.7 + 702.193: during the extra turn its power-up
/// grants, no power-up can be activated; the following turns are unlocked.
#[test]
fn kangs_extra_turn_locks_power_up() {
    let mut g = pod(2);
    let kang = ready(&mut g, 0, catalog::kang_the_conqueror());
    let chem = ready(&mut g, 0, catalog::bold_biochemist());
    for seat in 0..2 {
        for _ in 0..8 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    flood(&mut g, 0);
    activate(&mut g, 0, kang, 0).expect("power-up");
    assert_eq!(g.players[0].extra_turns, 1);
    to_next_turn(&mut g);
    assert_eq!(g.active_player_idx, 0, "the extra turn");
    g.step = TurnStep::PreCombatMain;
    flood(&mut g, 0);
    assert!(activate(&mut g.clone(), 0, chem, 0).is_err(), "locked during that turn");
    to_next_turn(&mut g);
    assert_eq!(g.active_player_idx, 1);
    to_next_turn(&mut g);
    assert_eq!(g.active_player_idx, 0);
    g.step = TurnStep::PreCombatMain;
    flood(&mut g, 0);
    activate(&mut g, 0, chem, 0).expect("a later turn is unlocked");
}

/// Mm'menon, the Right Hand — CR 106.6: its artifacts' granted {U} can't pay
/// for a spell cast from hand, but pays for one cast from the library top.
#[test]
fn mmmenon_artifact_mana_pays_only_spells_from_outside_the_hand() {
    let mut g = pod(2);
    ready(&mut g, 0, catalog::mmmenon_the_right_hand());
    let thopter = ready(&mut g, 0, catalog::ornithopter());
    activate(&mut g, 0, thopter, 0).expect("granted {T}: Add {U}");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1);
    let in_hand = g.add_card_to_hand(0, catalog::sol_ring());
    assert!(cast_plain(&mut g, in_hand).is_err(), "not from hand");
    let on_top = g.add_card_to_library(0, catalog::sol_ring());
    cast_plain(&mut g, on_top).expect("from the library top");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 0);
}

// ── Spend limits closed with Mm'menon's (INCOMPLETE_CARDS) ──

/// Geosurge — CR 106.6: its seven {R} cast creature spells, not instants.
#[test]
fn geosurge_mana_pays_artifact_or_creature_spells_only() {
    let mut g = pod(2);
    g.players[0].mana_pool.add(Color::Red, 4);
    let surge = g.add_card_to_hand(0, catalog::geosurge());
    cast_plain(&mut g, surge).expect("Geosurge");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 7);
    let shock = g.add_card_to_hand(0, catalog::shock());
    g.priority.player_with_priority = 0;
    let no = g.perform_action(GameAction::CastSpell {
        card_id: shock, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(no.is_err(), "not an instant");
    let dragon = g.add_card_to_hand(0, catalog::shivan_dragon());
    cast_plain(&mut g, dragon).expect("a creature spell");
}

/// Helga, Skittish Seer — CR 106.6: her mana casts a creature spell with mana
/// value 4 or greater, not a smaller one.
#[test]
fn helga_mana_pays_only_big_creature_spells() {
    let mut g = pod(2);
    let helga = ready(&mut g, 0, catalog::helga_skittish_seer());
    g.battlefield_find_mut(helga).unwrap().add_counters(crabomination::card::CounterType::PlusOnePlusOne, 3);
    activate(&mut g, 0, helga, 0).expect("tap for four");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 4);
    let sable = g.add_card_to_hand(0, catalog::bronze_sable());
    assert!(cast_plain(&mut g, sable).is_err(), "mana value 2");
    let su_chi = g.add_card_to_hand(0, catalog::su_chi());
    cast_plain(&mut g, su_chi).expect("mana value 4");
}

/// Heartwood Crafter — CR 106.6: "this mana can't be spent to cast spells
/// from your hand".
#[test]
fn heartwood_crafter_mana_skips_spells_from_hand() {
    let mut g = pod(2);
    let crafter = ready(&mut g, 0, catalog::heartwood_crafter());
    activate(&mut g, 0, crafter, 0).expect("tap for {C}");
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1);
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    assert!(cast_plain(&mut g, ring).is_err());
}
