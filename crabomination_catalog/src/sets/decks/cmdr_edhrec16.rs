//! Commander: the 2026-10-12 rescan's two- and four-card EDHREC gaps of the
//! top 600 (`cmdr_edhrec12` closed the ≤3-card ones; `cmdr_edhrec13` and
//! `cmdr_edhrec14` are another session's). Tests in `tests/recent_b/cmdr_edhrec16.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{cascade, etb, on_attack, on_dies, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost, SpendRestriction};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(n: i32, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(def) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn scry(n: i32) -> Effect {
    Effect::Scry { who: PlayerRef::You, amount: Value::Const(n) }
}

fn mine() -> R {
    R::Creature.and(R::ControlledByYou)
}

fn spell_cast(filter: R) -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter })
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

/// "You may discard a card. If you do, draw a card."
fn rummage() -> Effect {
    Effect::MayDo {
        description: "Discard a card to draw a card?".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            draw(1),
        ])),
    }
}

fn copy_of(source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    }
}

// ── Kelsien, the Plague ──────────────────────────────────────────────────────

/// Gorgon Flail — {2} Equipment. Equipped creature gets +1/+1 and has
/// deathtouch. Equip {2}.
pub fn gorgon_flail() -> CardDefinition {
    equipment(
        "Gorgon Flail",
        cost(&[generic(2)]),
        cost(&[generic(2)]),
        EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Deathtouch], ..Default::default() },
    )
}

/// Rune of Mortality — {1}{B} Aura, enchant permanent. When it enters, draw a
/// card. An enchanted creature has deathtouch; on an Equipment, the creature
/// it equips does.
pub fn rune_of_mortality() -> CardDefinition {
    CardDefinition {
        name: "Rune of Mortality",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Rune], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Deathtouch],
            relays_through_equipment: true,
            ..Default::default()
        }),
        triggered_abilities: vec![etb(draw(1))],
        ..Default::default()
    }
}

// ── Averna, the Chaos Bloom ──────────────────────────────────────────────────

/// Boarding Party — {5}{R} 6/3 Human Pirate, haste, cascade.
pub fn boarding_party() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Cascade],
        triggered_abilities: vec![cascade(6)],
        ..creature("Boarding Party", cost(&[generic(5), r()]), vec![CreatureType::Human, CreatureType::Pirate], 6, 3)
    }
}

/// Forceful Denial — {3}{U}{U} instant, cascade. Counter target spell.
pub fn forceful_denial() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![cascade(5)],
        ..spell(
            "Forceful Denial",
            cost(&[generic(3), u(), u()]),
            CardType::Instant,
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
        )
    }
}

// ── Shirei, Shizo's Caretaker ────────────────────────────────────────────────

/// Cadaver Imp — {1}{B}{B} 1/1 Imp, flying. When it enters, you may return
/// target creature card from your graveyard to your hand.
pub fn cadaver_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature("Cadaver Imp", cost(&[generic(1), b(), b()]), vec![CreatureType::Imp], 1, 1)
    }
}

/// Disciple of Perdition — {1}{B} 1/3 Human Warlock. When it dies, choose one
/// (both at exactly 13 life — taken whenever offered): you draw a card and
/// lose 1 life; or exile target opponent's graveyard and that player loses 1
/// life.
pub fn disciple_of_perdition() -> CardDefinition {
    let draw_mode = || Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]);
    // The target slot comes first so `Target(0)` names the opponent.
    let exile_mode = || {
        Effect::Seq(vec![
            Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
            Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
        ])
    };
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::If {
            cond: Predicate::PlayerLifeExactly { who: PlayerRef::You, life: 13 },
            then: Box::new(Effect::Seq(vec![exile_mode(), draw_mode()])),
            else_: Box::new(Effect::ChooseMode(vec![draw_mode(), exile_mode()])),
        })],
        ..creature("Disciple of Perdition", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Warlock], 1, 3)
    }
}

/// Fume Spitter — {B} 1/1 Phyrexian Horror. Sacrifice it: put a -1/-1 counter
/// on target creature.
pub fn fume_spitter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Fume Spitter", cost(&[b()]), vec![CreatureType::Phyrexian, CreatureType::Horror], 1, 1)
    }
}

/// Tattered Mummy — {1}{B} 1/2 Zombie Jackal. When it dies, each opponent
/// loses 2 life.
pub fn tattered_mummy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        ..creature("Tattered Mummy", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Jackal], 1, 2)
    }
}

// ── Raph & Mikey, Troublemakers ──────────────────────────────────────────────

/// Raph & Mikey, Troublemakers — {5}{R/G}{R/G} 7/7 Mutant Ninja Turtle,
/// trample, haste. Whenever they attack, reveal until a creature card; put it
/// onto the battlefield tapped and attacking, the rest on the bottom in a
/// random order.
pub fn raph_mikey_troublemakers() -> CardDefinition {
    let rg = || hybrid(Color::Red, Color::Green);
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                cap: Value::Const(200),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
            Effect::JoinCombatAttacking { what: Selector::LastMoved },
        ]))],
        ..legend(
            "Raph & Mikey, Troublemakers",
            cost(&[generic(5), rg(), rg()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            7,
            7,
        )
    }
}

/// Groundchuck & Dirtbag — {4}{G}{G} 8/8 Ox Mole Mutant, trample. Whenever
/// you tap a land for mana, add {G}.
pub fn groundchuck_dirtbag() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![static_ab(
            "Whenever you tap a land for mana, add {G}.",
            StaticEffect::ExtraManaOnLandTap {
                enchanted_only: false,
                filter: R::Land.and(R::ControlledByYou),
                extra: crate::effect::ExtraManaKind::Fixed(Color::Green),
                while_monarch: false,
            },
        )],
        ..legend(
            "Groundchuck & Dirtbag",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Ox, CreatureType::Mole, CreatureType::Mutant],
            8,
            8,
        )
    }
}

/// Raphael, Tag Team Tough — {4}{R}{R} 5/6 Mutant Ninja Turtle, menace. The
/// first time each turn it deals combat damage to a player, untap all
/// attacking creatures; after this combat there is an additional one.
pub fn raphael_tag_team_tough() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)), up_to: None },
                Effect::AdditionalCombatPhase { count: Value::ONE },
            ]),
        }],
        ..legend(
            "Raphael, Tag Team Tough",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Mutant, CreatureType::Ninja, CreatureType::Turtle],
            5,
            6,
        )
    }
}


// ── Minn, Wily Illusionist ───────────────────────────────────────────────────

/// Krovikan Mist — {1}{U} */* Illusion, flying; P/T = Illusions on the
/// battlefield.
pub fn krovikan_mist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::CreaturesOfTypeOnBattlefield {
            creature_type: CreatureType::Illusion,
            also_graveyards: false,
        }),
        ..creature("Krovikan Mist", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 0, 0)
    }
}

/// Oneirophage — {3}{U} 1/2 Squid Illusion, flying. Whenever you draw a card,
/// put a +1/+1 counter on it.
pub fn oneirophage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Oneirophage", cost(&[generic(3), u()]), vec![CreatureType::Squid, CreatureType::Illusion], 1, 2)
    }
}

/// You Find the Villains' Lair — {1}{U}{U} instant. Choose one: counter
/// target spell; or draw two cards, then discard two.
pub fn you_find_the_villains_lair() -> CardDefinition {
    spell(
        "You Find the Villains' Lair",
        cost(&[generic(1), u(), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::Seq(vec![
                draw(2),
                Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            ]),
        ]),
    )
}

/// Mordenkainen — {4}{U}{U} planeswalker, loyalty 5. +2: draw two, put a card
/// from your hand on the bottom. −2: a blue Dog Illusion whose P/T is twice
/// your hand size. −10: exchange your hand and library, then shuffle; an
/// emblem with no maximum hand size.
pub fn mordenkainen() -> CardDefinition {
    let twice_hand = || Value::Times(Box::new(Value::HandSizeOf(PlayerRef::You)), Box::new(Value::Const(2)));
    let dog = TokenDefinition {
        static_abilities: vec![static_ab(
            "This token's power and toughness are each equal to twice the number of cards in your hand.",
            StaticEffect::SelfBasePtFromValue { power: twice_hand(), toughness: twice_hand() },
        )],
        ..token("Dog Illusion", 0, 0, Color::Blue, vec![CreatureType::Dog, CreatureType::Illusion])
    };
    CardDefinition {
        name: "Mordenkainen",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Mordenkainen], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    draw(2),
                    Effect::PutCardsFromHandOnBottom { who: Selector::You, count: Value::ONE },
                ]),
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: -2, effect: make(1, dog), ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Seq(vec![
                    Effect::ExchangeHandAndLibrary { who: PlayerRef::You },
                    Effect::CreateEmblem {
                        who: PlayerRef::You,
                        name: "Mordenkainen".into(),
                        triggered: vec![],
                        statics: vec![static_ab("You have no maximum hand size.", StaticEffect::NoMaximumHandSize)],
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Aeve, Progenitor Ooze ────────────────────────────────────────────────────

/// Biowaste Blob — {2}{G}{G} 0/0 Ooze. Oozes you control get +1/+1. At the
/// beginning of your upkeep, if you control a commander, create a token copy
/// of it.
pub fn biowaste_blob() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Oozes you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(mine().and(R::HasCreatureType(CreatureType::Ooze))),
                power: 1,
                toughness: 1,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::YouControlACommander),
            effect: copy_of(Selector::This),
        }],
        ..creature("Biowaste Blob", cost(&[generic(2), g(), g()]), vec![CreatureType::Ooze], 0, 0)
    }
}

/// Consuming Blob — {3}{G}{G} */*+1 Ooze (card types in your graveyard). At
/// the beginning of your end step, create a green Ooze token with the same
/// characteristic-defining P/T.
pub fn consuming_blob() -> CardDefinition {
    let types = || Value::CardTypesInGraveyard(PlayerRef::You);
    let ooze = TokenDefinition {
        static_abilities: vec![static_ab(
            "This token's power is equal to the number of card types among cards in your graveyard and its toughness is equal to that number plus 1.",
            StaticEffect::SelfBasePtFromValue {
                power: types(),
                toughness: Value::Sum(vec![types(), Value::ONE]),
            },
        )],
        ..token("Ooze", 0, 1, Color::Green, vec![CreatureType::Ooze])
    };
    CardDefinition {
        dynamic_pt: Some(DynamicPt::CardTypesInControllerGraveyard { base_p: 0, base_t: 1 }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: make(1, ooze),
        }],
        ..creature("Consuming Blob", cost(&[generic(3), g(), g()]), vec![CreatureType::Ooze], 0, 1)
    }
}

/// Ochre Jelly — {X}{G} 0/0 Ooze, trample, enters with X +1/+1 counters.
/// Split: when it dies with two or more +1/+1 counters, a token copy of it
/// enters with half that many, rounded down (made as it dies, not at the
/// next end step — INCOMPLETE_CARDS).
pub fn ochre_jelly() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![on_dies(Effect::If {
            cond: Predicate::ValueAtLeast(counters(), Value::Const(2)),
            then: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
                no_mana_cost: false,
                enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::HalvedRoundDown(Box::new(counters())))),
                remove_keywords: vec![],
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Ochre Jelly", cost(&[x(), g()]), vec![CreatureType::Ooze], 0, 0)
    }
}

/// Oran-Rief Ooze — {2}{G} 2/2 Ooze. When it enters, put a +1/+1 counter on
/// target creature you control. Whenever it attacks, put a +1/+1 counter on
/// each attacking creature with a +1/+1 counter on it.
pub fn oran_rief_ooze() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::AddCounter { what: target_filtered(mine()), kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            on_attack(Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsAttacking).and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        ],
        ..creature("Oran-Rief Ooze", cost(&[generic(2), g()]), vec![CreatureType::Ooze], 2, 2)
    }
}

// ── Ral, Monsoon Mage ────────────────────────────────────────────────────────

/// Reiterate — {1}{R}{R} instant, buyback {3}. Copy target instant or sorcery
/// spell; you may choose new targets.
pub fn reiterate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        ..spell(
            "Reiterate",
            cost(&[generic(1), r(), r()]),
            CardType::Instant,
            Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
                count: Value::ONE,
            },
        )
    }
}



/// Rob the Archives — {1}{R} sorcery, casualty 1. Exile the top two cards;
/// you may play them this turn.
pub fn rob_the_archives() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Casualty(1)],
        ..spell(
            "Rob the Archives",
            cost(&[generic(1), r()]),
            CardType::Sorcery,
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(2),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        )
    }
}

// ── Kenessos, Priest of Thassa ───────────────────────────────────────────────

/// Kenessos, Priest of Thassa — {1}{U} 1/3 Merfolk Cleric. Scry one more. {3}{G/U}:
/// look at the top card; a Kraken, Leviathan, Octopus or Serpent creature card
/// may go onto the battlefield; a card not put there may go on the bottom.
pub fn kenessos_priest_of_thassa() -> CardDefinition {
    let sea = R::HasCreatureType(CreatureType::Kraken)
        .or(R::HasCreatureType(CreatureType::Leviathan))
        .or(R::HasCreatureType(CreatureType::Octopus))
        .or(R::HasCreatureType(CreatureType::Serpent));
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    let bottom = || Effect::MayDo {
        description: "Put the top card of your library on the bottom?".into(),
        body: Box::new(Effect::Move {
            what: top(),
            to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Bottom },
        }),
    };
    CardDefinition {
        static_abilities: vec![static_ab(
            "If you would scry a number of cards, scry that many cards plus one instead.",
            StaticEffect::ScryLooksExtra { n: 1 },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), hybrid(Color::Green, Color::Blue)]),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::Creature.and(sea) },
                then: Box::new(Effect::MayDoElse {
                    description: "Put it onto the battlefield?".into(),
                    body: Box::new(Effect::Move {
                        what: top(),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(bottom()),
                }),
                else_: Box::new(bottom()),
            },
            ..Default::default()
        }],
        ..legend(
            "Kenessos, Priest of Thassa",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Cleric],
            1,
            3,
        )
    }
}

/// In Search of Greatness — {G}{G} enchantment. At the beginning of your
/// upkeep, you may cast a permanent spell from your hand free with mana value
/// 1 + the greatest among your permanents (read as "or less" — INCOMPLETE);
/// if you don't, scry 1.
pub fn in_search_of_greatness() -> CardDefinition {
    CardDefinition {
        name: "In Search of Greatness",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayCastFromHandFreeMatching {
                filter: R::Permanent,
                max_mv: Value::Sum(vec![Value::GreatestManaValueAmongPermanents(PlayerRef::You), Value::ONE]),
                else_: Box::new(scry(1)),
            },
        }],
        ..Default::default()
    }
}

/// Deliberate — {1}{U} instant. Scry 2, then draw a card.
pub fn deliberate() -> CardDefinition {
    spell("Deliberate", cost(&[generic(1), u()]), CardType::Instant, Effect::Seq(vec![scry(2), draw(1)]))
}

/// Joint Exploration — {1}{U} instant, kicker {G}. Scry 2, then draw a card;
/// kicked, you may put a land card from your hand onto the battlefield.
pub fn joint_exploration() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[g()]))],
        ..spell(
            "Joint Exploration",
            cost(&[generic(1), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                scry(2),
                draw(1),
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Land,
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

// ── Aphelia, Viper Whisperer ─────────────────────────────────────────────────

fn snake_token() -> TokenDefinition {
    TokenDefinition {
        keywords: vec![Keyword::Deathtouch],
        ..token("Snake", 1, 1, Color::Black, vec![CreatureType::Snake])
    }
}

/// Aphelia, Viper Whisperer — {1}{B} 1/3 Gorgon Assassin, deathtouch.
/// Attacking, you may pay {1}{B/G} for a 1/1 deathtouch Snake. {4}{B}: until
/// end of turn, a Gorgon or Snake of yours dealing combat damage to a player
/// makes that player lose half their life, rounded up — once per batch
/// (CR 603.2c).
pub fn aphelia_viper_whisperer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {1}{B/G} for a 1/1 deathtouch Snake?".into(),
            mana_cost: cost(&[generic(1), hybrid(Color::Black, Color::Green)]),
            body: Box::new(make(1, snake_token())),
            else_: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            effect: Effect::MatchingCreaturesDealingCombatDamageBatchThisTurn {
                filter: R::HasCreatureType(CreatureType::Gorgon).or(R::HasCreatureType(CreatureType::Snake)),
                body: Box::new(Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    rounded_up: true,
                }),
            },
            ..Default::default()
        }],
        ..legend(
            "Aphelia, Viper Whisperer",
            cost(&[generic(1), b()]),
            vec![CreatureType::Gorgon, CreatureType::Assassin],
            1,
            3,
        )
    }
}

/// Hooded Blightfang — {2}{B} 1/4 Snake, deathtouch. Whenever a deathtouch
/// creature you control attacks, each opponent loses 1 life and you gain 1.
/// Whenever one deals damage to a planeswalker, destroy that planeswalker.
pub fn hooded_blightfang() -> CardDefinition {
    let deathtouch = || Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasKeyword(Keyword::Deathtouch) };
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(deathtouch()),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                // Combat or not (CR 120.3): the damaged object is slot 0.
                event: EventSpec::new(EventKind::DealsDamage, EventScope::YourControl).with_filter(Predicate::All(vec![
                    deathtouch(),
                    Predicate::EntityMatches { what: Selector::Target(0), filter: R::Planeswalker },
                ])),
                effect: Effect::Destroy { what: Selector::Target(0) },
            },
        ],
        ..creature("Hooded Blightfang", cost(&[generic(2), b()]), vec![CreatureType::Snake], 1, 4)
    }
}

/// Mire Boa — {1}{G} 2/1 Snake, swampwalk. {G}: regenerate it.
pub fn mire_boa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Mire Boa", cost(&[generic(1), g()]), vec![CreatureType::Snake], 2, 1)
    }
}

/// The Serpent Society — {1}{B}{G} 3/4 Human Snake Villain, deathtouch, ward —
/// get five poison counters. Whenever another deathtouch creature you control
/// dies, each opponent sacrifices a nontoken creature.
pub fn the_serpent_society() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Ward(WardCost::Poison(5))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Deathtouch).and(R::OtherThanSource),
                },
            ),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Creature.and(R::Not(Box::new(R::IsToken))),
            },
        }],
        ..legend(
            "The Serpent Society",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Snake, CreatureType::Villain],
            3,
            4,
        )
    }
}

// ── Odric, Lunarch Marshal ───────────────────────────────────────────────────

/// Skeleton Key — {1} Equipment. Equipped creature has skulk; when it deals
/// combat damage to a player, you may draw a card, then discard one. Equip {2}.
pub fn skeleton_key() -> CardDefinition {
    equipment(
        "Skeleton Key",
        cost(&[generic(1)]),
        cost(&[generic(2)]),
        EquipBonus {
            keywords: vec![Keyword::Skulk],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Draw a card, then discard a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        draw(1),
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ])),
                },
            }],
            ..Default::default()
        },
    )
}

/// Akroma, Vision of Ixidor — {5}{W}{W} 6/6 Angel, flying, first strike,
/// vigilance, trample, partner. At the beginning of each combat, each other
/// creature you control gets +1/+1 until end of turn for each listed keyword
/// it has (any protection counts once; partner is partner or "partner with").
pub fn akroma_vision_of_ixidor() -> CardDefinition {
    let listed = [
        Keyword::Flying,
        Keyword::FirstStrike,
        Keyword::DoubleStrike,
        Keyword::Deathtouch,
        Keyword::Haste,
        Keyword::Hexproof,
        Keyword::Indestructible,
        Keyword::Lifelink,
        Keyword::Menace,
        Keyword::Reach,
        Keyword::Trample,
        Keyword::Vigilance,
    ];
    let per_keyword: Vec<Effect> = listed
        .into_iter()
        .map(R::HasKeyword)
        .chain([R::HasProtection, R::HasPartner])
        .map(|filter| Effect::If {
            cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter },
            then: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })
        .collect();
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Vigilance, Keyword::Trample, Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(mine().and(R::OtherThanSource)),
                body: Box::new(Effect::Seq(per_keyword)),
            },
        }],
        ..legend("Akroma, Vision of Ixidor", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 6, 6)
    }
}

/// Angelic Overseer — {3}{W}{W} 5/3 Angel, flying; hexproof and indestructible
/// while you control a Human.
pub fn angelic_overseer() -> CardDefinition {
    let human = || Predicate::SelectorExists(Selector::EachPermanent(
        R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou),
    ));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            static_ab(
                "As long as you control a Human, this creature has hexproof.",
                StaticEffect::SelfHasKeywordIf { keyword: Keyword::Hexproof, condition: human() },
            ),
            static_ab(
                "As long as you control a Human, this creature has indestructible.",
                StaticEffect::SelfHasKeywordIf { keyword: Keyword::Indestructible, condition: human() },
            ),
        ],
        ..creature("Angelic Overseer", cost(&[generic(3), w(), w()]), vec![CreatureType::Angel], 5, 3)
    }
}

/// Oketra the True — {3}{W} 3/6 God, double strike, indestructible; can't
/// attack or block unless you control three other creatures. {3}{W}: a 1/1
/// white Warrior with vigilance.
pub fn oketra_the_true() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::DoubleStrike,
            Keyword::Indestructible,
            Keyword::CantAttackOrBlockUnlessYouControlCount {
                filter: Box::new(R::Creature),
                min: 3,
                attack_only: false,
                block_only: false,
                exclude_self: true,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: make(
                1,
                TokenDefinition {
                    keywords: vec![Keyword::Vigilance],
                    ..token("Warrior", 1, 1, Color::White, vec![CreatureType::Warrior])
                },
            ),
            ..Default::default()
        }],
        ..legend("Oketra the True", cost(&[generic(3), w()]), vec![CreatureType::God], 3, 6)
    }
}

// ── Ashling, Rekindled ───────────────────────────────────────────────────────

fn ashling_mana() -> Effect {
    Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::Restricted(
            Box::new(ManaPayload::AnyOneColor(Value::Const(2))),
            SpendRestriction::SpellManaValueAtLeast4,
        ),
    }
}

/// Ashling, Rekindled // Ashling, Rimebound — {1}{R} 1/3 Elemental Sorcerer.
/// Entering or transforming into it, you may discard a card to draw a card;
/// at the beginning of your first main phase you may pay {U} to transform.
/// Rimebound (1/3 Elemental Wizard): transforming into it and at the
/// beginning of your first main phase, add two mana of any one color for
/// spells with mana value 4 or greater; you may pay {R} to transform back.
pub fn ashling_rekindled() -> CardDefinition {
    let main_phase = || EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl);
    let flip = |pip: ManaCost| Effect::MayPay {
        description: "Transform Ashling?".into(),
        mana_cost: pip,
        body: Box::new(Effect::Transform { what: Selector::This }),
        else_: None,
    };
    let rimebound = CardDefinition {
        name: "Ashling, Rimebound",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental, CreatureType::Wizard], ..Default::default() },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource), effect: ashling_mana() },
            TriggeredAbility {
                event: main_phase(),
                effect: Effect::Seq(vec![ashling_mana(), flip(cost(&[r()]))]),
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(rummage()),
            TriggeredAbility { event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource), effect: rummage() },
            TriggeredAbility { event: main_phase(), effect: flip(cost(&[u()])) },
        ],
        back_face: Some(Box::new(rimebound)),
        ..legend(
            "Ashling, Rekindled",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            1,
            3,
        )
    }
}

/// Enraged Flamecaster — {2}{R} 3/2 Elemental Sorcerer, reach. Whenever you
/// cast a spell with mana value 4 or greater, 2 damage to each opponent.
pub fn enraged_flamecaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: spell_cast(R::ManaValueAtLeast(4)),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
        }],
        ..creature(
            "Enraged Flamecaster",
            cost(&[generic(2), r()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            3,
            2,
        )
    }
}

/// Tanufel Rimespeaker — {3}{U} 2/4 Elemental Wizard. Whenever you cast a
/// spell with mana value 4 or greater, draw a card.
pub fn tanufel_rimespeaker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility { event: spell_cast(R::ManaValueAtLeast(4)), effect: draw(1) }],
        ..creature(
            "Tanufel Rimespeaker",
            cost(&[generic(3), u()]),
            vec![CreatureType::Elemental, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Ashling's Command — {3}{U}{R} Kindred instant — Elemental. Choose two: a
/// token copy of target Elemental you control; target player draws two; 2
/// damage to each creature target player controls; target player creates two
/// Treasures.
pub fn ashlings_command() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..spell(
            "Ashling's Command",
            cost(&[generic(3), u(), r()]),
            CardType::Instant,
            Effect::ChooseModesCast {
                min: 2,
                max: 2,
                allow_repeats: false,
                modes: vec![
                    copy_of(target_filtered(mine().and(R::HasCreatureType(CreatureType::Elemental)))),
                    Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
                    Effect::DealDamage {
                        to: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        amount: Value::Const(2),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::Target(0),
                        count: Value::Const(2),
                        definition: Arc::new(crabomination_base::tokens::treasure_token()),
                    },
                ],
            },
        )
    }
}
