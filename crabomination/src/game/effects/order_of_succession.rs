//! Order of Succession — "choose left or right; each player chooses a creature
//! controlled by the next player in that direction and gains control of it"
//! (CR 101.4's seating order; left is the next seat in turn order).

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::Effect;
use crate::game::GameState;

impl GameState {
    /// `Effect::EachPlayerTakesCreatureOfNext`. The caster chooses the
    /// direction, then each player in APNAP order chooses a creature of the
    /// next living player (CR 608.2d). Headless: every chooser takes the most
    /// valuable one and the caster the direction netting it the most (left on
    /// a tie). The control changes happen together, after every choice.
    pub(super) fn each_player_takes_creature_of_next(&mut self, ctx: &EffectContext, effect: &Effect) {
        let n = self.players.len();
        let alive: Vec<usize> = (0..n).filter(|&p| self.players[p].is_alive()).collect();
        if alive.len() < 2 {
            return;
        }
        // Each creature with its controller, most valuable first.
        let mut creatures: Vec<(CardId, usize, i64)> = self
            .battlefield
            .iter()
            .filter(|c| self.computed_is_creature(c))
            .map(|c| {
                let (p, t) = self.computed_permanent(c.id).map_or((0, 0), |cp| (cp.power, cp.toughness));
                let v = i64::from(c.definition.cost.cmc()) * 10_000 + i64::from(p.max(0)) * 100 + i64::from(t.max(0));
                (c.id, c.controller, v)
            })
            .collect();
        creatures.sort_by_key(|&(id, _, v)| (std::cmp::Reverse(v), id));
        let best_of = |seat: usize| creatures.iter().find(|c| c.1 == seat).map(|c| (c.0, c.1, c.2));
        let me = ctx.controller;
        let net = |step: usize| -> i64 {
            alive
                .iter()
                .enumerate()
                .filter_map(|(i, &p)| best_of(alive[(i + step) % alive.len()]).map(|c| (p, c)))
                .map(|(p, (_, owner, v))| if p == me { v } else if owner == me { -v } else { 0 })
                .sum()
        };
        let right_first = net(alive.len() - 1) > net(1);
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let options: Vec<String> =
            if right_first { vec!["Right".into(), "Left".into()] } else { vec!["Left".into(), "Right".into()] };
        let Some(i) = self.ask_seat_option(&mut cursor, me, "Choose left or right".into(), source, options, effect)
        else {
            return;
        };
        let step = if (i == 0) == right_first { alive.len() - 1 } else { 1 };
        let mut moves: Vec<(usize, CardId)> = Vec::new();
        for p in self.apnap_sort(alive.clone()) {
            let k = alive.iter().position(|&q| q == p).unwrap_or(0);
            let next = alive[(k + step) % alive.len()];
            let theirs: Vec<CardId> = creatures.iter().filter(|c| c.1 == next).map(|c| c.0).collect();
            let Some(&best) = theirs.first() else { continue };
            let candidates: Vec<(CardId, String)> = theirs
                .iter()
                .filter_map(|&id| self.battlefield_find(id).map(|c| (id, c.definition.name.to_string())))
                .collect();
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Choose a creature to gain control of".into(),
                source,
                candidates,
                1,
                1,
                PickValue::Gain,
                effect,
                vec![best],
            ) else {
                return;
            };
            moves.push((p, picked.first().copied().unwrap_or(best)));
        }
        self.clear_answer_log();
        for (p, c) in moves {
            self.change_control(c, p);
        }
    }
}
