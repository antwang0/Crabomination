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
