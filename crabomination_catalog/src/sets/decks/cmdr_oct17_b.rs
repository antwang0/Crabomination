//! Commander: the 2026-10-17 (`01Htv6cF`) partner-pair scan's one-card gaps —
//! short-text bodies and spells. Tests in `tests/recent_b/cmdr_oct17_b.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    etb, etb_draw, investigate, landfall, mint_treasures, on_attack, on_dies, prowess_trigger, target_any, target_filtered,
};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crate::sets::{etb_gain_one, tapland_untyped};
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

fn pump_self(p: i32, t: i32) -> Effect {
    Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

/// "{cost}: Return this card from your graveyard to your hand."
fn graveyard_to_hand(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        from_graveyard: true,
        effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
        ..Default::default()
    }
}

fn mine(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

// ── Vanilla and keyword bodies ─────────────────────────────────────────────

/// Forest Bear — {1}{G} 2/2 Bear.
pub fn forest_bear() -> CardDefinition {
    creature("Forest Bear", cost(&[generic(1), g()]), vec![CreatureType::Bear], 2, 2)
}

/// Loxodon Wayfarer — {2}{W} 1/5 Elephant Monk.
pub fn loxodon_wayfarer() -> CardDefinition {
    creature("Loxodon Wayfarer", cost(&[generic(2), w()]), vec![CreatureType::Elephant, CreatureType::Monk], 1, 5)
}

/// Cerodon Yearling — {R}{W} 2/2 Beast. Vigilance, haste.
pub fn cerodon_yearling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        ..creature("Cerodon Yearling", cost(&[r(), w()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Large Bear — {3}{B/G}{B/G} 5/5 Bear. Reach, trample, haste.
pub fn large_bear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Haste],
        ..creature(
            "Large Bear",
            cost(&[generic(3), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
            vec![CreatureType::Bear],
            5,
            5,
        )
    }
}

/// Pilgrim of the Fires — {7} 6/4 artifact Golem. First strike, trample.
pub fn pilgrim_of_the_fires() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Trample],
        ..artifact_creature("Pilgrim of the Fires", cost(&[generic(7)]), vec![CreatureType::Golem], 6, 4)
    }
}

/// Woodland Changeling — {1}{G} 2/2 Shapeshifter. Changeling.
pub fn woodland_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        ..creature("Woodland Changeling", cost(&[generic(1), g()]), vec![CreatureType::Shapeshifter], 2, 2)
    }
}

/// Mischievous Sneakling — {1}{U/B} 2/2 Shapeshifter. Changeling, flash.
pub fn mischievous_sneakling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Flash],
        ..creature(
            "Mischievous Sneakling",
            cost(&[generic(1), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Shapeshifter],
            2,
            2,
        )
    }
}

/// Prideful Feastling — {2}{W/B} 2/3 Shapeshifter. Changeling, lifelink.
pub fn prideful_feastling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling, Keyword::Lifelink],
        ..creature(
            "Prideful Feastling",
            cost(&[generic(2), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Shapeshifter],
            2,
            3,
        )
    }
}

/// Greenwood Sentinel — {1}{G} 2/2 Elf Scout. Vigilance.
pub fn greenwood_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature("Greenwood Sentinel", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 2)
    }
}

/// Hippo-Cows — {4}{G} 5/4 Hippo Ox. Trample.
pub fn hippo_cows() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Hippo-Cows", cost(&[generic(4), g()]), vec![CreatureType::Hippo, CreatureType::Ox], 5, 4)
    }
}

/// Cystbearer — {2}{G} 2/3 Phyrexian Beast. Infect.
pub fn cystbearer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Infect],
        ..creature("Cystbearer", cost(&[generic(2), g()]), vec![CreatureType::Phyrexian, CreatureType::Beast], 2, 3)
    }
}

/// Lotus Path Djinn — {3}{U} 2/3 flying Djinn Monk. Prowess.
pub fn lotus_path_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        triggered_abilities: vec![prowess_trigger()],
        ..creature("Lotus Path Djinn", cost(&[generic(3), u()]), vec![CreatureType::Djinn, CreatureType::Monk], 2, 3)
    }
}

/// Iron Golem — {4} 5/3 artifact Golem. Vigilance; attacks or blocks each
/// combat if able.
pub fn iron_golem() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::MustAttackOrBlock],
        ..artifact_creature("Iron Golem", cost(&[generic(4)]), vec![CreatureType::Golem], 5, 3)
    }
}

/// Demon of Catastrophes — {2}{B}{B} 6/6 flying, trample Demon. Additional
/// cost: sacrifice a creature.
pub fn demon_of_catastrophes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature.and(R::ControlledByYou), count: 1 }],
        ..creature("Demon of Catastrophes", cost(&[generic(2), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

// ── Triggered and activated bodies ─────────────────────────────────────────

/// Catalyst Elemental — {2}{R} 2/2. Sacrifice it: add {R}{R}.
pub fn catalyst_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::Const(2)) },
            ..Default::default()
        }],
        ..creature("Catalyst Elemental", cost(&[generic(2), r()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Summit Sentinel — {1}{U} 1/3 Elemental Soldier. Dies: draw a card.
pub fn summit_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..creature("Summit Sentinel", cost(&[generic(1), u()]), vec![CreatureType::Elemental, CreatureType::Soldier], 1, 3)
    }
}

/// Roving Harper — {2}{W} 2/2 Elf Scout. ETB: draw a card.
pub fn roving_harper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb_draw(1)],
        ..creature("Roving Harper", cost(&[generic(2), w()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 2)
    }
}

/// Fathom Fleet Firebrand — {1}{R} 2/2 Human Pirate. {1}{R}: +1/+0.
pub fn fathom_fleet_firebrand() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(1), r()]), effect: pump_self(1, 0), ..Default::default() }],
        ..creature("Fathom Fleet Firebrand", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Grasping Scoundrel — {B} 1/1 Human Pirate. +1/+0 while attacking.
pub fn grasping_scoundrel() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 as long as it's attacking.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                power: 1,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature("Grasping Scoundrel", cost(&[b()]), vec![CreatureType::Human, CreatureType::Pirate], 1, 1)
    }
}

/// Battleground Geist — {4}{U} 3/3 flying Spirit. Other Spirits you control
/// get +1/+0.
pub fn battleground_geist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: mine(R::Creature.and(R::HasCreatureType(CreatureType::Spirit)).and(R::OtherThanSource)),
                power: 1,
                toughness: 0,
            },
        }],
        ..creature("Battleground Geist", cost(&[generic(4), u()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Destructive Digger — {2}{R} 3/2 Goblin. {3}, {T}, sacrifice an artifact or
/// land: draw a card.
pub fn destructive_digger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.or(R::Land), 1)),
            sac_other_may_be_source: false,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Destructive Digger", cost(&[generic(2), r()]), vec![CreatureType::Goblin], 3, 2)
    }
}

/// Boldwyr Aggressor — {3}{R}{R} 2/5 Giant Warrior. Double strike; other
/// Giants you control have double strike.
pub fn boldwyr_aggressor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "Other Giants you control have double strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: mine(R::Creature.and(R::HasCreatureType(CreatureType::Giant)).and(R::OtherThanSource)),
                keyword: Keyword::DoubleStrike,
            },
        }],
        ..creature("Boldwyr Aggressor", cost(&[generic(3), r(), r()]), vec![CreatureType::Giant, CreatureType::Warrior], 2, 5)
    }
}

/// Project Deathlok Soldier — {B} 1/2 artifact Zombie Soldier. {2}{B}:
/// return it from your graveyard to your hand.
pub fn project_deathlok_soldier() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![graveyard_to_hand(cost(&[generic(2), b()]))],
        ..artifact_creature("Project Deathlok Soldier", cost(&[b()]), vec![CreatureType::Zombie, CreatureType::Soldier], 1, 2)
    }
}

/// Blessed Ghoul — {W/B} 1/1 lifelink Zombie Cleric. {2}{W/B}: return it from
/// your graveyard to your hand.
pub fn blessed_ghoul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![graveyard_to_hand(cost(&[generic(2), hybrid(Color::White, Color::Black)]))],
        ..creature("Blessed Ghoul", cost(&[hybrid(Color::White, Color::Black)]), vec![CreatureType::Zombie, CreatureType::Cleric], 1, 1)
    }
}

/// Storm Caller — {2}{R} 3/2 Ogre Shaman. ETB: 2 damage to each opponent.
pub fn storm_caller() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) })],
        ..creature("Storm Caller", cost(&[generic(2), r()]), vec![CreatureType::Ogre, CreatureType::Shaman], 3, 2)
    }
}

/// Moonglove Extractor — {2}{B} 2/1 Elf Warlock. Attacks: draw, lose 1 life.
pub fn moonglove_extractor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..creature("Moonglove Extractor", cost(&[generic(2), b()]), vec![CreatureType::Elf, CreatureType::Warlock], 2, 1)
    }
}

/// Dauntless Survivor — {1}{G} 1/1 Human Warrior. ETB: a +1/+1 counter on
/// target creature.
pub fn dauntless_survivor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Dauntless Survivor", cost(&[generic(1), g()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

/// Black Cat — {1}{B} 1/1 Zombie Cat. Dies: target opponent discards a card
/// at random.
pub fn black_cat() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::ONE, random: true })],
        ..creature("Black Cat", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Cat], 1, 1)
    }
}

/// Moriok Replica — {3} 2/2 artifact Warrior. {1}{B}, sacrifice it: draw two
/// cards and lose 2 life.
pub fn moriok_replica() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..artifact_creature("Moriok Replica", cost(&[generic(3)]), vec![CreatureType::Warrior], 2, 2)
    }
}

/// Supply Runners — {4}{W} 2/2 Dog. ETB: a +1/+1 counter on each other
/// creature you control.
pub fn supply_runners() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: mine(R::Creature.and(R::OtherThanSource)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Supply Runners", cost(&[generic(4), w()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Angler Drake — {4}{U}{U} 4/4 flying Drake. ETB: you may return target
/// creature to its owner's hand.
pub fn angler_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature("Angler Drake", cost(&[generic(4), u(), u()]), vec![CreatureType::Drake], 4, 4)
    }
}

/// Rimekin Recluse — {2}{U} 3/2 Elemental Wizard. ETB: return up to one other
/// target creature to its owner's hand.
pub fn rimekin_recluse() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature.and(R::OtherThanSource),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature("Rimekin Recluse", cost(&[generic(2), u()]), vec![CreatureType::Elemental, CreatureType::Wizard], 3, 2)
    }
}

/// Canopy Baloth — {3}{G} 4/3 Beast. Landfall: +2/+2 until end of turn.
pub fn canopy_baloth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(pump_self(2, 2))],
        ..creature("Canopy Baloth", cost(&[generic(3), g()]), vec![CreatureType::Beast], 4, 3)
    }
}

/// Headstrong Brute — {2}{R} 3/3 Orc Pirate. Can't block; menace while you
/// control another Pirate.
pub fn headstrong_brute() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlock],
        static_abilities: vec![StaticAbility {
            description: "This creature has menace as long as you control another Pirate.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Pirate).and(R::OtherThanSource),
                }),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Menace],
            },
        }],
        ..creature("Headstrong Brute", cost(&[generic(2), r()]), vec![CreatureType::Orc, CreatureType::Pirate], 3, 3)
    }
}

/// Runaway Trash-Bot — {3} 0/4 trample artifact Construct. +1/+0 for each
/// artifact and/or enchantment card in your graveyard.
pub fn runaway_trash_bot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each artifact and/or enchantment card in your graveyard.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Enchantment)),
                }),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..artifact_creature("Runaway Trash-Bot", cost(&[generic(3)]), vec![CreatureType::Construct], 0, 4)
    }
}

/// Zephyr Winder — {1}{U} 2/1 flying Elemental. Combat damage to a player:
/// untap up to one target creature.
pub fn zephyr_winder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
            },
        }],
        ..creature("Zephyr Winder", cost(&[generic(1), u()]), vec![CreatureType::Elemental], 2, 1)
    }
}

fn thopter() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Experimental Aviator — {3}{U}{U} 0/3 flying Human Artificer. ETB: two 1/1
/// flying Thopter artifact creature tokens.
pub fn experimental_aviator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(thopter()) })],
        ..creature("Experimental Aviator", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Artificer], 0, 3)
    }
}

/// Stark Industries Executive — {R} 1/2 Human Advisor. {2}, {T}: create a
/// Treasure.
pub fn stark_industries_executive() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: mint_treasures(1),
            ..Default::default()
        }],
        ..creature("Stark Industries Executive", cost(&[r()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 2)
    }
}

/// Scarblade Scout — {1}{B} 2/2 lifelink Elf Scout. ETB: mill two.
pub fn scarblade_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(2) })],
        ..creature("Scarblade Scout", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Scout], 2, 2)
    }
}

/// Brazen Freebooter — {3}{R} 3/3 Human Pirate. ETB: create a Treasure.
pub fn brazen_freebooter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(mint_treasures(1))],
        ..creature("Brazen Freebooter", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Pirate], 3, 3)
    }
}

/// Agents of HYDRA — {1}{B} 1/1 Human Spy Villain. Dies: a 2/1 black Villain
/// token with menace.
pub fn agents_of_hydra() -> CardDefinition {
    let villain = TokenDefinition {
        name: "Villain".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Villain], ..Default::default() },
        keywords: vec![Keyword::Menace],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(villain) })],
        ..creature("Agents of HYDRA", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Spy, CreatureType::Villain], 1, 1)
    }
}

/// Byway Courier — {2}{G} 3/2 Human Scout. Dies: investigate.
pub fn byway_courier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(investigate(1))],
        ..creature("Byway Courier", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Scout], 3, 2)
    }
}

/// Scholar of the Ages — {5}{U}{U} 3/3 Human Wizard. ETB: return up to two
/// target instant and/or sorcery cards from your graveyard to your hand.
pub fn scholar_of_the_ages() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 2,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InYourGraveyard),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        })],
        ..creature("Scholar of the Ages", cost(&[generic(5), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Deathknell Berserker — {1}{B} 2/2 Elf Berserker. Dies with power 3 or
/// greater: a 2/2 black Zombie Berserker token.
pub fn deathknell_berserker() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie Berserker".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie, CreatureType::Berserker], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            // CR 603.10 — the dying creature's last-known power, read in the
            // body (the trigger filter sees the graveyard card's printed 2).
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::PowerOf(Box::new(Selector::TriggerSource)), Value::Const(3)),
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zombie) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Deathknell Berserker", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Berserker], 2, 2)
    }
}

/// Patron of the Valiant — {3}{W}{W} 4/4 flying Angel. ETB: a +1/+1 counter
/// on each creature you control with a +1/+1 counter on it.
pub fn patron_of_the_valiant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: mine(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Patron of the Valiant", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Avaricious Dragon — {2}{R}{R} 4/4 flying Dragon. Draw an extra card in
/// your draw step; discard your hand at your end step.
pub fn avaricious_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::YourControl),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::Discard { who: Selector::You, amount: Value::Const(100), random: false },
            },
        ],
        ..creature("Avaricious Dragon", cost(&[generic(2), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Boneclad Necromancer — {3}{B}{B} 3/3 Human Wizard. ETB: you may exile
/// target creature card from a graveyard; if you do, a 2/2 black Zombie.
pub fn boneclad_necromancer() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::HasCardType(CardType::Creature).and(R::InGraveyard),
            effect: Box::new(Effect::Seq(vec![
                Effect::Exile { what: Selector::Target(0) },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(zombie) },
            ])),
        })],
        ..creature("Boneclad Necromancer", cost(&[generic(3), b(), b()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 3)
    }
}

/// Chain Devil — {3}{B} 4/2 Devil. ETB: each player sacrifices a nontoken
/// creature of their choice.
pub fn chain_devil() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Creature.and(R::NotToken),
        })],
        ..creature("Chain Devil", cost(&[generic(3), b()]), vec![CreatureType::Devil], 4, 2)
    }
}

/// Selhoff Occultist — {2}{U} 2/3 Human Rogue. Whenever it or another
/// creature dies, target player mills a card.
pub fn selhoff_occultist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::Mill { who: target_filtered(R::Player), amount: Value::ONE },
        }],
        ..creature("Selhoff Occultist", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 3)
    }
}

/// Godhead of Awe — {W/U}x5 4/4 flying Spirit Avatar. Other creatures have
/// base power and toughness 1/1.
pub fn godhead_of_awe() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other creatures have base power and toughness 1/1.",
            effect: StaticEffect::SetBasePtForFilter { applies_to: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)), power: 1, toughness: 1 },
        }],
        ..creature("Godhead of Awe", cost(&[wu(), wu(), wu(), wu(), wu()]), vec![CreatureType::Spirit, CreatureType::Avatar], 4, 4)
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Unmake — {W/B}{W/B}{W/B} Instant. Exile target creature.
pub fn unmake() -> CardDefinition {
    let wb = || hybrid(Color::White, Color::Black);
    spell("Unmake", cost(&[wb(), wb(), wb()]), CardType::Instant, Effect::Exile { what: target_filtered(R::Creature) })
}

/// Death Ward — {W} Instant. Regenerate target creature.
pub fn death_ward() -> CardDefinition {
    spell("Death Ward", cost(&[w()]), CardType::Instant, Effect::Regenerate { what: target_filtered(R::Creature) })
}

/// Disorient — {3}{U} Instant. Target creature gets -7/-0 until end of turn.
pub fn disorient() -> CardDefinition {
    spell(
        "Disorient",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(-7), toughness: Value::Const(0), duration: Duration::EndOfTurn },
    )
}

/// Magmatic Chasm — {1}{R} Sorcery. Creatures without flying can't block this
/// turn.
pub fn magmatic_chasm() -> CardDefinition {
    spell(
        "Magmatic Chasm",
        cost(&[generic(1), r()]),
        CardType::Sorcery,
        Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying).negate())),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Inordinate Rage — {1}{R} Instant. Target creature gets +3/+2; scry 1.
pub fn inordinate_rage() -> CardDefinition {
    spell(
        "Inordinate Rage",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(3), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
    )
}

/// Shoot Down — {3}{G} Sorcery. Exile target artifact, enchantment, or
/// creature with flying.
pub fn shoot_down() -> CardDefinition {
    spell(
        "Shoot Down",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::Exile {
            what: target_filtered(R::Artifact.or(R::Enchantment).or(R::Creature.and(R::HasKeyword(Keyword::Flying)))),
        },
    )
}

/// "Target creature gets +2/+2 and gains `keyword` until end of turn. Untap
/// it." (Tethermage's Advantage, Riverguard's Reflexes.)
fn pump_grant_untap(name: &'static str, mana: ManaCost, keyword: Keyword) -> CardDefinition {
    spell(
        name,
        mana,
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
    )
}

/// Tethermage's Advantage — {G} Instant. +2/+2, reach, untap it.
pub fn tethermages_advantage() -> CardDefinition {
    pump_grant_untap("Tethermage's Advantage", cost(&[g()]), Keyword::Reach)
}

/// Riverguard's Reflexes — {1}{W} Instant. +2/+2, first strike, untap it.
pub fn riverguards_reflexes() -> CardDefinition {
    pump_grant_untap("Riverguard's Reflexes", cost(&[generic(1), w()]), Keyword::FirstStrike)
}

/// Rise Again — {4}{B} Sorcery. Return target creature card from your
/// graveyard to the battlefield.
pub fn rise_again() -> CardDefinition {
    spell(
        "Rise Again",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::HasCardType(CardType::Creature).and(R::InYourGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Restore with Empathy — {2}{G} Instant. Return target permanent card from
/// your graveyard to your hand; gain 4 life.
pub fn restore_with_empathy() -> CardDefinition {
    spell(
        "Restore with Empathy",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::PermanentCard.and(R::InYourGraveyard)), to: ZoneDest::Hand(PlayerRef::You) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
        ]),
    )
}

/// Compelling Argument — {1}{U} Sorcery. Target player mills five. Cycling
/// {U}.
pub fn compelling_argument() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[u()]))],
        ..spell(
            "Compelling Argument",
            cost(&[generic(1), u()]),
            CardType::Sorcery,
            Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(5) },
        )
    }
}

/// Djeru's Renunciation — {1}{W} Instant. Tap up to two target creatures.
/// Cycling {W}.
pub fn djerus_renunciation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[w()]))],
        ..spell(
            "Djeru's Renunciation",
            cost(&[generic(1), w()]),
            CardType::Instant,
            Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            },
        )
    }
}

/// Cruel Tutor — {2}{B} Sorcery. Search for a card, put it on top after
/// shuffling; lose 2 life.
pub fn cruel_tutor() -> CardDefinition {
    spell(
        "Cruel Tutor",
        cost(&[generic(2), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top } },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Perilous Voyage — {1}{U} Instant. Return target nonland permanent you
/// don't control to its owner's hand; scry 2 if its mana value was 2 or less.
pub fn perilous_voyage() -> CardDefinition {
    spell(
        "Perilous Voyage",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByYou.negate())),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::If {
                cond: Predicate::EntityMatchesAny { what: Selector::LastMoved, filter: R::ManaValueAtMost(2) },
                then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Dream Twist — {U} Instant. Target player mills three. Flashback {1}{U}.
pub fn dream_twist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u()]))],
        ..spell("Dream Twist", cost(&[u()]), CardType::Instant, Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(3) })
    }
}

/// Surgical Precision — {1}{W} Sorcery. Choose one: destroy target creature
/// with toughness 4 or greater and gain 1 life; or draw a card and gain 2.
pub fn surgical_precision() -> CardDefinition {
    spell(
        "Surgical Precision",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature.and(R::ToughnessAtLeast(4))) },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        ]),
    )
}

/// Incendiary Sabotage — {2}{R}{R} Instant. Additional cost: sacrifice an
/// artifact. 3 damage to each creature.
pub fn incendiary_sabotage() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Artifact.and(R::ControlledByYou), count: 1 }],
        ..spell(
            "Incendiary Sabotage",
            cost(&[generic(2), r(), r()]),
            CardType::Instant,
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(3) },
        )
    }
}

/// Spicy Oatmeal Pizza — {2}{R} Food. ETB: 4 damage to any target and 3 to
/// you. {2}, {T}, sacrifice: gain 3 life.
pub fn spicy_oatmeal_pizza() -> CardDefinition {
    CardDefinition {
        name: "Spicy Oatmeal Pizza",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Food], ..Default::default() },
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::Const(4) },
            Effect::DealDamage { to: Selector::You, amount: Value::Const(3) },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Permanents ─────────────────────────────────────────────────────────────

/// Swashbuckling — {1}{R} Aura. +2/+2 and haste.
pub fn swashbuckling() -> CardDefinition {
    aura("Swashbuckling", cost(&[generic(1), r()]), EquipBonus { power: 2, toughness: 2, keywords: vec![Keyword::Haste], ..Default::default() })
}

/// Wreath of Geists — {G} Aura. +X/+X, X = creature cards in your graveyard.
pub fn wreath_of_geists() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +X/+X, where X is the number of creature cards in your graveyard.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::HasCardType(CardType::Creature) }),
                toughness: Value::count(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::HasCardType(CardType::Creature) }),
            },
        }],
        ..aura("Wreath of Geists", cost(&[g()]), EquipBonus::default())
    }
}

/// Avacyn's Memorial — {5}{W}{W}{W} legendary artifact. Indestructible.
/// Other legendary permanents you control have indestructible.
pub fn avacyns_memorial() -> CardDefinition {
    CardDefinition {
        name: "Avacyn's Memorial",
        cost: cost(&[generic(5), w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "Other legendary permanents you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: mine(R::Permanent.and(R::HasSupertype(Supertype::Legendary)).and(R::OtherThanSource)),
                keyword: Keyword::Indestructible,
            },
        }],
        ..Default::default()
    }
}

/// Temur Banner — {3} Artifact. {T}: add {G}, {U}, or {R}. {G}{U}{R}, {T},
/// sacrifice it: draw a card.
pub fn temur_banner() -> CardDefinition {
    CardDefinition {
        name: "Temur Banner",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Green, Color::Blue, Color::Red], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g(), u(), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mind Unbound — {4}{U}{U} Enchantment. Upkeep: a lore counter, then draw a
/// card for each lore counter on it.
pub fn mind_unbound() -> CardDefinition {
    CardDefinition {
        name: "Mind Unbound",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Lore, amount: Value::ONE },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Lore },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Greataxe — {1} Equipment, equip {5}. +4/+0.
pub fn greataxe() -> CardDefinition {
    CardDefinition {
        name: "Greataxe",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equipped_bonus: Some(EquipBonus { power: 4, ..Default::default() }),
        ..Default::default()
    }
}

/// Thopter Arrest — {2}{W} Enchantment. ETB: exile target artifact or
/// creature an opponent controls until this leaves the battlefield.
pub fn thopter_arrest() -> CardDefinition {
    CardDefinition {
        name: "Thopter Arrest",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// The six Marvel gain lands: enters tapped, gain 1 life on entry, taps for
/// either of two colors.
fn gain_land(name: &'static str, a: Color, b: Color) -> CardDefinition {
    tapland_untyped(name, a, b, etb_gain_one())
}

/// Avengers Hangar — W/U gain land.
pub fn avengers_hangar() -> CardDefinition {
    gain_land("Avengers Hangar", Color::White, Color::Blue)
}

/// Dimension X — R/W gain land.
pub fn dimension_x() -> CardDefinition {
    gain_land("Dimension X", Color::Red, Color::White)
}

/// Foot Headquarters — W/B gain land.
pub fn foot_headquarters() -> CardDefinition {
    gain_land("Foot Headquarters", Color::White, Color::Black)
}

/// Illegitimate Business — B/G gain land.
pub fn illegitimate_business() -> CardDefinition {
    gain_land("Illegitimate Business", Color::Black, Color::Green)
}

/// Los Diablos Missile Base — R/G gain land.
pub fn los_diablos_missile_base() -> CardDefinition {
    gain_land("Los Diablos Missile Base", Color::Red, Color::Green)
}

/// Mutant Town — G/U gain land.
pub fn mutant_town() -> CardDefinition {
    gain_land("Mutant Town", Color::Green, Color::Blue)
}
