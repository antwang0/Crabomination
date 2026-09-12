//! Whole-catalog structural audit as a test: no shipped card may carry a
//! triggered / activated / loyalty ability whose effect resolves to nothing.
//!
//! Dead abilities are real bugs — the engine puts an empty object on the
//! stack and burns a priority round for an ability real Magic doesn't have.
//! Three shipped cards had one (Magosi and Oran-Rief via a `tapped_etb_land`
//! helper that emitted `etb(Noop)`; Annie Joins Up via a filler
//! `legend_enters(Noop)`), and nothing caught them until someone re-ran
//! `audit_incomplete` by hand. This is that auditor's pass 1, wired into the
//! suite so the class can't come back silently.
//!
//! Dead *modes* are gated too, but against an allowlist
//! ([`crabomination::audit::REVIEWED_DEAD_MODES`]) rather than outright: a
//! `Noop` arm is also the idiom for "you may … (or decline)", so it needs a
//! reviewer's judgement once — and then it needs to stop costing that
//! judgement every run. Before the allowlist the auditor reported exactly one
//! card forever, so the only signal a *new* dead mode gave was a count going
//! from 1 to 2 in a report nobody diffs.

use crabomination::audit::{DeadCapability, dead_capabilities};
use crabomination::catalog::all_known_factories;
use std::collections::HashSet;
use crabomination::game::*;

#[test]
fn no_shipped_card_has_a_dead_ability() {
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        for f in dead_capabilities(&def) {
            if matches!(f, DeadCapability::Ability { .. }) {
                bad.push(format!("{}: {f}", def.name));
            }
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} card(s) ship an ability that resolves to nothing — either give it \
         its printed effect or stop emitting the ability:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// The mode half, gated against the reviewed list — in both directions.
///
/// Forwards: a card that grows a dead mode fails here by name, so the
/// reviewer decides once instead of the auditor asking forever. Backwards: an
/// entry whose card no longer *has* a dead mode fails too. Without that, a
/// list entry outlives the thing it excused and silently licenses the next
/// dead mode on the same card — an allowlist that cannot go stale is the
/// only kind worth having.
#[test]
fn every_dead_mode_is_one_a_reviewer_signed_off() {
    use crabomination::audit::REVIEWED_DEAD_MODES;

    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut found: Vec<(&'static str, String)> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        for f in dead_capabilities(&def) {
            if matches!(f, DeadCapability::Mode { .. }) {
                found.push((def.name, f.to_string()));
            }
        }
    }
    found.sort();

    let reviewed: HashSet<&str> = REVIEWED_DEAD_MODES.iter().map(|(n, _)| *n).collect();
    let unreviewed: Vec<String> = found
        .iter()
        .filter(|(name, _)| !reviewed.contains(name))
        .map(|(name, f)| format!("{name}: {f}"))
        .collect();
    assert!(
        unreviewed.is_empty(),
        "{} card(s) have a mode that resolves to nothing and nobody has said \
         which kind it is. If the arm is a missing primitive, implement it. If \
         it is the printed \"you may … (or decline)\", add the card to \
         `crabomination::audit::REVIEWED_DEAD_MODES` with the printed text \
         that makes the empty arm correct:\n  {}",
        unreviewed.len(),
        unreviewed.join("\n  "),
    );

    let live: HashSet<&str> = found.iter().map(|(name, _)| *name).collect();
    let stale: Vec<&str> =
        REVIEWED_DEAD_MODES.iter().map(|(n, _)| *n).filter(|n| !live.contains(n)).collect();
    assert!(
        stale.is_empty(),
        "{} entr(ies) in `REVIEWED_DEAD_MODES` name a card with no dead mode \
         any more — the arm was implemented, or the card was renamed or \
         dropped. Remove them, or they will excuse the next dead mode on that \
         card:\n  {}",
        stale.len(),
        stale.join("\n  "),
    );
}

/// The carve-out the audit relies on: an activated ability whose cost moves
/// the source is complete with an empty resolution effect. Circling Vultures
/// ("You may discard this any time you could cast an instant") is the live
/// example — pin it so a future tightening of `ability_is_cost_only` fails
/// here rather than turning the test above into a false alarm.
#[test]
fn cost_only_abilities_are_not_dead() {
    let vultures = crabomination::catalog::circling_vultures();
    assert!(
        vultures.activated_abilities.iter().any(|a| a.discard_self_cost),
        "Circling Vultures still has its discard-self ability",
    );
    assert_eq!(dead_capabilities(&vultures), Vec::new());
}

/// The two helpers that used to emit a filler trigger now emit none.
#[test]
fn tapped_etb_lands_without_an_etb_effect_have_no_trigger() {
    for def in [
        crabomination::catalog::magosi_the_waterveil(),
        crabomination::catalog::oran_rief_the_vastwood(),
        crabomination::catalog::annie_joins_up(),
    ] {
        assert!(
            def.triggered_abilities.iter().all(|t| !matches!(
                serde_json::to_value(&t.effect).as_ref(),
                Ok(serde_json::Value::String(s)) if s == "Noop"
            )),
            "{} still carries an empty triggered ability",
            def.name,
        );
    }
    // Annie keeps her real ETB (5 damage) and loses only the filler.
    assert_eq!(crabomination::catalog::annie_joins_up().triggered_abilities.len(), 1);
    assert!(crabomination::catalog::magosi_the_waterveil().triggered_abilities.is_empty());
    assert!(crabomination::catalog::oran_rief_the_vastwood().triggered_abilities.is_empty());
}

/// `grant_bits::ANY_GRANT` is `CardDefinition::can_grant_keyword` at
/// `|_| true`, and `card_can_grant_keyword` returns `false` outright when it
/// is clear. That is sound only because the function is **monotone in its
/// predicate** — every leg is an `any` / `||` / a recursion, with no negation
/// — so this walks the whole catalog against a battery of the keywords real
/// callers ask about and pins the implication.
///
/// The battery is a sample; the argument is the proof. What the test catches
/// is a later edit that puts a negation in one of the legs, which would make
/// the bit stop being an over-approximation of the predicate form and start
/// hiding grants.
#[test]
fn a_clear_any_grant_bit_is_authoritative_for_every_predicate() {
    use crabomination::card::{Keyword, grant_bits};
    use crabomination::mana::Color;

    /// One named predicate of the battery — a `type` because the tuple is
    /// past clippy's complexity floor inline.
    type NamedPred<'a> = (&'a str, Box<dyn Fn(&Keyword) -> bool>);

    let battery: Vec<NamedPred<'_>> = vec![
        ("flying", Box::new(|k: &Keyword| matches!(k, Keyword::Flying))),
        ("haste", Box::new(|k: &Keyword| matches!(k, Keyword::Haste))),
        ("hexproof", Box::new(|k: &Keyword| matches!(k, Keyword::Hexproof))),
        ("menace", Box::new(|k: &Keyword| matches!(k, Keyword::Menace))),
        ("cant_block", Box::new(|k: &Keyword| matches!(k, Keyword::CantBlock))),
        ("trample", Box::new(|k: &Keyword| matches!(k, Keyword::Trample))),
        ("absorb", Box::new(|k: &Keyword| matches!(k, Keyword::Absorb(_)))),
        ("protection", Box::new(|k: &Keyword| matches!(k, Keyword::Protection(_)))),
        ("protection_white", Box::new(|k: &Keyword| matches!(k, Keyword::Protection(Color::White)))),
        ("cumulative_upkeep", Box::new(|k: &Keyword| matches!(k, Keyword::CumulativeUpkeep(_)))),
        ("must_attack", Box::new(|k: &Keyword| matches!(k, Keyword::MustAttack))),
        ("phasing", Box::new(|k: &Keyword| matches!(k, Keyword::Phasing))),
        ("any", Box::new(|_: &Keyword| true)),
    ];

    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    let (mut with_bit, mut total) = (0usize, 0usize);
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        total += 1;
        let set = def.grant_scan_bits() & grant_bits::ANY_GRANT != 0;
        if set {
            with_bit += 1;
            continue;
        }
        for (name, pred) in &battery {
            if def.can_grant_keyword(pred) {
                bad.push(format!("{}: ANY_GRANT clear but grants {name}", def.name));
            }
        }
    }
    assert!(bad.is_empty(), "the gate is not an over-approximation:\n{}", bad.join("\n"));
    // Not vacuous in either direction: the bit is set on a real population and
    // clear on a much larger one, which is the whole reason it is a gate.
    assert!(with_bit > 100, "only {with_bit} of {total} definitions can grant a keyword");
    assert!(total - with_bit > 1_000, "the gate excludes almost nothing");
}

/// CR 605.1a — an activated ability that **could add mana**, doesn't target
/// and isn't a loyalty ability *is* a mana ability, whatever else it does
/// alongside. `is_mana_ability` is that rule verbatim now, so what this
/// audits is the *catalog* side of it: a shipped ability that adds mana and
/// targets.
///
/// **Why the pair is worth a whole-catalog test.** The rule used to be an
/// allowlist of permitted riders, and a rider nobody had thought of turned
/// the whole ability into a stack ability — silently. The mana then lands
/// after the cost it was tapped for is being paid, so
/// `effective_mana_abilities_into` never offers the source and the bot never
/// taps it. 83 abilities across 55 cards were on the wrong side: every
/// painland, City of Brass, Ancient Tomb, the ten Talismans, Wall of Roots,
/// Gemstone Mine, Krark-Clan Ironworks.
///
/// The three survivors target, and targeting is CR 605.1a's own first
/// criterion. They are listed rather than excused in bulk, in both
/// directions, for [`every_dead_mode_is_one_a_reviewer_signed_off`]'s reason:
/// an entry whose card stops targeting is an entry that would go on
/// licensing the next one.
#[test]
fn every_ability_that_could_add_mana_is_a_mana_ability() {
    use crabomination::game::actions::{effect_could_add_mana, is_mana_ability_public};
    /// Mana-adding abilities that legitimately are NOT mana abilities,
    /// because they target (CR 605.1a). Deathrite Shaman exiles target land
    /// card from a graveyard; Priest of Forgotten Gods makes target players
    /// lose life; Witch Engine folds its "target opponent gains control"
    /// trigger onto the ability. Radiant Lotus and Spectral Searchlight add
    /// the mana to a *target player's* pool, which is the textbook example
    /// of a mana-adding ability that is not a mana ability.
    const REVIEWED_TARGETING_MANA: &[&str] = &[
        "Deathrite Shaman",
        "Priest of Forgotten Gods",
        "Radiant Lotus",
        "Spectral Searchlight",
        "Witch Engine",
    ];

    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    let mut hit: HashSet<&'static str> = HashSet::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        for (i, a) in def.activated_abilities.iter().enumerate() {
            if !effect_could_add_mana(&a.effect) || is_mana_ability_public(&a.effect) {
                continue;
            }
            if REVIEWED_TARGETING_MANA.contains(&def.name) {
                hit.insert(def.name);
                continue;
            }
            bad.push(format!("{} [ability {i}]", def.name));
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} ability/ies could add mana but are not mana abilities. Under CR \
         605.1a the only reason for that is a target, so either the card's \
         target is a modelling error or it belongs on \
         REVIEWED_TARGETING_MANA:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
    let stale: Vec<&str> =
        REVIEWED_TARGETING_MANA.iter().copied().filter(|n| !hit.contains(n)).collect();
    assert!(
        stale.is_empty(),
        "REVIEWED_TARGETING_MANA names {} card(s) whose mana ability no longer \
         targets — drop the entry rather than let it excuse the next one: {:?}",
        stale.len(),
        stale,
    );
}

/// Every `env::var` on a simulator path is read once through a `OnceLock`.
///
/// PERF `(-169)`: `GameState::adjust_life` asked `env::var_os` **per life
/// adjustment** for a flag nobody sets. `getenv` is a linear scan of the
/// environment block that `strncmp`s every entry, so 4,478 lookups a six-game
/// `fixed` run cost 13.3 M Ir — **1.50 % of the pool**, 1.64 % of `sealed` —
/// and the site's own doc comment had priced it at "one env lookup per life
/// change and nothing else". A count nobody took is how that survived.
///
/// The engine's `game/`, `recommend.rs` and `server/bot.rs` are the simulator;
/// a lookup there must be behind a process-lifetime cache. `server/`'s
/// per-connection reads and the `trig-census` feature's are not, and are not
/// checked here.
#[test]
fn no_bare_env_lookup_on_a_simulator_path() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).expect("engine source dir") {
            let p = e.expect("dir entry").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../crabomination/src");
    let mut files = vec![src.join("recommend.rs"), src.join("server/bot.rs")];
    walk(&src.join("game"), &mut files);

    let mut bad: Vec<String> = Vec::new();
    let mut seen = 0usize;
    for f in &files {
        let text = std::fs::read_to_string(f).expect("engine source file");
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if !line.contains("env::var(") && !line.contains("env::var_os(") {
                continue;
            }
            seen += 1;
            // The cache is a `OnceLock` initialised a few lines above the
            // lookup; eight lines covers every shape the crate uses.
            let ctx = lines[i.saturating_sub(8)..(i + 1).min(lines.len())].join("\n");
            if !ctx.contains("OnceLock") && !ctx.contains("get_or_init") {
                bad.push(format!("{}:{}: {}", f.display(), i + 1, line.trim()));
            }
        }
    }
    assert!(seen >= 5, "the scan found only {seen} env lookups — it has gone vacuous");
    assert!(
        bad.is_empty(),
        "{} env lookup(s) on a simulator path are not behind a process-lifetime \
         cache. `getenv` walks the whole environment block:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// The dispatcher serves `EventScope::EnchantedBySource` for a fixed set of
/// events (a death / exile / damage / attack / block / tap / face-up / draw /
/// targeting of the host); any other kind under that scope is a trigger
/// that can never fire. Paroxysm and Numbing Dose shipped an upkeep trigger
/// that way, and Sleeping Potion, Spinal Graft and Fractured Loyalty a
/// targeting one the matcher did not know (2026-09-10). Step triggers are
/// the wider case: `fire_step_triggers` treats every event-based scope as
/// "never", so a step trigger needs `AnyPlayer` / `ActivePlayer` /
/// `YourControl` / `SelfSource` / `OpponentControl` and a filter. The other
/// event-based scopes are read the same way: a targeting scope serves
/// `BecameTarget`, a damage scope a damage kind, `YouTapped` a tap, the
/// attacked scopes an attack (`FromYourGraveyard` steps are walked apart).
#[test]
fn no_trigger_sits_under_a_scope_the_dispatcher_never_matches() {
    use crabomination::card::{EventKind, EventScope};
    let served_by_host = |kind: &EventKind| {
        let name = format!("{kind:?}");
        matches!(
            kind,
            EventKind::CreatureDied
                | EventKind::PermanentDied
                | EventKind::CreatureOrArtifactDied
                | EventKind::PermanentLeavesBattlefield
                | EventKind::CardExiled
                | EventKind::Attacks
                | EventKind::TurnedFaceUp
                | EventKind::Tapped
                | EventKind::CardDrawn
                | EventKind::BecameTarget
        ) || name.contains("Damage")
            || name.contains("Block")
    };
    let event_based = |scope: &EventScope| {
        matches!(
            scope,
            EventScope::EnchantedBySource
                | EventScope::YourPermanentTargetedByOpponent
                | EventScope::YourCreatureTargeted
                | EventScope::YourSourceDamagedOpponent
                | EventScope::OpponentSourceDamagedYou
                | EventScope::YourOtherSourceDamagedOpponent
                | EventScope::YouTapped
                | EventScope::ControllerAttackedByOpponent
                | EventScope::ControllerPlaneswalkerAttackedByOpponent
        )
    };
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        for (i, ta) in def.triggered_abilities.iter().enumerate() {
            let step = matches!(ta.event.kind, EventKind::StepBegins(_) | EventKind::TurnBegins);
            let name = format!("{:?}", ta.event.kind);
            // What each event-based scope's matcher arm serves (events.rs).
            let served = match ta.event.scope {
                EventScope::EnchantedBySource => served_by_host(&ta.event.kind),
                EventScope::YourPermanentTargetedByOpponent | EventScope::YourCreatureTargeted => {
                    ta.event.kind == EventKind::BecameTarget
                }
                EventScope::YourSourceDamagedOpponent
                | EventScope::OpponentSourceDamagedYou
                | EventScope::YourOtherSourceDamagedOpponent => name.contains("Damage"),
                EventScope::YouTapped => ta.event.kind == EventKind::Tapped,
                EventScope::ControllerAttackedByOpponent
                | EventScope::ControllerPlaneswalkerAttackedByOpponent => {
                    ta.event.kind == EventKind::Attacks
                }
                _ => true,
            };
            if step && event_based(&ta.event.scope) {
                bad.push(format!("{}: trigger {i} is a step trigger under {:?}", def.name, ta.event.scope));
            } else if !served {
                bad.push(format!("{}: trigger {i} on {:?} under {:?}", def.name, ta.event.kind, ta.event.scope));
            }
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} trigger(s) sit under a scope the dispatcher never matches for their event:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// An Aura's own `effect:` is its attach; the cast path runs nothing after
/// it. Six shipped Auras spelled an entry effect as `Seq([Attach, ..])` and
/// their tails (a draw, a fight, an exile, a control change, X sleep
/// counters) never ran in a game — their tests resolved the effect by hand
/// (2026-09-10). The entry half is an ETB trigger (`etb(..)`).
#[test]
fn no_aura_spells_an_entry_effect_after_its_attach() {
    use crabomination::card::EnchantmentSubtype;
    use crabomination::effect::{Effect, Selector};
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut bad: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        if !def.subtypes.enchantment_subtypes.contains(&EnchantmentSubtype::Aura) {
            continue;
        }
        if let Effect::Seq(steps) = &def.effect
            && steps.len() > 1
            && matches!(steps.first(), Some(Effect::Attach { what: Selector::This, .. }))
        {
            bad.push(def.name.to_string());
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} Aura(s) spell an entry effect after their attach, which the cast path never runs — \
         move it to an ETB trigger:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}
/// The resolution answer log is ONE channel per resolution, and a nested
/// asking arm does not get a fresh one: `run_effect` recursion keeps the same
/// `resolution_depth`, so an inner arm's `cursor = 0` replays whatever the outer
/// arm logged, and `ask_seat_bool` accepts any answer *kind*. That is the half
/// the resolution-exit net cannot cover (ENGINE_BACKLOG, "the tenth find's
/// production half"), so the gate is structural: no shipped card may nest one
/// log-using effect inside another.
///
/// The seven allowlisted below are a HAZARD, not a live defect — a claim now
/// tested rather than assumed, on the suspending path, for the three shapes
/// that differ (`conspiracy_theorist_nested_asks_pay_once_when_both_suspend`,
/// `forbidden_ritual_nested_asks_each_take_their_own_answer_when_both_suspend`,
/// `giant_albatross_charges_each_creature_once_when_the_asks_suspend`). Two
/// properties hold for all five outer arms, and both are needed: each clears
/// the channel before running its body, and none has any work left after it, so
/// a suspend inside the body re-enters at the INNER arm with only the inner
/// arm's own answers in the log. A new nesting whose outer arm keeps working
/// after its body — or asks again after it, the `MayPayRepeatedly` shape —
/// breaks both, which is what this gate exists to catch.
///
/// The variant list is the 57 `Effect` arms that ask inline plus the 8 that reach
/// an asking helper (65 in all, from 63 `let mut cursor = 0` blocks) — regenerate
/// it with `scripts/audit_answer_log.py --variants`.
#[test]
fn no_shipped_card_nests_two_answer_log_arms() {
    use serde_json::Value;

    const LOG_ARMS: &[&str] = &[
        "AnteTopOfLibrary",
        "AnyPlayerMayAccept",
        "AnyPlayerMayExileFromGraveyard",
        "BidLifeToCounterTargetSpell",
        "ClashWithOpponent",
        "CoinFlipDestroyLoop",
        "CounterOnMatchingOfEachColor",
        "CounterUnless",
        "CounterUnlessPaid",
        "CumulativeUpkeepPayOrSacrifice",
        "DamageTargetPlayerMayRedirect",
        "DestroyEachUnlessPaysLife",
        "DiscardUnlessPutCardOnTop",
        "EachPlayerChoosesNumberHighestLoses",
        "EachPlayerDrawsUpToElseGainsLife",
        "EachPlayerMayExileAnyNumberFromGraveyard",
        "EchoPayOrSacrifice",
        "ExileHandThenReclaimLinked",
        "ExileTokensCreatedBySourceForCounters",
        "Forage",
        "GoblinGame",
        "GuessColorCountInHand",
        "Learn",
        "LifeBidding",
        "LookTopMayBottomAllElse",
        "MayDealPowerThenNoCombatDamage",
        "MayDiscard",
        "MayDiscardMatching",
        "MayDoBy",
        "MayExileSelfReturnNextUpkeepHaste",
        "MayExileSelfThen",
        "MayPay",
        "MayPayBy",
        "MayPayLife",
        "MayPayRepeatedly",
        "MaySacrifice",
        "MaySacrificeSource",
        "MayTap",
        "MoveChosenKeyword",
        "OtherPlayerMayPayToCounter",
        "PlayerMayPayLifeElse",
        "PlayerReturnsPermanentUnlessPaysLife",
        "PlayersMayAccept",
        "Process",
        "RemoveCountersToCreateTokens",
        "ReturnEachUnlessPays",
        "ReturnFromGraveyardOpponentChooses",
        "RevealHandDiscardMatchingUnlessPayLife",
        "RevealTopAndDrawIf",
        "RevealTopMayPutOntoBattlefield",
        "RevealTopOpponentChoosesToHand",
        "RevealTopPayOrTake",
        "RevealTopToHandLoseLifeRepeat",
        "SacrificeEachUnlessPays",
        "SacrificeSourceUnlessCost",
        "SacrificeSourceUnlessPay",
        "SacrificeSourceUnlessReturn",
        "SacrificeSourceUnlessSacrifice",
        "SacrificeSourceUnlessSacrificeTotalPower",
        "SeparateIntoPiles",
        "TemptingOffer",
        "TransmuteArtifact",
        "Tribute",
        "UnlessPlayerPays",
        "Vote",
    ];

    fn walk(v: &Value, outer: Option<&str>, hits: &mut Vec<String>) {
        match v {
            Value::Object(map) => {
                for (k, inner) in map {
                    if LOG_ARMS.contains(&k.as_str()) {
                        if let Some(o) = outer {
                            hits.push(format!("{o} > {k}"));
                        }
                        walk(inner, Some(k), hits);
                    } else {
                        walk(inner, outer, hits);
                    }
                }
            }
            Value::Array(a) => {
                for x in a {
                    walk(x, outer, hits);
                }
            }
            _ => {}
        }
    }

    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut flagged: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if !seen.insert(def.name) {
            continue;
        }
        let v = serde_json::to_value(&def).expect("CardDefinition serializes");
        let mut hits = Vec::new();
        walk(&v, None, &mut hits);
        hits.sort();
        hits.dedup();
        for h in hits {
            flagged.push(format!("{}: {h}", def.name));
        }
    }
    // The seven shipped nestings. Each is safe for the reason in this test's
    // doc comment — the outer arm clears before its body and has nothing left
    // to do after it — and not one of them is safe by construction, which is
    // why they are listed one by one rather than waved through by shape. This
    // list is the allowlist, not an endorsement: a NEW nesting fails here and
    // has to prove the same two properties before it joins.
    const KNOWN: &[&str] = &[
        "Conspiracy Theorist: MayPay > MayDiscard",
        "Emberwilde Djinn: MayPayBy > MayPayLife",
        "Forbidden Ritual: MaySacrifice > UnlessPlayerPays",
        "Giant Albatross: MayPay > DestroyEachUnlessPaysLife",
        "Rottenmouth Viper: MaySacrifice > MayDiscard",
        "Skirk Drill Sergeant: MayPay > RevealTopMayPutOntoBattlefield",
        "Worms of the Earth: AnyPlayerMayAccept > AnyPlayerMayAccept",
    ];
    flagged.sort();
    let fresh: Vec<&String> = flagged.iter().filter(|f| !KNOWN.contains(&f.as_str())).collect();
    assert!(
        fresh.is_empty(),
        "{} card(s) nest one answer-log arm inside another, so the inner arm \
         replays the outer's answers (they share one channel and `ask_seat_bool` \
         reads any kind):\n  {:#?}",
        fresh.len(),
        fresh,
    );
    // And the allowlist must not rot: a fixed card has to leave it.
    let gone: Vec<&&str> = KNOWN.iter().filter(|k| !flagged.iter().any(|f| f == *k)).collect();
    assert!(gone.is_empty(), "allowlisted nesting(s) no longer exist — drop them: {gone:#?}");
}

/// CR 202.1b / 601.3e — a card that prints NO mana cost can't be cast by
/// paying one, and the engine enforces that off `CardDefinition.no_mana_cost`.
/// Seven shipped cards were missing the flag and four of them had no `cost:`
/// either, so the catalog's only free tutor, a free Mox, a free mana rock and
/// a free 7/6 were all castable from hand for nothing; Resurgent Belief went
/// the other way and shipped at an invented `{3}{W}`. `scripts/
/// audit_printed_body.py` is the oracle-backed finder (it reads the printed
/// cost and body of every literal `CardDefinition` factory); this is the
/// regression, listed by name because the suite has no Scryfall.
#[test]
fn every_card_that_prints_no_mana_cost_says_so() {
    use crabomination::catalog as c;
    type Factory = fn() -> crabomination::card::CardDefinition;
    let cards: &[(&str, Factory)] = &[
        ("Ancestral Vision", c::ancestral_vision),
        ("Asmoranomardicadaistinaculdacar", c::asmoranomardicadaistinaculdacar),
        ("Crashing Footfalls", c::crashing_footfalls),
        ("Gaea's Will", c::gaeas_will),
        ("Glimpse of Tomorrow", c::glimpse_of_tomorrow),
        ("Hypergenesis", c::hypergenesis),
        ("Inevitable Betrayal", c::inevitable_betrayal),
        ("Living End", c::living_end),
        ("Lotus Bloom", c::lotus_bloom),
        ("Mox Tantalite", c::mox_tantalite),
        ("Profane Tutor", c::profane_tutor),
        ("Ragnarok, Divine Deliverance", c::ragnarok_divine_deliverance),
        ("Restore Balance", c::restore_balance),
        ("Resurgent Belief", c::resurgent_belief),
        ("Sol Talisman", c::sol_talisman),
        ("Urza, Planeswalker", c::urza_planeswalker),
        ("Wheel of Fate", c::wheel_of_fate),
    ];
    let mut bad: Vec<String> = Vec::new();
    for (name, factory) in cards {
        let def = factory();
        assert_eq!(def.name, *name, "the factory moved off this card");
        if !def.no_mana_cost {
            bad.push(format!("{name}: castable from hand for {:?}", def.cost.summary()));
        }
    }
    assert!(bad.is_empty(), "card(s) printing no mana cost but castable:\n  {bad:#?}");
}

/// The flag is not decoration: the cast path refuses it. Profane Tutor is
/// "search your library for a card, put it into your hand" and shipped
/// castable for free.
#[test]
fn a_card_with_no_mana_cost_cannot_be_cast_from_hand() {
    let mut g = two_player_game();
    let tutor = g.add_card_to_hand(0, crabomination::catalog::profane_tutor());
    g.players[0].mana_pool.add_colorless(6);
    let err = g.perform_action(GameAction::CastSpell {
        card_id: tutor,
        target: None,
        additional_targets: vec![],
        mode: None,
        x_value: None,
    });
    assert!(
        matches!(err, Err(GameError::NoManaCost)),
        "a card with no printed mana cost is not castable from hand: {err:?}",
    );
}

/// Every arm `Effect::with_asked_seat` lists actually pins its seat, and the
/// table here is exactly that list.
///
/// The ask helpers queue `effect.with_asked_seat(seat)` as the continuation of
/// a suspended ask, so an arm that derives its seat from a selector is fixed by
/// *appearing in that one match* — Ghost Quarter's re-run resolved
/// `ControllerOf(Target(0))` against a board whose land its own earlier clause
/// had destroyed, returned at its `let Some(seat)` with the replayed answer
/// still in the channel, and the compensation search never happened
/// (ENGINE_BACKLOG's seventeenth find). Fourteen more arms had the same shape.
///
/// Proved by injection rather than by its own green: each variant is built
/// with a `who` that is *not* a seat, and the pinned copy must read
/// `Seat(3)` — a rewrite that silently did nothing would fail. The arm list is
/// read out of the source so this table cannot fall behind the code, which is
/// the half that matters: a sixteenth arm added to the match without a row
/// here fails on the set comparison.
/// `scripts/audit_seat_from_selector.py` asks the other question — which
/// asking arms are still *outside* the list.
#[test]
fn every_listed_arm_pins_the_seat_its_ask_was_routed_to() {
    use crabomination::card::{SelectionRequirement as R, WardCost};
    use crabomination::effect::{Effect, PlayerRef, Value, ZoneDest};
    use crabomination::mana::ManaCost;

    let any = PlayerRef::EachOpponent;
    let body = || Box::new(Effect::Noop);
    let mc = || ManaCost::new(Vec::new());
    let table: Vec<(&str, Effect)> = vec![
        ("MayDoBy", Effect::MayDoBy { who: any.clone(), description: String::new(), body: body() }),
        ("MayPayBy", Effect::MayPayBy { who: any.clone(), description: String::new(), mana_cost: mc(), body: body(), else_: None }),
        ("MayPayRepeatedly", Effect::MayPayRepeatedly { who: any.clone(), description: String::new(), mana_cost: mc(), body: body() }),
        ("PlayerMayPayLifeElse", Effect::PlayerMayPayLifeElse { who: any.clone(), life: Value::ONE, else_: body() }),
        ("TokenCopyOfOpponentChoice", Effect::TokenCopyOfOpponentChoice { who: any.clone() }),
        ("Learn", Effect::Learn { who: any.clone() }),
        ("AttackMandateNextTurn", Effect::AttackMandateNextTurn { who: any.clone() }),
        ("UnlessPlayerPays", Effect::UnlessPlayerPays { who: any.clone(), cost: WardCost::Life(1), then: body(), if_paid: None }),
        ("PutFromHandOntoBattlefield", Effect::PutFromHandOntoBattlefield { who: any.clone(), filter: R::Any, count: Value::ONE, tapped: false, haste: false, sacrifice_eot: false, return_eot: false, then: None }),
        ("SearchAnyNumber", Effect::SearchAnyNumber { who: any.clone(), filter: R::Any, to: ZoneDest::Exile }),
        ("RevealHandDiscardMatchingUnlessPayLife", Effect::RevealHandDiscardMatchingUnlessPayLife { who: any.clone(), filter: R::Any, life: 1 }),
        ("UntapChosenPerCardInGraveyard", Effect::UntapChosenPerCardInGraveyard { who: any.clone() }),
        ("MayExileFromGraveyardElse", Effect::MayExileFromGraveyardElse { who: any.clone(), otherwise: body() }),
        ("TradeSecrets", Effect::TradeSecrets { who: any.clone() }),
        ("ExileUntilDuplicateName", Effect::ExileUntilDuplicateName { who: any.clone() }),
    ];

    for (name, eff) in &table {
        let pinned = serde_json::to_string(&eff.with_asked_seat(3)).expect("serialize");
        assert!(
            pinned.contains(r#""who":{"Seat":3}"#),
            "{name} is listed by `with_asked_seat` and did not pin its seat: {pinned}",
        );
    }

    // An arm that asks `ctx.controller` about ANOTHER seat's cards must stay
    // out: pinning `who` there rewrites the victim, not the asker.
    let victim = Effect::Fateseal { who: any.clone(), amount: Value::ONE };
    let untouched = serde_json::to_string(&victim.with_asked_seat(3)).expect("serialize");
    assert!(
        !untouched.contains(r#""who":{"Seat":3}"#),
        "Fateseal asks its own controller — its `who` is the victim and must not be pinned",
    );

    // The table is the arm list, read out of the source.
    let query_rs =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../crabomination_base/src/effect/query.rs");
    let src = std::fs::read_to_string(query_rs).expect("query.rs");
    let start = src.find("pub fn with_asked_seat").expect("with_asked_seat");
    let end = src[start..].find("\n    }").expect("end of fn") + start;
    let mut listed: Vec<&str> = src[start..end]
        .match_indices("Effect::")
        .map(|(i, _)| {
            let rest = &src[start + i + "Effect::".len()..];
            &rest[..rest.find(' ').unwrap_or(0)]
        })
        .collect();
    listed.sort_unstable();
    let mut covered: Vec<&str> = table.iter().map(|(n, _)| *n).collect();
    covered.sort_unstable();
    assert_eq!(
        listed, covered,
        "the table above must be exactly `with_asked_seat`'s arm list — an arm added there \
         without a row here is an unproved rewrite",
    );
}

/// No engine site adds to a P/T bonus field with a bare `+=`.
///
/// A power is bounded by nothing — Exponential Growth is "double target
/// creature's power {X} times" and reaches `i32::MAX` in one resolution — so
/// the *next* `+=` on the field is an overflow: a panic under
/// `overflow-checks` (the sweep binary) and, in release, the biggest creature
/// on the board evaluating as the smallest. `CardInstance::pump` and
/// `pump_permanent` saturate, and eighteen sites went through them; the field
/// has to stay `pub` for the engine crate to reach it, so this is the ratchet
/// that keeps a nineteenth from being written the old way.
///
/// Reads the source because there is nothing in the type system to ask: the
/// same extraction a `rg` would do, pinned so it runs with the suite.
#[test]
fn every_pump_goes_through_the_saturating_helper() {
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).expect("readable").flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let src = std::fs::read_to_string(&p).expect("utf-8");
                // Test modules set the fields directly to build a board; the
                // rule is about the engine's own accumulation.
                let live = src.split("#[cfg(test)]").next().unwrap_or(&src);
                for (i, line) in live.lines().enumerate() {
                    let t = line.trim();
                    if t.starts_with("//") {
                        continue;
                    }
                    for f in ["power_bonus", "toughness_bonus"] {
                        if t.contains(&format!("{f} +=")) || t.contains(&format!("{f} -=")) {
                            out.push(format!("{}:{} — {t}", p.display(), i + 1));
                        }
                    }
                }
            }
        }
    }
    let mut bad = Vec::new();
    for crate_src in ["/../crabomination/src", "/../crabomination_base/src"] {
        let root = format!("{}{crate_src}", env!("CARGO_MANIFEST_DIR"));
        walk(std::path::Path::new(&root), &mut bad);
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} site(s) accumulate a P/T bonus with a bare `+=`; use `CardInstance::pump` / \
         `pump_permanent`, which saturate:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// No engine site SUMS or DIFFERENCES a saturating rules quantity with plain
/// arithmetic.
///
/// The twin of `every_pump_goes_through_the_saturating_helper` one level down.
/// That one keeps the *write* to a P/T bonus from wrapping; this one keeps the
/// **consumers** from wrapping on top of it. A power saturates at `i32::MAX`
/// (Exponential Growth doubles one `{X}` times), and a life total does too
/// (Beacon of Immortality), so `a.power() + b.power()`, `sum()` over a board's
/// powers, and `mine.life - theirs.life` all overflow: a panic under
/// `overflow-checks` — which is the sweep binary and the actor's own
/// `overflow` profile — and, in `release`, a training label or a block plan
/// computed from a wrapped number.
///
/// Fifteen sites were written that way, all of them reachable from bot
/// self-play: the actor's own `life_diff` and per-side power totals
/// (`selfplay.rs`, which feeds `TrainRow`), the block planner's gang-damage
/// sums, the crew/saddle totals, `TotalPowerControlled` and three other
/// `Value`/`Predicate` sums, and the cost reduction that counts a board's
/// power. `saturating_add` / `saturating_sub` / `fold(0, saturating_add)`
/// everywhere; the difference of two life totals widens to `i64` instead,
/// because both ends saturate and the clamp is applied after.
///
/// ⚠ Test modules are skipped by BRACE MATCHING, not by cutting the file at
/// the first `#[cfg(test)]`: `bot.rs` has one at line 5,592 of 24,419, so the
/// cut version of this walk would read a fifth of the file that matters most.
#[test]
fn no_saturating_quantity_is_summed_with_plain_arithmetic() {
    /// Plain arithmetic ON a saturating read.
    const ARITH: [&str; 4] =
        [".power() +", ".toughness() +", ".power() -", ".toughness() -"];
    /// Accumulators that hold a sum of them.
    const TOTALS: [&str; 2] = ["total_power +=", "attacker_power +="];
    /// Byte ranges of every `#[cfg(test)]` item, by brace matching from the
    /// attribute to the close of the item it decorates.
    fn test_spans(src: &str) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        let mut from = 0usize;
        while let Some(rel) = src[from..].find("#[cfg(test)]") {
            let at = from + rel;
            let Some(open) = src[at..].find('{').map(|o| at + o) else { break };
            let (mut depth, mut i) = (0i32, open);
            let b = src.as_bytes();
            while i < b.len() {
                match b[i] {
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            out.push((at, i.min(src.len())));
            from = i.max(at + 1);
        }
        out
    }
    fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).expect("readable").flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let src = std::fs::read_to_string(&p).expect("utf-8");
                let spans = test_spans(&src);
                let mut at = 0usize;
                for (i, line) in src.lines().enumerate() {
                    let start = at;
                    at += line.len() + 1;
                    if spans.iter().any(|(a, b)| start >= *a && start < *b) {
                        continue;
                    }
                    let t = line.trim();
                    if t.starts_with("//") || t.contains("saturating_") {
                        continue;
                    }
                    if ARITH.iter().chain(TOTALS.iter()).any(|pat| t.contains(pat)) {
                        out.push(format!("{}:{} — {t}", p.display(), i + 1));
                    }
                }
            }
        }
    }
    let mut bad = Vec::new();
    for crate_src in ["/../crabomination/src", "/../crabomination_base/src"] {
        let root = format!("{}{crate_src}", env!("CARGO_MANIFEST_DIR"));
        walk(std::path::Path::new(&root), &mut bad);
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} site(s) do plain arithmetic on a saturating rules quantity; use \
         `saturating_add` / `saturating_sub` (or widen to `i64`):\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// CR 308.1 — a Kindred card always carries the creature type it shares.
///
/// "Kindred cards have another card type and one or more creature types": the
/// shared type is the whole reason the card type exists, so an empty
/// `creature_types` on a Kindred card is a defect that reads as a legal card.
/// Kozilek's Command and All Is Dust were `Kindred Instant` / `Kindred Sorcery`
/// with no Eldrazi on them, Crib Swap a `Kindred Instant` with no Shapeshifter;
/// six shipped that way, found by the subtype column of
/// `scripts/audit_printed_body.py`. The consumer is real —
/// `SelectionRequirement::HasCreatureType` over a spell, and a changeling's
/// every-type grant reads the same list.
///
/// That audit needs the oracle cache and skips what it cannot read; this needs
/// neither and cannot skip, which is why the class gets both.
///
/// ⚠ THE SAME RULE DOES NOT HOLD FOR PLANESWALKERS. It looks like it should
/// (CR 205.3k lists the types) and the first cut of this test asserted it —
/// then named six cards, and the oracle said three of them are right: The
/// Wanderer, The Wandering Emperor and The Eternal Wanderer all print
/// "Legendary Planeswalker" with nothing after it, and The Aetherspark prints
/// `— Equipment`. The other two (Dakkon, Ellywick) were real and are fixed.
/// A planeswalker's type is the oracle column's to check, not an invariant's.
#[test]
fn a_kindred_card_always_carries_the_creature_type_it_shares() {
    use crabomination::card::CardType;
    let mut bad: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        if def.card_types.contains(&CardType::Kindred)
            && def.subtypes.creature_types.is_empty()
        {
            bad.push(format!("{}: Kindred with no creature type", def.name));
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} Kindred card(s) with no creature type:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// CR 702.107 — the prowess KEYWORD and the prowess TRIGGER travel together.
///
/// `shortcut::prowess()`'s own doc states the convention: a factory declares
/// `Keyword::Prowess`, and the helper converts the tag into a functional
/// trigger — "the keyword itself remains in `card.keywords` for display +
/// future 'Prowess matters' payoffs to filter on". Nine shipped cards carried
/// the trigger with no keyword (Abbot of Keral Keep, Monastery Mentor, Drake
/// Hatcher, Meticulous Artisan, Jhessian Thief and four Prismari bodies), so
/// `has_keyword(&Keyword::Prowess)` was false for creatures that print it and
/// no filter could see them.
///
/// The other direction is fine and common — the keyword alone is the normal
/// spelling, and the engine mints the pump for it (see
/// `prowess_survives_an_unrelated_cast_trigger_and_is_not_doubled`, which is
/// the reason carrying BOTH is safe).
#[test]
fn a_card_with_the_prowess_trigger_also_carries_the_prowess_keyword() {
    use crabomination::card::Keyword;
    use crabomination::effect::shortcut::{prowess, prowess_trigger};
    // ⚠ EXACT EQUALITY, NOT THE PUMP'S SHAPE. "A `SpellCast` trigger whose
    // effect is +1/+1 until end of turn" is also Kami of the Hunt (Spirit or
    // Arcane), Lorehold's Lesson triggers and forty-odd other Kamigawa /
    // Strixhaven bodies — same pump, a different FILTER, and none of them
    // prowess. The claim here is narrow: a card built with the canonical
    // helper declares the keyword the helper's doc says it should.
    let canonical = [prowess(), prowess_trigger()];
    let mut bad: Vec<String> = Vec::new();
    for factory in all_known_factories() {
        let def = factory();
        let has_trigger = def
            .triggered_abilities
            .iter()
            .any(|ta| canonical.contains(ta));
        if has_trigger && !def.keywords.contains(&Keyword::Prowess) {
            bad.push(def.name.to_string());
        }
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} card(s) with the prowess trigger and no `Keyword::Prowess`:\n  {}",
        bad.len(),
        bad.join("\n  "),
    );
}

/// CR 305.6 — a basic land type IS the intrinsic `{T}: Add <color>`, so giving
/// one to a land that does not print it makes the land that type to everything
/// else that reads one. The ten MDFC pathways print "Land" with a plain tap
/// ability and no subtype, and every face carried the matching basic type until
/// 2026-09-12: a fetch searching for a Swamp found Blightstep Pathway, landwalk
/// evaded over it, Domain counted it.
///
/// Both halves are pinned, because the type was there as a (redundant) way to
/// spell the mana: no land type on either face, and each face still taps for
/// exactly the one colour it prints.
#[test]
fn no_pathway_face_carries_a_basic_land_type() {
    use crabomination::effect::{Effect, ManaPayload};
    use crabomination::mana::Color as C;
    /// A pathway factory and the colour each of its two faces taps for.
    type Pathway = (fn() -> crabomination::card::CardDefinition, C, C);
    let pathways: [Pathway; 10] = [
        (crabomination::catalog::blightstep_pathway, C::Black, C::Red),
        (crabomination::catalog::darkbore_pathway, C::Black, C::Green),
        (crabomination::catalog::branchloft_pathway, C::Green, C::White),
        (crabomination::catalog::clearwater_pathway, C::Blue, C::Black),
        (crabomination::catalog::cragcrown_pathway, C::Red, C::Green),
        (crabomination::catalog::hengegate_pathway, C::White, C::Blue),
        (crabomination::catalog::riverglide_pathway, C::Blue, C::Red),
        (crabomination::catalog::barkchannel_pathway, C::Green, C::Blue),
        (crabomination::catalog::brightclimb_pathway, C::White, C::Black),
        (crabomination::catalog::needleverge_pathway, C::Red, C::White),
    ];
    let taps_for = |def: &crabomination::card::CardDefinition, want: C| {
        def.activated_abilities.iter().any(|a| {
            a.tap_cost
                && matches!(&a.effect, Effect::AddMana { pool: ManaPayload::Colors(c), .. }
                    if c.as_slice() == [want])
        })
    };
    for (factory, front_color, back_color) in pathways {
        let front = factory();
        let back = front.back_face.as_deref().expect("pathway has a back face");
        for face in [&front, back] {
            assert!(
                face.subtypes.land_types.is_empty(),
                "{} prints no land subtype but ships {:?}",
                face.name,
                face.subtypes.land_types,
            );
        }
        assert!(taps_for(&front, front_color), "{} lost its mana", front.name);
        assert!(taps_for(back, back_color), "{} lost its mana", back.name);
    }
}
