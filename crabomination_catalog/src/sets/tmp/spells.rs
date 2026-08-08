//! Tempest (TMP) instants and sorceries — the buyback cycle and the block's
//! removal. Tests in `classic_sets/tmp`.

use crate::card::{CardDefinition, CardType, Keyword, SelectionRequirement as R};
use crate::effect::shortcut::{draw, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn with_buyback(mut def: CardDefinition, c: ManaCost) -> CardDefinition {
    def.keywords.push(Keyword::Buyback(c));
    def
}

// ── Buyback cycle ───────────────────────────────────────────────────────────

/// Anoint — {W} instant, buyback {3}. Prevent the next 3 damage to a creature.
pub fn anoint() -> CardDefinition {
    with_buyback(
        instant(
            "Anoint",
            cost(&[w()]),
            Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::Const(3),
            },
        ),
        cost(&[generic(3)]),
    )
}

/// Elvish Fury — {G} instant, buyback {4}. A +2/+2 pump.
pub fn elvish_fury() -> CardDefinition {
    with_buyback(
        instant("Elvish Fury", cost(&[g()]), crate::effect::shortcut::pump_target(2, 2)),
        cost(&[generic(4)]),
    )
}

/// Disturbed Burial — {1}{B} sorcery, buyback {3}. Recur a creature card.
pub fn disturbed_burial() -> CardDefinition {
    with_buyback(
        sorcery(
            "Disturbed Burial",
            cost(&[generic(1), b()]),
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ),
        cost(&[generic(3)]),
    )
}

/// Evincar's Justice — {2}{B}{B} sorcery, buyback {3}. A repeatable sweeper.
pub fn evincars_justice() -> CardDefinition {
    with_buyback(
        sorcery(
            "Evincar's Justice",
            cost(&[generic(2), b(), b()]),
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: crate::effect::shortcut::each_creature(),
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(2),
                },
            ]),
        ),
        cost(&[generic(3)]),
    )
}

/// Imps' Taunt — {1}{B} instant, buyback {3}. Drag a creature into combat.
pub fn imps_taunt() -> CardDefinition {
    with_buyback(
        instant(
            "Imps' Taunt",
            cost(&[generic(1), b()]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
        ),
        cost(&[generic(3)]),
    )
}

// ── Removal and burn ────────────────────────────────────────────────────────

/// Aftershock — {2}{R}{R} sorcery. Blow up anything; take 3 for it.
pub fn aftershock() -> CardDefinition {
    sorcery(
        "Aftershock",
        cost(&[generic(2), r(), r()]),
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            Effect::DealDamage { to: Selector::Player(PlayerRef::You), amount: Value::Const(3) },
        ]),
    )
}

/// Perish — {2}{B} sorcery. Green creatures die and stay dead.
pub fn perish() -> CardDefinition {
    sorcery(
        "Perish",
        cost(&[generic(2), b()]),
        Effect::DestroyNoRegen {
            what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Green))),
        },
    )
}

/// Extinction — {4}{B} sorcery. Name a creature type; it's gone.
pub fn extinction() -> CardDefinition {
    sorcery(
        "Extinction",
        cost(&[generic(4), b()]),
        Effect::Seq(vec![
            Effect::NameCreatureType { what: Selector::This },
            Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsSourceChosenCreatureType),
                ),
            },
        ]),
    )
}

/// Dregs of Sorrow — {X}{4}{B} sorcery. X nonblack creatures for X cards.
pub fn dregs_of_sorrow() -> CardDefinition {
    sorcery(
        "Dregs of Sorrow",
        cost(&[x(), generic(4), b()]),
        Effect::Seq(vec![
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
            Effect::Draw { who: Selector::You, amount: Value::XFromCost },
        ]),
    )
}

/// Repentance — {2}{W} sorcery. A creature beats itself up.
pub fn repentance() -> CardDefinition {
    sorcery(
        "Repentance",
        cost(&[generic(2), w()]),
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        },
    )
}

/// Kindle — {1}{R} instant. Two damage, plus one per Kindle already burnt.
pub fn kindle() -> CardDefinition {
    instant(
        "Kindle",
        cost(&[generic(1), r()]),
        Effect::DealDamage {
            to: target_any(),
            amount: Value::Sum(vec![
                Value::Const(2),
                Value::CardsNamedLikeSourceInAllGraveyards,
            ]),
        },
    )
}

/// Blood Frenzy — {1}{R} instant. A lethal pump for a creature in combat.
pub fn blood_frenzy() -> CardDefinition {
    instant(
        "Blood Frenzy",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::IsAttacking.or(R::IsBlocking)),
                },
                power: Value::Const(4),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::AtNextEndStep {
                body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            },
        ]),
    )
}

/// Apocalypse — {2}{R}{R}{R} sorcery. Everything goes, your hand included.
pub fn apocalypse() -> CardDefinition {
    sorcery(
        "Apocalypse",
        cost(&[generic(2), r(), r(), r()]),
        Effect::Seq(vec![
            Effect::Exile { what: Selector::EachPermanent(R::Any) },
            Effect::Discard {
                who: Selector::You,
                amount: Value::HandSizeOf(PlayerRef::You),
                random: false,
            },
        ]),
    )
}

// ── Card advantage and utility ──────────────────────────────────────────────

/// Dismiss — {2}{U}{U} instant. Counter, then draw.
pub fn dismiss() -> CardDefinition {
    instant(
        "Dismiss",
        cost(&[generic(2), u(), u()]),
        Effect::Seq(vec![crate::effect::shortcut::counter_target_spell(), draw(1)]),
    )
}

/// Gallantry — {1}{W} instant. A blocker gets huge and you draw.
pub fn gallantry() -> CardDefinition {
    instant(
        "Gallantry",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::IsBlocking),
                },
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Meditate — {2}{U} instant. Four cards for a turn.
pub fn meditate() -> CardDefinition {
    instant(
        "Meditate",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            draw(4),
            Effect::SkipTurns { who: PlayerRef::You, count: Value::ONE },
        ]),
    )
}

/// Reality Anchor — {1}{G} instant. Strip shadow, then cantrip.
pub fn reality_anchor() -> CardDefinition {
    instant(
        "Reality Anchor",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::LoseKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Shadow,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Respite — {1}{G} instant. A fog that pays you for the attack.
pub fn respite() -> CardDefinition {
    instant(
        "Respite",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::PreventAllCombatDamageThisTurn,
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(R::Creature.and(R::IsAttacking))),
            },
        ]),
    )
}

/// Mana Severance — {1}{U} sorcery. Thin every land out of the library.
pub fn mana_severance() -> CardDefinition {
    sorcery(
        "Mana Severance",
        cost(&[generic(1), u()]),
        Effect::SearchAnyNumber { who: PlayerRef::You, filter: R::Land, to: ZoneDest::Exile },
    )
}

/// Legerdemain — {2}{U}{U} sorcery. Swap an artifact or creature for a
/// permanent that shares one of those types.
pub fn legerdemain() -> CardDefinition {
    sorcery(
        "Legerdemain",
        cost(&[generic(2), u(), u()]),
        Effect::ExchangeControl {
            a: Selector::TargetFiltered { slot: 0, filter: R::Artifact.or(R::Creature) },
            b: Selector::TargetFiltered { slot: 1, filter: R::Artifact.or(R::Creature) },
        },
    )
}

/// Intuition — {2}{U} instant. Fetch three, an opponent picks which you keep.
pub fn intuition() -> CardDefinition {
    instant(
        "Intuition",
        cost(&[generic(2), u()]),
        Effect::SearchSplitOpponentChooses {
            opponent: Selector::Player(PlayerRef::Target(0)),
            count: 3,
            opponent_picks: 1,
            chosen_to: ZoneDest::Hand(PlayerRef::You),
            rest_to: ZoneDest::Graveyard,
        },
    )
}

/// Invulnerability — {1}{W} instant, buyback {3}. Shield yourself from the
/// next hit off a source of your choice.
pub fn invulnerability() -> CardDefinition {
    with_buyback(
        instant(
            "Invulnerability",
            cost(&[generic(1), w()]),
            Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
                exile_top_per_prevented: false,
            },
        ),
        cost(&[generic(3)]),
    )
}

// ── The rest of the buyback cycle ───────────────────────────────────────────

/// Searing Touch — {R} instant, buyback {4}. A recurring ping.
pub fn searing_touch() -> CardDefinition {
    with_buyback(
        instant("Searing Touch", cost(&[r()]), crate::effect::shortcut::deal(1, target_any())),
        cost(&[generic(4)]),
    )
}

/// Whispers of the Muse — {U} instant, buyback {5}. A cantrip forever.
pub fn whispers_of_the_muse() -> CardDefinition {
    with_buyback(instant("Whispers of the Muse", cost(&[u()]), draw(1)), cost(&[generic(5)]))
}

/// Worthy Cause — {W} instant, buyback {2}. Trade a creature for its toughness
/// in life.
pub fn worthy_cause() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..with_buyback(
            instant(
                "Worthy Cause",
                cost(&[w()]),
                Effect::GainLife { who: Selector::You, amount: Value::SacrificedToughness },
            ),
            cost(&[generic(2)]),
        )
    }
}

// ── Shadow-matters tricks ───────────────────────────────────────────────────

/// Shadow Rift — {U} instant. Slip a creature into shadow, then cantrip.
pub fn shadow_rift() -> CardDefinition {
    instant(
        "Shadow Rift",
        cost(&[u()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Shadow,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Shadowstorm — {R} sorcery. Two damage to every shadow creature.
pub fn shadowstorm() -> CardDefinition {
    sorcery(
        "Shadowstorm",
        cost(&[r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Shadow))),
            amount: Value::Const(2),
        },
    )
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Stun — {1}{R} instant. Take a blocker off the table, then cantrip.
pub fn stun() -> CardDefinition {
    instant(
        "Stun",
        cost(&[generic(1), r()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Twitch — {2}{U} instant. Tap or untap anything, then cantrip.
pub fn twitch() -> CardDefinition {
    instant(
        "Twitch",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            Effect::TapOrUntap {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            draw(1),
        ]),
    )
}

/// Verdigris — {2}{G} instant. Green artifact removal.
pub fn verdigris() -> CardDefinition {
    instant(
        "Verdigris",
        cost(&[generic(2), g()]),
        Effect::Destroy { what: target_filtered(R::Artifact) },
    )
}

/// Serene Offering — {1}{W} instant. Kill an enchantment, gain its mana value.
pub fn serene_offering() -> CardDefinition {
    instant(
        "Serene Offering",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
        ]),
    )
}

/// Spontaneous Combustion — {1}{B}{R} instant. Eat a creature to sweep three
/// damage across the board.
pub fn spontaneous_combustion() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..instant(
            "Spontaneous Combustion",
            cost(&[generic(1), b(), r()]),
            Effect::DealDamage {
                to: crate::effect::shortcut::each_creature(),
                amount: Value::Const(3),
            },
        )
    }
}

/// Deadshot — {3}{R} sorcery. Tap a creature and fire its power at another.
pub fn deadshot() -> CardDefinition {
    sorcery(
        "Deadshot",
        cost(&[generic(3), r()]),
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
    )
}

/// Scorched Earth — {X}{R} sorcery. Discard X lands to blow up X of theirs.
pub fn scorched_earth() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::DiscardXFromCost],
        ..sorcery(
            "Scorched Earth",
            cost(&[x(), r()]),
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Land,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
        )
    }
}

/// Lobotomy — {2}{U}{B} sorcery. Name a card off their hand and strip every
/// copy from all their zones.
pub fn lobotomy() -> CardDefinition {
    sorcery(
        "Lobotomy",
        cost(&[generic(2), u(), b()]),
        Effect::NameCardExileMatchingAllZones,
    )
}
