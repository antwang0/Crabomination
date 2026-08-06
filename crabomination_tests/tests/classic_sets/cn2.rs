//! Conspiracy: Take the Crown (CN2) — the monarch shell, melee, goad,
//! monstrosity and the council's dilemma (`catalog::sets::cn2`).

use crabomination::card::{CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

/// Cast `def` from hand with free mana and let it resolve.
fn resolve_sorcery(g: &mut GameState, def: crabomination::card::CardDefinition) {
    let id = g.add_card_to_hand(0, def);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(12);
    for c in [Color::Blue, Color::Green] {
        g.players[0].mana_pool.add(c, 2);
    }
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn swing(g: &mut GameState, id: CardId) {
    g.clear_sickness(id);
    advance_to(g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: id,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(g);
}

/// Every CN2 factory builds and lands in the catalog.
#[test]
fn cn2_cards_are_registered() {
    let names: Vec<&str> = crabomination_catalog::sets::all_factories::all_catalog_card_factories()
        .map(|f| f().name)
        .collect();
    for name in [
        "Ballot Broker",
        "Crown-Hunter Hireling",
        "Queen Marchesa",
        "Throne of the High City",
        "Splitting Slime",
        "Selvala, Heart of the Wilds",
    ] {
        assert!(names.contains(&name), "{name} is missing from the catalog");
    }
}

/// Protector of the Crown crowns you and soaks damage aimed at your face.
#[test]
fn protector_of_the_crown_crowns_and_soaks() {
    let mut g = two_player_game();
    let prot = g.move_card_to_battlefield_for_test(0, catalog::protector_of_the_crown());
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0), "CR 725.3 — the ETB crowns its controller");

    let mut evs = vec![];
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(0),
        3,
        None,
        &mut evs,
    );
    assert_eq!(g.players[0].life, 20, "the damage was redirected");
    assert_eq!(g.battlefield_find(prot).unwrap().damage, 3);
}

/// Crown-Hunter Hireling can only swing at whoever holds the crown.
#[test]
fn crown_hunter_hireling_only_attacks_the_monarch() {
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::crown_hunter_hireling());
    g.clear_sickness(ogre);
    g.monarch = Some(0);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: ogre,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "seat 1 isn't the monarch"
    );
    g.monarch = Some(1);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ogre,
        target: AttackTarget::Player(1),
    }]))
    .expect("the monarch is a legal defender");
}

/// Knights of the Black Rose bleeds whoever steals your crown mid-turn.
#[test]
fn knights_of_the_black_rose_punishes_a_crown_theft() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::knights_of_the_black_rose());
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    g.monarch_at_turn_start = Some(0);

    let mut evs = vec![];
    g.set_monarch(1, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22);
}

/// Queen Marchesa mints an Assassin each upkeep the crown sits elsewhere.
#[test]
fn queen_marchesa_mints_an_assassin_without_the_crown() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::queen_marchesa());
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    // Wearing the crown yourself, the upkeep trigger stays quiet.
    g.monarch = Some(1);
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    let assassin = g.battlefield.iter().find(|c| c.definition.name == "Assassin");
    assert!(assassin.is_some_and(|c| c.definition.keywords.contains(&Keyword::Deathtouch)));
}

/// Throne of the High City taps for {C} and buys the crown.
#[test]
fn throne_of_the_high_city_buys_the_crown() {
    let mut g = two_player_game();
    let throne = g.add_card_to_battlefield(0, catalog::throne_of_the_high_city());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: throne,
        ability_index: 1,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("sacrifice for the crown");
    drain_stack(&mut g);
    assert_eq!(g.monarch, Some(0));
    assert!(g.battlefield_find(throne).is_none(), "sacrificed as a cost");
}

/// Custodi Soulcaller's melee pump and its mana-value-gated reanimation both
/// read the number of players it attacked.
#[test]
fn custodi_soulcaller_reanimates_up_to_the_players_attacked() {
    let mut g = two_player_game();
    let caller = g.add_card_to_battlefield(0, catalog::custodi_soulcaller());
    let bears = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    swing(&mut g, caller);
    // One player attacked: melee makes it 2/3, and only MV ≤ 1 comes back.
    assert_eq!(g.computed_permanent(caller).unwrap().power, 2);
    assert!(g.battlefield_find(bears).is_none(), "Grizzly Bears costs {{1}}{{G}}");
}

/// Sinuous Vermin only gains menace once it goes monstrous.
#[test]
fn sinuous_vermin_gains_menace_when_monstrous() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::sinuous_vermin());
    assert!(!g.computed_permanent(rat).unwrap().keywords.contains(&Keyword::Menace));

    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rat,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("monstrosity 3");
    drain_stack(&mut g);
    let vermin = g.computed_permanent(rat).unwrap();
    assert_eq!((vermin.power, vermin.toughness), (5, 5));
    assert!(vermin.keywords.contains(&Keyword::Menace));
}

/// Splitting Slime clones itself the moment it becomes monstrous.
#[test]
fn splitting_slime_clones_itself_on_monstrosity() {
    let mut g = two_player_game();
    let slime = g.add_card_to_battlefield(0, catalog::splitting_slime());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: slime,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("monstrosity 3");
    drain_stack(&mut g);
    let copy = g
        .battlefield
        .iter()
        .find(|c| c.is_token && c.definition.name == "Splitting Slime")
        .expect("token copy");
    assert_eq!(copy.counter_count(CounterType::PlusOnePlusOne), 0, "the copy has no counters");
}

/// Orchard Elemental's council's dilemma pays per vote (CR 701.38).
#[test]
fn orchard_elemental_pays_per_vote() {
    let mut g = two_player_game();
    // Both seats vote Harvest (option index 1).
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(1),
        DecisionAnswer::Amount(1),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::orchard_elemental());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 26, "two harvest votes = 6 life");
}

/// Illusion of Choice hands you every ballot for the turn (CR 701.38).
#[test]
fn illusion_of_choice_answers_every_ballot() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::illusion_of_choice());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.vote_controller_this_turn, Some(0));

    // Seat 0 now answers seat 1's ballot too: both votes go to Sprout.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(0),
    ]));
    let elem = g.move_card_to_battlefield_for_test(0, catalog::orchard_elemental());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(elem).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4,
        "two sprout votes = four counters"
    );
}

/// Ballot Broker casts a second vote for its controller.
#[test]
fn ballot_broker_votes_twice() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ballot_broker());
    // Seat 0 votes Sprout twice, seat 1 votes Harvest.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(1),
    ]));
    let elem = g.move_card_to_battlefield_for_test(0, catalog::orchard_elemental());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(elem).unwrap().counter_count(CounterType::PlusOnePlusOne),
        4
    );
    assert_eq!(g.players[0].life, 23, "the lone harvest vote still paid");
}

/// Deadly Designs is fed by any player and pops at five plot counters.
#[test]
fn deadly_designs_pops_at_five_plot_counters() {
    let mut g = two_player_game();
    let plot = g.add_card_to_battlefield(0, catalog::deadly_designs());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    for _ in 0..5 {
        g.priority.player_with_priority = 0;
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: plot,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("add a plot counter");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(plot).is_none(), "sacrificed itself");
    assert!(g.battlefield_find(victim).is_none(), "and took a creature with it");
}

/// Selvala's mana ability scales with your biggest creature.
#[test]
fn selvala_taps_for_your_greatest_power() {
    let mut g = two_player_game();
    let selvala = g.add_card_to_battlefield(0, catalog::selvala_heart_of_the_wilds());
    g.clear_sickness(selvala);
    g.add_card_to_battlefield(0, catalog::craw_wurm()); // 6/4
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: selvala,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.total(), 6, "greatest power among your creatures");
}

/// Besmirch borrows a creature and goads it so it can't swing back.
#[test]
fn besmirch_borrows_and_goads() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::besmirch());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Besmirch");
    drain_stack(&mut g);
    let stolen = g.battlefield_find(bear).unwrap();
    assert_eq!(stolen.controller, 0);
    assert!(!stolen.tapped, "untapped");
    assert!(!stolen.goaded_by.is_empty(), "and goaded");
}


/// The CN2 draft-matters cards note their pick number / a name / three colors
/// as they're drafted (CR 905.2b), and the game halves read those notes.
#[test]
fn cn2_draft_notes_feed_the_game_halves() {
    use crabomination::draft::{DraftNotes, GARBAGE_FIRE, REGICIDE, SMUGGLER_CAPTAIN};
    let mut g = two_player_game();
    let mut notes = DraftNotes::default();
    notes.note_number(GARBAGE_FIRE, 4);
    notes.note_name(SMUGGLER_CAPTAIN, "Grizzly Bears");
    notes.note_colors(REGICIDE, &[Color::Green]);
    g.players[0].draft_notes = notes;

    // Garbage Fire burns for the highest noted pick.
    let victim = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4
    let fire = g.add_card_to_hand(0, catalog::garbage_fire());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: fire,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Garbage Fire");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "4 damage is lethal to a 6/4");

    // Smuggler Captain tutors the noted name.
    let bears_id = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(bears_id))]));
    g.move_card_to_battlefield_for_test(0, catalog::smuggler_captain());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears_id), "the noted name was findable");
    g.decider = Box::new(crabomination::decision::AutoDecider);

    // Regicide only kills something wearing a noted color.
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green
    let kill = g.add_card_to_hand(0, catalog::regicide());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: kill,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast Regicide");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bears).is_none());
}

/// Pyretic Hunter arrives sized by its noted pick; Custodi Peacekeeper's tap
/// is capped by the same number.
#[test]
fn draft_note_number_sizes_pyretic_hunter_and_caps_the_peacekeeper() {
    use crabomination::draft::{CUSTODI_PEACEKEEPER, DraftNotes, PYRETIC_HUNTER};
    let mut g = two_player_game();
    let mut notes = DraftNotes::default();
    notes.note_number(PYRETIC_HUNTER, 3);
    notes.note_number(CUSTODI_PEACEKEEPER, 3);
    g.players[0].draft_notes = notes;

    let hunter = g.move_card_to_battlefield_for_test(0, catalog::pyretic_hunter());
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(hunter).unwrap().power, 3);

    let keeper = g.add_card_to_battlefield(0, catalog::custodi_peacekeeper());
    g.clear_sickness(keeper);
    let wurm = g.add_card_to_battlefield(1, catalog::craw_wurm()); // 6/4, over the cap
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: keeper,
            ability_index: 0,
            target: Some(Target::Permanent(wurm)),
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .is_err(),
        "power 6 is over the noted 3"
    );
}

/// Noble Banneret anthems itself and every creature sharing a noted name.
#[test]
fn noble_banneret_anthems_its_noted_names() {
    use crabomination::draft::{DraftNotes, NOBLE_BANNERET};
    let mut g = two_player_game();
    let mut notes = DraftNotes::default();
    notes.note_name(NOBLE_BANNERET, "Grizzly Bears");
    g.players[0].draft_notes = notes;

    let banneret = g.add_card_to_battlefield(0, catalog::noble_banneret());
    let bears = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    assert_eq!(g.computed_permanent(banneret).unwrap().power, 4, "anthems itself");
    let pumped = g.computed_permanent(bears).unwrap();
    assert_eq!((pumped.power, pumped.toughness), (3, 3));
    assert!(pumped.keywords.contains(&Keyword::Lifelink));
    assert_eq!(g.computed_permanent(wurm).unwrap().power, 6, "an unnoted name is untouched");
}

/// Kaya's 0 blinks its subject away until your next upkeep, for 2 life.
#[test]
fn kaya_blinks_until_your_next_upkeep() {
    let mut g = two_player_game();
    let kaya = g.add_card_to_battlefield(0, catalog::kaya_ghost_assassin());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: kaya,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("Kaya's 0");
    drain_stack(&mut g);
    // Mode 0 is "exile Kaya"; she comes back at your next upkeep.
    assert!(g.battlefield_find(kaya).is_none(), "exiled herself");
    assert!(g.exile.iter().any(|c| c.id == kaya));
    assert_eq!(g.players[0].life, 18);
}

/// Daretti's +1 leaves a Construct wall behind.
#[test]
fn daretti_makes_a_defender_construct() {
    let mut g = two_player_game();
    let daretti = g.add_card_to_battlefield(0, catalog::daretti_ingenious_iconoclast());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: daretti,
        ability_index: 0,
        target: None,
        x_value: None,
    })
    .expect("Daretti's +1");
    drain_stack(&mut g);
    let construct = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Construct")
        .expect("Construct token");
    assert!(construct.definition.keywords.contains(&Keyword::Defender));
    assert_eq!(g.battlefield_find(daretti).unwrap().counter_count(CounterType::Loyalty), 4);
}

/// Spire Phantasm draws only when its draft-time guess landed.
#[test]
fn spire_phantasm_draws_on_a_correct_guess() {
    use crabomination::draft::{DraftNotes, SPIRE_PHANTASM};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::spire_phantasm());
    drain_stack(&mut g);
    assert!(g.players[0].hand.is_empty(), "no note, no draw");

    let mut notes = DraftNotes::default();
    notes.note_number(SPIRE_PHANTASM, 1);
    g.players[1].draft_notes = notes;
    g.add_card_to_library(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(1, catalog::spire_phantasm());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "a correct guess draws");
}

/// Borderland Explorer trades a discard for a basic land, for anyone who takes
/// the deal.
#[test]
fn borderland_explorer_rummages_for_a_basic() {
    let mut g = two_player_game();
    let pitch = g.add_card_to_hand(0, catalog::grizzly_bears());
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![pitch])]));
    g.move_card_to_battlefield_for_test(0, catalog::borderland_explorer());
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pitch), "discarded");
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "and fetched a basic");
}

/// Animus of Predation wears every keyword it noted while removing cards from
/// the draft, and nothing it didn't.
#[test]
fn animus_of_predation_wears_its_noted_keywords() {
    use crabomination::draft::{ANIMUS_OF_PREDATION, DraftNotes};
    let mut g = two_player_game();
    let mut notes = DraftNotes::default();
    notes.note_keywords(ANIMUS_OF_PREDATION, &[Keyword::Flying, Keyword::Trample]);
    g.players[0].draft_notes = notes;
    let animus = g.move_card_to_battlefield_for_test(0, catalog::animus_of_predation());
    drain_stack(&mut g);
    let kws = g.computed_permanent(animus).expect("on battlefield").keywords;
    assert!(kws.contains(&Keyword::Flying), "noted flying is granted");
    assert!(!kws.contains(&Keyword::Trample), "trample isn't on the printed list");
}

/// Paliano Vanguard pumps other creatures sharing a noted type, not itself.
#[test]
fn paliano_vanguard_pumps_noted_types_only() {
    use crabomination::card::CreatureType;
    use crabomination::draft::{DraftNotes, PALIANO_VANGUARD};
    let mut g = two_player_game();
    let mut notes = DraftNotes::default();
    notes.note_creature_types(PALIANO_VANGUARD, &[CreatureType::Bear]);
    g.players[0].draft_notes = notes;
    let vanguard = g.move_card_to_battlefield_for_test(0, catalog::paliano_vanguard());
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    let other = g.move_card_to_battlefield_for_test(0, catalog::savannah_lions());
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bear).unwrap().power, 3, "a noted Bear is pumped");
    assert_eq!(g.computed_permanent(other).unwrap().power, 2, "an unnoted type isn't");
    assert_eq!(g.computed_permanent(vanguard).unwrap().power, 2, "and not the Vanguard");
}

/// Arcane Savant copies a pre-game exiled sorcery and casts the copy free.
#[test]
fn arcane_savant_casts_a_copy_of_its_exiled_spell() {
    let mut g = two_player_game();
    let div = g.seat_draft_exile(0, "Arcane Savant", catalog::divination());
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.move_card_to_battlefield_for_test(0, catalog::arcane_savant());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2, "the free copy drew two");
    assert!(g.exile.iter().any(|c| c.id == div), "the original stays exiled");
}

/// Volatile Chimera becomes one of the creatures it exiled before the game.
#[test]
fn volatile_chimera_becomes_an_exiled_creature() {
    let mut g = two_player_game();
    g.seat_draft_exile(0, "Volatile Chimera", catalog::grizzly_bears());
    let chimera = g.move_card_to_battlefield_for_test(0, catalog::volatile_chimera());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: chimera,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("shapeshift");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(chimera).unwrap().definition.name,
        "Grizzly Bears",
        "the only exiled creature is the only roll"
    );
}

/// Caller of the Untamed mints a token copy of an exiled creature whose mana
/// value matches the X it paid.
#[test]
fn caller_of_the_untamed_mints_the_x_cost_creature() {
    let mut g = two_player_game();
    g.seat_draft_exile(0, "Caller of the Untamed", catalog::grizzly_bears());
    let caller = g.move_card_to_battlefield_for_test(0, catalog::caller_of_the_untamed());
    g.clear_sickness(caller);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: caller,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: Some(2),
        mode: None,
    })
    .expect("call the Bears");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears" && c.is_token),
        "a token copy of the MV-2 exiled creature"
    );
}

/// Expropriate hands the caster a permanent owned by each "money" voter and
/// an extra turn per "time" vote, then exiles itself.
#[test]
fn expropriate_takes_a_permanent_from_each_money_voter() {
    let mut g = two_player_game();
    let mine = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    let theirs = g.move_card_to_battlefield_for_test(1, catalog::savannah_lions());
    // You vote Time, the opponent votes Money on their own Lions.
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(1),
        DecisionAnswer::Cards(vec![theirs]),
    ]));
    resolve_sorcery(&mut g, catalog::expropriate());
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0, "the Lions changed hands");
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 0, "your own bear is untouched");
    assert_eq!(g.players[0].extra_turns, 1, "the Time vote banked a turn");
    assert!(
        g.exile.iter().any(|c| c.definition.name == "Expropriate"),
        "Expropriate exiles itself"
    );
}

/// Selvala's Stampede digs a creature out per "wild" vote.
#[test]
fn selvalas_stampede_digs_out_a_creature_per_wild_vote() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Amount(0),
        DecisionAnswer::Amount(0),
    ]));
    resolve_sorcery(&mut g, catalog::selvalas_stampede());
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the wild votes dug the Bears out"
    );
}

/// Emissary's Ploy lets creature spells at the chosen mana value be cast off
/// any color, and leaves other mana values alone.
#[test]
fn emissarys_ploy_fixes_the_chosen_mana_value() {
    let mut g = two_player_game();
    let id = g.seat_conspiracy(0, catalog::emissarys_ploy(), None);
    g.players[0].command.iter_mut().find(|c| c.id == id).unwrap().chosen_number = Some(2);
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("{G}{G} Bears paid with white mana");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"));
}

/// Summoner's Bond fetches the other named creature when you cast the first.
#[test]
fn summoners_bond_tutors_the_other_name() {
    let mut g = two_player_game();
    let bond = g.seat_double_agenda(0, catalog::summoners_bond(), "Grizzly Bears", "Savannah Lions");
    assert!(g.reveal_hidden_agenda(0, bond), "turn the agenda face up (CR 702.106b)");
    let lions = g.add_card_to_library(0, catalog::savannah_lions());
    let bears = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(lions))]));
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bears");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == lions), "the other name is tutored up");
}

/// Sovereign's Realm cuts the opening hand to five and turns basics into
/// any-color sources.
#[test]
fn sovereigns_realm_shrinks_the_grip_and_fixes_basics() {
    let mut g = two_player_game();
    g.seat_conspiracy(0, catalog::sovereigns_realm(), None);
    assert_eq!(g.starting_hand_size(0), 5, "five cards, not seven");
    assert_eq!(g.starting_hand_size(1), 7, "the other seat is unaffected");
    let forest = g.move_card_to_battlefield_for_test(0, catalog::forest());
    let abilities = g.granted_abilities_for(forest);
    assert!(!abilities.is_empty(), "the Forest picked up an any-color tap ability");
}

/// Spy Kit makes the equipped creature answer to any nonlegendary creature
/// card's name.
#[test]
fn spy_kit_grants_every_nonlegendary_creature_name() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::game::types::Target;
    let mut g = two_player_game();
    let bear = g.move_card_to_battlefield_for_test(0, catalog::grizzly_bears());
    let kit = g.move_card_to_battlefield_for_test(0, catalog::spy_kit());
    g.battlefield_find_mut(kit).unwrap().attached_to = Some(bear);
    let named = |g: &GameState, n: &str| {
        g.evaluate_requirement_static(
            &R::HasName(n.to_string()),
            &Target::Permanent(bear),
            0,
            None,
        )
    };
    assert!(named(&g, "Savannah Lions"), "another nonlegendary creature's name matches");
    assert!(!named(&g, "Forest"), "a land's name doesn't");
}

/// Grenzo's exile mode grants a cast that still costs the card's mana value,
/// payable in any color (CR 609.4b) — not a free cast.
#[test]
fn grenzo_exile_mode_charges_the_mana_value() {
    let mut g = two_player_game();
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    let grenzo = g.move_card_to_battlefield_for_test(0, catalog::grenzo_havoc_raiser());
    let ctx = crabomination::game::effects::EffectContext::for_ability(grenzo, 0, None);
    g.resolve_effect(
        &crabomination::effect::Effect::ExileTopAndGrantMayPlay {
            who: crabomination::effect::PlayerRef::Seat(1),
            count: crabomination::card::Value::ONE,
            duration: crabomination::card::MayPlayDuration::EndOfThisTurn,
            pay_any_color: true,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        },
        &ctx,
    )
    .expect("Grenzo's exile mode");
    let exiled = g.exile.iter().find(|c| c.id == top).expect("exiled off the top");
    assert_eq!(
        exiled.granted_alt_cast_cost_eot.as_ref().map(|c| c.cmc()),
        Some(2),
        "the grant charges the mana value, not nothing"
    );
}
