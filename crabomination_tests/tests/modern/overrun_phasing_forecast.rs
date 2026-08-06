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

// ── Team-pump finishers (Overrun variants) ──────────────────────────────────

/// Overwhelming Stampede: creatures you control get +X/+X (X = greatest power
/// you control) and gain trample. X is evaluated once, so every creature gets
/// the same bonus.
#[test]
fn overwhelming_stampede_pumps_by_greatest_power_and_grants_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); // 6/6
    let spell = g.add_card_to_hand(0, catalog::overwhelming_stampede());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Overwhelming Stampede for {3}{G}{G}");
    drain_stack(&mut g);
    // X = 6 (greatest power among your creatures), added to each.
    assert_eq!(g.battlefield_find(small).unwrap().power(), 8, "2 + 6");
    assert_eq!(g.battlefield_find(big).unwrap().power(), 12, "6 + 6");
    assert!(g.computed_permanent(small).unwrap().keywords.contains(&Keyword::Trample),
        "the bear gains trample");
}

/// Triumph of the Hordes: +1/+1, trample, and infect to your team.
#[test]
fn triumph_of_the_hordes_grants_infect_and_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::triumph_of_the_hordes());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Triumph of the Hordes for {2}{G}{G}");
    drain_stack(&mut g);
    let s = g.battlefield_find(bear).unwrap();
    assert_eq!((s.power(), s.toughness()), (3, 3), "+1/+1");
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample), "gains trample");
    assert!(cp.keywords.contains(&Keyword::Infect), "gains infect");
}

/// Aura Shards destroys an artifact/enchantment whenever a creature you control
/// enters.
#[test]
fn aura_shards_destroys_artifact_on_creature_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aura_shards());
    let stone = g.add_card_to_battlefield(1, catalog::mind_stone()); // opponent's artifact
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature under Aura Shards");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == stone),
        "creature ETB triggers Aura Shards to destroy the opponent's artifact");
}

/// Avenger of Zendikar makes a Plant per land on ETB, then landfall pumps every
/// Plant with a +1/+1 counter.
#[test]
fn avenger_of_zendikar_makes_plants_then_landfall_pumps() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); } // 3 lands
    let av = g.add_card_to_hand(0, catalog::avenger_of_zendikar());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpell {
        card_id: av, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Avenger of Zendikar for {5}{G}{G}");
    drain_stack(&mut g);
    let plants: Vec<_> = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Plant")
        .map(|c| c.id)
        .collect();
    assert_eq!(plants.len(), 3, "one 0/1 Plant per land you control");
    // Landfall: playing a land puts a +1/+1 counter on each Plant.
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land for landfall");
    drain_stack(&mut g);
    for pid in plants {
        let p = g.battlefield_find(pid).unwrap();
        assert_eq!((p.power(), p.toughness()), (1, 2), "landfall pumped the Plant to 1/2");
    }
}

/// Beastmaster Ascension's anthem switches on at seven quest counters.
#[test]
fn beastmaster_ascension_anthem_at_seven_quest_counters() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let asc = g.add_card_to_battlefield(0, catalog::beastmaster_ascension());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Quest, 6);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "no anthem below seven quest counters");
    g.battlefield_find_mut(asc).unwrap().add_counters(CounterType::Quest, 1); // → 7
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (7, 7), "+5/+5 once at seven quest counters");
}

/// Seedborn Muse untaps your permanents during another player's untap step.
#[test]
fn seedborn_muse_untaps_on_opponents_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::seedborn_muse());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    // Simulate the opponent's untap step.
    g.active_player_idx = 1;
    g.do_untap();
    assert!(!g.battlefield_find(land).unwrap().tapped,
        "Seedborn untaps your permanents on each other player's untap step");
}

/// Without Seedborn Muse, a non-active player's permanents stay tapped.
#[test]
fn without_seedborn_permanents_stay_tapped_on_opponents_turn() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(land).unwrap().tapped = true;
    g.active_player_idx = 1;
    g.do_untap();
    assert!(g.battlefield_find(land).unwrap().tapped,
        "normally a non-active player's permanents don't untap");
}

/// Birthing Pod sacrifices a creature and tutors a creature with mana value one
/// higher straight to the battlefield.
#[test]
fn birthing_pod_sacrifices_then_tutors_one_higher() {
    let mut g = two_player_game();
    let pod = g.add_card_to_battlefield(0, catalog::birthing_pod());
    g.clear_sickness(pod);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // MV 2
    let centaur = g.add_card_to_library(0, catalog::centaur_courser()); // MV 3
    g.add_card_to_library(0, catalog::shock()); // padding (not a creature)
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(centaur))]));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: pod, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Birthing Pod for {1}{G}, {T}");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == bear),
        "the sacrificed MV-2 creature is in the graveyard");
    assert!(g.battlefield.iter().any(|c| c.id == centaur),
        "an MV-3 creature (sacrificed + 1) is put onto the battlefield");
}

/// The Great Henge's cast cost drops by the greatest power among your creatures.
#[test]
fn the_great_henge_cost_reduced_by_greatest_power() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::colossal_dreadmaw()); // power 6
    let henge = g.add_card_to_hand(0, catalog::the_great_henge());
    // {7}{G}{G} − 6 = {1}{G}{G}: three mana is enough.
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: henge, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Henge costs {1}{G}{G} with a 6-power creature out");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.id == henge), "Henge resolved");
}

/// Leyline Binding's cast cost drops by {1} per basic land type you control
/// (Domain, CR 702.43), and its ETB exiles an opponent's nonland permanent
/// until it leaves.
#[test]
fn leyline_binding_domain_reduction_and_exile() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    // Three distinct basic land types → {3} off {5}{W} = {2}{W}.
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::mountain());
    let spell = crabomination::card::CardInstance::new(g.next_id(), catalog::leyline_binding(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 3, "domain = 3");
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::leyline_binding());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Leyline Binding costs {2}{W} with domain 3");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim exiled");
    // When the Binding leaves, the exiled permanent returns.
    g.remove_from_battlefield_to_graveyard_raw(id);
    assert!(g.battlefield.iter().any(|c| c.id == victim), "victim returns when Binding leaves");
}

/// Tribal Flames deals damage equal to your domain.
#[test]
fn tribal_flames_deals_domain_damage() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::swamp());
    let id = g.add_card_to_hand(0, catalog::tribal_flames());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life_before = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Tribal Flames");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "domain 2 → 2 damage");
}

/// The Great Henge taps for {G} and 2 life, and on a nontoken creature ETB it
/// draws a card and puts a +1/+1 counter on that creature.
#[test]
fn the_great_henge_mana_life_and_etb_payoff() {
    let mut g = two_player_game();
    let henge = g.add_card_to_battlefield(0, catalog::the_great_henge());
    g.clear_sickness(henge);
    // {T}: add {G}, gain 2 life.
    let life_before = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: henge, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap Henge for mana + life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1, "added {{G}}");
    assert_eq!(g.players[0].life, life_before + 2, "gained 2 life");
    // Nontoken creature ETB → draw + counter.
    g.add_card_to_library(0, catalog::shock()); // something to draw
    let lib_before = g.players[0].library.len();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1); // {G} already floating from the tap
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a nontoken creature");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (3, 3), "nontoken creature got a +1/+1 counter");
    assert_eq!(g.players[0].library.len(), lib_before - 1, "Henge drew a card");
}

/// Mana Vault taps for {C}{C}{C}, doesn't untap, offers the upkeep {4}
/// untap, and burns 1 at the draw step while tapped.
#[test]
fn mana_vault_taps_for_three_and_burns_at_draw_step() {
    let mut g = two_player_game();
    let mv = g.add_card_to_battlefield(0, catalog::mana_vault());
    g.clear_sickness(mv);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mv, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap Mana Vault");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 3, "added {{C}}{{C}}{{C}}");
    g.players[0].mana_pool = Default::default();
    g.do_untap();
    assert!(g.battlefield_find(mv).unwrap().tapped, "Mana Vault doesn't untap");
    let life = g.players[0].life;
    g.fire_step_triggers(crabomination::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life, "no upkeep burn (that's the draw step)");
    g.fire_step_triggers(crabomination::TurnStep::Draw);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "1 damage at the draw step while tapped");
}

/// Mana Vault's upkeep trigger pays {4} to untap when mana is up.
#[test]
fn mana_vault_upkeep_may_pay_four_to_untap() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    let mv = g.add_card_to_battlefield(0, catalog::mana_vault());
    g.battlefield_find_mut(mv).unwrap().tapped = true;
    g.players[0].mana_pool.add_colorless(4);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(crabomination::TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mv).unwrap().tapped, "paid {{4}}, untapped");
    assert_eq!(g.players[0].mana_pool.total(), 0, "the {{4}} was spent");
}

/// Chromatic Lantern grants your lands "{T}: Add one mana of any color".
#[test]
fn chromatic_lantern_lets_lands_tap_for_any_color() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::chromatic_lantern());
    let forest = g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(forest);
    // The granted "{T}: add any color" is a virtual ability at index 1 (after
    // the Forest's printed "{T}: add {G}").
    g.perform_action(GameAction::ActivateAbility {
        card_id: forest, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Forest taps via Chromatic Lantern's granted ability");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "Forest produced one mana");
}

/// Dramatic Reversal untaps all nonland permanents you control (but not lands).
#[test]
fn dramatic_reversal_untaps_nonlands_only() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(0, catalog::sol_ring());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.battlefield_find_mut(rock).unwrap().tapped = true;
    g.battlefield_find_mut(land).unwrap().tapped = true;
    let rev = g.add_card_to_hand(0, catalog::dramatic_reversal());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rev, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Dramatic Reversal");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(rock).unwrap().tapped, "Sol Ring untapped");
    assert!(g.battlefield_find(land).unwrap().tapped, "land NOT untapped");
}

/// Birgi adds {R} whenever you cast a spell.
#[test]
fn birgi_adds_red_on_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::birgi_god_of_storytelling());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "Birgi refunded {{R}} on cast");
}

/// Birgi // Harnfel — the DFC back face Harnfel is cast from hand (resolving
/// as an artifact); discarding a card then exiles the top two of your library.
#[test]
fn birgi_back_harnfel_castable_and_exiles_top_two_on_discard() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::birgi_god_of_storytelling());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::CastSpellBack {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Harnfel (back face) castable for {5}{R}");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.id == id && c.definition.name == "Harnfel, Horn of Bounty"),
        "Harnfel resolves onto the battlefield as an artifact",
    );

    // Seed the library; top two are `a`, `b` (add_card_to_library appends to bottom).
    let a = g.add_card_to_library(0, catalog::lightning_bolt());
    let b = g.add_card_to_library(0, catalog::grizzly_bears());
    let _c = g.add_card_to_library(0, catalog::forest());
    let lib_before = g.players[0].library.len();
    let exile_before = g.exile.len();

    // Discard a card → Harnfel triggers.
    let throwaway = g.add_card_to_hand(0, catalog::forest());
    let card = g.players[0].remove_from_hand(throwaway).unwrap();
    g.players[0].graveyard.push(card);
    g.dispatch_triggers_for_events(&[GameEvent::CardDiscarded { player: 0, card_id: throwaway }]);
    drain_stack(&mut g);

    assert_eq!(g.players[0].library.len(), lib_before - 2, "the top two cards left the library");
    assert_eq!(g.exile.len(), exile_before + 2, "two cards were exiled");
    assert!(
        g.exile.iter().any(|c| c.id == a) && g.exile.iter().any(|c| c.id == b),
        "the exiled cards are the top two",
    );
}

/// Field of Ruin taps for {C} and can sacrifice to destroy a nonbasic land.
#[test]
fn field_of_ruin_destroys_a_nonbasic_land() {
    let mut g = two_player_game();
    let field = g.add_card_to_battlefield(0, catalog::field_of_ruin());
    g.clear_sickness(field);
    let target = g.add_card_to_battlefield(1, catalog::reliquary_tower()); // nonbasic land
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: field, ability_index: 1, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Field of Ruin's destroy ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "nonbasic land destroyed");
}

/// Aura of Silence taxes artifact/enchantment spells and can sac to destroy one.
#[test]
fn aura_of_silence_taxes_and_destroys() {
    let mut g = two_player_game();
    let aura = g.add_card_to_battlefield(0, catalog::aura_of_silence());
    let relic = g.add_card_to_battlefield(1, catalog::mind_stone());
    // Sac to destroy the opponent's artifact.
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: Some(Target::Permanent(relic)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac Aura of Silence to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(relic).is_none(), "artifact destroyed");
}

/// Return to Dust exiles a target artifact or enchantment.
#[test]
fn return_to_dust_exiles_artifact() {
    let mut g = two_player_game();
    let relic = g.add_card_to_battlefield(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::return_to_dust());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(relic)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Return to Dust");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == relic), "artifact exiled");
}

/// Aetherflux Reservoir gains life per spell cast and can pay 50 life to deal 50.
#[test]
fn aetherflux_reservoir_gains_life_then_burns() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aetherflux_reservoir());
    let life0 = g.players[0].life;
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a spell");
    drain_stack(&mut g);
    // First (and only) spell this turn → gain 1 life.
    assert_eq!(g.players[0].life, life0 + 1, "gained 1 life for the first spell");
    // Pay 50 life: deal 50 to the opponent.
    g.players[0].life = 60;
    g.perform_action(GameAction::ActivateAbility {
        card_id: g.battlefield.iter().find(|c| c.definition.name == "Aetherflux Reservoir").unwrap().id,
        ability_index: 0, target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("pay 50 life");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 10, "paid 50 life");
    assert!(g.players[1].life <= 0, "opponent took 50 (lethal)");
}

/// Jeska's Will mode 0 adds {R} per card in TARGET opponent's hand
/// (audit fix: was a flat {R}{R}{R}).
#[test]
fn jeskas_will_ritual_mode() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let will = g.add_card_to_hand(0, catalog::jeskas_will());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: will, target: Some(Target::Player(1)), additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Jeska's Will, ritual mode");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 3,
        "added one {{R}} per card in the opponent's 3-card hand");
}

/// Helper: opponent (seat 1) casts a Lightning Bolt at seat 0, then seat 0
/// casts `counter` (already in hand, mana provided) targeting it. Returns the
/// bolt's id so the caller can assert it was countered.
fn cast_then_counter(g: &mut GameState, counter: crabomination::card::CardId, bolt: crabomination::card::CardId) {
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts Bolt");
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(bolt)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast counter");
    drain_stack(g);
}

#[test]
fn mana_drain_counters_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let md = g.add_card_to_hand(0, catalog::mana_drain());
    g.players[0].mana_pool.add(Color::Blue, 2);
    cast_then_counter(&mut g, md, bolt);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered");
    assert_eq!(g.players[0].life, 20, "no damage taken");
}

#[test]
fn fierce_guardianship_counters_noncreature() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let fg = g.add_card_to_hand(0, catalog::fierce_guardianship());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_then_counter(&mut g, fg, bolt);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered");
}

#[test]
fn deflecting_swat_counters_a_spell() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    let ds = g.add_card_to_hand(0, catalog::deflecting_swat());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast_then_counter(&mut g, ds, bolt);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered");
}

/// Opponent (seat 1) casts a Lightning Bolt at seat 0. Returns nothing; the
/// caller asserts the seat-0 payoff.
fn opponent_casts_bolt(g: &mut GameState) {
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts Bolt");
    drain_stack(g);
}

#[test]
fn rhystic_study_draws_when_opponent_casts() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rhystic_study());
    g.add_card_to_library(0, catalog::shock());
    let lib = g.players[0].library.len();
    opponent_casts_bolt(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "Rhystic Study drew a card");
}

/// When the opponent pays the {1} tax, Rhystic Study draws nothing.
#[test]
fn rhystic_study_no_draw_when_opponent_pays() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rhystic_study());
    g.add_card_to_library(0, catalog::shock());
    let lib = g.players[0].library.len();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // opponent pays
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1); // spare {1} to pay the tax
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a spell");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib, "opponent paid {{1}} → no draw");
}

#[test]
fn mystic_remora_draws_on_opponent_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mystic_remora());
    g.add_card_to_library(0, catalog::shock());
    let lib = g.players[0].library.len();
    opponent_casts_bolt(&mut g); // Bolt is noncreature
    assert_eq!(g.players[0].library.len(), lib - 1, "Mystic Remora drew a card");
}

#[test]
fn esper_sentinel_draws_on_opponent_noncreature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::esper_sentinel());
    g.add_card_to_library(0, catalog::shock());
    let lib = g.players[0].library.len();
    opponent_casts_bolt(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "Esper Sentinel drew a card");
}

#[test]
fn smothering_tithe_makes_treasure_on_opponent_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::smothering_tithe());
    g.add_card_to_library(1, catalog::shock());
    let mut evs = vec![];
    g.draw_one(1, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"),
        "Smothering Tithe created a Treasure for you");
}

#[test]
fn mox_diamond_discards_on_etb_and_taps_for_any() {
    let mut g = two_player_game();
    let mox = g.add_card_to_hand(0, catalog::mox_diamond());
    g.add_card_to_hand(0, catalog::forest()); // a card to discard
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::CastSpell {
        card_id: mox, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Mox Diamond for {0}");
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy + 1, "discarded a card on ETB");
    g.clear_sickness(mox);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "Mox Diamond tapped for one mana");
}

#[test]
fn chrome_mox_imprints_and_taps_for_imprinted_color() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    let mox = g.add_card_to_hand(0, catalog::chrome_mox());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // a red card to imprint
    g.perform_action(GameAction::CastSpell {
        card_id: mox, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chrome Mox for {0}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "imprinted card exiled");
    g.clear_sickness(mox);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for the imprinted card's color");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1, "produced red (the imprinted Bolt's color)");
}

/// Chrome Mox with nothing imprinted (empty hand) produces no mana.
#[test]
fn chrome_mox_with_no_imprint_makes_no_mana() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    let mox = g.add_card_to_hand(0, catalog::chrome_mox());
    g.perform_action(GameAction::CastSpell {
        card_id: mox, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Chrome Mox with empty hand");
    drain_stack(&mut g);
    g.clear_sickness(mox);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mox, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap (nothing imprinted)");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 0, "no imprint → no mana");
}

#[test]
fn fact_or_fiction_splits_revealed_into_hand_and_graveyard() {
    let mut g = two_player_game();
    // Three equal-value cards on top: the opponent isolates one (pile A, MV 1),
    // the other two form pile B (MV 2) — you keep the higher-value pile B.
    for _ in 0..3 { g.add_card_to_library(0, catalog::shock()); } // MV 1 each
    let fof = g.add_card_to_hand(0, catalog::fact_or_fiction());
    let gy_before = g.players[0].graveyard.len();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: fof, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fact or Fiction");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), 0, "all three revealed cards left the library");
    assert_eq!(g.players[0].hand.iter().filter(|c| c.definition.name == "Shock").count(), 2,
        "the larger pile (2 cards) went to hand");
    // One Shock to graveyard from the split, plus Fact or Fiction itself.
    assert_eq!(g.players[0].graveyard.len(), gy_before + 2, "one Shock + the spell hit the graveyard");
}

#[test]
fn austere_command_choose_two_destroys_all_creatures_by_default() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // MV 6
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let relic = g.add_card_to_battlefield(1, catalog::mind_stone()); // artifact
    let cmd = g.add_card_to_hand(0, catalog::austere_command());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: cmd, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Austere Command (default picks: both creature modes)");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "MV-6 creature destroyed");
    assert!(g.battlefield_find(small).is_none(), "MV-2 creature destroyed");
    assert!(g.battlefield_find(relic).is_some(), "artifact survives (modes 0/1 not chosen)");
}

#[test]
fn teferis_protection_phases_out_your_permanents_and_prevents_damage() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tp = g.add_card_to_hand(0, catalog::teferis_protection());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: tp, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Teferi's Protection");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "your permanents phase out (gone from battlefield)");
    assert!(g.phased_out.iter().any(|c| c.id == bear), "tracked in phased_out");
    let life = g.players[0].life;
    opponent_casts_bolt(&mut g);
    assert_eq!(g.players[0].life, life, "damage to you is prevented");
    // Phases back in at your next untap step.
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.battlefield_find(bear).is_some(), "phases back in on your untap");
}

#[test]
fn comeuppance_prevents_damage_to_you() {
    let mut g = two_player_game();
    let c = g.add_card_to_hand(0, catalog::comeuppance());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: c, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Comeuppance");
    drain_stack(&mut g);
    let life = g.players[0].life;
    opponent_casts_bolt(&mut g);
    assert_eq!(g.players[0].life, life, "damage to you is prevented");
}

/// Isochron Scepter imprints an instant on ETB, then free-casts it from exile.
#[test]
fn isochron_scepter_imprints_then_free_casts_the_instant() {
    let mut g = two_player_game();
    g.players[0].hand.clear();
    let scepter = g.add_card_to_hand(0, catalog::isochron_scepter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt()); // MV-1 instant
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: scepter, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Isochron Scepter for {2}");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bolt), "Bolt imprinted (exiled)");
    g.clear_sickness(scepter);
    g.players[0].mana_pool.add_colorless(2);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // yes, cast it
    g.perform_action(GameAction::ActivateAbility {
        card_id: scepter, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("free-cast the imprinted Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 3, "the imprinted Bolt dealt 3 to the opponent");
}

// ── Phasing (CR 702.26) ───────────────────────────────────────────────────

/// Tolarian Drake phases out at its controller's untap step, vanishing from
/// the battlefield, and phases back in the following untap step.
#[test]
fn tolarian_drake_phases_out_and_back_in_on_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::tolarian_drake());
    g.active_player_idx = 0;
    assert!(g.battlefield_find(id).is_some(), "starts phased in");
    g.do_phasing();
    assert!(g.battlefield_find(id).is_none(), "phased out — gone from battlefield");
    assert!(g.phased_out.iter().any(|c| c.id == id), "tracked in phased_out");
    g.do_phasing();
    assert!(g.battlefield_find(id).is_some(), "phased back in");
    assert!(g.phased_out.is_empty(), "side zone emptied");
}

/// CR 702.26 — a "when this phases in" trigger fires when the permanent
/// returns during its controller's untap step.
#[test]
fn phasing_in_fires_when_this_phases_in_trigger() {
    use crabomination::card::{CardDefinition, CardType, Keyword, TriggeredAbility};
    use crabomination::effect::{Effect, EventKind, EventScope, EventSpec, Selector, Value};
    let def = CardDefinition {
        name: "Phase Sentinel",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Phasing],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PhasesIn, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    };
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, def);
    g.add_card_to_library(0, catalog::forest());
    g.active_player_idx = 0;
    let hand0 = g.players[0].hand.len();
    g.do_phasing(); // out — no trigger
    assert_eq!(g.players[0].hand.len(), hand0, "no draw on phase-out");
    g.do_phasing(); // in — trigger fires
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_some(), "phased back in");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "phase-in trigger drew a card");
}

/// Phasing is not a zone change — counters and damage are retained across a
/// phase-out/phase-in cycle (CR 702.26e).
#[test]
fn phasing_retains_counters_across_cycle() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::tolarian_drake());
    g.battlefield_find_mut(id).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.active_player_idx = 0;
    g.do_phasing(); // out
    g.do_phasing(); // in
    let c = g.battlefield_find(id).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 2, "counters survive phasing");
}

/// Vodalian Illusionist's activated ability phases a target creature out;
/// it phases back in at its controller's next untap step.
#[test]
fn vodalian_illusionist_phases_target_out() {
    let mut g = two_player_game();
    let illu = g.add_card_to_battlefield(0, catalog::vodalian_illusionist());
    g.clear_sickness(illu);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: illu, ability_index: 0,
        target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activatable for {U}{U}, T");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim phased out");
    assert!(g.phased_out.iter().any(|c| c.id == victim), "tracked in phased_out");
    // Its controller (player 1) untaps next: phases back in.
    g.active_player_idx = 1;
    g.do_phasing();
    assert!(g.battlefield_find(victim).is_some(), "phased back in on owner's untap");
}

/// Changeling Hero (Champion, CR 702.77) exiles another creature you control
/// on ETB; when the Hero leaves, the championed creature returns.
#[test]
fn changeling_hero_champions_and_returns_on_leave() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let hero = g.add_card_to_hand(0, catalog::changeling_hero());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: hero, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Changeling Hero castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "championed bear is exiled");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear sits in exile");
    // Hero dies → the championed bear returns to the battlefield.
    let hero_id = g.battlefield.iter().find(|c| c.definition.name == "Changeling Hero").unwrap().id;
    g.remove_to_graveyard_with_triggers(hero_id);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear returns when the Hero leaves");
}

/// Breezekeeper is a 4/4 flyer with Phasing.
#[test]
fn breezekeeper_phases_on_untap() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::breezekeeper());
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.battlefield_find(id).is_none(), "Breezekeeper phased out");
}

/// Reality Ripple phases out a target creature.
#[test]
fn reality_ripple_phases_target_out() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rip = g.add_card_to_hand(0, catalog::reality_ripple());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rip, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reality Ripple castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear phased out");
    assert!(g.phased_out.iter().any(|c| c.id == bear));
}

// ── Forecast (CR 702.56) ───────────────────────────────────────────────────

/// Steeling Stance's Forecast ability pumps a target creature from hand
/// during the controller's upkeep, leaving the card in hand.
#[test]
fn steeling_stance_forecast_pumps_from_hand_in_upkeep() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let stance = g.add_card_to_hand(0, catalog::steeling_stance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: stance, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Forecast activatable from hand in upkeep");
    drain_stack(&mut g);
    let b = g.battlefield_find(bear).unwrap();
    assert_eq!((b.power(), b.toughness()), (3, 3), "Forecast pumped the bear +1/+1");
    assert!(g.players[0].hand.iter().any(|c| c.id == stance), "card stays in hand (revealed)");
}

/// The Forecast ability can't be activated outside the controller's upkeep.
#[test]
fn steeling_stance_forecast_blocked_outside_upkeep() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let stance = g.add_card_to_hand(0, catalog::steeling_stance());
    g.players[0].mana_pool.add(Color::White, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: stance, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "Forecast is upkeep-only");
}

/// Teferi's Drake (3/2 flyer) has Phasing and leaves play on its untap step.
#[test]
fn teferis_drake_phases_out() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::teferis_drake());
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.battlefield_find(id).is_none(), "Teferi's Drake phased out");
}

/// Frenetic Efreet's {0} ability phases it out on a won flip.
#[test]
fn frenetic_efreet_phases_out_on_won_flip() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::frenetic_efreet());
    g.clear_sickness(id);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // win
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("free activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "won flip → phased out");
    assert!(g.phased_out.iter().any(|c| c.id == id));
}

/// Frenetic Efreet sacrifices itself on a lost flip.
#[test]
fn frenetic_efreet_sacrifices_on_lost_flip() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::frenetic_efreet());
    g.clear_sickness(id);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)])); // lose
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("free activation");
    drain_stack(&mut g);
    assert!(g.battlefield_find(id).is_none(), "lost flip → sacrificed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "in graveyard");
}

/// Piercing Rays exiles a tapped creature; its Forecast taps from hand in upkeep.
#[test]
fn piercing_rays_exiles_tapped_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let rays = g.add_card_to_hand(0, catalog::piercing_rays());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: rays, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Piercing Rays castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "tapped creature exiled");
}

/// Piercing Rays' Forecast taps a target untapped creature from hand in upkeep.
#[test]
fn piercing_rays_forecast_taps_from_hand() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let rays = g.add_card_to_hand(0, catalog::piercing_rays());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rays, ability_index: 0,
        target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Forecast activatable from hand in upkeep");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().tapped, "Forecast tapped the bear");
    assert!(g.players[0].hand.iter().any(|c| c.id == rays), "card stays in hand");
}

/// Sandbar Crocodile is a 5/6 with Phasing.
#[test]
fn sandbar_crocodile_phases_out() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sandbar_crocodile());
    g.active_player_idx = 0;
    g.do_phasing();
    assert!(g.battlefield_find(id).is_none(), "Sandbar Crocodile phased out");
}

/// Changeling Titan champions a creature (Champion + 7/7 trample body).
#[test]
fn changeling_titan_champions_a_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let titan = g.add_card_to_hand(0, catalog::changeling_titan());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: titan, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Changeling Titan castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "championed bear exiled");
    let t = g.battlefield.iter().find(|c| c.definition.name == "Changeling Titan").unwrap();
    assert!(t.has_keyword(&crabomination::card::Keyword::Changeling), "Titan is every type");
}

/// Changeling Berserker enters with Haste (and champions a creature).
#[test]
fn changeling_berserker_has_haste() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let zerk = g.add_card_to_hand(0, catalog::changeling_berserker());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.perform_action(GameAction::CastSpell {
        card_id: zerk, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Changeling Berserker castable");
    drain_stack(&mut g);
    let z = g.battlefield.iter().find(|c| c.definition.name == "Changeling Berserker").unwrap();
    assert!(z.has_keyword(&crabomination::card::Keyword::Haste), "Berserker has haste");
}

/// Skyscribing makes each player draw X; its Forecast makes each draw one.
#[test]
fn skyscribing_each_player_draws_x_and_forecasts() {
    let mut g = two_player_game();
    for p in 0..2 { for _ in 0..5 { g.add_card_to_library(p, catalog::island()); } }
    let id = g.add_card_to_hand(0, catalog::skyscribing());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    let h0 = g.players[0].hand.len();
    let h1 = g.players[1].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("Skyscribing castable for X=2");
    drain_stack(&mut g);
    // P0 cast Skyscribing (-1 hand) then drew 2 (+2) → net +1; P1 drew 2.
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2);
    assert_eq!(g.players[1].hand.len(), h1 + 2);
}

/// Skyscribing's Forecast (each player draws one) fires from hand in upkeep.
#[test]
fn skyscribing_forecast_each_player_draws_one() {
    let mut g = two_player_game();
    for p in 0..2 { for _ in 0..3 { g.add_card_to_library(p, catalog::island()); } }
    let id = g.add_card_to_hand(0, catalog::skyscribing());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.priority.player_with_priority = 0;
    let h1 = g.players[1].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("Forecast activatable in upkeep");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), h1 + 1, "opponent drew one from Forecast");
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "Skyscribing stays in hand");
}

/// Changeling (CR 702.73) is honored by the general type-filter eval: an
/// Avian Changeling satisfies `HasCreatureType` for a type it doesn't print.
#[test]
fn changeling_satisfies_arbitrary_creature_type_filter() {
    use crabomination::card::{CreatureType, SelectionRequirement};
    let mut g = two_player_game();
    let avian = g.add_card_to_battlefield(0, catalog::avian_changeling());
    // It prints only Shapeshifter, but Changeling makes it every type.
    assert!(g.evaluate_requirement(
        &SelectionRequirement::HasCreatureType(CreatureType::Goblin),
        &Target::Permanent(avian), 0,
    ), "Changeling counts as a Goblin");
    assert!(g.evaluate_requirement(
        &SelectionRequirement::HasCreatureType(CreatureType::Elf),
        &Target::Permanent(avian), 0,
    ), "Changeling counts as an Elf");
}

/// Game-Trail Changeling is a 4/4 trampling Changeling.
#[test]
fn game_trail_changeling_is_a_changeling_trampler() {
    use crabomination::card::{CreatureType, SelectionRequirement};
    let mut g = two_player_game();
    let gt = g.add_card_to_battlefield(0, catalog::game_trail_changeling());
    assert!(g.battlefield_find(gt).unwrap().has_keyword(&crabomination::card::Keyword::Trample));
    assert!(g.evaluate_requirement(
        &SelectionRequirement::HasCreatureType(CreatureType::Dragon),
        &Target::Permanent(gt), 0,
    ), "Changeling counts as a Dragon");
}

/// Meekstone keeps power-3+ creatures from untapping; small ones untap.
#[test]
fn meekstone_locks_down_big_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::meekstone());
    let big = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.battlefield_find_mut(big).unwrap().tapped = true;
    g.battlefield_find_mut(small).unwrap().tapped = true;
    g.active_player_idx = 0;
    g.do_untap();
    assert!(g.battlefield_find(big).unwrap().tapped, "power-4 creature stays tapped");
    assert!(!g.battlefield_find(small).unwrap().tapped, "power-2 creature untaps");
}

/// Aether Flash deals 2 damage to each creature as it enters (killing a 2/2).
#[test]
fn aether_flash_punishes_entering_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::aether_flash());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "2/2 dies to Aether Flash's 2 damage");
}

/// Spitting Earth deals damage equal to Mountains you control.
#[test]
fn spitting_earth_scales_with_mountains() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::mountain()); }
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let se = g.add_card_to_hand(0, catalog::spitting_earth());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: se, target: Some(Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Spitting Earth castable");
    drain_stack(&mut g);
    // 3 Mountains → 3 damage to a 4/4 → survives with 3 marked.
    assert_eq!(g.battlefield_find(angel).unwrap().damage, 3, "dealt 3 (Mountains controlled)");
}

/// Carbonize deals 3 and exiles a creature that would die (no graveyard).
#[test]
fn carbonize_exiles_creature_it_kills() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let carb = g.add_card_to_hand(0, catalog::carbonize());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: carb, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Carbonize castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "killed creature is exiled");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == bear), "not in graveyard");
}

/// Rolling Thunder deals X damage spread across targets (auto-split even).
#[test]
fn rolling_thunder_deals_x_damage() {
    let mut g = two_player_game();
    let p1 = g.players[1].life;
    let rt = g.add_card_to_hand(0, catalog::rolling_thunder());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: rt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: Some(4),
    }).expect("Rolling Thunder castable for X=4");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 4, "X=4 damage to a single target player");
}

/// The bot's optional-trigger screen now inspects a static-ability reflexive
/// (`ExileDyingOpponentCreatures.when_you_do`): it takes a pure-upside body
/// (Valentin's make-a-Pest) but declines one with a self-cost.
#[test]
fn bot_declines_self_costly_static_reflexive() {
    use crabomination::card::{CardDefinition, CardType, Supertype};
    use crabomination::effect::{Effect, Selector, StaticAbility, StaticEffect, Value};
    use crabomination::server::bot::optional_trigger_beneficial;

    let mut g = two_player_game();
    // Real Valentin: reflexive makes a Pest (upside) → bot takes it.
    let valentin = g.add_card_to_battlefield(0, catalog::valentin_dean_of_the_vein());
    assert!(
        optional_trigger_beneficial(&g, valentin, "Pay {2} to create a 1/1 BG Pest."),
        "bot takes the upside Pest reflexive"
    );

    // Synthetic variant whose static reflexive loses you life → bot declines.
    let costly = CardDefinition {
        name: "Costly Reaper",
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "test",
            effect: StaticEffect::ExileDyingOpponentCreatures {
                when_you_do: Some(Box::new(Effect::MayPay {
                    description: "Pay {2}: lose 5 life.".into(),
                    mana_cost: crabomination::mana::cost(&[crabomination::mana::generic(2)]),
                    body: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::Const(5) }),
                    else_: None,
                })),
            },
        }],
        ..Default::default()
    };
    let reaper = g.add_card_to_battlefield(0, costly);
    assert!(
        !optional_trigger_beneficial(&g, reaper, "Pay {2}: lose 5 life."),
        "bot declines a self-costly static reflexive"
    );

    // A reflexive tap-cost trigger (Caparocti's "tap two → discover 3") is pure
    // upside once it's already attacking, so the bot accepts it.
    let cap = g.add_card_to_battlefield(0, catalog::caparocti_sunborn());
    assert!(
        optional_trigger_beneficial(
            &g, cap, "tap two untapped artifacts and/or creatures you control"),
        "bot accepts the tap-to-discover MayTap trigger",
    );
}

/// Cacophony Scamp's dies-trigger deals its *last-known* (counter-boosted)
/// power to any target via CR 603.10 LKI — not the 1 printed power its
/// graveyard copy would report.
#[test]
fn cacophony_scamp_dies_deals_power_to_target() {
    let mut g = two_player_game();
    let scamp = g.add_card_to_battlefield(0, catalog::cacophony_scamp());
    // Two +1/+1 counters → 3/3.
    g.battlefield_find_mut(scamp).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    g.players[1].life = 20;
    // Lethal damage → dies; the dies-trigger auto-targets the opponent.
    g.battlefield_find_mut(scamp).unwrap().damage = 3;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "3-power scamp dies → 3 damage to opponent");
}

/// Heartfire Hero's Valiant grows it the first time each turn it's targeted by
/// a spell/ability you control, and only once per turn (CR 603.3d).
#[test]
fn heartfire_hero_valiant_grows_once_per_turn() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::heartfire_hero());
    // First friendly target this turn → +1/+1.
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: hero, caster: 0 , by: None }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hero).unwrap().power, 2, "Valiant added a counter");
    // Second friendly target same turn → no additional counter.
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: hero, caster: 0 , by: None }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hero).unwrap().power, 2, "Valiant only fires once per turn");
}

/// Wildfire Wickerfolk grows to 4/3 with trample once delirium is active.
#[test]
fn wildfire_wickerfolk_delirium_pumps_and_grants_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::wildfire_wickerfolk());
    let c0 = g.computed_permanent(id).unwrap();
    assert_eq!((c0.power, c0.toughness), (3, 2));
    assert!(!c0.keywords.contains(&Keyword::Trample));
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::duress());
    g.add_card_to_graveyard(0, catalog::forest());
    let c1 = g.computed_permanent(id).unwrap();
    assert_eq!((c1.power, c1.toughness), (4, 3), "delirium → +1/+1");
    assert!(c1.keywords.contains(&Keyword::Trample), "delirium grants trample");
}

/// Kindlespark Duo taps to ping the opponent, and casting a noncreature spell
/// untaps it so it can ping again.
#[test]
fn kindlespark_duo_pings_and_untaps_on_noncreature_cast() {
    let mut g = two_player_game();
    let duo = g.add_card_to_battlefield(0, catalog::kindlespark_duo());
    g.clear_sickness(duo);
    g.players[1].life = 20;
    g.perform_action(GameAction::ActivateAbility {
        card_id: duo, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap to ping");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "ping dealt 1");
    assert!(g.battlefield_find(duo).unwrap().tapped, "tapped after activating");
    // Cast a noncreature spell → untap trigger.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(crabomination::game::Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bolt");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(duo).unwrap().tapped, "untapped by noncreature-cast trigger");
}

/// Mabel anthems other Mice and mints an equippable Cragflame on ETB.
#[test]
fn mabel_anthems_mice_and_mints_cragflame() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::heartfire_hero()); // 1/1 Mouse
    let mabel = g.add_card_to_battlefield(0, catalog::mabel_heir_to_cragflame());
    g.fire_self_etb_triggers(mabel, 0);
    drain_stack(&mut g);
    // Other Mouse gets +1/+1.
    assert_eq!(g.computed_permanent(hero).unwrap().power, 2, "Mabel anthems the Mouse");
    // Cragflame Equipment token exists and can be equipped onto the hero.
    let cragflame = g.battlefield.iter().find(|c| c.definition.name == "Cragflame")
        .expect("Cragflame minted").id;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: cragflame, target: hero })
        .expect("equip {2}");
    let cp = g.computed_permanent(hero).unwrap();
    // 1/1 base + anthem (+1/+1) + Cragflame (+1/+1) = 3/3 with haste.
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Haste) && cp.keywords.contains(&Keyword::Trample));
}

/// Daring Waverider's ETB free-casts an instant/sorcery (MV ≤ 4) from your
/// graveyard.
#[test]
fn daring_waverider_recasts_cheap_spell_from_graveyard() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.players[1].life = 20;
    // Accept the optional "you may cast" prompt.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let wave = g.add_card_to_battlefield(0, catalog::daring_waverider());
    g.fire_self_etb_triggers(wave, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "free-cast Bolt dealt 3");
    // Exiled after resolving (not back in graveyard).
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bolt), "Bolt exiled, not in graveyard");
}

/// Diresight surveils, draws two, and costs 2 life.
#[test]
fn diresight_draws_two_and_loses_two_life() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(0, catalog::forest()); }
    let hand0 = g.players[0].hand.len();
    let life0 = g.players[0].life;
    let id = g.add_card_to_hand(0, catalog::diresight());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Diresight");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 2, "drew two");
    assert_eq!(g.players[0].life, life0 - 2, "lost two life");
}

/// Starscape Cleric drains each opponent whenever you gain life.
#[test]
fn starscape_cleric_drains_on_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::starscape_cleric());
    g.players[1].life = 20;
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::LifeGained { player: 0, amount: 3 }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent loses 1 per lifegain event");
}

/// Splash Lasher taps a creature and stuns it on ETB.
#[test]
fn splash_lasher_taps_and_stuns_on_etb() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let lash = g.add_card_to_battlefield(0, catalog::splash_lasher());
    g.fire_self_etb_triggers(lash, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).unwrap();
    assert!(c.tapped, "bear tapped");
    assert_eq!(c.counter_count(CounterType::Stun), 1, "bear has a stun counter");
}

/// Hare Apparent mints a Rabbit per other Hare Apparent you control.
#[test]
fn hare_apparent_mints_rabbit_per_copy() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hare_apparent());
    g.add_card_to_battlefield(0, catalog::hare_apparent());
    let third = g.add_card_to_battlefield(0, catalog::hare_apparent());
    g.fire_self_etb_triggers(third, 0);
    drain_stack(&mut g);
    let rabbits = g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Rabbit").count();
    assert_eq!(rabbits, 2, "two other Hare Apparents → two Rabbit tokens");
}

/// Valley Questcaller anthems your small typal creatures.
#[test]
fn valley_questcaller_anthems_typal_creatures() {
    let mut g = two_player_game();
    let mouse = g.add_card_to_battlefield(0, catalog::heartfire_hero()); // 1/1 Mouse
    g.add_card_to_battlefield(0, catalog::valley_questcaller());
    let cp = g.computed_permanent(mouse).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 2), "Mouse anthemed +1/+1");
}

/// Roughshod Duo's "expend 4" trigger (CR 700.14) fires once the turn's
/// spell-mana total crosses four, pumping a creature you control.
#[test]
fn roughshod_duo_expend_four_pumps_creature() {
    let mut g = two_player_game();
    let duo = g.add_card_to_battlefield(0, catalog::roughshod_duo()); // 3/2
    // Cast a 6-mana spell — crosses the expend-4 threshold (0 → 6).
    let moose = g.add_card_to_hand(0, catalog::galewind_moose()); // {4}{G}{G}
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: moose, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Galewind Moose");
    drain_stack(&mut g);
    // The expend-4 trigger resolved above the Moose spell, pumping the only
    // creature on the battlefield (Roughshod Duo) to 4/3.
    assert_eq!(g.computed_permanent(duo).unwrap().power, 4, "expend-4 pumped the Duo");
}

/// Expend triggers fire on the crossing cast only, not on later casts that
/// stay above the threshold.
#[test]
fn expend_threshold_fires_only_on_crossing() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    // prev<4, total reaches 5 → ExpendReached(4) true.
    g.expend_prev_total = 2;
    let pred_true = g.evaluate_predicate(
        &crabomination::card::Predicate::ExpendReached(4),
        &crabomination::game::effects::EffectContext { event_amount: 5, ..crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0) },
    );
    assert!(pred_true, "crossing 4 fires");
    // Already above 4 before this cast → no re-fire.
    g.expend_prev_total = 5;
    let pred_false = g.evaluate_predicate(
        &crabomination::card::Predicate::ExpendReached(4),
        &crabomination::game::effects::EffectContext { event_amount: 7, ..crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0) },
    );
    assert!(!pred_false, "staying above 4 does not re-fire");
}

/// CR 702.146e — casting a daybound spell while it's neither day nor night
/// makes it day.
#[test]
fn daybound_spell_cast_makes_it_day() {
    use crabomination::game::types::DayNight;
    let mut g = two_player_game();
    assert_eq!(g.day_night, None, "neither day nor night at start");
    let vw = g.add_card_to_hand(0, catalog::village_watch());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: vw, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Village Watch");
    assert_eq!(g.day_night, Some(DayNight::Day), "daybound cast set it to day");
}

/// Repel Calamity destroys a big creature but not a small one.
#[test]
fn repel_calamity_destroys_large_creature() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::repel_calamity());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(big)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Repel Calamity");
    drain_stack(&mut g);
    assert!(g.battlefield_find(big).is_none(), "4/4 destroyed");
}

/// Pileated Provisioner puts a counter on a non-flying creature you control.
#[test]
fn pileated_provisioner_counters_nonflyer() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 no flying
    let prov = g.add_card_to_battlefield(0, catalog::pileated_provisioner());
    g.fire_self_etb_triggers(prov, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Three Tree Rootweaver taps for one mana of any color.
#[test]
fn three_tree_rootweaver_taps_for_any_color() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::three_tree_rootweaver());
    g.clear_sickness(id);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 1, "produced one mana");
}

/// Nettle Guard sacrifices to destroy an artifact.
#[test]
fn nettle_guard_sacrifices_to_destroy_artifact() {
    let mut g = two_player_game();
    let clue = g.add_card_to_battlefield(1, catalog::mind_stone()); // an artifact
    let guard = g.add_card_to_battlefield(0, catalog::nettle_guard());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: guard, ability_index: 0,
        target: Some(crabomination::game::Target::Permanent(clue)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac to destroy");
    drain_stack(&mut g);
    assert!(g.battlefield_find(guard).is_none(), "Nettle Guard sacrificed");
    assert!(g.battlefield_find(clue).is_none(), "artifact destroyed");
}

/// Frilled Sparkshooter enters bigger when an opponent lost life this turn.
#[test]
fn frilled_sparkshooter_grows_if_opponent_lost_life() {
    let mut g = two_player_game();
    g.players[1].lost_life_this_turn = true;
    let id = g.add_card_to_battlefield(0, catalog::frilled_sparkshooter());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "entered with a +1/+1 counter");
}

/// Iridescent Vinelasher pings the opponent on landfall.
#[test]
fn iridescent_vinelasher_pings_on_landfall() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::iridescent_vinelasher());
    g.players[1].life = 20;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "landfall pinged the opponent for 1");
}

/// Rabbit Response pumps your team and scries when you control a Rabbit.
#[test]
fn rabbit_response_pumps_team() {
    let mut g = two_player_game();
    let rabbit = g.add_card_to_battlefield(0, catalog::hare_apparent()); // 2/2 Rabbit
    let id = g.add_card_to_hand(0, catalog::rabbit_response());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rabbit Response");
    drain_stack(&mut g);
    let cp = g.computed_permanent(rabbit).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 3), "+2/+1 from Rabbit Response");
}

/// Brazen Collector adds {R} when it attacks.
#[test]
fn brazen_collector_adds_red_on_attack() {
    let mut g = two_player_game();
    let bc = g.add_card_to_battlefield(0, catalog::brazen_collector());
    g.battlefield_find_mut(bc).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: bc, target: AttackTarget::Player(1) }])
        .expect("Brazen Collector attacks");
    drain_stack(&mut g);
    assert!(g.players[0].mana_pool.amount(Color::Red) >= 1, "added red on attack");
}

/// Pawpatch Formation mode 0 destroys a flyer.
#[test]
fn pawpatch_formation_destroys_flyer() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // flyer
    let id = g.add_card_to_hand(0, catalog::pawpatch_formation());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(angel)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Pawpatch Formation mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "flyer destroyed");
}

/// Bant Charm mode 1 tucks a creature to the bottom of its owner's library.
#[test]
fn bant_charm_tucks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bant_charm());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("cast Bant Charm mode 1");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature left the battlefield");
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(bear), "tucked to library bottom");
}

/// Skyskipper Duo blinks one of your creatures on ETB.
#[test]
fn skyskipper_duo_blinks_your_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let duo = g.add_card_to_battlefield(0, catalog::skyskipper_duo());
    g.fire_self_etb_triggers(duo, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear exiled by the blink");
    assert!(g.exile.iter().any(|c| c.id == bear), "bear is in exile until end step");
}

/// Wax-Wane Witness pumps itself when you gain life.
#[test]
fn wax_wane_witness_grows_on_lifegain() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::wax_wane_witness());
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::LifeGained { player: 0, amount: 1 }]);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "Wax-Wane Witness grew +1/+0");
}

/// Might of the Meek grants trample, draws, and pumps when you control a Mouse.
#[test]
fn might_of_the_meek_pumps_with_mouse() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    // A Mouse on board satisfies the conditional; target a plain bear so the
    // pump is observed without Valiant noise.
    g.add_card_to_battlefield(0, catalog::heartfire_hero()); // 1/1 Mouse
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::forest()); // something to draw
    let id = g.add_card_to_hand(0, catalog::might_of_the_meek());
    g.players[0].mana_pool.add(Color::Red, 1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Might of the Meek");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::Trample), "gained trample");
    assert_eq!(cp.power, 3, "+1/+0 because you control a Mouse");
    assert_eq!(g.players[0].hand.len(), hand0, "cast one, drew one (net even)");
}

/// Daggerfang Duo enters with deathtouch and may mill two.
#[test]
fn daggerfang_duo_mills_two_on_etb() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_battlefield(0, catalog::daggerfang_duo());
    assert!(g.battlefield_find(id).unwrap().definition.keywords.contains(&Keyword::Deathtouch));
    let gy0 = g.players[0].graveyard.len();
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), gy0 + 2, "milled two cards");
}

/// Valley Rotcaller drains for each other Squirrel/Bat/Lizard/Rat on attack.
#[test]
fn valley_rotcaller_drains_per_tribal_creature() {
    let mut g = two_player_game();
    let rot = g.add_card_to_battlefield(0, catalog::valley_rotcaller());
    g.add_card_to_battlefield(0, catalog::daggerfang_duo()); // Rat Squirrel
    g.add_card_to_battlefield(0, catalog::iridescent_vinelasher()); // Lizard
    g.battlefield_find_mut(rot).unwrap().summoning_sick = false;
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: rot, target: AttackTarget::Player(1) }])
        .expect("Rotcaller attacks");
    drain_stack(&mut g);
    // Two other tribal creatures → drain 2.
    assert_eq!(g.players[1].life, 18, "opponent lost 2");
    assert_eq!(g.players[0].life, 22, "you gained 2");
}

/// Nocturnal Hunger destroys a creature.
#[test]
fn nocturnal_hunger_destroys_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::nocturnal_hunger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Nocturnal Hunger");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
}

/// Conduct Electricity deals 6 to a creature.
#[test]
fn conduct_electricity_deals_six() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::conduct_electricity());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(angel)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Conduct Electricity");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "6 damage kills the 4/4");
}

/// Carrot Cake makes a Rabbit on ETB and again when sacrificed for life.
#[test]
fn carrot_cake_makes_rabbits_on_etb_and_sacrifice() {
    let mut g = two_player_game();
    let cake = g.add_card_to_battlefield(0, catalog::carrot_cake());
    g.fire_self_etb_triggers(cake, 0);
    drain_stack(&mut g);
    let after_etb = g.battlefield.iter().filter(|c| c.definition.name == "Rabbit").count();
    assert_eq!(after_etb, 1, "ETB minted a Rabbit");
    // Sacrifice for life → second Rabbit + 3 life.
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: cake, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac Carrot Cake");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3, "gained 3 life");
    let after_sac = g.battlefield.iter().filter(|c| c.definition.name == "Rabbit").count();
    assert_eq!(after_sac, 2, "sacrifice trigger minted a second Rabbit");
}

/// Spineseeker Centipede tutors a basic land to hand on ETB.
#[test]
fn spineseeker_centipede_tutors_basic_land() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let hand0 = g.players[0].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::spineseeker_centipede());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "fetched a basic land");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Forest"), "Forest in hand");
}

/// Spineseeker Centipede's Delirium grants +1/+2 and vigilance at 4+ types.
#[test]
fn spineseeker_centipede_delirium_pumps_and_grants_vigilance() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::spineseeker_centipede());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (2, 1), "base 2/1 without delirium");
    assert!(!c.keywords.contains(&Keyword::Vigilance));
    // Four card types in graveyard: Creature, Land, Sorcery, Instant.
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::ponder());
    g.add_card_to_graveyard(0, catalog::manamorphose());
    let cp = g.compute_battlefield();
    let c = cp.iter().find(|c| c.id == id).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "delirium +1/+2");
    assert!(c.keywords.contains(&Keyword::Vigilance), "delirium grants vigilance");
}

/// Mind Drill Assailant gets +3/+0 once seven cards are in your graveyard.
#[test]
fn mind_drill_assailant_threshold_pumps_power() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::mind_drill_assailant());
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((2, 5)));
    for _ in 0..7 { g.add_card_to_graveyard(0, catalog::forest()); }
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == id).map(|c| (c.power, c.toughness)), Some((5, 5)),
        "Threshold grants +3/+0");
}

/// Adaptive Automaton pumps other creatures of the chosen type, not itself.
#[test]
fn adaptive_automaton_anthems_chosen_type() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let auto = g.add_card_to_battlefield(0, catalog::adaptive_automaton());
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_bushwhacker()); // 1/1 Goblin
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 non-Goblin
    // Stamp the chosen type directly (the ETB NameCreatureType path is tested
    // separately for Cavern of Souls).
    g.battlefield.iter_mut().find(|c| c.id == auto).unwrap().chosen_creature_type =
        Some(CreatureType::Goblin);
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == goblin).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "other Goblin +1/+1");
    assert_eq!(cp.iter().find(|c| c.id == bear).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "non-Goblin unaffected");
    assert_eq!(cp.iter().find(|c| c.id == auto).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "Automaton excludes itself (Other ...)");
}

/// Patchwork Banner pumps every creature of the chosen type (no self-exclude).
#[test]
fn patchwork_banner_anthems_chosen_type() {
    use crabomination::card::CreatureType;
    let mut g = two_player_game();
    let banner = g.add_card_to_battlefield(0, catalog::patchwork_banner());
    let goblin = g.add_card_to_battlefield(0, catalog::goblin_bushwhacker());
    let theirs = g.add_card_to_battlefield(1, catalog::goblin_bushwhacker()); // opp's Goblin
    g.battlefield.iter_mut().find(|c| c.id == banner).unwrap().chosen_creature_type =
        Some(CreatureType::Goblin);
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == goblin).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "your Goblin +1/+1");
    assert_eq!(cp.iter().find(|c| c.id == theirs).map(|c| (c.power, c.toughness)), Some((1, 1)),
        "opponent's Goblin unaffected (creatures you control)");
}

/// Daring Fiendbonder activates from the graveyard, exiling itself to put an
/// indestructible counter on a creature. Sorcery speed only.
#[test]
fn daring_fiendbonder_exiles_self_to_grant_indestructible() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fiend = g.add_card_to_graveyard(0, catalog::daring_fiendbonder());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: fiend,
        ability_index: 0,
        target: Some(crabomination::game::Target::Permanent(target)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("activate from graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == fiend), "source exiled as cost");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == fiend));
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::Indestructible),
        1,
        "indestructible counter placed on target",
    );
}

/// Dreg Mangler scavenges from the graveyard: exile it for 3 +1/+1 counters
/// (equal to its power) on a target creature.
#[test]
fn dreg_mangler_scavenge_grants_counters_equal_to_power() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mangler = g.add_card_to_graveyard(0, catalog::dreg_mangler());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: mangler,
        ability_index: 0,
        target: Some(crabomination::game::Target::Permanent(target)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("scavenge from graveyard");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == mangler), "source exiled");
    assert_eq!(
        g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne),
        3,
        "+1/+1 counters equal to Dreg Mangler's power (3)",
    );
}

/// Drift of Phantasms transmutes: discard it to tutor an MV-3 card to hand.
#[test]
fn drift_of_phantasms_transmute_tutors_same_mv() {
    let mut g = two_player_game();
    // An MV-3 card and an MV-1 card in library; transmute should fetch the MV-3.
    let mv3 = g.add_card_to_library(0, catalog::heros_downfall()); // {1}{B}{B} = MV 3
    let _mv1 = g.add_card_to_library(0, catalog::ponder()); // {U} = MV 1
    let drift = g.add_card_to_hand(0, catalog::drift_of_phantasms());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(mv3)),
    ]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: drift,
        ability_index: 0,
        target: None,
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("transmute");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == drift), "Drift discarded");
    assert!(g.players[0].hand.iter().any(|c| c.id == mv3), "MV-3 card tutored to hand");
}

/// Metallic Mimic: a creature of the chosen type entering under your control
/// gets an extra +1/+1 counter — for tokens and reanimation too.
#[test]
fn metallic_mimic_chosen_type_enters_with_extra_counter() {
    use crabomination::card::{CounterType, CreatureType};
    let mut g = two_player_game();
    let mimic = g.add_card_to_battlefield(0, catalog::metallic_mimic());
    g.battlefield.iter_mut().find(|c| c.id == mimic).unwrap().chosen_creature_type =
        Some(CreatureType::Goblin);
    // A Goblin entering via reanimation/move gains the counter.
    let goblin = g.move_card_to_battlefield_for_test(0, catalog::goblin_bushwhacker());
    assert_eq!(
        g.battlefield_find(goblin).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "chosen-type Goblin enters with an extra +1/+1 counter",
    );
    // A non-Goblin gets nothing.
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    assert_eq!(
        g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne),
        0,
    );
}

/// Bounding Krasis taps a target creature on ETB.
#[test]
fn bounding_krasis_taps_a_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::bounding_krasis());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Target(crabomination::game::Target::Permanent(foe)),
    ]));
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target creature tapped");
}

/// Charming Prince mode 2 gains 3 life.
#[test]
fn charming_prince_gains_three_life() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    let id = g.add_card_to_battlefield(0, catalog::charming_prince());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Modes(vec![1]),
    ]));
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3, "mode 2 gains 3 life");
}

/// Tatyova draws a card and gains a life on landfall.
#[test]
fn tatyova_landfall_draws_and_gains_life() {
    use crabomination::TurnStep;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tatyova_benthic_druid());
    let land = g.add_card_to_hand(0, catalog::forest());
    let life0 = g.players[0].life;
    let hand0 = g.players[0].hand.len();
    g.add_card_to_library(0, catalog::ponder()); // something to draw
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1, "landfall gains 1 life");
    // -1 for the land leaving hand, +1 drawn → net same count, but a card was drawn.
    assert_eq!(g.players[0].hand.len(), hand0, "drew a card (land left, card drawn)");
}

/// Dragonlord Atarka divides 5 ETB damage onto opponents' creatures.
#[test]
fn dragonlord_atarka_etb_divides_five_damage() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_battlefield(0, catalog::dragonlord_atarka());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    // AutoDecider spreads damage; a single 2/2 takes at least lethal and dies.
    assert!(g.battlefield_find(foe).is_none(), "opponent's 2/2 dies to divided damage");
}

/// Risen Reef triggers when an Elemental you control enters, putting a revealed
/// land onto the battlefield.
#[test]
fn risen_reef_etb_puts_land_onto_battlefield() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::risen_reef());
    // Top of library is a Forest; cast a second Risen Reef to trigger the first.
    let fid = g.next_id();
    g.players[0].add_to_library_top(fid, catalog::forest());
    let bf0 = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let reef2 = g.add_card_to_hand(0, catalog::risen_reef());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: reef2, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Risen Reef");
    drain_stack(&mut g);
    let bf1 = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert!(bf1 > bf0, "the revealed Forest entered the battlefield");
}

/// Trinket Mage tutors a small artifact to hand.
#[test]
fn trinket_mage_tutors_small_artifact() {
    let mut g = two_player_game();
    let relic = g.add_card_to_library(0, catalog::ornithopter()); // {0} artifact
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(relic)),
    ]));
    let id = g.add_card_to_battlefield(0, catalog::trinket_mage());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == relic), "tutored the MV-0 artifact");
}

/// Mistmeadow Witch blinks a creature: it leaves and returns.
#[test]
fn mistmeadow_witch_blinks_a_creature() {
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let witch = g.add_card_to_battlefield(0, catalog::mistmeadow_witch());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: witch,
        ability_index: 0,
        target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: Vec::new(),
        x_value: None, mode: None,
    }).expect("blink");
    drain_stack(&mut g);
    // The original bear is exiled (a new object returns at end step).
    assert!(g.battlefield_find(bear).is_none(), "original creature exiled by blink");
}

/// Master Splicer makes a Golem and anthems Golems +1/+1 (token is 4/4).
#[test]
fn master_splicer_creates_and_buffs_golem() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::master_splicer());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let golem_id = g.battlefield.iter().find(|c| c.definition.name == "Golem")
        .expect("Golem token minted").id;
    let cp = g.compute_battlefield();
    let golem = cp.iter().find(|c| c.id == golem_id).unwrap();
    assert_eq!((golem.power, golem.toughness), (4, 4), "3/3 base + anthem +1/+1");
}

/// Geist of Saint Traft makes a tapped+attacking 4/4 Angel when it attacks.
#[test]
fn geist_of_saint_traft_makes_attacking_angel() {
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let geist = g.add_card_to_battlefield(0, catalog::geist_of_saint_traft());
    if let Some(c) = g.battlefield_find_mut(geist) { c.summoning_sick = false; }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: geist, target: AttackTarget::Player(1) }])
        .expect("Geist attacks");
    drain_stack(&mut g);
    let angel = g.battlefield.iter().find(|c| c.definition.name == "Angel")
        .expect("Angel token created");
    assert!(angel.tapped, "Angel is tapped");
    assert!(g.attacking.iter().any(|a| a.attacker == angel.id), "Angel is attacking");
}

/// Drogskol Captain buffs other Spirits +1/+1 and grants hexproof, not itself.
#[test]
fn drogskol_captain_buffs_other_spirits() {
    use crabomination::card::{CreatureType, Keyword, CardType, CardDefinition, Subtypes};
    let mut g = two_player_game();
    let cap = g.add_card_to_battlefield(0, catalog::drogskol_captain());
    let spirit = g.add_card_to_battlefield(0, CardDefinition {
        name: "Test Spirit",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    });
    let cp = g.compute_battlefield();
    let s = cp.iter().find(|c| c.id == spirit).unwrap();
    assert_eq!((s.power, s.toughness), (2, 2), "other Spirit +1/+1");
    assert!(s.keywords.contains(&Keyword::Hexproof), "other Spirit gains hexproof");
    let c = cp.iter().find(|c| c.id == cap).unwrap();
    assert_eq!((c.power, c.toughness), (2, 2), "captain doesn't buff itself");
}

/// Lord of the Unreal buffs Illusions (including itself? no — only Illusions).
#[test]
fn lord_of_the_unreal_buffs_illusions() {
    use crabomination::card::{CreatureType, Keyword, CardType, CardDefinition, Subtypes};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lord_of_the_unreal());
    let illusion = g.add_card_to_battlefield(0, CardDefinition {
        name: "Phantom",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Illusion], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    });
    let cp = g.compute_battlefield();
    let i = cp.iter().find(|c| c.id == illusion).unwrap();
    assert_eq!((i.power, i.toughness), (3, 3), "Illusion +1/+1");
    assert!(i.keywords.contains(&Keyword::Hexproof));
}

/// Lord of Atlantis grants other Merfolk +1/+1 and islandwalk.
#[test]
fn lord_of_atlantis_buffs_merfolk_with_islandwalk() {
    use crabomination::card::{CreatureType, Keyword, LandType, CardType, CardDefinition, Subtypes};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lord_of_atlantis());
    let fish = g.add_card_to_battlefield(0, CardDefinition {
        name: "Merperson",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Merfolk], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    });
    let cp = g.compute_battlefield();
    let f = cp.iter().find(|c| c.id == fish).unwrap();
    assert_eq!((f.power, f.toughness), (2, 2), "other Merfolk +1/+1");
    assert!(f.keywords.contains(&Keyword::Landwalk(LandType::Island)), "gains islandwalk");
}

/// Death Baron's disjunctive anthem buffs both Zombies and Skeletons + deathtouch.
#[test]
fn death_baron_buffs_zombies_and_skeletons() {
    use crabomination::card::{CreatureType, Keyword, CardType, CardDefinition, Subtypes};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::death_baron());
    let mk = |name: &'static str, ct: CreatureType| CardDefinition {
        name,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    let zombie = g.add_card_to_battlefield(0, mk("Z", CreatureType::Zombie));
    let skeleton = g.add_card_to_battlefield(0, mk("S", CreatureType::Skeleton));
    let cp = g.compute_battlefield();
    for id in [zombie, skeleton] {
        let c = cp.iter().find(|c| c.id == id).unwrap();
        assert_eq!((c.power, c.toughness), (2, 2), "tribe member +1/+1");
        assert!(c.keywords.contains(&Keyword::Deathtouch), "tribe member gains deathtouch");
    }
}

/// Thoughtcast's affinity discounts it by {1} per artifact; it draws two.
#[test]
fn thoughtcast_affinity_discounts_and_draws_two() {
    let mut g = two_player_game();
    // Two artifacts → {2} off, so {2}{U} pays for {4}{U}.
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_library(0, catalog::ponder());
    g.add_card_to_library(0, catalog::index());
    let id = g.add_card_to_hand(0, catalog::thoughtcast());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2); // {2}{U} = 4U - {2} affinity
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("affinity-discounted Thoughtcast castable for {2}{U}");
    drain_stack(&mut g);
    // -1 for Thoughtcast leaving hand, +2 drawn → net +1.
    assert_eq!(g.players[0].hand.len(), hand0 - 1 + 2, "drew two cards");
}

/// Etched Champion gains protection from all colors only with Metalcraft (3+ artifacts).
#[test]
fn etched_champion_metalcraft_grants_protection() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::etched_champion()); // 1 artifact (itself)
    let no_metalcraft = g.computed_permanent(champ).unwrap().keywords;
    assert!(!no_metalcraft.contains(&Keyword::Protection(Color::Red)), "no protection below 3 artifacts");
    // Add two more artifacts → metalcraft on.
    g.add_card_to_battlefield(0, catalog::ornithopter());
    g.add_card_to_battlefield(0, catalog::ornithopter());
    let kws = g.computed_permanent(champ).unwrap().keywords;
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        assert!(kws.contains(&Keyword::Protection(c)), "metalcraft grants protection from {c:?}");
    }
}

/// Sai mints a Thopter whenever you cast an artifact spell.
#[test]
fn sai_makes_thopter_on_artifact_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sai_master_thopterist());
    let orn = g.add_card_to_hand(0, catalog::ornithopter()); // {0} artifact
    let thopters0 = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
    g.perform_action(GameAction::CastSpell {
        card_id: orn, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Ornithopter");
    drain_stack(&mut g);
    let thopters1 = g.battlefield.iter().filter(|c| c.definition.name == "Thopter").count();
    assert_eq!(thopters1, thopters0 + 1, "Sai mints a Thopter on the artifact cast");
}

/// Imperious Perfect anthems other Elves and taps to make an Elf Warrior.
#[test]
fn imperious_perfect_anthem_and_token() {
    use crabomination::card::{CreatureType, CardType, CardDefinition, Subtypes};
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let perfect = g.add_card_to_battlefield(0, catalog::imperious_perfect());
    let elf = g.add_card_to_battlefield(0, CardDefinition {
        name: "Llanowar",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        power: 1,
        toughness: 1,
        ..Default::default()
    });
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == elf).map(|c| (c.power, c.toughness)), Some((2, 2)),
        "other Elf +1/+1");
    // Activate the token maker.
    if let Some(c) = g.battlefield_find_mut(perfect) { c.summoning_sick = false; }
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let elves0 = g.battlefield.iter().filter(|c| c.definition.name == "Elf Warrior").count();
    g.perform_action(GameAction::ActivateAbility {
        card_id: perfect, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("make an Elf");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Elf Warrior").count(),
        elves0 + 1, "minted an Elf Warrior token");
}

/// Diregraf Captain drains an opponent when another Zombie dies.
#[test]
fn diregraf_captain_drains_on_zombie_death() {
    use crabomination::card::{CreatureType, CardType, CardDefinition, Subtypes};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::diregraf_captain());
    let zombie = g.add_card_to_battlefield(0, CardDefinition {
        name: "Walker",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        ..Default::default()
    });
    // Kill our own Zombie via lethal damage so the SBA death path dispatches
    // CreatureDied to the Captain's listener.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(zombie)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt own zombie");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "opponent loses 1 on Zombie death");
}

/// Undead Warchief gives Zombies +2/+1 (including itself).
#[test]
fn undead_warchief_buffs_zombies() {
    let mut g = two_player_game();
    let chief = g.add_card_to_battlefield(0, catalog::undead_warchief());
    let cp = g.compute_battlefield();
    assert_eq!(cp.iter().find(|c| c.id == chief).map(|c| (c.power, c.toughness)), Some((3, 2)),
        "Warchief is itself a Zombie: 1/1 + 2/1");
}

/// Kalonian Hydra doubles +1/+1 counters on each of your creatures when it attacks.
#[test]
fn kalonian_hydra_doubles_counters_on_attack() {
    use crabomination::game::types::{Attack, AttackTarget};
    use crabomination::card::CounterType;
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let hydra = g.add_card_to_battlefield(0, catalog::kalonian_hydra());
    // Manually seed the 4 ETB counters (add_card_to_battlefield bypasses ETB).
    g.battlefield_find_mut(hydra).unwrap().add_counters(CounterType::PlusOnePlusOne, 4);
    if let Some(c) = g.battlefield_find_mut(hydra) { c.summoning_sick = false; }
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::DeclareAttackers;
    g.declare_attackers(vec![Attack { attacker: hydra, target: AttackTarget::Player(1) }])
        .expect("Hydra attacks");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(hydra).unwrap().counter_count(CounterType::PlusOnePlusOne),
        8,
        "4 +1/+1 counters doubled to 8 on attack",
    );
}

/// Inspiring Overseer gains a life and draws on ETB.
#[test]
fn inspiring_overseer_gains_life_and_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::ponder());
    let life0 = g.players[0].life;
    let hand0 = g.players[0].hand.len();
    let id = g.add_card_to_battlefield(0, catalog::inspiring_overseer());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 1, "gained 1 life");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card");
}

/// Tukatongue Thallid leaves a Saproling behind when it dies.
#[test]
fn tukatongue_thallid_dies_into_saproling() {
    let mut g = two_player_game();
    let thallid = g.add_card_to_battlefield(0, catalog::tukatongue_thallid());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(thallid)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt thallid");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Saproling"), "Saproling token left behind");
}

/// Hanged Executioner makes a Spirit on ETB and exiles itself to exile a creature.
#[test]
fn hanged_executioner_etb_token_and_exile_ability() {
    use crabomination::TurnStep;
    let mut g = two_player_game();
    let exec = g.add_card_to_battlefield(0, catalog::hanged_executioner());
    g.fire_self_etb_triggers(exec, 0);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit"), "ETB Spirit token");
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: exec, ability_index: 0,
        target: Some(crabomination::game::Target::Permanent(foe)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("exile ability");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == exec), "Executioner exiled as cost");
    assert!(g.battlefield_find(foe).is_none(), "target creature exiled");
}

/// Steelshaper's Gift tutors an Equipment to hand.
#[test]
fn steelshapers_gift_tutors_equipment() {
    let mut g = two_player_game();
    let sword = g.add_card_to_library(0, catalog::bonesplitter()); // Equipment
    g.add_card_to_library(0, catalog::forest()); // non-equipment decoy
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Search(Some(sword)),
    ]));
    let id = g.add_card_to_hand(0, catalog::steelshapers_gift());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Steelshaper's Gift");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == sword), "tutored the Equipment");
}

/// Ballyrush Banneret reduces a Soldier spell's cost by {1}.
#[test]
fn ballyrush_banneret_discounts_soldier_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ballyrush_banneret());
    // A Soldier creature in hand: a {1}{W} 2/1 should be castable for just {W}.
    let soldier = g.add_card_to_hand(0, catalog::ballyrush_banneret());
    g.players[0].mana_pool.add(Color::White, 1); // only {W}, no generic
    g.perform_action(GameAction::CastSpell {
        card_id: soldier, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Soldier spell discounted to {W}");
}

/// Anafenza exiles an opponent's dying nontoken creature instead of graveyard.
#[test]
fn anafenza_exiles_opponents_dying_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::anafenza_the_foremost());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt opponent creature");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == foe), "opponent's creature exiled, not in graveyard");
    assert!(!g.players[1].graveyard.iter().any(|c| c.id == foe), "not in opponent graveyard");
}

/// Bushwhack mode 1 fights: your creature trades with theirs.
#[test]
fn bushwhack_fight_mode_trades_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::bushwhack());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(mine)),
        additional_targets: vec![crabomination::game::Target::Permanent(theirs)], mode: Some(1), x_value: None,
    }).expect("cast fight mode");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none(), "both 2/2s trade");
}

/// Gnaw to the Bone gains 2 life per creature card in the graveyard.
#[test]
fn gnaw_to_the_bone_gains_life_per_graveyard_creature() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::forest()); // not a creature — ignored
    let id = g.add_card_to_hand(0, catalog::gnaw_to_the_bone());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 4, "2 life x 2 creature cards");
}

/// Hardened-Scale Armor attaches and grants +3/+3.
#[test]
fn hardened_scale_armor_buffs_enchanted_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let id = g.add_card_to_hand(0, catalog::hardened_scale_armor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    let computed = g.compute_battlefield();
    let c = computed.iter().find(|c| c.id == bear).unwrap();
    assert_eq!((c.power, c.toughness), (5, 5), "bear is 5/5 with the aura");
}

/// Repel the Vile mode 0 exiles a power-4+ creature.
#[test]
fn repel_the_vile_exiles_big_creature() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let id = g.add_card_to_hand(0, catalog::repel_the_vile());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(angel)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast mode 0");
    drain_stack(&mut g);
    assert!(g.battlefield_find(angel).is_none(), "angel exiled");
    assert!(g.exile.iter().any(|c| c.id == angel), "in exile");
}

/// Wildwood Mentor grows when a token enters and pumps an ally on attack.
#[test]
fn wildwood_mentor_grows_on_token_and_pumps_on_attack() {
    let mut g = two_player_game();
    let mentor = g.add_card_to_battlefield(0, catalog::wildwood_mentor());
    // A token you control enters → +1/+1 on Mentor (now 2/2).
    let tok = g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: tok }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mentor).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "token entry grew Mentor");
    // Attack: another attacking creature gets +X/+X (X = Mentor power = 2).
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mentor).unwrap().summoning_sick = false;
    g.battlefield_find_mut(bear).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: mentor, target: AttackTarget::Player(1) },
        Attack { attacker: bear, target: AttackTarget::Player(1) },
    ]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 2 + 2, "bear pumped by Mentor's power (2)");
}

/// Conduit Pylons surveils on ETB and taps for colorless or any color.
#[test]
fn conduit_pylons_surveils_and_fixes_mana() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears()); // surveil sees this (AutoDecider keeps it)
    let id = g.add_card_to_battlefield(0, catalog::conduit_pylons());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    // {T}: Add {C}
    g.perform_action(GameAction::ActivateAbility { card_id: id, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None , mode: None})
        .expect("tap for C");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "added colorless");
}

/// Sinkhole Surveyor: attacking costs 1 life and endures 1.
#[test]
fn sinkhole_surveyor_attacks_loses_life_and_endures() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sinkhole_surveyor());
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life0 = g.players[0].life;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 1, "lost 1 life");
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "endured 1");
}

/// Warden of the Grove: another nontoken creature entering endures X = Warden's
/// +1/+1 counters.
#[test]
fn warden_of_the_grove_endures_entering_creatures() {
    let mut g = two_player_game();
    let warden = g.add_card_to_battlefield(0, catalog::warden_of_the_grove());
    g.battlefield_find_mut(warden).unwrap().add_counters(CounterType::PlusOnePlusOne, 2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bear }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "bear endured 2 (Warden's counters)");
}

/// Bake into a Pie destroys a creature and leaves a Food behind.
#[test]
fn bake_into_a_pie_destroys_and_makes_food() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bake_into_a_pie());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "creature destroyed");
    assert_eq!(g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Food").count(), 1, "Food token created");
}

/// Curious Forager: ETB forage (via graveyard) returns a permanent card to hand.
#[test]
fn curious_forager_returns_permanent_from_graveyard() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::forest()); } // forage fodder
    let bear_in_gy = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // target
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_battlefield(0, catalog::curious_forager());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear_in_gy), "bear returned to hand");
}

/// Sandskitter Outrider: menace + ETB endure 2 (default → counters).
#[test]
fn sandskitter_outrider_has_menace_and_endures() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::sandskitter_outrider());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(id).expect("alive");
    assert!(c.definition.keywords.contains(&crabomination::card::Keyword::Menace), "has menace");
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 2, "ETB endured 2");
}

/// Inspirited Vanguard endures 2 on enter AND again on attack.
#[test]
fn inspirited_vanguard_endures_on_enter_and_attack() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::inspirited_vanguard());
    g.fire_self_etb_triggers(id, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 2, "ETB endured 2");
    g.battlefield_find_mut(id).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: id, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(id).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 4, "attack endured 2 more");
}

/// Forage (CR 701.61): pays by exiling three graveyard cards, then draws.
#[test]
fn forage_exiles_three_graveyard_cards_then_draws() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_graveyard(0, catalog::forest()); }
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand0 = g.players[0].hand.len();
    let sentry = g.add_card_to_battlefield(0, catalog::treetop_sentries());
    g.fire_self_etb_triggers(sentry, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 0, "three cards left the graveyard");
    assert_eq!(g.exile.len(), 3, "they went to exile");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "foraged → drew a card");
}

/// Forage (CR 701.61): with an empty graveyard, falls back to sacrificing a
/// Food, then the payoff (two +1/+1 counters) resolves.
#[test]
fn forage_sacrifices_a_food_when_graveyard_too_small() {
    let mut g = two_player_game();
    let food = g.add_token_to_battlefield(0, &crabomination_base::tokens::food_token());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let body = g.add_card_to_battlefield(0, catalog::bushy_bodyguard());
    g.fire_self_etb_triggers(body, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(food).is_none(), "Food was sacrificed to forage");
    let c = g.battlefield_find(body).expect("bodyguard alive");
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 2, "foraged → +2 counters");
}

/// Endure (CR 701.63): default (AutoDecider) puts the +1/+1 counters on the
/// enduring creature rather than minting a Spirit.
#[test]
fn endure_default_grows_the_creature_with_counters() {
    let mut g = two_player_game();
    let guard = g.add_card_to_battlefield(0, catalog::dusyut_earthcarver()); // endure 3
    g.fire_self_etb_triggers(guard, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(guard).expect("guard alive");
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 3, "endure 3 added counters");
    let spirits = g.battlefield.iter().filter(|c| c.definition.name == "Spirit").count();
    assert_eq!(spirits, 0, "no token when counters chosen");
}

/// Endure (CR 701.63): the controller may instead create an N/N white Spirit.
#[test]
fn endure_can_mint_an_nn_spirit() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let guard = g.add_card_to_battlefield(0, catalog::fortress_kin_guard()); // endure 1
    g.fire_self_etb_triggers(guard, 0);
    drain_stack(&mut g);
    let c = g.battlefield_find(guard).expect("guard alive");
    assert_eq!(c.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 0, "no counters when token chosen");
    let spirit = g.battlefield.iter().find(|c| c.definition.name == "Spirit").expect("Spirit minted");
    assert_eq!((spirit.definition.power, spirit.definition.toughness), (1, 1), "1/1 Spirit");
    assert_eq!(spirit.controller, 0, "controller owns the Spirit");
}

/// Sunspine Lynx pings each player for their nonbasic-land count, locks
/// lifegain, and lets damage through any prevention shield.
#[test]
fn sunspine_lynx_pings_each_player_by_nonbasic_lands() {
    let mut g = two_player_game();
    // P0: 2 nonbasic + 1 basic; P1: 1 nonbasic.
    g.add_card_to_battlefield(0, catalog::wasteland());
    g.add_card_to_battlefield(0, catalog::wasteland());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::wasteland());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    let lynx = g.add_card_to_battlefield(0, catalog::sunspine_lynx());
    g.fire_self_etb_triggers(lynx, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, l0 - 2, "P0 took 2 (two nonbasics, basic ignored)");
    assert_eq!(g.players[1].life, l1 - 1, "P1 took 1 (one nonbasic)");
    // Lifegain is locked while the Lynx is out.
    g.adjust_life(0, 5);
    assert_eq!(g.players[0].life, l0 - 2, "no lifegain while Lynx is in play");
}

/// Carrot Cake's "when you sacrifice it" trigger is sacrifice-specific: a plain
/// destroy (non-sacrifice exit) does not mint a Rabbit.
#[test]
fn carrot_cake_sac_trigger_doesnt_fire_on_destroy() {
    let mut g = two_player_game();
    let cake = g.add_card_to_battlefield(0, catalog::carrot_cake());
    let evs = g.remove_to_graveyard_with_triggers(cake); // non-sacrifice exit
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let rabbits = g.battlefield.iter().filter(|c| c.definition.name == "Rabbit").count();
    assert_eq!(rabbits, 0, "destroy is not a sacrifice — no Rabbit");
}

/// Take Out the Trash deals 3 and loots when you control a Raccoon.
#[test]
fn take_out_the_trash_pings_and_loots_with_raccoon() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::brazen_collector()); // Raccoon
    g.add_card_to_library(0, catalog::forest());
    let extra = g.add_card_to_hand(0, catalog::forest()); // discard fodder
    let _ = extra;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let id = g.add_card_to_hand(0, catalog::take_out_the_trash());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(crabomination::game::Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Take Out the Trash");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "3 damage kills the 2/2");
}

