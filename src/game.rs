use crate::insert_dependant_component;
use bevy::{prelude::*, input::mouse::MouseMotion};
use bevy_rapier3d::prelude::*;
use bevy::transform::components::Transform;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub enum SoundMaterial {
    Metal,
    Wood,
    Rock,
    Cloth,
    Squishy,
    #[default]
    None,
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Player;
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct PlayerCamera;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo component showing auto injection of components
pub struct ShouldBeWithPlayer;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Interactible;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
/// Demo marker component
pub struct Pickable;

fn player_move_demo(
    keycode: Res<Input<KeyCode>>,
    mut players_query: Query<&mut Transform, With<Player>>,
    mut player_cameras_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    mut ev_motion: EventReader<MouseMotion>,
    primary_window: Query<&Window>,
) {
    let player_result = players_query.get_single_mut();
    if player_result.is_err() { return; }
    let player_camera_result = player_cameras_query.get_single_mut();
    if player_camera_result.is_err() { return; }
    let primary_window_result = primary_window.get_single();
    if player_camera_result.is_err() { return; }

    let window = primary_window_result.unwrap();
    let mut player = player_result.unwrap();
    let mut player_camera = player_camera_result.unwrap();

    let speed = 0.2;

    let f = player_camera.forward();
    let bearing_vector = Vec3::new(f.x, 0.0, f.z).normalize();
    if keycode.pressed(KeyCode::W) {
        player.translation += speed * bearing_vector;
    }

    if keycode.pressed(KeyCode::A) {
        player.translation += Vec3::new(bearing_vector.z, 0.0, bearing_vector.x * -1.0) * speed;
    }

    if keycode.pressed(KeyCode::D) {
        player.translation += Vec3::new(bearing_vector.z * -1.0, 0.0, bearing_vector.x) * speed;
    }

    if keycode.pressed(KeyCode::S) {
        player.translation += speed * bearing_vector * -1.0;
    }

    for ev in ev_motion.iter() {
        let (mut yaw, mut pitch, _) = player_camera.rotation.to_euler(EulerRot::YXZ);
        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
        let window_scale = window.height().min(window.width());
        pitch -= (1.0 * ev.delta.y * window_scale / 1000.0).to_radians();
        yaw -= (1.0 * ev.delta.x * window_scale / 1000.0).to_radians();

        pitch = pitch.clamp(-1.54, 1.54);

        // Order is important to prevent unintended roll
        player_camera.rotation =
            Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    }


}

fn configure_player_entity(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), Added<Player>>,
) {
    let entity_result = query.get_single_mut();

    if entity_result.is_err() { return; }
    let (entity, transform) = entity_result.unwrap();


    println!("player found, initializing");

    commands.entity(entity)
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(AdditionalMassProperties::Mass(10.0))
        .insert(ExternalForce {
            force: Vec3::new(0.0, 0.0, 30.0),
            torque: Vec3::ZERO,
        })
        .with_children(|parent| {
            parent
                .spawn(Camera3dBundle::default())
                .insert(PlayerCamera);
        });
}

// collision tests/debug
pub fn test_collision_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut contact_force_events: EventReader<ContactForceEvent>,
) {
    for collision_event in collision_events.iter() {
        println!("collision");
        match collision_event {
            CollisionEvent::Started(_entity1, _entity2, _) => {
                println!("collision started")
            }
            CollisionEvent::Stopped(_entity1, _entity2, _) => {
                println!("collision ended")
            }
        }
    }

    for contact_force_event in contact_force_events.iter() {
        println!("Received contact force event: {:?}", contact_force_event);
    }
}

pub struct DemoPlugin;
impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Interactible>()
            .register_type::<Pickable>()
            .register_type::<SoundMaterial>()
            .register_type::<Player>()
            .register_type::<PlayerCamera>()
            // little helper utility, to automatically inject components that are dependant on an other component
            // ie, here an Entity with a Player component should also always have a ShouldBeWithPlayer component
            // you get a warning if you use this, as I consider this to be stop-gap solution (usually you should have either a bundle, or directly define all needed components)
            .add_systems(
                Update,
                (
                    insert_dependant_component::<Player, ShouldBeWithPlayer>,
                    player_move_demo, //.run_if(in_state(AppState::Running)),
                    test_collision_events,
                    configure_player_entity,
                ),
            );
    }
}
