//! Commander: the two-card EDHREC gaps of the 2026-10-12 rescan, ninth file
//! (`cmdr_edhrec8` is the eighth). Tests in `tests/recent_b/cmdr_edhrec9.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost};
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

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn treasure() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(crabomination_base::tokens::treasure_token()),
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Workshop Assistant — {3} 1/2 artifact Construct. Dying returns another
/// artifact card from your graveyard to hand.
pub fn workshop_assistant() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_dies(Effect::Move {
            what: target_filtered(R::Artifact.and(R::InYourGraveyard).and(R::OtherThanSource)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Workshop Assistant", cost(&[generic(3)]), vec![CreatureType::Construct], 1, 2)
    }
}

/// Wizard's Retort — {1}{U}{U} Instant, {1} less with a Wizard. Counter
/// target spell.
pub fn wizards_retort() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_control: vec![(R::HasCreatureType(CreatureType::Wizard), 1)],
        ..spell(
            "Wizard's Retort",
            cost(&[generic(1), u(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        )
    }
}

/// Flood of Recollection — {U}{U} Sorcery. Regrow an instant or sorcery;
/// exile this.
pub fn flood_of_recollection() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Flood of Recollection",
            cost(&[u(), u()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).from_your_graveyard(),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Rancid Rats — {1}{B} 1/1 Zombie Rat, skulk and deathtouch.
pub fn rancid_rats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk, Keyword::Deathtouch],
        ..creature("Rancid Rats", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Rat], 1, 1)
    }
}

/// Stronghold Rats — {2}{B} 2/1 Rat, shadow. Its combat damage to a player
/// makes each player discard a card.
pub fn stronghold_rats() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE, random: false },
        }],
        ..creature("Stronghold Rats", cost(&[generic(2), b()]), vec![CreatureType::Rat], 2, 1)
    }
}

/// Angelic Ascension — {1}{W} Instant. Exile a creature or planeswalker; its
/// controller gets a 4/4 flying Angel.
pub fn angelic_ascension() -> CardDefinition {
    let angel = TokenDefinition { keywords: vec![Keyword::Flying], ..token("Angel", 4, 4, Color::White, vec![CreatureType::Angel]) };
    spell(
        "Angelic Ascension",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::ONE,
                definition: Arc::new(angel),
            },
            Effect::Exile { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        ]),
    )
}

/// Blight Rot — {2}{B} Instant. Four -1/-1 counters on a creature.
pub fn blight_rot() -> CardDefinition {
    spell(
        "Blight Rot",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::Const(4) },
    )
}

/// Kaervek, the Spiteful — {2}{B}{B} 3/2 Human Warlock. Other creatures get
/// -1/-1.
pub fn kaervek_the_spiteful() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Other creatures get -1/-1.",
            StaticEffect::PumpPT { applies_to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)), power: -1, toughness: -1 },
        )],
        ..legend("Kaervek, the Spiteful", cost(&[generic(2), b(), b()]), vec![CreatureType::Human, CreatureType::Warlock], 3, 2)
    }
}

/// Rafiq of the Many — {1}{G}{W}{U} 3/3 Human Knight, exalted. A lone
/// attacker of yours gains double strike.
pub fn rafiq_of_the_many() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            crate::effect::shortcut::exalted(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Rafiq of the Many",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Planar Bridge — {6} legendary artifact. {8}, {T}: tutor a permanent card
/// onto the battlefield.
pub fn planar_bridge() -> CardDefinition {
    CardDefinition {
        name: "Planar Bridge",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Permanent,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Urza's Workshop — Land — Urza's. {T}: {C}; metalcraft — {T}: {C} for each
/// Urza's land you control.
pub fn urzas_workshop() -> CardDefinition {
    CardDefinition {
        name: "Urza's Workshop",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Urza], ..Default::default() },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::CountOf(Box::new(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasLandType(LandType::Urza),
                    }))),
                },
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    n: Value::Const(3),
                }),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Foreboding Fruit — {2}{B} Sorcery. A target player draws two and loses 2;
/// adamant (three black) makes you a Food.
pub fn foreboding_fruit() -> CardDefinition {
    spell(
        "Foreboding Fruit",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
                Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
                Effect::If {
                    cond: Predicate::ManaSpentOfColorAtLeast { color: Color::Black, at_least: 3 },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(crabomination_base::tokens::food_token()),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
    )
}

/// Prying Blade — {1} Equipment, equip {2}. +1/+0; its creature's combat
/// damage to a player makes a Treasure.
pub fn prying_blade() -> CardDefinition {
    equipment(
        "Prying Blade",
        cost(&[generic(1)]),
        cost(&[generic(2)]),
        EquipBonus {
            power: 1,
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: treasure(),
            }],
            ..Default::default()
        },
    )
}

/// Lost Jitte — {1} legendary Equipment, equip {1}. Its creature's combat
/// damage charges it; a charge counter untaps a land, stops a blocker, or
/// grows the creature.
pub fn lost_jitte() -> CardDefinition {
    let charge = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
    };
    let spend = |effect| ActivatedAbility { remove_counter_cost: Some((CounterType::Charge, 1)), effect, ..Default::default() };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            spend(Effect::Untap { what: target_filtered(R::Land), up_to: None }),
            spend(Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn }),
            spend(Effect::AddCounter {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        ],
        ..equipment(
            "Lost Jitte",
            cost(&[generic(1)]),
            cost(&[generic(1)]),
            EquipBonus {
                triggers_on_equipment: true,
                triggered_abilities: vec![
                    charge(EventKind::DealsCombatDamageToPlayer),
                    charge(EventKind::DealsCombatDamageToCreature),
                ],
                ..Default::default()
            },
        )
    }
}

/// Compy Swarm — {1}{B}{G} 2/2 Dinosaur. Your end step after a death makes a
/// tapped copy of it.
pub fn compy_swarm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: true,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: None,
                remove_keywords: vec![],
            },
        }],
        ..creature("Compy Swarm", cost(&[generic(1), b(), g()]), vec![CreatureType::Dinosaur], 2, 2)
    }
}

/// Vraska, Golgari Queen — {2}{B}{G} planeswalker, loyalty 4. +2 may
/// sacrifice another permanent to gain 1 and draw; −3 destroys a cheap
/// nonland permanent; −9 an emblem whose combat damage ends games.
pub fn vraska_golgari_queen() -> CardDefinition {
    CardDefinition {
        name: "Vraska, Golgari Queen",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Vraska], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::MaySacrifice {
                    description: "Sacrifice another permanent to gain 1 life and draw a card?".into(),
                    filter: R::Permanent.and(R::OtherThanSource),
                    count: Value::ONE,
                    then: Box::new(Effect::Seq(vec![Effect::GainLife { who: Selector::You, amount: Value::ONE }, draw(1)])),
                    else_: None,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy { what: target_filtered(R::Permanent.and(R::Nonland).and(R::ManaValueAtMost(3))) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Vraska, Golgari Queen".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                        effect: Effect::LoseGame { who: PlayerRef::Target(0) },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hidden Stockpile — {W}{B} Enchantment. Revolt end steps make a Servo; {1},
/// sacrifice a creature: scry 1.
pub fn hidden_stockpile() -> CardDefinition {
    let servo = TokenDefinition {
        name: "Servo".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Servo], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Hidden Stockpile",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: make(1, servo),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ovalchase Daredevil — {3}{B} 4/2 Human Pilot. An artifact entering under
/// your control may bring it back from your graveyard to hand.
pub fn ovalchase_daredevil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::ControlledByYou) },
            ),
            effect: Effect::MayDo {
                description: "Return Ovalchase Daredevil to your hand?".into(),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
            },
        }],
        ..creature("Ovalchase Daredevil", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Pilot], 4, 2)
    }
}

/// Spitting Dilophosaurus — {2}{B} 3/2 Dinosaur. Entering or attacking puts a
/// -1/-1 counter on up to one creature; opponents' creatures with -1/-1
/// counters can't block.
pub fn spitting_dilophosaurus() -> CardDefinition {
    let spit = || Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::ONE }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(spit()), on_attack(spit())],
        static_abilities: vec![static_ab(
            "Creatures your opponents control with -1/-1 counters on them can't block.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByOpponent).and(R::WithCounter(CounterType::MinusOneMinusOne)),
                ),
                keyword: Keyword::CantBlock,
            },
        )],
        ..creature("Spitting Dilophosaurus", cost(&[generic(2), b()]), vec![CreatureType::Dinosaur], 3, 2)
    }
}

/// Staff of Eden, Vault's Key — {6} legendary artifact. ETB returns a
/// legendary permanent card from any graveyard under your control; {T}: draw
/// one per permanent you control but don't own.
pub fn staff_of_eden_vaults_key() -> CardDefinition {
    CardDefinition {
        name: "Staff of Eden, Vault's Key",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::HasSupertype(Supertype::Legendary)
                    .and(R::Permanent)
                    .and(R::InGraveyard)
                    .and(R::Not(Box::new(R::HasName("Staff of Eden, Vault's Key".into())))),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::ControlledByYou.and(R::Not(Box::new(R::OwnedByYou))),
                ))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mastermind Plum — {2}{B} 2/2 Human Wizard. Attacking exiles up to one
/// card from a graveyard (an artifact pays a Treasure); a spell paid with
/// Treasure mana draws a card for 1 life.
pub fn mastermind_plum() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::TargetFiltered { slot: 0, filter: R::InGraveyard } },
                    Effect::If {
                        cond: Predicate::EntityMatchesAny { what: Selector::Target(0), filter: R::Artifact },
                        then: Box::new(treasure()),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastWithTreasureMana { what: Selector::TriggerSource }),
                effect: Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
            },
        ],
        ..legend("Mastermind Plum", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Shadowheart, Dark Justiciar — {3}{B} 3/4 Human Elf Cleric, choose a
/// Background. {1}{B}, {T}, sacrifice another creature: draw its power.
pub fn shadowheart_dark_justiciar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
            ..Default::default()
        }],
        ..legend(
            "Shadowheart, Dark Justiciar",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Cleric],
            3,
            4,
        )
    }
}

/// Curious Colossus — {5}{W}{W} 7/7 Giant Warrior. ETB turns a target
/// opponent's creatures into ability-less 1/1 Cowards.
pub fn curious_colossus() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Seq(vec![
                Effect::LoseAllAbilities { what: theirs(), duration: Duration::Permanent },
                Effect::AddCreatureTypes { what: theirs(), creature_types: vec![CreatureType::Coward], duration: Duration::Permanent },
                Effect::SetBasePT { what: theirs(), power: Value::ONE, toughness: Value::ONE, duration: Duration::Permanent },
            ])),
        })],
        ..creature("Curious Colossus", cost(&[generic(5), w(), w()]), vec![CreatureType::Giant, CreatureType::Warrior], 7, 7)
    }
}

/// Bre of Clan Stoutarm — {2}{R}{W} 4/4 Giant Warrior. {1}{W}, {T}: another
/// creature of yours gains flying and lifelink; your end step after gaining
/// life digs to a nonland card, free if its mana value is at most the life
/// gained, else to hand.
pub fn bre_of_clan_stoutarm() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    let dug = || Selector::ExiledThisResolution { filter: R::Nonland };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(mine().and(R::OtherThanSource)),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                grant(Keyword::Lifelink),
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::LifeGainedThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE }),
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::LifeGainedThisTurn(PlayerRef::You),
                        Value::ManaValueOf(Box::new(dug())),
                    ),
                    then: Box::new(Effect::CastWithoutPayingImmediate {
                        what: dug(),
                        source_zone: Zone::Exile,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    }),
                    else_: Box::new(Effect::Noop),
                },
                // "Otherwise, put it into your hand" — whatever wasn't cast.
                Effect::Move {
                    what: Selector::ExiledThisResolution { filter: R::Nonland.and(R::InExile) },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
        }],
        ..legend(
            "Bre of Clan Stoutarm",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Tatsunari, Toad Rider — {2}{B} 3/3 Human Ninja. An enchantment spell
/// summons Keimi (a legendary 3/3 Frog that drains 1 per enchantment spell)
/// if you have none; {1}{G/U}: it and a Frog of yours can be blocked only by
/// fliers and reach this turn.
pub fn tatsunari_toad_rider() -> CardDefinition {
    let enchantment_cast = || {
        EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasCardType(CardType::Enchantment)))
    };
    let keimi = TokenDefinition {
        supertypes: vec![Supertype::Legendary],
        colors: vec![Color::Black, Color::Green],
        triggered_abilities: vec![TriggeredAbility {
            event: enchantment_cast(),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..token("Keimi", 3, 3, Color::Black, vec![CreatureType::Frog])
    };
    let no_keimi = Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::HasName("Keimi".into())),
    ))));
    let elusive = || Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Reach))));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: enchantment_cast().with_filter(no_keimi.clone()),
            effect: Effect::If { cond: no_keimi, then: Box::new(make(1, keimi)), else_: Box::new(Effect::Noop) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), hybrid(Color::Green, Color::Blue)]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(mine().and(R::HasCreatureType(CreatureType::Frog))),
                    keyword: elusive(),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::This, keyword: elusive(), duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..legend("Tatsunari, Toad Rider", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Ninja], 3, 3)
    }
}

/// Ivy, Gleeful Spellthief — {G}{U} 2/1 flying Faerie Rogue. A spell that
/// targets only one creature (not Ivy) may be copied at Ivy.
pub fn ivy_gleeful_spellthief() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellTargetsOnlyOneMatching(R::Creature.and(R::OtherThanSource))),
            effect: Effect::MayDo {
                description: "Copy that spell, targeting Ivy?".into(),
                body: Box::new(Effect::CopySpellTargeting { what: Selector::TriggerSource, target: Selector::This }),
            },
        }],
        ..legend("Ivy, Gleeful Spellthief", cost(&[g(), u()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 1)
    }
}

/// Apocalypse Hydra — {X}{R}{G} 0/0 Hydra. Enters with X +1/+1 counters, 2X
/// at X five or more; {1}{R}, remove a counter: 1 damage to any target.
pub fn apocalypse_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::ValueAtLeast(Value::XFromCost, Value::Const(5))),
                then: Box::new(Value::Sum(vec![Value::XFromCost, Value::XFromCost])),
                else_: Box::new(Value::XFromCost),
            },
        )),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Apocalypse Hydra", cost(&[x(), r(), g()]), vec![CreatureType::Hydra], 0, 0)
    }
}

/// Cavalier of Flame — {2}{R}{R}{R} 6/5 Elemental Knight. {1}{R}: your
/// creatures get +1/+0 and haste; ETB rummages any number; dying burns each
/// opponent and their planeswalkers for your graveyard's lands.
pub fn cavalier_of_flame() -> CardDefinition {
    let lands = || Value::CountOf(Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Land }));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(mine()),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::EachPermanent(mine()), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::DiscardAnyNumber { who: Selector::You, filter: R::Any, max: None },
                Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
            ])),
            on_dies(Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: lands() },
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByOpponent)),
                    amount: lands(),
                },
            ])),
        ],
        ..creature(
            "Cavalier of Flame",
            cost(&[generic(2), r(), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Knight],
            6,
            5,
        )
    }
}

/// Spiteful Banditry — {X}{R}{R} Enchantment. ETB X damage to each creature;
/// opponents' creatures dying makes a Treasure, once each turn.
pub fn spiteful_banditry() -> CardDefinition {
    CardDefinition {
        name: "Spiteful Banditry",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::XFromCost }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl).once_per_turn(),
                effect: treasure(),
            },
        ],
        ..Default::default()
    }
}

/// Allosaurus Shepherd — {G} 1/1 Elf Shaman, can't be countered; your green
/// spells can't be countered; {4}{G}{G}: your Elves become 5/5 Dinosaurs.
pub fn allosaurus_shepherd() -> CardDefinition {
    let elves = || Selector::EachPermanent(mine().and(R::HasCreatureType(CreatureType::Elf)));
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        static_abilities: vec![static_ab(
            "Green spells you control can't be countered.",
            StaticEffect::SpellsUncounterable { filter: R::HasColor(Color::Green).and(R::ControlledByYou) },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), g()]),
            effect: Effect::Seq(vec![
                Effect::SetBasePT { what: elves(), power: Value::Const(5), toughness: Value::Const(5), duration: Duration::EndOfTurn },
                Effect::AddCreatureTypes { what: elves(), creature_types: vec![CreatureType::Dinosaur], duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Allosaurus Shepherd", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Shaman], 1, 1)
    }
}

/// Legolas's Quick Reflexes — {G} Instant, split second. Untap a creature; it
/// gains hexproof, reach and "becoming tapped deals its power to up to one
/// creature" until end of turn.
pub fn legolass_quick_reflexes() -> CardDefinition {
    let grant = |keyword| Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn };
    CardDefinition {
        keywords: vec![Keyword::SplitSecond],
        ..spell(
            "Legolas's Quick Reflexes",
            cost(&[g()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                grant(Keyword::Hexproof),
                grant(Keyword::Reach),
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                        effect: Effect::OptionalTargets {
                            min: 0,
                            body: Box::new(Effect::DealDamage {
                                to: target_filtered(R::Creature),
                                amount: Value::PowerOf(Box::new(Selector::This)),
                            }),
                        },
                    }),
                    duration: Duration::EndOfTurn,
                },
            ]),
        )
    }
}

/// Nim Deathmantle — {2} Equipment, equip {4}. +2/+2, intimidate, a black
/// Zombie; a nontoken creature of yours dying may pay {4} to come back
/// wearing it.
pub fn nim_deathmantle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::IsToken.negate(),
            }),
            effect: Effect::MayPay {
                description: "Pay {4} to return it wearing Nim Deathmantle?".into(),
                mana_cost: cost(&[generic(4)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::AttachSourceTo { host: Selector::LastMoved },
                ])),
                else_: None,
            },
        }],
        ..equipment(
            "Nim Deathmantle",
            cost(&[generic(2)]),
            cost(&[generic(4)]),
            EquipBonus {
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Intimidate],
                set_colors: Some(vec![Color::Black]),
                set_creature_types: Some(vec![CreatureType::Zombie]),
                ..Default::default()
            },
        )
    }
}

/// Froghemoth — {3}{G}{G} 4/4 trample, haste Frog Horror. Its combat damage
/// to a player exiles up to that many cards from that player's graveyard,
/// growing per creature card and gaining 1 life per other card.
pub fn froghemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::WithX {
                x: Value::TriggerEventAmount,
                body: Box::new(Effect::ReflexiveTrigger {
                    body: Box::new(Effect::CapTargetsAtX {
                        body: Box::new(Effect::ApplyToTargets {
                            max_targets: 20,
                            min_targets: 0,
                            filter: R::InOpponentGraveyard,
                            effect: Box::new(Effect::Seq(vec![
                                Effect::If {
                                    cond: Predicate::EntityMatchesAny { what: Selector::Target(0), filter: R::Creature },
                                    then: Box::new(Effect::AddCounter {
                                        what: Selector::This,
                                        kind: CounterType::PlusOnePlusOne,
                                        amount: Value::ONE,
                                    }),
                                    else_: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
                                },
                                Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                            ])),
                        }),
                    }),
                }),
            },
        }],
        ..creature("Froghemoth", cost(&[generic(3), g(), g()]), vec![CreatureType::Frog, CreatureType::Horror], 4, 4)
    }
}

/// Haze Frog — {3}{G}{G} 2/1 flash Frog. ETB prevents all combat damage other
/// creatures would deal this turn.
pub fn haze_frog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PreventAllCombatDamageByMatchingThisTurn {
            filter: R::Creature.and(R::OtherThanSource),
        })],
        ..creature("Haze Frog", cost(&[generic(3), g(), g()]), vec![CreatureType::Frog], 2, 1)
    }
}

/// Mabel, Valley Hero — {1}{R}{W} 1/2 Mouse Soldier. Mabel or another
/// creature of yours entering grows a creature that entered this turn.
pub fn mabel_valley_hero() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::EnteredThisTurn)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..legend("Mabel, Valley Hero", cost(&[generic(1), r(), w()]), vec![CreatureType::Mouse, CreatureType::Soldier], 1, 2)
    }
}

/// Three Blind Mice — {2}{W} Saga. I: a 1/1 Mouse; II, III: copy a token of
/// yours; IV: your creatures get +1/+1 and vigilance.
pub fn three_blind_mice() -> CardDefinition {
    let copy = || {
        crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, target_filtered(R::IsToken.and(R::ControlledByYou)))
    };
    CardDefinition {
        name: "Three Blind Mice",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, make(1, token("Mouse", 1, 1, Color::White, vec![CreatureType::Mouse]))),
            (2, copy()),
            (3, copy()),
            (
                4,
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::EachPermanent(mine()),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(mine()),
                        keyword: Keyword::Vigilance,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Semblance Anvil — {3} Artifact. Imprint a nonland card from hand; spells
/// sharing a card type with it cost {2} less.
pub fn semblance_anvil() -> CardDefinition {
    CardDefinition {
        name: "Semblance Anvil",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Imprint: exile a nonland card from your hand?".into(),
            body: Box::new(Effect::ExileTaggedWithSource {
                what: Selector::take(
                    Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: R::Nonland },
                    Value::ONE,
                ),
            }),
        })],
        static_abilities: vec![static_ab(
            "Spells you cast that share a card type with the exiled card cost {2} less to cast.",
            StaticEffect::CostReduction { filter: R::SharesCardTypeWithExiledBySource, amount: 2 },
        )],
        ..Default::default()
    }
}

/// Artificer's Assistant — {U} 1/1 flying Bird. Historic spells you cast
/// scry 1.
pub fn artificers_assistant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::historic())),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature("Artificer's Assistant", cost(&[u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Tribute to Horobi // Echo of Death's Wail — {1}{B} Saga. I, II: each
/// opponent makes a 1/1 Rat Rogue; III: it returns transformed — a 3/3 flying,
/// hasty Spirit that takes every Rat token and trades another creature for a
/// card on attack.
pub fn tribute_to_horobi() -> CardDefinition {
    let rat = token("Rat Rogue", 1, 1, Color::Black, vec![CreatureType::Rat, CreatureType::Rogue]);
    let rats = || Effect::CreateToken { who: PlayerRef::EachOpponent, count: Value::ONE, definition: Arc::new(rat.clone()) };
    let echo = CardDefinition {
        name: "Echo of Death's Wail",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        color_indicator: vec![Color::Black],
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![
            etb(Effect::GainControl {
                what: Selector::EachPermanent(R::IsToken.and(R::HasCreatureType(CreatureType::Rat))),
                to: None,
                duration: Duration::Permanent,
            }),
            on_attack(Effect::MaySacrifice {
                description: "Sacrifice another creature to draw a card?".into(),
                filter: R::Creature.and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(draw(1)),
                else_: None,
            }),
        ],
        ..Default::default()
    };
    CardDefinition {
        name: "Tribute to Horobi",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, rats()), (2, rats()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(echo)),
        ..Default::default()
    }
}

/// Smaug the Impenetrable — {5}{B}{R} 8/7 flying, indestructible, haste
/// Dragon. Noncombat damage dealt to it makes that many Treasures.
pub fn smaug_the_impenetrable() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Indestructible, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::IfTriggerDamageNoncombat {
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                }),
            },
        }],
        ..legend("Smaug the Impenetrable", cost(&[generic(5), b(), r()]), vec![CreatureType::Dragon], 8, 7)
    }
}

/// Dragon-Cursed Halls — Land. {T}: {C}; {1}, {T}: a creature gains "combat
/// damage to a player makes a Treasure" until end of turn.
pub fn dragon_cursed_halls() -> CardDefinition {
    CardDefinition {
        name: "Dragon-Cursed Halls",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::GrantTriggeredAbility {
                    what: target_filtered(R::Creature),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                        effect: treasure(),
                    }),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Heliod, the Radiant Dawn // Heliod, the Warped Eclipse — {2}{W}{W} 4/4
/// enchantment God. ETB regrows a non-God enchantment; {3}{U/P}: transform
/// (sorcery speed). The back face gives your spells flash and makes them {1}
/// cheaper per card your opponents drew this turn.
pub fn heliod_the_radiant_dawn() -> CardDefinition {
    let eclipse = CardDefinition {
        name: "Heliod, the Warped Eclipse",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian, CreatureType::God], ..Default::default() },
        color_indicator: vec![Color::White, Color::Blue],
        power: 4,
        toughness: 6,
        static_abilities: vec![
            static_ab("You may cast spells as though they had flash.", StaticEffect::ControllerSpellsHaveFlash { filter: R::Any }),
            static_ab(
                "Spells you cast cost {1} less to cast for each card your opponents have drawn this turn.",
                StaticEffect::CostReductionByValue { filter: R::Any, amount: Value::CardsDrawnThisTurnByOpponents },
            ),
        ],
        ..Default::default()
    };
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::HasCardType(CardType::Enchantment)
                    .and(R::InYourGraveyard)
                    .and(R::HasCreatureType(CreatureType::God).negate()),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), crate::mana::phyrexian(Color::Blue)]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(eclipse)),
        ..legend("Heliod, the Radiant Dawn", cost(&[generic(2), w(), w()]), vec![CreatureType::God], 4, 4)
    }
}

/// Stenn, Paranoid Partisan — {W}{U} 2/2 Human Wizard. As it enters, choose a
/// card type other than creature or land; spells of it cost {1} less; {1}{W}{U}:
/// blink it until the next end step.
pub fn stenn_paranoid_partisan() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::ChooseCardTypeFromForSource {
            options: vec![
                CardType::Artifact,
                CardType::Enchantment,
                CardType::Instant,
                CardType::Sorcery,
                CardType::Planeswalker,
            ],
        }),
        static_abilities: vec![static_ab(
            "Spells you cast of the chosen type cost {1} less to cast.",
            StaticEffect::CostReduction { filter: R::IsSourceChosenCardType, amount: 1 },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), u()]),
            effect: Effect::ExileReturnToOwnerNextEndStep { what: Selector::This, tapped: false },
            ..Default::default()
        }],
        ..legend("Stenn, Paranoid Partisan", cost(&[w(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}

/// Briarbridge Patrol — {3}{G} 3/3 Human Warrior. Damaging creatures
/// investigates; each end step after you sacrificed three Clues this turn,
/// you may put a creature card from hand onto the battlefield.
pub fn briarbridge_patrol() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource),
                effect: crate::effect::shortcut::investigate(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::ValueAtLeast(Value::CluesSacrificedThisTurn(PlayerRef::You), Value::Const(3)),
                ),
                effect: Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            },
        ],
        ..creature("Briarbridge Patrol", cost(&[generic(3), g()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 3)
    }
}

/// Confront the Unknown — {G} Instant. Investigate, then a creature gets
/// +1/+1 per Clue you control.
pub fn confront_the_unknown() -> CardDefinition {
    let clues = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::HasArtifactSubtype(ArtifactSubtype::Clue).and(R::ControlledByYou),
        )))
    };
    spell(
        "Confront the Unknown",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![
            crate::effect::shortcut::investigate(1),
            Effect::PumpPT { what: target_filtered(R::Creature), power: clues(), toughness: clues(), duration: Duration::EndOfTurn },
        ]),
    )
}

/// Eirdu, Carrier of Dawn // Isilu, Carrier of Twilight — {3}{W}{W} 5/5
/// flying, lifelink God. Your creature spells have convoke; your first main
/// phase may pay {B} to transform. Isilu gives your other nontoken creatures
/// persist and may pay {W} to transform back.
pub fn eirdu_carrier_of_dawn() -> CardDefinition {
    let flip = |pip| {
        at_your(
            TurnStep::PreCombatMain,
            Effect::MayPay {
                description: "Pay to transform?".into(),
                mana_cost: cost(&[pip]),
                body: Box::new(Effect::Transform { what: Selector::This }),
                else_: None,
            },
        )
    };
    let isilu = CardDefinition {
        name: "Isilu, Carrier of Twilight",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental, CreatureType::God], ..Default::default() },
        color_indicator: vec![Color::Black],
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![static_ab(
            "Each other nontoken creature you control has persist.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(mine().and(R::IsToken.negate()).and(R::OtherThanSource)),
                keyword: Keyword::Persist,
            },
        )],
        triggered_abilities: vec![flip(w())],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![static_ab(
            "Creature spells you cast have convoke.",
            StaticEffect::GrantConvokeToSpells { filter: R::Creature },
        )],
        triggered_abilities: vec![flip(b())],
        back_face: Some(Box::new(isilu)),
        ..legend(
            "Eirdu, Carrier of Dawn",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Elemental, CreatureType::God],
            5,
            5,
        )
    }
}

/// Rhys, the Evermore — {1}{W} 2/2 flash Elf Warrior. ETB gives another
/// creature of yours persist this turn; {W}, {T}: remove counters from a
/// creature of yours, as a sorcery.
pub fn rhys_the_evermore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(mine().and(R::OtherThanSource)),
            keyword: Keyword::Persist,
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::RemoveAllCounters { what: target_filtered(mine()) },
            ..Default::default()
        }],
        ..legend("Rhys, the Evermore", cost(&[generic(1), w()]), vec![CreatureType::Elf, CreatureType::Warrior], 2, 2)
    }
}

/// Guile — {3}{U}{U}{U} 6/6 Elemental Incarnation. Blocked only by three or
/// more; your counterspells exile instead and let you play the card free;
/// put into a graveyard from anywhere, it shuffles into its library.
pub fn guile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptByN(3)],
        static_abilities: vec![static_ab(
            "If a spell or ability you control would counter a spell, instead exile that spell and you may play that card without paying its mana cost.",
            StaticEffect::CounterBecomesExileFreePlay,
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::ShuffleSelfIntoLibrary,
        }],
        ..creature(
            "Guile",
            cost(&[generic(3), u(), u(), u()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

/// Ertai's Scorn — {1}{U}{U} Instant, {U} less once an opponent has cast two
/// spells this turn. Counter target spell.
pub fn ertais_scorn() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(1), u()]),
            // The most spells any one opponent has cast this turn.
            condition: Some(Predicate::ValueAtLeast(Value::SpellsCastThisTurn(PlayerRef::EachOpponent), Value::Const(2))),
            ..Default::default()
        }),
        ..spell(
            "Ertai's Scorn",
            cost(&[generic(1), u(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        )
    }
}

/// Loki, God of Mischief — {1}{U} 2/1 God Sorcerer Villain. Once each turn,
/// a player or permanent becoming the target of an ability you control draws
/// you a card. The targeting is CR 601.2c's per-object choice: a spell chose
/// it when the chooser is a spell on the stack, otherwise an ability did.
pub fn loki_god_of_mischief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ChoseTargets, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsSpellOnStack,
                })))
                .once_per_turn(),
            effect: draw(1),
        }],
        ..legend(
            "Loki, God of Mischief",
            cost(&[generic(1), u()]),
            vec![CreatureType::God, CreatureType::Sorcerer, CreatureType::Villain],
            2,
            1,
        )
    }
}

/// Glasses of Urza — {1} Artifact. {T}: look at a target player's hand.
pub fn glasses_of_urza() -> CardDefinition {
    CardDefinition {
        name: "Glasses of Urza",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LookAtHand { who: target_filtered(R::Player) },
            ..Default::default()
        }],
        ..Default::default()
    }
}
