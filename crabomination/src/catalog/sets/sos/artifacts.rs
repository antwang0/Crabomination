//! Secrets of Strixhaven (SOS) — Artifacts.
//!
//! Most SOS artifacts are colorless utility pieces that could fit in any
//! deck (Diary of Dreams, Page-Loose Leaf), plus a few college-coloured
//! support pieces (Cauldron of Essence for Witherbloom, Ark of Hunger
//! for Lorehold). The set's Vehicles (Strixhaven Skycoach) live here
//! once the Crew primitive lands.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, Effect, EventKind, EventScope, EventSpec,
    SelectionRequirement, Subtypes, TriggeredAbility,
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
            exile_gy_cost: 0,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Diary of Dreams — {2} Artifact — Book.
/// "Whenever you cast an instant or sorcery spell, put a page counter on
/// this artifact. / {5}, {T}: Draw a card. This ability costs {1} less
/// to activate for each page counter on this artifact."
///
/// Approximation: the page-counter cost reduction on the activated
/// ability is omitted (the engine has no "self-counter scaled cost
/// reduction" primitive on activated abilities). The trigger that adds
/// page counters and the {5}, {T}: Draw a card activation are wired
/// faithfully — at high counter counts the ability still costs {5} but
/// the rest of the card behaves as printed. Tracked under the cost-
/// reduction stacking section in TODO.md.
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
            exile_gy_cost: 0,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tablet of Discovery — {2}{R} Artifact.
/// "When this artifact enters, mill a card. You may play that card this
/// turn. (To mill a card, put the top card of your library into your
/// graveyard.) / {T}: Add {R}. / {T}: Add {R}{R}. Spend this mana only
/// to cast instant and sorcery spells."
///
/// ETB-mill is wired faithfully via `Effect::Mill { count: 1 }`. The
/// "you may play that card this turn" rider is omitted (engine has no
/// per-card "may-play-from-graveyard-until-EOT" primitive — same gap as
/// Suspend Aggression's "may play exiled cards until end of next turn").
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
            exile_gy_cost: 0,
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
            exile_gy_cost: 0,
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Potioner's Trove — {3} Artifact.
/// "{T}: Add one mana of any color. / {T}: You gain 2 life. Activate
/// only if you've cast an instant or sorcery spell this turn."
///
/// Push XXXVII: ✅ — both abilities now fully wired. The mana ability
/// is unconditional. The lifegain ability gates on the new
/// `Predicate::InstantsOrSorceriesCastThisTurnAtLeast` predicate
/// (push XIII), backed by `Player.instants_or_sorceries_cast_this_turn`,
/// so the activation is rejected with `AbilityConditionNotMet` until
/// the controller has cast at least one IS spell that turn.
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
            exile_gy_cost: 0,
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
                exile_gy_cost: 0,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
/// The {T}: Mill activation is wired faithfully; the "you may play that
/// card this turn" rider is omitted (engine has no per-card "may-play-
/// from-graveyard-until-EOT" primitive — same gap as Tablet of Discovery
/// and Suspend Aggression).
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
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}


