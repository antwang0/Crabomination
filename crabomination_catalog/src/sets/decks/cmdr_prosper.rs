//! Commander: the cards the **Planar Portal** precon (AFC, Prosper,
//! Tome-Bound) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_prosper.rs`.
//!
//! Residuals (each also on its card):
//! - **Karazikar, the Eye Tyrant** — the creature it taps and goads is the
//!   engine's pick (the attacked player's strongest), not a target.
//! - **Hellish Rebuke** — modelled as your watcher for the turn, so the
//!   sacrifice-and-lose-life trigger is yours rather than the permanent
//!   controller's (same outcome).
//! - **Share the Spoils** — each player's pile of linked cards becomes
//!   playable at their upkeep; a land played from it doesn't refill it.
//! - **Danse Macabre** — your sacrifice is made after the others'.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, mint_treasures, on_attack, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r};
use crate::sets::{etb_scry_one, tap_add_colorless};
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

/// "Exile the top `n` cards of your library. You may play them [until]."
fn impulse(n: i32, duration: MayPlayDuration) -> Effect {
    Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::Const(n),
        duration,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost: true,
        uncast_penalty: None,
    }
}

fn zombie() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

fn if_matches(what: Selector, filter: R, then: Effect) -> Effect {
    Effect::If {
        cond: Predicate::EntityMatches { what, filter },
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    }
}

/// Prosper, Tome-Bound — an impulse at your end step lasting through your
/// next turn; a Treasure whenever you play a card from exile.
pub fn prosper_tome_bound() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: impulse(1, MayPlayDuration::EndOfControllersNextTurn),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellFromExile),
                effect: mint_treasures(1),
            },
            // "Play" a land from exile: a land drop (not one an effect put
            // onto the battlefield).
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::EnteredFromExileThisTurn },
                ),
                effect: mint_treasures(1),
            },
        ],
        ..legend(
            "Prosper, Tome-Bound",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Tiefling, CreatureType::Warlock],
            1,
            4,
        )
    }
}

/// Bag of Devouring — exiles what you sacrifice; sacrifice for a card; roll a
/// d10 to return that many of its exiled cards.
pub fn bag_of_devouring() -> CardDefinition {
    let fodder = || R::Artifact.or(R::Creature);
    CardDefinition {
        name: "Bag of Devouring",
        cost: cost(&[b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken.and(fodder()).and(R::OtherThanSource),
                },
            ),
            effect: if_matches(
                Selector::TriggerSource,
                R::InGraveyard,
                Effect::ExileWithSource { what: Selector::TriggerSource },
            ),
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_other_filter: Some((fodder(), 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::RollDie {
                    sides: 10,
                    count: Value::ONE,
                    modifier: Value::Const(0),
                    reroll_at_most: 0,
                    ignore_lowest: 0,
                    results: vec![(
                        1,
                        10,
                        Effect::MoveChosen {
                            from: Selector::CardExiledWithSource,
                            filter: None,
                            count: Value::LastDieRoll,
                            up_to: true,
                            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        },
                    )],
                    on_doubles: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Chaos Channeler — Wild Magic Surge: a d20 on attack for one to three
/// cards off the top, playable this turn.
pub fn chaos_channeler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::RollDie {
            sides: 20,
            count: Value::ONE,
            modifier: Value::Const(0),
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![
                (1, 9, impulse(1, MayPlayDuration::EndOfThisTurn)),
                (10, 19, impulse(2, MayPlayDuration::EndOfThisTurn)),
                (20, 20, impulse(3, MayPlayDuration::EndOfThisTurn)),
            ],
            on_doubles: None,
        })],
        ..creature("Chaos Channeler", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Shaman], 4, 3)
    }
}

/// Danse Macabre — each player sacrifices a nontoken creature; a d20 plus
/// your sacrifice's toughness returns one, or up to two, of them under your
/// control. Residual: yours is sacrificed after the others'.
pub fn danse_macabre() -> CardDefinition {
    let fodder = || R::Creature.and(R::NotToken);
    let back = |count: i32, up_to: bool| Effect::MoveChosen {
        from: Selector::SacrificedThisResolution { filter: R::Creature },
        filter: None,
        count: Value::Const(count),
        up_to,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    spell(
        "Danse Macabre",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Sacrifice { who: Selector::Player(PlayerRef::EachOpponent), count: Value::ONE, filter: fodder() },
            Effect::SacrificeAndRemember { who: PlayerRef::You, filter: fodder() },
            Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::SacrificedToughness,
                reroll_at_most: 0,
                ignore_lowest: 0,
                results: vec![(1, 14, back(1, false)), (15, u8::MAX, back(2, true))],
                on_doubles: None,
            },
        ]),
    )
}

/// Dark-Dweller Oracle — {1}, sacrifice a creature: impulse one this turn.
pub fn dark_dweller_oracle() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: impulse(1, MayPlayDuration::EndOfThisTurn),
            ..Default::default()
        }],
        ..creature("Dark-Dweller Oracle", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Shaman], 2, 2)
    }
}

/// Dead Man's Chest — when the enchanted opponent's creature dies, exile
/// cards equal to its power off its owner's library; cast them with any
/// mana while they stay exiled.
pub fn dead_mans_chest() -> CardDefinition {
    CardDefinition {
        name: "Dead Man's Chest",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                count: Value::PowerOf(Box::new(Selector::TriggerSource)),
                duration: MayPlayDuration::WhileExiled,
                pay_any_color: true,
                max_mana_value: None,
                pay_own_cost: false,
                uncast_penalty: None,
            },
                Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
        }],
        ..Default::default()
    }
}

/// Death Tyrant — menace; a Zombie whenever an attacker of yours or a
/// blocker of an opponent's dies; {5}{B}: back from the graveyard tapped.
pub fn death_tyrant() -> CardDefinition {
    let died = |scope, filter| TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter }),
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zombie()) },
    };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![
            died(EventScope::YourControl, R::IsAttacking),
            died(EventScope::OpponentControl, R::IsBlocking),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Death Tyrant", cost(&[generic(4), b()]), vec![CreatureType::Beholder, CreatureType::Skeleton], 4, 6)
    }
}

/// Fevered Suspicion — each opponent exiles to a nonland card; cast any of
/// them free. Rebound.
pub fn fevered_suspicion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Fevered Suspicion",
            cost(&[generic(6), b(), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::ForEachOpponent {
                    body: Box::new(Effect::ExileTopUntilNonland { who: PlayerRef::Triggerer }),
                },
                Effect::CastAnyOrderWithoutPaying {
                    what: Selector::ExiledThisResolution { filter: R::Nonland },
                    source_zone: Zone::Exile,
                    filter: None,
                    cap: None,
                    total_mana_value: None,
                },
            ]),
        )
    }
}

/// Fiend of the Shadows — flying; a player it hits exiles a card from hand
/// that you may play while it stays exiled; sacrifice a Human to regenerate.
pub fn fiend_of_the_shadows() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileFromHand { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::WhileExiled,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Human), 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Fiend of the Shadows", cost(&[generic(3), b(), b()]), vec![CreatureType::Vampire, CreatureType::Wizard], 3, 3)
    }
}

/// Fiendlash — +2/+0 and reach; damage dealt to the creature makes it hit a
/// player or planeswalker for its power.
pub fn fiendlash() -> CardDefinition {
    CardDefinition {
        name: "Fiendlash",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2), r()]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            keywords: vec![Keyword::Reach],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Hellish Rebuke — this turn, an opponent's permanent that damages you is
/// sacrificed and its controller loses 2 life. Residual: the trigger is
/// yours, not the permanent controller's.
pub fn hellish_rebuke() -> CardDefinition {
    let it = || Selector::TriggerSource;
    spell(
        "Hellish Rebuke",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::DelayUntil {
            kind: DelayedTriggerKind::OpponentPermanentDamagesYouThisTurn,
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(it()))),
                    amount: Value::Const(2),
                },
                Effect::SacrificePermanent { what: it() },
            ])),
        },
    )
}

/// Hurl Through Hell — exile a creature; you may cast it with any mana
/// through the end of your next turn.
pub fn hurl_through_hell() -> CardDefinition {
    spell(
        "Hurl Through Hell",
        cost(&[generic(2), b(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            Effect::GrantMayPlay {
                what: Selector::Target(0),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ]),
    )
}

/// Karazikar, the Eye Tyrant — attacking a player taps and goads one of
/// their creatures; an opponent attacking another opponent draws both of you
/// a card for 1 life. Residual: the goaded creature is the engine's pick.
pub fn karazikar_the_eye_tyrant() -> CardDefinition {
    let pick = || Selector::TakeGreatestPower {
        inner: Box::new(Selector::ControlledBy { who: PlayerRef::Triggerer, filter: R::Creature }),
        count: Box::new(Value::ONE),
    };
    let both = |e: fn(Selector) -> Effect| {
        vec![e(Selector::You), e(Selector::Player(PlayerRef::Target(0)))]
    };
    let mut body = both(|who| Effect::Draw { who, amount: Value::ONE });
    body.extend(both(|who| Effect::LoseLife { who, amount: Value::ONE }));
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YouAttackedPlayer),
                effect: Effect::Seq(vec![Effect::Tap { what: pick() }, Effect::Goad { what: pick() }]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::OpponentOfYoursAttacked)
                    .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::Target(0) }),
                effect: Effect::Seq(body),
            },
        ],
        ..legend("Karazikar, the Eye Tyrant", cost(&[generic(3), b(), r()]), vec![CreatureType::Beholder], 5, 5)
    }
}

/// Lorcan, Warlock Collector — pay life to steal creature cards hitting an
/// opponent's graveyard (as Warlocks); your Warlocks are exiled instead of
/// dying.
pub fn lorcan_warlock_collector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: if_matches(
                Selector::TriggerSource,
                R::InGraveyard,
                Effect::MayPayLife {
                    description: "Pay life equal to its mana value to put it onto the battlefield?".into(),
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::TriggerSource,
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::AddCreatureTypes {
                            what: Selector::LastMoved,
                            creature_types: vec![CreatureType::Warlock],
                            duration: Duration::Permanent,
                        },
                    ])),
                    else_: None,
                },
            ),
        }],
        static_abilities: vec![static_ab(
            "If a Warlock you control would die, exile it instead.",
            StaticEffect::DiesToExileInstead {
                filter: R::HasCreatureType(CreatureType::Warlock).and(R::ControlledByYou),
            },
        )],
        ..legend(
            "Lorcan, Warlock Collector",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Devil],
            6,
            6,
        )
    }
}

/// Orazca Relic — ascend; {C}; with the city's blessing, sacrifice for 3 life
/// and a card.
pub fn orazca_relic() -> CardDefinition {
    CardDefinition {
        name: "Orazca Relic",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Ascend { who: PlayerRef::You })],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                condition: Some(Predicate::Any(vec![
                    Predicate::HasCityBlessing { who: PlayerRef::You },
                    Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::ControlledByYou),
                        n: Value::Const(10),
                    },
                ])),
                effect: Effect::Seq(vec![
                    Effect::Ascend { who: PlayerRef::You },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Piper of the Swarm — Rats have menace; make Rats; three Rats steal a
/// creature.
pub fn piper_of_the_swarm() -> CardDefinition {
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![static_ab(
            "Rats you control have menace.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Rat).and(R::ControlledByYou)),
                keyword: Keyword::Menace,
            },
        )],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                tap_cost: true,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(rat) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b(), b()]),
                tap_cost: true,
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Rat), 3)),
                effect: Effect::GainControl {
                    what: target_filtered(R::Creature),
                    to: None,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
        ],
        ..creature("Piper of the Swarm", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Warlock], 1, 3)
    }
}

/// Reckless Endeavor — roll two d12, one result to damage every creature, the
/// other in Treasures.
pub fn reckless_endeavor() -> CardDefinition {
    spell(
        "Reckless Endeavor",
        cost(&[generic(5), r(), r()]),
        CardType::Sorcery,
        Effect::RollTwoDiceAssign {
            sides: 12,
            first: Box::new(Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::LastDieRoll,
            }),
            second: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::LastDieRoll,
                definition: Arc::new(crate::game::effects::treasure_token()),
            }),
        },
    )
}

/// Share the Spoils — exile the top card of each library on entry and when
/// an opponent loses; during each player's turn they may play one of those
/// with any mana, then exile their top card to replace it. Residual: the pile
/// opens at their upkeep, and a land played from it doesn't refill it.
pub fn share_the_spoils() -> CardDefinition {
    let refill = || Effect::Seq(vec![
        Effect::EndMayPlayOnCardsExiledWithSource,
        Effect::ExileTopOfLibrary {
            who: Selector::Player(PlayerRef::Triggerer),
            amount: Value::ONE,
            link_to_source: true,
            face_down: false,
        },
    ]);
    let seed = || Effect::ExileTopOfLibrary {
        who: Selector::Player(PlayerRef::EachPlayer),
        amount: Value::ONE,
        link_to_source: true,
        face_down: false,
    };
    CardDefinition {
        name: "Share the Spoils",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(seed()),
            TriggeredAbility { event: EventSpec::new(EventKind::PlayerLeftGame, EventScope::AnyPlayer), effect: seed() },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: Effect::AsPlayer {
                    who: PlayerRef::ActivePlayer,
                    body: Box::new(Effect::GrantMayPlay {
                        what: Selector::CardExiledWithSource,
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: true,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(Predicate::TriggerCardExiledWithSource),
                effect: refill(),
            },
        ],
        ..Default::default()
    }
}

/// You Find Some Prisoners — destroy an artifact, or exile an opponent's top
/// three and take one to play with any mana through your next turn.
pub fn you_find_some_prisoners() -> CardDefinition {
    spell(
        "You Find Some Prisoners",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::Const(3),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::ChooseOneAmong {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::GrantMayPlay {
                        what: Selector::SeparatedPile { chosen: true },
                        duration: MayPlayDuration::EndOfControllersNextTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: true,
                    }),
                    other: Box::new(Effect::Noop),
                },
            ]),
        ]),
    )
}

/// Zhalfirin Void — scry 1 on entry; {C}.
pub fn zhalfirin_void() -> CardDefinition {
    CardDefinition {
        name: "Zhalfirin Void",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb_scry_one()],
        activated_abilities: vec![tap_add_colorless()],
        ..Default::default()
    }
}
