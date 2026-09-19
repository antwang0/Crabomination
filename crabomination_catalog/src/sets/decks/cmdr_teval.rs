//! Commander: the Teval, the Balanced Scale batch — Teval plus the cards EDHREC
//! lists alongside it (and a few from the Umbris, Fear Manifest page) that the
//! catalog lacked. Graveyard churn: self-mill, "cards leave your graveyard"
//! payoffs, land recursion, and the Sultai mana base. Tests in
//! `tests/recent_b/cmdr_teval.rs`.
//!
//! New primitives this batch added (engine side):
//! * `Effect::ChooseUnchosenModeThisTurn` — "choose one that hasn't been chosen
//!   this turn" (Teval's Judgment).
//! * `Value::CardsToGraveyardThisTurn` — the per-turn graveyard tally as a count
//!   (Welcome the Dead).
//! * `SpendRestriction::ColoredSpellWithoutX` (Titans' Nest).
//! * `StaticEffect::GraveyardCardsHaveEscapeMatching` — a filtered,
//!   optionally your-turn-only Underworld Breach (Kotis, Sibsig Champion).
//!
//! Residuals (approximated or omitted clauses — each also on its card):
//! * Colossal Grave-Reaver — "put one of them onto the battlefield" puts the
//!   first creature card of the milled batch, not one of the controller's choice.
//! * Kotis — the graveyard cast isn't limited to once per turn.
//! * Syr Konrad — "from anywhere other than the battlefield" reads as "not put
//!   into a graveyard from the battlefield earlier this turn".
//! * The Scarab God — the token is a Zombie *in addition to* its copied
//!   creature types (the printed "except it's a 4/4 black Zombie" replaces them).
//! * Shigeki — the "rest into your graveyard" is modelled as milling all four and
//!   then moving the land, so the land briefly passes through the graveyard
//!   (a "cards leave your graveyard" payoff sees it).
//! * Steward of the Harvest — only the Steward itself (not every creature you
//!   control) has the exiled lands' activated abilities; the exile is a
//!   resolution-time pick rather than a target.
//! * River Kelpie — "casts a spell from a graveyard" also fires on exile casts
//!   (`SpellNotCastFromHand`).
//! * Lost Monarch of Ifnir — the second-main trigger reads "a Zombie *you
//!   control* dealt combat damage to a player".
//! * Lethal Scheme — the convoking creatures don't connive (the engine doesn't
//!   remember which creatures convoked a spell).
//! * Welcome the Dead — X counts every card put into your graveyard this turn,
//!   not only those from your hand or library.
//! * Necromantic Selection — the returned creature becomes a Zombie but not black
//!   (and so does any other creature you reanimated earlier that turn).
//! * Agadeem's Awakening — the returns are picked at resolution, not targeted.
//! * Palantír of Orthanc / Midnight Clock / The Soul Stone — influence and hour
//!   counters (and the harnessed state) are charge counters; The Soul Stone has
//!   no Infinity Stone subtype.
//! * Reflections of Littjara — the creature type is chosen by an ETB trigger,
//!   not "as this enters".
//! * Rogue Class — cards exiled before level 3 don't become playable when it
//!   reaches level 3 (only exiles made at level 3 are playable).
//! * Ashiok, Wicked Manipulator — the life-payment replacement is omitted.
//! * Scavenger Grounds — "Sacrifice a Desert" sacrifices Scavenger Grounds itself.
//! * Witch's Clinic — "target commander" is "target legendary creature".
//! * The Black Gate — the creature can't be blocked at all this turn, gated on an
//!   opponent having the most life, rather than "by that player's creatures".
//! * Minas Morgul — the Wraith type doesn't fall off with the shadow counter.
//! * Stonespeaker Crystal — exiles every opponent's graveyard (not "any number
//!   of target players'").

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, LandType, LoyaltyAbility, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::catalog::sets::{tap_add, tap_add_colorless};
use crate::effect::shortcut::{encore, etb, landfall, on_attack, on_dies, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{
    Color, ManaCost, SpendRestriction, b, cost, g, generic, hybrid, u, x,
};
use std::sync::Arc;

// ── Shared shapes ──────────────────────────────────────────────────────────────

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, power, toughness)
    }
}

fn permanent(name: &'static str, mana: ManaCost, types: Vec<CardType>) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: types, ..Default::default() }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn land(name: &'static str, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

fn enters_tapped_unless(description: &'static str, condition: Predicate) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::EntersTappedUnless { applies_to: Selector::This, condition },
    }
}

/// "As this enters, you may pay 3 life. If you don't, it enters tapped." —
/// the shape the catalog's Zendikar Rising pathway backs use (a self-ETB
/// `ChooseMode`: mode 0 pays the life, mode 1 taps).
fn pay_three_or_tapped() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::ChooseMode(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
            Effect::Tap { what: Selector::This },
        ]),
    }
}

fn mill(n: i32) -> Effect {
    Effect::Mill { who: Selector::You, amount: Value::Const(n) }
}

fn token(
    name: &str,
    colors: Vec<Color>,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power,
        toughness,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

/// 2/2 black Zombie Druid (Teval, Teval's Judgment, Welcome the Dead).
fn zombie_druid() -> TokenDefinition {
    token("Zombie Druid", vec![Color::Black], vec![CreatureType::Zombie, CreatureType::Druid], 2, 2)
}

fn make(def: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(def) }
}

/// "Whenever one or more cards leave your graveyard" — one fire per batch.
fn cards_leave_your_graveyard() -> EventSpec {
    EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl).once_per_batch()
}

/// "Whenever one or more creature cards are put into your graveyard from your
/// library" — one fire per mill batch, bound to the first creature card.
fn creature_cards_milled() -> EventSpec {
    EventSpec::new(EventKind::CardMilled, EventScope::YourControl)
        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
        .once_per_batch()
}

fn your_graveyard(filter: R) -> Selector {
    Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter }
}

/// "Return [up to one] `filter` card from your graveyard to `to`" — a
/// resolution-time pick.
fn from_graveyard(filter: R, up_to: bool, to: ZoneDest) -> Effect {
    Effect::MoveChosen { from: your_graveyard(filter), filter: None, count: Value::ONE, up_to, to }
}

fn to_battlefield(tapped: bool) -> ZoneDest {
    ZoneDest::Battlefield { controller: PlayerRef::You, tapped }
}

fn upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

fn your_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect,
    }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

// ── Teval and its creatures ────────────────────────────────────────────────────

/// Teval, the Balanced Scale — {1}{B}{G}{U} Legendary Creature — Spirit Dragon
/// 4/4. Flying. Whenever Teval attacks, mill three cards. Then you may return a
/// land card from your graveyard to the battlefield tapped. Whenever one or more
/// cards leave your graveyard, create a 2/2 black Zombie Druid creature token.
pub fn teval_the_balanced_scale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                mill(3),
                from_graveyard(R::Land, true, to_battlefield(true)),
            ])),
            TriggeredAbility {
                event: cards_leave_your_graveyard(),
                effect: make(zombie_druid(), Value::ONE),
            },
        ],
        ..legend(
            "Teval, the Balanced Scale",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Spirit, CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Thranduil, Sindarin Liege // Silvan Rally — {2}{G/U}{G/U} Legendary Creature
/// — Elf Noble 2/3. Other Elves you control get +1/+1. Landfall — Whenever a
/// land you control enters, create a 1/1 green Elf creature token.
/// Adventure: Silvan Rally {1}{G/U}{G/U} Sorcery — Mill four cards, then put up
/// to two land cards from among them into your hand.
pub fn thranduil_sindarin_liege() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Elves you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Elf))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![landfall(make(
            token("Elf", vec![Color::Green], vec![CreatureType::Elf], 1, 1),
            Value::ONE,
        ))],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Silvan Rally",
            cost: cost(&[generic(1), gu(), gu()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::MillThenToHandN {
                amount: Value::Const(4),
                filter: R::Land,
                take: Value::Const(2),
                otherwise: None,
            },
        })),
        ..legend(
            "Thranduil, Sindarin Liege",
            cost(&[generic(2), gu(), gu()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            2,
            3,
        )
    }
}

/// Colossal Grave-Reaver — {6}{B}{G} Creature — Dragon 7/6. Flying. Whenever
/// this creature enters or attacks, mill three cards. Whenever one or more
/// creature cards are put into your graveyard from your library, put one of
/// them onto the battlefield.
///
/// Approximation: the creature returned is the first creature card of the
/// milled batch (the trigger's bound subject), not one of the controller's
/// choice.
pub fn colossal_grave_reaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(mill(3)),
            on_attack(mill(3)),
            TriggeredAbility {
                event: creature_cards_milled(),
                effect: Effect::Move { what: Selector::TriggerSource, to: to_battlefield(false) },
            },
        ],
        ..creature(
            "Colossal Grave-Reaver",
            cost(&[generic(6), b(), g()]),
            vec![CreatureType::Dragon],
            7,
            6,
        )
    }
}

/// Sidisi, Brood Tyrant — {1}{B}{G}{U} Legendary Creature — Snake Shaman 3/3.
/// Whenever Sidisi enters or attacks, mill three cards. Whenever one or more
/// creature cards are put into your graveyard from your library, create a 2/2
/// black Zombie creature token.
pub fn sidisi_brood_tyrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(mill(3)),
            on_attack(mill(3)),
            TriggeredAbility {
                event: creature_cards_milled(),
                effect: make(
                    token("Zombie", vec![Color::Black], vec![CreatureType::Zombie], 2, 2),
                    Value::ONE,
                ),
            },
        ],
        ..legend(
            "Sidisi, Brood Tyrant",
            cost(&[generic(1), b(), g(), u()]),
            vec![CreatureType::Snake, CreatureType::Shaman],
            3,
            3,
        )
    }
}

/// River Kelpie — {3}{U}{U} Creature — Beast 3/3. Whenever this creature or
/// another permanent enters from a graveyard, draw a card. Whenever a player
/// casts a spell from a graveyard, draw a card. Persist.
///
/// Approximation: the cast trigger reads "a spell not cast from its owner's
/// hand", so exile casts fire it too.
pub fn river_kelpie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Persist],
        triggered_abilities: vec![
            TriggeredAbility {
                // `R::EnteredFromGraveyardThisTurn`, not
                // `Predicate::TriggerSourceEnteredFromGraveyard`: the latter also
                // reads "not cast from hand", which every played land and token
                // satisfies.
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::EnteredFromGraveyardThisTurn,
                    }),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::SpellNotCastFromHand,
                    },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature("River Kelpie", cost(&[generic(3), u(), u()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Kotis, Sibsig Champion — {B}{G}{U} Legendary Creature — Zombie Warrior 3/3.
/// Once during each of your turns, you may cast a creature spell from your
/// graveyard by exiling three other cards from your graveyard in addition to
/// paying its other costs. Whenever one or more creatures you control enter,
/// if one or more of them entered from a graveyard or was cast from a
/// graveyard, put two +1/+1 counters on Kotis.
///
/// The graveyard cast is `StaticEffect::GraveyardCardsHaveEscapeMatching` (the
/// escape cast path, your turn only), which stamps the creature as having
/// entered from a graveyard. Approximation: it isn't limited to once per turn.
pub fn kotis_sibsig_champion() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, you may cast a creature spell from your graveyard \
                          by exiling three other cards from your graveyard in addition to \
                          paying its other costs.",
            effect: StaticEffect::GraveyardCardsHaveEscapeMatching {
                filter: R::Creature,
                exile_count: 3,
                your_turn_only: true,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::EnteredFromGraveyardThisTurn),
                })
                .once_per_batch(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        }],
        ..legend(
            "Kotis, Sibsig Champion",
            cost(&[b(), g(), u()]),
            vec![CreatureType::Zombie, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Syr Konrad, the Grim — {3}{B}{B} Legendary Creature — Human Knight 5/4.
/// Whenever another creature dies, or a creature card is put into a graveyard
/// from anywhere other than the battlefield, or a creature card leaves your
/// graveyard, Syr Konrad deals 1 damage to each opponent. {1}{B}: Each player
/// mills a card.
///
/// Approximation: "from anywhere other than the battlefield" is read as "a
/// creature card that wasn't put into a graveyard from the battlefield this
/// turn".
pub fn syr_konrad_the_grim() -> CardDefinition {
    let ping = || Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::ONE,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::OtherThanSource,
                    },
                ),
                effect: ping(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::PutIntoGraveyardFromBattlefieldThisTurn.negate()),
                    }),
                effect: ping(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: ping(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Syr Konrad, the Grim",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            5,
            4,
        )
    }
}

/// The Scarab God — {3}{U}{B} Legendary Creature — God 5/5. At the beginning of
/// your upkeep, each opponent loses X life and you scry X, where X is the number
/// of Zombies you control. {2}{U}{B}: Exile target creature card from a
/// graveyard. Create a token that's a copy of it, except it's a 4/4 black
/// Zombie. When The Scarab God dies, return it to its owner's hand at the
/// beginning of the next end step.
///
/// Approximation: the token is a Zombie in addition to its copied creature types.
pub fn the_scarab_god() -> CardDefinition {
    let zombies = || {
        Value::count(Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou),
        ))
    };
    CardDefinition {
        triggered_abilities: vec![
            upkeep(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: zombies() },
                Effect::Scry { who: PlayerRef::You, amount: zombies() },
            ])),
            on_dies(Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                }),
            }),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), b()]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Exile,
                },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::Target(0),
                    extra_creature_types: vec![CreatureType::Zombie],
                    extra_card_types: vec![],
                    override_pt: Some((4, 4)),
                    override_colors: Some(vec![Color::Black]),
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "The Scarab God",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::God],
            5,
            5,
        )
    }
}

/// Shigeki, Jukai Visionary — {1}{G} Legendary Enchantment Creature — Snake
/// Druid 1/3. {1}{G}, {T}, Return Shigeki to its owner's hand: Reveal the top
/// four cards of your library. You may put a land card from among them onto the
/// battlefield tapped. Put the rest into your graveyard. Channel — {X}{X}{G}{G},
/// Discard this card: Return X target nonlegendary cards from your graveyard to
/// your hand.
///
/// Approximations: the four cards are milled and the land then moved out of the
/// graveyard (so it briefly passes through it); the Channel returns are picked
/// at resolution rather than targeted.
pub fn shigeki_jukai_visionary() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), g()]),
                tap_cost: true,
                return_self_cost: true,
                effect: Effect::Seq(vec![
                    mill(4),
                    Effect::MoveChosen {
                        from: Selector::LastMoved,
                        filter: Some(R::Land),
                        count: Value::ONE,
                        up_to: true,
                        to: to_battlefield(true),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x(), x(), g(), g()]),
                from_hand: true,
                discard_self_cost: true,
                effect: Effect::MoveChosen {
                    from: your_graveyard(R::HasSupertype(Supertype::Legendary).negate()),
                    filter: None,
                    count: Value::XFromCost,
                    up_to: true,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..legend(
            "Shigeki, Jukai Visionary",
            cost(&[generic(1), g()]),
            vec![CreatureType::Snake, CreatureType::Druid],
            1,
            3,
        )
    }
}

/// Floral Evoker — {2}{G} Creature — Snake Druid 2/3. Landfall — Whenever a land
/// you control enters, put a +1/+1 counter on this creature. {G}, Discard a
/// creature card: Return target land card from your graveyard to the
/// battlefield tapped.
pub fn floral_evoker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::Move {
                what: target_filtered(R::Land.and(R::InYourGraveyard)),
                to: to_battlefield(true),
            },
            ..Default::default()
        }],
        ..creature(
            "Floral Evoker",
            cost(&[generic(2), g()]),
            vec![CreatureType::Snake, CreatureType::Druid],
            2,
            3,
        )
    }
}

/// Steward of the Harvest — {3}{G} Creature — Human Druid 3/3. When this
/// creature enters, exile up to three target land cards from your graveyard.
/// Creatures you control have all activated abilities of all land cards exiled
/// with this creature.
///
/// Approximations: only the Steward itself has the exiled lands' activated
/// abilities (`HasActivatedAbilitiesOfExiledWithSelf`); the exile is a
/// resolution-time pick rather than targets.
pub fn steward_of_the_harvest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MoveChosen {
            from: your_graveyard(R::Land),
            filter: None,
            count: Value::Const(3),
            up_to: true,
            to: ZoneDest::ExileWithSourceStamp,
        })],
        static_abilities: vec![StaticAbility {
            description: "This creature has all activated abilities of all land cards exiled \
                          with it.",
            effect: StaticEffect::HasActivatedAbilitiesOfExiledWithSelf,
        }],
        ..creature(
            "Steward of the Harvest",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Diviner of Mist — {4}{U} Creature — Dragon 4/5. Flying. Whenever this
/// creature attacks, mill four cards. You may cast an instant or sorcery spell
/// from your graveyard with mana value 4 or less without paying its mana cost.
/// If that spell would be put into your graveyard, exile it instead.
pub fn diviner_of_mist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            mill(4),
            Effect::CastWithoutPayingImmediate {
                what: Selector::take(
                    your_graveyard(instant_or_sorcery().and(R::ManaValueAtMost(4))),
                    Value::ONE,
                ),
                source_zone: Zone::Graveyard,
                exile_after: true,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
        ]))],
        ..creature("Diviner of Mist", cost(&[generic(4), u()]), vec![CreatureType::Dragon], 4, 5)
    }
}

/// Lost Monarch of Ifnir — {3}{B} Creature — Zombie Noble 4/4. Afflict 3. Other
/// Zombies you control have afflict 3. At the beginning of your second main
/// phase, if a player was dealt combat damage by a Zombie this turn, mill three
/// cards, then you may return a creature card from your graveyard to your hand.
///
/// Afflict is its printed trigger ("whenever this creature becomes blocked,
/// defending player loses 3 life"), granted to the other Zombies via
/// `GrantTriggeredAbility`. Approximation: the main-phase gate reads a Zombie
/// *you control* dealing the damage.
pub fn lost_monarch_of_ifnir() -> CardDefinition {
    let afflict = || TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect: Effect::LoseLife {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::Const(3),
        },
    };
    CardDefinition {
        triggered_abilities: vec![
            afflict(),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::PostCombatMain),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::ProwlTypeDealtCombatDamage {
                    types: vec![CreatureType::Zombie],
                }),
                effect: Effect::Seq(vec![
                    mill(3),
                    from_graveyard(R::Creature, true, ZoneDest::Hand(PlayerRef::You)),
                ]),
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Other Zombies you control have afflict 3.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature
                    .and(R::HasCreatureType(CreatureType::Zombie))
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                ability: Box::new(afflict()),
            },
        }],
        ..creature(
            "Lost Monarch of Ifnir",
            cost(&[generic(3), b()]),
            vec![CreatureType::Zombie, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Lord of Extinction — {3}{B}{G} Creature — Elemental */*. Lord of
/// Extinction's power and toughness are each equal to the number of cards in
/// all graveyards.
pub fn lord_of_extinction() -> CardDefinition {
    let all = || Value::CardsInAllGraveyardsMatching { filter: R::Any };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Lord of Extinction's power and toughness are each equal to the \
                          number of cards in all graveyards.",
            effect: StaticEffect::SelfBasePtFromValue { power: all(), toughness: all() },
        }],
        ..creature(
            "Lord of Extinction",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Elemental],
            0,
            0,
        )
    }
}

/// Tormod, the Desecrator — {3}{B} Legendary Creature — Zombie Wizard 4/2.
/// Whenever one or more cards leave your graveyard, create a tapped 2/2 black
/// Zombie creature token. Partner.
pub fn tormod_the_desecrator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![TriggeredAbility {
            event: cards_leave_your_graveyard(),
            effect: make(
                TokenDefinition {
                    tapped: true,
                    ..token("Zombie", vec![Color::Black], vec![CreatureType::Zombie], 2, 2)
                },
                Value::ONE,
            ),
        }],
        ..legend(
            "Tormod, the Desecrator",
            cost(&[generic(3), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            4,
            2,
        )
    }
}

/// Amphin Mutineer — {3}{U} Creature — Salamander Pirate 3/3. When this
/// creature enters, exile up to one target non-Salamander creature. That
/// creature's controller creates a 4/3 blue Salamander Warrior creature token.
/// Encore {4}{U}{U}.
pub fn amphin_mutineer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::ONE,
                    definition: Arc::new(token(
                        "Salamander Warrior",
                        vec![Color::Blue],
                        vec![CreatureType::Salamander, CreatureType::Warrior],
                        4,
                        3,
                    )),
                },
                Effect::Exile {
                    what: target_filtered(
                        R::Creature.and(R::HasCreatureType(CreatureType::Salamander).negate()),
                    ),
                },
            ])),
        })],
        activated_abilities: vec![encore(cost(&[generic(4), u(), u()]))],
        ..creature(
            "Amphin Mutineer",
            cost(&[generic(3), u()]),
            vec![CreatureType::Salamander, CreatureType::Pirate],
            3,
            3,
        )
    }
}

// ── Instants and sorceries ─────────────────────────────────────────────────────

/// Lethal Scheme — {2}{B}{B} Instant. Convoke. Destroy target creature or
/// planeswalker. Each creature that convoked this spell connives.
///
/// Approximation: the connive rider is omitted — the engine doesn't remember
/// which creatures convoked a spell.
pub fn lethal_scheme() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Lethal Scheme",
            cost(&[generic(2), b(), b()]),
            CardType::Instant,
            Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        )
    }
}

/// Midnight Tilling — {1}{G} Instant. Mill four cards, then you may return a
/// permanent card from among them to your hand.
pub fn midnight_tilling() -> CardDefinition {
    spell(
        "Midnight Tilling",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::MillThenToHand { amount: Value::Const(4), filter: R::PermanentCard, otherwise: None },
    )
}

/// Welcome the Dead — {3}{B} Sorcery. Draw two cards, then discard a card and
/// you lose 2 life. Create X tapped 2/2 black Zombie Druid creature tokens,
/// where X is the number of cards that were put into your graveyard from your
/// hand or library this turn. Flashback {5}{B}.
///
/// Approximation: X is every card put into your graveyard this turn
/// (`Value::CardsToGraveyardThisTurn`), not only from hand or library.
pub fn welcome_the_dead() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), b()]))],
        ..spell(
            "Welcome the Dead",
            cost(&[generic(3), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
                make(
                    TokenDefinition { tapped: true, ..zombie_druid() },
                    Value::CardsToGraveyardThisTurn(PlayerRef::You),
                ),
            ]),
        )
    }
}

/// Necromantic Selection — {4}{B}{B}{B} Sorcery. Destroy all creatures, then
/// return a creature card put into a graveyard this way to the battlefield under
/// your control. It's a black Zombie in addition to its other colors and types.
/// Exile Necromantic Selection.
///
/// Approximations: the returned creature becomes a Zombie but not black; the
/// Zombie type lands on every creature you control that entered from a
/// graveyard this turn (normally just the one returned).
pub fn necromantic_selection() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Necromantic Selection",
            cost(&[generic(4), b(), b(), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
                Effect::MoveChosen {
                    from: Selector::DestroyedThisResolution {
                        filter: R::Creature.and(R::InGraveyard),
                    },
                    filter: None,
                    count: Value::ONE,
                    up_to: false,
                    to: to_battlefield(false),
                },
                // The creature just returned: `MoveChosen` stashes no
                // `LastMoved`, so it is read as the creature you control that
                // entered from a graveyard this turn.
                Effect::AddCreatureTypes {
                    what: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::EnteredFromGraveyardThisTurn),
                    ),
                    creature_types: vec![CreatureType::Zombie],
                    duration: Duration::Permanent,
                },
            ]),
        )
    }
}

/// Rise of the Witch-king — {2}{B}{G} Sorcery. Each player sacrifices a creature
/// of their choice. If you sacrificed a creature this way, you may return
/// another permanent card from your graveyard to the battlefield.
pub fn rise_of_the_witch_king() -> CardDefinition {
    spell(
        "Rise of the Witch-king",
        cost(&[generic(2), b(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::ONE,
                filter: R::Creature,
            },
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(from_graveyard(
                    R::PermanentCard.and(R::NotSacrificedThisResolution),
                    true,
                    to_battlefield(false),
                )),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// The deepest mana value Agadeem's Awakening's per-value return reaches.
const AGADEEM_MAX_MV: u32 = 16;

/// Agadeem's Awakening // Agadeem, the Undercrypt — {X}{B}{B}{B} Sorcery //
/// Land. Return from your graveyard to the battlefield any number of target
/// creature cards that each have a different mana value X or less. // As this
/// land enters, you may pay 3 life. If you don't, it enters tapped. {T}: Add {B}.
///
/// Each mana value 0..=X returns at most one creature card of that value, so
/// the "different mana value" rule holds by construction. Approximation: the
/// cards are picked at resolution rather than targeted.
pub fn agadeems_awakening() -> CardDefinition {
    let per_value = (0..=AGADEEM_MAX_MV)
        .map(|mv| Effect::If {
            cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(mv as i32)),
            then: Box::new(from_graveyard(
                R::Creature.and(R::ManaValueExactly(mv)),
                true,
                to_battlefield(false),
            )),
            else_: Box::new(Effect::Noop),
        })
        .collect();
    CardDefinition {
        back_face: Some(Box::new(CardDefinition {
            supertypes: vec![],
            triggered_abilities: vec![pay_three_or_tapped()],
            ..land("Agadeem, the Undercrypt", vec![tap_add(Color::Black)])
        })),
        ..spell(
            "Agadeem's Awakening",
            cost(&[x(), b(), b(), b()]),
            CardType::Sorcery,
            Effect::Seq(per_value),
        )
    }
}

// ── Enchantments ───────────────────────────────────────────────────────────────

/// Dancing from Dark to Dawn — {3}{G}{G} Enchantment. Whenever you cast a
/// creature spell, put X +1/+1 counters on target creature you control, where X
/// is that spell's mana value. Landfall — Whenever a land you control enters,
/// create a 2/2 green Bear creature token.
pub fn dancing_from_dark_to_dawn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    },
                ),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                },
            },
            landfall(make(
                token("Bear", vec![Color::Green], vec![CreatureType::Bear], 2, 2),
                Value::ONE,
            )),
        ],
        ..permanent(
            "Dancing from Dark to Dawn",
            cost(&[generic(3), g(), g()]),
            vec![CardType::Enchantment],
        )
    }
}

/// Teval's Judgment — {2}{B} Enchantment. Whenever one or more cards leave your
/// graveyard, choose one that hasn't been chosen this turn — • Draw a card.
/// • Create a Treasure token. • Create a 2/2 black Zombie Druid creature token.
pub fn tevals_judgment() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: cards_leave_your_graveyard(),
            effect: Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    make(crabomination_base::tokens::treasure_token(), Value::ONE),
                    make(zombie_druid(), Value::ONE),
                ],
            },
        }],
        ..permanent("Teval's Judgment", cost(&[generic(2), b()]), vec![CardType::Enchantment])
    }
}

/// Crawling Sensation — {2}{G} Enchantment. At the beginning of your upkeep, you
/// may mill two cards. Whenever one or more land cards are put into your
/// graveyard from anywhere for the first time each turn, create a 1/1 green
/// Insect creature token.
pub fn crawling_sensation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            upkeep(Effect::MayDo { description: "Mill two cards?".into(), body: Box::new(mill(2)) }),
            TriggeredAbility {
                event: EventSpec {
                    once_per_turn: true,
                    ..EventSpec::new(EventKind::LandPutIntoGraveyard, EventScope::YourControl)
                },
                effect: make(
                    token("Insect", vec![Color::Green], vec![CreatureType::Insect], 1, 1),
                    Value::ONE,
                ),
            },
        ],
        ..permanent("Crawling Sensation", cost(&[generic(2), g()]), vec![CardType::Enchantment])
    }
}

/// Titans' Nest — {1}{B}{G}{U} Enchantment. At the beginning of your upkeep,
/// surveil 1. Exile a card from your graveyard: Add {C}. Spend this mana only to
/// cast a spell that's one or more colors without {X} in its mana cost.
pub fn titans_nest() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![upkeep(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::Any, 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ONE)),
                    SpendRestriction::ColoredSpellWithoutX,
                ),
            },
            ..Default::default()
        }],
        ..permanent("Titans' Nest", cost(&[generic(1), b(), g(), u()]), vec![CardType::Enchantment])
    }
}

/// Reflections of Littjara — {4}{U} Enchantment. As this enchantment enters,
/// choose a creature type. Whenever you cast a spell of the chosen type, copy
/// that spell. (A copy of a permanent spell becomes a token.)
///
/// Approximation: the type is chosen by an ETB trigger rather than as it enters.
pub fn reflections_of_littjara() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::NameCreatureType { what: Selector::This }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::TriggerObjectIsChosenType),
                effect: Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
            },
        ],
        ..permanent(
            "Reflections of Littjara",
            cost(&[generic(4), u()]),
            vec![CardType::Enchantment],
        )
    }
}

/// `{cost}: Level N. Activate only as a sorcery.` — legal only from `from_level`.
fn level_up(mana: ManaCost, from_level: u8) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from_level)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    }
}

/// Rogue Class — {U}{B} Enchantment — Class. Whenever a creature you control
/// deals combat damage to a player, exile the top card of that player's library
/// face down. You may look at it for as long as it remains exiled. {1}{U}{B}:
/// Level 2 — Creatures you control have menace. {2}{U}{B}: Level 3 — You may
/// play cards exiled with this Class, and you may spend mana as though it were
/// mana of any color to cast those spells.
///
/// At level 3 the exile is `ExileTopFaceDownGrantPlay` (playable, any mana).
/// Approximation: cards exiled before level 3 don't become playable later.
pub fn rogue_class() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Class],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::SourceClassLevelAtLeast(3),
                then: Box::new(Effect::ExileTopFaceDownGrantPlay {
                    library: PlayerRef::Target(0),
                    grantee: PlayerRef::You,
                }),
                else_: Box::new(Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: true,
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Level 2+: Creatures you control have menace.",
            effect: StaticEffect::WhileClassLevelAtLeast {
                n: 2,
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Menace,
                }),
            },
        }],
        activated_abilities: vec![
            level_up(cost(&[generic(1), u(), b()]), 1),
            level_up(cost(&[generic(2), u(), b()]), 2),
        ],
        ..permanent("Rogue Class", cost(&[u(), b()]), vec![CardType::Enchantment])
    }
}

// ── Planeswalker ───────────────────────────────────────────────────────────────

/// Ashiok, Wicked Manipulator — {3}{B}{B} Legendary Planeswalker — Ashiok,
/// loyalty 5. If you would pay life while your library has at least that many
/// cards in it, exile that many cards from the top of your library instead.
/// +1: Look at the top two cards of your library. Exile one of them and put the
/// other into your hand. −2: Create two 1/1 black Nightmare creature tokens with
/// "At the beginning of combat on your turn, if a card was put into exile this
/// turn, put a +1/+1 counter on this token." −7: Target player exiles the top X
/// cards of their library, where X is the total mana value of cards you own in
/// exile.
///
/// Approximation: the life-payment replacement is omitted.
pub fn ashiok_wicked_manipulator() -> CardDefinition {
    let nightmare = TokenDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::CardsExiledThisTurnAtLeast {
                who: PlayerRef::EachPlayer,
                at_least: Value::ONE,
            }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..token("Nightmare", vec![Color::Black], vec![CreatureType::Nightmare], 1, 1)
    };
    CardDefinition {
        name: "Ashiok, Wicked Manipulator",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Ashiok],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::LookTopExileOneOfN { who: PlayerRef::You, count: Value::Const(2) },
                    Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ]),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: make(nightmare, Value::Const(2)),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::TotalManaValueOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Exile,
                        filter: R::Any,
                    })),
                    link_to_source: false,
                    face_down: false,
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}

// ── Artifacts ──────────────────────────────────────────────────────────────────

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..permanent(name, mana, vec![CardType::Artifact])
    }
}

/// Winged Boots — {1}{U} Artifact — Equipment. Equipped creature has flying and
/// ward {4}. Equip {1}.
pub fn winged_boots() -> CardDefinition {
    equipment(
        "Winged Boots",
        cost(&[generic(1), u()]),
        cost(&[generic(1)]),
        EquipBonus {
            keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(4)])))],
            ..Default::default()
        },
    )
}

/// Brotherhood Regalia — {2} Artifact — Equipment. Equipped creature has ward
/// {2}, is an Assassin in addition to its other types, and can't be blocked.
/// Equip legendary creature {1}. Equip {3}.
pub fn brotherhood_regalia() -> CardDefinition {
    CardDefinition {
        equip_filtered_cost: Some((R::HasSupertype(Supertype::Legendary), cost(&[generic(1)]))),
        ..equipment(
            "Brotherhood Regalia",
            cost(&[generic(2)]),
            cost(&[generic(3)]),
            EquipBonus {
                keywords: vec![
                    Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
                    Keyword::Unblockable,
                ],
                add_creature_types: vec![CreatureType::Assassin],
                ..Default::default()
            },
        )
    }
}

/// Conjurer's Closet — {5} Artifact. At the beginning of your end step, you may
/// exile target creature you control, then return that card to the battlefield
/// under your control.
pub fn conjurers_closet() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_end_step(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::ExileAndReturnToOwner {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
            }),
        })],
        ..permanent("Conjurer's Closet", cost(&[generic(5)]), vec![CardType::Artifact])
    }
}

/// Palantír of Orthanc — {3} Legendary Artifact. At the beginning of your end
/// step, put an influence counter on Palantír of Orthanc and scry 2. Then target
/// opponent may have you draw a card. If that player doesn't, you mill X cards,
/// where X is the number of influence counters on Palantír of Orthanc, and that
/// player loses life equal to the total mana value of those cards.
///
/// Approximation: influence counters are charge counters.
pub fn palantir_of_orthanc() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![your_end_step(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            Effect::AnyPlayerMayAccept {
                who: PlayerRef::Target(0),
                prompt: "Have Palantír's controller draw a card?".into(),
                accepted: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                otherwise: Box::new(Effect::Seq(vec![
                    Effect::Mill {
                        who: Selector::You,
                        amount: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::TotalManaValueOf(Box::new(Selector::LastMoved)),
                    },
                ])),
            },
        ]))],
        ..permanent("Palantír of Orthanc", cost(&[generic(3)]), vec![CardType::Artifact])
    }
}

/// Midnight Clock — {2}{U} Artifact. {T}: Add {U}. {2}{U}: Put an hour counter
/// on this artifact. At the beginning of each upkeep, put an hour counter on this
/// artifact. When the twelfth hour counter is put on this artifact, shuffle your
/// hand and graveyard into your library, then draw seven cards. Exile this
/// artifact.
///
/// Approximation: hour counters are charge counters.
pub fn midnight_clock() -> CardDefinition {
    let tick = || Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::Charge,
        amount: Value::ONE,
    };
    CardDefinition {
        activated_abilities: vec![
            tap_add(Color::Blue),
            ActivatedAbility { mana_cost: cost(&[generic(2), u()]), effect: tick(), ..Default::default() },
        ],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
                effect: tick(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CounterAdded(CounterType::Charge), EventScope::SelfSource)
                    .with_filter(Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::Charge,
                        n: 12,
                    }),
                effect: Effect::Seq(vec![
                    Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::You },
                    Effect::Draw { who: Selector::You, amount: Value::Const(7) },
                    Effect::Exile { what: Selector::This },
                ]),
            },
        ],
        ..permanent("Midnight Clock", cost(&[generic(2), u()]), vec![CardType::Artifact])
    }
}

/// The Soul Stone — {1}{B} Legendary Artifact — Infinity Stone. Indestructible.
/// {T}: Add {B}. {6}{B}, {T}, Exile a creature you control: Harness The Soul
/// Stone. ∞ — At the beginning of your upkeep, return target creature card from
/// your graveyard to the battlefield.
///
/// Approximations: "harnessed" is a charge counter (the ∞ trigger is gated on
/// having one); the Infinity Stone subtype isn't modelled.
pub fn the_soul_stone() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![
            tap_add(Color::Black),
            ActivatedAbility {
                mana_cost: cost(&[generic(6), b()]),
                tap_cost: true,
                exile_permanent_cost: Some((R::Creature.and(R::ControlledByYou), 1)),
                effect: Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Charge, n: 1 },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::Charge,
                    n: 1,
                }),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: to_battlefield(false),
            },
        }],
        ..permanent("The Soul Stone", cost(&[generic(1), b()]), vec![CardType::Artifact])
    }
}

/// Stonespeaker Crystal — {4} Artifact. {T}: Add {C}{C}. {2}, {T}, Sacrifice
/// this artifact: Exile any number of target players' graveyards. Draw a card.
///
/// Approximation: exiles every opponent's graveyard.
pub fn stonespeaker_crystal() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(2)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::ExileAllGraveyards { filter: None, opponents_only: true },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
        ],
        ..permanent("Stonespeaker Crystal", cost(&[generic(4)]), vec![CardType::Artifact])
    }
}

// ── Lands ──────────────────────────────────────────────────────────────────────

/// A two-type dual that enters tapped ("Land — Swamp Forest").
fn tapped_typed_dual(
    name: &'static str,
    types: [LandType; 2],
    colors: [Color; 2],
) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { land_types: types.to_vec(), ..Default::default() },
        static_abilities: vec![enters_tapped()],
        ..land(name, vec![tap_add(colors[0]), tap_add(colors[1])])
    }
}

/// Haunted Mire — Land — Swamp Forest. ({T}: Add {B} or {G}.) This land enters
/// tapped.
pub fn haunted_mire() -> CardDefinition {
    tapped_typed_dual("Haunted Mire", [LandType::Swamp, LandType::Forest], [Color::Black, Color::Green])
}

/// Contaminated Aquifer — Land — Island Swamp. ({T}: Add {U} or {B}.) This land
/// enters tapped.
pub fn contaminated_aquifer() -> CardDefinition {
    tapped_typed_dual(
        "Contaminated Aquifer",
        [LandType::Island, LandType::Swamp],
        [Color::Blue, Color::Black],
    )
}

/// Turbulent Wetlands — Land — Island Swamp. ({T}: Add {U} or {B}.) This land
/// enters tapped unless your opponents control eight or more lands.
pub fn turbulent_wetlands() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            land_types: vec![LandType::Island, LandType::Swamp],
            ..Default::default()
        },
        static_abilities: vec![enters_tapped_unless(
            "This land enters tapped unless your opponents control eight or more lands.",
            Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                n: Value::Const(8),
            },
        )],
        ..land("Turbulent Wetlands", vec![tap_add(Color::Blue), tap_add(Color::Black)])
    }
}

/// Choked Estuary — Land. As this land enters, you may reveal an Island or Swamp
/// card from your hand. If you don't, this land enters tapped. {T}: Add {U} or
/// {B}.
pub fn choked_estuary() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped_unless(
            "This land enters tapped unless you reveal an Island or Swamp card from your hand.",
            Predicate::SelectorExists(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Hand,
                filter: R::HasLandType(LandType::Island).or(R::HasLandType(LandType::Swamp)),
            }),
        )],
        ..land("Choked Estuary", vec![tap_add(Color::Blue), tap_add(Color::Black)])
    }
}

/// Shipwreck Marsh — Land. This land enters tapped unless you control two or
/// more other lands. {T}: Add {U} or {B}.
pub fn shipwreck_marsh() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped_unless(
            "This land enters tapped unless you control two or more other lands.",
            Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::Land.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                n: Value::Const(2),
            },
        )],
        ..land("Shipwreck Marsh", vec![tap_add(Color::Blue), tap_add(Color::Black)])
    }
}

/// Sunken Ruins — Land. {T}: Add {C}. {U/B}, {T}: Add {U}{U}, {U}{B}, or {B}{B}.
pub fn sunken_ruins() -> CardDefinition {
    let filter = |colors: Vec<Color>| ActivatedAbility {
        mana_cost: cost(&[hybrid(Color::Blue, Color::Black)]),
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(colors) },
        ..Default::default()
    };
    land(
        "Sunken Ruins",
        vec![
            tap_add_colorless(),
            filter(vec![Color::Blue, Color::Blue]),
            filter(vec![Color::Blue, Color::Black]),
            filter(vec![Color::Black, Color::Black]),
        ],
    )
}

/// River of Tears — Land. {T}: Add {U}. If you played a land this turn, add {B}
/// instead.
pub fn river_of_tears() -> CardDefinition {
    let gated = |color: Color, played: bool| {
        let lands = Value::LandsPlayedThisTurn(PlayerRef::You);
        ActivatedAbility {
            condition: Some(if played {
                Predicate::ValueAtLeast(lands, Value::ONE)
            } else {
                Predicate::ValueAtMost(lands, Value::Const(0))
            }),
            ..tap_add(color)
        }
    };
    land("River of Tears", vec![gated(Color::Blue, false), gated(Color::Black, true)])
}

/// Hidden Lair — Land. {T}: Add {C}. {T}: Add {U} or {B}. Activate only if this
/// land entered this turn or if you control a basic land.
pub fn hidden_lair() -> CardDefinition {
    let gated = |color: Color| ActivatedAbility {
        condition: Some(Predicate::Any(vec![
            Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn },
            Predicate::SelectorExists(Selector::EachPermanent(
                R::IsBasicLand.and(R::ControlledByYou),
            )),
        ])),
        ..tap_add(color)
    };
    land("Hidden Lair", vec![tap_add_colorless(), gated(Color::Blue), gated(Color::Black)])
}

/// Memorial to Folly — Land. This land enters tapped. {T}: Add {B}. {2}{B}, {T},
/// Sacrifice this land: Return target creature card from your graveyard to your
/// hand.
pub fn memorial_to_folly() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped()],
        ..land(
            "Memorial to Folly",
            vec![
                tap_add(Color::Black),
                ActivatedAbility {
                    mana_cost: cost(&[generic(2), b()]),
                    tap_cost: true,
                    sac_cost: true,
                    effect: Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    ..Default::default()
                },
            ],
        )
    }
}

/// Dakmor Salvage — Land. This land enters tapped. {T}: Add {B}. Dredge 2.
pub fn dakmor_salvage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Dredge(2)],
        static_abilities: vec![enters_tapped()],
        ..land("Dakmor Salvage", vec![tap_add(Color::Black)])
    }
}

/// Scavenger Grounds — Land — Desert. {T}: Add {C}. {2}, {T}, Sacrifice a
/// Desert: Exile all graveyards.
///
/// Approximation: the sacrificed Desert is Scavenger Grounds itself.
pub fn scavenger_grounds() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { land_types: vec![LandType::Desert], ..Default::default() },
        ..land(
            "Scavenger Grounds",
            vec![
                tap_add_colorless(),
                ActivatedAbility {
                    mana_cost: cost(&[generic(2)]),
                    tap_cost: true,
                    sac_cost: true,
                    effect: Effect::ExileAllGraveyards { filter: None, opponents_only: false },
                    ..Default::default()
                },
            ],
        )
    }
}

/// Nephalia Drownyard — Land. {T}: Add {C}. {1}{U}{B}, {T}: Target player mills
/// three cards.
pub fn nephalia_drownyard() -> CardDefinition {
    land(
        "Nephalia Drownyard",
        vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u(), b()]),
                tap_cost: true,
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
        ],
    )
}

/// Witch's Clinic — Land. {T}: Add {C}. {2}, {T}: Target commander gains
/// lifelink until end of turn.
///
/// Approximation: "target commander" is "target legendary creature".
pub fn witchs_clinic() -> CardDefinition {
    land(
        "Witch's Clinic",
        vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary))),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
    )
}

/// The Black Gate — Legendary Land — Gate. As The Black Gate enters, you may pay
/// 3 life. If you don't, it enters tapped. {T}: Add {B}. {1}{B}, {T}: Choose a
/// player with the most life or tied for most life. Target creature can't be
/// blocked by creatures that player controls this turn.
///
/// Approximation: while an opponent has the most life (or is tied for it), the
/// target can't be blocked at all this turn.
pub fn the_black_gate() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { land_types: vec![LandType::Gate], ..Default::default() },
        triggered_abilities: vec![pay_three_or_tapped()],
        ..land(
            "The Black Gate",
            vec![
                tap_add(Color::Black),
                ActivatedAbility {
                    mana_cost: cost(&[generic(1), b()]),
                    tap_cost: true,
                    effect: Effect::If {
                        cond: Predicate::PlayerHasMostLife { who: PlayerRef::EachOpponent },
                        then: Box::new(Effect::GrantKeyword {
                            what: target_filtered(R::Creature),
                            keyword: Keyword::Unblockable,
                            duration: Duration::EndOfTurn,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                    ..Default::default()
                },
            ],
        )
    }
}

/// Minas Morgul, Dark Fortress — Legendary Land. Minas Morgul enters tapped.
/// {T}: Add {B}. {3}{B}, {T}: Put a shadow counter on target creature. For as
/// long as that creature has a shadow counter on it, it's a Wraith in addition
/// to its other types.
///
/// The shadow counter is a CR 122.1b keyword counter. Approximation: the Wraith
/// type is added indefinitely rather than for as long as the counter stays.
pub fn minas_morgul_dark_fortress() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![enters_tapped()],
        ..land(
            "Minas Morgul, Dark Fortress",
            vec![
                tap_add(Color::Black),
                ActivatedAbility {
                    mana_cost: cost(&[generic(3), b()]),
                    tap_cost: true,
                    effect: Effect::Seq(vec![
                        Effect::AddKeywordCounter {
                            what: target_filtered(R::Creature),
                            keyword: Keyword::Shadow,
                            amount: Value::ONE,
                        },
                        Effect::AddCreatureTypes {
                            what: Selector::Target(0),
                            creature_types: vec![CreatureType::Wraith],
                            duration: Duration::Permanent,
                        },
                    ]),
                    ..Default::default()
                },
            ],
        )
    }
}
