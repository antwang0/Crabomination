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
