//! The table recorder: self-play games of any seat count turned into
//! [`TableRow`]s for the one-model-for-every-mode net (ML_NOTES "One model
//! for every mode — the user's 2026-11-02 additions, and step 2 begins").
//!
//! A game comes from a [`Mode`] — SOS or FRA sealed, cube, or a Commander pod
//! of 2..=6 seats — with the mode's **card pool**: every card that can appear
//! in it (the set, the cube list, the training precons' union), shared by
//! every row of the game. The game is driven by the pod driver
//! (`pod::play_pod_game_observed`, one pilot per seat, any seat count), and at
//! each snapshot point every living seat's view is encoded
//! ([`encode_table`]) with the pool and the hidden truth the belief head
//! learns from: each opponent's hand and library as pool indices.
//!
//! Labels (the 2026-09-29 note's): `win` one-hot on the winner in the row's
//! slot order, a draw split across the seats still in; `life` final over
//! starting life, clamped to [0, 1.5], 0 for a seat that lost; `game_len`
//! the turns still to come over 15, as the duel recorder's. A seat's rows
//! stop when it leaves; its last row carries the final result.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use crabomination_nn::table::{TablePool, TableRow};

use crate::card::CardDefinition;
use crate::game::{GameState, TurnStep};
use crate::mana::ColorSet;
use crate::pod::{PodDeck, PodOutcome};
use crate::recommend::Pilot;
use crate::server::card_tokens::definition_tokens;
use crate::server::encode::Vocab;
use crate::server::encode_table::{encode_table, table_slots};

type CardFactory = fn() -> CardDefinition;

/// A game's pool and its name → pool-index map, with what the honest
/// redeal (`server::honest`) needs to deal a pool card into a hidden zone:
/// each entry's factory, color identity and basic-land flag.
pub struct PoolIx {
    pub pool: Arc<TablePool>,
    index: HashMap<String, u16>,
    factories: Vec<CardFactory>,
    /// Built on first deal: most of a 22,907-card Commander pool is never
    /// dealt in a given process.
    defs: Vec<OnceLock<Arc<CardDefinition>>>,
    identity: Vec<ColorSet>,
    basic: Vec<bool>,
}

impl PoolIx {
    /// The pool of `cards`, one entry per distinct name, in first-seen order.
    pub fn new(cards: impl IntoIterator<Item = CardFactory>, vocab: &Vocab) -> Self {
        let mut pool = TablePool::default();
        let mut index = HashMap::new();
        let (mut factories, mut identity, mut basic) = (Vec::new(), Vec::new(), Vec::new());
        for f in cards {
            let def = f();
            let name: &str = def.name;
            if index.contains_key(name) {
                continue;
            }
            index.insert(name.to_string(), pool.len() as u16);
            pool.push(vocab.index_of(name), &definition_tokens(&def));
            factories.push(f);
            identity.push(crate::color_identity::color_identity(&def));
            basic.push(def.is_basic());
        }
        let defs = factories.iter().map(|_| OnceLock::new()).collect();
        Self { pool: Arc::new(pool), index, factories, defs, identity, basic }
    }

    /// `name`'s pool index, if the pool has it.
    pub fn of(&self, name: &str) -> Option<u16> {
        self.index.get(name).copied()
    }

    /// Card `i`'s definition, shared by every copy dealt from it.
    pub fn def(&self, i: usize) -> Arc<CardDefinition> {
        self.defs[i].get_or_init(|| Arc::new((self.factories[i])())).clone()
    }

    /// Card `i`'s color identity (CR 903.4).
    pub fn identity(&self, i: usize) -> ColorSet {
        self.identity[i]
    }

    /// Whether card `i` is a basic land — the one card a deck may run any
    /// number of (CR 100.2a, 903.5b).
    pub fn is_basic(&self, i: usize) -> bool {
        self.basic[i]
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pool.is_empty()
    }
}

/// What a recorded game is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    SealedSos,
    SealedFra,
    Cube,
    /// A Commander pod of this many seats (2..=6), decks dealt from the
    /// training precons.
    Pod(usize),
}

impl Mode {
    pub fn name(self) -> String {
        match self {
            Mode::SealedSos => "sos".into(),
            Mode::SealedFra => "fra".into(),
            Mode::Cube => "cube".into(),
            Mode::Pod(n) => format!("pod{n}"),
        }
    }
}

/// Decks never dealt to a training game: every 31st of `pod::target_decks()`
/// by index, fixed — ~200 of the 6,188 (the 2026-09-28 plan's "~20 precons"
/// was written against 183; ML_NOTES 2026-11-03 re-registers it).
pub fn is_held_out_deck(index: usize) -> bool {
    index % 31 == 7
}

fn pod_decks() -> &'static [PodDeck] {
    static DECKS: OnceLock<Vec<PodDeck>> = OnceLock::new();
    DECKS.get_or_init(crate::pod::target_decks)
}

/// The mode's card pool, built once per process.
pub fn mode_pool(mode: Mode, vocab: &Vocab) -> &'static PoolIx {
    static SOS: OnceLock<PoolIx> = OnceLock::new();
    static FRA: OnceLock<PoolIx> = OnceLock::new();
    static CUBE: OnceLock<PoolIx> = OnceLock::new();
    static POD: OnceLock<PoolIx> = OnceLock::new();
    match mode {
        Mode::SealedSos => SOS.get_or_init(|| PoolIx::new(crate::draft::sos_draft_pool_ref().iter().copied(), vocab)),
        Mode::SealedFra => FRA.get_or_init(|| {
            let p = crate::fra_mode::FraPacks::get();
            PoolIx::new(p.common.iter().chain(&p.uncommon).chain(&p.rare).chain(&p.mythic).copied(), vocab)
        }),
        Mode::Cube => CUBE.get_or_init(|| PoolIx::new(crate::cube::all_cube_cards(), vocab)),
        // Every precon's cards, held-out decks included: the pool is what a
        // player could know exists, not which lists were trained on.
        Mode::Pod(_) => POD.get_or_init(|| {
            PoolIx::new(pod_decks().iter().flat_map(|d| d.commanders.iter().chain(d.main).copied()), vocab)
        }),
    }
}

/// The pool a live game's states carry, judged off the game: more than two
/// seats or a commander is Commander; an FRA deck (`fra_mode::is_fra_game`)
/// is FRA; any other duel reads SOS's pool (the ladder's `--decks sealed`,
/// and the closest of the three to a constructed duel).
pub fn pool_for(g: &GameState, vocab: &Vocab) -> &'static PoolIx {
    let commander = g.players.len() > 2 || g.players.iter().any(|p| !p.command.is_empty());
    let mode = if commander {
        Mode::Pod(4)
    } else if crate::fra_mode::is_fra_game(g) {
        Mode::SealedFra
    } else {
        Mode::SealedSos
    };
    mode_pool(mode, vocab)
}

/// The opening state of a `mode` game, decks drawn from `seed`.
pub fn mode_template(mode: Mode, seed: u64) -> GameState {
    let mut rng = StdRng::seed_from_u64(seed ^ 0x7AB1_E5EE_D000_0001);
    match mode {
        Mode::SealedSos | Mode::SealedFra => {
            let set = if mode == Mode::SealedFra { crate::fra_mode::SealedSet::Fra } else { crate::fra_mode::SealedSet::Sos };
            let packs = crate::selfplay::SEALED_PACKS;
            let a = crate::selfplay::sealed_pool_of(set, seed ^ 1, packs);
            let b = crate::selfplay::sealed_pool_of(set, seed ^ 2, packs);
            let da = crate::selfplay::heuristic_sealed_build(&a, seed ^ 3);
            let db = crate::selfplay::heuristic_sealed_build(&b, seed ^ 4);
            crate::selfplay::sealed_game_template(&da, &db)
        }
        Mode::Cube => {
            let da = crate::cube::cube_deck(crate::cube::random_color_pair(&mut rng), &mut rng);
            let db = crate::cube::cube_deck(crate::cube::random_color_pair(&mut rng), &mut rng);
            crate::selfplay::sealed_game_template(&da, &db)
        }
        Mode::Pod(n) => {
            let all = pod_decks();
            let mut pick: Vec<usize> = (0..all.len()).filter(|&i| !is_held_out_deck(i)).collect();
            pick.shuffle(&mut rng);
            let decks: Vec<PodDeck> = pick.into_iter().take(n.clamp(2, 6)).map(|i| all[i]).collect();
            crate::pod::build_pod_template(&decks)
        }
    }
}

/// A mode drawn for one game: duels and pods by the weights of the
/// 2026-09-28 plan — ~40 % of *rows* from duels, ~60 % from pods, a pod
/// making roughly a duel's rows per seat — so games are drawn ~2:1 duels.
pub fn draw_mode(rng: &mut StdRng) -> Mode {
    match rng.random_range(0..100u32) {
        0..=21 => Mode::SealedSos,
        22..=43 => Mode::SealedFra,
        44..=65 => Mode::Cube,
        66..=73 => Mode::Pod(2),
        74..=81 => Mode::Pod(3),
        82..=93 => Mode::Pod(4),
        _ => Mode::Pod(6),
    }
}

/// One recorded table game.
pub struct TableGame {
    pub rows: Vec<TableRow>,
    pub outcome: PodOutcome,
}

/// Play `template` with `pilots` (one per seat) and record every living
/// seat's view at each snapshot point, labelled — see the module docs.
pub fn record_table_game(
    template: &GameState,
    pilots: &[Pilot],
    seed: u64,
    max_actions: usize,
    vocab: &Vocab,
    pool: &PoolIx,
) -> TableGame {
    let n = template.players.len();
    let start_life: Vec<i32> = template.players.iter().map(|p| p.life).collect();
    // (seat, turn, row without labels)
    let mut snaps: Vec<(usize, u32, TableRow)> = Vec::new();
    let mut last_turn = (u32::MAX, usize::MAX);
    let mut last_step = TurnStep::Untap;
    let mut last_life: Vec<i32> = start_life.clone();
    let mut alive_at_end: Vec<bool> = vec![true; n];
    let mut end_turn = 0u32;
    let mut observe = |g: &GameState| {
        end_turn = g.turn_number;
        for (i, p) in g.players.iter().enumerate() {
            last_life[i] = p.life;
            alive_at_end[i] = p.is_alive();
        }
        let new_turn = (g.turn_number, g.active_player_idx) != last_turn;
        let step_point = g.step != last_step && matches!(g.step, TurnStep::PostCombatMain | TurnStep::End);
        last_step = g.step;
        if !(new_turn || step_point) || g.is_game_over() {
            return;
        }
        last_turn = (g.turn_number, g.active_player_idx);
        for seat in 0..n {
            if !g.players[seat].is_alive() {
                continue;
            }
            snaps.push((seat, g.turn_number, table_row_unlabelled(g, seat, vocab, pool)));
        }
    };
    let outcome =
        crate::pod::play_pod_game_observed(template, pilots, max_actions, seed, None, None, Some(&mut observe));
    let winner = outcome.winner;
    let turns = outcome.turns.max(end_turn);
    let mut ply = vec![0u16; n];
    let rows = snaps
        .into_iter()
        .map(|(seat, turn, mut row)| {
            let k = row.seat_ids.len();
            row.win = vec![0.0; k];
            match winner {
                Some(w) => {
                    if let Some(j) = row.seat_ids.iter().position(|&s| s as usize == w) {
                        row.win[j] = 1.0;
                    }
                }
                None => {
                    let standing: Vec<usize> = (0..k).filter(|&j| alive_at_end[row.seat_ids[j] as usize]).collect();
                    for &j in &standing {
                        row.win[j] = 1.0 / standing.len() as f32;
                    }
                }
            }
            row.life = row
                .seat_ids
                .iter()
                .map(|&s| {
                    let s = s as usize;
                    if !alive_at_end[s] { 0.0 } else { (last_life[s] as f32 / start_life[s].max(1) as f32).clamp(0.0, 1.5) }
                })
                .collect();
            row.game_len = turns.saturating_sub(turn) as f32 / 15.0;
            row.traj = ((seed as u32) << 3) | seat as u32;
            row.ply = ply[seat];
            ply[seat] += 1;
            row
        })
        .collect();
    TableGame { rows, outcome }
}

/// `seat`'s view of `g` with the pool and the hidden truth; labels unset.
pub fn table_row_unlabelled(g: &GameState, seat: usize, vocab: &Vocab, pool: &PoolIx) -> TableRow {
    let mut state = encode_table(g, seat, vocab);
    state.pool = Some(pool.pool.clone());
    let slots = table_slots(g, seat);
    let mut hand = Vec::with_capacity(slots.len());
    let mut library = Vec::with_capacity(slots.len());
    for (k, &p) in slots.iter().enumerate() {
        if k == 0 {
            hand.push(Vec::new());
            library.push(Vec::new());
            continue;
        }
        let pl = &g.players[p];
        hand.push(pl.hand.iter().filter_map(|c| pool.of(c.definition.name)).collect());
        let mut counts: Vec<(u16, u16)> = Vec::new();
        for c in pl.library.iter() {
            let Some(i) = pool.of(c.definition.name) else { continue };
            match counts.iter_mut().find(|(j, _)| *j == i) {
                Some(e) => e.1 += 1,
                None => counts.push((i, 1)),
            }
        }
        counts.sort_unstable();
        library.push(counts);
    }
    TableRow {
        state,
        win: Vec::new(),
        life: Vec::new(),
        game_len: 0.0,
        traj: 0,
        ply: 0,
        seat_ids: slots.iter().map(|&s| s as u8).collect(),
        hand,
        library,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mode records a game: rows from every seat, labels that sum to
    /// one per row, slot 0's own hidden truth empty, opponents' hands and
    /// libraries named in the pool, and the pool shared by every row.
    #[test]
    fn every_mode_records_labelled_rows_with_the_hidden_truth() {
        let vocab = crate::server::net_eval::vocab();
        for (i, mode) in [Mode::SealedSos, Mode::SealedFra, Mode::Cube, Mode::Pod(4)].into_iter().enumerate() {
            let seed = 31 + i as u64;
            let t = mode_template(mode, seed);
            let pool = mode_pool(mode, vocab);
            let pilots = vec![Pilot::default(); t.players.len()];
            let game = record_table_game(&t, &pilots, seed, 3_000, vocab, pool);
            assert!(!game.rows.is_empty(), "{mode:?}: no rows");
            let seats: std::collections::HashSet<u8> = game.rows.iter().map(|r| r.seat_ids[0]).collect();
            assert_eq!(seats.len(), t.players.len(), "{mode:?}: every seat records");
            let mut opp_cards = 0usize;
            for r in &game.rows {
                let k = r.state.seat_count();
                assert_eq!((r.win.len(), r.life.len(), r.seat_ids.len(), r.hand.len(), r.library.len()), (k, k, k, k, k));
                let total: f32 = r.win.iter().sum();
                assert!(total == 0.0 || (total - 1.0).abs() < 1e-5, "{mode:?}: win shares {:?}", r.win);
                assert!(r.hand[0].is_empty() && r.library[0].is_empty(), "slot 0's own cards are not a target");
                assert!(Arc::ptr_eq(r.state.pool.as_ref().unwrap(), &pool.pool));
                for j in 1..k {
                    opp_cards += r.hand[j].len() + r.library[j].iter().map(|x| x.1 as usize).sum::<usize>();
                    assert!(r.hand[j].iter().all(|&c| (c as usize) < pool.len()));
                }
            }
            assert!(opp_cards > 0, "{mode:?}: the opponents' cards are in the pool");
            // The rows survive the shard format.
            let back = crabomination_nn::table::read_table_shard(&crabomination_nn::table::write_table_shard(&game.rows)).expect("decodes");
            assert_eq!(back.len(), game.rows.len());
            assert_eq!(back[0].hand, game.rows[0].hand);
        }
    }

    #[test]
    fn about_three_percent_of_the_decks_are_held_out() {
        let n = pod_decks().len();
        let held = (0..n).filter(|&i| is_held_out_deck(i)).count();
        assert!(held * 100 >= n * 3 && held * 100 <= n * 4, "{held} of {n}");
    }
}
