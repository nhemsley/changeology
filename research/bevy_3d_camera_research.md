# Bevy 3D Camera & Visualization Research

## Overview

This document explores camera systems and 3D visualization options for building a Bevy application with fly-mode camera controls. The goal is to understand different camera approaches and primitive rendering options for visualizing hierarchical data structures.

---

## Bevy Camera Fundamentals

### Camera3d Component

Bevy uses an ECS (Entity Component System) architecture. A basic 3D camera requires:

```rust
use bevy::prelude::*;

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
```

### Core Camera Components

| Component | Purpose |
|-----------|---------|
| `Camera3d` | Marks entity as a 3D camera |
| `Transform` | Position, rotation, scale |
| `Projection` | Perspective or Orthographic |
| `Camera` | General camera settings (order, clear color, etc.) |

---

## Fly-Mode Camera Implementation

### Approach 1: Custom Fly Camera

A fly camera allows free movement in 3D space with WASD + mouse look.

```rust
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

#[derive(Component)]
pub struct FlyCam {
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for FlyCam {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.1,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

fn fly_camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &FlyCam)>,
) {
    for (mut transform, fly_cam) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let forward = transform.forward();
        let right = transform.right();

        // WASD movement
        if keys.pressed(KeyCode::KeyW) { velocity += *forward; }
        if keys.pressed(KeyCode::KeyS) { velocity -= *forward; }
        if keys.pressed(KeyCode::KeyA) { velocity -= *right; }
        if keys.pressed(KeyCode::KeyD) { velocity += *right; }
        if keys.pressed(KeyCode::Space) { velocity += Vec3::Y; }
        if keys.pressed(KeyCode::ShiftLeft) { velocity -= Vec3::Y; }

        velocity = velocity.normalize_or_zero() * fly_cam.speed * time.delta_secs();
        transform.translation += velocity;
    }
}

fn fly_camera_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCam)>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    for (mut transform, mut fly_cam) in query.iter_mut() {
        fly_cam.yaw -= delta.x * fly_cam.sensitivity;
        fly_cam.pitch -= delta.y * fly_cam.sensitivity;
        fly_cam.pitch = fly_cam.pitch.clamp(-89.0, 89.0);

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            fly_cam.yaw.to_radians(),
            fly_cam.pitch.to_radians(),
            0.0,
        );
    }
}
```

### Approach 2: bevy_flycam Crate

Third-party crate for quick fly camera setup:

```toml
[dependencies]
bevy_flycam = "0.14"
```

```rust
use bevy::prelude::*;
use bevy_flycam::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PlayerPlugin)
        .run();
}
```

**Features:**
- WASD movement
- Mouse look
- Configurable speed and sensitivity
- Mouse grab handling

### Approach 3: bevy_panorbit_camera

Orbit camera with fly-mode capabilities:

```toml
[dependencies]
bevy_panorbit_camera = "0.19"
```

```rust
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5.0, 5.0, 5.0),
        PanOrbitCamera::default(),
    ));
}
```

---

## Camera Comparison

| Feature | Custom FlyCam | bevy_flycam | bevy_panorbit_camera |
|---------|---------------|-------------|----------------------|
| Free movement | ✅ | ✅ | ✅ |
| Orbit mode | ❌ | ❌ | ✅ |
| Pan mode | ❌ | ❌ | ✅ |
| Customizable | ✅ Full control | ⚠️ Limited | ✅ Many options |
| Maintenance | Self | Community | Active |
| Learning curve | Medium | Low | Low |
| Dependencies | None | Minimal | Minimal |

### Recommendation

For a file tree visualization tool:
1. **Start with `bevy_panorbit_camera`** - offers orbit + pan + zoom, good for examining 3D structures
2. **Add custom fly mode** if needed for navigation through large spaces
3. **Consider hybrid approach** - orbit for close inspection, fly for traversal

---

## 3D Primitive Options

### Plane

**Use Case:** Ground plane, flat surfaces, billboards

```rust
fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(10.0, 10.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
```

**Plane Variants:**

| Type | Description |
|------|-------------|
| `Plane3d::default()` | XZ plane (horizontal) |
| `Plane3d::new(Vec3::X, Vec3::Y)` | Custom orientation |

**Plane Properties:**
- Single-sided by default
- UV coordinates for texturing
- Subdivision for terrain/deformation

**Mesh Builder Options:**
```rust
// Simple plane
Plane3d::default().mesh().size(width, height)

// Subdivided plane (for terrain, deformation)
Plane3d::default().mesh().size(10.0, 10.0).subdivisions(10)
```

---

### Cube (Cuboid)

**Use Case:** Boxes, nodes, buildings, file representations

```rust
fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}
```

**Cuboid Constructors:**

| Constructor | Description |
|-------------|-------------|
| `Cuboid::new(x, y, z)` | Half-extents (total size = 2x) |
| `Cuboid::from_size(Vec3)` | Full dimensions |
| `Cuboid::from_length(f32)` | Uniform cube |

**Note:** `Cuboid::new(1.0, 1.0, 1.0)` creates a 2x2x2 cube (parameters are half-extents).

---

### Sphere

**Use Case:** Nodes, particles, points of interest

```rust
fn spawn_sphere(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(5).unwrap())),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.8))),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
}
```

**Sphere Mesh Types:**

| Type | Description | Performance |
|------|-------------|-------------|
| `.ico(subdivisions)` | Icosphere - uniform triangles | Good |
| `.uv(sectors, stacks)` | UV sphere - lat/long mapping | Moderate |

---

### Cylinder

**Use Case:** Connections, edges, tree branches

```rust
fn spawn_cylinder(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.2, 2.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.5, 0.5, 0.5))),
        Transform::from_xyz(2.0, 1.0, 0.0),
    ));
}
```

**Cylinder Properties:**
- `Cylinder::new(radius, height)`
- Resolution can be customized via mesh builder

---

### Capsule

**Use Case:** Rounded connections, softer node representations

```rust
fn spawn_capsule(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::new(0.3, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.2))),
        Transform::from_xyz(-2.0, 1.0, 0.0),
    ));
}
```

---

### Torus

**Use Case:** Rings, orbital paths, decorative elements

```rust
fn spawn_torus(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.8, 0.2))),
        Transform::from_xyz(0.0, 2.0, -3.0),
    ));
}
```

---

### Cone

**Use Case:** Cone trees, directional indicators, arrows

```rust
fn spawn_cone(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cone::new(0.5, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.5, 0.2))),
        Transform::from_xyz(3.0, 0.5, 0.0),
    ));
}
```

---

## Primitive Comparison for File Tree Visualization

| Primitive | Use Case | Pros | Cons |
|-----------|----------|------|------|
| **Cube** | File/folder nodes | Clear bounds, stackable | Can look blocky |
| **Sphere** | Abstract nodes | Smooth, no orientation | Harder to pack |
| **Plane** | Labels, icons | Billboarding friendly | Single-sided |
| **Cylinder** | Connections/edges | Natural for lines | Orientation math |
| **Cone** | Cone tree nodes | Hierarchical indication | Limited use |
| **Capsule** | Rounded nodes | Softer look | Less common |

---

## Rendering Considerations

### Materials

```rust
// Basic colored material
StandardMaterial {
    base_color: Color::srgb(0.8, 0.2, 0.2),
    ..default()
}

// Emissive (glowing)
StandardMaterial {
    base_color: Color::srgb(0.2, 0.2, 0.8),
    emissive: LinearRgba::rgb(0.5, 0.5, 1.0),
    ..default()
}

// Transparent
StandardMaterial {
    base_color: Color::srgba(0.8, 0.8, 0.8, 0.5),
    alpha_mode: AlphaMode::Blend,
    ..default()
}

// Unlit (no shading)
StandardMaterial {
    unlit: true,
    base_color: Color::srgb(1.0, 1.0, 1.0),
    ..default()
}
```

### Lighting

```rust
// Directional light (sun)
commands.spawn((
    DirectionalLight {
        illuminance: 10000.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
));

// Point light
commands.spawn((
    PointLight {
        intensity: 1500.0,
        shadows_enabled: true,
        ..default()
    },
    Transform::from_xyz(4.0, 8.0, 4.0),
));

// Ambient light
commands.insert_resource(AmbientLight {
    color: Color::WHITE,
    brightness: 200.0,
});
```

---

## Complete Example: Fly Camera with Primitives

```rust
use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (fly_camera_movement, fly_camera_look))
        .run();
}

#[derive(Component)]
struct FlyCam {
    speed: f32,
    sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for FlyCam {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.1,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam::default(),
    ));

    // Light
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.2, 0.0)),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
    ));

    // Cubes
    for x in -2..=2 {
        for z in -2..=2 {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(0.4, 0.4, 0.4))),
                MeshMaterial3d(materials.add(Color::srgb(
                    0.5 + x as f32 * 0.1,
                    0.3,
                    0.5 + z as f32 * 0.1,
                ))),
                Transform::from_xyz(x as f32 * 2.0, 0.4, z as f32 * 2.0),
            ));
        }
    }
}

fn fly_camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &FlyCam)>,
) {
    for (mut transform, fly_cam) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let forward = transform.forward();
        let right = transform.right();

        if keys.pressed(KeyCode::KeyW) { velocity += *forward; }
        if keys.pressed(KeyCode::KeyS) { velocity -= *forward; }
        if keys.pressed(KeyCode::KeyA) { velocity -= *right; }
        if keys.pressed(KeyCode::KeyD) { velocity += *right; }
        if keys.pressed(KeyCode::Space) { velocity += Vec3::Y; }
        if keys.pressed(KeyCode::ShiftLeft) { velocity -= Vec3::Y; }

        velocity = velocity.normalize_or_zero() * fly_cam.speed * time.delta_secs();
        transform.translation += velocity;
    }
}

fn fly_camera_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut FlyCam)>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    for (mut transform, mut fly_cam) in query.iter_mut() {
        fly_cam.yaw -= delta.x * fly_cam.sensitivity;
        fly_cam.pitch -= delta.y * fly_cam.sensitivity;
        fly_cam.pitch = fly_cam.pitch.clamp(-89.0, 89.0);

        transform.rotation = Quat::from_euler(
            EulerRot::YXZ,
            fly_cam.yaw.to_radians(),
            fly_cam.pitch.to_radians(),
            0.0,
        );
    }
}
```

---

## Performance Considerations

### Instancing for Many Objects

For large numbers of similar objects (e.g., many file nodes):

```rust
// Bevy automatically batches entities with same mesh + material
// Just ensure you reuse mesh and material handles

let cube_mesh = meshes.add(Cuboid::new(0.5, 0.5, 0.5));
let cube_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

for i in 0..1000 {
    commands.spawn((
        Mesh3d(cube_mesh.clone()),
        MeshMaterial3d(cube_material.clone()),
        Transform::from_xyz(/* position */),
    ));
}
```

### LOD (Level of Detail)

For large scenes, consider:
- Reducing mesh complexity at distance
- Culling distant objects
- Using billboards for far objects

---

## Recommendations for File Tree Visualization

### Primary Approach

1. **Use cubes for files/folders** - Clear, stackable, familiar
2. **Use cylinders for connections** - Natural edge representation
3. **Use `bevy_panorbit_camera`** - Orbit + pan + zoom for exploration
4. **Add fly mode toggle** - For navigating large structures

### Visual Hierarchy

```
Root (large cube)
├── Child (medium cube) ─── cylinder connection
│   ├── File (small cube)
│   └── File (small cube)
└── Child (medium cube)
    └── File (small cube)
```

### Color Coding

- Folders: Blue/Purple tones
- Files: Green/Orange based on type
- Selected: Yellow highlight
- Modified (git): Orange/Red

---

## References

- [Bevy Book](https://bevyengine.org/learn/book/introduction/)
- [Bevy Cheatbook - Cameras](https://bevy-cheatbook.github.io/3d/camera.html)
- [bevy_flycam](https://github.com/sburris0/bevy_flycam)
- [bevy_panorbit_camera](https://github.com/Plonq/bevy_panorbit_camera)
- [Bevy Examples - 3D](https://github.com/bevyengine/bevy/tree/main/examples/3d)