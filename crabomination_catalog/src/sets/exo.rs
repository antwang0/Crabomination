//! Exodus (EXO) — the Tempest block's third set. Shadow, buyback, the Oath
//! cycle's catch-up enchantments and the Keeper cycle. Tests in
//! `classic_sets/exo`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{deal, draw, gain_life, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

/// An Aura whose whole text is a static rider on the enchanted creature.
fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

// ── ETB recursion (the "you may return target X card" cycle) ────────────────

fn etb_recur(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    filter: R,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: format!("Return a card from your graveyard with {name}?"),
                body: Box::new(Effect::Move {
                    what: target_filtered(filter.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature(name, c, types, 2, 2)
    }
}

/// Anarchist — {4}{R} 2/2. ETB: may return a sorcery card from your graveyard.
pub fn anarchist() -> CardDefinition {
    etb_recur(
        "Anarchist",
        cost(&[generic(4), r()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        R::HasCardType(CardType::Sorcery),
    )
}

/// Cartographer — {2}{G} 2/2. ETB: may return a land card from your graveyard.
pub fn cartographer() -> CardDefinition {
    etb_recur("Cartographer", cost(&[generic(2), g()]), vec![CreatureType::Human], R::Land)
}

/// Scrivener — {4}{U} 2/2. ETB: may return an instant card from your graveyard.
pub fn scrivener() -> CardDefinition {
    etb_recur(
        "Scrivener",
        cost(&[generic(4), u()]),
        vec![CreatureType::Human, CreatureType::Wizard],
        R::HasCardType(CardType::Instant),
    )
}

// ── Shadow (the Dauthi) ─────────────────────────────────────────────────────

/// Dauthi Cutthroat — {1}{B} 1/1 shadow that snipes other shadow creatures.
pub fn dauthi_cutthroat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Shadow))),
            },
            ..Default::default()
        }],
        ..creature(
            "Dauthi Cutthroat",
            cost(&[generic(1), b()]),
            vec![CreatureType::Dauthi, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Dauthi Jackal — {2}{B} 2/1 shadow. Sacrifice it to kill a blocker.
pub fn dauthi_jackal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), b()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::IsBlocking)) },
            ..Default::default()
        }],
        ..creature(
            "Dauthi Jackal",
            cost(&[generic(2), b()]),
            vec![CreatureType::Dauthi, CreatureType::Jackal],
            2,
            1,
        )
    }
}

/// Dauthi Warlord — {1}{B} */1 shadow, as big as the shadow board.
pub fn dauthi_warlord() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        dynamic_pt: Some(DynamicPt::PermanentsOnBattlefieldMatching {
            base_p: 0,
            base_t: 1,
            filter: Box::new(R::Creature.and(R::HasKeyword(Keyword::Shadow))),
        }),
        ..creature(
            "Dauthi Warlord",
            cost(&[generic(1), b()]),
            vec![CreatureType::Dauthi, CreatureType::Soldier],
            0,
            1,
        )
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Cursed Flesh — {B} Aura. Enchanted creature gets -1/-1 and has fear.
pub fn cursed_flesh() -> CardDefinition {
    aura(
        "Cursed Flesh",
        cost(&[b()]),
        EquipBonus { power: -1, toughness: -1, keywords: vec![Keyword::Fear], ..Default::default() },
    )
}

/// Maniacal Rage — {1}{R} Aura. Enchanted creature gets +2/+2 and can't block.
pub fn maniacal_rage() -> CardDefinition {
    aura(
        "Maniacal Rage",
        cost(&[generic(1), r()]),
        EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::CantBlock],
            ..Default::default()
        },
    )
}

/// Robe of Mirrors — {U} Aura. Enchanted creature has shroud.
pub fn robe_of_mirrors() -> CardDefinition {
    aura(
        "Robe of Mirrors",
        cost(&[u()]),
        EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() },
    )
}

/// Bequeathal — {G} Aura. The enchanted creature's death draws you two.
pub fn bequeathal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: draw(2),
        }],
        ..aura("Bequeathal", cost(&[g()]), EquipBonus::default())
    }
}

// ── Buyback ─────────────────────────────────────────────────────────────────

/// Allay — {1}{W} Instant with buyback {3}. Destroy target enchantment.
pub fn allay() -> CardDefinition {
    CardDefinition {
        name: "Allay",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
        ..Default::default()
    }
}

/// Forbid — {1}{U}{U} Instant. Buyback—Discard two cards. Counter a spell.
pub fn forbid() -> CardDefinition {
    CardDefinition {
        name: "Forbid",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(ManaCost::new(vec![]))],
        buyback_additional_cost: Some(AdditionalCastCost::Discard { filter: None, count: 2 }),
        effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        ..Default::default()
    }
}

/// Pegasus Stampede — {1}{W} Sorcery. Buyback—Sacrifice a land.
pub fn pegasus_stampede() -> CardDefinition {
    CardDefinition {
        name: "Pegasus Stampede",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(ManaCost::new(vec![]))],
        buyback_additional_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Pegasus".to_string(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Pegasus],
                    ..Default::default()
                },
                keywords: vec![Keyword::Flying],
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Reaping the Rewards — {W} Instant. Buyback—Sacrifice a land. Gain 2 life.
pub fn reaping_the_rewards() -> CardDefinition {
    CardDefinition {
        name: "Reaping the Rewards",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(ManaCost::new(vec![]))],
        buyback_additional_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        effect: gain_life(2),
        ..Default::default()
    }
}

/// Memory Crystal — {3} Artifact. Buyback costs cost {2} less.
pub fn memory_crystal() -> CardDefinition {
    CardDefinition {
        name: "Memory Crystal",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Buyback costs cost {2} less.",
            effect: crate::effect::StaticEffect::BuybackCostsLess { amount: 2 },
        }],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Death's Duet — {2}{B} Sorcery. Return two creature cards from your yard.
pub fn deaths_duet() -> CardDefinition {
    CardDefinition {
        name: "Death's Duet",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature.and(R::InYourGraveyard),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Fugue — {3}{B}{B} Sorcery. Target player discards three cards.
pub fn fugue() -> CardDefinition {
    CardDefinition {
        name: "Fugue",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(3),
            random: false,
        },
        ..Default::default()
    }
}

/// Nausea — {1}{B} Sorcery. All creatures get -1/-1 until end of turn.
pub fn nausea() -> CardDefinition {
    CardDefinition {
        name: "Nausea",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Scare Tactics — {B} Instant. Creatures you control get +1/+0.
pub fn scare_tactics() -> CardDefinition {
    CardDefinition {
        name: "Scare Tactics",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Hatred — {3}{B}{B} Instant. Pay X life; target creature gets +X/+0.
pub fn hatred() -> CardDefinition {
    CardDefinition {
        name: "Hatred",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Instant],
        additional_cost_pay_x_life: true,
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::XFromCost,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Carnophage — {B} 2/2 that taps itself unless you bleed for it.
pub fn carnophage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::MayPayLife {
                description: "Pay 1 life to keep Carnophage untapped?".into(),
                amount: Value::ONE,
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Tap { what: Selector::This })),
            },
        }],
        ..creature("Carnophage", cost(&[b()]), vec![CreatureType::Zombie], 2, 2)
    }
}

/// School of Piranha — {1}{U} 3/3 with rent due every upkeep.
pub fn school_of_piranha() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1), u()]) },
        }],
        ..creature("School of Piranha", cost(&[generic(1), u()]), vec![CreatureType::Fish], 3, 3)
    }
}

/// Ephemeron — {4}{U}{U} 4/4 flier that pitches a card to save itself.
pub fn ephemeron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..creature("Ephemeron", cost(&[generic(4), u(), u()]), vec![CreatureType::Illusion], 4, 4)
    }
}

/// Ertai, Wizard Adept — {2}{U} 1/1 legend with a repeatable counterspell.
pub fn ertai_wizard_adept() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            tap_cost: true,
            effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            ..Default::default()
        }],
        ..creature(
            "Ertai, Wizard Adept",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Killer Whale — {3}{U}{U} 3/5 that can fly for {U}.
pub fn killer_whale() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Killer Whale", cost(&[generic(3), u(), u()]), vec![CreatureType::Whale], 3, 5)
    }
}

/// Mirri, Cat Warrior — {1}{G}{G} 2/3 with first strike, forestwalk, vigilance.
pub fn mirri_cat_warrior() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Landwalk(LandType::Forest),
            Keyword::Vigilance,
        ],
        ..creature(
            "Mirri, Cat Warrior",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Paladin en-Vec — {1}{W}{W} 2/2 first strike, pro-black and pro-red.
pub fn paladin_en_vec() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Protection(Color::Black),
            Keyword::Protection(Color::Red),
        ],
        ..creature(
            "Paladin en-Vec",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Sabertooth Wyvern — {4}{R} 3/2 flier with first strike.
pub fn sabertooth_wyvern() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..creature("Sabertooth Wyvern", cost(&[generic(4), r()]), vec![CreatureType::Drake], 3, 2)
    }
}

/// Exalted Dragon — {4}{W}{W} 5/5 flier that eats a land to attack.
pub fn exalted_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::AttackCostSacrifice(Box::new(R::Land), 1)],
        ..creature(
            "Exalted Dragon",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Plated Rootwalla — {4}{G} 3/3 with a once-a-turn +3/+3.
pub fn plated_rootwalla() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Plated Rootwalla", cost(&[generic(4), g()]), vec![CreatureType::Lizard], 3, 3)
    }
}

/// "Whenever this creature becomes blocked by a creature, it gets +1/+1 until
/// end of turn."
fn grows_when_blocked() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    }
}

/// Rabid Wolverines — {3}{G}{G} 4/4 that grows when blocked.
pub fn rabid_wolverines() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![grows_when_blocked()],
        ..creature(
            "Rabid Wolverines",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Wolverine],
            4,
            4,
        )
    }
}

/// Pygmy Troll — {1}{G} 1/1 that grows when blocked and regenerates.
pub fn pygmy_troll() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![grows_when_blocked()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Pygmy Troll", cost(&[generic(1), g()]), vec![CreatureType::Troll], 1, 1)
    }
}

/// Rootwater Alligator — {3}{G} 3/2 that eats Forests to survive.
pub fn rootwater_alligator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasLandType(LandType::Forest), 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Rootwater Alligator",
            cost(&[generic(3), g()]),
            vec![CreatureType::Crocodile],
            3,
            2,
        )
    }
}

/// Ravenous Baboons — {3}{R} 2/2. ETB: destroy target nonbasic land.
pub fn ravenous_baboons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(R::Land.and(R::IsBasicLand.negate())),
            },
        }],
        ..creature("Ravenous Baboons", cost(&[generic(3), r()]), vec![CreatureType::Monkey], 2, 2)
    }
}

/// Furnace Brood — {3}{R} 3/3 that turns off regeneration.
pub fn furnace_brood() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::CantBeRegeneratedThisTurn { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Furnace Brood", cost(&[generic(3), r()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Mage il-Vec — {2}{R} 2/2 that pitches a random card for a ping.
pub fn mage_il_vec() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: deal(1, target_any()),
            ..Default::default()
        }],
        ..creature(
            "Mage il-Vec",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Ogre Shaman — {3}{R}{R} 3/3 with a bigger, mana-fed version of the same.
pub fn ogre_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: deal(2, target_any()),
            ..Default::default()
        }],
        ..creature(
            "Ogre Shaman",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Ogre, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// Reckless Ogre — {3}{R} 3/2 that hits harder alone.
pub fn reckless_ogre() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttackingAlone,
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Reckless Ogre", cost(&[generic(3), r()]), vec![CreatureType::Ogre], 3, 2)
    }
}

/// Grollub — {2}{B} 3/3 whose every wound is life for the table.
pub fn grollub() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Grollub", cost(&[generic(2), b()]), vec![CreatureType::Beast], 3, 3)
    }
}

// ── Artifacts / enchantments ────────────────────────────────────────────────

/// Medicine Bag — {3} Artifact. Pitch a card to regenerate a creature.
pub fn medicine_bag() -> CardDefinition {
    CardDefinition {
        name: "Medicine Bag",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mindless Automaton — {4} 0/0 that eats cards to grow and counters to draw.
pub fn mindless_automaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                discard_cost: Some((R::Any, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 2)),
                effect: draw(1),
                ..Default::default()
            },
        ],
        ..creature("Mindless Automaton", cost(&[generic(4)]), vec![CreatureType::Construct], 0, 0)
    }
}

/// Peace of Mind — {1}{W} Enchantment. Trade cards for life, repeatedly.
pub fn peace_of_mind() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            discard_cost: Some((R::Any, 1)),
            effect: gain_life(3),
            ..Default::default()
        }],
        ..enchantment("Peace of Mind", cost(&[generic(1), w()]))
    }
}

/// Convalescence — {1}{W} Enchantment. A trickle of life while you're low.
pub fn convalescence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep()
                .with_filter(Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 10 }),
            effect: gain_life(1),
        }],
        ..enchantment("Convalescence", cost(&[generic(1), w()]))
    }
}

/// Onslaught — {R} Enchantment. Each creature spell you cast taps something.
pub fn onslaught() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
        }],
        ..enchantment("Onslaught", cost(&[r()]))
    }
}

/// Mana Breach — {2}{U} Enchantment. Every spell bounces its caster a land.
pub fn mana_breach() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::Move {
                what: Selector::ControlledBy {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    filter: R::Land,
                },
                to: ZoneDest::Hand(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
            },
        }],
        ..enchantment("Mana Breach", cost(&[generic(2), u()]))
    }
}
