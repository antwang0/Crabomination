//! Team grouping for multiplayer formats.
//!
//! Every seat is on exactly one team. In free-for-all each team is a
//! singleton (`team_of(seat) == TeamId(seat)`), so "opponents" is every other
//! seat; in team formats (Two-Headed Giant) the team is the unit for "target
//! / each opponent". `GameState::new` makes singleton teams; team formats
//! call `assign_teams` afterwards.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TeamId(pub usize);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: TeamId,
    /// Seat indices of the players on this team.
    pub members: Vec<usize>,
    /// Shared life pool: `Some(n)` when members share one total (Two-Headed
    /// Giant, CR 810.8), `None` for per-player `Player.life` (the default).
    ///
    /// When set, life mutations route here and `Player.life` is ignored;
    /// `effective_life(seat)` covers both. Old snapshots load as `None`.
    #[serde(default)]
    pub shared_life: Option<i32>,
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
