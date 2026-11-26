//! Common one-liner constructors for building card definitions tersely.
use super::*;

pub fn you() -> Selector { Selector::You }
pub fn this() -> Selector { Selector::This }
pub fn target() -> Selector { Selector::Target(0) }
pub fn target_n(n: u8) -> Selector { Selector::Target(n) }
pub fn target_filtered(filter: SelectionRequirement) -> Selector {
    Selector::TargetFiltered { slot: 0, filter }
}
/// "Any target" — creature, player or planeswalker; the burn-spell filter
/// (Lightning Bolt).
pub fn target_any() -> Selector {
    target_filtered(
        SelectionRequirement::any_target(),
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
/// "Target opponent loses `amount` life and you gain `amount` life" — one
/// chosen opponent (`drain` takes it from every opponent).
pub fn drain_target_opponent(amount: i32) -> Effect {
    Effect::Drain {
        from: target_filtered(crate::card::SelectionRequirement::OpponentPlayer),
        to: you(),
        amount: Value::Const(amount),
    }
}
/// "Each opponent loses `amount` life. You gain life equal to the life lost
/// this way" — the gain is the total across opponents (`DrainLifeLost`).
pub fn drain_life_lost(amount: Value) -> Effect {
    Effect::DrainLifeLost { from: Selector::Player(PlayerRef::EachOpponent), to: you(), amount }
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
pub fn exile_target() -> Effect { Effect::Exile { what: target() } }
/// "Destroy target **creature**. It can't be regenerated." Filtered: an
/// unfiltered slot 0 enumerates every permanent — Terminate, written with an
/// unfiltered target, destroyed anything. See TODO's unfiltered-slot census.
pub fn destroy_target_creature_no_regen() -> Effect {
    Effect::DestroyNoRegen { what: target_filtered(SelectionRequirement::Creature) }
}
/// "Return target **creature** to its owner's hand", filtered to creatures.
pub fn return_target_creature_to_hand() -> Effect {
    Effect::Move {
        what: target_filtered(SelectionRequirement::Creature),
        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(target()))),
    }
}
/// "Target creature gets +P/+T until end of turn."
pub fn pump_target(power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what: target_filtered(SelectionRequirement::Creature),
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

/// Dash (CR 702.109) alternative cost: cast for `mana_cost`; the creature
/// gains haste and returns to its owner's hand at the next end step.
/// CR 702.173 — freerunning [cost]: legal once an Assassin or a commander you
/// controlled dealt combat damage to a player this turn.
pub fn freerunning(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost,
        condition: Some(crate::effect::Predicate::FreerunningUnlocked { who: PlayerRef::You }),
        freerunning: true,
        ..Default::default()
    }
}
/// Web-slinging (CR 702.188): cast for `mana_cost` and return a tapped
/// creature you control to its owner's hand.
pub fn web_slinging(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost,
        return_to_hand: Some((crate::card::SelectionRequirement::Creature.and(crate::card::SelectionRequirement::Tapped), 1)),
        ..Default::default()
    }
}
pub fn dash(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, dash: true, ..Default::default() }
}

/// Blitz (CR 702.152) alternative cost: cast for `mana_cost`; the creature
/// gains haste and "When this creature dies, draw a card," then is
/// sacrificed at the beginning of the next end step.
pub fn blitz(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, blitz: true, ..Default::default() }
}

/// Warp (EOE) alternative cost: cast a permanent for `mana_cost`; at the next
/// end step it's exiled and may be recast from exile on a later turn (full
/// cost). Casting via warp also satisfies Void this turn.
pub fn warp(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, warp: true, ..Default::default() }
}

/// Evoke (CR 702.74) alternative cost: cast for `mana_cost`; the creature is
/// sacrificed when it enters, after its ETB triggers resolve.
pub fn evoke(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, evoke_sacrifice: true, ..Default::default() }
}

/// Sneak (CR 702.190) alternative cost: during your declare blockers step
/// (instant timing via `flash`), cast for `mana_cost` by returning one
/// unblocked creature you control to its owner's hand. Pair with
/// `Keyword::Sneak(mana_cost)` for display.
pub fn sneak(mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    use crate::card::SelectionRequirement as R;
    crate::card::AlternativeCost {
        mana_cost,
        flash: true,
        condition: Some(Predicate::CurrentStepIs(crate::turn_step::TurnStep::DeclareBlockers)),
        return_to_hand: Some((R::Creature.and(R::IsUnblocked).and(R::ControlledByYou), 1)),
        sneak: true,
        ..Default::default()
    }
}

/// Impending N—[cost] (CR 702.183): cast for `mana_cost`; the permanent enters
/// with `n` time counters and isn't a creature until they tick off (one per
/// controller's upkeep). Pair with `Keyword::Impending(n)` on the card so the
/// layer + upkeep machinery recognizes it.
pub fn impending(n: u32, mana_cost: crate::mana::ManaCost) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost { mana_cost, impending: n, ..Default::default() }
}

/// Surge (CR 702.117) alternative cost: cast for `mana_cost` if you or a
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
/// Wall of Resurgence / Cyclone Sire / Noyan Dar: put `counters` +1/+1 counters on the
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

/// "Target land [you control] becomes an N/N [Elemental] creature [with
/// haste] until end of turn. It's still a land." — no counters, so nothing
/// outlives the turn (Embodiment cycle, Elemental Uprising, Lifespark Spellbomb).
pub fn animate_land_until_eot(
    slot: u8,
    filter: SelectionRequirement,
    size: i32,
    creature_types: Vec<crate::card::CreatureType>,
    keywords: Vec<Keyword>,
) -> Effect {
    Effect::BecomeCreature {
        what: Selector::TargetFiltered { slot, filter },
        power: Value::Const(size),
        toughness: Value::Const(size),
        creature_types,
        keywords,
        duration: Duration::EndOfTurn,
    }
}

/// "… becomes a [P/T] [colors] [type] creature …" (CR 105.3, layer 5):
/// `animate` (a [`Effect::BecomeCreature`]) followed by a `SetColors` on the
/// same permanents for the same duration. A creature-land or Keyrune is
/// colorless until it animates; the printed color matters to protection,
/// "nonblack creature" removal and color-counting.
pub fn colored_animation(animate: Effect, colors: &[crate::mana::Color]) -> Effect {
    let Effect::BecomeCreature { what, duration, .. } = &animate else {
        // A catalog mistake, not a game state: keep the animation unpainted
        // (a release build) rather than abort the catalog load.
        debug_assert!(false, "colored_animation wraps an Effect::BecomeCreature");
        return animate;
    };
    let paint = Effect::BecomeColor {
        what: what.clone(),
        colors: colors.to_vec(),
        duration: *duration,
        additive: false,
    };
    Effect::Seq(vec![animate, paint])
}

/// `create`, then "It gains / They gain `keywords` until end of turn." — the
/// grant lands on the tokens this resolution made, never on their definition.
pub fn tokens_gain_until_eot(create: Effect, keywords: &[Keyword]) -> Effect {
    let mut seq = vec![create];
    seq.extend(keywords.iter().map(|k| Effect::GrantKeyword {
        what: Selector::LastCreatedTokens,
        keyword: k.clone(),
        duration: Duration::EndOfTurn,
    }));
    Effect::Seq(seq)
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
        awaken: true,
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

/// Spectacle (CR 702.137) alternative cost: cast for `mana_cost` rather
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
            condition: None,
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
            condition: None,
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
/// CR 506.2 — "whenever this creature attacks a player": attacking a
/// planeswalker or battle doesn't trigger it (Elder Brain, Shredder).
pub fn on_attack_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::EntityMatches {
            what: Selector::This,
            filter: SelectionRequirement::IsAttackingOpponentPlayer,
        }),
        effect,
    }
}
/// CR 701.43d — the **exert bonus**: "you may exert this creature as it
/// attacks. When you do, `effect`." The two are a linked pair (CR 607.2h), so
/// this rides `EventKind::Exerted` rather than `Attacks`: an attack without
/// the optional cost (CR 508.1g) gets neither the bonus nor the skipped
/// untap. Pair it with `Keyword::Exert`, which is what offers the cost.
pub fn on_exert(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Exerted, EventScope::SelfSource),
        effect,
    }
}
/// "Whenever you attack, `effect`" (CR 508) — fires once per combat for the
/// attacking player, regardless of attacker count (Razorkin Hordecaller).
pub fn on_you_attack(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
        effect,
    }
}
/// Valiant (CR 702.176): "Whenever this creature becomes the target of a spell
/// or ability you control for the first time each turn, `effect`." A
/// once-per-turn `BecameTarget` trigger — the implicit source==target guard
/// pins it to this creature and `YourControl` refines on your own casts.
pub fn valiant(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl).once_per_turn(),
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
/// "Revolt — this enters with `n` counters if ..." as an `enters_with_counters`
/// value (CR 614.12 — a replacement, not a trigger): `n` while revolt is
/// active, else 0.
pub fn revolt_value(n: i32) -> Value {
    Value::IfPred {
        pred: Box::new(Predicate::RevoltActive { who: PlayerRef::You }),
        then: Box::new(Value::Const(n)),
        else_: Box::new(Value::Const(0)),
    }
}
/// "Raid — this enters with `n` counters if you attacked this turn", the
/// `enters_with_counters` value.
pub fn raid_value(n: i32) -> Value {
    Value::IfPred {
        pred: Box::new(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
        then: Box::new(Value::Const(n)),
        else_: Box::new(Value::Const(0)),
    }
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
                    .and(SelectionRequirement::ControlledByDefendingPlayer),
            ),
        },
    }
}
/// "Whenever this creature blocks or becomes blocked by a [`filter`] creature,
/// destroy that creature at end of combat" (Dread Specter, Rock Basilisk,
/// Mercadian Masques' punishers). The partner is remembered on the source at
/// block time: by the end-of-combat step the block map is gone, so reading it
/// there would find nothing.
pub fn combat_partner_punisher(filter: SelectionRequirement) -> Vec<TriggeredAbility> {
    let body = Effect::Seq(vec![
        Effect::RememberPermanentOnSource {
            what: Selector::MatchingAmong {
                inner: Box::new(Selector::CreaturesInCombatWith(Box::new(Selector::This))),
                filter,
            },
        },
        Effect::AtEndOfCombat {
            body: Box::new(Effect::Destroy { what: Selector::ChosenPermanentOfSource }),
        },
    ]);
    [crate::card::EventKind::Blocks, crate::card::EventKind::BecomesBlocked]
        .into_iter()
        .map(|kind| TriggeredAbility {
            event: crate::card::EventSpec::new(kind, crate::card::EventScope::SelfSource),
            effect: body.clone(),
        })
        .collect()
}

pub fn on_dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect,
    }
}

/// CR 702.140f — "Whenever this creature mutates, `effect`."
pub fn on_mutate(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Mutated, EventScope::SelfSource),
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

/// "Whenever another creature you control dies, `effect`" — the `CreatureDied /
/// AnotherOfYours` scope, which excludes the source's own death (Felisa, Fang
/// of Silverquill).
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

/// "Whenever you attack with three or more creatures, `body`" — one trigger
/// per declaration, whether or not this creature attacks (unlike battalion).
pub fn you_attack_with_three_or_more(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
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

/// Evolve with a "whenever this creature evolves, [rider]" payoff. CR 702.100b:
/// it evolves when the evolve ability resolves and puts the counter, so the
/// rider runs after the counter in the same resolution (a parallel trigger
/// fired before the counter landed, and a Watchful Radstag copied itself
/// without end). A counter that can't be put (Blightbeetle) is no evolution, so
/// the rider is gated on it landing.
pub fn evolve_then(rider: Effect) -> TriggeredAbility {
    let mut t = evolve();
    let landed = Predicate::CounterWouldLand { what: Selector::This, kind: crate::card::CounterType::PlusOnePlusOne };
    t.effect = Effect::Seq(vec![
        t.effect,
        Effect::If { cond: landed, then: Box::new(rider), else_: Box::new(Effect::Noop) },
    ]);
    t
}

/// "Whenever [event], you may [body]. Do this only once each turn." The use
/// is spent when the "may" is taken, not when the trigger fires — a declined
/// copy leaves the turn's one for a later trigger (Donal, Ondu Spiritdancer,
/// Ancient Cornucopia).
pub fn may_once_each_turn(event: EventSpec, description: &str, body: Effect) -> TriggeredAbility {
    once_each_turn_on_take(TriggeredAbility {
        event,
        effect: Effect::MayDo { description: description.into(), body: Box::new(body) },
    })
}

/// [`may_once_each_turn`] for a trigger already written with
/// `.once_per_turn()` around a `MayDo` / `MayPayLife`: the per-turn limit
/// moves off the trigger and onto the taken branch. A mandatory body is
/// spent as it resolves, which is the same limit.
pub fn once_each_turn_on_take(mut t: TriggeredAbility) -> TriggeredAbility {
    let gate = Predicate::Not(Box::new(Predicate::SourceDoneThisTurn));
    t.event.once_per_turn = false;
    t.event.filter = Some(match t.event.filter.take() {
        Some(f) => Predicate::All(vec![f, gate]),
        None => gate,
    });
    let mark = |body: Box<Effect>| Box::new(Effect::Seq(vec![Effect::MarkDoneThisTurn, *body]));
    let taken = match t.effect {
        Effect::MayDo { description, body } => Effect::MayDo { description, body: mark(body) },
        Effect::MayPayLife { description, amount, body, else_ } => {
            Effect::MayPayLife { description, amount, body: mark(body), else_ }
        }
        other => Effect::Seq(vec![Effect::MarkDoneThisTurn, other]),
    };
    // Asked again as it resolves: two triggers from one event both pass the
    // filter, and not every dispatcher re-checks it (CR 603.4).
    t.effect = Effect::If {
        cond: Predicate::Not(Box::new(Predicate::SourceDoneThisTurn)),
        then: Box::new(taken),
        else_: Box::new(Effect::Noop),
    };
    t
}

/// Eerie shortcut (DSK ability word): "Whenever an enchantment you control
/// enters and whenever you fully unlock a Room, [body]." Returns the two
/// triggered abilities that share `body` — an enchantment-ETB watcher and a
/// `RoomFullyUnlocked` watcher (both `YourControl`).
pub fn eerie(body: Effect) -> Vec<TriggeredAbility> {
    vec![
        TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: body.clone(),
        },
        TriggeredAbility {
            event: EventSpec::new(EventKind::RoomFullyUnlocked, EventScope::YourControl),
            effect: body,
        },
    ]
}

/// "When this creature enters, each opponent loses `amount` life and you gain
/// `amount` life."
pub fn etb_drain(amount: i32) -> TriggeredAbility {
    etb(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// "When this creature enters, you gain `amount` life."
pub fn etb_gain_life(amount: i32) -> TriggeredAbility {
    etb(Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// "When this creature dies, you gain `amount` life" (the Pest token).
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

/// "When this creature enters, each opponent loses `amount` life" — the
/// asymmetric [`etb_drain`], with no life gain.
pub fn etb_drain_each_opp(amount: i32) -> TriggeredAbility {
    etb(Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(amount),
    })
}

/// "When this creature enters, draw a card, then discard a card."
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

/// Strixhaven Magecraft: "Whenever you cast or copy an instant or sorcery
/// spell, `effect`" — the cast trigger plus [`cast_is_instant_or_sorcery`].
pub fn magecraft(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_instant_or_sorcery()),
        effect,
    }
}

/// "Whenever you cast a noncreature spell" — any noncreature type, cast only
/// (Kessig Flamebreather, The Mechanist; `magecraft` is instants and sorceries).
pub fn on_cast_noncreature(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
        effect,
    }
}

/// Mark a cast trigger "whenever you cast **or copy** …" (`EventSpec::or_copy`):
/// the printed Magecraft wording. `magecraft` itself stays cast-only — it is
/// also the builder behind plain "whenever you cast an instant or sorcery
/// spell" cards (Young Pyromancer, Talrand), which a copy must not trigger.
pub fn with_copies(mut t: TriggeredAbility) -> TriggeredAbility {
    t.event = t.event.or_copy();
    t
}

/// Flurry (Tarkir: Dragonstorm): "Whenever you cast your second spell each
/// turn, `effect`." Reuses the `SpellsCastThisTurnEquals` predicate
/// (already incremented for the current cast at trigger time).
pub fn flurry(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::SpellsCastThisTurnEquals {
                who: crate::effect::PlayerRef::You,
                count: Value::Const(2),
            },
        ),
        effect,
    }
}

/// Spiritcraft: "Whenever you cast a Spirit or Arcane spell, `effect`."
/// (Kamigawa — Teller of Tales, Kami of Fire's Roar.)
pub fn spiritcraft(effect: Effect) -> TriggeredAbility {
    use crate::card::{CreatureType, SpellSubtype};
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::CastSpellMatches(
                SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                    .or(SelectionRequirement::HasSpellSubtype(SpellSubtype::Arcane)),
            ),
        ),
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
            filter: None,
        },
    }
}

/// Ripple N (CR 702.20). Wires the "when you cast this spell" trigger whose
/// body is [`Effect::Ripple`] — reveal the top N, free-cast same-named copies,
/// bottom the rest.
pub fn ripple(n: u32) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
        effect: Effect::Ripple { n: Value::Const(n as i32), of_trigger_spell: false },
    }
}

/// Station (CR 702.184): "Tap another untapped creature you control: Put a
/// number of charge counters on this permanent equal to the tapped creature's
/// power. Activate only as a sorcery." Every station card carries this ability
/// (CR 722.4); pair it with `CardDefinition.station` bands.
pub fn station() -> ActivatedAbility {
    ActivatedAbility {
        sorcery_speed: true,
        tap_other_filter: Some(crate::card::SelectionRequirement::Creature),
        effect: Effect::AddCounter {
            what: Selector::This,
            kind: crate::card::CounterType::Charge,
            amount: Value::TappedForCostPower,
        },
        ..Default::default()
    }
}

/// Unearth `cost` (CR 702.84). "[cost]: Return this card from your graveyard to
/// the battlefield. It gains haste. Exile it at the beginning of the next end
/// step or if it would leave the battlefield" — a sorcery-speed
/// graveyard-activated ability; the leave clause is a CR 614 replacement
/// (`ExileIfLeavesBattlefield`), so a dying unearthed creature is exiled.
pub fn unearth(cost: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        from_graveyard: true,
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::ExileIfLeavesBattlefield { what: Selector::This },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: crate::card::Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Exile { what: Selector::This }),
            },
        ]),
        ..Default::default()
    }
}

/// Squad (CR 702.157) — the ETB trigger that mints one token copy of this
/// creature per time its squad cost was paid (`Value::SquadCount`). Pair with
/// `Keyword::Squad(cost)` on the card.
pub fn squad_etb() -> TriggeredAbility {
    etb(Effect::CreateTokenCopyOf {
        extra_keywords: vec![],
        who: PlayerRef::You,
        count: Value::SquadCount,
        source: Selector::This,
        extra_creature_types: Vec::new(),
        extra_card_types: Vec::new(),
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    })
}

/// Demonstrate (CR 702.144) — the self-cast trigger that copies the spell for
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

/// A Magecraft trigger that pumps the source itself until end of turn (Symmetry
/// Sage's +1/+0): [`magecraft`] with a `PumpPT` on `Selector::This`.
pub fn magecraft_self_pump(power: i32, toughness: i32) -> TriggeredAbility {
    magecraft(Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    })
}

/// A Repartee trigger that pumps the source itself: [`magecraft_self_pump`]
/// gated on the "spell targets a creature" predicate (Rehearsed Debater).
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

/// Quandrix "spell with {X} in its mana cost" trigger: fires on any spell the
/// controller casts whose printed cost has an `{X}`
/// (`Predicate::CastSpellHasX`; Geometer's Arthropod).
pub fn cast_has_x_trigger(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellHasX),
        effect,
    }
}

/// Prowess: "Whenever you cast a noncreature spell, this creature gets +1/+1
/// until end of turn," pumping `Selector::This`. Card factories declaring
/// `Keyword::Prowess` add it to make the keyword functional; the keyword stays
/// in `card.keywords` for display and "prowess matters" filters.
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

/// "… you may discard a card. If you do, [body]" — the contingent half of a
/// loot. `Effect::Discard` no-ops on an empty hand, so a bare `Seq([Discard,
/// Draw])` hands the reward out anyway (an opponent with an empty hand took
/// Lorehold, the Historian's upkeep loot as a free draw). Gates `body` on
/// `Value::CardsDiscardedThisEffect`, which `discard_card` bumps and which
/// resets between resolutions.
///
/// ⚠ Not for every discard-then-draw: a printed "discard a card, then draw a
/// card" (Cathartic Reunion) does draw on an empty hand. Use this only where
/// the card says "if you do" / "to draw".
pub fn if_discarded(body: Effect) -> Effect {
    Effect::If {
        cond: Predicate::ValueAtLeast(Value::CardsDiscardedThisEffect, Value::Const(1)),
        then: Box::new(body),
        else_: Box::new(Effect::Noop),
    }
}

/// SOS Increment: "Whenever you cast a spell, if the amount of mana you spent
/// is greater than this creature's power or toughness, [body]," via
/// `Predicate::IncrementSatisfied` (the cast spell's stashed `mana_spent` vs
/// the listener's effective P/T). Body-agnostic (Pensive Professor: a counter
/// and scry 1). The predicate is an intervening "if" (CR 603.4), checked both
/// as the trigger event filter and again at resolution, so P/T gained in
/// between can suppress the body.
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

/// SOS Increment payoff putting one +1/+1 counter on the source
/// ([`increment_trigger`] with `AddCounter` on `Selector::This`; Cuboid
/// Colony).
pub fn increment_self_plus_one() -> TriggeredAbility {
    use crate::card::CounterType;
    increment_trigger(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    })
}

/// Strixhaven Opus: "Whenever you cast an instant or sorcery spell, [body]. If
/// five or more mana was spent to cast that spell, [bigger body] instead" — an
/// `If` on `Predicate::CastSpellManaSpentAtLeast(5)` (Deluge Virtuoso).
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

/// "Create a [token] with [keyword] until [duration]": `Seq([CreateToken,
/// GrantKeyword(LastCreatedToken, …)])` (Lorehold Skirmish).
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
            definition: std::sync::Arc::new(token),
        },
        Effect::GrantKeyword {
            what: Selector::LastCreatedToken,
            keyword,
            duration,
        },
    ])
}

/// "Create a [token] with N [counter] counters on it." The counters are part of
/// the entry (`TokenDefinition::enters_with_counters`), so a 0/0 body never
/// meets a state-based sweep bare.
pub fn create_token_with_counter(
    who: PlayerRef,
    count: i32,
    token: crate::card::TokenDefinition,
    counter: crate::card::CounterType,
    counter_n: i32,
) -> Effect {
    Effect::CreateToken {
        who,
        count: Value::Const(count),
        definition: std::sync::Arc::new(token.entering_with(counter, Value::Const(counter_n))),
    }
}

/// A Magecraft trigger pumping a caller-supplied target (Withergrowth
/// Apprentice: target friendly creature). Pass a `target_filtered(...)`
/// selector so the auto-target picker chooses at resolution.
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

/// A Magecraft trigger dealing `amount` damage to any target (Creature ∨ Player
/// ∨ Planeswalker) — Lorehold Apprentice and many other STX cards.
pub fn magecraft_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    magecraft(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::any_target(),
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

/// A Magecraft trigger gaining `amount` life.
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
        definition: std::sync::Arc::new(definition),
        count: Value::Const(count),
    })
}

/// "When this creature enters, scry `amount`."
pub fn etb_scry(amount: i32) -> TriggeredAbility {
    etb(Effect::Scry {
        who: PlayerRef::You,
        amount: Value::Const(amount),
    })
}

/// CR 701.44 — "this permanent explores." Reveal the top card of your
/// library; if it's a land, put it into your hand, otherwise put a
/// +1/+1 counter on the exploring permanent.
pub fn explore() -> Effect {
    Effect::Explore { who: Selector::This }
}

/// CR 701.16a — "investigate `n` times": `n` Clue tokens, each one an
/// investigation (`{2}, Sacrifice this artifact: Draw a card.`).
pub fn investigate(n: u32) -> Effect {
    investigate_by(PlayerRef::You, Value::Const(n as i32))
}

/// CR 701.16a — `who` investigates `count` times ("its controller
/// investigates", "investigate X times").
pub fn investigate_by(who: PlayerRef, count: Value) -> Effect {
    Effect::Investigate {
        body: Box::new(Effect::CreateToken {
            who,
            count,
            definition: std::sync::Arc::new(crate::tokens::clue_token()),
        }),
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

/// CR 701.37 — "`cost`: Monstrosity `n`." Grows the source to monstrous once.
///
/// ⚠ Not sorcery-speed: CR 701.37a gives monstrosity no timing rider, and
/// instant-speed Polukranos in combat is the card's point. This helper and
/// `cn2.rs`'s copy once carried `sorcery_speed: true`, taking the ability away
/// from every monstrosity card for most of the turn (found by
/// `every_card_has_the_activation_timing_it_prints`).
pub fn monstrosity(cost: crate::mana::ManaCost, n: i32) -> ActivatedAbility {
    ActivatedAbility {
        energy_cost: 0,
        mana_cost: cost,
        effect: Effect::Monstrosity { n: Value::Const(n) },
        ..Default::default()
    }
}

/// "When this becomes monstrous, …" trigger (CR 701.37).
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

/// "When this creature enters, draw `amount` cards" (Elvish Visionary).
pub fn etb_draw(amount: i32) -> TriggeredAbility {
    etb(Effect::Draw {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft — "draw a card, then discard a card."
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

/// Magecraft — "scry `amount`."
pub fn magecraft_scry(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Scry {
        who: PlayerRef::You,
        amount: Value::Const(amount),
    })
}

/// Magecraft — "create `count` copies of `definition`."
pub fn magecraft_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        definition: std::sync::Arc::new(definition),
        count: Value::Const(count),
    })
}

/// "Whenever you cast or copy an instant or sorcery spell, create `count` of
/// `definition`, then each opponent loses `amount` life and you gain `amount`
/// life" — [`magecraft_mint_token`] plus [`magecraft_drain`]. Mints first, so
/// the token is on the battlefield when a gained-life or sacrifice payoff sees
/// the drain.
pub fn magecraft_mint_and_drain(
    definition: crate::card::TokenDefinition,
    count: i32,
    amount: i32,
) -> TriggeredAbility {
    magecraft(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            definition: std::sync::Arc::new(definition),
            count: Value::Const(count),
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// "Whenever you cast or copy an instant or sorcery spell, put a +1/+1 counter
/// on target creature you control" (Quandrix Coursemage), targeting `Creature ∧
/// ControlledByYou`.
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

/// "When this creature enters, surveil `amount`."
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

/// "When this creature enters, it deals `amount` damage to any target" —
/// `magecraft_ping_any` for ETB.
pub fn etb_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    etb(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::any_target(),
        ),
        amount: Value::Const(amount),
    })
}

/// "When this creature enters, it deals `amount` damage to target creature" (no
/// player or planeswalker option).
pub fn etb_ping_creature(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    etb(Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(amount),
    })
}

/// Magecraft — "deal `amount` damage to target creature."
pub fn magecraft_ping_creature(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    magecraft(Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(amount),
    })
}

/// "When this creature enters, each opponent loses `drain` life and you gain
/// `drain` life. Surveil `surveil`."
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

/// "When this creature enters, each opponent loses `drain` life and you gain
/// `drain` life. Draw a card." [`etb`] with `Seq([Drain, Draw])`.
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

/// "Whenever this creature attacks, create a `token`."
pub fn on_attack_create_token(token: crate::card::TokenDefinition) -> TriggeredAbility {
    on_attack(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: std::sync::Arc::new(token),
    })
}

/// Create N `token`s as a standalone effect (a spell body or a `Seq` step, not
/// an ETB trigger): `Effect::CreateToken` with `who: PlayerRef::You`.
pub fn mint_token(token: crate::card::TokenDefinition, count: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition: std::sync::Arc::new(token),
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

/// Mint N SOS Fractal tokens (0/0 G/U creature; usually paired with
/// `Effect::AddCounter` against `Selector::LastCreatedToken` to
/// stamp +1/+1 counters on entry).
pub fn mint_fractals(count: i32) -> Effect {
    let token = crate::tokens::fractal_token();
    mint_token(token, count)
}

/// Mint N Treasure tokens (`{T}, Sacrifice: add one mana of any
/// color`). Uses [`crate::tokens::treasure_token`].
/// Mint N tapped Treasure tokens ("create N tapped Treasure tokens").
pub fn mint_tapped_treasures(count: i32) -> Effect {
    mint_token(crate::tokens::tapped_treasure_token(), count)
}

pub fn mint_treasures(count: i32) -> Effect {
    let token = crate::tokens::treasure_token();
    mint_token(token, count)
}

/// Create N Lorehold Spirit tokens (2/2 R/W).
pub fn mint_lorehold_spirits(count: i32) -> Effect {
    let token = crate::tokens::lorehold_spirit_token();
    mint_token(token, count)
}

/// `Seq([Drain(amount), Draw(1)])` as a raw `Effect`, for drain-and-cantrip
/// spells.
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

/// `Seq([Drain(amount), Surveil(N)])` as a raw `Effect`.
pub fn drain_and_surveil(amount: i32, surveil: i32) -> Effect {
    Effect::Seq(vec![
        drain(amount),
        Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(surveil),
        },
    ])
}

/// "When this creature enters, tap target creature an opponent controls."
pub fn etb_tap_opp_creature() -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    etb(Effect::Tap {
        what: target_filtered(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByOpponent),
        ),
    })
}

/// "When this creature dies, each opponent loses `amount` life" — the
/// asymmetric [`dies_drain`] with no life gain (Pest Mawcrawler).
pub fn dies_lose_life_each_opp(amount: i32) -> TriggeredAbility {
    on_dies(Effect::LoseLife {
        who: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(amount),
    })
}

/// Magecraft — "each opponent loses `amount` life and you gain `amount` life."
/// Unlike [`magecraft_drain_each_opp`] (opponents only) and
/// [`magecraft_drain_target`] (one target opponent).
pub fn magecraft_drain(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// "Whenever this creature attacks, each opponent loses `amount` life and you
/// gain `amount` life" — `etb_drain` / `dies_drain` for the attack event.
pub fn on_attack_drain(amount: i32) -> TriggeredAbility {
    on_attack(Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(amount),
    })
}

/// "Whenever this creature attacks, you gain `amount` life" — the gain-only
/// sibling of [`on_attack_drain`].
pub fn on_attack_gain_life(amount: i32) -> TriggeredAbility {
    on_attack(Effect::GainLife {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// "Whenever this creature attacks, it deals `amount` damage to any target."
pub fn on_attack_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    on_attack(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::any_target(),
        ),
        amount: Value::Const(amount),
    })
}

/// "When this creature dies, it deals `amount` damage to any target" (Mogg
/// Fanatic-style) — `on_attack_ping_any` / `etb_ping_any` for the death event.
pub fn dies_ping_any(amount: i32) -> TriggeredAbility {
    use crate::card::SelectionRequirement;
    on_dies(Effect::DealDamage {
        to: target_filtered(
            SelectionRequirement::any_target(),
        ),
        amount: Value::Const(amount),
    })
}

/// "When this creature dies, create `count` copies of `definition`" (a Pest
/// that mints a Pest on death).
pub fn dies_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition: std::sync::Arc::new(definition),
    })
}

/// "Whenever another creature you control dies, create one copy of
/// `definition`" — `dies_mint_token` for the another-dies scope (Witherbloom
/// aristocrats).
pub fn on_other_dies_mint_token(
    definition: crate::card::TokenDefinition,
    count: i32,
) -> TriggeredAbility {
    on_other_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(count),
        definition: std::sync::Arc::new(definition),
    })
}

/// "Whenever you cast or copy an instant or sorcery spell, create a 2/2 R/W
/// Spirit token" — `magecraft_mint_token` with the shared
/// `lorehold_spirit_token()`.
pub fn magecraft_mint_spirit() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: std::sync::Arc::new(crate::tokens::lorehold_spirit_token()),
    })
}

/// "Whenever you cast or copy an instant or sorcery spell, draw a card"
/// (Archmage Emeritus). Unlike [`magecraft_loot`], no discard.
pub fn magecraft_draw(amount: i32) -> TriggeredAbility {
    magecraft(Effect::Draw {
        who: Selector::You,
        amount: Value::Const(amount),
    })
}

/// "Whenever you cast or copy an instant or sorcery spell, create a Treasure
/// token."
pub fn magecraft_treasure() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: std::sync::Arc::new(crate::tokens::treasure_token()),
    })
}

/// "Whenever this creature attacks, draw a card, then discard a card."
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

/// "Whenever this creature attacks and isn't blocked, …" (CR 509.3g), on
/// `EventKind::AttacksAndIsntBlocked`: fires once per attacker left with no
/// blockers after the declare-blockers step.
pub fn on_unblocked(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
        effect,
    }
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

/// Heroic (CR 702.85): "Whenever you cast a spell that targets this creature,
/// `body`." A `SpellCast / YourControl` trigger gated on
/// `Predicate::CastSpellTargetsSource`.
pub fn heroic(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellTargetsSource),
        effect: body,
    }
}

/// Dethrone (CR 702.105): "Whenever this creature attacks the player with
/// the most life or tied for most life, put a +1/+1 counter on it." An
/// `Attacks / SelfSource` trigger gated on `PlayerHasMostLife` for the
/// defending player; the counter lands on `This`.
pub fn dethrone() -> TriggeredAbility {
    use crate::card::CounterType;
    TriggeredAbility {
        // CR 702.105a — "attacks the PLAYER with the most life": a planeswalker
        // or battle attack doesn't count, though `DefendingPlayer` names its
        // controller / protector.
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::All(vec![
            Predicate::EntityMatches {
                what: Selector::This,
                filter: crate::card::SelectionRequirement::IsAttackingOpponentPlayer,
            },
            Predicate::PlayerHasMostLife { who: PlayerRef::DefendingPlayer },
        ])),
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

/// Myriad (CR 702.116): an `Attacks / SelfSource` trigger minting a
/// tapped+attacking copy of the source for each other opponent, exiled
/// at end of combat.
pub fn myriad() -> TriggeredAbility {
    on_attack(Effect::Myriad)
}

/// Enlist (CR 702.154): an `Attacks / SelfSource` trigger that taps a
/// nonattacking creature and adds its power to the attacker EOT.
pub fn enlist() -> TriggeredAbility {
    on_attack(Effect::Enlist)
}

/// Mobilize N (CR 702.181): "Whenever this creature attacks, create N
/// 1/1 red Warrior creature tokens that are tapped and attacking.
/// Sacrifice them at the beginning of the next end step." The tokens stay
/// through the postcombat main phase and leave no longer attacking (Zurgo
/// Stormrender reads the difference). An `Attacks / SelfSource` trigger over
/// [`Effect::CreateTokenAttacking`].
pub fn mobilize(n: i32) -> TriggeredAbility {
    mobilize_value(Value::Const(n))
}

/// Mobilize with a runtime count (CR 702.181) — "Mobilize X, where X is …"
/// (Avenger of the Fallen: creature cards in your graveyard; Devoted Mardu:
/// devotion). Same tapped-and-attacking Warrior tokens, sacrificed at end of
/// combat, but the count is evaluated when the attack trigger resolves.
pub fn mobilize_value(count: Value) -> TriggeredAbility {
    on_attack(Effect::CreateTokenAttacking {
        who: PlayerRef::You,
        count,
        definition: std::sync::Arc::new(crate::card::TokenDefinition {
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
        }),
        cleanup: AttackingTokenCleanup::SacrificeAtNextEndStep,
        defender: None,
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
        definition: std::sync::Arc::new(crate::card::TokenDefinition {
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
            ..Default::default()
        }),
    })
}

/// CR 702.124c — "Partner with [name]"'s printed trigger: "When this creature
/// enters, target player may put [name] into their hand from their library,
/// then shuffle." The keyword itself is deck construction (CR 702.124a); this
/// trigger is carried explicitly, as [`extort`] is. Both `who`s are
/// `Target(0)`: the card says "target player", even though the pair usually
/// points it at its own controller.
pub fn partner_with_search(name: &str) -> TriggeredAbility {
    etb(Effect::MayDoBy {
        who: PlayerRef::Target(0),
        description: format!("Put {name} into your hand from your library?"),
        body: Box::new(Effect::SearchUpToN {
            who: PlayerRef::Target(0),
            filter: crate::card::SelectionRequirement::HasName(name.to_string()),
            to: ZoneDest::Hand(PlayerRef::Target(0)),
            count: Value::ONE,
        }),
    })
}

/// Extort (CR 702.101a): "Whenever you cast a spell, you may pay {W/B}. If
/// you do, each opponent loses 1 life and you gain life equal to the total
/// life lost this way." A `SpellCast / YourControl` trigger whose body is a
/// `MayPay` over [`drain_life_lost`]. Basilica Screecher, Crypt Ghast.
pub fn extort() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
        effect: Effect::MayPay {
            description: "Extort — pay {W/B}: drain 1".into(),
            mana_cost: crate::mana::cost(&[crate::mana::hybrid(Color::White, Color::Black)]),
            // CR 702.101a — "you gain life equal to the total life lost this
            // way": one per opponent at a table, not one.
            body: Box::new(drain_life_lost(Value::ONE)),
            else_: None,
        },
    }
}

/// Tribute N (CR 702.104): "As this creature enters, an opponent of your
/// choice may put N +1/+1 counters on it. When it enters, if tribute
/// wasn't paid, `otherwise`." The opponent answers via the synchronous
/// decider; AutoDecider declines, so the trigger half fires.
pub fn tribute(n: u32, otherwise: Effect) -> TriggeredAbility {
    etb(Effect::Tribute { n, otherwise: Box::new(otherwise) })
}

/// Exploit (CR 702.110a): "When this creature enters, you may sacrifice
/// a creature. When you exploit a creature, `payoff`." Modeled as an ETB
/// `MayDo([Sacrifice 1 creature (this can be itself), payoff])`. Declining
/// the sacrifice skips the payoff (CR 702.110b — the exploit trigger only
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
            // "When this exploits a creature": only once one was sacrificed.
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::Seq(vec![Effect::NoteExploited, payoff])),
                else_: Box::new(Effect::Noop),
            },
        ])),
    })
}

/// Devour N (CR 702.83): "As this creature enters, you may sacrifice any
/// number of creatures. It enters with N +1/+1 counters on it for each
/// creature sacrificed this way." Goes in `CardDefinition.as_enters_effect`
/// so a printed 0/0 devourer never faces SBAs at 0 toughness. AutoDecider
/// sacrifices none.
pub fn devour(n: i32) -> Effect {
    devour_filter(n, SelectionRequirement::Creature)
}

/// Typed Devour — "Devour land N" (Famished Worldsire) sacrifices `filter`
/// permanents instead of creatures. Same counter math as [`devour`].
pub fn devour_filter(n: i32, filter: SelectionRequirement) -> Effect {
    use crate::card::CounterType;
    Effect::SacrificeAnyNumber {
        who: PlayerRef::You,
        filter: filter.and(SelectionRequirement::OtherThanSource),
        per_each: Box::new(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(n),
        }),
    }
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
                SelectionRequirement::InYourGraveyard
                    .and(SelectionRequirement::HasCreatureType(
                        crate::card::CreatureType::Spirit,
                    ))
                    .and(SelectionRequirement::ManaValueAtMost(n)),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        }),
    })
}

/// Adapt N (CR 701.46) — the *effect* of an Adapt activated ability: "If
/// this creature has no +1/+1 counters on it, put N +1/+1 counters on it."
/// Pair it with an `ActivatedAbility` carrying the adapt mana cost.
pub fn adapt(n: i32) -> Effect {
    Effect::Adapt { amount: Value::Const(n) }
}

/// Renown N (CR 702.111) — a combat trigger: "Whenever this creature
/// deals combat damage to a player, if it isn't renowned, put N +1/+1
/// counters on it and it becomes renowned." Gated on the real `renowned`
/// flag (`Predicate::SourceIsRenowned`), so it renowns exactly once and
/// unrelated +1/+1 counters don't suppress it.
pub fn renown(n: i32) -> TriggeredAbility {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec};
    // CR 702.112 — "if it isn't renowned, put N +1/+1 counters on it and it
    // becomes renowned." Gated on the real `renowned` flag (not a counter
    // heuristic) so unrelated +1/+1 counters don't suppress it.
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::Not(Box::new(Predicate::SourceIsRenowned)),
            then: Box::new(Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(n),
                },
                Effect::BecomeRenowned { what: Selector::This },
            ])),
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
            link_to_source: false,
            face_down: false,
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(1),
        },
    }
}

/// Recover (CR 702.59): "When a creature is put into your graveyard from the
/// battlefield, you may pay [cost]. If you do, return this card from your
/// graveyard to your hand. Otherwise, exile this card." A
/// `CreatureDied / FromYourGraveyard` trigger that fires while this card sits
/// in its owner's graveyard; the `MayPay` gate returns it to hand on payment
/// or exiles it on decline.
pub fn recover(cost: crate::mana::ManaCost) -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    use crate::effect::ZoneDest;
    TriggeredAbility {
        // CR 702.59a — "put into *your* graveyard": only a creature you own
        // arms recover, not any creature dying anywhere.
        event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyard)
            .with_filter(recover_subject_is_yours(false)),
        effect: Effect::MayPay {
            description: "Recover".into(),
            mana_cost: cost,
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Some(Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Exile })),
        },
    }
}

/// Outlast (CR 702.107) — the activated ability "{cost}, {T}: Put a
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

/// Transfigure (CR 702.71): "[Cost], Sacrifice this permanent: Search your
/// library for a creature card with the same mana value as this permanent and
/// put it onto the battlefield. Then shuffle. Activate only as a sorcery."
/// `sac_cost` records the sacrificed MV (threaded via `WithSacrificedPt`), so
/// the `ManaValueEqualsSacrificedPlus(0)` search filter matches it.
pub fn transfigure(mana_cost: crate::mana::ManaCost) -> ActivatedAbility {
    use crate::card::SelectionRequirement;
    use crate::effect::{PlayerRef, ZoneDest};
    ActivatedAbility {
        mana_cost,
        sac_cost: true,
        sorcery_speed: true,
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::ManaValueEqualsSacrificedPlus(0)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Forecast (CR 702.57): "Forecast — [cost], Reveal this card from your hand:
/// [effect]. Activate only during your upkeep and only once each turn." A
/// from-hand activated ability gated to the controller's upkeep; the card
/// stays in hand (no discard/exile cost).
pub fn forecast(mana_cost: crate::mana::ManaCost, effect: Effect) -> ActivatedAbility {
    use crate::turn_step::TurnStep;
    ActivatedAbility {
        mana_cost,
        from_hand: true,
        once_per_turn: true,
        condition: Some(Predicate::All(vec![
            Predicate::CurrentStepIs(TurnStep::Upkeep),
            Predicate::IsTurnOf(PlayerRef::You),
        ])),
        effect,
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

/// Fabricate N (CR 702.122) — ETB: "put N +1/+1 counters on this creature, or
/// create N 1/1 colorless Servo artifact creature tokens," as
/// `etb(ChooseMode([AddCounter, CreateToken]))`. AutoDecider takes the
/// counters; a scripted decider can pick the Servos.
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
        ..Default::default()
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
            definition: std::sync::Arc::new(servo),
        },
    ]))
}

/// "When this creature enters, create `count` copies of `definition`, then put
/// `counter_amount` +1/+1 counters on each" — `Seq([CreateToken,
/// AddCounter(LastCreatedToken, …)])` for Fractal-style tokens.
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
            definition: std::sync::Arc::new(definition),
        },
        Effect::AddCounter {
            what: Selector::LastCreatedToken,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(counter_amount),
        },
    ]))
}

/// "When this creature enters, create one copy of `definition` and each
/// opponent loses `amount` life and you gain `amount` life."
pub fn etb_mint_token_and_drain(
    definition: crate::card::TokenDefinition,
    amount: i32,
) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: std::sync::Arc::new(definition),
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// "When this creature enters, create one copy of `definition` and gain
/// `amount` life" — the gain-only [`etb_mint_token_and_drain`].
pub fn etb_mint_token_and_gain_life(
    definition: crate::card::TokenDefinition,
    amount: i32,
) -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: std::sync::Arc::new(definition),
        },
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// "When this creature enters, scry `scry_amount`, then draw a card."
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

/// Target friendly creature gets +`power`/+`toughness` and gains `keyword`
/// until end of turn — a raw `Effect` for combat tricks, not a trigger.
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

/// Magecraft — "scry `scry_amount`, then draw a card"; `etb_scry_and_draw` for
/// the magecraft event.
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

/// "When this creature dies, create a token and each opponent loses `amount`
/// life and you gain `amount` life" — `etb_mint_token_and_drain` for the death
/// event.
pub fn dies_mint_token_and_drain(
    definition: crate::card::TokenDefinition,
    amount: i32,
) -> TriggeredAbility {
    on_dies(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: std::sync::Arc::new(definition),
        },
        Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(amount),
        },
    ]))
}

/// Magecraft — "put a +1/+1 counter on this creature and each opponent loses
/// `amount` life and you gain `amount` life" (Witherbloom Reapcaster). The
/// counter goes on before the drain, so "if you gained life" payoffs see the
/// grown creature.
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

/// "When this creature enters, each opponent loses `amount` life, you gain
/// `amount` life, and you draw a card."
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

/// Magecraft — "create a 1/1 B/G Pest token with 'When this token attacks, you
/// gain 1 life'" (the shared `stx_pest_token()`).
pub fn magecraft_mint_pest() -> TriggeredAbility {
    magecraft(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: std::sync::Arc::new(crate::tokens::stx_pest_token()),
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
        definition: std::sync::Arc::new(crate::tokens::inkling_token()),
    })
}

/// Magecraft — "create a 0/0 G/U Fractal token with X +1/+1 counters on it", X
/// = `counters`, put on [`Selector::LastCreatedToken`] as
/// [`crate::card::CounterType::PlusOnePlusOne`].
pub fn magecraft_mint_fractal(counters: i32) -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::Seq(vec![
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: std::sync::Arc::new(crate::tokens::fractal_token()),
        },
        Effect::AddCounter {
            what: Selector::LastCreatedToken,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(counters),
        },
    ]))
}

/// "When this creature dies, create a 1/1 B/G Pest token" (the shared
/// `stx_pest_token()`).
pub fn dies_mint_pest() -> TriggeredAbility {
    on_dies(Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: std::sync::Arc::new(crate::tokens::stx_pest_token()),
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
        definition: std::sync::Arc::new(crate::tokens::lorehold_spirit_token()),
    })
}

/// Magecraft — "put a +1/+1 counter on this creature" (Inkling Bookbinder).
pub fn magecraft_add_counter_self() -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    })
}

/// Magecraft — "put a shield counter on this creature" (CR 122.1c: each shield
/// stops one damage or destroy event).
pub fn magecraft_add_shield_self() -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Shield,
        amount: Value::Const(1),
    })
}

/// Magecraft — "put a finality counter on this creature" (CR 122.1h: it is
/// exiled instead the next time it would die).
pub fn magecraft_add_finality_self() -> TriggeredAbility {
    use crate::card::CounterType;
    magecraft(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Finality,
        amount: Value::Const(1),
    })
}

/// Put a finality counter on target creature you control, as a spell or ability
/// body.
pub fn add_finality_to_target_creature() -> Effect {
    use crate::card::CounterType;
    Effect::AddCounter {
        what: target_filtered(crate::card::SelectionRequirement::Creature),
        kind: CounterType::Finality,
        amount: Value::Const(1),
    }
}

/// Put a shield counter (CR 122.1c) on target creature.
pub fn add_shield_to_target_creature() -> Effect {
    use crate::card::CounterType;
    Effect::AddCounter {
        what: target_filtered(crate::card::SelectionRequirement::Creature),
        kind: CounterType::Shield,
        amount: Value::Const(1),
    }
}

/// "You have at least `n` cards matching `filter` in your graveyard" —
/// `Predicate::SelectorCountAtLeast { sel: CardsInZone(You, Graveyard, filter),
/// n: Const(n) }`, the spell mastery / delirium / threshold-style gate (Searing
/// Blaze: any two).
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

/// The printed "spell mastery" gate — "if there are two or more instant and/or
/// sorcery cards in your graveyard": `cards_in_graveyard_at_least(IS, 2)`
/// (Fiery Impulse).
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

/// "Whenever this creature deals combat damage to a player, each opponent loses
/// `amount` life and you gain `amount` life" — the drain sibling of
/// `on_combat_damage_to_player_gain_life`.
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

/// Modular N (CR 702.43), the dies half: "When this creature dies, you may put
/// its +1/+1 counters on target artifact creature." Pair with
/// `enters_with_counters: Some((PlusOnePlusOne, Const(n)))`. The source is
/// already in the graveyard, so the target gets the source's last-known +1/+1
/// count (`Value::CountersOn { This }`, as Hangarback Walker reads).
pub fn modular_dies() -> TriggeredAbility {
    on_dies(Effect::MayDo {
        description: "Put +1/+1 counters on target artifact creature".into(),
        body: Box::new(Effect::ModularCounters {
            what: target_filtered(
                SelectionRequirement::Artifact.and(SelectionRequirement::Creature),
            ),
        }),
    })
}

/// Embalm (CR 702.128) / Eternalize (CR 702.129) — the activated ability:
/// "[cost], Exile this card from your graveyard: Create a token that's a
/// copy of it, except it's a [white/black] Zombie [with no mana cost
/// / and 4/4]. Activate only as a sorcery." Both ride the
/// `from_graveyard` + `exile_self_cost` activation path; the token rides
/// `CreateTokenCopyOf` with the Zombie type, the color and no mana cost.
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
            extra_keywords: vec![],
            who: PlayerRef::You,
            count: Value::Const(1),
            source: Selector::This,
            extra_creature_types: vec![crate::card::CreatureType::Zombie],
            extra_card_types: Vec::new(),
            override_pt,
            override_colors: Some(vec![if override_pt.is_some() { crate::mana::Color::Black } else { crate::mana::Color::White }]),
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
            no_mana_cost: true,
            enters_with_counters: None,
            remove_keywords: vec![],
        },
        ..Default::default()
    }
}

/// Encore (CR 702.141): "[Cost], Exile this card from your graveyard: For
/// each opponent, create a token that's a copy of this card that attacks
/// that opponent this turn if able. The tokens gain haste. Sacrifice them at
/// the beginning of the next end step. Activate only as a sorcery."
pub fn encore(cost: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        from_graveyard: true,
        exile_self_cost: true,
        effect: Effect::EncoreTokens,
        ..Default::default()
    }
}

/// "Target opponent" for an ability whose only use of the slot is a
/// `PlayerRef::Target(0)` ("target opponent gains control of …"): an inert
/// life loss of 0 declares the slot's opponent filter.
pub fn declare_target_opponent() -> Effect {
    Effect::LoseLife { who: target_filtered(SelectionRequirement::OpponentPlayer), amount: Value::Const(0) }
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

/// Craft (CR 702.167): "[cost], Exile this artifact, Exile `count` other
/// [filter] from among permanents you control and/or [filter] cards from your
/// graveyard: Return this card transformed. Activate only as a sorcery." Rides
/// `ExileSelfReturnTransformed` plus `craft_exile_cost` (`count` 0 = "one or
/// more"). The front face must carry a `back_face`.
pub fn craft(cost: crate::mana::ManaCost, filter: SelectionRequirement, count: u32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        craft_exile_cost: Some((filter, count)),
        effect: Effect::ExileSelfReturnTransformed,
        ..Default::default()
    }
}

/// [`craft`] with one distinct object per requirement ("Craft with a
/// Dinosaur, a Merfolk, a Pirate, and a Vampire" — CR 702.167a).
pub fn craft_each(cost: crate::mana::ManaCost, each: Vec<SelectionRequirement>) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        craft_exile_each: each,
        effect: Effect::ExileSelfReturnTransformed,
        ..Default::default()
    }
}

/// [`craft`] "with `count` that share a card type" (CR 702.167a — Eye of Ojer
/// Taq).
pub fn craft_sharing_type(cost: crate::mana::ManaCost, count: u32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost,
        sorcery_speed: true,
        craft_exile_sharing_type: Some(count),
        effect: Effect::ExileSelfReturnTransformed,
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

/// Prowl (CR 702.76): "You may cast this for [cost] if you dealt combat
/// damage to a player this turn with a creature of any of `types`."
/// `marks_kicked` stamps the cast so "if its prowl cost was paid" riders
/// gate on `Predicate::SpellWasKicked`.
pub fn prowl(
    cost: crate::mana::ManaCost,
    types: Vec<crate::card::CreatureType>,
) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost: cost,
        condition: Some(Predicate::ProwlTypeDealtCombatDamage { types }),
        marks_kicked: true,
        ..Default::default()
    }
}

/// Offering (CR 702.48): "[Type] offering" — cast at instant speed by
/// sacrificing a creature of `creature_type`, reducing the cost by the
/// sacrificed creature's whole mana cost (color included).
pub fn offering(
    cost: crate::mana::ManaCost,
    creature_type: crate::card::CreatureType,
) -> crate::card::AlternativeCost {
    crate::card::AlternativeCost {
        mana_cost: cost,
        flash: true,
        offering: Some(crate::card::SelectionRequirement::HasCreatureType(creature_type)),
        ..Default::default()
    }
}

/// Graft N (CR 702.58), the trigger half: "Whenever another creature enters, if
/// this permanent has a +1/+1 counter on it, you may move a +1/+1 counter from
/// this permanent onto that creature." Pair with `enters_with_counters`. The
/// intervening if (CR 603.4) matters at a big table: without it an empty
/// grafter triggered on every entering creature.
pub fn graft() -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
            .with_filter(Predicate::All(vec![
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                },
                Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                },
            ])),
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
/// `Value::OpponentsAttackedThisCombat` counts the distinct defending
/// seats in the current attack declaration (one in 1v1, more in
/// multiplayer).
pub fn melee() -> TriggeredAbility {
    on_attack(Effect::PumpPT {
        what: Selector::This,
        power: Value::OpponentsAttackedThisCombat,
        toughness: Value::OpponentsAttackedThisCombat,
        duration: Duration::EndOfTurn,
    })
}

/// "Recover—Pay half your life, rounded up" (Garza's Assassin): CR 702.59
/// with a life cost instead of mana, and the printed "another creature".
pub fn recover_paying_half_life() -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    use crate::effect::ZoneDest;
    TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::FromYourGraveyard)
            .with_filter(recover_subject_is_yours(true)),
        effect: Effect::MayPayLife {
            description: "Recover: pay half your life, rounded up?".into(),
            amount: Value::HalfLifeRoundedUp(PlayerRef::You),
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Some(Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Exile })),
        },
    }
}

/// The dying creature must be one you own; `another` also excludes the
/// recover card itself.
fn recover_subject_is_yours(another: bool) -> crate::effect::Predicate {
    let mut filter = SelectionRequirement::OwnedByYou;
    if another {
        filter = filter.and(SelectionRequirement::OtherThanSource);
    }
    crate::effect::Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// Landfall (Zendikar ability word): "whenever a land you control enters, …".
/// `EventKind::LandPlayed` fires on any land entry, not only a land *play*.
pub fn landfall(effect: Effect) -> TriggeredAbility {
    use crate::card::{EventKind, EventScope, EventSpec};
    TriggeredAbility {
        event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
        effect,
    }
}

/// Rally (BFZ ability word): "whenever this creature or another Ally you
/// control enters, …".
pub fn rally(effect: Effect) -> TriggeredAbility {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, Predicate};
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasCreatureType(CreatureType::Ally),
            },
        ),
        effect,
    }
}

/// The Rally shape that grants a keyword to your creatures until end of turn
/// (Kor Bladewhirl, Makindi Patrol, Ondu Champion, Lantern Scout, …).
pub fn rally_grant(keyword: Keyword) -> TriggeredAbility {
    rally(Effect::GrantKeyword {
        what: each_your_creature(),
        keyword,
        duration: Duration::EndOfTurn,
    })
}

/// "Create a token that's a copy of `source`" — the plain CR 707.2 shape, no
/// riders. The rider-bearing printings still spell `Effect::CreateTokenCopyOf`
/// out.
pub fn token_copy_of(who: PlayerRef, count: Value, source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
        who,
        count,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    }
}

/// The Dragons of Tarkir cycle's cast condition (CR 601.2): "you may reveal a
/// Dragon card from your hand" as an additional cost, or control a Dragon. The
/// reveal costs nothing, so it is always made when a Dragon is in hand.
pub fn dragon_as_cast() -> Predicate {
    let dragon = || SelectionRequirement::HasCreatureType(crate::card::CreatureType::Dragon);
    Predicate::Any(vec![
        Predicate::SelectorExists(Selector::EachPermanent(dragon().and(SelectionRequirement::ControlledByYou))),
        Predicate::SelectorExists(Selector::CardsInZone {
            who: PlayerRef::You,
            zone: crate::card::Zone::Hand,
            filter: dragon(),
        }),
    ])
}

/// "`chooser` chooses one of `what`, then `body`" — a choice made on
/// resolution (CR 608.2d), not a target; `body` reads the pick as
/// [`chosen_one`] (God-Pharaoh's Gift, The Mending of Dominaria).
pub fn choose_one_then(what: Selector, chooser: PlayerRef, body: Effect) -> Effect {
    Effect::ChooseOneAmong { what, chooser, chosen: Box::new(body), other: Box::new(Effect::Noop) }
}

/// The card(s) or permanent(s) [`choose_one_then`] / [`choose_some_then`] picked.
pub fn chosen_one() -> Selector { Selector::SeparatedPile { chosen: true } }

/// [`choose_one_then`] for `count` picks (`up_to`: any number to `count`);
/// `body` reads them as [`chosen_one`].
pub fn choose_some_then(what: Selector, chooser: PlayerRef, count: Value, up_to: bool, body: Effect) -> Effect {
    Effect::ChooseSomeAmong { what, chooser, count, up_to, chosen: Box::new(body), other: Box::new(Effect::Noop) }
}

/// "Will of the Planeswalkers — Starting with you, each player votes for
/// planeswalk or chaos. If planeswalk gets more votes, planeswalk. If chaos
/// gets more votes or the vote is tied, chaos ensues." (the Path cycle). Outside
/// Planechase the vote is still held and both outcomes do nothing (CR 701.31a).
pub fn will_of_the_planeswalkers() -> Effect {
    Effect::Vote {
        options: vec![
            VoteOption::new("planeswalk", Effect::Planeswalk { who: PlayerRef::You }),
            VoteOption::new("chaos", Effect::ChaosEnsues { who: PlayerRef::You }),
        ],
        tally: VoteTally::Majority,
    }
}

/// "(You may) attach it to a creature you control" after an Equipment was put
/// onto the battlefield (`Selector::LastMoved`): the creature is chosen, not
/// targeted (CR 701.3a); `may` lets the controller choose none.
pub fn attach_moved_equipment_to_your_creature(may: bool) -> Effect {
    use crate::card::{ArtifactSubtype, SelectionRequirement as R};
    Effect::If {
        cond: Predicate::EntityMatches {
            what: Selector::LastMoved,
            filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
        },
        then: Box::new(choose_some_then(
            Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            PlayerRef::You,
            Value::ONE,
            may,
            Effect::Attach { what: Selector::LastMoved, to: chosen_one() },
        )),
        else_: Box::new(Effect::Noop),
    }
}

/// CR 105.4 — exactly two colors: one of the ten pairs and none of the rest.
pub fn exactly_two_colors() -> SelectionRequirement {
    let all = [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green];
    let mut out: Option<SelectionRequirement> = None;
    for i in 0..5 {
        for j in i + 1..5 {
            let mut pair = SelectionRequirement::HasColor(all[i]).and(SelectionRequirement::HasColor(all[j]));
            for (k, c) in all.iter().enumerate() {
                if k != i && k != j {
                    pair = pair.and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasColor(*c))));
                }
            }
            out = Some(match out {
                Some(o) => o.or(pair),
                None => pair,
            });
        }
    }
    out.expect("ten pairs")
}
