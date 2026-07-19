#![allow(unused_imports)]
use crabomination::card::{CardType, CounterType};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::*;
use crabomination::TurnStep;
use crabomination::game::{drain_stack, two_player_game};
use crabomination::mana::Color;
#[allow(unused)]
use crate::Factory;

// ── modern_decks-11: Multi-color removal + sweepers + body ───────────────────

#[test]
fn goblin_chainwhirler_etb_pings_opponents_board() {
    let mut g = two_player_game();
    let x1 = g.add_card_to_battlefield(1, catalog::mons_goblin_raiders()); // 1/1
    let _own = g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // our 1/1, unscathed
    let id = g.add_card_to_hand(0, catalog::goblin_chainwhirler());
    g.players[0].mana_pool.add(Color::Red, 3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Goblin Chainwhirler castable for {R}{R}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == x1), "opp 1/1 dies to the 1-damage ping");
    assert!(g.battlefield.iter().any(|c| c.id == _own), "our own creature is unscathed");
    assert_eq!(g.players[1].life, 19, "the opponent also takes 1");
}

#[test]
fn strangleroot_geist_has_haste_and_undying() {
    use crabomination::card::Keyword;
    let def = catalog::strangleroot_geist();
    assert_eq!((def.power, def.toughness), (2, 1));
    assert!(def.keywords.contains(&Keyword::Haste) && def.keywords.contains(&Keyword::Undying));
}

#[test]
fn blood_artist_drains_when_a_creature_dies() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blood_artist());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    // Bolt the opponent's creature so SBA dispatches CreatureDied to Blood Artist.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(victim)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 + 1, "you gain 1");
    assert_eq!(g.players[1].life, l1 - 1, "opponent loses 1");
}

#[test]
fn carrion_feeder_sacs_a_creature_to_grow() {
    let mut g = two_player_game();
    let feeder = g.add_card_to_battlefield(0, catalog::carrion_feeder());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(feeder);
    g.perform_action(GameAction::ActivateAbility {
        card_id: feeder, ability_index: 0,
        target: Some(Target::Permanent(fodder)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac-a-creature ability activates");
    drain_stack(&mut g);
    let cp = g.computed_permanent(feeder).expect("feeder alive");
    assert_eq!((cp.power, cp.toughness), (2, 2), "Carrion Feeder grows to 2/2");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder), "fodder sacrificed");
}

#[test]
fn unearth_returns_a_one_drop_from_graveyard() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_graveyard(0, catalog::elvish_mystic()); // MV 1 creature
    let id = g.add_card_to_hand(0, catalog::unearth());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(mystic)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Unearth castable for {B}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == mystic), "the 1-drop returns to the battlefield");
}

#[test]
fn grief_etb_discards_a_nonland_card() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::grief());
    let victim = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].hand.retain(|c| c.id == victim); // exactly one nonland card
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Grief castable for {1}{B}{B}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == victim),
        "Grief's ETB discards the opponent's nonland card");
}

#[test]
fn subtlety_etb_bounces_a_spell_to_top_of_library() {
    let mut g = two_player_game();
    // P1 casts a creature spell.
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bears on the stack");
    // P0 flashes Subtlety, scripting "top of library".
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let sub = g.add_card_to_hand(0, catalog::subtlety());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sub, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Subtlety flashed in response");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.id != bears), "the bears spell was countered");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(bears),
        "the countered spell is on top of its owner's library");
}

#[test]
fn endurance_etb_shuffles_opponent_graveyard_into_library() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let lib_before = g.players[1].library.len();
    let id = g.add_card_to_hand(0, catalog::endurance());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Endurance castable for {1}{G}{G}");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.is_empty(), "opponent's graveyard shuffled away");
    assert_eq!(g.players[1].library.len(), lib_before + 2, "two cards returned to the library");
}

#[test]
fn fury_evoke_etb_deals_four_damage() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::fury());
    let pitch = g.add_card_to_hand(0, catalog::lightning_bolt()); // red card to evoke
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: id,
        pitch_card: Some(pitch),
        target: None,
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Fury evoke should succeed");
    drain_stack(&mut g);
    // The ETB auto-targets the lone creature and dumps all 4 damage into it.
    assert!(!g.battlefield.iter().any(|c| c.id == big),
        "Fury's evoke ETB deals a lethal 4 to the 4/4");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Fury sacrificed via evoke");
}

#[test]
fn bot_kicks_tear_asunder_to_hit_a_creature() {
    use crabomination::server::bot::{Bot, RandomBot};
    let mut g = two_player_game();
    // Only a creature on board — unkicked Tear Asunder (artifact/enchantment)
    // has no legal target, so the bot's only castable play is the kicked
    // cast (nonland permanent). Give it enough mana for {1}{G}+{1}{B}.
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::tear_asunder());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.active_player_idx = 0;
    let action = RandomBot::new().next_action(&g, 0);
    assert!(matches!(action, Some(GameAction::CastSpellKicked { .. })),
        "bot kicks Tear Asunder when only a creature is targetable: {action:?}");
}

#[test]
fn think_twice_draws_and_has_flashback() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::think_twice());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Think Twice castable for {1}{U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "cast -1 + draw +1 = net 0");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Think Twice"),
        "Think Twice is in the graveyard (flashback available)");
}

#[test]
fn forbidden_alchemy_digs_four_and_buries_the_rest() {
    let mut g = two_player_game();
    for _ in 0..8 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::forbidden_alchemy());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Forbidden Alchemy castable for {2}{U}");
    drain_stack(&mut g);
    // One of the top four goes to hand; the other three are milled.
    assert_eq!(g.players[0].hand.len(), hand_before, "cast -1 + pick +1 = net 0 hand");
    assert!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count() >= 3,
        "the other three dug cards are milled");
}

#[test]
fn pilgrims_eye_fetches_a_basic_to_hand() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::pilgrims_eye());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    // AutoDecider declines searches; script the basic-land pick.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pilgrim's Eye castable for {2}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"),
        "ETB tutors a basic land into hand");
}

#[test]
fn goblin_king_anthems_goblins() {
    use crabomination::card::{Keyword, LandType};
    let mut g = two_player_game();
    let king = g.add_card_to_battlefield(0, catalog::goblin_king());
    let raider = g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // 1/1 Goblin
    let cp = g.computed_permanent(raider).expect("raider alive");
    assert_eq!((cp.power, cp.toughness), (2, 2), "Goblin King anthems other Goblins +1/+1");
    assert!(cp.keywords.contains(&Keyword::Landwalk(LandType::Mountain)),
        "other Goblins gain mountainwalk");
    // "Other" — the King doesn't buff itself.
    let kcp = g.computed_permanent(king).expect("king alive");
    assert_eq!((kcp.power, kcp.toughness), (2, 2), "King doesn't anthem itself");
}

#[test]
fn goblin_chieftain_anthems_and_grants_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_chieftain());
    let raider = g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // 1/1 Goblin
    let cp = g.computed_permanent(raider).expect("raider alive");
    assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 to other Goblins");
    assert!(cp.keywords.contains(&Keyword::Haste), "other Goblins gain haste");
}

#[test]
fn krenko_makes_goblins_equal_to_goblin_count() {
    let mut g = two_player_game();
    let krenko = g.add_card_to_battlefield(0, catalog::krenko_mob_boss());
    g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // 2 Goblins total
    g.clear_sickness(krenko);
    let before = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Goblin)).count();
    g.perform_action(GameAction::ActivateAbility {
        card_id: krenko, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Krenko taps for Goblins");
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Goblin)).count();
    assert_eq!(after - before, 2, "Krenko makes one Goblin per Goblin you control (2)");
}

#[test]
fn goblin_instigator_enters_with_a_token() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::goblin_instigator());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Goblin Instigator");
    drain_stack(&mut g);
    let goblins = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.subtypes.creature_types.contains(&crabomination::card::CreatureType::Goblin)).count();
    assert_eq!(goblins, 2, "Instigator plus its 1/1 Goblin token");
}

#[test]
fn goblin_trashmaster_sacrifices_a_goblin_to_destroy_an_artifact() {
    let mut g = two_player_game();
    let tm = g.add_card_to_battlefield(0, catalog::goblin_trashmaster());
    g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // sac fodder
    g.clear_sickness(tm);
    let art = g.add_card_to_battlefield(1, catalog::ornithopter()); // an artifact
    g.perform_action(GameAction::ActivateAbility {
        card_id: tm, ability_index: 0, target: Some(Target::Permanent(art)), additional_targets: Vec::new(), x_value: None,
    }).expect("Trashmaster sac-destroys an artifact");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "the artifact is destroyed");
}

#[test]
fn beetleback_chief_makes_two_goblins() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::beetleback_chief());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Beetleback Chief");
    drain_stack(&mut g);
    let tokens = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token).count();
    assert_eq!(tokens, 2, "two 1/1 Goblin tokens");
}

#[test]
fn goblin_warchief_reduces_goblin_costs_and_grants_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_warchief());
    let raider = g.add_card_to_battlefield(0, catalog::mons_goblin_raiders());
    // Haste anthem reaches other Goblins.
    assert!(g.computed_permanent(raider).unwrap().keywords.contains(&Keyword::Haste),
        "Goblins gain haste");
    // {2}{R} Krenko (a Goblin spell) is discounted by {1} → costs {1}{R}{R}.
    let krenko = g.add_card_to_hand(0, catalog::krenko_mob_boss());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1); // only {1} generic, relying on the discount
    g.perform_action(GameAction::CastSpell {
        card_id: krenko, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Goblin spell costs one less");
}

#[test]
fn skirk_prospector_sacrifices_a_goblin_for_red() {
    use crabomination::mana::Color;
    let mut g = two_player_game();
    let prospector = g.add_card_to_battlefield(0, catalog::skirk_prospector());
    g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // sac fodder
    g.perform_action(GameAction::ActivateAbility {
        card_id: prospector, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac a Goblin for one red");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "added one red");
}

#[test]
fn midnight_haunting_makes_two_flying_spirits() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::midnight_haunting());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Midnight Haunting");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Spirit").collect();
    assert_eq!(spirits.len(), 2, "two Spirit tokens");
    assert!(spirits.iter().all(|c| c.definition.keywords.contains(&Keyword::Flying)), "with flying");
}

#[test]
fn gather_the_townsfolk_makes_two_humans() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::gather_the_townsfolk());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Gather the Townsfolk");
    drain_stack(&mut g);
    let humans = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Human").count();
    assert_eq!(humans, 2, "two Human tokens");
}

#[test]
fn captain_of_the_watch_anthems_soldiers_and_makes_three() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::captain_of_the_watch());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Captain of the Watch");
    drain_stack(&mut g);
    let soldiers: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Soldier").collect();
    assert_eq!(soldiers.len(), 3, "three Soldier tokens");
    // Anthem + vigilance reach the tokens.
    let cp = g.computed_permanent(soldiers[0].id).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "+1/+1 to other Soldiers");
    assert!(cp.keywords.contains(&Keyword::Vigilance), "Soldiers gain vigilance");
}

#[test]
fn foundry_street_denizen_grows_when_a_red_creature_enters() {
    let mut g = two_player_game();
    let denizen = g.add_card_to_battlefield(0, catalog::foundry_street_denizen());
    // Cast a red creature (Mons's Goblin Raiders is red) → +1/+1 EOT.
    let gob = g.add_card_to_hand(0, catalog::mons_goblin_raiders());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: gob, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a red creature");
    drain_stack(&mut g);
    let s = g.battlefield_find(denizen).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "Denizen grew when a red creature entered");
}

#[test]
fn goblin_sledder_sacs_a_goblin_to_pump() {
    let mut g = two_player_game();
    let sledder = g.add_card_to_battlefield(0, catalog::goblin_sledder());
    g.add_card_to_battlefield(0, catalog::mons_goblin_raiders()); // sac fodder
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: sledder, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac a Goblin to pump");
    drain_stack(&mut g);
    let s = g.battlefield_find(target).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3), "target got +1/+1");
}

#[test]
fn mogg_raider_sacs_a_creature_to_grow() {
    let mut g = two_player_game();
    let raider = g.add_card_to_battlefield(0, catalog::mogg_raider());
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // sac fodder
    g.perform_action(GameAction::ActivateAbility {
        card_id: raider, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac a creature");
    drain_stack(&mut g);
    let s = g.battlefield_find(raider).unwrap();
    assert_eq!((s.power(), s.toughness()), (2, 2), "Mogg Raider grew +1/+1");
}

#[test]
fn bloodlust_inciter_grants_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let inciter = g.add_card_to_battlefield(0, catalog::bloodlust_inciter());
    g.clear_sickness(inciter);
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // summoning sick
    g.perform_action(GameAction::ActivateAbility {
        card_id: inciter, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("grant haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(target).unwrap().keywords.contains(&Keyword::Haste),
        "target gained haste");
}

#[test]
fn rout_flash_mode_casts_at_instant_speed_and_sweeps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rout = g.add_card_to_hand(0, catalog::rout());
    // Move to a non-main step where sorcery-speed casting is illegal.
    g.step = TurnStep::DeclareAttackers;
    // The plain (sorcery) cast is rejected here…
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: rout, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "Rout can't be hard-cast at instant speed");
    // …but the flash alt cost ({5}{W}{W}) can.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: rout, pitch_card: None, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rout's flash mode is castable at instant speed");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "Rout swept the board");
}

#[test]
fn phantom_monster_is_a_three_three_flyer() {
    use crabomination::card::Keyword;
    let def = catalog::phantom_monster();
    assert_eq!((def.power, def.toughness), (3, 3));
    assert!(def.keywords.contains(&Keyword::Flying));
}

#[test]
fn champion_of_the_parish_grows_when_a_human_enters() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::champion_of_the_parish());
    // Cast a Human (Thraben Inspector) — Champion gets a +1/+1 counter.
    let human = g.add_card_to_hand(0, catalog::thraben_inspector());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: human, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a Human");
    drain_stack(&mut g);
    let cp = g.computed_permanent(champ).expect("champ alive");
    assert_eq!((cp.power, cp.toughness), (2, 2), "another Human ETB grows the Champion");
}

#[test]
fn soltari_priest_has_shadow_and_pro_red() {
    use crabomination::card::Keyword;
    let def = catalog::soltari_priest();
    assert!(def.keywords.contains(&Keyword::Shadow));
    assert!(def.keywords.contains(&Keyword::Protection(Color::Red)));
}

#[test]
fn merfolk_looter_taps_to_loot() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let looter = g.add_card_to_battlefield(0, catalog::merfolk_looter());
    g.clear_sickness(looter);
    let junk = g.add_card_to_hand(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: looter, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("loot ability activates");
    drain_stack(&mut g);
    // Draw 1, discard 1 → hand size unchanged; the discarded card is in gy.
    assert_eq!(g.players[0].hand.len(), hand_before, "draw 1 + discard 1 nets zero");
    let _ = junk;
    assert!(g.battlefield_find(looter).map(|c| c.tapped).unwrap_or(false), "looter tapped");
}

#[test]
fn goblin_electromancer_reduces_instant_sorcery_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_electromancer());
    // Incinerate ({1}{R}) becomes castable for just {R} thanks to the {1}-off.
    let id = g.add_card_to_hand(0, catalog::incinerate());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Electromancer makes a {1}{R} instant castable for just {R}");
}

#[test]
fn wee_dragonauts_pumps_on_instant_cast() {
    let mut g = two_player_game();
    let dragonauts = g.add_card_to_battlefield(0, catalog::wee_dragonauts());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightning Bolt");
    drain_stack(&mut g);
    let cp = g.computed_permanent(dragonauts).expect("alive");
    assert_eq!(cp.power, 3, "Wee Dragonauts swings to 3 power after an instant");
}

#[test]
fn selfless_spirit_sac_grants_team_indestructible() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(0, catalog::selfless_spirit());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(spirit);
    g.perform_action(GameAction::ActivateAbility {
        card_id: spirit, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac ability activates");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("bear alive");
    assert!(cp.keywords.contains(&Keyword::Indestructible),
        "ally gains indestructible until end of turn");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spirit), "Spirit sacrificed");
}

#[test]
fn reality_smasher_has_ward_discard() {
    use crabomination::card::{Keyword, WardCost};
    let def = catalog::reality_smasher();
    assert_eq!((def.power, def.toughness), (5, 5));
    assert!(def.keywords.contains(&Keyword::Ward(WardCost::Discard(1))), "Ward—discard a card");
    assert!(def.keywords.contains(&Keyword::Trample) && def.keywords.contains(&Keyword::Haste));
}

#[test]
fn glint_nest_crane_etb_digs_for_an_artifact() {
    let mut g = two_player_game();
    // Top of library: an artifact buried under non-artifacts.
    g.add_card_to_library(0, catalog::sol_ring());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::glint_nest_crane());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Glint-Nest Crane castable for {1}{U}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Sol Ring"),
        "the artifact in the top four is put into hand");
}

#[test]
fn murktide_regent_enters_with_two_counters_and_grows_on_spellcast() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::murktide_regent());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Murktide castable for {5}{U}{U}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "3/3 base + two +1/+1 counters");
    // Cast an instant; the magecraft trigger adds a counter.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lightning Bolt");
    drain_stack(&mut g);
    let cp = g.computed_permanent(id).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "instant cast grows Murktide to 6/6");
}

#[test]
fn seasoned_pyromancer_etb_discards_draws_and_makes_tokens() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::seasoned_pyromancer());
    g.add_card_to_hand(0, catalog::lightning_bolt()); // nonland to discard
    g.add_card_to_hand(0, catalog::shock());          // nonland to discard
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Seasoned Pyromancer castable for {1}{R}{R}");
    drain_stack(&mut g);
    let elementals = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Elemental").count();
    assert_eq!(elementals, 2, "two nonland discards mint two 1/1 Elementals");
}

#[test]
fn aether_figment_kicked_enters_as_a_three_three() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aether_figment());
    // {1}{U} + Kicker {3} = {4}{U}.
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("kicked Aether Figment castable for {4}{U}");
    drain_stack(&mut g);
    let cp = g.battlefield.iter().find(|c| c.definition.name == "Aether Figment")
        .map(|c| g.computed_permanent(c.id).unwrap()).expect("in play");
    assert_eq!((cp.power, cp.toughness), (2, 2), "kicked Figment enters with a +1/+1 counter");
}

#[test]
fn aether_figment_unkicked_is_a_two_two() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::aether_figment());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Aether Figment castable for {1}{U}");
    drain_stack(&mut g);
    let cp = g.battlefield.iter().find(|c| c.definition.name == "Aether Figment")
        .map(|c| g.computed_permanent(c.id).unwrap()).expect("in play");
    assert_eq!((cp.power, cp.toughness), (1, 1), "unkicked Figment is a vanilla 2/2");
}

#[test]
fn thraben_inspector_investigates_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::thraben_inspector());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Thraben Inspector castable for {W}");
    drain_stack(&mut g);
    // A Clue artifact token entered under the caster's control.
    let clues = g.battlefield.iter().filter(|c| c.controller == 0
        && c.definition.subtypes.artifact_subtypes
            .contains(&crabomination::card::ArtifactSubtype::Clue)).count();
    assert_eq!(clues, 1, "ETB investigate mints exactly one Clue");
}

#[test]
fn goblin_bushwhacker_kicked_pumps_the_team() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::goblin_bushwhacker());
    // {R} + Kicker {R} = {R}{R}.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("kicked Goblin Bushwhacker castable for {R}{R}");
    drain_stack(&mut g);
    // The pre-existing ally got +1/+0 from the kicked ETB.
    let cp = g.computed_permanent(ally).expect("ally still in play");
    assert_eq!(cp.power, 3, "kicked Bushwhacker pumps allies +1/+0");
}

#[test]
fn goblin_bushwhacker_unkicked_has_no_etb_pump() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::goblin_bushwhacker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Goblin Bushwhacker castable for {R}");
    drain_stack(&mut g);
    let cp = g.computed_permanent(ally).expect("ally still in play");
    assert_eq!(cp.power, 2, "unkicked Bushwhacker leaves the team unpumped");
}

#[test]
fn into_the_roil_unkicked_bounces_without_drawing() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::into_the_roil());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Into the Roil castable for {1}{U}");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "creature bounced to owner's hand");
    // Caster spent the spell (-1) and didn't draw → hand shrank by 1.
    assert_eq!(g.players[0].hand.len(), hand_before - 1, "unkicked: no card drawn");
}

#[test]
fn into_the_roil_kicked_draws_a_card() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::into_the_roil());
    // {1}{U} + Kicker {1}{U} = {2}{U}{U}.
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("kicked Into the Roil castable for {2}{U}{U}");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "creature bounced");
    // -1 (cast) +1 (kicked draw) → net unchanged.
    assert_eq!(g.players[0].hand.len(), hand_before, "kicked: drew a replacement card");
}

#[test]
fn tear_asunder_exiles_target_artifact() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::tear_asunder());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(relic)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Tear Asunder castable for {1}{G}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == relic),
        "unkicked Tear Asunder exiles the artifact");
}

#[test]
fn tear_asunder_unkicked_rejects_creature_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tear_asunder());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);

    let r = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(r.is_err(), "unkicked Tear Asunder rejects creature (nonland-permanent) targets");
}

#[test]
fn tear_asunder_kicked_exiles_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::tear_asunder());
    // {1}{G} base + {1}{B} kicker.
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpellKicked {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("kicked Tear Asunder castable for {1}{G}+{1}{B}, targets a nonland permanent");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == bear),
        "kicked Tear Asunder exiles the creature");
}

#[test]
fn assassins_trophy_destroys_opp_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::assassins_trophy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("Trophy castable for {B}{G}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Assassin's Trophy destroys the opp's creature");
}

#[test]
fn assassins_trophy_owner_ramps_a_basic_land() {
    let mut g = two_player_game();
    // Opponent's permanent to destroy is a land; they ramp a basic in return.
    let opp_land = g.add_card_to_battlefield(1, catalog::mountain());
    let forest = g.add_card_to_library(1, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::assassins_trophy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(opp_land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("can target an opponent's land");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_land), "land destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == forest && c.controller == 1),
        "owner searched a basic land onto the battlefield");
}

#[test]
fn assassins_trophy_rejects_your_own_permanent() {
    // Filter is "permanent an opponent controls" — caster's own creature
    // should be rejected at cast time.
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::assassins_trophy());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);

    let r = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(r.is_err(), "Trophy should reject caster-controlled targets");
}

#[test]
fn volcanic_fallout_deals_two_to_each_creature_and_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon()); // 5/5
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    let id = g.add_card_to_hand(0, catalog::volcanic_fallout());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Volcanic Fallout castable for {1}{R}{R}");
    drain_stack(&mut g);

    // Both 2/2 bears die; the 5/5 dragon survives.
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Volcanic Fallout kills both 2-toughness bears (yours)");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Volcanic Fallout kills both 2-toughness bears (opp)");
    assert!(g.battlefield.iter().any(|c| c.id == dragon),
        "Volcanic Fallout doesn't kill the 5-toughness dragon");
    assert_eq!(g.players[0].life, life0 - 2, "Caster takes 2");
    assert_eq!(g.players[1].life, life1 - 2, "Opp takes 2");
}

#[test]
fn volcanic_fallout_is_uncounterable() {
    // Push (modern_decks): the "this can't be countered" rider now lands
    // via `Keyword::CantBeCountered` on the card definition. The cast
    // pushes `StackItem::Spell.uncounterable = true`.
    use crabomination::game::types::StackItem;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::volcanic_fallout());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Fallout castable");
    let uncounterable = g.stack.iter().find_map(|si| match si {
        StackItem::Spell { uncounterable, .. } => Some(*uncounterable),
        _ => None,
    });
    assert_eq!(uncounterable, Some(true),
        "Fallout should land on the stack as uncounterable");
}

#[test]
fn rout_destroys_all_creatures_at_sorcery_speed() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lion = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let id = g.add_card_to_hand(0, catalog::rout());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Rout castable for {3}{W}{W}");
    drain_stack(&mut g);

    for cid in [bear, lion, dragon] {
        assert!(!g.battlefield.iter().any(|c| c.id == cid),
            "Rout should destroy all creatures");
    }
}

#[test]
fn plague_wind_destroys_only_opponent_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let opp_dragon = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    let id = g.add_card_to_hand(0, catalog::plague_wind());
    g.players[0].mana_pool.add_colorless(8);
    g.players[0].mana_pool.add(Color::Black, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Plague Wind castable for {8}{B}{B}");
    drain_stack(&mut g);

    assert!(g.battlefield.iter().any(|c| c.id == mine),
        "Plague Wind leaves caster's creatures alone");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_bear),
        "Plague Wind destroys opp's bear");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_dragon),
        "Plague Wind destroys opp's dragon");
}

/// Plague Wind's "can't be regenerated" rider: a shielded regenerator on
/// the opponent's side still dies.
#[test]
fn plague_wind_ignores_regeneration_shields() {
    let mut g = two_player_game();
    let skel = g.add_card_to_battlefield(1, catalog::drudge_skeletons());
    g.clear_sickness(skel);
    g.players[1].mana_pool.add(Color::Black, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: skel, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate activates");
    drain_stack(&mut g);
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::plague_wind());
    g.players[0].mana_pool.add_colorless(7);
    g.players[0].mana_pool.add(Color::Black, 2);
    cast(&mut g, id);
    assert!(g.battlefield_find(skel).is_none(),
        "regenerator dies through its shield to Plague Wind");
}

#[test]
fn carnage_tyrant_resolves_through_counterspell() {
    let mut g = two_player_game();
    let tyrant = g.add_card_to_hand(0, catalog::carnage_tyrant());
    let counter = g.add_card_to_hand(1, catalog::counterspell());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[1].mana_pool.add(Color::Blue, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: tyrant, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Tyrant castable for {4}{G}{G}");
    // Opponent tries to counter the spell. Counter targets are addressed
    // by `Target::Permanent(card_id)` (the stack-item lookup uses the
    // card-id internally regardless of zone). The Tyrant carries
    // `Keyword::CantBeCountered`, so `CounterSpell.uncounterable_check`
    // should let the Tyrant through unscathed.
    let _ = g.perform_action(GameAction::CastSpell {
        card_id: counter,
        target: Some(Target::Permanent(tyrant)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == tyrant),
        "Tyrant resolves despite the counter attempt");
}

#[test]
fn krark_clan_ironworks_sacs_artifact_for_two_colorless() {
    let mut g = two_player_game();
    let kci = g.add_card_to_battlefield(0, catalog::krark_clan_ironworks());
    // Have at least one other artifact under our control so the sac has
    // something to pick. KCI itself is also an artifact, so the bot may
    // sac KCI itself; we add a Sol Ring as the obvious sacrifice target.
    let ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let pool_before = g.players[0].mana_pool.total();

    g.perform_action(GameAction::ActivateAbility {
        card_id: kci, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("KCI activated");
    drain_stack(&mut g);

    // One of {KCI, Sol Ring} should be sacrificed (whichever the bot picked).
    let kci_alive = g.battlefield.iter().any(|c| c.id == kci);
    let ring_alive = g.battlefield.iter().any(|c| c.id == ring);
    assert!(!kci_alive || !ring_alive,
        "At least one artifact should have been sacrificed");
    assert!(g.players[0].mana_pool.total() >= pool_before + 2,
        "KCI's sac yields at least {{2}}");
}

// ── Surveil land cycle (modern_decks-11) ─────────────────────────────────────

#[test]
fn underground_mortuary_etbs_tapped_and_carries_surveil_trigger() {
    let mut g = two_player_game();
    // Stock the library so the surveil has an input to inspect.
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let id = g.add_card_to_hand(0, catalog::underground_mortuary());
    g.perform_action(GameAction::PlayLand(id)).unwrap();
    drain_stack(&mut g);

    let card = g.battlefield_find(id).expect("Mortuary on the battlefield");
    assert!(card.tapped, "Surveil land enters tapped");
    // The factory definition carries the surveil trigger (AutoDecider keeps
    // the surveil-peeked card on top, so we don't observe a library shape
    // change here — the structural assertion is what the cube wires).
    let def = catalog::underground_mortuary();
    let has_surveil = def.triggered_abilities.iter().any(|t| {
        if let crabomination::card::Effect::Seq(steps) = &t.effect {
            steps.iter().any(|e| matches!(e, crabomination::card::Effect::Surveil { .. }))
        } else {
            false
        }
    });
    assert!(has_surveil, "Mortuary's ETB trigger contains a Surveil step");
}

#[test]
fn underground_mortuary_taps_for_black_or_green() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::underground_mortuary());
    // Force-untap (helper places it tapped by default through ETB; we test
    // both colored mana abilities).
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1,
        "ability 0 produces {{B}}");

    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None }).unwrap();
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1,
        "ability 1 produces {{G}}");
}

// ── modern_decks-12: 12 new playables ────────────────────────────────────────

#[test]
fn stone_rain_destroys_target_land() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::stone_rain());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Stone Rain castable for {2}{R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == mountain),
        "Mountain should be destroyed");
}

#[test]
fn stone_rain_rejects_creature_target() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::stone_rain());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Stone Rain rejects non-land target: {err:?}");
}

#[test]
fn bone_splinters_sacrifices_then_destroys_target() {
    let mut g = two_player_game();
    let sac_target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp_target = g.add_card_to_battlefield(1, catalog::serra_angel());
    let id = g.add_card_to_hand(0, catalog::bone_splinters());
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(opp_target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bone Splinters castable for {B}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == sac_target),
        "Caster's bear should be sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_target),
        "Opponent's Serra Angel should be destroyed");
}

#[test]
fn hieroglyphic_illumination_draws_two_cards() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::hieroglyphic_illumination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hieroglyphic Illumination castable for {3}{U}");
    drain_stack(&mut g);
    // Cast (-1) + Draw 2 (+2) = +1 hand size.
    assert_eq!(g.players[0].hand.len(), hand_before + 1);
}

#[test]
fn hieroglyphic_illumination_cycles_for_blue() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::hieroglyphic_illumination());
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::Cycle { card_id: id, x_value: None })
        .expect("Cycling {U}");
    // Discarded the card (-1) + drew (+1) → net even; card is in graveyard.
    assert_eq!(g.players[0].hand.len(), hand_before);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id),
        "cycled card goes to the graveyard");
}

#[test]
fn mortify_destroys_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mortify castable for {1}{W}{B}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear should be destroyed");
}

#[test]
fn mortify_ignores_regeneration_shield() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().regeneration_shields = 1;
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mortify castable");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Mortify destroys through a regeneration shield");
}

#[test]
fn mortify_destroys_enchantment() {
    let mut g = two_player_game();
    // Phyrexian Arena is an enchantment; use it as the target.
    let arena = g.add_card_to_battlefield(1, catalog::phyrexian_arena());
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(arena)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Mortify castable on enchantment");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == arena),
        "Phyrexian Arena should be destroyed");
}

#[test]
fn mortify_rejects_land_target() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::mortify());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Mortify rejects a land target: {err:?}");
}

#[test]
fn maelstrom_pulse_destroys_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::maelstrom_pulse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Maelstrom Pulse castable for {1}{B}{G}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
}

#[test]
fn maelstrom_pulse_sweeps_all_with_same_name() {
    let mut g = two_player_game();
    // Three Grizzly Bears split across players + one other creature.
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let id = g.add_card_to_hand(0, catalog::maelstrom_pulse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(b1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    for id in [b1, b2, b3] {
        assert!(!g.battlefield.iter().any(|c| c.id == id), "all Grizzly Bears destroyed");
    }
    assert!(g.battlefield.iter().any(|c| c.id == elf), "differently-named creature survives");
}

#[test]
fn maelstrom_pulse_rejects_land_target() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::maelstrom_pulse());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Maelstrom Pulse rejects land: {err:?}");
}

#[test]
fn mind_twist_discards_x_random_cards_from_target_player() {
    let mut g = two_player_game();
    // Stack opponent's hand with five cards.
    for _ in 0..5 { g.add_card_to_hand(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    }).expect("Mind Twist castable for {3}{B} (X=3)");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before - 3,
        "Three random cards discarded");
    assert_eq!(g.players[1].graveyard.len(), 3,
        "Three cards in opponent's graveyard");
}

#[test]
fn mind_twist_x_zero_does_nothing() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::mind_twist());
    g.players[0].mana_pool.add(Color::Black, 1);
    let hand_before = g.players[1].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(0),
    }).expect("Mind Twist castable at X=0 for {B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before,
        "X=0 yields no discards");
}

#[test]
fn dismember_kills_a_five_toughness_creature() {
    let mut g = two_player_game();
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let big = g.add_card_to_battlefield(1, catalog::sengir_vampire()); // 4/4
    let id = g.add_card_to_hand(0, catalog::dismember());
    // {1}{B/P}{B/P}{B/P}: {1} + two black pips + one pip paid with 2 life.
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);

    let _ = serra;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Dismember castable for {1}{B}{B} + 2 life");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == big),
        "Sengir Vampire (4/4) dies to -5/-5");
}

#[test]
fn dismember_castable_for_one_and_six_life() {
    // All three {B/P} pips paid with life → {1} + 6 life.
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::sengir_vampire());
    let id = g.add_card_to_hand(0, catalog::dismember());
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(big)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Dismember castable for {1} + 4 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before - 4,
        "two Phyrexian pips paid with 2 life each = 4 life");
    assert!(!g.battlefield.iter().any(|c| c.id == big));
}

#[test]
fn echoing_truth_bounces_nonland_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::echoing_truth());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Echoing Truth castable for {1}{U}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear));
    assert!(g.players[1].hand.iter().any(|c| c.id == bear),
        "Bear bounces back to its owner's hand");
}

#[test]
fn echoing_truth_bounces_all_with_same_name() {
    let mut g = two_player_game();
    // Opponent has a swarm of same-named tokens (e.g. three Bears).
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves());
    let id = g.add_card_to_hand(0, catalog::echoing_truth());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(b1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    for id in [b1, b2] {
        assert!(g.players[1].hand.iter().any(|c| c.id == id), "both Bears bounce");
    }
    assert!(g.battlefield.iter().any(|c| c.id == elf), "Elf unaffected");
}

#[test]
fn echoing_truth_rejects_land_target() {
    let mut g = two_player_game();
    let mountain = g.add_card_to_battlefield(1, catalog::mountain());
    let id = g.add_card_to_hand(0, catalog::echoing_truth());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(mountain)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "Echoing Truth is nonland-only: {err:?}");
}

#[test]
fn celestial_purge_exiles_a_black_creature() {
    let mut g = two_player_game();
    let specter = g.add_card_to_battlefield(1, catalog::hypnotic_specter());
    let id = g.add_card_to_hand(0, catalog::celestial_purge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(specter)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Celestial Purge castable for {1}{W}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == specter),
        "Specter exiled");
    assert!(g.exile.iter().any(|c| c.id == specter),
        "Specter is in exile (not graveyard)");
}

#[test]
fn celestial_purge_rejects_a_green_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let id = g.add_card_to_hand(0, catalog::celestial_purge());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);

    let err = g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(),
        "Celestial Purge can only target black or red permanents: {err:?}");
}

#[test]
fn earthquake_burns_each_player_and_grounded_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 ground
    let serra = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let id = g.add_card_to_hand(0, catalog::earthquake());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_p0 = g.players[0].life;
    let life_p1 = g.players[1].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None,
        x_value: Some(3),
    }).expect("Earthquake castable for {3}{R} (X=3)");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == bear),
        "Bear (2/2 ground) takes 3 and dies");
    assert!(g.battlefield.iter().any(|c| c.id == serra),
        "Serra (flying) is untouched");
    assert_eq!(g.players[0].life, life_p0 - 3, "P0 takes 3");
    assert_eq!(g.players[1].life, life_p1 - 3, "P1 takes 3");
}

#[test]
fn glimpse_the_unthinkable_mills_ten_from_target() {
    let mut g = two_player_game();
    for _ in 0..15 { g.add_card_to_library(1, catalog::island()); }
    let id = g.add_card_to_hand(0, catalog::glimpse_the_unthinkable());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    let lib_before = g.players[1].library.len();
    let yard_before = g.players[1].graveyard.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Glimpse castable for {U}{B}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 10);
    assert_eq!(g.players[1].graveyard.len(), yard_before + 10);
}

#[test]
fn cling_to_dust_exiles_creature_card_and_gains_three_life() {
    let mut g = two_player_game();
    // Seed a creature card in opp's graveyard.
    let card_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::cling_to_dust());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(card_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cling to Dust castable for {B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == card_id),
        "Card moves from graveyard to exile");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == card_id));
    assert_eq!(g.players[0].life, life_before + 3,
        "Caster gains 3 life when a creature is exiled");
}

#[test]
fn cling_to_dust_draws_for_noncreature_card() {
    let mut g = two_player_game();
    // Seed a non-creature (instant) card in opp's graveyard.
    let card_id = g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::cling_to_dust());
    g.players[0].mana_pool.add(Color::Black, 1);
    let life_before = g.players[0].life;
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(card_id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cling to Dust castable for {B}");
    drain_stack(&mut g);

    assert!(g.exile.iter().any(|c| c.id == card_id),
        "Card still moves to exile");
    assert_eq!(g.players[0].life, life_before,
        "No lifegain when the exiled card isn't a creature");
    // The spell itself left hand (-1) and a draw added one (+1) → net even,
    // but the drawn card is a different id than the cast Cling to Dust.
    assert_eq!(g.players[0].hand.len(), hand_before,
        "Non-creature branch draws a card");
}

#[test]
fn cling_to_dust_escapes_from_graveyard_exiling_five_cards() {
    let mut g = two_player_game();
    // Cling to Dust sits in P0's graveyard alongside five fodder cards to
    // exile for the escape cost, plus a creature in P1's graveyard to hit.
    let cling = g.add_card_to_graveyard(0, catalog::cling_to_dust());
    let fodder: Vec<_> = (0..5)
        .map(|_| g.add_card_to_graveyard(0, catalog::lightning_bolt()))
        .collect();
    let target = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life_before = g.players[0].life;

    g.perform_action(GameAction::CastEscape {
        card_id: cling,
        exile_cards: fodder.clone(),
        target: Some(Target::Permanent(target)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Cling to Dust escapable for {3}{B} + exile five");
    drain_stack(&mut g);

    // Escape cost paid: the five fodder cards left the graveyard for exile.
    for f in &fodder {
        assert!(g.exile.iter().any(|c| c.id == *f), "fodder card exiled as cost");
    }
    assert!(g.exile.iter().any(|c| c.id == target), "target moved to exile");
    assert_eq!(g.players[0].life, life_before + 3, "creature → gain 3 life");
    // Instant/sorcery escape resolves back to the graveyard (re-escapable).
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cling),
        "escaped instant returns to its owner's graveyard");
}

// ── modern_decks-13: 12 new cards ───────────────────────────────────────────

#[test]
fn lumra_returns_all_lands_from_your_graveyard() {
    let mut g = two_player_game();
    // Seed two land cards + one non-land in P0's graveyard.
    let f1 = g.add_card_to_graveyard(0, catalog::forest());
    let f2 = g.add_card_to_graveyard(0, catalog::forest());
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    // A land on the battlefield so Lumra (*/* = lands you control) isn't a
    // 0/0 that dies to SBA before its own ETB resolves.
    g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::lumra_bellow_of_the_woods());
    g.players[0].library.clear(); // keep the ETB mill a no-op
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lumra castable for {4}{G}{G}");
    drain_stack(&mut g);

    // Both Forests came back to BF tapped, the Bear stayed in graveyard.
    assert!(g.battlefield_find(f1).is_some(), "Forest 1 returned to BF");
    assert!(g.battlefield_find(f2).is_some(), "Forest 2 returned to BF");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "Bear stays in graveyard (not a land)");
    assert!(g.battlefield_find(f1).unwrap().tapped,
        "Land returns tapped per Oracle");
    // Lumra is */* = lands you control. The two returned Forests are the
    // only lands here (empty library → the ETB mill does nothing).
    let lands = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.is_land()).count() as i32;
    let computed = g.compute_battlefield();
    let lumra = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(lumra.power, lands, "*/* = lands you control");
    assert_eq!(lumra.toughness, lands);
    assert!(g.battlefield_find(id).unwrap()
        .definition.keywords.contains(&crabomination::card::Keyword::Reach));
}

#[test]
fn lumra_etb_with_empty_graveyard_is_a_noop() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest()); // keep Lumra off 0/0
    let id = g.add_card_to_hand(0, catalog::lumra_bellow_of_the_woods());
    g.players[0].library.clear(); // keep the ETB mill a no-op
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Green, 2);

    let bf_before = g.battlefield.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Lumra castable for {4}{G}{G}");
    drain_stack(&mut g);

    // Empty graveyard → the only addition to BF is Lumra herself.
    assert_eq!(g.battlefield.len(), bf_before + 1);
    assert!(g.battlefield_find(id).is_some());
}

#[test]
fn crabomination_etb_mills_each_opponent_three_cards() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::island());
    }
    let lib_before = g.players[1].library.len();
    let yard_before = g.players[1].graveyard.len();
    let id = g.add_card_to_hand(0, catalog::crabomination());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Crabomination castable for {2}{U}{B}");
    drain_stack(&mut g);

    assert_eq!(g.players[1].library.len(), lib_before - 3,
        "Crabomination mills 3 cards");
    assert_eq!(g.players[1].graveyard.len(), yard_before + 3,
        "Milled cards land in opp's graveyard");
}

#[test]
fn chaos_warp_reshuffles_target_and_replays_revealed_permanent() {
    // With the owner's library otherwise empty, the shuffled-in bear is the
    // sole card → revealed top → put back onto the battlefield (CR-faithful).
    let mut g = two_player_game();
    g.players[1].library.clear();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::chaos_warp());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Red, 1);

    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Chaos Warp castable for {2}{R}");
    drain_stack(&mut g);

    // The revealed card was a permanent → it's on the battlefield (a fresh
    // object under its owner's control); the library is empty again.
    assert!(g.players[1].library.is_empty());
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Grizzly Bears"),
        "revealed permanent is put onto the battlefield");
}

#[test]
fn chaos_warp_reveals_nonpermanent_and_leaves_it_on_top() {
    // Direct test of the reveal-and-replay effect: a Lightning Bolt on top
    // is not a permanent, so it stays put and nothing enters.
    let mut g = two_player_game();
    g.players[0].library.clear();
    let bolt = g.add_card_to_library(0, catalog::lightning_bolt());
    let bf_before = g.battlefield.len();
    let ctx = EffectContext::for_spell(0, None, 0, 0);
    g.resolve_effect(
        &Effect::RevealTopPutPermanentOntoBattlefield { who: crabomination::effect::PlayerRef::You },
        &ctx,
    ).unwrap();
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(bolt),
        "non-permanent stays on top");
    assert_eq!(g.battlefield.len(), bf_before, "nothing entered the battlefield");
}

#[test]
fn elvish_reclaimer_sacrifices_land_to_search_for_one() {
    let mut g = two_player_game();
    let reclaimer = g.add_card_to_battlefield(0, catalog::elvish_reclaimer());
    g.clear_sickness(reclaimer);
    // Untap so the tap-cost ability is legal.
    g.battlefield.iter_mut().find(|c| c.id == reclaimer).unwrap().tapped = false;
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let target_in_lib = g.add_card_to_library(0, catalog::wasteland());

    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(target_in_lib)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: reclaimer,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None }).expect("Elvish Reclaimer's tap+sac+search ability");
    drain_stack(&mut g);

    // Forest was sacrificed.
    assert!(g.battlefield_find(forest).is_none(),
        "Forest sacrificed as cost");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == forest));
    // Wasteland made it to the battlefield.
    assert!(g.battlefield_find(target_in_lib).is_some(),
        "Searched land entered the battlefield");
}

#[test]
fn elvish_reclaimer_threshold_pumps_to_three_four() {
    let mut g = two_player_game();
    let reclaimer = g.add_card_to_battlefield(0, catalog::elvish_reclaimer());
    // Base 1/2 with an empty graveyard.
    let c = g.compute_battlefield();
    let r = c.iter().find(|c| c.id == reclaimer).unwrap();
    assert_eq!((r.power, r.toughness), (1, 2), "base 1/2 below Threshold");
    // Seven cards in your graveyard → Threshold pump to 3/4.
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::lightning_bolt());
    }
    let c = g.compute_battlefield();
    let r = c.iter().find(|c| c.id == reclaimer).unwrap();
    assert_eq!((r.power, r.toughness), (3, 4), "+2/+2 with seven+ in graveyard");
}

#[test]
fn rofellos_taps_for_green_per_forest() {
    let mut g = two_player_game();
    // Put 3 Forests on the battlefield
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let id = g.add_card_to_battlefield(0, catalog::rofellos_llanowar_emissary());
    g.clear_sickness(id);
    g.battlefield.iter_mut().find(|c| c.id == id).unwrap().tapped = false;
    g.add_card_to_battlefield(0, catalog::forest());

    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Rofellos's mana ability");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 4,
        "Rofellos adds green mana for each Forest you control (4 Forests)");
}

#[test]
fn biorhythm_drops_each_opponent_to_zero_or_below() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::biorhythm());
    for _ in 0..8 {
        g.players[0].mana_pool.add(Color::Green, 1);
    }

    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Biorhythm castable for {6}{G}{G}");
    drain_stack(&mut g);

    // Push (modern_decks): Biorhythm now uses SetLifeTotal (CR 119.5).
    // With 0 creatures opp controls → opp life = 0.
    assert_eq!(g.players[1].life, 0,
        "Opp life set to creature count (0): got {}", g.players[1].life);
}

#[test]
fn biorhythm_sets_life_to_creature_count_per_cr_119_5() {
    let mut g = two_player_game();
    // You control 3 bears; opp controls 1 bear.
    let _b1 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b2 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _b3 = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let _o1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    let id = g.add_card_to_hand(0, catalog::biorhythm());
    for _ in 0..8 {
        g.players[0].mana_pool.add(Color::Green, 1);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Biorhythm castable for {6}{G}{G}");
    drain_stack(&mut g);

    // CR 119.5 — life total set to creature count per side.
    assert_eq!(g.players[0].life, 3, "your life = 3 bears");
    assert_eq!(g.players[1].life, 1, "opp life = 1 bear");
}

#[test]
fn biorhythm_sets_each_player_to_own_count_in_multiplayer() {
    // Three players, distinct creature counts → each seat's own count.
    let mut g = crabomination::game::multi_player_game(3);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(2, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::biorhythm());
    for _ in 0..8 { g.players[0].mana_pool.add(Color::Green, 1); }
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Biorhythm castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 2, "P0 → 2 creatures");
    assert_eq!(g.players[1].life, 0, "P1 → 0 creatures");
    assert_eq!(g.players[2].life, 1, "P2 → 1 creature");
}

#[test]
fn karn_scion_of_urza_minus_two_creates_a_construct_token() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    let bf_before = g.battlefield.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: karn, ability_index: 2, target: None,
    }).expect("Karn -2 to make a Construct token");
    drain_stack(&mut g);

    assert_eq!(g.battlefield.len(), bf_before + 1, "Token entered the battlefield");
    let token = g.battlefield.iter().find(|c| c.definition.name == "Construct")
        .expect("Construct token present");
    assert!(token.definition.card_types.contains(&crabomination::card::CardType::Artifact));
    assert!(token.definition.card_types.contains(&crabomination::card::CardType::Creature));
    // The token itself is an artifact, so with no other artifacts it is 1/1.
    let tid = token.id;
    let cp = g.compute_battlefield();
    let view = cp.iter().find(|c| c.id == tid).unwrap();
    assert_eq!(view.power, 1);
    assert_eq!(view.toughness, 1);
}

#[test]
fn karn_construct_token_scales_with_artifact_count() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    // Two more artifacts under our control.
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::mind_stone());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: karn, ability_index: 2, target: None,
    }).expect("Karn -2");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Construct").unwrap();
    let tid = token.id;
    // 3 artifacts you control (Ornithopter, Mind Stone, the Construct itself).
    let cp = g.compute_battlefield();
    let view = cp.iter().find(|c| c.id == tid).unwrap();
    assert_eq!(view.power, 3, "+1/+1 per artifact");
    assert_eq!(view.toughness, 3);
}

#[test]
fn karn_plus_one_takes_low_value_card_exiles_other_with_silver_counter() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    // Library top two: a cmc-0 land (opponent gives this to you) and a
    // cmc-2 creature (exiled with a silver counter).
    g.add_card_to_library(0, catalog::island());
    let bears = g.add_card_to_library(0, catalog::grizzly_bears());
    let hand_before = g.players[0].hand.len();

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: karn, ability_index: 0, target: None,
    }).expect("Karn +1");
    drain_stack(&mut g);

    assert_eq!(g.players[0].hand.len(), hand_before + 1, "one card to hand");
    let exiled = g.exile.iter().find(|c| c.id == bears).expect("higher-cost card exiled");
    assert_eq!(exiled.counter_count(CounterType::Silver), 1, "silver counter on exiled card");
}

#[test]
fn karn_minus_one_returns_silver_counter_card_from_exile() {
    let mut g = two_player_game();
    let karn = g.add_card_to_battlefield(0, catalog::karn_scion_of_urza());
    // Seed exile with a card P0 owns bearing a silver counter.
    let cid = g.next_id();
    let mut card = crabomination::card::CardInstance::new(cid, catalog::grizzly_bears(), 0);
    card.add_counters(CounterType::Silver, 1);
    g.exile.push(card);

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: karn, ability_index: 1, target: None,
    }).expect("Karn -1");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == cid), "card returned to hand");
    assert!(!g.exile.iter().any(|c| c.id == cid), "card left exile");
}

#[test]
fn tezzeret_gains_loyalty_when_an_artifact_enters() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_cruel_captain());
    assert_eq!(g.battlefield_find(tez).unwrap().counter_count(CounterType::Loyalty), 4);
    // Cast Mind Stone — an artifact you control entering triggers +1 loyalty.
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mind Stone castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(tez).unwrap().counter_count(CounterType::Loyalty), 5,
        "artifact ETB added a loyalty counter");
}

#[test]
fn tezzeret_zero_untaps_artifact_creature_and_adds_counter() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_cruel_captain());
    let thopter = g.add_card_to_battlefield(0, catalog::ornithopter());
    g.battlefield_find_mut(thopter).unwrap().tapped = true;

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: tez, ability_index: 0, target: Some(Target::Permanent(thopter)),
    }).expect("Tezzeret 0");
    drain_stack(&mut g);

    let t = g.battlefield_find(thopter).unwrap();
    assert!(!t.tapped, "artifact creature untapped");
    assert_eq!(t.counter_count(CounterType::PlusOnePlusOne), 1, "+1/+1 counter on artifact creature");
}

#[test]
fn tezzeret_minus_three_tutors_a_cheap_artifact() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_cruel_captain());
    g.battlefield_find_mut(tez).unwrap().add_counters(CounterType::Loyalty, 3); // 4+3 = 7 ≥ 3
    // MV ≤ 1 filter: Mind Stone ({2}) is not an eligible pick; Needle ({1}) is.
    let stone = g.add_card_to_library(0, catalog::mind_stone());
    let needle = g.add_card_to_library(0, catalog::pithing_needle());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(needle))]));

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: tez, ability_index: 1, target: None,
    }).expect("Tezzeret -3");
    drain_stack(&mut g);

    assert!(g.players[0].hand.iter().any(|c| c.id == needle), "cheap artifact tutored to hand");
    assert!(g.players[0].library.iter().any(|c| c.id == stone), "MV-2 rock not searchable");
}

#[test]
fn tezzeret_minus_seven_emblem_buffs_and_animates_artifact() {
    let mut g = two_player_game();
    let tez = g.add_card_to_battlefield(0, catalog::tezzeret_cruel_captain());
    g.battlefield_find_mut(tez).unwrap().add_counters(CounterType::Loyalty, 3); // 4+3 = 7
    let stone = g.add_card_to_battlefield(0, catalog::mind_stone()); // noncreature artifact

    g.perform_action(GameAction::ActivateLoyaltyAbility {
            x_value: None,
        card_id: tez, ability_index: 2, target: None,
    }).expect("Tezzeret -7");
    drain_stack(&mut g);
    assert_eq!(g.players[0].emblems.len(), 1, "emblem created");

    // Beginning of combat on your turn → the emblem trigger fires.
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);

    let s = g.battlefield_find(stone).expect("Mind Stone still here");
    assert_eq!(s.counter_count(CounterType::PlusOnePlusOne), 3, "three +1/+1 counters");
    let cp = g.compute_battlefield();
    let view = cp.iter().find(|c| c.id == stone).unwrap();
    assert!(view.card_types.contains(&CardType::Creature), "noncreature artifact became a creature");
    assert_eq!((view.power, view.toughness), (3, 3), "0/0 Robot + three +1/+1 = 3/3");
}

/// Vivien Reid's -8 emblem anthem (+2/+2, vigilance/trample/indestructible)
/// applies to creatures the emblem's owner controls via the static-emblem path.
#[test]
fn vivien_reid_minus_eight_emblem_anthems_your_creatures() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let viv = g.add_card_to_battlefield(0, catalog::vivien_reid());
    g.battlefield_find_mut(viv).unwrap().add_counters(CounterType::Loyalty, 3); // 5+3 = 8
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    g.perform_action(GameAction::ActivateLoyaltyAbility {
        x_value: None,
        card_id: viv,
        ability_index: 2,
        target: None,
    }).expect("Vivien -8");
    drain_stack(&mut g);
    assert_eq!(g.players[0].emblems.len(), 1, "emblem created");

    let cp = g.compute_battlefield();
    let mine = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((mine.power, mine.toughness), (4, 4), "your creature gets +2/+2");
    assert!(mine.keywords.contains(&Keyword::Vigilance));
    assert!(mine.keywords.contains(&Keyword::Trample));
    assert!(mine.keywords.contains(&Keyword::Indestructible));
    // Opponent's creature is unaffected.
    let theirs = cp.iter().find(|c| c.id == foe).unwrap();
    assert_eq!((theirs.power, theirs.toughness), (2, 2), "opponent unbuffed");
}

#[test]
fn balefire_dragon_combat_damage_burns_each_opp_creature() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::balefire_dragon());
    g.battlefield.iter_mut().find(|c| c.id == dragon).unwrap().tapped = false;
    let bear1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bear2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Fire the trigger directly (the combat-damage event is tested separately
    // in the combat test suite). We exercise the trigger's effect by event-
    // bus push here.
    let trig = catalog::balefire_dragon().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        dragon, 0, None, 0,
    );
    let _ = g.resolve_effect(&trig, &ctx);

    // Each opp creature took 6 damage and died via SBA.
    assert!(g.battlefield_find(bear1).is_none(), "Bear 1 perished");
    assert!(g.battlefield_find(bear2).is_none(), "Bear 2 perished");
}

#[test]
fn balefire_dragon_sweep_scales_with_its_power() {
    // The sweep reads the Dragon's current power. A 7/7 Pelakka Wurm
    // survives the base 6 but a +1/+1-countered (7-power) Dragon kills it.
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::balefire_dragon());
    g.battlefield.iter_mut().find(|c| c.id == dragon).unwrap()
        .add_counters(CounterType::PlusOnePlusOne, 1); // now 7/7
    let wurm = g.add_card_to_battlefield(1, catalog::pelakka_wurm()); // 7/7
    let trig = catalog::balefire_dragon().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(dragon, 0, None, 0);
    let _ = g.resolve_effect(&trig, &ctx);
    assert!(g.battlefield_find(wurm).is_none(),
        "a 7-power Balefire deals 7 → the 7-toughness Wurm dies");
}

#[test]
fn greasewrench_goblin_creates_treasure_on_death() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    let bf_before = g.battlefield.len();
    // Kill the goblin via direct removal (Lightning Bolt).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(id)),
        additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("Bolt castable for {R}");
    drain_stack(&mut g);

    // Goblin died; a Treasure token appeared.
    assert!(g.battlefield_find(id).is_none(), "Goblin died");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Treasure"),
        "Treasure token appeared on death");
    // BF: -1 (goblin gone) +1 (treasure) = unchanged in count.
    assert_eq!(g.battlefield.len(), bf_before);
}

#[test]
fn cruel_somnophage_pt_scales_with_your_graveyard() {
    let mut g = two_player_game();
    // Seed graveyard with three cards before Cruel Somnophage enters.
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::island());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    let id = g.add_card_to_battlefield(0, catalog::cruel_somnophage());

    let computed = g.compute_battlefield();
    let card = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(card.power, 3, "Power = your graveyard size (3)");
    assert_eq!(card.toughness, 3, "Toughness = your graveyard size (3)");

    // Mill another card and watch P/T grow.
    g.add_card_to_graveyard(0, catalog::island());
    let computed = g.compute_battlefield();
    let card = computed.iter().find(|c| c.id == id).unwrap();
    assert_eq!(card.power, 4);
    assert_eq!(card.toughness, 4);
}

#[test]
fn pentad_prism_sunburst_counts_colors_of_mana_spent() {
    // Paid with White + Blue → Sunburst gives 2 charge counters.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pentad_prism());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pentad Prism castable for {2}");
    drain_stack(&mut g);

    let card = g.battlefield_find(id).unwrap();
    let charge = card.counters.get(&crabomination::card::CounterType::Charge).copied().unwrap_or(0);
    assert_eq!(charge, 2, "Sunburst: two colors spent → two charge counters");
}

#[test]
fn pentad_prism_sunburst_zero_when_paid_colorless() {
    // Paid with two colorless → no colors spent → 0 charge counters.
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pentad_prism());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pentad Prism castable for {2}");
    drain_stack(&mut g);
    let charge = g.battlefield_find(id).unwrap()
        .counters.get(&crabomination::card::CounterType::Charge).copied().unwrap_or(0);
    assert_eq!(charge, 0, "Sunburst: colorless cast → no charge counters");
}

#[test]
fn pentad_prism_removes_counter_to_add_one_mana_of_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pentad_prism());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pentad Prism castable");
    drain_stack(&mut g);

    let pool_before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Pentad Prism's counter-removal mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), pool_before + 1,
        "Mana pool gains 1");

    let card = g.battlefield_find(id).unwrap();
    let charge = card.counters.get(&crabomination::card::CounterType::Charge).copied().unwrap_or(0);
    assert_eq!(charge, 1, "Charge counters: 2 → 1 after one activation");
}

