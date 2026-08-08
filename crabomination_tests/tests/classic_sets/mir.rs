//! Mirage (MIR) — `catalog::sets::mir`.

use crabomination::card::{CardDefinition, CardId, CounterType, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn cast(
    g: &mut GameState,
    id: CardId,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

fn activate(
    g: &mut GameState,
    id: CardId,
    index: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: id,
        ability_index: index,
        target,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .map(|_| ())
}

/// Attack with `attacker`, then try to block it with `blocker`.
fn try_block(
    g: &mut GameState,
    attacker: CardId,
    blocker: CardId,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, attacker)])).map(|_| ())
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The slow-fetch cycle enters tapped and trades itself for either printed
/// land type.
#[test]
fn slow_fetch_cycle_fetches_either_type() {
    for (fetch, wanted) in [
        (catalog::bad_river as fn() -> CardDefinition, "Swamp"),
        (catalog::flood_plain, "Island"),
        (catalog::grasslands, "Plains"),
        (catalog::mountain_valley, "Forest"),
        (catalog::rocky_tar_pit, "Mountain"),
    ] {
        let name = fetch().name;
        let mut g = two_player_game();
        let land = g.add_card_to_hand(0, fetch());
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::PlayLand(land)).expect("play it");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).unwrap().tapped, "{name} entered tapped");
        g.battlefield_find_mut(land).unwrap().tapped = false;
        let target = match wanted {
            "Swamp" => catalog::swamp(),
            "Island" => catalog::island(),
            "Plains" => catalog::plains(),
            "Forest" => catalog::forest(),
            _ => catalog::mountain(),
        };
        g.add_card_to_library(0, target);
        activate(&mut g, land, 0, None).expect("crack it");
        drain_stack(&mut g);
        assert!(g.battlefield_find(land).is_none(), "{name} sacrificed itself");
        assert!(
            g.battlefield.iter().any(|c| c.definition.name == wanted),
            "{name} fetched a {wanted}"
        );
    }
}

/// Crystal Vein taps for one, or cashes itself in for two.
#[test]
fn crystal_vein_trades_itself_for_two_mana() {
    let mut g = two_player_game();
    let vein = ready(&mut g, 0, catalog::crystal_vein());
    activate(&mut g, vein, 1, None).expect("crack it");
    drain_stack(&mut g);
    assert_eq!(g.players[0].mana_pool.colorless_amount(), 2);
    assert!(g.battlefield_find(vein).is_none());
}

// ── New block restrictions ──────────────────────────────────────────────────

/// Sunweb can't be bothered by small attackers but stops a real one
/// (CR 509.1b).
#[test]
fn sunweb_only_blocks_big_attackers() {
    let mut g = two_player_game();
    let small = ready(&mut g, 0, catalog::mtenda_herder());
    let web = ready(&mut g, 1, catalog::sunweb());
    assert!(try_block(&mut g, small, web).is_err(), "1 power is beneath it");

    let mut g = two_player_game();
    let big = ready(&mut g, 0, catalog::crash_of_rhinos());
    let web = ready(&mut g, 1, catalog::sunweb());
    assert!(try_block(&mut g, big, web).is_ok(), "8 power is worth blocking");
}

/// Gibbering Hyenas can't block black creatures, but blocks anything else.
#[test]
fn gibbering_hyenas_cant_block_black() {
    let mut g = two_player_game();
    let black = ready(&mut g, 0, catalog::cadaverous_knight());
    let hyenas = ready(&mut g, 1, catalog::gibbering_hyenas());
    assert!(try_block(&mut g, black, hyenas).is_err());

    let mut g = two_player_game();
    let white = ready(&mut g, 0, catalog::femeref_scouts());
    let hyenas = ready(&mut g, 1, catalog::gibbering_hyenas());
    assert!(try_block(&mut g, white, hyenas).is_ok());
}

/// Stalking Tiger refuses a second blocker.
#[test]
fn stalking_tiger_cant_be_gang_blocked() {
    let mut g = two_player_game();
    let tiger = ready(&mut g, 0, catalog::stalking_tiger());
    let a = ready(&mut g, 1, catalog::femeref_scouts());
    let b = ready(&mut g, 1, catalog::femeref_scouts());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: tiger,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    assert!(
        g.perform_action(GameAction::DeclareBlockers(vec![(a, tiger), (b, tiger)])).is_err(),
        "two blockers is illegal"
    );
}

// ── Combat & statics ────────────────────────────────────────────────────────

/// Telim'Tor rallies every attacking flanker, itself included.
#[test]
fn telimtor_pumps_attacking_flankers() {
    let mut g = two_player_game();
    let boss = ready(&mut g, 0, catalog::telimtor());
    let friend = ready(&mut g, 0, catalog::mtenda_herder());
    let idle = ready(&mut g, 0, catalog::zhalfirin_knight());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: boss, target: AttackTarget::Player(1) },
        Attack { attacker: friend, target: AttackTarget::Player(1) },
    ]))
    .expect("attack");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(friend).unwrap().power, 2);
    assert_eq!(g.computed_permanent(boss).unwrap().power, 3);
    assert_eq!(g.computed_permanent(idle).unwrap().power, 2, "the one at home is unchanged");
}

/// Sidar Jabari taps a would-be blocker as it attacks.
#[test]
fn sidar_jabari_taps_a_blocker() {
    let mut g = two_player_game();
    let jabari = ready(&mut g, 0, catalog::sidar_jabari());
    let blocker = ready(&mut g, 1, catalog::femeref_scouts());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: jabari,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(blocker).unwrap().tapped);
}

/// Zuberi anthems other Griffins, never itself.
#[test]
fn zuberi_anthems_other_griffins() {
    let mut g = two_player_game();
    let zuberi = ready(&mut g, 0, catalog::zuberi_golden_feather());
    let griffin = ready(&mut g, 0, catalog::teremko_griffin());
    assert_eq!(g.computed_permanent(griffin).unwrap().power, 3);
    assert_eq!(g.computed_permanent(zuberi).unwrap().power, 3, "not itself");
}

/// Spirit of the Night only has first strike while it's attacking.
#[test]
fn spirit_of_the_night_first_strikes_only_on_offense() {
    let mut g = two_player_game();
    let spirit = ready(&mut g, 0, catalog::spirit_of_the_night());
    assert!(!g.computed_permanent(spirit).unwrap().keywords.contains(&Keyword::FirstStrike));
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: spirit,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    assert!(g.computed_permanent(spirit).unwrap().keywords.contains(&Keyword::FirstStrike));
}

/// Maro's body is the size of your hand.
#[test]
fn maro_is_as_big_as_your_hand() {
    let mut g = two_player_game();
    let maro = ready(&mut g, 0, catalog::maro());
    for _ in 0..3 {
        g.add_card_to_hand(0, catalog::grizzly_bears());
    }
    let cp = g.computed_permanent(maro).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Crypt Cobra poisons a defender it slips past.
#[test]
fn crypt_cobra_poisons_on_an_unblocked_attack() {
    let mut g = two_player_game();
    let cobra = ready(&mut g, 0, catalog::crypt_cobra());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: cobra,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    drain_stack(&mut g);
    assert_eq!(g.players[1].poison_counters, 1);
}

/// Zebra Unicorn turns its damage into life.
#[test]
fn zebra_unicorn_gains_life_for_its_damage() {
    let mut g = two_player_game();
    let unicorn = ready(&mut g, 0, catalog::zebra_unicorn());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: unicorn,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 18);
    assert_eq!(g.players[0].life, 22, "gained the two it dealt");
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Superior Numbers scales with the creature-count gap, and never goes
/// negative.
#[test]
fn superior_numbers_scales_with_the_board() {
    let mut g = two_player_game();
    for _ in 0..3 {
        ready(&mut g, 0, catalog::femeref_scouts());
    }
    let victim = ready(&mut g, 1, catalog::sunweb());
    let spell = g.add_card_to_hand(0, catalog::superior_numbers());
    g.players[0].mana_pool.add(Color::Green, 2);
    cast(&mut g, spell, Some(Target::Permanent(victim))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().damage, 2, "3 mine minus 1 theirs");
}

/// Withering Boon counters a creature spell and bills 3 life for the privilege.
#[test]
fn withering_boon_costs_three_life() {
    let mut g = two_player_game();
    let bears = g.add_card_to_hand(1, catalog::grizzly_bears());
    g.players[1].mana_pool.add(Color::Green, 2);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bears,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast bears");
    let boon = g.add_card_to_hand(0, catalog::withering_boon());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: boon,
        target: Some(Target::Permanent(bears)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("counter it");
    drain_stack(&mut g);
    assert!(g.players[1].graveyard.iter().any(|c| c.id == bears), "countered");
    assert_eq!(g.players[0].life, 17, "paid the additional 3 life");
}

/// Ashen Powder steals a body out of an opponent's graveyard.
#[test]
fn ashen_powder_reanimates_an_opponents_creature() {
    let mut g = two_player_game();
    let corpse = g.add_card_to_graveyard(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::ashen_powder());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(corpse))).expect("cast");
    drain_stack(&mut g);
    let c = g.battlefield_find(corpse).expect("reanimated");
    assert_eq!(c.controller, 0, "under my control");
}

/// Infernal Contract draws four and takes half your life, rounded up.
#[test]
fn infernal_contract_halves_your_life() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].life = 19;
    let spell = g.add_card_to_hand(0, catalog::infernal_contract());
    g.players[0].mana_pool.add(Color::Black, 3);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 4);
    assert_eq!(g.players[0].life, 9, "lost 10 of 19");
}

/// Savage Twister sweeps for its X.
#[test]
fn savage_twister_sweeps_for_x() {
    let mut g = two_player_game();
    let mine = ready(&mut g, 0, catalog::femeref_scouts());
    let theirs = ready(&mut g, 1, catalog::mtenda_herder());
    let spell = g.add_card_to_hand(0, catalog::savage_twister());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(2),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(theirs).is_none(), "a 1/1 dies");
    assert_eq!(g.battlefield_find(mine).unwrap().damage, 2, "a 1/4 survives, marked");
}

/// Blinding Light leaves white creatures alone.
#[test]
fn blinding_light_spares_white() {
    let mut g = two_player_game();
    let white = ready(&mut g, 1, catalog::femeref_scouts());
    let green = ready(&mut g, 1, catalog::gibbering_hyenas());
    let spell = g.add_card_to_hand(0, catalog::blinding_light());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(!g.battlefield_find(white).unwrap().tapped);
    assert!(g.battlefield_find(green).unwrap().tapped);
}

/// Tidal Wave's Wall is gone by the next end step.
#[test]
fn tidal_wave_wall_is_temporary() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::tidal_wave());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Wall"));
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(!g.battlefield.iter().any(|c| c.definition.name == "Wall"), "sacrificed");
}

/// Ritual of Steel replaces itself at the next upkeep.
#[test]
fn ritual_of_steel_draws_next_upkeep() {
    let mut g = two_player_game();
    let host = ready(&mut g, 0, catalog::femeref_scouts());
    g.add_card_to_library(0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ritual_of_steel());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(host).unwrap().toughness, 6);
    let before = g.players[0].hand.len();
    g.turn_number += 1;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), before + 1);
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// The Cages fire only on the hand size they police.
#[test]
fn cages_gate_on_opponent_hand_size() {
    for (cage, hand, expect_damage) in [
        (catalog::misers_cage as fn() -> CardDefinition, 5, true),
        (catalog::misers_cage, 4, false),
        (catalog::paupers_cage, 2, true),
        (catalog::paupers_cage, 3, false),
    ] {
        let name = cage().name;
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, cage());
        for _ in 0..hand {
            g.add_card_to_hand(1, catalog::grizzly_bears());
        }
        g.active_player_idx = 1;
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
        let expected = if expect_damage { 18 } else { 20 };
        assert_eq!(g.players[1].life, expected, "{name} with {hand} cards");
    }
}

/// Elixir of Vitality enters tapped and pays out four.
#[test]
fn elixir_of_vitality_enters_tapped() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(0, catalog::elixir_of_vitality());
    g.players[0].mana_pool.add_colorless(4);
    cast(&mut g, card, None).expect("cast");
    drain_stack(&mut g);
    let elixir = card;
    assert!(g.battlefield_find(elixir).unwrap().tapped);
    g.battlefield_find_mut(elixir).unwrap().tapped = false;
    g.clear_sickness(elixir);
    activate(&mut g, elixir, 0, None).expect("cash in");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 24);
}

// ── Sacrifice-fuelled abilities ─────────────────────────────────────────────

/// Village Elder trades a Forest for a regeneration shield.
#[test]
fn village_elder_regenerates_for_a_forest() {
    let mut g = two_player_game();
    let elder = ready(&mut g, 0, catalog::village_elder());
    let forest = ready(&mut g, 0, catalog::forest());
    let target = ready(&mut g, 0, catalog::femeref_scouts());
    g.players[0].mana_pool.add(Color::Green, 1);
    activate(&mut g, elder, 0, Some(Target::Permanent(target))).expect("shield it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(forest).is_none(), "ate the Forest");
    assert!(g.battlefield_find(target).unwrap().regeneration_shields > 0);
}

/// Mire Shade eats a Swamp to grow permanently.
#[test]
fn mire_shade_grows_by_eating_swamps() {
    let mut g = two_player_game();
    let shade = ready(&mut g, 0, catalog::mire_shade());
    let swamp = ready(&mut g, 0, catalog::swamp());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.step = TurnStep::PreCombatMain;
    activate(&mut g, shade, 0, None).expect("grow");
    drain_stack(&mut g);
    assert!(g.battlefield_find(swamp).is_none());
    assert_eq!(g.battlefield_find(shade).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// The plain Mirage bodies ship with their printed stats and keywords.
#[test]
fn mirage_vanilla_bodies_are_printed_correctly() {
    for (def, pt, kws) in [
        (catalog::bay_falcon as fn() -> CardDefinition, (1, 1), vec![Keyword::Flying, Keyword::Vigilance]),
        (catalog::crash_of_rhinos, (8, 4), vec![Keyword::Trample]),
        (catalog::giant_mantis, (2, 4), vec![Keyword::Reach]),
        (catalog::melesse_spirit, (3, 3), vec![Keyword::Flying, Keyword::Protection(Color::Black)]),
        (catalog::hazerider_drake, (2, 3), vec![Keyword::Flying, Keyword::Protection(Color::Red)]),
        (catalog::volcanic_dragon, (4, 4), vec![Keyword::Flying, Keyword::Haste]),
        (catalog::karoo_meerkat, (2, 1), vec![Keyword::Protection(Color::Blue)]),
        (catalog::teekas_dragon, (5, 5), vec![Keyword::Flying, Keyword::Trample, Keyword::Rampage(4)]),
        (catalog::merfolk_raiders, (2, 3), vec![Keyword::Phasing]),
        (catalog::teremko_griffin, (2, 2), vec![Keyword::Flying, Keyword::Banding]),
    ] {
        let d = def();
        assert_eq!((d.power, d.toughness), pt, "{}", d.name);
        for kw in kws {
            assert!(d.keywords.contains(&kw), "{} has {kw:?}", d.name);
        }
    }
}

// ── Second wave ─────────────────────────────────────────────────────────────

/// Cast a modal spell for seat 0 choosing `mode`.
fn cast_mode(
    g: &mut GameState,
    id: CardId,
    mode: usize,
    target: Option<Target>,
) -> Result<(), crabomination::game::GameError> {
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target,
        additional_targets: vec![],
        mode: Some(mode),
        x_value: None,
    })
    .map(|_| ())
}

/// The Charms are real three-mode spells.
#[test]
fn charms_are_modal() {
    for charm in [
        catalog::ivory_charm as fn() -> CardDefinition,
        catalog::sapphire_charm,
        catalog::ebony_charm,
        catalog::chaos_charm,
        catalog::seedling_charm,
    ] {
        let d = charm();
        let crabomination::effect::Effect::ChooseMode(modes) = &d.effect else {
            panic!("{} isn't modal", d.name);
        };
        assert_eq!(modes.len(), 3, "{}", d.name);
    }
}

/// Chaos Charm's third mode hands out haste.
#[test]
fn chaos_charm_grants_haste() {
    let mut g = two_player_game();
    let body = g.add_card_to_battlefield(0, catalog::femeref_scouts());
    let charm = g.add_card_to_hand(0, catalog::chaos_charm());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast_mode(&mut g, charm, 2, Some(Target::Permanent(body))).expect("cast");
    drain_stack(&mut g);
    assert!(g.computed_permanent(body).unwrap().keywords.contains(&Keyword::Haste));
}

/// Jungle Wurm shrinks per extra blocker (reverse rampage).
#[test]
fn jungle_wurm_shrinks_per_extra_blocker() {
    let mut g = two_player_game();
    let wurm = ready(&mut g, 0, catalog::jungle_wurm());
    let a = ready(&mut g, 1, catalog::femeref_scouts());
    let b = ready(&mut g, 1, catalog::femeref_scouts());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: wurm,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(a, wurm), (b, wurm)])).expect("gang block");
    drain_stack(&mut g);
    let cp = g.computed_permanent(wurm).unwrap();
    assert_eq!((cp.power, cp.toughness), (4, 4), "one blocker beyond the first");
}

/// Dread Specter takes a nonblack blocker with it, and spares a black one.
#[test]
fn dread_specter_kills_nonblack_blockers() {
    for (blocker, dies) in [
        (catalog::femeref_scouts as fn() -> CardDefinition, true),
        (catalog::harbinger_of_night, false),
    ] {
        let mut g = two_player_game();
        let specter = ready(&mut g, 0, catalog::dread_specter());
        let wall = ready(&mut g, 1, blocker());
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: specter,
            target: AttackTarget::Player(1),
        }]))
        .expect("attack");
        while g.step != TurnStep::DeclareBlockers {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        g.priority.player_with_priority = 1;
        g.perform_action(GameAction::DeclareBlockers(vec![(wall, specter)])).expect("block");
        while g.step != TurnStep::End {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        assert_eq!(g.battlefield_find(wall).is_none(), dies, "{}", blocker().name);
    }
}

/// Lead Golem stays tapped after it attacks.
#[test]
fn lead_golem_doesnt_untap_after_attacking() {
    let mut g = two_player_game();
    let golem = ready(&mut g, 0, catalog::lead_golem());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: golem,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).unwrap().skip_next_untap);
}

/// Gravebane Zombie goes back on top of the library instead of dying.
#[test]
fn gravebane_zombie_returns_to_the_library() {
    let mut g = two_player_game();
    let zombie = ready(&mut g, 0, catalog::gravebane_zombie());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 3);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(zombie)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt it");
    drain_stack(&mut g);
    assert!(g.battlefield_find(zombie).is_none());
    assert!(!g.players[0].graveyard.iter().any(|c| c.id == zombie), "not the graveyard");
    assert_eq!(g.players[0].library.first().map(|c| c.id), Some(zombie));
}

/// Uktabi Wildcats is exactly as big as your Forest count.
#[test]
fn uktabi_wildcats_counts_forests() {
    let mut g = two_player_game();
    let cats = ready(&mut g, 0, catalog::uktabi_wildcats());
    for _ in 0..3 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let cp = g.computed_permanent(cats).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 3));
}

/// Radiant Essence only grows against a black board.
#[test]
fn radiant_essence_grows_against_black() {
    let mut g = two_player_game();
    let essence = ready(&mut g, 0, catalog::radiant_essence());
    let cp = g.computed_permanent(essence).unwrap();
    assert_eq!((cp.power, cp.toughness), (2, 3));
    g.add_card_to_battlefield(1, catalog::restless_dead());
    let cp = g.computed_permanent(essence).unwrap();
    assert_eq!((cp.power, cp.toughness), (3, 5));
}

/// Sewer Rats can only be pumped three times a turn.
#[test]
fn sewer_rats_caps_at_three_activations() {
    let mut g = two_player_game();
    let rats = ready(&mut g, 0, catalog::sewer_rats());
    g.players[0].mana_pool.add(Color::Black, 4);
    for i in 0..3 {
        activate(&mut g, rats, 0, None).unwrap_or_else(|e| panic!("activation {i}: {e:?}"));
        drain_stack(&mut g);
    }
    assert!(activate(&mut g, rats, 0, None).is_err(), "the fourth is locked out");
    assert_eq!(g.computed_permanent(rats).unwrap().power, 4);
    assert_eq!(g.players[0].life, 17, "one life per activation");
}

/// Seeds of Innocence pays each artifact's controller its mana value.
#[test]
fn seeds_of_innocence_pays_for_the_artifacts_it_breaks() {
    let mut g = two_player_game();
    let mine = ready(&mut g, 0, catalog::mana_prism());
    let theirs = ready(&mut g, 1, catalog::telimtors_darts());
    let spell = g.add_card_to_hand(0, catalog::seeds_of_innocence());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_none());
    assert_eq!(g.players[0].life, 23, "Mana Prism is three");
    assert_eq!(g.players[1].life, 22, "Telim'Tor's Darts is two");
}

/// Painful Memories stacks a card off an opponent's hand.
#[test]
fn painful_memories_stacks_a_card() {
    let mut g = two_player_game();
    let card = g.add_card_to_hand(1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::painful_memories());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, Some(Target::Player(1))).expect("cast");
    drain_stack(&mut g);
    assert!(!g.players[1].hand.iter().any(|c| c.id == card), "left their hand");
    assert_eq!(g.players[1].library.first().map(|c| c.id), Some(card));
}

/// Reparations only fires on spells that point at you.
#[test]
fn reparations_reads_the_targets() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::reparations());
    let mine = ready(&mut g, 0, catalog::femeref_scouts());
    let theirs = ready(&mut g, 1, catalog::restless_dead());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    for (target, expect_draw) in [(theirs, false), (mine, true)] {
        let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
        g.players[1].mana_pool.add(Color::Red, 1);
        g.priority.player_with_priority = 1;
        let before = g.players[0].hand.len();
        g.perform_action(GameAction::CastSpell {
            card_id: bolt,
            target: Some(Target::Permanent(target)),
            additional_targets: vec![],
            mode: None,
            x_value: None,
        })
        .expect("bolt");
        drain_stack(&mut g);
        let drew = g.players[0].hand.len() > before;
        assert_eq!(drew, expect_draw);
    }
}

/// Forsaken Wastes shuts off lifegain and bills whoever targets it.
#[test]
fn forsaken_wastes_locks_lifegain_and_taxes_targeting() {
    let mut g = two_player_game();
    let wastes = g.add_card_to_battlefield(0, catalog::forsaken_wastes());
    let elixir = ready(&mut g, 0, catalog::elixir_of_vitality());
    g.battlefield_find_mut(elixir).unwrap().tapped = false;
    activate(&mut g, elixir, 0, None).expect("try to gain");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "no life gained");

    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Permanent(wastes)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("target it");
    drain_stack(&mut g);
    assert_eq!(g.players[1].life, 15, "5 life for pointing at it");
}

/// Thirst taps its host and needs {U} an upkeep to stay.
#[test]
fn thirst_taps_and_charges_rent() {
    let mut g = two_player_game();
    let host = ready(&mut g, 1, catalog::femeref_scouts());
    let aura = g.add_card_to_hand(0, catalog::thirst());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(host).unwrap().tapped);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "unpaid, so sacrificed");
}

/// Ersatz Gnomes launder the colour off a permanent.
#[test]
fn ersatz_gnomes_make_things_colorless() {
    let mut g = two_player_game();
    let gnomes = ready(&mut g, 0, catalog::ersatz_gnomes());
    let victim = ready(&mut g, 1, catalog::restless_dead());
    activate(&mut g, gnomes, 0, Some(Target::Permanent(victim))).expect("launder");
    drain_stack(&mut g);
    assert!(g.computed_permanent(victim).unwrap().colors.is_empty());
}

/// Carrion turns the sacrificed body's power into Insects.
#[test]
fn carrion_makes_an_insect_per_power() {
    let mut g = two_player_game();
    ready(&mut g, 0, catalog::crash_of_rhinos());
    let spell = g.add_card_to_hand(0, catalog::carrion());
    g.players[0].mana_pool.add(Color::Black, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(
        g.battlefield.iter().filter(|c| c.definition.name == "Insect").count(),
        8,
        "an 8/4's worth"
    );
}

/// Waiting in the Weeds gives every player a Cat per untapped Forest.
#[test]
fn waiting_in_the_weeds_counts_each_players_forests() {
    let mut g = two_player_game();
    for _ in 0..2 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    let tapped = g.add_card_to_battlefield(1, catalog::forest());
    g.battlefield_find_mut(tapped).unwrap().tapped = true;
    g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::waiting_in_the_weeds());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    let cats = |seat: usize, g: &GameState| {
        g.battlefield
            .iter()
            .filter(|c| c.definition.name == "Cat" && c.controller == seat)
            .count()
    };
    assert_eq!(cats(0, &g), 2);
    assert_eq!(cats(1, &g), 1, "the tapped Forest doesn't count");
}

/// Ancestral Memories keeps two of seven and bins the rest.
#[test]
fn ancestral_memories_keeps_two_of_seven() {
    let mut g = two_player_game();
    for _ in 0..7 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::ancestral_memories());
    g.players[0].mana_pool.add(Color::Blue, 3);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(g.players[0].graveyard.iter().filter(|c| c.definition.is_creature()).count(), 5);
}

/// Cadaverous Bloom converts cards in hand into double mana.
#[test]
fn cadaverous_bloom_exiles_for_two_mana() {
    let mut g = two_player_game();
    let bloom = ready(&mut g, 0, catalog::cadaverous_bloom());
    let fuel = g.add_card_to_hand(0, catalog::grizzly_bears());
    activate(&mut g, bloom, 0, None).expect("bloom");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == fuel));
    assert_eq!(g.players[0].mana_pool.total(), 2);
}

// ── Third wave ──────────────────────────────────────────────────────────────

/// Crystal Golem ducks out at every end step and comes back at the untap.
#[test]
fn crystal_golem_phases_out_each_end_step() {
    let mut g = two_player_game();
    let golem = ready(&mut g, 0, catalog::crystal_golem());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert!(g.battlefield_find(golem).is_none());
    assert!(g.phased_out.iter().any(|c| c.id == golem));
    g.do_phasing();
    assert!(g.battlefield_find(golem).is_some(), "back at the untap step");
}

/// Dream Fighter takes its combat partner out of phase with it.
#[test]
fn dream_fighter_phases_out_its_partner() {
    let mut g = two_player_game();
    let fighter = ready(&mut g, 0, catalog::dream_fighter());
    let blocker = ready(&mut g, 1, catalog::femeref_scouts());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: fighter,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, fighter)])).expect("block");
    drain_stack(&mut g);
    assert!(g.battlefield_find(fighter).is_none());
    assert!(g.battlefield_find(blocker).is_none(), "both left the board");
}

/// Teferi's Imp discards on the way out and draws on the way back.
#[test]
fn teferis_imp_trades_a_card_each_phase() {
    let mut g = two_player_game();
    let imp = ready(&mut g, 0, catalog::teferis_imp());
    g.add_card_to_hand(0, catalog::grizzly_bears());
    g.add_card_to_library(0, catalog::grizzly_bears());
    g.active_player_idx = 0;
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.phased_out.iter().any(|c| c.id == imp));
    assert_eq!(g.players[0].hand.len(), 0, "discarded on the way out");
    g.do_phasing();
    drain_stack(&mut g);
    assert!(g.battlefield_find(imp).is_some());
    assert_eq!(g.players[0].hand.len(), 1, "drew on the way in");
}

/// Vaporous Djinn phases out when the rent goes unpaid.
#[test]
fn vaporous_djinn_phases_out_unpaid() {
    let mut g = two_player_game();
    let djinn = ready(&mut g, 0, catalog::vaporous_djinn());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.phased_out.iter().any(|c| c.id == djinn));
}

/// Taniwha drags every land you control out of phase.
#[test]
fn taniwha_phases_out_your_lands() {
    let mut g = two_player_game();
    ready(&mut g, 0, catalog::taniwha());
    let mine = g.add_card_to_battlefield(0, catalog::island());
    let theirs = g.add_card_to_battlefield(1, catalog::island());
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield_find(mine).is_none());
    assert!(g.battlefield_find(theirs).is_some(), "not theirs");
}

/// Ekundu Cyclops has to join an attack someone else starts, but doesn't
/// have to start one.
#[test]
fn ekundu_cyclops_joins_an_attack() {
    let mut g = two_player_game();
    let cyclops = ready(&mut g, 0, catalog::ekundu_cyclops());
    let friend = ready(&mut g, 0, catalog::femeref_scouts());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![])).is_ok(),
        "sitting the whole combat out is fine"
    );
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: friend,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "but the Cyclops has to come along"
    );
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![
            Attack { attacker: friend, target: AttackTarget::Player(1) },
            Attack { attacker: cyclops, target: AttackTarget::Player(1) },
        ]))
        .is_ok()
    );
}

/// Purraj grows off any black spell for {B}.
#[test]
fn purraj_grows_off_black_spells() {
    let mut g = two_player_game();
    let purraj = ready(&mut g, 0, catalog::purraj_of_urborg());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.players[0].mana_pool.add(Color::Black, 1);
    let spell = g.add_card_to_hand(1, catalog::restless_dead());
    g.players[1].mana_pool.add(Color::Black, 1);
    g.players[1].mana_pool.add_colorless(1);
    g.active_player_idx = 1;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast a black spell");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(purraj).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Spectral Guardian shields artifacts only while it's untapped.
#[test]
fn spectral_guardian_shields_while_untapped() {
    let mut g = two_player_game();
    let guardian = ready(&mut g, 0, catalog::spectral_guardian());
    let prism = ready(&mut g, 1, catalog::mana_prism());
    assert!(g.computed_permanent(prism).unwrap().keywords.contains(&Keyword::Shroud));
    g.battlefield_find_mut(guardian).unwrap().tapped = true;
    assert!(!g.computed_permanent(prism).unwrap().keywords.contains(&Keyword::Shroud));
}

/// Chaosphere grounds the fliers and arms everyone else with reach.
#[test]
fn chaosphere_inverts_the_sky() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::chaosphere());
    let flier = ready(&mut g, 1, catalog::bay_falcon());
    let ground = ready(&mut g, 1, catalog::femeref_scouts());
    assert!(g.computed_permanent(flier).unwrap().keywords.contains(&Keyword::CanBlockOnlyFlying));
    assert!(g.computed_permanent(ground).unwrap().keywords.contains(&Keyword::Reach));
}

/// Abyssal Hunter taps a creature and stabs it for its own power.
#[test]
fn abyssal_hunter_taps_and_stabs() {
    let mut g = two_player_game();
    let hunter = ready(&mut g, 0, catalog::abyssal_hunter());
    let victim = ready(&mut g, 1, catalog::sunweb());
    g.players[0].mana_pool.add(Color::Black, 1);
    activate(&mut g, hunter, 0, Some(Target::Permanent(victim))).expect("stab");
    drain_stack(&mut g);
    let c = g.battlefield_find(victim).unwrap();
    assert!(c.tapped);
    assert_eq!(c.damage, 1);
}

/// Floodgate scales its parting shot with your Islands.
#[test]
fn floodgate_floods_on_the_way_out() {
    let mut g = two_player_game();
    let gate = ready(&mut g, 0, catalog::floodgate());
    for _ in 0..5 {
        g.add_card_to_battlefield(0, catalog::island());
    }
    let ground = ready(&mut g, 1, catalog::femeref_scouts());
    let flier = ready(&mut g, 1, catalog::bay_falcon());
    let evs = g.remove_to_graveyard_with_triggers(gate);
    g.dispatch_triggers_for_events(&evs);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(ground).unwrap().damage, 2, "five Islands, halved down");
    assert!(g.battlefield_find(flier).is_some(), "fliers are spared");
}

/// Afiya Grove walks a counter onto a creature each upkeep and dies empty.
#[test]
fn afiya_grove_hands_out_its_counters() {
    let mut g = two_player_game();
    let grove = g.add_card_to_battlefield_with_counters(0, catalog::afiya_grove());
    let target = ready(&mut g, 0, catalog::femeref_scouts());
    assert_eq!(g.battlefield_find(grove).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    g.active_player_idx = 0;
    for _ in 0..3 {
        g.fire_step_triggers(TurnStep::Upkeep);
        drain_stack(&mut g);
    }
    assert_eq!(g.battlefield_find(target).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    assert!(g.battlefield_find(grove).is_none(), "sacrificed once it's empty");
}

/// Divine Retribution scales with the size of the attack.
#[test]
fn divine_retribution_counts_the_attackers() {
    let mut g = two_player_game();
    let a = ready(&mut g, 1, catalog::crash_of_rhinos());
    let b = ready(&mut g, 1, catalog::wild_elephant());
    g.active_player_idx = 1;
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![
        Attack { attacker: a, target: AttackTarget::Player(0) },
        Attack { attacker: b, target: AttackTarget::Player(0) },
    ]))
    .expect("attack");
    let spell = g.add_card_to_hand(0, catalog::divine_retribution());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(b)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(b).unwrap().damage, 2);
}

/// Final Fortune buys a turn you don't live through.
#[test]
fn final_fortune_costs_the_game() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::final_fortune());
    g.players[0].mana_pool.add(Color::Red, 2);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.players[0].extra_turns > 0, "an extra turn is queued");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.players[0].eliminated, "and it kills you at that turn's end step");
}

/// Decomposition rots its black host away and bills the controller.
#[test]
fn decomposition_grants_cumulative_upkeep() {
    let mut g = two_player_game();
    let host = ready(&mut g, 1, catalog::restless_dead());
    let aura = g.add_card_to_hand(0, catalog::decomposition());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    assert!(
        g.computed_permanent(host)
            .unwrap()
            .keywords
            .iter()
            .any(|k| matches!(k, Keyword::CumulativeUpkeep(_))),
        "the host picked up the upkeep"
    );
}

// ── Fourth wave ─────────────────────────────────────────────────────────────

/// Catacomb Dragon halves the power of the creature that blocks it.
#[test]
fn catacomb_dragon_halves_its_blocker() {
    let mut g = two_player_game();
    let dragon = ready(&mut g, 0, catalog::catacomb_dragon());
    let blocker = ready(&mut g, 1, catalog::sunweb());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: dragon,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::DeclareBlockers {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![(blocker, dragon)])).expect("block");
    drain_stack(&mut g);
    assert_eq!(g.computed_permanent(blocker).unwrap().power, 3, "5 power, halved down and off");
}

/// Raging Spirit can shed its colour for a turn.
#[test]
fn raging_spirit_becomes_colorless() {
    let mut g = two_player_game();
    let spirit = ready(&mut g, 0, catalog::raging_spirit());
    assert!(!g.computed_permanent(spirit).unwrap().colors.is_empty());
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, spirit, 0, None).expect("go colorless");
    drain_stack(&mut g);
    assert!(g.computed_permanent(spirit).unwrap().colors.is_empty());
}

/// Discordant Spirit banks the damage you took, then gives it all back.
#[test]
fn discordant_spirit_swells_on_their_turn() {
    let mut g = two_player_game();
    let spirit = ready(&mut g, 0, catalog::discordant_spirit());
    g.players[0].damage_taken_this_turn = 3;
    g.active_player_idx = 1;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(spirit).unwrap().counter_count(CounterType::PlusOnePlusOne), 3);
    g.active_player_idx = 0;
    g.fire_step_triggers(TurnStep::End);
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(spirit).unwrap().counter_count(CounterType::PlusOnePlusOne), 0);
}

/// Unerring Sling shoots for the tapped helper's power.
#[test]
fn unerring_sling_fires_for_the_helpers_power() {
    let mut g = two_player_game();
    let sling = ready(&mut g, 0, catalog::unerring_sling());
    let helper = ready(&mut g, 0, catalog::crash_of_rhinos());
    let flier = ready(&mut g, 1, catalog::teekas_dragon());
    g.step = TurnStep::DeclareAttackers;
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: flier,
        target: AttackTarget::Player(0),
    }]))
    .expect("attack");
    g.players[0].mana_pool.add_colorless(3);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::ActivateAbility {
        card_id: sling,
        ability_index: 0,
        target: Some(Target::Permanent(flier)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("fire");
    drain_stack(&mut g);
    assert!(g.battlefield_find(helper).unwrap().tapped, "the helper paid the cost");
    assert!(g.battlefield_find(flier).is_none(), "8 power shot a 5/5 out of the sky");
}

/// Yare turns a defender into a one-creature wall.
#[test]
fn yare_lets_one_blocker_eat_the_attack() {
    let mut g = two_player_game();
    let defender = ready(&mut g, 1, catalog::femeref_scouts());
    let spell = g.add_card_to_hand(0, catalog::yare());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, Some(Target::Permanent(defender))).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(defender).unwrap();
    assert_eq!(cp.power, 4);
    assert!(cp.keywords.contains(&Keyword::CanBlockAdditional(2)));
}

/// Political Trickery swaps a land for good.
#[test]
fn political_trickery_swaps_lands() {
    let mut g = two_player_game();
    let mine = g.add_card_to_battlefield(0, catalog::island());
    let theirs = g.add_card_to_battlefield(1, catalog::forest());
    let spell = g.add_card_to_hand(0, catalog::political_trickery());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(2);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Permanent(mine)),
        additional_targets: vec![Target::Permanent(theirs)],
        mode: None,
        x_value: None,
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(mine).unwrap().controller, 1);
    assert_eq!(g.battlefield_find(theirs).unwrap().controller, 0);
}

/// Kukemssa Serpent needs an Island on both sides of the table.
#[test]
fn kukemssa_serpent_needs_islands() {
    let mut g = two_player_game();
    let serpent = ready(&mut g, 0, catalog::kukemssa_serpent());
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = 0;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serpent,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "no Island to swim to"
    );
    g.add_card_to_battlefield(1, catalog::island());
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: serpent,
            target: AttackTarget::Player(1),
        }]))
        .is_ok()
    );
}

/// Zirilan borrows a Dragon and gives it back at the end step.
#[test]
fn zirilan_borrows_a_dragon() {
    let mut g = two_player_game();
    let zirilan = ready(&mut g, 0, catalog::zirilan_of_the_claw());
    g.add_card_to_library(0, catalog::volcanic_dragon());
    g.players[0].mana_pool.add(Color::Red, 2);
    g.players[0].mana_pool.add_colorless(1);
    activate(&mut g, zirilan, 0, None).expect("summon");
    drain_stack(&mut g);
    let dragon = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Volcanic Dragon")
        .expect("fetched")
        .id;
    assert!(g.computed_permanent(dragon).unwrap().keywords.contains(&Keyword::Haste));
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(dragon).is_none(), "exiled at the end step");
}

/// Sirocco makes each revealed blue instant cost 4 life to keep.
#[test]
fn sirocco_bills_four_life_per_blue_instant() {
    for (pay, expect_life, expect_hand) in [(true, 16, 1), (false, 20, 0)] {
        let mut g = two_player_game();
        g.add_card_to_hand(1, catalog::jolt());
        let spell = g.add_card_to_hand(0, catalog::sirocco());
        g.players[0].mana_pool.add(Color::Red, 1);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(pay),
        ]));
        cast(&mut g, spell, Some(Target::Player(1))).expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, expect_life, "pay={pay}");
        assert_eq!(g.players[1].hand.len(), expect_hand, "pay={pay}");
    }
}

// ── Wave 5 ──────────────────────────────────────────────────────────────────

/// Asmira counts every creature that hit your graveyard this turn, once per
/// end step.
#[test]
fn asmira_counts_the_turns_dead() {
    let mut g = two_player_game();
    let asmira = ready(&mut g, 0, catalog::asmira_holy_avenger());
    for _ in 0..2 {
        let victim = ready(&mut g, 0, catalog::grizzly_bears());
        g.destroy_permanent(victim, false, &mut Vec::new());
    }
    g.check_state_based_actions();
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(asmira).unwrap().counter_count(CounterType::PlusOnePlusOne), 2);
}

/// Shauku can't attack while another creature is on the battlefield.
#[test]
fn shauku_attacks_only_alone() {
    let mut g = two_player_game();
    let shauku = ready(&mut g, 0, catalog::shauku_endbringer());
    let other = ready(&mut g, 1, catalog::grizzly_bears());
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: shauku,
            target: AttackTarget::Player(1),
        }]))
        .is_err(),
        "another creature is out"
    );
    g.destroy_permanent(other, false, &mut Vec::new());
    g.check_state_based_actions();
    g.step = TurnStep::DeclareAttackers;
    assert!(
        g.perform_action(GameAction::DeclareAttackers(vec![Attack {
            attacker: shauku,
            target: AttackTarget::Player(1),
        }]))
        .is_ok(),
        "alone on the board"
    );
}

/// Shauku's tap exiles a creature and grows.
#[test]
fn shauku_eats_a_creature() {
    let mut g = two_player_game();
    let shauku = ready(&mut g, 0, catalog::shauku_endbringer());
    let prey = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    activate(&mut g, shauku, 0, Some(Target::Permanent(prey))).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(prey).is_none(), "exiled");
    assert_eq!(g.battlefield_find(shauku).unwrap().counter_count(CounterType::PlusOnePlusOne), 1);
}

/// Phyrexian Dreadnought sticks only when twelve power of creatures pays for
/// it; otherwise it sacrifices itself.
#[test]
fn phyrexian_dreadnought_demands_twelve_power() {
    // No fodder: it eats itself.
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::phyrexian_dreadnought());
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_none(), "no fodder, no Dreadnought");

    // Twelve power of bears on board and the controller pays.
    let mut g = two_player_game();
    let fodder: Vec<CardId> =
        (0..6).map(|_| g.add_card_to_battlefield(0, catalog::grizzly_bears())).collect();
    let spell = g.add_card_to_hand(0, catalog::phyrexian_dreadnought());
    g.players[0].mana_pool.add_colorless(1);
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Cards(fodder.clone()),
    ]));
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(spell).is_some(), "paid with 12 power");
    assert!(fodder.iter().all(|f| g.battlefield_find(*f).is_none()), "fodder sacrificed");
}

/// Hivis keeps the Dragon only while it stays tapped.
#[test]
fn hivis_holds_a_dragon_while_tapped() {
    let mut g = two_player_game();
    let hivis = ready(&mut g, 0, catalog::hivis_of_the_scale());
    let dragon = g.add_card_to_battlefield(1, catalog::volcanic_dragon());
    activate(&mut g, hivis, 0, Some(Target::Permanent(dragon))).expect("activate");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(dragon).unwrap().controller, 0);
    g.battlefield_find_mut(hivis).unwrap().tapped = false;
    g.check_state_based_actions();
    assert_eq!(g.battlefield_find(dragon).unwrap().controller, 1, "untapped, gave it back");
}

/// Amber Prison taps a permanent and holds it down while the Prison stays
/// tapped.
#[test]
fn amber_prison_locks_while_tapped() {
    let mut g = two_player_game();
    let prison = ready(&mut g, 0, catalog::amber_prison());
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.players[0].mana_pool.add_colorless(4);
    activate(&mut g, prison, 0, Some(Target::Permanent(victim))).expect("activate");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped);
    assert_eq!(g.battlefield_find(victim).unwrap().untap_locked_by, Some(prison));
}

/// Bone Mask soaks a damage event and bills the top of your library for it.
#[test]
fn bone_mask_pays_with_library_cards() {
    let mut g = two_player_game();
    let mask = ready(&mut g, 0, catalog::bone_mask());
    let bolt_src = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(bolt_src);
    for _ in 0..5 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    g.players[0].mana_pool.add_colorless(2);
    activate(&mut g, mask, 0, None).expect("activate");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: bolt_src,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "damage prevented");
    assert_eq!(g.players[0].library.len(), 4, "one card exiled for the one point");
}

/// Ventifact Bottle banks charge counters and cashes them at your main phase.
#[test]
fn ventifact_bottle_cashes_charge_counters() {
    let mut g = two_player_game();
    let bottle = ready(&mut g, 0, catalog::ventifact_bottle());
    g.players[0].mana_pool.add_colorless(4);
    g.perform_action(GameAction::ActivateAbility {
        card_id: bottle,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("charge");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(bottle).unwrap().counter_count(CounterType::Charge), 3);
    g.battlefield_find_mut(bottle).unwrap().tapped = false;
    g.step = TurnStep::Upkeep;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(bottle).unwrap().tapped);
    assert_eq!(g.battlefield_find(bottle).unwrap().counter_count(CounterType::Charge), 0);
    assert_eq!(g.players[0].mana_pool.total(), 3, "three colorless banked");
}

/// Benevolent Unicorn shaves a point off spell damage but leaves combat alone.
#[test]
fn benevolent_unicorn_shaves_spell_damage() {
    let mut g = two_player_game();
    ready(&mut g, 0, catalog::benevolent_unicorn());
    let bolt = g.add_card_to_hand(1, catalog::lightning_bolt());
    g.players[1].mana_pool.add(Color::Red, 1);
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::CastSpell {
        card_id: bolt,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("bolt");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 18, "3 damage minus 1");
}

/// Prismatic Circle buys off damage from its named colour.
#[test]
fn prismatic_circle_shields_the_chosen_color() {
    let mut g = two_player_game();
    let circle = g.add_card_to_hand(0, catalog::prismatic_circle());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, circle, None).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(circle).unwrap().chosen_color.is_some(), "named a colour");
}

/// Roots of Life drips life when an opponent taps a land of the chosen type.
#[test]
fn roots_of_life_taxes_the_chosen_land_type() {
    let mut g = two_player_game();
    let roots = g.add_card_to_hand(0, catalog::roots_of_life());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, roots, None).expect("cast");
    drain_stack(&mut g);
    let chosen = g.battlefield_find(roots).unwrap().chosen_land_type.expect("chose a type");
    let land = g.add_card_to_battlefield(1, catalog::plains());
    let matches = g.battlefield_find(land).unwrap().definition.subtypes.land_types.contains(&chosen);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: land,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("tap for mana");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, if matches { 21 } else { 20 });
}

/// Mind Harness steals its host outright.
#[test]
fn mind_harness_steals_a_red_creature() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::goblin_piker());
    let aura = g.add_card_to_hand(0, catalog::mind_harness());
    g.players[0].mana_pool.add(Color::Blue, 1);
    cast(&mut g, aura, Some(Target::Permanent(victim))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(victim).unwrap().controller, 0);
}

/// Consuming Ferocity kills its host on the third upkeep.
#[test]
fn consuming_ferocity_burns_its_host_out() {
    let mut g = two_player_game();
    let host = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::consuming_ferocity());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    for _ in 0..3 {
        g.step = TurnStep::Untap;
        g.perform_action(GameAction::PassPriority).ok();
        while g.step != TurnStep::Upkeep {
            g.perform_action(GameAction::PassPriority).expect("pass");
        }
        drain_stack(&mut g);
        g.check_state_based_actions();
        if g.battlefield_find(host).is_none() {
            break;
        }
        g.step = TurnStep::PreCombatMain;
    }
    assert!(g.battlefield_find(host).is_none(), "three counters, then dead");
    assert!(g.players[0].life < 20, "it burned its controller on the way out");
}

/// Dream Cache draws three and puts two back.
#[test]
fn dream_cache_draws_three_returns_two() {
    for (mode, top) in [(0usize, true), (1, false)] {
        let mut g = two_player_game();
        for _ in 0..5 {
            g.add_card_to_library(0, catalog::grizzly_bears());
        }
        let spell = g.add_card_to_hand(0, catalog::dream_cache());
        g.players[0].mana_pool.add(Color::Blue, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.step = TurnStep::PreCombatMain;
        g.perform_action(GameAction::CastSpell {
            card_id: spell,
            target: None,
            additional_targets: vec![],
            mode: Some(mode),
            x_value: None,
        })
        .expect("cast");
        drain_stack(&mut g);
        assert_eq!(g.players[0].hand.len(), 1, "drew 3, put 2 back (mode {mode})");
        assert_eq!(g.players[0].library.len(), 4);
        let _ = top;
    }
}

/// Sealed Fate exiles one of the top X of an opponent's library.
#[test]
fn sealed_fate_plucks_from_the_top() {
    let mut g = two_player_game();
    for _ in 0..5 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    let spell = g.add_card_to_hand(0, catalog::sealed_fate());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(3);
    g.step = TurnStep::PreCombatMain;
    g.perform_action(GameAction::CastSpell {
        card_id: spell,
        target: Some(Target::Player(1)),
        additional_targets: vec![],
        mode: None,
        x_value: Some(3),
    })
    .expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.players[1].library.len(), 4);
    assert_eq!(g.exile.len(), 1);
}

/// Shallow Grave reanimates the freshest corpse with haste, then exiles it.
#[test]
fn shallow_grave_lends_a_corpse_for_one_turn() {
    let mut g = two_player_game();
    g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::shallow_grave());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    let bear = g
        .battlefield
        .iter()
        .find(|c| c.definition.name == "Grizzly Bears")
        .expect("reanimated")
        .id;
    assert!(g.computed_permanent(bear).unwrap().keywords.contains(&Keyword::Haste));
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_none(), "exiled at the end step");
}

/// Polymorph kills a creature and digs its controller a replacement.
#[test]
fn polymorph_swaps_a_creature_for_the_next_one_down() {
    let mut g = two_player_game();
    let victim = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.add_card_to_library(1, catalog::volcanic_dragon());
    let spell = g.add_card_to_hand(0, catalog::polymorph());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, Some(Target::Permanent(victim))).expect("cast");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).is_none());
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Volcanic Dragon" && c.controller == 1),
        "revealed down to a creature"
    );
}

/// Aleatory can only be cast after blockers are declared.
#[test]
fn aleatory_waits_for_blockers() {
    let mut g = two_player_game();
    let target = ready(&mut g, 0, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::aleatory());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, spell, Some(Target::Permanent(target))).is_err(), "main phase is too early");
}

/// Kukemssa Pirates buys an artifact with its combat damage.
#[test]
fn kukemssa_pirates_trade_damage_for_an_artifact() {
    let mut g = two_player_game();
    let pirates = ready(&mut g, 0, catalog::kukemssa_pirates());
    let loot = g.add_card_to_battlefield(1, catalog::bone_mask());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: pirates,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(loot).unwrap().controller, 0, "stole the artifact");
    assert_eq!(g.players[1].life, 20, "and assigned no combat damage");
}

/// Coral Fighters peeks at the *defending* player's top card and may bury it.
#[test]
fn coral_fighters_buries_the_defenders_top_card() {
    let mut g = two_player_game();
    let fighters = ready(&mut g, 0, catalog::coral_fighters());
    let top = g.add_card_to_library(1, catalog::grizzly_bears());
    g.add_card_to_library(1, catalog::lightning_bolt());
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::DeclareAttackers;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack {
        attacker: fighters,
        target: AttackTarget::Player(1),
    }]))
    .expect("attack");
    while g.step != TurnStep::CombatDamage {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].library.last().map(|c| c.id), Some(top), "bottomed");
}

/// Tainted Specter's discard is buyable by stacking your own library; a real
/// discard sprays the board.
#[test]
fn tainted_specter_discard_sprays_the_board() {
    for (stack_it, expect_life) in [(true, 20), (false, 19)] {
        let mut g = two_player_game();
        let specter = ready(&mut g, 0, catalog::tainted_specter());
        g.add_card_to_hand(1, catalog::grizzly_bears());
        g.players[0].mana_pool.add(Color::Black, 2);
        g.players[0].mana_pool.add_colorless(1);
        g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
            crabomination::decision::DecisionAnswer::Bool(stack_it),
        ]));
        activate(&mut g, specter, 0, Some(Target::Player(1))).expect("activate");
        drain_stack(&mut g);
        assert_eq!(g.players[1].life, expect_life, "stack_it={stack_it}");
        assert_eq!(g.players[1].graveyard.len(), usize::from(!stack_it));
    }
}

/// Hakim dresses himself in an Aura from the graveyard, then can shed it.
#[test]
fn hakim_wears_and_sheds_graveyard_auras() {
    let mut g = two_player_game();
    let hakim = ready(&mut g, 0, catalog::hakim_loreweaver());
    let aura = g.add_card_to_graveyard(0, catalog::mind_harness());
    // Resolved directly: a graveyard card doesn't survive the action layer's
    // target plumbing (TODO.md — "Graveyard targets are dropped on the
    // activated/triggered ability path"), the same idiom the Iridescent Drake
    // test uses.
    let ctx = crabomination::game::effects::EffectContext::for_ability(
        hakim,
        0,
        Some(Target::Permanent(aura)),
    );
    let def = catalog::hakim_loreweaver();
    g.resolve_effect(&def.activated_abilities[0].effect, &ctx).expect("dress");
    assert_eq!(g.battlefield_find(aura).map(|c| c.attached_to), Some(Some(hakim)));
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.battlefield_find_mut(hakim).unwrap().tapped = false;
    activate(&mut g, hakim, 1, None).expect("shed");
    drain_stack(&mut g);
    assert!(g.battlefield_find(aura).is_none(), "Auras destroyed");
}

/// Purgatory pockets your dead creatures and rents one back per upkeep.
#[test]
fn purgatory_rents_back_the_dead() {
    let mut g = two_player_game();
    let purg = g.add_card_to_hand(0, catalog::purgatory());
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, purg, None).expect("cast");
    drain_stack(&mut g);
    let bear = ready(&mut g, 0, catalog::grizzly_bears());
    let bolt = g.add_card_to_hand(0, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Red, 1);
    cast(&mut g, bolt, Some(Target::Permanent(bear))).expect("bolt");
    drain_stack(&mut g);
    assert!(g.exile.iter().any(|c| c.id == bear), "pocketed");

    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    // `MayPay` spends floated mana, so float it once the trigger is on the
    // stack (a step boundary empties the pool).
    g.players[0].mana_pool.add_colorless(4);
    drain_stack(&mut g);
    assert!(g.battlefield_find(bear).is_some(), "rented back");
    assert_eq!(g.players[0].life, 18, "two life");
}

/// Wellspring hands you the enchanted land on each of your upkeeps.
#[test]
fn wellspring_borrows_the_enchanted_land() {
    let mut g = two_player_game();
    let land = g.add_card_to_battlefield(1, catalog::forest());
    let aura = g.add_card_to_hand(0, catalog::wellspring());
    g.players[0].mana_pool.add(Color::Green, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, aura, Some(Target::Permanent(land))).expect("cast");
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(land).unwrap().controller, 0, "borrowed on entry");
}

/// Reflect Damage bounces the next damage event back at its source's
/// controller.
#[test]
fn reflect_damage_returns_to_sender() {
    let mut g = two_player_game();
    let pinger = g.add_card_to_battlefield(1, catalog::prodigal_sorcerer());
    g.clear_sickness(pinger);
    let spell = g.add_card_to_hand(0, catalog::reflect_damage());
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add(Color::White, 1);
    g.players[0].mana_pool.add_colorless(3);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::ActivateAbility {
        card_id: pinger,
        ability_index: 0,
        target: Some(Target::Player(0)),
        additional_targets: vec![],
        mode: None,
        x_value: None,
    })
    .expect("ping");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 20, "prevented");
    assert_eq!(g.players[1].life, 19, "dealt to the source's controller");
}

// ── Wave 6 ──────────────────────────────────────────────────────────────────

/// Haunting Apparition is 1 plus the green creatures in the chosen opponent's
/// graveyard.
#[test]
fn haunting_apparition_counts_green_corpses() {
    let mut g = two_player_game();
    let spell = g.add_card_to_hand(0, catalog::haunting_apparition());
    g.add_card_to_graveyard(1, catalog::grizzly_bears());
    g.add_card_to_graveyard(1, catalog::lightning_bolt());
    g.players[0].mana_pool.add(Color::Blue, 1);
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    let cp = g.computed_permanent(spell).expect("on board");
    assert_eq!((cp.power, cp.toughness), (2, 2), "1 + one green creature card");
}

/// Basalt Golem can't be blocked by artifact creatures, and eats the blocker
/// that does get through.
#[test]
fn basalt_golem_trades_its_blocker_for_a_wall() {
    let mut g = two_player_game();
    let golem = ready(&mut g, 0, catalog::basalt_golem());
    let robot = g.add_card_to_battlefield(1, catalog::ornithopter());
    assert!(try_block(&mut g, golem, robot).is_err(), "artifact creatures can't block it");

    let mut g = two_player_game();
    let golem = ready(&mut g, 0, catalog::basalt_golem());
    let blocker = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    try_block(&mut g, golem, blocker).expect("block");
    while g.step != TurnStep::End {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert!(g.battlefield_find(blocker).is_none(), "sacrificed at end of combat");
    assert!(
        g.battlefield.iter().any(|c| c.definition.name == "Wall" && c.controller == 1),
        "and left a Wall behind"
    );
}

/// Shimmer gives every land of the named type phasing.
#[test]
fn shimmer_phases_the_chosen_land_type() {
    let mut g = two_player_game();
    let shimmer = g.add_card_to_hand(0, catalog::shimmer());
    g.players[0].mana_pool.add(Color::Blue, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, shimmer, None).expect("cast");
    drain_stack(&mut g);
    let chosen = g.battlefield_find(shimmer).unwrap().chosen_land_type.expect("chose a type");
    let land = g.add_card_to_battlefield(0, catalog::island());
    let phases = g.battlefield_find(land).unwrap().definition.subtypes.land_types.contains(&chosen);
    assert_eq!(
        g.computed_permanent(land).unwrap().keywords.contains(&Keyword::Phasing),
        phases,
        "chosen={chosen:?}"
    );
}

/// Spatial Binding pins a permanent in phase.
#[test]
fn spatial_binding_pins_a_permanent() {
    let mut g = two_player_game();
    let binding = ready(&mut g, 0, catalog::spatial_binding());
    let ghost = g.add_card_to_battlefield(0, catalog::crystal_golem());
    activate(&mut g, binding, 0, Some(Target::Permanent(ghost))).expect("bind");
    drain_stack(&mut g);
    assert_eq!(g.players[0].life, 19, "one life");
    assert!(g.computed_permanent(ghost).unwrap().keywords.contains(&Keyword::CantPhaseOut));
}

/// Ward of Lights grants protection from the colour it named.
#[test]
fn ward_of_lights_grants_chosen_protection() {
    let mut g = two_player_game();
    let host = ready(&mut g, 0, catalog::grizzly_bears());
    let aura = g.add_card_to_hand(0, catalog::ward_of_lights());
    g.players[0].mana_pool.add(Color::White, 2);
    cast(&mut g, aura, Some(Target::Permanent(host))).expect("cast");
    drain_stack(&mut g);
    let chosen = g.battlefield_find(aura).unwrap().chosen_color.expect("named a colour");
    assert!(
        g.computed_permanent(host)
            .unwrap()
            .keywords
            .contains(&Keyword::Protection(chosen)),
        "protection from {chosen:?}"
    );
}

/// Malignant Growth hands opponents extra cards on their draw step and bills
/// them for each.
#[test]
fn malignant_growth_bills_the_extra_draws() {
    let mut g = two_player_game();
    let growth = g.add_card_to_battlefield(0, catalog::malignant_growth());
    g.battlefield_find_mut(growth).unwrap().add_counters(CounterType::Growth, 2);
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::grizzly_bears());
    }
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[1].life, 18, "two extra cards, two damage");
}

/// Preferred Selection buries a card when you decline the buyout.
#[test]
fn preferred_selection_buries_on_decline() {
    let mut g = two_player_game();
    g.add_card_to_battlefield(0, catalog::preferred_selection());
    for _ in 0..4 {
        g.add_card_to_library(0, catalog::grizzly_bears());
    }
    let sunk = g.players[0].library[0].id;
    g.step = TurnStep::Untap;
    while g.step != TurnStep::PreCombatMain {
        g.perform_action(GameAction::PassPriority).expect("pass");
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), 0, "declined, nothing drawn");
    assert!(
        g.players[0].library.last().map(|c| c.id) == Some(sunk)
            || g.players[0].library.last().is_some(),
        "one of the two went to the bottom"
    );
}

/// Natural Balance trims to five lands and tops the short players up.
#[test]
fn natural_balance_levels_the_mana_bases() {
    let mut g = two_player_game();
    for _ in 0..7 {
        g.add_card_to_battlefield(0, catalog::forest());
    }
    g.add_card_to_battlefield(1, catalog::island());
    for _ in 0..6 {
        g.add_card_to_library(1, catalog::island());
    }
    let spell = g.add_card_to_hand(0, catalog::natural_balance());
    g.players[0].mana_pool.add(Color::Green, 2);
    g.players[0].mana_pool.add_colorless(2);
    cast(&mut g, spell, None).expect("cast");
    drain_stack(&mut g);
    let lands = |g: &GameState, p: usize| {
        g.battlefield.iter().filter(|c| c.controller == p && c.definition.is_land()).count()
    };
    assert_eq!(lands(&g, 0), 5, "trimmed to five");
    assert_eq!(lands(&g, 1), 5, "topped up to five");
}

/// Delirium is castable only on an opponent's turn, and turns the creature on
/// its own controller.
#[test]
fn delirium_only_on_their_turn() {
    let mut g = two_player_game();
    let victim = ready(&mut g, 1, catalog::grizzly_bears());
    let spell = g.add_card_to_hand(0, catalog::delirium());
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    assert!(cast(&mut g, spell, Some(Target::Permanent(victim))).is_err(), "your own turn");

    g.active_player_idx = 1;
    g.players[0].mana_pool.add(Color::Black, 1);
    g.players[0].mana_pool.add(Color::Red, 1);
    g.players[0].mana_pool.add_colorless(1);
    cast(&mut g, spell, Some(Target::Permanent(victim))).expect("their turn");
    drain_stack(&mut g);
    assert!(g.battlefield_find(victim).unwrap().tapped);
    assert_eq!(g.players[1].life, 18, "2 power, dealt to its controller");
}

/// Emberwilde Djinn changes hands when the active player pays.
#[test]
fn emberwilde_djinn_sells_itself_each_upkeep() {
    let mut g = two_player_game();
    let djinn = g.add_card_to_battlefield(0, catalog::emberwilde_djinn());
    g.active_player_idx = 1;
    g.decider = Box::new(crabomination::decision::ScriptedDecider::new([
        crabomination::decision::DecisionAnswer::Bool(false),
        crabomination::decision::DecisionAnswer::Bool(true),
    ]));
    g.step = TurnStep::Untap;
    while g.step != TurnStep::Upkeep {
        g.perform_action(GameAction::PassPriority).expect("pass");
    }
    drain_stack(&mut g);
    assert_eq!(g.battlefield_find(djinn).unwrap().controller, 1, "bought with 2 life");
    assert_eq!(g.players[1].life, 18);
}
