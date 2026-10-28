//! Commander: the 2026-10-21 (`01Jywzky`) pair-deck closers. Tests in
//! `tests/recent_b/cmdr_01jy_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, ConditionalEquipBonus, EnchantmentSubtype, EquipBonus, LoyaltyAbility, PlaneswalkerSubtype, Supertype, Zone, CounterType, LevelBand, SplitCard, SplitHalf, TokenDefinition, CardType, CreatureType, CumulativeUpkeepCost, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, on_attack, partner_with_search, pump_target, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LibraryPosition, LookPick, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, colorless, cost, g, generic, hybrid, r, u, w};
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
                    // CR 701.13 — "put into your graveyard" isn't a mill.
                    Effect::RevealTopMatchingToGraveyard { who: PlayerRef::You, count: Value::Const(3), filter: R::Any },
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
            event: EventSpec::new(EventKind::NthCardDrawnThisTurn(2), EventScope::YourControl),
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

/// Tromp the Domains — {5}{G} sorcery. Domain — until end of turn, creatures
/// you control gain trample and get +1/+1 for each basic land type among lands
/// you control.
pub fn tromp_the_domains() -> CardDefinition {
    let domain = || Value::DomainCount(PlayerRef::You);
    spell(
        "Tromp the Domains",
        cost(&[generic(5), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT { what: yours(), power: domain(), toughness: domain(), duration: Duration::EndOfTurn },
            grant_each(yours(), Keyword::Trample),
        ]),
    )
}

/// Dancing Sword — {1}{W} Equipment, equip {1}. Equipped creature gets +2/+1.
/// When equipped creature dies, you may have this become a 2/1 Construct
/// artifact creature with flying and ward {1}; if you do, it isn't an Equipment.
pub fn dancing_sword() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Have Dancing Sword become a 2/1 flying Construct?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::BecomeCreature {
                            what: Selector::This,
                            power: Value::Const(2),
                            toughness: Value::Const(1),
                            creature_types: vec![CreatureType::Construct],
                            keywords: vec![Keyword::Flying, Keyword::Ward(crate::card::WardCost::generic(1))],
                            duration: Duration::Permanent,
                        },
                        Effect::SetArtifactSubtypes { what: Selector::This, subtypes: vec![] },
                    ])),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..spell("Dancing Sword", cost(&[generic(1), w()]), CardType::Artifact, Effect::Noop)
    }
}

/// Racers' Ring — land, enters tapped. {T}: add {R} or {G}. {2}{R}{G}, {T},
/// sacrifice it: draw a card.
pub fn racers_ring() -> CardDefinition {
    let mut land = crate::sets::dual_land_untyped("Racers' Ring", Color::Red, Color::Green, vec![]);
    land.static_abilities.push(crate::sets::enters_tapped());
    land.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(2), r(), g()]),
        tap_cost: true,
        sac_cost: true,
        effect: draw(1),
        ..Default::default()
    });
    land
}

/// Riveteers Decoy — {1}{G} 3/1 Human Warrior; must be blocked if able.
/// Blitz {3}{G}.
pub fn riveteers_decoy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustBeBlocked],
        alternative_cost: Some(crate::effect::shortcut::blitz(cost(&[generic(3), g()]))),
        ..creature("Riveteers Decoy", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 1)
    }
}

/// Stunning Strike — {2}{U} Aura with flash. Entering, it taps enchanted
/// creature and removes it from combat. While that creature isn't legendary,
/// it doesn't untap during its controller's untap step.
pub fn stunning_strike() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: enchanted() },
            Effect::RemoveFromCombat { what: enchanted() },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "As long as enchanted creature isn't legendary, it doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: enchanted(),
                condition: Some(Predicate::EntityMatches {
                    what: enchanted(),
                    filter: R::HasSupertype(Supertype::Legendary).negate(),
                }),
            },
        }],
        ..spell(
            "Stunning Strike",
            cost(&[generic(2), u()]),
            CardType::Enchantment,
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        )
    }
}

/// Check for Traps — {1}{B} sorcery. Target opponent reveals their hand; you
/// exile a nonland card from it. An instant or flash card exiled this way costs
/// them 1 life; otherwise you lose 1 life.
pub fn check_for_traps() -> CardDefinition {
    let quick = R::HasCardType(CardType::Instant).or(R::HasKeyword(Keyword::Flash));
    spell(
        "Check for Traps",
        cost(&[generic(1), b()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Seq(vec![
                Effect::RevealHand { who: PlayerRef::Target(0) },
                Effect::MoveChosen {
                    from: Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Hand, filter: R::Nonland },
                    filter: None,
                    count: Value::ONE,
                    up_to: false,
                    to: ZoneDest::Exile,
                },
                Effect::If {
                    cond: Predicate::EntityMatchesAny { what: Selector::LastMoved, filter: quick },
                    then: Box::new(Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE }),
                    else_: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::ONE }),
                },
            ])),
        },
    )
}

/// Sliversmith — {2} 1/1 artifact creature Spellshaper. {1}, {T}, discard a
/// card: create a 1/1 colorless Sliver artifact creature token named Metallic
/// Sliver.
pub fn sliversmith() -> CardDefinition {
    let sliver = TokenDefinition {
        name: "Metallic Sliver".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sliver], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(sliver) },
            ..Default::default()
        }],
        ..creature("Sliversmith", cost(&[generic(2)]), vec![CreatureType::Spellshaper], 1, 1)
    }
}

/// Tezzeret the Schemer — {2}{U}{B} planeswalker, loyalty 5. +1: an Etherium
/// Cell token ("{T}, sacrifice it: one mana of any color"). −2: target creature
/// gets +X/−X until end of turn, X the artifacts you control. −7: an emblem —
/// at the beginning of combat on your turn, target artifact you control
/// becomes an artifact creature with base power and toughness 5/5.
pub fn tezzeret_the_schemer() -> CardDefinition {
    let cell = TokenDefinition {
        name: "Etherium Cell".into(),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![crabomination_base::tokens::treasure_token().activated_abilities[0].clone()],
        ..Default::default()
    };
    let artifacts = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Artifact);
    let loyalty = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { loyalty_cost, effect, ..Default::default() };
    CardDefinition {
        name: "Tezzeret the Schemer",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tezzeret], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            loyalty(1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(cell) }),
            loyalty(
                -2,
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: artifacts(),
                    toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(artifacts())),
                    duration: Duration::EndOfTurn,
                },
            ),
            loyalty(
                -7,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Tezzeret the Schemer".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(
                            EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                            EventScope::YourControl,
                        ),
                        effect: Effect::BecomeCreature {
                            what: target_filtered(R::Artifact.and(R::ControlledByYou)),
                            power: Value::Const(5),
                            toughness: Value::Const(5),
                            creature_types: vec![],
                            keywords: vec![],
                            duration: Duration::Permanent,
                        },
                    }],
                    statics: vec![],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Prepare // Fight — {1}{W} instant: untap target creature; it gets +2/+2 and
/// lifelink until end of turn. // {3}{G} sorcery, aftermath: target creature
/// you control fights target creature an opponent controls.
pub fn prepare_fight() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Fight {
                    attacker: target_filtered(R::Creature.and(R::ControlledByYou)),
                    defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                },
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Prepare // Fight",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                pump_each(Selector::Target(0), 2, 2),
                grant_each(Selector::Target(0), Keyword::Lifelink),
            ]),
        )
    }
}

/// Clay Champion — {X}{4} 2/2 artifact creature Construct. It enters with three
/// +1/+1 counters for each {G}{G} spent to cast it; entering, up to two other
/// target creatures you control each get a +1/+1 counter for each {W}{W} spent.
pub fn clay_champion() -> CardDefinition {
    let pairs = |c: Color| Value::DivDown(Box::new(Value::SourceCastManaOfColor(c)), 2);
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::Times(Box::new(pairs(Color::Green)), Box::new(Value::Const(3))),
        )),
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: pairs(Color::White),
            }),
        })],
        ..creature("Clay Champion", cost(&[crate::mana::x(), generic(4)]), vec![CreatureType::Construct], 2, 2)
    }
}

fn loyalty(loyalty_cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost, effect, ..Default::default() }
}

/// Vigorbloom Vanguard // Seed Suture — {1}{G/W} 2/2 Troll Druid that enters
/// prepared; each creature you control with a +1/+1 counter has vigilance.
/// Seed Suture ({G/W} sorcery): a +1/+1 counter on target creature, gain 1.
pub fn vigorbloom_vanguard() -> CardDefinition {
    let gw = || hybrid(Color::Green, Color::White);
    CardDefinition {
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        prepare_spell: Some(Arc::new(CardDefinition {
            name: "Seed Suture",
            cost: cost(&[gw()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        })),
        static_abilities: vec![StaticAbility {
            description: "Each creature you control with a +1/+1 counter on it has vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                keyword: Keyword::Vigilance,
            },
        }],
        ..creature("Vigorbloom Vanguard", cost(&[generic(1), gw()]), vec![CreatureType::Troll, CreatureType::Druid], 2, 2)
    }
}

/// Ajani, Valiant Protector — {4}{G}{W} planeswalker, loyalty 4. +2: two +1/+1
/// counters on up to one target creature. +1: reveal until a creature card, put
/// it into your hand, the rest on the bottom at random. −11: X +1/+1 counters on
/// target creature (X your life total); it gains trample until end of turn.
pub fn ajani_valiant_protector() -> CardDefinition {
    CardDefinition {
        name: "Ajani, Valiant Protector",
        cost: cost(&[generic(4), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Ajani], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            loyalty(
                2,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                },
            ),
            loyalty(
                1,
                Effect::RevealUntilNFound {
                    who: PlayerRef::You,
                    find: R::Creature,
                    count: Value::ONE,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ),
            loyalty(
                -11,
                Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(R::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::LifeOf(PlayerRef::You),
                    },
                    grant_each(Selector::Target(0), Keyword::Trample),
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Dovin Baan — {2}{W}{U} planeswalker, loyalty 3. +1: until your next turn,
/// up to one target creature gets −3/−0 and its activated abilities can't be
/// activated. −1: gain 2 life, draw a card. −7: an emblem — your opponents
/// can't untap more than two permanents during their untap steps.
pub fn dovin_baan() -> CardDefinition {
    let lock = Effect::Seq(vec![
        Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::UntilNextTurn,
        },
        Effect::GrantKeyword {
            what: Selector::Target(0),
            keyword: Keyword::CantActivateAbilities,
            duration: Duration::UntilNextTurn,
        },
    ]);
    CardDefinition {
        name: "Dovin Baan",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Dovin], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            loyalty(1, Effect::ApplyToTargets { max_targets: 1, min_targets: 0, filter: R::Creature, effect: Box::new(lock) }),
            loyalty(-1, Effect::Seq(vec![Effect::GainLife { who: Selector::You, amount: Value::Const(2) }, draw(1)])),
            loyalty(
                -7,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Dovin Baan".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "Your opponents can't untap more than two permanents during their untap steps.",
                        effect: StaticEffect::OpponentsMaxUntapsPerStep(2),
                    }],
                },
            ),
        ],
        ..Default::default()
    }
}

/// Predator Dragon — {3}{R}{R}{R} 4/4 flying, haste Dragon; devour 2.
pub fn predator_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Devour(2)],
        ..creature("Predator Dragon", cost(&[generic(3), r(), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Druidic Ritual — {2}{G} sorcery. You may mill three cards. Then return up
/// to one creature card and up to one land card from your graveyard to your
/// hand.
pub fn druidic_ritual() -> CardDefinition {
    spell(
        "Druidic Ritual",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::MayDo {
                description: "Mill three cards?".into(),
                body: Box::new(Effect::Mill { who: Selector::You, amount: Value::Const(3) }),
            },
            Effect::ReturnGraveyardCardsToHand { filter: R::Creature, max: Value::ONE },
            Effect::ReturnGraveyardCardsToHand { filter: R::Land, max: Value::ONE },
        ]),
    )
}

/// Nevermaker — {3}{U} 2/3 flying Elemental, evoke {3}{U}. When it leaves the
/// battlefield, put target nonland permanent on top of its owner's library.
pub fn nevermaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(crate::effect::shortcut::evoke(cost(&[generic(3), u()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            },
        }],
        ..creature("Nevermaker", cost(&[generic(3), u()]), vec![CreatureType::Elemental], 2, 3)
    }
}

/// Brazen Buccaneers — {3}{R} 2/2 Human Pirate with haste. It explores as it
/// enters.
pub fn brazen_buccaneers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb(crate::effect::shortcut::explore())],
        ..creature("Brazen Buccaneers", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Academy Loremaster — {U}{U} 2/3 Human Wizard. At the beginning of each
/// player's draw step, that player may draw an additional card; if they do,
/// spells they cast this turn cost {2} more.
pub fn academy_loremaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Draw), EventScope::AnyPlayer),
            effect: Effect::AnyPlayerMayAccept {
                who: PlayerRef::ActivePlayer,
                prompt: "Draw an additional card? Spells you cast this turn cost {2} more.".into(),
                accepted: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::Player(PlayerRef::AcceptingPlayer), amount: Value::ONE },
                    Effect::SpellsCostMoreThisTurn { who: PlayerRef::AcceptingPlayer, amount: 2 },
                ])),
                otherwise: Box::new(Effect::Noop),
            },
        }],
        ..creature("Academy Loremaster", cost(&[u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Voracious Dragon — {3}{R}{R} 4/4 flying Dragon, devour 1. Entering, it
/// deals damage to any target equal to twice the number of Goblins it devoured.
pub fn voracious_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Devour(1)],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: crate::effect::shortcut::target_any(),
            amount: Value::Times(
                Box::new(Value::DevouredMatching(R::HasCreatureType(CreatureType::Goblin))),
                Box::new(Value::Const(2)),
            ),
        })],
        ..creature("Voracious Dragon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Jace's Sentinel — {1}{U} 1/3 Merfolk Warrior. As long as you control a Jace
/// planeswalker, it gets +1/+0 and can't be blocked.
pub fn jaces_sentinel() -> CardDefinition {
    let jace = || {
        Predicate::SelectorExists(Selector::EachPermanent(
            R::Planeswalker.and(R::HasPlaneswalkerType(PlaneswalkerSubtype::Jace)).and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "As long as you control a Jace planeswalker, this creature gets +1/+0.",
                effect: StaticEffect::WhileCondition {
                    condition: jace(),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: Selector::This, power: 1, toughness: 0 }),
                },
            },
            StaticAbility {
                description: "As long as you control a Jace planeswalker, this creature can't be blocked.",
                effect: StaticEffect::WhileCondition {
                    condition: jace(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Unblockable }),
                },
            },
        ],
        ..creature("Jace's Sentinel", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Warrior], 1, 3)
    }
}

/// Thanos, the Mad Titan — {R}{W}{B} legendary 4/4 Eternal Villain with
/// deathtouch and lifelink. Power-up — {C}{W}{U}{B}{R}{G}: two +1/+1 counters on
/// Thanos; choose odd or even; destroy each other creature whose mana value is
/// of the chosen quality (zero is even).
pub fn thanos_the_mad_titan() -> CardDefinition {
    let snap = |odd: bool| Effect::Destroy {
        what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource).and(R::ManaValueParity { odd })),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[colorless(1), w(), u(), b(), r(), g()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::ChooseMode(vec![snap(true), snap(false)]),
            ]),
            ..Default::default()
        }],
        ..creature("Thanos, the Mad Titan", cost(&[r(), w(), b()]), vec![CreatureType::Eternal, CreatureType::Villain], 4, 4)
    }
}

/// Tombstone Stairwell — {2}{B}{B} World Enchantment, cumulative upkeep {1}{B}.
/// At the beginning of each upkeep, each player creates a 2/2 black Zombie
/// token with haste named Tombspawn for each creature card in their graveyard.
/// At the beginning of each end step and when it leaves the battlefield,
/// destroy all tokens created with it; they can't be regenerated.
pub fn tombstone_stairwell() -> CardDefinition {
    let tombspawn = TokenDefinition {
        name: "Tombspawn".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Black],
        keywords: vec![Keyword::Haste],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    let raze = || Effect::DestroyNoRegen { what: Selector::TokensCreatedBySource };
    CardDefinition {
        supertypes: vec![Supertype::World],
        keywords: vec![Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[generic(1), b()])))],
        triggered_abilities: vec![
            TriggeredAbility {
                // CR 603.4 — "if Tombstone Stairwell is on the battlefield",
                // re-checked as the trigger resolves.
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::AnyPlayer)
                    .with_filter(Predicate::SourceOnBattlefield),
                effect: Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::count(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        }),
                        definition: Arc::new(tombspawn),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::End), EventScope::AnyPlayer),
                effect: raze(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: raze(),
            },
        ],
        ..spell("Tombstone Stairwell", cost(&[generic(2), b(), b()]), CardType::Enchantment, Effect::Noop)
    }
}

fn kicked_cast() -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::CastSpellWasKicked)
}

fn kicked_etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::SpellWasKicked),
        effect,
    }
}

/// Hallar, the Firefletcher — {1}{R}{G} legendary 3/3 Elf Archer, trample.
/// Whenever you cast a kicked spell, a +1/+1 counter on Hallar, then it deals
/// damage equal to its +1/+1 counters to each opponent.
pub fn hallar_the_firefletcher() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: kicked_cast(),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                },
            ]),
        }],
        ..creature("Hallar, the Firefletcher", cost(&[generic(1), r(), g()]), vec![CreatureType::Elf, CreatureType::Archer], 3, 3)
    }
}

/// Ghitu Chronicler — {1}{R} 1/3 Human Wizard, kicker {3}{R}. Entering kicked,
/// return target instant or sorcery card from your graveyard to your hand.
pub fn ghitu_chronicler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), r()]))],
        triggered_abilities: vec![kicked_etb(Effect::Move {
            what: target_filtered(
                R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Ghitu Chronicler", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 3)
    }
}

/// Murasa Sproutling — {2}{G} 3/3 Plant Elemental, kicker {1}{G}. Entering
/// kicked, return target card with a kicker ability from your graveyard to your
/// hand.
pub fn murasa_sproutling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), g()]))],
        triggered_abilities: vec![kicked_etb(Effect::Move {
            what: target_filtered(R::HasKickerAbility.and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Murasa Sproutling", cost(&[generic(2), g()]), vec![CreatureType::Plant, CreatureType::Elemental], 3, 3)
    }
}

/// Vine Gecko — {1}{G} 2/2 Elemental Lizard. The first kicked spell you cast
/// each turn costs {1} less. Whenever you cast a kicked spell, a +1/+1 counter
/// on it.
pub fn vine_gecko() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The first kicked spell you cast each turn costs {1} less to cast.",
            effect: StaticEffect::FirstKickedSpellEachTurnCostsLess { amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: kicked_cast(),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Vine Gecko", cost(&[generic(1), g()]), vec![CreatureType::Elemental, CreatureType::Lizard], 2, 2)
    }
}

/// Reclaim the Wastes — {G} sorcery, kicker {3}. Search for a basic land card
/// (two if kicked), reveal it, put it into your hand, then shuffle.
pub fn reclaim_the_wastes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        ..spell(
            "Reclaim the Wastes",
            cost(&[g()]),
            CardType::Sorcery,
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Sum(vec![Value::ONE, Value::OneIf(Box::new(Predicate::SpellWasKicked))]),
            },
        )
    }
}

/// Alabaster Dragon — {4}{W}{W} 4/4 flying Dragon. When it dies, shuffle it
/// into its owner's library.
pub fn alabaster_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ShuffleSelfIntoLibrary,
        }],
        ..creature("Alabaster Dragon", cost(&[generic(4), w(), w()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Dragon Hunter — {W} 2/1 Human Warrior. Protection from Dragons; it can
/// block Dragons as though it had reach.
pub fn dragon_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::ProtectionFromCreatureType(CreatureType::Dragon),
            Keyword::CanBlockCreatureTypeAsThoughReach(CreatureType::Dragon),
        ],
        ..creature("Dragon Hunter", cost(&[w()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 1)
    }
}

/// Shieldhide Dragon — {5}{W} 3/3 flying, lifelink Dragon; megamorph
/// {5}{W}{W}. Turned face up, a +1/+1 counter on each other Dragon you control.
pub fn shieldhide_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Megamorph(cost(&[generic(5), w(), w()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Dragon))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Shieldhide Dragon", cost(&[generic(5), w()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Grand Master of Flowers — {2}{W}{W} Bahamut planeswalker, loyalty 3. With
/// seven or more loyalty he's a 7/7 Dragon God creature with flying and
/// indestructible. +1: target creature without first strike, double strike or
/// vigilance can't attack or block until your next turn. +1: put a Monk of the
/// Open Hand from your library and/or graveyard into your hand.
pub fn grand_master_of_flowers() -> CardDefinition {
    let big = || {
        Predicate::ValueAtLeast(
            Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Loyalty },
            Value::Const(7),
        )
    };
    let monk = || R::HasName("Monk of the Open Hand".into());
    let in_graveyard = || Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: monk() };
    let lock = |k: Keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword: k, duration: Duration::UntilNextTurn };
    CardDefinition {
        name: "Grand Master of Flowers",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Bahamut], ..Default::default() },
        base_loyalty: 3,
        static_abilities: vec![
            StaticAbility {
                description: "As long as Grand Master of Flowers has seven or more loyalty counters on him, he's a 7/7 Dragon God creature with flying and indestructible.",
                effect: StaticEffect::SelfIsCreatureIf {
                    condition: big(),
                    creature_types: vec![CreatureType::Dragon, CreatureType::God],
                },
            },
            StaticAbility {
                description: "…7/7…",
                effect: StaticEffect::SetBasePtIf { condition: big(), power: 7, toughness: 7 },
            },
            StaticAbility {
                description: "…with flying…",
                effect: StaticEffect::SelfHasKeywordIf { keyword: Keyword::Flying, condition: big() },
            },
            StaticAbility {
                description: "…and indestructible.",
                effect: StaticEffect::SelfHasKeywordIf { keyword: Keyword::Indestructible, condition: big() },
            },
        ],
        loyalty_abilities: vec![
            loyalty(
                1,
                Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(
                            R::Creature
                                .and(R::HasKeyword(Keyword::FirstStrike).negate())
                                .and(R::HasKeyword(Keyword::DoubleStrike).negate())
                                .and(R::HasKeyword(Keyword::Vigilance).negate()),
                        ),
                        keyword: Keyword::CantAttack,
                        duration: Duration::UntilNextTurn,
                    },
                    lock(Keyword::CantBlock),
                ]),
            ),
            loyalty(
                1,
                Effect::If {
                    cond: Predicate::SelectorExists(in_graveyard()),
                    then: Box::new(Effect::Move { what: Selector::one_of(in_graveyard()), to: ZoneDest::Hand(PlayerRef::You) }),
                    else_: Box::new(Effect::SearchUpToN {
                        who: PlayerRef::You,
                        filter: monk(),
                        to: ZoneDest::Hand(PlayerRef::You),
                        count: Value::ONE,
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

fn self_grant(mana: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, effect: grant_each(Selector::This, keyword), ..Default::default() }
}

fn self_pump(mana: ManaCost, p: i32, t: i32) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, effect: pump_each(Selector::This, p, t), ..Default::default() }
}

/// Endling — {2}{B}{B} 3/3 Zombie Shapeshifter. {B}: menace; {B}: deathtouch;
/// {B}: undying (each until end of turn). {1}: +1/−1 or −1/+1 until end of turn.
pub fn endling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            self_grant(cost(&[b()]), Keyword::Menace),
            self_grant(cost(&[b()]), Keyword::Deathtouch),
            self_grant(cost(&[b()]), Keyword::Undying),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::ChooseMode(vec![pump_each(Selector::This, 1, -1), pump_each(Selector::This, -1, 1)]),
                ..Default::default()
            },
        ],
        ..creature("Endling", cost(&[generic(2), b(), b()]), vec![CreatureType::Zombie, CreatureType::Shapeshifter], 3, 3)
    }
}

/// Hateflayer — {5}{R}{R} 5/5 Elemental with wither. {2}{R}, {Q}: it deals
/// damage equal to its power to any target.
pub fn hateflayer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            untap_self_cost: true,
            effect: Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..creature("Hateflayer", cost(&[generic(5), r(), r()]), vec![CreatureType::Elemental], 5, 5)
    }
}

/// Minion of Leshrac — {4}{B}{B}{B} 5/5 Demon Minion, protection from black.
/// At the beginning of your upkeep, it deals 5 damage to you unless you
/// sacrifice another creature; if it deals damage this way, tap it. {T}:
/// destroy target creature or land.
pub fn minion_of_leshrac() -> CardDefinition {
    let other = || R::Creature.and(R::ControlledByYou).and(R::OtherThanSource);
    let bite = || {
        Effect::Seq(vec![
            Effect::DealDamage { to: Selector::You, amount: Value::Const(5) },
            // "If this creature deals damage to you this way, tap it."
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::DamageDealtThisResolution, Value::ONE),
                then: Box::new(Effect::Tap { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(other())),
                then: Box::new(Effect::MayDoElse {
                    description: "Sacrifice another creature (else Minion of Leshrac deals 5 damage to you)?".into(),
                    body: Box::new(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: other() }),
                    else_: Box::new(bite()),
                }),
                else_: Box::new(bite()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.or(R::Land)) },
            ..Default::default()
        }],
        ..creature("Minion of Leshrac", cost(&[generic(4), b(), b(), b()]), vec![CreatureType::Demon, CreatureType::Minion], 5, 5)
    }
}

/// Torchling — {3}{R}{R} 3/3 Shapeshifter. {R}: untap it. {R}: target creature
/// blocks it this turn if able. {R}: change the target of target spell that
/// targets only it. {1}: +1/−1 until end of turn. {1}: −1/+1 until end of turn.
pub fn torchling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::MustBlockSource { what: target_filtered(R::Creature), chooser: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::ChangeSpellTarget {
                    what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsOnlySource)),
                },
                ..Default::default()
            },
            self_pump(cost(&[generic(1)]), 1, -1),
            self_pump(cost(&[generic(1)]), -1, 1),
        ],
        ..creature("Torchling", cost(&[generic(3), r(), r()]), vec![CreatureType::Shapeshifter], 3, 3)
    }
}

// ── Brigid, Clachan's Heart's Kithkin ──────────────────────────────────────

fn kithkin() -> R {
    R::HasCreatureType(CreatureType::Kithkin)
}

fn gw() -> crate::mana::ManaSymbol {
    hybrid(Color::Green, Color::White)
}

fn kithkin_token(types: Vec<CreatureType>, colors: Vec<Color>) -> TokenDefinition {
    TokenDefinition {
        name: "Kithkin".into(),
        power: 1,
        toughness: 1,
        colors,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn gw_kithkin() -> Arc<TokenDefinition> {
    Arc::new(kithkin_token(vec![CreatureType::Kithkin], vec![Color::Green, Color::White]))
}

/// Seedcradle Witch — {G/W} 1/1 Elf Shaman. {2}{G}{W}: target creature gets
/// +3/+3 until end of turn; untap it.
pub fn seedcradle_witch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), w()]),
            effect: Effect::Seq(vec![
                pump_each(target_filtered(R::Creature), 3, 3),
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ]),
            ..Default::default()
        }],
        ..creature("Seedcradle Witch", cost(&[gw()]), vec![CreatureType::Elf, CreatureType::Shaman], 1, 1)
    }
}

/// Champion of the Clachan — {3}{W} 4/5 Kithkin Knight with flash. As an
/// additional cost, behold a Kithkin and exile it. Other Kithkin you control
/// get +1/+1. When it leaves the battlefield, the exiled card returns to its
/// owner's hand.
pub fn champion_of_the_clachan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::BeholdAndExile { filter: kithkin() }],
        static_abilities: vec![StaticAbility {
            description: "Other Kithkin you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(kithkin()).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::CardExiledWithSource,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::CardExiledWithSource))),
            },
        }],
        ..creature("Champion of the Clachan", cost(&[generic(3), w()]), vec![CreatureType::Kithkin, CreatureType::Knight], 4, 5)
    }
}

/// Thoughtweft Lieutenant — {G}{W} 2/2 Kithkin Soldier. Whenever it or another
/// Kithkin you control enters, target creature you control gets +1/+1 and
/// trample until end of turn.
pub fn thoughtweft_lieutenant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(kithkin()) }),
            effect: Effect::Seq(vec![
                pump_each(target_filtered(R::Creature.and(R::ControlledByYou)), 1, 1),
                grant_each(Selector::Target(0), Keyword::Trample),
            ]),
        }],
        ..creature("Thoughtweft Lieutenant", cost(&[g(), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 2, 2)
    }
}

/// Figure of Fable — {G/W} 1/1 Kithkin. {G/W}: it becomes a Kithkin Scout with
/// base 2/3. {1}{G/W}{G/W}: if a Scout, a Kithkin Soldier with base 4/5.
/// {3}{G/W}{G/W}{G/W}: if a Soldier, a Kithkin Avatar with base 7/8 and
/// protection from each of your opponents.
pub fn figure_of_fable() -> CardDefinition {
    let become_ = |kind: CreatureType, p: i32, t: i32| {
        vec![
            Effect::BecomeCreatureType {
                what: Selector::This,
                creature_types: vec![CreatureType::Kithkin, kind],
                duration: Duration::Permanent,
            },
            Effect::SetBasePT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::Permanent },
        ]
    };
    let is = |kind: CreatureType| Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(kind) };
    let mut avatar = become_(CreatureType::Avatar, 7, 8);
    avatar.push(Effect::GrantKeyword {
        what: Selector::This,
        keyword: Keyword::ProtectionFromEachOpponent,
        duration: Duration::Permanent,
    });
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[gw()]),
                effect: Effect::Seq(become_(CreatureType::Scout, 2, 3)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), gw(), gw()]),
                effect: Effect::If {
                    cond: is(CreatureType::Scout),
                    then: Box::new(Effect::Seq(become_(CreatureType::Soldier, 4, 5))),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), gw(), gw(), gw()]),
                effect: Effect::If {
                    cond: is(CreatureType::Soldier),
                    then: Box::new(Effect::Seq(avatar)),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..creature("Figure of Fable", cost(&[gw()]), vec![CreatureType::Kithkin], 1, 1)
    }
}

/// Wary Farmer — {1}{G/W}{G/W} 3/3 Kithkin Citizen. At the beginning of your
/// end step, if another creature entered under your control this turn,
/// surveil 1.
pub fn wary_farmer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreatureEnteredThisTurnMatching {
                    who: PlayerRef::You,
                    filter: R::OtherThanSource,
                }),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature("Wary Farmer", cost(&[generic(1), gw(), gw()]), vec![CreatureType::Kithkin, CreatureType::Citizen], 3, 3)
    }
}

/// Brigid's Command — {1}{G}{W} Kindred sorcery — Kithkin. Choose two: a token
/// copy of target Kithkin you control; target player creates a 1/1 green and
/// white Kithkin; target creature you control gets +3/+3; target creature you
/// control fights target creature an opponent controls.
pub fn brigids_command() -> CardDefinition {
    let mine = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin], ..Default::default() },
        ..spell(
            "Brigid's Command",
            cost(&[generic(1), g(), w()]),
            CardType::Sorcery,
            Effect::ChooseModesCast {
                min: 2,
                max: 2,
                allow_repeats: false,
                modes: vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: target_filtered(mine().and(kithkin())),
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
                    Effect::TargetPlayerThen {
                        filter: R::Player,
                        then: Box::new(Effect::CreateToken { who: PlayerRef::Target(0), count: Value::ONE, definition: gw_kithkin() }),
                    },
                    pump_each(target_filtered(mine()), 3, 3),
                    Effect::Fight {
                        attacker: target_filtered(mine()),
                        defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
                    },
                ],
            },
        )
    }
}

/// Goldmeadow Nomad — {W} 1/2 Kithkin Scout. {W}, exile it from your
/// graveyard: create a 1/1 green and white Kithkin token. Sorcery speed.
pub fn goldmeadow_nomad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: gw_kithkin() },
            ..Default::default()
        }],
        ..creature("Goldmeadow Nomad", cost(&[w()]), vec![CreatureType::Kithkin, CreatureType::Scout], 1, 2)
    }
}

/// Thoughtweft Imbuer — {3}{W} 0/5 Kithkin Advisor. Whenever a creature you
/// control attacks alone, it gets +X/+X until end of turn, X the Kithkin you
/// control.
pub fn thoughtweft_imbuer() -> CardDefinition {
    let x = || Value::PermanentCountControlledByMatching(PlayerRef::You, R::Creature.and(kithkin()));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
            effect: Effect::PumpPT { what: Selector::TriggerSource, power: x(), toughness: x(), duration: Duration::EndOfTurn },
        }],
        ..creature("Thoughtweft Imbuer", cost(&[generic(3), w()]), vec![CreatureType::Kithkin, CreatureType::Advisor], 0, 5)
    }
}

/// Kithkin Harbinger — {2}{W} 1/3 Kithkin Wizard. Entering, you may search for
/// a Kithkin card, shuffle, and put it on top.
pub fn kithkin_harbinger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: kithkin(),
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        })],
        ..creature("Kithkin Harbinger", cost(&[generic(2), w()]), vec![CreatureType::Kithkin, CreatureType::Wizard], 1, 3)
    }
}

/// Crossroads Watcher — {2}{G} 3/3 trample Kithkin Ranger. Whenever another
/// creature you control enters, it gets +1/+0 until end of turn.
pub fn crossroads_watcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::OtherThanSource) },
            ),
            effect: pump_each(Selector::This, 1, 0),
        }],
        ..creature("Crossroads Watcher", cost(&[generic(2), g()]), vec![CreatureType::Kithkin, CreatureType::Ranger], 3, 3)
    }
}

/// Militia's Pride — {1}{W} Kindred enchantment — Kithkin. Whenever a nontoken
/// creature you control attacks, you may pay {W}; if you do, create a 1/1
/// white Kithkin Soldier token tapped and attacking — the player the triggering
/// creature attacks (CR 508.4 leaves the choice to its controller).
pub fn militias_pride() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsToken.negate(),
            }),
            effect: Effect::MayPay {
                description: "Pay {W} for a 1/1 Kithkin Soldier tapped and attacking?".into(),
                mana_cost: cost(&[w()]),
                body: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(kithkin_token(
                        vec![CreatureType::Kithkin, CreatureType::Soldier],
                        vec![Color::White],
                    )),
                    cleanup: Default::default(),
                    defender: None,
                }),
                else_: None,
            },
        }],
        ..spell("Militia's Pride", cost(&[generic(1), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Eclipsed Kithkin — {G/W}{G/W} 2/1 Kithkin Scout. Entering, look at the top
/// four; you may reveal a Kithkin, Forest or Plains card and take it; the rest
/// on the bottom at random.
pub fn eclipsed_kithkin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(kithkin().or(R::HasLandType(LandType::Forest)).or(R::HasLandType(LandType::Plains))),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..creature("Eclipsed Kithkin", cost(&[gw(), gw()]), vec![CreatureType::Kithkin, CreatureType::Scout], 2, 1)
    }
}

/// Aegar, the Freezing Flame — {1}{U}{R} legendary 3/3 Giant Wizard. Whenever
/// a creature or planeswalker an opponent controls is dealt excess damage
/// (CR 120.10 / 120.4a, combat or not), if a Giant, Wizard or spell you
/// controlled dealt damage to it this turn, draw a card.
pub fn aegar_the_freezing_flame() -> CardDefinition {
    // A resolved spell sits in its owner's graveyard by the time the event
    // dispatches, past any "controlled" read: the caster of an instant or
    // sorcery is its owner bar a stolen cast.
    let mine = R::HasCreatureType(CreatureType::Giant)
        .or(R::HasCreatureType(CreatureType::Wizard))
        .and(R::ControlledByYou)
        .or(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::OwnedByYou));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtExcessDamage { noncombat_only: false }, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.or(R::Planeswalker).and(R::DamagedThisTurnByMatching(Box::new(mine))),
                },
            ),
            effect: draw(1),
        }],
        ..creature(
            "Aegar, the Freezing Flame",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Giant, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Giant's Amulet — {U} Equipment, equip {2}. Entering, you may pay {3}{U}
/// for a 4/4 blue Giant Wizard token, then attach this Equipment to it.
/// Equipped creature gets +0/+1 and has hexproof as long as it's untapped.
pub fn giants_amulet() -> CardDefinition {
    let giant = TokenDefinition {
        name: "Giant Wizard".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant, CreatureType::Wizard], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Giant's Amulet",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            toughness: 1,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::Untapped,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Hexproof],
                ..Default::default()
            }],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {3}{U} for a 4/4 Giant Wizard wearing Giant's Amulet?".into(),
            mana_cost: cost(&[generic(3), u()]),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(giant) },
                Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

/// Battle of Frost and Fire — {3}{U}{R} Saga. I: 4 damage to each non-Giant
/// creature and each planeswalker. II: scry 3. III: whenever you cast a spell
/// with mana value 5 or greater this turn, draw two cards, then discard a card.
pub fn battle_of_frost_and_fire() -> CardDefinition {
    let victims = R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Giant)))).or(R::Planeswalker);
    saga(
        "Battle of Frost and Fire",
        cost(&[generic(3), u(), r()]),
        vec![
            (1, Effect::DealDamage { to: Selector::EachPermanent(victims), amount: Value::Const(4) }),
            (2, Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) }),
            (
                3,
                Effect::OnEachSpellCastThisTurn {
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ManaValueAtLeast(5) },
                        then: Box::new(Effect::Seq(vec![
                            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        ])),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ),
        ],
    )
}

/// Invasion of the Giants — {U}{R} Saga. I: scry 2. II: draw a card, then you
/// may reveal a Giant card from your hand; when you do, 2 damage to target
/// opponent or planeswalker. III: the next Giant spell you cast this turn
/// costs {2} less.
pub fn invasion_of_the_giants() -> CardDefinition {
    let giant = || R::HasCreatureType(CreatureType::Giant);
    saga(
        "Invasion of the Giants",
        cost(&[u(), r()]),
        vec![
            (1, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            (
                2,
                Effect::Seq(vec![
                    draw(1),
                    // CR 603.7 — the reflexive trigger targets once the
                    // reveal is made.
                    Effect::IfRevealFromHand {
                        filter: giant(),
                        then: Box::new(Effect::ReflexiveTrigger {
                            body: Box::new(Effect::DealDamage {
                                to: target_filtered(R::OpponentPlayer.or(R::Planeswalker)),
                                amount: Value::Const(2),
                            }),
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            ),
            (3, Effect::GrantNextMatchingSpellDiscountThisTurn { filter: giant(), amount: 2 }),
        ],
    )
}

/// Sharkey, Tyrant of the Shire — {2}{U}{B} legendary 2/4 Avatar Rogue.
/// Opponents' lands can't activate non-mana abilities; Sharkey has them all,
/// payable with mana of any type.
pub fn sharkey_tyrant_of_the_shire() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Activated abilities of lands your opponents control can't be activated unless they're mana abilities.",
                effect: StaticEffect::OpponentsLandNonmanaAbilitiesLocked,
            },
            StaticAbility {
                description: "Sharkey has all activated abilities of lands your opponents control except mana abilities. Mana of any type can be spent to activate Sharkey's abilities.",
                effect: StaticEffect::HasNonmanaActivatedAbilitiesOfOpponentLands,
            },
        ],
        ..creature(
            "Sharkey, Tyrant of the Shire",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Avatar, CreatureType::Rogue],
            2,
            4,
        )
    }
}

/// Fires of Mount Doom — {2}{R} legendary enchantment. Entering, 2 damage to
/// target creature an opponent controls and destroy all Equipment attached to
/// it. {2}{R}: exile the top card; you may play it this turn; when you play a
/// card this way, 2 damage to each player.
pub fn fires_of_mount_doom() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // The Equipment goes first: state-based actions wait for the
            // whole ability (CR 704.3), so the order changes nothing, and the
            // damage can leave the target unreadable as a permanent.
            Effect::ForEach {
                selector: Selector::AttachedToMe(Box::new(Selector::Target(0))),
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                    },
                    then: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                    else_: Box::new(Effect::Noop),
                }),
            },
            Effect::DealDamage { to: target_filtered(R::Creature.and(R::ControlledByOpponent)), amount: Value::Const(2) },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::ONE, link_to_source: false, face_down: false },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                Effect::WhenPlayedThisWayThisTurn {
                    what: Selector::ExiledThisResolution { filter: R::Any },
                    body: Box::new(Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) }),
                },
            ]),
            ..Default::default()
        }],
        ..spell("Fires of Mount Doom", cost(&[generic(2), r()]), CardType::Enchantment, Effect::Noop)
    }
}
