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
        state.battlefield.find_by_id(id).is_some_and(|c| c.cold_any(|k| !k.goaded_by.is_empty()))
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
                (c.goaded_by.contains(&d), cost, d)
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

/// A defender whose board prohibits *some* attackers ("creatures with power
/// 2 or less can't attack you") — the spill would have to ask which, so it
/// leaves that seat alone.
fn cant_be_spilled_at(state: &GameState, d: usize) -> bool {
    use crate::effect::StaticEffect;
    state.battlefield.iter().any(|c| {
        c.controller == d
            && c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::CreaturesCantAttackController { .. }))
    })
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
}
