//! Affordance / legality probes: which actions a seat could legally take
//! *right now* (castable hand cards, activatable permanents, legal
//! attackers/blockers, alt-cast modes). Each dry-runs `perform_action`
//! against a library-stripped clone — see `affordance_probe_template`.
//! Split out of `game/mod.rs` (no behavior change).

use super::*;

/// One activated ability's probe data: `(is_mana_ability, optional effect to
/// auto-target)`. The mana flag lets the probe loop skip mana abilities
/// without re-walking the effect tree.
type AbilityProbe = (bool, Option<Effect>);

impl GameState {
    /// Dry-run an action: clone the state, apply the action on the
    /// clone, return whether the engine would accept it. The caller's
    /// state is **not** modified. Used by the random bot to filter
    /// out actions the engine would reject — it's the most robust
    /// way to bottom out edge cases (Teferi sorcery-locking instants,
    /// Damping Sphere mana tax, hexproof targets, stolen permanents,
    /// summoning sickness, …) without re-implementing every engine
    /// rule on the bot side.
    ///
    /// Cost: one full `GameState::clone` + one `perform_action`. The
    /// random bot does this only on actions it's about to submit, so
    /// the overhead is bounded by the number of bot ticks, not by
    /// the size of the search space.
    pub fn would_accept(&self, action: GameAction) -> bool {
        // pending_decision routes through `submit_decision`, which
        // both reads AND writes `self.pending_decision`. Cloning is
        // safe but the dry-run can spuriously reject a legal answer
        // because the cloned decider drops scripted state. For the
        // bot's purposes — filtering out illegal `CastSpell` / land
        // taps / attacks — the no-pending-decision path is what
        // matters; if a decision is pending the bot uses
        // `SubmitDecision` directly which doesn't go through here.
        let mut probe = self.clone();
        probe.perform_action(action).is_ok()
    }

    /// A clone of `self` with every player's library emptied, for use as a
    /// reusable dry-run *template* by the from-hand affordance probes
    /// ([`would_accept_on`](Self::would_accept_on)).
    ///
    /// Why this is safe: cast / activate / play-land legality never reads
    /// library contents. A cast validates against hand, battlefield,
    /// graveyard (delve), and player flags, then pushes the spell to the
    /// stack and returns — resolution (the only library-touching step, e.g.
    /// a draw or fetch) happens on a *later* priority pass that the probe
    /// never reaches. An empty library is not itself a game-loss (deck-out
    /// fires only on a draw *attempt*, CR 104.3a/120.3), so clearing it
    /// can't flip `would_accept`'s `is_ok()` outcome.
    ///
    /// Why it's worth it: the affordance sweep dry-runs one `perform_action`
    /// per candidate hand card, each on a fresh `GameState` clone. The
    /// libraries are by far the largest part of that clone (a 60-card deck
    /// is ~53 `CardInstance`s vs. ~7 in hand). Cloning the template once and
    /// then cheaply re-cloning the library-less template per card turns N
    /// full-deck clones into one full clone + N light clones.
    fn affordance_probe_template(&self) -> GameState {
        let mut template = self.clone();
        for p in &mut template.players {
            p.library.clear();
        }
        template
    }

    /// Dry-run `action` against a prebuilt [`affordance_probe_template`]
    /// instead of cloning the whole `GameState`. Equivalent to
    /// [`would_accept`](Self::would_accept) for cast / activate / play-land
    /// actions (see the template doc for why library contents are
    /// irrelevant to their legality), but cheap to repeat across a hand.
    ///
    /// [`affordance_probe_template`]: Self::affordance_probe_template
    fn would_accept_on(template: &GameState, action: GameAction) -> bool {
        let mut probe = template.clone();
        probe.perform_action(action).is_ok()
    }

    /// CardIds in `caster`'s hand they could begin casting (or play, for
    /// lands) **right now**. Drives the client's "castable" hand-card
    /// highlight.
    ///
    /// Authoritative: every candidate is dry-run through [`would_accept`],
    /// so the result already reflects timing (sorcery vs. instant speed,
    /// remaining land drops), auto-tappable mana, cost taxes, and target
    /// availability — exactly the gates the real cast would hit. Mirrors
    /// the bot's candidate construction in `server::bot`, but only probes
    /// each card's default mode at X = 0, so a card castable *only* in a
    /// non-default mode (or at higher X) may be omitted — acceptable for a
    /// visual hint.
    ///
    /// Returns empty unless `caster` currently holds priority: you can't
    /// cast without it, and short-circuiting skips the per-card state
    /// clones on everyone else's priority, keeping view projection cheap.
    ///
    /// [`would_accept`]: Self::would_accept
    pub fn castable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.castable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`castable_hand_cards`] against a prebuilt probe template. The caller
    /// is responsible for the priority short-circuit; this runs the per-card
    /// dry-runs against `template` so a consolidated affordance sweep can
    /// share one template across every category.
    ///
    /// [`castable_hand_cards`]: Self::castable_hand_cards
    fn castable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        // Snapshot what each probe needs up front so the immutable borrow
        // of `self.players[caster].hand` is released before the cloning
        // probes run. The effect is cloned only for targeted non-lands.
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .map(|c| {
                let is_land = c.definition.is_land();
                let needs_target = !is_land && c.definition.effect.requires_target();
                (c.id, is_land, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();

        let mut out = Vec::new();
        for (id, is_land, targeted_effect) in &hand {
            let accepted = if *is_land {
                Self::would_accept_on(template, GameAction::PlayLand(*id))
            } else {
                // Auto-pick targets the way the bot does so a targeted
                // removal spell isn't reported uncastable merely for lack
                // of a target argument. A spell that needs a target but has
                // no legal one stays `target: None`, which `would_accept`
                // correctly rejects (CR 601.2c).
                let (target, additional_targets) = match targeted_effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                };
                Self::would_accept_on(template, GameAction::CastSpell {
                    card_id: *id,
                    target,
                    additional_targets,
                    mode: None,
                    x_value: None,
                })
            };
            if accepted {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds of permanents `seat` controls with at least one **non-mana**
    /// activated ability they could activate **right now** — dry-run through
    /// [`would_accept`] so timing, mana, tap state, and target availability
    /// are all honored. Drives a client "this permanent can do something"
    /// highlight (legal-plays hint, roadmap Tier 7/8) and the client's
    /// priority-stop heuristic (don't auto-pass when the viewer has a real
    /// instant-speed play). Empty off-priority.
    ///
    /// Pure mana abilities are intentionally excluded: they never use the
    /// stack and are auto-tapped on demand during payment, so a lone mana
    /// dork is not a reason to hold priority or stop step advancement.
    ///
    /// [`would_accept`]: Self::would_accept
    pub fn activatable_permanents(&self, seat: usize) -> Vec<CardId> {
        if self.player_with_priority() != seat {
            return Vec::new();
        }
        self.activatable_permanents_on(&self.affordance_probe_template(), seat)
    }

    /// [`activatable_permanents`] against a prebuilt probe template; the
    /// caller owns the priority short-circuit.
    ///
    /// [`activatable_permanents`]: Self::activatable_permanents
    fn activatable_permanents_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        // Snapshot (id, [ability probes]) so the borrow of `self.battlefield`
        // is released before the cloning probes run.
        let perms: Vec<(CardId, Vec<AbilityProbe>)> = self
            .battlefield
            .iter()
            // Seat's own permanents, plus opponents' permanents carrying an
            // `opponents_only` ability the seat may activate (Detention Vortex).
            .filter(|c| {
                !c.definition.activated_abilities.is_empty()
                    && (c.controller == seat
                        || (!self.same_team(c.controller, seat)
                            && c.definition.activated_abilities.iter().any(|a| a.opponents_only)))
            })
            .map(|c| {
                let owns = c.controller == seat;
                let effs = c
                    .definition
                    .activated_abilities
                    .iter()
                    .map(|a| {
                        // Only surface abilities the seat is actually allowed to
                        // use: own permanents' non-opponents_only abilities, or
                        // opponents' opponents_only abilities.
                        let usable = if owns { !a.opponents_only } else { a.opponents_only };
                        let targeted = (usable && a.effect.requires_target()).then(|| a.effect.clone());
                        (is_mana_ability_effect(&a.effect) || !usable, targeted)
                    })
                    .collect();
                (c.id, effs)
            })
            .collect();

        let mut out = Vec::new();
        for (id, ability_effects) in &perms {
            let any = ability_effects.iter().enumerate().any(|(idx, (is_mana, targeted))| {
                // Skip mana abilities — they don't use the stack and aren't a
                // meaningful instant-speed play (see method doc).
                if *is_mana {
                    return false;
                }
                let target = match targeted {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, seat, None).0,
                    None => None,
                };
                Self::would_accept_on(template, GameAction::ActivateAbility {
                    card_id: *id,
                    ability_index: idx,
                    target,
                    x_value: None,
                })
            });
            if any {
                out.push(*id);
            }
        }
        out
    }

    /// Creatures `seat` controls that could be declared as attackers right
    /// now — only meaningful during `seat`'s Declare Attackers step while it
    /// holds priority. Drives the client's legal-attacker highlight
    /// (roadmap Tier 8). Honors tapped / summoning-sickness / Defender /
    /// CantAttack via `CardInstance::can_attack`.
    pub fn legal_attackers(&self, seat: usize) -> Vec<CardId> {
        if self.step != crate::TurnStep::DeclareAttackers
            || self.active_player_idx != seat
            || self.player_with_priority() != seat
        {
            return Vec::new();
        }
        use crate::card::Keyword;
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.can_attack())
            .filter(|c| {
                // Honor layer-granted Defender / can't-attack and the
                // per-defender attack restriction (Dandân) so the client's
                // highlight matches what `declare_attackers` will accept.
                let kws = self
                    .computed_permanent(c.id)
                    .map(|cp| cp.keywords.clone())
                    .unwrap_or_else(|| c.definition.keywords.clone());
                if kws.contains(&Keyword::Defender) || kws.contains(&Keyword::CantAttack) {
                    return false;
                }
                kws.iter().all(|kw| match kw {
                    Keyword::CanAttackOnlyIfDefenderControls(req) => {
                        // Legal if at least one alive opponent's board satisfies
                        // the filter (they could be chosen as the defender).
                        (0..self.players.len()).any(|d| {
                            d != seat
                                && self.players[d].is_alive()
                                && self.battlefield.iter().any(|p| {
                                    p.controller == d
                                        && self.evaluate_requirement_on_card(req, p, d)
                                })
                        })
                    }
                    Keyword::CanAttackOnlyIfYouControl(req) => self
                        .battlefield
                        .iter()
                        .any(|p| p.controller == seat && self.evaluate_requirement_on_card(req, p, seat)),
                    Keyword::CantAttackOrBlockUnlessEvenCounters => {
                        c.counters.values().sum::<u32>() % 2 == 0
                    }
                    _ => true,
                })
            })
            .map(|c| c.id)
            .collect()
    }

    /// Creatures `seat` controls that could legally block at least one of the
    /// currently-declared attackers — only meaningful during the Declare
    /// Blockers step with attackers on the board. Drives the client's
    /// legal-blocker highlight (roadmap Tier 8). Uses
    /// `can_block_any_attacker` so flying / menace-style restrictions apply.
    pub fn legal_blockers(&self, seat: usize) -> Vec<CardId> {
        if self.step != crate::TurnStep::DeclareBlockers || self.attacking().is_empty() {
            return Vec::new();
        }
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.can_block())
            .filter(|c| self.can_block_any_attacker(c.id))
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards the viewer could cast *with their Kicker paid* right now
    /// (CR 702.32) — computed via a `CastSpellKicked` dry-run so it accounts
    /// for the full base+kicker cost, timing, and the kicked target set.
    /// Lets the client surface a "pay kicker?" affordance on those cards.
    /// Empty when the viewer doesn't hold priority.
    pub fn kickable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.kickable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`kickable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`kickable_hand_cards`]: Self::kickable_hand_cards
    fn kickable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_kicker().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            // Use the kicked target set (the broader `If(SpellWasKicked, …)`
            // branch) so a kicked Tear Asunder reports castable at a creature.
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => {
                        self.auto_targets_for_effect_all_slots_kicked(eff, caster, None, true)
                    }
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellKicked {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds in the caster's hand with a live Miracle window (CR 702.94) —
    /// revealed as the turn's first draw, castable for the cheaper miracle
    /// cost. A pure structural filter (no dry-run): the stamped
    /// `granted_alt_cast_cost_eot` + `may_play_until` for this seat is the
    /// signal `maybe_grant_miracle` leaves behind.
    pub fn miracle_hand_cards(&self, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.granted_alt_cast_cost_eot.is_some()
                    && c.may_play_until.as_ref().is_some_and(|p| p.player == caster)
            })
            .map(|c| c.id)
            .collect()
    }

    /// CardIds in the caster's hand they could cast right now paying the
    /// optional Buyback cost (CR 702.27). Mirrors `kickable_hand_cards`.
    pub fn buyback_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.buyback_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// CR 702.176 — hand cards with Bargain the caster could cast right now
    /// (probed at `sacrifice: None`, since the Bargain cost is optional).
    /// Mirrors `buyback_hand_cards`.
    fn bargainable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.keywords.contains(&Keyword::Bargain))
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellBargain {
                card_id: *id,
                sacrifice: None,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.157 — hand cards with Squad the caster could cast paying the
    /// squad cost at least once right now (probed at `times: 1`). Mirrors
    /// `bargainable_hand_cards_on`.
    fn squadable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.squad_cost().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellSquad {
                card_id: *id, times: 1, target, additional_targets, mode: None, x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CR 702.107 — hand cards with Replicate the caster could cast paying the
    /// replicate cost at least once right now (probed at `times: 1`).
    fn replicatable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.replicate_cost().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellReplicate {
                card_id: *id, times: 1, target, additional_targets, mode: None, x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// [`buyback_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`buyback_hand_cards`]: Self::buyback_hand_cards
    fn buyback_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let hand: Vec<(CardId, bool, Option<_>)> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_buyback().is_some())
            .map(|c| {
                let needs_target = c.definition.effect.requires_target();
                (c.id, needs_target, needs_target.then(|| c.definition.effect.clone()))
            })
            .collect();
        let mut out = Vec::new();
        for (id, needs_target, effect) in &hand {
            let (target, additional_targets) = if *needs_target {
                match effect {
                    Some(eff) => self.auto_targets_for_effect_all_slots(eff, caster, None),
                    None => (None, Vec::new()),
                }
            } else {
                (None, Vec::new())
            };
            if Self::would_accept_on(template, GameAction::CastSpellBuyback {
                card_id: *id,
                target,
                additional_targets,
                mode: None,
                x_value: None,
            }) {
                out.push(*id);
            }
        }
        out
    }

    /// CardIds in the caster's hand that have Bestow and could be cast as an
    /// Aura on some creature right now (CR 702.103). The auto-target picks a
    /// creature; an empty result means no legal host or the cost is unpayable.
    pub fn bestowable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.bestowable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`bestowable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`bestowable_hand_cards`]: Self::bestowable_hand_cards
    fn bestowable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        let candidates: Vec<CardId> = self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.has_bestow().is_some())
            .map(|c| c.id)
            .collect();
        let mut out = Vec::new();
        for id in candidates {
            // Bestow needs a creature host; pick any legal creature target.
            let host = self
                .battlefield
                .iter()
                .find(|c| {
                    c.definition.is_creature()
                        && self
                            .check_target_legality_with_source(
                                &Target::Permanent(c.id),
                                caster,
                                Some(id),
                            )
                            .is_ok()
                })
                .map(|c| c.id);
            let Some(host) = host else { continue };
            if Self::would_accept_on(template, GameAction::CastBestow {
                card_id: id,
                target: Some(Target::Permanent(host)),
                additional_targets: vec![],
                mode: None,
                x_value: None,
            }) {
                out.push(id);
            }
        }
        out
    }

    /// Hand cards the player can activate a `from_hand` ability of right now
    /// (Spirit-Guide-style "Exile this from your hand: Add mana"). Lets the
    /// client surface a pitch affordance distinct from the castable-for-value
    /// highlight. Empty when it isn't the player's priority.
    pub fn pitchable_hand_cards(&self, player: usize) -> Vec<CardId> {
        if self.player_with_priority() != player {
            return Vec::new();
        }
        self.players[player]
            .hand
            .iter()
            .filter(|c| {
                c.definition
                    .activated_abilities
                    .iter()
                    .any(|a| a.from_hand)
            })
            .map(|c| c.id)
            .collect()
    }

    /// Hand cards the player could cast right now via their **Dash**
    /// alternative cost (CR 702.110). Lets the client surface a dash
    /// affordance distinct from the normal castable highlight. Empty when
    /// it isn't the player's priority.
    pub fn dashable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.dashable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`dashable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`dashable_hand_cards`]: Self::dashable_hand_cards
    fn dashable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.dash))
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastSpellAlternative {
                    card_id: id,
                    pitch_card: None,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// Cards in `caster`'s hand they could cast for their Blitz cost right now
    /// (CR 702.152). Surfaced in `PlayerView.blitzable_hand` so the client can
    /// offer a "Blitz" affordance alongside Dash.
    pub fn blitzable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.blitzable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`blitzable_hand_cards`] against a prebuilt probe template; the caller
    /// owns the priority short-circuit.
    ///
    /// [`blitzable_hand_cards`]: Self::blitzable_hand_cards
    fn blitzable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.alternative_cost.as_ref().is_some_and(|a| a.blitz))
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(template, GameAction::CastSpellAlternative {
                    card_id: id,
                    pitch_card: None,
                    target: None,
                    additional_targets: vec![],
                    mode: None,
                    x_value: None,
                })
            })
            .collect()
    }

    /// Cards in `caster`'s hand they could suspend right now (CR 702.62):
    /// the card has `Keyword::Suspend` and the suspend action would be
    /// accepted (cost affordable + timing legal). Surfaced in
    /// `PlayerView.suspendable_hand` so the client can offer a "Suspend"
    /// affordance.
    pub fn suspendable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.suspendable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`suspendable_hand_cards`] against a prebuilt probe template; the
    /// caller owns the priority short-circuit.
    ///
    /// [`suspendable_hand_cards`]: Self::suspendable_hand_cards
    fn suspendable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        use crate::card::Keyword;
        self.players[caster]
            .hand
            .iter()
            .filter(|c| {
                c.definition.keywords.iter().any(|k| matches!(k, Keyword::Suspend(..)))
            })
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::Suspend { card_id: id }))
            .collect()
    }

    /// Cards in `caster`'s hand they could Foretell right now (CR 702.143):
    /// the card has a `foretell_cost` and paying {2} at sorcery speed is
    /// legal. Surfaced in `PlayerView.foretellable_hand`.
    pub fn foretellable_hand_cards(&self, caster: usize) -> Vec<CardId> {
        if self.player_with_priority() != caster {
            return Vec::new();
        }
        self.foretellable_hand_cards_on(&self.affordance_probe_template(), caster)
    }

    /// [`foretellable_hand_cards`] against a prebuilt probe template; the
    /// caller owns the priority short-circuit.
    ///
    /// [`foretellable_hand_cards`]: Self::foretellable_hand_cards
    fn foretellable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.foretell_cost.is_some())
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::Foretell { card_id: id }))
            .collect()
    }

    /// Cards in `caster`'s hand they could Plot right now (CR 702.170): the
    /// card has a `plot_cost` and paying it at sorcery speed is legal.
    fn plottable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter(|c| c.definition.plot_cost.is_some())
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::Plot { card_id: id }))
            .collect()
    }

    /// Cards in `caster`'s hand with an Adventure half they could cast right
    /// now (CR 715). The probe auto-targets the adventure effect.
    fn adventurable_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter_map(|c| {
                let adv = c.definition.has_adventure()?;
                let (target, additional_targets) = if adv.effect.requires_target() {
                    let (t, extras) =
                        template.auto_targets_for_effect_all_slots(&adv.effect, caster, None);
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                let id = c.id;
                Self::would_accept_on(
                    template,
                    GameAction::CastAdventure {
                        card_id: id, target, additional_targets, mode: None, x_value: None,
                    },
                )
                .then_some(id)
            })
            .collect()
    }

    /// Split cards in `caster`'s hand whose right half they could cast right
    /// now (CR 709). The probe auto-targets the right half's effect.
    fn splittable_right_hand_cards_on(&self, template: &GameState, caster: usize) -> Vec<CardId> {
        self.players[caster]
            .hand
            .iter()
            .filter_map(|c| {
                let split = c.definition.has_split()?;
                let (target, additional_targets) = if split.right.effect.requires_target() {
                    let (t, extras) = template
                        .auto_targets_for_effect_all_slots(&split.right.effect, caster, None);
                    t.as_ref()?;
                    (t, extras)
                } else {
                    (None, vec![])
                };
                let id = c.id;
                Self::would_accept_on(
                    template,
                    GameAction::CastSplitRight {
                        card_id: id, target, additional_targets, mode: None, x_value: None,
                    },
                )
                .then_some(id)
            })
            .collect()
    }

    /// Compute every from-hand affordance hint for `seat` in one pass.
    ///
    /// The individual `*_hand_cards` / `activatable_permanents` methods each
    /// build their own [`affordance_probe_template`] — fine when called in
    /// isolation (tests, debug export), but the per-seat view projection
    /// needs all of them on every accepted action. Building the template
    /// once here and threading it through the `_on` variants collapses what
    /// was eight independent full-`GameState` clones (plus the per-card
    /// clones) into a single template clone reused across every category.
    ///
    /// Returns all-empty when `seat` doesn't hold priority — the same
    /// short-circuit each individual method applies, hoisted so the template
    /// (and the whole sweep) is skipped entirely off-priority.
    ///
    /// [`affordance_probe_template`]: Self::affordance_probe_template
    pub fn compute_hand_affordances(&self, seat: usize) -> HandAffordances {
        if self.player_with_priority() != seat {
            return HandAffordances::default();
        }
        let template = self.affordance_probe_template();
        HandAffordances {
            castable: self.castable_hand_cards_on(&template, seat),
            // Pitchable is a pure structural filter (no dry-run), so it
            // needs no template and never touches the probe clone.
            pitchable: self.pitchable_hand_cards(seat),
            kickable: self.kickable_hand_cards_on(&template, seat),
            buyback: self.buyback_hand_cards_on(&template, seat),
            bestowable: self.bestowable_hand_cards_on(&template, seat),
            dashable: self.dashable_hand_cards_on(&template, seat),
            blitzable: self.blitzable_hand_cards_on(&template, seat),
            suspendable: self.suspendable_hand_cards_on(&template, seat),
            foretellable: self.foretellable_hand_cards_on(&template, seat),
            plottable: self.plottable_hand_cards_on(&template, seat),
            adventurable: self.adventurable_hand_cards_on(&template, seat),
            splittable_right: self.splittable_right_hand_cards_on(&template, seat),
            bargainable: self.bargainable_hand_cards_on(&template, seat),
            squadable: self.squadable_hand_cards_on(&template, seat),
            replicatable: self.replicatable_hand_cards_on(&template, seat),
            miracle: self.miracle_hand_cards(seat),
            activatable_permanents: self.activatable_permanents_on(&template, seat),
            hand_activatable: self.hand_activatable_cards(seat),
            morphable: self.morphable_hand_cards_on(&template, seat),
            turn_up_able: self.turn_up_able_permanents_on(&template, seat),
            reinforceable: self.reinforceable_hand_cards_on(&template, seat),
        }
    }

    /// CR 702.77 — hand cards `seat` could Reinforce right now: the card carries
    /// `Keyword::Reinforce` and the activation (pay cost, discard, +1/+1 on a
    /// creature) is legal against some creature target. Surfaced in
    /// `PlayerView.reinforceable_hand`.
    fn reinforceable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        let any_creature = self
            .battlefield
            .iter()
            .find(|c| c.definition.is_creature())
            .map(|c| c.id);
        let Some(tid) = any_creature else { return vec![] };
        self.players[seat]
            .hand
            .iter()
            .filter(|c| {
                c.definition
                    .keywords
                    .iter()
                    .any(|k| matches!(k, crate::card::Keyword::Reinforce(_, _)))
            })
            .map(|c| c.id)
            .filter(|&id| {
                Self::would_accept_on(
                    template,
                    GameAction::Reinforce {
                        card_id: id,
                        target: crate::game::types::Target::Permanent(tid),
                    },
                )
            })
            .collect()
    }

    /// CR 708.5 — face-down permanents `seat` controls whose turn-up cost is
    /// payable right now. Dry-run-gated through `would_accept` (which derives
    /// the cost and enforces the manifested-noncreature rule). Surfaced in
    /// `PlayerView.turn_up_able`.
    fn turn_up_able_permanents_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.face_down && c.face_up_def.is_some())
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::TurnFaceUp { card_id: id }))
            .collect()
    }

    /// Cards in `seat`'s hand they could cast face down via Morph right now
    /// (CR 702.36): the card has Morph/Megamorph and paying {3} at sorcery
    /// speed is legal. Surfaced in `PlayerView.morphable_hand`.
    fn morphable_hand_cards_on(&self, template: &GameState, seat: usize) -> Vec<CardId> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| {
                c.definition.keywords.iter().any(|k| {
                    matches!(
                        k,
                        crate::card::Keyword::Morph(_)
                            | crate::card::Keyword::Megamorph(_)
                            | crate::card::Keyword::Disguise(_)
                    )
                })
            })
            .map(|c| c.id)
            .filter(|&id| Self::would_accept_on(template, GameAction::CastFaceDown { card_id: id }))
            .collect()
    }

    /// Hand cards the seat owns that carry at least one `from_hand` activated
    /// ability (Talon Gates' `{4}` put-into-play, Spirit Guides' pitch mana).
    /// A pure structural filter — `activate_ability` re-checks cost/zone — so
    /// it needs no probe clone.
    fn hand_activatable_cards(&self, seat: usize) -> Vec<CardId> {
        self.players[seat]
            .hand
            .iter()
            .filter(|c| c.definition.activated_abilities.iter().any(|a| a.from_hand))
            .map(|c| c.id)
            .collect()
    }

    /// Extra generic mana the caster owes on top of `card`'s printed
    /// cost — Damping Sphere's "+1 after the first spell each turn,"
    /// Chancellor of the Annex's first-spell tax, etc. Public so the
    /// bot's affordability check can match the engine's payment path:
    /// `can_afford` ignoring this tax causes the bot to repeatedly
    /// submit a `CastSpell` that the engine then rejects with a mana
    /// shortfall, which (in spectate mode) deadlocks the match.
    pub fn extra_cost_for_card_in_hand(&self, caster: usize, card_id: CardId) -> u32 {
        let Some(card) = self.players[caster]
            .hand
            .iter()
            .find(|c| c.id == card_id)
        else {
            return 0;
        };
        crate::game::actions::extra_cost_for_spell(self, caster, card)
    }
}

/// True if `effect` is a pure mana ability (CR 605.1a) — it only adds mana
/// and has no other rider. Such abilities don't use the stack and are
/// auto-tapped on demand during cost payment, so the affordance probe
/// excludes them from `activatable_permanents` (a mana dork is not an
/// instant-speed "play" that should hold priority).
fn is_mana_ability_effect(effect: &Effect) -> bool {
    match effect {
        Effect::AddMana { .. } => true,
        Effect::Seq(steps) => !steps.is_empty() && steps.iter().all(is_mana_ability_effect),
        _ => false,
    }
}
