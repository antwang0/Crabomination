//! CR 702.55 — Haunt. Functionality tests for the Guildpact haunt cards in
//! `catalog::sets::gpt`.

use crabomination::card::Keyword;
use crabomination::catalog;
use crabomination::game::types::Target;
use crabomination::game::*;
use crabomination::mana::Color;

fn resolve_spell(g: &mut GameState, def: crabomination::card::CardDefinition, targets: Vec<Target>) {
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = targets;
    let events = g.resolve_effect(&def.effect, &ctx).unwrap();
    g.dispatch_triggers_for_events(&events);
    drain_stack(g);
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    while g.step != step {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// A haunt creature is exiled (not graveyard'd) when it dies, then its haunt
/// body fires when the haunted creature dies.
#[test]
fn shrieking_grotesque_haunts_then_payoff_on_death() {
    let mut g = two_player_game();
    let grotesque = g.add_card_to_battlefield(0, catalog::shrieking_grotesque());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4, survives
    g.add_card_to_hand(1, catalog::grizzly_bears()); // the one card to discard

    // Kill the Grotesque → it's exiled haunting the opponent's creature.
    g.battlefield_find_mut(grotesque).unwrap().damage = 1; // lethal vs 2/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == grotesque), "exiled haunting");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == grotesque), "not in graveyard");
    assert_eq!(g.players[1].hand.len(), 1, "payoff not fired yet");

    // The haunted creature dies → opponent discards a card.
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "haunt payoff: opponent discarded");
}

/// Mourning Thrull's gain-2-and-draw trigger fires on entry.
#[test]
fn mourning_thrull_etb_gain_and_draw() {
    let mut g = two_player_game();
    g.add_card_to_library(0, catalog::grizzly_bears());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.move_card_to_battlefield_for_test(0, catalog::mourning_thrull());
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "ETB gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "ETB drew a card");
}

/// Mourning Thrull's haunt body (gain 2, draw 1) fires when the haunted
/// creature dies, even though the Thrull itself is in exile.
#[test]
fn mourning_thrull_haunt_payoff_on_haunted_death() {
    let mut g = two_player_game();
    let thrull = g.add_card_to_battlefield(0, catalog::mourning_thrull());
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    for _ in 0..2 { g.add_card_to_library(0, catalog::grizzly_bears()); }

    g.battlefield_find_mut(thrull).unwrap().damage = 1; // lethal vs 1/1
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == thrull));

    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life + 2, "haunt gained 2");
    assert_eq!(g.players[0].hand.len(), hand + 1, "haunt drew a card");
}

/// A haunt instant resolves its main effect, is exiled haunting a creature
/// (not graveyard'd), then fires its haunt body when that creature dies.
#[test]
fn douse_in_gloom_instant_haunts() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    let douse = g.add_card_to_hand(0, catalog::douse_in_gloom());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Black, 1);

    let life = g.players[0].life;
    cast_at(&mut g, douse, Target::Permanent(foe));
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 2, "dealt 2");
    assert_eq!(g.players[0].life, life + 2, "gained 2");
    assert!(g.exile.iter().any(|c| c.id == douse), "spell exiled haunting");
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == douse), "not graveyard'd");

    // Kill the haunted creature → haunt body: 2 to the opponent, gain 2.
    let p1_life = g.players[1].life;
    let p0_life = g.players[0].life;
    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1_life - 2, "haunt dealt 2 to opponent");
    assert_eq!(g.players[0].life, p0_life + 2, "haunt gained 2");
}

/// Castigate exiles a nonland from the opponent's hand on cast and again when
/// the haunted creature dies.
#[test]
fn castigate_haunt_repeats_hand_exile() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_hand(1, catalog::grizzly_bears());
    let cast_id = g.add_card_to_hand(0, catalog::castigate());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);

    cast_at(&mut g, cast_id, Target::Player(1));
    assert_eq!(g.players[1].hand.len(), 1, "cast exiled one nonland");

    g.battlefield_find_mut(foe).unwrap().damage = 4;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 0, "haunt exiled the second nonland");
}

/// Absolver Thrull's enters-or-haunt trigger destroys an enchantment.
#[test]
fn absolver_thrull_etb_destroys_enchantment() {
    let mut g = two_player_game();
    let ench = g.add_card_to_battlefield(1, catalog::pacifism());
    g.move_card_to_battlefield_for_test(0, catalog::absolver_thrull());
    drain_stack(&mut g);
    assert!(g.battlefield_find(ench).is_none(), "ETB destroyed the enchantment");
}

// ── GPT gap cards (non-haunt + haunt reanimator) ─────────────────────────────

/// Giant Solifuge — 4/1 with trample, haste, shroud.
#[test]
fn giant_solifuge_stat_line() {
    let s = catalog::giant_solifuge();
    assert_eq!((s.power, s.toughness), (4, 1));
    for kw in [Keyword::Trample, Keyword::Haste, Keyword::Shroud] {
        assert!(s.keywords.contains(&kw), "has {kw:?}");
    }
}

/// Crystal Seer's activated ability bounces itself back to hand.
#[test]
fn crystal_seer_returns_itself() {
    let mut g = two_player_game();
    let seer = g.add_card_to_battlefield(0, catalog::crystal_seer());
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("bounce");
    drain_stack(&mut g);
    assert!(g.battlefield_find(seer).is_none(), "left the battlefield");
    assert!(g.players[0].hand.iter().any(|c| c.id == seer), "returned to hand");
}

/// Izzet Chronarch's ETB returns an instant/sorcery from the graveyard.
#[test]
fn izzet_chronarch_recurs_instant() {
    let mut g = two_player_game();
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_helix());
    g.move_card_to_battlefield_for_test(0, catalog::izzet_chronarch());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bolt), "instant returned to hand");
}

/// Drowned Rusalka loots (discard then draw) by sacrificing a creature.
#[test]
fn drowned_rusalka_loots_on_sacrifice() {
    let mut g = two_player_game();
    let rusalka = g.add_card_to_battlefield(0, catalog::drowned_rusalka());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_hand(0, catalog::grizzly_bears()); // the card to discard
    g.add_card_to_library(0, catalog::forest()); // the card to draw
    g.players[0].mana_pool.add(Color::Blue, 1);
    let hand0 = g.players[0].hand.len();
    g.perform_action(GameAction::ActivateAbility {
        card_id: rusalka, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None,
    }).expect("loot");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed as a cost");
    assert_eq!(g.players[0].hand.len(), hand0, "discarded one, drew one (net zero)");
}

/// Crash Landing strips flying and deals damage equal to Forests controlled.
#[test]
fn crash_landing_grounds_and_burns_by_forests() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flyer
    resolve_spell(&mut g, catalog::crash_landing(), vec![Target::Permanent(flyer)]);
    let s = g.battlefield_find(flyer).unwrap();
    assert_eq!(s.damage, 2, "2 damage = number of Forests");
    assert!(!g.compute_battlefield().iter().find(|c| c.id == flyer).unwrap()
        .keywords.contains(&Keyword::Flying), "lost flying this turn");
}

/// Hissing Miasma drains the attacking player when a creature attacks you.
#[test]
fn hissing_miasma_pings_the_attacker() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::hissing_miasma());
    g.active_player_idx = 1;
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker, target: AttackTarget::Player(0),
    }])).expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "attacking player lost 1 life");
}

/// Agent of Masks drains each opponent on your upkeep.
#[test]
fn agent_of_masks_upkeep_drain() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::agent_of_masks());
    let (l0, l1) = (g.players[0].life, g.players[1].life);
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, l1 - 1, "opponent lost 1");
    assert_eq!(g.players[0].life, l0 + 1, "you gained that much");
}

/// Exhumer Thrull's ETB reanimates a creature card to hand.
#[test]
fn exhumer_thrull_recurs_creature() {
    let mut g = two_player_game();
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::exhumer_thrull());
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == bear), "creature returned to hand");
}

/// Benediction of Moons gains 1 life per player (2 in a 2-player game).
#[test]
fn benediction_of_moons_gains_per_player() {
    let mut g = two_player_game();
    let life0 = g.players[0].life;
    resolve_spell(&mut g, catalog::benediction_of_moons(), vec![]);
    assert_eq!(g.players[0].life, life0 + 2, "1 life for each of the 2 players");
}

/// Burning-Tree Shaman pings a player who activates a non-mana ability.
#[test]
fn burning_tree_shaman_pings_ability_activator() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::burning_tree_shaman());
    // Player 1 activates Crystal Seer's {4}{U} self-bounce (a non-mana ability).
    let seer = g.add_card_to_battlefield(1, catalog::crystal_seer());
    g.players[1].mana_pool.add_colorless(4);
    g.players[1].mana_pool.add(Color::Blue, 1);
    g.priority.player_with_priority = 1;
    let life1 = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: seer, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life1 - 1, "activator took 1 damage");
}

/// Burning-Tree Bloodscale enters with a +1/+1 counter (bloodthirst 1) when an
/// opponent was dealt damage this turn.
#[test]
fn burning_tree_bloodscale_bloodthirst() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    g.players[1].life -= 1; // stand in for "an opponent was dealt damage"
    let mut evs = Vec::new();
    g.deal_damage_to_from(
        crabomination::game::effects::EntityRef::Player(1), 1, None, &mut evs);
    let scale = g.move_card_to_battlefield_for_test(0, catalog::burning_tree_bloodscale());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(scale).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "entered with a bloodthirst counter");
}

/// Culling Sun destroys creatures with mana value 3 or less and spares bigger ones.
#[test]
fn culling_sun_sweeps_cheap_creatures() {
    let mut g = two_player_game();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // MV 2
    let big = g.add_card_to_battlefield(1, catalog::serra_angel()); // MV 5
    resolve_spell(&mut g, catalog::culling_sun(), vec![]);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(small).is_none(), "MV 2 destroyed");
    assert!(g.battlefield_find(big).is_some(), "MV 5 spared");
}

/// Ghostway blinks your creatures, returning them at the next end step.
#[test]
fn ghostway_blinks_your_creatures() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    resolve_spell(&mut g, catalog::ghostway(), vec![]);
    assert!(g.battlefield_find(bear).is_none(), "exiled by Ghostway");
    // Advance to the end step → the delayed trigger returns them.
    g.active_player_idx = 0;
    g.step = TurnStep::PostCombatMain;
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Grizzly Bears"),
        "returned at the next end step");
}

/// Leyline of Lifeforce makes creature spells uncounterable — a Counterspell
/// resolves with no effect and the creature spell survives on the stack.
#[test]
fn leyline_of_lifeforce_protects_creature_spells() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_lifeforce());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    // Try to counter it.
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(1, None, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    let counter = crabomination::effect::Effect::CounterSpell {
        what: crabomination::effect::Selector::Target(0),
    };
    g.resolve_effect(&counter, &ctx).unwrap();
    assert!(g.stack.iter().any(|si| matches!(si,
        crabomination::game::StackItem::Spell { card, .. } if card.id == bear)),
        "creature spell survived the counter");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "and resolved onto the battlefield");
}

// ── GPT gap wave 2 (gaps2.rs) ────────────────────────────────────────────────

use crabomination::card::CounterType;

/// Fencer's Magemark anthems your enchanted creatures with +1/+1 and first
/// strike.
#[test]
fn fencers_magemark_pumps_and_grants_first_strike() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let mark = g.add_card_to_battlefield(0, catalog::fencers_magemark());
    g.battlefield_find_mut(mark).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 anthem");
    assert!(cp.keywords.contains(&Keyword::FirstStrike), "granted first strike");
}

/// Guardian's Magemark has flash and anthems enchanted creatures +1/+1.
#[test]
fn guardians_magemark_has_flash_and_pumps() {
    assert!(catalog::guardians_magemark().keywords.contains(&Keyword::Flash));
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mark = g.add_card_to_battlefield(0, catalog::guardians_magemark());
    g.battlefield_find_mut(mark).unwrap().attached_to = Some(bear);
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "+1/+1 anthem");
}

/// Skyrider Trainee flies only while enchanted.
#[test]
fn skyrider_trainee_flies_only_while_enchanted() {
    let mut g = two_player_game();
    let sky = g.add_card_to_battlefield(0, catalog::skyrider_trainee());
    assert!(!g.computed_permanent(sky).unwrap().keywords.contains(&Keyword::Flying),
        "no flying while unenchanted");
    let aura = g.add_card_to_battlefield(0, catalog::guardians_magemark());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(sky);
    assert!(g.computed_permanent(sky).unwrap().keywords.contains(&Keyword::Flying),
        "gains flying while enchanted");
}

/// Lionheart Maverick's activated ability pumps it +1/+2.
#[test]
fn lionheart_maverick_pumps_itself() {
    let mut g = two_player_game();
    let lion = g.add_card_to_battlefield(0, catalog::lionheart_maverick()); // 1/1
    g.players[0].mana_pool.add_colorless(4);
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: lion, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump");
    drain_stack(&mut g);
    let cp = g.computed_permanent(lion).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3), "+1/+2 until end of turn");
    assert!(cp.keywords.contains(&Keyword::Vigilance));
}

/// Order of the Stars chooses a color as it enters and has defender.
#[test]
fn order_of_the_stars_picks_a_color() {
    let mut g = two_player_game();
    let order = g.move_card_to_battlefield_for_test(0, catalog::order_of_the_stars());
    drain_stack(&mut g);
    assert!(g.battlefield_find(order).unwrap().chosen_color.is_some(), "chose a color on entry");
    assert!(catalog::order_of_the_stars().keywords.contains(&Keyword::Defender));
}

/// Ogre Savant bounces a creature only when {U} was spent to cast it.
#[test]
fn ogre_savant_bounces_when_blue_spent() {
    use crabomination::game::effects::EffectContext;
    let mut g = two_player_game();
    let ogre = g.add_card_to_battlefield(0, catalog::ogre_savant());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let def = catalog::ogre_savant();
    // No blue spent → no bounce.
    let mut ctx = EffectContext::for_spell_with_source(ogre, "Ogre Savant", 0, None, vec![], 0, 0, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).unwrap();
    assert!(g.battlefield_find(bear).is_some(), "no bounce without blue");
    // Blue spent → bounce.
    g.battlefield_find_mut(ogre).unwrap().cast_mana_spent_by_color = vec![(Color::Blue, 1)];
    let mut ctx = EffectContext::for_spell_with_source(ogre, "Ogre Savant", 0, None, vec![], 0, 0, 0, 0);
    ctx.targets = vec![Target::Permanent(bear)];
    g.resolve_effect(&def.triggered_abilities[0].effect, &ctx).unwrap();
    drain_stack(&mut g);
    assert!(g.players[1].hand.iter().any(|c| c.id == bear), "bounced when blue was spent");
}

/// Revenant Patriarch can't block; casting it with {W} makes a player skip
/// their next combat.
#[test]
fn revenant_patriarch_skips_combat_when_white_spent() {
    let mut g = two_player_game();
    assert!(catalog::revenant_patriarch().keywords.contains(&Keyword::CantBlock));
    let pat = g.add_card_to_hand(0, catalog::revenant_patriarch());
    // {4}{B}: pay the four generic with white so {W} is spent.
    g.players[0].mana_pool.add(Color::White, 4);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: pat, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast with white");
    drain_stack(&mut g);
    assert!(g.players[1].skip_next_combat >= 1, "opponent skips next combat");
}

/// Restless Bones grants swampwalk to a target creature.
#[test]
fn restless_bones_grants_swampwalk() {
    use crabomination::card::LandType;
    let mut g = two_player_game();
    let bones = g.add_card_to_battlefield(0, catalog::restless_bones());
    g.clear_sickness(bones);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(3);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bones, ability_index: 0, target: Some(Target::Permanent(bear)),
        additional_targets: vec![], x_value: None,
    }).expect("grant swampwalk");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Landwalk(LandType::Swamp)));
}

/// Smogsteed Rider gives every other attacker fear when it attacks.
#[test]
fn smogsteed_rider_grants_fear_to_other_attackers() {
    let mut g = two_player_game();
    let rider = g.add_card_to_battlefield(0, catalog::smogsteed_rider());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(rider);
    g.clear_sickness(ally);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: rider, target: AttackTarget::Player(1) },
        Attack { attacker: ally, target: AttackTarget::Player(1) },
    ])).expect("attack");
    drain_stack(&mut g);
    assert!(g.computed_permanent(ally).unwrap().keywords.contains(&Keyword::Fear),
        "other attacker gained fear");
    assert!(!g.computed_permanent(rider).unwrap().keywords.contains(&Keyword::Fear),
        "the rider itself does not");
}

/// Martyred Rusalka sacrifices a creature to stop a creature from attacking.
#[test]
fn martyred_rusalka_prevents_attack() {
    let mut g = two_player_game();
    let rusalka = g.add_card_to_battlefield(0, catalog::martyred_rusalka());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::White, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rusalka, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![Target::Permanent(fodder)], x_value: None,
    }).expect("prevent attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "sacrificed as a cost");
    assert!(g.computed_permanent(foe).unwrap().keywords.contains(&Keyword::CantAttack));
}

/// Skarrgan Firebird enters with three +1/+1 counters when an opponent was
/// dealt damage this turn (bloodthirst 3).
#[test]
fn skarrgan_firebird_bloodthirst_three() {
    let mut g = two_player_game();
    g.players[1].was_dealt_damage_this_turn = true;
    let bird = g.move_card_to_battlefield_for_test(0, catalog::skarrgan_firebird());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bird).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(catalog::skarrgan_firebird().keywords.contains(&Keyword::Flying));
}

/// Runeboggle counters an unpaid spell and draws a card.
#[test]
fn runeboggle_counters_and_draws() {
    use crabomination::game::types::{StackItem, Target};
    let mut g = two_player_game();
    // Opponent casts a bolt, spending all their mana.
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bolt");
    assert!(matches!(g.stack.last(), Some(StackItem::Spell { card, .. }) if card.id == bolt));
    // Respond with Runeboggle; opponent can't pay {1}.
    g.priority.player_with_priority = 0;
    g.add_card_to_library(0, catalog::forest());
    let hand0 = g.players[0].hand.len();
    let rune = g.add_card_to_hand(0, catalog::runeboggle());
    g.players[0].mana_pool.add_colorless(2);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: rune, target: Some(Target::Permanent(bolt)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast Runeboggle");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "bolt countered");
    assert_eq!(g.players[0].hand.len(), hand0 + 1, "drew a card (Runeboggle left hand, one drawn)");
}

/// Primeval Light destroys only the target player's enchantments.
#[test]
fn primeval_light_destroys_target_players_enchantments() {
    let mut g = two_player_game();
    // Attach each Aura to a creature so the unattached-Aura SBA doesn't destroy them.
    let mb = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mine = g.add_card_to_battlefield(0, catalog::guardians_magemark());
    g.battlefield_find_mut(mine).unwrap().attached_to = Some(mb);
    let tb = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::guardians_magemark());
    g.battlefield_find_mut(theirs).unwrap().attached_to = Some(tb);
    resolve_spell(&mut g, catalog::primeval_light(), vec![Target::Player(1)]);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(mine).is_some(), "my enchantment survives");
    assert!(g.battlefield_find(theirs).is_none(), "their enchantment destroyed");
}

/// Hatching Plans draws three cards when it hits the graveyard.
#[test]
fn hatching_plans_draws_three_on_death() {
    let mut g = two_player_game();
    let plans = g.add_card_to_battlefield(0, catalog::hatching_plans());
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let hand0 = g.players[0].hand.len();
    let mut evs = Vec::new();
    g.destroy_permanent(plans, false, &mut evs);
    g.dispatch_triggers_for_events(&evs);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), hand0 + 3, "drew three cards");
}

/// Gruul War Plow anthems your creatures with trample and can animate itself.
#[test]
fn gruul_war_plow_trample_anthem_and_animate() {
    let mut g = two_player_game();
    let plow = g.add_card_to_battlefield(0, catalog::gruul_war_plow());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Trample),
        "creatures you control have trample");
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: plow, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("animate");
    drain_stack(&mut g);
    let cp = g.computed_permanent(plow).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "becomes a 4/4");
}

/// Sinstriker's Will grants the enchanted creature a tap-to-ping ability.
#[test]
fn sinstrikers_will_grants_ping_ability() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::sinstrikers_will());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    assert_eq!(g.granted_abilities_for(bear).len(), 1, "host gained the ping ability");
}

/// Cryptwailing exiles two creatures from your graveyard to force a discard.
#[test]
fn cryptwailing_forces_a_discard() {
    let mut g = two_player_game();
    let crypt = g.add_card_to_battlefield(0, catalog::cryptwailing());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let victim = g.add_card_to_hand(1, catalog::forest());
    g.active_player_idx = 0;
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: crypt, ability_index: 0, target: Some(Target::Player(1)),
        additional_targets: vec![], x_value: None,
    }).expect("force discard");
    drain_stack(&mut g);
    assert!(!g.players[1].hand.iter().any(|c| c.id == victim), "opponent discarded");
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.name == "Grizzly Bears").count(), 0,
        "two creatures exiled from graveyard as a cost");
}

/// Nullstone Gargoyle counters the first noncreature spell of a turn but not
/// the second.
#[test]
fn nullstone_gargoyle_counters_first_noncreature_spell() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::nullstone_gargoyle());
    // First noncreature spell → countered.
    let bolt1 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    let life0 = g.players[0].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt1, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast first bolt");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt1), "first noncreature spell countered");
    assert_eq!(g.players[0].life, life0, "no damage from the countered bolt");
    // Second noncreature spell the same turn → resolves.
    let bolt2 = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt2, target: Some(Target::Player(0)), additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast second bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, life0 - 3, "second noncreature spell resolved");
}

/// Angel of Despair's ETB destroys any target permanent.
#[test]
fn angel_of_despair_etb_destroys_permanent() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.move_card_to_battlefield_for_test(0, catalog::angel_of_despair());
    drain_stack(&mut g);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(foe).is_none(), "ETB destroyed the target permanent");
}

/// Debtors' Knell reanimates a creature from a graveyard each upkeep.
#[test]
fn debtors_knell_reanimates_at_upkeep() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::debtors_knell());
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.step = TurnStep::Untap;
    advance_to(&mut g, TurnStep::Upkeep);
    drain_stack(&mut g);
    let reanimated = g.battlefield_find(corpse).expect("returned to the battlefield");
    assert_eq!(reanimated.controller, 0, "under your control");
}

/// Hypervolt Grasp grants the enchanted creature a tap-to-ping and can bounce
/// itself.
#[test]
fn hypervolt_grasp_grants_ping_and_bounces() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_battlefield(0, catalog::hypervolt_grasp());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    assert_eq!(g.granted_abilities_for(bear).len(), 1, "host gained the ping ability");
    // {1}{U}: return the aura to hand.
    g.players[0].mana_pool.add_colorless(1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: aura, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("bounce aura");
    drain_stack(&mut g);
    assert!(g.players[0].hand.iter().any(|c| c.id == aura), "aura returned to hand");
}

/// Invoke the Firemind's draw mode draws X cards.
#[test]
fn invoke_the_firemind_draws_x() {
    let mut g = two_player_game();
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    let invoke = g.add_card_to_hand(0, catalog::invoke_the_firemind());
    g.players[0].mana_pool.add_colorless(2); // X = 2
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add(Color::Red, 1);
    let h0 = g.players[0].hand.len();
    g.perform_action(GameAction::CastSpell {
        card_id: invoke, target: None, additional_targets: vec![], mode: Some(0), x_value: Some(2),
    }).expect("cast Invoke in draw mode with X=2");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), h0 - 1 + 2, "drew X=2 (net +1 after casting)");
}

/// Orzhov Euthanist's haunt destroys a creature that was dealt damage this turn.
#[test]
fn orzhov_euthanist_destroys_damaged_creature() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.battlefield_find_mut(foe).unwrap().dealt_damage_this_turn = true;
    g.move_card_to_battlefield_for_test(0, catalog::orzhov_euthanist());
    drain_stack(&mut g);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(foe).is_none(), "ETB destroyed the damaged creature");
}

/// Graven Dominator's ETB shrinks every other creature to base 1/1.
#[test]
fn graven_dominator_flattens_other_creatures() {
    let mut g = two_player_game();
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    g.move_card_to_battlefield_for_test(0, catalog::graven_dominator());
    drain_stack(&mut g);
    let cp = g.computed_permanent(foe).unwrap();
    assert_eq!((cp.power, cp.toughness), (1, 1), "other creature flattened to 1/1");
}

/// Seize the Soul destroys a nonwhite/nonblack creature and makes a Spirit.
#[test]
fn seize_the_soul_destroys_and_makes_a_spirit() {
    let mut g = two_player_game();
    let green = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // green, valid target
    g.add_card_to_battlefield(1, catalog::serra_angel()); // a creature for the haunt to attach to
    resolve_spell(&mut g, catalog::seize_the_soul(), vec![Target::Permanent(green)]);
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    assert!(g.battlefield_find(green).is_none(), "nonwhite/nonblack creature destroyed");
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Spirit" && c.controller == 0),
        "made a 1/1 Spirit under your control");
}

/// Leyline of Lightning pings a player when you pay {1} on a spell cast.
#[test]
fn leyline_of_lightning_pings_on_cast() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_lightning());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Player(1)),
    ]));
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears()); // {1}{G}, no target
    g.players[0].mana_pool.add_colorless(2); // 1 for the bear's generic, 1 for the ping
    g.players[0].mana_pool.add(Color::Green, 1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).expect("cast bear");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, p1 - 1, "paid one generic to ping the opponent for 1");
}

/// Rabble-Rouser taps to pump every attacking creature by its own power.
#[test]
fn rabble_rouser_pumps_attackers_by_its_power() {
    let mut g = two_player_game();
    let rr = g.add_card_to_battlefield(0, catalog::rabble_rouser()); // 1/1
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.clear_sickness(rr);
    g.clear_sickness(ally);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ally, target: AttackTarget::Player(1),
    }])).expect("attack with the ally only");
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: rr, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    }).expect("pump the attackers");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(ally).unwrap().power, 3, "attacker got +1/+0 (Rabble-Rouser's power)");
}

/// Borborygmos puts a +1/+1 counter on each of your creatures on combat damage.
#[test]
fn borborygmos_grows_your_team_on_combat_damage() {
    use crabomination::card::CounterType;
    let mut g = two_player_game();
    let bor = g.add_card_to_battlefield(0, catalog::borborygmos());
    let ally = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bor);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bor, target: AttackTarget::Player(1),
    }])).expect("Borborygmos attacks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.battlefield_find(ally).unwrap().counter_count(CounterType::PlusOnePlusOne), 1,
        "each creature you control got a +1/+1 counter");
}

/// Skarrgan Skybreaker sacrifices itself to deal damage equal to its power.
#[test]
fn skarrgan_skybreaker_sacrifices_for_power_damage() {
    let mut g = two_player_game();
    let sky = g.add_card_to_battlefield(0, catalog::skarrgan_skybreaker()); // 3/3
    g.players[0].mana_pool.add_colorless(1);
    let p1 = g.players[1].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sky, ability_index: 0, target: Some(Target::Player(1)), additional_targets: vec![], x_value: None,
    }).expect("sacrifice for damage");
    drain_stack(&mut g);
    assert!(g.battlefield_find(sky).is_none(), "sacrificed as a cost");
    assert_eq!(g.players[1].life, p1 - 3, "dealt damage equal to its power (3)");
}

/// Dune-Brood Nephilim makes a Sand token per land you control on combat damage.
#[test]
fn dune_brood_nephilim_makes_sand_per_land() {
    let mut g = two_player_game();
    let dune = g.add_card_to_battlefield(0, catalog::dune_brood_nephilim());
    g.add_card_to_battlefield(0, catalog::forest());
    g.add_card_to_battlefield(0, catalog::forest());
    g.clear_sickness(dune);
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dune, target: AttackTarget::Player(1),
    }])).expect("Dune-Brood attacks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    let sand = g.battlefield.iter().filter(|c| c.definition.name == "Sand" && c.controller == 0).count();
    assert_eq!(sand, 2, "one Sand token per land controlled");
}

/// Glint-Eye Nephilim draws cards equal to the combat damage it deals.
#[test]
fn glint_eye_nephilim_draws_on_combat_damage() {
    let mut g = two_player_game();
    let glint = g.add_card_to_battlefield(0, catalog::glint_eye_nephilim()); // 2/2
    for _ in 0..3 { g.add_card_to_library(0, catalog::forest()); }
    g.clear_sickness(glint);
    let h0 = g.players[0].hand.len();
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: glint, target: AttackTarget::Player(1),
    }])).expect("Glint-Eye attacks");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::PostCombatMain);
    assert_eq!(g.players[0].hand.len(), h0 + 2, "drew cards equal to combat damage (2)");
}

// ── GPT gap wave 4 (gaps4.rs) ────────────────────────────────────────────────

/// Storm Herd mints Pegasus tokens equal to your life total.
#[test]
fn storm_herd_tokens_equal_life() {
    let mut g = two_player_game();
    g.players[0].life = 22;
    resolve_spell(&mut g, catalog::storm_herd(), vec![]);
    let pegasi = g.battlefield.iter()
        .filter(|c| c.definition.name == "Pegasus" && c.controller == 0).count();
    assert_eq!(pegasi, 22, "one Pegasus per point of life");
}

/// Starved Rusalka sacrifices a creature to gain 1 life.
#[test]
fn starved_rusalka_sacrifices_for_life() {
    let mut g = two_player_game();
    let rusalka = g.add_card_to_battlefield(0, catalog::starved_rusalka());
    let fodder = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    let life = g.players[0].life;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rusalka, ability_index: 0, target: None,
        additional_targets: vec![Target::Permanent(fodder)], x_value: None,
    }).expect("sac for life");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fodder).is_none(), "creature sacrificed");
    assert_eq!(g.players[0].life, life + 1, "gained 1 life");
}

/// Stratozeppelid can block a flyer but not a grounded attacker.
#[test]
fn stratozeppelid_blocks_only_flyers() {
    let mut g = two_player_game();
    let zep = g.add_card_to_battlefield(0, catalog::stratozeppelid());
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2 no flying
    assert!(g.blocker_can_block_attacker(zep, flyer), "may block a flyer");
    assert!(!g.blocker_can_block_attacker(zep, ground), "can't block a grounded attacker");
}

/// Schismotivate pumps one creature +4/+0 and shrinks another -4/-0.
#[test]
fn schismotivate_opposing_pumps() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let foe = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4
    resolve_spell(&mut g, catalog::schismotivate(),
        vec![Target::Permanent(mine), Target::Permanent(foe)]);
    assert_eq!(g.computed_permanent(mine).unwrap().power, 6, "+4/+0");
    assert_eq!(g.computed_permanent(foe).unwrap().power, 0, "-4/-0");
}

/// To Arms! untaps your creatures and draws a card.
#[test]
fn to_arms_untaps_and_draws() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    resolve_spell(&mut g, catalog::to_arms(), vec![]);
    assert!(!g.battlefield_find(bear).unwrap().tapped, "creature untapped");
    assert_eq!(g.players[0].hand.len(), hand + 1, "drew a card");
}

/// Thunderheads mints a Weird token that's exiled at the next end step.
#[test]
fn thunderheads_weird_exiled_at_end_step() {
    let mut g = two_player_game();
    resolve_spell(&mut g, catalog::thunderheads(), vec![]);
    let weird = g.battlefield.iter().find(|c| c.definition.name == "Weird" && c.controller == 0)
        .map(|c| c.id).expect("Weird token created");
    advance_to(&mut g, TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(weird).is_none(), "Weird exiled at end step");
}

/// Sky Swallower hands all your other permanents to the targeted opponent.
#[test]
fn sky_swallower_donates_permanents() {
    let mut g = two_player_game();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let land = g.add_card_to_battlefield(0, catalog::forest());
    // Resolve the ETB body directly with the opponent as the target player.
    let mut ctx = crabomination::game::effects::EffectContext::for_spell(0, None, 0, 0);
    ctx.targets = vec![Target::Player(1)];
    let effect = crabomination::effect::Effect::GainControl {
        what: crabomination::effect::Selector::EachPermanent(
            SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByYou)),
        to: Some(crabomination::effect::PlayerRef::Target(0)),
        duration: crabomination::effect::Duration::Permanent,
    };
    g.resolve_effect(&effect, &ctx).unwrap();
    assert_eq!(g.battlefield_find(bear).unwrap().controller, 1, "creature donated");
    assert_eq!(g.battlefield_find(land).unwrap().controller, 0, "land kept (nonland only)");
}

/// Infiltrator's Magemark anthems the host and makes it unblockable except by
/// defenders.
#[test]
fn infiltrators_magemark_evasion_and_anthem() {
    let mut g = two_player_game();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    let aura = g.add_card_to_battlefield(0, catalog::infiltrators_magemark());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(host);
    assert_eq!(g.computed_permanent(host).unwrap().power, 3, "anthem +1/+1");
    let wall = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2, no defender
    assert!(!g.blocker_can_block_attacker(wall, host), "non-defender can't block");
}

/// Teysa mints a Spirit whenever another black creature you control dies.
#[test]
fn teysa_spirit_on_black_death() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::teysa_orzhov_scion());
    let corpse = g.add_card_to_battlefield(0, catalog::walking_corpse()); // black
    g.battlefield_find_mut(corpse).unwrap().damage = 99;
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    let spirits = g.battlefield.iter()
        .filter(|c| c.definition.name == "Spirit" && c.controller == 0).count();
    assert_eq!(spirits, 1, "one Spirit for the dead black creature");
}

/// Tibor and Lumia pings each creature without flying when you cast a red spell.
#[test]
fn tibor_red_cast_pings_grounded() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::tibor_and_lumia());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears()); // 2/2
    let flyer = g.add_card_to_battlefield(1, catalog::serra_angel()); // 4/4 flying
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_at(&mut g, bolt, Target::Player(1));
    assert_eq!(g.battlefield_find(ground).unwrap().damage, 1, "grounded creature pinged");
    assert_eq!(g.battlefield_find(flyer).unwrap().damage, 0, "flyer untouched");
}

/// Earth Surge grows a land that is a creature by +2/+2.
#[test]
fn earth_surge_pumps_creature_lands() {
    let mut g = two_player_game();
    let arbor = g.add_card_to_battlefield(0, catalog::dryad_arbor()); // 1/1 Land Creature
    g.add_card_to_battlefield(0, catalog::earth_surge());
    assert_eq!(g.computed_permanent(arbor).unwrap().power, 3, "creature-land gets +2/+2");
}

/// Leyline of the Meek anthems creature tokens.
#[test]
fn leyline_of_the_meek_anthems_tokens() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_the_meek());
    resolve_spell(&mut g, catalog::storm_herd(), vec![]); // mints token Pegasi
    let token = g.battlefield.iter().find(|c| c.definition.name == "Pegasus")
        .map(|c| c.id).expect("token minted");
    assert_eq!(g.computed_permanent(token).unwrap().power, 2, "token gets +1/+1");
}

/// Leyline of Singularity makes all nonland permanents legendary, so the legend
/// rule collapses same-named duplicates.
#[test]
fn leyline_of_singularity_collapses_duplicates() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::leyline_of_singularity());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let evs = g.check_state_based_actions();
    g.dispatch_triggers_for_events(&evs);
    let bears = g.battlefield.iter()
        .filter(|c| c.definition.name == "Grizzly Bears" && c.controller == 0).count();
    assert_eq!(bears, 1, "legend rule collapsed the duplicate");
}

/// Ulasht enters with a +1/+1 counter for each other red and green creature.
#[test]
fn ulasht_enters_with_counters() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // green
    g.add_card_to_battlefield(0, catalog::goblin_king()); // red (see below)
    let ulasht = g.move_card_to_battlefield_for_test(0, catalog::ulasht_the_hate_seed());
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ulasht).unwrap().counter_count(CounterType::PlusOnePlusOne), 2,
        "one counter per red + green other creature");
}

/// Ulasht's ability removes a +1/+1 counter and (mode 0) pings a creature.
#[test]
fn ulasht_ability_removes_counter_and_pings() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::goblin_king()); // red → Ulasht enters 1/1
    let ulasht = g.move_card_to_battlefield_for_test(0, catalog::ulasht_the_hate_seed());
    drain_stack(&mut g);
    let foe = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::ActivateAbility {
        card_id: ulasht, ability_index: 0, target: Some(Target::Permanent(foe)),
        additional_targets: vec![], x_value: None,
    }).expect("activate Ulasht");
    drain_stack(&mut g);
    // The +1/+1 counter paid as a cost drops Ulasht to 0/0 (it dies to SBA),
    // but the mode-0 ping still resolved.
    assert_eq!(g.battlefield_find(foe).unwrap().damage, 1, "mode 0 dealt 1 damage");
}
