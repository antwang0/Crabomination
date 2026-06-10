//! Secrets of Strixhaven (SOS) — Artifacts.
//!
//! Most SOS artifacts are colorless utility pieces that could fit in any
//! deck (Diary of Dreams, Page-Loose Leaf), plus a few college-coloured
//! support pieces (Cauldron of Essence for Witherbloom, Ark of Hunger
//! for Lorehold). The set's Vehicles (Strixhaven Skycoach) live here
//! once the Crew primitive lands.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, Effect, EventKind, EventScope,
    EventSpec, SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic};

use crate::mana::g;

/// Cauldron of Essence — {1}{B}{G} Artifact.
/// "Whenever a creature you control dies, each opponent loses 1 life
/// and you gain 1 life. / {1}{B}{G}, {T}, Sacrifice a creature: Return
/// target creature card from your graveyard to the battlefield. Activate
/// only as a sorcery."
///
/// Both halves are wired with existing primitives:
/// - Death drain trigger: `EventKind::CreatureDied` + `EventScope::AnotherOfYours`
///   gates on "another creature you control dies" (excludes the Cauldron
///   itself, an artifact). The body is a `Drain` from `EachOpponent` to
///   `You`.
/// - Activated reanimation: `tap_cost`, `sac_cost` (engine sacrifices a
///   creature on the controller's behalf via `AutoDecider`), `sorcery_speed`,
///   plus a {1}{B}{G} mana cost. The body picks a creature card from the
///   controller's graveyard and moves it to the battlefield.
pub fn cauldron_of_essence() -> CardDefinition {
    CardDefinition {
        name: "Cauldron of Essence",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Diary of Dreams — {2} Artifact — Book.
/// "Whenever you cast an instant or sorcery spell, put a page counter on
/// this artifact. / {5}, {T}: Draw a card. This ability costs {1} less
/// to activate for each page counter on this artifact."
///
/// Push (modern_decks): self-counter cost reduction **now wired** via
/// `ActivatedAbility.self_counter_cost_reduction: Some(Page)`. The
/// activation's generic mana cost reduces by 1 per Page counter on the
/// source (clamped at the printed {5} generic total), so at 5+ page
/// counters the ability is `{0}, {T}: Draw a card`. Page counters
/// accrue 1 per instant/sorcery cast as before.
pub fn diary_of_dreams() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Diary of Dreams",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: Some(CounterType::Page),
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Page,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Tablet of Discovery — {2}{R} Artifact.
/// "When this artifact enters, mill a card. You may play that card this
/// turn. (To mill a card, put the top card of your library into your
/// graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only
/// to cast instant and sorcery spells."
///
/// Push (modern_decks): the "you may play that card this turn" rider is
/// **now wired** via `Effect::GrantMayPlay` + `Selector::LastMoved`
/// (which reads the freshly-milled card's id from the resolution-scoped
/// scratch). The controller invokes
/// `GameAction::CastFromZoneWithoutPaying` during a later sorcery-speed
/// window to recur the milled card for free.
///
/// Both mana abilities are wired: `{T}: Add {R}` is a plain adder, and
/// `{T}: Add {R}{R}` produces spend-restricted mana via
/// `ManaPayload::Restricted(.., InstantSorceryOnly)`, so that {R}{R} can
/// only fund instant and sorcery spells (enforced by
/// `ManaPool::pay_for_spell`).
pub fn tablet_of_discovery() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::mana::{r, SpendRestriction};
    CardDefinition {
        name: "Tablet of Discovery",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red]),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Red, Color::Red])),
                        SpendRestriction::InstantSorceryOnly,
                    ),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Potioner's Trove — {3} Artifact.
/// "{T}: Add one mana of any color. / {T}: You gain 2 life. Activate
/// only if you've cast an instant or sorcery spell this turn."
///
/// Approximation: the conditional "activate only if you've cast an
/// instant or sorcery spell this turn" gate is omitted (the engine
/// has no per-turn-cast-tracking gate on activated abilities yet). The
/// mana ability is fully wired, and the lifegain ability is unconditional
/// in practice. Tracked under TODO.md "Activated-Ability Per-Turn-Cast
/// Gate".
pub fn potioners_trove() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Potioner's Trove",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                // "Activate only if you've cast an instant or sorcery
                // spell this turn." — wired exactly via the new
                // `Predicate::InstantsOrSorceriesCastThisTurnAtLeast`
                // (push XIII), backed by the new
                // `Player.instants_or_sorceries_cast_this_turn` tally.
                condition: Some(crate::card::Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                }),
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
        ],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Resonating Lute — {2}{U}{R} Prismari Artifact.
/// "Lands you control have '{T}: Add two mana of any one color. Spend
/// this mana only to cast instant and sorcery spells.' / {T}: Draw a
/// card. Activate only if you have seven or more cards in your hand."
///
/// The `{T}: Draw a card` activation is gated by
/// `condition: Predicate::ValueAtLeast(HandSizeOf(You), 7)`. The lands-grant
/// is wired via `grant_tap_for_any_color_restricted` — each land gains
/// `{T}: Add two mana of any one color`, tagged
/// `SpendRestriction::InstantSorceryOnly`, so that mana only funds instant
/// and sorcery spells (enforced by `ManaPool::pay_for_spell`).
pub fn resonating_lute() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::{r, u, SpendRestriction};
    CardDefinition {
        name: "Resonating Lute",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: Some(Predicate::ValueAtLeast(
                Value::HandSizeOf(PlayerRef::You),
                Value::Const(7),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        // "Lands you control have '{T}: Add two mana of any one color.
        // Spend this mana only to cast instant and sorcery spells.'"
        static_abilities: vec![crate::effect::shortcut::grant_tap_for_any_color_restricted(
            crate::card::SelectionRequirement::Land
                .and(crate::card::SelectionRequirement::ControlledByYou),
            2,
            SpendRestriction::InstantSorceryOnly,
        )],
        ..Default::default()
    }
}

/// Ark of Hunger — {2}{R}{W} Lorehold Artifact.
/// "Whenever one or more cards leave your graveyard, this artifact deals
/// 1 damage to each opponent and you gain 1 life. / {T}: Mill a card.
/// You may play that card this turn."
///
/// Wired against the new `EventKind::CardLeftGraveyard` event — every
/// gy-leave drains 1 (and the printed "one or more" wording is naturally
/// per-card emission, same approximation as Hardened Academic / Spirit
/// Mascot / Garrison Excavator).
///
/// Push (modern_decks): the {T}: Mill's "you may play that card this
/// turn" rider is **now wired** via `Effect::GrantMayPlay` +
/// `Selector::LastMoved`. After the mill, the controller can free-cast
/// the milled card during a later sorcery-speed window via
/// `GameAction::CastFromZoneWithoutPaying`.
pub fn ark_of_hunger() -> CardDefinition {
    use crate::mana::{r, w};
    CardDefinition {
        name: "Ark of Hunger",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Strixhaven Skycoach ─────────────────────────────────────────────────────

/// Strixhaven Skycoach — {3} Artifact — Vehicle, 3/2.
/// "Flying / When this Vehicle enters, you may search your library for
/// a basic land card, reveal it, put it into your hand, then shuffle. /
/// Crew 2."
///
/// ✅ Body wired: 3/2 Vehicle artifact with Flying and Crew 2 (CR 702.122,
/// `Keyword::Crew(2)` + `GameAction::Crew`). The ETB basic-land tutor-to-
/// hand is wired faithfully via `Effect::Search { filter: IsBasicLand, to:
/// Hand(You) }`. The Vehicle stays a non-creature artifact until crewed,
/// then animates to a 3/2 flier for the turn.
pub fn strixhaven_skycoach() -> CardDefinition {
    use crate::card::{ArtifactSubtype, CreatureType, EventKind, EventScope, EventSpec, Keyword, TriggeredAbility};
    use crate::effect::{PlayerRef as PR, ZoneDest as ZD};
    CardDefinition {
        name: "Strixhaven Skycoach",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        effect: Effect::Noop,
        activated_abilities: vec![],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PR::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZD::Hand(PR::You),
            },
        }],
        ..Default::default()
    }
}


