//! Blunder benchmark: fixed positions where one class of play is plainly
//! wrong, scored on the move the pilot makes rather than on a line a solver
//! certifies (`bot_puzzles --blunders`).
//!
//! Why a second benchmark beside [`super::puzzle`]. The outcome puzzles ask
//! "can you find the win?", and a pilot that plays every ordinary turn badly
//! can still pass them. The user's report was the other thing: the bot does
//! obviously bad things — attacks with a 0-power creature, leaves no blocker
//! home against an aggro board, spends removal on the wrong body. Those are
//! judged here, each on the position that isolates it.
//!
//! # Checks, and the twin that keeps them honest
//!
//! An item is a position plus a list of [`Check`]s over what seat 0 did in a
//! window (the rest of this turn, or through the opponent's next one) and
//! where the board ended up. A "never do X" check is passed by a bot that
//! does nothing, so every such item has a **twin**: the same position with
//! one fact changed so that X becomes right (the 0/4 gains an attack
//! trigger; the deathtouch blocker is tapped). A pair scores only when both
//! halves pass, which a pilot that never attacks, or always holds back,
//! cannot do. Outcome checks ([`Check::Alive`], [`Check::Dies`]) fail under
//! a do-nothing pilot by construction and need no twin; the test
//! `the_null_pilot_fails_every_pair_and_every_solo_item` enforces both.
//!
//! The opponent is the default heuristic, latched to any declaration the
//! position already contains, exactly as in [`super::puzzle::play_with`].

use crate::card::{
    CardDefinition, CardId, CardType, CounterType, EventKind, EventScope, EventSpec, Keyword, TriggeredAbility,
};
use crate::catalog;
use crate::effect::{Effect, Selector, Value};
use crate::game::types::TurnStep;
use crate::game::{Attack, AttackTarget, GameAction, GameState, Target, drain_stack, two_player_game};

use super::bot::Bot;
use super::puzzle::synced_bot;

/// The decision class an item tests, so a failure names a picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Area {
    /// Which creatures attack.
    Attack,
    /// Keeping blockers home against a crack-back.
    HoldBack,
    /// Which creatures block.
    Block,
    /// Which body the removal goes on.
    Removal,
    /// Instant-speed plays on the opponent's turn.
    Instant,
    /// Spending the mana there is.
    Mana,
    /// Aiming a spell at the right side of the table.
    Polarity,
    /// Finding the win that is on the board.
    Lethal,
}

impl Area {
    pub fn name(self) -> &'static str {
        match self {
            Area::Attack => "attack",
            Area::HoldBack => "holdback",
            Area::Block => "block",
            Area::Removal => "removal",
            Area::Instant => "instant",
            Area::Mana => "mana",
            Area::Polarity => "polarity",
            Area::Lethal => "lethal",
        }
    }
}

/// What an item's cards are. The first set is built from made-up bodies
/// and a handful of classics, which no net here trained on; the real-card
/// items say whether a failure is the mechanic or the unfamiliar cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cards {
    /// Made-up bodies (`body("Ogre", 4, 4)`) and classics.
    Plain,
    /// Secrets of Strixhaven cards — the champion's and the policy's pool.
    Sos,
    /// Reality Fracture cards. Read these on the FRA leaf
    /// (`CRAB_NET=nets/fra-champion.safetensors`), as the client plays them.
    Fra,
}

impl Cards {
    pub fn name(self) -> &'static str {
        match self {
            Cards::Plain => "plain",
            Cards::Sos => "sos",
            Cards::Fra => "fra",
        }
    }
}

/// One condition on what seat 0 did and where the board ended up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    /// This creature is in one of the seat's attack declarations.
    Attacks(CardId),
    /// It is in none of them.
    NeverAttacks(CardId),
    /// This creature is in one of the seat's block declarations.
    Blocks(CardId),
    /// It is in none of them.
    NeverBlocks(CardId),
    /// This permanent is off the battlefield when the window closes.
    Dies(CardId),
    /// It is still on the battlefield.
    Survives(CardId),
    /// The seat activated an ability of this permanent.
    Activates(CardId),
    /// The seat cast this card.
    Casts(CardId),
    /// The seat cast this card aimed at that permanent.
    CastOn(CardId, CardId),
    /// The seat did not cast this card.
    NeverCasts(CardId),
    /// The opponent has lost when the window closes.
    Wins,
    /// The seat has not lost when the window closes.
    Alive,
}

impl Check {
    fn holds(self, seat: usize, acts: &[GameAction], end: &GameState) -> bool {
        let attacked = |id: CardId| {
            acts.iter().any(|a| matches!(a, GameAction::DeclareAttackers(v) if v.iter().any(|x| x.attacker == id)))
        };
        let blocked = |id: CardId| {
            acts.iter().any(|a| matches!(a, GameAction::DeclareBlockers(v) if v.iter().any(|(b, _)| *b == id)))
        };
        match self {
            Check::Attacks(id) => attacked(id),
            Check::NeverAttacks(id) => !attacked(id),
            Check::Blocks(id) => blocked(id),
            Check::NeverBlocks(id) => !blocked(id),
            Check::Dies(id) => end.battlefield_find(id).is_none(),
            Check::Survives(id) => end.battlefield_find(id).is_some(),
            Check::Activates(id) => acts.iter().any(|a| {
                matches!(a, GameAction::ActivateAbility { card_id, .. } | GameAction::ActivateLoyaltyAbility { card_id, .. } if *card_id == id)
            }),
            Check::Casts(id) => acts.iter().any(|a| matches!(a, GameAction::CastSpell { card_id, .. } if *card_id == id)),
            Check::CastOn(id, at) => acts.iter().any(|a| {
                matches!(a, GameAction::CastSpell { card_id, target: Some(Target::Permanent(t)), .. } if *card_id == id && *t == at)
            }),
            Check::NeverCasts(id) => !Check::Casts(id).holds(seat, acts, end),
            Check::Wins => end.game_over == Some(Some(seat)) || !end.players[1 - seat].is_alive(),
            Check::Alive => end.game_over != Some(Some(1 - seat)) && end.players[seat].is_alive(),
        }
    }

    /// Names are read off the starting position: a game that ended has
    /// cleared its zones.
    fn describe(self, start: &GameState) -> String {
        let name = |id: CardId| {
            start
                .battlefield_find(id)
                .map(|c| c.definition.name)
                .or_else(|| start.players.iter().flat_map(|p| p.hand.iter()).find(|c| c.id == id).map(|c| c.definition.name))
                .unwrap_or("?")
        };
        match self {
            Check::Attacks(id) => format!("attack with {}", name(id)),
            Check::NeverAttacks(id) => format!("keep {} home", name(id)),
            Check::Blocks(id) => format!("block with {}", name(id)),
            Check::NeverBlocks(id) => format!("not block with {}", name(id)),
            Check::Dies(id) => format!("kill {}", name(id)),
            Check::Survives(id) => format!("keep {} alive", name(id)),
            Check::Activates(id) => format!("activate {}", name(id)),
            Check::Casts(id) => format!("cast {}", name(id)),
            Check::CastOn(id, at) => format!("cast {} on {}", name(id), name(at)),
            Check::NeverCasts(id) => format!("hold {}", name(id)),
            Check::Wins => "win".into(),
            Check::Alive => "survive".into(),
        }
    }
}

/// A built position and the checks it is scored on.
pub struct Position {
    pub state: GameState,
    pub checks: Vec<Check>,
}

/// One benchmark item.
pub struct Blunder {
    pub id: &'static str,
    /// The twin's id when this is half of a pair (both name each other).
    pub twin: Option<&'static str>,
    pub area: Area,
    pub cards: Cards,
    /// The set's version the item joined in: 1 is the original 22
    /// (2026-10-30), whose totals earlier notes quote; 2 the 2026-11-10
    /// additions (35 with the first, what the combat fix was gated on); 3
    /// the 2026-11-12 harder items, written after the client's pilot
    /// reached 35/35.
    pub since: u8,
    /// Turns past the current one the window runs: 0 closes it when this
    /// turn ends, 1 runs through the next.
    pub horizon: u32,
    pub build: fn() -> Position,
    /// What is being asked, for the report.
    pub prompt: &'static str,
}

/// The verdict on one item: which checks failed, and what the seat did.
pub struct Verdict {
    pub failed: Vec<String>,
    pub actions: Vec<GameAction>,
}

impl Verdict {
    pub fn ok(&self) -> bool {
        self.failed.is_empty()
    }
}

/// Play `item` with `bot` on seat 0 and judge it.
pub fn judge(item: &Blunder, bot: &mut dyn Bot) -> Verdict {
    let pos = (item.build)();
    let (end, actions) = play_recording(&pos.state, 0, bot, item.horizon);
    let failed = pos
        .checks
        .iter()
        .filter(|c| !c.holds(0, &actions, &end))
        .map(|c| c.describe(&pos.state))
        .collect();
    Verdict { failed, actions }
}

/// [`super::puzzle::play_with`], recording every action the seat made that
/// the engine accepted.
fn play_recording(state: &GameState, seat: usize, bot: &mut dyn Bot, horizon: u32) -> (GameState, Vec<GameAction>) {
    let mut g = state.clone();
    let opp = 1 - seat;
    let mut opp_bot = synced_bot(&g);
    let last_turn = g.turn_number + horizon;
    let mut fuel = 600u32 * (1 + horizon);
    let mut acts = Vec::new();
    while fuel > 0 && !g.is_game_over() && g.turn_number <= last_turn {
        fuel -= 1;
        if let Some(a) = bot.next_action(&g, seat)
            && g.perform_action(a.clone()).is_ok()
        {
            acts.push(a);
            continue;
        }
        if opp_bot.next_action(&g, opp).is_some_and(|a| g.perform_action(a).is_ok()) {
            continue;
        }
        if g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
    (g, acts)
}

// ── Leaf pairs: does an evaluator order two boards the right way? ──────

/// Two positions that differ in one thing, seat 0 in its precombat main in
/// both. A sound evaluator rates `better` above `worse` for seat 0 — the
/// value net's counterpart of a blunder item: the search's rollouts end in
/// a position like one of these and the leaf's reading is all it keeps.
pub struct LeafPair {
    pub id: &'static str,
    pub prompt: &'static str,
    /// `(better, worse)`.
    pub build: fn() -> (GameState, GameState),
}

/// The material heuristic's reading of a leaf, for the table beside the
/// net's (`bot_puzzles --leaf-pairs`).
pub fn material(g: &GameState, seat: usize) -> i32 {
    super::bot::eval_material(g, seat, &super::bot::EvalWeights::default())
}

/// A turn-six shell: five lands a side, empty hands, 20 life.
fn mid() -> GameState {
    let mut g = base();
    g.turn_number = 6;
    for seat in 0..2 {
        lands(&mut g, seat, catalog::island, 5);
    }
    g
}

fn pair(build: impl Fn(&mut GameState, bool)) -> (GameState, GameState) {
    let (mut better, mut worse) = (mid(), mid());
    build(&mut better, true);
    build(&mut worse, false);
    (better, worse)
}

pub fn leaf_pairs() -> Vec<LeafPair> {
    vec![
        LeafPair {
            id: "a_44_beats_nothing",
            prompt: "my 4/4 on an empty table, against an empty table",
            build: || {
                pair(|g, better| {
                    if better {
                        ready(g, 0, body("Ogre", 4, 4));
                    }
                })
            },
        },
        LeafPair {
            id: "their_44_is_bad",
            prompt: "an empty table, against their 4/4 alone",
            build: || {
                pair(|g, better| {
                    if !better {
                        ready(g, 1, body("Ogre", 4, 4));
                    }
                })
            },
        },
        LeafPair {
            id: "keep_the_44_facing_a_11",
            prompt: "my 4/4 against their vanilla 1/1, or both gone",
            build: || {
                pair(|g, better| {
                    if better {
                        ready(g, 0, body("Ogre", 4, 4));
                        ready(g, 1, body("Squire", 1, 1));
                    }
                })
            },
        },
        LeafPair {
            id: "keep_the_44_facing_deathtouch",
            prompt: "my 4/4 against their Typhoid Rats, or both gone (the trade the attack buys)",
            build: || {
                pair(|g, better| {
                    if better {
                        ready(g, 0, body("Ogre", 4, 4));
                        ready(g, 1, catalog::typhoid_rats());
                    }
                })
            },
        },
        LeafPair {
            id: "sos_44_beats_nothing",
            prompt: "my Pestbrood Sloth on an empty table, against an empty table",
            build: || {
                pair(|g, better| {
                    if better {
                        ready(g, 0, catalog::pestbrood_sloth());
                    }
                })
            },
        },
        LeafPair {
            id: "sos_keep_the_44_facing_deathtouch",
            prompt: "my Pestbrood Sloth against their Burrog Banemaker, or both gone",
            build: || {
                pair(|g, better| {
                    if better {
                        ready(g, 0, catalog::pestbrood_sloth());
                        ready(g, 1, catalog::burrog_banemaker());
                    }
                })
            },
        },
        LeafPair {
            id: "two_bodies_beat_one",
            prompt: "two 2/2s against their 2/2, or one",
            build: || {
                pair(|g, better| {
                    ready(g, 0, body("Guard", 2, 2));
                    ready(g, 1, body("Scout", 2, 2));
                    if better {
                        ready(g, 0, body("Guard", 2, 2));
                    }
                })
            },
        },
        LeafPair {
            id: "life_matters",
            prompt: "a 2/2 each at 20 life, or at 6",
            build: || {
                pair(|g, better| {
                    ready(g, 0, body("Guard", 2, 2));
                    ready(g, 1, body("Scout", 2, 2));
                    if !better {
                        g.players[0].life = 6;
                    }
                })
            },
        },
        LeafPair {
            id: "their_flier_is_worse_than_their_bear",
            prompt: "my 0/4 wall against their 2/2, or against their 2/2 flier",
            build: || {
                pair(|g, better| {
                    ready(g, 0, body("Wall", 0, 4));
                    if better {
                        ready(g, 1, body("Scout", 2, 2));
                    } else {
                        ready(g, 1, keyworded("Drake", 2, 2, Keyword::Flying));
                    }
                })
            },
        },
        LeafPair {
            id: "the_chump_blocker_kept",
            prompt: "15 life and my 1/1 against their 5/5, or 20 life and no 1/1 (the chump made at 20)",
            build: || {
                pair(|g, better| {
                    ready(g, 1, body("Giant", 5, 5));
                    if better {
                        ready(g, 0, body("Squire", 1, 1));
                        g.players[0].life = 15;
                    }
                })
            },
        },
        LeafPair {
            id: "a_card_in_hand",
            prompt: "a 2/2 each and Stand Up for Yourself in hand, or the empty hand",
            build: || {
                pair(|g, better| {
                    ready(g, 0, body("Guard", 2, 2));
                    ready(g, 1, body("Scout", 2, 2));
                    if better {
                        g.add_card_to_hand(0, catalog::stand_up_for_yourself());
                    }
                })
            },
        },
    ]
}

// ── Position-building helpers ──────────────────────────────────────────

fn body(name: &'static str, p: i32, t: i32) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Creature], power: p, toughness: t, ..Default::default() }
}

fn keyworded(name: &'static str, p: i32, t: i32, kw: Keyword) -> CardDefinition {
    CardDefinition { keywords: vec![kw], ..body(name, p, t) }
}

/// Seat 0's precombat main, both libraries stocked with 20 Islands so no
/// draw decks anyone, priority with seat 0. Set directly rather than
/// passed into (as `puzzle_fra::base`), so no draw step moves the hand.
fn base() -> GameState {
    let mut g = two_player_game();
    for seat in 0..2 {
        for _ in 0..20 {
            g.add_card_to_library(seat, catalog::island());
        }
        // The engine-read half of the default pilot (adopted round 67).
        g.players[seat].hostile_player_targets = true;
    }
    g.active_player_idx = 0;
    g.step = TurnStep::PreCombatMain;
    g.priority.player_with_priority = 0;
    g
}

/// The same, on the opponent's precombat main.
fn their_turn() -> GameState {
    let mut g = base();
    g.active_player_idx = 1;
    g.priority.player_with_priority = 1;
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

fn tap(g: &mut GameState, id: CardId) {
    if let Some(c) = g.battlefield_find_mut(id) {
        c.tapped = true;
    }
}

fn advance_to(g: &mut GameState, step: TurnStep) {
    let mut fuel = 50;
    while g.step != step && fuel > 0 && !g.is_game_over() {
        fuel -= 1;
        if g.perform_action(GameAction::PassPriority).is_err() {
            break;
        }
    }
}

/// On the opponent's turn: declare `attackers` at seat 0 and stop at the
/// first point seat 0 can answer (the declare-attackers priority window).
fn they_attack(g: &mut GameState, attackers: &[CardId]) {
    advance_to(g, TurnStep::DeclareAttackers);
    let atks = attackers.iter().map(|&attacker| Attack { attacker, target: AttackTarget::Player(0) }).collect();
    g.perform_action(GameAction::DeclareAttackers(atks)).expect("scripted attack");
    drain_stack(g);
}

/// "Whenever this creature attacks, draw a card."
fn draws_on_attack(mut def: CardDefinition) -> CardDefinition {
    def.triggered_abilities.push(TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
    });
    def
}

// ── Attack ─────────────────────────────────────────────────────────────

/// A vanilla 0/4 against a lone 2/2, both at 20: attacking deals nothing
/// and gains nothing.
fn zero_power_stays_home() -> Position {
    let mut g = base();
    let sentry = ready(&mut g, 0, body("Sentry", 0, 4));
    ready(&mut g, 1, body("Scout", 2, 2));
    Position { state: g, checks: vec![Check::NeverAttacks(sentry)] }
}

/// Twin: the 0/4 draws a card when it attacks, and the 2/2 can't hurt it.
fn zero_power_with_an_attack_trigger() -> Position {
    let mut g = base();
    let sentry = ready(&mut g, 0, draws_on_attack(body("Sentry", 0, 4)));
    ready(&mut g, 1, body("Scout", 2, 2));
    Position { state: g, checks: vec![Check::Attacks(sentry)] }
}

/// A 4/4 into an untapped Typhoid Rats: the block trades a one-drop for it.
fn no_attack_into_deathtouch() -> Position {
    let mut g = base();
    let ogre = ready(&mut g, 0, body("Ogre", 4, 4));
    ready(&mut g, 1, catalog::typhoid_rats());
    Position { state: g, checks: vec![Check::NeverAttacks(ogre)] }
}

/// Twin: the Rats are tapped, so the four damage is free.
fn attack_past_a_tapped_deathtoucher() -> Position {
    let mut g = base();
    let ogre = ready(&mut g, 0, body("Ogre", 4, 4));
    let rats = ready(&mut g, 1, catalog::typhoid_rats());
    tap(&mut g, rats);
    Position { state: g, checks: vec![Check::Attacks(ogre)] }
}

/// A 2/2 into an untapped 2/2 first striker: it dies before it deals damage.
fn no_attack_into_first_strike() -> Position {
    let mut g = base();
    let bear = ready(&mut g, 0, body("Bear", 2, 2));
    ready(&mut g, 1, keyworded("Duelist", 2, 2, Keyword::FirstStrike));
    Position { state: g, checks: vec![Check::NeverAttacks(bear)] }
}

/// Twin: the first striker is tapped.
fn attack_past_a_tapped_first_striker() -> Position {
    let mut g = base();
    let bear = ready(&mut g, 0, body("Bear", 2, 2));
    let duelist = ready(&mut g, 1, keyworded("Duelist", 2, 2, Keyword::FirstStrike));
    tap(&mut g, duelist);
    Position { state: g, checks: vec![Check::Attacks(bear)] }
}

/// A haste 3/1 in hand, three Mountains, an empty opposing board: cast it
/// before combat and attack.
fn haste_goes_in_before_combat() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::mountain, 3);
    let raider = g.add_card_to_hand(
        0,
        CardDefinition {
            cost: crate::mana::cost(&[crate::mana::generic(2), crate::mana::r()]),
            ..keyworded("Raider", 3, 1, Keyword::Haste)
        },
    );
    Position { state: g, checks: vec![Check::Attacks(raider)] }
}

// ── Holding back ───────────────────────────────────────────────────────

/// At 4 life with a 1/4 and a 2/2 against three untapped 2/2s. Held home,
/// the two block two of the crack-back and the seat takes 2; send either
/// and the swing back is lethal (the 2/2 survives only if they choose to
/// block it).
fn control_keeps_its_blockers_home() -> Position {
    let mut g = base();
    g.players[0].life = 4;
    ready(&mut g, 0, body("Warden", 1, 4));
    ready(&mut g, 0, body("Guard", 2, 2));
    for _ in 0..3 {
        ready(&mut g, 1, body("Raider", 2, 2));
    }
    Position { state: g, checks: vec![Check::Alive] }
}

/// Twin: the opponent has no creatures, so the same two attack for free.
fn attack_into_an_empty_board() -> Position {
    let mut g = base();
    g.players[0].life = 4;
    let warden = ready(&mut g, 0, body("Warden", 1, 4));
    let guard = ready(&mut g, 0, body("Guard", 2, 2));
    Position { state: g, checks: vec![Check::Attacks(warden), Check::Attacks(guard), Check::Alive] }
}

// ── Removal ────────────────────────────────────────────────────────────

/// Benalish Marshal and two 2/2s (3/3s under it) against a seat at 5 with
/// no creatures and Murder. Killing the Marshal leaves 4 power; killing
/// a 2/2 leaves 6.
fn kill_the_lord() -> Position {
    let mut g = base();
    g.players[0].life = 5;
    lands(&mut g, 0, catalog::swamp, 3);
    g.add_card_to_hand(0, catalog::murder());
    let marshal = ready(&mut g, 1, catalog::benalish_marshal());
    for _ in 0..2 {
        ready(&mut g, 1, body("Soldier", 2, 2));
    }
    Position { state: g, checks: vec![Check::Alive, Check::Dies(marshal)] }
}

/// Twin: the Marshal pumps only a 4/4, and the seat is at 4. Killing the
/// 4/4 leaves 3 power; killing the Marshal leaves 4.
fn kill_the_beater_not_the_lord() -> Position {
    let mut g = base();
    g.players[0].life = 4;
    lands(&mut g, 0, catalog::swamp, 3);
    g.add_card_to_hand(0, catalog::murder());
    ready(&mut g, 1, catalog::benalish_marshal());
    let brute = ready(&mut g, 1, body("Brute", 4, 4));
    Position { state: g, checks: vec![Check::Alive, Check::Dies(brute)] }
}

/// Two 0/4 walls hold the ground; the opponent's 2/2 flier is the only
/// thing that gets through, and the seat is at 2. Murder the flier.
fn kill_the_flier() -> Position {
    let mut g = base();
    g.players[0].life = 2;
    lands(&mut g, 0, catalog::swamp, 3);
    g.add_card_to_hand(0, catalog::murder());
    for _ in 0..2 {
        ready(&mut g, 0, keyworded("Wall", 0, 4, Keyword::Defender));
    }
    let drake = ready(&mut g, 1, keyworded("Drake", 2, 2, Keyword::Flying));
    ready(&mut g, 1, body("Ogre", 3, 3));
    Position { state: g, checks: vec![Check::Alive, Check::Dies(drake)] }
}

/// A 2/2 and a 3/1 attack a seat at 3 with a 3/4 and a Lightning Bolt.
/// Block one and Bolt the other; Bolting the one the 3/4 blocks wastes it
/// and the other's damage is lethal.
fn bolt_the_one_you_cannot_block() -> Position {
    let mut g = their_turn();
    g.players[0].life = 3;
    lands(&mut g, 0, catalog::mountain, 1);
    g.add_card_to_hand(0, catalog::lightning_bolt());
    ready(&mut g, 0, body("Guard", 3, 4));
    let scout = ready(&mut g, 1, body("Scout", 2, 2));
    let raider = ready(&mut g, 1, body("Raider", 3, 1));
    they_attack(&mut g, &[scout, raider]);
    Position { state: g, checks: vec![Check::Alive] }
}

// ── Blocks ─────────────────────────────────────────────────────────────

/// A 5/5 attacks a seat at 20 holding a 1/1: chumping throws the 1/1 away.
fn no_chump_at_high_life() -> Position {
    let mut g = their_turn();
    let squire = ready(&mut g, 0, body("Squire", 1, 1));
    let giant = ready(&mut g, 1, body("Giant", 5, 5));
    they_attack(&mut g, &[giant]);
    Position { state: g, checks: vec![Check::NeverBlocks(squire), Check::Alive] }
}

/// Twin: at 4 life the chump is the only way to live.
fn chump_at_low_life() -> Position {
    let mut g = their_turn();
    g.players[0].life = 4;
    let squire = ready(&mut g, 0, body("Squire", 1, 1));
    let giant = ready(&mut g, 1, body("Giant", 5, 5));
    they_attack(&mut g, &[giant]);
    Position { state: g, checks: vec![Check::Blocks(squire), Check::Alive] }
}

/// A 2/2 attacks into an untapped 3/3 with an empty hand behind it: the
/// block kills it for free.
fn take_the_free_trade() -> Position {
    let mut g = their_turn();
    let guard = ready(&mut g, 0, body("Guard", 3, 3));
    let scout = ready(&mut g, 1, body("Scout", 2, 2));
    they_attack(&mut g, &[scout]);
    Position { state: g, checks: vec![Check::Dies(scout), Check::Survives(guard)] }
}

/// A 5/5 and a 2/2 attack a seat at 20 holding Typhoid Rats: the
/// deathtoucher takes the 5/5.
fn deathtouch_blocks_the_biggest() -> Position {
    let mut g = their_turn();
    ready(&mut g, 0, catalog::typhoid_rats());
    let giant = ready(&mut g, 1, body("Giant", 5, 5));
    let scout = ready(&mut g, 1, body("Scout", 2, 2));
    they_attack(&mut g, &[giant, scout]);
    Position { state: g, checks: vec![Check::Dies(giant)] }
}

// ── Instant speed ──────────────────────────────────────────────────────

/// A 4/4 attacks; Ambush Viper (flash, deathtouch) and two Forests are in
/// hand and on the table. Flash it in and block.
fn flash_in_the_blocker() -> Position {
    let mut g = their_turn();
    lands(&mut g, 0, catalog::forest, 2);
    g.add_card_to_hand(0, catalog::ambush_viper());
    let brute = ready(&mut g, 1, body("Brute", 4, 4));
    they_attack(&mut g, &[brute]);
    Position { state: g, checks: vec![Check::Dies(brute)] }
}

// ── Mana ───────────────────────────────────────────────────────────────

/// Four Islands, Jayemdae Tome, an empty hand: the mana is the Tome's.
fn spend_the_spare_mana() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::island, 4);
    let tome = ready(&mut g, 0, catalog::jayemdae_tome());
    Position { state: g, checks: vec![Check::Activates(tome)] }
}

/// Two Forests out, a Forest and a three-drop in hand: land, then the
/// creature.
fn land_then_the_three_drop() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::forest, 2);
    g.add_card_to_hand(0, catalog::forest());
    let bear = g.add_card_to_hand(
        0,
        CardDefinition { cost: crate::mana::cost(&[crate::mana::generic(2), crate::mana::g()]), ..body("Bear", 3, 3) },
    );
    Position { state: g, checks: vec![Check::Casts(bear)] }
}

// ── Polarity ───────────────────────────────────────────────────────────

/// The seat's 2/2 attacks and is blocked by a 3/3; Giant Growth in hand.
/// Grow the attacker: it survives and the blocker dies.
fn grow_your_own_attacker() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::forest, 1);
    g.add_card_to_hand(0, catalog::giant_growth());
    let bear = ready(&mut g, 0, body("Bear", 2, 2));
    let ogre = ready(&mut g, 1, body("Ogre", 3, 3));
    advance_to(&mut g, TurnStep::DeclareAttackers);
    g.perform_action(GameAction::DeclareAttackers(vec![Attack { attacker: bear, target: AttackTarget::Player(1) }]))
        .expect("scripted attack");
    drain_stack(&mut g);
    advance_to(&mut g, TurnStep::DeclareBlockers);
    // Seat 1 declares; then the window is seat 0's.
    g.perform_action(GameAction::DeclareBlockers(vec![(ogre, bear)])).expect("scripted block");
    drain_stack(&mut g);
    Position { state: g, checks: vec![Check::Survives(bear), Check::Dies(ogre)] }
}

/// Rancor in hand, a 2/2 of ours and a 3/3 of theirs: the aura goes on ours.
fn aura_on_your_own_creature() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::forest, 1);
    let rancor = g.add_card_to_hand(0, catalog::rancor());
    let bear = ready(&mut g, 0, body("Bear", 2, 2));
    ready(&mut g, 1, body("Ogre", 3, 3));
    Position { state: g, checks: vec![Check::CastOn(rancor, bear)] }
}

// ── Added 2026-11-10: same-named permanents, removal before blocks ─────

/// Three 2/2s of one name attack a seat at 4 holding a 1/4 and a 2/2. One
/// blocker on each of two attackers lets 2 through; both on one attacker
/// lets 4. A move list that tells declarations apart by card name carries
/// only the second.
fn split_blocks_across_same_named_attackers() -> Position {
    let mut g = their_turn();
    g.players[0].life = 4;
    ready(&mut g, 0, body("Warden", 1, 4));
    ready(&mut g, 0, body("Guard", 2, 2));
    let raiders: Vec<CardId> = (0..3).map(|_| ready(&mut g, 1, body("Raider", 2, 2))).collect();
    they_attack(&mut g, &raiders);
    Position { state: g, checks: vec![Check::Alive] }
}

fn grow(g: &mut GameState, id: CardId, n: u32) {
    g.battlefield_find_mut(id).expect("on the battlefield").add_counters(CounterType::PlusOnePlusOne, n);
}

/// Two 2/2s of one name, one carrying two +1/+1 counters; a 3/3 attacks.
/// The grown one blocks and kills it; the plain one would die for nothing.
fn the_grown_twin_blocks() -> Position {
    let mut g = their_turn();
    let small = ready(&mut g, 0, body("Guard", 2, 2));
    let big = ready(&mut g, 0, body("Guard", 2, 2));
    grow(&mut g, big, 2);
    let brute = ready(&mut g, 1, body("Brute", 3, 3));
    they_attack(&mut g, &[brute]);
    Position { state: g, checks: vec![Check::Dies(brute), Check::Survives(small), Check::Survives(big)] }
}

/// Murder against two 2/2s of one name, one grown to 5/5, at 5 life with
/// no blockers: only the grown one's death survives the swing back.
fn murder_the_grown_twin() -> Position {
    let mut g = base();
    g.players[0].life = 5;
    lands(&mut g, 0, catalog::swamp, 3);
    let murder = g.add_card_to_hand(0, catalog::murder());
    ready(&mut g, 1, body("Raider", 2, 2));
    let big = ready(&mut g, 1, body("Raider", 2, 2));
    grow(&mut g, big, 3);
    Position { state: g, checks: vec![Check::CastOn(murder, big), Check::Alive] }
}

/// A 5/5 and a 2/2 attack a seat at 2 holding a 2/3 and Murder: the kill
/// has to come before the block — one dies, the 2/3 takes the other.
fn kill_one_attacker_then_block_the_other() -> Position {
    let mut g = their_turn();
    g.players[0].life = 2;
    lands(&mut g, 0, catalog::swamp, 3);
    let murder = g.add_card_to_hand(0, catalog::murder());
    let guard = ready(&mut g, 0, body("Guard", 2, 3));
    let giant = ready(&mut g, 1, body("Giant", 5, 5));
    let scout = ready(&mut g, 1, body("Scout", 2, 2));
    they_attack(&mut g, &[giant, scout]);
    Position { state: g, checks: vec![Check::Casts(murder), Check::Blocks(guard), Check::Alive] }
}

// ── Added 2026-11-10: the first set's shapes on real cards ─────────────
// Every body here is on the battlefield already, so its enters trigger
// never fires: Zealous Lorecaster is a 4/4, Eager Glyphmage a 3/3,
// Studious First-Year a 1/1; Burrog Banemaker's pump needs mana its side
// does not have.

fn sos_no_attack_into_deathtouch() -> Position {
    let mut g = base();
    let big = ready(&mut g, 0, catalog::zealous_lorecaster());
    ready(&mut g, 1, catalog::burrog_banemaker());
    Position { state: g, checks: vec![Check::NeverAttacks(big)] }
}

fn sos_attack_past_a_tapped_deathtoucher() -> Position {
    let mut g = base();
    let big = ready(&mut g, 0, catalog::zealous_lorecaster());
    let frog = ready(&mut g, 1, catalog::burrog_banemaker());
    tap(&mut g, frog);
    Position { state: g, checks: vec![Check::Attacks(big)] }
}

fn sos_no_chump_at_high_life() -> Position {
    let mut g = their_turn();
    let small = ready(&mut g, 0, catalog::studious_first_year());
    let big = ready(&mut g, 1, catalog::zealous_lorecaster());
    they_attack(&mut g, &[big]);
    Position { state: g, checks: vec![Check::NeverBlocks(small), Check::Alive] }
}

fn sos_chump_at_low_life() -> Position {
    let mut g = their_turn();
    g.players[0].life = 4;
    let small = ready(&mut g, 0, catalog::studious_first_year());
    let big = ready(&mut g, 1, catalog::zealous_lorecaster());
    they_attack(&mut g, &[big]);
    Position { state: g, checks: vec![Check::Blocks(small), Check::Alive] }
}

fn sos_take_the_free_trade() -> Position {
    let mut g = their_turn();
    let mine = ready(&mut g, 0, catalog::zealous_lorecaster());
    let theirs = ready(&mut g, 1, catalog::eager_glyphmage());
    they_attack(&mut g, &[theirs]);
    Position { state: g, checks: vec![Check::Dies(theirs), Check::Survives(mine)] }
}

/// Stand Up for Yourself only destroys power 3 or more, so the order is
/// forced: it takes the 4/4 and the 3/3 blocks the 1/1.
fn sos_kill_one_attacker_then_block_the_other() -> Position {
    let mut g = their_turn();
    g.players[0].life = 1;
    lands(&mut g, 0, catalog::plains, 3);
    let answer = g.add_card_to_hand(0, catalog::stand_up_for_yourself());
    let mine = ready(&mut g, 0, catalog::eager_glyphmage());
    let big = ready(&mut g, 1, catalog::zealous_lorecaster());
    let small = ready(&mut g, 1, catalog::studious_first_year());
    they_attack(&mut g, &[big, small]);
    Position { state: g, checks: vec![Check::Casts(answer), Check::Blocks(mine), Check::Alive] }
}

fn fra_no_attack_into_deathtouch() -> Position {
    let mut g = base();
    let big = ready(&mut g, 0, catalog::wrecking_gecko());
    ready(&mut g, 1, catalog::arcane_amphisbaena());
    Position { state: g, checks: vec![Check::NeverAttacks(big)] }
}

fn fra_attack_past_a_tapped_deathtoucher() -> Position {
    let mut g = base();
    let big = ready(&mut g, 0, catalog::wrecking_gecko());
    let snake = ready(&mut g, 1, catalog::arcane_amphisbaena());
    tap(&mut g, snake);
    Position { state: g, checks: vec![Check::Attacks(big)] }
}

fn fra_take_the_free_trade() -> Position {
    let mut g = their_turn();
    let mine = ready(&mut g, 0, catalog::wrecking_gecko());
    let theirs = ready(&mut g, 1, catalog::keeper_of_the_quiet_hour());
    they_attack(&mut g, &[theirs]);
    Position { state: g, checks: vec![Check::Dies(theirs), Check::Survives(mine)] }
}

// ── Added 2026-11-12: harder items, once the client's pilot read 35/35 ──
// Aimed at the leads ML_NOTES leaves open: lethal the search must see,
// tricks and counters it must hold mana for, removal it must not spend,
// activated abilities (the menu search has no ability arm of its own),
// and evasion keywords on attack.

fn costed(def: CardDefinition, generic: u32, green: bool) -> CardDefinition {
    let mut pips = Vec::new();
    if generic > 0 {
        pips.push(crate::mana::generic(generic));
    }
    if green {
        pips.push(crate::mana::g());
    }
    CardDefinition { cost: crate::mana::cost(&pips), ..def }
}

/// The opponent is at 3 with a 2/2; the seat has no creatures and a
/// Lightning Bolt. Bolt the player.
fn bolt_to_the_face_for_lethal() -> Position {
    let mut g = base();
    g.players[1].life = 3;
    lands(&mut g, 0, catalog::mountain, 1);
    g.add_card_to_hand(0, catalog::lightning_bolt());
    ready(&mut g, 1, catalog::grizzly_bears());
    Position { state: g, checks: vec![Check::Wins] }
}

/// Three 2/2s against one untapped 2/2, the opponent at 4: all three in
/// puts at least 4 through whatever it blocks.
fn alpha_strike_for_lethal() -> Position {
    let mut g = base();
    g.players[1].life = 4;
    for _ in 0..3 {
        ready(&mut g, 0, body("Bear", 2, 2));
    }
    ready(&mut g, 1, body("Guard", 2, 2));
    Position { state: g, checks: vec![Check::Wins] }
}

/// Prodigal Sorcerer, the opponent at 1 behind a 0/4 wall: the attack is
/// blocked, the ping is the win.
fn ping_for_lethal() -> Position {
    let mut g = base();
    g.players[1].life = 1;
    ready(&mut g, 0, catalog::prodigal_sorcerer());
    ready(&mut g, 1, keyworded("Wall", 0, 4, Keyword::Defender));
    Position { state: g, checks: vec![Check::Wins] }
}

/// Murder in hand at 20 life; the opponent's only creature is a 1/1
/// flier. Murder is for a threat, not this.
fn hold_the_murder_at_high_life() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::swamp, 3);
    let murder = g.add_card_to_hand(0, catalog::murder());
    ready(&mut g, 1, keyworded("Sprite", 1, 1, Keyword::Flying));
    Position { state: g, checks: vec![Check::NeverCasts(murder), Check::Alive] }
}

/// Twin: at 1 life the same 1/1 flier is lethal, and Murder is the answer.
fn murder_the_flier_at_one_life() -> Position {
    let mut g = base();
    g.players[0].life = 1;
    lands(&mut g, 0, catalog::swamp, 3);
    let murder = g.add_card_to_hand(0, catalog::murder());
    let sprite = ready(&mut g, 1, keyworded("Sprite", 1, 1, Keyword::Flying));
    Position { state: g, checks: vec![Check::CastOn(murder, sprite), Check::Alive] }
}

/// A 3/3 attacks into the seat's 2/2 with Giant Growth and a Forest up:
/// block and grow, the attacker dies and the 2/2 lives.
fn grow_the_blocker() -> Position {
    let mut g = their_turn();
    lands(&mut g, 0, catalog::forest, 1);
    g.add_card_to_hand(0, catalog::giant_growth());
    let bear = ready(&mut g, 0, body("Bear", 2, 2));
    let ogre = ready(&mut g, 1, body("Ogre", 3, 3));
    they_attack(&mut g, &[ogre]);
    Position { state: g, checks: vec![Check::Dies(ogre), Check::Survives(bear)] }
}

/// The opponent's main phase, six Forests and a Craw Wurm in its hand;
/// the seat has two Islands and Counterspell. The Wurm never lands.
fn counter_the_wurm() -> Position {
    let mut g = their_turn();
    lands(&mut g, 0, catalog::island, 2);
    let counter = g.add_card_to_hand(0, catalog::counterspell());
    lands(&mut g, 1, catalog::forest, 6);
    let wurm = g.add_card_to_hand(1, catalog::craw_wurm());
    Position { state: g, checks: vec![Check::Casts(counter), Check::Dies(wurm)] }
}

/// A 3/3 flier against a ground 4/4: nothing can block it.
fn flier_attacks_over_the_ground() -> Position {
    let mut g = base();
    let drake = ready(&mut g, 0, keyworded("Drake", 3, 3, Keyword::Flying));
    ready(&mut g, 1, body("Ogre", 4, 4));
    Position { state: g, checks: vec![Check::Attacks(drake)] }
}

/// Twin: the 4/4 has reach and kills the flier on the block.
fn no_flier_attack_into_reach() -> Position {
    let mut g = base();
    let drake = ready(&mut g, 0, keyworded("Drake", 3, 3, Keyword::Flying));
    ready(&mut g, 1, keyworded("Archer", 4, 4, Keyword::Reach));
    Position { state: g, checks: vec![Check::NeverAttacks(drake)] }
}

/// A 3/3 menace against one untapped 4/4: one creature cannot block it.
fn menace_past_a_lone_blocker() -> Position {
    let mut g = base();
    let brute = ready(&mut g, 0, keyworded("Brute", 3, 3, Keyword::Menace));
    ready(&mut g, 1, body("Ogre", 4, 4));
    Position { state: g, checks: vec![Check::Attacks(brute)] }
}

/// Twin: two untapped 4/4s double-block it, and it dies taking neither.
fn no_menace_attack_into_two_blockers() -> Position {
    let mut g = base();
    let brute = ready(&mut g, 0, keyworded("Brute", 3, 3, Keyword::Menace));
    for _ in 0..2 {
        ready(&mut g, 1, body("Ogre", 4, 4));
    }
    Position { state: g, checks: vec![Check::NeverAttacks(brute)] }
}

/// A 2/2 attacks into the seat's 2/2 first striker: the block is free.
fn first_striker_blocks_for_free() -> Position {
    let mut g = their_turn();
    let duelist = ready(&mut g, 0, keyworded("Duelist", 2, 2, Keyword::FirstStrike));
    let scout = ready(&mut g, 1, body("Scout", 2, 2));
    they_attack(&mut g, &[scout]);
    Position { state: g, checks: vec![Check::Dies(scout), Check::Survives(duelist)] }
}

/// Four Forests, a four-mana 4/4 and a one-mana 1/1 in hand: the mana
/// buys one of them, and the 4/4 is the one.
fn the_four_drop_over_the_one_drop() -> Position {
    let mut g = base();
    lands(&mut g, 0, catalog::forest, 4);
    let ogre = g.add_card_to_hand(0, costed(body("Ogre", 4, 4), 3, true));
    g.add_card_to_hand(0, costed(body("Elf", 1, 1), 0, true));
    Position { state: g, checks: vec![Check::Casts(ogre)] }
}

/// Every item, in a stable order.
pub fn items() -> Vec<Blunder> {
    vec![
        Blunder {
            id: "zero_power_stays_home",
            twin: Some("zero_power_with_an_attack_trigger"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: zero_power_stays_home,
            prompt: "vanilla 0/4 vs a 2/2 — don't attack",
        },
        Blunder {
            id: "zero_power_with_an_attack_trigger",
            twin: Some("zero_power_stays_home"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: zero_power_with_an_attack_trigger,
            prompt: "0/4 that draws on attack vs a 2/2 — attack",
        },
        Blunder {
            id: "no_attack_into_deathtouch",
            twin: Some("attack_past_a_tapped_deathtoucher"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: no_attack_into_deathtouch,
            prompt: "4/4 vs untapped Typhoid Rats — don't attack",
        },
        Blunder {
            id: "attack_past_a_tapped_deathtoucher",
            twin: Some("no_attack_into_deathtouch"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: attack_past_a_tapped_deathtoucher,
            prompt: "4/4 vs tapped Typhoid Rats — attack",
        },
        Blunder {
            id: "no_attack_into_first_strike",
            twin: Some("attack_past_a_tapped_first_striker"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: no_attack_into_first_strike,
            prompt: "2/2 vs an untapped 2/2 first striker — don't attack",
        },
        Blunder {
            id: "attack_past_a_tapped_first_striker",
            twin: Some("no_attack_into_first_strike"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: attack_past_a_tapped_first_striker,
            prompt: "2/2 vs a tapped first striker — attack",
        },
        Blunder {
            id: "haste_goes_in_before_combat",
            twin: None,
            area: Area::Attack,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: haste_goes_in_before_combat,
            prompt: "haste 3/1 in hand, empty board — cast it and attack",
        },
        Blunder {
            id: "control_keeps_its_blockers_home",
            twin: Some("attack_into_an_empty_board"),
            area: Area::HoldBack,
            cards: Cards::Plain,
            since: 1,
            horizon: 1,
            build: control_keeps_its_blockers_home,
            prompt: "4 life, 1/4 + 2/2 vs three 2/2s — hold both",
        },
        Blunder {
            id: "attack_into_an_empty_board",
            twin: Some("control_keeps_its_blockers_home"),
            area: Area::HoldBack,
            cards: Cards::Plain,
            since: 1,
            horizon: 1,
            build: attack_into_an_empty_board,
            prompt: "same two vs an empty board — attack",
        },
        Blunder {
            id: "kill_the_lord",
            twin: Some("kill_the_beater_not_the_lord"),
            area: Area::Removal,
            cards: Cards::Plain,
            since: 1,
            horizon: 1,
            build: kill_the_lord,
            prompt: "Murder: Benalish Marshal + two 2/2s, 5 life",
        },
        Blunder {
            id: "kill_the_beater_not_the_lord",
            twin: Some("kill_the_lord"),
            area: Area::Removal,
            cards: Cards::Plain,
            since: 1,
            horizon: 1,
            build: kill_the_beater_not_the_lord,
            prompt: "Murder: Benalish Marshal + a 4/4, 4 life",
        },
        Blunder {
            id: "kill_the_flier",
            twin: None,
            area: Area::Removal,
            cards: Cards::Plain,
            since: 1,
            horizon: 1,
            build: kill_the_flier,
            prompt: "Murder: 2/2 flier + 3/3 vs two 0/4 walls, 2 life",
        },
        Blunder {
            id: "bolt_the_one_you_cannot_block",
            twin: None,
            area: Area::Removal,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: bolt_the_one_you_cannot_block,
            prompt: "2/2 + 3/1 attack, 3/4 + Bolt, 3 life",
        },
        Blunder {
            id: "no_chump_at_high_life",
            twin: Some("chump_at_low_life"),
            area: Area::Block,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: no_chump_at_high_life,
            prompt: "5/5 attacks, 1/1, 20 life — don't chump",
        },
        Blunder {
            id: "chump_at_low_life",
            twin: Some("no_chump_at_high_life"),
            area: Area::Block,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: chump_at_low_life,
            prompt: "5/5 attacks, 1/1, 4 life — chump",
        },
        Blunder {
            id: "take_the_free_trade",
            twin: None,
            area: Area::Block,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: take_the_free_trade,
            prompt: "2/2 attacks into an untapped 3/3 — block",
        },
        Blunder {
            id: "deathtouch_blocks_the_biggest",
            twin: None,
            area: Area::Block,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: deathtouch_blocks_the_biggest,
            prompt: "5/5 + 2/2 attack, Typhoid Rats — block the 5/5",
        },
        Blunder {
            id: "flash_in_the_blocker",
            twin: None,
            area: Area::Instant,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: flash_in_the_blocker,
            prompt: "4/4 attacks, Ambush Viper in hand — flash and block",
        },
        Blunder {
            id: "spend_the_spare_mana",
            twin: None,
            area: Area::Mana,
            cards: Cards::Plain,
            since: 1,
            horizon: 1,
            build: spend_the_spare_mana,
            prompt: "four lands, Jayemdae Tome, empty hand — draw",
        },
        Blunder {
            id: "land_then_the_three_drop",
            twin: None,
            area: Area::Mana,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: land_then_the_three_drop,
            prompt: "two lands, a land and a three-drop in hand",
        },
        Blunder {
            id: "grow_your_own_attacker",
            twin: None,
            area: Area::Polarity,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: grow_your_own_attacker,
            prompt: "2/2 blocked by a 3/3, Giant Growth — grow yours",
        },
        Blunder {
            id: "aura_on_your_own_creature",
            twin: None,
            area: Area::Polarity,
            cards: Cards::Plain,
            since: 1,
            horizon: 0,
            build: aura_on_your_own_creature,
            prompt: "Rancor, our 2/2 and their 3/3 — enchant ours",
        },
        // ── Added 2026-11-10 (ML_NOTES "The combat window of the menu
        // search"): the shapes that diagnosis found, and real cards. ──
        Blunder {
            id: "split_blocks_across_same_named_attackers",
            twin: None,
            area: Area::Block,
            cards: Cards::Plain,
            since: 2,
            horizon: 0,
            build: split_blocks_across_same_named_attackers,
            prompt: "three 2/2s of one name attack, 1/4 + 2/2, 4 life — one blocker each",
        },
        Blunder {
            id: "the_grown_twin_blocks",
            twin: None,
            area: Area::Block,
            cards: Cards::Plain,
            since: 2,
            horizon: 0,
            build: the_grown_twin_blocks,
            prompt: "3/3 attacks, two 2/2s of one name, one with two +1/+1 counters — that one blocks",
        },
        Blunder {
            id: "murder_the_grown_twin",
            twin: None,
            area: Area::Removal,
            cards: Cards::Plain,
            since: 2,
            horizon: 1,
            build: murder_the_grown_twin,
            prompt: "Murder: two 2/2s of one name, one grown to 5/5, 5 life, no blockers",
        },
        Blunder {
            id: "kill_one_attacker_then_block_the_other",
            twin: None,
            area: Area::Instant,
            cards: Cards::Plain,
            since: 2,
            horizon: 0,
            build: kill_one_attacker_then_block_the_other,
            prompt: "5/5 + 2/2 attack, 2/3 + Murder, 2 life — kill one, block the other",
        },
        Blunder {
            id: "sos_no_attack_into_deathtouch",
            twin: Some("sos_attack_past_a_tapped_deathtoucher"),
            area: Area::Attack,
            cards: Cards::Sos,
            since: 2,
            horizon: 0,
            build: sos_no_attack_into_deathtouch,
            prompt: "Zealous Lorecaster (4/4) vs untapped Burrog Banemaker — don't attack",
        },
        Blunder {
            id: "sos_attack_past_a_tapped_deathtoucher",
            twin: Some("sos_no_attack_into_deathtouch"),
            area: Area::Attack,
            cards: Cards::Sos,
            since: 2,
            horizon: 0,
            build: sos_attack_past_a_tapped_deathtoucher,
            prompt: "Zealous Lorecaster vs tapped Burrog Banemaker — attack",
        },
        Blunder {
            id: "sos_no_chump_at_high_life",
            twin: Some("sos_chump_at_low_life"),
            area: Area::Block,
            cards: Cards::Sos,
            since: 2,
            horizon: 0,
            build: sos_no_chump_at_high_life,
            prompt: "Zealous Lorecaster attacks, Studious First-Year, 20 life — don't chump",
        },
        Blunder {
            id: "sos_chump_at_low_life",
            twin: Some("sos_no_chump_at_high_life"),
            area: Area::Block,
            cards: Cards::Sos,
            since: 2,
            horizon: 0,
            build: sos_chump_at_low_life,
            prompt: "Zealous Lorecaster attacks, Studious First-Year, 4 life — chump",
        },
        Blunder {
            id: "sos_take_the_free_trade",
            twin: None,
            area: Area::Block,
            cards: Cards::Sos,
            since: 2,
            horizon: 0,
            build: sos_take_the_free_trade,
            prompt: "Eager Glyphmage (3/3) attacks into an untapped Zealous Lorecaster — block",
        },
        Blunder {
            id: "sos_kill_one_attacker_then_block_the_other",
            twin: None,
            area: Area::Instant,
            cards: Cards::Sos,
            since: 2,
            horizon: 0,
            build: sos_kill_one_attacker_then_block_the_other,
            prompt: "Zealous Lorecaster + Studious First-Year attack, Eager Glyphmage + Stand Up for Yourself, 1 life",
        },
        Blunder {
            id: "fra_no_attack_into_deathtouch",
            twin: Some("fra_attack_past_a_tapped_deathtoucher"),
            area: Area::Attack,
            cards: Cards::Fra,
            since: 2,
            horizon: 0,
            build: fra_no_attack_into_deathtouch,
            prompt: "Wrecking Gecko (5/5) vs untapped Arcane Amphisbaena — don't attack",
        },
        Blunder {
            id: "fra_attack_past_a_tapped_deathtoucher",
            twin: Some("fra_no_attack_into_deathtouch"),
            area: Area::Attack,
            cards: Cards::Fra,
            since: 2,
            horizon: 0,
            build: fra_attack_past_a_tapped_deathtoucher,
            prompt: "Wrecking Gecko vs tapped Arcane Amphisbaena — attack",
        },
        Blunder {
            id: "fra_take_the_free_trade",
            twin: None,
            area: Area::Block,
            cards: Cards::Fra,
            since: 2,
            horizon: 0,
            build: fra_take_the_free_trade,
            prompt: "Keeper of the Quiet Hour (3/2) attacks into an untapped Wrecking Gecko — block",
        },
        // ── Added 2026-11-12: harder items (see the positions' header). ──
        Blunder {
            id: "bolt_to_the_face_for_lethal",
            twin: None,
            area: Area::Lethal,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: bolt_to_the_face_for_lethal,
            prompt: "opponent at 3 with a 2/2, Lightning Bolt — bolt the player",
        },
        Blunder {
            id: "alpha_strike_for_lethal",
            twin: None,
            area: Area::Lethal,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: alpha_strike_for_lethal,
            prompt: "three 2/2s vs one 2/2, opponent at 4 — attack with all three",
        },
        Blunder {
            id: "ping_for_lethal",
            twin: None,
            area: Area::Lethal,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: ping_for_lethal,
            prompt: "Prodigal Sorcerer, opponent at 1 behind a 0/4 wall — ping",
        },
        Blunder {
            id: "hold_the_murder_at_high_life",
            twin: Some("murder_the_flier_at_one_life"),
            area: Area::Removal,
            cards: Cards::Plain,
            since: 3,
            horizon: 1,
            build: hold_the_murder_at_high_life,
            prompt: "Murder, their 1/1 flier, 20 life — hold it",
        },
        Blunder {
            id: "murder_the_flier_at_one_life",
            twin: Some("hold_the_murder_at_high_life"),
            area: Area::Removal,
            cards: Cards::Plain,
            since: 3,
            horizon: 1,
            build: murder_the_flier_at_one_life,
            prompt: "Murder, their 1/1 flier, 1 life — kill it",
        },
        Blunder {
            id: "grow_the_blocker",
            twin: None,
            area: Area::Instant,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: grow_the_blocker,
            prompt: "3/3 attacks, our 2/2 + Giant Growth — block and grow",
        },
        Blunder {
            id: "counter_the_wurm",
            twin: None,
            area: Area::Instant,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: counter_the_wurm,
            prompt: "their main, Craw Wurm in their hand, Counterspell — counter it",
        },
        Blunder {
            id: "flier_attacks_over_the_ground",
            twin: Some("no_flier_attack_into_reach"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: flier_attacks_over_the_ground,
            prompt: "3/3 flier vs a ground 4/4 — attack",
        },
        Blunder {
            id: "no_flier_attack_into_reach",
            twin: Some("flier_attacks_over_the_ground"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: no_flier_attack_into_reach,
            prompt: "3/3 flier vs a 4/4 with reach — don't attack",
        },
        Blunder {
            id: "menace_past_a_lone_blocker",
            twin: Some("no_menace_attack_into_two_blockers"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: menace_past_a_lone_blocker,
            prompt: "3/3 menace vs one 4/4 — attack",
        },
        Blunder {
            id: "no_menace_attack_into_two_blockers",
            twin: Some("menace_past_a_lone_blocker"),
            area: Area::Attack,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: no_menace_attack_into_two_blockers,
            prompt: "3/3 menace vs two 4/4s — don't attack",
        },
        Blunder {
            id: "first_striker_blocks_for_free",
            twin: None,
            area: Area::Block,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: first_striker_blocks_for_free,
            prompt: "2/2 attacks into our 2/2 first striker — block",
        },
        Blunder {
            id: "the_four_drop_over_the_one_drop",
            twin: None,
            area: Area::Mana,
            cards: Cards::Plain,
            since: 3,
            horizon: 0,
            build: the_four_drop_over_the_one_drop,
            prompt: "four lands, a 4-drop and a 1-drop in hand — cast the 4-drop",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::bot::HeuristicBot;

    /// Never acts; prompts still go to the default heuristic (which owns
    /// the answer to a decision), as in `bot_puzzles`'s `pass` control.
    struct PassBot(HeuristicBot);

    impl Bot for PassBot {
        fn next_action(&mut self, state: &GameState, seat: usize) -> Option<GameAction> {
            if state.pending_decision.as_ref().is_some_and(|d| d.acting_player() == seat) {
                return self.0.next_action(state, seat);
            }
            None
        }
    }

    /// Every item builds a live position, twins name each other, and the
    /// opponent's scripted declarations stuck.
    #[test]
    fn every_item_builds_and_twins_pair_up() {
        let all = items();
        for it in &all {
            let pos = (it.build)();
            assert!(!pos.state.is_game_over(), "{}: game over at build", it.id);
            assert!(!pos.checks.is_empty(), "{}: no checks", it.id);
            if let Some(t) = it.twin {
                let other = all.iter().find(|o| o.id == t).unwrap_or_else(|| panic!("{}: twin {t} missing", it.id));
                assert_eq!(other.twin, Some(it.id), "{}: twin {t} doesn't point back", it.id);
            }
        }
    }

    /// Every leaf pair builds two live positions that differ, with seat 0
    /// to act in both.
    #[test]
    fn every_leaf_pair_builds_two_different_boards() {
        let pairs = leaf_pairs();
        let mut ids: Vec<&str> = pairs.iter().map(|p| p.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), pairs.len(), "ids are unique");
        for p in &pairs {
            let (better, worse) = (p.build)();
            for g in [&better, &worse] {
                assert!(!g.is_game_over(), "{}", p.id);
                assert_eq!(g.player_with_priority(), 0, "{}", p.id);
            }
            let sig = |g: &GameState| {
                let mut v: Vec<(usize, &str)> = g.battlefield.iter().map(|c| (c.controller, c.definition.name)).collect();
                v.sort_unstable();
                (v, g.players[0].life, g.players[1].life, g.players[0].hand.len())
            };
            assert_ne!(sig(&better), sig(&worse), "{}: the two boards are the same", p.id);
        }
    }

    /// The combat window's anchor (`MctsConfig::menu_anchor`): at 4 life
    /// with a 1/1 against a 5/5 every block loses inside the horizon, so
    /// the search's arms tie at zero — and a tie goes to the heuristic's
    /// declaration, the chump, where the bare search took whichever arm its
    /// noise favoured (`mcts-net67-menu-mc`, the client's pilot before the
    /// fix, passed this item 2 times in 20). No net: uniform priors, the material leaf.
    #[test]
    fn the_anchored_combat_search_chumps_when_every_line_loses() {
        use crate::server::mcts::{MENU_COMBAT, MctsBot, MctsConfig};
        let all = items();
        let it = all.iter().find(|i| i.id == "chump_at_low_life").expect("the item");
        for _ in 0..4 {
            let mut bot = MctsBot::new(MctsConfig {
                iterations: 32,
                horizon_turns: 3,
                menu_windows: MENU_COMBAT,
                menu_declare_fix: true,
                menu_anchor: 0.0,
                ..MctsConfig::default()
            });
            let v = judge(it, &mut bot);
            assert!(v.ok(), "failed to {:?}: {:?}", v.failed, v.actions);
        }
    }

    /// The benchmark's honesty condition: a pilot that does nothing scores
    /// nothing. A "never do X" item it passes must have a twin it fails,
    /// and every item without a twin must fail outright — otherwise that
    /// item measures nothing a bot could get wrong.
    #[test]
    fn the_null_pilot_fails_every_pair_and_every_solo_item() {
        let all = items();
        let ok = |id: &str| {
            let it = all.iter().find(|i| i.id == id).unwrap();
            judge(it, &mut PassBot(HeuristicBot::new())).ok()
        };
        for it in &all {
            let mine = ok(it.id);
            match it.twin {
                Some(t) => assert!(!(mine && ok(t)), "{}: the null pilot passes the pair with {t}", it.id),
                None => assert!(!mine, "{}: the null pilot passes a solo item", it.id),
            }
        }
    }
}
