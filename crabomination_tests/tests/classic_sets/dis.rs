//! Functionality tests for Dissension (DIS) gap cards in `catalog::sets::dis`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget};
use crabomination::game::*;
use crabomination::mana::Color;

/// Assault Zeppelid is a 3/3 with flying and trample.
#[test]
fn assault_zeppelid_flying_trample() {
    let z = catalog::assault_zeppelid();
    assert_eq!((z.power, z.toughness), (3, 3));
    assert!(z.keywords.contains(&Keyword::Flying));
    assert!(z.keywords.contains(&Keyword::Trample));
}

/// Sky Hussar untaps all your creatures when it enters.
#[test]
fn sky_hussar_untaps_your_creatures_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.move_card_to_battlefield_for_test(0, catalog::sky_hussar());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "ETB untapped your creature");
}

/// Stalking Vengeance turns a dying creature's power into damage to a player.
#[test]
fn stalking_vengeance_death_burns_target() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stalking_vengeance());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let life1 = g.players[1].life;
    // Kill the bear via lethal damage → SBA death → Stalking Vengeance trigger.
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 2, "dying 2/2 dealt 2 to the opponent");
}

/// Azorius Herald is unblockable, gains 4 on entry, and sticks when {U} was
/// spent to cast it.
#[test]
fn azorius_herald_stays_when_cast_with_blue() {
    let mut g = two_player_game();
    let herald = g.add_card_to_hand(0, catalog::azorius_herald());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: herald, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Azorius Herald with {U}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 4, "gained 4 life");
    assert!(g.battlefield.iter().any(|c| c.id == herald), "not sacrificed — U was spent");
    assert!(catalog::azorius_herald().keywords.contains(&Keyword::Unblockable));
}

/// Kill-Suit Cultist's sac ability replaces the next damage to a target
/// creature with destroying it (the damage is prevented, the creature dies).
#[test]
fn kill_suit_cultist_destroys_on_next_damage() {
    use crabomination::game::effects::EntityRef;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let cultist = g.add_card_to_battlefield(0, catalog::kill_suit_cultist());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cultist,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("activate Kill-Suit Cultist");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == cultist), "cultist sacrificed as a cost");
    // The next 1 damage to the bear is prevented and destroys it instead.
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(bear), 1, None, &mut events);
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear destroyed by the shield");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "bear went to its owner's graveyard");
    assert!(events.iter().any(|e| matches!(e, GameEvent::DamagePrevented { .. })), "damage prevented");
}

/// Nettling Curse: the enchanted creature attacking makes its controller lose
/// exactly 3 life (the aura's granted Attacks trigger fires once, not twice).
#[test]
fn nettling_curse_attack_drains_3() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let curse = g.add_card_to_battlefield(0, catalog::nettling_curse());
    g.battlefield_find_mut(curse).unwrap().attached_to = Some(bear);
    let life0 = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 3, "controller lost exactly 3 (no double-fire)");
}

/// Riot Spikes grants +2/-1 to the enchanted creature.
#[test]
fn riot_spikes_pumps_plus2_minus1() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spikes = g.add_card_to_battlefield(0, catalog::riot_spikes());
    g.battlefield_find_mut(spikes).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 1));
}

/// Proper Burial gains life equal to a dying creature's toughness.
#[test]
fn proper_burial_gains_toughness_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::proper_burial());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let life0 = g.players[0].life;
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2, "gained life equal to the dead 2/2's toughness");
}

/// Rain of Gore turns a controller's life gain into an equal loss.
#[test]
fn rain_of_gore_flips_lifegain_to_loss() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rain_of_gore());
    let life0 = g.players[0].life;
    g.adjust_life(0, 3); // a spell/ability would gain 3 → lose 3 instead
    assert_eq!(g.players[0].life, life0 - 3);
}

/// Skullmead Cauldron's plain tap ability gains 1 life.
#[test]
fn skullmead_cauldron_taps_for_1_life() {
    let mut g = two_player_game();
    let cauldron = g.add_card_to_battlefield(0, catalog::skullmead_cauldron());
    let life0 = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cauldron,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("tap for life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1);
}

/// Celestial Ancient bolsters your team when you cast an enchantment spell.
#[test]
fn celestial_ancient_counters_on_enchantment_cast() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::celestial_ancient());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Cast an enchantment (Rain of Gore) from hand.
    let ench = g.add_card_to_hand(0, catalog::rain_of_gore());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: ench, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast enchantment");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "+1/+1 counter added on enchantment cast"
    );
}

/// Slithering Shade can't attack normally (Defender) but its {B} pump works and,
/// while hellbent (empty hand), it may attack ignoring Defender.
#[test]
fn slithering_shade_pumps_and_hellbent_attacks() {
    let mut g = two_player_game();
    let shade = g.add_card_to_battlefield(0, catalog::slithering_shade());
    g.clear_sickness(shade);
    // {B}: +1/+1.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shade, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(shade).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 2));
    // Empty hand → hellbent → can attack despite Defender.
    g.players[0].hand.clear();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: shade, target: AttackTarget::Player(1),
    }]))
    .expect("hellbent attack despite Defender");
}

/// Ocular Halo grants the enchanted creature "{T}: Draw a card."
#[test]
fn ocular_halo_grants_tap_draw() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let halo = g.add_card_to_battlefield(0, catalog::ocular_halo());
    g.battlefield_find_mut(halo).unwrap().attached_to = Some(bear);
    g.add_card_to_library(0, catalog::grizzly_bears()); // something to draw
    let hand0 = g.players[0].hand.len();
    // The granted {T}: Draw sits after the bear's own abilities (none here) →
    // index 0 among granted; effective index is the first granted mana/ability.
    let abilities = g.granted_abilities_for(bear);
    assert!(!abilities.is_empty(), "Ocular Halo grants an activated ability");
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("tap to draw");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Sprouting Phytohydra makes a token copy of itself when dealt damage.
#[test]
fn sprouting_phytohydra_copies_on_damage() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // opt into the copy
    let hydra = g.add_card_to_battlefield(0, catalog::sprouting_phytohydra());
    let before = g.battlefield.iter().filter(|c| c.definition.name == "Sprouting Phytohydra").count();
    let mut events = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(hydra), 1, None, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.definition.name == "Sprouting Phytohydra").count();
    assert_eq!(after, before + 1, "a token copy was created");
}

/// Street Savvy grants +0/+2 to the enchanted creature.
#[test]
fn street_savvy_grants_plus0_plus2() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let ss = g.add_card_to_battlefield(0, catalog::street_savvy());
    g.battlefield_find_mut(ss).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4));
}

/// Ratcatcher has Fear and an upkeep may-search that resolves safely when the
/// library holds no Rat.
#[test]
fn ratcatcher_upkeep_search_is_safe() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // not a Rat
    g.add_card_to_battlefield(0, catalog::ratcatcher());
    g.active_player_idx = 0;
    let hand0 = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0, "no Rat to fetch → hand unchanged");
    assert!(catalog::ratcatcher().keywords.contains(&Keyword::Fear));
}

/// Nihilistic Glee's discard-drain ability: pay {2}{B}, discard → opponent
/// loses 1 and you gain 1.
#[test]
fn nihilistic_glee_discard_drains() {
    let mut g = two_player_game();
    let glee = g.add_card_to_battlefield(0, catalog::nihilistic_glee());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.perform_action(GameAction::ActivateAbility {
        card_id: glee, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate drain");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, l0 + 1, "you gained 1");
}

/// Cytospawn Shambler enters with six +1/+1 counters (Graft 6).
#[test]
fn cytospawn_shambler_enters_with_six_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let sh = g.move_card_to_battlefield_for_test(0, catalog::cytospawn_shambler());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sh).unwrap().counter_count(CounterType::PlusOnePlusOne), 6);
    let cp = g.computed_permanent(sh).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6));
}

/// Cytoplast Manipulator steals a counter-bearing creature while it remains.
#[test]
fn cytoplast_manipulator_steals_counter_creature() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let man = g.move_card_to_battlefield_for_test(0, catalog::cytoplast_manipulator());
    drain_stack(&mut g);
    g.clear_sickness(man);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: man,
        ability_index: 0,
        target: Some(crabomination::game::types::Target::Permanent(victim)),
        additional_targets: Vec::new(),
        x_value: None,
    })
    .expect("steal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0, "creature is now controlled by you");
}

/// Paladin of Prahv has lifelink (its "gain life on damage" body).
#[test]
fn paladin_of_prahv_has_lifelink() {
    let p = catalog::paladin_of_prahv();
    assert_eq!((p.power, p.toughness), (3, 4));
    assert!(p.keywords.contains(&Keyword::Lifelink));
}

/// Wit's End empties a target player's hand.
#[test]
fn wits_end_discards_target_hand() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let we = g.add_card_to_hand(0, catalog::wits_end());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: we, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Wit's End at P1");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent's hand emptied");
}

/// Weight of Spires deals damage equal to the target's controller's nonbasic
/// land count.
#[test]
fn weight_of_spires_scales_with_nonbasic_lands() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    // Two nonbasic lands under P1 → 2 damage → lethal to the 2/2.
    g.add_card_to_battlefield(1, catalog::watery_grave());
    g.add_card_to_battlefield(1, catalog::watery_grave());
    let wos = g.add_card_to_hand(0, catalog::weight_of_spires());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: wos, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Weight of Spires");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 took 2 damage (2 nonbasics) and died");
}

/// Tidespout Tyrant bounces a permanent whenever you cast a spell.
#[test]
fn tidespout_tyrant_bounces_on_cast() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tidespout_tyrant());
    let victim = g.add_card_to_battlefield(1, catalog::watery_grave()); // the only non-tyrant permanent
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Direct the tyrant's bounce trigger at the victim.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast a spell");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == victim), "victim bounced to hand");
}

/// Taste for Mayhem grants +2/+0, plus another +2/+0 while you're hellbent.
#[test]
fn taste_for_mayhem_grants_plus2_and_hellbent_bonus() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_battlefield(0, catalog::taste_for_mayhem());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    // Non-empty hand → base +2/+0 only.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "base +2/+0");
    // Empty hand → hellbent → an additional +2/+0.
    g.players[0].hand.clear();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 2), "hellbent adds +2/+0");
}

/// Windreaver can switch its power and toughness and bounce itself.
#[test]
fn windreaver_switch_pt_and_bounce() {
    let mut g = two_player_game();
    let wr = g.add_card_to_battlefield(0, catalog::windreaver()); // 1/3
    g.clear_sickness(wr);
    // {U}: switch P/T → 3/1.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wr, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("switch P/T");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wr).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1));
    // {U}: return to hand.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: wr, ability_index: 3, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("bounce self");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == wr), "Windreaver returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == wr));
}

/// Walking Archive draws the active player a card per +1/+1 counter on it.
#[test]
fn walking_archive_draws_per_counter() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let wa = g.move_card_to_battlefield_for_test(0, catalog::walking_archive());
    drain_stack(&mut g);
    // Enters with one +1/+1 counter (Graft-style ETB counter).
    assert_eq!(g.battlefield_find(wa).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    g.active_player_idx = 0;
    let hand0 = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew 1 card (one counter)");
}

/// Nightcreep turns every creature black and every land into a Swamp until EOT.
#[test]
fn nightcreep_recolors_creatures_and_lands() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::nightcreep());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Nightcreep");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().colors.contains(&Color::Black), "creature is black");
    assert!(
        g.computed_permanent(forest).unwrap().subtypes.land_types.contains(&LandType::Swamp),
        "land became a Swamp",
    );
}

/// Demonfire deals X damage to a creature and exiles it instead of letting it die.
#[test]
fn demonfire_exiles_what_it_kills() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::demonfire());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast Demonfire for X=2");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear died to 2 damage");
    assert!(g.exile.iter().any(|c| c.id == bear), "exiled instead of graveyard");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// Biomantic Mastery draws a card per creature each targeted player controls.
#[test]
fn biomantic_mastery_draws_per_targeted_creatures() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let spell = g.add_card_to_hand(0, catalog::biomantic_mastery());
    g.players[0].mana_pool.add(Color::Green, 3);
    g.players[0].mana_pool.add_colorless(4);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(0)),
        additional_targets: vec![Target::Player(1)],
        mode: None,
        x_value: None,
    })
    .expect("cast Biomantic Mastery");
    drain_stack(&mut g);
    // 2 creatures (player 0) + 1 creature (player 1) = 3 cards, minus the spell that left hand.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 3, "drew 3 cards");
}

/// Leafdrake Roost lets the enchanted land mint a 2/2 flying Drake.
#[test]
fn leafdrake_roost_land_makes_drakes() {
    use crabomination::card::{CreatureType, Keyword};
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let aura = g.add_card_to_battlefield(0, catalog::leafdrake_roost());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(land);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    // Index 0 is the land's intrinsic {T}: Add {G}; the granted Drake ability
    // is surfaced at the next index.
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate the granted Drake ability");
    drain_stack(&mut g);
    let drake = g.battlefield.iter().find(|c| c.definition.name == "Drake").expect("minted a Drake");
    assert_eq!((drake.definition.power, drake.definition.toughness), (2, 2));
    assert!(drake.definition.keywords.contains(&Keyword::Flying));
    assert!(drake.definition.subtypes.creature_types.contains(&CreatureType::Drake));
}

/// Brain Pry: name a card in the target's hand → they discard it. Name a card
/// they lack → the caster draws instead.
#[test]
fn brain_pry_discards_named_or_draws() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    use crabomination::game::types::Target;
    // Hit: name a card the opponent holds.
    let mut g = two_player_game();
    let victim = g.add_card_to_hand(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::brain_pry());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Grizzly Bears".into())]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Brain Pry (hit)");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == victim), "named card discarded");

    // Miss: opponent has no card with that name → caster draws.
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::brain_pry());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand0 = g.players[0].hand.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::NamedCard("Lightning Bolt".into())]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Brain Pry (miss)");
    drain_stack(&mut g);
    // Spell left hand (-1), then drew a card (+1) → net unchanged, and library shrank.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 1, "caster drew on a miss");
}

/// Grand Arbiter makes your blue spells cost {1} less and opponents' cost more.
#[test]
fn grand_arbiter_taxes_and_discounts() {
    use crabomination::game::actions::{cost_reduction_for_spell, extra_cost_for_spell};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grand_arbiter_augustin_iv());
    let blue = crabomination::card::CardInstance::new(g.next_id(), catalog::counterspell(), 0); // {U}{U}
    assert_eq!(extra_cost_for_spell(&g, 0, &blue, None, 0), 0, "your spell isn't taxed");
    assert_eq!(cost_reduction_for_spell(&g, 0, &blue, None), 1, "your blue spell is discounted {{1}}");
    assert_eq!(extra_cost_for_spell(&g, 1, &blue, None, 0), 1, "opponent spell taxed {{1}}");
}

/// Magewright's Stone untaps a target creature.
#[test]
fn magewrights_stone_untaps_a_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(0, catalog::magewrights_stone());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("untap the bear");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "bear untapped");
}

/// Hellhole Rats makes a player discard, then burns them for the discard's MV.
#[test]
fn hellhole_rats_discards_and_burns_by_mv() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // The opponent holds a single MV-2 card, so the discard is forced.
    g.add_card_to_hand(1, catalog::counterspell()); // {U}{U} → MV 2
    let rats = g.add_card_to_hand(0, catalog::hellhole_rats());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: rats, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Hellhole Rats");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded their card");
    assert_eq!(g.players[1].life, life1 - 2, "burned for the discarded MV (Counterspell = 2)");
}

/// Blessing of the Nephilim pumps +1/+1 per color of the enchanted creature.
#[test]
fn blessing_of_the_nephilim_scales_with_colors() {
    let mut g = two_player_game();
    // Mono-green host → +1/+1 (one color).
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 green
    let b1 = g.add_card_to_battlefield(0, catalog::blessing_of_the_nephilim());
    g.battlefield_find_mut(b1).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "mono-color → +1/+1");
    // Two-color host → +2/+2.
    let rats = g.add_card_to_battlefield(0, catalog::hellhole_rats()); // 2/2 B/R
    let b2 = g.add_card_to_battlefield(0, catalog::blessing_of_the_nephilim());
    g.battlefield_find_mut(b2).unwrap().attached_to = Some(rats);
    let cp = g.computed_permanent(rats).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "two colors → +2/+2");
}

/// Voidslime counters a spell on the stack.
#[test]
fn voidslime_counters_a_spell() {
    use crabomination::game::types::{StackItem, Target};
    let mut g = two_player_game();
    // Opponent casts a spell.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the bolt");
    assert!(matches!(g.stack.last(), Some(StackItem::Spell { card, .. }) if card.id == bolt));
    // Respond with Voidslime.
    g.priority.player_with_priority = 0;
    let vs = g.add_card_to_hand(0, catalog::voidslime());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: vs, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Voidslime at the spell");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "spell countered to graveyard");
}

/// Voidslime counters an activated ability on the stack.
#[test]
fn voidslime_counters_an_ability() {
    use crabomination::game::types::{StackItem, Target};
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.clear_sickness(stone);
    g.players[1].mana_pool.add_colorless(1);
    g.add_card_to_library(1, catalog::island());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate the draw/sac ability");
    assert!(matches!(g.stack.last(), Some(StackItem::Trigger { source, .. }) if *source == stone));
    g.priority.player_with_priority = 0;
    let vs = g.add_card_to_hand(0, catalog::voidslime());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: vs, target: Some(Target::Permanent(stone)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Voidslime at the ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before, "the ability was countered — no draw");
}

/// Dread Slag is 9/9 but shrinks −4/−4 per card in your hand.
#[test]
fn dread_slag_shrinks_with_hand() {
    let mut g = two_player_game();
    let slag = g.add_card_to_battlefield(0, catalog::dread_slag());
    // Empty hand → full 9/9.
    g.players[0].hand.clear();
    let cp = g.computed_permanent(slag).unwrap();
    assert_eq!((cp.power, cp.toughness), (9, 9), "empty hand → 9/9");
    // Two cards in hand → 9 − 8 = 1/1.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(slag).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "two cards → −8/−8");
}

/// Cytoshape turns a creature into a copy of another until end of turn.
#[test]
fn cytoshape_copies_a_creature() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::ancestor_dragon()); // 5/6
    let spell = g.add_card_to_hand(0, catalog::cytoshape());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(big)],
        mode: None,
        x_value: None,
    })
    .expect("cast Cytoshape");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 6), "bear became a 5/6 Ancestor Dragon copy");
}

/// Rakdos the Defiler sacrifices half your non-Demon permanents when it attacks.
#[test]
fn rakdos_the_defiler_sacs_half_on_attack() {
    let mut g = two_player_game();
    let rakdos = g.add_card_to_battlefield(0, catalog::rakdos_the_defiler());
    g.clear_sickness(rakdos);
    // Four non-Demon permanents → sacrifice half rounded up = 2.
    for _ in 0..4 { g.add_card_to_battlefield(0, catalog::grizzly_bears()); }
    let before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rakdos, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    let after = g.battlefield.iter().filter(|c| c.controller == 0).count();
    assert_eq!(before - after, 2, "sacrificed half (2 of 4 non-Demons)");
    assert!(g.battlefield.iter().any(|c| c.id == rakdos), "Rakdos (a Demon) is spared");
}

/// Avatar of Discord sacrifices itself on ETB when you can't discard two cards.
#[test]
fn avatar_of_discord_sacs_without_two_discards() {
    let mut g = two_player_game();
    g.players[0].hand.clear(); // can't pay the two-card discard
    let avatar = g.move_card_to_battlefield_for_test(0, catalog::avatar_of_discord());
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == avatar), "sacrificed — couldn't discard two");
}

/// Omnibian turns a creature into a 3/3 Frog until end of turn.
#[test]
fn omnibian_makes_a_3_3_frog() {
    use crabomination::card::CreatureType;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let omni = g.add_card_to_battlefield(0, catalog::omnibian());
    g.clear_sickness(omni);
    let target = g.add_card_to_battlefield(1, catalog::ancestor_dragon()); // 5/6
    g.perform_action(GameAction::ActivateAbility {
        card_id: omni, ability_index: 0,
        target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate Omnibian");
    drain_stack(&mut g);
    let cp = g.computed_permanent(target).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "became a 3/3");
    assert!(cp.subtypes.creature_types.contains(&CreatureType::Frog), "is a Frog");
}

/// Unliving Psychopath pumps +1/-1 and destroys weaker creatures.
#[test]
fn unliving_psychopath_pumps_and_kills() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let psycho = g.add_card_to_battlefield(0, catalog::unliving_psychopath()); // 0/4
    g.clear_sickness(psycho);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    // Pump twice → 2/2. Now power 2 is not < 2, so pump a third time → 3/1.
    g.players[0].mana_pool.add(Color::Black, 4);
    for _ in 0..3 {
        g.perform_action(GameAction::ActivateAbility {
            card_id: psycho, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("pump");
        drain_stack(&mut g);
    }
    assert_eq!(g.computed_permanent(psycho).unwrap().power, 3, "pumped to power 3");
    // {B},{T}: destroy the 2/2 (power 2 < 3).
    g.perform_action(GameAction::ActivateAbility {
        card_id: psycho, ability_index: 1,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("destroy the weaker bear");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear destroyed");
}

/// Govern the Guildless steals a monocolored creature.
#[test]
fn govern_the_guildless_steals_monocolored() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // mono-green
    let spell = g.add_card_to_hand(0, catalog::govern_the_guildless());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Govern the Guildless");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "you now control the bear");
}

/// Anthem of Rakdos pumps your attacker +2/+0 and pings you for 1.
#[test]
fn anthem_of_rakdos_pumps_attacker_and_pings_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::anthem_of_rakdos());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    let life0 = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "attacker got +2/+0");
    assert_eq!(g.players[0].life, life0 - 1, "Anthem pinged you for 1");
}

/// Plumes of Peace keeps the enchanted creature from untapping.
#[test]
fn plumes_of_peace_locks_untap() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let aura = g.add_card_to_battlefield(0, catalog::plumes_of_peace());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.do_untap();
    assert!(g.battlefield_find(bear).unwrap().tapped, "stayed tapped through untap");
}
