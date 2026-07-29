//! Tarkir: Dragonstorm (TDM) — assorted non-Omen cards. The set's Omen Dragon
//! cycle lives in `decks::omen`; this module collects the straightforward
//! spells/creatures that ride existing primitives. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement, Selector,
    StaticAbility, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    dies_mint_token, etb, flurry, mobilize, mobilize_value, on_attack, target_any, target_filtered,
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
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
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
                exclude_attacker_taps: false,
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
                pay_any_color: false, pay_own_cost: false,
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
            min_targets: 0,
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
            min_targets: 0,
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

// ── TDM batch 11: tutor / prowess / Harmonize draw ────────────────────────

/// Sarkhan's Triumph — {2}{R} Instant. Search your library for a Dragon creature
/// card, reveal it, put it into your hand, then shuffle.
pub fn sarkhans_triumph() -> CardDefinition {
    CardDefinition {
        name: "Sarkhan's Triumph",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Lotus-Eye Mystics — {3}{W} 3/2 Human Monk with Prowess. ETB: return target
/// enchantment card from your graveyard to your hand.
pub fn lotus_eye_mystics() -> CardDefinition {
    CardDefinition {
        name: "Lotus-Eye Mystics",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::InYourGraveyard.and(SelectionRequirement::Enchantment),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Winternight Stories — {2}{U} Sorcery. Draw three cards, then discard two.
/// Harmonize {4}{U}. (The "unless you discard a creature card" clause is
/// approximated as the plain discard-two downside.)
pub fn winternight_stories() -> CardDefinition {
    CardDefinition {
        name: "Winternight Stories",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(4), u()]))],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
        ]),
        ..Default::default()
    }
}

// ── TDM batch 12: Mobilize / Equipment / modal / landfall enchantment ─────



/// Heritage Reclamation — {1}{G} Instant. Choose one — destroy target artifact;
/// or destroy target enchantment; or exile target card from a graveyard, then
/// draw a card.
pub fn heritage_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Heritage Reclamation",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Enchantment) },
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::InGraveyard),
                    to: ZoneDest::Exile,
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        ]),
        ..Default::default()
    }
}


// ── TDM batch: simple commons/uncommons riding existing primitives ──────────

/// Dragon Sniper — {G} 1/1 Human Archer with vigilance, reach, deathtouch.
pub fn dragon_sniper() -> CardDefinition {
    CardDefinition {
        name: "Dragon Sniper",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance, Keyword::Reach, Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Twin Bolt — {1}{R} Instant. 2 damage divided as you choose among one or two
/// targets.
pub fn twin_bolt() -> CardDefinition {
    CardDefinition {
        name: "Twin Bolt",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamageDivided {
                retaliate_to_source: false,
            total: Value::Const(2),
            filter: SelectionRequirement::Any,
            max_targets: 2,
        },
        ..Default::default()
    }
}

/// Cruel Truths — {3}{B} Instant. Surveil 2, then draw two cards. You lose 2 life.
pub fn cruel_truths() -> CardDefinition {
    CardDefinition {
        name: "Cruel Truths",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Iceridge Serpent — {4}{U} 3/3 Serpent. ETB: return target creature an
/// opponent controls to its owner's hand.
pub fn iceridge_serpent() -> CardDefinition {
    CardDefinition {
        name: "Iceridge Serpent",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Serpent], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}

/// Loxodon Battle Priest — {4}{W} 3/5 Elephant Cleric. At the beginning of
/// combat on your turn, put a +1/+1 counter on another target creature you
/// control.
pub fn loxodon_battle_priest() -> CardDefinition {
    CardDefinition {
        name: "Loxodon Battle Priest",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Worthy Cost — {B} Sorcery. As an additional cost, sacrifice a creature.
/// Exile target creature or planeswalker.
pub fn worthy_cost() -> CardDefinition {
    CardDefinition {
        name: "Worthy Cost",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                to: ZoneDest::Exile,
            },
        ]),
        ..Default::default()
    }
}

/// Bearer of Glory — {1}{W} 2/1 Human Soldier. During your turn it has first
/// strike. {4}{W}: creatures you control get +1/+1 until end of turn.
pub fn bearer_of_glory() -> CardDefinition {
    CardDefinition {
        name: "Bearer of Glory",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Undergrowth Leopard — {1}{G} 2/2 Cat with vigilance. {1}, Sacrifice this:
/// destroy target artifact or enchantment.
pub fn undergrowth_leopard() -> CardDefinition {
    CardDefinition {
        name: "Undergrowth Leopard",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Summit Intimidator — {3}{R} 4/3 Yeti with reach. ETB: target creature can't
/// block this turn.
pub fn summit_intimidator() -> CardDefinition {
    CardDefinition {
        name: "Summit Intimidator",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Yeti], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Underfoot Underdogs — {2}{R} 1/2 Goblin Warrior. ETB: create a 1/1 red
/// Goblin. {1}, {T}: target creature you control with power 2 or less can't be
/// blocked this turn.
pub fn underfoot_underdogs() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Underfoot Underdogs",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Goblin".into(),
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Goblin],
                    ..Default::default()
                },
                colors: vec![Color::Red],
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtMost(2)),
                ),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rescue Leopard — {2}{R} 4/2 Cat. Whenever it becomes tapped, you may discard
/// a card. If you do, draw a card.
pub fn rescue_leopard() -> CardDefinition {
    CardDefinition {
        name: "Rescue Leopard",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Discard a card to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Salt Road Packbeast — {5}{W} 4/3 Beast with affinity for creatures. ETB:
/// draw a card.
pub fn salt_road_packbeast() -> CardDefinition {
    CardDefinition {
        name: "Salt Road Packbeast",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 3,
        affinity_filter: Some(SelectionRequirement::Creature),
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) })],
        ..Default::default()
    }
}

/// Humbling Elder — {U} 1/2 Human Monk with flash. ETB: target creature an
/// opponent controls gets -2/-0 until end of turn.
pub fn humbling_elder() -> CardDefinition {
    CardDefinition {
        name: "Humbling Elder",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Unsparing Boltcaster — {2}{R} 3/3 Ogre Wizard. ETB: deal 5 damage to target
/// creature an opponent controls that was dealt damage this turn.
pub fn unsparing_boltcaster() -> CardDefinition {
    CardDefinition {
        name: "Unsparing Boltcaster",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent)
                    .and(SelectionRequirement::DealtDamageThisTurn),
            ),
            amount: Value::Const(5),
        })],
        ..Default::default()
    }
}

/// Shocking Sharpshooter — {1}{R} 1/3 Human Archer with reach. Whenever another
/// creature you control enters, deal 1 damage to target opponent.
pub fn shocking_sharpshooter() -> CardDefinition {
    CardDefinition {
        name: "Shocking Sharpshooter",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Veteran Ice Climber — {1}{U} 1/3 Human Scout with vigilance that can't be
/// blocked. Whenever it attacks, up to one target player mills cards equal to
/// its power.
pub fn veteran_ice_climber() -> CardDefinition {
    CardDefinition {
        name: "Veteran Ice Climber",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Unblockable],
        triggered_abilities: vec![on_attack(Effect::Mill {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::PowerOf(Box::new(Selector::This)),
        })],
        ..Default::default()
    }
}

/// Trade Route Envoy — {3}{G} 4/3 Dog Soldier. ETB: draw a card if you control a
/// creature with a counter on it; otherwise put a +1/+1 counter on this.
pub fn trade_route_envoy() -> CardDefinition {
    CardDefinition {
        name: "Trade Route Envoy",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            else_: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        ..Default::default()
    }
}

/// Attuned Hunter — {2}{G} 3/3 Human Ranger with trample. Whenever one or more
/// cards leave your graveyard during your turn, put a +1/+1 counter on it.
pub fn attuned_hunter() -> CardDefinition {
    CardDefinition {
        name: "Attuned Hunter",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ranger],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::FromYourGraveyard)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Dragonologist — {2}{U} 1/3 Human Wizard. ETB: look at the top six cards;
/// you may put an instant, sorcery, or Dragon card from among them into your
/// hand, rest on the bottom in a random order. Untapped Dragons you control
/// have hexproof.
pub fn dragonologist() -> CardDefinition {
    CardDefinition {
        name: "Dragonologist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(6),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                    .or(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
            ),
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
        })],
        static_abilities: vec![StaticAbility {
            description: "Untapped Dragons you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Dragon)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Untapped),
                ),
                keyword: Keyword::Hexproof,
            },
        }],
        ..Default::default()
    }
}

/// Arashin Sunshield — {3}{W} 3/4 Human Warrior. ETB: exile up to two target
/// cards from a single graveyard. {W}, {T}: tap target creature.
pub fn arashin_sunshield() -> CardDefinition {
    CardDefinition {
        name: "Arashin Sunshield",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::ExileUpToNFromGraveyards { count: Value::Const(2), of: None, single: false })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Desperate Measures — {B} Instant. Target creature gets +1/-1 until end of
/// turn. When it dies this turn, draw two cards.
pub fn desperate_measures() -> CardDefinition {
    CardDefinition {
        name: "Desperate Measures",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                filter: None,
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                slot: 0,
            },
        ]),
        ..Default::default()
    }
}

// ── Exhale cycle (TDM "behold a Dragon") ────────────────────────────────────
//
// "Behold a Dragon" is an additional cost (choose a Dragon you control or
// reveal one from hand). We don't model the cast-time reveal; the unconditional
// half always resolves, and the "if a Dragon was beheld" rider is gated on
// controlling a Dragon (the common faithful case — hand-reveal is omitted).

/// Returns a predicate true when you control a Dragon (our "beheld" proxy).
fn beheld_a_dragon() -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Dragon)
                .and(SelectionRequirement::ControlledByYou),
        ),
        n: Value::Const(1),
    }
}

/// Caustic Exhale — {B} Instant. (Behold a Dragon or pay {1}.) Target creature
/// gets -3/-3 until end of turn.
pub fn caustic_exhale() -> CardDefinition {
    CardDefinition {
        name: "Caustic Exhale",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Osseous Exhale — {1}{W} Instant. Deal 5 damage to target attacking or
/// blocking creature. If a Dragon was beheld, you gain 2 life.
pub fn osseous_exhale() -> CardDefinition {
    CardDefinition {
        name: "Osseous Exhale",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                    ),
                ),
                amount: Value::Const(5),
            },
            Effect::If {
                cond: beheld_a_dragon(),
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Molten Exhale — {1}{R} Sorcery. Deal 4 damage to target creature or
/// planeswalker. (Castable as though it had flash if you behold a Dragon.)
pub fn molten_exhale() -> CardDefinition {
    CardDefinition {
        name: "Molten Exhale",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Dispelling Exhale — {1}{U} Instant. Counter target spell unless its
/// controller pays {2}. If a Dragon was beheld, pays {4} instead.
pub fn dispelling_exhale() -> CardDefinition {
    let counter = |amt: u32| Effect::CounterUnlessPaid {
        what: target_filtered(SelectionRequirement::IsSpellOnStack),
        mana_cost: cost(&[generic(amt)]),
        exile: false,
        extra_generic: None,
    };
    CardDefinition {
        name: "Dispelling Exhale",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: beheld_a_dragon(),
            then: Box::new(counter(4)),
            else_: Box::new(counter(2)),
        },
        ..Default::default()
    }
}

/// Piercing Exhale — {1}{G} Instant. Target creature you control deals damage
/// equal to its power to target creature or planeswalker. If a Dragon was
/// beheld, surveil 2.
pub fn piercing_exhale() -> CardDefinition {
    CardDefinition {
        name: "Piercing Exhale",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                },
                amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                })),
            },
            Effect::If {
                cond: beheld_a_dragon(),
                then: Box::new(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Monument cycle (ETB basic-land tutor + sorcery-speed sac payoff) ─────────

/// Shared ETB: search your library for one of three basic land types to hand.
fn monument_etb(a: crate::card::LandType, b: crate::card::LandType, c: crate::card::LandType) -> TriggeredAbility {
    etb(Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand.and(
            SelectionRequirement::HasLandType(a)
                .or(SelectionRequirement::HasLandType(b))
                .or(SelectionRequirement::HasLandType(c)),
        ),
        to: ZoneDest::Hand(PlayerRef::You),
    })
}

fn monument_sac(mana: crate::mana::ManaCost, payoff: Effect) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        sac_cost: true,
        sorcery_speed: true,
        mana_cost: mana,
        effect: payoff,
        ..Default::default()
    }
}

fn warrior_token() -> crate::card::TokenDefinition {
    use crate::card::TokenDefinition;
    TokenDefinition {
        name: "Warrior".into(),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Warrior], ..Default::default() },
        colors: vec![Color::Red],
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace, Keyword::Haste],
        ..Default::default()
    }
}

/// Jeskai Monument — {2} Artifact. ETB: tutor a basic Island/Mountain/Plains.
/// {1}{U}{R}{W}, {T}, Sacrifice: create two 1/1 white flying Birds.
pub fn jeskai_monument() -> CardDefinition {
    use crate::card::{LandType, TokenDefinition};
    CardDefinition {
        name: "Jeskai Monument",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![monument_etb(LandType::Island, LandType::Mountain, LandType::Plains)],
        activated_abilities: vec![monument_sac(
            cost(&[generic(1), u(), r(), w()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Bird".into(),
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
                    colors: vec![Color::White],
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
        )],
        ..Default::default()
    }
}

/// Mardu Monument — {2} Artifact. ETB: tutor a basic Mountain/Plains/Swamp.
/// {2}{R}{W}{B}, {T}, Sacrifice: create three 1/1 red Warriors with menace and
/// haste.
pub fn mardu_monument() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Mardu Monument",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![monument_etb(LandType::Mountain, LandType::Plains, LandType::Swamp)],
        activated_abilities: vec![monument_sac(
            cost(&[generic(2), r(), w(), b()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: warrior_token(),
            },
        )],
        ..Default::default()
    }
}

/// Sultai Monument — {2} Artifact. ETB: tutor a basic Swamp/Forest/Island.
/// {2}{B}{G}{U}, {T}, Sacrifice: create two 2/2 black Zombie Druids.
pub fn sultai_monument() -> CardDefinition {
    use crate::card::{LandType, TokenDefinition};
    CardDefinition {
        name: "Sultai Monument",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![monument_etb(LandType::Swamp, LandType::Forest, LandType::Island)],
        activated_abilities: vec![monument_sac(
            cost(&[generic(2), b(), g(), u()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
                    name: "Zombie Druid".into(),
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie, CreatureType::Druid],
                        ..Default::default()
                    },
                    colors: vec![Color::Black],
                    power: 2,
                    toughness: 2,
                    ..Default::default()
                },
            },
        )],
        ..Default::default()
    }
}

/// Temur Monument — {2} Artifact. ETB: tutor a basic Forest/Island/Mountain.
/// {3}{G}{U}{R}, {T}, Sacrifice: create a 5/5 green Elephant.
pub fn temur_monument() -> CardDefinition {
    use crate::card::{LandType, TokenDefinition};
    CardDefinition {
        name: "Temur Monument",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![monument_etb(LandType::Forest, LandType::Island, LandType::Mountain)],
        activated_abilities: vec![monument_sac(
            cost(&[generic(3), g(), u(), r()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Elephant".into(),
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Elephant], ..Default::default() },
                    colors: vec![Color::Green],
                    power: 5,
                    toughness: 5,
                    ..Default::default()
                },
            },
        )],
        ..Default::default()
    }
}

// ── Devotee cycle ({1}: add one of three colors, once each turn) ────────────

fn devotee_mana(colors: Vec<Color>) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        once_per_turn: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(colors, Value::Const(1)) },
        ..Default::default()
    }
}

/// Abzan Devotee — {1}{B} 2/2 Dog Cleric. {1}: add W/B/G (once each turn).
/// {2}{B}: return this from your graveyard to your hand.
pub fn abzan_devotee() -> CardDefinition {
    CardDefinition {
        name: "Abzan Devotee",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            devotee_mana(vec![Color::White, Color::Black, Color::Green]),
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                from_graveyard: true,
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Temur Devotee — {1}{U} 2/2 Defender. {1}: add G/U/R (once each turn).
pub fn temur_devotee() -> CardDefinition {
    CardDefinition {
        name: "Temur Devotee",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![devotee_mana(vec![Color::Green, Color::Blue, Color::Red])],
        ..Default::default()
    }
}

/// Sultai Devotee — {1}{G} 2/2 Deathtouch. {1}: add B/G/U (once each turn).
pub fn sultai_devotee() -> CardDefinition {
    CardDefinition {
        name: "Sultai Devotee",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Snake, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![devotee_mana(vec![Color::Black, Color::Green, Color::Blue])],
        ..Default::default()
    }
}

// ── Other white/blue/black creatures ────────────────────────────────────────

/// Tempest Hawk — {2}{W} 2/2 Bird with flying. Combat damage to a player: you
/// may search for a card named Tempest Hawk and put it into your hand.
pub fn tempest_hawk() -> CardDefinition {
    CardDefinition {
        name: "Tempest Hawk",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Search for another Tempest Hawk?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasName("Tempest Hawk".into()),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Starry-Eyed Skyrider — {2}{W} 1/3 Human Scout with flying. On attack, another
/// target creature you control gains flying until end of turn. Attacking tokens
/// you control have flying.
pub fn starry_eyed_skyrider() -> CardDefinition {
    CardDefinition {
        name: "Starry-Eyed Skyrider",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![StaticAbility {
            description: "Attacking tokens you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::IsToken
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsAttacking),
                ),
                keyword: Keyword::Flying,
            },
        }],
        ..Default::default()
    }
}

/// Aegis Sculptor — {3}{U} 2/3 Bird Wizard with flying and ward {2}. At the
/// beginning of your upkeep, you may exile two cards from your graveyard to put
/// a +1/+1 counter on it.
pub fn aegis_sculptor() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Aegis Sculptor",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Exile two cards from your graveyard to grow Aegis Sculptor?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachMatching {
                                zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                                filter: SelectionRequirement::Any,
                            }),
                            count: Box::new(Value::Const(2)),
                        },
                        to: ZoneDest::Exile,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Yathan Tombguard — {2}{B} 2/3 Human Warrior with menace. Whenever a creature
/// you control with a +1/+1 counter deals combat damage to a player, draw a
/// card and lose 1 life.
pub fn yathan_tombguard() -> CardDefinition {
    CardDefinition {
        name: "Yathan Tombguard",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                }),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Kishla Trawlers — {2}{U} 2/3 Human Citizen. ETB: you may exile a creature
/// card from your graveyard; when you do, return target instant or sorcery card
/// from your graveyard to your hand.
pub fn kishla_trawlers() -> CardDefinition {
    CardDefinition {
        name: "Kishla Trawlers",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Exile a creature card from your graveyard to return an instant/sorcery?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::EachMatching {
                            zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                            filter: SelectionRequirement::Creature,
                        }),
                        count: Box::new(Value::Const(1)),
                    },
                    to: ZoneDest::Exile,
                },
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::InYourGraveyard.and(
                            SelectionRequirement::HasCardType(CardType::Instant)
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        ),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Sunpearl Kirin — {1}{W} 2/1 Kirin with flash and flying. ETB: return up to
/// one other target nonland permanent you control to its owner's hand.
/// (The "if it was a token, draw a card" rider is omitted — the bounced object
/// is gone before the rider can read it.)
pub fn sunpearl_kirin() -> CardDefinition {
    CardDefinition {
        name: "Sunpearl Kirin",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kirin], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: SelectionRequirement::Nonland
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..Default::default()
    }
}

/// Formation Breaker — {1}{G} 2/1 Beast. Creatures with power less than its
/// power can't block it; gets +1/+2 while you control a creature with a counter.
pub fn formation_breaker() -> CardDefinition {
    CardDefinition {
        name: "Formation Breaker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::CantBeBlockedByPowerLess],
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+2 while you control a creature with a counter on it.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
                    ),
                    n: Value::Const(1),
                },
                power: 1,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Krotiq Nestguard — {2}{G} 4/4 Insect with defender. {2}{G}: it can attack
/// this turn as though it didn't have defender.
pub fn krotiq_nestguard() -> CardDefinition {
    CardDefinition {
        name: "Krotiq Nestguard",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Snowmelt Stag — {3}{U} 2/5 Elemental Elk with vigilance. During your turn it
/// has base power and toughness 5/2. {5}{U}{U}: it can't be blocked this turn.
pub fn snowmelt_stag() -> CardDefinition {
    CardDefinition {
        name: "Snowmelt Stag",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Elk],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has base power and toughness 5/2.",
            effect: StaticEffect::SetBasePtIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 5,
                toughness: 2,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u(), u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── TDM spells batch ────────────────────────────────────────────────────────

/// Knockout Maneuver — {2}{G} Sorcery. Put a +1/+1 counter on target creature
/// you control, then it deals damage equal to its power to target creature an
/// opponent controls.
pub fn knockout_maneuver() -> CardDefinition {
    CardDefinition {
        name: "Knockout Maneuver",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Rebellious Strike — {1}{W} Instant. Target creature gets +3/+0 until end of
/// turn. Draw a card.
pub fn rebellious_strike() -> CardDefinition {
    CardDefinition {
        name: "Rebellious Strike",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Lightfoot Technique — {1}{W} Instant. Put a +1/+1 counter on target creature;
/// it gains flying and indestructible until end of turn.
pub fn lightfoot_technique() -> CardDefinition {
    CardDefinition {
        name: "Lightfoot Technique",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Narset's Rebuke — {4}{R} Instant. Deal 5 damage to target creature. Add
/// {U}{R}{W}. If that creature would die this turn, exile it instead.
pub fn narsets_rebuke() -> CardDefinition {
    CardDefinition {
        name: "Narset's Rebuke",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Stamp the finality counter before the damage so a lethal hit
            // exiles instead of going to the graveyard.
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::Finality,
                amount: Value::Const(1),
            },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Blue, Color::Red, Color::White]),
            },
        ]),
        ..Default::default()
    }
}

/// Wail of War — {2}{B} Instant. Choose one — creatures your opponents control
/// get -1/-1 until end of turn; or return up to two target creature cards from
/// your graveyard to your hand.
pub fn wail_of_war() -> CardDefinition {
    CardDefinition {
        name: "Wail of War",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Bewildering Blizzard — {4}{U}{U} Instant. Draw three cards. Creatures your
/// opponents control get -3/-0 until end of turn.
pub fn bewildering_blizzard() -> CardDefinition {
    CardDefinition {
        name: "Bewildering Blizzard",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Duty Beyond Death — {1}{W} Instant. As an additional cost, sacrifice a
/// creature. Creatures you control gain indestructible until end of turn; put a
/// +1/+1 counter on each creature you control.
pub fn duty_beyond_death() -> CardDefinition {
    let your_creatures = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Duty Beyond Death",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::GrantKeyword {
                what: your_creatures(),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::AddCounter {
                what: your_creatures(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}
