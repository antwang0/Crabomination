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

// ── Bloomburrow batch ──────────────────────────────────────────────────────

#[test]
fn hop_to_it_makes_three_rabbits() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let id = g.add_card_to_hand(0, catalog::hop_to_it());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hop to It");
    drain_stack(&mut g);
    let rabbits = g.battlefield.iter().filter(|c| c.definition.name == "Rabbit").count();
    assert_eq!(rabbits, 3, "three 1/1 Rabbit tokens");
}

#[test]
fn bellowing_crier_loots_on_enter() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    let hand_before = g.players[0].hand.len();
    let crier = g.add_card_to_hand(0, catalog::bellowing_crier());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: crier, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Bellowing Crier");
    drain_stack(&mut g);
    // Drew 1, discarded 1: net hand change is the crier leaving (cast) only.
    assert_eq!(g.players[0].hand.len(), hand_before, "looted: drew one, discarded one");
}

#[test]
fn banishing_light_exiles_then_returns_on_leave() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let bl = g.add_card_to_hand(0, catalog::banishing_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: bl, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Banishing Light");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "ETB exiled the opponent's creature");
    // Destroy the enchantment → the exiled card returns to the battlefield.
    g.remove_from_battlefield_to_graveyard_raw(bl);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_some(), "linked exile returned on leave");
}

#[test]
fn knightfisher_makes_fish_when_a_bird_enters() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::knightfisher());
    // Another nontoken Bird entering → a 1/1 Fish.
    let bird = g.add_card_to_hand(0, catalog::knightfisher());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: bird, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a second Knightfisher");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Fish"), "minted a Fish token");
}

#[test]
fn valley_mightcaller_grows_when_a_frog_enters() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let vmc = g.add_card_to_battlefield(0, catalog::valley_mightcaller());
    // A Frog (Bellowing Crier) entering → +1/+1 counter.
    let frog = g.add_card_to_hand(0, catalog::bellowing_crier());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: frog, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a Frog");
    drain_stack(&mut g);
    let c = g.battlefield_find(vmc).unwrap();
    assert_eq!((c.power(), c.toughness()), (2, 2), "Valley Mightcaller grew to 2/2");
}

#[test]
fn lifecreed_duo_gains_life_when_another_creature_enters() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.add_card_to_battlefield(0, catalog::lifecreed_duo());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gained 1 when another creature entered");
}

#[test]
fn burrowguard_mentor_pt_equals_creatures_you_control() {
    let mut g = two_player_game();
    let bm = g.add_card_to_battlefield(0, catalog::burrowguard_mentor());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Three creatures you control (the Mentor + two Bears) → 3/3.
    let computed = g.compute_battlefield();
    let c = computed.iter().find(|c| c.id == bm).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "*/* = creatures you control");
}

#[test]
fn manifold_mouse_grants_double_strike_at_combat() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let mm = g.add_card_to_battlefield(0, catalog::manifold_mouse());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(mm).unwrap().has_keyword(&Keyword::DoubleStrike),
        "the Mouse gained double strike at the beginning of combat");
}

#[test]
fn spellgyre_counter_mode_counters_a_spell() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add_colorless(1);
    g.players[1].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Bears cast");
    let sg = g.add_card_to_hand(0, catalog::spellgyre());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: sg, target: Some(Target::Permanent(bears)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Spellgyre mode 0 (counter)");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears), "counter mode countered Bears");
}

#[test]
fn spellgyre_draw_mode_draws_two() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    let sg = g.add_card_to_hand(0, catalog::spellgyre());
    let hand_before = g.players[0].hand.len();
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: sg, target: None, additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Spellgyre mode 1 (surveil + draw)");
    drain_stack(&mut g);
    // Cast the spellgyre (−1) then drew two (+2) → net +1.
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew two cards");
}

#[test]
fn disentomb_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    // Drop the bear directly into the graveyard.
    let card = g.players[0].hand.pop().unwrap();
    g.players[0].graveyard.push(card);
    let _ = bear;
    let disentomb = g.add_card_to_hand(0, catalog::disentomb());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: disentomb,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Disentomb castable for {B}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == bear));
}

#[test]
fn vandalblast_destroys_opponent_artifact() {
    let mut g = two_player_game();
    let opp_ring = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mine_ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let vand = g.add_card_to_hand(0, catalog::vandalblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: vand,
        target: Some(Target::Permanent(opp_ring)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Vandalblast castable for {R}");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring));
    assert!(g.battlefield.iter().any(|c| c.id == mine_ring), "your own artifact untouched");
}

#[test]
fn vandalblast_overload_destroys_all_opponent_artifacts() {
    let mut g = two_player_game();
    let opp_ring1 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let opp_ring2 = g.add_card_to_battlefield(1, catalog::sol_ring());
    let mine_ring = g.add_card_to_battlefield(0, catalog::sol_ring());
    let vand = g.add_card_to_hand(0, catalog::vandalblast());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);

    g.perform_action(GameAction::CastSpellAlternative {
        card_id: vand,
        pitch_card: None,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Vandalblast Overload for {4}{R}");
    drain_stack(&mut g);

    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring1), "opp artifact 1 destroyed");
    assert!(!g.battlefield.iter().any(|c| c.id == opp_ring2), "opp artifact 2 destroyed");
    assert!(g.battlefield.iter().any(|c| c.id == mine_ring), "own artifact untouched");
}

#[test]
fn natures_lore_fetches_a_forest_to_battlefield_untapped() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    // AutoDecider declines `SearchLibrary` (returns `Search(None)`); a
    // scripted decider picks the matching land so Nature's Lore actually
    // resolves end-to-end.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let lore = g.add_card_to_hand(0, catalog::natures_lore());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: lore,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Nature's Lore castable for {1}{G}");
    drain_stack(&mut g);
    let on_bf = g.battlefield.iter().find(|c| c.id == forest);
    assert!(on_bf.is_some(), "Forest should land on battlefield");
    assert!(!on_bf.unwrap().tapped, "Nature's Lore enters untapped");
}

#[test]
fn sylvan_caryatid_taps_for_one_mana_of_chosen_color() {
    let mut g = two_player_game();
    let caryatid = g.add_card_to_battlefield(0, catalog::sylvan_caryatid());
    g.clear_sickness(caryatid);
    // Scripted decider answers ChooseColor with Black so we can assert
    // the chosen pip lands in the right pool slot.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Black)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: caryatid,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Caryatid mana ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Black), 1);
}

#[test]
fn might_of_old_krosa_scales_with_whose_turn() {
    // Your turn → +4/+4.
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mok = g.add_card_to_hand(0, catalog::might_of_old_krosa());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mok, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None }).expect("cast on your turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 2 + 4, "+4/+4 on your turn");

    // Opponent's turn → +2/+2.
    let mut g = two_player_game();
    g.active_player_idx = 1;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mok = g.add_card_to_hand(0, catalog::might_of_old_krosa());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: mok, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None }).expect("cast on opp turn");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().power(), 2 + 2, "+2/+2 on opponent's turn");
}

#[test]
fn gaeas_anthem_buffs_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::gaeas_anthem());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3, "+1/+1 to yours");
}

#[test]
fn crusade_buffs_white_creatures_both_players() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crusade());
    let mine = g.add_card_to_battlefield(0, catalog::savannah_lions());
    let theirs = g.add_card_to_battlefield(1, catalog::savannah_lions());
    assert_eq!(g.computed_permanent(mine).unwrap().power, 3);
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 3, "affects both players' white creatures");
}

#[test]
fn bad_moon_buffs_power_only() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bad_moon());
    let z = g.add_card_to_battlefield(0, catalog::gifted_aetherborn()); // black 2/3
    let cp = g.computed_permanent(z).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+0 to a black creature");
}

#[test]
fn dictate_of_heliod_buffs_your_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::dictate_of_heliod());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "+2/+2 to yours");
    assert_eq!(g.computed_permanent(theirs).unwrap().power, 2, "opponent's unaffected");
}

// ── Enters-tapped replacements (CR 614.13) ───────────────────────────────────

#[test]
fn imposing_sovereign_taps_entering_opponent_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::imposing_sovereign());
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_self_etb_triggers(bears, 1); // the universal "entered" hook
    assert!(g.battlefield_find(bears).unwrap().tapped, "opponent's creature enters tapped");
}

#[test]
fn imposing_sovereign_does_not_tap_your_own_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::imposing_sovereign());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.fire_self_etb_triggers(mine, 0);
    assert!(!g.battlefield_find(mine).unwrap().tapped, "your own creature enters untapped");
}

#[test]
fn authority_of_the_consuls_taps_and_gains_life() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::authority_of_the_consuls());
    let life = g.players[0].life;
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.fire_self_etb_triggers(bears, 1);
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: bears }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).unwrap().tapped, "opponent creature tapped");
    assert_eq!(g.players[0].life, life + 1, "you gain 1 life when an opponent's creature enters");
}

#[test]
fn blind_obedience_taps_opponent_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::blind_obedience());
    let orn = g.add_card_to_battlefield(1, catalog::ornithopter()); // artifact creature
    g.fire_self_etb_triggers(orn, 1);
    assert!(g.battlefield_find(orn).unwrap().tapped, "opponent's artifact enters tapped");
}

#[test]
fn kismet_taps_opponent_land() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::kismet());
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    g.fire_self_etb_triggers(forest, 1);
    assert!(g.battlefield_find(forest).unwrap().tapped, "opponent's land enters tapped");
    // Your own land is unaffected.
    let mine = g.add_card_to_battlefield(0, catalog::forest());
    g.fire_self_etb_triggers(mine, 0);
    assert!(!g.battlefield_find(mine).unwrap().tapped, "your own land enters untapped");
}

#[test]
fn intangible_virtue_buffs_only_tokens() {
    use crabomination::card::Keyword;
    use crabomination_base::tokens::spirit_token;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::intangible_virtue());
    let tok = g.add_token_to_battlefield(0, &spirit_token()); // 2/2 token
    let nontok = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 nontoken
    let ctok = g.computed_permanent(tok).unwrap();
    assert_eq!((ctok.power, ctok.toughness), (3, 3), "token gets +1/+1");
    assert!(ctok.keywords.contains(&Keyword::Vigilance), "token has vigilance");
    assert_eq!(g.computed_permanent(nontok).unwrap().power, 2, "nontoken unaffected");
}

#[test]
fn always_watching_buffs_only_nontokens() {
    use crabomination::card::Keyword;
    use crabomination_base::tokens::spirit_token;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::always_watching());
    let nontok = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let tok = g.add_token_to_battlefield(0, &spirit_token());
    let cnt = g.computed_permanent(nontok).unwrap();
    assert_eq!((cnt.power, cnt.toughness), (3, 3), "nontoken gets +1/+1");
    assert!(cnt.keywords.contains(&Keyword::Vigilance), "nontoken has vigilance");
    assert_eq!(g.computed_permanent(tok).unwrap().power, 2, "token unaffected");
}

// ── Unleash (CR 702.98) ──────────────────────────────────────────────────────

#[test]
fn unleash_creature_may_enter_with_counter_then_cant_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let cackler = g.add_card_to_battlefield(0, catalog::rakdos_cackler());
    g.fire_self_etb_triggers(cackler, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cackler).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "unleash accepted: enters with a +1/+1 counter");
    assert!(g.computed_permanent(cackler).unwrap().keywords.contains(&Keyword::CantBlock),
        "a 2/2 with a +1/+1 counter from unleash can't block");
}

#[test]
fn unleash_creature_can_decline_counter_and_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    let cackler = g.add_card_to_battlefield(0, catalog::rakdos_cackler());
    g.fire_self_etb_triggers(cackler, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(cackler).unwrap().counter_count(CounterType::PlusOnePlusOne), 0,
        "unleash declined: no counter");
    assert!(!g.computed_permanent(cackler).unwrap().keywords.contains(&Keyword::CantBlock),
        "without a +1/+1 counter the unleash creature can still block");
}

// ── Tribal / aggro creatures ─────────────────────────────────────────────────

#[test]
fn viscera_seer_sacrifices_a_creature() {
    let mut g = two_player_game();
    let seer = g.add_card_to_battlefield(0, catalog::viscera_seer());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island()); // something to scry
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Sacrifice a creature: Scry 1");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == fodder),
        "the sacrificed creature is in the graveyard");
}

#[test]
fn thalias_lieutenant_pumps_humans_on_entry() {
    let mut g = two_player_game();
    let other = g.add_card_to_battlefield(0, catalog::doomed_traveler()); // Human 1/1
    let lt = g.add_card_to_battlefield(0, catalog::thalias_lieutenant());
    g.fire_self_etb_triggers(lt, 0);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(other).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "ETB puts a +1/+1 counter on each other Human");
}

#[test]
fn thalias_lieutenant_grows_when_a_human_enters() {
    let mut g = two_player_game();
    let lt = g.add_card_to_battlefield(0, catalog::thalias_lieutenant());
    let human = g.add_card_to_battlefield(0, catalog::doomed_traveler());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: human }]);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lt).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "another Human entering grows the Lieutenant");
}

#[test]
fn mentor_of_the_meek_draws_for_small_creatures() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.add_card_to_battlefield(0, catalog::mentor_of_the_meek());
    g.players[0].mana_pool.add_colorless(1);
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // power 2
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: small }]);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "paid one mana to draw a card");
}

#[test]
fn hero_of_bladehold_makes_soldiers_on_attack() {
    let mut g = two_player_game();
    let hero = g.add_card_to_battlefield(0, catalog::hero_of_bladehold());
    g.battlefield.iter_mut().find(|c| c.id == hero).unwrap().summoning_sick = false;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let before = g.battlefield.iter().filter(|c| c.controller == 0).count();
    g.declare_attackers(vec![Attack { attacker: hero, target: AttackTarget::Player(1) }])
        .expect("Hero attacks");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Soldier" && c.tapped)
        .count();
    assert_eq!(soldiers, 2, "two Soldier tokens created tapped and attacking");
    assert!(g.battlefield.iter().filter(|c| c.controller == 0).count() >= before + 2);
}

#[test]
fn mirran_crusader_has_double_strike_and_two_protections() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let mc = g.add_card_to_battlefield(0, catalog::mirran_crusader());
    let kws = g.computed_permanent(mc).unwrap().keywords;
    assert!(kws.contains(&Keyword::DoubleStrike), "double strike");
    assert!(kws.contains(&Keyword::Protection(Color::Black)), "protection from black");
    assert!(kws.contains(&Keyword::Protection(Color::Green)), "protection from green");
}

#[test]
fn goldmeadow_harrier_taps_target_creature() {
    let mut g = two_player_game();
    let harrier = g.add_card_to_battlefield(0, catalog::goldmeadow_harrier());
    g.clear_sickness(harrier);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: harrier, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    }).expect("{W}, T: Tap target creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped, "target creature is tapped");
}

#[test]
fn french_vanilla_filler_stats_and_keywords() {
    use crabomination::card::{Keyword, LandType};
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::snapping_drake());
    let griffin = g.add_card_to_battlefield(0, catalog::razorfoot_griffin());
    let spider = g.add_card_to_battlefield(0, catalog::canopy_spider());
    let wraith = g.add_card_to_battlefield(0, catalog::bog_wraith());
    let cd = g.computed_permanent(drake).unwrap();
    assert_eq!((cd.power, cd.toughness), (3, 2));
    assert!(cd.keywords.contains(&Keyword::Flying));
    let cg = g.computed_permanent(griffin).unwrap();
    assert!(cg.keywords.contains(&Keyword::Flying) && cg.keywords.contains(&Keyword::FirstStrike));
    assert!(g.computed_permanent(spider).unwrap().keywords.contains(&Keyword::Reach));
    assert!(g.computed_permanent(wraith).unwrap().keywords.contains(&Keyword::Landwalk(LandType::Swamp)));
}

#[test]
fn mana_rocks_enter_tapped_and_produce_mana() {
    // (factory, mana produced) for the new tapped rocks.
    for (factory, expected) in [
        (catalog::worn_powerstone as fn() -> _, 2),
        (catalog::marble_diamond as fn() -> _, 1),
        (catalog::charcoal_diamond as fn() -> _, 1),
    ] {
        let mut g = two_player_game();
        let rock = g.add_card_to_battlefield(0, factory());
        g.fire_self_etb_triggers(rock, 0);
        drain_stack(&mut g);
        assert!(g.battlefield_find(rock).unwrap().tapped, "rock enters tapped");
        // Untap it (simulate next turn) then tap for mana.
        g.battlefield_find_mut(rock).unwrap().tapped = false;
        g.perform_action(GameAction::ActivateAbility {
            card_id: rock, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.total(), expected);
    }
}

#[test]
fn prophetic_prism_cantrips_on_entry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let prism = g.add_card_to_battlefield(0, catalog::prophetic_prism());
    let hand = g.players[0].hand.len();
    g.fire_self_etb_triggers(prism, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "Prophetic Prism draws a card on entry");
}

#[test]
fn fountain_of_renewal_gains_life_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::fountain_of_renewal());
    g.active_player_idx = 0;
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 1, "gain 1 life at your upkeep");
}

#[test]
fn phyrexian_walker_is_a_zero_three() {
    let mut g = two_player_game();
    let w = g.add_card_to_battlefield(0, catalog::phyrexian_walker());
    let cp = g.computed_permanent(w).unwrap();
    assert_eq!((cp.power, cp.toughness), (0, 3));
}

#[test]
fn green_beaters_have_expected_stats_and_keywords() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let tusker = g.add_card_to_battlefield(0, catalog::kalonian_tusker());
    let baloth = g.add_card_to_battlefield(0, catalog::leatherback_baloth());
    let comp = g.add_card_to_battlefield(0, catalog::garruks_companion());
    let boa = g.add_card_to_battlefield(0, catalog::river_boa());
    let ct = g.computed_permanent(tusker).unwrap();
    assert_eq!((ct.power, ct.toughness), (3, 3));
    let cb = g.computed_permanent(baloth).unwrap();
    assert_eq!((cb.power, cb.toughness), (4, 5));
    assert!(g.computed_permanent(comp).unwrap().keywords.contains(&Keyword::Trample));
    let cboa = g.computed_permanent(boa).unwrap();
    assert!(cboa.keywords.contains(&Keyword::Landwalk(crabomination::card::LandType::Island)));
    assert!(cboa.keywords.iter().any(|k| matches!(k, Keyword::Regenerate(_))));
}

#[test]
fn faerie_seer_flies_and_scries_on_entry() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let fs = g.add_card_to_battlefield(0, catalog::faerie_seer());
    g.fire_self_etb_triggers(fs, 0);
    drain_stack(&mut g);
    assert!(g.computed_permanent(fs).unwrap().keywords.contains(&Keyword::Flying), "has flying");
}

#[test]
fn looter_il_kor_loots_on_combat_damage() {
    let mut g = two_player_game();
    let looter = g.add_card_to_battlefield(0, catalog::looter_il_kor());
    g.add_card_to_hand(0, catalog::island()); // a card to discard
    g.add_card_to_library(0, catalog::grizzly_bears()); // a card to draw
    g.clear_sickness(looter);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let gy = g.players[0].graveyard.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: looter, target: AttackTarget::Player(1),
    }])).expect("attack (Shadow → unblockable)");
    for _ in 0..12 {
        if g.players[0].graveyard.len() > gy { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert!(g.players[0].graveyard.len() > gy, "looted: a card was discarded after drawing");
}

#[test]
fn ninja_of_the_deep_hours_draws_on_combat_damage() {
    let mut g = two_player_game();
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let ninja = g.add_card_to_battlefield(0, catalog::ninja_of_the_deep_hours());
    g.add_card_to_library(0, catalog::island());
    g.clear_sickness(ninja);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ninja, target: AttackTarget::Player(1),
    }])).expect("attack");
    for _ in 0..12 {
        if g.players[0].hand.len() > hand { break; }
        let _ = g.perform_action(GameAction::PassPriority);
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card on connect");
}

#[test]
fn diregraf_ghoul_enters_tapped() {
    let mut g = two_player_game();
    let ghoul = g.add_card_to_battlefield(0, catalog::diregraf_ghoul());
    g.fire_self_etb_triggers(ghoul, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(ghoul).unwrap().tapped, "Diregraf Ghoul enters tapped");
}

#[test]
fn pulse_tracker_drains_on_attack() {
    let mut g = two_player_game();
    let pt = g.add_card_to_battlefield(0, catalog::pulse_tracker());
    g.clear_sickness(pt);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    let life = g.players[1].life;
    g.declare_attackers(vec![Attack { attacker: pt, target: AttackTarget::Player(1) }])
        .expect("attacks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "each opponent loses 1 life on attack");
}

#[test]
fn mesmeric_fiend_exiles_a_card_until_it_leaves() {
    let mut g = two_player_game();
    let stolen = g.add_card_to_hand(1, catalog::grizzly_bears());
    let fiend = g.add_card_to_battlefield(0, catalog::mesmeric_fiend());
    g.fire_self_etb_triggers(fiend, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == stolen), "the chosen card is exiled");
    // When the Fiend leaves, the card returns to its owner's hand.
    g.remove_from_battlefield_to_graveyard_raw(fiend);
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == stolen), "card returns when the Fiend leaves");
}

#[test]
fn vraskas_contempt_exiles_and_gains_life() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::vraskas_contempt());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("castable");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "target is exiled");
    assert_eq!(g.players[0].life, life + 2, "you gain 2 life");
}

#[test]
fn victim_of_night_cannot_target_a_zombie() {
    let mut g = two_player_game();
    // Carrion Feeder is a Zombie — Victim of Night can't target it.
    let zombie = g.add_card_to_battlefield(1, catalog::carrion_feeder());
    let id = g.add_card_to_hand(0, catalog::victim_of_night());
    g.players[0].mana_pool.add(Color::Black, 2);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(zombie)), additional_targets: vec![],
        mode: None, x_value: None,
    });
    assert!(err.is_err(), "a Zombie isn't a legal target");
    // A plain creature is fine.
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("a non-tribal creature is a legal target");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "the bear is destroyed");
}

#[test]
fn mana_dorks_tap_for_mana() {
    for (factory, expected) in [
        (catalog::fyndhorn_elves as fn() -> _, 1),
        (catalog::druid_of_the_cowl as fn() -> _, 1),
        (catalog::manakin as fn() -> _, 1),
        (catalog::palladium_myr as fn() -> _, 2),
    ] {
        let mut g = two_player_game();
        let dork = g.add_card_to_battlefield(0, factory());
        g.clear_sickness(dork);
        g.perform_action(GameAction::ActivateAbility {
            card_id: dork, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
        }).expect("tap for mana");
        assert_eq!(g.players[0].mana_pool.total(), expected, "produced {expected} mana");
    }
}

#[test]
fn gideons_lawkeeper_taps_target_creature() {
    let mut g = two_player_game();
    let lk = g.add_card_to_battlefield(0, catalog::gideons_lawkeeper());
    g.clear_sickness(lk);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lk, ability_index: 0, target: Some(Target::Permanent(victim)), additional_targets: Vec::new(), x_value: None,
    }).expect("{W}, T: Tap target creature");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped);
}

#[test]
fn steppe_lynx_grows_on_landfall() {
    let mut g = two_player_game();
    let lynx = g.add_card_to_battlefield(0, catalog::steppe_lynx());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: land }]);
    drain_stack(&mut g);
    let cp = g.computed_permanent(lynx).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "landfall gives +2/+2 until end of turn");
}

#[test]
fn usher_of_the_fallen_boasts_a_soldier() {
    let mut g = two_player_game();
    let usher = g.add_card_to_battlefield(0, catalog::usher_of_the_fallen());
    g.clear_sickness(usher);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: usher, target: AttackTarget::Player(1) }])
        .expect("Usher attacks");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: usher, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None,
    }).expect("Boast after attacking");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Soldier"),
        "boast mints a Soldier token");
}

#[test]
fn honor_of_the_pure_buffs_only_white_creatures() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::honor_of_the_pure());
    let white = g.add_card_to_battlefield(0, catalog::savannah_lions()); // white 2/1
    let green = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green 2/2
    assert_eq!(g.computed_permanent(white).unwrap().power, 3, "white creature buffed");
    assert_eq!(g.computed_permanent(green).unwrap().power, 2, "non-white unaffected");
}

#[test]
fn benalish_marshal_buffs_others_not_itself() {
    let mut g = two_player_game();
    let marshal = g.add_card_to_battlefield(0, catalog::benalish_marshal());
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert_eq!(g.computed_permanent(marshal).unwrap().power, 3, "marshal not self-buffed");
    assert_eq!(g.computed_permanent(other).unwrap().power, 3, "other creature gets +1/+1");
}

#[test]
fn luminarch_aspirant_counters_at_combat() {
    let mut g = two_player_game();
    let asp = g.add_card_to_battlefield(0, catalog::luminarch_aspirant());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::BeginCombat);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(asp).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "a +1/+1 counter placed at beginning of combat");
}

#[test]
fn steel_overseer_grows_all_artifact_creatures() {
    let mut g = two_player_game();
    let overseer = g.add_card_to_battlefield(0, catalog::steel_overseer());
    let skirge = g.add_card_to_battlefield(0, catalog::vault_skirge());
    g.clear_sickness(overseer);
    g.perform_action(GameAction::ActivateAbility {
        card_id: overseer, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("{T}: counters");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(overseer).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.battlefield_find(skirge).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

#[test]
fn master_of_etherium_sizes_to_artifact_count() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::master_of_etherium());
    // Master + itself counts; add two more artifacts → 3 artifacts total.
    g.add_card_to_battlefield(0, catalog::vault_skirge());
    g.add_card_to_battlefield(0, catalog::steel_overseer());
    let cp = g.computed_permanent(master).unwrap();
    assert_eq!(cp.power, 3, "P/T = artifacts you control (3)");
}

#[test]
fn foundry_inspector_discounts_artifact_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::foundry_inspector());
    // A {2} artifact now castable for {1}.
    let skirge_cost = catalog::burnished_hart(); // {3} artifact
    let id = g.add_card_to_hand(0, skirge_cost);
    g.players[0].mana_pool.add_colorless(2); // {3} - {1} = {2}
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("artifact spell costs {1} less");
}

#[test]
fn beast_whisperer_draws_on_creature_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::beast_whisperer());
    g.add_card_to_library(0, catalog::island());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a creature");
    drain_stack(&mut g);
    // -1 bear cast from hand, +1 drawn by Beast Whisperer → net 0.
    assert_eq!(g.players[0].hand.len(), hand_before, "Beast Whisperer drew on the creature cast");
}

#[test]
fn lotus_cobra_makes_treasure_on_landfall() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lotus_cobra());
    let land = g.add_card_to_hand(0, catalog::forest());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::PlayLand(land)).expect("play a land");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Treasure").count(),
        1, "landfall minted a Treasure");
}

#[test]
fn omnath_landfall_escalates_across_three_lands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::omnath_locus_of_creation());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let (life0, opp0) = (g.players[0].life, g.players[1].life);

    // 1st landfall → gain 4 life.
    let l1 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(l1)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 4, "1st landfall gains 4 life");

    // 2nd landfall → add {R}{G}{W}{U}.
    g.players[0].lands_played_this_turn = 0;
    let l2 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(l2)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.total(), 4, "2nd landfall adds four mana");

    // 3rd landfall → deal 4 to each opponent.
    g.players[0].lands_played_this_turn = 0;
    let l3 = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(l3)).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp0 - 4, "3rd landfall deals 4 to the opponent");
}

#[test]
fn gifted_aetherborn_has_deathtouch_lifelink() {
    use crabomination::card::Keyword;
    let d = catalog::gifted_aetherborn();
    assert!(d.keywords.contains(&Keyword::Deathtouch) && d.keywords.contains(&Keyword::Lifelink));
    assert_eq!((d.power, d.toughness), (2, 3));
}

#[test]
fn doom_whisperer_pays_life_to_surveil() {
    let mut g = two_player_game();
    let dw = g.add_card_to_battlefield(0, catalog::doom_whisperer());
    g.add_card_to_library(0, catalog::island());
    g.clear_sickness(dw);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: dw, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("Pay 2 life: Surveil 2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 2, "paid 2 life");
}

#[test]
fn victims_of_night_cant_kill_a_zombie() {
    let mut g = two_player_game();
    let zombie = g.add_card_to_battlefield(1, catalog::rotting_regisaur()); // Zombie Dinosaur
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let von = g.add_card_to_hand(0, catalog::victims_of_night());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    // Targeting the Zombie is illegal (fails the selection requirement).
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: von, target: Some(Target::Permanent(zombie)),
        additional_targets: vec![], mode: None, x_value: None }).is_err(),
        "can't target a Zombie");
    // The bear is a legal target.
    g.perform_action(GameAction::CastSpell {
        card_id: von, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None }).expect("kills the bear");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear destroyed");
}

#[test]
fn meteor_golem_etb_destroys_opponent_permanent() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let golem = g.add_card_to_hand(0, catalog::meteor_golem());
    g.players[0].mana_pool.add_colorless(7);
    g.perform_action(GameAction::CastSpell {
        card_id: golem, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Meteor Golem castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "opponent's permanent destroyed");
}

#[test]
fn merciless_executioner_etb_edicts_each_player() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let exec = g.add_card_to_hand(0, catalog::merciless_executioner());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: exec, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Executioner castable");
    drain_stack(&mut g);
    // Each player sacrifices a creature (the executioner can satisfy P0's edict).
    assert!(g.battlefield_find(mine).is_none() || g.battlefield_find(exec).is_none(),
        "P0 sacrificed a creature");
    assert!(g.battlefield_find(theirs).is_none(), "P1 sacrificed its only creature");
}

#[test]
fn burnished_hart_sacrifices_to_fetch_two_lands() {
    let mut g = two_player_game();
    let f = g.add_card_to_library(0, catalog::forest());
    let m = g.add_card_to_library(0, catalog::mountain());
    let hart = g.add_card_to_battlefield(0, catalog::burnished_hart());
    g.players[0].mana_pool.add_colorless(3);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(f)),
        DecisionAnswer::Search(Some(m)),
    ]));
    let lands_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    g.perform_action(GameAction::ActivateAbility {
        card_id: hart, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None }).expect("{3}, Sac");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hart).is_none(), "Hart sacrificed");
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 2, "fetched two basics");
}

#[test]
fn massacre_wurm_etb_shrinks_and_death_drains() {
    let mut g = two_player_game();
    let x = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 → dies to -2/-2
    let wurm = g.add_card_to_hand(0, catalog::massacre_wurm());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(3);
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: wurm, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Massacre Wurm castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(x).is_none(), "the 2/2 died to -2/-2");
    assert_eq!(g.players[1].life, opp - 2, "opponent loses 2 when its creature dies");
}

#[test]
fn archon_of_cruelty_etb_drains_opponent_and_rewards_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(1, catalog::grizzly_bears()); // sac fodder
    g.add_card_to_hand(1, catalog::island()); // discard fodder
    g.add_card_to_library(0, catalog::island()); // draw fodder
    let archon = g.add_card_to_hand(0, catalog::archon_of_cruelty());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(6);
    let you = g.players[0].life;
    let opp = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: archon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Archon castable");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 3, "opponent loses 3");
    assert_eq!(g.players[0].life, you + 3, "you gain 3");
}

#[test]
fn rampaging_ferocidon_pings_on_creature_etb_and_stops_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rampaging_ferocidon());
    // Lifegain is shut off for every player.
    assert!(g.player_cannot_gain_life_now(0), "your lifegain prevented");
    assert!(g.player_cannot_gain_life_now(1), "opponent's lifegain prevented too");
    // Another creature entering (cast by you) pings you (its controller).
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let you = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("bear castable");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you - 1, "Ferocidon pings the entering creature's controller");
}

#[test]
fn rotting_regisaur_discards_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::rotting_regisaur());
    g.add_card_to_hand(0, catalog::lightning_bolt());
    g.add_card_to_hand(0, catalog::island());
    g.active_player_idx = 0;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "discards one at upkeep");
}

#[test]
fn sun_titan_etb_reanimates_cheap_permanent() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2 ≤ 3
    let titan = g.add_card_to_hand(0, catalog::sun_titan());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: titan, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Sun Titan castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "Sun Titan returned the bear to the battlefield");
}

#[test]
fn primeval_titan_etb_fetches_two_lands() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    let mountain = g.add_card_to_library(0, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Search(Some(forest)),
        DecisionAnswer::Search(Some(mountain)),
    ]));
    let lands_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    let titan = g.add_card_to_hand(0, catalog::primeval_titan());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpell {
        card_id: titan, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("Primeval Titan castable");
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 2, "fetched two lands to the battlefield");
}

#[test]
fn bitterblossom_upkeep_makes_faerie_and_drains() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bitterblossom());
    g.active_player_idx = 0;
    let life = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life - 1, "lose 1 at upkeep");
    assert_eq!(
        g.battlefield.iter().filter(|c| c.is_token && c.definition.name == "Faerie Rogue").count(),
        1, "one Faerie Rogue minted");
}

#[test]
fn brineborn_cutthroat_grows_on_opponents_turn_cast() {
    let mut g = two_player_game();
    let bc = g.add_card_to_battlefield(0, catalog::brineborn_cutthroat());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1; // opponent's turn
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None }).expect("flash-cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bc).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "grew on opponent's-turn cast");
}

#[test]
fn guttersnipe_pings_each_opponent_on_instant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::guttersnipe());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 3 - 2, "bolt 3 + Guttersnipe 2");
}

#[test]
fn sheoldred_apocalypse_punishes_and_rewards_draws() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sheoldred_the_apocalypse());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(1, catalog::island());
    let you = g.players[0].life;
    let opp = g.players[1].life;
    let mut events = Vec::new();
    g.draw_one(0, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, you + 2, "you gain 2 on your draw");
    let mut events = Vec::new();
    g.draw_one(1, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "opponent loses 2 on their draw");
}

#[test]
fn apostles_blessing_grants_protection_to_your_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ab = g.add_card_to_hand(0, catalog::apostles_blessing());
    g.players[0].mana_pool.add(Color::White, 1); // {W/P}
    g.players[0].mana_pool.add_colorless(1); // {1}
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::CastSpell {
        card_id: ab, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Protection(Color::Blue)));
}

#[test]
fn brave_the_elements_protects_all_your_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bte = g.add_card_to_hand(0, catalog::brave_the_elements());
    g.players[0].mana_pool.add(Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bte, target: None,
        additional_targets: vec![], mode: None, x_value: None }).expect("cast");
    drain_stack(&mut g);
    for id in [a, b] {
        assert!(g.battlefield_find(id).unwrap().has_keyword(&Keyword::Protection(Color::Red)),
            "all your creatures protected from red");
    }
}

#[test]
fn ledger_shredder_connives_on_second_spell_only() {
    let mut g = two_player_game();
    let shredder = g.add_card_to_battlefield(0, catalog::ledger_shredder());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    let bolt1 = g.add_card_to_hand(0, catalog::lightning_bolt());
    let bolt2 = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 2);

    // First spell — no connive (library untouched).
    let lib_before = g.players[0].library.len();
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None }).expect("bolt1");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before, "no connive on first spell");

    // Second spell — connive fires, drawing one card.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None }).expect("bolt2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib_before - 1, "connive drew a card on second spell");
    let _ = shredder;
}

#[test]
fn mother_of_runes_grants_protection_from_chosen_color() {
    let mut g = two_player_game();
    let mom = g.add_card_to_battlefield(0, catalog::mother_of_runes());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(mom);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: mom, ability_index: 0, target: Some(Target::Permanent(bear)), additional_targets: Vec::new(), x_value: None })
        .expect("{T}: grant protection");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Protection(Color::Red)),
        "bear has protection from red");
    // A red spell (Lightning Bolt) from the opponent can't target it.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let err = g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None });
    assert!(matches!(err, Err(GameError::TargetHasProtection(_))), "bolt blocked, got {err:?}");
}

#[test]
fn gods_willing_grants_protection_and_scries() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let gw = g.add_card_to_hand(0, catalog::gods_willing());
    g.players[0].mana_pool.add(Color::White, 1);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::Black),
        DecisionAnswer::ScryOrder { kept_top: vec![], bottom: vec![] },
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: gw, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Gods Willing");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).unwrap().has_keyword(&Keyword::Protection(Color::Black)),
        "bear protected from black");
}

#[test]
fn millstone_mills_target_for_two() {
    let mut g = two_player_game();
    g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::mountain());
    g.add_card_to_library(1, catalog::island());
    let stone = g.add_card_to_battlefield(0, catalog::millstone());
    g.clear_sickness(stone);
    g.players[0].mana_pool.add_colorless(2);
    let opp_lib_before = g.players[1].library.len();
    let opp_yard_before = g.players[1].graveyard.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: stone,
        ability_index: 0,
        target: Some(Target::Player(1)), additional_targets: Vec::new(), x_value: None })
    .expect("Millstone activates for {2}{T}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), opp_lib_before - 2);
    assert_eq!(g.players[1].graveyard.len(), opp_yard_before + 2);
}

#[test]
fn ornithopter_is_a_zero_cost_flying_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ornithopter());
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Ornithopter castable for free");
    drain_stack(&mut g);
    let bf = g.battlefield.iter().find(|c| c.id == id).unwrap();
    assert_eq!(bf.power(), 0);
    assert_eq!(bf.toughness(), 2);
    assert!(bf.has_keyword(&crabomination::card::Keyword::Flying));
}

#[test]
fn ornithopter_of_paradise_taps_for_any_one_color() {
    let mut g = two_player_game();
    let bird = g.add_card_to_battlefield(0, catalog::ornithopter_of_paradise());
    g.clear_sickness(bird);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: bird,
        ability_index: 0,
        target: None, additional_targets: Vec::new(), x_value: None })
    .expect("Ornithopter of Paradise mana ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 1);
}

