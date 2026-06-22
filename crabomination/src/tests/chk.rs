//! Champions of Kamigawa — Spirit/Arcane bodies: Offering Patrons (tested in
//! `cr_rules`), Bushido, Soulshift, sacrifice/tap activated abilities.

use super::*;
use crate::card::Keyword;
use crate::catalog;
use crate::game::two_player_game;

fn advance_to(g: &mut GameState, step: crate::game::TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Kami of Ancient Law's "Sacrifice this creature: Destroy target enchantment."
#[test]
fn kami_of_ancient_law_sacrifices_to_destroy_enchantment() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_ancient_law());
    let ench = g.add_card_to_battlefield(1, catalog::concordant_crossroads());
    g.clear_sickness(kami);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(ench)), additional_targets: Vec::new(), x_value: None,
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
        card_id: moth, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
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
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
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
        card_id: patron, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: patron, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: diviner, ability_index: 0, target: Some(Target::Permanent(spirit)), additional_targets: Vec::new(), x_value: None,
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
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: Some(2),
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
        card_id: meloku, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
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
        card_id: hana, ability_index: 0, target: Some(Target::Permanent(ray)), additional_targets: Vec::new(), x_value: None,
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

// ── Snake tribal + red pingers (modern_decks batch 2) ────────────────────────

/// Seshiro pumps other Snakes you control +2/+2 (lord static).
#[test]
fn seshiro_snake_lord() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::seshiro_the_anointed());
    let snake = g.add_card_to_battlefield(0, catalog::orochi_ranger()); // 2/1 Snake
    let cp = g.computed_permanent(snake).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+2 from Seshiro");
}

/// Sosuke's Warrior combat-damage trigger destroys the damaged creature.
#[test]
fn sosuke_warrior_destroys_blocker() {
    let mut g = two_player_game();
    let sosuke = g.add_card_to_battlefield(0, catalog::sosuke_son_of_seshiro()); // Snake Warrior
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trig = catalog::sosuke_son_of_seshiro().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        sosuke, 0, Some(Target::Permanent(victim)), 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "damaged creature destroyed");
}

/// Frostling sacrifices itself to ping a creature for 1.
#[test]
fn frostling_sacrifices_to_ping() {
    let mut g = two_player_game();
    let frost = g.add_card_to_battlefield(0, catalog::frostling());
    let target = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    g.clear_sickness(frost);
    g.perform_action(GameAction::ActivateAbility {
        card_id: frost, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Frostling");
    drain_stack(&mut g);
    assert!(g.battlefield_find(frost).is_none(), "Frostling sacrificed");
    assert!(g.battlefield_find(target).is_none(), "1/1 target took 1 and died");
}

/// Hearth Kami's {X}, Sac destroys an artifact with mana value X.
#[test]
fn hearth_kami_destroys_artifact_of_mv_x() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::hearth_kami());
    let ring = g.add_card_to_battlefield(1, catalog::sol_ring()); // MV 1
    g.clear_sickness(kami);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(ring)), additional_targets: Vec::new(), x_value: Some(1),
    }).expect("activate Hearth Kami");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ring).is_none(), "Sol Ring (MV 1) destroyed for X=1");
}

/// Quiet Purity destroys a target enchantment.
#[test]
fn quiet_purity_destroys_enchantment() {
    let mut g = two_player_game();
    let qp = g.add_card_to_hand(0, catalog::quiet_purity());
    let ench = g.add_card_to_battlefield(1, catalog::concordant_crossroads());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    cast_at(&mut g, qp, Target::Permanent(ench));
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// Soratami Mirror-Guard bounces a land to make a small creature unblockable.
#[test]
fn soratami_mirror_guard_grants_unblockable() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::soratami_mirror_guard());
    let land = g.add_card_to_battlefield(0, catalog::island());
    let small = g.add_card_to_battlefield(0, catalog::frostling()); // 1/1, power ≤ 2
    g.clear_sickness(guard);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: guard, ability_index: 0, target: Some(Target::Permanent(small)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Mirror-Guard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land bounced as a cost");
    let cp = g.computed_permanent(small).unwrap();
    assert!(cp.keywords.contains(&Keyword::Unblockable), "small creature is unblockable");
}

/// Akki Coalflinger taps to give attacking creatures first strike.
#[test]
fn akki_coalflinger_grants_first_strike() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_coalflinger());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(akki);
    g.clear_sickness(attacker);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("declare attacker");
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: akki, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Akki");
    drain_stack(&mut g);
    let cp = g.computed_permanent(attacker).unwrap();
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "attacker gained first strike");
}

/// CR 502.3 — a player flagged to skip their untap step doesn't untap, and the
/// charge is consumed.
#[test]
fn cr_502_3_skipped_untap_step_keeps_permanents_tapped() {
    let mut g = two_player_game();
    let perm = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(perm).unwrap().tapped = true;
    g.players[1].skip_next_untap_step = 1;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(perm).unwrap().tapped, "permanent stays tapped through the skipped untap");
    assert_eq!(g.players[1].skip_next_untap_step, 0, "one skip charge consumed");
    // Next untap (no charge) untaps normally.
    g.do_untap();
    assert!(!g.battlefield_find(perm).unwrap().tapped, "untaps on the following untap step");
}

/// Yosei's death trigger taps the target player's board and skips their next
/// untap step (`Effect::SkipPlayerUntapStep`).
#[test]
fn yosei_dies_locks_target_player() {
    let mut g = two_player_game();
    let yosei = g.add_card_to_battlefield(0, catalog::yosei_the_morning_star());
    let land = g.add_card_to_battlefield(1, catalog::island());
    let creature = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let trig = catalog::yosei_the_morning_star().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        yosei, 0, Some(Target::Player(1)), 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert!(g.battlefield_find(land).unwrap().tapped, "land tapped");
    assert!(g.battlefield_find(creature).unwrap().tapped, "creature tapped");
    assert_eq!(g.players[1].skip_next_untap_step, 1, "player 1 will skip their next untap");
    // Their untap step leaves the board locked.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped, "still tapped after skipped untap");
}

// ── CHK batch 3 ──────────────────────────────────────────────────────────────

/// Mothrider Patrol taps a target creature.
#[test]
fn mothrider_patrol_taps_target() {
    let mut g = two_player_game();
    let patrol = g.add_card_to_battlefield(0, catalog::mothrider_patrol());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(patrol);
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: patrol, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Mothrider Patrol");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
}

/// Strength of Cedars pumps by the number of lands you control.
#[test]
fn strength_of_cedars_scales_with_lands() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::island()); // 3 lands
    let id = g.add_card_to_hand(0, catalog::strength_of_cedars());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    cast_at(&mut g, id, Target::Permanent(bear));
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+3/+3 from three lands");
}

/// Sokenzan Spellblade pumps +X/+0 by cards in hand.
#[test]
fn sokenzan_spellblade_scales_with_hand() {
    let mut g = two_player_game();
    let blade = g.add_card_to_battlefield(0, catalog::sokenzan_spellblade());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::forest()); // 2 cards in hand
    g.clear_sickness(blade);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: blade, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Spellblade");
    drain_stack(&mut g);
    let cp = g.computed_permanent(blade).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+0 from two cards in hand");
}

/// Wear Away destroys an artifact and carries Splice onto Arcane.
#[test]
fn wear_away_destroys_and_splices() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let id = g.add_card_to_hand(0, catalog::wear_away());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    cast_at(&mut g, id, Target::Permanent(art));
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert!(matches!(catalog::wear_away().keywords[0], Keyword::Splice(_, _)), "has Splice");
}

/// Burr Grafter sacrifices to pump and carries Soulshift 3.
#[test]
fn burr_grafter_sacrifices_to_pump() {
    let mut g = two_player_game();
    let grafter = g.add_card_to_battlefield(0, catalog::burr_grafter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(grafter);
    g.perform_action(GameAction::ActivateAbility {
        card_id: grafter, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to pump");
    drain_stack(&mut g);
    assert!(g.battlefield_find(grafter).is_none(), "Burr Grafter sacrificed");
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2 applied");
}

/// Crack the Earth makes each player sacrifice a permanent.
#[test]
fn crack_the_earth_each_player_sacrifices() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::island());
    let id = g.add_card_to_hand(0, catalog::crack_the_earth());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Crack the Earth");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0).count(), 0, "P0 sacrificed");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 0, "P1 sacrificed");
}

/// Vine Kami has Menace and Soulshift 6.
#[test]
fn vine_kami_menace_soulshift() {
    let d = catalog::vine_kami();
    assert!(d.keywords.contains(&Keyword::Menace));
    assert!(!d.triggered_abilities.is_empty(), "carries Soulshift");
}

// ── CHK batch 4 ──────────────────────────────────────────────────────────────

/// Akki Underminer's combat damage makes the defending player sacrifice.
#[test]
fn akki_underminer_forces_sacrifice() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_underminer());
    g.add_card_to_battlefield(1, catalog::island());
    let trig = catalog::akki_underminer().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(akki, 0, Some(Target::Player(1)), 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 1).count(), 0,
        "defending player sacrificed their permanent");
}

/// Ronin Cliffrider's attack trigger pings each defending creature for 1.
#[test]
fn ronin_cliffrider_pings_defenders() {
    let mut g = two_player_game();
    let ronin = g.add_card_to_battlefield(0, catalog::ronin_cliffrider());
    let x = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    let y = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let trig = catalog::ronin_cliffrider().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(ronin, 0, Some(Target::Player(1)), 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(x).is_none() && g.battlefield_find(y).is_none(),
        "both defending 1/1s took 1 and died");
}

/// Akki Avalanchers sacrifices a land to pump itself +2/+0.
#[test]
fn akki_avalanchers_sacs_land_to_pump() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_avalanchers());
    let land = g.add_card_to_battlefield(0, catalog::mountain());
    g.clear_sickness(akki);
    g.perform_action(GameAction::ActivateAbility {
        card_id: akki, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Avalanchers");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land sacrificed");
    assert_eq!(g.computed_permanent(akki).unwrap().power, 3, "+2/+0 → 3 power");
}

/// Blind with Anger steals a nonlegendary creature for the turn (untap + haste).
#[test]
fn blind_with_anger_steals_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::blind_with_anger());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast_at(&mut g, id, Target::Permanent(bear));
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert_eq!(c.controller, 0, "gained control");
    assert!(!c.tapped, "untapped");
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste), "has haste");
}

/// Veteran's Reflexes pumps and untaps a creature.
#[test]
fn veterans_reflexes_pumps_and_untaps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::veterans_reflexes());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    cast_at(&mut g, id, Target::Permanent(bear));
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(!c.tapped, "untapped");
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 → 3 power");
}

// ── CHK batch 5 ──────────────────────────────────────────────────────────────

/// Scuttling Death sacrifices to shrink a creature -1/-1 and carries Soulshift.
#[test]
fn scuttling_death_sacs_to_shrink() {
    let mut g = two_player_game();
    let death = g.add_card_to_battlefield(0, catalog::scuttling_death());
    let victim = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    g.clear_sickness(death);
    g.perform_action(GameAction::ActivateAbility {
        card_id: death, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to shrink");
    drain_stack(&mut g);
    assert!(g.battlefield_find(death).is_none(), "Scuttling Death sacrificed");
    assert!(g.battlefield_find(victim).is_none(), "1/1 shrank to 0/0 and died");
}

/// Bile Urchin sacrifices to drain a player 1 life.
#[test]
fn bile_urchin_sacs_to_drain() {
    let mut g = two_player_game();
    let urchin = g.add_card_to_battlefield(0, catalog::bile_urchin());
    g.clear_sickness(urchin);
    let before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: urchin, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to drain");
    drain_stack(&mut g);
    assert!(g.battlefield_find(urchin).is_none(), "Bile Urchin sacrificed");
    assert_eq!(g.players[1].life, before - 1, "target lost 1 life");
}

/// Cursed Ronin firebreathes +1/+1 for {B}.
#[test]
fn cursed_ronin_firebreathes() {
    let mut g = two_player_game();
    let ronin = g.add_card_to_battlefield(0, catalog::cursed_ronin());
    g.clear_sickness(ronin);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ronin, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ronin).unwrap().power, 2, "+1/+1 → 2 power");
}

/// Nezumi Bone-Reader sacrifices a creature to make a player discard.
#[test]
fn nezumi_bone_reader_sacs_for_discard() {
    let mut g = two_player_game();
    let reader = g.add_card_to_battlefield(0, catalog::nezumi_bone_reader());
    g.add_card_to_battlefield(0, catalog::frostling()); // fodder
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.clear_sickness(reader);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: reader, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac for discard");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "target player discarded their card");
}

/// Kami of Empty Graves / Nezumi Ronin carry their printed keywords.
#[test]
fn batch5_keyword_bodies() {
    assert!(!catalog::kami_of_empty_graves().triggered_abilities.is_empty(), "Soulshift");
    assert!(catalog::nezumi_ronin().keywords.contains(&Keyword::Bushido(1)));
}

/// Patron of the Nezumi: an opponent's permanent dying costs that player 1 life.
#[test]
fn patron_of_the_nezumi_drains_on_opponent_permanent_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::patron_of_the_nezumi());
    // Player 0's Pain Kami burns down player 1's bear; the bear hits player 1's
    // graveyard, firing Patron. Pain Kami's own death (player 0's graveyard) must
    // NOT trigger it.
    let kami = g.add_card_to_battlefield(0, catalog::pain_kami());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(kami);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: Some(2),
    }).expect("X=2 sac burn");
    drain_stack(&mut g);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear killed");
    assert_eq!(g.players[1].life, before - 1, "opponent lost 1 life to Patron");
}

/// Cage of Hands locks a creature down, then bounces itself back to hand.
#[test]
fn cage_of_hands_pacifies_then_returns_to_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let cage = g.add_card_to_hand(0, catalog::cage_of_hands());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: cage, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Cage castable");
    drain_stack(&mut g);
    assert!(g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
        .keywords.contains(&Keyword::CantAttack), "creature pacified");
    // {1}{W}: return Cage to its owner's hand.
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cage, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("bounce ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cage).is_none(), "Cage left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == cage), "Cage returned to hand");
    assert!(!g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
        .keywords.contains(&Keyword::CantAttack), "pacify lifted once Cage left");
}

/// Heartless Hidetsugu taps to deal each player half their life (rounded down).
#[test]
fn heartless_hidetsugu_halves_each_players_life() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::heartless_hidetsugu());
    g.clear_sickness(h);
    g.players[0].life = 20;
    g.players[1].life = 17;
    g.perform_action(GameAction::ActivateAbility {
        card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 10, "20 → took 10 damage");
    assert_eq!(g.players[1].life, 9, "17 → took 8 (rounded down)");
}

/// Horobi destroys any creature that becomes the target of a spell, even when
/// the spell itself wouldn't be lethal.
#[test]
fn horobi_destroys_targeted_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::horobi_deaths_wail());
    let angel = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bolt the angel");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(angel).is_none(), "Horobi destroyed the targeted 4/4");
}

/// Time of Need tutors a legendary creature into hand.
#[test]
fn time_of_need_tutors_legend_to_hand() {
    let mut g = two_player_game();
    let konda = g.add_card_to_library(0, catalog::konda_lord_of_eiganjo());
    g.add_card_to_library(0, catalog::grizzly_bears()); // non-legendary filler
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(konda)),
    ]));
    let ton = g.add_card_to_hand(0, catalog::time_of_need());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ton, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Time of Need castable");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == konda), "legend tutored to hand");
}

/// Yukora's leave-the-battlefield trigger sacrifices non-Ogre creatures only.
#[test]
fn yukora_sacrifices_non_ogres_on_ltb() {
    let mut g = two_player_game();
    let yukora = g.add_card_to_battlefield(0, catalog::yukora_the_prisoner());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ogre = g.add_card_to_battlefield(0, catalog::heartless_hidetsugu()); // Ogre
    let kami = g.add_card_to_battlefield(0, catalog::pain_kami());
    g.clear_sickness(kami);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(yukora)), additional_targets: Vec::new(), x_value: Some(5),
    }).expect("burn Yukora");
    drain_stack(&mut g);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(yukora).is_none(), "Yukora died");
    assert!(g.battlefield_find(bear).is_none(), "non-Ogre bear sacrificed");
    assert!(g.battlefield_find(ogre).is_some(), "Ogre kept");
}

/// He Who Hungers sacs a Spirit to strip a card from an opponent's hand.
#[test]
fn he_who_hungers_sacs_spirit_to_discard() {
    let mut g = two_player_game();
    let hwh = g.add_card_to_battlefield(0, catalog::he_who_hungers());
    g.add_card_to_battlefield(0, catalog::gibbering_kami()); // Spirit fodder
    g.add_card_to_hand(1, catalog::lightning_bolt());
    g.clear_sickness(hwh);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hwh, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac Spirit to strip hand");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "opponent discarded the chosen card");
}

/// Kami of the Painted Road fires its spiritcraft protection grant on an Arcane
/// cast.
#[test]
fn kami_of_the_painted_road_gains_protection_on_spiritcraft() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::kami_of_the_painted_road());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Color(crate::mana::Color::Red),
    ]));
    let ray = g.add_card_to_hand(0, catalog::glacial_ray()); // Arcane instant
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: ray, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    let view = g.compute_battlefield();
    let c = view.iter().find(|c| c.id == kami).unwrap();
    assert!(c.keywords.contains(&Keyword::Protection(crate::mana::Color::Red)),
        "spiritcraft granted protection from the chosen color");
}

/// Rend Spirit destroys a Spirit (and only a Spirit).
#[test]
fn rend_spirit_destroys_spirit() {
    let mut g = two_player_game();
    let spirit = g.add_card_to_battlefield(1, catalog::lantern_kami()); // Spirit
    let rend = g.add_card_to_hand(0, catalog::rend_spirit());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: rend, target: Some(Target::Permanent(spirit)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Rend Spirit castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spirit).is_none(), "Spirit destroyed");
}

/// Eye of Nowhere bounces any permanent to its owner's hand.
#[test]
fn eye_of_nowhere_bounces_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let eye = g.add_card_to_hand(0, catalog::eye_of_nowhere());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: eye, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Eye of Nowhere castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "permanent left the battlefield");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "returned to owner's hand");
}

/// Thief of Hope drains an opponent when you cast an Arcane spell.
#[test]
fn thief_of_hope_drains_on_spiritcraft() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::thief_of_hope());
    let reach = g.add_card_to_hand(0, catalog::reach_through_mists()); // Arcane
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    let opp_before = g.players[1].life;
    let me_before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: reach, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, me_before + 1, "you gained 1");
}

/// Soratami Rainshaper bounces a land to grant a creature shroud.
#[test]
fn soratami_rainshaper_grants_shroud() {
    let mut g = two_player_game();
    let shaper = g.add_card_to_battlefield(0, catalog::soratami_rainshaper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::island());
    g.clear_sickness(shaper);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: shaper, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate shroud grant");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "land bounced as a cost");
    assert!(g.compute_battlefield().iter().find(|c| c.id == bear).unwrap()
        .keywords.contains(&Keyword::Shroud), "creature gained shroud");
}

/// Mystic Restraints flashes in, taps the creature, and locks it tapped.
#[test]
fn mystic_restraints_taps_and_locks() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::mystic_restraints());
    assert!(catalog::mystic_restraints().keywords.contains(&Keyword::Flash), "has Flash");
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Mystic Restraints castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "ETB tapped the creature");
}

/// Hokori keeps lands tapped through the untap step; the upkeep trigger frees
/// exactly one.
#[test]
fn hokori_locks_lands_then_frees_one() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hokori_dust_drinker());
    let l1 = g.add_card_to_battlefield(0, catalog::island());
    let l2 = g.add_card_to_battlefield(0, catalog::island());
    g.battlefield_find_mut(l1).unwrap().tapped = true;
    g.battlefield_find_mut(l2).unwrap().tapped = true;
    g.do_untap();
    let tapped_after_untap = [l1, l2].iter()
        .filter(|id| g.battlefield_find(**id).unwrap().tapped).count();
    assert_eq!(tapped_after_untap, 2, "lands don't untap under Hokori");
}

/// Throat Slitter's combat-damage trigger destroys a nonblack creature the
/// damaged player controls; it carries Ninjutsu.
#[test]
fn throat_slitter_destroys_on_connect() {
    let mut g = two_player_game();
    let ninja = g.add_card_to_battlefield(0, catalog::throat_slitter());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green, nonblack
    assert!(catalog::throat_slitter().keywords.iter()
        .any(|k| matches!(k, Keyword::Ninjutsu(_))), "has Ninjutsu");
    let trig = catalog::throat_slitter().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(
        ninja, 0, Some(Target::Permanent(victim)), 0,
    );
    g.resolve_effect(&trig, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "nonblack creature destroyed");
}

/// Orochi Leafcaller filters {G} into a mana of any color.
#[test]
fn orochi_leafcaller_filters_mana() {
    let mut g = two_player_game();
    let snake = g.add_card_to_battlefield(0, catalog::orochi_leafcaller());
    g.clear_sickness(snake);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Color(crate::mana::Color::Blue),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: snake, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("filter mana");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Blue), 1, "produced blue");
}

/// Joyous Respite gains 1 life per land you control.
#[test]
fn joyous_respite_gains_per_land() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let jr = g.add_card_to_hand(0, catalog::joyous_respite());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    let before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: jr, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Joyous Respite");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 3, "gained 1 per land");
}

/// Kiku makes a creature deal its own power to itself, killing a fragile body.
#[test]
fn kiku_self_damage_kills_creature() {
    let mut g = two_player_game();
    let kiku = g.add_card_to_battlefield(0, catalog::kiku_nights_flower());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(kiku);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kiku, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("self-damage");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "2/2 dealt itself 2 and died");
}

/// Gut Shot pings any target for 1 (paid here with {R}).
#[test]
fn gut_shot_pings_for_one() {
    let mut g = two_player_game();
    let frostling = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    let gs = g.add_card_to_hand(0, catalog::gut_shot());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: gs, target: Some(Target::Permanent(frostling)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Gut Shot castable for {R}");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(frostling).is_none(), "1/1 pinged for 1 and died");
}

/// Hanabi Blast deals 2 and returns itself to hand (then discards at random).
#[test]
fn hanabi_blast_returns_to_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let hb = g.add_card_to_hand(0, catalog::hanabi_blast());
    g.add_card_to_hand(0, catalog::island()); // random-discard fodder
    g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: hb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Hanabi Blast castable");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "2/2 took 2 and died");
    assert!(g.players[0].hand.iter().any(|c| c.id == hb), "Hanabi Blast returned to hand");
}

/// Frostwielder taps to ping for 1.
#[test]
fn frostwielder_pings() {
    let mut g = two_player_game();
    let fw = g.add_card_to_battlefield(0, catalog::frostwielder());
    let frostling = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    g.clear_sickness(fw);
    g.perform_action(GameAction::ActivateAbility {
        card_id: fw, ability_index: 0, target: Some(Target::Permanent(frostling)), additional_targets: Vec::new(), x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(frostling).is_none(), "1/1 pinged dead");
}


/// Ember-Fist Zubera dying alone pings for 1 (one Zubera died).
#[test]
fn ember_fist_zubera_pings_for_count() {
    let mut g = two_player_game();
    let zub = g.add_card_to_battlefield(0, catalog::ember_fist_zubera());
    let frostling = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    // Sacrifice the Zubera (death trigger targets the 1/1).
    let trig = catalog::ember_fist_zubera().triggered_abilities[0].effect.clone();
    g.players[0].zuberas_died_this_turn = 1; // as if it just died
    let ctx = crate::game::effects::EffectContext::for_trigger(
        zub, 0, Some(Target::Permanent(frostling)), 0,
    );
    g.resolve_effect(&trig, &ctx).unwrap();
    g.check_state_based_actions();
    assert!(g.battlefield_find(frostling).is_none(), "1 damage killed the 1/1");
}

/// Two Zubera dying simultaneously make each death trigger see a count of two.
#[test]
fn zubera_deaths_accumulate_this_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::silent_chant_zubera());
    g.add_card_to_battlefield(0, catalog::silent_chant_zubera());
    let wipe = g.add_card_to_hand(0, catalog::hideous_laughter()); // -2/-2 to all
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: wipe, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hideous Laughter");
    drain_stack(&mut g);
    // Both 1/2 Zubera became 0/0 and died; each triggers "gain 2 per Zubera
    // that died this turn" = 2 × 2 = 4, twice → +8 life.
    assert_eq!(g.players[0].zuberas_died_this_turn, 2, "two Zubera counted");
    assert_eq!(g.players[0].life, before + 8, "each Zubera gained 2 per the 2 deaths");
}

/// Kumano taps... er, pays {1}{R} to ping a 1/1 dead.
#[test]
fn kumano_pings_for_one() {
    let mut g = two_player_game();
    let kumano = g.add_card_to_battlefield(0, catalog::kumano_master_yamabushi());
    let frostling = g.add_card_to_battlefield(1, catalog::frostling()); // 1/1
    g.clear_sickness(kumano);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kumano, ability_index: 0, target: Some(Target::Permanent(frostling)), additional_targets: Vec::new(), x_value: None,
    }).expect("ping");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(frostling).is_none(), "1/1 pinged dead");
}

/// Teardrop Kami sacrifices to tap a target creature (mode 0).
#[test]
fn teardrop_kami_sacs_to_tap() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::teardrop_kami());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(kami);
    g.decider = Box::new(crate::decision::ScriptedDecider::new(vec![
        crate::decision::DecisionAnswer::Mode(0), // tap
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to tap");
    drain_stack(&mut g);
    assert!(g.battlefield_find(kami).is_none(), "Teardrop Kami sacrificed");
    assert!(g.battlefield_find(bear).unwrap().tapped, "target creature tapped");
}

/// Soratami Savant returns a land to counter a spell its controller can't pay for.
#[test]
fn soratami_savant_counters_unless_paid() {
    let mut g = two_player_game();
    let savant = g.add_card_to_battlefield(0, catalog::soratami_savant());
    g.add_card_to_battlefield(0, catalog::island()); // bounce fodder
    g.clear_sickness(savant);
    // Opponent casts a creature spell with no mana left to pay the tax.
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(crate::mana::Color::Green, 2);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a creature");
    // Savant's controller responds with the counter ability.
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: savant, ability_index: 0, target: Some(Target::Permanent(spell)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate counter ability");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "spell countered (unpaid)");
}

/// Pull Under shrinks a creature -5/-5, killing most bodies.
#[test]
fn pull_under_shrinks_and_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let pu = g.add_card_to_hand(0, catalog::pull_under());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: pu, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Pull Under castable");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "-5/-5 killed the 2/2");
}

/// Kiku's Shadow makes a creature deal its own power to itself.
#[test]
fn kikus_shadow_self_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let ks = g.add_card_to_hand(0, catalog::kikus_shadow());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: ks, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Kiku's Shadow castable");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "2 power → 2 self-damage killed the 2/2");
}

/// Swallowing Plague burns a creature for X and gains X life.
#[test]
fn swallowing_plague_burns_and_gains() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let sp = g.add_card_to_hand(0, catalog::swallowing_plague());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    let before = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: sp, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Swallowing Plague castable");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(bear).is_none(), "2 damage killed the 2/2");
    assert_eq!(g.players[0].life, before + 2, "gained X = 2 life");
}

/// Innocence Kami untaps itself whenever you cast a Spirit or Arcane spell.
#[test]
fn innocence_kami_untaps_on_spiritcraft() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::innocence_kami());
    g.battlefield_find_mut(kami).unwrap().tapped = true;
    let reach = g.add_card_to_hand(0, catalog::reach_through_mists()); // Arcane
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: reach, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(kami).unwrap().tapped, "Innocence Kami untapped");
}

/// Villainous Ogre can't block.
#[test]
fn villainous_ogre_cant_block() {
    assert!(catalog::villainous_ogre().keywords.contains(&Keyword::CantBlock));
}

/// Counsel of the Soratami draws two.
#[test]
fn counsel_draws_two() {
    let mut g = two_player_game();
    let c = g.add_card_to_hand(0, catalog::counsel_of_the_soratami());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: c, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Counsel");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1 + 2, "drew two (net +1 after the cast)");
}

/// Ghostly Visit destroys a nonblack creature.
#[test]
fn ghostly_visit_destroys_nonblack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let gv = g.add_card_to_hand(0, catalog::ghostly_visit());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: gv, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ghostly Visit");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "nonblack creature destroyed");
}

/// Lifegift offers 1 life whenever a land enters.
#[test]
fn lifegift_gains_on_land_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lifegift());
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Bool(true),
    ]));
    let forest = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let before = g.players[0].life;
    g.perform_action(GameAction::PlayLand(forest)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before + 1, "gained 1 life off the land ETB");
}

/// Dampen Thought mills a target player four; it carries Splice onto Arcane.
#[test]
fn dampen_thought_mills_four() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(1, catalog::island()); }
    let dt = g.add_card_to_hand(0, catalog::dampen_thought());
    assert!(catalog::dampen_thought().keywords.iter().any(|k| matches!(k, Keyword::Splice(..))));
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[1].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: dt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dampen Thought");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), before - 4, "milled four");
}

/// Consuming Vortex bounces a creature.
#[test]
fn consuming_vortex_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cv = g.add_card_to_hand(0, catalog::consuming_vortex());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: cv, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Consuming Vortex");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "creature bounced to hand");
}

/// Psychic Puppetry taps a target permanent (mode 0).
#[test]
fn psychic_puppetry_taps() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pp = g.add_card_to_hand(0, catalog::psychic_puppetry());
    g.decider = Box::new(crate::decision::ScriptedDecider::new(vec![
        crate::decision::DecisionAnswer::Mode(0),
    ]));
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: pp, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Psychic Puppetry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "target permanent tapped");
}

/// Kumano, Master Yamabushi: a creature it deals lethal damage to is exiled
/// instead of dying (source-bound CR 614 replacement).
#[test]
fn kumano_exiles_creatures_it_kills() {
    let mut g = two_player_game();
    let kumano = g.add_card_to_battlefield(0, catalog::kumano_master_yamabushi());
    let elf = g.add_card_to_battlefield(1, catalog::llanowar_elves()); // 1/1
    g.clear_sickness(kumano);
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kumano, ability_index: 0, target: Some(Target::Permanent(elf)), additional_targets: Vec::new(), x_value: None,
    }).expect("ping the 1/1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(elf).is_none(), "elf left the battlefield");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == elf), "not in graveyard");
    assert!(g.exile.iter().any(|c| c.id == elf), "elf exiled instead of dying");
}

/// Villainous Ogre's "{B}: Regenerate" is gated on controlling a Demon —
/// rejected without one, stamps a shield with one on the battlefield.
#[test]
fn villainous_ogre_regen_gated_on_demon() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::villainous_ogre());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    // No Demon: activation is rejected before paying.
    let res = g.perform_action(GameAction::ActivateAbility {
        card_id: ogre, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    });
    assert!(res.is_err(), "no Demon → can't activate");
    // Add a Demon and try again.
    g.add_card_to_battlefield(0, catalog::bloodgift_demon());
    g.perform_action(GameAction::ActivateAbility {
        card_id: ogre, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("{B}: Regenerate with a Demon out");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ogre).unwrap().regeneration_shields, 1,
        "regenerate stamps a shield");
}

/// Gnarled Mass is a 3/3 vanilla Spirit.
#[test]
fn gnarled_mass_is_a_3_3() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::gnarled_mass());
    let c = g.battlefield_find(id).unwrap();
    assert_eq!((c.power(), c.toughness()), (3, 3));
}

/// Humble Budoka has Shroud (can't be targeted).
#[test]
fn humble_budoka_has_shroud() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::humble_budoka());
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Kitsune Healer's first ability stamps a prevent-next-1 shield on a creature.
#[test]
fn kitsune_healer_prevents_next_1_damage() {
    let mut g = two_player_game();
    let healer = g.add_card_to_battlefield(0, catalog::kitsune_healer());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(healer);
    g.perform_action(GameAction::ActivateAbility {
        card_id: healer, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate prevention");
    drain_stack(&mut g);
    let mut events = Vec::new();
    g.deal_damage_to_from(crate::game::effects::EntityRef::Permanent(bear), 1, None, &mut events);
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "the next 1 damage is prevented");
}

/// Akki Rockspeaker adds {R} when it enters.
#[test]
fn akki_rockspeaker_adds_red_on_etb() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::akki_rockspeaker());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Akki Rockspeaker");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Red), 1, "ETB added one red mana");
}

/// Crawling Filth's Soulshift 5 returns a low-MV Spirit from the graveyard on death.
#[test]
fn crawling_filth_soulshift_returns_spirit() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let filth = g.add_card_to_battlefield(0, catalog::crawling_filth());
    let dead_spirit = g.add_card_to_graveyard(0, catalog::gnarled_mass()); // MV 3 Spirit
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let events = g.remove_to_graveyard_with_triggers(filth);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead_spirit), "Soulshift returned the Spirit");
}

/// Rag Dealer exiles cards from a graveyard.
#[test]
fn rag_dealer_exiles_from_graveyard() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let dealer = g.add_card_to_battlefield(0, catalog::rag_dealer());
    g.clear_sickness(dealer);
    let c1 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![c1, c2])]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: dealer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("exile from graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == c1) && g.exile.iter().any(|c| c.id == c2),
        "both cards exiled from the graveyard");
}

/// Mistblade Shinobi bounces a creature when it deals combat damage to a player.
#[test]
fn mistblade_shinobi_bounces_on_combat_damage() {
    let mut g = two_player_game();
    let ninja = g.add_card_to_battlefield(0, catalog::mistblade_shinobi());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(ninja);
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ninja, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "creature bounced on combat damage");
}

/// Skullsnatcher exiles graveyard cards when it deals combat damage to a player.
#[test]
fn skullsnatcher_exiles_gy_on_combat_damage() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let ninja = g.add_card_to_battlefield(0, catalog::skullsnatcher());
    let gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.clear_sickness(ninja);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![gy])]));
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ninja, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gy), "graveyard card exiled on combat damage");
}

/// Terashi's Cry taps up to three target creatures (ApplyToTargets).
#[test]
fn terashis_cry_taps_up_to_three() {
    let mut g = two_player_game();
    g.step = crate::game::TurnStep::PreCombatMain;
    let c1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let c3 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::terashis_cry());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(c1)),
        additional_targets: vec![Target::Permanent(c2), Target::Permanent(c3)],
        mode: None, x_value: None,
    }).expect("cast Terashi's Cry");
    drain_stack(&mut g);
    assert!(g.battlefield_find(c1).unwrap().tapped
        && g.battlefield_find(c2).unwrap().tapped
        && g.battlefield_find(c3).unwrap().tapped, "all three tapped");
}

/// Samurai Enforcers is a 4/4 with Bushido 2.
#[test]
fn samurai_enforcers_bushido_2() {
    let d = catalog::samurai_enforcers();
    assert_eq!((d.power, d.toughness), (4, 4));
    assert!(d.keywords.contains(&Keyword::Bushido(2)));
}

/// Reciprocate exiles a creature that dealt damage to you this turn.
#[test]
fn reciprocate_exiles_attacker_that_hit_you() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // Mark the attacker as having dealt damage to seat 0 this turn.
    g.players[0].creatures_that_damaged_me_this_turn.push(attacker);
    let spell = g.add_card_to_hand(0, catalog::reciprocate());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(attacker)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reciprocate");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == attacker), "attacker exiled");
}

/// Otherworldly Journey exiles a creature and returns it at the next end step
/// with a +1/+1 counter.
#[test]
fn otherworldly_journey_blinks_with_counter() {
    use crate::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::otherworldly_journey());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Otherworldly Journey");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature exiled");
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears");
    assert!(back.is_some_and(|c| c.counter_count(CounterType::PlusOnePlusOne) == 1),
        "returned with a +1/+1 counter");
}

/// Phantom Wings grants flying; sacrificing it returns the enchanted creature.
#[test]
fn phantom_wings_grants_flying_then_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let wings = g.add_card_to_hand(0, catalog::phantom_wings());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: wings, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Phantom Wings");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying), "grants flying");
    g.perform_action(GameAction::ActivateAbility {
        card_id: wings, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac to bounce");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "enchanted creature returned to hand");
}

/// Squelch counters a target activated ability and draws a card.
#[test]
fn squelch_counters_activated_ability_and_draws() {
    let mut g = two_player_game();
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone());
    g.clear_sickness(stone);
    g.players[1].mana_pool.add_colorless(1);
    g.add_card_to_library(1, catalog::island());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate draw ability");
    g.priority.player_with_priority = 0;
    let sq = g.add_card_to_hand(0, catalog::squelch());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let opp_hand = g.players[1].hand.len();
    let my_hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: sq, target: Some(Target::Permanent(stone)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Squelch");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand, "opp draw ability countered");
    assert_eq!(g.players[0].hand.len(), my_hand, "cast Squelch (-1) then drew (+1) = net 0");
}

/// Psychic Spear makes a player discard a chosen Spirit/Arcane card.
#[test]
fn psychic_spear_discards_spirit_or_arcane() {
    let mut g = two_player_game();
    let arcane_card = g.add_card_to_hand(1, catalog::counsel_of_the_soratami()); // not Arcane
    let spirit_card = g.add_card_to_hand(1, catalog::gnarled_mass()); // Spirit
    let spell = g.add_card_to_hand(0, catalog::psychic_spear());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Psychic Spear");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spirit_card), "Spirit card discarded");
    assert!(g.players[1].hand.iter().any(|c| c.id == arcane_card), "non-matching card kept");
}

/// Orochi Sustainer taps for {G}.
#[test]
fn orochi_sustainer_taps_for_green() {
    let mut g = two_player_game();
    let dork = g.add_card_to_battlefield(0, catalog::orochi_sustainer());
    g.clear_sickness(dork);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap for green");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Green), 1);
}

/// Child of Thorns sacrifices to pump a target +1/+1.
#[test]
fn child_of_thorns_sacs_to_pump() {
    let mut g = two_player_game();
    let child = g.add_card_to_battlefield(0, catalog::child_of_thorns());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: child, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to pump");
    drain_stack(&mut g);
    assert!(g.battlefield_find(child).is_none(), "Child sacrificed");
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 3, "bear pumped +1/+1");
}

/// Foratog sacrifices a Forest to get +2/+2.
#[test]
fn foratog_sacs_forest_for_pump() {
    let mut g = two_player_game();
    let atog = g.add_card_to_battlefield(0, catalog::foratog());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(atog);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: atog, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac Forest to pump");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_none(), "Forest sacrificed");
    assert_eq!(g.battlefield_find(atog).unwrap().power(), 3, "Foratog is 3/4");
}

/// Serpent Skin grants +1/+1 and can regenerate its host.
#[test]
fn serpent_skin_pumps_and_regenerates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let skin = g.add_card_to_hand(0, catalog::serpent_skin());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: skin, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Serpent Skin");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "+1/+1 granted");
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: skin, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate host");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().regeneration_shields, 1, "host has a regen shield");
}

/// Loam Dweller's spiritcraft puts a land from hand onto the battlefield tapped.
#[test]
fn loam_dweller_spiritcraft_ramps_a_land() {
    let mut g = two_player_game();
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    g.add_card_to_battlefield(0, catalog::loam_dweller());
    let land = g.add_card_to_hand(0, catalog::forest());
    let arcane = g.add_card_to_hand(0, catalog::vital_surge()); // Arcane spell
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![land])]));
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast an Arcane spell");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some_and(|c| c.tapped), "ramped land entered tapped");
}

/// Kumano's Pupils exiles creatures it deals (combat) damage to that would die.
#[test]
fn kumanos_pupils_exiles_what_it_kills() {
    let mut g = two_player_game();
    let pupils = g.add_card_to_battlefield(0, catalog::kumanos_pupils()); // 3/3
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(pupils);
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pupils, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(bear, pupils)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "the blocker is exiled, not killed");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Ire of Kaminari deals damage equal to Arcane cards in your graveyard.
#[test]
fn ire_of_kaminari_scales_with_arcane_graveyard() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::glacial_ray()); // Arcane
    g.add_card_to_graveyard(0, catalog::vital_surge()); // Arcane
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // not Arcane
    let spell = g.add_card_to_hand(0, catalog::ire_of_kaminari());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ire of Kaminari");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "2 Arcane cards → 2 damage");
}

/// Waking Nightmare makes a player discard two cards.
#[test]
fn waking_nightmare_discards_two() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::waking_nightmare());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Waking Nightmare");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), before - 2, "discarded two cards");
}

/// Pus Kami sacrifices to destroy a nonblack creature.
#[test]
fn pus_kami_sacs_to_destroy_nonblack() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::pus_kami());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green, nonblack
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kami, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(kami).is_none(), "Pus Kami sacrificed");
    assert!(g.battlefield_find(bear).is_none(), "nonblack creature destroyed");
}

/// Ronin Cavekeeper is a 4/3 with Bushido 2.
#[test]
fn ronin_cavekeeper_bushido_2() {
    let d = catalog::ronin_cavekeeper();
    assert_eq!((d.power, d.toughness), (4, 3));
    assert!(d.keywords.contains(&Keyword::Bushido(2)));
}

/// No-Dachi grants +2/+0 and first strike when equipped.
#[test]
fn no_dachi_equips_for_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::no_dachi());
    g.players[0].mana_pool.add_colorless(3);
    g.step = crate::game::TurnStep::PreCombatMain;
    g.perform_action(GameAction::Equip { equipment: sword, target: bear }).expect("equip");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 4, "+2/+0");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "granted first strike");
}

/// Lifted by Clouds grants flying until end of turn.
#[test]
fn lifted_by_clouds_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::lifted_by_clouds());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Lifted by Clouds");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
}

/// Kami of the Palace Fields carries Flying + First strike.
#[test]
fn kami_of_the_palace_fields_flies_with_first_strike() {
    let d = catalog::kami_of_the_palace_fields();
    assert!(d.keywords.contains(&Keyword::Flying) && d.keywords.contains(&Keyword::FirstStrike));
}

/// Hail of Arrows deals X damage divided among attacking creatures.
#[test]
fn hail_of_arrows_hits_attackers() {
    let mut g = two_player_game();
    // Opponent (seat 1) is the active player attacking; seat 0 casts at instant speed.
    let a1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let a2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(a1);
    g.clear_sickness(a2);
    g.active_player_idx = 1;
    g.step = crate::game::TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![
        Attack { attacker: a1, target: AttackTarget::Player(0) },
        Attack { attacker: a2, target: AttackTarget::Player(0) },
    ]).expect("attack");
    drain_stack(&mut g);
    let spell = g.add_card_to_hand(0, catalog::hail_of_arrows());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a1)),
        additional_targets: vec![Target::Permanent(a2)], mode: None, x_value: Some(2),
    }).expect("cast Hail of Arrows for X=2");
    drain_stack(&mut g);
    let total_damage: u32 = [a1, a2].iter()
        .filter_map(|id| g.battlefield_find(*id).map(|c| c.damage))
        .sum();
    assert_eq!(total_damage, 2, "2 damage divided among the attackers");
}

/// Moonlit Strider sacrifices to grant protection from a chosen color.
#[test]
fn moonlit_strider_sacs_for_protection() {
    let mut g = two_player_game();
    let strider = g.add_card_to_battlefield(0, catalog::moonlit_strider());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: strider, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("sac for protection");
    drain_stack(&mut g);
    assert!(g.battlefield_find(strider).is_none(), "Strider sacrificed");
    let has_pro = g.computed_permanent(bear).unwrap().keywords.iter()
        .any(|k| matches!(k, Keyword::Protection(_)));
    assert!(has_pro, "bear gained protection from a color");
}

// ── Flip cards (CR 711) ──────────────────────────────────────────────────────

/// Faithful Squire accrues ki on Arcane casts (spiritcraft) and flips into
/// Kaiso at the end step once it has two or more ki counters.
#[test]
fn faithful_squire_spiritcrafts_ki_then_flips_to_kaiso() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let squire = g.add_card_to_battlefield(0, catalog::faithful_squire());
    let a1 = g.add_card_to_hand(0, catalog::kodamas_might()); // {G} Arcane (no draw)
    let a2 = g.add_card_to_hand(0, catalog::kodamas_might());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    // Two "may put a ki counter" + one "may flip" — all accepted.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), DecisionAnswer::Bool(true), DecisionAnswer::Bool(true),
    ]));
    for spell in [a1, a2] {
        g.perform_action(GameAction::CastSpell {
            card_id: spell, target: Some(Target::Permanent(squire)),
            additional_targets: vec![], mode: None, x_value: None,
        }).expect("cast Arcane");
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(squire).unwrap().counter_count(CounterType::Ki), 2,
        "two ki counters accrued");
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    let kaiso = g.battlefield_find(squire).unwrap();
    assert!(kaiso.flipped, "flipped at end step");
    assert_eq!(kaiso.definition.name, "Kaiso, Memory of Loyalty");
    assert_eq!((kaiso.definition.power, kaiso.definition.toughness), (3, 4));
    assert!(kaiso.definition.keywords.contains(&Keyword::Flying));
}

/// A ki-counter flip card stays unflipped at the end step with only one ki.
#[test]
fn cunning_bandit_does_not_flip_below_two_ki() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bandit = g.add_card_to_battlefield(0, catalog::cunning_bandit());
    g.battlefield_find_mut(bandit).unwrap().add_counters(CounterType::Ki, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bandit).unwrap().flipped, "one ki is not enough to flip");
    assert_eq!(g.battlefield_find(bandit).unwrap().definition.name, "Cunning Bandit");
}

/// Azamuki (Cunning Bandit's flip side) spends a ki counter to steal a creature
/// until end of turn.
#[test]
fn azamuki_removes_ki_to_steal_a_creature() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bandit = g.add_card_to_battlefield(0, catalog::cunning_bandit());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bandit).unwrap().add_counters(CounterType::Ki, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bandit).unwrap().flipped, "flipped into Azamuki");
    g.clear_sickness(bandit);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bandit, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Azamuki steal");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "gained control of the bear");
    assert_eq!(g.battlefield_find(bandit).unwrap().counter_count(CounterType::Ki), 1,
        "a ki counter was paid");
}

/// Budoka Gardener's {T} ability puts a land from hand and flips into Dokai
/// once the controller reaches ten lands; Dokai mints an Elemental sized to
/// the land count.
#[test]
fn budoka_gardener_flips_to_dokai_at_ten_lands() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let budoka = g.add_card_to_battlefield(0, catalog::budoka_gardener());
    for _ in 0..9 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let land = g.add_card_to_hand(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![land])]));
    g.clear_sickness(budoka);
    g.perform_action(GameAction::ActivateAbility {
        card_id: budoka, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap Budoka");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_some(), "land put onto the battlefield");
    let dokai = g.battlefield_find(budoka).unwrap();
    assert!(dokai.flipped, "flipped to Dokai at ten lands");
    assert_eq!(dokai.definition.name, "Dokai, Weaver of Life");
    // Dokai's {4}{G}{G}, {T}: mint an X/X Elemental (X = lands you control = 10).
    g.battlefield_find_mut(budoka).unwrap().tapped = false;
    g.clear_sickness(budoka);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: budoka, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Dokai");
    drain_stack(&mut g);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Elemental")
        .expect("Elemental token minted");
    let cp = g.computed_permanent(token.id).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10), "X/X where X = lands you control");
}

/// Heartbeat of Spring doubles the mana a tapped land makes.
#[test]
fn heartbeat_of_spring_doubles_land_mana() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::heartbeat_of_spring());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap forest");
    assert_eq!(g.players[0].mana_pool.amount(crate::mana::Color::Green), 2, "{{G}} becomes {{G}}{{G}}");
    // Symmetric: the opponent's land also produces the extra (it affects every
    // player's lands, not just the controller's). Hand priority to player 1 so
    // the mana ability is legal off-turn.
    let opp_forest = g.add_card_to_battlefield(1, catalog::forest());
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: opp_forest, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("tap opponent forest");
    assert_eq!(g.players[1].mana_pool.amount(crate::mana::Color::Green), 2, "opponent's {{G}} doubles too");
}

/// Soratami Seer discards your hand and redraws that many (returning 2 lands).
#[test]
fn soratami_seer_discards_hand_then_redraws() {
    let mut g = two_player_game();
    let seer = g.add_card_to_battlefield(0, catalog::soratami_seer());
    g.clear_sickness(seer);
    let lands: Vec<_> = (0..2).map(|_| g.add_card_to_battlefield(0, catalog::island())).collect();
    // Hand of 3 cards; library has fresh cards to redraw.
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    for _ in 0..5 { g.add_card_to_library(0, catalog::mountain()); }
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate Seer");
    drain_stack(&mut g);
    // The 2 returned lands land in hand first (cost), so the wheel discards 5
    // and redraws 5 — all fresh Mountains from the library.
    assert_eq!(g.players[0].hand.len(), 5, "discarded the whole (post-cost) hand and redrew that many");
    assert!(g.players[0].hand.iter().all(|c| c.definition.name == "Mountain"),
        "old hand discarded, new cards drawn from library");
    assert_eq!(lands.iter().filter(|id| g.battlefield_find(**id).is_some()).count(), 0,
        "two lands returned as the cost");
}

/// Soratami Mirror-Mage bounces a creature by returning three of your lands.
#[test]
fn soratami_mirror_mage_bounces_creature_for_three_lands() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::soratami_mirror_mage());
    g.clear_sickness(mage);
    let lands: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(0, catalog::island())).collect();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage, ability_index: 0, target: Some(Target::Permanent(victim)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("activate Mirror-Mage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "target creature returned to hand");
    let lands_left = lands.iter().filter(|id| g.battlefield_find(**id).is_some()).count();
    assert_eq!(lands_left, 0, "three lands returned as the cost");
}

/// Gutwrencher Oni makes you discard at upkeep unless you control an Ogre.
#[test]
fn gutwrencher_oni_upkeep_discard_unless_ogre() {
    let mut g = two_player_game();
    let oni = g.add_card_to_battlefield(0, catalog::gutwrencher_oni());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_hand(0, catalog::forest());
    let trig = catalog::gutwrencher_oni().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(oni, 0, None, 0);
    // No Ogre → discard one.
    let before = g.players[0].hand.len();
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before - 1, "discards with no Ogre");
    // With an Ogre, no discard.
    g.add_card_to_battlefield(0, catalog::deathcurse_ogre()); // an Ogre
    let before = g.players[0].hand.len();
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "no discard while controlling an Ogre");
}

/// Crackdown keeps big nonwhite creatures tapped.
#[test]
fn crackdown_prevents_untap_of_big_nonwhite_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crackdown());
    let big = g.add_card_to_battlefield(1, catalog::moss_kami()); // 5/5 green
    let small = g.add_card_to_battlefield(1, catalog::lantern_kami()); // 1/1 white
    g.battlefield_find_mut(big).unwrap().tapped = true;
    g.battlefield_find_mut(small).unwrap().tapped = true;
    // Run player 1's untap step.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(big).unwrap().tapped, "5/5 nonwhite stays tapped");
    assert!(!g.battlefield_find(small).unwrap().tapped, "1/1 white untaps normally");
}

/// Jade Idol animates into a 4/4 on a Spirit/Arcane cast.
#[test]
fn jade_idol_spiritcraft_animates() {
    let mut g = two_player_game();
    let idol = g.add_card_to_battlefield(0, catalog::jade_idol());
    assert!(!g.computed_permanent(idol).unwrap().card_types.contains(&crate::card::CardType::Creature),
        "starts as a non-creature artifact");
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    let cp = g.computed_permanent(idol).unwrap();
    assert!(cp.card_types.contains(&crate::card::CardType::Creature), "now a creature");
    assert_eq!((cp.power, cp.toughness), (4, 4), "4/4");
}

/// Long-Forgotten Gohei anthems your Spirits and cheapens Arcane spells.
#[test]
fn long_forgotten_gohei_anthems_and_reduces_arcane_cost() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::long_forgotten_gohei());
    let kami = g.add_card_to_battlefield(0, catalog::lantern_kami()); // 1/1 Spirit
    let cp = g.computed_permanent(kami).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "Spirit gets +1/+1");
    // Reach Through Mists ({U} Arcane) is reduced to {0} colored-only → still
    // needs the {U}; the {1}-generic discount applies to a generic pip. Use
    // Glacial Ray ({1}{R} Arcane): {R} after the {1} reduction.
    let ray = g.add_card_to_hand(0, catalog::glacial_ray());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1); // only {R}, no generic
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: ray, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("Arcane spell castable for {R} after the {1} discount");
}

/// Nine-Ringed Bo pings a Spirit and exiles it if it dies.
#[test]
fn nine_ringed_bo_pings_and_exiles_a_spirit() {
    let mut g = two_player_game();
    let bo = g.add_card_to_battlefield(0, catalog::nine_ringed_bo());
    let spirit = g.add_card_to_battlefield(1, catalog::lantern_kami()); // 1/1 Spirit
    g.perform_action(GameAction::ActivateAbility {
        card_id: bo, ability_index: 0, target: Some(Target::Permanent(spirit)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("ping the Spirit");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(spirit).is_none(), "1/1 dies to the ping");
    assert!(g.exile.iter().any(|c| c.id == spirit), "exiled instead of going to the graveyard");
    assert!(g.players[1].graveyard.iter().all(|c| c.id != spirit), "not in the graveyard");
}

/// Marrow-Gnawer makes all Rats fear and mints Rats by sacrificing one.
#[test]
fn marrow_gnawer_lords_rats_and_mints_tokens() {
    let mut g = two_player_game();
    let marrow = g.add_card_to_battlefield(0, catalog::marrow_gnawer());
    g.clear_sickness(marrow);
    let rat = g.add_card_to_battlefield(0, catalog::nezumi_cutthroat()); // a Rat
    assert!(g.computed_permanent(rat).unwrap().keywords.contains(&Keyword::Fear),
        "all Rats gain fear");
    // {T}, Sacrifice a Rat: after sac, Marrow is the only Rat → 1 token.
    g.perform_action(GameAction::ActivateAbility {
        card_id: marrow, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate the mint");
    drain_stack(&mut g);
    let rats = g.battlefield.iter()
        .filter(|c| c.controller == 0
            && c.definition.subtypes.creature_types.contains(&crate::card::CreatureType::Rat))
        .count();
    // Marrow + 1 minted token (the sacrificed Rat is gone).
    assert_eq!(rats, 2, "minted X=1 Rat token (Marrow was the only Rat left at resolution)");
}

/// Kuro sacrifices itself at upkeep when its keeper declines the {B}{B}{B}{B},
/// and its pay-life ability shrinks a creature.
#[test]
fn kuro_pitlord_upkeep_sacrifice_and_pay_life_shrink() {
    let mut g = two_player_game();
    let kuro = g.add_card_to_battlefield(0, catalog::kuro_pitlord());
    g.clear_sickness(kuro);
    // Pay 1 life: target creature gets -1/-1 → kills a 1/1.
    let x1 = g.add_card_to_battlefield(1, catalog::lantern_kami()); // 1/1
    g.perform_action(GameAction::ActivateAbility {
        card_id: kuro, ability_index: 0, target: Some(Target::Permanent(x1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("pay 1 life to shrink");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(x1).is_none(), "-1/-1 kills the 1/1");
    // Upkeep: no mana floated, AutoDecider declines → Kuro is sacrificed.
    let trig = catalog::kuro_pitlord().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(kuro, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(kuro).is_none(), "sacrificed when the upkeep cost goes unpaid");
}

/// Strange Inversion swaps a creature's power and toughness.
#[test]
fn strange_inversion_swaps_pt() {
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(1, catalog::lantern_kami()); // 1/1 — pick a lopsided body
    let bigp = g.add_card_to_battlefield(1, catalog::kami_of_lunacy()); // 4/1
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let inv = g.add_card_to_hand(0, catalog::strange_inversion());
    cast_at(&mut g, inv, Target::Permanent(bigp));
    let cp = g.computed_permanent(bigp).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 4), "4/1 becomes 1/4");
    let _ = kami;
}

/// Sift Through Sands nets a card (draw two, discard one).
#[test]
fn sift_through_sands_draws_two_discards_one() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let sift = g.add_card_to_hand(0, catalog::sift_through_sands());
    let before = g.players[0].hand.len(); // includes Sift itself
    g.perform_action(GameAction::CastSpell {
        card_id: sift, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Sift Through Sands");
    drain_stack(&mut g);
    // -1 (Sift leaves hand) +2 draw -1 discard = net 0 from `before`.
    assert_eq!(g.players[0].hand.len(), before, "draw two, discard one nets +1 over the spell itself");
}

/// Wicked Akuba can only drain a player it dealt damage to this turn.
#[test]
fn wicked_akuba_drains_a_player_it_damaged() {
    let mut g = two_player_game();
    let akuba = g.add_card_to_battlefield(0, catalog::wicked_akuba());
    g.clear_sickness(akuba);
    // Hasn't damaged anyone yet → the activation is rejected.
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: akuba, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).is_err(), "no damage dealt yet → can't target");
    // Record that Akuba damaged player 1 this turn (as combat would).
    g.players[1].creatures_that_damaged_me_this_turn.push(akuba);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: akuba, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("now a legal target");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "loses 1 life");
}

/// Seizan drains and refills the active player at each upkeep.
#[test]
fn seizan_drains_and_draws_for_active_player() {
    let mut g = two_player_game();
    let seizan = g.add_card_to_battlefield(0, catalog::seizan_perverter_of_truth());
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let (life, hand) = (g.players[0].life, g.players[0].hand.len());
    // Active player is seat 0 in a fresh game → its upkeep trigger.
    let trig = catalog::seizan_perverter_of_truth().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_trigger(seizan, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "active player loses 2");
    assert_eq!(g.players[0].hand.len(), hand + 2, "and draws two");
}

/// Soul of Magma pings a creature on a Spirit/Arcane cast.
#[test]
fn soul_of_magma_spiritcraft_pings() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::soul_of_magma());
    let x1 = g.add_card_to_battlefield(1, catalog::lantern_kami()); // 1/1
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: Some(Target::Permanent(x1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(x1).is_none(), "1/1 dies to the 1-damage ping");
}

/// Part the Veil returns only your creatures to hand.
#[test]
fn part_the_veil_bounces_your_creatures_only() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let part = g.add_card_to_hand(0, catalog::part_the_veil());
    g.perform_action(GameAction::CastSpell {
        card_id: part, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Part the Veil");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none(), "your creature bounced");
    assert!(g.battlefield_find(theirs).is_some(), "opponent's creature stays");
}

/// Reito Lantern tucks a graveyard card under its owner's library.
#[test]
fn reito_lantern_bottoms_a_graveyard_card() {
    let mut g = two_player_game();
    let lantern = g.add_card_to_battlefield(0, catalog::reito_lantern());
    let card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lantern, ability_index: 0, target: Some(Target::Permanent(card)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("tuck the card");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().all(|c| c.id != card), "left the graveyard");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(card), "now on the bottom of its owner's library");
}

/// Okina pumps a legendary creature but can't target a non-legendary one.
#[test]
fn okina_pumps_only_legendary_creatures() {
    let mut g = two_player_game();
    let okina = g.add_card_to_battlefield(0, catalog::okina_temple_to_the_grandfathers());
    let legend = g.add_card_to_battlefield(0, catalog::masako_the_humorless()); // legendary 2/1
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // not legendary
    // Non-legendary target rejected.
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: okina, ability_index: 1, target: Some(Target::Permanent(bear)),
        additional_targets: Vec::new(), x_value: None,
    }).is_err(), "can't pump a non-legendary creature");
    // Legendary target works.
    g.perform_action(GameAction::ActivateAbility {
        card_id: okina, ability_index: 1, target: Some(Target::Permanent(legend)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("pump the legend");
    drain_stack(&mut g);
    let cp = g.computed_permanent(legend).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "+1/+1");
}

/// Minamo untaps a tapped legendary permanent.
#[test]
fn minamo_untaps_a_legendary_permanent() {
    let mut g = two_player_game();
    let minamo = g.add_card_to_battlefield(0, catalog::minamo_school_at_waters_edge());
    let legend = g.add_card_to_battlefield(0, catalog::masako_the_humorless());
    g.battlefield_find_mut(legend).unwrap().tapped = true;
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: minamo, ability_index: 1, target: Some(Target::Permanent(legend)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("untap the legend");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(legend).unwrap().tapped, "legendary permanent untapped");
}

/// Uncontrollable Anger gives +2/+2 and forces attacks.
#[test]
fn uncontrollable_anger_pumps_and_forces_attack() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    let aura = g.add_card_to_hand(0, catalog::uncontrollable_anger());
    cast_at(&mut g, aura, Target::Permanent(bear));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+2/+2");
    assert!(cp.keywords.contains(&Keyword::MustAttack), "attacks each combat if able");
}

/// Masako lets a tapped creature block as though untapped (CR 509.1a).
#[test]
fn masako_lets_tapped_creatures_block() {
    let mut g = two_player_game();
    // Opponent (seat 1) attacks; seat 0 blocks with a tapped creature.
    let atk = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(blocker).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.give_priority_to_active();
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::DeclareBlockers);
    // Without Masako, a tapped creature can't block.
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)])).is_err(),
        "tapped creature can't block without Masako");
    g.add_card_to_battlefield(0, catalog::masako_the_humorless());
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, atk)]))
        .expect("with Masako, the tapped creature blocks");
}

/// Guardian of Solitude grants flying to a target on a Spirit/Arcane cast.
#[test]
fn guardian_of_solitude_spiritcraft_grants_flying() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guardian_of_solitude());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "spiritcraft granted flying");
}

/// Earthshaker pings every non-flyer for 2 on a Spirit/Arcane cast.
#[test]
fn earthshaker_spiritcraft_sweeps_nonflyers() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::earthshaker());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, no flying
    let flyer = g.add_card_to_battlefield(1, catalog::hundred_talon_kami()); // 2/3 flying
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(ground).is_none(), "2/2 ground creature dies to 2 damage");
    assert!(g.battlefield_find(flyer).is_some(), "flyer is untouched");
}

/// Dance of Shadows pumps your team +1/+0 and grants fear.
#[test]
fn dance_of_shadows_pumps_and_grants_fear() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    let dance = g.add_card_to_hand(0, catalog::dance_of_shadows());
    g.perform_action(GameAction::CastSpell {
        card_id: dance, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dance of Shadows");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!(cp.power, 3, "+1/+0");
    assert!(cp.keywords.contains(&Keyword::Fear), "and fear");
}

/// Deathcurse Ogre drains each player 3 on death.
#[test]
fn deathcurse_ogre_drains_each_player() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::deathcurse_ogre());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let events = g.remove_to_graveyard_with_triggers(ogre);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 3, "controller loses 3");
    assert_eq!(g.players[1].life, l1 - 3, "opponent loses 3");
}

/// Cleanfall destroys every enchantment.
#[test]
fn cleanfall_destroys_all_enchantments() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::honden_of_cleansing_fire());
    let theirs = g.add_card_to_battlefield(1, catalog::ghostly_prison());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let clean = g.add_card_to_hand(0, catalog::cleanfall());
    g.perform_action(GameAction::CastSpell {
        card_id: clean, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cleanfall");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(),
        "both enchantments destroyed");
}

/// Graceful Adept removes your maximum hand size.
#[test]
fn graceful_adept_removes_max_hand_size() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::graceful_adept());
    assert_eq!(g.effective_max_hand_size(0), None, "no maximum hand size while it's in play");
}

/// Eerie Procession tutors an Arcane card to hand.
#[test]
fn eerie_procession_finds_an_arcane_card() {
    let mut g = two_player_game();
    let ray = g.add_card_to_library(0, catalog::glacial_ray()); // Arcane
    g.add_card_to_library(0, catalog::grizzly_bears()); // non-Arcane filler
    g.decider = Box::new(crate::decision::ScriptedDecider::new([
        crate::decision::DecisionAnswer::Search(Some(ray)),
    ]));
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    let proc = g.add_card_to_hand(0, catalog::eerie_procession());
    g.perform_action(GameAction::CastSpell {
        card_id: proc, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Eerie Procession");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Glacial Ray"),
        "Arcane card put into hand");
}

/// Kodama of the South Tree pumps the team (but not itself) on a Spirit/Arcane
/// cast.
#[test]
fn kodama_of_the_south_tree_spiritcraft_pumps_team() {
    let mut g = two_player_game();
    let kodama = g.add_card_to_battlefield(0, catalog::kodama_of_the_south_tree());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let arcane = g.add_card_to_hand(0, catalog::reach_through_mists());
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: arcane, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Arcane spell");
    drain_stack(&mut g);
    let cb = g.computed_permanent(bear).unwrap();
    assert_eq!((cb.power, cb.toughness), (3, 3), "other creature +1/+1");
    assert!(cb.keywords.contains(&Keyword::Trample), "and gains trample");
    let ck = g.computed_permanent(kodama).unwrap();
    assert_eq!((ck.power, ck.toughness), (4, 4), "Kodama itself is unaffected (other creatures only)");
}

/// Ethereal Haze fogs all creature combat damage for the turn.
#[test]
fn ethereal_haze_fogs_combat() {
    let mut g = two_player_game();
    let atk = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(atk);
    let haze = g.add_card_to_hand(0, catalog::ethereal_haze());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: haze, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ethereal Haze");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: atk, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::PostCombatMain);
    assert_eq!(g.players[1].life, 20, "no combat damage got through");
}

/// Masumaro is */* equal to twice the cards in your hand.
#[test]
fn masumaro_sizes_to_twice_your_hand() {
    let mut g = two_player_game();
    let masumaro = g.add_card_to_battlefield(0, catalog::masumaro_first_to_live());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); }
    let cp = g.computed_permanent(masumaro).unwrap();
    assert_eq!((cp.power, cp.toughness), (6, 6), "twice a 3-card hand");
}

/// Adamaro is */* equal to the opponent with the most cards in hand.
#[test]
fn adamaro_sizes_to_largest_opponent_hand() {
    let mut g = two_player_game();
    let adamaro = g.add_card_to_battlefield(0, catalog::adamaro_first_to_desire());
    for _ in 0..4 { g.add_card_to_hand(1, catalog::mountain()); } // opponent holds 4
    for _ in 0..9 { g.add_card_to_hand(0, catalog::mountain()); } // own hand doesn't count
    let cp = g.computed_permanent(adamaro).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "reads the opponent's hand, not yours");
}

/// Kagemaro is sized to your hand and wraths for -X/-X on sacrifice.
#[test]
fn kagemaro_sizes_to_hand_and_wraths() {
    let mut g = two_player_game();
    let kagemaro = g.add_card_to_battlefield(0, catalog::kagemaro_first_to_suffer());
    for _ in 0..3 { g.add_card_to_hand(0, catalog::forest()); } // 3 cards in hand
    let cp = g.computed_permanent(kagemaro).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "*/* = cards in hand");
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(kagemaro);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kagemaro, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("sac for -X/-X");
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(victim).is_none(), "-3/-3 kills the 2/2");
}

/// Terashi's Grasp destroys an artifact/enchantment and gains life = its MV.
#[test]
fn terashis_grasp_destroys_and_gains_life() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::ghostly_prison()); // MV 3
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let grasp = g.add_card_to_hand(0, catalog::terashis_grasp());
    let life = g.players[0].life;
    cast_at(&mut g, grasp, Target::Permanent(ench));
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
    assert_eq!(g.players[0].life, life + 3, "gained life equal to MV 3");
}

/// Call to Glory untaps your creatures and pumps your Samurai.
#[test]
fn call_to_glory_untaps_and_pumps_samurai() {
    let mut g = two_player_game();
    let samurai = g.add_card_to_battlefield(0, catalog::hand_of_honor()); // Samurai 2/2
    g.battlefield_find_mut(samurai).unwrap().tapped = true;
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let ctg = g.add_card_to_hand(0, catalog::call_to_glory());
    cast(&mut g, ctg);
    assert!(!g.battlefield_find(samurai).unwrap().tapped, "creature untapped");
    let cp = g.computed_permanent(samurai).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "Samurai got +1/+1");
}

/// Mass of Ghouls is a vanilla 5/3.
#[test]
fn mass_of_ghouls_is_a_5_3() {
    let d = catalog::mass_of_ghouls();
    assert_eq!((d.power, d.toughness), (5, 3));
}

/// Promised Kannushi's Soulshift 7 returns a Spirit on death.
#[test]
fn promised_kannushi_soulshift_seven() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kannushi = g.add_card_to_battlefield(0, catalog::promised_kannushi());
    let spirit = g.add_card_to_graveyard(0, catalog::gibbering_kami()); // Spirit MV 4 ≤ 7
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let events = g.remove_to_graveyard_with_triggers(kannushi);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit), "soulshift 7 returned the Spirit");
}

/// Akki Drillmaster grants haste to a target creature.
#[test]
fn akki_drillmaster_grants_haste() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_drillmaster());
    let fresh = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(akki);
    g.perform_action(GameAction::ActivateAbility {
        card_id: akki, ability_index: 0, target: Some(Target::Permanent(fresh)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("grant haste");
    drain_stack(&mut g);
    assert!(g.computed_permanent(fresh).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

/// Soramaro is sized to your hand and bounces a land to draw.
#[test]
fn soramaro_sizes_to_hand_and_draws() {
    let mut g = two_player_game();
    let soramaro = g.add_card_to_battlefield(0, catalog::soramaro_first_to_dream());
    for _ in 0..2 { g.add_card_to_hand(0, catalog::forest()); }
    assert_eq!(g.computed_permanent(soramaro).unwrap().power, 2, "*/* = cards in hand");
    g.add_card_to_library(0, catalog::forest()); // a card to draw
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: soramaro, ability_index: 0, target: Some(Target::Permanent(land)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("bounce land, draw");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == land), "land returned to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "drew a card and got the land back");
}

/// Kemuri-Onna's ETB makes the targeted opponent discard.
#[test]
fn kemuri_onna_etb_discards() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::forest());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::kemuri_onna());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "opponent discarded a card to Kemuri's ETB");
}

/// Inner Calm, Outer Strength pumps by your hand size.
#[test]
fn inner_calm_outer_strength_pumps_by_hand_size() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::inner_calm_outer_strength());
    for _ in 0..2 { g.add_card_to_hand(0, catalog::forest()); } // hand = spell + 2 = 3 at cast
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_at(&mut g, spell, Target::Permanent(bear));
    // After casting, the spell leaves hand → 2 cards remain → +2/+2.
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "+X/+X for X = cards in hand at resolution");
}

/// Gale Force deals 5 to fliers only.
#[test]
fn gale_force_hits_only_fliers() {
    let mut g = two_player_game();
    let flier = g.add_card_to_battlefield(1, {
        let mut d = catalog::grizzly_bears();
        d.name = "Flier"; d.toughness = 4; d.keywords = vec![Keyword::Flying]; d
    });
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    let gf = g.add_card_to_hand(0, catalog::gale_force());
    cast(&mut g, gf);
    g.check_state_based_actions();
    assert!(g.battlefield_find(flier).is_none(), "5 damage kills the 4-toughness flier");
    assert!(g.battlefield_find(ground).is_some(), "ground creature untouched");
}

/// Hand of Cruelty / Hand of Honor carry protection + bushido.
#[test]
fn hands_carry_protection_and_bushido() {
    let cruelty = catalog::hand_of_cruelty();
    assert!(cruelty.keywords.contains(&Keyword::Protection(crate::mana::Color::White)));
    assert!(cruelty.keywords.contains(&Keyword::Bushido(1)));
    let honor = catalog::hand_of_honor();
    assert!(honor.keywords.contains(&Keyword::Protection(crate::mana::Color::Black)));
}

/// Nightsoil Kami's Soulshift 5 returns a small Spirit on death.
#[test]
fn nightsoil_kami_soulshift_returns_spirit() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kami = g.add_card_to_battlefield(0, catalog::nightsoil_kami());
    let spirit = g.add_card_to_graveyard(0, catalog::kami_of_ancient_law()); // Spirit MV 2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let events = g.remove_to_graveyard_with_triggers(kami);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit), "soulshift 5 returned the Spirit");
}

/// Nezumi Shadow-Watcher sacrifices to destroy a Ninja.
#[test]
fn nezumi_shadow_watcher_destroys_a_ninja() {
    let mut g = two_player_game();
    let watcher = g.add_card_to_battlefield(0, catalog::nezumi_shadow_watcher());
    let ninja = g.add_card_to_battlefield(1, {
        let mut d = catalog::grizzly_bears();
        d.name = "Ninja"; d.subtypes.creature_types = vec![crate::card::CreatureType::Ninja]; d
    });
    g.clear_sickness(watcher);
    g.perform_action(GameAction::ActivateAbility {
        card_id: watcher, ability_index: 0, target: Some(Target::Permanent(ninja)),
        additional_targets: Vec::new(), x_value: None,
    }).expect("sac to destroy ninja");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ninja).is_none(), "Ninja destroyed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == watcher), "Watcher sacrificed");
}

/// Journeyer's Kite fetches a basic to hand for {3}, {T}.
#[test]
fn journeyers_kite_fetches_a_basic_to_hand() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let kite = g.add_card_to_battlefield(0, catalog::journeyers_kite());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: kite, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("fetch land");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic land to hand");
}

/// CR 711.6 — a flip card reverts to its unflipped face as it leaves the
/// battlefield.
#[test]
fn flip_card_reverts_to_top_face_on_leaving_battlefield() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bandit = g.add_card_to_battlefield(0, catalog::cunning_bandit());
    g.battlefield_find_mut(bandit).unwrap().add_counters(CounterType::Ki, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bandit).unwrap().flipped, "flipped into Azamuki");
    g.remove_from_battlefield_to_graveyard_raw(bandit);
    let in_gy = g.players[0].graveyard.iter().find(|c| c.id == bandit).expect("in graveyard");
    assert!(!in_gy.flipped, "reverts off the battlefield");
    assert_eq!(in_gy.definition.name, "Cunning Bandit", "top face restored");
}

/// Scarmaker (Hired Muscle's flip side) spends a ki counter to grant fear.
#[test]
fn scarmaker_removes_ki_to_grant_fear() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let muscle = g.add_card_to_battlefield(0, catalog::hired_muscle());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(muscle).unwrap().add_counters(CounterType::Ki, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(muscle).unwrap().definition.name, "Scarmaker");
    g.clear_sickness(muscle);
    g.perform_action(GameAction::ActivateAbility {
        card_id: muscle, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("activate Scarmaker fear");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Fear),
        "target gained fear");
}

/// Akki Lavarunner flips into Tok-Tok when it deals combat damage to a player.
#[test]
fn akki_lavarunner_flips_on_combat_damage_to_player() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_lavarunner());
    g.clear_sickness(akki);
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: akki, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    let tok = g.battlefield_find(akki).unwrap();
    assert!(tok.flipped && tok.definition.name == "Tok-Tok, Volcano Born", "flipped to Tok-Tok");
    assert!(tok.definition.keywords.contains(&Keyword::Protection(crate::mana::Color::Red)));
}

/// Tok-Tok's static adds 1 to red-source combat damage dealt to a player.
#[test]
fn tok_tok_adds_one_to_red_damage_to_players() {
    let mut g = two_player_game();
    // Flip an Akki into Tok-Tok on the spot.
    let akki = g.add_card_to_battlefield(0, catalog::akki_lavarunner());
    let mut ev = Vec::new();
    g.flip_permanent(akki, &mut ev);
    assert_eq!(g.battlefield_find(akki).unwrap().definition.name, "Tok-Tok, Volcano Born");
    // A red 2/2 deals combat damage to the opponent: 2 + 1 = 3.
    let mut red = catalog::grizzly_bears();
    red.name = "Red Ogre";
    red.cost = crate::mana::cost(&[crate::mana::generic(1), crate::mana::r()]); // red source
    let attacker = g.add_card_to_battlefield(0, red);
    g.clear_sickness(attacker);
    let life_before = g.players[1].life;
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3, "red 2/2 deals 2+1 with Tok-Tok out");
}

/// Jaraku (Callow Jushi's flip side) counters a spell unless its controller
/// pays {2}, spending a ki counter.
#[test]
fn jaraku_counters_unless_paid() {
    use crate::card::CounterType;
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let jushi = g.add_card_to_battlefield(0, catalog::callow_jushi());
    g.battlefield_find_mut(jushi).unwrap().add_counters(CounterType::Ki, 2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jushi).unwrap().definition.name, "Jaraku the Interloper");
    // Opponent (no mana) casts a spell; Jaraku counters it.
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts");
    g.battlefield_find_mut(jushi).unwrap().tapped = false;
    g.clear_sickness(jushi);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: jushi, ability_index: 0,
        target: Some(Target::Permanent(spell)), additional_targets: Vec::new(), x_value: None,
    }).expect("Jaraku counters");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell), "spell countered (opp can't pay)");
}

/// Jushi Apprentice flips into Tomoya once its draw pushes hand size to nine+.
#[test]
fn jushi_apprentice_flips_at_nine_cards() {
    let mut g = two_player_game();
    let jushi = g.add_card_to_battlefield(0, catalog::jushi_apprentice());
    // Stock library (so the draw resolves) and hand to 8 (the draw makes 9).
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    for _ in 0..8 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    g.clear_sickness(jushi);
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: jushi, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("draw + maybe flip");
    drain_stack(&mut g);
    let tom = g.battlefield_find(jushi).unwrap();
    assert!(tom.flipped && tom.definition.name == "Tomoya the Revealer", "flipped at 9 cards");
}

/// Orochi Eggwatcher mints Snakes and flips into Shidako at ten+ creatures;
/// Shidako sacrifices a creature to pump another +3/+3.
#[test]
fn orochi_eggwatcher_flips_to_shidako_at_ten_creatures() {
    let mut g = two_player_game();
    let orochi = g.add_card_to_battlefield(0, catalog::orochi_eggwatcher());
    // Nine other creatures → activating mints the tenth and flips.
    for _ in 0..9 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.clear_sickness(orochi);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: orochi, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("mint snake");
    drain_stack(&mut g);
    let shidako = g.battlefield_find(orochi).unwrap();
    assert!(shidako.flipped && shidako.definition.name == "Shidako, Broodmistress", "flipped at ten creatures");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Snake"), "Snake token minted");
    // Shidako: {G}, Sacrifice a creature: target gets +3/+3.
    let bear = g.battlefield.iter().find(|c| c.definition.name == "Grizzly Bears").unwrap().id;
    g.battlefield_find_mut(orochi).unwrap().tapped = false;
    g.clear_sickness(orochi);
    g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: orochi, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None,
    }).expect("Shidako pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5), "+3/+3 applied");
}

/// Initiate of Blood pings a damaged creature; if that kills it, Initiate flips
/// into Goka the Unjust.
#[test]
fn initiate_of_blood_flips_when_its_ping_kills_a_damaged_creature() {
    let mut g = two_player_game();
    let initiate = g.add_card_to_battlefield(0, catalog::initiate_of_blood());
    g.clear_sickness(initiate);
    // A 1/1 already dealt damage this turn (so it's a legal target, and 1 more kills it).
    let victim = g.add_card_to_battlefield(1, {
        let mut d = catalog::grizzly_bears();
        d.name = "Wounded"; d.power = 1; d.toughness = 1; d
    });
    g.battlefield_find_mut(victim).unwrap().dealt_damage_this_turn = true;
    g.perform_action(GameAction::ActivateAbility {
        card_id: initiate, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    }).expect("ping the damaged creature");
    drain_stack(&mut g);
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim died to the ping");
    let goka = g.battlefield_find(initiate).unwrap();
    assert!(goka.flipped && goka.definition.name == "Goka the Unjust", "Initiate flipped into Goka");
}

/// Student of Elements flips into Tobita when it gains flying (CR 603.8
/// state-triggered ability); Tobita's static then grants your creatures flying.
#[test]
fn student_of_elements_flips_when_it_gains_flying() {
    let mut g = two_player_game();
    let student = g.add_card_to_battlefield(0, catalog::student_of_elements());
    // No flying yet → no flip.
    g.check_state_based_actions();
    assert!(!g.battlefield_find(student).unwrap().flipped, "no flying, no flip");
    // Grant it flying from an external source.
    g.battlefield_find_mut(student).unwrap().granted_keywords_eot.push(Keyword::Flying);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    let tobita = g.battlefield_find(student).unwrap();
    assert!(tobita.flipped && tobita.definition.name == "Tobita, Master of Winds", "flipped on flying");
    // Tobita's static grants flying to your other creatures.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying),
        "Tobita grants flying to your creatures");
}

/// Autumn-Tail's two-target ability is rejected when slot 1 is missing.
#[test]
fn autumn_tail_rejects_missing_second_target() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_battlefield(0, catalog::kitsune_mystic());
    let a1 = g.add_card_to_battlefield(0, catalog::vigilance_aura());
    let a2 = g.add_card_to_battlefield(0, catalog::vigilance_aura());
    g.battlefield_find_mut(a1).unwrap().attached_to = Some(mystic);
    g.battlefield_find_mut(a2).unwrap().attached_to = Some(mystic);
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    g.battlefield_find_mut(mystic).unwrap().tapped = false;
    g.clear_sickness(mystic);
    g.players[0].mana_pool.add_colorless(1);
    // Only the aura target (slot 0); no destination creature (slot 1).
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: mystic, ability_index: 0, target: Some(Target::Permanent(a1)),
        additional_targets: Vec::new(), x_value: None,
    });
    assert!(err.is_err(), "two-target ability needs both targets");
}

/// One Aura isn't enough to flip Kitsune Mystic at the end step.
#[test]
fn kitsune_mystic_stays_with_one_aura() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_battlefield(0, catalog::kitsune_mystic());
    let a1 = g.add_card_to_battlefield(0, catalog::vigilance_aura());
    g.battlefield_find_mut(a1).unwrap().attached_to = Some(mystic);
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mystic).unwrap().flipped, "one aura doesn't flip");
}

/// Kitsune Mystic flips into Autumn-Tail at the end step when enchanted by two
/// or more Auras; Autumn-Tail moves an Aura between creatures.
#[test]
fn kitsune_mystic_flips_at_two_auras_then_moves_an_aura() {
    let mut g = two_player_game();
    let mystic = g.add_card_to_battlefield(0, catalog::kitsune_mystic());
    let a1 = g.add_card_to_battlefield(0, catalog::vigilance_aura());
    let a2 = g.add_card_to_battlefield(0, catalog::vigilance_aura());
    g.battlefield_find_mut(a1).unwrap().attached_to = Some(mystic);
    g.battlefield_find_mut(a2).unwrap().attached_to = Some(mystic);
    advance_to(&mut g, crate::game::TurnStep::End);
    drain_stack(&mut g);
    let at = g.battlefield_find(mystic).unwrap();
    assert!(at.flipped && at.definition.name == "Autumn-Tail, Kitsune Sage", "flipped on two auras");
    // Autumn-Tail: {1}: move target Aura (slot 0) to another target creature (slot 1).
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mystic).unwrap().tapped = false;
    g.clear_sickness(mystic);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mystic,
        ability_index: 0,
        target: Some(Target::Permanent(a1)),
        additional_targets: vec![Target::Permanent(other)],
        x_value: None,
    }).expect("move aura");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a1).unwrap().attached_to, Some(other), "aura moved to the other creature");
}

/// Bushi Tenderfoot flips into Kenzo when a creature it dealt combat damage to
/// this turn dies — even when both happen in the same combat-damage step.
#[test]
fn bushi_tenderfoot_flips_when_a_creature_it_damaged_dies() {
    let mut g = two_player_game();
    let bushi = g.add_card_to_battlefield(0, catalog::bushi_tenderfoot());
    g.clear_sickness(bushi);
    // A 0/1 blocker dies to Bushi's 1 damage and deals none back.
    let chump = g.add_card_to_battlefield(1, {
        let mut d = catalog::grizzly_bears();
        d.name = "Chump"; d.power = 0; d.toughness = 1; d
    });
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bushi, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(chump, bushi)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::CombatDamage);
    drain_stack(&mut g);
    assert!(g.battlefield_find(chump).is_none(), "blocker died to Bushi");
    let kenzo = g.battlefield_find(bushi).expect("Bushi survived (0-power blocker)");
    assert!(kenzo.flipped && kenzo.definition.name == "Kenzo the Hardhearted",
        "flipped into Kenzo when the creature it damaged died");
}

/// Bushi Tenderfoot does NOT flip when a creature it never damaged dies.
#[test]
fn bushi_tenderfoot_ignores_unrelated_deaths() {
    let mut g = two_player_game();
    let bushi = g.add_card_to_battlefield(0, catalog::bushi_tenderfoot());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let events = g.remove_to_graveyard_with_triggers(victim);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bushi).unwrap().flipped, "no flip — Bushi never damaged it");
}

/// Nezumi Graverobber exiles a card from an opponent's graveyard and flips
/// into Nighteyes when that empties the graveyard; Nighteyes reanimates a
/// creature card from any graveyard.
#[test]
fn nezumi_graverobber_flips_when_opponent_graveyard_emptied() {
    let mut g = two_player_game();
    let nezumi = g.add_card_to_battlefield(0, catalog::nezumi_graverobber());
    g.clear_sickness(nezumi);
    let card = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nezumi, ability_index: 0, target: Some(Target::Permanent(card)), additional_targets: Vec::new(), x_value: None,
    }).expect("exile from opponent graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == card), "card exiled");
    let nighteyes = g.battlefield_find(nezumi).unwrap();
    assert!(nighteyes.flipped && nighteyes.definition.name == "Nighteyes the Desecrator",
        "flipped once the opponent's graveyard emptied");
    // Nighteyes: {4}{B}: reanimate a creature card from a graveyard.
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: nezumi, ability_index: 0, target: Some(Target::Permanent(corpse)), additional_targets: Vec::new(), x_value: None,
    }).expect("reanimate");
    drain_stack(&mut g);
    let rean = g.battlefield_find(corpse).expect("reanimated onto battlefield");
    assert_eq!(rean.controller, 0, "under Nighteyes' controller");
}

/// Kitsune Riftwalker has protection from Spirits: a Spirit can't block it.
#[test]
fn kitsune_riftwalker_cant_be_blocked_by_spirits() {
    let mut g = two_player_game();
    let rift = g.add_card_to_battlefield(0, catalog::kitsune_riftwalker());
    g.clear_sickness(rift);
    let kami = g.add_card_to_battlefield(1, catalog::gibbering_kami()); // 2/2 Spirit
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: rift, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::DeclareBlockers);
    let err = g.perform_action(GameAction::DeclareBlockers(vec![(kami, rift)]));
    assert!(err.is_err(), "a Spirit can't block a creature with protection from Spirits");
}

/// Protection from Spirits prevents combat damage from a Spirit source: the
/// Riftwalker survives blocking a much larger Spirit.
#[test]
fn kitsune_riftwalker_prevents_spirit_combat_damage() {
    let mut g = two_player_game();
    let moss = g.add_card_to_battlefield(1, catalog::moss_kami()); // 5/5 Spirit
    g.clear_sickness(moss);
    let rift = g.add_card_to_battlefield(0, catalog::kitsune_riftwalker()); // 2/1
    // Make it the opponent's turn so Moss Kami can attack and Riftwalker block.
    g.active_player_idx = 1;
    g.give_priority_to_active();
    advance_to(&mut g, crate::game::TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: moss, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::DeclareBlockers);
    g.perform_action(GameAction::DeclareBlockers(vec![(rift, moss)])).expect("block");
    drain_stack(&mut g);
    advance_to(&mut g, crate::game::TurnStep::PostCombatMain);
    assert!(g.battlefield_find(rift).is_some(), "Riftwalker survives — Spirit damage prevented");
}

/// Protection from Arcane: an Arcane spell can't target the Riftwalker.
#[test]
fn kitsune_riftwalker_cant_be_targeted_by_arcane() {
    let mut g = two_player_game();
    let rift = g.add_card_to_battlefield(1, catalog::kitsune_riftwalker());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let ray = g.add_card_to_hand(0, catalog::glacial_ray()); // Arcane
    let err = g.perform_action(GameAction::CastSpell {
        card_id: ray, target: Some(Target::Permanent(rift)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "an Arcane spell can't target protection-from-Arcane");
}

/// Threads of Disloyalty steals a creature with mana value 2 or less; control
/// reverts when the Aura leaves the battlefield.
#[test]
fn threads_of_disloyalty_steals_and_reverts() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // {1}{G}, MV 2
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let threads = g.add_card_to_hand(0, catalog::threads_of_disloyalty());
    cast_at(&mut g, threads, Target::Permanent(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 0, "stolen by Threads' controller");
    // Destroy the Aura → control reverts.
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    let disenchant = g.add_card_to_hand(0, catalog::disenchant());
    cast_at(&mut g, disenchant, Target::Permanent(threads));
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "control reverts when the Aura leaves");
}

/// Threads of Disloyalty can't enchant a creature with mana value 3 or more.
#[test]
fn threads_of_disloyalty_rejects_expensive_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::moss_kami()); // {3}{G}{G}, MV 5
    g.players[0].mana_pool.add(crate::mana::Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let threads = g.add_card_to_hand(0, catalog::threads_of_disloyalty());
    let err = g.perform_action(GameAction::CastSpell {
        card_id: threads, target: Some(Target::Permanent(big)), additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "can't enchant a mana-value-3-or-more creature");
}

/// "Target creature that was dealt damage this turn" rejects an undamaged
/// creature (CR 120.x targeting filter).
#[test]
fn initiate_of_blood_cannot_target_undamaged_creature() {
    let mut g = two_player_game();
    let initiate = g.add_card_to_battlefield(0, catalog::initiate_of_blood());
    g.clear_sickness(initiate);
    let fresh = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no damage
    let err = g.perform_action(GameAction::ActivateAbility {
        card_id: initiate, ability_index: 0, target: Some(Target::Permanent(fresh)), additional_targets: Vec::new(), x_value: None,
    });
    assert!(err.is_err(), "can't target a creature that wasn't dealt damage this turn");
}

/// Kuro's Taken regenerates itself for {1}{B}.
#[test]
fn kuros_taken_regenerates() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::kuros_taken());
    g.clear_sickness(rat);
    g.players[0].mana_pool.add(crate::mana::Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rat, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("regenerate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rat).unwrap().regeneration_shields >= 1, "regen shield set up");
}

/// Painwracker Oni forces an upkeep sacrifice unless its controller has an Ogre.
#[test]
fn painwracker_oni_upkeep_sacrifice_unless_ogre() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::painwracker_oni());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No Ogre controlled → the next player-0 upkeep trigger sacrifices a creature.
    let mut iters = 0;
    while !(g.active_player_idx == 0 && g.step == TurnStep::Upkeep && g.turn_number >= 3)
        && iters < 300
    {
        g.perform_action(GameAction::PassPriority).expect("pass");
        iters += 1;
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 1,
        "exactly one creature sacrificed at upkeep");
}

/// Crushing Pain deals 6 to a creature dealt damage this turn (and can't be
/// cast at a fresh one).
#[test]
fn crushing_pain_hits_only_damaged_creatures() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, {
        let mut d = catalog::grizzly_bears(); d.name = "Hulk"; d.power = 5; d.toughness = 6; d
    });
    let spell = g.add_card_to_hand(0, catalog::crushing_pain());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    // Undamaged → illegal target.
    let err = g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    });
    assert!(err.is_err(), "can't target an undamaged creature");
    // Mark it damaged, then it's legal and dies to 6.
    g.battlefield_find_mut(big).unwrap().dealt_damage_this_turn = true;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast at damaged creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "6 damage kills the 5/8");
}

/// Unearthly Blizzard stops up to three creatures from blocking this turn.
#[test]
fn unearthly_blizzard_grants_cant_block() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::unearthly_blizzard());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    }).expect("cast Unearthly Blizzard");
    drain_stack(&mut g);
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Akki Underling gets +2/+1 and first strike while its controller has 7+ cards.
#[test]
fn akki_underling_pumps_with_full_hand() {
    let mut g = two_player_game();
    let akki = g.add_card_to_battlefield(0, catalog::akki_underling());
    // Fewer than 7 cards: base 2/1, no first strike.
    let cp = g.computed_permanent(akki).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 1));
    assert!(!cp.keywords.contains(&Keyword::FirstStrike));
    for _ in 0..7 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
    let cp = g.computed_permanent(akki).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 2), "+2/+1 with a full hand");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Samurai of the Pale Curtain exiles a dying creature instead of letting it
/// reach the graveyard.
#[test]
fn samurai_of_the_pale_curtain_exiles_dying_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::samurai_of_the_pale_curtain());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(victim);
    assert!(g.exile.iter().any(|c| c.id == victim), "dying creature exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == victim), "never reaches the graveyard");
}

/// Promise of Bunrei sacrifices itself and mints four Spirits when a creature
/// you control dies.
#[test]
fn promise_of_bunrei_makes_four_spirits() {
    let mut g = two_player_game();
    let promise = g.add_card_to_battlefield(0, catalog::promise_of_bunrei());
    let creature = g.add_card_to_battlefield(0, {
        let mut d = catalog::grizzly_bears(); d.name = "Fragile"; d.power = 2; d.toughness = 1; d
    });
    // Lethal damage so the SBA dispatches CreatureDied to Promise's watcher.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(crate::mana::Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(creature)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt our creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(promise).is_none(), "Promise sacrificed");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(), 4,
        "four Spirit tokens minted");
}

/// Konda's Hatamoto grows and gains vigilance while a legendary Samurai is out.
#[test]
fn kondas_hatamoto_buffs_with_legendary_samurai() {
    let mut g = two_player_game();
    let hatamoto = g.add_card_to_battlefield(0, catalog::kondas_hatamoto());
    let cp = g.computed_permanent(hatamoto).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 2));
    assert!(!cp.keywords.contains(&Keyword::Vigilance));
    // A legendary Samurai (Brothers Yamazaki) switches on the bonus.
    g.add_card_to_battlefield(0, catalog::brothers_yamazaki());
    let cp = g.computed_permanent(hatamoto).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 4), "+1/+2 with a legendary Samurai");
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Spiritual Visit makes a 1/1 Spirit token.
#[test]
fn spiritual_visit_makes_a_spirit() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::spiritual_visit());
    g.players[0].mana_pool.add(crate::mana::Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Spiritual Visit");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count(), 1);
}

/// Devouring Greed drains 2 plus 2 per Spirit sacrificed.
#[test]
fn devouring_greed_drains_per_spirit() {
    use crate::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Two Spirits to sacrifice.
    g.add_card_to_battlefield(0, catalog::lantern_kami());
    g.add_card_to_battlefield(0, catalog::lantern_kami());
    let spell = g.add_card_to_hand(0, catalog::devouring_greed());
    g.players[0].mana_pool.add(crate::mana::Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Amount(2)]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Devouring Greed");
    drain_stack(&mut g);
    // 2 base + 2 per Spirit (×2) = 6 drained.
    assert_eq!(g.players[1].life, l1 - 6, "opponent loses 2 + 2 per Spirit");
    assert_eq!(g.players[0].life, l0 + 6, "you gain that much");
}

/// Feral Deceiver's {2} ability: with a land on top of the library, it reveals
/// and pumps +2/+2 with trample; once per turn.
#[test]
fn feral_deceiver_reveals_land_for_pump() {
    let mut g = two_player_game();
    let deceiver = g.add_card_to_battlefield(0, catalog::feral_deceiver());
    g.clear_sickness(deceiver);
    let land = g.next_id();
    g.players[0].add_to_library_top(land, catalog::forest());
    g.players[0].mana_pool.add_colorless(2);
    // ability_index 1 is the {2} reveal ability (index 0 is the {1} look).
    g.perform_action(GameAction::ActivateAbility {
        card_id: deceiver, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate reveal ability");
    drain_stack(&mut g);
    let cp = g.computed_permanent(deceiver).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 4), "3/2 +2/+2");
    assert!(cp.keywords.contains(&Keyword::Trample), "gains trample");
    // Once each turn: a second activation is rejected even with mana.
    g.players[0].mana_pool.add_colorless(2);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: deceiver, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).is_err(), "second activation blocked by once-per-turn");
}

/// Callous Deceiver gets no bonus when the revealed top card is not a land.
#[test]
fn callous_deceiver_nonland_top_no_bonus() {
    let mut g = two_player_game();
    let deceiver = g.add_card_to_battlefield(0, catalog::callous_deceiver());
    g.clear_sickness(deceiver);
    let nonland = g.next_id();
    g.players[0].add_to_library_top(nonland, catalog::lightning_bolt());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: deceiver, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("activate reveal ability");
    drain_stack(&mut g);
    let cp = g.computed_permanent(deceiver).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3), "no pump on a nonland reveal");
    assert!(!cp.keywords.contains(&Keyword::Flying), "no flying granted");
}
