use std::f32::consts::FRAC_PI_2;

use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};
use rand::Rng;

use crate::landscape::LANDSCAPE_SIZE_HALF;

pub const LASER_SPEED: f32 = 70.0;

#[derive(Component)]
pub struct Gun {
    pub color: Color,
    pub timer: Timer,
}

#[derive(Component)]
pub struct AwaitingSpawnGun {
    pub color: Color,
}

#[derive(Component)]
pub struct Laser {
    pub velocity: Vec3,
}

pub struct LaserPlugin;

impl Plugin for LaserPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_guns, shoot_guns, update_lasers));
    }
}

pub fn spawn_guns(
    query: Query<(Entity, &AwaitingSpawnGun)>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut commands: Commands,
) {
    for (entity, spawn_gun) in query.iter() {
        let mut spawned_guns = false;
        for child in children.iter_descendants(entity) {
            let Ok(name) = names.get(child) else {
                continue;
            };
            if name.contains("laser") {
                spawned_guns = true;
                commands.entity(child).insert(Gun {
                    color: spawn_gun.color,
                    timer: Timer::from_seconds(
                        rand::thread_rng().gen_range(0.6..1.3),
                        TimerMode::Repeating,
                    ),
                });
            }
        }
        if spawned_guns {
            commands.entity(entity).remove::<AwaitingSpawnGun>();
        }
    }
}

pub fn shoot_guns(
    mut commands: Commands,
    mut query: Query<(&mut Gun, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    cameras: Query<&GlobalTransform, With<Camera>>,
    asset_server: Res<AssetServer>,
) {
    let camera_pos = cameras.single().translation();
    for (mut gun, transform) in query.iter_mut() {
        if !gun.timer.tick(time.delta()).just_finished() {
            continue;
        }
        let velocity = transform.forward() * LASER_SPEED;
        let speed = rand::thread_rng().gen_range(0.7..1.4);
        let d = 100.0;
        let volume = 1.0 - (transform.translation().distance(camera_pos) / d).clamp(0.0, 1.0);
        commands.spawn((
            AudioBundle {
                source: asset_server.load("sounds/laser.ogg"),
                settings: PlaybackSettings {
                    mode: PlaybackMode::Once,
                    volume: Volume::new_absolute(volume),
                    speed,
                    paused: false,
                },
            },
            Laser { velocity },
            PbrBundle {
                material: materials.add(StandardMaterial {
                    emissive: gun.color * 5.0,
                    ..default()
                }),
                mesh: meshes.add(
                    shape::Capsule {
                        radius: 0.1,
                        depth: 4.0,
                        ..default()
                    }
                    .into(),
                ),
                transform: Transform::from_translation(transform.translation()).with_rotation(
                    Quat::from_rotation_y(f32::atan2(velocity.x, velocity.z))
                        * Quat::from_rotation_x(FRAC_PI_2),
                ),
                ..default()
            },
        ));
    }
}

fn update_lasers(
    mut query: Query<(&mut Transform, &mut Laser, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut transform, mut laser, entity) in query.iter_mut() {
        // BOOOOOOST
        laser.velocity *= 1.0 + time.delta_seconds() * 6.0;
        transform.translation += laser.velocity * time.delta_seconds();
        if transform.translation.distance(Vec3::ZERO) > LANDSCAPE_SIZE_HALF {
            commands.entity(entity).despawn_recursive();
        }
    }
}
