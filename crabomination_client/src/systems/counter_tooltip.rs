//! Alt-key tooltip showing counter detail and modified P/T for the
//! battlefield card under the cursor.
//!
//! Hold either Alt (left/right) while hovering a card to surface a small
//! HUD panel with:
//!
//! - Current power/toughness (if creature) and `(printed X/Y)`
//!   when the values differ from the printed P/T.
//! - Loyalty count (for planeswalkers).
//! - One row per counter type and quantity (`+1/+1 ×3`, `Stun ×2`, …).
//!
//! The 3-D counter coins handle the at-a-glance "this card has stuff on
//! it" indicator; this tooltip is the click-through for the details
//! a player needs when the coin column gets dense.
//!
//! Anchored to the bottom-right corner of the viewport rather than
//! floating next to the 3-D card, because the peek popup
//! (`systems::ui::peek_popup`) also lights up on Alt-hold and centers a
//! large card-art image — a card-adjacent tooltip would overlap it.

use bevy::prelude::*;
use crabomination::card::{CardId, CardType, CounterType};

use crate::card::{BattlefieldCard, CardHovered, GameCardId};
use crate::net_plugin::CurrentView;
use crate::theme::UiFonts;

/// Root marker for the floating tooltip panel.
#[derive(Component)]
pub struct AltTooltip;

/// Marker on the tooltip's text node so the update system can rewrite it
/// without doing a child walk.
#[derive(Component)]
pub struct AltTooltipText;

#[allow(clippy::too_many_arguments)]
pub fn update_alt_tooltip(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    hovered: Query<&GameCardId, (With<BattlefieldCard>, With<CardHovered>)>,
    mut tooltip_q: Query<Entity, With<AltTooltip>>,
    mut text_q: Query<&mut Text, With<AltTooltipText>>,
) {
    let alt_held = keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight);

    // No alt or no hovered card → tear down any tooltip.
    let hovered_card_id: Option<CardId> = if alt_held {
        hovered.iter().next().map(|gid| gid.0)
    } else {
        None
    };

    let Some(card_id) = hovered_card_id else {
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    let Some(cv) = &view.0 else { return };
    let Some(p) = cv.battlefield.iter().find(|p| p.id == card_id) else {
        // Card left the battlefield — drop the tooltip.
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };

    // Build the body without the card name (the peek popup already
    // shows the card art with its name). If there's nothing
    // interesting (no P/T mod, no loyalty, no counters, not tapped),
    // skip the tooltip entirely so we don't render an empty panel.
    let Some(mut body) = build_tooltip_body(p) else {
        for e in tooltip_q.iter() {
            commands.entity(e).despawn();
        }
        return;
    };
    // The peek art can't show the *board* context that makes the legend rule
    // (CR 704.5j) bite. When this legendary permanent has a same-named,
    // same-controller twin already in play, warn that one is about to die.
    if legend_rule_at_risk(&cv.battlefield, p) {
        body.push_str("\n⚠ legend rule: another copy is in play");
    }

    if tooltip_q.single_mut().is_ok() {
        // Existing tooltip — just refresh its text.
        if let Ok(mut text) = text_q.single_mut()
            && text.0 != body
        {
            text.0 = body;
        }
        return;
    }

    // Spawn fresh tooltip pinned to the bottom-right corner so it
    // never overlaps the centered peek-popup card art.
    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.06, 0.12, 0.93)),
            AltTooltip,
            crate::systems::game_ui::InGameRoot,
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(body),
            ui_fonts.tf(13.0),
            TextColor(Color::srgba(0.95, 0.95, 1.0, 1.0)),
            AltTooltipText,
            Pickable::IGNORE,
        ));
    });
}

/// True when this legendary permanent shares its name and controller with
/// another legendary permanent already on the battlefield — the pair the legend
/// rule (CR 704.5j) will collapse. `is_legendary` reads the *computed*
/// supertype server-side, so a Leyline-of-Singularity grant counts too.
fn legend_rule_at_risk(
    battlefield: &[crabomination::net::PermanentView],
    p: &crabomination::net::PermanentView,
) -> bool {
    p.is_legendary
        && battlefield.iter().any(|q| {
            q.id != p.id && q.controller == p.controller && q.name == p.name && q.is_legendary
        })
}

/// Build the tooltip body. Returns `None` when the card has nothing
/// the peek-popup art doesn't already show — we don't want a tiny
/// dark panel popping up just to repeat "this is a creature with 2/2"
/// while the user is looking at the full card art.
fn build_tooltip_body(p: &crabomination::net::PermanentView) -> Option<String> {
    let mut lines = Vec::new();

    // CR 708 — face-down permanents (morph / manifest) render as a 2/2. Show
    // the hidden status, and — for the controller, who may look at their own
    // face-down cards (708.2) — the real card's name the server revealed.
    if p.face_down {
        match &p.face_down_name {
            Some(name) => lines.push(format!("Face-down 2/2 (yours: {name})")),
            None => lines.push("Face-down 2/2".to_string()),
        }
    }

    // P/T summary — only if modified, since the peek popup shows the
    // printed P/T as part of the card art.
    if p.card_types.contains(&CardType::Creature)
        && (p.power != p.base_power || p.toughness != p.base_toughness)
    {
        lines.push(format!(
            "{}/{}  (printed {}/{})",
            p.power, p.toughness, p.base_power, p.base_toughness
        ));
    }

    // Creature type-line: surface tribal context ("Inkling Wizard",
    // "Pest", "Spirit Warrior") so the player can see at a glance
    // which tribal anthems / dies-trigger groups this body feeds.
    // Only render when there are creature types to show (filters out
    // colorless artifacts and non-creature permanents). Push
    // (claude/modern_decks batch 198).
    if p.card_types.contains(&CardType::Creature) && !p.creature_types.is_empty() {
        lines.push(format!("Type: {}", p.creature_types.join(" ")));
    }

    // Legendary marker (CR 704.5j legend rule) — the peek art shows the type
    // line, but on the crowded board a one-word reminder helps players spot a
    // second copy that's about to die to the legend rule.
    if p.is_legendary {
        lines.push(String::from("Legendary"));
    }

    // Chosen color for "choose-a-color" mana rocks (Coldsteel Heart): show
    // which color this source now taps for.
    if let Some(c) = p.chosen_color {
        let name = match c {
            crabomination::mana::Color::White => "White",
            crabomination::mana::Color::Blue => "Blue",
            crabomination::mana::Color::Black => "Black",
            crabomination::mana::Color::Red => "Red",
            crabomination::mana::Color::Green => "Green",
        };
        lines.push(format!("Taps for: {name}"));
    }

    // Auras / Equipment attached to this permanent (CR 303 / 301): list them
    // so the player sees what's enchanting/equipping a creature without
    // hovering each attachment.
    if !p.attachments.is_empty() {
        lines.push(format!("Attached: {}", p.attachments.join(", ")));
    }

    // For an Aura/Equipment, the host it's attached to (CR 301.5 / 303) — so a
    // player hovering the attachment itself sees what it's buffing.
    if let Some(host) = &p.attached_to_name {
        let verb = if p.card_types.contains(&CardType::Enchantment) { "Enchanting" } else { "Equipping" };
        lines.push(format!("{verb}: {host}"));
    } else if let Some(seat) = p.attached_to_player {
        // CR 303.4a — an "enchant player" Aura has no host permanent.
        lines.push(format!("Enchanting: player {}", seat + 1));
    }

    // Soulbond pairing (CR 702.95): flag the pair so the player sees the
    // creature is sharing its partner's bonus.
    if p.soulbond_partner.is_some() {
        lines.push("Soulbonded".to_string());
    }

    // Card name chosen by Pithing Needle / Phyrexian Revoker (CR 201.3).
    if let Some(name) = &p.named_card {
        lines.push(format!("Naming: {name}"));
    }

    // Creature type chosen at ETB (Cavern of Souls, Metallic Mimic, Adaptive
    // Automaton, Patchwork Banner) — drives the chosen-type anthem /
    // uncounterable / enters-with-counter riders.
    if let Some(ct) = &p.chosen_creature_type {
        lines.push(format!("Chosen type: {ct}"));
    }

    // CR 614 — persistent mode chosen at ETB (the Tarkir Siege cycle) drives
    // which arm of the two-mode enchantment is active.
    if let Some(mode) = &p.chosen_mode_label {
        lines.push(format!("Mode: {mode}"));
    }

    // Loyalty for planeswalkers (separate from counters list since it's
    // the headline number on every walker).
    if p.card_types.contains(&CardType::Planeswalker) {
        let loyalty = p
            .counters
            .iter()
            .find_map(|(k, v)| matches!(k, CounterType::Loyalty).then_some(*v))
            .unwrap_or(0);
        match p.loyalty_uses_remaining {
            Some(0) => lines.push(format!("Loyalty: {loyalty} (no activations left this turn)")),
            Some(n) if n > 1 => {
                lines.push(format!("Loyalty: {loyalty} ({n} activations this turn)"))
            }
            _ => lines.push(format!("Loyalty: {loyalty}")),
        }
        // List the walker's loyalty abilities with their signed cost, so a
        // hover shows "+1: Draw a card / -X: Make a token" without opening the
        // activator UI. Variable-X abilities render their cost as "-X".
        for a in &p.loyalty_abilities {
            let cost = if a.x_cost {
                "-X".to_string()
            } else if a.loyalty_cost >= 0 {
                format!("+{}", a.loyalty_cost)
            } else {
                a.loyalty_cost.to_string()
            };
            lines.push(format!("{cost}: {}", a.effect_label));
        }
    }

    // Counters (excluding loyalty, which we already broke out).
    let mut counters: Vec<(CounterType, u32)> = p
        .counters
        .iter()
        .filter(|(k, n)| !matches!(k, CounterType::Loyalty) && *n > 0)
        .map(|(k, n)| (*k, *n))
        .collect();
    counters.sort_by_key(|(k, _)| sort_key(*k));
    if !counters.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for (kind, n) in &counters {
            lines.push(format!("{} ×{}", counter_label(*kind), n));
        }
        // Dim reminder line per counter kind that has one, deduped — the
        // counter analogue of the keyword-reminder lines below, so newer
        // players see what "Stun" / "Finality" / "Luck" actually do.
        let mut reminders: Vec<String> = counters
            .iter()
            .filter_map(|(k, _)| counter_reminder(*k))
            .map(|r| format!("· {r}"))
            .collect();
        reminders.sort();
        reminders.dedup();
        for r in reminders {
            lines.push(r);
        }
    }
    // Split equip cost: the cheaper "Equip creature token {N}" line
    // (Team Pennant) — shown alongside the keyword list's "Equip {M}".
    if let Some(cost) = &p.equip_token_cost {
        lines.push(format!("Equip creature token {}", cost.summary()));
    }

    // Saddled (CR 702.171) — flag that the Mount's attacks-while-saddled riders
    // are armed for this combat.
    if p.saddled {
        lines.push("⛨ saddled".to_string());
    }

    // Station progress (CR 721) — show the next {N+} charge threshold so the
    // player can see how close the Spacecraft is to its next striation.
    if let Some(threshold) = p.station_next_threshold {
        let charges = p
            .counters
            .iter()
            .find(|(k, _)| matches!(k, CounterType::Charge))
            .map(|(_, n)| *n)
            .unwrap_or(0);
        lines.push(format!(
            "Station → {} ({} more)",
            threshold,
            threshold.saturating_sub(charges)
        ));
    }

    // Effective keywords (after layer effects). Show these so anthem
    // effects ("All your creatures have Lifelink", Inkling Verselord)
    // are visible even when the printed text doesn't include the
    // keyword. Keep the list compact — one line, comma-separated.
    if !p.keywords.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        let mut kw_strs: Vec<String> = p.keywords.iter().map(keyword_label).collect();
        kw_strs.sort();
        kw_strs.dedup();
        lines.push(kw_strs.join(", "));
        // Reminder text for the evergreen keywords on this permanent — one
        // dim line per keyword that has reminder text, deduped, so newer
        // players don't have to look up what "Deathtouch" or "Trample"
        // does. (Roadmap Tier 8 — reminder-text tooltips.)
        let mut reminders: Vec<String> = p
            .keywords
            .iter()
            .filter_map(keyword_reminder)
            .map(|r| format!("· {r}"))
            .collect();
        reminders.sort();
        reminders.dedup();
        for r in reminders {
            lines.push(r);
        }
    }
    // Split equip cost: the cheaper "Equip creature token {N}" line
    // (Team Pennant) — shown alongside the keyword list's "Equip {M}".
    if let Some(cost) = &p.equip_token_cost {
        lines.push(format!("Equip creature token {}", cost.summary()));
    }

    // Activated abilities — show the cost + effect label so players
    // can see "this Witherbloom Pledgemage has {1}{B}, Pay 1 life:
    // Draw a card" without clicking through to the activator UI.
    let ability_lines: Vec<String> = p
        .abilities
        .iter()
        .filter(|a| !a.is_mana)
        .map(|a| {
            let cost = if a.cost_label.is_empty() { "—".to_string() } else { a.cost_label.clone() };
            format!("{}: {}", cost, a.effect_label)
        })
        .collect();
    if !ability_lines.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in ability_lines {
            lines.push(l);
        }
    }

    // Triggered abilities — short event-prefixed labels exposed by the
    // server via `PermanentView.triggered_ability_labels`. Lets players
    // see "ETB: Draw a card", "Magecraft: Drain 1", "Dies: Mill 2"
    // without round-tripping through the card-text panel.
    if !p.triggered_ability_labels.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in &p.triggered_ability_labels {
            lines.push(l.clone());
        }
    }

    // Static abilities — printed Oracle wording exposed by the server
    // via `PermanentView.static_ability_labels`. Lets players see
    // "Other Inkling creatures you control get +2/+2." or "Each
    // opponent can't gain life." without scrolling through the card
    // detail panel. Added per the TODO improvement "Static-ability
    // tooltip exposure" — pairs with the existing
    // `triggered_ability_labels` block above. Also carries the
    // "Equipped: {cost}: {effect}" lines an Equipment grants via
    // `EquipBonus.activated_abilities` (Wrench's tap ability).
    if !p.static_ability_labels.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in &p.static_ability_labels {
            lines.push(l.clone());
        }
    }

    // Activated abilities — "{cost}: {effect}" summaries exposed by the server
    // via `PermanentView.activated_ability_labels`, so a hover shows what a
    // creature or artifact can *do* ("{2}{T}: Draw a card") without opening the
    // detail panel. The activated analogue of the trigger/static blocks above.
    if !p.activated_ability_labels.is_empty() {
        if !lines.is_empty() {
            lines.push(String::from("─────────"));
        }
        for l in &p.activated_ability_labels {
            lines.push(l.clone());
        }
    }

    if p.ward_cost > 0 {
        lines.push(format!("Ward {{{}}}", p.ward_cost));
    }

    // Counter-state highlights — surface the high-signal CR 122.1
    // counter states (shield/finality) that have engine effects
    // beyond their printed +1/+1 cousins. Helps the player see "this
    // creature absorbs one damage/destroy" or "this creature exiles
    // on death" without scrolling the counters list.
    // CR 122.1c shield counters: each absorbs one damage/destroy event.
    // Surface the per-counter count when N > 1 so the player sees how
    // many incoming events the creature can shrug off. Falls back to the
    // legacy boolean badge when the explicit count isn't populated
    // (older server projection / older snapshot).
    if p.shield_counter_count > 1 {
        lines.push(format!(
            "(shielded ×{}: absorbs {} damage/destroy events)",
            p.shield_counter_count, p.shield_counter_count
        ));
    } else if p.has_shield_counters {
        lines.push(String::from("(shielded: next damage/destroy is absorbed)"));
    }
    // CR 615 prevention shields (distinct from shield *counters*): a
    // protective shield soaks damage this turn, while a Kill-Suit Cultist
    // "destroy on next damage" shield is a death sentence, not protection.
    if p.doomed_next_damage {
        lines.push(String::from("(doomed: next damage destroys this instead)"));
    } else if p.has_prevention_shield {
        lines.push(format!("(warded: {})", prevention_summary(p.prevention_remaining, &p.prevention_source_colors)));
    }
    // CR 615.7 — the deal-side shield: this permanent's own damage is off for
    // the turn (Hallow, Burrenton Forge-Tender's chosen source).
    if p.damage_prevented_as_source {
        lines.push(String::from("(defanged: deals no damage this turn)"));
    }
    if p.finality_counter_count > 0 || p.has_finality_counters {
        lines.push(String::from("(finality: exiles instead of going to graveyard)"));
    } else if p.dies_to_exile {
        lines.push(String::from("(exiles instead of dying)"));
    }
    // CR 701.15 regeneration shields: each replaces the next destruction
    // this turn with a tap + heal + remove-from-combat. Surface the count
    // so the player knows how many destructions the creature can shrug off
    // before it actually dies.
    if p.cant_regenerate {
        // CR 701.15g — the shields are still on the permanent but inert, so
        // say so rather than promising a save it can no longer make.
        lines.push(String::from("(can't be regenerated this turn)"));
    } else if p.regeneration_shields > 1 {
        lines.push(format!(
            "(regen ×{}: absorbs {} destructions this turn)",
            p.regeneration_shields, p.regeneration_shields
        ));
    } else if p.regeneration_shields == 1 {
        lines.push(String::from(
            "(regen: next destruction taps & heals instead of dying)",
        ));
    }
    if p.stun_counter_count > 1 {
        lines.push(format!(
            "(stunned ×{}: next {} untap steps skipped)",
            p.stun_counter_count, p.stun_counter_count
        ));
    } else if p.has_stun_counters {
        lines.push(String::from("(stunned: next untap is skipped)"));
    } else if p.wont_untap {
        // A continuous untap lock (Plumes of Peace, Winter Orb, …) — distinct
        // from the one-shot stun case above, which has its own line.
        lines.push(String::from("(locked: won't untap during its next untap step)"));
    }
    // Surface +1/+1 and -1/-1 counter highlights — the most common
    // counter shapes carry a P/T delta that's often more important than
    // the printed body. Push (modern_decks batch 174): added the
    // has_plus_one_counters / has_minus_one_counters helpers on
    // PermanentView so the client doesn't have to scan the `counters`
    // vec; surface them here.
    // Read the explicit counts off the `counters` vec so the badge can
    // show the actual P/T swing (e.g. "(boosted: +3/+3 from 3 +1/+1
    // counters)") — far more useful than a bare boolean when an enrage
    // creature or a Quandrix Fractal has stacked several counters. Falls
    // back to the legacy boolean badge if the explicit vec is empty but
    // the helper flag is set (older server projection / snapshot).
    let plus_n = p
        .counters
        .iter()
        .find_map(|(k, n)| matches!(k, CounterType::PlusOnePlusOne).then_some(*n))
        .unwrap_or(0);
    let minus_n = p
        .counters
        .iter()
        .find_map(|(k, n)| matches!(k, CounterType::MinusOneMinusOne).then_some(*n))
        .unwrap_or(0);
    if plus_n > 0 {
        lines.push(format!(
            "(boosted: +{plus_n}/+{plus_n} from {plus_n} +1/+1 counter{})",
            if plus_n == 1 { "" } else { "s" }
        ));
    } else if p.has_plus_one_counters {
        lines.push(String::from("(boosted: +1/+1 counters)"));
    }
    if minus_n > 0 {
        lines.push(format!(
            "(weakened: -{minus_n}/-{minus_n} from {minus_n} -1/-1 counter{})",
            if minus_n == 1 { "" } else { "s" }
        ));
    } else if p.has_minus_one_counters {
        lines.push(String::from("(weakened: -1/-1 counters)"));
    }
    // Surface CR 122.1b keyword counters — one line per active counter type
    // (flying, first strike, deathtouch, menace, trample, …). Uses the shared
    // `keyword_label` (with its cost/argument formatting) rather than the raw
    // debug form, so parameterized keywords read cleanly.
    for (kw, n) in &p.keyword_counters {
        let label = keyword_label(kw);
        if *n == 1 {
            lines.push(format!("({label} counter)"));
        } else {
            lines.push(format!("({n} {label} counters)"));
        }
    }

    // CR 702.183 — Impending countdown: not a creature until the last time
    // counter comes off (one per controller's end step).
    if let Some(n) = p.impending_counters
        && n > 0
    {
        lines.push(format!(
            "(impending: becomes a creature in {n} end step{})",
            if n == 1 { "" } else { "s" }
        ));
    }

    if p.tapped {
        lines.push(String::from("(tapped)"));
    }

    // Combat status: surface "(attacking)" / "(blocking attacker N)"
    // so the player can tell at a glance which creatures are committed
    // to combat. Push (claude/modern_decks batch 202).
    if p.attacking {
        lines.push(String::from("(attacking)"));
    }
    if !p.blocking_attackers.is_empty() {
        let ids: Vec<String> =
            p.blocking_attackers.iter().map(|a| format!("#{}", a.0)).collect();
        lines.push(format!("(blocking {})", ids.join(", ")));
    }

    // Designation badges surfaced over the wire (CR 701.60 Suspect,
    // CR 701.15 Goad, CR 701.31 Monstrosity). Each is a sticky game-state
    // flag the player needs at a glance — a suspected creature has menace
    // and can't block, a goaded creature must attack a player other than
    // the goader if able, and a monstrous creature has already paid its
    // one-shot monstrosity cost. Push (claude/modern_decks).
    if p.suspected {
        lines.push(String::from("(suspected — has menace, can't block)"));
    }
    if p.goaded {
        lines.push(String::from("(goaded — must attack, and not the goader if able)"));
    }
    if p.monstrous {
        lines.push(String::from("(monstrous)"));
    }
    // CR 702.158b Sector designation — which sector a space sculptor assigned
    // this creature to; block restrictions and sector-wide effects key on it.
    if let Some(sector) = &p.sector {
        lines.push(format!("({sector} sector)"));
    }
    // CR 702.93 Renown — its renown trigger has already fired (it became
    // renowned), so it won't add more counters on later combat hits.
    if p.renowned {
        lines.push(String::from("(renowned)"));
    }
    // CR 701.35 Detain — can't attack/block and its abilities can't be
    // activated until the detaining player's next turn.
    if p.detained {
        lines.push(String::from("(detained — can't attack/block/activate)"));
    }
    // Entrancing Lyre — locked from untapping while the source stays tapped.
    if p.untap_locked {
        lines.push(String::from("(won't untap)"));
    }

    // CR 714 — Saga chapter progress. The current chapter is the Lore counter
    // count; `saga_final_chapter` is the highest chapter number (sacrificed
    // after it resolves).
    if let Some(final_ch) = p.saga_final_chapter {
        let lore = p
            .counters
            .iter()
            .find(|(k, _)| *k == CounterType::Lore)
            .map(|(_, n)| *n)
            .unwrap_or(0);
        lines.push(format!("(saga — chapter {lore} / {final_ch})"));
    }

    // Marked damage: every creature with non-zero damage is one toughness-
    // threshold away from death. Surface "marked: N damage" plus a
    // (lethal? Y/N) shorthand so the player sees at a glance how close
    // the creature is to dying. Push (claude/modern_decks batch 162) —
    // covers CR 121-style damage tracking. Hidden when no damage marked
    // (the common case for fresh permanents).
    if p.damage > 0 && p.card_types.contains(&CardType::Creature) {
        if p.damage as i32 >= p.toughness {
            lines.push(format!("(marked: {} damage — LETHAL)", p.damage));
        } else {
            // Surface the survival margin so the player sees how much
            // more damage kills the creature — pairs with the combat
            // status lines for at-a-glance combat math.
            let to_live = p.toughness - p.damage as i32;
            lines.push(format!(
                "(marked: {} damage; {} more lethal)",
                p.damage, to_live
            ));
        }
    }

    // Summoning sickness: creatures that entered this turn can't attack
    // or use {T} activated abilities (per CR 302.1). Show this in the
    // tooltip so players don't accidentally tap a fresh creature
    // expecting an attack.
    if p.summoning_sick && p.card_types.contains(&CardType::Creature) {
        lines.push(String::from("(summoning sick)"));
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}


/// Printed deck-construction restriction for a companion (CR 702.139c),
/// for the card-info panel. Mirrors `format::companion_restriction_met`.
pub(crate) fn companion_restriction_text(rule: &crabomination::card::CompanionRule) -> &'static str {
    use crabomination::card::CompanionRule as C;
    match rule {
        C::PermanentsManaValueAtMost(_) => "Each permanent card in your starting deck has a low enough mana value.",
        C::NonlandManaValueAtLeast(_) => "Your starting deck contains only high-mana-value cards and lands.",
        C::NonlandEvenManaValue => "Your starting deck contains only cards with even mana values and lands.",
        C::NonlandOddManaValue => "Your starting deck contains only cards with odd mana values and lands.",
        C::NoDuplicateManaSymbols => "No card in your starting deck has more than one of the same mana symbol in its cost.",
        C::Singleton => "Each nonland card in your starting deck has a different name.",
        C::CreatureTypesAmong(_) => "Each creature card in your starting deck is one of the named types.",
        C::NonlandShareACardType => "Each nonland card in your starting deck shares a card type.",
        C::DeckSizeAtLeastOverMinimum(_) => "Your starting deck is well above the minimum size.",
        C::PermanentsHaveActivatedAbility => "Each permanent card in your starting deck has an activated ability.",
    }
}

/// Render a CR 615 shield's remaining points and source restriction as the
/// tooltip's parenthetical: "all damage prevented this turn", "prevents the
/// next 3 damage", plus " from red sources" when the shield is color-scoped.
pub(crate) fn prevention_summary(
    remaining: Option<u32>,
    colors: &[crabomination::mana::Color],
) -> String {
    let mut out = match remaining {
        None => "all damage prevented this turn".to_string(),
        Some(0) => "damage prevented this turn".to_string(),
        Some(1) => "prevents the next 1 damage".to_string(),
        Some(n) => format!("prevents the next {n} damage"),
    };
    if !colors.is_empty() {
        let names: Vec<String> =
            colors.iter().map(|c| format!("{c:?}").to_lowercase()).collect();
        out.push_str(&format!(" from {} sources", names.join("/")));
    }
    out
}

/// Short reminder text for the evergreen keywords. `None` for keywords
/// whose name is self-explanatory or that carry their own cost label
/// (Ward, Cycling, Flashback, …), so the reminder block stays compact.
pub(crate) fn keyword_reminder(kw: &crabomination::card::Keyword) -> Option<&'static str> {
    use crabomination::card::Keyword as K;
    Some(match kw {
        K::Flying => "Can only be blocked by creatures with flying or reach.",
        K::Reach => "Can block creatures with flying.",
        K::FirstStrike => "Deals combat damage before creatures without first strike.",
        K::DoubleStrike => "Deals both first-strike and regular combat damage.",
        K::Deathtouch => "Any amount of damage it deals to a creature is lethal.",
        K::Trample => "Excess combat damage is dealt to the player or planeswalker.",
        K::Lifelink => "Damage it deals also gains you that much life.",
        K::Vigilance => "Doesn't tap when attacking.",
        K::Menace => "Can only be blocked by two or more creatures.",
        K::Defender => "Can't attack.",
        K::Decayed => "Can't block; when it attacks, sacrifice it at end of combat.",
        K::Haste => "Can attack and tap the turn it comes under your control.",
        K::Indestructible => "Can't be destroyed by damage or \"destroy\" effects.",
        K::Hexproof => "Can't be the target of spells or abilities opponents control.",
        K::HexproofFromColor(_) => "Can't be targeted by that color's spells or abilities opponents control.",
        K::HexproofExceptColors(_) => "Can't be targeted by opponents' spells or abilities unless they include one of the listed colors.",
        K::HexproofFromAbilities => "Can't be the target of activated or triggered abilities opponents control.",
        K::Shroud => "Can't be the target of any spells or abilities.",
        K::Infect => "Damages creatures with -1/-1 counters and players with poison.",
        K::Wither => "Damages creatures as -1/-1 counters instead.",
        K::Persist => "Returns with a -1/-1 counter when it dies (if it had none).",
        K::Undying => "Returns with a +1/+1 counter when it dies (if it had none).",
        K::Prowess => "Gets +1/+1 until end of turn when you cast a noncreature spell.",
        K::Fear => "Can only be blocked by artifact and/or black creatures.",
        K::Intimidate => "Can only be blocked by artifacts and creatures sharing a color.",
        K::Skulk => "Can't be blocked by creatures with greater power.",
        K::Shadow => "Can only block or be blocked by creatures with shadow.",
        K::Horsemanship => "Can only be blocked by creatures with horsemanship.",
        K::Landwalk(_) | K::LandwalkFiltered(_) => "Can't be blocked if the defending player controls a land of the named type.",
        K::Unblockable => "Can't be blocked.",
        K::CantBeBlockedByMoreThanOne => "Can't be blocked by more than one creature.",
        K::CantBeBlockedExceptByN(_) => "Can't be blocked except by that many or more creatures.",
        K::CantBeBlockedExceptBy(_) => "Can only be blocked by creatures matching the named quality.",
        K::CantBeBlockedBy(_) => "Can't be blocked by creatures matching the named quality.",
        K::CantBeBlockedByPowerLess => "Can't be blocked by creatures with less power than it.",
        K::CantBeBlockedByPowerAtMost(_) => "Can't be blocked by creatures with that much power or less.",
        K::CantBeBlockedByPowerAtLeast(_) => "Can't be blocked by creatures with that much power or greater.",
        K::Changeling => "Is every creature type.",
        K::Flash => "You may cast it any time you could cast an instant.",
        K::Flanking => "Creatures without flanking blocking it get -1/-1 until end of turn.",
        K::Banding => "Helps creatures attack and block as a group; its controller assigns the combat damage of creatures it bands with.",
        K::Exert => "You may exert it as it attacks — an exerted creature won't untap during your next untap step.",
        K::Phasing => "Phases out (and back in) during its controller's untap step; while phased out it's treated as though it doesn't exist.",
        K::Toxic(_) => "Players it deals combat damage to also get that many poison counters.",
        K::Modular(_) => "Enters with that many +1/+1 counters; when it dies, you may move them to target artifact creature.",
        K::Sunburst => "Enters with a counter for each color of mana spent to cast it.",
        K::Annihilator(_) => "Whenever it attacks, the defending player sacrifices that many permanents.",
        K::Firebending(_) | K::FirebendingPower | K::FirebendingCreaturesYouControl => "Whenever it attacks, add that much {R}; the mana lasts until end of combat.",
        K::Convoke => "You may tap untapped creatures to help pay this spell's cost.",
        K::Delve => "You may exile cards from your graveyard, each paying for {1} of this spell's cost.",
        K::Cascade => "When you cast it, exile cards from the top of your library until you hit a cheaper nonland card; you may cast that card for free.",
        K::Storm => "When you cast it, copy it for each spell cast before it this turn.",
        K::SplitSecond => {
            "While this spell is on the stack, players can't cast spells or activate non-mana abilities."
        }
        K::Dredge(_) => "Instead of drawing, you may mill that many cards to return this from your graveyard to your hand.",
        K::Rebound => "If cast from your hand, it's exiled instead of going to the graveyard, and you may cast it for free next turn.",
        K::CantBeCountered => "Can't be countered.",
        K::CanAttackOnlyIfDefenderControls(_) => {
            "Can attack only if the defending player controls a matching permanent."
        }
        K::Ninjutsu(_) => {
            "Return an unblocked attacker you control to hand to put this onto the battlefield tapped and attacking."
        }
        K::Fading(_) => "Enters with that many fade counters; remove one each upkeep, and sacrifice it when you can't.",
        K::Vanishing(_) => "Enters with that many time counters; remove one each upkeep, and sacrifice it when the last is gone.",
        K::Protection(_) => "Can't be blocked, targeted, dealt damage, enchanted, or equipped by anything of that color.",
        K::Bushido(_) => "Whenever it blocks or becomes blocked, it gets +N/+N until end of turn.",
        K::Melee => "When it attacks, it gets +1/+1 until end of turn for each opponent you attacked.",
        K::Absorb(_) => "If a source would deal damage to this creature, prevent N of that damage.",
        K::Rampage(_) => "Whenever it becomes blocked, it gets +N/+N for each blocker beyond the first.",
        K::Crew(_) => "Tap any number of creatures with total power N or greater to turn this Vehicle into an artifact creature.",
        K::Madness(_) => "If you discard it, you may cast it for its madness cost instead of putting it in your graveyard.",
        K::Ward(_) => "Whenever it becomes the target of a spell or ability an opponent controls, counter it unless they pay the ward cost.",
        K::Gravestorm => "When you cast it, copy it for each permanent put into a graveyard this turn.",
        K::Unleash => "You may have it enter with a +1/+1 counter; if it has one, it can't block.",
        K::Bargain => "You may sacrifice an artifact, enchantment, or token as you cast this spell.",
        K::AttacksAlone => "Can only attack alone.",
        K::CantAttackAlone => "Can't attack alone (another creature must also attack).",
        K::CantAttackOrBlockAlone => {
            "Can't attack or block alone (another creature must also attack/block)."
        }
        K::CantBlock => "Can't block.",
        K::CantAttack => "Can't attack.",
        K::MustAttack => "Attacks each combat if able.",
        // Three distinct block requirements (CR 509.1c) — split so the tooltip
        // says which one, instead of one vague "forced into combat" line.
        K::MustBlock => "Blocks each combat if able.",
        K::AllMustBlock => "All creatures able to block this creature do so (Lure).",
        K::MustBeBlocked => "Must be blocked if able.",
        K::CantBeCopied => "Can't be copied.",
        K::DealsNoCombatDamage => "Assigns no combat damage.",
        K::AssignsCombatDamageByToughness => {
            "Assigns combat damage equal to its toughness rather than its power."
        }
        K::Offspring(_) => {
            "You may pay an additional cost as you cast this; if you do, it enters making a 1/1 token copy of itself."
        }
        K::Daybound => "If it's neither day nor night, it becomes day; transforms when it becomes night.",
        K::Nightbound => "Transforms back when it becomes day.",
        K::Conspire => "As you cast it, you may tap two untapped creatures that share a color with it to copy it.",
        K::Disturb(_) => "You may cast it from your graveyard transformed for its disturb cost.",
        K::Entwine(_) => "Choose both modes if you pay the entwine cost.",
        K::Epic => "Copy this spell at the start of each of your upkeeps; you can't cast other spells.",
        K::Improvise => "You may tap untapped artifacts to help pay this spell's cost.",
        K::JumpStart => "You may cast it from your graveyard by also discarding a card, then exile it.",
        K::Replicate(_) => "As you cast it, pay its replicate cost any number of times to copy it that many times.",
        K::ReplicateEnergy(_) => "As you cast it, pay its energy replicate cost any number of times to copy it that many times.",
        K::Splice(_, _) => "As you cast an Arcane spell, you may reveal this from hand and pay its splice cost to add its effects.",
        K::Squad(_) => "As you cast it, pay its squad cost any number of times to make that many extra token copies.",
        K::UmbraArmor => "If enchanted creature would be destroyed, instead remove all damage and destroy this Aura.",
        K::Companion => "If your deck meets its condition, you may play it from outside the game once per game.",
        K::ProtectionFromColoredSpells => "Can't be targeted, blocked, or damaged by colored spells.",
        K::ProtectionFromSpells => "Can't be targeted or damaged by spells.",
        K::ProtectionFromCreatures => "Can't be blocked, targeted, or damaged by creatures.",
        K::ProtectionFromMulticolored => "Can't be blocked, targeted, or damaged by multicolored sources.",
        K::ProtectionFromMonocolored => "Can't be blocked, targeted, or damaged by monocolored (exactly one color) sources.",
        K::ProtectionFromInstants => "Can't be targeted or damaged by instant spells.",
        K::ProtectionFromEverything => "Can't be blocked, targeted, enchanted, equipped, or damaged by anything.",
        K::ProtectionFromManaValueExcept(_) => "Has protection from each mana value other than the named one.",
        K::ProtectionFromManaValueParity { odd } => if *odd {
            "Has protection from each odd mana value."
        } else {
            "Has protection from each even mana value."
        },
        K::ProtectionFromCreatureType(_) => "Can't be blocked, targeted, or damaged by sources of the named creature type.",
        K::ProtectionFromMatching(_) => "Can't be blocked, targeted, or damaged by any source matching the named filter.",
        K::ProtectionFromSpellSubtype(_) => "Can't be targeted or damaged by spells of the named subtype.",
        K::Cycling(_) => "Pay its cycling cost and discard it to draw a card, any time you could cast an instant.",
        K::CyclingLife(_) => "Pay that much life and discard it to draw a card.",
        K::Kicker(_) | K::Multikicker(_) => "You may pay an additional kicker cost as you cast it for a bonus effect.",
        K::Flashback(_) | K::FlashbackTap(_) => "You may cast it from your graveyard for its flashback cost, then exile it.",
        K::Suspend(_, _) => "You may exile it with that many time counters and pay its suspend cost; remove one each upkeep and cast it free when the last is gone.",
        K::SuspendAccelerant => "While this is suspended, an opponent's action removes time counters from it.",
        K::Echo(_) => "Pay its echo cost at the beginning of your next upkeep after it enters, or sacrifice it.",
        K::Impending(_) => "Cast for its impending cost to enter with that many time counters; it isn't a creature until the last is removed.",
        K::Casualty(_) => "As you cast it, you may sacrifice a creature with that much power to copy it.",
        K::Saddle(_) => "Tap other creatures with total power N to saddle it; saddled abilities work when it attacks.",
        K::Buyback(_) => "You may pay an additional buyback cost; if you do, it returns to your hand instead of the graveyard.",
        K::CumulativeUpkeep(_) => "At your upkeep, put an age counter on it, then pay its cumulative upkeep cost for each age counter or sacrifice it.",
        K::Devoid => "It has no color.",
        K::Morph(_) => "You may cast it face down as a 2/2; turn it face up any time for its morph cost.",
        K::Megamorph(_) => "You may cast it face down as a 2/2; turn it face up for its megamorph cost, entering with a +1/+1 counter.",
        K::Disguise(_) => "You may cast it face down as a 2/2 with ward {2}; turn it face up any time for its disguise cost.",
        K::Equip(_) => "Pay its equip cost to attach it to a creature you control, any time you could cast a sorcery.",
        K::Fortify(_) => "Pay its fortify cost to attach it to a land you control.",
        K::Reconfigure(_) => "Pay its reconfigure cost to attach it to a creature, or to unattach it; it's a creature while unattached.",
        K::Escape(_, _) => "You may cast it from your graveyard for its escape cost, also exiling other cards from your graveyard.",
        K::Retrace => "You may cast it from your graveyard by also discarding a land card.",
        K::Regenerate(_) => "The next time it would be destroyed this turn, instead tap it, remove it from combat, and heal its damage.",
        K::Reinforce(_, _) => "Pay its reinforce cost and discard it to put that many +1/+1 counters on a creature.",
        K::Soulbond => "You may pair it with another unpaired creature when either enters; the pair shares a bonus.",
        K::Inspired => "Whenever it becomes untapped, its inspired ability triggers.",
        K::Landcycling(_, _) => "Pay its landcycling cost and discard it to search your library for a matching land.",
        K::Typecycling(_) => "Pay its typecycling cost and discard it to search your library for a matching card.",
        K::Mayhem(_) => "You may cast it from your graveyard for its mayhem cost if you discarded a card this turn; then exile it.",
        K::Harmonize(_) => "You may cast it from your graveyard for its harmonize cost; you may tap an untapped creature to pay {1} of that cost. Then exile it.",
        K::CantActivateAbilities => "Its activated abilities can't be activated.",
        K::CantAttackUnlessCastCreatureThisTurn => "Can't attack unless you cast a creature spell this turn.",
        K::CanAttackOnlyIfYouControl(_) => "Can attack only if you control a matching permanent.",
        K::CantAttackOrBlockUnlessEvenCounters => "Can't attack or block unless it has an even number of counters on it.",
        K::CantAttackOrBlockUnlessYouControlCount { attack_only: true, .. } => "Can't attack (but may still block) unless you control enough matching permanents.",
        K::CantAttackOrBlockUnlessYouControlCount { block_only: true, .. } => "Can't block (but may still attack) unless you control enough matching permanents.",
        K::CantAttackOrBlockUnlessYouControlCount { .. } => "Can't attack or block unless you control enough matching permanents.",
        K::CantBeCounteredIfXAtLeast(_) => "Can't be countered if X was paid at or above the named amount.",
        K::StartYourEngines => "When it enters, if you have no speed, your speed becomes 1. Your speed then increases by 1 the first time an opponent loses life on each of your turns (max 4).",
        K::Poisonous(_) => "Whenever it deals combat damage to a player, that player gets that many poison counters.",
        K::CanBlockOnlyFlying => "Can block only creatures with flying.",
        K::CantAttackOrBlockUnlessHandSizeAtMost(_) => "Can't attack or block unless you have that many or fewer cards in hand.",
        K::CantAttackOrBlockUnlessDelirium => "Can't attack or block unless you have delirium (four or more card types among cards in your graveyard).",
        K::CantAttackUnlessLandCount(_, _) => "Can't attack unless that many lands of the named type are on the battlefield (anyone's count).",
        K::CantAttackUnlessOpponentDamaged => "Can't attack unless an opponent has been dealt damage this turn.",
        K::CantAttackOrBlockUnlessPay(_) => "Can't attack or block unless its controller pays the listed mana. The cost is charged as attackers or blockers are declared.",
        K::CantAttackOrBlockUnlessDescend(_) => "Descend — can't attack or block unless there are that many or more permanent cards in your graveyard.",
        K::CantAttackOrBlockUnlessCityBlessing => "Can't attack or block unless you have the city's blessing.",
        K::Bloodthirst(_) => "If an opponent was dealt damage this turn, it enters with that many +1/+1 counters.",
        K::CantBeBlockedIfControllerCastSpells(_) => "Can't be blocked if you've cast that many or more spells this turn.",
        K::Sneak(_) => "You may cast it for its sneak cost by returning an unblocked attacker you control to its owner's hand.",
        K::Frenzy(_) => "Whenever this attacks and isn't blocked, it gets +N/+0 until end of turn.",
        K::CantBlockPowerAtLeast(_) => "Can't block creatures with power that high or greater.",
        K::CantAttackOrBlockUnlessCreatureDiedThisTurn => "Can't attack or block unless a creature died this turn.",
        K::TrampleOverPlaneswalkers => "Excess combat damage dealt to a planeswalker it's attacking tramples over to that planeswalker's controller.",
        K::Compleated => "If a Phyrexian pip was paid with life as it was cast, it enters with that many fewer loyalty counters.",
        K::ProtectionFromCardType(_) => "Can't be blocked, targeted, dealt damage, enchanted, or equipped by sources of the named card type.",
        K::HexproofFromMonocolored => "Can't be the target of monocolored spells or abilities opponents control.",
        K::EchoDiscard => "At your next upkeep after it enters, discard a card or sacrifice it.",
        K::DoesntUntapWhileCounter(_) => "Doesn't untap during your untap step while it has a counter of the named kind.",
        _ => return None,
    })
}

/// Plain-language label for the count-gate filter on
/// `CantAttackOrBlockUnlessYouControlCount` (Topiary Stomper, Tiger-Dillo,
/// Lambholt Pacifist). Covers the printed shapes; falls back to a generic
/// phrase for filters the gate never actually uses.
fn describe_count_filter(req: &crabomination::card::SelectionRequirement) -> String {
    use crabomination::card::SelectionRequirement as R;
    match req {
        R::Land => "lands".into(),
        R::Creature => "creatures".into(),
        R::Artifact => "artifacts".into(),
        R::Enchantment => "enchantments".into(),
        R::Planeswalker => "planeswalkers".into(),
        R::Nonland => "nonland permanents".into(),
        R::Noncreature => "noncreature permanents".into(),
        R::IsBasicLand => "basic lands".into(),
        R::IsNonbasicLand => "nonbasic lands".into(),
        R::HasCreatureType(t) => format!("{t:?}s"),
        R::HasArtifactSubtype(a) => format!("{a:?}s"),
        R::HasLandType(l) => format!("{l:?}s"),
        R::PowerAtLeast(n) => format!("creatures with power {n} or greater"),
        R::And(a, b) => match (&**a, &**b) {
            // The common "creature with power N or greater" pairing.
            (R::Creature, R::PowerAtLeast(n)) | (R::PowerAtLeast(n), R::Creature) => {
                format!("creatures with power {n} or greater")
            }
            _ => format!("{} that are {}", describe_count_filter(a), describe_count_filter(b)),
        },
        R::Or(a, b) => format!("{} or {}", describe_count_filter(a), describe_count_filter(b)),
        _ => "matching permanents".into(),
    }
}

/// Render a `Keyword` as a short human string for the tooltip. Keeps the
/// labels short ("Lifelink", "First Strike") so a card with several granted
/// keywords doesn't blow out the tooltip line.
pub(crate) fn keyword_label(kw: &crabomination::card::Keyword) -> String {
    use crabomination::card::Keyword as K;
    use crabomination::mana::Color;
    let color_word = |c: &Color| -> &'static str {
        match c {
            Color::White => "white",
            Color::Blue => "blue",
            Color::Black => "black",
            Color::Red => "red",
            Color::Green => "green",
        }
    };
    match kw {
        K::Flying => "Flying".into(),
        K::FirstStrike => "First Strike".into(),
        K::DoubleStrike => "Double Strike".into(),
        K::Lifelink => "Lifelink".into(),
        K::Vigilance => "Vigilance".into(),
        K::Trample => "Trample".into(),
        K::Deathtouch => "Deathtouch".into(),
        K::Haste => "Haste".into(),
        K::Menace => "Menace".into(),
        K::Reach => "Reach".into(),
        K::Defender => "Defender".into(),
        K::Decayed => "Decayed".into(),
        K::Indestructible => "Indestructible".into(),
        K::Hexproof => "Hexproof".into(),
        K::HexproofFromColor(c) => format!("Hexproof from {c:?}"),
        K::HexproofExceptColors(colors) => format!("Hexproof except from {colors:?}"),
        K::Flash => "Flash".into(),
        K::Shroud => "Shroud".into(),
        // Surface Ward's cost as "Ward {2}" or "Ward—pay 2 life"
        // instead of the prior `{:?}` shape that printed the raw
        // enum variant text.
        K::Ward(wc) => match wc {
            crabomination::card::WardCost::Mana(c) => format!("Ward {}", c.summary()),
            crabomination::card::WardCost::Life(n) => format!("Ward—Pay {n} life"),
            crabomination::card::WardCost::ManaAndLife(c, n) => {
                format!("Ward—{}, Pay {n} life", c.summary())
            }
            crabomination::card::WardCost::Discard(n) => format!("Ward—Discard {n}"),
            crabomination::card::WardCost::DiscardMatching(_, n) => {
                format!("Ward—Discard {n} matching")
            }
            crabomination::card::WardCost::SacrificeMatching(_) => {
                "Ward—Sacrifice a matching permanent".into()
            }
            crabomination::card::WardCost::DiscardHand => "Ward—Discard your hand".into(),
            crabomination::card::WardCost::Blight(n) => format!("Ward—Blight {n}"),
            crabomination::card::WardCost::RemoveCounterFromPermanent => {
                "Ward—Remove a counter from a permanent you control".into()
            }
            crabomination::card::WardCost::CollectEvidence(n) => {
                format!("Ward—Collect evidence {n}")
            }
            crabomination::card::WardCost::SacrificeCreature => "Ward—Sacrifice a creature".into(),
            crabomination::card::WardCost::SacrificePermanents(n) => {
                format!("Ward—Sacrifice {n} permanents")
            }
            crabomination::card::WardCost::GenericSourcePower => {
                "Pay {X}, X = its power".into()
            }
            crabomination::card::WardCost::LifeSourcePower => {
                "Ward—Pay life equal to its power".into()
            }
        },
        // Protection rolls up the color name in lowercase to match
        // printed Oracle ("protection from white", not "from White").
        K::Protection(c) => format!("Protection from {}", color_word(c)),
        // Cycling / Flashback should expose their cost so the activator
        // can see what they'd pay.
        K::Cycling(cost) => format!("Cycling {}", cost.summary()),
        K::Flashback(cost) => format!("Flashback {}", cost.summary()),
        K::Convoke => "Convoke".into(),
        K::Soulbond => "Soulbond".into(),
        K::Persist => "Persist".into(),
        K::Undying => "Undying".into(),
        K::CantBeCountered => "Can't be countered".into(),
        // Combat / evasion riders that previously fell through to the raw
        // `{:?}` debug shape — give them printed-Oracle phrasing.
        K::CantBlock => "Can't block".into(),
        K::CantAttack => "Can't attack".into(),
        K::CantActivateAbilities => "Activated abilities can't be activated".into(),
        K::AttacksAlone => "Attacks only alone".into(),
        K::CantAttackAlone => "Can't attack alone".into(),
        K::CantAttackOrBlockAlone => "Can't attack or block alone".into(),
        K::DealsNoCombatDamage => "Deals no combat damage".into(),
        K::AssignsCombatDamageByToughness => "Assigns combat damage by toughness".into(),
        K::MustBeBlocked => "Must be blocked if able".into(),
        K::AllMustBlock => "All creatures able to block this do so".into(),
        K::Skulk => "Skulk".into(),
        K::Fear => "Fear".into(),
        K::Intimidate => "Intimidate".into(),
        K::Infect => "Infect".into(),
        K::Wither => "Wither".into(),
        K::Toxic(n) => format!("Toxic {n}"),
        K::Prowess => "Prowess".into(),
        K::Shadow => "Shadow".into(),
        // Keyword-action / ability words that previously printed the raw
        // `{:?}` debug shape — give them printed-Oracle phrasing, surfacing
        // their cost/count where one exists.
        K::Storm => "Storm".into(),
        K::Exert => "Exert".into(),
        K::Inspired => "Inspired".into(),
        K::Changeling => "Changeling".into(),
        K::Daybound => "Daybound".into(),
        K::Nightbound => "Nightbound".into(),
        K::Phasing => "Phasing".into(),
        K::Banding => "Banding".into(),
        K::Rebound => "Rebound".into(),
        K::Retrace => "Retrace".into(),
        K::Delve => "Delve".into(),
        K::Cascade => "Cascade".into(),
        K::Annihilator(n) => format!("Annihilator {n}"),
        K::Frenzy(n) => format!("Frenzy {n}"),
        K::Sneak(c) => format!("Sneak {}", c.summary()),
        K::CantBlockPowerAtLeast(n) => format!("Can't block power {n}+"),
        K::Firebending(n) => format!("Firebending {n}"),
        K::FirebendingPower | K::FirebendingCreaturesYouControl => "Firebending X".into(),
        K::Dredge(n) => format!("Dredge {n}"),
        K::Crew(n) => format!("Crew {n}"),
        K::Madness(cost) => format!("Madness {}", cost.summary()),
        K::Kicker(cost) => format!("Kicker {}", cost.summary()),
        K::Multikicker(cost) => format!("Multikicker {}", cost.summary()),
        K::Echo(cost) => format!("Echo {}", cost.summary()),
        K::CumulativeUpkeep(cost) => format!("Cumulative upkeep {}", cost.summary()),
        K::Fading(n) => format!("Fading {n}"),
        K::Vanishing(n) => format!("Vanishing {n}"),
        K::Equip(cost) => format!("Equip {}", cost.summary()),
        K::Reconfigure(cost) => format!("Reconfigure {}", cost.summary()),
        K::Offspring(cost) => format!("Offspring {}", cost.summary()),
        K::CantAttackOrBlockUnlessEvenCounters =>
            "Can't attack or block unless it has an even number of counters".into(),
        K::CantAttackOrBlockUnlessYouControlCount {
            filter, min, attack_only, block_only, exclude_self,
        } => {
            let verb = if *attack_only {
                "attack"
            } else if *block_only {
                "block"
            } else {
                "attack or block"
            };
            let other = if *exclude_self { "other " } else { "" };
            format!(
                "Can't {verb} unless you control {min} or more {other}{}",
                describe_count_filter(filter),
            )
        }
        // Landwalk: "Forestwalk", "Islandwalk", … (the printed Oracle shape).
        K::Landwalk(lt) => format!("{lt:?}walk"),
        K::LandwalkFiltered(f) => {
            format!("{}walk", f.target_noun().unwrap_or_else(|| "land".into()))
        }
        K::CanAttackOnlyIfDefenderControls(_) => "Conditional attacker".into(),
        K::CantBeBlockedExceptBy(_) => "Can't be blocked except by certain creatures".into(),
        K::CantBeBlockedBy(_) => "Can't be blocked by certain creatures".into(),
        K::CantBeBlockedByMoreThanOne => "Can't be blocked by more than one creature".into(),
        K::CantBeBlockedExceptByN(n) => format!("Can't be blocked except by {n} or more creatures"),
        K::CantBeBlockedByPowerLess => "Can't be blocked by creatures with less power".into(),
        K::CantBeBlockedByPowerAtMost(n) => format!("Can't be blocked by creatures with power {n} or less"),
        K::CantBeBlockedByPowerAtLeast(n) => format!("Can't be blocked by creatures with power {n} or greater"),
        K::Ninjutsu(cost) => format!("Ninjutsu {}", cost.summary()),
        K::Suspend(n, cost) => format!("Suspend {n}—{}", cost.summary()),
        // Cost/count-bearing keywords that otherwise fell through to the raw
        // `{:?}` debug shape — give them printed-Oracle phrasing.
        K::Bushido(n) => format!("Bushido {n}"),
        K::Absorb(n) => format!("Absorb {n}"),
        K::Rampage(n) => format!("Rampage {n}"),
        K::Regenerate(_) => "Regeneration".into(),
        K::Buyback(cost) => format!("Buyback {}", cost.summary()),
        K::Morph(cost) => format!("Morph {}", cost.summary()),
        K::Megamorph(cost) => format!("Megamorph {}", cost.summary()),
        K::Disguise(cost) => format!("Disguise {}", cost.summary()),
        K::Reinforce(n, cost) => format!("Reinforce {n}—{}", cost.summary()),
        K::Saddle(n) => format!("Saddle {n}"),
        K::Casualty(n) => format!("Casualty {n}"),
        K::Escape(cost, n) => format!("Escape {}, exile {n}", cost.summary()),
        K::Fortify(cost) => format!("Fortify {}", cost.summary()),
        K::FlashbackTap(n) => format!("Flashback—Tap {n} creatures"),
        K::Unleash => "Unleash".into(),
        K::Bargain => "Bargain".into(),
        K::MustBlock => "Blocks each combat if able".into(),
        K::MustAttack => "Attacks each combat if able".into(),
        K::CantBeCopied => "Can't be copied".into(),
        // Player-facing keywords that previously fell through to the raw
        // `{:?}` debug shape.
        K::Devoid => "Devoid (colorless)".into(),
        K::Landcycling(cost, lt) => format!("{lt:?}cycling {}", cost.summary()),
        K::Typecycling(spec) => format!("Typecycling {}", spec.0.summary()),
        K::CantBeCounteredIfXAtLeast(n) => {
            format!("Can't be countered if X is {n} or more")
        }
        K::StartYourEngines => "Start your engines!".into(),
        // Evasion / combat-restriction keywords that previously fell through to
        // the raw `{:?}` debug shape.
        K::Unblockable => "Can't be blocked".into(),
        K::Horsemanship => "Horsemanship".into(),
        K::Flanking => "Flanking".into(),
        K::Melee => "Melee".into(),
        K::SplitSecond => "Split second".into(),
        K::CanBlockOnlyFlying => "Can block only creatures with flying".into(),
        K::CantBeBlockedIfControllerCastSpells(n) => {
            format!("Can't be blocked if its controller cast {n} or more spells this turn")
        }
        K::CantAttackUnlessCastCreatureThisTurn => {
            "Can't attack unless you cast a creature spell this turn".into()
        }
        K::CantAttackOrBlockUnlessDelirium => "Can't attack or block unless you have delirium".into(),
        K::CantAttackUnlessLandCount(lt, n) => {
            format!("Can't attack unless there are {n} or more {lt:?}s on the battlefield")
        }
        K::CantAttackUnlessOpponentDamaged => {
            "Can't attack unless an opponent has been dealt damage this turn".into()
        }
        K::CantAttackOrBlockUnlessPay(n) => {
            format!("Can't attack or block unless its controller pays {{{n}}}")
        }
        K::CantAttackOrBlockUnlessCreatureDiedThisTurn => {
            "Can't attack or block unless a creature died under your control this turn".into()
        }
        K::CantAttackOrBlockUnlessHandSizeAtMost(n) => {
            format!("Can't attack or block unless you have {n} or fewer cards in hand")
        }
        K::CantAttackOrBlockUnlessDescend(n) => {
            format!("Can't attack or block unless you descended {n}")
        }
        K::CantAttackOrBlockUnlessCityBlessing => {
            "Can't attack or block unless you have the city's blessing".into()
        }
        K::CanAttackOnlyIfYouControl(_) => {
            "Can attack only if you control a matching permanent".into()
        }
        // Protection variants beyond the single-color case.
        K::ProtectionFromEverything => "Protection from everything".into(),
        K::ProtectionFromMulticolored => "Protection from multicolored".into(),
        K::ProtectionFromMonocolored => "Protection from monocolored".into(),
        K::ProtectionFromInstants => "Protection from instants".into(),
        K::ProtectionFromColoredSpells => "Protection from colored spells".into(),
        K::ProtectionFromSpells => "Protection from spells".into(),
        K::ProtectionFromCreatures => "Protection from creatures".into(),
        K::ProtectionFromCreatureType(t) => format!("Protection from {t:?}"),
        K::ProtectionFromMatching(f) => {
            format!("Protection from {}", describe_count_filter(f))
        }
        K::ProtectionFromSpellSubtype(s) => format!("Protection from {s:?} spells"),
        K::ProtectionFromManaValueExcept(n) => {
            format!("Protection from each mana value except {n}")
        }
        K::ProtectionFromManaValueParity { odd } => {
            format!("Protection from {} mana value", if *odd { "odd" } else { "even" })
        }
        K::UmbraArmor => "Totem armor".into(),
        // Counter / cost keywords that previously printed the raw `{:?}` shape.
        K::Poisonous(n) => format!("Poisonous {n}"),
        K::Bloodthirst(n) => format!("Bloodthirst {n}"),
        K::CyclingLife(n) => format!("Cycling—Pay {n} life"),
        K::Impending(n) => format!("Impending {n}"),
        K::Entwine(c) => format!("Entwine {}", c.summary()),
        K::Squad(c) => format!("Squad {}", c.summary()),
        K::Replicate(c) => format!("Replicate {}", c.summary()),
        K::ReplicateEnergy(n) => format!("Replicate—Pay {}", "{E}".repeat(*n as usize)),
        K::HexproofFromAbilities => "Hexproof from abilities".into(),
        K::Mayhem(c) => format!("Mayhem {}", c.summary()),
        K::Harmonize(c) => format!("Harmonize {}", c.summary()),
        K::Disturb(c) => format!("Disturb {}", c.summary()),
        K::Splice(c, st) => format!("Splice onto {st:?} {}", c.summary()),
        // Ability words / static keywords with no payload.
        K::Conspire => "Conspire".into(),
        K::Improvise => "Improvise".into(),
        K::Gravestorm => "Gravestorm".into(),
        K::Epic => "Epic".into(),
        K::JumpStart => "Jump-start".into(),
        K::Companion => "Companion".into(),
        K::SuspendAccelerant => "Suspend".into(),
        // Payload/static keywords that used to render their raw `{:?}` debug shape.
        K::TrampleOverPlaneswalkers => "Trample over planeswalkers".into(),
        K::Compleated => "Compleated".into(),
        K::ProtectionFromCardType(t) => format!("Protection from {t:?}s"),
        K::HexproofFromMonocolored => "Hexproof from monocolored".into(),
        K::EchoDiscard => "Echo—Discard".into(),
        K::DoesntUntapWhileCounter(k) => format!("Doesn't untap with a {k:?} counter"),
        // Any keyword without a hand-written label reads as a humanized variant
        // name ("Split second") rather than a raw Rust debug shape
        // ("SplitSecond(2)") — a readable floor for future keywords.
        other => humanize_keyword_debug(other),
    }
}

/// Turn a keyword's `{:?}` debug shape into a readable label: keep the variant
/// name (up to its first payload delimiter), split CamelCase into words, and
/// lowercase all but the first ("CantBeBlocked(...)" → "Cant be blocked").
fn humanize_keyword_debug(kw: &crabomination::card::Keyword) -> String {
    let dbg = format!("{kw:?}");
    let name = dbg
        .split(['(', '{', ' '])
        .next()
        .unwrap_or(&dbg);
    let mut out = String::new();
    for (i, ch) in name.char_indices() {
        if ch.is_uppercase() && i != 0 {
            out.push(' ');
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn sort_key(kind: CounterType) -> u8 {
    match kind {
        CounterType::PlusOnePlusOne => 0,
        CounterType::MinusOneMinusOne => 1,
        CounterType::Charge => 2,
        CounterType::Stun => 3,
        CounterType::Time => 4,
        CounterType::Poison => 5,
        CounterType::Energy => 6,
        _ => 7,
    }
}

fn counter_label(kind: CounterType) -> &'static str {
    match kind {
        CounterType::PlusOnePlusOne => "+1/+1",
        CounterType::MinusOneMinusOne => "-1/-1",
        CounterType::MinusZeroMinusOne => "-0/-1",
        CounterType::MinusOneMinusZero => "-1/-0",
        CounterType::Loyalty => "Loyalty",
        CounterType::Charge => "Charge",
        CounterType::Manifestation => "Manifestation",
        CounterType::Stun => "Stun",
        CounterType::Time => "Time",
        CounterType::Poison => "Poison",
        CounterType::Lore => "Lore",
        CounterType::Fade => "Fade",
        CounterType::Blood => "Blood",
        CounterType::Plague => "Plague",
        CounterType::Fuse => "Fuse",
        CounterType::Age => "Age",
        CounterType::Level => "Level",
        CounterType::Energy => "Energy",
        CounterType::Experience => "Experience",
        CounterType::Verse => "Verse",
        CounterType::Shield => "Shield",
        CounterType::Wish => "Wish",
        CounterType::Invitation => "Invitation",
        CounterType::Page => "Page",
        CounterType::Growth => "Growth",
        CounterType::Prepared => "Prepared",
        CounterType::Finality => "Finality",
        CounterType::Indestructible => "Indestructible",
        CounterType::Silver => "Silver",
        CounterType::Luck => "Luck",
        CounterType::Quest => "Quest",
        CounterType::Study => "Study",
        CounterType::Book => "Book",
        CounterType::Point => "Point",
        CounterType::Hone => "Hone",
        CounterType::Burden => "Burden",
        CounterType::Fate => "Fate",
        CounterType::Ice => "Ice",
        CounterType::Soot => "Soot",
        CounterType::Void => "Void",
        CounterType::Ki => "Ki",
        CounterType::Coin => "Coin",
        CounterType::Tide => "Tide",
        CounterType::Flood => "Flood",
        CounterType::Bounty => "Bounty",
        CounterType::Oil => "Oil",
        CounterType::Blight => "Blight",
        CounterType::Valor => "Valor",
        CounterType::Defense => "Defense",
        CounterType::Possession => "Possession",
        CounterType::Nest => "Nest",
        CounterType::Fire => "Fire",
        CounterType::Conqueror => "Conqueror",
        CounterType::Muster => "Muster",
        CounterType::Acorn => "Acorn",
        CounterType::Incubation => "Incubation",
        CounterType::Revival => "Revival",
        CounterType::Stash => "Stash",
        CounterType::Divinity => "Divinity",
        CounterType::Devotion => "Devotion",
        CounterType::Aim => "Aim",
        CounterType::Theft => "Theft",
        CounterType::Training => "Training",
        CounterType::Fellowship => "Fellowship",
        CounterType::Bait => "Bait",
        CounterType::Supply => "Supply",
        CounterType::Unlock => "Unlock",
        CounterType::Palliation => "Palliation",
        CounterType::Eon => "Eon",
        CounterType::Blaze => "Blaze",
        CounterType::Phylactery => "Phylactery",
        CounterType::Filibuster => "Filibuster",
        CounterType::Petal => "Petal",
        CounterType::Arrow => "Arrow",
        CounterType::Infection => "Infection",
        CounterType::Fungus => "Fungus",
        CounterType::Storage => "Storage",
        CounterType::Depletion => "Depletion",
    }
}

/// Short reminder text for counters whose effect isn't obvious from the
/// label. `None` for self-explanatory counters (+1/+1, Loyalty, …) so the
/// tooltip stays compact.
fn counter_reminder(kind: CounterType) -> Option<&'static str> {
    Some(match kind {
        CounterType::Stun => "Skips its next untap (remove instead of untapping).",
        CounterType::Finality => "If it would die, exile it instead.",
        CounterType::Shield => "Remove to prevent the next damage/destruction.",
        CounterType::Indestructible => "Can't be destroyed.",
        CounterType::Poison => "Ten poison counters and that player loses.",
        CounterType::Luck => "Chance Encounter wins the game at ten.",
        CounterType::Hone => "Ticks down each of your upkeeps; cast from exile for {4} less when the last is removed.",
        CounterType::Burden => "The One Ring's tally: draw one per burden counter; lose that much life each upkeep.",
        CounterType::Ice => "Removed by triggered effects; the permanent transforms when the last one is gone.",
        CounterType::Soot => "Each player sacrifices a permanent per soot counter at their upkeep.",
        CounterType::Tide => "Ominous Seas: at four or more, remove them to make an 8/8 Kraken.",
        CounterType::Flood => "Quicksilver Fountain: a land with a flood counter on it is an Island.",
        CounterType::Bounty => "When this bountied creature dies, its bounty's owner draws a card and gains 1 life.",
        CounterType::Possession => "DSK Eerie tally — counted by the creature's death-replacement payoff.",
        CounterType::Nest => "DSK Twitching Doll tally — one Spider token per counter when sacrificed.",
        CounterType::Fire => "Firebending tally — the permanent becomes a creature (or scales damage) once enough accumulate.",
        CounterType::Conqueror => "Zhao's conquest tally — while present, nonbasic lands become Mountains.",
        CounterType::Muster => "Assemble the Legion tally — mints a Soldier token per counter each upkeep.",
        CounterType::Acorn => "Chitterspitter tally — Squirrels you control get +1/+1 per acorn.",
        CounterType::Verse => "Verse tally — one added each of your upkeeps; the payoff scales with the total.",
        CounterType::Petal => "Lotus Blossom tally — sacrifice it for that many mana of one colour.",
        CounterType::Page => "Book tally — the host's ability scales with (or is discounted by) the page count.",
        CounterType::Lore => "Saga chapter tally — one added each turn; the matching chapter ability triggers.",
        CounterType::Level => "Level-up tally — reaching a level band grants larger stats and abilities.",
        CounterType::Fade => "Fading — remove one each upkeep; sacrifice this when none are left.",
        CounterType::Age => "Cumulative upkeep tally — one added each upkeep; pay the cost per age counter.",
        CounterType::Defense => "The battle's defense — combat and effects remove them; it's defeated at zero.",
        CounterType::Oil => "Oil tally — fuels this card's 'per oil counter' payoffs, or is removed to pay its abilities.",
        CounterType::Incubation => "Drake Hatcher tally — remove three to mint a 2/2 flying Drake.",
        CounterType::Revival => "Nine-Lives Familiar returns with one fewer each time it dies, until none remain.",
        CounterType::Stash => "Tinybones tally — you may play these exiled cards you don't own.",
        CounterType::Divinity => "Grants indestructibility; remove it to fire this Myojin's one-shot ability.",
        CounterType::Devotion => "A devotion tally — spent by the permanent that banks it.",
        CounterType::Aim => "Banked damage; remove them all to fire the shot.",
        CounterType::Theft => "Damage banked as a tutor cost: remove X to find a mana-value-X card.",
        CounterType::Training => "Marks a creature trained by Sensei Golden-Tail (bushido 1, Samurai).",
        CounterType::Fellowship => "Banner of Kinship tally — the chosen type gets +1/+1 per counter.",
        CounterType::Bait => "Fishing Pole tally — removed on untap to make a 1/1 Fish.",
        CounterType::Supply => "Stocking the Pantry tally — remove one to draw a card.",
        CounterType::Unlock => "Cryptex tally — one per collect-evidence; sacrifice for its payoff at five or more.",
        CounterType::Phylactery => "Phylactery Lich's anchor — the Lich is sacrificed once you control no permanent with one of these.",
        CounterType::Blaze => "Obsidian Fireheart's fire — the land deals 1 damage to its controller each upkeep, for as long as it burns.",
        CounterType::Eon => "Magosi, the Waterveil tally — banked by skipping a turn; cash it in for an extra turn.",
        CounterType::Palliation => "Palliation Accord tally — one per opponent's creature tapped; remove one to prevent 1 damage to you.",
        CounterType::Filibuster => "Azor's Elocutors tally — one each upkeep; at five you win the game (removed when a source deals damage to you).",
        CounterType::Storage => "Banked mana — remove any number to add that much of this land's colour at once.",
        CounterType::Depletion => "This land's remaining taps — it's sacrificed once the last one is spent.",
        CounterType::Fungus => "Sporogenesis tally — this creature mints one Saproling per counter when it dies.",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{build_tooltip_body, companion_restriction_text, humanize_keyword_debug, keyword_label, keyword_reminder, legend_rule_at_risk, prevention_summary};
    use crabomination::card::{CardId, CardType, CounterType, Keyword};
    use crabomination::net::PermanentView;

    /// The humanized fallback splits CamelCase and drops payloads, so an
    /// unlabelled keyword never leaks a raw Rust debug shape into a tooltip.
    #[test]
    fn humanize_keyword_debug_splits_camelcase_and_drops_payload() {
        assert_eq!(humanize_keyword_debug(&Keyword::FirstStrike), "First strike");
        assert_eq!(humanize_keyword_debug(&Keyword::Flying), "Flying");
        assert_eq!(humanize_keyword_debug(&Keyword::Crew(3)), "Crew");
        // Every keyword's rendered label is free of Rust debug punctuation.
        for kw in [Keyword::Trample, Keyword::Menace, Keyword::Crew(2), Keyword::Annihilator(1)] {
            let label = keyword_label(&kw);
            assert!(!label.contains('(') && !label.contains('{'), "debug leaked: {label}");
        }
    }

    fn make_permanent_view(damage: u32, toughness: i32) -> PermanentView {
        PermanentView {
            prevention_remaining: None,
            prevention_source_colors: Vec::new(),
            id: CardId(0),
            name: "Grizzly Bears".into(),
            controller: 0,
            owner: 0,
            card_types: vec![CardType::Creature],
            tapped: false,
            damage,
            summoning_sick: false,
            power: 2,
            toughness,
            base_power: 2,
            base_toughness: 2,
            keywords: vec![],
            counters: vec![],
            attached_to: None,
            is_token: false,
            attacking: false,
            blocking_attackers: Vec::new(),
            triggered_ability_labels: vec![],
            static_ability_labels: vec![],
            activated_ability_labels: vec![],
            abilities: vec![],
            loyalty_abilities: vec![],
            loyalty_uses_remaining: None,
            has_stun_counters: false,
            wont_untap: false,
            has_finality_counters: false,
            dies_to_exile: false,
            dealt_damage_this_turn: false,
            has_shield_counters: false,
            has_prevention_shield: false,
            damage_prevented_as_source: false,
            doomed_next_damage: false,
            goaded: false,
            monstrous: false,
            sector: None,
            suspected: false,
            renowned: false,
            case_solved: None,
            detained: false,
            untap_locked: false,
            pt_modified: false,
            mana_cost_display: String::new(),
            creature_types: vec![],
            ward_cost: 0,
            ward_label: String::new(),
            mana_value: 0,
            is_legendary: false,
            has_plus_one_counters: false,
            has_minus_one_counters: false,
            total_counter_count: 0,
            keyword_counters: vec![],
            shield_counter_count: 0,
            stun_counter_count: 0,
            finality_counter_count: 0,
            regeneration_shields: 0,
            cant_regenerate: false,
            equippable: false,
            equip_token_cost: None,
            crew_value: 0,
            marked_lethal: false,
            named_card: None,
            chosen_color: None,
            chosen_creature_type: None,
            chosen_mode_label: None,
            attachments: vec![],
            attached_to_name: None,
            attached_to_player: None,
            soulbond_partner: None,
            saga_final_chapter: None,
            has_other_face: false,
            transformed: false,
            squad_count: None,
            impending_counters: None,
            face_down: false,
            face_down_name: None,
            prepare_spell_name: None,
            prepare_cost_label: String::new(),
            prepare_needs_target: false,
            creature_subtypes: vec![],
            lost_all_abilities: false,
            colors: vec![],
            crew_power_bonus: 0,
            saddled: false,
            class_level: None,
            station_charges: None,
            station_next_threshold: None,
            crewed_count: 0,
            modified: false,
            can_attack_despite_defender: false,
        }
    }

    #[test]
    fn marked_damage_shows_when_creature_has_damage() {
        let p = make_permanent_view(1, 2);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("marked: 1 damage"), "got: {body}");
        assert!(!body.contains("LETHAL"), "1 damage on a 2-tough body isn't lethal: {body}");
    }

    #[test]
    fn marked_damage_shows_survival_margin_when_not_lethal() {
        // 2 damage on a 5-toughness body → 3 more is lethal.
        let p = make_permanent_view(2, 5);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("marked: 2 damage; 3 more lethal"), "got: {body}");
        assert!(!body.contains("LETHAL"), "not lethal yet: {body}");
    }

    #[test]
    fn marked_damage_calls_out_lethal_when_equal_or_greater_than_toughness() {
        let p = make_permanent_view(2, 2);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("marked: 2 damage"), "got: {body}");
        assert!(body.contains("LETHAL"),
            "2 damage on a 2-tough body should be flagged lethal: {body}");
    }

    #[test]
    fn legendary_marker_surfaces_for_legends_only() {
        let mut p = make_permanent_view(0, 2);
        p.creature_types = vec!["Spirit".into()];
        p.is_legendary = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("Legendary"), "legends are flagged: {body}");

        p.is_legendary = false;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(!body.contains("Legendary"), "non-legends are not: {body}");
    }

    #[test]
    fn marked_damage_hidden_when_zero() {
        let p = make_permanent_view(0, 2);
        // No counters, no abilities, no other lines — body might be None.
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("marked:"), "no damage marked, should not surface: {s}");
        }
    }

    #[test]
    fn alt_cast_keywords_have_reminder_text() {
        use crabomination::card::Keyword as K;
        use crabomination::mana::{cost, generic};
        // Previously these fell through to `None` (no tooltip line).
        for kw in [
            K::Cycling(cost(&[generic(2)])),
            K::Kicker(cost(&[generic(1)])),
            K::Flashback(cost(&[generic(3)])),
            K::Echo(cost(&[generic(2)])),
            K::Impending(3),
            K::Casualty(2),
            K::Saddle(3),
        ] {
            assert!(keyword_reminder(&kw).is_some(), "missing reminder for {kw:?}");
        }
    }

    #[test]
    fn newly_covered_keywords_have_reminder_text() {
        use crabomination::card::{CardType, CounterType, Keyword as K};
        // Previously these fell through to `None` (no tooltip line).
        for kw in [
            K::TrampleOverPlaneswalkers,
            K::Compleated,
            K::ProtectionFromCardType(CardType::Artifact),
            K::HexproofFromMonocolored,
            K::EchoDiscard,
            K::DoesntUntapWhileCounter(CounterType::Stun),
        ] {
            assert!(keyword_reminder(&kw).is_some(), "missing reminder for {kw:?}");
        }
    }

    /// These keywords carry reminder text but used to fall through to the raw
    /// `{:?}` debug shape for their short label; assert they now read cleanly.
    #[test]
    fn newly_covered_keywords_have_readable_labels() {
        use crabomination::card::{CardType, CounterType, Keyword as K};
        assert_eq!(
            keyword_label(&K::TrampleOverPlaneswalkers),
            "Trample over planeswalkers",
        );
        assert_eq!(keyword_label(&K::Compleated), "Compleated");
        assert_eq!(keyword_label(&K::HexproofFromMonocolored), "Hexproof from monocolored");
        assert_eq!(keyword_label(&K::EchoDiscard), "Echo—Discard");
        assert_eq!(
            keyword_label(&K::ProtectionFromCardType(CardType::Artifact)),
            "Protection from Artifacts",
        );
        assert_eq!(
            keyword_label(&K::DoesntUntapWhileCounter(CounterType::Stun)),
            "Doesn't untap with a Stun counter",
        );
    }

    #[test]
    fn counter_reminder_line_shows_for_counters_with_reminder_text() {
        let mut p = make_permanent_view(0, 3);
        p.counters = vec![(CounterType::Stun, 1)];
        let body = build_tooltip_body(&p).expect("body should render");
        assert!(body.contains("Stun ×1"), "counter label present: {body}");
        assert!(body.contains("Skips its next untap"), "reminder line present: {body}");
    }

    #[test]
    fn counter_without_reminder_shows_no_reminder_line() {
        let mut p = make_permanent_view(0, 3);
        p.counters = vec![(CounterType::PlusOnePlusOne, 2)];
        let body = build_tooltip_body(&p).expect("body should render");
        assert!(body.contains("+1/+1 ×2"), "counter label present: {body}");
        assert!(!body.contains("·"), "no reminder bullet for self-explanatory counters: {body}");
    }

    #[test]
    fn creature_type_line_renders_when_creature_types_present() {
        let mut p = make_permanent_view(0, 2);
        p.creature_types = vec!["Bear".into()];
        let body = build_tooltip_body(&p).expect("body should render");
        assert!(body.contains("Type: Bear"), "got: {body}");
    }

    #[test]
    fn attachments_render_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.attachments = vec!["Gift of Orzhova".into(), "Shuko".into()];
        let body = build_tooltip_body(&p).expect("body should render");
        assert!(body.contains("Attached: Gift of Orzhova, Shuko"), "got: {body}");
    }

    #[test]
    fn equipment_host_renders_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.card_types = vec![CardType::Artifact];
        p.attached_to_name = Some("Grizzly Bears".into());
        let body = build_tooltip_body(&p).expect("body should render");
        assert!(body.contains("Equipping: Grizzly Bears"), "got: {body}");
    }

    #[test]
    fn creature_type_line_hidden_for_non_creature_even_when_subtypes_set() {
        let mut p = make_permanent_view(0, 2);
        p.card_types = vec![CardType::Enchantment];
        p.creature_types = vec!["Bear".into()];
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("Type:"), "non-creature should not show Type: line, got: {s}");
        }
    }

    #[test]
    fn marked_damage_unused_for_non_creature() {
        let mut p = make_permanent_view(3, 2);
        // Re-shape as an enchantment — damage on non-creatures is bogus,
        // but if we ever stamp it (engine bug), the tooltip should hide it.
        p.card_types = vec![CardType::Enchantment];
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("marked:"), "non-creature should never surface damage: {s}");
        }
    }

    #[test]
    fn keyword_counter_surfaces_in_tooltip() {
        use crabomination::card::Keyword;
        let mut p = make_permanent_view(0, 2);
        p.keyword_counters = vec![(Keyword::Flying, 1)];
        let body = build_tooltip_body(&p).expect("tooltip should render");
        // Surface a flying counter.
        assert!(body.to_lowercase().contains("flying"), "got: {body}");
    }

    #[test]
    fn shield_counter_count_renders_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.shield_counter_count = 3;
        p.has_shield_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        // Should show the explicit count, not just the boolean.
        assert!(body.contains("shielded ×3"), "got: {body}");
    }

    #[test]
    fn shield_counter_single_falls_back_to_boolean_message() {
        let mut p = make_permanent_view(0, 2);
        p.shield_counter_count = 1;
        p.has_shield_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.to_lowercase().contains("shielded"), "got: {body}");
        assert!(!body.contains("shielded ×"), "single shield uses boolean form: {body}");
    }

    #[test]
    fn stun_counter_count_renders_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.stun_counter_count = 2;
        p.has_stun_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("stunned ×2"), "got: {body}");
    }

    #[test]
    fn wont_untap_lock_renders_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.wont_untap = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.to_lowercase().contains("locked"), "got: {body}");
    }

    #[test]
    fn stun_takes_precedence_over_generic_untap_lock() {
        // A stunned permanent is also `wont_untap`, but shows the stun line,
        // not the generic lock line (they'd be redundant).
        let mut p = make_permanent_view(0, 2);
        p.has_stun_counters = true;
        p.wont_untap = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("stunned"), "got: {body}");
        assert!(!body.contains("locked:"), "no duplicate lock line: {body}");
    }

    #[test]
    fn no_keyword_counter_no_keyword_line() {
        let p = make_permanent_view(0, 2);
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.to_lowercase().contains("counter granting"),
                "no keyword counters: {s}");
        }
    }

    // Silence unused-import warnings for items only used in some tests.
    #[allow(dead_code)]
    fn _ensure_counter_type_import_used() {
        let _ = CounterType::PlusOnePlusOne;
    }

    #[test]
    fn attacking_status_surfaces_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.attacking = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(attacking)"), "got: {body}");
    }

    #[test]
    fn blocking_status_shows_attacker_id() {
        let mut p = make_permanent_view(0, 2);
        p.blocking_attackers = vec![CardId(7)];
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(blocking #7)"), "got: {body}");
    }

    #[test]
    fn suspect_goad_monstrous_badges_surface_in_tooltip() {
        let mut p = make_permanent_view(0, 2);
        p.suspected = true;
        p.goaded = true;
        p.monstrous = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("suspected"), "got: {body}");
        assert!(body.contains("goaded"), "got: {body}");
        assert!(body.contains("monstrous"), "got: {body}");
    }

    #[test]
    fn combat_status_hidden_when_idle() {
        let p = make_permanent_view(0, 2);
        let body = build_tooltip_body(&p);
        if let Some(s) = body {
            assert!(!s.contains("(attacking)"), "no attack flag: {s}");
            assert!(!s.contains("(blocking"), "no block flag: {s}");
        }
    }

    #[test]
    fn plus_one_counters_show_numeric_pt_delta() {
        // Push (claude/modern_decks batch 205): the +1/+1 badge now shows
        // the actual swing read off the counters vec — useful for enrage
        // creatures that have stacked several counters.
        let mut p = make_permanent_view(0, 4);
        p.counters = vec![(CounterType::PlusOnePlusOne, 3)];
        p.has_plus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(boosted: +3/+3 from 3 +1/+1 counters)"),
            "expected numeric +1/+1 badge: {body}");
    }

    #[test]
    fn single_plus_one_counter_uses_singular_wording() {
        let mut p = make_permanent_view(0, 3);
        p.counters = vec![(CounterType::PlusOnePlusOne, 1)];
        p.has_plus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(boosted: +1/+1 from 1 +1/+1 counter)"),
            "expected singular wording: {body}");
        assert!(!body.contains("counters)"), "no plural 's' on a single counter: {body}");
    }

    #[test]
    fn minus_one_counters_show_numeric_pt_delta() {
        let mut p = make_permanent_view(0, 5);
        p.counters = vec![(CounterType::MinusOneMinusOne, 2)];
        p.has_minus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(weakened: -2/-2 from 2 -1/-1 counters)"),
            "expected numeric -1/-1 badge: {body}");
    }

    #[test]
    fn plus_one_badge_falls_back_to_boolean_without_explicit_count() {
        // Older server projection: helper flag set but counters vec empty.
        let mut p = make_permanent_view(0, 3);
        p.has_plus_one_counters = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(boosted: +1/+1 counters)"),
            "expected legacy boolean badge fallback: {body}");
    }

    #[test]
    fn regeneration_shield_badge_shows_singular_and_plural() {
        let mut p = make_permanent_view(0, 2);
        p.regeneration_shields = 1;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(regen: next destruction taps & heals instead of dying)"),
            "expected singular regen badge: {body}");

        p.regeneration_shields = 3;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(regen ×3: absorbs 3 destructions this turn)"),
            "expected plural regen badge: {body}");

        // CR 701.15g — a blanked shield must not promise a save.
        p.cant_regenerate = true;
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("(can't be regenerated this turn)") && !body.contains("(regen"),
            "expected the regen badge to be replaced: {body}");
    }

    #[test]
    fn companion_rules_render_restriction_text() {
        use crabomination::card::CompanionRule as C;
        for rule in [
            C::PermanentsManaValueAtMost(2), C::NonlandManaValueAtLeast(3),
            C::NonlandEvenManaValue, C::NonlandOddManaValue, C::NoDuplicateManaSymbols,
            C::Singleton, C::CreatureTypesAmong(vec![]), C::NonlandShareACardType,
            C::DeckSizeAtLeastOverMinimum(20), C::PermanentsHaveActivatedAbility,
        ] {
            assert!(!companion_restriction_text(&rule).is_empty(), "text for {rule:?}");
        }
    }

    #[test]
    fn evasion_keywords_carry_reminder_text() {
        use crabomination::card::Keyword;
        for kw in [Keyword::Prowess, Keyword::Fear, Keyword::Skulk,
                   Keyword::Shadow, Keyword::Unblockable, Keyword::Changeling,
                   Keyword::Flash, Keyword::Intimidate, Keyword::Horsemanship,
                   Keyword::CantBeBlockedByMoreThanOne, Keyword::CantBeBlockedExceptByN(2)] {
            assert!(keyword_reminder(&kw).is_some(),
                "expected reminder text for {kw:?}");
        }
    }

    /// The creatures-you-control Firebending variant surfaces in the reminder
    /// panel and ability label like the fixed/power variants (Sun Warriors).
    #[test]
    fn firebending_creatures_variant_has_ui_text() {
        use crabomination::card::Keyword;
        assert!(keyword_reminder(&Keyword::FirebendingCreaturesYouControl).is_some());
        assert_eq!(keyword_label(&Keyword::FirebendingCreaturesYouControl), "Firebending X");
    }

    #[test]
    fn combat_and_cost_keywords_carry_reminder_text() {
        use crabomination::card::Keyword;
        use crabomination::mana::{Color, ManaCost};
        for kw in [
            Keyword::MustAttack, Keyword::CantBlock, Keyword::CantAttack,
            Keyword::Gravestorm, Keyword::Unleash, Keyword::AttacksAlone,
            Keyword::CantBeCopied, Keyword::DealsNoCombatDamage,
            Keyword::Protection(Color::Red), Keyword::Bushido(2),
            Keyword::Rampage(1), Keyword::Crew(3), Keyword::Madness(ManaCost::default()),
            // Restriction / poison keywords that previously fell through to None.
            Keyword::Poisonous(1), Keyword::CanBlockOnlyFlying,
            Keyword::CantAttackOrBlockUnlessHandSizeAtMost(1),
            Keyword::CantAttackOrBlockUnlessDelirium,
        ] {
            assert!(keyword_reminder(&kw).is_some(),
                "expected reminder text for {kw:?}");
        }
    }

    #[test]
    fn alt_cost_and_protection_keywords_carry_reminder_text() {
        use crabomination::card::{CreatureType, Keyword, SpellSubtype};
        use crabomination::mana::ManaCost;
        for kw in [
            Keyword::Disturb(ManaCost::default()), Keyword::Entwine(ManaCost::default()),
            Keyword::Epic, Keyword::Improvise, Keyword::JumpStart,
            Keyword::Replicate(ManaCost::default()),
            Keyword::Splice(ManaCost::default(), SpellSubtype::Arcane),
            Keyword::Squad(ManaCost::default()), Keyword::UmbraArmor, Keyword::Companion,
            Keyword::ProtectionFromColoredSpells, Keyword::ProtectionFromSpells,
            Keyword::ProtectionFromCreatures, Keyword::ProtectionFromMulticolored,
            Keyword::ProtectionFromManaValueExcept(3),
            Keyword::ProtectionFromCreatureType(CreatureType::Human),
            Keyword::ProtectionFromSpellSubtype(SpellSubtype::Arcane),
            Keyword::ProtectionFromInstants, Keyword::ProtectionFromEverything,
        ] {
            assert!(keyword_reminder(&kw).is_some(),
                "expected reminder text for {kw:?}");
        }
    }

    #[test]
    fn graveyard_recast_and_restriction_keywords_carry_reminder_text() {
        use crabomination::card::Keyword;
        use crabomination::mana::ManaCost;
        for kw in [
            Keyword::Mayhem(ManaCost::default()),
            Keyword::Harmonize(ManaCost::default()),
            Keyword::CantActivateAbilities,
            Keyword::CantAttackUnlessCastCreatureThisTurn,
            Keyword::CantAttackOrBlockUnlessEvenCounters,
            Keyword::CantBeCounteredIfXAtLeast(5),
            Keyword::CantAttackOrBlockUnlessCreatureDiedThisTurn,
        ] {
            assert!(keyword_reminder(&kw).is_some(),
                "expected reminder text for {kw:?}");
        }
    }

    #[test]
    fn impending_countdown_shows_in_tooltip() {
        let mut p = make_permanent_view(0, 4);
        p.impending_counters = Some(3);
        let body = build_tooltip_body(&p).expect("tooltip should render");
        assert!(body.contains("impending: becomes a creature in 3 end steps"), "got: {body}");
    }

    #[test]
    fn new_keywords_have_printed_labels_not_debug_shape() {
        use crabomination::card::Keyword;
        use crabomination::mana::ManaCost;
        let recon = keyword_label(&Keyword::Reconfigure(ManaCost::default()));
        assert!(recon.starts_with("Reconfigure"), "got {recon}");
        let parity = keyword_label(&Keyword::CantAttackOrBlockUnlessEvenCounters);
        assert!(parity.contains("even number of counters"), "got {parity}");
        assert!(!parity.contains("CantAttack"), "no raw debug shape: {parity}");
    }

    /// The legend-rule warning fires only for a legendary permanent with a
    /// same-named, same-controller legendary twin in play.
    #[test]
    fn legend_rule_at_risk_detects_duplicate() {
        let mut a = make_permanent_view(0, 2);
        a.id = CardId(1);
        a.name = "Ulasht, the Hate Seed".into();
        a.is_legendary = true;
        let mut b = a.clone();
        b.id = CardId(2);
        // A lone legend is safe; its twin makes both at risk.
        assert!(!legend_rule_at_risk(std::slice::from_ref(&a), &a));
        let board = vec![a.clone(), b.clone()];
        assert!(legend_rule_at_risk(&board, &a), "duplicate triggers the warning");
        // A non-legendary permanent never warns, even with a same-name twin.
        let mut c = a.clone();
        c.is_legendary = false;
        let mut d = c.clone();
        d.id = CardId(3);
        assert!(!legend_rule_at_risk(&[c.clone(), d], &c));
        // Different controller isn't a legend-rule pair.
        let mut e = b.clone();
        e.controller = 1;
        assert!(!legend_rule_at_risk(&[a.clone(), e], &a));
    }

    /// A CR 615 shield reads as its remaining points and (when scoped) its
    /// source colors, not a bare "warded" badge.
    #[test]
    fn prevention_summary_reports_points_and_colors() {
        use crabomination::mana::Color;
        assert_eq!(prevention_summary(None, &[]), "all damage prevented this turn");
        assert_eq!(prevention_summary(Some(3), &[]), "prevents the next 3 damage");
        assert_eq!(prevention_summary(Some(1), &[]), "prevents the next 1 damage");
        assert_eq!(
            prevention_summary(None, &[Color::Red]),
            "all damage prevented this turn from red sources",
        );
        assert_eq!(
            prevention_summary(Some(2), &[Color::White, Color::Blue]),
            "prevents the next 2 damage from white/blue sources",
        );
    }

    /// The two Darksteel/Fifth Dawn keywords carry reminder text.
    #[test]
    fn modular_and_sunburst_have_reminders() {
        assert!(keyword_reminder(&Keyword::Modular(3)).unwrap().contains("+1/+1 counters"));
        assert!(keyword_reminder(&Keyword::Sunburst).unwrap().contains("color of mana spent"));
    }
}
