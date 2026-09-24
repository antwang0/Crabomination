//! Commander: the cards the **Lorehold Legacies** precon (C21, Osgir, the
//! Reconstructor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_lorehold.rs`.
//!
//! Residuals (each also on its card):
//! - **Archaeomancer's Map** — "that player controls more lands than you" is
//!   read as *an* opponent controlling more.
//! - **Key to the City** — "up to one target creature" always takes a target.
//! - **Laelia, the Blade Reforged** — only her own attack's library exile
//!   counts (no event announces one), and an exile from your battlefield
//!   counts beside your graveyard.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{ManaCost, cost, generic, r, w};
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

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn dies(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource), effect }
}

/// "Whenever an artifact [creature] you control enters" — the entering
/// permanent is the trigger source.
fn your_artifact_enters(filter: R) -> EventSpec {
    EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter })
}

fn golem(keyword: Keyword) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        keywords: vec![keyword],
        ..Default::default()
    })
}

/// Osgir, the Reconstructor — vigilance; {1}, sacrifice an artifact: target
/// creature you control gets +2/+0; {X}, {T}, exile an artifact card with
/// mana value X from your graveyard: two token copies of it (sorcery speed).
pub fn osgir_the_reconstructor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_filter: Some((R::Artifact, 1)),
                sac_other_may_be_source: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[crate::mana::x()]),
                tap_cost: true,
                sorcery_speed: true,
                exile_other_filter: Some((R::Artifact.and(R::ManaValueExactlyXFromCost), 1)),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    source: Selector::CostExiledCards,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Osgir, the Reconstructor",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Giant, CreatureType::Artificer],
            4,
            4,
        )
    }
}

/// Alibou, Ancient Witness — other artifact creatures you control have haste;
/// whenever one or more artifact creatures you control attack, X damage to
/// any target and scry X, X the tapped artifacts you control.
pub fn alibou_ancient_witness() -> CardDefinition {
    let x = || Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::Tapped).and(R::ControlledByYou))));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::Creature),
                })
            },
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Any), amount: x() },
                Effect::Scry { who: PlayerRef::You, amount: x() },
            ]),
        }],
        ..artifact_creature(
            "Alibou, Ancient Witness",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Golem],
            4,
            5,
        )
    }
}

/// Archaeomancer's Map — enters: up to two basic Plains to hand; an
/// opponent's land entering while an opponent has more lands than you puts a
/// land from your hand onto the battlefield (an approximation: *an*
/// opponent, not necessarily that land's controller).
pub fn archaeomancers_map() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::HasLandType(LandType::Plains)),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                        Predicate::OpponentControlsMoreLandsThanYou,
                    ]),
                ),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Land,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            },
        ],
        ..artifact("Archaeomancer's Map", cost(&[generic(2), w()]))
    }
}

/// Audacious Reshapers — {T}, sacrifice an artifact: reveal until an
/// artifact, put it onto the battlefield, the rest on the bottom; it deals
/// you damage equal to the cards revealed.
pub fn audacious_reshapers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::RevealUntilOneToBattlefieldRestBottom { filter: R::Artifact, damage_controller: true },
            ..Default::default()
        }],
        ..creature(
            "Audacious Reshapers",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Battlemage's Bracers — equipped creature has haste; its non-mana
/// activated abilities can be copied for {1}. Equip {2}.
pub fn battlemages_bracers() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Haste],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::AbilityActivated, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "Pay {1} to copy that ability?".into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::CopyActivatedAbilityMayChooseTargets),
                    else_: None,
                },
            }],
            ..Default::default()
        }),
        ..artifact("Battlemage's Bracers", cost(&[generic(2), r()]))
    }
}

/// Bronze Guardian — double strike, ward {2}; other artifacts you control
/// have ward {2}; power is the number of artifacts you control.
pub fn bronze_guardian() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Ward(WardCost::generic(2))],
        dynamic_pt: Some(DynamicPt::ArtifactsControlledPower { base_p: 0, base_t: 5 }),
        static_abilities: vec![StaticAbility {
            description: "Other artifacts you control have ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Ward(WardCost::generic(2)),
            },
        }],
        ..artifact_creature("Bronze Guardian", cost(&[generic(4), w()]), vec![CreatureType::Golem], 0, 5)
    }
}

/// Combustible Gearhulk — first strike; entering, target opponent may let you
/// draw three; if not, you mill three and it deals that player their total
/// mana value.
pub fn combustible_gearhulk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![etb(Effect::PlayersMayAccept {
            who: PlayerRef::Target(0),
            description: "Let the Gearhulk's controller draw three cards?".into(),
            on_accept: Box::new(Effect::Noop),
            if_any: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
            otherwise: Box::new(Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::TotalManaValueOf(Box::new(Selector::LastMoved)),
                },
            ])),
        })],
        ..artifact_creature(
            "Combustible Gearhulk",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Construct],
            6,
            6,
        )
    }
}

/// Digsite Engineer — casting an artifact spell, you may pay {2} for a 0/0
/// Construct that gets +1/+1 for each artifact you control.
pub fn digsite_engineer() -> CardDefinition {
    let construct = TokenDefinition {
        name: "Construct".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token gets +1/+1 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Artifact,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact)),
            effect: Effect::MayPay {
                description: "Pay {2} to create a Construct?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(construct),
                }),
                else_: None,
            },
        }],
        ..creature(
            "Digsite Engineer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Dwarf, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Dispeller's Capsule — {2}{W}, {T}, sacrifice it: destroy target artifact
/// or enchantment.
pub fn dispellers_capsule() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..artifact("Dispeller's Capsule", cost(&[w()]))
    }
}

/// Key to the City — {T}, discard a card: target creature can't be blocked
/// this turn ("up to one" always takes a target); untapping, you may pay {2}
/// to draw.
pub fn key_to_the_city() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2} to draw a card?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(draw(1)),
                else_: None,
            },
        }],
        ..artifact("Key to the City", cost(&[generic(2)]))
    }
}

/// Laelia, the Blade Reforged — haste; attacking exiles your top card, which
/// you may play this turn; cards of yours put into exile grow her (her own
/// attack's exile, and exiles from your graveyard or battlefield).
pub fn laelia_the_blade_reforged() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            // The attack's own exile is the one "put into exile from your
            // library" the engine announces no event for, so its counter
            // rides the same trigger; a graveyard exile has its own event.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                ]),
            },
            TriggeredAbility {
                event: EventSpec {
                    once_per_batch: true,
                    ..EventSpec::new(EventKind::CardExiledFromPlayOrGraveyard, EventScope::YourControl)
                },
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Laelia, the Blade Reforged",
            cost(&[generic(2), r()]),
            vec![CreatureType::Spirit, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Losheel, Clockwork Scholar — combat damage to your attacking artifact
/// creatures is prevented; artifact creatures of yours entering draw a card,
/// once a turn.
pub fn losheel_clockwork_scholar() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to attacking artifact creatures \
                          you control.",
            effect: StaticEffect::PreventAllCombatDamageToMatching {
                filter: R::Artifact.and(R::Creature).and(R::IsAttacking).and(R::ControlledByYou),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..your_artifact_enters(R::Artifact.and(R::Creature))
            },
            effect: draw(1),
        }],
        ..creature(
            "Losheel, Clockwork Scholar",
            cost(&[generic(2), w()]),
            vec![CreatureType::Elephant, CreatureType::Artificer],
            2,
            4,
        )
    }
}

/// Monologue Tax — an opponent casting their second spell each turn makes
/// you a Treasure.
pub fn monologue_tax() -> CardDefinition {
    CardDefinition {
        name: "Monologue Tax",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::Const(2) },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        }],
        ..Default::default()
    }
}

/// Quicksmith Genius — an artifact of yours entering lets you rummage.
pub fn quicksmith_genius() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_artifact_enters(R::Artifact),
            effect: Effect::MayDo {
                description: "Discard a card, then draw a card.".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    draw(1),
                ])),
            },
        }],
        ..creature(
            "Quicksmith Genius",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            2,
        )
    }
}

/// Ruin Grinder — menace; dying, each player may discard their hand and draw
/// seven; mountaincycling {2}.
pub fn ruin_grinder() -> CardDefinition {
    let accepter = || Selector::Player(PlayerRef::Target(0));
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain)],
        triggered_abilities: vec![dies(Effect::PlayersMayAccept {
            who: PlayerRef::EachPlayer,
            description: "Discard your hand and draw seven cards?".into(),
            // Each accepter is bound to slot 0 while its `on_accept` runs.
            on_accept: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: accepter(),
                    amount: Value::HandSizeOf(PlayerRef::Target(0)),
                    random: false,
                },
                Effect::Draw { who: accepter(), amount: Value::Const(7) },
            ])),
            if_any: Box::new(Effect::Noop),
            otherwise: Box::new(Effect::Noop),
        })],
        ..artifact_creature("Ruin Grinder", cost(&[generic(5), r()]), vec![CreatureType::Construct], 7, 4)
    }
}

/// Triplicate Titan — flying, vigilance, trample; dying, three 3/3 Golems
/// with one keyword each.
pub fn triplicate_titan() -> CardDefinition {
    let one = |kw| Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: golem(kw) };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        triggered_abilities: vec![dies(Effect::Seq(vec![
            one(Keyword::Flying),
            one(Keyword::Vigilance),
            one(Keyword::Trample),
        ]))],
        ..artifact_creature("Triplicate Titan", cost(&[generic(9)]), vec![CreatureType::Golem], 9, 9)
    }
}

/// Wake the Past — every artifact card in your graveyard returns, with haste
/// this turn.
pub fn wake_the_past() -> CardDefinition {
    CardDefinition {
        name: "Wake the Past",
        cost: cost(&[generic(5), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ReturnAllMatchingFromGraveyardToBattlefield {
                who: PlayerRef::You,
                filter: R::Artifact,
                sacrifice_eot: false,
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}
