//! Commander: the Virtue and Valor precon (WOC, Ellivere, `decks::cmdr_ellivere`).

use crabomination::card::{CardId, Keyword};
use crabomination::catalog;
use crabomination::game::types::{Attack, AttackTarget, GameAction, Target, TurnStep};
use crabomination::game::*;
use crabomination::mana::Color;

fn pod(n: usize) -> GameState {
    let mut g = if n == 2 { two_player_game() } else { multi_player_game(n) };
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

fn flood(g: &mut GameState, seat: usize) {
    for c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {
        g.players[seat].mana_pool.add(c, 20);
    }
    g.players[seat].mana_pool.add_colorless(20);
}

fn cast_x(g: &mut GameState, id: CardId, targets: &[Target], x: Option<u32>) {
    flood(g, 0);
    g.priority.player_with_priority = 0;
    g.perform_action(GameAction::CastSpell {
        card_id: id,
        target: targets.first().cloned(),
        additional_targets: targets.iter().skip(1).cloned().collect(),
        mode: None,
        x_value: x,
    })
    .expect("cast");
    drain_stack(g);
}

fn cast(g: &mut GameState, id: CardId, targets: &[Target]) {
    cast_x(g, id, targets, None);
}

fn pt(g: &GameState, id: CardId) -> (i32, i32) {
    let c = g.computed_permanent(id).expect("on battlefield");
    (c.power, c.toughness)
}

fn attached_to(g: &GameState, host: CardId) -> Vec<CardId> {
    g.battlefield.iter().filter(|c| c.attached_to == Some(host)).map(|c| c.id).collect()
}

fn attack(g: &mut GameState, attacker: CardId, defender: usize) {
    g.clear_sickness(attacker);
    g.step = TurnStep::DeclareAttackers;
    g.priority.player_with_priority = g.active_player_idx;
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker, target: AttackTarget::Player(defender) }]))
        .expect("attack");
    drain_stack(g);
}

/// Ellivere: a Virtuous Role (CR 303.7 — a Role is an Aura token) scales with
/// your enchantments; an enchanted creature's combat damage draws.
#[test]
fn ellivere_crowns_a_virtuous_role() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let e = g.add_card_to_hand(0, catalog::ellivere_of_the_wild_court());
    cast(&mut g, e, &[Target::Permanent(bear)]);
    assert_eq!(attached_to(&g, bear).len(), 1);
    assert_eq!(pt(&g, bear), (3, 3), "one enchantment: the Role");
    g.add_card_to_library(0, catalog::island());
    attack(&mut g, bear, 1);
    g.step = TurnStep::DeclareBlockers;
    g.priority.player_with_priority = 1;
    g.perform_action(GameAction::DeclareBlockers(vec![])).expect("no blocks");
    while g.step != TurnStep::PostCombatMain {
        let _ = g.advance_step(Vec::new());
        drain_stack(&mut g);
    }
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Mantle of the Ancients returns Aura and Equipment cards attached to its
/// creature (CR 303.4f) and counts every attachment.
#[test]
fn mantle_of_the_ancients_rebuilds_the_host() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.add_card_to_graveyard(0, catalog::rancor());
    g.add_card_to_graveyard(0, catalog::bonesplitter());
    let m = g.add_card_to_hand(0, catalog::mantle_of_the_ancients());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert_eq!(attached_to(&g, bear).len(), 3);
    // 2/2 + Rancor +2/+0 + Bonesplitter +2/+0 + three attachments.
    assert_eq!(pt(&g, bear), (9, 5));
}

/// Unfinished Business returns a creature and the Auras TARGETED with it,
/// attached to it — an untargeted Aura stays in the graveyard; Retether finds
/// each Aura a creature (CR 303.4i: none without a host).
#[test]
fn unfinished_business_and_retether() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rancor = g.add_card_to_graveyard(0, catalog::rancor());
    let steel = g.add_card_to_graveyard(0, catalog::spectral_steel());
    let ub = g.add_card_to_hand(0, catalog::unfinished_business());
    cast(&mut g, ub, &[Target::Permanent(bear), Target::Permanent(rancor), Target::Permanent(steel)]);
    assert_eq!(attached_to(&g, bear).len(), 2);
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let rancor = g.add_card_to_graveyard(0, catalog::rancor());
    let steel = g.add_card_to_graveyard(0, catalog::spectral_steel());
    let ub = g.add_card_to_hand(0, catalog::unfinished_business());
    cast(&mut g, ub, &[Target::Permanent(bear), Target::Permanent(steel)]);
    assert_eq!(attached_to(&g, bear), vec![steel], "only the targeted Aura");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == rancor), "Rancor wasn't a target");
    let mut g = pod(2);
    let rancor = g.add_card_to_graveyard(0, catalog::rancor());
    let r = g.add_card_to_hand(0, catalog::retether());
    cast(&mut g, r, &[]);
    assert!(g.players[0].graveyard.iter().any(|c| c.id == rancor), "no creature, no host");
    let wurm = g.add_card_to_battlefield(0, catalog::craw_wurm());
    let r = g.add_card_to_hand(0, catalog::retether());
    cast(&mut g, r, &[]);
    assert_eq!(g.battlefield_find(rancor).and_then(|c| c.attached_to), Some(wurm));
}

/// CR 303.4f — an Aura put onto the battlefield without being cast enchants
/// what the player putting it there chooses: Retether's Rancor goes on the
/// Bears, not the bigger Wurm the headless pick would take.
#[test]
fn retether_host_is_the_players_choice() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let rancor = g.add_card_to_graveyard(0, catalog::rancor());
    g.add_card_to_battlefield(0, catalog::craw_wurm());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::Target(Target::Permanent(bear))]));
    let r = g.add_card_to_hand(0, catalog::retether());
    cast(&mut g, r, &[]);
    assert_eq!(g.battlefield_find(rancor).and_then(|c| c.attached_to), Some(bear));
}

/// Timber Paladin's base P/T tracks the Auras on it.
#[test]
fn timber_paladin_grows_with_auras() {
    let mut g = pod(2);
    let tp = g.add_card_to_battlefield(0, catalog::timber_paladin());
    assert_eq!(pt(&g, tp), (1, 1));
    let s = g.add_card_to_hand(0, catalog::spectral_steel());
    cast(&mut g, s, &[Target::Permanent(tp)]);
    assert_eq!(pt(&g, tp), (5, 5), "base 3/3 +2/+2");
    let s = g.add_card_to_hand(0, catalog::spectral_steel());
    cast(&mut g, s, &[Target::Permanent(tp)]);
    assert_eq!(pt(&g, tp), (9, 9), "base 5/5 +4/+4");
    assert!(g.computed_permanent(tp).unwrap().keywords().contains(&Keyword::Vigilance));
}

/// Umbra Mystic gives every Aura on your permanents umbra armor (CR 702.89).
#[test]
fn umbra_mystic_armors_auras() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::umbra_mystic());
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::spectral_steel());
    cast(&mut g, s, &[Target::Permanent(bear)]);
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert!(g.battlefield_find(bear).is_some(), "the Aura died instead");
    assert!(g.battlefield_find(s).is_none());
}

/// Knickknack Ouphe puts Auras of mana value X or less from the top X onto
/// the battlefield attached.
#[test]
fn knickknack_ouphe_digs_for_auras() {
    let mut g = pod(2);
    let rancor = g.add_card_to_library(0, catalog::rancor());
    let bear_umbra = g.add_card_to_library(0, catalog::bear_umbra());
    g.add_card_to_library(0, catalog::island());
    let o = g.add_card_to_hand(0, catalog::knickknack_ouphe());
    cast_x(&mut g, o, &[], Some(3));
    assert_eq!(g.battlefield_find(rancor).and_then(|c| c.attached_to), Some(o));
    assert!(g.battlefield_find(bear_umbra).is_none(), "mana value 4 > X");
}

/// Liberated Livestock's tokens each may wear an Aura from hand or graveyard.
#[test]
fn liberated_livestock_dresses_its_tokens() {
    let mut g = pod(2);
    let ll = g.add_card_to_battlefield(0, catalog::liberated_livestock());
    let rancor = g.add_card_to_graveyard(0, catalog::rancor());
    let steel = g.add_card_to_hand(0, catalog::spectral_steel());
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(ll)]);
    let tokens = g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token && c.definition.is_creature()).count();
    assert_eq!(tokens, 3);
    assert!(g.battlefield_find(rancor).is_some_and(|c| c.attached_to.is_some()));
    assert!(g.battlefield_find(steel).is_some_and(|c| c.attached_to.is_some()));
}

/// Liberated Livestock's "may": each token's Aura is the player's pick, and
/// one may go bare (here the second).
#[test]
fn liberated_livestock_auras_are_the_players_picks() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    let mut g = pod(2);
    let ll = g.add_card_to_battlefield(0, catalog::liberated_livestock());
    let rancor = g.add_card_to_graveyard(0, catalog::rancor());
    let steel = g.add_card_to_hand(0, catalog::spectral_steel());
    let m = g.add_card_to_hand(0, catalog::murder());
    g.decider = Box::new(ScriptedDecider::new([
        DecisionAnswer::Cards(vec![steel]),
        DecisionAnswer::Cards(vec![]),
        DecisionAnswer::Cards(vec![rancor]),
    ]));
    cast(&mut g, m, &[Target::Permanent(ll)]);
    let tokens: Vec<CardId> =
        g.battlefield.iter().filter(|c| c.controller == 0 && c.is_token && c.definition.is_creature()).map(|c| c.id).collect();
    assert_eq!(tokens.len(), 3);
    let worn = tokens.iter().filter(|t| !attached_to(&g, **t).is_empty()).count();
    assert_eq!(worn, 2, "one token declined");
    assert!(g.battlefield_find(rancor).is_some() && g.battlefield_find(steel).is_some());
}

/// Songbirds' Blessing: an attack reveals to an Aura that "may" go onto the
/// battlefield — on a host you choose — else into your hand.
#[test]
fn songbirds_blessing_puts_or_keeps_the_aura() {
    use crabomination::decision::{DecisionAnswer, ScriptedDecider};
    for decline in [false, true] {
        let mut g = pod(2);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.clear_sickness(bear);
        g.add_card_to_library(0, catalog::island());
        let rancor = g.add_card_to_library(0, catalog::rancor());
        let sb = g.add_card_to_hand(0, catalog::songbirds_blessing());
        cast(&mut g, sb, &[Target::Permanent(bear)]);
        if decline {
            g.decider = Box::new(ScriptedDecider::new([DecisionAnswer::DeclineTarget]));
        }
        g.step = TurnStep::DeclareAttackers;
        g.declare_attackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]).expect("attack");
        drain_stack(&mut g);
        if decline {
            assert!(g.players[0].hand.iter().any(|c| c.id == rancor), "declined: to hand");
        } else {
            assert_eq!(g.battlefield_find(rancor).and_then(|c| c.attached_to), Some(bear));
        }
    }
}

/// Gylwain puts a Role on each nontoken creature entering under you.
#[test]
fn gylwain_casts_roles() {
    let mut g = pod(2);
    g.add_card_to_battlefield(0, catalog::gylwain_casting_director());
    let bear = g.add_card_to_hand(0, catalog::grizzly_bears());
    cast(&mut g, bear, &[]);
    assert_eq!(attached_to(&g, bear).len(), 1);
    assert_eq!(pt(&g, bear), (3, 3));
}

/// Sage's Reverie draws for each Aura you control on a creature, itself
/// included, and pumps by the same count.
#[test]
fn sages_reverie_counts_auras() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::spectral_steel());
    cast(&mut g, s, &[Target::Permanent(bear)]);
    for _ in 0..3 {
        g.add_card_to_library(0, catalog::island());
    }
    let sr = g.add_card_to_hand(0, catalog::sages_reverie());
    cast(&mut g, sr, &[Target::Permanent(bear)]);
    assert_eq!(g.players[0].hand.len(), 2);
    assert_eq!(pt(&g, bear), (6, 6), "2/2 +2/+2 +2/+2");
}

/// Aura Gnarlid: +1/+1 per Aura anywhere on the battlefield.
#[test]
fn aura_gnarlid_counts_every_aura() {
    let mut g = pod(2);
    let ag = g.add_card_to_battlefield(0, catalog::aura_gnarlid());
    let bear = g.add_card_to_battlefield(1, catalog::grizzly_bears());
    let s = g.add_card_to_hand(0, catalog::spectral_steel());
    cast(&mut g, s, &[Target::Permanent(bear)]);
    assert_eq!(pt(&g, ag), (3, 3));
}

/// Giant Inheritance returns to hand when it goes to the graveyard.
#[test]
fn giant_inheritance_comes_home() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let gi = g.add_card_to_hand(0, catalog::giant_inheritance());
    cast(&mut g, gi, &[Target::Permanent(bear)]);
    assert_eq!(pt(&g, bear), (7, 7));
    let m = g.add_card_to_hand(0, catalog::murder());
    cast(&mut g, m, &[Target::Permanent(bear)]);
    assert!(g.players[0].hand.iter().any(|c| c.id == gi));
}

/// Ox Drover gives an opponent an Ox and draws.
#[test]
fn ox_drover_gifts_an_ox() {
    let mut g = pod(2);
    g.add_card_to_library(0, catalog::island());
    let od = g.add_card_to_hand(0, catalog::ox_drover());
    cast(&mut g, od, &[Target::Player(1)]);
    assert!(g.battlefield.iter().any(|c| c.controller == 1 && c.definition.name == "Ox"));
    assert_eq!(g.players[0].hand.len(), 1);
}

/// Careful Cultivation's channel mints a mana Monk.
#[test]
fn careful_cultivation_channels_a_monk() {
    let mut g = pod(2);
    let cc = g.add_card_to_hand(0, catalog::careful_cultivation());
    flood(&mut g, 0);
    g.perform_action(GameAction::ActivateAbility {
        card_id: cc,
        ability_index: 0,
        target: None,
        additional_targets: vec![],
        x_value: None,
        mode: None,
    })
    .expect("channel");
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.definition.name == "Human Monk"));
    assert!(g.players[0].graveyard.iter().any(|c| c.id == cc));
}

/// Verdant Embrace mints a Saproling at each upkeep.
#[test]
fn verdant_embrace_grows_saprolings() {
    let mut g = pod(2);
    let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
    let v = g.add_card_to_hand(0, catalog::verdant_embrace());
    cast(&mut g, v, &[Target::Permanent(bear)]);
    g.active_player_idx = 1;
    g.step = TurnStep::Upkeep;
    g.fire_step_triggers(TurnStep::Upkeep);
    drain_stack(&mut g);
    assert!(g.battlefield.iter().any(|c| c.controller == 0 && c.definition.name == "Saproling"));
}

/// CR 303.4i — a targeted Aura that can't legally enchant "that creature"
/// stays in the graveyard: Unfinished Business's Wild Growth (enchant land)
/// doesn't come back with the Bears.
#[test]
fn cr_303_4i_unfinished_business_leaves_an_aura_that_cant_enchant() {
    let mut g = pod(2);
    let bear = g.add_card_to_graveyard(0, catalog::grizzly_bears());
    let growth = g.add_card_to_graveyard(0, catalog::wild_growth());
    let ub = g.add_card_to_hand(0, catalog::unfinished_business());
    cast(&mut g, ub, &[Target::Permanent(bear), Target::Permanent(growth)]);
    assert!(g.battlefield_find(bear).is_some(), "the Bears returned");
    assert!(g.players[0].graveyard.iter().any(|c| c.id == growth), "Wild Growth can't enchant a creature");
}
