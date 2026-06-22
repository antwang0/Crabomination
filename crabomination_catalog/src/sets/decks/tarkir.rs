//! Tarkir: Dragonstorm (TDM) — assorted non-Omen cards. The set's Omen Dragon
//! cycle lives in `decks::omen`; this module collects the straightforward
//! spells/creatures that ride existing primitives. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement, Selector,
    StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    dies_mint_token, etb, flurry, mobilize, mobilize_value, target_any, target_filtered,
};
use crate::effect::{Duration, LibraryPosition, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, mono_hybrid, r, u, w, Color};

/// Sarkhan's Resolve — {1}{G} Instant. Choose one — target creature gets +3/+3
/// until end of turn; or destroy target creature with flying.
pub fn sarkhans_resolve() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan's Resolve",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Dragonback Lancer — {3}{W} 3/3 Human Soldier with Flying and Mobilize 1.
pub fn dragonback_lancer() -> CardDefinition {
    CardDefinition {
        name: "Dragonback Lancer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![mobilize(1)],
        ..Default::default()
    }
}

/// Sibsig Appraiser — {2}{U} 2/1 Zombie Advisor. ETB: look at the top two cards,
/// put one into your hand and the other into your graveyard.
pub fn sibsig_appraiser() -> CardDefinition {
    CardDefinition {
        name: "Sibsig Appraiser",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Defibrillating Current — {2/R}{2/W}{2/B} Sorcery. Deal 4 damage to target
/// creature or planeswalker and gain 2 life.
pub fn defibrillating_current() -> CardDefinition {
    CardDefinition {
        name: "Defibrillating Current",
        cost: cost(&[
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Mardu Devotee — {W} 1/2 Human Scout. ETB: scry 2. `{1}: Add {R}, {W}, or
/// {B}. Activate only once each turn.`
pub fn mardu_devotee() -> CardDefinition {
    CardDefinition {
        name: "Mardu Devotee",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Red, Color::White, Color::Black], Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sibsig Host — {4}{B} 2/6 Zombie. ETB: each player mills three cards.
pub fn sibsig_host() -> CardDefinition {
    CardDefinition {
        name: "Sibsig Host",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 6,
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::Const(3),
        })],
        ..Default::default()
    }
}

/// Stormscale Scion — {4}{R}{R} 4/4 Dragon with Flying and Storm. Other Dragons
/// you control get +1/+1.
pub fn stormscale_scion() -> CardDefinition {
    CardDefinition {
        name: "Stormscale Scion",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Storm],
        static_abilities: vec![StaticAbility {
            description: "Other Dragons you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Roilmage's Trick — {3}{U} Instant. Converge — creatures your opponents
/// control get -X/-0 until end of turn, where X is the number of colors of mana
/// spent to cast this spell. Draw a card.
pub fn roilmages_trick() -> CardDefinition {
    CardDefinition {
        name: "Roilmage's Trick",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Diff(Box::new(Value::Const(0)), Box::new(Value::ConvergedValue)),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Kishla Skimmer — {G}{U} 2/2 Bird Scout with Flying. Whenever a card leaves
/// your graveyard during your turn, draw a card (only once each turn).
pub fn kishla_skimmer() -> CardDefinition {
    CardDefinition {
        name: "Kishla Skimmer",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                kind: EventKind::CardLeftGraveyard,
                scope: EventScope::YourControl,
                filter: Some(Predicate::IsTurnOf(PlayerRef::You)),
                once_per_turn: true,
                per_subject_cap: None,
                actor_is_opponent: false,
            },
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Inevitable Defeat — {1}{R}{W}{B} Instant that can't be countered. Exile
/// target nonland permanent; its controller loses 3 life and you gain 3 life.
pub fn inevitable_defeat() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Inevitable Defeat",
        cost: cost(&[generic(1), r(), w(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
                to: ZoneDest::Exile,
            },
        ]),
        ..Default::default()
    }
}

/// Magmatic Hellkite — {2}{R}{R} 4/5 Dragon with Flying. ETB: destroy target
/// nonbasic land an opponent controls; its controller searches for a basic land
/// and puts it onto the battlefield tapped with a stun counter on it.
pub fn magmatic_hellkite() -> CardDefinition {
    use crate::card::{CounterType, Supertype};
    use crate::effect::ZoneDest;
    let opp_land = SelectionRequirement::Land
        .and(SelectionRequirement::HasSupertype(Supertype::Basic).negate())
        .and(SelectionRequirement::ControlledByOpponent);
    let basic = SelectionRequirement::Land.and(SelectionRequirement::HasSupertype(Supertype::Basic));
    CardDefinition {
        name: "Magmatic Hellkite",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            // The land's controller ramps a stunned basic before the land is
            // destroyed, so `ControllerOf(target)` still resolves to them; the
            // net effect matches the printed "destroy, then its controller…".
            Effect::SearchUpToN {
                who: PlayerRef::ControllerOf(Box::new(target_filtered(opp_land.clone()))),
                filter: basic,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(target_filtered(opp_land.clone()))),
                    tapped: true,
                },
                count: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
            Effect::Destroy { what: target_filtered(opp_land) },
        ]))],
        ..Default::default()
    }
}

/// Cori Mountain Stalwart — {1}{R}{W} 3/3 Human Monk. Flurry — when you cast
/// your second spell each turn, deal 2 damage to each opponent and gain 2 life.
pub fn cori_mountain_stalwart() -> CardDefinition {
    CardDefinition {
        name: "Cori Mountain Stalwart",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![flurry(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Equilibrium Adept — {3}{R} 2/4 Dog Monk. ETB exile the top card; you may
/// play it until the end of your next turn. Flurry — gains double strike EOT.
pub fn equilibrium_adept() -> CardDefinition {
    CardDefinition {
        name: "Equilibrium Adept",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                uncast_penalty: None,
            }),
            flurry(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..Default::default()
    }
}

/// Cunning Coyote — {1}{R} 2/2 Coyote with Haste. ETB another target creature
/// you control gets +1/+1 and gains haste until end of turn. Plot {1}{R}.
pub fn cunning_coyote() -> CardDefinition {
    CardDefinition {
        name: "Cunning Coyote",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Coyote], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        plot_cost: Some(cost(&[generic(1), r()])),
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
            effect: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Monastery Messenger — {2/U}{2/R}{2/W} 2/3 Bird Scout with Flying, Vigilance.
/// ETB put up to one target noncreature, nonland card from your graveyard on
/// top of your library.
pub fn monastery_messenger() -> CardDefinition {
    CardDefinition {
        name: "Monastery Messenger",
        cost: cost(&[
            mono_hybrid(2, Color::Blue),
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::White),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: SelectionRequirement::InGraveyard
                .and(SelectionRequirement::Creature.negate())
                .and(SelectionRequirement::Land.negate()),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            }),
        })],
        ..Default::default()
    }
}

/// Bone-Cairn Butcher — {1}{R}{W}{B} 4/4 Demon with Mobilize 2. Attacking
/// tokens you control have deathtouch.
pub fn bone_cairn_butcher() -> CardDefinition {
    CardDefinition {
        name: "Bone-Cairn Butcher",
        cost: cost(&[generic(1), r(), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![mobilize(2)],
        static_abilities: vec![StaticAbility {
            description: "Attacking tokens you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::IsToken
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsAttacking),
                ),
                keyword: Keyword::Deathtouch,
            },
        }],
        ..Default::default()
    }
}

/// Salt Road Patrol — {3}{W} 2/5 Human Scout. Outlast {1}{W}.
pub fn salt_road_patrol() -> CardDefinition {
    use crate::effect::shortcut::outlast;
    CardDefinition {
        name: "Salt Road Patrol",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        activated_abilities: vec![outlast(cost(&[generic(1), w()]))],
        ..Default::default()
    }
}

/// Twin-Silk Spider — {2}{G} 1/2 Spider with Reach. ETB create a 1/2 green
/// Spider creature token with reach.
pub fn twin_silk_spider() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Twin-Silk Spider",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Spider".into(),
                power: 1,
                toughness: 2,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                keywords: vec![Keyword::Reach],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Spider],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Auroral Procession — {G}{U} Instant. Return target card from your graveyard
/// to your hand.
pub fn auroral_procession() -> CardDefinition {
    CardDefinition {
        name: "Auroral Procession",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::InGraveyard),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Ironpaw Aspirant — {1}{W} 1/2 Cat Warrior. ETB put a +1/+1 counter on target
/// creature.
pub fn ironpaw_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Ironpaw Aspirant",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Stormplain Detainment — {2}{W} Enchantment. ETB exile target nonland
/// permanent an opponent controls until this enchantment leaves the battlefield.
pub fn stormplain_detainment() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Stormplain Detainment",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Strategic Betrayal — {1}{B} Sorcery. Target opponent exiles a creature they
/// control and their graveyard. (Modeled as a sacrifice edict — the creature
/// dies, then the graveyard wipe exiles it along with the rest.)
pub fn strategic_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Strategic Betrayal",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Target(0),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::ExileAllGraveyards { filter: None, opponents_only: true },
        ]),
        ..Default::default()
    }
}

/// Sonic Shrieker — {2}{R}{W}{B} 4/4 Dragon with Flying. ETB deal 2 damage to
/// any target and gain 2 life. (The "if a player was dealt damage, they discard"
/// rider is dropped — the headline drain is modeled.)
pub fn sonic_shrieker() -> CardDefinition {
    CardDefinition {
        name: "Sonic Shrieker",
        cost: cost(&[generic(2), r(), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Sky Skiff — {2} Vehicle 2/3 with Flying. Crew 1.
pub fn sky_skiff() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Sky Skiff",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        ..Default::default()
    }
}

/// Frontline Rush — {R}{W} Instant. Choose one — create two 1/1 red Goblin
/// creature tokens; or target creature gets +X/+X until end of turn, where X is
/// the number of creatures you control.
pub fn frontline_rush() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Frontline Rush",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Goblin".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::CreatureCountControlledBy(PlayerRef::You),
                toughness: Value::CreatureCountControlledBy(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Severance Priest — {W}{B}{G} 3/3 Djinn Cleric with Deathtouch. ETB exile a
/// nonland card from a target opponent's hand until this leaves the battlefield.
pub fn severance_priest() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Severance Priest",
        cost: cost(&[w(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::ExileChosenUntilSourceLeaves {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland,
            return_to: ExileReturnZone::Hand,
        })],
        ..Default::default()
    }
}

/// Naga Fleshcrafter — {3}{U} 0/0 Snake Shapeshifter. May enter as a copy of a
/// creature you control. Renew — `{2}{U}, Exile this from your graveyard: Put a
/// +1/+1 counter on target nonlegendary creature you control.`
pub fn naga_fleshcrafter() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Naga Fleshcrafter",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Shapeshifter],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            extra_creature_types: vec![CreatureType::Snake, CreatureType::Shapeshifter],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasSupertype(Supertype::Legendary).negate()),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Marang River Skeleton — {1}{B} 1/1 Skeleton. `{B}: Regenerate this creature.`
/// Megamorph {3}{B}.
pub fn marang_river_skeleton() -> CardDefinition {
    CardDefinition {
        name: "Marang River Skeleton",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Megamorph(cost(&[generic(3), b()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mox Jasper — {0} Legendary Artifact. `{T}: Add one mana of any color.
/// Activate only if you control a Dragon.`
pub fn mox_jasper() -> CardDefinition {
    CardDefinition {
        name: "Mox Jasper",
        cost: cost(&[]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                ),
                n: Value::Const(1),
            }),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sage of the Fang — {2}{G} 2/2 Human Druid. ETB put a +1/+1 counter on target
/// creature. Renew — `{3}{G}, Exile this from your graveyard: Put a +1/+1
/// counter on target creature, then double the number of +1/+1 counters on it.`
pub fn sage_of_the_fang() -> CardDefinition {
    CardDefinition {
        name: "Sage of the Fang",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::DoubleCountersOnEach {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hardened Tactician — {1}{W}{B} 2/4 Human Warrior. `{1}, Sacrifice a token:
/// Draw a card.`
pub fn hardened_tactician() -> CardDefinition {
    CardDefinition {
        name: "Hardened Tactician",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((SelectionRequirement::IsToken, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Channeled Dragonfire — {R} Sorcery. Deals 2 damage to any target.
/// Harmonize {5}{R}{R} (CR 702.180 — recast from graveyard, reduce by a tapped
/// creature's power; exiled after).
pub fn channeled_dragonfire() -> CardDefinition {
    CardDefinition {
        name: "Channeled Dragonfire",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(5), r(), r()]))],
        effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Unending Whisper — {U} Sorcery. Draw a card. Harmonize {5}{U}.
pub fn unending_whisper() -> CardDefinition {
    CardDefinition {
        name: "Unending Whisper",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(5), u()]))],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ..Default::default()
    }
}

/// Ureni's Rebuff — {1}{U} Sorcery. Return target creature to its owner's hand.
/// Harmonize {5}{U}.
pub fn urenis_rebuff() -> CardDefinition {
    CardDefinition {
        name: "Ureni's Rebuff",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(5), u()]))],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        },
        ..Default::default()
    }
}

/// Wild Ride — {R} Sorcery. Target creature gets +3/+0 and gains haste until
/// end of turn. Harmonize {4}{R}.
pub fn wild_ride() -> CardDefinition {
    CardDefinition {
        name: "Wild Ride",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(4), r()]))],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Mammoth Bellow — {2}{G}{U}{R} Sorcery. Create a 5/5 green Elephant creature
/// token. Harmonize {5}{G}{U}{R}.
pub fn mammoth_bellow() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Mammoth Bellow",
        cost: cost(&[generic(2), g(), u(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(5), g(), u(), r()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Elephant".into(),
                power: 5,
                toughness: 5,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elephant],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

// ── TDM batch 7: Mobilize / Renew / Flurry creatures ──────────────────────

/// Nightblade Brigade — {2}{B} 1/3 Goblin Soldier with Deathtouch and
/// Mobilize 1. ETB: surveil 1.
pub fn nightblade_brigade() -> CardDefinition {
    CardDefinition {
        name: "Nightblade Brigade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            mobilize(1),
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) }),
        ],
        ..Default::default()
    }
}

/// Shock Brigade — {1}{R} 1/3 Goblin Soldier with Menace and Mobilize 1.
pub fn shock_brigade() -> CardDefinition {
    CardDefinition {
        name: "Shock Brigade",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![mobilize(1)],
        ..Default::default()
    }
}

/// Venerated Stormsinger — {3}{B} 3/3 Orc Cleric with Mobilize 1. Whenever this
/// or another creature you control dies, each opponent loses 1 life and you
/// gain 1 life.
pub fn venerated_stormsinger() -> CardDefinition {
    CardDefinition {
        name: "Venerated Stormsinger",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            mobilize(1),
            // "this or another creature you control dies" — self-death
            // (`SelfSource`) plus the others (`AnotherOfYours`), since the
            // death-event actor binding doesn't cover dying-creature control.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: stormsinger_drain(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: stormsinger_drain(),
            },
        ],
        ..Default::default()
    }
}

fn stormsinger_drain() -> Effect {
    Effect::Drain {
        from: Selector::Player(PlayerRef::EachOpponent),
        to: Selector::You,
        amount: Value::Const(1),
    }
}

/// Stadium Headliner — {R} 1/1 Goblin Warrior with Mobilize 1. `{1}{R},
/// Sacrifice this: It deals damage equal to the number of creatures you control
/// to target creature.`
pub fn stadium_headliner() -> CardDefinition {
    CardDefinition {
        name: "Stadium Headliner",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![mobilize(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Champion of Dusan — {2}{G} 4/2 Human Warrior with Trample. Renew — `{1}{G},
/// Exile this from your graveyard: Put a +1/+1 counter and a trample counter on
/// target creature. Sorcery speed.`
pub fn champion_of_dusan() -> CardDefinition {
    CardDefinition {
        name: "Champion of Dusan",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::AddKeywordCounter {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sagu Pummeler — {3}{G} 4/4 Beast with Reach. Renew — `{4}{G}, Exile this from
/// your graveyard: Put two +1/+1 counters and a reach counter on target
/// creature. Sorcery speed.`
pub fn sagu_pummeler() -> CardDefinition {
    CardDefinition {
        name: "Sagu Pummeler",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::AddKeywordCounter {
                    what: Selector::Target(0),
                    keyword: Keyword::Reach,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Adorned Crocodile — {4}{B} 5/3 Crocodile. When it dies, create a 2/2 black
/// Zombie Druid token. Renew — `{B}, Exile this from your graveyard: Put a
/// +1/+1 counter on target creature. Sorcery speed.`
pub fn adorned_crocodile() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Adorned Crocodile",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Crocodile], ..Default::default() },
        power: 5,
        toughness: 3,
        triggered_abilities: vec![dies_mint_token(
            TokenDefinition {
                name: "Zombie Druid".into(),
                power: 2,
                toughness: 2,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Black],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Zombie, CreatureType::Druid],
                    ..Default::default()
                },
                ..Default::default()
            },
            1,
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Lasyd Prowler — {2}{G}{G} 5/5 Snake Ranger. ETB: you may mill cards equal to
/// the number of lands you control. Renew — `{1}{G}, Exile this from your
/// graveyard: Put X +1/+1 counters on target creature, where X is the number of
/// land cards in your graveyard. Sorcery speed.`
pub fn lasyd_prowler() -> CardDefinition {
    CardDefinition {
        name: "Lasyd Prowler",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Ranger],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Mill cards equal to the number of lands you control?".into(),
            body: Box::new(Effect::Mill {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                )),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Monk of the Open Hand — {W} 1/1 Elf Monk. Flurry: put a +1/+1 counter on it.
pub fn monk_of_the_open_hand() -> CardDefinition {
    CardDefinition {
        name: "Monk of the Open Hand",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![flurry(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Jeskai Devotee — {1}{R} 2/2 Orc Monk. Flurry: it gets +1/+1 until end of
/// turn. `{1}: Add {U}, {R}, or {W}. Activate only once each turn.`
pub fn jeskai_devotee() -> CardDefinition {
    CardDefinition {
        name: "Jeskai Devotee",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![flurry(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            once_per_turn: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(
                    vec![Color::Blue, Color::Red, Color::White],
                    Value::Const(1),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wingblade Disciple — {2}{U} 2/2 Human Monk with Flying. Flurry: create a 1/1
/// white Bird creature token with flying.
pub fn wingblade_disciple() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Wingblade Disciple",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![flurry(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Bird".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                keywords: vec![Keyword::Flying],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Bird],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Poised Practitioner — {2}{W} 2/3 Human Monk. Flurry: put a +1/+1 counter on
/// it, then scry 1.
pub fn poised_practitioner() -> CardDefinition {
    CardDefinition {
        name: "Poised Practitioner",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![flurry(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Devoted Duelist — {1}{R} 2/1 Goblin Monk with Haste. Flurry: it deals 1
/// damage to each opponent.
pub fn devoted_duelist() -> CardDefinition {
    CardDefinition {
        name: "Devoted Duelist",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![flurry(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── TDM batch 8: more Renew / Mobilize / Flurry ───────────────────────────

/// Avenger of the Fallen — {2}{B} 2/4 Human Warrior with Deathtouch and
/// Mobilize X, where X is the number of creature cards in your graveyard.
pub fn avenger_of_the_fallen() -> CardDefinition {
    CardDefinition {
        name: "Avenger of the Fallen",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![mobilize_value(Value::CardsInGraveyardMatching {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
        })],
        ..Default::default()
    }
}

/// Dalkovan Packbeasts — {2}{W} 0/4 Ox with Vigilance and Mobilize 3.
pub fn dalkovan_packbeasts() -> CardDefinition {
    CardDefinition {
        name: "Dalkovan Packbeasts",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ox], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![mobilize(3)],
        ..Default::default()
    }
}

/// Reigning Victor — {2/R}{2/W}{2/B} 3/3 Orc Warrior with Mobilize 1. ETB:
/// target creature gets +1/+0 and gains indestructible until end of turn.
pub fn reigning_victor() -> CardDefinition {
    CardDefinition {
        name: "Reigning Victor",
        cost: cost(&[
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            mobilize(1),
            etb(Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ])),
        ],
        ..Default::default()
    }
}

/// Agent of Kotis — {1}{U} 2/1 Human Rogue. Renew — `{3}{U}, Exile this from
/// your graveyard: Put two +1/+1 counters on target creature. Sorcery speed.`
pub fn agent_of_kotis() -> CardDefinition {
    CardDefinition {
        name: "Agent of Kotis",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Alchemist's Assistant — {1}{B} 2/1 Monkey with Lifelink. Renew — `{1}{B},
/// Exile this from your graveyard: Put a lifelink counter on target creature.
/// Sorcery speed.`
pub fn alchemists_assistant() -> CardDefinition {
    CardDefinition {
        name: "Alchemist's Assistant",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Monkey], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Lifelink,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Qarsi Revenant — {1}{B}{B} 3/3 Vampire with Flying, Deathtouch, Lifelink.
/// Renew — `{2}{B}, Exile this from your graveyard: Put a flying counter, a
/// deathtouch counter, and a lifelink counter on target creature. Sorcery speed.`
pub fn qarsi_revenant() -> CardDefinition {
    CardDefinition {
        name: "Qarsi Revenant",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddKeywordCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Flying,
                    amount: Value::Const(1),
                },
                Effect::AddKeywordCounter {
                    what: Selector::Target(0),
                    keyword: Keyword::Deathtouch,
                    amount: Value::Const(1),
                },
                Effect::AddKeywordCounter {
                    what: Selector::Target(0),
                    keyword: Keyword::Lifelink,
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Constrictor Sage — {4}{U} 4/4 Snake Wizard. ETB: tap target creature an
/// opponent controls and put a stun counter on it. Renew — `{2}{U}, Exile this
/// from your graveyard: same. Sorcery speed.`
pub fn constrictor_sage() -> CardDefinition {
    let tap_and_stun = || {
        Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
        ])
    };
    CardDefinition {
        name: "Constrictor Sage",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(tap_and_stun())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: tap_and_stun(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wayspeaker Bodyguard — {3}{W} 3/4 Orc Monk. ETB: return target nonland
/// permanent card with mana value 2 or less from your graveyard to your hand.
/// Flurry: tap target creature an opponent controls.
pub fn wayspeaker_bodyguard() -> CardDefinition {
    CardDefinition {
        name: "Wayspeaker Bodyguard",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::Move {
                what: target_filtered(
                    SelectionRequirement::InYourGraveyard
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ManaValueAtMost(2))
                        .and(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Artifact)
                                .or(SelectionRequirement::Enchantment)
                                .or(SelectionRequirement::Planeswalker),
                        ),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            flurry(Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            }),
        ],
        ..Default::default()
    }
}

// ── TDM batch 9: spells + small creatures ─────────────────────────────────

/// Coordinated Maneuver — {1}{W} Instant. Choose one — deal damage equal to the
/// number of creatures you control to target creature or planeswalker; or
/// destroy target enchantment.
pub fn coordinated_maneuver() -> CardDefinition {
    CardDefinition {
        name: "Coordinated Maneuver",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
            },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Enchantment) },
        ]),
        ..Default::default()
    }
}

// ── TDM batch 10: Harmonize land-ramp + Reconfigure equipment ─────────────

/// Roamer's Routine — {2}{G} Sorcery. Search your library for a basic land card,
/// put it onto the battlefield tapped, then shuffle. Harmonize {4}{G}.
pub fn roamers_routine() -> CardDefinition {
    CardDefinition {
        name: "Roamer's Routine",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(4), g()]))],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Webspinner Cuff — {2}{G} 1/4 Artifact Creature — Equipment Spider with Reach.
/// Equipped creature gets +1/+4 and has reach. Reconfigure {4}.
pub fn webspinner_cuff() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Webspinner Cuff",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::Reconfigure(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 4,
            keywords: vec![Keyword::Reach],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        ..Default::default()
    }
}
