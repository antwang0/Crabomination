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

    /// CR 614.9 — if damage aimed at `ent` (a player, or a permanent that
    /// player controls) is covered by a `RedirectDamageToSelf` static
    /// (Palisade Giant), return the redirecting permanent. The redirector
    /// never re-redirects its own damage.
    pub(crate) fn damage_redirect_target(&self, ent: EntityRef) -> Option<crate::card::CardId> {
        use crate::effect::StaticEffect;
        let protected = match ent {
            EntityRef::Player(p) => p,
            EntityRef::Permanent(c) => self.battlefield_find(c)?.controller,
            EntityRef::Card(_) => return None,
        };
        let aimed_at = match ent {
            EntityRef::Permanent(c) => Some(c),
            _ => None,
        };
        self.battlefield.iter().find_map(|c| {
            (c.controller == protected
                && Some(c.id) != aimed_at
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::RedirectDamageToSelf)))
            .then_some(c.id)
        })
    }

    /// CR 615.1 / 615.7 / 615.12 — apply prevention shields to a pending
    /// damage event aimed at `ent`. "Prevent all" shields zero the event;
    /// "prevent next N" shields soak up to N and then expire. The whole
    /// step is bypassed while `damage_cant_be_prevented_this_turn` is set.
    /// Emits `GameEvent::DamagePrevented` for the prevented portion
    /// (CR 615.13) and returns the unprevented remainder.
    pub(crate) fn apply_prevention_shields(
        &mut self,
        ent: EntityRef,
        amount: u32,
        events: &mut Vec<GameEvent>,
    ) -> u32 {
        use crate::game::types::PreventionTarget;
        if self.damage_cant_be_prevented_this_turn
            || self.prevention_shields.is_empty()
            || self.damage_cant_be_prevented_now()
        {
            return amount;
        }
        let (to_player, to_card, key) = match ent {
            EntityRef::Player(p) => (Some(p), None, PreventionTarget::Player(p)),
            EntityRef::Permanent(c) => (None, Some(c), PreventionTarget::Permanent(c)),
            EntityRef::Card(_) => return amount,
        };
        let mut remaining = amount;
        let mut prevented = 0u32;
        // CR 615.1 — life gained by `gain_life` shields that soak damage.
        let mut life_gain = 0u32;
        for shield in self.prevention_shields.iter_mut().filter(|s| s.target == key) {
            if remaining == 0 {
                break;
            }
            let soak = match shield.remaining {
                // Prevent-all: soak everything, shield stays for the turn.
                None => std::mem::take(&mut remaining),
                Some(ref mut n) => {
                    let soak = remaining.min(*n);
                    remaining -= soak;
                    *n -= soak;
                    soak
                }
            };
            prevented += soak;
            if shield.gain_life {
                life_gain += soak;
            }
        }
        // Drop spent "next N" shields (those reduced to 0).
        self.prevention_shields.retain(|s| s.remaining != Some(0));
        if prevented > 0 {
            events.push(GameEvent::DamagePrevented { amount: prevented, to_player, to_card });
        }
        if life_gain > 0 && let Some(p) = to_player {
            self.adjust_life(p, life_gain as i32);
            events.push(GameEvent::LifeGained { player: p, amount: life_gain });
        }
        remaining
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
        // CR 120.8 — "If a source would deal 0 damage, it does not deal
        // damage at all. That means abilities that trigger on damage
        // being dealt won't trigger. It also means that replacement
        // effects that would increase the damage dealt by that source,
        // or would have that source deal that damage to a different
        // object or player, have no event to replace, so they have no
        // effect." We bail out of the entire damage-delivery sequence
        // when `amount == 0`, so no `GameEvent::DamageDealt`,
        // `LifeLost`, `PoisonAdded`, or `LoyaltyChanged` event is
        // emitted. Damage-watching triggered abilities won't fire on
        // 0-damage events.
        if amount == 0 {
            return;
        }
        // CR 615.7 — "prevent all damage [chosen source] would deal this
        // turn" (Burrenton Forge-Tender), unless prevention is off (615.12).
        if let Some(src) = source
            && !self.damage_cant_be_prevented_this_turn
            && self.damage_prevented_sources.contains(&src)
        {
            return;
        }
        // CR 614.9 — redirect the whole event to a Palisade-Giant-style
        // permanent. One redirect per event (CR 614.5; the flag also stops
        // two redirectors ping-ponging).
        if !self.in_damage_redirect
            && let Some(redirect) = self.damage_redirect_target(ent)
        {
            self.in_damage_redirect = true;
            self.deal_damage_to_from(EntityRef::Permanent(redirect), amount, source, events);
            self.in_damage_redirect = false;
            return;
        }
        // CR 702.16e — protection from the source's color prevents the whole
        // damage event to a permanent (noncombat damage path).
        if let (EntityRef::Permanent(tgt), Some(src)) = (ent, source)
            && self.damage_prevented_by_protection(src, tgt)
        {
            return;
        }
        // CR 614.2 — global damage-doubling replacement (Furnace of Rath /
        // Gratuitous Violence). Each `DoubleDamageDealt` permanent doubles
        // the amount; applied before prevention so a shield soaks the
        // already-doubled total (CR 616 lets the affected player order the
        // two replacements — doubling-first is the common case and keeps the
        // event single-pass here).
        let doublers = self.damage_doublers();
        let amount = if doublers > 0 {
            amount.saturating_mul(1u32 << doublers.min(16))
        } else {
            amount
        };
        // CR 615.1 — prevention shields. Before applying the damage, let
        // any shield around the target soak it (unless a "damage can't be
        // prevented this turn" effect is active, CR 615.12). Returns the
        // unprevented remainder; 0 means the whole event is prevented.
        let amount = self.apply_prevention_shields(ent, amount, events);
        if amount == 0 {
            return;
        }
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
                // Bloodthirst (CR 702.54) window: any damage to a player
                // (combat or not, incl. infect→poison) marks them damaged
                // this turn.
                self.players[p].was_dealt_damage_this_turn = true;
                // Record the damaging creature so "destroy target creature
                // that dealt damage to you this turn" (Spear of Heliod) can
                // filter targets. Only track battlefield creatures.
                if let Some(src) = source {
                    let is_creature = self
                        .computed_permanent(src)
                        .map(|cp| cp.card_types.contains(&crate::card::CardType::Creature))
                        .unwrap_or(false);
                    if is_creature && !self.players[p].creatures_that_damaged_me_this_turn.contains(&src) {
                        self.players[p].creatures_that_damaged_me_this_turn.push(src);
                    }
                }
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
                    self.adjust_life(p, -(amount as i32));
                    events.push(GameEvent::DamageDealt { amount, to_player: Some(p), to_card: None });
                    events.push(GameEvent::LifeLost { player: p, amount });
                }
                // Phase M: direct damage from a commander source also
                // counts toward the 21-commander-damage SBA
                // (CR 704.5v doesn't restrict the damage type — combat
                // and non-combat both apply).
                if let Some(src) = source
                    && self.is_commander(src)
                {
                    self.record_commander_damage(p, src, amount);
                }
            }
            EntityRef::Permanent(cid) => {
                // CR 122.1c — Shield counters: if damage would be dealt
                // to this permanent, prevent that damage and remove a
                // shield counter from it.
                let has_shield = self
                    .battlefield_find(cid)
                    .map(|c| c.counter_count(CounterType::Shield) > 0)
                    .unwrap_or(false);
                if has_shield {
                    if let Some(c) = self.battlefield_find_mut(cid) {
                        let cur = c.counter_count(CounterType::Shield);
                        c.counters.insert(CounterType::Shield, cur.saturating_sub(1));
                    }
                    return;
                }
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
        // CR 702.15 — lifelink on the non-combat damage path: if the source is
        // a lifelink permanent (a ping ability) or an instant/sorcery spell
        // whose controller has "your spells have lifelink" (Radiant
        // Scrollwielder), that controller gains life equal to the damage dealt.
        // (Combat damage handles its own lifelink in `combat.rs`.)
        if let Some(seat) = self.noncombat_lifelink_seat(source) {
            self.adjust_life(seat, amount as i32);
            events.push(GameEvent::LifeGained { player: seat, amount });
        }
    }

    /// Seat that gains life from lifelink on a *non-combat* damage event from
    /// `source`, if any (CR 702.15). Returns the source's controller when the
    /// source is a lifelink permanent, or the caster of an instant/sorcery
    /// spell whose controller grants spells lifelink.
    fn noncombat_lifelink_seat(&self, source: Option<crate::card::CardId>) -> Option<usize> {
        use crate::card::Keyword;
        // A lifelink permanent (e.g. a ping ability from a lifelink creature).
        if let Some(src) = source
            && let Some(cp) = self.computed_permanent(src)
            && cp.keywords.contains(&Keyword::Lifelink)
        {
            return Some(cp.controller);
        }
        // The currently-resolving instant/sorcery whose controller grants
        // spells lifelink (stamped in `resolve_top_of_stack`).
        self.resolving_spell_lifelink_seat
    }

    /// True if `seat` controls a permanent granting
    /// `StaticEffect::YourInstantSorcerySpellsHaveLifelink` (Radiant Scrollwielder).
    pub(crate) fn controller_grants_spell_lifelink(&self, seat: usize) -> bool {
        use crate::effect::StaticEffect;
        self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition.static_abilities.iter().any(|sa| {
                    matches!(sa.effect, StaticEffect::YourInstantSorcerySpellsHaveLifelink)
                })
        })
    }

    /// CR 701.34 — manifest the card `cid` (in player `p`'s library): flip it
    /// face down in place so it enters as a vanilla 2/2 (no real-card ETB
    /// triggers), then put it onto the battlefield under `p`'s control.
    pub(crate) fn manifest_card(
        &mut self,
        cid: CardId,
        p: usize,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        if let Some(c) = self.players[p].library.iter_mut().find(|c| c.id == cid) {
            c.turn_face_down();
        }
        let dest = ZoneDest::Battlefield {
            controller: crate::effect::PlayerRef::Seat(p),
            tapped: false,
        };
        self.move_card_to(cid, &dest, ctx, events);
    }

    pub(crate) fn move_card_to(
        &mut self,
        cid: CardId,
        dest: &ZoneDest,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) {
        // Grafdigger's Cage — creature cards in graveyards and libraries
        // can't enter the battlefield.
        if matches!(dest, ZoneDest::Battlefield { .. })
            && self.graveyard_library_locked()
            && self.players.iter().any(|pl| {
                pl.graveyard
                    .iter()
                    .chain(pl.library.iter())
                    .any(|c| c.id == cid && c.definition.is_creature())
            })
        {
            return;
        }
        // Resolve any selector-based player refs in the destination *now*,
        // while the card is still findable in its source zone — otherwise
        // `PlayerRef::OwnerOf(Target(0))` can't see the card after we remove
        // it. The resolved dest uses concrete `PlayerRef::You`-anchored refs.
        let resolved_dest = self.resolve_zonedest_player(dest, ctx);

        // Try battlefield first.
        if let Some(pos) = self.battlefield.iter().position(|c| c.id == cid) {
            let mut card = self.battlefield.remove(pos);
            self.remove_effects_from_source(cid);
            // CR 708.10 — a face-down permanent is turned face up as it leaves
            // the battlefield (no-op unless it carries a stashed real def).
            card.turn_face_up();
            // CR 707 — a temporary copy reverts as it leaves.
            self.revert_copy_on_leave(&mut card);
            card.damage = 0;
            card.tapped = false;
            card.attached_to = None;
            // CR 506.4 — A permanent leaving the battlefield is removed
            // from combat. The helper prunes `self.attacking` and
            // `self.block_map` so the post-move combat state stays
            // consistent for downstream selectors and trigger dispatchers.
            self.remove_from_combat(cid);
            self.place_card_in_dest(card, ctx.controller, &resolved_dest, events);
            self.return_linked_exiles(cid, events);
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
                // Flatten `PlayerRef::You` to the caster's seat now —
                // `place_card_in_dest` builds its own context anchored to
                // the card's *origin owner* (which is the graveyard owner
                // for gy-to-bf moves like Mind Roots, not the caster). If
                // we don't flatten here, "controller: PlayerRef::You" on a
                // ZoneDest::Battlefield would end up resolving to the
                // graveyard's owner instead of the caster, putting the
                // stolen land back under the opp's control.
                PlayerRef::You => PlayerRef::Seat(ctx.controller),
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
        // Phase H — consult the replacement-effect registry. The
        // resolver only sees the *destination kind* (a `Zone`); the
        // origin is left unconstrained here (passed as
        // `Zone::Battlefield` for now, which covers the Commander
        // case since its replacement effect uses `from: None`).
        // If the resolver redirects to a different zone, we hand off
        // to `place_card_at_resolved_zone` which handles the
        // terminal-zone placement uniformly. Same-zone return falls
        // through to the existing rich `ZoneDest` logic so player /
        // tapped / library-position information is preserved.
        let intended = match dest {
            ZoneDest::Hand(_) => crate::card::Zone::Hand,
            ZoneDest::Library { .. } => crate::card::Zone::Library,
            ZoneDest::Battlefield { .. } => crate::card::Zone::Battlefield,
            ZoneDest::Graveyard => crate::card::Zone::Graveyard,
            ZoneDest::Exile => crate::card::Zone::Exile,
        };
        let resolved = self.resolve_zone_change(
            card.id,
            crate::card::Zone::Battlefield,
            intended,
        );
        if resolved != intended {
            self.place_card_at_resolved_zone(card, resolved);
            return;
        }
        match dest {
            ZoneDest::Hand(who) => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                // `OwnerOfMoved` routes the card to *its own* owner (per-card
                // board-bounce — Aetherize / Evacuation).
                let p = match who {
                    PlayerRef::OwnerOfMoved => card.owner,
                    _ => self.resolve_player(who, &ctx).unwrap_or(default_player),
                };
                card.controller = p;
                self.players[p].hand.push(card);
            }
            ZoneDest::Library { who, pos } => {
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = match who {
                    PlayerRef::OwnerOfMoved => card.owner,
                    _ => self.resolve_player(who, &ctx).unwrap_or(default_player),
                };
                match pos {
                    LibraryPosition::Top => self.players[p].library.insert(0, card),
                    LibraryPosition::Bottom => self.players[p].library.push(card),
                    LibraryPosition::OwnerChoice => {
                        // CR 701: "owner's choice" library placement.
                        // Ask the *owner* of the moved card (= the
                        // library we're putting it in — `p` resolved
                        // above) yes/no via `Decision::OptionalTrigger`.
                        // True = top, false = bottom. AutoDecider
                        // defaults to false (bottom). Run Behind is the
                        // only printed user today.
                        let decision = crate::decision::Decision::OptionalTrigger {
                            source: card.id,
                            description: "Put on top of library? (no = bottom)".into(),
                        };
                        let answer = self.decider.decide(&decision);
                        let put_on_top = matches!(
                            answer,
                            crate::decision::DecisionAnswer::Bool(true)
                        );
                        if put_on_top {
                            self.players[p].library.insert(0, card);
                        } else {
                            self.players[p].library.push(card);
                        }
                    }
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
                    LibraryPosition::FromTop(n) => {
                        // CR 401.7: "If a player is instructed to put a
                        // card 'Nth from the top' of a library, and there
                        // are fewer than N cards in that library, the
                        // card is put on the bottom of that library."
                        // `FromTop(0)` = top; otherwise insert at index
                        // `n` if the library has at least `n` cards,
                        // else `push` (= bottom).
                        let lib_len = self.players[p].library.len();
                        if *n >= lib_len {
                            self.players[p].library.push(card);
                        } else {
                            self.players[p].library.insert(*n, card);
                        }
                    }
                }
            }
            ZoneDest::Graveyard => {
                // CR 614.6 — graveyard-hate statics redirect to exile.
                self.route_to_graveyard(card, events);
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
                // CR 614.x — Containment Priest. A nontoken creature put onto
                // the battlefield without being cast (reanimate / blink /
                // reveal-and-put — anything routed through this path rather
                // than `resolve_spell`) is exiled instead.
                if !card.is_token
                    && card.definition.is_creature()
                    && self.nontoken_creature_etb_exile_active()
                {
                    let cid = card.id;
                    self.exile.push(card);
                    events.push(GameEvent::PermanentExiled { card_id: cid });
                    return;
                }
                let ctx = EffectContext::for_spell(default_player, None, 0, 0);
                let p = self.resolve_player(controller, &ctx).unwrap_or(default_player);
                card.controller = p;
                card.tapped = *tapped;
                card.summoning_sick = card.definition.is_creature();
                // CR 603.4 — stamp the entry turn immediately (the central
                // dispatch loop also sets it, but doing it here makes it
                // readable mid-resolution — Emergent Sequence counts the land
                // it just put in).
                card.entered_turn = Some(self.turn_number);
                // A permanent entering the battlefield from another zone is
                // a brand-new object (rule 400.7) — clear residual damage,
                // pump bonuses, and attachment.
                card.damage = 0;
                card.power_bonus = 0;
                card.toughness_bonus = 0;
                card.attached_to = None;
                // CR 122.2 — counters cease to exist when a permanent leaves
                // the battlefield; the new object enters with none. Re-seed a
                // planeswalker's starting loyalty (CR 306.5b) so a reanimated
                // / blinked planeswalker enters with full base loyalty rather
                // than its last-known (possibly 0) value.
                card.counters.clear();
                card.keyword_counters.clear();
                if card.definition.is_planeswalker() && card.definition.base_loyalty > 0 {
                    card.counters
                        .insert(CounterType::Loyalty, card.definition.base_loyalty);
                }
                let cid = card.id;
                // CR 614.12 — apply "enters with N counters" replacement
                // BEFORE the new permanent is exposed to state-based-action
                // sweeps and BEFORE ETB triggers fire. This lets a printed
                // 0/0 or 1/0 body (Pterafractyl, Symmathematics) survive
                // without the historic base-toughness bump workaround. The
                // Value is evaluated against a self-ability ctx anchored
                // to the new permanent's `CardId` so `Value::XFromCost`
                // reads via a `for_ability` shim — for spells using
                // `Value::Const(N)` (Symmathematics) this is exact; for
                // X-on-cast bodies (Pterafractyl) the x_value would need
                // additional plumbing through `move_card_to` from the
                // cast-time ctx, tracked separately.
                let enters_spec = card.definition.enters_with_counters.clone();
                let mut card = card;
                card.controller = self.apply_etb_control_replacement(&card, card.controller);
                self.battlefield.push(card);
                // CR 122.1 — Solemnity drops the enters-with-counters too.
                let mut counter_specs: Vec<(crate::card::CounterType, crate::effect::Value)> =
                    Vec::new();
                if let Some(spec) = enters_spec {
                    counter_specs.push(spec);
                }
                // Metallic Mimic-style chosen-type ETB counters (any matching
                // creature entry — tokens, reanimation, search-to-battlefield).
                for kind in self.chosen_type_etb_counter_specs(cid, p) {
                    counter_specs.push((kind, crate::effect::Value::Const(1)));
                }
                if self.counters_locked() { counter_specs.clear(); }
                for (kind, value) in counter_specs {
                    let etb_ctx = crate::game::effects::EffectContext::for_ability(cid, p, None);
                    let base = self.evaluate_value(&value, &etb_ctx);
                    if base > 0 {
                        // CR 614.16: counter-doubling statics also apply
                        // to the "enters with N counters" replacement.
                        let target_ctrl = self
                            .battlefield
                            .iter()
                            .find(|c| c.id == cid)
                            .map(|c| c.controller);
                        let mut n = base as u32;
                        if let Some(ctrl) = target_ctrl {
                            let doublers = self.counter_doublers_for(ctrl);
                            for _ in 0..doublers {
                                n = n.saturating_mul(2);
                            }
                        }
                        if let Some(card_mut) =
                            self.battlefield.iter_mut().find(|c| c.id == cid)
                        {
                            card_mut.add_counters(kind, n);
                        }
                        events.push(GameEvent::CounterAdded {
                            card_id: cid,
                            counter_type: kind,
                            count: n,
                        });
                    }
                }
                // CR 702.32 / 702.62 — Fading / Vanishing enter-with-counters.
                self.apply_fading_vanishing_etb(cid, events);
                events.push(GameEvent::PermanentEntered { card_id: cid });
                // Fire self-source ETB triggers so reanimate / flicker /
                // search-to-battlefield paths trigger creature ETBs the same
                // way casting does.
                self.fire_self_etb_triggers(cid, p);
            }
        }
    }

    /// CR 603.6e — when a permanent that exiled card(s) via
    /// `Effect::ExileUntilSourceLeaves` leaves the battlefield, return the
    /// linked card(s) to the zone the linking ability specified
    /// (battlefield for Banisher Priest / Oblivion Ring, hand for Brain
    /// Maggot / Tidehollow Sculler). Called from every battlefield-removal
    /// path. The return is resolved directly rather than as a stack
    /// trigger — a deliberate simplification; the observable result (the
    /// card comes back) matches the printed linked ability.
    pub(crate) fn return_linked_exiles(
        &mut self,
        source: CardId,
        events: &mut Vec<GameEvent>,
    ) {
        use crate::card::ExileReturnZone;
        let linked: Vec<CardId> = self
            .exile
            .iter()
            .filter(|c| c.exiled_by.map(|l| l.source) == Some(source))
            .map(|c| c.id)
            .collect();
        for cid in linked {
            let Some(pos) = self.exile.iter().position(|c| c.id == cid) else {
                continue;
            };
            // Skyclave Apparition: the card stays in exile; its owner gets
            // an X/X blue Illusion (X = the card's mana value) instead.
            if self.exile[pos].exiled_by.map(|l| l.return_to)
                == Some(ExileReturnZone::IllusionToken)
            {
                let owner = self.exile[pos].owner;
                let mv = self.exile[pos].definition.cost.cmc() as i32;
                self.exile[pos].exiled_by = None;
                let def = crate::card::CardDefinition {
                    name: "Illusion",
                    cost: crate::mana::ManaCost::default(),
                    card_types: vec![crate::card::CardType::Creature],
                    subtypes: crate::card::Subtypes {
                        creature_types: vec![crate::card::CreatureType::Illusion],
                        ..Default::default()
                    },
                    power: mv,
                    toughness: mv,
                    ..Default::default()
                };
                self.mint_token_onto_battlefield(def, owner, false, events);
                continue;
            }
            let mut card = self.exile.remove(pos);
            let return_to = card.exiled_by.take().map(|l| l.return_to);
            let owner = card.owner;
            let dest = match return_to {
                Some(ExileReturnZone::Hand) => ZoneDest::Hand(PlayerRef::Seat(owner)),
                Some(ExileReturnZone::BattlefieldTapped) => ZoneDest::Battlefield {
                    controller: PlayerRef::Seat(owner),
                    tapped: true,
                },
                _ => ZoneDest::Battlefield {
                    controller: PlayerRef::Seat(owner),
                    tapped: false,
                },
            };
            self.place_card_in_dest(card, owner, &dest, events);
        }
    }
}
