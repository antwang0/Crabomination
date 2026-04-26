//! Authoritative match server.
//!
//! A match owns a [`GameState`] and talks to N seats. Each seat is either a
//! network-style [`SeatChannel`] (human client on the other end, possibly
//! over TCP) or an in-process [`Bot`] that reads state directly. The actor
//! validates every action against the seat that sent it, forwards it to the
//! engine, then broadcasts a per-seat [`ClientView`] plus the generated
//! events to every human seat.
//!
//! The actor is synchronous — it blocks on a merged receiver and runs on a
//! single thread. Transports (in-process channel bridge for singleplayer,
//! TCP for networked play) wrap this with their own I/O.

use std::sync::mpsc;
use std::thread;

use crate::game::{GameAction, GameState, TurnStep};
use crate::net::{ClientMsg, GameEventWire, ServerMsg};

pub mod bot;
pub mod tcp;
pub mod view;

pub use bot::{Bot, RandomBot};
pub use tcp::{tcp_client, tcp_seat};
pub use view::project;

/// A safety limit on how many actions a bot (or chain of bots) can take
/// between human inputs. The loop polls bots to a fixed point; if that fixed
/// point never arrives, something is wrong and we'd rather panic than spin.
const BOT_TICK_BUDGET: usize = 10_000;

/// The server-side end of a seat connection. The `tx` is where the server
/// sends [`ServerMsg`]s to this seat's client; `rx` is where the server
/// receives [`ClientMsg`]s from it.
pub struct SeatChannel {
    pub tx: mpsc::Sender<ServerMsg>,
    pub rx: mpsc::Receiver<ClientMsg>,
}

/// The client-side end of a seat connection. Mirror of [`SeatChannel`].
pub struct ClientChannel {
    pub tx: mpsc::Sender<ClientMsg>,
    pub rx: mpsc::Receiver<ServerMsg>,
}

/// Create a linked pair of channels — one end for the server (a seat), one
/// for the client that occupies that seat.
pub fn seat_pair() -> (SeatChannel, ClientChannel) {
    let (c_tx, s_rx) = mpsc::channel(); // client → server
    let (s_tx, c_rx) = mpsc::channel(); // server → client
    (
        SeatChannel { tx: s_tx, rx: s_rx },
        ClientChannel { tx: c_tx, rx: c_rx },
    )
}

/// How a seat is being driven for this match.
pub enum SeatOccupant {
    /// A remote (or in-process-wrapped) human client reached over a channel.
    Human(SeatChannel),
    /// An in-process bot that reads authoritative state directly.
    Bot(Box<dyn Bot>),
}

/// Run a match to completion on the current thread. Returns when the game
/// ends, all human seat channels have been dropped, or — if every seat is a
/// bot — when the game ends.
pub fn run_match(mut state: GameState, occupants: Vec<SeatOccupant>) {
    let n = occupants.len();
    assert_eq!(n, state.players.len(), "occupant count must match player count");

    // Only run the pre-game mulligan when the state is genuinely at game
    // start (Untap of turn 1). Tests/fixtures that hand-craft a state mid-game
    // bypass this; otherwise PlayLand etc. would be rejected with
    // `DecisionPending` because the mulligan installs a pending decision.
    if state.step == TurnStep::Untap && state.turn_number == 1 {
        state.start_mulligan_phase();
    }

    let (merged_tx, merged_rx) = mpsc::channel::<(usize, ClientMsg)>();
    let mut seat_tx: Vec<Option<mpsc::Sender<ServerMsg>>> = Vec::with_capacity(n);
    let mut bots: Vec<Option<Box<dyn Bot>>> = Vec::with_capacity(n);
    let mut human_seat_count = 0;

    for (i, occ) in occupants.into_iter().enumerate() {
        match occ {
            SeatOccupant::Human(seat) => {
                let _ = seat.tx.send(ServerMsg::YourSeat(i));
                let _ = seat.tx.send(ServerMsg::MatchStarted);
                let _ = seat.tx.send(ServerMsg::View(view::project(&state, i)));
                let forward_tx = merged_tx.clone();
                let rx = seat.rx;
                thread::spawn(move || {
                    while let Ok(msg) = rx.recv() {
                        if forward_tx.send((i, msg)).is_err() {
                            break;
                        }
                    }
                });
                seat_tx.push(Some(seat.tx));
                bots.push(None);
                human_seat_count += 1;
            }
            SeatOccupant::Bot(b) => {
                seat_tx.push(None);
                bots.push(Some(b));
            }
        }
    }
    drop(merged_tx);

    loop {
        if drive_bots(&mut state, &mut bots, &seat_tx) {
            broadcast_match_over(&state, &seat_tx);
            return;
        }

        if human_seat_count == 0 {
            return;
        }

        let Ok((seat, msg)) = merged_rx.recv() else {
            return;
        };

        match msg {
            ClientMsg::JoinMatch { .. } => {}
            ClientMsg::SubmitAction(action) => {
                let _ = handle_action(&mut state, seat, action, &seat_tx);
                if state.is_game_over() {
                    broadcast_match_over(&state, &seat_tx);
                    return;
                }
            }
        }
    }
}

/// Poll every bot seat to a fixed point: each pass asks every bot whether it
/// wants to act, and we repeat until a full pass produces no actions.
/// Returns `true` if the game ended during bot play.
fn drive_bots(
    state: &mut GameState,
    bots: &mut [Option<Box<dyn Bot>>],
    seat_tx: &[Option<mpsc::Sender<ServerMsg>>],
) -> bool {
    let mut budget = BOT_TICK_BUDGET;
    loop {
        let mut any_acted = false;
        for (seat, slot) in bots.iter_mut().enumerate() {
            let Some(bot) = slot.as_mut() else { continue };
            let Some(action) = bot.next_action(state, seat) else {
                continue;
            };
            // Only count the action if it actually changed state. A rejected
            // action (wrong priority, illegal move, etc.) must not count as
            // progress — otherwise the loop burns the entire budget retrying
            // the same failing action.
            if handle_action(state, seat, action, seat_tx) {
                if state.is_game_over() {
                    return true;
                }
                any_acted = true;
            }
        }
        if !any_acted {
            return false;
        }
        budget = budget.checked_sub(1).expect("bot loop exceeded BOT_TICK_BUDGET");
    }
}

fn broadcast_match_over(state: &GameState, seat_tx: &[Option<mpsc::Sender<ServerMsg>>]) {
    let winner = state.game_over.unwrap_or(None);
    for tx in seat_tx.iter().flatten() {
        let _ = tx.send(ServerMsg::MatchOver { winner });
    }
}

/// Apply one action and broadcast results. Returns `true` if the action was
/// accepted (state changed), `false` if it was rejected.
fn handle_action(
    state: &mut GameState,
    seat: usize,
    action: GameAction,
    seat_tx: &[Option<mpsc::Sender<ServerMsg>>],
) -> bool {
    let expected = expected_actor(state, &action);
    if seat != expected {
        let err = format!("seat {seat} may not act now (expected seat {expected})");
        report_error(seat, &err, seat_tx);
        return false;
    }
    match state.perform_action(action) {
        Ok(events) => {
            let wire_events: Vec<GameEventWire> = events.iter().map(Into::into).collect();
            for (i, maybe_tx) in seat_tx.iter().enumerate() {
                if let Some(tx) = maybe_tx {
                    let _ = tx.send(ServerMsg::Events(wire_events.clone()));
                    let _ = tx.send(ServerMsg::View(view::project(state, i)));
                }
            }
            true
        }
        Err(e) => {
            report_error(seat, &e.to_string(), seat_tx);
            false
        }
    }
}

fn report_error(seat: usize, err: &str, seat_tx: &[Option<mpsc::Sender<ServerMsg>>]) {
    if let Some(tx) = seat_tx[seat].as_ref() {
        let _ = tx.send(ServerMsg::ActionError(err.to_string()));
    } else {
        // Bot seat — no channel. Surface for debugging; bots shouldn't be
        // submitting illegal actions in normal play.
        eprintln!("server: bot seat {seat} action rejected: {err}");
    }
}

/// Which seat the engine will attribute the next action to. Used by the
/// server to reject actions submitted by the wrong client.
fn expected_actor(state: &GameState, _action: &GameAction) -> usize {
    if let Some(pd) = &state.pending_decision {
        return pd.acting_player();
    }
    // The actor is whoever currently holds priority (including each
    // defender's separate window during DeclareBlockers in multiplayer).
    state.player_with_priority()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardId;
    use crate::catalog;
    use crate::game::{GameAction, TurnStep};
    use crate::net::{ClientMsg, ServerMsg};
    use crate::player::Player;
    use crate::server::bot::RandomBot;

    fn two_player_game() -> GameState {
        let mut state = GameState::new(vec![Player::new(0, "P0"), Player::new(1, "P1")]);
        // Start in main phase so PlayLand is legal.
        state.step = TurnStep::PreCombatMain;
        state
    }

    fn drain_initial(seat: &ClientChannel) {
        // Discard YourSeat, MatchStarted, and initial View.
        for _ in 0..3 {
            let _ = seat.rx.recv();
        }
    }

    #[test]
    fn wrong_seat_action_rejected() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Seat 1 tries to play seat 0's land — must be rejected.
        c1.tx.send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        // Give the actor a moment, then expect an ActionError back on c1.
        let reply = c1.rx.recv().unwrap();
        match reply {
            ServerMsg::ActionError(_) => {}
            other => panic!("expected ActionError, got {:?}", other),
        }

        // Drop channels to end the match.
        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    #[test]
    fn correct_seat_action_broadcasts_view() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        c0.tx.send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        // Both seats should receive Events + View.
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::Events(_)));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::View(_)));
        assert!(matches!(c1.rx.recv().unwrap(), ServerMsg::Events(_)));
        assert!(matches!(c1.rx.recv().unwrap(), ServerMsg::View(_)));

        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    #[test]
    fn unknown_card_error_surfaces_as_action_error() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let (s1, c1) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(state, vec![SeatOccupant::Human(s0), SeatOccupant::Human(s1)])
        });

        drain_initial(&c0);
        drain_initial(&c1);

        // Reference a card that doesn't exist.
        c0.tx.send(ClientMsg::SubmitAction(GameAction::PlayLand(CardId(999))))
            .unwrap();

        match c0.rx.recv().unwrap() {
            ServerMsg::ActionError(_) => {}
            other => panic!("expected ActionError, got {:?}", other),
        }

        drop(c0);
        drop(c1);
        handle.join().unwrap();
    }

    #[test]
    fn human_vs_bot_initial_handshake() {
        let state = two_player_game();
        let (s0, c0) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Human(s0),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            )
        });

        // Human (seat 0) receives the standard three opening messages.
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::YourSeat(0)));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::MatchStarted));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::View(_)));

        // Dropping the human channel terminates the match cleanly.
        drop(c0);
        handle.join().unwrap();
    }

    #[test]
    fn human_vs_bot_action_broadcasts_to_human() {
        let mut state = two_player_game();
        let card_id = state.add_card_to_hand(0, catalog::plains());

        let (s0, c0) = seat_pair();
        let handle = thread::spawn(move || {
            run_match(
                state,
                vec![
                    SeatOccupant::Human(s0),
                    SeatOccupant::Bot(Box::new(RandomBot::new())),
                ],
            )
        });

        drain_initial(&c0);

        // Human plays a land — the human seat should still receive Events + View.
        c0.tx
            .send(ClientMsg::SubmitAction(GameAction::PlayLand(card_id)))
            .unwrap();

        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::Events(_)));
        assert!(matches!(c0.rx.recv().unwrap(), ServerMsg::View(_)));

        drop(c0);
        handle.join().unwrap();
    }
}
