//! N-player Commander pods for bot self-play — the format's smoke test.
//!
//! [`crate::recommend`]'s play loop is two-seat by signature (`[Pilot; 2]`,
//! `build_match_template(seat0, seat1)`), and a Commander game is three or four
//! seats with a command zone seated before the first draw. This module is the
//! N-seat sibling: same fixed points (`stop_reason`, `STALE_ROUNDS`, the
//! settled-state adoption), same seeded-shuffle discipline, different arity.
//!
//! It is not on the 2-player throughput path and does not touch it.

pub mod decks;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{RngExt, SeedableRng};

use crate::cube::CardFactory;
use crate::game::GameState;
use crate::player::Player;
use crate::recommend::{ActionCensus, Pilot, StopReason, stop_reason};
use crate::server::bot::Bot;

/// One Commander deck: the commander(s) that start in the command zone plus
/// the rest of the 99. Kept as factories so a pod template is built once and
/// cloned per game, the way [`crate::recommend::build_match_template`] is.
#[derive(Clone, Copy)]
pub struct PodDeck {
    pub name: &'static str,
    /// One commander, or two under Partner / Background (CR 702.124).
    pub commanders: &'static [CardFactory],
    /// The rest of the deck — `commanders.len() + main.len()` must be 100.
    pub main: &'static [CardFactory],
}

impl PodDeck {
    /// CR 903.5a — the deck is exactly 100 cards, commanders included.
    pub fn card_count(&self) -> usize {
        self.commanders.len() + self.main.len()
    }
}

/// The total-action backstop, as a multiple of the play budget: a game that
/// passes this many times per allowed play is stuck, whatever its plays say.
/// Passes run ~10-65 per play from four seats to thirty-four.
const PASSES_PER_PLAY_BACKSTOP: usize = 200;

/// One finished pod game.
#[derive(Debug, Clone, Copy)]
pub struct PodOutcome {
    /// The surviving seat, or `None` for a draw or an undecided game.
    pub winner: Option<usize>,
    pub actions: usize,
    /// The accepted actions that were not a priority pass. Passes are
    /// ~90 % of a 34-seat game (every stack item and step waits on every
    /// live seat), so they grow with seats squared; plays grow with seats.
    pub plays: usize,
    pub turns: u32,
    pub stop: StopReason,
}

/// What a batch of pod games did. The Commander smoke test's whole report:
/// a panic aborts the process, so everything here is about the games that
/// finished and the ones that did not decide.
#[derive(Debug, Default, Clone)]
pub struct PodTally {
    pub games: u32,
    /// Wins by seat index.
    pub wins: Vec<u32>,
    /// `is_game_over()` with no winner (CR 104.4 draw).
    pub draws: u32,
    /// Ran out of the action budget.
    pub action_capped: u32,
    /// Blew past `MAX_BATTLEFIELD`.
    pub board_capped: u32,
    /// [`crate::recommend::STALE_ROUNDS`] rounds with no bot able to move.
    pub no_legal_move: u32,
    pub total_turns: u64,
    pub total_actions: u64,
    pub total_plays: u64,
    /// The single longest game's `(actions, turns)` — a cap is read against
    /// this: a capped game near it is a long game, one far below it a loop.
    pub longest: (usize, u32),
    /// `(game index, turns, actions)` of every undecided game, so a stall can
    /// be replayed (`bot_ladder --commander --first I --games 1`).
    pub undecided_games: Vec<(u32, u32, usize)>,
}

impl PodTally {
    fn record(&mut self, index: u32, o: &PodOutcome) {
        self.games += 1;
        self.total_turns += u64::from(o.turns);
        self.total_actions += o.actions as u64;
        self.total_plays += o.plays as u64;
        self.longest = self.longest.max((o.actions, o.turns));
        if !(o.stop == StopReason::GameOver && o.winner.is_some()) {
            self.undecided_games.push((index, o.turns, o.actions));
        }
        match o.stop {
            StopReason::GameOver => match o.winner {
                Some(s) => {
                    if let Some(n) = self.wins.get_mut(s) {
                        *n += 1;
                    }
                }
                None => self.draws += 1,
            },
            StopReason::ActionCap => self.action_capped += 1,
            StopReason::BoardCap => self.board_capped += 1,
            StopReason::NoLegalMove => self.no_legal_move += 1,
        }
    }

    /// Games that ended without a winner, for any reason. The number the
    /// smoke test watches.
    pub fn undecided(&self) -> u32 {
        self.draws + self.action_capped + self.board_capped + self.no_legal_move
    }

    /// Fold another worker's tally in. Order-independent, so the smoke
    /// test's thread split does not change what it reports.
    pub fn merge(&mut self, other: &PodTally) {
        self.games += other.games;
        if self.wins.len() < other.wins.len() {
            self.wins.resize(other.wins.len(), 0);
        }
        for (a, b) in self.wins.iter_mut().zip(&other.wins) {
            *a += b;
        }
        self.draws += other.draws;
        self.action_capped += other.action_capped;
        self.board_capped += other.board_capped;
        self.no_legal_move += other.no_legal_move;
        self.total_turns += other.total_turns;
        self.total_actions += other.total_actions;
        self.total_plays += other.total_plays;
        self.longest = self.longest.max(other.longest);
        self.undecided_games.extend_from_slice(&other.undecided_games);
        self.undecided_games.sort_unstable();
    }

    pub fn mean_turns(&self) -> f64 {
        if self.games == 0 { 0.0 } else { self.total_turns as f64 / f64::from(self.games) }
    }
}

/// The target decks a Commander pod run plays. Each is a legal 100-card list
/// (`cr_903_5a_target_decks_are_legal_commander_decks` proves it), so the pod
/// run is a smoke test of the format and not of the deck builder.
pub fn target_decks() -> Vec<PodDeck> {
    vec![
        PodDeck { name: "Sigarda (GW)", commanders: decks::SIGARDA_COMMANDERS, main: decks::SIGARDA_MAIN },
        PodDeck { name: "Judith (BR)", commanders: decks::JUDITH_COMMANDERS, main: decks::JUDITH_MAIN },
        PodDeck { name: "Hanna (UW)", commanders: decks::HANNA_COMMANDERS, main: decks::HANNA_MAIN },
        PodDeck { name: "Tatyova (GU)", commanders: decks::TATYOVA_COMMANDERS, main: decks::TATYOVA_MAIN },
        PodDeck { name: "Krark/Rograkh (R)", commanders: decks::KRARK_COMMANDERS, main: decks::KRARK_MAIN },
        // Last on purpose, exactly as the Partner seat above is: appending
        // leaves `pod_field(4)` and `pod_field(5)` — and the committed outcome
        // table — the games they already were. `--seats 6` is what reaches it.
        PodDeck { name: "Edgar Markov (BRW)", commanders: decks::EDGAR_COMMANDERS, main: decks::EDGAR_MAIN },
        // Seventh, same reason again: the pod's only seat led by a
        // **non-creature** (CR 903.3a). `--seats 7` is what reaches it.
        PodDeck {
            name: "Freyalise (G)",
            commanders: decks::FREYALISE_COMMANDERS,
            main: decks::FREYALISE_MAIN,
        },
        // Eighth, appended for the third time for the same reason: the pod's
        // only **Choose a Background** pair (CR 702.124k), whose second
        // commander is a legendary enchantment rather than a creature.
        // `--seats 8` is what reaches it.
        PodDeck {
            name: "Zellix + Background (UR)",
            commanders: decks::ZELLIX_COMMANDERS,
            main: decks::ZELLIX_MAIN,
        },
        // Ninth, appended for the fourth time for the same reason: the pod's
        // only **commander ninjutsu** seat (CR 702.49d), and the only one
        // whose commander leaves the command zone by an action that is not a
        // cast. `--seats 9` is what reaches it.
        PodDeck {
            name: "Yuriko (UB)",
            commanders: decks::YURIKO_COMMANDERS,
            main: decks::YURIKO_MAIN,
        },
        // Tenth, appended for the fifth time for the same reason: the pod's
        // only seat that plays the **multiplayer-native** mechanics — the
        // monarch (CR 725), goad (CR 701.15), melee (CR 702.121),
        // voting (CR 701.38), tempting offer and join forces (CR 207.2c),
        // the initiative (CR 726) and myriad (CR 702.116) — none of which
        // any of the nine above had ever piloted. `--seats 10` reaches it.
        PodDeck {
            name: "Adriana (RW)",
            commanders: decks::ADRIANA_COMMANDERS,
            main: decks::ADRIANA_MAIN,
        },
        // Eleventh, appended for the sixth time for the same reason: the
        // pod's first **official precon** (Sultai Arisen, TDC) and its
        // graveyard seat. `--seats 11` reaches it.
        PodDeck {
            name: "Teval (BGU)",
            commanders: decks::TEVAL_COMMANDERS,
            main: decks::TEVAL_MAIN,
        },
        // Twelfth, appended for the seventh time: the second official precon
        // (Mind Flayarrrs, CLB), Horror mill-and-steal. `--seats 12`.
        PodDeck {
            name: "N'ghathrod (UB)",
            commanders: decks::NGHATHROD_COMMANDERS,
            main: decks::NGHATHROD_MAIN,
        },
        // Thirteenth: the third official precon (Blood Rites, LCC), Vampire
        // aristocrats. `--seats 13`.
        PodDeck {
            name: "Clavileño (WB)",
            commanders: decks::CLAVILENO_COMMANDERS,
            main: decks::CLAVILENO_MAIN,
        },
        // Fourteenth: the fourth official list (Heads I Win, Tails You Lose,
        // SLD) — coin flips, and the pod's "Partner with" pair. `--seats 14`.
        PodDeck {
            name: "Zndrsplt/Okaun (UR)",
            commanders: decks::ZNDRSPLT_COMMANDERS,
            main: decks::ZNDRSPLT_MAIN,
        },
        // Fifteenth: the fifth official list (Goblin Storm, SLD) — rituals and
        // Zada's copy-onto-the-team. `--seats 15`.
        PodDeck {
            name: "Zada (R)",
            commanders: decks::ZADA_COMMANDERS,
            main: decks::ZADA_MAIN,
        },
        // Sixteenth: the sixth official list (Wretched Ranks, FDC) — mono-black
        // Zombie tokens, the field's first 33-basic mana base. `--seats 16`.
        PodDeck {
            name: "Gisa (B)",
            commanders: decks::GISA_COMMANDERS,
            main: decks::GISA_MAIN,
        },
        // Seventeenth: the seventh official list (Tramplesaurus Rex, FDC) —
        // mono-green stompy, the field's biggest commander. `--seats 17`.
        PodDeck {
            name: "Ghalta (G)",
            commanders: decks::GHALTA_COMMANDERS,
            main: decks::GHALTA_MAIN,
        },
        // Eighteenth: the eighth official list (Keen Engineering, FDC) —
        // mono-blue artifacts and Vehicles. `--seats 18`.
        PodDeck {
            name: "Sai (U)",
            commanders: decks::SAI_COMMANDERS,
            main: decks::SAI_MAIN,
        },
        // Nineteenth: the ninth official list (Reap the Tides, CMR) — Simic
        // lands-matter under Aesi. `--seats 19`.
        PodDeck {
            name: "Aesi (GU)",
            commanders: decks::AESI_COMMANDERS,
            main: decks::AESI_MAIN,
        },
        // Twentieth: the tenth official list (Corrupting Influence, ONC) —
        // the field's poison deck: toxic, infect, proliferate. `--seats 20`.
        PodDeck {
            name: "Ixhel (WBG)",
            commanders: decks::IXHEL_COMMANDERS,
            main: decks::IXHEL_MAIN,
        },
        // Twenty-first: the eleventh official list (Sneak Attack, ZNC) —
        // Dimir Rogues that mill what they hit. `--seats 21`.
        PodDeck {
            name: "Anowon (UB)",
            commanders: decks::ANOWON_COMMANDERS,
            main: decks::ANOWON_MAIN,
        },
        // Twenty-second: the twelfth official list (Jeskai Striker, TDC) —
        // spellslinging and flurry. `--seats 22`.
        PodDeck {
            name: "Shiko and Narset (URW)",
            commanders: decks::SHIKO_COMMANDERS,
            main: decks::SHIKO_MAIN,
        },
        // Twenty-third: the thirteenth official list (Sliver Swarm, CMM) — the
        // field's five-color deck, Slivers granting to the hive. `--seats 23`.
        PodDeck {
            name: "Sliver Gravemother (WUBRG)",
            commanders: decks::GRAVEMOTHER_COMMANDERS,
            main: decks::GRAVEMOTHER_MAIN,
        },
        // Twenty-fourth: the fourteenth official list (Quick Draw, OTC) —
        // Izzet storm and cascade under Stella Lee. `--seats 24`.
        PodDeck {
            name: "Stella Lee (UR)",
            commanders: decks::STELLA_COMMANDERS,
            main: decks::STELLA_MAIN,
        },
        // Twenty-fifth: the fifteenth official list (Vampiric Bloodlust,
        // C17) — Edgar's own precon, the field's second Edgar seat, with the
        // curses that pay whoever attacks the cursed player. `--seats 25`.
        PodDeck {
            name: "Edgar Markov C17 (BRW)",
            commanders: decks::EDGAR_C17_COMMANDERS,
            main: decks::EDGAR_C17_MAIN,
        },
        // Twenty-sixth: the sixteenth official list (Calling All Angels, FDC)
        // — the field's first mono-white seat. `--seats 26`.
        PodDeck {
            name: "Giada (W)",
            commanders: decks::GIADA_COMMANDERS,
            main: decks::GIADA_MAIN,
        },
        // Twenty-seventh: the seventeenth official list (Guided by Nature, C14) —
        // mono-green Elves under a planeswalker commander. `--seats 27`.
        PodDeck {
            name: "Freyalise C14 (G)",
            commanders: decks::FREYALISE_C14_COMMANDERS,
            main: decks::FREYALISE_C14_MAIN,
        },
        // Twenty-eighth: the eighteenth official list (Animated Army, BLC)
        // — the field's first Gruul seat. `--seats 28`.
        PodDeck {
            name: "Bello (RG)",
            commanders: decks::BELLO_COMMANDERS,
            main: decks::BELLO_MAIN,
        },
        // Twenty-ninth: the nineteenth official list (Grave Danger, SCD) —
        // Dimir Zombies recast from the graveyard. `--seats 29`.
        PodDeck {
            name: "Gisa and Geralf (UB)",
            commanders: decks::GISA_GERALF_COMMANDERS,
            main: decks::GISA_GERALF_MAIN,
        },
        // Thirtieth: the twentieth official list (Forged in Stone, C14) —
        // mono-white Equipment under a planeswalker commander. `--seats 30`.
        PodDeck {
            name: "Nahiri (W)",
            commanders: decks::NAHIRI_COMMANDERS,
            main: decks::NAHIRI_MAIN,
        },
        // Thirty-first: the twenty-first official list (Graveyard Overdrive,
        // M3C) — the field's first Jund seat. `--seats 31`.
        PodDeck {
            name: "Disa (BRG)",
            commanders: decks::DISA_COMMANDERS,
            main: decks::DISA_MAIN,
        },
        // Thirty-second: the twenty-second official list (Swell the Host, C15) —
        // Simic counters and myriad. `--seats 32`.
        PodDeck {
            name: "Ezuri, Claw of Progress (GU)",
            commanders: decks::EZURI_COMMANDERS,
            main: decks::EZURI_MAIN,
        },
        // Thirty-third: the twenty-third official list (Angels, SLD) —
        // mono-white Angels; Gisela melds into Brisela. `--seats 33`.
        PodDeck {
            name: "Gisela, the Broken Blade (W)",
            commanders: decks::GISELA_COMMANDERS,
            main: decks::GISELA_MAIN,
        },
        // Thirty-fourth: the twenty-fourth official list (Built From
        // Scratch, C14) — mono-red artifacts, a planeswalker commander.
        // `--seats 34`.
        PodDeck {
            name: "Daretti, Scrap Savant (R)",
            commanders: decks::DARETTI_COMMANDERS,
            main: decks::DARETTI_MAIN,
        },
        // Thirty-fifth: the twenty-fifth official list (Sworn to Darkness,
        // C14) — mono-black Demons under a planeswalker commander.
        // `--seats 35`.
        PodDeck {
            name: "Ob Nixilis (B)",
            commanders: decks::OB_NIXILIS_COMMANDERS,
            main: decks::OB_NIXILIS_MAIN,
        },
        // Thirty-sixth: the twenty-sixth official list (Vampiric Bloodline,
        // VOC) — Rakdos Vampires and Blood tokens. `--seats 36`.
        PodDeck {
            name: "Strefan, Maurer Progenitor (BR)",
            commanders: decks::STREFAN_COMMANDERS,
            main: decks::STREFAN_MAIN,
        },
        // Thirty-seventh: the twenty-seventh official list (Plunder the
        // Graves, C15) — Golgari sacrifice and recursion. `--seats 37`.
        PodDeck {
            name: "Meren of Clan Nel Toth (BG)",
            commanders: decks::MEREN_COMMANDERS,
            main: decks::MEREN_MAIN,
        },
        // Thirty-eighth: the twenty-eighth official list (Seize Control,
        // C15) — Izzet spells under Mizzix. `--seats 38`.
        PodDeck {
            name: "Mizzix of the Izmagnus (UR)",
            commanders: decks::MIZZIX_COMMANDERS,
            main: decks::MIZZIX_MAIN,
        },
        // Thirty-ninth: the twenty-ninth official list (Quantum Quandrix,
        // C21) — Simic Fractals under a token doubler. `--seats 39`.
        PodDeck {
            name: "Adrix and Nev (GU)",
            commanders: decks::ADRIX_NEV_COMMANDERS,
            main: decks::ADRIX_NEV_MAIN,
        },
    ]
}

/// `seats` decks for a pod, cycling [`target_decks`] when there are fewer
/// lists than seats. A pod of identical lists is still a valid smoke test —
/// mirror pods are what the two-player ladder uses for the same reason.
pub fn pod_field(seats: usize) -> Vec<PodDeck> {
    let decks = target_decks();
    (0..seats).map(|i| decks[i % decks.len()]).collect()
}

/// An unshuffled N-seat Commander state with every library loaded and every
/// command zone seated — the clone-me template for [`play_one_pod_game`].
///
/// `apply_format` runs before the commanders are seated so the 40-life and
/// multiplayer-draw rules are in place when the zone is populated.
pub fn build_pod_template(decks: &[PodDeck]) -> GameState {
    let seats: Vec<SeatDeck<'_>> = decks.iter().map(SeatDeck::from).collect();
    build_pod_template_from(&seats)
}

/// One seat's Commander deck by reference — a [`PodDeck`], or a list the
/// caller owns (a decklist imported in the client). Same shape and same
/// 100-card contract as [`PodDeck`]; it just doesn't have to be `'static`.
#[derive(Clone, Copy)]
pub struct SeatDeck<'a> {
    pub commanders: &'a [CardFactory],
    pub main: &'a [CardFactory],
}

impl From<&PodDeck> for SeatDeck<'static> {
    fn from(d: &PodDeck) -> Self {
        SeatDeck { commanders: d.commanders, main: d.main }
    }
}

/// [`build_pod_template`] over arbitrary seat decks, so a pod can seat a
/// player's own list beside the stock [`target_decks`].
pub fn build_pod_template_from(decks: &[SeatDeck<'_>]) -> GameState {
    let players = (0..decks.len()).map(|i| Player::new(i, format!("Seat {i}"))).collect();
    let mut g = GameState::new(players);
    g.apply_format(crate::format::Format::Commander);
    for (seat, deck) in decks.iter().enumerate() {
        for &f in deck.main {
            g.add_card_to_library(seat, crate::cube::card_arc(f));
        }
        g.seat_commanders(seat, deck.commanders.iter().map(|f| f()).collect());
        g.players[seat].wants_ui = true;
    }
    g
}

/// Play one seeded pod game from a prebuilt template.
///
/// The seat loop is [`crate::recommend`]'s, generalised past two seats: poll
/// each live seat in turn, adopt the bot's settled state when it hands one
/// back, and stop on the first of game-over / action cap / board cap /
/// staleness that [`stop_reason`] reports.
pub fn play_one_pod_game(
    template: &GameState,
    pilots: &[Pilot],
    max_actions: usize,
    seed: u64,
) -> PodOutcome {
    play_one_pod_game_censused(template, pilots, max_actions, seed, None)
}

/// [`play_one_pod_game`] with the action census handed in, so a caller can
/// total across games. `None` keeps the `CRAB_CAP_DIAG` behaviour: a census
/// armed for this game only, rendered if the game did not decide.
/// `max_actions` is the budget in plays (non-pass actions); see
/// `PodOutcome::plays`.
pub fn play_one_pod_game_censused(
    template: &GameState,
    pilots: &[Pilot],
    max_actions: usize,
    seed: u64,
    into: Option<&mut ActionCensus>,
) -> PodOutcome {
    crate::server::bot::set_jitter_seed(Some(seed));
    let mut g = template.clone();
    let mut shuffle = StdRng::seed_from_u64(seed);
    // `zip`, not `players[seat]`: a caller that hands over more pilots than
    // the template has seats gets the extras ignored rather than a panic.
    for (player, pilot) in g.players.iter_mut().zip(pilots) {
        if let Some(w) = pilot.weights() {
            player.smart_tap = w.smart_tap;
            player.converge_rarest = w.converge_rarest;
        }
        player.hostile_player_targets = match pilot {
            Pilot::Scored(w) => w.hostile_player_targets,
            Pilot::Mcts(cfg) => cfg.weights.hostile_player_targets,
            Pilot::Uniform => false,
        };
        player.library.shuffle(&mut shuffle);
    }
    // A seeded deal implies a seeded game — mulligan reshuffles and every
    // other in-game roll come off the state's own stream (see
    // `play_one_game_traced`, which this mirrors).
    g.rng.reseed(shuffle.random());
    g.start_mulligan_phase();

    let mut bots: Vec<Box<dyn Bot>> =
        pilots.iter().take(g.players.len()).map(|p| p.build()).collect();
    let (mut actions, mut plays, mut stale) = (0usize, 0usize, 0usize);
    let (diag_floor, mut diag_said) = (crate::recommend::cap_diag_floor().flatten(), false);
    // One `OnceLock` read a game, not a bool per action: off, `record` is a
    // field test and the `Debug` format below never runs.
    let mut census =
        if into.is_some() { ActionCensus::forced() } else { ActionCensus::armed() };
    // The budget counts plays, not priority passes (`PodOutcome::plays`), with
    // a total-action backstop far above anything a play budget allows.
    let spent = |actions: usize, plays: usize| {
        if actions >= max_actions.saturating_mul(PASSES_PER_PLAY_BACKSTOP) { max_actions } else { plays }
    };
    while stop_reason(&g, spent(actions, plays), max_actions, stale).is_none() {
        let mut any = false;
        for (seat, bot) in bots.iter_mut().enumerate() {
            // Eliminated seats are polled like any other: the bot answers
            // `None` for a seat without priority, and CR 800.4a has already
            // taken their objects and any decision addressed to them. Skipping
            // them here instead was a deadlock — a pending decision the loop
            // never polled suppressed every other seat's actions.
            let Some(step) = bot.next_action_settled(&g, seat) else { continue };
            let crate::server::bot::BotStep { action, settled } = step;
            let is_pass = matches!(action, crate::game::GameAction::PassPriority);
            // Keyed against the pre-action state: a cast names the card while
            // it is still in the zone it is cast from.
            let key = census.key_for(&g, seat, &action);
            let ok = if let Some(settled) = settled {
                g = *settled;
                true
            } else {
                match g.perform_action(action) {
                    Ok(events) => {
                        g.recycle_events(events);
                        true
                    }
                    Err(_) => false,
                }
            };
            if ok {
                census.bump(key);
                any = true;
                actions += 1;
                plays += usize::from(!is_pass);
                if g.is_game_over() {
                    break;
                }
            }
        }
        // CR 800.4f/g/h — no ask may be owed by a seat that has left the game.
        // `seat_prompts` and `route_ask` are what keep it so, and this is the
        // audit over real boards rather than against a re-derived list: the
        // whole class was found by a run of this check as an env-var probe
        // (88 of 2,000 four-seat games), and a new ask site that forgets the
        // rule reintroduces it silently. Debug-only, one `Option` test a round.
        debug_assert!(
            g.pending_decision
                .as_ref()
                .is_none_or(|pd| g.players[pd.acting_player()].is_alive()),
            "seed {seed}: seat {} has left the game and still owes an ask",
            g.pending_decision.as_ref().map(|pd| pd.acting_player()).unwrap_or(usize::MAX),
        );
        if any { stale = 0 } else { stale += 1 }
        // `CRAB_CAP_DIAG=<n>` names a *slow* game's board too, once, as it
        // passes `n` actions — a decided game never reaches the line below.
        if let Some(n) = diag_floor.filter(|n| actions >= *n && !diag_said) {
            diag_said = true;
            eprintln!("pod past {n} actions seed {seed}: {}", crate::recommend::cap_diagnosis(&g, actions));
        }
    }
    crate::server::bot::set_jitter_seed(None);
    let stop = stop_reason(&g, spent(actions, plays), max_actions, stale).unwrap_or(StopReason::NoLegalMove);
    // `CRAB_CAP_DIAG` is the two-player loop's knob and it says the same thing
    // here: what was an undecided game actually doing. One `OnceLock` read a
    // game, on the undecided ones only.
    if crate::recommend::cap_diag_floor().is_some() && !matches!(stop, StopReason::GameOver) {
        eprintln!(
            "pod {stop:?} seed {seed}: {}\n  actions: {}",
            crate::recommend::cap_diagnosis(&g, actions),
            census.render(),
        );
    }
    if let Some(total) = into {
        total.merge(&census);
    }
    PodOutcome { winner: g.game_over.flatten(), actions, plays, turns: g.turn_number, stop }
}

/// Play games `first .. first + count`, rotating the decks through the seats
/// so turn order is not confounded with deck strength (seat 0 is worth a lot
/// in a pod). Game `i` puts deck `d` in seat `(d + i) % n`, and the tally
/// reports wins by *deck*.
///
/// Game `i`'s seed is a pure function of `seed_base` and `i`, so a worker
/// pool can split the range any way it likes and still reproduce the run.
pub fn run_pod_games(
    decks: &[PodDeck],
    first: u32,
    count: u32,
    seed_base: u64,
    max_actions: usize,
    pilot: Pilot,
) -> PodTally {
    run_pod_games_censused(decks, first, count, seed_base, max_actions, pilot, None)
}

/// [`run_pod_games`] with an action census totalled across the batch — the
/// deck-coverage run. Handing one in turns the census on for every game, so
/// this is the slower path and the smoke test does not take it.
pub fn run_pod_games_censused(
    decks: &[PodDeck],
    first: u32,
    count: u32,
    seed_base: u64,
    max_actions: usize,
    pilot: Pilot,
    mut census: Option<&mut ActionCensus>,
) -> PodTally {
    let n = decks.len();
    let mut tally = PodTally { wins: vec![0; n], ..Default::default() };
    if n < 2 {
        return tally;
    }
    let pilots = vec![pilot; n];
    // One template per rotation, not per game: a `GameState` clone is a
    // reference bump per zone where rebuilding 100 card definitions a seat
    // is not.
    let templates: Vec<GameState> = (0..n)
        .map(|rot| {
            let seated: Vec<PodDeck> = (0..n).map(|seat| decks[(seat + n - rot) % n]).collect();
            build_pod_template(&seated)
        })
        .collect();
    for i in first..first.saturating_add(count) {
        let rot = (i as usize) % n;
        let seed = seed_base.wrapping_add(u64::from(i).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let o = play_one_pod_game_censused(
            &templates[rot],
            &pilots,
            max_actions,
            seed,
            census.as_deref_mut(),
        );
        // Seat `s` in rotation `rot` holds deck `(s + n - rot) % n`.
        let by_deck = PodOutcome { winner: o.winner.map(|s| (s + n - rot) % n), ..o };
        tally.record(i, &by_deck);
    }
    tally
}

/// [`run_pod_games`] over `0 .. games`.
pub fn run_pod(
    decks: &[PodDeck],
    games: u32,
    seed_base: u64,
    max_actions: usize,
    pilot: Pilot,
) -> PodTally {
    run_pod_games(decks, 0, games, seed_base, max_actions, pilot)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rofellos_pod(seats: usize) -> Vec<PodDeck> {
        pod_field(seats)
    }

    /// CR 903.5a/903.5b — every target deck is 100 cards, singleton, on
    /// identity and led by a legal commander. The pod run is only a smoke
    /// test of the *format* if its decks are legal to begin with.
    #[test]
    fn cr_903_5a_target_decks_are_legal_commander_decks() {
        for d in target_decks() {
            assert_eq!(d.card_count(), 100, "{}: deck size", d.name);
            let deck = crate::format::Deck {
                main: d.main.iter().map(|f| f()).collect(),
                commanders: d.commanders.iter().map(|f| f()).collect(),
                ..Default::default()
            };
            if let Err(e) = crate::format::validate_commander_deck(&deck) {
                panic!("{} is not a legal Commander deck: {e:?}", d.name);
            }
        }
    }

    /// CR 903.7 — every seat starts with its commander in the command zone,
    /// and CR 903.6 gives every seat 40 life.
    #[test]
    fn cr_903_7_every_seat_starts_with_a_seated_commander() {
        let g = build_pod_template(&rofellos_pod(4));
        assert_eq!(g.players.len(), 4);
        for p in &g.players {
            assert_eq!(p.life, 40);
            assert_eq!(p.command.len(), 1);
            assert_eq!(p.commanders.len(), 1);
            assert_eq!(p.library.len(), 99);
        }
    }

    /// CR 702.124b/d — the partner seat is the pod's only two-commander deck,
    /// and it is here so the pair is exercised by real games rather than only
    /// by a fixture: both commanders start in the command zone, its 99 is 98
    /// (CR 702.124b counts both toward the 100), and the games finish.
    ///
    /// Krark/Rograkh sits past `pod_field(4)` in `target_decks`, so a four-seat
    /// pod never draws it and the committed outcome table is untouched by its
    /// existence; this test names the field explicitly. It is found by its
    /// *pairing* rather than by position — Edgar Markov was appended after it
    /// and `last()` silently stopped being the partner seat.
    ///
    /// ⚠ "Two commanders" stopped being unique when the Choose-a-Background
    /// seat was appended (CR 702.124k is also a pair), so the shape here is
    /// the one CR 702.124h names: two *creature* cards, each with Partner.
    #[test]
    fn cr_702_124b_a_two_commander_seat_plays_a_pod_game() {
        let field = target_decks();
        let partner = *field
            .iter()
            .find(|d| d.commanders.len() == 2 && d.commanders.iter().all(|f| f().is_creature()))
            .expect("a two-creature-commander deck");
        assert_eq!(partner.commanders.len(), 2);
        assert_eq!(partner.card_count(), 100, "CR 702.124b counts both commanders");

        let decks = vec![partner, field[0], field[1], field[2]];
        let t = build_pod_template(&decks);
        assert_eq!(t.players[0].command.len(), 2, "both begin in the command zone");
        assert_eq!(t.players[0].commanders.len(), 2);
        assert_eq!(t.players[0].library.len(), 98);
        for p in &t.players {
            assert_eq!(p.life, 40);
        }

        let pilots = vec![Pilot::default(); 4];
        for seed in [0xC0FFEE_u64, 43, 4242] {
            let o = play_one_pod_game(&t, &pilots, 50_000, seed);
            assert!(o.winner.is_some(), "seed {seed} left the pod undecided");
            assert!(o.turns > 0);
        }
    }

    /// CR 903.3a — "[this card] can be your commander" on a **non-creature**.
    /// `CardDefinition::can_be_commander` had been validated since it shipped
    /// and never piloted; this is the seat that pilots it. Found by its
    /// *shape* and not its position, for the reason the Partner test above
    /// records.
    ///
    /// Two consequences of a planeswalker commander and both hold: it is
    /// recast from the command zone under the CR 903.8 tax like any other,
    /// and its (commander, player) damage tally stays at zero for the whole
    /// game, because CR 903.10a counts **combat** damage and a planeswalker
    /// deals none — so the seat wins and loses by every other route instead.
    #[test]
    fn cr_903_3a_a_planeswalker_commander_seat_plays_a_pod_game() {
        let field = target_decks();
        // One commander, and it is not a creature: the Background seat's
        // second commander is also a non-creature, so the arity is what keeps
        // this pinned to the planeswalker.
        let pw = *field
            .iter()
            .find(|d| d.commanders.len() == 1 && !d.commanders[0]().is_creature())
            .expect("a lone non-creature commander");
        assert_eq!(pw.card_count(), 100);
        let def = pw.commanders[0]();
        assert!(def.can_be_commander, "CR 903.3a — the printed permission");
        assert!(!def.is_creature(), "and it is not a creature");

        let decks = vec![pw, field[0], field[1], field[3]];
        let t = build_pod_template(&decks);
        assert_eq!(t.players[0].command.len(), 1, "it begins in the command zone");
        assert_eq!(t.players[0].commanders.len(), 1);
        assert_eq!(t.players[0].library.len(), 99);

        let pilots = vec![Pilot::default(); 4];
        for seed in [0xC0FFEE_u64, 43, 4242] {
            let o = play_one_pod_game(&t, &pilots, 50_000, seed);
            assert!(o.winner.is_some(), "seed {seed} left the pod undecided");
            assert!(o.turns > 0);
        }
    }

    /// CR 702.124k — "Choose a Background": two cards lead the deck when one
    /// has the keyword and the other is a legendary Background enchantment.
    /// The keyword and `format::is_background_pair` shipped validated and
    /// never piloted; this is the seat that pilots them.
    ///
    /// What this asserts that the Partner test does not: the second commander
    /// is **not a creature** (CR 702.124k's Background half, which
    /// `is_legal_commander` rejects on its own), CR 702.124c combines the two
    /// identities across a creature and an enchantment, and the games finish.
    /// CR 702.124d's second tally is zero for the whole game by construction
    /// here — CR 903.10a counts combat damage and an enchantment deals none —
    /// which is the planeswalker seat's consequence reached from the other
    /// direction, and why this pair is worth running rather than the mono-red
    /// Gut + Archaeologist one.
    #[test]
    fn cr_702_124k_a_background_pair_plays_a_pod_game() {
        use crate::card::KeywordSlice;
        use crate::format::is_legal_commander;
        let field = target_decks();
        let bg = *field
            .iter()
            .find(|d| d.commanders.len() == 2 && !d.commanders[1]().is_creature())
            .expect("a Choose-a-Background deck");
        assert_eq!(bg.card_count(), 100, "CR 903.5a counts both commanders");

        let (lead, back) = (bg.commanders[0](), bg.commanders[1]());
        assert!(
            lead.keywords.has_kw(&crate::card::Keyword::ChooseABackground),
            "CR 702.124k — the first half carries the keyword",
        );
        assert!(back.is_enchantment() && back.is_legendary(), "and the second is a legendary enchantment");
        assert!(
            !is_legal_commander(&back),
            "CR 702.124k — a Background is not a commander on its own",
        );
        assert!(
            crate::format::commanders_may_pair(&lead, &back),
            "but the pair is legal together",
        );

        let decks = vec![bg, field[0], field[1], field[2]];
        let t = build_pod_template(&decks);
        assert_eq!(t.players[0].command.len(), 2, "both begin in the command zone");
        assert_eq!(t.players[0].commanders.len(), 2);
        assert_eq!(t.players[0].library.len(), 98);

        let pilots = vec![Pilot::default(); 4];
        for seed in [0xC0FFEE_u64, 43, 4242] {
            let o = play_one_pod_game(&t, &pilots, 50_000, seed);
            assert!(o.winner.is_some(), "seed {seed} left the pod undecided");
            assert!(o.turns > 0);
        }
    }

    /// CR 702.49d — **commander ninjutsu**: the one route out of the command
    /// zone that is not a cast, so CR 903.8's tax never applies to it. The
    /// keyword shipped validated and, until this seat, was never piloted.
    ///
    /// What this asserts that no other seat does: the commander begins in the
    /// command zone like any other, its `commander_cast_count` is untouched
    /// by the ninjutsu route, and the deck finishes seeded games. The 21-damage
    /// tally is live here where the planeswalker and Background seats have it
    /// at zero — a Ninja that connects is combat damage from a commander
    /// (CR 903.10a) — which is the third corner of the same square.
    #[test]
    fn cr_702_49d_a_commander_ninjutsu_seat_plays_a_pod_game() {
        use crate::card::{Keyword, KeywordSlice};
        let field = target_decks();
        let nin = *field
            .iter()
            .find(|d| {
                d.commanders.len() == 1
                    && d.commanders[0]()
                        .keywords
                        .iter()
                        .any(|k| matches!(k, Keyword::CommanderNinjutsu(_)))
            })
            .expect("a commander-ninjutsu deck");
        assert_eq!(nin.card_count(), 100);
        let def = nin.commanders[0]();
        assert!(def.is_creature(), "CR 702.49d rides a creature");
        assert!(
            !def.keywords.has_kw(&Keyword::Ninjutsu(Default::default())),
            "the command-zone keyword is its own, not plain ninjutsu",
        );

        let decks = vec![nin, field[0], field[1], field[2]];
        let t = build_pod_template(&decks);
        assert_eq!(t.players[0].command.len(), 1, "it begins in the command zone");
        assert_eq!(t.players[0].commanders.len(), 1);
        assert_eq!(t.players[0].library.len(), 99);
        assert!(
            t.commander_cast_count.is_empty(),
            "CR 903.8's tally starts empty and ninjutsu never adds to it",
        );

        let pilots = vec![Pilot::default(); 4];
        for seed in [0xC0FFEE_u64, 43, 4242] {
            let o = play_one_pod_game(&t, &pilots, 50_000, seed);
            assert!(o.winner.is_some(), "seed {seed} left the pod undecided");
            assert!(o.turns > 0);
        }
    }

    /// CR 903.5a/c — the eleventh to seventeenth seats are official lists
    /// taken card for card (Sultai Arisen, Mind Flayarrrs, Blood Rites, Heads I
    /// Win, Tails You Lose, Goblin Storm, Wretched Ranks, Tramplesaurus Rex). They seat what no
    /// hand-built list did — graveyard casts (Kotis, CR 601.2a), graveyard-only
    /// mana (Lord of the Forsaken, CR 106.6), a pay-one-of-three additional
    /// cost (Dusk Mangler, CR 601.2b), a trigger that works while suspended
    /// (Nihilith, CR 702.62b) — so a four-seat pod with each has to finish.
    #[test]
    fn cr_903_5a_the_official_precon_seats_play_pod_games() {
        let field = target_decks();
        for (name, seeds) in [
            ("Teval", [0x7E7A1_u64, 78, 9002]),
            ("N'ghathrod", [0xC1B, 79, 9003]),
            ("Clavileño", [0xB100D, 80, 9004]),
            ("Zndrsplt", [0xC0141, 81, 9005]),
            ("Zada", [0x2ADA, 82, 9006]),
            ("Gisa", [0x6154, 83, 9007]),
            ("Ghalta", [0x6A17A, 84, 9008]),
        ] {
            let precon = *field.iter().find(|d| d.name.starts_with(name)).expect("the precon seat");
            assert_eq!(precon.card_count(), 100);
            let decks = vec![precon, field[0], field[1], field[2]];
            let t = build_pod_template(&decks);
            assert_eq!(t.players[0].library.len(), 100 - precon.commanders.len());
            let pilots = vec![Pilot::default(); 4];
            for seed in seeds {
                let o = play_one_pod_game(&t, &pilots, 50_000, seed);
                assert!(o.winner.is_some(), "{name} seed {seed} left the pod undecided");
                assert!(o.turns > 0);
            }
        }
    }

    /// CR 702.121 (Melee) — the tenth seat is the field's only
    /// multiplayer-native list, so this asserts both halves: its commander
    /// grants melee to the rest of the board, and the 99 actually carries the
    /// mechanics that only mean something past two seats. A pod game at four
    /// seats then has to finish, because none of that code had ever run in
    /// self-play before this deck existed.
    #[test]
    fn cr_702_121_the_multiplayer_native_seat_plays_a_pod_game() {
        let field = target_decks();
        let mp = *field.iter().find(|d| d.name.starts_with("Adriana")).expect("the RW seat");
        assert_eq!(mp.card_count(), 100);
        let def = mp.commanders[0]();
        // Melee is modelled as its attack trigger rather than as
        // `Keyword::Melee` here, because the printed second line grants it.
        assert!(
            format!("{:?}", def.triggered_abilities).contains("OpponentsAttackedThisCombat"),
            "Adriana has melee herself",
        );
        assert_eq!(
            def.static_abilities.len(), 1,
            "and grants it to the other creatures, which is what makes the 99 wide",
        );

        // The mechanics the other nine lists never played. Each is a distinct
        // multiplayer rule, so a miss here means the seat stopped testing it.
        let texts: Vec<String> =
            mp.main.iter().map(|f| format!("{:?}", f())).collect();
        for (mechanic, needle) in [
            ("the monarch", "Monarch"),
            ("goad", "Goad"),
            ("melee", "Melee"),
            ("voting", "WillOfTheCouncil"),
            ("tempting offer", "TemptingOffer"),
            ("join forces", "JoinForces"),
            ("myriad", "Myriad"),
        ] {
            assert!(
                texts.iter().any(|t| t.contains(needle)),
                "the 99 stopped carrying {mechanic}",
            );
        }

        let decks = vec![mp, field[0], field[1], field[2]];
        let t = build_pod_template(&decks);
        assert_eq!(t.players[0].library.len(), 99);
        let pilots = vec![Pilot::default(); 4];
        for seed in [0xADD1A_u64, 77, 9001] {
            let o = play_one_pod_game(&t, &pilots, 50_000, seed);
            assert!(o.winner.is_some(), "seed {seed} left the pod undecided");
            assert!(o.turns > 0);
        }
    }

    /// The pod loop is reproducible: same seed, same outcome. Cross-process
    /// determinism is what the seeded smoke test rests on.
    #[test]
    fn a_seeded_pod_game_replays_identically() {
        let decks = rofellos_pod(3);
        let t = build_pod_template(&decks);
        let pilots = vec![Pilot::default(); 3];
        let a = play_one_pod_game(&t, &pilots, 3_000, 0xC0FFEE);
        let b = play_one_pod_game(&t, &pilots, 3_000, 0xC0FFEE);
        assert_eq!((a.winner, a.actions, a.turns), (b.winner, b.actions, b.turns));
    }

    /// The pod's half of the two-player `thread_determinism` gate: a chunk
    /// split is what a thread count changes, and nothing else, so a run
    /// split three ways must merge to what one chunk produced.
    ///
    /// The per-game seed is `seed_base + i * <golden>` rather than anything
    /// derived from the worker, which is what makes that true — this is the
    /// test that says so, and it fails the moment a future scheduler reaches
    /// for a per-worker stream. `--threads 1/2/3` over 400 four-seat games
    /// reads byte-identical at the tip; this is the same fact at suite cost.
    #[test]
    fn cr_903_a_pod_run_is_independent_of_how_it_is_chunked() {
        // Three seats and six games: the split is what is under test, not the
        // sample size, and a four-seat dozen costs the suite 55 s of critical
        // path for the same fact.
        let field = pod_field(3);
        let whole = run_pod_games(&field, 0, 6, 43, 20_000, Pilot::default());
        let mut split = PodTally { wins: vec![0; 3], ..Default::default() };
        for first in [0u32, 2, 4] {
            split.merge(&run_pod_games(&field, first, 2, 43, 20_000, Pilot::default()));
        }
        assert_eq!(whole.games, split.games);
        assert_eq!(whole.wins, split.wins, "the same seats win the same games");
        assert_eq!(whole.total_turns, split.total_turns);
        assert_eq!(whole.total_actions, split.total_actions);
        assert_eq!(whole.undecided(), split.undecided());
        // And the run is decided end to end, which is the smoke test's own
        // claim at a size the suite can carry.
        assert_eq!(whole.undecided(), 0, "six three-seat pods, all decided");
    }

    /// The Commander guardrail the two-player golden traces are, at pod
    /// scale: a committed outcome per fixed seed, so a change that moves pod
    /// play shows up here as a diff rather than as a number in a smoke-test
    /// log a reviewer has to remember. The committed values were produced by
    /// a different process on a different day, which makes this a
    /// cross-process determinism check too.
    ///
    /// The triple, not a line-per-action trace: a pod game is ~2,000 actions,
    /// so a real trace would be a 400 KB file that every Commander commit
    /// re-blesses, and the winner/turns/actions triple moves on exactly the
    /// changes a trace would move on.
    ///
    /// When a rules change legitimately moves one, re-bless it in the same
    /// commit and say why in the message.
    #[test]
    fn cr_903_seeded_pod_outcomes_match_the_committed_table() {
        // (seed, winner, turns, actions)
        // Re-blessed 2026-09-24 (CR 119.3, "you gain life equal to the life
        // lost this way" gains the table's total): Judith's Gray Merchant and
        // Kokusho drain three opponents, so seed 0xC0FFEE is 52→44 turns /
        // 2248→1829 actions, same winner; the other two unmoved. Aggregate,
        // 2,000 games at seed 9981, before/after: 45.83/45.86 turns at four
        // seats, 65.90/66.02 at six, every block 2,000/2,000 decided.
        // Re-blessed 2026-09-23 (CR 106.6, the payment floats a spend-
        // restricted source the spell may spend): seed 4242 55→53 turns /
        // 2436→2318 actions, same winner; the other two unmoved. Aggregate,
        // 2,000 games a seat count at seed 9971, before/after: 19.48/19.46,
        // 32.33/32.32, 45.12/45.11, 57.00/57.01, 64.47/64.46, 77.53/77.56,
        // 93.79/93.86, every block 2,000/2,000 decided; 13 seats (500, seed
        // 9972) 162.15/161.86. `--bench` byte-identical.
        // Re-blessed 2026-09-19 (Shifting Woodland, and the card is less
        // interesting than how it was found): the census's reader ended a
        // card's body at the next `pub fn`, so a PRIVATE helper between two
        // factories was read as part of the preceding card — and the card
        // after such a helper had its own body hidden. Ending at the next
        // `fn` of any visibility surfaced one more row, this one.
        // **Nine actions across two of the three games** (4068 → 4061,
        // 2542 → 2540), same winners and same turns.
        // Re-blessed 2026-09-19 (the fastlands, same class): "enters tapped
        // unless you control two or fewer other lands" off the trigger and
        // onto `EntersTappedUnless`, 17 call sites through one helper.
        // **Three actions in one of the three games** (4071 → 4068) and
        // nothing else — same winners, same turns — beside the seed-9102
        // aggregate's largest move of 0.29 turns at eight seats
        // (19.38/19.36, 32.40/32.40, 45.26/45.24, 57.01/57.08, 64.89/64.87,
        // 77.66/77.80, 93.43/93.14), every block 2,000/2,000 decided.
        // `--bench` byte-identical again.
        // Re-blessed 2026-09-19 (CR 614.1c, "enters tapped" is a REPLACEMENT):
        // the shared `etb_tap()` helper and its call sites moved from an
        // `EntersBattlefield` trigger to `StaticEffect::EntersTapped`, so a
        // tapland is tapped as it enters rather than by a trigger resolving
        // after its controller has had priority. Every pod deck runs taplands,
        // so all three games are different games — and the first one is 47→97
        // turns, which is the largest single move this table has taken.
        // ⚠ **Three games is a re-bless gate, not a measurement, and this is
        // the clearest case of it yet.** The aggregate is 2,000 games a seat
        // count at seed 9102, after against before, and it barely moves:
        // 19.36/19.37 turns at two seats, 32.40/32.35 at three, 45.24/45.30 at
        // four, 57.08/56.88 at five, 64.87/64.74 at six, 77.80/77.85 at seven,
        // 93.14/93.34 at eight — largest move 0.20 turns — with every block
        // 2,000/2,000 decided on both sides. `--bench` is byte-identical
        // (195,806 / 27.49 / 611.9 / 0 stalls, both determinism checks ok):
        // `archetypes()` plays basic lands, which have no such replacement.
        // Re-blessed 2026-09-19 (the sinkless sacrifice): the bot no longer
        // volunteers a sacrifice whose ability only makes mana, so a board with
        // a mana-rock-shaped sacrifice outlet stops spending actions it never
        // cashes in. All three games get SHORTER and cheaper — 55→47 turns /
        // 2360→1909 actions, 101→67 / 3969→2831, 57→49 / 2480→2116 — which is
        // the shape the fix predicts; two winners hold and seed 4242's moves
        // 3→2. ⚠ Three games is a re-bless gate, not a measurement: the
        // aggregate is 2,000 games a seat count at seed 9101, after vs before,
        // and it barely moves — 19.33/19.30 turns at two seats (the control),
        // 32.62/32.59 at three, 45.15/45.30 at four, 57.21/57.20 at five,
        // 64.11/64.50 at six, 77.71/77.57 at seven, 92.84/92.89 at eight, every
        // block 2,000/2,000 decided on both sides. The seeded games move far
        // more than the aggregate because the outlet has to be on the board
        // with something to feed it; when it is, the loop was thousands of
        // actions long. `--bench` is byte-identical (the 2-player pool has no
        // such outlet), so no golden trace moves.
        // Re-blessed twice on 2026-09-19, and the second one rides the first.
        // ① **The Judith list's retune** — 28 of its 72 nonbasics changed, so
        // all three games are different games. Its aggregate is 3,000 games a
        // configuration at seed 43: Judith 23.2→33.6 % at two seats,
        // 16.0→26.2 at three and 6.2→14.6 at four, the four-seat field
        // flattening 43.5/6.2/22.2/28.0 → 40.8/14.6/20.3/24.3, 100 % decided
        // and zero stalls on both sides.
        // ② **Two target-deck card defects** — Sowing Mycospawn's tutored land
        // was entering tapped against its oracle (Sigarda) and Delighted
        // Halfling's legendary-only + uncounterable rider was dropped
        // (Sigarda, Tatyova). ⚠ Neither touched `--bench` (195,806 / 27.49 /
        // 611.9 / 0 stalls, byte-identical) or a golden trace, which is the
        // answer to "these are cube cards, so they need a bench re-bless":
        // they are, and it did not move. Against the pre-retune list they
        // moved this table by four actions in one of three games; against the
        // retuned one they move it by **nothing**, so the values below are ①'s
        // unchanged — which is the same reading from the other side.
        // ⚠ Three games is a re-bless gate, not a measurement — read the
        // aggregates above, and read the two-seat row as the control.
        // Re-blessed twice. ① The auto-targeter's ranked "target opponent":
        // the same `default_hostile_opponent` the defender already used now
        // fills an open opponent slot on every cast too, so two of the three
        // seeded games diverged (92→63, 79→69 turns; same winners). The
        // aggregate behind that one is 32,000 pod games a side — **identical**
        // at two seats (the control: a duel has one candidate and the ranking
        // never runs), -0.06/0.00 turns at three, -0.24/-0.17 at four,
        // -0.37/-0.43 at five. ② Two target-deck card defects fixed (Sowing
        // Mycospawn's tutored land was entering tapped against its oracle,
        // Delighted Halfling's legendary-only + uncounterable rider was
        // dropped): **four actions** in one of the three games, same winners
        // and same turns everywhere, and the 4,000-game aggregate reads
        // 39.91 vs 39.90 turns at four seats and 50.15 vs 50.15 at five, deck
        // shares within noise. ⚠ Neither move touched `--bench` (195,806 /
        // 27.49 / 611.9 / 0 stalls, byte-identical) or a golden trace, which
        // is the answer to "these are cube cards, so they need a bench
        // re-bless": they are, and it did not move.
        // Re-blessed 2026-09-19 (the target-clause class): three pod-deck
        // cards stopped fanning a printed "target opponent / target player"
        // clause out over the whole table — Endurance ("up to one target
        // player puts their graveyard on the bottom", Tatyova), Indulgent
        // Tormentor ("unless **target opponent** sacrifices … or pays 3
        // life", Judith) and Nihil Spellbomb ("exile **target player's**
        // graveyard", Judith). At two seats each is the same object it always
        // was; at four it is one seat instead of three, so two of the three
        // seeded games are different games. ⚠ Three games is a re-bless gate,
        // not a measurement — the aggregate is in DECK_FEATURES.
        // Re-blessed 2026-09-19 (board wipes): DECK_FEATURES' standing item was
        // that Sigarda, Hanna and Tatyova had no way to break a stalled board.
        // Five swaps close it — Wrath of God and Fumigate for Sigarda, River's
        // Rebuke for Hanna, Evacuation and Bane of Progress for Tatyova — and
        // the aggregate moved with them: the four-seat spread TIGHTENS from
        // 25.1 to 23.6 points (39.8/14.7/20.0/25.5 → 40.9/17.1/17.3/24.7,
        // 3,000 games at seed 43), and games run ~9 % longer.
        // Re-blessed 2026-09-23 (overkill spill, `server/pod_attack.rs`):
        // face attackers past a kill now go at the next opponent instead of
        // piling onto one seat; seed 43 is the same winner 13 turns sooner.
        // Re-blessed 2026-09-23 (sacrifice costs): Bone Splinters and Village
        // Rites (Judith) sacrifice on cast, not at resolution, so they can't be
        // cast creatureless; seed 0xC0FFEE is the same winner 45 turns sooner.
        // Aggregate within noise (3,000 games, seed 43: Judith 16.9 → 17.1 %,
        // 44.90 → 44.96 turns).
        // Re-blessed 2026-09-23 (Toxic Deluge's X): the bot sizes a pay-X-life
        // X to the toughest opposing creature instead of its spare mana, so
        // Judith's Deluge is a sweeper; seed 4242 is the same winner two
        // turns sooner.
        // Re-blessed 2026-09-23 (CR 800.4a): a player who has left the game is
        // no longer a legal target, so nothing is aimed at a departed seat and
        // a trigger that was is countered. Seeds 43 and 4242: same winners.
        const GOLDEN: [(u64, Option<usize>, u32, usize); 3] = [
            (0xC0FFEE, Some(1), 44, 1829),
            (43, Some(3), 57, 2394),
            (4242, Some(2), 53, 2245),
        ];
        let decks = rofellos_pod(4);
        let t = build_pod_template(&decks);
        let pilots = vec![Pilot::default(); 4];
        let got: Vec<(u64, Option<usize>, u32, usize)> = GOLDEN
            .iter()
            .map(|&(seed, ..)| {
                let o = play_one_pod_game(&t, &pilots, 50_000, seed);
                (seed, o.winner, o.turns, o.actions)
            })
            .collect();
        assert_eq!(got.as_slice(), GOLDEN.as_slice(), "pod outcomes moved");
    }

    /// A four-seat pod plays to a finish without panicking — the whole point
    /// of the module.
    #[test]
    fn a_four_seat_pod_finishes() {
        let t = run_pod(&rofellos_pod(4), 2, 7, 20_000, Pilot::default());
        assert_eq!(t.games, 2);
        assert!(t.total_turns > 0);
    }
}
