//! CR 400.7 / 607 — a card that leaves exile is a new object: the exiler's
//! "until this leaves" link (`exiled_by`) does not follow it into its next
//! zone, nor into a later exile. Plus a CR 608.2 answer-log leak.

use crabomination::catalog;
use crabomination::effect::{Effect, PlayerRef, Selector, ZoneDest};
use crabomination::game::effects::EffectContext;
use crabomination::game::types::{Target, TurnStep};
use crabomination::game::*;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    g
}

fn resolve(g: &mut GameState, source: crabomination::card::CardId, target: Option<Target>, e: &Effect) {
    let events = g.resolve_effect(e, &EffectContext::for_ability(source, 0, target)).expect("resolves");
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

/// Foreboding Steamboat exiles a Bear, its attack trigger puts the Bear into
/// the graveyard, and a later exile of that card is a plain exile: the
/// Steamboat leaving doesn't bring it back. (Two 6-seat debug pods, seed
/// 130026 games 10 and 27, found the card still "exiled until the Steamboat
/// leaves" with the Steamboat long gone.)
#[test]
fn cr_400_7_a_card_out_of_exile_loses_its_exilers_link() {
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::foreboding_steamboat().triggered_abilities[0].effect.clone();
    resolve(&mut g, boat, None, &etb);
    assert!(g.exile.iter().any(|c| c.id == bear && c.exiled_by.is_some()));
    let to_gy = Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Graveyard };
    resolve(&mut g, boat, None, &to_gy);
    let in_gy = g.players[1].graveyard.iter().find(|c| c.id == bear).expect("in the graveyard");
    assert!(in_gy.exiled_by.is_none() && in_gy.exiled_with.is_none());
    resolve(&mut g, boat, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, boat, None, &Effect::Destroy { what: Selector::ExactObjects(vec![boat]) });
    assert!(g.exile.iter().any(|c| c.id == bear), "a plain exile: the Steamboat leaving doesn't return it");
}

/// Hostage Taker exiles a Bear; the Bear comes back by another route, is
/// exiled again by something else, and Hostage Taker leaving must not return
/// it (CR 610.3c reads only the object it exiled).
#[test]
fn cr_400_7_a_reexiled_card_does_not_return_with_its_old_exiler() {
    let mut g = main_phase();
    let taker = g.add_card_to_battlefield(0, catalog::hostage_taker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::hostage_taker().triggered_abilities[0].effect.clone();
    resolve(&mut g, taker, Some(Target::Permanent(bear)), &etb);
    assert!(g.exile.iter().any(|c| c.id == bear), "the Taker holds it");
    resolve(
        &mut g,
        taker,
        None,
        &Effect::Move {
            what: Selector::ExactObjects(vec![bear]),
            to: ZoneDest::Battlefield { controller: PlayerRef::Seat(1), tapped: false },
        },
    );
    assert!(g.battlefield_find(bear).is_some_and(|c| c.exiled_by.is_none()));
    resolve(&mut g, taker, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, taker, None, &Effect::Destroy { what: Selector::ExactObjects(vec![taker]) });
    assert!(g.battlefield_find(bear).is_none(), "the Taker's leave returns only what it holds");
}

/// CR 400.7 / 607 — a card cast from exile leaves the link behind with the
/// exile: the Bear a Steamboat held resolves as a new permanent, and when a
/// later effect exiles it, it is not "exiled with" the Steamboat again.
#[test]
fn cr_400_7_a_card_cast_from_exile_is_not_exiled_with_its_old_exiler() {
    use crabomination::game::types::StackItem;
    let mut g = main_phase();
    let boat = g.add_card_to_battlefield(0, catalog::foreboding_steamboat());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    let mut card = g.players[0].hand.pop().unwrap();
    assert_eq!(card.id, bear);
    card.exiled_with = Some(boat);
    card.cast_from_exile = true;
    g.stack.push(StackItem::Spell {
        card: Box::new(card),
        caster: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: 0,
        converged_value: 0,
        mana_spent: 2,
        uncounterable: false,
    });
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "the Bear resolved");
    g.remove_from_battlefield_to_exile(bear);
    let exiled = g.exile.iter().find(|c| c.id == bear).expect("exiled");
    assert_eq!(exiled.exiled_with, None, "a plain exile, not the Steamboat's");
}

/// CR 608.2 — an asked pick whose candidates all left while it waited spends
/// its answer. Yuffie took Lightning Greaves "for as long as you control
/// Yuffie" and asked which Equipment to attach; the steal ended before the
/// answer came back (Clever Concealment phased Yuffie out), the re-run found
/// no Equipment, and the stashed pick leaked (a strict 6-seat debug pod, seed
/// 201019 game 18). Nextest runs each test in its own process, so the strict
/// leak check (read once) is switched on here.
#[test]
fn cr_608_2_a_pick_whose_candidates_left_spends_its_answer() {
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::GameAction;
    // SAFETY: set before any thread reads the environment.
    unsafe { std::env::set_var("CRAB_ANSWER_LOG", "strict") };
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let greaves = g.add_card_to_battlefield(1, catalog::lightning_greaves());
    let yuffie = g.add_card_to_hand(0, catalog::yuffie_materia_hunter());
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell { card_id: yuffie, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Yuffie");
    let mut asked = false;
    for _ in 0..20 {
        if let Some(d) = g.pending_decision.as_ref() {
            let answer = match &d.decision {
                Decision::ChooseTarget { .. } => DecisionAnswer::Target(Target::Permanent(greaves)),
                Decision::OptionalTrigger { .. } => DecisionAnswer::Bool(true),
                Decision::ChooseCards { .. } => {
                    // The steal ends before the answer comes back.
                    g.battlefield_find_mut(greaves).unwrap().controller = 1;
                    asked = true;
                    DecisionAnswer::Cards(vec![greaves])
                }
                other => panic!("unexpected ask {other:?}"),
            };
            g.submit_decision(answer).expect("answer");
            continue;
        }
        if g.stack.is_empty() {
            break;
        }
        g.resolve_top_of_stack().expect("resolve");
    }
    assert!(asked, "the Equipment pick was asked");
    assert!(g.pending_decision.is_none());
    assert_eq!(g.battlefield_find(greaves).and_then(|c| c.attached_to), None, "nothing to attach");
}

/// CR 608.2 / 800.4a — Florian's look counts the life the caster's opponents
/// lost; a seat that leaves while the pick waits takes its loss out of the
/// count, the re-run looks at nothing, and the logged pick is spent rather
/// than leaked (a strict 5-seat debug pod with concessions, seed 203023
/// game 5).
#[test]
fn cr_608_2_a_look_that_shrinks_to_nothing_spends_its_pick() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::effect::{LookExileGrant, Value};
    use crabomination::game::types::GameAction;
    // SAFETY: set before any thread reads the environment.
    unsafe { std::env::set_var("CRAB_ANSWER_LOG", "strict") };
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].wants_ui = true;
    let top = g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[1].life_lost_this_turn = 2;
    let look = g.add_card_to_hand(0, CardDefinition {
        name: "Florian's Look",
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookTopExileOneMayPlay {
            count: Value::TotalLifeLostThisTurn(PlayerRef::EachOpponent),
            who: PlayerRef::You,
            grant: LookExileGrant::PlayThisTurn,
        },
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: look, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    assert!(matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::ChooseCards { .. })));
    g.concede(1);
    g.submit_decision(DecisionAnswer::Cards(vec![top])).expect("answer");
    assert!(g.pending_decision.is_none());
    assert!(g.exile.iter().all(|c| c.id != top), "nothing was looked at, so nothing is exiled");
}

/// The cast route: Hostage Taker exiles a Bear and its controller casts it.
/// The permanent it becomes carries no exile link, so exiled again later and
/// the Taker gone, it stays exiled.
#[test]
fn cr_400_7_a_card_cast_out_of_exile_sheds_its_exile_links() {
    use crabomination::game::types::GameAction;
    use crabomination::mana::Color;
    let mut g = main_phase();
    let taker = g.add_card_to_battlefield(0, catalog::hostage_taker());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let etb = catalog::hostage_taker().triggered_abilities[0].effect.clone();
    resolve(&mut g, taker, Some(Target::Permanent(bear)), &etb);
    g.players[0].mana_pool.add(Color::Black, 2);
    g.perform_action(GameAction::CastFromZoneWithoutPaying {
        card_id: bear,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the hostage");
    drain_stack(&mut g);
    let c = g.battlefield_find(bear).expect("on the battlefield");
    assert!(c.exiled_by.is_none() && c.exiled_with.is_none(), "{:?} / {:?}", c.exiled_by, c.exiled_with);
    resolve(&mut g, taker, None, &Effect::Move { what: Selector::ExactObjects(vec![bear]), to: ZoneDest::Exile });
    resolve(&mut g, taker, None, &Effect::Destroy { what: Selector::ExactObjects(vec![taker]) });
    assert!(g.exile.iter().any(|c| c.id == bear));
}

/// CR 608.2 — a resumed asker whose question went moot while it waited spends
/// its stashed answer, whatever arm it is. "You may discard a card. If you do,
/// draw" was answered yes after the hand emptied; the re-run's may stops
/// offering (an unpayable cost) and the yes used to leak into the next may in
/// the same resolution, which then gained 5 life without asking.
#[test]
fn cr_608_2_a_moot_asker_does_not_hand_its_answer_to_the_next() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::effect::Value;
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let fodder = g.add_card_to_hand(0, catalog::grizzly_bears());
    let may = |body: Effect| Effect::MayDo { description: "may".into(), body: Box::new(body) };
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "Two Mays",
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            may(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
            may(Effect::GainLife { who: Selector::You, amount: Value::Const(5) }),
        ]),
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    assert!(matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::OptionalTrigger { .. })));
    // The hand empties before the answer comes back.
    let pos = g.players[0].hand.iter().position(|c| c.id == fodder).unwrap();
    let card = g.players[0].hand.remove(pos);
    g.players[0].graveyard.push(card);
    let life = g.players[0].life;
    g.submit_decision(DecisionAnswer::Bool(true)).expect("answer");
    assert!(
        matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::OptionalTrigger { .. })),
        "the second may asks for itself"
    );
    g.submit_decision(DecisionAnswer::Bool(false)).expect("decline");
    assert!(g.pending_decision.is_none());
    assert_eq!(g.players[0].life, life, "declined, so no life");
}

/// CR 115.3 / 601.2c — "destroy X target artifacts and/or enchantments" is
/// one instance of "target": X slots, each a different object. A prompting
/// seat's Heliod's Intervention at X = 2 was offered a slot past X, and the
/// first artifact again, without end — a default-pilot 4-seat strict pod
/// (seed 250011 game 21) appended the same Sol Ring 255 times and overflowed
/// the slot count.
#[test]
fn cr_115_3_x_target_slots_stop_at_x_and_never_repeat() {
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let rocks: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::sol_ring())).collect();
    let heliod = g.add_card_to_hand(0, catalog::heliods_intervention());
    let cast = |g: &mut GameState, extra: Vec<Target>| {
        g.players[0].mana_pool.add(crabomination::mana::Color::White, 2);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::CastSpell {
            card_id: heliod,
            target: Some(Target::Permanent(rocks[0])),
            additional_targets: extra,
            mode: Some(0),
            x_value: Some(2),
        })
    };
    assert!(cast(&mut g, vec![Target::Permanent(rocks[0])]).is_err(), "the same Sol Ring twice is not two targets");
    g.players[0].mana_pool = Default::default();
    cast(&mut g, vec![]).expect("cast");
    let Some(Decision::ChooseTarget { legal, .. }) = g.pending_decision.as_ref().map(|d| d.decision.clone()) else {
        panic!("the second slot is asked");
    };
    assert!(!legal.contains(&Target::Permanent(rocks[0])), "the first pick is not offered again");
    g.submit_decision(DecisionAnswer::Target(Target::Permanent(rocks[1]))).expect("second target");
    assert!(g.pending_decision.is_none(), "no third slot at X = 2");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rocks[0]).is_none() && g.battlefield_find(rocks[1]).is_none());
    assert!(g.battlefield_find(rocks[2]).is_some());
}

/// CR 601.2c / 115.3 — "destroy X target artifacts" takes exactly X
/// different targets: X = 1 with three is not a cast. The target walker that
/// fills a bot's cast filled all sixteen slots, repeating the first artifact
/// once fresh ones ran out, and the cast path took any count at any X; once
/// repeats were refused, the bot stopped casting By Force at all.
#[test]
fn cr_601_2c_destroy_x_targets_takes_exactly_x_and_a_bot_still_casts_it() {
    use crabomination::game::types::GameAction;
    use crabomination::server::bot::{Bot, HeuristicBot};
    let mut g = main_phase();
    let rocks: Vec<_> = (0..3).map(|_| g.add_card_to_battlefield(1, catalog::sol_ring())).collect();
    let force = g.add_card_to_hand(0, catalog::by_force());
    for _ in 0..4 {
        g.add_card_to_battlefield(0, catalog::mountain());
    }
    let all: Vec<Target> = rocks.iter().map(|&r| Target::Permanent(r)).collect();
    let mut h = g.clone();
    let over = h.perform_action(GameAction::CastSpell {
        card_id: force, target: Some(all[0].clone()), additional_targets: all[1..].to_vec(), mode: None, x_value: Some(1),
    });
    assert!(over.is_err(), "three targets at X = 1");
    let mut cast = None;
    for _ in 0..12 {
        let Some(a) = HeuristicBot::new().next_action(&g, 0) else { break };
        if let GameAction::CastSpell { card_id, ref target, ref additional_targets, x_value, .. } = a
            && card_id == force
        {
            cast = Some((target.iter().chain(additional_targets).cloned().collect::<Vec<_>>(), x_value));
        }
        g.perform_action(a).expect("the bot's action is legal");
        if cast.is_some() {
            break;
        }
    }
    let (targets, x) = cast.expect("the bot casts By Force");
    assert_eq!(Some(targets.len() as u32), x, "one target per X");
    assert!(targets.iter().all(|t| targets.iter().filter(|u| *u == t).count() == 1), "no repeats");
}

/// CR 709.5 / 105.2 — a Room permanent has only its unlocked doors' mana
/// cost, so a locked Room is colorless and an unlocked red door makes it red.
/// The layer system seeded colors from both doors (a locked Spiked Corridor
/// // Torture Pit read red, and the requirement walker's colorless answer
/// tripped the colour gate's ratchet in a default-pilot 2-seat strict pod,
/// seed 261036 game 7).
#[test]
fn cr_709_5_a_locked_room_is_colorless_and_a_door_colors_it() {
    use crabomination::card::SelectionRequirement as R;
    use crabomination::mana::Color;
    let mut g = main_phase();
    let room = g.add_card_to_battlefield(0, catalog::spiked_corridor_torture_pit());
    let red = |g: &GameState| {
        (
            g.computed_permanent(room).expect("on the battlefield").colors.contains(Color::Red),
            g.evaluate_requirement_static(&R::HasColor(Color::Red), &Target::Permanent(room), 0, None),
        )
    };
    assert_eq!(red(&g), (false, false), "locked: no mana cost, no color");
    assert!(g.battlefield_find_mut(room).unwrap().unlock_room_door(false));
    assert_eq!(red(&g), (true, true), "the Corridor's {{R}} colors it");
}

/// CR 709.5 — a Room permanent has only its unlocked doors' names: a locked
/// Room has no name, and unlocking a door gives it that door's name only.
#[test]
fn cr_709_5_a_locked_door_has_no_name() {
    use crabomination::card::SelectionRequirement as R;
    let mut g = main_phase();
    let room = g.add_card_to_battlefield(0, catalog::spiked_corridor_torture_pit());
    let named = |g: &GameState, n: &str| {
        g.evaluate_requirement_static(&R::HasName(n.into()), &Target::Permanent(room), 0, None)
    };
    assert!(!named(&g, "Spiked Corridor") && !named(&g, "Torture Pit"), "locked: no name");
    assert!(g.battlefield_find_mut(room).unwrap().unlock_room_door(false));
    assert!(named(&g, "Spiked Corridor"), "the unlocked door's name");
    assert!(!named(&g, "Torture Pit"), "the locked door's name is still gone");
}

/// CR 709.5 — a static's scope over permanents reads a Room's unlocked
/// doors' mana value: "permanents with mana value 1 or greater have
/// hexproof" skips a locked Room (mana value 0) and covers it once a door
/// is unlocked.
#[test]
fn cr_709_5_a_static_scope_reads_a_rooms_unlocked_mana_value() {
    use crabomination::card::{CardDefinition, CardType, Keyword, SelectionRequirement as R};
    use crabomination::effect::{StaticAbility, StaticEffect};
    let mut g = main_phase();
    g.add_card_to_battlefield(0, CardDefinition {
        name: "Costly Ward",
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Permanents with mana value 1 or greater have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::ManaValueAtLeast(1).and(R::Enchantment)),
                keyword: Keyword::Hexproof,
            },
        }],
        ..Default::default()
    });
    let room = g.add_card_to_battlefield(0, catalog::spiked_corridor_torture_pit());
    let hexproof = |g: &GameState| g.computed_permanent(room).unwrap().keywords().contains(&Keyword::Hexproof);
    assert!(!hexproof(&g), "locked: mana value 0");
    assert!(g.battlefield_find_mut(room).unwrap().unlock_room_door(false));
    assert!(hexproof(&g), "the Corridor's {{R}}: mana value 1");
}

/// CR 709.4a — a split card has each half's name: Meddling Mage naming
/// "Hit" stops Hit // Run, where comparing the whole printed name let it
/// through.
#[test]
fn cr_709_4a_naming_one_half_of_a_split_card_names_the_card() {
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    let mage = g.add_card_to_battlefield(1, catalog::meddling_mage());
    g.battlefield_find_mut(mage).unwrap().named_card = Some("Hit".into());
    let hit = g.add_card_to_hand(0, catalog::hit_run());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 1);
    g.players[0].mana_pool.add(crabomination::mana::Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    let cast = g.perform_action(GameAction::CastSpell { card_id: hit, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None });
    assert!(matches!(cast, Err(GameError::SpellNameLocked)), "{cast:?}");
    // A restricted name ask ("a nonland card name") looks the half up too.
    let half = catalog::lookup_by_name("Run").map(|d| d.name);
    assert_eq!(half, Some("Hit // Run"));
}

/// CR 400.1 / 608.2 — an answer of the wrong shape to a paused resolution is
/// rejected and the ask stands: the resolving card is held off-zone until the
/// resolution resumes, and dropping the ask deleted it (a fuzzed 8-seat pod,
/// seed 487016, answered Trap the Trespassers' option with a mode).
#[test]
fn cr_400_1_a_wrong_shape_answer_leaves_the_resolution_paused() {
    use crabomination::card::{CardDefinition, CardType};
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::effect::Value;
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    g.players[0].wants_ui = true;
    let spell = g.add_card_to_hand(0, CardDefinition {
        name: "A May",
        card_types: vec![CardType::Sorcery],
        effect: Effect::MayDo {
            description: "may".into(),
            body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
        },
        ..Default::default()
    });
    g.perform_action(GameAction::CastSpell { card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    g.resolve_top_of_stack().expect("resolve");
    assert!(g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Mode(0))).is_err(), "a mode is not a yes/no");
    assert!(
        matches!(g.pending_decision.as_ref().map(|d| &d.decision), Some(Decision::OptionalTrigger { .. })),
        "the ask is posed again"
    );
    let life = g.players[0].life;
    g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Bool(true))).expect("answer");
    assert_eq!(g.players[0].life, life + 3);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == spell), "the spell finished resolving");
}

/// CR 603.2 — a cast trigger fires the moment the spell is cast, so a listener
/// the same resolution destroys after casting still triggered: a "whenever you
/// cast a spell, gain 2" creature sees the free cast before the sweep.
#[test]
fn cr_603_2_a_cast_listener_destroyed_later_in_the_resolution_saw_the_cast() {
    use crabomination::card::{CardDefinition, CardType, TriggeredAbility};
    use crabomination::effect::{EventKind, EventScope, EventSpec, Value};
    let mut g = main_phase();
    let watcher = g.add_card_to_battlefield(0, CardDefinition {
        name: "Cast Watcher",
        card_types: vec![CardType::Creature],
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    });
    let freebie = g.add_card_to_exile(0, CardDefinition {
        name: "Freebie",
        card_types: vec![CardType::Sorcery],
        effect: Effect::Noop,
        ..Default::default()
    });
    let life = g.players[0].life;
    let effect = Effect::Seq(vec![
        Effect::CastExiledFree { what: Selector::ExactObjects(vec![freebie]) },
        Effect::Destroy { what: Selector::ExactObjects(vec![watcher]) },
    ]);
    resolve(&mut g, watcher, None, &effect);
    assert!(g.battlefield_find(watcher).is_none(), "destroyed");
    assert_eq!(g.players[0].life, life + 2, "the cast was seen before the sweep");
}

/// CR 122.2 — Me, the Immortal keeps its counters outside a hand or library,
/// so cast from the graveyard it enters already carrying a keyword counter,
/// and the whole-board keyword gate must see it (its debug audit fired in a
/// bot's look-ahead in a fuzzed 5-seat pod, seed 469022: a vigilance counter
/// on Me after a cleanup had cleared the gate).
#[test]
fn cr_122_2_a_card_entering_with_its_keyword_counters_arms_the_board_gate() {
    use crabomination::card::Keyword;
    use crabomination::game::types::GameAction;
    use crabomination::mana::Color;
    let mut g = main_phase();
    let me = g.add_card_to_graveyard(0, catalog::me_the_immortal());
    g.players[0].graveyard.iter_mut().find(|c| c.id == me).unwrap().keyword_counters.add(Keyword::Vigilance, 1);
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_hand(0, catalog::island());
    let _ = g.do_cleanup(&mut Vec::new());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    for c in [Color::Green, Color::Blue, Color::Red] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFlashback { card_id: me, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast Me from the graveyard");
    drain_stack(&mut g);
    assert!(g.battlefield_find(me).is_some_and(|c| !c.keyword_counters.is_empty()), "entered with its counter");
    // A whole-board keyword ask (the cumulative-upkeep gate).
    let _ = g.process_cumulative_upkeep();
}

/// CR 704.5m — Minimus Containment ("enchant nonland permanent") on a
/// creature that an Aura then makes a land is put into its owner's
/// graveyard by the sweep that follows (a fuzzed 6-seat pod, seed 710168
/// game 7: Imprisoned in the Moon on a contained Drakuseth). CR 613.7d — the
/// resolving Aura's timestamp is its entry time; that sweep ran before the
/// event dispatch stamped it, so it read as older than Minimus. (Imprisoned
/// itself can't enchant a Treasure; a bare "it's a land" Aura stands in.)
#[test]
fn cr_704_5m_an_aura_whose_host_became_a_land_is_shed() {
    let mut g = main_phase();
    // Drawn before the others: an unstamped object's timestamp falls back to
    // its id, which is then older than Minimus Containment's.
    use crabomination::card::{CardDefinition, CardType, EnchantmentSubtype, EquipBonus, Subtypes};
    let moon = g.add_card_to_hand(0, CardDefinition {
        name: "Land Shroud",
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: crabomination::effect::shortcut::target_filtered(crabomination::card::SelectionRequirement::Permanent),
        },
        equipped_bonus: Some(EquipBonus { set_card_types: Some(vec![CardType::Land]), ..Default::default() }),
        ..Default::default()
    });
    let host = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let minimus = g.add_card_to_battlefield(1, catalog::minimus_containment());
    g.battlefield_find_mut(minimus).unwrap().attached_to = Some(host);
    g.check_state_based_actions();
    assert!(g.battlefield_find(minimus).is_some(), "a contained creature is a legal host");
    g.perform_action(crabomination::game::types::GameAction::CastSpell {
        card_id: moon,
        target: Some(Target::Permanent(host)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    while !g.stack.is_empty() {
        g.perform_action(crabomination::game::types::GameAction::PassPriority).expect("pass");
    }
    assert!(g.battlefield_find(minimus).is_none(), "the host is a land now");
}

/// CR 613.7d — a permanent's timestamp is the time it entered, already as it
/// enters: before the event dispatch runs, a token is newer than everything
/// on the battlefield. Unstamped, it read as its `CardId`, which in a long
/// game is far behind the timestamp counter (tokens, persist / undying
/// returns, a played land and an opening-hand start all were).
#[test]
fn cr_613_7d_an_entering_permanent_is_timestamped_before_dispatch() {
    use crabomination::effect::Value;
    let mut g = main_phase();
    for _ in 0..1000 {
        g.next_timestamp(); // a long game's worth of effects
    }
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.resolve_effect(
        &Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: std::sync::Arc::new(crabomination_base::tokens::fractal_token()),
        },
        &EffectContext::for_ability(src, 0, None),
    )
    .expect("resolves");
    let token = g.battlefield.iter().find(|c| c.definition.name == "Fractal").unwrap().id;
    let ts = |id| g.battlefield_find(id).unwrap().object_timestamp();
    assert!(ts(token) > ts(wall) && ts(token) > ts(src), "newest on entry");
}

/// CR 610.3 — "exile it until this leaves the battlefield" aimed at the
/// source itself: the leave IS the exile, so the return follows at once (a
/// fuzzed 5-seat audit-build pod, seed 740008 game 7: Aboleth Spawn copied
/// Constricting Sliver's trigger and aimed it at the Sliver, which then sat
/// in exile for good — its leave hook ran before the link was made).
#[test]
fn cr_610_3_a_source_that_exiles_itself_until_it_leaves_comes_back() {
    let mut g = main_phase();
    let sliver = g.add_card_to_battlefield(0, catalog::constricting_sliver());
    resolve(
        &mut g,
        sliver,
        None,
        &Effect::ExileUntilSourceLeaves {
            what: Selector::ExactObjects(vec![sliver]),
            return_to: crabomination::card::ExileReturnZone::Battlefield,
        },
    );
    assert!(g.exile.iter().all(|c| c.definition.name != "Constricting Sliver"), "not left in exile");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Constricting Sliver"), "back on the battlefield");
}

/// CR 704.3 — no state-based sweep while a resolution is paused on its next
/// ask: Pox Plague's seat 0 sacrificed the land under seat 1's Spreading
/// Seas, the post-answer sweep put the Seas in the graveyard under seat 1's
/// pending pick, the pick was refused every round and the game froze (cube
/// seed 830002, eight "stuck" games).
#[test]
fn cr_704_3_pox_plague_takes_the_offered_sacrifice() {
    use crabomination::decision::{Decision, DecisionAnswer};
    use crabomination::game::types::GameAction;
    let mut g = main_phase();
    for s in 0..2 {
        g.players[s].wants_ui = true;
        for _ in 0..6 {
            g.add_card_to_battlefield(s, catalog::swamp());
        }
    }
    // Seat 1's Spreading Seas on one of seat 0's lands.
    let land = g.battlefield.iter().find(|c| c.controller == 0).unwrap().id;
    let seas = g.add_card_to_battlefield(1, catalog::spreading_seas());
    g.battlefield_find_mut(seas).unwrap().attached_to = Some(land);
    let plague = g.add_card_to_hand(0, catalog::pox_plague());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 5);
    g.perform_action(GameAction::CastSpell { card_id: plague, target: None, additional_targets: vec![], mode: None, x_value: None })
        .expect("cast");
    for _ in 0..20 {
        let Some(d) = g.pending_decision.as_ref() else {
            if g.stack.is_empty() {
                break;
            }
            g.resolve_top_of_stack().expect("resolve");
            continue;
        };
        let answer = match &d.decision {
            // Seat 0 gives up the land under the Seas; seat 1 the Seas.
            Decision::ChooseCards { candidates, min, .. } => {
                let mut ids: Vec<_> = candidates.iter().map(|c| c.0).collect();
                ids.sort_by_key(|id| !(*id == land || *id == seas));
                DecisionAnswer::Cards(ids.into_iter().take(*min as usize).collect())
            }
            Decision::Discard { .. } => DecisionAnswer::Discard(vec![]),
            other => panic!("unexpected ask {other:?}"),
        };
        let shown = format!("{:?}", d.decision);
        g.submit_decision(answer).unwrap_or_else(|e| panic!("{e:?} answering {shown}"));
    }
    for s in 0..2 {
        assert!(g.battlefield.iter().filter(|c| c.controller == s).count() <= 4, "seat {s} gave up half");
    }
}

/// CR 800.4a / 800.4h — Master Warcraft's caster leaves before declare
/// attackers: the declaration goes back to the active player. The chooser
/// stayed set and the step handed priority to the departed seat (an 8-seat
/// pod under concessions, seed 817083 game 7).
#[test]
fn cr_800_4a_a_departed_combat_chooser_hands_the_declaration_back() {
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    let mw = g.add_card_to_hand(1, catalog::master_warcraft());
    g.players[1].mana_pool.add(crabomination::mana::Color::Red, 4);
    g.step = TurnStep::BeginCombat;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: mw, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast on another seat's turn");
    drain_stack(&mut g);
    assert_eq!(g.combat_chooser, Some(1));
    g.concede(1);
    assert_eq!(g.combat_chooser, None, "the chooser left");
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    assert_eq!(g.player_with_priority(), 0, "the active player declares");
    g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(2) }]).expect("declares");
    assert_eq!(g.attacking.len(), 1);
}

/// CR 400.7 — an "enchant player" Aura exiled by an effect is a new object
/// in exile, attached to nobody. `move_card_to` cleared the creature-side
/// attachment only, so Urza's Ruinous Blast left Curse of Clinging Webs
/// attached to its player in exile (pod seed 819001, the CR 400.7 invariant).
#[test]
fn cr_400_7_an_exiled_curse_is_attached_to_no_player() {
    let mut g = main_phase();
    let curse = g.add_card_to_battlefield(0, catalog::curse_of_clinging_webs());
    g.battlefield_find_mut(curse).unwrap().attached_to_player = Some(1);
    let blast = g.add_card_to_hand(0, catalog::urzas_ruinous_blast());
    g.add_card_to_battlefield(0, catalog::urza_lord_high_artificer());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 5);
    g.perform_action(crabomination::game::types::GameAction::CastSpell {
        card_id: blast, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast with a legendary creature out");
    drain_stack(&mut g);
    let exiled = g.exile.iter().find(|c| c.id == curse).expect("exiled");
    assert_eq!(exiled.attached_to_player, None, "a new object, attached to no one");
}

/// CR 400.7 / 406.3 / 903.9a — a commander exiled face down (off its library,
/// after its owner declined the CR 903.9b redirect) goes home a new, face-up
/// object. It used to arrive face down (pod seed 818221 game 19).
#[test]
fn cr_903_9a_a_face_down_exiled_commander_goes_home_face_up() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let mut card = g.players[0].command.remove(pos);
    card.face_down = true;
    g.exile.push(card);
    g.check_state_based_actions();
    let home = g.players[0].command.iter().find(|c| c.id == cmd).expect("home");
    assert!(!home.face_down, "face up in the command zone");
}

fn cast_semesters_end(g: &mut GameState, target: crabomination::card::CardId) {
    let id = g.add_card_to_hand(0, catalog::semesters_end());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 4);
    g.perform_action(crabomination::game::types::GameAction::CastSpell {
        card_id: id, target: Some(Target::Permanent(target)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(g);
}

fn to_end_step(g: &mut GameState) {
    while g.step != TurnStep::End {
        g.perform_action(crabomination::game::types::GameAction::PassPriority).expect("pass");
    }
    drain_stack(g);
}

/// CR 400.7 / 903.9a — Semester's End exiles a commander, which goes home;
/// at the end step its delayed return finds nothing in exile. It used to put
/// the "additional counter" on the card in the command zone (pod seed 818238).
#[test]
fn cr_400_7_semesters_end_leaves_a_commander_that_went_home_alone() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let card = g.players[0].command.remove(pos);
    g.battlefield.push(card);
    cast_semesters_end(&mut g, cmd);
    g.check_state_based_actions();
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "home under CR 903.9a");
    to_end_step(&mut g);
    let home = g.players[0].command.iter().find(|c| c.id == cmd).expect("still home");
    assert!(home.counters.is_empty(), "no counter on the new object");
    assert!(g.battlefield_find(cmd).is_none());
}

/// Semester's End returns each card "under its owner's control" — a stolen
/// creature comes back to its owner, with its +1/+1 counter.
#[test]
fn semesters_end_returns_a_stolen_creature_to_its_owner() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().controller = 0;
    cast_semesters_end(&mut g, bear);
    to_end_step(&mut g);
    let back = g.battlefield_find(bear).expect("returned");
    assert_eq!(back.controller, 1, "its owner's control");
    assert_eq!(back.counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
}

/// CR 404.2 — a graveyard is a face-up pile: a processor cost (Void
/// Attendant) that takes a face-down exiled card (a hideaway pick) puts it
/// there face up. It arrived face down (pod seed 818272).
#[test]
fn cr_404_2_a_processed_face_down_card_lands_face_up() {
    let mut g = main_phase();
    let va = g.add_card_to_battlefield(0, catalog::void_attendant());
    let hidden = g.add_card_to_library(1, catalog::grizzly_bears());
    let pos = g.players[1].library.iter().position(|c| c.id == hidden).unwrap();
    let mut card = g.players[1].library.remove(pos);
    card.face_down = true;
    g.exile.push(card);
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 2);
    g.perform_action(crabomination::game::types::GameAction::ActivateAbility {
        card_id: va, ability_index: 0, target: None, additional_targets: vec![], x_value: None, mode: None,
    })
    .expect("process");
    let gy = g.players[1].graveyard.iter().find(|c| c.id == hidden).expect("processed");
    assert!(!gy.face_down, "face up in the graveyard");
}

/// CR 712.4 — a disturbed DFC bounced to a hand is its front face there
/// (Evacuation took a Generous Soul back as its back face, pod seed 818288).
#[test]
fn cr_712_4_a_bounced_disturb_back_face_is_its_front_in_hand() {
    let mut g = main_phase();
    let beggar = g.add_card_to_graveyard(0, catalog::beloved_beggar());
    g.players[0].mana_pool.add(crabomination::mana::Color::White, 6);
    g.perform_action(crabomination::game::types::GameAction::CastDisturb {
        card_id: beggar, target: None, additional_targets: vec![],
    })
    .expect("disturb");
    drain_stack(&mut g);
    assert!(g.battlefield_find(beggar).is_some_and(|c| c.transformed), "Generous Soul");
    resolve(&mut g, beggar, None, &Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) });
    let held = g.players[0].hand.iter().find(|c| c.id == beggar).expect("bounced");
    assert!(!held.transformed);
    assert_eq!(held.definition.name, "Beloved Beggar");
}

/// CR 400.7 / 608.2b — a trigger aimed at a commander that went home before
/// it resolved puts no counter on the card in the command zone (Omo's token
/// copies' "everything counter" did, pod seed 818308). Only a plane counts
/// counters there (CR 901.7).
#[test]
fn cr_400_7_a_counter_aimed_at_a_commander_gone_home_is_lost() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve(
        &mut g,
        src,
        Some(Target::Permanent(cmd)),
        &Effect::AddCounter {
            what: Selector::Target(0),
            kind: crabomination::card::CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::Const(1),
        },
    );
    let home = g.players[0].command.iter().find(|c| c.id == cmd).expect("home");
    assert!(home.counters.is_empty());
}

/// CR 400.7 — an "as this enters, choose a creature type" answer belongs to
/// the permanent: bounced, the card in hand holds none, and the next object
/// asks again (a bounced Unclaimed Territory kept its Knight, pod seed 818500).
#[test]
fn cr_400_7_a_bounced_permanent_forgets_its_chosen_type() {
    let mut g = main_phase();
    let banner = g.add_card_to_battlefield(0, catalog::vanquishers_banner());
    g.battlefield_find_mut(banner).unwrap().chosen_creature_type = Some(crabomination::card::CreatureType::Human);
    resolve(&mut g, banner, None, &Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) });
    let held = g.players[0].hand.iter().find(|c| c.id == banner).expect("bounced");
    assert_eq!(held.chosen_creature_type, None);
}

/// CR 400.7 / 716 — a Class's level is battlefield-only: destroyed, it is a
/// level-less card in the graveyard (Fortune Teller's Talent sat there at
/// level 1, pod seed 818500).
#[test]
fn cr_716_a_destroyed_class_has_no_level_in_the_graveyard() {
    let mut g = main_phase();
    let class = g.add_card_to_battlefield(0, catalog::fortune_tellers_talent());
    g.battlefield_find_mut(class).unwrap().class_level = 2;
    let mut events = Vec::new();
    g.destroy_permanent(class, false, &mut events);
    let gy = g.players[0].graveyard.iter().find(|c| c.id == class).expect("destroyed");
    assert_eq!(gy.class_level, 0);
}

/// CR 702.95e — a paired creature that leaves the battlefield is unpaired: a
/// bounced Cathodion kept its soulbond link in hand (pod seed 818500).
#[test]
fn cr_702_95e_a_bounced_partner_is_unpaired() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(a).unwrap().soulbond_partner = Some(b);
    g.battlefield_find_mut(b).unwrap().soulbond_partner = Some(a);
    resolve(&mut g, a, None, &Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) });
    assert_eq!(g.players[0].hand.iter().find(|c| c.id == a).expect("bounced").soulbond_partner, None);
}

/// CR 302.6 — summoning sickness is about control since the turn began, so a
/// skipped untap step lifts it all the same: the skip kept the seat out of
/// the untap loop that clears it (a pod's Kotori stayed sick all turn, seed
/// 818652, the CR 302.6 turn-boundary invariant).
#[test]
fn cr_302_6_a_skipped_untap_step_still_lifts_sickness() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().summoning_sick = true;
    let src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    resolve(&mut g, src, None, &Effect::SkipPlayerUntapStep { player: PlayerRef::Seat(0) });
    g.do_untap();
    assert!(!g.battlefield_find(bear).unwrap().summoning_sick);
}

/// Push a trigger aimed at `target` / `rest` (CR 608.2b's "when it was
/// targeted" is now), let `meanwhile` change the board, then resolve it.
fn push_then_resolve(
    g: &mut GameState,
    src: crabomination::card::CardId,
    body: Effect,
    target: Target,
    rest: Vec<Target>,
    meanwhile: impl FnOnce(&mut GameState),
) {
    let item = crabomination::game::types::TriggerPush::new(src, 0, body)
        .target(Some(target))
        .additional_targets(rest)
        .build();
    g.push_stack(item);
    meanwhile(g);
    drain_stack(g);
}

/// A two-slot trigger body: a counter on target land, then exile target
/// creature (slot 1).
fn mark_land_exile_creature() -> Effect {
    use crabomination::card::SelectionRequirement as R;
    Effect::Seq(vec![
        Effect::AddCounter {
            what: Selector::TargetFiltered { slot: 0, filter: R::Land },
            kind: crabomination::card::CounterType::PlusOnePlusOne,
            amount: crabomination::effect::Value::Const(1),
        },
        Effect::Move { what: Selector::TargetFiltered { slot: 1, filter: R::Creature }, to: ZoneDest::Exile },
    ])
}

/// CR 608.2b — "a target that's no longer in the zone it was in when it was
/// targeted is illegal", for a trigger's later slots too: the slot-1 creature
/// died, so the exile finds nothing (it used to exile the card from the
/// graveyard — Omo's counter reached a commander gone home the same way),
/// while the still-legal land slot resolves.
#[test]
fn cr_608_2b_a_later_trigger_slot_that_left_its_zone_is_illegal() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    push_then_resolve(&mut g, src, mark_land_exile_creature(), Target::Permanent(land), vec![Target::Permanent(bear)], |g| {
        g.destroy_permanent(bear, false, &mut Vec::new());
    });
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "not exiled from the graveyard");
}

/// CR 608.2b — an ability is removed only if EVERY target is illegal: slot 0
/// names a player who has left the game, slot 1's creature is still there and
/// is exiled. The trigger used to be removed whole on its first slot.
#[test]
fn cr_608_2b_a_trigger_with_one_legal_target_left_still_resolves() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let body = Effect::Seq(vec![
        Effect::LoseLife { who: Selector::Target(0), amount: crabomination::effect::Value::Const(1) },
        Effect::Move {
            what: Selector::TargetFiltered { slot: 1, filter: crabomination::card::SelectionRequirement::Creature },
            to: ZoneDest::Exile,
        },
    ]);
    push_then_resolve(&mut g, src, body, Target::Player(2), vec![Target::Permanent(bear)], |g| {
        g.concede(2);
    });
    assert!(g.exile.iter().any(|c| c.id == bear), "the legal target is exiled");
}

/// CR 608.2b — a trigger's "exile target creature" whose creature died in
/// response does nothing: the creature card in the graveyard is a new object,
/// though it still matches "creature". The move used to find it there and
/// exile it from the graveyard.
#[test]
fn cr_608_2b_a_trigger_target_that_died_is_not_exiled_from_the_graveyard() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let exile = Effect::Move {
        what: Selector::TargetFiltered { slot: 0, filter: crabomination::card::SelectionRequirement::Creature },
        to: ZoneDest::Exile,
    };
    push_then_resolve(&mut g, src, exile, Target::Permanent(bear), vec![], |g| {
        g.destroy_permanent(bear, false, &mut Vec::new());
    });
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "still in the graveyard");
}

/// CR 704.5m / 800.4a — a Curse on a player who leaves the game is attached
/// to no player, so it goes to its owner's graveyard. Curses stayed on the
/// battlefield on a departed seat (pod seed 819008, the new Aura invariant).
#[test]
fn cr_704_5m_a_curse_on_a_departed_player_goes_to_the_graveyard() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let curse = g.add_card_to_battlefield(0, catalog::curse_of_clinging_webs());
    g.battlefield_find_mut(curse).unwrap().attached_to_player = Some(2);
    g.concede(2);
    g.perform_action(crabomination::game::types::GameAction::PassPriority).expect("pass");
    assert!(g.battlefield_find(curse).is_none(), "shed");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == curse));
}


/// CR 605.3b / 106.6 — a payment whose ordinary auto-tap falls short retries
/// with spend-restricted sources (Unclaimed Territory) from the snapshot; a
/// Treasure the short attempt sacrificed can't be untapped, so its mana stays
/// in the pool and its sacrifice stands. The retry used to drop both: the
/// Treasure sat in a graveyard (pod seed 819163) and its mana was lost, so
/// this cast failed.
#[test]
fn cr_605_3b_a_treasure_sacrificed_by_a_short_auto_tap_still_pays() {
    let mut g = main_phase();
    let territory = g.add_card_to_battlefield(0, catalog::unclaimed_territory());
    g.battlefield_find_mut(territory).unwrap().chosen_creature_type = Some(crabomination::card::CreatureType::Beast);
    let tok = EffectContext::for_ability(crabomination::card::CardId(0), 0, None);
    g.resolve_effect(
        &Effect::CreateToken {
            who: PlayerRef::You,
            count: crabomination::card::Value::ONE,
            definition: std::sync::Arc::new(crabomination::game::effects::treasure_token()),
        },
        &tok,
    )
    .unwrap();
    let treasure = g.battlefield.iter().find(|c| c.definition.name == "Treasure").unwrap().id;
    // {G}{G}: the Territory's {C} can't help, so the ordinary attempt taps the
    // Treasure, comes up a pip short, and the restricted retry runs.
    let beast = g.add_card_to_hand(0, catalog::kalonian_tusker());
    g.perform_action(crabomination::game::types::GameAction::CastSpell {
        card_id: beast, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("Territory + Treasure pay {G}{G}");
    assert!(g.battlefield_find(treasure).is_none() && !g.players[0].graveyard.iter().any(|c| c.id == treasure));
}

/// CR 506.4 — a combatant that stops being a creature is removed from combat
/// at once, even while another player's ask is still pending: Klothys lost
/// devotion to a Butcher of Malakir sacrifice and kept attacking as an
/// enchantment, because the answer's sweep waited on the next asker (pod
/// seed 819257, the CR 506.4 invariant).
#[test]
fn cr_506_4_a_god_that_loses_devotion_leaves_combat_while_an_ask_is_pending() {
    use crabomination::decision::DecisionAnswer;
    use crabomination::game::types::{Attack, AttackTarget, GameAction};
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    let klothys = g.add_card_to_battlefield(0, catalog::klothys_god_of_destiny());
    g.add_card_to_battlefield(0, catalog::leatherback_baloth());
    let tusker = g.add_card_to_battlefield(0, catalog::kalonian_tusker());
    // Two, so the second asker has a real choice to wait on.
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let butcher = g.add_card_to_battlefield(2, catalog::butcher_of_malakir());
    g.clear_sickness(klothys);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: klothys, target: AttackTarget::Player(2) }]))
        .expect("Klothys attacks at devotion 7");
    for seat in 0..3 {
        g.players[seat].wants_ui = true;
    }
    g.destroy_permanent(butcher, false, &mut Vec::new());
    g.priority.player_with_priority = 2;
    for _ in 0..8 {
        if g.pending_decision.is_some() || g.stack.is_empty() {
            break;
        }
        let p = g.player_with_priority();
        g.priority.player_with_priority = p;
        let _ = g.perform_action(GameAction::PassPriority);
    }
    // The first asker sacrifices the Tusker; the second ask is left pending.
    let mut answered = false;
    for _ in 0..2 {
        let Some(pd) = g.pending_decision.as_ref() else { break };
        if let crabomination::decision::Decision::ChooseTarget { legal, .. } = &pd.decision
            && legal.contains(&Target::Permanent(tusker))
        {
            g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(Target::Permanent(tusker))))
                .expect("sacrifice the Tusker");
            answered = true;
            break;
        }
        let first = match &pd.decision {
            crabomination::decision::Decision::ChooseTarget { legal, .. } => legal[0].clone(),
            _ => break,
        };
        g.perform_action(GameAction::SubmitDecision(DecisionAnswer::Target(first))).expect("answer");
    }
    assert!(answered, "the Tusker's controller was asked");
    assert!(g.battlefield_find(tusker).is_none());
    assert!(!g.attacking.iter().any(|a| a.attacker == klothys), "Klothys left combat");
}

/// A free sorcery around a two-slot body, cast at `target` / `rest`, with
/// `meanwhile` run while it is on the stack.
fn cast_then_resolve(
    g: &mut GameState,
    body: Effect,
    target: Target,
    rest: Vec<Target>,
    meanwhile: impl FnOnce(&mut GameState),
) {
    use crabomination::card::{CardDefinition, CardType};
    let spell = g.add_card_to_hand(
        0,
        CardDefinition { name: "Two-Slot Sorcery", card_types: vec![CardType::Sorcery], effect: body, ..Default::default() },
    );
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(target),
        additional_targets: rest,
        mode: None,
        x_value: None,
    })
    .expect("cast");
    meanwhile(g);
    drain_stack(g);
}

/// CR 608.2b — a SPELL's later slot that left the battlefield is illegal and
/// the spell does nothing to it, while its legal land slot resolves: the
/// dead creature's card stays in the graveyard (the exile used to find it
/// there — only slot 0 carried a "was on the battlefield" mark).
#[test]
fn cr_608_2b_a_later_spell_slot_that_left_its_zone_is_illegal() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::forest());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    cast_then_resolve(&mut g, mark_land_exile_creature(), Target::Permanent(land), vec![Target::Permanent(bear)], |g| {
        g.destroy_permanent(bear, false, &mut Vec::new());
    });
    assert_eq!(g.battlefield_find(land).unwrap().counter_count(crabomination::card::CounterType::PlusOnePlusOne), 1);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "not exiled from the graveyard");
}

/// CR 608.2b — the mark is per slot, so a spell whose FIRST target is a
/// player still re-checks its creature slot: the player loses the life, the
/// creature that died in response is not exiled from the graveyard.
#[test]
fn cr_608_2b_a_spell_aimed_at_a_player_first_rechecks_its_creature_slot() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let body = Effect::Seq(vec![
        Effect::LoseLife { who: Selector::Target(0), amount: crabomination::effect::Value::Const(1) },
        Effect::Move {
            what: Selector::TargetFiltered { slot: 1, filter: crabomination::card::SelectionRequirement::Creature },
            to: ZoneDest::Exile,
        },
    ]);
    let life = g.players[1].life;
    cast_then_resolve(&mut g, body, Target::Player(1), vec![Target::Permanent(bear)], |g| {
        g.destroy_permanent(bear, false, &mut Vec::new());
    });
    assert_eq!(g.players[1].life, life - 1, "the legal player slot resolves");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear), "not exiled from the graveyard");
}

/// CR 608.2b — and when every target is illegal the spell doesn't resolve:
/// the player left the game (CR 800.4a) and the creature died.
#[test]
fn cr_608_2b_a_spell_whose_player_and_creature_targets_are_both_gone_fizzles() {
    let mut g = multi_player_game(3);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PreCombatMain;
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let body = Effect::Seq(vec![
        Effect::LoseLife { who: Selector::Target(0), amount: crabomination::effect::Value::Const(1) },
        Effect::Move {
            what: Selector::TargetFiltered { slot: 1, filter: crabomination::card::SelectionRequirement::Creature },
            to: ZoneDest::Exile,
        },
        Effect::GainLife { who: Selector::You, amount: crabomination::effect::Value::Const(5) },
    ]);
    let life = g.players[0].life;
    cast_then_resolve(&mut g, body, Target::Player(2), vec![Target::Permanent(bear)], |g| {
        g.concede(2);
        g.destroy_permanent(bear, false, &mut Vec::new());
    });
    assert_eq!(g.players[0].life, life, "the spell did not resolve");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bear));
}

/// CR 712.4 / 903.9a — a transformed commander that went to exile goes home
/// front face up: Sephiroth, One-Winged Angel reached the command zone still
/// transformed after a mass exile (eight-seat pod, seed 3121014 game 3).
#[test]
fn cr_712_4_a_transformed_commander_goes_home_front_face_up() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::sephiroth_fabled_soldier()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let mut card = g.players[0].command.remove(pos);
    let back = card.definition.back_face.clone().expect("a back face");
    card.front_face = Some(card.definition.arc());
    card.set_definition(std::sync::Arc::new((*back).clone()));
    card.transformed = true;
    g.exile.push(card);
    g.check_state_based_actions();
    let home = g.players[0].command.iter().find(|c| c.id == cmd).expect("home");
    assert!(!home.transformed);
    assert_eq!(home.definition.name, "Sephiroth, Fabled SOLDIER");
}

/// CR 903.9b — "each player shuffles their hand into their library, then
/// draws that many cards": a commander in hand may go to the command zone
/// instead, and "that many" counts what was shuffled in. Molten Psyche put
/// Olivia, Opulent Outlaw into a library (eight-seat pod, seed 3121026 game 17).
#[test]
fn cr_903_9b_a_hand_shuffled_into_a_library_sends_its_commander_home() {
    let mut g = main_phase();
    let cmd = g.seat_commanders(0, vec![catalog::grizzly_bears()])[0];
    let pos = g.players[0].command.iter().position(|c| c.id == cmd).unwrap();
    let card = g.players[0].command.remove(pos);
    g.players[0].hand.push(card);
    g.add_card_to_hand(0, catalog::forest());
    let hand = g.players[0].hand.len();
    resolve(&mut g, crabomination::card::CardId(0), None, &Effect::ShuffleHandsDrawSame { who: PlayerRef::You });
    assert!(g.players[0].command.iter().any(|c| c.id == cmd), "home");
    assert!(g.players[0].library.iter().all(|c| c.id != cmd));
    assert_eq!(g.players[0].hand.len(), hand - 1, "draws the one card shuffled in");
}

/// CR 712.4 — a transformed permanent exiled "until this leaves" is exiled
/// front face up; only a defeated battle keeps its back face in exile. Aerial
/// Extortionist exiled an Abolisher of Bloodlines as its back face (audit pod
/// seed 3130108 game 5).
#[test]
fn cr_712_4_a_transformed_permanent_is_exiled_front_face_up() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let dfc = g.add_card_to_battlefield(1, catalog::voldaren_bloodcaster());
    g.transform_permanent(dfc, &mut Vec::new());
    assert!(g.battlefield_find(dfc).unwrap().transformed);
    let exile = Effect::Move {
        what: Selector::TargetFiltered { slot: 0, filter: crabomination::card::SelectionRequirement::Creature },
        to: ZoneDest::Exile,
    };
    resolve(&mut g, src, Some(Target::Permanent(dfc)), &exile);
    let c = g.exile.iter().find(|c| c.id == dfc).expect("exiled");
    assert!(!c.transformed);
    assert_eq!(c.definition.name, "Voldaren Bloodcaster");
}

/// CR 701.40a / 708 — a card that was to be manifested but never reached the
/// battlefield stays where it was, face up: Grafdigger's Cage keeps a Bear in
/// the graveyard against Ghastly Conscription. It used to stay there face
/// down (seven-seat fuzzed audit pod, seed 3141188 game 7).
#[test]
fn cr_701_40a_a_manifest_that_never_entered_leaves_the_card_face_up() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grafdiggers_cage());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let body = Effect::ManifestFromGraveyard {
        who: PlayerRef::Seat(1),
        filter: crabomination::card::SelectionRequirement::Creature,
    };
    resolve(&mut g, src, None, &body);
    let c = g.players[1].graveyard.iter().find(|c| c.id == bear).expect("kept in the graveyard");
    assert!(!c.face_down);
    assert_eq!(c.definition.name, "Grizzly Bears");
}

/// CR 122.2 — a card that keeps its counters (Skullbriar) still loses them
/// going into a library: Learn from the Past shuffled one in with its +1/+1
/// counter (six-seat census pod, seed 440117 game 13).
#[test]
fn cr_122_2_a_graveyard_shuffled_into_a_library_sheds_kept_counters() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let sb = g.add_card_to_graveyard(1, catalog::skullbriar_the_walking_grave());
    g.players[1].graveyard.iter_mut().find(|c| c.id == sb).unwrap()
        .add_counters(crabomination::card::CounterType::PlusOnePlusOne, 1);
    resolve(&mut g, src, None, &Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::Seat(1) });
    let c = g.players[1].library.iter().find(|c| c.id == sb).expect("in the library");
    assert!(c.counters.is_empty());
}

/// CR 400.7 — a kicked spell's card in the graveyard is a new object: back in
/// hand and cast again without kicker, Into the Roil doesn't draw. The kicked
/// flag used to ride into the graveyard and read as kicked on the recast (an
/// audit-pod invariant found it on 60+ cards, seed 31731).
#[test]
fn cr_400_7_a_recast_card_is_not_kicked_by_its_last_cast() {
    use crabomination::mana::Color;
    let mut g = main_phase();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let roil = g.add_card_to_hand(0, catalog::into_the_roil());
    let b1 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.perform_action(GameAction::CastSpellKicked {
        card_id: roil, target: Some(Target::Permanent(b1)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("kicked");
    drain_stack(&mut g);
    let pos = g.players[0].graveyard.iter().position(|c| c.id == roil).expect("in the graveyard");
    assert!(!g.players[0].graveyard[pos].kicked, "the graveyard card is not kicked");
    let card = g.players[0].graveyard.remove(pos);
    g.players[0].hand.push(card);
    let b2 = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: roil, target: Some(Target::Permanent(b2)), additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("unkicked");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1, "no kicker draw");
}

/// CR 400.7 — a permission stamped on a library card ends when it is drawn.
#[test]
fn cr_400_7_a_drawn_card_loses_its_play_permission() {
    let mut g = main_phase();
    let top = g.add_card_to_library(0, catalog::island());
    let perm = crabomination::card::MayPlayPermission {
        cast_only: false, locks_further_casts: false, one_cast_group: None, player: 0, granted_turn: g.turn_number,
        duration: crabomination::card::MayPlayDuration::EndOfThisTurn, exile_after: false, miracle: false,
        pay_life: false, bottom_after: false, undaunted: false,
    };
    g.players[0].library.iter_mut().find(|c| c.id == top).unwrap().may_play_until = Some(perm);
    g.draw_one(0, &mut Vec::new());
    let c = g.players[0].hand.iter().find(|c| c.id == top).expect("drawn");
    assert!(c.may_play_until.is_none());
}

/// CR 708 — Missy's "put it onto the battlefield face down as a Cyberman"
/// that never entered (Grafdigger's Cage) leaves the card face up in the
/// graveyard; it sat there face down (fuzzed audit pod, seed 3184380 game 7).
#[test]
fn cr_708_a_cyberman_that_never_entered_stays_face_up() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grafdiggers_cage());
    let bear = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let body = Effect::PutFaceDownAsCyberman { what: Selector::CardsInZone {
        who: PlayerRef::Seat(1), zone: crabomination::card::Zone::Graveyard,
        filter: crabomination::card::SelectionRequirement::Creature,
    }, tapped: false };
    resolve(&mut g, src, None, &body);
    let c = g.players[1].graveyard.iter().find(|c| c.id == bear).expect("kept in the graveyard");
    assert!(!c.face_down);
    assert_eq!(c.definition.name, "Grizzly Bears");
}

/// CR 400.7 — a graveyard cast permission (Silas Renn's "you may cast that
/// card this turn") doesn't follow the card back to hand.
#[test]
fn cr_400_7_a_graveyard_permission_does_not_follow_a_card_to_hand() {
    let mut g = main_phase();
    let src = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let card = g.add_card_to_graveyard(0, catalog::sol_ring());
    let perm = crabomination::card::MayPlayPermission {
        cast_only: true, locks_further_casts: false, one_cast_group: None, player: 0, granted_turn: g.turn_number,
        duration: crabomination::card::MayPlayDuration::EndOfThisTurn, exile_after: false, miracle: false,
        pay_life: false, bottom_after: false, undaunted: false,
    };
    g.players[0].graveyard.iter_mut().find(|c| c.id == card).unwrap().may_play_until = Some(perm);
    let back = Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) };
    resolve(&mut g, src, Some(Target::Permanent(card)), &back);
    let c = g.players[0].hand.iter().find(|c| c.id == card).expect("in hand");
    assert!(c.may_play_until.is_none());
}

/// CR 122.2 — a card cast with counters on it (a void counter from exile)
/// is a spell without them; Me, the Immortal-style "keeps its counters" cards
/// aside. (Squee, the Immortal reached the stack with a void counter;
/// fuzzed audit pod, seed 3206648.)
#[test]
fn cr_122_2_a_spell_carries_no_counters_onto_the_stack() {
    use crabomination::mana::Color;
    let mut g = main_phase();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].hand.iter_mut().find(|c| c.id == bear).unwrap()
        .add_counters(crabomination::card::CounterType::Void, 1);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    let on_stack = g.stack.iter().find_map(|si| match si {
        crabomination::game::types::StackItem::Spell { card, .. } if card.id == bear => Some(card.counters.is_empty()),
        _ => None,
    });
    assert_eq!(on_stack, Some(true));
}

/// CR 400.7 — a kicked spell countered to the bottom of its owner's library
/// (Spell Crumple) is a new object there: not kicked (audit pod, seed
/// 4200188 game 6).
#[test]
fn cr_400_7_a_spell_countered_into_the_library_forgets_its_kicker() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let thirst = g.add_card_to_hand(0, catalog::bloodchiefs_thirst());
    g.players[0].mana_pool.add(crabomination::mana::Color::Black, 4);
    g.perform_action(crabomination::game::types::GameAction::CastSpellKicked {
        card_id: thirst,
        target: Some(Target::Permanent(bear)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("kicked Thirst");
    let src = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let crumple = Effect::CounterSpellToZone {
        what: Selector::Target(0),
        zone: crabomination::effect::CounteredSpellZone::OwnerLibraryBottom,
    };
    resolve(&mut g, src, Some(Target::Permanent(thirst)), &crumple);
    let c = g.players[0].library.iter().find(|c| c.id == thirst).expect("bottomed");
    assert!(!c.kicked);
}

/// CR 701.9a — "you may discard a card. If you do, …" (Pia, Aether Ascetic):
/// the discarding player picks the card, not the engine's highest-mana-value
/// default.
#[test]
fn cr_701_8a_a_may_discard_payoff_discards_the_chosen_card() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = main_phase();
    let ring = g.add_card_to_hand(0, catalog::sol_ring());
    let big = g.add_card_to_hand(0, catalog::serra_angel());
    g.add_card_to_library(0, catalog::glorious_anthem());
    let pia = g.add_card_to_hand(0, catalog::pia_aether_ascetic());
    g.players[0].mana_pool.add(crabomination::mana::Color::Green, 3);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true), DecisionAnswer::Cards(vec![ring])]));
    cast(&mut g, pia);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == ring), "the chosen card was discarded");
    assert!(g.players[0].hand.iter().any(|c| c.id == big), "the default pick stayed in hand");
    assert!(g.players[0].hand.iter().any(|c| c.definition.name == "Glorious Anthem"), "the tutor ran");
}

/// CR 115.1 / 800.4 pods — the slot walker aims a hostile multi-target spell
/// at opponents first for every pod seat, not only one flagged
/// `hostile_player_targets`: the frozen two-player control burned its own
/// face with Crackle with Power at a four-seat table (audit sweep 43001).
#[test]
fn a_pod_seats_crackle_with_power_never_names_its_caster() {
    let mut g = multi_player_game(4);
    g.players[0].hostile_player_targets = false;
    let crackle = catalog::crackle_with_power();
    let (first, rest) = g.auto_targets_for_effect_all_slots_x(&crackle.effect, 0, None, false, None, Some(3));
    let picks: Vec<Target> = first.into_iter().chain(rest).collect();
    assert!(!picks.is_empty());
    assert!(!picks.contains(&Target::Player(0)), "the caster is not a target: {picks:?}");
}

/// CR 603.2 — "at the beginning of each opponent's upkeep" (Fatespinner)
/// never triggers on its controller's own upkeep: a 4-seat pod skipped its
/// own draw step for 70 turns into a no-progress draw (seed 4400248).
#[test]
fn cr_603_2_an_each_opponents_upkeep_trigger_skips_its_controllers_upkeep() {
    let mut g = multi_player_game(3);
    g.add_card_to_battlefield(0, catalog::fatespinner());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    assert!(g.stack.is_empty(), "not on its controller's upkeep");
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    assert_eq!(g.stack.len(), 1, "on an opponent's");
}
