use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::TAU;
use crate::components::{CamelColor, CrazyCamelColor, CrazyCamel};
use crate::components::board::{PyramidRollButton, PyramidShakeAnimation, PyramidHovered, PyramidHoverBorder};
use crate::systems::movement::{MoveCamelEvent, MoveCrazyCamelEvent};

/// Component for entities that are animating their position
#[derive(Component)]
pub struct MovementAnimation {
    pub start_pos: Vec3,
    pub end_pos: Vec3,
    pub elapsed: f32,
    pub duration: f32,
}

impl MovementAnimation {
    pub fn new(start: Vec3, end: Vec3, duration: f32) -> Self {
        Self {
            start_pos: start,
            end_pos: end,
            elapsed: 0.0,
            duration,
        }
    }
}

/// Component for fading out entities
#[derive(Component)]
pub struct FadeOut {
    pub elapsed: f32,
    pub duration: f32,
    pub despawn_on_complete: bool,
}

/// Component for dice roll result display
#[derive(Component)]
pub struct DiceResultPopup {
    pub display_timer: f32,
    pub fade_timer: f32,
}

impl Default for DiceResultPopup {
    fn default() -> Self {
        Self {
            display_timer: 1.5,
            fade_timer: 0.5,
        }
    }
}

/// System to animate movement with easing
pub fn animate_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MovementAnimation, Option<&CrazyCamel>)>,
) {
    for (entity, mut transform, mut animation, is_crazy) in query.iter_mut() {
        animation.elapsed += time.delta_secs();

        let t = (animation.elapsed / animation.duration).clamp(0.0, 1.0);

        // Ease-out cubic for smooth deceleration
        let eased_t = 1.0 - (1.0 - t).powi(3);

        transform.translation = animation.start_pos.lerp(animation.end_pos, eased_t);

        // Update facing direction based on target row
        // Racing camels: Top row faces left (toward finish), bottom row faces right (toward finish)
        // Crazy camels: OPPOSITE - Top row faces right (away from finish), bottom row faces left
        let target_y = animation.end_pos.y;
        let on_top_row = target_y > 0.0;
        let is_crazy_camel = is_crazy.is_some();

        // Racing camels face toward finish, crazy camels face away from finish
        let should_face_left = if is_crazy_camel { !on_top_row } else { on_top_row };

        if should_face_left {
            transform.scale.x = -transform.scale.x.abs();
        } else {
            transform.scale.x = transform.scale.x.abs();
        }

        if t >= 1.0 {
            commands.entity(entity).remove::<MovementAnimation>();
        }
    }
}

/// System to handle fade out animations
pub fn fade_out_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut FadeOut)>,
) {
    for (entity, mut sprite, mut fade) in query.iter_mut() {
        fade.elapsed += time.delta_secs();

        let alpha = 1.0 - (fade.elapsed / fade.duration).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);

        if fade.elapsed >= fade.duration && fade.despawn_on_complete {
            commands.entity(entity).despawn();
        }
    }
}

/// System to update dice result popup
pub fn dice_result_popup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut DiceResultPopup, &mut Sprite)>,
) {
    for (entity, mut popup, mut sprite) in query.iter_mut() {
        if popup.display_timer > 0.0 {
            popup.display_timer -= time.delta_secs();
        } else if popup.fade_timer > 0.0 {
            popup.fade_timer -= time.delta_secs();
            let alpha = (popup.fade_timer / 0.5).clamp(0.0, 1.0);
            sprite.color = sprite.color.with_alpha(alpha);
        } else {
            commands.entity(entity).despawn();
        }
    }
}

/// Multi-step movement animation for hop-by-hop camel movement
#[derive(Component)]
pub struct MultiStepMovementAnimation {
    pub waypoints: Vec<Vec3>,      // All positions including start
    pub current_segment: usize,    // Which hop we're on (0 = first hop)
    pub segment_elapsed: f32,      // Time in current hop
    pub segment_duration: f32,     // Duration per hop
}

impl MultiStepMovementAnimation {
    pub fn new(waypoints: Vec<Vec3>, segment_duration: f32) -> Self {
        Self {
            waypoints,
            current_segment: 0,
            segment_elapsed: 0.0,
            segment_duration,
        }
    }

    pub fn total_segments(&self) -> usize {
        if self.waypoints.len() > 1 {
            self.waypoints.len() - 1
        } else {
            0
        }
    }

    pub fn current_start(&self) -> Vec3 {
        self.waypoints[self.current_segment]
    }

    pub fn current_end(&self) -> Vec3 {
        self.waypoints[self.current_segment + 1]
    }

    pub fn is_complete(&self) -> bool {
        self.current_segment >= self.total_segments()
    }
}

/// System to animate multi-step movement (hop by hop through spaces)
pub fn animate_multi_step_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MultiStepMovementAnimation, Option<&CrazyCamel>)>,
) {
    for (entity, mut transform, mut animation, is_crazy) in query.iter_mut() {
        if animation.is_complete() {
            commands.entity(entity).remove::<MultiStepMovementAnimation>();
            continue;
        }

        animation.segment_elapsed += time.delta_secs();

        // Calculate progress within current segment
        let t = (animation.segment_elapsed / animation.segment_duration).clamp(0.0, 1.0);

        // Ease-out quadratic for each hop
        let eased_t = 1.0 - (1.0 - t).powi(2);

        // Interpolate position
        let start = animation.current_start();
        let end = animation.current_end();
        transform.translation = start.lerp(end, eased_t);

        // Update facing direction based on which row we're on
        // Racing camels: Top row faces left (toward finish), bottom row faces right (toward finish)
        // Crazy camels: OPPOSITE - Top row faces right (away from finish), bottom row faces left
        let target_y = end.y;
        let on_top_row = target_y > 0.0;
        let is_crazy_camel = is_crazy.is_some();

        // Racing camels face toward finish, crazy camels face away from finish
        let should_face_left = if is_crazy_camel { !on_top_row } else { on_top_row };

        if should_face_left {
            transform.scale.x = -transform.scale.x.abs();
        } else {
            transform.scale.x = transform.scale.x.abs();
        }

        // Check if segment complete
        if t >= 1.0 {
            animation.current_segment += 1;
            animation.segment_elapsed = 0.0;

            // Snap to exact end position of this segment
            if !animation.is_complete() {
                transform.translation = end;
            }
        }
    }
}

// ============================================================================
// Dice Roll Animation System
// ============================================================================

/// Phase of the dice roll animation
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DiceRollPhase {
    Shaking,      // Die shakes rapidly
    Settling,     // Die bounces to final position
    Display,      // Shows result clearly
    MovingToTent, // Moving to tent position
    InTent,       // Resting in tent (final state, no despawn)
}

/// Component for animated dice roll visual
#[derive(Component)]
pub struct DiceRollAnimation {
    pub phase: DiceRollPhase,
    pub elapsed: f32,
    pub shake_duration: f32,
    pub settle_duration: f32,
    pub display_duration: f32,
    pub move_to_tent_duration: f32,
    pub shake_intensity: f32,
    pub original_pos: Vec3,
    pub target_tent_pos: Vec3,  // Where to move after display phase
}

impl DiceRollAnimation {
    pub fn new(original_pos: Vec3, target_tent_pos: Vec3) -> Self {
        Self {
            phase: DiceRollPhase::Shaking,
            elapsed: 0.0,
            shake_duration: 0.5,
            settle_duration: 0.25,
            display_duration: 0.8,
            move_to_tent_duration: 0.4,
            shake_intensity: 12.0,
            original_pos,
            target_tent_pos,
        }
    }

    /// Create a faster dice animation (4x speed) for initial setup
    /// Note: Initial setup dice don't go to tents, so target_tent_pos is same as original_pos
    pub fn new_fast(original_pos: Vec3) -> Self {
        Self {
            phase: DiceRollPhase::Shaking,
            elapsed: 0.0,
            shake_duration: 0.125,
            settle_duration: 0.06,
            display_duration: 0.2,
            move_to_tent_duration: 0.1,
            shake_intensity: 12.0,
            original_pos,
            target_tent_pos: original_pos,  // Initial setup dice fade out, don't go to tent
        }
    }
}

/// Marker component for the dice sprite
#[derive(Component)]
pub struct DiceSprite;

/// Marker component for dice value text
#[derive(Component)]
pub struct DiceValueText;

/// Component to store pending regular camel movement (triggered when dice animation settles)
#[derive(Component)]
pub struct PendingCamelMove {
    pub color: CamelColor,
    pub spaces: u8,
}

/// Component to store pending crazy camel movement (triggered when dice animation settles)
#[derive(Component)]
pub struct PendingCrazyCamelMove {
    pub color: CrazyCamelColor,
    pub spaces: u8,
}

/// System to animate dice roll through phases
pub fn dice_roll_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut DiceRollAnimation, Option<&PendingCamelMove>, Option<&PendingCrazyCamelMove>), With<DiceSprite>>,
    _children_query: Query<&Children>,
    _text_query: Query<&mut Transform, (With<DiceValueText>, Without<DiceSprite>)>,
    mut move_camel: MessageWriter<MoveCamelEvent>,
    mut move_crazy_camel: MessageWriter<MoveCrazyCamelEvent>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut transform, sprite, mut anim, pending_move, pending_crazy_move) in query.iter_mut() {
        anim.elapsed += time.delta_secs();

        match anim.phase {
            DiceRollPhase::Shaking => {
                // Random shake offset
                let shake_x = rng.gen_range(-anim.shake_intensity..anim.shake_intensity);
                let shake_y = rng.gen_range(-anim.shake_intensity..anim.shake_intensity);
                transform.translation.x = anim.original_pos.x + shake_x;
                transform.translation.y = anim.original_pos.y + shake_y;

                // Random rotation
                let rotation = rng.gen_range(-0.3..0.3);
                transform.rotation = Quat::from_rotation_z(rotation);

                // Scale pulse
                let scale_pulse = 1.0 + (anim.elapsed * 25.0).sin() * 0.15;
                transform.scale = Vec3::splat(scale_pulse);

                if anim.elapsed >= anim.shake_duration {
                    anim.elapsed = 0.0;
                    anim.phase = DiceRollPhase::Settling;
                    // Spawn particles when settling begins
                    spawn_dice_particles(&mut commands, anim.original_pos, sprite.color);

                    // Trigger camel movement now that dice has settled on a value
                    if let Some(pending) = pending_move {
                        move_camel.write(MoveCamelEvent {
                            color: pending.color,
                            spaces: pending.spaces,
                        });
                        commands.entity(entity).remove::<PendingCamelMove>();
                    }
                    if let Some(pending) = pending_crazy_move {
                        move_crazy_camel.write(MoveCrazyCamelEvent {
                            color: pending.color,
                            spaces: pending.spaces,
                        });
                        commands.entity(entity).remove::<PendingCrazyCamelMove>();
                    }
                }
            }
            DiceRollPhase::Settling => {
                // Ease-out bounce effect
                let t = (anim.elapsed / anim.settle_duration).clamp(0.0, 1.0);
                let bounce_t = ease_out_bounce(t);

                // Settle to original position
                let current_offset = (1.0 - bounce_t) * 10.0;
                transform.translation.x = anim.original_pos.x;
                transform.translation.y = anim.original_pos.y + current_offset;

                // Settle rotation to 0
                let current_rotation = (1.0 - bounce_t) * 0.2;
                transform.rotation = Quat::from_rotation_z(current_rotation);

                // Settle scale
                let scale = 1.0 + (1.0 - bounce_t) * 0.1;
                transform.scale = Vec3::splat(scale);

                if anim.elapsed >= anim.settle_duration {
                    anim.elapsed = 0.0;
                    anim.phase = DiceRollPhase::Display;
                    transform.translation = anim.original_pos;
                    transform.rotation = Quat::IDENTITY;
                    transform.scale = Vec3::ONE;
                }
            }
            DiceRollPhase::Display => {
                // Subtle pulse during display
                let pulse = 1.0 + (anim.elapsed * 4.0).sin() * 0.03;
                transform.scale = Vec3::splat(pulse);

                if anim.elapsed >= anim.display_duration {
                    anim.elapsed = 0.0;
                    // Check if this is an initial setup dice (target == original) - fade out
                    // Otherwise move to tent
                    if anim.target_tent_pos == anim.original_pos {
                        // Initial setup dice - despawn directly (fast animation)
                        commands.entity(entity).despawn();
                    } else {
                        // Game dice - move to tent
                        anim.phase = DiceRollPhase::MovingToTent;
                    }
                }
            }
            DiceRollPhase::MovingToTent => {
                let t = (anim.elapsed / anim.move_to_tent_duration).clamp(0.0, 1.0);

                // Ease-out cubic for smooth deceleration
                let eased_t = 1.0 - (1.0 - t).powi(3);

                // Interpolate position from original to tent
                transform.translation = anim.original_pos.lerp(anim.target_tent_pos, eased_t);

                // Shrink slightly as it moves to tent (dice in tent are smaller)
                let scale = 1.0 - eased_t * 0.3;  // Shrink to 70% size
                transform.scale = Vec3::splat(scale);

                if t >= 1.0 {
                    anim.elapsed = 0.0;
                    anim.phase = DiceRollPhase::InTent;
                    transform.translation = anim.target_tent_pos;
                    transform.scale = Vec3::splat(0.7);  // Final tent size
                }
            }
            DiceRollPhase::InTent => {
                // Dice stays in tent permanently - do nothing
                // No despawn, no animation changes
            }
        }
    }
}

/// Ease-out bounce function
fn ease_out_bounce(t: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;

    if t < 1.0 / D1 {
        N1 * t * t
    } else if t < 2.0 / D1 {
        let t = t - 1.5 / D1;
        N1 * t * t + 0.75
    } else if t < 2.5 / D1 {
        let t = t - 2.25 / D1;
        N1 * t * t + 0.9375
    } else {
        let t = t - 2.625 / D1;
        N1 * t * t + 0.984375
    }
}

// ============================================================================
// Particle System
// ============================================================================

/// Component for individual particles
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub elapsed: f32,
    pub gravity: f32,
}

/// Marker for particle entities
#[derive(Component)]
pub struct ParticleMarker;

/// Spawn burst of particles at position
pub fn spawn_dice_particles(commands: &mut Commands, pos: Vec3, base_color: Color) {
    let mut rng = rand::thread_rng();
    let particle_count = 16;

    for _ in 0..particle_count {
        // Random direction outward
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(80.0..200.0);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        // Random size
        let size = rng.gen_range(4.0..8.0);

        // Vary the alpha
        let alpha = rng.gen_range(0.6..1.0);
        let color = base_color.with_alpha(alpha);

        // Random lifetime
        let lifetime = rng.gen_range(0.6..1.0);

        commands.spawn((
            ParticleMarker,
            Particle {
                velocity,
                lifetime,
                elapsed: 0.0,
                gravity: 150.0,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(pos + Vec3::new(
                rng.gen_range(-5.0..5.0),
                rng.gen_range(-5.0..5.0),
                1.0,
            )),
        ));
    }
}

/// System to update particles
pub fn particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut Particle), With<ParticleMarker>>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut sprite, mut particle) in query.iter_mut() {
        particle.elapsed += dt;

        // Apply velocity
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Apply gravity
        particle.velocity.y -= particle.gravity * dt;

        // Fade out over lifetime
        let life_ratio = (particle.elapsed / particle.lifetime).clamp(0.0, 1.0);
        let alpha = 1.0 - life_ratio;
        sprite.color = sprite.color.with_alpha(alpha);

        // Shrink over time
        let scale = 1.0 - life_ratio * 0.5;
        transform.scale = Vec3::splat(scale);

        // Despawn when lifetime exceeded
        if particle.elapsed >= particle.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Firework Celebration System
// ============================================================================

/// Marker for firework entities
#[derive(Component)]
pub struct FireworkMarker;

/// A firework projectile that launches upward then explodes
#[derive(Component)]
pub struct Firework {
    pub velocity: Vec2,
    pub fuse_time: f32,      // Time until explosion
    pub elapsed: f32,
    pub color: Color,
}

/// Marker for explosion particle entities
#[derive(Component)]
pub struct ExplosionParticleMarker;

/// An explosion particle from a firework
#[derive(Component)]
pub struct ExplosionParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub elapsed: f32,
    pub gravity: f32,
}

/// Spawn a firework from the bottom of the screen
pub fn spawn_firework(commands: &mut Commands, x_pos: f32, color: Color) {
    let mut rng = rand::thread_rng();

    // Launch position at bottom of screen
    let start_pos = Vec3::new(x_pos, -400.0, 50.0);

    // Upward velocity with slight random angle
    let angle_offset = rng.gen_range(-0.2..0.2);
    let speed = rng.gen_range(800.0..1000.0);
    let velocity = Vec2::new(angle_offset * 100.0, speed);

    // Fuse time determines how high it goes before exploding
    let fuse_time = rng.gen_range(0.2..0.8);

    commands.spawn((
        FireworkMarker,
        Firework {
            velocity,
            fuse_time,
            elapsed: 0.0,
            color,
        },
        Sprite {
            color,
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(start_pos),
    ));
}

/// Spawn explosion particles when a firework explodes
fn spawn_firework_explosion(commands: &mut Commands, pos: Vec3, base_color: Color) {
    let mut rng = rand::thread_rng();
    let particle_count = rng.gen_range(25..40);

    for _ in 0..particle_count {
        // Random direction in full circle
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(80.0..200.0);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        // Vary the color slightly
        let color_variation = rng.gen_range(0.8..1.2);
        let r = (base_color.to_srgba().red * color_variation).clamp(0.0, 1.0);
        let g = (base_color.to_srgba().green * color_variation).clamp(0.0, 1.0);
        let b = (base_color.to_srgba().blue * color_variation).clamp(0.0, 1.0);
        let particle_color = Color::srgba(r, g, b, 1.0);

        let size = rng.gen_range(4.0..8.0);
        let lifetime = rng.gen_range(0.8..1.5);

        commands.spawn((
            ExplosionParticleMarker,
            ExplosionParticle {
                velocity,
                lifetime,
                elapsed: 0.0,
                gravity: 120.0,
            },
            Sprite {
                color: particle_color,
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(pos + Vec3::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(-3.0..3.0),
                0.0,
            )),
        ));
    }
}

/// System to update fireworks - move them and trigger explosions
pub fn firework_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Firework), With<FireworkMarker>>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut firework) in query.iter_mut() {
        firework.elapsed += dt;

        // Apply velocity (fireworks slow down as they rise)
        transform.translation.x += firework.velocity.x * dt;
        transform.translation.y += firework.velocity.y * dt;

        // Apply gravity to slow the ascent
        firework.velocity.y -= 200.0 * dt;

        // Check if fuse has burned - time to explode!
        if firework.elapsed >= firework.fuse_time {
            // Spawn explosion particles
            spawn_firework_explosion(&mut commands, transform.translation, firework.color);
            // Despawn the firework projectile
            commands.entity(entity).despawn();
        }
    }
}

/// System to update explosion particles
pub fn explosion_particle_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut ExplosionParticle), With<ExplosionParticleMarker>>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut sprite, mut particle) in query.iter_mut() {
        particle.elapsed += dt;

        // Apply velocity
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Apply gravity - particles fall
        particle.velocity.y -= particle.gravity * dt;

        // Add some drag to slow horizontal movement
        particle.velocity.x *= 0.99;

        // Fade out over lifetime
        let life_ratio = (particle.elapsed / particle.lifetime).clamp(0.0, 1.0);
        let alpha = 1.0 - life_ratio;
        sprite.color = sprite.color.with_alpha(alpha);

        // Shrink slightly over time
        let scale = 1.0 - life_ratio * 0.3;
        transform.scale = Vec3::splat(scale);

        // Despawn when lifetime exceeded
        if particle.elapsed >= particle.lifetime {
            commands.entity(entity).despawn();
        }
    }
}

/// Get a random celebration color for fireworks
pub fn random_firework_color() -> Color {
    let mut rng = rand::thread_rng();
    let colors = [
        Color::srgb(1.0, 0.84, 0.0),   // Gold
        Color::srgb(1.0, 0.2, 0.2),    // Red
        Color::srgb(0.2, 0.6, 1.0),    // Blue
        Color::srgb(0.2, 1.0, 0.4),    // Green
        Color::srgb(0.8, 0.2, 1.0),    // Purple
        Color::srgb(1.0, 0.5, 0.0),    // Orange
        Color::srgb(1.0, 1.0, 1.0),    // White
    ];
    colors[rng.gen_range(0..colors.len())]
}

// ============================================================================
// Crown Animation System (for winning camel)
// ============================================================================

/// Marker for crown entities
#[derive(Component)]
pub struct CrownMarker;

/// Component for crown drop animation
#[derive(Component)]
pub struct CrownDropAnimation {
    pub final_pos: Vec3,
    pub drop_height: f32,
    pub elapsed: f32,
    pub duration: f32,
}

impl CrownDropAnimation {
    pub fn new(final_pos: Vec3, drop_height: f32, duration: f32) -> Self {
        Self {
            final_pos,
            drop_height,
            elapsed: 0.0,
            duration,
        }
    }
}

/// Spawn a crown entity with layered sprites matching the camel style
/// If drop_height is Some, the crown will animate dropping from above
pub fn spawn_crown(commands: &mut Commands, position: Vec3, drop_height: Option<f32>) -> Entity {
    // Crown colors
    let gold = Color::srgb(1.0, 0.84, 0.0);
    let gold_dark = Color::srgb(0.6, 0.5, 0.0);      // Border
    let gold_light = Color::srgb(1.0, 0.95, 0.6);    // Highlight
    let jewel_red = Color::srgb(0.8, 0.1, 0.1);
    let shadow_color = Color::srgba(0.0, 0.0, 0.0, 0.3);

    // Crown dimensions
    let base_width = 18.0;
    let base_height = 6.0;
    let point_width = 6.0;
    let point_height = 10.0;
    let point_top_size = 4.0;
    let jewel_size = 4.0;

    // Point positions (left, center, right)
    let point_spacing = 6.0;
    let point_y = base_height / 2.0 + point_height / 2.0;

    // Start position: offset upward if animating, otherwise at final position
    let start_pos = if let Some(height) = drop_height {
        Vec3::new(position.x, position.y + height, position.z)
    } else {
        position
    };

    let mut entity_commands = commands.spawn((
        CrownMarker,
        Transform::from_translation(start_pos),
        Visibility::default(),
    ));

    // Add drop animation if requested
    if let Some(height) = drop_height {
        entity_commands.insert(CrownDropAnimation::new(position, height, 0.8));
    }

    let crown_entity = entity_commands.with_children(|parent| {
        // Shadow layer (offset +2, -2)
        let shadow_offset = Vec3::new(2.0, -2.0, -0.3);

        // Shadow: base
        parent.spawn((
            Sprite {
                color: shadow_color,
                custom_size: Some(Vec2::new(base_width, base_height)),
                ..default()
            },
            Transform::from_translation(shadow_offset),
        ));
        // Shadow: points
        for i in [-1, 0, 1] {
            parent.spawn((
                Sprite {
                    color: shadow_color,
                    custom_size: Some(Vec2::new(point_width, point_height)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(i as f32 * point_spacing, point_y, 0.0) + shadow_offset),
            ));
        }

        // Border layer (enlarged by 2px)
        let border_expand = 2.0;

        // Border: base
        parent.spawn((
            Sprite {
                color: gold_dark,
                custom_size: Some(Vec2::new(base_width + border_expand, base_height + border_expand)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.2),
        ));
        // Border: points
        for i in [-1, 0, 1] {
            parent.spawn((
                Sprite {
                    color: gold_dark,
                    custom_size: Some(Vec2::new(point_width + border_expand, point_height + border_expand)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * point_spacing, point_y, -0.2),
            ));
            // Border: point tops
            parent.spawn((
                Sprite {
                    color: gold_dark,
                    custom_size: Some(Vec2::splat(point_top_size + border_expand)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * point_spacing, point_y + point_height / 2.0 + point_top_size / 2.0, -0.2),
            ));
        }

        // Main gold layer
        // Base band
        parent.spawn((
            Sprite {
                color: gold,
                custom_size: Some(Vec2::new(base_width, base_height)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -0.1),
        ));
        // Points
        for i in [-1, 0, 1] {
            parent.spawn((
                Sprite {
                    color: gold,
                    custom_size: Some(Vec2::new(point_width, point_height)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * point_spacing, point_y, -0.1),
            ));
            // Point tops (small squares)
            parent.spawn((
                Sprite {
                    color: gold,
                    custom_size: Some(Vec2::splat(point_top_size)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * point_spacing, point_y + point_height / 2.0 + point_top_size / 2.0, -0.1),
            ));
        }

        // Highlights on point tops
        for i in [-1, 0, 1] {
            parent.spawn((
                Sprite {
                    color: gold_light.with_alpha(0.5),
                    custom_size: Some(Vec2::new(point_top_size - 1.0, 2.0)),
                    ..default()
                },
                Transform::from_xyz(i as f32 * point_spacing, point_y + point_height / 2.0 + point_top_size / 2.0 + 0.5, 0.0),
            ));
        }

        // Jewel in center of base
        parent.spawn((
            Sprite {
                color: jewel_red,
                custom_size: Some(Vec2::splat(jewel_size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.1),
        ));
    }).id();

    crown_entity
}

/// System to animate the crown dropping onto the winning camel
pub fn crown_drop_system(
    mut commands: Commands,
    time: Res<Time>,
    mut crown_query: Query<(Entity, &mut Transform, &mut CrownDropAnimation), With<CrownMarker>>,
) {
    for (entity, mut crown_transform, mut animation) in crown_query.iter_mut() {
        animation.elapsed += time.delta_secs();

        let t = (animation.elapsed / animation.duration).clamp(0.0, 1.0);

        // Ease-out bounce for satisfying landing
        let eased_t = ease_out_bounce(t);

        // Interpolate Y position from start (final + drop_height) down to final
        let start_y = animation.final_pos.y + animation.drop_height;
        let current_y = start_y + (animation.final_pos.y - start_y) * eased_t;

        crown_transform.translation.x = animation.final_pos.x;
        crown_transform.translation.y = current_y;
        crown_transform.translation.z = animation.final_pos.z;

        // Animation complete
        if t >= 1.0 {
            commands.entity(entity).remove::<CrownDropAnimation>();
            // Snap to final position
            crown_transform.translation = animation.final_pos;
        }
    }
}

// ============================================================================
// Pyramid Shake Animation System
// ============================================================================

/// System to animate the pyramid roll button shake when clicked
pub fn animate_pyramid_shake(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut PyramidShakeAnimation), With<PyramidRollButton>>,
    mut commands: Commands,
) {
    for (entity, mut transform, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_secs();
        let progress = (anim.elapsed / anim.duration).clamp(0.0, 1.0);

        // Rotation shake with decay
        let intensity = 0.15; // ~8.5 degrees max rotation
        let frequency = 4.0;  // number of full oscillations
        let decay = 1.0 - progress;  // decreases over time
        let angle = (progress * frequency * TAU).sin() * intensity * decay;

        transform.rotation = Quat::from_rotation_z(angle);

        // Animation complete
        if progress >= 1.0 {
            transform.rotation = Quat::IDENTITY;
            commands.entity(entity).remove::<PyramidShakeAnimation>();
        }
    }
}

/// System to animate the pyramid hover effect (scale up when hovered)
pub fn animate_pyramid_hover(
    time: Res<Time>,
    mut pyramid_query: Query<(&mut Transform, Option<&PyramidHovered>, Option<&PyramidShakeAnimation>, &Children), With<PyramidRollButton>>,
    mut border_query: Query<&mut Visibility, With<PyramidHoverBorder>>,
) {
    const HOVER_SCALE: f32 = 1.12;
    const NORMAL_SCALE: f32 = 1.0;
    const LERP_SPEED: f32 = 8.0;

    for (mut transform, hovered, shaking, children) in pyramid_query.iter_mut() {
        let is_hovered = hovered.is_some();

        // Update border visibility for all children
        for child in children.iter() {
            if let Ok(mut visibility) = border_query.get_mut(child) {
                *visibility = if is_hovered {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
            }
        }

        // Don't modify scale while shaking (shake animation handles its own transforms)
        if shaking.is_some() {
            continue;
        }

        let target_scale = if is_hovered { HOVER_SCALE } else { NORMAL_SCALE };
        let current_scale = transform.scale.x;

        // Smoothly lerp to target scale
        let new_scale = current_scale + (target_scale - current_scale) * LERP_SPEED * time.delta_secs();
        transform.scale = Vec3::splat(new_scale);
    }
}

// ============================================================================
// Camera Zoom Animation System
// ============================================================================

/// Component for smooth camera zoom transitions
#[derive(Component)]
pub struct CameraZoomAnimation {
    pub start_scale: f32,
    pub target_scale: f32,
    pub elapsed: f32,
    pub duration: f32,
}

impl CameraZoomAnimation {
    pub fn new(start: f32, target: f32, duration: f32) -> Self {
        Self {
            start_scale: start,
            target_scale: target,
            elapsed: 0.0,
            duration,
        }
    }
}

/// System to animate camera zoom transitions with easing
pub fn animate_camera_zoom(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Projection, &mut CameraZoomAnimation), With<Camera2d>>,
    mut commands: Commands,
) {
    for (entity, mut projection, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta_secs();
        let progress = (anim.elapsed / anim.duration).clamp(0.0, 1.0);

        // Ease-out cubic for smooth deceleration
        let eased = 1.0 - (1.0 - progress).powi(3);

        let scale = anim.start_scale + (anim.target_scale - anim.start_scale) * eased;

        if let Projection::Orthographic(ref mut ortho) = *projection {
            ortho.scale = scale;
        }

        if progress >= 1.0 {
            commands.entity(entity).remove::<CameraZoomAnimation>();
        }
    }
}
