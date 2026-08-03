//! Onslaught (ONS) — the tribal block's opener: lords, tribal counts and the
//! "creature type of your choice" spells.

use crabomination::card::{CardId, CounterType, CreatureType, Keyword};
use crabomination::catalog;
use crabomination::decision::{DecisionAnswer, ScriptedDecider};
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target};
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
    cast_x(g, seat, id, target, None);
}

fn cast_x(g: &mut GameState, seat: usize, id: CardId, target: Option<Target>, x: Option<u32>) {
    mana(g, seat);
    g.priority.player_with_priority = seat;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: x,
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

/// Aven Brigadier lords over Birds and Soldiers at once.
#[test]
fn aven_brigadier_lords_two_tribes() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::aven_brigadier());
    let bird = g.add_card_to_battlefield(0, catalog::screeching_buzzard());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    assert_eq!(g.computed_permanent(bird).unwrap().power, 3, "2/2 Bird +1/+1");
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2, "1/1 Soldier +1/+1");
}

/// Airborne Aid counts every Bird on the battlefield, not just yours.
#[test]
fn airborne_aid_counts_every_bird() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::screeching_buzzard());
    g.add_card_to_battlefield(1, catalog::aven_brigadier());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::airborne_aid());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 2);
}

/// Tribal Unity pumps only the named type, by the paid X.
#[test]
fn tribal_unity_pumps_the_named_tribe() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    let beast = g.add_card_to_battlefield(0, catalog::barkhide_mauler());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let spell = g.add_card_to_hand(0, catalog::tribal_unity());
    cast_x(&mut g, 0, spell, None, Some(3));
    assert_eq!(g.computed_permanent(elf).unwrap().power, 4, "1/2 Elf +3/+3");
    assert_eq!(g.computed_permanent(beast).unwrap().power, 4, "Beasts untouched");
}

/// Brightstone Ritual scales with the Goblin count.
#[test]
fn brightstone_ritual_pays_per_goblin() {
    let mut g = main_phase();
    for seat in [0, 1] {
        g.add_card_to_battlefield(seat, catalog::greasewrench_goblin());
    }
    let spell = g.add_card_to_hand(0, catalog::brightstone_ritual());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.amount(Color::Red), 21, "20 seeded, one red paid, 2 Goblins");
}

/// Infest shrinks the whole board and kills the small.
#[test]
fn infest_sweeps_two_toughness() {
    let mut g = main_phase();
    let small = g.add_card_to_battlefield(1, catalog::wirewood_elf());
    let big = g.add_card_to_battlefield(1, catalog::barkhide_mauler());
    let spell = g.add_card_to_hand(0, catalog::infest());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(small).is_none(), "1/2 dies");
    assert_eq!(g.computed_permanent(big).unwrap().toughness, 2);
}

/// Symbiotic Wurm leaves seven Insects behind.
#[test]
fn symbiotic_wurm_leaves_a_swarm() {
    let mut g = main_phase();
    let wurm = g.add_card_to_battlefield(0, catalog::symbiotic_wurm());
    let _ = g.remove_to_graveyard_with_triggers(wurm);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(), 7);
}

/// Cabal Archon turns Clerics into a two-point drain.
#[test]
fn cabal_archon_drains_for_a_cleric() {
    let mut g = main_phase();
    let archon = g.add_card_to_battlefield(0, catalog::cabal_archon());
    g.add_card_to_battlefield(0, catalog::nova_cleric());
    activate(&mut g, 0, archon, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22);
}

/// Wretched Anurid bleeds you for every other creature that arrives.
#[test]
fn wretched_anurid_bleeds_on_arrivals() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wretched_anurid());
    let bear = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.active_player_idx = 1;
    cast(&mut g, 1, bear, None);
    assert_eq!(g.players[0].life, 19);
}

/// Visara kills anything, and it stays dead.
#[test]
fn visara_kills_without_regeneration() {
    let mut g = main_phase();
    let visara = g.add_card_to_battlefield(0, catalog::visara_the_dreadful());
    g.clear_sickness(visara);
    let victim = g.add_card_to_battlefield(1, catalog::silvos_rogue_elemental());
    g.battlefield_find_mut(victim).unwrap().regeneration_shields = 1;
    activate(&mut g, 0, visara, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none());
}

/// Wave of Indifference sidelines exactly X blockers.
#[test]
fn wave_of_indifference_benches_x_creatures() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::wave_of_indifference());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(1),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(a).unwrap().keywords.contains(&Keyword::CantBlock));
    assert!(!g.computed_permanent(b).unwrap().keywords.contains(&Keyword::CantBlock));
}

/// Gustcloak Runner ducks out of the block it walked into.
#[test]
fn gustcloak_runner_slips_the_block() {
    let mut g = main_phase();
    let runner = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(runner);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: runner,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, runner)])).expect("block");
    drain_stack(&mut g);
    assert!(
        !g.attacking().iter().any(|a| a.attacker == runner),
        "the Runner slipped out of combat"
    );
}

/// Mobilization hands Soldiers vigilance and mints more of them.
#[test]
fn mobilization_arms_the_soldiers() {
    let mut g = main_phase();
    let mob = g.add_card_to_battlefield(0, catalog::mobilization());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    assert!(g.computed_permanent(soldier).unwrap().keywords.contains(&Keyword::Vigilance));
    activate(&mut g, 0, mob, 0, None);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Soldier").count(), 1);
}

/// Overwhelming Instinct only pays off a three-wide attack.
#[test]
fn overwhelming_instinct_needs_three_attackers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::overwhelming_instinct());
    let attackers: Vec<CardId> = (0..3)
        .map(|_| {
            let id = g.add_card_to_battlefield(0, catalog::grizzly_bears());
            g.clear_sickness(id);
            id
        })
        .collect();
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(
        attackers.iter().map(|&a| Attack { attacker: a, target: AttackTarget::Player(1) }).collect(),
    ))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// True Believer makes you untargetable.
#[test]
fn true_believer_gives_you_shroud() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::true_believer());
    assert!(g.player_has_static_shroud(0));
    assert!(!g.player_has_static_shroud(1));
}

/// Slate of Ancestry trades your hand for a card per creature.
#[test]
fn slate_of_ancestry_refills_off_the_board() {
    let mut g = main_phase();
    let slate = g.add_card_to_battlefield(0, catalog::slate_of_ancestry());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
    }
    g.add_card_to_hand(0, catalog::forest());
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::forest());
    }
    activate(&mut g, 0, slate, 0, None);
    assert_eq!(g.players[0].hand.len(), 3, "hand pitched, three creatures drawn");
}

/// Aphetto Dredging buys back up to three of the named tribe.
#[test]
fn aphetto_dredging_returns_a_tribe() {
    let mut g = main_phase();
    for _ in 0..2 {
        g.add_card_to_graveyard(0, catalog::wirewood_elf());
    }
    g.add_card_to_graveyard(0, catalog::barkhide_mauler());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let spell = g.add_card_to_hand(0, catalog::aphetto_dredging());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.players[0].hand.len(), 2, "both Elves, not the Beast");
}

/// Riptide Laboratory saves a Wizard from removal.
#[test]
fn riptide_laboratory_rescues_a_wizard() {
    let mut g = main_phase();
    let lab = g.add_card_to_battlefield(0, catalog::riptide_laboratory());
    let wizard = g.add_card_to_battlefield(0, catalog::crafty_pathmage());
    activate(&mut g, 0, lab, 1, Some(Target::Permanent(wizard)));
    assert!(g.battlefield_find(wizard).is_none());
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Silklash Spider shoots down every flier for X.
#[test]
fn silklash_spider_sweeps_the_sky() {
    let mut g = main_phase();
    let spider = g.add_card_to_battlefield(0, catalog::silklash_spider());
    let flier = g.add_card_to_battlefield(1, catalog::screeching_buzzard());
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: spider,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(flier).is_none());
    assert!(g.battlefield_find(ground).is_some());
}

/// Nova Cleric wipes every enchantment, including its own team's.
#[test]
fn nova_cleric_sweeps_enchantments() {
    let mut g = main_phase();
    let cleric = g.add_card_to_battlefield(0, catalog::nova_cleric());
    g.clear_sickness(cleric);
    let mine = g.add_card_to_battlefield(0, catalog::mobilization());
    let theirs = g.add_card_to_battlefield(1, catalog::righteous_cause());
    activate(&mut g, 0, cleric, 0, None);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
}

/// Wall of Mulch turns spare Walls into cards.
#[test]
fn wall_of_mulch_cashes_in_a_wall() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::wall_of_mulch());
    g.add_card_to_battlefield(0, catalog::wall_of_mulch());
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, 0, wall, 0, None);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Mythic Proportions is +8/+8 and trample.
#[test]
fn mythic_proportions_is_a_monster() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::mythic_proportions());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (10, 10));
    assert!(cp.keywords.contains(&Keyword::Trample));
}

/// Reminisce puts a graveyard back into its library.
#[test]
fn reminisce_recycles_a_graveyard() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_graveyard(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::reminisce());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert!(g.players[1].graveyard.is_empty());
    assert_eq!(g.players[1].library.len(), 4);
}

/// Blatant Thievery takes a permanent outright.
#[test]
fn blatant_thievery_steals_a_permanent() {
    let mut g = main_phase();
    let prize = g.add_card_to_battlefield(1, catalog::silvos_rogue_elemental());
    let spell = g.add_card_to_hand(0, catalog::blatant_thievery());
    cast(&mut g, 0, spell, Some(Target::Permanent(prize)));
    assert_eq!(g.battlefield_find(prize).unwrap().controller, 0);
}

/// Cabal Slaver strips a card off every Goblin hit.
#[test]
fn cabal_slaver_punishes_goblin_damage() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::cabal_slaver());
    let goblin = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    g.add_card_to_hand(1, catalog::forest());
    g.clear_sickness(goblin);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: goblin,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
}

/// Centaur Glade makes a 3/3 on demand.
#[test]
fn centaur_glade_mints_a_centaur() {
    let mut g = main_phase();
    let glade = g.add_card_to_battlefield(0, catalog::centaur_glade());
    activate(&mut g, 0, glade, 0, None);
    let token = g.battlefield.iter().find(|c| c.definition.name == "Centaur").expect("token");
    assert_eq!((token.definition.power, token.definition.toughness), (3, 3));
}

/// Aggravated Assault untaps the team and buys another combat.
#[test]
fn aggravated_assault_buys_a_combat() {
    let mut g = main_phase();
    let assault = g.add_card_to_battlefield(0, catalog::aggravated_assault());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, 0, assault, 0, None);
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.additional_post_main_combats, 1);
}

/// Wirewood Savage draws off every Beast that shows up.
#[test]
fn wirewood_savage_draws_for_beasts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::wirewood_savage());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let beast = g.add_card_to_hand(0, catalog::barkhide_mauler());
    cast(&mut g, 0, beast, None);
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Elvish Scrapper eats an artifact.
#[test]
fn elvish_scrapper_eats_an_artifact() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::elvish_scrapper());
    g.clear_sickness(elf);
    let slate = g.add_card_to_battlefield(1, catalog::slate_of_ancestry());
    activate(&mut g, 0, elf, 0, Some(Target::Permanent(slate)));
    assert!(g.battlefield_find(slate).is_none());
}

/// Grand Melee forces everyone into combat.
#[test]
fn grand_melee_forces_attacks_and_blocks() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grand_melee());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let cp = g.computed_permanent(bear).unwrap();
    assert!(cp.keywords.contains(&Keyword::MustAttack));
    assert!(cp.keywords.contains(&Keyword::MustBlock));
}

/// Krosan Groundshaker hands trample to any Beast.
#[test]
fn krosan_groundshaker_grants_trample() {
    let mut g = main_phase();
    let shaker = g.add_card_to_battlefield(0, catalog::krosan_groundshaker());
    let beast = g.add_card_to_battlefield(0, catalog::barkhide_mauler());
    activate(&mut g, 0, shaker, 0, Some(Target::Permanent(beast)));
    assert!(g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::Trample));
}

/// Daru Encampment pumps a Soldier off a colorless land.
#[test]
fn daru_encampment_pumps_a_soldier() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::daru_encampment());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    activate(&mut g, 0, land, 1, Some(Target::Permanent(soldier)));
    assert_eq!(g.computed_permanent(soldier).unwrap().power, 2);
}

/// Accursed Centaur eats a creature on the way in.
#[test]
fn accursed_centaur_eats_a_friend() {
    let mut g = main_phase();
    let friend = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let centaur = g.add_card_to_hand(0, catalog::accursed_centaur());
    cast(&mut g, 0, centaur, None);
    assert!(g.battlefield_find(friend).is_none(), "the bear, not the Centaur");
}

/// Screeching Buzzard takes a card with it.
#[test]
fn screeching_buzzard_strips_on_death() {
    let mut g = main_phase();
    let bird = g.add_card_to_battlefield(0, catalog::screeching_buzzard());
    g.add_card_to_hand(1, catalog::forest());
    let _ = g.remove_to_graveyard_with_triggers(bird);
    drain_stack(&mut g);
    assert!(g.players[1].hand.is_empty());
}

/// Dispersing Orb buys a bounce with a spare permanent.
#[test]
fn dispersing_orb_bounces_for_a_permanent() {
    let mut g = main_phase();
    let orb = g.add_card_to_battlefield(0, catalog::dispersing_orb());
    g.add_card_to_battlefield(0, catalog::forest());
    let prize = g.add_card_to_battlefield(1, catalog::silvos_rogue_elemental());
    activate(&mut g, 0, orb, 0, Some(Target::Permanent(prize)));
    assert!(g.battlefield_find(prize).is_none());
    assert_eq!(g.players[1].hand.len(), 1);
}

/// Crafty Pathmage waves small creatures through.
#[test]
fn crafty_pathmage_sneaks_the_small() {
    let mut g = main_phase();
    let mage = g.add_card_to_battlefield(0, catalog::crafty_pathmage());
    g.clear_sickness(mage);
    let runner = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    activate(&mut g, 0, mage, 0, Some(Target::Permanent(runner)));
    assert!(g.computed_permanent(runner).unwrap().keywords.contains(&Keyword::Unblockable));
}

/// Righteous Cause pays a life for every attack on the table.
#[test]
fn righteous_cause_gains_off_any_attack() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::righteous_cause());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(bear);
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: bear,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 21);
}

/// Silvos regenerates for a single green.
#[test]
fn silvos_regenerates_for_one_green() {
    let mut g = main_phase();
    let silvos = g.add_card_to_battlefield(0, catalog::silvos_rogue_elemental());
    activate(&mut g, 0, silvos, 0, None);
    assert_eq!(g.battlefield_find(silvos).unwrap().regeneration_shields, 1);
}

/// Barkhide Mauler cycles for a card.
#[test]
fn barkhide_mauler_cycles() {
    let mut g = main_phase();
    let mauler = g.add_card_to_hand(0, catalog::barkhide_mauler());
    g.add_card_to_library(0, catalog::forest());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::Cycle { card_id: mauler, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 1);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == mauler));
}

/// Inspirit untaps a blocker and makes it survive.
#[test]
fn inspirit_untaps_and_grows() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::inspirit());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
    assert_eq!(g.computed_permanent(bear).unwrap().toughness, 6);
}

/// Crowd Favorites taps a blocker out of the way.
#[test]
fn crowd_favorites_taps_a_blocker() {
    let mut g = main_phase();
    let crowd = g.add_card_to_battlefield(0, catalog::crowd_favorites());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, crowd, 0, Some(Target::Permanent(blocker)));
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

/// Defensive Maneuvers braces only the named tribe.
#[test]
fn defensive_maneuvers_braces_one_tribe() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    let beast = g.add_card_to_battlefield(0, catalog::barkhide_mauler());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    let spell = g.add_card_to_hand(0, catalog::defensive_maneuvers());
    cast(&mut g, 0, spell, None);
    assert_eq!(g.computed_permanent(elf).unwrap().toughness, 6);
    assert_eq!(g.computed_permanent(beast).unwrap().toughness, 4);
}

/// Goblin Burrows swings a Goblin off a colorless land.
#[test]
fn goblin_burrows_pumps_a_goblin() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(0, catalog::goblin_burrows());
    let goblin = g.add_card_to_battlefield(0, catalog::greasewrench_goblin());
    let before = g.computed_permanent(goblin).unwrap().power;
    activate(&mut g, 0, land, 1, Some(Target::Permanent(goblin)));
    assert_eq!(g.computed_permanent(goblin).unwrap().power, before + 2);
}

/// Symbiotic Beast leaves four Insects behind.
#[test]
fn symbiotic_beast_leaves_four_insects() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::symbiotic_beast());
    let _ = g.remove_to_graveyard_with_triggers(beast);
    drain_stack(&mut g);
    assert_eq!(g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(), 4);
}

/// Searing Flesh throws seven at a face.
#[test]
fn searing_flesh_burns_for_seven() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::searing_flesh());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, 13);
}

/// Rorix Bladewing attacks the turn it lands.
#[test]
fn rorix_bladewing_is_hasty() {
    let mut g = main_phase();
    let rorix = g.add_card_to_hand(0, catalog::rorix_bladewing());
    cast(&mut g, 0, rorix, None);
    let id = g.battlefield.iter().find(|c| c.definition.name == "Rorix Bladewing").unwrap().id;
    let cp = g.computed_permanent(id).unwrap();
    assert!(cp.keywords.contains(&Keyword::Haste) && cp.keywords.contains(&Keyword::Flying));
}

/// Disciple of Malice can't be touched by white.
#[test]
fn disciple_of_malice_dodges_white() {
    let mut g = main_phase();
    let disciple = g.add_card_to_battlefield(0, catalog::disciple_of_malice());
    assert!(
        g.computed_permanent(disciple)
            .unwrap()
            .keywords
            .contains(&Keyword::Protection(Color::White))
    );
}

/// Wirewood Elf taps for green.
#[test]
fn wirewood_elf_taps_for_green() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.clear_sickness(elf);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: elf,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    assert_eq!(g.players[0].mana_pool.amount(Color::Green), 1);
}

/// Counters ride Onslaught's tribal bodies the same as anywhere else.
#[test]
fn onslaught_bodies_take_counters() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.battlefield_find_mut(elf).unwrap().add_counters(CounterType::PlusOnePlusOne, 1);
    assert_eq!(g.computed_permanent(elf).unwrap().power, 2);
}

// ── Wave 2 ───────────────────────────────────────────────────────────────────

fn advance_to_attackers(g: &mut GameState) {
    while g.step != TurnStep::DeclareAttackers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
}

/// Attack player 1 with `attacker` and resolve the combat-damage triggers.
fn swing(g: &mut GameState, attacker: CardId) {
    advance_to_attackers(g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(g);
}

fn count_named(g: &GameState, controller: usize, name: &str) -> usize {
    g.battlefield.iter().filter(|c| c.controller == controller && c.definition.name == name).count()
}

/// Destroy a battlefield permanent, firing its dies triggers.
fn kill(g: &mut GameState, id: CardId) {
    use crabomination::effect::{Effect, Selector};
    let ctl = g.battlefield_find(id).unwrap().controller;
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        id,
        ctl,
        Some(Target::Permanent(id)),
    );
    let ev = g.resolve_effect(&Effect::Destroy { what: Selector::Target(0) }, &ctx).unwrap();
    g.dispatch_triggers_for_events(&ev);
    drain_stack(g);
}

/// The Morph commons cast face down as 2/2s and flip up for their morph cost.
#[test]
fn ons_morph_bodies_cast_down_and_flip_up() {
    for (make, name, up) in [
        (catalog::battering_craghorn as fn() -> _, "Battering Craghorn", (3, 1)),
        (catalog::crude_rampart, "Crude Rampart", (4, 5)),
        (catalog::spined_basher, "Spined Basher", (3, 1)),
        (catalog::spitting_gourna, "Spitting Gourna", (3, 4)),
        (catalog::krosan_colossus, "Krosan Colossus", (9, 9)),
        (catalog::towering_baloth, "Towering Baloth", (7, 6)),
        (catalog::treespring_lorian, "Treespring Lorian", (5, 4)),
    ] {
        let mut g = main_phase();
        let id = g.add_card_to_hand(0, make());
        mana(&mut g, 0);
        g.perform_action(GameAction::CastFaceDown { card_id: id }).expect("cast face down");
        drain_stack(&mut g);
        let fd = g.battlefield_find(id).expect("entered");
        assert!(fd.face_down, "{name} entered face down");
        assert_eq!((fd.power(), fd.toughness()), (2, 2), "{name} is a 2/2 face down");
        mana(&mut g, 0);
        g.perform_action(GameAction::TurnFaceUp { card_id: id }).expect("turn up");
        let cp = g.computed_permanent(id).unwrap();
        assert_eq!((cp.power, cp.toughness), up, "{name} flips to its printed size");
    }
}

/// The protection-from-a-tribe Morph creatures carry their printed protection.
#[test]
fn ons_tribal_protection_bodies() {
    let mut g = main_phase();
    for (make, tribe) in [
        (catalog::fallen_cleric as fn() -> _, CreatureType::Cleric),
        (catalog::foothill_guide, CreatureType::Goblin),
        (catalog::riptide_biologist, CreatureType::Beast),
    ] {
        let id = g.add_card_to_battlefield(0, make());
        assert!(
            g.computed_permanent(id)
                .unwrap()
                .keywords
                .contains(&Keyword::ProtectionFromCreatureType(tribe)),
            "protection from {tribe:?}"
        );
    }
}

/// Ascending Aven can't block a ground creature.
#[test]
fn ascending_aven_blocks_only_fliers() {
    let mut g = main_phase();
    let aven = g.add_card_to_battlefield(1, catalog::ascending_aven());
    let ground = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(ground);
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: ground,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(aven, ground)])).is_err(),
        "the Aven can only block fliers"
    );
}

/// Grinning Demon bleeds its controller for 2 each upkeep.
#[test]
fn grinning_demon_upkeep_drain() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grinning_demon());
    let before = g.players[0].life;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, before - 2);
}

/// The combat-damage Morph creatures each fire their printed rider.
#[test]
fn ons_combat_damage_riders() {
    // Headhunter strips a card; Silent Specter strips two.
    for (make, discarded) in
        [(catalog::headhunter as fn() -> _, 1), (catalog::silent_specter, 2)]
    {
        let mut g = main_phase();
        let hitter = g.add_card_to_battlefield(0, make());
        g.clear_sickness(hitter);
        for _ in 0..3 {
            g.add_card_to_hand(1, catalog::forest());
        }
        let before = g.players[1].hand.len();
        swing(&mut g, hitter);
        assert_eq!(g.players[1].hand.len(), before - discarded, "discard on connect");
    }
    // Cabal Executioner edicts the defender.
    let mut g = main_phase();
    let exec = g.add_card_to_battlefield(0, catalog::cabal_executioner());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(exec);
    swing(&mut g, exec);
    assert!(g.battlefield_find(victim).is_none(), "defender sacrificed a creature");
    // Hystrodon draws.
    let mut g = main_phase();
    let hys = g.add_card_to_battlefield(0, catalog::hystrodon());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.clear_sickness(hys);
    let before = g.players[0].hand.len();
    swing(&mut g, hys);
    assert_eq!(g.players[0].hand.len(), before + 1, "drew on connect");
}

/// Skirk Commando shoots a creature the damaged player controls.
#[test]
fn skirk_commando_pings_a_defender_creature() {
    let mut g = main_phase();
    let commando = g.add_card_to_battlefield(0, catalog::skirk_commando());
    let target = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.clear_sickness(commando);
    swing(&mut g, commando);
    assert!(g.battlefield_find(target).is_none(), "2 damage killed the 2/2");
}

/// Dawning Purist cracks an enchantment on connect.
#[test]
fn dawning_purist_destroys_an_enchantment() {
    let mut g = main_phase();
    let purist = g.add_card_to_battlefield(0, catalog::dawning_purist());
    let ench = g.add_card_to_battlefield(1, catalog::dragon_roost());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.clear_sickness(purist);
    swing(&mut g, purist);
    assert!(g.battlefield_find(ench).is_none(), "enchantment destroyed");
}

/// The Avatar cycle sizes itself off every creature of its tribe in play —
/// Soulless One also counts Zombie cards in graveyards.
#[test]
fn ons_avatar_cycle_counts_its_tribe() {
    let mut g = main_phase();
    let avatar = g.add_card_to_battlefield(0, catalog::heedless_one());
    assert_eq!(g.computed_permanent(avatar).unwrap().power, 1, "counts itself");
    g.add_card_to_battlefield(1, catalog::wirewood_elf()); // an opponent's Elf counts too
    assert_eq!(g.computed_permanent(avatar).unwrap().power, 2);

    let mut g = main_phase();
    let soulless = g.add_card_to_battlefield(0, catalog::soulless_one());
    g.add_card_to_graveyard(1, catalog::gluttonous_zombie());
    assert_eq!(
        g.computed_permanent(soulless).unwrap().power,
        2,
        "one in play plus one in a graveyard"
    );
}

/// Doubtless One gains you life equal to the damage it deals.
#[test]
fn doubtless_one_drains_on_damage() {
    let mut g = main_phase();
    let one = g.add_card_to_battlefield(0, catalog::doubtless_one());
    g.add_card_to_battlefield(0, catalog::ancestors_prophet()); // a second Cleric
    g.clear_sickness(one);
    let life = g.players[0].life;
    swing(&mut g, one);
    assert_eq!(g.players[0].life, life + 2, "gained life equal to its power");
}

/// The tap-a-tribe activations pay by tapping the named creatures.
#[test]
fn ancestors_prophet_taps_five_clerics_for_ten_life() {
    let mut g = main_phase();
    let prophet = g.add_card_to_battlefield(0, catalog::ancestors_prophet());
    g.clear_sickness(prophet);
    let mut clerics = vec![prophet];
    for _ in 0..4 {
        let c = g.add_card_to_battlefield(0, catalog::rotlung_reanimator());
        g.clear_sickness(c);
        clerics.push(c);
    }
    let life = g.players[0].life;
    activate(&mut g, 0, prophet, 0, None);
    assert_eq!(g.players[0].life, life + 10);
    assert!(clerics.iter().all(|&c| g.battlefield_find(c).unwrap().tapped), "all five tapped");
}

/// Aphetto Grifter taps two Wizards to tap a permanent.
#[test]
fn aphetto_grifter_taps_a_permanent() {
    let mut g = main_phase();
    let grifter = g.add_card_to_battlefield(0, catalog::aphetto_grifter());
    let helper = g.add_card_to_battlefield(0, catalog::nameless_one());
    g.clear_sickness(grifter);
    g.clear_sickness(helper);
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, grifter, 0, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).unwrap().tapped);
}

/// Birchlore Rangers turns two Elves into a mana of any color.
#[test]
fn birchlore_rangers_taps_elves_for_mana() {
    let mut g = main_phase();
    let rangers = g.add_card_to_battlefield(0, catalog::birchlore_rangers());
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.clear_sickness(rangers);
    g.clear_sickness(elf);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: rangers,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap two Elves");
    assert_eq!(g.players[0].mana_pool.total(), 1);
}

/// Gravespawn Sovereign reanimates out of any graveyard.
#[test]
fn gravespawn_sovereign_reanimates() {
    let mut g = main_phase();
    let sov = g.add_card_to_battlefield(0, catalog::gravespawn_sovereign());
    g.clear_sickness(sov);
    for _ in 0..4 {
        let z = g.add_card_to_battlefield(0, catalog::gluttonous_zombie());
        g.clear_sickness(z);
    }
    let corpse_id = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    activate(&mut g, 0, sov, 0, Some(Target::Permanent(corpse_id)));
    assert_eq!(
        g.battlefield_find(corpse_id).map(|c| c.controller),
        Some(0),
        "reanimated under your control"
    );
}

/// Gangrenous Goliath buys itself back by tapping three Clerics.
#[test]
fn gangrenous_goliath_returns_from_the_graveyard() {
    let mut g = main_phase();
    for _ in 0..3 {
        let c = g.add_card_to_battlefield(0, catalog::rotlung_reanimator());
        g.clear_sickness(c);
    }
    let id = g.add_card_to_graveyard(0, catalog::gangrenous_goliath());
    activate(&mut g, 0, id, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "back in hand");
}

/// Skirk Fire Marshal's five-Goblin tap wipes the board and both players.
#[test]
fn skirk_fire_marshal_burns_everything() {
    let mut g = main_phase();
    let marshal = g.add_card_to_battlefield(0, catalog::skirk_fire_marshal());
    g.clear_sickness(marshal);
    for _ in 0..4 {
        let gob = g.add_card_to_battlefield(0, catalog::reckless_one());
        g.clear_sickness(gob);
    }
    let bystander = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let life = g.players[1].life;
    activate(&mut g, 0, marshal, 0, None);
    assert!(g.battlefield_find(bystander).is_none(), "creatures took 10");
    assert_eq!(g.players[1].life, life - 10, "players took 10");
    assert!(g.battlefield_find(marshal).is_some(), "protection from red spared the Marshal");
}

/// Wirewood Pride and Feeding Frenzy scale off every creature of their tribe.
#[test]
fn ons_tribal_count_pumps() {
    let mut g = main_phase();
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.add_card_to_battlefield(1, catalog::elvish_pioneer());
    let pride = g.add_card_to_hand(0, catalog::wirewood_pride());
    cast(&mut g, 0, pride, Some(Target::Permanent(elf)));
    assert_eq!(g.computed_permanent(elf).unwrap().power, 3, "1/1 + two Elves");

    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_battlefield(0, catalog::gluttonous_zombie());
    g.add_card_to_battlefield(0, catalog::spined_basher());
    let frenzy = g.add_card_to_hand(0, catalog::feeding_frenzy());
    cast(&mut g, 0, frenzy, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "-2/-2 killed the 2/2");
}

/// Profane Prayers drains for the number of Clerics on the battlefield.
#[test]
fn profane_prayers_drains_by_cleric_count() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::rotlung_reanimator());
    g.add_card_to_battlefield(1, catalog::foothill_guide());
    let spell = g.add_card_to_hand(0, catalog::profane_prayers());
    let (mine, theirs) = (g.players[0].life, g.players[1].life);
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, theirs - 2);
    assert_eq!(g.players[0].life, mine + 2);
}

/// Thunder of Hooves spares fliers and hits both players.
#[test]
fn thunder_of_hooves_spares_fliers() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::snapping_thragg()); // a Beast
    let ground = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let flier = g.add_card_to_battlefield(1, catalog::screaming_seahawk());
    let spell = g.add_card_to_hand(0, catalog::thunder_of_hooves());
    let life = g.players[1].life;
    cast(&mut g, 0, spell, None);
    assert_eq!(g.battlefield_find(ground).unwrap().damage, 1, "ground creatures take X");
    assert_eq!(g.battlefield_find(flier).unwrap().damage, 0, "fliers are spared");
    assert_eq!(g.players[1].life, life - 1, "one Beast → 1 damage to each player");
}

/// Elvish Pioneer drops a basic land onto the battlefield tapped.
#[test]
fn elvish_pioneer_ramps_a_basic() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::forest());
    let pioneer = g.add_card_to_hand(0, catalog::elvish_pioneer());
    cast(&mut g, 0, pioneer, None);
    assert!(g.battlefield_find(land).is_some_and(|c| c.tapped), "basic entered tapped");
}

/// Doomed Necromancer sacrifices itself to reanimate.
#[test]
fn doomed_necromancer_reanimates() {
    let mut g = main_phase();
    let necro = g.add_card_to_battlefield(0, catalog::doomed_necromancer());
    g.clear_sickness(necro);
    let id = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    activate(&mut g, 0, necro, 0, Some(Target::Permanent(id)));
    assert!(g.battlefield_find(id).is_some(), "reanimated");
    assert!(g.battlefield_find(necro).is_none(), "the Necromancer sacrificed itself");
}

/// Rotlung Reanimator leaves a Zombie behind for every dying Cleric — itself
/// included.
#[test]
fn rotlung_reanimator_mints_on_cleric_deaths() {
    let mut g = main_phase();
    let rot = g.add_card_to_battlefield(0, catalog::rotlung_reanimator());
    let other = g.add_card_to_battlefield(0, catalog::foothill_guide());
    kill(&mut g, other);
    assert_eq!(count_named(&g, 0, "Zombie"), 1, "another Cleric died");
    kill(&mut g, rot);
    assert_eq!(count_named(&g, 0, "Zombie"), 2, "and its own death counts");
}

/// Aphetto Vulture stacks a Zombie back on top of your library when it dies.
#[test]
fn aphetto_vulture_restocks_a_zombie() {
    let mut g = main_phase();
    let vulture = g.add_card_to_battlefield(0, catalog::aphetto_vulture());
    let zid = g.add_card_to_graveyard(0, catalog::gluttonous_zombie());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    kill(&mut g, vulture);
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(zid), "Zombie on top");
}

/// The "search your library for another copy" ETB cycle fetches its twin.
#[test]
fn ons_sibling_fetchers_find_their_twin() {
    for make in [
        catalog::daru_cavalier as fn() -> _,
        catalog::avarax,
        catalog::embermage_goblin,
        catalog::screaming_seahawk,
    ] {
        let mut g = main_phase();
        let twin = g.add_card_to_library(0, make());
        let id = g.add_card_to_hand(0, make());
        g.decider = Box::new(ScriptedDecider::new([
            DecisionAnswer::Bool(true),
            DecisionAnswer::Search(Some(twin)),
        ]));
        cast(&mut g, 0, id, None);
        assert!(g.players[0].hand.iter().any(|c| c.id == twin), "fetched the twin");
    }
}

/// Stag Beetle enters sized by the rest of the board.
#[test]
fn stag_beetle_counts_the_other_creatures() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let beetle = g.add_card_to_hand(0, catalog::stag_beetle());
    cast(&mut g, 0, beetle, None);
    assert_eq!(g.computed_permanent(beetle).unwrap().power, 2, "two other creatures");
}

/// Unified Strike exiles an attacker only while your Soldier count covers it.
#[test]
fn unified_strike_gates_on_soldier_count() {
    let mut g = main_phase();
    let soldier = g.add_card_to_battlefield(1, catalog::ascending_aven()); // 3/2 Bird Soldier
    g.clear_sickness(soldier);
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: soldier,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    let strike = g.add_card_to_hand(0, catalog::unified_strike());
    cast(&mut g, 0, strike, Some(Target::Permanent(soldier)));
    assert!(g.battlefield_find(soldier).is_some(), "power 3 > the lone Soldier: no exile");

    g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    g.add_card_to_battlefield(0, catalog::catapult_squad());
    let strike = g.add_card_to_hand(0, catalog::unified_strike());
    cast(&mut g, 0, strike, Some(Target::Permanent(soldier)));
    assert!(g.exile.iter().any(|c| c.id == soldier), "3 Soldiers now cover its power");
}

/// Ixidor's Will taxes by two per Wizard on the battlefield.
#[test]
fn ixidors_will_taxes_per_wizard() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::aphetto_grifter());
    g.add_card_to_battlefield(1, catalog::nameless_one());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bolt");
    let will = g.add_card_to_hand(0, catalog::ixidors_will());
    g.players[1].mana_pool.empty();
    cast(&mut g, 0, will, Some(Target::Permanent(bolt)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "unpaid → countered");
}

/// Blackmail lets the caster pick the discard from three revealed cards.
#[test]
fn blackmail_picks_the_discard() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::blackmail());
    let before = g.players[1].hand.len();
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), before - 1);
}

/// Dragon Roost mints a 5/5 flier.
#[test]
fn dragon_roost_makes_a_dragon() {
    let mut g = main_phase();
    let roost = g.add_card_to_battlefield(0, catalog::dragon_roost());
    activate(&mut g, 0, roost, 0, None);
    assert_eq!(count_named(&g, 0, "Dragon"), 1);
}

/// Starstorm sweeps for X and can be cycled away.
#[test]
fn starstorm_sweeps_for_x() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    let storm = g.add_card_to_hand(0, catalog::starstorm());
    cast_x(&mut g, 0, storm, None, Some(3));
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 died");
    assert!(g.battlefield_find(big).is_some(), "the 9/9 survived");
}

/// Pinpoint Avalanche's 4 damage ignores prevention shields.
#[test]
fn pinpoint_avalanche_ignores_prevention() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    {
        use crabomination::effect::{Effect, Selector, Value};
        let ctx = crabomination::game::effects::EffectContext::for_ability(
            victim,
            1,
            Some(Target::Permanent(victim)),
        );
        g.resolve_effect(
            &Effect::PreventNextDamage { target: Selector::Target(0), amount: Value::Const(4) },
            &ctx,
        )
        .unwrap();
    }
    let spell = g.add_card_to_hand(0, catalog::pinpoint_avalanche());
    cast(&mut g, 0, spell, Some(Target::Permanent(victim)));
    assert_eq!(
        g.battlefield_find(victim).unwrap().damage,
        4,
        "the shield didn't stop it"
    );
}

/// Unholy Grotto recycles a Zombie from the graveyard.
#[test]
fn unholy_grotto_restocks_a_zombie() {
    let mut g = main_phase();
    let grotto = g.add_card_to_battlefield(0, catalog::unholy_grotto());
    let zid = g.add_card_to_graveyard(0, catalog::gluttonous_zombie());
    activate(&mut g, 0, grotto, 1, Some(Target::Permanent(zid)));
    assert_eq!(g.players[0].library.last().map(|c| c.id), Some(zid));
}

/// Seaside Haven cashes a Bird in for a card.
#[test]
fn seaside_haven_draws_off_a_bird() {
    let mut g = main_phase();
    let haven = g.add_card_to_battlefield(0, catalog::seaside_haven());
    let bird = g.add_card_to_battlefield(0, catalog::screaming_seahawk());
    g.add_card_to_library(0, catalog::forest());
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, haven, 1, None);
    assert_eq!(g.players[0].hand.len(), hand + 1);
    assert!(g.battlefield_find(bird).is_none(), "the Bird was sacrificed");
}

/// Contested Cliffs sics a Beast on an opposing creature.
#[test]
fn contested_cliffs_fights() {
    let mut g = main_phase();
    let cliffs = g.add_card_to_battlefield(0, catalog::contested_cliffs());
    let beast = g.add_card_to_battlefield(0, catalog::snapping_thragg()); // 3/3 Beast
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.priority.player_with_priority = 0;
    mana(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cliffs,
        ability_index: 1,
        target: Some(Target::Permanent(beast)),
        additional_targets: vec![Target::Permanent(victim)],
        mode: None,
        x_value: None,
    })
    .expect("fight");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none(), "the 2/2 lost the fight");
    assert_eq!(g.battlefield_find(beast).unwrap().damage, 2);
}

/// Grand Coliseum enters tapped and pings you for colored mana.
#[test]
fn grand_coliseum_hurts_for_color() {
    let mut g = main_phase();
    let land = g.add_card_to_hand(0, catalog::grand_coliseum());
    g.perform_action(GameAction::PlayLand(land)).expect("play land");
    drain_stack(&mut g);
    assert!(g.battlefield_find(land).unwrap().tapped, "enters tapped");
    g.battlefield_find_mut(land).unwrap().tapped = false;
    let life = g.players[0].life;
    activate(&mut g, 0, land, 1, None);
    assert_eq!(g.players[0].life, life - 1, "colored mana costs a life");
}

/// Starlit Sanctum trades a Cleric for life or a drain.
#[test]
fn starlit_sanctum_cashes_in_clerics() {
    let mut g = main_phase();
    let sanctum = g.add_card_to_battlefield(0, catalog::starlit_sanctum());
    g.add_card_to_battlefield(0, catalog::rotlung_reanimator()); // 2/2 Cleric
    let life = g.players[0].life;
    activate(&mut g, 0, sanctum, 1, None);
    assert_eq!(g.players[0].life, life + 2, "gained its toughness");

    let mut g = main_phase();
    let sanctum = g.add_card_to_battlefield(0, catalog::starlit_sanctum());
    g.add_card_to_battlefield(0, catalog::rotlung_reanimator());
    let theirs = g.players[1].life;
    activate(&mut g, 0, sanctum, 2, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, theirs - 2, "drained for its power");
}

/// Jareth grows to 11/14 when it blocks.
#[test]
fn jareth_grows_on_block() {
    let mut g = main_phase();
    let jareth = g.add_card_to_battlefield(0, catalog::jareth_leonine_titan());
    let attacker = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(jareth, attacker)])).expect("block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(jareth).unwrap();
    assert_eq!((cp.power, cp.toughness), (11, 14), "+7/+7 on block");
}

/// Sage Aven and Aven Fateshaper both stack the top four.
#[test]
fn ons_top_four_lookers() {
    let mut g = main_phase();
    for _ in 0..6 {
        g.add_card_to_library(0, catalog::forest());
    }
    let aven = g.add_card_to_hand(0, catalog::sage_aven());
    let lib = g.players[0].library.len();
    cast(&mut g, 0, aven, None);
    assert_eq!(g.players[0].library.len(), lib, "looking doesn't move cards");
    let shaper = g.add_card_to_battlefield(0, catalog::aven_fateshaper());
    activate(&mut g, 0, shaper, 0, None);
    assert_eq!(g.players[0].library.len(), lib);
}

/// Discombobulate counters and then rearranges.
#[test]
fn discombobulate_counters() {
    let mut g = main_phase();
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::forest());
    }
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bolt");
    let counter = g.add_card_to_hand(0, catalog::discombobulate());
    cast(&mut g, 0, counter, Some(Target::Permanent(bolt)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "countered");
}

/// The remaining wave-2 bodies ship with their printed keywords.
#[test]
fn ons_wave2_keyword_bodies() {
    let mut g = main_phase();
    let murkdiver = g.add_card_to_battlefield(0, catalog::anurid_murkdiver());
    assert!(
        g.computed_permanent(murkdiver)
            .unwrap()
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Swamp))
    );
    let zombie = g.add_card_to_battlefield(0, catalog::gluttonous_zombie());
    assert!(g.computed_permanent(zombie).unwrap().keywords.contains(&Keyword::Fear));
    let bomber = g.add_card_to_battlefield(0, catalog::dive_bomber());
    assert!(g.computed_permanent(bomber).unwrap().keywords.contains(&Keyword::Flying));
}

/// Dive Bomber sacrifices itself to shoot a blocker.
#[test]
fn dive_bomber_shoots_a_blocker() {
    let mut g = main_phase();
    let bomber = g.add_card_to_battlefield(0, catalog::dive_bomber());
    g.clear_sickness(bomber);
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    activate(&mut g, 0, bomber, 0, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none(), "the attacker took 2");
}

/// Grassland Crusader, Elvish Pathcutter and Snarling Undorak each pump their
/// printed tribe.
#[test]
fn ons_tribal_pumpers() {
    let mut g = main_phase();
    let crusader = g.add_card_to_battlefield(0, catalog::grassland_crusader());
    g.clear_sickness(crusader);
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    activate(&mut g, 0, crusader, 0, Some(Target::Permanent(elf)));
    assert_eq!(g.computed_permanent(elf).unwrap().power, 3);

    let mut g = main_phase();
    let cutter = g.add_card_to_battlefield(0, catalog::elvish_pathcutter());
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    activate(&mut g, 0, cutter, 0, Some(Target::Permanent(elf)));
    assert!(
        g.computed_permanent(elf)
            .unwrap()
            .keywords
            .contains(&Keyword::Landwalk(crabomination::card::LandType::Forest))
    );

    let mut g = main_phase();
    let undorak = g.add_card_to_battlefield(0, catalog::snarling_undorak());
    activate(&mut g, 0, undorak, 0, Some(Target::Permanent(undorak)));
    assert_eq!(g.computed_permanent(undorak).unwrap().power, 4);
}

/// Spurred Wolverine taps two Beasts to grant first strike.
#[test]
fn spurred_wolverine_grants_first_strike() {
    let mut g = main_phase();
    let wolverine = g.add_card_to_battlefield(0, catalog::spurred_wolverine());
    let beast = g.add_card_to_battlefield(0, catalog::snapping_thragg());
    g.clear_sickness(wolverine);
    g.clear_sickness(beast);
    activate(&mut g, 0, wolverine, 0, Some(Target::Permanent(beast)));
    assert!(g.computed_permanent(beast).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Catapult Squad taps two Soldiers to shoot a combatant.
#[test]
fn catapult_squad_shoots_an_attacker() {
    let mut g = main_phase();
    let squad = g.add_card_to_battlefield(0, catalog::catapult_squad());
    let soldier = g.add_card_to_battlefield(0, catalog::gustcloak_runner());
    g.clear_sickness(squad);
    g.clear_sickness(soldier);
    let attacker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    activate(&mut g, 0, squad, 0, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none(), "the attacker took 2");
}

// ── Wave 3 ───────────────────────────────────────────────────────────────────

/// A Crown pumps its host, then spreads the same bonus to the host's whole
/// tribe when sacrificed.
#[test]
fn crown_of_fury_spreads_to_the_tribe() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::wirewood_elf()); // 1/2 Elf
    let kin = g.add_card_to_battlefield(0, catalog::elvish_pioneer()); // a 1/1 Elf
    let outsider = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let crown = g.add_card_to_hand(0, catalog::crown_of_fury());
    cast(&mut g, 0, crown, Some(Target::Permanent(host)));
    assert_eq!(g.computed_permanent(host).unwrap().power, 2, "+1/+0 on the host");
    activate(&mut g, 0, crown, 0, None);
    assert_eq!(g.computed_permanent(kin).unwrap().power, 2, "the tribe got +1/+0");
    assert!(
        g.computed_permanent(kin).unwrap().keywords.contains(&Keyword::FirstStrike),
        "and first strike"
    );
    assert_eq!(g.computed_permanent(outsider).unwrap().power, 2, "off-tribe untouched");
}

/// The other Crowns carry their printed host bonus.
#[test]
fn ons_crown_cycle_host_bonuses() {
    for (make, name, pt, kw) in [
        (
            catalog::crown_of_ascension as fn() -> _,
            "Crown of Ascension",
            (1, 2),
            Some(Keyword::Flying),
        ),
        (catalog::crown_of_awe, "Crown of Awe", (1, 2), Some(Keyword::Protection(Color::Black))),
        (catalog::crown_of_suspicion, "Crown of Suspicion", (3, 1), None),
        (catalog::crown_of_vigor, "Crown of Vigor", (2, 3), None),
    ] {
        let mut g = main_phase();
        let host = g.add_card_to_battlefield(0, catalog::wirewood_elf()); // 1/1
        let crown = g.add_card_to_hand(0, make());
        cast(&mut g, 0, crown, Some(Target::Permanent(host)));
        let cp = g.computed_permanent(host).unwrap();
        assert_eq!((cp.power, cp.toughness), pt, "{name} bonus");
        if let Some(k) = kw {
            assert!(cp.keywords.contains(&k), "{name} keyword");
        }
    }
}

/// A Courier lends +2/+2 and its keyword for as long as it stays tapped.
#[test]
fn everglove_courier_lends_while_tapped() {
    let mut g = main_phase();
    let courier = g.add_card_to_battlefield(0, catalog::everglove_courier());
    let elf = g.add_card_to_battlefield(0, catalog::wirewood_elf());
    g.clear_sickness(courier);
    activate(&mut g, 0, courier, 0, Some(Target::Permanent(elf)));
    let cp = g.computed_permanent(elf).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 4), "+2/+2 while the Courier stays tapped");
    assert!(cp.keywords.contains(&Keyword::Trample));
    g.battlefield_find_mut(courier).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.computed_permanent(elf).unwrap().power, 1, "the bonus ends when it untaps");
}

/// Every Courier carries "you may choose not to untap" and its tribe's keyword.
#[test]
fn ons_courier_cycle_shapes() {
    for (make, kw) in [
        (catalog::flamestick_courier as fn() -> _, Keyword::Haste),
        (catalog::frightshroud_courier, Keyword::Fear),
        (catalog::ghosthelm_courier, Keyword::Shroud),
        (catalog::pearlspear_courier, Keyword::Vigilance),
    ] {
        let def = make();
        assert!(def.keywords.contains(&Keyword::MayChooseNotToUntap), "{}", def.name);
        let granted = &def.activated_abilities[0].effect;
        assert!(format!("{granted:?}").contains(&format!("{kw:?}")), "{} grants {kw:?}", def.name);
    }
}

/// The Mistforms become the creature type of your choice.
#[test]
fn mistform_dreamer_becomes_a_chosen_type() {
    let mut g = main_phase();
    let dreamer = g.add_card_to_battlefield(0, catalog::mistform_dreamer());
    g.decider =
        Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Goblin)]));
    activate(&mut g, 0, dreamer, 0, None);
    assert!(
        g.computed_permanent(dreamer)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&CreatureType::Goblin)
    );
}

/// Mistform Wall keeps defender only while it's still a Wall.
#[test]
fn mistform_wall_loses_defender_when_retyped() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::mistform_wall());
    assert!(g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Bird)]));
    activate(&mut g, 0, wall, 0, None);
    assert!(
        !g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender),
        "no longer a Wall, no longer a defender"
    );
}

/// Imagecrafter and Mistform Mutant retype someone else; Standardize retypes
/// the whole board.
#[test]
fn ons_retypers_hit_other_creatures() {
    let mut g = main_phase();
    let crafter = g.add_card_to_battlefield(0, catalog::imagecrafter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crafter);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Elf)]));
    activate(&mut g, 0, crafter, 0, Some(Target::Permanent(bear)));
    assert!(
        g.computed_permanent(bear).unwrap().subtypes.creature_types.contains(&CreatureType::Elf)
    );

    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::standardize());
    g.decider =
        Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Zombie)]));
    cast(&mut g, 0, spell, None);
    for id in [mine, theirs] {
        assert!(
            g.computed_permanent(id)
                .unwrap()
                .subtypes
                .creature_types
                .contains(&CreatureType::Zombie)
        );
    }
}

/// Mistform Mask retypes the creature it enchants.
#[test]
fn mistform_mask_retypes_its_host() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let mask = g.add_card_to_hand(0, catalog::mistform_mask());
    cast(&mut g, 0, mask, Some(Target::Permanent(host)));
    g.decider =
        Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Wizard)]));
    activate(&mut g, 0, mask, 0, None);
    assert!(
        g.computed_permanent(host)
            .unwrap()
            .subtypes
            .creature_types
            .contains(&CreatureType::Wizard)
    );
}

/// Solar Blast burns for 3 on cast and 1 on the cycle.
#[test]
fn solar_blast_casts_big_and_cycles_small() {
    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::solar_blast());
    let life = g.players[1].life;
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);

    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::solar_blast());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    mana(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: spell, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 1, "the cycle trigger pinged for 1");
}

/// Slice and Dice sweeps for 4, or for 1 when cycled.
#[test]
fn slice_and_dice_sweeps() {
    let mut g = main_phase();
    let small = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let big = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    let spell = g.add_card_to_hand(0, catalog::slice_and_dice());
    cast(&mut g, 0, spell, None);
    assert!(g.battlefield_find(small).is_none());
    assert_eq!(g.battlefield_find(big).unwrap().damage, 4);
}

/// Choking Tethers taps up to four creatures.
#[test]
fn choking_tethers_taps_up_to_four() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::choking_tethers());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(a).unwrap().tapped && g.battlefield_find(b).unwrap().tapped);
}

/// Complicate taxes {3}; the cycle trigger taxes {1}.
#[test]
fn complicate_taxes_three() {
    let mut g = main_phase();
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bolt");
    g.players[1].mana_pool.empty();
    let comp = g.add_card_to_hand(0, catalog::complicate());
    cast(&mut g, 0, comp, Some(Target::Permanent(bolt)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "unpaid → countered");
}

/// Death Pulse and Primal Boost swing P/T by their printed amounts.
#[test]
fn ons_cycling_pump_spells() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::snapping_thragg()); // 3/3
    let pulse = g.add_card_to_hand(0, catalog::death_pulse());
    cast(&mut g, 0, pulse, Some(Target::Permanent(victim)));
    assert!(g.battlefield_find(victim).is_none(), "-4/-4 killed the 3/3");

    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let boost = g.add_card_to_hand(0, catalog::primal_boost());
    cast(&mut g, 0, boost, Some(Target::Permanent(bear)));
    assert_eq!(g.computed_permanent(bear).unwrap().power, 6);
}

/// Dirge of Dread gives the whole board fear.
#[test]
fn dirge_of_dread_grants_fear() {
    let mut g = main_phase();
    let mine = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::dirge_of_dread());
    cast(&mut g, 0, spell, None);
    for id in [mine, theirs] {
        assert!(g.computed_permanent(id).unwrap().keywords.contains(&Keyword::Fear));
    }
}

/// Sunfire Balm shields the next 4 damage.
#[test]
fn sunfire_balm_prevents_four() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::krosan_colossus());
    let balm = g.add_card_to_hand(0, catalog::sunfire_balm());
    cast(&mut g, 0, balm, Some(Target::Permanent(bear)));
    let blast = g.add_card_to_hand(0, catalog::solar_blast());
    cast(&mut g, 0, blast, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 0, "all 3 prevented");
}

/// Akroma's Blessing gives your team protection from a chosen color.
#[test]
fn akromas_blessing_protects_the_team() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::akromas_blessing());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Color(Color::Red)]));
    cast(&mut g, 0, spell, None);
    assert!(
        g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Protection(Color::Red))
    );
}

/// Aura Extraction stacks an enchantment on its owner's library.
#[test]
fn aura_extraction_stacks_an_enchantment() {
    let mut g = main_phase();
    let roost = g.add_card_to_battlefield(1, catalog::dragon_roost());
    let spell = g.add_card_to_hand(0, catalog::aura_extraction());
    cast(&mut g, 0, spell, Some(Target::Permanent(roost)));
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(roost));
}

/// Fade from Memory exiles a graveyard card.
#[test]
fn fade_from_memory_exiles() {
    let mut g = main_phase();
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::fade_from_memory());
    cast(&mut g, 0, spell, Some(Target::Permanent(corpse)));
    assert!(g.exile.iter().any(|c| c.id == corpse));
}

/// Mage's Guile hands out shroud.
#[test]
fn mages_guile_grants_shroud() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::mages_guile());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Essence Fracture bounces two creatures.
#[test]
fn essence_fracture_bounces_two() {
    let mut g = main_phase();
    let a = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let b = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::essence_fracture());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(a)),
        additional_targets: vec![Target::Permanent(b)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].hand.len(), 2, "both went home");
}

/// Improvised Armor is a +2/+5 Aura.
#[test]
fn improvised_armor_pumps() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::improvised_armor());
    cast(&mut g, 0, aura, Some(Target::Permanent(bear)));
    let cp = g.computed_permanent(bear).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 7));
}

/// Slipstream Eel can't attack without an Island across the table.
#[test]
fn slipstream_eel_needs_an_island() {
    let mut g = main_phase();
    let eel = g.add_card_to_battlefield(0, catalog::slipstream_eel());
    g.clear_sickness(eel);
    advance_to_attackers(&mut g);
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: eel,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "no Island → no attack"
    );
    g.add_card_to_battlefield(1, catalog::island());
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: eel,
        target: AttackTarget::Player(1),
    }]))
    .expect("Island across the table → attack");
}

/// Undead Gladiator buys itself back, but only during your upkeep.
#[test]
fn undead_gladiator_returns_during_upkeep() {
    let mut g = main_phase();
    let id = g.add_card_to_graveyard(0, catalog::undead_gladiator());
    g.add_card_to_hand(0, catalog::forest());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::ActivateAbility {
            card_id: id,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .is_err(),
        "main phase is too late"
    );
    g.step = TurnStep::Upkeep;
    activate(&mut g, 0, id, 0, None);
    assert!(g.players[0].hand.iter().any(|c| c.id == id), "back in hand at upkeep");
}

// ── Wave 4 ───────────────────────────────────────────────────────────────────

/// CR 506.4 — a Gustcloak untaps and leaves combat when it's blocked, so no
/// combat damage is exchanged.
#[test]
fn gustcloak_sentinel_slips_its_blocker() {
    let mut g = main_phase();
    let sentinel = g.add_card_to_battlefield(0, catalog::gustcloak_sentinel());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(sentinel);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: sentinel,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(g.battlefield_find(sentinel).unwrap().tapped, "attacking taps it");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, sentinel)])).expect("block");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(sentinel).unwrap().tapped, "untapped by the trigger");
    assert!(!g.attacking.iter().any(|a| a.attacker == sentinel), "removed from combat");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 0, "no damage was exchanged");
}

/// Gustcloak Savior extends the dodge to any creature you control.
#[test]
fn gustcloak_savior_saves_a_teammate() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::gustcloak_savior());
    let attacker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let blocker = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    g.clear_sickness(attacker);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    drain_stack(&mut g);
    assert!(!g.attacking.iter().any(|a| a.attacker == attacker), "pulled out of combat");
    assert!(g.battlefield_find(attacker).is_some(), "and survived the 9/9");
}

/// Leery Fogbeast fogs the whole combat when it's blocked.
#[test]
fn leery_fogbeast_fogs_combat() {
    let mut g = main_phase();
    let beast = g.add_card_to_battlefield(0, catalog::leery_fogbeast());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(beast);
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: beast,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, beast)])).expect("block");
    drain_stack(&mut g);
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    assert_eq!(g.battlefield_find(blocker).unwrap().damage, 0, "no damage dealt");
    assert!(g.battlefield_find(beast).is_some(), "the 4/2 survived the 2/2");
}

/// Shaleskin Bruiser gets +3/+0 for each other attacking Beast.
#[test]
fn shaleskin_bruiser_scales_with_the_herd() {
    let mut g = main_phase();
    let bruiser = g.add_card_to_battlefield(0, catalog::shaleskin_bruiser());
    let pal = g.add_card_to_battlefield(0, catalog::snapping_thragg()); // a Beast
    g.clear_sickness(bruiser);
    g.clear_sickness(pal);
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: bruiser, target: AttackTarget::Player(1) },
        Attack { attacker: pal, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(bruiser).unwrap().power, 7, "4 + 3 for one other Beast");
}

/// Ebonblade Reaper halves both life totals.
#[test]
fn ebonblade_reaper_halves_both_ways() {
    let mut g = main_phase();
    let reaper = g.add_card_to_battlefield(0, catalog::ebonblade_reaper());
    g.clear_sickness(reaper);
    g.players[0].life = 20;
    g.players[1].life = 20;
    swing(&mut g, reaper);
    assert_eq!(g.players[0].life, 10, "you lose half on attack");
    // 20 − 1 combat damage = 19, then half of 19 rounded up.
    assert_eq!(g.players[1].life, 9, "they lose half on connect");
}

/// Haunted Cadaver trades itself for three cards.
#[test]
fn haunted_cadaver_strips_three() {
    let mut g = main_phase();
    let cadaver = g.add_card_to_battlefield(0, catalog::haunted_cadaver());
    g.clear_sickness(cadaver);
    for _ in 0..4 {
        g.add_card_to_hand(1, catalog::forest());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    swing(&mut g, cadaver);
    assert_eq!(g.players[1].hand.len(), 1, "discarded three");
    assert!(g.battlefield_find(cadaver).is_none(), "and sacrificed itself");
}

/// CR 707.9 — Break Open and Ixidor turn a face-down creature up for free;
/// Nosy Goblin blows one up.
#[test]
fn ons_face_down_matters() {
    let mut g = main_phase();
    let hidden = g.add_card_to_hand(1, catalog::krosan_colossus());
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastFaceDown { card_id: hidden }).expect("cast face down");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    g.priority.player_with_priority = 0;
    let spell = g.add_card_to_hand(0, catalog::break_open());
    cast(&mut g, 0, spell, Some(Target::Permanent(hidden)));
    let up = g.battlefield_find(hidden).expect("still there");
    assert!(!up.face_down, "flipped up for free");
    assert_eq!(up.definition.name, "Krosan Colossus");
}

/// Ixidor pumps face-down creatures and unmasks them.
#[test]
fn ixidor_pumps_and_unmasks() {
    let mut g = main_phase();
    let hidden = g.add_card_to_hand(0, catalog::spined_basher());
    mana(&mut g, 0);
    g.perform_action(GameAction::CastFaceDown { card_id: hidden }).expect("cast face down");
    drain_stack(&mut g);
    let ixidor = g.add_card_to_battlefield(0, catalog::ixidor_reality_sculptor());
    let cp = g.computed_permanent(hidden).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3), "face-down 2/2 gets +1/+1");
    activate(&mut g, 0, ixidor, 0, Some(Target::Permanent(hidden)));
    assert!(!g.battlefield_find(hidden).unwrap().face_down, "unmasked");
}

/// Nosy Goblin destroys a face-down creature.
#[test]
fn nosy_goblin_destroys_a_morph() {
    let mut g = main_phase();
    let hidden = g.add_card_to_hand(1, catalog::krosan_colossus());
    g.players[1].mana_pool.add_colorless(3);
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastFaceDown { card_id: hidden }).expect("cast face down");
    drain_stack(&mut g);
    g.active_player_idx = 0;
    let goblin = g.add_card_to_battlefield(0, catalog::nosy_goblin());
    g.clear_sickness(goblin);
    activate(&mut g, 0, goblin, 0, Some(Target::Permanent(hidden)));
    assert!(g.battlefield_find(hidden).is_none(), "the morph died");
}

/// Dream Chisel makes the morph cast cost {2}.
#[test]
fn dream_chisel_discounts_morphs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::dream_chisel());
    let hidden = g.add_card_to_hand(0, catalog::krosan_colossus());
    g.players[0].mana_pool.add_colorless(2);
    g.perform_action(GameAction::CastFaceDown { card_id: hidden }).expect("cast face down for {2}");
    drain_stack(&mut g);
    assert!(g.battlefield_find(hidden).is_some_and(|c| c.face_down));
    assert_eq!(g.players[0].mana_pool.total(), 0, "exactly two mana spent");
}

/// CR 205.3m — "a creature type other than Wall" can't be answered with Wall.
#[test]
fn cr_205_3m_imagecrafter_cant_name_wall() {
    let mut g = main_phase();
    let crafter = g.add_card_to_battlefield(0, catalog::imagecrafter());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(crafter);
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Wall)]));
    activate(&mut g, 0, crafter, 0, Some(Target::Permanent(bear)));
    assert!(
        !g.computed_permanent(bear).unwrap().subtypes.creature_types.contains(&CreatureType::Wall),
        "the forbidden type is overruled"
    );
}

/// CR 613.8 — a keyword grant gated on a creature type sees the layer-4 retype
/// even though it is evaluated inside the layer gather.
#[test]
fn cr_613_8_type_gated_grant_sees_a_retype() {
    let mut g = main_phase();
    let wall = g.add_card_to_battlefield(0, catalog::mistform_wall());
    assert!(g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::CreatureType(CreatureType::Bird)]));
    activate(&mut g, 0, wall, 0, None);
    assert!(!g.computed_permanent(wall).unwrap().keywords.contains(&Keyword::Defender));
}

// ── Wave 5 ───────────────────────────────────────────────────────────────────

/// Aether Charge fires a Beast at an opponent for 4.
#[test]
fn aether_charge_bolts_for_beasts() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::aether_charge());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let beast = g.add_card_to_hand(0, catalog::snapping_thragg());
    let life = g.players[1].life;
    cast(&mut g, 0, beast, None);
    assert_eq!(g.players[1].life, life - 4);
}

/// Airdrop Condor throws a Goblin's power at any target.
#[test]
fn airdrop_condor_throws_a_goblin() {
    let mut g = main_phase();
    let condor = g.add_card_to_battlefield(0, catalog::airdrop_condor());
    g.add_card_to_battlefield(0, catalog::nosy_goblin()); // a 2/1 Goblin
    let life = g.players[1].life;
    activate(&mut g, 0, condor, 0, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 2, "damage equal to the sacrificed power");
}

/// Aphetto Alchemist untaps anything.
#[test]
fn aphetto_alchemist_untaps() {
    let mut g = main_phase();
    let alch = g.add_card_to_battlefield(0, catalog::aphetto_alchemist());
    g.clear_sickness(alch);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.battlefield_find_mut(bear).unwrap().tapped = true;
    activate(&mut g, 0, alch, 0, Some(Target::Permanent(bear)));
    assert!(!g.battlefield_find(bear).unwrap().tapped);
}

/// Boneknitter regenerates a Zombie.
#[test]
fn boneknitter_regenerates_a_zombie() {
    let mut g = main_phase();
    let knitter = g.add_card_to_battlefield(0, catalog::boneknitter());
    let zombie = g.add_card_to_battlefield(0, catalog::gluttonous_zombie());
    activate(&mut g, 0, knitter, 0, Some(Target::Permanent(zombie)));
    kill(&mut g, zombie);
    assert!(g.battlefield_find(zombie).is_some(), "the shield saved it");
}

/// Battlefield Medic's shield scales with your Cleric count.
#[test]
fn battlefield_medic_shields_by_cleric_count() {
    let mut g = main_phase();
    let medic = g.add_card_to_battlefield(0, catalog::battlefield_medic());
    g.add_card_to_battlefield(0, catalog::rotlung_reanimator()); // a second Cleric
    g.clear_sickness(medic);
    let bear = g.add_card_to_battlefield(0, catalog::krosan_colossus());
    activate(&mut g, 0, medic, 0, Some(Target::Permanent(bear)));
    let blast = g.add_card_to_hand(0, catalog::solar_blast());
    cast(&mut g, 0, blast, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 1, "2 of the 3 prevented");
}

/// Daunting Defender shaves a point off every hit on your Clerics.
#[test]
fn daunting_defender_shields_clerics() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::daunting_defender());
    let cleric = g.add_card_to_battlefield(0, catalog::ancestors_prophet()); // 1/5 Cleric
    let other = g.add_card_to_battlefield(0, catalog::krosan_colossus());
    for target in [cleric, other] {
        let blast = g.add_card_to_hand(0, catalog::solar_blast());
        cast(&mut g, 0, blast, Some(Target::Permanent(target)));
    }
    assert_eq!(g.battlefield_find(cleric).unwrap().damage, 2, "Clerics take one less");
    assert_eq!(g.battlefield_find(other).unwrap().damage, 3, "everyone else takes it all");
}

/// Broodhatch Nantuko converts damage into Insects.
#[test]
fn broodhatch_nantuko_hatches_on_damage() {
    let mut g = main_phase();
    let nantuko = g.add_card_to_battlefield(0, catalog::broodhatch_nantuko());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let blast = g.add_card_to_hand(0, catalog::solar_blast());
    cast(&mut g, 0, blast, Some(Target::Permanent(nantuko)));
    assert_eq!(count_named(&g, 0, "Insect"), 3, "one per point of damage");
}

/// Disruptive Pitmage taxes every spell for {1}.
#[test]
fn disruptive_pitmage_taxes() {
    let mut g = main_phase();
    let mage = g.add_card_to_battlefield(0, catalog::disruptive_pitmage());
    g.clear_sickness(mage);
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bolt");
    g.players[1].mana_pool.empty();
    activate(&mut g, 0, mage, 0, Some(Target::Permanent(bolt)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt), "unpaid → countered");
}

/// Dwarven Blastminer blows up a nonbasic land.
#[test]
fn dwarven_blastminer_kills_a_nonbasic() {
    let mut g = main_phase();
    let miner = g.add_card_to_battlefield(0, catalog::dwarven_blastminer());
    g.clear_sickness(miner);
    let land = g.add_card_to_battlefield(1, catalog::grand_coliseum());
    activate(&mut g, 0, miner, 0, Some(Target::Permanent(land)));
    assert!(g.battlefield_find(land).is_none());
}

/// Spitfire Handler can't block anything bigger than itself.
#[test]
fn spitfire_handler_blocks_only_small() {
    let mut g = main_phase();
    let handler = g.add_card_to_battlefield(1, catalog::spitfire_handler());
    let big = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(big);
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: big,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(handler, big)])).is_err(),
        "a 1/1 can't block a 2/2"
    );
}

/// Serpentine Basilisk kills whatever it touches at end of combat.
#[test]
fn serpentine_basilisk_petrifies() {
    let mut g = main_phase();
    let basilisk = g.add_card_to_battlefield(0, catalog::serpentine_basilisk());
    let blocker = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    g.clear_sickness(basilisk);
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: basilisk,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, basilisk)])).expect("block");
    while g.step != TurnStep::PostCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).is_none(), "the 9/9 died at end of combat");
}

/// Goblin Pyromancer overruns the Goblins, then buries them.
#[test]
fn goblin_pyromancer_overruns_then_wipes() {
    let mut g = main_phase();
    let gob = g.add_card_to_battlefield(0, catalog::reckless_one());
    let pyro = g.add_card_to_hand(0, catalog::goblin_pyromancer());
    cast(&mut g, 0, pyro, None);
    // Reckless One is */* by Goblin count — two Goblins in play, then +3/+0.
    assert_eq!(g.computed_permanent(gob).unwrap().power, 5);
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(gob).is_none(), "the Goblins died at end of turn");
}

/// Cruel Revival kills a non-Zombie and buys a Zombie back.
#[test]
fn cruel_revival_kills_and_raises() {
    let mut g = main_phase();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let corpse = g.add_card_to_graveyard(0, catalog::gluttonous_zombie());
    let spell = g.add_card_to_hand(0, catalog::cruel_revival());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(victim)),
        additional_targets: vec![Target::Permanent(corpse)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert!(g.players[0].hand.iter().any(|c| c.id == corpse));
}

/// Oversold Cemetery only fires once the graveyard is deep enough.
#[test]
fn oversold_cemetery_needs_four_bodies() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::oversold_cemetery());
    let corpse = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(!g.players[0].hand.iter().any(|c| c.id == corpse), "one body isn't enough");
    for _ in 0..3 {
        g.add_card_to_graveyard(0, catalog::grizzly_bears());
    }
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].graveyard.len(), 3, "one creature came back");
}

/// Convalescent Care only pays out while you're low.
#[test]
fn convalescent_care_gates_on_life() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::convalescent_care());
    g.add_card_to_library(0, catalog::forest());
    g.players[0].life = 20;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "healthy → nothing");
    g.players[0].life = 4;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 7, "at 5 or less → gain 3");
}

/// Lightning Rift and Invigorating Boon both fire off a cycle.
#[test]
fn ons_cycling_payoffs() {
    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::lightning_rift());
    let fodder = g.add_card_to_hand(0, catalog::fade_from_memory());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    let life = g.players[1].life;
    mana(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: fodder, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2, "the Rift shot for 2");

    let mut g = main_phase();
    g.add_card_to_battlefield(0, catalog::invigorating_boon());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let fodder = g.add_card_to_hand(0, catalog::fade_from_memory());
    g.add_card_to_library(0, catalog::forest());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    mana(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: fodder, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bear).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Sea's Claim turns a land into an Island; Sandskin fogs its host.
#[test]
fn ons_utility_auras() {
    let mut g = main_phase();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let claim = g.add_card_to_hand(0, catalog::seas_claim());
    cast(&mut g, 0, claim, Some(Target::Permanent(land)));
    assert!(
        g.computed_permanent(land)
            .unwrap()
            .subtypes
            .land_types
            .contains(&crabomination::card::LandType::Island)
    );

    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(0, catalog::krosan_colossus());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let skin = g.add_card_to_hand(0, catalog::sandskin());
    cast(&mut g, 0, skin, Some(Target::Permanent(attacker)));
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    assert!(g.battlefield_find(blocker).is_some(), "Sandskin fogged the 9/9");
    assert_eq!(g.battlefield_find(attacker).unwrap().damage, 0, "and shielded it back");
}

/// Withering Hex shrinks its host as the table cycles.
#[test]
fn withering_hex_rots_with_cycling() {
    let mut g = main_phase();
    let host = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    let hex = g.add_card_to_hand(0, catalog::withering_hex());
    cast(&mut g, 0, hex, Some(Target::Permanent(host)));
    let fodder = g.add_card_to_hand(0, catalog::fade_from_memory());
    g.add_card_to_library(0, catalog::forest());
    mana(&mut g, 0);
    g.perform_action(GameAction::Cycle { card_id: fodder, x_value: None }).expect("cycle");
    drain_stack(&mut g);
    let cp = g.computed_permanent(host).unwrap();
    assert_eq!((cp.power, cp.toughness), (8, 8), "-1/-1 per plague counter");
}

/// Insurrection steals and swings the whole board.
#[test]
fn insurrection_takes_everything() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    g.battlefield_find_mut(theirs).unwrap().tapped = true;
    let spell = g.add_card_to_hand(0, catalog::insurrection());
    cast(&mut g, 0, spell, None);
    let stolen = g.battlefield_find(theirs).unwrap();
    assert_eq!(stolen.controller, 0, "gained control");
    assert!(!stolen.tapped, "untapped");
    assert!(g.computed_permanent(theirs).unwrap().keywords.contains(&Keyword::Haste));
}

/// Tribal Golem picks up a keyword per tribe you field.
#[test]
fn tribal_golem_borrows_keywords() {
    let mut g = main_phase();
    let golem = g.add_card_to_battlefield(0, catalog::tribal_golem());
    assert!(g.computed_permanent(golem).unwrap().keywords.is_empty(), "bare board, bare Golem");
    g.add_card_to_battlefield(0, catalog::snapping_thragg()); // Beast
    g.add_card_to_battlefield(0, catalog::reckless_one()); // Goblin
    let kws = g.computed_permanent(golem).unwrap().keywords;
    assert!(kws.contains(&Keyword::Trample) && kws.contains(&Keyword::Haste));
    assert!(!kws.contains(&Keyword::Flying), "no Wizard, no flying");
}

/// Run Wild hands out trample and a regeneration outlet.
#[test]
fn run_wild_grants_trample_and_regen() {
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::run_wild());
    cast(&mut g, 0, spell, Some(Target::Permanent(bear)));
    let kws = g.computed_permanent(bear).unwrap().keywords;
    assert!(kws.contains(&Keyword::Trample));
    assert!(kws.iter().any(|k| matches!(k, Keyword::Regenerate(_))));
}

/// The Onslaught charms each resolve their printed modes.
#[test]
fn ons_charms() {
    // Fever Charm mode 2 — 3 damage to a Wizard.
    let mut g = main_phase();
    let wizard = g.add_card_to_battlefield(1, catalog::aphetto_grifter());
    let charm = g.add_card_to_hand(0, catalog::fever_charm());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Permanent(wizard)),
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(wizard).is_none(), "the 1/1 Wizard died");

    // Misery Charm mode 2 — drain for 2.
    let mut g = main_phase();
    let charm = g.add_card_to_hand(0, catalog::misery_charm());
    let life = g.players[1].life;
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, life - 2);

    // Piety Charm mode 2 — team vigilance.
    let mut g = main_phase();
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let charm = g.add_card_to_hand(0, catalog::piety_charm());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: None,
        additional_targets: vec![],
        mode: Some(2),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Vigilance));

    // Vitality Charm mode 0 — an Insect.
    let mut g = main_phase();
    let charm = g.add_card_to_hand(0, catalog::vitality_charm());
    mana(&mut g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: charm,
        target: None,
        additional_targets: vec![],
        mode: Some(0),
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(count_named(&g, 0, "Insect"), 1);
}

/// The wave-5 utility activations do what they say.
#[test]
fn ons_wave5_activations() {
    // Whipcorder taps a creature.
    let mut g = main_phase();
    let corder = g.add_card_to_battlefield(0, catalog::whipcorder());
    g.clear_sickness(corder);
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, 0, corder, 0, Some(Target::Permanent(bear)));
    assert!(g.battlefield_find(bear).unwrap().tapped);

    // Voidmage Prodigy eats a Wizard to counter.
    let mut g = main_phase();
    let prodigy = g.add_card_to_battlefield(0, catalog::voidmage_prodigy());
    g.add_card_to_battlefield(0, catalog::aphetto_grifter());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.priority.player_with_priority = 1;
    mana(&mut g, 1);
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast the Bolt");
    activate(&mut g, 0, prodigy, 0, Some(Target::Permanent(bolt)));
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bolt));

    // Rummaging Wizard surveils.
    let mut g = main_phase();
    let wiz = g.add_card_to_battlefield(0, catalog::rummaging_wizard());
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, 0, wiz, 0, None);
    assert!(g.players[0].library.len() + g.players[0].graveyard.len() == 1);

    // Information Dealer looks at as many cards as you have Wizards.
    let mut g = main_phase();
    let dealer = g.add_card_to_battlefield(0, catalog::information_dealer());
    g.clear_sickness(dealer);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::forest());
    }
    let lib = g.players[0].library.len();
    activate(&mut g, 0, dealer, 0, None);
    assert_eq!(g.players[0].library.len(), lib, "looking moves nothing");

    // Venomspout Brackus shoots an attacking flier.
    let mut g = main_phase();
    let brackus = g.add_card_to_battlefield(0, catalog::venomspout_brackus());
    g.clear_sickness(brackus);
    let flier = g.add_card_to_battlefield(1, catalog::screaming_seahawk());
    g.clear_sickness(flier);
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flier,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    activate(&mut g, 0, brackus, 0, Some(Target::Permanent(flier)));
    assert!(g.battlefield_find(flier).is_none());

    // Gravel Slinger pings a combatant.
    let mut g = main_phase();
    let slinger = g.add_card_to_battlefield(0, catalog::gravel_slinger());
    g.clear_sickness(slinger);
    let attacker = g.add_card_to_battlefield(1, catalog::foothill_guide()); // 1/1
    g.clear_sickness(attacker);
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    activate(&mut g, 0, slinger, 0, Some(Target::Permanent(attacker)));
    assert!(g.battlefield_find(attacker).is_none());

    // Daru Healer prevents a point.
    let mut g = main_phase();
    let healer = g.add_card_to_battlefield(0, catalog::daru_healer());
    g.clear_sickness(healer);
    let bear = g.add_card_to_battlefield(0, catalog::krosan_colossus());
    activate(&mut g, 0, healer, 0, Some(Target::Permanent(bear)));
    let blast = g.add_card_to_hand(0, catalog::solar_blast());
    cast(&mut g, 0, blast, Some(Target::Permanent(bear)));
    assert_eq!(g.battlefield_find(bear).unwrap().damage, 2, "1 of the 3 prevented");
}

/// CR 614 — the Words cycle replaces your next draw for the turn, one charge
/// per activation, and unused charges evaporate at end of turn.
#[test]
fn words_cycle_replaces_the_next_draw() {
    let mut g = main_phase();
    let words = g.add_card_to_battlefield(0, catalog::words_of_worship());
    g.add_card_to_library(0, catalog::forest());
    let life = g.players[0].life;
    let hand = g.players[0].hand.len();
    activate(&mut g, 0, words, 0, None);
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    assert_eq!(g.players[0].life, life + 5, "the draw became 5 life");
    assert_eq!(g.players[0].hand.len(), hand, "and no card was drawn");
    // The charge is spent — the next draw is a real one.
    g.draw_one(0, &mut ev);
    assert_eq!(g.players[0].hand.len(), hand + 1);

    // Words of War turns the draw into 2 damage.
    let mut g = main_phase();
    let words = g.add_card_to_battlefield(0, catalog::words_of_war());
    g.add_card_to_library(0, catalog::forest());
    let their_life = g.players[1].life;
    activate(&mut g, 0, words, 0, None);
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, their_life - 2);
}

/// Words of Wilding and Words of Waste swap the draw for a Bear / a discard.
#[test]
fn words_of_wilding_and_waste() {
    let mut g = main_phase();
    let words = g.add_card_to_battlefield(0, catalog::words_of_wilding());
    g.add_card_to_library(0, catalog::forest());
    activate(&mut g, 0, words, 0, None);
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    assert_eq!(count_named(&g, 0, "Bear"), 1);

    let mut g = main_phase();
    let words = g.add_card_to_battlefield(0, catalog::words_of_waste());
    g.add_card_to_library(0, catalog::forest());
    g.add_card_to_hand(1, catalog::forest());
    activate(&mut g, 0, words, 0, None);
    let mut ev = vec![];
    g.draw_one(0, &mut ev);
    assert!(g.players[1].hand.is_empty(), "the opponent discarded instead");
}

/// CR 706 — a Chain spell hands the copy to the affected player, who may
/// retarget it; declining (the default) ends the chain.
#[test]
fn chain_of_acid_offers_the_copy_onward() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::dragon_roost());
    let spell = g.add_card_to_hand(0, catalog::chain_of_acid());
    cast(&mut g, 0, spell, Some(Target::Permanent(theirs)));
    assert!(g.battlefield_find(theirs).is_none(), "the enchantment died");
    assert!(g.stack.is_empty(), "the opponent declined the chain");

    // Accepting hands the affected player a copy they control, which
    // retargets onto the other side of the table.
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::dragon_roost());
    let mine = g.add_card_to_battlefield(0, catalog::dream_chisel());
    let spell = g.add_card_to_hand(0, catalog::chain_of_acid());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Bool(true),
        DecisionAnswer::Target(Target::Permanent(mine)),
    ]));
    cast(&mut g, 0, spell, Some(Target::Permanent(theirs)));
    assert!(g.battlefield_find(theirs).is_none(), "the first link killed their Roost");
    assert!(g.battlefield_find(mine).is_none(), "and their retargeted copy came back at mine");
}

/// Chain of Vapor's toll is a land sacrifice; with no land the chain can't
/// continue.
#[test]
fn chain_of_vapor_bounces_and_tolls() {
    let mut g = main_phase();
    let theirs = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::chain_of_vapor());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Bool(true)]));
    cast(&mut g, 0, spell, Some(Target::Permanent(theirs)));
    assert!(g.players[1].hand.iter().any(|c| c.id == theirs), "bounced home");
    assert!(g.stack.is_empty(), "no land to pay the toll, so no copy");
}

/// Chain of Silence blanks all the damage its target would deal.
#[test]
fn chain_of_silence_blanks_a_creature() {
    let mut g = main_phase();
    let attacker = g.add_card_to_battlefield(1, catalog::krosan_colossus());
    let blocker = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.clear_sickness(attacker);
    let spell = g.add_card_to_hand(0, catalog::chain_of_silence());
    cast(&mut g, 0, spell, Some(Target::Permanent(attacker)));
    g.active_player_idx = 1;
    advance_to_attackers(&mut g);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).expect("block");
    while g.step != TurnStep::EndCombat {
        g.perform_action(GameAction::PassPriority).expect("pass priority");
    }
    assert!(g.battlefield_find(blocker).is_some(), "the 9/9 dealt no damage");
}

/// Chain of Smog strips two cards; Chain of Plasma burns for 3.
#[test]
fn ons_chain_smog_and_plasma() {
    let mut g = main_phase();
    for _ in 0..3 {
        g.add_card_to_hand(1, catalog::forest());
    }
    let spell = g.add_card_to_hand(0, catalog::chain_of_smog());
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].hand.len(), 1);

    let mut g = main_phase();
    let spell = g.add_card_to_hand(0, catalog::chain_of_plasma());
    let life = g.players[1].life;
    cast(&mut g, 0, spell, Some(Target::Player(1)));
    assert_eq!(g.players[1].life, life - 3);
}
