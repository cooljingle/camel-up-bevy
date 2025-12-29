use bevy::prelude::*;
use crate::components::*;
use crate::systems::animation::{MovementAnimation, MultiStepMovementAnimation};

/// Message fired when a camel needs to move
#[derive(Message)]
pub struct MoveCamelEvent {
    pub color: CamelColor,
    pub spaces: u8,
}

/// Message fired when a crazy camel needs to move
#[derive(Message)]
pub struct MoveCrazyCamelEvent {
    pub color: CrazyCamelColor,
    pub spaces: u8,
}

/// Message fired when movement is complete
#[derive(Message)]
pub struct MovementCompleteEvent {
    pub crossed_finish: bool,
}

/// Animation duration per hop in seconds
const CAMEL_HOP_DURATION: f32 = 0.15;
/// Animation duration for simple vertical shifts (existing camels being displaced)
const CAMEL_SHIFT_DURATION: f32 = 0.3;

/// Generate waypoints for multi-step movement animation
fn generate_waypoints(
    board: &GameBoard,
    start_space: u8,
    end_space: u8,
    stack_offset: f32,
    z_index: f32,
    backwards: bool,
) -> Vec<Vec3> {
    let mut waypoints = Vec::new();

    if backwards {
        // Moving backwards (for crazy camels)
        let mut current = start_space;
        while current >= end_space {
            let base_pos = board.get_position(current);
            waypoints.push(Vec3::new(base_pos.x, base_pos.y + stack_offset, z_index));
            if current == 0 || current == end_space {
                break;
            }
            current -= 1;
        }
    } else {
        // Moving forwards
        for space in start_space..=end_space.min(TRACK_LENGTH - 1) {
            let base_pos = board.get_position(space);
            waypoints.push(Vec3::new(base_pos.x, base_pos.y + stack_offset, z_index));
        }
    }

    waypoints
}

/// System to handle regular camel movement
pub fn move_camel_system(
    mut commands: Commands,
    mut events: MessageReader<MoveCamelEvent>,
    mut camels: Query<(Entity, &Camel, &mut BoardPosition, &mut Transform)>,
    mut crazy_camels: Query<(Entity, &CrazyCamel, &mut BoardPosition, &mut Transform), Without<Camel>>,
    board: Res<GameBoard>,
    mut movement_complete: MessageWriter<MovementCompleteEvent>,
    placed_tiles: Option<Res<PlacedDesertTiles>>,
    mut players: Option<ResMut<Players>>,
) {
    for event in events.read() {
        // Find the camel that needs to move
        let mut moving_camel_entity = None;
        let mut start_space = 0u8;
        let mut start_stack_pos = 0u8;

        for (entity, camel, pos, _) in camels.iter() {
            if camel.color == event.color {
                moving_camel_entity = Some(entity);
                start_space = pos.space_index;
                start_stack_pos = pos.stack_position;
                break;
            }
        }

        let Some(moving_entity) = moving_camel_entity else { continue };

        // Calculate initial target space
        let mut target_space = start_space + event.spaces;
        let mut crossed_finish = target_space >= TRACK_LENGTH;
        let mut land_underneath = false;  // For mirage tiles

        // Check for desert tile at target space
        if !crossed_finish {
            if let Some(ref tiles) = placed_tiles {
                if let Some((owner_id, is_oasis)) = tiles.get_tile(target_space) {
                    // Pay the owner 1 coin
                    if let Some(ref mut players) = players {
                        if let Some(owner) = players.players.iter_mut().find(|p| p.id == owner_id) {
                            owner.money += 1;
                            info!("{} earned $1 from spectator tile!", owner.name);
                        }
                    }

                    if is_oasis {
                        // Oasis: move 1 more space forward, land on top
                        target_space += 1;
                        crossed_finish = target_space >= TRACK_LENGTH;
                        info!("Oasis! Camel moves 1 extra space forward");
                    } else {
                        // Mirage: move 1 space backward, land underneath
                        target_space = target_space.saturating_sub(1);
                        land_underneath = true;
                        info!("Mirage! Camel moves 1 space backward and lands underneath");
                    }
                }
            }
        }

        // Collect all camels that need to move (the moving camel and all camels on top of it)
        let mut camels_to_move: Vec<Entity> = Vec::new();

        // Add the moving camel
        camels_to_move.push(moving_entity);

        // Find all camels stacked on top of the moving camel (same space, higher stack position)
        for (entity, _, pos, _) in camels.iter() {
            if pos.space_index == start_space && pos.stack_position > start_stack_pos {
                camels_to_move.push(entity);
            }
        }

        // Also check crazy camels on top
        for (entity, _, pos, _) in crazy_camels.iter() {
            if pos.space_index == start_space && pos.stack_position > start_stack_pos {
                camels_to_move.push(entity);
            }
        }

        // Sort by stack position so we maintain relative order
        let mut camel_stack_positions: Vec<(Entity, u8)> = camels_to_move
            .iter()
            .filter_map(|&e| {
                if let Ok((_, _, pos, _)) = camels.get(e) {
                    Some((e, pos.stack_position))
                } else if let Ok((_, _, pos, _)) = crazy_camels.get(e) {
                    Some((e, pos.stack_position))
                } else {
                    None
                }
            })
            .collect();
        camel_stack_positions.sort_by_key(|(_, pos)| *pos);

        let final_space = target_space.min(TRACK_LENGTH - 1);

        if land_underneath {
            // Landing underneath: shift existing camels up, place moving camels at bottom
            let num_moving = camel_stack_positions.len() as u8;

            // First, collect entities at destination that need to be shifted (not the ones moving)
            let entities_to_shift: Vec<Entity> = camels
                .iter()
                .filter(|(entity, _, pos, _)| {
                    pos.space_index == final_space && !camels_to_move.contains(entity)
                })
                .map(|(entity, _, _, _)| entity)
                .collect();

            let crazy_entities_to_shift: Vec<Entity> = crazy_camels
                .iter()
                .filter(|(entity, _, pos, _)| {
                    pos.space_index == final_space && !camels_to_move.contains(entity)
                })
                .map(|(entity, _, _, _)| entity)
                .collect();

            // Shift existing camels up with animation (simple vertical shift)
            for entity in entities_to_shift {
                if let Ok((_, _, mut pos, transform)) = camels.get_mut(entity) {
                    pos.stack_position += num_moving;
                    let stack_offset = pos.stack_position as f32 * 25.0;
                    let base_pos = board.get_position(final_space);
                    let start = transform.translation;
                    let end = Vec3::new(base_pos.x, base_pos.y + stack_offset, 10.0 + pos.stack_position as f32);
                    commands.entity(entity).insert(MovementAnimation::new(start, end, CAMEL_SHIFT_DURATION));
                }
            }

            for entity in crazy_entities_to_shift {
                if let Ok((_, _, mut pos, transform)) = crazy_camels.get_mut(entity) {
                    pos.stack_position += num_moving;
                    let stack_offset = pos.stack_position as f32 * 25.0;
                    let base_pos = board.get_position(final_space);
                    let start = transform.translation;
                    let end = Vec3::new(base_pos.x, base_pos.y + stack_offset, 10.0 + pos.stack_position as f32);
                    commands.entity(entity).insert(MovementAnimation::new(start, end, CAMEL_SHIFT_DURATION));
                }
            }

            // Place moving camels at bottom with multi-step animation
            // Calculate the space before desert tile effect for proper waypoint generation
            let pre_desert_space = start_space + event.spaces;

            for (i, (entity, _)) in camel_stack_positions.iter().enumerate() {
                let new_stack_pos = i as u8;
                let stack_offset = new_stack_pos as f32 * 25.0;
                let z_index = 10.0 + new_stack_pos as f32;

                // Generate waypoints from start to pre-desert-tile space, then add final position
                let mut waypoints = generate_waypoints(
                    &board,
                    start_space,
                    pre_desert_space.min(TRACK_LENGTH - 1),
                    stack_offset,
                    z_index,
                    false,
                );

                // Add final position after mirage effect (going back one space, underneath)
                let final_base_pos = board.get_position(final_space);
                let final_pos = Vec3::new(final_base_pos.x, final_base_pos.y + stack_offset, z_index);
                if waypoints.last() != Some(&final_pos) {
                    waypoints.push(final_pos);
                }

                if let Ok((_, _, mut pos, _)) = camels.get_mut(*entity) {
                    pos.space_index = final_space;
                    pos.stack_position = new_stack_pos;
                    commands.entity(*entity).insert(MultiStepMovementAnimation::new(waypoints.clone(), CAMEL_HOP_DURATION));
                } else if let Ok((_, _, mut pos, _)) = crazy_camels.get_mut(*entity) {
                    pos.space_index = final_space;
                    pos.stack_position = new_stack_pos;
                    commands.entity(*entity).insert(MultiStepMovementAnimation::new(waypoints.clone(), CAMEL_HOP_DURATION));
                }
            }
        } else {
            // Normal landing on top
            // Count how many camels are already at the target space (excluding moving ones)
            let mut target_stack_height = 0u8;
            for (entity, _, pos, _) in camels.iter() {
                if pos.space_index == final_space && !camels_to_move.contains(&entity) {
                    target_stack_height = target_stack_height.max(pos.stack_position + 1);
                }
            }
            for (entity, _, pos, _) in crazy_camels.iter() {
                if pos.space_index == final_space && !camels_to_move.contains(&entity) {
                    target_stack_height = target_stack_height.max(pos.stack_position + 1);
                }
            }

            // Move all the camels with multi-step animation
            // Calculate the space before desert tile effect for proper waypoint generation
            let pre_desert_space = start_space + event.spaces;

            for (i, (entity, _old_stack_pos)) in camel_stack_positions.iter().enumerate() {
                let new_stack_pos = target_stack_height + i as u8;
                let stack_offset = new_stack_pos as f32 * 25.0;
                let z_index = 10.0 + new_stack_pos as f32;

                // Generate waypoints from start to pre-desert-tile space
                let mut waypoints = generate_waypoints(
                    &board,
                    start_space,
                    pre_desert_space.min(TRACK_LENGTH - 1),
                    stack_offset,
                    z_index,
                    false,
                );

                // If oasis triggered, add extra space at the end
                if final_space > pre_desert_space {
                    let final_base_pos = board.get_position(final_space);
                    let final_pos = Vec3::new(final_base_pos.x, final_base_pos.y + stack_offset, z_index);
                    waypoints.push(final_pos);
                }

                if let Ok((_, _, mut pos, _)) = camels.get_mut(*entity) {
                    pos.space_index = final_space;
                    pos.stack_position = new_stack_pos;
                    commands.entity(*entity).insert(MultiStepMovementAnimation::new(waypoints.clone(), CAMEL_HOP_DURATION));
                } else if let Ok((_, _, mut pos, _)) = crazy_camels.get_mut(*entity) {
                    pos.space_index = final_space;
                    pos.stack_position = new_stack_pos;
                    commands.entity(*entity).insert(MultiStepMovementAnimation::new(waypoints.clone(), CAMEL_HOP_DURATION));
                }
            }
        }

        movement_complete.write(MovementCompleteEvent { crossed_finish });
    }
}

/// System to handle crazy camel movement (backwards!)
pub fn move_crazy_camel_system(
    mut commands: Commands,
    mut events: MessageReader<MoveCrazyCamelEvent>,
    mut camels: Query<(Entity, &Camel, &mut BoardPosition, &mut Transform)>,
    mut crazy_camels: Query<(Entity, &CrazyCamel, &mut BoardPosition, &mut Transform), Without<Camel>>,
    board: Res<GameBoard>,
) {
    for event in events.read() {
        // Find the crazy camel that needs to move
        let mut moving_entity = None;
        let mut start_space = 0u8;
        let mut start_stack_pos = 0u8;

        for (entity, camel, pos, _) in crazy_camels.iter() {
            if camel.color == event.color {
                moving_entity = Some(entity);
                start_space = pos.space_index;
                start_stack_pos = pos.stack_position;
                break;
            }
        }

        let Some(moving_ent) = moving_entity else { continue };

        // Crazy camels move backwards
        let target_space = start_space.saturating_sub(event.spaces);

        // Collect all camels/crazy camels on top
        let mut entities_to_move: Vec<(Entity, u8, bool)> = Vec::new(); // (entity, stack_pos, is_crazy)

        entities_to_move.push((moving_ent, start_stack_pos, true));

        for (entity, _, pos, _) in camels.iter() {
            if pos.space_index == start_space && pos.stack_position > start_stack_pos {
                entities_to_move.push((entity, pos.stack_position, false));
            }
        }

        for (entity, _, pos, _) in crazy_camels.iter() {
            if entity != moving_ent && pos.space_index == start_space && pos.stack_position > start_stack_pos {
                entities_to_move.push((entity, pos.stack_position, true));
            }
        }

        entities_to_move.sort_by_key(|(_, pos, _)| *pos);

        // Crazy camels land ON TOP of existing camels when they move
        // Count how many camels are already at the target space (excluding moving ones)
        let mut target_stack_height = 0u8;
        for (entity, _, pos, _) in camels.iter() {
            if pos.space_index == target_space && !entities_to_move.iter().any(|(e, _, _)| *e == entity) {
                target_stack_height = target_stack_height.max(pos.stack_position + 1);
            }
        }
        for (entity, _, pos, _) in crazy_camels.iter() {
            if pos.space_index == target_space && !entities_to_move.iter().any(|(e, _, _)| *e == entity) {
                target_stack_height = target_stack_height.max(pos.stack_position + 1);
            }
        }

        // Move all the camels with multi-step backwards animation, landing on top
        for (i, (entity, _, is_crazy)) in entities_to_move.iter().enumerate() {
            let new_stack_pos = target_stack_height + i as u8;
            let stack_offset = new_stack_pos as f32 * 25.0;
            let z_index = 10.0 + new_stack_pos as f32;

            // Generate waypoints for backwards movement
            let waypoints = generate_waypoints(
                &board,
                start_space,
                target_space,
                stack_offset,
                z_index,
                true, // backwards
            );

            if *is_crazy {
                if let Ok((_, _, mut pos, _)) = crazy_camels.get_mut(*entity) {
                    pos.space_index = target_space;
                    pos.stack_position = new_stack_pos;
                    commands.entity(*entity).insert(MultiStepMovementAnimation::new(waypoints.clone(), CAMEL_HOP_DURATION));
                }
            } else if let Ok((_, _, mut pos, _)) = camels.get_mut(*entity) {
                pos.space_index = target_space;
                pos.stack_position = new_stack_pos;
                commands.entity(*entity).insert(MultiStepMovementAnimation::new(waypoints.clone(), CAMEL_HOP_DURATION));
            }
        }
    }
}

/// Get the leading camel (first place)
pub fn get_leading_camel(
    camels: &Query<(&Camel, &BoardPosition)>,
) -> Option<CamelColor> {
    let mut best: Option<(CamelColor, u8, u8)> = None; // (color, space, stack_pos)

    for (camel, pos) in camels.iter() {
        match best {
            None => best = Some((camel.color, pos.space_index, pos.stack_position)),
            Some((_, best_space, best_stack)) => {
                if pos.space_index > best_space ||
                   (pos.space_index == best_space && pos.stack_position > best_stack) {
                    best = Some((camel.color, pos.space_index, pos.stack_position));
                }
            }
        }
    }

    best.map(|(color, _, _)| color)
}

/// Get the second place camel
pub fn get_second_place_camel(
    camels: &Query<(&Camel, &BoardPosition)>,
) -> Option<CamelColor> {
    let mut rankings: Vec<(CamelColor, u8, u8)> = camels
        .iter()
        .map(|(c, p)| (c.color, p.space_index, p.stack_position))
        .collect();

    // Sort by space descending, then stack position descending
    rankings.sort_by(|a, b| {
        b.1.cmp(&a.1).then(b.2.cmp(&a.2))
    });

    rankings.get(1).map(|(color, _, _)| *color)
}

/// Get the last place camel (for end-game betting)
pub fn get_last_place_camel(
    camels: &Query<(&Camel, &BoardPosition)>,
) -> Option<CamelColor> {
    let mut worst: Option<(CamelColor, u8, u8)> = None;

    for (camel, pos) in camels.iter() {
        match worst {
            None => worst = Some((camel.color, pos.space_index, pos.stack_position)),
            Some((_, worst_space, worst_stack)) => {
                if pos.space_index < worst_space ||
                   (pos.space_index == worst_space && pos.stack_position < worst_stack) {
                    worst = Some((camel.color, pos.space_index, pos.stack_position));
                }
            }
        }
    }

    worst.map(|(color, _, _)| color)
}
