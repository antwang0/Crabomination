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
    CardDefinition, CardId, CardType, EventKind, EventScope, EventSpec, Keyword, TriggeredAbility,
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

/// Every item, in a stable order.
pub fn items() -> Vec<Blunder> {
    vec![
        Blunder {
            id: "zero_power_stays_home",
            twin: Some("zero_power_with_an_attack_trigger"),
            area: Area::Attack,
            horizon: 0,
            build: zero_power_stays_home,
            prompt: "vanilla 0/4 vs a 2/2 — don't attack",
        },
        Blunder {
            id: "zero_power_with_an_attack_trigger",
            twin: Some("zero_power_stays_home"),
            area: Area::Attack,
            horizon: 0,
            build: zero_power_with_an_attack_trigger,
            prompt: "0/4 that draws on attack vs a 2/2 — attack",
        },
        Blunder {
            id: "no_attack_into_deathtouch",
            twin: Some("attack_past_a_tapped_deathtoucher"),
            area: Area::Attack,
            horizon: 0,
            build: no_attack_into_deathtouch,
            prompt: "4/4 vs untapped Typhoid Rats — don't attack",
        },
        Blunder {
            id: "attack_past_a_tapped_deathtoucher",
            twin: Some("no_attack_into_deathtouch"),
            area: Area::Attack,
            horizon: 0,
            build: attack_past_a_tapped_deathtoucher,
            prompt: "4/4 vs tapped Typhoid Rats — attack",
        },
        Blunder {
            id: "no_attack_into_first_strike",
            twin: Some("attack_past_a_tapped_first_striker"),
            area: Area::Attack,
            horizon: 0,
            build: no_attack_into_first_strike,
            prompt: "2/2 vs an untapped 2/2 first striker — don't attack",
        },
        Blunder {
            id: "attack_past_a_tapped_first_striker",
            twin: Some("no_attack_into_first_strike"),
            area: Area::Attack,
            horizon: 0,
            build: attack_past_a_tapped_first_striker,
            prompt: "2/2 vs a tapped first striker — attack",
        },
        Blunder {
            id: "haste_goes_in_before_combat",
            twin: None,
            area: Area::Attack,
            horizon: 0,
            build: haste_goes_in_before_combat,
            prompt: "haste 3/1 in hand, empty board — cast it and attack",
        },
        Blunder {
            id: "control_keeps_its_blockers_home",
            twin: Some("attack_into_an_empty_board"),
            area: Area::HoldBack,
            horizon: 1,
            build: control_keeps_its_blockers_home,
            prompt: "4 life, 1/4 + 2/2 vs three 2/2s — hold both",
        },
        Blunder {
            id: "attack_into_an_empty_board",
            twin: Some("control_keeps_its_blockers_home"),
            area: Area::HoldBack,
            horizon: 1,
            build: attack_into_an_empty_board,
            prompt: "same two vs an empty board — attack",
        },
        Blunder {
            id: "kill_the_lord",
            twin: Some("kill_the_beater_not_the_lord"),
            area: Area::Removal,
            horizon: 1,
            build: kill_the_lord,
            prompt: "Murder: Benalish Marshal + two 2/2s, 5 life",
        },
        Blunder {
            id: "kill_the_beater_not_the_lord",
            twin: Some("kill_the_lord"),
            area: Area::Removal,
            horizon: 1,
            build: kill_the_beater_not_the_lord,
            prompt: "Murder: Benalish Marshal + a 4/4, 4 life",
        },
        Blunder {
            id: "kill_the_flier",
            twin: None,
            area: Area::Removal,
            horizon: 1,
            build: kill_the_flier,
            prompt: "Murder: 2/2 flier + 3/3 vs two 0/4 walls, 2 life",
        },
        Blunder {
            id: "bolt_the_one_you_cannot_block",
            twin: None,
            area: Area::Removal,
            horizon: 0,
            build: bolt_the_one_you_cannot_block,
            prompt: "2/2 + 3/1 attack, 3/4 + Bolt, 3 life",
        },
        Blunder {
            id: "no_chump_at_high_life",
            twin: Some("chump_at_low_life"),
            area: Area::Block,
            horizon: 0,
            build: no_chump_at_high_life,
            prompt: "5/5 attacks, 1/1, 20 life — don't chump",
        },
        Blunder {
            id: "chump_at_low_life",
            twin: Some("no_chump_at_high_life"),
            area: Area::Block,
            horizon: 0,
            build: chump_at_low_life,
            prompt: "5/5 attacks, 1/1, 4 life — chump",
        },
        Blunder {
            id: "take_the_free_trade",
            twin: None,
            area: Area::Block,
            horizon: 0,
            build: take_the_free_trade,
            prompt: "2/2 attacks into an untapped 3/3 — block",
        },
        Blunder {
            id: "deathtouch_blocks_the_biggest",
            twin: None,
            area: Area::Block,
            horizon: 0,
            build: deathtouch_blocks_the_biggest,
            prompt: "5/5 + 2/2 attack, Typhoid Rats — block the 5/5",
        },
        Blunder {
            id: "flash_in_the_blocker",
            twin: None,
            area: Area::Instant,
            horizon: 0,
            build: flash_in_the_blocker,
            prompt: "4/4 attacks, Ambush Viper in hand — flash and block",
        },
        Blunder {
            id: "spend_the_spare_mana",
            twin: None,
            area: Area::Mana,
            horizon: 1,
            build: spend_the_spare_mana,
            prompt: "four lands, Jayemdae Tome, empty hand — draw",
        },
        Blunder {
            id: "land_then_the_three_drop",
            twin: None,
            area: Area::Mana,
            horizon: 0,
            build: land_then_the_three_drop,
            prompt: "two lands, a land and a three-drop in hand",
        },
        Blunder {
            id: "grow_your_own_attacker",
            twin: None,
            area: Area::Polarity,
            horizon: 0,
            build: grow_your_own_attacker,
            prompt: "2/2 blocked by a 3/3, Giant Growth — grow yours",
        },
        Blunder {
            id: "aura_on_your_own_creature",
            twin: None,
            area: Area::Polarity,
            horizon: 0,
            build: aura_on_your_own_creature,
            prompt: "Rancor, our 2/2 and their 3/3 — enchant ours",
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
