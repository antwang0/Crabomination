//! CR 614.1a / 508.1 — Island Sanctuary: "If you would draw a card during
//! your draw step, instead you may skip that draw. If you do, until your next
//! turn, you can't be attacked except by creatures with flying and/or
//! islandwalk." The static is
//! `StaticEffect::ControllerMaySkipDrawStepDrawForAttackShield`; a taken skip
//! stamps `Player.attackable_only_by_until_next_turn`, which
//! `declare_attackers` reads and the player's next turn start clears.

use super::GameState;
use crate::card::{CardId, SelectionRequirement};
use crate::effect::StaticEffect;
use crate::game::Target;

impl GameState {
    /// Offer `p`'s attack-shield draw replacements for one turn-based draw.
    /// True when one was taken (the draw is skipped and the shield is up).
    pub(crate) fn offer_draw_skip_shield(&mut self, p: usize) -> bool {
        let offers: Vec<(CardId, &'static str, SelectionRequirement)> = self
            .battlefield
            .iter()
            .filter(|c| c.controller == p)
            .flat_map(|c| {
                c.definition.static_abilities.iter().filter_map(move |sa| match &sa.effect {
                    StaticEffect::ControllerMaySkipDrawStepDrawForAttackShield { attackers } => {
                        Some((c.id, c.definition.name, attackers.clone()))
                    }
                    _ => None,
                })
            })
            .collect();
        for (id, name, attackers) in offers {
            let yes = if matches!(self.decider.kind(), crate::decision::DeciderKind::Auto) {
                self.shield_worth_a_draw(p, id, &attackers)
            } else {
                matches!(
                    self.decider.decide(&crate::decision::Decision::OptionalTrigger {
                        source: id,
                        description: format!(
                            "Skip this draw so only flying or islandwalk creatures can attack you until your next turn ({name})?"
                        ),
                        kind: crate::decision::OptionalKind::Neutral,
                    }),
                    crate::decision::DecisionAnswer::Bool(true)
                )
            };
            if yes {
                self.players[p].attackable_only_by_until_next_turn = Some(Box::new(attackers));
                return true;
            }
        }
        false
    }

    /// The auto policy: skip on an empty library (the draw would lose), or
    /// when the opponents' creatures the shield would turn away carry at
    /// least three power or a quarter of `p`'s life — a card is worth less
    /// than a turn cycle of their ground damage.
    fn shield_worth_a_draw(&self, p: usize, source: CardId, attackers: &SelectionRequirement) -> bool {
        if self.players[p].library.is_empty() {
            return true;
        }
        let stopped: i32 = self
            .battlefield
            .iter()
            .filter(|c| c.controller != p && !self.same_team(p, c.controller))
            .filter(|c| self.computed_is_creature(c))
            .filter(|c| !self.evaluate_requirement_static(attackers, &Target::Permanent(c.id), p, Some(source)))
            .map(|c| self.computed_permanent(c.id).map_or(0, |cp| cp.power.max(0)))
            .sum();
        stopped >= 3 || stopped * 4 >= self.players[p].life
    }

    /// CR 508.1 — `attacker` may not attack `defender` under their shield.
    pub(crate) fn attacker_barred_by_shield(&self, attacker: CardId, defender: usize) -> bool {
        self.players[defender]
            .attackable_only_by_until_next_turn
            .as_deref()
            .is_some_and(|f| !self.evaluate_requirement_static(f, &Target::Permanent(attacker), defender, None))
    }
}
