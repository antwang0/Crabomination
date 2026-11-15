//! "What would the bot do?" — a hint for a human seat.
//!
//! A seat sends [`ClientMsg::RequestHint`](crate::net::ClientMsg::RequestHint)
//! while it is the one to act; the match answers with
//! [`ServerMsg::Hint`](crate::net::ServerMsg::Hint): the action the hint bot
//! would take for that seat, named the way a take-back names it ("cast
//! Lightning Bolt"), and the card it is about, for the client to point at.
//!
//! The hint bot is whatever the host registers with [`set_hint_bot`] — the
//! client registers its own local pilot (the same search its bots play), so
//! a hint is as good as the opponent you are playing. A host that registers
//! none (the network server) answers that hints are off: a search per hint
//! would let one player spend the server's CPU at will.

use std::sync::OnceLock;

use crate::card::CardId;
use crate::game::{GameAction, GameState};
use crate::net::ServerMsg;

use super::Bot;

/// Builds the bot a hint asks, for the state it is asked in.
pub type HintBotFactory = fn(&GameState) -> Box<dyn Bot>;

static HINT_BOT: OnceLock<HintBotFactory> = OnceLock::new();

/// Register the bot hints ask. Once per process; a second call is ignored.
pub fn set_hint_bot(factory: HintBotFactory) {
    let _ = HINT_BOT.set(factory);
}

/// The seat that must act now: a pending decision's player, else the
/// priority holder.
fn to_act(state: &GameState) -> usize {
    state.pending_decision.as_ref().map_or(state.priority.player_with_priority, |pd| pd.acting_player())
}

/// The card an action is about, if it names one (its `card_id`, or a tuple
/// variant's bare id) — what the client pings on the board.
pub fn action_card(action: &GameAction) -> Option<CardId> {
    let json = serde_json::to_value(action).ok()?;
    let (_, fields) = json.as_object()?.iter().next()?;
    let id = match fields {
        serde_json::Value::Object(f) => f.get("card_id").or_else(|| f.get("attacker")).or_else(|| f.get("equipment"))?,
        number @ serde_json::Value::Number(_) => number,
        _ => return None,
    };
    serde_json::from_value(id.clone()).ok()
}

/// A take-back label ("played Forest", "declared attackers") as an
/// instruction ("play Forest", "declare attackers"): the label names a move
/// already made, a hint one to make.
pub fn imperative(label: &str) -> String {
    const VERBS: &[(&str, &str)] = &[
        ("acted with", "act with"),
        ("activated", "activate"),
        ("answered", "answer"),
        ("conceded", "concede"),
        ("crewed", "crew"),
        ("cycled", "cycle"),
        ("declared", "declare"),
        ("equipped", "equip"),
        ("foretold", "foretell"),
        ("passed", "pass"),
        ("played", "play"),
        ("plotted", "plot"),
        ("put", "put"),
        ("reconfigured", "reconfigure"),
        ("reinforced", "reinforce"),
        ("revealed", "reveal"),
        ("rolled", "roll"),
        ("saddled", "saddle"),
        ("suspended", "suspend"),
        ("turned", "turn"),
        ("unlocked", "unlock"),
    ];
    for (past, now) in VERBS {
        if let Some(rest) = label.strip_prefix(past)
            && (rest.is_empty() || rest.starts_with(' '))
        {
            return format!("{now}{rest}");
        }
    }
    label.to_string()
}

/// The hint for `seat` in `state`.
pub fn hint_for(state: &GameState, seat: usize) -> ServerMsg {
    let none = |note: &str| ServerMsg::Hint { label: None, card: None, note: note.to_string() };
    let Some(factory) = HINT_BOT.get() else { return none("hints are off on this server") };
    if state.is_game_over() {
        return none("the game is over");
    }
    if to_act(state) != seat {
        return none("it isn't your move");
    }
    let mut bot = factory(state);
    match bot.next_action(state, seat) {
        Some(action) => ServerMsg::Hint {
            label: Some(imperative(&super::undo::action_label(state, &action))),
            card: action_card(&action),
            note: String::new(),
        },
        None => none("the bot has nothing to suggest"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_label_reads_as_an_instruction() {
        assert_eq!(imperative("played Forest"), "play Forest");
        assert_eq!(imperative("declared attackers"), "declare attackers");
        assert_eq!(imperative("cast Lightning Bolt"), "cast Lightning Bolt");
        assert_eq!(imperative("passed priority"), "pass priority");
        assert_eq!(imperative("puttering"), "puttering", "only a whole word");
    }

    #[test]
    fn an_action_names_its_card() {
        let id = CardId(42);
        assert_eq!(action_card(&GameAction::PlayLand(id)), Some(id));
        let cast = GameAction::CastSpell { card_id: id, target: None, additional_targets: vec![], mode: None, x_value: None };
        assert_eq!(action_card(&cast), Some(id));
        assert_eq!(action_card(&GameAction::PassPriority), None);
    }
}
