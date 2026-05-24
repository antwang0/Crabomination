//! Secrets of Strixhaven (SOS) — Artifacts.
//!
//! Most SOS artifacts are colorless utility pieces that could fit in any
//! deck (Diary of Dreams, Page-Loose Leaf), plus a few college-coloured
//! support pieces (Cauldron of Essence for Witherbloom, Ark of Hunger
//! for Lorehold). The set's Vehicles (Strixhaven Skycoach) live here
//! once the Crew primitive lands.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Subtypes, TriggeredAbility,
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
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
/// Both mana abilities are wired as plain `{T}: Add {R}` adders — the
/// "spend only on instant/sorcery" restriction on the {T}: Add {R}{R}
/// ability is omitted (engine has no spend-restricted mana primitive).
/// In practice the produced mana flows through the same pool as the
/// {R}-only ability, slightly more flexible than the printed card.
pub fn tablet_of_discovery() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::mana::r;
    CardDefinition {
        name: "Tablet of Discovery",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
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
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red, Color::Red]),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
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
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
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
            },
            ActivatedAbility {
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
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Resonating Lute — {2}{U}{R} Prismari Artifact.
/// "Lands you control have '{T}: Add two mana of any one color. Spend
/// this mana only to cast instant and sorcery spells.' / {T}: Draw a
/// card. Activate only if you have seven or more cards in your hand."
///
/// The `{T}: Draw a card` activation is wired with the new
/// `condition: Predicate::ValueAtLeast(HandSizeOf(You), 7)` gate — the
/// engine rejects the activation before paying the tap cost when hand
/// size < 7. The lands-grant static is omitted (engine has no per-color
/// "spend this mana only to cast X" restriction yet — tracked under
/// "Spend-Restricted Mana" in TODO.md). Without the lands-grant the
/// card is a 4-mana "draw a card if your hand is overflowing" — a
/// strictly weaker version, but the printed gate enforces a meaningful
/// hand-size threshold.
pub fn resonating_lute() -> CardDefinition {
    use crate::card::Predicate;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Resonating Lute",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Strixhaven Skycoach ─────────────────────────────────────────────────────

/// Strixhaven Skycoach — {3} Artifact — Vehicle, 3/2.
/// "Flying / When this Vehicle enters, you may search your library for
/// a basic land card, reveal it, put it into your hand, then shuffle. /
/// Crew 2."
///
/// 🟡 Body wired: 3/2 Vehicle artifact (subtype tag) with Flying. The
/// ETB basic-land tutor-to-hand is wired faithfully via `Effect::Search
/// { filter: IsBasicLand, to: Hand(You) }`. Crew is not enforced — the
/// engine has no crew-as-tap-cost primitive (TODO.md), so the Skycoach
/// stays a non-creature artifact until that lands. Body resolves
/// end-to-end for the ETB tutor, which is the most impactful clause.
pub fn strixhaven_skycoach() -> CardDefinition {
    use crate::card::{CreatureType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::{PlayerRef as PR, ZoneDest as ZD};
    // Push (modern_decks, batch 88): Crew approximated as "Skycoach is
    // always a creature." Printed Vehicle subtype kept for catalog
    // filtering; `CardType::Creature` added so the 3/2 Flying body can
    // attack and block. Stronger than printed (which requires tapping
    // creatures with total power ≥ 2 to crew the Vehicle into a
    // creature until EOT), but functional and captures the printed
    // play pattern. The full Crew mechanic would need a tap-creatures-
    // as-cost activation that transiently flips `CardType::Creature`
    // on — same engine gap as Crew across the catalog.
    CardDefinition {
        name: "Strixhaven Skycoach",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

