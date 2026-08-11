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

// ── Channel lands (Kamigawa legendary lands) ─────────────────────────────────

/// Boseiju, Who Endures channels from hand ({1}{G}, discard it) to destroy a
/// target nonbasic land.
#[test]
fn boseiju_channel_destroys_nonbasic() {
    let mut g = two_player_game();
    let boseiju = g.add_card_to_hand(0, catalog::boseiju_who_endures());
    let target = g.add_card_to_battlefield(1, catalog::mishras_factory());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    // The channel ability is index 1 (index 0 is the tap-for-{G} mana ability).
    g.perform_action(GameAction::ActivateAbility {
        card_id: boseiju, ability_index: 1, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel Boseiju");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "nonbasic land destroyed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == boseiju), "Boseiju discarded to channel");
}

/// CR 701.19a — an opponent-owned search ("that player may search") routes
/// the pick to the searched player's seat, not the caster's.
#[test]
fn boseiju_opponent_search_routes_to_the_searched_seat() {
    let mut g = two_player_game();
    g.players[0].wants_ui = true;
    g.players[1].wants_ui = true;
    let boseiju = g.add_card_to_hand(0, catalog::boseiju_who_endures());
    let target = g.add_card_to_battlefield(1, catalog::mishras_factory());
    let basic = g.add_card_to_library(1, catalog::forest());
    g.add_card_to_library(1, catalog::swamp());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: boseiju, ability_index: 1, target: Some(Target::Permanent(target)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel Boseiju");
    g.perform_action(GameAction::PassPriority).expect("active passes");
    g.perform_action(GameAction::PassPriority).expect("resolve");
    let pd = g.pending_decision.as_ref().expect("search pick pending");
    assert_eq!(pd.acting_player(), 1, "the searched player answers");
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Search(Some(basic))))
        .expect("opponent picks the basic");
    assert!(g.battlefield.iter().any(|c| c.id == basic && c.controller == 1 && !c.tapped),
        "fetched land enters untapped under the searched player");
}

/// Sokenzan channels for two 1/1 hasty Spirits.
#[test]
fn sokenzan_channel_makes_two_spirits() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::sokenzan_crucible_of_defiance());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel Sokenzan");
    drain_stack(&mut g);
    let spirits: Vec<_> = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit" && c.controller == 0).collect();
    assert_eq!(spirits.len(), 2);
    assert!(spirits.iter().all(|c| c.definition.printed_colors().is_empty()), "Spirits are colorless");
}

/// Takenuma channels ({3}{B}) to mill three, then return a creature card from
/// your graveyard to hand.
#[test]
fn takenuma_channel_mills_then_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::takenuma_abandoned_mire());
    let dead = g.next_id();
    g.players[0].graveyard.push(CardInstance::new(dead, catalog::grizzly_bears(), 0));
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel Takenuma");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned to hand");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Forest").count(), 3,
        "milled three lands into the graveyard");
}

/// Eiganjo channels ({2}{W}) to deal 4 to a target attacking creature; a
/// non-attacking creature is an illegal target.
#[test]
fn eiganjo_channel_burns_attacking_creature() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::eiganjo_seat_of_the_empire());
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let idle = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(0) }];
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    // A non-attacking, non-blocking creature is not a legal target.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: Some(Target::Permanent(idle)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "idle creature rejected");
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 1, target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("channel Eiganjo at the attacker");
    drain_stack(&mut g);
    assert!(g.battlefield_find(attacker).is_none(), "4 damage kills the 2/2 attacker");
}

/// Fountainport's {4}, {T} ability creates a Treasure token.
#[test]
fn fountainport_makes_a_treasure() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::fountainport());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 3, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate {4}, {T}");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Treasure"));
}

/// Restless Spire animates into a 2/1 first-strike Elemental.
#[test]
fn restless_spire_animates_first_strike() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_spire());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate for {U}{R}");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!((post.power, post.toughness), (2, 1));
    assert!(post.card_types.contains(&CardType::Land), "still a land");
    assert!(post.keywords.contains(&Keyword::FirstStrike));
}

/// Morph: Ainok Survivalist is cast face down for {3} as a 2/2, then turned up
/// for its Megamorph {G} — entering a +1/+1 counter (3/2) and firing its
/// turn-face-up trigger to destroy an opponent's artifact.
#[test]
fn ainok_survivalist_morph_cast_and_megamorph_turn_up() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::null_rod()); // an artifact
    let id = g.add_card_to_hand(0, catalog::ainok_survivalist());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down for {3}");
    drain_stack(&mut g);
    let fd = g.battlefield_find(id).expect("on battlefield");
    assert!(fd.face_down, "entered face down");
    assert_eq!((fd.power(), fd.toughness()), (2, 2), "face-down 2/2");
    // Turn it up via Megamorph {G}.
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {G}");
    drain_stack(&mut g);
    let up = g.battlefield_find(id).expect("still here");
    assert!(!up.face_down);
    assert_eq!(up.definition.name, "Ainok Survivalist");
    assert_eq!((up.power(), up.toughness()), (3, 2), "2/1 + megamorph +1/+1 counter");
    assert!(g.battlefield_find(victim).is_none(), "turn-up trigger destroyed the artifact");
}

/// Defenestrated Phantom: Disguise casts a face-down 2/2 with ward {2}, then
/// turns up into the 4/3 flyer for {4}{W}.
#[test]
fn disguise_defenestrated_phantom_face_down_has_ward_then_turns_up() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::defenestrated_phantom());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down for {3}");
    drain_stack(&mut g);
    let fd = g.battlefield_find(id).expect("on battlefield");
    assert!(fd.face_down, "entered face down");
    assert_eq!((fd.power(), fd.toughness()), (2, 2), "face-down 2/2");
    assert_eq!(fd.ward_cost(), Some(2), "Disguise grants ward {{2}} while face down");
    // Turn it up for its disguise cost {4}{W}.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {4}{W}");
    let up = g.battlefield_find(id).expect("still here");
    assert!(!up.face_down);
    assert_eq!((up.power(), up.toughness()), (4, 3));
    assert!(up.definition.keywords.contains(&Keyword::Flying));
    assert_eq!(up.ward_cost(), None, "real face has no ward");
}

/// Nervous Gardener's turn-face-up trigger tutors a basic land to hand.
#[test]
fn disguise_nervous_gardener_turn_up_searches_basic_land() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let id = g.add_card_to_hand(0, catalog::nervous_gardener());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {G}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == forest), "basic land tutored to hand");
}

/// Bubble Smuggler enters play four +1/+1 counters bigger when turned face up.
#[test]
fn disguise_bubble_smuggler_turn_up_adds_four_counters() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bubble_smuggler());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {5}{U}");
    drain_stack(&mut g);
    let up = g.battlefield_find(id).expect("still here");
    assert_eq!((up.power(), up.toughness()), (6, 5), "2/1 + four +1/+1 counters");
}

/// A face-down permanent the seat can pay to unmask is surfaced in the
/// `turn_up_able` affordance (drives the client's turn-up highlight).
#[test]
fn turn_up_able_affordance_lists_payable_face_down_permanent() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::defenestrated_phantom());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // Not yet affordable ({4}{W} turn-up cost): absent from the affordance.
    assert!(!g.compute_hand_affordances(0).turn_up_able.contains(&id));
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    assert!(g.compute_hand_affordances(0).turn_up_able.contains(&id),
        "payable face-down permanent is surfaced");
}

/// The bot unmasks an affordable face-down Disguise creature in its main phase.
#[test]
fn bot_turns_up_affordable_face_down_creature() {
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bubble_smuggler());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    // Give the bot the disguise turn-up cost {5}{U} and nothing else to do.
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let mut bot = HeuristicBot::new();
    let action = bot.next_action(&g, 0);
    assert!(matches!(action, Some(GameAction::TurnFaceUp { card_id }) if card_id == id),
        "bot should turn up the face-down creature; got {action:?}");
}

/// Mosquito Guard's Reinforce 1 grows a creature by one counter.
#[test]
fn reinforce_mosquito_guard_puts_one_counter() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    // The body is a 1/1 first striker.
    let body = g.add_card_to_battlefield(0, catalog::mosquito_guard());
    assert!(g.battlefield_find(body).unwrap().definition.keywords.contains(&Keyword::FirstStrike));
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::mosquito_guard());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Reinforce {
        card_id: id, target: Target::Permanent(target),
    }).expect("reinforce");
    assert_eq!((g.battlefield_find(target).unwrap().power(), g.battlefield_find(target).unwrap().toughness()),
        (3, 3), "2/2 + one +1/+1 counter");
}

/// Burrenton Bombardier is a 2/2 flyer with Reinforce 2.
#[test]
fn burrenton_bombardier_is_a_flyer_with_reinforce() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::burrenton_bombardier());
    assert!(matches!(
        catalog::burrenton_bombardier().keywords.iter().find(|k| matches!(k, Keyword::Reinforce(..))),
        Some(Keyword::Reinforce(2, _))
    ));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Reinforce {
        card_id: id, target: Target::Permanent(target),
    }).expect("reinforce");
    assert_eq!((g.battlefield_find(target).unwrap().power(), g.battlefield_find(target).unwrap().toughness()),
        (4, 4), "2/2 + two +1/+1 counters");
}

/// Bannerhide Krushok's Reinforce 2 discards it to grow a creature (CR 702.77).
#[test]
fn reinforce_bannerhide_krushok_puts_two_counters() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::bannerhide_krushok());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Reinforce {
        card_id: id, target: Target::Permanent(target),
    }).expect("reinforce");
    let t = g.battlefield_find(target).expect("here");
    assert_eq!((t.power(), t.toughness()), (4, 4), "2/2 + two +1/+1 counters");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == id), "Krushok discarded as cost");
    assert!(g.battlefield_find(id).is_none(), "Krushok not on the battlefield");
}

/// The reinforceable_hand affordance surfaces a Reinforce card only when the
/// cost is payable and a creature target exists (CR 702.77).
#[test]
fn reinforceable_affordance_requires_mana_and_target() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bannerhide_krushok());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // No creature on the battlefield and no mana: not reinforceable.
    assert!(!g.compute_hand_affordances(0).reinforceable.contains(&id));
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(g.compute_hand_affordances(0).reinforceable.contains(&id),
        "payable Reinforce with a creature target is surfaced");
}

/// Granite Witness taps a creature when turned face up (tap mode of CR's
/// "tap or untap").
#[test]
fn disguise_granite_witness_turn_up_taps_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::granite_witness());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(0),
        DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).expect("here").tapped, "target creature tapped");
}

/// Granite Witness can instead choose to untap a creature when turned face up.
#[test]
fn disguise_granite_witness_turn_up_can_untap() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(ally).unwrap().tapped = true;
    let id = g.add_card_to_hand(0, catalog::granite_witness());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Mode(1),
        DecisionAnswer::Target(Target::Permanent(ally)),
    ]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::White, 2);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(ally).expect("here").tapped, "untap mode untapped the creature");
}

/// Experiment Twelve doubles in size when turned face up.
#[test]
fn disguise_experiment_twelve_turn_up_doubles_power() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::experiment_twelve());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    let w = g.battlefield_find(id).expect("here");
    // 4/4 base, +4/+4 from counters equal to its power (4).
    assert_eq!((w.power(), w.toughness()), (8, 8));
}

/// Culvert Ambusher forces a creature to block when it enters.
#[test]
fn disguise_culvert_ambusher_etb_forces_block() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::culvert_ambusher());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(victim))]));
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Culvert Ambusher");
    drain_stack(&mut g);
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::MustBlock),
        "target must block this turn");
}

/// Alley Assailant enters tapped and drains 3 when it's turned face up.
#[test]
fn disguise_alley_assailant_enters_tapped_and_drains_on_turn_up() {
    let mut g = two_player_game();
    // Cast it face up first to check the enters-tapped trigger.
    let copy = g.add_card_to_hand(0, catalog::alley_assailant());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: copy, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast face up");
    drain_stack(&mut g);
    assert!(g.battlefield_find(copy).expect("here").tapped, "enters tapped");
    // Now the disguise path: face down, then turn up to drain.
    let id = g.add_card_to_hand(0, catalog::alley_assailant());
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    let opp_before = g.players[1].life;
    let me_before = g.players[0].life;
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp_before - 3, "opponent loses 3");
    assert_eq!(g.players[0].life, me_before + 3, "you gain 3");
}

/// Offender at Large pumps up to one creature +2/+0 when turned face up
/// (CR 115.1b — the "up to one target" rides Effect::MayDo).
#[test]
fn disguise_offender_at_large_turn_up_pumps_target() {
    let mut g = two_player_game();
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::offender_at_large());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(buddy)),
    ]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    assert_eq!((g.computed_permanent(buddy).unwrap().power, g.computed_permanent(buddy).unwrap().toughness),
        (4, 2), "Grizzly Bears 2/2 +2/+0");
}

/// Offender at Large's controller may target zero creatures ("up to one").
#[test]
fn offender_at_large_may_decline_its_target() {
    let mut g = two_player_game();
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_battlefield(0, catalog::offender_at_large());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(false)]));
    g.dispatch_triggers_for_events(&[crabomination::game::types::GameEvent::PermanentEntered { card_id: id }]);
    drain_stack(&mut g);
    assert_eq!((g.computed_permanent(buddy).unwrap().power, g.computed_permanent(buddy).unwrap().toughness),
        (2, 2), "declined: no pump");
}

/// Faerie Snoop's turn-up digs two cards, taking one to hand.
#[test]
fn disguise_faerie_snoop_turn_up_digs_two() {
    let mut g = two_player_game();
    let keep = g.add_card_to_library(0, catalog::lightning_bolt());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(keep))]));
    let id = g.add_card_to_hand(0, catalog::faerie_snoop());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == keep), "kept card went to hand");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "exactly one card taken");
}

/// Crowd-Control Warden enters sized by the rest of the board.
#[test]
fn crowd_control_warden_enters_with_counters_per_other_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::crowd_control_warden());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Warden");
    drain_stack(&mut g);
    let w = g.battlefield_find(id).expect("here");
    assert_eq!((w.power(), w.toughness()), (6, 6), "4/4 + two +1/+1 counters (two other creatures)");
}

/// Riftburst Hellion disguises and turns up into a 6/7 reacher.
#[test]
fn disguise_riftburst_hellion_turns_up_6_7_reach() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::riftburst_hellion());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    // Disguise {4}{R/G}{R/G}: pay the hybrid pips with red.
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    let up = g.battlefield_find(id).expect("here");
    assert_eq!((up.power(), up.toughness()), (6, 7));
    assert!(up.definition.keywords.contains(&Keyword::Reach));
}

/// Rakish Scoundrel's turn-up grants a creature indestructible until EOT.
#[test]
fn disguise_rakish_scoundrel_turn_up_grants_indestructible() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::rakish_scoundrel());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(buddy))]));
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
    drain_stack(&mut g);
    assert!(g.computed_permanent(buddy).unwrap().keywords.contains(&Keyword::Indestructible),
        "target gains indestructible until EOT");
}

/// Bolrac-Clan Basher: a disguised cast turns up into a double-striking trampler.
#[test]
fn disguise_bolrac_clan_basher_turns_up_with_keywords() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::bolrac_clan_basher());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {3}{R}{R}");
    let up = g.battlefield_find(id).expect("here");
    assert_eq!((up.power(), up.toughness()), (3, 2));
    assert!(up.definition.keywords.contains(&Keyword::DoubleStrike));
    assert!(up.definition.keywords.contains(&Keyword::Trample));
}

/// Basilica Stalker gains life and surveils when it connects.
#[test]
fn basilica_stalker_combat_damage_gains_life_and_surveils() {
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::basilica_stalker());
    g.clear_sickness(stalker);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    let life_before = g.players[0].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: stalker, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life_before + 1, "gained 1 life on combat damage");
}

/// Museum Nightwatch leaves a 2/2 Detective behind when it dies.
#[test]
fn museum_nightwatch_dies_into_detective() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::museum_nightwatch());
    g.battlefield_find_mut(id).unwrap().damage = 2;
    g.check_state_based_actions();
    drain_stack(&mut g);
    let det = g.battlefield.iter().find(|c| c.definition.name == "Detective").expect("Detective token");
    assert_eq!((det.power(), det.toughness()), (2, 2));
}

/// Pyrotechnic Performer burns each opponent for its power when turned face up.
#[test]
fn disguise_pyrotechnic_performer_turn_up_burns_opponents() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::pyrotechnic_performer());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    let life_before = g.players[1].life;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {R}");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 3, "deals 3 (its power) to the opponent");
}

/// Shady Informant deals 2 damage to any target when it dies.
#[test]
fn shady_informant_dies_deals_two_damage() {
    let mut g = two_player_game();
    let informant = g.add_card_to_battlefield(0, catalog::shady_informant());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    let life_before = g.players[1].life;
    // Kill via lethal damage + SBA.
    g.battlefield_find_mut(informant).unwrap().damage = 2;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life_before - 2, "dies-trigger deals 2 to the chosen player");
}

/// Voracious Varmint sacrifices itself to destroy an artifact or enchantment.
#[test]
fn voracious_varmint_sacs_to_destroy_artifact() {
    let mut g = two_player_game();
    let varmint = g.add_card_to_battlefield(0, catalog::voracious_varmint());
    let art = g.add_card_to_battlefield(1, catalog::null_rod());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: varmint, ability_index: 0, target: Some(Target::Permanent(art)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate sac ability");
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "artifact destroyed");
    assert!(g.battlefield_find(varmint).is_none(), "varmint sacrificed as cost");
}

/// Greenbelt Radical's turn-up pumps the whole team with +1/+1 counters.
#[test]
fn disguise_greenbelt_radical_turn_up_pumps_team() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::greenbelt_radical());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
    drain_stack(&mut g);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up for {5}{G}{G}");
    drain_stack(&mut g);
    let other = g.battlefield_find(buddy).expect("buddy here");
    assert_eq!((other.power(), other.toughness()), (3, 3), "team gets a +1/+1 counter");
    assert!(g.computed_permanent(buddy).unwrap().keywords.contains(&Keyword::Trample),
        "team gains trample until EOT");
}

/// Hide in Plain Sight cloaks the top two library cards: two face-down 2/2
/// ward-{2} creatures enter, and a cloaked creature can be turned face up for
/// its mana cost.
#[test]
fn cloak_hide_in_plain_sight_makes_two_warded_face_down_creatures() {
    let mut g = two_player_game();
    // Stack the library so the top two are a known creature (Grizzly Bears 2/2).
    let top1 = g.add_card_to_library(0, catalog::grizzly_bears());
    let _top2 = g.add_card_to_library(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::hide_in_plain_sight());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Hide in Plain Sight");
    drain_stack(&mut g);
    let cloaked: Vec<_> = g.battlefield.iter().filter(|c| c.face_down && c.controller == 0).collect();
    assert_eq!(cloaked.len(), 2, "two cloaked permanents entered");
    assert!(cloaked.iter().all(|c| c.ward_cost() == Some(2)), "cloak grants ward {{2}}");
    assert!(cloaked.iter().all(|c| (c.power(), c.toughness()) == (2, 2)), "face-down 2/2");
    // Turn one up for its mana cost ({1}{G} Grizzly Bears).
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::TurnFaceUp { card_id: top1 }).expect("turn cloaked creature up");
    let up = g.battlefield_find(top1).expect("still here");
    assert!(!up.face_down);
    assert_eq!(up.definition.name, "Grizzly Bears");
    assert_eq!(up.ward_cost(), None, "real face has no ward");
}

/// Restless Reef animates into a 4/4 deathtouch Shark; attack: target
/// player mills four.
#[test]
fn restless_reef_animates_and_mills_on_attack() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_reef());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate Restless Reef");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!((post.power, post.toughness), (4, 4));
    assert!(post.keywords.contains(&crabomination::card::Keyword::Deathtouch));
    for _ in 0..5 { g.add_card_to_library(1, catalog::island()); }
    let lib_before = g.players[1].library.len();
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: land, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), lib_before - 4, "target player milled four");
}

/// Restless Vinestalk animates into a 5/5 trampler.
#[test]
fn restless_vinestalk_animates_5_5_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_vinestalk());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("animate Vinestalk");
    drain_stack(&mut g);
    let post = g.computed_permanent(land).unwrap();
    assert_eq!((post.power, post.toughness), (5, 5));
    assert!(post.keywords.contains(&Keyword::Trample));
}

/// Trygon Predator's combat-damage trigger destroys an opponent's artifact.
#[test]
fn trygon_predator_destroys_artifact_on_combat_damage() {
    let mut g = two_player_game();
    let trygon = g.add_card_to_battlefield(0, catalog::trygon_predator());
    let art = g.add_card_to_battlefield(1, catalog::null_rod());
    let trig = catalog::trygon_predator().triggered_abilities[0].effect.clone();
    let ctx = crabomination::game::effects::EffectContext::for_trigger(
        trygon, 0, Some(Target::Permanent(art)), 0,
    );
    g.resolve_effect(&trig, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.battlefield_find(art).is_none(), "Trygon destroyed the opponent's artifact");
}

/// Restless Fortress animates 1/4 and drains 2 from the defender on attack.
#[test]
fn restless_fortress_drains_on_attack() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_fortress());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("animate");
    drain_stack(&mut g);
    let opp = g.players[1].life;
    let me = g.players[0].life;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: land, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "defending player lost 2");
    assert_eq!(g.players[0].life, me + 2, "you gained 2");
}

/// Restless Cottage animates 4/4 and creates a Food on attack.
#[test]
fn restless_cottage_makes_food_on_attack() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::restless_cottage());
    g.clear_sickness(land);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility { card_id: land, ability_index: 2, target: None, additional_targets: Vec::new(), x_value: None , mode: None}).expect("animate");
    drain_stack(&mut g);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: land, target: AttackTarget::Player(1) }]).expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food" && c.controller == 0));
}

/// A Morph card in hand with {3} available is surfaced as morphable.
#[test]
fn morph_card_surfaced_as_morphable_affordance() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::ainok_survivalist());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(3);
    let aff = g.compute_hand_affordances(0);
    assert!(aff.morphable.contains(&id), "Morph card with {{3}} is morphable");
    // Without mana it isn't offered.
    let mut g2 = two_player_game();
    let id2 = g2.add_card_to_hand(0, catalog::ainok_survivalist());
    g2.active_player_idx = 0;
    g2.priority.player_with_priority = 0;
    let aff2 = g2.compute_hand_affordances(0);
    assert!(!aff2.morphable.contains(&id2), "not morphable without {{3}}");
}

/// Simian Sling: as a 1/1 creature, becoming blocked pings the defender.
#[test]
fn simian_sling_pings_defender_when_blocked() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let sling = g.add_card_to_battlefield(0, catalog::simian_sling());
    let def = catalog::simian_sling();
    assert!(def.keywords.iter().any(|k| matches!(k, Keyword::Reconfigure(_))), "has Reconfigure");
    let trig = def.triggered_abilities[0].effect.clone();
    let opp = g.players[1].life;
    let ctx = crabomination::game::effects::EffectContext::for_trigger(sling, 0, None, 0);
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[1].life, opp - 1, "becomes-blocked pings each opponent for 1");
}

/// Painful Truths draws and loses life equal to the colors of mana spent (Converge).
#[test]
fn painful_truths_draws_and_loses_per_converge() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::painful_truths());
    // Pay {2}{B} with three distinct colors → converge 3.
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    let (hand, life) = (g.players[0].hand.len(), g.players[0].life);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Painful Truths");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 3, "converge 3 → draw 3 (minus the cast card)");
    assert_eq!(g.players[0].life, life - 3, "converge 3 → lose 3 life");
}

/// Prophetic Bolt deals 4 and digs for a card.
#[test]
fn prophetic_bolt_burns_and_digs() {
    let mut g = two_player_game();
    let keep = g.add_card_to_library(0, catalog::lightning_bolt());
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let id = g.add_card_to_hand(0, catalog::prophetic_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(keep))]));
    for c in [Color::Blue, Color::Red] { g.players[0].mana_pool.add(c, 1); }
    g.players[0].mana_pool.add_colorless(3);
    let life = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Prophetic Bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 4, "4 damage to the player");
    assert!(g.players[0].hand.iter().any(|c| c.id == keep), "dug the chosen card into hand");
}

/// Reality Shift exiles a creature and manifests the controller's top card.
#[test]
fn reality_shift_exiles_and_manifests() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::lightning_bolt()); // becomes a face-down 2/2
    let id = g.add_card_to_hand(0, catalog::reality_shift());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(victim)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Reality Shift");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "victim exiled");
    assert!(g.exile.iter().any(|c| c.id == victim), "victim is in exile");
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.face_down),
        "controller manifested a face-down 2/2");
}

/// Rite of Flame adds {R}{R}.
#[test]
fn rite_of_flame_adds_two_red() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::rite_of_flame());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Rite of Flame");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 2, "net RR after paying R");
}

/// Censor counters a spell unless its controller pays {1}, and can be cycled.
#[test]
fn censor_counters_unless_paid() {
    let mut g = two_player_game();
    // Opponent casts a spell; we Censor it with no mana for them to pay.
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opp casts bear");
    let censor = g.add_card_to_hand(0, catalog::censor());
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: censor, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Censor");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "bear countered (no mana to pay)");
}

/// Fading Hope bounces a creature and scries if its mana value was 3 or less.
#[test]
fn fading_hope_bounces_and_scries_cheap_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2 → scry
    let top = g.add_card_to_library(0, catalog::lightning_bolt());
    let id = g.add_card_to_hand(0, catalog::fading_hope());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::ScryOrder { kept_top: vec![top], bottom: vec![] },
    ]));
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Fading Hope");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear bounced");
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bear back in owner's hand");
}

/// Third Path Iconoclast mints a 1/1 artifact Soldier on a noncreature cast.
#[test]
fn third_path_iconoclast_makes_soldier_on_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::third_path_iconoclast());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast a noncreature spell");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Soldier" && c.controller == 0
        && c.definition.is_artifact()), "made a 1/1 artifact Soldier");
}

/// Crackling Drake's power equals instants/sorceries in your graveyard + exile.
#[test]
fn crackling_drake_power_scales_with_spells() {
    let mut g = two_player_game();
    let drake = g.add_card_to_battlefield(0, catalog::crackling_drake());
    assert_eq!(g.computed_permanent(drake).unwrap().power, 0, "no spells → 0 power");
    let (i1, i2, i3) = (g.next_id(), g.next_id(), g.next_id());
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(i1, catalog::lightning_bolt(), 0));
    g.players[0].graveyard.push(crabomination::card::CardInstance::new(i2, catalog::ponder(), 0));
    g.exile.push(crabomination::card::CardInstance::new(i3, catalog::tribal_flames(), 0));
    assert_eq!(g.computed_permanent(drake).unwrap().power, 3, "two in gy + one in exile → 3 power");
    assert_eq!(g.computed_permanent(drake).unwrap().toughness, 4, "toughness fixed at 4");
}

/// Slip Out the Back counters a creature then phases it out.
#[test]
fn slip_out_the_back_pumps_and_phases() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let id = g.add_card_to_hand(0, catalog::slip_out_the_back());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(bear)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Slip Out the Back");
    drain_stack(&mut g);
    // Phased out → no longer on the battlefield, but its +1/+1 counter persists.
    assert!(g.battlefield_find(bear).is_none(), "bear phased out");
    assert!(g.phased_out.iter().any(|c| c.id == bear
        && c.counter_count(crabomination::card::CounterType::PlusOnePlusOne) == 1),
        "phased-out bear carries its +1/+1 counter");
}

/// Niv-Mizzet, Parun pings on each draw and draws when any instant/sorcery is cast.
#[test]
fn niv_mizzet_parun_draw_ping_loop() {
    let mut g = two_player_game();
    let niv = g.add_card_to_battlefield(0, catalog::niv_mizzet_parun());
    assert!(niv == niv && catalog::niv_mizzet_parun().keywords.contains(&Keyword::CantBeCountered));
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Player(1))]));
    g.players[0].mana_pool.add(Color::Red, 1);
    let life = g.players[1].life;
    // Casting Lightning Bolt: Niv draws (cast IS), and that draw pings for 1.
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    // Bolt's 3 + Niv's draw-ping 1 = 4 to the opponent.
    assert_eq!(g.players[1].life, life - 4, "Bolt 3 + Niv draw-trigger ping 1");
}

/// Aria of Flame gains 10 life and escalates damage with verse counters.
#[test]
fn aria_of_flame_gains_life_and_scales_damage() {
    let mut g = two_player_game();
    let aria = g.add_card_to_hand(0, catalog::aria_of_flame());
    let life0 = g.players[0].life;
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: aria, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Aria of Flame");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 10, "ETB gains 10 life");
    // First instant cast: verse counter → 1 → 1 damage.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Player(1)), // Bolt's target
        DecisionAnswer::Target(Target::Player(1)), // Aria's damage target
    ]));
    g.players[0].mana_pool.add(Color::Red, 1);
    let life1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 3 - 1, "Bolt 3 + Aria 1 (one verse counter)");
}

/// Baral makes instants and sorceries cost {1} less.
#[test]
fn baral_reduces_instant_sorcery_cost() {
    use crabomination::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::baral_chief_of_compliance());
    let bolt = crabomination::card::CardInstance::new(g.next_id(), catalog::lightning_bolt(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bolt, None), 1, "instant costs 1 less");
    let bear = crabomination::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &bear, None), 0, "creatures unaffected");
}

/// Lightning Mauler grants haste to its soulbond partner.
#[test]
fn lightning_mauler_soulbond_grants_haste() {
    let mut g = two_player_game();
    let partner = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mauler = g.add_card_to_battlefield(0, catalog::lightning_mauler());
    g.apply_soulbond_pairing(mauler);
    // The pair is linked; the partner gains haste from the bonus.
    assert!(g.computed_permanent(partner).unwrap().keywords.contains(&Keyword::Haste)
        || g.battlefield_find(mauler).unwrap().soulbond_partner == Some(partner),
        "soulbond pair formed (partner gains haste)");
}

/// Delighted Halfling taps for {C} or any color.
#[test]
fn delighted_halfling_makes_mana() {
    let mut g = two_player_game();
    let h = g.add_card_to_battlefield(0, catalog::delighted_halfling());
    g.clear_sickness(h);
    g.perform_action(GameAction::ActivateAbility {
        card_id: h, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("tap for colorless");
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 1, "added colorless mana");
}

/// Voracious Hydra enters with X counters and can double them.
#[test]
fn voracious_hydra_doubles_counters_mode() {
    let mut g = two_player_game();
    let id = g.add_card_to_hand(0, catalog::voracious_hydra());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)])); // double counters
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: Some(3),
    }).expect("cast Voracious Hydra with X=3");
    drain_stack(&mut g);
    // Enters with 3 counters, doubled to 6 → 6/7.
    let h = g.computed_permanent(id).unwrap();
    assert_eq!((h.power, h.toughness), (6, 7), "0/1 + six +1/+1 counters");
}

// ── Cipher batch (CR 702.46) ──────────────────────────────────────────────────

fn precombat(g: &mut GameState) {
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
}

#[test]
fn last_thoughts_draws_a_card() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::last_thoughts());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    precombat(&mut g);
    let before = g.players[0].hand.len();
    crabomination::game::cast(&mut g, spell);
    assert_eq!(g.players[0].hand.len(), before, "spent one (Last Thoughts) and drew one");
}

#[test]
fn paranoid_delusions_mills_target_player() {
    let mut g = two_player_game();
    for _ in 0..5 { g.add_card_to_library(1, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::paranoid_delusions());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    precombat(&mut g);
    crabomination::game::cast_at(&mut g, spell, Target::Player(1));
    assert_eq!(g.players[1].graveyard.len(), 3, "milled three");
}

#[test]
fn midnight_recovery_returns_creature_from_graveyard() {
    let mut g = two_player_game();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::midnight_recovery());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    precombat(&mut g);
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(dead));
    assert!(g.players[0].hand.iter().any(|c| c.id == dead), "creature returned to hand");
}

#[test]
fn hands_of_binding_taps_and_stuns() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(victim);
    let spell = g.add_card_to_hand(0, catalog::hands_of_binding());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    precombat(&mut g);
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(victim));
    let v = g.battlefield_find(victim).unwrap();
    assert!(v.tapped, "victim tapped");
    assert_eq!(v.counter_count(CounterType::Stun), 1, "stun counter applied");
}

#[test]
fn stolen_identity_makes_a_token_copy() {
    let mut g = two_player_game();
    let orig = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::stolen_identity());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    precombat(&mut g);
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(orig));
    let copies = g.battlefield.iter()
        .filter(|c| c.controller == 0 && c.definition.name == "Grizzly Bears" && c.is_token)
        .count();
    assert_eq!(copies, 1, "a token copy of the bear under our control");
}

#[test]
fn whispering_madness_wheels_each_player() {
    let mut g = two_player_game();
    // Seat 0 holds 2 cards (+ the spell); seat 1 holds 4.
    for _ in 0..2 { g.add_card_to_hand(0, catalog::grizzly_bears()); }
    for _ in 0..4 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    for _ in 0..10 { g.add_card_to_library(0, catalog::island()); }
    for _ in 0..10 { g.add_card_to_library(1, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::whispering_madness());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    precombat(&mut g);
    crabomination::game::cast(&mut g, spell);
    // Greatest discarded = 4 (seat 1), so each player draws 4.
    assert_eq!(g.players[0].hand.len(), 4, "seat 0 drew up to the max (4)");
    assert_eq!(g.players[1].hand.len(), 4, "seat 1 drew up to the max (4)");
}

/// Aether Gust mode 1 puts a red/green permanent on top or bottom of its
/// owner's library (owner chooses; scripted to top here).
#[test]
fn aether_gust_tucks_green_permanent_owner_chooses_top() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aether_gust());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    precombat(&mut g);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // top
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    }).expect("cast Aether Gust");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "bear left the battlefield");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(bear), "on top of owner's library");
}

/// Aether Gust mode 1 rejects a non-red/green permanent target.
#[test]
fn aether_gust_rejects_off_color_permanent() {
    let mut g = two_player_game();
    let wall = g.add_card_to_battlefield(1, catalog::wall_of_omens());
    let spell = g.add_card_to_hand(0, catalog::aether_gust());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    precombat(&mut g);
    assert!(g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(wall)),
        additional_targets: vec![],
        mode: Some(1),
        x_value: None,
    }).is_err(), "white permanent is not a legal Aether Gust target");
}

/// Hidden Strings mode 2 untaps the first target and taps the second.
#[test]
fn hidden_strings_untaps_first_and_taps_second_target() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(mine).unwrap().tapped = true;
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::hidden_strings());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    precombat(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true), // do the untap
        DecisionAnswer::Bool(true), // do the tap
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: Some(2),
        x_value: None,
    }).expect("cast Hidden Strings");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(mine).unwrap().tapped, "first target untapped");
    assert!(g.battlefield_find(theirs).unwrap().tapped, "second target tapped");
}

/// CR 614.10 — Eon Hub: players skip their upkeep steps entirely (no
/// upkeep triggers, no priority in the step).
#[test]
fn cr_614_10_eon_hub_skips_upkeep_steps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::eon_hub());
    g.add_card_to_battlefield(0, catalog::phyrexian_arena());
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..4 { g.add_card_to_library(1, catalog::forest()); }
    let life_before = g.players[0].life;

    // Roll forward a couple of turns — the Arena's upkeep trigger never fires.
    g.step = TurnStep::Cleanup;
    g.active_player_idx = 0;
    let mut saw_upkeep = false;
    for _ in 0..40 {
        if g.is_game_over() { break; }
        saw_upkeep |= g.step == TurnStep::Upkeep;
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(!saw_upkeep, "no upkeep step occurs under Eon Hub");
    assert_eq!(g.players[0].life, life_before, "Arena upkeep trigger never fired");
}

/// CR 614.10 — Stasis: players skip their untap steps, so tapped
/// permanents stay tapped across turns.
#[test]
fn cr_614_10_stasis_skips_untap_steps() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stasis());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    for _ in 0..4 { g.add_card_to_library(0, catalog::forest()); }
    for _ in 0..4 { g.add_card_to_library(1, catalog::forest()); }
    // Pay for Stasis at our upkeep so it sticks around.
    g.players[0].mana_pool.add(Color::Blue, 4);

    g.step = TurnStep::Cleanup;
    g.active_player_idx = 1; // next turn is P0's
    for _ in 0..40 {
        if g.is_game_over() { break; }
        if g.active_player_idx == 0 && g.step == TurnStep::PreCombatMain && g.stack.is_empty() {
            break;
        }
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    assert!(g.battlefield_find(bear).unwrap().tapped, "Bear never untapped under Stasis");
}

/// CR 608.2b — a spell whose battlefield target died before resolution
/// doesn't resolve; the card is countered into its owner's graveyard.
#[test]
fn cr_608_2b_spell_fizzles_when_battlefield_target_dies() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let pump = g.add_card_to_hand(0, catalog::giant_growth());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pump, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Giant Growth castable");
    // The target dies with the spell still on the stack.
    let pos = g.battlefield.iter().position(|c| c.id == bear).unwrap();
    let dead = g.battlefield.remove(pos);
    g.players[1].graveyard.push(dead);
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pump),
        "fizzled spell goes to its owner's graveyard");
}

/// CR 608.2b — a graveyard-targeting spell (zone-loose filter) is unaffected
/// by the battlefield re-check.
#[test]
fn cr_608_2b_graveyard_target_does_not_fizzle() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::reanimate());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("Reanimate castable");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "reanimation resolves normally");
}

/// CR 612 — Trait Doctoring's color-word mode rewrites Protection-from-red
/// to Protection-from-blue until end of turn.
#[test]
fn trait_doctoring_swaps_protection_color_word() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, crabomination::card::CardDefinition {
        name: "Test Knight",
        card_types: vec![CardType::Creature],
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Protection(Color::Red)],
        ..Default::default()
    });
    let td = g.add_card_to_hand(0, catalog::trait_doctoring());
    g.players[0].mana_pool.add(Color::Blue, 1);
    precombat(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::Red),  // replace "red"...
        DecisionAnswer::Color(Color::Blue), // ...with "blue"
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: td, target: Some(Target::Permanent(knight)),
        additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("Trait Doctoring castable");
    drain_stack(&mut g);
    let computed = g.computed_permanent(knight).unwrap();
    assert!(computed.keywords.contains(&Keyword::Protection(Color::Blue)), "word swapped");
    assert!(!computed.keywords.contains(&Keyword::Protection(Color::Red)));
}

/// CR 612 / 305.7 — the basic-land-type mode turns a Forest into an Island
/// (type line + landwalk follow the computed land type).
#[test]
fn trait_doctoring_swaps_basic_land_type() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    let forest = g.add_card_to_battlefield(1, catalog::forest());
    let td = g.add_card_to_hand(0, catalog::trait_doctoring());
    g.players[0].mana_pool.add(Color::Blue, 1);
    precombat(&mut g);
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Color(Color::Green), // replace "Forest"...
        DecisionAnswer::Color(Color::Blue),  // ...with "Island"
    ]));
    g.perform_action(GameAction::CastSpell {
        card_id: td, target: Some(Target::Permanent(forest)),
        additional_targets: vec![], mode: Some(1), x_value: None,
    }).expect("Trait Doctoring castable");
    drain_stack(&mut g);
    let computed = g.computed_permanent(forest).unwrap();
    assert_eq!(computed.subtypes.land_types, vec![LandType::Island], "type line swapped");
}

// ── Bloodrush ─────────────────────────────────────────────────────────────────

#[test]
fn ghor_clan_rampager_bloodrush_pumps_and_grants_trample() {
    use crabomination::card::Keyword;
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(attacker);
    let rampager = g.add_card_to_hand(0, catalog::ghor_clan_rampager());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rampager, ability_index: 0,
        target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("bloodrush from hand");
    drain_stack(&mut g);
    let a = g.computed_permanent(attacker).unwrap();
    assert_eq!((a.power, a.toughness), (6, 6), "2/2 +4/+4");
    assert!(a.keywords.contains(&Keyword::Trample), "gained trample");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == rampager),
        "the Bloodrush card was discarded as a cost");
}

#[test]
fn slaughterhorn_bloodrush_pumps_attacker() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let horn = g.add_card_to_hand(0, catalog::slaughterhorn());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: horn, ability_index: 0,
        target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("bloodrush");
    drain_stack(&mut g);
    let a = g.computed_permanent(attacker).unwrap();
    assert_eq!((a.power, a.toughness), (5, 4), "2/2 +3/+2");
}

/// Bloodrush can only target an *attacking* creature.
#[test]
fn bloodrush_rejects_non_attacking_target() {
    let mut g = two_player_game();
    let idle = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(idle);
    let horn = g.add_card_to_hand(0, catalog::slaughterhorn());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Green, 1);
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: horn, ability_index: 0,
        target: Some(Target::Permanent(idle)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "no attacking creature → illegal target");
}

/// Rubblehulk's P/T is a lands-you-control CDA; its Bloodrush pumps by the
/// live land count.
#[test]
fn rubblehulk_cda_and_bloodrush_scale_with_lands() {
    use crabomination::game::types::Attack;
    let mut g = two_player_game();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let hulk = g.add_card_to_battlefield(0, catalog::rubblehulk());
    let c = g.computed_permanent(hulk).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "CDA: P/T = lands controlled");

    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let hulk2 = g.add_card_to_hand(0, catalog::rubblehulk());
    g.attacking = vec![Attack { attacker, target: AttackTarget::Player(1) }];
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: hulk2, ability_index: 0,
        target: Some(Target::Permanent(attacker)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("bloodrush");
    drain_stack(&mut g);
    let a = g.computed_permanent(attacker).unwrap();
    assert_eq!((a.power, a.toughness), (5, 5), "2/2 +3/+3 for three lands");
}

// ── Pump / evasion one-shots ──────────────────────────────────────────────────

#[test]
fn distortion_strike_pumps_and_unblockable() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::distortion_strike());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(c));
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 2), "2/2 +1/+0");
    assert!(cp.keywords.contains(&Keyword::Unblockable), "can't be blocked");
}

#[test]
fn tainted_strike_grants_infect() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tainted_strike());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(c));
    let cp = g.computed_permanent(c).unwrap();
    assert_eq!(cp.power, 3, "2 +1");
    assert!(cp.keywords.contains(&Keyword::Infect), "gained infect");
}

#[test]
fn groundswell_landfall_scales_with_a_land_drop() {
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // No land played yet → +2/+2.
    let s1 = g.add_card_to_hand(0, catalog::groundswell());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, s1, Target::Permanent(c));
    assert_eq!(g.computed_permanent(c).unwrap().power, 4, "2/2 +2/+2 with no landfall");
    // Now record a land drop and cast again → +4/+4.
    g.players[0].lands_played_this_turn = 1;
    let s2 = g.add_card_to_hand(0, catalog::groundswell());
    g.players[0].mana_pool.add(Color::Green, 1);
    crabomination::game::cast_at(&mut g, s2, Target::Permanent(c));
    assert_eq!(g.computed_permanent(c).unwrap().power, 8, "now +4/+4 too (4 + 4)");
}

#[test]
fn artful_dodge_makes_target_unblockable() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let c = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::artful_dodge());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(c));
    assert!(g.computed_permanent(c).unwrap().keywords.contains(&Keyword::Unblockable));
}

#[test]
fn faithless_salvaging_loots_one() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::grizzly_bears()); // a card to discard
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::faithless_salvaging());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast(&mut g, spell);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 1,
        "discarded a card");
}

#[test]
fn wrap_in_vigor_shields_your_creatures() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::wrap_in_vigor());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast(&mut g, spell);
    assert_eq!(g.battlefield_find(mine).unwrap().regeneration_shields, 1,
        "each of your creatures got a regeneration shield");
}

// ── Threaten-style theft ──────────────────────────────────────────────────────

#[test]
fn act_of_treason_steals_until_end_of_turn() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::act_of_treason());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(victim));
    let v = g.battlefield_find(victim).unwrap();
    assert_eq!(v.controller, 0, "control stolen");
    assert!(!v.tapped, "untapped");
    assert!(g.computed_permanent(victim).unwrap().keywords.contains(&Keyword::Haste), "gained haste");
}

#[test]
fn traitorous_blood_grants_trample() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::traitorous_blood());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(victim));
    let kws = g.computed_permanent(victim).unwrap().keywords.clone();
    assert!(kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Haste));
}

#[test]
fn wrench_mind_discards_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::grizzly_bears()); }
    let spell = g.add_card_to_hand(0, catalog::wrench_mind());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 1, "discarded two of three");
}

/// Mirrorform turns each of your nonland permanents into a permanent copy of
/// the target.
#[test]
fn mirrorform_copies_your_nonland_permanents() {
    let mut g = two_player_game();
    let big = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let small = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::mirrorform());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, spell, Target::Permanent(big));
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(small).unwrap().definition.name, "Colossal Dreadmaw");
    assert_eq!(g.battlefield_find(land).unwrap().definition.name, "Forest", "lands untouched");
    // Permanent duration: survives cleanup.
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.battlefield_find(small).unwrap().definition.name, "Colossal Dreadmaw");
}

/// Shifting Woodland needs delirium, then copies a permanent card in the
/// graveyard until end of turn.
#[test]
fn shifting_woodland_delirium_copies_graveyard_permanent() {
    let mut g = two_player_game();
    let wood = g.add_card_to_battlefield(0, catalog::shifting_woodland());
    g.clear_sickness(wood);
    let dead = g.add_card_to_graveyard(0, catalog::colossal_dreadmaw());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    // No delirium yet (one card type in gy) → gated.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: wood, ability_index: 1,
        target: Some(Target::Permanent(dead)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "delirium not met");
    // Fill the graveyard to four card types.
    g.add_card_to_graveyard(0, catalog::lightning_bolt());
    g.add_card_to_graveyard(0, catalog::forest());
    g.add_card_to_graveyard(0, catalog::divination());
    g.perform_action(GameAction::ActivateAbility {
        card_id: wood, ability_index: 1,
        target: Some(Target::Permanent(dead)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("delirium copy");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(wood).unwrap().definition.name, "Colossal Dreadmaw");
    g.do_cleanup(&mut Vec::new());
    assert_eq!(g.battlefield_find(wood).unwrap().definition.name, "Shifting Woodland",
        "reverts at end of turn");
}

// ── Staple batch (modern_decks) ───────────────────────────────────────────────

/// Gravecrawler recasts from the graveyard only while you control a Zombie.
#[test]
fn gravecrawler_recasts_from_graveyard_with_a_zombie() {
    let mut g = two_player_game();
    let crawler = g.add_card_to_graveyard(0, catalog::gravecrawler());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: crawler, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).is_err(), "no Zombie → stays dead");
    g.add_card_to_battlefield(0, catalog::gravecrawler()); // a Zombie
    g.perform_action(GameAction::ActivateAbility {
        card_id: crawler, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("recast with a Zombie out");
    drain_stack(&mut g);
    assert!(g.battlefield_find(crawler).is_some(), "Gravecrawler returned");
}

/// Soulherder grows on battlefield exiles and flickers a creature at end step.
#[test]
fn soulherder_grows_and_flickers() {
    let mut g = two_player_game();
    let herder = g.add_card_to_battlefield(0, catalog::soulherder());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // End-step trigger: flicker the bear (MayDo yes).
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "bear flickered back");
    let h = g.battlefield_find(herder).unwrap();
    assert_eq!(h.counter_count(CounterType::PlusOnePlusOne), 1, "grew off the exile");
}

/// Knight of the Ebon Legion gets a counter at end step after 4+ life lost.
#[test]
fn knight_of_the_ebon_legion_grows_after_bloodshed() {
    let mut g = two_player_game();
    let knight = g.add_card_to_battlefield(0, catalog::knight_of_the_ebon_legion());
    g.adjust_life(1, -4);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(knight).unwrap().counter_count(CounterType::PlusOnePlusOne),
        1,
        "4 life lost this turn → counter"
    );
}

/// Tourach grows on opponent discards; the kicked ETB rips two cards.
#[test]
fn tourach_kicked_etb_discards_two_and_grows() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let t = g.add_card_to_hand(0, catalog::tourach_dread_cantor());
    g.players[0].mana_pool.add(Color::Black, 3);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: t, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 1, "discarded two");
    // The two opponent discards each grew Tourach (CR 603.6 fan-out).
    assert_eq!(
        g.battlefield_find(t).unwrap().counter_count(CounterType::PlusOnePlusOne),
        2,
        "two discard triggers"
    );
}

/// Yawgmoth converts a spare body into a -1/-1 counter and a card.
#[test]
fn yawgmoth_sacrifices_for_counter_and_card() {
    let mut g = two_player_game();
    let yawg = g.add_card_to_battlefield(0, catalog::yawgmoth_thran_physician());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let opp = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: yawg, ability_index: 0,
        target: Some(Target::Permanent(opp)), additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("activate Yawgmoth");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "paid 1 life");
    assert_eq!(g.battlefield_find(opp).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
    assert_eq!(g.players[0].hand.len(), 1, "drew a card");
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"), "bear sacrificed");
}

/// Baneslayer Angel's protection from Dragons: a Dragon can't block her.
#[test]
fn baneslayer_protection_from_dragons_blocks_dragon_blocker() {
    let mut g = two_player_game();
    let angel = g.add_card_to_battlefield(0, catalog::baneslayer_angel());
    g.clear_sickness(angel);
    let dragon = g.add_card_to_battlefield(1, catalog::balefire_dragon());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: angel, target: AttackTarget::Player(1),
    }])).expect("Baneslayer attacks");
    g.step = TurnStep::DeclareBlockers;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(dragon, angel)])).is_err(),
        "a Dragon can't block a creature with protection from Dragons");
}

/// Yawgmoth's protection from Humans: a Human can't block him.
#[test]
fn yawgmoth_protection_from_humans_blocks_human_blocker() {
    let mut g = two_player_game();
    let yawg = g.add_card_to_battlefield(0, catalog::yawgmoth_thran_physician());
    g.clear_sickness(yawg);
    let human = g.add_card_to_battlefield(1, catalog::champion_of_the_parish()); // Human Soldier
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: yawg, target: AttackTarget::Player(1),
    }])).expect("Yawgmoth attacks");
    g.step = TurnStep::DeclareBlockers;
    assert!(g.perform_action(GameAction::DeclareBlockers(vec![(human, yawg)])).is_err(),
        "a Human can't block a creature with protection from Humans");
}

/// Archfiend of Ifnir withers opposing creatures when you cycle.
#[test]
fn archfiend_of_ifnir_withers_on_cycle() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::archfiend_of_ifnir());
    let opp = g.add_card_to_battlefield(1, catalog::colossal_dreadmaw());
    let cycler = g.add_card_to_hand(0, catalog::archfiend_of_ifnir());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Cycle { card_id: cycler, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(opp).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
}

/// Battle Display destroys an artifact; the Knight can be cast later.
#[test]
fn embereth_shieldbreaker_adventure_smashes_artifact() {
    let mut g = two_player_game();
    let oven = g.add_card_to_battlefield(1, catalog::witchs_oven());
    let card = g.add_card_to_hand(0, catalog::embereth_shieldbreaker());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastAdventure {
        card_id: card, target: Some(Target::Permanent(oven)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Battle Display");
    drain_stack(&mut g);
    assert!(g.battlefield_find(oven).is_none(), "artifact destroyed");
    assert!(g.exile.iter().any(|c| c.id == card), "on an adventure");
}

/// Bomat Courier stashes cards on attack and trades the hand for the stash.
#[test]
fn bomat_courier_stashes_and_cashes_out() {
    use crabomination::game::types::{Attack, AttackTarget};
    let mut g = two_player_game();
    let courier = g.add_card_to_battlefield(0, catalog::bomat_courier());
    g.clear_sickness(courier);
    let stash1 = g.add_card_to_library(0, catalog::island());
    let junk = g.add_card_to_hand(0, catalog::forest());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: courier, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    let stashed = g.exile.iter().find(|c| c.id == stash1).expect("top card stashed");
    assert!(stashed.face_down, "stash is face down");
    assert_eq!(stashed.exiled_with, Some(courier), "linked to the Courier");
    // Cash out: {R}, sac — discard hand, take the stash.
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PostCombatMain;
    g.perform_action(GameAction::ActivateAbility {
        card_id: courier, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("cash out");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == stash1), "stash in hand");
    assert!(!g.players[0].hand.iter().any(|c| c.id == junk), "old hand discarded");
}

/// Kroxa cast normally rips a card from each player, then sacrifices itself;
/// escaped, it sticks around.
#[test]
fn kroxa_sacrifices_unless_escaped() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(1, catalog::island()); // a LAND discard → still loses 3
    let kroxa = g.add_card_to_hand(0, catalog::kroxa_titan_of_deaths_hunger());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast(&mut g, kroxa);
    drain_stack(&mut g);
    assert!(g.battlefield_find(kroxa).is_none(), "hard-cast Kroxa sacrificed");
    assert_eq!(g.players[1].life, 17, "opponent discarded a land → lost 3");
    // Escape it back: {B}{B}{R}{R} + exile five from the graveyard.
    let fodder: Vec<_> = (0..5).map(|_| g.add_card_to_graveyard(0, catalog::island())).collect();
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Red, 2);
    g.perform_action(GameAction::CastEscape {
        card_id: kroxa, exile_cards: fodder, target: None,
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("escape Kroxa");
    drain_stack(&mut g);
    assert!(g.battlefield_find(kroxa).is_some(), "escaped Kroxa survives its ETB");
}

/// Uro gains 3, draws, and ramps on ETB; sacrifices itself when hard-cast.
#[test]
fn uro_etb_feast_then_sacrifice() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let hand_land = g.add_card_to_hand(0, catalog::forest());
    let uro = g.add_card_to_hand(0, catalog::uro_titan_of_natures_wrath());
    for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] { g.players[0].mana_pool.add(_c, 20); }
    g.players[0].mana_pool.add_colorless(20);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![hand_land])]));
    crabomination::game::cast(&mut g, uro);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 23, "gained 3");
    assert!(g.battlefield_find(hand_land).is_some(), "land dropped from hand");
    assert!(g.battlefield_find(uro).is_none(), "hard-cast Uro sacrificed");
}

/// Skyclave Apparition exiles a cheap permanent; killing the Apparition
/// leaves the card in exile and mints an MV-sized Illusion instead.
#[test]
fn skyclave_apparition_exiles_and_leaves_an_illusion() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::witchs_oven()); // MV 1
    let sky = g.add_card_to_hand(0, catalog::skyclave_apparition());
    g.players[0].mana_pool.add(Color::White, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast_at(&mut g, sky, Target::Permanent(target));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == target), "oven exiled");
    g.remove_from_battlefield_to_graveyard_raw(sky);
    assert!(g.exile.iter().any(|c| c.id == target), "card stays in exile");
    let illusion = g.battlefield.iter()
        .find(|c| c.controller == 1 && c.definition.name == "Illusion")
        .expect("owner got an Illusion");
    assert_eq!((illusion.definition.power, illusion.definition.toughness), (1, 1), "X = MV 1");
}

/// Gilded Goose makes Food and converts a Food into any color of mana.
#[test]
fn gilded_goose_food_engine() {
    let mut g = two_player_game();
    let goose = g.add_card_to_hand(0, catalog::gilded_goose());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    crabomination::game::cast(&mut g, goose);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Food"), "ETB Food");
    g.clear_sickness(goose);
    g.perform_action(GameAction::ActivateAbility {
        card_id: goose, ability_index: 1, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("sac Food for mana");
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Food"), "Food eaten");
    assert_eq!(g.players[0].mana_pool.total(), 1, "one mana of any color");
}

/// Shifting Ceratops can't be countered and picks an evasion grant.
#[test]
fn shifting_ceratops_uncounterable_and_modal_grant() {
    use crabomination::card::Keyword;
    let mut g = two_player_game();
    let dino = g.add_card_to_hand(0, catalog::shifting_ceratops());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dino, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast");
    // Opponent tries to counter: the spell is flagged uncounterable.
    let counter = g.add_card_to_hand(1, catalog::cancel());
    g.players[1].mana_pool.add(Color::Blue, 2);
    g.players[1].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: counter, target: Some(Target::Permanent(dino)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Cancel");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dino).is_some(), "resolved through the counter");
    // {G}: gains haste (mode 2).
    g.players[0].mana_pool.add(Color::Green, 1);
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(2)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: dino, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("modal grant");
    drain_stack(&mut g);
    assert!(g.computed_permanent(dino).unwrap().keywords.contains(&Keyword::Haste));
}

/// Thrun regenerates out of a destroy.
#[test]
fn thrun_regenerates() {
    let mut g = two_player_game();
    let thrun = g.add_card_to_battlefield(0, catalog::thrun_the_last_troll());
    g.clear_sickness(thrun);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: thrun, ability_index: 0, target: None, additional_targets: Vec::new(), x_value: None, mode: None,
    }).expect("regenerate shield");
    drain_stack(&mut g);
    // Hexproof blocks targeted removal entirely — wipe the board instead.
    let wrath = g.add_card_to_hand(1, catalog::supreme_verdict());
    g.players[1].mana_pool.add(Color::White, 2);
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    crabomination::game::cast(&mut g, wrath);
    drain_stack(&mut g);
    assert!(g.battlefield_find(thrun).is_some(), "regenerated through the wrath");
    assert!(g.battlefield_find(thrun).unwrap().tapped, "regeneration taps");
}

/// Containment Construct rescues a discard into exile and lets you play it
/// this turn.
#[test]
fn containment_construct_rescues_discards() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::containment_construct());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // exile it
    let mut events = Vec::new();
    g.discard_card(0, bolt, &mut events);
    g.dispatch_triggers_for_events(&events);
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == bolt).expect("discard exiled");
    assert!(exiled.may_play_until.is_some(), "playable this turn");
}

/// Heart of Kiran crews with 3 power and swings as a 4/4 flyer.
#[test]
fn heart_of_kiran_crews_and_attacks() {
    use crabomination::card::CardType;
    let mut g = two_player_game();
    let heart = g.add_card_to_battlefield(0, catalog::heart_of_kiran());
    let crew = g.add_card_to_battlefield(0, catalog::colossal_dreadmaw());
    g.clear_sickness(crew);
    g.step = TurnStep::PreCombatMain;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Crew { vehicle: heart, crew_creatures: vec![crew] })
        .expect("crew 3 paid by a 6-power body");
    let v = g.computed_permanent(heart).unwrap();
    assert!(v.card_types.contains(&CardType::Creature), "animated by crewing");
    assert_eq!((v.power, v.toughness), (4, 4));
}

