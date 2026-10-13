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
mod controlled;
pub(crate) mod fuzz;
pub mod power;
mod power_table;

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

/// Consecutive priority passes with no play between them that call a game an
/// action cap: a mandatory trigger loop that makes progress (a life total
/// growing every pass) never repeats a board, so the loop watch can't name
/// it, and the play-count backstop above let one 8-seat game pass 1.6 M
/// times — 90 minutes on the audit build (Golgari Grave-Troll under Out of
/// the Tombs and Moldervine Reclamation, fuzzed seed 19200324, game 7). A
/// legal trigger chain is far shorter: the board cap stops a stack at
/// `MAX_STACK_ITEMS` long before.
const PASS_STREAK_CAP: usize = 50_000;

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
        // Fortieth: the thirtieth official list (Call the Spirits, C15) —
        // Orzhov enchantments under Daxos the Returned. `--seats 40`.
        PodDeck {
            name: "Daxos the Returned (WB)",
            commanders: decks::DAXOS_COMMANDERS,
            main: decks::DAXOS_MAIN,
        },
        // Forty-first: the thirty-first official list (Rebellion Rising,
        // ONC) — Boros tokens and Equipment under Neyali. `--seats 41`.
        PodDeck {
            name: "Neyali, Suns' Vanguard (RW)",
            commanders: decks::NEYALI_COMMANDERS,
            main: decks::NEYALI_MAIN,
        },
        // Forty-second: the thirty-second official list (Peer Through Time,
        // C14) — mono-blue under a planeswalker commander. `--seats 42`.
        PodDeck {
            name: "Teferi, Temporal Archmage (U)",
            commanders: decks::TEFERI_COMMANDERS,
            main: decks::TEFERI_MAIN,
        },
        // Forty-third: the thirty-third official list (Lorehold Legacies,
        // C21) — Boros artifact recursion under Osgir. `--seats 43`.
        PodDeck {
            name: "Osgir, the Reconstructor (RW)",
            commanders: decks::OSGIR_COMMANDERS,
            main: decks::OSGIR_MAIN,
        },
        // Forty-fourth: the thirty-fourth official list (Counterpunch, CMD) —
        // Abzan Saprolings under Ghave, Guru of Spores. `--seats 44`.
        PodDeck {
            name: "Ghave, Guru of Spores (WBG)",
            commanders: decks::GHAVE_COMMANDERS,
            main: decks::GHAVE_MAIN,
        },
        // Forty-fifth: the thirty-fifth official list (Fae Dominion, WOC) —
        // Dimir Faeries under Tegwyll, Duke of Splendor. `--seats 45`.
        PodDeck {
            name: "Tegwyll, Duke of Splendor (UB)",
            commanders: decks::TEGWYLL_COMMANDERS,
            main: decks::TEGWYLL_MAIN,
        },
        // Forty-sixth: the thirty-sixth official list (Wade into Battle,
        // C15) — Boros Giants under Kalemne. `--seats 46`.
        PodDeck {
            name: "Kalemne, Disciple of Iroas (RW)",
            commanders: decks::KALEMNE_COMMANDERS,
            main: decks::KALEMNE_MAIN,
        },
        // Forty-seventh: the thirty-seventh official list (Arm for Battle, CMR) —
        // Boros Auras and Equipment under Wyleth, Soul of Steel. `--seats 47`.
        PodDeck {
            name: "Wyleth, Soul of Steel (RW)",
            commanders: decks::WYLETH_COMMANDERS,
            main: decks::WYLETH_MAIN,
        },
        // Forty-eighth: the thirty-eighth official list (Invent Superiority,
        // C16) — four-colour artifacts under Breya. `--seats 48`.
        PodDeck {
            name: "Breya, Etherium Shaper (WUBR)",
            commanders: decks::BREYA_COMMANDERS,
            main: decks::BREYA_MAIN,
        },
        // Forty-ninth: the thirty-ninth official list (Chaos Incarnate, SCD)
        // — Rakdos goad and punishment under Kardur. `--seats 49`.
        PodDeck {
            name: "Kardur, Doomscourge (BR)",
            commanders: decks::KARDUR_COMMANDERS,
            main: decks::KARDUR_MAIN,
        },
        // Fiftieth: the fortieth official list (Eternal Might, DRC) —
        // Esper Zombies under Temmet, Naktamun's Will. `--seats 50`.
        PodDeck {
            name: "Temmet, Naktamun's Will (WUB)",
            commanders: decks::TEMMET_COMMANDERS,
            main: decks::TEMMET_MAIN,
        },
        // Fifty-first: the forty-first official list (Entropic Uprising, C16)
        // — UBRG cascade and chaos under Yidris. `--seats 51`.
        PodDeck {
            name: "Yidris, Maelstrom Wielder (UBRG)",
            commanders: decks::YIDRIS_COMMANDERS,
            main: decks::YIDRIS_MAIN,
        },
        // Fifty-second: the forty-second official list (Feline Ferocity,
        // C17) — Selesnya Cats and Equipment under Arahbo. `--seats 52`.
        PodDeck {
            name: "Arahbo, Roar of the World (GW)",
            commanders: decks::ARAHBO_COMMANDERS,
            main: decks::ARAHBO_MAIN,
        },
        // Fifty-third: the forty-third official list (Heavenly Inferno, CMD)
        // — Mardu Angels, Demons and Dragons under Kaalia. `--seats 53`.
        PodDeck {
            name: "Kaalia of the Vast (RWB)",
            commanders: decks::KAALIA_COMMANDERS,
            main: decks::KAALIA_MAIN,
        },
        // Fifty-fourth: the forty-fourth official list (Land's Wrath, ZNC) —
        // Naya landfall under Obuun, Mul Daya Ancestor. `--seats 54`.
        PodDeck {
            name: "Obuun, Mul Daya Ancestor (RGW)",
            commanders: decks::OBUUN_COMMANDERS,
            main: decks::OBUUN_MAIN,
        },
        // Fifty-fifth: the forty-fifth official list (Political Puppets, CMD)
        // — Jeskai donations and politics under Zedruu. `--seats 55`.
        PodDeck {
            name: "Zedruu the Greathearted (URW)",
            commanders: decks::ZEDRUU_COMMANDERS,
            main: decks::ZEDRUU_MAIN,
        },
        // Fifty-sixth: the forty-sixth official list (Primal Genesis, C19)
        // — Naya tokens and populate under Ghired. `--seats 56`.
        PodDeck {
            name: "Ghired, Conclave Exile (RGW)",
            commanders: decks::GHIRED_COMMANDERS,
            main: decks::GHIRED_MAIN,
        },
        // Fifty-seventh: the forty-seventh official list (Stalwart Unity, C16) —
        // four-colour group hug under Kynaios and Tiro. `--seats 57`.
        PodDeck {
            name: "Kynaios and Tiro of Meletis (RGWU)",
            commanders: decks::KYNAIOS_COMMANDERS,
            main: decks::KYNAIOS_MAIN,
        },
        // Fifty-eighth: the forty-eighth official list (Devour for Power, CMD)
        // — Sultai graveyard value under The Mimeoplasm. `--seats 58`.
        PodDeck {
            name: "The Mimeoplasm (BGU)",
            commanders: decks::MIMEOPLASM_COMMANDERS,
            main: decks::MIMEOPLASM_MAIN,
        },
        // Fifty-ninth: the forty-ninth official list (Endless Punishment, DSC)
        // — Rakdos punisher under Valgavoth, Harrower of Souls. `--seats 59`.
        PodDeck {
            name: "Valgavoth, Harrower of Souls (BR)",
            commanders: decks::VALGAVOTH_COMMANDERS,
            main: decks::VALGAVOTH_MAIN,
        },
        // Sixtieth: the fiftieth official list (Token Triumph, SCD) —
        // Selesnya tokens and anthems under Emmara. `--seats 60`.
        PodDeck {
            name: "Emmara, Soul of the Accord (GW)",
            commanders: decks::EMMARA_COMMANDERS,
            main: decks::EMMARA_MAIN,
        },
        // Sixty-first: the fifty-first official list (Breed Lethality, C16)
        // — four-color counters under Atraxa. `--seats 61`.
        PodDeck {
            name: "Atraxa, Praetors' Voice (WUBG)",
            commanders: decks::ATRAXA_COMMANDERS,
            main: decks::ATRAXA_MAIN,
        },
        // Sixty-second: the fifty-second official list (Mirror Mastery, CMD)
        // — Temur copies under Riku of Two Reflections. `--seats 62`.
        PodDeck {
            name: "Riku of Two Reflections (GUR)",
            commanders: decks::RIKU_COMMANDERS,
            main: decks::RIKU_MAIN,
        },
        // Sixty-third: the fifty-third official list (Raining Cats and Dogs,
        // SLD) — Naya Cats and Dogs under Rin and Seri. `--seats 63`.
        PodDeck {
            name: "Rin and Seri, Inseparable (RGW)",
            commanders: decks::RIN_SERI_COMMANDERS,
            main: decks::RIN_SERI_MAIN,
        },
        // Sixty-fourth: the fifty-fourth official list (Open Hostility, C16)
        // — four-color (no blue) aggression under Saskia. `--seats 64`.
        PodDeck {
            name: "Saskia the Unyielding (WBRG)",
            commanders: decks::SASKIA_COMMANDERS,
            main: decks::SASKIA_MAIN,
        },
        // Sixty-fifth: the fifty-fifth official list (First Flight, SCD) —
        // Azorius fliers under Isperia, with Gideon Jura's lure. `--seats 65`.
        PodDeck {
            name: "Isperia, Supreme Judge (WU)",
            commanders: decks::ISPERIA_COMMANDERS,
            main: decks::ISPERIA_MAIN,
        },
        // Sixty-sixth: the fifty-sixth official list (Arcane Wizardry, C17)
        // — Grixis Wizards under Inalla, Archmage Ritualist. `--seats 66`.
        PodDeck {
            name: "Inalla, Archmage Ritualist (UBR)",
            commanders: decks::INALLA_COMMANDERS,
            main: decks::INALLA_MAIN,
        },
        // Sixty-seventh: the fifty-seventh official list (Growing Threat, MOC) —
        // Orzhov Phyrexians and incubate under Brimaz. `--seats 67`.
        PodDeck {
            name: "Brimaz, Blight of Oreskos (WB)",
            commanders: decks::BRIMAZ_COMMANDERS,
            main: decks::BRIMAZ_MAIN,
        },
        // Sixty-eighth: the fifty-eighth official list (Enduring
        // Enchantments, CMM) — Abzan enchantments and Sagas under Anikthea.
        // `--seats 68`.
        PodDeck {
            name: "Anikthea, Hand of Erebos (WBG)",
            commanders: decks::ANIKTHEA_COMMANDERS,
            main: decks::ANIKTHEA_MAIN,
        },
        // Sixty-ninth: the fifty-ninth official list (Urza's Iron Alliance,
        // BRC) — Esper artifacts under Urza, Chief Artificer. `--pod-decks 69`
        // (past `MAX_SEATS`, so not every list fits one pod).
        PodDeck {
            name: "Urza, Chief Artificer (WUB)",
            commanders: decks::URZA_COMMANDERS,
            main: decks::URZA_MAIN,
        },
        // Seventieth: the sixtieth official list (Phantom Premonition, KHC)
        // — Azorius foretell and flicker under Ranar. `--seats 70`.
        PodDeck {
            name: "Ranar the Ever-Watchful (WU)",
            commanders: decks::RANAR_COMMANDERS,
            main: decks::RANAR_MAIN,
        },
        // Seventy-first: the sixty-first official list (Hatsune Miku, SLD) —
        // Selesnya lifegain and tokens under Trostani. `--pod-decks 71`.
        PodDeck {
            name: "Trostani, Selesnya's Voice (GW)",
            commanders: decks::TROSTANI_COMMANDERS,
            main: decks::TROSTANI_MAIN,
        },
        // Seventy-second: the sixty-second official list (Exit from Exile,
        // CLB) — Gruul impulse draw and cascade under Faldorn. `--pod-decks 72`.
        PodDeck {
            name: "Faldorn, Dread Wolf Herald (RG)",
            commanders: decks::FALDORN_COMMANDERS,
            main: decks::FALDORN_MAIN,
        },
        // Seventy-third: the sixty-third official list (Undead Unleashed, MIC)
        // — Dimir Zombies under Wilhelt, the Rotcleaver. `--pod-decks 73`.
        PodDeck {
            name: "Wilhelt, the Rotcleaver (UB)",
            commanders: decks::WILHELT_COMMANDERS,
            main: decks::WILHELT_MAIN,
        },
        // Seventy-fourth: the sixty-fourth official list (Call for Backup, MOC)
        // — Naya +1/+1 counters and backup under Bright-Palm. `--pod-decks 74`.
        PodDeck {
            name: "Bright-Palm, Soul Awakener (RGW)",
            commanders: decks::BRIGHT_PALM_COMMANDERS,
            main: decks::BRIGHT_PALM_MAIN,
        },
        // Seventy-fifth: the sixty-fifth official list (Squirreled Away, BLC)
        // — Golgari Squirrels and Food under Hazel. `--pod-decks 75`.
        PodDeck {
            name: "Hazel of the Rootbloom (BG)",
            commanders: decks::HAZEL_COMMANDERS,
            main: decks::HAZEL_MAIN,
        },
        // Seventy-sixth: the sixty-sixth official list (Exquisite Invention,
        // C18) — Izzet artifacts under Saheeli, the Gifted, a planeswalker
        // commander. `--pod-decks 76`.
        PodDeck {
            name: "Saheeli, the Gifted (UR)",
            commanders: decks::SAHEELI_COMMANDERS,
            main: decks::SAHEELI_MAIN,
        },
        // Seventy-seventh: the sixty-seventh official list (Death Toll, DSC)
        // — Golgari self-mill and delirium under Winter. `--pod-decks 77`.
        PodDeck {
            name: "Winter, Cynical Opportunist (BG)",
            commanders: decks::WINTER_COMMANDERS,
            main: decks::WINTER_MAIN,
        },
        // Seventy-eighth: the sixty-eighth official list (Witherbloom
        // Witchcraft, C21) — Golgari life gain and loss under Willowdusk.
        // `--pod-decks 78`.
        PodDeck {
            name: "Willowdusk, Essence Seer (BG)",
            commanders: decks::WILLOWDUSK_COMMANDERS,
            main: decks::WILLOWDUSK_MAIN,
        },
        // Seventy-ninth: the sixty-ninth official list (World Shaper, EOC) —
        // Jund land sacrifice under Hearthhull. `--pod-decks 79`.
        PodDeck {
            name: "Hearthhull, the Worldseed (BRG)",
            commanders: decks::HEARTHHULL_COMMANDERS,
            main: decks::HEARTHHULL_MAIN,
        },
        // Eightieth: the seventieth official list (Nature's Vengeance,
        // C18) — Jund lands under the planeswalker Lord Windgrace.
        // `--pod-decks 80`.
        PodDeck {
            name: "Lord Windgrace (BRG)",
            commanders: decks::WINDGRACE_COMMANDERS,
            main: decks::WINDGRACE_MAIN,
        },
        // Eighty-first: the seventy-first official list (Evasive Maneuvers,
        // C13) — Bant tempo and curses under Derevi. `--pod-decks 81`.
        PodDeck {
            name: "Derevi, Empyrial Tactician (GWU)",
            commanders: decks::DEREVI_COMMANDERS,
            main: decks::DEREVI_MAIN,
        },
        // Eighty-second: the seventy-second official list (Peace Offering, BLC)
        // — Bant group hug under Ms. Bumbleflower. `--pod-decks 82`.
        PodDeck {
            name: "Ms. Bumbleflower (GWU)",
            commanders: decks::BUMBLEFLOWER_COMMANDERS,
            main: decks::BUMBLEFLOWER_MAIN,
        },
        // Eighty-third: the seventy-third official list (Mishra's Burnished
        // Banner, BRC) — Grixis artifacts under Mishra, Eminent One.
        // `--pod-decks 83`.
        PodDeck {
            name: "Mishra, Eminent One (UBR)",
            commanders: decks::MISHRA_COMMANDERS,
            main: decks::MISHRA_MAIN,
        },
        // Eighty-fourth: the seventy-fourth official list (Tinker Time, MOC)
        // — Temur artifact tokens under Gimbal, Gremlin Prodigy.
        // `--pod-decks 84`.
        PodDeck {
            name: "Gimbal, Gremlin Prodigy (GUR)",
            commanders: decks::GIMBAL_COMMANDERS,
            main: decks::GIMBAL_MAIN,
        },
        // Eighty-fifth: the seventy-fifth official list (Legends' Legacy,
        // DMC) — Mardu legends under Dihada, Binder of Wills, a planeswalker
        // commander. `--pod-decks 85`.
        PodDeck {
            name: "Dihada, Binder of Wills (RWB)",
            commanders: decks::DIHADA_COMMANDERS,
            main: decks::DIHADA_MAIN,
        },
        // Eighty-sixth: the seventy-sixth official list (Witherbloom
        // Pestilence, SOC) — Golgari Pests and sacrifice under Dina.
        // `--pod-decks 86`.
        PodDeck {
            name: "Dina, Essence Brewer (BG)",
            commanders: decks::DINA_COMMANDERS,
            main: decks::DINA_MAIN,
        },
        // Eighty-seventh: the seventy-seventh official list (Draconic
        // Destruction, SCD) — Gruul Dragons under Atarka. `--pod-decks 87`.
        PodDeck {
            name: "Atarka, World Render (RG)",
            commanders: decks::ATARKA_COMMANDERS,
            main: decks::ATARKA_MAIN,
        },
        // Eighty-eighth: the seventy-eighth official list (Nature of the Beast,
        // C13) — Naya big creatures under Marath. `--pod-decks 88`.
        PodDeck {
            name: "Marath, Will of the Wild (RGW)",
            commanders: decks::MARATH_COMMANDERS,
            main: decks::MARATH_MAIN,
        },
        // Eighty-ninth: the seventy-ninth official list (Mind Seize, C13)
        // — Grixis spells and theft under Jeleva. `--pod-decks 89`.
        PodDeck {
            name: "Jeleva, Nephalia's Scourge (UBR)",
            commanders: decks::JELEVA_COMMANDERS,
            main: decks::JELEVA_MAIN,
        },
        // Ninetieth: the eightieth official list (Silverquill Statement,
        // C21) — Orzhov politics under Breena, the Demagogue.
        // `--pod-decks 90`.
        PodDeck {
            name: "Breena, the Demagogue (WB)",
            commanders: decks::BREENA_COMMANDERS,
            main: decks::BREENA_MAIN,
        },
        // Ninety-first: the eighty-first official list (Planeswalker Party,
        // CMM) — Jeskai superfriends under Commodore Guff, a planeswalker
        // commander. `--pod-decks 91`.
        PodDeck {
            name: "Commodore Guff (URW)",
            commanders: decks::GUFF_COMMANDERS,
            main: decks::GUFF_MAIN,
        },
        // Ninety-second: the eighty-second official list (20 Ways to Win, SLD)
        // — alternate wins, Gates and Shrines under Go-Shintai. `--pod-decks
        // 92`.
        PodDeck {
            name: "Go-Shintai of Life's Origin (WUBRG)",
            commanders: decks::GOSHINTAI_COMMANDERS,
            main: decks::GOSHINTAI_MAIN,
        },
        // Ninety-third: the eighty-third official list (Blame Game, MKC) —
        // Boros goad under Nelly Borca. `--pod-decks 93`.
        PodDeck {
            name: "Nelly Borca, Impulsive Accuser (RW)",
            commanders: decks::NELLY_COMMANDERS,
            main: decks::NELLY_MAIN,
        },
        // Ninety-fourth: the eighty-fourth official list (Family Matters,
        // BLC) — Jeskai tokens and offspring under Zinnia, Valley's Voice.
        // `--pod-decks 94`.
        PodDeck {
            name: "Zinnia, Valley's Voice (URW)",
            commanders: decks::ZINNIA_COMMANDERS,
            main: decks::ZINNIA_MAIN,
        },
        // Ninety-fifth: the eighty-fifth official list (Reign of Dragons,
        // FDC) — mono-red Dragons under Lathliss. `--pod-decks 95`.
        PodDeck {
            name: "Lathliss, Dragon Queen (R)",
            commanders: decks::LATHLISS_COMMANDERS,
            main: decks::LATHLISS_MAIN,
        },
        // Ninety-sixth: the eighty-sixth official list (Virtue and Valor,
        // WOC) — Selesnya Auras and Roles under Ellivere of the Wild Court.
        // `--pod-decks 96`.
        PodDeck {
            name: "Ellivere of the Wild Court (GW)",
            commanders: decks::ELLIVERE_COMMANDERS,
            main: decks::ELLIVERE_MAIN,
        },
        // Ninety-seventh: the eighty-seventh official list (Temur Roar, TDC) —
        // Temur Dragons under Eshki, Temur's Roar. `--pod-decks 97`.
        PodDeck {
            name: "Eshki, Temur's Roar (GUR)",
            commanders: decks::ESHKI_COMMANDERS,
            main: decks::ESHKI_MAIN,
        },
        // Ninety-eighth: the eighty-eighth official list (Cabaretti Cacophony, NCC).
        // `--pod-decks 98`.
        PodDeck {
            name: "Kitt Kanto, Mayhem Diva (RGW)",
            commanders: decks::KITT_COMMANDERS,
            main: decks::KITT_MAIN,
        },
        // Ninety-ninth: the eighty-ninth official list (Draconic
        // Domination, C17) — five-color Dragons under The Ur-Dragon.
        // `--pod-decks 99`.
        PodDeck {
            name: "The Ur-Dragon (WUBRG)",
            commanders: decks::UR_DRAGON_COMMANDERS,
            main: decks::UR_DRAGON_MAIN,
        },
        // Hundredth: the ninetieth official list (Eldrazi Incursion, M3C) —
        // colorless Eldrazi under Ulalek, Fused Atrocity. `--pod-decks 100`.
        PodDeck {
            name: "Ulalek, Fused Atrocity (WUBRG)",
            commanders: decks::ULALEK_COMMANDERS,
            main: decks::ULALEK_MAIN,
        },
        // Hundred-and-first: the ninety-first official list (Divine
        // Convocation, MOC) — Jeskai convoke under Kasla, the Broken Halo.
        // `--pod-decks 101`.
        PodDeck {
            name: "Kasla, the Broken Halo (URW)",
            commanders: decks::KASLA_COMMANDERS,
            main: decks::KASLA_MAIN,
        },
        // Hundred-and-second: the ninety-second official list (Adaptive
        // Enchantment, C18) — Bant enchantments under Estrid, the Masked, a
        // planeswalker commander. `--pod-decks 102`.
        PodDeck {
            name: "Estrid, the Masked (GWU)",
            commanders: decks::ESTRID_COMMANDERS,
            main: decks::ESTRID_MAIN,
        },
        // Hundred-and-third: the ninety-third official list (Symbiotic Swarm,
        // C20) — Abzan keyword counters under Kathril, Aspect Warper.
        // `--pod-decks 103`.
        PodDeck {
            name: "Kathril, Aspect Warper (WBG)",
            commanders: decks::KATHRIL_COMMANDERS,
            main: decks::KATHRIL_MAIN,
        },
        // Hundred-and-fourth: the ninety-fourth official list (Draconic Rage,
        // AFC) — Gruul dragons and dice under Vrondiss, Rage of Ancients.
        // `--pod-decks 104`.
        PodDeck {
            name: "Vrondiss, Rage of Ancients (RG)",
            commanders: decks::DRACONIC_RAGE_COMMANDERS,
            main: decks::DRACONIC_RAGE_MAIN,
        },
        // Hundred-and-fifth: the ninety-fifth official list (Silverquill
        // Influence, SOC) — Orzhov Auras and goad under Killian, Decisive
        // Mentor. `--pod-decks 105`.
        PodDeck {
            name: "Killian, Decisive Mentor (WB)",
            commanders: decks::KILLIAN_COMMANDERS,
            main: decks::KILLIAN_MAIN,
        },
        // Hundred-and-sixth: the ninety-sixth official list (Aura of Courage,
        // AFC) — Bant Auras and Equipment under Galea, Kindler of Hope.
        // `--pod-decks 106`.
        PodDeck {
            name: "Galea, Kindler of Hope (GWU)",
            commanders: decks::GALEA_COMMANDERS,
            main: decks::GALEA_MAIN,
        },
        // Hundred-and-seventh: the ninety-seventh official list (Draconic
        // Dissent, CLB) — Izzet goad and Dragons under Firkraag, Cunning
        // Instigator. `--pod-decks 107`.
        PodDeck {
            name: "Firkraag, Cunning Instigator (UR)",
            commanders: decks::FIRKRAAG_COMMANDERS,
            main: decks::FIRKRAAG_MAIN,
        },
        // Hundred-and-eighth: the ninety-eighth official list (Eldrazi
        // Unbound, CMM) — colorless Eldrazi ramp under Zhulodok, Void Gorger.
        // `--pod-decks 108`.
        PodDeck {
            name: "Zhulodok, Void Gorger (C)",
            commanders: decks::ZHULODOK_COMMANDERS,
            main: decks::ZHULODOK_MAIN,
        },
        // Hundred-and-ninth: the ninety-ninth official list (Upgrades
        // Unleashed, NEC) — Gruul "modified" creatures under Chishiro, the
        // Shattered Blade. `--pod-decks 109`.
        PodDeck {
            name: "Chishiro, the Shattered Blade (RG)",
            commanders: decks::CHISHIRO_COMMANDERS,
            main: decks::CHISHIRO_MAIN,
        },
        // Hundred-and-tenth: the hundredth official list (Painbow, DMC) —
        // five-color multicolor under Jared Carthalion, a planeswalker
        // commander (CR 903.3a). `--pod-decks 110`.
        PodDeck {
            name: "Jared Carthalion (WUBRG)",
            commanders: decks::JARED_COMMANDERS,
            main: decks::JARED_MAIN,
        },
        // Hundred-and-eleventh: the hundred-and-first official list (Ruthless
        // Regiment, C20) — Mardu Humans under Jirina Kudro. `--pod-decks 111`.
        PodDeck {
            name: "Jirina Kudro (RWB)",
            commanders: decks::JIRINA_COMMANDERS,
            main: decks::JIRINA_MAIN,
        },
        // Hundred-and-twelfth: the hundred-and-second official list (Prismari
        // Performance, C21) — Izzet big-spell magecraft under Zaffai, Thunder
        // Conductor. `--pod-decks 112`.
        PodDeck {
            name: "Zaffai, Thunder Conductor (UR)",
            commanders: decks::ZAFFAI_COMMANDERS,
            main: decks::ZAFFAI_MAIN,
        },
        // Hundred-and-thirteenth: the hundred-and-third official list (Mardu
        // Surge, TDC) — Mardu tokens and attack triggers under Zurgo
        // Stormrender. `--pod-decks 113`.
        PodDeck {
            name: "Zurgo Stormrender (RWB)",
            commanders: decks::ZURGO_COMMANDERS,
            main: decks::ZURGO_MAIN,
        },
        // Hundred-and-fourteenth: the hundred-and-fourth official list (Subjective
        // Reality, C18) — Esper top-of-library under Aminatou, the Fateshifter,
        // a planeswalker commander. `--pod-decks 114`.
        PodDeck {
            name: "Aminatou, the Fateshifter (WUB)",
            commanders: decks::AMINATOU_COMMANDERS,
            main: decks::AMINATOU_MAIN,
        },
        // Hundred-and-fifteenth: the hundred-and-fifth official list (Prismari Artistry, SOC) — Izzet spells and Elementals under Rootha.
        // `--pod-decks 115`.
        PodDeck {
            name: "Rootha, Mastering the Moment (UR)",
            commanders: decks::ROOTHA_COMMANDERS,
            main: decks::ROOTHA_MAIN,
        },
        // Hundred-and-sixteenth: the hundred-and-sixth official list (Most
        // Wanted, OTC) — Mardu outlaws and Treasure under Olivia, Opulent
        // Outlaw. `--pod-decks 116`.
        PodDeck {
            name: "Olivia, Opulent Outlaw (RWB)",
            commanders: decks::OLIVIA_COMMANDERS,
            main: decks::OLIVIA_MAIN,
        },
        // Hundred-and-seventeenth: the hundred-and-seventh official list
        // (Arcane Maelstrom, C20) — Temur instants and copies under Kalamax,
        // the Stormsire. `--pod-decks 117`.
        PodDeck {
            name: "Kalamax, the Stormsire (GUR)",
            commanders: decks::KALAMAX_COMMANDERS,
            main: decks::KALAMAX_MAIN,
        },
        // Hundred-and-eighteenth: the hundred-and-eighth official list (Miracle
        // Worker, DSC) — Esper enchantments and miracles under Aminatou, Veil
        // Piercer. `--pod-decks 118`.
        PodDeck {
            name: "Aminatou, Veil Piercer (WUB)",
            commanders: decks::VEIL_PIERCER_COMMANDERS,
            main: decks::VEIL_PIERCER_MAIN,
        },
        // Hundred-and-nineteenth: the hundred-and-ninth official list
        // (Enhanced Evolution, C20) — Sultai mutate under Otrimi, the
        // Ever-Playful. `--pod-decks 119`.
        PodDeck {
            name: "Otrimi, the Ever-Playful (BGU)",
            commanders: decks::OTRIMI_COMMANDERS,
            main: decks::OTRIMI_MAIN,
        },
        // Hundred-and-twentieth: the hundred-and-tenth official list (Coven
        // Counters, MIC) — Selesnya counters and coven under Leinore.
        // `--pod-decks 120`.
        PodDeck {
            name: "Leinore, Autumn Sovereign (GW)",
            commanders: decks::LEINORE_COMMANDERS,
            main: decks::LEINORE_MAIN,
        },
        // Hundred-and-twenty-first: the hundred-and-eleventh official list
        // (Cavalry Charge, MOC) — Esper Knights under Sidar Jabari of Zhalfir,
        // whose eminence loots from the command zone. `--pod-decks 121`.
        PodDeck {
            name: "Sidar Jabari of Zhalfir (WUB)",
            commanders: decks::SIDAR_COMMANDERS,
            main: decks::SIDAR_MAIN,
        },
        // Hundred-and-twenty-second: the hundred-and-twelfth official list
        // (Tricky Terrain, M3C) — Simic lands-matter under Omo, Queen of
        // Vesuva. `--pod-decks 122`.
        PodDeck {
            name: "Omo, Queen of Vesuva (GU)",
            commanders: decks::OMO_COMMANDERS,
            main: decks::OMO_MAIN,
        },
        // Hundred-and-twenty-third: the hundred-and-thirteenth official list
        // (Desert Bloom, OTC) — Naya Deserts and lands-matter under Yuma,
        // Proud Protector. `--pod-decks 123`.
        PodDeck {
            name: "Yuma, Proud Protector (RGW)",
            commanders: decks::YUMA_COMMANDERS,
            main: decks::YUMA_MAIN,
        },
        // Hundred-and-twenty-fourth: the hundred-and-fourteenth official list
        // (Spirit Squadron, VOC) — Azorius Spirits under Millicent.
        // `--pod-decks 124`.
        PodDeck {
            name: "Millicent, Restless Revenant (WU)",
            commanders: decks::MILLICENT_COMMANDERS,
            main: decks::MILLICENT_MAIN,
        },
        // Hundred-and-twenty-fifth: the hundred-and-fifteenth official list
        // (Riveteers Rampage, NCC) — Jund blitz under Henzie "Toolbox" Torre.
        // `--pod-decks 125`.
        PodDeck {
            name: "Henzie \"Toolbox\" Torre (BRG)",
            commanders: decks::HENZIE_COMMANDERS,
            main: decks::HENZIE_MAIN,
        },
        // Hundred-and-twenty-sixth: the hundred-and-sixteenth official list
        // (Eternal Bargain, C13) — Esper lifegain under Oloro, Ageless Ascetic
        // (CR 113.6b command-zone upkeep life). `--pod-decks 126`.
        PodDeck {
            name: "Oloro, Ageless Ascetic (WUB)",
            commanders: decks::OLORO_COMMANDERS,
            main: decks::OLORO_MAIN,
        },
        // Hundred-and-twenty-seventh: the hundred-and-seventeenth official
        // list (Lorehold Spirit, SOC) — Boros Spirits under the planeswalker
        // commander Quintorius, History Chaser. `--pod-decks 127`.
        PodDeck {
            name: "Quintorius, History Chaser (RW)",
            commanders: decks::QUINTORIUS_COMMANDERS,
            main: decks::QUINTORIUS_MAIN,
        },
        // Hundred-and-twenty-eighth: the hundred-and-eighteenth official list
        // (Timeless Wisdom, C20) — Jeskai cycling under Gavi, Nest Warden.
        // `--pod-decks 128`.
        PodDeck {
            name: "Gavi, Nest Warden (URW)",
            commanders: decks::GAVI_COMMANDERS,
            main: decks::GAVI_MAIN,
        },
        // Hundred-and-twenty-ninth: the hundred-and-nineteenth official list
        // (Grand Larceny, OTC) — Sultai theft under Gonti, Canny Acquisitor.
        // `--pod-decks 129`.
        PodDeck {
            name: "Gonti, Canny Acquisitor (BGU)",
            commanders: decks::GONTI_COMMANDERS,
            main: decks::GONTI_MAIN,
        },
        // Hundred-and-thirtieth: the hundred-and-twentieth official list
        // (Power Hungry, C13) — Jund tokens and sacrifice under Prossh,
        // Skyraider of Kher. `--pod-decks 130`.
        PodDeck {
            name: "Prossh, Skyraider of Kher (BRG)",
            commanders: decks::PROSSH_COMMANDERS,
            main: decks::PROSSH_MAIN,
        },
        // Hundred-and-thirty-first: the hundred-and-twenty-first official
        // list (Everyone's Invited!, SLD) — five-color changelings under
        // Morophon, the Boundless. `--pod-decks 131`.
        PodDeck {
            name: "Morophon, the Boundless (WUBRG)",
            commanders: decks::MOROPHON_COMMANDERS,
            main: decks::MOROPHON_MAIN,
        },
        // Hundred-and-thirty-second: the hundred-and-twenty-second official
        // list (Dungeons of Death, AFC) — Esper reanimation and venturing
        // under Sefris of the Hidden Ways. `--pod-decks 132`.
        PodDeck {
            name: "Sefris of the Hidden Ways (WUB)",
            commanders: decks::SEFRIS_COMMANDERS,
            main: decks::SEFRIS_MAIN,
        },
        // Hundred-and-thirty-third: the hundred-and-twenty-third official list
        // (Jump Scare!, DSC) — Simic manifest dread under Zimone.
        // `--pod-decks 133`.
        PodDeck {
            name: "Zimone, Mystery Unraveler (GU)",
            commanders: decks::ZIMONE_COMMANDERS,
            main: decks::ZIMONE_MAIN,
        },
        // Hundred-and-thirty-fourth: the hundred-and-twenty-fourth official
        // list (Deep Clue Sea, MKC) — Bant Clues and second-draw payoffs under
        // Morska, Undersea Sleuth. `--pod-decks 134`.
        PodDeck {
            name: "Morska, Undersea Sleuth (GWU)",
            commanders: decks::MORSKA_COMMANDERS,
            main: decks::MORSKA_MAIN,
        },
        // Hundred-and-thirty-fifth: Deadly Disguise (Murders at Karlov
        // Manor) — Naya morph, disguise and cloak under Kaust, Eyes of
        // the Glade. `--pod-decks 135`.
        PodDeck {
            name: "Kaust, Eyes of the Glade (RGW)",
            commanders: decks::KAUST_COMMANDERS,
            main: decks::KAUST_MAIN,
        },
        // Hundred-and-thirty-sixth: the hundred-and-twenty-sixth official list
        // (Party Time, CLB) — Orzhov parties under Nalia de'Arnise.
        // `--pod-decks 136`.
        PodDeck {
            name: "Nalia de'Arnise (WB)",
            commanders: decks::NALIA_COMMANDERS,
            main: decks::NALIA_MAIN,
        },
        // Hundred-and-thirty-seventh: Quandrix Unlimited (Secrets of
        // Strixhaven) — Simic X spells and +1/+1 counters under Zimone,
        // Infinite Analyst. `--pod-decks 137`.
        PodDeck {
            name: "Zimone, Infinite Analyst (GU)",
            commanders: decks::ZIMONE_IA_COMMANDERS,
            main: decks::ZIMONE_IA_MAIN,
        },
        // Seat 138: Living Energy (DRC) — Temur energy and artifacts under
        // Saheeli, Radiant Creator (CR 107.16 energy; CR 118.9 energy as an
        // alternative cost). `--pod-decks 138`.
        PodDeck {
            name: "Saheeli, Radiant Creator (GUR)",
            commanders: decks::SAHEELI_RADIANT_COMMANDERS,
            main: decks::SAHEELI_RADIANT_MAIN,
        },
        // Seat 139: Counter Intelligence (EOC) — Jeskai artifacts and counters
        // under the Spacecraft commander Inspirit, Flagship Vessel (CR 903.3's
        // Spacecraft clause). `--pod-decks 139`.
        PodDeck {
            name: "Inspirit, Flagship Vessel (URW)",
            commanders: decks::INSPIRIT_COMMANDERS,
            main: decks::INSPIRIT_MAIN,
        },
        // Hundred-and-fortieth: Maestros Massacre (Streets of New
        // Capenna) — Grixis spell-copying and casualty under Anhelo,
        // the Painter. `--pod-decks 140`.
        PodDeck {
            name: "Anhelo, the Painter (UBR)",
            commanders: decks::ANHELO_COMMANDERS,
            main: decks::ANHELO_MAIN,
        },
        // Hundred-and-forty-first: the hundred-and-thirty-first official
        // list (Planar Portal, AFC) — Rakdos exile-and-play under Prosper,
        // Tome-Bound. `--pod-decks 141`.
        PodDeck {
            name: "Prosper, Tome-Bound (BR)",
            commanders: decks::PROSPER_COMMANDERS,
            main: decks::PROSPER_MAIN,
        },
        // Hundred-and-forty-second: Faceless Menace (C19) — Sultai morph
        // under Kadena, Slinking Sorcerer. `--pod-decks 142`.
        PodDeck {
            name: "Kadena, Slinking Sorcerer (BGU)",
            commanders: decks::KADENA_COMMANDERS,
            main: decks::KADENA_MAIN,
        },
        // Hundred-and-forty-third: the hundred-and-thirty-third official
        // list (Obscura Operation, NCC) — Esper connive under Kamiz.
        // `--pod-decks 143`.
        PodDeck {
            name: "Kamiz, Obscura Oculus (WUB)",
            commanders: decks::KAMIZ_COMMANDERS,
            main: decks::KAMIZ_MAIN,
        },
        // Hundred-and-forty-fourth: Creative Energy (Modern Horizons 3)
        // — Jeskai energy and token copies under Satya, Aetherflux
        // Genius. `--pod-decks 144`.
        PodDeck {
            name: "Satya, Aetherflux Genius (URW)",
            commanders: decks::SATYA_COMMANDERS,
            main: decks::SATYA_MAIN,
        },
        // Hundred-and-forty-fifth: the hundred-and-thirty-fifth official
        // list (Explorers of the Deep, LCC) — Simic Merfolk and explore under
        // Hakbal of the Surging Soul. `--pod-decks 145`.
        PodDeck {
            name: "Hakbal of the Surging Soul (GU)",
            commanders: decks::HAKBAL_COMMANDERS,
            main: decks::HAKBAL_MAIN,
        },
        // Seat 146: Bedecked Brokers (NCC) — Bant counters and shields under
        // Perrie, the Pulverizer (CR 122.1b keyword counters as kinds; CR
        // 122.1c shields). `--pod-decks 146`.
        PodDeck {
            name: "Perrie, the Pulverizer (GWU)",
            commanders: decks::PERRIE_COMMANDERS,
            main: decks::PERRIE_MAIN,
        },
        // Hundred-and-forty-seventh: Riders of Rohan (The Lord of the
        // Rings: Tales of Middle-earth) — Jeskai Humans and the monarch
        // under Éowyn, Shieldmaiden. `--pod-decks 147`.
        PodDeck {
            name: "Éowyn, Shieldmaiden (URW)",
            commanders: decks::EOWYN_COMMANDERS,
            main: decks::EOWYN_MAIN,
        },
        // Hundred-and-forty-eighth: Dance of the Elements (Lorwyn Eclipsed)
        // — five-color Elementals under Ashling, the Limitless, evoking from
        // hand and copying what they sacrifice. `--pod-decks 148`.
        PodDeck {
            name: "Ashling, the Limitless (WUBRG)",
            commanders: decks::ASHLING_COMMANDERS,
            main: decks::ASHLING_MAIN,
        },
        // Hundred-and-forty-ninth: Abzan Armor (Tarkir: Dragonstorm) —
        // Abzan toughness-matters and defenders under Felothar the
        // Steadfast. `--pod-decks 149`.
        PodDeck {
            name: "Felothar the Steadfast (WBG)",
            commanders: decks::FELOTHAR_COMMANDERS,
            main: decks::FELOTHAR_MAIN,
        },
        // Hundred-and-fiftieth: the hundred-and-fortieth official
        // list (Elven Council, LTC) — Simic Elves and votes under Galadriel.
        // `--pod-decks 150`.
        PodDeck {
            name: "Galadriel, Elven-Queen (GU)",
            commanders: decks::GALADRIEL_COMMANDERS,
            main: decks::GALADRIEL_MAIN,
        },
        // Hundred-and-fifty-first: Buckle Up (Kamigawa: Neon Dynasty) —
        // Azorius Vehicles and artifacts under Kotori, Pilot Prodigy.
        // `--pod-decks 151`.
        PodDeck {
            name: "Kotori, Pilot Prodigy (WU)",
            commanders: decks::KOTORI_COMMANDERS,
            main: decks::KOTORI_MAIN,
        },
        // Seat 152: Merciless Rage (C19) — Rakdos madness under Anje
        // Falkenrath (CR 702.35 madness, madness {X}; Aeon Engine's reversed
        // turn order). `--pod-decks 152`.
        PodDeck {
            name: "Anje Falkenrath (BR)",
            commanders: decks::ANJE_COMMANDERS,
            main: decks::ANJE_MAIN,
        },
        // Hundred-and-fifty-third: Mystic Intellect (Commander 2019) — Jeskai
        // flashback under Sevinne, the Chronoclasm (CR 724.2 Mandate of
        // Peace, CR 508.1a Pramikon). `--pod-decks 153`.
        PodDeck {
            name: "Sevinne, the Chronoclasm (URW)",
            commanders: decks::SEVINNE_COMMANDERS,
            main: decks::SEVINNE_MAIN,
        },
        // Hundred-and-fifty-fourth: The Hosts of Mordor (LTC) — Grixis
        // Orc armies, the Ring tempting you, and the opponents' own
        // creatures and spells turned against them. `--pod-decks 154`.
        PodDeck {
            name: "Sauron, Lord of the Rings (UBR)",
            commanders: decks::SAURON_COMMANDERS,
            main: decks::SAURON_MAIN,
        },
        // Hundred-and-fifty-fifth: Veloci-Ramp-Tor (Lost Caverns of Ixalan)
        // — Naya Dinosaurs and enrage under Pantlaza, Sun-Favored.
        // `--pod-decks 155`.
        PodDeck {
            name: "Pantlaza, Sun-Favored (RGW)",
            commanders: decks::PANTLAZA_COMMANDERS,
            main: decks::PANTLAZA_MAIN,
        },
        // Hundred-and-fifty-sixth: Elven Empire (Kaldheim Commander) —
        // Golgari Elves under Lathril, Blade of the Elves. `--pod-decks 156`.
        PodDeck {
            name: "Lathril, Blade of the Elves (BG)",
            commanders: decks::LATHRIL_COMMANDERS,
            main: decks::LATHRIL_MAIN,
        },
        // Seat 157: Limit Break (FIC, Final Fantasy VII) — Naya Equipment
        // under Cloud, Ex-SOLDIER (CR 702.6 equip, CR 702.143 foretell riders
        // read "cast from exile"). `--pod-decks 157`.
        PodDeck {
            name: "Cloud, Ex-SOLDIER (RGW)",
            commanders: decks::CLOUD_FIC_COMMANDERS,
            main: decks::CLOUD_FIC_MAIN,
        },
        // Hundred-and-fifty-eighth: the hundred-and-forty-eighth official
        // list (Ahoy Mateys, LCC) — Grixis Pirates under Admiral Brass.
        // `--pod-decks 158`.
        PodDeck {
            name: "Admiral Brass, Unsinkable (UBR)",
            commanders: decks::BRASS_COMMANDERS,
            main: decks::BRASS_MAIN,
        },
        // Hundred-and-fifty-ninth: Revenant Recon (Murders at Karlov Manor)
        // — Dimir surveil, reanimation and copies under Mirko, Obsessive
        // Theorist (CR 701.42 surveil events, CR 500.8 Sphinx of the Second
        // Sun). `--pod-decks 159`.
        PodDeck {
            name: "Mirko, Obsessive Theorist (UB)",
            commanders: decks::MIRKO_COMMANDERS,
            main: decks::MIRKO_MAIN,
        },
        // Seat 160: Revival Trance (FIC, Final Fantasy VI) — Mardu graveyard
        // recursion under Terra, Herald of Hope (random modes, opponents'
        // reanimation picks, blitz from the graveyard). `--pod-decks 160`.
        PodDeck {
            name: "Terra, Herald of Hope (RWB)",
            commanders: decks::TERRA_COMMANDERS,
            main: decks::TERRA_MAIN,
        },
        // Hundred-and-sixty-first: Multiverse Reforged (Final Reforging) —
        // four-color legends under Jace, Multiverse Architect, a
        // planeswalker commander with Empower Jace. `--pod-decks 161`.
        PodDeck {
            name: "Jace, Multiverse Architect (WUBR)",
            commanders: decks::JACE_COMMANDERS,
            main: decks::JACE_MAIN,
        },
        // Hundred-and-sixty-second: Food and Fellowship (Tales of
        // Middle-earth) — Abzan Food and lifegain under Frodo and Sam, a
        // "partner with" pair (CR 702.124j). `--pod-decks 162`.
        PodDeck {
            name: "Frodo + Sam (WBG)",
            commanders: decks::FRODO_COMMANDERS,
            main: decks::FRODO_MAIN,
        },
        // Hundred-and-sixty-third: the hundred-and-fifty-third official
        // list (Scions & Spellcraft, FIC) — Esper spells under Y'shtola.
        // `--pod-decks 163`.
        PodDeck {
            name: "Y'shtola, Night's Blessed (WUB)",
            commanders: decks::YSHTOLA_COMMANDERS,
            main: decks::YSHTOLA_MAIN,
        },
        // Hundred-and-sixty-fourth: Blight Curse (Lorwyn Eclipsed) — Jund
        // -1/-1 counters under Auntie Ool, Cursewretch (CR 701.68 blight as a
        // cost, CR 701.30 Hoarder's Greed's repeating clash). `--pod-decks 164`.
        PodDeck {
            name: "Auntie Ool, Cursewretch (BRG)",
            commanders: decks::AUNTIE_COMMANDERS,
            main: decks::AUNTIE_MAIN,
        },
        // Hundred-and-sixty-fifth: Counter Blitz (FIC, Final Fantasy X) —
        // Bant +1/+1 counters and proliferate under Tidus, Yuna's Guardian
        // (CR 122.6 "you put" counters, Saga creatures). `--pod-decks 165`.
        PodDeck {
            name: "Tidus, Yuna's Guardian (GWU)",
            commanders: decks::TIDUS_COMMANDERS,
            main: decks::TIDUS_MAIN,
        },
        // Hundred-and-sixty-sixth: Mutant Menace (Fallout) — Sultai rad
        // counters, mills and Mutants under The Wise Mothman (CR 728 healing
        // radiation, CR 615 Bloatfly Swarm). `--pod-decks 166`.
        PodDeck {
            name: "The Wise Mothman (BGU)",
            commanders: decks::MOTHMAN_COMMANDERS,
            main: decks::MOTHMAN_MAIN,
        },
        // Seat 167: Wakanda Forever (MSC) — Selesnya artifacts, Vibranium
        // and the monarch under T'Challa, the Black Panther (CR 508.1c gated
        // attack restrictions, CR 615.10 spell sources). `--pod-decks 167`.
        PodDeck {
            name: "T'Challa, the Black Panther (GW)",
            commanders: decks::TCHALLA_COMMANDERS,
            main: decks::TCHALLA_MAIN,
        },
        PodDeck {
            name: "Esika, God of the Tree (WUBRG)",
            commanders: decks::ESIKA_COMMANDERS,
            main: decks::ESIKA_MAIN,
        },
        // Hundred-and-sixty-ninth: Science! (Fallout) — Jeskai energy and
        // artifacts under Dr. Madison Li (CR 707.2 Curie's cost-exiled copy
        // source, Rex's brain-countered abilities). `--pod-decks 169`.
        PodDeck {
            name: "Dr. Madison Li (URW)",
            commanders: decks::MADISON_COMMANDERS,
            main: decks::MADISON_MAIN,
        },
        // Hundred-and-seventieth: Hail, Caesar (Fallout) — Mardu tokens
        // and sacrifice under Caesar, Legion's Emperor. `--pod-decks 170`.
        PodDeck {
            name: "Caesar, Legion's Emperor (RWB)",
            commanders: decks::CAESAR_COMMANDERS,
            main: decks::CAESAR_MAIN,
        },
        // Hundred-and-seventy-first: The Fantastic Four (MSC, Marvel Super
        // Heroes) — four-color noncreature spells under Invisible Woman, whose
        // {R}{G}{W}{U} rules text sets her identity (CR 903.4). `--pod-decks 171`.
        PodDeck {
            name: "Invisible Woman (WURG)",
            commanders: decks::INVISIBLE_WOMAN_COMMANDERS,
            main: decks::INVISIBLE_WOMAN_MAIN,
        },
        PodDeck {
            name: "Inquisitor Greyfax (WUB)",
            commanders: decks::GREYFAX_COMMANDERS,
            main: decks::GREYFAX_MAIN,
        },
        // Hundred-and-seventy-third: Scrappy Survivors (Fallout) — Naya
        // Auras, Equipment and Junk under Dogmeat, Ever Loyal (CR 603.10a
        // Cass and Gunner Conscript read what rode the dead creature).
        // `--pod-decks 173`.
        PodDeck {
            name: "Dogmeat, Ever Loyal (RGW)",
            commanders: decks::DOGMEAT_COMMANDERS,
            main: decks::DOGMEAT_MAIN,
        },
        // Hundred-and-seventy-fourth: Avengers Assemble (MSC, Marvel Super
        // Heroes) — Jeskai Heroes under Captain America, Team Leader
        // (Jocasta's commander-attack return, CR 903.3). `--pod-decks 174`.
        PodDeck {
            name: "Captain America, Team Leader (URW)",
            commanders: decks::CAPTAIN_AMERICA_COMMANDERS,
            main: decks::CAPTAIN_AMERICA_MAIN,
        },
        // Hundred-and-seventy-fifth: Tyranid Swarm (Warhammer 40,000) — Temur
        // +1/+1 counters and {X} creatures under The Swarmlord (CR 106.6a
        // Biophagus's creature-cast counter rider). `--pod-decks 175`.
        PodDeck {
            name: "The Swarmlord (GUR)",
            commanders: decks::SWARMLORD_COMMANDERS,
            main: decks::SWARMLORD_MAIN,
        },
        // Hundred-and-seventy-sixth: Doom Prevails (MSC, Marvel Super
        // Heroes) — Grixis Villains under Doctor Doom, King of Latveria
        // (CR 701.50 connive payoffs). `--pod-decks 176`.
        PodDeck {
            name: "Doctor Doom, King of Latveria (UBR)",
            commanders: decks::DOOM_COMMANDERS,
            main: decks::DOOM_MAIN,
        },
        // Hundred-and-seventy-seventh: Necron Dynasties (40K) — mono-black
        // Necron artifacts under Szarekh, the Silent King (unearth's CR
        // 702.84a leave-exile, Out of the Tombs' CR 614 empty-draw
        // replacement). `--pod-decks 177`.
        PodDeck {
            name: "Szarekh, the Silent King (B)",
            commanders: decks::SZAREKH_COMMANDERS,
            main: decks::SZAREKH_MAIN,
        },
        // Hundred-and-seventy-eighth: Paradox Power (WHO, Doctor Who) — Temur
        // paradox under The Thirteenth Doctor and her companion Yasmin Khan
        // (CR 702.124m's Doctor's companion pair). `--pod-decks 178`.
        PodDeck {
            name: "The Thirteenth Doctor + Yasmin Khan (GUR)",
            commanders: decks::THIRTEENTH_COMMANDERS,
            main: decks::THIRTEENTH_MAIN,
        },
        // Hundred-and-seventy-ninth: The Ruinous Powers (Warhammer 40,000) —
        // Grixis Chaos cascade and Demons under Abaddon the Despoiler
        // (CR 508.1d Seeker of Slaanesh's forced attack). `--pod-decks 179`.
        PodDeck {
            name: "Abaddon the Despoiler (UBR)",
            commanders: decks::ABADDON_COMMANDERS,
            main: decks::ABADDON_MAIN,
        },
        // Hundred-and-eightieth: Turtle Power! (TMC, Teenage Mutant
        // Ninja Turtles) — five-color +1/+1 counters under Heroes in a Half
        // Shell (Krang's draw-trigger doubling, CR 603.2; Coin of Mastery's
        // artifact-mana count). `--pod-decks 180`.
        PodDeck {
            name: "Heroes in a Half Shell (WUBRG)",
            commanders: decks::HEROES_COMMANDERS,
            main: decks::HEROES_MAIN,
        },
        // Hundred-and-eighty-first: Masters of Evil (WHO, Doctor Who) — Grixis
        // Daleks, Cybermen and villainous choices under Davros, Dalek Creator
        // (CR 708.2 face-down Cybermen, CR 701.55 The Valeyard). `--pod-decks
        // 181`.
        PodDeck {
            name: "Davros, Dalek Creator (UBR)",
            commanders: decks::DAVROS_COMMANDERS,
            main: decks::DAVROS_MAIN,
        },
        // Hundred-and-eighty-second: Blast from the Past (WHO, Doctor Who) —
        // Bant historic spells and Sagas under The Fourth Doctor + Sarah
        // Jane Smith (CR 702.124m's Doctor's companion pair; Barbara
        // Wright's granted read ahead, CR 714.3c). `--pod-decks 182`.
        PodDeck {
            name: "The Fourth Doctor + Sarah Jane Smith (GWU)",
            commanders: decks::FOURTH_DOCTOR_COMMANDERS,
            main: decks::FOURTH_DOCTOR_MAIN,
        },
        // Hundred-and-eighty-third: Timey-Wimey (WHO, Doctor Who) — Jeskai
        // time travel and suspend under The Tenth Doctor + Rose Tyler (CR
        // 702.124m's Doctor's companion pair, CR 701.54 time travel).
        // `--pod-decks 183`.
        PodDeck {
            name: "The Tenth Doctor + Rose Tyler (URW)",
            commanders: decks::TENTH_DOCTOR_COMMANDERS,
            main: decks::TENTH_DOCTOR_MAIN,
        },
        // Hundred-and-eighty-fourth, and the first not taken from a precon:
        // Ob Nixilis, Captive Kingpin's EDHREC average deck (COMMANDER_BACKLOG
        // §1, CR 603.2c's "each lose exactly 1 life" batch). `--pod-decks 184`.
        PodDeck {
            name: "Ob Nixilis, Captive Kingpin (BR)",
            commanders: decks::OB_NIXILIS_KINGPIN_COMMANDERS,
            main: decks::OB_NIXILIS_KINGPIN_MAIN,
        },
        // Hundred-and-eighty-fifth: Storm, Force of Nature's EDHREC average
        // deck (CR 702.40 storm granted to the next spell). `--pod-decks 185`.
        PodDeck {
            name: "Storm, Force of Nature (GUR)",
            commanders: decks::STORM_FORCE_COMMANDERS,
            main: decks::STORM_FORCE_MAIN,
        },
        // Hundred-and-eighty-sixth: Indoraptor, the Perfect Hybrid's EDHREC
        // average deck (CR 702.130 enrage, a random opponent's punisher).
        // `--pod-decks 186`.
        PodDeck {
            name: "Indoraptor, the Perfect Hybrid (BGR)",
            commanders: decks::INDORAPTOR_COMMANDERS,
            main: decks::INDORAPTOR_MAIN,
        },
        // Hundred-and-eighty-seventh: Shadow the Hedgehog's EDHREC average
        // deck (CR 702.61 split second bought with artifact mana).
        // `--pod-decks 187`.
        PodDeck {
            name: "Shadow the Hedgehog (BR)",
            commanders: decks::SHADOW_HEDGEHOG_COMMANDERS,
            main: decks::SHADOW_HEDGEHOG_MAIN,
        },
        // Hundred-and-eighty-eighth: Ojer Axonil, Deepest Might's EDHREC
        // average deck (CR 614.1a's red noncombat floor, a god that dies into
        // a land). `--pod-decks 188`.
        PodDeck {
            name: "Ojer Axonil, Deepest Might (R)",
            commanders: decks::OJER_AXONIL_COMMANDERS,
            main: decks::OJER_AXONIL_MAIN,
        },
        // Hundred-and-eighty-ninth: Rowan, Scion of War's EDHREC average deck
        // (a turn discount sized by the life lost). `--pod-decks 189`.
        PodDeck {
            name: "Rowan, Scion of War (BR)",
            commanders: decks::ROWAN_SCION_COMMANDERS,
            main: decks::ROWAN_SCION_MAIN,
        },
        // Hundred-and-ninetieth: Deadpool, Trading Card's EDHREC average deck
        // (CR 612's exchanged text boxes). `--pod-decks 190`.
        PodDeck {
            name: "Deadpool, Trading Card (BR)",
            commanders: decks::DEADPOOL_COMMANDERS,
            main: decks::DEADPOOL_MAIN,
        },
        // Hundred-and-ninety-first: Satoru Umezawa's EDHREC average deck (CR
        // 702.49 granted ninjutsu). `--pod-decks 191`.
        PodDeck {
            name: "Satoru Umezawa (UB)",
            commanders: decks::SATORU_COMMANDERS,
            main: decks::SATORU_MAIN,
        },
        // Hundred-and-ninety-second: Volo, Guide to Monsters' EDHREC average
        // deck (copied creature spells, noted creature types). `--pod-decks 192`.
        PodDeck {
            name: "Volo, Guide to Monsters (GU)",
            commanders: decks::VOLO_COMMANDERS,
            main: decks::VOLO_MAIN,
        },
        // Hundred-and-ninety-third: Dr. Eggman's EDHREC average deck (CR
        // 701.55 villainous choices, Transformers converting). `--pod-decks 193`.
        PodDeck {
            name: "Dr. Eggman (BRU)",
            commanders: decks::EGGMAN_COMMANDERS,
            main: decks::EGGMAN_MAIN,
        },
        // Seat 194: Teysa Karlov's EDHREC average deck. `--pod-decks 194`.
        PodDeck {
            name: "Teysa Karlov (BW)",
            commanders: decks::TEYSA_COMMANDERS,
            main: decks::TEYSA_MAIN,
        },
        // Seat 195: Kilo, Apogee Mind's EDHREC average deck. `--pod-decks 195`.
        PodDeck {
            name: "Kilo, Apogee Mind (RUW)",
            commanders: decks::KILO_COMMANDERS,
            main: decks::KILO_MAIN,
        },
        // Seat 196: Niv-Mizzet, Parun's EDHREC average deck. `--pod-decks 196`.
        PodDeck {
            name: "Niv-Mizzet, Parun (RU)",
            commanders: decks::NIV_PARUN_COMMANDERS,
            main: decks::NIV_PARUN_MAIN,
        },
        // Seat 197: Yuna, Grand Summoner's EDHREC average deck. `--pod-decks 197`.
        PodDeck {
            name: "Yuna, Grand Summoner (GUW)",
            commanders: decks::YUNA_COMMANDERS,
            main: decks::YUNA_MAIN,
        },
        // Seat 198: Elsha, Threefold Master's EDHREC average deck. `--pod-decks 198`.
        PodDeck {
            name: "Elsha, Threefold Master (RUW)",
            commanders: decks::ELSHA_COMMANDERS,
            main: decks::ELSHA_MAIN,
        },
        // Seat 199: Kinnan, Bonder Prodigy's EDHREC average deck. `--pod-decks 199`.
        PodDeck {
            name: "Kinnan, Bonder Prodigy (GU)",
            commanders: decks::KINNAN_COMMANDERS,
            main: decks::KINNAN_MAIN,
        },
        // Seat 200: Toxrill, the Corrosive's EDHREC average deck. `--pod-decks 200`.
        PodDeck {
            name: "Toxrill, the Corrosive (BU)",
            commanders: decks::TOXRILL_COMMANDERS,
            main: decks::TOXRILL_MAIN,
        },
        // Seat 201: The Wandering Minstrel's EDHREC average deck. `--pod-decks 201`.
        PodDeck {
            name: "The Wandering Minstrel (BGRUW)",
            commanders: decks::MINSTREL_COMMANDERS,
            main: decks::MINSTREL_MAIN,
        },
        // Seat 202: The First Sliver's EDHREC average deck. `--pod-decks 202`.
        PodDeck {
            name: "The First Sliver (BGRUW)",
            commanders: decks::FIRST_SLIVER_COMMANDERS,
            main: decks::FIRST_SLIVER_MAIN,
        },
        // Seat 203: Zur the Enchanter's EDHREC average deck. `--pod-decks 203`.
        PodDeck {
            name: "Zur the Enchanter (BUW)",
            commanders: decks::ZUR_COMMANDERS,
            main: decks::ZUR_MAIN,
        },
        // Seat 204: Etali, Primal Conqueror's EDHREC average deck. `--pod-decks 204`.
        PodDeck {
            name: "Etali, Primal Conqueror (GR)",
            commanders: decks::ETALI_PC_COMMANDERS,
            main: decks::ETALI_PC_MAIN,
        },
        // Seat 205: Magus Lucea Kane's EDHREC average deck. `--pod-decks 205`.
        PodDeck {
            name: "Magus Lucea Kane (GRU)",
            commanders: decks::LUCEA_COMMANDERS,
            main: decks::LUCEA_MAIN,
        },
        // Seat 206: Tivit, Seller of Secrets's EDHREC average deck. `--pod-decks 206`.
        PodDeck {
            name: "Tivit, Seller of Secrets (BUW)",
            commanders: decks::TIVIT_COMMANDERS,
            main: decks::TIVIT_MAIN,
        },
        // Seat 207: Umbris, Fear Manifest's EDHREC average deck. `--pod-decks 207`.
        PodDeck {
            name: "Umbris, Fear Manifest (BU)",
            commanders: decks::UMBRIS_COMMANDERS,
            main: decks::UMBRIS_MAIN,
        },
        // Seat 208: Prismari, the Inspiration's EDHREC average deck. `--pod-decks 208`.
        PodDeck {
            name: "Prismari, the Inspiration (RU)",
            commanders: decks::PRISMARI_COMMANDERS,
            main: decks::PRISMARI_MAIN,
        },
        // Seat 209: Shalai and Hallar's EDHREC average deck. `--pod-decks 209`.
        PodDeck {
            name: "Shalai and Hallar (GRW)",
            commanders: decks::SHALAI_COMMANDERS,
            main: decks::SHALAI_MAIN,
        },
        // Seat 210: Phenax, God of Deception's EDHREC average deck. `--pod-decks 210`.
        PodDeck {
            name: "Phenax, God of Deception (BU)",
            commanders: decks::PHENAX_COMMANDERS,
            main: decks::PHENAX_MAIN,
        },
        // Seat 211: Baylen, the Haymaker's EDHREC average deck. `--pod-decks 211`.
        PodDeck {
            name: "Baylen, the Haymaker (GRW)",
            commanders: decks::BAYLEN_COMMANDERS,
            main: decks::BAYLEN_MAIN,
        },
        // Seat 212: Chatterfang, Squirrel General's EDHREC average deck. `--pod-decks 212`.
        PodDeck {
            name: "Chatterfang, Squirrel General (BG)",
            commanders: decks::CHATTERFANG_COMMANDERS,
            main: decks::CHATTERFANG_MAIN,
        },
        // Seat 213: Ureni of the Unwritten's EDHREC average deck. `--pod-decks 213`.
        PodDeck {
            name: "Ureni of the Unwritten (GRU)",
            commanders: decks::URENI_COMMANDERS,
            main: decks::URENI_MAIN,
        },
        // Seat 214: Xyris, the Writhing Storm's EDHREC average deck. `--pod-decks 214`.
        PodDeck {
            name: "Xyris, the Writhing Storm (GRU)",
            commanders: decks::XYRIS_COMMANDERS,
            main: decks::XYRIS_MAIN,
        },
        // Seat 215: Yarok, the Desecrated's EDHREC average deck. `--pod-decks 215`.
        PodDeck {
            name: "Yarok, the Desecrated (BGU)",
            commanders: decks::YAROK_COMMANDERS,
            main: decks::YAROK_MAIN,
        },
        // Seat 216: Rendmaw, Creaking Nest's EDHREC average deck. `--pod-decks 216`.
        PodDeck {
            name: "Rendmaw, Creaking Nest (BG)",
            commanders: decks::RENDMAW_COMMANDERS,
            main: decks::RENDMAW_MAIN,
        },
        // Seat 217: Celes, Rune Knight's EDHREC average deck. `--pod-decks 217`.
        PodDeck {
            name: "Celes, Rune Knight (BRW)",
            commanders: decks::CELES_COMMANDERS,
            main: decks::CELES_MAIN,
        },
        // Seat 218: Sheoldred, the Apocalypse's EDHREC average deck. `--pod-decks 218`.
        PodDeck {
            name: "Sheoldred, the Apocalypse (B)",
            commanders: decks::SHEOLDRED_APOC_COMMANDERS,
            main: decks::SHEOLDRED_APOC_MAIN,
        },
        // Seat 219: Silverquill, the Disputant's EDHREC average deck. `--pod-decks 219`.
        PodDeck {
            name: "Silverquill, the Disputant (BW)",
            commanders: decks::SILVERQUILL_COMMANDERS,
            main: decks::SILVERQUILL_MAIN,
        },
        // Seat 220: Chulane, Teller of Tales's EDHREC average deck. `--pod-decks 220`.
        PodDeck {
            name: "Chulane, Teller of Tales (GUW)",
            commanders: decks::CHULANE_COMMANDERS,
            main: decks::CHULANE_MAIN,
        },
        // Seat 221: Finneas, Ace Archer's EDHREC average deck. `--pod-decks 221`.
        PodDeck {
            name: "Finneas, Ace Archer (GW)",
            commanders: decks::FINNEAS_COMMANDERS,
            main: decks::FINNEAS_MAIN,
        },
        // Seat 222: The Infamous Cruelclaw's EDHREC average deck. `--pod-decks 222`.
        PodDeck {
            name: "The Infamous Cruelclaw (BR)",
            commanders: decks::CRUELCLAW_COMMANDERS,
            main: decks::CRUELCLAW_MAIN,
        },
        // Seat 223: Vivi Ornitier's EDHREC average deck. `--pod-decks 223`.
        PodDeck {
            name: "Vivi Ornitier (RU)",
            commanders: decks::VIVI_COMMANDERS,
            main: decks::VIVI_MAIN,
        },
        // Seat 224: Nekusar, the Mindrazer's EDHREC average deck. `--pod-decks 224`.
        PodDeck {
            name: "Nekusar, the Mindrazer (BRU)",
            commanders: decks::NEKUSAR_COMMANDERS,
            main: decks::NEKUSAR_MAIN,
        },
        // Seat 225: Sephiroth, Fabled SOLDIER's EDHREC average deck. `--pod-decks 225`.
        PodDeck {
            name: "Sephiroth, Fabled SOLDIER (B)",
            commanders: decks::SEPHIROTH_COMMANDERS,
            main: decks::SEPHIROTH_MAIN,
        },
        // Seat 226: Zaxara, the Exemplary's EDHREC average deck. `--pod-decks 226`.
        PodDeck {
            name: "Zaxara, the Exemplary (BGU)",
            commanders: decks::ZAXARA_COMMANDERS,
            main: decks::ZAXARA_MAIN,
        },
        // Seat 227: Omnath, Locus of Creation's EDHREC average deck. `--pod-decks 227`.
        PodDeck {
            name: "Omnath, Locus of Creation (GRUW)",
            commanders: decks::OMNATH_LOC_COMMANDERS,
            main: decks::OMNATH_LOC_MAIN,
        },
        // Seat 228: Marneus Calgar's EDHREC average deck. `--pod-decks 228`.
        PodDeck {
            name: "Marneus Calgar (BUW)",
            commanders: decks::MARNEUS_COMMANDERS,
            main: decks::MARNEUS_MAIN,
        },
        // Seat 229: Kykar, Wind's Fury's EDHREC average deck. `--pod-decks 229`.
        PodDeck {
            name: "Kykar, Wind's Fury (RUW)",
            commanders: decks::KYKAR_COMMANDERS,
            main: decks::KYKAR_MAIN,
        },
        // Seat 230: Varina, Lich Queen's EDHREC average deck. `--pod-decks 230`.
        PodDeck {
            name: "Varina, Lich Queen (BUW)",
            commanders: decks::VARINA_COMMANDERS,
            main: decks::VARINA_MAIN,
        },
        // Seat 231: Alania, Divergent Storm's EDHREC average deck. `--pod-decks 231`.
        PodDeck {
            name: "Alania, Divergent Storm (RU)",
            commanders: decks::ALANIA_COMMANDERS,
            main: decks::ALANIA_MAIN,
        },
        // Seat 232: Veyran, Voice of Duality's EDHREC average deck. `--pod-decks 232`.
        PodDeck {
            name: "Veyran, Voice of Duality (RU)",
            commanders: decks::VEYRAN_COMMANDERS,
            main: decks::VEYRAN_MAIN,
        },
        // Seat 233: Bria, Riptide Rogue's EDHREC average deck. `--pod-decks 233`.
        PodDeck {
            name: "Bria, Riptide Rogue (RU)",
            commanders: decks::BRIA_COMMANDERS,
            main: decks::BRIA_MAIN,
        },
        // Seat 234: Noctis, Prince of Lucis's EDHREC average deck. `--pod-decks 234`.
        PodDeck {
            name: "Noctis, Prince of Lucis (BUW)",
            commanders: decks::NOCTIS_COMMANDERS,
            main: decks::NOCTIS_MAIN,
        },
        // Seat 235: Omnath, Locus of All's EDHREC average deck (CR 106.4 mana
        // that becomes black, CR 310.12b a Siege's sorcery back face).
        // `--pod-decks 235`.
        PodDeck {
            name: "Omnath, Locus of All (WUBRG)",
            commanders: decks::OMNATH_LOCUS_COMMANDERS,
            main: decks::OMNATH_LOCUS_MAIN,
        },
        // Seat 236: Muldrotha, the Gravetide's EDHREC average deck. `--pod-decks 236`.
        PodDeck {
            name: "Muldrotha, the Gravetide (BGU)",
            commanders: decks::MULDROTHA_COMMANDERS,
            main: decks::MULDROTHA_MAIN,
        },
        // Seat 237: Flubs, the Fool's EDHREC average deck. `--pod-decks 237`.
        PodDeck {
            name: "Flubs, the Fool (GRU)",
            commanders: decks::FLUBS_COMMANDERS,
            main: decks::FLUBS_MAIN,
        },
        // Seat 238: Liesa, Shroud of Dusk's EDHREC average deck. `--pod-decks 238`.
        PodDeck {
            name: "Liesa, Shroud of Dusk (BW)",
            commanders: decks::LIESA_SHROUD_COMMANDERS,
            main: decks::LIESA_SHROUD_MAIN,
        },
        // Seat 239: Ramos, Dragon Engine's EDHREC average deck. `--pod-decks 239`.
        PodDeck {
            name: "Ramos, Dragon Engine (BGRUW)",
            commanders: decks::RAMOS_COMMANDERS,
            main: decks::RAMOS_MAIN,
        },
        // Seat 240: Omnath, Locus of Mana's EDHREC average deck. `--pod-decks 240`.
        PodDeck {
            name: "Omnath, Locus of Mana (G)",
            commanders: decks::OMNATH_MANA_COMMANDERS,
            main: decks::OMNATH_MANA_MAIN,
        },
        // Seat 241: Tannuk, Memorial Ensign's EDHREC average deck. `--pod-decks 241`.
        PodDeck {
            name: "Tannuk, Memorial Ensign (GR)",
            commanders: decks::TANNUK_COMMANDERS,
            main: decks::TANNUK_MAIN,
        },
        // Seat 242: Gargos, Vicious Watcher's EDHREC average deck. `--pod-decks 242`.
        PodDeck {
            name: "Gargos, Vicious Watcher (G)",
            commanders: decks::GARGOS_COMMANDERS,
            main: decks::GARGOS_MAIN,
        },
        // Seat 243: Animar, Soul of Elements's EDHREC average deck. `--pod-decks 243`.
        PodDeck {
            name: "Animar, Soul of Elements (GRU)",
            commanders: decks::ANIMAR_COMMANDERS,
            main: decks::ANIMAR_MAIN,
        },
        // Seat 244: Ghyrson Starn, Kelermorph's EDHREC average deck. `--pod-decks 244`.
        PodDeck {
            name: "Ghyrson Starn, Kelermorph (RU)",
            commanders: decks::GHYRSON_COMMANDERS,
            main: decks::GHYRSON_MAIN,
        },
        // Seat 245: Sliver Overlord's EDHREC average deck. `--pod-decks 245`.
        PodDeck {
            name: "Sliver Overlord (BGRUW)",
            commanders: decks::SLIVER_OVERLORD_COMMANDERS,
            main: decks::SLIVER_OVERLORD_MAIN,
        },
        // Seat 246: The Gitrog Monster's EDHREC average deck. `--pod-decks 246`.
        PodDeck {
            name: "The Gitrog Monster (BG)",
            commanders: decks::GITROG_COMMANDERS,
            main: decks::GITROG_MAIN,
        },
        // Seat 247: Dina, Soul Steeper's EDHREC average deck. `--pod-decks 247`.
        PodDeck {
            name: "Dina, Soul Steeper (BG)",
            commanders: decks::DINA_STEEPER_COMMANDERS,
            main: decks::DINA_STEEPER_MAIN,
        },
        // Seat 248: Betor, Ancestor's Voice's EDHREC average deck. `--pod-decks 248`.
        PodDeck {
            name: "Betor, Ancestor's Voice (BGW)",
            commanders: decks::BETOR_COMMANDERS,
            main: decks::BETOR_MAIN,
        },
        // Seat 249: Witherbloom, the Balancer's EDHREC average deck. `--pod-decks 249`.
        PodDeck {
            name: "Witherbloom, the Balancer (BG)",
            commanders: decks::WITHERBLOOM_COMMANDERS,
            main: decks::WITHERBLOOM_MAIN,
        },
        // Seat 250: Judith, Carnage Connoisseur's EDHREC average deck. `--pod-decks 250`.
        PodDeck {
            name: "Judith, Carnage Connoisseur (BR)",
            commanders: decks::JUDITH_CARNAGE_COMMANDERS,
            main: decks::JUDITH_CARNAGE_MAIN,
        },
        // Seat 251: Korvold, Fae-Cursed King's EDHREC average deck. `--pod-decks 251`.
        PodDeck {
            name: "Korvold, Fae-Cursed King (BGR)",
            commanders: decks::KORVOLD_COMMANDERS,
            main: decks::KORVOLD_MAIN,
        },
        // Seat 252: Hope Estheim's EDHREC average deck. `--pod-decks 252`.
        PodDeck {
            name: "Hope Estheim (UW)",
            commanders: decks::HOPE_ESTHEIM_COMMANDERS,
            main: decks::HOPE_ESTHEIM_MAIN,
        },
        // Seat 253: Alela, Artful Provocateur's EDHREC average deck. `--pod-decks 253`.
        PodDeck {
            name: "Alela, Artful Provocateur (BUW)",
            commanders: decks::ALELA_COMMANDERS,
            main: decks::ALELA_MAIN,
        },
        // Seat 254: The Scarab God's EDHREC average deck. `--pod-decks 254`.
        PodDeck {
            name: "The Scarab God (BU)",
            commanders: decks::SCARAB_GOD_COMMANDERS,
            main: decks::SCARAB_GOD_MAIN,
        },
        // Seat 255: Grand Arbiter Augustin IV's EDHREC average deck. `--pod-decks 255`.
        PodDeck {
            name: "Grand Arbiter Augustin IV (UW)",
            commanders: decks::GRAND_ARBITER_COMMANDERS,
            main: decks::GRAND_ARBITER_MAIN,
        },
        // Seat 256: Thalia and The Gitrog Monster's EDHREC average deck. `--pod-decks 256`.
        PodDeck {
            name: "Thalia and The Gitrog Monster (BGW)",
            commanders: decks::THALIA_GITROG_COMMANDERS,
            main: decks::THALIA_GITROG_MAIN,
        },
        // Seat 257: Queza, Augur of Agonies's EDHREC average deck. `--pod-decks 257`.
        PodDeck {
            name: "Queza, Augur of Agonies (BUW)",
            commanders: decks::QUEZA_COMMANDERS,
            main: decks::QUEZA_MAIN,
        },
        // Seat 258: Omnath, Locus of Rage's EDHREC average deck. `--pod-decks 258`.
        PodDeck {
            name: "Omnath, Locus of Rage (GR)",
            commanders: decks::OMNATH_RAGE_COMMANDERS,
            main: decks::OMNATH_RAGE_MAIN,
        },
        // Seat 259: The Necrobloom's EDHREC average deck. `--pod-decks 259`.
        PodDeck {
            name: "The Necrobloom (BGW)",
            commanders: decks::NECROBLOOM_COMMANDERS,
            main: decks::NECROBLOOM_MAIN,
        },
        // Seat 260: Atraxa, Grand Unifier's EDHREC average deck. `--pod-decks 260`.
        PodDeck {
            name: "Atraxa, Grand Unifier (BGUW)",
            commanders: decks::ATRAXA_GU_COMMANDERS,
            main: decks::ATRAXA_GU_MAIN,
        },
        // Seat 261: Codie, Vociferous Codex's EDHREC average deck. `--pod-decks 261`.
        PodDeck {
            name: "Codie, Vociferous Codex (BGRUW)",
            commanders: decks::CODIE_COMMANDERS,
            main: decks::CODIE_MAIN,
        },
        // Seat 262: Blech, Loafing Pest's EDHREC average deck. `--pod-decks 262`.
        PodDeck {
            name: "Blech, Loafing Pest (BG)",
            commanders: decks::BLECH_COMMANDERS,
            main: decks::BLECH_MAIN,
        },
        // Seat 263: Fynn, the Fangbearer's EDHREC average deck. `--pod-decks 263`.
        PodDeck {
            name: "Fynn, the Fangbearer (G)",
            commanders: decks::FYNN_COMMANDERS,
            main: decks::FYNN_MAIN,
        },
        // Seat 264: Feather, the Redeemed's EDHREC average deck. `--pod-decks 264`.
        PodDeck {
            name: "Feather, the Redeemed (RW)",
            commanders: decks::FEATHER_COMMANDERS,
            main: decks::FEATHER_MAIN,
        },
        // Seat 265: Narset, Enlightened Exile's EDHREC average deck. `--pod-decks 265`.
        PodDeck {
            name: "Narset, Enlightened Exile (RUW)",
            commanders: decks::NARSET_EXILE_COMMANDERS,
            main: decks::NARSET_EXILE_MAIN,
        },
        // Seat 266: Raffine, Scheming Seer's EDHREC average deck. `--pod-decks 266`.
        PodDeck {
            name: "Raffine, Scheming Seer (BUW)",
            commanders: decks::RAFFINE_COMMANDERS,
            main: decks::RAFFINE_MAIN,
        },
        // Seat 267: The Mindskinner's EDHREC average deck. `--pod-decks 267`.
        PodDeck {
            name: "The Mindskinner (U)",
            commanders: decks::MINDSKINNER_COMMANDERS,
            main: decks::MINDSKINNER_MAIN,
        },
        // Seat 268: Atla Palani, Nest Tender's EDHREC average deck. `--pod-decks 268`.
        PodDeck {
            name: "Atla Palani, Nest Tender (GRW)",
            commanders: decks::ATLA_COMMANDERS,
            main: decks::ATLA_MAIN,
        },
        // Seat 269: Betor, Kin to All's EDHREC average deck. `--pod-decks 269`.
        PodDeck {
            name: "Betor, Kin to All (BGW)",
            commanders: decks::BETOR_KIN_COMMANDERS,
            main: decks::BETOR_KIN_MAIN,
        },
        // Seat 270: Lumra, Bellow of the Woods's EDHREC average deck. `--pod-decks 270`.
        PodDeck {
            name: "Lumra, Bellow of the Woods (G)",
            commanders: decks::LUMRA_COMMANDERS,
            main: decks::LUMRA_MAIN,
        },
        // Seat 271: Marina Vendrell's EDHREC average deck. `--pod-decks 271`.
        PodDeck {
            name: "Marina Vendrell (BGRUW)",
            commanders: decks::MARINA_COMMANDERS,
            main: decks::MARINA_MAIN,
        },
        // Seat 272: Aloy, Savior of Meridian's EDHREC average deck. `--pod-decks 272`.
        PodDeck {
            name: "Aloy, Savior of Meridian (GU)",
            commanders: decks::ALOY_COMMANDERS,
            main: decks::ALOY_MAIN,
        },
        // Seat 273: Marchesa, the Black Rose's EDHREC average deck. `--pod-decks 273`.
        PodDeck {
            name: "Marchesa, the Black Rose (BRU)",
            commanders: decks::MARCHESA_BR_COMMANDERS,
            main: decks::MARCHESA_BR_MAIN,
        },
        // Seat 274: Brago, King Eternal's EDHREC average deck. `--pod-decks 274`.
        PodDeck {
            name: "Brago, King Eternal (UW)",
            commanders: decks::BRAGO_COMMANDERS,
            main: decks::BRAGO_MAIN,
        },
        // Seat 275: Jetmir, Nexus of Revels's EDHREC average deck. `--pod-decks 275`.
        PodDeck {
            name: "Jetmir, Nexus of Revels (GRW)",
            commanders: decks::JETMIR_COMMANDERS,
            main: decks::JETMIR_MAIN,
        },
        // Seat 276: Kefka, Court Mage's EDHREC average deck. `--pod-decks 276`.
        PodDeck {
            name: "Kefka, Court Mage (BRU)",
            commanders: decks::KEFKA_COMMANDERS,
            main: decks::KEFKA_MAIN,
        },
        // Seat 277: Glarb, Calamity's Augur's EDHREC average deck. `--pod-decks 277`.
        PodDeck {
            name: "Glarb, Calamity's Augur (BGU)",
            commanders: decks::GLARB_COMMANDERS,
            main: decks::GLARB_MAIN,
        },
        // Seat 278: Hashaton, Scarab's Fist's EDHREC average deck. `--pod-decks 278`.
        PodDeck {
            name: "Hashaton, Scarab's Fist (BUW)",
            commanders: decks::HASHATON_COMMANDERS,
            main: decks::HASHATON_MAIN,
        },
        // Seat 279: Kraum, Ludevic's Opus + Tymna the Weaver's EDHREC average deck. `--pod-decks 279`.
        PodDeck {
            name: "Kraum, Ludevic's Opus + Tymna the Weaver (BRUW)",
            commanders: decks::KRAUM_COMMANDERS,
            main: decks::KRAUM_MAIN,
        },
        // Seat 280: Thrasios, Triton Hero + Tymna the Weaver's EDHREC average deck. `--pod-decks 280`.
        PodDeck {
            name: "Thrasios, Triton Hero + Tymna the Weaver (BGUW)",
            commanders: decks::THRASIOS_COMMANDERS,
            main: decks::THRASIOS_MAIN,
        },
        // Seat 281: Infinite Guideline Station's EDHREC average deck. `--pod-decks 281`.
        PodDeck {
            name: "Infinite Guideline Station (BGRUW)",
            commanders: decks::GUIDELINE_COMMANDERS,
            main: decks::GUIDELINE_MAIN,
        },
        // Seat 282: Gishath, Sun's Avatar's EDHREC average deck. `--pod-decks 282`.
        PodDeck {
            name: "Gishath, Sun's Avatar (GRW)",
            commanders: decks::GISHATH_COMMANDERS,
            main: decks::GISHATH_MAIN,
        },
        // Seat 283: Shorikai, Genesis Engine's EDHREC average deck. `--pod-decks 283`.
        PodDeck {
            name: "Shorikai, Genesis Engine (UW)",
            commanders: decks::SHORIKAI_COMMANDERS,
            main: decks::SHORIKAI_MAIN,
        },
        // Seat 284: Golbez, Crystal Collector's EDHREC average deck. `--pod-decks 284`.
        PodDeck {
            name: "Golbez, Crystal Collector (BU)",
            commanders: decks::GOLBEZ_COMMANDERS,
            main: decks::GOLBEZ_MAIN,
        },
        // Seat 285: Urza, Lord High Artificer's EDHREC average deck. `--pod-decks 285`.
        PodDeck {
            name: "Urza, Lord High Artificer (U)",
            commanders: decks::URZA_LHA_COMMANDERS,
            main: decks::URZA_LHA_MAIN,
        },
        // Seat 286: Mendicant Core, Guidelight's EDHREC average deck. `--pod-decks 286`.
        PodDeck {
            name: "Mendicant Core, Guidelight (UW)",
            commanders: decks::MENDICANT_COMMANDERS,
            main: decks::MENDICANT_MAIN,
        },
        // Seat 287: Wick, the Whorled Mind's EDHREC average deck. `--pod-decks 287`.
        PodDeck {
            name: "Wick, the Whorled Mind (BRU)",
            commanders: decks::WICK_COMMANDERS,
            main: decks::WICK_MAIN,
        },
        // Seat 288: Vren, the Relentless's EDHREC average deck. `--pod-decks 288`.
        PodDeck {
            name: "Vren, the Relentless (BU)",
            commanders: decks::VREN_COMMANDERS,
            main: decks::VREN_MAIN,
        },
        // Seat 289: Krenko, Mob Boss's EDHREC average deck. `--pod-decks 289`.
        PodDeck {
            name: "Krenko, Mob Boss (R)",
            commanders: decks::KRENKO_MB_COMMANDERS,
            main: decks::KRENKO_MB_MAIN,
        },
        // Seat 290: Arabella, Abandoned Doll's EDHREC average deck. `--pod-decks 290`.
        PodDeck {
            name: "Arabella, Abandoned Doll (RW)",
            commanders: decks::ARABELLA_COMMANDERS,
            main: decks::ARABELLA_MAIN,
        },
        // Seat 291: Coram, the Undertaker's EDHREC average deck. `--pod-decks 291`.
        PodDeck {
            name: "Coram, the Undertaker (BGR)",
            commanders: decks::CORAM_COMMANDERS,
            main: decks::CORAM_MAIN,
        },
        // Seat 292: Sythis, Harvest's Hand's EDHREC average deck. `--pod-decks 292`.
        PodDeck {
            name: "Sythis, Harvest's Hand (GW)",
            commanders: decks::SYTHIS_COMMANDERS,
            main: decks::SYTHIS_MAIN,
        },
        // Seat 293: Jin Sakai, Ghost of Tsushima's EDHREC average deck. `--pod-decks 293`.
        PodDeck {
            name: "Jin Sakai, Ghost of Tsushima (BUW)",
            commanders: decks::JIN_SAKAI_COMMANDERS,
            main: decks::JIN_SAKAI_MAIN,
        },
        // Seat 294: Sonic the Hedgehog's EDHREC average deck. `--pod-decks 294`.
        PodDeck {
            name: "Sonic the Hedgehog (RUW)",
            commanders: decks::SONIC_COMMANDERS,
            main: decks::SONIC_MAIN,
        },
        // Seat 295: Arcades, the Strategist's EDHREC average deck. `--pod-decks 295`.
        PodDeck {
            name: "Arcades, the Strategist (GUW)",
            commanders: decks::ARCADES_COMMANDERS,
            main: decks::ARCADES_MAIN,
        },
        // Seat 296: Queen Marchesa's EDHREC average deck. `--pod-decks 296`.
        PodDeck {
            name: "Queen Marchesa (BRW)",
            commanders: decks::QUEEN_MARCHESA_COMMANDERS,
            main: decks::QUEEN_MARCHESA_MAIN,
        },
        // Seat 297: Kenrith, the Returned King's EDHREC average deck. `--pod-decks 297`.
        PodDeck {
            name: "Kenrith, the Returned King (BGRUW)",
            commanders: decks::KENRITH_COMMANDERS,
            main: decks::KENRITH_MAIN,
        },
        // Seat 298: Fire Lord Azula's EDHREC average deck. `--pod-decks 298`.
        PodDeck {
            name: "Fire Lord Azula (BRU)",
            commanders: decks::FIRE_LORD_AZULA_COMMANDERS,
            main: decks::FIRE_LORD_AZULA_MAIN,
        },
        // Seat 299: Minsc & Boo, Timeless Heroes's EDHREC average deck. `--pod-decks 299`.
        PodDeck {
            name: "Minsc & Boo, Timeless Heroes (GR)",
            commanders: decks::MINSC_BOO_COMMANDERS,
            main: decks::MINSC_BOO_MAIN,
        },
        // Seat 300: The Jolly Balloon Man's EDHREC average deck. `--pod-decks 300`.
        PodDeck {
            name: "The Jolly Balloon Man (RW)",
            commanders: decks::JOLLY_BALLOON_MAN_COMMANDERS,
            main: decks::JOLLY_BALLOON_MAN_MAIN,
        },
        // Seat 301: Eriette of the Charmed Apple's EDHREC average deck. `--pod-decks 301`.
        PodDeck {
            name: "Eriette of the Charmed Apple (BW)",
            commanders: decks::ERIETTE_COMMANDERS,
            main: decks::ERIETTE_MAIN,
        },
        // Seat 302: Alela, Cunning Conqueror's EDHREC average deck. `--pod-decks 302`.
        PodDeck {
            name: "Alela, Cunning Conqueror (BU)",
            commanders: decks::ALELA_CUNNING_COMMANDERS,
            main: decks::ALELA_CUNNING_MAIN,
        },
        // Seat 303: Ovika, Enigma Goliath's EDHREC average deck. `--pod-decks 303`.
        PodDeck {
            name: "Ovika, Enigma Goliath (RU)",
            commanders: decks::OVIKA_COMMANDERS,
            main: decks::OVIKA_MAIN,
        },
        // Seat 304: Tannuk, Steadfast Second's EDHREC average deck. `--pod-decks 304`.
        PodDeck {
            name: "Tannuk, Steadfast Second (R)",
            commanders: decks::TANNUK_STEADFAST_COMMANDERS,
            main: decks::TANNUK_STEADFAST_MAIN,
        },
        // Seat 305: Gev, Scaled Scorch's EDHREC average deck. `--pod-decks 305`.
        PodDeck {
            name: "Gev, Scaled Scorch (BR)",
            commanders: decks::GEV_COMMANDERS,
            main: decks::GEV_MAIN,
        },
        // Seat 306: Kudo, King Among Bears's EDHREC average deck. `--pod-decks 306`.
        PodDeck {
            name: "Kudo, King Among Bears (GW)",
            commanders: decks::KUDO_COMMANDERS,
            main: decks::KUDO_MAIN,
        },
        // Seat 307: Rocco, Cabaretti Caterer's EDHREC average deck. `--pod-decks 307`.
        PodDeck {
            name: "Rocco, Cabaretti Caterer (GRW)",
            commanders: decks::ROCCO_COMMANDERS,
            main: decks::ROCCO_MAIN,
        },
        // Seat 308: Sokka, Tenacious Tactician's EDHREC average deck. `--pod-decks 308`.
        PodDeck {
            name: "Sokka, Tenacious Tactician (RUW)",
            commanders: decks::SOKKA_COMMANDERS,
            main: decks::SOKKA_MAIN,
        },
        // Seat 309: Tifa Lockhart's EDHREC average deck. `--pod-decks 309`.
        PodDeck {
            name: "Tifa Lockhart (G)",
            commanders: decks::TIFA_COMMANDERS,
            main: decks::TIFA_MAIN,
        },
        // Seat 310: Winota, Joiner of Forces's EDHREC average deck. `--pod-decks 310`.
        PodDeck {
            name: "Winota, Joiner of Forces (RW)",
            commanders: decks::WINOTA_COMMANDERS,
            main: decks::WINOTA_MAIN,
        },
        // Seat 311: Ziatora, the Incinerator's EDHREC average deck. `--pod-decks 311`.
        PodDeck {
            name: "Ziatora, the Incinerator (BGR)",
            commanders: decks::ZIATORA_COMMANDERS,
            main: decks::ZIATORA_MAIN,
        },
        // Seat 312: Helga, Skittish Seer's EDHREC average deck. `--pod-decks 312`.
        PodDeck {
            name: "Helga, Skittish Seer (GUW)",
            commanders: decks::HELGA_COMMANDERS,
            main: decks::HELGA_MAIN,
        },
        // Seat 313: Vihaan, Goldwaker's EDHREC average deck. `--pod-decks 313`.
        PodDeck {
            name: "Vihaan, Goldwaker (BRW)",
            commanders: decks::VIHAAN_COMMANDERS,
            main: decks::VIHAAN_MAIN,
        },
        // Seat 314: Tayam, Luminous Enigma's EDHREC average deck. `--pod-decks 314`.
        PodDeck {
            name: "Tayam, Luminous Enigma (BGW)",
            commanders: decks::TAYAM_COMMANDERS,
            main: decks::TAYAM_MAIN,
        },
        // Seat 315: Brenard, Ginger Sculptor's EDHREC average deck. `--pod-decks 315`.
        PodDeck {
            name: "Brenard, Ginger Sculptor (GUW)",
            commanders: decks::BRENARD_COMMANDERS,
            main: decks::BRENARD_MAIN,
        },
        // Seat 316: Hazezon, Shaper of Sand's EDHREC average deck. `--pod-decks 316`.
        PodDeck {
            name: "Hazezon, Shaper of Sand (GRW)",
            commanders: decks::HAZEZON_COMMANDERS,
            main: decks::HAZEZON_MAIN,
        },
        // Seat 317: Eluge, the Shoreless Sea's EDHREC average deck. `--pod-decks 317`.
        PodDeck {
            name: "Eluge, the Shoreless Sea (U)",
            commanders: decks::ELUGE_COMMANDERS,
            main: decks::ELUGE_MAIN,
        },
        // Seat 318: Roxanne, Starfall Savant's EDHREC average deck. `--pod-decks 318`.
        PodDeck {
            name: "Roxanne, Starfall Savant (GR)",
            commanders: decks::ROXANNE_COMMANDERS,
            main: decks::ROXANNE_MAIN,
        },
        // Seat 319: Lightning, Army of One's EDHREC average deck. `--pod-decks 319`.
        PodDeck {
            name: "Lightning, Army of One (RW)",
            commanders: decks::LIGHTNING_ARMY_COMMANDERS,
            main: decks::LIGHTNING_ARMY_MAIN,
        },
        // Seat 320: Isshin, Two Heavens as One's EDHREC average deck. `--pod-decks 320`.
        PodDeck {
            name: "Isshin, Two Heavens as One (BRW)",
            commanders: decks::ISSHIN_COMMANDERS,
            main: decks::ISSHIN_MAIN,
        },
        // Seat 321: Jodah, the Unifier's EDHREC average deck. `--pod-decks 321`.
        PodDeck {
            name: "Jodah, the Unifier (BGRUW)",
            commanders: decks::JODAH_UNIFIER_COMMANDERS,
            main: decks::JODAH_UNIFIER_MAIN,
        },
        // Seat 322: Ygra, Eater of All's EDHREC average deck. `--pod-decks 322`.
        PodDeck {
            name: "Ygra, Eater of All (BG)",
            commanders: decks::YGRA_COMMANDERS,
            main: decks::YGRA_MAIN,
        },
        // Seat 323: Orvar, the All-Form's EDHREC average deck. `--pod-decks 323`.
        PodDeck {
            name: "Orvar, the All-Form (U)",
            commanders: decks::ORVAR_COMMANDERS,
            main: decks::ORVAR_MAIN,
        },
        // Seat 324: Sisay, Weatherlight Captain's EDHREC average deck. `--pod-decks 324`.
        PodDeck {
            name: "Sisay, Weatherlight Captain (BGRUW)",
            commanders: decks::SISAY_COMMANDERS,
            main: decks::SISAY_MAIN,
        },
        // Seat 325: Kuja, Genome Sorcerer's EDHREC average deck. `--pod-decks 325`.
        PodDeck {
            name: "Kuja, Genome Sorcerer (BR)",
            commanders: decks::KUJA_COMMANDERS,
            main: decks::KUJA_MAIN,
        },
        // Seat 326: Urtet, Remnant of Memnarch's EDHREC average deck. `--pod-decks 326`.
        PodDeck {
            name: "Urtet, Remnant of Memnarch (BGRUW)",
            commanders: decks::URTET_COMMANDERS,
            main: decks::URTET_MAIN,
        },
        // Seat 327: Kona, Rescue Beastie's EDHREC average deck. `--pod-decks 327`.
        PodDeck {
            name: "Kona, Rescue Beastie (G)",
            commanders: decks::KONA_COMMANDERS,
            main: decks::KONA_MAIN,
        },
        // Seat 328: Marrow-Gnawer's EDHREC average deck. `--pod-decks 328`.
        PodDeck {
            name: "Marrow-Gnawer (B)",
            commanders: decks::MARROW_GNAWER_COMMANDERS,
            main: decks::MARROW_GNAWER_MAIN,
        },
        // Seat 329: Alexios, Deimos of Kosmos's EDHREC average deck. `--pod-decks 329`.
        PodDeck {
            name: "Alexios, Deimos of Kosmos (R)",
            commanders: decks::ALEXIOS_COMMANDERS,
            main: decks::ALEXIOS_MAIN,
        },
        // Seat 330: Raggadragga, Goreguts Boss's EDHREC average deck. `--pod-decks 330`.
        PodDeck {
            name: "Raggadragga, Goreguts Boss (GR)",
            commanders: decks::RAGGADRAGGA_COMMANDERS,
            main: decks::RAGGADRAGGA_MAIN,
        },
        // Seat 331: Athreos, Shroud-Veiled's EDHREC average deck. `--pod-decks 331`.
        PodDeck {
            name: "Athreos, Shroud-Veiled (WB)",
            commanders: decks::ATHREOS_COMMANDERS,
            main: decks::ATHREOS_MAIN,
        },
        // Seat 332: Rakdos, the Muscle's EDHREC average deck. `--pod-decks 332`.
        PodDeck {
            name: "Rakdos, the Muscle (BR)",
            commanders: decks::RAKDOS_MUSCLE_COMMANDERS,
            main: decks::RAKDOS_MUSCLE_MAIN,
        },
        // Seat 333: Captain N'ghathrod's EDHREC average deck (the precon is an
        // earlier seat). `--pod-decks 333`.
        PodDeck {
            name: "Captain N'ghathrod, EDHREC (UB)",
            commanders: decks::NGHATHROD_EDHREC_COMMANDERS,
            main: decks::NGHATHROD_EDHREC_MAIN,
        },
        // Seat 334: Kotis, the Fangkeeper's EDHREC average deck. `--pod-decks 334`.
        PodDeck {
            name: "Kotis, the Fangkeeper (BGU)",
            commanders: decks::KOTIS_COMMANDERS,
            main: decks::KOTIS_MAIN,
        },
        // Seat 335: Henzie "Toolbox" Torre's EDHREC average deck (the
        // Riveteers Rampage precon is an earlier seat). `--pod-decks 335`.
        PodDeck {
            name: "Henzie \"Toolbox\" Torre, EDHREC (BRG)",
            commanders: decks::HENZIE_EDHREC_COMMANDERS,
            main: decks::HENZIE_EDHREC_MAIN,
        },
        // Seat 336: Frodo + Sam's EDHREC average deck (the Food and
        // Fellowship precon is an earlier seat). `--pod-decks 336`.
        PodDeck {
            name: "Frodo + Sam, EDHREC (WBG)",
            commanders: decks::FRODO_EDHREC_COMMANDERS,
            main: decks::FRODO_EDHREC_MAIN,
        },
        // Seat 337: K'rrik, Son of Yawgmoth's EDHREC average deck (mono-B).
        // `--pod-decks 337`.
        PodDeck {
            name: "K'rrik, Son of Yawgmoth, EDHREC (B)",
            commanders: decks::KRRIK_EDHREC_COMMANDERS,
            main: decks::KRRIK_EDHREC_MAIN,
        },
        // Seat 338: Voja, Jaws of the Conclave's EDHREC average deck.
        // `--pod-decks 338`.
        PodDeck {
            name: "Voja, Jaws of the Conclave (GRW)",
            commanders: decks::VOJA_EDHREC_COMMANDERS,
            main: decks::VOJA_EDHREC_MAIN,
        },
        // Seat 339: Choco, Seeker of Paradise's EDHREC average deck.
        // `--pod-decks 339`.
        PodDeck {
            name: "Choco, Seeker of Paradise (GWU)",
            commanders: decks::CHOCO_EDHREC_COMMANDERS,
            main: decks::CHOCO_EDHREC_MAIN,
        },
        // Seat 340: Light-Paws, Emperor's Voice's EDHREC average deck.
        // `--pod-decks 340`.
        PodDeck {
            name: "Light-Paws, Emperor's Voice (W)",
            commanders: decks::LIGHT_PAWS_EDHREC_COMMANDERS,
            main: decks::LIGHT_PAWS_EDHREC_MAIN,
        },
        // Seat 341: Yurlok of Scorch Thrash's EDHREC average deck.
        // `--pod-decks 341`.
        PodDeck {
            name: "Yurlok of Scorch Thrash (BRG)",
            commanders: decks::YURLOK_EDHREC_COMMANDERS,
            main: decks::YURLOK_EDHREC_MAIN,
        },
        // Seat 342: Rocco, Street Chef's EDHREC average deck.
        // `--pod-decks 342`.
        PodDeck {
            name: "Rocco, Street Chef (GRW)",
            commanders: decks::ROCCO_EDHREC_COMMANDERS,
            main: decks::ROCCO_EDHREC_MAIN,
        },
        // Seat 343: Tinybones, Bauble Burglar's EDHREC average deck.
        // `--pod-decks 343`.
        PodDeck {
            name: "Tinybones, Bauble Burglar (B)",
            commanders: decks::TINYBONES_EDHREC_COMMANDERS,
            main: decks::TINYBONES_EDHREC_MAIN,
        },
        // Seat 344: Indominus Rex, Alpha's EDHREC average deck.
        // `--pod-decks 344`.
        PodDeck {
            name: "Indominus Rex, Alpha (BGU)",
            commanders: decks::INDOMINUS_EDHREC_COMMANDERS,
            main: decks::INDOMINUS_EDHREC_MAIN,
        },
        // Seat 345: Tergrid, God of Fright's EDHREC average deck.
        // `--pod-decks 345`.
        PodDeck {
            name: "Tergrid, God of Fright (B)",
            commanders: decks::TERGRID_EDHREC_COMMANDERS,
            main: decks::TERGRID_EDHREC_MAIN,
        },
        // Seat 346: Athreos, God of Passage's EDHREC average deck (Athreos,
        // Shroud-Veiled is seat 331). `--pod-decks 346`.
        PodDeck {
            name: "Athreos, God of Passage (WB)",
            commanders: decks::ATHREOS_GOP_EDHREC_COMMANDERS,
            main: decks::ATHREOS_GOP_EDHREC_MAIN,
        },
        // Seat 347: Ketramose, the New Dawn's EDHREC average deck.
        // `--pod-decks 347`.
        PodDeck {
            name: "Ketramose, the New Dawn (WB)",
            commanders: decks::KETRAMOSE_EDHREC_COMMANDERS,
            main: decks::KETRAMOSE_EDHREC_MAIN,
        },
        // Seat 348: Niko, Light of Hope's EDHREC average deck. `--pod-decks 348`.
        PodDeck {
            name: "Niko, Light of Hope (WU)",
            commanders: decks::NIKO_EDHREC_COMMANDERS,
            main: decks::NIKO_EDHREC_MAIN,
        },
        // Seat 349: Syr Gwyn, Hero of Ashvale's EDHREC average deck.
        // `--pod-decks 349`.
        PodDeck {
            name: "Syr Gwyn, Hero of Ashvale (RWB)",
            commanders: decks::SYR_GWYN_EDHREC_COMMANDERS,
            main: decks::SYR_GWYN_EDHREC_MAIN,
        },
        // Seat 350: Kastral, the Windcrested's EDHREC average deck.
        // `--pod-decks 350`.
        PodDeck {
            name: "Kastral, the Windcrested (WU)",
            commanders: decks::KASTRAL_EDHREC_COMMANDERS,
            main: decks::KASTRAL_EDHREC_MAIN,
        },
        // Seat 351: Jodah, Archmage Eternal's EDHREC average deck.
        // `--pod-decks 351`.
        PodDeck {
            name: "Jodah, Archmage Eternal (WUBRG)",
            commanders: decks::JODAH_EDHREC_COMMANDERS,
            main: decks::JODAH_EDHREC_MAIN,
        },
        // Seat 352: Child of Alara's EDHREC average deck. `--pod-decks 352`.
        PodDeck {
            name: "Child of Alara (WUBRG)",
            commanders: decks::CHILD_OF_ALARA_EDHREC_COMMANDERS,
            main: decks::CHILD_OF_ALARA_EDHREC_MAIN,
        },
        // Seat 353: Rakdos, Lord of Riots's EDHREC average deck (Rakdos, the
        // Muscle is seat 332). `--pod-decks 353`.
        PodDeck {
            name: "Rakdos, Lord of Riots (BR)",
            commanders: decks::RAKDOS_RIOTS_EDHREC_COMMANDERS,
            main: decks::RAKDOS_RIOTS_EDHREC_MAIN,
        },
        // Seat 354: Myrel, Shield of Argive's EDHREC average deck.
        // `--pod-decks 354`.
        PodDeck {
            name: "Myrel, Shield of Argive (W)",
            commanders: decks::MYREL_EDHREC_COMMANDERS,
            main: decks::MYREL_EDHREC_MAIN,
        },
        // Seat 355: Be'lakor, the Dark Master's EDHREC average deck.
        // `--pod-decks 355`.
        PodDeck {
            name: "Be'lakor, the Dark Master (UBR)",
            commanders: decks::BELAKOR_EDHREC_COMMANDERS,
            main: decks::BELAKOR_EDHREC_MAIN,
        },
        // Seat 356: Anti-Venom, Horrifying Healer's EDHREC average deck.
        // `--pod-decks 356`.
        PodDeck {
            name: "Anti-Venom, Horrifying Healer (W)",
            commanders: decks::ANTI_VENOM_EDHREC_COMMANDERS,
            main: decks::ANTI_VENOM_EDHREC_MAIN,
        },
        // Seat 357: Fire Lord Zuko's EDHREC average deck. `--pod-decks 357`.
        PodDeck {
            name: "Fire Lord Zuko (RWB)",
            commanders: decks::FIRE_LORD_ZUKO_COMMANDERS,
            main: decks::FIRE_LORD_ZUKO_MAIN,
        },
        // Seat 358: Arna Kennerüd, Skycaptain's EDHREC average deck. `--pod-decks 358`.
        PodDeck {
            name: "Arna Kennerüd, Skycaptain (WUB)",
            commanders: decks::ARNA_KENNERUD_COMMANDERS,
            main: decks::ARNA_KENNERUD_MAIN,
        },
        // Seat 359: Terra, Magical Adept's EDHREC average deck. `--pod-decks 359`.
        PodDeck {
            name: "Terra, Magical Adept (WUBRG)",
            commanders: decks::TERRA_ADEPT_COMMANDERS,
            main: decks::TERRA_ADEPT_MAIN,
        },
        // Seat 360: Atreus, Impulsive Son's EDHREC average deck. `--pod-decks 360`.
        PodDeck {
            name: "Atreus, Impulsive Son + Kratos, Stoic Father (URW)",
            commanders: decks::ATREUS_KRATOS_COMMANDERS,
            main: decks::ATREUS_KRATOS_MAIN,
        },
        // Seat 361: Kratos, God of War's EDHREC average deck. `--pod-decks 361`.
        PodDeck {
            name: "Kratos, God of War (R)",
            commanders: decks::KRATOS_GOD_OF_WAR_COMMANDERS,
            main: decks::KRATOS_GOD_OF_WAR_MAIN,
        },
        // Seat 362: Eddie Brock's EDHREC average deck. `--pod-decks 362`.
        PodDeck {
            name: "Eddie Brock (BRG)",
            commanders: decks::EDDIE_BROCK_COMMANDERS,
            main: decks::EDDIE_BROCK_MAIN,
        },
        // Seat 363: Nicol Bolas, the Ravager's EDHREC average deck. `--pod-decks 363`.
        PodDeck {
            name: "Nicol Bolas, the Ravager (UBR)",
            commanders: decks::BOLAS_RAVAGER_COMMANDERS,
            main: decks::BOLAS_RAVAGER_MAIN,
        },
        // Seat 364: Shilgengar, Sire of Famine's EDHREC average deck. `--pod-decks 364`.
        PodDeck {
            name: "Shilgengar, Sire of Famine (WB)",
            commanders: decks::SHILGENGAR_COMMANDERS,
            main: decks::SHILGENGAR_MAIN,
        },
        // Seat 365: Norin the Wary's EDHREC average deck. `--pod-decks 365`.
        PodDeck {
            name: "Norin the Wary (R)",
            commanders: decks::NORIN_COMMANDERS,
            main: decks::NORIN_MAIN,
        },
        // Seat 366: Katara, the Fearless's EDHREC average deck. `--pod-decks 366`.
        PodDeck {
            name: "Katara, the Fearless (GWU)",
            commanders: decks::KATARA_THE_FEARLESS_COMMANDERS,
            main: decks::KATARA_THE_FEARLESS_MAIN,
        },
        // Seat 367: Azlask, the Swelling Scourge's EDHREC average deck. `--pod-decks 367`.
        PodDeck {
            name: "Azlask, the Swelling Scourge (WUBRG)",
            commanders: decks::AZLASK_COMMANDERS,
            main: decks::AZLASK_MAIN,
        },
        // Seat 368: The Mycotyrant's EDHREC average deck. `--pod-decks 368`.
        PodDeck {
            name: "The Mycotyrant (BG)",
            commanders: decks::THE_MYCOTYRANT_COMMANDERS,
            main: decks::THE_MYCOTYRANT_MAIN,
        },
        // Seat 369: Shroofus Sproutsire's EDHREC average deck. `--pod-decks 369`.
        PodDeck {
            name: "Shroofus Sproutsire (G)",
            commanders: decks::SHROOFUS_COMMANDERS,
            main: decks::SHROOFUS_MAIN,
        },
        // Seat 370: Miirym, Sentinel Wyrm's EDHREC average deck. `--pod-decks 370`.
        PodDeck {
            name: "Miirym, Sentinel Wyrm (GUR)",
            commanders: decks::MIIRYM_COMMANDERS,
            main: decks::MIIRYM_MAIN,
        },
        // Seat 371: Tiamat's EDHREC average deck. `--pod-decks 371`.
        PodDeck {
            name: "Tiamat (WUBRG)",
            commanders: decks::TIAMAT_COMMANDERS,
            main: decks::TIAMAT_MAIN,
        },
        // Seat 372: Thranduil, the Elvenking's EDHREC average deck. `--pod-decks 372`.
        PodDeck {
            name: "Thranduil, the Elvenking (BGU)",
            commanders: decks::THRANDUIL_COMMANDERS,
            main: decks::THRANDUIL_MAIN,
        },
        // Seat 373: Maralen, Fae Ascendant's EDHREC average deck. `--pod-decks 373`.
        PodDeck {
            name: "Maralen, Fae Ascendant (BGU)",
            commanders: decks::MARALEN_COMMANDERS,
            main: decks::MARALEN_MAIN,
        },
        // Seat 374: Najeela, the Blade-Blossom's EDHREC average deck. `--pod-decks 374`.
        PodDeck {
            name: "Najeela, the Blade-Blossom (WUBRG)",
            commanders: decks::NAJEELA_COMMANDERS,
            main: decks::NAJEELA_MAIN,
        },
        // Seat 375: Toph, the First Metalbender's EDHREC average deck. `--pod-decks 375`.
        PodDeck {
            name: "Toph, the First Metalbender (RGW)",
            commanders: decks::TOPH_FIRST_METALBENDER_COMMANDERS,
            main: decks::TOPH_FIRST_METALBENDER_MAIN,
        },
        // Seat 376: Wolverine, Best There Is's EDHREC average deck. `--pod-decks 376`.
        PodDeck {
            name: "Wolverine, Best There Is (RG)",
            commanders: decks::WOLVERINE_COMMANDERS,
            main: decks::WOLVERINE_MAIN,
        },
        // Seat 377: Captain Howler, Sea Scourge's EDHREC average deck. `--pod-decks 377`.
        PodDeck {
            name: "Captain Howler, Sea Scourge (UR)",
            commanders: decks::HOWLER_COMMANDERS,
            main: decks::HOWLER_MAIN,
        },
        // Seat 378: Magda, Brazen Outlaw's EDHREC average deck. `--pod-decks 378`.
        PodDeck {
            name: "Magda, Brazen Outlaw (R)",
            commanders: decks::MAGDA_COMMANDERS,
            main: decks::MAGDA_MAIN,
        },
        // Seat 379: Avatar Aang's EDHREC average deck. `--pod-decks 379`.
        PodDeck {
            name: "Avatar Aang (WUBRG)",
            commanders: decks::AVATAR_AANG_COMMANDERS,
            main: decks::AVATAR_AANG_MAIN,
        },
        // Seat 380: Lorehold, the Historian's EDHREC average deck. `--pod-decks 380`.
        PodDeck {
            name: "Lorehold, the Historian (RW)",
            commanders: decks::LOREHOLD_HISTORIAN_COMMANDERS,
            main: decks::LOREHOLD_HISTORIAN_MAIN,
        },
        // Seat 381: High Perfect Morcant's EDHREC average deck. `--pod-decks 381`.
        PodDeck {
            name: "High Perfect Morcant (BG)",
            commanders: decks::HIGH_PERFECT_MORCANT_COMMANDERS,
            main: decks::HIGH_PERFECT_MORCANT_MAIN,
        },
        // Seat 382: Iroh, Grand Lotus's EDHREC average deck. `--pod-decks 382`.
        PodDeck {
            name: "Iroh, Grand Lotus (URG)",
            commanders: decks::IROH_GRAND_LOTUS_COMMANDERS,
            main: decks::IROH_GRAND_LOTUS_MAIN,
        },
        // Seat 383: Galadriel, Light of Valinor's EDHREC average deck. `--pod-decks 383`.
        PodDeck {
            name: "Galadriel, Light of Valinor (GWU)",
            commanders: decks::GALADRIEL_LIGHT_OF_VALINOR_COMMANDERS,
            main: decks::GALADRIEL_LIGHT_OF_VALINOR_MAIN,
        },
        // Seat 384: Skullbriar, the Walking Grave's EDHREC average deck. `--pod-decks 384`.
        PodDeck {
            name: "Skullbriar, the Walking Grave (BG)",
            commanders: decks::SKULLBRIAR_COMMANDERS,
            main: decks::SKULLBRIAR_MAIN,
        },
        // Seat 385: Aragorn, the Uniter's EDHREC average deck. `--pod-decks 385`.
        PodDeck {
            name: "Aragorn, the Uniter (WURG)",
            commanders: decks::ARAGORN_THE_UNITER_COMMANDERS,
            main: decks::ARAGORN_THE_UNITER_MAIN,
        },
        // Seat 386: Ragost, Deft Gastronaut's EDHREC average deck. `--pod-decks 386`.
        PodDeck {
            name: "Ragost, Deft Gastronaut (RW)",
            commanders: decks::RAGOST_COMMANDERS,
            main: decks::RAGOST_MAIN,
        },
        // Seat 387: Iron Man, Titan of Innovation's EDHREC average deck. `--pod-decks 387`.
        PodDeck {
            name: "Iron Man, Titan of Innovation (UR)",
            commanders: decks::IRON_MAN_TITAN_COMMANDERS,
            main: decks::IRON_MAN_TITAN_MAIN,
        },
        // Seat 388: Jon Irenicus, Shattered One's EDHREC average deck. `--pod-decks 388`.
        PodDeck {
            name: "Jon Irenicus, Shattered One (UB)",
            commanders: decks::JON_IRENICUS_COMMANDERS,
            main: decks::JON_IRENICUS_MAIN,
        },
        // Seat 389: Obeka, Splitter of Seconds's EDHREC average deck. `--pod-decks 389`.
        PodDeck {
            name: "Obeka, Splitter of Seconds (UBR)",
            commanders: decks::OBEKA_COMMANDERS,
            main: decks::OBEKA_MAIN,
        },
        // Seat 390: Captain America, First Avenger's EDHREC average deck. `--pod-decks 390`.
        PodDeck {
            name: "Captain America, First Avenger (WUR)",
            commanders: decks::CAPTAIN_AMERICA_FIRST_COMMANDERS,
            main: decks::CAPTAIN_AMERICA_FIRST_MAIN,
        },
        // Seat 391: Doran, Besieged by Time's EDHREC average deck. `--pod-decks 391`.
        PodDeck {
            name: "Doran, Besieged by Time (WBG)",
            commanders: decks::DORAN_BESIEGED_COMMANDERS,
            main: decks::DORAN_BESIEGED_MAIN,
        },
        // Seat 392: Lord of the Nazgûl's EDHREC average deck. `--pod-decks 392`.
        PodDeck {
            name: "Lord of the Nazgûl (UB)",
            commanders: decks::LORD_OF_THE_NAZGUL_COMMANDERS,
            main: decks::LORD_OF_THE_NAZGUL_MAIN,
        },
        // Seat 393: Leonardo, the Balance // Michelangelo, the Heart's EDHREC
        // average deck (Partner—Character select). `--pod-decks 393`.
        PodDeck {
            name: "Leonardo, the Balance // Michelangelo, the Heart (WUBRG)",
            commanders: decks::LEONARDO_MICHELANGELO_COMMANDERS,
            main: decks::LEONARDO_MICHELANGELO_MAIN,
        },
        // Seat 394: Norman Osborn's EDHREC average deck. `--pod-decks 394`.
        PodDeck {
            name: "Norman Osborn (UBR)",
            commanders: decks::NORMAN_OSBORN_COMMANDERS,
            main: decks::NORMAN_OSBORN_MAIN,
        },
        // Seat 395: Reaper King's EDHREC average deck. `--pod-decks 395`.
        PodDeck {
            name: "Reaper King (WUBRG)",
            commanders: decks::REAPER_KING_COMMANDERS,
            main: decks::REAPER_KING_MAIN,
        },
        // Seat 396: Sauron, the Dark Lord's EDHREC average deck. `--pod-decks 396`.
        PodDeck {
            name: "Sauron, the Dark Lord (UBR)",
            commanders: decks::SAURON_DARK_LORD_COMMANDERS,
            main: decks::SAURON_DARK_LORD_MAIN,
        },
        // Seat 397: Saruman, the White Hand's EDHREC average deck. `--pod-decks 397`.
        PodDeck {
            name: "Saruman, the White Hand (UBR)",
            commanders: decks::SARUMAN_WHITE_HAND_COMMANDERS,
            main: decks::SARUMAN_WHITE_HAND_MAIN,
        },
        // Seat 398: Kibo, Uktabi Prince's EDHREC average deck. `--pod-decks 398`.
        PodDeck {
            name: "Kibo, Uktabi Prince (RG)",
            commanders: decks::KIBO_COMMANDERS,
            main: decks::KIBO_MAIN,
        },
        // Seat 399: Shelob, Child of Ungoliant's EDHREC average deck. `--pod-decks 399`.
        PodDeck {
            name: "Shelob, Child of Ungoliant (BG)",
            commanders: decks::SHELOB_COMMANDERS,
            main: decks::SHELOB_MAIN,
        },
        // Seat 400: Marvo, Deep Operative's EDHREC average deck. `--pod-decks 400`.
        PodDeck {
            name: "Marvo, Deep Operative (UB)",
            commanders: decks::MARVO_COMMANDERS,
            main: decks::MARVO_MAIN,
        },
        // Seat 401: Tom Bombadil's EDHREC average deck. `--pod-decks 401`.
        PodDeck {
            name: "Tom Bombadil (WUBRG)",
            commanders: decks::TOM_BOMBADIL_COMMANDERS,
            main: decks::TOM_BOMBADIL_MAIN,
        },
        // Seat 402: Edward Kenway's EDHREC average deck. `--pod-decks 402`.
        PodDeck {
            name: "Edward Kenway (UBR)",
            commanders: decks::EDWARD_KENWAY_COMMANDERS,
            main: decks::EDWARD_KENWAY_MAIN,
        },
        // Seat 403: Hylda of the Icy Crown's EDHREC average deck. `--pod-decks 403`.
        PodDeck {
            name: "Hylda of the Icy Crown (WU)",
            commanders: decks::HYLDA_OF_THE_ICY_CROWN_COMMANDERS,
            main: decks::HYLDA_OF_THE_ICY_CROWN_MAIN,
        },
        // Seat 404: Mr. House, President and CEO's EDHREC average deck. `--pod-decks 404`.
        PodDeck {
            name: "Mr. House, President and CEO (RWB)",
            commanders: decks::MR_HOUSE_COMMANDERS,
            main: decks::MR_HOUSE_MAIN,
        },
        // Seat 405: Ezio Auditore da Firenze's EDHREC average deck. `--pod-decks 405`.
        PodDeck {
            name: "Ezio Auditore da Firenze (WUBRG)",
            commanders: decks::EZIO_COMMANDERS,
            main: decks::EZIO_MAIN,
        },
        // Seat 406: Tovolar, Dire Overlord's EDHREC average deck. `--pod-decks 406`.
        PodDeck {
            name: "Tovolar, Dire Overlord (RG)",
            commanders: decks::TOVOLAR_COMMANDERS,
            main: decks::TOVOLAR_MAIN,
        },
        // Seat 407: Xenagos, God of Revels's EDHREC average deck. `--pod-decks 407`.
        PodDeck {
            name: "Xenagos, God of Revels (RG)",
            commanders: decks::XENAGOS_COMMANDERS,
            main: decks::XENAGOS_MAIN,
        },
        // Seat 408: Kess, Dissident Mage's EDHREC average deck. `--pod-decks 408`.
        PodDeck {
            name: "Kess, Dissident Mage (UBR)",
            commanders: decks::KESS_COMMANDERS,
            main: decks::KESS_MAIN,
        },
        // Seat 409: Vnwxt, Verbose Host's EDHREC average deck. `--pod-decks 409`.
        PodDeck {
            name: "Vnwxt, Verbose Host (U)",
            commanders: decks::VNWXT_COMMANDERS,
            main: decks::VNWXT_MAIN,
        },
        // Seat 410: Szarel, Genesis Shepherd's EDHREC average deck. `--pod-decks 410`.
        PodDeck {
            name: "Szarel, Genesis Shepherd (BRG)",
            commanders: decks::SZAREL_COMMANDERS,
            main: decks::SZAREL_MAIN,
        },
        // Seat 411: Winter, Misanthropic Guide's EDHREC average deck. `--pod-decks 411`.
        PodDeck {
            name: "Winter, Misanthropic Guide (BRG)",
            commanders: decks::WINTER_411_COMMANDERS,
            main: decks::WINTER_411_MAIN,
        },
        // Seat 412: Niv-Mizzet, Visionary's EDHREC average deck. `--pod-decks 412`.
        PodDeck {
            name: "Niv-Mizzet, Visionary (UR)",
            commanders: decks::NIV_MIZZET_COMMANDERS,
            main: decks::NIV_MIZZET_MAIN,
        },
        // Seat 413: Bristly Bill, Spine Sower's EDHREC average deck. `--pod-decks 413`.
        PodDeck {
            name: "Bristly Bill, Spine Sower (G)",
            commanders: decks::BRISTLY_BILL_COMMANDERS,
            main: decks::BRISTLY_BILL_MAIN,
        },
        // Seat 414: Talion, the Kindly Lord's EDHREC average deck. `--pod-decks 414`.
        PodDeck {
            name: "Talion, the Kindly Lord (UB)",
            commanders: decks::TALION_COMMANDERS,
            main: decks::TALION_MAIN,
        },
        // Seat 415: Tifa, Martial Artist's EDHREC average deck. `--pod-decks 415`.
        PodDeck {
            name: "Tifa, Martial Artist (WRG)",
            commanders: decks::TIFA_415_COMMANDERS,
            main: decks::TIFA_415_MAIN,
        },
        // Seat 416: Lathiel, the Bounteous Dawn's EDHREC average deck. `--pod-decks 416`.
        PodDeck {
            name: "Lathiel, the Bounteous Dawn (WG)",
            commanders: decks::LATHIEL_COMMANDERS,
            main: decks::LATHIEL_MAIN,
        },
        // Seat 417: Koma, Cosmos Serpent's EDHREC average deck. `--pod-decks 417`.
        PodDeck {
            name: "Koma, Cosmos Serpent (UG)",
            commanders: decks::KOMA_COMMANDERS,
            main: decks::KOMA_MAIN,
        },
        // Seat 418: Bilbo, Birthday Celebrant's EDHREC average deck. `--pod-decks 418`.
        PodDeck {
            name: "Bilbo, Birthday Celebrant (WBG)",
            commanders: decks::BILBO_COMMANDERS,
            main: decks::BILBO_MAIN,
        },
        // Seat 419: Bruvac the Grandiloquent's EDHREC average deck. `--pod-decks 419`.
        PodDeck {
            name: "Bruvac the Grandiloquent (U)",
            commanders: decks::BRUVAC_THE_GRANDILOQUENT_COMMANDERS,
            main: decks::BRUVAC_THE_GRANDILOQUENT_MAIN,
        },
        // Seat 420: Ayara, First of Locthwain's EDHREC average deck. `--pod-decks 420`.
        PodDeck {
            name: "Ayara, First of Locthwain (B)",
            commanders: decks::AYARA_COMMANDERS,
            main: decks::AYARA_MAIN,
        },
        // Seat 421: Zurgo, Thunder's Decree's EDHREC average deck. `--pod-decks 421`.
        PodDeck {
            name: "Zurgo, Thunder's Decree (WBR)",
            commanders: decks::ZURGO_421_COMMANDERS,
            main: decks::ZURGO_421_MAIN,
        },
        // Seat 422: Laughing Jasper Flint's EDHREC average deck. `--pod-decks 422`.
        PodDeck {
            name: "Laughing Jasper Flint (BR)",
            commanders: decks::LAUGHING_JASPER_FLINT_COMMANDERS,
            main: decks::LAUGHING_JASPER_FLINT_MAIN,
        },
        // Seat 423: Jenova, Ancient Calamity's EDHREC average deck. `--pod-decks 423`.
        PodDeck {
            name: "Jenova, Ancient Calamity (BG)",
            commanders: decks::JENOVA_COMMANDERS,
            main: decks::JENOVA_MAIN,
        },
        // Seat 424: The Lord of Pain's EDHREC average deck. `--pod-decks 424`.
        PodDeck {
            name: "The Lord of Pain (BR)",
            commanders: decks::THE_LORD_OF_PAIN_COMMANDERS,
            main: decks::THE_LORD_OF_PAIN_MAIN,
        },
        // Seat 425: Marisi, Breaker of the Coil's EDHREC average deck. `--pod-decks 425`.
        PodDeck {
            name: "Marisi, Breaker of the Coil (WRG)",
            commanders: decks::MARISI_COMMANDERS,
            main: decks::MARISI_MAIN,
        },
        // Seat 426: Elenda, the Dusk Rose's EDHREC average deck. `--pod-decks 426`.
        PodDeck {
            name: "Elenda, the Dusk Rose (WB)",
            commanders: decks::ELENDA_COMMANDERS,
            main: decks::ELENDA_MAIN,
        },
        // Seat 427: Emry, Lurker of the Loch's EDHREC average deck. `--pod-decks 427`.
        PodDeck {
            name: "Emry, Lurker of the Loch (U)",
            commanders: decks::EMRY_COMMANDERS,
            main: decks::EMRY_MAIN,
        },
        // Seat 428: Felothar, Dawn of the Abzan's EDHREC average deck. `--pod-decks 428`.
        PodDeck {
            name: "Felothar, Dawn of the Abzan (WBG)",
            commanders: decks::FELOTHAR_428_COMMANDERS,
            main: decks::FELOTHAR_428_MAIN,
        },
        // Seat 429: Sin, Spira's Punishment's EDHREC average deck. `--pod-decks 429`.
        PodDeck {
            name: "Sin, Spira's Punishment (UBG)",
            commanders: decks::SIN_COMMANDERS,
            main: decks::SIN_MAIN,
        },
        // Seat 430: Mr. Foxglove's EDHREC average deck. `--pod-decks 430`.
        PodDeck {
            name: "Mr. Foxglove (WUG)",
            commanders: decks::MR_FOXGLOVE_COMMANDERS,
            main: decks::MR_FOXGLOVE_MAIN,
        },
        // Seat 431: Tasigur, the Golden Fang's EDHREC average deck. `--pod-decks 431`.
        PodDeck {
            name: "Tasigur, the Golden Fang (UBG)",
            commanders: decks::TASIGUR_COMMANDERS,
            main: decks::TASIGUR_MAIN,
        },
        // Seat 432: Loot, the Pathfinder's EDHREC average deck. `--pod-decks 432`.
        PodDeck {
            name: "Loot, the Pathfinder (URG)",
            commanders: decks::LOOT_COMMANDERS,
            main: decks::LOOT_MAIN,
        },
        // Seat 433: Jinnie Fay, Jetmir's Second's EDHREC average deck. `--pod-decks 433`.
        PodDeck {
            name: "Jinnie Fay, Jetmir's Second (WRG)",
            commanders: decks::JINNIE_FAY_COMMANDERS,
            main: decks::JINNIE_FAY_MAIN,
        },
        // Seat 434: Loot, Exuberant Explorer's EDHREC average deck. `--pod-decks 434`.
        PodDeck {
            name: "Loot, Exuberant Explorer (G)",
            commanders: decks::LOOT_434_COMMANDERS,
            main: decks::LOOT_434_MAIN,
        },
        // Seat 435: Scion of the Ur-Dragon's EDHREC average deck. `--pod-decks 435`.
        PodDeck {
            name: "Scion of the Ur-Dragon (WUBRG)",
            commanders: decks::SCION_OF_THE_UR_DRAGON_COMMANDERS,
            main: decks::SCION_OF_THE_UR_DRAGON_MAIN,
        },
        // Seat 436: Rhys the Redeemed's EDHREC average deck. `--pod-decks 436`.
        PodDeck {
            name: "Rhys the Redeemed (WG)",
            commanders: decks::RHYS_THE_REDEEMED_COMMANDERS,
            main: decks::RHYS_THE_REDEEMED_MAIN,
        },
        // Seat 437: Jarad, Golgari Lich Lord's EDHREC average deck. `--pod-decks 437`.
        PodDeck {
            name: "Jarad, Golgari Lich Lord (BG)",
            commanders: decks::JARAD_COMMANDERS,
            main: decks::JARAD_MAIN,
        },
        // Seat 438: Alesha, Who Smiles at Death's EDHREC average deck. `--pod-decks 438`.
        PodDeck {
            name: "Alesha, Who Smiles at Death (WBR)",
            commanders: decks::ALESHA_COMMANDERS,
            main: decks::ALESHA_MAIN,
        },
        // Seat 439: Quandrix, the Proof's EDHREC average deck. `--pod-decks 439`.
        PodDeck {
            name: "Quandrix, the Proof (UG)",
            commanders: decks::QUANDRIX_COMMANDERS,
            main: decks::QUANDRIX_MAIN,
        },
        // Seat 440: Yuna, Hope of Spira's EDHREC average deck. `--pod-decks 440`.
        PodDeck {
            name: "Yuna, Hope of Spira (WG)",
            commanders: decks::YUNA_440_COMMANDERS,
            main: decks::YUNA_440_MAIN,
        },
        // Seat 441: Goreclaw, Terror of Qal Sisma's EDHREC average deck. `--pod-decks 441`.
        PodDeck {
            name: "Goreclaw, Terror of Qal Sisma (G)",
            commanders: decks::GORECLAW_COMMANDERS,
            main: decks::GORECLAW_MAIN,
        },
        // Seat 442: Garth One-Eye's EDHREC average deck. `--pod-decks 442`.
        PodDeck {
            name: "Garth One-Eye (WUBRG)",
            commanders: decks::GARTH_ONE_EYE_COMMANDERS,
            main: decks::GARTH_ONE_EYE_MAIN,
        },
        // Seat 443: Rat King, Verminister's EDHREC average deck. `--pod-decks 443`.
        PodDeck {
            name: "Rat King, Verminister (B)",
            commanders: decks::RAT_KING_COMMANDERS,
            main: decks::RAT_KING_MAIN,
        },
        // Seat 444: Liberty Prime, Recharged's EDHREC average deck. `--pod-decks 444`.
        PodDeck {
            name: "Liberty Prime, Recharged (WUR)",
            commanders: decks::LIBERTY_PRIME_COMMANDERS,
            main: decks::LIBERTY_PRIME_MAIN,
        },
        // Seat 445: Samut, the Driving Force's EDHREC average deck. `--pod-decks 445`.
        PodDeck {
            name: "Samut, the Driving Force (WRG)",
            commanders: decks::SAMUT_COMMANDERS,
            main: decks::SAMUT_MAIN,
        },
        // Seat 446: Black Waltz No. 3's EDHREC average deck. `--pod-decks 446`.
        PodDeck {
            name: "Black Waltz No. 3 (BR)",
            commanders: decks::BLACK_WALTZ_NO_3_COMMANDERS,
            main: decks::BLACK_WALTZ_NO_3_MAIN,
        },
        // Seat 447: Otharri, Suns' Glory's EDHREC average deck. `--pod-decks 447`.
        PodDeck {
            name: "Otharri, Suns' Glory (WR)",
            commanders: decks::OTHARRI_COMMANDERS,
            main: decks::OTHARRI_MAIN,
        },
        // Seat 448: Vito, Thorn of the Dusk Rose's EDHREC average deck. `--pod-decks 448`.
        PodDeck {
            name: "Vito, Thorn of the Dusk Rose (B)",
            commanders: decks::VITO_COMMANDERS,
            main: decks::VITO_MAIN,
        },
        // Seat 449: Cloud, Midgar Mercenary's EDHREC average deck. `--pod-decks 449`.
        PodDeck {
            name: "Cloud, Midgar Mercenary (W)",
            commanders: decks::CLOUD_COMMANDERS,
            main: decks::CLOUD_MAIN,
        },
        // Seat 450: Elas il-Kor, Sadistic Pilgrim's EDHREC average deck. `--pod-decks 450`.
        PodDeck {
            name: "Elas il-Kor, Sadistic Pilgrim (WB)",
            commanders: decks::ELAS_IL_KOR_COMMANDERS,
            main: decks::ELAS_IL_KOR_MAIN,
        },
        // Seat 451: Braids, Arisen Nightmare's EDHREC average deck. `--pod-decks 451`.
        PodDeck {
            name: "Braids, Arisen Nightmare (B)",
            commanders: decks::BRAIDS_COMMANDERS,
            main: decks::BRAIDS_MAIN,
        },
        // Seat 452: Haldan, Avid Arcanist // Pako, Arcane Retriever's EDHREC average deck. `--pod-decks 452`.
        PodDeck {
            name: "Haldan, Avid Arcanist // Pako, Arcane Retriever (URG)",
            commanders: decks::HALDAN_COMMANDERS,
            main: decks::HALDAN_MAIN,
        },
        // Seat 453: The Master of Keys's EDHREC average deck. `--pod-decks 453`.
        PodDeck {
            name: "The Master of Keys (WUB)",
            commanders: decks::THE_MASTER_OF_KEYS_COMMANDERS,
            main: decks::THE_MASTER_OF_KEYS_MAIN,
        },
        // Seat 454: The Unbeatable Squirrel Girl's EDHREC average deck. `--pod-decks 454`.
        PodDeck {
            name: "The Unbeatable Squirrel Girl (G)",
            commanders: decks::THE_UNBEATABLE_SQUIRREL_GIRL_COMMANDERS,
            main: decks::THE_UNBEATABLE_SQUIRREL_GIRL_MAIN,
        },
        // Seat 455: Squall, SeeD Mercenary's EDHREC average deck. `--pod-decks 455`.
        PodDeck {
            name: "Squall, SeeD Mercenary (WB)",
            commanders: decks::SQUALL_COMMANDERS,
            main: decks::SQUALL_MAIN,
        },
        // Seat 456: Kellan, the Kid's EDHREC average deck. `--pod-decks 456`.
        PodDeck {
            name: "Kellan, the Kid (WUG)",
            commanders: decks::KELLAN_COMMANDERS,
            main: decks::KELLAN_MAIN,
        },
        // Seat 457: Tuvasa the Sunlit's EDHREC average deck. `--pod-decks 457`.
        PodDeck {
            name: "Tuvasa the Sunlit (WUG)",
            commanders: decks::TUVASA_THE_SUNLIT_COMMANDERS,
            main: decks::TUVASA_THE_SUNLIT_MAIN,
        },
        // Seat 458: Sidisi, Brood Tyrant's EDHREC average deck. `--pod-decks 458`.
        PodDeck {
            name: "Sidisi, Brood Tyrant (UBG)",
            commanders: decks::SIDISI_COMMANDERS,
            main: decks::SIDISI_MAIN,
        },
        // Seat 459: Extus, Oriq Overlord's EDHREC average deck. `--pod-decks 459`.
        PodDeck {
            name: "Extus, Oriq Overlord (WBR)",
            commanders: decks::EXTUS_COMMANDERS,
            main: decks::EXTUS_MAIN,
        },
        // Seat 460: Nethroi, Apex of Death's EDHREC average deck. `--pod-decks 460`.
        PodDeck {
            name: "Nethroi, Apex of Death (WBG)",
            commanders: decks::NETHROI_COMMANDERS,
            main: decks::NETHROI_MAIN,
        },
        // Seat 461: Sigarda, Font of Blessings's EDHREC average deck. `--pod-decks 461`.
        PodDeck {
            name: "Sigarda, Font of Blessings (WG)",
            commanders: decks::SIGARDA_461_COMMANDERS,
            main: decks::SIGARDA_461_MAIN,
        },
        // Seat 462: Azusa, Lost but Seeking's EDHREC average deck. `--pod-decks 462`.
        PodDeck {
            name: "Azusa, Lost but Seeking (G)",
            commanders: decks::AZUSA_COMMANDERS,
            main: decks::AZUSA_MAIN,
        },
        // Seat 463: Yawgmoth, Thran Physician's EDHREC average deck. `--pod-decks 463`.
        PodDeck {
            name: "Yawgmoth, Thran Physician (B)",
            commanders: decks::YAWGMOTH_COMMANDERS,
            main: decks::YAWGMOTH_MAIN,
        },
        // Seat 464: Astarion, the Decadent's EDHREC average deck. `--pod-decks 464`.
        PodDeck {
            name: "Astarion, the Decadent (WB)",
            commanders: decks::ASTARION_COMMANDERS,
            main: decks::ASTARION_MAIN,
        },
        // Seat 465: The Locust God's EDHREC average deck. `--pod-decks 465`.
        PodDeck {
            name: "The Locust God (UR)",
            commanders: decks::THE_LOCUST_GOD_COMMANDERS,
            main: decks::THE_LOCUST_GOD_MAIN,
        },
        // Seat 466: Jaws, Relentless Predator's EDHREC average deck. `--pod-decks 466`.
        PodDeck {
            name: "Jaws, Relentless Predator (R)",
            commanders: decks::JAWS_COMMANDERS,
            main: decks::JAWS_MAIN,
        },
        // Seat 467: Don Andres, the Renegade's EDHREC average deck. `--pod-decks 467`.
        PodDeck {
            name: "Don Andres, the Renegade (UBR)",
            commanders: decks::DON_ANDRES_COMMANDERS,
            main: decks::DON_ANDRES_MAIN,
        },
        // Seat 468: Karador, Ghost Chieftain's EDHREC average deck. `--pod-decks 468`.
        PodDeck {
            name: "Karador, Ghost Chieftain (WBG)",
            commanders: decks::KARADOR_COMMANDERS,
            main: decks::KARADOR_MAIN,
        },
        // Seat 469: Delney, Streetwise Lookout's EDHREC average deck. `--pod-decks 469`.
        PodDeck {
            name: "Delney, Streetwise Lookout (W)",
            commanders: decks::DELNEY_COMMANDERS,
            main: decks::DELNEY_MAIN,
        },
        // Seat 470: Sorin of House Markov's EDHREC average deck. `--pod-decks 470`.
        PodDeck {
            name: "Sorin of House Markov (WB)",
            commanders: decks::SORIN_OF_HOUSE_MARKOV_COMMANDERS,
            main: decks::SORIN_OF_HOUSE_MARKOV_MAIN,
        },
        // Seat 471: Teval, Arbiter of Virtue's EDHREC average deck. `--pod-decks 471`.
        PodDeck {
            name: "Teval, Arbiter of Virtue (UBG)",
            commanders: decks::TEVAL_471_COMMANDERS,
            main: decks::TEVAL_471_MAIN,
        },
        // Seat 472: Gisa, the Hellraiser's EDHREC average deck. `--pod-decks 472`.
        PodDeck {
            name: "Gisa, the Hellraiser (B)",
            commanders: decks::GISA_472_COMMANDERS,
            main: decks::GISA_472_MAIN,
        },
        // Seat 473: Felix Five-Boots's EDHREC average deck. `--pod-decks 473`.
        PodDeck {
            name: "Felix Five-Boots (UBG)",
            commanders: decks::FELIX_FIVE_BOOTS_COMMANDERS,
            main: decks::FELIX_FIVE_BOOTS_MAIN,
        },
        // Seat 474: Black Panther, Wakandan King's EDHREC average deck. `--pod-decks 474`.
        PodDeck {
            name: "Black Panther, Wakandan King (WG)",
            commanders: decks::BLACK_PANTHER_COMMANDERS,
            main: decks::BLACK_PANTHER_MAIN,
        },
        // Seat 475: Camellia, the Seedmiser's EDHREC average deck. `--pod-decks 475`.
        PodDeck {
            name: "Camellia, the Seedmiser (BG)",
            commanders: decks::CAMELLIA_COMMANDERS,
            main: decks::CAMELLIA_MAIN,
        },
        // Seat 476: Karlov of the Ghost Council's EDHREC average deck. `--pod-decks 476`.
        PodDeck {
            name: "Karlov of the Ghost Council (WB)",
            commanders: decks::KARLOV_OF_THE_GHOST_COUNCIL_COMMANDERS,
            main: decks::KARLOV_OF_THE_GHOST_COUNCIL_MAIN,
        },
        // Seat 477: Hamza, Guardian of Arashin's EDHREC average deck. `--pod-decks 477`.
        PodDeck {
            name: "Hamza, Guardian of Arashin (WG)",
            commanders: decks::HAMZA_COMMANDERS,
            main: decks::HAMZA_MAIN,
        },
        // Seat 478: Anim Pakal, Thousandth Moon's EDHREC average deck. `--pod-decks 478`.
        PodDeck {
            name: "Anim Pakal, Thousandth Moon (WR)",
            commanders: decks::ANIM_PAKAL_COMMANDERS,
            main: decks::ANIM_PAKAL_MAIN,
        },
        // Seat 479: Sami, Wildcat Captain's EDHREC average deck. `--pod-decks 479`.
        PodDeck {
            name: "Sami, Wildcat Captain (WR)",
            commanders: decks::SAMI_COMMANDERS,
            main: decks::SAMI_MAIN,
        },
        // Seat 480: Jan Jansen, Chaos Crafter's EDHREC average deck. `--pod-decks 480`.
        PodDeck {
            name: "Jan Jansen, Chaos Crafter (WBR)",
            commanders: decks::JAN_JANSEN_COMMANDERS,
            main: decks::JAN_JANSEN_MAIN,
        },
        // Seat 481: Tamiyo, Inquisitive Student's EDHREC average deck. `--pod-decks 481`.
        PodDeck {
            name: "Tamiyo, Inquisitive Student (UG)",
            commanders: decks::TAMIYO_COMMANDERS,
            main: decks::TAMIYO_MAIN,
        },
        // Seat 482: Karona, False God's EDHREC average deck. `--pod-decks 482`.
        PodDeck {
            name: "Karona, False God (WUBRG)",
            commanders: decks::KARONA_COMMANDERS,
            main: decks::KARONA_MAIN,
        },
        // Seat 483: Solphim, Mayhem Dominus's EDHREC average deck. `--pod-decks 483`.
        PodDeck {
            name: "Solphim, Mayhem Dominus (R)",
            commanders: decks::SOLPHIM_COMMANDERS,
            main: decks::SOLPHIM_MAIN,
        },
        // Seat 484: Neera, Wild Mage's EDHREC average deck. `--pod-decks 484`.
        PodDeck {
            name: "Neera, Wild Mage (UR)",
            commanders: decks::NEERA_COMMANDERS,
            main: decks::NEERA_MAIN,
        },
        // Seat 485: Vincent Valentine's EDHREC average deck. `--pod-decks 485`.
        PodDeck {
            name: "Vincent Valentine (B)",
            commanders: decks::VINCENT_VALENTINE_COMMANDERS,
            main: decks::VINCENT_VALENTINE_MAIN,
        },
        // Seat 486: Heliod, Sun-Crowned's EDHREC average deck. `--pod-decks 486`.
        PodDeck {
            name: "Heliod, Sun-Crowned (W)",
            commanders: decks::HELIOD_COMMANDERS,
            main: decks::HELIOD_MAIN,
        },
        // Seat 487: Syr Vondam, Sunstar Exemplar's EDHREC average deck. `--pod-decks 487`.
        PodDeck {
            name: "Syr Vondam, Sunstar Exemplar (WB)",
            commanders: decks::SYR_VONDAM_COMMANDERS,
            main: decks::SYR_VONDAM_MAIN,
        },
        // Seat 488: Ghired, Mirror of the Wilds's EDHREC average deck. `--pod-decks 488`.
        PodDeck {
            name: "Ghired, Mirror of the Wilds (WRG)",
            commanders: decks::GHIRED_488_COMMANDERS,
            main: decks::GHIRED_488_MAIN,
        },
        // Seat 489: Vishgraz, the Doomhive's EDHREC average deck. `--pod-decks 489`.
        PodDeck {
            name: "Vishgraz, the Doomhive (WBG)",
            commanders: decks::VISHGRAZ_COMMANDERS,
            main: decks::VISHGRAZ_MAIN,
        },
        // Seat 490: Herigast, Erupting Nullkite's EDHREC average deck. `--pod-decks 490`.
        PodDeck {
            name: "Herigast, Erupting Nullkite (R)",
            commanders: decks::HERIGAST_COMMANDERS,
            main: decks::HERIGAST_MAIN,
        },
        // Seat 491: Tinybones, Trinket Thief's EDHREC average deck. `--pod-decks 491`.
        PodDeck {
            name: "Tinybones, Trinket Thief (B)",
            commanders: decks::TINYBONES_COMMANDERS,
            main: decks::TINYBONES_MAIN,
        },
        // Seat 492: Cayth, Famed Mechanist's EDHREC average deck. `--pod-decks 492`.
        PodDeck {
            name: "Cayth, Famed Mechanist (WUR)",
            commanders: decks::CAYTH_COMMANDERS,
            main: decks::CAYTH_MAIN,
        },
        // Seat 493: G'raha Tia, Scion Reborn's EDHREC average deck. `--pod-decks 493`.
        PodDeck {
            name: "G'raha Tia, Scion Reborn (WUB)",
            commanders: decks::G_RAHA_TIA_COMMANDERS,
            main: decks::G_RAHA_TIA_MAIN,
        },
        // Seat 494: Slimefoot and Squee's EDHREC average deck. `--pod-decks 494`.
        PodDeck {
            name: "Slimefoot and Squee (BRG)",
            commanders: decks::SLIMEFOOT_AND_SQUEE_COMMANDERS,
            main: decks::SLIMEFOOT_AND_SQUEE_MAIN,
        },
        // Seat 495: Elesh Norn, Mother of Machines's EDHREC average deck. `--pod-decks 495`.
        PodDeck {
            name: "Elesh Norn, Mother of Machines (W)",
            commanders: decks::ELESH_NORN_COMMANDERS,
            main: decks::ELESH_NORN_MAIN,
        },
        // Seat 496: Evereth, Viceroy of Plunder's EDHREC average deck. `--pod-decks 496`.
        PodDeck {
            name: "Evereth, Viceroy of Plunder (BR)",
            commanders: decks::EVERETH_COMMANDERS,
            main: decks::EVERETH_MAIN,
        },
        // Seat 497: Kiora, Sovereign of the Deep's EDHREC average deck. `--pod-decks 497`.
        PodDeck {
            name: "Kiora, Sovereign of the Deep (UG)",
            commanders: decks::KIORA_COMMANDERS,
            main: decks::KIORA_MAIN,
        },
        // Seat 498: Juri, Master of the Revue's EDHREC average deck. `--pod-decks 498`.
        PodDeck {
            name: "Juri, Master of the Revue (BR)",
            commanders: decks::JURI_COMMANDERS,
            main: decks::JURI_MAIN,
        },
        // Seat 499: Eshki Dragonclaw's EDHREC average deck. `--pod-decks 499`.
        PodDeck {
            name: "Eshki Dragonclaw (URG)",
            commanders: decks::ESHKI_DRAGONCLAW_COMMANDERS,
            main: decks::ESHKI_DRAGONCLAW_MAIN,
        },
        // Seat 500: Aang, at the Crossroads's EDHREC average deck. `--pod-decks 500`.
        PodDeck {
            name: "Aang, at the Crossroads (WUG)",
            commanders: decks::AANG_COMMANDERS,
            main: decks::AANG_MAIN,
        },
        // Seat 501: Aragorn, King of Gondor's EDHREC average deck. `--pod-decks 501`.
        PodDeck {
            name: "Aragorn, King of Gondor (WUR)",
            commanders: decks::ARAGORN_COMMANDERS,
            main: decks::ARAGORN_MAIN,
        },
        // Seat 502: Arixmethes, Slumbering Isle's EDHREC average deck. `--pod-decks 502`.
        PodDeck {
            name: "Arixmethes, Slumbering Isle (UG)",
            commanders: decks::ARIXMETHES_COMMANDERS,
            main: decks::ARIXMETHES_MAIN,
        },
        // Seat 503: Rivaz of the Claw's EDHREC average deck. `--pod-decks 503`.
        PodDeck {
            name: "Rivaz of the Claw (BR)",
            commanders: decks::RIVAZ_OF_THE_CLAW_COMMANDERS,
            main: decks::RIVAZ_OF_THE_CLAW_MAIN,
        },
        // Seat 504: Aurelia, the Warleader's EDHREC average deck. `--pod-decks 504`.
        PodDeck {
            name: "Aurelia, the Warleader (WR)",
            commanders: decks::AURELIA_COMMANDERS,
            main: decks::AURELIA_MAIN,
        },
        // Seat 505: Toph, Hardheaded Teacher's EDHREC average deck. `--pod-decks 505`.
        PodDeck {
            name: "Toph, Hardheaded Teacher (RG)",
            commanders: decks::TOPH_COMMANDERS,
            main: decks::TOPH_MAIN,
        },
        // Seat 506: Xantcha, Sleeper Agent's EDHREC average deck. `--pod-decks 506`.
        PodDeck {
            name: "Xantcha, Sleeper Agent (BR)",
            commanders: decks::XANTCHA_COMMANDERS,
            main: decks::XANTCHA_MAIN,
        },
        // Seat 507: Kyler, Sigardian Emissary's EDHREC average deck. `--pod-decks 507`.
        PodDeck {
            name: "Kyler, Sigardian Emissary (WG)",
            commanders: decks::KYLER_COMMANDERS,
            main: decks::KYLER_MAIN,
        },
        // Seat 508: Muerra, Trash Tactician's EDHREC average deck. `--pod-decks 508`.
        PodDeck {
            name: "Muerra, Trash Tactician (RG)",
            commanders: decks::MUERRA_COMMANDERS,
            main: decks::MUERRA_MAIN,
        },
        // Seat 509: Alesha, Who Laughs at Fate's EDHREC average deck. `--pod-decks 509`.
        PodDeck {
            name: "Alesha, Who Laughs at Fate (BR)",
            commanders: decks::ALESHA_509_COMMANDERS,
            main: decks::ALESHA_509_MAIN,
        },
        // Seat 510: Plagon, Lord of the Beach's EDHREC average deck. `--pod-decks 510`.
        PodDeck {
            name: "Plagon, Lord of the Beach (WU)",
            commanders: decks::PLAGON_COMMANDERS,
            main: decks::PLAGON_MAIN,
        },
        // Seat 511: Toph, Earthbending Master's EDHREC average deck. `--pod-decks 511`.
        PodDeck {
            name: "Toph, Earthbending Master (G)",
            commanders: decks::TOPH_511_COMMANDERS,
            main: decks::TOPH_511_MAIN,
        },
        // Seat 512: Amalia Benavides Aguirre's EDHREC average deck. `--pod-decks 512`.
        PodDeck {
            name: "Amalia Benavides Aguirre (WB)",
            commanders: decks::AMALIA_BENAVIDES_AGUIRRE_COMMANDERS,
            main: decks::AMALIA_BENAVIDES_AGUIRRE_MAIN,
        },
        // Seat 513: Sen Triplets's EDHREC average deck. `--pod-decks 513`.
        PodDeck {
            name: "Sen Triplets (WUB)",
            commanders: decks::SEN_TRIPLETS_COMMANDERS,
            main: decks::SEN_TRIPLETS_MAIN,
        },
        // Seat 514: Tasha, the Witch Queen's EDHREC average deck. `--pod-decks 514`.
        PodDeck {
            name: "Tasha, the Witch Queen (UB)",
            commanders: decks::TASHA_COMMANDERS,
            main: decks::TASHA_MAIN,
        },
        // Seat 515: Clement, the Worrywort's EDHREC average deck. `--pod-decks 515`.
        PodDeck {
            name: "Clement, the Worrywort (UG)",
            commanders: decks::CLEMENT_COMMANDERS,
            main: decks::CLEMENT_MAIN,
        },
        // Seat 516: Marwyn, the Nurturer's EDHREC average deck. `--pod-decks 516`.
        PodDeck {
            name: "Marwyn, the Nurturer (G)",
            commanders: decks::MARWYN_COMMANDERS,
            main: decks::MARWYN_MAIN,
        },
        // Seat 517: Imotekh the Stormlord's EDHREC average deck. `--pod-decks 517`.
        PodDeck {
            name: "Imotekh the Stormlord (B)",
            commanders: decks::IMOTEKH_THE_STORMLORD_COMMANDERS,
            main: decks::IMOTEKH_THE_STORMLORD_MAIN,
        },
        // Seat 518: Clive, Ifrit's Dominant's EDHREC average deck. `--pod-decks 518`.
        PodDeck {
            name: "Clive, Ifrit's Dominant (R)",
            commanders: decks::CLIVE_COMMANDERS,
            main: decks::CLIVE_MAIN,
        },
        // Seat 519: Mabel, Heir to Cragflame's EDHREC average deck. `--pod-decks 519`.
        PodDeck {
            name: "Mabel, Heir to Cragflame (WR)",
            commanders: decks::MABEL_COMMANDERS,
            main: decks::MABEL_MAIN,
        },
        // Seat 520: Jhoira, Weatherlight Captain's EDHREC average deck. `--pod-decks 520`.
        PodDeck {
            name: "Jhoira, Weatherlight Captain (UR)",
            commanders: decks::JHOIRA_COMMANDERS,
            main: decks::JHOIRA_MAIN,
        },
        // Seat 521: Talrand, Sky Summoner's EDHREC average deck. `--pod-decks 521`.
        PodDeck {
            name: "Talrand, Sky Summoner (U)",
            commanders: decks::TALRAND_COMMANDERS,
            main: decks::TALRAND_MAIN,
        },
        // Seat 522: Karumonix, the Rat King's EDHREC average deck. `--pod-decks 522`.
        PodDeck {
            name: "Karumonix, the Rat King (B)",
            commanders: decks::KARUMONIX_COMMANDERS,
            main: decks::KARUMONIX_MAIN,
        },
        // Seat 523: Kambal, Profiteering Mayor's EDHREC average deck. `--pod-decks 523`.
        PodDeck {
            name: "Kambal, Profiteering Mayor (WB)",
            commanders: decks::KAMBAL_COMMANDERS,
            main: decks::KAMBAL_MAIN,
        },
        // Seat 524: Smaug the Impenetrable's EDHREC average deck. `--pod-decks 524`.
        PodDeck {
            name: "Smaug the Impenetrable (C)",
            commanders: decks::SMAUG_THE_IMPENETRABLE_COMMANDERS,
            main: decks::SMAUG_THE_IMPENETRABLE_MAIN,
        },
        // Seat 525: Zur, Eternal Schemer's EDHREC average deck. `--pod-decks 525`.
        PodDeck {
            name: "Zur, Eternal Schemer (WUB)",
            commanders: decks::ZUR_525_COMMANDERS,
            main: decks::ZUR_525_MAIN,
        },
        // Seat 526: The Reaper, King No More's EDHREC average deck. `--pod-decks 526`.
        PodDeck {
            name: "The Reaper, King No More (BRG)",
            commanders: decks::THE_REAPER_COMMANDERS,
            main: decks::THE_REAPER_MAIN,
        },
        // Seat 527: Progenitus's EDHREC average deck. `--pod-decks 527`.
        PodDeck {
            name: "Progenitus (WUBRG)",
            commanders: decks::PROGENITUS_COMMANDERS,
            main: decks::PROGENITUS_MAIN,
        },
        // Seat 528: Massacre Girl, Known Killer's EDHREC average deck. `--pod-decks 528`.
        PodDeck {
            name: "Massacre Girl, Known Killer (B)",
            commanders: decks::MASSACRE_GIRL_COMMANDERS,
            main: decks::MASSACRE_GIRL_MAIN,
        },
        // Seat 529: Bre of Clan Stoutarm's EDHREC average deck. `--pod-decks 529`.
        PodDeck {
            name: "Bre of Clan Stoutarm (WR)",
            commanders: decks::BRE_OF_CLAN_STOUTARM_COMMANDERS,
            main: decks::BRE_OF_CLAN_STOUTARM_MAIN,
        },
        // Seat 530: Eirdu, Carrier of Dawn's EDHREC average deck. `--pod-decks 530`.
        PodDeck {
            name: "Eirdu, Carrier of Dawn (WB)",
            commanders: decks::EIRDU_COMMANDERS,
            main: decks::EIRDU_MAIN,
        },
        // Seat 531: Tatsunari, Toad Rider's EDHREC average deck. `--pod-decks 531`.
        PodDeck {
            name: "Tatsunari, Toad Rider (UBG)",
            commanders: decks::TATSUNARI_COMMANDERS,
            main: decks::TATSUNARI_MAIN,
        },
        // Seat 532: Gonti, Night Minister's EDHREC average deck. `--pod-decks 532`.
        PodDeck {
            name: "Gonti, Night Minister (B)",
            commanders: decks::GONTI_532_COMMANDERS,
            main: decks::GONTI_532_MAIN,
        },
        // Seat 533: Minthara, Merciless Soul's EDHREC average deck. `--pod-decks 533`.
        PodDeck {
            name: "Minthara, Merciless Soul (WB)",
            commanders: decks::MINTHARA_COMMANDERS,
            main: decks::MINTHARA_MAIN,
        },
        // Seat 534: Asmoranomardicadaistinaculdacar's EDHREC average deck. `--pod-decks 534`.
        PodDeck {
            name: "Asmoranomardicadaistinaculdacar (BR)",
            commanders: decks::ASMORANOMARDICADAISTINACULDACAR_COMMANDERS,
            main: decks::ASMORANOMARDICADAISTINACULDACAR_MAIN,
        },
        // Seat 535: Slicer, Hired Muscle's EDHREC average deck. `--pod-decks 535`.
        PodDeck {
            name: "Slicer, Hired Muscle (R)",
            commanders: decks::SLICER_COMMANDERS,
            main: decks::SLICER_MAIN,
        },
        // Seat 536: Kozilek, the Great Distortion's EDHREC average deck. `--pod-decks 536`.
        PodDeck {
            name: "Kozilek, the Great Distortion (C)",
            commanders: decks::KOZILEK_COMMANDERS,
            main: decks::KOZILEK_MAIN,
        },
        // Seat 537: Savra, Queen of the Golgari's EDHREC average deck. `--pod-decks 537`.
        PodDeck {
            name: "Savra, Queen of the Golgari (BG)",
            commanders: decks::SAVRA_COMMANDERS,
            main: decks::SAVRA_MAIN,
        },
        // Seat 538: Lonis, Cryptozoologist's EDHREC average deck. `--pod-decks 538`.
        PodDeck {
            name: "Lonis, Cryptozoologist (UG)",
            commanders: decks::LONIS_COMMANDERS,
            main: decks::LONIS_MAIN,
        },
        // Seat 539: Baral, Chief of Compliance's EDHREC average deck. `--pod-decks 539`.
        PodDeck {
            name: "Baral, Chief of Compliance (U)",
            commanders: decks::BARAL_COMMANDERS,
            main: decks::BARAL_MAIN,
        },
        // Seat 540: Marchesa, Dealer of Death's EDHREC average deck. `--pod-decks 540`.
        PodDeck {
            name: "Marchesa, Dealer of Death (UBR)",
            commanders: decks::MARCHESA_COMMANDERS,
            main: decks::MARCHESA_MAIN,
        },
        // Seat 541: Grolnok, the Omnivore's EDHREC average deck. `--pod-decks 541`.
        PodDeck {
            name: "Grolnok, the Omnivore (UG)",
            commanders: decks::GROLNOK_COMMANDERS,
            main: decks::GROLNOK_MAIN,
        },
        // Seat 542: Arthur, Marigold Knight's EDHREC average deck. `--pod-decks 542`.
        PodDeck {
            name: "Arthur, Marigold Knight (WUR)",
            commanders: decks::ARTHUR_COMMANDERS,
            main: decks::ARTHUR_MAIN,
        },
        // Seat 543: Uril, the Miststalker's EDHREC average deck. `--pod-decks 543`.
        PodDeck {
            name: "Uril, the Miststalker (WRG)",
            commanders: decks::URIL_COMMANDERS,
            main: decks::URIL_MAIN,
        },
        // Seat 544: Zurgo Helmsmasher's EDHREC average deck. `--pod-decks 544`.
        PodDeck {
            name: "Zurgo Helmsmasher (WBR)",
            commanders: decks::ZURGO_HELMSMASHER_COMMANDERS,
            main: decks::ZURGO_HELMSMASHER_MAIN,
        },
        // Seat 545: Taii Wakeen, Perfect Shot's EDHREC average deck. `--pod-decks 545`.
        PodDeck {
            name: "Taii Wakeen, Perfect Shot (WR)",
            commanders: decks::TAII_WAKEEN_COMMANDERS,
            main: decks::TAII_WAKEEN_MAIN,
        },
        // Seat 546: Silvar, Devourer of the Free // Trynn, Champion of Freedom's EDHREC average deck. `--pod-decks 546`.
        PodDeck {
            name: "Silvar, Devourer of the Free // Trynn, Champion of Freedom (WBR)",
            commanders: decks::SILVAR_COMMANDERS,
            main: decks::SILVAR_MAIN,
        },
        // Seat 547: Lady Octopus, Inspired Inventor's EDHREC average deck. `--pod-decks 547`.
        PodDeck {
            name: "Lady Octopus, Inspired Inventor (U)",
            commanders: decks::LADY_OCTOPUS_COMMANDERS,
            main: decks::LADY_OCTOPUS_MAIN,
        },
        // Seat 548: Elsha of the Infinite's EDHREC average deck. `--pod-decks 548`.
        PodDeck {
            name: "Elsha of the Infinite (WUR)",
            commanders: decks::ELSHA_OF_THE_INFINITE_COMMANDERS,
            main: decks::ELSHA_OF_THE_INFINITE_MAIN,
        },
        // Seat 549: Cazur, Ruthless Stalker and Ukkima, Stalking Shadow's EDHREC average deck. `--pod-decks 549`.
        PodDeck {
            name: "Cazur, Ruthless Stalker and Ukkima, Stalking Shadow (UBG)",
            commanders: decks::CAZUR_COMMANDERS,
            main: decks::CAZUR_MAIN,
        },
        // Seat 550: Purphoros, God of the Forge's EDHREC average deck. `--pod-decks 550`.
        PodDeck {
            name: "Purphoros, God of the Forge (R)",
            commanders: decks::PURPHOROS_COMMANDERS,
            main: decks::PURPHOROS_MAIN,
        },
        // Seat 551: Mogis, God of Slaughter's EDHREC average deck. `--pod-decks 551`.
        PodDeck {
            name: "Mogis, God of Slaughter (BR)",
            commanders: decks::MOGIS_COMMANDERS,
            main: decks::MOGIS_MAIN,
        },
        // Seat 552: Yargle and Multani's EDHREC average deck. `--pod-decks 552`.
        PodDeck {
            name: "Yargle and Multani (BG)",
            commanders: decks::YARGLE_AND_MULTANI_COMMANDERS,
            main: decks::YARGLE_AND_MULTANI_MAIN,
        },
        // Seat 553: Will, Scion of Peace's EDHREC average deck. `--pod-decks 553`.
        PodDeck {
            name: "Will, Scion of Peace (WU)",
            commanders: decks::WILL_COMMANDERS,
            main: decks::WILL_MAIN,
        },
        // Seat 554: Kruphix, God of Horizons's EDHREC average deck. `--pod-decks 554`.
        PodDeck {
            name: "Kruphix, God of Horizons (UG)",
            commanders: decks::KRUPHIX_COMMANDERS,
            main: decks::KRUPHIX_MAIN,
        },
        // Seat 555: Beledros Witherbloom's EDHREC average deck. `--pod-decks 555`.
        PodDeck {
            name: "Beledros Witherbloom (BG)",
            commanders: decks::BELEDROS_WITHERBLOOM_COMMANDERS,
            main: decks::BELEDROS_WITHERBLOOM_MAIN,
        },
        // Seat 556: Muddle, the Ever-Changing's EDHREC average deck. `--pod-decks 556`.
        PodDeck {
            name: "Muddle, the Ever-Changing (UR)",
            commanders: decks::MUDDLE_COMMANDERS,
            main: decks::MUDDLE_MAIN,
        },
        // Seat 557: Dionus, Elvish Archdruid's EDHREC average deck. `--pod-decks 557`.
        PodDeck {
            name: "Dionus, Elvish Archdruid (G)",
            commanders: decks::DIONUS_COMMANDERS,
            main: decks::DIONUS_MAIN,
        },
        // Seat 558: Rukarumel, Biologist's EDHREC average deck. `--pod-decks 558`.
        PodDeck {
            name: "Rukarumel, Biologist (WUBRG)",
            commanders: decks::RUKARUMEL_COMMANDERS,
            main: decks::RUKARUMEL_MAIN,
        },
        // Seat 559: Sliver Hivelord's EDHREC average deck. `--pod-decks 559`.
        PodDeck {
            name: "Sliver Hivelord (WUBRG)",
            commanders: decks::SLIVER_HIVELORD_COMMANDERS,
            main: decks::SLIVER_HIVELORD_MAIN,
        },
        // Seat 560: Emet-Selch, Unsundered's EDHREC average deck. `--pod-decks 560`.
        PodDeck {
            name: "Emet-Selch, Unsundered (UB)",
            commanders: decks::EMET_SELCH_COMMANDERS,
            main: decks::EMET_SELCH_MAIN,
        },
        // Seat 561: Radagast of Rhosgobel's EDHREC average deck. `--pod-decks 561`.
        PodDeck {
            name: "Radagast of Rhosgobel (G)",
            commanders: decks::RADAGAST_OF_RHOSGOBEL_COMMANDERS,
            main: decks::RADAGAST_OF_RHOSGOBEL_MAIN,
        },
        // Seat 562: Carmen, Cruel Skymarcher's EDHREC average deck. `--pod-decks 562`.
        PodDeck {
            name: "Carmen, Cruel Skymarcher (WB)",
            commanders: decks::CARMEN_COMMANDERS,
            main: decks::CARMEN_MAIN,
        },
        // Seat 563: Zenos yae Galvus's EDHREC average deck. `--pod-decks 563`.
        PodDeck {
            name: "Zenos yae Galvus (B)",
            commanders: decks::ZENOS_YAE_GALVUS_COMMANDERS,
            main: decks::ZENOS_YAE_GALVUS_MAIN,
        },
        // Seat 564: Archelos, Lagoon Mystic's EDHREC average deck. `--pod-decks 564`.
        PodDeck {
            name: "Archelos, Lagoon Mystic (UBG)",
            commanders: decks::ARCHELOS_COMMANDERS,
            main: decks::ARCHELOS_MAIN,
        },
        // Seat 565: Halana and Alena, Partners's EDHREC average deck. `--pod-decks 565`.
        PodDeck {
            name: "Halana and Alena, Partners (RG)",
            commanders: decks::HALANA_AND_ALENA_COMMANDERS,
            main: decks::HALANA_AND_ALENA_MAIN,
        },
        // Seat 566: Homer, the Hermit's EDHREC average deck. `--pod-decks 566`.
        PodDeck {
            name: "Homer, the Hermit (UBG)",
            commanders: decks::HOMER_COMMANDERS,
            main: decks::HOMER_MAIN,
        },
        // Seat 567: Sovereign Okinec Ahau's EDHREC average deck. `--pod-decks 567`.
        PodDeck {
            name: "Sovereign Okinec Ahau (WG)",
            commanders: decks::SOVEREIGN_OKINEC_AHAU_COMMANDERS,
            main: decks::SOVEREIGN_OKINEC_AHAU_MAIN,
        },
        // Seat 568: Torbran, Thane of Red Fell's EDHREC average deck. `--pod-decks 568`.
        PodDeck {
            name: "Torbran, Thane of Red Fell (R)",
            commanders: decks::TORBRAN_COMMANDERS,
            main: decks::TORBRAN_MAIN,
        },
        // Seat 569: Bumi, Unleashed's EDHREC average deck. `--pod-decks 569`.
        PodDeck {
            name: "Bumi, Unleashed (RG)",
            commanders: decks::BUMI_COMMANDERS,
            main: decks::BUMI_MAIN,
        },
        // Seat 570: Kresh the Bloodbraided's EDHREC average deck. `--pod-decks 570`.
        PodDeck {
            name: "Kresh the Bloodbraided (BRG)",
            commanders: decks::KRESH_THE_BLOODBRAIDED_COMMANDERS,
            main: decks::KRESH_THE_BLOODBRAIDED_MAIN,
        },
        // Seat 571: Merry, Warden of Isengard and Pippin, Warden of Isengard's EDHREC average deck. `--pod-decks 571`.
        PodDeck {
            name: "Merry, Warden of Isengard and Pippin, Warden of Isengard (WBG)",
            commanders: decks::MERRY_COMMANDERS,
            main: decks::MERRY_MAIN,
        },
        // Seat 572: Phelia, Exuberant Shepherd's EDHREC average deck. `--pod-decks 572`.
        PodDeck {
            name: "Phelia, Exuberant Shepherd (W)",
            commanders: decks::PHELIA_COMMANDERS,
            main: decks::PHELIA_MAIN,
        },
        // Seat 573: Urza, Lord Protector's EDHREC average deck. `--pod-decks 573`.
        PodDeck {
            name: "Urza, Lord Protector (WU)",
            commanders: decks::URZA_2_COMMANDERS,
            main: decks::URZA_2_MAIN,
        },
        // Seat 574: Kutzil, Malamet Exemplar's EDHREC average deck. `--pod-decks 574`.
        PodDeck {
            name: "Kutzil, Malamet Exemplar (WG)",
            commanders: decks::KUTZIL_COMMANDERS,
            main: decks::KUTZIL_MAIN,
        },
        // Seat 575: Rashmi and Ragavan's EDHREC average deck. `--pod-decks 575`.
        PodDeck {
            name: "Rashmi and Ragavan (URG)",
            commanders: decks::RASHMI_AND_RAGAVAN_COMMANDERS,
            main: decks::RASHMI_AND_RAGAVAN_MAIN,
        },
        // Seat 576: Zimone, Paradox Sculptor's EDHREC average deck. `--pod-decks 576`.
        PodDeck {
            name: "Zimone, Paradox Sculptor (UG)",
            commanders: decks::ZIMONE_2_COMMANDERS,
            main: decks::ZIMONE_2_MAIN,
        },
        // Seat 577: Oviya, Automech Artisan's EDHREC average deck. `--pod-decks 577`.
        PodDeck {
            name: "Oviya, Automech Artisan (G)",
            commanders: decks::OVIYA_COMMANDERS,
            main: decks::OVIYA_MAIN,
        },
        // Seat 578: Sab-Sunen, Luxa Embodied's EDHREC average deck. `--pod-decks 578`.
        PodDeck {
            name: "Sab-Sunen, Luxa Embodied (UG)",
            commanders: decks::SAB_SUNEN_COMMANDERS,
            main: decks::SAB_SUNEN_MAIN,
        },
        // Seat 579: Bonny Pall, Clearcutter's EDHREC average deck. `--pod-decks 579`.
        PodDeck {
            name: "Bonny Pall, Clearcutter (UG)",
            commanders: decks::BONNY_PALL_COMMANDERS,
            main: decks::BONNY_PALL_MAIN,
        },
        // Seat 580: Ardyn, the Usurper's EDHREC average deck. `--pod-decks 580`.
        PodDeck {
            name: "Ardyn, the Usurper (B)",
            commanders: decks::ARDYN_COMMANDERS,
            main: decks::ARDYN_MAIN,
        },
        // Seat 581: Kykar, Zephyr Awakener's EDHREC average deck. `--pod-decks 581`.
        PodDeck {
            name: "Kykar, Zephyr Awakener (WU)",
            commanders: decks::KYKAR_2_COMMANDERS,
            main: decks::KYKAR_2_MAIN,
        },
        // Seat 582: The Master, Transcendent's EDHREC average deck. `--pod-decks 582`.
        PodDeck {
            name: "The Master, Transcendent (UBG)",
            commanders: decks::THE_MASTER_COMMANDERS,
            main: decks::THE_MASTER_MAIN,
        },
        // Seat 583: Neheb, the Eternal's EDHREC average deck. `--pod-decks 583`.
        PodDeck {
            name: "Neheb, the Eternal (R)",
            commanders: decks::NEHEB_COMMANDERS,
            main: decks::NEHEB_MAIN,
        },
        // Seat 584: Niv-Mizzet, Guildpact's EDHREC average deck. `--pod-decks 584`.
        PodDeck {
            name: "Niv-Mizzet, Guildpact (WUBRG)",
            commanders: decks::NIV_MIZZET_2_COMMANDERS,
            main: decks::NIV_MIZZET_2_MAIN,
        },
        // Seat 585: Cadira, Caller of the Small's EDHREC average deck. `--pod-decks 585`.
        PodDeck {
            name: "Cadira, Caller of the Small (WG)",
            commanders: decks::CADIRA_COMMANDERS,
            main: decks::CADIRA_MAIN,
        },
        // Seat 586: Primo, the Unbounded's EDHREC average deck. `--pod-decks 586`.
        PodDeck {
            name: "Primo, the Unbounded (UG)",
            commanders: decks::PRIMO_COMMANDERS,
            main: decks::PRIMO_MAIN,
        },
        // Seat 587: Brallin, Skyshark Rider and Shabraz, the Skyshark's EDHREC average deck. `--pod-decks 587`.
        PodDeck {
            name: "Brallin, Skyshark Rider and Shabraz, the Skyshark (WUR)",
            commanders: decks::BRALLIN_COMMANDERS,
            main: decks::BRALLIN_MAIN,
        },
        // Seat 588: Yenna, Redtooth Regent's EDHREC average deck. `--pod-decks 588`.
        PodDeck {
            name: "Yenna, Redtooth Regent (WG)",
            commanders: decks::YENNA_COMMANDERS,
            main: decks::YENNA_MAIN,
        },
        // Seat 589: Koma, World-Eater's EDHREC average deck. `--pod-decks 589`.
        PodDeck {
            name: "Koma, World-Eater (UG)",
            commanders: decks::KOMA_2_COMMANDERS,
            main: decks::KOMA_2_MAIN,
        },
        // Seat 590: Shiko, Paragon of the Way's EDHREC average deck. `--pod-decks 590`.
        PodDeck {
            name: "Shiko, Paragon of the Way (WUR)",
            commanders: decks::SHIKO_2_COMMANDERS,
            main: decks::SHIKO_2_MAIN,
        },
        // Seat 591: Octavia, Living Thesis's EDHREC average deck. `--pod-decks 591`.
        PodDeck {
            name: "Octavia, Living Thesis (U)",
            commanders: decks::OCTAVIA_COMMANDERS,
            main: decks::OCTAVIA_MAIN,
        },
        // Seat 592: General Tazri's EDHREC average deck. `--pod-decks 592`.
        PodDeck {
            name: "General Tazri (WUBRG)",
            commanders: decks::GENERAL_TAZRI_COMMANDERS,
            main: decks::GENERAL_TAZRI_MAIN,
        },
        // Seat 593: Valgavoth, Terror Eater's EDHREC average deck. `--pod-decks 593`.
        PodDeck {
            name: "Valgavoth, Terror Eater (B)",
            commanders: decks::VALGAVOTH_2_COMMANDERS,
            main: decks::VALGAVOTH_2_MAIN,
        },
        // Seat 594: Mog, Moogle Warrior's EDHREC average deck. `--pod-decks 594`.
        PodDeck {
            name: "Mog, Moogle Warrior (WR)",
            commanders: decks::MOG_COMMANDERS,
            main: decks::MOG_MAIN,
        },
        // Seat 595: Sephiroth, Fallen Hero's EDHREC average deck. `--pod-decks 595`.
        PodDeck {
            name: "Sephiroth, Fallen Hero (WR)",
            commanders: decks::SEPHIROTH_2_COMMANDERS,
            main: decks::SEPHIROTH_2_MAIN,
        },
        // Seat 596: Adeline, Resplendent Cathar's EDHREC average deck. `--pod-decks 596`.
        PodDeck {
            name: "Adeline, Resplendent Cathar (W)",
            commanders: decks::ADELINE_COMMANDERS,
            main: decks::ADELINE_MAIN,
        },
        // Seat 597: Mazirek, Kraul Death Priest's EDHREC average deck. `--pod-decks 597`.
        PodDeck {
            name: "Mazirek, Kraul Death Priest (BG)",
            commanders: decks::MAZIREK_COMMANDERS,
            main: decks::MAZIREK_MAIN,
        },
        // Seat 598: Roon of the Hidden Realm's EDHREC average deck. `--pod-decks 598`.
        PodDeck {
            name: "Roon of the Hidden Realm (WUG)",
            commanders: decks::ROON_OF_THE_HIDDEN_REALM_COMMANDERS,
            main: decks::ROON_OF_THE_HIDDEN_REALM_MAIN,
        },
        // Seat 599: Errant and Giada's EDHREC average deck. `--pod-decks 599`.
        PodDeck {
            name: "Errant and Giada (WU)",
            commanders: decks::ERRANT_AND_GIADA_COMMANDERS,
            main: decks::ERRANT_AND_GIADA_MAIN,
        },
        // Seat 600: Ashcoat of the Shadow Swarm's EDHREC average deck. `--pod-decks 600`.
        PodDeck {
            name: "Ashcoat of the Shadow Swarm (B)",
            commanders: decks::ASHCOAT_OF_THE_SHADOW_SWARM_COMMANDERS,
            main: decks::ASHCOAT_OF_THE_SHADOW_SWARM_MAIN,
        },
        // Seat 601: Lord Xander, the Collector's EDHREC average deck. `--pod-decks 601`.
        PodDeck {
            name: "Lord Xander, the Collector (UBR)",
            commanders: decks::LORD_XANDER_COMMANDERS,
            main: decks::LORD_XANDER_MAIN,
        },
        // Seat 602: Wildsear, Scouring Maw's EDHREC average deck. `--pod-decks 602`.
        PodDeck {
            name: "Wildsear, Scouring Maw (RG)",
            commanders: decks::WILDSEAR_COMMANDERS,
            main: decks::WILDSEAR_MAIN,
        },
        // Seat 603: Sephiroth, Planet's Heir's EDHREC average deck. `--pod-decks 603`.
        PodDeck {
            name: "Sephiroth, Planet's Heir (UB)",
            commanders: decks::SEPHIROTH_3_COMMANDERS,
            main: decks::SEPHIROTH_3_MAIN,
        },
        // Seat 604: Damia, Sage of Stone's EDHREC average deck. `--pod-decks 604`.
        PodDeck {
            name: "Damia, Sage of Stone (UBG)",
            commanders: decks::DAMIA_COMMANDERS,
            main: decks::DAMIA_MAIN,
        },
        // Seat 605: Lavinia, Azorius Renegade's EDHREC average deck. `--pod-decks 605`.
        PodDeck {
            name: "Lavinia, Azorius Renegade (WU)",
            commanders: decks::LAVINIA_COMMANDERS,
            main: decks::LAVINIA_MAIN,
        },
        // Seat 606: Chainer, Nightmare Adept's EDHREC average deck. `--pod-decks 606`.
        PodDeck {
            name: "Chainer, Nightmare Adept (BR)",
            commanders: decks::CHAINER_COMMANDERS,
            main: decks::CHAINER_MAIN,
        },
        // Seat 607: Elenda, Saint of Dusk's EDHREC average deck. `--pod-decks 607`.
        PodDeck {
            name: "Elenda, Saint of Dusk (WB)",
            commanders: decks::ELENDA_2_COMMANDERS,
            main: decks::ELENDA_2_MAIN,
        },
        // Seat 608: Katara, Waterbending Master's EDHREC average deck. `--pod-decks 608`.
        PodDeck {
            name: "Katara, Waterbending Master (U)",
            commanders: decks::KATARA_COMMANDERS,
            main: decks::KATARA_MAIN,
        },
        // Seat 609: Rionya, Fire Dancer's EDHREC average deck. `--pod-decks 609`.
        PodDeck {
            name: "Rionya, Fire Dancer (R)",
            commanders: decks::RIONYA_COMMANDERS,
            main: decks::RIONYA_MAIN,
        },
        // Seat 610: Joshua, Phoenix's Dominant's EDHREC average deck. `--pod-decks 610`.
        PodDeck {
            name: "Joshua, Phoenix's Dominant (WR)",
            commanders: decks::JOSHUA_COMMANDERS,
            main: decks::JOSHUA_MAIN,
        },
        // Seat 611: Círdan the Shipwright's EDHREC average deck. `--pod-decks 611`.
        PodDeck {
            name: "Círdan the Shipwright (UG)",
            commanders: decks::CIRDAN_THE_SHIPWRIGHT_COMMANDERS,
            main: decks::CIRDAN_THE_SHIPWRIGHT_MAIN,
        },
        // Seat 612: Zul Ashur, Lich Lord's EDHREC average deck. `--pod-decks 612`.
        PodDeck {
            name: "Zul Ashur, Lich Lord (B)",
            commanders: decks::ZUL_ASHUR_COMMANDERS,
            main: decks::ZUL_ASHUR_MAIN,
        },
        // Seat 613: Licia, Sanguine Tribune's EDHREC average deck. `--pod-decks 613`.
        PodDeck {
            name: "Licia, Sanguine Tribune (WBR)",
            commanders: decks::LICIA_COMMANDERS,
            main: decks::LICIA_MAIN,
        },
        // Seat 614: Baeloth Barrityl, Entertainer and Noble Heritage's EDHREC average deck. `--pod-decks 614`.
        PodDeck {
            name: "Baeloth Barrityl, Entertainer and Noble Heritage (WR)",
            commanders: decks::BAELOTH_BARRITYL_COMMANDERS,
            main: decks::BAELOTH_BARRITYL_MAIN,
        },
        // Seat 615: Mimeoplasm, Revered One's EDHREC average deck. `--pod-decks 615`.
        PodDeck {
            name: "Mimeoplasm, Revered One (UBG)",
            commanders: decks::MIMEOPLASM_2_COMMANDERS,
            main: decks::MIMEOPLASM_2_MAIN,
        },
        // Seat 616: Amy Rose's EDHREC average deck. `--pod-decks 616`.
        PodDeck {
            name: "Amy Rose (WR)",
            commanders: decks::AMY_ROSE_COMMANDERS,
            main: decks::AMY_ROSE_MAIN,
        },
        // Seat 617: Calix, Guided by Fate's EDHREC average deck. `--pod-decks 617`.
        PodDeck {
            name: "Calix, Guided by Fate (WG)",
            commanders: decks::CALIX_COMMANDERS,
            main: decks::CALIX_MAIN,
        },
        // Seat 618: Ashnod the Uncaring's EDHREC average deck. `--pod-decks 618`.
        PodDeck {
            name: "Ashnod the Uncaring (UBR)",
            commanders: decks::ASHNOD_THE_UNCARING_COMMANDERS,
            main: decks::ASHNOD_THE_UNCARING_MAIN,
        },
        // Seat 619: Sharuum the Hegemon's EDHREC average deck. `--pod-decks 619`.
        PodDeck {
            name: "Sharuum the Hegemon (WUB)",
            commanders: decks::SHARUUM_THE_HEGEMON_COMMANDERS,
            main: decks::SHARUUM_THE_HEGEMON_MAIN,
        },
        // Seat 620: Cloud, Planet's Champion's EDHREC average deck. `--pod-decks 620`.
        PodDeck {
            name: "Cloud, Planet's Champion (WR)",
            commanders: decks::CLOUD_2_COMMANDERS,
            main: decks::CLOUD_2_MAIN,
        },
        // Seat 621: Lara Croft, Tomb Raider's EDHREC average deck. `--pod-decks 621`.
        PodDeck {
            name: "Lara Croft, Tomb Raider (URG)",
            commanders: decks::LARA_CROFT_COMMANDERS,
            main: decks::LARA_CROFT_MAIN,
        },
        // Seat 622: Preston, the Vanisher's EDHREC average deck. `--pod-decks 622`.
        PodDeck {
            name: "Preston, the Vanisher (W)",
            commanders: decks::PRESTON_COMMANDERS,
            main: decks::PRESTON_MAIN,
        },
        // Seat 623: Jenson Carthalion, Druid Exile's EDHREC average deck. `--pod-decks 623`.
        PodDeck {
            name: "Jenson Carthalion, Druid Exile (WUBRG)",
            commanders: decks::JENSON_CARTHALION_COMMANDERS,
            main: decks::JENSON_CARTHALION_MAIN,
        },
        // Seat 624: Lazav, Dimir Mastermind's EDHREC average deck. `--pod-decks 624`.
        PodDeck {
            name: "Lazav, Dimir Mastermind (UB)",
            commanders: decks::LAZAV_COMMANDERS,
            main: decks::LAZAV_MAIN,
        },
        // Seat 625: Elenda and Azor's EDHREC average deck. `--pod-decks 625`.
        PodDeck {
            name: "Elenda and Azor (WUB)",
            commanders: decks::ELENDA_AND_AZOR_COMMANDERS,
            main: decks::ELENDA_AND_AZOR_MAIN,
        },
        // Seat 626: Me, the Immortal's EDHREC average deck. `--pod-decks 626`.
        PodDeck {
            name: "Me, the Immortal (URG)",
            commanders: decks::ME_COMMANDERS,
            main: decks::ME_MAIN,
        },
        // Seat 627: Sydri, Galvanic Genius's EDHREC average deck. `--pod-decks 627`.
        PodDeck {
            name: "Sydri, Galvanic Genius (WUB)",
            commanders: decks::SYDRI_COMMANDERS,
            main: decks::SYDRI_MAIN,
        },
        // Seat 628: Ragavan, Nimble Pilferer's EDHREC average deck. `--pod-decks 628`.
        PodDeck {
            name: "Ragavan, Nimble Pilferer (R)",
            commanders: decks::RAGAVAN_COMMANDERS,
            main: decks::RAGAVAN_MAIN,
        },
        // Seat 629: Xolatoyac, the Smiling Flood's EDHREC average deck. `--pod-decks 629`.
        PodDeck {
            name: "Xolatoyac, the Smiling Flood (UG)",
            commanders: decks::XOLATOYAC_COMMANDERS,
            main: decks::XOLATOYAC_MAIN,
        },
        // Seat 630: Aziza, Mage Tower Captain's EDHREC average deck. `--pod-decks 630`.
        PodDeck {
            name: "Aziza, Mage Tower Captain (WR)",
            commanders: decks::AZIZA_COMMANDERS,
            main: decks::AZIZA_MAIN,
        },
        // Seat 631: Mondrak, Glory Dominus's EDHREC average deck. `--pod-decks 631`.
        PodDeck {
            name: "Mondrak, Glory Dominus (W)",
            commanders: decks::MONDRAK_COMMANDERS,
            main: decks::MONDRAK_MAIN,
        },
        // Seat 632: Gisela, Blade of Goldnight's EDHREC average deck. `--pod-decks 632`.
        PodDeck {
            name: "Gisela, Blade of Goldnight (WR)",
            commanders: decks::GISELA_2_COMMANDERS,
            main: decks::GISELA_2_MAIN,
        },
        // Seat 633: Treebeard, Gracious Host's EDHREC average deck. `--pod-decks 633`.
        PodDeck {
            name: "Treebeard, Gracious Host (WG)",
            commanders: decks::TREEBEARD_COMMANDERS,
            main: decks::TREEBEARD_MAIN,
        },
        // Seat 634: Narset, Jeskai Waymaster's EDHREC average deck. `--pod-decks 634`.
        PodDeck {
            name: "Narset, Jeskai Waymaster (WUR)",
            commanders: decks::NARSET_COMMANDERS,
            main: decks::NARSET_MAIN,
        },
        // Seat 635: Aerith, Last Ancient's EDHREC average deck. `--pod-decks 635`.
        PodDeck {
            name: "Aerith, Last Ancient (WG)",
            commanders: decks::AERITH_COMMANDERS,
            main: decks::AERITH_MAIN,
        },
        // Seat 636: Lurrus of the Dream-Den's EDHREC average deck. `--pod-decks 636`.
        PodDeck {
            name: "Lurrus of the Dream-Den (WB)",
            commanders: decks::LURRUS_OF_THE_DREAM_DEN_COMMANDERS,
            main: decks::LURRUS_OF_THE_DREAM_DEN_MAIN,
        },
        // Seat 637: Rona, Herald of Invasion's EDHREC average deck. `--pod-decks 637`.
        PodDeck {
            name: "Rona, Herald of Invasion (UB)",
            commanders: decks::RONA_COMMANDERS,
            main: decks::RONA_MAIN,
        },
        // Seat 638: Olivia Voldaren's EDHREC average deck. `--pod-decks 638`.
        PodDeck {
            name: "Olivia Voldaren (BR)",
            commanders: decks::OLIVIA_VOLDAREN_COMMANDERS,
            main: decks::OLIVIA_VOLDAREN_MAIN,
        },
        // Seat 639: Greymond, Avacyn's Stalwart's EDHREC average deck. `--pod-decks 639`.
        PodDeck {
            name: "Greymond, Avacyn's Stalwart (W)",
            commanders: decks::GREYMOND_COMMANDERS,
            main: decks::GREYMOND_MAIN,
        },
        // Seat 640: Molecule Man's EDHREC average deck. `--pod-decks 640`.
        PodDeck {
            name: "Molecule Man (C)",
            commanders: decks::MOLECULE_MAN_COMMANDERS,
            main: decks::MOLECULE_MAN_MAIN,
        },
        // Seat 641: General Kreat, the Boltbringer's EDHREC average deck. `--pod-decks 641`.
        PodDeck {
            name: "General Kreat, the Boltbringer (R)",
            commanders: decks::GENERAL_KREAT_COMMANDERS,
            main: decks::GENERAL_KREAT_MAIN,
        },
        // Seat 642: Agent Frank Horrigan's EDHREC average deck. `--pod-decks 642`.
        PodDeck {
            name: "Agent Frank Horrigan (BG)",
            commanders: decks::AGENT_FRANK_HORRIGAN_COMMANDERS,
            main: decks::AGENT_FRANK_HORRIGAN_MAIN,
        },
        // Seat 643: Nissa, Leyline Tamer's EDHREC average deck. `--pod-decks 643`.
        PodDeck {
            name: "Nissa, Leyline Tamer (WUBR)",
            commanders: decks::NISSA_COMMANDERS,
            main: decks::NISSA_MAIN,
        },
        // Seat 644: Kosei, Penitent Warlord's EDHREC average deck. `--pod-decks 644`.
        PodDeck {
            name: "Kosei, Penitent Warlord (G)",
            commanders: decks::KOSEI_COMMANDERS,
            main: decks::KOSEI_MAIN,
        },
        // Seat 645: Ureni, the Song Unending's EDHREC average deck. `--pod-decks 645`.
        PodDeck {
            name: "Ureni, the Song Unending (URG)",
            commanders: decks::URENI_2_COMMANDERS,
            main: decks::URENI_2_MAIN,
        },
        // Seat 646: Katilda, Dawnhart Prime's EDHREC average deck. `--pod-decks 646`.
        PodDeck {
            name: "Katilda, Dawnhart Prime (WG)",
            commanders: decks::KATILDA_COMMANDERS,
            main: decks::KATILDA_MAIN,
        },
        // Seat 647: Titania, Protector of Argoth's EDHREC average deck. `--pod-decks 647`.
        PodDeck {
            name: "Titania, Protector of Argoth (G)",
            commanders: decks::TITANIA_COMMANDERS,
            main: decks::TITANIA_MAIN,
        },
        // Seat 648: Aurelia, the Law Above's EDHREC average deck. `--pod-decks 648`.
        PodDeck {
            name: "Aurelia, the Law Above (WR)",
            commanders: decks::AURELIA_2_COMMANDERS,
            main: decks::AURELIA_2_MAIN,
        },
        // Seat 649: Dyadrine, Synthesis Amalgam's EDHREC average deck. `--pod-decks 649`.
        PodDeck {
            name: "Dyadrine, Synthesis Amalgam (WG)",
            commanders: decks::DYADRINE_COMMANDERS,
            main: decks::DYADRINE_MAIN,
        },
        // Seat 650: Thalisse, Reverent Medium's EDHREC average deck. `--pod-decks 650`.
        PodDeck {
            name: "Thalisse, Reverent Medium (WB)",
            commanders: decks::THALISSE_COMMANDERS,
            main: decks::THALISSE_MAIN,
        },
        // Seat 651: Dragonhawk, Fate's Tempest's EDHREC average deck. `--pod-decks 651`.
        PodDeck {
            name: "Dragonhawk, Fate's Tempest (R)",
            commanders: decks::DRAGONHAWK_COMMANDERS,
            main: decks::DRAGONHAWK_MAIN,
        },
        // Seat 652: Liberator, Urza's Battlethopter's EDHREC average deck. `--pod-decks 652`.
        PodDeck {
            name: "Liberator, Urza's Battlethopter (C)",
            commanders: decks::LIBERATOR_COMMANDERS,
            main: decks::LIBERATOR_MAIN,
        },
        // Seat 653: Neriv, Crackling Vanguard's EDHREC average deck. `--pod-decks 653`.
        PodDeck {
            name: "Neriv, Crackling Vanguard (WBR)",
            commanders: decks::NERIV_COMMANDERS,
            main: decks::NERIV_MAIN,
        },
        // Seat 654: Eladamri, Korvecdal's EDHREC average deck. `--pod-decks 654`.
        PodDeck {
            name: "Eladamri, Korvecdal (G)",
            commanders: decks::ELADAMRI_COMMANDERS,
            main: decks::ELADAMRI_MAIN,
        },
        // Seat 655: Yorion, Sky Nomad's EDHREC average deck. `--pod-decks 655`.
        PodDeck {
            name: "Yorion, Sky Nomad (WU)",
            commanders: decks::YORION_COMMANDERS,
            main: decks::YORION_MAIN,
        },
        // Seat 656: Alandra, Sky Dreamer's EDHREC average deck. `--pod-decks 656`.
        PodDeck {
            name: "Alandra, Sky Dreamer (U)",
            commanders: decks::ALANDRA_COMMANDERS,
            main: decks::ALANDRA_MAIN,
        },
        // Seat 657: Kroxa, Titan of Death's Hunger's EDHREC average deck. `--pod-decks 657`.
        PodDeck {
            name: "Kroxa, Titan of Death's Hunger (BR)",
            commanders: decks::KROXA_COMMANDERS,
            main: decks::KROXA_MAIN,
        },
        // Seat 658: Thraximundar's EDHREC average deck. `--pod-decks 658`.
        PodDeck {
            name: "Thraximundar (UBR)",
            commanders: decks::THRAXIMUNDAR_COMMANDERS,
            main: decks::THRAXIMUNDAR_MAIN,
        },
        // Seat 659: Kambal, Consul of Allocation's EDHREC average deck. `--pod-decks 659`.
        PodDeck {
            name: "Kambal, Consul of Allocation (WB)",
            commanders: decks::KAMBAL_2_COMMANDERS,
            main: decks::KAMBAL_2_MAIN,
        },
        // Seat 660: Victor, Valgavoth's Seneschal's EDHREC average deck. `--pod-decks 660`.
        PodDeck {
            name: "Victor, Valgavoth's Seneschal (WB)",
            commanders: decks::VICTOR_COMMANDERS,
            main: decks::VICTOR_MAIN,
        },
        // Seat 661: Alibou, Ancient Witness's EDHREC average deck. `--pod-decks 661`.
        PodDeck {
            name: "Alibou, Ancient Witness (WR)",
            commanders: decks::ALIBOU_COMMANDERS,
            main: decks::ALIBOU_MAIN,
        },
        // Seat 662: Felisa, Fang of Silverquill's EDHREC average deck. `--pod-decks 662`.
        PodDeck {
            name: "Felisa, Fang of Silverquill (WB)",
            commanders: decks::FELISA_COMMANDERS,
            main: decks::FELISA_MAIN,
        },
        // Seat 663: Missy's EDHREC average deck. `--pod-decks 663`.
        PodDeck {
            name: "Missy (UBR)",
            commanders: decks::MISSY_COMMANDERS,
            main: decks::MISSY_MAIN,
        },
        // Seat 664: Ephara, God of the Polis's EDHREC average deck. `--pod-decks 664`.
        PodDeck {
            name: "Ephara, God of the Polis (WU)",
            commanders: decks::EPHARA_COMMANDERS,
            main: decks::EPHARA_MAIN,
        },
        // Seat 665: Greensleeves, Maro-Sorcerer's EDHREC average deck. `--pod-decks 665`.
        PodDeck {
            name: "Greensleeves, Maro-Sorcerer (G)",
            commanders: decks::GREENSLEEVES_COMMANDERS,
            main: decks::GREENSLEEVES_MAIN,
        },
        // Seat 666: Klauth, Unrivaled Ancient's EDHREC average deck. `--pod-decks 666`.
        PodDeck {
            name: "Klauth, Unrivaled Ancient (RG)",
            commanders: decks::KLAUTH_COMMANDERS,
            main: decks::KLAUTH_MAIN,
        },
        // Seat 667: Lae'zel, Vlaakith's Champion and Master Chef's EDHREC average deck. `--pod-decks 667`.
        PodDeck {
            name: "Lae'zel, Vlaakith's Champion and Master Chef (WG)",
            commanders: decks::LAEZEL_COMMANDERS,
            main: decks::LAEZEL_MAIN,
        },
        // Seat 668: Galazeth Prismari's EDHREC average deck. `--pod-decks 668`.
        PodDeck {
            name: "Galazeth Prismari (UR)",
            commanders: decks::GALAZETH_PRISMARI_COMMANDERS,
            main: decks::GALAZETH_PRISMARI_MAIN,
        },
        // Seat 669: Grimgrin, Corpse-Born's EDHREC average deck. `--pod-decks 669`.
        PodDeck {
            name: "Grimgrin, Corpse-Born (UB)",
            commanders: decks::GRIMGRIN_COMMANDERS,
            main: decks::GRIMGRIN_MAIN,
        },
        // Seat 670: Rev, Tithe Extractor's EDHREC average deck. `--pod-decks 670`.
        PodDeck {
            name: "Rev, Tithe Extractor (B)",
            commanders: decks::REV_COMMANDERS,
            main: decks::REV_MAIN,
        },
        // Seat 671: Chiss-Goria, Forge Tyrant's EDHREC average deck. `--pod-decks 671`.
        PodDeck {
            name: "Chiss-Goria, Forge Tyrant (R)",
            commanders: decks::CHISS_GORIA_COMMANDERS,
            main: decks::CHISS_GORIA_MAIN,
        },
        // Seat 672: Knuckles the Echidna's EDHREC average deck. `--pod-decks 672`.
        PodDeck {
            name: "Knuckles the Echidna (R)",
            commanders: decks::KNUCKLES_THE_ECHIDNA_COMMANDERS,
            main: decks::KNUCKLES_THE_ECHIDNA_MAIN,
        },
        // Seat 673: The Watcher in the Water's EDHREC average deck. `--pod-decks 673`.
        PodDeck {
            name: "The Watcher in the Water (U)",
            commanders: decks::THE_WATCHER_IN_THE_WATER_COMMANDERS,
            main: decks::THE_WATCHER_IN_THE_WATER_MAIN,
        },
        // Seat 674: Urabrask's EDHREC average deck. `--pod-decks 674`.
        PodDeck {
            name: "Urabrask (R)",
            commanders: decks::URABRASK_COMMANDERS,
            main: decks::URABRASK_MAIN,
        },
        // Seat 675: Graaz, Unstoppable Juggernaut's EDHREC average deck. `--pod-decks 675`.
        PodDeck {
            name: "Graaz, Unstoppable Juggernaut (C)",
            commanders: decks::GRAAZ_COMMANDERS,
            main: decks::GRAAZ_MAIN,
        },
        // Seat 676: Volrath, the Shapestealer's EDHREC average deck. `--pod-decks 676`.
        PodDeck {
            name: "Volrath, the Shapestealer (UBG)",
            commanders: decks::VOLRATH_COMMANDERS,
            main: decks::VOLRATH_MAIN,
        },
        // Seat 677: Liesa, Forgotten Archangel's EDHREC average deck. `--pod-decks 677`.
        PodDeck {
            name: "Liesa, Forgotten Archangel (WB)",
            commanders: decks::LIESA_COMMANDERS,
            main: decks::LIESA_MAIN,
        },
        // Seat 678: The Pride of Hull Clade's EDHREC average deck. `--pod-decks 678`.
        PodDeck {
            name: "The Pride of Hull Clade (UG)",
            commanders: decks::THE_PRIDE_OF_HULL_CLADE_COMMANDERS,
            main: decks::THE_PRIDE_OF_HULL_CLADE_MAIN,
        },
        // Seat 679: Dawnsire, Sunstar Dreadnought's EDHREC average deck. `--pod-decks 679`.
        PodDeck {
            name: "Dawnsire, Sunstar Dreadnought (C)",
            commanders: decks::DAWNSIRE_COMMANDERS,
            main: decks::DAWNSIRE_MAIN,
        },
        // Seat 680: Glissa, the Traitor's EDHREC average deck. `--pod-decks 680`.
        PodDeck {
            name: "Glissa, the Traitor (BG)",
            commanders: decks::GLISSA_COMMANDERS,
            main: decks::GLISSA_MAIN,
        },
        // Seat 681: Ezuri, Renegade Leader's EDHREC average deck. `--pod-decks 681`.
        PodDeck {
            name: "Ezuri, Renegade Leader (G)",
            commanders: decks::EZURI_2_COMMANDERS,
            main: decks::EZURI_2_MAIN,
        },
        // Seat 682: Grismold, the Dreadsower's EDHREC average deck. `--pod-decks 682`.
        PodDeck {
            name: "Grismold, the Dreadsower (BG)",
            commanders: decks::GRISMOLD_COMMANDERS,
            main: decks::GRISMOLD_MAIN,
        },
        // Seat 683: God-Eternal Oketra's EDHREC average deck. `--pod-decks 683`.
        PodDeck {
            name: "God-Eternal Oketra (W)",
            commanders: decks::GOD_ETERNAL_OKETRA_COMMANDERS,
            main: decks::GOD_ETERNAL_OKETRA_MAIN,
        },
        // Seat 684: The Ghoul, Gunslinger's EDHREC average deck. `--pod-decks 684`.
        PodDeck {
            name: "The Ghoul, Gunslinger (B)",
            commanders: decks::THE_GHOUL_COMMANDERS,
            main: decks::THE_GHOUL_MAIN,
        },
        // Seat 685: Goro-Goro and Satoru's EDHREC average deck. `--pod-decks 685`.
        PodDeck {
            name: "Goro-Goro and Satoru (UBR)",
            commanders: decks::GORO_GORO_AND_SATORU_COMMANDERS,
            main: decks::GORO_GORO_AND_SATORU_MAIN,
        },
        // Seat 686: Neriv, Heart of the Storm's EDHREC average deck. `--pod-decks 686`.
        PodDeck {
            name: "Neriv, Heart of the Storm (WBR)",
            commanders: decks::NERIV_686_COMMANDERS,
            main: decks::NERIV_686_MAIN,
        },
        // Seat 687: Xavier Sal, Infested Captain's EDHREC average deck. `--pod-decks 687`.
        PodDeck {
            name: "Xavier Sal, Infested Captain (UBG)",
            commanders: decks::XAVIER_SAL_COMMANDERS,
            main: decks::XAVIER_SAL_MAIN,
        },
        // Seat 688: Myrkul, Lord of Bones's EDHREC average deck. `--pod-decks 688`.
        PodDeck {
            name: "Myrkul, Lord of Bones (WBG)",
            commanders: decks::MYRKUL_COMMANDERS,
            main: decks::MYRKUL_MAIN,
        },
        // Seat 689: The Beamtown Bullies's EDHREC average deck. `--pod-decks 689`.
        PodDeck {
            name: "The Beamtown Bullies (BRG)",
            commanders: decks::THE_BEAMTOWN_BULLIES_COMMANDERS,
            main: decks::THE_BEAMTOWN_BULLIES_MAIN,
        },
        // Seat 690: Kwain, Itinerant Meddler's EDHREC average deck. `--pod-decks 690`.
        PodDeck {
            name: "Kwain, Itinerant Meddler (WU)",
            commanders: decks::KWAIN_COMMANDERS,
            main: decks::KWAIN_MAIN,
        },
        // Seat 691: Zimone and Dina's EDHREC average deck. `--pod-decks 691`.
        PodDeck {
            name: "Zimone and Dina (UBG)",
            commanders: decks::ZIMONE_AND_DINA_COMMANDERS,
            main: decks::ZIMONE_AND_DINA_MAIN,
        },
        // Seat 692: Brudiclad, Telchor Engineer's EDHREC average deck. `--pod-decks 692`.
        PodDeck {
            name: "Brudiclad, Telchor Engineer (UR)",
            commanders: decks::BRUDICLAD_COMMANDERS,
            main: decks::BRUDICLAD_MAIN,
        },
        // Seat 693: Riku of Many Paths's EDHREC average deck. `--pod-decks 693`.
        PodDeck {
            name: "Riku of Many Paths (URG)",
            commanders: decks::RIKU_OF_MANY_PATHS_COMMANDERS,
            main: decks::RIKU_OF_MANY_PATHS_MAIN,
        },
        // Seat 694: Ratadrabik of Urborg's EDHREC average deck. `--pod-decks 694`.
        PodDeck {
            name: "Ratadrabik of Urborg (WB)",
            commanders: decks::RATADRABIK_OF_URBORG_COMMANDERS,
            main: decks::RATADRABIK_OF_URBORG_MAIN,
        },
        // Seat 695: Narset, Enlightened Master's EDHREC average deck. `--pod-decks 695`.
        PodDeck {
            name: "Narset, Enlightened Master (WUR)",
            commanders: decks::NARSET_695_COMMANDERS,
            main: decks::NARSET_695_MAIN,
        },
        // Seat 696: Syr Konrad, the Grim's EDHREC average deck. `--pod-decks 696`.
        PodDeck {
            name: "Syr Konrad, the Grim (B)",
            commanders: decks::SYR_KONRAD_COMMANDERS,
            main: decks::SYR_KONRAD_MAIN,
        },
        // Seat 697: Runo Stromkirk's EDHREC average deck. `--pod-decks 697`.
        PodDeck {
            name: "Runo Stromkirk (UB)",
            commanders: decks::RUNO_STROMKIRK_COMMANDERS,
            main: decks::RUNO_STROMKIRK_MAIN,
        },
        // Seat 698: Altaïr Ibn-La'Ahad's EDHREC average deck. `--pod-decks 698`.
        PodDeck {
            name: "Altaïr Ibn-La'Ahad (WBR)",
            commanders: decks::ALTAIR_IBN_LA_AHAD_COMMANDERS,
            main: decks::ALTAIR_IBN_LA_AHAD_MAIN,
        },
        // Seat 699: Kumena, Tyrant of Orazca's EDHREC average deck. `--pod-decks 699`.
        PodDeck {
            name: "Kumena, Tyrant of Orazca (UG)",
            commanders: decks::KUMENA_COMMANDERS,
            main: decks::KUMENA_MAIN,
        },
        // Seat 700: Kediss, Emberclaw Familiar // Malcolm, Keen-Eyed Navigator's EDHREC average deck. `--pod-decks 700`.
        PodDeck {
            name: "Kediss, Emberclaw Familiar // Malcolm, Keen-Eyed Navigator (UR)",
            commanders: decks::KEDISS_COMMANDERS,
            main: decks::KEDISS_MAIN,
        },
        // Seat 701: Nahiri, Forged in Fury's EDHREC average deck. `--pod-decks 701`.
        PodDeck {
            name: "Nahiri, Forged in Fury (WR)",
            commanders: decks::NAHIRI_701_COMMANDERS,
            main: decks::NAHIRI_701_MAIN,
        },
        // Seat 702: Selvala, Heart of the Wilds's EDHREC average deck. `--pod-decks 702`.
        PodDeck {
            name: "Selvala, Heart of the Wilds (G)",
            commanders: decks::SELVALA_COMMANDERS,
            main: decks::SELVALA_MAIN,
        },
        // Seat 703: Liliana, Heretical Healer's EDHREC average deck. `--pod-decks 703`.
        PodDeck {
            name: "Liliana, Heretical Healer (B)",
            commanders: decks::LILIANA_COMMANDERS,
            main: decks::LILIANA_MAIN,
        },
        // Seat 704: Vraska, Soul of Stone's EDHREC average deck. `--pod-decks 704`.
        PodDeck {
            name: "Vraska, Soul of Stone (WUR)",
            commanders: decks::VRASKA_COMMANDERS,
            main: decks::VRASKA_MAIN,
        },
        // Seat 705: Darien, King of Kjeldor's EDHREC average deck. `--pod-decks 705`.
        PodDeck {
            name: "Darien, King of Kjeldor (W)",
            commanders: decks::DARIEN_COMMANDERS,
            main: decks::DARIEN_MAIN,
        },
        // Seat 706: Zacama, Primal Calamity's EDHREC average deck. `--pod-decks 706`.
        PodDeck {
            name: "Zacama, Primal Calamity (WRG)",
            commanders: decks::ZACAMA_COMMANDERS,
            main: decks::ZACAMA_MAIN,
        },
        // Seat 707: Selvala, Explorer Returned's EDHREC average deck. `--pod-decks 707`.
        PodDeck {
            name: "Selvala, Explorer Returned (WG)",
            commanders: decks::SELVALA_2_COMMANDERS,
            main: decks::SELVALA_2_MAIN,
        },
        // Seat 708: Abdel Adrian, Gorion's Ward and Candlekeep Sage's EDHREC average deck. `--pod-decks 708`.
        PodDeck {
            name: "Abdel Adrian, Gorion's Ward and Candlekeep Sage (WU)",
            commanders: decks::ABDEL_ADRIAN_COMMANDERS,
            main: decks::ABDEL_ADRIAN_MAIN,
        },
        // Seat 709: Crystal, Inhuman Princess's EDHREC average deck. `--pod-decks 709`.
        PodDeck {
            name: "Crystal, Inhuman Princess (WURG)",
            commanders: decks::CRYSTAL_COMMANDERS,
            main: decks::CRYSTAL_MAIN,
        },
        // Seat 710: Zilortha, Strength Incarnate's EDHREC average deck. `--pod-decks 710`.
        PodDeck {
            name: "Zilortha, Strength Incarnate (RG)",
            commanders: decks::ZILORTHA_COMMANDERS,
            main: decks::ZILORTHA_MAIN,
        },
        // Seat 711: Illuna, Apex of Wishes's EDHREC average deck. `--pod-decks 711`.
        PodDeck {
            name: "Illuna, Apex of Wishes (URG)",
            commanders: decks::ILLUNA_COMMANDERS,
            main: decks::ILLUNA_MAIN,
        },
        // Seat 712: Mm'menon, the Right Hand's EDHREC average deck. `--pod-decks 712`.
        PodDeck {
            name: "Mm'menon, the Right Hand (U)",
            commanders: decks::MMMENON_COMMANDERS,
            main: decks::MMMENON_MAIN,
        },
        // Seat 713: Niv-Mizzet, Ghost Counsel's EDHREC average deck. `--pod-decks 713`.
        PodDeck {
            name: "Niv-Mizzet, Ghost Counsel (WB)",
            commanders: decks::NIV_MIZZET_3_COMMANDERS,
            main: decks::NIV_MIZZET_3_MAIN,
        },
        // Seat 714: Bjorna, Nightfall Alchemist and Wernog, Rider's Chaplain's EDHREC average deck. `--pod-decks 714`.
        PodDeck {
            name: "Bjorna, Nightfall Alchemist and Wernog, Rider's Chaplain (WUBR)",
            commanders: decks::BJORNA_COMMANDERS,
            main: decks::BJORNA_MAIN,
        },
        // Seat 715: Yarus, Roar of the Old Gods's EDHREC average deck. `--pod-decks 715`.
        PodDeck {
            name: "Yarus, Roar of the Old Gods (RG)",
            commanders: decks::YARUS_COMMANDERS,
            main: decks::YARUS_MAIN,
        },
        // Seat 716: Burakos, Party Leader and Folk Hero's EDHREC average deck. `--pod-decks 716`.
        PodDeck {
            name: "Burakos, Party Leader and Folk Hero (WB)",
            commanders: decks::BURAKOS_COMMANDERS,
            main: decks::BURAKOS_MAIN,
        },
        // Seat 717: Super Shredder's EDHREC average deck. `--pod-decks 717`.
        PodDeck {
            name: "Super Shredder (B)",
            commanders: decks::SUPER_SHREDDER_COMMANDERS,
            main: decks::SUPER_SHREDDER_MAIN,
        },
        // Seat 718: Gyome, Master Chef's EDHREC average deck. `--pod-decks 718`.
        PodDeck {
            name: "Gyome, Master Chef (BG)",
            commanders: decks::GYOME_COMMANDERS,
            main: decks::GYOME_MAIN,
        },
        // Seat 719: Skithiryx, the Blight Dragon's EDHREC average deck. `--pod-decks 719`.
        PodDeck {
            name: "Skithiryx, the Blight Dragon (B)",
            commanders: decks::SKITHIRYX_COMMANDERS,
            main: decks::SKITHIRYX_MAIN,
        },
        // Seat 720: Old Stickfingers's EDHREC average deck. `--pod-decks 720`.
        PodDeck {
            name: "Old Stickfingers (BG)",
            commanders: decks::OLD_STICKFINGERS_COMMANDERS,
            main: decks::OLD_STICKFINGERS_MAIN,
        },
        // Seat 721: Balmor, Battlemage Captain's EDHREC average deck. `--pod-decks 721`.
        PodDeck {
            name: "Balmor, Battlemage Captain (UR)",
            commanders: decks::BALMOR_COMMANDERS,
            main: decks::BALMOR_MAIN,
        },
        // Seat 722: Iroas, God of Victory's EDHREC average deck. `--pod-decks 722`.
        PodDeck {
            name: "Iroas, God of Victory (WR)",
            commanders: decks::IROAS_COMMANDERS,
            main: decks::IROAS_MAIN,
        },
        // Seat 723: Kellan, the Fae-Blooded's EDHREC average deck. `--pod-decks 723`.
        PodDeck {
            name: "Kellan, the Fae-Blooded (WR)",
            commanders: decks::KELLAN_2_COMMANDERS,
            main: decks::KELLAN_2_MAIN,
        },
        // Seat 724: Ares, God of War's EDHREC average deck. `--pod-decks 724`.
        PodDeck {
            name: "Ares, God of War (BR)",
            commanders: decks::ARES_COMMANDERS,
            main: decks::ARES_MAIN,
        },
        // Seat 725: Gwenom, Remorseless's EDHREC average deck. `--pod-decks 725`.
        PodDeck {
            name: "Gwenom, Remorseless (B)",
            commanders: decks::GWENOM_COMMANDERS,
            main: decks::GWENOM_MAIN,
        },
        // Seat 726: Imskir Iron-Eater's EDHREC average deck. `--pod-decks 726`.
        PodDeck {
            name: "Imskir Iron-Eater (BR)",
            commanders: decks::IMSKIR_IRON_EATER_COMMANDERS,
            main: decks::IMSKIR_IRON_EATER_MAIN,
        },
        // Seat 727: Jolene, the Plunder Queen's EDHREC average deck. `--pod-decks 727`.
        PodDeck {
            name: "Jolene, the Plunder Queen (RG)",
            commanders: decks::JOLENE_COMMANDERS,
            main: decks::JOLENE_MAIN,
        },
        // Seat 728: Balan, Wandering Knight's EDHREC average deck. `--pod-decks 728`.
        PodDeck {
            name: "Balan, Wandering Knight (W)",
            commanders: decks::BALAN_COMMANDERS,
            main: decks::BALAN_MAIN,
        },
        // Seat 729: Red Death, Shipwrecker's EDHREC average deck. `--pod-decks 729`.
        PodDeck {
            name: "Red Death, Shipwrecker (UR)",
            commanders: decks::RED_DEATH_COMMANDERS,
            main: decks::RED_DEATH_MAIN,
        },
        // Seat 730: Delina, Wild Mage's EDHREC average deck. `--pod-decks 730`.
        PodDeck {
            name: "Delina, Wild Mage (R)",
            commanders: decks::DELINA_COMMANDERS,
            main: decks::DELINA_MAIN,
        },
        // Seat 731: Valduk, Keeper of the Flame's EDHREC average deck. `--pod-decks 731`.
        PodDeck {
            name: "Valduk, Keeper of the Flame (R)",
            commanders: decks::VALDUK_COMMANDERS,
            main: decks::VALDUK_MAIN,
        },
        // Seat 732: Eris, Roar of the Storm's EDHREC average deck. `--pod-decks 732`.
        PodDeck {
            name: "Eris, Roar of the Storm (UR)",
            commanders: decks::ERIS_COMMANDERS,
            main: decks::ERIS_MAIN,
        },
        // Seat 733: Zabaz, the Glimmerwasp's EDHREC average deck. `--pod-decks 733`.
        PodDeck {
            name: "Zabaz, the Glimmerwasp (WR)",
            commanders: decks::ZABAZ_COMMANDERS,
            main: decks::ZABAZ_MAIN,
        },
        // Seat 734: Killian, Ink Duelist's EDHREC average deck. `--pod-decks 734`.
        PodDeck {
            name: "Killian, Ink Duelist (WB)",
            commanders: decks::KILLIAN_2_COMMANDERS,
            main: decks::KILLIAN_2_MAIN,
        },
        // Seat 735: Mathas, Fiend Seeker's EDHREC average deck. `--pod-decks 735`.
        PodDeck {
            name: "Mathas, Fiend Seeker (WBR)",
            commanders: decks::MATHAS_COMMANDERS,
            main: decks::MATHAS_MAIN,
        },
        // Seat 736: Pir, Imaginative Rascal and Toothy, Imaginary Friend's EDHREC average deck. `--pod-decks 736`.
        PodDeck {
            name: "Pir, Imaginative Rascal and Toothy, Imaginary Friend (UG)",
            commanders: decks::PIR_COMMANDERS,
            main: decks::PIR_MAIN,
        },
        // Seat 737: Iroh, Tea Master's EDHREC average deck. `--pod-decks 737`.
        PodDeck {
            name: "Iroh, Tea Master (WR)",
            commanders: decks::IROH_COMMANDERS,
            main: decks::IROH_MAIN,
        },
        // Seat 738: Borborygmos Enraged's EDHREC average deck. `--pod-decks 738`.
        PodDeck {
            name: "Borborygmos Enraged (RG)",
            commanders: decks::BORBORYGMOS_ENRAGED_COMMANDERS,
            main: decks::BORBORYGMOS_ENRAGED_MAIN,
        },
        // Seat 739: Tyvar the Bellicose's EDHREC average deck. `--pod-decks 739`.
        PodDeck {
            name: "Tyvar the Bellicose (BG)",
            commanders: decks::TYVAR_THE_BELLICOSE_COMMANDERS,
            main: decks::TYVAR_THE_BELLICOSE_MAIN,
        },
        // Seat 740: Avacyn, Angel of Hope's EDHREC average deck. `--pod-decks 740`.
        PodDeck {
            name: "Avacyn, Angel of Hope (W)",
            commanders: decks::AVACYN_COMMANDERS,
            main: decks::AVACYN_MAIN,
        },
        // Seat 741: Ultima, Origin of Oblivion's EDHREC average deck. `--pod-decks 741`.
        PodDeck {
            name: "Ultima, Origin of Oblivion (C)",
            commanders: decks::ULTIMA_COMMANDERS,
            main: decks::ULTIMA_MAIN,
        },
        // Seat 742: Esix, Fractal Bloom's EDHREC average deck. `--pod-decks 742`.
        PodDeck {
            name: "Esix, Fractal Bloom (UG)",
            commanders: decks::ESIX_COMMANDERS,
            main: decks::ESIX_MAIN,
        },
        // Seat 743: Teysa, Orzhov Scion's EDHREC average deck. `--pod-decks 743`.
        PodDeck {
            name: "Teysa, Orzhov Scion (WB)",
            commanders: decks::TEYSA_2_COMMANDERS,
            main: decks::TEYSA_2_MAIN,
        },
        // Seat 744: Soul of Windgrace's EDHREC average deck. `--pod-decks 744`.
        PodDeck {
            name: "Soul of Windgrace (BRG)",
            commanders: decks::SOUL_OF_WINDGRACE_COMMANDERS,
            main: decks::SOUL_OF_WINDGRACE_MAIN,
        },
        // Seat 745: Urza, Prince of Kroog's EDHREC average deck. `--pod-decks 745`.
        PodDeck {
            name: "Urza, Prince of Kroog (WU)",
            commanders: decks::URZA_3_COMMANDERS,
            main: decks::URZA_3_MAIN,
        },
        // Seat 746: The Goose Mother's EDHREC average deck. `--pod-decks 746`.
        PodDeck {
            name: "The Goose Mother (UG)",
            commanders: decks::THE_GOOSE_MOTHER_COMMANDERS,
            main: decks::THE_GOOSE_MOTHER_MAIN,
        },
        // Seat 747: Toski, Bearer of Secrets's EDHREC average deck. `--pod-decks 747`.
        PodDeck {
            name: "Toski, Bearer of Secrets (G)",
            commanders: decks::TOSKI_COMMANDERS,
            main: decks::TOSKI_MAIN,
        },
        // Seat 748: Avacyn, Angel of Horror's EDHREC average deck. `--pod-decks 748`.
        PodDeck {
            name: "Avacyn, Angel of Horror (B)",
            commanders: decks::AVACYN_2_COMMANDERS,
            main: decks::AVACYN_2_MAIN,
        },
        // Seat 749: Tor Wauki the Younger's EDHREC average deck. `--pod-decks 749`.
        PodDeck {
            name: "Tor Wauki the Younger (BR)",
            commanders: decks::TOR_WAUKI_THE_YOUNGER_COMMANDERS,
            main: decks::TOR_WAUKI_THE_YOUNGER_MAIN,
        },
        // Seat 750: Yisan, the Wanderer Bard's EDHREC average deck. `--pod-decks 750`.
        PodDeck {
            name: "Yisan, the Wanderer Bard (G)",
            commanders: decks::YISAN_COMMANDERS,
            main: decks::YISAN_MAIN,
        },
        // Seat 751: Absolute Virtue's EDHREC average deck. `--pod-decks 751`.
        PodDeck {
            name: "Absolute Virtue (WU)",
            commanders: decks::ABSOLUTE_VIRTUE_COMMANDERS,
            main: decks::ABSOLUTE_VIRTUE_MAIN,
        },
        // Seat 752: Kirri, Talented Sprout's EDHREC average deck. `--pod-decks 752`.
        PodDeck {
            name: "Kirri, Talented Sprout (WRG)",
            commanders: decks::KIRRI_COMMANDERS,
            main: decks::KIRRI_MAIN,
        },
        // Seat 753: Commissar Severina Raine's EDHREC average deck. `--pod-decks 753`.
        PodDeck {
            name: "Commissar Severina Raine (WB)",
            commanders: decks::COMMISSAR_SEVERINA_RAINE_COMMANDERS,
            main: decks::COMMISSAR_SEVERINA_RAINE_MAIN,
        },
        // Seat 754: Uro, Titan of Nature's Wrath's EDHREC average deck. `--pod-decks 754`.
        PodDeck {
            name: "Uro, Titan of Nature's Wrath (UG)",
            commanders: decks::URO_COMMANDERS,
            main: decks::URO_MAIN,
        },
        // Seat 755: Phylath, World Sculptor's EDHREC average deck. `--pod-decks 755`.
        PodDeck {
            name: "Phylath, World Sculptor (RG)",
            commanders: decks::PHYLATH_COMMANDERS,
            main: decks::PHYLATH_MAIN,
        },
        // Seat 756: Pearl-Ear, Imperial Advisor's EDHREC average deck. `--pod-decks 756`.
        PodDeck {
            name: "Pearl-Ear, Imperial Advisor (W)",
            commanders: decks::PEARL_EAR_COMMANDERS,
            main: decks::PEARL_EAR_MAIN,
        },
        // Seat 757: Feldon of the Third Path's EDHREC average deck. `--pod-decks 757`.
        PodDeck {
            name: "Feldon of the Third Path (R)",
            commanders: decks::FELDON_OF_THE_THIRD_PATH_COMMANDERS,
            main: decks::FELDON_OF_THE_THIRD_PATH_MAIN,
        },
        // Seat 758: Memnarch's EDHREC average deck. `--pod-decks 758`.
        PodDeck {
            name: "Memnarch (U)",
            commanders: decks::MEMNARCH_COMMANDERS,
            main: decks::MEMNARCH_MAIN,
        },
        // Seat 759: Ishai, Ojutai Dragonspeaker and Reyhan, Last of the Abzan's EDHREC average deck. `--pod-decks 759`.
        PodDeck {
            name: "Ishai, Ojutai Dragonspeaker and Reyhan, Last of the Abzan (WUBG)",
            commanders: decks::ISHAI_COMMANDERS,
            main: decks::ISHAI_MAIN,
        },
        // Seat 760: Sliver Queen's EDHREC average deck. `--pod-decks 760`.
        PodDeck {
            name: "Sliver Queen (WUBRG)",
            commanders: decks::SLIVER_QUEEN_COMMANDERS,
            main: decks::SLIVER_QUEEN_MAIN,
        },
        // Seat 761: Varragoth, Bloodsky Sire's EDHREC average deck. `--pod-decks 761`.
        PodDeck {
            name: "Varragoth, Bloodsky Sire (B)",
            commanders: decks::VARRAGOTH_COMMANDERS,
            main: decks::VARRAGOTH_MAIN,
        },
        // Seat 762: Akul the Unrepentant's EDHREC average deck. `--pod-decks 762`.
        PodDeck {
            name: "Akul the Unrepentant (BR)",
            commanders: decks::AKUL_THE_UNREPENTANT_COMMANDERS,
            main: decks::AKUL_THE_UNREPENTANT_MAIN,
        },
        // Seat 763: Jenny Flint and Madame Vastra's EDHREC average deck. `--pod-decks 763`.
        PodDeck {
            name: "Jenny Flint and Madame Vastra (URG)",
            commanders: decks::JENNY_FLINT_COMMANDERS,
            main: decks::JENNY_FLINT_MAIN,
        },
        // Seat 764: Jadzi, Oracle of Arcavios's EDHREC average deck. `--pod-decks 764`.
        PodDeck {
            name: "Jadzi, Oracle of Arcavios (UG)",
            commanders: decks::JADZI_COMMANDERS,
            main: decks::JADZI_MAIN,
        },
        // Seat 765: Scriv, the Obligator's EDHREC average deck. `--pod-decks 765`.
        PodDeck {
            name: "Scriv, the Obligator (WB)",
            commanders: decks::SCRIV_COMMANDERS,
            main: decks::SCRIV_MAIN,
        },
        // Seat 766: Sygg, River Cutthroat's EDHREC average deck. `--pod-decks 766`.
        PodDeck {
            name: "Sygg, River Cutthroat (UB)",
            commanders: decks::SYGG_COMMANDERS,
            main: decks::SYGG_MAIN,
        },
        // Seat 767: Klothys, God of Destiny's EDHREC average deck. `--pod-decks 767`.
        PodDeck {
            name: "Klothys, God of Destiny (RG)",
            commanders: decks::KLOTHYS_COMMANDERS,
            main: decks::KLOTHYS_MAIN,
        },
        // Seat 768: Preston Garvey, Minuteman's EDHREC average deck. `--pod-decks 768`.
        PodDeck {
            name: "Preston Garvey, Minuteman (WRG)",
            commanders: decks::PRESTON_GARVEY_COMMANDERS,
            main: decks::PRESTON_GARVEY_MAIN,
        },
        // Seat 769: Omnath, Locus of the Void's EDHREC average deck. `--pod-decks 769`.
        PodDeck {
            name: "Omnath, Locus of the Void (C)",
            commanders: decks::OMNATH_COMMANDERS,
            main: decks::OMNATH_MAIN,
        },
        // Seat 770: Zoyowa Lava-Tongue's EDHREC average deck. `--pod-decks 770`.
        PodDeck {
            name: "Zoyowa Lava-Tongue (BR)",
            commanders: decks::ZOYOWA_LAVA_TONGUE_COMMANDERS,
            main: decks::ZOYOWA_LAVA_TONGUE_MAIN,
        },
        // Seat 771: Khârn the Betrayer's EDHREC average deck. `--pod-decks 771`.
        PodDeck {
            name: "Khârn the Betrayer (R)",
            commanders: decks::KHARN_THE_BETRAYER_COMMANDERS,
            main: decks::KHARN_THE_BETRAYER_MAIN,
        },
        // Seat 772: The Valeyard's EDHREC average deck. `--pod-decks 772`.
        PodDeck {
            name: "The Valeyard (UBR)",
            commanders: decks::THE_VALEYARD_COMMANDERS,
            main: decks::THE_VALEYARD_MAIN,
        },
        // Seat 773: Aclazotz, Deepest Betrayal's EDHREC average deck. `--pod-decks 773`.
        PodDeck {
            name: "Aclazotz, Deepest Betrayal (B)",
            commanders: decks::ACLAZOTZ_COMMANDERS,
            main: decks::ACLAZOTZ_MAIN,
        },
        // Seat 774: Ayli, Eternal Pilgrim's EDHREC average deck. `--pod-decks 774`.
        PodDeck {
            name: "Ayli, Eternal Pilgrim (WB)",
            commanders: decks::AYLI_COMMANDERS,
            main: decks::AYLI_MAIN,
        },
        // Seat 775: Celestine, the Living Saint's EDHREC average deck. `--pod-decks 775`.
        PodDeck {
            name: "Celestine, the Living Saint (W)",
            commanders: decks::CELESTINE_COMMANDERS,
            main: decks::CELESTINE_MAIN,
        },
        // Seat 776: Momir Vig, Simic Visionary's EDHREC average deck. `--pod-decks 776`.
        PodDeck {
            name: "Momir Vig, Simic Visionary (UG)",
            commanders: decks::MOMIR_VIG_COMMANDERS,
            main: decks::MOMIR_VIG_MAIN,
        },
        // Seat 777: Lotho, Corrupt Shirriff's EDHREC average deck. `--pod-decks 777`.
        PodDeck {
            name: "Lotho, Corrupt Shirriff (WB)",
            commanders: decks::LOTHO_COMMANDERS,
            main: decks::LOTHO_MAIN,
        },
        // Seat 778: Sram, Senior Edificer's EDHREC average deck. `--pod-decks 778`.
        PodDeck {
            name: "Sram, Senior Edificer (W)",
            commanders: decks::SRAM_COMMANDERS,
            main: decks::SRAM_MAIN,
        },
        // Seat 779: Ojer Taq, Deepest Foundation's EDHREC average deck. `--pod-decks 779`.
        PodDeck {
            name: "Ojer Taq, Deepest Foundation (W)",
            commanders: decks::OJER_TAQ_COMMANDERS,
            main: decks::OJER_TAQ_MAIN,
        },
        // Seat 780: Saheeli, the Sun's Brilliance's EDHREC average deck. `--pod-decks 780`.
        PodDeck {
            name: "Saheeli, the Sun's Brilliance (UR)",
            commanders: decks::SAHEELI_2_COMMANDERS,
            main: decks::SAHEELI_2_MAIN,
        },
        // Seat 781: Vanille, Cheerful l'Cie's EDHREC average deck. `--pod-decks 781`.
        PodDeck {
            name: "Vanille, Cheerful l'Cie (BG)",
            commanders: decks::VANILLE_COMMANDERS,
            main: decks::VANILLE_MAIN,
        },
        // Seat 782: Rith, Liberated Primeval's EDHREC average deck. `--pod-decks 782`.
        PodDeck {
            name: "Rith, Liberated Primeval (WRG)",
            commanders: decks::RITH_COMMANDERS,
            main: decks::RITH_MAIN,
        },
        // Seat 783: Angus Mackenzie's EDHREC average deck. `--pod-decks 783`.
        PodDeck {
            name: "Angus Mackenzie (WUG)",
            commanders: decks::ANGUS_MACKENZIE_COMMANDERS,
            main: decks::ANGUS_MACKENZIE_MAIN,
        },
        // Seat 784: Braids, Cabal Minion's EDHREC average deck. `--pod-decks 784`.
        PodDeck {
            name: "Braids, Cabal Minion (B)",
            commanders: decks::BRAIDS_2_COMMANDERS,
            main: decks::BRAIDS_2_MAIN,
        },
        // Seat 785: Brokkos, Apex of Forever's EDHREC average deck. `--pod-decks 785`.
        PodDeck {
            name: "Brokkos, Apex of Forever (UBG)",
            commanders: decks::BROKKOS_COMMANDERS,
            main: decks::BROKKOS_MAIN,
        },
        // Seat 786: Venser, Corpse Puppet's EDHREC average deck. `--pod-decks 786`.
        PodDeck {
            name: "Venser, Corpse Puppet (UB)",
            commanders: decks::VENSER_COMMANDERS,
            main: decks::VENSER_MAIN,
        },
        // Seat 787: Bladewing, Deathless Tyrant's EDHREC average deck. `--pod-decks 787`.
        PodDeck {
            name: "Bladewing, Deathless Tyrant (BR)",
            commanders: decks::BLADEWING_COMMANDERS,
            main: decks::BLADEWING_MAIN,
        },
        // Seat 788: Drana and Linvala's EDHREC average deck. `--pod-decks 788`.
        PodDeck {
            name: "Drana and Linvala (WB)",
            commanders: decks::DRANA_AND_LINVALA_COMMANDERS,
            main: decks::DRANA_AND_LINVALA_MAIN,
        },
        // Seat 789: Totentanz, Swarm Piper's EDHREC average deck. `--pod-decks 789`.
        PodDeck {
            name: "Totentanz, Swarm Piper (BR)",
            commanders: decks::TOTENTANZ_COMMANDERS,
            main: decks::TOTENTANZ_MAIN,
        },
        // Seat 790: Ashad, the Lone Cyberman's EDHREC average deck. `--pod-decks 790`.
        PodDeck {
            name: "Ashad, the Lone Cyberman (UBR)",
            commanders: decks::ASHAD_COMMANDERS,
            main: decks::ASHAD_MAIN,
        },
        // Seat 791: Cromat's EDHREC average deck. `--pod-decks 791`.
        PodDeck {
            name: "Cromat (WUBRG)",
            commanders: decks::CROMAT_COMMANDERS,
            main: decks::CROMAT_MAIN,
        },
        // Seat 792: Kodama of the West Tree's EDHREC average deck. `--pod-decks 792`.
        PodDeck {
            name: "Kodama of the West Tree (G)",
            commanders: decks::KODAMA_OF_THE_WEST_TREE_COMMANDERS,
            main: decks::KODAMA_OF_THE_WEST_TREE_MAIN,
        },
        // Seat 793: Jegantha, the Wellspring's EDHREC average deck. `--pod-decks 793`.
        PodDeck {
            name: "Jegantha, the Wellspring (WUBRG)",
            commanders: decks::JEGANTHA_COMMANDERS,
            main: decks::JEGANTHA_MAIN,
        },
        // Seat 794: Malcolm, Alluring Scoundrel's EDHREC average deck. `--pod-decks 794`.
        PodDeck {
            name: "Malcolm, Alluring Scoundrel (U)",
            commanders: decks::MALCOLM_COMMANDERS,
            main: decks::MALCOLM_MAIN,
        },
        // Seat 795: Vorel of the Hull Clade's EDHREC average deck. `--pod-decks 795`.
        PodDeck {
            name: "Vorel of the Hull Clade (UG)",
            commanders: decks::VOREL_OF_THE_HULL_CLADE_COMMANDERS,
            main: decks::VOREL_OF_THE_HULL_CLADE_MAIN,
        },
        // Seat 796: Firja, Judge of Valor's EDHREC average deck. `--pod-decks 796`.
        PodDeck {
            name: "Firja, Judge of Valor (WB)",
            commanders: decks::FIRJA_COMMANDERS,
            main: decks::FIRJA_MAIN,
        },
        // Seat 797: The Reality Chip's EDHREC average deck. `--pod-decks 797`.
        PodDeck {
            name: "The Reality Chip (U)",
            commanders: decks::THE_REALITY_CHIP_COMMANDERS,
            main: decks::THE_REALITY_CHIP_MAIN,
        },
        // Seat 798: Shigeki, Jukai Visionary's EDHREC average deck. `--pod-decks 798`.
        PodDeck {
            name: "Shigeki, Jukai Visionary (G)",
            commanders: decks::SHIGEKI_COMMANDERS,
            main: decks::SHIGEKI_MAIN,
        },
        // Seat 799: Ojer Kaslem, Deepest Growth's EDHREC average deck. `--pod-decks 799`.
        PodDeck {
            name: "Ojer Kaslem, Deepest Growth (G)",
            commanders: decks::OJER_KASLEM_COMMANDERS,
            main: decks::OJER_KASLEM_MAIN,
        },
        // Seat 800: Vincent, Vengeful Atoner's EDHREC average deck. `--pod-decks 800`.
        PodDeck {
            name: "Vincent, Vengeful Atoner (R)",
            commanders: decks::VINCENT_COMMANDERS,
            main: decks::VINCENT_MAIN,
        },
        // Seat 801: Melek, Izzet Paragon's EDHREC average deck. `--pod-decks 801`.
        PodDeck {
            name: "Melek, Izzet Paragon (UR)",
            commanders: decks::MELEK_COMMANDERS,
            main: decks::MELEK_MAIN,
        },
        // Seat 802: Byrke, Long Ear of the Law's EDHREC average deck. `--pod-decks 802`.
        PodDeck {
            name: "Byrke, Long Ear of the Law (WG)",
            commanders: decks::BYRKE_COMMANDERS,
            main: decks::BYRKE_MAIN,
        },
        // Seat 803: Vohar, Vodalian Desecrator's EDHREC average deck. `--pod-decks 803`.
        PodDeck {
            name: "Vohar, Vodalian Desecrator (UB)",
            commanders: decks::VOHAR_COMMANDERS,
            main: decks::VOHAR_MAIN,
        },
        // Seat 804: Gisa, Glorious Resurrector's EDHREC average deck. `--pod-decks 804`.
        PodDeck {
            name: "Gisa, Glorious Resurrector (B)",
            commanders: decks::GISA_2_COMMANDERS,
            main: decks::GISA_2_MAIN,
        },
        // Seat 805: Mina and Denn, Wildborn's EDHREC average deck. `--pod-decks 805`.
        PodDeck {
            name: "Mina and Denn, Wildborn (RG)",
            commanders: decks::MINA_AND_DENN_COMMANDERS,
            main: decks::MINA_AND_DENN_MAIN,
        },
        // Seat 806: Akiri, Line-Slinger and Silas Renn, Seeker Adept's EDHREC average deck. `--pod-decks 806`.
        PodDeck {
            name: "Akiri, Line-Slinger and Silas Renn, Seeker Adept (WUBR)",
            commanders: decks::AKIRI_COMMANDERS,
            main: decks::AKIRI_MAIN,
        },
        // Seat 807: Ulamog, the Defiler's EDHREC average deck. `--pod-decks 807`.
        PodDeck {
            name: "Ulamog, the Defiler (C)",
            commanders: decks::ULAMOG_COMMANDERS,
            main: decks::ULAMOG_MAIN,
        },
        // Seat 808: Zellix, Sanity Flayer and Haunted One's EDHREC average deck. `--pod-decks 808`.
        PodDeck {
            name: "Zellix, Sanity Flayer and Haunted One (UB)",
            commanders: decks::ZELLIX_2_COMMANDERS,
            main: decks::ZELLIX_2_MAIN,
        },
        // Seat 809: Kaervek the Merciless's EDHREC average deck. `--pod-decks 809`.
        PodDeck {
            name: "Kaervek the Merciless (BR)",
            commanders: decks::KAERVEK_THE_MERCILESS_COMMANDERS,
            main: decks::KAERVEK_THE_MERCILESS_MAIN,
        },
        // Seat 810: Vadrok, Apex of Thunder's EDHREC average deck. `--pod-decks 810`.
        PodDeck {
            name: "Vadrok, Apex of Thunder (WUR)",
            commanders: decks::VADROK_COMMANDERS,
            main: decks::VADROK_MAIN,
        },
        // Seat 811: Aang, Swift Savior's EDHREC average deck. `--pod-decks 811`.
        PodDeck {
            name: "Aang, Swift Savior (WU)",
            commanders: decks::AANG_2_COMMANDERS,
            main: decks::AANG_2_MAIN,
        },
        // Seat 812: Hermes, Overseer of Elpis's EDHREC average deck. `--pod-decks 812`.
        PodDeck {
            name: "Hermes, Overseer of Elpis (U)",
            commanders: decks::HERMES_COMMANDERS,
            main: decks::HERMES_MAIN,
        },
        // Seat 813: Aang and Katara's EDHREC average deck. `--pod-decks 813`.
        PodDeck {
            name: "Aang and Katara (WUG)",
            commanders: decks::AANG_AND_KATARA_COMMANDERS,
            main: decks::AANG_AND_KATARA_MAIN,
        },
        // Seat 814: Bruna, the Fading Light's EDHREC average deck. `--pod-decks 814`.
        PodDeck {
            name: "Bruna, the Fading Light (W)",
            commanders: decks::BRUNA_COMMANDERS,
            main: decks::BRUNA_MAIN,
        },
        // Seat 815: Erebos, God of the Dead's EDHREC average deck. `--pod-decks 815`.
        PodDeck {
            name: "Erebos, God of the Dead (B)",
            commanders: decks::EREBOS_COMMANDERS,
            main: decks::EREBOS_MAIN,
        },
        // Seat 816: Kefka, Dancing Mad's EDHREC average deck. `--pod-decks 816`.
        PodDeck {
            name: "Kefka, Dancing Mad (BR)",
            commanders: decks::KEFKA_2_COMMANDERS,
            main: decks::KEFKA_2_MAIN,
        },
        // Seat 817: Kodama of the East Tree and Toggo, Goblin Weaponsmith's EDHREC average deck. `--pod-decks 817`.
        PodDeck {
            name: "Kodama of the East Tree and Toggo, Goblin Weaponsmith (RG)",
            commanders: decks::KODAMA_OF_THE_EAST_TREE_COMMANDERS,
            main: decks::KODAMA_OF_THE_EAST_TREE_MAIN,
        },
        // Seat 818: Nissa, Resurgent Animist's EDHREC average deck. `--pod-decks 818`.
        PodDeck {
            name: "Nissa, Resurgent Animist (G)",
            commanders: decks::NISSA_2_COMMANDERS,
            main: decks::NISSA_2_MAIN,
        },
        // Seat 819: Mm'menon, Uthros Exile's EDHREC average deck. `--pod-decks 819`.
        PodDeck {
            name: "Mm'menon, Uthros Exile (UR)",
            commanders: decks::MMMENON_2_COMMANDERS,
            main: decks::MMMENON_2_MAIN,
        },
        // Seat 820: Kodama of the East Tree and Sakashima of a Thousand Faces's EDHREC average deck. `--pod-decks 820`.
        PodDeck {
            name: "Kodama of the East Tree and Sakashima of a Thousand Faces (UG)",
            commanders: decks::KODAMA_OF_THE_EAST_TREE_2_COMMANDERS,
            main: decks::KODAMA_OF_THE_EAST_TREE_2_MAIN,
        },
        // Seat 821: Zimone, All-Questioning's EDHREC average deck. `--pod-decks 821`.
        PodDeck {
            name: "Zimone, All-Questioning (UG)",
            commanders: decks::ZIMONE_3_COMMANDERS,
            main: decks::ZIMONE_3_MAIN,
        },
        // Seat 822: Wasitora, Nekoru Queen's EDHREC average deck. `--pod-decks 822`.
        PodDeck {
            name: "Wasitora, Nekoru Queen (BRG)",
            commanders: decks::WASITORA_COMMANDERS,
            main: decks::WASITORA_MAIN,
        },
        // Seat 823: Belisarius Cawl's EDHREC average deck. `--pod-decks 823`.
        PodDeck {
            name: "Belisarius Cawl (WU)",
            commanders: decks::BELISARIUS_CAWL_COMMANDERS,
            main: decks::BELISARIUS_CAWL_MAIN,
        },
        // Seat 824: Yahenni, Undying Partisan's EDHREC average deck. `--pod-decks 824`.
        PodDeck {
            name: "Yahenni, Undying Partisan (B)",
            commanders: decks::YAHENNI_COMMANDERS,
            main: decks::YAHENNI_MAIN,
        },
        // Seat 825: Ishai, Ojutai Dragonspeaker and Jeska, Thrice Reborn's EDHREC average deck. `--pod-decks 825`.
        PodDeck {
            name: "Ishai, Ojutai Dragonspeaker and Jeska, Thrice Reborn (WUR)",
            commanders: decks::ISHAI_2_COMMANDERS,
            main: decks::ISHAI_2_MAIN,
        },
        // Seat 826: Inga and Esika's EDHREC average deck. `--pod-decks 826`.
        PodDeck {
            name: "Inga and Esika (UG)",
            commanders: decks::INGA_AND_ESIKA_COMMANDERS,
            main: decks::INGA_AND_ESIKA_MAIN,
        },
        // Seat 827: Sin, Unending Cataclysm's EDHREC average deck. `--pod-decks 827`.
        PodDeck {
            name: "Sin, Unending Cataclysm (UG)",
            commanders: decks::SIN_2_COMMANDERS,
            main: decks::SIN_2_MAIN,
        },
        // Seat 828: Nath of the Gilt-Leaf's EDHREC average deck. `--pod-decks 828`.
        PodDeck {
            name: "Nath of the Gilt-Leaf (BG)",
            commanders: decks::NATH_OF_THE_GILT_LEAF_COMMANDERS,
            main: decks::NATH_OF_THE_GILT_LEAF_MAIN,
        },
        // Seat 829: Plargg and Nassari's EDHREC average deck. `--pod-decks 829`.
        PodDeck {
            name: "Plargg and Nassari (R)",
            commanders: decks::PLARGG_AND_NASSARI_COMMANDERS,
            main: decks::PLARGG_AND_NASSARI_MAIN,
        },
        // Seat 830: Zahur, Glory's Past's EDHREC average deck. `--pod-decks 830`.
        PodDeck {
            name: "Zahur, Glory's Past (WB)",
            commanders: decks::ZAHUR_COMMANDERS,
            main: decks::ZAHUR_MAIN,
        },
        // Seat 831: Hugs, Grisly Guardian's EDHREC average deck. `--pod-decks 831`.
        PodDeck {
            name: "Hugs, Grisly Guardian (RG)",
            commanders: decks::HUGS_COMMANDERS,
            main: decks::HUGS_MAIN,
        },
        // Seat 832: Alpharael, Stonechosen's EDHREC average deck. `--pod-decks 832`.
        PodDeck {
            name: "Alpharael, Stonechosen (B)",
            commanders: decks::ALPHARAEL_COMMANDERS,
            main: decks::ALPHARAEL_MAIN,
        },
        // Seat 833: Cait, Cage Brawler's EDHREC average deck. `--pod-decks 833`.
        PodDeck {
            name: "Cait, Cage Brawler (RG)",
            commanders: decks::CAIT_COMMANDERS,
            main: decks::CAIT_MAIN,
        },
        // Seat 834: Nikara, Lair Scavenger and Yannik, Scavenging Sentinel's EDHREC average deck. `--pod-decks 834`.
        PodDeck {
            name: "Nikara, Lair Scavenger and Yannik, Scavenging Sentinel (WBG)",
            commanders: decks::NIKARA_COMMANDERS,
            main: decks::NIKARA_MAIN,
        },
        // Seat 835: Bruse Tarl, Boorish Herder and Ikra Shidiqi, the Usurper's EDHREC average deck. `--pod-decks 835`.
        PodDeck {
            name: "Bruse Tarl, Boorish Herder and Ikra Shidiqi, the Usurper (WBRG)",
            commanders: decks::BRUSE_TARL_COMMANDERS,
            main: decks::BRUSE_TARL_MAIN,
        },
        // Seat 836: Radagast the Brown's EDHREC average deck. `--pod-decks 836`.
        PodDeck {
            name: "Radagast the Brown (G)",
            commanders: decks::RADAGAST_THE_BROWN_COMMANDERS,
            main: decks::RADAGAST_THE_BROWN_MAIN,
        },
        // Seat 837: Rodolf Duskbringer's EDHREC average deck. `--pod-decks 837`.
        PodDeck {
            name: "Rodolf Duskbringer (WB)",
            commanders: decks::RODOLF_DUSKBRINGER_COMMANDERS,
            main: decks::RODOLF_DUSKBRINGER_MAIN,
        },
        // Seat 838: Locke, Treasure Hunter's EDHREC average deck. `--pod-decks 838`.
        PodDeck {
            name: "Locke, Treasure Hunter (BR)",
            commanders: decks::LOCKE_COMMANDERS,
            main: decks::LOCKE_MAIN,
        },
        // Seat 839: Nezahal, Primal Tide's EDHREC average deck. `--pod-decks 839`.
        PodDeck {
            name: "Nezahal, Primal Tide (U)",
            commanders: decks::NEZAHAL_COMMANDERS,
            main: decks::NEZAHAL_MAIN,
        },
        // Seat 840: Sliver Legion's EDHREC average deck. `--pod-decks 840`.
        PodDeck {
            name: "Sliver Legion (WUBRG)",
            commanders: decks::SLIVER_LEGION_COMMANDERS,
            main: decks::SLIVER_LEGION_MAIN,
        },
        // Seat 841: Sanar, Unfinished Genius's EDHREC average deck. `--pod-decks 841`.
        PodDeck {
            name: "Sanar, Unfinished Genius (UR)",
            commanders: decks::SANAR_COMMANDERS,
            main: decks::SANAR_MAIN,
        },
        // Seat 842: Exdeath, Void Warlock's EDHREC average deck. `--pod-decks 842`.
        PodDeck {
            name: "Exdeath, Void Warlock (BG)",
            commanders: decks::EXDEATH_COMMANDERS,
            main: decks::EXDEATH_MAIN,
        },
        // Seat 843: The Raven Man's EDHREC average deck. `--pod-decks 843`.
        PodDeck {
            name: "The Raven Man (B)",
            commanders: decks::THE_RAVEN_MAN_COMMANDERS,
            main: decks::THE_RAVEN_MAN_MAIN,
        },
        // Seat 844: Abuelo, Ancestral Echo's EDHREC average deck. `--pod-decks 844`.
        PodDeck {
            name: "Abuelo, Ancestral Echo (WU)",
            commanders: decks::ABUELO_COMMANDERS,
            main: decks::ABUELO_MAIN,
        },
        // Seat 845: Malik, Grim Manipulator's EDHREC average deck. `--pod-decks 845`.
        PodDeck {
            name: "Malik, Grim Manipulator (WBR)",
            commanders: decks::MALIK_COMMANDERS,
            main: decks::MALIK_MAIN,
        },
        // Seat 846: Etali, Primal Storm's EDHREC average deck. `--pod-decks 846`.
        PodDeck {
            name: "Etali, Primal Storm (R)",
            commanders: decks::ETALI_COMMANDERS,
            main: decks::ETALI_MAIN,
        },
        // Seat 847: Rashmi, Eternities Crafter's EDHREC average deck. `--pod-decks 847`.
        PodDeck {
            name: "Rashmi, Eternities Crafter (UG)",
            commanders: decks::RASHMI_COMMANDERS,
            main: decks::RASHMI_MAIN,
        },
        // Seat 848: Sephara, Sky's Blade's EDHREC average deck. `--pod-decks 848`.
        PodDeck {
            name: "Sephara, Sky's Blade (W)",
            commanders: decks::SEPHARA_COMMANDERS,
            main: decks::SEPHARA_MAIN,
        },
        // Seat 849: The Rani's EDHREC average deck. `--pod-decks 849`.
        PodDeck {
            name: "The Rani (UBR)",
            commanders: decks::THE_RANI_COMMANDERS,
            main: decks::THE_RANI_MAIN,
        },
        // Seat 850: Traxos, Scourge of Kroog's EDHREC average deck. `--pod-decks 850`.
        PodDeck {
            name: "Traxos, Scourge of Kroog (C)",
            commanders: decks::TRAXOS_COMMANDERS,
            main: decks::TRAXOS_MAIN,
        },
        // Seat 851: Momo, Friendly Flier's EDHREC average deck. `--pod-decks 851`.
        PodDeck {
            name: "Momo, Friendly Flier (W)",
            commanders: decks::MOMO_COMMANDERS,
            main: decks::MOMO_MAIN,
        },
        // Seat 852: Toph, Greatest Earthbender's EDHREC average deck. `--pod-decks 852`.
        PodDeck {
            name: "Toph, Greatest Earthbender (RG)",
            commanders: decks::TOPH_2_COMMANDERS,
            main: decks::TOPH_2_MAIN,
        },
        // Seat 853: Stiltzkin, Moogle Merchant's EDHREC average deck. `--pod-decks 853`.
        PodDeck {
            name: "Stiltzkin, Moogle Merchant (W)",
            commanders: decks::STILTZKIN_COMMANDERS,
            main: decks::STILTZKIN_MAIN,
        },
        // Seat 854: Surrak Dragonclaw's EDHREC average deck. `--pod-decks 854`.
        PodDeck {
            name: "Surrak Dragonclaw (URG)",
            commanders: decks::SURRAK_DRAGONCLAW_COMMANDERS,
            main: decks::SURRAK_DRAGONCLAW_MAIN,
        },
        // Seat 855: Mirri, Weatherlight Duelist's EDHREC average deck. `--pod-decks 855`.
        PodDeck {
            name: "Mirri, Weatherlight Duelist (WG)",
            commanders: decks::MIRRI_COMMANDERS,
            main: decks::MIRRI_MAIN,
        },
        // Seat 856: Kangee, Sky Warden's EDHREC average deck. `--pod-decks 856`.
        PodDeck {
            name: "Kangee, Sky Warden (WU)",
            commanders: decks::KANGEE_COMMANDERS,
            main: decks::KANGEE_MAIN,
        },
        // Seat 857: Laelia, the Blade Reforged's EDHREC average deck. `--pod-decks 857`.
        PodDeck {
            name: "Laelia, the Blade Reforged (R)",
            commanders: decks::LAELIA_COMMANDERS,
            main: decks::LAELIA_MAIN,
        },
        // Seat 858: Gorma, the Gullet's EDHREC average deck. `--pod-decks 858`.
        PodDeck {
            name: "Gorma, the Gullet (BG)",
            commanders: decks::GORMA_COMMANDERS,
            main: decks::GORMA_MAIN,
        },
        // Seat 859: Berta, Wise Extrapolator's EDHREC average deck. `--pod-decks 859`.
        PodDeck {
            name: "Berta, Wise Extrapolator (UG)",
            commanders: decks::BERTA_COMMANDERS,
            main: decks::BERTA_MAIN,
        },
        // Seat 860: Bess, Soul Nourisher's EDHREC average deck. `--pod-decks 860`.
        PodDeck {
            name: "Bess, Soul Nourisher (WG)",
            commanders: decks::BESS_COMMANDERS,
            main: decks::BESS_MAIN,
        },
        // Seat 861: MacCready, Lamplight Mayor's EDHREC average deck. `--pod-decks 861`.
        PodDeck {
            name: "MacCready, Lamplight Mayor (WB)",
            commanders: decks::MACCREADY_COMMANDERS,
            main: decks::MACCREADY_MAIN,
        },
        // Seat 862: The Notary Hobbits's EDHREC average deck. `--pod-decks 862`.
        PodDeck {
            name: "The Notary Hobbits (G)",
            commanders: decks::THE_NOTARY_HOBBITS_COMMANDERS,
            main: decks::THE_NOTARY_HOBBITS_MAIN,
        },
        // Seat 863: Samut, Voice of Dissent's EDHREC average deck. `--pod-decks 863`.
        PodDeck {
            name: "Samut, Voice of Dissent (WRG)",
            commanders: decks::SAMUT_2_COMMANDERS,
            main: decks::SAMUT_2_MAIN,
        },
        // Seat 864: Shay Cormac's EDHREC average deck. `--pod-decks 864`.
        PodDeck {
            name: "Shay Cormac (WB)",
            commanders: decks::SHAY_CORMAC_COMMANDERS,
            main: decks::SHAY_CORMAC_MAIN,
        },
        // Seat 865: Emiel the Blessed's EDHREC average deck. `--pod-decks 865`.
        PodDeck {
            name: "Emiel the Blessed (WG)",
            commanders: decks::EMIEL_THE_BLESSED_COMMANDERS,
            main: decks::EMIEL_THE_BLESSED_MAIN,
        },
        // Seat 866: Bhaal, Lord of Murder's EDHREC average deck. `--pod-decks 866`.
        PodDeck {
            name: "Bhaal, Lord of Murder (BRG)",
            commanders: decks::BHAAL_COMMANDERS,
            main: decks::BHAAL_MAIN,
        },
        // Seat 867: Syr Ginger, the Meal Ender's EDHREC average deck. `--pod-decks 867`.
        PodDeck {
            name: "Syr Ginger, the Meal Ender (C)",
            commanders: decks::SYR_GINGER_COMMANDERS,
            main: decks::SYR_GINGER_MAIN,
        },
        // Seat 868: Wan Shi Tong, Librarian's EDHREC average deck. `--pod-decks 868`.
        PodDeck {
            name: "Wan Shi Tong, Librarian (U)",
            commanders: decks::WAN_SHI_TONG_COMMANDERS,
            main: decks::WAN_SHI_TONG_MAIN,
        },
        // Seat 869: Yeva, Nature's Herald's EDHREC average deck. `--pod-decks 869`.
        PodDeck {
            name: "Yeva, Nature's Herald (G)",
            commanders: decks::YEVA_COMMANDERS,
            main: decks::YEVA_MAIN,
        },
        // Seat 870: Chainer, Dementia Master's EDHREC average deck. `--pod-decks 870`.
        PodDeck {
            name: "Chainer, Dementia Master (B)",
            commanders: decks::CHAINER_2_COMMANDERS,
            main: decks::CHAINER_2_MAIN,
        },
        // Seat 871: Moseo, Vein's New Dean's EDHREC average deck. `--pod-decks 871`.
        PodDeck {
            name: "Moseo, Vein's New Dean (B)",
            commanders: decks::MOSEO_COMMANDERS,
            main: decks::MOSEO_MAIN,
        },
        // Seat 872: Kroxa and Kunoros's EDHREC average deck. `--pod-decks 872`.
        PodDeck {
            name: "Kroxa and Kunoros (WBR)",
            commanders: decks::KROXA_AND_KUNOROS_COMMANDERS,
            main: decks::KROXA_AND_KUNOROS_MAIN,
        },
        // Seat 873: Prime Speaker Vannifar's EDHREC average deck. `--pod-decks 873`.
        PodDeck {
            name: "Prime Speaker Vannifar (UG)",
            commanders: decks::PRIME_SPEAKER_VANNIFAR_COMMANDERS,
            main: decks::PRIME_SPEAKER_VANNIFAR_MAIN,
        },
        // Seat 874: Slogurk, the Overslime's EDHREC average deck. `--pod-decks 874`.
        PodDeck {
            name: "Slogurk, the Overslime (UG)",
            commanders: decks::SLOGURK_COMMANDERS,
            main: decks::SLOGURK_MAIN,
        },
        // Seat 875: Hofri Ghostforge's EDHREC average deck. `--pod-decks 875`.
        PodDeck {
            name: "Hofri Ghostforge (WR)",
            commanders: decks::HOFRI_GHOSTFORGE_COMMANDERS,
            main: decks::HOFRI_GHOSTFORGE_MAIN,
        },
        // Seat 876: Shao Jun's EDHREC average deck. `--pod-decks 876`.
        PodDeck {
            name: "Shao Jun (UR)",
            commanders: decks::SHAO_JUN_COMMANDERS,
            main: decks::SHAO_JUN_MAIN,
        },
        // Seat 877: Jalira, Master Polymorphist's EDHREC average deck. `--pod-decks 877`.
        PodDeck {
            name: "Jalira, Master Polymorphist (U)",
            commanders: decks::JALIRA_COMMANDERS,
            main: decks::JALIRA_MAIN,
        },
        // Seat 878: Krenko, Tin Street Kingpin's EDHREC average deck. `--pod-decks 878`.
        PodDeck {
            name: "Krenko, Tin Street Kingpin (R)",
            commanders: decks::KRENKO_COMMANDERS,
            main: decks::KRENKO_MAIN,
        },
        // Seat 879: Kemba, Kha Regent's EDHREC average deck. `--pod-decks 879`.
        PodDeck {
            name: "Kemba, Kha Regent (W)",
            commanders: decks::KEMBA_COMMANDERS,
            main: decks::KEMBA_MAIN,
        },
        // Seat 880: Nymris, Oona's Trickster's EDHREC average deck. `--pod-decks 880`.
        PodDeck {
            name: "Nymris, Oona's Trickster (UB)",
            commanders: decks::NYMRIS_COMMANDERS,
            main: decks::NYMRIS_MAIN,
        },
        // Seat 881: Tariel, Reckoner of Souls's EDHREC average deck. `--pod-decks 881`.
        PodDeck {
            name: "Tariel, Reckoner of Souls (WBR)",
            commanders: decks::TARIEL_COMMANDERS,
            main: decks::TARIEL_MAIN,
        },
        // Seat 882: Greta, Sweettooth Scourge's EDHREC average deck. `--pod-decks 882`.
        PodDeck {
            name: "Greta, Sweettooth Scourge (BG)",
            commanders: decks::GRETA_COMMANDERS,
            main: decks::GRETA_MAIN,
        },
        // Seat 883: Zodiark, Umbral God's EDHREC average deck. `--pod-decks 883`.
        PodDeck {
            name: "Zodiark, Umbral God (B)",
            commanders: decks::ZODIARK_COMMANDERS,
            main: decks::ZODIARK_MAIN,
        },
        // Seat 884: The Cyber-Controller's EDHREC average deck. `--pod-decks 884`.
        PodDeck {
            name: "The Cyber-Controller (UB)",
            commanders: decks::THE_CYBER_CONTROLLER_COMMANDERS,
            main: decks::THE_CYBER_CONTROLLER_MAIN,
        },
        // Seat 885: Rune-Tail, Kitsune Ascendant's EDHREC average deck. `--pod-decks 885`.
        PodDeck {
            name: "Rune-Tail, Kitsune Ascendant (W)",
            commanders: decks::RUNE_TAIL_COMMANDERS,
            main: decks::RUNE_TAIL_MAIN,
        },
        // Seat 886: Lyra Dawnbringer's EDHREC average deck. `--pod-decks 886`.
        PodDeck {
            name: "Lyra Dawnbringer (W)",
            commanders: decks::LYRA_DAWNBRINGER_COMMANDERS,
            main: decks::LYRA_DAWNBRINGER_MAIN,
        },
        // Seat 887: Appa, Steadfast Guardian's EDHREC average deck. `--pod-decks 887`.
        PodDeck {
            name: "Appa, Steadfast Guardian (W)",
            commanders: decks::APPA_COMMANDERS,
            main: decks::APPA_MAIN,
        },
        // Seat 888: Venat, Heart of Hydaelyn's EDHREC average deck. `--pod-decks 888`.
        PodDeck {
            name: "Venat, Heart of Hydaelyn (W)",
            commanders: decks::VENAT_COMMANDERS,
            main: decks::VENAT_MAIN,
        },
        // Seat 889: Ayara, Widow of the Realm's EDHREC average deck. `--pod-decks 889`.
        PodDeck {
            name: "Ayara, Widow of the Realm (BR)",
            commanders: decks::AYARA_2_COMMANDERS,
            main: decks::AYARA_2_MAIN,
        },
        // Seat 890: Duke Ulder Ravengard's EDHREC average deck. `--pod-decks 890`.
        PodDeck {
            name: "Duke Ulder Ravengard (WR)",
            commanders: decks::DUKE_ULDER_RAVENGARD_COMMANDERS,
            main: decks::DUKE_ULDER_RAVENGARD_MAIN,
        },
        // Seat 891: Archangel Avacyn's EDHREC average deck. `--pod-decks 891`.
        PodDeck {
            name: "Archangel Avacyn (WR)",
            commanders: decks::ARCHANGEL_AVACYN_COMMANDERS,
            main: decks::ARCHANGEL_AVACYN_MAIN,
        },
        // Seat 892: Taigam, Ojutai Master's EDHREC average deck. `--pod-decks 892`.
        PodDeck {
            name: "Taigam, Ojutai Master (WU)",
            commanders: decks::TAIGAM_COMMANDERS,
            main: decks::TAIGAM_MAIN,
        },
        // Seat 893: Mica, Reader of Ruins's EDHREC average deck. `--pod-decks 893`.
        PodDeck {
            name: "Mica, Reader of Ruins (R)",
            commanders: decks::MICA_COMMANDERS,
            main: decks::MICA_MAIN,
        },
        // Seat 894: Far Fortune, End Boss's EDHREC average deck. `--pod-decks 894`.
        PodDeck {
            name: "Far Fortune, End Boss (BR)",
            commanders: decks::FAR_FORTUNE_COMMANDERS,
            main: decks::FAR_FORTUNE_MAIN,
        },
        // Seat 895: The Odd Acorn Gang's EDHREC average deck. `--pod-decks 895`.
        PodDeck {
            name: "The Odd Acorn Gang (BG)",
            commanders: decks::THE_ODD_ACORN_GANG_COMMANDERS,
            main: decks::THE_ODD_ACORN_GANG_MAIN,
        },
        // Seat 896: Sarkhan, Dragon Ascendant's EDHREC average deck. `--pod-decks 896`.
        PodDeck {
            name: "Sarkhan, Dragon Ascendant (R)",
            commanders: decks::SARKHAN_COMMANDERS,
            main: decks::SARKHAN_MAIN,
        },
        // Seat 897: Cadric, Soul Kindler's EDHREC average deck. `--pod-decks 897`.
        PodDeck {
            name: "Cadric, Soul Kindler (WR)",
            commanders: decks::CADRIC_COMMANDERS,
            main: decks::CADRIC_MAIN,
        },
        // Seat 898: Haliya, Guided by Light's EDHREC average deck. `--pod-decks 898`.
        PodDeck {
            name: "Haliya, Guided by Light (W)",
            commanders: decks::HALIYA_COMMANDERS,
            main: decks::HALIYA_MAIN,
        },
        // Seat 899: Beza, the Bounding Spring's EDHREC average deck. `--pod-decks 899`.
        PodDeck {
            name: "Beza, the Bounding Spring (W)",
            commanders: decks::BEZA_COMMANDERS,
            main: decks::BEZA_MAIN,
        },
        // Seat 900: Katara, Water Tribe's Hope's EDHREC average deck. `--pod-decks 900`.
        PodDeck {
            name: "Katara, Water Tribe's Hope (WU)",
            commanders: decks::KATARA_2_COMMANDERS,
            main: decks::KATARA_2_MAIN,
        },
        // Seat 901: Karlach, Fury of Avernus and Sword Coast Sailor's EDHREC average deck. `--pod-decks 901`.
        PodDeck {
            name: "Karlach, Fury of Avernus and Sword Coast Sailor (UR)",
            commanders: decks::KARLACH_COMMANDERS,
            main: decks::KARLACH_MAIN,
        },
        // Seat 902: Breeches, the Blastmaker's EDHREC average deck. `--pod-decks 902`.
        PodDeck {
            name: "Breeches, the Blastmaker (UR)",
            commanders: decks::BREECHES_COMMANDERS,
            main: decks::BREECHES_MAIN,
        },
        // Seat 903: Seizan, Perverter of Truth's EDHREC average deck. `--pod-decks 903`.
        PodDeck {
            name: "Seizan, Perverter of Truth (B)",
            commanders: decks::SEIZAN_COMMANDERS,
            main: decks::SEIZAN_MAIN,
        },
        // Seat 904: Jaxis, the Troublemaker's EDHREC average deck. `--pod-decks 904`.
        PodDeck {
            name: "Jaxis, the Troublemaker (R)",
            commanders: decks::JAXIS_COMMANDERS,
            main: decks::JAXIS_MAIN,
        },
        // Seat 905: Tellah, Great Sage's EDHREC average deck. `--pod-decks 905`.
        PodDeck {
            name: "Tellah, Great Sage (UR)",
            commanders: decks::TELLAH_COMMANDERS,
            main: decks::TELLAH_MAIN,
        },
        // Seat 906: Garna, Bloodfist of Keld's EDHREC average deck. `--pod-decks 906`.
        PodDeck {
            name: "Garna, Bloodfist of Keld (BR)",
            commanders: decks::GARNA_COMMANDERS,
            main: decks::GARNA_MAIN,
        },
        // Seat 907: Torens, Fist of the Angels's EDHREC average deck. `--pod-decks 907`.
        PodDeck {
            name: "Torens, Fist of the Angels (WG)",
            commanders: decks::TORENS_COMMANDERS,
            main: decks::TORENS_MAIN,
        },
        // Seat 908: Rith, the Awakener's EDHREC average deck. `--pod-decks 908`.
        PodDeck {
            name: "Rith, the Awakener (WRG)",
            commanders: decks::RITH_2_COMMANDERS,
            main: decks::RITH_2_MAIN,
        },
        // Seat 909: Longshot, Rebel Bowman's EDHREC average deck. `--pod-decks 909`.
        PodDeck {
            name: "Longshot, Rebel Bowman (R)",
            commanders: decks::LONGSHOT_COMMANDERS,
            main: decks::LONGSHOT_MAIN,
        },
        // Seat 910: Cid, Timeless Artificer's EDHREC average deck. `--pod-decks 910`.
        PodDeck {
            name: "Cid, Timeless Artificer (WU)",
            commanders: decks::CID_COMMANDERS,
            main: decks::CID_MAIN,
        },
        // Seat 911: Birgi, God of Storytelling's EDHREC average deck. `--pod-decks 911`.
        PodDeck {
            name: "Birgi, God of Storytelling (R)",
            commanders: decks::BIRGI_COMMANDERS,
            main: decks::BIRGI_MAIN,
        },
        // Seat 912: Snapdax, Apex of the Hunt's EDHREC average deck. `--pod-decks 912`.
        PodDeck {
            name: "Snapdax, Apex of the Hunt (WBR)",
            commanders: decks::SNAPDAX_COMMANDERS,
            main: decks::SNAPDAX_MAIN,
        },
        // Seat 913: Monk Gyatso's EDHREC average deck. `--pod-decks 913`.
        PodDeck {
            name: "Monk Gyatso (W)",
            commanders: decks::MONK_GYATSO_COMMANDERS,
            main: decks::MONK_GYATSO_MAIN,
        },
        // Seat 914: Basim Ibn Ishaq's EDHREC average deck. `--pod-decks 914`.
        PodDeck {
            name: "Basim Ibn Ishaq (UB)",
            commanders: decks::BASIM_IBN_ISHAQ_COMMANDERS,
            main: decks::BASIM_IBN_ISHAQ_MAIN,
        },
        // Seat 915: Iron Spider, Stark Upgrade's EDHREC average deck. `--pod-decks 915`.
        PodDeck {
            name: "Iron Spider, Stark Upgrade (C)",
            commanders: decks::IRON_SPIDER_COMMANDERS,
            main: decks::IRON_SPIDER_MAIN,
        },
        // Seat 916: Piru, the Volatile's EDHREC average deck. `--pod-decks 916`.
        PodDeck {
            name: "Piru, the Volatile (WBR)",
            commanders: decks::PIRU_COMMANDERS,
            main: decks::PIRU_MAIN,
        },
        // Seat 917: Mahadi, Emporium Master's EDHREC average deck. `--pod-decks 917`.
        PodDeck {
            name: "Mahadi, Emporium Master (BR)",
            commanders: decks::MAHADI_COMMANDERS,
            main: decks::MAHADI_MAIN,
        },
        // Seat 918: Zurgo and Ojutai's EDHREC average deck. `--pod-decks 918`.
        PodDeck {
            name: "Zurgo and Ojutai (WUR)",
            commanders: decks::ZURGO_AND_OJUTAI_COMMANDERS,
            main: decks::ZURGO_AND_OJUTAI_MAIN,
        },
        // Seat 919: Kokusho, the Evening Star's EDHREC average deck. `--pod-decks 919`.
        PodDeck {
            name: "Kokusho, the Evening Star (B)",
            commanders: decks::KOKUSHO_COMMANDERS,
            main: decks::KOKUSHO_MAIN,
        },
        // Seat 920: Lagomos, Hand of Hatred's EDHREC average deck. `--pod-decks 920`.
        PodDeck {
            name: "Lagomos, Hand of Hatred (BR)",
            commanders: decks::LAGOMOS_COMMANDERS,
            main: decks::LAGOMOS_MAIN,
        },
        // Seat 921: Inquisitor Eisenhorn's EDHREC average deck. `--pod-decks 921`.
        PodDeck {
            name: "Inquisitor Eisenhorn (UB)",
            commanders: decks::INQUISITOR_EISENHORN_COMMANDERS,
            main: decks::INQUISITOR_EISENHORN_MAIN,
        },
        // Seat 922: Oswald Fiddlebender's EDHREC average deck. `--pod-decks 922`.
        PodDeck {
            name: "Oswald Fiddlebender (W)",
            commanders: decks::OSWALD_FIDDLEBENDER_COMMANDERS,
            main: decks::OSWALD_FIDDLEBENDER_MAIN,
        },
        // Seat 923: Abomination of Llanowar's EDHREC average deck. `--pod-decks 923`.
        PodDeck {
            name: "Abomination of Llanowar (BG)",
            commanders: decks::ABOMINATION_OF_LLANOWAR_COMMANDERS,
            main: decks::ABOMINATION_OF_LLANOWAR_MAIN,
        },
        // Seat 924: Skrelv, Defector Mite's EDHREC average deck. `--pod-decks 924`.
        PodDeck {
            name: "Skrelv, Defector Mite (W)",
            commanders: decks::SKRELV_COMMANDERS,
            main: decks::SKRELV_MAIN,
        },
        // Seat 925: Orthion, Hero of Lavabrink's EDHREC average deck. `--pod-decks 925`.
        PodDeck {
            name: "Orthion, Hero of Lavabrink (R)",
            commanders: decks::ORTHION_COMMANDERS,
            main: decks::ORTHION_MAIN,
        },
        // Seat 926: Olag and Miau, New Friends's EDHREC average deck. `--pod-decks 926`.
        PodDeck {
            name: "Olag and Miau, New Friends (WBG)",
            commanders: decks::OLAG_AND_MIAU_COMMANDERS,
            main: decks::OLAG_AND_MIAU_MAIN,
        },
        // Seat 927: Lagrella, the Magpie's EDHREC average deck. `--pod-decks 927`.
        PodDeck {
            name: "Lagrella, the Magpie (WUG)",
            commanders: decks::LAGRELLA_COMMANDERS,
            main: decks::LAGRELLA_MAIN,
        },
        // Seat 928: Caradora, Heart of Alacria's EDHREC average deck. `--pod-decks 928`.
        PodDeck {
            name: "Caradora, Heart of Alacria (WG)",
            commanders: decks::CARADORA_COMMANDERS,
            main: decks::CARADORA_MAIN,
        },
        // Seat 929: Ezuri, Stalker of Spheres's EDHREC average deck. `--pod-decks 929`.
        PodDeck {
            name: "Ezuri, Stalker of Spheres (UG)",
            commanders: decks::EZURI_3_COMMANDERS,
            main: decks::EZURI_3_MAIN,
        },
        // Seat 930: Toluz, Clever Conductor's EDHREC average deck. `--pod-decks 930`.
        PodDeck {
            name: "Toluz, Clever Conductor (WUB)",
            commanders: decks::TOLUZ_COMMANDERS,
            main: decks::TOLUZ_MAIN,
        },
        // Seat 931: Urianger Augurelt's EDHREC average deck. `--pod-decks 931`.
        PodDeck {
            name: "Urianger Augurelt (WU)",
            commanders: decks::URIANGER_AUGURELT_COMMANDERS,
            main: decks::URIANGER_AUGURELT_MAIN,
        },
        // Seat 932: Maelstrom Wanderer's EDHREC average deck. `--pod-decks 932`.
        PodDeck {
            name: "Maelstrom Wanderer (URG)",
            commanders: decks::MAELSTROM_WANDERER_COMMANDERS,
            main: decks::MAELSTROM_WANDERER_MAIN,
        },
        // Seat 933: Shirei, Shizo's Caretaker's EDHREC average deck. `--pod-decks 933`.
        PodDeck {
            name: "Shirei, Shizo's Caretaker (B)",
            commanders: decks::SHIREI_COMMANDERS,
            main: decks::SHIREI_MAIN,
        },
        // Seat 934: Gluntch, the Bestower's EDHREC average deck. `--pod-decks 934`.
        PodDeck {
            name: "Gluntch, the Bestower (WG)",
            commanders: decks::GLUNTCH_COMMANDERS,
            main: decks::GLUNTCH_MAIN,
        },
        // Seat 935: Zoraline, Cosmos Caller's EDHREC average deck. `--pod-decks 935`.
        PodDeck {
            name: "Zoraline, Cosmos Caller (WB)",
            commanders: decks::ZORALINE_COMMANDERS,
            main: decks::ZORALINE_MAIN,
        },
        // Seat 936: Raph & Mikey, Troublemakers's EDHREC average deck. `--pod-decks 936`.
        PodDeck {
            name: "Raph & Mikey, Troublemakers (RG)",
            commanders: decks::RAPH_MIKEY_COMMANDERS,
            main: decks::RAPH_MIKEY_MAIN,
        },
        // Seat 937: The Gitrog, Ravenous Ride's EDHREC average deck. `--pod-decks 937`.
        PodDeck {
            name: "The Gitrog, Ravenous Ride (BG)",
            commanders: decks::THE_GITROG_COMMANDERS,
            main: decks::THE_GITROG_MAIN,
        },
        // Seat 938: Thantis, the Warweaver's EDHREC average deck. `--pod-decks 938`.
        PodDeck {
            name: "Thantis, the Warweaver (BRG)",
            commanders: decks::THANTIS_COMMANDERS,
            main: decks::THANTIS_MAIN,
        },
        // Seat 939: Pramikon, Sky Rampart's EDHREC average deck. `--pod-decks 939`.
        PodDeck {
            name: "Pramikon, Sky Rampart (WUR)",
            commanders: decks::PRAMIKON_COMMANDERS,
            main: decks::PRAMIKON_MAIN,
        },
        // Seat 940: Ral, Monsoon Mage's EDHREC average deck. `--pod-decks 940`.
        PodDeck {
            name: "Ral, Monsoon Mage (UR)",
            commanders: decks::RAL_COMMANDERS,
            main: decks::RAL_MAIN,
        },
        // Seat 941: Ruric Thar, the Unbowed's EDHREC average deck. `--pod-decks 941`.
        PodDeck {
            name: "Ruric Thar, the Unbowed (RG)",
            commanders: decks::RURIC_THAR_COMMANDERS,
            main: decks::RURIC_THAR_MAIN,
        },
        // Seat 942: Ognis, the Dragon's Lash's EDHREC average deck. `--pod-decks 942`.
        PodDeck {
            name: "Ognis, the Dragon's Lash (BRG)",
            commanders: decks::OGNIS_COMMANDERS,
            main: decks::OGNIS_MAIN,
        },
        // Seat 943: Zethi, Arcane Blademaster's EDHREC average deck. `--pod-decks 943`.
        PodDeck {
            name: "Zethi, Arcane Blademaster (WU)",
            commanders: decks::ZETHI_COMMANDERS,
            main: decks::ZETHI_MAIN,
        },
        // Seat 944: Karametra, God of Harvests's EDHREC average deck. `--pod-decks 944`.
        PodDeck {
            name: "Karametra, God of Harvests (WG)",
            commanders: decks::KARAMETRA_COMMANDERS,
            main: decks::KARAMETRA_MAIN,
        },
        // Seat 945: Heliod, the Radiant Dawn's EDHREC average deck. `--pod-decks 945`.
        PodDeck {
            name: "Heliod, the Radiant Dawn (WU)",
            commanders: decks::HELIOD_2_COMMANDERS,
            main: decks::HELIOD_2_MAIN,
        },
        // Seat 946: Minn, Wily Illusionist's EDHREC average deck. `--pod-decks 946`.
        PodDeck {
            name: "Minn, Wily Illusionist (U)",
            commanders: decks::MINN_COMMANDERS,
            main: decks::MINN_MAIN,
        },
        // Seat 947: Karn, Legacy Reforged's EDHREC average deck. `--pod-decks 947`.
        PodDeck {
            name: "Karn, Legacy Reforged (C)",
            commanders: decks::KARN_COMMANDERS,
            main: decks::KARN_MAIN,
        },
        // Seat 948: Aeve, Progenitor Ooze's EDHREC average deck. `--pod-decks 948`.
        PodDeck {
            name: "Aeve, Progenitor Ooze (G)",
            commanders: decks::AEVE_COMMANDERS,
            main: decks::AEVE_MAIN,
        },
        // Seat 949: Kelsien, the Plague's EDHREC average deck. `--pod-decks 949`.
        PodDeck {
            name: "Kelsien, the Plague (WBR)",
            commanders: decks::KELSIEN_COMMANDERS,
            main: decks::KELSIEN_MAIN,
        },
        // Seat 950: Mister Negative's EDHREC average deck. `--pod-decks 950`.
        PodDeck {
            name: "Mister Negative (WB)",
            commanders: decks::MISTER_NEGATIVE_COMMANDERS,
            main: decks::MISTER_NEGATIVE_MAIN,
        },
        // Seat 951: Imoti, Celebrant of Bounty's EDHREC average deck. `--pod-decks 951`.
        PodDeck {
            name: "Imoti, Celebrant of Bounty (UG)",
            commanders: decks::IMOTI_COMMANDERS,
            main: decks::IMOTI_MAIN,
        },
        // Seat 952: Averna, the Chaos Bloom's EDHREC average deck. `--pod-decks 952`.
        PodDeck {
            name: "Averna, the Chaos Bloom (URG)",
            commanders: decks::AVERNA_COMMANDERS,
            main: decks::AVERNA_MAIN,
        },
        // Seat 953: Ashling, Rekindled's EDHREC average deck. `--pod-decks 953`.
        PodDeck {
            name: "Ashling, Rekindled (UR)",
            commanders: decks::ASHLING_2_COMMANDERS,
            main: decks::ASHLING_2_MAIN,
        },
        // Seat 954: Kenessos, Priest of Thassa's EDHREC average deck. `--pod-decks 954`.
        PodDeck {
            name: "Kenessos, Priest of Thassa (UG)",
            commanders: decks::KENESSOS_COMMANDERS,
            main: decks::KENESSOS_MAIN,
        },
        // Seat 955: Aphelia, Viper Whisperer's EDHREC average deck. `--pod-decks 955`.
        PodDeck {
            name: "Aphelia, Viper Whisperer (BG)",
            commanders: decks::APHELIA_COMMANDERS,
            main: decks::APHELIA_MAIN,
        },
        // Seat 956: Baba Lysaga, Night Witch's EDHREC average deck. `--pod-decks 956`.
        PodDeck {
            name: "Baba Lysaga, Night Witch (BG)",
            commanders: decks::BABA_LYSAGA_COMMANDERS,
            main: decks::BABA_LYSAGA_MAIN,
        },
        // Seat 957: Kros, Defense Contractor's EDHREC average deck. `--pod-decks 957`.
        PodDeck {
            name: "Kros, Defense Contractor (WUG)",
            commanders: decks::KROS_COMMANDERS,
            main: decks::KROS_MAIN,
        },
        // Seat 958: Ultron, Artificial Malevolence's EDHREC average deck. `--pod-decks 958`.
        PodDeck {
            name: "Ultron, Artificial Malevolence (C)",
            commanders: decks::ULTRON_COMMANDERS,
            main: decks::ULTRON_MAIN,
        },
        // Seat 959: The Council of Four's EDHREC average deck. `--pod-decks 959`.
        PodDeck {
            name: "The Council of Four (WU)",
            commanders: decks::THE_COUNCIL_OF_FOUR_COMMANDERS,
            main: decks::THE_COUNCIL_OF_FOUR_MAIN,
        },
        // Seat 960: Odric, Lunarch Marshal's EDHREC average deck. `--pod-decks 960`.
        PodDeck {
            name: "Odric, Lunarch Marshal (W)",
            commanders: decks::ODRIC_COMMANDERS,
            main: decks::ODRIC_MAIN,
        },
        // Seat 961: Obosh, the Preypiercer's EDHREC average deck. `--pod-decks 961`.
        PodDeck {
            name: "Obosh, the Preypiercer (BR)",
            commanders: decks::OBOSH_COMMANDERS,
            main: decks::OBOSH_MAIN,
        },
        // Seat 962: Amareth, the Lustrous's EDHREC average deck. `--pod-decks 962`.
        PodDeck {
            name: "Amareth, the Lustrous (WUG)",
            commanders: decks::AMARETH_COMMANDERS,
            main: decks::AMARETH_MAIN,
        },
        // Seat 963: Cecil, Dark Knight's EDHREC average deck. `--pod-decks 963`.
        PodDeck {
            name: "Cecil, Dark Knight (WB)",
            commanders: decks::CECIL_COMMANDERS,
            main: decks::CECIL_MAIN,
        },
        // Seat 964: Muxus, Goblin Grandee's EDHREC average deck. `--pod-decks 964`.
        PodDeck {
            name: "Muxus, Goblin Grandee (R)",
            commanders: decks::MUXUS_COMMANDERS,
            main: decks::MUXUS_MAIN,
        },
        // Seat 965: Minwu, White Mage's EDHREC average deck. `--pod-decks 965`.
        PodDeck {
            name: "Minwu, White Mage (W)",
            commanders: decks::MINWU_COMMANDERS,
            main: decks::MINWU_MAIN,
        },
        // Seat 966: Gylwain, Casting Director's EDHREC average deck. `--pod-decks 966`.
        PodDeck {
            name: "Gylwain, Casting Director (WG)",
            commanders: decks::GYLWAIN_COMMANDERS,
            main: decks::GYLWAIN_MAIN,
        },
        // Seat 967: Ghalta and Mavren's EDHREC average deck. `--pod-decks 967`.
        PodDeck {
            name: "Ghalta and Mavren (WG)",
            commanders: decks::GHALTA_AND_MAVREN_COMMANDERS,
            main: decks::GHALTA_AND_MAVREN_MAIN,
        },
        // Seat 968: King T'Challa's EDHREC average deck. `--pod-decks 968`.
        PodDeck {
            name: "King T'Challa (WU)",
            commanders: decks::KING_TCHALLA_COMMANDERS,
            main: decks::KING_TCHALLA_MAIN,
        },
        // Seat 969: Mikaeus, the Unhallowed's EDHREC average deck. `--pod-decks 969`.
        PodDeck {
            name: "Mikaeus, the Unhallowed (B)",
            commanders: decks::MIKAEUS_COMMANDERS,
            main: decks::MIKAEUS_MAIN,
        },
        // Seat 970: Taborax, Hope's Demise's EDHREC average deck. `--pod-decks 970`.
        PodDeck {
            name: "Taborax, Hope's Demise (B)",
            commanders: decks::TABORAX_COMMANDERS,
            main: decks::TABORAX_MAIN,
        },
        // Seat 971: Satoru, the Infiltrator's EDHREC average deck. `--pod-decks 971`.
        PodDeck {
            name: "Satoru, the Infiltrator (UB)",
            commanders: decks::SATORU_2_COMMANDERS,
            main: decks::SATORU_2_MAIN,
        },
        // Seat 972: Arjun, the Shifting Flame's EDHREC average deck. `--pod-decks 972`.
        PodDeck {
            name: "Arjun, the Shifting Flame (UR)",
            commanders: decks::ARJUN_COMMANDERS,
            main: decks::ARJUN_MAIN,
        },
        // Seat 973: Oona, Queen of the Fae's EDHREC average deck. `--pod-decks 973`.
        PodDeck {
            name: "Oona, Queen of the Fae (UB)",
            commanders: decks::OONA_COMMANDERS,
            main: decks::OONA_MAIN,
        },
        // Seat 974: Gollum, Obsessed Stalker's EDHREC average deck. `--pod-decks 974`.
        PodDeck {
            name: "Gollum, Obsessed Stalker (B)",
            commanders: decks::GOLLUM_COMMANDERS,
            main: decks::GOLLUM_MAIN,
        },
        // Seat 975: Eruth, Tormented Prophet's EDHREC average deck. `--pod-decks 975`.
        PodDeck {
            name: "Eruth, Tormented Prophet (UR)",
            commanders: decks::ERUTH_COMMANDERS,
            main: decks::ERUTH_MAIN,
        },
        // Seat 976: Captain America, Living Legend's EDHREC average deck. `--pod-decks 976`.
        PodDeck {
            name: "Captain America, Living Legend (WU)",
            commanders: decks::CAPTAIN_AMERICA_2_COMMANDERS,
            main: decks::CAPTAIN_AMERICA_2_MAIN,
        },
        // Seat 977: Gretchen Titchwillow's EDHREC average deck. `--pod-decks 977`.
        PodDeck {
            name: "Gretchen Titchwillow (UG)",
            commanders: decks::GRETCHEN_TITCHWILLOW_COMMANDERS,
            main: decks::GRETCHEN_TITCHWILLOW_MAIN,
        },
        // Seat 978: Super-Skrull's EDHREC average deck. `--pod-decks 978`.
        PodDeck {
            name: "Super-Skrull (WUBRG)",
            commanders: decks::SUPER_SKRULL_COMMANDERS,
            main: decks::SUPER_SKRULL_MAIN,
        },
        // Seat 979: Godo, Bandit Warlord's EDHREC average deck. `--pod-decks 979`.
        PodDeck {
            name: "Godo, Bandit Warlord (R)",
            commanders: decks::GODO_COMMANDERS,
            main: decks::GODO_MAIN,
        },
        // Seat 980: Mortarion, Daemon Primarch's EDHREC average deck. `--pod-decks 980`.
        PodDeck {
            name: "Mortarion, Daemon Primarch (B)",
            commanders: decks::MORTARION_COMMANDERS,
            main: decks::MORTARION_MAIN,
        },
        // Seat 981: Sek'Kuar, Deathkeeper's EDHREC average deck. `--pod-decks 981`.
        PodDeck {
            name: "Sek'Kuar, Deathkeeper (BRG)",
            commanders: decks::SEKKUAR_COMMANDERS,
            main: decks::SEKKUAR_MAIN,
        },
        // Seat 982: Gallia of the Endless Dance's EDHREC average deck. `--pod-decks 982`.
        PodDeck {
            name: "Gallia of the Endless Dance (RG)",
            commanders: decks::GALLIA_OF_THE_ENDLESS_DANCE_COMMANDERS,
            main: decks::GALLIA_OF_THE_ENDLESS_DANCE_MAIN,
        },
        // Seat 983: Anje, Maid of Dishonor's EDHREC average deck. `--pod-decks 983`.
        PodDeck {
            name: "Anje, Maid of Dishonor (BR)",
            commanders: decks::ANJE_2_COMMANDERS,
            main: decks::ANJE_2_MAIN,
        },
        // Seat 984: Edgar, Charmed Groom's EDHREC average deck. `--pod-decks 984`.
        PodDeck {
            name: "Edgar, Charmed Groom (WB)",
            commanders: decks::EDGAR_2_COMMANDERS,
            main: decks::EDGAR_2_MAIN,
        },
        // Seat 985: Lazav, the Multifarious's EDHREC average deck. `--pod-decks 985`.
        PodDeck {
            name: "Lazav, the Multifarious (UB)",
            commanders: decks::LAZAV_2_COMMANDERS,
            main: decks::LAZAV_2_MAIN,
        },
        // Seat 986: Eight-and-a-Half-Tails's EDHREC average deck. `--pod-decks 986`.
        PodDeck {
            name: "Eight-and-a-Half-Tails (W)",
            commanders: decks::EIGHT_AND_A_HALF_TAILS_COMMANDERS,
            main: decks::EIGHT_AND_A_HALF_TAILS_MAIN,
        },
        // Seat 987: Quintorius, Loremaster's EDHREC average deck. `--pod-decks 987`.
        PodDeck {
            name: "Quintorius, Loremaster (WR)",
            commanders: decks::QUINTORIUS_2_COMMANDERS,
            main: decks::QUINTORIUS_2_MAIN,
        },
        // Seat 988: Tataru Taru's EDHREC average deck. `--pod-decks 988`.
        PodDeck {
            name: "Tataru Taru (W)",
            commanders: decks::TATARU_TARU_COMMANDERS,
            main: decks::TATARU_TARU_MAIN,
        },
        // Seat 989: Black Cat, Cunning Thief's EDHREC average deck. `--pod-decks 989`.
        PodDeck {
            name: "Black Cat, Cunning Thief (B)",
            commanders: decks::BLACK_CAT_COMMANDERS,
            main: decks::BLACK_CAT_MAIN,
        },
        // Seat 990: Dragonlord Ojutai's EDHREC average deck. `--pod-decks 990`.
        PodDeck {
            name: "Dragonlord Ojutai (WU)",
            commanders: decks::DRAGONLORD_OJUTAI_COMMANDERS,
            main: decks::DRAGONLORD_OJUTAI_MAIN,
        },
        // Seat 991: Grumgully, the Generous's EDHREC average deck. `--pod-decks 991`.
        PodDeck {
            name: "Grumgully, the Generous (RG)",
            commanders: decks::GRUMGULLY_COMMANDERS,
            main: decks::GRUMGULLY_MAIN,
        },
        // Seat 992: Wylie Duke, Atiin Hero's EDHREC average deck. `--pod-decks 992`.
        PodDeck {
            name: "Wylie Duke, Atiin Hero (WG)",
            commanders: decks::WYLIE_DUKE_COMMANDERS,
            main: decks::WYLIE_DUKE_MAIN,
        },
        // Seat 993: Yue, the Moon Spirit's EDHREC average deck. `--pod-decks 993`.
        PodDeck {
            name: "Yue, the Moon Spirit (U)",
            commanders: decks::YUE_COMMANDERS,
            main: decks::YUE_MAIN,
        },
        // Seat 994: Velomachus Lorehold's EDHREC average deck. `--pod-decks 994`.
        PodDeck {
            name: "Velomachus Lorehold (WR)",
            commanders: decks::VELOMACHUS_LOREHOLD_COMMANDERS,
            main: decks::VELOMACHUS_LOREHOLD_MAIN,
        },
        // Seat 995: Akiri, Fearless Voyager's EDHREC average deck. `--pod-decks 995`.
        PodDeck {
            name: "Akiri, Fearless Voyager (WR)",
            commanders: decks::AKIRI_2_COMMANDERS,
            main: decks::AKIRI_2_MAIN,
        },
        // Seat 996: Nicol Bolas's EDHREC average deck. `--pod-decks 996`.
        PodDeck {
            name: "Nicol Bolas (UBR)",
            commanders: decks::NICOL_BOLAS_COMMANDERS,
            main: decks::NICOL_BOLAS_MAIN,
        },
        // Seat 997: Thassa, Deep-Dwelling's EDHREC average deck. `--pod-decks 997`.
        PodDeck {
            name: "Thassa, Deep-Dwelling (U)",
            commanders: decks::THASSA_COMMANDERS,
            main: decks::THASSA_MAIN,
        },
        // Seat 998: Doctor Doom, Unrivaled's EDHREC average deck. `--pod-decks 998`.
        PodDeck {
            name: "Doctor Doom, Unrivaled (B)",
            commanders: decks::DOCTOR_DOOM_COMMANDERS,
            main: decks::DOCTOR_DOOM_MAIN,
        },
        // Seat 999: Toralf, God of Fury's EDHREC average deck. `--pod-decks 999`.
        PodDeck {
            name: "Toralf, God of Fury (R)",
            commanders: decks::TORALF_COMMANDERS,
            main: decks::TORALF_MAIN,
        },
        // Seat 1000: Magda, the Hoardmaster's EDHREC average deck. `--pod-decks 1000`.
        PodDeck {
            name: "Magda, the Hoardmaster (R)",
            commanders: decks::MAGDA_2_COMMANDERS,
            main: decks::MAGDA_2_MAIN,
        },
        // Seat 1001: Ghalta, Stampede Tyrant's EDHREC average deck. `--pod-decks 1001`.
        PodDeck {
            name: "Ghalta, Stampede Tyrant (G)",
            commanders: decks::GHALTA_2_COMMANDERS,
            main: decks::GHALTA_2_MAIN,
        },
        // Seat 1002: Kain, Traitorous Dragoon's EDHREC average deck. `--pod-decks 1002`.
        PodDeck {
            name: "Kain, Traitorous Dragoon (B)",
            commanders: decks::KAIN_COMMANDERS,
            main: decks::KAIN_MAIN,
        },
        // Seat 1003: Anzrag, the Quake-Mole's EDHREC average deck. `--pod-decks 1003`.
        PodDeck {
            name: "Anzrag, the Quake-Mole (RG)",
            commanders: decks::ANZRAG_COMMANDERS,
            main: decks::ANZRAG_MAIN,
        },
        // Seat 1004: Ardbert, Warrior of Darkness's EDHREC average deck. `--pod-decks 1004`.
        PodDeck {
            name: "Ardbert, Warrior of Darkness (WB)",
            commanders: decks::ARDBERT_COMMANDERS,
            main: decks::ARDBERT_MAIN,
        },
        // Seat 1005: Fire Lord Ozai's EDHREC average deck. `--pod-decks 1005`.
        PodDeck {
            name: "Fire Lord Ozai (BR)",
            commanders: decks::FIRE_LORD_OZAI_COMMANDERS,
            main: decks::FIRE_LORD_OZAI_MAIN,
        },
        // Seat 1006: Hinata, Dawn-Crowned's EDHREC average deck. `--pod-decks 1006`.
        PodDeck {
            name: "Hinata, Dawn-Crowned (WUR)",
            commanders: decks::HINATA_COMMANDERS,
            main: decks::HINATA_MAIN,
        },
        // Seat 1007: Ian Malcolm, Chaotician's EDHREC average deck. `--pod-decks 1007`.
        PodDeck {
            name: "Ian Malcolm, Chaotician (UR)",
            commanders: decks::IAN_MALCOLM_COMMANDERS,
            main: decks::IAN_MALCOLM_MAIN,
        },
        // Seat 1008: Imodane, the Pyrohammer's EDHREC average deck. `--pod-decks 1008`.
        PodDeck {
            name: "Imodane, the Pyrohammer (R)",
            commanders: decks::IMODANE_COMMANDERS,
            main: decks::IMODANE_MAIN,
        },
        // Seat 1009: Ivy, Gleeful Spellthief's EDHREC average deck. `--pod-decks 1009`.
        PodDeck {
            name: "Ivy, Gleeful Spellthief (UG)",
            commanders: decks::IVY_COMMANDERS,
            main: decks::IVY_MAIN,
        },
        // Seat 1010: Maarika, Brutal Gladiator's EDHREC average deck. `--pod-decks 1010`.
        PodDeck {
            name: "Maarika, Brutal Gladiator (BRG)",
            commanders: decks::MAARIKA_COMMANDERS,
            main: decks::MAARIKA_MAIN,
        },
        // Seat 1011: Megatron, Tyrant's EDHREC average deck. `--pod-decks 1011`.
        PodDeck {
            name: "Megatron, Tyrant (WBR)",
            commanders: decks::MEGATRON_COMMANDERS,
            main: decks::MEGATRON_MAIN,
        },
        // Seat 1012: Saruman of Many Colors's EDHREC average deck. `--pod-decks 1012`.
        PodDeck {
            name: "Saruman of Many Colors (WUB)",
            commanders: decks::SARUMAN_MANY_COLORS_COMMANDERS,
            main: decks::SARUMAN_MANY_COLORS_MAIN,
        },
        // Seat 1013: Heartless Hidetsugu's EDHREC average deck. `--pod-decks 1013`.
        PodDeck {
            name: "Heartless Hidetsugu (R)",
            commanders: decks::HEARTLESS_HIDETSUGU_COMMANDERS,
            main: decks::HEARTLESS_HIDETSUGU_MAIN,
        },
        // Seat 1014: Chromium, the Mutable's EDHREC average deck. `--pod-decks 1014`.
        PodDeck {
            name: "Chromium, the Mutable (WUB)",
            commanders: decks::CHROMIUM_COMMANDERS,
            main: decks::CHROMIUM_MAIN,
        },
        // Seat 1015: Themberchaud's EDHREC average deck. `--pod-decks 1015`.
        PodDeck {
            name: "Themberchaud (R)",
            commanders: decks::THEMBERCHAUD_COMMANDERS,
            main: decks::THEMBERCHAUD_MAIN,
        },
        // Seat 1016: Emet-Selch of the Third Seat's EDHREC average deck. `--pod-decks 1016`.
        PodDeck {
            name: "Emet-Selch of the Third Seat (UB)",
            commanders: decks::EMET_SELCH_OF_THE_THIRD_SEAT_COMMANDERS,
            main: decks::EMET_SELCH_OF_THE_THIRD_SEAT_MAIN,
        },
        // Seat 1017: Erinis, Gloom Stalker and Street Urchin's EDHREC average deck. `--pod-decks 1017`.
        PodDeck {
            name: "Erinis, Gloom Stalker and Street Urchin (RG)",
            commanders: decks::ERINIS_COMMANDERS,
            main: decks::ERINIS_MAIN,
        },
        // Seat 1018: Sarkhan, Soul Aflame's EDHREC average deck. `--pod-decks 1018`.
        PodDeck {
            name: "Sarkhan, Soul Aflame (UR)",
            commanders: decks::SARKHAN_2_COMMANDERS,
            main: decks::SARKHAN_2_MAIN,
        },
        // Seat 1019: Fandaniel, Telophoroi Ascian's EDHREC average deck. `--pod-decks 1019`.
        PodDeck {
            name: "Fandaniel, Telophoroi Ascian (B)",
            commanders: decks::FANDANIEL_COMMANDERS,
            main: decks::FANDANIEL_MAIN,
        },
        // Seat 1020: Tawnos, the Toymaker's EDHREC average deck. `--pod-decks 1020`.
        PodDeck {
            name: "Tawnos, the Toymaker (UG)",
            commanders: decks::TAWNOS_COMMANDERS,
            main: decks::TAWNOS_MAIN,
        },
        // Seat 1021: Alistair, the Brigadier's EDHREC average deck. `--pod-decks 1021`.
        PodDeck {
            name: "Alistair, the Brigadier (WUG)",
            commanders: decks::ALISTAIR_COMMANDERS,
            main: decks::ALISTAIR_MAIN,
        },
        // Seat 1022: Rigo, Streetwise Mentor's EDHREC average deck. `--pod-decks 1022`.
        PodDeck {
            name: "Rigo, Streetwise Mentor (WUG)",
            commanders: decks::RIGO_COMMANDERS,
            main: decks::RIGO_MAIN,
        },
        // Seat 1023: Krav, the Unredeemed and Regna, the Redeemer's EDHREC average deck. `--pod-decks 1023`.
        PodDeck {
            name: "Krav, the Unredeemed and Regna, the Redeemer (WB)",
            commanders: decks::KRAV_COMMANDERS,
            main: decks::KRAV_MAIN,
        },
        // Seat 1024: Queen Allenal of Ruadach's EDHREC average deck. `--pod-decks 1024`.
        PodDeck {
            name: "Queen Allenal of Ruadach (WG)",
            commanders: decks::QUEEN_ALLENAL_OF_RUADACH_COMMANDERS,
            main: decks::QUEEN_ALLENAL_OF_RUADACH_MAIN,
        },
        // Seat 1025: Lo and Li, Royal Advisors's EDHREC average deck. `--pod-decks 1025`.
        PodDeck {
            name: "Lo and Li, Royal Advisors (UB)",
            commanders: decks::LO_AND_LI_COMMANDERS,
            main: decks::LO_AND_LI_MAIN,
        },
        // Seat 1026: Umori, the Collector's EDHREC average deck. `--pod-decks 1026`.
        PodDeck {
            name: "Umori, the Collector (BG)",
            commanders: decks::UMORI_COMMANDERS,
            main: decks::UMORI_MAIN,
        },
        // Seat 1027: Ingris Stingerquill's EDHREC average deck. `--pod-decks 1027`.
        PodDeck {
            name: "Ingris Stingerquill (BR)",
            commanders: decks::INGRIS_STINGERQUILL_COMMANDERS,
            main: decks::INGRIS_STINGERQUILL_MAIN,
        },
        // Seat 1028: Wulfgar of Icewind Dale's EDHREC average deck. `--pod-decks 1028`.
        PodDeck {
            name: "Wulfgar of Icewind Dale (RG)",
            commanders: decks::WULFGAR_OF_ICEWIND_DALE_COMMANDERS,
            main: decks::WULFGAR_OF_ICEWIND_DALE_MAIN,
        },
        // Seat 1029: Arbaaz Mir's EDHREC average deck. `--pod-decks 1029`.
        PodDeck {
            name: "Arbaaz Mir (WR)",
            commanders: decks::ARBAAZ_MIR_COMMANDERS,
            main: decks::ARBAAZ_MIR_MAIN,
        },
        // Seat 1030: Rip, Spawn Hunter's EDHREC average deck. `--pod-decks 1030`.
        PodDeck {
            name: "Rip, Spawn Hunter (WG)",
            commanders: decks::RIP_COMMANDERS,
            main: decks::RIP_MAIN,
        },
        // Seat 1031: Greven, Predator Captain's EDHREC average deck. `--pod-decks 1031`.
        PodDeck {
            name: "Greven, Predator Captain (BR)",
            commanders: decks::GREVEN_COMMANDERS,
            main: decks::GREVEN_MAIN,
        },
        // Seat 1032: Vraska, the Silencer's EDHREC average deck. `--pod-decks 1032`.
        PodDeck {
            name: "Vraska, the Silencer (BG)",
            commanders: decks::VRASKA_2_COMMANDERS,
            main: decks::VRASKA_2_MAIN,
        },
        // Seat 1033: Karazikar, the Eye Tyrant's EDHREC average deck. `--pod-decks 1033`.
        PodDeck {
            name: "Karazikar, the Eye Tyrant (BR)",
            commanders: decks::KARAZIKAR_COMMANDERS,
            main: decks::KARAZIKAR_MAIN,
        },
        // Seat 1034: Ratonhnhaké꞉ton's EDHREC average deck. `--pod-decks 1034`.
        PodDeck {
            name: "Ratonhnhaké꞉ton (WUB)",
            commanders: decks::RATONHNHAKE_TON_COMMANDERS,
            main: decks::RATONHNHAKE_TON_MAIN,
        },
        // Seat 1035: Arcanis the Omnipotent's EDHREC average deck. `--pod-decks 1035`.
        PodDeck {
            name: "Arcanis the Omnipotent (U)",
            commanders: decks::ARCANIS_THE_OMNIPOTENT_COMMANDERS,
            main: decks::ARCANIS_THE_OMNIPOTENT_MAIN,
        },
        // Seat 1036: Trostani, Three Whispers's EDHREC average deck. `--pod-decks 1036`.
        PodDeck {
            name: "Trostani, Three Whispers (WG)",
            commanders: decks::TROSTANI_2_COMMANDERS,
            main: decks::TROSTANI_2_MAIN,
        },
        // Seat 1037: Runadi, Behemoth Caller's EDHREC average deck. `--pod-decks 1037`.
        PodDeck {
            name: "Runadi, Behemoth Caller (G)",
            commanders: decks::RUNADI_COMMANDERS,
            main: decks::RUNADI_MAIN,
        },
        // Seat 1038: Rakdos, the Showstopper's EDHREC average deck. `--pod-decks 1038`.
        PodDeck {
            name: "Rakdos, the Showstopper (BR)",
            commanders: decks::RAKDOS_COMMANDERS,
            main: decks::RAKDOS_MAIN,
        },
        // Seat 1039: Krang, Utrom Warlord's EDHREC average deck. `--pod-decks 1039`.
        PodDeck {
            name: "Krang, Utrom Warlord (C)",
            commanders: decks::KRANG_COMMANDERS,
            main: decks::KRANG_MAIN,
        },
        // Seat 1040: Yes Man, Personal Securitron's EDHREC average deck. `--pod-decks 1040`.
        PodDeck {
            name: "Yes Man, Personal Securitron (W)",
            commanders: decks::YES_MAN_COMMANDERS,
            main: decks::YES_MAN_MAIN,
        },
        // Seat 1041: Nissa, Worldsoul Speaker's EDHREC average deck. `--pod-decks 1041`.
        PodDeck {
            name: "Nissa, Worldsoul Speaker (G)",
            commanders: decks::NISSA_3_COMMANDERS,
            main: decks::NISSA_3_MAIN,
        },
        // Seat 1042: Mazzy, Truesword Paladin's EDHREC average deck. `--pod-decks 1042`.
        PodDeck {
            name: "Mazzy, Truesword Paladin (WRG)",
            commanders: decks::MAZZY_COMMANDERS,
            main: decks::MAZZY_MAIN,
        },
        // Seat 1043: Sedris, the Traitor King's EDHREC average deck. `--pod-decks 1043`.
        PodDeck {
            name: "Sedris, the Traitor King (UBR)",
            commanders: decks::SEDRIS_COMMANDERS,
            main: decks::SEDRIS_MAIN,
        },
        // Seat 1044: Starscream, Power Hungry's EDHREC average deck. `--pod-decks 1044`.
        PodDeck {
            name: "Starscream, Power Hungry (B)",
            commanders: decks::STARSCREAM_COMMANDERS,
            main: decks::STARSCREAM_MAIN,
        },
        // Seat 1045: Noyan Dar, Roil Shaper's EDHREC average deck. `--pod-decks 1045`.
        PodDeck {
            name: "Noyan Dar, Roil Shaper (WU)",
            commanders: decks::NOYAN_DAR_COMMANDERS,
            main: decks::NOYAN_DAR_MAIN,
        },
        // Seat 1046: Vaan, Street Thief's EDHREC average deck. `--pod-decks 1046`.
        PodDeck {
            name: "Vaan, Street Thief (R)",
            commanders: decks::VAAN_COMMANDERS,
            main: decks::VAAN_MAIN,
        },
        // Seat 1047: Magnus the Red's EDHREC average deck. `--pod-decks 1047`.
        PodDeck {
            name: "Magnus the Red (UR)",
            commanders: decks::MAGNUS_THE_RED_COMMANDERS,
            main: decks::MAGNUS_THE_RED_MAIN,
        },
        // Seat 1048: Yusri, Fortune's Flame's EDHREC average deck. `--pod-decks 1048`.
        PodDeck {
            name: "Yusri, Fortune's Flame (UR)",
            commanders: decks::YUSRI_COMMANDERS,
            main: decks::YUSRI_MAIN,
        },
        // Seat 1049: Nine-Fingers Keene's EDHREC average deck. `--pod-decks 1049`.
        PodDeck {
            name: "Nine-Fingers Keene (UBG)",
            commanders: decks::NINE_FINGERS_KEENE_COMMANDERS,
            main: decks::NINE_FINGERS_KEENE_MAIN,
        },
        // Seat 1050: Xu-Ifit, Osteoharmonist's EDHREC average deck. `--pod-decks 1050`.
        PodDeck {
            name: "Xu-Ifit, Osteoharmonist (B)",
            commanders: decks::XU_IFIT_COMMANDERS,
            main: decks::XU_IFIT_MAIN,
        },
        // Seat 1051: Duskana, the Rage Mother's EDHREC average deck. `--pod-decks 1051`.
        PodDeck {
            name: "Duskana, the Rage Mother (WRG)",
            commanders: decks::DUSKANA_COMMANDERS,
            main: decks::DUSKANA_MAIN,
        },
        // Seat 1052: Hapatra, Vizier of Poisons's EDHREC average deck. `--pod-decks 1052`.
        PodDeck {
            name: "Hapatra, Vizier of Poisons (BG)",
            commanders: decks::HAPATRA_COMMANDERS,
            main: decks::HAPATRA_MAIN,
        },
        // Seat 1053: Grenzo, Dungeon Warden's EDHREC average deck. `--pod-decks 1053`.
        PodDeck {
            name: "Grenzo, Dungeon Warden (BR)",
            commanders: decks::GRENZO_COMMANDERS,
            main: decks::GRENZO_MAIN,
        },
        // Seat 1054: Sergeant John Benton's EDHREC average deck. `--pod-decks 1054`.
        PodDeck {
            name: "Sergeant John Benton (WG)",
            commanders: decks::SERGEANT_JOHN_BENTON_COMMANDERS,
            main: decks::SERGEANT_JOHN_BENTON_MAIN,
        },
        // Seat 1055: Magar of the Magic Strings's EDHREC average deck. `--pod-decks 1055`.
        PodDeck {
            name: "Magar of the Magic Strings (BR)",
            commanders: decks::MAGAR_OF_THE_MAGIC_STRINGS_COMMANDERS,
            main: decks::MAGAR_OF_THE_MAGIC_STRINGS_MAIN,
        },
        // Seat 1056: Narci, Fable Singer's EDHREC average deck. `--pod-decks 1056`.
        PodDeck {
            name: "Narci, Fable Singer (WBG)",
            commanders: decks::NARCI_COMMANDERS,
            main: decks::NARCI_MAIN,
        },
        // Seat 1057: Yennett, Cryptic Sovereign's EDHREC average deck. `--pod-decks 1057`.
        PodDeck {
            name: "Yennett, Cryptic Sovereign (WUB)",
            commanders: decks::YENNETT_COMMANDERS,
            main: decks::YENNETT_MAIN,
        },
        // Seat 1058: Maha, Its Feathers Night's EDHREC average deck. `--pod-decks 1058`.
        PodDeck {
            name: "Maha, Its Feathers Night (B)",
            commanders: decks::MAHA_COMMANDERS,
            main: decks::MAHA_MAIN,
        },
        // Seat 1059: General Ferrous Rokiric's EDHREC average deck. `--pod-decks 1059`.
        PodDeck {
            name: "General Ferrous Rokiric (WR)",
            commanders: decks::GENERAL_FERROUS_ROKIRIC_COMMANDERS,
            main: decks::GENERAL_FERROUS_ROKIRIC_MAIN,
        },
        // Seat 1060: Greasefang, Okiba Boss's EDHREC average deck. `--pod-decks 1060`.
        PodDeck {
            name: "Greasefang, Okiba Boss (WB)",
            commanders: decks::GREASEFANG_COMMANDERS,
            main: decks::GREASEFANG_MAIN,
        },
        // Seat 1061: The Emperor of Palamecia's EDHREC average deck. `--pod-decks 1061`.
        PodDeck {
            name: "The Emperor of Palamecia (UR)",
            commanders: decks::THE_EMPEROR_OF_PALAMECIA_COMMANDERS,
            main: decks::THE_EMPEROR_OF_PALAMECIA_MAIN,
        },
        // Seat 1062: Owen Grady, Raptor Trainer's EDHREC average deck. `--pod-decks 1062`.
        PodDeck {
            name: "Owen Grady, Raptor Trainer (RG)",
            commanders: decks::OWEN_GRADY_COMMANDERS,
            main: decks::OWEN_GRADY_MAIN,
        },
        // Seat 1063: Rydia, Summoner of Mist's EDHREC average deck. `--pod-decks 1063`.
        PodDeck {
            name: "Rydia, Summoner of Mist (RG)",
            commanders: decks::RYDIA_COMMANDERS,
            main: decks::RYDIA_MAIN,
        },
        // Seat 1064: Sheoldred's EDHREC average deck. `--pod-decks 1064`.
        PodDeck {
            name: "Sheoldred (B)",
            commanders: decks::SHEOLDRED_COMMANDERS,
            main: decks::SHEOLDRED_MAIN,
        },
        // Seat 1065: Patron of the Moon's EDHREC average deck. `--pod-decks 1065`.
        PodDeck {
            name: "Patron of the Moon (U)",
            commanders: decks::PATRON_OF_THE_MOON_COMMANDERS,
            main: decks::PATRON_OF_THE_MOON_MAIN,
        },
        // Seat 1066: Pia Nalaar, Chief Mechanic's EDHREC average deck. `--pod-decks 1066`.
        PodDeck {
            name: "Pia Nalaar, Chief Mechanic (URG)",
            commanders: decks::PIA_NALAAR_COMMANDERS,
            main: decks::PIA_NALAAR_MAIN,
        },
        // Seat 1067: Bane, Lord of Darkness's EDHREC average deck. `--pod-decks 1067`.
        PodDeck {
            name: "Bane, Lord of Darkness (WUB)",
            commanders: decks::BANE_COMMANDERS,
            main: decks::BANE_MAIN,
        },
        // Seat 1068: Nevinyrral, Urborg Tyrant's EDHREC average deck. `--pod-decks 1068`.
        PodDeck {
            name: "Nevinyrral, Urborg Tyrant (WUB)",
            commanders: decks::NEVINYRRAL_COMMANDERS,
            main: decks::NEVINYRRAL_MAIN,
        },
        // Seat 1069: Smaug, Wicked Worm's EDHREC average deck. `--pod-decks 1069`.
        PodDeck {
            name: "Smaug, Wicked Worm (BR)",
            commanders: decks::SMAUG_COMMANDERS,
            main: decks::SMAUG_MAIN,
        },
        // Seat 1070: Nikya of the Old Ways's EDHREC average deck. `--pod-decks 1070`.
        PodDeck {
            name: "Nikya of the Old Ways (RG)",
            commanders: decks::NIKYA_OF_THE_OLD_WAYS_COMMANDERS,
            main: decks::NIKYA_OF_THE_OLD_WAYS_MAIN,
        },
        // Seat 1071: Phelddagrif's EDHREC average deck. `--pod-decks 1071`.
        PodDeck {
            name: "Phelddagrif (WUG)",
            commanders: decks::PHELDDAGRIF_COMMANDERS,
            main: decks::PHELDDAGRIF_MAIN,
        },
        // Seat 1072: Ashaya, Soul of the Wild's EDHREC average deck. `--pod-decks 1072`.
        PodDeck {
            name: "Ashaya, Soul of the Wild (G)",
            commanders: decks::ASHAYA_COMMANDERS,
            main: decks::ASHAYA_MAIN,
        },
        // Seat 1073: Zidane, Tantalus Thief's EDHREC average deck. `--pod-decks 1073`.
        PodDeck {
            name: "Zidane, Tantalus Thief (WR)",
            commanders: decks::ZIDANE_COMMANDERS,
            main: decks::ZIDANE_MAIN,
        },
        // Seat 1074: Shadrix Silverquill's EDHREC average deck. `--pod-decks 1074`.
        PodDeck {
            name: "Shadrix Silverquill (WB)",
            commanders: decks::SHADRIX_SILVERQUILL_COMMANDERS,
            main: decks::SHADRIX_SILVERQUILL_MAIN,
        },
        // Seat 1075: Gandalf the White's EDHREC average deck. `--pod-decks 1075`.
        PodDeck {
            name: "Gandalf the White (W)",
            commanders: decks::GANDALF_THE_WHITE_COMMANDERS,
            main: decks::GANDALF_THE_WHITE_MAIN,
        },
        // Seat 1076: Oskar, Rubbish Reclaimer's EDHREC average deck. `--pod-decks 1076`.
        PodDeck {
            name: "Oskar, Rubbish Reclaimer (UB)",
            commanders: decks::OSKAR_COMMANDERS,
            main: decks::OSKAR_MAIN,
        },
        // Seat 1077: Xanathar, Guild Kingpin's EDHREC average deck. `--pod-decks 1077`.
        PodDeck {
            name: "Xanathar, Guild Kingpin (UB)",
            commanders: decks::XANATHAR_COMMANDERS,
            main: decks::XANATHAR_MAIN,
        },
        // Seat 1078: Y'shtola Rhul's EDHREC average deck. `--pod-decks 1078`.
        PodDeck {
            name: "Y'shtola Rhul (U)",
            commanders: decks::YSHTOLA_RHUL_COMMANDERS,
            main: decks::YSHTOLA_RHUL_MAIN,
        },
        // Seat 1079: Mass of Mysteries's EDHREC average deck. `--pod-decks 1079`.
        PodDeck {
            name: "Mass of Mysteries (WUBRG)",
            commanders: decks::MASS_OF_MYSTERIES_COMMANDERS,
            main: decks::MASS_OF_MYSTERIES_MAIN,
        },
        // Seat 1080: Ozox, the Clattering King's EDHREC average deck. `--pod-decks 1080`.
        PodDeck {
            name: "Ozox, the Clattering King (B)",
            commanders: decks::OZOX_COMMANDERS,
            main: decks::OZOX_MAIN,
        },
        // Seat 1081: The Sixth Doctor and Romana II's EDHREC average deck. `--pod-decks 1081`.
        PodDeck {
            name: "The Sixth Doctor and Romana II (WUG)",
            commanders: decks::THE_SIXTH_DOCTOR_COMMANDERS,
            main: decks::THE_SIXTH_DOCTOR_MAIN,
        },
        // Seat 1082: Electro, Assaulting Battery's EDHREC average deck. `--pod-decks 1082`.
        PodDeck {
            name: "Electro, Assaulting Battery (R)",
            commanders: decks::ELECTRO_COMMANDERS,
            main: decks::ELECTRO_MAIN,
        },
        // Seat 1083: Ozai, the Phoenix King's EDHREC average deck. `--pod-decks 1083`.
        PodDeck {
            name: "Ozai, the Phoenix King (BR)",
            commanders: decks::OZAI_COMMANDERS,
            main: decks::OZAI_MAIN,
        },
        // Seat 1084: Ruby, Daring Tracker's EDHREC average deck. `--pod-decks 1084`.
        PodDeck {
            name: "Ruby, Daring Tracker (RG)",
            commanders: decks::RUBY_COMMANDERS,
            main: decks::RUBY_MAIN,
        },
        // Seat 1085: Kami of the Crescent Moon's EDHREC average deck. `--pod-decks 1085`.
        PodDeck {
            name: "Kami of the Crescent Moon (U)",
            commanders: decks::KAMI_OF_THE_CRESCENT_MOON_COMMANDERS,
            main: decks::KAMI_OF_THE_CRESCENT_MOON_MAIN,
        },
        // Seat 1086: Ashling, Flame Dancer's EDHREC average deck. `--pod-decks 1086`.
        PodDeck {
            name: "Ashling, Flame Dancer (R)",
            commanders: decks::ASHLING_3_COMMANDERS,
            main: decks::ASHLING_3_MAIN,
        },
        // Seat 1087: Captain Sisay's EDHREC average deck. `--pod-decks 1087`.
        PodDeck {
            name: "Captain Sisay (WG)",
            commanders: decks::CAPTAIN_SISAY_COMMANDERS,
            main: decks::CAPTAIN_SISAY_MAIN,
        },
        // Seat 1088: Kethis, the Hidden Hand's EDHREC average deck. `--pod-decks 1088`.
        PodDeck {
            name: "Kethis, the Hidden Hand (WBG)",
            commanders: decks::KETHIS_COMMANDERS,
            main: decks::KETHIS_MAIN,
        },
        // Seat 1089: Chevill, Bane of Monsters's EDHREC average deck. `--pod-decks 1089`.
        PodDeck {
            name: "Chevill, Bane of Monsters (BG)",
            commanders: decks::CHEVILL_COMMANDERS,
            main: decks::CHEVILL_MAIN,
        },
        // Seat 1090: Nashi, Searcher in the Dark's EDHREC average deck. `--pod-decks 1090`.
        PodDeck {
            name: "Nashi, Searcher in the Dark (UB)",
            commanders: decks::NASHI_COMMANDERS,
            main: decks::NASHI_MAIN,
        },
        // Seat 1091: Mirko Vosk, Mind Drinker's EDHREC average deck. `--pod-decks 1091`.
        PodDeck {
            name: "Mirko Vosk, Mind Drinker (UB)",
            commanders: decks::MIRKO_VOSK_COMMANDERS,
            main: decks::MIRKO_VOSK_MAIN,
        },
        // Seat 1092: Purphoros, Bronze-Blooded's EDHREC average deck. `--pod-decks 1092`.
        PodDeck {
            name: "Purphoros, Bronze-Blooded (R)",
            commanders: decks::PURPHOROS_2_COMMANDERS,
            main: decks::PURPHOROS_2_MAIN,
        },
        // Seat 1093: Wilson, Refined Grizzly and Flaming Fist's EDHREC average deck. `--pod-decks 1093`.
        PodDeck {
            name: "Wilson, Refined Grizzly and Flaming Fist (WG)",
            commanders: decks::WILSON_COMMANDERS,
            main: decks::WILSON_MAIN,
        },
        // Seat 1094: Titania, Nature's Force's EDHREC average deck. `--pod-decks 1094`.
        PodDeck {
            name: "Titania, Nature's Force (G)",
            commanders: decks::TITANIA_2_COMMANDERS,
            main: decks::TITANIA_2_MAIN,
        },
        // Seat 1095: Titania, Voice of Gaea's EDHREC average deck. `--pod-decks 1095`.
        PodDeck {
            name: "Titania, Voice of Gaea (G)",
            commanders: decks::TITANIA_3_COMMANDERS,
            main: decks::TITANIA_3_MAIN,
        },
        // Seat 1096: Queen Kayla bin-Kroog's EDHREC average deck. `--pod-decks 1096`.
        PodDeck {
            name: "Queen Kayla bin-Kroog (WR)",
            commanders: decks::QUEEN_KAYLA_BIN_KROOG_COMMANDERS,
            main: decks::QUEEN_KAYLA_BIN_KROOG_MAIN,
        },
        // Seat 1097: Jackal, Genius Geneticist's EDHREC average deck. `--pod-decks 1097`.
        PodDeck {
            name: "Jackal, Genius Geneticist (UG)",
            commanders: decks::JACKAL_COMMANDERS,
            main: decks::JACKAL_MAIN,
        },
        // Seat 1098: Lluwen, Imperfect Naturalist's EDHREC average deck. `--pod-decks 1098`.
        PodDeck {
            name: "Lluwen, Imperfect Naturalist (BG)",
            commanders: decks::LLUWEN_COMMANDERS,
            main: decks::LLUWEN_MAIN,
        },
        // Seat 1099: Marvin, Murderous Mimic's EDHREC average deck. `--pod-decks 1099`.
        PodDeck {
            name: "Marvin, Murderous Mimic (C)",
            commanders: decks::MARVIN_COMMANDERS,
            main: decks::MARVIN_MAIN,
        },
        // Seat 1100: Tawnos, Solemn Survivor's EDHREC average deck. `--pod-decks 1100`.
        PodDeck {
            name: "Tawnos, Solemn Survivor (WUB)",
            commanders: decks::TAWNOS_2_COMMANDERS,
            main: decks::TAWNOS_2_MAIN,
        },
        // Seat 1101: Inniaz, the Gale Force's EDHREC average deck. `--pod-decks 1101`.
        PodDeck {
            name: "Inniaz, the Gale Force (WU)",
            commanders: decks::INNIAZ_COMMANDERS,
            main: decks::INNIAZ_MAIN,
        },
        // Seat 1102: Vaevictis Asmadi, the Dire's EDHREC average deck. `--pod-decks 1102`.
        PodDeck {
            name: "Vaevictis Asmadi, the Dire (BRG)",
            commanders: decks::VAEVICTIS_ASMADI_COMMANDERS,
            main: decks::VAEVICTIS_ASMADI_MAIN,
        },
        // Seat 1103: Cormela, Glamour Thief's EDHREC average deck. `--pod-decks 1103`.
        PodDeck {
            name: "Cormela, Glamour Thief (UBR)",
            commanders: decks::CORMELA_COMMANDERS,
            main: decks::CORMELA_MAIN,
        },
        // Seat 1104: Pia Nalaar, Consul of Revival's EDHREC average deck. `--pod-decks 1104`.
        PodDeck {
            name: "Pia Nalaar, Consul of Revival (WR)",
            commanders: decks::PIA_NALAAR_2_COMMANDERS,
            main: decks::PIA_NALAAR_2_MAIN,
        },
        // Seat 1105: Smaug the Magnificent's EDHREC average deck. `--pod-decks 1105`.
        PodDeck {
            name: "Smaug the Magnificent (R)",
            commanders: decks::SMAUG_THE_MAGNIFICENT_COMMANDERS,
            main: decks::SMAUG_THE_MAGNIFICENT_MAIN,
        },
        // Seat 1106: Omnath, Locus of the Roil's EDHREC average deck. `--pod-decks 1106`.
        PodDeck {
            name: "Omnath, Locus of the Roil (URG)",
            commanders: decks::OMNATH_2_COMMANDERS,
            main: decks::OMNATH_2_MAIN,
        },
        // Seat 1107: Radha, Heir to Keld's EDHREC average deck. `--pod-decks 1107`.
        PodDeck {
            name: "Radha, Heir to Keld (RG)",
            commanders: decks::RADHA_COMMANDERS,
            main: decks::RADHA_MAIN,
        },
        // Seat 1108: Kylox, Visionary Inventor's EDHREC average deck. `--pod-decks 1108`.
        PodDeck {
            name: "Kylox, Visionary Inventor (UR)",
            commanders: decks::KYLOX_COMMANDERS,
            main: decks::KYLOX_MAIN,
        },
        // Seat 1109: Geralf, the Fleshwright's EDHREC average deck. `--pod-decks 1109`.
        PodDeck {
            name: "Geralf, the Fleshwright (U)",
            commanders: decks::GERALF_COMMANDERS,
            main: decks::GERALF_MAIN,
        },
        // Seat 1110: The Sentry, Golden Guardian's EDHREC average deck. `--pod-decks 1110`.
        PodDeck {
            name: "The Sentry, Golden Guardian (W)",
            commanders: decks::THE_SENTRY_COMMANDERS,
            main: decks::THE_SENTRY_MAIN,
        },
        // Seat 1111: Taskmaster, Mercenary Mimic's EDHREC average deck. `--pod-decks 1111`.
        PodDeck {
            name: "Taskmaster, Mercenary Mimic (UB)",
            commanders: decks::TASKMASTER_COMMANDERS,
            main: decks::TASKMASTER_MAIN,
        },
        // Seat 1112: Huatli, Poet of Unity's EDHREC average deck. `--pod-decks 1112`.
        PodDeck {
            name: "Huatli, Poet of Unity (WRG)",
            commanders: decks::HUATLI_COMMANDERS,
            main: decks::HUATLI_MAIN,
        },
        // Seat 1113: Nashi, Moon's Legacy's EDHREC average deck. `--pod-decks 1113`.
        PodDeck {
            name: "Nashi, Moon's Legacy (UBG)",
            commanders: decks::NASHI_2_COMMANDERS,
            main: decks::NASHI_2_MAIN,
        },
        // Seat 1114: Tekuthal, Inquiry Dominus's EDHREC average deck. `--pod-decks 1114`.
        PodDeck {
            name: "Tekuthal, Inquiry Dominus (U)",
            commanders: decks::TEKUTHAL_COMMANDERS,
            main: decks::TEKUTHAL_MAIN,
        },
        // Seat 1115: The Serpent Society's EDHREC average deck. `--pod-decks 1115`.
        PodDeck {
            name: "The Serpent Society (BG)",
            commanders: decks::THE_SERPENT_SOCIETY_COMMANDERS,
            main: decks::THE_SERPENT_SOCIETY_MAIN,
        },
        // Seat 1116: The Master, Multiplied's EDHREC average deck. `--pod-decks 1116`.
        PodDeck {
            name: "The Master, Multiplied (BR)",
            commanders: decks::THE_MASTER_2_COMMANDERS,
            main: decks::THE_MASTER_2_MAIN,
        },
        // Seat 1117: Mayael the Anima's EDHREC average deck. `--pod-decks 1117`.
        PodDeck {
            name: "Mayael the Anima (WRG)",
            commanders: decks::MAYAEL_THE_ANIMA_COMMANDERS,
            main: decks::MAYAEL_THE_ANIMA_MAIN,
        },
        // Seat 1118: Gix, Yawgmoth Praetor's EDHREC average deck. `--pod-decks 1118`.
        PodDeck {
            name: "Gix, Yawgmoth Praetor (B)",
            commanders: decks::GIX_COMMANDERS,
            main: decks::GIX_MAIN,
        },
        // Seat 1119: Arahbo, the First Fang's EDHREC average deck. `--pod-decks 1119`.
        PodDeck {
            name: "Arahbo, the First Fang (W)",
            commanders: decks::ARAHBO_2_COMMANDERS,
            main: decks::ARAHBO_2_MAIN,
        },
        // Seat 1120: Edric, Spymaster of Trest's EDHREC average deck. `--pod-decks 1120`.
        PodDeck {
            name: "Edric, Spymaster of Trest (UG)",
            commanders: decks::EDRIC_COMMANDERS,
            main: decks::EDRIC_MAIN,
        },
        // Seat 1121: Six's EDHREC average deck. `--pod-decks 1121`.
        PodDeck {
            name: "Six (G)",
            commanders: decks::SIX_COMMANDERS,
            main: decks::SIX_MAIN,
        },
        // Seat 1122: Thrun, Breaker of Silence's EDHREC average deck. `--pod-decks 1122`.
        PodDeck {
            name: "Thrun, Breaker of Silence (G)",
            commanders: decks::THRUN_COMMANDERS,
            main: decks::THRUN_MAIN,
        },
        // Seat 1123: Obyra, Dreaming Duelist's EDHREC average deck. `--pod-decks 1123`.
        PodDeck {
            name: "Obyra, Dreaming Duelist (UB)",
            commanders: decks::OBYRA_COMMANDERS,
            main: decks::OBYRA_MAIN,
        },
        // Seat 1124: Ajani, Nacatl Pariah's EDHREC average deck. `--pod-decks 1124`.
        PodDeck {
            name: "Ajani, Nacatl Pariah (WR)",
            commanders: decks::AJANI_COMMANDERS,
            main: decks::AJANI_MAIN,
        },
        // Seat 1125: Shattergang Brothers's EDHREC average deck. `--pod-decks 1125`.
        PodDeck {
            name: "Shattergang Brothers (BRG)",
            commanders: decks::SHATTERGANG_BROTHERS_COMMANDERS,
            main: decks::SHATTERGANG_BROTHERS_MAIN,
        },
        // Seat 1126: Tinybones, the Pickpocket's EDHREC average deck. `--pod-decks 1126`.
        PodDeck {
            name: "Tinybones, the Pickpocket (B)",
            commanders: decks::TINYBONES_2_COMMANDERS,
            main: decks::TINYBONES_2_MAIN,
        },
        // Seat 1127: Toshiro Umezawa's EDHREC average deck. `--pod-decks 1127`.
        PodDeck {
            name: "Toshiro Umezawa (B)",
            commanders: decks::TOSHIRO_UMEZAWA_COMMANDERS,
            main: decks::TOSHIRO_UMEZAWA_MAIN,
        },
        // Seat 1128: Aveline de Grandpré's EDHREC average deck. `--pod-decks 1128`.
        PodDeck {
            name: "Aveline de Grandpré (BG)",
            commanders: decks::AVELINE_DE_GRANDPRE_COMMANDERS,
            main: decks::AVELINE_DE_GRANDPRE_MAIN,
        },
        // Seat 1129: Ganax, Astral Hunter and Feywild Visitor's EDHREC average deck. `--pod-decks 1129`.
        PodDeck {
            name: "Ganax, Astral Hunter and Feywild Visitor (UR)",
            commanders: decks::GANAX_COMMANDERS,
            main: decks::GANAX_MAIN,
        },
        // Seat 1130: Brion Stoutarm's EDHREC average deck. `--pod-decks 1130`.
        PodDeck {
            name: "Brion Stoutarm (WR)",
            commanders: decks::BRION_STOUTARM_COMMANDERS,
            main: decks::BRION_STOUTARM_MAIN,
        },
        // Seat 1131: Aerith Gainsborough's EDHREC average deck. `--pod-decks 1131`.
        PodDeck {
            name: "Aerith Gainsborough (W)",
            commanders: decks::AERITH_GAINSBOROUGH_COMMANDERS,
            main: decks::AERITH_GAINSBOROUGH_MAIN,
        },
        // Seat 1132: Olivia, Crimson Bride's EDHREC average deck. `--pod-decks 1132`.
        PodDeck {
            name: "Olivia, Crimson Bride (BR)",
            commanders: decks::OLIVIA_2_COMMANDERS,
            main: decks::OLIVIA_2_MAIN,
        },
        // Seat 1133: Katilda and Lier's EDHREC average deck. `--pod-decks 1133`.
        PodDeck {
            name: "Katilda and Lier (WUG)",
            commanders: decks::KATILDA_AND_LIER_COMMANDERS,
            main: decks::KATILDA_AND_LIER_MAIN,
        },
        // Seat 1134: Endrek Sahr, Master Breeder's EDHREC average deck. `--pod-decks 1134`.
        PodDeck {
            name: "Endrek Sahr, Master Breeder (B)",
            commanders: decks::ENDREK_SAHR_COMMANDERS,
            main: decks::ENDREK_SAHR_MAIN,
        },
        // Seat 1135: Avatar Roku, Firebender's EDHREC average deck. `--pod-decks 1135`.
        PodDeck {
            name: "Avatar Roku, Firebender (R)",
            commanders: decks::AVATAR_ROKU_COMMANDERS,
            main: decks::AVATAR_ROKU_MAIN,
        },
        // Seat 1136: Kraven the Hunter's EDHREC average deck. `--pod-decks 1136`.
        PodDeck {
            name: "Kraven the Hunter (BG)",
            commanders: decks::KRAVEN_THE_HUNTER_COMMANDERS,
            main: decks::KRAVEN_THE_HUNTER_MAIN,
        },
        // Seat 1137: Amarant Coral's EDHREC average deck. `--pod-decks 1137`.
        PodDeck {
            name: "Amarant Coral (RG)",
            commanders: decks::AMARANT_CORAL_COMMANDERS,
            main: decks::AMARANT_CORAL_MAIN,
        },
        // Seat 1138: Inferno of the Star Mounts's EDHREC average deck. `--pod-decks 1138`.
        PodDeck {
            name: "Inferno of the Star Mounts (R)",
            commanders: decks::INFERNO_OF_THE_STAR_MOUNTS_COMMANDERS,
            main: decks::INFERNO_OF_THE_STAR_MOUNTS_MAIN,
        },
        // Seat 1139: Commander Mustard's EDHREC average deck. `--pod-decks 1139`.
        PodDeck {
            name: "Commander Mustard (WR)",
            commanders: decks::COMMANDER_MUSTARD_COMMANDERS,
            main: decks::COMMANDER_MUSTARD_MAIN,
        },
        // Seat 1140: Ashling the Pilgrim's EDHREC average deck. `--pod-decks 1140`.
        PodDeck {
            name: "Ashling the Pilgrim (R)",
            commanders: decks::ASHLING_THE_PILGRIM_COMMANDERS,
            main: decks::ASHLING_THE_PILGRIM_MAIN,
        },
        // Seat 1141: Tetsuko Umezawa, Fugitive's EDHREC average deck. `--pod-decks 1141`.
        PodDeck {
            name: "Tetsuko Umezawa, Fugitive (U)",
            commanders: decks::TETSUKO_UMEZAWA_COMMANDERS,
            main: decks::TETSUKO_UMEZAWA_MAIN,
        },
        // Seat 1142: Blor the Impervious's EDHREC average deck. `--pod-decks 1142`.
        PodDeck {
            name: "Blor the Impervious (G)",
            commanders: decks::BLOR_THE_IMPERVIOUS_COMMANDERS,
            main: decks::BLOR_THE_IMPERVIOUS_MAIN,
        },
        // Seat 1143: Phlage, Titan of Fire's Fury's EDHREC average deck. `--pod-decks 1143`.
        PodDeck {
            name: "Phlage, Titan of Fire's Fury (WR)",
            commanders: decks::PHLAGE_COMMANDERS,
            main: decks::PHLAGE_MAIN,
        },
        // Seat 1144: Parnesse, the Subtle Brush's EDHREC average deck. `--pod-decks 1144`.
        PodDeck {
            name: "Parnesse, the Subtle Brush (UBR)",
            commanders: decks::PARNESSE_COMMANDERS,
            main: decks::PARNESSE_MAIN,
        },
        // Seat 1145: Kassandra, Eagle Bearer's EDHREC average deck. `--pod-decks 1145`.
        PodDeck {
            name: "Kassandra, Eagle Bearer (WR)",
            commanders: decks::KASSANDRA_COMMANDERS,
            main: decks::KASSANDRA_MAIN,
        },
        // Seat 1146: Minsc, Beloved Ranger's EDHREC average deck. `--pod-decks 1146`.
        PodDeck {
            name: "Minsc, Beloved Ranger (WRG)",
            commanders: decks::MINSC_COMMANDERS,
            main: decks::MINSC_MAIN,
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
/// CR 732.1b / 732.2a — an optional loop repeats as many times as its player
/// chooses. A seat past this many activations of one ability of one permanent
/// in a turn is in a loop the bot cannot price (Woe Strider sacrificing
/// Prowling Geistcatcher's returns, which pay Gravespawn Sovereign to bring
/// the Geistcatcher back: 2,723 scries in one main phase); its next one is a
/// pass instead. Casts of one card count the same way: a cast its own answer
/// made illegal is rewound (CR 733.1), and a seat that answers the same way
/// again re-casts it until the cap (Launch the Fleet, 643 casts in one turn).
/// Pods only — the two-player drivers never reach it.
const POD_SAME_ACTIVATION_PER_TURN: u32 = 64;

/// This turn's activation counts by (seat, source, ability index; a cast is
/// index `usize::MAX`), and casts again by (seat, card name); see
/// [`POD_SAME_ACTIVATION_PER_TURN`].
#[derive(Default)]
struct RepeatGuard {
    turn: u32,
    seen: Vec<(usize, crate::card::CardId, usize, u32)>,
    /// Casts by name: a free creature that dies and comes back to hand spreads
    /// its loop over every copy (Shadowborn Apostle under Edgewalker and
    /// Athreos, God of Passage: 760 casts over sixteen copies in one turn,
    /// none past the per-card cap).
    names: Vec<(usize, &'static str, u32)>,
}

impl RepeatGuard {
    /// Count `action` for `seat`; `false` once it is past the per-turn cap.
    #[cfg(test)]
    fn admit(&mut self, turn: u32, seat: usize, action: &crate::game::GameAction) -> bool {
        self.admit_named(turn, seat, action, None)
    }

    /// [`Self::admit`], with `name` the cast card's when the action is a
    /// cast.
    fn admit_named(
        &mut self,
        turn: u32,
        seat: usize,
        action: &crate::game::GameAction,
        name: Option<&'static str>,
    ) -> bool {
        use crate::game::GameAction as A;
        // The activations with their own action carry a pseudo-index under
        // the cast's `usize::MAX`: a free Equip (Puresteel Paladin) moved one
        // Equipment between two creatures 401 times in a turn (sweep 306001).
        let (card_id, ability_index) = match *action {
            A::ActivateAbility { card_id, ability_index, .. }
            | A::ActivateAbilityWaterbend { card_id, ability_index, .. } => (card_id, ability_index),
            A::Equip { equipment, .. } => (equipment, usize::MAX - 1),
            A::Reconfigure { equipment, .. } => (equipment, usize::MAX - 2),
            A::Crew { vehicle, .. } => (vehicle, usize::MAX - 3),
            A::Saddle { mount, .. } => (mount, usize::MAX - 4),
            _ => match action.cast_card_id() {
                Some(id) => (id, usize::MAX),
                None => return true,
            },
        };
        if turn != self.turn {
            self.turn = turn;
            self.seen.clear();
            self.names.clear();
        }
        if let Some(name) = name.filter(|_| ability_index == usize::MAX) {
            let n = match self.names.iter_mut().find(|r| r.0 == seat && r.1 == name) {
                Some(r) => {
                    r.2 += 1;
                    r.2
                }
                None => {
                    self.names.push((seat, name, 1));
                    1
                }
            };
            if n > POD_SAME_ACTIVATION_PER_TURN {
                return false;
            }
        }
        let key = (seat, card_id, ability_index);
        let n = match self.seen.iter_mut().find(|r| (r.0, r.1, r.2) == key) {
            Some(r) => {
                r.3 += 1;
                r.3
            }
            None => {
                self.seen.push((seat, card_id, ability_index, 1));
                1
            }
        };
        n <= POD_SAME_ACTIVATION_PER_TURN
    }
}

/// `CRAB_POD_TRACE=<n>`: from action `n` on, every accepted action of a pod
/// game on stderr — turn, step, stack depth, seat, action — and every
/// rejected one as a `REJECT` line with the engine's error. Names a capped
/// game's loop without a rebuild (`--first I --games 1` replays it).
fn pod_trace_from() -> Option<usize> {
    static FROM: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    *FROM.get_or_init(|| std::env::var("CRAB_POD_TRACE").ok().and_then(|s| s.parse().ok()))
}

/// The action's `Debug`, then the name of every `CardId` it mentions; an
/// answer also names the ask it answers.
fn trace_line(g: &GameState, action: &crate::game::GameAction) -> String {
    let mut dbg = format!("{action:?}");
    if matches!(action, crate::game::GameAction::SubmitDecision(_))
        && let Some(d) = g.pending_decision.as_ref()
    {
        let ask = format!("{:?}", d.decision);
        dbg = format!("{dbg} <- {}", &ask[..ask.len().min(160)]);
    }
    let names: Vec<String> = dbg
        .split("CardId(")
        .skip(1)
        .filter_map(|rest| rest.split(')').next()?.parse::<u32>().ok())
        .map(|n| {
            let id = crate::card::CardId(n);
            g.find_card_anywhere(id).map_or_else(|| format!("{n}=?"), |c| format!("{n}={}", c.definition.name))
        })
        .collect();
    let dbg = if names.is_empty() { dbg } else { format!("{dbg} [{}]", names.join(", ")) };
    // Name the stack's top object, so a trigger loop names its trigger.
    match g.stack.last() {
        Some(crate::game::StackItem::Trigger { source, .. }) => {
            let name = g.find_card_anywhere(*source).map_or("?", |c| c.definition.name);
            format!("{dbg} {{top: trigger of {name}}}")
        }
        Some(crate::game::StackItem::Spell { card, .. }) => format!("{dbg} {{top: {}}}", card.definition.name),
        None => dbg,
    }
}

/// The nontoken cards each seat owns, across every zone a card can be in (a
/// melded or merged permanent counts its component cards). A spell copy is a
/// token here (CR 707.10a). For the debug conservation invariant.
#[cfg(debug_assertions)]
fn owned_card_counts(g: &GameState) -> Vec<Vec<crate::card::CardId>> {
    let mut n = vec![Vec::new(); g.players.len()];
    let mut add = |c: &crate::card::CardInstance| {
        let parts: Vec<&crate::card::CardInstance> = if !c.meld_parts.is_empty() {
            c.meld_parts.iter().collect()
        } else if !c.mutate_stack.is_empty() {
            c.mutate_stack.iter().collect()
        } else {
            vec![c]
        };
        for part in parts {
            if !part.is_token
                && let Some(slot) = n.get_mut(part.owner)
            {
                slot.push(part.id);
            }
        }
    };
    g.battlefield.iter().for_each(&mut add);
    g.phased_out.iter().for_each(&mut add);
    g.exile.iter().for_each(&mut add);
    for si in g.stack.iter() {
        if let crate::game::types::StackItem::Spell { card, .. } = si {
            add(card);
        }
    }
    for p in g.players.iter() {
        for z in [&*p.graveyard, &*p.hand, &*p.library, &*p.command, &*p.ante, &*p.sideboard] {
            z.iter().for_each(&mut add);
        }
    }
    for ids in &mut n {
        ids.sort_unstable();
    }
    n
}

/// Which zone `id` is in, for an invariant's message. Debug-only.
#[cfg(debug_assertions)]
fn zone_label(g: &GameState, id: crate::card::CardId) -> &'static str {
    let has = |z: &[crate::card::CardInstance]| z.iter().any(|c| c.id == id);
    if g.battlefield_find(id).is_some() {
        "the battlefield"
    } else if g.battlefield.iter().any(|c| c.meld_parts.iter().chain(c.mutate_stack.iter()).any(|p| p.id == id)) {
        "a melded or merged permanent"
    } else if has(&g.phased_out)
        // A phased-out merged or melded permanent keeps its parts (Talon Gates
        // of Madara phasing out a mutated commander, seed 221030).
        || g.phased_out.iter().any(|c| c.meld_parts.iter().chain(c.mutate_stack.iter()).any(|p| p.id == id))
    {
        "phased out"
    } else if has(&g.exile) {
        "exile"
    } else if g.stack.iter().any(|si| matches!(si, crate::game::types::StackItem::Spell { card, .. } if card.id == id)) {
        "the stack"
    } else if g.players.iter().any(|p| has(&p.graveyard)) {
        "a graveyard"
    } else if g.players.iter().any(|p| has(&p.hand)) {
        "a hand"
    } else if g.players.iter().any(|p| has(&p.library)) {
        "a library"
    } else if g.players.iter().any(|p| has(&p.command)) {
        "a command zone"
    } else if let Some(zone) = {
        // Still a failure (CR 712.4: off the battlefield a melded or merged
        // card is its parts), but one that names where the shell went.
        let inside = |z: &[crate::card::CardInstance]| {
            z.iter().any(|c| c.meld_parts.iter().chain(c.mutate_stack.iter()).any(|p| p.id == id))
        };
        if inside(&g.exile) {
            Some("nowhere (inside a melded or merged card in exile)")
        } else if g.players.iter().any(|p| inside(&p.graveyard)) {
            Some("nowhere (inside a melded or merged card in a graveyard)")
        } else if g.players.iter().any(|p| inside(&p.hand) || inside(&p.library)) {
            Some("nowhere (inside a melded or merged card in a hand or library)")
        } else if g.players.iter().any(|p| inside(&p.command)) {
            Some("nowhere (inside a melded or merged card in a command zone)")
        } else {
            None
        }
    } {
        zone
    } else {
        "nowhere"
    }
}

/// The per-action commander invariants (debug pods): CR 903.3 a living
/// seat's commander is a card still in the game (a melded or merged one is in
/// its permanent); CR 903.8 the tax count never falls, and rises when a
/// commander goes from the command zone to the stack; CR 903.10a only a
/// commander deals commander damage. `seen` is each commander's zone and
/// count after the previous action.
#[cfg(debug_assertions)]
fn check_commanders(
    g: &GameState,
    seed: u64,
    actions: usize,
    seen: &mut Vec<(crate::card::CardId, &'static str, u32)>,
) {
    if g.is_game_over() {
        return;
    }
    for (i, p) in g.players.iter().enumerate().filter(|(_, p)| p.is_alive()) {
        for &cmd in &p.commanders {
            let zone = zone_label(g, cmd);
            let now = g.commander_cast_count.get(&cmd).copied().unwrap_or(0);
            let (was, before) = seen.iter().find(|s| s.0 == cmd).map_or(("unknown", 0), |s| (s.1, s.2));
            assert!(
                !zone.starts_with("nowhere"),
                "seed {seed}: p{i}'s commander {cmd:?} left {was} for {zone} (turn {}, {:?}, after {actions} actions; pending {:?})",
                g.turn_number,
                g.step,
                g.pending_decision.as_ref().map(|p| &p.decision),
            );
            assert!(
                now >= before && (was != "a command zone" || zone != "the stack" || now > before),
                "seed {seed}: commander {cmd:?}'s cast count went {before} -> {now} moving from {was} to {zone} (turn {}, after {actions} actions)",
                g.turn_number,
            );
        }
    }
    if let Some(&(victim, id)) =
        g.commander_damage.keys().find(|(_, id)| !g.players.iter().any(|p| p.commanders.contains(id)))
    {
        panic!("seed {seed}: p{victim} took commander damage from {id:?}, which is no commander (turn {})", g.turn_number);
    }
    seen.clear();
    for p in g.players.iter() {
        seen.extend(
            p.commanders.iter().map(|&c| (c, zone_label(g, c), g.commander_cast_count.get(&c).copied().unwrap_or(0))),
        );
    }
}

/// `CRAB_POD_SWEEP_PROBE=1` (debug builds): after every settled action, a
/// clone's fresh CR 704.3 sweep must do nothing — the strict form of the
/// per-rule invariants below, one clone an action, so opt-in.
#[cfg(debug_assertions)]
fn pod_sweep_probe() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("CRAB_POD_SWEEP_PROBE").is_some())
}

/// `CRAB_POD_CONCEDE=<n>`: after each accepted action, a living seat concedes
/// with probability `n / 10_000` (CR 104.3a) — CR 800.4a's leave path driven
/// at arbitrary points (mid-combat, mid-payment, under a pending ask), where a
/// pod's own losses only ever reach it from a state-based check. Its own
/// seeded stream, so an unset run is the run it always was.
fn pod_concede_rate() -> Option<u32> {
    static RATE: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
    *RATE.get_or_init(|| std::env::var("CRAB_POD_CONCEDE").ok().and_then(|s| s.parse().ok()))
}

pub fn play_one_pod_game_censused(
    template: &GameState,
    pilots: &[Pilot],
    max_actions: usize,
    seed: u64,
    into: Option<&mut ActionCensus>,
) -> PodOutcome {
    play_pod_game(template, pilots, max_actions, seed, into, pod_concede_rate())
}

/// The pod loop; `concede_rate` is [`pod_concede_rate`]'s knob as a value.
fn play_pod_game(
    template: &GameState,
    pilots: &[Pilot],
    max_actions: usize,
    seed: u64,
    into: Option<&mut ActionCensus>,
    concede_rate: Option<u32>,
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
    if let Some(rate) = fuzz::fuzz_rate() {
        bots = bots
            .into_iter()
            .enumerate()
            .map(|(i, b)| fuzz::FuzzBot::wrap(b, rate, seed ^ 0xF0CC_0000 ^ i as u64))
            .collect();
    }
    let (mut actions, mut plays, mut stale) = (0usize, 0usize, 0usize);
    let mut pass_streak = 0usize;
    let (diag_floor, mut diag_said) = (crate::recommend::cap_diag_floor().flatten(), false);
    let trace_from = pod_trace_from();
    let mut repeats = RepeatGuard::default();
    #[cfg(debug_assertions)]
    let mut last_turn: Option<(u32, usize, Option<usize>)> = None;
    #[cfg(debug_assertions)]
    let mut commanders_seen: Vec<(crate::card::CardId, &'static str, u32)> = Vec::new();
    let mut concede = concede_rate.map(|n| (n, StdRng::seed_from_u64(seed ^ 0xC0DE_C0DE)));
    // One `OnceLock` read a game, not a bool per action: off, `record` is a
    // field test and the `Debug` format below never runs.
    // CR 400.1 — a card is always somewhere: each seat's nontoken card
    // count holds while it is in the game (debug; checked on settled boards).
    #[cfg(debug_assertions)]
    let owned_at_start = owned_card_counts(&g);
    let mut census =
        if into.is_some() { ActionCensus::forced() } else { ActionCensus::armed() };
    // The budget counts plays, not priority passes (`PodOutcome::plays`), with
    // a total-action backstop far above anything a play budget allows.
    let spent = |actions: usize, plays: usize, pass_streak: usize| {
        if actions >= max_actions.saturating_mul(PASSES_PER_PLAY_BACKSTOP) || pass_streak >= PASS_STREAK_CAP {
            max_actions
        } else {
            plays
        }
    };
    while stop_reason(&g, spent(actions, plays, pass_streak), max_actions, stale).is_none() {
        let mut any = false;
        for (seat, bot) in bots.iter_mut().enumerate() {
            // Eliminated seats are polled like any other: the bot answers
            // `None` for a seat without priority, and CR 800.4a has already
            // taken their objects and any decision addressed to them. Skipping
            // them here instead was a deadlock — a pending decision the loop
            // never polled suppressed every other seat's actions.
            let Some(step) = bot.next_action_settled(&g, seat) else { continue };
            let crate::server::bot::BotStep { mut action, mut settled } = step;
            let cast_name = action.cast_card_id().and_then(|id| g.find_card_anywhere(id)).map(|c| c.definition.name);
            if !repeats.admit_named(g.turn_number, seat, &action, cast_name) {
                action = crate::game::GameAction::PassPriority;
                settled = None;
            }
            // CR 723.5 — a controlled seat plays the controller's policy; an
            // override the rules reject falls back to the seat's own action.
            let mut fallback = None;
            if let Some(forced) = controlled::controller_override(&g, seat, &action) {
                fallback = Some(std::mem::replace(&mut action, forced));
                settled = None;
            }
            let is_pass = matches!(action, crate::game::GameAction::PassPriority);
            // Keyed against the pre-action state: a cast names the card while
            // it is still in the zone it is cast from.
            let key = census.key_for(&g, seat, &action);
            let traced = trace_from.is_some_and(|n| actions >= n).then(|| trace_line(&g, &action));
            let ok = if let Some(settled) = settled {
                g = *settled;
                true
            } else {
                let result = match (g.perform_action(action), fallback) {
                    (Err(_), Some(own)) => g.perform_action(own),
                    (r, _) => r,
                };
                match result {
                    Ok(events) => {
                        g.recycle_events(events);
                        true
                    }
                    Err(e) => {
                        if let Some(a) = &traced {
                            let (t, step, act) = (g.turn_number, g.step, g.active_player_idx);
                            eprintln!("REJECT t{t} {step:?} active p{act} p{seat} {a} :: {e:?}");
                        }
                        false
                    }
                }
            };
            if ok {
                if let Some(a) = traced {
                    let life: Vec<i32> = g.players.iter().map(|p| p.life).collect();
                    eprintln!("{actions} t{} {:?} stack {} life {life:?} p{seat} {a}", g.turn_number, g.step, g.stack.len());
                    // A live seat at 0 or less life names what spares it (CR 704.5a).
                    for (i, p) in g.players.iter().enumerate().filter(|&(i, p)| p.is_alive() && g.effective_life(i) <= 0) {
                        eprintln!(
                            "  p{i} alive at {} life: cant_lose_game {} (this turn {}), unlife {}",
                            p.life,
                            g.player_cant_lose_game(i),
                            p.cant_lose_this_turn,
                            g.player_unlife_active(i),
                        );
                    }
                }
                census.bump(key);
                census.note_triggers(&g);
                #[cfg(debug_assertions)]
                check_commanders(&g, seed, actions, &mut commanders_seen);
                // CR 514.1 — the turn that just ended left its active player at
                // or under their maximum hand size. Debug-only.
                #[cfg(debug_assertions)]
                {
                    // The maximum as the turn's last action left it: CR 514.1
                    // discards before 514.2 ends "until end of turn" effects,
                    // so a borrowed Body of Knowledge ("no maximum hand size")
                    // covers the discard and is gone by the next turn.
                    if let Some((turn, prev, Some(max))) = last_turn
                        && turn != g.turn_number
                        && !g.is_game_over()
                        && g.players.get(prev).is_some_and(|p| p.is_alive())
                    {
                        assert!(
                            g.players[prev].hand.len() <= max + g.players[prev].cards_drawn_this_turn as usize,
                            "seed {seed}: p{prev} ended turn {turn} with {} cards in hand, max {max} (after {actions} actions, now {:?})",
                            g.players[prev].hand.len(),
                            g.step,
                        );
                    }
                    // CR 514.2 — and its cleanup removed every permanent's
                    // marked damage (phased-out ones too) and ended every
                    // "until end of turn" effect; "until end of combat" ended
                    // before that (CR 511.3). A turn that ends without its
                    // cleanup, or a cleanup window (CR 514.3a) that marks
                    // damage and is not followed by another, leaves one.
                    if let Some((turn, ..)) = last_turn
                        && turn != g.turn_number
                        && !g.is_game_over()
                    {
                        use crate::game::layers::EffectDuration;
                        if let Some(c) = g
                            .battlefield
                            .iter()
                            .chain(g.phased_out.iter())
                            .find(|c| {
                                // Ancient Adamantoise keeps its damage (CR 514.2's own exception).
                                let keeps = c.definition.static_abilities.iter().any(|sa| {
                                    matches!(sa.effect, crate::effect::StaticEffect::KeepsDamageThroughCleanup)
                                });
                                (c.damage != 0 && !keeps) || !c.end_of_turn_effects_are_clear()
                            })
                        {
                            panic!(
                                "seed {seed}: {} {:?} kept {} damage / an end-of-turn bonus past turn {turn}'s cleanup (after {actions} actions, now {:?})",
                                c.definition.name, c.id, c.damage, g.step,
                            );
                        }
                        if let Some(e) = g.continuous_effects.iter().find(|e| {
                            matches!(e.duration, EffectDuration::UntilEndOfTurn | EffectDuration::UntilEndOfCombat)
                        }) {
                            panic!(
                                "seed {seed}: a {:?} effect from {:?} outlived turn {turn}'s cleanup (after {actions} actions, now {:?})",
                                e.duration, e.source, g.step,
                            );
                        }
                        // CR 302.6 — the new active player has controlled
                        // each of their permanents since their turn began.
                        let ap = g.active_player_idx;
                        if let Some(c) = g.battlefield.iter().find(|c| c.controller == ap && c.summoning_sick) {
                            panic!(
                                "seed {seed}: p{ap}'s {} {:?} is still summoning sick in turn {} (after {actions} actions, {:?})",
                                c.definition.name, c.id, g.turn_number, g.step,
                            );
                        }
                        // CR 500.4 — and every step change since emptied the
                        // pools, save what a retention static keeps.
                        let keeps = g.battlefield.iter().any(|c| {
                            c.definition.static_abilities.iter().any(|sa| {
                                matches!(
                                    sa.effect,
                                    crate::effect::StaticEffect::UnspentManaBecomesColorless
                                        | crate::effect::StaticEffect::UnspentManaBecomesBlack
                                        | crate::effect::StaticEffect::UnspentManaBecomesRed
                                        | crate::effect::StaticEffect::ManaPoolsNeverEmpty
                                        | crate::effect::StaticEffect::UnspentColorManaPersists(_)
                                )
                            })
                        });
                        // Braid of Fire pays its cumulative upkeep with mana
                        // in the new turn's upkeep (CR 702.24a).
                        let upkeep_mana = |i: usize| {
                            i == ap
                                && g.step == crate::game::types::TurnStep::Upkeep
                                && g.battlefield.iter().any(|c| {
                                    c.controller == i
                                        && c.definition.keywords.iter().any(|k| {
                                            matches!(
                                                k,
                                                crate::card::Keyword::CumulativeUpkeep(
                                                    crate::card::CumulativeUpkeepCost::AddMana(_)
                                                )
                                            )
                                        })
                                })
                        };
                        if !keeps
                            && let Some((i, p)) = g.players.iter().enumerate().find(|(i, p)| {
                                p.is_alive()
                                    && !p.mana_pool.is_empty()
                                    && p.kept_mana_this_turn.is_empty()
                                    && p.firebending_kept_red == 0
                                    && !upkeep_mana(*i)
                            })
                        {
                            panic!(
                                "seed {seed}: p{i} carried {} mana past turn {turn} (after {actions} actions, now {:?})",
                                p.mana_pool.total(),
                                g.step,
                            );
                        }
                        // CR 511.3 / 508.5 — and its end of combat removed the
                        // attacked permanents' defender record with the attackers.
                        if let Some((pw, d)) = g.attacked_permanent_defenders.first() {
                            panic!(
                                "seed {seed}: {pw:?}'s attack record (defender p{d}) outlived turn {turn} (after {actions} actions, now {:?})",
                                g.step,
                            );
                        }
                    }
                    last_turn =
                        Some((g.turn_number, g.active_player_idx, g.effective_max_hand_size(g.active_player_idx)));
                }
                any = true;
                actions += 1;
                plays += usize::from(!is_pass);
                pass_streak = if is_pass { pass_streak + 1 } else { 0 };
                if let Some((rate, rng)) = concede.as_mut()
                    && !g.is_game_over()
                    && rng.random_range(0..10_000u32) < *rate
                {
                    let live: Vec<usize> = g.living_seats().collect();
                    if let Some(&quitter) = live.get(rng.random_range(0..live.len().max(1))) {
                        if trace_from.is_some_and(|n| actions >= n) {
                            eprintln!("{actions} t{} {:?} p{quitter} CONCEDES", g.turn_number, g.step);
                        }
                        let events = g.concede(quitter);
                        g.recycle_events(events);
                    }
                }
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
            "seed {seed}: seat {} has left the game and still owes an ask: {:?}",
            g.pending_decision.as_ref().map(|pd| pd.acting_player()).unwrap_or(usize::MAX),
            g.pending_decision.as_ref().map(|pd| &pd.decision),
        );
        // CR 400.1 — one object, one zone: the golden traces check this in
        // duels only, and a pod reaches far more cards. Debug-only.
        #[cfg(debug_assertions)]
        if let Some(dup) = g.duplicate_zone_id() {
            panic!("seed {seed}: card {dup:?} is in two zones (turn {}, after {actions} actions)", g.turn_number);
        }
        // CR 704.3 — once an action has settled the state-based actions have
        // run: no living player at 0 life (704.5a), no creature at 0
        // toughness (704.5f), no 0-loyalty planeswalker (704.5i) and no
        // permanent attached to a host that is gone (704.5m/n). Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() && g.pending_decision.is_none() && g.suspend_signal.is_none() {
            // CR 704.5a — a living player at 0 or less life who can lose has lost.
            if let Some(i) = (0..g.players.len()).find(|&i| {
                g.players[i].is_alive()
                    && g.effective_life(i) <= 0
                    && !g.player_cant_lose_game(i)
                    && !g.player_unlife_active(i)
            }) {
                panic!(
                    "seed {seed}: p{i} is alive at {} life after the sweep (turn {}, {:?}, after {actions} actions)",
                    g.effective_life(i), g.turn_number, g.step,
                );
            }
            for c in g.battlefield.iter() {
                if let Some(cp) = g.computed_permanent(c.id)
                    && cp.card_types().contains(&crate::card::CardType::Creature)
                    && cp.toughness <= 0
                {
                    let mut probe = g.clone();
                    probe.check_state_based_actions();
                    panic!(
                        "seed {seed}: {} is a {}-toughness creature after the sweep (turn {}, {:?}, after {actions} actions; counters {:?}; a fresh sweep removes it: {})",
                        c.definition.name,
                        cp.toughness,
                        g.turn_number,
                        g.step,
                        c.counters,
                        probe.battlefield_find(c.id).is_none(),
                    );
                }
                if c.definition.is_planeswalker()
                    && c.counter_count(crate::card::CounterType::Loyalty) == 0
                    && !g.planeswalkers_survive_zero_loyalty(c.controller)
                    && g.computed_permanent(c.id).is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Planeswalker))
                {
                    panic!(
                        "seed {seed}: {} is a 0-loyalty planeswalker after the sweep (turn {}, {:?}, after {actions} actions)",
                        c.definition.name, g.turn_number, g.step,
                    );
                }
                if let Some(h) = c.attached_to
                    && g.battlefield_find(h).is_none()
                {
                    let zone = zone_label(&g, h);
                    panic!(
                        "seed {seed}: {} is attached to {h:?}, which is in {zone} (turn {}, {:?}, after {actions} actions)",
                        c.definition.name, g.turn_number, g.step,
                    );
                }
            }
        }
        // CR 506.4 — an attacker or blocker that left the battlefield, phased
        // out, changed controller or stopped being a creature is removed from
        // combat. Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            let name = |id: crate::card::CardId| g.battlefield_find(id).map_or_else(|| "<gone>".to_string(), |c| c.definition.name.to_string());
            let in_combat = |id: crate::card::CardId, want: Option<usize>| {
                g.battlefield_find(id).is_some_and(|c| want.is_none_or(|p| c.controller == p))
                    && g.computed_permanent(id)
                        .is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Creature))
            };
            let active = g.active_player_idx;
            if let Some(a) = g.attacking.iter().find(|a| !in_combat(a.attacker, Some(active))) {
                panic!(
                    "seed {seed}: {:?} ({}) is still attacking after leaving combat (turn {}, {:?}, after {actions} actions)",
                    a.attacker,
                    name(a.attacker),
                    g.turn_number,
                    g.step,
                );
            }
            if let Some(&b) = g.block_map.keys().find(|&&b| !in_combat(b, None)) {
                panic!(
                    "seed {seed}: {b:?} ({}) is still blocking after leaving combat (turn {}, {:?}, after {actions} actions)",
                    name(b),
                    g.turn_number,
                    g.step,
                );
            }
        }
        // CR 511.3 — outside the combat phase nothing is attacking or
        // blocking (an "enters attacking" effect outside combat does nothing,
        // CR 506.3). Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over()
            && !matches!(
                g.step,
                crate::game::types::TurnStep::DeclareAttackers
                    | crate::game::types::TurnStep::DeclareBlockers
                    | crate::game::types::TurnStep::FirstStrikeDamage
                    | crate::game::types::TurnStep::CombatDamage
                    | crate::game::types::TurnStep::EndCombat
            )
            && (!g.attacking.is_empty() || !g.block_map.is_empty())
        {
            panic!(
                "seed {seed}: {} attackers / {} blockers outside combat (turn {}, {:?}, after {actions} actions)",
                g.attacking.len(),
                g.block_map.len(),
                g.turn_number,
                g.step,
            );
        }
        // CR 506.2 / 508.1b — an attacker attacking a player attacks a living
        // opponent of its controller; CR 506.4 — a blocker's attackers are all
        // still attacking. Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            for a in g.attacking.iter() {
                if let crate::game::types::AttackTarget::Player(q) = a.target
                    && let Some(c) = g.battlefield_find(a.attacker)
                    && !g.opponents_of(c.controller).contains(&q)
                {
                    panic!(
                        "seed {seed}: {} {:?} (p{}) attacks p{q}, no living opponent (turn {}, {:?}, after {actions} actions)",
                        c.definition.name,
                        a.attacker,
                        c.controller,
                        g.turn_number,
                        g.step,
                    );
                }
            }
            if let Some((b, atk)) = g.block_map.iter().find_map(|(b, atks)| {
                atks.iter().find(|x| !g.attacking.iter().any(|a| a.attacker == **x)).map(|x| (*b, *x))
            }) {
                panic!(
                    "seed {seed}: {b:?} blocks {atk:?}, which is not attacking (turn {}, {:?}, after {actions} actions)",
                    g.turn_number,
                    g.step,
                );
            }
        }
        // CR 704.5m / 800.4a — an Aura on a player is on one still in the
        // game; CR 701.42 — a melded permanent's halves were exiled into it,
        // so they hold no battlefield state of their own. Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            if let Some(c) = g
                .battlefield
                .iter()
                .find(|c| c.attached_to_player.is_some_and(|p| !g.players.get(p).is_some_and(|q| q.is_alive())))
            {
                panic!(
                    "seed {seed}: {} {:?} is attached to p{:?}, who has left (turn {}, after {actions} actions)",
                    c.definition.name, c.id, c.attached_to_player, g.turn_number,
                );
            }
            if let Some((shell, part)) = g.battlefield.iter().find_map(|c| {
                c.meld_parts.iter().find(|p| p.tapped || p.attached_to.is_some() || !p.counters.is_empty()).map(|p| (c, p))
            }) {
                panic!(
                    "seed {seed}: {}'s half {} kept battlefield state (tapped {}, counters {:?}; turn {}, after {actions} actions)",
                    shell.definition.name,
                    part.definition.name,
                    part.tapped,
                    part.counters.iter().collect::<Vec<_>>(),
                    g.turn_number,
                );
            }
        }
        // CR 408.1 / 903.3 — a command zone holds its own player's objects,
        // and a card there is a commander or a card that lives there (a
        // plane, phenomenon, scheme, conspiracy or vanguard). Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            use crate::card::CardType as T;
            for (i, p) in g.players.iter().enumerate() {
                if let Some(c) = p.command.iter().find(|c| {
                    c.owner != i
                        || (!p.commanders.contains(&c.id)
                            && !c.definition.card_types.iter().any(|t| {
                                matches!(t, T::Plane | T::Phenomenon | T::Scheme | T::Conspiracy | T::Vanguard)
                            }))
                }) {
                    panic!(
                        "seed {seed}: {} {:?} (owner p{}) is in p{i}'s command zone (turn {}, after {actions} actions)",
                        c.definition.name, c.id, c.owner, g.turn_number,
                    );
                }
            }
        }
        // CR 725.4 / 726.4 — the monarch and the initiative pass on when their
        // holder leaves; neither stays with a seat that has left. Debug-only.
        #[cfg(debug_assertions)]
        // As for the ownership check below: a departure deferred past a
        // pending ask hands the monarchy on once it runs (a win drawn off an
        // empty library under Chulane, fuzzed audit pod, seed 3206512 game 19).
        if !g.is_game_over()
            && (g.pending_decision.is_none() || !g.players.iter().any(|p| p.eliminated && !p.left_game))
        {
            for (what, holder) in [("monarch", g.monarch), ("initiative", g.initiative)] {
                if let Some(p) = holder.filter(|&p| !g.players.get(p).is_some_and(|q| q.is_alive())) {
                    panic!("seed {seed}: p{p} left holding the {what} (turn {}, after {actions} actions)", g.turn_number);
                }
            }
        }
        // CR 400.7 — a card that changed zones is a new object: off the
        // battlefield it is untapped, unpumped and attached to nothing, and
        // carries no counters but the printed loyalty / defense it was built
        // with (exile keeps suspend's time counters). Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            // Which zone a card sits in, for what it may keep there.
            #[derive(Clone, Copy, PartialEq)]
            enum Off {
                Hand,
                Library,
                Graveyard,
                Command,
                Exile,
            }
            let stale = |c: &crate::card::CardInstance, z: Off| {
                let counters_kept = match z {
                    // Suspend's time counters, a hideaway card, …
                    Off::Exile => true,
                    // CR 122.2 — a card that keeps its counters (Skullbriar),
                    // into the command zone too: only a hand or library strips them.
                    Off::Graveyard | Off::Command => c.definition.keeps_counters_off_battlefield,
                    Off::Hand | Off::Library => false,
                };
                // Not `damage`: it is cleared as a card enters, not as it
                // leaves (ENGINE_BACKLOG 2026-09-08, a priced CoW write).
                c.tapped
                    || c.attached_to.is_some()
                    || c.attached_to_player.is_some()
                    || c.perm_power_bonus != 0
                    || c.perm_toughness_bonus != 0
                    || c.echo_paid
                    // CR 708.9 / 712.8 — face up and front face up off the
                    // battlefield (exile keeps foretold / hidden cards down).
                    || (z != Off::Exile && c.face_down)
                    || c.transformed
                    // CR 710.4 / 709.5c / 716 / MKM — flip, Room, Class and
                    // Case designations are battlefield-only; so are a soulbond
                    // pair, a phasing link and this turn's loyalty activations
                    // (a planeswalker back the same turn is a new object).
                    || c.flipped
                    || c.unlocked_doors != 0
                    || c.class_level != 0
                    || c.case_solved
                    || c.soulbond_partner.is_some()
                    || c.phased_out_by.is_some()
                    || c.loyalty_uses_this_turn != 0
                    || c.loyalty_twice_this_turn
                    // CR 701.37b / 702.112b / 701.60a / 701.64b — monstrous,
                    // renowned, suspected and harnessed end as it leaves.
                    || c.monstrous
                    || c.renowned
                    || c.suspected
                    || c.harnessed
                    // CR 400.7 — an "as this enters, choose" answer belongs to
                    // the permanent (exile may hold a link's chosen card; a
                    // spell's gift recipient, `chosen_player`, rides along).
                    || (z != Off::Exile
                        && (c.chosen_creature_type.is_some()
                            || c.chosen_land_type.is_some()
                            || c.chosen_number.is_some()
                            || c.chosen_card_type.is_some()
                            || c.chosen_permanent.is_some()))
                    || (!counters_kept
                        && c.counters.iter().any(|(k, &n)| match k {
                            // A new instance is built holding its printed
                            // loyalty / defense (`CardInstance::new`).
                            crate::card::CounterType::Loyalty => n != c.definition.base_loyalty,
                            crate::card::CounterType::Defense => n != c.definition.defense,
                            _ => true,
                        }))
            };
            let found = g
                .players
                .iter()
                .flat_map(|p| {
                    p.hand
                        .iter()
                        .map(|c| (c, Off::Hand))
                        .chain(p.library.iter().map(|c| (c, Off::Library)))
                        .chain(p.graveyard.iter().map(|c| (c, Off::Graveyard)))
                        .chain(p.command.iter().map(|c| (c, Off::Command)))
                })
                .chain(g.exile.iter().map(|c| (c, Off::Exile)))
                .find(|(c, z)| stale(c, *z));
            if let Some((c, _)) = found {
                panic!(
                    "seed {seed}: {} {:?} in {} kept battlefield state (tapped {}, pump {}/{}, echo {}, face down {}, transformed {}, attached {:?}/{:?}, counters {:?}; flipped {} doors {} class {} case {} soulbond {:?} phased-by {:?} loyalty uses {}/{} chosen {:?}/{:?}/{:?}/{:?}/{:?} designations {}/{}/{}/{}; turn {}, after {actions} actions)",
                    c.definition.name,
                    c.id,
                    zone_label(&g, c.id),
                    c.tapped,
                    c.perm_power_bonus,
                    c.perm_toughness_bonus,
                    c.echo_paid,
                    c.face_down,
                    c.transformed,
                    c.attached_to,
                    c.attached_to_player,
                    c.counters.iter().collect::<Vec<_>>(),
                    c.flipped,
                    c.unlocked_doors,
                    c.class_level,
                    c.case_solved,
                    c.soulbond_partner,
                    c.phased_out_by,
                    c.loyalty_uses_this_turn,
                    c.loyalty_twice_this_turn,
                    c.chosen_creature_type,
                    c.chosen_land_type,
                    c.chosen_number,
                    c.chosen_card_type,
                    c.chosen_permanent,
                    c.monstrous,
                    c.renowned,
                    c.suspected,
                    c.harnessed,
                    g.turn_number,
                );
            }
            // CR 400.7 — links and spell choices a new object doesn't carry:
            // an "until this leaves" exile link outside exile, a may-play
            // permission on a card in a hand or command zone ("that card"
            // stops naming it once it moves; a miracle window excepted), a kicked flag off the
            // battlefield, stack and exile.
            let linked = g
                .players
                .iter()
                .flat_map(|p| {
                    p.hand
                        .iter()
                        .map(|c| (c, Off::Hand))
                        .chain(p.library.iter().map(|c| (c, Off::Library)))
                        .chain(p.graveyard.iter().map(|c| (c, Off::Graveyard)))
                        .chain(p.command.iter().map(|c| (c, Off::Command)))
                })
                .find(|(c, z)| {
                    // CR 702.94 — a miracle reveal's permission IS a hand
                    // permission, live for the step it was drawn in.
                    // A grant made to a card in hand for this turn (Zaffai and
                    // the Tempests' free cast) is a hand permission too.
                    c.exiled_by.is_some()
                        || (matches!(z, Off::Hand | Off::Command)
                            && c.may_play_until.as_ref().is_some_and(|m| {
                                !m.miracle
                                    && !(matches!(z, Off::Hand)
                                        && m.duration == crate::card::MayPlayDuration::EndOfThisTurn
                                        && m.granted_turn == g.turn_number)
                            }))
                        || c.kicked
                });
            if let Some((c, _)) = linked.filter(|_| g.pending_decision.is_none()) {
                panic!(
                    "seed {seed}: {} {:?} in {} kept a link (exiled_by {:?}, may_play {}, kicked {}; turn {}, {:?}, after {actions} actions)",
                    c.definition.name,
                    c.id,
                    zone_label(&g, c.id),
                    c.exiled_by.map(|l| l.source),
                    c.may_play_until.is_some(),
                    c.kicked,
                    g.turn_number,
                    g.step,
                );
            }
            // CR 704.5d / 704.5e — a token or a copy of a card off the
            // battlefield and the stack ceases to exist at the next sweep, so
            // none survives a settled action (one may wait out a paused one).
            let token = g
                .players
                .iter()
                .flat_map(|p| p.hand.iter().chain(p.library.iter()).chain(p.graveyard.iter()).chain(p.command.iter()))
                .chain(g.exile.iter())
                .find(|c| c.is_token);
            if let Some(c) = token.filter(|_| g.pending_decision.is_none()) {
                panic!(
                    "seed {seed}: token {} {:?} survived in {} (turn {}, {:?}, after {actions} actions)",
                    c.definition.name,
                    c.id,
                    zone_label(&g, c.id),
                    g.turn_number,
                    g.step,
                );
            }
            // CR 800.4a — a departed player's spells and abilities on the
            // stack cease to exist with them.
            let orphan = g.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Spell { card, caster, .. } => Some((card.definition.name, *caster)),
                crate::game::types::StackItem::Trigger { source, controller, .. } => {
                    Some((g.find_card_anywhere(*source).map_or("?", |c| c.definition.name), *controller))
                }
            }.filter(|&(_, p)| !g.players.get(p).is_some_and(|q| q.is_alive())));
            if let Some((name, p)) = orphan {
                panic!(
                    "seed {seed}: {name} on the stack is controlled by departed p{p} ({:?}) (turn {}, {:?}, after {actions} actions)",
                    g.players.get(p).and_then(|q| q.loss_cause),
                    g.turn_number, g.step,
                );
            }
            // CR 400.7 — a spell is a new object too: it carries no
            // battlefield state onto the stack (a face-down or transformed
            // cast is the spell's own characteristics, so those may stand).
            let spell = g.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Spell { card, .. }
                    if card.tapped
                        || card.attached_to.is_some()
                        || card.attached_to_player.is_some()
                        || card.perm_power_bonus != 0
                        || card.perm_toughness_bonus != 0
                        || card.flipped
                        || card.class_level != 0
                        || card.case_solved
                        || card.soulbond_partner.is_some()
                        || card.phased_out_by.is_some()
                        || card.loyalty_uses_this_turn != 0
                        // CR 122.2 — a card that keeps its counters (Me, the Immortal).
                        || (!card.definition.keeps_counters_off_battlefield
                            && card.counters.iter().any(|(k, &n)| match k {
                            crate::card::CounterType::Loyalty => n != card.definition.base_loyalty,
                            crate::card::CounterType::Defense => n != card.definition.defense,
                            _ => n != 0,
                        })) =>
                {
                    Some(card)
                }
                _ => None,
            });
            if let Some(c) = spell {
                panic!(
                    "seed {seed}: spell {} {:?} on the stack kept battlefield state (tapped {}, pump {}/{}, attached {:?}/{:?}, counters {:?}, flipped {} class {} case {} soulbond {:?} phased-by {:?} loyalty uses {}; turn {}, after {actions} actions)",
                    c.definition.name,
                    c.id,
                    c.tapped,
                    c.perm_power_bonus,
                    c.perm_toughness_bonus,
                    c.attached_to,
                    c.attached_to_player,
                    c.counters.iter().collect::<Vec<_>>(),
                    c.flipped,
                    c.class_level,
                    c.case_solved,
                    c.soulbond_partner,
                    c.phased_out_by,
                    c.loyalty_uses_this_turn,
                    g.turn_number,
                );
            }
        }
        // CR 509.1a — a blocker is controlled by the defending player of each
        // attacker it blocks (control changes remove it, CR 506.4); CR 400.3 —
        // a card in a hand, library, graveyard or command zone is in its
        // owner's. Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            for (&b, atks) in g.block_map.iter() {
                let Some(bc) = g.battlefield_find(b).map(|c| c.controller) else { continue };
                for a in atks.iter().filter_map(|&a| g.attack_for(a)) {
                    let defender = match a.target {
                        crate::game::types::AttackTarget::Player(q) => Some(q),
                        crate::game::types::AttackTarget::Planeswalker(id)
                        | crate::game::types::AttackTarget::Battle(id) => g
                            .attacked_permanent_defenders
                            .iter()
                            .find(|(c, _)| *c == id)
                            .map(|&(_, d)| d)
                            .or_else(|| g.defender_for(a.target)),
                    };
                    assert!(
                        defender.is_none_or(|d| d == bc),
                        "seed {seed}: p{bc}'s {} blocks {:?}, which attacks p{defender:?} (turn {}, {:?}, after {actions} actions)",
                        g.battlefield_find(b).map_or("?", |c| c.definition.name),
                        a.attacker,
                        g.turn_number,
                        g.step,
                    );
                }
            }
            for (i, p) in g.players.iter().enumerate() {
                for (zone, cards) in
                    [("hand", &p.hand[..]), ("library", &p.library[..]), ("graveyard", &p.graveyard[..]), ("command zone", &p.command[..])]
                {
                    if let Some(c) = cards.iter().find(|c| c.owner != i) {
                        panic!(
                            "seed {seed}: p{}'s {} is in p{i}'s {zone} (turn {}, {:?}, after {actions} actions)",
                            c.owner, c.definition.name, g.turn_number, g.step,
                        );
                    }
                }
            }
        }
        // CR 613.7d — every permanent carries its entry timestamp once the
        // action is done; an unstamped one reads as its `CardId`, older than
        // everything (the resolving-spell and token paths were). Debug-only.
        #[cfg(debug_assertions)]
        if let Some(c) = g.battlefield.iter().find(|c| c.battlefield_timestamp == 0) {
            panic!(
                "seed {seed}: {} ({:?}) is on the battlefield with no timestamp (turn {}, {:?}, after {actions} actions)",
                c.definition.name,
                c.id,
                g.turn_number,
                g.step,
            );
        }
        // CR 610.3 — a card exiled "until [source] leaves the battlefield"
        // returns once the source has left (a phased-out source hasn't). A
        // printed leave TRIGGER returns it as it resolves, so only with the
        // stack empty. Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over()
            && g.stack.is_empty()
            && g.pending_decision.is_none()
            && g.suspend_signal.is_none()
            && let Some((c, link)) = g.exile.iter().find_map(|c| {
                c.exiled_by
                    .filter(|l| {
                        l.monarch_guard.is_none()
                            && g.battlefield_find(l.source).is_none()
                            && !g.phased_out.iter().any(|p| p.id == l.source)
                    })
                    .map(|l| (c, l))
            })
        {
            panic!(
                "seed {seed}: {} is still exiled until {:?} leaves, which is in {} (turn {}, {:?}, after {actions} actions)",
                c.definition.name,
                link.source,
                zone_label(&g, link.source),
                g.turn_number,
                g.step,
            );
        }
        // CR 800.4a — priority passes on from a player who left. Debug-only.
        #[cfg(debug_assertions)]
        if !g.is_game_over() {
            let holder = g.priority.player_with_priority;
            assert!(
                g.players.get(holder).is_some_and(|p| p.is_alive()),
                "seed {seed}: departed p{holder} holds priority (turn {}, {:?}, after {actions} actions)",
                g.turn_number,
                g.step,
            );
        }
        #[cfg(debug_assertions)]
        if !g.is_game_over() && g.pending_decision.is_none() && g.suspend_signal.is_none() {
            let now = owned_card_counts(&g);
            if let Some(p) = (0..g.players.len()).find(|&p| g.players[p].is_alive() && now[p] != owned_at_start[p]) {
                let gone: Vec<String> = owned_at_start[p]
                    .iter()
                    .filter(|id| now[p].binary_search(id).is_err())
                    .map(|id| format!("{id:?} {:?}", g.find_card_anywhere(*id).map(|c| (c.definition.name, c.owner))))
                    .collect();
                let new: Vec<String> = now[p]
                    .iter()
                    .filter(|id| owned_at_start[p].binary_search(id).is_err())
                    .map(|id| format!("{id:?} {:?}", g.find_card_anywhere(*id).map(|c| (c.definition.name, c.owner))))
                    .collect();
                panic!(
                    "seed {seed}: p{p} owns {} nontoken cards, {} at the start (turn {}, {:?}, after {actions} actions): gone {gone:?}, new {new:?}",
                    now[p].len(), owned_at_start[p].len(), g.turn_number, g.step,
                );
            }
        }
        // CR 704.3 — the sweep repeats until a pass does nothing, so a second
        // sweep over a settled state finds nothing either. Debug-only, opt-in.
        #[cfg(debug_assertions)]
        if pod_sweep_probe()
            && !g.is_game_over()
            && g.pending_decision.is_none()
            && g.suspend_signal.is_none()
        {
            let mut probe = g.clone();
            let events = probe.check_state_based_actions();
            if !events.is_empty() || probe.pending_decision.is_some() {
                // The creatures it kills, by name, P/T and attachments.
                let died: Vec<String> = events
                    .iter()
                    .filter_map(|e| match e {
                        crate::game::GameEvent::CreatureDied { card_id } => g.battlefield_find(*card_id).map(|c| {
                            let pt = g.computed_permanent(c.id).map(|cp| (cp.power, cp.toughness, c.damage));
                            let on: Vec<&str> =
                                g.battlefield.iter().filter(|a| a.attached_to == Some(c.id)).map(|a| a.definition.name).collect();
                            format!("{} {pt:?} attached {on:?}", c.definition.name)
                        }),
                        _ => None,
                    })
                    .collect();
                panic!(
                    "seed {seed}: a fresh sweep still acts after {actions} actions (turn {}, {:?}, stack {}): {events:?} {died:?}; asks {:?}",
                    g.turn_number,
                    g.step,
                    g.stack.len(),
                    probe.pending_decision.as_ref().map(|p| &p.decision),
                );
            }
        }
        // CR 800.4a — a player who left took every object they owned, and
        // control they held over another player's object ended. Debug-only.
        // A loss marked mid-resolution runs its departure pass once the
        // action settles, so a pending ask defers the check (a win drawn off
        // an empty library while a stolen Chulane's ask was open, fuzzed
        // audit pod, seed 3195569 game 17).
        #[cfg(debug_assertions)]
        if !g.is_game_over()
            && (g.pending_decision.is_none() || !g.players.iter().any(|p| p.eliminated && !p.left_game))
        {
            let gone = |p: usize| g.players.get(p).is_none_or(|pl| !pl.is_alive());
            if let Some(c) = g.battlefield.iter().find(|c| gone(c.owner) || gone(c.controller)) {
                panic!(
                    "seed {seed}: {} (owner p{}, controller p{}) outlived a departed seat (turn {}, {:?}, after {actions} actions; left {:?}; pending {:?})",
                    c.definition.name,
                    c.owner,
                    c.controller,
                    g.turn_number,
                    g.step,
                    g.players.iter().map(|p| (p.eliminated, p.left_game)).collect::<Vec<_>>(),
                    g.pending_decision.as_ref().map(|p| &p.decision),
                );
            }
            if let Some((src, controller)) = g.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Trigger { source, controller, .. } if gone(*controller) => {
                    Some((*source, *controller))
                }
                _ => None,
            }) {
                panic!(
                    "seed {seed}: a trigger of {src:?} controlled by departed p{controller} is on the stack (turn {})",
                    g.turn_number,
                );
            }
            if let Some((c, caster)) = g.stack.iter().find_map(|si| match si {
                crate::game::types::StackItem::Spell { card, caster, .. } if gone(*caster) || gone(card.owner) => {
                    Some((card, *caster))
                }
                _ => None,
            }) {
                panic!(
                    "seed {seed}: spell {} (owner p{}, caster p{caster}) outlived a departed seat (turn {})",
                    c.definition.name, c.owner, g.turn_number,
                );
            }
        }
        // CR 903.9b — every pod seat answers the library redirect "yes", so a
        // commander in a library and not waiting on its offer reached it past
        // `resolve_zone_change` (an inline push the replacement never saw).
        // Debug-only; off under fuzz, whose seats may decline (CR 903.9b allows it).
        #[cfg(debug_assertions)]
        if fuzz::fuzz_rate().is_none()
            && let Some(c) = g
            .players
            .iter()
            .flat_map(|p| p.library.iter())
            .find(|c| {
                g.players.iter().any(|q| q.commanders.contains(&c.id)) && !g.commander_redirect_offers.contains(&c.id)
            })
        {
            panic!(
                "seed {seed}: commander {} reached a library past the CR 903.9b replacement (turn {}, after {actions} actions)",
                c.definition.name, g.turn_number,
            );
        }
        if any { stale = 0 } else { stale += 1 }
        // `CRAB_CAP_DIAG=<n>` names a *slow* game's board too, once, as it
        // passes `n` actions — a decided game never reaches the line below.
        if let Some(n) = diag_floor.filter(|n| actions >= *n && !diag_said) {
            diag_said = true;
            eprintln!("pod past {n} actions seed {seed}: {}", crate::recommend::cap_diagnosis(&g, actions));
        }
    }
    crate::server::bot::set_jitter_seed(None);
    let stop = stop_reason(&g, spent(actions, plays, pass_streak), max_actions, stale).unwrap_or(StopReason::NoLegalMove);
    // `CRAB_CAP_DIAG` is the two-player loop's knob and it says the same thing
    // here: what was an undecided game actually doing. One `OnceLock` read a
    // game, on the undecided ones only.
    // A draw (CR 104.4a) is undecided too, so it gets the board as well.
    let undecided = !matches!(stop, StopReason::GameOver) || g.game_over == Some(None);
    if crate::recommend::cap_diag_floor().is_some() && undecided {
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

/// A pod A/B's result: one pilot (the *hero*) in one seat, another in every
/// other seat. Kept per deal group — the `n` games of one deal with the hero
/// in each seat once — because the group, not the game, is the independent
/// unit (its games share decks and shuffle).
#[derive(Debug, Default, Clone)]
pub struct HeroTally {
    pub tally: PodTally,
    /// Games the hero's seat won.
    pub hero_wins: u32,
    /// `group -> (hero wins, games played)` for every group touched.
    pub groups: std::collections::BTreeMap<u32, (u32, u32)>,
}

impl HeroTally {
    /// Fold another worker's tally in; a group split across workers joins up.
    pub fn merge(&mut self, other: &HeroTally) {
        self.tally.merge(&other.tally);
        self.hero_wins += other.hero_wins;
        for (g, (w, n)) in &other.groups {
            let e = self.groups.entry(*g).or_default();
            e.0 += w;
            e.1 += n;
        }
    }

    /// The hero's share of games won, and its standard error over the deal
    /// groups (independent draws; the games inside one are not). Equal
    /// pilots win `1 / seats` of the games.
    pub fn share(&self) -> (f64, f64) {
        let shares: Vec<f64> =
            self.groups.values().filter(|(_, n)| *n > 0).map(|(w, n)| f64::from(*w) / f64::from(*n)).collect();
        let k = shares.len() as f64;
        if k == 0.0 {
            return (0.0, 0.0);
        }
        let mean = shares.iter().sum::<f64>() / k;
        let var = if k > 1.0 { shares.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (k - 1.0) } else { 0.0 };
        (mean, (var / k).sqrt())
    }
}

/// Play pod A/B games `first .. first + count`: `hero` in one seat, `field`
/// in the rest. Game `i` is deal group `i / n` with the hero in seat `i % n`,
/// so every seat hosts the hero once per deal — the seat advantage and the
/// deal cancel inside a group, the pod version of the two-player ladder's
/// seat-swapped pairs. Group `g` rotates the decks by `g % n` and shuffles
/// off a seed that is a pure function of `seed_base` and `g`, so a worker
/// pool can split the range anywhere.
pub fn run_pod_hero_games(
    decks: &[PodDeck],
    first: u32,
    count: u32,
    seed_base: u64,
    max_actions: usize,
    hero: Pilot,
    field: Pilot,
) -> HeroTally {
    let n = decks.len();
    let mut out = HeroTally { tally: PodTally { wins: vec![0; n], ..Default::default() }, ..Default::default() };
    if n < 2 {
        return out;
    }
    let templates: Vec<GameState> = (0..n)
        .map(|rot| {
            let seated: Vec<PodDeck> = (0..n).map(|seat| decks[(seat + n - rot) % n]).collect();
            build_pod_template(&seated)
        })
        .collect();
    for i in first..first.saturating_add(count) {
        let (group, hero_seat) = (i / n as u32, i as usize % n);
        let rot = group as usize % n;
        let seed = seed_base.wrapping_add(u64::from(group).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let pilots: Vec<Pilot> = (0..n).map(|s| if s == hero_seat { hero } else { field }).collect();
        let o = play_one_pod_game(&templates[rot], &pilots, max_actions, seed);
        let won = o.winner == Some(hero_seat) && o.stop == StopReason::GameOver;
        out.hero_wins += u32::from(won);
        let e = out.groups.entry(group).or_default();
        e.0 += u32::from(won);
        e.1 += 1;
        // Wins by seat, as the smoke test tallies them.
        out.tally.record(i, &o);
    }
    out
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
    /// CR 601.2g — a `{1}, {T}` mana source must not pay its own {1}. The
    /// payment snapshot was taken before the {T} cost and the restricted-source
    /// search (Vedalken Engineer's artifact-only mana, allowed on an artifact's
    /// ability) restored it, untapping Dimir Signet so it funded itself until
    /// the stack overflowed: this exact game, casting Breya. (A replay: once
    /// the bot or these decks change, the game stops reaching that board and
    /// this only proves the game still finishes.)
    /// CR 704.3 — the step advance into the next turn's upkeep sweeps before
    /// anyone has priority: a commander that went home across End → Upkeep
    /// left its Darksteel Mutation attached to nothing (a replay: debug
    /// builds assert the settled-board invariants after every action).
    #[test]
    fn a_step_advance_sweeps_before_upkeep_priority() {
        let d = target_decks();
        let field = [d[88], d[89], d[90], d[91]];
        let pilot = Pilot::Scored(crate::server::bot::EvalWeights::baseline());
        let t = run_pod_games(&field, 17, 1, 43022, 4_000, pilot);
        assert_eq!(t.games, 1);
    }

    /// CR 704.5j — two Vensers, Fervent Forger at one table: each seat's
    /// token-copy ETB took the other's Venser, whose tokens' ETBs copied it
    /// again, 1,538 triggers to the action cap. The copy now takes a
    /// nonlegendary permanent (a replay: it proves the game decides).
    #[test]
    fn two_vensers_do_not_copy_each_other_to_the_action_cap() {
        let d = target_decks();
        let field = [d[160], d[161], d[162], d[163]];
        let pilot = Pilot::Scored(crate::server::bot::EvalWeights::baseline());
        let t = run_pod_games(&field, 11, 1, 34040, 4_000, pilot);
        assert_eq!((t.games, t.undecided()), (1, 0));
    }

    #[test]
    fn a_mana_source_paying_for_itself_does_not_overflow() {
        let d = target_decks();
        let field = [d[50], d[47], d[44], d[38]];
        let pilot = Pilot::Scored(crate::server::bot::EvalWeights::baseline());
        let t = run_pod_games(&field, 544, 1, 10129, 4_000, pilot);
        assert_eq!(t.games, 1);
    }

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
        // One seeded game a seat: the test is the suite's long pole. Three a
        // seat had grown it past half an hour; at 741 seats the first 290
        // rows' second and third seeds took it to 55 minutes.
        let seats: &[(&str, &[u64])] = &[
            ("Teval", &[0x7E7A1_u64]),
            ("N'ghathrod", &[0xC1B]),
            ("Clavileño", &[0xB100D]),
            ("Zndrsplt", &[0xC0141]),
            ("Zada", &[0x2ADA]),
            ("Gisa", &[0x6154]),
            ("Ghalta", &[0x6A17A]),
            // CR 903.3a — a planeswalker commander with a lieutenant in its 99.
            ("Teferi", &[0x7EFE41]),
            ("Kalemne", &[0xCA1E]),
            ("Kardur", &[0xCA2D]),
            ("Kaalia", &[0xCAA1]),
            ("Zedruu", &[0x2ED7]),
            ("The Mimeoplasm", &[0x3133]),
            ("Riku", &[0x21C0]),
            ("Saskia", &[0x5A5C]),
            ("Isperia", &[0x15BE]),
            ("Ranar", &[0x5A4A]),
            ("Wilhelt", &[0x3117]),
            ("Hazel", &[0x4A2E]),
            ("Lord Windgrace", &[0x7D18]),
            ("Derevi", &[0xDE7E]),
            ("Atarka", &[0xA7A4]),
            // CR 701.15 — goad from resolutions, holds and attachments.
            ("Nelly Borca", &[0xBE11]),
            // A random-opponent attack requirement (Territorial Hellkite).
            ("Eshki", &[0xE5C1]),
            ("Zinnia", &[0x2177]),
            ("Ellivere", &[0xE111]),
            // CR 702.51 — convoke taps fire "becomes tapped" triggers.
            ("Kasla", &[0xCA51]),
            // CR 903.9 — Myth Unbound's command-zone draws, Empyrial Storm's copies.
            ("Estrid", &[0xE571]),
            // CR 122.1b — keyword counters from the graveyard (Kathril).
            ("Kathril", &[0xCA7B]),
            // Library-top Equipment riders, CR 706.2 natural-max draws.
            ("Galea", &[0x6A1E]),
            // CR 700.9 — modified creatures; CR 603.10 computed LKI power.
            ("Chishiro", &[0xC415]),
            // CR 903.3a — a planeswalker commander; CR 105.2c all-colors tokens.
            ("Jared", &[0x7A2E]),
            // CR 614.1c — enters-with counters read from a graveyard (Dearly
            // Departed); CR 702.16 protection from a chosen creature type.
            ("Jirina", &[0x1714]),
            // CR 702.181a Mobilize, CR 614.1a token replacement, CR 903.3 commander attacks.
            ("Zurgo Stormrender", &[0x2B60]),
            // CR 613.1d layered outlaws; CR 122.1 hit counters in exile.
            ("Olivia", &[0x0117]),
            // CR 707.10 — copies, Twinning Staff's +1, granted conspire.
            ("Kalamax", &[0xCA1A]),
            // CR 702.94 granted miracle, CR 122.1d stun lock, CR 709 Rooms.
            ("Aminatou, Veil", &[0xA417]),
            // CR 702.140 mutate, CR 306.5b X loyalty, CR 207.2c spell mastery.
            ("Otrimi", &[0x0791]),
            // Coven (three different powers), CR 303.4a player Curses, CR 107.3 X.
            ("Leinore", &[0x1E17]),
            // CR 113.6b eminence from the command zone; CR 602.5b tap-X costs.
            ("Sidar", &[0x51DA]),
            // Everything counters (every land type), CR 701.5 Summary Dismissal.
            ("Omo", &[0x0A0A]),
            // CR 702.152 granted blitz, CR 508.1a Weathered Sentinels, CR 701.15 goad.
            ("Henzie", &[0x4E21]),
            // CR 113.6b command-zone upkeep life; CR 701.10g power exchange;
            // Order of Succession's pass around the table.
            ("Oloro", &[0x0107]),
            // CR 702.29 cycling for {0} (Gavi, New Perspectives); CR 614.1a Sarcophagus.
            ("Gavi", &[0x6A71]),
            // CR 701.34 manifest from another library, CR 106.6 spend
            // restrictions, CR 702.99 cipher.
            ("Gonti, Canny", &[0x6071]),
            // CR 614.13/614.16 for every player (Primal Vigor); owners regain
            // control (Brooding Saurian); a cast trigger counting mana spent.
            ("Prossh", &[0x9055]),
            // CR 700.8 parties (Stick Together), CR 611.2b next-turn copies.
            ("Morophon", &[0x0A0B]),
            // CR 700.8 full party, CR 702.84 granted unearth, CR 603.3d
            // once-each-turn grants (Folk Hero).
            ("Nalia", &[0x4A11]),
            // CR 107.16 energy; CR 118.9 an energy alternative cost.
            ("Saheeli, Radiant", &[0x5A4E]),
            // CR 614 Warlocks exiled instead of dying (Lorcan); CR 603.10 a dead
            // blocker (Death Tyrant); CR 508.1 "whenever you attack a player".
            ("Prosper", &[0x9A05]),
            // CR 702.37 morph and megamorph, CR 614 Rayami's death exile,
            // CR 903.9 commanders sent home.
            ("Kadena", &[0x6ADE]),
            // CR 122.1b kinds of counters, CR 122.1c shields, CR 707.10c
            // copies that spread their removal.
            ("Perrie", &[0x9E44]),
            // CR 702.74 granted evoke (Ashling), CR 702.143 foretold
            // Haunting Voyage, CR 702.141 encore.
            ("Ashling", &[0xA541]),
            // CR 702.35 madness (paid, {X}, targeted), a reversed turn order.
            ("Anje", &[0xA41E]),
            // CR 702.34 flashback copied by Sevinne, CR 724.2 ending the
            // combat phase.
            ("Sevinne", &[0x5E71]),
            // CR 603.2d Wayta's doubled enrage, CR 615 Temple Altisaur's
            // prevention, CR 610.3 Bronzebeak Foragers' exile.
            ("Pantlaza", &[0x9A47]),
            // CR 702.143 foretell, CR 107.3 an {X}{X} payment (Numa).
            ("Lathril", &[0x1A7A]),
            // CR 702.124j partner with, CR 701.54 the Ring tempts you.
            ("Frodo", &[0xF20D]),
            // CR 702.6 equip (a legendary equip cost), CR 702.143 foretold
            // "cast from exile" riders, one destroy per opponent.
            ("Cloud, Ex", &[0xC10D]),
            // CR 702.152 blitz from the graveyard, CR 700.2 a random mode,
            // CR 101.4 opponents choosing in turn order.
            ("Terra, Herald", &[0x7E44]),
            // CR 306.5b Empower Jace tokens, CR 508.1 Jace's attack tax,
            // CR 702.26 Teferi's Reproach.
            ("Jace, Multiverse", &[0x7ACE]),
            // CR 725 the monarch, CR 508.1c a gated attack restriction,
            // restricted Vibranium mana.
            ("T'Challa", &[0x7C4A]),
            // CR 702.157 squad's non-mana cost, CR 701.38 a most-votes
            // council, CR 702.138 granted escape.
            ("Caesar", &[0xCAE5]),
            // CR 701.50 connive triggers, CR 702.62 granted suspend.
            ("Doctor Doom", &[0xD00D]),
            // CR 702.85 granted cascade, CR 615 Khârn's control swap, CR
            // 508.1d a forced attack.
            ("Abaddon", &[0xABAD]),
            // CR 701.55 villainous choices (doubled by The Valeyard), CR 708.2
            // face-down Cybermen.
            ("Davros", &[0xDA05]),
            // CR 701.54 time travel, CR 702.62 suspend, a Doctor's companion.
            ("The Tenth Doctor", &[0x7E17]),
            // CR 603.2c "each lose exactly 1 life" batches, Manabarbs' per-land
            // triggers, Mount Doom's choose-two wipe.
            ("Ob Nixilis, Captive", &[0x0B41]),
            // CR 702.40 a granted storm, Ashling's escalating Magecraft.
            ("Storm, Force", &[0x5704]),
            // CR 702.130 enrage loops (Polyraptor, Forerunner of the Empire).
            ("Indoraptor", &[0x1D0A]),
            // CR 702.61 split second from artifact mana, Knuckles' Treasures.
            ("Shadow the", &[0x5ADE]),
            // CR 614.1a the noncombat floor, CR 712 Ojer and Urabrask flipping.
            ("Ojer Axonil", &[0x03E5]),
            // A turn discount sized by life lost (Rowan), Vilis' draws.
            ("Rowan, Scion", &[0x2043]),
            // CR 612 exchanged text boxes, CR 702.116 myriad.
            ("Deadpool", &[0xDEAD]),
            // CR 702.49 ninjutsu granted to every creature card in hand.
            ("Satoru", &[0x5A70]),
            // Volo's copied creature spells, Volo's Journal's noted types.
            ("Volo, Guide", &[0x7010]),
            // CR 701.55 villainous choices each end step, Blitzwing converting.
            ("Dr. Eggman", &[0xE66]),
            // Death triggers twice (Teysa), CR 701.21 untap-proliferate (Kilo),
            // draw pings (Niv-Mizzet), enchantment flicker (Yuna), library-top
            // casting (Elsha), extra mana (Kinnan), slime counters (Toxrill).
            ("Teysa Karlov", &[0x7E5A]),
            ("Kilo, Apogee", &[0x4110]),
            ("Niv-Mizzet, Parun", &[0x41F0]),
            ("Yuna, Grand", &[0x4A1A]),
            ("Elsha", &[0xE15A]),
            ("Kinnan", &[0x4144]),
            ("Toxrill", &[0x7055]),
            // Towns (Minstrel), CR 702.85 granted cascade (First Sliver),
            // enchantment tutors (Zur), free casts off every library (Etali),
            // X copies (Lucea), CR 701.38 council's dilemma votes (Tivit),
            // exile-until-land (Umbris), granted storm (Prismari), counter
            // pings (Shalai and Hallar), granted mill abilities (Phenax).
            ("The Wandering Minstrel", &[0x3135]),
            ("The First Sliver", &[0x5115]),
            ("Zur the Enchanter", &[0x2042]),
            ("Etali, Primal Conqueror", &[0xE7A1]),
            ("Magus Lucea", &[0x1CEA]),
            ("Tivit", &[0x7171]),
            ("Umbris", &[0x0B15]),
            ("Prismari", &[0x9215]),
            ("Shalai and Hallar", &[0x5A1A]),
            ("Phenax", &[0x9E4A]),
            // Token taps (Baylen), CR 614.1a token replacement (Chatterfang),
            // Dragon digs (Ureni), draw triggers (Xyris, Sheoldred), CR 603.2d
            // doubled enters triggers (Yarok), goaded Birds (Rendmaw),
            // graveyard returns (Celes), CR 702.153 granted casualty
            // (Silverquill), cast triggers and bounce (Chulane), Rabbit
            // pumps (Finneas), discard-to-cast (Cruelclaw).
            ("Baylen, the Haymaker", &[0xBA71]),
            ("Chatterfang, Squirrel General", &[0xC4A7]),
            ("Ureni of the Unwritten", &[0x02E1]),
            ("Xyris, the Writhing Storm", &[0x0715]),
            ("Yarok, the Desecrated", &[0x7A40]),
            ("Rendmaw, Creaking Nest", &[0x2E4D]),
            ("Celes, Rune Knight", &[0xCE1E]),
            ("Sheoldred, the Apocalypse", &[0x5E0A]),
            ("Silverquill, the Disputant", &[0x5117]),
            ("Chulane, Teller of Tales", &[0xC401]),
            ("Finneas, Ace Archer", &[0xF1EA]),
            ("The Infamous Cruelclaw", &[0xC2E1]),
            // Spell-cast payoffs (Vivi, Kykar, Alania, Veyran, Bria), draw
            // pings (Nekusar), death drains and a transform (Sephiroth), X
            // Hydras (Zaxara), landfall tiers (Omnath), token draws (Marneus),
            // Zombie loots (Varina), graveyard artifact casts (Noctis).
            ("Vivi Ornitier", &[0x0171]),
            ("Nekusar, the Mindrazer", &[0x4E45]),
            ("Sephiroth, Fabled SOLDIER", &[0x5E91]),
            ("Zaxara, the Exemplary", &[0x2A4A]),
            ("Omnath, Locus of Creation", &[0x0C2E]),
            ("Marneus Calgar", &[0x4A2E]),
            ("Kykar, Wind's Fury", &[0x4F4A]),
            ("Varina, Lich Queen", &[0x7A21]),
            ("Alania, Divergent Storm", &[0xA1A1]),
            ("Veyran, Voice of Duality", &[0x7E72]),
            ("Bria, Riptide Rogue", &[0xB21A]),
            ("Noctis, Prince of Lucis", &[0x40C7]),
            // CR 106.4 lost mana turned black; CR 310.12b Invasion of Alara.
            ("Omnath, Locus of All", &[0x0A11]),
            // Graveyard replays (Muldrotha), play/cast loots (Flubs), cast
            // drains and a life-paid CR 903.8 tax (Liesa), counters spent as mana
            // (Ramos), kept green mana (Omnath), landfall pings (Tannuk),
            // Hydra cost cuts and fights (Gargos).
            ("Muldrotha, the Gravetide", &[0x40D2]),
            ("Flubs, the Fool", &[0xF1B5]),
            ("Liesa, Shroud of Dusk", &[0x1E5A]),
            ("Ramos, Dragon Engine", &[0x2A05]),
            ("Omnath, Locus of Mana", &[0x0A7A]),
            ("Tannuk, Memorial Ensign", &[0x7A44]),
            ("Gargos, Vicious Watcher", &[0x6A26]),
            // Creature-spell discounts (Animar), 1-damage pings (Ghyrson),
            // Sliver tutors and thefts (Sliver Overlord), land-to-graveyard
            // draws (Gitrog), lifegain drains (Dina).
            ("Animar, Soul of Elements", &[0xA41A]),
            ("Ghyrson Starn, Kelermorph", &[0x6475]),
            ("Sliver Overlord", &[0x5110]),
            ("The Gitrog Monster", &[0x6170]),
            ("Dina, Soul Steeper", &[0xD1A5]),
            ("Betor, Ancestor's Voice", &[0xBE70]),
            ("Witherbloom, the Balancer", &[0x3B10]),
            ("Judith, Carnage Connoisseur", &[0x1D17]),
            ("Korvold, Fae-Cursed King", &[0xC0B0]),
            ("Hope Estheim", &[0x40BE]),
            ("Alela, Artful Provocateur", &[0xA1E1]),
            ("The Scarab God", &[0x5CAB]),
            ("Grand Arbiter Augustin IV", &[0x6A4B]),
            ("Thalia and The Gitrog Monster", &[0x7A16]),
            ("Queza, Augur of Agonies", &[0x0E2A]),
            ("Omnath, Locus of Rage", &[0x0AA6]),
            ("The Necrobloom", &[0xEC0B]),
            ("Atraxa, Grand Unifier", &[0xA7AA]),
            ("Codie, Vociferous Codex", &[0xC0D1]),
            ("Blech, Loafing Pest", &[0xB1EC]),
            ("Fynn, the Fangbearer", &[0xF1BB]),
            ("Feather, the Redeemed", &[0xFEA7]),
            ("Narset, Enlightened Exile", &[0x4A25]),
            ("Raffine, Scheming Seer", &[0x4AFF]),
            ("The Mindskinner", &[0x3155]),
            ("Atla Palani, Nest Tender", &[0xA71A]),
            ("Betor, Kin to All", &[0xBE71]),
            ("Lumra, Bellow of the Woods", &[0x1B4A]),
            // 9196, not 9195: once the bot stopped buying evasion under Sphere
            // of Safety (CR 508.1g), 9195 reached Ghostly Dancers + Secret
            // Arcade, a true unbounded mandatory loop that ends as a board cap.
            ("Marina Vendrell", &[0x3A41]),
            ("Aloy, Savior of Meridian", &[0xA109]),
            ("Marchesa, the Black Rose", &[0x3A4C]),
            ("Brago, King Eternal", &[0xB4A6]),
            ("Jetmir, Nexus of Revels", &[0x7E71]),
            ("Kefka, Court Mage", &[0xEFCA]),
            ("Glarb, Calamity's Augur", &[0x61A8]),
            ("Hashaton, Scarab's Fist", &[0x4A54]),
            ("Kraum, Ludevic's Opus", &[0x7A4B]),
            ("Thrasios, Triton Hero", &[0x7A51]),
            ("Infinite Guideline Station", &[0x16E5]),
            ("Gishath, Sun's Avatar", &[0x615A]),
            ("Shorikai, Genesis Engine", &[0x5401]),
            ("Golbez, Crystal Collector", &[0x601B]),
            ("Urza, Lord High Artificer", &[0x0F2A]),
            ("Mendicant Core, Guidelight", &[0x3E4D]),
            ("Wick, the Whorled Mind", &[0x31C4]),
            ("Vren, the Relentless", &[0x7E0E]),
            ("Krenko, Mob Boss", &[0x4E7B]),
            ("Arabella, Abandoned Doll", &[0xA4AB]),
            ("Coram, the Undertaker", &[0xC0A4]),
            ("Sythis, Harvest's Hand", &[0x5717]),
            ("Jin Sakai, Ghost of Tsushima", &[0x715A]),
            ("Sonic the Hedgehog", &[0x5021]),
            ("Arcades, the Strategist", &[0xA4CA]),
            ("Queen Marchesa", &[0x4A2C]),
            ("Kenrith, the Returned King", &[0x4E7A]),
            ("Fire Lord Azula", &[0xA2A1]),
            ("Minsc & Boo, Timeless Heroes", &[0xB00B]),
            ("The Jolly Balloon Man", &[0xBA11]),
            ("Eriette of the Charmed Apple", &[0xE21E]),
            ("Alela, Cunning Conqueror", &[0xA1E1]),
            ("Ovika, Enigma Goliath", &[0x0F1C]),
            ("Tannuk, Steadfast Second", &[0x7A23]),
            ("Gev, Scaled Scorch", &[0x6E55]),
            ("Kudo, King Among Bears", &[0xBEA2]),
            ("Rocco, Cabaretti Caterer", &[0x20CC]),
            ("Sokka, Tenacious Tactician", &[0x50CA]),
            ("Tifa Lockhart", &[0x71FA]),
            ("Winota, Joiner of Forces", &[0x3107]),
            ("Ziatora, the Incinerator", &[0x21A7]),
            ("Helga, Skittish Seer", &[0x4E16]),
            ("Vihaan, Goldwaker", &[0x71AA]),
            ("Tayam, Luminous Enigma", &[0x7A4A]),
            ("Brenard, Ginger Sculptor", &[0xB2E9]),
            ("Hazezon, Shaper of Sand", &[0x4A2E]),
            ("Eluge, the Shoreless Sea", &[0xE106]),
            ("Roxanne, Starfall Savant", &[0x20A2]),
            ("Lightning, Army of One", &[0x1197]),
            ("Isshin, Two Heavens as One", &[0x1551]),
            ("Jodah, the Unifier", &[0x10DA]),
            ("Ygra, Eater of All", &[0x1964]),
            ("Orvar, the All-Form", &[0x0A7F]),
            ("Sisay, Weatherlight Captain", &[0x515A]),
            ("Kuja, Genome Sorcerer", &[0x0C1A]),
            ("Urtet, Remnant of Memnarch", &[0x0B7E]),
            ("Kona, Rescue Beastie", &[0x0C0A]),
            ("Marrow-Gnawer", &[0x0A12]),
            ("Alexios, Deimos of Kosmos", &[0x0A1E]),
            ("Raggadragga, Goreguts Boss", &[0x0A6A]),
            ("Athreos, Shroud-Veiled", &[0x0A71]),
            ("Rakdos, the Muscle", &[0x0A72]),
            ("Captain N'ghathrod, EDHREC", &[0x0A73]),
            ("Kotis, the Fangkeeper", &[0x0A74]),
            ("Henzie \"Toolbox\" Torre, EDHREC", &[0x0A75, 335, 9260]),
            ("Frodo + Sam, EDHREC", &[0x0A76]),
            ("K'rrik, Son of Yawgmoth, EDHREC", &[0x0A77]),
            ("Voja, Jaws of the Conclave", &[0x0A78]),
            ("Choco, Seeker of Paradise", &[0x0A79]),
            ("Light-Paws, Emperor's Voice", &[0x0A7A]),
            ("Yurlok of Scorch Thrash", &[0x0A7B]),
            ("Rocco, Street Chef", &[0x0A7C]),
            ("Tinybones, Bauble Burglar", &[0x0A7D]),
            ("Indominus Rex, Alpha", &[0x0A7E]),
            ("Tergrid, God of Fright", &[0x0A7F]),
            ("Athreos, God of Passage", &[0x0A80]),
            ("Ketramose, the New Dawn", &[0x0A81]),
            ("Niko, Light of Hope", &[0x0A82]),
            ("Syr Gwyn, Hero of Ashvale", &[0x0A83]),
            ("Kastral, the Windcrested", &[0x0A84]),
            ("Jodah, Archmage Eternal", &[0x0A85]),
            ("Child of Alara", &[0x0A86]),
            ("Rakdos, Lord of Riots", &[0x0A87]),
            ("Myrel, Shield of Argive", &[0x0A88]),
            ("Be'lakor, the Dark Master", &[0x0A89]),
            ("Anti-Venom, Horrifying Healer", &[0x0A8A]),
            ("Fire Lord Zuko", &[0x0A8B]),
            ("Arna Kennerüd, Skycaptain", &[0x0A8C]),
            ("Terra, Magical Adept", &[0x0A8D]),
            ("Atreus, Impulsive Son", &[0x0A8E]),
            ("Kratos, God of War", &[0x0A8F]),
            ("Eddie Brock", &[0x0A90]),
            ("Nicol Bolas, the Ravager", &[0x0A91]),
            ("Shilgengar, Sire of Famine", &[0x0A92]),
            ("Norin the Wary", &[0x0A93]),
            ("Katara, the Fearless", &[0x0A94]),
            ("Azlask, the Swelling Scourge", &[0x0A95]),
            ("The Mycotyrant", &[0x0A96]),
            ("Shroofus Sproutsire", &[0x0A97]),
            ("Miirym, Sentinel Wyrm", &[0x0A98]),
            ("Tiamat", &[0x0A99]),
            ("Thranduil, the Elvenking", &[0x0A9A]),
            ("Maralen, Fae Ascendant", &[0x0A9B]),
            ("Najeela, the Blade-Blossom", &[0x0A9C]),
            ("Toph, the First Metalbender", &[0x0A9D]),
            ("Wolverine, Best There Is", &[0x0A9E]),
            ("Captain Howler, Sea Scourge", &[0x0A9F]),
            ("Magda, Brazen Outlaw", &[0x0AA0]),
            ("Avatar Aang", &[0x0AA1]),
            ("Lorehold, the Historian", &[0x0AA2]),
            ("High Perfect Morcant", &[0x0AA3]),
            ("Iroh, Grand Lotus", &[0x0AA4]),
            ("Galadriel, Light of Valinor", &[0x0AA5]),
            ("Skullbriar, the Walking Grave", &[0x0AA6]),
            ("Aragorn, the Uniter", &[0x0AA7]),
            // 9311 is a CR 104.4a draw: Descent into Avernus drops both
            // survivors to 0 in one upkeep. The seat test wants a winner.
            ("Ragost, Deft Gastronaut", &[0x0AA8]),
            ("Iron Man, Titan of Innovation", &[0x0AA9]),
            ("Jon Irenicus, Shattered One", &[0x0AAA]),
            ("Obeka, Splitter of Seconds", &[0x0AAB]),
            ("Captain America, First Avenger", &[0x0AAC]),
            ("Doran, Besieged by Time", &[0x0AAD]),
            ("Lord of the Nazgûl", &[0x0AAE]),
            ("Leonardo, the Balance", &[0x0AAF]),
            ("Norman Osborn", &[0x0AB0]),
            ("Reaper King", &[0x0AB1]),
            ("Sauron, the Dark Lord", &[0x0AB2]),
            ("Saruman, the White Hand", &[0x0AB3]),
            ("Kibo, Uktabi Prince", &[0x0AB4]),
            ("Shelob, Child of Ungoliant", &[0x0AB5]),
            ("Marvo, Deep Operative", &[0x0AB6]),
            ("Tom Bombadil", &[0x0AB7]),
            ("Edward Kenway", &[0x0AB8]),
            ("Hylda of the Icy Crown", &[0x0AB9]),
            ("Mr. House, President and CEO", &[0x0ABA]),
            ("Ezio Auditore da Firenze", &[0x0ABB]),
            ("Tovolar, Dire Overlord", &[0x0ABC]),
            ("Xenagos, God of Revels", &[0x0ABD]),
            ("Kess, Dissident Mage", &[0x0ABE]),
            ("Vnwxt, Verbose Host", &[0x0ABF]),
            ("Szarel, Genesis Shepherd", &[0x0AC0]),
            ("Winter, Misanthropic Guide", &[0x0AC1]),
            ("Niv-Mizzet, Visionary", &[0x0AC2]),
            ("Bristly Bill, Spine Sower", &[0x0AC3]),
            ("Talion, the Kindly Lord", &[0x0AC4]),
            ("Tifa, Martial Artist", &[0x0AC5]),
            ("Lathiel, the Bounteous Dawn", &[0x0AC6]),
            ("Koma, Cosmos Serpent", &[0x0AC7]),
            ("Bilbo, Birthday Celebrant", &[0x0AC8]),
            ("Bruvac the Grandiloquent", &[0x0AC9]),
            ("Ayara, First of Locthwain", &[0x0ACA]),
            ("Zurgo, Thunder's Decree", &[0x0ACB]),
            ("Laughing Jasper Flint", &[0x0ACC]),
            ("Jenova, Ancient Calamity", &[0x0ACD]),
            ("The Lord of Pain", &[0x0ACE]),
            ("Marisi, Breaker of the Coil", &[0x0ACF]),
            ("Elenda, the Dusk Rose", &[0x0AD0]),
            ("Emry, Lurker of the Loch", &[0x0AD1]),
            ("Felothar, Dawn of the Abzan", &[0x0AD2]),
            ("Sin, Spira's Punishment", &[0x0AD3]),
            ("Mr. Foxglove", &[0x0AD4]),
            ("Tasigur, the Golden Fang", &[0x0AD5]),
            ("Loot, the Pathfinder", &[0x0AD6]),
            ("Jinnie Fay, Jetmir's Second", &[0x0AD7]),
            ("Loot, Exuberant Explorer", &[0x0AD8]),
            ("Scion of the Ur-Dragon", &[0x0AD9]),
            ("Rhys the Redeemed", &[0x0ADA]),
            ("Jarad, Golgari Lich Lord", &[0x0ADB]),
            ("Alesha, Who Smiles at Death", &[0x0ADC]),
            ("Quandrix, the Proof", &[0x0ADD]),
            ("Yuna, Hope of Spira", &[0x0ADE]),
            ("Goreclaw, Terror of Qal Sisma", &[0x0ADF]),
            ("Garth One-Eye", &[0x0AE0]),
            ("Rat King, Verminister", &[0x0AE1]),
            ("Liberty Prime, Recharged", &[0x0AE2]),
            ("Samut, the Driving Force", &[0x0AE3]),
            ("Black Waltz No. 3", &[0x0AE4]),
            ("Otharri, Suns' Glory", &[0x0AE5]),
            ("Vito, Thorn of the Dusk Rose", &[0x0AE6]),
            ("Cloud, Midgar Mercenary", &[0x0AE7]),
            ("Elas il-Kor, Sadistic Pilgrim", &[0x0AE8]),
            ("Braids, Arisen Nightmare", &[0x0AE9]),
            ("Haldan, Avid Arcanist", &[0x0AEA]),
            ("The Master of Keys", &[0x0AEB]),
            ("The Unbeatable Squirrel Girl", &[0x0AEC]),
            ("Squall, SeeD Mercenary", &[0x0AED]),
            ("Kellan, the Kid", &[0x0AEE]),
            ("Tuvasa the Sunlit", &[0x0AEF]),
            ("Sidisi, Brood Tyrant", &[0x0AF0]),
            ("Extus, Oriq Overlord", &[0x0AF1]),
            ("Nethroi, Apex of Death", &[0x0AF2]),
            ("Sigarda, Font of Blessings", &[0x0AF3]),
            ("Azusa, Lost but Seeking", &[0x0AF4]),
            ("Yawgmoth, Thran Physician", &[0x0AF5]),
            ("Astarion, the Decadent", &[0x0AF6]),
            ("The Locust God", &[0x0AF7]),
            ("Jaws, Relentless Predator", &[0x0AF8]),
            ("Don Andres, the Renegade", &[0x0AF9]),
            ("Karador, Ghost Chieftain", &[0x0AFA]),
            ("Delney, Streetwise Lookout", &[0x0AFB]),
            ("Sorin of House Markov", &[0x0AFC]),
            ("Teval, Arbiter of Virtue", &[0x0AFD]),
            ("Gisa, the Hellraiser", &[0x0AFE]),
            ("Felix Five-Boots", &[0x0AFF]),
            ("Black Panther, Wakandan King", &[0x0B00]),
            ("Camellia, the Seedmiser", &[0x0B01]),
            ("Karlov of the Ghost Council", &[0x0B02]),
            ("Hamza, Guardian of Arashin", &[0x0B03]),
            ("Anim Pakal, Thousandth Moon", &[0x0B04]),
            ("Sami, Wildcat Captain", &[0x0B05]),
            ("Jan Jansen, Chaos Crafter", &[0x0B06]),
            ("Tamiyo, Inquisitive Student", &[0x0B07]),
            ("Karona, False God", &[0x0B08]),
            ("Solphim, Mayhem Dominus", &[0x0B09]),
            ("Neera, Wild Mage", &[0x0B0A]),
            ("Vincent Valentine", &[0x0B0B]),
            ("Heliod, Sun-Crowned", &[0x0B0C]),
            ("Syr Vondam, Sunstar Exemplar", &[0x0B0D]),
            ("Ghired, Mirror of the Wilds", &[0x0B0E]),
            ("Vishgraz, the Doomhive", &[0x0B0F]),
            ("Herigast, Erupting Nullkite", &[0x0B10]),
            ("Tinybones, Trinket Thief", &[0x0B11]),
            ("Cayth, Famed Mechanist", &[0x0B12]),
            ("G'raha Tia, Scion Reborn", &[0x0B13]),
            ("Slimefoot and Squee", &[0x0B14]),
            ("Elesh Norn, Mother of Machines", &[0x0B15]),
            ("Evereth, Viceroy of Plunder", &[0x0B16]),
            ("Kiora, Sovereign of the Deep", &[0x0B17]),
            ("Juri, Master of the Revue", &[0x0B18]),
            ("Eshki Dragonclaw", &[0x0B19]),
            ("Aang, at the Crossroads", &[0x0B1A]),
            ("Aragorn, King of Gondor", &[0x0B1B]),
            ("Arixmethes, Slumbering Isle", &[0x0B1C]),
            ("Rivaz of the Claw", &[0x0B1D]),
            ("Aurelia, the Warleader", &[0x0B1E]),
            ("Toph, Hardheaded Teacher", &[0x0B1F]),
            ("Xantcha, Sleeper Agent", &[0x0B20]),
            ("Kyler, Sigardian Emissary", &[0x0B21]),
            ("Muerra, Trash Tactician", &[0x0B22]),
            ("Alesha, Who Laughs at Fate", &[0x0B23]),
            ("Plagon, Lord of the Beach", &[0x0B24]),
            ("Toph, Earthbending Master", &[0x0B25]),
            ("Amalia Benavides Aguirre", &[0x0B26]),
            ("Sen Triplets", &[0x0B27]),
            ("Tasha, the Witch Queen", &[0x0B28]),
            ("Clement, the Worrywort", &[0x0B29]),
            ("Marwyn, the Nurturer", &[0x0B2A]),
            ("Imotekh the Stormlord", &[0x0B2B]),
            ("Clive, Ifrit's Dominant", &[0x0B2C]),
            ("Mabel, Heir to Cragflame", &[0x0B2D]),
            ("Jhoira, Weatherlight Captain", &[0x0B2E]),
            ("Talrand, Sky Summoner", &[0x0B2F]),
            ("Karumonix, the Rat King", &[0x0B30]),
            ("Kambal, Profiteering Mayor", &[0x0B31]),
            ("Smaug the Impenetrable", &[0x0B32]),
            ("Zur, Eternal Schemer", &[0x0B33]),
            ("The Reaper, King No More", &[0x0B34]),
            ("Progenitus", &[0x0B35]),
            ("Massacre Girl, Known Killer", &[0x0B36]),
            ("Bre of Clan Stoutarm", &[0x0B37]),
            ("Eirdu, Carrier of Dawn", &[0x0B38]),
            ("Tatsunari, Toad Rider", &[0x0B39]),
            ("Gonti, Night Minister", &[0x0B3A]),
            ("Minthara, Merciless Soul", &[0x0B3B]),
            ("Asmoranomardicadaistinaculdacar", &[0x0B3C]),
            ("Slicer, Hired Muscle", &[0x0B3D]),
            ("Kozilek, the Great Distortion", &[0x0B3E]),
            ("Savra, Queen of the Golgari", &[0x0B3F]),
            ("Lonis, Cryptozoologist", &[0x0B40]),
            ("Baral, Chief of Compliance", &[0x0B41]),
            ("Marchesa, Dealer of Death", &[0x0B42]),
            ("Grolnok, the Omnivore", &[0x0B43]),
            ("Arthur, Marigold Knight", &[0x0B44]),
            ("Uril, the Miststalker", &[0x0B45]),
            ("Zurgo Helmsmasher", &[0x0B46]),
            ("Taii Wakeen, Perfect Shot", &[0x0B47]),
            ("Silvar, Devourer of the Free", &[0x0B48]),
            ("Lady Octopus, Inspired Inventor", &[0x0B49]),
            ("Elsha of the Infinite", &[0x0B4A]),
            ("Cazur, Ruthless Stalker and Ukkima, Stalking Shadow", &[0x0B4B]),
            ("Purphoros, God of the Forge", &[0x0B4C]),
            ("Mogis, God of Slaughter", &[0x0B4D]),
            ("Yargle and Multani", &[0x0B4E]),
            ("Will, Scion of Peace", &[0x0B4F]),
            ("Kruphix, God of Horizons", &[0x0B50]),
            ("Beledros Witherbloom", &[0x0B51]),
            ("Muddle, the Ever-Changing", &[0x0B52]),
            ("Dionus, Elvish Archdruid", &[0x0B53]),
            ("Rukarumel, Biologist", &[0x0B54]),
            ("Sliver Hivelord", &[0x0B55]),
            ("Emet-Selch, Unsundered", &[0x0B56]),
            ("Radagast of Rhosgobel", &[0x0B57]),
            ("Carmen, Cruel Skymarcher", &[0x0B58]),
            ("Zenos yae Galvus", &[0x0B59]),
            ("Archelos, Lagoon Mystic", &[0x0B5A]),
            ("Halana and Alena, Partners", &[0x0B5B]),
            ("Homer, the Hermit", &[0x0B5C]),
            ("Sovereign Okinec Ahau", &[0x0B5D]),
            ("Torbran, Thane of Red Fell", &[0x0B5E]),
            ("Bumi, Unleashed", &[0x0B5F]),
            ("Kresh the Bloodbraided", &[0x0B60]),
            ("Merry, Warden of Isengard and Pippin, Warden of Isengard", &[0x0B61]),
            ("Phelia, Exuberant Shepherd", &[0x0B62]),
            ("Urza, Lord Protector", &[0x0B63]),
            ("Kutzil, Malamet Exemplar", &[0x0B64]),
            ("Rashmi and Ragavan", &[0x0B65]),
            ("Zimone, Paradox Sculptor", &[0x0B66]),
            ("Oviya, Automech Artisan", &[0x0B67]),
            ("Sab-Sunen, Luxa Embodied", &[0x0B68]),
            ("Bonny Pall, Clearcutter", &[0x0B69]),
            ("Ardyn, the Usurper", &[0x0B6A]),
            ("Kykar, Zephyr Awakener", &[0x0B6B]),
            ("The Master, Transcendent", &[0x0B6C]),
            ("Neheb, the Eternal", &[0x0B6D]),
            ("Niv-Mizzet, Guildpact", &[0x0B6E]),
            ("Cadira, Caller of the Small", &[0x0B6F]),
            ("Primo, the Unbounded", &[0x0B70]),
            ("Brallin, Skyshark Rider and Shabraz, the Skyshark", &[0x0B71]),
            ("Yenna, Redtooth Regent", &[0x0B72]),
            ("Koma, World-Eater", &[0x0B73]),
            ("Shiko, Paragon of the Way", &[0x0B74]),
            ("Octavia, Living Thesis", &[0x0B75]),
            ("General Tazri", &[0x0B76]),
            ("Valgavoth, Terror Eater", &[0x0B77]),
            ("Mog, Moogle Warrior", &[0x0B78]),
            ("Sephiroth, Fallen Hero", &[0x0B79]),
            ("Adeline, Resplendent Cathar", &[0x0B7A]),
            ("Mazirek, Kraul Death Priest", &[0x0B7B]),
            ("Roon of the Hidden Realm", &[0x0B7C]),
            ("Errant and Giada", &[0x0B7D]),
            ("Ashcoat of the Shadow Swarm", &[0x0B7E]),
            ("Lord Xander, the Collector", &[0x0B7F]),
            ("Wildsear, Scouring Maw", &[0x0B80]),
            ("Sephiroth, Planet's Heir", &[0x0B81]),
            ("Damia, Sage of Stone", &[0x0B82]),
            ("Lavinia, Azorius Renegade", &[0x0B83]),
            ("Chainer, Nightmare Adept", &[0x0B84]),
            ("Elenda, Saint of Dusk", &[0x0B85]),
            ("Katara, Waterbending Master", &[0x0B86]),
            ("Rionya, Fire Dancer", &[0x0B87]),
            ("Joshua, Phoenix's Dominant", &[0x0B88]),
            ("Círdan the Shipwright", &[0x0B89]),
            ("Zul Ashur, Lich Lord", &[0x0B8A]),
            ("Licia, Sanguine Tribune", &[0x0B8B]),
            ("Baeloth Barrityl, Entertainer and Noble Heritage", &[0x0B8C]),
            ("Mimeoplasm, Revered One", &[0x0B8D]),
            ("Amy Rose", &[0x0B8E]),
            ("Calix, Guided by Fate", &[0x0B8F]),
            ("Ashnod the Uncaring", &[0x0B90]),
            ("Sharuum the Hegemon", &[0x0B91]),
            ("Cloud, Planet's Champion", &[0x0B92]),
            ("Lara Croft, Tomb Raider", &[0x0B93]),
            ("Preston, the Vanisher", &[0x0B94]),
            ("Jenson Carthalion, Druid Exile", &[0x0B95]),
            ("Lazav, Dimir Mastermind", &[0x0B96]),
            ("Elenda and Azor", &[0x0B97]),
            ("Me, the Immortal", &[0x0B98]),
            ("Sydri, Galvanic Genius", &[0x0B99]),
            ("Ragavan, Nimble Pilferer", &[0x0B9A]),
            ("Xolatoyac, the Smiling Flood", &[0x0B9B]),
            ("Aziza, Mage Tower Captain", &[0x0B9C]),
            ("Mondrak, Glory Dominus", &[0x0B9D]),
            ("Gisela, Blade of Goldnight", &[0x0B9E]),
            ("Treebeard, Gracious Host", &[0x0B9F]),
            ("Narset, Jeskai Waymaster", &[0x0BA0]),
            ("Aerith, Last Ancient", &[0x0BA1]),
            ("Lurrus of the Dream-Den", &[0x0BA2]),
            ("Rona, Herald of Invasion", &[0x0BA3]),
            ("Olivia Voldaren", &[0x0BA4]),
            ("Greymond, Avacyn's Stalwart", &[0x0BA5]),
            ("Molecule Man", &[0x0BA6]),
            ("General Kreat, the Boltbringer", &[0x0BA7]),
            ("Agent Frank Horrigan", &[0x0BA8]),
            ("Nissa, Leyline Tamer", &[0x0BA9]),
            ("Kosei, Penitent Warlord", &[0x0BAA]),
            ("Ureni, the Song Unending", &[0x0BAB]),
            ("Katilda, Dawnhart Prime", &[0x0BAC]),
            ("Titania, Protector of Argoth", &[0x0BAD]),
            ("Aurelia, the Law Above", &[0x0BAE]),
            ("Dyadrine, Synthesis Amalgam", &[0x0BAF]),
            ("Thalisse, Reverent Medium", &[0x0BB0]),
            ("Dragonhawk, Fate's Tempest", &[0x0BB1]),
            ("Liberator, Urza's Battlethopter", &[0x0BB2]),
            ("Neriv, Crackling Vanguard", &[0x0BB3]),
            ("Eladamri, Korvecdal", &[0x0BB4]),
            ("Yorion, Sky Nomad", &[0x0BB5]),
            ("Alandra, Sky Dreamer", &[0x0BB6]),
            ("Kroxa, Titan of Death's Hunger", &[0x0BB7]),
            ("Thraximundar", &[0x0BB8]),
            ("Kambal, Consul of Allocation", &[0x0BB9]),
            ("Victor, Valgavoth's Seneschal", &[0x0BBA]),
            ("Alibou, Ancient Witness", &[0x0BBB]),
            ("Felisa, Fang of Silverquill", &[0x0BBC]),
            ("Missy", &[0x0BBD]),
            ("Ephara, God of the Polis", &[0x0BBE]),
            ("Greensleeves, Maro-Sorcerer", &[0x0BBF]),
            ("Klauth, Unrivaled Ancient", &[0x0BC0]),
            ("Lae'zel, Vlaakith's Champion and Master Chef", &[0x0BC1]),
            ("Galazeth Prismari", &[0x0BC2]),
            ("Grimgrin, Corpse-Born", &[0x0BC3]),
            ("Rev, Tithe Extractor", &[0x0BC4]),
            ("Chiss-Goria, Forge Tyrant", &[0x0BC5]),
            ("Knuckles the Echidna", &[0x0BC6]),
            ("The Watcher in the Water", &[0x0BC7]),
            ("Urabrask", &[0x0BC8]),
            ("Graaz, Unstoppable Juggernaut", &[0x0BC9]),
            ("Volrath, the Shapestealer", &[0x0BCA]),
            ("Liesa, Forgotten Archangel", &[0x0BCB]),
            ("The Pride of Hull Clade", &[0x0BCC]),
            ("Dawnsire, Sunstar Dreadnought", &[0x0BCD]),
            ("Glissa, the Traitor", &[0x0BCE]),
            ("Ezuri, Renegade Leader", &[0x0BCF]),
            ("Grismold, the Dreadsower", &[0x0BD0]),
            ("God-Eternal Oketra", &[0x0BD1]),
            ("The Ghoul, Gunslinger", &[0x0BD2]),
            ("Goro-Goro and Satoru", &[0x0BD3]),
            ("Neriv, Heart of the Storm", &[0x0BD4]),
            ("Xavier Sal, Infested Captain", &[0x0BD5]),
            ("Myrkul, Lord of Bones", &[0x0BD6]),
            ("The Beamtown Bullies", &[0x0BD7]),
            ("Kwain, Itinerant Meddler", &[0x0BD8]),
            ("Zimone and Dina", &[0x0BD9]),
            ("Brudiclad, Telchor Engineer", &[0x0BDA]),
            ("Riku of Many Paths", &[0x0BDB]),
            ("Ratadrabik of Urborg", &[0x0BDC]),
            ("Narset, Enlightened Master", &[0x0BDD]),
            ("Syr Konrad, the Grim", &[0x0BDE]),
            ("Runo Stromkirk", &[0x0BDF]),
            ("Altaïr Ibn-La'Ahad", &[0x0BE0]),
            ("Kumena, Tyrant of Orazca", &[0x0BE1]),
            ("Kediss, Emberclaw Familiar", &[0x0BE2]),
            ("Nahiri, Forged in Fury", &[0x0BE3]),
            ("Selvala, Heart of the Wilds", &[0x0BE4]),
            ("Liliana, Heretical Healer", &[0x0BE5]),
            ("Vraska, Soul of Stone", &[0x0BE6]),
            ("Darien, King of Kjeldor", &[0x0BE7]),
            ("Zacama, Primal Calamity", &[0x0BE8]),
            ("Selvala, Explorer Returned", &[0x0BE9]),
            ("Abdel Adrian, Gorion's Ward and Candlekeep Sage", &[0x0BEA]),
            ("Crystal, Inhuman Princess", &[0x0BEB]),
            ("Zilortha, Strength Incarnate", &[0x0BEC]),
            ("Illuna, Apex of Wishes", &[0x0BED]),
            ("Mm'menon, the Right Hand", &[0x0BEE]),
            ("Niv-Mizzet, Ghost Counsel", &[0x0BEF]),
            ("Bjorna, Nightfall Alchemist and Wernog, Rider's Chaplain", &[0x0BF0]),
            ("Yarus, Roar of the Old Gods", &[0x0BF1]),
            ("Burakos, Party Leader and Folk Hero", &[0x0BF2]),
            ("Super Shredder", &[0x0BF3]),
            ("Gyome, Master Chef", &[0x0BF4]),
            ("Skithiryx, the Blight Dragon", &[0x0BF5]),
            ("Old Stickfingers", &[0x0BF6]),
            ("Balmor, Battlemage Captain", &[0x0BF7]),
            ("Iroas, God of Victory", &[0x0BF8]),
            ("Kellan, the Fae-Blooded", &[0x0BF9]),
            ("Ares, God of War", &[0x0BFA]),
            ("Gwenom, Remorseless", &[0x0BFB]),
            ("Imskir Iron-Eater", &[0x0BFC]),
            ("Jolene, the Plunder Queen", &[0x0BFD]),
            ("Balan, Wandering Knight", &[0x0BFE]),
            ("Red Death, Shipwrecker", &[0x0BFF]),
            ("Delina, Wild Mage", &[0x0C00]),
            ("Valduk, Keeper of the Flame", &[0x0C01]),
            ("Eris, Roar of the Storm", &[0x0C02]),
            ("Zabaz, the Glimmerwasp", &[0x0C03]),
            ("Killian, Ink Duelist", &[0x0C04]),
            ("Mathas, Fiend Seeker", &[0x0C05]),
            ("Pir, Imaginative Rascal and Toothy, Imaginary Friend", &[0x0C06]),
            ("Iroh, Tea Master", &[0x0C07]),
            ("Borborygmos Enraged", &[0x0C08]),
            ("Tyvar the Bellicose", &[0x0C09]),
            ("Avacyn, Angel of Hope", &[0x0C0A]),
            ("Ultima, Origin of Oblivion", &[0x0C0B]),
            ("Esix, Fractal Bloom", &[0x0C0C]),
            ("Teysa, Orzhov Scion", &[0x0C0D]),
            ("Soul of Windgrace", &[0x0C0E]),
            ("Urza, Prince of Kroog", &[0x0C0F]),
            ("The Goose Mother", &[0x0C10]),
            ("Toski, Bearer of Secrets", &[0x0C11]),
            ("Avacyn, Angel of Horror", &[0x0C12]),
            ("Tor Wauki the Younger", &[0x0C13]),
            ("Yisan, the Wanderer Bard", &[0x0C14]),
            ("Absolute Virtue", &[0x0C15]),
            ("Kirri, Talented Sprout", &[0x0C16]),
            ("Commissar Severina Raine", &[0x0C17]),
            ("Uro, Titan of Nature's Wrath", &[0x0C18]),
            ("Phylath, World Sculptor", &[0x0C19]),
            ("Pearl-Ear, Imperial Advisor", &[0x0C1A]),
            ("Feldon of the Third Path", &[0x0C1B]),
            ("Memnarch", &[0x0C1C]),
            ("Ishai, Ojutai Dragonspeaker and Reyhan, Last of the Abzan", &[0x0C1D]),
            ("Sliver Queen", &[0x0C1E]),
            ("Varragoth, Bloodsky Sire", &[0x0C1F]),
            ("Akul the Unrepentant", &[0x0C20]),
            ("Jenny Flint and Madame Vastra", &[0x0C21]),
            ("Jadzi, Oracle of Arcavios", &[0x0C22]),
            ("Scriv, the Obligator", &[0x0C23]),
            ("Sygg, River Cutthroat", &[0x0C24]),
            ("Klothys, God of Destiny", &[0x0C25]),
            ("Preston Garvey, Minuteman", &[0x0C26]),
            ("Omnath, Locus of the Void", &[0x0C27]),
            ("Zoyowa Lava-Tongue", &[0x0C28]),
            ("Khârn the Betrayer", &[0x0C29]),
            ("The Valeyard", &[0x0C2A]),
            ("Aclazotz, Deepest Betrayal", &[0x0C2B]),
            ("Ayli, Eternal Pilgrim", &[0x0C2C]),
            ("Celestine, the Living Saint", &[0x0C2D]),
            ("Momir Vig, Simic Visionary", &[0x0C2E]),
            ("Lotho, Corrupt Shirriff", &[0x0C2F]),
            ("Sram, Senior Edificer", &[0x0C30]),
            ("Ojer Taq, Deepest Foundation", &[0x0C31]),
            ("Saheeli, the Sun's Brilliance", &[0x0C32]),
            ("Vanille, Cheerful l'Cie", &[0x0C33]),
            ("Rith, Liberated Primeval", &[0x0C34]),
            ("Angus Mackenzie", &[0x0C35]),
            ("Braids, Cabal Minion", &[0x0C36]),
            ("Brokkos, Apex of Forever", &[0x0C37]),
            ("Venser, Corpse Puppet", &[0x0C38]),
            ("Bladewing, Deathless Tyrant", &[0x0C39]),
            ("Drana and Linvala", &[0x0C3A]),
            ("Totentanz, Swarm Piper", &[0x0C3B]),
            ("Ashad, the Lone Cyberman", &[0x0C3C]),
            ("Cromat", &[0x0C3D]),
            ("Kodama of the West Tree", &[0x0C3E]),
            ("Jegantha, the Wellspring", &[0x0C3F]),
            ("Malcolm, Alluring Scoundrel", &[0x0C40]),
            ("Vorel of the Hull Clade", &[0x0C41]),
            ("Firja, Judge of Valor", &[0x0C42]),
            ("The Reality Chip", &[0x0C43]),
            ("Shigeki, Jukai Visionary", &[0x0C44]),
            ("Ojer Kaslem, Deepest Growth", &[0x0C45]),
            ("Vincent, Vengeful Atoner", &[0x0C46]),
            ("Melek, Izzet Paragon", &[0x0C47]),
            ("Byrke, Long Ear of the Law", &[0x0C48]),
            ("Vohar, Vodalian Desecrator", &[0x0C49]),
            ("Gisa, Glorious Resurrector", &[0x0C4A]),
            ("Mina and Denn, Wildborn", &[0x0C4B]),
            ("Akiri, Line-Slinger and Silas Renn, Seeker Adept", &[0x0C4C]),
            ("Ulamog, the Defiler", &[0x0C4D]),
            ("Zellix, Sanity Flayer and Haunted One", &[0x0C4E]),
            ("Kaervek the Merciless", &[0x0C4F]),
            ("Vadrok, Apex of Thunder", &[0x0C50]),
            ("Aang, Swift Savior", &[0x0C51]),
            ("Hermes, Overseer of Elpis", &[0x0C52]),
            ("Aang and Katara", &[0x0C53]),
            ("Bruna, the Fading Light", &[0x0C54]),
            ("Erebos, God of the Dead", &[0x0C55]),
            ("Kefka, Dancing Mad", &[0x0C56]),
            ("Kodama of the East Tree and Toggo, Goblin Weaponsmith", &[0x0C57]),
            ("Nissa, Resurgent Animist", &[0x0C58]),
            ("Mm'menon, Uthros Exile", &[0x0C59]),
            ("Kodama of the East Tree and Sakashima of a Thousand Faces", &[0x0C5A]),
            ("Zimone, All-Questioning", &[0x0C5B]),
            ("Wasitora, Nekoru Queen", &[0x0C5C]),
            ("Belisarius Cawl", &[0x0C5D]),
            ("Yahenni, Undying Partisan", &[0x0C5E]),
            ("Ishai, Ojutai Dragonspeaker and Jeska, Thrice Reborn", &[0x0C5F]),
            ("Inga and Esika", &[0x0C60]),
            ("Sin, Unending Cataclysm", &[0x0C61]),
            ("Nath of the Gilt-Leaf", &[0x0C62]),
            ("Plargg and Nassari", &[0x0C63]),
            ("Zahur, Glory's Past", &[0x0C64]),
            ("Hugs, Grisly Guardian", &[0x0C65]),
            ("Alpharael, Stonechosen", &[0x0C66]),
            ("Cait, Cage Brawler", &[0x0C67]),
            ("Nikara, Lair Scavenger and Yannik, Scavenging Sentinel", &[0x0C68]),
            ("Bruse Tarl, Boorish Herder and Ikra Shidiqi, the Usurper", &[0x0C69]),
            ("Radagast the Brown", &[0x0C6A]),
            ("Rodolf Duskbringer", &[0x0C6B]),
            ("Locke, Treasure Hunter", &[0x0C6C]),
            ("Nezahal, Primal Tide", &[0x0C6D]),
            ("Sliver Legion", &[0x0C6E]),
            ("Sanar, Unfinished Genius", &[0x0C6F]),
            ("Exdeath, Void Warlock", &[0x0C70]),
            ("The Raven Man", &[0x0C71]),
            ("Abuelo, Ancestral Echo", &[0x0C72]),
            ("Malik, Grim Manipulator", &[0x0C73]),
            ("Etali, Primal Storm", &[0x0C74]),
            ("Rashmi, Eternities Crafter", &[0x0C75]),
            ("Sephara, Sky's Blade", &[0x0C76]),
            ("The Rani", &[0x0C77]),
            ("Traxos, Scourge of Kroog", &[0x0C78]),
            ("Momo, Friendly Flier", &[0x0C79]),
            ("Toph, Greatest Earthbender", &[0x0C7A]),
            ("Stiltzkin, Moogle Merchant", &[0x0C7B]),
            ("Surrak Dragonclaw", &[0x0C7C]),
            ("Mirri, Weatherlight Duelist", &[0x0C7D]),
            ("Kangee, Sky Warden", &[0x0C7E]),
            ("Laelia, the Blade Reforged", &[0x0C7F]),
            ("Gorma, the Gullet", &[0x0C80]),
            ("Berta, Wise Extrapolator", &[0x0C81]),
            ("Bess, Soul Nourisher", &[0x0C82]),
            ("MacCready, Lamplight Mayor", &[0x0C83]),
            ("The Notary Hobbits", &[0x0C84]),
            ("Samut, Voice of Dissent", &[0x0C85]),
            ("Shay Cormac", &[0x0C86]),
            ("Emiel the Blessed", &[0x0C87]),
            ("Bhaal, Lord of Murder", &[0x0C88]),
            ("Syr Ginger, the Meal Ender", &[0x0C89]),
            ("Wan Shi Tong, Librarian", &[0x0C8A]),
            ("Yeva, Nature's Herald", &[0x0C8B]),
            ("Chainer, Dementia Master", &[0x0C8C]),
            ("Moseo, Vein's New Dean", &[0x0C8D]),
            ("Kroxa and Kunoros", &[0x0C8E]),
            ("Prime Speaker Vannifar", &[0x0C8F]),
            ("Slogurk, the Overslime", &[0x0C90]),
            ("Hofri Ghostforge", &[0x0C91]),
            ("Shao Jun", &[0x0C92]),
            ("Jalira, Master Polymorphist", &[0x0C93]),
            ("Krenko, Tin Street Kingpin", &[0x0C94]),
            ("Kemba, Kha Regent", &[0x0C95]),
            ("Nymris, Oona's Trickster", &[0x0C96]),
            ("Tariel, Reckoner of Souls", &[0x0C97]),
            ("Greta, Sweettooth Scourge", &[0x0C98]),
            ("Zodiark, Umbral God", &[0x0C99]),
            ("The Cyber-Controller", &[0x0C9A]),
            ("Rune-Tail, Kitsune Ascendant", &[0x0C9B]),
            ("Lyra Dawnbringer", &[0x0C9C]),
            ("Appa, Steadfast Guardian", &[0x0C9D]),
            ("Venat, Heart of Hydaelyn", &[0x0C9E]),
            ("Ayara, Widow of the Realm", &[0x0C9F]),
            ("Duke Ulder Ravengard", &[0x0CA0]),
            ("Archangel Avacyn", &[0x0CA1]),
            ("Taigam, Ojutai Master", &[0x0CA2]),
            ("Mica, Reader of Ruins", &[0x0CA3]),
            ("Far Fortune, End Boss", &[0x0CA4]),
            ("The Odd Acorn Gang", &[0x0CA5]),
            ("Sarkhan, Dragon Ascendant", &[0x0CA6]),
            ("Cadric, Soul Kindler", &[0x0CA7]),
            ("Haliya, Guided by Light", &[0x0CA8]),
            ("Beza, the Bounding Spring", &[0x0CA9]),
            ("Katara, Water Tribe's Hope", &[0x0CAA]),
            ("Karlach, Fury of Avernus and Sword Coast Sailor", &[0x0CAB]),
            ("Breeches, the Blastmaker", &[0x0CAC]),
            ("Seizan, Perverter of Truth", &[0x0CAD]),
            ("Jaxis, the Troublemaker", &[0x0CAE]),
            ("Tellah, Great Sage", &[0x0CAF]),
            ("Garna, Bloodfist of Keld", &[0x0CB0]),
            ("Torens, Fist of the Angels", &[0x0CB1]),
            ("Rith, the Awakener", &[0x0CB2]),
            ("Longshot, Rebel Bowman", &[0x0CB3]),
            ("Cid, Timeless Artificer", &[0x0CB4]),
            ("Birgi, God of Storytelling", &[0x0CB5]),
            ("Snapdax, Apex of the Hunt", &[0x0CB6]),
            ("Monk Gyatso", &[0x0CB7]),
            ("Basim Ibn Ishaq", &[0x0CB8]),
            ("Iron Spider, Stark Upgrade", &[0x0CB9]),
            ("Piru, the Volatile", &[0x0CBA]),
            ("Mahadi, Emporium Master", &[0x0CBB]),
            ("Zurgo and Ojutai", &[0x0CBC]),
            ("Kokusho, the Evening Star", &[0x0CBD]),
            ("Lagomos, Hand of Hatred", &[0x0CBE]),
            ("Inquisitor Eisenhorn", &[0x0CBF]),
            ("Oswald Fiddlebender", &[0x0CC0]),
            ("Abomination of Llanowar", &[0x0CC1]),
            ("Skrelv, Defector Mite", &[0x0CC2]),
            ("Orthion, Hero of Lavabrink", &[0x0CC3]),
            ("Olag and Miau, New Friends", &[0x0CC4]),
            ("Lagrella, the Magpie", &[0x0CC5]),
            ("Caradora, Heart of Alacria", &[0x0CC6]),
            ("Ezuri, Stalker of Spheres", &[0x0CC7]),
            ("Toluz, Clever Conductor", &[0x0CC8]),
            ("Urianger Augurelt", &[0x0CC9]),
            ("Maelstrom Wanderer", &[0x0CCA]),
            ("Shirei, Shizo's Caretaker", &[0x0CCB]),
            ("Gluntch, the Bestower", &[0x0CCC]),
            ("Zoraline, Cosmos Caller", &[0x0CCD]),
            ("Raph & Mikey, Troublemakers", &[0x0CCE]),
            ("The Gitrog, Ravenous Ride", &[0x0CCF]),
            ("Thantis, the Warweaver", &[0x0CD0]),
            ("Pramikon, Sky Rampart", &[0x0CD1]),
            ("Ral, Monsoon Mage", &[0x0CD2]),
            ("Ruric Thar, the Unbowed", &[0x0CD3]),
            ("Ognis, the Dragon's Lash", &[0x0CD4]),
            ("Zethi, Arcane Blademaster", &[0x0CD5]),
            ("Karametra, God of Harvests", &[0x0CD6]),
            ("Heliod, the Radiant Dawn", &[0x0CD7]),
            ("Minn, Wily Illusionist", &[0x0CD8]),
            ("Karn, Legacy Reforged", &[0x0CD9]),
            ("Aeve, Progenitor Ooze", &[0x0CDA]),
            ("Kelsien, the Plague", &[0x0CDB]),
            ("Mister Negative", &[0x0CDC]),
            ("Imoti, Celebrant of Bounty", &[0x0CDD]),
            ("Averna, the Chaos Bloom", &[0x0CDE]),
            ("Ashling, Rekindled", &[0x0CDF]),
            ("Kenessos, Priest of Thassa", &[0x0CE0]),
            ("Aphelia, Viper Whisperer", &[0x0CE1]),
            ("Baba Lysaga, Night Witch", &[0x0CE2]),
            ("Kros, Defense Contractor", &[0x0CE3]),
            ("Ultron, Artificial Malevolence", &[0x0CE4]),
            ("The Council of Four", &[0x0CE5]),
            ("Odric, Lunarch Marshal", &[0x0CE6]),
            ("Obosh, the Preypiercer", &[0x0CE7]),
            ("Amareth, the Lustrous", &[0x0CE8]),
            ("Cecil, Dark Knight", &[0x0CE9]),
            ("Muxus, Goblin Grandee", &[0x0CEA]),
            ("Minwu, White Mage", &[0x0CEB]),
            ("Gylwain, Casting Director", &[0x0CEC]),
            ("Ghalta and Mavren", &[0x0CED]),
            ("King T'Challa", &[0x0CEE]),
            ("Mikaeus, the Unhallowed", &[0x0CEF]),
            ("Taborax, Hope's Demise", &[0x0CF0]),
            ("Satoru, the Infiltrator", &[0x0CF1]),
            ("Arjun, the Shifting Flame", &[0x0CF2]),
            ("Oona, Queen of the Fae", &[0x0CF3]),
            ("Gollum, Obsessed Stalker", &[0x0CF4]),
            ("Eruth, Tormented Prophet", &[0x0CF5]),
            ("Captain America, Living Legend", &[0x0CF6]),
            ("Gretchen Titchwillow", &[0x0CF7]),
            ("Super-Skrull", &[0x0CF8]),
            ("Godo, Bandit Warlord", &[0x0CF9]),
            ("Mortarion, Daemon Primarch", &[0x0CFA]),
            ("Sek'Kuar, Deathkeeper", &[0x0CFB]),
            ("Gallia of the Endless Dance", &[0x0CFC]),
            ("Anje, Maid of Dishonor", &[0x0CFD]),
            ("Edgar, Charmed Groom", &[0x0CFE]),
            ("Lazav, the Multifarious", &[0x0CFF]),
            ("Eight-and-a-Half-Tails", &[0x0D00]),
            ("Quintorius, Loremaster", &[0x0D01]),
            ("Tataru Taru", &[0x0D02]),
            ("Black Cat, Cunning Thief", &[0x0D03]),
            ("Dragonlord Ojutai", &[0x0D04]),
            ("Grumgully, the Generous", &[0x0D05]),
            ("Wylie Duke, Atiin Hero", &[0x0D06]),
            ("Yue, the Moon Spirit", &[0x0D07]),
            ("Velomachus Lorehold", &[0x0D08]),
            ("Akiri, Fearless Voyager", &[0x0D09]),
            ("Nicol Bolas", &[0x0D0A]),
            ("Thassa, Deep-Dwelling", &[0x0D0B]),
            ("Doctor Doom, Unrivaled", &[0x0D0C]),
            ("Toralf, God of Fury", &[0x0D0D]),
            ("Magda, the Hoardmaster", &[0x0D0E]),
            ("Ghalta, Stampede Tyrant", &[0x0D0F]),
            ("Kain, Traitorous Dragoon", &[0x0D10]),
            ("Anzrag, the Quake-Mole", &[0x0E00]),
            ("Ardbert, Warrior of Darkness", &[0x0E01]),
            ("Fire Lord Ozai", &[0x0E02]),
            ("Hinata, Dawn-Crowned", &[0x0E03]),
            ("Ian Malcolm, Chaotician", &[0x0E04]),
            ("Imodane, the Pyrohammer", &[0x0E05]),
            ("Ivy, Gleeful Spellthief", &[0x0E06]),
            ("Maarika, Brutal Gladiator", &[0x0E07]),
            ("Megatron, Tyrant", &[0x0E08]),
            ("Saruman of Many Colors", &[0x0E09]),
            ("Heartless Hidetsugu", &[0x0E0A]),
            ("Chromium, the Mutable", &[0x0E0B]),
            ("Themberchaud", &[0x0E0C]),
            ("Emet-Selch of the Third Seat", &[0x0E0D]),
            ("Erinis, Gloom Stalker and Street Urchin", &[0x0E0E]),
            ("Sarkhan, Soul Aflame", &[0x0E0F]),
            ("Fandaniel, Telophoroi Ascian", &[0x0E10]),
            ("Tawnos, the Toymaker", &[0x0E11]),
            ("Alistair, the Brigadier", &[0x0E12]),
            ("Rigo, Streetwise Mentor", &[0x0E13]),
            ("Krav, the Unredeemed and Regna, the Redeemer", &[0x0E14]),
            ("Queen Allenal of Ruadach", &[0x0E15]),
            ("Lo and Li, Royal Advisors", &[0x0E16]),
            ("Umori, the Collector", &[0x0E17]),
            ("Ingris Stingerquill", &[0x0E18]),
            ("Wulfgar of Icewind Dale", &[0x0E19]),
            ("Arbaaz Mir", &[0x0E1A]),
            ("Rip, Spawn Hunter", &[0x0E1B]),
            ("Greven, Predator Captain", &[0x0E1C]),
            ("Vraska, the Silencer", &[0x0E1D]),
            ("Karazikar, the Eye Tyrant", &[0x0E1E]),
            ("Ratonhnhaké꞉ton", &[0x0E1F]),
            ("Arcanis the Omnipotent", &[0x0E20]),
            ("Trostani, Three Whispers", &[0x0E21]),
            ("Runadi, Behemoth Caller", &[0x0E22]),
            ("Rakdos, the Showstopper", &[0x0E23]),
            ("Krang, Utrom Warlord", &[0x0E24]),
            ("Yes Man, Personal Securitron", &[0x0E25]),
            ("Nissa, Worldsoul Speaker", &[0x0E26]),
            ("Mazzy, Truesword Paladin", &[0x0E27]),
            ("Sedris, the Traitor King", &[0x0E28]),
            ("Starscream, Power Hungry", &[0x0E29]),
            ("Noyan Dar, Roil Shaper", &[0x0E2A]),
            ("Vaan, Street Thief", &[0x0E2B]),
            ("Magnus the Red", &[0x0E2C]),
            ("Yusri, Fortune's Flame", &[0x0E2D]),
            ("Nine-Fingers Keene", &[0x0E2E]),
            ("Xu-Ifit, Osteoharmonist", &[0x0E2F]),
            ("Duskana, the Rage Mother", &[0x0E30]),
            ("Hapatra, Vizier of Poisons", &[0x0E31]),
            ("Grenzo, Dungeon Warden", &[0x0E32]),
            ("Sergeant John Benton", &[0x0E33]),
            ("Magar of the Magic Strings", &[0x0E34]),
            ("Narci, Fable Singer", &[0x0E35]),
            ("Yennett, Cryptic Sovereign", &[0x0E36]),
            ("Maha, Its Feathers Night", &[0x0E37]),
            ("General Ferrous Rokiric", &[0x0E38]),
            ("Greasefang, Okiba Boss", &[0x0E39]),
            ("The Emperor of Palamecia", &[0x0E3A]),
            ("Owen Grady, Raptor Trainer", &[0x0E3B]),
            ("Rydia, Summoner of Mist", &[0x0E3C]),
            ("Sheoldred", &[0x0E3D]),
            ("Patron of the Moon", &[0x0E3E]),
            ("Pia Nalaar, Chief Mechanic", &[0x0E3F]),
            ("Bane, Lord of Darkness", &[0x0E40]),
            ("Nevinyrral, Urborg Tyrant", &[0x0E41]),
            ("Smaug, Wicked Worm", &[0x0E42]),
            ("Nikya of the Old Ways", &[0x0E43]),
            ("Phelddagrif", &[0x0E44]),
            ("Ashaya, Soul of the Wild", &[0x0E45]),
            ("Zidane, Tantalus Thief", &[0x0E46]),
            ("Shadrix Silverquill", &[0x0E47]),
            ("Gandalf the White", &[0x0E48]),
            ("Oskar, Rubbish Reclaimer", &[0x0E49]),
            ("Xanathar, Guild Kingpin", &[0x0E4A]),
            ("Y'shtola Rhul", &[0x0E4B]),
            ("Mass of Mysteries", &[0x0E4C]),
            ("Ozox, the Clattering King", &[0x0E4D]),
            ("The Sixth Doctor and Romana II", &[0x0E4E]),
            ("Electro, Assaulting Battery", &[0x0E4F]),
            ("Ozai, the Phoenix King", &[0x0E50]),
            ("Ruby, Daring Tracker", &[0x0E51]),
            ("Kami of the Crescent Moon", &[0x0E52]),
            ("Ashling, Flame Dancer", &[0x0E53]),
            ("Captain Sisay", &[0x0E54]),
            ("Kethis, the Hidden Hand", &[0x0E55]),
            ("Chevill, Bane of Monsters", &[0x0E56]),
            ("Nashi, Searcher in the Dark", &[0x0E57]),
            ("Mirko Vosk, Mind Drinker", &[0x0E58]),
            ("Purphoros, Bronze-Blooded", &[0x0E59]),
            ("Wilson, Refined Grizzly and Flaming Fist", &[0x0E5A]),
            ("Titania, Nature's Force", &[0x0E5B]),
            ("Titania, Voice of Gaea", &[0x0E5C]),
            ("Queen Kayla bin-Kroog", &[0x0E5D]),
            ("Jackal, Genius Geneticist", &[0x0E5E]),
            ("Lluwen, Imperfect Naturalist", &[0x0E5F]),
            ("Marvin, Murderous Mimic", &[0x0E60]),
            ("Tawnos, Solemn Survivor", &[0x0E61]),
            ("Inniaz, the Gale Force", &[0x0E62]),
            ("Vaevictis Asmadi, the Dire", &[0x0E63]),
            ("Cormela, Glamour Thief", &[0x0E64]),
            ("Pia Nalaar, Consul of Revival", &[0x0E65]),
            ("Smaug the Magnificent", &[0x0E66]),
            ("Omnath, Locus of the Roil", &[0x0E67]),
            ("Radha, Heir to Keld", &[0x0E68]),
            ("Kylox, Visionary Inventor", &[0x0E69]),
            ("Geralf, the Fleshwright", &[0x0E6A]),
            ("The Sentry, Golden Guardian", &[0x0E6B]),
            ("Taskmaster, Mercenary Mimic", &[0x0E6C]),
            ("Huatli, Poet of Unity", &[0x0E6D]),
            ("Nashi, Moon's Legacy", &[0x0E6E]),
            ("Tekuthal, Inquiry Dominus", &[0x0E6F]),
            ("The Serpent Society", &[0x0E70]),
            ("The Master, Multiplied", &[0x0E71]),
            ("Mayael the Anima", &[0x0E72]),
            ("Gix, Yawgmoth Praetor", &[0x0E73]),
            ("Arahbo, the First Fang", &[0x0E74]),
            ("Edric, Spymaster of Trest", &[0x0E75]),
            ("Six", &[0x0E76]),
            ("Thrun, Breaker of Silence", &[0x0E77]),
            ("Obyra, Dreaming Duelist", &[0x0E78]),
            ("Ajani, Nacatl Pariah", &[0x0E79]),
            ("Shattergang Brothers", &[0x0E7A]),
            ("Tinybones, the Pickpocket", &[0x0E7B]),
            ("Toshiro Umezawa", &[0x0E7C]),
            ("Aveline de Grandpré", &[0x0E7D]),
            ("Ganax, Astral Hunter and Feywild Visitor", &[0x0E7E]),
            ("Brion Stoutarm", &[0x0E7F]),
            ("Aerith Gainsborough", &[0x0E80]),
            ("Olivia, Crimson Bride", &[0x0E81]),
            ("Katilda and Lier", &[0x0E82]),
            ("Endrek Sahr, Master Breeder", &[0x0E83]),
            ("Avatar Roku, Firebender", &[0x0E84]),
            ("Kraven the Hunter", &[0x0E85]),
            ("Amarant Coral", &[0x0E86]),
            ("Inferno of the Star Mounts", &[0x0E87]),
            ("Commander Mustard", &[0x0E88]),
            ("Ashling the Pilgrim", &[0x0E89]),
            ("Tetsuko Umezawa, Fugitive", &[0x0E8A]),
            ("Blor the Impervious", &[0x0E8B]),
            ("Phlage, Titan of Fire's Fury", &[0x0E8C]),
            ("Parnesse, the Subtle Brush", &[0x0E8D]),
            ("Kassandra, Eagle Bearer", &[0x0E8E]),
            ("Minsc, Beloved Ranger", &[0x0E8F]),
        ];
        // One work item per (seat, seed), pulled by a worker per core: this
        // test is the suite's long pole (~630 s of a ~640 s debug run on four
        // cores when it walked the list on one thread). Each game is seeded,
        // so the order they run in changes nothing. `RUST_MIN_STACK` (32 MiB,
        // `.cargo/config.toml`) sizes the workers' stacks.
        let jobs: Vec<(&str, u64)> =
            seats.iter().flat_map(|(name, seeds)| seeds.iter().map(move |&seed| (*name, seed))).collect();
        let next = std::sync::atomic::AtomicUsize::new(0);
        let workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(jobs.len());
        std::thread::scope(|s| {
            for _ in 0..workers {
                s.spawn(|| {
                    while let Some(&(name, seed)) = jobs.get(next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)) {
                        let precon = *field.iter().find(|d| d.name.starts_with(name)).expect("the precon seat");
                        assert_eq!(precon.card_count(), 100);
                        let decks = vec![precon, field[0], field[1], field[2]];
                        let t = build_pod_template(&decks);
                        assert_eq!(t.players[0].library.len(), 100 - precon.commanders.len());
                        let pilots = vec![Pilot::default(); 4];
                        let o = play_one_pod_game(&t, &pilots, 50_000, seed);
                        assert!(
                            o.winner.is_some(),
                            "{name} seed {seed} left the pod undecided ({:?}, turn {}, {} actions)",
                            o.stop,
                            o.turns,
                            o.actions
                        );
                        assert!(o.turns > 0);
                    }
                });
            }
        });
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

    /// CR 732.2a — an optional loop stops where its player chooses: past the
    /// per-turn cap the same activation of the same permanent is a pass.
    #[test]
    fn cr_732_2a_a_seat_stops_repeating_one_activation_past_the_cap() {
        let act = |card, ability_index| crate::game::GameAction::ActivateAbility {
            card_id: crate::card::CardId(card),
            ability_index,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        };
        let mut guard = RepeatGuard::default();
        for _ in 0..POD_SAME_ACTIVATION_PER_TURN {
            assert!(guard.admit(7, 1, &act(408, 0)));
        }
        assert!(!guard.admit(7, 1, &act(408, 0)), "the 65th in one turn is a pass");
        assert!(guard.admit(7, 1, &act(408, 1)), "another ability");
        assert!(guard.admit(7, 2, &act(408, 0)), "another seat");
        assert!(guard.admit(7, 1, &crate::game::GameAction::PassPriority));
        assert!(guard.admit(8, 1, &act(408, 0)), "a new turn resets it");
    }

    /// CR 732.2a — an Equip (its own action, not `ActivateAbility`) counts
    /// too: a free equip moved back and forth is the same loop (Puresteel
    /// Paladin and Assimilation Aegis, sweep 306001, 401 equips in a turn).
    #[test]
    fn cr_732_2a_a_seat_stops_re_equipping_past_the_cap() {
        let equip = |to| crate::game::GameAction::Equip {
            equipment: crate::card::CardId(303),
            target: crate::card::CardId(to),
        };
        let mut guard = RepeatGuard::default();
        for i in 0..POD_SAME_ACTIVATION_PER_TURN {
            assert!(guard.admit(38, 3, &equip(343 + (i % 2) * 57)));
        }
        assert!(!guard.admit(38, 3, &equip(343)), "the 65th equip of one Equipment is a pass");
    }

    /// CR 733.1 — a cast its own answer made illegal is rewound; casting the
    /// same card again past the per-turn cap is a pass (Launch the Fleet, 643
    /// rewound casts in one turn under `--a uniform`).
    #[test]
    fn cr_733_1_a_seat_stops_recasting_one_card_past_the_cap() {
        let cast = |card| crate::game::GameAction::CastSpell {
            card_id: crate::card::CardId(card),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let mut guard = RepeatGuard::default();
        for _ in 0..POD_SAME_ACTIVATION_PER_TURN {
            assert!(guard.admit(3, 0, &cast(6)));
        }
        assert!(!guard.admit(3, 0, &cast(6)), "the 65th cast in one turn is a pass");
        assert!(guard.admit(3, 0, &cast(7)), "another card");
        assert!(guard.admit(4, 0, &cast(6)), "a new turn resets it");
    }

    /// CR 732.2a — an optional loop spread over many copies of one card is
    /// still one loop: the 65th cast of a name in a turn is a pass, whichever
    /// copy it is (Shadowborn Apostle under Edgewalker and Athreos, God of
    /// Passage, seed 17100148).
    #[test]
    fn cr_732_2a_a_seat_stops_recasting_one_name_past_the_cap() {
        let cast = |card| crate::game::GameAction::CastSpell {
            card_id: crate::card::CardId(card),
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        };
        let mut guard = RepeatGuard::default();
        for i in 0..POD_SAME_ACTIVATION_PER_TURN {
            assert!(guard.admit_named(3, 0, &cast(100 + i % 16), Some("Shadowborn Apostle")));
        }
        assert!(!guard.admit_named(3, 0, &cast(200), Some("Shadowborn Apostle")), "a fresh copy, same loop");
        assert!(guard.admit_named(3, 0, &cast(201), Some("Grizzly Bears")), "another name");
        assert!(guard.admit_named(3, 1, &cast(202), Some("Shadowborn Apostle")), "another seat");
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

    /// CR 104.3a / 800.4a — a player may concede at any time, so the leave
    /// path runs mid-combat, mid-payment and under a pending ask, not only
    /// from a state-based check. Four seats, a concession roughly every 200
    /// actions: every game ends, nothing panics, and a seat that left owes
    /// nothing (the loop's own debug assertion).
    #[test]
    fn cr_800_4a_pods_survive_concessions_at_arbitrary_points() {
        let field = pod_field(4);
        let t = build_pod_template(&field);
        let pilots = vec![Pilot::default(); 4];
        for seed in [11u64, 12, 13, 14] {
            let o = play_pod_game(&t, &pilots, 20_000, seed, None, Some(50));
            assert!(matches!(o.stop, StopReason::GameOver), "seed {seed}: {:?}", o.stop);
        }
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

    /// The pod A/B's null: with the same pilot everywhere, the `n` games of
    /// a deal group are one game played `n` times, so the "hero" wins it
    /// exactly once when it decides — a share of `1 / n` with no spread. Any
    /// seat- or deal-dependence leaking into the hero's count breaks that.
    #[test]
    fn a_mirror_pod_ab_gives_the_hero_one_seat_in_n_exactly() {
        let field = pod_field(3);
        let pilot = Pilot::Scored(crate::server::bot::EvalWeights::baseline());
        let t = run_pod_hero_games(&field, 0, 6, 7, 3_000, pilot, pilot);
        assert_eq!(t.tally.games, 6);
        assert_eq!(t.groups.len(), 2);
        for (g, (w, n)) in &t.groups {
            assert_eq!(*n, 3, "group {g}");
            assert_eq!(*w, 1, "group {g}: one game, won once from whichever seat won it");
        }
        let (share, se) = t.share();
        assert!((share - 1.0 / 3.0).abs() < 1e-9 && se < 1e-9, "{share} ± {se}");
        // Split across workers, the same answer.
        let mut split = run_pod_hero_games(&field, 0, 4, 7, 3_000, pilot, pilot);
        split.merge(&run_pod_hero_games(&field, 4, 2, 7, 3_000, pilot, pilot));
        assert_eq!((split.hero_wins, split.groups), (t.hero_wins, t.groups));
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
        // Re-blessed 2026-10-07 (`server/hideaway.rs`, a pod bot plays a
        // hideaway land's hidden card once its condition holds — Mosswort
        // Bridge had never been activated; the suite passed on the parent
        // with the session's other bot pickers in): seed 4242 70→58 turns,
        // 3143→2771 actions, same winner; 0xC0FFEE and 43 unmoved. Release
        // pods, 1,000 games at seeds 152003/4/6/8, before/after: 32.85/32.81,
        // 46.01/45.95, 69.71/69.59, 97.00/97.23 turns at 3/4/6/8 seats, all
        // decided; `--bench` byte-identical.
        // Re-blessed 2026-10-06 (CR 603.3d, the asking queue also takes a
        // prompting seat's non-cast ETB, attack, "whenever you attack",
        // spell-cast and self-cast triggers): seed 4242 67→70 turns, same
        // winner; the other two unmoved. 2,000 games at seed 9991,
        // before/after: 40.85/40.86, 62.81/62.67, 82.69/82.99 turns at 3/4/6
        // seats, every block decided.
        // Re-blessed 2026-10-06 (CR 603.3d, a prompting seat's targeted
        // self-death trigger goes through the asking queue, so a pod bot
        // picks its targets by policy like every other trigger — bisected by
        // disabling that routing alone): seed 43 77→74 turns, 4242 97→67,
        // same winners; 0xC0FFEE unmoved. `bot_ladder --commander --a dflt`,
        // 2,000 games at seed 9991, before/after: 40.45/40.85, 62.51/62.81,
        // 82.27/82.69 turns at 3/4/6 seats, every block 2,000/2,000 decided.
        // Re-blessed 2026-10-05 (`01VKHeZo`, Hideaway asks its player which
        // looked-at card to hide; bisected: its parent passes and every later
        // commit to the tip reads the same rows). A pod bot ranks a library
        // pick by mana value with the FIRST of equal cards, where the old
        // engine pick took the last: seed 4242 89→97 turns, 4409→4745
        // actions; 0xC0FFEE 2482→2486 and 43 3646→3649 actions; same winners.
        // Re-blessed 2026-10-02 (`01NqgJKc`, two causes, each bisected):
        // 0xC0FFEE 58→64 turns, 2585→3089 actions, same winner, at 2edb33140
        // (a prompting seat may decline a clone's copy — its parent
        // e51c60cf8 passes); 4242 4753→4757 actions, same winner and turns,
        // at df2425130 (`Punisher` asks each chooser — its parent passes on
        // that row). Seed 43 unmoved.
        // Re-blessed 2026-10-02 (Kozilek's Command's fourth mode exiles up to
        // X target cards from any graveyards instead of one target player's;
        // bisected: its parent passes, and the later commits to the tip read
        // the same rows): seed 43 71→60 turns, 4242 72→99, same winners;
        // 0xC0FFEE unmoved.
        // Re-blessed 2026-09-29 (seven "Remove a counter" costs paid on
        // activation — a380ef08a; bisected: its parent 1f1b7bc71 passes):
        // Icatian Javelineers' ping became a bare `DealDamage` the bot's ping
        // pickers fire. Seed 43 71→66 turns, 4242 61→72, same winners;
        // 0xC0FFEE unmoved. Pod smoke at that tip: 98 release groups (3 seats
        // × 12 games, 5 × 10, seeds 9473000+ / 9475000+) all decided; cube /
        // sos / sealed × 150 all decided; `--bench` byte-identical.
        // Re-blessed 2026-09-29 (a pod bot no longer casts Walking Ballista /
        // Hangarback Walker for X=0 — `dies_at_x0`; bisected: that commit
        // alone): seed 4242 104→61 turns, same winner; the other two unmoved.
        // `bot_ladder --commander --a dflt`, 2,000 games at seed 9991,
        // before/after: 40.32/40.24, 60.37/59.84, 82.93/82.05 turns at 3/4/6
        // seats, every block 2,000/2,000 decided; `--bench` byte-identical.
        // Re-blessed 2026-09-28 (CR 700.2a — 23 `ChooseN` spells choose their
        // modes at cast, so a pod bot picks Kolaghan's / Kozilek's / Austere
        // Command's modes instead of the fixed defaults; `git bisect` names
        // that commit alone): seed 43 54→71 turns, same winner; the other two
        // unmoved. Pod smoke after it: 3,960 release games (3/4/5/6/8 seats ×
        // every deck × 20, seeds 9931000+ … 9981000+), all decided.
        // Re-blessed 2026-09-28 (Judith, the Scourge Diva pings on her own
        // death — ruling; `git bisect` over bc45fd069..c9adf798b names that
        // commit alone, and every later commit to 42b269a1e reads the same
        // rows): seed 0xC0FFEE seat 2 → seat 1, 68→58 turns; seed 43 seat 2
        // → seat 0, 86→54 turns; 4242 same winner and turns, 2962→2970
        // actions. Pod smoke after it: 3,000 strict debug-assertion games
        // (4/6/8 seats, seeds 2004000+), all decided.
        // Re-blessed 2026-09-27 (`server/pre_combat_tap.rs`, a pod bot taps
        // an opponent's biggest would-be attacker at their beginning of
        // combat): seed 4242 74→61 turns, same winner; the other two
        // unmoved. `bot_ladder --commander --a dflt`, 2,000 games at seed
        // 9991, before/after: 41.02/41.02, 61.08/61.07, 83.87/83.83 turns at
        // 3/4/6 seats, every block 2,000/2,000 decided.
        // Re-blessed 2026-09-27 (pod end-step sinks — bisected by disabling
        // each alone): `pick_cash_in` (a rock's "sacrifice: draw" at six
        // lands) moves seed 43 65→67 turns, same winner;
        // `pick_self_counter_sink` moves 0xC0FFEE one action; 4242 unmoved.
        // (A first cut of `end_step_ping` also fired Walking Ballista's
        // remove-a-counter ping and moved 0xC0FFEE to seat 1 in 45 turns;
        // counter-paid pings are excluded now.)
        // `bot_ladder --commander --a dflt`, 2,000 games at seed 9991,
        // before/after: 41.04/41.02, 61.20/61.08, 84.15/83.87 turns at
        // 3/4/6 seats, every block 2,000/2,000 decided; `--bench` 197,136
        // byte-identical.
        // Re-blessed 2026-09-27 (`server/channel.rs`, a pod bot channels a
        // spare Kamigawa land — bisected by disabling it alone): seed 43
        // 86→65 turns, same winner; 0xC0FFEE 3171→3174 actions; 4242
        // unmoved. 6 seats × decks 1-6, 150 games at seed 3100001,
        // before/after: 68.51/68.49 turns, all decided; `--bench` 197,136
        // byte-identical.
        // Re-blessed 2026-09-27 (CR 103.5c, a multiplayer game's first
        // mulligan is free — bisected: reverting it alone restores the old
        // rows): seed 0xC0FFEE seat 1 → seat 3, 68→71 turns; 4242 seat 0 →
        // seat 3, 60→74; 43 unmoved. `bot_ladder --commander` (baseline
        // pilot), 2,000 games at seed 9991, before/after: 32.56/32.38,
        // 45.41/45.30, 67.85/68.19 turns at 3/4/6 seats, all decided.
        // Re-blessed 2026-09-27 (`EvalWeights::pod_horizon` adopted: the
        // attack sim prices the crack-back of every opponent before our next
        // turn, not just the next seat's). Seed 0xC0FFEE 44→63 turns, same
        // winner; seed 43 58→70 turns and seat 0 → seat 2; 4242 unmoved.
        // The default pilot's aggregate, 2,000 games a seat count at seed
        // 9991, `podhorizon-off` / `dflt`: 22.10/22.10 (duels untouched),
        // 39.42/41.47, 56.92/62.88, 69.60/78.65, 78.81/86.11, 94.72/107.00,
        // 110.99/124.69 turns, every block 2,000/2,000 decided — fewer alpha
        // strikes, longer pods. (`bot_ladder --commander` without `--a`
        // pilots `baseline`, which this does not touch.)
        // Re-blessed 2026-09-26 (CR 605.3b, auto-tap resolves a prompting
        // seat's any-color source inline — bisected to 34774e835, and which
        // lands pay a generic pip changes with it): a pod seat's Command Tower /
        // Arcane Signet tapped for a generic pip used to add nothing and
        // leave its color prompt pending. Seed 43 61→58 turns and 4242
        // 49→62, both now won by seat 0; 0xC0FFEE unmoved. Aggregate, 2,000
        // games a seat count at seed 9991, before/after: 19.67/19.51,
        // 32.92/32.62, 45.96/45.62, 57.25/56.80, 68.47/67.89, 83.14/81.91,
        // 98.68/97.39 turns, every block 2,000/2,000 decided. `--bench`
        // byte-identical (197,136).
        // Re-blessed 2026-09-24 (CR 117.3c, a bot seat lets its own ability
        // from a source resolve before activating it again): seed 43
        // 2394→2395 actions, same winner and turns; the other two unmoved.
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
        // Re-blessed 2026-09-26 (CR 109.2): a "target creature" slot can't
        // name a creature card in a graveyard. Judith's Slaughter Pact was
        // cast at a Walking Ballista that had died (and fizzled); the cast is
        // rejected now and the bot takes its next play. Seed 4242 changes
        // winner (seat 2 → 3); the other two are untouched.
        // Re-blessed 2026-09-26 (`server/generic_sink.rs`): in a pod the bot
        // activates, post-combat, any non-mana ability no shape generator
        // covers when the resolved outcome beats passing. Same winners; seed
        // 43 runs four turns longer, 0xC0FFEE seven actions.
        // Re-blessed 2026-09-27 (`server/manland.rs`): the bot animates a
        // creature-land with idle first-main mana so it can attack. Seed 43
        // changes winner (seat 2 → 0); 0xC0FFEE and 4242 keep theirs.
        // Re-blessed 2026-09-27 (`server/counter_sink.rs`): idle mana puts a
        // Forge of Heroes-style counter on the bot's own permanent. Same
        // winners; seed 43 two turns sooner.
        // Re-blessed 2026-09-28 (`server/graveyard_hate.rs`): at an
        // opponent's end step Ghost Vacuum (Sigarda, Hanna, Tatyova) exiles
        // an opponent's best graveyard card. Seeds 0xC0FFEE (3 → 2) and 43
        // (0 → 2) change winner; aggregate within noise (3,000 games, seed
        // 43: 43.2/17.9/14.3/24.6 → 43.0/17.3/14.3/25.4 %, 45.67 → 45.63
        // turns).
        // Re-blessed 2026-09-28 (Dauthi Voidwalker, Judith's seat: "an exiled
        // card AN OPPONENT OWNS", chosen on resolution — it could free its
        // controller's own void card and targeted the first one): 4242
        // 108→104 turns / 4818→4725 actions, same winner; the other two
        // unmoved. Vesuva and Gideon's Sacrifice sit in no golden seat.
        // Re-blessed 2026-09-28 (CR 605.1a, 2026-09-25 text: an ability that
        // draws or mills beside its mana is not a mana ability — Sigarda's
        // Chromatic Sphere now uses the stack; bisected against the same
        // tip's CR 800.4i change, which moves nothing): 0xC0FFEE 2583→2585
        // and 4242 4815→4818 actions, same winners and turns.
        // Re-blessed 2026-09-28 (`targeting.rs::best_graveyard_card`: a
        // graveyard target names the richest card and reaches every
        // opponent's graveyard — bisected: the picker alone moves it, the
        // same commit's card conversions do not): seed 4242 61→108 turns,
        // same winner; the other two unmoved. Aggregate, 2,000 games at
        // seed 9991, before/after: 40.12/40.06, 60.43/60.02, 83.77/83.32
        // turns at 3/4/6 seats, every block 2,000/2,000 decided; `--bench`
        // 197,136 byte-identical.
        // Re-blessed 2026-09-28 (`server/generic_sink.rs`): the generic
        // activation sink also runs at the end step of the opponent seated
        // just before the bot. Same winners; seed 43 takes 21 more actions.
        // Aggregate within noise (3,000 games, seed 43: 43.0/17.3/14.3/25.4
        // → 42.9/17.2/14.6/25.3 %, 45.63 → 45.62 turns).
        // Re-blessed 2026-09-29 (`server/land_ramp.rs`): pod seats now take
        // their land-fetch activations (Sakura-Tribe Elder, Burnished Hart).
        // Same winners; seed 43 66→71 turns, the other two unmoved.
        // Re-blessed 2026-10-02 (`game/commander_return.rs`): a pod seat
        // prompts, so its CR 903.9a return is a pending `CommanderRedirect`
        // posed once the action settles rather than an answer inside the
        // sweep. Seeds 43 (0 → 3) and 4242 (3 → 1) change winner; aggregate
        // within noise (seed 9301, every deck in 4- and 6-seat blocks, 4,600
        // / 3,100 games, all decided: 42.28 → 42.25 / 66.19 → 66.14 turns).
        // Re-blessed 2026-10-02 (`EvalWeights::pod_share_eval` adopted at
        // T = 60, round 80: a pod position is the seat's share of the table,
        // not its material less the table's sum; reverting the default to 0
        // alone restores the rows above): 0xC0FFEE seat 1 → seat 2, 61→54
        // turns; 43 71→65, same winner; 4242 seat 1 → seat 0, 91→187 turns
        // (a tail game — `podshare-off`'s own four-seat block below runs to
        // 144). Aggregate, `bot_ladder --commander`, 2,000 games at seed
        // 9991, `podshare-off` / `dflt`: 40.63/40.52, 60.44/61.91,
        // 82.80/81.82 turns at 3/4/6 seats, every block 2,000/2,000 decided,
        // longest 87/91, 144/154, 173/164 turns; `--bench` counters
        // byte-identical (duels run the old difference).
        // Re-blessed 2026-10-03 (auto-tapper: a free source pays a coloured
        // pip before a filter ability, so a "{1}, {T}: any color" land no
        // longer eats the pool — reverting that hunk alone restores the rows
        // above): 0xC0FFEE 2491→2492 actions, same winner; 43 seat 3 → seat
        // 0, 65→77 turns; 4242 seat 0 → seat 3, 187→82 turns. Aggregate,
        // `bot_ladder --commander`, 2,000 games at seed 9991, before/after:
        // 32.36/32.43, 45.66/45.60, 68.34/68.20 turns at 3/4/6 seats, every
        // block 2,000/2,000 decided; `--bench` 196,176 byte-identical.
        // Re-blessed 2026-10-04 (hideaway: "you may play that card" now plays
        // a hidden LAND too, where it was cast-only and stranded the land in
        // exile — bisected to that commit alone): 0xC0FFEE 2492→2482
        // actions, same winner; 43 unchanged; 4242 82→89 turns, same winner.
        // Re-blessed 2026-10-06 (CR 903.9b: a pod seat prompts, so a commander
        // bounced to hand or tucked is OFFERED the command zone once the
        // action settles — `commander_redirect_offers` — and the answer is
        // an action; bisected to that commit alone): 0xC0FFEE 2486→2490
        // actions, same winner and turns; 43 and 4242 unmoved.
        const GOLDEN: [(u64, Option<usize>, u32, usize); 3] = [
            (0xC0FFEE, Some(2), 54, 2490),
            (43, Some(0), 74, 3800),
            (4242, Some(3), 58, 2771),
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

    /// CR 732.4 — a mandatory loop that makes progress (each life gain
    /// triggers another) repeats no board, so the loop watch can't call it;
    /// the pass-streak cap ends it as an action cap after `PASS_STREAK_CAP`
    /// passes rather than at the 200-passes-per-play backstop (1.6 M actions
    /// in an 8-seat pod).
    #[test]
    fn a_progressing_mandatory_loop_stops_at_the_pass_streak_cap() {
        use crate::card::{CardDefinition, CardType, EventKind, EventScope, EventSpec, TriggeredAbility};
        use crate::effect::{Effect, Selector, Value};
        let mut t = build_pod_template(&rofellos_pod(4));
        let gain = || Effect::GainLife { who: Selector::You, amount: Value::ONE };
        t.add_card_to_battlefield(0, CardDefinition {
            name: "Endless Font",
            card_types: vec![CardType::Enchantment],
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl),
                    effect: gain(),
                },
                TriggeredAbility { event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl), effect: gain() },
            ],
            ..Default::default()
        });
        let o = play_pod_game(&t, &vec![Pilot::default(); 4], 4_000, 11, None, None);
        assert_eq!(o.stop, StopReason::ActionCap);
        assert!(o.turns <= 2, "the loop starts at the first upkeep, turn {}", o.turns);
        assert!(o.actions < PASS_STREAK_CAP + 5_000, "{} actions", o.actions);
    }
}
