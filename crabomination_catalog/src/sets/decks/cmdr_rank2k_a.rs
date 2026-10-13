//! Commander: the one-card gaps of EDHREC's commanders ranked 1501-2000 (the
//! 2026-10-13 twenty-page rescan), first file. Tests in
//! `tests/recent_b/cmdr_rank2k_a.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_dies, sneak, target_filtered};
use crate::effect::{
    ActivatedAbility, CounteredSpellZone, Duration, Effect, ManaPayload, PlayerRef, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, snow_mana, u, w, Color, ManaCost};

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

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
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

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn grant_eot(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn pump_eot(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn mountain() -> R {
    R::HasLandType(LandType::Mountain)
}

/// Adult Gold Dragon — {3}{R}{W} 4/3 Dragon; flying, lifelink, haste.
pub fn adult_gold_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Haste],
        ..creature("Adult Gold Dragon", cost(&[generic(3), r(), w()]), vec![CreatureType::Dragon], 4, 3)
    }
}

/// Alpine Guide — {2}{R} snow 3/3 Human Scout. Entering may fetch a Mountain
/// card tapped; attacks each combat; leaving sacrifices a Mountain.
pub fn alpine_guide() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![Keyword::MustAttack],
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search your library for a Mountain card?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(mountain()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Land.and(mountain()) },
            },
        ],
        ..creature("Alpine Guide", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Scout], 3, 3)
    }
}

/// Beguiler of Wills — {3}{U}{U} 1/1 Human Wizard. {T}: steal a creature
/// with power at most the number of creatures you control.
pub fn beguiler_of_wills() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControl {
                what: target_filtered(
                    R::Creature.and(R::PowerAtMostYourCount(Box::new(R::Creature.and(R::ControlledByYou)))),
                ),
                to: None,
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature("Beguiler of Wills", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Blanchwood Prowler — {1}{G} 1/1 Elemental. Entering mills three; a land
/// among them may go to hand, else it gets a +1/+1 counter.
pub fn blanchwood_prowler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: R::Land,
            otherwise: Some(Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            })),
        })],
        ..creature("Blanchwood Prowler", cost(&[generic(1), g()]), vec![CreatureType::Elemental], 1, 1)
    }
}

/// Bloodmad Vampire — {2}{R} 4/1 Vampire Berserker; grows on combat damage to
/// a player; madness {1}{R}.
pub fn bloodmad_vampire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[generic(1), r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Bloodmad Vampire", cost(&[generic(2), r()]), vec![CreatureType::Vampire, CreatureType::Berserker], 4, 1)
    }
}

/// Breath of Life — {3}{W} sorcery: reanimate a creature card of yours.
pub fn breath_of_life() -> CardDefinition {
    spell(
        "Breath of Life",
        cost(&[generic(3), w()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Creature.and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Cathartic Adept — {U} 1/1 Human Wizard. {T}: target player mills a card.
pub fn cathartic_adept() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Cathartic Adept", cost(&[u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Cavalier of Night — {2}{B}{B}{B} 4/5 lifelink Elemental Knight. Entering
/// may sacrifice another creature to destroy an opponent's creature; dying
/// reanimates a creature card with mana value 3 or less.
pub fn cavalier_of_night() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![
            etb(Effect::MaySacrifice {
                description: "sacrifice another creature".into(),
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                count: Value::ONE,
                then: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Destroy {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    }),
                }),
                else_: None,
            }),
            on_dies(Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        ],
        ..creature(
            "Cavalier of Night",
            cost(&[generic(2), b(), b(), b()]),
            vec![CreatureType::Elemental, CreatureType::Knight],
            4,
            5,
        )
    }
}

/// Cerulean Drake — {1}{U} 1/1 flying Drake, protection from red. Sacrifice
/// it: counter target spell that targets you.
pub fn cerulean_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::SpellTargetsMatching(Box::new(R::Player.and(R::ControlledByYou)))),
                ),
            },
            ..Default::default()
        }],
        ..creature("Cerulean Drake", cost(&[generic(1), u()]), vec![CreatureType::Drake], 1, 1)
    }
}

/// Crystalline Armor — {3}{G} Aura: +1/+1 per land you control and trample.
pub fn crystalline_armor() -> CardDefinition {
    let lands = || Value::CountOf(Box::new(yours(R::Land)));
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "Enchanted creature gets +1/+1 for each land you control.",
                StaticEffect::PumpPTByValue { applies_to: host(), power: lands(), toughness: lands() },
            ),
            static_ab(
                "Enchanted creature has trample.",
                StaticEffect::GrantKeyword { applies_to: host(), keyword: Keyword::Trample },
            ),
        ],
        ..aura("Crystalline Armor", cost(&[generic(3), g()]))
    }
}

/// Evil Presence — {B} Aura: enchanted land is a Swamp.
pub fn evil_presence() -> CardDefinition {
    CardDefinition {
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![static_ab(
            "Enchanted land is a Swamp.",
            StaticEffect::LandTypeChanger { applies_to: host(), land_type: LandType::Swamp, replace: true },
        )],
        ..aura("Evil Presence", cost(&[b()]))
    }
}

/// Gregor, Shrewd Magistrate — {1}{W}{U} 1/3 skulk Human Advisor; combat
/// damage to a player draws cards equal to its power.
pub fn gregor_shrewd_magistrate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: draw(Value::PowerOf(Box::new(Selector::This))),
        }],
        ..legend(
            "Gregor, Shrewd Magistrate",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            3,
        )
    }
}

/// Hawkeye's Bow — {R} Equipment: +1/+0 and reach; the equipped creature
/// becoming tapped deals 1 damage to each opponent. Equip {1}.
pub fn hawkeyes_bow() -> CardDefinition {
    equipment(
        "Hawkeye's Bow",
        cost(&[r()]),
        cost(&[generic(1)]),
        EquipBonus {
            power: 1,
            keywords: vec![Keyword::Reach],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            }],
            ..Default::default()
        },
    )
}

/// Impolite Entrance — {R} sorcery: trample and haste until end of turn; draw.
pub fn impolite_entrance() -> CardDefinition {
    spell(
        "Impolite Entrance",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            grant_eot(target_filtered(R::Creature), Keyword::Trample),
            grant_eot(Selector::Target(0), Keyword::Haste),
            draw(Value::ONE),
        ]),
    )
}

/// Inherited Envelope — {3} artifact: the Ring tempts you on entry; taps for
/// one mana of any color.
pub fn inherited_envelope() -> CardDefinition {
    CardDefinition {
        name: "Inherited Envelope",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::RingTempts { who: PlayerRef::You })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Jaheira's Respite — {4}{G} instant: basics onto the battlefield tapped, one
/// per creature attacking you; prevent all combat damage this turn.
pub fn jaheiras_respite() -> CardDefinition {
    spell(
        "Jaheira's Respite",
        cost(&[generic(4), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttackingYou)))),
            },
            Effect::PreventAllCombatDamageThisTurn,
        ]),
    )
}

/// Kels, Fight Fixer — {2}{B}{B} 4/3 menace Azra Warlock. Sacrificing a
/// creature may pay {U/B} to draw; {1}, sacrifice a creature: indestructible.
pub fn kels_fight_fixer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {U/B} to draw a card?".into(),
                mana_cost: cost(&[hybrid(Color::Blue, Color::Black)]),
                body: Box::new(draw(Value::ONE)),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: grant_eot(Selector::This, Keyword::Indestructible),
            ..Default::default()
        }],
        ..legend("Kels, Fight Fixer", cost(&[generic(2), b(), b()]), vec![CreatureType::Azra, CreatureType::Warlock], 4, 3)
    }
}

/// Knight of Dawn's Light — {1}{W} 2/2 first strike Human Knight; life gain
/// gets +1; {1}{W}: +1/+1 until end of turn.
pub fn knight_of_dawns_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![static_ab(
            "If you would gain life, you gain that much life plus 1 instead.",
            StaticEffect::LifeGainBonus { target: crate::effect::PlayerStaticTarget::Controller, amount: 1 },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: pump_eot(Selector::This, Value::ONE, Value::ONE),
            ..Default::default()
        }],
        ..creature("Knight of Dawn's Light", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Lapse of Certainty — {2}{W} instant: counter target spell; it goes on top
/// of its owner's library instead.
pub fn lapse_of_certainty() -> CardDefinition {
    spell(
        "Lapse of Certainty",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::CounterSpellToZone {
            what: target_filtered(R::IsSpellOnStack),
            zone: CounteredSpellZone::OwnerLibraryTop,
        },
    )
}

/// Lovisa Coldeyes — {3}{R}{R} legendary 3/3 Human. Every Barbarian, Warrior
/// and Berserker gets +2/+2 and has haste.
pub fn lovisa_coldeyes() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Each creature that's a Barbarian, a Warrior, or a Berserker gets +2/+2 and has haste.",
            StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Barbarian)
                    .or(R::HasCreatureType(CreatureType::Warrior))
                    .or(R::HasCreatureType(CreatureType::Berserker)),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Haste],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        )],
        ..legend("Lovisa Coldeyes", cost(&[generic(3), r(), r()]), vec![CreatureType::Human], 3, 3)
    }
}

/// Magus of the Tabernacle — {3}{W} 2/6 Human Wizard. Every creature has "at
/// the beginning of your upkeep, sacrifice this creature unless you pay {1}."
pub fn magus_of_the_tabernacle() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "All creatures have \"At the beginning of your upkeep, sacrifice this creature unless you pay {1}.\"",
            StaticEffect::GrantTriggeredAbility {
                filter: R::Creature,
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                    effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1)]) },
                }),
            },
        )],
        ..creature("Magus of the Tabernacle", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 6)
    }
}

/// Merrow Bonegnawer — {B} 1/1 Merfolk Rogue. {T}: target player exiles a
/// card from their graveyard; casting a black spell may untap it.
pub fn merrow_bonegnawer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ExileFromGraveyard { who: PlayerRef::Target(0), count: Value::ONE, filter: R::Any },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Black))),
            effect: Effect::MayDo {
                description: "untap Merrow Bonegnawer".into(),
                body: Box::new(Effect::Untap { what: Selector::This, up_to: None }),
            },
        }],
        ..creature("Merrow Bonegnawer", cost(&[b()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 1, 1)
    }
}

/// Molten Man, Inferno Incarnate — {2}{R} legendary 0/0 Elemental Villain.
/// Entering fetches a basic Mountain tapped; +1/+1 per Mountain you control;
/// leaving sacrifices a land.
pub fn molten_man_inferno_incarnate() -> CardDefinition {
    let mountains = || Value::CountOf(Box::new(yours(mountain())));
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(mountain()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Land },
            },
        ],
        static_abilities: vec![static_ab(
            "Molten Man gets +1/+1 for each Mountain you control.",
            StaticEffect::PumpPTByValue { applies_to: Selector::This, power: mountains(), toughness: mountains() },
        )],
        ..legend(
            "Molten Man, Inferno Incarnate",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Villain],
            0,
            0,
        )
    }
}

/// New Generation's Technique — {3}{G} sorcery, sneak {2}{G}: two basic lands
/// onto the battlefield tapped.
pub fn new_generations_technique() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Sneak(cost(&[generic(2), g()]))],
        alternative_cost: Some(sneak(cost(&[generic(2), g()]))),
        ..spell(
            "New Generation's Technique",
            cost(&[generic(3), g()]),
            CardType::Sorcery,
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            },
        )
    }
}

/// Pestilence Rats — {2}{B} */3 Rat; power is the number of other Rats on
/// the battlefield.
pub fn pestilence_rats() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsOnBattlefieldMatching {
            base_p: 0,
            base_t: 3,
            filter: Box::new(R::HasCreatureType(CreatureType::Rat).and(R::OtherThanSource)),
        }),
        ..creature("Pestilence Rats", cost(&[generic(2), b()]), vec![CreatureType::Rat], 0, 3)
    }
}

/// Prodigious Growth — {4}{G}{G} Aura: +7/+7 and trample.
pub fn prodigious_growth() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { power: 7, toughness: 7, keywords: vec![Keyword::Trample], ..Default::default() }),
        ..aura("Prodigious Growth", cost(&[generic(4), g(), g()]))
    }
}

/// Radha, Heart of Keld — {1}{R}{G} 3/3 Elf Warrior. First strike on your
/// turn; look at and play lands from the top of your library; {4}{R}{G}:
/// +X/+X where X is the lands you control.
pub fn radha_heart_of_keld() -> CardDefinition {
    let lands = || Value::CountOf(Box::new(yours(R::Land)));
    CardDefinition {
        static_abilities: vec![
            static_ab(
                "During your turn, Radha has first strike.",
                StaticEffect::SelfHasKeywordIf { keyword: Keyword::FirstStrike, condition: Predicate::IsTurnOf(PlayerRef::You) },
            ),
            static_ab("You may look at the top card of your library any time.", StaticEffect::MayLookAtOwnLibraryTop),
            static_ab(
                "You may play lands from the top of your library.",
                StaticEffect::PlayFromLibraryTop { filter: R::Land },
            ),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), r(), g()]),
            effect: pump_eot(Selector::This, lands(), lands()),
            ..Default::default()
        }],
        ..legend("Radha, Heart of Keld", cost(&[generic(1), r(), g()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 3)
    }
}

/// Shenanigans — {1}{R} sorcery: destroy target artifact. Dredge 1.
pub fn shenanigans() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Dredge(1)],
        ..spell(
            "Shenanigans",
            cost(&[generic(1), r()]),
            CardType::Sorcery,
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Skanos Dragonheart — {4}{G} legendary 4/4 Dragon Ranger. Attacking, it
/// gets +X/+X where X is the greatest power among other Dragons you control
/// and Dragon cards in your graveyard. Choose a Background.
pub fn skanos_dragonheart() -> CardDefinition {
    let dragon = || R::HasCreatureType(CreatureType::Dragon);
    let x = || {
        Value::Max(
            Box::new(Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(
                R::Creature.and(dragon()).and(R::OtherThanSource),
            )))),
            Box::new(Value::GreatestPowerAmongCards(Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature.and(dragon()),
            }))),
        )
    };
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![on_attack(pump_eot(Selector::This, x(), x()))],
        ..legend("Skanos Dragonheart", cost(&[generic(4), g()]), vec![CreatureType::Dragon, CreatureType::Ranger], 4, 4)
    }
}

/// Social Climber — {2}{G} 3/2 Human Druid. Alliance: gain 1 life.
pub fn social_climber() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Social Climber", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Druid], 3, 2)
    }
}

/// Sprite Noble — {1}{U}{U} 2/2 flying Faerie Noble. Your other fliers get
/// +0/+1; {T}: they get +1/+0 until end of turn.
pub fn sprite_noble() -> CardDefinition {
    let fliers = || R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::ControlledByYou).and(R::OtherThanSource);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "Other creatures you control with flying get +0/+1.",
            StaticEffect::AnthemForFilter {
                filter: fliers(),
                power: 0,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: pump_eot(Selector::EachPermanent(fliers()), Value::ONE, Value::ZERO),
            ..Default::default()
        }],
        ..creature("Sprite Noble", cost(&[generic(1), u(), u()]), vec![CreatureType::Faerie, CreatureType::Noble], 2, 2)
    }
}

/// Stoic Angel — {1}{G}{W}{U} 3/4 flying, vigilance Angel. Players untap at
/// most one creature during their untap steps.
pub fn stoic_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![static_ab(
            "Players can't untap more than one creature during their untap steps.",
            StaticEffect::MaxOneUntapPerStep { filter: R::Creature },
        )],
        ..creature("Stoic Angel", cost(&[generic(1), g(), w(), u()]), vec![CreatureType::Angel], 3, 4)
    }
}

/// Study — land (MKM Commander's Study): enters tapped, taps for {W} or {U};
/// {4}, {T}: investigate.
pub fn study() -> CardDefinition {
    CardDefinition {
        name: "Study",
        card_types: vec![CardType::Land],
        static_abilities: vec![static_ab("This land enters tapped.", StaticEffect::EntersTapped { applies_to: Selector::This })],
        activated_abilities: vec![
            crate::sets::tap_add(Color::White),
            crate::sets::tap_add(Color::Blue),
            ActivatedAbility { mana_cost: cost(&[generic(4)]), tap_cost: true, effect: investigate(1), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Tasigur's Cruelty — {5}{B} sorcery with delve: each opponent discards two.
pub fn tasigurs_cruelty() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Delve],
        ..spell(
            "Tasigur's Cruelty",
            cost(&[generic(5), b()]),
            CardType::Sorcery,
            Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2), random: false },
        )
    }
}

/// Thermopod — {4}{R} snow 4/3 Slug. {S}: haste; sacrifice a creature: {R}.
pub fn thermopod() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[snow_mana()]),
                effect: grant_eot(Selector::This, Keyword::Haste),
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
                ..Default::default()
            },
        ],
        ..creature("Thermopod", cost(&[generic(4), r()]), vec![CreatureType::Slug], 4, 3)
    }
}

/// Transmogrant's Crown — {2} Equipment: +2/+0; the equipped creature dying
/// draws you a card. Equip {2} or {B}.
pub fn transmogrants_crown() -> CardDefinition {
    CardDefinition {
        equip_filtered_cost: Some((R::Creature, cost(&[b()]))),
        ..equipment(
            "Transmogrant's Crown",
            cost(&[generic(2)]),
            cost(&[generic(2)]),
            EquipBonus {
                power: 2,
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                    effect: draw(Value::ONE),
                }],
                ..Default::default()
            },
        )
    }
}

/// Unsubstantiate — {1}{U} instant: return target spell or creature to its
/// owner's hand.
pub fn unsubstantiate() -> CardDefinition {
    spell(
        "Unsubstantiate",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::IsSpellOnStack.or(R::Creature)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Writhing Necromass — {6}{B} 5/5 deathtouch Zombie Giant; costs {1} less
/// per creature card in your graveyard.
pub fn writhing_necromass() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        affinity_graveyard_filter: Some(R::Creature),
        ..creature("Writhing Necromass", cost(&[generic(6), b()]), vec![CreatureType::Zombie, CreatureType::Giant], 5, 5)
    }
}

/// Yorvo, Lord of Garenbrig — {G}{G}{G} legendary 0/0 Giant Noble; enters with
/// four +1/+1 counters. Another green creature of yours entering adds one,
/// and a second if that creature's power is greater than Yorvo's.
pub fn yorvo_lord_of_garenbrig() -> CardDefinition {
    let grow = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::HasColor(Color::Green)) },
            ),
            effect: Effect::Seq(vec![
                grow(),
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::PowerGreaterThanSource,
                    },
                    then: Box::new(grow()),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend("Yorvo, Lord of Garenbrig", cost(&[g(), g(), g()]), vec![CreatureType::Giant, CreatureType::Noble], 0, 0)
    }
}
