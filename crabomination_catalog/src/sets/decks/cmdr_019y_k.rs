//! Commander: 2026-10-17 (`019Y6J4R`) closers, eleventh batch — each the one
//! missing card of an otherwise-buildable EDHREC average deck. Tests in
//! `tests/recent_b/cmdr_019y_k.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_dies, spell_mastery_gate, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
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

fn aura(name: &'static str, mana: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..Default::default()
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// "Counter target spell unless its controller pays {N}."
fn counter_unless(n: u32) -> Effect {
    Effect::CounterUnlessPaid {
        what: target_filtered(R::IsSpellOnStack),
        mana_cost: cost(&[generic(n)]),
        exile: false,
        extra_generic: None,
        if_paid: None,
    }
}

/// "Activate only during your turn, before attackers are declared."
fn your_turn_before_attackers() -> Predicate {
    Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::Any(vec![
            Predicate::CurrentStepIs(TurnStep::Upkeep),
            Predicate::CurrentStepIs(TurnStep::Draw),
            Predicate::CurrentStepIs(TurnStep::PreCombatMain),
            Predicate::CurrentStepIs(TurnStep::BeginCombat),
        ]),
    ])
}

/// "At the beginning of your end step, if …" (an intervening if, CR 603.4).
fn your_end_step_if(cond: Predicate, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(cond),
        effect,
    }
}

/// Whisper Agent — {1}{U/B}{U/B} 3/2 flash Human Rogue. ETB: surveil 1.
pub fn whisper_agent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        ..creature(
            "Whisper Agent",
            cost(&[generic(1), hybrid(Color::Blue, Color::Black), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Human, CreatureType::Rogue],
            3,
            2,
        )
    }
}

/// Scribe of the Mindful — {2}{U} 2/2 Human Cleric. {1}, {T}, sacrifice it:
/// return target instant or sorcery card from your graveyard to your hand.
pub fn scribe_of_the_mindful() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    R::InGraveyard
                        .and(R::OwnedByYou)
                        .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature("Scribe of the Mindful", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Deadbridge Shaman — {2}{B} 3/1 Elf Shaman. Dies: target opponent discards.
pub fn deadbridge_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Discard {
            who: target_filtered(R::OpponentPlayer),
            amount: Value::ONE,
            random: false,
        })],
        ..creature("Deadbridge Shaman", cost(&[generic(2), b()]), vec![CreatureType::Elf, CreatureType::Shaman], 3, 1)
    }
}

/// Noxious Dragon — {4}{B}{B} 4/4 flying Dragon. Dies: you may destroy target
/// creature with mana value 3 or less.
pub fn noxious_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.and(R::ManaValueAtMost(3)),
            effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        })],
        ..creature("Noxious Dragon", cost(&[generic(4), b(), b()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Wild Wanderer — {3}{G} 3/2 Elf Druid. ETB: you may search for a basic land
/// card, put it onto the battlefield tapped.
pub fn wild_wanderer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search your library for a basic land?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        })],
        ..creature("Wild Wanderer", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Druid], 3, 2)
    }
}

/// Weaver of Currents — {1}{G}{U} 2/2 Snake Druid. {T}: add {C}{C}.
pub fn weaver_of_currents() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
            ..Default::default()
        }],
        ..creature("Weaver of Currents", cost(&[generic(1), g(), u()]), vec![CreatureType::Snake, CreatureType::Druid], 2, 2)
    }
}

/// Contract Killing — {3}{B}{B} sorcery. Destroy target creature; create two
/// Treasure tokens.
pub fn contract_killing() -> CardDefinition {
    spell(
        "Contract Killing",
        cost(&[generic(3), b(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        ]),
    )
}

/// Aerial Guide — {2}{U} 2/2 flying Drake. Attacks: another target attacking
/// creature gains flying until end of turn.
pub fn aerial_guide() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Aerial Guide", cost(&[generic(2), u()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Viashino Branchrider — {R} 1/1 haste Lizard Warrior, kicker {2}{G}: kicked,
/// it enters with two +1/+1 counters. {2}{R}: +2/+0 until end of turn.
pub fn viashino_branchrider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Kicker(cost(&[generic(2), g()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Viashino Branchrider", cost(&[r()]), vec![CreatureType::Lizard, CreatureType::Warrior], 1, 1)
    }
}

/// Lion Vulture — {3}{B} 2/2 flying Cat Bird. Your end step, if an opponent
/// lost life this turn: a +1/+1 counter and draw a card.
pub fn lion_vulture() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![your_end_step_if(
            Predicate::PlayerLostLifeThisTurn { who: PlayerRef::EachOpponent },
            Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                draw(1),
            ]),
        )],
        ..creature("Lion Vulture", cost(&[generic(3), b()]), vec![CreatureType::Cat, CreatureType::Bird], 2, 2)
    }
}

/// Soul Sear — {2}{R} instant. 5 damage to target creature or planeswalker;
/// it loses indestructible until end of turn.
pub fn soul_sear() -> CardDefinition {
    spell(
        "Soul Sear",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::LoseKeyword {
                what: target_filtered(R::Creature.or(R::Planeswalker)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) },
        ]),
    )
}

/// Sage's Dousing — {2}{U} Kindred instant — Wizard. Counter target spell
/// unless its controller pays {3}; if you control a Wizard, draw a card.
pub fn sages_dousing() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        ..spell(
            "Sage's Dousing",
            cost(&[generic(2), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                counter_unless(3),
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::HasCreatureType(CreatureType::Wizard),
                    }),
                    then: Box::new(draw(1)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Calculated Dismissal — {2}{U} instant. Counter target spell unless its
/// controller pays {3}. Spell mastery — scry 2.
pub fn calculated_dismissal() -> CardDefinition {
    spell(
        "Calculated Dismissal",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            counter_unless(3),
            Effect::If {
                cond: spell_mastery_gate(),
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Ruin Raider — {2}{B} 3/2 Orc Pirate. Raid — your end step, if you attacked
/// this turn: reveal the top card, put it into your hand, lose life equal to
/// its mana value.
pub fn ruin_raider() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![your_end_step_if(
            Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
            Effect::Seq(vec![
                // The mana value is read before the card leaves the library.
                Effect::LoseLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(top())) },
                Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) },
            ]),
        )],
        ..creature("Ruin Raider", cost(&[generic(2), b()]), vec![CreatureType::Orc, CreatureType::Pirate], 3, 2)
    }
}

/// Sifter Wurm — {5}{G}{G} 7/7 trample Wurm. ETB: scry 3, then reveal the top
/// card; gain life equal to its mana value.
pub fn sifter_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE })),
            },
        ]))],
        ..creature("Sifter Wurm", cost(&[generic(5), g(), g()]), vec![CreatureType::Wurm], 7, 7)
    }
}

/// Armor of Faith — {W} Aura. Enchanted creature gets +1/+1. {W}: enchanted
/// creature gets +0/+1 until end of turn.
pub fn armor_of_faith() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::attached_to(Selector::This),
                power: Value::Const(0),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..aura("Armor of Faith", cost(&[w()]), R::Creature)
    }
}

/// Aether Meltdown — {1}{U} flash Aura, enchant creature or Vehicle. ETB: you
/// get {E}{E}. Enchanted creature gets -4/-0.
pub fn aether_meltdown() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(2)))],
        equipped_bonus: Some(EquipBonus { power: -4, ..Default::default() }),
        ..aura(
            "Aether Meltdown",
            cost(&[generic(1), u()]),
            R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
        )
    }
}

/// Stasis Field — {1}{U} Aura. Enchanted creature has base power and
/// toughness 0/2, has defender, and loses all other abilities.
pub fn stasis_field() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            set_base_pt: Some((0, 2)),
            keywords: vec![Keyword::Defender],
            remove_abilities: true,
            ..Default::default()
        }),
        ..aura("Stasis Field", cost(&[generic(1), u()]), R::Creature)
    }
}

/// Lay Claim — {5}{U}{U} Aura, enchant permanent. You control enchanted
/// permanent. Cycling {2}.
pub fn lay_claim() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::attached_to(Selector::This),
        })],
        ..aura("Lay Claim", cost(&[generic(5), u(), u()]), R::Permanent)
    }
}

/// Enthralling Hold — {3}{U}{U} Aura. You can't choose an untapped creature
/// as its target as you cast it; you control enchanted creature.
pub fn enthralling_hold() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::attached_to(Selector::This),
        })],
        ..aura("Enthralling Hold", cost(&[generic(3), u(), u()]), R::Creature.and(R::Tapped))
    }
}

/// Coastal Wizard — {2}{U}{U} 1/1 Human Wizard. {T}: return it and another
/// target creature to their owners' hands; your turn, before attackers.
pub fn coastal_wizard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(your_turn_before_attackers()),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ]),
            ..Default::default()
        }],
        ..creature("Coastal Wizard", cost(&[generic(2), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 1)
    }
}

/// Puresight Merrow — {W/U}{W/U} 2/2 Merfolk Wizard. {W/U}, {Q}: look at the
/// top card of your library; you may exile it.
pub fn puresight_merrow() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[wu()]),
            untap_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::LookAtTop { who: PlayerRef::You, amount: Value::ONE },
                Effect::MayDo {
                    description: "Exile the top card of your library?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::Exile,
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Puresight Merrow", cost(&[wu(), wu()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 2)
    }
}

/// Ardent Electromancer — {2}{R} 3/2 Human Wizard. ETB: add {R} for each
/// creature in your party.
pub fn ardent_electromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Red, Value::PartyCount),
        })],
        ..creature("Ardent Electromancer", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 2)
    }
}

#[allow(unused)]
fn _unused(_: Zone, _: Supertype) {}
