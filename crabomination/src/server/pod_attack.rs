//! Which player each of the bot's face attackers goes at, at N > 2 seats.
//!
//! Its own module rather than more of `bot.rs`: the duel answer is "the one
//! opponent", and everything here is what a table of opponents adds.

use crate::card::CardId;
use crate::game::GameState;
use crate::game::types::{Attack, AttackTarget};

/// CR 506.1 / 508.1b — split the face attackers among the table. They all go
/// at `first` (the bot's chosen defender) until that seat is dead with a
/// blocker's margin — its effective life, after its untapped creatures each
/// stop one of the biggest attackers — and the surplus spills to the next
/// opponent by [`GameState::hostile_opponent_score`], the same way, then the
/// next. A defender the surplus can't finish gets none of it: what is left
/// stays on `first`, as every attacker did before (a Krenko board killed one
/// seat a turn with 292 Goblins while the table sat on 40 life).
///
/// A duel has one defender, so the answer there is unchanged. Goaded
/// attackers stay on `first` (CR 701.15b is the planner's business).
pub(crate) fn spread_face_attacks(
    state: &GameState,
    seat: usize,
    first: usize,
    attackers: Vec<CardId>,
    out: &mut Vec<Attack>,
) {
    let at = |id: CardId, p: usize| Attack { attacker: id, target: AttackTarget::Player(p) };
    // A duel (the simulator's hot path) never allocates the defender list.
    if state.players.len() <= 2 || attackers.len() < 2 {
        out.extend(attackers.into_iter().map(|id| at(id, first)));
        return;
    }
    let defenders = state.attackable_players_for(seat);
    if defenders.len() < 2 || !defenders.contains(&first) {
        out.extend(attackers.into_iter().map(|id| at(id, first)));
        return;
    }
    let power = |id: CardId| state.battlefield.find_by_id(id).map_or(0, |c| c.power().max(0));
    let (pinned, mut free): (Vec<CardId>, Vec<CardId>) = attackers.into_iter().partition(|&id| {
        state.battlefield.find_by_id(id).is_some_and(|c| state.is_goaded(c))
    });
    free.sort_by_key(|&id| (std::cmp::Reverse(power(id)), id));
    let mut others: Vec<usize> =
        defenders.into_iter().filter(|&d| d != first && !cant_be_spilled_at(state, d)).collect();
    others.sort_by_key(|&d| (std::cmp::Reverse(state.hostile_opponent_score(seat, d)), d));

    let finished = |d: usize, powers: &mut Vec<i32>| {
        let blockers = state
            .battlefield
            .iter()
            .filter(|c| c.controller == d && !c.tapped && c.definition.is_creature())
            .count();
        powers.sort_unstable_by(|a, b| b.cmp(a));
        powers.iter().skip(blockers).fold(0i32, |a, &p| a.saturating_add(p)) >= state.effective_life(d).max(1)
    };
    let mut next = 0;
    let mut first_powers: Vec<i32> = pinned.iter().map(|&id| power(id)).collect();
    while !finished(first, &mut first_powers) && next < free.len() {
        first_powers.push(power(free[next]));
        next += 1;
    }
    out.extend(pinned.into_iter().chain(free[..next].iter().copied()).map(|id| at(id, first)));
    if !finished(first, &mut first_powers) {
        return;
    }
    for d in others {
        let start = next;
        let mut powers = Vec::new();
        while !finished(d, &mut powers) && next < free.len() {
            powers.push(power(free[next]));
            next += 1;
        }
        if !finished(d, &mut powers) {
            next = start;
            continue;
        }
        out.extend(free[start..next].iter().map(|&id| at(id, d)));
    }
    out.extend(free[next..].iter().map(|&id| at(id, first)));
}

/// CR 508.1a — `pick` if `seat` may attack it, else the most hostile seat it
/// may attack. Mystic Barrier bars all but one opponent: aimed at a barred
/// seat, every creature failed the declaration gate and a 947-creature board
/// never attacked (eight-seat pod, seed 23081 game 50). A duel keeps `pick`.
pub(crate) fn attackable_or(state: &GameState, seat: usize, pick: usize) -> usize {
    if state.players.len() <= 2 {
        return pick;
    }
    let open = state.attackable_players_for(seat);
    if open.is_empty() || open.contains(&pick) {
        return pick;
    }
    open.into_iter()
        .min_by_key(|&d| (std::cmp::Reverse(state.hostile_opponent_score(seat, d)), d))
        .unwrap_or(pick)
}

/// CR 508.1g — re-aim each attacker whose declared player costs a tax at the
/// cheapest defender it may legally attack instead (a goader only if nothing
/// else is open, CR 701.15b). Returns whether any attack moved. A duel has
/// one defender and never moves.
pub(crate) fn retarget_taxed_attacks(
    state: &GameState,
    seat: usize,
    statics: u32,
    keyword_tax: &impl Fn(CardId) -> u32,
    attacks: &mut [Attack],
) -> bool {
    if state.players.len() <= 2 {
        return false;
    }
    let defenders = state.attackable_players_for(seat);
    let tax = |a: &Attack| state.attack_tax_for(std::slice::from_ref(a), statics, keyword_tax);
    let mut moved = false;
    for a in attacks.iter_mut() {
        let AttackTarget::Player(_) = a.target else { continue };
        let now = tax(a);
        if now == 0 {
            continue;
        }
        let Some(c) = state.battlefield.find_by_id(a.attacker) else { continue };
        let kws = state.computed_permanent(a.attacker).map(|cp| cp.keywords().to_vec()).unwrap_or_default();
        let best = defenders
            .iter()
            .copied()
            .filter(|&d| state.attacker_target_block(seat, a.attacker, &kws, Some(d)).is_none())
            .map(|d| {
                let cost = tax(&Attack { attacker: a.attacker, target: AttackTarget::Player(d) });
                (state.goaded_by_player(c, d), cost, d)
            })
            .min();
        if let Some((_, cost, d)) = best
            && cost < now
        {
            a.target = AttackTarget::Player(d);
            moved = true;
        }
    }
    moved
}

/// CR 508.1g — how much of a pre-combat "you may pay {X}" (`max` X at most)
/// `seat` leaves unpaid so the attack it plans can still pay its tax. Numa,
/// Joraga Chieftain's beginning-of-combat {X}{X} tapped its seat out under
/// Ghostly Prison every turn and its 4,000-power Elf never attacked again:
/// 1,459 frozen turns, `--pod-decks 605,3041,5751,5998 --seed 15255`.
/// 0 off `seat`'s turn, from declare attackers on, and on an untaxed board.
pub(crate) fn x_held_for_attack_tax(state: &GameState, seat: usize, max: u32) -> u32 {
    use crate::game::types::TurnStep;
    if max == 0 || state.active_player_idx != seat || state.step > TurnStep::BeginCombat {
        return 0;
    }
    let statics = crate::game::combat::attack_static_scan(state);
    if !state.attack_tax_possible(statics) {
        return 0;
    }
    let attacks = super::bot::pick_attacks(state, seat);
    let keyword_tax = |id: CardId| {
        state.computed_permanent(id).map(|cp| state.attack_block_keyword_tax(id, cp.keywords(), true)).unwrap_or(0)
    };
    let tax = state.attack_tax_for(&attacks, statics, keyword_tax);
    if tax == 0 {
        return 0;
    }
    // Mana an X costs, as the ask's `max` implies it ({X}{X} is two).
    let per_x = (super::bot::mana_upper_bound(state, seat) / max).max(1);
    tax.div_ceil(per_x).min(max)
}

/// CR 508.1g / 119.4 — a declaration under a life tax (Sivitri) that would
/// cost `seat` its life, or more life than it has, loses taxed attackers from
/// the back until it costs less than its life total. Without a life tax up anywhere
/// the declaration is returned untouched.
pub(crate) fn trim_life_taxed_attacks(state: &GameState, seat: usize, mut attacks: Vec<Attack>) -> Vec<Attack> {
    let life = state.players[seat].life;
    while state.attack_life_tax_for(&attacks) as i32 >= life {
        let Some(i) = attacks.iter().rposition(|a| state.attack_life_tax_for(std::slice::from_ref(a)) > 0) else { break };
        attacks.remove(i);
    }
    attacks
}

/// A defender whose board prohibits *some* attackers ("creatures with power
/// 2 or less can't attack you") — the spill would have to ask which, so it
/// leaves that seat alone.
fn cant_be_spilled_at(state: &GameState, d: usize) -> bool {
    use crate::effect::StaticEffect;
    // Island Sanctuary's shield is the same shape: only some attackers may go.
    state.players[d].attackable_only_by_until_next_turn.is_some()
        || state.battlefield.iter().any(|c| {
        c.controller == d
            && c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect.ungated(), StaticEffect::CreaturesCantAttackController { .. }))
    })
}

/// CR 508.1 — an attack a defender-reading prohibition bars (Island
/// Sanctuary's shield, a filtered "can't attack you") re-aims at another
/// player the attacker may attack, or the attacker stays home. A barred
/// attack used to sink the whole declaration, so the seat attacked with
/// nothing — two Sanctuary seats passed for 3,000 turns (8-seat pod, seed
/// 31664). Commander games only; a declaration nothing bars is untouched.
pub(crate) fn reaim_prohibited_attacks(state: &GameState, seat: usize, mut attacks: Vec<Attack>) -> Vec<Attack> {
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) {
        return attacks;
    }
    let statics = crate::game::combat::attack_static_scan(state);
    if attacks.iter().all(|a| state.attack_target_prohibited(a, statics).is_none()) {
        return attacks;
    }
    let defenders = state.attackable_players_for(seat);
    attacks.retain_mut(|a| {
        if state.attack_target_prohibited(a, statics).is_none() {
            return true;
        }
        let kws = state.computed_permanent(a.attacker).map(|cp| cp.keywords().to_vec()).unwrap_or_default();
        let open = defenders
            .iter()
            .map(|&d| Attack { attacker: a.attacker, target: AttackTarget::Player(d) })
            .find(|c| {
                let AttackTarget::Player(d) = c.target else { return false };
                state.attacker_target_block(seat, a.attacker, &kws, Some(d)).is_none()
                    && state.attack_target_prohibited(c, statics).is_none()
            });
        match open {
            Some(c) => {
                *a = c;
                true
            }
            None => false,
        }
    });
    attacks
}

/// CR 506.2 — a per-defender attacker cap (Crawlspace's "no more than two
/// creatures can attack you", Tomik's "no more than one creature can attack
/// this planeswalker") rejects the whole declaration when exceeded, so an
/// attack past a full defender re-aims at a player with room, or stays home
/// (8-seat pod, seed 2941252: three declarations at Tomik's walkers sunk).
/// Commander games only; with no cap on the board nothing is touched.
pub(crate) fn fit_attacker_caps(state: &GameState, seat: usize, mut attacks: Vec<Attack>) -> Vec<Attack> {
    use crate::game::combat::attack_static;
    if state.players.get(seat).is_none_or(|p| p.commanders.is_empty()) {
        return attacks;
    }
    let statics = crate::game::combat::attack_static_scan(state);
    if statics & attack_static::ATTACKER_CAP == 0 {
        return attacks;
    }
    let cap_of = |t: AttackTarget| match t {
        AttackTarget::Player(d) => state.attacker_cap_against(d),
        AttackTarget::Planeswalker(pw) => {
            state.battlefield_find(pw).and_then(|c| state.planeswalker_attacker_cap(c.controller))
        }
        AttackTarget::Battle(_) => None,
    };
    let defenders = state.attackable_players_for(seat);
    let mut taken: Vec<(AttackTarget, usize)> = Vec::new();
    let room = |taken: &[(AttackTarget, usize)], t: AttackTarget| {
        let n = taken.iter().find(|(k, _)| *k == t).map_or(0, |&(_, n)| n);
        cap_of(t).is_none_or(|cap| n < cap)
    };
    attacks.retain_mut(|a| {
        if !room(&taken, a.target) {
            let kws = state.computed_permanent(a.attacker).map(|cp| cp.keywords().to_vec()).unwrap_or_default();
            let open = defenders.iter().map(|&d| AttackTarget::Player(d)).find(|&t| {
                let AttackTarget::Player(d) = t else { return false };
                room(&taken, t)
                    && state.attacker_target_block(seat, a.attacker, &kws, Some(d)).is_none()
                    && state.attack_target_prohibited(&Attack { attacker: a.attacker, target: t }, statics).is_none()
            });
            match open {
                Some(t) => a.target = t,
                None => return false,
            }
        }
        match taken.iter_mut().find(|(k, _)| *k == a.target) {
            Some((_, n)) => *n += 1,
            None => taken.push((a.target, 1)),
        }
        true
    });
    attacks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::multi_player_game;
    use crate::game::types::TurnStep;

    fn table(lives: [i32; 4]) -> (GameState, Vec<CardId>) {
        let mut g = multi_player_game(4);
        for (p, life) in lives.into_iter().enumerate() {
            g.players[p].life = life;
        }
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        let bears = (0..6).map(|_| g.add_card_to_battlefield(0, crate::catalog::grizzly_bears())).collect();
        (g, bears)
    }

    fn aimed(out: &[Attack]) -> [usize; 4] {
        let mut n = [0; 4];
        for a in out {
            if let AttackTarget::Player(p) = a.target {
                n[p] += 1;
            }
        }
        n
    }

    /// CR 508.1a — under Mystic Barrier (right) seat 0 may attack only seat 3,
    /// so the bot swings there even though seat 1 is the softer target.
    #[test]
    fn cr_508_1a_a_barrier_aims_the_attack_at_the_one_open_seat() {
        let (mut g, bears) = table([40, 4, 40, 40]);
        for &b in &bears {
            g.clear_sickness(b);
        }
        let barrier = g.add_card_to_battlefield(1, crate::catalog::mystic_barrier());
        g.battlefield_find_mut(barrier).unwrap().modes_chosen = vec![1];
        g.priority.player_with_priority = 0;
        assert_eq!(g.attackable_players_for(0), vec![3]);
        let out = crate::server::bot::pick_attacks(&g, 0);
        assert!(!out.is_empty(), "the bears attack");
        assert_eq!(aimed(&out), [0, 0, 0, out.len()]);
    }

    /// CR 508.1b — each attacker picks its own defender: two 2/2s finish a
    /// seat on 4, so the surplus goes at the next seat on 4 rather than
    /// piling on; the 40-life seat can't be finished and gets none of it.
    #[test]
    fn cr_508_1b_overkill_spills_to_the_next_opponent() {
        let (g, bears) = table([40, 4, 4, 40]);
        let mut out = Vec::new();
        spread_face_attacks(&g, 0, 1, bears, &mut out);
        assert_eq!(aimed(&out), [0, 4, 2, 0]);
    }

    /// An untapped creature is one blocker's margin: seat 2 now needs three.
    #[test]
    fn cr_508_1b_spill_leaves_a_blockers_margin() {
        let (mut g, bears) = table([40, 4, 4, 40]);
        g.add_card_to_battlefield(2, crate::catalog::grizzly_bears());
        let mut out = Vec::new();
        spread_face_attacks(&g, 0, 1, bears, &mut out);
        assert_eq!(aimed(&out), [0, 3, 3, 0]);
    }

    /// Not lethal on the chosen defender → everything stays on it, as before.
    #[test]
    fn no_kill_no_spill() {
        let (g, bears) = table([40, 40, 4, 4]);
        let mut out = Vec::new();
        spread_face_attacks(&g, 0, 1, bears, &mut out);
        assert_eq!(aimed(&out), [0, 6, 0, 0]);
    }

    /// CR 508.1d / 508.1g — a Juggernaut must attack if able, and it is able:
    /// Ghostly Prison taxes only attacks on *its* controller. With no mana the
    /// planner used to aim it at the Prison, drop it for the tax, and have the
    /// empty declaration rejected — the bot's whole combat, twice in 40
    /// eighteen-seat games.
    #[test]
    fn a_must_attacker_goes_around_a_tax_it_cannot_pay() {
        let mut g = multi_player_game(3);
        g.add_card_to_battlefield(1, crate::catalog::ghostly_prison());
        let jugg = g.add_card_to_battlefield(0, crate::catalog::juggernaut());
        g.clear_sickness(jugg);
        g.monarch = Some(1); // the planner's defender is the Prison's controller
        g.active_player_idx = 0;
        g.step = TurnStep::DeclareAttackers;
        g.priority.player_with_priority = 0;
        let attacks = crate::server::bot::pick_attacks(&g, 0);
        assert_eq!(
            attacks.iter().find(|a| a.attacker == jugg).map(|a| a.target),
            Some(AttackTarget::Player(2)),
            "{attacks:?}"
        );
        let mut g2 = g.clone();
        g2.declare_attackers(attacks).expect("a legal declaration");
    }

    /// CR 508.1 — bug fix: a Bear aimed at a player behind Island Sanctuary's
    /// shield re-aims at an open opponent instead of sinking the declaration.
    #[test]
    fn a_shielded_player_s_attackers_re_aim() {
        let (mut g, bears) = table([40; 4]);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        for &b in &bears {
            g.clear_sickness(b);
        }
        g.players[1].attackable_only_by_until_next_turn =
            Some(Box::new(crate::card::SelectionRequirement::HasKeyword(crate::card::Keyword::Flying)));
        let attacks = vec![
            Attack { attacker: bears[0], target: AttackTarget::Player(1) },
            Attack { attacker: bears[1], target: AttackTarget::Player(2) },
        ];
        let out = reaim_prohibited_attacks(&g, 0, attacks);
        assert_eq!(out.len(), 2);
        assert!(out.iter().all(|a| a.target != AttackTarget::Player(1)));
        assert!(g.would_accept(crate::game::GameAction::DeclareAttackers(out)));
    }

    /// CR 506.2 — bug fix (seed 2941252): two Bears at a Tomik seat's
    /// planeswalker and three at a Crawlspace seat were one rejected
    /// declaration; the surplus now re-aims at a seat with room or stays home.
    #[test]
    fn cr_506_2_attacks_past_a_defender_s_cap_re_aim() {
        let (mut g, bears) = table([40; 4]);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        for &b in &bears {
            g.clear_sickness(b);
        }
        g.add_card_to_battlefield(1, crate::catalog::tomik_orzhov_lawmage());
        let walker = g.add_card_to_battlefield(1, crate::catalog::jace_beleren());
        g.add_card_to_battlefield(2, crate::catalog::crawlspace());
        g.add_card_to_battlefield(3, crate::catalog::crawlspace());
        let at = |i: usize, target| Attack { attacker: bears[i], target };
        let attacks = vec![
            at(0, AttackTarget::Planeswalker(walker)),
            at(1, AttackTarget::Planeswalker(walker)),
            at(2, AttackTarget::Player(2)),
            at(3, AttackTarget::Player(2)),
            at(4, AttackTarget::Player(2)),
            at(5, AttackTarget::Player(3)),
        ];
        assert!(!g.would_accept(crate::game::GameAction::DeclareAttackers(attacks.clone())));
        let out = fit_attacker_caps(&g, 0, attacks);
        assert_eq!(out.iter().filter(|a| a.target == AttackTarget::Planeswalker(walker)).count(), 1);
        assert_eq!(aimed(&out), [0, 2, 2, 1], "{out:?}");
        assert!(g.would_accept(crate::game::GameAction::DeclareAttackers(out)));
    }

    /// CR 508.1g — bug fix (seed 15255's 1,459-turn stalemate): before
    /// combat, a pre-combat {X}{X} leaves Ghostly Prison's {2} unpaid, so the
    /// big creature still attacks; off its turn, or untaxed, nothing is held.
    #[test]
    fn cr_508_1g_a_pre_combat_x_leaves_the_attack_tax_unpaid() {
        let mut g = multi_player_game(2);
        g.seat_commanders(0, vec![crate::catalog::grizzly_bears()]);
        let giant = g.add_card_to_battlefield(0, crate::catalog::craw_wurm());
        g.clear_sickness(giant);
        for _ in 0..6 {
            g.add_card_to_battlefield(0, crate::catalog::forest());
        }
        g.active_player_idx = 0;
        g.step = TurnStep::BeginCombat;
        g.priority.player_with_priority = 0;
        assert_eq!(x_held_for_attack_tax(&g, 0, 3), 0, "untaxed");
        g.add_card_to_battlefield(1, crate::catalog::ghostly_prison());
        assert_eq!(x_held_for_attack_tax(&g, 0, 3), 1, "one X unit of {{X}}{{X}} covers the {{2}}");
        g.step = TurnStep::DeclareBlockers;
        assert_eq!(x_held_for_attack_tax(&g, 0, 3), 0, "attackers already declared");
        g.step = TurnStep::BeginCombat;
        g.active_player_idx = 1;
        assert_eq!(x_held_for_attack_tax(&g, 0, 3), 0, "not our combat");
    }
}
