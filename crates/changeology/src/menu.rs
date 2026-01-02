use gpui::*;

// Define actions using the actions! macro
actions!(
    changeology,
    [
        OpenRepository,
        CloseRepository,
        Refresh,
        Quit,
        ToggleSidebar,
        ShowHistory,
        ShowFileTree,
    ]
);

pub fn register_actions(cx: &mut App) {
    // Register global action handlers
    cx.on_action(|_: &Quit, cx| {
        cx.quit();
    });
}
