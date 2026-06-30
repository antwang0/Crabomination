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

/// Hithlain Knots taps a creature, scries, and draws.
#[test]
fn hithlain_knots_taps_and_draws() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::hithlain_knots());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped");
    assert_eq!(g.players[0].hand.len(), hand, "net hand unchanged (cast 1, drew 1)");
}

/// Lossarnach Captain taps an opponent's creature when a Human enters.
#[test]
fn lossarnach_captain_taps_on_human_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lossarnach_captain());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Casting another Human fires the captain's "another Human enters" tap.
    let human = g.add_card_to_hand(0, catalog::rohirrim_lancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, human);
    assert!(g.battlefield_find(foe).unwrap().tapped, "opponent's creature tapped");
}

/// Dúnedain Blade equips for +2/+1.
#[test]
fn dunedain_blade_equips() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::dunedain_blade());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: blade, target: bear }).expect("equip");
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 3), "bear got +2/+1");
}

/// Erkenbrand pumps the team when a Human enters.
#[test]
fn erkenbrand_pumps_team_on_human_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::erkenbrand_lord_of_westfold());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let human = g.add_card_to_hand(0, catalog::rohirrim_lancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, human);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 2), "team got +1/+0");
}

/// Many Partings tutors a basic land to hand and makes a Food.
#[test]
fn many_partings_fetches_and_makes_food() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::many_partings());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "basic to hand");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food made");
}

/// Goblin Fireleaper deals its power to a creature when it dies.
#[test]
fn goblin_fireleaper_pings_on_death() {
    let mut g = two_player_game();
    let gob = g.add_card_to_battlefield(0, catalog::goblin_fireleaper());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(foe))]));
    let ev = g.remove_to_graveyard_with_triggers(gob);
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 1, "1 damage = Fireleaper's power");
}

/// Bitter Downfall destroys a creature and drains its controller.
#[test]
fn bitter_downfall_destroys_and_drains() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bitter_downfall());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(foe)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "creature destroyed");
    assert_eq!(g.players[1].life, life - 2, "controller lost 2 life");
}

/// Uruk-hai Berserker tempts on ETB.
#[test]
fn uruk_hai_berserker_tempts_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::uruk_hai_berserker());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert_eq!(g.players[0].ring_temptations, 1);
}

/// Ranger's Firebrand burns and tempts.
#[test]
fn rangers_firebrand_burns_and_tempts() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rangers_firebrand());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 damage to face");
    assert_eq!(g.players[0].ring_temptations, 1, "and tempted");
}





/// Andúril pumps and spawns two tapped flying Spirits on attack (CR 702.6e
/// equip-granted Attacks trigger).
#[test]
fn anduril_equips_and_spawns_spirits() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::anduril_flame_of_the_west());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::Equip { equipment: blade, target: bear }).expect("equip");
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (5, 3), "bear got +3/+1");
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit").count();
    assert_eq!(spirits, 2, "two Spirit tokens on attack");
}

/// Galadhrim Guide scries 2 on ETB (no crash; resolves).
#[test]
fn galadhrim_guide_scries_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::galadhrim_guide());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Galadhrim Guide"), "resolved");
}

/// Protector of Gondor makes a Soldier on ETB.
#[test]
fn protector_of_gondor_makes_soldier() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::protector_of_gondor());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Human Soldier").count();
    assert_eq!(soldiers, 1, "one Soldier token");
}

/// Shire Terrace taps for {C} and fetches a basic to the battlefield tapped.
#[test]
fn shire_terrace_fetches_basic() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::shire_terrace());
    let forest = g.add_card_to_library(0, catalog::forest());
    // Mana ability: tap for {C}.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("tap for C");
    assert_eq!(g.players[0].mana_pool.total(), 1, "added one colorless");
    // Untap so the sac-fetch can tap it.
    g.battlefield_find_mut(land).unwrap().tapped = false;
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac-fetch");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "Shire Terrace sacrificed");
    assert!(g.battlefield_find(forest).is_some(), "basic fetched to battlefield");
}

/// Eastfarthing Farmer makes a Food and pumps a creature per Food.
#[test]
fn eastfarthing_farmer_food_and_pump() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::eastfarthing_farmer());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food made");
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "+1/+1 for the one Food");
}

/// Grey Havens Navigator scries on ETB (resolves).
#[test]
fn grey_havens_navigator_resolves() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::grey_havens_navigator());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grey Havens Navigator"), "resolved");
}

/// Knights of Dol Amroth grows on your second draw each turn.
#[test]
fn knights_of_dol_amroth_grows_on_second_draw() {
    let mut g = two_player_game();
    let k = g.add_card_to_battlefield(0, catalog::knights_of_dol_amroth());
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].cards_drawn_this_turn = 0;
    let mut ev = vec![];
    g.draw_one(0, &mut ev); // first
    g.dispatch_triggers_for_events(&ev);
    drain_stack(&mut g);
    let mut ev2 = vec![];
    g.draw_one(0, &mut ev2); // second
    g.dispatch_triggers_for_events(&ev2);
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let kc = cp.iter().find(|c| c.id == k).unwrap();
    assert_eq!((kc.power, kc.toughness), (4, 4), "got a +1/+1 counter on the second draw");
}

/// Generous Ent makes a Food on ETB.
#[test]
fn generous_ent_makes_food() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::generous_ent());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    cast(&mut g, id);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food made");
}

/// Quickbeam pumps up to two creatures (and grants trample) when a Treefolk —
/// itself — enters.
#[test]
fn quickbeam_pumps_on_treefolk_enter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::quickbeam_upstart_ent());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast(&mut g, id);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "bear got +2/+2");
    assert!(b.keywords.contains(&crate::card::Keyword::Trample), "and trample");
}

/// Now for Wrath puts a counter on each creature, grants vigilance, and tempts.
#[test]
fn now_for_wrath_counters_and_tempts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::now_for_wrath_now_for_ruin());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, id);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "+1/+1 counter");
    assert!(b.keywords.contains(&crate::card::Keyword::Vigilance), "vigilance granted");
    assert_eq!(g.players[0].ring_temptations, 1, "tempted");
}

/// Shower of Arrows destroys a flying creature.
#[test]
fn shower_of_arrows_destroys_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flier
    let id = g.add_card_to_hand(0, catalog::shower_of_arrows());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, id, Target::Permanent(flyer));
    assert!(!g.battlefield.iter().any(|c| c.id == flyer), "flier destroyed");
}

/// Rising of the Day grants haste and pumps legendary creatures you control.
#[test]
fn rising_of_the_day_haste_and_legendary_anthem() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rising_of_the_day());
    let legend = g.add_card_to_battlefield(0, catalog::bilbo_retired_burglar()); // 1/3 legendary
    let cp = g.compute_battlefield();
    let l = cp.iter().find(|c| c.id == legend).unwrap();
    assert!(l.keywords.contains(&crate::card::Keyword::Haste), "haste granted");
    assert_eq!((l.power, l.toughness), (2, 3), "legendary +1/+0");
}

/// Frodo tempts you when a legendary creature you control enters.
#[test]
fn frodo_tempts_on_legendary_enter() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::frodo_baggins());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    cast(&mut g, id); // Frodo itself is legendary → tempts on its own ETB
    assert_eq!(g.players[0].ring_temptations, 1, "tempted on legendary enter");
}

/// Samwise mints a Food when a nontoken creature enters and can sac three to
/// recur a creature from the graveyard.
#[test]
fn samwise_food_and_recursion() {
    let mut g = two_player_game();
    let sam = g.add_card_to_battlefield(0, catalog::samwise_gamgee());
    // Three Foods to pay the activated cost; a creature in the graveyard to recur.
    for _ in 0..3 {
        g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    }
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let food_count = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Food").count();
    assert_eq!(food_count, 3, "three Foods present");
    g.perform_action(GameAction::ActivateAbility {
        card_id: sam, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Samwise");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "bear back in hand");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Food").count(), 0, "Foods sacrificed");
}

/// Mirror of Galadriel's scry-draw ability costs {1} less per legendary creature.
#[test]
fn mirror_of_galadriel_cost_reduction() {
    let mut g = two_player_game();
    let mirror = g.add_card_to_battlefield(0, catalog::mirror_of_galadriel());
    g.add_card_to_battlefield(0, catalog::bilbo_retired_burglar()); // 1 legendary creature
    g.add_card_to_library(0, catalog::forest());
    // {5} reduced by 1 = pay 4 generic.
    g.players[0].mana_pool.add_colorless(4);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: mirror, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate mirror at reduced cost");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew a card");
}

/// CR 701.54c — the level-1 Ring emblem makes the Ring-bearer legendary.
#[test]
fn ring_bearer_becomes_legendary() {
    use crate::card::Supertype;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // nonlegendary 2/2
    g.ring_tempts(0, &mut vec![]);
    assert_eq!(g.effective_ring_bearer(0), Some(bear));
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert!(b.supertypes.contains(&Supertype::Legendary), "Ring-bearer is legendary");
}

/// Olog-hai Crusher can't block without a Goblin/Orc, but can always attack.
#[test]
fn olog_hai_crusher_block_gate() {
    let mut g = two_player_game();
    // Defender (seat 1) has Olog-hai; attacker (seat 0) has a bear.
    let olog = g.add_card_to_battlefield(1, catalog::olog_hai_crusher());
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No Goblin/Orc → can't block.
    assert!(!g.blocker_can_block_attacker(olog, atk), "can't block without Goblin/Orc");
    // Give seat 1 a Goblin → now allowed to block.
    g.add_card_to_battlefield(1, catalog::goblin_guide());
    assert!(g.blocker_can_block_attacker(olog, atk), "can block with a Goblin");

    // The block-only gate never restricts attacking: the active player's own
    // Olog (no Goblin/Orc) can still be declared as an attacker.
    let attacker = g.add_card_to_battlefield(0, catalog::olog_hai_crusher());
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("block-only gate doesn't stop attacks");
}

/// Stew the Coneys: your creature deals power-damage to an enemy and you get a Food.
#[test]
fn stew_the_coneys_one_sided_fight() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::stew_the_coneys());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(foe)], mode: None, x_value: None,
    }).expect("cast Stew");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "2/2 took 3 and died");
    assert!(g.battlefield.iter().any(|c| c.id == mine), "our creature is unharmed (one-sided)");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food made");
}

/// Glóin makes a Treasure on a historic (legendary) cast, once per turn.
#[test]
fn gloin_treasure_on_historic_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gloin_dwarf_emissary());
    // Cast a legendary creature (historic).
    let bilbo = g.add_card_to_hand(0, catalog::bilbo_retired_burglar());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bilbo);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "Treasure made for the historic cast");
}

/// Improvised Club requires sacrificing an artifact/creature and deals 4.
#[test]
fn improvised_club_sac_and_burn() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::improvised_club());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Improvised Club");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, before - 4, "4 damage to face");
    assert!(!g.battlefield.iter().any(|c| c.id == fodder), "fodder sacrificed to additional cost");
}

/// Cast into the Fire mode 1 pings up to two creatures.
#[test]
fn cast_into_the_fire_pings_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::cast_into_the_fire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Mode 0 (damage); target both bears.
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: Some(0), x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().damage, 1, "bear a took 1");
    assert_eq!(g.battlefield_find(b).unwrap().damage, 1, "bear b took 1");
}

/// Dunland Crebain amasses Orcs 2 on ETB.
#[test]
fn dunland_crebain_amasses() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::dunland_crebain());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let army = g.compute_battlefield().into_iter()
        .find(|c| c.controller == 0 && c.subtypes.creature_types.contains(&CreatureType::Army));
    assert_eq!(army.map(|a| (a.power, a.toughness)), Some((2, 2)), "0/0 Army with two +1/+1");
}

/// Saruman's Trickery counters a spell and amasses.
#[test]
fn sarumans_trickery_counters_and_amasses() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    let id = g.add_card_to_hand(0, catalog::sarumans_trickery());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, id, Target::Permanent(bolt));
    assert!(g.players[1].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"), "bolt countered");
    assert!(g.compute_battlefield().iter().any(|c| c.controller == 0
        && c.subtypes.creature_types.contains(&CreatureType::Army)), "Army amassed");
}

/// Voracious Fell Beast edicts each opponent and makes a Food.
#[test]
fn voracious_fell_beast_edict_and_food() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::voracious_fell_beast());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "opponent sacrificed its creature");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food made");
}

/// Meriadoc makes a Food when a Halfling attacks.
#[test]
fn meriadoc_food_on_halfling_attack() {
    let mut g = two_player_game();
    let merry = g.add_card_to_battlefield(0, catalog::meriadoc_brandybuck());
    g.clear_sickness(merry);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: merry, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food on Halfling attack");
}

/// Hobbit's Sting deals damage equal to creatures + Foods you control.
#[test]
fn hobbits_sting_scales_with_board() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 1 creature
    g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token()); // 1 Food
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let id = g.add_card_to_hand(0, catalog::hobbits_sting());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(foe));
    // 1 creature + 1 Food = 2 damage; the 3/3 survives with 2 marked.
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "X = creatures + Foods");
}

/// Nazgûl tempts on ETB and grows every Wraith when the Ring tempts.
#[test]
fn nazgul_grows_wraiths_on_temptation() {
    let mut g = two_player_game();
    let n1 = g.add_card_to_battlefield(0, catalog::nazgul());
    let id = g.add_card_to_hand(0, catalog::nazgul());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id); // second Nazgûl ETB tempts → +1/+1 on each Wraith
    let cp = g.compute_battlefield();
    let first = cp.iter().find(|c| c.id == n1).unwrap();
    assert!(first.power >= 2, "the existing Wraith grew from the temptation trigger");
}

/// Stern Marshal pumps a target +2/+2.
#[test]
fn stern_marshal_pumps() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let marshal = g.add_card_to_battlefield(0, catalog::stern_marshal());
    g.clear_sickness(marshal);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: marshal, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == bear).map(|c| (c.power, c.toughness)), Some((4, 4)), "+2/+2");
}

/// Quarrel's End: discard a card, draw two, make a Soldier token.
#[test]
fn quarrels_end_discard_draw_token() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest()); // discard fodder
    for _ in 0..2 { g.add_card_to_library(0, catalog::forest()); }
    let id = g.add_card_to_hand(0, catalog::quarrels_end());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    // -1 spell, -1 discard, +2 draw = net +0 vs before-cast hand (which counted the spell).
    assert_eq!(g.players[0].hand.len(), hand_before - 2 + 2, "discard 1, draw 2");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human Soldier"),
        "Soldier token made");
}

/// Gandalf's Sanction scales with instants/sorceries in your graveyard.
#[test]
fn gandalfs_sanction_scales_with_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    g.add_card_to_graveyard(0, catalog::lightning_bolt()); // instant
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let id = g.add_card_to_hand(0, catalog::gandalfs_sanction());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(foe));
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "2 I/S in gy → 2 damage");
}

/// Shelob's Ambush grants +1/+2 and deathtouch and makes a Food.
#[test]
fn shelobs_ambush_pumps_and_food() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::shelobs_ambush());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 4), "+1/+2");
    assert!(b.keywords.contains(&crate::card::Keyword::Deathtouch), "deathtouch granted");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"), "Food made");
}

/// Soothing of Sméagol bounces a creature and tempts.
#[test]
fn soothing_of_smeagol_bounces_and_tempts() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::soothing_of_smeagol());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast_at(&mut g, id, Target::Permanent(foe));
    assert!(!g.battlefield.iter().any(|c| c.id == foe), "creature bounced");
    assert!(g.players[1].hand.iter().any(|c| c.definition.name == "Grizzly Bears"), "to owner's hand");
    assert_eq!(g.players[0].ring_temptations, 1, "tempted");
}

/// Mushroom Watchdogs sacrifices a Food to grow and gain vigilance.
#[test]
fn mushroom_watchdogs_sac_food_grows() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let dog = g.add_card_to_battlefield(0, catalog::mushroom_watchdogs());
    g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    g.perform_action(GameAction::ActivateAbility {
        card_id: dog, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let d = cp.iter().find(|c| c.id == dog).unwrap();
    assert_eq!((d.power, d.toughness), (3, 3), "+1/+1 counter");
    assert!(d.keywords.contains(&crate::card::Keyword::Vigilance), "vigilance granted");
}

/// Gollum's Bite shrinks a creature, and its graveyard ability tempts.
#[test]
fn gollums_bite_shrinks_then_gy_tempts() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let id = g.add_card_to_hand(0, catalog::gollums_bite());
    g.players[0].mana_pool.add(Color::Black, 1);
    cast_at(&mut g, id, Target::Permanent(foe));
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == foe).map(|c| (c.power, c.toughness)), Some((1, 1)), "-2/-2");
    // Now activate the graveyard ability.
    g.step = crate::game::TurnStep::PreCombatMain;
    let gy_id = g.players[0].graveyard.iter().find(|c| c.definition.name == "Gollum's Bite").unwrap().id;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: gy_id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("gy activate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].ring_temptations, 1, "graveyard ability tempts");
}

/// Lembas draws on ETB and can be sacrificed for life.
#[test]
fn lembas_etb_draw_and_sac_life() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::lembas());
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1, "spent Lembas, drew a card");
    let lem = g.battlefield.iter().find(|c| c.definition.name == "Lembas").unwrap().id;
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lem, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
}
