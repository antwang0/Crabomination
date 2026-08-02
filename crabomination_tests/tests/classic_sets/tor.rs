//! Torment (TOR) — the Cephalid self-mill shell, Madness and Threshold.

use crabomination::card::CounterType;
use crabomination::catalog;
use crabomination::game::types::{GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn mana(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn cast(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn activate(g: &mut GameState, seat: usize, card_id: CardId, index: usize, target: Option<Target>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::ActivateAbility {
        card_id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("activate");
    drain_stack(g);
}

/// Seven cards in the graveyard turns Threshold on.
fn fill_graveyard(g: &mut GameState, seat: usize) {
    for _ in 0..7 {
        g.add_card_to_graveyard(seat, catalog::forest());
    }
}

/// Aquamoeba flips its stats for a card.
#[test]
fn aquamoeba_switches_power_and_toughness() {
    let mut g = main_phase();
    let moeba = g.add_card_to_battlefield(0, catalog::aquamoeba());
    g.add_card_to_hand(0, catalog::forest());
    let cp = g.computed_permanent(moeba).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 3));
    activate(&mut g, 0, moeba, 0, None);
    let cp = g.computed_permanent(moeba).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 1));
}

/// Pointing anything at a Cephalid fills its controller's graveyard.
#[test]
fn cephalid_illusionist_mills_on_being_targeted() {
    let mut g = main_phase();
    let ceph = g.add_card_to_battlefield(0, catalog::cephalid_illusionist());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, 0, pump, Some(Target::Permanent(ceph)));
    assert_eq!(g.players[0].graveyard.len(), 4, "three milled plus the spell");
}

/// Cephalid Vandal mills one more each upkeep.
#[test]
fn cephalid_vandal_accelerates_each_upkeep() {
    let mut g = main_phase();
    let vandal = g.add_card_to_battlefield(0, catalog::cephalid_vandal());
    for _ in 0..10 {
        g.add_card_to_library(0, catalog::forest());
    }
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 1);
    assert_eq!(g.battlefield_find(vandal).unwrap().counter_count(CounterType::Shred), 1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "two more on the second tick");
}

/// Cephalid Sage draws three past Threshold and nothing before it.
#[test]
fn cephalid_sage_draws_past_threshold() {
    let mut g = main_phase();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    fill_graveyard(&mut g, 0);
    let sage = g.add_card_to_hand(0, catalog::cephalid_sage());
    let before = g.players[0].hand.len();
    cast(&mut g, 0, sage, None);
    // -1 for the cast, +3 drawn, -2 discarded.
    assert_eq!(g.players[0].hand.len(), before - 1 + 3 - 2);
}

/// Boneshard Slasher swells but becomes brittle past Threshold.
#[test]
fn boneshard_slasher_grows_and_gets_brittle() {
    let mut g = main_phase();
    let slasher = g.add_card_to_battlefield(0, catalog::boneshard_slasher());
    let cp = g.computed_permanent(slasher).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(slasher).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    cast(&mut g, 0, pump, Some(Target::Permanent(slasher)));
    assert!(g.battlefield_find(slasher).is_none(), "targeting it kills it past Threshold");
}

/// Cabal Torturer's bigger shrink needs Threshold.
#[test]
fn cabal_torturer_second_ability_needs_threshold() {
    let mut g = main_phase();
    let torturer = g.add_card_to_battlefield(0, catalog::cabal_torturer());
    g.battlefield_find_mut(torturer).unwrap().summoning_sick = false;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: torturer,
            ability_index: 1,
            target: Some(Target::Permanent(bear)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the -2/-2 is Threshold-gated"
    );
    fill_graveyard(&mut g, 0);
    activate(&mut g, 0, torturer, 1, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 killed the 2/2");
}

/// Circular Logic taxes the countered spell by your whole graveyard.
#[test]
fn circular_logic_taxes_by_your_graveyard() {
    let mut g = main_phase();
    fill_graveyard(&mut g, 0);
    let victim = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(1, catalog::giant_growth());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the pump");
    let logic = g.add_card_to_hand(0, catalog::circular_logic());
    // Seat 1 is tapped out, so the {7} tax can't be paid.
    g.players[1].mana_pool = Default::default();
    cast(&mut g, 0, logic, Some(Target::Permanent(pump)));
    assert!(
        g.players[1].graveyard.iter().any(|c| c.id == pump),
        "countered — they could not pay the tax"
    );
}

/// Ambassador Laquatus mills three for {3}.
#[test]
fn ambassador_laquatus_mills_three() {
    let mut g = main_phase();
    let laq = g.add_card_to_battlefield(0, catalog::ambassador_laquatus());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::forest());
    }
    activate(&mut g, 0, laq, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].graveyard.len(), 3);
}

/// Chainer reanimates a corpse as a Nightmare under his control.
#[test]
fn chainer_reanimates_as_a_nightmare() {
    let mut g = main_phase();
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_dementia_master());
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, 0, chainer, 0, Some(Target::Permanent(corpse)));
    let cp = g.computed_permanent(corpse).expect("reanimated");
    assert_eq!(cp.controller, 0);
    assert!(cp.subtypes.creature_types.contains(&crabomination::card::CreatureType::Nightmare));
}

/// Coral Net taxes its host a card every upkeep.
#[test]
fn coral_net_taxes_a_card_each_upkeep() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let net = g.add_card_to_hand(0, catalog::coral_net());
    cast(&mut g, 0, net, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(net).unwrap().attached_to, Some(bear));
    // With an empty hand the host can't pay, so it dies.
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "nothing to discard");
}

/// Compulsion loots for {1}{U}.
#[test]
fn compulsion_loots() {
    let mut g = main_phase();
    let comp = g.add_card_to_battlefield(0, catalog::compulsion());
    g.add_card_to_hand(0, catalog::forest());
    g.add_card_to_library(0, catalog::grizzly_bears());
    activate(&mut g, 0, comp, 0, None);
    assert_eq!(g.players[0].graveyard.len(), 1, "the discard");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Acorn Harvest makes two Squirrels, and again from the graveyard.
#[test]
fn acorn_harvest_makes_squirrels_twice() {
    let mut g = main_phase();
    let harvest = g.add_card_to_hand(0, catalog::acorn_harvest());
    cast(&mut g, 0, harvest, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 2);
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: harvest,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flashback");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Squirrel").count(), 4);
}

/// Churning Eddy bounces a creature and a land.
#[test]
fn churning_eddy_bounces_both() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let eddy = g.add_card_to_hand(0, catalog::churning_eddy());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: eddy,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![Target::Permanent(land)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none() && g.battlefield_find(land).is_none());
}

/// CR 611.2b — Chainer's permanent Nightmare stamp outlives him, so his own
/// leaves-the-battlefield trigger exiles what he reanimated.
#[test]
fn chainer_exiles_his_nightmares_when_he_dies() {
    let mut g = main_phase();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let chainer = g.add_card_to_battlefield(0, catalog::chainer_dementia_master());
    activate(&mut g, 0, chainer, 0, Some(Target::Permanent(corpse)));
    assert!(g.battlefield_find(corpse).is_some(), "reanimated");
    let mut events = Vec::new();
    g.destroy_permanent(chainer, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(corpse).is_none(), "the Nightmare was exiled");
    assert!(g.exile.iter().any(|c| c.id == corpse));
}

/// Anurid Scavenger bottoms a graveyard card each upkeep, and dies without one.
#[test]
fn anurid_scavenger_eats_its_graveyard() {
    let mut g = main_phase();
    let frog = g.add_card_to_battlefield(0, catalog::anurid_scavenger());
    g.add_card_to_graveyard(0, catalog::forest());
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(frog).is_some(), "paid with the graveyard card");
    assert!(g.players[0].graveyard.is_empty(), "the card went to the library bottom");
}

/// Crazed Firecat grows by however many flips it won.
#[test]
fn crazed_firecat_counts_its_flips() {
    let mut g = main_phase();
    let cat = g.add_card_to_battlefield(0, catalog::crazed_firecat());
    g.fire_self_etb_triggers(cat, 0);
    drain_stack(&mut g);
    let cp = g.computed_permanent(cat).unwrap();
    let counters = g.battlefield_find(cat).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert_eq!(cp.power, 4 + counters as i32, "power tracks the won flips");
}

/// Faceless Butcher jails a creature and gives it back when he leaves.
#[test]
fn faceless_butcher_jails_and_releases() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let butcher = g.add_card_to_battlefield(0, catalog::faceless_butcher());
    g.fire_self_etb_triggers(butcher, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "jailed");
    let mut events = Vec::new();
    g.destroy_permanent(butcher, false, &mut events);
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_some(), "released");
}

/// Gravegouger holds graveyard cards hostage and returns them to the graveyard.
#[test]
fn gravegouger_holds_graveyard_cards() {
    let mut g = main_phase();
    let a = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let gouger = g.add_card_to_battlefield(0, catalog::gravegouger());
    g.fire_self_etb_triggers(gouger, 0);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == a), "exiled from the graveyard: {:?}", g.exile.len());
    let mut events = Vec::new();
    g.destroy_permanent(gouger, false, &mut events);
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == a), "back in the graveyard");
}

/// Far Wanderings fetches one basic, or three past Threshold.
#[test]
fn far_wanderings_scales_with_threshold() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let picks: Vec<CardId> = (0..4).map(|_| g.add_card_to_library(0, catalog::forest())).collect();
    fill_graveyard(&mut g, 0);
    let spell = g.add_card_to_hand(0, catalog::far_wanderings());
    let lands = g.battlefield.iter().filter(|c| c.definition.is_land()).count();
    g.decider = Box::new(ScriptedDecider::new(
        picks.iter().map(|id| DecisionAnswer::Search(Some(*id))).collect::<Vec<_>>(),
    ));
    cast(&mut g, 0, spell, None);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.is_land()).count(),
        lands + 3,
        "Threshold fetched three"
    );
}

/// Flash of Defiance stops green and white blockers for the turn.
#[test]
fn flash_of_defiance_stops_green_blockers() {
    let mut g = main_phase();
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::flash_of_defiance());
    cast(&mut g, 0, spell, None);
    assert!(
        g.computed_permanent(blocker).unwrap().keywords.contains(&Keyword::CantBlock),
        "the green bear can't block"
    );
}

/// Hydromorph Guardian counters a spell aimed at your own creature.
#[test]
fn hydromorph_guardian_counters_targeted_removal() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let guardian = g.add_card_to_battlefield(0, catalog::hydromorph_guardian());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    activate(&mut g, 0, guardian, 0, Some(Target::Permanent(bolt)));
    assert!(g.battlefield_find(mine).is_some(), "the Bolt was countered");
}

/// Crippling Fatigue shrinks a creature, and flashes back for 3 life.
#[test]
fn crippling_fatigue_flashback_costs_life() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_graveyard(0, catalog::crippling_fatigue());
    mana(&mut g, 0);
    let life = g.players[0].life;
    g.perform_action(GameAction::CastFlashback {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("flash back Crippling Fatigue");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "the 2/2 died to -2/-2");
    assert_eq!(g.players[0].life, life - 3, "paid the flashback life");
}

/// Ghostly Wings buys back its host for a card.
#[test]
fn ghostly_wings_bounces_its_host() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wings = g.add_card_to_battlefield(0, catalog::ghostly_wings());
    g.battlefield.iter_mut().find(|c| c.id == wings).unwrap().attached_to = Some(bear);
    g.add_card_to_hand(0, catalog::forest());
    activate(&mut g, 0, wings, 0, None);
    assert!(g.battlefield_find(bear).is_none(), "the host was bounced");
    assert!(g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Overmaster makes your next instant or sorcery uncounterable, once.
#[test]
fn overmaster_protects_one_spell() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::overmaster());
    cast(&mut g, 0, spell, None);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    let second = g.add_card_to_hand(0, catalog::lightning_bolt());
    assert!(g.caster_grants_uncounterable(0, g.players[0].hand.iter().find(|c| c.id == bolt).unwrap()));
    mana(&mut g, 0);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    drain_stack(&mut g);
    assert!(
        !g.caster_grants_uncounterable(0, g.players[0].hand.iter().find(|c| c.id == second).unwrap()),
        "the grant was one-shot"
    );
}

/// Llawan bounces blue creatures and stops opponents casting more.
#[test]
fn llawan_bounces_and_locks_blue() {
    let mut g = main_phase();
    let merfolk = g.add_card_to_battlefield(1, catalog::cephalid_illusionist());
    let llawan = g.add_card_to_battlefield(0, catalog::llawan_cephalid_empress());
    g.fire_self_etb_triggers(llawan, 0);
    drain_stack(&mut g);
    assert!(g.battlefield_find(merfolk).is_none(), "the blue creature bounced");
    let recast = g.players[1].hand.iter().find(|c| c.id == merfolk).map(|c| c.id).unwrap();
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    assert!(g
        .perform_action(GameAction::CastSpell {
            card_id: recast,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(), "and can't be recast");
}

/// Invigorating Falls counts creature cards in every graveyard.
#[test]
fn invigorating_falls_counts_all_graveyards() {
    let mut g = main_phase();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::invigorating_falls());
    let life = g.players[0].life;
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].life, life + 2, "two creature cards across both graveyards");
}

/// Mind Sludge scales with your Swamps.
#[test]
fn mind_sludge_counts_swamps() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::swamp());
    }
    for _ in 0..5 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::mind_sludge());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 2, "discarded three");
}

/// Liquify exiles the spell it counters.
#[test]
fn liquify_exiles_what_it_counters() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Bolt");
    let liquify = g.add_card_to_hand(0, catalog::liquify());
    cast(&mut g, 0, liquify, Some(Target::Permanent(bolt)));
    assert!(g.exile.iter().any(|c| c.id == bolt), "the Bolt was exiled, not binned");
}

/// Mystic Familiar hardens once Threshold is on.
#[test]
fn mystic_familiar_grows_past_threshold() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::mystic_familiar());
    let cp = g.computed_permanent(bird).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 2));
    fill_graveyard(&mut g, 0);
    let cp = g.computed_permanent(bird).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
    assert!(cp.keywords.contains(&Keyword::Protection(Color::Black)));
}

/// Organ Grinder cashes three graveyard cards for three life.
#[test]
fn organ_grinder_drains_for_graveyard_cards() {
    let mut g = main_phase();
    let grinder = g.add_card_to_battlefield(0, catalog::organ_grinder());
    g.clear_sickness(grinder);
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::forest());
    }
    activate(&mut g, 0, grinder, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 17);
    assert!(g.players[0].graveyard.is_empty(), "the three cards were exiled");
}

/// Mortal Combat wins with twenty creature cards in the graveyard.
#[test]
fn mortal_combat_wins_at_twenty() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mortal_combat());
    for _ in 0..20 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.game_over.is_some(), "the game ended");
}
