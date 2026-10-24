//! Commander: the 2026-10-12 rescan's ≤3-card EDHREC gaps of the top 600,
//! twelfth file (`cmdr_edhrec11` is the eleventh). Tests in
//! `tests/recent_b/cmdr_edhrec12.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost, EnchantmentSubtype,
    EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType, LoyaltyAbility,
    PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{
    ActivatedAbility, DelayedTriggerKind, Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, ZoneDest,
    ZoneRef,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost, SpendRestriction};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn spell_cast(scope: EventScope, filter: R) -> EventSpec {
    EventSpec::new(EventKind::SpellCast, scope)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter })
}

/// "Look at the top `count` cards, one into your hand, the rest on the bottom
/// in a random order."
fn look_pick_one(count: Value) -> Effect {
    Effect::LookPickToHand(Box::new(crate::effect::LookPick {
        count,
        rest_bottom_random: true,
        ..Default::default()
    }))
}

/// "Create a token that's a copy of [source]. It gains haste. Sacrifice it at
/// the beginning of the next end step." (Chrome Dome, after Kiki-Jiki.)
fn hasty_copy_until_end_step(source: Selector) -> Effect {
    Effect::Seq(vec![
        Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::ONE,
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
        },
        Effect::GrantKeyword { what: Selector::LastCreatedToken, keyword: Keyword::Haste, duration: Duration::Permanent },
        Effect::DelayUntil {
            kind: DelayedTriggerKind::NextEndStep,
            body: Box::new(Effect::SacrificeSelected { what: Selector::LastCreatedToken }),
        },
    ])
}

// ── Ruric Thar, the Unbowed ──────────────────────────────────────────────────

/// Primal Surge — {8}{G}{G} sorcery. Exile the top card of your library; if
/// it's a permanent card you may put it onto the battlefield, and if you do,
/// repeat. The loop is `RepeatWhileMay` over "the card just exiled landed on
/// the battlefield"; the extra "repeat?" ask is the same choice as declining
/// the next put. A seat whose board is past 150 permanents stops putting
/// (the engine's board bound is the reason, not the card).
pub fn primal_surge() -> CardDefinition {
    spell(
        "Primal Surge",
        cost(&[generic(8), g(), g()]),
        CardType::Sorcery,
        Effect::RepeatWhileMay {
            body: Box::new(Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::ONE,
                    link_to_source: false,
                    face_down: false,
                },
                Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::LastMoved, filter: R::PermanentCard },
                        Predicate::ValueAtMost(Value::count(Selector::EachPermanent(R::Permanent)), Value::Const(150)),
                    ]),
                    then: Box::new(Effect::MayDo {
                        description: "Put it onto the battlefield?".into(),
                        body: Box::new(Effect::Move {
                            what: Selector::LastMoved,
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            again: Predicate::EntityMatches { what: Selector::LastMoved, filter: R::OnBattlefield },
            description: "Repeat the process?".into(),
        },
    )
}

// ── The Gitrog, Ravenous Ride ────────────────────────────────────────────────

/// Relic Golem — {3} 6/6 Golem. Can't attack or block unless an opponent has
/// eight or more cards in their graveyard. {2}, {T}: target player mills two.
pub fn relic_golem() -> CardDefinition {
    let shut = || {
        Predicate::Not(Box::new(Predicate::ValueAtLeast(
            Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent),
            Value::Const(8),
        )))
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![
            static_ab(
                "Can't attack unless an opponent has eight or more cards in their graveyard.",
                StaticEffect::SelfHasKeywordWhilePredicate { keyword: Keyword::CantAttack, condition: shut() },
            ),
            static_ab(
                "Can't block unless an opponent has eight or more cards in their graveyard.",
                StaticEffect::SelfHasKeywordWhilePredicate { keyword: Keyword::CantBlock, condition: shut() },
            ),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature("Relic Golem", cost(&[generic(3)]), vec![CreatureType::Golem], 6, 6)
    }
}

/// Shakedown Heavy — {2}{B} 6/4 Ogre Warrior, menace. Whenever it attacks,
/// defending player may have you draw a card; if they do, untap it and remove
/// it from combat.
pub fn shakedown_heavy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::AnyPlayerMayAccept {
            who: PlayerRef::DefendingPlayer,
            prompt: "Have Shakedown Heavy's controller draw a card (it leaves combat)?".into(),
            accepted: Box::new(Effect::Seq(vec![
                draw(1),
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::RemoveFromCombat { what: Selector::This },
            ])),
            otherwise: Box::new(Effect::Noop),
        })],
        ..creature("Shakedown Heavy", cost(&[generic(2), b()]), vec![CreatureType::Ogre, CreatureType::Warrior], 6, 4)
    }
}

// ── Pramikon, Sky Rampart ────────────────────────────────────────────────────

/// Aether Membrane — {1}{R}{R} 0/5 Wall, defender, reach. Whenever it blocks
/// a creature, return that creature to its owner's hand at end of combat.
pub fn aether_membrane() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Move {
                    what: Selector::BlockedAttacker,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::BlockedAttacker))),
                }),
            },
        }],
        ..creature("Aether Membrane", cost(&[generic(1), r(), r()]), vec![CreatureType::Wall], 0, 5)
    }
}

/// Suppressor Skyguard — {2}{W}{U} 2/4 Human Knight, flying. Whenever a
/// player attacks you, if that player has another opponent who isn't being
/// attacked, prevent all combat damage that would be dealt to you this
/// combat (the shield runs to end of turn; a turn's second combat is the gap).
pub fn suppressor_skyguard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedDirectlyByOpponent)
                .once_per_batch(),
            // Read as the trigger resolves, against the whole declared batch.
            effect: Effect::If {
                cond: Predicate::AttackingPlayerLeftAnotherOpponentUnattacked,
                then: Box::new(Effect::PreventAllCombatDamageToPlayerThisTurn { who: PlayerRef::You }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Suppressor Skyguard",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            4,
        )
    }
}

// ── Zethi, Arcane Blademaster ────────────────────────────────────────────────

/// Zethi, Arcane Blademaster — {1}{W}{U} 3/3 Human Soldier, multikicker
/// {W/U}. When it enters, exile up to X target instant cards from your
/// graveyard (X = times kicked), each with a kick counter. Whenever Zethi
/// attacks, copy each exiled card you own with a kick counter; you may cast
/// the copies.
pub fn zethi_arcane_blademaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Multikicker(cost(&[hybrid(Color::White, Color::Blue)]))],
        triggered_abilities: vec![
            // The exile chooses rather than targets (INCOMPLETE_CARDS): an
            // ETB's auto-targeting fills one slot of an "up to X".
            etb(Effect::Seq(vec![
                Effect::ExileFromGraveyard {
                    who: PlayerRef::You,
                    count: Value::TimesKicked,
                    filter: R::HasCardType(CardType::Instant),
                },
                Effect::ForEach {
                    selector: Selector::ExiledThisResolution { filter: R::HasCardType(CardType::Instant) },
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::Kick,
                        amount: Value::ONE,
                    }),
                },
            ])),
            on_attack(Effect::ForEach {
                selector: Selector::EachMatching {
                    zone: ZoneRef::Exile,
                    filter: R::HasCardType(CardType::Instant).and(R::OwnedByYou).and(R::WithCounter(CounterType::Kick)),
                },
                body: Box::new(Effect::CopyCardAndCastFree { what: Selector::TriggerSource }),
            }),
        ],
        ..legend(
            "Zethi, Arcane Blademaster",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// You Come to a River — {1}{U} instant. Choose one: return target nonland
/// permanent to its owner's hand; or target creature gets +1/+0 and can't be
/// blocked this turn.
pub fn you_come_to_a_river() -> CardDefinition {
    spell(
        "You Come to a River",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ]),
        ]),
    )
}

// ── Karametra, God of Harvests ───────────────────────────────────────────────

/// Arctic Treeline — snow land, Forest Plains; enters tapped.
pub fn arctic_treeline() -> CardDefinition {
    CardDefinition {
        name: "Arctic Treeline",
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Forest, LandType::Plains], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add(Color::Green), crate::sets::tap_add(Color::White)],
        static_abilities: vec![crate::sets::enters_tapped()],
        ..Default::default()
    }
}

/// Yasharn, Implacable Earth — {2}{G}{W} 4/4 Elemental Boar. When it enters,
/// search for a basic Forest and a basic Plains, reveal them, put them into
/// your hand. Players can't pay life or sacrifice nonland permanents to cast
/// spells or activate abilities.
pub fn yasharn_implacable_earth() -> CardDefinition {
    let fetch = |lt: LandType| Effect::Search {
        who: PlayerRef::You,
        filter: R::IsBasicLand.and(R::HasLandType(lt)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![fetch(LandType::Forest), fetch(LandType::Plains)]))],
        static_abilities: vec![static_ab(
            "Players can't pay life or sacrifice nonland permanents to cast spells or activate abilities.",
            StaticEffect::PlayersCantPayLifeOrSacrificeNonlandForCosts,
        )],
        ..legend(
            "Yasharn, Implacable Earth",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Elemental, CreatureType::Boar],
            4,
            4,
        )
    }
}

// ── Karn, Legacy Reforged / Ultron ───────────────────────────────────────────


/// Karn, Living Legacy — {4} Karn, loyalty 4. +1: a tapped Powerstone. −1:
/// pay any amount of mana, look at that many cards, one into your hand, the
/// rest on the bottom in a random order. −7: an emblem with "Tap an untapped
/// artifact you control: this emblem deals 1 damage to any target" (granted
/// to each artifact as its own {T} ability — the residual is in
/// INCOMPLETE_CARDS).
pub fn karn_living_legacy() -> CardDefinition {
    CardDefinition {
        name: "Karn, Living Legacy",
        cost: cost(&[generic(4)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Karn], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(crate::game::effects::powerstone_token()),
                    },
                    Effect::Tap { what: Selector::LastCreatedToken },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::MayPayX {
                    description: "Pay any amount of mana to look at that many cards".into(),
                    body: Box::new(look_pick_one(Value::XFromCost)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Karn, Living Legacy".into(),
                    triggered: vec![],
                    statics: vec![static_ab(
                        "Tap an untapped artifact you control: This emblem deals 1 damage to any target.",
                        StaticEffect::GrantActivatedAbility {
                            applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                            ability: ActivatedAbility {
                                tap_cost: true,
                                effect: Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::ONE },
                                ..Default::default()
                            },
                            condition: None,
                        },
                    )],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Chrome Dome — {2} 1/3 artifact creature Robot Ninja. Other artifact
/// creatures you control get +1/+0. {5}: a hasty token copy of another target
/// artifact you control, sacrificed at the next end step.
pub fn chrome_dome() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![static_ab(
            "Other artifact creatures you control get +1/+0.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: hasty_copy_until_end_step(target_filtered(
                R::Artifact.and(R::ControlledByYou).and(R::OtherThanSource),
            )),
            ..Default::default()
        }],
        ..creature("Chrome Dome", cost(&[generic(2)]), vec![CreatureType::Robot, CreatureType::Ninja], 1, 3)
    }
}

/// The Vision — {4} 2/5 legendary artifact creature Robot Hero, flying,
/// vigilance. Whenever you cast a noncreature spell, choose one not chosen
/// this turn: double strike, indestructible (both until end of turn), or draw.
pub fn the_vision() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: spell_cast(EventScope::YourControl, R::Noncreature),
            effect: Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
                    draw(1),
                ],
            },
        }],
        ..legend("The Vision", cost(&[generic(4)]), vec![CreatureType::Robot, CreatureType::Hero], 2, 5)
    }
}

// ── Maelstrom Wanderer / Imoti ───────────────────────────────────────────────

/// Averna, the Chaos Bloom — {G}{U}{R} 4/2 Elemental Shaman. As you cascade,
/// you may put a land card from among the exiled cards onto the battlefield
/// tapped.
pub fn averna_the_chaos_bloom() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "As you cascade, you may put a land card from among the exiled cards onto the battlefield tapped.",
            StaticEffect::CascadeMayPutExiledLandTapped,
        )],
        ..legend(
            "Averna, the Chaos Bloom",
            cost(&[g(), u(), r()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            4,
            2,
        )
    }
}

/// Sakashima's Protege — {4}{U}{U} 3/1 Shapeshifter, flash, cascade. May
/// enter as a copy of any permanent that entered this turn.
pub fn sakashimas_protege() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Cascade],
        triggered_abilities: vec![crate::effect::shortcut::cascade(6)],
        enters_as_copy: Some(EntersAsCopy { filter: R::Permanent.and(R::EnteredThisTurn), ..Default::default() }),
        ..creature("Sakashima's Protege", cost(&[generic(4), u(), u()]), vec![CreatureType::Shapeshifter], 3, 1)
    }
}

/// Release to the Wind — {2}{U} instant. Exile target nonland permanent; for
/// as long as it remains exiled, its owner may cast it without paying its
/// mana cost.
pub fn release_to_the_wind() -> CardDefinition {
    spell(
        "Release to the Wind",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Permanent.and(R::Nonland)), to: ZoneDest::Exile },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::WhileExiled,
                to_owner: true,
                exile_after: false,
                pay_own_cost: false,
                any_color: false,
            },
        ]),
    )
}

/// Gilanra, Caller of Wirewood — {2}{G} 1/2 Elf Druid, partner. {T}: add
/// {G}; when you spend this mana on a spell with mana value 6 or greater,
/// draw a card (the `MarksCast` rider, which does not name its source).
pub fn gilanra_caller_of_wirewood() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Green])),
                    SpendRestriction::MarksCast,
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ManaValueAtLeast(6) },
                Predicate::CastWithMarkedMana { what: Selector::TriggerSource },
            ])),
            effect: draw(1),
        }],
        ..legend(
            "Gilanra, Caller of Wirewood",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            2,
        )
    }
}

/// Thunderous Snapper — {G/U}{G/U}{G/U}{G/U} 4/4 Turtle Hydra. Whenever you
/// cast a spell with mana value 5 or greater, draw a card.
pub fn thunderous_snapper() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: spell_cast(EventScope::YourControl, R::ManaValueAtLeast(5)),
            effect: draw(1),
        }],
        ..creature(
            "Thunderous Snapper",
            cost(&[gu(), gu(), gu(), gu()]),
            vec![CreatureType::Turtle, CreatureType::Hydra],
            4,
            4,
        )
    }
}

// ── Gluntch / The Council of Four ────────────────────────────────────────────


/// Rootweaver Druid — {2}{G} 2/1 Elf Druid. When it enters, each opponent may
/// search for up to three basic lands; each puts one onto the battlefield
/// tapped under your control and the rest tapped under their own.
pub fn rootweaver_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachOpponent,
            body: Box::new(Effect::MayDo {
                description: "Search for up to three basic lands (one goes to Rootweaver Druid's controller)?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SearchUpToN {
                        who: PlayerRef::You,
                        filter: R::IsBasicLand,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        count: Value::Const(3),
                    },
                    Effect::GainControl {
                        what: Selector::one_of(Selector::LastMoved),
                        to: Some(PlayerRef::ControllerOf(Box::new(Selector::This))),
                        duration: Duration::Permanent,
                    },
                ])),
            }),
        })],
        ..creature("Rootweaver Druid", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 1)
    }
}

/// Your Temple Is Under Attack — {2}{W} instant. Choose one: creatures you
/// control gain indestructible until end of turn; or you and target opponent
/// each draw two cards.
pub fn your_temple_is_under_attack() -> CardDefinition {
    spell(
        "Your Temple Is Under Attack",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(mine()),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Seq(vec![
                draw(2),
                Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
            ]),
        ]),
    )
}

/// Haytham Kenway — {2}{W}{U} 3/3 Human Knight, protection from Assassins.
/// Other Knights you control get +2/+2 and have protection from Assassins.
/// When it enters, for each opponent, exile up to one target creature that
/// player controls until Haytham leaves the battlefield.
pub fn haytham_kenway() -> CardDefinition {
    let knights = || Selector::EachPermanent(
        R::Creature.and(R::HasCreatureType(CreatureType::Knight)).and(R::ControlledByYou).and(R::OtherThanSource),
    );
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCreatureType(CreatureType::Assassin)],
        static_abilities: vec![
            static_ab(
                "Other Knights you control get +2/+2.",
                StaticEffect::PumpPT { applies_to: knights(), power: 2, toughness: 2 },
            ),
            static_ab(
                "Other Knights you control have protection from Assassins.",
                StaticEffect::GrantKeyword {
                    applies_to: knights(),
                    keyword: Keyword::ProtectionFromCreatureType(CreatureType::Assassin),
                },
            ),
        ],
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::Target(0),
                    return_to: ExileReturnZone::Battlefield,
                }),
            }),
        })],
        ..legend("Haytham Kenway", cost(&[generic(2), w(), u()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Dream Fracture — {1}{U}{U} instant. Counter target spell; its controller
/// draws a card. Draw a card.
pub fn dream_fracture() -> CardDefinition {
    spell(
        "Dream Fracture",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(target_filtered(R::IsSpellOnStack)))),
                amount: Value::ONE,
            },
            Effect::CounterSpell { what: Selector::Target(0) },
            draw(1),
        ]),
    )
}

// ── Zoraline, Cosmos Caller ──────────────────────────────────────────────────

/// Ghastly Gloomhunter — {1}{B} 1/1 Zombie Bat, kicker {3}{B}, flying,
/// lifelink; enters with two +1/+1 counters if kicked.
pub fn ghastly_gloomhunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), b()])), Keyword::Flying, Keyword::Lifelink],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SpellWasKicked),
                then: Box::new(Value::Const(2)),
                else_: Box::new(Value::ZERO),
            },
        )),
        ..creature("Ghastly Gloomhunter", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Bat], 1, 1)
    }
}

/// Stirge — {B} 1/1 Insect Bat, flying, can't block. {1}{B}, pay 1 life,
/// sacrifice it: draw a card.
pub fn stirge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            life_cost: 1,
            sac_cost: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..creature("Stirge", cost(&[b()]), vec![CreatureType::Insect, CreatureType::Bat], 1, 1)
    }
}

/// Batwing Brume — {1}{W/B} instant. Prevent all combat damage this turn if
/// {W} was spent; each player loses 1 life per attacking creature they
/// control if {B} was spent.
pub fn batwing_brume() -> CardDefinition {
    spell(
        "Batwing Brume",
        cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::White, at_least: 1 },
                then: Box::new(Effect::PreventAllCombatDamageThisTurn),
                else_: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Black, at_least: 1 },
                then: Box::new(Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::count(Selector::EachPermanent(
                            R::Creature.and(R::IsAttacking).and(R::ControlledByYou),
                        )),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

// ── Thantis, the Warweaver ───────────────────────────────────────────────────

/// Dragonlair Spider — {2}{R}{R}{G}{G} 5/6 Spider, reach. Whenever an
/// opponent casts a spell, create a 1/1 green Insect.
pub fn dragonlair_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: make(1, token("Insect", 1, 1, Color::Green, vec![CreatureType::Insect])),
        }],
        ..creature(
            "Dragonlair Spider",
            cost(&[generic(2), r(), r(), g(), g()]),
            vec![CreatureType::Spider],
            5,
            6,
        )
    }
}

/// Frenzied Saddlebrute — {4}{R} 5/4 Orc Warrior, haste. All creatures can
/// attack your opponents and their planeswalkers as though they had haste.
pub fn frenzied_saddlebrute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![static_ab(
            "All creatures can attack your opponents and planeswalkers your opponents control as though those creatures had haste.",
            StaticEffect::CreaturesAttackYourOpponentsAsThoughHaste,
        )],
        ..creature("Frenzied Saddlebrute", cost(&[generic(4), r()]), vec![CreatureType::Orc, CreatureType::Warrior], 5, 4)
    }
}

/// Winds of Qal Sisma — {1}{G} instant. Prevent all combat damage this turn;
/// ferocious: only the damage creatures your opponents control would deal.
pub fn winds_of_qal_sisma() -> CardDefinition {
    spell(
        "Winds of Qal Sisma",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::FerociousActive { who: PlayerRef::You },
            then: Box::new(Effect::PreventAllCombatDamageByMatchingThisTurn {
                filter: R::Creature.and(R::ControlledByOpponent),
            }),
            else_: Box::new(Effect::PreventAllCombatDamageThisTurn),
        },
    )
}

// ── Ognis, the Dragon's Lash ─────────────────────────────────────────────────

/// Ognis, the Dragon's Lash — {1}{B/R}{R}{R/G} 3/3 Lizard Warrior, haste.
/// Whenever a creature you control with haste attacks, create a tapped
/// Treasure.
pub fn ognis_the_dragons_lash() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasKeyword(Keyword::Haste),
            }),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
                Effect::Tap { what: Selector::LastCreatedToken },
            ]),
        }],
        ..legend(
            "Ognis, the Dragon's Lash",
            cost(&[generic(1), hybrid(Color::Black, Color::Red), r(), hybrid(Color::Red, Color::Green)]),
            vec![CreatureType::Lizard, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Kalain, Reclusive Painter — {B}{R} 1/2 Human Elf Bard. When it enters,
/// create a Treasure. Other creatures you control enter with an additional
/// +1/+1 counter for each mana from a Treasure spent to cast them.
pub fn kalain_reclusive_painter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(crabomination_base::tokens::treasure_token()),
        })],
        static_abilities: vec![static_ab(
            "Other creatures you control enter with an additional +1/+1 counter on them for each mana from a Treasure spent to cast them.",
            StaticEffect::OtherCreaturesEnterWithCountersPerTreasureMana { kind: CounterType::PlusOnePlusOne },
        )],
        ..legend(
            "Kalain, Reclusive Painter",
            cost(&[b(), r()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Bard],
            1,
            2,
        )
    }
}

/// The Balrog, Durin's Bane — {5}{B}{R} 7/5 Avatar Demon, haste. Costs {1}
/// less for each permanent sacrificed this turn; can't be blocked except by
/// legendary creatures; when it dies, destroy target artifact or creature an
/// opponent controls.
pub fn the_balrog_durins_bane() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Haste,
            Keyword::CantBeBlockedExceptBy(Box::new(R::HasSupertype(Supertype::Legendary))),
        ],
        static_abilities: vec![static_ab(
            "This spell costs {1} less to cast for each permanent sacrificed this turn.",
            StaticEffect::SelfCostReducedByValue { amount: Value::PermanentsSacrificedThisTurn(PlayerRef::EachPlayer) },
        )],
        triggered_abilities: vec![on_dies(Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByOpponent)),
        })],
        ..legend(
            "The Balrog, Durin's Bane",
            cost(&[generic(5), b(), r()]),
            vec![CreatureType::Avatar, CreatureType::Demon],
            7,
            5,
        )
    }
}

// ── Heliod, the Radiant Dawn ─────────────────────────────────────────────────

/// Even the Score — {X}{U}{U}{U} instant. Costs {U}{U}{U} less if an opponent
/// has drawn four or more cards this turn. Draw X cards.
pub fn even_the_score() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "This spell costs {U}{U}{U} less to cast if an opponent has drawn four or more cards this turn.",
            StaticEffect::SelfColoredCostReducedIf {
                condition: Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::EachOpponent, n: 4 },
                less: cost(&[u(), u(), u()]),
            },
        )],
        ..spell(
            "Even the Score",
            cost(&[x(), u(), u(), u()]),
            CardType::Instant,
            Effect::Draw { who: Selector::You, amount: Value::XFromCost },
        )
    }
}

/// Fascination — {X}{U}{U} sorcery. Choose one: each player draws X cards; or
/// each player mills X cards.
pub fn fascination() -> CardDefinition {
    spell(
        "Fascination",
        cost(&[x(), u(), u()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::XFromCost },
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::XFromCost },
        ]),
    )
}

/// Game Plan — {5}{U} sorcery, assist (not modelled — INCOMPLETE_CARDS). Each
/// player shuffles their hand and graveyard into their library, then draws
/// seven cards. Exile Game Plan.
pub fn game_plan() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Game Plan",
            cost(&[generic(5), u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
                Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
            ]),
        )
    }
}

// ── Mister Negative ──────────────────────────────────────────────────────────

/// Mister Negative — {5}{W}{B} 5/5 Human Villain, vigilance, lifelink. When
/// it enters, you may exchange life totals with target opponent; if you lost
/// life this way, draw that many cards.
pub fn mister_negative() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exchange life totals with target opponent?".into(),
            body: Box::new(Effect::WithX {
                x: Value::Max(
                    Box::new(Value::Diff(
                        Box::new(Value::LifeOf(PlayerRef::You)),
                        Box::new(Value::LifeOf(PlayerRef::Target(0))),
                    )),
                    Box::new(Value::ZERO),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::ExchangeLifeTotals { a: Selector::You, b: target_filtered(R::OpponentPlayer) },
                    Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                ])),
            }),
        })],
        ..legend("Mister Negative", cost(&[generic(5), w(), b()]), vec![CreatureType::Human, CreatureType::Villain], 5, 5)
    }
}

/// Pact Weapon — {3}{B} Equipment. While attached to a creature, you don't
/// lose the game for having 0 or less life. Whenever equipped creature
/// attacks, draw a card and reveal it; the creature gets +X/+X until end of
/// turn and you lose X life (X = its mana value). Equip — discard a card.
pub fn pact_weapon() -> CardDefinition {
    CardDefinition {
        name: "Pact Weapon",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        static_abilities: vec![static_ab(
            "As long as this Equipment is attached to a creature, you don't lose the game for having 0 or less life.",
            StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::AttachedToCreature },
                inner: Box::new(StaticEffect::ControllerDoesntLoseForZeroLife),
            },
        )],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                // X is the top card's mana value, read before the draw moves it.
                effect: Effect::WithX {
                    x: Value::ManaValueOf(Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE })),
                    body: Box::new(Effect::Seq(vec![
                        draw(1),
                        Effect::PumpPT {
                            what: Selector::This,
                            power: Value::XFromCost,
                            toughness: Value::XFromCost,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::LoseLife { who: Selector::You, amount: Value::XFromCost },
                    ])),
                },
            }],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Attach { what: Selector::This, to: target_filtered(mine()) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tainted Sigil — {1}{W}{B} artifact. {T}, sacrifice it: gain life equal to
/// the total life lost by all players this turn.
pub fn tainted_sigil() -> CardDefinition {
    CardDefinition {
        name: "Tainted Sigil",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::TotalLifeLostThisTurn(PlayerRef::EachPlayer),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Baba Lysaga, Night Witch ─────────────────────────────────────────────────

/// Baba Lysaga, Night Witch — {1}{B}{G} 3/3 Human Warlock. {T}, sacrifice up
/// to three permanents: if there were three or more card types among them,
/// each opponent loses 3 life, you gain 3 life and draw three cards.
pub fn baba_lysaga_night_witch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_any_number_filter: Some(R::Permanent),
            sac_any_number_max: Some(3),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::CardTypesAmongCostSacrificed, Value::Const(3)),
                then: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    draw(3),
                ])),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..legend(
            "Baba Lysaga, Night Witch",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Erinis, Gloom Stalker — {2}{G} 3/3 Halfling Ranger, deathtouch, choose a
/// Background. Whenever it attacks, return target land card from your
/// graveyard to the battlefield.
pub fn erinis_gloom_stalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::ChooseABackground],
        triggered_abilities: vec![on_attack(Effect::Move {
            what: target_filtered(R::Land.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        ..legend(
            "Erinis, Gloom Stalker",
            cost(&[generic(2), g()]),
            vec![CreatureType::Halfling, CreatureType::Ranger],
            3,
            3,
        )
    }
}

/// Aspect of Mongoose — {1}{G} Aura. Enchanted creature has shroud. When this
/// is put into a graveyard from the battlefield, return it to its owner's
/// hand.
pub fn aspect_of_mongoose() -> CardDefinition {
    CardDefinition {
        name: "Aspect of Mongoose",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..Default::default()
    }
}

// ── Kros, Defense Contractor ─────────────────────────────────────────────────

/// Sheltering Ancient — {1}{G} 5/5 Treefolk, trample. Cumulative upkeep — put
/// a +1/+1 counter on a creature an opponent controls.
pub fn sheltering_ancient() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Trample,
            Keyword::CumulativeUpkeep(CumulativeUpkeepCost::PutCounterOnOpponentCreature(CounterType::PlusOnePlusOne)),
        ],
        ..creature("Sheltering Ancient", cost(&[generic(1), g()]), vec![CreatureType::Treefolk], 5, 5)
    }
}

/// Tolarian Contempt — {3}{U}{U} enchantment. When it enters, put a rejection
/// counter on each creature your opponents control. At the beginning of your
/// end step, for each opponent, choose up to one target creature they control
/// with a rejection counter; its owner puts it on the top or bottom of their
/// library.
pub fn tolarian_contempt() -> CardDefinition {
    CardDefinition {
        name: "Tolarian Contempt",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                kind: CounterType::Rejection,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 5,
                        min_targets: 0,
                        filter: R::Creature.and(R::ControlledByOpponent).and(R::WithCounter(CounterType::Rejection)),
                        effect: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::OwnerChoice },
                        }),
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Regna's Sanction — {3}{W} sorcery. For each player, choose friend or foe.
/// Each friend puts a +1/+1 counter on each creature they control; each foe
/// chooses one untapped creature they control, then taps the rest.
pub fn regnas_sanction() -> CardDefinition {
    spell(
        "Regna's Sanction",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::FriendOrFoe {
            friend: Box::new(Effect::AddCounter {
                what: Selector::EachPermanent(mine()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            foe: Box::new(Effect::ChooseOneAmong {
                what: Selector::EachPermanent(mine().and(R::Untapped)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::Noop),
                other: Box::new(Effect::Tap { what: Selector::SeparatedPile { chosen: false } }),
            }),
        },
    )
}
