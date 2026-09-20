//! The EDHREC Commander-staple gap — cards `COMMANDER_BACKLOG.md`'s top-1000
//! slice named and which needed **no new primitive**, only the card.
//!
//! The rule for what lands here rather than in `sets::cmdr`: `cmdr.rs` is for
//! cards whose printed text is *about* the format (it reads the command zone,
//! a commander, or a colour identity). These are ordinary cards that happen to
//! be Commander staples.
//!
//! Tests in `tests/modern/lands_equipment_vehicles.rs` (the module that
//! already holds this branch's card-shape tables).

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus, CreatureType, EquipBonus,
    EquipScale, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::PlayerStaticTarget;
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, EventKind, EventScope, EventSpec, PlayerRef, StaticEffect};
use crate::mana::{ManaCost, cost, g, generic, r, u, w};

/// Parallel Lives — {3}{G} Enchantment. "If an effect would create one or
/// more tokens under your control, it creates twice that many of those tokens
/// instead."
///
/// Doubling Season's token half on its own, and the same primitive:
/// `StaticEffect::DoubleTokens` is already controller-scoped, which is the
/// printed "under your control".
pub fn parallel_lives() -> CardDefinition {
    CardDefinition {
        name: "Parallel Lives",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If an effect would create one or more tokens under your control, \
                          it creates twice that many of those tokens instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        ..Default::default()
    }
}

/// Avacyn, Angel of Hope — {5}{W}{W}{W} Legendary Creature — Angel 8/8.
/// "Flying, vigilance, indestructible. Other permanents you control have
/// indestructible."
///
/// ⚠ **Permanents, not creatures.** `AnthemForFilter`'s filter is matched
/// against the controller's permanents rather than their creatures, so the
/// printed noun is carried by leaving `R::Creature` out — Avacyn's lands and
/// artifacts are indestructible too, which is most of what the card does in a
/// Commander pod. `R::OtherThanSource` is the printed "Other"; Avacyn's own
/// indestructible is printed separately and is a keyword on the card.
pub fn avacyn_angel_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Avacyn, Angel of Hope",
        cost: cost(&[generic(5), w(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Indestructible],
        static_abilities: vec![StaticAbility {
            description: "Other permanents you control have indestructible.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::OtherThanSource,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Indestructible],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

// ── Equipment ──────────────────────────────────────────────────────────────
//
// Four of the top-1000 gap's five Equipment. The fifth, Commander's Plate,
// reads a colour identity and so lives in `sets::cmdr`.
//
// ⚠ Nothing here needed a new primitive, including the two with a **second,
// restricted equip cost** ("Equip legendary creature {3}. Equip {7}"):
// `CardDefinition::equip_filtered_cost` already holds a `(filter, cost)` pair
// and `equip` takes it whenever the host matches, ahead of `Keyword::Equip`.
// TODO's standing lead listed that as an engine gap; it had shipped.

/// The shared shell: an Artifact — Equipment with one plain equip cost.
fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Blackblade Reforged — {2} Legendary Artifact — Equipment. "Equipped
/// creature gets +1/+1 for each land you control. Equip legendary creature
/// {3}. Equip {7}." (EDHREC 350.)
///
/// `EquipScale` counts permanents matching its filter that the **source's**
/// controller controls, which is the printed "you control" — the Equipment's
/// controller, not the host's.
pub fn blackblade_reforged() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        equip_filtered_cost: Some((R::HasSupertype(Supertype::Legendary), cost(&[generic(3)]))),
        ..equipment(
            "Blackblade Reforged",
            cost(&[generic(2)]),
            cost(&[generic(7)]),
            EquipBonus {
                scale: Some(EquipScale {
                    filter: R::Land,
                    per_power: 1,
                    per_toughness: 1,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}

/// Champion's Helm — {3} Artifact — Equipment. "Equipped creature gets +2/+2.
/// As long as equipped creature is legendary, it has hexproof. Equip {1}."
/// (EDHREC 745.)
///
/// The +2/+2 is unconditional and only the hexproof is gated, so the pump goes
/// on the bonus and the keyword on a `ConditionalEquipBonus` — a host that
/// stops being legendary loses the hexproof and keeps the stats.
pub fn champions_helm() -> CardDefinition {
    equipment(
        "Champion's Helm",
        cost(&[generic(3)]),
        cost(&[generic(1)]),
        EquipBonus {
            power: 2,
            toughness: 2,
            conditional: vec![ConditionalEquipBonus {
                host_filter: R::HasSupertype(Supertype::Legendary),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Hexproof],
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Mithril Coat — {3} Legendary Artifact — Equipment. "Flash. Indestructible.
/// When Mithril Coat enters, attach it to target legendary creature you
/// control. Equipped creature has indestructible. Equip {3}." (EDHREC 238.)
///
/// Both indestructibles are printed and they are different clauses: the
/// keyword protects the Coat itself, the `equipped_bonus` protects the host.
/// The ETB attach is **targeted**, so with no legal legendary creature the
/// trigger is removed from the stack (CR 603.3d) and the Coat sits there — the
/// flash-in-response line the card is played for still leaves it on the board.
pub fn mithril_coat() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        // Explicit, because `equipment`'s shell carries only the equip
        // keyword and both of these are printed on the Coat itself.
        keywords: vec![Keyword::Flash, Keyword::Indestructible, Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AttachSourceTo {
                host: target_filtered(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                ),
            },
        }],
        ..equipment(
            "Mithril Coat",
            cost(&[generic(3)]),
            cost(&[generic(3)]),
            EquipBonus { keywords: vec![Keyword::Indestructible], ..Default::default() },
        )
    }
}

/// The Reaver Cleaver — {2}{R} Legendary Artifact — Equipment. "Equipped
/// creature gets +1/+1 and has trample and 'Whenever this creature deals
/// combat damage to a player or planeswalker, create that many Treasure
/// tokens.' Equip {3}." (EDHREC 537.)
///
/// "a player **or planeswalker**" is two event kinds, so it is two granted
/// abilities; one creature only ever attacks one of the two in a combat, so
/// they cannot both fire off the same damage. "That many" is the damage
/// dealt — `Value::TriggerEventAmount`. `triggers_on_equipment` stays false:
/// CR 702.6e makes the granted ability the *creature's*, so "you" is the
/// creature's controller.
pub fn the_reaver_cleaver() -> CardDefinition {
    let treasures = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::TriggerEventAmount,
        definition: std::sync::Arc::new(crate::game::effects::treasure_token()),
    };
    let on = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: treasures(),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..equipment(
            "The Reaver Cleaver",
            cost(&[generic(2), r()]),
            cost(&[generic(3)]),
            EquipBonus {
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Trample],
                triggered_abilities: vec![
                    on(EventKind::DealsCombatDamageToPlayer),
                    on(EventKind::DealsCombatDamageToPlaneswalker),
                ],
                ..Default::default()
            },
        )
    }
}

// ── "Whenever an opponent …" watchers ──────────────────────────────────────
//
// Commander staples whose text is keyed on what an *opponent* does. Each fires
// once per opponent that does the thing, so at four seats they are three times
// the card they are in a duel — which is the whole reason they are staples.

/// Archivist of Oghma — {1}{W} Creature — Halfling Cleric 2/2. "Flash.
/// Whenever an opponent searches their library, you gain 1 life and draw a
/// card." (EDHREC 690.)
///
/// `EventScope::OpponentControl` on `PlayerSearchedLibrary` is per searching
/// player, so three opponents fetching on the same turn is three triggers.
pub fn archivist_of_oghma() -> CardDefinition {
    CardDefinition {
        name: "Archivist of Oghma",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Halfling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerSearchedLibrary, EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Mangara, the Diplomat — {3}{W} Legendary Creature — Human Cleric 2/4.
/// "Lifelink. Whenever an opponent attacks with creatures, if two or more of
/// those creatures are attacking you and/or planeswalkers you control, draw a
/// card. Whenever an opponent casts their second spell each turn, draw a
/// card." (EDHREC 611.)
///
/// ⚠ **The attack gate is inside the effect, not in the `EventSpec` filter,
/// and that is not a style choice.** A defender-side (`ControllerAttackedBy
/// Opponent`) trigger's filter is evaluated per attacker while the declaration
/// is still being committed to `GameState.attacking`, so a count read there
/// sees a partial batch; an `Effect::If` resolves off the stack after the
/// whole step. `.once_per_batch()` is what makes it one card a declaration
/// rather than one a creature (CR 603.2c).
///
/// The count is `include_planeswalkers: true` because the card says "you
/// and/or planeswalkers you control" in as many words.
pub fn mangara_the_diplomat() -> CardDefinition {
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        name: "Mangara, the Diplomat",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::Attacks,
                    EventScope::ControllerAttackedByOpponent,
                )
                .once_per_batch(),
                effect: Effect::If {
                    cond: Predicate::AttackedDefenderWithCountAtLeast {
                        who: PlayerRef::Triggerer,
                        defender: PlayerRef::You,
                        at_least: 2,
                        include_planeswalkers: true,
                    },
                    then: Box::new(draw()),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::SpellsCastThisTurnEquals {
                        who: PlayerRef::Triggerer,
                        count: Value::Const(2),
                    }),
                effect: draw(),
            },
        ],
        ..Default::default()
    }
}

/// Trouble in Pairs — {2}{W}{W} Enchantment. "If an opponent would begin an
/// extra turn, that player skips that turn instead. Whenever an opponent
/// attacks you with two or more creatures, draws their second card each turn,
/// or casts their second spell each turn, you draw a card." (EDHREC 625.)
///
/// One printed sentence, three event kinds, three `TriggeredAbility`s — there
/// is no "or" in an `EventSpec`, and the three are mutually exclusive in
/// practice anyway (an attack declaration, a draw and a cast are three
/// different events), so a single turn can legitimately produce three cards.
///
/// ⚠ **"attacks **you**" is `include_planeswalkers: false`**, unlike
/// Mangara's "you and/or planeswalkers you control": a creature attacking your
/// planeswalker is attacking the planeswalker (CR 506.2). The two cards are
/// the reason that flag is a flag. The gate is inside the effect for the same
/// reason as Mangara's — a defender-side filter runs mid-declaration.
pub fn trouble_in_pairs() -> CardDefinition {
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        name: "Trouble in Pairs",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would begin an extra turn, that player skips \
                          that turn instead.",
            effect: StaticEffect::OpponentsSkipExtraTurns,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::Attacks,
                    EventScope::ControllerAttackedByOpponent,
                )
                .once_per_batch(),
                effect: Effect::If {
                    cond: Predicate::AttackedDefenderWithCountAtLeast {
                        who: PlayerRef::Triggerer,
                        defender: PlayerRef::You,
                        at_least: 2,
                        include_planeswalkers: false,
                    },
                    then: Box::new(draw()),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::SecondCardDrawnThisTurn,
                    EventScope::OpponentControl,
                ),
                effect: draw(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::SpellsCastThisTurnEquals {
                        who: PlayerRef::Triggerer,
                        count: Value::Const(2),
                    }),
                effect: draw(),
            },
        ],
        ..Default::default()
    }
}

// ── The doubling / tripling replacements ───────────────────────────────────
//
// CR 614 replacement effects that multiply something. Two printed clauses and
// four cards: the draw doubler with the draw-step exception, and the ×3 damage
// multiplier.

/// The clause Alhammarret's Archive and Teferi's Ageless Insight share: "If
/// you would draw a card except the first one you draw in each of your draw
/// steps, draw two cards instead."
///
/// ⚠ Not `ControllerDrawsDoubled` (Thought Reflection). The exception is the
/// difference: under Thought Reflection your draw-step draw is doubled and
/// under these two it is not, which at one card a turn is the whole gap
/// between a four-mana enchantment and a seven-mana one.
fn draws_doubled_except_the_draw_step() -> StaticAbility {
    StaticAbility {
        description: "If you would draw a card except the first one you draw in each of \
                      your draw steps, draw two cards instead.",
        effect: StaticEffect::ControllerDrawsDoubledExceptFirstEachDrawStep,
    }
}

/// Alhammarret's Archive — {5} Legendary Artifact. "If you would gain life,
/// you gain twice that much life instead. If you would draw a card except the
/// first one you draw in each of your draw steps, draw two cards instead."
/// (EDHREC 975.)
pub fn alhammarrets_archive() -> CardDefinition {
    CardDefinition {
        name: "Alhammarret's Archive",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "If you would gain life, you gain twice that much life instead.",
                effect: StaticEffect::LifeGainMultiplier {
                    target: PlayerStaticTarget::Controller,
                    factor: 2,
                },
            },
            draws_doubled_except_the_draw_step(),
        ],
        ..Default::default()
    }
}

/// Teferi's Ageless Insight — {2}{U}{U} Legendary Enchantment. The Archive's
/// draw half on its own. (EDHREC 590.)
pub fn teferis_ageless_insight() -> CardDefinition {
    CardDefinition {
        name: "Teferi's Ageless Insight",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![draws_doubled_except_the_draw_step()],
        ..Default::default()
    }
}

/// The clause Fiery Emancipation and City on Fire share: "If a source you
/// control would deal damage to a permanent or player, it deals triple that
/// damage instead."
fn your_sources_deal_triple() -> StaticAbility {
    StaticAbility {
        description: "If a source you control would deal damage to a permanent or player, \
                      it deals triple that damage instead.",
        effect: StaticEffect::MultiplyDamageFromYourSources { factor: 3 },
    }
}

/// Fiery Emancipation — {3}{R}{R}{R} Enchantment. (EDHREC 855.)
///
/// ⚠ **Triple, not two doublings.** The damage funnel accumulates *doublings*
/// (`amount << doublers`), so a ×3 has no exponent to add — and ×2 is not an
/// acceptable stand-in on a six-mana enchantment whose whole text is the
/// multiplier. `MultiplyDamageFromYourSources` rides its own accumulator.
pub fn fiery_emancipation() -> CardDefinition {
    CardDefinition {
        name: "Fiery Emancipation",
        cost: cost(&[generic(3), r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![your_sources_deal_triple()],
        ..Default::default()
    }
}

/// City on Fire — {5}{R}{R}{R} Enchantment with Convoke. The same multiplier
/// two mana later, with the creatures to pay for it. (EDHREC 874.)
pub fn city_on_fire() -> CardDefinition {
    CardDefinition {
        name: "City on Fire",
        cost: cost(&[generic(5), r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Convoke],
        static_abilities: vec![your_sources_deal_triple()],
        ..Default::default()
    }
}

// ── Vivid: one mana of EACH colour among your permanents ───────────────────

/// The ability both cards print: "**Vivid** — {T}: For each color among
/// permanents you control, add one mana of that color."
///
/// ⚠ Not `ManaPayload::AnyColorAmongYourPermanents` (Meteor Crater), which
/// reads the same colour set and adds **one** mana chosen from it. These two
/// agree exactly on a one-colour board and differ by everything above it,
/// which is where the mistake would hide.
fn vivid_tap() -> crate::card::ActivatedAbility {
    crate::card::ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: crate::effect::ManaPayload::OneOfEachColorAmongYourPermanents,
        },
        ..Default::default()
    }
}

/// The count both cards scale on: distinct colours among permanents you
/// control. `Selector::EachPermanent(ControlledByYou)` reads **computed**
/// colours, so a permanent made another colour counts as that colour — the
/// same set the mana ability taps for.
fn colors_among_your_permanents() -> Value {
    Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou)))
}

/// Bloom Tender — {1}{G} Creature — Elf Druid 1/1. "Vivid — {T}: For each
/// color among permanents you control, add one mana of that color."
/// (EDHREC 257, the highest-ranked card left in the top-1000 gap.)
pub fn bloom_tender() -> CardDefinition {
    CardDefinition {
        name: "Bloom Tender",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![vivid_tap()],
        ..Default::default()
    }
}

/// Faeburrow Elder — {1}{G}{W} Creature — Treefolk Druid 0/0. "Vigilance.
/// This creature gets +1/+1 for each color among permanents you control.
/// {T}: For each color among permanents you control, add one mana of that
/// color." (EDHREC 501.)
///
/// ⚠ The printed body is **0/0** and the card survives only on the pump —
/// itself is a permanent you control, so a G/W Elder is at least 2/2 on an
/// otherwise empty board. A 0/0 printed body with a live self-anthem is the
/// shape where a layer bug is a state-based death rather than a wrong number.
pub fn faeburrow_elder() -> CardDefinition {
    CardDefinition {
        name: "Faeburrow Elder",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each color among permanents you control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: colors_among_your_permanents(),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        activated_abilities: vec![vivid_tap()],
        ..Default::default()
    }
}
