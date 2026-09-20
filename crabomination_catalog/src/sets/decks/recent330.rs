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

// ── Blink: exile a permanent you control, then return it ───────────────────
//
// Two cards, one `Effect::ExileAndReturnToOwner`, and the same "up to one
// target" wrapper. ⚠ **"Up to one target" is `OptionalTargets { min: 0 }`,
// not an absent target**: the trigger still goes on the stack with a slot the
// controller may decline, and with `min: 1` a board holding nothing legal
// would remove the trigger (CR 603.3d) instead of resolving it emptily.

/// The shared body: blink the permanent in slot 0, or nothing.
fn blink_up_to_one(filter: R) -> Effect {
    Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::ExileAndReturnToOwner { what: target_filtered(filter) }),
    }
}

/// Displacer Kitten — {3}{U} Creature — Cat Beast 2/2. "Avoidance — Whenever
/// you cast a noncreature spell, exile up to one target nonland permanent you
/// control, then return that card to the battlefield under its owner's
/// control." (EDHREC 550.)
///
/// The trigger is on the **cast**, so it resolves above the spell that caused
/// it — which is the whole card: the blinked permanent's enters trigger
/// happens before the noncreature spell resolves.
pub fn displacer_kitten() -> CardDefinition {
    CardDefinition {
        name: "Displacer Kitten",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Noncreature,
                },
            ),
            effect: blink_up_to_one(R::Nonland.and(R::ControlledByYou)),
        }],
        ..Default::default()
    }
}

/// Teleportation Circle — {3}{W} Enchantment. "At the beginning of your end
/// step, exile up to one target artifact or creature you control, then return
/// that card to the battlefield under its owner's control." (EDHREC 992.)
pub fn teleportation_circle() -> CardDefinition {
    CardDefinition {
        name: "Teleportation Circle",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::YourControl,
            ),
            effect: blink_up_to_one(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
        }],
        ..Default::default()
    }
}

// ── {3} artifacts that tap for any colour, plus a rider ────────────────────

/// The half both share: `{T}: Add one mana of any color.`
fn tap_for_any_color() -> crate::card::ActivatedAbility {
    crate::card::ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
        },
        ..Default::default()
    }
}

/// Relic of Legends — {3} Artifact. "{T}: Add one mana of any color. Tap an
/// untapped legendary creature you control: Add one mana of any color."
/// (EDHREC 536.)
///
/// ⚠ The second ability taps **another** permanent and not itself, so
/// `tap_cost` stays false and the filter carries `Untapped` — the printed word
/// is "an untapped legendary creature", and without it the cost could be paid
/// by a creature that is already tapped. Two mana a turn off one artifact is
/// the whole card.
pub fn relic_of_legends() -> CardDefinition {
    CardDefinition {
        name: "Relic of Legends",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            tap_for_any_color(),
            crate::card::ActivatedAbility {
                tap_other_filter: Some(
                    R::Creature
                        .and(R::HasSupertype(Supertype::Legendary))
                        .and(R::ControlledByYou)
                        .and(R::Untapped),
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Decanter of Endless Water — {3} Artifact. "You have no maximum hand size.
/// {T}: Add one mana of any color." (EDHREC 318.)
pub fn decanter_of_endless_water() -> CardDefinition {
    CardDefinition {
        name: "Decanter of Endless Water",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        activated_abilities: vec![tap_for_any_color()],
        ..Default::default()
    }
}

// ── Cards the engine already documented and had never shipped ──────────────
//
// `scripts/audit_doc_card_names.py` lists card names cited in a `StaticEffect`
// / `Effect` / `Keyword` doc comment and absent from the catalog. Most are
// harmless design rationale, but a few name a primitive that fits the card
// exactly — a shipped ability with no card on it. These are those.

/// Boon Reflection — {4}{W} Enchantment. "If you would gain life, you gain
/// twice that much life instead."
///
/// `StaticEffect::LifeGainMultiplier`'s own doc has said "Rhox Faithmender /
/// Boon Reflection" since it was written; only the Faithmender shipped.
pub fn boon_reflection() -> CardDefinition {
    CardDefinition {
        name: "Boon Reflection",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you would gain life, you gain twice that much life instead.",
            effect: StaticEffect::LifeGainMultiplier {
                target: PlayerStaticTarget::Controller,
                factor: 2,
            },
        }],
        ..Default::default()
    }
}

/// Thousand-Year Elixir — {3} Artifact. "You may activate abilities of
/// creatures you control as though those creatures had haste. {1}, {T}: Untap
/// target creature." (EDHREC 882.)
///
/// `StaticEffect::ControllerCreatureAbilitiesAsThoughHaste` was documented as
/// "Tyvar, Jubilant Brawler; **Thousand-Year Elixir kin**" — the Elixir is the
/// card the ability is named after and was the one not in the catalog.
///
/// ⚠ The two halves are independent and only the first is the famous one: the
/// static exempts the controller's creatures from CR 602.5g's summoning-
/// sickness gate on `{T}` costs, and the untap is an ordinary targeted ability
/// that works on anyone's creature.
pub fn thousand_year_elixir() -> CardDefinition {
    CardDefinition {
        name: "Thousand-Year Elixir",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "You may activate abilities of creatures you control as though \
                          those creatures had haste.",
            effect: StaticEffect::ControllerCreatureAbilitiesAsThoughHaste,
        }],
        activated_abilities: vec![crate::card::ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── COMMANDER_BACKLOG top-1000, 2026-09-20 batch ────────────────────────────
// Seven rows off section 2 that needed no new primitive. Two of them are the
// first users of this run's two: Witch's Cottage names **your graveyard** in a
// target clause (the fifty-second find) and Brash Taunter prints "another
// target creature" (the fifty-third).

/// Reconnaissance Mission — {2}{U}{U} Enchantment. "Whenever a creature you
/// control deals combat damage to a player, you may draw a card. Cycling {2}."
/// (EDHREC 894.)
///
/// Printed "**a** creature", not "one or more", so no `once_per_batch`: an
/// alpha strike draws one card per connecting creature (CR 603.2c).
pub fn reconnaissance_mission() -> CardDefinition {
    CardDefinition {
        name: "Reconnaissance Mission",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

/// Moldervine Reclamation — {3}{B}{G} Enchantment. "Whenever a creature you
/// control dies, you gain 1 life and draw a card." (EDHREC 986.)
pub fn moldervine_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Moldervine Reclamation",
        cost: cost(&[generic(3), crate::mana::b(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Tribute to the World Tree — {G}{G}{G} Enchantment. "Whenever a creature you
/// control enters, draw a card if its power is 3 or greater. Otherwise, put
/// two +1/+1 counters on it." (EDHREC 609.)
///
/// One trigger with two branches, not two triggers: the printed "otherwise"
/// makes them exclusive, and both read the *entering* creature.
pub fn tribute_to_the_world_tree() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Tribute to the World Tree",
        cost: cost(&[g(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::PowerAtLeast(3)),
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Dragon Tempest — {1}{R} Enchantment. "Whenever a creature you control with
/// flying enters, it gains haste until end of turn. Whenever a Dragon you
/// control enters, it deals X damage to any target, where X is the number of
/// Dragons you control." (EDHREC 847.)
///
/// Two separate triggers, and a Dragon with flying fires both.
pub fn dragon_tempest() -> CardDefinition {
    use crate::effect::{Duration, shortcut::target_any};
    CardDefinition {
        name: "Dragon Tempest",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                    }),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::DealDamage {
                    to: target_any(),
                    // "the number of Dragons you control" — counted as the
                    // trigger resolves, so the entering Dragon is included.
                    amount: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Dragon).and(R::ControlledByYou),
                        )),
                        filter: R::Any,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Thopter Spy Network — {2}{U}{U} Enchantment. "At the beginning of your
/// upkeep, if you control an artifact, create a 1/1 colorless Thopter artifact
/// creature token with flying. Whenever one or more artifact creatures you
/// control deal combat damage to a player, draw a card." (EDHREC 940.)
///
/// The second ability *does* print "one or more", so it carries
/// `once_per_batch` (CR 603.2c): three Thopters connecting with one player is
/// one card, and with two players it is two.
pub fn thopter_spy_network() -> CardDefinition {
    use crate::card::TokenDefinition;
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Thopter Spy Network",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    n: Value::ONE,
                }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(thopter),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.and(R::Creature),
                    })
                    .once_per_batch(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Witch's Cottage — Land — Swamp. "This land enters tapped unless you control
/// three or more other Swamps. When this land enters untapped, you may put
/// target creature card from your graveyard on top of your library."
/// (EDHREC 887.)
///
/// Mystic Sanctuary's shape in black, and the two things it keeps from that
/// card: the conditional entry is a **replacement** (CR 614.1c,
/// `EntersTappedUnless`) and the recursion is a real trigger beside it, whose
/// intervening `if` is the same predicate. ⚠ The filter names **your
/// graveyard** — without the zone the same requirement is met by a creature on
/// the battlefield (ENGINE_BACKLOG's fifty-second find).
pub fn witchs_cottage() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::{LibraryPosition, ManaPayload, ZoneDest};
    use crate::mana::Color;
    // "three or more OTHER Swamps" — four including this one.
    let four_swamps = || Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(
            R::HasLandType(LandType::Swamp).and(R::ControlledByYou),
        ),
        n: Value::Const(4),
    };
    CardDefinition {
        name: "Witch's Cottage",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Swamp],
            ..Default::default()
        },
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control three or more other Swamps.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: four_swamps(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::ONE),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: four_swamps(),
                then: Box::new(Effect::MayDo {
                    description: "Put a creature card from your graveyard on top of your library?"
                        .into(),
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.from_your_graveyard()),
                        to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Brash Taunter — {4}{R} 1/1 Goblin. "Indestructible. Whenever this creature
/// is dealt damage, it deals that much damage to target opponent. {2}{R}, {T}:
/// This creature fights another target creature." (EDHREC 791.)
///
/// The first ability is the pod half: an indestructible 1/1 that redirects
/// every point it takes at **one** opponent, so its rate is per-fight rather
/// than per-table. ⚠ The fight's printed clause is "**another** target
/// creature" and is narrowed here to a creature you don't control, for the
/// reason `scripts/audit_another_target.py`'s allowlist records: a mandatory
/// fight slot with nothing else legal hands the bot its own creature, and a
/// narrowing cannot make an illegal play.
pub fn brash_taunter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Brash Taunter",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Indestructible],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            effect: Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Void Rend — {W}{U}{B} Instant. "This spell can't be countered. Destroy
/// target nonland permanent." (EDHREC 765.)
pub fn void_rend() -> CardDefinition {
    CardDefinition {
        name: "Void Rend",
        cost: cost(&[w(), u(), crate::mana::b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Destroy { what: target_filtered(R::Nonland.and(R::Permanent)) },
        ..Default::default()
    }
}

/// Unnatural Growth — {1}{G}{G}{G}{G} Enchantment. "At the beginning of each
/// combat, double the power and toughness of each creature you control until
/// end of turn." (EDHREC 442.)
///
/// **Each** combat, so `EventScope::AnyPlayer` — in a pod that is one doubling
/// per seat's combat, which is where the card's Commander rank comes from.
/// Zopandrel's shape: a `ForEach` that adds each creature's own P/T back to
/// itself, read per-creature off the loop binding rather than once.
pub fn unnatural_growth() -> CardDefinition {
    use crate::effect::Duration;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Unnatural Growth",
        cost: cost(&[generic(1), g(), g(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::AnyPlayer,
            ),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    toughness: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Ayara, First of Locthwain — {B}{B}{B} 2/3 Legendary Elf Noble. "Whenever
/// Ayara or another black creature you control enters, each opponent loses 1
/// life and you gain 1 life. {T}, Sacrifice another black creature: Draw a
/// card." (EDHREC 854.)
///
/// "Ayara **or another**" is exactly `EventScope::YourControl` — the scope
/// already includes the source, so no second trigger. The drain is
/// per-opponent, which is the pod half.
pub fn ayara_first_of_locthwain() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::mana::{Color, b};
    CardDefinition {
        name: "Ayara, First of Locthwain",
        cost: cost(&[b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasColor(Color::Black)),
                }),
            effect: crate::effect::shortcut::drain(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            // "another black creature" — `sac_other_filter` cannot name the
            // source, which is the printed "another".
            sac_other_filter: Some((R::Creature.and(R::HasColor(Color::Black)), 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tempt with Discovery — {3}{G} Sorcery. "**Tempting offer** — Search your
/// library for a land card and put it onto the battlefield. Each opponent may
/// search their library for a land card and put it onto the battlefield. For
/// each opponent who searches a library this way, search your library for a
/// land card and put it onto the battlefield." (EDHREC 870.)
///
/// `Effect::TemptingOffer` is the whole clause (CR 207.2c): the body runs for
/// the controller, each opponent may copy it, and the controller re-runs it
/// once per acceptor. A five-seat pod where everyone accepts is five lands.
pub fn tempt_with_discovery() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Tempt with Discovery",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::TemptingOffer {
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
        ..Default::default()
    }
}

/// Cut a Deal — {2}{W} Sorcery. "Each opponent draws a card, then you draw a
/// card for each opponent who drew a card this way." (EDHREC 990.)
///
/// Nobody may decline, so "each opponent who drew" is the opponent count —
/// `Value::OpponentCount`, which is 1 in a duel and scales with the pod.
pub fn cut_a_deal() -> CardDefinition {
    CardDefinition {
        name: "Cut a Deal",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            Effect::Draw { who: Selector::You, amount: Value::OpponentCount },
        ]),
        ..Default::default()
    }
}

/// Padeem, Consul of Innovation — {3}{U} 1/4 Legendary Vedalken Artificer.
/// "Artifacts you control have hexproof. At the beginning of your upkeep, if
/// you control the artifact with the greatest mana value or tied for the
/// greatest mana value, draw a card." (EDHREC 909.)
///
/// The tie is what the comparison has to allow, so it is "the greatest among
/// **your** artifacts is at least the greatest among **all** artifacts" —
/// `Value::HighestManaValueAmong` on both sides of `Predicate::ValueAtLeast`.
/// ⚠ With no artifact anywhere both sides are 0 and the gate would pass, so
/// the intervening `if` also asks that you control one.
pub fn padeem_consul_of_innovation() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Padeem, Consul of Innovation",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Artifacts you control have hexproof.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                keyword: Keyword::Hexproof,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            )
            .with_filter(Predicate::All(vec![
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    n: Value::ONE,
                },
                Predicate::ValueAtLeast(
                    Value::HighestManaValueAmong(Box::new(Selector::EachPermanent(
                        R::Artifact.and(R::ControlledByYou),
                    ))),
                    Value::HighestManaValueAmong(Box::new(Selector::EachPermanent(R::Artifact))),
                ),
            ])),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Cyberdrive Awakener — {5}{U} 4/4 Artifact Creature — Construct. "Flying.
/// Other artifact creatures you control have flying. When this creature
/// enters, each noncreature artifact you control becomes a 4/4 artifact
/// creature until end of turn." (EDHREC 1003.)
///
/// `BecomeCreature`, not `BecomeCreatureLosingTypes`: the printed line adds
/// the creature type and keeps the artifact one, which is also why the
/// animated Signets pick up the anthem above them on the same resolution.
pub fn cyberdrive_awakener() -> CardDefinition {
    use crate::effect::{Duration, shortcut::etb};
    CardDefinition {
        name: "Cyberdrive Awakener",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![etb(Effect::BecomeCreature {
            what: Selector::EachPermanent(
                R::Artifact.and(R::Creature.negate()).and(R::ControlledByYou),
            ),
            power: Value::Const(4),
            toughness: Value::Const(4),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

// ── Symmetric escalation: the whole table, growing every turn ──────────────

/// Descent into Avernus — {2}{R} Enchantment. "At the beginning of your
/// upkeep, put two descent counters on this enchantment. Then each player
/// creates X Treasure tokens and this enchantment deals X damage to each
/// player, where X is the number of descent counters on this enchantment."
/// (EDHREC 883.)
///
/// ⚠ **Symmetric, and that is the card**: X is read *after* the two counters
/// go on, both clauses read the same X, and "each player" includes its own
/// controller. At four seats it is eight Treasures and eight damage split
/// across the table on the second upkeep — the scaling is why it is a
/// Commander card and why the test is a four-seat game.
///
/// `CounterType::Descent` is its own kind rather than `Charge`: the printed
/// word is what a "remove a charge counter" cost would look for.
pub fn descent_into_avernus() -> CardDefinition {
    let x = || Value::CountersOn {
        what: Box::new(Selector::This),
        kind: crate::card::CounterType::Descent,
    };
    CardDefinition {
        name: "Descent into Avernus",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::Descent,
                    amount: Value::Const(2),
                },
                Effect::CreateToken {
                    who: PlayerRef::EachPlayer,
                    count: x(),
                    definition: std::sync::Arc::new(crate::game::effects::treasure_token()),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: x(),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Eerie Interlude — {2}{W} Instant. "Exile any number of target creatures you
/// control. Return those cards to the battlefield under their owner's control
/// at the beginning of the next end step." (EDHREC 971.)
///
/// ⚠ **The deferred return is the point, and it is what separates this from
/// the immediate blink above.** A board wipe resolves while the creatures are
/// in exile, so they come back at the end step having missed it — a flicker
/// that returned them straight away would save nothing. `min_targets: 0`
/// because "any number" includes none.
pub fn eerie_interlude() -> CardDefinition {
    CardDefinition {
        name: "Eerie Interlude",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByYou),
            effect: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                what: Selector::Target(0),
                tapped: false,
            }),
        },
        ..Default::default()
    }
}
