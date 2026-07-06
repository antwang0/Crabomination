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
use crabomination::card::{CardId, Keyword};

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
        Shroud => "Shr",
        Unblockable => "Unb",
        Intimidate => "Int",
        Fear => "Fear",
        Infect => "Inf",
        Wither => "Wth",
        Skulk => "Skk",
        Shadow => "Shd",
        Horsemanship => "Hrs",
        Landwalk(_) => "Wlk",
        Protection(_) => "Pro",
        ProtectionFromManaValueExcept(_) => "ProMV",
        ProtectionFromManaValueParity { odd } => if *odd { "Pro-odd" } else { "Pro-even" },
        ProtectionFromMulticolored => "ProMC",
        ProtectionFromInstants => "ProI",
        ProtectionFromEverything => "Pro★",
        Ward(_) => "Ward",
        Toxic(_) => "Tox",
        Poisonous(_) => "Psn",
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
        // Hazoret-class hellbent gate reads at a glance on the board.
        CantAttackOrBlockUnlessHandSizeAtMost(_) => "Hand?",
        CantAttackOrBlockUnlessDelirium => "Dlr?",
        CantAttackOrBlockUnlessCreatureDiedThisTurn => "Died?",
        CantAttackOrBlockUnlessDescend(_) => "Dsc?",
        CantAttackOrBlockUnlessCityBlessing => "Bless?",
        Decayed => "Dcy",
        Flanking => "Flk",
        // Combat-pump statics from the Kamigawa/legacy sets read at a glance.
        Bushido(_) => "Bsd",
        Rampage(_) => "Rmp",
        Frenzy(_) => "Frz",
        Banding => "Bnd",
        // Generalized menace — "can't be blocked except by N or more."
        CantBeBlockedExceptByN(_) => "Men+",
        // Evasion by blocker quality: "can only be blocked by [filter]"
        // (Serpent of Yawning Depths) or "can't be blocked by [filter]"
        // (Temple Thief) — both read as evasion at a glance.
        CantBeBlockedExceptBy(_) | CantBeBlockedBy(_) => "Eva",
        // "Can't be blocked by more than one creature" (anti-gang-block).
        CantBeBlockedByMoreThanOne => "1Blk",
        // Power-gated evasion — "can't be blocked by creatures with power
        // less than this" (Formation Breaker) / "power N or less" (Questing
        // Beast).
        CantBeBlockedByPowerLess | CantBeBlockedByPowerAtMost(_) => "Eva",
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
        _ => return None,
    })
}

/// The numeric magnitude worth appending to a count-carrying keyword's tag —
/// the N in Rampage N / Toxic N / Annihilator N etc. materially changes how the
/// creature reads in combat, so surface it ("Rmp2", "Tox3") rather than dropping
/// it. Cost-carrying keywords (Ward, Crew) aren't plain integers, so skip them.
fn keyword_value_suffix(kw: &Keyword) -> Option<String> {
    use Keyword::*;
    let n = match kw {
        Rampage(n) | Bushido(n) | Frenzy(n) | Annihilator(n) | Absorb(n) | Toxic(n)
        | Poisonous(n) | CantBeBlockedExceptByN(n) => *n,
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
            if !p.is_creature() {
                continue;
            }
            let strip = keyword_strip(&p.keywords);
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
    use super::keyword_strip;
    use crabomination::card::Keyword;

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
    fn strip_surfaces_resilience_and_status_keywords() {
        assert_eq!(keyword_strip(&[Keyword::Persist]), "Per");
        assert_eq!(keyword_strip(&[Keyword::Undying]), "Und");
        assert_eq!(keyword_strip(&[Keyword::Annihilator(2)]), "Ann2");
        assert_eq!(keyword_strip(&[Keyword::Changeling]), "Chg");
        assert_eq!(keyword_strip(&[Keyword::Prowess]), "Prw");
        assert_eq!(keyword_strip(&[Keyword::FirebendingPower]), "FB");
        assert_eq!(keyword_strip(&[Keyword::Crew(2)]), "Crew");
        assert_eq!(keyword_strip(&[Keyword::Saddle(3)]), "Sdl");
        assert_eq!(keyword_strip(&[Keyword::Regenerate(0)]), "Rgn");
        assert_eq!(keyword_strip(&[Keyword::UmbraArmor]), "TArm");
        assert_eq!(keyword_strip(&[Keyword::ProtectionFromCreatures]), "ProCr");
        assert_eq!(keyword_strip(&[Keyword::CantAttackAlone]), "Pack");
        assert_eq!(keyword_strip(&[Keyword::CantAttackOrBlockAlone]), "Pack");
    }

    #[test]
    fn strip_surfaces_generalized_menace_and_lure() {
        assert_eq!(keyword_strip(&[Keyword::CantBeBlockedExceptByN(3)]), "Men+3");
        assert_eq!(keyword_strip(&[Keyword::MustBeBlocked]), "Lure");
    }

    #[test]
    fn strip_surfaces_block_quality_evasion() {
        use crabomination::card::SelectionRequirement;
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedExceptBy(Box::new(
                SelectionRequirement::HasKeyword(Keyword::Flying),
            ))]),
            "Eva"
        );
        assert_eq!(
            keyword_strip(&[Keyword::CantBeBlockedBy(Box::new(SelectionRequirement::Enchantment))]),
            "Eva"
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
    }

    #[test]
    fn strip_surfaces_must_attack_and_crew() {
        assert_eq!(keyword_strip(&[Keyword::MustAttack]), "Atk!");
        assert_eq!(keyword_strip(&[Keyword::Crew(2)]), "Crew");
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
}
