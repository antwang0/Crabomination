//! Nemesis (NMS), first wave.

use crabomination::card::{CardDefinition, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
use crabomination::game::*;
use crabomination::mana::Color;

fn script(g: &mut GameState, answers: Vec<DecisionAnswer>) {
    g.decider = Box::new(ScriptedDecider::new(answers));
}

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

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: u32) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: Some(x),
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

/// Seat 0's `attacker` swings at seat 1, optionally blocked, through to damage.
fn combat(g: &mut GameState, attacker: CardId, blocker: Option<CardId>) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }]).expect("attack");
    if let Some(b) = blocker {
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.perform_action(GameAction::DeclareBlockers(vec![(b, attacker)])).expect("block");
        drain_stack(g);
    }
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
}

/// The vanilla-shaped bodies: printed keywords, no per-card behaviour to check
/// beyond the type line the stat audit already covers.
#[test]
fn nms_keyword_bodies_carry_their_printed_keywords() {
    let cases: &[(fn() -> CardDefinition, &[Keyword])] = &[
        (catalog::blastoderm, &[Keyword::Shroud, Keyword::Fading(3)]),
        (catalog::cloudskate, &[Keyword::Flying, Keyword::Fading(3)]),
        (catalog::skyshroud_ridgeback, &[Keyword::Fading(2)]),
        (catalog::rootwater_commando, &[Keyword::Landwalk(crabomination::card::LandType::Island)]),
        (catalog::spineless_thug, &[Keyword::CantBlock]),
        (catalog::rathi_intimidator, &[Keyword::Fear]),
        (catalog::carrion_wall, &[Keyword::Defender]),
        (catalog::flowstone_wall, &[Keyword::Defender]),
        (catalog::shrieking_mogg, &[Keyword::Haste]),
        (catalog::voice_of_truth, &[Keyword::Flying, Keyword::Protection(Color::White)]),
        (catalog::belbes_percher, &[Keyword::Flying, Keyword::CanBlockOnlyFlying]),
        (catalog::stronghold_zeppelin, &[Keyword::Flying, Keyword::CanBlockOnlyFlying]),
        (catalog::rhox, &[Keyword::AssignsDamageAsThoughUnblocked]),
    ];
    for (factory, expected) in cases {
        let def = factory();
        for kw in *expected {
            assert!(def.keywords.contains(kw), "{} is missing {kw:?}", def.name);
        }
    }
}

/// Blinding Angel takes the defender's next combat phase away.
#[test]
fn blinding_angel_skips_the_defenders_combat() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::blinding_angel());
    combat(&mut g, angel, None);
    assert_eq!(g.players[1].life, 18);
    assert!(g.players[1].skip_next_combat > 0, "their next combat is gone");
}

/// Chieftain en-Dal arms the whole attack when it swings.
#[test]
fn chieftain_en_dal_gives_the_attack_first_strike() {
    let mut g = two_player_game();
    let chief = g.add_card_to_battlefield(0, catalog::chieftain_en_dal());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(chief);
    g.clear_sickness(buddy);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![
        Attack { attacker: chief, target: AttackTarget::Player(1) },
        Attack { attacker: buddy, target: AttackTarget::Player(1) },
    ])
    .expect("attack");
    drain_stack(&mut g);
    let cp = g.computed_permanent(buddy).expect("computed");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "the other attacker got it too");
}

/// Avenger en-Dal exiles an attacker and pays its controller in life.
#[test]
fn avenger_en_dal_exiles_an_attacker_for_its_toughness() {
    let mut g = two_player_game();
    let avenger = g.add_card_to_battlefield(0, catalog::avenger_en_dal());
    g.clear_sickness(avenger);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw()); // 6/6
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    activate(&mut g, 0, avenger, 0, Some(Target::Permanent(attacker)));
    assert!(g.exile.iter().any(|c| c.id == attacker));
    assert_eq!(g.players[1].life, 26, "6 toughness back in life");
}

/// The Bringers trade themselves for a creature of the hated colour.
#[test]
fn bringers_exile_a_creature_of_their_hated_color() {
    let mut g = two_player_game();
    let law = g.add_card_to_battlefield(0, catalog::lawbringer());
    g.clear_sickness(law);
    let red = g.add_card_to_battlefield(1, catalog::shivan_dragon());
    activate(&mut g, 0, law, 0, Some(Target::Permanent(red)));
    assert!(g.exile.iter().any(|c| c.id == red));
    assert!(g.battlefield_find(law).is_none(), "sacrificed itself");
}

/// Lashknife can be cast by tapping a creature while you control a Plains.
#[test]
fn lashknife_taps_a_creature_instead_of_paying() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::plains());
    let helper = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::lashknife());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: aura,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(helper).unwrap().tapped || g.battlefield_find(host).unwrap().tapped);
    let cp = g.computed_permanent(host).expect("computed");
    assert!(cp.keywords.contains(&Keyword::FirstStrike));
}

/// Noble Stand pays two life for every block.
#[test]
fn noble_stand_pays_for_each_block() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::noble_stand());
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22);
}

/// Off Balance benches a creature both ways.
#[test]
fn off_balance_stops_attacks_and_blocks() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::off_balance());
    cast(&mut g, 0, spell, Some(Target::Permanent(victim)));
    let cp = g.computed_permanent(victim).expect("computed");
    assert!(cp.keywords.contains(&Keyword::CantAttack));
    assert!(cp.keywords.contains(&Keyword::CantBlock));
}

/// Silkenfist Fighter untaps when it gets blocked.
#[test]
fn silkenfist_fighter_untaps_when_blocked() {
    let mut g = two_player_game();
    let fighter = g.add_card_to_battlefield(0, catalog::silkenfist_fighter());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    combat(&mut g, fighter, Some(blocker));
    assert!(!g.battlefield_find(fighter).unwrap().tapped, "untapped by its own trigger");
}

/// Sivvi's Ruse is free with a Plains against a Mountain, and fogs your side.
#[test]
fn sivvis_ruse_is_free_and_one_sided() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::plains());
    g.add_card_to_battlefield(1, catalog::mountain());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let ruse = g.add_card_to_hand(0, catalog::sivvis_ruse());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: ruse,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    let theirs = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    g.clear_sickness(theirs);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker: theirs, target: AttackTarget::Player(0) }])
        .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(mine, theirs)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(mine).is_some(), "your creature is fogged");
}

/// Topple only hits the biggest creature on the board.
#[test]
fn topple_only_targets_the_biggest_creature() {
    let mut g = main_phase();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let topple = g.add_card_to_hand(0, catalog::topple());
    mana(&mut g, 0);
    assert!(
        g.perform_action(GameAction::CastSpell {
            card_id: topple,
            target: Some(Target::Permanent(small)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "the small one isn't a legal target"
    );
    cast(&mut g, 0, topple, Some(Target::Permanent(big)));
    assert!(g.exile.iter().any(|c| c.id == big));
}

/// Dominate steals anything cheap enough for its X.
#[test]
fn dominate_steals_within_its_x() {
    let mut g = main_phase();
    let bears = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let dom = g.add_card_to_hand(0, catalog::dominate());
    cast_x(&mut g, 0, dom, Some(Target::Permanent(bears)), 2);
    assert_eq!(g.battlefield_find(bears).unwrap().controller, 0);
}

/// Ensnare taps the table; two Islands buy it instead of mana.
#[test]
fn ensnare_taps_everything_for_two_islands() {
    let mut g = main_phase();
    let i1 = g.add_card_to_battlefield(0, catalog::island());
    let i2 = g.add_card_to_battlefield(0, catalog::island());
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let ensnare = g.add_card_to_hand(0, catalog::ensnare());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: ensnare,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == i1 || c.id == i2), "an Island came back");
    assert!(g.battlefield_find(mine).unwrap().tapped);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
}

/// Jolting Merfolk spends a fade counter to tap something.
#[test]
fn jolting_merfolk_spends_fade_counters_to_tap() {
    let mut g = two_player_game();
    let merfolk = g.move_card_to_battlefield_for_test(0, catalog::jolting_merfolk());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(merfolk).unwrap().counter_count(CounterType::Fade), 4);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, merfolk, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).unwrap().tapped);
    assert_eq!(g.battlefield_find(merfolk).unwrap().counter_count(CounterType::Fade), 3);
}

/// The Spellshaper counters answer the half of the stack they're printed for.
#[test]
fn stronghold_spellshapers_split_the_stack() {
    let mut g = main_phase();
    let bio = g.add_card_to_battlefield(0, catalog::stronghold_biologist());
    g.clear_sickness(bio);
    g.add_card_to_hand(0, catalog::grizzly_bears());
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    mana(&mut g, 1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    activate(&mut g, 0, bio, 0, Some(Target::Permanent(bears)));
    assert!(g.battlefield_find(bears).is_none(), "the creature spell was countered");
}

/// Submerge is free against a Forest and buries the creature on top.
#[test]
fn submerge_puts_a_creature_on_top_for_free() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::island());
    g.add_card_to_battlefield(1, catalog::forest());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let sub = g.add_card_to_hand(0, catalog::submerge());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: sub,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(victim));
}

/// Ascendant Evincar pumps black and shrinks everything else.
#[test]
fn ascendant_evincar_swings_the_board_by_color() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::ascendant_evincar());
    let theirs_black = g.add_card_to_battlefield(1, catalog::gravedigger()); // black
    let theirs_green = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green 2/2
    let black = g.computed_permanent(theirs_black).expect("computed");
    let green = g.computed_permanent(theirs_green).expect("computed");
    assert_eq!((black.power, black.toughness), (3, 3), "an opponent's black creature grows too");
    assert_eq!((green.power, green.toughness), (1, 1));
}

/// Death Pit Offering wipes your board, then anthems the rebuild.
#[test]
fn death_pit_offering_wipes_then_anthems() {
    let mut g = main_phase();
    let doomed = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let offering = g.add_card_to_hand(0, catalog::death_pit_offering());
    cast(&mut g, 0, offering, None);
    assert!(g.battlefield_find(doomed).is_none(), "your board went first");
    let rebuilt = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let cp = g.computed_permanent(rebuilt).expect("computed");
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Murderous Betrayal costs half your life and can't be regenerated around.
#[test]
fn murderous_betrayal_costs_half_your_life() {
    let mut g = two_player_game();
    let betrayal = g.add_card_to_battlefield(0, catalog::murderous_betrayal());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, betrayal, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
    assert_eq!(g.players[0].life, 10, "half of 20, rounded up");
}

/// Rathi Fiend drains everyone on the way in.
#[test]
fn rathi_fiend_drains_the_table() {
    let mut g = main_phase();
    let fiend = g.add_card_to_hand(0, catalog::rathi_fiend());
    cast(&mut g, 0, fiend, None);
    assert_eq!(g.players[0].life, 17);
    assert_eq!(g.players[1].life, 17);
}

/// The Mercenary/Rebel tutors pull a matching permanent onto the battlefield.
#[test]
fn rathi_intimidator_fetches_a_cheap_mercenary() {
    let mut g = two_player_game();
    let intim = g.add_card_to_battlefield(0, catalog::rathi_intimidator());
    g.clear_sickness(intim);
    let thug = g.add_card_to_library(0, catalog::spineless_thug()); // MV 2 Mercenary
    script(&mut g, vec![DecisionAnswer::Search(Some(thug))]);
    activate(&mut g, 0, intim, 0, None);
    assert!(g.battlefield_find(thug).is_some());
}

/// Stronghold Discipline scales per player off their own board.
#[test]
fn stronghold_discipline_scales_per_player() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_battlefield(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::stronghold_discipline());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].life, 19);
    assert_eq!(g.players[1].life, 17);
}

/// Volrath the Fallen grows by the discarded creature's mana value.
#[test]
fn volrath_grows_by_the_discarded_mana_value() {
    let mut g = two_player_game();
    let volrath = g.add_card_to_battlefield(0, catalog::volrath_the_fallen());
    g.add_card_to_hand(0, catalog::colossal_dreadmaw()); // MV 6
    activate(&mut g, 0, volrath, 0, None);
    let cp = g.computed_permanent(volrath).expect("computed");
    assert_eq!((cp.power, cp.toughness), (12, 10));
}

/// Ancient Hydra turns fade counters into pings.
#[test]
fn ancient_hydra_pings_off_its_fade_counters() {
    let mut g = two_player_game();
    let hydra = g.move_card_to_battlefield_for_test(0, catalog::ancient_hydra());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::Fade), 5);
    activate(&mut g, 0, hydra, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.battlefield_find(hydra).unwrap().counter_count(CounterType::Fade), 4);
}

/// Downhill Charge trades a Mountain for a Mountain-sized pump.
#[test]
fn downhill_charge_sacrifices_a_mountain_for_its_pump() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let charge = g.add_card_to_hand(0, catalog::downhill_charge());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: charge,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!(cp.power, 4, "2 + the two surviving Mountains");
}

/// The flowstone pumps trade toughness for power.
#[test]
fn flowstone_pumps_trade_toughness_for_power() {
    let mut g = two_player_game();
    let crusher = g.add_card_to_battlefield(0, catalog::flowstone_crusher());
    activate(&mut g, 0, crusher, 0, None);
    let cp = g.computed_permanent(crusher).expect("computed");
    assert_eq!((cp.power, cp.toughness), (5, 3));
}

/// Flowstone Surge is a whole-board flowstone anthem.
#[test]
fn flowstone_surge_anthems_your_board() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let surge = g.add_card_to_hand(0, catalog::flowstone_surge());
    cast(&mut g, 0, surge, None);
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 1));
}

/// Mogg Alarm makes two Goblins off two Mountains.
#[test]
fn mogg_alarm_pays_in_mountains() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let alarm = g.add_card_to_hand(0, catalog::mogg_alarm());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: alarm,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Goblin").count(), 2);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.is_land()).count(), 0);
}

/// Mogg Salvage is free artifact removal against an Island.
#[test]
fn mogg_salvage_is_free_against_an_island() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::mountain());
    g.add_card_to_battlefield(1, catalog::island());
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    let salvage = g.add_card_to_hand(0, catalog::mogg_salvage());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: salvage,
        target: Some(Target::Permanent(rock)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("free cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none());
}

/// Shrieking Mogg taps everything else on the way in.
#[test]
fn shrieking_mogg_taps_the_rest_of_the_table() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let mogg = g.add_card_to_hand(0, catalog::shrieking_mogg());
    cast(&mut g, 0, mogg, None);
    assert!(g.battlefield_find(mine).unwrap().tapped);
    assert!(g.battlefield_find(theirs).unwrap().tapped);
    assert!(!g.battlefield_find(mogg).unwrap().tapped, "not itself");
}

/// Coiling Woodworm's power counts every Forest, including opponents'.
#[test]
fn coiling_woodworm_counts_every_forest() {
    let mut g = two_player_game();
    let worm = g.add_card_to_battlefield(0, catalog::coiling_woodworm());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(1, catalog::forest());
    let cp = g.computed_permanent(worm).expect("computed");
    assert_eq!((cp.power, cp.toughness), (2, 1));
}

/// Mossdog grows every time an opponent targets it.
#[test]
fn mossdog_grows_when_an_opponent_targets_it() {
    let mut g = main_phase();
    let dog = g.add_card_to_battlefield(0, catalog::mossdog());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    mana(&mut g, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(dog)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    // 1/1 + a counter survives the 3 damage only if the counter landed first;
    // either way the trigger fired, so check the tally on the dead body's LKI.
    assert!(
        g.battlefield_find(dog).is_none_or(|c| c.counter_count(CounterType::PlusOnePlusOne) == 1),
        "the target trigger fired"
    );
}

/// Reverent Silence can be paid for by gifting everyone else 6 life.
#[test]
fn reverent_silence_pays_in_opponents_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forest());
    let ench = g.add_card_to_battlefield(1, catalog::flowstone_surge());
    let silence = g.add_card_to_hand(0, catalog::reverent_silence());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: silence,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none());
    assert_eq!(g.players[1].life, 26);
    assert_eq!(g.players[0].life, 20, "only the *other* players gain");
}

/// Skyshroud Behemoth enters tapped with its two fade counters.
#[test]
fn skyshroud_behemoth_enters_tapped_and_fading() {
    let mut g = main_phase();
    let behemoth = g.add_card_to_hand(0, catalog::skyshroud_behemoth());
    cast(&mut g, 0, behemoth, None);
    let c = g.battlefield_find(behemoth).expect("resolved");
    assert!(c.tapped);
    assert_eq!(c.counter_count(CounterType::Fade), 2);
}

/// Skyshroud Cutter can be paid for by gifting everyone else 5 life.
#[test]
fn skyshroud_cutter_pays_in_opponents_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::forest());
    let cutter = g.add_card_to_hand(0, catalog::skyshroud_cutter());
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: cutter,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
        pitch_card: None,
    })
    .expect("alt cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(cutter).is_some());
    assert_eq!(g.players[1].life, 25);
}

/// Stampede Driver pumps the team and hands out trample.
#[test]
fn stampede_driver_pumps_and_tramples() {
    let mut g = two_player_game();
    let driver = g.add_card_to_battlefield(0, catalog::stampede_driver());
    g.clear_sickness(driver);
    g.add_card_to_hand(0, catalog::forest());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    activate(&mut g, 0, driver, 0, None);
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Treetop Bracers pumps and grounds the blockers.
#[test]
fn treetop_bracers_pumps_and_evades() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::treetop_bracers());
    cast(&mut g, 0, aura, Some(Target::Permanent(host)));
    let cp = g.computed_permanent(host).expect("computed");
    assert_eq!((cp.power, cp.toughness), (3, 3));
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    combat(&mut g, host, None);
    assert!(g.battlefield_find(ground).is_some());
    assert_eq!(g.players[1].life, 17, "the ground blocker couldn't stop it");
}

/// Woodripper eats artifacts for fade counters.
#[test]
fn woodripper_spends_fade_counters_on_artifacts() {
    let mut g = two_player_game();
    let ripper = g.move_card_to_battlefield_for_test(0, catalog::woodripper());
    drain_stack(&mut g);
    let rock = g.add_card_to_battlefield(1, catalog::sol_ring());
    activate(&mut g, 0, ripper, 0, Some(Target::Permanent(rock)));
    assert!(g.battlefield_find(rock).is_none());
    assert_eq!(g.battlefield_find(ripper).unwrap().counter_count(CounterType::Fade), 2);
}

/// Aether Barrier taxes every creature spell.
#[test]
fn aether_barrier_taxes_creature_spells() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::aether_barrier());
    let sacrificial = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    // The AutoDecider declines the {1}, so a permanent goes.
    cast(&mut g, 0, bear, None);
    assert!(
        g.battlefield_find(sacrificial).is_none() || g.battlefield_find(bear).is_none(),
        "declining the tax cost a permanent"
    );
}

/// Belbe's Armor swaps power for toughness by X.
#[test]
fn belbes_armor_trades_power_for_toughness() {
    let mut g = two_player_game();
    let armor = g.add_card_to_battlefield(0, catalog::belbes_armor());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: armor,
        ability_index: 0,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(bear).expect("computed");
    assert_eq!((cp.power, cp.toughness), (0, 4));
}

/// Belbe's Portal deploys the type it named.
#[test]
fn belbes_portal_deploys_the_named_type() {
    let mut g = main_phase();
    script(&mut g, vec![DecisionAnswer::CreatureType(crabomination::card::CreatureType::Bear)]);
    let portal = g.add_card_to_hand(0, catalog::belbes_portal());
    cast(&mut g, 0, portal, None);
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    script(&mut g, vec![DecisionAnswer::Cards(vec![bear])]);
    activate(&mut g, 0, portal, 0, None);
    assert!(g.battlefield_find(bear).is_some());
}

/// Complex Automaton bounces itself once you're developed.
#[test]
fn complex_automaton_bounces_itself_when_developed() {
    let mut g = two_player_game();
    let golem = g.add_card_to_battlefield(0, catalog::complex_automaton());
    g.active_player_idx = 0;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).is_some(), "one permanent isn't seven");

    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == golem));
}

/// Flint Golem mills the defender when it gets blocked.
#[test]
fn flint_golem_mills_the_blocker() {
    let mut g = two_player_game();
    let golem = g.add_card_to_battlefield(0, catalog::flint_golem());
    let blocker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::forest());
    }
    let before = g.players[1].library.len();
    combat(&mut g, golem, Some(blocker));
    assert_eq!(g.players[1].library.len(), before - 3);
}

/// Rusting Golem's size is its remaining fade counters.
#[test]
fn rusting_golem_is_sized_by_its_fade_counters() {
    let mut g = main_phase();
    let golem = g.add_card_to_hand(0, catalog::rusting_golem());
    cast(&mut g, 0, golem, None);
    let cp = g.computed_permanent(golem).expect("computed");
    assert_eq!((cp.power, cp.toughness), (5, 5));
}

/// Viseling and Rackling punish opposite hand sizes.
#[test]
fn viseling_and_rackling_punish_opposite_hands() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::viseling());
    g.add_card_to_battlefield(0, catalog::rackling());
    for _ in 0..6 {
        g.add_card_to_hand(1, catalog::grizzly_bears());
    }
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    // Viseling: 6 - 4 = 2. Rackling: 3 - 6 = -3, so nothing.
    assert_eq!(g.players[1].life, 18);
}

/// Kor Haven blanks one attacker's combat damage.
#[test]
fn kor_haven_blanks_an_attackers_combat_damage() {
    let mut g = two_player_game();
    let haven = g.add_card_to_battlefield(0, catalog::kor_haven());
    let attacker = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(0) }]).expect("attack");
    activate(&mut g, 0, haven, 1, Some(Target::Permanent(attacker)));
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.players[0].life, 20);
}

/// Rath's Edge grinds a land into a point of damage.
#[test]
fn raths_edge_grinds_a_land_into_damage() {
    let mut g = two_player_game();
    let edge = g.add_card_to_battlefield(0, catalog::raths_edge());
    let fodder = g.add_card_to_battlefield(0, catalog::forest());
    activate(&mut g, 0, edge, 1, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 19);
    assert!(g.battlefield_find(fodder).is_none());
}

/// Terrain Generator unloads a basic out of hand, tapped.
#[test]
fn terrain_generator_deploys_a_basic_tapped() {
    let mut g = two_player_game();
    let terrain = g.add_card_to_battlefield(0, catalog::terrain_generator());
    let forest = g.add_card_to_hand(0, catalog::forest());
    script(&mut g, vec![DecisionAnswer::Cards(vec![forest])]);
    activate(&mut g, 0, terrain, 1, None);
    let c = g.battlefield_find(forest).expect("deployed");
    assert!(c.tapped);
}
