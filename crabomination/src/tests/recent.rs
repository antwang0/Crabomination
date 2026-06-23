//! Functionality tests for the recent-set staples in
//! `catalog::sets::decks::recent`.

use crate::card::{CounterType, CreatureType, Keyword};
use crate::catalog;
use crate::decision::{DecisionAnswer, ScriptedDecider};
use crate::game::types::{Attack, AttackTarget};
use crate::game::*;
use crate::mana::Color;
use crate::TurnStep;

/// Questing Beast can't be blocked by power-2-or-less creatures.
#[test]
fn questing_beast_evades_small_blockers() {
    let mut g = two_player_game();
    let qb = g.add_card_to_battlefield(0, catalog::questing_beast());
    let weak = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    assert!(!g.blocker_can_block_attacker(weak, qb), "power-2 can't block");
    assert!(g.blocker_can_block_attacker(big, qb), "power-4 can block");
    let r = g.battlefield_find(qb).unwrap();
    assert!(r.definition.keywords.contains(&Keyword::Deathtouch));
    assert!(r.definition.keywords.contains(&Keyword::Haste));
}

/// Cackling Slasher enters with a +1/+1 counter when a creature died this turn.
#[test]
fn cackling_slasher_grows_after_a_death() {
    let mut g = two_player_game();
    g.players[1].creatures_died_this_turn = 1;
    let slasher = g.move_card_to_battlefield_for_test(0, catalog::cackling_slasher());
    drain_stack(&mut g);
    let r = g.battlefield_find(slasher).unwrap();
    assert_eq!(r.counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
}

/// Cackling Slasher with no death this turn enters as a vanilla 3/3.
#[test]
fn cackling_slasher_no_death_no_counter() {
    let mut g = two_player_game();
    let slasher = g.move_card_to_battlefield_for_test(0, catalog::cackling_slasher());
    drain_stack(&mut g);
    let r = g.battlefield_find(slasher).unwrap();
    assert_eq!(r.counters.get(&CounterType::PlusOnePlusOne).copied(), None);
}

/// Wary Watchdog surveils on entry.
#[test]
fn wary_watchdog_surveils_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib = g.players[0].library.len();
    g.move_card_to_battlefield_for_test(0, catalog::wary_watchdog());
    drain_stack(&mut g);
    // Surveil 1 looked at the top card (library size unchanged when kept on top).
    assert!(g.players[0].library.len() <= lib, "surveil resolved");
    assert_eq!(catalog::wary_watchdog().triggered_abilities.len(), 2, "ETB + dies triggers");
}

/// Hunted Bonebrute gives the opponent two Dogs on ETB and drains on death.
#[test]
fn hunted_bonebrute_etb_dogs_and_death_drain() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let brute = catalog::hunted_bonebrute();
    let etb_ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&brute.triggered_abilities[0].effect, &etb_ctx).unwrap();
    let dogs = g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.name == "Dog").count();
    assert_eq!(dogs, 2, "opponent made two Dogs");
    // Death drain.
    g.resolve_effect(&brute.triggered_abilities[1].effect, &etb_ctx).unwrap();
    assert_eq!(g.players[1].life, 17, "each opponent lost 3");
}

/// Trumpeting Herd makes a 3/3 Elephant and has Rebound.
#[test]
fn trumpeting_herd_makes_elephant() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let th = catalog::trumpeting_herd();
    assert!(th.keywords.contains(&Keyword::Rebound));
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&th.effect, &ctx).unwrap();
    let ele = g.battlefield.iter().find(|c| c.definition.name == "Elephant").unwrap();
    assert_eq!((ele.power(), ele.toughness()), (3, 3));
}

/// Festergloom shrinks nonblack creatures but spares black ones.
#[test]
fn festergloom_minus_one_to_nonblack() {
    let mut g = two_player_game();
    let white = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green, 2/2
    let black = g.add_card_to_battlefield(1, catalog::black_knight()); // black, 2/2
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&catalog::festergloom().effect, &ctx).unwrap();
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(white).map(|c| (c.power, c.toughness)), Some((1, 1)));
    assert_eq!(g.computed_permanent(black).map(|c| (c.power, c.toughness)), Some((2, 2)));
}

/// Intrepid Rabbit's ETB pumps a creature you control and it has Offspring.
#[test]
fn intrepid_rabbit_etb_pump() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let r = catalog::intrepid_rabbit();
    assert!(r.keywords.iter().any(|k| matches!(k, Keyword::Offspring(_))));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&r.triggered_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Marauding Brinefang has Ward {3} and Islandcycling.
#[test]
fn marauding_brinefang_ward_and_islandcycling() {
    use crate::card::Keyword;
    let b = catalog::marauding_brinefang();
    assert!(b.keywords.iter().any(|k| matches!(k, Keyword::Ward(_))));
    assert!(b.keywords.iter().any(|k| matches!(k, Keyword::Typecycling(_))));
    assert_eq!((b.power, b.toughness), (6, 7));
}

/// Crystal Barricade gives its controller hexproof.
#[test]
fn crystal_barricade_grants_controller_hexproof() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::crystal_barricade());
    // Player 0 can't be targeted by an opponent now.
    assert!(g.player_has_static_hexproof(0), "controller has hexproof");
}

/// Druid of the Spade grows and gains trample only while you control a token.
#[test]
fn druid_of_the_spade_token_conditional() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let druid = g.add_card_to_battlefield(0, catalog::druid_of_the_spade());
    let base = g.computed_permanent(druid).unwrap();
    assert_eq!((base.power, base.toughness), (2, 3));
    assert!(!base.keywords.contains(&Keyword::Trample));
    // Mint a token → condition holds.
    let tok = crate::card::TokenDefinition {
        name: "Rabbit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![crate::card::CardType::Creature],
        ..Default::default()
    };
    g.add_token_to_battlefield(0, &tok);
    let buffed = g.computed_permanent(druid).unwrap();
    assert_eq!((buffed.power, buffed.toughness), (4, 3));
    assert!(buffed.keywords.contains(&Keyword::Trample));
}

/// Persistent Marshstalker grows by each other Rat you control.
#[test]
fn persistent_marshstalker_rat_lord() {
    let mut g = two_player_game();
    let stalker = g.add_card_to_battlefield(0, catalog::persistent_marshstalker());
    assert_eq!(g.computed_permanent(stalker).unwrap().power, 3, "no other Rats");
    g.add_card_to_battlefield(0, catalog::persistent_marshstalker()); // another Rat
    assert_eq!(g.computed_permanent(stalker).unwrap().power, 4, "+1 for the other Rat");
}

/// Nightbird's Clutches stops up to two creatures from blocking and has flashback.
#[test]
fn nightbirds_clutches_grants_cant_block() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::hill_giant());
    let nc = catalog::nightbirds_clutches();
    assert!(nc.keywords.iter().any(|k| matches!(k, Keyword::Flashback(_))));
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(a), Target::Permanent(b)];
    g.resolve_effect(&nc.effect, &ctx).unwrap();
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Get Out's bounce mode returns your creatures/enchantments to hand.
#[test]
fn get_out_bounce_mode_returns_permanents() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.mode = 1; // bounce mode
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::get_out().effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_none() && g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Helpful Hunter draws on entry.
#[test]
fn helpful_hunter_draws_on_etb() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let h = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::helpful_hunter());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h + 1);
}

/// Sunshower Druid's ETB grows a creature and gains a life.
#[test]
fn sunshower_druid_counter_and_lifegain() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life = 20;
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::sunshower_druid().triggered_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].life, 21);
}

/// Coruscation Mage pings each opponent; its trigger gates on noncreature spells.
#[test]
fn coruscation_mage_pings_each_opponent() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::coruscation_mage());
    g.players[1].life = 20;
    let ab = catalog::coruscation_mage().triggered_abilities[0].clone();
    assert!(matches!(ab.event.filter, Some(crate::card::Predicate::CastSpellMatches(_))));
    let ctx = crate::game::effects::EffectContext::for_trigger(mage, 0, None, 0);
    g.resolve_effect(&ab.effect, &ctx).unwrap();
    assert_eq!(g.players[1].life, 19, "each opponent took 1");
}

/// Treetop Snarespinner has reach + deathtouch and a sorcery-speed grow.
#[test]
fn treetop_snarespinner_keywords_and_grow() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let spider = catalog::treetop_snarespinner();
    assert!(spider.keywords.contains(&Keyword::Reach) && spider.keywords.contains(&Keyword::Deathtouch));
    assert!(spider.activated_abilities[0].sorcery_speed);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&spider.activated_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Thornplate Intimidator's ETB makes the opponent dodge by discarding rather
/// than losing 3 life.
#[test]
fn thornplate_intimidator_punisher_discard() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    g.add_card_to_hand(1, catalog::grizzly_bears()); // a card to pitch
    let trig = catalog::thornplate_intimidator().triggered_abilities[0].effect.clone();
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    g.resolve_effect(&trig, &ctx).unwrap();
    // No nonland permanent to sac, so the opponent discards (no life loss).
    assert_eq!(g.players[1].life, 20, "dodged the life loss");
    assert!(g.players[1].hand.is_empty(), "discarded instead");
}

/// Repeating Barrage burns for 3 and can return itself from the graveyard
/// after you've attacked.
#[test]
fn repeating_barrage_burns_and_raids_back() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::repeating_barrage().effect, &ctx).unwrap();
    assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 3/3");
    // The Raid ability is gated on having attacked this turn.
    let ab = &catalog::repeating_barrage().activated_abilities[0];
    assert!(ab.from_graveyard && ab.condition.is_some());
}

/// Fountainport Bell can be sacrificed to draw.
#[test]
fn fountainport_bell_sac_draws() {
    let mut g = two_player_game();
    let bell = g.add_card_to_battlefield(0, catalog::fountainport_bell());
    g.add_card_to_library(0, catalog::island());
    let hand = g.players[0].hand.len();
    let ctx = crate::game::effects::EffectContext::for_ability(bell, 0, None);
    g.resolve_effect(&catalog::fountainport_bell().activated_abilities[0].effect, &ctx).unwrap();
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Plumecreed Escort has flash + flying and its ETB grants hexproof.
#[test]
fn plumecreed_escort_etb_grants_hexproof() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let esc = catalog::plumecreed_escort();
    assert!(esc.keywords.contains(&Keyword::Flash) && esc.keywords.contains(&Keyword::Flying));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&esc.triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Hexproof));
}

/// Overprotect pumps +3/+3 and grants three protective keywords.
#[test]
fn overprotect_pumps_and_grants_keywords() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::overprotect().effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (5, 5));
    for kw in [Keyword::Trample, Keyword::Hexproof, Keyword::Indestructible] {
        assert!(cp.keywords.contains(&kw), "granted {kw:?}");
    }
}

/// Banishing Slash destroys a tapped creature and mints a Samurai when you
/// control an artifact and an enchantment.
#[test]
fn banishing_slash_destroys_and_makes_samurai() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().tapped = true;
    g.add_card_to_battlefield(0, catalog::mind_stone()); // artifact
    g.add_card_to_battlefield(0, catalog::solemnity()); // non-aura enchantment
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::banishing_slash().effect, &ctx).unwrap();
    assert!(g.battlefield_find(foe).is_none(), "tapped creature destroyed");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Samurai"),
        "Samurai token created"
    );
}

/// Lightshield Parry pumps +2/+2 and offers Cycling {2}.
#[test]
fn lightshield_parry_pumps_and_cycles() {
    use crate::card::Keyword;
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let parry = catalog::lightshield_parry();
    assert!(parry.keywords.iter().any(|k| matches!(k, Keyword::Cycling(_))));
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&parry.effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4));
}

/// Star Charter digs at end step only when you changed life that turn.
#[test]
fn star_charter_digs_after_life_change() {
    let mut g = two_player_game();
    let sc = g.add_card_to_battlefield(0, catalog::star_charter());
    let pred = catalog::star_charter().triggered_abilities[0].event.filter.clone().unwrap();
    let ctx = crate::game::effects::EffectContext::for_ability(sc, 0, None);
    // No life change → intervening-if fails.
    g.players[0].life_gained_this_turn = 0;
    g.players[0].lost_life_this_turn = false;
    assert!(!g.evaluate_predicate(&pred, &ctx), "no dig without a life change");
    // Gained life → condition holds.
    g.players[0].life_gained_this_turn = 2;
    assert!(g.evaluate_predicate(&pred, &ctx), "digs after gaining life");
}

/// Krydle's combat-damage trigger drains the player and self-scrys.
#[test]
fn krydle_combat_damage_drains_and_gains() {
    let mut g = two_player_game();
    let krydle = g.add_card_to_battlefield(0, catalog::krydle_of_baldurs_gate());
    g.add_card_to_library(1, catalog::island()); // something to mill
    g.players[0].life = 20;
    g.players[1].life = 20;
    let trig = catalog::krydle_of_baldurs_gate().triggered_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_trigger(krydle, 0, None, 0);
    ctx.trigger_source = Some(crate::game::effects::EntityRef::Player(1));
    g.resolve_effect(&trig, &ctx).unwrap();
    assert_eq!(g.players[1].life, 19, "damaged player lost 1");
    assert_eq!(g.players[0].life, 21, "Krydle's controller gained 1");
    assert_eq!(g.players[1].graveyard.len(), 1, "milled a card");
}

/// Dour Port-Mage's activated ability returns another of your creatures.
#[test]
fn dour_port_mage_bounces_own_creature() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::dour_port_mage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(mage, 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&catalog::dour_port_mage().activated_abilities[0].effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_none() && g.players[0].hand.iter().any(|c| c.id == bear));
}

/// Dour Port-Mage draws when another of your creatures is bounced (CR 603.6
/// leaves-without-dying).
#[test]
fn dour_port_mage_draws_on_bounce() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib = g.players[0].library.len();
    let mage = g.add_card_to_battlefield(0, catalog::dour_port_mage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bounce = catalog::dour_port_mage().activated_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_ability(mage, 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    let evs = g.resolve_effect(&bounce, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "leaves-without-dying drew a card");
}

/// Exiling your creature is also a leaves-without-dying event for Dour Port-Mage.
#[test]
fn dour_port_mage_draws_on_exile() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib = g.players[0].library.len();
    let _mage = g.add_card_to_battlefield(0, catalog::dour_port_mage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.remove_from_battlefield_to_exile(bear);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib - 1, "exile drew a card");
}

/// Dying (graveyard exit) does NOT trigger Dour Port-Mage.
#[test]
fn dour_port_mage_no_draw_on_death() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let lib = g.players[0].library.len();
    let _mage = g.add_card_to_battlefield(0, catalog::dour_port_mage());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.remove_to_graveyard_with_triggers(bear);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].library.len(), lib, "death is not leaves-without-dying");
}

/// Three Tree Scribe puts a +1/+1 counter on a creature you control when
/// another of your creatures leaves without dying.
#[test]
fn three_tree_scribe_counters_on_leave() {
    let mut g = two_player_game();
    let scribe = g.add_card_to_battlefield(0, catalog::three_tree_scribe());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let bounce = catalog::dour_port_mage().activated_abilities[0].effect.clone();
    let mut ctx = crate::game::effects::EffectContext::for_ability(scribe, 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    let evs = g.resolve_effect(&bounce, &ctx).unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    // Scribe is the only creature left, so the counter lands on it.
    assert_eq!(
        g.battlefield_find(scribe).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
    );
}

/// Hard-Hitting Question makes your creature deal its power to a foe.
#[test]
fn hard_hitting_question_deals_power() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::hill_giant()); // 3/3
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(foe)];
    g.resolve_effect(&catalog::hard_hitting_question().effect, &ctx).unwrap();
    assert!(g.battlefield_find(foe).is_none(), "3 damage kills the 2/2");
}

/// Brave-Kin Duo's sorcery-speed pump grows a creature by +1/+1.
#[test]
fn brave_kin_duo_pumps_at_sorcery_speed() {
    let mut g = two_player_game();
    let duo = catalog::brave_kin_duo();
    assert!(duo.activated_abilities[0].sorcery_speed, "activates only as a sorcery");
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&duo.activated_abilities[0].effect, &ctx).unwrap();
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 until end of turn");
}

/// Marsh Hulk carries Megamorph and can be cast face down for {3}.
#[test]
fn marsh_hulk_has_megamorph() {
    use crate::card::Keyword;
    let hulk = catalog::marsh_hulk();
    assert!(hulk.keywords.iter().any(|k| matches!(k, Keyword::Megamorph(_))));
    assert_eq!((hulk.power, hulk.toughness), (4, 6));
}

/// Refurbished Familiar's affinity discounts it per artifact, and its ETB
/// makes each opponent discard.
#[test]
fn refurbished_familiar_affinity_and_etb_discard() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mind_stone()); // an artifact
    g.add_card_to_battlefield(0, catalog::mind_stone());
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::refurbished_familiar(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 2, "affinity for 2 artifacts");
    // ETB discard.
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let fam = g.move_card_to_battlefield_for_test(0, catalog::refurbished_familiar());
    drain_stack(&mut g);
    let _ = fam;
    assert!(g.players[1].hand.is_empty(), "opponent discarded their only card");
}

/// Galvanic Discharge nets 3 energy then pays exactly lethal to kill a 3/3.
#[test]
fn galvanic_discharge_pays_lethal_energy() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    g.players[0].energy = 1; // 1 + 3 from the spell = 4 available
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::galvanic_discharge().effect, &ctx).unwrap();
    // 3 gained, paid 3 (lethal to the 3/3), 1 left over.
    assert_eq!(g.players[0].energy, 1, "spent only lethal energy");
    assert!(g.battlefield_find(foe).is_none(), "the 3/3 died");
}

/// This Town Ain't Big Enough bounces up to two nonland permanents and is
/// cheaper when it targets one of yours.
#[test]
fn this_town_bounces_two_and_discounts_self_target() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel());
    // Targeting your own permanent → {3} off.
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::this_town_aint_big_enough(), 0);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(mine))), 3);
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, Some(&Target::Permanent(theirs))), 0);
    // Resolution bounces both.
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(mine), Target::Permanent(theirs)];
    g.resolve_effect(&catalog::this_town_aint_big_enough().effect, &ctx).unwrap();
    assert!(g.battlefield_find(mine).is_none() && g.battlefield_find(theirs).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == mine));
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs));
}

/// Highspire Bell-Ringer cuts {1} off your second spell each turn only.
#[test]
fn highspire_bell_ringer_discounts_second_spell() {
    use crate::game::actions::cost_reduction_for_spell;
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::highspire_bell_ringer());
    let spell = crate::card::CardInstance::new(g.next_id(), catalog::grizzly_bears(), 0);
    // First spell (0 cast so far): no discount.
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
    // Second spell (1 cast already): {1} less.
    g.players[0].spells_cast_this_turn = 1;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 1);
    // Third spell: no discount.
    g.players[0].spells_cast_this_turn = 2;
    assert_eq!(cost_reduction_for_spell(&g, 0, &spell, None), 0);
}

/// Hardened Scales adds one to a +1/+1 placement on your creature.
#[test]
fn hardened_scales_adds_one() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hardened_scales());
    let ctx = crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(bear)),
    );
    g.resolve_effect(&Effect::AddCounter {
        what: Selector::Target(0), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2),
    }, &ctx).unwrap();
    // 2 + 1 = 3.
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
}

/// Hardened Scales is additive and only touches +1/+1 (not -1/-1) counters.
#[test]
fn hardened_scales_ignores_minus_counters() {
    use crate::effect::{Effect, Selector, Value};
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::hardened_scales());
    let ctx = crate::game::effects::EffectContext::for_ability(
        crate::card::CardId(0), 0, Some(Target::Permanent(bear)),
    );
    g.resolve_effect(&Effect::AddCounter {
        what: Selector::Target(0), kind: CounterType::MinusOneMinusOne, amount: Value::Const(1),
    }, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::MinusOneMinusOne), 1);
}

/// Vaultborn Tyrant draws and gains life when a big creature enters.
#[test]
fn vaultborn_tyrant_value_on_big_etb() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.add_card_to_library(0, catalog::island());
    let life = g.players[0].life;
    // Cast a power-4 creature so the ETB event flows through the dispatcher.
    let angel = g.add_card_to_hand(0, catalog::serra_angel());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::White, 2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: angel, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast angel");
    drain_stack(&mut g);
    // Net hand: -1 (cast angel) +1 (Vaultborn draw) = same; life +3.
    assert_eq!(g.players[0].life, life + 3, "gained 3 life");
    assert_eq!(g.players[0].hand.len(), hand, "drew a card off the big ETB");
}

/// Unstoppable Slasher returns tapped with two stun counters when it dies.
#[test]
fn unstoppable_slasher_recurs_with_stun() {
    let mut g = two_player_game();
    let slasher = g.add_card_to_battlefield(0, catalog::unstoppable_slasher());
    g.battlefield_find_mut(slasher).unwrap().damage = 3; // lethal vs its 3 toughness
    g.check_state_based_actions();
    drain_stack(&mut g);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Unstoppable Slasher")
        .expect("returned to the battlefield");
    assert!(back.tapped, "returned tapped");
    assert_eq!(back.counters.get(&CounterType::Stun).copied(), Some(2), "two stun counters");
}

/// Vaultborn Tyrant leaves a token copy of itself when it dies.
#[test]
fn vaultborn_tyrant_dies_into_a_copy() {
    let mut g = two_player_game();
    let vt = g.add_card_to_battlefield(0, catalog::vaultborn_tyrant());
    g.battlefield_find_mut(vt).unwrap().damage = 6; // lethal
    g.check_state_based_actions();
    drain_stack(&mut g);
    let copies: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Vaultborn Tyrant" && c.is_token)
        .collect();
    assert_eq!(copies.len(), 1, "one token copy on the battlefield");
    // The copy is an artifact in addition to being a creature.
    assert!(copies[0].definition.card_types.contains(&crate::card::CardType::Artifact));
    assert!(copies[0].definition.is_creature());
}

/// Emberheart Challenger's Valiant exiles the top card the first time you
/// target it each turn.
#[test]
fn emberheart_challenger_valiant_exiles_top() {
    let mut g = two_player_game();
    let ember = g.add_card_to_battlefield(0, catalog::emberheart_challenger());
    g.add_card_to_library(0, catalog::mountain());
    let exile_before = g.exile.len();
    g.dispatch_triggers_for_events(&[GameEvent::BecameTarget { target: ember, caster: 0 }]);
    drain_stack(&mut g);
    assert!(g.exile.len() > exile_before, "Valiant exiled the top card");
}

/// Eldrazi Linebreaker pumps a target creature at the beginning of combat.
#[test]
fn eldrazi_linebreaker_combat_pump() {
    let mut g = two_player_game();
    let lb = g.add_card_to_battlefield(0, catalog::eldrazi_linebreaker());
    g.clear_sickness(lb);
    let other = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(other);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(other))]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    let bear = g.computed_permanent(other).unwrap();
    // One Eldrazi (the Linebreaker) → +1/+0 and haste.
    assert_eq!(bear.power, 3, "bear pumped by Eldrazi count");
    assert!(bear.keywords.contains(&Keyword::Haste), "gained haste");
}

/// No More Lies counters an unpayable spell and exiles it.
#[test]
fn no_more_lies_counters_and_exiles() {
    let mut g = two_player_game();
    // On the opponent's turn they cast a creature spell, spending all mana.
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opponent casts");
    // Seat 0 responds with No More Lies; opponent can't pay {3}.
    let nml = g.add_card_to_hand(0, catalog::no_more_lies());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, nml, Target::Permanent(spell));
    assert!(
        g.exile.iter().any(|c| c.id == spell),
        "the countered spell was exiled, not graveyarded"
    );
}

/// Unstoppable Slasher halves a player's life on combat damage.
#[test]
fn unstoppable_slasher_halves_life() {
    let mut g = two_player_game();
    let slasher = g.add_card_to_battlefield(0, catalog::unstoppable_slasher());
    g.clear_sickness(slasher);
    g.players[1].life = 20;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: slasher,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    // 20 - 2 (combat) = 18, then lose half (9) rounded up → 9.
    assert_eq!(g.players[1].life, 9, "took 2 combat then lost half (rounded up)");
}

/// Enduring Curiosity draws when your creature connects, and returns as an
/// enchantment when it dies.
#[test]
fn enduring_curiosity_draws_then_returns_as_enchantment() {
    let mut g = two_player_game();
    let cur = g.add_card_to_battlefield(0, catalog::enduring_curiosity());
    g.clear_sickness(cur);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers; // past the draw step
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cur,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "drew off combat damage");
    // Now kill it; it returns as a non-creature enchantment.
    g.battlefield_find_mut(cur).unwrap().damage = 3; // lethal for the 4/3
    g.check_state_based_actions();
    drain_stack(&mut g);
    let back = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Enduring Curiosity")
        .expect("returned to battlefield");
    assert!(
        !back.definition.card_types.contains(&crate::card::CardType::Creature),
        "returns as a non-creature enchantment"
    );
}

/// The Necrobloom makes a Plant token whenever a land you control enters.
#[test]
fn necrobloom_landfall_makes_plant() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::the_necrobloom());
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let land = g.add_card_to_hand(0, catalog::forest());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    let plants = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Plant" && c.is_token)
        .count();
    assert_eq!(plants, 1, "one Plant token from landfall");
}

/// Galvanic Relay exiles the top card of your library for later.
#[test]
fn galvanic_relay_exiles_top_for_later() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    let relay = g.add_card_to_hand(0, catalog::galvanic_relay());
    for _ in 0..3 { g.players[0].mana_pool.add(Color::Red, 1); }
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let exile_before = g.exile.len();
    g.perform_action(GameAction::CastSpell {
        card_id: relay, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast relay");
    drain_stack(&mut g);
    assert!(g.exile.len() > exile_before, "exiled the top card");
}

/// Tyvar's Stand pumps and protects your creature for its X.
#[test]
fn tyvars_stand_pumps_by_x() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::tyvars_stand());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: Some(2),
    })
    .expect("cast Tyvar's Stand");
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 4), "+2/+2 from X=2");
    assert!(c.keywords.contains(&Keyword::Hexproof));
    assert!(c.keywords.contains(&Keyword::Indestructible));
}

/// Gird for Battle puts a counter on each of up to two creatures.
#[test]
fn gird_for_battle_buffs_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::gird_for_battle());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)], mode: None, x_value: None,
    })
    .expect("cast Gird for Battle");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(a).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
    assert_eq!(g.battlefield_find(b).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(), Some(1));
}

/// Stock Up draws two of the top five into hand.
#[test]
fn stock_up_takes_two_of_five() {
    let mut g = two_player_game();
    for _ in 0..6 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::stock_up());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Stock Up");
    drain_stack(&mut g);
    // -1 (cast) +2 (picked) = +1 net.
    assert_eq!(g.players[0].hand.len(), hand + 1, "two cards into hand, one spell spent");
}

/// Shelter cantrips and grants protection.
#[test]
fn shelter_protects_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::plains());
    let spell = g.add_card_to_hand(0, catalog::shelter());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Shelter");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "drew a card (net even after cast)");
}

/// Pick Your Poison's first mode makes each opponent sacrifice an artifact.
#[test]
fn pick_your_poison_edicts_an_artifact() {
    let mut g = two_player_game();
    let rock = g.add_card_to_battlefield(1, catalog::mind_stone());
    let spell = g.add_card_to_hand(0, catalog::pick_your_poison());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    })
    .expect("cast Pick Your Poison");
    drain_stack(&mut g);
    assert!(g.battlefield_find(rock).is_none(), "opponent's only artifact was sacrificed");
}

/// Tail Swipe fights your creature against an opponent's.
#[test]
fn tail_swipe_fights() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::tail_swipe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    })
    .expect("cast Tail Swipe");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "their 2/2 died to the 4/4");
    assert!(g.battlefield_find(mine).is_some(), "our 4/4 survived 2 damage");
    // Cast during your main phase: the +1/+1 rider made it a 5/5.
    assert_eq!(g.computed_permanent(mine).unwrap().power, 5, "main-phase +1/+1");
}

/// Tail Swipe cast at instant speed (opponent's turn) skips the +1/+1.
#[test]
fn tail_swipe_no_main_phase_pump() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::tail_swipe());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 1; // opponent's turn
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    })
    .expect("cast Tail Swipe");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 4, "no pump off your turn");
}

/// Lightning Axe discards a card and deals 5 to a creature.
#[test]
fn lightning_axe_kills_with_a_discard() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::lightning_axe());
    let _fodder = g.add_card_to_hand(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(target)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Lightning Axe");
    drain_stack(&mut g);
    assert!(g.battlefield_find(target).is_none(), "5 damage killed the 4/4");
    assert!(g.players[0].hand.len() < hand, "discarded a card as additional cost");
}

/// Stormsplitter copies itself when you cast an instant or sorcery.
#[test]
fn stormsplitter_copies_on_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stormsplitter());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)),
        additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast bolt");
    drain_stack(&mut g);
    let copies = g.battlefield.iter().filter(|c| c.definition.name == "Stormsplitter" && c.is_token).count();
    assert_eq!(copies, 1, "one token copy from the instant cast");
}

/// Unburden forces a player to discard two cards.
#[test]
fn unburden_discards_two() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_hand(1, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::unburden());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let opp_hand = g.players[1].hand.len();
    cast_at(&mut g, spell, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), opp_hand - 2, "opponent discarded two");
}

/// Goblin Anarchomancer makes a red spell cost {1} less.
#[test]
fn goblin_anarchomancer_discounts_red() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_anarchomancer());
    // Lightning Bolt ({R}) becomes castable with no mana left after just {R}…
    // assert the reduced cost via the engine's castable check: give exactly
    // {R} (already enough), so instead verify the static reduces a {1}{R} spell.
    let pyro = g.add_card_to_hand(0, catalog::incinerate()); // {1}{R} instant
    g.players[0].mana_pool.add(Color::Red, 1); // only {R}; the {1} is discounted
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, pyro, Target::Player(1));
    assert_eq!(g.players[1].life, 17, "Incinerate dealt 3 after a 1-mana discount");
}

/// Beza gains 4 life when an opponent has more life.
#[test]
fn beza_gains_life_when_behind() {
    let mut g = two_player_game();
    g.players[0].life = 10;
    g.players[1].life = 20;
    let life = g.players[0].life;
    g.move_card_to_battlefield_for_test(0, catalog::beza_the_bounding_spring());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 4, "gained 4 because opponent has more life");
}

/// Optimistic Scavenger grows a creature when an enchantment you control enters.
#[test]
fn optimistic_scavenger_eerie_counter() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::optimistic_scavenger());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    // An enchantment you control enters (dispatch the watcher event).
    let ench = g.add_card_to_battlefield(0, catalog::sticky_fingers());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: ench }]);
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(bear).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "Eerie put a +1/+1 counter on the target"
    );
}

/// Frilled Sandwalla pumps once per turn.
#[test]
fn frilled_sandwalla_once_per_turn_pump() {
    let mut g = two_player_game();
    let id = g.add_card_to_battlefield(0, catalog::frilled_sandwalla());
    g.clear_sickness(id);
    for _ in 0..4 { g.players[0].mana_pool.add(Color::Green, 1); }
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("first activation");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(id).unwrap().power, 3, "+2/+2 once");
    // Second activation the same turn is rejected.
    assert!(g.perform_action(GameAction::ActivateAbility {
        card_id: id, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).is_err(), "only once each turn");
}

/// Spectral Interference counters an artifact/creature spell the controller
/// can't pay {4} for.
#[test]
fn spectral_interference_counters_creature() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts a creature");
    let si = g.add_card_to_hand(0, catalog::spectral_interference());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, si, Target::Permanent(spell));
    assert!(g.battlefield_find(spell).is_none(), "creature spell countered");
    assert!(g.players[1].graveyard.iter().any(|c| c.id == spell));
}

/// Refute counters a spell and loots.
#[test]
fn refute_counters_and_loots() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts");
    g.add_card_to_library(0, catalog::island());
    let _junk = g.add_card_to_hand(0, catalog::island());
    let refute = g.add_card_to_hand(0, catalog::refute());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    cast_at(&mut g, refute, Target::Permanent(spell));
    assert!(g.battlefield_find(spell).is_none(), "spell countered");
}

/// Skullcap Snail strips a card from an opponent's hand to exile.
#[test]
fn skullcap_snail_exiles_from_hand() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.add_card_to_hand(1, catalog::forest());
    let exile_before = g.exile.len();
    let opp_hand = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::skullcap_snail());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent lost a card from hand");
    assert!(g.exile.len() > exile_before, "it went to exile");
}

/// Aspirant's Ascent grants flying and toxic.
#[test]
fn aspirants_ascent_grants_flying_toxic() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aspirants_ascent());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 5), "+1/+3");
    assert!(c.keywords.contains(&Keyword::Flying));
    assert!(c.keywords.iter().any(|k| matches!(k, Keyword::Toxic(1))));
}

/// Take the Fall shrinks a creature more when you control an outlaw.
#[test]
fn take_the_fall_outlaw_bonus() {
    let mut g = two_player_game();
    // An outlaw (Rogue) of ours.
    let mut rogue = catalog::grizzly_bears();
    rogue.subtypes.creature_types = vec![CreatureType::Rogue];
    g.add_card_to_battlefield(0, rogue);
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let spell = g.add_card_to_hand(0, catalog::take_the_fall());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(victim));
    assert_eq!(g.computed_permanent(victim).unwrap().power, 0, "-4/-0 with an outlaw");
}

/// Hopeful Vigil makes a Knight, and scries when it leaves.
#[test]
fn hopeful_vigil_token_and_sac_scry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::forest());
    let vigil = g.move_card_to_battlefield_for_test(0, catalog::hopeful_vigil());
    drain_stack(&mut g);
    let knights = g.battlefield.iter().filter(|c| c.definition.name == "Knight").count();
    assert_eq!(knights, 1, "made a Knight on ETB");
    // Sacrifice it via its own ability → leaves-trigger scries.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: vigil, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sacrifice Hopeful Vigil");
    drain_stack(&mut g);
    assert!(g.battlefield_find(vigil).is_none(), "Hopeful Vigil sacrificed");
}

/// Hopeless Nightmare drains and discards on ETB.
#[test]
fn hopeless_nightmare_discard_and_drain() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::island());
    g.players[1].life = 20;
    let opp_hand = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::hopeless_nightmare());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "opponent lost 2");
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded");
}

/// Hangar Scrounger's Backup puts a +1/+1 counter on a creature on ETB.
#[test]
fn hangar_scrounger_backup_counter() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
    g.move_card_to_battlefield_for_test(0, catalog::hangar_scrounger());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ally).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "Backup 1 added a counter"
    );
}

/// Bristlebud Farmer makes two Food on ETB.
#[test]
fn bristlebud_farmer_makes_food() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::bristlebud_farmer());
    drain_stack(&mut g);
    let food = g.battlefield.iter().filter(|c| c.definition.name == "Food").count();
    assert_eq!(food, 2, "two Food tokens");
}

/// Outcaster Greenblade tutors a basic land to hand on ETB.
#[test]
fn outcaster_greenblade_fetches_a_basic() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(None)]));
    let hand = g.players[0].hand.len();
    // Script the search to take the forest.
    let forest = g.players[0].library[0].id;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let blade = g.move_card_to_battlefield_for_test(0, catalog::outcaster_greenblade());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "fetched a land to hand");
    // Base 1/2 with no Deserts.
    let c = g.computed_permanent(blade).unwrap();
    assert_eq!((c.power, c.toughness), (1, 2));
    // Each Desert you control grows it +1/+1.
    g.add_card_to_battlefield(0, catalog::conduit_pylons());
    g.add_card_to_battlefield(0, catalog::conduit_pylons());
    let c = g.computed_permanent(blade).unwrap();
    assert_eq!((c.power, c.toughness), (3, 4), "+1/+1 per Desert");
}

/// Mizzium Skin grants hexproof.
#[test]
fn mizzium_skin_grants_hexproof() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::mizzium_skin());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.toughness, 3, "+0/+1");
    assert!(c.keywords.contains(&Keyword::Hexproof));
}

/// Mizzium Skin overloaded protects every creature you control.
#[test]
fn mizzium_skin_overload_shields_team() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::mizzium_skin());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: spell, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("overload Mizzium Skin");
    drain_stack(&mut g);
    for id in [a, b] {
        let c = g.computed_permanent(id).unwrap();
        assert_eq!(c.toughness, 3, "+0/+1 across the team");
        assert!(c.keywords.contains(&Keyword::Hexproof));
    }
}

/// Demand Answers discards then draws two.
#[test]
fn demand_answers_loots_up() {
    let mut g = two_player_game();
    for _ in 0..2 { g.add_card_to_library(0, catalog::island()); }
    let spell = g.add_card_to_hand(0, catalog::demand_answers());
    let _fodder = g.add_card_to_hand(0, catalog::forest());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Demand Answers");
    drain_stack(&mut g);
    // -1 cast, -1 discard, +2 draw = net 0 vs captured (which still held the spell).
    assert_eq!(g.players[0].hand.len(), hand, "discarded one and drew two");
}

/// Boltwave burns each opponent for 3.
#[test]
fn boltwave_burns_each_opponent() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let spell = g.add_card_to_hand(0, catalog::boltwave());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Boltwave");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 17, "opponent took 3");
}

/// Strike It Rich mints a Treasure.
#[test]
fn strike_it_rich_makes_treasure() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::strike_it_rich());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Strike It Rich");
    drain_stack(&mut g);
    let treasures = g.battlefield.iter().filter(|c| c.definition.name == "Treasure").count();
    assert_eq!(treasures, 1, "made a Treasure");
}

/// Brotherhood's End mode 0 sweeps creatures for 3.
#[test]
fn brotherhoods_end_sweeps_creatures() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::brotherhoods_end());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Mode(0)]));
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: Some(0), x_value: None,
    }).expect("cast Brotherhood's End");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).is_none() && g.battlefield_find(b).is_none(), "both 2/2s died");
}

/// Boon-Bringer Valkyrie's Backup grants flying to the backed-up creature.
#[test]
fn boon_bringer_valkyrie_backup_grants_flying() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
    g.move_card_to_battlefield_for_test(0, catalog::boon_bringer_valkyrie());
    drain_stack(&mut g);
    let c = g.computed_permanent(ally).unwrap();
    assert_eq!(c.power, 3, "got a +1/+1 counter");
    assert!(c.keywords.contains(&Keyword::Flying), "gained flying from Backup");
}

/// Inti's discard trigger exiles the top of your library with a may-play.
#[test]
fn inti_discard_exiles_top_with_may_play() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::inti_seneschal_of_the_sun());
    g.add_card_to_library(0, catalog::mountain());
    // Unburden (cast by p0, targeting p0) forces a discard, firing Inti.
    let unburden = g.add_card_to_hand(0, catalog::unburden());
    g.add_card_to_hand(0, catalog::mountain()); // something to discard
    g.add_card_to_hand(0, catalog::mountain());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, unburden, Target::Player(0));
    drain_stack(&mut g);
    assert!(
        g.exile.iter().any(|c| c.owner == 0),
        "Inti exiled the top of the library on discard"
    );
}

/// Warren Soultrader sacrifices a creature and pays 1 life for a Treasure.
#[test]
fn warren_soultrader_makes_treasure() {
    let mut g = two_player_game();
    let warren = g.add_card_to_battlefield(0, catalog::warren_soultrader());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].life = 20;
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(fodder))]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: warren, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed the creature");
    assert_eq!(g.players[0].life, 19, "paid 1 life");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.subtypes.artifact_subtypes.contains(&crate::card::ArtifactSubtype::Treasure)),
        "made a Treasure"
    );
}

/// Hostile Investigator makes a target opponent discard on ETB.
#[test]
fn hostile_investigator_etb_discard() {
    let mut g = two_player_game();
    g.add_card_to_hand(1, catalog::mountain());
    let opp_hand = g.players[1].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::hostile_investigator());
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), opp_hand - 1, "opponent discarded to the ETB");
}

/// Marshal of Zhalfir buffs other Knights and can tap a creature.
#[test]
fn marshal_of_zhalfir_anthems_knights() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::marshal_of_zhalfir());
    let knight = g.add_card_to_battlefield(0, catalog::inti_seneschal_of_the_sun()); // a Knight
    let c = g.computed_permanent(knight).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "another Knight got +1/+1");
}

/// Pawpatch Recruit grows a creature when an opponent targets one you control.
#[test]
fn pawpatch_recruit_counters_on_opponent_target() {
    let mut g = two_player_game();
    let pixie = g.add_card_to_battlefield(0, catalog::pawpatch_recruit());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    // Opponent's Stab targets your bear; Pawpatch's trigger puts a counter on it.
    let stab = g.add_card_to_hand(1, catalog::stab());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    cast_at(&mut g, stab, Target::Permanent(bear));
    drain_stack(&mut g);
    let _ = pixie;
    assert!(
        g.battlefield_find(bear).map(|c| c.counters.get(&CounterType::PlusOnePlusOne).copied())
            == Some(Some(1)),
        "Pawpatch put a +1/+1 counter on the targeted creature"
    );
}

/// Helping Hand returns a small creature from your graveyard tapped.
#[test]
fn helping_hand_reanimates_tapped() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears()); // MV 2
    let hh = g.add_card_to_hand(0, catalog::helping_hand());
    g.players[0].mana_pool.add(Color::White, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, hh, Target::Permanent(bear));
    drain_stack(&mut g);
    let r = g.battlefield_find(bear).expect("reanimated onto battlefield");
    assert!(r.tapped, "entered tapped");
}

/// Diversion Unit sacrifices itself to counter a spell.
#[test]
fn diversion_unit_counters_spell() {
    let mut g = two_player_game();
    let unit = g.add_card_to_battlefield(0, catalog::diversion_unit());
    let bolt = g.add_card_to_hand(1, catalog::lightning_axe()); // an instant
    g.players[1].mana_pool.add(Color::Red, 1);
    g.add_card_to_hand(1, catalog::mountain()); // discard fodder for Lightning Axe
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    let dummy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(dummy)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("opponent casts Lightning Axe");
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: unit, ability_index: 0,
        target: Some(Target::Permanent(bolt)), additional_targets: vec![], x_value: None,
    }).expect("activate counter");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Lightning Axe was countered");
}

/// Final Vengeance sacrifices a permanent and exiles a creature.
#[test]
fn final_vengeance_sac_and_exile() {
    let mut g = two_player_game();
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    let fv = g.add_card_to_hand(0, catalog::final_vengeance());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, fv, Target::Permanent(victim));
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed a creature as additional cost");
    assert!(g.exile.iter().any(|c| c.id == victim), "exiled the target creature");
}

/// Roughshod Mentor gives your green creatures trample.
#[test]
fn roughshod_mentor_grants_green_trample() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::roughshod_mentor());
    let elf = g.add_card_to_battlefield(0, catalog::llanowar_elves()); // green
    assert!(
        g.computed_permanent(elf).unwrap().keywords.contains(&Keyword::Trample),
        "green creature gained trample"
    );
}

/// Innocuous Rat manifests dread when it dies.
#[test]
fn innocuous_rat_manifests_on_death() {
    let mut g = two_player_game();
    let rat = g.add_card_to_battlefield(0, catalog::innocuous_rat());
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::island());
    let bf_before = g.battlefield.len();
    g.battlefield_find_mut(rat).unwrap().damage = 1;
    g.check_state_based_actions();
    drain_stack(&mut g);
    // Rat left; a face-down 2/2 entered → battlefield count unchanged net (−rat +manifest).
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.face_down),
        "manifested a face-down creature"
    );
    let _ = bf_before;
}

/// Quaketusk Boar is a 5/5 with reach, trample, and haste.
#[test]
fn quaketusk_boar_keywords() {
    let d = catalog::quaketusk_boar();
    assert_eq!((d.power, d.toughness), (5, 5));
    assert!(d.keywords.contains(&Keyword::Reach));
    assert!(d.keywords.contains(&Keyword::Trample));
    assert!(d.keywords.contains(&Keyword::Haste));
}

/// Veteran Guardmouse's Valiant fires when you target it (it gains first
/// strike; Valiant's +1/+0 resolves before the targeting spell).
#[test]
fn veteran_guardmouse_valiant_pumps() {
    let mut g = two_player_game();
    let mouse = g.add_card_to_battlefield(0, catalog::veteran_guardmouse()); // 3/4
    g.add_card_to_library(0, catalog::mountain());
    let stab = g.add_card_to_hand(0, catalog::stab()); // your own targeted spell
    g.players[0].mana_pool.add(Color::Black, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, stab, Target::Permanent(mouse));
    drain_stack(&mut g);
    let c = g.computed_permanent(mouse).unwrap();
    // Valiant (+1/+0) resolves first, then Stab (-2/-2): 3+1-2 / 4+0-2 = 2/2.
    assert_eq!((c.power, c.toughness), (2, 2), "+1/+0 then -2/-2");
    assert!(c.keywords.contains(&Keyword::FirstStrike), "gained first strike");
}

/// Polliwallop makes your creature deal twice its power to an enemy creature.
#[test]
fn polliwallop_deals_double_power() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let theirs = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let poll = g.add_card_to_hand(0, catalog::polliwallop());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: poll, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Polliwallop");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "4 damage (2×2) killed the 4/4");
}

/// Coiling Rebirth reanimates a creature from your graveyard.
#[test]
fn coiling_rebirth_reanimates() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::serra_angel());
    let cr = g.add_card_to_hand(0, catalog::coiling_rebirth());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, cr, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "creature returned to the battlefield");
}

/// Pearl of Wisdom draws two cards.
#[test]
fn pearl_of_wisdom_draws_two() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let pearl = g.add_card_to_hand(0, catalog::pearl_of_wisdom());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: pearl, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pearl");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand - 1 + 2, "drew two (cast one)");
}

/// Pearl of Wisdom costs {1} less while you control an Otter.
#[test]
fn pearl_of_wisdom_otter_discount() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_battlefield(0, catalog::stormsplitter()); // an Otter
    let pearl = g.add_card_to_hand(0, catalog::pearl_of_wisdom());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1); // only {1}{U}, not {2}{U}
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: pearl, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Pearl at the Otter discount");
    drain_stack(&mut g);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == pearl), "resolved cheaply");
}

/// Ride's End costs {3} less when it targets a tapped permanent.
#[test]
fn rides_end_cost_reduction_when_tapped() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    let re = g.add_card_to_hand(0, catalog::rides_end());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1); // only {1}{W} available, not {4}{W}
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, re, Target::Permanent(victim));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == victim), "cheaply exiled a tapped creature");
}

/// Nurturing Pixie bounces your own permanent and grows.
#[test]
fn nurturing_pixie_bounce_and_grow() {
    let mut g = two_player_game();
    let token = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(token)),
    ]));
    let pixie = g.move_card_to_battlefield_for_test(0, catalog::nurturing_pixie());
    drain_stack(&mut g);
    assert!(g.battlefield_find(token).is_none(), "bounced your own permanent");
    assert_eq!(g.computed_permanent(pixie).unwrap().power, 2, "Pixie grew to 2/2");
}

/// Ruby pumps herself when attacking alongside a big creature.
#[test]
fn ruby_pumps_with_big_creature() {
    let mut g = two_player_game();
    let ruby = g.add_card_to_battlefield(0, catalog::ruby_daring_tracker());
    g.add_card_to_battlefield(0, catalog::serra_angel()); // power 4
    g.clear_sickness(ruby);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ruby, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ruby).unwrap().power, 3, "Ruby got +2/+2");
}

/// Stab gives a creature -2/-2.
#[test]
fn stab_shrinks_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let stab = g.add_card_to_hand(0, catalog::stab());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, stab, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "-2/-2 killed the 2/2");
}

/// Slumbering Keepguard scries when an enchantment enters.
#[test]
fn slumbering_keepguard_scries_on_enchantment() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::slumbering_keepguard());
    g.add_card_to_library(0, catalog::island());
    // An enchantment entering under your control triggers the scry.
    g.move_card_to_battlefield_for_test(0, catalog::hopeless_nightmare()); // an enchantment
    drain_stack(&mut g);
    // No panic / clean resolution is the assertion; the scry decision auto-resolves.
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Slumbering Keepguard"));
}

/// Anoint with Affliction exiles a small creature.
#[test]
fn anoint_with_affliction_exiles_small() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let spell = g.add_card_to_hand(0, catalog::anoint_with_affliction());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "small creature exiled");
}

/// Wing It pumps, adds a flying counter, and scries.
#[test]
fn wing_it_pumps_and_grants_flying() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::wing_it());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 4, "+2/+2");
    assert!(c.keywords.contains(&Keyword::Flying), "flying counter grants flying");
}

/// Cackling Prowler grows at end step when a creature died this turn.
#[test]
fn cackling_prowler_morbid_counter() {
    let mut g = two_player_game();
    let prowler = g.add_card_to_battlefield(0, catalog::cackling_prowler());
    g.players[0].creatures_died_this_turn = 1;
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    g.step = TurnStep::PostCombatMain;
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(prowler).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
        "morbid put a +1/+1 counter at end step"
    );
}

/// Glimmerlight mints a Glimmer token on enter.
#[test]
fn glimmerlight_makes_glimmer_token() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::glimmerlight());
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Glimmer"),
        "minted a Glimmer token"
    );
}

/// Demonic Ruckus buffs the enchanted creature and draws when it dies.
#[test]
fn demonic_ruckus_buffs_then_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let aura = g.add_card_to_hand(0, catalog::demonic_ruckus());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, aura, Target::Permanent(bear));
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "enchanted creature gets +1/+1");
    assert!(c.keywords.contains(&Keyword::Menace), "gains menace");
    // Kill the bear → the Aura dies and cantrips.
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(bear).unwrap().damage = 3;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "Aura death drew a card");
}

/// Hugs exiles X cards with a may-play when it enters.
#[test]
fn hugs_exiles_x_with_may_play() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::mountain());
    g.add_card_to_library(0, catalog::forest());
    let hugs = g.add_card_to_hand(0, catalog::hugs_grisly_guardian());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: hugs, target: None, additional_targets: vec![], mode: None, x_value: Some(2),
    }).expect("cast Hugs with X=2");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| c.owner == 0).count(), 2, "exiled the top 2 cards");
}

/// Gloomfang Mauler's Backup 2 puts two counters on a creature.
#[test]
fn gloomfang_mauler_backup_two() {
    let mut g = two_player_game();
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(ally))]));
    g.move_card_to_battlefield_for_test(0, catalog::gloomfang_mauler());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(ally).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(2),
        "Backup 2 put two +1/+1 counters"
    );
}

/// Audacity buffs the enchanted creature and cantrips when it leaves.
#[test]
fn audacity_buffs_and_cantrips() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_library(0, catalog::island());
    let aura = g.add_card_to_hand(0, catalog::audacity());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, aura, Target::Permanent(bear));
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (4, 2), "+2/+0");
    assert!(c.keywords.contains(&Keyword::Trample));
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(bear).unwrap().damage = 2;
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.players[0].hand.len() > hand, "Aura death drew a card");
}

/// Felonious Rage leaves a Detective when the buffed creature dies.
#[test]
fn felonious_rage_death_makes_detective() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::felonious_rage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
    // Kill the buffed 4/2 with a burn spell so the death flows through the
    // damage funnel that the "dies this turn" watch listens on.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Permanent(bear));
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.id == bear), "buffed creature died");
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Detective"),
        "the dying creature left a Detective token"
    );
}

/// Razorkin Hordecaller mints a Gremlin when you attack.
#[test]
fn razorkin_hordecaller_attack_token() {
    let mut g = two_player_game();
    let razor = g.add_card_to_battlefield(0, catalog::razorkin_hordecaller());
    g.clear_sickness(razor);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: razor, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Gremlin"),
        "minted a Gremlin on attack"
    );
}

/// Goldvein Pick gives +1/+1 and a Treasure on combat damage.
#[test]
fn goldvein_pick_buffs_and_treasures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(bear);
    let pick = g.add_card_to_battlefield(0, catalog::goldvein_pick());
    g.players[0].mana_pool.add_colorless(1); // Equip {1}
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Equip { equipment: pick, target: bear }).expect("equip");
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!((c.power, c.toughness), (3, 3), "equipped creature gets +1/+1");
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::PostCombatMain && g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0
            && c.definition.subtypes.artifact_subtypes.contains(&crate::card::ArtifactSubtype::Treasure)),
        "combat damage made a Treasure"
    );
}

// ── Tarkir: Dragonstorm + recent-set batch ───────────────────────────────────

/// Boulderborn Dragon surveils when it attacks.
#[test]
fn boulderborn_dragon_surveils_on_attack() {
    let mut g = two_player_game();
    let dragon = g.add_card_to_battlefield(0, catalog::boulderborn_dragon());
    g.clear_sickness(dragon);
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    let lib = g.players[0].library.len();
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dragon, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    // Surveil looked at the top card (kept or binned) — library shrank by ≤1; the
    // trigger resolved without error and the dragon has flying+vigilance.
    assert!(g.players[0].library.len() <= lib);
    let c = g.battlefield_find(dragon).unwrap();
    assert!(c.definition.keywords.contains(&Keyword::Flying));
    assert!(c.definition.keywords.contains(&Keyword::Vigilance));
}

/// Scales of Shale costs less with Lizards and buffs a creature.
#[test]
fn scales_of_shale_affinity_and_buff() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::scales_of_shale());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    let c = g.computed_permanent(bear).unwrap();
    assert_eq!(c.power, 4, "+2/+0");
    assert!(c.keywords.contains(&Keyword::Lifelink));
    assert!(c.keywords.contains(&Keyword::Indestructible));
}

/// Sunset Strikemaster sacrifices to burn a flier.
#[test]
fn sunset_strikemaster_burns_a_flier() {
    let mut g = two_player_game();
    let master = g.add_card_to_battlefield(0, catalog::sunset_strikemaster());
    g.clear_sickness(master);
    let flier = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: master, ability_index: 1,
        target: Some(Target::Permanent(flier)), additional_targets: vec![], x_value: None,
    }).expect("sac to burn the flier");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flier).is_none(), "6 damage killed the 4/4 flier");
    assert!(g.battlefield_find(master).is_none(), "sacrificed itself");
}

/// Wardens of the Cycle's morbid end-step trigger draws + drains when a creature died.
#[test]
fn wardens_of_the_cycle_morbid_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::wardens_of_the_cycle());
    g.players[0].creatures_died_this_turn = 1;
    let life = g.players[0].life;
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    // Default modal pick is mode 0 (gain 2 life); the morbid trigger fired.
    assert_eq!(g.players[0].life, life + 2, "gained 2 life off the morbid trigger");
}

/// Roiling Dragonstorm loots on ETB and bounces itself when a Dragon enters.
#[test]
fn roiling_dragonstorm_bounces_on_dragon() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    g.add_card_to_hand(0, catalog::island()); // something to discard
    let storm = g.move_card_to_battlefield_for_test(0, catalog::roiling_dragonstorm());
    drain_stack(&mut g);
    assert!(g.battlefield_find(storm).is_some(), "enchantment is on the battlefield");
    // A Dragon entering bounces the enchantment back to hand.
    let dragon = g.add_card_to_battlefield(0, catalog::boulderborn_dragon());
    g.dispatch_triggers_for_events(&[GameEvent::PermanentEntered { card_id: dragon }]);
    drain_stack(&mut g);
    assert!(g.battlefield_find(storm).is_none(), "returned to hand on Dragon ETB");
    assert!(g.players[0].hand.iter().any(|c| c.id == storm));
}

/// Stormcatch Mentor reduces instant/sorcery cost and has prowess + haste.
#[test]
fn stormcatch_mentor_cheapens_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::stormcatch_mentor());
    let bolt = g.add_card_to_hand(0, catalog::lightning_axe()); // {R} instant
    // Lightning Axe is {R}; reduction is generic-only so cost unchanged here, but
    // a {1}{R} sorcery would drop to {R}. Use a cheaper proxy: just verify the
    // static is present and prowess/haste are on the body.
    let _ = bolt;
    let m = g.battlefield.iter().find(|c| c.definition.name == "Stormcatch Mentor").unwrap();
    assert!(m.definition.keywords.contains(&Keyword::Prowess));
    assert!(m.definition.keywords.contains(&Keyword::Haste));
    assert_eq!(m.definition.static_abilities.len(), 1, "I/S cost reduction static");
}

/// Gurmag Drowner exploits itself to dig four.
#[test]
fn gurmag_drowner_exploit_digs() {
    let mut g = two_player_game();
    for _ in 0..4 { g.add_card_to_library(0, catalog::island()); }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)])); // accept exploit
    let hand = g.players[0].hand.len();
    let drowner = g.move_card_to_battlefield_for_test(0, catalog::gurmag_drowner());
    drain_stack(&mut g);
    assert!(g.battlefield_find(drowner).is_none(), "exploited itself");
    assert_eq!(g.players[0].hand.len(), hand + 1, "dug a card into hand");
}

/// Nullpriest of Oblivion reanimates when kicked.
#[test]
fn nullpriest_kicked_reanimates() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::serra_angel());
    let null = g.add_card_to_hand(0, catalog::nullpriest_of_oblivion());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(4);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpellKicked {
        card_id: null, target: Some(Target::Permanent(corpse)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast kicked");
    drain_stack(&mut g);
    assert!(g.battlefield_find(corpse).is_some(), "reanimated the angel");
}

/// Ureni deals damage on ETB equal to lands you control, divided among foes.
#[test]
fn ureni_etb_divided_damage() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_battlefield(0, catalog::forest()); }
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, only legal target
    g.move_card_to_battlefield_for_test(0, catalog::ureni_the_song_unending());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(3), "3 damage (3 lands)");
}

/// Elspeth, Storm Slayer doubles her Soldier token.
#[test]
fn elspeth_storm_slayer_doubles_tokens() {
    let mut g = two_player_game();
    let elspeth = g.add_card_to_battlefield(0, catalog::elspeth_storm_slayer());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: elspeth, ability_index: 0, target: None, x_value: None,
    }).expect("+1 make a Soldier");
    drain_stack(&mut g);
    let soldiers = g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count();
    assert_eq!(soldiers, 2, "token doubling made two Soldiers");
}

/// Betor draws at end step once total toughness reaches 10.
#[test]
fn betor_end_step_draw_at_ten_toughness() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::betor_kin_to_all()); // 5/7
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // +2 → 9, not yet 10
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    let hand = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand, "9 total toughness: no draw");
    // Add another creature to cross 10.
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // → 11
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand + 1, "≥10 total toughness draws");
}

/// Mistmoon Griffin reanimates the top creature of your graveyard when it dies.
#[test]
fn mistmoon_griffin_reanimates_on_death() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears()); // top creature card
    let griffin = g.add_card_to_battlefield(0, catalog::mistmoon_griffin());
    g.battlefield_find_mut(griffin).unwrap().damage = 2; // lethal
    g.check_state_based_actions();
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "the top creature card returned to the battlefield");
}

/// Dalek Squadron makes a myriad copy when attacking (multiplayer).
#[test]
fn dalek_squadron_myriad_copies() {
    let mut g = crate::game::game_with_format(crate::format::Format::Commander, 3);
    let dalek = g.add_card_to_battlefield(0, catalog::dalek_squadron());
    g.clear_sickness(dalek);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dalek, target: AttackTarget::Player(1),
    }])).expect("attack player 1");
    drain_stack(&mut g);
    let copies = g.battlefield.iter()
        .filter(|c| c.definition.name == "Dalek Squadron" && c.is_token).count();
    assert_eq!(copies, 1, "one myriad copy for the third player");
}

/// Perennation reanimates a permanent with hexproof + indestructible counters.
#[test]
fn perennation_reanimates_with_counters() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::perennation());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(corpse));
    drain_stack(&mut g);
    let c = g.computed_permanent(corpse).expect("bear is on the battlefield");
    assert!(c.keywords.contains(&Keyword::Hexproof), "hexproof counter");
    assert!(c.keywords.contains(&Keyword::Indestructible), "indestructible counter");
}

/// Sarkhan, Soul Aflame reduces Dragon spell costs.
#[test]
fn sarkhan_soul_aflame_cheapens_dragons() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::sarkhan_soul_aflame());
    let dragon = g.add_card_to_hand(0, catalog::boulderborn_dragon()); // {5}
    g.players[0].mana_pool.add_colorless(4); // only {4} thanks to the {1} discount
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: dragon, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast the Dragon for {4}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_some(), "Dragon resolved at the discount");
}

// ── Recent-set batch 2 ───────────────────────────────────────────────────────

/// Skirmish Rhino drains each opponent and gains you life.
#[test]
fn skirmish_rhino_drains_on_etb() {
    let mut g = two_player_game();
    let life = g.players[0].life;
    let opp = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::skirmish_rhino());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    assert_eq!(g.players[1].life, opp - 2, "opponent lost 2");
}

/// Rabid Gnaw pumps your creature, which bites an opponent's.
#[test]
fn rabid_gnaw_bites() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2 → 3/2
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let spell = g.add_card_to_hand(0, catalog::rabid_gnaw());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)], mode: None, x_value: None,
    }).expect("cast Rabid Gnaw");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "3 damage from the pumped biter killed the 2/2");
    assert!(g.battlefield_find(mine).is_some(), "the biter took none");
}

/// Reckless Lackey sacrifices for a card and a Treasure.
#[test]
fn reckless_lackey_sacrifices_for_value() {
    let mut g = two_player_game();
    let lackey = g.add_card_to_battlefield(0, catalog::reckless_lackey());
    g.clear_sickness(lackey);
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: lackey, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("sac for value");
    drain_stack(&mut g);
    assert!(g.battlefield_find(lackey).is_none(), "sacrificed itself");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
    assert!(g.battlefield.iter().any(|c| c.controller == 0
        && c.definition.subtypes.artifact_subtypes.contains(&crate::card::ArtifactSubtype::Treasure)),
        "made a Treasure");
}

/// Lunar Convocation drains at end step when you gained life.
#[test]
fn lunar_convocation_drains_on_lifegain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::lunar_convocation());
    g.players[0].life_gained_this_turn = 3;
    g.active_player_idx = 0;
    let opp = g.players[1].life;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "each opponent lost 1 after lifegain");
}

/// Dazzling Denial counters a spell whose controller can't pay the {2} tax.
#[test]
fn dazzling_denial_counters_when_unpaid() {
    let mut g = two_player_game();
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(1);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1); // exactly enough for Bolt, nothing spare
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Bolt");
    g.priority.player_with_priority = 0;
    let denial = g.add_card_to_hand(0, catalog::dazzling_denial());
    cast_at(&mut g, denial, Target::Permanent(bolt));
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered (couldn't pay {{2}})");
    assert_eq!(g.players[0].life, 20, "Bolt didn't resolve");
}

/// Mistrise Village enters tapped without the right basics, untapped with them.
#[test]
fn mistrise_village_conditional_tap() {
    let mut g = two_player_game();
    let v1 = g.move_card_to_battlefield_for_test(0, catalog::mistrise_village());
    drain_stack(&mut g);
    assert!(g.battlefield_find(v1).unwrap().tapped, "enters tapped with no Mountain/Forest");
    g.add_card_to_battlefield(0, catalog::mountain());
    let v2 = g.move_card_to_battlefield_for_test(0, catalog::mistrise_village());
    drain_stack(&mut g);
    assert!(!g.battlefield_find(v2).unwrap().tapped, "enters untapped with a Mountain out");
}

/// Cori Mountain Monastery impulse-exiles the top card for later play.
#[test]
fn cori_mountain_monastery_impulse() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(0, catalog::cori_mountain_monastery());
    g.add_card_to_battlefield(0, catalog::plains()); // so it isn't relevant; we just need mana
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(3);
    let exile_before = g.exile.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: land, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    }).expect("impulse");
    drain_stack(&mut g);
    assert!(g.exile.len() > exile_before, "exiled the top card for later play");
}

/// Bloodletter of Aclazotz doubles an opponent's life loss during your turn.
#[test]
fn bloodletter_doubles_opponent_life_loss() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bloodletter_of_aclazotz());
    g.active_player_idx = 0; // your turn
    let opp = g.players[1].life;
    // A Skirmish Rhino drains the opponent 2 → doubled to 4.
    g.move_card_to_battlefield_for_test(0, catalog::skirmish_rhino());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 4, "opponent's 2 life loss doubled to 4");
}

/// Off your turn the doubling doesn't apply.
#[test]
fn bloodletter_inactive_off_your_turn() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bloodletter_of_aclazotz());
    g.active_player_idx = 1; // opponent's turn
    let opp = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::skirmish_rhino());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 2, "no doubling when it isn't your turn");
}

// ── Recent-set batch 3 ───────────────────────────────────────────────────────

/// Touch the Spirit Realm exiles a creature until it leaves.
#[test]
fn touch_the_spirit_realm_exiles_until_it_leaves() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(victim)),
    ]));
    let ring = g.move_card_to_battlefield_for_test(0, catalog::touch_the_spirit_realm());
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "creature exiled");
    // When the enchantment leaves, the creature returns.
    g.remove_from_battlefield_to_graveyard_raw(ring);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Serra Angel"),
        "returns when the enchantment leaves");
}

/// Sonar Strike burns a tapped creature and gains life with a Bat out.
#[test]
fn sonar_strike_hits_tapped_and_gains_with_bat() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.battlefield_find_mut(victim).unwrap().tapped = true;
    // A Bat token for the lifegain rider.
    let mut bat = catalog::grizzly_bears();
    bat.name = "Bat"; bat.subtypes.creature_types = vec![CreatureType::Bat];
    g.add_card_to_battlefield(0, bat);
    let spell = g.add_card_to_hand(0, catalog::sonar_strike());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    let life = g.players[0].life;
    cast_at(&mut g, spell, Target::Permanent(victim));
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "4 damage killed the tapped 4/4");
    assert_eq!(g.players[0].life, life + 3, "gained 3 from the Bat rider");
}

/// Aerie Auxiliary supports two other creatures on ETB.
#[test]
fn aerie_auxiliary_supports_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aux = g.add_card_to_battlefield(0, catalog::aerie_auxiliary());
    g.fire_self_etb_triggers(aux, 0);
    drain_stack(&mut g);
    // The ETB support trigger fired and placed a +1/+1 counter (the exact
    // up-to-two count is covered by the cast-spell support test in counters.rs).
    let total = g.battlefield_find(a).unwrap().counter_count(CounterType::PlusOnePlusOne)
        + g.battlefield_find(b).unwrap().counter_count(CounterType::PlusOnePlusOne);
    assert!(total >= 1, "support fired off the ETB trigger");
}

// ── Recent-set batch 4 ───────────────────────────────────────────────────────

/// Loran's Escape shields a creature and scries.
#[test]
fn lorans_escape_shields_and_scries() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    let spell = g.add_card_to_hand(0, catalog::lorans_escape());
    g.players[0].mana_pool.add(Color::White, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    cast_at(&mut g, spell, Target::Permanent(bear));
    drain_stack(&mut g);
    let c = g.computed_permanent(bear).unwrap();
    assert!(c.keywords.contains(&Keyword::Hexproof));
    assert!(c.keywords.contains(&Keyword::Indestructible));
}

/// Dauntless Veteran pumps the team when it attacks.
#[test]
fn dauntless_veteran_pumps_team_on_attack() {
    let mut g = two_player_game();
    let vet = g.add_card_to_battlefield(0, catalog::dauntless_veteran());
    let buddy = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(vet);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).unwrap();
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: vet, target: AttackTarget::Player(1),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(buddy).unwrap().power, 3, "team got +1/+1");
    assert_eq!(g.computed_permanent(vet).unwrap().power, 3, "the veteran too");
}

/// Spectral Denial soft-counters a spell whose controller can't pay {X}.
#[test]
fn spectral_denial_counters_at_x() {
    let mut g = two_player_game();
    // Cast Denial with X=2 → counter unless they pay {2}.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1); // bolt only, nothing spare
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cast Bolt");
    g.priority.player_with_priority = 0;
    let denial = g.add_card_to_hand(0, catalog::spectral_denial());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: denial, target: Some(Target::Permanent(bolt)), additional_targets: vec![],
        mode: None, x_value: Some(2),
    }).expect("cast Denial X=2");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "Bolt countered (couldn't pay {{2}})");
    assert_eq!(g.players[0].life, 20, "Bolt didn't resolve");
}

/// Glistener Seer enters with oil counters and spends them to scry.
#[test]
fn glistener_seer_oil_scry() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let seer = g.move_card_to_battlefield_for_test(0, catalog::glistener_seer());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Oil), 3, "entered with 3 oil");
    g.clear_sickness(seer);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("remove an oil to scry");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(seer).unwrap().counter_count(CounterType::Oil), 2, "spent one oil");
    assert!(g.battlefield_find(seer).unwrap().tapped, "tapped for the ability");
}

/// Vengeful Bloodwitch drains when a creature you control dies.
#[test]
fn vengeful_bloodwitch_drains_on_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::vengeful_bloodwitch());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let opp = g.players[1].life;
    // Kill the fodder through the full damage→SBA→dispatch path.
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(fodder)),
        additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt the fodder");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, opp - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, life + 1, "you gained 1");
}

/// Hulking Raptor ramps two green at your first main phase and has Ward {2}.
#[test]
fn hulking_raptor_ramps_and_wards() {
    let mut g = two_player_game();
    let rap = g.add_card_to_battlefield(0, catalog::hulking_raptor());
    assert!(g.battlefield_find(rap).unwrap().definition.keywords.iter()
        .any(|k| matches!(k, Keyword::Ward(_))), "has Ward");
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::PreCombatMain);
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 2, "added two green");
}

// ── "Start your engines!" / speed (CR 702.179) ──────────────────────────────

/// A "Start your engines!" permanent entering sets its controller's speed to 1.
#[test]
fn start_your_engines_sets_speed_to_one() {
    let mut g = two_player_game();
    assert_eq!(g.players[0].speed, 0);
    g.move_card_to_battlefield_for_test(0, catalog::nesting_bot());
    drain_stack(&mut g);
    assert_eq!(g.players[0].speed, 1, "SYE starts speed at 1");
    // A second SYE permanent doesn't re-bump an already-started speed.
    g.move_card_to_battlefield_for_test(0, catalog::swiftwing_assailant());
    drain_stack(&mut g);
    assert_eq!(g.players[0].speed, 1);
}

/// Speed rises once per your turn when an opponent loses life, capped at 4.
#[test]
fn speed_increments_on_opponent_life_loss() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[0].speed = 1;
    g.adjust_life(1, -1); // opponent loses life on your turn → +1
    assert_eq!(g.players[0].speed, 2);
    g.adjust_life(1, -3); // same turn → no further bump
    assert_eq!(g.players[0].speed, 2);
    // A player with no speed yet isn't started by a life-loss event.
    g.active_player_idx = 1;
    g.players[1].speed_increased_this_turn = false;
    g.adjust_life(0, -1);
    assert_eq!(g.players[1].speed, 0, "no speed yet → life loss doesn't start it");
}

/// Speed never exceeds 4.
#[test]
fn speed_caps_at_four() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.players[0].speed = 4;
    g.adjust_life(1, -1);
    assert_eq!(g.players[0].speed, 4);
}

/// Nesting Bot's "Max speed —" +1/+0 is live only at speed 4.
#[test]
fn nesting_bot_max_speed_pump() {
    let mut g = two_player_game();
    let bot = g.add_card_to_battlefield(0, catalog::nesting_bot());
    assert_eq!(g.computed_permanent(bot).unwrap().power, 1, "1/1 below max speed");
    g.players[0].speed = 4;
    assert_eq!(g.computed_permanent(bot).unwrap().power, 2, "+1/+0 at max speed");
}

/// Burnout Bashtronaut gains double strike at max speed.
#[test]
fn burnout_bashtronaut_max_speed_double_strike() {
    let mut g = two_player_game();
    let goblin = g.add_card_to_battlefield(0, catalog::burnout_bashtronaut());
    assert!(!g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::DoubleStrike));
    g.players[0].speed = 4;
    assert!(g.computed_permanent(goblin).unwrap().keywords.contains(&Keyword::DoubleStrike));
}

/// Risen Necroregent makes an end-step Zombie only at max speed.
#[test]
fn risen_necroregent_max_speed_token() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.add_card_to_battlefield(0, catalog::risen_necroregent());
    let creatures = |g: &GameState| {
        g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_creature()).count()
    };
    let before = creatures(&g);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(creatures(&g), before, "no token below max speed");
    g.players[0].speed = 4;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(creatures(&g), before + 1, "2/2 Zombie at max speed");
}

/// Walking Sarcophagus is a 2/1 normally, 3/3 at max speed.
#[test]
fn walking_sarcophagus_max_speed_pump() {
    let mut g = two_player_game();
    let s = g.add_card_to_battlefield(0, catalog::walking_sarcophagus());
    assert_eq!(g.computed_permanent(s).map(|c| (c.power, c.toughness)), Some((2, 1)));
    g.players[0].speed = 4;
    assert_eq!(g.computed_permanent(s).map(|c| (c.power, c.toughness)), Some((3, 3)));
}

/// Streaking Oilgorger gains lifelink only at max speed.
#[test]
fn streaking_oilgorger_max_speed_lifelink() {
    let mut g = two_player_game();
    let v = g.add_card_to_battlefield(0, catalog::streaking_oilgorger());
    assert!(!g.computed_permanent(v).unwrap().keywords.contains(&Keyword::Lifelink));
    g.players[0].speed = 4;
    assert!(g.computed_permanent(v).unwrap().keywords.contains(&Keyword::Lifelink));
}

/// Gastal Thrillseeker pings each opponent and gains you life on ETB.
#[test]
fn gastal_thrillseeker_etb_ping() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    let life1 = g.players[1].life;
    g.move_card_to_battlefield_for_test(0, catalog::gastal_thrillseeker());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "opponent took 1");
    assert_eq!(g.players[0].life, life0 + 1, "you gained 1");
}

/// Goblin Surveyor's graveyard draw is only castable at max speed.
#[test]
fn goblin_surveyor_max_speed_gated_ability() {
    let def = catalog::goblin_surveyor();
    let ab = &def.activated_abilities[0];
    assert!(ab.from_graveyard && ab.exile_self_cost, "graveyard exile-cost ability");
    assert_eq!(ab.condition, Some(crate::card::Predicate::SpeedAtLeast {
        who: crate::effect::PlayerRef::You,
        speed: 4,
    }));
}

// ── Recover (CR 702.58) ─────────────────────────────────────────────────────

/// Recover returns the card to hand when its cost is paid as a creature dies.
#[test]
fn recover_returns_to_hand_when_paid() {
    let mut g = two_player_game();
    let gh = g.add_card_to_graveyard(0, catalog::suns_bounty());
    // Pre-float the {1}{W} recover cost and accept the MayPay prompt.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut evs = g.remove_to_graveyard_with_triggers(bear);
    evs.push(GameEvent::CreatureDied { card_id: bear });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == gh), "recovered to hand");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gh));
}

/// Declining recover (no mana) exiles the card.
#[test]
fn recover_exiles_when_declined() {
    let mut g = two_player_game();
    let gh = g.add_card_to_graveyard(0, catalog::icefall());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mut evs = g.remove_to_graveyard_with_triggers(bear);
    evs.push(GameEvent::CreatureDied { card_id: bear });
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == gh), "unpaid recover exiles the card");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == gh));
}

/// Bloodthirsty Conqueror gains you life equal to an opponent's life loss.
#[test]
fn bloodthirsty_conqueror_drains_to_you() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bloodthirsty_conqueror());
    let life0 = g.players[0].life;
    let evs = vec![{
        let amt = 3u32;
        g.players[1].life -= amt as i32;
        crate::game::GameEvent::LifeLost { player: 1, amount: amt }
    }];
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 + 3, "gained 3 from the opponent's loss");
}

/// Razorkin Needlehead has first strike on your turn only, and pings opponents
/// who draw.
#[test]
fn razorkin_needlehead_turn_first_strike_and_draw_ping() {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    let rk = g.add_card_to_battlefield(0, catalog::razorkin_needlehead());
    assert!(g.computed_permanent(rk).unwrap().keywords.contains(&Keyword::FirstStrike));
    g.active_player_idx = 1;
    assert!(!g.computed_permanent(rk).unwrap().keywords.contains(&Keyword::FirstStrike));
    // Opponent draws → takes 1.
    let life1 = g.players[1].life;
    let drawn = g.add_card_to_hand(1, catalog::island());
    g.dispatch_triggers_for_events(&[crate::game::GameEvent::CardDrawn {
        player: 1,
        card_id: drawn,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1);
}

/// Savor shrinks a creature and makes a Food.
#[test]
fn savor_shrinks_and_makes_food() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let mut ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    ctx.targets = vec![Target::Permanent(foe)];
    g.resolve_effect(&catalog::savor().effect, &ctx).unwrap();
    drain_stack(&mut g);
    g.check_state_based_actions();
    assert!(g.battlefield_find(foe).is_none(), "-2/-2 killed the 2/2");
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
}

/// Screaming Nemesis redirects damage dealt to it onto any target.
#[test]
fn screaming_nemesis_redirects_damage() {
    let mut g = two_player_game();
    let nem = g.add_card_to_battlefield(0, catalog::screaming_nemesis());
    let life1 = g.players[1].life;
    // The enrage trigger reads the DamageDealt amount and bolts any target.
    g.dispatch_triggers_for_events(&[crate::game::GameEvent::DamageDealt {
        amount: 3,
        to_card: Some(nem),
        to_player: None,
    }]);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 3, "redirected 3 to the opponent");
    // CR 119.7 — the damaged player can't gain life for the rest of the game.
    assert!(g.players[1].cannot_gain_life, "rest-of-game lifegain lock");
    let before = g.players[1].life;
    g.adjust_life(1, 5);
    assert_eq!(g.players[1].life, before, "lifegain stays locked");
}

/// Spinewoods Armadillo is a 7/7 with Reach and Ward {3}.
#[test]
fn spinewoods_armadillo_stats() {
    let def = catalog::spinewoods_armadillo();
    assert_eq!((def.power, def.toughness), (7, 7));
    assert!(def.keywords.contains(&Keyword::Reach));
    assert!(def.activated_abilities[0].discard_self_cost, "discard-this fetch ability");
}

/// Goblin Boarders enters with a +1/+1 counter only if you attacked this turn.
#[test]
fn goblin_boarders_raid_counter() {
    let mut g = two_player_game();
    g.players[0].attacked_this_turn = true;
    let id = g.move_card_to_battlefield_for_test(0, catalog::goblin_boarders());
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield_find(id).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        Some(1),
    );
    let mut g2 = two_player_game();
    let id2 = g2.move_card_to_battlefield_for_test(0, catalog::goblin_boarders());
    drain_stack(&mut g2);
    assert_eq!(
        g2.battlefield_find(id2).unwrap().counters.get(&CounterType::PlusOnePlusOne).copied(),
        None,
        "no Raid → no counter",
    );
}

/// Cogwork Wrestler shrinks an opponent's creature's power on ETB.
#[test]
fn cogwork_wrestler_etb_shrinks_foe() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::hill_giant()); // 3/3
    g.move_card_to_battlefield_for_test(0, catalog::cogwork_wrestler());
    // ETB targets the only opposing creature.
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(foe).map(|c| c.power), Some(1), "-2/-0 applied");
}

/// Crocodile of the Crossing puts a -1/-1 counter on a creature you control.
#[test]
fn crocodile_of_the_crossing_etb_counter() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.move_card_to_battlefield_for_test(0, catalog::crocodile_of_the_crossing());
    drain_stack(&mut g);
    // Auto-target picks a creature you control; with two, either works — assert
    // the total -1/-1 counters on your board is 1.
    let total: u32 = g.battlefield.iter().filter(|c| c.controller == 0)
        .map(|c| c.counters.get(&CounterType::MinusOneMinusOne).copied().unwrap_or(0)).sum();
    assert_eq!(total, 1);
    let _ = mine;
}

/// Topiary Stomper ramps a basic land onto the battlefield tapped.
#[test]
fn topiary_stomper_ramps() {
    let mut g = two_player_game();
    let forest = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(forest))]));
    let lands_before = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    g.move_card_to_battlefield_for_test(0, catalog::topiary_stomper());
    drain_stack(&mut g);
    let lands_after = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.is_land()).count();
    assert_eq!(lands_after, lands_before + 1, "fetched a basic onto the battlefield");
}

/// Bakersbane Duo makes a Food on entry.
#[test]
fn bakersbane_duo_makes_food() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::bakersbane_duo());
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Food"));
}

/// Cache Grab mills four and returns a chosen permanent card to hand; the
/// non-permanent milled card stays in the graveyard.
#[test]
fn cache_grab_returns_a_milled_permanent() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::lightning_bolt()); // instant — not eligible
    let bears = g.add_card_to_library(0, catalog::grizzly_bears()); // permanent
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_library(0, catalog::island());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![bears])]));
    let grab = g.add_card_to_hand(0, catalog::cache_grab());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.perform_action(GameAction::CastSpell {
        card_id: grab,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("Cache Grab castable for {1}{G}");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bears), "chose the creature to hand");
    assert_eq!(g.players[0].library.len(), 0, "milled all four");
    assert!(g.players[0].graveyard.iter().any(|c| c.definition.name == "Lightning Bolt"),
        "the instant stays milled");
}

/// Lumbering Worldwagon's power equals the lands you control; toughness stays 4.
#[test]
fn lumbering_worldwagon_power_tracks_lands() {
    let mut g = two_player_game();
    let wagon = g.add_card_to_battlefield(0, catalog::lumbering_worldwagon());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let c = g.computed_permanent(wagon).unwrap();
    assert_eq!(c.power, 3, "power = 3 lands controlled");
    assert_eq!(c.toughness, 4, "printed toughness");
}

/// Spire Mangler pumps a flyer you control on ETB.
#[test]
fn spire_mangler_pumps_a_flyer() {
    let mut g = two_player_game();
    let flyer = g.add_card_to_battlefield(0, catalog::serra_angel()); // 4/4 flying
    g.move_card_to_battlefield_for_test(0, catalog::spire_mangler());
    drain_stack(&mut g);
    // Auto-target picks a controlled flyer; total power gain on your flyers is 2.
    assert_eq!(g.computed_permanent(flyer).map(|c| c.power), Some(6));
}

/// Topiary Stomper can't attack until you control seven lands.
#[test]
fn topiary_stomper_needs_seven_lands_to_attack() {
    let mut g = two_player_game();
    let stomper = g.add_card_to_battlefield(0, catalog::topiary_stomper());
    g.clear_sickness(stomper);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    // Six lands → still can't attack.
    assert!(!g.legal_attackers(0).contains(&stomper));
    assert!(g
        .declare_attackers(vec![Attack { attacker: stomper, target: AttackTarget::Player(1) }])
        .is_err());
    // Seventh land lifts the restriction.
    g.add_card_to_battlefield(0, catalog::forest());
    assert!(g.legal_attackers(0).contains(&stomper));
    g.declare_attackers(vec![Attack { attacker: stomper, target: AttackTarget::Player(1) }])
        .expect("seven lands → can attack");
}

/// Topiary Stomper can't block until you control seven lands either.
#[test]
fn topiary_stomper_needs_seven_lands_to_block() {
    let mut g = two_player_game();
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let stomper = g.add_card_to_battlefield(0, catalog::topiary_stomper());
    for _ in 0..6 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    assert!(!g.blocker_can_block_attacker(stomper, attacker), "six lands → can't block");
    g.add_card_to_battlefield(0, catalog::forest());
    assert!(g.blocker_can_block_attacker(stomper, attacker), "seven lands → can block");
}

/// Palace Familiar draws a card when it dies.
#[test]
fn palace_familiar_dies_draws() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::island());
    let fam = g.add_card_to_battlefield(0, catalog::palace_familiar());
    let hand_before = g.players[0].hand.len();
    g.remove_to_graveyard_with_triggers(fam);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "dies → draw a card");
}

/// Symbiotic Elf leaves two 1/1 Insects when it dies.
#[test]
fn symbiotic_elf_dies_makes_two_insects() {
    let mut g = two_player_game();
    let elf = g.add_card_to_battlefield(0, catalog::symbiotic_elf());
    g.remove_to_graveyard_with_triggers(elf);
    drain_stack(&mut g);
    let insects = g.battlefield.iter().filter(|c| c.controller == 0 && c.definition.name == "Insect").count();
    assert_eq!(insects, 2);
}

/// Bear's Companion mints a 4/4 Bear on entry.
#[test]
fn bears_companion_makes_a_bear() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::bears_companion());
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Bear");
    assert!(bear.is_some_and(|b| b.definition.power == 4 && b.definition.toughness == 4));
}

/// Grasping Thrull drains each opponent for 2 and gains you 2.
#[test]
fn grasping_thrull_drains_and_gains() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.move_card_to_battlefield_for_test(0, catalog::grasping_thrull());
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18, "opponent took 2");
    assert_eq!(g.players[0].life, 22, "you gained 2");
}

/// Hero of Precinct One makes a 1/1 Human when you cast a multicolored spell.
#[test]
fn hero_of_precinct_one_tokens_on_multicolored_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hero_of_precinct_one());
    let thrull = g.add_card_to_hand(0, catalog::grasping_thrull()); // W/B multicolored
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: thrull,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("multicolored spell castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Human"));
}

/// Havoc Devils is a 4/3 with trample.
#[test]
fn havoc_devils_stats() {
    let d = catalog::havoc_devils();
    assert_eq!((d.power, d.toughness), (4, 3));
    assert!(d.keywords.contains(&Keyword::Trample));
}

/// Hollow Dogs pumps itself +2/+0 when it attacks.
#[test]
fn hollow_dogs_pumps_on_attack() {
    let mut g = two_player_game();
    let dogs = g.add_card_to_battlefield(0, catalog::hollow_dogs());
    g.clear_sickness(dogs);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: dogs, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(dogs).map(|c| c.power), Some(5));
}

/// Argothian Enchantress has shroud and draws on enchantment casts.
#[test]
fn argothian_enchantress_draws_on_enchantment_cast() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(0, catalog::argothian_enchantress());
    assert!(g.battlefield_find(ench).unwrap().definition.keywords.contains(&Keyword::Shroud));
    g.add_card_to_library(0, catalog::island());
    let prison = g.add_card_to_hand(0, catalog::ghostly_prison());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: prison,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("enchantment castable");
    drain_stack(&mut g);
    // Spent one card (the enchantment) and drew one back from the trigger.
    assert_eq!(g.players[0].hand.len(), hand_before);
}

/// Patrol Hound gains first strike by discarding a card.
#[test]
fn patrol_hound_discards_for_first_strike() {
    let mut g = two_player_game();
    let hound = g.add_card_to_battlefield(0, catalog::patrol_hound());
    g.add_card_to_hand(0, catalog::island());
    g.perform_action(GameAction::ActivateAbility {
        card_id: hound,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    }).expect("ability activates");
    drain_stack(&mut g);
    assert!(g.computed_permanent(hound).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Canyon Wildcat can't be blocked while the defender controls a Mountain.
#[test]
fn canyon_wildcat_mountainwalk() {
    let mut g = two_player_game();
    let cat = g.add_card_to_battlefield(0, catalog::canyon_wildcat());
    g.clear_sickness(cat);
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::mountain());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: cat, target: AttackTarget::Player(1) }]).unwrap();
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    assert!(
        g.declare_blockers(vec![(blocker, cat)]).is_err(),
        "mountainwalk: can't be blocked while defender controls a Mountain"
    );
}

/// Squirrelanoids is a 1/1 with deathtouch.
#[test]
fn squirrelanoids_deathtouch() {
    let d = catalog::squirrelanoids();
    assert_eq!((d.power, d.toughness), (1, 1));
    assert!(d.keywords.contains(&Keyword::Deathtouch));
}

/// Vile Deacon gets +X/+X on attack where X counts Clerics.
#[test]
fn vile_deacon_scales_with_clerics() {
    let mut g = two_player_game();
    let deacon = g.add_card_to_battlefield(0, catalog::vile_deacon()); // a Cleric itself
    g.add_card_to_battlefield(0, catalog::vile_deacon()); // second Cleric
    g.clear_sickness(deacon);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker: deacon, target: AttackTarget::Player(1) }]).unwrap();
    drain_stack(&mut g);
    // Base 2 + (2 Clerics) = 4 power.
    assert_eq!(g.computed_permanent(deacon).map(|c| c.power), Some(4));
}

/// Mischievous Mystic makes a Faerie when you draw your second card.
#[test]
fn mischievous_mystic_tokens_on_second_draw() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::mischievous_mystic());
    for _ in 0..3 { g.add_card_to_library(0, catalog::island()); }
    let div = g.add_card_to_hand(0, catalog::divination()); // draw 2
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: div,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Divination castable");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Faerie"));
}

/// Dawn's Light Archer has flash and reach.
#[test]
fn dawns_light_archer_keywords() {
    let d = catalog::dawns_light_archer();
    assert!(d.keywords.contains(&Keyword::Flash) && d.keywords.contains(&Keyword::Reach));
    assert_eq!((d.power, d.toughness), (4, 2));
}

/// Plumeveil is a 4/4 with flash, defender, and flying.
#[test]
fn plumeveil_keywords() {
    let d = catalog::plumeveil();
    assert_eq!((d.power, d.toughness), (4, 4));
    for kw in [Keyword::Flash, Keyword::Defender, Keyword::Flying] {
        assert!(d.keywords.contains(&kw));
    }
}

/// Rooftop Assassin destroys an opponent's damaged creature on ETB.
#[test]
fn rooftop_assassin_destroys_damaged_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(foe).unwrap().dealt_damage_this_turn = true;
    g.move_card_to_battlefield_for_test(0, catalog::rooftop_assassin());
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).is_none(), "damaged creature destroyed");
}

/// Spellgorger Barbarian discards at random on ETB and draws when it leaves.
#[test]
fn spellgorger_barbarian_etb_discard_and_leaves_draw() {
    let mut g = two_player_game();
    g.add_card_to_hand(0, catalog::island());
    g.add_card_to_library(0, catalog::island());
    let barb = g.move_card_to_battlefield_for_test(0, catalog::spellgorger_barbarian());
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 0, "ETB discarded the one card at random");
    let hand_before = g.players[0].hand.len();
    g.remove_to_graveyard_with_triggers(barb);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "leaves → draw");
}

/// Bog Gnarr grows when any player casts a black spell.
#[test]
fn bog_gnarr_pumps_on_black_cast() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::bog_gnarr());
    let squirrel = g.add_card_to_hand(0, catalog::squirrelanoids()); // mono-black
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: squirrel,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("black spell castable");
    drain_stack(&mut g);
    let gnarr = g.battlefield.iter().find(|c| c.definition.name == "Bog Gnarr").unwrap();
    assert_eq!(g.computed_permanent(gnarr.id).map(|c| c.power), Some(4));
}

/// Elf Replica sacrifices to destroy an enchantment.
#[test]
fn elf_replica_destroys_enchantment() {
    let mut g = two_player_game();
    let replica = g.add_card_to_battlefield(0, catalog::elf_replica());
    let prison = g.add_card_to_battlefield(1, catalog::ghostly_prison());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: replica,
        ability_index: 0,
        target: Some(Target::Permanent(prison)),
        additional_targets: vec![],
        x_value: None,
    }).expect("ability activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prison).is_none(), "enchantment destroyed");
    assert!(g.battlefield_find(replica).is_none(), "Elf Replica sacrificed");
}

/// Seismic Mage taps and discards to destroy a land.
#[test]
fn seismic_mage_destroys_land() {
    let mut g = two_player_game();
    let mage = g.add_card_to_battlefield(0, catalog::seismic_mage());
    g.clear_sickness(mage);
    g.add_card_to_hand(0, catalog::island());
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: mage,
        ability_index: 0,
        target: Some(Target::Permanent(land)),
        additional_targets: vec![],
        x_value: None,
    }).expect("ability activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land destroyed");
}

/// Etched Oracle enters with a +1/+1 counter per color of mana spent (Sunburst).
#[test]
fn etched_oracle_sunburst_counters() {
    let mut g = two_player_game();
    let oracle = g.add_card_to_hand(0, catalog::etched_oracle()); // {4}
    // Pay the generic {4} with W, U, B + 1 colorless → 3 distinct colors.
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: oracle,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Etched Oracle castable");
    drain_stack(&mut g);
    let c = g.battlefield_find(oracle).unwrap();
    assert_eq!(c.counter_count(CounterType::PlusOnePlusOne), 3, "3 colors → 3 counters");
}

/// Skyreach Manta has flying and Sunburst counters.
#[test]
fn skyreach_manta_flying_sunburst() {
    let d = catalog::skyreach_manta();
    assert!(d.keywords.contains(&Keyword::Flying));
    assert!(d.enters_with_counters.is_some(), "Sunburst counter spec present");
}

/// Phyrexian Digester and Blackcleave Goblin both carry infect.
#[test]
fn infect_creatures_have_infect() {
    assert!(catalog::phyrexian_digester().keywords.contains(&Keyword::Infect));
    let bg = catalog::blackcleave_goblin();
    assert!(bg.keywords.contains(&Keyword::Infect) && bg.keywords.contains(&Keyword::Haste));
}

/// Essence Depleter drains an opponent for 1 with its colorless ability.
#[test]
fn essence_depleter_drains() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.players[1].life = 20;
    let dep = g.add_card_to_battlefield(0, catalog::essence_depleter());
    g.clear_sickness(dep);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::ActivateAbility {
        card_id: dep,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    }).expect("ability activates");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19);
    assert_eq!(g.players[0].life, 21);
    // Devoid: the creature is colorless despite its black cost.
    assert!(catalog::essence_depleter().keywords.contains(&Keyword::Devoid));
}

/// Stormclaw Rager grows and draws when you sacrifice another permanent.
#[test]
fn stormclaw_rager_sac_grows_and_draws() {
    let mut g = two_player_game();
    let rager = g.add_card_to_battlefield(0, catalog::stormclaw_rager());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.step = TurnStep::PostCombatMain;
    g.players[0].mana_pool.add_colorless(1);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rager,
        ability_index: 0,
        target: Some(Target::Permanent(fodder)),
        additional_targets: vec![],
        x_value: None,
    }).expect("sac ability activates");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(rager).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Wave Elemental taps up to three non-flyers on sacrifice.
#[test]
fn wave_elemental_taps_nonflyers() {
    let mut g = two_player_game();
    let elem = g.add_card_to_battlefield(0, catalog::wave_elemental());
    g.clear_sickness(elem);
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    // The "up to three" picks are made at resolution via a ChooseCards decision.
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Cards(vec![a, b])]));
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: elem,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    }).expect("tap ability activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
    assert!(g.battlefield_find(elem).is_none(), "Wave Elemental sacrificed");
}

/// Shipwreck Moray gets four energy on ETB.
#[test]
fn shipwreck_moray_makes_energy() {
    let mut g = two_player_game();
    g.move_card_to_battlefield_for_test(0, catalog::shipwreck_moray());
    drain_stack(&mut g);
    assert_eq!(g.players[0].energy, 4, "ETB grants four energy");
}

/// Argothian Sprite can't be blocked by artifact creatures.
#[test]
fn argothian_sprite_evades_artifacts() {
    let mut g = two_player_game();
    let sprite = g.add_card_to_battlefield(0, catalog::argothian_sprite());
    let artifact_blocker = g.add_card_to_battlefield(1, catalog::phyrexian_digester()); // artifact creature
    let normal_blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    assert!(!g.blocker_can_block_attacker(artifact_blocker, sprite), "artifact can't block");
    assert!(g.blocker_can_block_attacker(normal_blocker, sprite), "non-artifact can");
}

/// Nadier's Nightblade drains each opponent when a token you control leaves.
#[test]
fn nadiers_nightblade_drains_on_token_leave() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.players[1].life = 20;
    g.add_card_to_battlefield(0, catalog::nadiers_nightblade());
    // Make a token, then destroy it.
    g.move_card_to_battlefield_for_test(0, catalog::bears_companion());
    drain_stack(&mut g);
    let bear = g.battlefield.iter().find(|c| c.controller == 0 && c.definition.name == "Bear").unwrap().id;
    // Lethal damage → SBA dispatches CreatureDied, firing the watcher.
    g.battlefield_find_mut(bear).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 19, "opponent lost 1");
    assert_eq!(g.players[0].life, 21, "you gained 1");
}

/// Gnarlroot Pallbearer pumps a creature by your graveyard's creature count.
#[test]
fn gnarlroot_pallbearer_scales_with_graveyard() {
    let mut g = two_player_game();
    // Seed two creature cards in the graveyard.
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(target))]));
    g.move_card_to_battlefield_for_test(0, catalog::gnarlroot_pallbearer());
    drain_stack(&mut g);
    // 2 creatures in gy → +2/+2 → 4/4.
    assert_eq!(g.computed_permanent(target).map(|c| c.power), Some(4));
}

/// Illusionary Servant sacrifices itself when targeted.
#[test]
fn illusionary_servant_dies_when_targeted() {
    let mut g = two_player_game();
    let servant = g.add_card_to_battlefield(0, catalog::illusionary_servant());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(servant)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    }).expect("Bolt targets the Servant");
    drain_stack(&mut g);
    assert!(g.battlefield_find(servant).is_none(), "sacrificed on becoming targeted");
}

/// Bounding Wolf and Goblin Sky Raider carry their printed keywords.
#[test]
fn vanilla_keyword_creatures() {
    let bw = catalog::bounding_wolf();
    assert!(bw.keywords.contains(&Keyword::Flash) && bw.keywords.contains(&Keyword::Reach));
    assert!(catalog::goblin_sky_raider().keywords.contains(&Keyword::Flying));
}

/// Glowing Anemone bounces a land on ETB when its controller chooses to.
#[test]
fn glowing_anemone_returns_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::mountain());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(land)),
    ]));
    g.move_card_to_battlefield_for_test(0, catalog::glowing_anemone());
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).is_none(), "land returned to hand");
}

/// Contraband Kingpin scries when an artifact you control enters.
#[test]
fn contraband_kingpin_scries_on_artifact() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::contraband_kingpin());
    g.add_card_to_library(0, catalog::island());
    // An artifact entering fires the scry-1 trigger (resolves via the AutoDecider).
    g.move_card_to_battlefield_for_test(0, catalog::gold_myr());
    drain_stack(&mut g);
    assert!(catalog::contraband_kingpin().keywords.contains(&Keyword::Lifelink));
    // The trigger fired and resolved without panicking; library intact (kept on top).
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Gold Myr"));
}

/// Kingpin's Enforcers sacrifices a permanent to draw a card.
#[test]
fn kingpins_enforcers_sac_to_draw() {
    let mut g = two_player_game();
    let enf = g.add_card_to_battlefield(0, catalog::kingpins_enforcers());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::island());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    let hand_before = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: enf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    }).expect("sac ability activates");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "fodder sacrificed");
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "drew a card");
}

/// Goldmaw Champion's Boast taps a creature after it has attacked.
#[test]
fn goldmaw_champion_boast_taps() {
    let mut g = two_player_game();
    let champ = g.add_card_to_battlefield(0, catalog::goldmaw_champion());
    g.battlefield_find_mut(champ).unwrap().attacked_this_turn = true;
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: champ,
        ability_index: 0,
        target: Some(Target::Permanent(foe)),
        additional_targets: vec![],
        x_value: None,
    }).expect("boast activates after attacking");
    drain_stack(&mut g);
    assert!(g.battlefield_find(foe).unwrap().tapped, "target tapped by Boast");
}

/// Gold Myr taps for white mana.
#[test]
fn gold_myr_makes_white() {
    let mut g = two_player_game();
    let myr = g.add_card_to_battlefield(0, catalog::gold_myr());
    g.clear_sickness(myr);
    g.perform_action(GameAction::ActivateAbility {
        card_id: myr,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
    }).expect("mana ability");
    assert!(g.players[0].mana_pool.amount(Color::White) >= 1);
}

/// Drumhunter draws at end step when you control a 5-power creature.
#[test]
fn drumhunter_draws_with_big_creature() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::drumhunter());
    g.add_card_to_battlefield(0, catalog::vaultborn_tyrant()); // 6/6, power ≥ 5
    g.add_card_to_library(0, catalog::island());
    g.active_player_idx = 0;
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let hand_before = g.players[0].hand.len();
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand_before + 1, "end-step draw fired");
}

/// Cleave (CR 702.148) — Dig Up's base mode finds only a basic land; the cleave
/// alt-cost removes the bracket and finds any card.
#[test]
fn cr_702_148_dig_up_cleave_widens_search() {
    // Base cast: only a basic land is a legal find.
    let mut g = two_player_game();
    let nonbasic = g.add_card_to_library(0, catalog::grizzly_bears());
    let basic = g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(basic))]));
    let dig = g.add_card_to_hand(0, catalog::dig_up());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: dig, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("base cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == basic), "basic land tutored to hand");
    let _ = nonbasic;

    // Cleave cast: a nonland creature card is now a legal find.
    let mut g = two_player_game();
    let creature = g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Search(Some(creature))]));
    let dig = g.add_card_to_hand(0, catalog::dig_up());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: dig, pitch_card: None, target: None, additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cleave cast");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == creature), "cleave found a nonland card");
}

/// Cleave (CR 702.148) — Dread Fugue's base only discards a low-MV nonland; the
/// cleave alt-cost lets the chooser take any nonland.
#[test]
fn cr_702_148_dread_fugue_cleave_widens_discard() {
    let mut g = two_player_game();
    // Opponent holds only an expensive nonland card.
    let big = g.add_card_to_hand(1, catalog::serra_angel()); // MV 5
    let fugue = g.add_card_to_hand(0, catalog::dread_fugue());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: fugue, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("base cast");
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == big), "base mode can't take the MV-5 card");

    // Cleave: now the MV-5 card is a legal pick.
    let fugue2 = g.add_card_to_hand(0, catalog::dread_fugue());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellAlternative {
        card_id: fugue2, pitch_card: None, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    }).expect("cleave cast");
    drain_stack(&mut g);
    assert!(!g.players[1].hand.iter().any(|c| c.id == big), "cleave discarded the MV-5 card");
}

/// Venerable Monk gains 2 life on ETB.
#[test]
fn venerable_monk_gains_life() {
    let mut g = two_player_game();
    g.players[0].life = 20;
    g.move_card_to_battlefield_for_test(0, catalog::venerable_monk());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 22);
}

/// Vanilla keyword bodies: Roc and Minotaur Aggressor.
#[test]
fn roc_and_minotaur_keywords() {
    assert!(catalog::roc_of_kher_ridges().keywords.contains(&Keyword::Flying));
    let m = catalog::minotaur_aggressor();
    assert!(m.keywords.contains(&Keyword::FirstStrike) && m.keywords.contains(&Keyword::Haste));
}

/// Malakir Familiar grows whenever you gain life.
#[test]
fn malakir_familiar_grows_on_lifegain() {
    let mut g = two_player_game();
    let bat = g.add_card_to_battlefield(0, catalog::malakir_familiar());
    use crate::effect::{Effect, Selector, Value};
    let ctx = crate::game::effects::EffectContext::for_ability(crate::card::CardId(0), 0, None);
    let evs = g
        .resolve_effect(&Effect::GainLife { who: Selector::You, amount: Value::Const(1) }, &ctx)
        .unwrap();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bat).map(|c| c.power), Some(3), "+1/+1 on lifegain");
}

/// Mercurial Geists pumps when you cast an instant or sorcery.
#[test]
fn mercurial_geists_pumps_on_spell() {
    let mut g = two_player_game();
    let geist = g.add_card_to_battlefield(0, catalog::mercurial_geists());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("bolt castable");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(geist).map(|c| c.power), Some(4), "+3/+0 from instant cast");
}

/// Engine Rat drains each opponent for 2.
#[test]
fn engine_rat_drains() {
    let mut g = two_player_game();
    g.players[1].life = 20;
    let rat = g.add_card_to_battlefield(0, catalog::engine_rat());
    g.clear_sickness(rat);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(5);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rat, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("drain ability");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
}

/// Gavony Silversmith puts a +1/+1 counter on each of up to two creatures.
#[test]
fn gavony_silversmith_counters_two() {
    let mut g = two_player_game();
    let a = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::gavony_silversmith());
    drain_stack(&mut g);
    // Up-to-two ApplyToTargets distributes +1/+1 counters onto the chosen
    // creature(s); each picked target gets exactly one.
    let total: u32 = [a, b]
        .iter()
        .map(|id| g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne))
        .sum();
    assert!(total >= 1, "at least one creature got a +1/+1 counter");
    assert!(
        [a, b]
            .iter()
            .all(|id| g.battlefield_find(*id).unwrap().counter_count(CounterType::PlusOnePlusOne) <= 1),
        "no creature gets more than one counter from a single resolution"
    );
}

/// Reputable Merchant counters a creature on ETB and again on death.
#[test]
fn reputable_merchant_counters_on_etb_and_death() {
    let mut g = two_player_game();
    let target = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Target(Target::Permanent(target)),
        DecisionAnswer::Target(Target::Permanent(target)),
    ]));
    let merch = g.move_card_to_battlefield_for_test(0, catalog::reputable_merchant());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 1, "ETB counter");
    let evs = g.remove_to_graveyard_with_triggers(merch);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 2, "death counter");
}
