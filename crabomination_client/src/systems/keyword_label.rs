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
//! displayable keywords or leaves the battlefield). It sits on the card's
//! P/T line, ending at the P/T badge (or the printed box), and renders below
//! default-z UI so popups / tooltips win.
//!
//! The bottom of a card is the part a card in front of it leaves showing —
//! a back-row creature's, one under a wrapped row — and on the card no
//! neighbour's overlays land. The strip hung above the card's top edge as
//! seen, which is where the card behind it peeks out: an opponent's front
//! row printed its strips over the P/T badges of the back row.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::{CardId, Keyword, WardCost};

use crate::card::{BattlefieldCard, GameCardId, CARD_WIDTH};
use crate::systems::pt_label::{PT_BOX, PtLabel};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::UiFonts;
use crate::MainCamera;

/// Renders below default-z (0) UI so popups / tooltips / modals win — same
/// band as the P/T badge.
const KW_Z: i32 = crate::theme::layer::CARD_OVERLAY;
/// Between the strip and the P/T badge it ends at.
const KW_GAP: f32 = 3.0;

/// Which way a strip runs from where it ends, by where the card's left
/// (local −X) falls on screen: an opponent's cards face them, and a tapped
/// card's P/T line runs up or down the screen.
fn strip_run(card_left: Vec2) -> Val2 {
    if card_left.x.abs() >= card_left.y.abs() {
        if card_left.x < 0.0 { Val2::percent(-100.0, -50.0) } else { Val2::percent(0.0, -50.0) }
    } else if card_left.y < 0.0 {
        Val2::percent(-50.0, -100.0)
    } else {
        Val2::percent(-50.0, 0.0)
    }
}

/// The strip's font size on a card `card_width` UI px across.
fn strip_font_size(card_width: f32) -> f32 {
    (card_width * 0.1).round().clamp(12.0, 26.0)
}

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
        ProtectionFromOwnColors => "Pro-own",
        ProtectionFromColorsOutsideCommanderIdentity => "Pro-off-identity",
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
        // Sunweb — the low-power mirror: it can't stop small attackers.
        CantBlockPowerAtMost(_) => "NoBlk≤",
        // Gibbering Hyenas — can't block a whole class of attacker.
        CantBlockMatching(_) => "NoBlk*",
        // "Can't attack" (Pacifism / Cage of Hands) and the conditional
        // Goblin-Cohort lock both read at a glance on the board.
        CantAttack => "NoAtk",
        CantAttackUnlessCastCreatureThisTurn => "Atk?",
        // Harbor Serpent's land-count gate and Bloodcrazed Goblin's first-blood
        // gate — both read at a glance next to the other attack locks.
        CantAttackUnlessLandCount(_, _) => "Land?",
        CantAttackUnlessOpponentDamaged => "Blood?",
        CantAttackUnlessMoreLandsThanDefender => "Land lead (atk)",
        CantBlockUnlessMoreLandsThanAttacker => "Land lead (blk)",
        // Mogg Toady's "more creatures than the other player" gates.
        CantAttackUnlessMoreCreaturesThanDefender => "Horde?",
        CantBlockUnlessMoreCreaturesThanAttacker => "NoBlkHorde?",
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
        BandsWithOther(_) => "Bnd+",
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
        // CR 725 — "can't attack unless defending player is the monarch"
        // (Crown-Hunter Hireling): the crown decides whether it can swing.
        CantAttackUnlessDefenderIsMonarch => "Atk👑",
        // "Can't attack/block unless you control N or more [filter]" (Topiary
        // Stomper, Lambholt Pacifist, Olog-hai Crusher) — surface which side is
        // gated so the player sees why it can't be declared.
        CantAttackOrBlockUnlessYouControlCount { attack_only, block_only, .. } => {
            if *attack_only { "Atk?" } else if *block_only { "Blk?" } else { "A/B?" }
        }
        // "Can't attack or block unless it has an even number of counters on it"
        // (Sab-Sunen) — a live combat gate that flips as counters change.
        CantAttackOrBlockUnlessEvenCounters => "Even?",
        // Attack tolls the controller has to cover before the swing is legal
        // (Brainwash's {N}, Leviathan's two Islands) — the player needs to see
        // the price before declaring, not after the rejection.
        CantAttackUnlessPay(_) | AttackCostSacrifice(_, _) => "Atk$",
        // "Doesn't untap if it attacked during your last turn" (Goblin Rock
        // Sled, Tangle Kelp) — the swing costs a whole turn of availability.
        DoesntUntapIfAttackedLastTurn => "Rest",
        // "Can't be destroyed by lethal damage unless one source dealt it all"
        // (Ogre Enforcer) — chump-blocking maths change completely.
        SurvivesSplitLethalDamage => "1Src",
        // "Can't be the target of spells unless it attacked or blocked this
        // turn" (Lurker) — reads as conditional shroud to the opponent.
        CantBeTargetedBySpellsUnlessAttackedOrBlocked => "Shr?",
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
        Morph(_) | MorphCost(_) | Megamorph(_) => "Mph",
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
        Discard(n) | DiscardMatching(_, n) => format!("{n}↓"),
        DiscardRandom(n) => format!("{n}↓?"),
        DiscardHand => "hand↓".into(),
        Blight(n) => format!("{n}☠"),
        GenericCountersOnSource(_) => "◆".into(),
        CollectEvidence(n) => format!("ev{n}"),
        ExileFromGraveyard(n) => format!("{n}gy⌫"),
        BottomFromGraveyard(n) => format!("{n}gy↓"),
        DamageFromSource(n) => format!("{n}🗲"),
        SacrificeCreature | SacrificeMatching(_) => "sac".into(),
        SacrificePermanents(n) | SacrificeMatchingN(_, n) => format!("sac{n}"),
        ReturnMatchingToHand(_, n) => format!("rtn{n}"),
        ExileTopFromGraveyardMatching(_) => "gy⌫".into(),
        ReturnMatchingFromGraveyardToHand(_) => "gy↑".into(),
        RemoveCounterFromPermanent => "ctr-".into(),
        ManaCostOfAttached => "MC".into(),
        ManaOrLife(c, n) => format!("{}/{n}♥", c.cmc()),
        SacrificeAttachedHost => "sacE".into(),
        GenericSourcePower => "P".into(),
        GenericXFromCost => "X".into(),
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
        | CantBlockPowerAtMost(n) | CantBeBlockedByPowerAtLeast(n) => *n,
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

/// Compact strip tag for a goaded creature (CR 701.38): "Goad" when the
/// goaders are unknown, else "Goad←" and each goader's name cut to six
/// characters so a three-goader pod stays one short chip.
fn goad_tag(goaders: &[String]) -> String {
    if goaders.is_empty() {
        return "Goad".to_string();
    }
    let short: Vec<String> = goaders.iter().map(|n| n.chars().take(6).collect()).collect();
    format!("Goad←{}", short.join(","))
}

/// Full board-glance strip: the keyword chips plus status prefixes that aren't
/// keywords — "Susp" for a suspected creature (CR 701.60) and "Zzz" for
/// summoning sickness. Empty when there's nothing to show.
fn board_status_strip(
    keywords: &[Keyword],
    summoning_sick: bool,
    suspected: bool,
    goaders: Option<&[String]>,
    detained: bool,
    case_solved: Option<bool>,
    class_level: Option<u8>,
    room_doors: &[(String, bool)],
    saddled: bool,
    crewed_count: u32,
    stun: u32,
    wont_untap: bool,
    attack_mandated: bool,
    attack_benched: bool,
    must_block: bool,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    // CR 716 Class: show the current level. Leads the strip so the level reads
    // at a glance on the non-creature enchantment.
    if let Some(n) = class_level {
        parts.push(format!("Lvl {n}"));
    }
    // CR 709.5c Room: how many of the two doors are open, so the board read
    // matches what the unlock action can still target.
    if !room_doors.is_empty() {
        let open = room_doors.iter().filter(|(_, unlocked)| *unlocked).count();
        parts.push(format!("Doors {open}/{}", room_doors.len()));
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
    // In a pod the goaders are named ("Goad←Bob"): CR 701.38b steers the
    // creature away from *those* seats, so "Goad" alone doesn't say where
    // it will be sent.
    if let Some(names) = goaders {
        parts.push(goad_tag(names));
    }
    // Oracle en-Vec's mandate — an opponent has named who attacks on this
    // creature's controller's next turn. Same family of opponent-imposed
    // combat compulsions as Goad, so it sits beside it: named creatures must
    // attack (or die at end of turn), the rest can't attack at all.
    if attack_mandated {
        parts.push("Must!".to_string());
    } else if attack_benched {
        parts.push("Benched".to_string());
    }
    // CR 509.1c — conscripted into blocking one specific attacker this turn.
    // The same opponent-imposed-combat family as Goad and the mandate.
    if must_block {
        parts.push("Blk!".to_string());
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
    // it stays tapped-out of future combats/activations. A real board read, so
    // it sits by the "Zzz" can't-act tag — and it is the stun counters' only
    // readout on a card with a strip (`counter_coins` leaves them out).
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

/// Whether `p` gets a status strip: creatures get the keyword/status strip;
/// Cases, Classes and Rooms (non-creatures) get a state chip.
pub(crate) fn has_status_strip(p: &crabomination::net::PermanentView) -> bool {
    p.is_creature() || p.case_solved.is_some() || p.class_level.is_some() || !p.room_doors.is_empty()
}

/// Reconcile keyword strips with the engine view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity)]
pub fn sync_keyword_labels(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    cards: Query<(Entity, &GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    cover_cards: crate::card::cover::CoverQuery,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_scale: Res<UiScale>,
    badges: Query<(&PtLabel, &Node, &ComputedNode), Without<KeywordLabel>>,
    mut labels: Query<(Entity, &KeywordLabel, &mut Node, &mut Text, &mut TextFont, &mut UiTransform)>,
    mut desired_cache: Local<HashMap<CardId, String>>,
) {
    // No view (between matches): clear every strip and bail.
    let Some(cv) = &view.0 else {
        for (e, ..) in &mut labels {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    // Desired strips: creatures with at least one displayable keyword.
    // Rebuilt only on view change (keyword_strip allocates a String per
    // creature); anchoring/positioning below still tracks every frame, and
    // ids without a live entity yet are handled at use time (hidden or
    // parked offscreen until the anchor exists).
    if view.is_changed() {
        desired_cache.clear();
        for p in &cv.battlefield {
            if !has_status_strip(p) {
                continue;
            }
            let stun = p
                .counters
                .iter()
                .find(|(k, _)| *k == crabomination::card::CounterType::Stun)
                .map(|(_, n)| *n)
                .unwrap_or(0);
            // Goader seats → names ("You" for the viewer). A goaded flag
            // with no seats (an older server) still reads as plain "Goad".
            let goader_names: Option<Vec<String>> = p.goaded.then(|| {
                p.goaded_by
                    .iter()
                    .map(|&s| {
                        crate::systems::game_ui::table_awareness::seat_label(
                            &cv.players,
                            cv.your_seat,
                            s,
                        )
                    })
                    .collect()
            });
            let strip = board_status_strip(
                &p.keywords,
                p.summoning_sick,
                p.suspected,
                goader_names.as_deref(),
                p.detained,
                p.case_solved,
                p.class_level,
                &p.room_doors,
                p.saddled,
                p.crewed_count,
                stun,
                p.wont_untap,
                p.attack_mandated,
                p.attack_benched,
                p.must_block,
            );
            if !strip.is_empty() {
                desired_cache.insert(p.id, strip);
            }
        }
    }
    let desired = &*desired_cache;

    // The P/T badges' laid-out sizes, UI px — a frame behind, which the
    // badge settles in.
    let badge_size: HashMap<CardId, Vec2> = badges
        .iter()
        .filter(|(_, node, _)| node.display != Display::None)
        .map(|(badge, _, computed)| (badge.0, computed.size() * computed.inverse_scale_factor()))
        .collect();

    // card_id → (where the strip ends, which way it runs, its font size).
    // A strip hides while another card lies over that line (`card::cover`):
    // it printed on the card on top, whose keywords it then seemed to be.
    let cover = crate::card::cover::CardCover::new(cam_xform, &cover_cards);
    let mut placed: HashMap<CardId, (Vec2, Val2, f32)> = HashMap::new();
    for (e, gid, gtf) in &cards {
        if !desired.contains_key(&gid.0) || cover.hides_local(e, gtf, PT_BOX - Vec3::X * CARD_WIDTH * 0.3) {
            continue;
        }
        let project =
            |local: Vec3| crate::theme::project_to_ui(camera, cam_xform, &ui_scale, gtf.transform_point(local));
        let on_line = |x: f32| project(Vec3::new(x, PT_BOX.y, 0.0));
        let (Some(pt), Some(left_edge), Some(right_edge)) =
            (project(PT_BOX), on_line(-CARD_WIDTH / 2.0), on_line(CARD_WIDTH / 2.0))
        else {
            continue;
        };
        let card_left = (left_edge - right_edge).normalize_or_zero();
        let card_width = left_edge.distance(right_edge);
        // Half the badge along the way the strip runs; the printed box's
        // when the P/T is as printed and there is no badge.
        let badge = badge_size.get(&gid.0).copied();
        let half = if card_left.x.abs() >= card_left.y.abs() {
            badge.map_or(card_width * 0.075, |b| b.x / 2.0)
        } else {
            badge.map_or(card_width * 0.05, |b| b.y / 2.0)
        };
        let end = pt + card_left * (half + KW_GAP);
        placed.insert(gid.0, (end, strip_run(card_left), strip_font_size(card_width)));
    }

    // Update existing strips; despawn any whose creature lost all keywords or
    // left the battlefield.
    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, label, mut node, mut text, mut font, mut transform) in &mut labels {
        let Some(strip) = desired.get(&label.0) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(label.0);
        if text.0 != *strip {
            *text = Text::new(strip.clone());
        }
        let Some(&(end, run, size)) = placed.get(&label.0) else {
            crate::theme::place_overlay(&mut node, None);
            continue;
        };
        crate::theme::place_overlay(&mut node, Some(end));
        if transform.translation != run {
            transform.translation = run;
        }
        let size = FontSize::Px(size);
        if font.font_size != size {
            font.font_size = size;
        }
    }

    // Spawn strips for newly-keyworded creatures, parked off-screen until
    // the next frame places them.
    for (id, strip) in desired.iter() {
        if seen.contains(id) {
            continue;
        }
        commands.spawn((
            KeywordLabel(*id),
            crate::systems::focus::CardOverlay(*id),
            Text::new(strip.clone()),
            ui_fonts.tf(12.0),
            TextColor(Color::srgb(0.96, 0.94, 0.80)),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.72)),
            // One line: a strip that wrapped stood taller than the P/T line.
            TextLayout::no_wrap(),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(-1000.0),
                top: Val::Px(-1000.0),
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
    use super::{board_status_strip, goad_tag, keyword_strip, req_short};
    use crabomination::card::Keyword;

    #[test]
    fn a_strip_runs_toward_the_cards_left_as_seen() {
        use super::strip_run;
        use bevy::prelude::{Val2, Vec2};
        // Upright: the card's left is screen-left, so the strip ends at the
        // P/T badge and runs left of it.
        assert_eq!(strip_run(Vec2::new(-1.0, 0.1)), Val2::percent(-100.0, -50.0));
        // An opponent's card faces them: its left is screen-right.
        assert_eq!(strip_run(Vec2::new(1.0, -0.1)), Val2::percent(0.0, -50.0));
        // A tapped card's P/T line runs up or down the screen.
        assert_eq!(strip_run(Vec2::new(0.1, -1.0)), Val2::percent(-50.0, -100.0));
        assert_eq!(strip_run(Vec2::new(-0.1, 1.0)), Val2::percent(-50.0, 0.0));
    }

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
            board_status_strip(&[Keyword::Menace, Keyword::CantBlock], false, true, None, false, None, None, &[], false, 0, 0, false, false, false, false),
            "Susp Men NoBlk",
        );
        // Summoning sickness tags "Zzz"; Haste suppresses it.
        assert_eq!(board_status_strip(&[], true, false, None, false, None, None, &[], false, 0, 0, false, false, false, false), "Zzz");
        assert_eq!(board_status_strip(&[Keyword::Haste], true, false, None, false, None, None, &[], false, 0, 0, false, false, false, false), "Hst");
        // Both statuses stack, suspected first.
        assert_eq!(board_status_strip(&[], true, true, None, false, None, None, &[], false, 0, 0, false, false, false, false), "Susp Zzz");
        assert_eq!(board_status_strip(&[], false, false, None, false, None, None, &[], false, 0, 0, false, false, false, false), "");
    }

    #[test]
    fn board_status_surfaces_goaded() {
        // A goaded creature flags "Goad" after suspected, before its keywords.
        assert_eq!(board_status_strip(&[], false, false, Some(&[]), false, None, None, &[], false, 0, 0, false, false, false, false), "Goad");
        assert_eq!(
            board_status_strip(&[Keyword::Menace], false, true, Some(&[]), false, None, None, &[], false, 0, 0, false, false, false, false),
            "Susp Goad Men",
        );
        // CR 701.38b — in a pod the goaders are named, so the strip says
        // which seats the creature is steered away from.
        let bob = ["Bob".to_string()];
        assert_eq!(
            board_status_strip(&[], false, false, Some(&bob), false, None, None, &[], false, 0, 0, false, false, false, false),
            "Goad←Bob",
        );
        // Several goaders, long names cut to six characters.
        let two = ["Carolina".to_string(), "You".to_string()];
        assert_eq!(goad_tag(&two), "Goad←Caroli,You");
    }

    #[test]
    fn board_status_surfaces_room_doors() {
        // A Room leads with how many of its two doors stand open.
        assert_eq!(
            board_status_strip(
                &[],
                false,
                false,
                None,
                false,
                None,
                None,
                &[("Prop Room".to_string(), true), ("Dazzling Theater".to_string(), false)],
                false,
                0,
                0,
                false,
                false,
                false,
                false,
            ),
            "Doors 1/2",
        );
    }

    #[test]
    fn board_status_surfaces_detained() {
        // A detained permanent flags "Detain" after Goad (both are opponent-
        // imposed combat locks).
        assert_eq!(board_status_strip(&[], false, false, None, true, None, None, &[], false, 0, 0, false, false, false, false), "Detain");
        assert_eq!(
            board_status_strip(&[Keyword::Flying], false, false, Some(&[]), true, None, None, &[], false, 0, 0, false, false, false, false),
            "Goad Detain Fly",
        );
    }

    #[test]
    fn board_status_surfaces_saddled() {
        // A saddled Mount flags "Sdl✓" (active state) after Goad, distinct from
        // the "Sdl N" cost chip that comes from its Saddle keyword.
        assert_eq!(board_status_strip(&[], false, false, None, false, None, None, &[], true, 0, 0, false, false, false, false), "Sdl✓");
        assert_eq!(
            board_status_strip(&[Keyword::Saddle(3)], false, false, None, false, None, None, &[], true, 0, 0, false, false, false, false),
            "Sdl✓ Sdl3",
        );
    }

    #[test]
    fn board_status_shows_case_solve_state() {
        // An unsolved Case reads "Case"; a solved one reads "Solved".
        assert_eq!(board_status_strip(&[], false, false, None, false, Some(false), None, &[], false, 0, 0, false, false, false, false), "Case");
        assert_eq!(board_status_strip(&[], false, false, None, false, Some(true), None, &[], false, 0, 0, false, false, false, false), "Solved");
    }

    #[test]
    fn board_status_shows_class_level() {
        // A Class enchantment reads "Lvl N".
        assert_eq!(board_status_strip(&[], false, false, None, false, None, Some(1), &[], false, 0, 0, false, false, false, false), "Lvl 1");
        assert_eq!(board_status_strip(&[], false, false, None, false, None, Some(3), &[], false, 0, 0, false, false, false, false), "Lvl 3");
    }

    #[test]
    fn board_status_shows_crew_count() {
        // A Vehicle crewed by two creatures this turn reads "Crew×2".
        assert_eq!(board_status_strip(&[], false, false, None, false, None, None, &[], false, 2, 0, false, false, false, false), "Crew×2");
        // No crewers → no badge.
        assert_eq!(board_status_strip(&[], false, false, None, false, None, None, &[], false, 0, 0, false, false, false, false), "");
    }

    #[test]
    fn board_status_shows_stun_counters() {
        // Stun counters (CR 122.1c — skip that many untaps) read as "Stun N",
        // sitting before the "Zzz" summoning-sickness tag.
        assert_eq!(board_status_strip(&[], false, false, None, false, None, None, &[], false, 0, 2, false, false, false, false), "Stun 2");
        assert_eq!(board_status_strip(&[], true, false, None, false, None, None, &[], false, 0, 1, false, false, false, false), "Stun 1 Zzz");
        // No stun → no badge.
        assert_eq!(board_status_strip(&[], false, false, None, false, None, None, &[], false, 0, 0, false, false, false, false), "");
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
