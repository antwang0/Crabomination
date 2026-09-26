//! Commander: the cards the **Divine Convocation** precon (MOC, Kasla, the
//! Broken Halo) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kasla.rs`.
//!
//! Residuals (each also on its card):
//! - **Path of the Ghosthunter** — with no planar deck the Will of the
//!   Planeswalkers vote isn't held (its outcome would change nothing).
//! - **Joyful Stormsculptor** — battles take no damage (the engine has none).

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{counter_target_spell, etb, on_attack, target_filtered};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, hybrid, r, u, w, x};
use crate::sets::{enters_tapped, tap_add};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], keywords: vec![Keyword::Convoke], effect, ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, ct: CreatureType, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ct], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn mint(t: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(t) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// "Whenever this becomes tapped, [effect]."
fn on_tapped(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource), effect }
}

/// "Whenever you cast a spell that has convoke, [effect]."
fn on_convoke_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasConvoke }),
        effect,
    }
}

/// Kasla, the Broken Halo — convoke; flying, vigilance, haste; each other
/// convoke spell you cast scries 2 and draws.
pub fn kasla_the_broken_halo() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Convoke, Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![on_convoke_cast(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            draw(1),
        ]))],
        ..creature(
            "Kasla, the Broken Halo",
            cost(&[generic(3), u(), r(), w()]),
            vec![CreatureType::Angel, CreatureType::Ally],
            5,
            4,
        )
    }
}

/// Angel of Salvation — flash, convoke, flying; entering, prevent the next 5
/// damage divided among any number of targets.
pub fn angel_of_salvation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Convoke, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::PreventNextDamageDivided {
            total: Value::Const(5),
            filter: R::Any,
            max_targets: 5,
        })],
        ..creature("Angel of Salvation", cost(&[generic(6), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Artistic Refusal — convoke; counter a spell and/or loot two.
pub fn artistic_refusal() -> CardDefinition {
    spell(
        "Artistic Refusal",
        cost(&[generic(4), u(), u()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                counter_target_spell(),
                Effect::Seq(vec![draw(2), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
            ],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Cut Short — convoke; destroy a tapped creature or a planeswalker activated
/// this turn.
pub fn cut_short() -> CardDefinition {
    spell(
        "Cut Short",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::Destroy {
            what: target_filtered(
                R::Creature.and(R::Tapped).or(R::Planeswalker.and(R::LoyaltyActivatedThisTurn)),
            ),
        },
    )
}

/// Deluxe Dragster — only Vehicles block it; its combat damage lets you cast
/// an instant or sorcery from that player's graveyard free, exiled after.
pub fn deluxe_dragster() -> CardDefinition {
    CardDefinition {
        name: "Deluxe Dragster",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![
            Keyword::Crew(2),
            Keyword::CantBeBlockedExceptBy(Box::new(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CastWithoutPayingImmediate {
                what: target_filtered(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::InGraveyard)
                        .and(R::ControlledByTriggerPlayer),
                ),
                source_zone: crate::card::Zone::Graveyard,
                exile_after: true,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
        }],
        ..Default::default()
    }
}

/// Fallowsage — becoming tapped, you may draw a card.
pub fn fallowsage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_tapped(Effect::MayDo {
            description: "Draw a card?".into(),
            body: Box::new(draw(1)),
        })],
        ..creature("Fallowsage", cost(&[generic(3), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 2)
    }
}

/// Flight of Equenauts — convoke, flying.
pub fn flight_of_equenauts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Flying],
        ..creature(
            "Flight of Equenauts",
            cost(&[generic(7), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            5,
        )
    }
}

/// Flockchaser Phantom — convoke, flying, vigilance; attacking, your next
/// spell this turn has convoke.
pub fn flockchaser_phantom() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke, Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::NextSpellGainsConvokeThisTurn)],
        ..creature("Flockchaser Phantom", cost(&[generic(4), w(), u()]), vec![CreatureType::Spirit], 5, 5)
    }
}

/// Joyful Stormsculptor — two 1/1 Elementals on entry; each convoke spell you
/// cast pings each opponent.
/// Residual: battles take no damage.
pub fn joyful_stormsculptor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(mint(
                token("Elemental", vec![Color::Blue, Color::Red], CreatureType::Elemental, 1, 1, vec![]),
                Value::Const(2),
            )),
            on_convoke_cast(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE }),
        ],
        ..creature(
            "Joyful Stormsculptor",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            2,
            3,
        )
    }
}

/// Meeting of Minds — convoke; draw two.
pub fn meeting_of_minds() -> CardDefinition {
    spell("Meeting of Minds", cost(&[generic(3), u()]), CardType::Instant, draw(2))
}

/// Mistmeadow Vanisher — becoming tapped, flicker up to one nonland,
/// nontoken permanent until the next end step.
pub fn mistmeadow_vanisher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_tapped(Effect::ExileReturnToOwnerNextEndStep {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::NotToken)),
            tapped: false,
        })],
        ..creature(
            "Mistmeadow Vanisher",
            cost(&[generic(2), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Kithkin, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Nesting Dovehawk — flying; populates at the beginning of your combat; your
/// creature tokens entering grow it.
pub fn nesting_dovehawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Populate { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsToken) },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..creature("Nesting Dovehawk", cost(&[generic(3), w()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Path of the Ghosthunter — X 1/1 flying Spirits.
/// Residual: with no planar deck the Will of the Planeswalkers vote isn't
/// held (planeswalk and chaos do nothing without one).
pub fn path_of_the_ghosthunter() -> CardDefinition {
    CardDefinition {
        name: "Path of the Ghosthunter",
        cost: cost(&[x(), generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: mint(token("Spirit", vec![Color::White], CreatureType::Spirit, 1, 1, vec![Keyword::Flying]), Value::XFromCost),
        ..Default::default()
    }
}

/// Saint Traft and Rem Karolus — becoming tapped makes a Human, then a
/// Spirit, then an Angel each turn; a convoke spell you cast untaps it.
pub fn saint_traft_and_rem_karolus() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            on_tapped(Effect::NthResolutionThisTurn {
                branches: vec![
                    mint(token("Human", vec![Color::Red], CreatureType::Human, 1, 1, vec![]), Value::ONE),
                    mint(token("Spirit", vec![Color::Blue], CreatureType::Spirit, 1, 1, vec![Keyword::Flying]), Value::ONE),
                    mint(token("Angel", vec![Color::White], CreatureType::Angel, 4, 4, vec![Keyword::Flying]), Value::ONE),
                ],
            }),
            on_convoke_cast(Effect::Untap { what: Selector::This, up_to: None }),
        ],
        ..creature(
            "Saint Traft and Rem Karolus",
            cost(&[u(), r(), w()]),
            vec![CreatureType::Spirit, CreatureType::Human],
            3,
            4,
        )
    }
}

/// Shatter the Source — convoke; 6 damage to a creature or planeswalker, or
/// destroy an artifact.
pub fn shatter_the_source() -> CardDefinition {
    spell(
        "Shatter the Source",
        cost(&[generic(5), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::Planeswalker)),
                amount: Value::Const(6),
            },
            Effect::Destroy { what: target_filtered(R::Artifact) },
        ]),
    )
}

/// Temporal Cleansing — convoke; a nonland permanent's owner puts it second
/// from the top or on the bottom.
pub fn temporal_cleansing() -> CardDefinition {
    spell(
        "Temporal Cleansing",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::SecondFromTopOrBottom,
            },
        },
    )
}

/// Venerated Loxodon — convoke; a +1/+1 counter on each creature that
/// convoked it.
pub fn venerated_loxodon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::CreaturesThatConvokedSource,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Venerated Loxodon",
            cost(&[generic(4), w()]),
            vec![CreatureType::Elephant, CreatureType::Cleric],
            4,
            4,
        )
    }
}

/// Wand of the Worldsoul — enters tapped; {T}: {W}; {T}: your next spell this
/// turn has convoke.
pub fn wand_of_the_worldsoul() -> CardDefinition {
    CardDefinition {
        name: "Wand of the Worldsoul",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::White),
            crate::card::ActivatedAbility {
                tap_cost: true,
                effect: Effect::NextSpellGainsConvokeThisTurn,
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wildfire Awakener — convoke; X 1/1 Elementals that ping a player whenever
/// they become tapped.
pub fn wildfire_awakener() -> CardDefinition {
    let mut elemental = token("Elemental", vec![Color::Red], CreatureType::Elemental, 1, 1, vec![]);
    elemental.triggered_abilities = vec![on_tapped(Effect::DealDamage {
        to: target_filtered(R::Player),
        amount: Value::ONE,
    })];
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(mint(elemental, Value::XFromCost))],
        ..creature(
            "Wildfire Awakener",
            cost(&[x(), generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}
