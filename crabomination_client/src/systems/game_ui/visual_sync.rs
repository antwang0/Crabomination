//! Visual sync: reconcile the 3-D card entities — battlefield, hands, piles,
//! command zones — with the server-projected view, and animate them between
//! zones.

use super::*;

/// In-flight bf→{hand, graveyard} animation queries bundled together so
/// `sync_game_visuals` stays under Bevy's 16-param tuple limit.
/// `hand_zoom` rides along here for the same reason — sync_game_visuals
/// needs the current zoom to compute hand-target transforms.
#[derive(bevy::ecs::system::SystemParam)]
pub struct InFlightAnims<'w, 's> {
    pub gy: Query<'w, 's, &'static SendToGraveyardAnimation>,
    /// The shared exile pile's visual — bundled here for the same reason
    /// everything else in this struct is: the 16-parameter system cap.
    pub exile_pile: Query<
        'w,
        's,
        (
            &'static mut Transform,
            &'static mut Visibility,
            &'static mut CardHoverLift,
        ),
        (
            With<crate::card::ExilePile>,
            Without<DeckPile>,
            Without<GraveyardPile>,
            Without<GameCardId>,
        ),
    >,
    pub to_hand: Query<'w, 's, (&'static GameCardId, &'static crate::card::ReturnToHandAnimation)>,
    pub hand_zoom: Res<'w, crate::card::HandZoom>,
    pub gameplay: Res<'w, crate::config::GameplayConfig>,
    /// Where card art is on disk, so an art-less token can be drawn a face
    /// (`battlefield_face_path`). Absent on the web build.
    pub art: Option<Res<'w, crate::scryfall::CardArtDir>>,
}

/// The image a permanent entering the battlefield shows: its card art, or —
/// for a token whose art isn't on disk — a drawn face (`card::proxy`) with
/// its name, colours, types, keywords and base P/T. A token's art is
/// fetched only for a short list of common tokens, so without this most
/// tokens were a white card with their name in faint grey.
fn battlefield_face_path(c: &crabomination::net::PermanentView, art: Option<&crate::scryfall::CardArtDir>) -> String {
    if !c.is_token || art.is_none_or(|a| a.has_art(&c.name)) {
        return crate::scryfall::card_asset_path(&c.name);
    }
    let types: Vec<String> = c.card_types.iter().map(|t| format!("{t:?}")).collect();
    let mut type_line = format!("Token {}", types.join(" "));
    if !c.creature_types.is_empty() {
        type_line.push_str(" — ");
        type_line.push_str(&c.creature_types.join(" "));
    }
    let face = crate::card::proxy::ProxyFace {
        name: c.name.clone(),
        colors: c.colors.clone(),
        type_line,
        keywords: c.keywords.iter().map(crate::systems::counter_tooltip::keyword_label).collect(),
        pt: c.is_creature().then_some((c.base_power, c.base_toughness)),
    };
    crate::card::proxy::proxy_asset_path(&face)
}

/// Process `SwapFrontMaterial` markers: walk each entity's children,
/// find the `FrontFaceMesh` child, swap its `MeshMaterial3d` to the
/// requested handle, update the parent's `CardFrontTexture` so peek /
/// hover popups stay in sync, and remove the marker. Used by the
/// hand→battlefield transition for flipped MDFCs (the played-back-face
/// image needs to land on the front-child mesh under standard bf
/// orientation).
#[allow(clippy::type_complexity)]
pub fn apply_swap_front_material(
    mut commands: Commands,
    swap_q: Query<
        (Entity, &Children, &crate::card::SwapFrontMaterial),
        With<crate::card::SwapFrontMaterial>,
    >,
    mut front_meshes: Query<
        &mut MeshMaterial3d<StandardMaterial>,
        With<crate::card::FrontFaceMesh>,
    >,
    mut tex_q: Query<&mut crate::card::CardFrontTexture>,
) {
    for (entity, children, swap) in &swap_q {
        for child in children.iter() {
            if let Ok(mut mat) = front_meshes.get_mut(child) {
                *mat = MeshMaterial3d(swap.new_front.clone());
                break;
            }
        }
        if let Ok(mut tex) = tex_q.get_mut(entity) {
            tex.0 = swap.new_path.clone();
        }
        commands
            .entity(entity)
            .remove::<crate::card::SwapFrontMaterial>();
    }
}

// ── Visual sync: reconcile 3D card entities with the server-projected view ───

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
/// Move a pile to rest at `pos` — only when it has moved. These run every
/// frame (above `sync_game_visuals`' change gate), and an unconditional write
/// re-propagates and re-extracts every pile card each frame and keeps the
/// reactive frame loop awake. The lift is re-applied here so a hovered pile
/// stays up whichever of this and `animate_hover_lift` runs first.
fn rest_pile_at(
    transform: &mut Mut<Transform>,
    lift: &mut Mut<crate::card::CardHoverLift>,
    pos: Vec3,
) {
    if lift.base_translation != pos {
        lift.base_translation = pos;
        transform.translation = pos + Vec3::Y * lift.current_lift;
    }
}

pub fn sync_game_visuals(
    mut commands: Commands,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // Paired: a system takes at most sixteen parameters.
    (card_assets, camera_home): (Option<Res<CardMeshAssets>>, Res<crate::systems::camera_zoom::CameraHome>),
    removed_animating: RemovedComponents<Animating>,
    hand_cards: Query<
        (
            Entity,
            &GameCardId,
            &Transform,
            Option<&StackCard>,
            &CardHoverLift,
            Option<&crate::card::FlippedFace>,
        ),
        (With<HandCard>, Without<Animating>),
    >,
    bf_cards: Query<
        (
            Entity,
            &GameCardId,
            &CardOwner,
            &BattlefieldCard,
            &Transform,
            (Option<&TapState>, Option<&CardHoverLift>),
        ),
        (Without<HandCard>, Without<Animating>),
    >,
    mut deck_pile_q: Query<
        (Entity, &DeckPile, &mut Transform, &mut CardHoverLift),
        (Without<GameCardId>, Without<OpponentHandCard>, Without<crate::card::ExilePile>),
    >,
    mut graveyard_q: Query<
        (
            &GraveyardPile,
            &mut Transform,
            &mut Visibility,
            &mut CardHoverLift,
            &mut MeshMaterial3d<StandardMaterial>,
        ),
        (Without<DeckPile>, Without<GameCardId>, Without<crate::card::ExilePile>),
    >,
    opponent_hand_q: Query<
        (Entity, &OpponentHandCard, &Transform, Has<Animating>),
        (Without<DeckPile>, Without<GraveyardPile>, Without<crate::card::ExilePile>),
    >,
    mut inflight: InFlightAnims,
    all_bf_entities: Query<&GameCardId, With<BattlefieldCard>>,
    all_hand_entity_ids: Query<&GameCardId, With<HandCard>>,
    // Entities currently on the stack (StackCard but not HandCard = opponent cards).
    all_stack_entities: Query<(Entity, &GameCardId, &Transform), (With<StackCard>, Without<HandCard>)>,
) {
    let Some(cv) = &view.0 else { return };
    let viewer = cv.your_seat;
    let n_seats = cv.players.len();
    // How far a duel's crowded creature rows have pushed the piles beside
    // them.
    let spread = crate::card::Spread::of(&cv.battlefield, n_seats);
    // A spectator's `viewer` is the sentinel `SPECTATOR_SEAT`, which indexes no
    // real player — they have no hand or library of their own. Resolve the
    // viewer's hand/library through these panic-safe accessors so the board
    // still renders for spectators (empty viewer hand, zero-height deck).
    let gy_sizes: Vec<usize> = cv.players.iter().map(|p| p.graveyard.len()).collect();
    let gy_size = |owner: usize| gy_sizes.get(owner).copied().unwrap_or(0);
    let deck_size = |owner: usize| {
        cv.players
            .iter()
            .find(|p| p.seat == owner)
            .map(|p| p.library.size)
            .unwrap_or(0)
    };

    // ── Always update: deck pile heights, spawn/despawn to match deck sizes ───

    // Existing pile cards: shrink to current deck sizes.
    let mut deck_pile_counts: std::collections::HashMap<usize, usize> =
        std::collections::HashMap::new();
    for (entity, pile, mut transform, mut lift) in &mut deck_pile_q {
        let size = deck_size(pile.owner);
        if pile.index >= size {
            commands.entity(entity).despawn();
        } else {
            let base = deck_position(pile.owner, viewer, n_seats);
            let y = pile.index as f32 * crate::card::pile_step(size) + 0.01;
            let pos = Vec3::new(base.x, y, base.z);
            rest_pile_at(&mut transform, &mut lift, pos);
            *deck_pile_counts.entry(pile.owner).or_default() += 1;
        }
    }
    // Spawn missing deck pile cards per seat to match library size.
    if let Some(card_assets_ref) = &card_assets {
        for seat in 0..n_seats {
            let target_size = deck_size(seat);
            let current = deck_pile_counts.get(&seat).copied().unwrap_or(0);
            if current >= target_size {
                continue;
            }
            let base = deck_position(seat, viewer, n_seats);
            let rot = back_face_rotation(seat, viewer, n_seats);
            for i in current..target_size {
                let y = i as f32 * crate::card::pile_step(target_size) + 0.01;
                let pos = Vec3::new(base.x, y, base.z);
                commands.spawn((
                    Mesh3d(card_assets_ref.card_mesh.clone()),
                    MeshMaterial3d(card_assets_ref.back_material.clone()),
                    Transform::from_translation(pos).with_rotation(rot),
                    Visibility::default(),
                    DeckPile { owner: seat, index: i },
                    CardHoverLift { current_lift: 0.0, target_lift: 0.0, base_translation: pos },
                ));
            }
        }
    }

    let mut gy_in_flight: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
    for anim in &inflight.gy {
        *gy_in_flight.entry(anim.owner).or_default() += 1;
    }
    // A card that has just left the battlefield/hand for a graveyard still has
    // its on-board entity *this* frame — the `SendToGraveyardAnimation` insert
    // and `BattlefieldCard`/`HandCard` removal are deferred commands, so the
    // entity is still a `BattlefieldCard`/`HandCard` and `inflight.gy` can't
    // see the new animation yet. Without counting it, the pile pops the card in
    // at full count while the soon-to-fly board copy is still drawn — the "two
    // copies" flicker. So treat any live battlefield/hand entity whose id is
    // already in a graveyard view as in-flight too. (Next frame it carries
    // `SendToGraveyardAnimation` and is counted via `inflight.gy` instead, so
    // there's no double-count.)
    let gy_id_sets: Vec<HashSet<CardId>> = cv
        .players
        .iter()
        .map(|p| p.graveyard.iter().map(|c| c.id).collect())
        .collect();
    for (_, gid, owner, _, _, _) in &bf_cards {
        if gy_id_sets.get(owner.0).is_some_and(|s| s.contains(&gid.0)) {
            *gy_in_flight.entry(owner.0).or_default() += 1;
        }
    }
    for (_, gid, _, _, _, _) in &hand_cards {
        if gy_id_sets.get(viewer).is_some_and(|s| s.contains(&gid.0)) {
            *gy_in_flight.entry(viewer).or_default() += 1;
        }
    }

    // The shared exile pile: one stack for the whole zone, as tall as the
    // number of exiled cards, hidden while exile is empty.
    for (mut transform, mut vis, mut lift) in &mut inflight.exile_pile {
        if cv.exile.is_empty() {
            vis.set_if_neq(Visibility::Hidden);
        } else {
            vis.set_if_neq(Visibility::Visible);
            let base = crate::card::exile_position(n_seats, &spread);
            let y = cv.exile.len() as f32 * DECK_CARD_Y_STEP + 0.01;
            rest_pile_at(&mut transform, &mut lift, Vec3::new(base.x, y, base.z));
        }
    }

    for (gy, mut transform, mut vis, mut lift, _mat) in &mut graveyard_q {
        let gy_count = gy_size(gy.owner);
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_count.saturating_sub(in_flight);
        if arrived == 0 {
            vis.set_if_neq(Visibility::Hidden);
        } else {
            vis.set_if_neq(Visibility::Visible);
            let base_pos = graveyard_position(gy.owner, viewer, n_seats, &spread);
            let y = arrived as f32 * DECK_CARD_Y_STEP + 0.01;
            rest_pile_at(&mut transform, &mut lift, Vec3::new(base_pos.x, y, base_pos.z));
        }
    }

    if !view.is_changed() && removed_animating.is_empty() {
        return;
    }
    let Some(card_assets) = card_assets else { return };

    // Everything below (the hand sort in particular, which clones a String
    // per card) runs only on a view change or animation completion — keep it
    // after the early-out above.
    let empty_hand: Vec<crabomination::net::HandCardView> = Vec::new();
    let viewer_hand = cv.players.get(viewer).map(|p| &p.hand).unwrap_or(&empty_hand);
    // Client-side hand sort (config `gameplay.sort_hand`): lands first,
    // then by mana value, then name — Arena-style layout regardless of
    // draw order. Hidden cards keep server order at the end. Purely a
    // display ordering; every interaction below keys off card ids.
    let sorted_hand: Vec<crabomination::net::HandCardView>;
    let viewer_hand: &Vec<crabomination::net::HandCardView> = if inflight.gameplay.sort_hand {
        let mut h = viewer_hand.clone();
        h.sort_by_key(|c| match c {
            crabomination::net::HandCardView::Known(k) => (
                if k.card_types.contains(&crabomination::card::CardType::Land) { 0 } else { 1 },
                k.cost.cmc(),
                k.name.clone(),
            ),
            crabomination::net::HandCardView::Hidden { .. } => (2, 0, String::new()),
        });
        sorted_hand = h;
        &sorted_hand
    } else {
        viewer_hand
    };
    let viewer_lib_size = cv.players.get(viewer).map(|p| p.library.size).unwrap_or(0);

    // Snapshot of every opponent hand visual entity (entity, owner, slot, pos,
    // rot, is_animating). We feed a per-owner hand pool so an opponent
    // "playing" a card consumes one of their face-down hand visuals as the
    // animation start position.
    let all_opp_hand: Vec<(Entity, usize, usize, Vec3, Quat, bool)> = opponent_hand_q
        .iter()
        .map(|(e, bh, t, is_animating)| {
            (e, bh.owner, bh.slot, t.translation, t.rotation, is_animating)
        })
        .collect();

    // Per-opponent hand pool of available (non-animating) cards to consume on
    // a hand→battlefield promotion. Sort by slot so we consume in stable
    // order, then drop the slot field.
    let mut pool_with_slot: std::collections::HashMap<usize, Vec<(Entity, usize, Vec3, Quat)>> =
        std::collections::HashMap::new();
    for (e, owner, slot, pos, rot, is_anim) in &all_opp_hand {
        if *is_anim {
            continue;
        }
        pool_with_slot
            .entry(*owner)
            .or_default()
            .push((*e, *slot, *pos, *rot));
    }
    for pool in pool_with_slot.values_mut() {
        pool.sort_by_key(|(_, slot, _, _)| *slot);
    }
    let mut hand_pool_by_owner: std::collections::HashMap<usize, Vec<(Entity, Vec3, Quat)>> =
        pool_with_slot
            .into_iter()
            .map(|(owner, pool)| {
                (
                    owner,
                    pool.into_iter().map(|(e, _, pos, rot)| (e, pos, rot)).collect(),
                )
            })
            .collect();
    let mut promoted: HashSet<Entity> = HashSet::new();

    // ── Update graveyard pile face-up material ────────────────────────────────
    for (gy, _transform, _vis, _lift, mut mat) in &mut graveyard_q {
        let in_flight = gy_in_flight.get(&gy.owner).copied().unwrap_or(0);
        let arrived = gy_size(gy.owner).saturating_sub(in_flight);
        if arrived > 0 {
            let top_name: Option<String> = cv.players[gy.owner].graveyard.get(arrived - 1).map(|c| c.name.clone());
            // A new material only when the top card's art changes: this runs
            // on every view, and a fresh material per view re-specialized
            // and re-uploaded the pile and left the old one to the allocator.
            let showing = materials
                .get(&mat.0)
                .and_then(|m| m.base_color_texture.as_ref())
                .and_then(|t| asset_server.get_path(t.id()));
            if let Some(name) = top_name
                && showing.is_none_or(|p| p.path() != std::path::Path::new(&crate::scryfall::card_asset_path(&name)))
            {
                *mat = MeshMaterial3d(card_front_material(&name, &mut materials, &asset_server));
            }
        }
    }

    // ── Build zone ID sets ────────────────────────────────────────────────────
    let stack_ids: HashSet<CardId> = cv.stack.iter().filter_map(|item| {
        if let StackItemView::Known(k) = item { Some(k.source) } else { None }
    }).collect();
    // Sources of *spell* stack items only. A viewer hand card is pulled out to
    // the stack only when the card itself is a spell on the stack (cast from
    // hand). A trigger or ability whose source merely happens to be a card in
    // hand — e.g. Chancellor of the Tangle's opening-hand mana trigger — must
    // NOT yank the source card out of the hand (it never left), or it strands
    // mid-table once the trigger resolves.
    let stack_spell_ids: HashSet<CardId> = cv.stack.iter().filter_map(|item| {
        if let StackItemView::Known(k) = item {
            (k.kind == crabomination::net::StackItemKind::Spell).then_some(k.source)
        } else { None }
    }).collect();
    let hand_ids: HashSet<CardId> = viewer_hand.iter().map(|c| c.id()).collect();
    let bf_ids_by_owner: std::collections::HashMap<usize, HashSet<CardId>> = (0..n_seats)
        .map(|seat| {
            (
                seat,
                cv.battlefield.iter().filter(|c| c.owner == seat).map(|c| c.id).collect(),
            )
        })
        .collect();
    // Server data drives the owner seat here — fall back to an empty set
    // rather than panicking on a seat outside 0..n_seats.
    static EMPTY_BF_IDS: std::sync::LazyLock<HashSet<CardId>> =
        std::sync::LazyLock::new(HashSet::new);
    let bf_ids_for = |seat: usize| -> &HashSet<CardId> {
        bf_ids_by_owner.get(&seat).unwrap_or(&EMPTY_BF_IDS)
    };
    let hand_total = hand_ids.len();
    let all_bf_ids: HashSet<CardId> = cv.battlefield.iter().map(|c| c.id).collect();
    let visual_bf_ids: HashSet<CardId> = all_bf_entities.iter().map(|gid| gid.0).collect();
    // Opponent stack-card entities that already have a 3-D visual.
    let visual_opp_stack_ids: HashSet<CardId> =
        all_stack_entities.iter().map(|(_, gid, _)| gid.0).collect();
    // IDs visible in any player's graveyard. Used to disambiguate where a
    // disappeared hand card actually went: present in a graveyard → was
    // discarded/resolved-as-spell → fly to graveyard. Absent → it was
    // shuffled back to library (mulligan) → fly to deck pile.
    let in_any_graveyard: HashSet<CardId> = cv
        .players
        .iter()
        .flat_map(|p| p.graveyard.iter().map(|c| c.id))
        .collect();
    // Top-of-deck position for the viewer (used as the destination of a
    // mulligan put-back animation and the start of fetch/tutor animations).
    let viewer_deck_base = deck_position(viewer, viewer, n_seats);
    let viewer_deck_top_y = crate::card::pile_height(viewer_lib_size) + 0.5;
    let viewer_deck_top = Vec3::new(viewer_deck_base.x, viewer_deck_top_y, viewer_deck_base.z);
    let viewer_deck_back_rot = back_face_rotation(viewer, viewer, n_seats);

    let hand_zoom = inflight.hand_zoom.0;

    // ── Spawn viewer hand cards that have no visual entity yet ───────────────
    {
        let has_entity: HashSet<CardId> = all_hand_entity_ids.iter().map(|gid| gid.0)
            // A bf permanent currently animating back to hand will become
            // a HandCard entity once the animation completes — don't spawn
            // a duplicate hand card in the meantime.
            .chain(inflight.to_hand.iter().filter_map(|(gid, anim)| {
                anim.to_viewer.then_some(gid.0)
            }))
            .collect();
        let deck_base = deck_position(viewer, viewer, n_seats);
        let deck_y = crate::card::pile_height(viewer_lib_size) + 0.5;
        let deck_pos = Vec3::new(deck_base.x, deck_y, deck_base.z);
        for (slot, card_view) in viewer_hand.iter().enumerate() {
            use crabomination::net::HandCardView;
            let HandCardView::Known(known) = card_view else { continue };
            let card_id = known.id;
            if has_entity.contains(&card_id) { continue; }
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total, hand_zoom);
            let front_mat = card_front_material(&known.name, &mut materials, &asset_server);
            // For MDFC cards (Pathways), paint the back-child with the
            // back face's Scryfall image instead of the cardback so a
            // 180° flip animation reveals the alternate face. Uses the
            // `_back`-suffixed asset path that the prefetch downloaded
            // with `face=back`.
            let back_mat = if let Some(back_name) = known.back_face_name.as_deref() {
                card_back_face_material(back_name, &mut materials, &asset_server)
            } else {
                card_assets.back_material.clone()
            };
            let entity = spawn_single_card(
                &mut commands,
                &card_assets.card_mesh,
                front_mat,
                back_mat,
                Transform::from_translation(deck_pos),
                GameCardId(card_id),
                &known.name,
                target.translation,
            );
            commands.entity(entity).insert((
                HandCard { slot },
                Animating,
                DrawCardAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: deck_pos,
                    start_rotation: Transform::from_translation(deck_pos).rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                },
            ));
        }
    }

    // ── Viewer hand → Battlefield / Stack / graveyard transitions ───────────
    for (entity, game_id, transform, stack_card, _lift, flipped_marker) in &hand_cards {
        if bf_ids_for(viewer).contains(&game_id.0) {
            let bf_card = cv.battlefield.iter().find(|c| c.id == game_id.0);
            let is_land = bf_card.is_some_and(in_back_row);
            let target = if is_land {
                back_row_card_transform(&cv.battlefield, viewer, viewer, n_seats, game_id.0)
                    .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, true, false))
            } else {
                creature_card_transform(&cv.battlefield, viewer, viewer, n_seats, game_id.0, false)
                    .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, false, false))
            };
            // Flipped MDFC played as its back face: the engine swapped
            // the card's definition to the back face, so the bf card's
            // name is the back-face name. We need the front-child mesh
            // to display the back-face image (via the `_back` asset
            // path) so the played face is up on the battlefield. Animate
            // back to the standard (un-flipped) bf rotation; without
            // this, the play animation would un-rotate the card and
            // expose the original front face (the wrong side).
            // For a flipped MDFC, snap the play animation's start
            // rotation to the bf target so it never interpolates the
            // 180° back-flip on screen. The front-child material gets
            // swapped to the back-face image in the same frame
            // (`apply_swap_front_material`), so the visual stays on the
            // back-face image throughout: the animation overwrites
            // transform.rotation = start_rotation (target) on its first
            // tick, before render. Both children hold the back-face
            // image at that moment, so the snap is invisible.
            let anim_start_rot = if flipped_marker.is_some() {
                target.rotation
            } else {
                transform.rotation
            };
            if flipped_marker.is_some()
                && let Some(bf) = bf_card
            {
                let new_front = card_back_face_material(&bf.name, &mut materials, &asset_server);
                commands.entity(entity).insert(crate::card::SwapFrontMaterial {
                    new_front,
                    new_path: crate::scryfall::card_back_face_asset_path(&bf.name),
                });
                commands.entity(entity).remove::<crate::card::FlippedFace>();
            }
            let is_token = bf_card.is_some_and(|c| c.is_token);
            commands
                .entity(entity)
                .remove::<HandCard>()
                .remove::<StackCard>()
                .remove::<CardHovered>()
                .insert(BattlefieldCard { is_land, is_token })
                .insert(CardOwner(viewer))
                .insert(TapState { tapped: false })
                .insert(Animating)
                .insert(PlayCardAnimation {
                    progress: 0.0,
                    speed: 2.0,
                    start_translation: transform.translation,
                    start_rotation: anim_start_rot,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    start_scale: transform.scale.x,
                    target_scale: 1.0,
                });
        } else if stack_spell_ids.contains(&game_id.0) {
            if stack_card.is_none() {
                let idx = cv.stack.iter().position(|item| matches!(item, StackItemView::Known(k) if k.source == game_id.0)).unwrap_or(0);
                let target = camera_home.stack_lane.card(idx);
                commands
                    .entity(entity)
                    .remove::<CardHovered>()
                    .insert(StackCard)
                    .insert(Animating)
                    .insert(PlayCardAnimation {
                        progress: 0.0,
                        speed: 2.0,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                        start_scale: transform.scale.x,
                        target_scale: camera_home.stack_lane.scale,
                    });
            }
        } else if !hand_ids.contains(&game_id.0) {
            // The hand card is no longer anywhere visible. If a graveyard
            // now holds it the card was discarded or resolved as a spell —
            // fly it to the graveyard. Otherwise it was shuffled back into a
            // library (mulligan put-back) — fly it to the deck pile.
            if in_any_graveyard.contains(&game_id.0) {
                let gy_pos = graveyard_position(viewer, viewer, n_seats, &spread);
                let gy_rot = back_face_rotation(viewer, viewer, n_seats);
                commands
                    .entity(entity)
                    .remove::<HandCard>()
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(SendToGraveyardAnimation {
                        progress: 0.0,
                        speed: 1.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: gy_pos,
                        target_rotation: gy_rot,
                        owner: viewer,
                    });
            } else {
                commands
                    .entity(entity)
                    .remove::<HandCard>()
                    .remove::<CardHovered>()
                    .insert(Animating)
                    .insert(crate::card::ReturnToDeckAnimation {
                        progress: 0.0,
                        speed: 1.5,
                        start_translation: transform.translation,
                        start_rotation: transform.rotation,
                        target_translation: viewer_deck_top,
                        target_rotation: viewer_deck_back_rot,
                    });
            }
        }
    }

    // ── Battlefield → {Hand, Graveyard, despawn} transitions ─────────────────
    // A permanent that has left the battlefield could have:
    //   • been bounced to its owner's hand (Unsummon, Boomerang) — fly back
    //     to the hand fan;
    //   • been a token whose state-based action removed it — despawn outright
    //     (no graveyard pile entry; CR 704.5d);
    //   • died / been destroyed / sacrificed — fly to the graveyard pile.
    let viewer_hand_ids: HashSet<CardId> = viewer_hand
        .iter()
        .map(|c| c.id())
        .collect();
    let opp_hand_card_ids: HashSet<(usize, CardId)> = cv
        .players
        .iter()
        .filter(|p| p.seat != viewer)
        .flat_map(|p| p.hand.iter().map(|c| (p.seat, c.id())))
        .collect();
    for (entity, game_id, owner, bf, transform, _) in &bf_cards {
        if all_bf_ids.contains(&game_id.0) { continue; }
        // Tokens leaving the battlefield cease to exist: no graveyard arc,
        // they shrink away where they lay (after burning, if they died).
        if bf.is_token {
            commands
                .entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardHovered>()
                .insert((Animating, crate::card::Vanishing::default()));
            continue;
        }
        // Bounced to viewer's hand — animate to a hand slot and convert.
        if viewer_hand_ids.contains(&game_id.0) {
            let slot = viewer_hand
                .iter()
                .position(|c| c.id() == game_id.0)
                .unwrap_or(hand_total.saturating_sub(1));
            let target = hand_card_transform(viewer, viewer, n_seats, slot, hand_total, hand_zoom);
            commands
                .entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardOwner>()
                .remove::<CardHovered>()
                .insert(Animating)
                .insert(crate::card::ReturnToHandAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    to_viewer: true,
                    target_slot: slot,
                    target_owner: viewer,
                    target_scale: hand_zoom,
                });
            continue;
        }
        // Bounced to an opponent's hand — fly toward their hand area, then
        // despawn so the next sync frame replaces it with a face-down
        // OpponentHandCard at the correct slot.
        if opp_hand_card_ids.iter().any(|(_, id)| *id == game_id.0) {
            let opp_seat = opp_hand_card_ids
                .iter()
                .find(|(_, id)| *id == game_id.0)
                .map(|(s, _)| *s)
                .unwrap_or(owner.0);
            let opp_hand_size = cv
                .players
                .iter()
                .find(|p| p.seat == opp_seat)
                .map(|p| p.hand.len())
                .unwrap_or(1);
            // Opponent hand zoom stays at 1.0 — only the viewer's own
            // fan is enlarged for readability.
            let target = hand_card_transform(
                opp_seat,
                viewer,
                n_seats,
                opp_hand_size.saturating_sub(1),
                opp_hand_size.max(1),
                1.0,
            );
            commands
                .entity(entity)
                .remove::<BattlefieldCard>()
                .remove::<TapState>()
                .remove::<CardOwner>()
                .remove::<CardHovered>()
                .insert(Animating)
                .insert(crate::card::ReturnToHandAnimation {
                    progress: 0.0,
                    speed: 1.5,
                    start_translation: transform.translation,
                    start_rotation: transform.rotation,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    to_viewer: false,
                    target_slot: 0,
                    target_owner: opp_seat,
                    target_scale: 1.0,
                });
            continue;
        }
        // Default: fly to the owner's graveyard pile.
        let gy_pos = graveyard_position(owner.0, viewer, n_seats, &spread);
        let gy_rot = back_face_rotation(owner.0, viewer, n_seats);
        commands
            .entity(entity)
            .remove::<BattlefieldCard>()
            .remove::<TapState>()
            .remove::<CardHovered>()
            .insert(Animating)
            .insert(SendToGraveyardAnimation {
                progress: 0.0,
                speed: 1.5,
                start_translation: transform.translation,
                start_rotation: transform.rotation,
                target_translation: gy_pos,
                target_rotation: gy_rot,
                owner: owner.0,
            });
    }

    // ── Spawn new opponent battlefield cards (one pass per opponent seat) ───
    for seat in 0..n_seats {
        if seat == viewer {
            continue;
        }
        let to_spawn: Vec<(CardId, String, bool, bool, bool, String)> = cv
            .battlefield
            .iter()
            .filter(|c| {
                c.owner == seat
                    && !visual_bf_ids.contains(&c.id)
                    // Already on-screen as a stack-card entity (transition step above handles it).
                    && !visual_opp_stack_ids.contains(&c.id)
            })
            .map(|c| {
                let face = battlefield_face_path(c, inflight.art.as_deref());
                (c.id, c.name.clone(), in_back_row(c), c.tapped, c.is_token, face)
            })
            .collect();

        for (card_id, card_name, is_land, tapped, is_token, face) in to_spawn {
            // Always animate to the *untapped* battlefield pose first. If the
            // engine state already has the card tapped (typical for a land
            // played-and-auto-tapped to pay a spell cost in the same tick),
            // the tap-state-sync pass below detects the resulting mismatch
            // on the next frame and queues the tap animation as a chained
            // follow-up. This used to spawn directly into the tapped pose,
            // making the tap invisible.
            let target = if is_land {
                back_row_card_transform(&cv.battlefield, seat, viewer, n_seats, card_id)
                    .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, true, false))
            } else {
                creature_card_transform(&cv.battlefield, seat, viewer, n_seats, card_id, false)
                    .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, false, false))
            };
            let _ = tapped; // tap state applied on the next sync pass.

            // Consume one of this opponent's face-down hand visuals as the
            // animation start (so the card appears to fly out of their hand).
            let pool = hand_pool_by_owner.entry(seat).or_default();
            let (start_pos, start_rot) = if let Some((hand_entity, pos, rot)) = pool.pop() {
                commands.entity(hand_entity).despawn();
                promoted.insert(hand_entity);
                (pos, rot)
            } else {
                // Opponent has no hand-card visual to consume — the card
                // came from their library or graveyard (fetchland, tutor,
                // reanimate, token). Animate from the top of their deck
                // pile rather than from where their hand fan would be.
                let opp_deck_base = deck_position(seat, viewer, n_seats);
                let opp_deck_size = cv
                    .players
                    .iter()
                    .find(|p| p.seat == seat)
                    .map(|p| p.library.size)
                    .unwrap_or(0);
                let y = crate::card::pile_height(opp_deck_size) + 0.5;
                let pos = Vec3::new(opp_deck_base.x, y, opp_deck_base.z);
                (pos, back_face_rotation(seat, viewer, n_seats))
            };

            let front_mat = crate::card::spawn::card_face_material(&face, &mut materials, &asset_server);
            let entity = spawn_single_card(
                &mut commands,
                &card_assets.card_mesh,
                front_mat,
                card_assets.back_material.clone(),
                Transform::from_translation(start_pos).with_rotation(start_rot),
                GameCardId(card_id),
                &card_name,
                target.translation,
            );
            commands.entity(entity).insert((
                // The previews show what the face shows.
                crate::card::CardFrontTexture(face),
                BattlefieldCard { is_land, is_token },
                CardOwner(seat),
                // Spawn untapped so the tap-state-sync pass below detects
                // the engine vs visual mismatch and animates the tap.
                TapState { tapped: false },
                Animating,
                PlayCardAnimation {
                    progress: 0.0,
                    speed: 2.0,
                    start_translation: start_pos,
                    start_rotation: start_rot,
                    target_translation: target.translation,
                    target_rotation: target.rotation,
                    // Opponent hand visual → battlefield: never zoomed.
                    start_scale: 1.0,
                    target_scale: 1.0,
                },
            ));
        }
    }

    // ── Stack-card visuals for items that arrived without one ────────────────
    // For each opponent spell on the stack that doesn't yet have a 3-D entity,
    // consume one face-down hand visual and spawn a face-up card that animates
    // to the stack hover position (matching how the viewer's own spells behave).
    //
    // The viewer's own spells come out of their hand (the hand pass above
    // tags the hand entity). One already on the stack when the view arrives —
    // a reconnect, a resume, a spectator joining — or cast from somewhere
    // other than the hand (a commander, a flashback) had no hand card to come
    // from, so it had no 3-D card at all, and no target arrows, which start
    // at it. It drops onto the stack from above.
    use crabomination::net::StackItemView;
    let hand_entity_ids: HashSet<CardId> = all_hand_entity_ids.iter().map(|gid| gid.0).collect();
    for (idx, item) in cv.stack.iter().enumerate() {
        let StackItemView::Known(k) = item else { continue };
        if k.controller == viewer
            && (k.kind != crabomination::net::StackItemKind::Spell || hand_entity_ids.contains(&k.source))
        {
            continue;
        }
        if visual_opp_stack_ids.contains(&k.source) { continue; }
        if visual_bf_ids.contains(&k.source) { continue; }

        let seat = k.controller;
        let target = camera_home.stack_lane.card(idx);

        let pool = hand_pool_by_owner.entry(seat).or_default();
        let (start_pos, start_rot) = if seat == viewer {
            (target.translation + Vec3::Y * 2.0, target.rotation)
        } else if let Some((hand_entity, pos, rot)) = pool.pop() {
            commands.entity(hand_entity).despawn();
            promoted.insert(hand_entity);
            (pos, rot)
        } else {
            let base = deck_position(seat, viewer, n_seats);
            let lib_size = cv.players.iter().find(|p| p.seat == seat)
                .map(|p| p.library.size).unwrap_or(0);
            let y = crate::card::pile_height(lib_size) + 0.5;
            (Vec3::new(base.x, y, base.z), back_face_rotation(seat, viewer, n_seats))
        };

        let front_mat = card_front_material(&k.name, &mut materials, &asset_server);
        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(start_pos).with_rotation(start_rot),
            GameCardId(k.source),
            &k.name,
            target.translation,
        );
        commands.entity(entity).insert((
            StackCard,
            CardOwner(seat),
            Animating,
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: start_pos,
                start_rotation: start_rot,
                target_translation: target.translation,
                target_rotation: target.rotation,
                start_scale: 1.0,
                target_scale: camera_home.stack_lane.scale,
            },
        ));
    }

    // Transition opponent stack entities whose spell resolved to the battlefield.
    for (entity, game_id, transform) in all_stack_entities.iter() {
        let Some(bf_card) = cv.battlefield.iter().find(|c| c.id == game_id.0) else {
            // Not on battlefield — still on stack or resolved to graveyard.
            if !stack_ids.contains(&game_id.0) {
                // Spell has left the stack (resolved to graveyard / exile).
                commands.entity(entity).despawn();
            }
            continue;
        };
        let seat = bf_card.owner;
        let is_land = in_back_row(bf_card);
        let target = if is_land {
            back_row_card_transform(&cv.battlefield, seat, viewer, n_seats, game_id.0)
                .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, true, false))
        } else {
            creature_card_transform(&cv.battlefield, seat, viewer, n_seats, game_id.0, bf_card.tapped)
                .unwrap_or_else(|| bf_card_transform(seat, viewer, n_seats, 0, 1, false, bf_card.tapped))
        };
        commands.entity(entity)
            .remove::<StackCard>()
            .insert(BattlefieldCard { is_land, is_token: bf_card.is_token })
            .insert(CardOwner(seat))
            .insert(TapState { tapped: bf_card.tapped })
            .insert(Animating)
            .insert(PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: transform.translation,
                start_rotation: transform.rotation,
                target_translation: target.translation,
                target_rotation: target.rotation,
                // From the stack lane's size (`framing::StackLane`).
                start_scale: transform.scale.x,
                target_scale: 1.0,
            });
    }

    // ── Opponent hand reconciliation (one pass per opponent seat) ───────────
    let mut removed_opp_entities: HashSet<Entity> = promoted.clone();
    for seat in 0..n_seats {
        if seat == viewer {
            continue;
        }
        let target_hand_size = cv
            .players
            .iter()
            .find(|p| p.seat == seat)
            .map(|p| p.hand.len())
            .unwrap_or(0);

        let opp_hand_for_seat: Vec<&(Entity, usize, usize, Vec3, Quat, bool)> = all_opp_hand
            .iter()
            .filter(|(_, owner, _, _, _, _)| *owner == seat)
            .collect();
        // In-flight bounces toward this opponent count as already-spawned
        // hand visuals — they despawn on completion and the next sync
        // frame sees a normal OpponentHandCard for the bounced card. Without
        // this, the reconciler races the bounce by spawning a duplicate
        // face-down placeholder.
        let inflight_to_seat = inflight
            .to_hand
            .iter()
            .filter(|(_, anim)| !anim.to_viewer && anim.target_owner == seat)
            .count();
        let visual_count = opp_hand_for_seat
            .iter()
            .filter(|(e, _, _, _, _, _)| !promoted.contains(e))
            .count()
            + inflight_to_seat;

        if visual_count > target_hand_size {
            let mut sorted: Vec<_> = opp_hand_for_seat
                .iter()
                .filter(|(e, _, _, _, _, _)| !promoted.contains(e))
                .collect();
            sorted.sort_by_key(|(_, _, slot, _, _, _)| std::cmp::Reverse(*slot));
            for entry in sorted.iter().take(visual_count - target_hand_size) {
                let entity = entry.0;
                commands.entity(entity).despawn();
                removed_opp_entities.insert(entity);
            }
        } else if visual_count < target_hand_size {
            let base = deck_position(seat, viewer, n_seats);
            let deck_y = target_hand_size as f32 * DECK_CARD_Y_STEP + 0.5;
            let deck_pos = Vec3::new(base.x, deck_y, base.z);
            for slot in visual_count..target_hand_size {
                // Opponent hand — zoom stays at 1.0.
                let target = hand_card_transform(seat, viewer, n_seats, slot, target_hand_size, 1.0);
                let back_mat = card_assets.back_material.clone();
                let card_mesh = card_assets.card_mesh.clone();
                let start_transform = Transform::from_translation(deck_pos);
                commands
                    .spawn((
                        start_transform,
                        Visibility::default(),
                        OpponentHandCard { owner: seat, slot },
                        CardHoverLift {
                            current_lift: 0.0,
                            target_lift: 0.0,
                            base_translation: target.translation,
                        },
                        Animating,
                        PlayCardAnimation {
                            progress: 0.0,
                            speed: 2.0,
                            start_translation: deck_pos,
                            start_rotation: start_transform.rotation,
                            target_translation: target.translation,
                            target_rotation: target.rotation,
                            // Opponent hand visual — unscaled.
                            start_scale: 1.0,
                            target_scale: 1.0,
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Mesh3d(card_mesh.clone()),
                            MeshMaterial3d(back_mat.clone()),
                            Transform::from_xyz(0.0, 0.0, CARD_THICKNESS / 2.0),
                        ));
                        parent.spawn((
                            Mesh3d(card_mesh.clone()),
                            MeshMaterial3d(back_mat.clone()),
                            Transform::from_xyz(0.0, 0.0, -CARD_THICKNESS / 2.0)
                                .with_rotation(Quat::from_rotation_y(PI)),
                        ));
                    });
            }
        }

        // Despawn any cards whose slot is now out of range (animating card
        // with high slot may have escaped the count pass).
        for entry in &opp_hand_for_seat {
            let (entity, _, slot, _, _, _) = **entry;
            if !removed_opp_entities.contains(&entity) && slot >= target_hand_size {
                commands.entity(entity).despawn();
                removed_opp_entities.insert(entity);
            }
        }

        // Slide remaining non-animating cards to their new positions.
        for entry in &opp_hand_for_seat {
            let (entity, _, slot, pos, _rot, is_animating) = **entry;
            if removed_opp_entities.contains(&entity) || is_animating {
                continue;
            }
            // Opponent hand — zoom stays at 1.0.
            let target = hand_card_transform(seat, viewer, n_seats, slot, target_hand_size, 1.0);
            if pos.distance(target.translation) > 0.001 {
                commands
                    .entity(entity)
                    .insert(Animating)
                    .insert(HandSlideAnimation {
                        progress: 0.0,
                        speed: 4.0,
                        start_translation: pos,
                        target_translation: target.translation,
                        target_rotation: target.rotation,
                    })
                    .insert(CardHoverLift {
                        current_lift: 0.0,
                        target_lift: 0.0,
                        base_translation: target.translation,
                    });
            }
        }
    }

    // ── Spawn new viewer battlefield cards that don't have entities yet ──────
    let viewer_to_spawn: Vec<(CardId, String, bool, bool, bool, String)> = cv
        .battlefield
        .iter()
        .filter(|c| {
            c.owner == viewer
                && !visual_bf_ids.contains(&c.id)
                && !hand_cards.iter().any(|(_, gid, _, _, _, _)| gid.0 == c.id)
        })
        .map(|c| {
            let face = battlefield_face_path(c, inflight.art.as_deref());
            (c.id, c.name.clone(), in_back_row(c), c.tapped, c.is_token, face)
        })
        .collect();

    // Battlefield cards that didn't come from the viewer's hand (fetchlands,
    // tutors that drop directly onto the battlefield, reanimate, tokens) get
    // a `library → battlefield` arc animation starting from the top of the
    // viewer's deck pile, instead of teleporting in.
    for (card_id, card_name, is_land, tapped, is_token, face) in viewer_to_spawn {
        // Same untapped-spawn pattern as the opponent path above: land at
        // the untapped pose, let tap-state-sync animate the tap on the
        // next frame if the engine state has the card already tapped.
        let target = if is_land {
            back_row_card_transform(&cv.battlefield, viewer, viewer, n_seats, card_id)
                .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, true, false))
        } else {
            creature_card_transform(&cv.battlefield, viewer, viewer, n_seats, card_id, false)
                .unwrap_or_else(|| bf_card_transform(viewer, viewer, n_seats, 0, 1, false, false))
        };
        let _ = tapped;
        let front_mat = crate::card::spawn::card_face_material(&face, &mut materials, &asset_server);
        let entity = spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            card_assets.back_material.clone(),
            Transform::from_translation(viewer_deck_top).with_rotation(viewer_deck_back_rot),
            GameCardId(card_id),
            &card_name,
            target.translation,
        );
        commands.entity(entity).insert((
            crate::card::CardFrontTexture(face),
            BattlefieldCard { is_land, is_token },
            CardOwner(viewer),
            TapState { tapped: false },
            Animating,
            PlayCardAnimation {
                progress: 0.0,
                speed: 2.0,
                start_translation: viewer_deck_top,
                start_rotation: viewer_deck_back_rot,
                target_translation: target.translation,
                target_rotation: target.rotation,
                // From viewer's deck (unscaled) — straight to battlefield (unscaled).
                start_scale: 1.0,
                target_scale: 1.0,
            },
        ));
    }

    // ── Rebalance viewer hand slots ──────────────────────────────────────────
    for (entity, game_id, _transform, stack_card, lift, flipped_marker) in &hand_cards {
        if stack_card.is_some() { continue; }
        if !hand_ids.contains(&game_id.0) { continue; }
        let Some(new_slot) = viewer_hand.iter().position(|c| c.id() == game_id.0) else { continue };
        let target = hand_card_transform(viewer, viewer, n_seats, new_slot, hand_total, hand_zoom);
        let dist = (lift.base_translation - target.translation).length();
        if dist > 0.1 {
            // A flipped MDFC keeps its 180° face-flip through the slide —
            // the fan target rotation alone would silently show the front
            // again while the click handler still casts the back.
            let target_rotation = if flipped_marker.is_some() {
                target.rotation * Quat::from_rotation_y(std::f32::consts::PI)
            } else {
                target.rotation
            };
            commands
                .entity(entity)
                .insert(Animating)
                .insert(HandSlideAnimation {
                    progress: 0.0,
                    speed: 3.0,
                    start_translation: lift.base_translation,
                    target_translation: target.translation,
                    target_rotation,
                });
        }
        commands.entity(entity).insert(HandCard { slot: new_slot });
    }

    // ── Rebalance battlefield positions + sync tapped state ──────────────────
    for (entity, game_id, owner, bf, _transform, (tap_state, hover)) in &bf_cards {
        if !all_bf_ids.contains(&game_id.0) { continue; }
        let is_land = bf.is_land;

        let game_tapped = cv.battlefield.iter().find(|c| c.id == game_id.0).is_some_and(|c| c.tapped);
        let visual_tapped = tap_state.is_some_and(|ts| ts.tapped);

        let target = if is_land {
            let Some(t) = back_row_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0) else { continue };
            let base_rot = t.rotation;
            let tapped_rot = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * base_rot;
            let rot = if game_tapped { tapped_rot } else { base_rot };
            Transform { translation: t.translation, rotation: rot, scale: t.scale }
        } else {
            // Grouped placement — the same token-pile cascade the spawn
            // path uses, so a rebalance never scatters a pile.
            let Some(t) = creature_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0, game_tapped) else { continue };
            t
        };

        if game_tapped != visual_tapped {
            // Row rotations depend only on the seat's edge and tap state,
            // not the slot, so a 1-card probe transform supplies both.
            let (untapped_rot, tapped_rot) = if is_land {
                let base = back_row_card_transform(&cv.battlefield, owner.0, viewer, n_seats, game_id.0)
                    .map(|t| t.rotation).unwrap_or(target.rotation);
                (base, Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2) * base)
            } else {
                (
                    bf_card_transform(owner.0, viewer, n_seats, 0, 1, false, false).rotation,
                    bf_card_transform(owner.0, viewer, n_seats, 0, 1, false, true).rotation,
                )
            };
            let (start, end) = if game_tapped { (untapped_rot, tapped_rot) } else { (tapped_rot, untapped_rot) };
            commands
                .entity(entity)
                .insert(Animating)
                .insert(TapAnimation { progress: 0.0, speed: 4.0, start_rotation: start, target_rotation: end })
                .insert(TapState { tapped: game_tapped });
        }

        // A new view moves the card's resting place, not its hover: zeroing
        // the lift here dropped a hovered card back onto the table whenever
        // a view landed under the pointer (an opponent acting), though it
        // stayed hovered until the pointer left it.
        let (current_lift, target_lift) = hover.map_or((0.0, 0.0), |h| (h.current_lift, h.target_lift));
        commands.entity(entity).insert(CardHoverLift {
            current_lift,
            target_lift,
            base_translation: target.translation,
        });
    }
}

// ── MDFC flip sync ────────────────────────────────────────────────────────────

/// Reconcile each viewer hand card's persistent flip state
/// (`FlippedFace` marker on the entity) against the user's intent
/// (`FlippedHandCards.flipped`). When they disagree, attach a 180°
/// `MdfcFlipAnimation` and toggle the marker. Both card faces are
/// already painted with their proper Scryfall images at spawn time, so
/// the rotation alone reveals the alternate face — no material swap.
/// Also drops stale flip entries when cards leave the hand.
#[allow(clippy::type_complexity)]
pub fn sync_flipped_hand_cards(
    mut commands: Commands,
    cv: Res<CurrentView>,
    mut flipped: ResMut<crate::game::FlippedHandCards>,
    hand_cards: Query<
        (
            Entity,
            &GameCardId,
            &Transform,
            Option<&crate::card::FlippedFace>,
        ),
        (With<HandCard>, Without<crate::card::MdfcFlipAnimation>),
    >,
) {
    let Some(view) = cv.0.as_ref() else { return };
    let viewer = view.your_seat;
    if viewer >= view.players.len() { return; }

    // Drop flips for cards that are no longer in the viewer's hand.
    let in_hand: HashSet<CardId> = view.players[viewer]
        .hand
        .iter()
        .map(|h| h.id())
        .collect();
    flipped.flipped.retain(|id| in_hand.contains(id));

    for (entity, game_id, transform, marker) in &hand_cards {
        let card_id = game_id.0;
        let should_be_flipped = flipped.flipped.contains(&card_id);
        let is_flipped = marker.is_some();
        if should_be_flipped == is_flipped {
            continue;
        }
        commands.entity(entity).insert(crate::card::MdfcFlipAnimation {
            progress: 0.0,
            speed: 2.5,
            start_rotation: transform.rotation,
        });
        if should_be_flipped {
            commands.entity(entity).insert(crate::card::FlippedFace);
        } else {
            commands.entity(entity).remove::<crate::card::FlippedFace>();
        }
    }
}

// ── Command zone sync ─────────────────────────────────────────────────────────

/// Spawn/despawn visual entities for command-zone cards (Commander
/// commanders, Conspiracies). Each card is rendered as a face-up
/// fixed-position card near the seat's edge of the table. Click
/// handling routes through `CastFromCommandZone` (see
/// `try_cast_from_command_zone`).
///
/// Simple model: each command-zone card gets one visual entity.
/// On view sync, despawn entities for cards that are no longer
/// present, spawn new ones for arrivals, and move the rest to their
/// slot of the zone as it now stands (one partner cast, the other
/// takes the first slot). No animations — cards just appear,
/// disappear or step over with the view update.
#[allow(clippy::type_complexity)]
pub fn sync_command_zone(
    mut commands: Commands,
    cv: Res<CurrentView>,
    card_assets: Option<Res<crate::card::CardMeshAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut existing: Query<
        (Entity, &GameCardId, &mut Transform, &mut crate::card::CardHoverLift),
        With<crate::card::CommandZoneCard>,
    >,
) {
    let Some(view) = cv.0.as_ref() else { return };
    let Some(card_assets) = card_assets else { return };
    let viewer = view.your_seat;
    let n_seats = view.players.len();
    let spread = crate::card::Spread::of(&view.battlefield, n_seats);

    // Collect every (CardId, owner, slot, zone size) currently in any
    // command zone, so the spawn loop can reuse the layout helper.
    let mut want: HashMap<CardId, (usize, usize, usize)> = HashMap::new();
    // `None` = render the card back: CR 315.7 hides another player's
    // face-down hidden-agenda conspiracy, but the object is still there.
    let mut want_name: HashMap<CardId, Option<String>> = HashMap::new();
    for player in &view.players {
        for (slot, entry) in player.command.iter().enumerate() {
            match entry {
                crabomination::net::HandCardView::Known(k) => {
                    want.insert(k.id, (player.seat, slot, player.command.len()));
                    want_name.insert(k.id, Some(k.name.clone()));
                }
                crabomination::net::HandCardView::Hidden { id } => {
                    want.insert(*id, (player.seat, slot, player.command.len()));
                    want_name.insert(*id, None);
                }
            }
        }
    }

    // Despawn visuals for cards no longer in any command zone; move the
    // rest to their slot.
    let mut have: HashSet<CardId> = HashSet::new();
    for (entity, game_id, mut transform, mut lift) in &mut existing {
        let Some(&(owner, slot, count)) = want.get(&game_id.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        have.insert(game_id.0);
        let target = crate::card::command_zone_card_transform(owner, viewer, n_seats, slot, count, &spread);
        rest_pile_at(&mut transform, &mut lift, target.translation);
    }

    // Spawn fresh visuals for newly-arrived command-zone cards.
    for (card_id, (owner, slot, count)) in &want {
        if have.contains(card_id) {
            continue;
        }
        let hidden = want_name.get(card_id).map(|n| n.is_none()).unwrap_or(false);
        let name = want_name.get(card_id).cloned().flatten().unwrap_or_default();
        let target = crate::card::command_zone_card_transform(*owner, viewer, n_seats, *slot, *count, &spread);
        let back_mat = card_assets.back_material.clone();
        let front_mat = if hidden {
            back_mat.clone()
        } else {
            card_front_material(&name, &mut materials, &asset_server)
        };
        let entity = crate::card::spawn_single_card(
            &mut commands,
            &card_assets.card_mesh,
            front_mat,
            back_mat,
            target,
            GameCardId(*card_id),
            &name,
            target.translation,
        );
        commands.entity(entity).insert(crate::card::CommandZoneCard { owner: *owner });
    }
}
