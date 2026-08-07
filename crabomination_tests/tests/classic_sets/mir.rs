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
