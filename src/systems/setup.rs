use crate::components::*;
use crate::systems::turn::{PlayerLegBetsStore, PlayerPyramidTokens, TurnState};
use crate::ui::player_setup::PlayerSetupConfig;
use bevy::color::Srgba;
use bevy::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;

// ============================================================================
// Color Manipulation Helpers
// ============================================================================

/// Darken a color by a given amount (0.0 = no change, 1.0 = black)
fn darken_color(color: Color, amount: f32) -> Color {
    let rgba: Srgba = color.into();
    Color::srgba(
        (rgba.red * (1.0 - amount)).max(0.0),
        (rgba.green * (1.0 - amount)).max(0.0),
        (rgba.blue * (1.0 - amount)).max(0.0),
        rgba.alpha,
    )
}

/// Lighten a color by a given amount (0.0 = no change, 1.0 = white)
fn lighten_color(color: Color, amount: f32) -> Color {
    let rgba: Srgba = color.into();
    Color::srgba(
        (rgba.red + (1.0 - rgba.red) * amount).min(1.0),
        (rgba.green + (1.0 - rgba.green) * amount).min(1.0),
        (rgba.blue + (1.0 - rgba.blue) * amount).min(1.0),
        rgba.alpha,
    )
}

// ============================================================================
// Board Space Spawning
// ============================================================================

/// Spawn a polished board space with shadow, border, and highlight layers
fn spawn_board_space(commands: &mut Commands, pos: Vec2, index: u8) {
    let space_size = Vec2::new(70.0, 50.0);

    // Shadow layer (offset down-right, darker)
    commands.spawn((
        GameEntity,
        Sprite {
            color: Color::srgba(0.3, 0.25, 0.15, 0.5),
            custom_size: Some(space_size),
            ..default()
        },
        Transform::from_xyz(pos.x + 3.0, pos.y - 3.0, 0.0),
    ));

    // Border layer (slightly larger, dark brown)
    commands.spawn((
        GameEntity,
        Sprite {
            color: Color::srgb(0.4, 0.3, 0.2),
            custom_size: Some(space_size + Vec2::new(4.0, 4.0)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.1),
    ));

    // Main space (sand color)
    commands.spawn((
        GameEntity,
        BoardSpace { index },
        Sprite {
            color: Color::srgb(0.85, 0.75, 0.55),
            custom_size: Some(space_size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, 0.2),
    ));

    // Inner highlight (top portion, subtle)
    commands.spawn((
        GameEntity,
        Sprite {
            color: Color::srgba(1.0, 0.95, 0.85, 0.3),
            custom_size: Some(Vec2::new(space_size.x - 8.0, space_size.y * 0.4)),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y + 8.0, 0.3),
    ));

    // Space number label (below the space)
    commands.spawn((
        GameEntity,
        Text2d::new(format!("{}", index + 1)),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Transform::from_xyz(pos.x, pos.y - 35.0, 1.0),
    ));

    // Spectator tile sprite (initially invisible, updated by update_spectator_tile_sprites system)
    let tile_size = Vec2::new(35.0, 18.0);
    commands
        .spawn((
            GameEntity,
            crate::components::board::SpectatorTileSprite { space_index: index },
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0), // Start invisible
                custom_size: Some(tile_size),
                ..default()
            },
            Transform::from_xyz(pos.x, pos.y + 35.0, 4.5), // Above board, below placed tiles (z=5)
        ))
        .with_children(|parent| {
            // Symbol text (initially invisible)
            parent.spawn((
                Text2d::new("+".to_string()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });

    // Add finish line marker at space 16 (index 15)
    if index == 15 {
        spawn_finish_line(commands, pos);
    }
}

/// Spawn a vertical checkered finish line on the left edge of the final space
/// This emphasizes that camels must cross this threshold to win
fn spawn_finish_line(commands: &mut Commands, pos: Vec2) {
    let checker_size = 8.0;
    let rows = 8; // Tall vertical flag
    let cols = 3; // Narrow width

    // Position on the LEFT edge of space 15 (the finish threshold)
    // Camels crossing this line have finished the race
    let start_x = pos.x - 40.0 - (cols as f32 * checker_size) / 2.0 + checker_size / 2.0;
    let start_y = pos.y - (rows as f32 * checker_size) / 2.0 + checker_size / 2.0;

    for row in 0..rows {
        for col in 0..cols {
            let is_white = (row + col) % 2 == 0;
            let color = if is_white { Color::WHITE } else { Color::BLACK };
            commands.spawn((
                GameEntity,
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(checker_size)),
                    ..default()
                },
                Transform::from_xyz(
                    start_x + col as f32 * checker_size,
                    start_y + row as f32 * checker_size,
                    0.5,
                ),
            ));
        }
    }

    // Add a flagpole on the left side
    let pole_height = rows as f32 * checker_size + 20.0;
    let pole_x = start_x - checker_size / 2.0 - 2.0;
    let pole_y = pos.y;

    commands.spawn((
        GameEntity,
        Sprite {
            color: Color::srgb(0.4, 0.3, 0.2), // Brown pole
            custom_size: Some(Vec2::new(4.0, pole_height)),
            ..default()
        },
        Transform::from_xyz(pole_x, pole_y, 0.4),
    ));

    // Add a small ball on top of the pole
    commands.spawn((
        GameEntity,
        Sprite {
            color: Color::srgb(0.8, 0.7, 0.2), // Gold ball
            custom_size: Some(Vec2::splat(8.0)),
            ..default()
        },
        Transform::from_xyz(pole_x, pole_y + pole_height / 2.0 + 4.0, 0.4),
    ));
}

// ============================================================================
// Camel Spawning
// ============================================================================

/// Spawn the camel silhouette shape components as children
/// Creates a stylized camel profile using multiple overlapping shapes:
/// - Body (large horizontal oval)
/// - Hump (oval on top of body)
/// - Neck (tall narrow rectangle)
/// - Head (small oval)
/// - Legs (4 thin rectangles)
fn spawn_camel_shape(
    parent: &mut ChildSpawnerCommands,
    base_color: Color,
    border_color: Color,
    highlight_color: Color,
) {
    // Camel dimensions (overall bounding box roughly 50x35)
    let body_size = Vec2::new(32.0, 18.0);
    let hump_size = Vec2::new(14.0, 12.0);
    let neck_size = Vec2::new(8.0, 16.0);
    let head_size = Vec2::new(14.0, 10.0);
    let leg_size = Vec2::new(5.0, 14.0);

    // Positions relative to camel center
    let body_pos = Vec2::new(0.0, 0.0);
    let hump_pos = Vec2::new(-2.0, 10.0);
    let neck_pos = Vec2::new(16.0, 8.0);
    let head_pos = Vec2::new(22.0, 16.0);
    let leg_positions = [
        Vec2::new(-10.0, -14.0), // Back left
        Vec2::new(-4.0, -14.0),  // Back right
        Vec2::new(8.0, -14.0),   // Front left
        Vec2::new(14.0, -14.0),  // Front right
    ];

    // Shadow layer - all parts offset
    let shadow_offset = Vec3::new(2.0, -2.0, -0.3);
    let shadow_color = Color::srgba(0.0, 0.0, 0.0, 0.3);

    // Shadow: body
    parent.spawn((
        Sprite {
            color: shadow_color,
            custom_size: Some(body_size),
            ..default()
        },
        Transform::from_translation(body_pos.extend(0.0) + shadow_offset),
    ));
    // Shadow: hump
    parent.spawn((
        Sprite {
            color: shadow_color,
            custom_size: Some(hump_size),
            ..default()
        },
        Transform::from_translation(hump_pos.extend(0.0) + shadow_offset),
    ));
    // Shadow: neck
    parent.spawn((
        Sprite {
            color: shadow_color,
            custom_size: Some(neck_size),
            ..default()
        },
        Transform::from_translation(neck_pos.extend(0.0) + shadow_offset),
    ));
    // Shadow: head
    parent.spawn((
        Sprite {
            color: shadow_color,
            custom_size: Some(head_size),
            ..default()
        },
        Transform::from_translation(head_pos.extend(0.0) + shadow_offset),
    ));
    // Shadow: legs
    for leg_pos in &leg_positions {
        parent.spawn((
            Sprite {
                color: shadow_color,
                custom_size: Some(leg_size),
                ..default()
            },
            Transform::from_translation(leg_pos.extend(0.0) + shadow_offset),
        ));
    }

    // Border layer - all parts slightly larger
    let border_expand = 3.0;

    // Border: body
    parent.spawn((
        Sprite {
            color: border_color,
            custom_size: Some(body_size + Vec2::splat(border_expand)),
            ..default()
        },
        Transform::from_xyz(body_pos.x, body_pos.y, -0.2),
    ));
    // Border: hump
    parent.spawn((
        Sprite {
            color: border_color,
            custom_size: Some(hump_size + Vec2::splat(border_expand)),
            ..default()
        },
        Transform::from_xyz(hump_pos.x, hump_pos.y, -0.2),
    ));
    // Border: neck
    parent.spawn((
        Sprite {
            color: border_color,
            custom_size: Some(neck_size + Vec2::splat(border_expand)),
            ..default()
        },
        Transform::from_xyz(neck_pos.x, neck_pos.y, -0.2),
    ));
    // Border: head
    parent.spawn((
        Sprite {
            color: border_color,
            custom_size: Some(head_size + Vec2::splat(border_expand)),
            ..default()
        },
        Transform::from_xyz(head_pos.x, head_pos.y, -0.2),
    ));
    // Border: legs
    for leg_pos in &leg_positions {
        parent.spawn((
            Sprite {
                color: border_color,
                custom_size: Some(leg_size + Vec2::splat(border_expand - 1.0)),
                ..default()
            },
            Transform::from_xyz(leg_pos.x, leg_pos.y, -0.2),
        ));
    }

    // Main color layer
    // Body
    parent.spawn((
        Sprite {
            color: base_color,
            custom_size: Some(body_size),
            ..default()
        },
        Transform::from_xyz(body_pos.x, body_pos.y, -0.1),
    ));
    // Hump
    parent.spawn((
        Sprite {
            color: base_color,
            custom_size: Some(hump_size),
            ..default()
        },
        Transform::from_xyz(hump_pos.x, hump_pos.y, -0.1),
    ));
    // Neck
    parent.spawn((
        Sprite {
            color: base_color,
            custom_size: Some(neck_size),
            ..default()
        },
        Transform::from_xyz(neck_pos.x, neck_pos.y, -0.1),
    ));
    // Head
    parent.spawn((
        Sprite {
            color: base_color,
            custom_size: Some(head_size),
            ..default()
        },
        Transform::from_xyz(head_pos.x, head_pos.y, -0.1),
    ));
    // Legs
    for leg_pos in &leg_positions {
        parent.spawn((
            Sprite {
                color: base_color,
                custom_size: Some(leg_size),
                ..default()
            },
            Transform::from_xyz(leg_pos.x, leg_pos.y, -0.1),
        ));
    }

    // Highlight on hump and head
    parent.spawn((
        Sprite {
            color: highlight_color.with_alpha(0.4),
            custom_size: Some(Vec2::new(hump_size.x - 4.0, 4.0)),
            ..default()
        },
        Transform::from_xyz(hump_pos.x, hump_pos.y + 3.0, 0.0),
    ));
    parent.spawn((
        Sprite {
            color: highlight_color.with_alpha(0.4),
            custom_size: Some(Vec2::new(head_size.x - 4.0, 3.0)),
            ..default()
        },
        Transform::from_xyz(head_pos.x, head_pos.y + 2.0, 0.0),
    ));

    // Eye (small dark circle on head)
    parent.spawn((
        Sprite {
            color: Color::srgb(0.1, 0.1, 0.1),
            custom_size: Some(Vec2::new(3.0, 3.0)),
            ..default()
        },
        Transform::from_xyz(head_pos.x + 3.0, head_pos.y + 1.0, 0.1),
    ));
}

/// Spawn a polished racing camel with camel-shaped silhouette
fn spawn_racing_camel(
    commands: &mut Commands,
    color: CamelColor,
    space_index: u8,
    stack_pos: u8,
    spawn_pos: Vec3,
    pending_move: bool,
) {
    let base_color = color.to_bevy_color();
    let border_color = darken_color(base_color, 0.4);
    let highlight_color = lighten_color(base_color, 0.3);

    // Parent entity with game logic components
    let mut entity_commands = commands.spawn((
        GameEntity,
        Camel { color },
        BoardPosition {
            space_index,
            stack_position: stack_pos,
        },
        CamelSprite,
        Transform::from_translation(spawn_pos),
        Visibility::default(),
    ));

    if pending_move {
        entity_commands.insert(PendingInitialMove);
    }

    entity_commands.with_children(|parent| {
        spawn_camel_shape(parent, base_color, border_color, highlight_color);
    });
}

/// Spawn a polished crazy camel with camel-shaped silhouette (facing left on top row)
fn spawn_crazy_camel(
    commands: &mut Commands,
    color: CrazyCamelColor,
    space_index: u8,
    stack_pos: u8,
    spawn_pos: Vec3,
    pending_move: bool,
) {
    let base_color = color.to_bevy_color();
    let border_color = darken_color(base_color, 0.4);
    let highlight_color = lighten_color(base_color, 0.3);

    // Parent entity with game logic components
    // Crazy camels face right (same as racing camels) but move backwards on the track
    let mut entity_commands = commands.spawn((
        GameEntity,
        CrazyCamel { color },
        BoardPosition {
            space_index,
            stack_position: stack_pos,
        },
        CamelSprite,
        Transform::from_translation(spawn_pos),
        Visibility::default(),
    ));

    if pending_move {
        entity_commands.insert(PendingInitialMove);
    }

    entity_commands.with_children(|parent| {
        spawn_camel_shape(parent, base_color, border_color, highlight_color);
    });
}

// ============================================================================
// Dice Tent Spawning
// ============================================================================

/// Tent dimensions in world space
const TENT_WIDTH: f32 = 50.0;
const TENT_ROOF_HEIGHT: f32 = 35.0;
const TENT_BASE_HEIGHT: f32 = 40.0;
const TENT_SPACING: f32 = 60.0;
const TENT_Y_POSITION: f32 = 200.0; // Above the track
const TENT_BASE_Z: f32 = 2.0; // Above board spaces (0-1), below spectator tiles (4.5) and camels (10+)

/// Spawn a polished dice tent with multi-layer visuals (shadow, border, main, highlight)
fn spawn_dice_tent(commands: &mut Commands, position: Vec3, tent_index: usize) {
    // Colors for empty tent (sandy brown)
    let base_color = Color::srgb(0.65, 0.55, 0.40);
    let border_color = Color::srgb(0.4, 0.3, 0.2);
    let shadow_color = Color::srgba(0.0, 0.0, 0.0, 0.3);
    let highlight_color = Color::srgba(1.0, 0.95, 0.85, 0.3);

    // Parent tent entity
    commands
        .spawn((
            GameEntity,
            DiceTent { index: tent_index },
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .with_children(|parent| {
            spawn_tent_layers(
                parent,
                base_color,
                border_color,
                shadow_color,
                highlight_color,
            );
        });
}

/// Spawn all visual layers for a tent
fn spawn_tent_layers(
    parent: &mut ChildSpawnerCommands,
    base_color: Color,
    border_color: Color,
    shadow_color: Color,
    highlight_color: Color,
) {
    let shadow_offset = Vec3::new(3.0, -3.0, -0.3);
    let border_expand = 3.0;

    // === BASE/SUPPORT AREA (rectangle where dice sits) ===
    let base_size = Vec2::new(TENT_WIDTH, TENT_BASE_HEIGHT);
    let base_y = -TENT_BASE_HEIGHT / 2.0;

    // Shadow: base
    parent.spawn((
        TentSprite,
        Sprite {
            color: shadow_color,
            custom_size: Some(base_size),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, base_y, 0.0) + shadow_offset),
    ));

    // Border: base
    parent.spawn((
        TentSprite,
        Sprite {
            color: border_color,
            custom_size: Some(base_size + Vec2::splat(border_expand)),
            ..default()
        },
        Transform::from_xyz(0.0, base_y, 0.1),
    ));

    // Main: base
    parent.spawn((
        TentSprite,
        Sprite {
            color: base_color,
            custom_size: Some(base_size),
            ..default()
        },
        Transform::from_xyz(0.0, base_y, 0.2),
    ));

    // === ROOF (triangular top made of two halves) ===
    // We'll approximate the triangle with thin rectangles stacked
    // This creates a stepped pyramid effect that looks good at game scale
    let roof_base_y = 0.0; // Bottom of roof
    let roof_width = TENT_WIDTH + 6.0; // Slightly wider than base
    let num_steps = 8;

    for i in 0..num_steps {
        let progress = i as f32 / num_steps as f32;
        let step_y = roof_base_y + (progress * TENT_ROOF_HEIGHT);
        let step_width = roof_width * (1.0 - progress);
        let step_height = TENT_ROOF_HEIGHT / num_steps as f32 + 1.0; // +1 for overlap

        if step_width < 4.0 {
            continue; // Skip very thin top pieces
        }

        // Shadow for this step
        parent.spawn((
            TentSprite,
            Sprite {
                color: shadow_color,
                custom_size: Some(Vec2::new(step_width, step_height)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, step_y, 0.0) + shadow_offset),
        ));

        // Border for this step
        parent.spawn((
            TentSprite,
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(step_width + 2.0, step_height + 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, step_y, 0.1),
        ));

        // Main color for this step
        parent.spawn((
            TentSprite,
            Sprite {
                color: base_color,
                custom_size: Some(Vec2::new(step_width, step_height)),
                ..default()
            },
            Transform::from_xyz(0.0, step_y, 0.2),
        ));
    }

    // Highlight on top portion of roof
    parent.spawn((
        TentSprite,
        Sprite {
            color: highlight_color,
            custom_size: Some(Vec2::new(12.0, 6.0)),
            ..default()
        },
        Transform::from_xyz(0.0, TENT_ROOF_HEIGHT - 8.0, 0.3),
    ));

    // Side poles/supports
    let pole_height = TENT_BASE_HEIGHT;
    let pole_width = 4.0;
    let pole_x_offset = TENT_WIDTH / 2.0 - 2.0;

    for x_sign in [-1.0, 1.0] {
        let pole_x = x_sign * pole_x_offset;

        // Shadow
        parent.spawn((
            TentSprite,
            Sprite {
                color: shadow_color,
                custom_size: Some(Vec2::new(pole_width, pole_height)),
                ..default()
            },
            Transform::from_translation(Vec3::new(pole_x, base_y, 0.25) + shadow_offset),
        ));

        // Main pole (darker than base)
        parent.spawn((
            TentSprite,
            Sprite {
                color: border_color,
                custom_size: Some(Vec2::new(pole_width, pole_height)),
                ..default()
            },
            Transform::from_xyz(pole_x, base_y, 0.3),
        ));
    }
}

// ============================================================================
// Pyramid Roll Button
// ============================================================================

const PYRAMID_Y_POSITION: f32 = -220.0; // Below the track
pub const PYRAMID_SIZE: f32 = 150.0;
const PYRAMID_BASE_Z: f32 = 15.0;

/// Marker for pyramid roll button child sprites
#[derive(Component)]
pub struct PyramidSprite;

/// Spawn the pyramid roll button as a game board sprite
fn spawn_pyramid_button(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    // Pyramid gold colors (matching the egui version)
    let pyramid_light = Color::srgb(0.83, 0.66, 0.29); // #D4A84B
    let pyramid_dark = Color::srgb(0.63, 0.48, 0.19); // #A07A30
    let outline_color = Color::srgb(0.42, 0.29, 0.10); // #6B4A1A
    let shadow_color = Color::srgba(0.0, 0.0, 0.0, 0.3);

    let position = Vec3::new(0.0, PYRAMID_Y_POSITION, PYRAMID_BASE_Z);

    // Create coin mesh and material handles for the gold coin
    let coin_radius = 12.0;
    let coin_gold = Color::srgb(0.83, 0.66, 0.29); // #D4A84B
    let coin_dark = Color::srgb(0.63, 0.48, 0.19); // #A07A30

    let coin_outer_mesh = meshes.add(Circle::new(coin_radius + 2.0));
    let coin_inner_mesh = meshes.add(Circle::new(coin_radius));
    let coin_outer_material = materials.add(ColorMaterial::from_color(coin_dark));
    let coin_inner_material = materials.add(ColorMaterial::from_color(coin_gold));

    // Parent pyramid entity with clickable marker
    commands
        .spawn((
            GameEntity,
            PyramidRollButton,
            Transform::from_translation(position),
            Visibility::default(),
        ))
        .with_children(|parent| {
            spawn_pyramid_layers(
                parent,
                PYRAMID_SIZE,
                pyramid_light,
                pyramid_dark,
                outline_color,
                shadow_color,
                coin_outer_mesh.clone(),
                coin_inner_mesh.clone(),
                coin_outer_material.clone(),
                coin_inner_material.clone(),
            );
        });
}

/// Spawn the visual layers for the pyramid button
fn spawn_pyramid_layers(
    parent: &mut ChildSpawnerCommands,
    size: f32,
    light_color: Color,
    dark_color: Color,
    outline_color: Color,
    shadow_color: Color,
    coin_outer_mesh: Handle<Mesh>,
    coin_inner_mesh: Handle<Mesh>,
    coin_outer_material: Handle<ColorMaterial>,
    coin_inner_material: Handle<ColorMaterial>,
) {
    // The pyramid is built from triangular-ish shapes using rectangles
    // We'll approximate with a stepped pyramid similar to the tents

    let base_width = size;
    let height = size * 0.85;
    let num_steps = 10;

    // Hover border (initially hidden) - drawn behind the pyramid
    // Gold color matching egui::Color32::GOLD
    let border_color = Color::srgb(1.0, 0.84, 0.0);
    let border_thickness = 4.0;
    let border_width = base_width + border_thickness * 2.0;
    let border_height = height + border_thickness * 2.0;

    parent.spawn((
        board::PyramidHoverBorder,
        Sprite {
            color: border_color,
            custom_size: Some(Vec2::new(border_width, border_height)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -0.2), // Behind pyramid and shadow
        Visibility::Hidden,                  // Start hidden, shown on hover
    ));

    // Shadow offset
    let shadow_offset = Vec3::new(3.0, -3.0, -0.1);

    // Build pyramid from bottom to top with stepped layers
    for i in 0..num_steps {
        let progress = i as f32 / num_steps as f32;
        let step_y = -height / 2.0 + (progress * height);
        let step_width = base_width * (1.0 - progress * 0.85); // Narrower at top
        let step_height = height / num_steps as f32 + 1.0; // +1 for overlap

        if step_width < 6.0 {
            continue;
        }

        // Shadow for this step
        parent.spawn((
            PyramidSprite,
            Sprite {
                color: shadow_color,
                custom_size: Some(Vec2::new(step_width, step_height)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, step_y, 0.0) + shadow_offset),
        ));

        // Left half (darker)
        parent.spawn((
            PyramidSprite,
            Sprite {
                color: dark_color,
                custom_size: Some(Vec2::new(step_width / 2.0, step_height)),
                ..default()
            },
            Transform::from_xyz(-step_width / 4.0, step_y, 0.1),
        ));

        // Right half (lighter)
        parent.spawn((
            PyramidSprite,
            Sprite {
                color: light_color,
                custom_size: Some(Vec2::new(step_width / 2.0, step_height)),
                ..default()
            },
            Transform::from_xyz(step_width / 4.0, step_y, 0.1),
        ));

        // Outline edge at the middle seam (subtle vertical line)
        if i < num_steps - 2 {
            parent.spawn((
                PyramidSprite,
                Sprite {
                    color: outline_color,
                    custom_size: Some(Vec2::new(2.0, step_height)),
                    ..default()
                },
                Transform::from_xyz(0.0, step_y, 0.2),
            ));
        }
    }

    // Add "Roll" text above center
    parent.spawn((
        PyramidSprite,
        Text2d::new("Roll"),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(outline_color),
        Transform::from_xyz(0.0, -4.0, 1.0),
    ));

    // Add gold coin below "Roll" text
    spawn_gold_coin(
        parent,
        Vec3::new(0.0, -26.0, 1.0),
        coin_outer_mesh,
        coin_inner_mesh,
        coin_outer_material,
        coin_inner_material,
    );
}

/// Spawn a gold coin with circular mesh and "1" inside
fn spawn_gold_coin(
    parent: &mut ChildSpawnerCommands,
    position: Vec3,
    coin_outer_mesh: Handle<Mesh>,
    coin_inner_mesh: Handle<Mesh>,
    coin_outer_material: Handle<ColorMaterial>,
    coin_inner_material: Handle<ColorMaterial>,
) {
    let text_color = Color::srgb(0.42, 0.29, 0.10); // #6B4A1A

    // Coin outer border (darker circle)
    parent.spawn((
        PyramidSprite,
        Mesh2d(coin_outer_mesh),
        MeshMaterial2d(coin_outer_material),
        Transform::from_translation(position + Vec3::new(0.0, 0.0, 0.0)),
    ));

    // Coin main (gold circle)
    parent.spawn((
        PyramidSprite,
        Mesh2d(coin_inner_mesh),
        MeshMaterial2d(coin_inner_material),
        Transform::from_translation(position + Vec3::new(0.0, 0.0, 0.1)),
    ));

    // "1" text in center
    parent.spawn((
        PyramidSprite,
        Text2d::new("1"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(text_color),
        Transform::from_translation(position + Vec3::new(0.0, 0.0, 0.2)),
    ));
}

/// Marker component for all game entities that should be cleaned up when leaving Playing state
#[derive(Component)]
pub struct GameEntity;

/// Identifies which type of camel is being rolled for initial setup
#[derive(Clone, Copy)]
pub enum InitialRollCamel {
    Racing(CamelColor),
    Crazy(CrazyCamelColor),
}

impl InitialRollCamel {
    pub fn to_bevy_color(&self) -> Color {
        match self {
            InitialRollCamel::Racing(c) => c.to_bevy_color(),
            InitialRollCamel::Crazy(c) => c.to_bevy_color(),
        }
    }
}

/// Resource to manage the initial setup roll animations
#[derive(Resource, Default)]
pub struct InitialSetupRolls {
    pub camel_rolls: Vec<(InitialRollCamel, u8, u8, u8, Vec3)>, // (camel type, roll value, space_index, stack_pos, target position)
    pub current_roll_index: usize,                      // Which roll is currently animating
    pub all_complete: bool,                             // All rolls have been shown
    pub current_dice_spawned: bool,                     // Whether dice for current roll was spawned
    pub current_camel_moving: bool, // Whether camel for current roll has started moving
    pub started: bool,              // Whether the player has clicked the "Set up camels" button
    pub placed_camels: Vec<(u8, u8)>, // (space_index, stack_pos) for camels that finished moving
}

/// Marker component for camels waiting to move onto the board during initial setup
#[derive(Component)]
pub struct PendingInitialMove;

/// Staging position for camels before they move onto the board
const CAMEL_STAGING_X: f32 = -450.0;
const CAMEL_STAGING_Y: f32 = -100.0;

/// Duration per hop for initial camel movement
const CAMEL_HOP_DURATION: f32 = 0.15;

/// Generate waypoints for initial racing camel movement (forward from space 0)
fn generate_initial_waypoints_racing(
    board: &GameBoard,
    staging_pos: Vec3,
    target_space: u8,
    target_stack_pos: u8,
    placed_camels: &[(u8, u8)],  // (space_index, stack_pos) for already-placed camels
) -> Vec<Vec3> {
    let mut waypoints = vec![staging_pos];

    // Hop through spaces 0 to target_space
    for space in 0..=target_space {
        let base_pos = board.get_position(space);

        // Calculate stack height at this space
        let stack_height = if space == target_space {
            target_stack_pos  // Final position uses calculated stack pos
        } else {
            // Intermediate spaces: hop to top of existing camels
            placed_camels.iter().filter(|(s, _)| *s == space).count() as u8
        };

        let y = base_pos.y + stack_height as f32 * 25.0;
        waypoints.push(Vec3::new(base_pos.x, y, 10.0 + stack_height as f32));
    }

    waypoints
}

/// Generate waypoints for initial crazy camel movement (backward from finish line)
fn generate_initial_waypoints_crazy(
    board: &GameBoard,
    staging_pos: Vec3,
    target_space: u8,
    target_stack_pos: u8,
    placed_camels: &[(u8, u8)],  // (space_index, stack_pos) for already-placed camels
) -> Vec<Vec3> {
    let mut waypoints = vec![staging_pos];

    // Crazy camels start near finish (space 15) and hop backwards to their target
    // They hop from space 15 down to target_space
    let start_space = 15u8;  // Finish line area
    let mut current = start_space;

    loop {
        let base_pos = board.get_position(current);

        // Calculate stack height at this space
        let stack_height = if current == target_space {
            target_stack_pos  // Final position uses calculated stack pos
        } else {
            // Intermediate spaces: hop to top of existing camels
            placed_camels.iter().filter(|(s, _)| *s == current).count() as u8
        };

        let y = base_pos.y + stack_height as f32 * 25.0;
        waypoints.push(Vec3::new(base_pos.x, y, 10.0 + stack_height as f32));

        if current == target_space {
            break;
        }
        current -= 1;
    }

    waypoints
}

pub fn setup_game(
    mut commands: Commands,
    config: Res<PlayerSetupConfig>,
    existing_camels: Query<Entity, With<Camel>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Don't setup if game entities already exist (returning from leg scoring)
    if !existing_camels.is_empty() {
        info!("Game already setup, skipping...");
        return;
    }

    // Create players from setup config
    let players = Players::new(config.to_player_configs());
    let player_count = players.players.len();

    // Insert game resources
    commands.insert_resource(GameBoard::new());
    commands.insert_resource(players);
    commands.insert_resource(Pyramid::new());
    commands.insert_resource(LegBettingTiles::new());
    commands.insert_resource(RaceBets::default());
    commands.insert_resource(PlacedSpectatorTiles::default());

    // Insert turn-related resources
    commands.insert_resource(TurnState::default());
    commands.insert_resource(PlayerLegBetsStore::new(player_count));
    commands.insert_resource(PlayerPyramidTokens::new(player_count));

    // Get board for positioning
    let board = GameBoard::new();

    // Spawn the track spaces with polished layered visuals
    for i in 0..TRACK_LENGTH {
        let pos = board.get_position(i);
        spawn_board_space(&mut commands, pos, i);
    }

    // Roll initial positions for racing camels (spaces 1-3, i.e., indices 0-2)
    let mut rng = rand::thread_rng();
    let mut camel_positions: Vec<(u8, u8)> = Vec::new(); // (space_index, stack_pos)
    let mut initial_rolls = InitialSetupRolls::default();

    let mut racing_order: Vec<CamelColor> = CamelColor::all().into();
    racing_order.shuffle(&mut rng);

    for (i, color) in racing_order.into_iter().enumerate() {
        // Roll 1-3 for starting position (space index 0-2)
        let roll_value = rng.gen_range(1..=3) as u8;
        let space_index = roll_value - 1; // Convert 1-3 roll to 0-2 index

        // Count how many camels are already on this space
        let stack_pos = camel_positions
            .iter()
            .filter(|(s, _)| *s == space_index)
            .count() as u8;

        camel_positions.push((space_index, stack_pos));

        // Calculate target position on the board
        let base_pos = board.get_position(space_index);
        let stack_offset = stack_pos as f32 * 25.0; // Stack camels vertically
        let target_pos = Vec3::new(
            base_pos.x,
            base_pos.y + stack_offset,
            10.0 + stack_pos as f32,
        );

        // Record the roll and target position for animation
        initial_rolls
            .camel_rolls
            .push((InitialRollCamel::Racing(color), roll_value, space_index, stack_pos, target_pos));

        // Spawn camels at staging position (staggered vertically so they're visible)
        let staging_pos = Vec3::new(
            CAMEL_STAGING_X,
            CAMEL_STAGING_Y + (i as f32 * 35.0), // Stack them vertically at staging
            10.0 + i as f32,
        );
        spawn_racing_camel(
            &mut commands,
            color,
            space_index,
            stack_pos,
            staging_pos,
            true,
        );
    }

    // Roll initial positions for BOTH crazy camels (spaces 14-16, i.e., indices 13-15)
    // Each crazy camel gets its own roll of 1-3 mapped to spaces 14-16
    let racing_camel_count = CamelColor::all().len();
    let mut crazy_positions: Vec<(u8, u8)> = Vec::new(); // (space_index, stack_pos)

    let mut crazy_order: Vec<CrazyCamelColor> = CrazyCamelColor::all().into();
    crazy_order.shuffle(&mut rng);

    for (i, crazy_color) in crazy_order.into_iter().enumerate() {
        // Roll 1-3 for starting position (mapped to space indices 13-15)
        // Roll 1 → space 16 (index 15), Roll 2 → space 15 (index 14), Roll 3 → space 14 (index 13)
        let roll_value = rng.gen_range(1..=3) as u8;
        let space_index = 16 - roll_value; // Convert 1-3 roll inversely to 15-13 index

        // Count how many crazy camels are already on this space
        let stack_pos = crazy_positions
            .iter()
            .filter(|(s, _)| *s == space_index)
            .count() as u8;

        crazy_positions.push((space_index, stack_pos));

        // Calculate target position on the board
        let base_pos = board.get_position(space_index);
        let stack_offset = stack_pos as f32 * 25.0;
        let target_pos = Vec3::new(
            base_pos.x,
            base_pos.y + stack_offset,
            10.0 + stack_pos as f32,
        );

        // Record the roll and target position for animation
        initial_rolls.camel_rolls.push((
            InitialRollCamel::Crazy(crazy_color),
            roll_value,
            space_index,
            stack_pos,
            target_pos,
        ));

        // Spawn crazy camels at staging position (on the left side, same as racing camels)
        let staging_pos = Vec3::new(
            CAMEL_STAGING_X, // Left side (negative X) - same as racing camels
            CAMEL_STAGING_Y + ((racing_camel_count + i) as f32 * 35.0),
            10.0 + i as f32,
        );
        spawn_crazy_camel(
            &mut commands,
            crazy_color,
            space_index,
            stack_pos,
            staging_pos,
            true,
        );
    }

    // Insert the initial rolls resource for display
    commands.insert_resource(initial_rolls);

    // Spawn dice tents below the track
    let num_tents = 5;
    let total_tent_width = (num_tents as f32 - 1.0) * TENT_SPACING;
    let tent_start_x = -total_tent_width / 2.0;

    for i in 0..num_tents {
        let tent_x = tent_start_x + (i as f32 * TENT_SPACING);
        let tent_pos = Vec3::new(tent_x, TENT_Y_POSITION, TENT_BASE_Z);
        spawn_dice_tent(&mut commands, tent_pos, i);
    }

    // Spawn pyramid roll button below the track
    spawn_pyramid_button(&mut commands, &mut meshes, &mut materials);

    info!("Game setup complete!");
}

/// Clean up all game entities when leaving the Playing state
pub fn cleanup_game(
    mut commands: Commands,
    game_entities: Query<Entity, With<GameEntity>>,
    mut ui_state: ResMut<crate::ui::hud::UiState>,
    mut celebration_state: ResMut<crate::ui::scoring::CelebrationState>,
    mut camera_state: ResMut<crate::CameraState>,
    mut camel_position_anims: ResMut<crate::ui::hud::CamelPositionAnimations>,
    mut rules_state: ResMut<crate::ui::rules::RulesState>,
    mut ai_think_timer: ResMut<crate::game::ai::AiThinkTimer>,
) {
    for entity in game_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Reset all UI and game state
    *ui_state = crate::ui::hud::UiState::default();
    *celebration_state = crate::ui::scoring::CelebrationState::default();
    *camera_state = crate::CameraState::default();
    *camel_position_anims = crate::ui::hud::CamelPositionAnimations::default();
    *rules_state = crate::ui::rules::RulesState::default();
    *ai_think_timer = crate::game::ai::AiThinkTimer::default();

    // Remove all game resources that are inserted during setup_game
    // These must be removed so they can be re-inserted fresh on next game start
    commands.remove_resource::<crate::ui::scoring::GameEndState>();
    commands.remove_resource::<GameBoard>();
    commands.remove_resource::<Players>();
    commands.remove_resource::<Pyramid>();
    commands.remove_resource::<LegBettingTiles>();
    commands.remove_resource::<RaceBets>();
    commands.remove_resource::<PlacedSpectatorTiles>();
    commands.remove_resource::<TurnState>();
    commands.remove_resource::<PlayerLegBetsStore>();
    commands.remove_resource::<PlayerPyramidTokens>();
    commands.remove_resource::<InitialSetupRolls>();

    info!("Game cleanup complete!");
}

/// System to animate initial camel placement rolls
pub fn initial_roll_animation_system(
    mut commands: Commands,
    mut initial_rolls: Option<ResMut<InitialSetupRolls>>,
    mut ui_state: ResMut<crate::ui::hud::UiState>,
    board: Res<GameBoard>,
    dice_query: Query<
        &crate::systems::animation::DiceRollAnimation,
        With<crate::systems::animation::DiceSprite>,
    >,
    mut racing_camel_query: Query<(Entity, &Camel, &Transform), With<PendingInitialMove>>,
    mut crazy_camel_query: Query<
        (Entity, &CrazyCamel, &Transform),
        (With<PendingInitialMove>, Without<Camel>),
    >,
    moving_camels: Query<
        Entity,
        (
            With<CamelSprite>,
            Or<(
                With<crate::systems::animation::MovementAnimation>,
                With<crate::systems::animation::MultiStepMovementAnimation>,
            )>,
        ),
    >,
) {
    let Some(ref mut rolls) = initial_rolls else {
        // If no InitialSetupRolls resource, consider rolls complete
        ui_state.initial_rolls_complete = true;
        return;
    };

    // Skip if all rolls are complete
    if rolls.all_complete {
        ui_state.initial_rolls_complete = true;
        return;
    }

    // Wait for player to click the "Set up camels" button
    if !rolls.started {
        return;
    }

    // Skip if no rolls to show
    if rolls.camel_rolls.is_empty() {
        rolls.all_complete = true;
        ui_state.initial_rolls_complete = true;
        return;
    }

    // Check dice animation state
    let dice_in_display_or_later = dice_query.iter().next().map_or(false, |anim| {
        matches!(
            anim.phase,
            crate::systems::animation::DiceRollPhase::Settling
                | crate::systems::animation::DiceRollPhase::Display
                | crate::systems::animation::DiceRollPhase::MovingToTent
                | crate::systems::animation::DiceRollPhase::InTent
        )
    });
    let dice_finished = dice_query.is_empty();
    let camel_still_moving = !moving_camels.is_empty();

    // Start camel moving when dice enters settling phase
    // Also trigger if dice has already finished (despawned) but camel hasn't moved yet
    // This handles cases where the fast dice animation completes between frames
    if rolls.current_dice_spawned && !rolls.current_camel_moving && (dice_in_display_or_later || dice_finished) {
        let (camel_type, _value, space_index, stack_pos, _target_pos) = rolls.camel_rolls[rolls.current_roll_index];

        // Find the camel and start its movement animation
        match camel_type {
            InitialRollCamel::Racing(color) => {
                for (entity, camel, transform) in racing_camel_query.iter_mut() {
                    if camel.color == color {
                        let start_pos = transform.translation;
                        let waypoints = generate_initial_waypoints_racing(
                            &*board,
                            start_pos,
                            space_index,
                            stack_pos,
                            &rolls.placed_camels,
                        );
                        commands
                            .entity(entity)
                            .insert(crate::systems::animation::MultiStepMovementAnimation::new(
                                waypoints,
                                CAMEL_HOP_DURATION,
                            ))
                            .remove::<PendingInitialMove>();
                        rolls.current_camel_moving = true;
                        info!("Started moving {:?} racing camel to board", color);
                        break;
                    }
                }
            }
            InitialRollCamel::Crazy(color) => {
                for (entity, camel, transform) in crazy_camel_query.iter_mut() {
                    if camel.color == color {
                        let start_pos = transform.translation;
                        let waypoints = generate_initial_waypoints_crazy(
                            &*board,
                            start_pos,
                            space_index,
                            stack_pos,
                            &rolls.placed_camels,
                        );
                        commands
                            .entity(entity)
                            .insert(crate::systems::animation::MultiStepMovementAnimation::new(
                                waypoints,
                                CAMEL_HOP_DURATION,
                            ))
                            .remove::<PendingInitialMove>();
                        rolls.current_camel_moving = true;
                        info!("Started moving {:?} crazy camel to board", color);
                        break;
                    }
                }
            }
        }
        return;
    }

    // Wait for both dice and camel animations to finish before next roll
    if rolls.current_dice_spawned && (camel_still_moving || !dice_finished) {
        return;
    }

    // If current roll is done, immediately advance to next (no delay for initial setup)
    if rolls.current_dice_spawned
        && rolls.current_camel_moving
        && !camel_still_moving
        && dice_finished
    {
        // Record this camel as placed for stack height calculations
        let (_camel_type, _value, space_index, stack_pos, _target_pos) = rolls.camel_rolls[rolls.current_roll_index];
        rolls.placed_camels.push((space_index, stack_pos));

        rolls.current_roll_index += 1;
        rolls.current_dice_spawned = false;
        rolls.current_camel_moving = false;

        // Check if all rolls are done
        if rolls.current_roll_index >= rolls.camel_rolls.len() {
            rolls.all_complete = true;
            ui_state.initial_rolls_complete = true;
            info!("All initial rolls complete!");
            return;
        }
    }

    // If we haven't spawned the current dice yet, spawn it
    if !rolls.current_dice_spawned && rolls.current_roll_index < rolls.camel_rolls.len() {
        let (camel_type, value, _space_index, _stack_pos, _target_pos) = rolls.camel_rolls[rolls.current_roll_index];

        // Spawn animated dice sprite in center of board
        let dice_pos = Vec3::new(0.0, 0.0, 100.0);
        let dice_color = camel_type.to_bevy_color();

        // Spawn the dice sprite with animation
        commands
            .spawn((
                crate::systems::animation::DiceSprite,
                crate::systems::animation::DiceRollAnimation::new_fast(dice_pos),
                Sprite {
                    color: dice_color,
                    custom_size: Some(Vec2::new(60.0, 60.0)),
                    ..default()
                },
                Transform::from_translation(dice_pos),
            ))
            .with_children(|parent| {
                // Spawn pips as children
                let pip_positions = get_pip_positions(value);
                for pip_pos in pip_positions {
                    parent.spawn((
                        Sprite {
                            color: Color::WHITE,
                            custom_size: Some(Vec2::new(10.0, 10.0)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(pip_pos.x, pip_pos.y, 1.0)),
                    ));
                }
            });

        rolls.current_dice_spawned = true;
        rolls.current_camel_moving = false;
        let camel_name = match camel_type {
            InitialRollCamel::Racing(c) => format!("{:?}", c),
            InitialRollCamel::Crazy(c) => format!("{:?} (crazy)", c),
        };
        info!("Initial roll: {} rolled {}", camel_name, value);
    }
}

/// Get pip positions for dice display (like a real die)
fn get_pip_positions(value: u8) -> Vec<Vec2> {
    let offset = 15.0;
    match value {
        1 => vec![Vec2::ZERO],
        2 => vec![Vec2::new(-offset, offset), Vec2::new(offset, -offset)],
        3 => vec![
            Vec2::new(-offset, offset),
            Vec2::ZERO,
            Vec2::new(offset, -offset),
        ],
        _ => vec![Vec2::ZERO],
    }
}
