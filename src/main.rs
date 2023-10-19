pub mod landscape;
pub mod laser;

use bevy::{
    core_pipeline::bloom::{BloomCompositeMode, BloomPrefilterSettings, BloomSettings},
    input::mouse::{MouseMotion, MouseWheel},
    math::vec3,
    pbr::DirectionalLightShadowMap,
    prelude::*,
};
use landscape::{LandscapePlugin, MoveWithLandscapeTag, LANDSCAPE_SIZE_HALF};
use laser::{AwaitingSpawnGun, LaserPlugin};
use rand::Rng;
use std::f32::consts::*;

fn main() {
    App::new()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_plugins(LandscapePlugin)
        .add_plugins(LaserPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_light_direction)
        .add_systems(Update, start_walker_animation)
        .add_systems(Update, spawn_objects)
        .add_systems(Update, camera_input)
        .add_systems(Update, move_plane)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(PlaneSettings {
        move_interval: 1.3,
        box_area: 6.0,
        speed: 1.5,
        wobble_speed: 5.0,
        rotation_speed: 0.7,
    });
    commands.insert_resource(ClearColor(Color::rgb(0.7, 0.92, 0.96)));
    commands.insert_resource(WalkerAnimation(
        asset_server.load("models/walker/walker.gltf#Animation0"),
    ));
    commands.spawn((
        CameraController {
            rotation: Quat::IDENTITY,
            zoom: 20.0,
        },
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_xyz(0.7, 20.0, 40.0)
                .looking_at(Vec3::new(0.0, 0.3, 0.0), Vec3::Y),
            ..default()
        },
        FogSettings {
            color: Color::rgb_u8(117, 202, 215),
            directional_light_color: Color::WHITE,
            directional_light_exponent: 30.0,
            falloff: FogFalloff::Linear {
                start: 0.0,
                end: LANDSCAPE_SIZE_HALF,
            },
        },
        BloomSettings {
            intensity: 1.0,
            low_frequency_boost: 0.5,
            low_frequency_boost_curvature: 0.5,
            high_pass_frequency: 0.5,
            prefilter_settings: BloomPrefilterSettings {
                threshold: 3.0,
                threshold_softness: 0.6,
            },
            composite_mode: BloomCompositeMode::Additive,
        },
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });
    commands.spawn((
        PlaneMovement {
            target_pos: Vec3::ZERO,
            timer: 0.0,
        },
        AwaitingSpawnGun { color: Color::RED },
        SceneBundle {
            scene: asset_server.load("models/xwing/xwing.gltf#Scene0"),
            ..default()
        },
    ));
}

fn random_transform(x_close: f32) -> Transform {
    let mut rng = rand::thread_rng();
    let x_far = 400.0;
    let flip = (rng.gen_range(0..=1) * 2 - 1) as f32;
    Transform::from_xyz(
        rng.gen_range(x_close..x_far) * flip,
        -20.0,
        -LANDSCAPE_SIZE_HALF,
    )
    .with_rotation(Quat::from_rotation_y(rng.gen_range(0.0..PI * 2.0)))
}

fn spawn_objects(
    mut timer: Local<f32>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    *timer -= time.delta_seconds();
    if *timer >= 0.0 {
        return;
    }
    *timer += 1.0;
    let mut rng = rand::thread_rng();
    match rng.gen_range(0..=100) {
        0..=70 => {
            commands.spawn((
                MoveWithLandscapeTag,
                AwaitingSpawnGun {
                    color: Color::GREEN,
                },
                SceneBundle {
                    scene: asset_server.load("models/walker/walker.gltf#Scene0"),
                    transform: random_transform(36.0),
                    ..default()
                },
            ));
        }
        71..=80 => {
            commands.spawn((
                MoveWithLandscapeTag,
                SceneBundle {
                    scene: asset_server.load("models/desert_rock_column/scene.gltf#Scene0"),
                    transform: random_transform(30.0),
                    ..default()
                },
            ));
        }
        _ => {
            commands.spawn((
                MoveWithLandscapeTag,
                SceneBundle {
                    scene: asset_server.load("models/desert_cliff_6/scene.gltf#Scene0"),
                    transform: random_transform(90.0),
                    ..default()
                },
            ));
        }
    }
}

#[derive(Component)]
pub struct CameraController {
    pub rotation: Quat,
    pub zoom: f32,
}

pub fn camera_input(
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    buttons: Res<Input<MouseButton>>,
    mut query: Query<(&mut CameraController, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut controller, mut transform) in query.iter_mut() {
        for wheel in mouse_wheel.iter() {
            controller.zoom += wheel.y;
        }
        if buttons.pressed(MouseButton::Right) {
            for mouse in mouse_motion.iter() {
                let delta = mouse.delta * time.delta_seconds() * 0.3;
                controller.rotation *= Quat::from_euler(EulerRot::XYZ, -delta.y, -delta.x, 0.0);
            }
        }
        transform.translation = controller.rotation * Vec3::Z * controller.zoom;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}

#[derive(Resource)]
pub struct WalkerAnimation(pub Handle<AnimationClip>);

fn start_walker_animation(
    walker_animation: Res<WalkerAnimation>,
    mut walkers: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut animation_player in walkers.iter_mut() {
        animation_player
            .play(walker_animation.0.clone_weak())
            .repeat();
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

#[derive(Component)]
pub struct PlaneMovement {
    target_pos: Vec3,
    timer: f32,
}

#[derive(Resource, Reflect)]
pub struct PlaneSettings {
    wobble_speed: f32,
    rotation_speed: f32,
    move_interval: f32,
    box_area: f32,
    speed: f32,
}

fn move_plane(
    mut query: Query<(&mut Transform, &mut PlaneMovement)>,
    time: Res<Time>,
    plane_settings: Res<PlaneSettings>,
) {
    let mut rng = rand::thread_rng();
    for (mut transform, mut plane_movement) in query.iter_mut() {
        plane_movement.timer -= time.delta_seconds();
        let dir =
            (transform.translation - (plane_movement.target_pos + Vec3::Z * 10.0)).normalize();
        let rot = Quat::from_rotation_arc(transform.forward(), dir);
        transform.rotation = transform
            .rotation
            .lerp(rot, plane_settings.rotation_speed * time.delta_seconds());

        let scaled_time = time.elapsed_seconds() * plane_settings.wobble_speed;
        let wobble = vec3(scaled_time.sin(), scaled_time.cos(), 0.0);
        let target_pos = plane_movement.target_pos + wobble;
        if plane_movement.timer < 0.0 {
            plane_movement.timer += plane_settings.move_interval;
            plane_movement.target_pos = vec3(
                rng.gen_range(-plane_settings.box_area..plane_settings.box_area),
                rng.gen_range(-plane_settings.box_area..plane_settings.box_area),
                rng.gen_range(-plane_settings.box_area..plane_settings.box_area),
            );
        }
        transform.translation = transform
            .translation
            .lerp(target_pos, plane_settings.speed * time.delta_seconds());
    }
}
