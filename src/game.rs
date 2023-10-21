use bevy::{prelude::*, input::mouse::MouseMotion, window::NormalizedWindowRef};
use bevy_rapier3d::prelude::*;
use bevy::transform::components::Transform;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct Player;
#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct PlayerCamera;

macro_rules! get_single_mut {
    ($x:expr) => {
        {
            let result = $x.get_single_mut();
            if result.is_err() { 
                return; 
            }
            result.unwrap()
        }
    };
}

macro_rules! get_single {
    ($x:expr) => {
        {
            let result = $x.get_single();
            if result.is_err() { 
                return; 
            }
            result.unwrap()
        }
    };
}

fn player_move(
    keycode: Res<Input<KeyCode>>,
    mut players_query: Query<(&mut KinematicCharacterController, Option<&mut KinematicCharacterControllerOutput>), With<Player>>,
    mut player_cameras_query: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    ev_motion: EventReader<MouseMotion>,
    primary_window: Query<&Window>,
    time: Res<Time>,
) {
    let sensitivity = 5.0; 
    let friction = 1.0; 
    let max_accel = 32.0;
    let run_speed = 18.0;
    let stop_speed = 20.0;
    let gravity = 30.;
    let jump_force = 10.;

    let (mut controller, controller_output_option) = get_single_mut!(players_query);
    let player_camera = get_single_mut!(player_cameras_query);
    let window = get_single!(primary_window);

    let (current_velocity, grounded) = {
        if let Some(controller_output) = controller_output_option {
            (
                controller_output.effective_translation / time.delta_seconds(),
                controller_output.grounded,
            )
        } else { (Vec3::ZERO, false) }
    };

    println!("{}", current_velocity.length());

    let current_velocity_xz = Vec3::new(
        current_velocity.x,
        0.0,
        current_velocity.z,
    );

    let speed = current_velocity_xz.length();

    let current_velocity_with_friction_xz = {
        if speed < 0.1 {
            Vec3::ZERO
        } else if grounded {
            let control = if speed > stop_speed { speed } else { stop_speed };
            let speed_loss = control * friction * time.delta_seconds();


            let mut new_speed = speed - speed_loss;

            if new_speed < 0. { new_speed = 0.; }

            let new_speed_ratio = new_speed / speed;


            Vec3::new(
                current_velocity.x * new_speed_ratio,
                0.0,
                current_velocity.z * new_speed_ratio,
            )
        } else {
            Vec3::new(
                current_velocity.x,
                0.0,
                current_velocity.z,
            )
        }
    };

    let wished_direction = {
        let f = player_camera.forward();
        let bearing_vector = Vec3::new(f.x, 0.0, f.z).normalize_or_zero();
        let mut wished_direction = Vec3::ZERO;
        if keycode.pressed(KeyCode::W) {
            wished_direction += bearing_vector;
        }

        if keycode.pressed(KeyCode::A) {
            wished_direction += Vec3::new(bearing_vector.z, 0.0, bearing_vector.x * -1.0);
        }

        if keycode.pressed(KeyCode::D) {
            wished_direction += Vec3::new(bearing_vector.z * -1.0, 0.0, bearing_vector.x);
        }

        if keycode.pressed(KeyCode::S) {
            wished_direction += bearing_vector * -1.0;
        }
        wished_direction.normalize_or_zero()
    };
    
    //Quake style speed calculations
    let new_velocity_xz = {
        let current_speed = current_velocity_with_friction_xz.dot(wished_direction);
        let add_speed = run_speed - current_speed;
        let add_speed_with_cap = f32::max(f32::min(add_speed, max_accel * time.delta_seconds()), 0.0);
        current_velocity_with_friction_xz + (wished_direction * add_speed_with_cap)
    };

    let jump_velocity = if keycode.pressed(KeyCode::Space) && grounded {
        jump_force
    } else {
        0.0
    };

    let new_velocity_y = Vec3::new(
        0.0, 
        current_velocity.y - (gravity * time.delta_seconds()) + jump_velocity,
        0.0,
    );


    let new_velocity = new_velocity_y + new_velocity_xz;

    controller.translation = Some(new_velocity * time.delta_seconds());
    

    adjust_player_camera(
        ev_motion,
        player_camera,
        window,
        sensitivity,
    );

}

fn adjust_player_camera(
    mut ev_motion: EventReader<MouseMotion>,
    mut player_camera: Mut<Transform>,
    window: &Window,
    sensitivity: f32,
){
    for ev in ev_motion.iter() {
        let (mut yaw, mut pitch, _) = player_camera.rotation.to_euler(EulerRot::YXZ);
        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
        let window_scale = window.height().min(window.width());
        pitch -= (sensitivity * ev.delta.y * window_scale / 10000.0).to_radians();
        yaw -= (sensitivity * ev.delta.x * window_scale / 10000.0).to_radians();

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
        .insert(RigidBody::KinematicPositionBased)
        .insert(Collider::ball(0.8))
        .insert(KinematicCharacterController{
            offset: CharacterLength::Absolute(0.5),
            ..default()
        })
        .insert(Restitution {
            coefficient: 0.2,
            combine_rule: CoefficientCombineRule::Min,
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
        app
            .register_type::<Player>()
            .register_type::<PlayerCamera>()
            .add_systems(
                Update,
                (
                    player_move,
                    test_collision_events,
                    configure_player_entity,
                ),
            );
    }
}
