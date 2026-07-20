//! Tests for famous MTG rules misconceptions and weird interactions —
//! things players at the kitchen table reliably get wrong:
//!
//! - Deathtouch doesn't kill an indestructible creature (CR 702.12b) — but
//!   0 toughness still does, because that SBA isn't destruction (CR 704.5f).
//! - Protection from white doesn't save a creature from Wrath of God
//!   (CR 702.16 — protection is D.E.B.T.; a non-targeted "destroy all"
//!   is none of those), and neither does hexproof stop an edict
//!   (CR 702.11 — hexproof only gates *targeting*).
//! - Regeneration can't beat "can't be regenerated" (CR 701.15g).
//! - Multiple instances of lifelink are redundant (CR 702.15f).
//! - Killing a blocker doesn't make the attacker unblocked: it stays
//!   blocked and deals no combat damage without trample (CR 509.1h /
//!   510.1c), while a trampler assigns everything to the player.
//! - Tapping a blocker after blocks doesn't stop its combat damage
//!   (CR 509.1g — tapped creatures can't *be declared* blockers, but a
//!   declared blocker deals damage even while tapped).
//! - Summoning sickness doesn't stop blocking (CR 302.6 only restricts
//!   attacking and {T} costs).
//! - Damage doesn't reduce toughness; it's marked and wears off in the
//!   cleanup step (CR 120.6 / 514.2), which is why Giant Growth "saves"
//!   a bolted creature (CR 608.2 — last in, first resolved).
//! - Shroud stops your *own* spells, unlike hexproof (CR 702.18).
//! - An Aura spell whose target dies in response fizzles — it never
//!   enters the battlefield (CR 608.2b).
//! - Trample requires assigning lethal to the blocker first; a 3/3
//!   trampler blocked by a 4/4 puts nothing through (CR 510.1c).
//! - Attacking taps; blocking never does (CR 508 / 509).
//! - An exile-and-return blink makes a new object: damage gone,
//!   summoning-sick again (CR 400.7).
//! - Exile isn't "dies" — death triggers need battlefield → graveyard
//!   (CR 700.4).
//! - Mana pools empty at the end of every step and phase (CR 500.4),
//!   and mana burn no longer exists.

use crabomination::card::{CardDefinition, CardType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::two_player_game;
use crabomination::game::*;
use crabomination::mana::Color;

/// A bare N/M creature carrying the given keywords.
fn body_kw(name: &'static str, p: i32, t: i32, keywords: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        power: p,
        toughness: t,
        keywords,
        ..Default::default()
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── CR 702.12b — deathtouch vs indestructible ────────────────────────────────

/// "Deathtouch kills everything" — no: deathtouch makes damage *lethal*,
/// and indestructible ignores lethal-damage destruction.
#[test]
fn misconception_deathtouch_does_not_kill_indestructible() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Asp", 1, 1, vec![Keyword::Deathtouch]));
    let blk = g.add_card_to_battlefield(
        1, body_kw("Statue", 3, 3, vec![Keyword::Indestructible]));
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield_find(blk).is_some(),
        "indestructible blocker survives deathtouch damage");
    assert!(g.battlefield_find(atk).is_none(),
        "the 1/1 attacker dies to the 3/3's damage as usual");
}

// ── CR 704.5f — indestructible vs 0 toughness ─────────────────────────────────

/// "Indestructible can't die" — the 0-toughness SBA isn't destruction,
/// so -X/-X effects kill indestructible creatures just fine.
#[test]
fn misconception_indestructible_dies_to_zero_toughness() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(
        0, body_kw("Statue", 3, 3, vec![Keyword::Indestructible]));
    g.battlefield_find_mut(c).unwrap().toughness_bonus -= 3;
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(c).is_none(),
        "0 toughness puts an indestructible creature in the graveyard (not destruction)");
}

// ── CR 702.16 — protection doesn't stop board wipes ──────────────────────────

/// "Protection from white saves it from Wrath of God" — protection only
/// covers Damage, Enchant/Equip, Block, Target. A sweep does none of those.
#[test]
fn misconception_protection_does_not_stop_wrath_of_god() {
    let mut g = two_player_game();
    let pro = g.add_card_to_battlefield(
        1, body_kw("Paladin", 2, 2, vec![Keyword::Protection(Color::White)]));
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, wrath);
    assert!(g.battlefield_find(pro).is_none(),
        "Wrath of God doesn't target and doesn't deal damage — protection is irrelevant");
}

// ── CR 701.15g — "can't be regenerated" ───────────────────────────────────────

/// "I regenerate in response to Wrath" — a stamped shield does nothing
/// against destruction that forbids regeneration.
#[test]
fn misconception_regeneration_does_not_beat_wrath_of_god() {
    let mut g = two_player_game();
    let troll = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(troll).unwrap().regeneration_shields = 1;
    let wrath = g.add_card_to_hand(0, catalog::wrath_of_god());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast(&mut g, wrath);
    assert!(g.battlefield_find(troll).is_none(),
        "\"they can't be regenerated\" ignores the shield");
}

// ── CR 702.11 — hexproof vs sacrifice ─────────────────────────────────────────

/// "Hexproof protects my creature from everything" — an edict targets the
/// *player*, so hexproof (a targeting gate on the creature) never applies.
#[test]
fn misconception_hexproof_does_not_stop_an_edict() {
    let mut g = two_player_game();
    let hex = g.add_card_to_battlefield(
        1, body_kw("Hexer", 2, 2, vec![Keyword::Hexproof]));
    let edict = g.add_card_to_hand(0, catalog::diabolic_edict());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, edict, Target::Player(1));
    assert!(g.battlefield_find(hex).is_none(),
        "Diabolic Edict targets the player; the hexproof creature is sacrificed");
}

// ── CR 702.15f — multiple lifelink instances ──────────────────────────────────

/// "Double lifelink means double life" — multiple instances of lifelink
/// are redundant; 3 damage dealt gains exactly 3 life.
#[test]
fn misconception_multiple_lifelink_instances_do_not_stack() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(
        0, body_kw("Cleric", 3, 3, vec![Keyword::Lifelink, Keyword::Lifelink]));
    g.clear_sickness(atk);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].life, life + 3,
        "two lifelink instances still gain only the 3 damage dealt");
}

// ── CR 509.1h / 510.1c — removing a blocker mid-combat ───────────────────────

/// "Bolt the blocker and my attacker gets through" — no: the attacker
/// stays *blocked* and (without trample) deals no combat damage at all.
#[test]
fn misconception_killing_the_blocker_does_not_unblock_the_attacker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Brute", 3, 3, vec![]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    // Bolt the blocker after blocks are locked in.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bolt, Target::Permanent(blk));
    assert!(g.battlefield_find(blk).is_none(), "the blocker died to the bolt");
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life,
        "the attacker remains blocked; without trample no damage reaches the player");
}

/// …but a *trampler* whose blocker was removed assigns all its combat
/// damage to the defending player (CR 510.1c / 702.19g).
#[test]
fn misconception_trampler_with_dead_blocker_hits_for_full() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Crasher", 3, 3, vec![Keyword::Trample]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bolt, Target::Permanent(blk));
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life - 3,
        "with no blockers left, the trampler assigns everything to the player");
}

// ── CR 509.1g — tapping a declared blocker ────────────────────────────────────

/// "Tap his blocker so it can't deal damage" — tapped creatures can't be
/// *declared* as blockers, but an already-declared blocker deals its combat
/// damage even while tapped.
#[test]
fn misconception_tapping_a_declared_blocker_does_not_stop_its_damage() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Brute", 2, 3, vec![]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    // Tap the blocker after blocks are declared.
    g.battlefield_find_mut(blk).unwrap().tapped = true;
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.battlefield_find(atk).unwrap().damage, 2,
        "the tapped blocker still dealt its 2 combat damage");
    assert!(g.battlefield_find(blk).is_none(),
        "the exchange happened normally: the 2/2 blocker died to the attacker's 2 power");
}

// ── CR 506.4b — tapping a declared attacker ──────────────────────────────────

/// "Tap the attacker before damage so it can't hit me" — CR 506.4b: tapping
/// (or untapping) a creature that's already been declared as an attacker
/// doesn't remove it from combat and doesn't prevent its combat damage.
#[test]
fn misconception_tapping_a_declared_attacker_does_not_stop_its_damage() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Tap the attacker after it's declared — it stays in combat.
    g.battlefield_find_mut(atk).unwrap().tapped = true;
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 18, "the tapped attacker still dealt its 2 damage");
}

// ── CR 302.6 — summoning sickness only restricts attacking ───────────────────

/// "It just came down, it can't block" — summoning sickness stops attacking
/// and {T} abilities, never blocking.
#[test]
fn misconception_summoning_sick_creature_can_block() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    // Fresh blocker for seat 0 — do NOT clear sickness.
    let blk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::Upkeep;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)]))
        .expect("a summoning-sick creature can legally block");
}

// ── CR 120.6 / 514.2 — damage is marked, not toughness loss ──────────────────

/// "The bolt made it a 4/1" — damage doesn't change toughness; it's marked
/// on the creature and all of it wears off during cleanup.
#[test]
fn misconception_damage_does_not_reduce_toughness_and_wears_off() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, body_kw("Golem", 4, 4, vec![]));
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, bolt, Target::Permanent(big));
    let inst = g.battlefield_find(big).expect("survives with damage marked");
    assert_eq!(inst.damage, 3, "3 damage marked");
    let computed = g.computed_permanent(big).unwrap();
    assert_eq!(computed.toughness, 4, "toughness is still 4 — damage isn't a -N/-N");
    // Pass through cleanup into the next turn: the damage wears off.
    advance_to(&mut g, TurnStep::End);
    advance_to(&mut g, TurnStep::Upkeep);
    assert_eq!(g.battlefield_find(big).unwrap().damage, 0,
        "all marked damage is removed in the cleanup step");
}

// ── CR 608.2 — the classic combat trick ───────────────────────────────────────

/// "You can't save it, the bolt already targeted it" — Giant Growth cast in
/// response resolves first, so the 2/2 is a 5/5 when the bolt's 3 damage lands.
#[test]
fn misconception_giant_growth_in_response_saves_the_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let growth = g.add_card_to_hand(1, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    // Respond with Giant Growth before the bolt resolves.
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: growth, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("respond with growth");
    drain_stack(&mut g);
    let inst = g.battlefield_find(bear).expect("the bear survives");
    assert_eq!(inst.damage, 3, "the bolt still dealt its 3");
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 5,
        "growth resolved first, making it a 5/5");
}

// ── CR 702.18 — shroud vs hexproof ────────────────────────────────────────────

/// "Shroud is just old hexproof" — no: shroud stops *everyone's* spells,
/// including your own Giant Growth.
#[test]
fn misconception_shroud_blocks_your_own_spells_too() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, body_kw("Hermit", 2, 2, vec![Keyword::Shroud]));
    let growth = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let res = g.perform_action(GameAction::CastSpell {
        card_id: growth, target: Some(Target::Permanent(c)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(res.is_err(), "shroud stops the controller's own targeted spells");
}

// ── CR 608.2b — an Aura fizzles if its target disappears ─────────────────────

/// "The Aura enters anyway and just falls off" — no: an Aura *spell* whose
/// target is gone at resolution doesn't resolve; it goes straight to the
/// graveyard without ever hitting the battlefield.
#[test]
fn misconception_aura_fizzles_when_its_target_dies_in_response() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::holy_strength());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the Aura");
    // Opponent bolts the enchant target in response.
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("respond with bolt");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the bear died to the bolt");
    assert!(g.battlefield_find(aura).is_none(), "the Aura never entered the battlefield");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == aura),
        "the fizzled Aura spell goes to its owner's graveyard");
}

// ── CR 510.1c — trample requires lethal to the blocker first ─────────────────

/// "Trample means the extra power always gets through" — the attacker must
/// assign *lethal* damage to the blocker first; a 3/3 trampler blocked by a
/// 4/4 has nothing left over.
#[test]
fn misconception_trample_assigns_lethal_to_the_blocker_first() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Boar", 3, 3, vec![Keyword::Trample]));
    let blk = g.add_card_to_battlefield(1, body_kw("Wall", 4, 4, vec![]));
    g.clear_sickness(atk);
    let life = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, life,
        "3 power < 4 toughness: all damage goes to the blocker, none tramples over");
    assert_eq!(g.battlefield_find(blk).unwrap().damage, 3, "the wall soaked all 3");
}

// ── CR 509 — blocking doesn't tap ─────────────────────────────────────────────

/// "Blocking taps the creature" — attacking taps (without vigilance);
/// blocking never does.
#[test]
fn misconception_blocking_does_not_tap_the_blocker() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, body_kw("Brute", 1, 4, vec![]));
    let blk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(atk).unwrap().tapped, "attacking taps the attacker");
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blk, atk)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(!g.battlefield_find(blk).unwrap().tapped,
        "the blocker never taps for blocking");
}

// ── CR 400.7 — a blinked creature is a new object ─────────────────────────────

/// "It's the same creature, it can still attack" — exile-and-return
/// (Cloudshift) makes a *new object*: marked damage is gone, and so is any
/// memory of having been under your control since the turn began — it's
/// summoning-sick again.
#[test]
fn misconception_blinked_creature_is_a_new_object() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Mark 1 damage on it (survives: 1 < 2 toughness).
    g.battlefield_find_mut(bear).unwrap().damage = 1;
    g.step = TurnStep::PreCombatMain;
    // Blink it.
    let shift = g.add_card_to_hand(0, catalog::cloudshift());
    g.players[0].mana_pool.add(Color::White, 1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, shift, Target::Permanent(bear));
    let inst = g.battlefield_find(bear).expect("the bear returned");
    assert_eq!(inst.damage, 0, "marked damage doesn't follow the new object");
    assert!(inst.summoning_sick,
        "the returned creature is a new object — summoning-sick again");
}

// ── CR 700.4 — exile is not "dies" ────────────────────────────────────────────

/// "Removing it still triggers its death ability" — "dies" means battlefield
/// → graveyard. Exiling an Afterlife creature mints no Spirits.
#[test]
fn misconception_exile_does_not_trigger_dies_abilities() {
    use crabomination::card::CreatureType;
    use crabomination::effect::shortcut;
    let mut g = two_player_game();
    let cleric = CardDefinition {
        name: "Cleric",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![shortcut::afterlife(1)],
        ..Default::default()
    };
    let c = g.add_card_to_battlefield(1, cleric);
    let swords = g.add_card_to_hand(0, catalog::swords_to_plowshares());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, swords, Target::Permanent(c));
    assert!(g.battlefield_find(c).is_none(), "the cleric was exiled");
    let spirits = g.battlefield.iter().filter(|p| p.controller == 1
        && p.definition.subtypes.creature_types.contains(&CreatureType::Spirit)).count();
    assert_eq!(spirits, 0, "exile isn't dying — afterlife never triggers");
}

// ── CR 704.5m — protection strips Auras of that quality ──────────────────────

/// "Protection only matters for new spells" — no: an Aura already on the
/// creature becomes illegally attached the moment the creature gains
/// protection from its color, and the SBA puts it in the graveyard.
#[test]
fn misconception_gaining_protection_makes_an_attached_aura_fall_off() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Attach Holy Strength (a white Aura) the normal way.
    let aura = g.add_card_to_hand(0, catalog::holy_strength());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, aura, Target::Permanent(bear));
    assert_eq!(g.battlefield_find(aura).unwrap().attached_to, Some(bear),
        "the Aura is attached");
    // The bear gains protection from white.
    std::sync::Arc::make_mut(&mut g.battlefield_find_mut(bear).unwrap().definition)
        .keywords.push(Keyword::Protection(Color::White));
    let _ = g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the creature stays");
    assert!(g.battlefield_find(aura).is_none(),
        "the white Aura is illegally attached and falls off");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == aura),
        "…into its owner's graveyard (CR 704.5m)");
}

// ── CR 500.4 — mana pools empty every step ────────────────────────────────────

/// "I'll float this mana for later in the turn" — unspent mana empties at
/// the end of *every* step and phase, not just end of turn. (And since 2010,
/// no mana burn — it just vanishes, no life loss.)
#[test]
fn misconception_floated_mana_does_not_survive_the_step() {
    let mut g = two_player_game();
    advance_to(&mut g, TurnStep::PreCombatMain);
    g.players[0].mana_pool.add(Color::Green, 3);
    let life = g.players[0].life;
    advance_to(&mut g, TurnStep::BeginCombat);
    assert_eq!(g.players[0].mana_pool.total(), 0,
        "the floated mana emptied at the phase boundary");
    assert_eq!(g.players[0].life, life, "and no mana burn — that rule died in M10");
}
