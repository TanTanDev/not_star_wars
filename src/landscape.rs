use bevy::{
    math::vec3,
    prelude::*,
    render::{
        render_resource::{AsBindGroup, ShaderRef},
        texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor},
    },
};

pub const LANDSCAPE_SIZE: f32 = 1200.0;
pub const LANDSCAPE_SIZE_HALF: f32 = LANDSCAPE_SIZE * 0.5;

#[derive(Component)]
pub struct MoveWithLandscapeTag;

#[derive(Resource)]
pub struct CurrentLandscapeMaterial(pub Handle<LandscapeMaterial>);

pub struct LandscapePlugin;

impl Plugin for LandscapePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_time_uniform,
                    set_textures_repeating,
                    move_with_landscape,
                ),
            )
            .add_plugins(MaterialPlugin::<LandscapeMaterial>::default());
    }
}

fn move_with_landscape(
    mut query: Query<(&mut Transform, Entity), With<MoveWithLandscapeTag>>,
    materials: Res<Assets<LandscapeMaterial>>,
    current_landscape_materials: Res<CurrentLandscapeMaterial>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let landscape_material = materials.get(&current_landscape_materials.0).expect("wops");
    let delta = landscape_material.speed * time.delta_seconds();
    for (mut transform, entity) in query.iter_mut() {
        transform.translation.z -= delta;
        if transform.translation.z >= LANDSCAPE_SIZE_HALF {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn set_textures_repeating(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    for event in texture_events.read() {
        match event {
            AssetEvent::Added { id } => {
                let Some(texture) = textures.get_mut(*id) else {
                    continue;
                };
                texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                    address_mode_u: ImageAddressMode::Repeat,
                    address_mode_v: ImageAddressMode::Repeat,
                    ..default()
                });
            }
            _ => (),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<LandscapeMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let quad = shape::Plane {
        size: LANDSCAPE_SIZE,
        subdivisions: 1000,
    };
    let material = materials.add(LandscapeMaterial {
        time: 0.0,
        speed: -80.0,
        terrain_height: 10.0,
        terrain_size: 1.7,
        uv_scaling: 2.5,
        quad_size: LANDSCAPE_SIZE,
        color_texture: asset_server.load("textures/ground.png"),
    });

    commands.insert_resource(CurrentLandscapeMaterial(material.clone()));
    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(quad.into()),
        material,
        transform: Transform::from_translation(vec3(0.0, -25.0, 0.0)),
        ..default()
    });
}

pub fn update_time_uniform(mut materials: ResMut<Assets<LandscapeMaterial>>, time: Res<Time>) {
    for material in materials.iter_mut() {
        material.1.time = time.elapsed_seconds();
    }
}

#[derive(Reflect, Asset, AsBindGroup, Debug, Clone)]
pub struct LandscapeMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(0)]
    speed: f32,
    #[uniform(0)]
    terrain_height: f32,
    #[uniform(0)]
    terrain_size: f32,
    #[uniform(0)]
    uv_scaling: f32,
    #[uniform(0)]
    quad_size: f32,

    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

impl Material for LandscapeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/landscape.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/landscape.wgsl".into()
    }
}
