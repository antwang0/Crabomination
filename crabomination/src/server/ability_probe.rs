//! The ability probe: `CRAB_PROBE=names.txt cargo nextest run -p crabomination
//! probe_abilities --no-capture` puts each named card, untapped and unsick,
//! onto seat 0's battlefield in a three-seat Commander game — fifteen basic
//! lands, a Grizzly Bears commander, a Hill Giant, Glorious Anthem and Sol
//! Ring across the table, stocked libraries and graveyards — and prints
//! whether the bot's next action at its post-combat main and at an
//! opponent's end step activates it. Feed it the
//! `never activated` names from a `--card-census` run: a card the census never
//! saw used but the probe does is a coverage gap, and one the probe skips too
//! is a bot gap. A planeswalker gets `CRAB_PROBE_LOYALTY` loyalty on top of
//! its printed number (default 4, enough for most ultimates; 0 shows the
//! everyday pick). Without `CRAB_PROBE` the test does nothing.

use crate::game::types::{GameAction, TurnStep};
use crate::server::bot::{Bot, HeuristicBot};

#[test]
fn probe_abilities() {
    let Ok(list) = std::env::var("CRAB_PROBE") else {
        return;
    };
    let names = std::fs::read_to_string(list).unwrap();
    for name in names.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Some(def) = crate::card_registry::lookup_by_name(name) else {
            eprintln!("PROBE {name}: missing");
            continue;
        };
        let mut hits = Vec::new();
        for step in [TurnStep::PostCombatMain, TurnStep::End] {
            let mut g = crate::game::multi_player_game(3);
            g.active_player_idx = if step == TurnStep::End { 1 } else { 0 };
            g.step = step;
            g.priority.player_with_priority = 0;
            let m = g.add_card_to_battlefield(0, def.clone());
            g.clear_sickness(m);
            if let Some(c) = g.battlefield_find_mut(m)
                && c.definition.is_planeswalker()
            {
                c.add_counters(crate::card::CounterType::Loyalty, std::env::var("CRAB_PROBE_LOYALTY").ok().and_then(|v| v.parse().ok()).unwrap_or(4));
            }
            for f in [
                crate::catalog::forest,
                crate::catalog::island,
                crate::catalog::plains,
                crate::catalog::swamp,
                crate::catalog::mountain,
            ] {
                for _ in 0..3 {
                    let l = g.add_card_to_battlefield(0, f());
                    g.clear_sickness(l);
                }
            }
            let b = g.add_card_to_battlefield(0, crate::catalog::grizzly_bears());
            g.clear_sickness(b);
            g.players[0].commanders.push(b);
            g.add_card_to_battlefield(1, crate::catalog::hill_giant());
            g.add_card_to_battlefield(1, crate::catalog::glorious_anthem());
            g.add_card_to_battlefield(1, crate::catalog::sol_ring());
            for _ in 0..3 {
                g.add_card_to_graveyard(0, crate::catalog::grizzly_bears());
                g.add_card_to_graveyard(1, crate::catalog::grizzly_bears());
            }
            for seat in 0..3 {
                for _ in 0..20 {
                    g.add_card_to_library(seat, crate::catalog::grizzly_bears());
                }
            }
            let mut bot = HeuristicBot::new();
            let a = bot.next_action(&g, 0);
            let used = match &a {
                Some(GameAction::ActivateAbility { card_id, ability_index, .. }) if *card_id == m => {
                    Some(format!("USED #{ability_index}"))
                }
                Some(GameAction::ActivateLoyaltyAbility { card_id, ability_index, .. }) if *card_id == m => {
                    Some(format!("USED loyalty #{ability_index}"))
                }
                _ => None,
            };
            hits.push(format!("{step:?}={}", used.unwrap_or_else(|| format!("{a:?}").chars().take(40).collect())));
        }
        eprintln!("PROBE {name}: {}", hits.join(" | "));
    }
}

