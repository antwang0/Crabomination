//! Commander: the 2026-10-21 (`01Jywzky`) pair-deck closers. Tests in
//! `tests/recent_b/cmdr_01jy_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CounterType, LevelBand, SplitCard, SplitHalf, TokenDefinition, CardType, CreatureType, CumulativeUpkeepCost, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, on_attack, partner_with_search, pump_target, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, LookPick, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
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
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// "{cost}, {T}: this deals 2 damage to target player or planeswalker."
fn ping_player_or_planeswalker(mana: ManaCost, amount: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        effect: Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::Const(amount),
        },
        ..Default::default()
    }
}

/// Burnout — {1}{R} instant. Counter target instant spell if it's blue. Draw a
/// card at the beginning of the next turn's upkeep.
pub fn burnout() -> CardDefinition {
    CardDefinition {
        name: "Burnout",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::HasColor(Color::Blue) },
                then: Box::new(Effect::CounterSpell {
                    what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Instant))),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::DelayUntil { kind: DelayedTriggerKind::NextUpkeep, body: Box::new(draw(1)) },
        ]),
        ..Default::default()
    }
}

/// Mana Short — {2}{U} instant. Tap all lands target player controls and that
/// player loses all unspent mana.
pub fn mana_short() -> CardDefinition {
    CardDefinition {
        name: "Mana Short",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Land } },
            Effect::LoseUnspentMana { who: PlayerRef::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Energy Storm — {1}{W} enchantment, cumulative upkeep {1}. Prevent all damage
/// that would be dealt by instant and sorcery spells. Creatures with flying
/// don't untap during their controllers' untap steps.
pub fn energy_storm() -> CardDefinition {
    CardDefinition {
        name: "Energy Storm",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1)])))],
        static_abilities: vec![
            StaticAbility {
                description: "Prevent all damage that would be dealt by instant and sorcery spells.",
                effect: StaticEffect::PreventAllSpellDamage,
            },
            StaticAbility {
                description: "Creatures with flying don't untap during their controllers' untap steps.",
                effect: StaticEffect::PreventUntapGlobal {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Ambitious Farmhand // Seasoned Cathar — {1}{W} 1/1 Human Peasant. Entering,
/// you may search for a basic Plains to hand. Coven — {1}{W}{W}: transform,
/// only with three creatures of different powers. Back: 3/3 lifelink Knight.
pub fn ambitious_farmhand() -> CardDefinition {
    let cathar = CardDefinition {
        keywords: vec![Keyword::Lifelink],
        ..creature("Seasoned Cathar", ManaCost::default(), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::IsBasicLand.and(R::HasLandType(LandType::Plains)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), w()]),
            condition: Some(Predicate::CovenActive { who: PlayerRef::You }),
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(cathar)),
        ..creature("Ambitious Farmhand", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Peasant], 1, 1)
    }
}

/// Arborea Pegasus — {3}{W} 2/3 flying Pegasus. Entering, target creature gets
/// +1/+1 and gains flying until end of turn.
pub fn arborea_pegasus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Arborea Pegasus", cost(&[generic(3), w()]), vec![CreatureType::Pegasus], 2, 3)
    }
}

/// Daybreak Charger — {1}{W} 3/1 Unicorn. Entering, target creature gets +2/+0
/// until end of turn.
pub fn daybreak_charger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(pump_target(2, 0))],
        ..creature("Daybreak Charger", cost(&[generic(1), w()]), vec![CreatureType::Unicorn], 3, 1)
    }
}

/// Pit Trap — {2} artifact. {2}, {T}, Sacrifice it: destroy target attacking
/// creature without flying. It can't be regenerated.
pub fn pit_trap() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying).negate())),
            },
            ..Default::default()
        }],
        ..spell("Pit Trap", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Aven Fleetwing — {3}{U} 2/2 Bird Soldier with flying and hexproof.
pub fn aven_fleetwing() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Hexproof],
        ..creature("Aven Fleetwing", cost(&[generic(3), u()]), vec![CreatureType::Bird, CreatureType::Soldier], 2, 2)
    }
}

/// Covenant of Minds — {4}{U} sorcery. Reveal the top three cards; target
/// opponent may have you put them into your hand. If they don't, they go to
/// your graveyard and you draw five.
pub fn covenant_of_minds() -> CardDefinition {
    spell(
        "Covenant of Minds",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::AnyPlayerMayAccept {
                who: PlayerRef::Target(0),
                prompt: "Put the three revealed cards into their hand (else they bin them and draw five)?".into(),
                accepted: Box::new(Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    filter: R::Any,
                    distinct_powers: false,
                }),
                otherwise: Box::new(Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                    draw(5),
                ])),
            }),
        },
    )
}

/// Chakram Slinger — {4}{R} 2/4 Human Warrior, partner with Chakram Retriever.
/// {R}, {T}: 2 damage to target player or planeswalker.
pub fn chakram_slinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Chakram Retriever".into())],
        triggered_abilities: vec![partner_with_search("Chakram Retriever")],
        activated_abilities: vec![ping_player_or_planeswalker(cost(&[r()]), 2)],
        ..creature("Chakram Slinger", cost(&[generic(4), r()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 4)
    }
}

/// Proud Mentor — {2}{W} 1/1 Human Warrior, partner with Impetuous Protege.
/// {W}, {T}: tap target creature.
pub fn proud_mentor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Impetuous Protege".into())],
        triggered_abilities: vec![partner_with_search("Impetuous Protege")],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Proud Mentor", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

/// Impetuous Protege — {2}{R} 0/4 Human Warrior, partner with Proud Mentor.
/// Attacking, it gets +X/+0, X the greatest power among tapped creatures your
/// opponents control.
pub fn impetuous_protege() -> CardDefinition {
    let greatest = Value::PowerOf(Box::new(Selector::TakeGreatestPower {
        inner: Box::new(Selector::EachPermanent(R::Creature.and(R::Tapped).and(R::ControlledByOpponent))),
        count: Box::new(Value::ONE),
    }));
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Proud Mentor".into())],
        triggered_abilities: vec![
            partner_with_search("Proud Mentor"),
            on_attack(Effect::PumpPT {
                what: Selector::This,
                power: greatest,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
        ],
        ..creature("Impetuous Protege", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Warrior], 0, 4)
    }
}

/// Inscribed Tablet — {1} artifact. {1}, {T}, Sacrifice it: reveal the top five,
/// a land among them to hand, the rest on the bottom in a random order; with no
/// land taken, draw a card.
pub fn inscribed_tablet() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::Land),
                rest_bottom_random: true,
                then_if_not_picked: Some(Box::new(draw(1))),
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..spell("Inscribed Tablet", cost(&[generic(1)]), CardType::Artifact, Effect::Noop)
    }
}

/// Bloodhaze Wolverine — {1}{R} 2/1 Wolverine. Whenever you draw your second
/// card each turn, it gets +1/+1 and gains first strike until end of turn.
pub fn bloodhaze_wolverine() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Bloodhaze Wolverine", cost(&[generic(1), r()]), vec![CreatureType::Wolverine], 2, 1)
    }
}

/// Direct Current — {1}{R}{R} sorcery, jump-start. 2 damage to any target.
pub fn direct_current() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::JumpStart],
        ..spell(
            "Direct Current",
            cost(&[generic(1), r(), r()]),
            CardType::Sorcery,
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
        )
    }
}

fn yours() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::ControlledByYou))
}

fn pump_each(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant_each(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

/// Cloudchaser Kestrel — {1}{W}{W} 2/2 flying Bird. Entering, destroy target
/// enchantment. {W}: target permanent becomes white until end of turn.
pub fn cloudchaser_kestrel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Destroy { what: target_filtered(R::Enchantment) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::BecomeColor {
                what: target_filtered(R::Permanent),
                colors: vec![Color::White],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature("Cloudchaser Kestrel", cost(&[generic(1), w(), w()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Knight of Cliffhaven — {1}{W} 2/2 Kor Knight leveler. Level up {3}.
/// LEVEL 1-3: 2/3 flying. LEVEL 4+: 4/4 flying, vigilance.
pub fn knight_of_cliffhaven() -> CardDefinition {
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 1, max: Some(3), power: 2, toughness: 3, keywords: vec![Keyword::Flying] },
            LevelBand { min: 4, max: None, power: 4, toughness: 4, keywords: vec![Keyword::Flying, Keyword::Vigilance] },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sorcery_speed: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Knight of Cliffhaven", cost(&[generic(1), w()]), vec![CreatureType::Kor, CreatureType::Knight], 2, 2)
    }
}

/// Tenacity — {3}{W} instant. Creatures you control get +1/+1 and gain
/// lifelink until end of turn. Untap those creatures.
pub fn tenacity() -> CardDefinition {
    spell(
        "Tenacity",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump_each(yours(), 1, 1),
            grant_each(yours(), Keyword::Lifelink),
            Effect::Untap { what: yours(), up_to: None },
        ]),
    )
}

/// Engineered Might — {3}{G}{W} sorcery. Choose one — target creature gets
/// +5/+5 and trample; or creatures you control get +2/+2 and vigilance.
pub fn engineered_might() -> CardDefinition {
    spell(
        "Engineered Might",
        cost(&[generic(3), g(), w()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                pump_each(target_filtered(R::Creature), 5, 5),
                grant_each(Selector::Target(0), Keyword::Trample),
            ]),
            Effect::Seq(vec![pump_each(yours(), 2, 2), grant_each(yours(), Keyword::Vigilance)]),
        ]),
    )
}

/// Flower // Flourish — {G/W} sorcery: search for a basic Forest or Plains to
/// hand. // {4}{G}{W} sorcery: creatures you control get +2/+2 until end of turn.
pub fn flower_flourish() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), g(), w()]),
                card_types: vec![CardType::Sorcery],
                effect: pump_each(yours(), 2, 2),
            },
            fuse: false,
            aftermath: false,
        })),
        ..spell(
            "Flower // Flourish",
            cost(&[hybrid(Color::Green, Color::White)]),
            CardType::Sorcery,
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::HasLandType(LandType::Forest).or(R::HasLandType(LandType::Plains))),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Needle Storm — {2}{G} sorcery. 4 damage to each creature with flying.
pub fn needle_storm() -> CardDefinition {
    spell(
        "Needle Storm",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            amount: Value::Const(4),
        },
    )
}

/// Weathered Runestone — {2} artifact. Nonland permanent cards in graveyards
/// and libraries can't enter the battlefield; players can't cast spells from
/// graveyards or libraries.
pub fn weathered_runestone() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Nonland permanent cards in graveyards and libraries can't enter the battlefield. Players can't cast spells from graveyards or libraries.",
            effect: StaticEffect::NonlandPermanentGraveyardLibraryLockdown,
        }],
        ..spell("Weathered Runestone", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Shadowstorm Vizier — {U}{B} 1/3 flying Human Cleric. Whenever you cycle or
/// discard a card, it gets +1/+1 until end of turn.
pub fn shadowstorm_vizier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: pump_each(Selector::This, 1, 1),
        }],
        ..creature("Shadowstorm Vizier", cost(&[u(), b()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 3)
    }
}

/// Sacred Excavation — {3}{U} sorcery. Return up to two target cards with
/// cycling from your graveyard to your hand.
pub fn sacred_excavation() -> CardDefinition {
    spell(
        "Sacred Excavation",
        cost(&[generic(3), u()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::HasCyclingAbility.and(R::InYourGraveyard),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        },
    )
}

/// Aviation Pioneer — {2}{U} 1/2 Human Artificer. Entering, create a 1/1
/// colorless Thopter artifact creature token with flying.
pub fn aviation_pioneer() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(thopter),
        })],
        ..creature("Aviation Pioneer", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 2)
    }
}

/// Mimic — {2} artifact Treasure. {T}, sacrifice it: one mana of any color.
/// {2}: it becomes a 3/3 Shapeshifter artifact creature until end of turn.
pub fn mimic() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Treasure], ..Default::default() },
        activated_abilities: vec![
            crabomination_base::tokens::treasure_token().activated_abilities[0].clone(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Shapeshifter],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..spell("Mimic", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Rimeshield Frost Giant — {3}{U}{U} 4/5 Giant Warrior, ward {3}.
pub fn rimeshield_frost_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ward(crate::card::WardCost::generic(3))],
        ..creature("Rimeshield Frost Giant", cost(&[generic(3), u(), u()]), vec![CreatureType::Giant, CreatureType::Warrior], 4, 5)
    }
}
