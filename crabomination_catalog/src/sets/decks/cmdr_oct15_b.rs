//! Commander: one-card gaps of the 2026-10-15 afternoon scan (all 6,799
//! EDHREC commanders). Tests at the end of `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{enlist, etb, flurry, on_attack, target_filtered};
use crate::effect::DelayedTriggerKind;
use crate::effect::{ActivatedAbility, Duration, Effect, LookExileGrant, PlayerRef, Selector, ZoneDest};
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

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

/// Screeching Soulbreaker — {2}{B} 1/4 flying Siren Bard. Attacking: 1 damage
/// to each opponent and you gain 1 life.
pub fn screeching_soulbreaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..creature("Screeching Soulbreaker", cost(&[generic(2), b()]), vec![CreatureType::Siren, CreatureType::Bard], 1, 4)
    }
}

/// Goldbug, Scrappy Scout — the Vehicle back (1/3, living metal). Human spells
/// you control can't be countered; whenever it and at least one Human attack,
/// draw a card and convert it.
pub fn goldbug_scrappy_scout() -> CardDefinition {
    let attacking_human = R::IsAttacking.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou);
    CardDefinition {
        name: "Goldbug, Scrappy Scout",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        color_indicator: vec![Color::White, Color::Blue],
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::LivingMetal],
        static_abilities: vec![StaticAbility {
            description: "Human spells you control can't be countered.",
            effect: StaticEffect::SpellsUncounterable { filter: R::HasCreatureType(CreatureType::Human) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::EachPermanent(attacking_human))),
                Value::ONE,
            )),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Transform { what: Selector::This },
            ]),
        }],
        ..Default::default()
    }
}

/// Goldbug, Humanity's Ally — {1}{W}{U} legendary 3/3 artifact creature Robot;
/// More Than Meets the Eye {W}{U}. Prevent all combat damage that would be
/// dealt to attacking Humans you control; your second spell each turn
/// converts it.
pub fn goldbug_humanitys_ally() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        alternative_cost: Some(AlternativeCost { mana_cost: cost(&[w(), u()]), converted: true, ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Prevent all combat damage that would be dealt to attacking Humans you control.",
            effect: StaticEffect::PreventAllCombatDamageToMatching {
                filter: R::IsAttacking.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou),
            },
        }],
        triggered_abilities: vec![flurry(Effect::Transform { what: Selector::This })],
        back_face: Some(Box::new(goldbug_scrappy_scout())),
        ..creature("Goldbug, Humanity's Ally", cost(&[generic(1), w(), u()]), vec![CreatureType::Robot], 3, 3)
    }
}

/// Keldon Flamesage — {2}{R} 2/3 Human Shaman with enlist. Attacking: look at
/// the top X (X = its power); you may exile an instant or sorcery card with
/// mana value X or less from among them and cast it free; the rest go to the
/// bottom in a random order.
pub fn keldon_flamesage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::LookTopExileOneMayPlay {
                count: Value::PowerOf(Box::new(Selector::This)),
                who: PlayerRef::You,
                grant: LookExileGrant::InstantSorceryMvAtMostCountFree,
            }),
            enlist(),
        ],
        ..creature("Keldon Flamesage", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 3)
    }
}

/// Animus of Night's Reach — the back: 0/4 black enchantment creature Spirit,
/// menace; attacking, +X/+0 where X is the creature cards in the defending
/// player's graveyard.
pub fn animus_of_nights_reach() -> CardDefinition {
    CardDefinition {
        name: "Animus of Night's Reach",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        color_indicator: vec![Color::Black],
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::CardsInGraveyardMatching { who: PlayerRef::DefendingPlayer, filter: R::Creature },
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// The Long Reach of Night — {3}{B} Saga. I, II: each opponent sacrifices a
/// creature of their choice unless they discard a card. III: exile it, then
/// return it transformed (Animus of Night's Reach).
pub fn the_long_reach_of_night() -> CardDefinition {
    let squeeze = || Effect::Punisher {
        chooser: Selector::Player(PlayerRef::EachOpponent),
        options: vec![Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }],
        otherwise: Box::new(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::Triggerer),
            count: Value::ONE,
            filter: R::Creature,
        }),
    };
    CardDefinition {
        name: "The Long Reach of Night",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, squeeze()), (2, squeeze()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(animus_of_nights_reach())),
        ..Default::default()
    }
}

/// Ib Halfheart, Goblin Tactician — {3}{R} legendary 3/2 Goblin Advisor.
/// Whenever another Goblin you control becomes blocked, sacrifice it; if you
/// do, it deals 4 damage to each creature blocking it. Sacrifice two
/// Mountains: two 1/1 red Goblins. Approximation: the damage is dealt just
/// before the sacrifice (the blockers leave the block map with the attacker),
/// and only while that Goblin is still yours on the battlefield.
pub fn ib_halfheart_goblin_tactician() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    };
    let still_ours = trigger_is(R::ControlledByYou);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl)
                .with_filter(trigger_is(R::HasCreatureType(CreatureType::Goblin).and(R::OtherThanSource))),
            effect: Effect::If {
                cond: still_ours,
                then: Box::new(Effect::Seq(vec![
                    Effect::DealDamageFrom {
                        source: Selector::TriggerSource,
                        to: Selector::CreaturesBlockingTriggerSource,
                        amount: Value::Const(4),
                    },
                    Effect::SacrificeSelected { what: Selector::TriggerSource },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasLandType(LandType::Mountain), 2)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(goblin) },
            ..Default::default()
        }],
        ..creature(
            "Ib Halfheart, Goblin Tactician",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Advisor],
            3,
            2,
        )
    }
}

/// Destined Confrontation — {2}{W}{W} sorcery. Each player keeps any creatures
/// they control with total power 4 or less and sacrifices the rest.
pub fn destined_confrontation() -> CardDefinition {
    CardDefinition {
        name: "Destined Confrontation",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerKeepsCreaturesTotalPower { max_power: 4 },
        ..Default::default()
    }
}

/// Enduring Renewal — {2}{W}{W} enchantment. Play with your hand revealed. A
/// draw reveals the top card instead: a creature card goes to your graveyard,
/// anything else is drawn. A creature put into your graveyard from the
/// battlefield returns to your hand.
pub fn enduring_renewal() -> CardDefinition {
    CardDefinition {
        name: "Enduring Renewal",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Play with your hand revealed.",
                effect: StaticEffect::ControllerPlaysWithHandRevealed,
            },
            StaticAbility {
                description: "If you would draw a card, reveal the top card of your library instead. If it's a creature card, put it into your graveyard. Otherwise, draw a card.",
                effect: StaticEffect::ReplaceDrawCreaturesToGraveyard,
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer)
                .with_filter(trigger_is(R::Creature.and(R::OwnedByYou))),
            effect: Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..Default::default()
    }
}

/// Ancient Brontodon — {6}{G}{G} 9/9 Dinosaur.
pub fn ancient_brontodon() -> CardDefinition {
    creature("Ancient Brontodon", cost(&[generic(6), g(), g()]), vec![CreatureType::Dinosaur], 9, 9)
}

/// Garruk's Gorehorn — {4}{G} 7/3 Beast.
pub fn garruks_gorehorn() -> CardDefinition {
    creature("Garruk's Gorehorn", cost(&[generic(4), g()]), vec![CreatureType::Beast], 7, 3)
}

/// Armored Warhorse — {W}{W} 2/3 Horse.
pub fn armored_warhorse() -> CardDefinition {
    creature("Armored Warhorse", cost(&[w(), w()]), vec![CreatureType::Horse], 2, 3)
}

/// Amphin Cutthroat — {3}{U} 2/4 Salamander Rogue.
pub fn amphin_cutthroat() -> CardDefinition {
    creature("Amphin Cutthroat", cost(&[generic(3), u()]), vec![CreatureType::Salamander, CreatureType::Rogue], 2, 4)
}

/// Cloistered Telepath — {1}{U} 2/1 flash Talosian Advisor. Enters: draw a
/// card, then discard a card.
pub fn cloistered_telepath() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]))],
        ..creature("Cloistered Telepath", cost(&[generic(1), u()]), vec![CreatureType::Talosian, CreatureType::Advisor], 2, 1)
    }
}

/// Muraganda Petroglyphs — {3}{G} enchantment. Creatures with no abilities get
/// +2/+2.
pub fn muraganda_petroglyphs() -> CardDefinition {
    CardDefinition {
        name: "Muraganda Petroglyphs",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures with no abilities get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasNoAbilities),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                all_players: true,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Glint Hawk — {W} 2/2 flying Bird. Enters: sacrifice it unless you return an
/// artifact you control to its owner's hand.
pub fn glint_hawk() -> CardDefinition {
    let mine = || Selector::EachPermanent(R::Artifact.and(R::ControlledByYou));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorExists(mine()),
            then: Box::new(Effect::MoveChosen {
                from: mine(),
                filter: None,
                count: Value::ONE,
                up_to: false,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
            else_: Box::new(Effect::SacrificeSource),
        })],
        ..creature("Glint Hawk", cost(&[w()]), vec![CreatureType::Bird], 2, 2)
    }
}

/// Salvage Scout — {W} 1/1 Human Scout. {W}, sacrifice it: return target
/// artifact card from your graveyard to your hand.
pub fn salvage_scout() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature("Salvage Scout", cost(&[w()]), vec![CreatureType::Human, CreatureType::Scout], 1, 1)
    }
}

/// Argent Sphinx — {2}{U}{U} 4/3 flying Sphinx. Metalcraft — {U}: exile it;
/// return it under your control at the beginning of the next end step.
pub fn argent_sphinx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            condition: Some(Predicate::MetalcraftActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Argent Sphinx", cost(&[generic(2), u(), u()]), vec![CreatureType::Sphinx], 4, 3)
    }
}

/// Stone Retrieval Unit — {4} 2/3 artifact creature Construct. Enters: a
/// tapped Powerstone token.
pub fn stone_retrieval_unit() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }),
        })],
        ..creature("Stone Retrieval Unit", cost(&[generic(4)]), vec![CreatureType::Construct], 2, 3)
    }
}

/// Black Poplar Shaman — {2}{B} 1/3 Treefolk Shaman. {2}{B}: regenerate target
/// Treefolk.
pub fn black_poplar_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Regenerate { what: target_filtered(R::HasCreatureType(CreatureType::Treefolk)) },
            ..Default::default()
        }],
        ..creature("Black Poplar Shaman", cost(&[generic(2), b()]), vec![CreatureType::Treefolk, CreatureType::Shaman], 1, 3)
    }
}

/// Patrol Signaler — {1}{W} 1/1 Kithkin Soldier. {1}{W}, {Q}: a 1/1 white
/// Kithkin Soldier.
pub fn patrol_signaler() -> CardDefinition {
    let kithkin = TokenDefinition {
        name: "Kithkin Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin, CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            untap_self_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(kithkin) },
            ..Default::default()
        }],
        ..creature("Patrol Signaler", cost(&[generic(1), w()]), vec![CreatureType::Kithkin, CreatureType::Soldier], 1, 1)
    }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// "As long as enchanted creature is `color`, `inner`."
fn while_enchanted_is(description: &'static str, color: Color, inner: StaticEffect) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::WhileCondition {
            condition: Predicate::EntityMatches { what: enchanted(), filter: R::HasColor(color) },
            inner: Box::new(inner),
        },
    }
}

/// Gift of the Deity — {4}{B/G} Aura. Black: +1/+1 and deathtouch. Green:
/// +1/+1 and all creatures able to block it do so.
pub fn gift_of_the_deity() -> CardDefinition {
    let pump = || StaticEffect::PumpPT { applies_to: enchanted(), power: 1, toughness: 1 };
    let has = |keyword| StaticEffect::GrantKeyword { applies_to: enchanted(), keyword };
    CardDefinition {
        name: "Gift of the Deity",
        cost: cost(&[generic(4), hybrid(Color::Black, Color::Green)]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![
            while_enchanted_is("As long as enchanted creature is black, it gets +1/+1.", Color::Black, pump()),
            while_enchanted_is("As long as enchanted creature is black, it has deathtouch.", Color::Black, has(Keyword::Deathtouch)),
            while_enchanted_is("As long as enchanted creature is green, it gets +1/+1.", Color::Green, pump()),
            while_enchanted_is(
                "As long as enchanted creature is green, all creatures able to block it do so.",
                Color::Green,
                has(Keyword::AllMustBlock),
            ),
        ],
        ..Default::default()
    }
}

/// Crush Underfoot — {1}{R} Kindred instant — Giant. Choose a Giant creature
/// you control; it deals damage equal to its power to target creature.
pub fn crush_underfoot() -> CardDefinition {
    CardDefinition {
        name: "Crush Underfoot",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        effect: Effect::ChooseOneAmong {
            what: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Giant)).and(R::ControlledByYou)),
            chooser: PlayerRef::You,
            chosen: Box::new(Effect::DealDamageEqualToPower {
                source: Selector::SeparatedPile { chosen: true },
                target: target_filtered(R::Creature),
            }),
            other: Box::new(Effect::Noop),
        },
        ..Default::default()
    }
}

/// Touch of Moonglove — {B} instant. Target creature you control gets +1/+0
/// and deathtouch until end of turn; whenever a creature it dealt damage to
/// dies this turn, that creature's controller loses 2 life. Residual: the
/// granted trigger doesn't look back (CR 603.10a) when the pumped creature
/// dies in the same event as its victim.
pub fn touch_of_moonglove() -> CardDefinition {
    let toll = TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer)
            .with_filter(trigger_is(R::Creature.and(R::DamagedBySourceThisTurn))),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
            amount: Value::Const(2),
        },
    };
    CardDefinition {
        name: "Touch of Moonglove",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility { what: Selector::Target(0), trigger: Box::new(toll), duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

fn archer() -> R {
    R::HasCreatureType(CreatureType::Archer)
}

/// Greatbow Doyen — {4}{G} 2/4 Elf Archer. Other Archers you control get
/// +1/+1. Whenever an Archer you control deals damage to a creature, that
/// Archer deals that much damage to that creature's controller.
pub fn greatbow_doyen() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Archer creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(archer().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::YourControl).with_filter(trigger_is(archer())),
            effect: Effect::DealDamageFrom {
                source: Selector::TriggerSource,
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Greatbow Doyen", cost(&[generic(4), g()]), vec![CreatureType::Elf, CreatureType::Archer], 2, 4)
    }
}

/// Ohabi Caleria — {1}{G}{W} legendary 1/3 reach Elf Archer. Untap your
/// Archers during each other player's untap step. Whenever an Archer you
/// control deals damage to a creature, you may pay {2} to draw a card.
pub fn ohabi_caleria() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "Untap all Archers you control during each other player's untap step.",
            effect: StaticEffect::UntapYoursEachUntapStepFiltered(archer()),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::YourControl).with_filter(trigger_is(archer())),
            effect: Effect::MayPay {
                description: "Pay {2} to draw a card?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..creature("Ohabi Caleria", cost(&[generic(1), g(), w()]), vec![CreatureType::Elf, CreatureType::Archer], 1, 3)
    }
}

/// Gnostro, Voice of the Crags — {1}{U}{R}{W} legendary 3/3 Chimera. {T}:
/// choose one, X = spells you've cast this turn — scry X; X damage to target
/// creature; or gain X life.
pub fn gnostro_voice_of_the_crags() -> CardDefinition {
    let x = || Value::SpellsCastThisTurn(PlayerRef::You);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::ChooseMode(vec![
                Effect::Scry { who: PlayerRef::You, amount: x() },
                Effect::DealDamage { to: target_filtered(R::Creature), amount: x() },
                Effect::GainLife { who: Selector::You, amount: x() },
            ]),
            ..Default::default()
        }],
        ..creature("Gnostro, Voice of the Crags", cost(&[generic(1), u(), r(), w()]), vec![CreatureType::Chimera], 3, 3)
    }
}

/// Swans of Bryn Argoll — {2}{W/U}{W/U} 4/3 flying Bird Spirit. Damage to it is
/// prevented; the source's controller draws that many cards.
pub fn swans_of_bryn_argoll() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to this creature, prevent that damage. The source's controller draws cards equal to the damage prevented this way.",
            effect: StaticEffect::PreventDamageToSelfSourceControllerDraws,
        }],
        ..creature(
            "Swans of Bryn Argoll",
            cost(&[generic(2), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Bird, CreatureType::Spirit],
            4,
            3,
        )
    }
}
