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

/// Sky Hussar's Forecast taps two W/U creatures you control to draw a card,
/// from hand during your upkeep, leaving the card in hand.
#[test]
fn sky_hussar_forecast_taps_two_to_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // Two W/U creatures to feed the tap cost.
    let a = g.add_card_to_battlefield(0, catalog::azorius_first_wing()); // {W}{U} → W+U
    let b = g.add_card_to_battlefield(0, catalog::azorius_first_wing());
    g.clear_sickness(a);
    g.clear_sickness(b);
    let hussar = g.add_card_to_hand(0, catalog::sky_hussar());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: hussar, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Forecast activatable from hand in upkeep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped,
        "both W/U creatures tapped as the cost");
    assert_eq!(g.players[0].hand.len(), h0 + 1, "drew a card (Hussar stays, +1 net)");
    assert!(g.players[0].hand.iter().any(|c| c.id == hussar), "Sky Hussar stays in hand");
}

/// Plumes of Peace's Forecast taps a target creature from hand in upkeep.
#[test]
fn plumes_of_peace_forecast_taps_target() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let plumes = g.add_card_to_hand(0, catalog::plumes_of_peace());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: plumes, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Forecast activatable from hand in upkeep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "Forecast tapped the bear");
    assert!(g.players[0].hand.iter().any(|c| c.id == plumes), "card stays in hand");
}

/// Govern the Guildless's Forecast recolors a target creature from hand; the
/// upkeep-only gate rejects activation at other times.
#[test]
fn govern_the_guildless_forecast_upkeep_only() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let govern = g.add_card_to_hand(0, catalog::govern_the_guildless());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Outside upkeep the Forecast is illegal.
    g.step = TurnStep::PreCombatMain;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: govern, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).is_err(), "Forecast is upkeep-only");
    // During upkeep it recolors the creature.
    g.step = TurnStep::Upkeep;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Color(Color::Red),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: govern, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Forecast activatable in upkeep");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().colors, vec![Color::Red],
        "target became the chosen color");
    assert!(g.players[0].hand.iter().any(|c| c.id == govern), "card stays in hand");
}

/// A Karoo bounce-land enters tapped and returns a land you control to hand.
#[test]
fn karoo_enters_tapped_and_bounces_a_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest()); // a land to bounce
    let h0 = g.players[0].hand.len();
    let karoo = g.move_card_to_battlefield_for_test(0, catalog::azorius_chancery());
    drain_stack(&mut g);
    assert!(g.battlefield_find(karoo).unwrap().tapped, "Karoo enters tapped");
    assert_eq!(g.players[0].hand.len(), h0 + 1, "a land was returned to hand");
}

/// A Karoo taps for both of its guild colors at once.
#[test]
fn karoo_taps_for_two_colors() {
    let mut g = two_player_game();
    // Bypass the ETB (add_card enters untapped, no trigger); tap for {G}{U}.
    let karoo = g.add_card_to_battlefield(0, catalog::simic_growth_chamber());
    g.clear_sickness(karoo);
    g.perform_action(GameAction::ActivateAbility {
        card_id: karoo, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap Karoo for two mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "added green");
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "added blue");
}

/// Writ of Passage's Forecast makes a power-2-or-less creature unblockable.
#[test]
fn writ_of_passage_forecast_grants_unblockable() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → power 2
    let writ = g.add_card_to_hand(0, catalog::writ_of_passage());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: writ, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Forecast activatable in upkeep");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Unblockable),
        "power-2 creature became unblockable");
    assert!(g.players[0].hand.iter().any(|c| c.id == writ), "card stays in hand");
}

/// Flaring Flame-Kin gets +2/+2 and trample only while enchanted.
#[test]
fn flaring_flame_kin_pumps_while_enchanted() {
    let mut g = two_player_game();
    let kin = g.add_card_to_battlefield(0, catalog::flaring_flame_kin());
    // Bare: 2/2, no trample.
    let base = g.computed_permanent(kin).unwrap();
    assert_eq!((base.power, base.toughness), (2, 2));
    assert!(!base.keywords.contains(&Keyword::Trample));
    // Enchant it with any aura.
    let aura = g.add_card_to_hand(0, catalog::riot_spikes()); // +2/-1 aura
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(crabomination::game::types::Target::Permanent(kin)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("attach aura");
    drain_stack(&mut g);
    let buffed = g.computed_permanent(kin).unwrap();
    // 2/2 base +2/+2 (enchanted) +2/-1 (Riot Spikes) = 6/3, with trample.
    assert_eq!((buffed.power, buffed.toughness), (6, 3), "enchanted pump + aura");
    assert!(buffed.keywords.contains(&Keyword::Trample), "gains trample while enchanted");
}

/// Haazda Shield Mate sacrifices itself at upkeep if {W}{W} isn't paid.
#[test]
fn haazda_shield_mate_sacrifices_without_payment() {
    let mut g = two_player_game();
    let haazda = g.add_card_to_battlefield(0, catalog::haazda_shield_mate());
    g.active_player_idx = 0;
    // No mana available → the MayPay declines → sacrifice.
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(haazda).is_none(), "sacrificed without paying WW");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == haazda), "in graveyard");
}

/// Prahv, Spires of Order taps for {C} and can prevent a chosen source's damage.
#[test]
fn prahv_taps_for_colorless() {
    let mut g = two_player_game();
    let prahv = g.add_card_to_battlefield(0, catalog::prahv_spires_of_order());
    g.clear_sickness(prahv);
    g.perform_action(GameAction::ActivateAbility {
        card_id: prahv, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap Prahv for {C}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "added colorless");
}

/// Jagged Poppet discards cards equal to damage it's dealt.
#[test]
fn jagged_poppet_discards_on_damage() {
    let mut g = two_player_game();
    let poppet = g.add_card_to_battlefield(0, catalog::jagged_poppet());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    let h0 = g.players[0].hand.len();
    // Deal 2 damage to the Poppet, then dispatch the DamageDealt event.
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Permanent(poppet), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 2, "discarded 2 (the damage dealt)");
}

/// Palliation Accord accrues a counter when an opponent's creature is tapped.
#[test]
fn palliation_accord_counters_on_opponent_tap() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let accord = g.add_card_to_battlefield(0, catalog::palliation_accord());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    g.dispatch_triggers_for_events(&[GameEvent::PermanentTapped { card_id: foe, actor: None }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(accord).unwrap().counter_count(CounterType::Palliation), 1,
        "opponent tap added a palliation counter");
}

/// Paladin of Prahv's Forecast: you gain life when the watched creature deals
/// combat damage this turn.
#[test]
fn paladin_of_prahv_forecast_gains_life_on_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Watch our own attacker.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    let paladin = g.add_card_to_hand(0, catalog::paladin_of_prahv());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: paladin, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Forecast activatable in upkeep");
    drain_stack(&mut g);
    let life0 = g.players[0].life;
    // Bear deals 2 noncombat damage to the opponent → we gain 2.
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1), 2, Some(bear), &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 2, "gained life equal to the watched damage");
}

/// Pain Magnification: an opponent dealt 3+ by a single source discards.
#[test]
fn pain_magnification_discards_on_big_hit() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pain_magnification());
    g.add_card_to_hand(1, catalog::forest());
    let h1 = g.players[1].hand.len();
    // 3 damage to the opponent → they discard.
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1), 3, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1 - 1, "opponent discarded after a 3-damage hit");
}

/// Pain Magnification does not fire on a hit below 3.
#[test]
fn pain_magnification_ignores_small_hit() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pain_magnification());
    g.add_card_to_hand(1, catalog::forest());
    let h1 = g.players[1].hand.len();
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1), 2, None, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1, "2 damage is below the threshold");
}

/// A Forecast card is surfaced as `hand_activatable` only during the owner's
/// upkeep (its printed timing gate), not in other steps.
#[test]
fn forecast_hand_activatable_only_in_upkeep() {
    let mut g = two_player_game();
    let plumes = g.add_card_to_hand(0, catalog::plumes_of_peace());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    assert!(!g.compute_hand_affordances(0).hand_activatable.contains(&plumes),
        "Forecast is not offered outside upkeep");
    g.step = TurnStep::Upkeep;
    assert!(g.compute_hand_affordances(0).hand_activatable.contains(&plumes),
        "Forecast is offered during upkeep");
}

/// Rakdos Augermage's {T} makes both players discard a card.
#[test]
fn rakdos_augermage_mutual_discard() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::rakdos_augermage());
    g.clear_sickness(mage);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let (h0, h1) = (g.players[0].hand.len(), g.players[1].hand.len());
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Augermage (sorcery speed, our main)");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1, "you discarded one");
    assert_eq!(g.players[1].hand.len(), h1 - 1, "opponent discarded one");
}

/// Drekavac sacrifices itself unless a noncreature card is discarded.
#[test]
fn drekavac_sacrificed_without_noncreature_discard() {
    let mut g = two_player_game();
    // Only creatures in hand → no valid discard → sacrifice.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let drek = g.move_card_to_battlefield_for_test(0, catalog::drekavac());
    drain_stack(&mut g);
    assert!(g.battlefield_find(drek).is_none(), "sacrificed with no noncreature card");
}

/// Drekavac survives when a noncreature card is discarded.
#[test]
fn drekavac_kept_with_noncreature_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::forest()); // a noncreature (land) to discard
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    let drek = g.move_card_to_battlefield_for_test(0, catalog::drekavac());
    drain_stack(&mut g);
    assert!(g.battlefield_find(drek).is_some(), "kept by discarding a land");
    assert_eq!(g.players[0].hand.len(), 0, "the land was discarded");
}

/// Crypt Champion reanimates a MV≤3 creature for each player on entry.
#[test]
fn crypt_champion_each_player_reanimates() {
    let mut g = two_player_game();
    // Each player has an eligible creature in their graveyard.
    let mine = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.move_card_to_battlefield_for_test(0, catalog::crypt_champion());
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_some(), "your creature came back");
    assert!(g.battlefield_find(theirs).is_some(), "opponent's creature came back too");
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
    g.add_card_to_hand(0, catalog::grizzly_bears()); // non-empty hand → not hellbent
    let life0 = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 4, "attacker got +2/+0");
    assert_eq!(g.players[0].life, life0 - 1, "Anthem pinged you for 1 (not hellbent)");
}

/// Anthem of Rakdos doubles your sources' damage while you're hellbent.
#[test]
fn anthem_of_rakdos_hellbent_doubles_damage() {
    use crabomination::game::effects::EntityRef;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::anthem_of_rakdos());
    // Empty hand → hellbent → your sources deal double.
    g.players[0].hand.clear();
    let life1 = g.players[1].life;
    let mut events = Vec::new();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.deal_damage_to_from(EntityRef::Player(1), 3, Some(src), &mut events);
    assert_eq!(g.players[1].life, life1 - 6, "3 damage doubled to 6 while hellbent");
    // Non-empty hand → no doubling.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let life1b = g.players[1].life;
    g.deal_damage_to_from(EntityRef::Player(1), 3, Some(src), &mut events);
    assert_eq!(g.players[1].life, life1b - 3, "not hellbent → normal damage");
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

/// Freewind Equenaut only gains its {T}: ping ability while it's enchanted.
#[test]
fn freewind_equenaut_ability_requires_enchant() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let equ = g.add_card_to_battlefield(0, catalog::freewind_equenaut());
    g.clear_sickness(equ);
    // No aura yet → the granted ability isn't available (index out of bounds).
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking.push(Attack { attacker, target: AttackTarget::Player(0) });
    let before = g.perform_action(GameAction::ActivateAbility {
        card_id: equ, ability_index: 0,
        target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None,
    });
    assert!(before.is_err(), "no ability while unenchanted");
    // Attach an aura → the ping ability appears at index 0.
    let aura = g.add_card_to_battlefield(0, catalog::plumes_of_peace());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(equ);
    g.perform_action(GameAction::ActivateAbility {
        card_id: equ, ability_index: 0,
        target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None,
    })
    .expect("ping the attacker while enchanted");
    drain_stack(&mut g);
    let _ = g.check_state_based_actions();
    assert!(!g.battlefield.iter().any(|c| c.id == attacker), "2 damage killed the 2/2 attacker");
}

/// Rix Maadi's sorcery-speed ability makes each player discard a card.
#[test]
fn rix_maadi_each_player_discards() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::rix_maadi_dungeon_palace());
    g.clear_sickness(land);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate the discard ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "P0 discarded");
    assert_eq!(g.players[1].hand.len(), 0, "P1 discarded");
}

/// Novijen counters up every creature that entered this turn.
#[test]
fn novijen_counters_entered_this_turn() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::novijen_heart_of_progress());
    g.clear_sickness(land);
    let fresh = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("activate Novijen");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(fresh).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "the just-entered creature got a +1/+1 counter",
    );
}

/// Pillar of the Paruns adds mana that only pays for multicolored spells.
#[test]
fn pillar_of_the_paruns_multicolored_only() {
    use crabomination::mana::{SpellKind, SpendRestriction};
    let mono = SpellKind { multicolored: false, ..Default::default() };
    let gold = SpellKind { multicolored: true, ..Default::default() };
    assert!(!SpendRestriction::MulticoloredSpell.allows(&mono), "rejects a monocolored spell");
    assert!(SpendRestriction::MulticoloredSpell.allows(&gold), "funds a multicolored spell");
    // And the land actually produces restricted mana.
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::pillar_of_the_paruns());
    g.clear_sickness(land);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Color(Color::Red),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    })
    .expect("tap Pillar for restricted mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "Pillar tapped to add mana");
}

// ─── DIS second gap wave (gaps2) ────────────────────────────────────────────

/// Protean Hulk's death fetches creatures with total mana value 6 or less onto
/// the battlefield.
#[test]
fn protean_hulk_dies_reanimates_from_library() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let hulk = g.add_card_to_battlefield(0, catalog::protean_hulk());
    let b1 = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    let b2 = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![b1, b2])]));
    let evs = g.remove_to_graveyard_with_triggers(hulk);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let bears = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears, 2, "both MV-2 bears (total 4 ≤ 6) came onto the battlefield");
}

/// Swift Silence counters every other spell on the stack and draws one card per
/// spell countered.
#[test]
fn swift_silence_counters_others_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    // A creature spell sits on the stack; Swift Silence answers it at instant
    // speed and draws for the one spell it counters.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear onto the stack");
    let silence = g.add_card_to_hand(0, catalog::swift_silence());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 2);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: silence, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Swift Silence in response");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"),
        "the bear spell was countered before resolving");
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 1, "drew a card for the one spell countered");
}

/// Lyzolda deals 2 damage when the sacrificed creature was red.
#[test]
fn lyzolda_sac_red_deals_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let lyz = g.add_card_to_battlefield(0, catalog::lyzolda_the_blood_witch());
    g.clear_sickness(lyz);
    g.add_card_to_battlefield(0, catalog::goblin_guide()); // a red creature to sacrifice
    g.players[0].mana_pool.add_colorless(2);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: lyz, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Lyzolda sacrificing a red creature");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 2, "red sacrifice dealt 2 to the opponent");
}

/// Lyzolda draws a card when the sacrificed creature was black.
#[test]
fn lyzolda_sac_black_draws() {
    let mut g = two_player_game();
    let lyz = g.add_card_to_battlefield(0, catalog::lyzolda_the_blood_witch());
    g.clear_sickness(lyz);
    g.add_card_to_battlefield(0, catalog::black_knight()); // a black creature
    g.add_card_to_library(0, catalog::swamp());
    g.players[0].mana_pool.add_colorless(2);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: lyz, ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Lyzolda sacrificing a black creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 + 1, "black sacrifice drew a card");
}

/// Stormscale Anarch deals 4 when the discarded card was multicolored.
#[test]
fn stormscale_anarch_multicolor_discard_deals_four() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let anarch = g.add_card_to_battlefield(0, catalog::stormscale_anarch());
    g.clear_sickness(anarch);
    g.add_card_to_hand(0, catalog::lightning_helix()); // a multicolored (R/W) card
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: anarch, ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Stormscale discarding a multicolored card");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 4, "multicolored discard dealt 4");
}

/// Crime reanimates a creature from an opponent's graveyard under your control.
#[test]
fn crime_reanimates_from_opponent_graveyard() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let crime = g.add_card_to_hand(0, catalog::crime_punishment());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: crime, target: Some(Target::Permanent(dead)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Crime");
    drain_stack(&mut g);
    let bear = g.battlefield_find(dead).expect("bear reanimated");
    assert_eq!(bear.controller, 0, "under your control");
}

/// Punishment destroys each artifact/creature/enchantment with mana value X.
#[test]
fn punishment_destroys_mana_value_x() {
    let mut g = two_player_game();
    let two_drop = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let one_drop = g.add_card_to_battlefield(1, catalog::goblin_guide()); // MV 1
    let cp = g.add_card_to_hand(0, catalog::crime_punishment());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSplitRight {
        card_id: cp, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Punishment for X=2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(two_drop).is_none(), "MV-2 creature destroyed");
    assert!(g.battlefield_find(one_drop).is_some(), "MV-1 creature survived");
}

/// Hit makes the target player sacrifice a creature and deals its mana value.
#[test]
fn hit_edict_and_mana_value_damage() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2, the only sac fodder
    let hr = g.add_card_to_hand(0, catalog::hit_run());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: hr, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hit");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().all(|c| c.definition.name != "Grizzly Bears"),
        "the opponent's creature was sacrificed");
    assert_eq!(g.players[1].life, foe_life - 2, "took damage equal to the sacrificed MV");
}

/// Fall makes the target player discard the nonland cards among two revealed at
/// random; a hand of only nonlands loses two.
#[test]
fn fall_discards_nonlands() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::goblin_guide());
    let rf = g.add_card_to_hand(0, catalog::rise_fall());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let h1 = g.players[1].hand.len();
    g.perform_action(GameAction::CastSplitRight {
        card_id: rf, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fall");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1 - 2, "both revealed nonlands were discarded");
}

/// Azorius Ploy: clause 1 prevents all combat damage the first target would
/// *deal* (its blocker survives); clause 2 prevents all combat damage dealt
/// *to* the second target (it survives its blocker's strike).
#[test]
fn azorius_ploy_prevents_both_ways() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // Two 2/2 attackers for player 0.
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(a);
    g.clear_sickness(c);
    // Two 2/2 blockers for player 1.
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let d = g.add_card_to_battlefield(1, catalog::grizzly_bears());

    // slot0 = a (prevent a's outgoing), slot1 = c (prevent incoming to c).
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(c)];
    let evs = g.resolve_effect(&catalog::azorius_ploy().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);

    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: c, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(b, a), (d, c)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(b).is_some(), "a dealt no combat damage — its blocker lives");
    assert!(g.battlefield_find(c).is_some(), "damage to c prevented — c lives");
    assert!(g.battlefield_find(a).is_none(), "a still takes its blocker's damage and dies");
    assert!(g.battlefield_find(d).is_none(), "c still deals damage — its blocker dies");
}

/// Carom redirects the next 1 damage from one creature onto another, and draws.
#[test]
fn carom_redirects_next_damage_and_draws() {
    use crabomination::game::effects::EntityRef;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let hand0 = g.players[0].hand.len();
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
    g.resolve_effect(&catalog::carom().effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "Carom cantrips");
    let mut evs = Vec::new();
    g.deal_damage_to_from(EntityRef::Permanent(a), 3, None, &mut evs);
    assert_eq!(g.battlefield_find(a).unwrap().damage, 2, "1 of 3 redirected away");
    assert_eq!(g.battlefield_find(b).unwrap().damage, 1, "1 redirected onto b");
}

/// Trial returns all creatures blocking or blocked by the target creature.
#[test]
fn trial_bounces_combat_partners() {
    use crabomination::game::types::{Target, TurnStep};
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    // Trial targeting the attacker returns its blocker to hand (the attacker stays).
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Permanent(attacker)];
    let evs = g.resolve_effect(&catalog::trial_error().effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(blocker).is_none(), "blocker returned to hand");
    assert!(g.battlefield_find(attacker).is_some(), "the targeted attacker stays");
    assert_eq!(g.players[1].hand.len(), 1, "blocker in its owner's hand");
}

/// Error counters a multicolored spell on the stack.
#[test]
fn error_counters_only_multicolored() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    // The active player casts a multicolored creature ({2}{G}{U}), then
    // responds to their own spell with Error.
    let multi = g.add_card_to_hand(0, catalog::assault_zeppelid());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: multi, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast multicolored");
    let err = g.add_card_to_hand(0, catalog::trial_error());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSplitRight {
        card_id: err, target: Some(Target::Permanent(multi)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Error");
    drain_stack(&mut g);
    assert!(g.battlefield_find(multi).is_none(), "multicolored spell countered — never resolved");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == multi), "countered spell in graveyard");
}

/// Momir Vig's blue trigger reveals the top card on a blue creature cast and
/// takes it if it's a creature.
#[test]
fn momir_vig_blue_cast_reveals_top_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::momir_vig_simic_visionary());
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::merfolk_of_the_pearl_trident()); // {U} blue creature
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast blue creature");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == top), "revealed top creature went to hand");
}

/// Momir Vig's green trigger tutors a creature to the top of the library.
#[test]
fn momir_vig_green_cast_tutors_to_top() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::momir_vig_simic_visionary());
    let tutored = g.add_card_to_library(0, catalog::phantom_warrior());
    let spell = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G} green creature
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(tutored))]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast green creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(tutored),
        "tutored creature sits on top of the library");
}

/// Sphinx of the Chimes discards two same-named cards to draw four.
#[test]
fn sphinx_of_the_chimes_discard_pair_draws_four() {
    let sphinx = catalog::sphinx_of_the_chimes();
    assert_eq!((sphinx.power, sphinx.toughness), (5, 6));
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::sphinx_of_the_chimes());
    // Two copies of the same nonland card + a distinct card that must stay.
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let keep = g.add_card_to_hand(0, catalog::phantom_warrior());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: s, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate Sphinx");
    drain_stack(&mut g);
    // Drew 4, discarded 2 Bears; the distinct card is untouched.
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 2);
    assert!(g.players[0].hand.iter().any(|c| c.id == keep), "the odd card stays in hand");
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Island").count(), 4, "drew four");
}

/// Elemental Resonance adds mana equal to the enchanted permanent's cost at the
/// start of the controller's first main phase.
#[test]
fn elemental_resonance_ramps_enchanted_cost() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // {1}{G}
    let aura = g.add_card_to_battlefield(0, catalog::elemental_resonance());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    // {1}{G} → one colorless + one green = two mana total.
    assert_eq!(g.players[0].mana_pool.total(), 2, "ramped the enchanted cost");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "green pip from the colored symbol");
}


/// Vigean Intuition (AutoDecider picks Creature) takes creatures from the top
/// four to hand and buries the rest.
#[test]
fn vigean_intuition_partitions_by_type() {
    let mut g = two_player_game();
    // Top four: two creatures, two lands (Creature is the auto-picked type).
    let c1 = g.add_card_to_library(0, catalog::island());
    let c2 = g.add_card_to_library(0, catalog::island());
    let cr1 = g.add_card_to_library(0, catalog::grizzly_bears());
    let cr2 = g.add_card_to_library(0, catalog::phantom_warrior());
    let _ = (c1, c2);
    let spell = g.add_card_to_hand(0, catalog::vigean_intuition());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Vigean Intuition");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == cr1) && g.players[0].hand.iter().any(|c| c.id == cr2),
        "creatures went to hand");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Island").count(), 2,
        "the non-creatures were buried");
}

/// Fertile Imagination makes two Saprolings per matching card in the opponent's
/// hand (AutoDecider picks Creature).
#[test]
fn fertile_imagination_tokens_per_match() {
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::phantom_warrior());
    g.add_card_to_hand(1, catalog::island()); // non-creature, ignored
    let spell = g.add_card_to_hand(0, catalog::fertile_imagination());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fertile Imagination");
    drain_stack(&mut g);
    // Two creatures in hand × 2 = four Saprolings.
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Saproling").count(), 4,
        "two Saprolings per creature revealed");
}

/// Aethermage's Touch flashes a creature onto the battlefield; it returns to
/// hand at the caster's end step.
#[test]
fn aethermages_touch_deploys_then_bounces() {
    let mut g = two_player_game();
    // Top of library: a land then a creature (creature is auto-picked).
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::phantom_warrior());
    let creature = g.add_card_to_library(0, catalog::snapping_drake());
    let _ = creature;
    let spell = g.add_card_to_hand(0, catalog::aethermages_touch());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aethermage's Touch");
    drain_stack(&mut g);
    // Highest-power creature (Snapping Drake, 3/2) hit the battlefield.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Snapping Drake"), "creature deployed");
    // At the end step it returns to hand.
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Snapping Drake"), "returned to hand at end step");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Snapping Drake"), "no longer on the battlefield");
}
