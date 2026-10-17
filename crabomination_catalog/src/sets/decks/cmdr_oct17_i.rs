//! Commander: the 2026-10-17 (`01Htv6cF`) one-card-gap closers, second
//! round — each the last missing card of an unseated deck. Tests in
//! `tests/recent_b/cmdr_oct17_h.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
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

fn kw(keywords: Vec<Keyword>, def: CardDefinition) -> CardDefinition {
    CardDefinition { keywords, ..def }
}

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
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

fn act(mana: ManaCost, tap: bool, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, tap_cost: tap, effect, ..Default::default() }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn mine(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    })
}

fn mint(n: i32, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: def }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Goblin Mountaineer — {R} 1/1 Goblin Scout, mountainwalk.
pub fn goblin_mountaineer() -> CardDefinition {
    kw(vec![Keyword::Landwalk(LandType::Mountain)], creature("Goblin Mountaineer", cost(&[r()]), vec![CreatureType::Goblin, CreatureType::Scout], 1, 1))
}

/// Baxter Stockman — {3}{U}{R} legendary 3/3 Human Scientist. ETB: a 1/1
/// colorless Robot artifact creature token. Beginning of combat on your turn:
/// target artifact creature you control gets +3/+0, first strike, vigilance.
pub fn baxter_stockman() -> CardDefinition {
    let robot = Arc::new(TokenDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..(*token("Robot", 1, 1, vec![], vec![CreatureType::Robot], vec![])).clone()
    });
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        triggered_abilities: vec![
            etb(mint(1, robot)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    pump(target_filtered(R::Artifact.and(R::Creature).and(R::ControlledByYou)), 3, 0),
                    grant(Selector::Target(0), Keyword::FirstStrike),
                    grant(Selector::Target(0), Keyword::Vigilance),
                ]),
            },
        ],
        ..creature("Baxter Stockman", cost(&[generic(3), u(), r()]), vec![CreatureType::Human, CreatureType::Scientist], 3, 3)
    }
}

/// Ruthless Sniper — {B} 1/2 Human Archer. Whenever you cycle or discard a
/// card, you may pay {1}; if you do, a -1/-1 counter on target creature.
pub fn ruthless_sniper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} to put a -1/-1 counter on target creature?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..creature("Ruthless Sniper", cost(&[b()]), vec![CreatureType::Human, CreatureType::Archer], 1, 2)
    }
}

/// Creakwood Safewright — {1}{B} 5/5 Elf Warrior; enters with three -1/-1
/// counters. Your end step, if an Elf card is in your graveyard and it has a
/// -1/-1 counter: remove one.
pub fn creakwood_safewright() -> CardDefinition {
    let minus = CounterType::MinusOneMinusOne;
    CardDefinition {
        enters_with_counters: Some((minus, Value::Const(3))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::SelectorCountAtLeast {
                    sel: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::HasCreatureType(CreatureType::Elf),
                    },
                    n: Value::ONE,
                },
                Predicate::ValueAtLeast(Value::CountersOn { what: Box::new(Selector::This), kind: minus }, Value::ONE),
            ])),
            effect: Effect::RemoveCounter { what: Selector::This, kind: minus, amount: Value::ONE },
        }],
        ..creature("Creakwood Safewright", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Warrior], 5, 5)
    }
}

/// Trusty Retriever — {3}{W} 2/3 Dog. ETB, choose one — a +1/+1 counter on
/// it; or return target artifact or enchantment card from your graveyard to
/// your hand.
pub fn trusty_retriever() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Move {
                what: target_filtered(R::Artifact.or(R::Enchantment).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]))],
        ..creature("Trusty Retriever", cost(&[generic(3), w()]), vec![CreatureType::Dog], 2, 3)
    }
}

/// Wicked Wolf — {2}{G}{G} 3/3 Wolf. ETB: it fights up to one target creature
/// you don't control. Sacrifice a Food: a +1/+1 counter on it; indestructible
/// until end of turn; tap it.
pub fn wicked_wolf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature.and(R::ControlledByYou.negate()),
            effect: Box::new(Effect::Fight { attacker: Selector::This, defender: Selector::Target(0) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                grant(Selector::This, Keyword::Indestructible),
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Wicked Wolf", cost(&[generic(2), g(), g()]), vec![CreatureType::Wolf], 3, 3)
    }
}

/// Inexorable Blob — {2}{G} 3/3 Ooze. Delirium — whenever it attacks, if four
/// or more card types are among cards in your graveyard, a 3/3 green Ooze
/// token tapped and attacking.
pub fn inexorable_blob() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::DeliriumActive { who: PlayerRef::You }),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token("Ooze", 3, 3, vec![Color::Green], vec![CreatureType::Ooze], vec![]),
                cleanup: Default::default(),
                defender: None,
            },
        }],
        ..creature("Inexorable Blob", cost(&[generic(2), g()]), vec![CreatureType::Ooze], 3, 3)
    }
}

/// Meadowboon — {2}{W}{W} 3/3 Elemental. Leaves the battlefield: a +1/+1
/// counter on each creature target player controls. Evoke {3}{W}.
pub fn meadowboon() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(crate::effect::shortcut::evoke(cost(&[generic(3), w()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                min_targets: 1,
                max_targets: 1,
                filter: R::Player,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Meadowboon", cost(&[generic(2), w(), w()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Mindwrack Liege — {3}{U/R}{U/R}{U/R} 4/4 Horror. Other blue creatures you
/// control get +1/+1; other red ones get +1/+1. {U/R}x4: you may put a blue
/// or red creature card from your hand onto the battlefield.
pub fn mindwrack_liege() -> CardDefinition {
    let ur = || crate::mana::hybrid(Color::Blue, Color::Red);
    let lord = |c: Color, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource).and(R::HasColor(c))),
            power: 1,
            toughness: 1,
        },
    };
    CardDefinition {
        static_abilities: vec![
            lord(Color::Blue, "Other blue creatures you control get +1/+1."),
            lord(Color::Red, "Other red creatures you control get +1/+1."),
        ],
        activated_abilities: vec![act(
            cost(&[ur(), ur(), ur(), ur()]),
            false,
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasColor(Color::Blue).or(R::HasColor(Color::Red))),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        )],
        ..creature("Mindwrack Liege", cost(&[generic(3), ur(), ur(), ur()]), vec![CreatureType::Horror], 4, 4)
    }
}

/// Henge Walker — {3} 2/2 artifact Golem. Adamant — enters with a +1/+1
/// counter if at least three mana of the same color was spent to cast it.
pub fn henge_walker() -> CardDefinition {
    let adamant = Predicate::SourceCastWithOneColorSpent { at_least: 3 };
    CardDefinition {
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred { pred: Box::new(adamant), then: Box::new(Value::ONE), else_: Box::new(Value::ZERO) },
        )),
        ..artifact_creature("Henge Walker", cost(&[generic(3)]), vec![CreatureType::Golem], 2, 2)
    }
}

/// Aerial Doombot — {U} 1/1 flying artifact Robot Villain. Power-up — {5}{U}:
/// three +1/+1 counters on it.
pub fn aerial_doombot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            power_up: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..artifact_creature("Aerial Doombot", cost(&[u()]), vec![CreatureType::Robot, CreatureType::Villain], 1, 1)
    }
}

/// Ninja of the Hand — {2}{B} 2/2 deathtouch Human Ninja Villain. Power-up —
/// {4}{B}: each opponent discards a card; a +1/+1 counter on it.
pub fn ninja_of_the_hand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            power_up: true,
            effect: Effect::Seq(vec![
                Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature("Ninja of the Hand", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Ninja, CreatureType::Villain], 2, 2)
    }
}

/// Bolg's Company — {B}{R} 2/2 Goblin Soldier. Haste while you control another
/// Goblin. {T}, sacrifice another Goblin: add {B}{R}.
pub fn bolgs_company() -> CardDefinition {
    let goblin = || R::HasCreatureType(CreatureType::Goblin);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has haste as long as you control another Goblin.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Haste,
                condition: Predicate::SelectorCountAtLeast { sel: mine(goblin().and(R::OtherThanSource)), n: Value::ONE },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((goblin(), 1)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black, Color::Red]) },
            ..Default::default()
        }],
        ..creature("Bolg's Company", cost(&[b(), r()]), vec![CreatureType::Goblin, CreatureType::Soldier], 2, 2)
    }
}

/// Hobgoblin Captain — {1}{R} 3/1 Goblin Barbarian. Pack tactics — attacks,
/// if you attacked with total power 6 or greater: first strike until end of
/// turn.
pub fn hobgoblin_captain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackedWithTotalPowerAtLeast { who: PlayerRef::You, at_least: 6 }),
            effect: grant(Selector::This, Keyword::FirstStrike),
        }],
        ..creature("Hobgoblin Captain", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Barbarian], 3, 1)
    }
}

/// Crater Elemental — {2}{R} 0/6 Elemental. {R}, {T}, sacrifice it: 4 damage
/// to target creature. Formidable — {2}{R}: base power 8 until end of turn,
/// only if creatures you control have total power 8 or greater.
pub fn crater_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                condition: Some(Predicate::FormidableActive { who: PlayerRef::You }),
                effect: Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::Const(8),
                    toughness: Value::ToughnessOf(Box::new(Selector::This)),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Crater Elemental", cost(&[generic(2), r()]), vec![CreatureType::Elemental], 0, 6)
    }
}

/// Gladewalker Ritualist — {2}{G} 3/3 changeling Shapeshifter. Whenever
/// another creature you control named Gladewalker Ritualist enters, draw a
/// card.
pub fn gladewalker_ritualist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::HasName("Gladewalker Ritualist".into())),
            }),
            effect: draw(1),
        }],
        ..creature("Gladewalker Ritualist", cost(&[generic(2), g()]), vec![CreatureType::Shapeshifter], 3, 3)
    }
}

// ── Artifacts, Auras, Enchantments ────────────────────────────────────────

/// Golden Urn — {1} Artifact. Your upkeep: you may put a charge counter on it.
/// {T}, sacrifice it: gain life equal to its charge counters.
pub fn golden_urn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put a charge counter on Golden Urn?".into(),
                body: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            // CR 602.2 / 608.2h — the counters are read last-known after the
            // sacrifice cost is paid.
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
            },
            ..Default::default()
        }],
        ..artifact("Golden Urn", cost(&[generic(1)]))
    }
}

/// Solitary Cell — {R}{W} Artifact. ETB: exile target nonland permanent an
/// opponent controls with mana value 3 or less until it leaves. {1}, {T},
/// discard a legendary card: draw a card.
pub fn solitary_cell() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent).and(R::ManaValueAtMost(3))),
            return_to: ExileReturnZone::Battlefield,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::HasSupertype(crate::card::Supertype::Legendary), 1)),
            effect: draw(1),
            ..Default::default()
        }],
        ..artifact("Solitary Cell", cost(&[r(), w()]))
    }
}

/// Barbed Bloodletter — {1}{B} Equipment, flash, equip {2}. ETB: attach it to
/// target creature you control; that creature gains wither until end of
/// turn. +1/+2.
pub fn barbed_bloodletter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            grant(Selector::Target(0), Keyword::Wither),
        ]))],
        ..equipment(
            "Barbed Bloodletter",
            cost(&[generic(1), b()]),
            cost(&[generic(2)]),
            EquipBonus { power: 1, toughness: 2, ..Default::default() },
        )
    }
}

/// Indrik Umbra — {4}{G}{W} Aura. +4/+4, first strike, and all creatures able
/// to block it do so. Umbra armor.
pub fn indrik_umbra() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::UmbraArmor],
        ..aura(
            "Indrik Umbra",
            cost(&[generic(4), g(), w()]),
            EquipBonus { power: 4, toughness: 4, keywords: vec![Keyword::FirstStrike, Keyword::AllMustBlock], ..Default::default() },
        )
    }
}

/// Lunarch Mantle — {1}{W} Aura. +2/+2 and "{1}, sacrifice a permanent: this
/// creature gains flying until end of turn."
pub fn lunarch_mantle() -> CardDefinition {
    aura(
        "Lunarch Mantle",
        cost(&[generic(1), w()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_filter: Some((R::Permanent, 1)),
                sac_other_may_be_source: true,
                effect: grant(Selector::This, Keyword::Flying),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Fable of Wolf and Owl — {3}{G/U}{G/U}{G/U} Enchantment. Cast a green
/// spell: you may create a 2/2 green Wolf. Cast a blue spell: you may create
/// a 1/1 blue flying Bird.
pub fn fable_of_wolf_and_owl() -> CardDefinition {
    let gu = || crate::mana::hybrid(Color::Green, Color::Blue);
    let on_color = |c: Color, what: &'static str, t: Arc<TokenDefinition>| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(c) }),
        effect: Effect::MayDo { description: what.into(), body: Box::new(mint(1, t)) },
    };
    CardDefinition {
        triggered_abilities: vec![
            on_color(Color::Green, "Create a 2/2 green Wolf?", token("Wolf", 2, 2, vec![Color::Green], vec![CreatureType::Wolf], vec![])),
            on_color(
                Color::Blue,
                "Create a 1/1 blue Bird with flying?",
                token("Bird", 1, 1, vec![Color::Blue], vec![CreatureType::Bird], vec![Keyword::Flying]),
            ),
        ],
        ..enchantment("Fable of Wolf and Owl", cost(&[generic(3), gu(), gu(), gu()]))
    }
}

/// Goblinslide — {2}{R} Enchantment. Cast a noncreature spell: you may pay
/// {1}; if you do, a 1/1 red Goblin with haste.
pub fn goblinslide() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.negate() }),
            effect: Effect::MayPay {
                description: "Pay {1} to create a 1/1 red Goblin with haste?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(mint(1, token("Goblin", 1, 1, vec![Color::Red], vec![CreatureType::Goblin], vec![Keyword::Haste]))),
                else_: None,
            },
        }],
        ..enchantment("Goblinslide", cost(&[generic(2), r()]))
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Join Shields — {3}{G}{W} Instant. Untap all creatures you control; they
/// gain hexproof and indestructible until end of turn.
pub fn join_shields() -> CardDefinition {
    spell(
        "Join Shields",
        cost(&[generic(3), g(), w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Untap { what: mine(R::Creature), up_to: None },
            grant(mine(R::Creature), Keyword::Hexproof),
            grant(mine(R::Creature), Keyword::Indestructible),
        ]),
    )
}

/// Gift of Growth — {1}{G} Instant, kicker {2}. Untap target creature; +2/+2
/// (+4/+4 if kicked) until end of turn.
pub fn gift_of_growth() -> CardDefinition {
    let amount = || Value::IfPred { pred: Box::new(Predicate::SpellWasKicked), then: Box::new(Value::Const(4)), else_: Box::new(Value::Const(2)) };
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2)]))],
        ..spell(
            "Gift of Growth",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                Effect::PumpPT { what: Selector::Target(0), power: amount(), toughness: amount(), duration: Duration::EndOfTurn },
            ]),
        )
    }
}

/// Board the Weatherlight — {1}{W} Sorcery. Look at the top five; you may
/// reveal a historic card and put it into your hand; the rest go to the
/// bottom in a random order.
pub fn board_the_weatherlight() -> CardDefinition {
    spell(
        "Board the Weatherlight",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::Const(5),
            pick_filter: Some(R::historic()),
            optional: true,
            ..Default::default()
        })),
    )
}

/// Eclipsed Boggart — {B/R}{B/R}{B/R} 2/3 Goblin Scout. ETB: look at the top
/// four; you may reveal a Goblin, Swamp, or Mountain card and put it into
/// your hand; the rest go to the bottom in a random order.
pub fn eclipsed_boggart() -> CardDefinition {
    let br = || crate::mana::hybrid(Color::Black, Color::Red);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(
                R::HasCreatureType(CreatureType::Goblin)
                    .or(R::HasLandType(LandType::Swamp))
                    .or(R::HasLandType(LandType::Mountain)),
            ),
            optional: true,
            ..Default::default()
        })))],
        ..creature("Eclipsed Boggart", cost(&[br(), br(), br()]), vec![CreatureType::Goblin, CreatureType::Scout], 2, 3)
    }
}

/// Safewright Quest — {G/W} Sorcery. Search your library for a Forest or
/// Plains card, reveal it, put it into your hand.
pub fn safewright_quest() -> CardDefinition {
    spell(
        "Safewright Quest",
        cost(&[crate::mana::hybrid(Color::Green, Color::White)]),
        CardType::Sorcery,
        Effect::Search {
            who: PlayerRef::You,
            filter: R::HasLandType(LandType::Forest).or(R::HasLandType(LandType::Plains)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// You Find a Cursed Idol — {1}{G} Sorcery. Choose one — destroy target
/// artifact; destroy target enchantment; or create a Treasure and venture
/// into the dungeon.
pub fn you_find_a_cursed_idol() -> CardDefinition {
    spell(
        "You Find a Cursed Idol",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::Seq(vec![crate::effect::shortcut::mint_treasures(1), Effect::Venture]),
        ]),
    )
}
