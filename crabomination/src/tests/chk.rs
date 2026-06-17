//! Champions of Kamigawa — Spirit/Arcane bodies: Offering Patrons (tested in
//! `cr_rules`), Bushido, Soulshift, sacrifice/tap activated abilities.

use super::*;
use crate::card::Keyword;
use crate::catalog;
use crate::game::two_player_game;

/// Kami of Ancient Law's "Sacrifice this creature: Destroy target enchantment."
#[test]
fn kami_of_ancient_law_sacrifices_to_destroy_enchantment() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_ancient_law());
    let ench = g.add_card_to_battlefield(1, catalog::concordant_crossroads());
    g.clear_sickness(kami);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(ench)), x_value: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == kami), "Kami sacrificed as a cost");
    assert!(!g.battlefield.iter().any(|c| c.id == ench), "enchantment destroyed");
}

/// Kabuto Moth's "{T}: target creature gets +1/+2 until end of turn."
#[test]
fn kabuto_moth_taps_to_pump() {
    let mut g = two_player_game();
    let moth = g.add_card_to_battlefield(0, catalog::kabuto_moth());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(moth);
    g.perform_action(GameAction::ActivateAbility {
        card_id: moth, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("tap to pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+1/+2 applied");
    assert!(g.battlefield_find(moth).unwrap().tapped, "Moth tapped for the ability");
}

/// Gibbering Kami's Soulshift 3 returns a Spirit with MV ≤ 3 on death.
#[test]
fn gibbering_kami_soulshift_returns_small_spirit() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::gibbering_kami());
    let spirit = g.add_card_to_graveyard(0, catalog::kami_of_ancient_law()); // Spirit, MV 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let events = g.remove_to_graveyard_with_triggers(kami);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit), "soulshift returned the Spirit");
}

/// Kitsune Blademaster carries First strike + Bushido 1.
#[test]
fn kitsune_blademaster_has_first_strike_and_bushido() {
    let d = catalog::kitsune_blademaster();
    assert!(d.keywords.contains(&Keyword::FirstStrike));
    assert!(d.keywords.contains(&Keyword::Bushido(1)));
}

/// Kodama of the North Tree is a 6/4 with Trample + Shroud.
#[test]
fn kodama_of_the_north_tree_trample_shroud() {
    let d = catalog::kodama_of_the_north_tree();
    assert_eq!((d.power, d.toughness), (6, 4));
    assert!(d.keywords.contains(&Keyword::Trample) && d.keywords.contains(&Keyword::Shroud));
}

/// Spiritcraft: casting an Arcane spell triggers Kami of Fire's Roar, making
/// a target creature unable to block.
#[test]
fn kami_of_fires_roar_spiritcraft_on_arcane_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kami_of_fires_roar());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists()); // {U} Arcane
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::CantBlock),
        "spiritcraft granted CantBlock to the opponent's creature");
}

/// Nezumi Cutthroat has Fear and can't block.
#[test]
fn nezumi_cutthroat_fear_cant_block() {
    let d = catalog::nezumi_cutthroat();
    assert!(d.keywords.contains(&Keyword::Fear) && d.keywords.contains(&Keyword::CantBlock));
}

/// Befoul destroys a land.
#[test]
fn befoul_destroys_a_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::befoul());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(land)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Befoul on a land");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == land), "land destroyed");
}

/// Rend Flesh destroys a non-Spirit creature but can't target a Spirit.
#[test]
fn rend_flesh_only_hits_non_spirits() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spirit = g.add_card_to_battlefield(1, catalog::river_kaijin()); // Spirit
    let id = g.add_card_to_hand(0, catalog::rend_flesh());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(spirit)),
        additional_targets: vec![], mode: None, x_value: None,
    }).is_err(), "can't target a Spirit");
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("destroy a non-Spirit");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "non-Spirit destroyed");
}

/// Yamabushi's Flame exiles a creature it would kill instead of it dying.
#[test]
fn yamabushis_flame_exiles_what_it_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::yamabushis_flame());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("burn the bear");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "bear is gone");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "exiled, not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear exiled");
}

/// Kami of Twisted Reflection sacrifices itself to bounce your own creature.
#[test]
fn kami_of_twisted_reflection_bounces_your_creature() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_twisted_reflection());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(kami);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: None,
    }).expect("sac to bounce");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == kami), "Kami sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "your creature returned to hand");
}

/// Patron of the Moon's {1} ability puts up to two lands from hand onto the
/// battlefield tapped.
#[test]
fn patron_of_the_moon_ramps_lands_from_hand() {
    let mut g = two_player_game();
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let patron = g.add_card_to_battlefield(0, catalog::patron_of_the_moon());
    g.clear_sickness(patron);
    let l1 = g.add_card_to_hand(0, catalog::island());
    let l2 = g.add_card_to_hand(0, catalog::island());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![l1, l2])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: patron, ability_index: 0, target: None, x_value: None,
    }).expect("activate land ramp");
    drain_stack(&mut g);
    assert!(g.battlefield_find(l1).is_some_and(|c| c.tapped), "land 1 entered tapped");
    assert!(g.battlefield_find(l2).is_some_and(|c| c.tapped), "land 2 entered tapped");
}

/// Patron of the Orochi's {T} ability untaps Forests and green creatures.
#[test]
fn patron_of_the_orochi_untaps_forests_and_green() {
    let mut g = two_player_game();
    let patron = g.add_card_to_battlefield(0, catalog::patron_of_the_orochi());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    g.clear_sickness(patron);
    g.battlefield_find_mut(forest).unwrap().tapped = true;
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: patron, ability_index: 0, target: None, x_value: None,
    }).expect("activate untap");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(forest).unwrap().tapped, "Forest untapped");
    assert!(!g.battlefield_find(bear).unwrap().tapped, "green creature untapped");
}

/// Orochi Ranger's combat-damage trigger taps the damaged creature and stops
/// its next untap (`Effect::SkipNextUntap`).
#[test]
fn orochi_ranger_tap_lock_trigger() {
    let mut g = two_player_game();
    let ranger = g.add_card_to_battlefield(0, catalog::orochi_ranger());
    let wall = g.add_card_to_battlefield(1, catalog::kami_of_old_stone());
    let trig = catalog::orochi_ranger().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        ranger, 0, Some(Target::Permanent(wall)), 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    let w = g.battlefield_find(wall).unwrap();
    assert!(w.tapped, "damaged creature tapped");
    assert!(w.skip_next_untap, "and it won't untap next turn");
}

/// Hideous Laughter shrinks every creature -2/-2, killing the small ones.
#[test]
fn hideous_laughter_wraths_small_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → 0/0 dies
    let wall = g.add_card_to_battlefield(1, catalog::kami_of_old_stone()); // 1/7 survives
    let id = g.add_card_to_hand(0, catalog::hideous_laughter());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hideous Laughter");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2/2 died to -2/-2");
    assert!(g.battlefield.iter().any(|c| c.id == wall), "1/7 survived");
}

/// Yamabushi's Storm pings every creature for 1 and exiles what it kills.
#[test]
fn yamabushis_storm_pings_and_exiles() {
    let mut g = two_player_game();
    let token = g.add_card_to_battlefield(1, catalog::lantern_kami()); // 1/1
    let id = g.add_card_to_hand(0, catalog::yamabushis_storm());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Yamabushi's Storm");
    drain_stack(&mut g);
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == token), "exiled, not graveyard");
    assert!(g.exile.iter().any(|c| c.id == token), "1/1 exiled by the storm");
}

/// Vigilance aura grants vigilance to the enchanted creature.
#[test]
fn vigilance_aura_grants_vigilance() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vigilance_aura());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("enchant the bear");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance));
}

/// Kitsune Diviner taps a target Spirit.
#[test]
fn kitsune_diviner_taps_a_spirit() {
    let mut g = two_player_game();
    let diviner = g.add_card_to_battlefield(0, catalog::kitsune_diviner());
    let spirit = g.add_card_to_battlefield(1, catalog::lantern_kami());
    g.clear_sickness(diviner);
    g.perform_action(GameAction::ActivateAbility {
        card_id: diviner, ability_index: 0, target: Some(Target::Permanent(spirit)), x_value: None,
    }).expect("tap the Spirit");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spirit).unwrap().tapped, "Spirit tapped");
}

/// Sire of the Storm draws when you cast a Spirit or Arcane spell.
#[test]
fn sire_of_the_storm_spiritcraft_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sire_of_the_storm());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane");
    drain_stack(&mut g);
    // -1 spell out of hand, +1 Reach Through Mists draw, +1 spiritcraft draw.
    assert_eq!(g.players[0].hand.len(), hand_before - 1 + 1 + 1, "spiritcraft drew a card");
}

/// Soulshift / evasion keywords on the late-batch spirits.
#[test]
fn kamigawa_spirit_keywords() {
    let lunacy = catalog::kami_of_lunacy();
    assert!(lunacy.keywords.contains(&Keyword::Flying));
    assert_eq!((lunacy.power, lunacy.toughness), (4, 1));
    assert!(catalog::venerable_kumo().keywords.contains(&Keyword::Reach));
    let nagao = catalog::nagao_bound_by_honor();
    assert!(nagao.keywords.contains(&Keyword::Bushido(1)));
    assert_eq!(catalog::kami_of_old_stone().toughness, 7);
}

/// Kami of the Hunt pumps itself when you cast an Arcane spell (spiritcraft).
#[test]
fn kami_of_the_hunt_spiritcraft_self_pump() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_the_hunt()); // 2/2
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    let cp = g.computed_permanent(kami).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "spiritcraft +1/+1");
}

/// Soilshaper animates a land into a 3/3 when you cast an Arcane spell.
#[test]
fn soilshaper_spiritcraft_animates_a_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soilshaper());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    let cp = g.computed_permanent(land).unwrap();
    assert!(cp.card_types.contains(&crate::card::CardType::Creature), "land is now a creature");
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Pain Kami sacrifices itself to deal X damage to a creature.
#[test]
fn pain_kami_x_sacrifice_burn() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::pain_kami());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(kami);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), x_value: Some(2),
    }).expect("X=2 sac burn");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == kami), "Pain Kami sacrificed");
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "2 damage killed the 2/2");
}

/// The Samurai cycle carries Bushido (plus their evasion).
#[test]
fn kamigawa_samurai_keywords() {
    assert!(catalog::devoted_retainer().keywords.contains(&Keyword::Bushido(1)));
    let ronin = catalog::ronin_houndmaster();
    assert!(ronin.keywords.contains(&Keyword::Haste) && ronin.keywords.contains(&Keyword::Bushido(1)));
    let moth = catalog::mothrider_samurai();
    assert!(moth.keywords.contains(&Keyword::Flying) && moth.keywords.contains(&Keyword::Bushido(1)));
}

/// Sokenzan Bruiser / Moss Kami carry their evasion / combat keywords.
#[test]
fn kamigawa_vanilla_keywords() {
    assert!(catalog::sokenzan_bruiser().keywords
        .contains(&Keyword::Landwalk(crate::card::LandType::Mountain)));
    assert!(catalog::moss_kami().keywords.contains(&Keyword::Trample));
    let numai = catalog::numai_outcast();
    assert!(numai.keywords.contains(&Keyword::Bushido(2)));
    assert_eq!(numai.activated_abilities.len(), 1, "regenerate ability present");
}

// ── Legendary Dragon Spirits + Honden cycle (modern_decks batch) ─────────────

/// Konda is indestructible with Bushido 5.
#[test]
fn konda_keywords() {
    let k = catalog::konda_lord_of_eiganjo();
    assert!(k.keywords.contains(&Keyword::Indestructible));
    assert!(k.keywords.contains(&Keyword::Vigilance));
    assert!(k.keywords.contains(&Keyword::Bushido(5)));
}

/// Keiga's death trigger gains control of a target creature.
#[test]
fn keiga_dies_steals_creature() {
    let mut g = two_player_game();
    let keiga = g.add_card_to_battlefield(0, catalog::keiga_the_tide_star());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let events = g.remove_to_graveyard_with_triggers(keiga);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "Keiga stole the bear");
}

/// Jugan distributes five +1/+1 counters among target creatures
/// (`Effect::DistributeCounters`).
#[test]
fn jugan_dies_distributes_five_counters() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let jugan = g.add_card_to_battlefield(0, catalog::jugan_the_rising_star());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let events = g.remove_to_graveyard_with_triggers(jugan);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let total = g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne)
        + g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(total, 5, "all five +1/+1 counters distributed");
}

/// Ryusei's death trigger deals 5 damage to each creature without flying,
/// sparing flyers.
#[test]
fn ryusei_dies_burns_nonflyers() {
    let mut g = two_player_game();
    let ryusei = g.add_card_to_battlefield(0, catalog::ryusei_the_falling_star());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no fly
    let flyer = g.add_card_to_battlefield(1, catalog::mothrider_samurai()); // 2/2 flying
    let events = g.remove_to_graveyard_with_triggers(ryusei);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ground).is_none(), "ground creature took 5 and died");
    assert!(g.battlefield_find(flyer).is_some(), "flyer was spared");
}

/// Meloku bounces a land you control to mint a 1/1 flying Illusion.
#[test]
fn meloku_bounces_land_for_illusion() {
    let mut g = two_player_game();
    let meloku = g.add_card_to_battlefield(0, catalog::meloku_the_clouded_mirror());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(meloku);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: meloku, ability_index: 0, target: None, x_value: None,
    }).expect("activate Meloku");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land returned to hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "land in hand");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Illusion"),
        "1/1 flying Illusion minted");
}

/// Hana Kami sacrifices itself to return an Arcane card from your graveyard.
#[test]
fn hana_kami_returns_arcane() {
    let mut g = two_player_game();
    let hana = g.add_card_to_battlefield(0, catalog::hana_kami());
    let ray = g.add_card_to_graveyard(0, catalog::glacial_ray()); // Arcane instant
    g.clear_sickness(hana);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hana, ability_index: 0, target: Some(Target::Permanent(ray)), x_value: None,
    }).expect("activate Hana Kami");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hana).is_none(), "Hana Kami sacrificed");
    assert!(g.players[0].hand.iter().any(|c| c.id == ray), "Arcane card returned to hand");
}

/// Honden of Cleansing Fire's upkeep trigger gains 2 life per Shrine.
#[test]
fn honden_cleansing_fire_scales_with_shrines() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::honden_of_cleansing_fire());
    g.add_card_to_battlefield(0, catalog::honden_of_seeing_winds()); // second Shrine
    let honden = g.battlefield.iter().find(|c| c.definition.name == "Honden of Cleansing Fire").unwrap().id;
    let start = g.players[0].life;
    let trig = catalog::honden_of_cleansing_fire().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(honden, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[0].life - start, 4, "2 life × 2 Shrines");
}

/// Kami of the Crescent Moon draws each player an extra card at their draw step.
#[test]
fn crescent_moon_extra_draw() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_the_crescent_moon());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    let trig = catalog::kami_of_the_crescent_moon().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(kami, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), before + 1, "active player drew an extra card");
}
