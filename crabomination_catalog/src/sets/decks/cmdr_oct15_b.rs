//! Commander: one-card gaps of the 2026-10-15 afternoon scan (all 6,799
//! EDHREC commanders). Tests at the end of `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, LoyaltyAbility, PlaneswalkerSubtype, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{enlist, enrage, etb, flurry, on_attack, target_filtered};
use crate::effect::DelayedTriggerKind;
use crate::effect::{ActivatedAbility, Duration, Effect, LookExileGrant, ManaPayload, PlayerRef, Selector, ZoneDest};
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

/// Dragon-Scarred Bear — {2}{G} 3/2 Bear. Formidable — {1}{G}: regenerate it.
pub fn dragon_scarred_bear() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            condition: Some(Predicate::FormidableActive { who: PlayerRef::You }),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Dragon-Scarred Bear", cost(&[generic(2), g()]), vec![CreatureType::Bear], 3, 2)
    }
}

/// River Bear — {3}{G} 3/3 Bear with islandwalk.
pub fn river_bear() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        ..creature("River Bear", cost(&[generic(3), g()]), vec![CreatureType::Bear], 3, 3)
    }
}

/// Cacophodon — {3}{G} 2/5 Dinosaur. Enrage — untap target permanent.
pub fn cacophodon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::Untap { what: target_filtered(R::Permanent), up_to: None })],
        ..creature("Cacophodon", cost(&[generic(3), g()]), vec![CreatureType::Dinosaur], 2, 5)
    }
}

/// Needletooth Raptor — {3}{R} 2/2 Dinosaur. Enrage — 5 damage to target
/// creature an opponent controls.
pub fn needletooth_raptor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            amount: Value::Const(5),
        })],
        ..creature("Needletooth Raptor", cost(&[generic(3), r()]), vec![CreatureType::Dinosaur], 2, 2)
    }
}

/// Alpine Watchdog — {1}{W} 2/2 vigilance Dog.
pub fn alpine_watchdog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature("Alpine Watchdog", cost(&[generic(1), w()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Blink Dog — {2}{W} 1/1 double strike Dog. Teleport — {3}{W}: it phases out.
pub fn blink_dog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
            ..Default::default()
        }],
        ..creature("Blink Dog", cost(&[generic(2), w()]), vec![CreatureType::Dog], 1, 1)
    }
}

/// Pitiless Gorgon — {1}{B/G}{B/G} 2/2 deathtouch Gorgon.
pub fn pitiless_gorgon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        ..creature(
            "Pitiless Gorgon",
            cost(&[generic(1), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
            vec![CreatureType::Gorgon],
            2,
            2,
        )
    }
}

/// Xathrid Gorgon — {5}{B} 3/6 deathtouch Gorgon. {2}{B}, {T}: a
/// petrification counter on target creature; it gains defender, becomes a
/// colorless artifact in addition to its other types, and its activated
/// abilities can't be activated. Residual: its mana abilities still can.
pub fn xathrid_gorgon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::Petrification, amount: Value::ONE },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Defender, duration: Duration::Permanent },
                Effect::AddCardTypeIndefinitely { what: Selector::Target(0), card_type: CardType::Artifact, until_eot: false },
                Effect::BecomeColor { what: Selector::Target(0), colors: vec![], duration: Duration::Permanent, additive: false },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantActivateAbilities,
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Xathrid Gorgon", cost(&[generic(5), b()]), vec![CreatureType::Gorgon], 3, 6)
    }
}

fn you_control_a_legendary_creature() -> Predicate {
    Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
    ))
}

/// Errand-Rider of Gondor — {2}{W} 3/2 Human Soldier. Enters: draw a card;
/// then without a legendary creature, put a card from your hand on the bottom.
pub fn errand_rider_of_gondor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::If {
                cond: Predicate::Not(Box::new(you_control_a_legendary_creature())),
                then: Box::new(Effect::PutCardsFromHandOnBottom { who: Selector::You, count: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Errand-Rider of Gondor", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 2)
    }
}

/// Esquire of the King — {W} 1/1 Human Soldier. {4}{W}, {T}: creatures you
/// control get +1/+1 until end of turn; {2} less with a legendary creature.
pub fn esquire_of_the_king() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            tap_cost: true,
            cost_reduction_if_control: Some((R::Creature.and(R::HasSupertype(Supertype::Legendary)), 2)),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Esquire of the King", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Recruit: draw a card, then discard a card; a discarded nonland card makes
/// a 1/1 white Human Soldier.
fn recruit() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        Effect::If {
            cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
            then: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Human Soldier".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            }),
            else_: Box::new(Effect::Noop),
        },
    ])
}

/// Patient Instructor — {2}{W/U} 2/2 vigilance Human Citizen. Enters: recruit.
pub fn patient_instructor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(recruit())],
        ..creature(
            "Patient Instructor",
            cost(&[generic(2), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Human, CreatureType::Citizen],
            2,
            2,
        )
    }
}

/// Eerie Soultender — {2}{B} 3/1 Spirit Cleric. Enters: mill three. {4}{B},
/// exile it from your graveyard: return another target creature card from
/// your graveyard to your hand.
pub fn eerie_soultender() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(3) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature("Eerie Soultender", cost(&[generic(2), b()]), vec![CreatureType::Spirit, CreatureType::Cleric], 3, 1)
    }
}

/// Spider-Man Noir — {4}{B} legendary 4/4 menace Spider Human Hero. Whenever a
/// creature you control attacks alone, a +1/+1 counter on it, then surveil X
/// (X = counters on it).
pub fn spider_man_noir() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
                },
            ]),
        }],
        ..creature(
            "Spider-Man Noir",
            cost(&[generic(4), b()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    }
}

/// Mister Immortal — {1}{G} legendary 2/1 Mutant Hero. {2}{G}: return it from
/// your graveyard or from exile to the battlefield tapped.
pub fn mister_immortal() -> CardDefinition {
    let back = |from_graveyard: bool| ActivatedAbility {
        mana_cost: cost(&[generic(2), g()]),
        from_graveyard,
        from_exile: !from_graveyard,
        effect: Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true } },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![back(true), back(false)],
        ..creature("Mister Immortal", cost(&[generic(1), g()]), vec![CreatureType::Mutant, CreatureType::Hero], 2, 1)
    }
}

/// Flatman — {3}{G} legendary 1/10 Mutant Hero. Origami-Fu — {2}{G}: switch
/// its power and toughness until end of turn.
pub fn flatman() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::SwitchPT { what: Selector::This, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Flatman", cost(&[generic(3), g()]), vec![CreatureType::Mutant, CreatureType::Hero], 1, 10)
    }
}

/// Pestered Wellguard — {3}{U} 3/2 Merfolk Soldier. Whenever it becomes
/// tapped, a 1/1 blue and black flying Faerie.
pub fn pestered_wellguard() -> CardDefinition {
    let faerie = TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(faerie) },
        }],
        ..creature("Pestered Wellguard", cost(&[generic(3), u()]), vec![CreatureType::Merfolk, CreatureType::Soldier], 3, 2)
    }
}

/// Tocasia, Dig Site Mentor — {1}{G}{W}{U} legendary 4/3 Human Artificer.
/// Creatures you control have vigilance and "{T}: Surveil 1." From your
/// graveyard, {2}{G}{G}{W}{W}{U}{U}, exile it: return artifact cards with
/// total mana value 10 or less from your graveyard to the battlefield
/// (sorcery speed). Residual: the return picks cheapest-first rather than
/// targeting.
pub fn tocasia_dig_site_mentor() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(), keyword: Keyword::Vigilance },
            },
            StaticAbility {
                description: "Creatures you control have \"{T}: Surveil 1.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: yours(),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g(), g(), w(), w(), u(), u()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::MoveWithinTotalManaValue {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Artifact },
                filter: R::Artifact,
                cap: Value::Const(10),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                max_count: None,
                by_power: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Tocasia, Dig Site Mentor",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            4,
            3,
        )
    }
}

// ── Three-card-gap decks: Lady Orca, Ruxa, Mirri, Jedit, Galina, Terror,
// Lo and Li, Casey Jones, Masumaro, Kopala ─────────────────────────────────

/// Canal Monitor — {4}{B} 5/3 Lizard.
pub fn canal_monitor() -> CardDefinition {
    creature("Canal Monitor", cost(&[generic(4), b()]), vec![CreatureType::Lizard], 5, 3)
}

/// Minotaur Warrior — {2}{R} 2/3 Minotaur Warrior.
pub fn minotaur_warrior() -> CardDefinition {
    creature("Minotaur Warrior", cost(&[generic(2), r()]), vec![CreatureType::Minotaur, CreatureType::Warrior], 2, 3)
}

/// Rotting Mastodon — {4}{B} 2/8 Zombie Elephant.
pub fn rotting_mastodon() -> CardDefinition {
    creature("Rotting Mastodon", cost(&[generic(4), b()]), vec![CreatureType::Zombie, CreatureType::Elephant], 2, 8)
}

/// Orazca Frillback — {2}{G} 4/2 Dinosaur.
pub fn orazca_frillback() -> CardDefinition {
    creature("Orazca Frillback", cost(&[generic(2), g()]), vec![CreatureType::Dinosaur], 4, 2)
}

/// Terrain Elemental — {1}{G} 3/2 Elemental.
pub fn terrain_elemental() -> CardDefinition {
    creature("Terrain Elemental", cost(&[generic(1), g()]), vec![CreatureType::Elemental], 3, 2)
}

/// Howl of the Night Pack — {6}{G} sorcery. A 2/2 green Wolf per Forest you
/// control.
pub fn howl_of_the_night_pack() -> CardDefinition {
    let wolf = TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Howl of the Night Pack",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::count(Selector::EachPermanent(R::HasLandType(LandType::Forest).and(R::ControlledByYou))),
            definition: Arc::new(wolf),
        },
        ..Default::default()
    }
}

/// Elite Cat Warrior — {2}{G} 2/3 Cat Warrior with forestwalk.
pub fn elite_cat_warrior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature("Elite Cat Warrior", cost(&[generic(2), g()]), vec![CreatureType::Cat, CreatureType::Warrior], 2, 3)
    }
}

/// Lynx — {1}{G} 2/1 Cat with forestwalk.
pub fn lynx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature("Lynx", cost(&[generic(1), g()]), vec![CreatureType::Cat], 2, 1)
    }
}

/// Lush Growth — {G} Aura, enchant land. Enchanted land is a Mountain, Forest,
/// and Plains.
pub fn lush_growth() -> CardDefinition {
    let changer = |land_type, replace| StaticEffect::LandTypeChanger { applies_to: enchanted(), land_type, replace };
    CardDefinition {
        name: "Lush Growth",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Land) },
        static_abilities: vec![
            StaticAbility { description: "Enchanted land is a Mountain, Forest, and Plains.", effect: changer(LandType::Mountain, true) },
            StaticAbility { description: "", effect: changer(LandType::Forest, false) },
            StaticAbility { description: "", effect: changer(LandType::Plains, false) },
        ],
        ..Default::default()
    }
}

/// Thelonite Monk — {2}{G}{G} 1/2 Insect Monk Cleric. {T}, sacrifice a green
/// creature: target land becomes a Forest (indefinitely).
pub fn thelonite_monk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::HasColor(Color::Green)), 1)),
            sac_other_may_be_source: true,
            effect: Effect::BecomeBasicLand { what: target_filtered(R::Land), land_type: LandType::Forest, duration: Duration::Permanent },
            ..Default::default()
        }],
        ..creature(
            "Thelonite Monk",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Insect, CreatureType::Monk, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Jace, Ingenious Mind-Mage — {4}{U}{U} legendary planeswalker, loyalty 5.
/// +1: draw a card. +1: untap all creatures you control. −9: gain control of
/// up to three target creatures.
pub fn jace_ingenious_mind_mage() -> CardDefinition {
    let steal = |slot| Effect::GainControl {
        what: Selector::TargetFiltered { slot, filter: R::Creature },
        to: None,
        duration: Duration::Permanent,
    };
    CardDefinition {
        name: "Jace, Ingenious Mind-Mage",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility { loyalty_cost: 1, effect: Effect::Draw { who: Selector::You, amount: Value::ONE }, ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)), up_to: None },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -9,
                effect: Effect::OptionalTargets { min: 0, body: Box::new(Effect::Seq(vec![steal(0), steal(1), steal(2)])) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Invoke the Winds — {1}{U}{U}{U}{U} sorcery. Gain control of target artifact
/// or creature and untap it.
pub fn invoke_the_winds() -> CardDefinition {
    CardDefinition {
        name: "Invoke the Winds",
        cost: cost(&[generic(1), u(), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Artifact.or(R::Creature)), to: None, duration: Duration::Permanent },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// The Terror of Serpent's Pass — {5}{U}{U} legendary 8/8 hexproof Serpent.
pub fn the_terror_of_serpents_pass() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Hexproof],
        ..creature("The Terror of Serpent's Pass", cost(&[generic(5), u(), u()]), vec![CreatureType::Serpent], 8, 8)
    }
}

/// Take Flight — {3}{U} Aura. Enchanted creature gets +1/+0, flying, and
/// "Whenever this creature attacks, draw a card."
pub fn take_flight() -> CardDefinition {
    CardDefinition {
        name: "Take Flight",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+0.",
                effect: StaticEffect::PumpPT { applies_to: enchanted(), power: 1, toughness: 0 },
            },
            StaticAbility {
                description: "Enchanted creature has flying.",
                effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword: Keyword::Flying },
            },
        ],
        equipped_bonus: Some(crate::card::EquipBonus {
            triggered_abilities: vec![on_attack(Effect::Draw { who: Selector::You, amount: Value::ONE })],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Displacement Wave — {X}{U}{U} sorcery. Return all nonland permanents with
/// mana value X or less to their owners' hands.
pub fn displacement_wave() -> CardDefinition {
    CardDefinition {
        name: "Displacement Wave",
        cost: cost(&[crate::mana::x(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachPermanent(R::Nonland.and(R::ManaValueAtMostXFromCost)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Malevolent Noble — {1}{B} 2/2 Human Noble. {2}, sacrifice an artifact or
/// another creature: a +1/+1 counter on it.
pub fn malevolent_noble() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Malevolent Noble", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Noble], 2, 2)
    }
}

/// Vampire Sovereign — {3}{B}{B} 3/4 flying Vampire Noble. Enters: target
/// opponent loses 3 life and you gain 3.
pub fn vampire_sovereign() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Drain {
            from: target_filtered(R::OpponentPlayer),
            to: Selector::You,
            amount: Value::Const(3),
        })],
        ..creature("Vampire Sovereign", cost(&[generic(3), b(), b()]), vec![CreatureType::Vampire, CreatureType::Noble], 3, 4)
    }
}

/// Wicked Guardian — {3}{B} 4/2 Human Noble. Enters: you may have it deal 2
/// damage to another creature you control; if you do, draw a card.
pub fn wicked_guardian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Have it deal 2 damage to another creature you control to draw a card?".into(),
            body: Box::new(Effect::ChooseOneAmong {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::Seq(vec![
                    Effect::DealDamage { to: Selector::SeparatedPile { chosen: true }, amount: Value::Const(2) },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
                other: Box::new(Effect::Noop),
            }),
        })],
        ..creature("Wicked Guardian", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Noble], 4, 2)
    }
}

/// Iron Hills Stalwart — {4}{R} 4/5 reach trample Dwarf Warrior. Enters:
/// attach target Equipment you control to up to one target creature you
/// control.
pub fn iron_hills_stalwart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Attach {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou),
                },
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
            }),
        })],
        ..creature("Iron Hills Stalwart", cost(&[generic(4), r()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 4, 5)
    }
}

/// Purple Dragon Punks — {1}{R} 2/2 Human Rogue. {T}: add {R}; spend it only
/// to cast an artifact spell or activate an ability.
pub fn purple_dragon_punks() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Red, Value::ONE)),
                    crate::mana::SpendRestriction::NoNonartifactSpells,
                ),
            },
            ..Default::default()
        }],
        ..creature("Purple Dragon Punks", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 2)
    }
}

/// Power Boost — {2}{R} enchantment. Creatures you control get +1/+0.
pub fn power_boost() -> CardDefinition {
    CardDefinition {
        name: "Power Boost",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Burlfist Oak — {2}{G}{G} 2/3 Treefolk. Whenever you draw a card, +2/+2
/// until end of turn.
pub fn burlfist_oak() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::PumpPT { what: Selector::This, power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
        }],
        ..creature("Burlfist Oak", cost(&[generic(2), g(), g()]), vec![CreatureType::Treefolk], 2, 3)
    }
}

/// Sylvan Yeti — {2}{G}{G} */4 Yeti; power = cards in your hand.
pub fn sylvan_yeti() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sylvan Yeti's power is equal to the number of cards in your hand.",
            effect: StaticEffect::SelfBasePtFromValue { power: Value::HandSizeOf(PlayerRef::You), toughness: Value::Const(4) },
        }],
        ..creature("Sylvan Yeti", cost(&[generic(2), g(), g()]), vec![CreatureType::Yeti], 0, 4)
    }
}

/// Triumph of Ferocity — {2}{G} enchantment. Your upkeep: draw a card if you
/// control the creature with the greatest power or tied for it.
pub fn triumph_of_ferocity() -> CardDefinition {
    let biggest = Predicate::ControlsGreatestPowerCreature { who: PlayerRef::You };
    CardDefinition {
        name: "Triumph of Ferocity",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(biggest.clone()),
            effect: Effect::If {
                cond: biggest,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Divining Duelist — {2}{U} 3/2 flash Merfolk Wizard. Enters, choose one: tap
/// target creature; untap target creature; or draw, then discard.
pub fn divining_duelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        ]))],
        ..creature("Divining Duelist", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 3, 2)
    }
}

/// Wake Thrasher — {2}{U} 1/1 Merfolk Soldier. Whenever a permanent you
/// control becomes untapped, +1/+1 until end of turn.
pub fn wake_thrasher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::YourControl),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
        }],
        ..creature("Wake Thrasher", cost(&[generic(2), u()]), vec![CreatureType::Merfolk, CreatureType::Soldier], 1, 1)
    }
}

/// Crashing Tide — {2}{U} sorcery, flash while you control a Merfolk. Return
/// target creature to its owner's hand; draw a card.
pub fn crashing_tide() -> CardDefinition {
    CardDefinition {
        name: "Crashing Tide",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell has flash as long as you control a Merfolk.",
            effect: StaticEffect::SelfFlashIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::ControlledBy { who: PlayerRef::You, filter: R::HasCreatureType(CreatureType::Merfolk) },
                    n: Value::ONE,
                },
            },
        }],
        effect: Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

// ── Two- to five-card-gap decks: Iroh, Lu Meng, Syr Alin, Jasmine Boreal,
// Ihsan's Shade, Guan Yu, Arasta, The Lady of the Mountain, Dori, Smaug,
// Jacques le Vert, Flatman ──────────────────────────────────────────────────

fn soldier(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    creature(name, mana, vec![CreatureType::Human, CreatureType::Soldier], p, t)
}

/// Capital Guard — {1}{R} 2/2 Human Soldier.
pub fn capital_guard() -> CardDefinition {
    soldier("Capital Guard", cost(&[generic(1), r()]), 2, 2)
}

/// Dragon Moose — {3}{R} 3/3 haste Dragon Elk.
pub fn dragon_moose() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..creature("Dragon Moose", cost(&[generic(3), r()]), vec![CreatureType::Dragon, CreatureType::Elk], 3, 3)
    }
}

/// Komodo Rhino — {3}{R} 5/2 trample Lizard Rhino.
pub fn komodo_rhino() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Komodo Rhino", cost(&[generic(3), r()]), vec![CreatureType::Lizard, CreatureType::Rhino], 5, 2)
    }
}

/// Warship Scout — {R} 2/1 Human Scout.
pub fn warship_scout() -> CardDefinition {
    creature("Warship Scout", cost(&[r()]), vec![CreatureType::Human, CreatureType::Scout], 2, 1)
}

/// Explosive Shot — {1}{R} sorcery. 4 damage to target creature.
pub fn explosive_shot() -> CardDefinition {
    CardDefinition {
        name: "Explosive Shot",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
        ..Default::default()
    }
}

/// Lu Meng, Wu General — {3}{U}{U} legendary 4/4 Human Soldier, horsemanship.
pub fn lu_meng_wu_general() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Horsemanship],
        ..soldier("Lu Meng, Wu General", cost(&[generic(3), u(), u()]), 4, 4)
    }
}

/// Wu Elite Cavalry — {3}{U} 2/3 Human Soldier, horsemanship.
pub fn wu_elite_cavalry() -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Horsemanship], ..soldier("Wu Elite Cavalry", cost(&[generic(3), u()]), 2, 3) }
}

/// Wu Light Cavalry — {1}{U} 1/2 Human Soldier, horsemanship.
pub fn wu_light_cavalry() -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Horsemanship], ..soldier("Wu Light Cavalry", cost(&[generic(1), u()]), 1, 2) }
}

fn before_attackers_on_your_turn() -> Predicate {
    use crate::game::types::TurnStep as S;
    Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::Any(vec![
            Predicate::CurrentStepIs(S::Upkeep),
            Predicate::CurrentStepIs(S::Draw),
            Predicate::CurrentStepIs(S::PreCombatMain),
            Predicate::CurrentStepIs(S::BeginCombat),
        ]),
    ])
}

/// Wu Longbowman — {2}{U} 1/1 Human Soldier Archer. {T}: 1 damage to any
/// target; only during your turn, before attackers are declared.
pub fn wu_longbowman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(before_attackers_on_your_turn()),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Wu Longbowman",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Wu Spy — {1}{U} 1/1 Human Soldier Rogue. Enters: look at the top two of
/// target player's library; put one of them into their graveyard.
pub fn wu_spy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::TargetPlayerThen {
            filter: R::Player,
            then: Box::new(Effect::LookTopKeepOneRestToGraveyard {
                count: Value::Const(2),
                who: Some(PlayerRef::Target(0)),
                exile_rest: false,
                rest_bottom_random: false,
            }),
        })],
        ..creature("Wu Spy", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Rogue], 1, 1)
    }
}

/// Ardenvale Paladin — {3}{W} 2/5 Human Knight. Adamant — three white mana
/// spent: a +1/+1 counter as it enters. Residual: the counter arrives by an
/// enters trigger, not the enters-with replacement.
pub fn ardenvale_paladin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SourceCastWithColorSpent { color: Color::White, at_least: 3 },
            then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Ardenvale Paladin", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 5)
    }
}

/// Dwarven Provisioner — {1}{W} 2/2 Dwarf Citizen. {3}{W}: creatures you
/// control get +1/+1 until end of turn.
pub fn dwarven_provisioner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dwarven Provisioner", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Citizen], 2, 2)
    }
}

/// Venerable Knight — {W} 2/1 Human Knight. Dies: a +1/+1 counter on target
/// Knight you control.
pub fn venerable_knight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::AddCounter {
            what: target_filtered(R::HasCreatureType(CreatureType::Knight).and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature("Venerable Knight", cost(&[w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 1)
    }
}

/// Thorin's Last Stand — {2}{W}{W} instant. Choose one: creatures you control
/// get +2/+1 until end of turn; or destroy target artifact or enchantment and
/// gain 2 life.
pub fn thorins_last_stand() -> CardDefinition {
    CardDefinition {
        name: "Thorin's Last Stand",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        ]),
        ..Default::default()
    }
}

/// Devilthorn Fox — {1}{W} 3/1 Fox.
pub fn devilthorn_fox() -> CardDefinition {
    creature("Devilthorn Fox", cost(&[generic(1), w()]), vec![CreatureType::Fox], 3, 1)
}

/// Raptor Companion — {1}{W} 3/1 Dinosaur.
pub fn raptor_companion() -> CardDefinition {
    creature("Raptor Companion", cost(&[generic(1), w()]), vec![CreatureType::Dinosaur], 3, 1)
}

/// Dromoka Warrior — {1}{W} 3/1 Human Warrior.
pub fn dromoka_warrior() -> CardDefinition {
    creature("Dromoka Warrior", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 1)
}

/// Grizzled Leotau — {G}{W} 1/5 Cat.
pub fn grizzled_leotau() -> CardDefinition {
    creature("Grizzled Leotau", cost(&[g(), w()]), vec![CreatureType::Cat], 1, 5)
}

fn shade_pump() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[b()]),
        effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
        ..Default::default()
    }
}

/// Dread Shade — {B}{B}{B} 3/3 Shade. {B}: +1/+1 until end of turn.
pub fn dread_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![shade_pump()],
        ..creature("Dread Shade", cost(&[b(), b(), b()]), vec![CreatureType::Shade], 3, 3)
    }
}

/// Drifting Shade — {3}{B} 1/1 flying Shade. {B}: +1/+1 until end of turn.
pub fn drifting_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![shade_pump()],
        ..creature("Drifting Shade", cost(&[generic(3), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Hoar Shade — {3}{B} 1/2 Shade. {B}: +1/+1 until end of turn.
pub fn hoar_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![shade_pump()],
        ..creature("Hoar Shade", cost(&[generic(3), b()]), vec![CreatureType::Shade], 1, 2)
    }
}

/// Liliana's Shade — {2}{B}{B} 1/1 Shade. Enters: you may search for a Swamp
/// card to your hand. {B}: +1/+1 until end of turn.
pub fn lilianas_shade() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search your library for a Swamp card?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Swamp),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        activated_abilities: vec![shade_pump()],
        ..creature("Liliana's Shade", cost(&[generic(2), b(), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Deathrender — {4} Equipment. +2/+2. Whenever the equipped creature dies,
/// you may put a creature card from your hand onto the battlefield and attach
/// this to it. Equip {2}.
pub fn deathrender() -> CardDefinition {
    CardDefinition {
        name: "Deathrender",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 2, toughness: 2, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::MayDo {
                description: "Put a creature card from your hand onto the battlefield and attach Deathrender to it?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::ClearLastMoved,
                    Effect::MoveChosen {
                        from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: R::Creature },
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::Attach { what: Selector::This, to: Selector::LastMoved },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Shu Elite Companions — {4}{W} 3/3 Human Soldier, horsemanship.
pub fn shu_elite_companions() -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Horsemanship], ..soldier("Shu Elite Companions", cost(&[generic(4), w()]), 3, 3) }
}

/// Shu Grain Caravan — {2}{W} 2/2 Human Soldier. Enters: gain 2 life.
pub fn shu_grain_caravan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })],
        ..soldier("Shu Grain Caravan", cost(&[generic(2), w()]), 2, 2)
    }
}

/// Volunteer Militia — {W} 1/2 Human Soldier.
pub fn volunteer_militia() -> CardDefinition {
    soldier("Volunteer Militia", cost(&[w()]), 1, 2)
}

fn spider(name: &'static str, mana: ManaCost, p: i32, t: i32, keywords: Vec<Keyword>) -> CardDefinition {
    CardDefinition { keywords, ..creature(name, mana, vec![CreatureType::Spider], p, t) }
}

/// Acid Web Spider — {3}{G}{G} 3/5 reach Spider. Enters: you may destroy
/// target Equipment.
pub fn acid_web_spider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Destroy { what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)) }),
        })],
        ..spider("Acid Web Spider", cost(&[generic(3), g(), g()]), 3, 5, vec![Keyword::Reach])
    }
}

/// Juvenile Gloomwidow — {G}{G} 1/3 reach, wither Spider.
pub fn juvenile_gloomwidow() -> CardDefinition {
    spider("Juvenile Gloomwidow", cost(&[g(), g()]), 1, 3, vec![Keyword::Reach, Keyword::Wither])
}

/// Kessig Recluse — {2}{G}{G} 2/3 reach, deathtouch Spider.
pub fn kessig_recluse() -> CardDefinition {
    spider("Kessig Recluse", cost(&[generic(2), g(), g()]), 2, 3, vec![Keyword::Reach, Keyword::Deathtouch])
}

/// Skysnare Spider — {4}{G}{G} 6/6 vigilance, reach Spider.
pub fn skysnare_spider() -> CardDefinition {
    spider("Skysnare Spider", cost(&[generic(4), g(), g()]), 6, 6, vec![Keyword::Vigilance, Keyword::Reach])
}

/// Attercop — {1}{G} 2/1 reach, deathtouch Spider. Landfall: +1/+1 until end
/// of turn.
pub fn attercop() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::landfall(Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..spider("Attercop", cost(&[generic(1), g()]), 2, 1, vec![Keyword::Reach, Keyword::Deathtouch])
    }
}

/// Basalt Ravager — {3}{R} 4/2 Giant Wizard. Enters: X damage to any target,
/// X = the greatest number of creatures you control sharing a creature type.
pub fn basalt_ravager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: crate::effect::shortcut::target_any(),
            amount: Value::GreatestSharedCreatureTypeCount,
        })],
        ..creature("Basalt Ravager", cost(&[generic(3), r()]), vec![CreatureType::Giant, CreatureType::Wizard], 4, 2)
    }
}

/// Blind-Spot Giant — {2}{R} 4/3 Giant Warrior; can't attack or block unless
/// you control another Giant.
pub fn blind_spot_giant() -> CardDefinition {
    let alone = Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
        R::HasCreatureType(CreatureType::Giant).and(R::ControlledByYou).and(R::OtherThanSource),
    ))));
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "This creature can't attack unless you control another Giant.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate { keyword: Keyword::CantAttack, condition: alone.clone() },
            },
            StaticAbility {
                description: "This creature can't block unless you control another Giant.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate { keyword: Keyword::CantBlock, condition: alone },
            },
        ],
        ..creature("Blind-Spot Giant", cost(&[generic(2), r()]), vec![CreatureType::Giant, CreatureType::Warrior], 4, 3)
    }
}

/// Giant Harbinger — {4}{R} 3/4 Giant Shaman. Enters: you may search for a
/// Giant card and put it on top of your library.
pub fn giant_harbinger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search your library for a Giant card to put on top?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Giant),
                to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top },
            }),
        })],
        ..creature("Giant Harbinger", cost(&[generic(4), r()]), vec![CreatureType::Giant, CreatureType::Shaman], 3, 4)
    }
}

/// Seven Dwarves — {1}{R} 2/2 Dwarf; +1/+1 per other Seven Dwarves you
/// control. A deck can have up to seven cards named Seven Dwarves.
pub fn seven_dwarves() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "This creature gets +1/+1 for each other creature named Seven Dwarves you control.",
                effect: StaticEffect::PumpSelfByControlledPermanents {
                    filter: R::Creature.and(R::HasName("Seven Dwarves".into())).and(R::OtherThanSource).and(R::ControlledByYou),
                    per_power: 1,
                    per_toughness: 1,
                },
            },
            StaticAbility {
                description: "A deck can have up to seven cards named Seven Dwarves.",
                effect: StaticEffect::DeckMayHaveCopies(Some(7)),
            },
        ],
        ..creature("Seven Dwarves", cost(&[generic(1), r()]), vec![CreatureType::Dwarf], 2, 2)
    }
}

/// Sparktongue Dragon — {3}{R}{R} 3/3 flying Dragon. Enters: you may pay
/// {2}{R}; when you do, 3 damage to any target.
pub fn sparktongue_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2}{R} to deal 3 damage to any target?".into(),
            mana_cost: cost(&[generic(2), r()]),
            body: Box::new(Effect::ReflexiveTrigger {
                body: Box::new(Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) }),
            }),
            else_: None,
        })],
        ..creature("Sparktongue Dragon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Young Red Dragon — {3}{R} 3/2 flying Dragon that can't block. Adventure:
/// Bathe in Gold {1}{R} instant — create a Treasure.
pub fn young_red_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Bathe in Gold",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        })),
        ..creature("Young Red Dragon", cost(&[generic(3), r()]), vec![CreatureType::Dragon], 3, 2)
    }
}

/// Amethyst Dragon — {4}{R}{R} 4/4 flying, haste Dragon. Adventure: Explosive
/// Crystal {4}{R} sorcery — 4 damage divided among any number of targets.
pub fn amethyst_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Explosive Crystal",
            cost: cost(&[generic(4), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamageDivided {
                total: Value::Const(4),
                filter: R::any_target(),
                max_targets: 4,
                retaliate_to_source: false,
            },
        })),
        ..creature("Amethyst Dragon", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Fang Dragon — {5}{R}{R} 6/3 flying Dragon. Adventure: Forktail Sweep {1}{R}
/// sorcery — 1 damage to each creature you don't control.
pub fn fang_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Forktail Sweep",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::ControlledByYou)))),
                amount: Value::ONE,
            },
        })),
        ..creature("Fang Dragon", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 6, 3)
    }
}

/// Abzan Beastmaster — {2}{G} 2/1 Dog Shaman. Your upkeep: draw a card if you
/// control the creature with the greatest toughness or tied for it.
pub fn abzan_beastmaster() -> CardDefinition {
    let toughest = Predicate::ControlsGreatestToughnessCreature { who: PlayerRef::You };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(toughest.clone()),
            effect: Effect::If {
                cond: toughest,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Abzan Beastmaster", cost(&[generic(2), g()]), vec![CreatureType::Dog, CreatureType::Shaman], 2, 1)
    }
}

/// Overgrown Armasaur — {3}{G}{G} 4/4 Dinosaur. Enrage — a 1/1 green Saproling.
pub fn overgrown_armasaur() -> CardDefinition {
    let saproling = TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Saproling], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(saproling) })],
        ..creature("Overgrown Armasaur", cost(&[generic(3), g(), g()]), vec![CreatureType::Dinosaur], 4, 4)
    }
}

/// Doorman — {1}{G} legendary 2/3 Mutant Hero. {T}: until end of turn, target
/// creature can't be blocked by creatures with power 2 or less and/or Walls.
pub fn doorman() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBeBlockedBy(Box::new(R::PowerAtMost(2).or(R::HasCreatureType(CreatureType::Wall)))),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Doorman", cost(&[generic(1), g()]), vec![CreatureType::Mutant, CreatureType::Hero], 2, 3)
    }
}

// ── Beckett Brass, Swarm, Horobi, Marhault, Butterbur, Aryel, Bill the
// Pony, Baldin, Gimli ───────────────────────────────────────────────────────

fn pirate() -> R {
    R::HasCreatureType(CreatureType::Pirate)
}

/// Dire Fleet Captain — {B}{R} 2/2 Orc Pirate. Attacking: +1/+1 until end of
/// turn for each other attacking Pirate.
pub fn dire_fleet_captain() -> CardDefinition {
    let others = || Value::count(Selector::EachPermanent(pirate().and(R::IsAttacking).and(R::OtherThanSource)));
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: others(),
            toughness: others(),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Dire Fleet Captain", cost(&[b(), r()]), vec![CreatureType::Orc, CreatureType::Pirate], 2, 2)
    }
}

/// Dire Fleet Neckbreaker — {2}{B}{R} 3/2 Orc Pirate. Attacking Pirates you
/// control get +2/+0.
pub fn dire_fleet_neckbreaker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Attacking Pirates you control get +2/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(pirate().and(R::IsAttacking).and(R::ControlledByYou)),
                power: 2,
                toughness: 0,
            },
        }],
        ..creature("Dire Fleet Neckbreaker", cost(&[generic(2), b(), r()]), vec![CreatureType::Orc, CreatureType::Pirate], 3, 2)
    }
}

/// Dire Fleet Poisoner — {1}{B} 2/2 flash deathtouch Human Pirate. Enters:
/// target attacking Pirate you control gets +1/+1 and deathtouch until end of
/// turn.
pub fn dire_fleet_poisoner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(pirate().and(R::IsAttacking).and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
        ]))],
        ..creature("Dire Fleet Poisoner", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Admiral's Order — {1}{U}{U} instant. Raid — {U} instead if you attacked
/// this turn. Counter target spell.
pub fn admirals_order() -> CardDefinition {
    CardDefinition {
        name: "Admiral's Order",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[u()]),
            condition: Some(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            ..Default::default()
        }),
        effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        ..Default::default()
    }
}

/// Lookout's Dispersal — {2}{U} instant, {1} less with a Pirate. Counter target
/// spell unless its controller pays {4}.
pub fn lookouts_dispersal() -> CardDefinition {
    CardDefinition {
        name: "Lookout's Dispersal",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast if you control a Pirate.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 1,
                condition: Predicate::SelectorExists(Selector::EachPermanent(pirate().and(R::ControlledByYou))),
            },
        }],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: cost(&[generic(4)]),
            exile: false,
            extra_generic: None,
            if_paid: None,
        },
        ..Default::default()
    }
}

/// Swarm, Being of Bees — {2}{B} legendary 2/2 flash, flying Insect Villain.
/// Mayhem {B}.
pub fn swarm_being_of_bees() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Mayhem(cost(&[b()]))],
        ..creature("Swarm, Being of Bees", cost(&[generic(2), b()]), vec![CreatureType::Insect, CreatureType::Villain], 2, 2)
    }
}

/// Foundry Hornet — {3}{B} 2/3 flying Insect. Enters, if you control a
/// creature with a +1/+1 counter: creatures your opponents control get -1/-1
/// until end of turn.
pub fn foundry_hornet() -> CardDefinition {
    let countered = Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::WithCounter(CounterType::PlusOnePlusOne)),
    ));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(countered.clone()),
            effect: Effect::If {
                cond: countered,
                then: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Foundry Hornet", cost(&[generic(3), b()]), vec![CreatureType::Insect], 2, 3)
    }
}

/// Great Fierce Bee — {2}{B} 2/2 flying Insect. Whenever one or more other
/// creatures die, scry 1.
pub fn great_fierce_bee() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.2c — one scry per batch.
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(trigger_is(R::OtherThanSource))
                .once_per_batch(),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature("Great Fierce Bee", cost(&[generic(2), b()]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Kraul Swarm — {4}{B} 4/1 flying Insect Warrior. {2}{B}, discard a creature
/// card: return it from your graveyard to your hand.
pub fn kraul_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature("Kraul Swarm", cost(&[generic(4), b()]), vec![CreatureType::Insect, CreatureType::Warrior], 4, 1)
    }
}

/// Orc Sureshot — {3}{B} 4/2 Orc Archer. Whenever another creature you
/// control enters, target creature an opponent controls gets -1/-1 until end
/// of turn.
pub fn orc_sureshot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(trigger_is(R::Creature)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Orc Sureshot", cost(&[generic(3), b()]), vec![CreatureType::Orc, CreatureType::Archer], 4, 2)
    }
}

/// Rathi Trapper — {1}{B} 1/2 Human Rebel Rogue. {B}, {T}: tap target creature.
pub fn rathi_trapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature("Rathi Trapper", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Rogue], 1, 2)
    }
}

/// Aphotic Wisps — {B} instant. Target creature becomes black and gains fear
/// until end of turn. Draw a card.
pub fn aphotic_wisps() -> CardDefinition {
    CardDefinition {
        name: "Aphotic Wisps",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeColor { what: target_filtered(R::Creature), colors: vec![Color::Black], duration: Duration::EndOfTurn, additive: false },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Fear, duration: Duration::EndOfTurn },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Nighthaze — {B} sorcery. Target creature gains swampwalk until end of turn.
/// Draw a card.
pub fn nighthaze() -> CardDefinition {
    CardDefinition {
        name: "Nighthaze",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Landwalk(LandType::Swamp),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Assembled Alphas — {5}{R} 5/5 Wolf. Whenever it blocks or becomes blocked
/// by a creature, it deals 3 damage to that creature and 3 to its controller.
pub fn assembled_alphas() -> CardDefinition {
    let maul = |each: Selector| Effect::ForEach {
        selector: each,
        body: Box::new(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(3) },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::Const(3),
            },
        ])),
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: maul(Selector::BlockedAttacker) },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: maul(Selector::BlockingCreatures),
            },
        ],
        ..creature("Assembled Alphas", cost(&[generic(5), r()]), vec![CreatureType::Wolf], 5, 5)
    }
}

/// Engulfing Slagwurm — {5}{G}{G} 7/7 Wurm. Whenever it blocks or becomes
/// blocked by a creature, destroy that creature; you gain life equal to its
/// toughness.
pub fn engulfing_slagwurm() -> CardDefinition {
    let engulf = |each: Selector| Effect::ForEach {
        selector: each,
        body: Box::new(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)) },
            Effect::Destroy { what: Selector::TriggerSource },
        ])),
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: engulf(Selector::BlockedAttacker) },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: engulf(Selector::BlockingCreatures),
            },
        ],
        ..creature("Engulfing Slagwurm", cost(&[generic(5), g(), g()]), vec![CreatureType::Wurm], 7, 7)
    }
}

/// Prized Unicorn — {3}{G} 2/2 Unicorn. All creatures able to block it do so.
pub fn prized_unicorn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AllMustBlock],
        ..creature("Prized Unicorn", cost(&[generic(3), g()]), vec![CreatureType::Unicorn], 2, 2)
    }
}

fn food() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Food)
}

fn make_food() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(crabomination_base::tokens::food_token()) }
}

/// Nimble Hobbit — {1}{W} 1/3 Halfling Peasant. Attacking: you may sacrifice a
/// Food or pay {2}{W}; when you do, tap target creature an opponent controls.
pub fn nimble_hobbit() -> CardDefinition {
    let tap = || Effect::ReflexiveTrigger {
        body: Box::new(Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) }),
    };
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice a Food to tap a creature an opponent controls?".into(),
            filter: food(),
            count: Value::ONE,
            then: Box::new(tap()),
            else_: Some(Box::new(Effect::MayPay {
                description: "Pay {2}{W} to tap a creature an opponent controls?".into(),
                mana_cost: cost(&[generic(2), w()]),
                body: Box::new(tap()),
                else_: None,
            })),
        })],
        ..creature("Nimble Hobbit", cost(&[generic(1), w()]), vec![CreatureType::Halfling, CreatureType::Peasant], 1, 3)
    }
}

/// Insatiable Appetite — {1}{G} instant. Target creature gets +3/+3 until end
/// of turn, or +5/+5 if you sacrifice a Food.
pub fn insatiable_appetite() -> CardDefinition {
    let pump = |what, n| Effect::PumpPT { what, power: Value::Const(n), toughness: Value::Const(n), duration: Duration::EndOfTurn };
    CardDefinition {
        name: "Insatiable Appetite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            pump(target_filtered(R::Creature), 3),
            Effect::MaySacrifice {
                description: "Sacrifice a Food for +5/+5 instead?".into(),
                filter: food(),
                count: Value::ONE,
                then: Box::new(pump(Selector::Target(0), 2)),
                else_: None,
            },
        ]),
        ..Default::default()
    }
}

/// Second Breakfast — {2}{W} instant. Up to two target creatures each get
/// +2/+1 until end of turn. Create a Food.
pub fn second_breakfast() -> CardDefinition {
    let pump = |slot| Effect::PumpPT {
        what: Selector::TargetFiltered { slot, filter: R::Creature },
        power: Value::Const(2),
        toughness: Value::ONE,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Second Breakfast",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::OptionalTargets { min: 0, body: Box::new(Effect::Seq(vec![pump(0), pump(1)])) },
            make_food(),
        ]),
        ..Default::default()
    }
}

fn knight() -> R {
    R::HasCreatureType(CreatureType::Knight)
}

/// Belle of the Brawl — {2}{B} 3/2 menace Human Knight. Attacking: other
/// Knights you control get +1/+0 until end of turn.
pub fn belle_of_the_brawl() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(knight().and(R::ControlledByYou).and(R::OtherThanSource)),
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Belle of the Brawl", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Knight], 3, 2)
    }
}

/// Blacklance Paragon — {1}{B} 3/1 flash Human Knight. Enters: target Knight
/// gains deathtouch and lifelink until end of turn.
pub fn blacklance_paragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::GrantKeywords {
            what: target_filtered(knight()),
            keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
            duration: Duration::EndOfTurn,
        })],
        ..creature("Blacklance Paragon", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Knight], 3, 1)
    }
}

/// Knight of Dusk's Shadow — {1}{B} 2/2 menace Human Knight. Your opponents
/// can't gain life. {1}{B}: +1/+1 until end of turn.
pub fn knight_of_dusks_shadow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife { target: crate::effect::PlayerStaticTarget::EachOpponent },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Knight of Dusk's Shadow", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Call the Cavalry — {3}{W} sorcery. Two 2/2 white Knights with vigilance.
pub fn call_the_cavalry() -> CardDefinition {
    let knight_token = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Call the Cavalry",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(knight_token) },
        ..Default::default()
    }
}

/// Eland Umbra — {1}{W} Aura. Enchanted creature gets +0/+4; umbra armor.
pub fn eland_umbra() -> CardDefinition {
    CardDefinition {
        name: "Eland Umbra",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::UmbraArmor],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +0/+4.",
            effect: StaticEffect::PumpPT { applies_to: enchanted(), power: 0, toughness: 4 },
        }],
        ..Default::default()
    }
}

/// Fortifying Provisions — {2}{W} enchantment. Creatures you control get
/// +0/+1. Enters: create a Food.
pub fn fortifying_provisions() -> CardDefinition {
    CardDefinition {
        name: "Fortifying Provisions",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 0,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![etb(make_food())],
        ..Default::default()
    }
}

/// Aegis of the Heavens — {1}{W} instant. Target creature gets +1/+7 until end
/// of turn.
pub fn aegis_of_the_heavens() -> CardDefinition {
    CardDefinition {
        name: "Aegis of the Heavens",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT { what: target_filtered(R::Creature), power: Value::ONE, toughness: Value::Const(7), duration: Duration::EndOfTurn },
        ..Default::default()
    }
}

/// Springjack Shepherd — {3}{W} 1/2 Kithkin Wizard. Chroma — enters: a 0/1
/// white Goat for each white mana symbol among your permanents' mana costs.
pub fn springjack_shepherd() -> CardDefinition {
    let goat = TokenDefinition {
        name: "Goat".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goat], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::DevotionTo(vec![Color::White]),
            definition: Arc::new(goat),
        })],
        ..creature("Springjack Shepherd", cost(&[generic(3), w()]), vec![CreatureType::Kithkin, CreatureType::Wizard], 1, 2)
    }
}

/// Tasseled Dromedary — {W} 0/4 Camel.
pub fn tasseled_dromedary() -> CardDefinition {
    creature("Tasseled Dromedary", cost(&[w()]), vec![CreatureType::Camel], 0, 4)
}

/// Smite the Deathless — {1}{R} instant. 3 damage to target creature; it loses
/// indestructible until end of turn; if it would die this turn, exile it.
pub fn smite_the_deathless() -> CardDefinition {
    CardDefinition {
        name: "Smite the Deathless",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LoseKeyword { what: target_filtered(R::Creature), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

