# Tree Viewer Input System Architecture

## Overview

The tree-viewer input system is built around a central **dual-mode paradigm** that separates UI interaction from 3D navigation. This design allows users to seamlessly switch between traditional pointer-based UI interaction and immersive 3D exploration.

## Core Concept: InputMode

The `InputMode` enum is a **central UI concept** that fundamentally changes how the application interprets user input.

```rust
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Pointer mode: Mouse is visible and free, can interact with UI
    Pointer,
    /// Navigator mode: Mouse is grabbed for camera control, no cursor visible
    Navigator,
}
```

### Design Philosophy

This dual-mode system recognizes two distinct interaction paradigms:

1. **Pointer Mode** - Traditional desktop interaction
   - Mouse cursor is visible and controllable
   - Used for clicking UI elements, selecting items, reading information
   - Camera movement is disabled
   - Standard desktop application behavior

2. **Navigator Mode** - Immersive 3D exploration
   - Mouse is grabbed and controls camera rotation
   - Cursor becomes invisible
   - Full 3D movement enabled (WASD + Q/E + Alt modifier)
   - Game-like navigation experience

## Implementation Details

### Resource Management

`InputMode` is stored as a Bevy `Resource`, making it globally accessible throughout the application:

```rust
.init_resource::<InputMode>()
```

Default mode is `Pointer` to maintain expected desktop application behavior on startup.

### System Architecture

Four main systems manage the input mode:

1. **`toggle_input_mode`** - Handles mode switching
   - Listens for Tab key press
   - Toggles between Pointer and Navigator
   - Logs mode changes for debugging

2. **`update_cursor_state`** - Manages cursor appearance
   - Reacts to InputMode changes
   - Sets cursor visibility
   - Controls cursor grab mode (None vs Locked)
   - Only runs when InputMode changes (efficient)

3. **`update_camera_controller`** - Enables/disables camera rotation
   - Reacts to InputMode changes
   - Disables FpsCameraController in Pointer mode
   - Enables FpsCameraController in Navigator mode
   - Prevents camera rotation when interacting with UI
   - Only runs when InputMode changes (efficient)

4. **`camera_movement`** - Handles navigation
   - Only active in Navigator mode
   - Implements custom WASD + Q/E movement
   - Applies Alt speed modifier (5x)
   - Respects time delta for frame-rate independence

### Controls Mapping

#### Navigator Mode Controls

| Input | Action | Notes |
|-------|--------|-------|
| `W` | Move forward | Based on camera forward vector |
| `S` | Move backward | Opposite of forward |
| `A` | Strafe left | Based on camera right vector |
| `D` | Strafe right | Based on camera right vector |
| `E` | Move up | World-space up (Y+) |
| `Q` | Move down | World-space down (Y-) |
| `Alt` (hold) | Speed boost | 5x multiplier |
| Mouse movement | Camera rotation | Handled by smooth-bevy-cameras |

#### Global Controls

| Input | Action | Available In |
|-------|--------|--------------|
| `Tab` | Toggle input mode | Both modes |

### Speed System

Movement speed is calculated as:

```
velocity = direction * base_speed * speed_multiplier * delta_time
```

Where:
- `base_speed` = 5.0 units/second (configurable per camera)
- `speed_multiplier` = 5.0 when Alt pressed, 1.0 otherwise
- `delta_time` = Frame time delta for smooth movement

This provides:
- Normal speed: 5 units/second
- Boosted speed: 25 units/second

### Component Design

```rust
#[derive(Component)]
struct FlyCam {
    base_speed: f32,
}
```

The `FlyCam` component marks cameras that support navigation and stores their movement parameters. This allows for:
- Multiple cameras with different speeds
- Per-camera configuration
- Easy extension with additional parameters

## Integration with smooth-bevy-cameras

The system works alongside `smooth-bevy-cameras` FPS controller:

- `smooth-bevy-cameras` handles mouse look (rotation)
- Our custom system handles translation (position)
- `FpsCameraController.enabled` is toggled based on InputMode
- Controller is disabled in Pointer mode (no rotation)
- Controller is enabled in Navigator mode (full rotation)
- Smooth interpolation provided by the library

## Future Extensions

### Planned Features

1. **Configurable Controls**
   - Remappable keys
   - Adjustable speeds
   - Custom speed multipliers

2. **UI Integration**
   - On-screen mode indicator
   - Speed display
   - Control hints overlay

3. **Additional Modes**
   - `Orbit` mode for object inspection
   - `Cinematic` mode for smooth camera paths
   - `Selection` mode for multi-object selection

4. **Smooth Transitions**
   - Animated camera transitions between modes
   - Smooth cursor appearance/disappearance
   - Visual feedback for mode changes

### Extension Points

The architecture is designed for extensibility:

```rust
pub enum InputMode {
    Pointer,
    Navigator,
    // Future modes:
    // Orbit { target: Entity },
    // Cinematic { path: CameraPath },
    // Selection { mode: SelectionMode },
}
```

Each mode can carry its own state and parameters.

## Best Practices

### When to Use Each Mode

**Use Pointer Mode for:**
- File selection and manipulation
- Reading information panels
- Adjusting settings
- Interacting with menus and dialogs
- Precision clicking

**Use Navigator Mode for:**
- Exploring large 3D structures
- Quickly moving through space
- Getting overview of hierarchies
- Cinematic viewing
- Free-form exploration

### User Experience Guidelines

1. **Always start in Pointer mode** - Matches user expectations
2. **Make mode obvious** - Visual indicators of current mode
3. **Smooth transitions** - No jarring changes
4. **Escape hatch** - Tab always works to switch modes
5. **Preserve context** - Camera position maintained across mode switches

## Technical Considerations

### Performance

- Mode checking uses `InputMode::is_changed()` to avoid unnecessary work
- Movement system early-returns in Pointer mode
- No per-frame cursor updates unless mode changes

### Platform Compatibility

- Cursor grab mode uses Bevy's `CursorGrabMode`
- `Locked` mode may behave differently on some platforms
- Fallback to `Confined` if needed (platform-specific)

### Accessibility

- Keyboard-only navigation fully supported in Navigator mode
- Clear visual feedback for mode state
- Alternative control schemes can be added per mode

## Testing Strategy

### Manual Testing

- [ ] Tab toggles modes correctly
- [ ] Cursor visibility matches mode
- [ ] WASD works only in Navigator mode
- [ ] Q/E vertical movement works
- [ ] Alt speed boost functions
- [ ] Mouse look only in Navigator mode
- [ ] UI interaction only in Pointer mode

### Future Automated Tests

- Unit tests for InputMode state transitions
- Integration tests for cursor state management
- Input handling tests for each mode
- Performance tests for mode switching overhead

## References

- Bevy Input Handling: https://bevyengine.org/learn/book/input/
- smooth-bevy-cameras: https://github.com/bonsairobo/smooth-bevy-cameras
- Game Camera Systems: Design patterns for dual-mode interaction