//! A sacrifice that only renews its own fodder is not a value play.

use crate::card::CardId;
use crate::effect::{ActivatedAbility, Effect};
use crate::game::GameState;

/// The names of the tokens `e` creates, one level through `Seq`.
fn created_token_names(e: &Effect, out: &mut Vec<String>) {
    match e {
        Effect::CreateToken { definition, .. } | Effect::CreateTokenAttacking { definition, .. } => {
            out.push(definition.name.clone())
        }
        Effect::Seq(steps) => steps.iter().for_each(|s| created_token_names(s, out)),
        _ => {}
    }
}

/// True when activating `ab` on `source` would sacrifice nothing but tokens of
/// a kind the ability itself makes — Tooth and Claw feeding on its own
/// Carnivores, which under a token doubler is a net-zero exchange the
/// evaluator can score above passing forever (3,936 activations in one turn,
/// pod seed 10530 game 241). The fodder is the engine's own auto-pick.
pub(super) fn renews_its_own_fodder(state: &GameState, seat: usize, source: CardId, ab: &ActivatedAbility) -> bool {
    let Some((filter, count)) = ab.sac_other_filter.as_ref() else { return false };
    let mut names = Vec::new();
    created_token_names(&ab.effect, &mut names);
    if names.is_empty() {
        return false;
    }
    let candidates: Vec<CardId> = state
        .battlefield
        .iter()
        .filter(|c| c.id != source && c.controller == seat && state.evaluate_requirement_on_card(filter, c, seat))
        .map(|c| c.id)
        .collect();
    let picks = state.auto_pick_lowest_power(&candidates, *count as usize);
    !picks.is_empty()
        && picks.iter().all(|id| {
            state.battlefield_find(*id).is_some_and(|c| c.is_token && names.iter().any(|n| n == c.definition.name))
        })
}

/// True when `seat`'s board is already overkill: at least
/// [`SATURATED_CREATURES`] creatures whose total power is three times the
/// life every live opponent has left. Another token then changes nothing but
/// the board size, and a sink that nets its own resource back loops until
/// the pod's board cap — Whirler Virtuoso under Decoction Module,
/// Panharmonicon and Stridehangar Automaton (3 energy in, 4 back) made 980
/// Thopters in one game (seat 138 pod, seed 138 game 561).
pub(super) fn board_is_saturated(state: &GameState, seat: usize) -> bool {
    let mut count = 0usize;
    let mut power = 0i64;
    for c in state.battlefield.iter().filter(|c| c.controller == seat) {
        if let Some(cp) = state.computed_permanent(c.id).filter(|cp| cp.card_types().contains(&crate::card::CardType::Creature)) {
            count += 1;
            power += i64::from(cp.power.max(0));
        }
    }
    if count < SATURATED_CREATURES {
        return false;
    }
    let life: i64 = state.opponents_of(seat).iter().map(|&o| i64::from(state.players[o].life.max(0))).sum();
    power >= 3 * life
}

/// True when `seat` already controls [`CLUTTERED_PERMANENTS`] permanents. A
/// sacrifice outlet whose value is paid back by triggers — Woe Strider
/// feeding Squirrels to Eloise, Nephalia Sleuth under Chatterfang, a Clue and
/// a new Squirrel per scry — scores above passing every time and loops until
/// the pod's board cap (968 Clues, decks 73-78, seed 20073 game 184). Past
/// this many permanents one more trade changes nothing but the board size.
pub(super) fn board_is_cluttered(state: &GameState, seat: usize) -> bool {
    state.battlefield.iter().filter(|c| c.controller == seat).count() >= CLUTTERED_PERMANENTS
}

/// The permanent count at which [`board_is_cluttered`] fires — far past any
/// duel board, so the 2-player pool never reaches it.
const CLUTTERED_PERMANENTS: usize = 150;

/// The creature count below which [`board_is_saturated`] never fires — far
/// past any duel board, so the 2-player pool never reaches it.
const SATURATED_CREATURES: usize = 60;

#[cfg(test)]
mod tests {
    use crate::catalog;
    use crate::game::types::{GameAction, TurnStep};
    use crate::game::*;
    use crate::server::bot::{EvalWeights, pick_sacrifice_value, pick_token_maker};

    /// Tooth and Claw under Primal Vigor: two Carnivores in, two out — the
    /// pod ran it 3,936 times in one turn (seed 10530, game 241). Bears are
    /// fodder worth trading; the Carnivores it made are not.
    #[test]
    fn a_sacrifice_that_only_renews_its_fodder_is_not_taken() {
        let mut g = multi_player_game(4);
        let me = 1;
        g.add_card_to_battlefield(me, catalog::tooth_and_claw());
        g.add_card_to_battlefield(me, catalog::primal_vigor());
        for _ in 0..2 {
            let id = g.add_card_to_battlefield(me, catalog::grizzly_bears());
            g.clear_sickness(id);
        }
        let tac = g.battlefield.iter().find(|c| c.definition.name == "Tooth and Claw").unwrap().id;
        let ab = g.battlefield_find(tac).unwrap().definition.activated_abilities[0].clone();
        assert!(!super::renews_its_own_fodder(&g, me, tac, &ab), "Bears are not Carnivores");
        g.active_player_idx = me;
        g.step = TurnStep::PreCombatMain;
        g.priority.player_with_priority = me;
        g.perform_action(GameAction::ActivateAbility {
            card_id: tac,
            ability_index: 0,
            target: None,
            additional_targets: vec![],
            x_value: None,
            mode: None,
        })
        .expect("activate");
        drain_stack(&mut g);
        assert!(super::renews_its_own_fodder(&g, me, tac, &ab), "two Carnivores for two Carnivores");
        g.step = TurnStep::PostCombatMain;
        for w in [EvalWeights::default(), EvalWeights::baseline()] {
            assert!(pick_sacrifice_value(&g, me, &w).is_none());
            assert!(pick_token_maker(&g, me, &w).is_none());
        }
    }

    /// Sixty Bears against two opponents at 20 life: 120 power is three
    /// times 40, so the token sink stops (the 980-Thopter pod game, seed 138
    /// game 561); at 59 creatures it does not.
    #[test]
    fn an_overkill_board_stops_minting_tokens() {
        let mut g = multi_player_game(3);
        let me = 0;
        for p in 1..3 {
            g.players[p].life = 20;
        }
        for _ in 0..59 {
            g.add_card_to_battlefield(me, catalog::grizzly_bears());
        }
        assert!(!super::board_is_saturated(&g, me), "59 creatures");
        g.add_card_to_battlefield(me, catalog::grizzly_bears());
        assert!(super::board_is_saturated(&g, me));
        g.players[1].life = 21;
        assert!(!super::board_is_saturated(&g, me), "120 power < 3 x 41");
    }

    /// Woe Strider beside Eloise, Nephalia Sleuth and Chatterfang: every
    /// token it eats pays a Clue and a Squirrel back, so the trade scores above
    /// passing forever — 968 Clues at the pod's board cap (decks 73-78,
    /// seed 20073 game 184). At 150 permanents the outlet is left alone.
    #[test]
    fn a_cluttered_board_stops_feeding_a_sacrifice_outlet() {
        let mut g = multi_player_game(4);
        let me = 1;
        let ws = g.add_card_to_battlefield(me, catalog::woe_strider());
        g.add_card_to_battlefield(me, catalog::eloise_nephalia_sleuth());
        g.add_card_to_battlefield(me, catalog::chatterfang_squirrel_general());
        for _ in 0..3 {
            g.add_token_to_battlefield(me, &crabomination_base::tokens::eldrazi_spawn_token());
        }
        g.clear_sickness(ws);
        g.active_player_idx = me;
        g.step = TurnStep::PostCombatMain;
        g.priority.player_with_priority = me;
        let w = EvalWeights::default();
        let open = pick_sacrifice_value(&g, me, &w);
        assert!(
            matches!(open, Some(GameAction::ActivateAbility { card_id, .. }) if card_id == ws),
            "an ordinary board takes the Clue: {open:?}"
        );
        while g.battlefield.iter().filter(|c| c.controller == me).count() < 150 {
            g.add_card_to_battlefield(me, catalog::forest());
        }
        assert!(super::board_is_cluttered(&g, me));
        assert!(pick_sacrifice_value(&g, me, &w).is_none(), "150 permanents: enough");
    }

    /// A "you may create a token" trigger is declined on an overkill board:
    /// Flourishing Defenses took every Elf Warrior it was offered under
    /// Everlasting Torment until the pod's board cap (seed 17302, game 37).
    #[test]
    fn an_overkill_board_declines_optional_tokens() {
        use crate::server::bot::optional_trigger_beneficial;
        let mut g = multi_player_game(3);
        let me = 0;
        let fd = g.add_card_to_battlefield(me, catalog::flourishing_defenses());
        let ask = "Create an Elf Warrior for each -1/-1 counter?";
        assert!(optional_trigger_beneficial(&g, fd, ask), "an ordinary board takes the Elves");
        for _ in 0..60 {
            g.add_card_to_battlefield(me, catalog::grizzly_bears());
        }
        assert!(!optional_trigger_beneficial(&g, fd, ask), "sixty Bears against 40 life: enough");
    }
}
