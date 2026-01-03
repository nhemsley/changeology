# Tree Viewer

A 3D visualization tool for exploring hierarchical file structures using Bevy engine.

## Features

- **Dual Input Modes**: Switch between Pointer and Navigator modes
- **Fly Camera Controls**: Free movement through 3D space using smooth-bevy-cameras
- **3D Primitives**: Visual representation using cubes, spheres, and other primitives
- **Interactive Navigation**: WASD movement + mouse look in Navigator mode

## Input Modes

### Pointer Mode (Default)
- Mouse is visible and free
- Can interact with UI elements
- Camera movement disabled
- Use for selecting, clicking, and UI interaction

### Navigator Mode
- Mouse is grabbed for camera control (invisible)
- Full 3D camera navigation enabled
- Use for flying through and exploring the 3D space

**Toggle**: Press `Tab` to switch between modes

## Controls

### Navigator Mode Controls

**Movement:**
- `W` - Move forward
- `S` - Move backward
- `A` - Strafe left
- `D` - Strafe right
- `E` - Move up
- `Q` - Move down

**Speed:**
- `Alt` (hold) - 5x speed multiplier

**Look:**
- Mouse movement - Look around (in Navigator mode only)

### Global Controls

- `Tab` - Toggle between Pointer and Navigator modes

## Running

```bash
cargo run --package tree-viewer
```

Or from the crates/tree-viewer directory:

```bash
cargo run
```

## Architecture

### Core Concepts

**InputMode Resource**: Central UI concept that determines interaction paradigm
- `InputMode::Pointer` - UI interaction mode
- `InputMode::Navigator` - 3D navigation mode

This dual-mode system allows seamless switching between:
1. Traditional UI interaction (menus, buttons, selections)
2. Immersive 3D exploration (flying through data structures)

### Components

- `FlyCam` - Camera movement component with configurable speed
- `InputMode` - Global resource controlling input interpretation
- `Transform` - Camera position and rotation

## Dependencies

- **bevy**: Game engine for 3D rendering
- **smooth-bevy-cameras**: FPS camera controller with smooth interpolation

## Future Development

- [ ] Load actual file tree data
- [ ] Represent folders as larger cubes
- [ ] Represent files as smaller cubes
- [ ] Color coding by file type
- [ ] Git status visualization
- [ ] Interactive selection (in Pointer mode)
- [ ] Cone tree layout
- [ ] Reconfigurable disc tree layout
- [ ] UI overlays for file information
- [ ] Keyboard shortcuts for common actions
- [ ] Configurable speed and sensitivity settings