//! Reality Fracture puzzles: the FRA benchmark set
//! (`.ladder/fra_puzzles/PUZZLES.md` is the spec — position, solution and
//! trap per item). Same contract as [`super::puzzle_corpus`]: a candidate is
//! a puzzle only if [`super::puzzle::solve`] finds a line and the default
//! heuristic doesn't (`bot_puzzles --set fra --certify`).
//!
//! Every position uses the real catalog cards: the point of the set is FRA
//! rules the net's features can't express (277 of FRA's 280 names were off
//! the champion's vocabulary), so a hand-made stand-in would test nothing.
//! Filler bodies are vanilla, coloured by their cost where a colour matters.

use crate::card::{CardDefinition, CardId, CardType, CounterType};
use crate::catalog;
use crate::effect::{Effect, PlayerRef};
use crate::game::types::TurnStep;
use crate::game::{EffectContext, GameAction, GameState, two_player_game};
use crate::mana::{ManaCost, b, cost, g as green, generic, r};

use super::puzzle::Goal;
use super::puzzle_corpus::{Mechanic, Puzzle};

fn vanilla(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Creature], power: p, toughness: t, ..Default::default() }
}

fn colorless(name: &'static str, p: i32, t: i32) -> CardDefinition {
    vanilla(name, ManaCost::default(), p, t)
}

/// Seat 0's precombat main on turn 1, both libraries stocked with 20
/// Islands, priority with seat 0. Set directly rather than passed into, so
/// no draw step adds a land to the hand and changes a puzzle's mana.
fn base() -> GameState {
    let mut g = two_player_game();
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::island());
        }
    }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    // The engine-read half of the default pilot (`hostile_player_targets`,
    // adopted r67): the play loop pushes it onto each seat, and a puzzle
    // isn't played through that loop.
    for seat in 0..2 {
        g.players[seat].hostile_player_targets = true;
    }
    g
}

fn ready(g: &mut GameState, seat: usize, def: CardDefinition) -> CardId {
    let id = g.add_card_to_battlefield(seat, def);
    g.clear_sickness(id);
    id
}

fn lands(g: &mut GameState, seat: usize, f: fn() -> CardDefinition, n: usize) {
    for _ in 0..n {
        ready(g, seat, f());
    }
}

fn set_loyalty(g: &mut GameState, id: CardId, n: u32) {
    let c = g.battlefield_find_mut(id).expect("planeswalker");
    let have = c.counter_count(CounterType::Loyalty);
    c.remove_counters(CounterType::Loyalty, have);
    c.add_counters(CounterType::Loyalty, n);
}

/// A Jace token with `n` loyalty, made the way the cards make it.
fn jace_token(g: &mut GameState, n: i32) {
    let e = Effect::EmpowerJace { who: PlayerRef::You, count: crate::card::Value::Const(n) };
    g.resolve_effect(&e, &EffectContext::for_spell(0, None, 0, 0)).expect("empower");
}

/// Pass priority until `step` (both seats passing).
fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut fuel = 50;
    while g.step != step && fuel > 0 {
        fuel -= 1;
        if g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
}

fn fp1_tetsuko_pump_after_blocks() -> GameState {
    let mut g = base();
    g.players[1].life = 7;
    ready(&mut g, 0, catalog::tetsuko_umezawa_fugitive());
    ready(&mut g, 0, colorless("Illusion", 1, 1));
    ready(&mut g, 0, colorless("Illusion", 1, 1));
    lands(&mut g, 0, catalog::mountain, 3);
    g.add_card_to_hand(0, catalog::charge_the_sanctum());
    for _ in 0..3 {
        ready(&mut g, 1, vanilla("Ogre", cost(&[generic(2), r()]), 3, 3));
    }
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g
}

fn fp2_yuriko_attack_alone() -> GameState {
    let mut g = base();
    g.players[1].life = 10;
    ready(&mut g, 0, catalog::yuriko_blade_of_the_mighty());
    ready(&mut g, 0, colorless("Brute", 5, 5));
    ready(&mut g, 0, colorless("Squire", 1, 1));
    ready(&mut g, 0, colorless("Squire", 1, 1));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g
}

fn fp3_loot_negative_power() -> GameState {
    let mut g = base();
    g.players[1].life = 4;
    ready(&mut g, 0, catalog::loot_the_anomaly());
    ready(&mut g, 0, colorless("Fodder", 1, 1));
    for _ in 0..7 {
        g.add_card_to_graveyard(0, catalog::island());
    }
    g
}

fn fp4_ghalta_toughness_damage() -> GameState {
    let mut g = base();
    g.players[1].life = 9;
    ready(&mut g, 0, catalog::yargle_goliath_of_otaria());
    lands(&mut g, 0, catalog::plains, 1);
    g.add_card_to_hand(0, catalog::ghalta_the_immovable());
    g
}

fn fp5_ingris_haste_cadets() -> GameState {
    let mut g = base();
    g.players[1].life = 3;
    ready(&mut g, 0, catalog::ingris_stingerquill());
    lands(&mut g, 0, catalog::mountain, 8);
    g.add_card_to_hand(0, catalog::heartstring_puller());
    for _ in 0..3 {
        let mut wall = colorless("Wall", 0, 5);
        wall.keywords = vec![crate::card::Keyword::Defender, crate::card::Keyword::Reach];
        ready(&mut g, 1, wall);
    }
    g
}

fn fp6_darklight_phoenix_main_one() -> GameState {
    let mut g = base();
    g.players[1].life = 3;
    g.add_card_to_graveyard(0, catalog::darklight_phoenix());
    lands(&mut g, 0, catalog::mountain, 3);
    g.add_card_to_hand(0, catalog::fulminous_forte());
    ready(&mut g, 1, colorless("Goblin", 1, 1));
    ready(&mut g, 1, colorless("Goblin", 1, 1));
    g
}

fn fp7_germinate_after_lifegain() -> GameState {
    let mut g = base();
    g.players[1].life = 6;
    ready(&mut g, 0, catalog::samut_hazorets_champion());
    lands(&mut g, 0, catalog::plains, 5);
    g.add_card_to_hand(0, catalog::surgical_precision());
    g.add_card_to_hand(0, catalog::germinate_recruits());
    g
}

fn fp8_overwrite_into_warlord() -> GameState {
    let mut g = base();
    g.players[1].life = 2;
    g.add_card_to_battlefield(0, catalog::way_of_the_warlord());
    lands(&mut g, 0, catalog::swamp, 6);
    g.add_card_to_hand(0, catalog::overwrite_the_multiverse());
    for _ in 0..4 {
        ready(&mut g, 1, colorless("Soldier", 3, 3));
    }
    g
}

fn fp9_sculptor_shield() -> GameState {
    let mut g = base();
    g.players[0].life = 3;
    let jace = g.add_card_to_battlefield(0, catalog::jace_reality_sculptor());
    set_loyalty(&mut g, jace, 4);
    lands(&mut g, 0, catalog::island, 5);
    for _ in 0..3 {
        ready(&mut g, 1, colorless("Ogre", 4, 4));
    }
    g
}

fn fp10_chandra_stun_the_threat() -> GameState {
    let mut g = base();
    g.players[0].life = 6;
    let chandra = g.add_card_to_battlefield(0, catalog::chandra_chill_of_compliance());
    // One loyalty, and a 1/1 beside the 8/8: the opponent can kill Chandra
    // with the 1/1 and still swing lethal, so a passive hero dies. (At three
    // loyalty and no 1/1 the default opponent spent the 8/8 on Chandra and a
    // do-nothing hero survived — the `pass` control caught it.)
    set_loyalty(&mut g, chandra, 1);
    lands(&mut g, 0, catalog::island, 3);
    let mut wurm = colorless("Wurm", 8, 8);
    wurm.keywords = vec![crate::card::Keyword::Trample];
    ready(&mut g, 1, wurm);
    ready(&mut g, 1, colorless("Imp", 1, 1));
    g
}

fn fp11_kiora_activate_first() -> GameState {
    let mut g = base();
    g.players[1].life = 5;
    ready(&mut g, 0, catalog::kiora_of_salt_and_sand());
    ready(&mut g, 0, colorless("Brute", 5, 5));
    jace_token(&mut g, 2);
    ready(&mut g, 1, colorless("Giant", 6, 6));
    g
}

/// Four cards left. Theorix Charm's third mode (mill three, then draw)
/// empties the library; Fblthp's combat-damage draw then finds it empty and
/// wins. (The spec's first version surveiled with a Jace token instead —
/// a decision answer, which the solver doesn't search; a mode is an action.)
fn fp12_fblthp_empty_library() -> GameState {
    let mut g = base();
    g.players[0].library.truncate(4);
    ready(&mut g, 0, catalog::fblthp_impossibly_lost());
    lands(&mut g, 0, catalog::island, 1);
    lands(&mut g, 0, catalog::swamp, 1);
    g.add_card_to_hand(0, catalog::theorix_charm());
    g
}

fn fp13_ferocity_exile_assignment() -> GameState {
    let mut g = base();
    lands(&mut g, 0, catalog::mountain, 2);
    lands(&mut g, 0, catalog::swamp, 2);
    g.add_card_to_hand(0, catalog::essence_burn());
    g.add_card_to_hand(0, catalog::last_gasp());
    let bear = ready(&mut g, 1, vanilla("Wild Bear", cost(&[generic(2), green()]), 3, 3));
    let aura = g.add_card_to_battlefield(1, catalog::ferocity_of_the_hunt());
    g.battlefield_find_mut(aura).unwrap().attached_to = Some(bear);
    ready(&mut g, 1, vanilla("Grave Ghoul", cost(&[generic(2), b()]), 3, 3));
    g
}

fn fp14_stingcaster_flashback_burn() -> GameState {
    let mut g = base();
    g.players[1].life = 5;
    lands(&mut g, 0, catalog::mountain, 4);
    g.add_card_to_graveyard(0, catalog::no_admittance());
    g.add_card_to_hand(0, catalog::stingcaster_mage());
    g.add_card_to_hand(0, catalog::heartstring_puller());
    g
}

fn fp15_face_yourself_target_them() -> GameState {
    let mut g = base();
    g.players[1].life = 10;
    lands(&mut g, 0, catalog::mountain, 7);
    g.add_card_to_hand(0, catalog::face_yourself());
    for (n, p) in [("Big Beast", 4), ("Beast", 3), ("Beast", 3)] {
        let id = ready(&mut g, 1, colorless(n, p, p));
        g.battlefield_find_mut(id).unwrap().tapped = true;
    }
    g
}

fn fp16_sanctum_lurker_ping() -> GameState {
    let mut g = base();
    g.players[1].life = 4;
    ready(&mut g, 0, catalog::sanctum_lurker());
    lands(&mut g, 0, catalog::mountain, 2);
    g.add_card_to_hand(0, catalog::no_admittance());
    ready(&mut g, 1, colorless("Giant", 5, 5));
    g
}

/// The FRA candidates, in the spec's order.
pub fn candidates() -> Vec<Puzzle> {
    use Goal::*;
    use Mechanic::*;
    let p = |id, mechanic, goal, build, prompt| Puzzle { id, mechanic, goal, seat: 0, build, prompt };
    vec![
        p("fp1_tetsuko_pump_after_blocks", Trick, WinThisTurn, fp1_tetsuko_pump_after_blocks as fn() -> GameState,
          "Tetsuko + two 1/1s vs three 3/3s, opp at 7: pump after blocks"),
        p("fp2_yuriko_attack_alone", Combat, WinThisTurn, fp2_yuriko_attack_alone, "Yuriko: the 5/5 attacks alone for 10"),
        p("fp3_loot_negative_power", Combat, WinThisTurn, fp3_loot_negative_power, "Loot: sacrifice the 1/1, −4 power deals 4"),
        p("fp4_ghalta_toughness_damage", Sequencing, WinThisTurn, fp4_ghalta_toughness_damage,
          "Ghalta for {W} before combat; Yargle assigns 9"),
        p("fp5_ingris_haste_cadets", Mana, WinThisTurn, fp5_ingris_haste_cadets, "Ingris: three attackers ping for 3"),
        p("fp6_darklight_phoenix_main_one", Sequencing, WinThisTurn, fp6_darklight_phoenix_main_one,
          "Forte the two 1/1s in main 1; the Phoenix returns at combat"),
        p("fp7_germinate_after_lifegain", Sequencing, WinThisTurn, fp7_germinate_after_lifegain,
          "gain life first, then Germinate; Samut gives haste"),
        p("fp8_overwrite_into_warlord", Loyalty, WinThisTurn, fp8_overwrite_into_warlord,
          "Overwrite four creatures → Jace 4 → Warlord's −4 for 2"),
        p("fp9_sculptor_shield", Horizon, SurviveWithin(1), fp9_sculptor_shield, "Jace −3: attackers get −5/−0"),
        p("fp10_chandra_stun_the_threat", Horizon, SurviveWithin(1), fp10_chandra_stun_the_threat,
          "Chandra −X stuns the 8/8"),
        p("fp11_kiora_activate_first", Loyalty, WinThisTurn, fp11_kiora_activate_first,
          "a loyalty ability first, then Kiora unblocks the 5/5"),
        p("fp12_fblthp_empty_library", Graveyard, WinThisTurn, fp12_fblthp_empty_library,
          "Theorix Charm mills three and draws; Fblthp draws the empty library"),
        p("fp13_ferocity_exile_assignment", Removal, ClearOpposingCreatures, fp13_ferocity_exile_assignment,
          "Essence Burn the Ferocity creature, Last Gasp the other"),
        p("fp14_stingcaster_flashback_burn", Graveyard, WinThisTurn, fp14_stingcaster_flashback_burn,
          "Stingcaster Mage flashes back No Admittance: 3 + 2"),
        p("fp15_face_yourself_target_them", Targeting, WinThisTurn, fp15_face_yourself_target_them,
          "Face Yourself the opponent: three hasty copies"),
        p("fp16_sanctum_lurker_ping", Loyalty, WinThisTurn, fp16_sanctum_lurker_ping,
          "No Admittance face + the Jace token's granted +2"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::types::Target;

    fn cast(g: &mut GameState, name: &str, target: Option<Target>, mode: Option<usize>) {
        let id = g.players[0].hand.iter().find(|c| c.definition.name == name).expect(name).id;
        g.perform_action(GameAction::CastSpell { card_id: id, target, additional_targets: vec![], mode, x_value: None })
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        crate::game::drain_stack(g);
    }

    fn opp_creature(g: &GameState, name: &str) -> CardId {
        g.battlefield.iter().find(|c| c.controller == 1 && c.definition.name == name).expect(name).id
    }

    /// FP13's intended line meets its goal: Essence Burn exiles the
    /// Ferocity creature (no return), Last Gasp kills the other.
    #[test]
    fn fp13_the_spec_line_clears_the_board() {
        let mut g = fp13_ferocity_exile_assignment();
        let bear = opp_creature(&g, "Wild Bear");
        let ghoul = opp_creature(&g, "Grave Ghoul");
        cast(&mut g, "Essence Burn", Some(Target::Permanent(bear)), None);
        cast(&mut g, "Last Gasp", Some(Target::Permanent(ghoul)), None);
        let left: Vec<&str> =
            g.battlefield.iter().filter(|c| c.controller == 1 && c.definition.is_creature()).map(|c| c.definition.name).collect();
        assert!(left.is_empty(), "left: {left:?}");
    }
}

