//! Commander: partner-pair closers, batch one (`01FnAVVR`) — single cards
//! each finishing one or more unseated pair decks. Tests in
//! `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{blocks, etb, mint_token, mint_treasures, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use crabomination_base::tokens::food_token;

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

fn pump(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

fn d6(results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides: 6,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

/// Velukan Dragon — {5}{R}{R} 5/5 flying Dragon. Attacking or blocking, roll
/// a d6: +X/+0 until end of turn, X the result minus 1.
pub fn velukan_dragon() -> CardDefinition {
    let roll = || d6(vec![(1, u8::MAX, pump(Selector::This, Value::Diff(Box::new(Value::LastDieRoll), Box::new(Value::ONE)), Value::ZERO))]);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(roll()), blocks(roll())],
        ..creature("Velukan Dragon", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Glamermite — {2}{U} 2/2 flash flying Faerie Rogue. ETB: tap or untap
/// target creature.
pub fn glamermite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::Untap { what: target_filtered(R::Creature), up_to: None },
        ]))],
        ..creature("Glamermite", cost(&[generic(2), u()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 2)
    }
}

/// Eyeblight Assassin — {2}{B} 2/2 Elf Assassin. ETB: target creature an
/// opponent controls gets -1/-1 until end of turn.
pub fn eyeblight_assassin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(pump(
            target_filtered(R::Creature.and(R::ControlledByOpponent)),
            Value::Const(-1),
            Value::Const(-1),
        ))],
        ..creature("Eyeblight Assassin", cost(&[generic(2), b()]), vec![CreatureType::Elf, CreatureType::Assassin], 2, 2)
    }
}

/// Featherbrained Filcher — {W} 0/2 flying Bird Mutant. Leaving the
/// battlefield, it makes a Food.
pub fn featherbrained_filcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: mint_token(food_token(), 1),
        }],
        ..creature("Featherbrained Filcher", cost(&[w()]), vec![CreatureType::Bird, CreatureType::Mutant], 0, 2)
    }
}

/// Noggle Robber — {1}{R/G}{R/G} 3/3 Noggle Rogue. ETB or dies: a Treasure.
pub fn noggle_robber() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(mint_treasures(1)), on_dies(mint_treasures(1))],
        ..creature(
            "Noggle Robber",
            ManaCost::new(vec![generic(1), hybrid(Color::Red, Color::Green), hybrid(Color::Red, Color::Green)]),
            vec![CreatureType::Noggle, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Embodiment of Spring — {U} 0/3 Elemental. {1}{G}, {T}, sacrifice it:
/// search for a basic land, onto the battlefield tapped.
pub fn embodiment_of_spring() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..creature("Embodiment of Spring", cost(&[u()]), vec![CreatureType::Elemental], 0, 3)
    }
}

/// Desperate Castaways — {1}{B} 2/3 Human Pirate; can't attack unless you
/// control an artifact.
pub fn desperate_castaways() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CanAttackOnlyIfYouControl(Box::new(R::Artifact))],
        ..creature("Desperate Castaways", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 3)
    }
}

/// Mardu Blazebringer — {2}{R} 4/4 Ogre Warrior. Attacking or blocking, it
/// is sacrificed at end of combat.
pub fn mardu_blazebringer() -> CardDefinition {
    let doom = || Effect::AtEndOfCombat { body: Box::new(Effect::SacrificeSource) };
    CardDefinition {
        triggered_abilities: vec![on_attack(doom()), blocks(doom())],
        ..creature("Mardu Blazebringer", cost(&[generic(2), r()]), vec![CreatureType::Ogre, CreatureType::Warrior], 4, 4)
    }
}

/// Screeching Phoenix — {4}{R}{R} 4/4 flying Phoenix. {2}{R}: creatures you
/// control get +1/+0 until end of turn.
pub fn screeching_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: pump(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)), Value::ONE, Value::ZERO),
            ..Default::default()
        }],
        ..creature("Screeching Phoenix", cost(&[generic(4), r(), r()]), vec![CreatureType::Phoenix], 4, 4)
    }
}

fn kithkin() -> TokenDefinition {
    token("Kithkin", 1, 1, vec![Color::Green, Color::White], vec![CreatureType::Kithkin])
}

/// Clachan Festival — {2}{W} Kindred enchantment — Kithkin. ETB: two 1/1
/// green and white Kithkin; {4}{W}: another.
pub fn clachan_festival() -> CardDefinition {
    CardDefinition {
        name: "Clachan Festival",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Kindred, CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin], ..Default::default() },
        triggered_abilities: vec![etb(mint_token(kithkin(), 2))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: mint_token(kithkin(), 1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stingerquill Charm — {B}{R} instant. Choose one: 3 damage to any target;
/// target creature gains first strike and deathtouch; or a 2/2 colorless
/// Wizard Soldier named Cadet with haste until end of turn.
pub fn stingerquill_charm() -> CardDefinition {
    let cadet = token("Cadet", 2, 2, vec![], vec![CreatureType::Wizard, CreatureType::Soldier]);
    spell(
        "Stingerquill Charm",
        cost(&[b(), r()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(3) },
            Effect::Seq(vec![
                Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            ]),
            crate::effect::shortcut::tokens_gain_until_eot(mint_token(cadet, 1), &[Keyword::Haste]),
        ]),
    )
}

/// Fathom Fleet Cutthroat — {3}{B} 3/3 Human Pirate. ETB: destroy target
/// creature an opponent controls that was dealt damage this turn.
pub fn fathom_fleet_cutthroat() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::DealtDamageThisTurn)),
        })],
        ..creature("Fathom Fleet Cutthroat", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Pirate], 3, 3)
    }
}

/// Escarpment Fortress — {4}{W} 3/5 Wall, defender, reach. Other creatures
/// you control get +1/+0; attacking with two or more draws a card.
pub fn escarpment_fortress() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::AttackingWithAtLeast(2)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Escarpment Fortress", cost(&[generic(4), w()]), vec![CreatureType::Wall], 3, 5)
    }
}

/// Moonlit Lamenter — {2}{W} 2/5 Treefolk Cleric that enters with a -1/-1
/// counter. {1}{W}, remove a counter: draw a card (sorcery speed). The cost
/// reads its -1/-1 counters, the only kind it puts on itself.
pub fn moonlit_lamenter() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            sorcery_speed: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Moonlit Lamenter", cost(&[generic(2), w()]), vec![CreatureType::Treefolk, CreatureType::Cleric], 2, 5)
    }
}

/// Burdened Stoneback — {1}{W} 4/4 Giant Warrior that enters with two -1/-1
/// counters. {1}{W}, remove a counter: target creature gains indestructible
/// until end of turn (sorcery speed).
pub fn burdened_stoneback() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::MinusOneMinusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            remove_counter_cost: Some((CounterType::MinusOneMinusOne, 1)),
            sorcery_speed: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Burdened Stoneback", cost(&[generic(1), w()]), vec![CreatureType::Giant, CreatureType::Warrior], 4, 4)
    }
}

/// Dissatisfied Customer — {2}{B} 2/1 flying, haste Vampire Guest. ETB: roll
/// a d6; on 3 or less, lose that much life.
pub fn dissatisfied_customer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![etb(d6(vec![(1, 3, Effect::LoseLife { who: Selector::You, amount: Value::LastDieRoll })]))],
        ..creature("Dissatisfied Customer", cost(&[generic(2), b()]), vec![CreatureType::Vampire, CreatureType::Guest], 2, 1)
    }
}

/// Freejam Regent — {4}{R}{R} 4/4 flying Cat Dragon with improvise. {1}{R}:
/// +2/+0 until end of turn.
pub fn freejam_regent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Improvise, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: pump(Selector::This, Value::Const(2), Value::ZERO),
            ..Default::default()
        }],
        ..creature("Freejam Regent", cost(&[generic(4), r(), r()]), vec![CreatureType::Cat, CreatureType::Dragon], 4, 4)
    }
}

/// Foundry Assembler — {5} 3/3 Assembly-Worker artifact creature, improvise.
pub fn foundry_assembler() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Improvise],
        ..creature("Foundry Assembler", cost(&[generic(5)]), vec![CreatureType::AssemblyWorker], 3, 3)
    }
}
