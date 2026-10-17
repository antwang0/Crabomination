//! Commander: the 2026-10-17 (`01Htv6cF`) one-card-gap closers — the single
//! missing card of an unseated deck, each written against an existing
//! primitive. Tests in `tests/recent_b/cmdr_oct17_h.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, ConditionalEquipBonus, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bolster, etb, fabricate, investigate, landfall, on_attack, raid_etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};
use crate::sets::{enters_tapped, tap_add};
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

fn kw(keywords: Vec<Keyword>, def: CardDefinition) -> CardDefinition {
    CardDefinition { keywords, ..def }
}

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
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

fn act(mana: ManaCost, tap: bool, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, tap_cost: tap, effect, ..Default::default() }
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn mine(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn count_mine(filter: R) -> Value {
    Value::count(mine(filter))
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    })
}

fn mint(n: i32, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: def }
}

fn bounce(what: Selector) -> Effect {
    Effect::Move { what, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))) }
}

// ── Lands ──────────────────────────────────────────────────────────────────

/// The Karlov dual cyclers: enters tapped; {T}: either color; {2}{A}{B}, {T},
/// sacrifice it: draw a card.
fn sac_draw_dual(name: &'static str, a: Color, b2: Color) -> CardDefinition {
    let pip = |c: Color| crate::mana::ManaSymbol::Colored(c);
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(a),
            tap_add(b2),
            ActivatedAbility { mana_cost: cost(&[generic(2), pip(a), pip(b2)]), tap_cost: true, sac_cost: true, effect: draw(1), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Waterfront District — U/B.
pub fn waterfront_district() -> CardDefinition {
    sac_draw_dual("Waterfront District", Color::Blue, Color::Black)
}
/// Skybridge Towers — W/U.
pub fn skybridge_towers() -> CardDefinition {
    sac_draw_dual("Skybridge Towers", Color::White, Color::Blue)
}
/// Tramway Station — B/R.
pub fn tramway_station() -> CardDefinition {
    sac_draw_dual("Tramway Station", Color::Black, Color::Red)
}
/// Botanical Plaza — G/W.
pub fn botanical_plaza() -> CardDefinition {
    sac_draw_dual("Botanical Plaza", Color::Green, Color::White)
}

/// Lounge — enters tapped; {T}: {B} or {G}; {4}, {T}: investigate.
pub fn lounge() -> CardDefinition {
    CardDefinition {
        name: "Lounge",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_add(Color::Black), tap_add(Color::Green), act(cost(&[generic(4)]), true, investigate(1))],
        ..Default::default()
    }
}

// ── Creatures ──────────────────────────────────────────────────────────────

/// Copper Host Crusher — {6}{G}{G} 8/8 trample, hexproof Phyrexian Bear Rhino.
pub fn copper_host_crusher() -> CardDefinition {
    kw(
        vec![Keyword::Trample, Keyword::Hexproof],
        creature(
            "Copper Host Crusher",
            cost(&[generic(6), g(), g()]),
            vec![CreatureType::Phyrexian, CreatureType::Bear, CreatureType::Rhino],
            8,
            8,
        ),
    )
}

/// Feisty Spikeling — {1}{R/W} 2/1 changeling Shapeshifter; first strike
/// during your turn.
pub fn feisty_spikeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordIf { keyword: Keyword::FirstStrike, condition: Predicate::IsTurnOf(PlayerRef::You) },
        }],
        ..creature(
            "Feisty Spikeling",
            cost(&[generic(1), crate::mana::hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Shapeshifter],
            2,
            1,
        )
    }
}

/// "Whenever another creature you control leaves the battlefield, put a +1/+1
/// counter on this creature."
fn grows_on_leave() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::AnotherOfYours)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
    }
}

/// Flaming Fist Officer — {2}{W} 2/2 Gnome Soldier; grows when another
/// creature you control leaves.
pub fn flaming_fist_officer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![grows_on_leave()],
        ..creature("Flaming Fist Officer", cost(&[generic(2), w()]), vec![CreatureType::Gnome, CreatureType::Soldier], 2, 2)
    }
}

/// Ninth Bridge Patrol — {1}{W} 1/1 Dwarf Soldier; same ability.
pub fn ninth_bridge_patrol() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![grows_on_leave()],
        ..creature("Ninth Bridge Patrol", cost(&[generic(1), w()]), vec![CreatureType::Dwarf, CreatureType::Soldier], 1, 1)
    }
}

/// Cunning Survivor — {1}{U} 1/3 Human Warrior. Whenever you cycle or discard
/// a card, +1/+0 and can't be blocked this turn.
pub fn cunning_survivor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::Seq(vec![pump(Selector::This, 1, 0), grant(Selector::This, Keyword::Unblockable)]),
        }],
        ..creature("Cunning Survivor", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 3)
    }
}

/// Living Library — {2} 0/4 artifact Book Illusion. {6}, sacrifice it: target
/// creature or planeswalker an opponent controls is shuffled into its owner's
/// library.
pub fn living_library() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
                to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Shuffled },
            },
            ..Default::default()
        }],
        ..artifact_creature("Living Library", cost(&[generic(2)]), vec![CreatureType::Book, CreatureType::Illusion], 0, 4)
    }
}

/// Dundoolin Weaver — {1}{G} 2/1 Kithkin Druid. ETB, if you control three or
/// more creatures: return target permanent card from your graveyard to your
/// hand.
pub fn dundoolin_weaver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SelectorCountAtLeast { sel: mine(R::Creature), n: Value::Const(3) }),
            effect: Effect::Move { what: target_filtered(R::Permanent.from_your_graveyard()), to: ZoneDest::Hand(PlayerRef::You) },
        }],
        ..creature("Dundoolin Weaver", cost(&[generic(1), g()]), vec![CreatureType::Kithkin, CreatureType::Druid], 2, 1)
    }
}

/// The Howling Commandos — {1}{W} legendary 3/1 Human Soldier Hero. {5}:
/// creatures you control get +1/+1 and Soldiers you control gain vigilance
/// until end of turn.
pub fn the_howling_commandos() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![act(
            cost(&[generic(5)]),
            false,
            Effect::Seq(vec![
                pump(mine(R::Creature), 1, 1),
                grant(mine(R::Creature.and(R::HasCreatureType(CreatureType::Soldier))), Keyword::Vigilance),
            ]),
        )],
        ..creature(
            "The Howling Commandos",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            1,
        )
    }
}

/// High-Flying Ace — {2}{W} 2/3 flying Bird Mutant. {3}{W}: target creature
/// without flying gains flying until end of turn; sorcery speed.
pub fn high_flying_ace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            sorcery_speed: true,
            effect: grant(target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying).negate())), Keyword::Flying),
            ..Default::default()
        }],
        ..creature("High-Flying Ace", cost(&[generic(2), w()]), vec![CreatureType::Bird, CreatureType::Mutant], 2, 3)
    }
}

/// Storm Fleet Pyromancer — {4}{R} 3/2 Human Pirate Wizard. Raid — ETB, if
/// you attacked this turn: 2 damage to any target.
pub fn storm_fleet_pyromancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![raid_etb(Effect::DealDamage { to: target_any(), amount: Value::Const(2) })],
        ..creature(
            "Storm Fleet Pyromancer",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Pirate, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Maulfist Squad — {3}{B} 3/1 menace Human Artificer. Fabricate 1.
pub fn maulfist_squad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![fabricate(1)],
        ..creature("Maulfist Squad", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Artificer], 3, 1)
    }
}

/// Mirkwood Meditator — {2}{U} 2/4 Elf Druid. Landfall — you may have its
/// base power and toughness become 4/2 until end of turn.
pub fn mirkwood_meditator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![landfall(Effect::MayDo {
            description: "Have Mirkwood Meditator become 4/2 until end of turn?".into(),
            body: Box::new(Effect::SetBasePT {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..creature("Mirkwood Meditator", cost(&[generic(2), u()]), vec![CreatureType::Elf, CreatureType::Druid], 2, 4)
    }
}

/// Keldon Warlord — {2}{R}{R} */* Human Barbarian: non-Wall creatures you
/// control.
pub fn keldon_warlord() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::Creature.and(R::HasCreatureType(CreatureType::Wall).negate())),
        }),
        ..creature("Keldon Warlord", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Barbarian], 0, 0)
    }
}

/// Serpent of the Endless Sea — {4}{U} */* Serpent: Islands you control.
/// Can't attack unless defending player controls an Island.
pub fn serpent_of_the_endless_sea() -> CardDefinition {
    let island = || R::HasLandType(LandType::Island);
    CardDefinition {
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(island()))],
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching { base_p: 0, base_t: 0, filter: Box::new(island()) }),
        ..creature("Serpent of the Endless Sea", cost(&[generic(4), u()]), vec![CreatureType::Serpent], 0, 0)
    }
}

/// Afterthought Sentry — {2} 2/2 artifact Gargoyle. {2}: flying until end of
/// turn. Attacks: exile up to one target card from a graveyard.
pub fn afterthought_sentry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![act(cost(&[generic(2)]), false, grant(Selector::This, Keyword::Flying))],
        triggered_abilities: vec![on_attack(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Any.from_any_graveyard(),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
        })],
        ..artifact_creature("Afterthought Sentry", cost(&[generic(2)]), vec![CreatureType::Gargoyle], 2, 2)
    }
}

/// Roving Keep — {7} 5/7 artifact Wall, defender. {7}: +2/+0 and trample
/// until end of turn; it can attack this turn as though it lacked defender.
pub fn roving_keep() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![act(
            cost(&[generic(7)]),
            false,
            Effect::Seq(vec![
                pump(Selector::This, 2, 0),
                grant(Selector::This, Keyword::Trample),
                Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
            ]),
        )],
        ..artifact_creature("Roving Keep", cost(&[generic(7)]), vec![CreatureType::Wall], 5, 7)
    }
}

/// Great Desert Prospector — {4}{W} 3/2 Human Artificer. ETB: a tapped
/// Powerstone token for each other creature you control.
pub fn great_desert_prospector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: count_mine(R::Creature.and(R::OtherThanSource)),
            definition: Arc::new(TokenDefinition { tapped: true, ..crabomination_base::tokens::powerstone_token() }),
        })],
        ..creature("Great Desert Prospector", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Artificer], 3, 2)
    }
}

// ── Artifacts and Auras ───────────────────────────────────────────────────

/// Ring of Three Wishes — {5} Artifact; enters with three wish counters. {5},
/// {T}, remove a wish counter: search your library for a card.
pub fn ring_of_three_wishes() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::Wish, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Wish, 1)),
            effect: Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..artifact("Ring of Three Wishes", cost(&[generic(5)]))
    }
}

/// Ranger's Longbow — {1}{G} Equipment, equip {3}. +2/+1 and reach.
pub fn rangers_longbow() -> CardDefinition {
    equipment(
        "Ranger's Longbow",
        cost(&[generic(1), g()]),
        cost(&[generic(3)]),
        EquipBonus { power: 2, toughness: 1, keywords: vec![Keyword::Reach], ..Default::default() },
    )
}

/// Amorphous Axe — {2} Equipment, equip {3}. +3/+0 and every creature type.
pub fn amorphous_axe() -> CardDefinition {
    equipment(
        "Amorphous Axe",
        cost(&[generic(2)]),
        cost(&[generic(3)]),
        EquipBonus { power: 3, keywords: vec![Keyword::Changeling], ..Default::default() },
    )
}

/// Glaring Aegis — {W} Aura. ETB: tap target creature an opponent controls.
/// +1/+3.
pub fn glaring_aegis() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) })],
        ..aura("Glaring Aegis", cost(&[w()]), EquipBonus { power: 1, toughness: 3, ..Default::default() })
    }
}

/// Rousing Read — {2}{U} Aura. ETB: draw two, then discard one. +1/+1 and
/// flying.
pub fn rousing_read() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            draw(2),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]))],
        ..aura("Rousing Read", cost(&[generic(2), u()]), EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Flying], ..Default::default() })
    }
}

/// Serra's Boon — {2}{W} Aura. +1/+2 while the creature is white, otherwise
/// -2/-1.
pub fn serras_boon() -> CardDefinition {
    let white = || R::HasColor(Color::White);
    aura(
        "Serra's Boon",
        cost(&[generic(2), w()]),
        EquipBonus {
            conditional: vec![
                ConditionalEquipBonus { host_filter: white(), power: 1, toughness: 2, ..Default::default() },
                ConditionalEquipBonus { host_filter: white().negate(), power: -2, toughness: -1, ..Default::default() },
            ],
            ..Default::default()
        },
    )
}

/// Consulate Crackdown — {3}{W}{W} Enchantment. ETB: exile all artifacts your
/// opponents control until it leaves the battlefield.
pub fn consulate_crackdown() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..enchantment("Consulate Crackdown", cost(&[generic(3), w(), w()]))
    }
}

// ── Spells ─────────────────────────────────────────────────────────────────

/// Dromoka's Gift — {4}{G} Instant. Bolster 4.
pub fn dromokas_gift() -> CardDefinition {
    spell("Dromoka's Gift", cost(&[generic(4), g()]), CardType::Instant, bolster(4))
}

/// Anchor to the Aether — {2}{U} Sorcery. Target creature on top of its
/// owner's library; scry 1.
pub fn anchor_to_the_aether() -> CardDefinition {
    spell(
        "Anchor to the Aether",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Top },
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
    )
}

/// Vanishment — {4}{U} Instant. Target nonland permanent on top of its
/// owner's library. Miracle {U}.
pub fn vanishment() -> CardDefinition {
    CardDefinition {
        miracle: Some(cost(&[u()])),
        ..spell(
            "Vanishment",
            cost(&[generic(4), u()]),
            CardType::Instant,
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland)),
                to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))), pos: LibraryPosition::Top },
            },
        )
    }
}

/// Confidence from Strength — {2}{G} Sorcery. +4/+4 and trample.
pub fn confidence_from_strength() -> CardDefinition {
    spell(
        "Confidence from Strength",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![pump(target_filtered(R::Creature), 4, 4), grant(Selector::Target(0), Keyword::Trample)]),
    )
}

/// I Am Iron Man — {2}{U} Instant. Target artifact or creature becomes a 4/4
/// artifact creature with flying until end of turn; draw a card.
pub fn i_am_iron_man() -> CardDefinition {
    spell(
        "I Am Iron Man",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::BecomeCreature {
                what: target_filtered(R::Artifact.or(R::Creature)),
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![],
                keywords: vec![Keyword::Flying],
                duration: Duration::EndOfTurn,
            },
            // CR 205.1b — it becomes an artifact in addition to its types.
            Effect::AddCardTypeIndefinitely { what: Selector::Target(0), card_type: CardType::Artifact, until_eot: true },
            draw(1),
        ]),
    )
}

/// Machine Over Matter — {1}{U} Instant; {1} less if you control an artifact
/// creature. Return target nonland permanent to its owner's hand.
pub fn machine_over_matter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast if you control an artifact creature.",
            effect: StaticEffect::SelfCostReducedIfPredicate {
                amount: 1,
                condition: Predicate::SelectorCountAtLeast { sel: mine(R::Artifact.and(R::Creature)), n: Value::ONE },
            },
        }],
        ..spell("Machine Over Matter", cost(&[generic(1), u()]), CardType::Instant, bounce(target_filtered(R::Permanent.and(R::Nonland))))
    }
}

/// Undercity Uprising — {2}{B}{G} Sorcery. Your creatures gain deathtouch
/// until end of turn, then target creature you control fights target creature
/// you don't control.
pub fn undercity_uprising() -> CardDefinition {
    spell(
        "Undercity Uprising",
        cost(&[generic(2), b(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            grant(mine(R::Creature), Keyword::Deathtouch),
            Effect::Fight {
                attacker: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
            },
        ]),
    )
}

/// Unwilling Recruit — {X}{R}{R}{R} Sorcery. Gain control of target creature
/// until end of turn, untap it; +X/+0 and haste until end of turn.
pub fn unwilling_recruit() -> CardDefinition {
    spell(
        "Unwilling Recruit",
        cost(&[x(), r(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::PumpPT { what: Selector::Target(0), power: Value::XFromCost, toughness: Value::ZERO, duration: Duration::EndOfTurn },
            grant(Selector::Target(0), Keyword::Haste),
        ]),
    )
}

/// Broadcast Takeover — {2}{R}{R}{R} Sorcery. Gain control of all artifacts
/// your opponents control until end of turn, untap them; they gain haste.
pub fn broadcast_takeover() -> CardDefinition {
    let theirs = || Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent));
    let stolen = || Selector::EachPermanent(R::Artifact.and(R::ControlledByYou));
    spell(
        "Broadcast Takeover",
        cost(&[generic(2), r(), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GainControl { what: theirs(), to: Some(PlayerRef::You), duration: Duration::EndOfTurn },
            Effect::Untap { what: stolen(), up_to: None },
            grant(stolen(), Keyword::Haste),
        ]),
    )
}

/// You're Confronted by Robbers — {3}{W} Instant. Choose one — tap up to three
/// target creatures; or three 1/1 white Soldiers.
pub fn youre_confronted_by_robbers() -> CardDefinition {
    spell(
        "You're Confronted by Robbers",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 3,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::Target(0) }),
            },
            mint(3, token("Soldier", 1, 1, vec![Color::White], vec![CreatureType::Soldier], vec![])),
        ]),
    )
}

/// Break Under Pressure — {2}{B} Instant. Target opponent sacrifices a
/// creature or planeswalker with the greatest mana value among those they
/// control; you gain 2 life.
pub fn break_under_pressure() -> CardDefinition {
    let big = || R::Creature.or(R::Planeswalker);
    spell(
        "Break Under Pressure",
        cost(&[generic(2), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Sacrifice {
                who: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: big().and(R::HasGreatestManaValueAmongControlled(Box::new(big()))),
            },
            gain(2),
        ]),
    )
}
