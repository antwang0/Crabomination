//! At-a-glance keyword flags floated over battlefield creatures.
//!
//! A creature's evergreen combat keywords (flying, deathtouch, lifelink, …)
//! only live in the card's text box, which is illegible once the card is
//! minified at the table's oblique angle — especially across the table on an
//! opponent's board. This floats a small abbreviated strip ("Fly DT LL") over
//! the top of each creature so the board reads at a glance.
//!
//! Mechanism mirrors `pt_label`: a screen-space UI strip reprojected from the
//! card's world position every frame, reconciled against the engine view
//! (spawned for newly-keyworded creatures, despawned when a creature loses all
//! displayable keywords or leaves the battlefield). It sits at the card's top
//! edge so it never collides with the bottom-right P/T badge, and renders
//! below default-z UI so popups / tooltips win.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::{CardId, Keyword, WardCost};

use crate::card::{BattlefieldCard, GameCardId, CARD_HEIGHT};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::UiFonts;
use crate::MainCamera;

/// Renders below default-z (0) UI so popups / tooltips / modals win — same
/// band as the P/T badge.
const KW_Z: i32 = -1;
/// Lift the strip a few px above the card's projected top edge so it reads as
/// a banner sitting on the card rather than overlapping the title.
const KW_LIFT: f32 = 14.0;
/// Rough px width per character at the strip's font size, used only to centre
/// the strip horizontally over the card (the node itself auto-sizes).
const KW_CHAR_PX: f32 = 6.5;

/// Screen-space keyword strip tied to a battlefield card's `CardId`.
#[derive(Component)]
pub struct KeywordLabel(pub CardId);

/// Short tag for the combat/board-relevant keywords worth surfacing on a
/// permanent. Casting-only keywords (Flashback, Kicker, Buyback, …) return
/// `None` — they never matter for a creature already on the battlefield.
fn keyword_tag(kw: &Keyword) -> Option<&'static str> {
    use Keyword::*;
    Some(match kw {
        Flying => "Fly",
        Reach => "Rch",
        Menace => "Men",
        Trample => "Tmp",
        Vigilance => "Vig",
        FirstStrike => "FS",
        DoubleStrike => "DS",
        Deathtouch => "DT",
        Lifelink => "LL",
        Haste => "Hst",
        Defender => "Def",
        Indestructible => "Ind",
        Hexproof => "Hex",
        HexproofFromColor(_) => "HexC",
        HexproofExceptColors(_) => "HexX",
        HexproofFromAbilities => "HexA",
        HexproofFromMonocolored => "HexM",
        Shroud => "Shr",
        Unblockable => "Unb",
        Intimidate => "Int",
        Fear => "Fear",
        Infect => "Inf",
        Wither => "Wth",
        Skulk => "Skk",
        Shadow => "Shd",
        Horsemanship => "Hrs",
        Landwalk(_) | LandwalkFiltered(_) => "Wlk",
        Protection(_) => "Pro",
        ProtectionFromManaValueExcept(_) => "ProMV",
        ProtectionFromManaValueParity { odd } => if *odd { "Pro-odd" } else { "Pro-even" },
        ProtectionFromMulticolored => "ProMC",
        ProtectionFromMonocolored => "ProM1",
        ProtectionFromInstants => "ProI",
        ProtectionFromSpells => "ProS",
        ProtectionFromColoredSpells => "ProCS",
        ProtectionFromEverything => "Pro★",
        Ward(_) => "Ward",
        // Glasskite — the first spell or ability to target it each turn is
        // countered outright, so it reads like an untaxed Ward.
        CounterFirstTargetingEachTurn => "Ward1",
        Toxic(_) => "Tox",
        Poisonous(_) => "Psn",
        // Modular N — the counters it carries (and hands off on death) are a
        // real board read.
        Modular(_) => "Mod",
        // Sunburst — the counters it lands with are a board read.
        Sunburst => "Sun",
        // Prowess — a noncreature spell can swing this creature's combat math,
        // so an opponent should weigh the controller's open cards before blocking.
        Prowess => "Prw",
        // Combat-relevant statuses worth a glance on the board.
        CantBlock => "NoBlk",
        // Ironclaw Orcs — can't block creatures with power N or greater.
        CantBlockPowerAtLeast(_) => "NoBlk≥",
        // "Can't attack" (Pacifism / Cage of Hands) and the conditional
        // Goblin-Cohort lock both read at a glance on the board.
        CantAttack => "NoAtk",
        CantAttackUnlessCastCreatureThisTurn => "Atk?",
        // Harbor Serpent's land-count gate and Bloodcrazed Goblin's first-blood
        // gate — both read at a glance next to the other attack locks.
        CantAttackUnlessLandCount(_, _) => "Land?",
        CantAttackUnlessOpponentDamaged => "Blood?",
        // Hazoret-class hellbent gate reads at a glance on the board.
        CantAttackOrBlockUnlessHandSizeAtMost(_) => "Hand?",
        CantAttackOrBlockUnlessDelirium => "Dlr?",
        // The Oppressive Rays pay gate — the number is the point.
        CantAttackOrBlockUnlessPay(_) => "Pay?",
        CantAttackOrBlockUnlessCreatureDiedThisTurn => "Died?",
        CantAttackOrBlockUnlessDescend(_) => "Dsc?",
        CantAttackOrBlockUnlessCityBlessing => "Bless?",
        Decayed => "Dcy",
        Flanking => "Flk",
        // Combat-pump statics from the Kamigawa/legacy sets read at a glance.
        Bushido(_) => "Bsd",
        Melee => "Mle",
        Rampage(_) => "Rmp",
        Frenzy(_) => "Frz",
        Banding => "Bnd",
        // Generalized menace — "can't be blocked except by N or more."
        CantBeBlockedExceptByN(_) => "Men+",
        // Evasion by blocker quality. "Can only be blocked by [filter]"
        // (Serpent of Yawning Depths) is strong evasion → "Eva". "Can't be
        // blocked by [filter]" (Vindictive Mob's "…by Saprolings") only
        // excludes a slice of blockers → "Eva-" so the board doesn't read it
        // as fully evasive.
        // "Eva+·X" = can *only* be blocked by X (restrictive); its mirror
        // "Eva-·X" = can't be blocked by X (exclusion). The +/- makes the two
        // read as a pair rather than the ambiguous bare "Eva".
        CantBeBlockedExceptBy(_) => "Eva+",
        CantBeBlockedBy(_) => "Eva-",
        // "Can't be blocked by more than one creature" (anti-gang-block).
        CantBeBlockedByMoreThanOne => "1Blk",
        // Power-gated evasion — split so the board reads which way the gate
        // points: "less power than this" (Formation Breaker), "power N or less"
        // (Questing Beast, Stormkeld Vanguard), "power N or more".
        CantBeBlockedByPowerLess => "Eva<",
        CantBeBlockedByPowerAtMost(_) => "Eva≤",
        CantBeBlockedByPowerAtLeast(_) => "Eva≥",
        // "Can block only creatures with flying" (Wanderlight Spirit).
        CanBlockOnlyFlying => "FlyBlk",
        MustBeBlocked => "Lure",
        // "Attacks each combat if able" (Impending Doom, The Akroan War II) —
        // a board-relevant combat compulsion.
        MustAttack => "Atk!",
        // Crew N on a Vehicle — a glanceable reminder it can be animated.
        Crew(_) => "Crew",
        // Saddle N on a Mount (CR 702.171) — like Crew, a board reminder its
        // saddled riders come online when it attacks.
        Saddle(_) => "Sdl",
        // Resilience keywords — "this dies but comes back" reads at a glance and
        // changes how an opponent should attack/block into it.
        Persist => "Per",
        Undying => "Und",
        // Eldrazi annihilator — a combat threat worth surfacing on the board.
        Annihilator(_) => "Ann",
        // Absorb N (CR 702.64) — prevents N damage from each source per event,
        // so an opponent should weigh whether an attacker punches through.
        Absorb(_) => "Abs",
        // Firebending — attack-triggered red mana worth flagging on the board.
        Firebending(_) | FirebendingPower | FirebendingCreaturesYouControl => "FB",
        // "Assigns combat damage equal to its toughness" (Doran) — changes how
        // its combat math reads at a glance.
        AssignsCombatDamageByToughness => "T-dmg",
        // "Excess trample damage tramples over planeswalkers" (Questing Beast)
        // — a real combat read when attacking a walker behind a blocker.
        TrampleOverPlaneswalkers => "Tmp→PW",
        // Status keywords that change what a creature can be targeted/blocked by
        // or how it reads in combat.
        Phasing => "Phs",
        Changeling => "Chg",
        Reconfigure(_) => "Rcfg",
        // Ability-lock granted by auras/effects (Petrify) — its activated
        // abilities can't be activated, worth surfacing alongside NoAtk/NoBlk.
        CantActivateAbilities => "NoAbil",
        // Resilience: regenerate shields and totem/umbra armor both mean "the
        // next destruction is soaked" — it changes how an opponent trades.
        Regenerate(_) => "Rgn",
        UmbraArmor => "TArm",
        // Protection from creatures / a creature type is board-relevant: it
        // gates blocking and combat damage, not just spell targeting.
        ProtectionFromCreatures => "ProCr",
        ProtectionFromCreatureType(_) => "ProCT",
        // The general filtered form ("protection from non-Spirit creatures").
        ProtectionFromMatching(_) => "ProF",
        // Protection from a card type (e.g. from artifacts) likewise gates
        // which attackers/blockers connect; the suffix names the dodged type.
        ProtectionFromCardType(_) => "ProT",
        // Protection from a spell subtype (e.g. from Auras) — a targeting/
        // attachment gate, the last of the Protection* family to surface.
        ProtectionFromSpellSubtype(_) => "ProSub",
        // Combat compulsions/restrictions that change how an opponent should
        // attack or block into this creature — the mirror side of MustAttack.
        MustBlock | AllMustBlock => "MBlk",
        AttacksAlone => "Solo",
        // Pack-tactics restrictions — the creature can't be declared alone, so
        // an opponent reads that it needs a companion to attack/block.
        CantAttackAlone | CantAttackOrBlockAlone => "Pack",
        // "Assigns no combat damage" (Illusionist's Gambit-style) — a real
        // combat read: it can chump/soak without dealing back.
        DealsNoCombatDamage => "0dmg",
        // Exert — "you may exert as it attacks" for a bonus; a glanceable
        // reminder the attack carries an optional payoff.
        Exert => "Exrt",
        // Soulbond — a paired keyword-grant; worth flagging that it (un)pairs.
        Soulbond => "Bond",
        // Spell-count evasion (Illvoi Infiltrator) — reads as evasion.
        CantBeBlockedIfControllerCastSpells(_) => "Eva",
        // "Can attack only if the defending player controls a [permanent]"
        // (Sea Serpent, Dandân) — a conditional attacker worth flagging so the
        // player sees why it can't always be declared.
        CanAttackOnlyIfDefenderControls(_) => "Atk?",
        // "Can attack only if you control a [permanent]" (the you-side mirror).
        CanAttackOnlyIfYouControl(_) => "Atk?",
        // "Can't attack/block unless you control N or more [filter]" (Topiary
        // Stomper, Lambholt Pacifist, Olog-hai Crusher) — surface which side is
        // gated so the player sees why it can't be declared.
        CantAttackOrBlockUnlessYouControlCount { attack_only, block_only, .. } => {
            if *attack_only { "Atk?" } else if *block_only { "Blk?" } else { "A/B?" }
        }
        // "Can't attack or block unless it has an even number of counters on it"
        // (Sab-Sunen) — a live combat gate that flips as counters change.
        CantAttackOrBlockUnlessEvenCounters => "Even?",
        // Upkeep obligations & count-down timers change how long a permanent
        // sticks around — a real board read for both players (the remaining
        // count rides the counter coins; these tags flag the mechanic).
        Echo(_) => "Echo",
        CumulativeUpkeep(_) => "CmUp",
        Fading(_) => "Fade",
        Vanishing(_) => "Vanish",
        // "Doesn't untap while it has a [kind] counter" (Steel Dromedary) — a
        // board read: the creature stays tapped until the counter comes off.
        DoesntUntapWhileCounter(_) => "NoUntap",
        // CR 502.3 — "you may choose not to untap this" (Hisoka's Guard,
        // Vedalken Shackles). Distinct from NoUntap: nothing is stopping it,
        // its controller gets a choice each untap step.
        MayChooseNotToUntap => "MayHold",
        // Start your engines! (CR 702.179) — flags that this permanent feeds the
        // speed mechanic and carries "Max speed —" abilities that come online
        // once its controller reaches speed 4.
        StartYourEngines => "Eng",
        // Devoid (CR 702.114) — the permanent is colorless regardless of its
        // mana cost; a board read for color-matters interactions (protection,
        // devotion, "another colorless creature").
        Devoid => "Dvd",
        // Day/Night transform state (CR 702.145) — which face a
        // daybound/nightbound permanent currently shows.
        Daybound => "Day",
        Nightbound => "Night",
        // Disguise (CR 702.168) — a face-down 2/2 with ward {2} that can be
        // turned face up; the chip flags the hidden card.
        Disguise(_) => "Dsg",
        // Morph / Megamorph (CR 702.37) — the face-down 2/2 sibling of Disguise
        // (no ward), turnable face up for its unmorph cost; flag the hidden card.
        Morph(_) | Megamorph(_) => "Mph",
        // (Rampage/Bushido/Annihilator/Absorb/Frenzy are labelled above.)
        // Unleash (CR 702.98, Rakdos/GTC) — the marker flags an unleashed
        // creature; once it carries a +1/+1 counter the injected `CantBlock`
        // adds the "NoBlk" read, but the tag identifies the mechanic up front.
        Unleash => "Unl",
        _ => return None,
    })
}

/// Render a Ward cost compactly for its "Ward…" tag: the mana total for a mana
/// ward ("Ward2"), the life for "Ward—Pay N life" ("Ward7♥"), and terse markers
/// for the discard / blight / sacrifice / dynamic variants. What the opponent
/// has to pay to target this permanent is a real board read.
fn ward_suffix(cost: &WardCost) -> String {
    use WardCost::*;
    match cost {
        Mana(c) => c.cmc().to_string(),
        Life(n) => format!("{n}♥"),
        ManaAndLife(c, n) => format!("{}+{n}♥", c.cmc()),
        Discard(n) => format!("{n}↓"),
        DiscardHand => "hand↓".into(),
        Blight(n) => format!("{n}☠"),
        CollectEvidence(n) => format!("ev{n}"),
        SacrificeCreature => "sac".into(),
        SacrificePermanents(n) => format!("sac{n}"),
        RemoveCounterFromPermanent => "ctr-".into(),
        GenericSourcePower => "P".into(),
        LifeSourcePower => "P♥".into(),
    }
}

/// A short board-glance label for the common simple blocker filters used by
/// filtered evasion ("can't be blocked by [filter]" / "…except by [filter]").
/// Returns `None` for compound/complex filters so the strip stays uncluttered.
fn req_short(req: &crabomination::card::SelectionRequirement) -> Option<String> {
    use crabomination::card::SelectionRequirement as R;
    Some(match req {
        R::HasKeyword(k) => keyword_tag(k)?.to_string(),
        R::HasCreatureType(t) => format!("{t:?}"),
        R::HasColor(c) => format!("{c:?}"),
        R::Artifact => "Art".to_string(),
        R::Enchantment => "Ench".to_string(),
        R::Creature => "Cre".to_string(),
        R::Land => "Land".to_string(),
        R::Planeswalker => "PW".to_string(),
        R::HasLandType(t) => format!("{t:?}"),
        // Composite filters (e.g. "can't be blocked by white creatures" =
        // And(Creature, HasColor(White))) — name the more specific half so
        // the chip reads "Eva-·White" rather than a bare "Eva-". A plain
        // "Cre" qualifier yields to the informative sibling.
        R::And(a, b) => match (req_short(a), req_short(b)) {
            // "Creature and X" (the common "X creatures" filter) names X; the
            // generic "Cre" qualifier yields to its informative sibling.
            (Some(sa), Some(sb)) if sa == "Cre" => sb,
            (Some(sa), Some(sb)) if sb == "Cre" => sa,
            // Two distinct specific classes (flying AND artifact) can't be
            // summarized by one half — a blocker needs both — so stay unadorned.
            (Some(_), Some(_)) => return None,
            // One nameable half beside an unnameable one still names itself.
            (Some(sa), None) => sa,
            (None, Some(sb)) => sb,
            _ => return None,
        },
        // Negated classes read with a leading "!" — "non-Spirit creatures"
        // (Harbinger of Spring) renders as "!Spirit".
        R::Not(inner) => format!("!{}", req_short(inner)?),
        // Disjunctive blocker classes (Spire Tracer — "except by creatures with
        // flying or reach") read as "Fly/Rch" so both required classes show.
        // Both halves must name themselves, else the chip stays unadorned.
        R::Or(a, b) => match (req_short(a), req_short(b)) {
            (Some(sa), Some(sb)) => format!("{sa}/{sb}"),
            _ => return None,
        },
        _ => return None,
    })
}

/// The numeric magnitude worth appending to a count-carrying keyword's tag —
/// the N in Rampage N / Toxic N / Annihilator N etc. materially changes how the
/// creature reads in combat, so surface it ("Rmp2", "Tox3") rather than dropping
/// it. Crew N / Saddle N carry the total *power* needed to online the
/// Vehicle/Mount, a real board read ("Crew3" is much harder to turn on than
/// "Crew1"), so include them too. Ward renders its concrete cost via
/// `ward_suffix` ("Ward2", "Ward7♥").
fn keyword_value_suffix(kw: &Keyword) -> Option<String> {
    use Keyword::*;
    if let Ward(cost) = kw {
        return Some(ward_suffix(cost));
    }
    // Protection from a creature type names the type ("ProCT·Coyote"): which
    // type it dodges is the whole board read (who can block it / damage it).
    if let ProtectionFromCreatureType(t) = kw {
        return Some(format!("·{t:?}"));
    }
    // The filtered form names what it dodges when the filter is simple enough
    // to render — Harbinger of Spring reads "ProF·!Spirit".
    if let ProtectionFromMatching(f) = kw {
        return req_short(f).map(|d| format!("·{d}"));
    }
    // Filtered evasion names the excluded/required blocker class when it's a
    // simple filter — "Eva-·Fly" (Gnat Alley Creeper can't be blocked by
    // flyers) reads far better than a bare "Eva-". Complex filters stay
    // unadorned.
    if let CantBeBlockedBy(f) | CantBeBlockedExceptBy(f) = kw {
        return req_short(f).map(|s| format!("·{s}"));
    }
    // Protection from a card type names the dodged type ("ProT·Artifact").
    if let ProtectionFromCardType(t) = kw {
        return Some(format!("·{t:?}"));
    }
    let n = match kw {
        Rampage(n) | Bushido(n) | Frenzy(n) | Annihilator(n) | Absorb(n) | Toxic(n)
        | Poisonous(n) | CantBeBlockedExceptByN(n) | Crew(n) | Saddle(n) | Modular(n) => *n,
        // The power threshold in the evasion/blocker restrictions is a real
        // combat read — "Eva≤2" (Rust-Shield Rampager) vs "Eva≤3" gate
        // different blockers; "NoBlk≥4" says which attackers this can't stop.
        // "Eva≥N" (can't be blocked by power N or more) carries its threshold
        // too so it reads symmetrically with its "Eva≤N" sibling.
        CantBeBlockedByPowerAtMost(n) | CantBlockPowerAtLeast(n)
        | CantBeBlockedByPowerAtLeast(n) => *n,
        _ => return None,
    };
    Some(n.to_string())
}

/// Build the displayed strip for a permanent's keyword list: each displayable
/// keyword's tag (with its count suffix where one applies), first-occurrence
/// order, de-duplicated, space-joined. Empty string when nothing is worth
/// showing.
fn keyword_strip(keywords: &[Keyword]) -> String {
    let mut seen: HashSet<String> = HashSet::new();
    let mut tags: Vec<String> = Vec::new();
    for kw in keywords {
        if let Some(tag) = keyword_tag(kw) {
            let full = match keyword_value_suffix(kw) {
                Some(sfx) => format!("{tag}{sfx}"),
                None => tag.to_string(),
            };
            if seen.insert(full.clone()) {
                tags.push(full);
            }
        }
    }
    tags.join(" ")
}

/// Full board-glance strip: the keyword chips plus status prefixes that aren't
/// keywords — "Susp" for a suspected creature (CR 701.60) and "Zzz" for
/// summoning sickness. Empty when there's nothing to show.
fn board_status_strip(
    keywords: &[Keyword],
    summoning_sick: bool,
    suspected: bool,
    goaded: bool,
    detained: bool,
    case_solved: Option<bool>,
    class_level: Option<u8>,
    saddled: bool,
    crewed_count: u32,
    stun: u32,
    wont_untap: bool,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    // CR 716 Class: show the current level. Leads the strip so the level reads
    // at a glance on the non-creature enchantment.
    if let Some(n) = class_level {
        parts.push(format!("Lvl {n}"));
    }
    // MKM Case (CR — Solve): show whether it's solved yet. Leads the strip so the
    // solve state reads at a glance on the non-creature enchantment.
    match case_solved {
        Some(true) => parts.push("Solved".to_string()),
        Some(false) => parts.push("Case".to_string()),
        None => {}
    }
    // Suspected reads at a glance — it's *why* the creature shows Men/NoBlk.
    if suspected {
        parts.push("Susp".to_string());
    }
    // Goaded (CR 701.38) — the creature must attack a player other than the
    // goader if able. A combat compulsion imposed by an opponent, so it belongs
    // next to the MustAttack ("Atk!") read; surfaced from the view's goaded flag
    // rather than a keyword since goad is a status, not a printed keyword.
    if goaded {
        parts.push("Goad".to_string());
    }
    // CR 701.35 — a detained permanent can't attack/block and its abilities
    // can't be activated until the detainer's next turn. An opponent-imposed
    // lockdown that the tapped/counter coins don't convey, so it sits by the
    // other combat-restriction reads (Goad/MustAttack).
    if detained {
        parts.push("Detain".to_string());
    }
    // CR 702.171 — a Mount that's been saddled this turn has its
    // "attacks while saddled" riders armed for this combat. The transient
    // active state (distinct from the "Sdl N" cost chip) is a real board read,
    // so surface it always, not just in the hover tooltip.
    if saddled {
        parts.push("Sdl✓".to_string());
    }
    // CR 702.9 — a Vehicle crewed this turn shows its crewer count so
    // "for each creature that crewed it this turn" payoffs (Luxurious
    // Locomotive) read at a glance before it attacks.
    if crewed_count > 0 {
        parts.push(format!("Crew×{crewed_count}"));
    }
    // CR 122.1c — a permanent with stun counters skips that many untaps, so
    // it stays tapped-out of future combats/activations. A real board read that
    // the counter coin alone doesn't convey, so it sits by the "Zzz" can't-act tag.
    if stun > 0 {
        parts.push(format!("Stun {stun}"));
    }
    // A `PreventUntap` static (Paralyzing Grasp, Stasis Cell) keeps the
    // permanent from untapping during its controller's untap step — a lasting
    // opponent lock the tapped state alone doesn't explain. Sits by Stun (the
    // other "stays tapped" read); skipped when a Stun chip already says as much.
    if wont_untap && stun == 0 {
        parts.push("NoUntap".to_string());
    }
    // Summoning sickness gets a board-visible tag — skipped when Haste lifts it.
    if summoning_sick && !keywords.contains(&Keyword::Haste) {
        parts.push("Zzz".to_string());
    }
    let strip = keyword_strip(keywords);
    if !strip.is_empty() {
        parts.push(strip);
    }
    parts.join(" ")
}

/// Reconcile keyword strips with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_keyword_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(&GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut labels: Query<(Entity, &KeywordLabel, &mut Node, &mut Text)>,
    mut desired_cache: Local<HashMap<CardId, String>>,
) {
    // No view (between matches): clear every strip and bail.
    let Some(cv) = &view.0 else {
        for (e, _, _, _) in &mut labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // card_id → world position of the card's top-centre (the title edge),
    // transformed through the flat battlefield rotation.
    let top_center_local = Vec3::new(0.0, CARD_HEIGHT / 2.0, 0.0);
    let mut card_top: HashMap<CardId, Vec3> = HashMap::new();
    for (gid, gtf) in &cards {
        card_top.insert(gid.0, gtf.transform_point(top_center_local));
    }

    // Desired strips: creatures with at least one displayable keyword.
    // Rebuilt only on view change (keyword_strip allocates a String per
    // creature); anchoring/positioning below still tracks every frame, and
    // ids without a live entity yet are handled at use time (hidden or
    // parked offscreen until the anchor exists).
    if view.is_changed() {
        desired_cache.clear();
        for p in &cv.battlefield {
            // Creatures get the keyword/status strip; Cases and Classes
            // (non-creatures) get a state chip.
            if !p.is_creature() && p.case_solved.is_none() && p.class_level.is_none() {
                continue;
            }
            let stun = p
                .counters
                .iter()
                .find(|(k, _)| *k == crabomination::card::CounterType::Stun)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            let strip = board_status_strip(
                &p.keywords,
                p.summoning_sick,
                p.suspected,
                p.goaded,
                p.detained,
                p.case_solved,
                p.class_level,
                p.saddled,
                p.crewed_count,
                stun,
                p.wont_untap,
            );
            if !strip.is_empty() {
                desired_cache.insert(p.id, strip);
            }
        }
    }
    let desired = &*desired_cache;

    // Project a card-top world point to a viewport pixel, centring a strip of
    // `chars` glyphs over the card and lifting it above the top edge.
    let anchor = |world: Vec3, chars: usize| -> Option<(f32, f32)> {
        camera.world_to_viewport(cam_xform, world).ok().map(|v| {
            (v.x - chars as f32 * KW_CHAR_PX * 0.5, v.y - KW_LIFT)
        })
    };

    // Update existing strips; despawn any whose creature lost all keywords or
    // left the battlefield.
    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, label, mut node, mut text) in &mut labels {
        match desired.get(&label.0) {
            Some(strip) => {
                seen.insert(label.0);
                if let Some(world) = card_top.get(&label.0).copied()
                    && let Some((x, y)) = anchor(world, strip.chars().count())
                {
                    node.display = Display::Flex;
                    node.left = Val::Px(x);
                    node.top = Val::Px(y);
                } else {
                    node.display = Display::None;
                }
                if text.0 != *strip {
                    *text = Text::new(strip.clone());
                }
            }
            None => {
                commands.entity(e).despawn();
            }
        }
    }

    // Spawn strips for newly-keyworded creatures.
    for (id, strip) in desired.iter() {
        if seen.contains(id) {
            continue;
        }
        let (left, top) = card_top
            .get(id)
            .copied()
            .and_then(|world| anchor(world, strip.chars().count()))
            .unwrap_or((-1000.0, -1000.0));
        commands.spawn((
            KeywordLabel(*id),
            Text::new(strip.clone()),
            ui_fonts.tf(12.0),
            TextColor(Color::srgb(0.96, 0.94, 0.80)),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.62)),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                padding: UiRect::axes(Val::Px(4.0), Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(KW_Z),
            InGameRoot,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{board_status_strip, keyword_strip, req_short};
    use crabomination::card::Keyword;

    #[test]
    fn req_short_names_composite_filter_half() {
        use crabomination::card::SelectionRequirement as R;
        use crabomination::mana::Color;
        // "can't be blocked by white creatures" → name the color, not "Cre".
        let f = R::Creature.and(R::HasColor(Color::White));
        assert_eq!(req_short(&f).as_deref(), Some("White"));
        // A bare creature-type filter still names the type.
        let f2 = R::Creature.and(R::HasCreatureType(crabomination::card::CreatureType::Goblin));
        assert_eq!(req_short(&f2).as_deref(), Some("Goblin"));
        // The full card-type filter set is named, including planeswalkers.
        assert_eq!(req_short(&R::Planeswalker).as_deref(), Some("PW"));
        assert_eq!(
            req_short(&R::HasLandType(crabomination::card::LandType::Island)).as_deref(),
            Some("Island"),
        );
    }

    #[test]
    fn strip_dedupes_and_orders_combat_keywords() {
        let kws = vec![
            Keyword::Flying,
            Keyword::Deathtouch,
            Keyword::Flying, // duplicate — dropped
            Keyword::CantBlock,
            Keyword::Decayed,
        ];
        assert_eq!(keyword_strip(&kws), "Fly DT NoBlk Dcy");
    }

    #[test]
    fn strip_skips_non_displayable_keywords() {
        // Flash isn't a board-glance combat status → no badge.
        assert_eq!(keyword_strip(&[Keyword::Flash]), "");
    }

    #[test]
    fn strip_surfaces_morph_and_spell_subtype_protection() {
        use crabomination::card::SpellSubtype;
        use crabomination::mana::{cost, generic};
        // Face-down Morph flags the hidden card like Disguise does.
        assert_eq!(keyword_strip(&[Keyword::Morph(cost(&[generic(3)]))]), "Mph");
        assert_eq!(keyword_strip(&[Keyword::Megamorph(cost(&[generic(4)]))]), "Mph");
        // The last Protection* variant now surfaces too.
        assert_eq!(keyword_strip(&[Keyword::ProtectionFromSpellSubtype(SpellSubtype::Arcane)]), "ProSub");
    }

    #[test]
    fn strip_surfaces_ward_cost() {
        use crabomination::card::WardCost;
        use crabomination::mana::{cost, generic};
        // Ward—Pay 7 life (Sire of Seven Deaths) reads the concrete life cost.
        assert_eq!(keyword_strip(&[Keyword::Ward(WardCost::Life(7))]), "Ward7♥");
        // Ward {2} shows the mana total.
        assert_eq!(keyword_strip(&[Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))]), "Ward2");
    }

    #[test]
    fn strip_surfaces_resilience_and_status_keywords() {
        assert_eq!(keyword_strip(&[Keyword::Persist]), "Per");
        assert_eq!(keyword_strip(&[Keyword::Undying]), "Und");
        assert_eq!(keyword_strip(&[Keyword::Annihilator(2)]), "Ann2");
        assert_eq!(keyword_strip(&[Keyword::Changeling]), "Chg");
        assert_eq!(keyword_strip(&[Keyword::HexproofFromMonocolored]), "HexM");
        assert_eq!(keyword_strip(&[Keyword::Prowess]), "Prw");
        assert_eq!(keyword_strip(&[Keyword::FirebendingPower]), "FB");
        assert_eq!(keyword_strip(&[Keyword::Crew(2)]), "Crew2");
        assert_eq!(keyword_strip(&[Keyword::Saddle(3)]), "Sdl3");
        assert_eq!(keyword_strip(&[Keyword::StartYourEngines]), "Eng");
        assert_eq!(keyword_strip(&[Keyword::Regenerate(0)]), "Rgn");
        assert_eq!(keyword_strip(&[Keyword::UmbraArmor]), "TArm");
        assert_eq!(keyword_strip(&[Keyword::ProtectionFromCreatures]), "ProCr");
        assert_eq!(
            keyword_strip(&[Keyword::ProtectionFromCreatureType(
                crabomination::card::CreatureType::Coyote
            )]),
            "ProCT·Coyote",
            "protection-from-type names the dodged type",
        );
        assert_eq!(
            keyword_strip(&[Keyword::ProtectionFromCardType(crabomination::card::CardType::Artifact)]),
            "ProT·Artifact",
            "protection-from-card-type names the dodged type",
        );
        assert_eq!(keyword_strip(&[Keyword::CantAttackAlone]), "Pack");
        assert_eq!(
            keyword_strip(&[Keyword::MayChooseNotToUntap]),
            "MayHold",
            "the choice reads differently from a hard untap lock",
        );
        assert_eq!(keyword_strip(&[Keyword::CantAttackOrBlockAlone]), "Pack");
        assert_eq!(
            keyword_strip(&[Keyword::DoesntUntapWhileCounter(
                crabomination::card::CounterType::Charge
            )]),
            "NoUntap",
        );
    }

    #[test]
    fn strip_surfaces_power_thresholds() {
        // Rust-Shield Rampager — "can't be blocked by power 2 or less".
        assert_eq!(keyword_strip(&[Keyword::CantBeBlockedByPowerAtMost(2)]), "Eva≤2");
        assert_eq!(keyword_strip(&[Keyword::CantBlockPowerAtLeast(4)]), "NoBlk≥4");
        // "Eva≥N" carries its threshold symmetrically with "Eva≤N".
        assert_eq!(keyword_strip(&[Keyword::CantBeBlockedByPowerAtLeast(3)]), "Eva≥3");
    }

    #[test]
    fn evasion_restriction_and_exclusion_read_as_a_pair() {
        use crabomination::card::SelectionRequirement as R;
        // Sabertooth Alley Cat — can only be blocked by defenders ("Eva+·Def").
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedExceptBy(Box::new(R::HasKeyword(Keyword::Defender)))]),
            "Eva+·Def",
        );
        // Gnat Alley Creeper — can't be blocked by flyers ("Eva-·Fly").
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedBy(Box::new(R::HasKeyword(Keyword::Flying)))]),
            "Eva-·Fly",
        );
        // Simple Creature / Land blocker classes now name themselves too.
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedExceptBy(Box::new(R::Creature))]),
            "Eva+·Cre",
        );
        // Spire Tracer — "except by creatures with flying or reach" names both
        // required classes ("Eva+·Fly/Rch").
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedExceptBy(Box::new(
                R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Reach)),
            ))]),
            "Eva+·Fly/Rch",
        );
    }

    #[test]
    fn strip_surfaces_generalized_menace_and_lure() {
        assert_eq!(keyword_strip(&[Keyword::CantBeBlockedExceptByN(3)]), "Men+3");
        assert_eq!(keyword_strip(&[Keyword::MustBeBlocked]), "Lure");
    }

    #[test]
    fn strip_surfaces_block_quality_evasion() {
        use crabomination::card::SelectionRequirement;
        // Filtered evasion now names the simple blocker class it dodges. The
        // "except by" (restrictive) side reads "Eva+·X"; its "by" (exclusion)
        // sibling reads "Eva-·X".
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedExceptBy(Box::new(
                SelectionRequirement::HasKeyword(Keyword::Flying),
            ))]),
            "Eva+·Fly"
        );
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::Enchantment))]),
            "Eva-·Ench"
        );
        // Gnat Alley Creeper — can't be blocked by creatures with flying.
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedBy(Box::new(
                SelectionRequirement::HasKeyword(Keyword::Flying),
            ))]),
            "Eva-·Fly"
        );
        // A compound filter stays unadorned.
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedBy(Box::new(
                SelectionRequirement::HasKeyword(Keyword::Flying)
                    .and(SelectionRequirement::Artifact),
            ))]),
            "Eva-"
        );
        assert_eq!(keyword_strip(&[Keyword::CantBeBlockedByMoreThanOne]), "1Blk");
    }

    #[test]
    fn strip_surfaces_cant_attack_statuses() {
        assert_eq!(keyword_strip(&[Keyword::CantAttack]), "NoAtk");
        assert_eq!(
            keyword_strip(&[Keyword::CantAttackUnlessCastCreatureThisTurn]),
            "Atk?"
        );
    }

    #[test]
    fn strip_surfaces_conditional_attack_block_gates() {
        use crabomination::card::SelectionRequirement;
        assert_eq!(keyword_strip(&[Keyword::CanAttackOnlyIfYouControl(Box::new(
            SelectionRequirement::Creature))]), "Atk?");
        assert_eq!(keyword_strip(&[Keyword::CantAttackOrBlockUnlessEvenCounters]), "Even?");
        let gate = |a, b| Keyword::CantAttackOrBlockUnlessYouControlCount {
            filter: Box::new(SelectionRequirement::Land),
            min: 7, attack_only: a, block_only: b, exclude_self: false,
        };
        assert_eq!(keyword_strip(&[gate(true, false)]), "Atk?");
        assert_eq!(keyword_strip(&[gate(false, true)]), "Blk?");
        assert_eq!(keyword_strip(&[gate(false, false)]), "A/B?");
    }

    #[test]
    fn strip_surfaces_combat_pump_statics() {
        assert_eq!(keyword_strip(&[Keyword::Bushido(2)]), "Bsd2");
        assert_eq!(keyword_strip(&[Keyword::Rampage(1)]), "Rmp1");
        assert_eq!(keyword_strip(&[Keyword::Banding]), "Bnd");
        assert_eq!(keyword_strip(&[Keyword::Absorb(1)]), "Abs1");
    }

    #[test]
    fn strip_surfaces_count_suffix_for_scaling_keywords() {
        // The N in Rampage/Toxic/Poisonous/Annihilator changes the combat read,
        // so it rides along with the tag instead of being dropped.
        assert_eq!(keyword_strip(&[Keyword::Rampage(2)]), "Rmp2");
        assert_eq!(keyword_strip(&[Keyword::Toxic(3)]), "Tox3");
        assert_eq!(keyword_strip(&[Keyword::Poisonous(1)]), "Psn1");
        // Two different Rampage magnitudes are distinct chips, not deduped.
        assert_eq!(keyword_strip(&[Keyword::Rampage(1), Keyword::Rampage(2)]), "Rmp1 Rmp2");
        // Unleash surfaces its own marker chip (CR 702.98).
        assert_eq!(keyword_strip(&[Keyword::Unleash]), "Unl");
    }

    #[test]
    fn strip_surfaces_must_attack_and_crew() {
        assert_eq!(keyword_strip(&[Keyword::MustAttack]), "Atk!");
        assert_eq!(keyword_strip(&[Keyword::Crew(2)]), "Crew2");
    }

    #[test]
    fn strip_surfaces_upkeep_and_countdown_obligations() {
        use crabomination::mana::cost;
        assert_eq!(keyword_strip(&[Keyword::Echo(cost(&[]))]), "Echo");
        assert_eq!(keyword_strip(&[Keyword::Fading(3)]), "Fade");
        assert_eq!(keyword_strip(&[Keyword::Vanishing(2)]), "Vanish");
    }

    #[test]
    fn strip_surfaces_conditional_attacker() {
        use crabomination::card::SelectionRequirement;
        assert_eq!(
            keyword_strip(&[Keyword::CanAttackOnlyIfDefenderControls(Box::new(
                SelectionRequirement::HasLandType(crabomination::card::LandType::Island)
            ))]),
            "Atk?"
        );
    }

    #[test]
    fn board_status_prefixes_suspected_and_sick() {
        // A suspected creature shows "Susp" ahead of its (injected) Men/NoBlk.
        assert_eq!(
            board_status_strip(&[Keyword::Menace, Keyword::CantBlock], false, true, false, false, None, None, false, 0, 0, false),
            "Susp Men NoBlk",
        );
        // Summoning sickness tags "Zzz"; Haste suppresses it.
        assert_eq!(board_status_strip(&[], true, false, false, false, None, None, false, 0, 0, false), "Zzz");
        assert_eq!(board_status_strip(&[Keyword::Haste], true, false, false, false, None, None, false, 0, 0, false), "Hst");
        // Both statuses stack, suspected first.
        assert_eq!(board_status_strip(&[], true, true, false, false, None, None, false, 0, 0, false), "Susp Zzz");
        assert_eq!(board_status_strip(&[], false, false, false, false, None, None, false, 0, 0, false), "");
    }

    #[test]
    fn board_status_surfaces_goaded() {
        // A goaded creature flags "Goad" after suspected, before its keywords.
        assert_eq!(board_status_strip(&[], false, false, true, false, None, None, false, 0, 0, false), "Goad");
        assert_eq!(
            board_status_strip(&[Keyword::Menace], false, true, true, false, None, None, false, 0, 0, false),
            "Susp Goad Men",
        );
    }

    #[test]
    fn board_status_surfaces_detained() {
        // A detained permanent flags "Detain" after Goad (both are opponent-
        // imposed combat locks).
        assert_eq!(board_status_strip(&[], false, false, false, true, None, None, false, 0, 0, false), "Detain");
        assert_eq!(
            board_status_strip(&[Keyword::Flying], false, false, true, true, None, None, false, 0, 0, false),
            "Goad Detain Fly",
        );
    }

    #[test]
    fn board_status_surfaces_saddled() {
        // A saddled Mount flags "Sdl✓" (active state) after Goad, distinct from
        // the "Sdl N" cost chip that comes from its Saddle keyword.
        assert_eq!(board_status_strip(&[], false, false, false, false, None, None, true, 0, 0, false), "Sdl✓");
        assert_eq!(
            board_status_strip(&[Keyword::Saddle(3)], false, false, false, false, None, None, true, 0, 0, false),
            "Sdl✓ Sdl3",
        );
    }

    #[test]
    fn board_status_shows_case_solve_state() {
        // An unsolved Case reads "Case"; a solved one reads "Solved".
        assert_eq!(board_status_strip(&[], false, false, false, false, Some(false), None, false, 0, 0, false), "Case");
        assert_eq!(board_status_strip(&[], false, false, false, false, Some(true), None, false, 0, 0, false), "Solved");
    }

    #[test]
    fn board_status_shows_class_level() {
        // A Class enchantment reads "Lvl N".
        assert_eq!(board_status_strip(&[], false, false, false, false, None, Some(1), false, 0, 0, false), "Lvl 1");
        assert_eq!(board_status_strip(&[], false, false, false, false, None, Some(3), false, 0, 0, false), "Lvl 3");
    }

    #[test]
    fn board_status_shows_crew_count() {
        // A Vehicle crewed by two creatures this turn reads "Crew×2".
        assert_eq!(board_status_strip(&[], false, false, false, false, None, None, false, 2, 0, false), "Crew×2");
        // No crewers → no badge.
        assert_eq!(board_status_strip(&[], false, false, false, false, None, None, false, 0, 0, false), "");
    }

    #[test]
    fn board_status_shows_stun_counters() {
        // Stun counters (CR 122.1c — skip that many untaps) read as "Stun N",
        // sitting before the "Zzz" summoning-sickness tag.
        assert_eq!(board_status_strip(&[], false, false, false, false, None, None, false, 0, 2, false), "Stun 2");
        assert_eq!(board_status_strip(&[], true, false, false, false, None, None, false, 0, 1, false), "Stun 1 Zzz");
        // No stun → no badge.
        assert_eq!(board_status_strip(&[], false, false, false, false, None, None, false, 0, 0, false), "");
    }

    #[test]
    fn strip_surfaces_board_state_keywords() {
        use crabomination::mana::cost;
        assert_eq!(keyword_strip(&[Keyword::Devoid]), "Dvd");
        assert_eq!(keyword_strip(&[Keyword::Daybound]), "Day");
        assert_eq!(keyword_strip(&[Keyword::Nightbound]), "Night");
        assert_eq!(keyword_strip(&[Keyword::Disguise(cost(&[]))]), "Dsg");
    }
}
