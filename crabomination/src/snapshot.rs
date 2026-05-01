//! Serializable snapshot of `GameState` for round-trip debug replay.
//!
//! `GameState` itself now `derive`s `Serialize`/`Deserialize`, so the
//! preferred path for full-fidelity replay is `serde_json::to_string(&state)`
//! / `from_str` directly. This module's `GameSnapshot` predates that
//! work and remains as a smaller, schema-stable wrapper for the
//! "in-game export" workflow — it captures only the user-facing engine
//! state (life, board, hands, stack spells, turn/step) so the resulting
//! JSON file is more readable and resilient to engine refactors than
//! a raw `GameState` dump. Both formats coexist:
//!
//! - **`GameSnapshot`** — schema-stable, slightly lossy (drops
//!   trigger-stack items, transient fields). Use when the export
//!   should round-trip across engine versions.
//! - **Direct `GameState`** — fully lossless including triggers,
//!   delayed triggers, continuous effects, pending decision, decider
//!   state. Use when bit-exact replay matters.
//!
//! # `GameSnapshot` fidelity
//!
//! - **Lossless**: player life, mana pool, library/hand/graveyard
//!   contents, battlefield state (tap, damage, counters, P/T bonuses,
//!   token-ness, attached_to), exile, turn/step/priority, generic
//!   per-game counters (`spells_cast_this_turn`, `next_id`), attack /
//!   block declarations, Spell stack items.
//! - **Best-effort**: `Trigger` stack items carry a `Box<Effect>`; the
//!   snapshot path drops them and reports `dropped_triggers`. The full
//!   `GameState` round-trip preserves them.
//! - **Reset on load**: `pending_decision`, `suspend_signal`,
//!   `delayed_triggers`, `continuous_effects` (rebuilt from static
//!   abilities of permanents on load).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::card::{
    CardDefinition, CardId, CardInstance, CounterType, CreatureType,
};
use crate::game::{
    Attack, AttackTarget, GameState, PriorityState, StackItem, Target, TurnStep,
};
use crate::mana::ManaPool;
use crate::player::{Player, PlayerId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub players: Vec<PlayerSnapshot>,
    pub battlefield: Vec<CardSnapshot>,
    pub exile: Vec<CardSnapshot>,
    pub stack: Vec<StackItemSnapshot>,
    pub step: TurnStep,
    pub active_player_idx: usize,
    pub turn_number: u32,
    pub game_over: Option<Option<usize>>,
    pub priority: PriorityState,
    pub spells_cast_this_turn: u32,
    pub next_id: u32,
    pub attacking: Vec<AttackSnapshot>,
    pub block_map: Vec<(CardId, CardId)>,
    pub blockers_declared: bool,
    pub skip_first_draw: bool,
    /// How many `StackItem::Trigger` entries were on the stack at
    /// snapshot time. Triggers carry a `Box<Effect>` we don't serialize,
    /// so they're dropped; this lets the load path warn the user
    /// instead of silently mismatching the original board.
    pub dropped_triggers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub seat: usize,
    pub name: String,
    pub life: i32,
    pub mana_pool: ManaPool,
    pub library: Vec<CardSnapshot>,
    pub hand: Vec<CardSnapshot>,
    pub graveyard: Vec<CardSnapshot>,
    pub lands_played_this_turn: u32,
    pub spells_cast_this_turn: u32,
    pub first_spell_tax_charges: u32,
    pub sorceries_as_flash: bool,
    pub poison_counters: u32,
    pub eliminated: bool,
    pub wants_ui: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSnapshot {
    pub id: CardId,
    /// Card name; resolved through [`crate::catalog::lookup_by_name`] on
    /// load. If a token whose definition isn't in the standard catalog
    /// (Clue / Treasure / Food / Blood handle the common cases) is
    /// snapshotted, restore will fail with `LoadError::UnknownCard`.
    pub name: String,
    pub owner: usize,
    pub controller: usize,
    pub tapped: bool,
    pub damage: u32,
    pub summoning_sick: bool,
    pub power_bonus: i32,
    pub toughness_bonus: i32,
    pub counters: Vec<(CounterType, u32)>,
    pub attached_to: Option<CardId>,
    pub kicked: bool,
    pub face_down: bool,
    pub is_token: bool,
    pub used_loyalty_ability_this_turn: bool,
    pub evoked: bool,
    pub cast_from_hand: bool,
    pub chosen_creature_type: Option<CreatureType>,
    #[serde(default)]
    pub once_per_turn_used: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackItemSnapshot {
    pub card: CardSnapshot,
    pub caster: usize,
    pub target: Option<Target>,
    pub mode: Option<usize>,
    pub x_value: u32,
    pub converged_value: u32,
    pub uncounterable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackSnapshot {
    pub attacker: CardId,
    pub target: AttackTarget,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("unknown card name: {0:?}")]
    UnknownCard(String),
}

// ── Capture (GameState → GameSnapshot) ────────────────────────────────────────

impl GameSnapshot {
    /// Take a snapshot of `state`. Trigger stack items are dropped
    /// (see module docs); their count surfaces in `dropped_triggers`.
    pub fn capture(state: &GameState) -> Self {
        let mut dropped_triggers = 0usize;
        let stack = state
            .stack
            .iter()
            .filter_map(|item| match item {
                StackItem::Spell {
                    card,
                    caster,
                    target,
                    mode,
                    x_value,
                    converged_value,
                    uncounterable,
                } => Some(StackItemSnapshot {
                    card: card_snap(card),
                    caster: *caster,
                    target: target.clone(),
                    mode: *mode,
                    x_value: *x_value,
                    converged_value: *converged_value,
                    uncounterable: *uncounterable,
                }),
                StackItem::Trigger { .. } => {
                    dropped_triggers += 1;
                    None
                }
            })
            .collect();

        Self {
            players: state.players.iter().map(player_snap).collect(),
            battlefield: state.battlefield.iter().map(card_snap).collect(),
            exile: state.exile.iter().map(card_snap).collect(),
            stack,
            step: state.step,
            active_player_idx: state.active_player_idx,
            turn_number: state.turn_number,
            game_over: state.game_over,
            priority: state.priority.clone(),
            spells_cast_this_turn: state.spells_cast_this_turn,
            next_id: state.peek_next_id(),
            attacking: state
                .attacking()
                .iter()
                .map(|a| AttackSnapshot {
                    attacker: a.attacker,
                    target: a.target,
                })
                .collect(),
            block_map: state.block_map().iter().map(|(b, a)| (*b, *a)).collect(),
            blockers_declared: state.blockers_declared(),
            skip_first_draw: state.skip_first_draw(),
            dropped_triggers,
        }
    }
}

fn card_snap(c: &CardInstance) -> CardSnapshot {
    CardSnapshot {
        id: c.id,
        name: c.definition.name.to_string(),
        owner: c.owner,
        controller: c.controller,
        tapped: c.tapped,
        damage: c.damage,
        summoning_sick: c.summoning_sick,
        power_bonus: c.power_bonus,
        toughness_bonus: c.toughness_bonus,
        counters: c.counters.iter().map(|(k, v)| (*k, *v)).collect(),
        attached_to: c.attached_to,
        kicked: c.kicked,
        face_down: c.face_down,
        is_token: c.is_token,
        used_loyalty_ability_this_turn: c.used_loyalty_ability_this_turn,
        evoked: c.evoked,
        cast_from_hand: c.cast_from_hand,
        chosen_creature_type: c.chosen_creature_type,
        once_per_turn_used: c.once_per_turn_used.clone(),
    }
}

fn player_snap(p: &Player) -> PlayerSnapshot {
    PlayerSnapshot {
        seat: p.id.0,
        name: p.name.clone(),
        life: p.life,
        mana_pool: p.mana_pool.clone(),
        library: p.library.iter().map(card_snap).collect(),
        hand: p.hand.iter().map(card_snap).collect(),
        graveyard: p.graveyard.iter().map(card_snap).collect(),
        lands_played_this_turn: p.lands_played_this_turn,
        spells_cast_this_turn: p.spells_cast_this_turn,
        first_spell_tax_charges: p.first_spell_tax_charges,
        sorceries_as_flash: p.sorceries_as_flash,
        poison_counters: p.poison_counters,
        eliminated: p.eliminated,
        wants_ui: p.wants_ui,
    }
}

// ── Restore (GameSnapshot → GameState) ────────────────────────────────────────

impl GameSnapshot {
    /// Reconstruct a playable `GameState` from a snapshot. Resolves card
    /// names through [`crate::catalog::lookup_by_name`]; returns
    /// `LoadError::UnknownCard` if any card name doesn't resolve.
    ///
    /// On success, the resulting state has:
    /// - `decider = AutoDecider` (snapshot doesn't preserve scripted deciders)
    /// - `pending_decision = None` (would need a scripted answer to fire)
    /// - `delayed_triggers = vec![]`, `continuous_effects = vec![]`
    ///   (the latter are reapplied by the layer system on next access)
    pub fn restore(self) -> Result<GameState, LoadError> {
        let GameSnapshot {
            players,
            battlefield,
            exile,
            stack,
            step,
            active_player_idx,
            turn_number,
            game_over,
            priority,
            spells_cast_this_turn,
            next_id,
            attacking,
            block_map,
            blockers_declared,
            skip_first_draw,
            dropped_triggers: _,
        } = self;

        let mut restored_players = Vec::with_capacity(players.len());
        for ps in players {
            restored_players.push(restore_player(ps)?);
        }

        let mut state = GameState::new(restored_players);
        state.battlefield = battlefield
            .into_iter()
            .map(restore_card)
            .collect::<Result<Vec<_>, _>>()?;
        state.exile = exile
            .into_iter()
            .map(restore_card)
            .collect::<Result<Vec<_>, _>>()?;

        let mut restored_stack = Vec::with_capacity(stack.len());
        for s in stack {
            restored_stack.push(StackItem::Spell {
                card: Box::new(restore_card(s.card)?),
                caster: s.caster,
                target: s.target,
                mode: s.mode,
                x_value: s.x_value,
                converged_value: s.converged_value,
                uncounterable: s.uncounterable,
            });
        }
        state.stack = restored_stack;

        state.step = step;
        state.active_player_idx = active_player_idx;
        state.turn_number = turn_number;
        state.game_over = game_over;
        state.priority = priority;
        state.spells_cast_this_turn = spells_cast_this_turn;
        state.set_next_id(next_id);
        state.set_attacking(
            attacking
                .into_iter()
                .map(|a| Attack {
                    attacker: a.attacker,
                    target: a.target,
                })
                .collect(),
        );
        let bm: HashMap<CardId, CardId> = block_map.into_iter().collect();
        state.set_block_map(bm);
        state.set_blockers_declared(blockers_declared);
        state.set_skip_first_draw(skip_first_draw);
        Ok(state)
    }
}

fn restore_player(ps: PlayerSnapshot) -> Result<Player, LoadError> {
    let mut p = Player::new(ps.seat, ps.name);
    p.id = PlayerId(ps.seat);
    p.life = ps.life;
    p.mana_pool = ps.mana_pool;
    p.library = ps
        .library
        .into_iter()
        .map(restore_card)
        .collect::<Result<Vec<_>, _>>()?;
    p.hand = ps
        .hand
        .into_iter()
        .map(restore_card)
        .collect::<Result<Vec<_>, _>>()?;
    p.graveyard = ps
        .graveyard
        .into_iter()
        .map(restore_card)
        .collect::<Result<Vec<_>, _>>()?;
    p.lands_played_this_turn = ps.lands_played_this_turn;
    p.spells_cast_this_turn = ps.spells_cast_this_turn;
    p.first_spell_tax_charges = ps.first_spell_tax_charges;
    p.sorceries_as_flash = ps.sorceries_as_flash;
    p.poison_counters = ps.poison_counters;
    p.eliminated = ps.eliminated;
    p.wants_ui = ps.wants_ui;
    Ok(p)
}

fn restore_card(cs: CardSnapshot) -> Result<CardInstance, LoadError> {
    let def: CardDefinition = crate::catalog::lookup_by_name(&cs.name)
        .ok_or_else(|| LoadError::UnknownCard(cs.name.clone()))?;
    let mut c = CardInstance::new(cs.id, def, cs.owner);
    c.controller = cs.controller;
    c.tapped = cs.tapped;
    c.damage = cs.damage;
    c.summoning_sick = cs.summoning_sick;
    c.power_bonus = cs.power_bonus;
    c.toughness_bonus = cs.toughness_bonus;
    c.counters = cs.counters.into_iter().collect();
    c.attached_to = cs.attached_to;
    c.kicked = cs.kicked;
    c.face_down = cs.face_down;
    c.is_token = cs.is_token;
    c.used_loyalty_ability_this_turn = cs.used_loyalty_ability_this_turn;
    c.evoked = cs.evoked;
    c.cast_from_hand = cs.cast_from_hand;
    c.chosen_creature_type = cs.chosen_creature_type;
    c.once_per_turn_used = cs.once_per_turn_used;
    Ok(c)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog;
    use crate::game::{GameAction, TurnStep};
    use crate::player::Player;

    fn two_player_game() -> GameState {
        let mut g = GameState::new(vec![Player::new(0, "Alice"), Player::new(1, "Bob")]);
        g.step = TurnStep::PreCombatMain;
        g
    }

    #[test]
    fn snapshot_round_trips_basic_fields() {
        let mut g = two_player_game();
        g.players[0].life = 17;
        g.players[1].life = 12;
        g.players[0].mana_pool.add(crate::mana::Color::Green, 2);
        let bear = g.add_card_to_battlefield(0, catalog::grizzly_bears());
        if let Some(c) = g.battlefield_find_mut(bear) {
            c.tapped = true;
            c.damage = 1;
            c.power_bonus = 2;
        }
        g.add_card_to_hand(1, catalog::lightning_bolt());

        let snap = GameSnapshot::capture(&g);
        let restored = snap.clone().restore().expect("restore should succeed");

        assert_eq!(restored.players[0].life, 17);
        assert_eq!(restored.players[1].life, 12);
        assert_eq!(
            restored.players[0].mana_pool.amount(crate::mana::Color::Green),
            2
        );
        assert_eq!(restored.battlefield.len(), 1);
        let bear_back = &restored.battlefield[0];
        assert_eq!(bear_back.definition.name, "Grizzly Bears");
        assert!(bear_back.tapped);
        assert_eq!(bear_back.damage, 1);
        assert_eq!(bear_back.power_bonus, 2);
        assert_eq!(restored.players[1].hand.len(), 1);
        assert_eq!(restored.players[1].hand[0].definition.name, "Lightning Bolt");
    }

    #[test]
    fn snapshot_round_trips_once_per_turn_used() {
        // Activate Mindful Biomancer's once-per-turn pump, capture the
        // game, then restore. The "used" tracker must come back so the
        // engine still rejects a second activation post-restore.
        let mut g = two_player_game();
        let bio = g.add_card_to_battlefield(0, catalog::mindful_biomancer());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;
        // Pre-fill mana so the activation succeeds.
        g.players[0].mana_pool.add(crate::mana::Color::Green, 1);
        g.players[0].mana_pool.add_colorless(2);
        g.perform_action(GameAction::ActivateAbility {
            card_id: bio,
            ability_index: 0,
            target: None,
        })
        .expect("Mindful Biomancer pump activatable");
        // Drain the stack so the activation commits.
        while !g.stack.is_empty() {
            g.perform_action(GameAction::PassPriority).unwrap();
            g.perform_action(GameAction::PassPriority).unwrap();
        }
        let snap = GameSnapshot::capture(&g);
        let restored = snap.restore().expect("restore");
        let bio_back = restored
            .battlefield
            .iter()
            .find(|c| c.id == bio)
            .expect("Mindful Biomancer should round-trip");
        assert_eq!(bio_back.once_per_turn_used, vec![0],
            "Once-per-turn tracker must persist through snapshot/restore");
    }

    #[test]
    fn snapshot_round_trips_through_json() {
        let mut g = two_player_game();
        g.add_card_to_battlefield(0, catalog::forest());
        let json = serde_json::to_string(&GameSnapshot::capture(&g)).expect("serialize");
        let parsed: GameSnapshot = serde_json::from_str(&json).expect("deserialize");
        let restored = parsed.restore().expect("restore");
        assert_eq!(restored.battlefield.len(), 1);
        assert_eq!(restored.battlefield[0].definition.name, "Forest");
    }

    #[test]
    fn restore_after_action_keeps_engine_consistent() {
        // Restored state must still drive the engine: capture mid-game,
        // restore, and play a `PlayLand`. The fact that the action is
        // accepted (no `CardNotInHand` / `NotALand` error) is enough —
        // we're proving the snapshot reattaches the catalog cleanly,
        // not exercising bolt resolution semantics.
        let mut g = two_player_game();
        let forest = g.add_card_to_hand(0, catalog::forest());
        g.priority.player_with_priority = 0;
        g.active_player_idx = 0;

        let snap = GameSnapshot::capture(&g);
        let mut restored = snap.restore().expect("restore");

        restored
            .perform_action(GameAction::PlayLand(forest))
            .expect("Forest should play on the restored state");
        assert!(
            restored
                .battlefield
                .iter()
                .any(|c| c.id == forest && c.definition.name == "Forest"),
            "Forest must land on the battlefield after restore",
        );
    }

    #[test]
    fn maypay_effect_serde_round_trip() {
        // Push XVI: `Effect::MayPay { description, mana_cost, body }`
        // round-trips through serde without dropping fields. Important
        // because the catalog and any future replay tooling needs to
        // serialize Effect values; a missed field would silently corrupt
        // the rebuilt body.
        use crate::card::Effect;
        use crate::mana::{generic, ManaCost};
        let original = Effect::MayPay {
            description: "Pay {1} for great glory?".into(),
            mana_cost: ManaCost::new(vec![generic(1)]),
            body: Box::new(Effect::Draw {
                who: crate::card::Selector::You,
                amount: crate::card::Value::Const(2),
            }),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: Effect = serde_json::from_str(&json).expect("deserialize");
        match parsed {
            Effect::MayPay { description, mana_cost, body } => {
                assert_eq!(description, "Pay {1} for great glory?");
                assert_eq!(mana_cost.cmc(), 1);
                assert!(matches!(*body, Effect::Draw { .. }));
            }
            other => panic!("expected Effect::MayPay, got {:?}", other),
        }
    }

    #[test]
    fn unknown_card_fails_with_clear_error() {
        let cs = CardSnapshot {
            id: CardId(99),
            name: "Definitely Not A Real Card".into(),
            owner: 0,
            controller: 0,
            tapped: false,
            damage: 0,
            summoning_sick: false,
            power_bonus: 0,
            toughness_bonus: 0,
            counters: vec![],
            attached_to: None,
            kicked: false,
            face_down: false,
            is_token: false,
            used_loyalty_ability_this_turn: false,
            evoked: false,
            cast_from_hand: false,
            chosen_creature_type: None,
            once_per_turn_used: vec![],
        };
        match restore_card(cs) {
            Err(LoadError::UnknownCard(name)) => {
                assert_eq!(name, "Definitely Not A Real Card");
            }
            Ok(_) => panic!("expected UnknownCard error"),
        }
    }

    #[test]
    fn trigger_stack_items_are_counted_and_dropped() {
        // Hand-craft a stack with one Spell and one Trigger. Snapshot
        // should preserve the Spell and report dropped_triggers == 1.
        use crate::effect::Effect;
        let mut g = two_player_game();
        let bolt_id = g.add_card_to_battlefield(0, catalog::lightning_bolt());
        let bolt_card = g
            .battlefield_find(bolt_id)
            .cloned()
            .expect("bolt on bf");
        // Pop the bolt off the battlefield and stuff it on the stack as
        // a faux in-flight Spell, then add a Trigger alongside.
        g.battlefield.retain(|c| c.id != bolt_id);
        g.stack.push(StackItem::Spell {
            card: Box::new(bolt_card),
            caster: 0,
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
            uncounterable: false,
        });
        g.stack.push(StackItem::Trigger {
            source: bolt_id,
            controller: 0,
            effect: Box::new(Effect::Noop),
            target: None,
            mode: None,
            x_value: 0,
            converged_value: 0,
        });

        let snap = GameSnapshot::capture(&g);
        assert_eq!(snap.dropped_triggers, 1);
        assert_eq!(snap.stack.len(), 1);
        assert_eq!(snap.stack[0].card.name, "Lightning Bolt");
    }

    /// Full GameState now derives `Serialize`/`Deserialize` directly.
    /// Round-trip via serde_json including a Trigger on the stack
    /// (which the snapshot path drops but the direct serde path keeps).
    #[test]
    fn full_game_state_round_trips_through_json() {
        use crate::effect::Effect;
        let mut g = two_player_game();
        g.players[0].life = 13;
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.stack.push(StackItem::Trigger {
            source: CardId(99),
            controller: 0,
            effect: Box::new(Effect::Noop),
            target: None,
            mode: None,
            x_value: 7,
            converged_value: 0,
        });

        let json = serde_json::to_string(&g).expect("serialize GameState");
        let restored: GameState = serde_json::from_str(&json).expect("deserialize GameState");
        assert_eq!(restored.players[0].life, 13);
        assert_eq!(restored.battlefield.len(), 1);
        assert_eq!(restored.battlefield[0].definition.name, "Grizzly Bears");
        assert_eq!(restored.stack.len(), 1);
        match &restored.stack[0] {
            StackItem::Trigger { x_value, .. } => {
                assert_eq!(*x_value, 7,
                    "Trigger x_value should round-trip through serde");
            }
            other => panic!("Expected StackItem::Trigger, got {:?}", other),
        }
    }
}
