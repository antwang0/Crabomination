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

// ── modern_decks: cube maybeboard additions ─────────────────────────────────

#[test]
fn bloodbraid_challenger_has_haste_and_cascades() {
    let mut g = two_player_game();
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // MV 2 nonland
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::bloodbraid_challenger());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable for {3}{R}{G}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == id), "Challenger resolves");
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&crabomination::card::Keyword::Haste),
        "has haste");
    assert!(g.battlefield.iter().any(|c| c.id == bears), "cascade casts the MV-2 card free");
}

#[test]
fn legion_extruder_etb_pings_and_makes_golems() {
    let mut g = two_player_game();
    let scrap = g.add_card_to_battlefield(0, catalog::ornithopter());
    let id = g.add_card_to_hand(0, catalog::legion_extruder());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let foe_life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::types::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, foe_life - 2, "ETB dealt 2 to the opponent");
    // {2}, {T}, Sacrifice another artifact (the Ornithopter): make a 3/3 Golem.
    g.players[0].mana_pool.add_colorless(2);
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("golem ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(scrap).is_none(), "sacrificed the other artifact");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Golem" && c.controller == 0),
        "minted a Golem token");
}

#[test]
fn dragonback_assault_sweeps_then_makes_dragons_on_landfall() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to 3
    let id = g.add_card_to_hand(0, catalog::dragonback_assault());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "ETB 3 damage killed the 2/2");
    // Landfall: play a land → 4/4 Dragon.
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("land drop");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Dragon" && c.controller == 0),
        "landfall minted a Dragon");
}

#[test]
fn landscape_cycle_taps_for_colorless_and_fetches_a_basic() {
    type LandscapeCase = (fn() -> crabomination::card::CardDefinition, &'static str);
    let cases: [LandscapeCase; 3] = [
        (catalog::twisted_landscape, "Swamp"),
        (catalog::sheltering_landscape, "Plains"),
        (catalog::bountiful_landscape, "Island"),
    ];
    for (factory, fetch_name) in cases {
        let mut g = two_player_game();
        // A basic of one of the three searchable types in the library.
        let basic = match fetch_name {
            "Swamp" => catalog::swamp(),
            "Plains" => catalog::plains(),
            _ => catalog::island(),
        };
        let target_basic = g.add_card_to_library(0, basic);
        let land = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(land);
        // Ability 0: {T}: Add {C}.
        g.perform_action(GameAction::ActivateAbility {
            card_id: land, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("taps for {C}");
        drain_stack(&mut g);
        assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "colorless produced");
        // Ability 1: {T}, Sac: search a basic onto the battlefield tapped.
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new(vec![
            DecisionAnswer::Search(Some(target_basic)),
        ]));
        let land2 = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(land2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: land2, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
        }).expect("sac-fetch");
        drain_stack(&mut g);
        assert!(g.battlefield.iter().any(|c| c.id == target_basic && c.tapped),
            "fetched basic enters tapped");
        assert!(g.battlefield_find(land2).is_none(), "the Landscape sacrificed itself");
    }
}

#[test]
fn enduring_vitality_grants_mana_and_returns_as_enchantment() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::enduring_vitality());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    // Granted "{T}: Add one mana of any color" (index 0 on the bear).
    let before = g.players[0].mana_pool.total();
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("granted mana ability");
    assert_eq!(g.players[0].mana_pool.total() - before, 1, "creature tapped for a mana");
    // Dies (Bolt the 3/3) → returns as a noncreature enchantment.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(id)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt Enduring Vitality");
    drain_stack(&mut g);
    let back = g.battlefield_find(id).expect("returned to the battlefield");
    assert!(back.definition.card_types.contains(&CardType::Enchantment));
    assert!(!back.definition.card_types.contains(&CardType::Creature),
        "comes back as a noncreature enchantment");
}

#[test]
fn fangkeepers_familiar_mode_zero_gains_life_and_surveils() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    let id = g.add_card_to_hand(0, catalog::fangkeepers_familiar());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Mode(0)]));
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("castable (Flash body)");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 3, "mode 0 gained 3 life");
    assert!(g.battlefield.iter().any(|c| c.id == id), "the Snake resolves");
}

#[test]
fn broodspinner_etb_surveils_and_sac_makes_insects_per_gy_type() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::broodspinner());
    g.clear_sickness(id);
    // Seed the graveyard with three distinct card types: creature, instant, land.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac for Insects");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "Broodspinner sacrificed itself");
    let insects = g.battlefield.iter().filter(|c| c.definition.name == "Insect" && c.controller == 0).count();
    assert_eq!(insects, 3, "one Insect per distinct gy card type (creature/instant/land)");
}

#[test]
fn trenchpost_mills_one_per_locus_you_control() {
    let mut g = two_player_game();
    // Three Locus lands you control → opponent mills three.
    let src = g.add_card_to_battlefield(0, catalog::trenchpost());
    g.add_card_to_battlefield(0, catalog::trenchpost());
    g.add_card_to_battlefield(0, catalog::trenchpost());
    g.clear_sickness(src);
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    g.players[0].mana_pool.add_colorless(3);
    let lib_before = g.players[1].library.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: src, ability_index: 1, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Trenchpost {3},{T} mills");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 3, "milled one card per Locus controlled");
}

#[test]
fn ouroboroid_counters_each_creature_by_its_own_power_at_combat() {
    let mut g = two_player_game();
    let ouro = g.add_card_to_battlefield(0, catalog::ouroboroid()); // 1/3
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    // X = Ouroboroid's power (1) → one counter on each of your creatures.
    assert_eq!(g.battlefield_find(ouro).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn railway_brawler_counters_new_creatures_by_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::railway_brawler());
    // Cast a 2/2 so it routes through the stack and fires the ETB event.
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "+1/+1 counters equal to the new creature's power");
}

#[test]
fn profts_eidetic_memory_etb_draws_and_grants_no_max_hand() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let id = g.add_card_to_battlefield(0, catalog::profts_eidetic_memory());
    let hand_before = g.players[0].hand.len();
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "ETB draws one");
    assert_eq!(g.players[0].max_hand_size, None, "no maximum hand size");
}

#[test]
fn profts_eidetic_memory_combat_counters_scale_with_draws() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::profts_eidetic_memory());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    // Draw three cards this turn (so X = 3 - 1 = 2).
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].cards_drawn_this_turn = 3;
    let _ = id;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "X = cards drawn this turn minus one");
}

#[test]
fn baloth_prime_enters_tapped_with_stun_and_sac_land_untaps() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::baloth_prime());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    {
        let c = g.battlefield_find(id).unwrap();
        assert!(c.tapped, "enters tapped");
        assert_eq!(c.counter_count(CounterType::Stun), 6, "six stun counters");
    }
    // Sacrifice a land via the activated ability → Beast + untap (removes a stun).
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let _ = land;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac a land for 2 life");
    drain_stack(&mut g);
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::Stun), 5, "untap replaced by removing a stun counter");
    let beasts = g.battlefield.iter().filter(|c| c.definition.name == "Beast" && c.controller == 0).count();
    assert_eq!(beasts, 1, "sacrificing a land mints a Beast");
    assert_eq!(g.players[0].life, 22, "gained 2 life");
}

#[test]
fn icetill_explorer_mills_on_landfall_and_grants_extra_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::icetill_explorer());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let lib_before = g.players[0].library.len();
    // Play a land → landfall mills one.
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "landfall mills a card");
}

#[test]
fn the_endstone_draws_on_land_and_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_endstone());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    let land = g.add_card_to_hand(0, catalog::forest());
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "playing a land draws a card");
}

#[test]
fn mightform_harmonizer_doubles_power_on_landfall() {
    let mut g = two_player_game();
    // Sole creature, so the auto-targeted "double power" lands on it.
    let mh = g.add_card_to_battlefield(0, catalog::mightform_harmonizer()); // 4/4
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == mh).unwrap();
    assert_eq!(c.power, 8, "power doubled (4 → 8) until end of turn");
    assert_eq!(c.toughness, 4, "toughness unchanged");
}

#[test]
fn pinnacle_emissary_makes_drone_on_artifact_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::pinnacle_emissary());
    let rock = g.add_card_to_hand(0, catalog::sol_ring()); // {1} artifact
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rock, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sol Ring castable");
    drain_stack(&mut g);
    let drones = g.battlefield.iter().filter(|c| c.definition.name == "Drone" && c.controller == 0).count();
    assert_eq!(drones, 1, "casting an artifact spell mints a Drone");
}

#[test]
fn metamorphosis_fanatic_reanimates_with_lifelink_counter() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::metamorphosis_fanatic());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(bear).expect("bear returned to battlefield");
    assert_eq!(reanimated.controller, 0, "under your control");
    assert!(reanimated.has_keyword(&crabomination::card::Keyword::Lifelink), "lifelink counter grants lifelink");
}

#[test]
fn springleaf_parade_makes_x_changelings_that_tap_for_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::springleaf_parade());
    // {X}{G}{G} with X=2.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("castable for X=2");
    drain_stack(&mut g);
    let tokens: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Shapeshifter" && c.controller == 0).collect();
    assert_eq!(tokens.len(), 2, "X=2 → two Shapeshifter tokens");
    let tok = tokens[0].id;
    assert!(tokens[0].has_keyword(&crabomination::card::Keyword::Changeling), "changeling");
    // The granted mana ability lets a token tap for any color.
    g.clear_sickness(tok);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: tok, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("token mana ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Blue), 1, "token taps for a color");
}

#[test]
fn static_prison_exiles_and_gives_energy() {
    let mut g = two_player_game();
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::static_prison());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == prey), "an opponent permanent is exiled");
    assert_eq!(g.players[0].energy, 2, "you get two energy");
    // When it leaves, the exiled creature returns.
    g.remove_to_graveyard_with_triggers(id);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == prey), "exiled card returns when Static Prison leaves");
}

#[test]
fn shorikai_draws_two_discards_one_and_makes_a_pilot() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::shorikai_genesis_engine());
    g.clear_sickness(id);
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.add_card_to_hand(0, catalog::forest()); // a card to discard
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Shorikai activates");
    drain_stack(&mut g);
    // Net hand: +2 draw, -1 discard = +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "draw two, discard one");
    let pilots = g.battlefield.iter().filter(|c| c.definition.name == "Pilot" && c.controller == 0).count();
    assert_eq!(pilots, 1, "mints a Pilot token");
}

#[test]
fn kestia_draws_when_an_enchantment_creature_attacks() {
    let mut g = two_player_game();
    let kestia = g.add_card_to_battlefield(0, catalog::kestia_the_cultivator());
    g.clear_sickness(kestia);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    // Kestia (an enchantment creature you control) attacks → draw a card.
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: kestia, target: AttackTarget::Player(1) }]))
        .expect("Kestia attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "attack draws a card");
}

#[test]
fn kestia_draws_when_an_enchanted_creature_attacks() {
    // A vanilla creature wearing an Aura is "enchanted" → Kestia's board-wide
    // trigger draws when it attacks.
    let mut g = two_player_game();
    let _kestia = g.add_card_to_battlefield(0, catalog::kestia_the_cultivator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::gift_of_orzhova());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("enchanted bear attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "enchanted attacker draws");
}

#[test]
fn kestia_no_draw_when_a_plain_creature_attacks() {
    // A vanilla, unenchanted creature attacking does NOT satisfy Kestia.
    let mut g = two_player_game();
    let _kestia = g.add_card_to_battlefield(0, catalog::kestia_the_cultivator());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("plain bear attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before, "plain attacker draws nothing");
}

#[test]
fn urza_chief_artificer_grants_menace_and_makes_scaling_construct() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let urza = g.add_card_to_battlefield(0, catalog::urza_chief_artificer());
    let _ = urza;
    // An artifact creature you control gains menace.
    let memnite = g.add_card_to_battlefield(0, catalog::memnite()); // 1/1 artifact creature
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == memnite).unwrap().keywords.contains(&Keyword::Menace),
        "artifact creatures you control have menace");
    // End step → a Construct sized by artifacts you control (Urza is not an
    // artifact; Memnite + the Construct itself + ... count artifacts).
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let construct = g.battlefield.iter().find(|c| c.definition.name == "Construct" && c.controller == 0)
        .expect("a Construct token was created");
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == construct.id).unwrap();
    // Artifacts you control: Memnite + the Construct = 2 → 2/2.
    assert_eq!((c.power, c.toughness), (2, 2), "Construct sized by artifacts controlled");
}

#[test]
fn korvold_enters_sacrifices_and_grows_with_draw() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // A spare permanent for Korvold's ETB to sacrifice, plus a card to draw.
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let id = g.add_card_to_hand(0, catalog::korvold_fae_cursed_king());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    cast(&mut g, id);
    let korvold = g.battlefield_find(id).expect("Korvold on bf");
    // ETB sacrifices the Forest → +1/+1 counter and a draw.
    assert_eq!(korvold.counter_count(CounterType::PlusOnePlusOne), 1, "grows on sacrifice");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Forest"), "Forest sacrificed");
    // -1 cast (Korvold), +1 draw = net same hand size.
    assert_eq!(g.players[0].hand.len(), hand_before, "sacrifice payoff drew a card");
}

#[test]
fn maelstrom_nexus_gives_first_spell_cascade() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::maelstrom_nexus());
    // Top of library: a cheaper nonland (Memnite, MV 0) for the cascade.
    let memnite = g.add_card_to_library(0, catalog::memnite());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    // First spell of the turn — a 2-MV creature — gains cascade.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == memnite),
        "first spell cascades into the cheaper Memnite");
}

#[test]
fn maelstrom_nexus_only_first_spell_each_turn() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::maelstrom_nexus());
    let memnite = g.add_card_to_library(0, catalog::memnite());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Bool(true)]));
    // Burn the "first spell" on a cheap spell with nothing to cascade-bottom.
    let first = g.add_card_to_hand(0, catalog::memnite());
    cast(&mut g, first);
    // Second spell this turn: no cascade.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears castable");
    drain_stack(&mut g);
    assert!(g.players[0].library.iter().any(|c| c.id == memnite),
        "second spell does not cascade; library Memnite untouched");
}

#[test]
fn backup_conclave_sledge_captain_counters_and_grants_trample() {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // A separate creature for backup to target (and gain Trample).
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::conclave_sledge_captain());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(5);
    // Three Backup-1 ETB triggers each pick the bear.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Target(Target::Permanent(bear)),
        DecisionAnswer::Target(Target::Permanent(bear)),
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    cast(&mut g, id);
    let b = g.battlefield_find(bear).expect("bear on bf");
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 3, "three Backup-1 counters");
    let cp = g.compute_battlefield();
    assert!(cp.iter().find(|c| c.id == bear).unwrap().keywords.contains(&Keyword::Trample),
        "backed-up creature gains Trample until end of turn");
}

#[test]
fn backup_bola_slinger_grants_attack_trigger() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let enemy = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // tap target
    let id = g.add_card_to_hand(0, catalog::bola_slinger());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    // Backup ETB targets the bear; the on-attack tap auto-targets the enemy.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Target(Target::Permanent(bear)),
    ]));
    cast(&mut g, id);
    // The bear now carries the granted "on attack, tap target opp permanent".
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack with backed-up bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(enemy).unwrap().tapped,
        "granted attack trigger taps an opponent's permanent");
}

#[test]
fn backup_death_greeters_champion_self_counter() {
    use crabomination::card::{CounterType, Keyword};
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::death_greeters_champion());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    // Backup targets itself → +1/+1 counter, already double strike.
    g.decider = Box::new(ScriptedDecider::new(vec![
        DecisionAnswer::Target(Target::Permanent(id)),
    ]));
    cast(&mut g, id);
    let c = g.battlefield_find(id).expect("champion on bf");
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 1, "self backup counter");
    assert!(c.has_keyword(&Keyword::DoubleStrike));
}

#[test]
fn shiko_flurry_copies_targeted_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shiko_and_narset_unified());
    g.active_player_idx = 0;
    // First spell of the turn (no target).
    let m = g.add_card_to_hand(0, catalog::memnite());
    cast(&mut g, m);
    // Second spell targets a player → Flurry copies it (AutoDecider keeps target).
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let before = g.players[1].life;
    cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.players[1].life, before - 6, "Flurry copies the bolt → 3+3 damage");
}

#[test]
fn shiko_flurry_draws_on_nontargeted_second_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::shiko_and_narset_unified());
    g.active_player_idx = 0;
    let drawn = g.add_card_to_library(0, catalog::island());
    let m1 = g.add_card_to_hand(0, catalog::memnite());
    cast(&mut g, m1);
    // Second spell has no target → Flurry draws instead of copying.
    let m2 = g.add_card_to_hand(0, catalog::memnite());
    cast(&mut g, m2);
    assert!(g.players[0].hand.iter().any(|c| c.id == drawn),
        "no-target second spell draws a card");
}

#[test]
fn parallax_dementia_destroys_enchanted_creature_when_it_leaves() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::parallax_dementia());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    g.battlefield_find_mut(aura).unwrap().add_counters(CounterType::Fade, 1);
    // Enchanted creature gets +3/+2.
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (5, 4), "+3/+2 from Parallax Dementia");
    g.active_player_idx = 0;
    // Upkeep tick 1: removes the lone fade counter (aura survives).
    let _ = g.process_fading_vanishing();
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_some(), "aura survives the first Fading tick");
    // Upkeep tick 2: no counter to remove → sacrifice → LTB destroys the creature.
    let _ = g.process_fading_vanishing();
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "aura sacrificed by Fading");
    assert!(g.battlefield_find(bear).is_none(), "enchanted creature destroyed when aura leaves");
}

#[test]
fn mutable_explorer_makes_tapped_mutavault_token() {
    use crabomination::card::{CardType, Keyword};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::mutable_explorer());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    // Explorer is a Changeling body.
    assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Changeling));
    // ETB minted a tapped Mutavault land token.
    let mv = g.battlefield.iter()
        .find(|c| c.definition.name == "Mutavault" && c.controller == 0)
        .expect("Mutavault token minted");
    assert!(mv.definition.card_types.contains(&CardType::Land));
    assert!(mv.tapped, "Mutavault token enters tapped");
    assert!(mv.is_token);
}

#[test]
fn teval_loses_life_equal_to_cast_spell_mana_value() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let teval = g.add_card_to_battlefield(0, catalog::teval_arbiter_of_virtue());
    assert!(g.battlefield_find(teval).unwrap().has_keyword(&Keyword::Flying));
    g.active_player_idx = 0;
    let life_before = g.players[0].life;
    // Cast a 2-MV spell → lose 2 life.
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, bears);
    assert_eq!(g.players[0].life, life_before - 2, "lose life = cast spell's mana value");
}

#[test]
fn teval_grants_spells_delve() {
    // CR 702.66 — Teval's static lets any spell pay generic with graveyard
    // exile even though Grizzly Bears isn't printed with delve.
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teval_arbiter_of_virtue());
    let gy: Vec<_> = (0..2).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    // Only one green mana — the {1} generic must come from delving one card.
    g.players[0].mana_pool.add(Color::Green, 1);
    let exile_before = g.exile.len();
    g.perform_action(GameAction::CastSpellDelve {
        card_id: bears, target: None, additional_targets: vec![],
        mode: None, x_value: None, delve_cards: vec![gy[0]],
    }).expect("Bears castable for G after delving one (Teval grants delve)");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
    assert_eq!(g.exile.len(), exile_before + 1, "one delved card moved to exile");
}

#[test]
fn sab_sunen_draws_two_on_odd_counter_main_phase() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sab_sunen_luxa_embodied());
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.active_player_idx = 0;
    let hand0 = g.players[0].hand.len();
    // Main phase tick 1: 0 → 1 counter (odd) → draw 2.
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "odd counter count draws two");
    // Main phase tick 2: 1 → 2 counters (even) → no draw.
    let hand1 = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
    assert_eq!(g.players[0].hand.len(), hand1, "even counter count draws nothing");
}

#[test]
fn sab_sunen_cant_attack_with_odd_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sab_sunen_luxa_embodied());
    g.clear_sickness(id);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    // Zero counters is even → may attack.
    assert!(g.legal_attackers(0).contains(&id), "even (zero) counters → can attack");
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }])
        .expect("even-counter Sab-Sunen attacks");
    // One counter is odd → can't attack.
    g.attacking.clear();
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert!(!g.legal_attackers(0).contains(&id), "odd counters → not a legal attacker");
    let err = g
        .declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }])
        .unwrap_err();
    assert!(matches!(err, GameError::CannotAttack(x) if x == id));
}

#[test]
fn nettlecyst_living_weapon_scales_germ_by_artifacts_and_enchantments() {
    let mut g = two_player_game();
    // An extra artifact + a (non-Aura) enchantment already on the battlefield.
    g.add_card_to_battlefield(0, catalog::memnite()); // artifact creature
    g.add_card_to_battlefield(0, catalog::maelstrom_nexus()); // enchantment
    let id = g.add_card_to_hand(0, catalog::nettlecyst());
    g.players[0].mana_pool.add_colorless(3);
    // Cast Nettlecyst → living-weapon ETB mints a Germ and attaches.
    cast(&mut g, id);
    let germ = g.battlefield.iter()
        .find(|c| c.definition.name == "Phyrexian Germ" && c.controller == 0)
        .expect("living weapon mints a Germ");
    assert_eq!(g.battlefield_find(id).unwrap().attached_to, Some(germ.id),
        "Nettlecyst attaches to its Germ");
    // Artifacts/enchantments you control: Memnite + Nettlecyst (artifacts) +
    // Maelstrom Nexus (enchantment) = 3 → Germ is 3/3.
    let cp = g.compute_battlefield();
    let g_cp = cp.iter().find(|c| c.id == germ.id).unwrap();
    assert_eq!((g_cp.power, g_cp.toughness), (3, 3),
        "Germ scales +1/+1 per artifact/enchantment you control");
}

#[test]
fn helm_of_the_host_copies_equipped_creature_with_haste() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let helm = g.add_card_to_battlefield(0, catalog::helm_of_the_host());
    g.battlefield_find_mut(helm).unwrap().attached_to = Some(bear);
    g.active_player_idx = 0;
    let bears_before = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    // Begin combat → mint a hasty token copy of the equipped Bear.
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let bears_after = g.battlefield.iter().filter(|c| c.definition.name == "Grizzly Bears").count();
    assert_eq!(bears_after, bears_before + 1, "Helm mints a token copy of the equipped creature");
    let token = g.battlefield.iter()
        .find(|c| c.definition.name == "Grizzly Bears" && c.is_token)
        .expect("token copy exists");
    assert!(token.has_keyword(&Keyword::Haste), "the copy has haste");
}

/// CR 707.2 — the token copies the host's *copiable* values, so counters and
/// until-end-of-turn pumps on the host don't ride along.
#[test]
fn helm_of_the_host_copies_only_copiable_values() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().counters.insert(CounterType::PlusOnePlusOne, 3);
    g.battlefield_find_mut(bear).unwrap().power_bonus = 5;
    let helm = g.add_card_to_battlefield(0, catalog::helm_of_the_host());
    g.battlefield_find_mut(helm).unwrap().attached_to = Some(bear);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let token = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears" && c.is_token)
        .expect("token copy");
    assert_eq!(
        g.computed_permanent(token.id).map(|c| (c.power, c.toughness)),
        Some((2, 2)),
        "a plain 2/2 — the host's counters and pump aren't copiable"
    );
}

/// CR 707.2e — Helm's copy of a *legendary* host isn't legendary, so the
/// legend-rule SBA doesn't destroy the original alongside it.
#[test]
fn helm_of_the_host_copy_of_legend_is_not_legendary() {
    use crabomination::card::Supertype;
    let mut g = two_player_game();
    let krenko = g.add_card_to_battlefield(0, catalog::krenko_mob_boss());
    let helm = g.add_card_to_battlefield(0, catalog::helm_of_the_host());
    g.battlefield_find_mut(helm).unwrap().attached_to = Some(krenko);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    let krenkos: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Krenko, Mob Boss").collect();
    assert_eq!(krenkos.len(), 2, "both the host and its non-legendary copy survive");
    let token = krenkos.iter().find(|c| c.is_token).expect("token copy");
    assert!(!token.definition.supertypes.contains(&Supertype::Legendary),
        "the token copy isn't legendary");
}

#[test]
fn lion_sash_exiles_permanent_card_grows_and_scales_equipped() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let sash = g.add_card_to_battlefield(0, catalog::lion_sash());
    // A permanent (creature) card in the opponent's graveyard.
    let gy = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: sash, ability_index: 0, target: Some(Target::Permanent(gy)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate exile ability");
    drain_stack(&mut g);
    // Card exiled, Lion Sash grew a +1/+1 counter (now 2/2).
    assert!(g.exile.iter().any(|c| c.id == gy), "target card exiled");
    assert_eq!(g.battlefield_find(sash).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    // Reconfigure onto a creature via the Equip path (Reconfigure reuses it).
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: sash, target: bear })
        .expect("Reconfigure attaches via the equip path");
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (3, 3), "equipped scales by Lion Sash's counter");
    // CR 702.151c — Lion Sash isn't a creature while attached.
    let s = cp.iter().find(|c| c.id == sash).unwrap();
    assert!(!s.card_types.contains(&crabomination::card::CardType::Creature),
        "Reconfigure Equipment isn't a creature while attached");
    assert!(s.card_types.contains(&crabomination::card::CardType::Artifact), "still an artifact");
}

/// CR 702.151 — Reconfigure unattach detaches the Equipment and restores its
/// creature-ness.
#[test]
fn reconfigure_unattach_restores_creatureness() {
    let mut g = two_player_game();
    let sash = g.add_card_to_battlefield(0, catalog::lion_sash());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // Attach first.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Reconfigure { equipment: sash, target: Some(bear) })
        .expect("reconfigure attach");
    assert_eq!(g.battlefield_find(sash).unwrap().attached_to, Some(bear));
    // Unattach: it becomes a creature again.
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Reconfigure { equipment: sash, target: None })
        .expect("reconfigure unattach");
    assert_eq!(g.battlefield_find(sash).unwrap().attached_to, None);
    let s = g.computed_permanent(sash).unwrap();
    assert!(s.card_types.contains(&crabomination::card::CardType::Creature),
        "unattached Reconfigure Equipment is a creature again");
}

#[test]
fn leyline_of_the_guildpact_makes_your_lands_all_basic_types() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::leyline_of_the_guildpact());
    let cp = g.compute_battlefield();
    let f = cp.iter().find(|c| c.id == forest).unwrap();
    for lt in [LandType::Plains, LandType::Island, LandType::Swamp, LandType::Mountain, LandType::Forest] {
        assert!(f.subtypes.land_types.contains(&lt), "Forest gains {lt:?}");
    }
    // It now taps for any color via intrinsic basic-land mana abilities.
    let abilities = g.effective_mana_abilities(forest);
    assert!(abilities.len() >= 5, "taps for all five colors, got {}", abilities.len());
}

#[test]
fn leyline_of_the_guildpact_makes_your_permanents_all_colors() {
    let mut g = two_player_game();
    // A colorless artifact-ish body: a plain Forest is mono-green; with the
    // Leyline it should read as all five colors.
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::leyline_of_the_guildpact());
    let cp = g.compute_battlefield();
    let f = cp.iter().find(|c| c.id == forest).unwrap();
    for col in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        assert!(f.colors.contains(&col), "your land is now {col:?}");
    }
}

#[test]
fn consult_the_star_charts_digs_x_equal_to_lands() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    // Three lands → look at top three.
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let want = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(want))]));
    let id = g.add_card_to_hand(0, catalog::consult_the_star_charts());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, id);
    assert!(g.players[0].hand.iter().any(|c| c.id == want),
        "picks the chosen card from the top three into hand");
}

#[test]
fn consult_the_star_charts_kicked_takes_two() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let want = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new(vec![DecisionAnswer::Search(Some(want))]));
    let id = g.add_card_to_hand(0, catalog::consult_the_star_charts());
    let hand0 = g.players[0].hand.len();
    // Pay base {1}{U} + kicker {1}{U} = {2}{U}{U}.
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2,
        "kicked Consult puts two cards into hand");
    assert!(g.players[0].hand.iter().any(|c| c.id == want), "chosen card is one of the two");
}

#[test]
fn power_depot_enters_tapped_and_fixes_mana() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::power_depot());
    // Entered via add_card_to_battlefield doesn't run ETB; verify the tap
    // trigger fires through the normal play path instead.
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).unwrap().tapped, "Power Depot enters tapped");
    assert!(g.battlefield_find(id).unwrap().definition.card_types.contains(&crabomination::card::CardType::Artifact));
    // Untap and tap for any color (the second ability) — the pip is
    // spend-restricted to artifact spells / artifact abilities.
    g.battlefield_find_mut(id).unwrap().tapped = false;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: Some(0), mode: None,
    }).expect("tap for any color");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.restricted_total(), 1, "restricted artifact-only pip");
}

#[test]
fn power_depot_mana_spends_only_on_artifacts() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::power_depot());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::Green),
        DecisionAnswer::Color(Color::Green),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for restricted mana");

    // Can't fund a creature spell…
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
        })
        .is_err(),
        "artifact-only mana can't fund a creature spell"
    );

    // …but funds an artifact spell.
    g.battlefield_find_mut(id).unwrap().tapped = false;
    let stone = g.add_card_to_hand(0, catalog::mind_stone());
    g.perform_action(GameAction::CastSpell {
        card_id: stone, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("artifact-only mana + {C} covers Mind Stone's {2}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == stone), "Mind Stone resolved");
}

#[test]
fn virtue_of_loyalty_adventure_makes_knight_then_enchantment_buffs() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    // Adventure side: cast Ardenvale Fealty → 2/2 vigilant Knight + exile card.
    let id = g.add_card_to_hand(0, catalog::virtue_of_loyalty());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastAdventure {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ardenvale Fealty");
    drain_stack(&mut g);
    let (knight_id, knight_vigilant) = {
        let knight = g.battlefield.iter().find(|c| c.definition.name == "Knight" && c.controller == 0)
            .expect("Knight token minted");
        (knight.id, knight.has_keyword(&Keyword::Vigilance))
    };
    assert!(knight_vigilant);
    assert!(g.exile.iter().any(|c| c.id == id), "adventure card exiled, castable later");

    // Enchantment side: cast it from exile, then end step buffs/untaps creatures.
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastAdventureCreature {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Virtue of Loyalty enchantment");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "enchantment resolves onto battlefield");
    g.battlefield_find_mut(knight_id).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    let k = g.battlefield_find(knight_id).unwrap();
    assert_eq!(k.counter_count(CounterType::PlusOnePlusOne), 1, "end step grows each creature");
    assert!(!k.tapped, "end step untaps those creatures");
}

#[test]
fn minds_desire_storm_exiles_top_cards_with_free_play() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.active_player_idx = 0;
    // Cast a prior spell so Storm copies Mind's Desire once (2 resolutions).
    let m = g.add_card_to_hand(0, catalog::memnite());
    cast(&mut g, m);
    let id = g.add_card_to_hand(0, catalog::minds_desire());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, id);
    // Two cards exiled (original + one Storm copy), each playable for free.
    let free = g.exile.iter().filter(|c| c.may_play_until.is_some()).count();
    assert_eq!(free, 2, "Storm exiles top N cards with may-play-free, got {free}");
}

#[test]
fn sword_of_body_and_mind_buffs_and_grants_double_protection() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_body_and_mind());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(bear);
    let cp = g.compute_battlefield();
    let b = cp.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((b.power, b.toughness), (4, 4), "+2/+2 from Sword");
    assert!(b.keywords.contains(&Keyword::Protection(Color::Green)));
    assert!(b.keywords.contains(&Keyword::Protection(Color::Blue)));
}

#[test]
fn sword_of_body_and_mind_combat_damage_makes_wolf_and_mills() {
    let mut g = two_player_game();
    // Shadow → unblockable, so combat damage reaches the player.
    let attacker = g.add_card_to_battlefield(0, catalog::looter_il_kor());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_body_and_mind());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(attacker);
    for _ in 0..12 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let opp_gy = g.players[1].graveyard.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[1].graveyard.len() >= opp_gy + 10 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Wolf" && c.controller == 0),
        "Sword mints a Wolf on combat damage");
    assert!(g.players[1].graveyard.len() >= opp_gy + 10, "defender milled ten");
}

/// Umezawa's Jitte gains two charge counters when the equipped creature deals
/// combat damage to a player, then a charge counter can be spent for -1/-1.
#[test]
fn umezawas_jitte_charges_on_combat_damage_then_shrinks() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::looter_il_kor()); // shadow → unblockable
    for _ in 0..5 { g.add_card_to_library(0, catalog::grizzly_bears()); } // loot needs a library
    let jitte = g.add_card_to_battlefield(0, catalog::umezawas_jitte());
    g.battlefield_find_mut(jitte).unwrap().attached_to = Some(attacker);
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.battlefield_find(jitte).map(|c| c.counter_count(CounterType::Charge)).unwrap_or(0) >= 2 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(jitte).unwrap().counter_count(CounterType::Charge), 2,
        "two charge counters from combat damage");
    // Spend a charge counter to shrink an opponent's creature.
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::ActivateAbility {
        card_id: jitte, ability_index: 1, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate -1/-1");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jitte).unwrap().counter_count(CounterType::Charge), 1,
        "one charge counter spent");
    let v = g.battlefield_find(victim).unwrap();
    assert_eq!((v.power(), v.toughness()), (1, 1), "2/2 → 1/1 from -1/-1");
}

#[test]
fn sword_of_feast_and_famine_discards_and_untaps_lands() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::looter_il_kor());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_feast_and_famine());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(attacker);
    g.add_card_to_hand(1, catalog::grizzly_bears()); // defender's card to discard
    let tapped_forest = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(tapped_forest).unwrap().tapped = true;
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let opp_hand = g.players[1].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[1].hand.len() < opp_hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.players[1].hand.len() < opp_hand, "defender discarded a card");
    assert!(!g.battlefield_find(tapped_forest).unwrap().tapped, "your land untapped");
}

#[test]
fn sword_of_war_and_peace_burns_by_hand_and_gains_life() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::looter_il_kor());
    let sword = g.add_card_to_battlefield(0, catalog::sword_of_war_and_peace());
    g.battlefield_find_mut(sword).unwrap().attached_to = Some(attacker);
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); } // defender hand = 3
    for _ in 0..2 { g.add_card_to_hand(0, catalog::forest()); } // your hand contributes life
    g.clear_sickness(attacker);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let opp_life = g.players[1].life;
    let my_life = g.players[0].life;
    let my_hand = g.players[0].hand.len() as i32;
    let opp_hand = g.players[1].hand.len() as i32;
    // Equipped (Sword grants +2/+2), so combat damage is the buffed power.
    let combat = g.compute_battlefield().iter().find(|c| c.id == attacker).unwrap().power;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..14 {
        if g.players[0].life > my_life { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, opp_life - combat - opp_hand, "burned by defender's hand size");
    assert_eq!(g.players[0].life, my_life + my_hand, "gained life by your hand size");
}

// ── Soulbond (CR 702.95) ─────────────────────────────────────────────────────

/// Wolfir Silverheart pairs with a creature on ETB and gives both +4/+4.
#[test]
fn soulbond_wolfir_silverheart_pairs_and_buffs_both() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wolfir = g.add_card_to_hand(0, catalog::wolfir_silverheart());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, wolfir); // ETB pairing auto-resolves
    assert_eq!(g.battlefield_find(wolfir).unwrap().soulbond_partner, Some(bear));
    assert_eq!(g.battlefield_find(bear).unwrap().soulbond_partner, Some(wolfir));
    let cw = g.computed_permanent(wolfir).unwrap();
    let cb = g.computed_permanent(bear).unwrap();
    assert_eq!((cw.power, cw.toughness), (8, 8));
    assert_eq!((cb.power, cb.toughness), (6, 6));
}

/// Wingcrafter grants flying to both members of its pair.
#[test]
fn soulbond_wingcrafter_grants_flying_to_both() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wing = g.add_card_to_hand(0, catalog::wingcrafter());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, wing);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Flying));
    assert!(g.computed_permanent(wing).unwrap().keywords.contains(&Keyword::Flying));
}

/// Nightshade Peddler grants deathtouch to both members of its pair.
#[test]
fn soulbond_nightshade_peddler_grants_deathtouch() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let peddler = g.add_card_to_hand(0, catalog::nightshade_peddler());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, peddler);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// Hanweir Lancer grants first strike to both members of its pair.
#[test]
fn soulbond_hanweir_lancer_grants_first_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lancer = g.add_card_to_hand(0, catalog::hanweir_lancer());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, lancer);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::FirstStrike));
    assert!(g.computed_permanent(lancer).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// A Soulbond pair breaks when one member leaves: the bonus and link both clear.
#[test]
fn soulbond_pair_breaks_when_partner_leaves() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let forcemage = g.add_card_to_hand(0, catalog::trusted_forcemage());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, forcemage);
    assert_eq!(g.computed_permanent(forcemage).unwrap().power, 3, "paired +1/+1");
    // Kill the bear; the SBA clears the surviving member's link.
    g.battlefield_find_mut(bear).unwrap().damage = 99;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(forcemage).unwrap().soulbond_partner, None);
    assert_eq!(g.computed_permanent(forcemage).unwrap().power, 2, "bonus gone once unpaired");
}

/// Deadeye Navigator grants its partner the "{1}{U}: flicker this" activated
/// ability; activating it exiles and returns the creature.
#[test]
fn soulbond_deadeye_navigator_grants_flicker_to_partner() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let deadeye = g.add_card_to_hand(0, catalog::deadeye_navigator());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, deadeye);
    assert_eq!(g.battlefield_find(bear).unwrap().soulbond_partner, Some(deadeye));
    // The partner Bear now has one granted activated ability (index 0).
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bear, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("flicker the partner");
    drain_stack(&mut g);
    // It re-enters the battlefield (a Grizzly Bears is still in play).
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "flickered creature returns to the battlefield");
}

// ── Mentor (CR 702.134) ──────────────────────────────────────────────────────

/// Hammer Dropper's Mentor puts a +1/+1 counter on a lesser-power co-attacker.
#[test]
fn mentor_hammer_dropper_counters_lesser_attacker() {
    let mut g = two_player_game();
    let dropper = g.add_card_to_battlefield(0, catalog::hammer_dropper()); // 5/2
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(dropper).unwrap().summoning_sick = false;
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: dropper, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Mentor counters the lesser-power attacker");
}

/// Goblin Banneret's `{1}{R}: +2/+0` pump resolves.
#[test]
fn goblin_banneret_pump_ability() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::goblin_banneret());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("pump");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "1/1 → 3/1");
}

// ── Afflict / Provoke / Soulshift (existing primitives, now carded) ──────────

/// Khenra Eternal's Afflict 1 drains the defender when it becomes blocked.
#[test]
fn afflict_khenra_eternal_drains_on_block() {
    let mut g = two_player_game();
    let khenra = g.add_card_to_battlefield(0, catalog::khenra_eternal());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(khenra).unwrap().summoning_sick = false;
    let life_before = g.players[1].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: khenra, target: AttackTarget::Player(1) }])
        .expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, khenra)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 1, "Afflict 1 drains the blocking player");
}

/// Crested Craghorn's Provoke untaps and forces a defender to block.
#[test]
fn provoke_crested_craghorn_forces_block() {
    let mut g = two_player_game();
    let craghorn = g.add_card_to_battlefield(0, catalog::crested_craghorn());
    let defender = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(defender).unwrap().tapped = true;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: craghorn, target: AttackTarget::Player(1) }])
        .expect("attack"); // Craghorn has haste
    drain_stack(&mut g);
    // Provoke untaps the target and sets a must-block requirement on it.
    assert!(!g.battlefield_find(defender).unwrap().tapped, "Provoke untaps the target");
    assert_eq!(g.battlefield_find(defender).unwrap().must_block, Some(craghorn),
        "Provoke forces the target to block the attacker");
}

/// Hundred-Talon Kami's Soulshift 4 returns a small Spirit on death.
#[test]
fn soulshift_hundred_talon_kami_returns_spirit() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // accept Soulshift
    // A cheap (MV-2) Spirit in the graveyard for Soulshift 4 to retrieve.
    let spirit = g.add_card_to_battlefield(0, catalog::mistwalker());
    g.remove_from_battlefield_to_graveyard_raw(spirit);
    let kami = g.add_card_to_battlefield(0, catalog::hundred_talon_kami());
    g.battlefield_find_mut(kami).unwrap().damage = 99; // lethal → dies → Soulshift 4
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == spirit),
        "Soulshift returns the MV-2 Spirit to hand");
}

/// Marsh Viper's Poisonous 2 (CR 702.70) gives the damaged player two poison.
#[test]
fn poisonous_marsh_viper_adds_two_poison_on_combat_damage() {
    let mut g = two_player_game();
    let viper = g.add_card_to_battlefield(0, catalog::marsh_viper());
    g.clear_sickness(viper);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: viper, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g); // Poisonous is a triggered ability — resolve it off the stack
    assert_eq!(g.players[1].poison_counters, 2, "Poisonous 2 adds two poison counters");
}

/// Frenzy Sliver's Frenzy 1 (CR 702.68) pumps it +1/+0 when unblocked.
#[test]
fn frenzy_sliver_pumps_when_unblocked() {
    let mut g = two_player_game();
    let sliver = g.add_card_to_battlefield(0, catalog::frenzy_sliver());
    g.clear_sickness(sliver);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sliver, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(sliver).unwrap().power(), 2,
        "Frenzy 1 pumps unblocked attacker to 2 power");
}

/// Ominous Harvest's Gravestorm (CR 702.69) copies once per permanent that
/// died this turn.
#[test]
fn gravestorm_ominous_harvest_copies_per_dead_permanent() {
    let mut g = two_player_game();
    // Two permanents die this turn → two Gravestorm copies + the original = 3
    // resolutions, each "target player draws 1, loses 1".
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(a);
    g.remove_from_battlefield_to_graveyard_raw(b);
    assert_eq!(g.permanents_to_graveyard_this_turn, 2);
    for _ in 0..6 { g.add_card_to_library(1, catalog::island()); }
    let harvest = g.add_card_to_hand(0, catalog::ominous_harvest());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life_before = g.players[1].life;
    let hand_before = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: harvest, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), hand_before + 3, "original + 2 copies each draw 1");
    assert_eq!(g.players[1].life, life_before - 3, "original + 2 copies each lose 1");
}

#[test]
fn knight_exemplar_buffs_and_protects_other_knights() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::knight_exemplar());
    let pal = g.add_card_to_battlefield(0, catalog::silverblade_paladin()); // 2/2 Human Knight
    let c = g.computed_permanent(pal).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "other Knight gets +1/+1");
    assert!(g.computed_permanent(pal).unwrap().keywords.contains(&Keyword::Indestructible),
        "other Knight gains indestructible");
}

#[test]
fn archangel_of_thune_counters_each_creature_on_lifegain() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archangel_of_thune());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Gain life via Faithful Mending mode 2 (Draw 2 + GainLife 2).
    let mending = g.add_card_to_hand(0, catalog::faithful_mending());
    for _ in 0..5 { g.add_card_to_library(0, catalog::island()); }
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mending, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Faithful Mending castable");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "lifegain puts a +1/+1 counter on each creature you control");
}

#[test]
fn wingmate_roc_raid_makes_token_and_gains_life_on_attack() {
    let mut g = two_player_game();
    // Attack first so Raid is on.
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    // Now resolve the Roc's ETB with Raid satisfied → makes a Bird token.
    g.step = TurnStep::PostCombatMain;
    let roc = g.add_card_to_hand(0, catalog::wingmate_roc());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    cast(&mut g, roc);
    drain_stack(&mut g);
    let birds = g.battlefield.iter().filter(|c| c.definition.name == "Bird").count();
    assert_eq!(birds, 1, "Raid creates a Bird token");
}

#[test]
fn boros_elite_battalion_pumps_with_two_other_attackers() {
    let mut g = two_player_game();
    let elite = g.add_card_to_battlefield(0, catalog::boros_elite());
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for id in [elite, a, b] { g.clear_sickness(id); }
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: elite, target: AttackTarget::Player(1) },
        Attack { attacker: a, target: AttackTarget::Player(1) },
        Attack { attacker: b, target: AttackTarget::Player(1) },
    ])).expect("attack with three");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(elite).unwrap().power(), 3, "Battalion pumps +2/+2 → 3 power");
}

#[test]
fn boros_elite_battalion_silent_attacking_alone() {
    let mut g = two_player_game();
    let elite = g.add_card_to_battlefield(0, catalog::boros_elite());
    g.clear_sickness(elite);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: elite, target: AttackTarget::Player(1),
    }])).expect("attack alone");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(elite).unwrap().power(), 1, "no Battalion when attacking alone");
}

#[test]
fn brimaz_makes_attacking_cat_token() {
    let mut g = two_player_game();
    let brimaz = g.add_card_to_battlefield(0, catalog::brimaz_king_of_oreskos());
    g.clear_sickness(brimaz);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: brimaz, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    let cat = g.battlefield.iter().find(|c| c.definition.name == "Cat Soldier").expect("cat token");
    assert!(g.attacking.iter().any(|a| a.attacker == cat.id), "Cat token enters attacking");
}

#[test]
fn adeline_power_scales_with_creatures() {
    let mut g = two_player_game();
    let adeline = g.add_card_to_battlefield(0, catalog::adeline_resplendent_cathar());
    // Adeline alone counts herself → power 1.
    assert_eq!(g.computed_permanent(adeline).unwrap().power, 1, "power = 1 creature (herself)");
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(adeline).unwrap().power, 3, "power = 3 creatures you control");
}

#[test]
fn pia_and_kiran_make_two_thopters_and_sac_for_damage() {
    let mut g = two_player_game();
    let pk = g.add_card_to_hand(0, catalog::pia_and_kiran_nalaar());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    cast(&mut g, pk);
    drain_stack(&mut g);
    let thopters: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Thopter").map(|c| c.id).collect();
    assert_eq!(thopters.len(), 2, "ETB makes two Thopters");
    // Sacrifice one Thopter to deal 2 to the opponent.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pk, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate sac-for-damage");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "sac an artifact deals 2 to any target");
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count(), 1,
        "one Thopter sacrificed");
}

#[test]
fn zo_zu_punishes_land_drops() {
    let mut g = two_player_game();
    // Zo-Zu under P1; P0 (active) plays a land → P0 is punished as the
    // land's controller.
    g.add_card_to_battlefield(1, catalog::zo_zu_the_punisher());
    let land = g.add_card_to_hand(0, catalog::forest());
    let life = g.players[0].life;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "Zo-Zu deals 2 to the land's controller");
}

#[test]
fn midnight_reaper_draws_and_pings_on_nontoken_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::midnight_reaper());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    // Bolt our own bear so the damage→death path dispatches CreatureDied.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[0].life;
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "draw a card on nontoken death");
    assert_eq!(g.players[0].life, life - 1, "1 damage to you on nontoken death");
}

#[test]
fn midnight_reaper_silent_on_token_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::midnight_reaper());
    // A token creature dying should not trigger Midnight Reaper.
    let mut tok = catalog::grizzly_bears();
    tok.name = "Bear Token";
    let t = g.add_card_to_battlefield(0, tok);
    g.battlefield_find_mut(t).unwrap().is_token = true;
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[0].life;
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(t)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt token");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no ping on token death");
    assert_eq!(g.players[0].library.len(), lib, "no draw on token death");
}

#[test]
fn grim_haruspex_draws_on_other_nontoken_death() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let grim = g.add_card_to_battlefield(0, catalog::grim_haruspex());
    assert!(g.battlefield_find(grim).unwrap().definition.keywords
        .iter().any(|k| matches!(k, Keyword::Morph(_))), "has Morph");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own bear");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "draw when another nontoken creature dies");
}

#[test]
fn avatar_of_the_resolute_enters_with_counters_per_counter_creature() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Two other creatures already carry a +1/+1 counter.
    for _ in 0..2 {
        let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.battlefield_find_mut(c).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    }
    let avatar = g.add_card_to_hand(0, catalog::avatar_of_the_resolute());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 0;
    cast(&mut g, avatar);
    drain_stack(&mut g);
    let a = g.battlefield.iter().find(|c| c.definition.name == "Avatar of the Resolute").unwrap();
    assert_eq!(a.counter_count(CounterType::PlusOnePlusOne), 2,
        "enters with one counter per other counter-bearing creature");
}

#[test]
fn pelt_collector_grows_and_gains_trample() {
    use crabomination::card::{CounterType, Keyword};
    let mut g = two_player_game();
    let pelt = g.add_card_to_battlefield(0, catalog::pelt_collector());
    // A bigger creature entering bumps Pelt Collector once.
    let big = g.add_card_to_hand(0, catalog::grizzly_bears()); // 2/2 > 1/1
    g.players[0].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 0;
    cast(&mut g, big);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(pelt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "bigger creature entering grows Pelt Collector");
    // Force it to 3 counters → trample.
    g.battlefield_find_mut(pelt).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    assert!(g.computed_permanent(pelt).unwrap().keywords.contains(&Keyword::Trample),
        "trample at 3+ counters");
}

#[test]
fn glen_elendra_counters_noncreature_spell() {
    let mut g = two_player_game();
    let glen = g.add_card_to_battlefield(0, catalog::glen_elendra_archmage());
    // Opponent casts a noncreature spell; Glen Elendra sacs to counter it.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp bolt");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: glen, ability_index: 0, target: Some(Target::Permanent(bolt)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac to counter");
    drain_stack(&mut g);
    // Persist returns Glen Elendra with a -1/-1 counter.
    let glen_back = g.battlefield.iter().find(|c| c.definition.name == "Glen Elendra Archmage");
    assert!(glen_back.is_some(), "Persist returns Glen Elendra");
    assert_eq!(g.players[0].life, 20, "the bolt was countered (no damage)");
}

#[test]
fn persist_returns_creature_destroyed_by_removal() {
    use crabomination::card::CounterType;
    // CR 702.79 — Persist applies on death by destruction, not just lethal
    // combat damage. Murder a Persist creature → it returns with a -1/-1.
    let mut g = two_player_game();
    let glen = g.add_card_to_battlefield(0, catalog::glen_elendra_archmage());
    let murder = g.add_card_to_hand(1, catalog::murder());
    g.players[1].mana_pool.add(Color::Black, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: murder, target: Some(Target::Permanent(glen)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("murder Glen Elendra");
    drain_stack(&mut g);
    let back = g.battlefield.iter().find(|c| c.definition.name == "Glen Elendra Archmage")
        .expect("Persist returns the creature after destroy");
    assert_eq!(back.counter_count(CounterType::MinusOneMinusOne), 1,
        "returns with a -1/-1 counter");
}

#[test]
fn inventors_apprentice_pumps_with_an_artifact() {
    let mut g = two_player_game();
    let app = g.add_card_to_battlefield(0, catalog::inventors_apprentice());
    assert_eq!(g.computed_permanent(app).unwrap().power, 1, "1/2 with no artifact");
    g.add_card_to_battlefield(0, catalog::sol_ring()); // an artifact
    let cp = g.computed_permanent(app).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+1 while you control an artifact");
}

#[test]
fn signal_pest_blocked_only_by_flying_or_reach() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let pest = g.add_card_to_battlefield(0, catalog::signal_pest());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // no evasion-blocker kw
    let flyer = g.add_card_to_battlefield(1, catalog::wind_drake()); // flying
    assert!(!g.blocker_can_block_attacker(ground, pest), "ground creature can't block Signal Pest");
    assert!(g.blocker_can_block_attacker(flyer, pest), "flyer can block Signal Pest");
    let _ = Keyword::Flying;
}

#[test]
fn kitesail_freebooter_exiles_noncreature_until_it_leaves() {
    let mut g = two_player_game();
    // Opponent holds a noncreature spell to be exiled.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let kite = g.add_card_to_hand(0, catalog::kitesail_freebooter());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    cast(&mut g, kite);
    drain_stack(&mut g);
    assert!(!g.players[1].hand.iter().any(|c| c.id == bolt), "noncreature card exiled from hand");
    // When Kitesail leaves, the card returns.
    let kite_id = g.battlefield.iter().find(|c| c.definition.name == "Kitesail Freebooter").unwrap().id;
    g.remove_from_battlefield_to_graveyard_raw(kite_id);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bolt), "card returns when Kitesail leaves");
}

#[test]
fn bygone_bishop_investigates_on_small_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bygone_bishop());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // MV 2 creature
    g.players[0].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 0;
    cast(&mut g, bear);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Clue"),
        "investigate makes a Clue on a small creature cast");
}

#[test]
fn sram_draws_on_equipment_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sram_senior_edificer());
    g.add_card_to_library(0, catalog::island());
    // Bonesplitter is an Equipment.
    let equip = g.add_card_to_hand(0, catalog::bonesplitter());
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    let lib = g.players[0].library.len();
    cast(&mut g, equip);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "Sram draws when you cast an Equipment");
}

#[test]
fn narnam_renegade_revolt_enters_bigger_after_a_permanent_left() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // No revolt yet → enters as a plain 1/2.
    let plain = g.add_card_to_hand(0, catalog::narnam_renegade());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 0;
    cast(&mut g, plain);
    drain_stack(&mut g);
    let n1 = g.battlefield.iter().find(|c| c.definition.name == "Narnam Renegade").unwrap();
    assert_eq!(n1.counter_count(CounterType::PlusOnePlusOne), 0, "no Revolt → no counter");
    // Sacrifice a permanent → Revolt active for the rest of the turn.
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(fodder);
    assert!(g.players[0].permanent_left_battlefield_this_turn);
    let revolt = g.add_card_to_hand(0, catalog::narnam_renegade());
    g.players[0].mana_pool.add(Color::Green, 1);
    cast(&mut g, revolt);
    drain_stack(&mut g);
    let n2 = g.battlefield.iter().filter(|c| c.definition.name == "Narnam Renegade")
        .max_by_key(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap();
    assert_eq!(n2.counter_count(CounterType::PlusOnePlusOne), 1, "Revolt → +1/+1 counter");
}

#[test]
fn hidden_herbalists_revolt_adds_green_mana() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(fodder); // Revolt on
    let herb = g.add_card_to_hand(0, catalog::hidden_herbalists());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    cast(&mut g, herb);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "Revolt adds two green mana");
}

#[test]
fn fanatic_of_mogis_deals_devotion_to_each_opponent() {
    let mut g = two_player_game();
    // A {R}{R} red permanent → devotion 2, plus the Fanatic's own {R} = 3.
    g.add_card_to_battlefield(0, catalog::pia_and_kiran_nalaar());
    let fan = g.add_card_to_hand(0, catalog::fanatic_of_mogis());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    let life = g.players[1].life;
    cast(&mut g, fan);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "Fanatic deals devotion-to-red (3) to the opponent");
}

#[test]
fn ridgescale_tusker_counters_each_other_creature() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tusker = g.add_card_to_hand(0, catalog::ridgescale_tusker());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    cast(&mut g, tusker);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "other creature gets a +1/+1 counter");
    let t = g.battlefield.iter().find(|c| c.definition.name == "Ridgescale Tusker").unwrap();
    assert_eq!(t.counter_count(CounterType::PlusOnePlusOne), 0, "Tusker itself excluded");
}

#[test]
fn solemn_recruit_grows_on_revolt_end_step() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let rec = g.add_card_to_battlefield(0, catalog::solemn_recruit());
    g.active_player_idx = 0;
    // No revolt yet → no growth.
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rec).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
    // Trigger Revolt, then the end step grows it.
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_from_battlefield_to_graveyard_raw(fodder);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rec).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Revolt end step adds a +1/+1 counter");
}

#[test]
fn manic_vandal_destroys_an_artifact_on_etb() {
    let mut g = two_player_game();
    let art = g.add_card_to_battlefield(1, catalog::sol_ring());
    let van = g.add_card_to_hand(0, catalog::manic_vandal());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    cast(&mut g, van);
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "ETB destroys the artifact");
}

#[test]
fn fairgrounds_warden_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let warden = g.add_card_to_hand(0, catalog::fairgrounds_warden());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    cast(&mut g, warden);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim exiled");
    let wid = g.battlefield.iter().find(|c| c.definition.name == "Fairgrounds Warden").unwrap().id;
    g.remove_from_battlefield_to_graveyard_raw(wid);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "victim returns when Warden leaves");
}

#[test]
fn goblin_cratermaker_pings_a_creature() {
    let mut g = two_player_game();
    let crater = g.add_card_to_battlefield(0, catalog::goblin_cratermaker());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crater, ability_index: 0, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac for mode 1 (2 damage)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "2 damage kills the 2/2");
    assert!(g.battlefield_find(crater).is_none(), "Cratermaker sacrificed itself");
}

#[test]
fn green_beaters_have_their_printed_keywords() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let crusher = g.add_card_to_battlefield(0, catalog::plated_crusher());
    let c = g.battlefield_find(crusher).unwrap();
    assert!(c.has_keyword(&Keyword::Trample) && c.has_keyword(&Keyword::Hexproof));
    assert_eq!((c.power(), c.toughness()), (7, 6));
    let stomper = g.add_card_to_battlefield(0, catalog::terra_stomper());
    assert!(g.battlefield_find(stomper).unwrap().has_keyword(&Keyword::CantBeCountered));
    let wurm = g.add_card_to_battlefield(0, catalog::bellowing_tanglewurm());
    assert!(g.battlefield_find(wurm).unwrap().has_keyword(&Keyword::Intimidate));
}

#[test]
fn vinelasher_kudzu_grows_on_landfall() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let kudzu = g.add_card_to_battlefield(0, catalog::vinelasher_kudzu());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.active_player_idx = 0;
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(kudzu).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "landfall puts a +1/+1 counter on Vinelasher Kudzu");
}

#[test]
fn spikeshot_elder_pings_for_its_power() {
    let mut g = two_player_game();
    let elder = g.add_card_to_battlefield(0, catalog::spikeshot_elder());
    g.battlefield_find_mut(elder).unwrap().power_bonus = 2; // 1 + 2 = 3 power
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elder, ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("ping any target");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3, "deals damage equal to its power");
}

#[test]
fn tormented_soul_is_unblockable_and_cant_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let soul = g.add_card_to_battlefield(0, catalog::tormented_soul());
    let c = g.battlefield_find(soul).unwrap();
    assert!(c.has_keyword(&Keyword::Unblockable) && c.has_keyword(&Keyword::CantBlock));
}

#[test]
fn bloodcrazed_neonate_grows_on_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let neo = g.add_card_to_battlefield(0, catalog::bloodcrazed_neonate());
    g.clear_sickness(neo);
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: neo, target: AttackTarget::Player(1),
    }])).expect("attack");
    g.step = TurnStep::CombatDamage;
    g.resolve_combat().unwrap();
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(neo).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "combat damage to a player grows the Neonate");
}

#[test]
fn stormblood_berserker_bloodthirst_enters_bigger() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    // Damage the opponent so Bloodthirst is active.
    g.players[1].was_dealt_damage_this_turn = true;
    let berserker = g.add_card_to_hand(0, catalog::stormblood_berserker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    cast(&mut g, berserker);
    drain_stack(&mut g);
    let b = g.battlefield.iter().find(|c| c.definition.name == "Stormblood Berserker").unwrap();
    assert_eq!(b.counter_count(CounterType::PlusOnePlusOne), 2, "Bloodthirst 2 → two counters");
}

/// Silverblade Paladin's Soulbond grants double strike to both members.
#[test]
fn soulbond_silverblade_paladin_grants_double_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pal = g.add_card_to_hand(0, catalog::silverblade_paladin());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, pal);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::DoubleStrike));
    assert!(g.computed_permanent(pal).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Nearheath Pilgrim's Soulbond grants lifelink to both members.
#[test]
fn soulbond_nearheath_pilgrim_grants_lifelink() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pilgrim = g.add_card_to_hand(0, catalog::nearheath_pilgrim());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, pilgrim);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Weaponcraft Enthusiast's Fabricate 2 takes the counter mode by default
/// (AutoDecider), entering as a 2/3.
#[test]
fn fabricate_weaponcraft_enthusiast_counter_mode() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::weaponcraft_enthusiast());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, id);
    let c = g.computed_permanent(id).unwrap();
    assert_eq!((c.power, c.toughness), (2, 3), "0/1 + two +1/+1 counters");
}

/// Tandem Lookout's Soulbond grants "deals combat damage → draw" to its partner.
#[test]
fn soulbond_tandem_lookout_partner_draws_on_combat_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let lookout = g.add_card_to_battlefield(0, catalog::tandem_lookout());
    // Pair them directly (ETB pairing is exercised by the other Soulbond tests).
    g.apply_soulbond_pairing(lookout);
    assert_eq!(g.battlefield_find(bear).unwrap().soulbond_partner, Some(lookout));
    g.add_card_to_library(0, catalog::grizzly_bears()); // a card to draw
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..12 {
        if g.players[0].hand.len() > hand_before { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand_before + 1,
        "the unblocked partner draws on combat damage");
}

/// Mardu Hateblade's `{B}: deathtouch until EOT` grants deathtouch.
#[test]
fn mardu_hateblade_grants_deathtouch() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mardu_hateblade());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("grant deathtouch");
    drain_stack(&mut g);
    assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Deathtouch));
}

/// War Falcon can't attack with no Knight/Soldier, but can once one is present.
#[test]
fn war_falcon_attack_gated_on_knight_or_soldier() {
    let mut g = two_player_game();
    let falcon = g.add_card_to_battlefield(0, catalog::war_falcon());
    g.clear_sickness(falcon);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(g.declare_attackers(vec![Attack { attacker: falcon, target: AttackTarget::Player(1) }])
        .is_err(), "no Soldier/Knight → can't attack");
    // A Soldier on the board satisfies the requirement.
    let sol = g.add_card_to_battlefield(0, catalog::mardu_hateblade()); // Human Warrior… not soldier
    let _ = sol;
    let knight = g.add_card_to_battlefield(0, catalog::silverblade_paladin()); // Human Knight
    g.clear_sickness(knight);
    assert!(g.declare_attackers(vec![Attack { attacker: falcon, target: AttackTarget::Player(1) }])
        .is_ok(), "Knight present → War Falcon may attack");
}

/// Stromkirk Patrol grows when it connects.
#[test]
fn stromkirk_patrol_grows_on_combat_damage() {
    let mut g = two_player_game();
    let pat = g.add_card_to_battlefield(0, catalog::stromkirk_patrol());
    g.clear_sickness(pat);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pat, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..12 {
        if g.battlefield_find(pat).map(|c| c.counter_count(CounterType::PlusOnePlusOne)).unwrap_or(0) > 0 { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(pat).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Boros Recruit is a {R/W} 1/1 with first strike, castable with either pip.
#[test]
fn boros_recruit_first_strike_hybrid() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boros_recruit());
    g.players[0].mana_pool.add(Color::Red, 1); // pay the {R/W} with red
    cast(&mut g, id);
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::FirstStrike));
}

/// A hybrid ({R/W}) card counts as BOTH its colors for hidden-zone color
/// filters (CR 105.2c), matching the battlefield `colors_from_card` path.
/// Regression: `R::HasColor` previously scanned only plain `Colored` pips and
/// read Boros Recruit as colorless in hand/graveyard/library.
#[test]
fn hybrid_card_matches_both_colors_in_hidden_zone() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::boros_recruit()); // {R/W}
    let card = g.players[0].hand.iter().find(|c| c.id == id).unwrap().clone();
    assert!(g.evaluate_requirement_on_card(&R::HasColor(Color::Red), &card, 0),
        "{{R/W}} hybrid card is red");
    assert!(g.evaluate_requirement_on_card(&R::HasColor(Color::White), &card, 0),
        "{{R/W}} hybrid card is white");
    assert!(!g.evaluate_requirement_on_card(&R::HasColor(Color::Blue), &card, 0),
        "{{R/W}} hybrid card is not blue");
}

/// Bloodrock Cyclops must attack when able.
#[test]
fn bloodrock_cyclops_must_attack() {
    let mut g = two_player_game();
    let cy = g.add_card_to_battlefield(0, catalog::bloodrock_cyclops());
    g.clear_sickness(cy);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // Declaring no attackers is illegal while a MustAttack creature can attack.
    assert!(g.declare_attackers(vec![]).is_err(),
        "Bloodrock Cyclops must be declared as an attacker");
}

// ── Dethrone (CR 702.105) primitive ──────────────────────────────────────────

/// Dethrone (via `shortcut::dethrone()`, granted for the test) grows a creature
/// only when it attacks the player with the most life.
#[test]
fn dethrone_grows_when_attacking_highest_life_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.granted_triggers_eot.insert(bear, vec![crabomination::effect::shortcut::dethrone()]);
    g.players[1].life = 30; // the defender has the most life
    g.players[0].life = 20;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "Dethrone fires against the highest-life player");
}

/// Dethrone does nothing when the attacked player is not the highest-life one.
#[test]
fn dethrone_silent_when_attacking_lower_life_player() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    g.granted_triggers_eot.insert(bear, vec![crabomination::effect::shortcut::dethrone()]);
    g.players[1].life = 10; // the defender has the LEAST life
    g.players[0].life = 30;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "Dethrone is silent against a non-highest-life player");
}

