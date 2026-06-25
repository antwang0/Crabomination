//! Functionality tests for `catalog::sets::decks::ltr` and the underlying
//! Ring mechanic (CR 701.54 — `Effect::RingTempts`, per-player temptation
//! level + Ring-bearer designation).

use crate::card::CreatureType;
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget, Target, TurnStep};
use crate::game::*;
use crate::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

// ── The Ring engine (CR 701.54) ─────────────────────────────────────────────

/// Temptation designates the highest-power creature you control as Ring-bearer.
#[test]
fn ring_tempts_picks_strongest_creature() {
    let mut g = two_player_game();
    let _small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let mut ev = vec![];
    g.ring_tempts(0, &mut ev);
    assert_eq!(g.players[0].ring_temptations, 1);
    assert_eq!(g.effective_ring_bearer(0), Some(big), "biggest creature is bearer");
}

/// Level 1 — the Ring-bearer can't be blocked by creatures with greater power.
#[test]
fn ring_bearer_cant_be_blocked_by_greater_power() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ev = vec![];
    g.ring_tempts(0, &mut ev);
    assert_eq!(g.effective_ring_bearer(0), Some(bearer));
    assert!(!g.blocker_can_block_attacker(big, bearer), "3/3 can't block the 2/2 bearer");
    assert!(g.blocker_can_block_attacker(small, bearer), "equal-power blocker is fine");
}

/// Level 2 — when the Ring-bearer attacks, its controller loots (draw, discard).
#[test]
fn ring_level_2_bearer_attack_loots() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bearer);
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(0, catalog::mountain());
    let mut ev = vec![];
    g.ring_tempts(0, &mut ev);
    g.ring_tempts(0, &mut ev); // level 2
    let lib = g.players[0].library.len();
    let gy = g.players[0].graveyard.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bearer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "drew a card for the loot");
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded a card for the loot");
}

/// Level 3 — a creature blocking the Ring-bearer is sacrificed at end of combat.
#[test]
fn ring_level_3_blocker_sacrificed_at_end_of_combat() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    g.clear_sickness(bearer);
    // Library + spare hand card cover the level-2 attack loot that also fires.
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_hand(0, catalog::forest());
    let blocker = g.add_card_to_battlefield(1, catalog::wall_of_wood()); // survives 3 dmg
    let mut ev = vec![];
    for _ in 0..3 { g.ring_tempts(0, &mut ev); } // level 3
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bearer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, bearer)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield_find(blocker).is_none(), "blocker sacrificed at end of combat");
}

/// Level 4 — when the Ring-bearer deals combat damage to a player, each
/// opponent loses 3 life (on top of the combat damage).
#[test]
fn ring_level_4_combat_damage_drains() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bearer);
    // Library + spare hand card cover the level-2 attack loot that also fires.
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.add_card_to_hand(0, catalog::forest());
    let mut ev = vec![];
    for _ in 0..4 { g.ring_tempts(0, &mut ev); } // level 4
    let before = g.players[1].life;
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bearer, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, before - 2 - 3, "2 combat + 3 Ring drain");
}

/// The designation clears when the bearer leaves the battlefield (CR 701.54b).
#[test]
fn ring_bearer_clears_when_creature_leaves() {
    let mut g = two_player_game();
    let bearer = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = vec![];
    g.ring_tempts(0, &mut ev);
    assert_eq!(g.effective_ring_bearer(0), Some(bearer));
    g.remove_from_battlefield_to_exile(bearer);
    assert_eq!(g.effective_ring_bearer(0), None, "bearer gone → no designation");
}

/// Temptations cap at 4 even past four triggers.
#[test]
fn ring_temptations_cap_at_four() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ev = vec![];
    for _ in 0..6 { g.ring_tempts(0, &mut ev); }
    assert_eq!(g.players[0].ring_temptations, 4);
}

// ── LTR cards ────────────────────────────────────────────────────────────

/// Birthday Escape draws a card and tempts.
#[test]
fn birthday_escape_draws_and_tempts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::birthday_escape());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, id);
    assert_eq!(g.players[0].ring_temptations, 1, "tempted once");
}

/// The Black Breath gives opponents' creatures -1/-1 and tempts.
#[test]
fn the_black_breath_shrinks_opponents_and_tempts() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies at -1/-1? no, 1/1
    let id = g.add_card_to_hand(0, catalog::the_black_breath());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let foe_cp = g.compute_battlefield().into_iter().find(|c| c.id == foe).unwrap();
    assert_eq!((foe_cp.power, foe_cp.toughness), (1, 1), "opponent creature shrunk");
    assert_eq!(g.players[0].ring_temptations, 1);
}

/// Rohirrim Lancer tempts when it dies.
#[test]
fn rohirrim_lancer_tempts_on_death() {
    let mut g = two_player_game();
    let lancer = g.add_card_to_battlefield(0, catalog::rohirrim_lancer());
    let ev = g.remove_to_graveyard_with_triggers(lancer);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].ring_temptations, 1, "tempted on death");
}

/// Bilbo tempts on ETB and makes a Treasure on combat damage.
#[test]
fn bilbo_tempts_on_etb_and_treasures_on_damage() {
    let mut g = two_player_game();
    let bilbo = g.add_card_to_hand(0, catalog::bilbo_retired_burglar());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bilbo);
    assert_eq!(g.players[0].ring_temptations, 1, "tempted on ETB");
    let bilbo_id = g.battlefield.iter().find(|c| c.definition.name == "Bilbo, Retired Burglar").unwrap().id;
    g.clear_sickness(bilbo_id);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bilbo_id, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "Treasure made on combat damage");
}

/// Call of the Ring tempts at upkeep, and pays 2 life to draw on choosing a bearer.
#[test]
fn call_of_the_ring_upkeep_tempt_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::call_of_the_ring());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    // Fire player 0's upkeep step directly (active player is 0 by default).
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].ring_temptations >= 1, "tempted at upkeep");
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Easterling Vanguard amasses Orcs 1 when it dies.
#[test]
fn easterling_vanguard_amasses_on_death() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::easterling_vanguard());
    let ev = g.remove_to_graveyard_with_triggers(v);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let army = g.battlefield.iter().find(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&CreatureType::Army));
    assert!(army.is_some(), "Orc Army token created");
}

/// Mirkwood Bats drains each opponent when you create a token.
#[test]
fn mirkwood_bats_drains_on_token_creation() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mirkwood_bats());
    let before = g.players[1].life;
    // Resolve a Treasure-making spell to create a token under player 0.
    let id = g.add_card_to_hand(0, catalog::strike_it_rich());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, id);
    assert_eq!(g.players[1].life, before - 1, "opponent drained on token creation");
}

/// Snarling Warg grows while you control a Goblin or Orc.
#[test]
fn snarling_warg_grows_with_goblin() {
    let mut g = two_player_game();
    let warg = g.add_card_to_battlefield(0, catalog::snarling_warg());
    let base = g.compute_battlefield().into_iter().find(|c| c.id == warg).unwrap();
    assert_eq!(base.power, 3, "3/4 with no Goblin/Orc");
    g.add_card_to_battlefield(0, catalog::battle_scarred_goblin()); // a Goblin
    let buffed = g.compute_battlefield().into_iter().find(|c| c.id == warg).unwrap();
    assert_eq!((buffed.power, buffed.toughness), (4, 4), "+1/+0 with a Goblin");
}

/// Wose Pathfinder taps for any color and can pump another creature.
#[test]
fn wose_pathfinder_pumps_another() {
    let mut g = two_player_game();
    let wose = g.add_card_to_battlefield(0, catalog::wose_pathfinder());
    g.clear_sickness(wose);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(6);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wose, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate pump");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (5, 5), "bear got +3/+3");
    assert!(b.keywords.contains(&crate::card::Keyword::Trample), "and trample");
}

/// Battle-Scarred Goblin pings its blocker when blocked.
#[test]
fn battle_scarred_goblin_pings_blocker() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::battle_scarred_goblin());
    g.clear_sickness(gob);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: gob, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, gob)])).expect("block");
    drain_stack(&mut g);
    let blk = g.battlefield_find(blocker).expect("blocker still alive");
    assert_eq!(blk.damage, 1, "blocker took 1 from the becomes-blocked trigger");
}

/// Banish from Edoras exiles a creature, cheaper against a tapped one.
#[test]
fn banish_from_edoras_exiles_tapped_for_less() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::banish_from_edoras());
    // {4}{W} normally; {2} less against a tapped target → {2}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast at reduced cost");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == foe), "creature exiled");
}

/// Wizard's Rockets: sacrificing it for mana draws a card.
#[test]
fn wizards_rockets_sac_for_mana_draws() {
    let mut g = two_player_game();
    let rockets = g.add_card_to_battlefield(0, catalog::wizards_rockets());
    g.battlefield_find_mut(rockets).unwrap().tapped = false;
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rockets, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate mana ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rockets).is_none(), "sacrificed");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew on death");
    assert!(g.players[0].mana_pool.total() >= 1, "mana added");
}

/// Took Reaper tempts on death.
#[test]
fn took_reaper_tempts_on_death() {
    let mut g = two_player_game();
    let r = g.add_card_to_battlefield(0, catalog::took_reaper());
    let ev = g.remove_to_graveyard_with_triggers(r);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.players[0].ring_temptations, 1);
}

/// Erebor Flamesmith pings each opponent when you cast an instant.
#[test]
fn erebor_flamesmith_pings_on_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::erebor_flamesmith());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 1, "Flamesmith pinged opponent for 1");
}

/// Prince Imrahil makes a Soldier on your second draw each turn.
#[test]
fn prince_imrahil_second_draw_makes_token() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::prince_imrahil_the_fair());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first draw — no token
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second draw — token
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").count();
    assert_eq!(soldiers, 1, "one Soldier on the second draw");
}

/// Slip On the Ring blinks your own creature and tempts.
#[test]
fn slip_on_the_ring_blinks_and_tempts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::slip_on_the_ring());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Slip On the Ring");
    drain_stack(&mut g);
    assert_eq!(g.players[0].ring_temptations, 1, "tempted");
    let creatures = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_creature()).count();
    assert_eq!(creatures, 1, "creature returned to the battlefield");
}

/// Rally at the Hornburg makes two Soldiers and gives Humans haste.
#[test]
fn rally_at_the_hornburg_tokens_and_haste() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rally_at_the_hornburg());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    let soldiers: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").map(|c| c.id).collect();
    assert_eq!(soldiers.len(), 2, "two Soldier tokens");
    let cp = g.compute_battlefield();
    let s = cp.iter().find(|c| c.id == soldiers[0]).unwrap();
    assert!(s.keywords.contains(&crate::card::Keyword::Haste), "Humans gain haste");
}

/// Haradrim Spearmaster pumps another creature at the beginning of your combat.
#[test]
fn haradrim_spearmaster_pumps_another() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::haradrim_spearmaster());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::BeginCombat;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 2), "bear got +1/+0");
}

/// Fog on the Barrow-Downs stops the enchanted creature from attacking.
#[test]
fn fog_on_the_barrow_downs_locks_down() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::fog_on_the_barrow_downs());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("enchant the bear");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == foe).unwrap();
    assert!(b.keywords.contains(&crate::card::Keyword::CantAttack), "enchanted creature can't attack");
}

/// Soldier of the Grey Host pumps a creature on ETB.
#[test]
fn soldier_of_the_grey_host_pumps_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::soldier_of_the_grey_host());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 2), "bear got +2/+0");
}

/// Westfold Rider sacrifices itself to destroy an enchantment.
#[test]
fn westfold_rider_destroys_enchantment() {
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::westfold_rider());
    let ench = g.add_card_to_battlefield(1, catalog::pacifism());
    g.perform_action(GameAction::ActivateAbility {
        card_id: rider, ability_index: 0, target: Some(Target::Permanent(ench)),
        additional_targets: vec![], x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rider).is_none(), "Rider sacrificed");
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// Bombadil's Song pumps a creature, grants hexproof, and tempts.
#[test]
fn bombadils_song_pumps_hexproof_and_tempts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bombadils_song());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "bear got +1/+1");
    assert!(b.keywords.contains(&crate::card::Keyword::Hexproof), "and hexproof");
    assert_eq!(g.players[0].ring_temptations, 1, "and tempted");
}

/// Mordor Muster draws, loses 1 life, and amasses.
#[test]
fn mordor_muster_draws_loses_amasses() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::mordor_muster());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    cast(&mut g, id);
    assert_eq!(g.players[0].life, life - 1, "lost 1 life");
    assert!(g.battlefield.iter().any(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&CreatureType::Army)), "amassed an Army");
}

/// Bag End Porter grows by the number of legendary creatures you control.
#[test]
fn bag_end_porter_scales_with_legends() {
    let mut g = two_player_game();
    let porter = g.add_card_to_battlefield(0, catalog::bag_end_porter());
    g.clear_sickness(porter);
    g.add_card_to_battlefield(0, catalog::prince_imrahil_the_fair()); // legendary
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: porter, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let p = cp.iter().find(|c| c.id == porter).unwrap();
    assert_eq!((p.power, p.toughness), (5, 5), "4/4 +1/+1 for the one legend");
}
