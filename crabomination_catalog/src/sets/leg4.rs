//! Legends (LEG) wave 5 — the Elder Dragon cycle, the Glyph cycle, the
//! remaining legends and the set's utility artifacts, spells and Auras.
//! Tests in `classic_sets/leg4`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{gain_life, target, target_any, target_filtered, you},
};
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

fn legend(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, c, types, p, t) }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
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

fn world(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::World], ..enchantment(name, c) }
}

fn aura(name: &'static str, c: ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        ..enchantment(name, c)
    }
}

fn host() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn upkeep(scope: EventScope) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), scope)
}

// ── The Elder Dragon cycle ─────────────────────────────────────────────────

/// The five Elder Dragons share a flying body and an upkeep tithe.
fn elder_dragon(
    name: &'static str,
    c: ManaCost,
    upkeep_cost: ManaCost,
    keywords: Vec<Keyword>,
    extra: Vec<ActivatedAbility>,
) -> CardDefinition {
    CardDefinition {
        keywords,
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: upkeep_cost },
        }],
        activated_abilities: extra,
        ..legend(name, c, vec![CreatureType::Elder, CreatureType::Dragon], 7, 7)
    }
}

/// A "{colour}: this gets +P/+T until end of turn" firebreathing line.
fn self_pump(mana_cost: ManaCost, power: i32, toughness: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Arcades Sabboth — the defensive Elder Dragon.
pub fn arcades_sabboth() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each untapped creature you control gets +0/+2 as long as it's not \
                          attacking.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::ControlledByYou)
                    .and(R::Untapped)
                    .and(R::Not(Box::new(R::IsAttacking))),
                power: 0,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                all_players: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..elder_dragon(
            "Arcades Sabboth",
            cost(&[generic(2), g(), g(), w(), w(), u(), u()]),
            cost(&[g(), w(), u()]),
            vec![Keyword::Flying],
            vec![self_pump(cost(&[w()]), 0, 1)],
        )
    }
}

/// Chromium — the rampaging Elder Dragon.
pub fn chromium() -> CardDefinition {
    elder_dragon(
        "Chromium",
        cost(&[generic(2), w(), w(), u(), u(), b(), b()]),
        cost(&[w(), u(), b()]),
        vec![Keyword::Flying, Keyword::Rampage(2)],
        vec![],
    )
}

/// Nicol Bolas — the Elder Dragon that empties a hand.
pub fn nicol_bolas() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: upkeep(EventScope::YourControl),
                effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[u(), b(), r()]) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::HandSizeOf(PlayerRef::EachOpponent),
                    random: false,
                },
            },
        ],
        ..elder_dragon(
            "Nicol Bolas",
            cost(&[generic(2), u(), u(), b(), b(), r(), r()]),
            cost(&[u(), b(), r()]),
            vec![Keyword::Flying],
            vec![],
        )
    }
}

/// Palladia-Mors — the trampling Elder Dragon.
pub fn palladia_mors() -> CardDefinition {
    elder_dragon(
        "Palladia-Mors",
        cost(&[generic(2), r(), r(), g(), g(), w(), w()]),
        cost(&[r(), g(), w()]),
        vec![Keyword::Flying, Keyword::Trample],
        vec![],
    )
}

/// Vaevictis Asmadi — the firebreathing Elder Dragon.
pub fn vaevictis_asmadi() -> CardDefinition {
    elder_dragon(
        "Vaevictis Asmadi",
        cost(&[generic(2), b(), b(), r(), r(), g(), g()]),
        cost(&[b(), r(), g()]),
        vec![Keyword::Flying],
        vec![self_pump(cost(&[b()]), 1, 0), self_pump(cost(&[r()]), 1, 0), self_pump(cost(&[g()]), 1, 0)],
    )
}

// ── The Glyph cycle ────────────────────────────────────────────────────────

/// Everything the targeted Wall blocked this turn.
fn blocked_by_target() -> Selector {
    Selector::CreaturesBlockedByThisTurn(Box::new(target()))
}

fn glyph(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { effect, ..instant(name, c, Effect::Noop) }
}

/// Glyph of Doom — everything the Wall stopped dies at end of combat.
pub fn glyph_of_doom() -> CardDefinition {
    CardDefinition {
        effect: Effect::AtEndOfCombat {
            body: Box::new(Effect::Destroy { what: blocked_by_target() }),
        },
        ..instant("Glyph of Doom", cost(&[b()]), Effect::Noop)
    }
}

/// Glyph of Reincarnation — cast after combat; the Wall's victims die and
/// their controllers reanimate. (The printed "creature card from the graveyard
/// of the player who controlled that creature" is modelled as each victim's
/// own controller reanimating.)
pub fn glyph_of_reincarnation() -> CardDefinition {
    glyph(
        "Glyph of Reincarnation",
        cost(&[g()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen { what: blocked_by_target() },
            Effect::EachPlayerReturnsAMatchingPermanent { filter: R::Creature },
        ]),
    )
}

/// Glyph of Destruction — a Wall becomes a wall of blades, then dies.
pub fn glyph_of_destruction() -> CardDefinition {
    glyph(
        "Glyph of Destruction",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsBlocking)),
                power: Value::Const(10),
                toughness: Value::ZERO,
                duration: Duration::EndOfCombat,
            },
            Effect::PreventAllDamageThisTurn { target: target(), redirect_to: None },
            Effect::AtNextEndStep {
                body: Box::new(Effect::Destroy { what: target() }),
            },
        ]),
    )
}

/// Glyph of Life — the Wall's beating turns into life.
pub fn glyph_of_life() -> CardDefinition {
    glyph(
        "Glyph of Life",
        cost(&[w()]),
        Effect::GrantTriggeredAbility {
            what: target_filtered(R::Creature),
            trigger: Box::new(TriggeredAbility {
                event: EventSpec::new(EventKind::DealtCombatDamage, EventScope::SelfSource),
                effect: Effect::GainLife { who: you(), amount: Value::TriggerEventAmount },
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Glyph of Delusion — the Wall's victim goes to sleep.
pub fn glyph_of_delusion() -> CardDefinition {
    glyph(
        "Glyph of Delusion",
        cost(&[u()]),
        Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::Glyph,
                amount: Value::PowerOf(Box::new(target())),
            },
            Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::DoesntUntapWhileCounter(CounterType::Glyph),
                duration: Duration::Permanent,
            },
            Effect::GrantTriggeredAbility {
                what: target(),
                trigger: Box::new(TriggeredAbility {
                    event: upkeep(EventScope::YourControl),
                    effect: Effect::RemoveCounter {
                        what: Selector::This,
                        kind: CounterType::Glyph,
                        amount: Value::ONE,
                    },
                }),
                duration: Duration::Permanent,
            },
        ]),
    )
}

// ── Legends ────────────────────────────────────────────────────────────────

/// Bartel Runeaxe — a vigilant giant no Aura can touch.
pub fn bartel_runeaxe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::CantBeTargetedByAuras],
        ..legend(
            "Bartel Runeaxe",
            cost(&[generic(3), b(), r(), g()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            6,
            5,
        )
    }
}

/// Tetsuo Umezawa — Aura-proof, and a repeatable assassination.
pub fn tetsuo_umezawa() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeTargetedByAuras],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[u(), b(), b(), r()]),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::Tapped.or(R::IsBlocking))),
            },
            ..Default::default()
        }],
        ..legend(
            "Tetsuo Umezawa",
            cost(&[u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Archer],
            3,
            3,
        )
    }
}

/// Livonya Silone — first strike plus legendary landwalk.
pub fn livonya_silone() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::LegendaryLandwalk],
        ..legend(
            "Livonya Silone",
            cost(&[generic(2), r(), r(), g(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Rubinia Soulsinger — tap her down to keep a creature.
pub fn rubinia_soulsinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..legend(
            "Rubinia Soulsinger",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Faerie],
            2,
            3,
        )
    }
}

/// Willow Satyr — Rubinia for legends only.
pub fn willow_satyr() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GainControlWhileSourceTapped {
                what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
            },
            ..Default::default()
        }],
        ..creature("Willow Satyr", cost(&[generic(2), g(), g()]), vec![CreatureType::Satyr], 1, 1)
    }
}

/// Hell's Caretaker — upkeep reanimation, one body for another.
pub fn hells_caretaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            condition: Some(Predicate::CurrentStepIs(crate::game::types::TurnStep::Upkeep)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Hell's Caretaker", cost(&[generic(3), b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Ichneumon Druid — the second instant each turn hurts.
pub fn ichneumon_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCardType(CardType::Instant),
                    },
                    Predicate::Not(Box::new(Predicate::FirstNoncreatureSpellThisTurn)),
                ])),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(4),
            },
        }],
        ..creature(
            "Ichneumon Druid",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Cosmic Horror — a 7/7 first striker that bites back when unpaid.
pub fn cosmic_horror() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {3}{B}{B}{B} to keep Cosmic Horror?".into(),
                mana_cost: cost(&[generic(3), b(), b(), b()]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::Seq(vec![
                    Effect::Destroy { what: Selector::This },
                    Effect::DealDamage { to: you(), amount: Value::Const(7) },
                ]))),
            },
        }],
        ..creature(
            "Cosmic Horror",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Horror],
            7,
            7,
        )
    }
}

/// Mold Demon — pay two Swamps or it never sticks.
pub fn mold_demon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SacrificeSourceUnlessSacrifice {
                filter: R::Land.and(R::HasLandType(LandType::Swamp)),
            },
        }],
        ..creature(
            "Mold Demon",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Fungus, CreatureType::Demon],
            6,
            6,
        )
    }
}

/// Rabid Wombat — every Aura on it is +2/+2.
pub fn rabid_wombat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+2 for each Aura attached to it.",
            effect: StaticEffect::PumpTeamPerAttachmentOnSource {
                applies_to: R::IsSource,
                attachment_filter: R::Enchantment,
                per_power: 2,
                per_toughness: 2,
            },
        }],
        ..creature("Rabid Wombat", cost(&[generic(2), g(), g()]), vec![CreatureType::Wombat], 0, 1)
    }
}

/// Time Elemental — bounces anything unenchanted, and dies for swinging.
pub fn time_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Seq(vec![
                    Effect::SacrificePermanent { what: Selector::This },
                    Effect::DealDamage { to: you(), amount: Value::Const(5) },
                ])),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::Not(Box::new(R::IsEnchanted)))),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature("Time Elemental", cost(&[generic(2), u()]), vec![CreatureType::Elemental], 0, 2)
    }
}

/// Evil Eye of Orms-by-Gore — everything else stays home.
pub fn evil_eye_of_orms_by_gore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(
            CreatureType::Wall,
        )))],
        static_abilities: vec![StaticAbility {
            description: "Non-Eye creatures you control can't attack.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Eye)))),
                ),
                keyword: Keyword::CantAttack,
            },
        }],
        ..creature("Evil Eye of Orms-by-Gore", cost(&[generic(4), b()]), vec![CreatureType::Eye], 3, 6)
    }
}

/// Akron Legionnaire — an 8/4 that benches your non-artifact army.
pub fn akron_legionnaire() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Except for creatures named Akron Legionnaire and artifact creatures, \
                          creatures you control can't attack.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::Not(Box::new(R::HasName("Akron Legionnaire".into()))))
                        .and(R::Not(Box::new(R::Artifact))),
                ),
                keyword: Keyword::CantAttack,
            },
        }],
        ..creature(
            "Akron Legionnaire",
            cost(&[generic(6), w(), w()]),
            vec![CreatureType::Giant, CreatureType::Soldier],
            8,
            4,
        )
    }
}

/// Shimian Night Stalker — soaks one attacker's damage onto itself.
pub fn shimian_night_stalker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b()]),
            effect: Effect::PreventAllDamageThisTurn {
                target: you(),
                redirect_to: Some(Selector::This),
            },
            ..Default::default()
        }],
        ..creature(
            "Shimian Night Stalker",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Nightstalker],
            4,
            4,
        )
    }
}

// ── Enchantments ───────────────────────────────────────────────────────────

/// Spiritual Sanctuary — a Plains is worth a life every upkeep.
pub fn spiritual_sanctuary() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::AnyPlayer).with_filter(Predicate::SelectorExists(
                Selector::ControlledBy {
                    who: PlayerRef::ActivePlayer,
                    filter: R::Land.and(R::HasLandType(LandType::Plains)),
                },
            )),
            effect: Effect::GainLife {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::ONE,
            },
        }],
        ..enchantment("Spiritual Sanctuary", cost(&[generic(2), w(), w()]))
    }
}

/// Lifeblood — their Mountains pay you rent.
pub fn lifeblood() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land.and(R::HasLandType(LandType::Mountain)),
                },
            ),
            effect: gain_life(1),
        }],
        ..enchantment("Lifeblood", cost(&[generic(2), w(), w()]))
    }
}

/// Storm World — an empty hand burns.
pub fn storm_world() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::Const(4)),
                    Box::new(Value::HandSizeOf(PlayerRef::ActivePlayer)),
                ))),
            },
        }],
        ..world("Storm World", cost(&[r()]))
    }
}

/// The Abyss — a nonartifact creature dies every upkeep.
pub fn the_abyss() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::AnyPlayer),
            effect: Effect::DestroyNoRegen {
                what: Selector::ControlledBy {
                    who: PlayerRef::ActivePlayer,
                    filter: R::Creature.and(R::Not(Box::new(R::Artifact))),
                },
            },
        }],
        ..world("The Abyss", cost(&[generic(3), b()]))
    }
}

/// Invoke Prejudice — off-colour creature spells pay their cost twice.
pub fn invoke_prejudice() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::Not(Box::new(Predicate::TargetSharesColorWithControlled {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                })),
            ),
            effect: Effect::CounterUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: ManaCost::default(),
                exile: false,
                extra_generic: Some(Value::ManaValueOf(Box::new(Selector::TriggerSource))),
            },
        }],
        ..enchantment("Invoke Prejudice", cost(&[u(), u(), u(), u()]))
    }
}

/// Angelic Voices — an anthem for a mono-white board.
pub fn angelic_voices() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1 as long as you control no nonartifact, \
                          nonwhite creatures.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::ControlledByYou),
                power: 1,
                toughness: 1,
                keywords: vec![],
                all_players: false,
                condition: Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::Not(Box::new(R::Artifact)))
                            .and(R::Not(Box::new(R::HasColor(Color::White)))),
                    ),
                ))),
            },
        }],
        ..enchantment("Angelic Voices", cost(&[generic(2), w(), w()]))
    }
}

/// Horror of Horrors — Swamps regenerate your black creatures.
pub fn horror_of_horrors() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land.and(R::HasLandType(LandType::Swamp)), 1)),
            effect: Effect::Regenerate {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black))),
            },
            ..Default::default()
        }],
        ..enchantment("Horror of Horrors", cost(&[generic(3), b(), b()]))
    }
}

/// Presence of the Master's red cousin: Land's Edge turns land cards into
/// burn, for anybody.
pub fn lands_edge() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            any_player: true,
            discard_cost: Some((R::Land, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..world("Land's Edge", cost(&[generic(1), r(), r()]))
    }
}

// ── Auras ──────────────────────────────────────────────────────────────────

/// Venarian Gold — X sleep counters keep the host down.
pub fn venarian_gold() -> CardDefinition {
    CardDefinition {
        cost: cost(&[crate::mana::x(), u(), u()]),
        effect: Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
            Effect::Tap { what: host() },
            Effect::AddCounter {
                what: host(),
                kind: CounterType::Sleep,
                amount: Value::XFromCost,
            },
        ]),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step if \
                          it has a sleep counter on it.",
            effect: StaticEffect::GrantKeyword {
                applies_to: host(),
                keyword: Keyword::DoesntUntapWhileCounter(CounterType::Sleep),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::EnchantedBySource),
            effect: Effect::RemoveCounter {
                what: host(),
                kind: CounterType::Sleep,
                amount: Value::ONE,
            },
        }],
        ..aura("Venarian Gold", cost(&[crate::mana::x(), u(), u()]), R::Creature)
    }
}

/// Infinite Authority — the host eats small creatures and grows.
pub fn infinite_authority() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Seq(vec![
                        Effect::Destroy {
                            what: Selector::EachPermanent(
                                R::Creature.and(R::InCombatWithSource).and(R::ToughnessAtMost(3)),
                            ),
                        },
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                    ])),
                },
            }],
            ..Default::default()
        }),
        ..aura("Infinite Authority", cost(&[w(), w(), w()]), R::Creature)
    }
}

/// Spectral Cloak's black cousin: Imprison's simplified half — the host can't
/// attack or block. (The printed pay-{1}-or-destroy-the-Aura clauses have no
/// engine analog.)
pub fn imprison() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature can't attack.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::CantAttack,
                },
            },
            StaticAbility {
                description: "Enchanted creature can't block.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: host(),
                    keyword: Keyword::CantBlock,
                },
            },
        ],
        ..aura("Imprison", cost(&[b()]), R::Creature)
    }
}

/// Takklemaggot — a wasting disease on the host.
pub fn takklemaggot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::EnchantedBySource),
            effect: Effect::AddCounter {
                what: host(),
                kind: CounterType::MinusZeroMinusOne,
                amount: Value::ONE,
            },
        }],
        ..aura("Takklemaggot", cost(&[generic(2), b(), b()]), R::Creature)
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────

/// Al-abara's Carpet — a repeatable ground fog for you alone.
pub fn al_abaras_carpet() -> CardDefinition {
    artifact(
        "Al-abara's Carpet",
        cost(&[generic(5)]),
        vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(5)]),
            effect: Effect::PreventAllCombatDamageByMatchingThisTurn {
                filter: R::Creature
                    .and(R::IsAttacking)
                    .and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            },
            ..Default::default()
        }],
    )
}

/// Kry Shield — a creature stops swinging and starts soaking.
pub fn kry_shield() -> CardDefinition {
    artifact(
        "Kry Shield",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::PreventAllDamageByTargetThisTurn {
                    target: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
                Effect::PumpPT {
                    what: target(),
                    power: Value::ZERO,
                    toughness: Value::ManaValueOf(Box::new(target())),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Life Matrix — hand out a reusable regeneration shield.
pub fn life_matrix() -> CardDefinition {
    artifact(
        "Life Matrix",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            condition: Some(Predicate::CurrentStepIs(crate::game::types::TurnStep::Upkeep)),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::Matrix,
                    amount: Value::ONE,
                },
                Effect::GrantActivatedAbilityToMatching {
                    filter: R::WithCounter(CounterType::Matrix),
                    ability: Box::new(ActivatedAbility {
                        remove_counter_cost: Some((CounterType::Matrix, 1)),
                        effect: Effect::Regenerate { what: Selector::This },
                        ..Default::default()
                    }),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Forethought Amulet — caps burn at two, for rent.
pub fn forethought_amulet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(3)]) },
        }],
        static_abilities: vec![StaticAbility {
            description: "If an instant or sorcery source would deal 3 or more damage to you, it \
                          deals 2 damage to you instead.",
            effect: StaticEffect::CapLargeDamage { at_least: 3, capped: 2 },
        }],
        ..artifact("Forethought Amulet", cost(&[generic(5)]), vec![])
    }
}

/// North Star — your spells ignore their colours for the turn. (The printed
/// "for one spell this turn" narrowing isn't modelled; the permission runs to
/// cleanup.)
pub fn north_star() -> CardDefinition {
    artifact(
        "North Star",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4)]),
            effect: Effect::MaySpendManaAsAnyColorThisTurn { who: PlayerRef::You },
            ..Default::default()
        }],
    )
}

/// Triassic Egg — bank hatchling counters, then crack it for a body.
pub fn triassic_egg() -> CardDefinition {
    artifact(
        "Triassic Egg",
        cost(&[generic(4)]),
        vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Hatchling,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                condition: Some(Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Hatchling,
                    },
                    Value::Const(2),
                )),
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
    )
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Hellfire — a black sweeper that bills you for its own kills.
pub fn hellfire() -> CardDefinition {
    sorcery(
        "Hellfire",
        cost(&[generic(2), b(), b(), b()]),
        Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
            },
            Effect::DealDamage {
                to: you(),
                amount: Value::Sum(vec![
                    Value::Const(3),
                    Value::CreaturesDiedThisResolution,
                ]),
            },
        ]),
    )
}

/// Typhoon — their Islands become damage.
pub fn typhoon() -> CardDefinition {
    sorcery(
        "Typhoon",
        cost(&[generic(2), g()]),
        Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::CountOf(Box::new(Selector::ControlledBy {
                who: PlayerRef::EachOpponent,
                filter: R::Land.and(R::HasLandType(LandType::Island)),
            })),
        },
    )
}

/// Winter Blast — tap X, and the fliers take two.
pub fn winter_blast() -> CardDefinition {
    sorcery(
        "Winter Blast",
        cost(&[crate::mana::x(), g()]),
        Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 8,
            filter: R::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: target() },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: target(),
                        filter: R::HasKeyword(Keyword::Flying),
                    },
                    then: Box::new(Effect::DealDamage { to: target(), amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        },
    )
}

/// Energy Tap — a creature's mana value, in colourless.
pub fn energy_tap() -> CardDefinition {
    sorcery(
        "Energy Tap",
        cost(&[u()]),
        Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::Untapped)),
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::ManaValueOf(Box::new(target()))),
            },
        ]),
    )
}

/// Part Water — X creatures walk over Islands.
pub fn part_water() -> CardDefinition {
    sorcery(
        "Part Water",
        cost(&[crate::mana::x(), crate::mana::x(), u()]),
        Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 8,
            filter: R::Creature,
            effect: Box::new(Effect::GrantKeyword {
                what: target(),
                keyword: Keyword::Landwalk(LandType::Island),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Blood Lust — +4 power for almost all of its toughness.
pub fn blood_lust() -> CardDefinition {
    instant(
        "Blood Lust",
        cost(&[generic(1), r()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Negate(Box::new(Value::Min(
                Box::new(Value::Const(4)),
                Box::new(Value::Diff(
                    Box::new(Value::ToughnessOf(Box::new(target()))),
                    Box::new(Value::ONE),
                )),
            ))),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Visions — look at five, then decide whether to scramble them.
pub fn visions() -> CardDefinition {
    sorcery(
        "Visions",
        cost(&[w()]),
        Effect::Seq(vec![
            Effect::LookAtTop { who: PlayerRef::Target(0), amount: Value::Const(5) },
            Effect::MayDo {
                description: "Shuffle that library?".into(),
                body: Box::new(Effect::ShuffleLibrary { who: PlayerRef::Target(0) }),
            },
        ]),
    )
}

/// Telekinesis — a creature is out of the fight for two turns.
pub fn telekinesis() -> CardDefinition {
    instant(
        "Telekinesis",
        cost(&[u(), u()]),
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::PreventCombatDamageByTargetThisTurn { target: target() },
            Effect::AddCounter {
                what: target(),
                kind: CounterType::Stun,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Feint — the blockers tap and nobody connects.
pub fn feint() -> CardDefinition {
    CardDefinition {
        cast_only_during_combat: true,
        ..instant(
            "Feint",
            cost(&[r()]),
            Effect::Seq(vec![
                Effect::Tap { what: Selector::BlockingCreatures },
                Effect::PreventAllCombatDamageInvolving {
                    target: target_filtered(R::Creature.and(R::IsAttacking)),
                },
            ]),
        )
    }
}

/// Reincarnation — the next thing to die comes back as something better.
pub fn reincarnation() -> CardDefinition {
    instant(
        "Reincarnation",
        cost(&[generic(1), g(), g()]),
        Effect::WhenTargetDiesThisTurn {
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::OwnerOf(Box::new(target())),
                    tapped: false,
                },
            }),
            slot: 0,
            filter: Some(R::Creature),
        },
    )
}
