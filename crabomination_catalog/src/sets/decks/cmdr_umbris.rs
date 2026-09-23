//! Commander batch — Umbris, Fear Manifest and the cards EDHREC lists as
//! commonly played with it (a Dimir Horror/Nightmare exile-and-mill deck) that
//! the catalog lacked. Tests in `tests/recent_b/cmdr_umbris.rs`.
//!
//! New primitives this batch added (all general):
//! - `CounterType::Slime` (Sludge Monster, Toxrill).
//! - `StaticEffect::MatchingLoseAllAbilities` — the filtered sibling of
//!   `CreaturesLoseAllAbilities` (Sludge Monster).
//! - `Value::GraveyardsWithAtLeast` (The Master of Lake-town) and
//!   `Value::GreatestPowerAmongCards` (Szat's Will).
//! - `SelectionRequirement::IsCommander` (Falthis, Shadowcat Familiar).
//! - `StaticEffect::NotCreatureUnless` — "isn't a creature unless [condition]"
//!   (Arvinox, the Mind Flail).
//! - `Effect::RemoveAllPlayerCounters` (Final Act).
//! - `Effect::GrantMayPlayForLife` + `MayPlayPermission::pay_life` — "pay life
//!   equal to its mana value rather than pay its mana cost" (Inside
//!   Information).
//!
//! - `StaticEffect::ControlOpponentsSearches` — Opposition Agent's search
//!   hijack (`GameState::search_hijacker` + `exile_found_for_hijacker`).
//!
//! Residuals (approximations, each also on the card's doc comment):
//! - "Put there from their library this turn" (The Weaver King, Captain
//!   N'ghathrod) counts mills; a surveil isn't recorded.
//! - Aboleth Spawn's Probing Telepathy puts the copy on the stack with the
//!   original (no Aboleth trigger of its own to respond to), and its "you may"
//!   is asked as the copy resolves rather than as it is made.
//! - Grell Philosopher: only Grell itself (a Horror) gains the artifact's
//!   activated abilities, not every Horror you control; the blue-mana-as-any-
//!   colour rider is omitted.
//! - Defiler of Flesh's optional "pay 2 life, {B} less" additional cost is
//!   omitted (the cast trigger ships).
//! - Psionic Ritual's "Replicate — tap an untapped Horror" is omitted (the
//!   engine's replicate is mana-only).
//! - Arvinox exiles face up (the look/face-down part is informational only).
//! - Panharmonicon doubles ETB-caused triggers from any permanent entering,
//!   not just artifacts and creatures.
//! - Toxrill's -1/-1 per slime counter is stacked per counter up to 15.
//! - Opponent-chosen picks the engine auto-resolves (Yarok's Fenlurker exiles
//!   by hand order, Blot Out / Szat's Will break ties by board order, Braids'
//!   "shares a card type" is the type you picked).
//! - Stone of Erech exiles opponents' dying *nontoken* creatures (the engine's
//!   Valentin replacement); a dying token still dies.
//! - Opposition Agent hijacks the general search resolvers (`Search` and
//!   everything built on it — tutors, fetch lands, `SearchUpToN`,
//!   `SearchZones`, `SearchPickedBy` — plus `SearchAnyNumber` and
//!   `SearchEachBasicLandType`); ~15 bespoke search effects (Signal the Clans,
//!   Transmute Artifact, the opponent-splits-the-pile searches, ...) still
//!   resolve un-hijacked. Only cards found in the *library* are exiled — a
//!   multi-zone search's graveyard/hand find goes where it was headed.
//! - Mind Flayer's control lasts while it remains on the battlefield (not
//!   "while you control it"); Elder Brain's trigger also fires attacking a
//!   planeswalker (reading its controller).

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EnterMode, Keyword, LandType, MayPlayDuration, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, RevealMissDest,
    Selector, StaticEffect, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, u, x};

// ── helpers ──────────────────────────────────────────────────────────────────

fn creature(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    power: i32,
    toughness: i32,
    types: Vec<CreatureType>,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, power, toughness, types, keywords)
    }
}

fn spell(name: &'static str, mana: ManaCost, sorcery: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if sorcery { CardType::Sorcery } else { CardType::Instant }],
        effect,
        ..Default::default()
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn mint(def: TokenDefinition, n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: std::sync::Arc::new(def),
    }
}

fn black_horror() -> TokenDefinition {
    token("Horror", 1, 1, vec![Color::Black], vec![CreatureType::Horror])
}

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

fn each_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::AnyPlayer)
}

fn trigger_matches(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn your_creature() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn artifact_or_creature() -> R {
    R::Artifact.or(R::Creature)
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// "Choose one. If you control a commander as you cast this spell, you may
/// choose both instead." — the Akroma's Will / Jeska's Will shape (the
/// commander check reads at resolution).
fn commander_both(modes: impl Fn() -> Vec<Effect>) -> Effect {
    Effect::If {
        cond: Predicate::YouControlACommander,
        then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
        else_: Box::new(Effect::ChooseMode(modes())),
    }
}

/// "Exile [a card] and you may play it for as long as it remains exiled, and
/// you may spend mana as though it were mana of any color to cast it."
fn may_play_any_color(what: Selector) -> Effect {
    Effect::GrantMayPlay {
        what,
        duration: MayPlayDuration::WhileExiled,
        to_owner: false,
        exile_after: false,
        pay_own_cost: true,
        any_color: true,
    }
}

// ── the commander ────────────────────────────────────────────────────────────

/// Umbris, Fear Manifest — {3}{U}{B} Legendary Creature — Nightmare Horror 1/1.
/// "Umbris gets +1/+1 for each card your opponents own in exile. Whenever
/// Umbris or another Nightmare or Horror you control enters, target opponent
/// exiles cards from the top of their library until they exile a land card."
pub fn umbris_fear_manifest() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Umbris gets +1/+1 for each card your opponents own in exile.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CardsInExileOwnedBy(PlayerRef::EachOpponent),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                trigger_matches(
                    R::HasCreatureType(CreatureType::Nightmare)
                        .or(R::HasCreatureType(CreatureType::Horror)),
                ),
            ),
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::Target(0),
                    find: R::Land,
                    to: ZoneDest::Exile,
                    cap: Value::Const(500),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::Exile,
                }),
            },
        }],
        ..legend(
            "Umbris, Fear Manifest",
            cost(&[generic(3), u(), b()]),
            1,
            1,
            vec![CreatureType::Nightmare, CreatureType::Horror],
            vec![],
        )
    }
}

// ── legendary creatures ──────────────────────────────────────────────────────

/// Gollum, Riddle Master — {1}{B} Legendary Creature — Halfling Horror 3/1.
/// "As Gollum enters, choose odd or even. Whenever an opponent casts a spell
/// with mana value of the chosen quality, choose one that hasn't been chosen —
/// +1/+1 counter on Gollum; each opponent loses 2 life and you gain 2 life;
/// draw a card." The odd/even pick is an as-enters mode choice (`enter_modes`).
pub fn gollum_riddle_master() -> CardDefinition {
    let riddle = |odd: bool| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
            .with_filter(Predicate::CastSpellMatches(R::ManaValueParity { odd })),
        effect: Effect::ChooseUnchosenMode {
            modes: vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ],
        },
    };
    CardDefinition {
        enter_modes: Some(vec![
            EnterMode { label: "Odd", triggered_abilities: vec![riddle(true)], ..Default::default() },
            EnterMode { label: "Even", triggered_abilities: vec![riddle(false)], ..Default::default() },
        ]),
        ..legend(
            "Gollum, Riddle Master",
            cost(&[generic(1), b()]),
            3,
            1,
            vec![CreatureType::Halfling, CreatureType::Horror],
            vec![],
        )
    }
}

/// Gollum the Abandoned — {1}{B} Legendary Creature — Halfling Horror 2/2.
/// "Gollum can't block. When Gollum enters, exile up to one target card from
/// an opponent's graveyard. Each opponent loses 2 life. {2}, Sacrifice an
/// artifact or creature: Return this card from your graveyard to your hand.
/// Activate only as a sorcery."
pub fn gollum_the_abandoned() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::InOpponentGraveyard) },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            from_graveyard: true,
            sorcery_speed: true,
            sac_other_filter: Some((artifact_or_creature().and(R::ControlledByYou), 1)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..legend(
            "Gollum the Abandoned",
            cost(&[generic(1), b()]),
            2,
            2,
            vec![CreatureType::Halfling, CreatureType::Horror],
            vec![Keyword::CantBlock],
        )
    }
}

/// The Weaver King — {4}{U}{B} Legendary Creature — Horror Wizard 3/6. Shadow.
/// "Whenever The Weaver King deals combat damage to a player, they mill that
/// many cards. Then for each opponent, put a creature card from that player's
/// graveyard that was put there from their library this turn onto the
/// battlefield under your control." "Put there from their library this turn"
/// is `PutIntoGraveyardFromLibraryThisTurn`, recorded as the mill happens.
pub fn the_weaver_king() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::TriggerEventAmount,
                },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardsInZone {
                                who: PlayerRef::Triggerer,
                                zone: crate::card::Zone::Graveyard,
                                filter: R::Creature.and(R::PutIntoGraveyardFromLibraryThisTurn),
                            }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                },
            ]),
        }],
        ..legend(
            "The Weaver King",
            cost(&[generic(4), u(), b()]),
            3,
            6,
            vec![CreatureType::Horror, CreatureType::Wizard],
            vec![Keyword::Shadow],
        )
    }
}

/// The Master of Lake-town — {1}{B}{B} Legendary Creature — Human Advisor 3/2.
/// Deathtouch. "Whenever a player loses life, that player mills that many
/// cards. When The Master of Lake-town dies, draw a card for each graveyard
/// with seven or more cards in it."
pub fn the_master_of_lake_town() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeLost, EventScope::AnyPlayer),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::TriggerEventAmount,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::GraveyardsWithAtLeast(7),
                },
            },
        ],
        ..legend(
            "The Master of Lake-town",
            cost(&[generic(1), b(), b()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Advisor],
            vec![Keyword::Deathtouch],
        )
    }
}

/// Captain N'ghathrod — {3}{U}{B} Legendary Creature — Horror Pirate 3/6.
/// "Horrors you control have menace. Whenever a Horror you control deals
/// combat damage to a player, that player mills that many cards. At the
/// beginning of your end step, choose target artifact or creature card in an
/// opponent's graveyard that was put there from their library this turn. Put
/// it onto the battlefield under your control." "Put there from their library
/// this turn" is `PutIntoGraveyardFromLibraryThisTurn` (milled; a surveil
/// isn't counted).
pub fn captain_nghathrod() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Horrors you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Horror).and(R::ControlledByYou),
                ),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .dealt_by(R::HasCreatureType(CreatureType::Horror)),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::TriggerEventAmount,
                },
            },
            TriggeredAbility {
                event: your_step(TurnStep::End),
                effect: Effect::Move {
                    what: target_filtered(
                        artifact_or_creature()
                            .and(R::InOpponentGraveyard)
                            .and(R::PutIntoGraveyardFromLibraryThisTurn),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..legend(
            "Captain N'ghathrod",
            cost(&[generic(3), u(), b()]),
            3,
            6,
            vec![CreatureType::Horror, CreatureType::Pirate],
            vec![],
        )
    }
}

/// Grazilaxx, Illithid Scholar — {1}{U}{U} Legendary Creature — Horror 3/2.
/// "Whenever a creature you control becomes blocked, you may return it to its
/// owner's hand. Whenever one or more creatures you control deal combat damage
/// to a player, draw a card."
pub fn grazilaxx_illithid_scholar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Return the blocked creature to its owner's hand?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .once_per_batch(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..legend(
            "Grazilaxx, Illithid Scholar",
            cost(&[generic(1), u(), u()]),
            3,
            2,
            vec![CreatureType::Horror],
            vec![],
        )
    }
}

/// Zellix, Sanity Flayer — {2}{U} Legendary Creature — Horror 2/3.
/// "Hive Mind — Whenever a player mills one or more creature cards, you
/// create a 1/1 black Horror creature token. {1}, {T}: Target player mills
/// three cards. Choose a Background."
pub fn zellix_sanity_flayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardMilled, EventScope::AnyPlayer)
                .with_filter(trigger_matches(R::Creature))
                .once_per_batch(),
            effect: mint(black_horror(), 1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..legend(
            "Zellix, Sanity Flayer",
            cost(&[generic(2), u()]),
            2,
            3,
            vec![CreatureType::Horror],
            vec![Keyword::ChooseABackground],
        )
    }
}

/// Toxrill, the Corrosive — {5}{B}{B} Legendary Creature — Slug Horror 7/7.
/// "At the beginning of each end step, put a slime counter on each creature
/// you don't control. Creatures you don't control get -1/-1 for each slime
/// counter on them. Whenever a creature you don't control with a slime counter
/// on it dies, create a 1/1 black Slug creature token. {U}{B}, Sacrifice a
/// Slug: Draw a card." The per-counter shrink is one -1/-1 anthem per counter
/// threshold, stacked to 15 counters.
pub fn toxrill_the_corrosive() -> CardDefinition {
    let shrink = (1..=15)
        .map(|n| StaticAbility {
            description: "Creatures you don't control get -1/-1 for each slime counter on them.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::WithCounterAtLeast(CounterType::Slime, n)),
                power: -1,
                toughness: -1,
                keywords: vec![],
                opponents: true,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        })
        .collect();
    CardDefinition {
        static_abilities: shrink,
        triggered_abilities: vec![
            TriggeredAbility {
                event: each_step(TurnStep::End),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    kind: CounterType::Slime,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl)
                    .with_filter(trigger_matches(R::WithCounter(CounterType::Slime))),
                effect: mint(token("Slug", 1, 1, vec![Color::Black], vec![CreatureType::Slug]), 1),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), b()]),
            sac_other_filter: Some((
                R::HasCreatureType(CreatureType::Slug).and(R::ControlledByYou),
                1,
            )),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Toxrill, the Corrosive",
            cost(&[generic(5), b(), b()]),
            7,
            7,
            vec![CreatureType::Slug, CreatureType::Horror],
            vec![],
        )
    }
}

/// Falthis, Shadowcat Familiar — {2}{B} Legendary Creature — Nightmare Cat 2/2.
/// "Commanders you control have menace and deathtouch. Partner."
pub fn falthis_shadowcat_familiar() -> CardDefinition {
    CardDefinition {
        can_be_commander: true,
        static_abilities: vec![StaticAbility {
            description: "Commanders you control have menace and deathtouch.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::IsCommander,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Menace, Keyword::Deathtouch],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..legend(
            "Falthis, Shadowcat Familiar",
            cost(&[generic(2), b()]),
            2,
            2,
            vec![CreatureType::Nightmare, CreatureType::Cat],
            vec![Keyword::Partner],
        )
    }
}

/// Braids, Arisen Nightmare — {1}{B}{B} Legendary Creature — Nightmare 3/3.
/// "At the beginning of your end step, you may sacrifice an artifact,
/// creature, enchantment, land, or planeswalker. If you do, each opponent may
/// sacrifice a permanent of their choice that shares a card type with it. For
/// each opponent who doesn't, that player loses 2 life and you draw a card."
/// Modelled as a choice among the five card types (or declining), then the
/// sacrifice of a permanent of that type; "shares a card type" reads as "has
/// the chosen type".
pub fn braids_arisen_nightmare() -> CardDefinition {
    let braid = |ty: CardType| {
        Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::HasCardType(ty.clone()).and(R::ControlledByYou),
            },
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::MaySacrifice {
                        description: format!("Sacrifice a {ty:?} to deny Braids?"),
                        filter: R::HasCardType(ty).and(R::ControlledByYou),
                        count: Value::ONE,
                        then: Box::new(Effect::Noop),
                        else_: Some(Box::new(Effect::Seq(vec![
                            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                            Effect::Draw {
                                who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                                    Selector::This,
                                ))),
                                amount: Value::ONE,
                            },
                        ]))),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::ChooseMode(vec![
                braid(CardType::Artifact),
                braid(CardType::Creature),
                braid(CardType::Enchantment),
                braid(CardType::Land),
                braid(CardType::Planeswalker),
                // "You may sacrifice" — declining is its own choice.
                Effect::Noop,
            ]),
        }],
        ..legend(
            "Braids, Arisen Nightmare",
            cost(&[generic(1), b(), b()]),
            3,
            3,
            vec![CreatureType::Nightmare],
            vec![],
        )
    }
}

/// Arvinox, the Mind Flail — {4}{B}{B}{B} Legendary Enchantment Creature —
/// Horror 9/9. "Arvinox isn't a creature unless you control three or more
/// permanents you don't own. At the beginning of your end step, exile the
/// bottom card of each opponent's library face down. For as long as those
/// cards remain exiled, you may look at them, you may cast permanent spells
/// from among them, and you may spend mana as though it were mana of any color
/// to cast those spells." Approximation: the cards are exiled face up.
pub fn arvinox_the_mind_flail() -> CardDefinition {
    CardDefinition {
        name: "Arvinox, the Mind Flail",
        cost: cost(&[generic(4), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 9,
        toughness: 9,
        static_abilities: vec![StaticAbility {
            description: "Arvinox isn't a creature unless you control three or more permanents you don't own.",
            effect: StaticEffect::NotCreatureUnless {
                condition: Predicate::ValueAtLeast(
                    Value::PermanentCountControlledByMatching(
                        PlayerRef::You,
                        R::OwnedByYou.negate(),
                    ),
                    Value::Const(3),
                ),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::Seq(vec![
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::Move {
                        what: Selector::BottomOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::ExileWithSourceStamp,
                    }),
                },
                may_play_any_color(Selector::MatchingAmong {
                    inner: Box::new(Selector::CardExiledWithSource),
                    filter: R::PermanentCard.and(R::Nonland),
                }),
            ]),
        }],
        ..Default::default()
    }
}

// ── creatures ────────────────────────────────────────────────────────────────

/// Ancient Cellarspawn — {1}{B}{B} Enchantment Creature — Horror 3/3.
/// "Each spell you cast that's a Demon, Horror, or Nightmare costs {1} less
/// to cast. Whenever you cast a spell, if the amount of mana spent to cast it
/// was less than its mana value, target opponent loses life equal to the
/// difference."
pub fn ancient_cellarspawn() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Each spell you cast that's a Demon, Horror, or Nightmare costs {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasCreatureType(CreatureType::Demon)
                    .or(R::HasCreatureType(CreatureType::Horror))
                    .or(R::HasCreatureType(CreatureType::Nightmare)),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    Value::Sum(vec![Value::CastSpellManaSpent, Value::ONE]),
                ),
            ),
            effect: Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::Diff(
                    Box::new(Value::ManaValueOf(Box::new(Selector::TriggerSource))),
                    Box::new(Value::CastSpellManaSpent),
                ),
            },
        }],
        ..creature(
            "Ancient Cellarspawn",
            cost(&[generic(1), b(), b()]),
            3,
            3,
            vec![CreatureType::Horror],
            vec![],
        )
    }
}

fn slime_target() -> Effect {
    Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::OtherThanSource)),
            kind: CounterType::Slime,
            amount: Value::ONE,
        }),
    }
}

/// Sludge Monster — {3}{U}{U} Creature — Horror 5/5. "Whenever this creature
/// enters or attacks, put a slime counter on up to one other target creature.
/// Non-Horror creatures with slime counters on them lose all abilities and
/// have base power and toughness 2/2."
pub fn sludge_monster() -> CardDefinition {
    let slimed = || {
        Selector::EachPermanent(
            R::Creature
                .and(R::WithCounter(CounterType::Slime))
                .and(R::HasCreatureType(CreatureType::Horror).negate()),
        )
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Non-Horror creatures with slime counters on them lose all abilities.",
                effect: StaticEffect::MatchingLoseAllAbilities { applies_to: slimed() },
            },
            StaticAbility {
                description: "Non-Horror creatures with slime counters on them have base power and toughness 2/2.",
                effect: StaticEffect::SetBasePtForFilter { applies_to: slimed(), power: 2, toughness: 2 },
            },
        ],
        triggered_abilities: vec![etb(slime_target()), on_attack(slime_target())],
        ..creature(
            "Sludge Monster",
            cost(&[generic(3), u(), u()]),
            5,
            5,
            vec![CreatureType::Horror],
            vec![],
        )
    }
}

/// Nemesis of Reason — {3}{U}{B} Creature — Leviathan Horror 3/7. "Whenever
/// this creature attacks, defending player mills ten cards."
pub fn nemesis_of_reason() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Mill {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::Const(10),
        })],
        ..creature(
            "Nemesis of Reason",
            cost(&[generic(3), u(), b()]),
            3,
            7,
            vec![CreatureType::Leviathan, CreatureType::Horror],
            vec![],
        )
    }
}

/// Aboleth Spawn — {2}{U} Creature — Fish Horror 2/3. Flash, ward {2}.
/// "Probing Telepathy — Whenever a creature entering under an opponent's
/// control causes a triggered ability of that creature to trigger, you may
/// copy that ability. You may choose new targets for the copy."
///
/// Probing Telepathy is `StaticEffect::CopyOpponentsEnteringCreatureTriggers`,
/// read where an entering creature's own ETB-caused triggers are pushed: each
/// fire gets a copy controlled by Aboleth's controller, targets picked for
/// them. Residual: the copy goes on the stack alongside the original rather
/// than via an Aboleth trigger resolving, and the "you may" is asked as the
/// copy resolves.
pub fn aboleth_spawn() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Probing Telepathy — Whenever a creature entering under an opponent's \
                control causes a triggered ability of that creature to trigger, you may copy \
                that ability. You may choose new targets for the copy.",
            effect: StaticEffect::CopyOpponentsEnteringCreatureTriggers,
        }],
        ..creature(
            "Aboleth Spawn",
            cost(&[generic(2), u()]),
            2,
            3,
            vec![CreatureType::Fish, CreatureType::Horror],
            vec![Keyword::Flash, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        )
    }
}

/// Wharf Infiltrator — {1}{U} Creature — Human Horror 1/1. Skulk. "Whenever
/// this creature deals combat damage to a player, you may draw a card. If you
/// do, discard a card. Whenever you discard a creature card, you may pay {2}.
/// If you do, create a 3/2 colorless Eldrazi Horror creature token."
pub fn wharf_infiltrator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Draw a card, then discard a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ])),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
                    .with_filter(trigger_matches(R::Creature)),
                effect: Effect::MayPay {
                    description: "Pay {2} to create a 3/2 Eldrazi Horror?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(mint(
                        token(
                            "Eldrazi Horror",
                            3,
                            2,
                            vec![],
                            vec![CreatureType::Eldrazi, CreatureType::Horror],
                        ),
                        1,
                    )),
                    else_: None,
                },
            },
        ],
        ..creature(
            "Wharf Infiltrator",
            cost(&[generic(1), u()]),
            1,
            1,
            vec![CreatureType::Human, CreatureType::Horror],
            vec![Keyword::Skulk],
        )
    }
}

/// Yarok's Fenlurker — {B}{B} Creature — Horror 1/1. "When this creature
/// enters, each opponent exiles a card from their hand. {2}{B}: This creature
/// gets +1/+1 until end of turn." (The exiled card is auto-picked by hand
/// order.)
pub fn yaroks_fenlurker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Yarok's Fenlurker", cost(&[b(), b()]), 1, 1, vec![CreatureType::Horror], vec![])
    }
}

/// Elder Brain — {5}{B}{B} Creature — Horror 6/6. Menace. "Whenever this
/// creature attacks a player, exile all cards from that player's hand, then
/// they draw that many cards. You may play lands and cast spells from among
/// the exiled cards for as long as they remain exiled. If you cast a spell
/// this way, you may spend mana as though it were mana of any color to cast
/// it." (Attacking a planeswalker reads its controller.)
pub fn elder_brain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::DefendingPlayer,
                    zone: crate::card::Zone::Hand,
                    filter: R::Any,
                },
                to: ZoneDest::ExileWithSourceStamp,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::CountOf(Box::new(Selector::LastMoved)),
            },
            may_play_any_color(Selector::LastMoved),
        ]))],
        ..creature(
            "Elder Brain",
            cost(&[generic(5), b(), b()]),
            6,
            6,
            vec![CreatureType::Horror],
            vec![Keyword::Menace],
        )
    }
}

/// Mind Flayer — {3}{U}{U} Creature — Horror 3/3. "Dominate Monster — When
/// this creature enters, gain control of target creature for as long as you
/// control this creature." (Control lasts while Mind Flayer remains on the
/// battlefield — the Sower of Temptation duration.)
pub fn mind_flayer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: target_filtered(R::Creature),
        })],
        ..creature("Mind Flayer", cost(&[generic(3), u(), u()]), 3, 3, vec![CreatureType::Horror], vec![])
    }
}

/// Dread Presence — {3}{B} Creature — Nightmare 3/3. "Whenever a Swamp you
/// control enters, choose one — • You draw a card and you lose 1 life.
/// • This creature deals 2 damage to any target and you gain 2 life."
pub fn dread_presence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(trigger_matches(R::HasLandType(LandType::Swamp))),
            effect: Effect::ChooseMode(vec![
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            ]),
        }],
        ..creature("Dread Presence", cost(&[generic(3), b()]), 3, 3, vec![CreatureType::Nightmare], vec![])
    }
}

/// Brainstealer Dragon — {5}{B}{B} Creature — Dragon Horror 6/6. Flying. "At
/// the beginning of your end step, exile the top card of each opponent's
/// library. You may play those cards for as long as they remain exiled. If you
/// cast a spell this way, you may spend mana as though it were mana of any
/// color to cast it. Whenever a nonland permanent an opponent owns enters the
/// battlefield under your control, they lose life equal to its mana value."
pub fn brainstealer_dragon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_step(TurnStep::End),
                effect: Effect::Seq(vec![
                    Effect::EachPlayerDoes {
                        who: PlayerRef::EachOpponent,
                        body: Box::new(Effect::Move {
                            what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                            to: ZoneDest::ExileWithSourceStamp,
                        }),
                    },
                    may_play_any_color(Selector::LastMoved),
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(trigger_matches(R::Nonland.and(R::OwnedByYou.negate()))),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..creature(
            "Brainstealer Dragon",
            cost(&[generic(5), b(), b()]),
            6,
            6,
            vec![CreatureType::Dragon, CreatureType::Horror],
            vec![Keyword::Flying],
        )
    }
}

/// Grell Philosopher — {2}{U} Creature — Horror Wizard 1/4. "Aberrant
/// Tinkering — When this creature enters and at the beginning of your upkeep,
/// each Horror you control gains all activated abilities of target artifact
/// an opponent controls until end of turn. You may spend blue mana as though
/// it were mana of any color to activate those abilities."
/// Approximation: only Grell itself (a Horror) gains the abilities; the
/// blue-mana rider is omitted.
pub fn grell_philosopher() -> CardDefinition {
    let tinker = || Effect::GainAllActivatedAbilitiesOf {
        what: target_filtered(R::Artifact.and(R::ControlledByOpponent)),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(tinker()),
            TriggeredAbility { event: your_step(TurnStep::Upkeep), effect: tinker() },
        ],
        ..creature(
            "Grell Philosopher",
            cost(&[generic(2), u()]),
            1,
            4,
            vec![CreatureType::Horror, CreatureType::Wizard],
            vec![],
        )
    }
}

/// Intellect Devourer — {3}{B} Creature — Horror 2/4. "Devour Intellect —
/// When this creature enters, each opponent exiles a card from their hand
/// until this creature leaves the battlefield. Body Thief — You may play lands
/// and cast spells from among cards exiled with this creature. If you cast a
/// spell this way, you may spend mana as though it were mana of any color to
/// cast it."
pub fn intellect_devourer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::ExileChosenUntilSourceLeaves {
                    from: Selector::Player(PlayerRef::You),
                    count: Value::ONE,
                    filter: R::Any,
                    return_to: crate::card::ExileReturnZone::Hand,
                }),
            },
            may_play_any_color(Selector::CardExiledWithSource),
        ]))],
        ..creature(
            "Intellect Devourer",
            cost(&[generic(3), b()]),
            2,
            4,
            vec![CreatureType::Horror],
            vec![],
        )
    }
}

/// Defiler of Flesh — {2}{B}{B} Creature — Phyrexian Horror 4/4. Menace. "As
/// an additional cost to cast black permanent spells, you may pay 2 life.
/// Those spells cost {B} less to cast if you paid life this way. Whenever you
/// cast a black permanent spell, target creature you control gets +1/+1 and
/// gains menace until end of turn."
/// Residual: the optional pay-2-life discount is omitted.
pub fn defiler_of_flesh() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(R::HasColor(Color::Black).and(R::PermanentCard)),
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(your_creature()),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Defiler of Flesh",
            cost(&[generic(2), b(), b()]),
            4,
            4,
            vec![CreatureType::Phyrexian, CreatureType::Horror],
            vec![Keyword::Menace],
        )
    }
}

/// Haunt of the Dead Marshes — {B} Creature — Nightmare Elf 1/1. "When this
/// creature enters, scry 1. {2}{B}: Return this card from your graveyard to
/// the battlefield tapped. Activate only if you control a legendary creature."
pub fn haunt_of_the_dead_marshes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
                your_creature().and(R::HasSupertype(Supertype::Legendary)),
            ))),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature(
            "Haunt of the Dead Marshes",
            cost(&[b()]),
            1,
            1,
            vec![CreatureType::Nightmare, CreatureType::Elf],
            vec![],
        )
    }
}

/// Grimdancer — {1}{B}{B} Creature — Nightmare 3/3. "This creature enters with
/// your choice of two different counters on it from among menace, deathtouch,
/// and lifelink." (The pair is picked by an ETB mode choice — the Boot Nipper
/// shape.)
pub fn grimdancer() -> CardDefinition {
    let pair = |a: Keyword, b: Keyword| {
        Effect::Seq(vec![
            Effect::AddKeywordCounter { what: Selector::This, keyword: a, amount: Value::ONE },
            Effect::AddKeywordCounter { what: Selector::This, keyword: b, amount: Value::ONE },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            pair(Keyword::Menace, Keyword::Deathtouch),
            pair(Keyword::Menace, Keyword::Lifelink),
            pair(Keyword::Deathtouch, Keyword::Lifelink),
        ]))],
        ..creature("Grimdancer", cost(&[generic(1), b(), b()]), 3, 3, vec![CreatureType::Nightmare], vec![])
    }
}

/// Abyssal Harvester — {1}{B}{B} Creature — Demon Warlock 3/2. "{T}: Exile
/// target creature card from a graveyard that was put there this turn. Create
/// a token that's a copy of it, except it's a Nightmare in addition to its
/// other types. Then exile all other Nightmare tokens you control."
/// (The old Nightmare tokens are exiled before the new copy is made — the
/// same end state.)
pub fn abyssal_harvester() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(
                        R::Creature.and(R::InGraveyard).and(R::PutIntoGraveyardThisTurn),
                    ),
                },
                Effect::Exile {
                    what: Selector::EachPermanent(
                        R::IsToken
                            .and(R::HasCreatureType(CreatureType::Nightmare))
                            .and(R::ControlledByYou),
                    ),
                },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![CreatureType::Nightmare],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Abyssal Harvester",
            cost(&[generic(1), b(), b()]),
            3,
            2,
            vec![CreatureType::Demon, CreatureType::Warlock],
            vec![],
        )
    }
}

/// Forgotten Creation — {3}{U} Creature — Zombie Horror 3/3. Skulk. "At the
/// beginning of your upkeep, you may discard all the cards in your hand. If
/// you do, draw that many cards."
pub fn forgotten_creation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::Upkeep),
            effect: Effect::MayDo {
                description: "Discard your hand and draw that many cards?".into(),
                body: Box::new(Effect::DiscardHandDrawThatMany { who: Selector::You }),
            },
        }],
        ..creature(
            "Forgotten Creation",
            cost(&[generic(3), u()]),
            3,
            3,
            vec![CreatureType::Zombie, CreatureType::Horror],
            vec![Keyword::Skulk],
        )
    }
}

/// Vashta Nerada — {2}{B} Creature — Alien Horror 1/1. Indestructible, shadow.
/// "Morbid — At the beginning of each end step, if a creature died this turn,
/// put a +1/+1 counter on this creature."
pub fn vashta_nerada() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: each_step(TurnStep::End).with_filter(
                Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Vashta Nerada",
            cost(&[generic(2), b()]),
            1,
            1,
            vec![CreatureType::Alien, CreatureType::Horror],
            vec![Keyword::Indestructible, Keyword::Shadow],
        )
    }
}

// ── enchantment / artifacts ──────────────────────────────────────────────────

/// Endless Evil — {2}{U} Enchantment — Aura. "Enchant creature you control. At
/// the beginning of your upkeep, create a token that's a copy of enchanted
/// creature, except the token is 1/1. When enchanted creature dies, if that
/// creature was a Horror, return this card to its owner's hand."
pub fn endless_evil() -> CardDefinition {
    CardDefinition {
        name: "Endless Evil",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        attach_only_filter: Some(your_creature()),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(your_creature()) },
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_step(TurnStep::Upkeep),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::AttachedTo(Box::new(Selector::This)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource)
                    .with_filter(trigger_matches(R::HasCreatureType(CreatureType::Horror))),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
            },
        ],
        ..Default::default()
    }
}

/// Stone of Erech — {1} Legendary Artifact. "If a creature an opponent
/// controls would die, exile it instead. {2}, {T}, Sacrifice Stone of Erech:
/// Exile target player's graveyard. Draw a card." (The replacement is the
/// engine's Valentin one, which covers nontoken creatures.)
pub fn stone_of_erech() -> CardDefinition {
    CardDefinition {
        name: "Stone of Erech",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: None },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::Seq(vec![
                    Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Panharmonicon — {4} Artifact. "If an artifact or creature entering causes
/// a triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time." (Doubles triggers caused by any permanent
/// entering — the Yarok static.)
pub fn panharmonicon() -> CardDefinition {
    CardDefinition {
        name: "Panharmonicon",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If an artifact or creature entering causes a triggered ability of a permanent you control to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerEtbTriggers,
        }],
        ..Default::default()
    }
}

// ── instants ─────────────────────────────────────────────────────────────────

/// Ghostly Flicker — {2}{U} Instant. "Exile two target artifacts, creatures,
/// and/or lands you control, then return those cards to the battlefield under
/// your control."
pub fn ghostly_flicker() -> CardDefinition {
    spell(
        "Ghostly Flicker",
        cost(&[generic(2), u()]),
        false,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Artifact.or(R::Creature).or(R::Land).and(R::ControlledByYou),
            effect: Box::new(Effect::ExileAndReturnToOwner { what: Selector::Target(0) }),
        },
    )
}

/// Drown in Dreams — {X}{2}{U} Instant. "Choose one. If you control a
/// commander as you cast this spell, you may choose both instead. • Target
/// player draws X cards. • Target player mills twice X cards."
pub fn drown_in_dreams() -> CardDefinition {
    spell(
        "Drown in Dreams",
        cost(&[x(), generic(2), u()]),
        false,
        commander_both(|| {
            vec![
                Effect::Draw { who: target_filtered(R::Player), amount: Value::XFromCost },
                Effect::Mill {
                    who: target_filtered(R::Player),
                    amount: Value::Times(Box::new(Value::XFromCost), Box::new(Value::Const(2))),
                },
            ]
        }),
    )
}

/// Essence Flux — {U} Instant. "Exile target creature you control, then
/// return that card to the battlefield under its owner's control. If it's a
/// Spirit, put a +1/+1 counter on it."
pub fn essence_flux() -> CardDefinition {
    spell(
        "Essence Flux",
        cost(&[u()]),
        false,
        Effect::Seq(vec![
            Effect::ExileAndReturnToOwner { what: target_filtered(your_creature()) },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasCreatureType(CreatureType::Spirit),
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Miscast — {U} Instant. "Counter target instant or sorcery spell unless its
/// controller pays {3}."
pub fn miscast() -> CardDefinition {
    spell(
        "Miscast",
        cost(&[u()]),
        false,
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery())),
            mana_cost: cost(&[generic(3)]),
            exile: false,
            extra_generic: None,
            if_paid: None,
        },
    )
}

/// Displace — {2}{U} Instant. "Exile up to two target creatures you control,
/// then return those cards to the battlefield under their owner's control."
pub fn displace() -> CardDefinition {
    spell(
        "Displace",
        cost(&[generic(2), u()]),
        false,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: your_creature(),
            effect: Box::new(Effect::ExileAndReturnToOwner { what: Selector::Target(0) }),
        },
    )
}

/// Blot Out — {2}{B} Instant. "Target opponent exiles a creature or
/// planeswalker they control with the greatest mana value among creatures and
/// planeswalkers they control." (Ties break by board order.)
pub fn blot_out() -> CardDefinition {
    spell(
        "Blot Out",
        cost(&[generic(2), b()]),
        false,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Exile {
                what: Selector::GreatestManaValueControlledMatching {
                    who: PlayerRef::Target(0),
                    filter: R::Creature.or(R::Planeswalker),
                },
            }),
        },
    )
}

/// Spoils of Blood — {B} Instant. "Create an X/X black Horror creature token,
/// where X is the number of creatures that died this turn."
pub fn spoils_of_blood() -> CardDefinition {
    let horror = TokenDefinition {
        dynamic_pt: Some((Value::CreaturesDiedThisTurnTotal, Value::CreaturesDiedThisTurnTotal)),
        ..token("Horror", 0, 0, vec![Color::Black], vec![CreatureType::Horror])
    };
    spell("Spoils of Blood", cost(&[b()]), false, mint(horror, 1))
}

/// Didn't Say Please — {1}{U}{U} Instant. "Counter target spell. Its
/// controller mills three cards."
pub fn didnt_say_please() -> CardDefinition {
    spell(
        "Didn't Say Please",
        cost(&[generic(1), u(), u()]),
        false,
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::Mill {
                who: Selector::Player(PlayerRef::CounteredSpellController),
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Hide on the Ceiling — {X}{U} Instant. "Exile X target artifacts and/or
/// creatures. Return the exiled cards to the battlefield under their owners'
/// control at the beginning of the next end step."
pub fn hide_on_the_ceiling() -> CardDefinition {
    spell(
        "Hide on the Ceiling",
        cost(&[x(), u()]),
        false,
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: artifact_or_creature(),
                effect: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                    what: Selector::Target(0),
                    tapped: false,
                }),
            }),
        },
    )
}

/// Planar Incision — {1}{U} Instant. "Exile target artifact or creature, then
/// return it to the battlefield under its owner's control with a +1/+1
/// counter on it."
pub fn planar_incision() -> CardDefinition {
    spell(
        "Planar Incision",
        cost(&[generic(1), u()]),
        false,
        Effect::Seq(vec![
            Effect::ExileAndReturnToOwner { what: target_filtered(artifact_or_creature()) },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Countersquall — {U}{B} Instant. "Counter target noncreature spell. Its
/// controller loses 2 life."
pub fn countersquall() -> CardDefinition {
    spell(
        "Countersquall",
        cost(&[u(), b()]),
        false,
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)) },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::CounteredSpellController),
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Szat's Will — {4}{B} Instant. "Choose one. If you control a commander as
/// you cast this spell, you may choose both instead. • Each opponent
/// sacrifices a creature they control with the greatest power. • Exile all
/// opponents' graveyards, then create X 0/1 black Thrull creature tokens,
/// where X is the greatest power among creature cards exiled this way."
/// (X is read off the graveyards just before they are exiled.)
pub fn szats_will() -> CardDefinition {
    spell(
        "Szat's Will",
        cost(&[generic(4), b()]),
        false,
        commander_both(|| {
            vec![
                Effect::SacrificeGreatestMV {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::ONE,
                    filter: R::Creature,
                    by_power: true,
                },
                Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::GreatestPowerAmongCards(Box::new(Selector::CardsInZone {
                            who: PlayerRef::EachOpponent,
                            zone: crate::card::Zone::Graveyard,
                            filter: R::Creature,
                        })),
                        definition: std::sync::Arc::new(token(
                            "Thrull",
                            0,
                            1,
                            vec![Color::Black],
                            vec![CreatureType::Thrull],
                        )),
                    },
                    Effect::ExileAllGraveyards { filter: None, opponents_only: true },
                ]),
            ]
        }),
    )
}

// ── sorceries ────────────────────────────────────────────────────────────────

/// Inside Information — {X}{B}{B} Sorcery. "Exile the top X cards of target
/// opponent's library. You may play those cards this turn. If you cast a
/// spell this way, pay life equal to its mana value rather than pay its mana
/// cost."
pub fn inside_information() -> CardDefinition {
    spell(
        "Inside Information",
        cost(&[x(), b(), b()]),
        true,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::Target(0), count: Value::XFromCost },
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlayForLife {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfThisTurn,
                },
            ])),
        },
    )
}

/// Essence Harvest — {2}{B} Sorcery. "Target player loses X life and you gain
/// X life, where X is the greatest power among creatures you control."
pub fn essence_harvest() -> CardDefinition {
    let x_power = || Value::GreatestPowerControlled { who: PlayerRef::You };
    spell(
        "Essence Harvest",
        cost(&[generic(2), b()]),
        true,
        Effect::Seq(vec![
            Effect::LoseLife { who: target_filtered(R::Player), amount: x_power() },
            Effect::GainLife { who: Selector::You, amount: x_power() },
        ]),
    )
}

/// Nightmare Unmaking — {3}{B}{B} Sorcery. "Choose one — • Exile each creature
/// with power greater than the number of cards in your hand. • Exile each
/// creature with power less than the number of cards in your hand."
pub fn nightmare_unmaking() -> CardDefinition {
    let hand = || Value::HandSizeOf(PlayerRef::You);
    let power = || Value::PowerOf(Box::new(Selector::TriggerSource));
    let exile_each = |cond: Predicate| Effect::ForEach {
        selector: Selector::EachPermanent(R::Creature),
        body: Box::new(Effect::If {
            cond,
            then: Box::new(Effect::Exile { what: Selector::TriggerSource }),
            else_: Box::new(Effect::Noop),
        }),
    };
    spell(
        "Nightmare Unmaking",
        cost(&[generic(3), b(), b()]),
        true,
        Effect::ChooseMode(vec![
            exile_each(Predicate::ValueAtLeast(power(), Value::Sum(vec![hand(), Value::ONE]))),
            exile_each(Predicate::ValueAtLeast(hand(), Value::Sum(vec![power(), Value::ONE]))),
        ]),
    )
}

/// Dream Harvest — {5}{U/B}{U/B} Sorcery. "Each opponent exiles cards from the
/// top of their library until they have exiled cards with total mana value 5
/// or greater this way. Until end of turn, you may cast cards exiled this way
/// without paying their mana costs."
pub fn dream_harvest() -> CardDefinition {
    spell(
        "Dream Harvest",
        cost(&[
            generic(5),
            hybrid(Color::Blue, Color::Black),
            hybrid(Color::Blue, Color::Black),
        ]),
        true,
        Effect::Seq(vec![
            Effect::EachPlayerDoes {
                who: PlayerRef::EachOpponent,
                body: Box::new(Effect::Move {
                    what: Selector::TopOfLibraryUntilMvAtLeast {
                        who: PlayerRef::You,
                        threshold: Value::Const(5),
                    },
                    to: ZoneDest::Exile,
                }),
            },
            Effect::GrantMayPlay {
                what: Selector::MatchingAmong {
                    inner: Box::new(Selector::LastMoved),
                    filter: R::Nonland,
                },
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: false,
                any_color: false,
            },
        ]),
    )
}

/// Final Act — {4}{B}{B} Sorcery. "Choose one or more — • Destroy all
/// creatures. • Destroy all planeswalkers. • Destroy all battles. • Exile all
/// graveyards. • Each opponent loses all counters."
pub fn final_act() -> CardDefinition {
    spell(
        "Final Act",
        cost(&[generic(4), b(), b()]),
        true,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
                Effect::Destroy { what: Selector::EachPermanent(R::Planeswalker) },
                Effect::Destroy { what: Selector::EachPermanent(R::HasCardType(CardType::Battle)) },
                Effect::ExileAllGraveyards { filter: None, opponents_only: false },
                Effect::RemoveAllPlayerCounters { who: PlayerRef::EachOpponent },
            ],
            min: 1,
            max: 5,
            allow_repeats: false,
        },
    )
}

/// Mandate of Abaddon — {3}{B} Sorcery. "Choose target creature you control.
/// Destroy all creatures with power less than that creature's power."
pub fn mandate_of_abaddon() -> CardDefinition {
    spell(
        "Mandate of Abaddon",
        cost(&[generic(3), b()]),
        true,
        // The target rides the walked set (it never has less power than
        // itself) so the cast declares it.
        Effect::ForEach {
            selector: Selector::Both(
                Box::new(target_filtered(your_creature())),
                Box::new(Selector::EachPermanent(R::Creature)),
            ),
            body: Box::new(Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::Target(0))),
                    Value::Sum(vec![
                        Value::PowerOf(Box::new(Selector::TriggerSource)),
                        Value::ONE,
                    ]),
                ),
                then: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                else_: Box::new(Effect::Noop),
            }),
        },
    )
}

/// Rite of Consumption — {1}{B} Sorcery. "As an additional cost to cast this
/// spell, sacrifice a creature. Rite of Consumption deals damage equal to the
/// sacrificed creature's power to target player or planeswalker. You gain
/// life equal to the damage dealt this way."
pub fn rite_of_consumption() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..spell(
            "Rite of Consumption",
            cost(&[generic(1), b()]),
            true,
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::XFromCost,
                },
                Effect::GainLife { who: Selector::You, amount: Value::DamageDealtThisResolution },
            ]),
        )
    }
}

/// Distant Melody — {3}{U} Sorcery. "Choose a creature type. Draw a card for
/// each permanent you control of that type."
pub fn distant_melody() -> CardDefinition {
    spell(
        "Distant Melody",
        cost(&[generic(3), u()]),
        true,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::ControlledByYou.and(R::IsSourceChosenCreatureType),
                ))),
            }),
        },
    )
}

/// Psionic Ritual — {4}{U}{U} Sorcery. "Replicate—Tap an untapped Horror you
/// control. Exile target instant or sorcery card from a graveyard and copy it.
/// You may cast the copy without paying its mana cost. Exile Psionic Ritual."
/// Residual: the tap-a-Horror replicate cost is omitted.
pub fn psionic_ritual() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Psionic Ritual",
            cost(&[generic(4), u(), u()]),
            true,
            Effect::Seq(vec![
                Effect::ExileWithSource {
                    what: target_filtered(instant_or_sorcery().and(R::InGraveyard)),
                },
                Effect::CopyCardAndCastFree { what: Selector::Target(0) },
            ]),
        )
    }
}

/// Cut Your Losses — {4}{U}{U} Sorcery. Casualty 2. "Target player mills half
/// their library, rounded down."
pub fn cut_your_losses() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(2)],
        ..spell(
            "Cut Your Losses",
            cost(&[generic(4), u(), u()]),
            true,
            Effect::MillHalf { who: target_filtered(R::Player), rounded_up: false },
        )
    }
}

/// Startled Awake // Persistent Nightmare — {2}{U}{U} Sorcery // Creature —
/// Nightmare 1/1. Front: "Target opponent mills thirteen cards. {3}{U}{U}: Put
/// this card from your graveyard onto the battlefield transformed. Activate
/// only as a sorcery." Back: "Skulk. When this creature deals combat damage to
/// a player, return it to its owner's hand."
pub fn startled_awake() -> CardDefinition {
    let nightmare = CardDefinition {
        name: "Persistent Nightmare",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(nightmare)),
        ..spell(
            "Startled Awake",
            cost(&[generic(2), u(), u()]),
            true,
            Effect::Mill { who: target_filtered(R::OpponentPlayer), amount: Value::Const(13) },
        )
    }
}

/// Opposition Agent — {2}{B} Creature — Human Rogue 3/2. Flash.
/// "You control your opponents while they're searching their libraries.
/// While an opponent is searching their library, they exile each card they
/// find. You may play those cards for as long as they remain exiled, and you
/// may spend mana as though it were mana of any color to cast them."
///
/// `StaticEffect::ControlOpponentsSearches`: the search resolvers route the
/// pick to this permanent's controller and exile each library find with a
/// `WhileExiled` may-play grant for them, cast cost = mana value as generic.
/// The rest of the searching effect (shuffle, a fetch land's life and
/// sacrifice) still happens. See the module Residuals for the search effects
/// the hijack doesn't reach.
pub fn opposition_agent() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You control your opponents while they're searching their libraries. \
                          While an opponent is searching their library, they exile each card \
                          they find. You may play those cards for as long as they remain \
                          exiled, and you may spend mana as though it were mana of any color \
                          to cast them.",
            effect: StaticEffect::ControlOpponentsSearches,
        }],
        ..creature(
            "Opposition Agent",
            cost(&[generic(2), b()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Rogue],
            vec![Keyword::Flash],
        )
    }
}
