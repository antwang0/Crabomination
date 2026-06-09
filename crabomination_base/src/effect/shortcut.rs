//! Common one-liner constructors for building card definitions tersely.
use super::*;

pub fn you() -> Selector { Selector::You }
pub fn this() -> Selector { Selector::This }
pub fn target() -> Selector { Selector::Target(0) }
pub fn target_n(n: u8) -> Selector { Selector::Target(n) }
pub fn target_filtered(filter: SelectionRequirement) -> Selector {
    Selector::TargetFiltered { slot: 0, filter }
}
/// "Any target" — creature, player, or planeswalker. The canonical
/// burn-spell target filter (Lightning Bolt, Shock, Lightning
/// Strike), pulled out into a helper so cards don't have to spell
/// out the 3-way `or` inline. Push (claude/modern_decks, batches
/// 192-197).
pub fn target_any() -> Selector {
    target_filtered(
        SelectionRequirement::Creature
            .or(SelectionRequirement::Player)
            .or(SelectionRequirement::Planeswalker),
    )
}
pub fn trigger_source() -> Selector { Selector::TriggerSource }

pub fn each_creature() -> Selector {
    Selector::EachPermanent(SelectionRequirement::Creature)
}
pub fn each_your_creature() -> Selector {
    Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    )
}
pub fn each_opponent_creature() -> Selector {
    Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
    )
}
pub fn each_opponent() -> Selector { Selector::Player(PlayerRef::EachOpponent) }

pub fn n(x: i32) -> Value { Value::Const(x) }
pub fn count(s: Selector) -> Value { Value::count(s) }

pub fn deal(amount: i32, to: Selector) -> Effect {
    Effect::DealDamage { to, amount: Value::Const(amount) }
}
pub fn gain_life(amount: i32) -> Effect {
    Effect::GainLife { who: you(), amount: Value::Const(amount) }
}
/// Canonical Witherbloom / Silverquill drain shape: "each opponent
/// loses N life, you gain N life." Returns the raw `Effect::Drain`
/// value so it can compose with `Seq`, `MayDo`, or be used as the
/// body of a spell directly.
pub fn drain(amount: i32) -> Effect {
    Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: you(),
        amount: Value::Const(amount),
    }
}
pub fn lose_life(amount: i32, who: Selector) -> Effect {
    Effect::LoseLife { who, amount: Value::Const(amount) }
}
pub fn draw(n: i32) -> Effect {
    Effect::Draw { who: you(), amount: Value::Const(n) }
}
pub fn discard(who: Selector, n: i32, random: bool) -> Effect {
    Effect::Discard { who, amount: Value::Const(n), random }
}
pub fn destroy_target() -> Effect { Effect::Destroy { what: target() } }
/// "Destroy target ... It can't be regenerated." (CR 701.15g)
pub fn destroy_target_no_regen() -> Effect {
    Effect::DestroyNoRegen { what: target() }
}
pub fn exile_target() -> Effect { Effect::Exile { what: target() } }
pub fn return_target_to_hand() -> Effect {
    Effect::Move { what: target(), to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(target()))) }
}
pub fn pump_target(power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what: target(),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}
pub fn counter_target_spell() -> Effect {
    Effect::CounterSpell {
        what: target_filtered(SelectionRequirement::IsSpellOnStack),
    }
}
pub fn add_mana(colors: Vec<Color>) -> Effect {
    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(colors) }
}
pub fn add_colorless(n: i32) -> Effect {
    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(n)) }
}
pub fn add_any_one_color(n: i32) -> Effect {
    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(n)) }
}

/// Dash (CR 702.110) alternative cost: cast for `mana_cost`; the creature
/// gains haste and returns to its owner's hand at the next end step.
pub fn dash(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, dash: true, ..Default::default() }
}

/// Blitz (CR 702.152) alternative cost: cast for `mana_cost`; the creature
/// gains haste and "When this creature dies, draw a card," then is
/// sacrificed at the beginning of the next end step.
pub fn blitz(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, blitz: true, ..Default::default() }
}

/// Evoke (CR 702.74) alternative cost: cast for `mana_cost`; the creature is
/// sacrificed when it enters, after its ETB triggers resolve.
pub fn evoke(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, evoke_sacrifice: true, ..Default::default() }
}

/// Impending N—[cost] (CR 702.183): cast for `mana_cost`; the permanent enters
/// with `n` time counters and isn't a creature until they tick off (one per
/// controller's upkeep). Pair with `Keyword::Impending(n)` on the card so the
/// layer + upkeep machinery recognizes it.
pub fn impending(n: u32, mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, impending: n, ..Default::default() }
}

/// Surge (CR 702.108) alternative cost: cast for `mana_cost` if you or a
/// teammate cast another spell this turn. `with_rider` stamps the spell
/// "kicked" so "if its surge cost was paid" ETB riders fire.
pub fn surge(mana_cost: crate::mana::ManaCost, with_rider: bool) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost,
        condition: Some(Predicate::SpellsCastThisTurnAtLeast {
            who: PlayerRef::You,
            at_least: Value::Const(1),
        }),
        marks_kicked: with_rider,
        ..Default::default()
    }
}

/// The "animate a land" rider shared by Awaken (CR 702.113) and
/// Wall of Resurgence / Cyclone Sire: put `counters` +1/+1 counters on the
/// land in target slot `slot` and it becomes a 0/0 Elemental creature with
/// haste that's still a land.
pub fn animate_land(slot: u8, counters: i32) -> Effect {
    use crate::card::CreatureType;
    let land = SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou);
    Effect::Seq(vec![
        Effect::AddCounter {
            what: Selector::TargetFiltered { slot, filter: land },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(counters),
        },
        Effect::BecomeCreature {
            what: Selector::Target(slot),
            power: Value::Const(0),
            toughness: Value::Const(0),
            creature_types: vec![CreatureType::Elemental],
            keywords: vec![Keyword::Haste],
            duration: Duration::Permanent,
        },
    ])
}

/// Awaken N—`mana_cost` (CR 702.113) alternative cost: cast for `mana_cost`;
/// the spell resolves its `base_effect` and additionally animates the land in
/// target slot `land_slot` with N +1/+1 counters. `base_effect` keeps its own
/// target slots (0..land_slot); the land occupies `land_slot`.
pub fn awaken(
    n: i32,
    mana_cost: crate::mana::ManaCost,
    land_slot: u8,
    base_effect: Effect,
) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost,
        effect_override: Some(Effect::Seq(vec![base_effect, animate_land(land_slot, n)])),
        ..Default::default()
    }
}

/// Emerge (CR 702.119) alternative cost: cast for `mana_cost` by sacrificing a
/// creature you control, reducing the cost generically by that creature's MV.
pub fn emerge(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost,
        emerge: Some(SelectionRequirement::Creature),
        impending: 0,
        ..Default::default()
    }
}

/// Spectacle (CR 702.111) alternative cost: cast for `mana_cost` rather
/// than the printed cost if an opponent lost life this turn.
pub fn spectacle(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost,
        condition: Some(Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent }),
        ..Default::default()
    }
}

/// "[Permanents matching `filter`] have '{T}: Add one mana of any
/// color.'" — the mana-dork-anthem static shared by Galazeth Prismari
/// (artifacts), Cryptolith Rite (creatures), Resonating Lute (lands).
pub fn grant_tap_for_any_color(filter: SelectionRequirement) -> StaticAbility {
    StaticAbility {
        description: "{T}: Add one mana of any color.",
        effect: StaticEffect::GrantActivatedAbility {
            applies_to: Selector::EachPermanent(filter),
            ability: ActivatedAbility {
                energy_cost: 0,
                tap_cost: true,
                effect: add_any_one_color(1),
                ..Default::default()
            },
        },
    }
}

/// "[Permanents matching `filter`] have '{T}: Add `count` mana of any
/// one color. Spend this mana only to …'" — the spend-restricted
/// sibling of [`grant_tap_for_any_color`], used by Resonating Lute.
pub fn grant_tap_for_any_color_restricted(
    filter: SelectionRequirement,
    count: i32,
    restriction: SpendRestriction,
) -> StaticAbility {
    StaticAbility {
        description: "{T}: Add mana of any one color (instants/sorceries only).",
        effect: StaticEffect::GrantActivatedAbility {
            applies_to: Selector::EachPermanent(filter),
            ability: ActivatedAbility {
                energy_cost: 0,
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::Const(count))),
                        restriction,
                    ),
                },
                ..Default::default()
            },
        },
    }
}

pub fn etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect,
    }
}

/// Raid (CR 702.108): "When this enters, if you attacked this turn, `body`."
/// An ETB trigger gated on `Predicate::PlayerAttackedThisTurn { You }`.
pub fn raid_etb(body: Effect) -> TriggeredAbility {
    etb(Effect::If {
        cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
        then: Box::new(body),
        else_: Box::new(Effect::Noop),
    })
}
pub fn on_attack(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect,
    }
}
/// Revolt (CR 702.139): "When this enters, if a permanent left the
/// battlefield under your control this turn, `body`." An ETB trigger
/// gated on `Predicate::RevoltActive { You }`. Models "enters with a
/// +1/+1 counter if Revolt" as an ETB add-counter (the counter lands
/// just after the creature enters rather than as a true ETB replacement).
pub fn revolt_etb(body: Effect) -> TriggeredAbility {
    etb(Effect::If {
        cond: Predicate::RevoltActive { who: PlayerRef::You },
        then: Box::new(body),
        else_: Box::new(Effect::Noop),
    })
}
/// CR 702.171 — "Whenever this Mount attacks while saddled, [effect]."
/// The `SourceSaddled` filter gates the trigger on the Mount's saddled
/// state (set by a Saddle activation earlier in the turn).
pub fn attacks_while_saddled(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::SourceSaddled),
        effect,
    }
}
/// CR 702.39 — Provoke: "Whenever this attacks, you may have target
/// creature defending player controls untap and block it this combat
/// if able." (The "you may" collapses — the attacker provokes a legal
/// opponent creature when one exists.)
pub fn provoke() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::Provoke {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        },
    }
}
pub fn on_dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect,
    }
}

/// "When you cast this spell, `effect`." A cast trigger resolving above
/// the spell on the stack (CR 601.2 / the Eldrazi titans). Targeted bodies
/// pick their target via `target_filtered` as the trigger goes on the stack.
pub fn on_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
        effect,
    }
}

/// On-other-dies shortcut: "Whenever another creature you control
/// dies, `effect`." Wraps the `CreatureDied / AnotherOfYours` event
/// scope, which excludes the source's own death. Used by Pest
/// Hivewatcher (batch 119), Inkling Confessor's lifegain rider,
/// Felisa Fang of Silverquill, and any future "another creature
/// dies" payoff that doesn't need a creature-type filter.
pub fn on_other_dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
        effect,
    }
}

/// Enrage shortcut (CR 702.130): "Whenever this creature is dealt
/// damage, `effect`." Wraps the `DealtDamage / SelfSource` event,
/// which fires on any damage to the source — combat, burn, Fight,
/// or pingers. The damage amount is reachable inside `effect` via
/// `Value::TriggerEventAmount` for scaling enrage payoffs.
pub fn enrage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
        effect,
    }
}

/// Exalted shortcut (CR 702.83): "Whenever a creature you control
/// attacks alone, that creature gets +1/+1 until end of turn." Wraps
/// an `Attacks / YourControl` trigger gated on the new
/// `Predicate::AttackingAlone`; the pump targets the lone attacker
/// (`Selector::TriggerSource`), not the Exalted source, so multiple
/// Exalted permanents stack on the same lone attacker per 702.83b.
pub fn exalted() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
            .with_filter(Predicate::AttackingAlone),
        effect: Effect::PumpPT {
            what: Selector::TriggerSource,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
    }
}

/// CR 509.1 — "Whenever this creature blocks a creature, `body`." A
/// `Blocks / SelfSource` trigger; `body` typically operates on
/// `Selector::BlockedAttacker` (the creature this is blocking) — Wall of
/// Frost.
pub fn blocks(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
        effect: body,
    }
}

/// Battalion shortcut (ability word): "Whenever this creature and at
/// least two other creatures attack, `body`." An `Attacks / SelfSource`
/// trigger gated on `Predicate::AttackingWithAtLeast(3)`.
pub fn battalion(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::AttackingWithAtLeast(3)),
        effect: body,
    }
}

/// Battle Cry shortcut (CR 702.92): "Whenever this creature attacks,
/// each *other* attacking creature gets +`amount`/+0 until end of turn."
/// An `Attacks / SelfSource` trigger that pumps every attacking
/// permanent except the source (`IsAttacking ∧ OtherThanSource`).
pub fn battle_cry(amount: i32) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::IsAttacking
                    .and(SelectionRequirement::OtherThanSource),
            ),
            power: Value::Const(amount),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    }
}

/// Training shortcut (CR 702.149): "Whenever this creature attacks with
/// another creature with greater power, put a +1/+1 counter on this
/// creature." An `Attacks / SelfSource` trigger gated on the existence
/// of another attacking creature with `PowerGreaterThanSource`; the
/// counter lands on `This`.
pub fn training() -> TriggeredAbility {
    use crate::card::CounterType;
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                SelectionRequirement::IsAttacking
                    .and(SelectionRequirement::OtherThanSource)
                    .and(SelectionRequirement::PowerGreaterThanSource),
            )),
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// Backup N shortcut (CR 702.164): "When this creature enters, put N
/// +1/+1 counters on target creature. If that's another creature, it
/// gains the granted keywords until end of turn." An `EntersBattlefield /
/// SelfSource` trigger targeting a creature: the counters land on the
/// target, and each `granted` keyword is granted until end of turn
/// (idempotent when the target is the source — it already prints them).
pub fn backup(n: i32, granted: Vec<Keyword>) -> TriggeredAbility {
    backup_with(n, granted, vec![])
}

/// Backup N (CR 702.164) that also grants the source's *triggered*
/// abilities to a backed-up other creature until end of turn. Keyword
/// grants are idempotent on the source, but trigger grants are gated on
/// the target being another creature (else the source would double-fire
/// its own printed trigger when it targets itself). Bola Slinger
/// (granted "whenever this attacks, tap target opponent permanent").
pub fn backup_with(
    n: i32,
    granted: Vec<Keyword>,
    triggers: Vec<TriggeredAbility>,
) -> TriggeredAbility {
    use crate::card::CounterType;
    let target = || Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Creature };
    let mut steps = vec![Effect::AddCounter {
        what: target(),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    }];
    for kw in granted {
        steps.push(Effect::GrantKeyword {
            what: target(),
            keyword: kw,
            duration: Duration::EndOfTurn,
        });
    }
    for t in triggers {
        steps.push(Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::Target(0),
                filter: SelectionRequirement::OtherThanSource,
            },
            then: Box::new(Effect::GrantTriggeredAbility {
                what: target(),
                trigger: Box::new(t),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        });
    }
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(steps),
    }
}

/// Evolve shortcut (CR 702.100): "Whenever a creature enters the
/// battlefield under your control, if that creature has greater power
/// or toughness than this creature, put a +1/+1 counter on this
/// creature." An `EntersBattlefield / YourControl` trigger gated on
/// the entering creature (`TriggerSource`) being another creature with
/// `GreaterPowerOrToughnessThanSource`; the counter lands on `This`.
pub fn evolve() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::OtherThanSource)
                    .and(SelectionRequirement::GreaterPowerOrToughnessThanSource),
            }),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    }
}

/// ETB-Drain shortcut: "When this creature enters, each opponent loses
/// `amount` life and you gain `amount` life." Wraps [`etb`] with the
/// canonical drain-each-opp body. Used by ~40 STX/SOS Silverquill /
/// Witherbloom drain creatures (Inkling Stormcaller, Silverquill
/// Drainmaster, Inkling Magister, etc.) to collapse the recurring
/// 7-line trigger pattern into one helper call.
pub fn etb_drain(amount: i32) -> TriggeredAbility {
    etb(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// ETB-Gain-Life shortcut: "When this creature enters, you gain
/// `amount` life." Wraps [`etb`] with the canonical gain-life body.
/// Used by ~25 STX/SOS Silverquill / Lorehold lifegain creatures
/// (Silverquill Marshal, Silverquill Loremender, Lorehold
/// Skydefender, etc.).
pub fn etb_gain_life(amount: i32) -> TriggeredAbility {
    etb(Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// Dies-Gain-Life shortcut: "When this creature dies, you gain
/// `amount` life." Wraps [`on_dies`] with the canonical gain-life
/// body. Used by the Pest token cycle (1/1 with on-die gain 1) and
/// any future "when this creature dies, you gain N life" cards
/// (Selfless Spirit's death rider, Resilient Khenra-class).
pub fn dies_gain_life(amount: i32) -> TriggeredAbility {
    on_dies(Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// Dies-Drain shortcut: "When this creature dies, each opponent
/// loses `amount` life and you gain `amount` life." Mirrors
/// [`etb_drain`] for the on-death event, used by aristocrats-style
/// payoffs where the source itself dies (Witherbloom Saproot,
/// Witherbloom Reaper-Hand, Witherbloom Drainbreath templates).
pub fn dies_drain(amount: i32) -> TriggeredAbility {
    on_dies(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// ETB-Mill-Each-Opp shortcut: "When this creature enters, each
/// opponent mills `amount` cards." Wraps [`etb`] with the
/// canonical opponent-mill body. Useful for delirium / graveyard-
/// matters payoffs that put opp cards into their own graveyard
/// (Witherbloom Tomeshade template).
pub fn etb_mill_each_opp(amount: i32) -> TriggeredAbility {
    etb(Effect::Mill {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(amount),
    })
}

/// ETB-Drain-Each-Opp shortcut: "When this creature enters, each
/// opponent loses `amount` life." This is the asymmetric variant of
/// [`etb_drain`] — opponents lose life but you do *not* gain any.
/// Used by point-drain bodies like Witherbloom Toxinspeaker and
/// Silverquill Drainscholar where the printed text omits the
/// you-gain rider.
pub fn etb_drain_each_opp(amount: i32) -> TriggeredAbility {
    etb(Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(amount),
    })
}

/// ETB-Loot shortcut: "When this creature enters, draw a card,
/// then discard a card." Wraps [`etb`] with the canonical loot
/// body. Used by ~10 STX/SOS Prismari / Witherbloom loot creatures
/// (Prismari Cinderpoet, Prismari Stormbearer) to collapse the
/// recurring 6-line Seq into one helper call.
pub fn etb_loot() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::Const(1),
            random: false,
        },
    ]))
}

/// Predicate matching "the just-cast spell is an instant or a sorcery".
/// Built around `Selector::TriggerSource` — at the spell-cast site,
/// `fire_spell_cast_triggers` binds the just-cast `CardId` to
/// TriggerSource for the duration of filter evaluation, so a
/// `Predicate::EntityMatches { what: TriggerSource, filter: … }` reads
/// the cast spell.
pub fn cast_is_instant_or_sorcery() -> Predicate {
    Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: SelectionRequirement::HasCardType(crate::card::CardType::Instant)
            .or(SelectionRequirement::HasCardType(crate::card::CardType::Sorcery)),
    }
}

/// Strixhaven Magecraft trigger: "Whenever you cast or copy an instant
/// or sorcery spell, `effect`." Bundles the spell-cast trigger with
/// the [`cast_is_instant_or_sorcery`] predicate. Used by Eager
/// First-Year, Witherbloom Apprentice, etc.
pub fn magecraft(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_instant_or_sorcery()),
        effect,
    }
}

/// "Whenever you cast a colorless spell, `effect`." (Kozilek's Sentinel.)
/// `SelectionRequirement::Colorless` reads cost pips, so genuinely colorless
/// (generic-cost) spells match; Devoid spells with colored pips slip through
/// (the known Devoid/colorless-filter gap tracked in TODO.md).
pub fn cast_colorless(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Colorless,
            },
        ),
        effect,
    }
}

/// Cascade (CR 702.85). Wires the standard "when you cast this spell"
/// (`SpellCast` / `SelfSource`) trigger whose body is
/// [`Effect::Cascade`]. `mv` is the cascading spell's printed mana
/// value — the gate is "exile until a nonland card with MV < `mv`".
pub fn cascade(mv: u32) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
        effect: Effect::Cascade {
            max_mv: Value::Const(mv as i32),
        },
    }
}

/// Squad (CR 702.157) — the ETB trigger that mints one token copy of this
/// creature per time its squad cost was paid (`Value::SquadCount`). Pair with
/// `Keyword::Squad(cost)` on the card.
pub fn squad_etb() -> TriggeredAbility {
    etb(Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::SquadCount,
        source: Selector::This,
        extra_creature_types: Vec::new(),
        override_pt: None,
        non_legendary: false,
    })
}

/// Demonstrate (CR 702.150) — the self-cast trigger that copies the spell for
/// its caster and an opponent (each copy may choose new targets). Attach to a
/// card's `triggered_abilities`; see `Effect::Demonstrate`.
pub fn demonstrate() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
        effect: Effect::Demonstrate,
    }
}

/// Strixhaven Repartee trigger: "Whenever you cast an instant or sorcery
/// spell that targets a creature, `effect`." Bundles the magecraft
/// filter (instant or sorcery) with `Predicate::CastSpellTargetsMatch`
/// (target is a creature). The spell's chosen target is read from the
/// cast-time `StackItem::Spell.target` slot — Repartee fires only when
/// the target is currently a creature.
pub fn repartee(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::All(vec![
                cast_is_instant_or_sorcery(),
                Predicate::CastSpellTargetsMatch(SelectionRequirement::Creature),
            ]),
        ),
        effect,
    }
}

/// Convenience: a Magecraft trigger that pumps the source itself.
/// Wraps [`magecraft`] with a `PumpPT` body whose `what:` is the
/// triggering permanent (`Selector::This`). Used by self-pump
/// magecraft creatures (Symmetry Sage's +1/+0; future Witherbloom /
/// Lorehold apprentices) so the call site stays one line. Duration
/// defaults to end-of-turn since every printed magecraft self-pump
/// uses that duration.
pub fn magecraft_self_pump(power: i32, toughness: i32) -> TriggeredAbility {
    magecraft(Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    })
}

/// Convenience: a Repartee trigger that pumps the source itself.
/// Same shape as [`magecraft_self_pump`] but gated on the additional
/// "spell targets a creature" Repartee predicate. Used by Rehearsed
/// Debater (current SOS catalog), and any future Repartee creature
/// that scales with cast events targeting a creature.
pub fn repartee_self_pump(power: i32, toughness: i32) -> TriggeredAbility {
    repartee(Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    })
}

/// Convenience: a Magecraft trigger that untaps the source itself.
pub fn magecraft_self_untap() -> TriggeredAbility {
    magecraft(Effect::Untap {
        what: Selector::This,
        up_to: None,
    })
}

/// Classic Innistrad werewolf day-side transform: "At the beginning of each
/// upkeep, if no spells were cast last turn, transform this creature."
pub fn werewolf_day_transform() -> TriggeredAbility {
    use crate::turn_step::TurnStep;
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
            .with_filter(Predicate::NoSpellsCastLastTurn),
        effect: Effect::Transform { what: Selector::This },
    }
}

/// Classic Innistrad werewolf night-side transform back: "At the beginning of
/// each upkeep, if a player cast two or more spells last turn, transform this
/// creature."
pub fn werewolf_night_transform() -> TriggeredAbility {
    use crate::turn_step::TurnStep;
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
            .with_filter(Predicate::TwoOrMoreSpellsCastLastTurn),
        effect: Effect::Transform { what: Selector::This },
    }
}

/// Convenience: a Magecraft trigger that drains `amount` life from
/// each opponent into the controller.
pub fn magecraft_drain_each_opp(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft-Drain-Target shortcut for per-target drain cards
/// (Promising Duskmage, Inkling Coursebinder, etc.).
pub fn magecraft_drain_target(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Drain {
        from: Selector::TargetFiltered {
            slot: 0,
            filter: SelectionRequirement::Player,
        },
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// ETB-Pump-Each-with-Type shortcut: "When this creature enters,
/// put a +1/+1 counter on each creature you control of the given type."
pub fn etb_pump_each_with_type(creature_type: crate::card::CreatureType) -> TriggeredAbility {
    use crate::card::CounterType;
    etb(Effect::ForEach {
        selector: Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::HasCreatureType(creature_type))
                .and(SelectionRequirement::ControlledByYou),
        ),
        body: Box::new(Effect::AddCounter {
            what: Selector::TriggerSource,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        }),
    })
}

/// Predicate matching "it's your turn and the current step is a main phase"
/// — the resolution-time test for Addendum (CR 702.124): a spell cast during
/// your main phase resolves during that same step, so this is exact.
pub fn cast_during_your_main() -> Predicate {
    use crate::turn_step::TurnStep;
    Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::Any(vec![
            Predicate::CurrentStepIs(TurnStep::PreCombatMain),
            Predicate::CurrentStepIs(TurnStep::PostCombatMain),
        ]),
    ])
}

/// Addendum (CR 702.124): run `base`, then — if the spell was cast during the
/// caster's main phase — also run `bonus`.
pub fn addendum(base: Effect, bonus: Effect) -> Effect {
    Effect::Seq(vec![
        base,
        Effect::If {
            cond: cast_during_your_main(),
            then: Box::new(bonus),
            else_: Box::new(Effect::Noop),
        },
    ])
}

/// Predicate matching "the just-cast spell is a noncreature spell".
pub fn cast_is_noncreature() -> Predicate {
    Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: SelectionRequirement::HasCardType(crate::card::CardType::Creature).negate(),
    }
}

/// Prowess trigger: "Whenever you cast a noncreature spell, this creature
/// gets +1/+1 until end of turn."
pub fn prowess_trigger() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_noncreature()),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
    }
}

/// Strixhaven Quandrix "spell with `{X}` in its mana cost" trigger:
/// fires on any spell cast by the controller whose printed cost
/// contains an `{X}` symbol. Powered by `Predicate::CastSpellHasX`.
/// Used by Geometer's Arthropod, Matterbending Mage, and any future
/// Quandrix card that pays off X-cost spells.
pub fn cast_has_x_trigger(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellHasX),
        effect,
    }
}

/// Prowess trigger: "Whenever you cast a noncreature spell, this
/// creature gets +1/+1 until end of turn." Fires on every cast you
/// control whose card type is **not** Creature (the printed Prowess
/// keyword's reminder text). The pumped target is the source itself
/// via `Selector::This`, so a single Prowess creature can drop the
/// helper in one line and the trigger source is correctly threaded.
///
/// Wired into card factories declaring `Keyword::Prowess` —
/// Spectacle Mage, Eccentric Apprentice, etc. — to convert the
/// keyword tag into a functional trigger. (The keyword itself
/// remains in `card.keywords` for display + future "Prowess matters"
/// payoffs to filter on.)
pub fn prowess() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCardType(crate::card::CardType::Creature)
                    .negate(),
            },
        ),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        },
    }
}

/// SOS Increment trigger: "Whenever you cast a spell, if the amount
/// of mana you spent is greater than this creature's power or
/// toughness, [body]." Powered by `Predicate::IncrementSatisfied`,
/// which compares the just-cast spell's stashed `mana_spent` to the
/// listening permanent's effective P/T. The canonical Increment
/// payoff drops a +1/+1 counter on `Selector::This`, but the helper
/// is body-agnostic so cards like Pensive Professor (gain a +1/+1
/// counter and scry 1) can plug arbitrary effects in.
///
/// Implements MTG comp rules 603.4 ("intervening 'if' clause"): the
/// `IncrementSatisfied` predicate is checked both at trigger-event
/// time (the `EventSpec.filter` gate, controlling whether the
/// trigger goes on the stack) AND at resolution time (the wrapping
/// `Effect::If`, controlling whether the body actually runs). If
/// the source gains counters after this trigger goes on the stack
/// but before it resolves, the resolution-time check can suppress
/// the body even though the trigger fired.
pub fn increment_trigger(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::IncrementSatisfied),
        effect: Effect::If {
            cond: Predicate::IncrementSatisfied,
            then: Box::new(effect),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// SOS Increment payoff that drops one +1/+1 counter on the source.
/// Wraps [`increment_trigger`] with the standard `AddCounter` body
/// targeting `Selector::This`. Used by Cuboid Colony / Fractal
/// Tender / Berta and every other vanilla-Increment creature.
pub fn increment_self_plus_one() -> TriggeredAbility {
    use crate::card::CounterType;
    increment_trigger(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    })
}

/// Strixhaven Opus payoff trigger: "Whenever you cast an instant or
/// sorcery spell, [body]. If five or more mana was spent to cast
/// that spell, [bigger body] instead." Emits an `If`-gated effect
/// whose `Predicate::CastSpellManaSpentAtLeast(5)` arm fires the
/// bigger payoff. Used by Deluge Virtuoso, Expressive Firedancer,
/// Magmablood Archaic and other Opus creatures.
pub fn opus_trigger(small_body: Effect, big_body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_instant_or_sorcery()),
        effect: Effect::If {
            cond: Predicate::CastSpellManaSpentAtLeast(5),
            then: Box::new(big_body),
            else_: Box::new(small_body),
        },
    }
}

/// Convenience: "Create a [token] with [keyword] until [duration]."
/// Mints `count` copies of `token`, then grants `keyword` to the
/// last-created token batch for `duration`. Used by Lorehold Skirmish
/// (mint Spirit + grant Haste EOT) and similar mint-then-pump shapes.
/// Wraps the explicit `Seq([CreateToken, GrantKeyword(LastCreatedToken, …)])`
/// pattern at a single call site for clarity.
pub fn create_token_with_keyword(
    who: PlayerRef,
    count: i32,
    token: crate::card::TokenDefinition,
    keyword: crate::card::Keyword,
    duration: Duration,
) -> Effect {
    Effect::Seq(vec![
        Effect::CreateToken {
            who,
            count: Value::Const(count),
            definition: token,
        },
        Effect::GrantKeyword {
            what: Selector::LastCreatedToken,
            keyword,
            duration,
        },
    ])
}

/// Convenience: "Create a [token] with N [counter] counters on it."
/// Mints `count` copies of `token`, then drops `counter_n` copies of
/// `counter` on the last-created token batch. Used by Quandrix
/// Summoner (mint Fractal + add +1/+1 counter), Fractal Harvest
/// (mint Fractal + 3 +1/+1 counters), and any "create a Fractal /
/// Phyrexian / generic token with N counters" pattern.
pub fn create_token_with_counter(
    who: PlayerRef,
    count: i32,
    token: crate::card::TokenDefinition,
    counter: crate::card::CounterType,
    counter_n: i32,
) -> Effect {
    Effect::Seq(vec![
        Effect::CreateToken {
            who,
            count: Value::Const(count),
            definition: token,
        },
        Effect::AddCounter {
            what: Selector::LastCreatedToken,
            kind: counter,
            amount: Value::Const(counter_n),
        },
    ])
}

/// Convenience: Magecraft trigger pumping any chosen target.
/// Wraps [`magecraft`] with a `PumpPT` body whose `what:` is
/// caller-supplied. Used for patterns like Withergrowth Apprentice
/// (magecraft → +1/+1 EOT to target friendly creature) or Quandrix
/// Scholar-style "magecraft → pump target friendly creature". The
/// caller passes a `target_filtered(...)` selector so the auto-target
/// picker still gets a chance to choose at trigger-resolve time.
pub fn magecraft_target_pump(
    what: Selector,
    power: i32,
    toughness: i32,
) -> TriggeredAbility {
    magecraft(Effect::PumpPT {
        what,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    })
}

/// Convenience: Magecraft trigger that pumps every controlled
/// creature of a given tribe (e.g. Spirit / Pest / Inkling /
/// Fractal) by `(power, toughness)` until end of turn. Wraps the
/// canonical `ForEach(Creature ∧ HasCreatureType(t) ∧ ControlledByYou)
/// → PumpPT` body used by Spirit Bannerer-template cards in a single
/// helper call. Tribal-bannerer drop-in for any college.
pub fn magecraft_pump_each_creature_type(
    creature_type: crate::card::CreatureType,
    power: i32,
    toughness: i32,
) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    magecraft(Effect::PumpPT {
        what: Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(creature_type)
                .and(SelectionRequirement::ControlledByYou),
        ),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    })
}

/// Convenience: Magecraft trigger dealing `amount` damage to any
/// chosen target (Creature ∨ Player ∨ Planeswalker). Wraps
/// [`magecraft`] with a `DealDamage` body whose target is a
/// generic "any target" selector. Used by ~20 STX cards
/// (Lorehold Apprentice, Bombastic Strixhaven Mage's magecraft half,
/// Prismari Pyrowriter, Reverberator, Strikevanguard, Sparkmage,
/// etc.) to collapse the recurring 6-line pattern into one line.
pub fn magecraft_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    magecraft(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
        ),
        amount: Value::Const(amount),
    })
}

/// Convenience: Magecraft trigger dealing `amount` damage to each
/// opponent. The drain-burn template for Prismari/Lorehold ping-each-
/// opp creatures (Lorehold Pyrescribe, Pyrosage, Bombastic spell-
/// slingers).
pub fn magecraft_ping_each_opp(amount: i32) -> TriggeredAbility {
    magecraft(Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(amount),
    })
}

/// Convenience: Magecraft trigger gaining `amount` life. Used by
/// Silverquill Lifeglyph / Spectrescribe / Witness / Vinetender style
/// "gain N life on each IS cast" payoffs.
pub fn magecraft_gain_life(amount: i32) -> TriggeredAbility {
    magecraft(Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// ETB-Mint-Token shortcut: "When this creature enters, create
/// `count` copies of `definition`." Wraps [`etb`] with the
/// canonical create-token body. Replaces the 7-line trigger
/// boilerplate at the call site with a one-liner; pairs nicely
/// with the existing `inkling_token()`, `lorehold_spirit_token()`,
/// `stx_pest_token()`, `treasure_token()` factory helpers.
pub fn etb_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    etb(Effect::CreateToken {
        who: PlayerRef::You,
        definition,
        count: Value::Const(count),
    })
}

/// ETB-Scry shortcut: "When this creature enters, scry `amount`."
/// Wraps [`etb`] with the canonical scry body. Used by Witherbloom
/// Cauldronkeeper / Quandrix Symmetrist / Silverquill Bookbearer
/// / Silverquill Archivist / Inkling Treasurer-style "scry on ETB"
/// bodies.
pub fn etb_scry(amount: i32) -> TriggeredAbility {
    etb(Effect::Scry {
        who: PlayerRef::You,
        amount: Value::Const(amount),
    })
}

/// CR 701.40 — "this permanent explores." Reveal the top card of your
/// library; if it's a land, put it into your hand, otherwise put a
/// +1/+1 counter on the exploring permanent.
pub fn explore() -> Effect {
    Effect::Explore { who: Selector::This }
}

/// CR 701.13 — "Investigate `n`": create `n` colorless Clue artifact
/// tokens (`{2}, Sacrifice this artifact: Draw a card.`).
pub fn investigate(n: u32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n as i32),
        definition: crate::tokens::clue_token(),
    }
}

/// CR 702.142 — "Boast — `cost`: `effect`." Activate only if this
/// creature attacked this turn, and only once each turn.
pub fn boast(cost: crate::mana::ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        mana_cost: cost,
        effect,
        once_per_turn: true,
        condition: Some(Predicate::SourceAttackedThisTurn),
        ..Default::default()
    }
}

/// CR 701.31 — "`cost`: Monstrosity `n`." A sorcery-speed activated
/// ability that grows the source to monstrous once.
pub fn monstrosity(cost: crate::mana::ManaCost, n: i32) -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        mana_cost: cost,
        effect: Effect::Monstrosity { n: Value::Const(n) },
        sorcery_speed: true,
        ..Default::default()
    }
}

/// "When this becomes monstrous, …" trigger (CR 701.31).
pub fn on_becomes_monstrous(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecameMonstrous, EventScope::SelfSource),
        effect,
    }
}

/// ETB-explore: "When this creature enters, it explores." Merfolk
/// Branchwalker / Tishana's Wayfinder style bodies. Chain `Seq` two of
/// these (or wrap one in a count) for "explores twice" cards.
pub fn etb_explore() -> TriggeredAbility {
    etb(explore())
}

/// ETB-Draw shortcut: "When this creature enters, draw `amount`
/// cards." Wraps [`etb`] with the canonical draw body. Used by
/// Spirited Companion / Elvish Visionary style cantrip ETB bodies.
pub fn etb_draw(amount: i32) -> TriggeredAbility {
    etb(Effect::Draw {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft-Loot shortcut: "Whenever you cast or copy an instant or
/// sorcery spell, draw a card, then discard a card." Wraps
/// [`magecraft`] with the canonical loot body (Seq[Draw 1, Discard 1]).
/// Used by Prismari Looter / Storm-Caller / Stormcaster / Aquamancer.
pub fn magecraft_loot() -> TriggeredAbility {
    magecraft(Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::Const(1),
            random: false,
        },
    ]))
}

/// Magecraft-Scry shortcut: "Whenever you cast or copy an instant or
/// sorcery spell, scry `amount`." Wraps [`magecraft`] with the
/// canonical scry body. Used by Silverquill Pen-Pusher,
/// Quandrix Mistshaper, etc. — the "smooth on cast" pattern.
pub fn magecraft_scry(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Scry {
        who: PlayerRef::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft-Mint-Token shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create `count` copies of `definition`."
/// Wraps [`magecraft`] with a `CreateToken` body. Used by Inkling
/// Penmaster / Witherbloom Pestmancer / Prismari Alchemist /
/// Sedgemoor Witch-style "magecraft → mint a token" payoffs.
pub fn magecraft_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        definition,
        count: Value::Const(count),
    })
}

/// Magecraft-mint-and-drain shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create `count` of `definition`, then each
/// opponent loses `amount` life and you gain `amount` life." A composite
/// of [`magecraft_mint_token`] and [`magecraft_drain`] for the spells-
/// matter Pest-aristocrats shape (mint a body, drain the table per
/// spell). The `Seq` mints before draining so the freshly-created token
/// is on the battlefield by the time any "if you gained life" / sacrifice
/// payoff sees the drain.
pub fn magecraft_mint_and_drain(
    definition: crate::card::TokenDefinition,
    count: i32,
    amount: i32,
) -> TriggeredAbility {
    magecraft(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            definition,
            count: Value::Const(count),
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// Magecraft-add-+1/+1-counter-to-friendly shortcut: "Whenever you
/// cast or copy an instant or sorcery spell, put a +1/+1 counter on
/// target creature you control." Wraps [`magecraft`] with an
/// `AddCounter` body targeting a friendly creature via
/// `target_filtered(Creature ∧ ControlledByYou)`. The auto-target
/// picker picks any controlled creature at trigger-resolve time.
/// Used by Quandrix Coursemage (b122) and any other "magecraft fans
/// counters" payoff. Refactor target for ~5 quandrix.rs callsites.
pub fn magecraft_add_counter_to_friendly() -> TriggeredAbility {
    use crate::card::{CounterType, SelectionRequirement};
    magecraft(Effect::AddCounter {
        what: target_filtered(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou),
        ),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    })
}

/// ETB-Surveil shortcut: "When this creature enters, surveil
/// `amount`." Wraps [`etb`] with the canonical surveil body. Used
/// by ~5 STX/SOS Witherbloom / Silverquill surveil creatures
/// (Silverquill Scrivener, Witherbloom Toxicpath, etc.).
pub fn etb_surveil(amount: i32) -> TriggeredAbility {
    etb(Effect::Surveil {
        who: PlayerRef::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft-Surveil shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, surveil `amount`." Wraps
/// [`magecraft`] with a `Surveil` body. The Witherbloom
/// counterpart to `magecraft_scry`; useful for "smooth + dig"
/// payoffs that want graveyard fuel.
pub fn magecraft_surveil(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Surveil {
        who: PlayerRef::You,
        amount: Value::Const(amount),
    })
}

/// ETB-Ping-Any shortcut: "When this creature enters, deal
/// `amount` damage to any target." Wraps [`etb`] with the
/// canonical "any target" damage body (creature OR player OR
/// planeswalker filter). Mirrors `magecraft_ping_any` for the
/// ETB trigger flavor. Used by Lorehold Emberspeaker / Prismari
/// Smiteforge-style "ETB shock-on-entry" creatures.
pub fn etb_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    etb(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
        ),
        amount: Value::Const(amount),
    })
}

/// ETB-Ping-Creature shortcut: "When this creature enters, deal
/// `amount` damage to target creature." Wraps [`etb`] with a
/// creature-only damage body. Used by Lorehold Sparkscholar /
/// Lorehold Ironhand-style "ETB ping creature" creatures (no
/// player/planeswalker target option).
pub fn etb_ping_creature(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    etb(Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(amount),
    })
}

/// Magecraft-Ping-Creature shortcut: "Whenever you cast or copy
/// an instant or sorcery spell, deal `amount` damage to target
/// creature." Wraps [`magecraft`] with a creature-only damage
/// body. Used by Lorehold Sparkscholar II and other "creature-
/// removal-only magecraft" cards.
pub fn magecraft_ping_creature(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    magecraft(Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(amount),
    })
}

/// ETB-Drain-and-Surveil shortcut: "When this creature enters, each
/// opponent loses `drain` life and you gain `drain` life. Surveil
/// `surveil`." Wraps [`etb`] with a `Seq([Drain, Surveil])` body.
/// Used by Silverquill Quillthane, Witherbloom Toxicpath, Silverquill
/// Conviction-style "ETB drain + select" creatures to collapse the
/// recurring 10-line pattern.
pub fn etb_drain_and_surveil(drain: i32, surveil: i32) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(drain),
        },
        Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(surveil),
        },
    ]))
}

/// ETB-Drain-and-Scry shortcut: "When this creature enters, each
/// opponent loses `drain` life and you gain `drain` life. Scry
/// `scry`." Wraps [`etb`] with a `Seq([Drain, Scry])` body. Used
/// by Silverquill Quillscribe / Inkling Stormcaller-style "ETB drain
/// + smooth" creatures to collapse the recurring pattern.
pub fn etb_drain_and_scry(drain: i32, scry: i32) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(drain),
        },
        Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(scry),
        },
    ]))
}

/// ETB-Drain-and-Draw shortcut: "When this creature enters, each
/// opponent loses `drain` life and you gain `drain` life. Draw a
/// card." Wraps [`etb`] with a `Seq([Drain, Draw])` body. Used by
/// Silverquill drain-and-cantrip creatures.
///
/// Push claude/modern_decks batch 137: shipped to collapse the
/// recurring drain+draw ETB pattern.
pub fn etb_drain_and_draw(drain: i32) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(drain),
        },
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
    ]))
}

/// On-Attack-Create-Token shortcut: "Whenever this creature attacks,
/// create a `token`." Wraps [`on_attack`] with a `CreateToken` body.
/// Used by Spirit/Pest/Inkling tribal attack-token creators.
///
/// Push claude/modern_decks batch 137: shipped to collapse the
/// recurring "attack → mint token" pattern.
pub fn on_attack_create_token(token: crate::card::TokenDefinition) -> TriggeredAbility {
    on_attack(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: token,
    })
}

/// Mint N copies of `token` as a standalone Effect (not wrapped in
/// an ETB trigger). Useful as the body of a sorcery / instant or
/// inside a `Seq([…])` step. Wraps `Effect::CreateToken` with
/// `who: PlayerRef::You`.
///
/// Push claude/modern_decks batch 105: shipped as part of the
/// `mint_pests`/`mint_inklings`/`mint_spirits` / `mint_fractals` /
/// `mint_treasures` family that centralises the canonical token
/// mints for STX/SOS catalog cards.
pub fn mint_token(token: crate::card::TokenDefinition, count: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition: token,
    }
}

/// Mint N STX Pest tokens. Pest body comes from
/// `catalog::stx_pest_token` and includes the standard
/// "this creature dies → you gain 1 life" trigger.
pub fn mint_pests(count: i32) -> Effect {
    let token = crate::tokens::stx_pest_token();
    mint_token(token, count)
}

/// Mint N SOS Inkling tokens (1/1 W/B flying creature).
pub fn mint_inklings(count: i32) -> Effect {
    let token = crate::tokens::inkling_token();
    mint_token(token, count)
}

/// Mint N SOS Spirit tokens (1/1 W flying creature, from SOS's
/// Spirit Mascot template).
pub fn mint_spirits(count: i32) -> Effect {
    let token = crate::tokens::spirit_token();
    mint_token(token, count)
}

/// Mint N SOS Fractal tokens (0/0 G/U creature; usually paired with
/// `Effect::AddCounter` against `Selector::LastCreatedToken` to
/// stamp +1/+1 counters on entry).
pub fn mint_fractals(count: i32) -> Effect {
    let token = crate::tokens::fractal_token();
    mint_token(token, count)
}

/// Mint N Treasure tokens (`{T}, Sacrifice: add one mana of any
/// color`). Uses [`crate::tokens::treasure_token`].
pub fn mint_treasures(count: i32) -> Effect {
    let token = crate::tokens::treasure_token();
    mint_token(token, count)
}

/// Mint N Lorehold Spirit tokens (2/2 R/W creature). Used by
/// `stx::lorehold::lorehold_excavation`-template cards and the
/// `stx::extras::lorehold_*` mint bodies.
pub fn mint_lorehold_spirits(count: i32) -> Effect {
    let token = crate::tokens::lorehold_spirit_token();
    mint_token(token, count)
}

/// Drain-and-Draw composite: `Seq([Drain(amount), Draw(1)])` as a
/// raw `Effect` (not wrapped in a trigger). Used by Silverquill
/// drain+cantrip sorceries (Silverquill Quillsweep, Silverquill
/// Chronicle, Defend the Inkwell-style spells) to collapse the
/// recurring 7-line `Seq` body to a one-liner.
pub fn drain_and_draw(amount: i32) -> Effect {
    Effect::Seq(vec![drain(amount), draw(1)])
}

/// Drain-and-Scry composite: `Seq([Drain(amount), Scry(scry)])` as
/// a raw `Effect` (not wrapped in a trigger). Companion to
/// `etb_drain_and_scry` for sorceries / instants where the drain
/// fires at spell resolution rather than ETB.
pub fn drain_and_scry(amount: i32, scry: i32) -> Effect {
    Effect::Seq(vec![
        drain(amount),
        Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(scry),
        },
    ])
}

/// Drain-and-Surveil composite: `Seq([Drain(amount), Surveil(N)])`
/// as a raw `Effect`. Used by Witherbloom / Silverquill drain +
/// graveyard-select sorceries (Silverquill Conviction, Silverquill
/// Inkletter, Witherspell Witness-template cards).
pub fn drain_and_surveil(amount: i32, surveil: i32) -> Effect {
    Effect::Seq(vec![
        drain(amount),
        Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(surveil),
        },
    ])
}

/// ETB-Tap-Opp-Creature shortcut: "When this creature enters, tap
/// target creature an opponent controls." Wraps [`etb`] with the
/// canonical "tempo tap" body. Used by Silverquill Lawkeeper-style
/// "ETB tap a creature" Soldiers / Wizards.
pub fn etb_tap_opp_creature() -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    etb(Effect::Tap {
        what: target_filtered(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByOpponent),
        ),
    })
}

/// Dies-Lose-Life-Each-Opp shortcut: "When this creature dies, each
/// opponent loses `amount` life." This is the asymmetric variant of
/// [`dies_drain`] — opponents lose life on death but you do NOT gain
/// any. Used by Pest Mawcrawler (batch 119), Witherbloom Reaper-Hand
/// templates, and any future on-death drain body where the printed
/// text omits the symmetric you-gain rider.
///
/// Push claude/modern_decks batch 123: shipped as part of the
/// asymmetric-drain helper family.
pub fn dies_lose_life_each_opp(amount: i32) -> TriggeredAbility {
    on_dies(Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(amount),
    })
}

/// Magecraft-Drain shortcut: "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, each opponent loses `amount` life
/// and you gain `amount` life." Wraps [`magecraft`] with the
/// canonical symmetric drain body. Distinct from
/// [`magecraft_drain_each_opp`] (asymmetric, opp-only) and
/// [`magecraft_drain_target`] (target a single opp).
///
/// Push claude/modern_decks batch 123: shipped as part of the
/// magecraft-drain helper family. Used by Witherbloom Apprentice-
/// template magecraft drains across batches 119–123.
pub fn magecraft_drain(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// On-Attack-Drain shortcut: "Whenever this creature attacks, each
/// opponent loses `amount` life and you gain `amount` life." Wraps
/// [`on_attack`] with the canonical symmetric drain body. Used by
/// Witherbloom Vinekeeper II-template attack-drain creatures, and
/// any Vampire / Inkling whose printed text is "whenever attacks,
/// drain N".
///
/// Push claude/modern_decks batch 125: shipped as part of the
/// attack-trigger drain helper family. Mirrors `etb_drain` /
/// `dies_drain` for the attack event.
pub fn on_attack_drain(amount: i32) -> TriggeredAbility {
    on_attack(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// On-Attack-Gain-Life shortcut: "Whenever this creature attacks,
/// you gain `amount` life." Wraps [`on_attack`] with the
/// canonical gain-life body. Used by lifelink-adjacent attack
/// triggers (Lorehold Warrior-Priest template, Spirit Mentor's
/// scaling lifegain) where the printed text omits the opp-loses
/// half of a drain.
///
/// Push claude/modern_decks batch 125: shipped alongside
/// [`on_attack_drain`] for asymmetric attack-trigger payoffs.
pub fn on_attack_gain_life(amount: i32) -> TriggeredAbility {
    on_attack(Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// On-Attack-Ping shortcut: "Whenever this creature attacks, it
/// deals `amount` damage to any target." Wraps [`on_attack`] with
/// a `DealDamage` body whose target is `target_filtered(Creature ∨
/// Player ∨ Planeswalker)`. Used by Lorehold Pyrostriker-style
/// "attack triggers a ping" creatures.
///
/// Push claude/modern_decks batch 125: shipped to collapse the
/// recurring attack-trigger ping pattern.
pub fn on_attack_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    on_attack(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
        ),
        amount: Value::Const(amount),
    })
}

/// Dies-Ping-Any shortcut: "When this creature dies, it deals
/// `amount` damage to any target." Wraps [`on_dies`] with a
/// `DealDamage` body whose target is `target_filtered(Creature ∨
/// Player ∨ Planeswalker)`. Mirrors `on_attack_ping_any` /
/// `etb_ping_any` for the on-death event. Used by parting-shot
/// creatures (Mogg Fanatic / Goblin Cratermaker template) and
/// Lorehold's death-pyromancer cycle.
///
/// Push claude/modern_decks batch 126: shipped to collapse the
/// recurring on-death ping pattern across STX Lorehold / Prismari
/// catalog cards.
pub fn dies_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    on_dies(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
        ),
        amount: Value::Const(amount),
    })
}

/// Dies-Mint-Token shortcut: "When this creature dies, create
/// `count` copies of `definition`." Wraps [`on_dies`] with the
/// canonical mint-on-death body. Used by Pest Swarmer-style
/// self-replacing bodies (a Pest mints a Pest on death) and the
/// Lorehold death-spirit cycle.
///
/// Push claude/modern_decks batch 126: shipped to collapse the
/// recurring death-mint pattern (Witherbloom Mossgrower, Lorehold
/// Spiritbinder, etc.).
pub fn dies_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition,
    })
}

/// Dies-Ping-Creature shortcut: "When this creature dies, deal
/// `amount` damage to target creature." Mirror of `dies_ping_any`
/// / `dies_drain` for the creature-only target case. Used by
/// Mogg Fanatic-style "dies dealing N to a creature" cards.
///
/// Push claude/modern_decks batch 141: shipped to collapse the
/// recurring "dies → ping creature" pattern.
pub fn dies_ping_creature(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    on_dies(Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(amount),
    })
}

/// On-other-dies-Mint-Token shortcut: "Whenever another creature
/// you control dies, create one copy of `definition`." Witherbloom
/// aristocrats payoff that scales with sacrifice fodder. Mirror of
/// `dies_mint_token` for the another-dies event scope.
///
/// Push claude/modern_decks batch 141: shipped to collapse the
/// recurring "another dies → mint" pattern (Witherbloom Pestcaller
/// II b141 + future aristocrats-style payoffs).
pub fn on_other_dies_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    on_other_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition,
    })
}

/// Magecraft-Mint-Spirit shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create one 2/2 R/W Spirit token."
/// Lorehold-specific sibling of `magecraft_mint_token` that uses
/// the shared `lorehold_spirit_token()` definition. Pulled into a
/// helper so future Lorehold mint-on-cast creatures collapse to
/// one line.
///
/// Push claude/modern_decks batch 141: shipped per the TODO
/// suggestion under "Suggested next-up tasks (additions from batch
/// 129)" to collapse Lorehold Sparkscholar II's spell-mint pattern.
pub fn magecraft_mint_spirit() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crate::tokens::lorehold_spirit_token(),
    })
}

/// Magecraft-Draw shortcut: "Whenever you cast or copy an instant
/// or sorcery spell, draw a card." Wraps [`magecraft`] with the
/// canonical Draw 1 body. Distinct from [`magecraft_loot`] which
/// also discards. Used by Archmage Emeritus' draw-on-cast payoff
/// and any future "magecraft → draw" engine creature.
///
/// Push claude/modern_decks batch 126: shipped to collapse the
/// recurring magecraft-draw pattern.
pub fn magecraft_draw(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Draw {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft-Treasure shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create a Treasure token." Wraps
/// [`magecraft`] with a Treasure-mint body. Used by Prismari
/// Inventor / Prismari Treasure Smith / Symphony of the Wilds-
/// style treasure-on-cast bodies.
///
/// Push claude/modern_decks batch 126: shipped to collapse the
/// recurring magecraft-treasure pattern.
pub fn magecraft_treasure() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crate::tokens::treasure_token(),
    })
}

/// On-Attack-Loot shortcut: "Whenever this creature attacks, draw
/// a card, then discard a card." Wraps [`on_attack`] with the
/// canonical Seq[Draw 1, Discard 1] body. Used by attack-trigger
/// looters (Stormchaser-class fliers, Prismari attack-tempo
/// bodies).
///
/// Push claude/modern_decks batch 126: shipped to collapse the
/// recurring attack-loot pattern (Prismari Stormbearer template).
pub fn on_attack_loot() -> TriggeredAbility {
    on_attack(Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::Const(1),
            random: false,
        },
    ]))
}

/// "Whenever this creature attacks and isn't blocked, …" trigger
/// (CR 509.3g). Wraps the new `EventKind::AttacksAndIsntBlocked`
/// event added in batch 127. Fired once per attacker that
/// finishes the declare-blockers step with zero blockers
/// assigned. Used by ninja-style "combat-damage-only" payoffs and
/// any "swing in for free" attack riders.
pub fn on_unblocked(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
        effect,
    }
}

/// Frenzy N (CR 702.68): "Whenever this creature attacks and isn't
/// blocked, it gets +N/+0 until end of turn." An
/// `AttacksAndIsntBlocked / SelfSource` pump of `This`.
pub fn frenzy(n: i32) -> TriggeredAbility {
    on_unblocked(Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(n),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    })
}

/// Afflict N (CR 702.131): "Whenever this creature becomes blocked,
/// defending player loses N life." A `BecomesBlocked / SelfSource`
/// trigger draining the `DefendingPlayer` (resolved while the source
/// is still attacking).
pub fn afflict(n: i32) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::Const(n),
        },
    }
}

/// Mentor (CR 702.134): "Whenever this creature attacks, put a +1/+1
/// counter on target attacking creature with lesser power." An `Attacks /
/// SelfSource` trigger targeting another attacking creature whose power is
/// less than the source's; the counter lands on that target. If no such
/// creature is attacking, the trigger has no legal target and does nothing.
pub fn mentor() -> TriggeredAbility {
    use crate::card::CounterType;
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::AddCounter {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::IsAttacking
                    .and(SelectionRequirement::OtherThanSource)
                    .and(SelectionRequirement::PowerLessThanSource),
            },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    }
}

/// Dethrone (CR 702.105): "Whenever this creature attacks the player with
/// the most life or tied for most life, put a +1/+1 counter on it." An
/// `Attacks / SelfSource` trigger gated on `PlayerHasMostLife` for the
/// defending player; the counter lands on `This`.
pub fn dethrone() -> TriggeredAbility {
    use crate::card::CounterType;
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
            Predicate::PlayerHasMostLife { who: PlayerRef::DefendingPlayer },
        ),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    }
}

/// Support N (CR 701.32): "Put a +1/+1 counter on each of up to N target
/// creatures." A bare `Effect` for use as a spell effect or ability body.
pub fn support(n: u8) -> Effect {
    Effect::SupportCounters { max_targets: n, filter: SelectionRequirement::Creature }
}

/// Amass N (CR 701.43): grow (or create) an Army by N +1/+1 counters.
/// A bare `Effect`, so callers wrap it in whatever trigger/ETB/spell
/// shell the card uses.
pub fn amass(n: i32) -> Effect {
    Effect::Amass { who: PlayerRef::You, count: Value::Const(n), extra_type: None }
}

/// Amass Zombies N — like [`amass`] but the minted Army is also a Zombie.
pub fn amass_zombies(n: i32) -> Effect {
    Effect::Amass {
        who: PlayerRef::You,
        count: Value::Const(n),
        extra_type: Some(crate::card::CreatureType::Zombie),
    }
}

/// Myriad (CR 702.115): an `Attacks / SelfSource` trigger minting a
/// tapped+attacking copy of the source for each other opponent, exiled
/// at end of combat.
pub fn myriad() -> TriggeredAbility {
    on_attack(Effect::Myriad)
}

/// Enlist (CR 702.151): an `Attacks / SelfSource` trigger that taps a
/// nonattacking creature and adds its power to the attacker EOT.
pub fn enlist() -> TriggeredAbility {
    on_attack(Effect::Enlist)
}

/// Mobilize N (CR 702.169): "Whenever this creature attacks, create N
/// 1/1 red Warrior creature tokens that are tapped and attacking.
/// Sacrifice them at the beginning of the next end step." (Modeled as
/// end-of-combat sacrifice — the tokens vanish before postcombat main
/// either way.) An `Attacks / SelfSource` trigger over
/// [`Effect::CreateTokenAttacking`].
pub fn mobilize(n: i32) -> TriggeredAbility {
    on_attack(Effect::CreateTokenAttacking {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: crate::card::TokenDefinition {
            name: "Warrior".into(),
            power: 1,
            toughness: 1,
            card_types: vec![crate::card::CardType::Creature],
            colors: vec![crate::mana::Color::Red],
            subtypes: crate::card::Subtypes {
                creature_types: vec![crate::card::CreatureType::Warrior],
                ..Default::default()
            },
            ..Default::default()
        },
        cleanup: AttackingTokenCleanup::SacrificeAtEndOfCombat,
    })
}

/// Afterlife N (CR 702.135): "When this creature dies, create N 1/1
/// white and black Spirit creature tokens with flying." A
/// `CreatureDied / SelfSource` trigger minting the standard Afterlife
/// Spirit body.
pub fn afterlife(n: i32) -> TriggeredAbility {
    on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: crate::card::TokenDefinition {
            name: "Spirit".into(),
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            card_types: vec![crate::card::CardType::Creature],
            colors: vec![crate::mana::Color::White, crate::mana::Color::Black],
            supertypes: vec![],
            subtypes: crate::card::Subtypes {
                creature_types: vec![crate::card::CreatureType::Spirit],
                ..Default::default()
            },
            activated_abilities: vec![],
            triggered_abilities: vec![],
            static_abilities: vec![],
            equipped_bonus: None,
        },
    })
}

/// Extort (CR 702.99): "Whenever you cast a spell, you may pay
/// {W/B}. If you do, each opponent loses 1 life and you gain that
/// much life." A `SpellCast / YourControl` trigger whose body is a
/// `MayPay` over the canonical [`drain`] shape. Basilica Screecher,
/// Crypt Ghast, Pontiff of Blight.
pub fn extort() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
        effect: Effect::MayPay {
            description: "Extort — pay {W/B}: drain 1".into(),
            mana_cost: crate::mana::cost(&[crate::mana::hybrid(Color::White, Color::Black)]),
            body: Box::new(drain(1)),
        },
    }
}

/// Exploit (CR 702.105): "When this creature enters, you may sacrifice
/// a creature. When you exploit a creature, `payoff`." Modeled as an ETB
/// `MayDo([Sacrifice 1 creature (this can be itself), payoff])`. Declining
/// the sacrifice skips the payoff (CR 702.105d — the exploit trigger only
/// does something if a creature is actually sacrificed). AutoDecider
/// declines; a value-aware bot / scripted decider accepts.
pub fn exploit(payoff: Effect) -> TriggeredAbility {
    etb(Effect::MayDo {
        description: "Exploit — sacrifice a creature?".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            payoff,
        ])),
    })
}

/// Devour N (CR 702.83): "As this creature enters, you may sacrifice any
/// number of creatures. It enters with N +1/+1 counters on it for each
/// creature sacrificed this way." Modeled as an ETB `SacrificeAnyNumber`
/// over other creatures, each sacrifice dropping N +1/+1 counters on the
/// devourer (`Selector::This`). AutoDecider sacrifices none.
pub fn devour(n: i32) -> TriggeredAbility {
    use crate::card::CounterType;
    etb(Effect::SacrificeAnyNumber {
        who: PlayerRef::You,
        filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
        per_each: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        }),
    })
}

/// Riot (CR 702.137): "This creature enters the battlefield with
/// your choice of a +1/+1 counter or haste." Modeled as an ETB
/// `ChooseMode([grant Haste permanently, add a +1/+1 counter])`.
/// AutoDecider takes mode 0 (haste); scripted deciders can pick the
/// counter. Zhur-Taa Goblin, Gruul Spellbreaker.
pub fn riot() -> TriggeredAbility {
    use crate::card::CounterType;
    etb(Effect::ChooseMode(vec![
        Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Haste,
            duration: Duration::Permanent,
        },
        Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    ]))
}

/// Unleash (CR 702.98): "You may have this permanent enter with an
/// additional +1/+1 counter on it." Modeled as an ETB `MayDo(add a +1/+1
/// counter to this)`. The companion "can't block while it has a +1/+1
/// counter" restriction is enforced as a computed `CantBlock` keyword in
/// `gather_continuous_effects`. Pair with `Keyword::Unleash` on the card.
pub fn unleash() -> TriggeredAbility {
    use crate::card::CounterType;
    etb(Effect::MayDo {
        description: "Unleash — enter with a +1/+1 counter?".into(),
        body: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        }),
    })
}

/// Soulshift N (CR 702.46): "When this creature dies, you may return
/// target Spirit card with mana value N or less from your graveyard
/// to your hand." A `CreatureDied / SelfSource` trigger wrapping a
/// `MayDo(Move target → hand)` over a graveyard Spirit filter.
pub fn soulshift(n: u32) -> TriggeredAbility {
    on_dies(Effect::MayDo {
        description: format!("Soulshift {n} — return a Spirit from your graveyard"),
        body: Box::new(Effect::Move {
            what: target_filtered(
                SelectionRequirement::InGraveyard
                    .and(SelectionRequirement::HasCreatureType(
                        crate::card::CreatureType::Spirit,
                    ))
                    .and(SelectionRequirement::ManaValueAtMost(n)),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        }),
    })
}

/// Adapt N (CR 702.108) — the *effect* of an Adapt activated ability:
/// "If this creature has no +1/+1 counters on it, put N +1/+1 counters
/// on it." Built from existing primitives (`If` + `EntityMatches` +
/// `AddCounter`); pair it with an `ActivatedAbility` carrying the
/// adapt mana cost. Used by Pteramander, Incubation Druid-style cards.
pub fn adapt(n: i32) -> Effect {
    use crate::card::CounterType;
    Effect::If {
        cond: Predicate::Not(Box::new(Predicate::EntityMatches {
            what: Selector::This,
            filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
        })),
        then: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        }),
        else_: Box::new(Effect::Noop),
    }
}

/// Bloodthirst N (CR 702.54) — an ETB trigger standing in for the
/// printed static "this creature enters with N +1/+1 counters on it if
/// an opponent was dealt damage this turn." Modeled as `etb(If(an
/// opponent was dealt damage this turn → AddCounter This N))` — the
/// counters add to the printed (positive) base P/T.
pub fn bloodthirst(n: i32) -> TriggeredAbility {
    use crate::card::CounterType;
    etb(Effect::If {
        cond: Predicate::PlayerDamagedThisTurn { who: PlayerRef::EachOpponent },
        then: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        }),
        else_: Box::new(Effect::Noop),
    })
}

/// Renown N (CR 702.111) — a combat trigger: "Whenever this creature
/// deals combat damage to a player, if it isn't renowned, put N +1/+1
/// counters on it and it becomes renowned." The "isn't renowned" gate
/// is approximated by "has no +1/+1 counters" (same shape as Adapt) —
/// it only renowns once because the counters then block re-triggering.
pub fn renown(n: i32) -> TriggeredAbility {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::Not(Box::new(Predicate::EntityMatches {
                what: Selector::This,
                filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
            })),
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(n),
            }),
            else_: Box::new(Effect::Noop),
        },
    }
}

/// Poisonous N (CR 702.70): "Whenever this creature deals combat damage
/// to a player, that player gets N poison counters." A
/// `DealsCombatDamageToPlayer / SelfSource` trigger; the damaged player is
/// bound to target slot 0 by `fire_combat_damage_to_player_triggers`.
pub fn poisonous(n: u32) -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::AddPoison {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(n as i32),
        },
    }
}

/// Ingest (CR 702.115): "Whenever this creature deals combat damage to a
/// player, that player exiles the top card of their library." A
/// `DealsCombatDamageToPlayer / SelfSource` trigger; the damaged player is
/// bound to target slot 0 by `fire_combat_damage_to_player_triggers`.
pub fn ingest() -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::ExileTopOfLibrary {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(1),
        },
    }
}

/// Outlast (CR 702.97) — the activated ability "{cost}, {T}: Put a
/// +1/+1 counter on this creature. Activate only as a sorcery." Returns
/// the `ActivatedAbility`; pass the (already mana-loaded) cost in.
pub fn outlast(mana_cost: crate::mana::ManaCost) -> ActivatedAbility {
    use crate::card::CounterType;
    ActivatedAbility {
        energy_cost: 0,
        tap_cost: true,
        mana_cost,
        sorcery_speed: true,
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
        ..Default::default()
    }
}

/// Connive N (CR 702.158) — "Draw N cards, then discard N cards. For
/// each nonland card discarded this way, put a +1/+1 counter on this
/// creature." Built from `Draw` + `Discard` + an `AddCounter` whose
/// amount reads `DiscardedThisResolution { Nonland }` (the discard
/// scratch is captured within the same resolution). The discarder
/// chooses which cards to pitch (AutoDecider pitches from the front).
pub fn connive(n: i32) -> Effect {
    use crate::card::CounterType;
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(n) },
        Effect::Discard { who: Selector::You, amount: Value::Const(n), random: false },
        Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::count(Selector::DiscardedThisResolution {
                filter: SelectionRequirement::Nonland,
            }),
        },
    ])
}

/// Bolster N (CR 701.21) — "put N +1/+1 counters on the creature you
/// control with the least toughness." Built from `AddCounter` over the
/// `Selector::LeastToughnessYouControl` selector; pair it with whatever
/// trigger/ability carries the bolster (ETB, attack, etc.).
pub fn bolster(n: i32) -> Effect {
    use crate::card::CounterType;
    Effect::AddCounter {
        what: Selector::LeastToughnessYouControl,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    }
}

/// Fabricate N (CR 702.122) — an ETB triggered ability: "put N +1/+1
/// counters on this creature, or create N 1/1 colorless Servo artifact
/// creature tokens." Modeled as `etb(ChooseMode([AddCounter, CreateToken]))`;
/// AutoDecider takes mode 0 (counters), a scripted decider can pick the
/// Servos. Used by Cultivator of Blades, Angel of Invention-style cards.
pub fn fabricate(n: i32) -> TriggeredAbility {
    use crate::card::CounterType;
    let servo = crate::card::TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![crate::card::CardType::Artifact, crate::card::CardType::Creature],
        colors: vec![],
        supertypes: vec![],
        subtypes: crate::card::Subtypes {
            creature_types: vec![crate::card::CreatureType::Servo],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
        equipped_bonus: None,
    };
    etb(Effect::ChooseMode(vec![
        Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        },
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(n),
            definition: servo,
        },
    ]))
}

/// ETB-Mint-Token-With-Counters shortcut: "When this creature
/// enters, create `count` copies of `definition`, then put
/// `counter_amount` +1/+1 counters on each." Wraps [`etb`] with
/// `Seq([CreateToken, AddCounter(LastCreatedToken, +1/+1, N)])`.
/// Used by Quandrix Bloomforge (b128, 4-counter Fractal token) and
/// Quandrix Geometer (b128, 2-counter Fractal). Mirrors the
/// Body of Research and Fractal Summoning patterns.
///
/// Push claude/modern_decks batch 128: shipped to collapse the
/// recurring "ETB mint a counter-scaled token" pattern across
/// Quandrix Fractal-engine cards.
pub fn etb_mint_token_with_counters(
    definition: crate::card::TokenDefinition,
    count: i32,
    counter_amount: i32,
) -> TriggeredAbility {
    use crate::card::CounterType;
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(count),
            definition,
        },
        Effect::AddCounter {
            what: Selector::LastCreatedToken,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(counter_amount),
        },
    ]))
}

/// ETB-Mint-And-Drain shortcut: "When this creature enters, create
/// one copy of `definition` and each opponent loses `amount` life
/// and you gain `amount` life." Wraps [`etb`] with
/// `Seq([CreateToken, Drain(amount)])`. Used by Witherbloom
/// Cauldronherder (b129, Pest mint + drain 2) and any future
/// "mint a body, drain the table" Witherbloom Pest cards.
///
/// Push claude/modern_decks batch 129: collapses the 12-line
/// `Seq[CreateToken, Drain]` pattern at the call site.
pub fn etb_mint_token_and_drain(
    definition: crate::card::TokenDefinition,
    amount: i32,
) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition,
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// ETB-Mint-Token-And-Gain-Life shortcut: "When this creature
/// enters, create one copy of `definition` and gain `amount`
/// life." Wraps [`etb`] with `Seq([CreateToken, GainLife])`.
/// Asymmetric variant of [`etb_mint_token_and_drain`] (Witherbloom
/// uses drain; Lorehold/Selesnya-style cards just gain).
///
/// Push claude/modern_decks batch 132: shipped to collapse the
/// recurring "mint + gain" pattern (Lorehold Bell-Ringer template
/// — mints a Spirit + gains 2 life, etc.).
pub fn etb_mint_token_and_gain_life(
    definition: crate::card::TokenDefinition,
    amount: i32,
) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition,
        },
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// ETB-Scry-and-Draw shortcut: "When this creature enters,
/// scry `scry_amount`, then draw a card." Wraps [`etb`] with
/// `Seq([Scry, Draw])`. Used by smoothing creatures that combine
/// library-quality and card draw (Silverquill Scrivener-Apprentice
/// template). The scry resolves first per the printed sequence.
///
/// Push claude/modern_decks batch 132: shipped to collapse the
/// recurring "scry + draw" ETB pattern.
pub fn etb_scry_and_draw(scry_amount: i32) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(scry_amount),
        },
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
    ]))
}

/// ETB-Pump-And-Grant-Keyword shortcut: target friendly creature
/// gets +`power`/+`toughness` and gains `keyword` until end of
/// turn. Returns a raw `Effect` (not wrapped in a trigger). Used
/// by combat-trick instants like Lorehold Final Lesson (b132)
/// that combine stat-pump with a keyword grant.
///
/// Push claude/modern_decks batch 132: shipped to collapse the
/// recurring "+P/+T + keyword EOT" combat-trick pattern.
pub fn pump_and_grant_keyword(
    power: i32,
    toughness: i32,
    keyword: crate::card::Keyword,
) -> Effect {
    use crate::card::SelectionRequirement;
    Effect::Seq(vec![
        Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword,
            duration: Duration::EndOfTurn,
        },
    ])
}

/// Magecraft-Scry-And-Draw shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, scry `scry_amount`, then draw a card."
/// Wraps [`magecraft`] with `Seq([Scry, Draw])`. Mirror of
/// `etb_scry_and_draw` for the magecraft event. Used by Sphinx's
/// Insight-template magecraft engines that combine smoothing with
/// card draw.
///
/// Push claude/modern_decks batch 133: shipped to collapse the
/// recurring "magecraft → scry + draw" pattern.
pub fn magecraft_scry_and_draw(scry_amount: i32) -> TriggeredAbility {
    magecraft(Effect::Seq(vec![
        Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(scry_amount),
        },
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
    ]))
}

/// Dies-Mint-Token-And-Drain shortcut: "When this creature dies,
/// create a token and each opponent loses `amount` life and you
/// gain `amount` life." Mirror of `etb_mint_token_and_drain` for
/// the on-death event. Used by Witherbloom Pest-aristocrats
/// templates that replace the dying body with a fresh Pest while
/// also draining the table.
///
/// Push claude/modern_decks batch 133: shipped to collapse the
/// recurring "die → mint + drain" pattern.
pub fn dies_mint_token_and_drain(
    definition: crate::card::TokenDefinition,
    amount: i32,
) -> TriggeredAbility {
    on_dies(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition,
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// Magecraft trigger that grows the source AND drains each opponent:
/// "Whenever you cast or copy an instant or sorcery spell, put a
/// +1/+1 counter on this creature and each opponent loses `amount`
/// life and you gain `amount` life." A common Witherbloom "build-
/// your-own-Apprentice" shape — see Witherbloom Reapcaster (b146)
/// for the original card. The Seq order matters: counter ride before
/// the drain so SBA-evaluated counter pump is visible to "if you
/// gained life" payoffs that fire on the drain.
pub fn magecraft_self_pump_and_drain(amount: i32) -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::Seq(vec![
        Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// ETB trigger that drains each opponent AND draws you a card:
/// "When this creature enters, each opponent loses `amount` life,
/// you gain `amount` life, and you draw a card." A Silverquill /
/// Witherbloom mid-curve value body shape. Closes a recurring
/// pattern observed in batch 146 (`Inkling Decree`, `Inkling
/// Bloodscribe` future revisions).
pub fn etb_drain_and_draw_one(amount: i32) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
    ]))
}

/// Magecraft-Mint-Pest shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create a 1/1 B/G Pest token with
/// 'When this token attacks, you gain 1 life.'" Wraps
/// [`magecraft`] with the shared `stx_pest_token()` mint body.
/// Pulled out per the TODO suggestion under "Suggested next-up
/// tasks" — collapses the Witherbloom Pestmancer / Sedgemoor
/// Witch-template mint pattern to one line.
pub fn magecraft_mint_pest() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crate::tokens::stx_pest_token(),
    })
}

/// Magecraft-Mint-Inkling shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create a 1/1 W/B Inkling token with
/// flying." Wraps [`magecraft`] with the shared `inkling_token()`
/// mint body. Silverquill counterpart to [`magecraft_mint_spirit`]
/// (Lorehold) and [`magecraft_mint_pest`] (Witherbloom).
pub fn magecraft_mint_inkling() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crate::tokens::inkling_token(),
    })
}

/// Magecraft-Mint-Fractal shortcut: "Whenever you cast or copy an
/// instant or sorcery spell, create a 0/0 G/U Fractal token with
/// X +1/+1 counters on it" — `count` controls the number of
/// counters stamped via [`crate::card::CounterType::PlusOnePlusOne`]
/// against [`Selector::LastCreatedToken`]. Pairs with the printed
/// "Quandrix mage casts a spell → 1/1 Fractal" cycle. Wraps
/// [`magecraft`] with `Seq[CreateToken(0/0 Fractal), AddCounter]`.
pub fn magecraft_mint_fractal(counters: i32) -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::tokens::fractal_token(),
        },
        Effect::AddCounter {
            what: Selector::LastCreatedToken,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(counters),
        },
    ]))
}

/// Dies-Mint-Pest shortcut: "When this creature dies, create a
/// 1/1 B/G Pest token." Wraps [`on_dies`] with the shared
/// `stx_pest_token()` mint body. Pulled out per the TODO
/// suggestion: collapses Witherbloom Pest-Spawner-template
/// on-death replacement bodies (Pest Swarmer / future Pest cards
/// that self-replace on death) to one line.
pub fn dies_mint_pest() -> TriggeredAbility {
    on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crate::tokens::stx_pest_token(),
    })
}

/// On-attack-mint-Spirit shortcut: "Whenever this creature
/// attacks, create a 2/2 R/W Spirit token." Wraps [`on_attack`]
/// with the shared `lorehold_spirit_token()` mint body. Lorehold
/// counterpart to attack-trigger token engines — fills the slot
/// for Spirit-tribal "attack mints a Spirit" creatures.
pub fn on_attack_mint_lorehold_spirit() -> TriggeredAbility {
    on_attack(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: crate::tokens::lorehold_spirit_token(),
    })
}

/// Magecraft-AddCounter-Self shortcut: "Whenever you cast or copy
/// an instant or sorcery spell, put a +1/+1 counter on this
/// creature." Wraps [`magecraft`] with an `AddCounter` body
/// targeting `Selector::This`. Used by self-growing magecraft
/// bodies (Pensive Professor secondary trigger, Inkling
/// Bookbinder, Inkling Calligraphist, Silverquill Soulbinder,
/// Witherbloom Sproutchant, etc.) — currently inlined ~15 times
/// across `stx::*`; the helper collapses each call site to one
/// line and locks in the Self selector against future renames.
pub fn magecraft_add_counter_self() -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    })
}

/// Magecraft shortcut: "Whenever you cast or copy an instant or
/// sorcery spell, put a shield counter on this creature." Combos
/// with the CR 122.1c shield-counter wire (each shield blocks one
/// damage or one destroy and pops). Push (modern_decks batch 173).
pub fn magecraft_add_shield_self() -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Shield,
        amount: Value::Const(1),
    })
}

/// Magecraft shortcut: "Whenever you cast or copy an instant or
/// sorcery spell, put a finality counter on this creature." Combos
/// with the CR 122.1h finality wire (the creature exiles instead of
/// going to graveyard on next death). Push (modern_decks batch 173).
pub fn magecraft_add_finality_self() -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Finality,
        amount: Value::Const(1),
    })
}

/// Add a finality counter to a target creature you control. Used as
/// the resolution body of cards that grant the finality "exile on
/// death" rider to a target (Doom Whisperer-style enabler — though
/// Doom Whisperer's printed line is "you may put a -1/-1 counter on
/// it", this helper supports the "+finality counter" branch). Push
/// (modern_decks batch 175).
pub fn add_finality_to_target_creature() -> Effect {
    use crate::card::CounterType;
    Effect::AddCounter {
        what: target_filtered(crate::card::SelectionRequirement::Creature),
        kind: CounterType::Finality,
        amount: Value::Const(1),
    }
}

/// Add a shield counter to a target creature. Used by cards that
/// grant the CR 122.1c "shield" protection to a creature (Aegisblade,
/// Wardward analogs). Push (modern_decks batch 175).
pub fn add_shield_to_target_creature() -> Effect {
    use crate::card::CounterType;
    Effect::AddCounter {
        what: target_filtered(crate::card::SelectionRequirement::Creature),
        kind: CounterType::Shield,
        amount: Value::Const(1),
    }
}

/// Predicate shortcut: "You have at least `n` cards matching
/// `filter` in your graveyard." Wraps the canonical spell-mastery
/// / delirium-threshold / Murderous-Cut-style gate pattern:
/// `Predicate::SelectorCountAtLeast { sel: CardsInZone(You,
/// Graveyard, filter), n: Const(n) }`. Per the TODO suggestion
/// (batch 153 follow-ups), this collapses ~15 call sites in
/// `decks::modern` / `stx::*` that all rebuild the same shape.
/// Use for Fiery Impulse (IS x 2), Searing Blaze (any x 2),
/// Murderous Cut (any x 7 for delve), Mishra's Bauble (any x N),
/// any "spell mastery", "delirium", or "threshold" gate.
pub fn cards_in_graveyard_at_least(
    filter: crate::card::SelectionRequirement,
    n: i32,
) -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::CardsInZone {
            who: PlayerRef::You,
            zone: crate::card::Zone::Graveyard,
            filter,
        },
        n: Value::Const(n),
    }
}

/// Predicate shortcut for the printed "spell mastery" threshold —
/// "if there are two or more instant and/or sorcery cards in your
/// graveyard." Equivalent to `cards_in_graveyard_at_least(IS, 2)`.
/// Used by Fiery Impulse, Magmatic Insight-promotion-style cards.
pub fn spell_mastery_gate() -> Predicate {
    use crate::card::{CardType, SelectionRequirement};
    cards_in_graveyard_at_least(
        SelectionRequirement::HasCardType(CardType::Instant)
            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
        2,
    )
}

/// ETB-drain + add +1/+1 counter to self shortcut.
/// Models the "comes into play, drains N, then has a +1/+1
/// counter" cards (Silverquill Soulbinder template — drain then
/// scale). Wraps `etb` with a `Seq(Drain, AddCounter Self+1/+1)`.
pub fn etb_drain_and_counter_self(amount: i32) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
        Effect::AddCounter {
            what: Selector::This,
            kind: crate::card::CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
    ]))
}

/// On-Combat-Damage-To-Player + Gain Life shortcut: "Whenever this
/// creature deals combat damage to a player, you gain `amount`
/// life." Inkrise Lifedrainer template. Wraps the standard
/// `EventKind::DealsCombatDamageToPlayer / SelfSource` event spec
/// with a `GainLife { who: You }` body.
///
/// Push (claude/modern_decks batch 201): collapses the recurring
/// inline "DealsCombatDamageToPlayer → GainLife" pattern across
/// Inkrise-template cards. Keeps the call site to one line and
/// gives the spec stability across refactors.
pub fn on_combat_damage_to_player_gain_life(amount: i32) -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(amount),
        },
    }
}

/// On-Combat-Damage-To-Player + Drain shortcut: "Whenever this
/// creature deals combat damage to a player, each opponent loses
/// `amount` life and you gain `amount` life." Asymmetric drain-
/// on-connect — same shape as `on_combat_damage_to_player_gain_life`
/// but with a drain body instead of pure gain.
///
/// Push (claude/modern_decks batch 201): companion to
/// `on_combat_damage_to_player_gain_life`. Used by Witherbloom
/// connect-drain cards.
pub fn on_combat_damage_to_player_drain(amount: i32) -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    }
}

/// Modular N (CR 702.43) — the *dies* half: "When this creature dies,
/// you may put its +1/+1 counters on target artifact creature." Pair it
/// with `enters_with_counters: Some((PlusOnePlusOne, Const(n)))` on the
/// card def (the "enters with N +1/+1 counters" half). The dying source
/// is already in the graveyard, so this adds counters to the target
/// equal to the source's *last-known* +1/+1 count (`Value::CountersOn
/// { This }`, the same last-known path Hangarback Walker reads) rather
/// than a literal move from a permanent that no longer exists.
pub fn modular_dies() -> TriggeredAbility {
    on_dies(Effect::MayDo {
        description: "Put +1/+1 counters on target artifact creature".into(),
        body: Box::new(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Artifact.and(SelectionRequirement::Creature),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CountersOn {
                what: Box::new(Selector::This),
                kind: CounterType::PlusOnePlusOne,
            },
        }),
    })
}

/// Embalm (CR 702.88) / Eternalize (CR 702.91) — the activated ability:
/// "[cost], Exile this card from your graveyard: Create a token that's a
/// copy of it, except it's a [white/black] Zombie [with no mana cost
/// / and 4/4]. Activate only as a sorcery." Both ride the
/// `from_graveyard` + `exile_self_cost` activation path; the token rides
/// `CreateTokenCopyOf` with a Zombie type added (color/cost overrides are
/// approximated — the copy keeps the original's color).
pub fn embalm(cost: crate::mana::ManaCost) -> ActivatedAbility {
    embalm_like(cost, None)
}
pub fn eternalize(cost: crate::mana::ManaCost) -> ActivatedAbility {
    embalm_like(cost, Some((4, 4)))
}
fn embalm_like(
    cost: crate::mana::ManaCost,
    override_pt: Option<(i32, i32)>,
) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        from_graveyard: true,
        exile_self_cost: true,
        effect: Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::Const(1),
            source: Selector::This,
            extra_creature_types: vec![crate::card::CreatureType::Zombie],
            override_pt,
            non_legendary: false,
        },
        ..Default::default()
    }
}

/// Scavenge (CR 702.97): "[cost], Exile this card from your graveyard: Put a
/// number of +1/+1 counters equal to this card's power on target creature.
/// Activate only as a sorcery." Rides the gy-activation + exile-self-cost path;
/// the counter count reads the exiled card's printed power via `Value::PowerOf`.
pub fn scavenge(cost: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        from_graveyard: true,
        exile_self_cost: true,
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::PowerOf(Box::new(Selector::This)),
        },
        ..Default::default()
    }
}

/// Transmute (CR 702.53): "[cost], Discard this card: Search your library for a
/// card with the same mana value as this card, reveal it, put it into your
/// hand, then shuffle. Activate only as a sorcery." `mv` is the card's own mana
/// value (the filter the search uses). Rides the from-hand discard-self path.
pub fn transmute(cost: crate::mana::ManaCost, mv: u32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        from_hand: true,
        discard_self_cost: true,
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::ManaValueExactly(mv),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Graft N (CR 702.57) — the trigger half: "Whenever another creature
/// enters, you may move a +1/+1 counter from this creature onto it."
/// Pair with `enters_with_counters: Some((PlusOnePlusOne, Const(n)))`.
/// The counter only moves while this creature still has one (the
/// `MoveCounter` is a no-op once empty).
pub fn graft() -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::OtherThanSource),
            }),
        effect: Effect::MayDo {
            description: "Move a +1/+1 counter from this creature onto it".into(),
            body: Box::new(Effect::MoveCounter {
                from: Selector::This,
                to: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
    }
}

/// Melee (CR 702.121): "Whenever this creature attacks, it gets +1/+1
/// until end of turn for each opponent you attacked this combat."
/// Approximated as a flat +1/+1 on attack (one opponent in the common
/// 1v1 / single-defender case — the engine has no per-combat
/// attacked-opponent tally yet).
pub fn melee() -> TriggeredAbility {
    on_attack(Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(1),
        toughness: Value::Const(1),
        duration: Duration::EndOfTurn,
    })
}
