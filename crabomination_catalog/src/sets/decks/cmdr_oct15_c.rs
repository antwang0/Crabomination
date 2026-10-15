//! Commander: short-text two- and three-card gaps of the 2026-10-15 evening
//! rescan. Tests in `tests/recent_b/cmdr_oct15_c.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, evolve, on_attack, on_other_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, hybrid, r, u, w, x};
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

fn legendary(d: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..d }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn token(name: &str, p: i32, t: i32, color: Option<Color>, types: Vec<CreatureType>, artifact: bool) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: if artifact { vec![CardType::Artifact, CardType::Creature] } else { vec![CardType::Creature] },
        colors: color.into_iter().collect(),
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn at_your(step: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl), effect }
}

fn make(n: i32, t: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(t) }
}

fn saproling() -> TokenDefinition {
    token("Saproling", 1, 1, Some(Color::Green), vec![CreatureType::Saproling], false)
}

fn drain_one() -> Effect {
    Effect::Seq(vec![
        Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        Effect::GainLife { who: Selector::You, amount: Value::ONE },
    ])
}

fn tapped_self(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource), effect }
}

fn counters_on_another_of_yours(n: i32) -> Effect {
    Effect::AddCounter {
        what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    }
}

// ── Picard, Steadfast Captain ───────────────────────────────────────────────

/// Picard, Steadfast Captain — {4}{W}{W} legendary 4/4 Human Officer. Entering
/// or attacking puts two +1/+1 counters on another creature you control.
pub fn picard_steadfast_captain() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![etb(counters_on_another_of_yours(2)), on_attack(counters_on_another_of_yours(2))],
        ..creature("Picard, Steadfast Captain", cost(&[generic(4), w(), w()]), vec![CreatureType::Human, CreatureType::Officer], 4, 4)
    })
}

/// Will Riker, Assuming Command — {4}{W} legendary 4/5 Human Officer. Enters:
/// a +1/+1 counter on another creature you control.
pub fn will_riker_assuming_command() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![etb(counters_on_another_of_yours(1))],
        ..creature("Will Riker, Assuming Command", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Officer], 4, 5)
    })
}

/// Worf, Chief Tactical Officer — {2}{W} legendary 2/3 vigilance Klingon
/// Warrior Officer.
pub fn worf_chief_tactical_officer() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature(
            "Worf, Chief Tactical Officer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            2,
            3,
        )
    })
}

// ── The Mary Janes ──────────────────────────────────────────────────────────

/// Eardrum Rattler — {1}{R} 2/2 Human Bard. {1}, {T}: another creature you
/// control with power 2 or less can't be blocked this turn.
pub fn eardrum_rattler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2)).and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Eardrum Rattler", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Bard], 2, 2)
    }
}

/// Fíli and Kíli, Joyous — {2}{R} legendary 3/3 haste Dwarf Bard. {T}: add
/// {R}{R}, spent only on Dwarf, Equipment, and Saga spells.
pub fn fili_and_kili_joyous() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Red, Color::Red])),
                    SpendRestriction::DwarfEquipmentOrSagaSpells,
                ),
            },
            ..Default::default()
        }],
        ..creature("Fíli and Kíli, Joyous", cost(&[generic(2), r()]), vec![CreatureType::Dwarf, CreatureType::Bard], 3, 3)
    })
}

// ── Badgey, Malicious Glitch ────────────────────────────────────────────────

/// Badgey, Malicious Glitch — {3}{B} legendary 4/3 artifact Illusion Horror.
/// Becoming tapped: each opponent loses 1, you gain 1.
pub fn badgey_malicious_glitch() -> CardDefinition {
    legendary(CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![tapped_self(drain_one())],
        ..creature("Badgey, Malicious Glitch", cost(&[generic(3), b()]), vec![CreatureType::Illusion, CreatureType::Horror], 4, 3)
    })
}

/// Bio-Asset Allocator — {3}{B}{B} 3/4 artifact Borg. Your artifact creatures
/// have "attacking: each opponent loses 1, you gain 1."
pub fn bio_asset_allocator() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have \"Whenever this creature attacks, each opponent loses 1 life and you gain 1 life.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Artifact.and(R::Creature).and(R::ControlledByYou),
                ability: Box::new(on_attack(drain_one())),
            },
        }],
        ..creature("Bio-Asset Allocator", cost(&[generic(3), b(), b()]), vec![CreatureType::Borg], 3, 4)
    }
}

// ── Quake, Agent of S.H.I.E.L.D. ────────────────────────────────────────────

/// Super Villain Lockup — {1}{W} flash enchantment. Enters: exile a tapped
/// creature an opponent controls until it leaves.
pub fn super_villain_lockup() -> CardDefinition {
    CardDefinition {
        name: "Super Villain Lockup",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::Tapped).and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Don't Move — {3}{W}{W} sorcery. Destroy all tapped creatures; until your
/// next turn, a creature that becomes tapped is destroyed (CR 611.2b).
pub fn dont_move() -> CardDefinition {
    spell(
        "Don't Move",
        cost(&[generic(3), w(), w()]),
        false,
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Tapped)) },
            Effect::TriggersUntilYourNextTurn {
                triggered: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                    effect: Effect::Destroy { what: Selector::TriggerSource },
                }],
            },
        ]),
    )
}

// ── Curie, Emergent Intelligence ────────────────────────────────────────────

/// Mordenkainen's Polymorph — {1}{U} instant. Until end of turn a creature
/// becomes a 4/4 flying Dragon (losing its other creature types).
pub fn mordenkainens_polymorph() -> CardDefinition {
    spell(
        "Mordenkainen's Polymorph",
        cost(&[generic(1), u()]),
        true,
        Effect::BecomeCreature {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(4),
            creature_types: vec![CreatureType::Dragon],
            keywords: vec![Keyword::Flying],
            duration: Duration::EndOfTurn,
        },
    )
}

/// Perfected Theory — {U} instant. A creature has base P/T 1/1, or 4/5, until
/// end of turn.
pub fn perfected_theory() -> CardDefinition {
    let base = |p: i32, t: i32| Effect::SetBasePT {
        what: target_filtered(R::Creature),
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    };
    spell("Perfected Theory", cost(&[u()]), true, Effect::ChooseMode(vec![base(1, 1), base(4, 5)]))
}

// ── Shang-Chi, Martial Mentor ───────────────────────────────────────────────

/// Hulk, Strongest There Is — {2}{G}{G} legendary 3/3 trample Gamma Berserker
/// Hero; enters with a +1/+1 counter; at your upkeep, double the +1/+1
/// counters on each Gamma creature you control.
pub fn hulk_strongest_there_is() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::DoubleCountersOnEach {
                what: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Gamma)).and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
            },
        )],
        ..creature(
            "Hulk, Strongest There Is",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Gamma, CreatureType::Berserker, CreatureType::Hero],
            3,
            3,
        )
    })
}

/// Hulkling, Burgeoning Bruiser — {2}{G} legendary 2/3 vigilance Kree Skrull
/// Hero. Another creature of yours entering bigger in power or toughness puts
/// a +1/+1 counter on it (evolve's trigger, CR 702.100).
pub fn hulkling_burgeoning_bruiser() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![evolve()],
        ..creature(
            "Hulkling, Burgeoning Bruiser",
            cost(&[generic(2), g()]),
            vec![CreatureType::Kree, CreatureType::Skrull, CreatureType::Hero],
            2,
            3,
        )
    })
}

/// Advancing the Spirit — {2}{G} enchantment. Enters: draw a card. The first
/// power-up you activate each of your turns may cost {0} (CR 702.193).
pub fn advancing_the_spirit() -> CardDefinition {
    CardDefinition {
        name: "Advancing the Spirit",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        static_abilities: vec![StaticAbility {
            description: "You may pay {0} rather than pay the power-up cost of the first power-up ability you activate during each of your turns.",
            effect: StaticEffect::FirstPowerUpEachYourTurnFree,
        }],
        ..Default::default()
    }
}

// ── General Chang, Cold Warrior ─────────────────────────────────────────────

/// General Chang, Cold Warrior — {B} legendary 1/2 Klingon Warrior Officer.
/// {1}, sacrifice a creature: a +1/+1 counter on each Klingon you control.
pub fn general_chang_cold_warrior() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_may_be_source: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Klingon).and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "General Chang, Cold Warrior",
            cost(&[b()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            1,
            2,
        )
    })
}

/// Battle-Scarred Survivalist — {1}{B} 1/1 menace Klingon Scout. Another
/// creature dying puts a +1/+1 counter on it, once each turn.
pub fn battle_scarred_survivalist() -> CardDefinition {
    let mut grow = on_other_dies(Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    });
    grow.event = grow.event.once_per_turn();
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![grow],
        ..creature("Battle-Scarred Survivalist", cost(&[generic(1), b()]), vec![CreatureType::Klingon, CreatureType::Scout], 1, 1)
    }
}

/// Rooftop Percher — {5} 3/3 changeling flying Shapeshifter. Enters: exile up
/// to two cards from graveyards; you gain 3 life.
pub fn rooftop_percher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExileUpToNFromGraveyards { count: Value::Const(2), of: None, single: false },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..creature("Rooftop Percher", cost(&[generic(5)]), vec![CreatureType::Shapeshifter], 3, 3)
    }
}

// ── Nemata, Primeval Warden ─────────────────────────────────────────────────

/// Sporesower Thallid — {2}{G}{G} 4/4 Fungus. At your upkeep, a spore counter
/// on each Fungus you control; remove three: a 1/1 Saproling.
pub fn sporesower_thallid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::AddCounter {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Fungus).and(R::ControlledByYou)),
                kind: CounterType::Spore,
                amount: Value::ONE,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Spore, 3)),
            effect: make(1, saproling()),
            ..Default::default()
        }],
        ..creature("Sporesower Thallid", cost(&[generic(2), g(), g()]), vec![CreatureType::Fungus], 4, 4)
    }
}

/// Sporoloth Ancient — {3}{G}{G} 4/4 Fungus. A spore counter at your upkeep;
/// your creatures have "remove two spore counters: a 1/1 Saproling."
pub fn sporoloth_ancient() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your(
            TurnStep::Upkeep,
            Effect::AddCounter { what: Selector::This, kind: CounterType::Spore, amount: Value::ONE },
        )],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have \"Remove two spore counters from this creature: Create a 1/1 green Saproling creature token.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    remove_counter_cost: Some((CounterType::Spore, 2)),
                    effect: make(1, saproling()),
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..creature("Sporoloth Ancient", cost(&[generic(3), g(), g()]), vec![CreatureType::Fungus], 4, 4)
    }
}

/// Saproling Migration — {1}{G} sorcery, kicker {4}. Two 1/1 Saprolings, or
/// four if kicked.
pub fn saproling_migration() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        ..spell(
            "Saproling Migration",
            cost(&[generic(1), g()]),
            false,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(4)),
                    else_: Box::new(Value::Const(2)),
                },
                definition: Arc::new(saproling()),
            },
        )
    }
}

// ── Oviya Pashiri, Sage Lifecrafter ─────────────────────────────────────────

/// Oviya Pashiri, Sage Lifecrafter — {G} legendary 1/2 Human Artificer.
/// {2}{G}, {T}: a 1/1 Servo. {4}{G}, {T}: an X/X Construct, X your creatures.
pub fn oviya_pashiri_sage_lifecrafter() -> CardDefinition {
    let yours = || Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))));
    let construct = TokenDefinition {
        dynamic_pt: Some((yours(), yours())),
        ..token("Construct", 0, 0, None, vec![CreatureType::Construct], true)
    };
    legendary(CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                tap_cost: true,
                effect: make(1, token("Servo", 1, 1, None, vec![CreatureType::Servo], true)),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), g()]),
                tap_cost: true,
                effect: make(1, construct),
                ..Default::default()
            },
        ],
        ..creature("Oviya Pashiri, Sage Lifecrafter", cost(&[g()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 2)
    })
}

/// Neverwinter Dryad — {G} 1/1 Dryad. {2}, sacrifice it: a basic Forest onto
/// the battlefield tapped.
pub fn neverwinter_dryad() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand.and(R::HasLandType(crate::card::LandType::Forest)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Neverwinter Dryad", cost(&[g()]), vec![CreatureType::Dryad], 1, 1)
    }
}

/// Shapers' Sanctuary — {G} enchantment. An opponent targeting your creature:
/// you may draw a card.
pub fn shapers_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Shapers' Sanctuary",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourPermanentTargetedByOpponent).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

// ── Spider-Gwen, Free Spirit ────────────────────────────────────────────────

/// Thrór's Map — {2} legendary artifact. Enters: a basic land to hand. {2},
/// {T}: draw, then discard.
pub fn thrors_map() -> CardDefinition {
    CardDefinition {
        name: "Thrór's Map",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Search { who: PlayerRef::You, filter: R::IsBasicLand, to: ZoneDest::Hand(PlayerRef::You) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spireside Infiltrator — {2}{R} 3/2 Human Rogue. Becoming tapped: 1 damage
/// to each opponent.
pub fn spireside_infiltrator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![tapped_self(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
        })],
        ..creature("Spireside Infiltrator", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Rogue], 3, 2)
    }
}

/// Senseless Rage — {1}{R} Aura, +2/+2; madness {1}{R}.
pub fn senseless_rage() -> CardDefinition {
    CardDefinition {
        name: "Senseless Rage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Madness(cost(&[generic(1), r()]))],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        ..Default::default()
    }
}

// ── Hercules, Olympian Hero / Strong, the Brutish Thespian ──────────────────

/// Arena — land. {3}, {T}: tap a creature you control and a creature of an
/// opponent's choice they control; they fight (Magus of the Arena's ability).
pub fn arena() -> CardDefinition {
    CardDefinition {
        name: "Arena",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Tap { what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) } },
                Effect::OpponentChoosesPermanentThen {
                    filter: R::Creature,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::Fight { attacker: Selector::Target(1), defender: Selector::Target(0) },
                    ])),
                    chooser: None,
                    theirs: true,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ent's Fury — {1}{G} sorcery. A +1/+1 counter on your creature if its power
/// is 4 or more; then it gets +1/+1 and fights a creature you don't control.
pub fn ents_fury() -> CardDefinition {
    let yours = || Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) };
    spell(
        "Ent's Fury",
        cost(&[generic(1), g()]),
        false,
        Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: yours(), filter: R::PowerAtLeast(4) },
                then: Box::new(Effect::AddCounter { what: yours(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
            Effect::PumpPT { what: yours(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::Fight {
                attacker: yours(),
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
            },
        ]),
    )
}

// ── Jet, Freedom Fighter ────────────────────────────────────────────────────

/// Hook Swords — {2}{R/W} Equipment. Enters: a 1/1 white Ally, suited up.
/// During your turn, +1/+1 and first strike. Equip {3}.
pub fn hook_swords() -> CardDefinition {
    CardDefinition {
        name: "Hook Swords",
        cost: cost(&[generic(2), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(1, token("Ally", 1, 1, Some(Color::White), vec![CreatureType::Ally], false)),
            Effect::AttachSourceTo { host: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(EquipBonus {
            during_your_turn_pt: (1, 1),
            during_your_turn_keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Freedom Fighter Recruit — {1}{R} */2 Human Rebel Ally; power is the number
/// of creatures you control.
pub fn freedom_fighter_recruit() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(crate::card::DynamicPt::CreaturesYouControl { base_t: 2 }),
        ..creature(
            "Freedom Fighter Recruit",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            0,
            2,
        )
    }
}

// ── Lady Caleria / Tobias, Doomed Conqueror ─────────────────────────────────

/// Coordinated Barrage — {W} instant. Choose a creature type; damage to an
/// attacking or blocking creature equal to your permanents of that type.
pub fn coordinated_barrage() -> CardDefinition {
    spell(
        "Coordinated Barrage",
        cost(&[w()]),
        true,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::ControlledByYou.and(R::IsSourceChosenCreatureType),
                ))),
            }),
        },
    )
}

/// Rally the Ancestors — {X}{W}{W} instant. Your creature cards of mana value
/// X or less return; they're exiled at your next upkeep. Exile Rally.
pub fn rally_the_ancestors() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Rally the Ancestors",
            cost(&[x(), w(), w()]),
            true,
            Effect::Seq(vec![
                Effect::Move {
                    what: Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                        filter: R::Creature.and(R::ManaValueAtMostXFromCost),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::ExileAtYourNextUpkeep { what: Selector::LastMoved },
            ]),
        )
    }
}

// ── Absorbing Man and Titania ───────────────────────────────────────────────

/// Absorbing Man and Titania — {3}{R}{G} legendary 4/5 Human Villain. Your
/// creature sources deal double damage.
pub fn absorbing_man_and_titania() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Double all damage that creature sources you control would deal.",
            effect: StaticEffect::DoubleDamageFromControlledMatching { filter: R::Creature },
        }],
        ..creature("Absorbing Man and Titania", cost(&[generic(3), r(), g()]), vec![CreatureType::Human, CreatureType::Villain], 4, 5)
    })
}

/// Charging Tuskodon — {3}{R}{R} 4/4 trample Dinosaur; its combat damage to a
/// player is doubled.
pub fn charging_tuskodon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If this creature would deal combat damage to a player, it deals double that damage to that player instead.",
            effect: StaticEffect::SelfDealsDoubleCombatDamageToPlayers,
        }],
        ..creature("Charging Tuskodon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dinosaur], 4, 4)
    }
}

/// Patriot, Young Avenger — {2}{R/W} legendary 3/2 prowess Human Hero. While
/// equipped, other creatures you control get +1/+0.
pub fn patriot_young_avenger() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "As long as Patriot is equipped, other creatures you control get +1/+0.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SourceIsEquipped,
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    power: 1,
                    toughness: 0,
                }),
            },
        }],
        ..creature(
            "Patriot, Young Avenger",
            cost(&[generic(2), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Human, CreatureType::Hero],
            3,
            2,
        )
    })
}

/// Agent Maria Hill — {W} legendary 2/1 Human Spy Hero. Tapped to pay a
/// teamwork cost: a +1/+1 counter on her and draw a card.
pub fn agent_maria_hill() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(crate::effect::KeywordAct::TappedForTeamwork), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource }),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature("Agent Maria Hill", cost(&[w()]), vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero], 2, 1)
    })
}
