use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use smooth_bevy_cameras::{
    controllers::fps::{ControlEvent, FpsCameraBundle, FpsCameraController, FpsCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(FpsCameraPlugin::new(false)) // Override default input system
        .init_resource::<InputMode>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_input_mode,
                update_cursor_state,
                update_camera_controller,
                custom_input_map,
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera with FPS controller from smooth-bevy-cameras
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(FpsCameraBundle::new(
            FpsCameraController {
                enabled: false, // Start disabled (Pointer mode)
                mouse_rotate_sensitivity: Vec2::splat(0.2),
                translate_sensitivity: 5.0,
                smoothing_weight: 0.9,
            },
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

/// Custom input map using smooth-bevy-cameras message system
/// Overrides smooth-bevy-cameras default_input_map
/// - Uses Q/E for vertical movement instead of Shift/Space
/// - Applies Alt modifier for 5x speed boost

pub fn custom_input_map(
    mut events: EventWriter<ControlEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    controllers: Query<&FpsCameraController>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().find(|c| c.enabled) {
        controller
    } else {
        return;
    };
    let FpsCameraController {
        translate_sensitivity,
        mouse_rotate_sensitivity,
        ..
    } = *controller;

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        cursor_delta += event.delta;
    }

    events.send(ControlEvent::Rotate(
        mouse_rotate_sensitivity * cursor_delta,
    ));

    for (key, dir) in [
        (KeyCode::KeyW, Vec3::Z),
        (KeyCode::KeyA, Vec3::X),
        (KeyCode::KeyS, -Vec3::Z),
        (KeyCode::KeyD, -Vec3::X),
        (KeyCode::KeyQ, -Vec3::Y),
        (KeyCode::KeyE, Vec3::Y),
    ]
    .iter()
    .cloned()
    {
        if keyboard.pressed(key) {
            events.send(ControlEvent::TranslateEye(translate_sensitivity * dir));
        }
    }
}
