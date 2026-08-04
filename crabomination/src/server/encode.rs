//! GameState → net-input encoding for the SOS sealed value net.
//!
//! The other half of the contract lives in `crabomination_nn` (the tensor
//! types, the shard format, and the forward pass); this module owns the one
//! thing only the engine can do — reading a [`GameState`] into those types.
//! Everything encoded here is information the encoded seat could see across
//! the table: its own hand, both boards and graveyards, and *counts* of the
//! hidden zones. The opponent's hand and either library's contents never
//! enter the feature vector, so a net trained on these rows can't learn to
//! peek.
//!
//! Scope is deliberately SOS sealed ([`Vocab::sos_sealed`]): one set's worth
//! of names keeps the embedding table small and well-fed. Tokens and
//! anything off-vocabulary encode as index 0 (unknown) and are represented
//! by their object features alone — a Pest token is "an unknown 1/1" to the
//! net, which is most of what it needs to know.

use std::collections::HashMap;

use crabomination_nn::{
    EncodedObject, EncodedState, G_BF_OPP, G_BF_SELF, G_GY_OPP, G_GY_SELF, G_HAND_SELF,
    GLOBAL_FEATS, OBJ_FEATS,
};

use crate::card::{CardInstance, CounterType};
use crate::game::{GameState, TurnStep};

/// Card-name → embedding-index table. Index 0 is reserved for unknown
/// names; real cards are indexed 1.. in sorted-name order, so the mapping
/// is stable as long as the set list is (and the trainer's embedding table
/// is sized off [`Vocab::size`], so the two stay in lockstep).
pub struct Vocab {
    map: HashMap<&'static str, u16>,
}

impl Vocab {
    /// The SOS sealed universe: every card in the draftable pool plus the
    /// five basic lands the sealed builder adds.
    pub fn sos_sealed() -> Vocab {
        let mut names: std::collections::BTreeSet<&'static str> =
            crate::draft::sos_draft_pool().iter().map(|f| f().name).collect();
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            names.insert(basic);
        }
        let map = names.into_iter().zip(1u16..).collect();
        Vocab { map }
    }

    /// Total index count including the reserved unknown slot — the
    /// embedding table's row count.
    pub fn size(&self) -> usize {
        self.map.len() + 1
    }

    /// 0 for anything unrecognized (tokens, off-set cards).
    pub fn index_of(&self, name: &str) -> u16 {
        self.map.get(name).copied().unwrap_or(0)
    }
}

/// Encode the position from `seat`'s perspective. Two-player only, like
/// the rest of the bot stack.
pub fn encode_state(g: &GameState, seat: usize, vocab: &Vocab) -> EncodedState {
    let opp = 1 - seat;
    let mut s = EncodedState::default();

    for c in g.battlefield.iter() {
        let group = if c.controller == seat { G_BF_SELF } else { G_BF_OPP };
        s.groups[group].push(encode_battlefield_object(g, c, vocab));
    }
    for c in g.players[seat].hand.iter() {
        s.groups[G_HAND_SELF].push(encode_card_object(c, vocab));
    }
    for (group, p) in [(G_GY_SELF, seat), (G_GY_OPP, opp)] {
        for c in g.players[p].graveyard.iter() {
            s.groups[group].push(encode_card_object(c, vocab));
        }
    }

    let (mut lands, mut untapped, mut creatures, mut power) = ([0i32; 2], [0i32; 2], [0i32; 2], [0i32; 2]);
    for c in g.battlefield.iter() {
        let side = if c.controller == seat { 0 } else { 1 };
        if c.definition.is_land() {
            lands[side] += 1;
            if !c.tapped {
                untapped[side] += 1;
            }
        }
        if c.definition.is_creature() {
            creatures[side] += 1;
            power[side] += c.power().max(0);
        }
    }

    let gl = &mut s.global;
    gl[0] = g.players[seat].life as f32 / 20.0;
    gl[1] = g.players[opp].life as f32 / 20.0;
    gl[2] = g.players[seat].hand.len() as f32 / 7.0;
    gl[3] = g.players[opp].hand.len() as f32 / 7.0;
    gl[4] = g.players[seat].library.len() as f32 / 40.0;
    gl[5] = g.players[opp].library.len() as f32 / 40.0;
    gl[6] = g.players[seat].graveyard.len() as f32 / 15.0;
    gl[7] = g.players[opp].graveyard.len() as f32 / 15.0;
    gl[8] = g.turn_number as f32 / 15.0;
    gl[9] = if g.active_player_idx == seat { 1.0 } else { 0.0 };
    let step_slot = match g.step {
        TurnStep::Untap | TurnStep::Upkeep | TurnStep::Draw | TurnStep::PreCombatMain => 10,
        TurnStep::BeginCombat
        | TurnStep::DeclareAttackers
        | TurnStep::DeclareBlockers
        | TurnStep::FirstStrikeDamage
        | TurnStep::CombatDamage
        | TurnStep::EndCombat => 11,
        TurnStep::PostCombatMain => 12,
        TurnStep::End | TurnStep::Cleanup => 13,
    };
    gl[step_slot] = 1.0;
    gl[14] = untapped[0] as f32 / 6.0;
    gl[15] = untapped[1] as f32 / 6.0;
    gl[16] = lands[0] as f32 / 8.0;
    gl[17] = lands[1] as f32 / 8.0;
    gl[18] = g.stack.len() as f32 / 3.0;
    gl[19] = g.attacking.len() as f32 / 4.0;
    gl[20] = creatures[0] as f32 / 6.0;
    gl[21] = creatures[1] as f32 / 6.0;
    gl[22] = power[0] as f32 / 12.0;
    gl[23] = power[1] as f32 / 12.0;
    const _: () = assert!(GLOBAL_FEATS == 24, "extend the fill above when adding globals");

    s
}

/// Encode a decklist for the build net: vocab indices plus deck-level
/// features (spell curve, land/creature counts, color pips). The factory
/// list is the same shape the sealed builder and `recommend_pool` deal
/// in, so both can score builds without touching a `GameState`.
pub fn encode_deck(
    deck: &[crate::cube::CardFactory],
    vocab: &Vocab,
) -> (Vec<u16>, [f32; crabomination_nn::DECK_FEATS]) {
    use crate::mana::Color;
    let mut cards = Vec::with_capacity(deck.len());
    let mut feats = [0.0f32; crabomination_nn::DECK_FEATS];
    let (mut lands, mut creatures, mut mv_sum, mut spells) = (0u32, 0u32, 0u32, 0u32);
    let mut pips = [0u32; 5];
    for f in deck {
        let def = f();
        cards.push(vocab.index_of(def.name));
        if def.is_land() {
            lands += 1;
            continue;
        }
        spells += 1;
        let mv = def.cost.cmc();
        mv_sum += mv;
        // Curve buckets 0..=6: mv ≤1, 2, 3, 4, 5, 6, 7+.
        let bucket = (mv.clamp(1, 7) - 1) as usize;
        feats[bucket] += 1.0 / 8.0;
        if def.is_creature() {
            creatures += 1;
        }
        for c in def.cost.colored_symbols() {
            let i = match c {
                Color::White => 0,
                Color::Blue => 1,
                Color::Black => 2,
                Color::Red => 3,
                Color::Green => 4,
            };
            pips[i] += 1;
        }
    }
    feats[7] = lands as f32 / 17.0;
    feats[8] = creatures as f32 / 23.0;
    for (i, p) in pips.iter().enumerate() {
        feats[9 + i] = *p as f32 / 20.0;
    }
    feats[14] = pips.iter().filter(|&&p| p > 0).count() as f32 / 3.0;
    feats[15] = if spells > 0 { mv_sum as f32 / spells as f32 / 4.0 } else { 0.0 };
    (cards, feats)
}

/// Printed-card features shared by every zone (hand, graveyard, and the
/// base of a battlefield object).
fn encode_card_object(c: &CardInstance, vocab: &Vocab) -> EncodedObject {
    use crate::card::Keyword;
    let def = &c.definition;
    let mut feats = [0.0f32; OBJ_FEATS];
    feats[0] = def.cost.cmc() as f32 / 8.0;
    feats[1] = if def.is_creature() { 1.0 } else { 0.0 };
    feats[2] = if def.is_land() { 1.0 } else { 0.0 };
    feats[3] = if def.is_planeswalker() { 1.0 } else { 0.0 };
    feats[4] = def.power.max(0) as f32 / 8.0;
    feats[5] = def.toughness.max(0) as f32 / 8.0;
    // Evasion/combat keywords, granted ones included (`has_keyword`
    // reads printed + granted lists; the granted lists are simply empty
    // off the battlefield). First and double strike share a flag — for a
    // value function they mean the same thing at this resolution.
    for (i, kw) in [
        Keyword::Flying,
        Keyword::Reach,
        Keyword::Menace,
        Keyword::Deathtouch,
        Keyword::Lifelink,
        Keyword::Trample,
        Keyword::FirstStrike,
        Keyword::Vigilance,
    ]
    .iter()
    .enumerate()
    {
        if c.has_keyword(kw) {
            feats[12 + i] = 1.0;
        }
    }
    if c.has_keyword(&Keyword::DoubleStrike) {
        feats[18] = 1.0;
    }
    EncodedObject { card: vocab.index_of(def.name), feats }
}

/// Battlefield objects add live state on top of the printed features:
/// effective P/T (counters and pumps included), damage marked, tapped,
/// summoning sickness, loyalty, SOS prepared status, attacking.
fn encode_battlefield_object(g: &GameState, c: &CardInstance, vocab: &Vocab) -> EncodedObject {
    let mut o = encode_card_object(c, vocab);
    let f = &mut o.feats;
    f[4] = c.power().max(0) as f32 / 8.0;
    f[5] = (c.toughness() - c.damage as i32).max(0) as f32 / 8.0;
    f[6] = if c.tapped { 1.0 } else { 0.0 };
    f[7] = if c.summoning_sick { 1.0 } else { 0.0 };
    f[8] = c.counter_count(CounterType::Loyalty) as f32 / 8.0;
    f[9] = if c.counter_count(CounterType::Prepared) > 0 { 1.0 } else { 0.0 };
    f[10] = if g.attacking.iter().any(|a| a.attacker == c.id) { 1.0 } else { 0.0 };
    f[11] = if c.is_token { 1.0 } else { 0.0 };
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::player::Player;
    use crabomination_nn::NUM_GROUPS;

    fn two_player_game() -> GameState {
        let players = vec![Player::new(0, "Alice"), Player::new(1, "Bob")];
        let mut g = GameState::new(players);
        g.step = TurnStep::PreCombatMain;
        g
    }

    #[test]
    fn sos_vocab_is_substantial_and_stable() {
        let v = Vocab::sos_sealed();
        // Sanity range, not an exact count: the set list may grow a card
        // or two, but a collapse to near-zero means the pool wiring broke.
        assert!(v.size() > 150 && v.size() < 500, "vocab size {}", v.size());
        for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            assert_ne!(v.index_of(basic), 0, "{basic} missing from vocab");
        }
        assert_eq!(v.index_of("Definitely Not A Card"), 0);
        // Indices are a function of the sorted name list — two builds agree.
        let v2 = Vocab::sos_sealed();
        assert_eq!(v.index_of("Plains"), v2.index_of("Plains"));
    }

    #[test]
    fn encode_reads_the_position_seat_relative() {
        let vocab = Vocab::sos_sealed();
        let mut g = two_player_game();
        g.players[0].life = 12;
        g.players[1].life = 20;
        g.turn_number = 3;
        g.active_player_idx = 1;

        // A bear for seat 1, tapped; a card in seat 0's hand and graveyard.
        let bear = catalog::grizzly_bears();
        let mut inst = CardInstance::new(crate::card::CardId(901), bear, 1);
        inst.controller = 1;
        inst.tapped = true;
        g.battlefield.push(inst);
        let hand_card = CardInstance::new(crate::card::CardId(902), catalog::grizzly_bears(), 0);
        g.players[0].hand.push(hand_card);
        let dead = CardInstance::new(crate::card::CardId(903), catalog::grizzly_bears(), 0);
        g.players[0].graveyard.push(dead);

        let s0 = encode_state(&g, 0, &vocab);
        assert_eq!(s0.groups[G_BF_SELF].len(), 0);
        assert_eq!(s0.groups[G_BF_OPP].len(), 1);
        assert_eq!(s0.groups[G_HAND_SELF].len(), 1);
        assert_eq!(s0.groups[G_GY_SELF].len(), 1);
        assert_eq!(s0.groups[G_GY_OPP].len(), 0);
        assert!((s0.global[0] - 12.0 / 20.0).abs() < 1e-6);
        assert!((s0.global[1] - 1.0).abs() < 1e-6);
        assert_eq!(s0.global[9], 0.0, "seat 0 is not the active player");
        let opp_bear = &s0.groups[G_BF_OPP][0];
        assert_eq!(opp_bear.feats[6], 1.0, "tapped flag");
        assert!((opp_bear.feats[4] - 2.0 / 8.0).abs() < 1e-6, "2 power");
        // Grizzly Bears is off the SOS vocab — unknown index, features carry it.
        assert_eq!(opp_bear.card, vocab.index_of("Grizzly Bears"));

        // The same position from the other seat mirrors every self/opp pair.
        let s1 = encode_state(&g, 1, &vocab);
        assert_eq!(s1.groups[G_BF_SELF].len(), 1);
        assert_eq!(s1.groups[G_BF_OPP].len(), 0);
        assert_eq!(s1.groups[G_HAND_SELF].len(), 0, "opponent hand is hidden");
        assert!((s1.global[0] - 1.0).abs() < 1e-6);
        assert!((s1.global[1] - 12.0 / 20.0).abs() < 1e-6);
        assert_eq!(s1.global[9], 1.0);
        assert_eq!(s1.groups[G_GY_SELF].len(), 0);
        assert_eq!(s1.groups[G_GY_OPP].len(), 1);

        // Empty groups exist but are empty, never dropped.
        assert_eq!(s0.groups.len(), NUM_GROUPS);
    }
}
