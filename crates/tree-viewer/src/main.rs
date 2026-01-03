use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use smooth_bevy_cameras::{
    controllers::fps::{FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::default())
        .init_resource::<InputMode>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_input_mode,
                camera_movement,
                update_cursor_state,
                update_camera_controller,
            ),
        )
        .run();
}

/// Core input mode system - central UI concept
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Pointer mode: Mouse is visible and free, can interact with UI
    Pointer,
    /// Navigator mode: Mouse is grabbed for camera control, no cursor visible
    Navigator,
}

impl Default for InputMode {
    fn default() -> Self {
        Self::Pointer
    }
}

#[derive(Component)]
struct FlyCam {
    /// Base movement speed
    base_speed: f32,
}

impl Default for FlyCam {
    fn default() -> Self {
        Self { base_speed: 5.0 }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with FPS controller and FlyCam component
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            FlyCam::default(),
        ))
        .insert(FpsCameraBundle::new(
            FpsCameraController::default(),
            Vec3::new(0.0, 5.0, 10.0),
            Vec3::ZERO,
            Vec3::Y,
        ));

    // Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.2, 0.0)),
    ));

    // Ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 200.0,
    });

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Grid of cubes to demonstrate 3D space
    for x in -3..=3 {
        for z in -3..=3 {
            let height = ((x * x + z * z) as f32).sqrt() * 0.3;
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.8, height + 0.5, 0.8))),
                MeshMaterial3d(materials.add(Color::srgb(
                    0.5 + x as f32 * 0.07,
                    0.3 + height * 0.2,
                    0.5 + z as f32 * 0.07,
                ))),
                Transform::from_xyz(x as f32 * 2.0, (height + 0.5) / 2.0, z as f32 * 2.0),
            ));
        }
    }

    // Central sphere as a reference point
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 2.0, 0.0),
    ));
}

/// Toggle between Pointer and Navigator input modes
fn toggle_input_mode(keys: Res<ButtonInput<KeyCode>>, mut input_mode: ResMut<InputMode>) {
    // Tab key toggles between modes
    if keys.just_pressed(KeyCode::Tab) {
        *input_mode = match *input_mode {
            InputMode::Pointer => {
                info!("Switched to Navigator mode - Mouse grabbed for camera control");
                InputMode::Navigator
            }
            InputMode::Navigator => {
                info!("Switched to Pointer mode - Mouse visible and free");
                InputMode::Pointer
            }
        };
    }
}

/// Update cursor visibility and grab mode based on input mode
fn update_cursor_state(input_mode: Res<InputMode>, mut windows: Query<&mut Window>) {
    if !input_mode.is_changed() {
        return;
    }

    for mut window in windows.iter_mut() {
        match *input_mode {
            InputMode::Pointer => {
                window.cursor_options.visible = true;
                window.cursor_options.grab_mode = CursorGrabMode::None;
            }
            InputMode::Navigator => {
                window.cursor_options.visible = false;
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
            }
        }
    }
}

/// Custom camera movement system with Q/E vertical controls and Alt speed modifier
fn camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    input_mode: Res<InputMode>,
    mut query: Query<(&mut Transform, &FlyCam)>,
) {
    // Only move camera in Navigator mode
    if *input_mode != InputMode::Navigator {
        return;
    }

    for (mut transform, fly_cam) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let forward = transform.forward();
        let right = transform.right();

        // Speed multiplier: 5x when Alt is held
        let speed_multiplier = if keys.pressed(KeyCode::AltLeft) || keys.pressed(KeyCode::AltRight)
        {
            5.0
        } else {
            1.0
        };

        // WASD for horizontal movement
        if keys.pressed(KeyCode::KeyW) {
            velocity += *forward;
        }
        if keys.pressed(KeyCode::KeyS) {
            velocity -= *forward;
        }
        if keys.pressed(KeyCode::KeyA) {
            velocity -= *right;
        }
        if keys.pressed(KeyCode::KeyD) {
            velocity += *right;
        }

        // Q for down, E for up (vertical movement)
        if keys.pressed(KeyCode::KeyQ) {
            velocity -= Vec3::Y;
        }
        if keys.pressed(KeyCode::KeyE) {
            velocity += Vec3::Y;
        }

        // Apply movement with speed modifiers
        velocity = velocity.normalize_or_zero()
            * fly_cam.base_speed
            * speed_multiplier
            * time.delta_secs();

        transform.translation += velocity;
    }
}

/// Enable/disable camera controller based on input mode
fn update_camera_controller(
    input_mode: Res<InputMode>,
    mut query: Query<&mut FpsCameraController>,
) {
    if !input_mode.is_changed() {
        return;
    }

    for mut controller in query.iter_mut() {
        controller.enabled = *input_mode == InputMode::Navigator;
    }
}
