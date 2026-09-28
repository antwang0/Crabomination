//! GameState → table-net input: any seat count, the duel as its two-seat
//! case (ML_NOTES "One net for every mode", step 1).
//!
//! The layout is `crabomination_nn::table`'s. Slot 0 is the encoded seat;
//! slots 1.. are the other living seats in turn order ([`table_slots`]).
//! Per slot: its battlefield, graveyard, stack items and command zone as
//! objects, and [`SEAT_FEATS`] of seat state. Slot 0 alone also gets its hand
//! and library — no other seat's hidden cards are ever read, and an
//! opponent's face-down permanent encodes as the nameless 2/2 it is to
//! everyone else at the table.
//!
//! Objects reuse the duel encoder's per-object features exactly (feats
//! 0..=58, same code) and add the four table-only ones `TABLE_OBJ_FEATS`
//! documents. Each distinct card also carries a descriptor
//! ([`super::card_tokens`]).
//!
//! **Seat features** (index: meaning, scale):
//!
//! | idx | feature |
//! |---|---|
//! | 0 | life / the seat's starting life |
//! | 1 | life / 20 — damage is absolute, so both scales |
//! | 2 | hand size / 7 |
//! | 3 | library / 40 |
//! | 4 | min(library, 8) / 8 — how close to decking |
//! | 5 | graveyard / 15 |
//! | 6 | exile it owns / 10 |
//! | 7 | its turn |
//! | 8-11 | lands / 8, untapped lands / 6, creatures / 6, power / 12 |
//! | 12-16 | untapped sources making W U B R G / 6 |
//! | 17 | untapped sources / 6 |
//! | 18 | poison / 10 |
//! | 19 | most commander damage taken from one commander / 21 |
//! | 20 | most commander damage its commanders dealt one living seat / 21 |
//! | 21 | its commanders in the command zone / 2 |
//! | 22 | its commanders on the battlefield / 2 |
//! | 23 | its highest commander tax (`{2}` per earlier cast) / 6 |
//! | 24 | the monarch |
//! | 25 | has the initiative |
//! | 26 | land drops left this turn / 2 |
//! | 27-29 | this turn: life gained / 5, instants and sorceries / 3, spells / 4 |
//! | 30-32 | this turn: creatures died / 3, cards left its graveyard / 3, exiled / 3 |
//! | 33 | attacking power that gets through to it / 12 |
//! | 34 | attackers aimed at it / 4 |
//! | 35 | the encoded seat's teammate |
//! | 36 | cards it owns (tokens aside) / 100 — 40, 60 and 100 name the format |
//! | 37 | slot / 8 — turns until its turn after the encoded seat's |
//!
//! **Globals**: 0 turn / 15; 1 table rounds (turn / seats) / 10; 2-5 the
//! coarse step (beginning, combat, second main, end); 6-8 the fine combat
//! step (attackers, blockers, damage); 9 stack / 3; 10 attackers / 4; 11
//! living seats / 8; 12 seats / 8; 13 a Commander game; 14 a team game; 15
//! a shared life total.

use std::sync::Arc;

use crabomination_nn::table::{
    NO_DESC, SEAT_FEATS, TABLE_GLOBAL_FEATS, TABLE_OBJ_FEATS, TableObject, TableState, Z_BF,
    Z_COMMAND, Z_GY, Z_HAND, Z_LIB, Z_STACK,
};
use crabomination_nn::{EncodedObject, OBJ_FEATS};

use super::card_tokens::tokens_of;
use super::encode::{
    BoardContext, Vocab, affordable_covered, cover_with_extra, encode_board_object_into,
    encode_card_object, scaled, scaled_u, source_cover, stack_item_card, stack_item_controller,
    stack_item_object,
};
use crate::card::CardInstance;
use crate::fxhash::HashMap;
use crate::game::types::AttackTarget;
use crate::game::{GameState, TurnStep};

/// The seats in slot order: `seat`, then every other living seat in the
/// order they take turns after it (CR 101.4, a reversed turn order
/// included). A departed seat is not at the table (CR 800.4a).
pub fn table_slots(g: &GameState, seat: usize) -> Vec<usize> {
    let n = g.players.len();
    let next = |s: usize| if g.turn_order_reversed { (s + n - 1) % n } else { (s + 1) % n };
    let mut out = Vec::with_capacity(n);
    out.push(seat);
    let mut s = seat;
    for _ in 1..n {
        s = next(s);
        if g.players[s].is_alive() {
            out.push(s);
        }
    }
    out
}

/// Encode the table from `seat`'s point of view.
pub fn encode_table(g: &GameState, seat: usize, vocab: &Vocab) -> TableState {
    // One frozen-layer scope for every computed read below, as the duel
    // encoder does.
    g.with_frozen_layers(|g| encode_table_inner(g, seat, vocab))
}

/// One descriptor per distinct card in the state. Keyed by the memoized
/// token list's address, which is the memo's own notion of "the same
/// card"; the `Arc`s ride along so the addresses stay live.
#[derive(Default)]
struct Descs {
    index: HashMap<usize, u16>,
    held: Vec<Arc<[u16]>>,
}

impl Descs {
    fn of(&mut self, t: &mut TableState, c: &CardInstance) -> u16 {
        let toks = tokens_of(c);
        let key = toks.as_ptr() as usize;
        if let Some(&d) = self.index.get(&key) {
            return d;
        }
        let d = t.add_desc(&toks);
        self.index.insert(key, d);
        self.held.push(toks);
        d
    }
}

/// Append `eo` (a duel-encoder object) to the group and return the table
/// object for its table-only features.
fn put<'t>(
    t: &'t mut TableState,
    slot: usize,
    zone: usize,
    eo: &EncodedObject,
    desc: u16,
) -> &'t mut TableObject {
    let o = t.push_default(slot, zone);
    o.card = eo.card;
    o.desc = desc;
    o.feats[..OBJ_FEATS].copy_from_slice(&eo.feats);
    o
}

/// The per-object table features every zone shares: 59 (a commander).
fn mark_commander(g: &GameState, c: &CardInstance, o: &mut TableObject) {
    if g.is_commander(c.id) {
        o.feats[59] = 1.0;
    }
}

fn encode_table_inner(g: &GameState, seat: usize, vocab: &Vocab) -> TableState {
    const _: () =
        assert!(TABLE_OBJ_FEATS == OBJ_FEATS + 4, "extend the table-only object features");
    let slots = table_slots(g, seat);
    let mut t = TableState::with_seats(slots.len());
    let mut descs = Descs::default();
    let ctx = BoardContext::new(g);
    t.reserve(
        g.battlefield.len()
            + g.stack.len()
            + g.players[seat].hand.len()
            + g.players[seat].library.len()
            + slots
                .iter()
                .map(|&p| g.players[p].graveyard.len() + g.players[p].command.len())
                .sum::<usize>(),
    );
    let n_stack = g.stack.len();
    let seat_of_slot_zero = seat;

    for (slot, &p) in slots.iter().enumerate() {
        // Battlefield — what `p` controls.
        let (mut lands, mut untapped, mut creatures, mut power) = (0i32, 0i32, 0i32, 0i32);
        for c in g.battlefield.iter().filter(|c| c.controller == p) {
            let mut eo = EncodedObject::default();
            let (is_land, is_creature, pw) = encode_board_object_into(g, c, vocab, &ctx, &mut eo);
            if is_land {
                lands += 1;
                if !c.tapped {
                    untapped += 1;
                }
            }
            if is_creature {
                creatures += 1;
                power = power.saturating_add(pw);
            }
            // CR 708.5: a face-down permanent's identity is hidden from
            // everyone but its controller. The live state (tapped, the
            // computed 2/2, counters) is public; the printed half is not.
            let hidden = c.face_down && p != seat_of_slot_zero;
            let desc = if hidden { NO_DESC } else { descs.of(&mut t, c) };
            let o = put(&mut t, slot, Z_BF, &eo, desc);
            if hidden {
                o.card = 0;
                o.feats[0] = 0.0;
                o.feats[20..25].fill(0.0);
                o.feats[35] = 0.0;
            } else {
                mark_commander(g, c, o);
            }
            if g.attacking.iter().any(|a| {
                a.attacker == c.id
                    && match a.target {
                        AttackTarget::Player(q) => q == seat_of_slot_zero,
                        AttackTarget::Planeswalker(id) => g
                            .battlefield
                            .find_by_id(id)
                            .is_some_and(|pw| pw.controller == seat_of_slot_zero),
                        AttackTarget::Battle(_) => false,
                    }
            }) {
                o.feats[61] = 1.0;
            }
            if !g.goaders(c).is_empty() {
                o.feats[62] = 1.0;
            }
        }
        {
            let f = t.seat_mut(slot);
            f[8] = lands as f32 / 8.0;
            f[9] = untapped as f32 / 6.0;
            f[10] = creatures as f32 / 6.0;
            f[11] = scaled(power) / 12.0;
        }

        // Graveyard.
        for c in g.players[p].graveyard.iter() {
            let eo = encode_card_object(c, vocab);
            let desc = descs.of(&mut t, c);
            let o = put(&mut t, slot, Z_GY, &eo, desc);
            mark_commander(g, c, o);
        }

        // Stack items `p` controls, depth from the top in feature 36.
        for (i, item) in g.stack.iter().enumerate() {
            if stack_item_controller(item) != p {
                continue;
            }
            let mut eo = stack_item_object(g, item, vocab);
            eo.feats[36] = (n_stack - 1 - i) as f32 / 4.0;
            let card = stack_item_card(g, item);
            let hidden = card.is_some_and(|c| c.face_down) && p != seat_of_slot_zero;
            let desc = match card {
                Some(c) if !hidden => descs.of(&mut t, c),
                _ => NO_DESC,
            };
            let o = put(&mut t, slot, Z_STACK, &eo, desc);
            if hidden {
                o.card = 0;
                o.feats[0] = 0.0;
                o.feats[20..25].fill(0.0);
                o.feats[35] = 0.0;
            } else if let Some(c) = card {
                mark_commander(g, c, o);
            }
        }

        // The command zone — public (CR 408.2).
        for c in g.players[p].command.iter() {
            let eo = encode_card_object(c, vocab);
            let desc = descs.of(&mut t, c);
            let o = put(&mut t, slot, Z_COMMAND, &eo, desc);
            if g.is_commander(c.id) {
                o.feats[59] = 1.0;
                let casts = g.commander_cast_count.get(&c.id).copied().unwrap_or(0);
                o.feats[60] = scaled_u(casts.saturating_mul(2)) / 6.0;
            }
        }

        if slot == 0 {
            encode_own_hand(&mut t, &mut descs, g, seat, vocab);
            encode_own_library(&mut t, &mut descs, g, seat, vocab);
        }

        fill_seat(&mut t, g, &ctx, slot, p, seat);
    }
    fill_globals(&mut t, g);
    t
}

/// Slot 0's hand, with the duel encoder's castability block (feats 25/26)
/// against the seat's own untapped sources.
fn encode_own_hand(
    t: &mut TableState,
    descs: &mut Descs,
    g: &GameState,
    seat: usize,
    vocab: &Vocab,
) {
    let sources = g.untapped_mana_colors(seat);
    let n_sources = sources.len() as u32;
    let cover = source_cover(&sources);
    let cover_extra = cover_with_extra(&cover);
    for c in g.players[seat].hand.iter() {
        let mut eo = encode_card_object(c, vocab);
        if !c.definition.is_land() {
            eo.feats[25] =
                if affordable_covered(&c.definition.cost, n_sources, &cover) { 1.0 } else { 0.0 };
            eo.feats[26] = if affordable_covered(&c.definition.cost, n_sources + 1, &cover_extra) {
                1.0
            } else {
                0.0
            };
        }
        let desc = descs.of(t, c);
        let o = put(t, 0, Z_HAND, &eo, desc);
        mark_commander(g, c, o);
    }
}

/// Slot 0's library as a multiset: one object per distinct *name*, its
/// remaining copies in feature 27, in name order so the shuffle never
/// shows. By name rather than by vocabulary index (the duel encoder's
/// grouping): most of a Commander library is off the vocabulary, and by
/// index it would fold into one "unknown" entry.
fn encode_own_library(
    t: &mut TableState,
    descs: &mut Descs,
    g: &GameState,
    seat: usize,
    vocab: &Vocab,
) {
    let mut by_name: Vec<(&str, &CardInstance, u32)> = Vec::new();
    for c in g.players[seat].library.iter() {
        let name: &str = c.definition.name;
        match by_name.iter_mut().find(|e| e.0 == name) {
            Some(e) => e.2 += 1,
            None => by_name.push((name, c, 1)),
        }
    }
    by_name.sort_unstable_by(|a, b| a.0.cmp(b.0));
    for (_, c, n) in by_name {
        let mut eo = encode_card_object(c, vocab);
        eo.feats[27] = n as f32 / 4.0;
        let desc = descs.of(t, c);
        let o = put(t, 0, Z_LIB, &eo, desc);
        mark_commander(g, c, o);
    }
}

/// Seat features for slot `slot` (seat `p`), everything but the board
/// totals (8-11), which the battlefield walk filled. See the module doc.
fn fill_seat(
    t: &mut TableState,
    g: &GameState,
    ctx: &BoardContext,
    slot: usize,
    p: usize,
    seat: usize,
) {
    let pl = &g.players[p];
    let life = g.effective_life(p);
    let lib = pl.library.len();
    let sources = g.untapped_mana_colors(p);
    let commander_taken = g
        .commander_damage
        .iter()
        .filter(|((victim, _), _)| *victim == p)
        .map(|(_, d)| *d)
        .max()
        .unwrap_or(0);
    let commander_dealt = g
        .commander_damage
        .iter()
        .filter(|((victim, id), _)| {
            *victim != p && g.players[*victim].is_alive() && pl.commanders.contains(id)
        })
        .map(|(_, d)| *d)
        .max()
        .unwrap_or(0);
    let commanders_home = pl.command.iter().filter(|c| pl.commanders.contains(&c.id)).count();
    let commanders_out = g.battlefield.iter().filter(|c| pl.commanders.contains(&c.id)).count();
    let tax = pl
        .commanders
        .iter()
        .map(|id| g.commander_cast_count.get(id).copied().unwrap_or(0))
        .max()
        .unwrap_or(0);
    let drops = if g.can_player_play_land(p) {
        g.max_lands_per_turn(p).saturating_sub(pl.lands_played_this_turn)
    } else {
        0
    };
    // Power that gets through to `p`: unblocked attackers aimed at it in
    // full, blocked tramplers by their excess — the duel encoder's globals
    // 39/40, per seat.
    let (mut through, mut aimed) = (0i32, 0u32);
    for a in g.attacking.iter() {
        if a.target != AttackTarget::Player(p) {
            continue;
        }
        aimed += 1;
        let Some(c) = g.battlefield.find_by_id(a.attacker) else { continue };
        // CR 509.1b: `blocked_attackers` is what the damage step reads — an
        // attacker whose blocker died to first strike deals nothing here.
        let blocked = g.blocked_attackers().contains(&a.attacker)
            || ctx.blocker_sums.contains_key(&a.attacker)
            || g.block_map.values().any(|att| att.contains(&a.attacker));
        let (pw, trample) = match g.computed_permanent_on(c) {
            Some(cp) => (cp.power.max(0), cp.keywords().contains(&crate::card::Keyword::Trample)),
            None => (c.power().max(0), c.has_keyword(&crate::card::Keyword::Trample)),
        };
        let blockers_t = ctx.blocker_sums.get(&a.attacker).map_or(0, |&(_, t)| t);
        let got = if !blocked {
            pw
        } else if trample {
            (pw - blockers_t).max(0)
        } else {
            0
        };
        through = through.saturating_add(got);
    }
    let owned = pl.library.len()
        + pl.hand.len()
        + pl.graveyard.len()
        + pl.command.len()
        + g.exile.iter().filter(|c| c.owner == p).count()
        + g.battlefield.iter().filter(|c| c.owner == p && !c.is_token).count();
    let teammate = p != seat && g.team_of(p) == g.team_of(seat);

    let f = t.seat_mut(slot);
    f[0] = scaled(life) / pl.starting_life.max(1) as f32;
    f[1] = scaled(life) / 20.0;
    f[2] = pl.hand.len() as f32 / 7.0;
    f[3] = lib as f32 / 40.0;
    f[4] = lib.min(8) as f32 / 8.0;
    f[5] = pl.graveyard.len() as f32 / 15.0;
    f[6] = g.exile.iter().filter(|c| c.owner == p).count() as f32 / 10.0;
    f[7] = if g.active_player_idx == p { 1.0 } else { 0.0 };
    for ci in 0..5 {
        f[12 + ci] = sources.iter().filter(|m| m[ci]).count() as f32 / 6.0;
    }
    f[17] = sources.len() as f32 / 6.0;
    f[18] = scaled_u(pl.poison_counters) / 10.0;
    f[19] = scaled_u(commander_taken) / 21.0;
    f[20] = scaled_u(commander_dealt) / 21.0;
    f[21] = commanders_home as f32 / 2.0;
    f[22] = commanders_out as f32 / 2.0;
    f[23] = scaled_u(tax.saturating_mul(2)) / 6.0;
    f[24] = if g.monarch == Some(p) { 1.0 } else { 0.0 };
    f[25] = if g.initiative == Some(p) { 1.0 } else { 0.0 };
    f[26] = drops as f32 / 2.0;
    f[27] = scaled(pl.life_gained_this_turn.min(i32::MAX as u32) as i32) / 5.0;
    f[28] = pl.instants_or_sorceries_cast_this_turn as f32 / 3.0;
    f[29] = pl.spells_cast_this_turn as f32 / 4.0;
    f[30] = pl.creatures_died_this_turn as f32 / 3.0;
    f[31] = pl.cards_left_graveyard_this_turn as f32 / 3.0;
    f[32] = pl.cards_exiled_this_turn as f32 / 3.0;
    f[33] = scaled(through) / 12.0;
    f[34] = aimed as f32 / 4.0;
    f[35] = if teammate { 1.0 } else { 0.0 };
    f[36] = owned as f32 / 100.0;
    f[37] = slot as f32 / 8.0;
    const _: () = assert!(SEAT_FEATS == 38, "extend fill_seat when adding seat features");
}

fn fill_globals(t: &mut TableState, g: &GameState) {
    let seats = g.players.len().max(1);
    let gl = &mut t.global;
    gl[0] = g.turn_number as f32 / 15.0;
    gl[1] = g.turn_number as f32 / seats as f32 / 10.0;
    let coarse = match g.step {
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw | TurnStep::PreCombatMain => 2,
        TurnStep::BeginCombat
        | TurnStep::DeclareAttackers
        | TurnStep::DeclareBlockers
        | TurnStep::FirstStrikeDamage
        | TurnStep::CombatDamage
        | TurnStep::EndCombat => 3,
        TurnStep::PostCombatMain => 4,
        TurnStep::End | TurnStep::Cleanup => 5,
    };
    gl[coarse] = 1.0;
    let fine = match g.step {
        TurnStep::DeclareAttackers => Some(6),
        TurnStep::DeclareBlockers => Some(7),
        TurnStep::FirstStrikeDamage | TurnStep::CombatDamage | TurnStep::EndCombat => Some(8),
        _ => None,
    };
    if let Some(i) = fine {
        gl[i] = 1.0;
    }
    gl[9] = g.stack.len() as f32 / 3.0;
    gl[10] = g.attacking.len() as f32 / 4.0;
    gl[11] = g.living_seats().count() as f32 / 8.0;
    gl[12] = g.players.len() as f32 / 8.0;
    gl[13] = if g.players.iter().any(|p| !p.commanders.is_empty()) { 1.0 } else { 0.0 };
    gl[14] = if g.teams.iter().any(|t| t.members.len() > 1) { 1.0 } else { 0.0 };
    gl[15] = if g.teams.iter().any(|t| t.shared_life.is_some()) { 1.0 } else { 0.0 };
    const _: () = assert!(TABLE_GLOBAL_FEATS == 16, "extend fill_globals when adding globals");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardId;
    use crate::catalog;
    use crate::player::Player;
    use crate::server::encode::{encode_state, tests::encode_guard};
    use crabomination_nn::table::{DESC_BUCKETS, SLOT_TAGS, TABLE_ZONES, TableNet};
    use crabomination_nn::{G_BF_OPP, G_BF_SELF, G_HAND_SELF};

    fn game(n: usize) -> GameState {
        let players = (0..n).map(|i| Player::new(i, format!("P{i}"))).collect();
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    fn on_board(g: &mut GameState, id: u32, def: crate::card::CardDefinition, seat: usize) {
        let mut c = CardInstance::new(CardId(id), def, seat);
        c.controller = seat;
        g.battlefield.push(c);
    }

    #[test]
    fn slots_follow_turn_order_from_the_encoded_seat() {
        let mut g = game(4);
        assert_eq!(table_slots(&g, 2), vec![2, 3, 0, 1]);
        g.turn_order_reversed = true;
        assert_eq!(table_slots(&g, 2), vec![2, 1, 0, 3], "CR 101.4 reversed");
        g.turn_order_reversed = false;
        g.players[3].eliminated = true;
        assert_eq!(table_slots(&g, 2), vec![2, 0, 1], "a departed seat is not at the table");
        let t = encode_table(&g, 2, &Vocab::sos_sealed());
        assert_eq!(t.seat_count(), 3);
        assert!((t.global[11] - 3.0 / 8.0).abs() < 1e-6 && (t.global[12] - 4.0 / 8.0).abs() < 1e-6);
    }

    /// A duel is the two-slot table, and every object carries exactly the
    /// duel encoder's features in its first `OBJ_FEATS` — the two encoders
    /// read one board the same way.
    #[test]
    fn a_duel_is_two_slots_with_the_duel_encoders_objects() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = game(2);
        g.players[0].life = 12;
        on_board(&mut g, 1, catalog::serra_angel(), 0);
        on_board(&mut g, 2, catalog::grizzly_bears(), 1);
        g.players[0].hand.push(CardInstance::new(CardId(3), catalog::lightning_bolt(), 0));
        g.players[1].hand.push(CardInstance::new(CardId(4), catalog::shock(), 1));

        let t = encode_table(&g, 0, &vocab);
        let duel = encode_state(&g, 0, &vocab);
        assert_eq!(t.seat_count(), 2);
        let feats = |o: &TableObject| o.feats[..OBJ_FEATS].to_vec();
        assert_eq!(feats(&t.group(0, Z_BF)[0]), duel.group(G_BF_SELF)[0].feats.to_vec());
        assert_eq!(feats(&t.group(1, Z_BF)[0]), duel.group(G_BF_OPP)[0].feats.to_vec());
        assert_eq!(feats(&t.group(0, Z_HAND)[0]), duel.group(G_HAND_SELF)[0].feats.to_vec());
        // The opponent's hand is a count, never objects.
        assert_eq!(t.group_len(1, Z_HAND), 0);
        assert!((t.seat(1)[2] - 1.0 / 7.0).abs() < 1e-6);
        // Life on both scales: 12 of a 20 start.
        assert!((t.seat(0)[0] - 0.6).abs() < 1e-6 && (t.seat(0)[1] - 0.6).abs() < 1e-6);
        // Every object has a descriptor, and different cards different ones.
        let d_angel = t.group(0, Z_BF)[0].desc;
        let d_bear = t.group(1, Z_BF)[0].desc;
        assert!(d_angel != NO_DESC && d_bear != NO_DESC && d_angel != d_bear);
        assert!(t.desc(d_angel as usize).iter().all(|&x| x >= 1 && (x as usize) < DESC_BUCKETS));
        // From the other seat the table rotates.
        let t1 = encode_table(&g, 1, &vocab);
        assert_eq!(t1.group_len(0, Z_BF), 1);
        assert_eq!(t1.group(0, Z_BF)[0].card, t.group(1, Z_BF)[0].card);
        assert_eq!(t1.group_len(0, Z_HAND), 1);
        assert_eq!(t1.group_len(1, Z_HAND), 0);
    }

    /// CR 708.5: an opponent's face-down permanent is a nameless 2/2 to
    /// the encoded seat, and fully itself to its controller.
    #[test]
    fn an_opponents_face_down_permanent_is_nameless() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = game(2);
        let mut c = CardInstance::new(CardId(7), catalog::serra_angel(), 1);
        c.controller = 1;
        c.face_down = true;
        g.battlefield.push(c);
        let theirs = &encode_table(&g, 0, &vocab);
        let o = &theirs.group(1, Z_BF)[0];
        assert_eq!((o.card, o.desc), (0, NO_DESC), "the name and text are hidden");
        assert_eq!(o.feats[0], 0.0, "and so is the mana value");
        assert!(o.feats[20..25].iter().all(|&f| f == 0.0), "and the pips");
        let own = &encode_table(&g, 1, &vocab);
        let o = &own.group(0, Z_BF)[0];
        assert_ne!(o.desc, NO_DESC, "its controller may look at it (CR 708.6)");
    }

    /// Commander state — the command zone, tax, commander damage both ways
    /// and the monarch — lands where the table net can read it.
    #[test]
    fn commander_state_reaches_seats_and_objects() {
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = game(4);
        g.apply_format(crate::format::Format::Commander);
        let ids = g.seat_commanders(1, vec![catalog::serra_angel()]);
        let cmdr = ids[0];
        g.commander_cast_count.insert(cmdr, 2);
        g.commander_damage.insert((0, cmdr), 15);
        g.monarch = Some(2);

        let t = encode_table(&g, 0, &vocab);
        assert_eq!(t.seat_count(), 4);
        assert_eq!(t.global[13], 1.0, "a Commander game");
        // Seat 1 is slot 1 from seat 0.
        let home = &t.group(1, Z_COMMAND)[0];
        assert_eq!(home.feats[59], 1.0, "a commander");
        assert!((home.feats[60] - 4.0 / 6.0).abs() < 1e-6, "{{4}} of tax after two casts");
        assert!((t.seat(0)[19] - 15.0 / 21.0).abs() < 1e-6, "15 taken");
        assert!((t.seat(1)[20] - 15.0 / 21.0).abs() < 1e-6, "15 dealt");
        assert_eq!(t.seat(1)[21], 0.5, "one commander at home");
        assert!((t.seat(1)[23] - 4.0 / 6.0).abs() < 1e-6);
        assert_eq!((t.seat(2)[24], t.seat(0)[24]), (1.0, 0.0), "the monarch");
        // 40 life of a 40 start.
        assert!((t.seat(3)[0] - 1.0).abs() < 1e-6 && (t.seat(3)[1] - 2.0).abs() < 1e-6);
        // Only slot 0 has hidden zones.
        for slot in 1..4 {
            assert_eq!(t.group_len(slot, Z_HAND) + t.group_len(slot, Z_LIB), 0);
        }
    }

    /// A small random-weight table net, as the trainer would export it.
    fn random_net(vocab: usize, blocks: usize) -> TableNet {
        use crabomination_nn::table::{SEAT_FEATS, TABLE_GLOBAL_FEATS, TABLE_OBJ_FEATS};
        let (e, h, sh, h1, h2, vh) = (4usize, 8usize, 8usize, 16usize, 8usize, 8usize);
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        let mut w = |n: usize| -> Vec<f32> {
            (0..n)
                .map(|_| {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    ((seed >> 40) as f32 / (1u64 << 24) as f32 - 0.5) * 0.5
                })
                .collect()
        };
        let mut t: Vec<(String, Vec<usize>, Vec<f32>)> = vec![
            ("name_emb.weight".into(), vec![vocab, e], w(vocab * e)),
            ("tok_emb.weight".into(), vec![DESC_BUCKETS, e], w(DESC_BUCKETS * e)),
            (
                "obj.weight".into(),
                vec![h, 2 * e + TABLE_OBJ_FEATS],
                w(h * (2 * e + TABLE_OBJ_FEATS)),
            ),
            ("obj.bias".into(), vec![h], w(h)),
            (
                "seat.weight".into(),
                vec![sh, TABLE_ZONES * 2 * h + SEAT_FEATS],
                w(sh * (TABLE_ZONES * 2 * h + SEAT_FEATS)),
            ),
            ("seat.bias".into(), vec![sh], w(sh)),
            ("seat_slot.weight".into(), vec![SLOT_TAGS, sh], w(SLOT_TAGS * sh)),
            (
                "trunk1.weight".into(),
                vec![h1, 3 * sh + TABLE_GLOBAL_FEATS],
                w(h1 * (3 * sh + TABLE_GLOBAL_FEATS)),
            ),
            ("trunk1.bias".into(), vec![h1], w(h1)),
            ("trunk2.weight".into(), vec![h2, h1], w(h2 * h1)),
            ("trunk2.bias".into(), vec![h2], w(h2)),
            ("value1.weight".into(), vec![vh, h2 + sh], w(vh * (h2 + sh))),
            ("value1.bias".into(), vec![vh], w(vh)),
            ("value2.weight".into(), vec![1, vh], w(vh)),
            ("value2.bias".into(), vec![1], w(1)),
        ];
        if blocks > 0 {
            t.push(("tag_zone.weight".into(), vec![TABLE_ZONES, h], w(TABLE_ZONES * h)));
            t.push(("tag_slot.weight".into(), vec![SLOT_TAGS, h], w(SLOT_TAGS * h)));
            for i in 0..blocks {
                let p = |s: &str| format!("tblocks.{i}.{s}");
                for (name, shape) in [
                    ("ln1.weight", vec![h]),
                    ("ln1.bias", vec![h]),
                    ("attn.q.weight", vec![h, h]),
                    ("attn.q.bias", vec![h]),
                    ("attn.k.weight", vec![h, h]),
                    ("attn.k.bias", vec![h]),
                    ("attn.v.weight", vec![h, h]),
                    ("attn.v.bias", vec![h]),
                    ("attn.o.weight", vec![h, h]),
                    ("attn.o.bias", vec![h]),
                    ("ln2.weight", vec![h]),
                    ("ln2.bias", vec![h]),
                    ("ffn1.weight", vec![2 * h, h]),
                    ("ffn1.bias", vec![2 * h]),
                    ("ffn2.weight", vec![h, 2 * h]),
                    ("ffn2.bias", vec![h]),
                ] {
                    let n = shape.iter().product();
                    t.push((p(name), shape, w(n)));
                }
            }
        }
        let refs: Vec<(&str, Vec<usize>, Vec<f32>)> =
            t.iter().map(|(n, s, d)| (n.as_str(), s.clone(), d.clone())).collect();
        TableNet::load(&crabomination_nn::to_safetensors(&refs)).expect("a well-formed random net")
    }

    /// End to end on a real four-seat Commander pod, mid-game: the heuristic
    /// plays a few turns, then every seat's table encodes (deterministically),
    /// shows its commanders, hides everyone else's hand and library, and runs
    /// through a table net to one probability per living seat.
    #[test]
    fn a_real_pod_encodes_and_runs_through_a_table_net() {
        use crate::server::bot::Bot;
        let _guard = encode_guard();
        let vocab = Vocab::sos_sealed();
        let mut g = crate::pod::build_pod_template(&crate::pod::pod_field(4));
        g.rng.reseed(7);
        g.start_mulligan_phase();
        let mut bots: Vec<crate::server::bot::HeuristicBot> =
            (0..4).map(|_| crate::server::bot::HeuristicBot::new()).collect();
        let mut actions = 0;
        while g.turn_number < 6 && !g.is_game_over() && actions < 4_000 {
            for (seat, bot) in bots.iter_mut().enumerate() {
                if let Some(a) = bot.next_action(&g, seat) {
                    if let Ok(events) = g.perform_action(a) {
                        g.recycle_events(events);
                    }
                    actions += 1;
                }
            }
        }
        assert!(g.turn_number >= 6, "the pod reached turn 6 ({actions} actions)");
        let (net, net_blocks) = (random_net(vocab.size(), 0), random_net(vocab.size(), 1));
        for seat in 0..4 {
            let t = encode_table(&g, seat, &vocab);
            assert_eq!(t, encode_table(&g, seat, &vocab), "seat {seat}: encoding is deterministic");
            assert_eq!(t.seat_count(), g.living_seats().count());
            assert!(t.group_len(0, Z_LIB) > 0, "slot 0 sees its own library");
            for slot in 1..t.seat_count() {
                assert_eq!(t.group_len(slot, Z_HAND) + t.group_len(slot, Z_LIB), 0);
            }
            let commanders: usize = (0..t.seat_count())
                .map(|s| {
                    t.group(s, Z_COMMAND).len()
                        + t.group(s, Z_BF).iter().filter(|o| o.feats[59] == 1.0).count()
                })
                .sum();
            assert!(commanders >= 1, "seat {seat}: some commander is visible");
            assert!(t.desc_count() > 0);
            for n in [&net, &net_blocks] {
                let p = n.forward(&t);
                assert_eq!(p.len(), t.seat_count());
                assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-5, "seat {seat}: {p:?}");
                assert!(p.iter().all(|x| x.is_finite() && *x > 0.0));
            }
        }
    }
}
