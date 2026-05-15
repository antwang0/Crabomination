//! Team grouping for multiplayer formats.
//!
//! Every seat belongs to exactly one team. In free-for-all formats each
//! team is a singleton (`team_of(seat) == TeamId(seat)`), so "opponents of
//! X" reduces to "every other seat". In team formats (Two-Headed Giant,
//! Three-Player Star) multiple seats share a team and that team becomes
//! the unit for "target opponent" / "each opponent" semantics.
//!
//! `GameState::new` populates `teams` with one singleton team per seat
//! automatically. Team formats call `assign_teams` to re-partition seats
//! after construction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    /// Seat indices of the players on this team.
    pub members: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeamError {
    /// A seat appeared in more than one partition.
    DuplicateSeat(usize),
    /// A seat number is past the end of the player list.
    UnknownSeat { seat: usize, num_players: usize },
    /// Not every seat was assigned to a team.
    MissingSeat(usize),
    /// A partition was empty (a team must have at least one member).
    EmptyTeam(usize),
}

impl std::fmt::Display for TeamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamError::DuplicateSeat(s) => write!(f, "seat {s} appears in more than one team"),
            TeamError::UnknownSeat { seat, num_players } => {
                write!(f, "seat {seat} is out of range (only {num_players} players)")
            }
            TeamError::MissingSeat(s) => write!(f, "seat {s} was not assigned to any team"),
            TeamError::EmptyTeam(i) => write!(f, "team index {i} has no members"),
        }
    }
}

impl std::error::Error for TeamError {}
