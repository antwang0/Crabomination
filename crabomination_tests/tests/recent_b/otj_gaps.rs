//! Outlaws of Thunder Junction gap batch (`decks::recent309`) — plot payoffs,
//! the "Joins Up" cycle, and the outlaw legends.

use crabomination::card::{CardDefinition, CardInstance, CounterType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn main_phase() -> GameState {
    let mut g = two_player_game();
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// Put `def` onto seat 0's battlefield and fire its ETB.
fn etb(g: &mut GameState, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(0, def);
    g.fire_self_etb_triggers(id, 0);
    drain_stack(g);
    id
}

/// Deal combat damage to seat 1 with `attacker` (already on the battlefield).
fn swing(g: &mut GameState, attacker: CardId) {
    g.clear_sickness(attacker);
    g.active_player_idx = 0;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.declare_attackers(vec![Attack { attacker, target: AttackTarget::Player(1) }])
        .expect("attack");
    g.step = TurnStep::DeclareBlockers;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::EndCombat {
        let _ = g.advance_step(Vec::new());
        drain_stack(g);
    }
    drain_stack(g);
}

/// Hard-cast Obeka from seat 0's hand — the cast path is what fires other
/// permanents' "whenever a legendary creature you control enters" watchers.
fn cast_a_legend(g: &mut GameState) -> CardId {
    let id = g.add_card_to_hand(0, catalog::obeka_splitter_of_seconds());
    for c in [Color::Blue, Color::Black, Color::Red] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast the legend");
    drain_stack(g);
    id
}

// ── Plot (CR 702.170) ───────────────────────────────────────────────────────

/// Kellan Joins Up plots a cheap nonland card out of hand — it lands in exile
/// marked plotted, castable for free next turn.
#[test]
fn kellan_joins_up_plots_a_hand_card_and_it_is_castable_next_turn() {
    let mut g = main_phase();
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    etb(&mut g, catalog::kellan_joins_up());
    assert!(g.exile.iter().any(|c| c.id == bear), "the bear was exiled");
    assert!(g.plotted_cards.contains(&bear), "and marked plotted");
    // Not this turn (CR 702.170d) …
    assert!(g.perform_action(GameAction::CastPlotted {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    }).is_err());
    // … but free on a later turn.
    g.plotted_this_turn.clear();
    g.perform_action(GameAction::CastPlotted {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("free plotted cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some());
}

/// Kellan Joins Up's second half pumps the whole team when a legendary
/// creature arrives.
#[test]
fn kellan_joins_up_counters_the_team_on_a_legend() {
    let mut g = main_phase();
    etb(&mut g, catalog::kellan_joins_up());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    cast_a_legend(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Make Your Own Luck plots one of the top three and takes the rest.
#[test]
fn make_your_own_luck_plots_one_and_draws_the_rest() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::make_your_own_luck());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    let hand_before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.exile.iter().filter(|c| g.plotted_cards.contains(&c.id)).count(), 1);
    assert_eq!(g.players[0].hand.len(), hand_before + 2, "the other two went to hand");
}

/// Aven Interrupter's ETB lifts a spell off the stack and plots it for its
/// owner.
#[test]
fn aven_interrupter_plots_the_spell_it_exiles() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(0)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast bolt");
    g.priority.player_with_priority = 0;
    let aven = g.add_card_to_battlefield(0, catalog::aven_interrupter());
    g.fire_self_etb_triggers(aven, 0);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "the bolt never resolved");
    assert!(g.plotted_cards.contains(&bolt), "it's plotted instead");
}

/// Fblthp lets you plot straight off the top of your library, paying the
/// card's own mana cost.
#[test]
fn fblthp_plots_from_the_library_top() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::fblthp_lost_on_the_range());
    let top = g.next_id();
    g.players[0].library.insert(0, CardInstance::new(top, catalog::grizzly_bears(), 0));
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::Plot { card_id: top }).expect("plot off the top");
    assert!(g.plotted_cards.contains(&top));
    assert!(g.players[0].library.iter().all(|c| c.id != top), "it left the library");
}

// ── "Joins Up" cycle ────────────────────────────────────────────────────────

/// Annie Joins Up shoots on arrival and doubles your legends' triggers.
#[test]
fn annie_joins_up_shoots_then_doubles_legendary_triggers() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::elder_gargaroth());
    etb(&mut g, catalog::annie_joins_up());
    assert_eq!(g.battlefield_find(victim).map(|c| c.damage), Some(5));
    // Ertha Jo's own ETB now fires twice → two Mercenaries.
    let ertha = g.add_card_to_hand(0, catalog::ertha_jo_frontier_mentor());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpell {
        card_id: ertha, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Ertha Jo");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(),
        2,
        "the legend's ETB fired twice"
    );
}

/// Rakdos Joins Up reanimates with two extra counters, then drains on a
/// legend's death.
#[test]
fn rakdos_joins_up_reanimates_then_burns_on_a_legend_dying() {
    let mut g = main_phase();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    etb(&mut g, catalog::rakdos_joins_up());
    let back = g.battlefield_find(dead).expect("reanimated");
    assert_eq!(back.counter_count(CounterType::PlusOnePlusOne), 2);
    // Bolt a 2/3 legend down: the damage → SBA → dispatch cycle is what fires
    // the "whenever a legendary creature you control dies" watcher.
    let satoru = g.add_card_to_battlefield(0, catalog::satoru_the_infiltrator());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Permanent(satoru)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("bolt the legend");
    drain_stack(&mut g);
    assert!(g.battlefield_find(satoru).is_none());
    assert_eq!(g.players[1].life, 18, "2 power off Satoru");
}

/// Tinybones Joins Up strips a card, then mills and drains on each legend.
#[test]
fn tinybones_joins_up_discards_then_mills_on_a_legend() {
    let mut g = main_phase();
    g.add_card_to_hand(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::grizzly_bears());
    etb(&mut g, catalog::tinybones_joins_up());
    assert_eq!(g.players[1].hand.len(), 0, "discarded");
    cast_a_legend(&mut g);
    assert_eq!(g.players[1].graveyard.len(), 2, "milled on top of the discard");
}

/// Vraska Joins Up hands out deathtouch counters and draws off a legend's hit.
#[test]
fn vraska_joins_up_gives_deathtouch_then_draws() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    etb(&mut g, catalog::vraska_joins_up());
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Deathtouch),
        "deathtouch counter grants the keyword"
    );
    let legend = g.add_card_to_battlefield(0, catalog::obeka_splitter_of_seconds());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    swing(&mut g, legend);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

// ── Outlaw legends ──────────────────────────────────────────────────────────

/// Obeka banks an extra upkeep per point of combat damage.
#[test]
fn obeka_banks_extra_upkeeps_equal_to_the_damage() {
    let mut g = main_phase();
    let obeka = g.add_card_to_battlefield(0, catalog::obeka_splitter_of_seconds());
    swing(&mut g, obeka);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.additional_upkeep_steps, 2);
}

/// Akul sacrifices three creatures to cheat a fatty in from hand.
#[test]
fn akul_sacrifices_three_to_deploy_from_hand() {
    let mut g = main_phase();
    let akul = g.add_card_to_battlefield(0, catalog::akul_the_unrepentant());
    g.clear_sickness(akul);
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    let fatty = g.add_card_to_hand(0, catalog::elder_gargaroth());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: akul, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac three");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fatty).is_some(), "the hand creature is in play");
    assert_eq!(g.players[0].graveyard.len(), 3);
}

/// Geralf mints a Zombie off the turn's second spell, and the next Zombie
/// enters with a counter per earlier arrival.
#[test]
fn geralf_makes_zombies_that_grow_on_each_other() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::geralf_the_fleshwright());
    g.players[0].spells_cast_this_turn = 1;
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("second spell");
    drain_stack(&mut g);
    let zombies: Vec<_> = g
        .battlefield
        .iter()
        .filter(|c| c.definition.name == "Zombie Rogue")
        .map(|c| (c.id, c.counter_count(CounterType::PlusOnePlusOne)))
        .collect();
    assert_eq!(zombies.len(), 1, "one Zombie from the second spell");
    assert_eq!(zombies[0].1, 0, "no earlier Zombie to count");
}

/// Selvala taps for one mana per distinct power among your creatures.
#[test]
fn selvala_scales_mana_with_distinct_powers() {
    let mut g = main_phase();
    let selvala = g.add_card_to_battlefield(0, catalog::selvala_eager_trailblazer());
    g.clear_sickness(selvala);
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // 2/2
    g.add_card_to_battlefield(0, catalog::grizzly_bears()); // duplicate power
    g.perform_action(GameAction::ActivateAbility {
        card_id: selvala, ability_index: 0, target: None, additional_targets: vec![],
        x_value: None,
    })
    .expect("tap for mana");
    // Powers present: Selvala 4, bears 2 → two distinct values.
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

/// Selvala's other half makes a Mercenary whenever you cast a creature.
#[test]
fn selvala_mints_a_mercenary_on_a_creature_spell() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::selvala_eager_trailblazer());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(), 1);
}

/// Satoru draws off an uncast arrival but not off a hard-cast one.
#[test]
fn satoru_draws_only_for_uncast_arrivals() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::satoru_the_infiltrator());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let cheated = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let zombify = g.add_card_to_hand(0, catalog::zombify());
    let before = g.players[0].hand.len() - 1; // Zombify itself leaves hand
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: zombify, target: Some(Target::Permanent(cheated)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("reanimate");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1, "put onto the battlefield → draw");
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    let before = g.players[0].hand.len() - 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bear, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("hard cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before, "a cast creature draws nothing");
}

/// Bonny Pall brings a land-scaled Ox and draws on the attack.
#[test]
fn bonny_pall_makes_beau_and_draws_on_attack() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let bonny = etb(&mut g, catalog::bonny_pall_clearcutter());
    let beau = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Beau")
        .expect("Beau token")
        .id;
    let cp = g.computed_permanent(beau).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "P/T tracks your three lands");
    g.add_card_to_library(0, catalog::grizzly_bears());
    let before = g.players[0].hand.len();
    swing(&mut g, bonny);
    assert!(g.players[0].hand.len() > before, "attack drew");
}

/// Kambal copies an opponent's token once a turn and drains on your own.
#[test]
fn kambal_copies_opposing_tokens_and_drains_on_yours() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::kambal_profiteering_mayor());
    let fodder = g.add_card_to_hand(1, catalog::dragon_fodder());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: fodder, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("opponent makes Goblins");
    drain_stack(&mut g);
    assert!(
        g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Goblin"),
        "copied under your control"
    );
    assert!(g.players[0].life > 20, "your own token arrival drained");
}

/// Lazav grows on your first crime each turn.
#[test]
fn lazav_grows_on_your_first_crime() {
    let mut g = main_phase();
    let lazav = g.add_card_to_battlefield(0, catalog::lazav_familiar_stranger());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("commit a crime");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(lazav).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Rakdos, the Muscle impulses off a sacrifice and can go indestructible.
#[test]
fn rakdos_the_muscle_impulses_on_a_sacrifice() {
    let mut g = main_phase();
    let rakdos = g.add_card_to_battlefield(0, catalog::rakdos_the_muscle());
    g.clear_sickness(rakdos);
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    for _ in 0..3 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: rakdos, ability_index: 0, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("sac a creature");
    drain_stack(&mut g);
    let r = g.battlefield_find(rakdos).unwrap();
    assert!(r.tapped, "the ability taps it");
    assert!(
        g.computed_permanent(rakdos).unwrap().keywords.contains(&Keyword::Indestructible)
    );
    assert_eq!(g.exile.len(), 2, "two cards impulsed for the bear's mana value");
}

/// Laughing Jasper Flint makes stolen creatures Mercenaries.
#[test]
fn laughing_jasper_flint_retypes_borrowed_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::laughing_jasper_flint());
    let stolen = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.battlefield_find_mut(stolen).unwrap().controller = 0;
    let types = &g.computed_permanent(stolen).unwrap().subtypes.creature_types;
    assert!(types.contains(&crabomination::card::CreatureType::Mercenary));
}

/// Annie Flash reanimates a cheap permanent tapped when she's hard-cast.
#[test]
fn annie_flash_reanimates_when_cast() {
    let mut g = main_phase();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let annie = g.add_card_to_hand(0, catalog::annie_flash_the_veteran());
    for c in [Color::Red, Color::Green, Color::White] {
        g.players[0].mana_pool.add(c, 1);
    }
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: annie, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast Annie");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some_and(|c| c.tapped), "reanimated tapped");
}

/// Ghired's grant lets a nontoken creature copy a fresh token.
#[test]
fn ghired_grants_the_token_copy_ability() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::ghired_mirror_of_the_wilds());
    let body = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(body);
    let token = g.add_card_to_battlefield(0, catalog::elder_gargaroth());
    {
        let t = g.battlefield_find_mut(token).unwrap();
        t.is_token = true;
        t.entered_turn = Some(1);
    }
    g.perform_action(GameAction::ActivateAbility {
        card_id: body, ability_index: 0, target: Some(Target::Permanent(token)),
        additional_targets: vec![], x_value: None,
    })
    .expect("granted copy ability");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Elder Gargaroth").count(),
        2
    );
}

/// Stop Cold taps its host and blanks it.
#[test]
fn stop_cold_taps_and_blanks_the_enchanted_permanent() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(1, catalog::elder_gargaroth());
    let aura = g.add_card_to_hand(0, catalog::stop_cold());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: aura, target: Some(Target::Permanent(host)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast the Aura");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert!(g.battlefield_find(host).unwrap().tapped, "tapped on arrival");
    assert!(cp.keywords.is_empty(), "loses all abilities");
}

/// Arid Archway enters tapped, bounces a land, and surveils off a Desert.
#[test]
fn arid_archway_bounces_a_land_and_surveils_on_a_desert() {
    let mut g = main_phase();
    let other = g.add_card_to_battlefield(0, catalog::arid_archway());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let archway = g.add_card_to_battlefield(0, catalog::arid_archway());
    g.fire_self_etb_triggers(archway, 0);
    drain_stack(&mut g);
    assert!(
        g.players[0].hand.iter().any(|c| c.id == other || c.id == archway),
        "a land went back to hand"
    );
    assert!(g.battlefield_find(archway).is_none_or(|c| c.tapped), "enters tapped");
}

/// Shifting Grift's creature mode swaps control of two creatures.
#[test]
fn shifting_grift_swaps_two_creatures() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::elder_gargaroth());
    let grift = g.add_card_to_hand(0, catalog::shifting_grift());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: grift, spree_modes: vec![0], target: Some(Target::Permanent(theirs)),
        additional_targets: vec![], x_value: None,
    })
    .expect("cast with the creature mode");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
}

/// One Last Job's first mode reanimates a creature.
#[test]
fn one_last_job_reanimates_a_creature() {
    let mut g = main_phase();
    let dead = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let job = g.add_card_to_hand(0, catalog::one_last_job());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: job, spree_modes: vec![0], target: Some(Target::Permanent(dead)),
        additional_targets: vec![], x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(dead).is_some());
}

/// Great Train Heist's untap mode also grants an extra combat phase.
#[test]
fn great_train_heist_untaps_and_adds_a_combat() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let heist = g.add_card_to_hand(0, catalog::great_train_heist());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastSpellSpree {
        card_id: heist, spree_modes: vec![0], target: None, additional_targets: vec![],
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.additional_combat_phases, 1);
}

/// Step Between Worlds refills both players and exiles itself.
#[test]
fn step_between_worlds_refills_and_exiles_itself() {
    let mut g = main_phase();
    for seat in 0..2 {
        for _ in 0..10 {
            g.add_card_to_library(seat, catalog::grizzly_bears());
        }
    }
    let step = g.add_card_to_hand(0, catalog::step_between_worlds());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(3);
    g.perform_action(GameAction::CastSpell {
        card_id: step, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 7);
    assert_eq!(g.players[1].hand.len(), 7);
    assert!(g.exile.iter().any(|c| c.id == step), "exiled on resolution");
}

/// Archmage's Newt hands a graveyard spell flashback when it connects.
#[test]
fn archmages_newt_grants_flashback_on_combat_damage() {
    let mut g = main_phase();
    let newt = g.add_card_to_battlefield(0, catalog::archmages_newt());
    let bolt = g.add_card_to_graveyard(0, catalog::lightning_bolt());
    swing(&mut g, newt);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PostCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastFlashback {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("granted flashback");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 20 - 2 - 3);
}

/// Ertha Jo brings a Mercenary along.
#[test]
fn ertha_jo_makes_a_mercenary() {
    let mut g = main_phase();
    etb(&mut g, catalog::ertha_jo_frontier_mentor());
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Mercenary").count(), 1);
}

// ── Batch 2: planeswalkers, graveyard theft, the Desert ─────────────────────

/// Jace's +1 loots, and his second +1 plots a cheap card out of hand.
#[test]
fn jace_reawakened_loots_then_plots() {
    let mut g = main_phase();
    let jace = g.add_card_to_battlefield(0, catalog::jace_reawakened());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let plottable = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace, ability_index: 0, target: None, x_value: None,
    })
    .expect("+1 loot");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(jace).unwrap().counter_count(CounterType::Loyalty), 4);
    g.battlefield_find_mut(jace).unwrap().loyalty_uses_this_turn = 0;
    g.players[0].activated_loyalty_this_turn = false;
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace, ability_index: 1, target: None, x_value: None,
    })
    .expect("+1 plot");
    drain_stack(&mut g);
    assert!(
        g.plotted_cards.contains(&plottable) || g.plotted_cards.len() == 1,
        "a cheap nonland card was plotted"
    );
}

/// Jace's ultimate copies each spell cast for the rest of the turn.
#[test]
fn jace_reawakened_ultimate_copies_your_spells() {
    let mut g = main_phase();
    let jace = g.add_card_to_battlefield(0, catalog::jace_reawakened());
    g.battlefield_find_mut(jace).unwrap().add_counters(CounterType::Loyalty, 6);
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: jace, ability_index: 2, target: None, x_value: None,
    })
    .expect("-6");
    drain_stack(&mut g);
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("cast bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 14, "the bolt plus its copy");
}

/// Oko's −1 makes a 3/3 Elk; his begin-combat trigger copies one of your
/// creatures with hexproof bolted on.
#[test]
fn oko_makes_an_elk_then_copies_a_creature_at_combat() {
    let mut g = main_phase();
    let oko = g.add_card_to_battlefield(0, catalog::oko_the_ringleader());
    g.perform_action(GameAction::ActivateLoyaltyAbility {
        card_id: oko, ability_index: 1, target: None, x_value: None,
    })
    .expect("-1");
    drain_stack(&mut g);
    let elk = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Elk")
        .expect("Elk token")
        .id;
    assert_eq!(g.computed_permanent(elk).map(|c| (c.power, c.toughness)), Some((3, 3)));
    g.step = TurnStep::Upkeep;
    while g.step != TurnStep::BeginCombat {
        let _ = g.advance_step(Vec::new());
    }
    drain_stack(&mut g);
    let cp = g.computed_permanent(oko).expect("Oko is a copy now");
    assert!(cp.keywords.contains(&Keyword::Hexproof), "the copy keeps hexproof");
    assert_eq!((cp.power, cp.toughness), (3, 3), "copied the Elk");
}

/// Kaervek recasts a black graveyard card as a copy when you commit a crime.
#[test]
fn kaervek_recasts_a_black_graveyard_card_on_a_crime() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::kaervek_the_punisher());
    g.add_card_to_graveyard(0, catalog::dark_ritual());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.perform_action(GameAction::CastSpell {
        card_id: bolt, target: Some(Target::Player(1)), additional_targets: vec![],
        mode: None, x_value: None,
    })
    .expect("commit a crime");
    drain_stack(&mut g);
    assert!(g.players[0].life < 20, "paid life for the recast copy");
}

/// Tinybones steals a permanent card out of the player he hit.
#[test]
fn tinybones_the_pickpocket_casts_from_their_graveyard() {
    let mut g = main_phase();
    let bones = g.add_card_to_battlefield(0, catalog::tinybones_the_pickpocket());
    let theirs = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    swing(&mut g, bones);
    assert_eq!(
        g.battlefield_find(theirs).map(|c| c.controller),
        Some(0),
        "cast from their graveyard, under your control"
    );
}

/// The Key to the Vault digs as deep as the damage and hands you a free cast.
#[test]
fn the_key_to_the_vault_digs_on_combat_damage() {
    let mut g = main_phase();
    let key = g.add_card_to_battlefield(0, catalog::the_key_to_the_vault());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::Equip { equipment: key, target: bear }).expect("equip");
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::lightning_bolt());
    }
    swing(&mut g, bear);
    assert_eq!(g.players[0].library.len(), 3, "two seen, one exiled, one bottomed");
}

/// Bucolic Ranch's second ability makes Mount-only mana that casts a Mount.
#[test]
fn bucolic_ranch_mount_mana_casts_a_mount() {
    let mut g = main_phase();
    let ranch = g.add_card_to_battlefield(0, catalog::bucolic_ranch());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Blue)]));
    g.perform_action(GameAction::ActivateAbility {
        card_id: ranch, ability_index: 1, target: None, additional_targets: vec![], x_value: None,
    })
    .expect("tap for Mount mana");
    g.players[0].mana_pool.add_colorless(1);
    let newt = g.add_card_to_hand(0, catalog::archmages_newt());
    g.perform_action(GameAction::CastSpell {
        card_id: newt, target: None, additional_targets: vec![], mode: None, x_value: None,
    })
    .expect("the Mount spell accepts the restricted mana");
    drain_stack(&mut g);
    assert!(g.battlefield_find(newt).is_some());
}
