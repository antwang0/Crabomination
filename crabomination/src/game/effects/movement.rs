//! Helpers that move cards between zones (battlefield ↔ graveyard / hand /
//! library / exile) and apply damage to entities. Called from the resolver
//! `match` arms for `Effect::Move`, `Effect::Destroy`, `Effect::DealDamage`,
//! etc.

use super::{EffectContext, EntityRef};
use crate::card::{CardId, CardInstance, CounterType};
use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
use crate::game::{GameEvent, GameState};

impl GameState {
    pub(super) fn deal_damage_to(&mut self, ent: EntityRef, amount: u32, events: &mut Vec<GameEvent>) {
        self.deal_damage_to_from(ent, amount, None, events);
    }

    /// Damage delivery with the source's identity threaded through, so
    /// CR 702.90b (Infect) can convert player damage into poison
    /// counters when the source has the Infect keyword. `source` is
    /// the `CardId` of the damaging permanent (typically `ctx.source`).
    /// Combat damage uses a separate path in `combat.rs` that already
    /// honors infect for combat damage.
    pub(super) fn deal_damage_to_from(
        &mut self,
        ent: EntityRef,
        amount: u32,
        source: Option<crate::card::CardId>,
        events: &mut Vec<GameEvent>,
    ) {
        // CR 702.90b — damage dealt to a player by a source with infect
        // doesn't cause life loss; it gives the player poison counters
        // equal to that damage. We check the source's effective
        // keywords via `computed_permanent` so layered grants (e.g.
        // Triumph of the Hordes-style anthems) are honored.
        let source_has_infect = source
            .and_then(|s| self.computed_permanent(s))
            .map(|cp| cp.keywords.contains(&crate::card::Keyword::Infect))
            .unwrap_or(false);
        match ent {
            EntityRef::Player(p) => {
                if source_has_infect {
                    self.players[p].poison_counters =
                        self.players[p].poison_counters.saturating_add(amount);
                    events.push(GameEvent::PoisonAdded { player: p, amount });
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: Some(p),
                        to_card: None,
                    });
                } else {
                    self.players[p].life -= amount as i32;
                    events.push(GameEvent::DamageDealt { amount, to_player: Some(p), to_card: None });
                    events.push(GameEvent::LifeLost { player: p, amount });
                }
            }
            EntityRef::Permanent(cid) => {
                // CR 120.3c — damage dealt to a planeswalker causes that
                // many loyalty counters to be removed from that
                // planeswalker. Before this branch, non-combat
                // `Effect::DealDamage` was marking the damage on `c.damage`
                // regardless of card type, so a Lightning Bolt at a 3-loyalty
                // PW correctly removed 3 damage to be applied to toughness
                // (toughness = 0 → die!) but skipped the printed
                // loyalty-loss path. Combat damage already routes through
                // `combat.rs::AttackTarget::Planeswalker` which decrements
                // loyalty — this aligns spell damage with the same rule.
                let is_pw = self
                    .battlefield_find(cid)
                    .map(|c| c.definition.is_planeswalker())
                    .unwrap_or(false);
                if is_pw {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        let current = c.counter_count(CounterType::Loyalty);
                        let new_loyalty = current.saturating_sub(amount);
                        c.counters
                            .insert(CounterType::Loyalty, new_loyalty);
                        events.push(GameEvent::DamageDealt {
                            amount,
                            to_player: None,
                            to_card: Some(cid),
                        });
                        events.push(GameEvent::LoyaltyChanged {
                            card_id: cid,
                            new_loyalty: new_loyalty as i32,
                        });
                    }
                } else if let Some(c) = self.battlefield_find_mut(cid) {
                    c.damage += amount;
                    events.push(GameEvent::DamageDealt {
                        amount,
                        to_player: None,
                        to_card: Some(cid),
                    });
                }
            }
            EntityRef::Card(_) => {}
        }
    }

    pub(super) fn move_card_to(
        &mut self,
        cid: CardId,
        dest: &ZoneDest,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        // Resolve any selector-based player refs in the destination *now*,
        // while the card is still findable in its source zone — otherwise
        // `PlayerRef::OwnerOf(Target(0))` can't see the card after we remove
        // it. The resolved dest uses concrete `PlayerRef::You`-anchored refs.
        let resolved_dest = self.resolve_zonedest_player(dest, ctx);

        // Try battlefield first.
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == cid) {
            let mut card = self.battlefield.remove(pos);
            self.remove_effects_from_source(cid);
            card.damage = 0;
            card.tapped = false;
            card.attached_to = None;
            // CR 506.4 — A permanent leaving the battlefield is removed
            // from combat. The helper prunes `self.attacking` and
            // `self.block_map` so the post-move combat state stays
            // consistent for downstream selectors and trigger dispatchers.
            self.remove_from_combat(cid);
            self.place_card_in_dest(card, ctx.controller, &resolved_dest, events);
            return;
        }
        // Then graveyards. Emit `CardLeftGraveyard` so Strixhaven
        // "cards leave your graveyard" payoffs (Garrison Excavator,
        // Living History, Spirit Mascot, Hardened Academic) trigger.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].graveyard.iter().position(|c| c.id == cid) {
                let card = self.players[p].graveyard.remove(pos);
                self.players[p].cards_left_graveyard_this_turn =
                    self.players[p].cards_left_graveyard_this_turn.saturating_add(1);
                events.push(GameEvent::CardLeftGraveyard { player: p, card_id: cid });
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Then exile.
        if let Some(pos) = self.exile.iter().position(|c| c.id == cid) {
            let card = self.exile.remove(pos);
            let owner = card.owner;
            self.place_card_in_dest(card, owner, &resolved_dest, events);
            return;
        }
        // Hands. Used by start-of-game opening-hand effects
        // (Leyline of Sanctity, Gemstone Caverns) that move a hand card
        // to the battlefield.
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].hand.iter().position(|c| c.id == cid) {
                let card = self.players[p].hand.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
        // Libraries. Used by `Selector::TopOfLibrary` → `ZoneDest::Exile`
        // / `Hand` / etc. (Suspend Aggression's exile-top-of-library half,
        // Daydream's exile-then-return flicker pattern in passing).
        for p in 0..self.players.len() {
            if let Some(pos) = self.players[p].library.iter().position(|c| c.id == cid) {
                let card = self.players[p].library.remove(pos);
                self.place_card_in_dest(card, p, &resolved_dest, events);
                return;
            }
        }
    }

    /// Pre-resolve any selector-based player refs in a `ZoneDest` against
    /// the active ctx. `place_card_in_dest` constructs its own bare ctx and
    /// can't see the caster's targets, so any `PlayerRef::OwnerOf(Selector)`
    /// / `ControllerOf(Selector)` need to be flattened to a concrete
    /// `PlayerRef::Seat(n)` while the source card is still in its origin
    /// zone. Other ref kinds (You / ActivePlayer / etc.) pass through.
    pub(super) fn resolve_zonedest_player(&self, dest: &ZoneDest, ctx: &EffectContext) -> ZoneDest {
        let flatten = |who: &PlayerRef| -> PlayerRef {
            match who {
                PlayerRef::OwnerOf(_) | PlayerRef::ControllerOf(_) => {
                    if let Some(p) = self.resolve_player(who, ctx) {
                        PlayerRef::Seat(p)
                    } else {
                        who.clone()
                    }
                }
                _ => who.clone(),
            }
        };
        match dest {
            ZoneDest::Hand(who) => ZoneDest::Hand(flatten(who)),
            ZoneDest::Library { who, pos } => ZoneDest::Library {
                who: flatten(who),
                pos: *pos,
            },
            ZoneDest::Battlefield { controller, tapped } => ZoneDest::Battlefield {
                controller: flatten(controller),
                tapped: *tapped,
            },
            ZoneDest::Graveyard | ZoneDest::Exile => dest.clone(),
        }
    }

    pub(crate) fn place_card_in_dest(
        &mut self,
        mut card: CardInstance,
        default_player: usize,
        dest: &ZoneDest,
        events: &mut Vec<GameEvent>,
    ) {
        match dest {
            ZoneDest::Hand(who) => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(who, &ctx).unwrap_or(default_player);
                card.controller = p;
                self.players[p].hand.push(card);
            }
            ZoneDest::Library { who, pos } => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(who, &ctx).unwrap_or(default_player);
                match pos {
                    LibraryPosition::Top => self.players[p].library.insert(0, card),
                    LibraryPosition::Bottom => self.players[p].library.push(card),
                    LibraryPosition::Shuffled => {
                        // Push the card in, then shuffle the entire library
                        // so the card lands at a random position (Chaos Warp,
                        // bottom-of-library reanimate-prevention effects, etc.).
                        // Pre-fix this fell through to `push` (effectively
                        // sending to bottom), which exposed deterministic
                        // ordering across cards that semantically should
                        // randomize.
                        use rand::seq::SliceRandom;
                        let mut rng = rand::rng();
                        self.players[p].library.push(card);
                        self.players[p].library.shuffle(&mut rng);
                    }
                }
            }
            ZoneDest::Graveyard => {
                let owner = card.owner;
                self.players[owner].send_to_graveyard(card);
            }
            ZoneDest::Exile => {
                let cid = card.id;
                self.exile.push(card);
                // Bump the controller-of-the-exile-effect's per-turn
                // exile tally for Strixhaven "if one or more cards were
                // put into exile this turn" payoffs (Ennis the Debate
                // Moderator). Reset on `do_untap`.
                if default_player < self.players.len() {
                    self.players[default_player].cards_exiled_this_turn =
                        self.players[default_player].cards_exiled_this_turn.saturating_add(1);
                }
                events.push(GameEvent::PermanentExiled { card_id: cid });
            }
            ZoneDest::Battlefield { controller, tapped } => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(controller, &ctx).unwrap_or(default_player);
                card.controller = p;
                card.tapped = *tapped;
                card.summoning_sick = card.definition.is_creature();
                // A permanent entering the battlefield from another zone is
                // a brand-new object (rule 400.7) — clear residual damage,
                // pump bonuses, and attachment.
                card.damage = 0;
                card.power_bonus = 0;
                card.toughness_bonus = 0;
                card.attached_to = None;
                let cid = card.id;
                self.battlefield.push(card);
                events.push(GameEvent::PermanentEntered { card_id: cid });
                // Fire self-source ETB triggers so reanimate / flicker /
                // search-to-battlefield paths trigger creature ETBs the same
                // way casting does.
                self.fire_self_etb_triggers(cid, p);
            }
        }
    }
}
