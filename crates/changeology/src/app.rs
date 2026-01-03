use gpui::*;
use gpui_component::scroll::ScrollableElement;

use gpui_component::{
    button::{Button, ButtonVariants},
    clipboard::Clipboard,
    h_flex,
    list::ListItem,
    menu::{DropdownMenu, PopupMenu},
    resizable::{h_resizable, resizable_panel},
    tab::{Tab, TabBar},
    tree::{tree, TreeState},
    v_flex, ActiveTheme, Icon, IconName, Root, Sizable, TitleBar,
};

use crate::menu::*;
use crate::panels::file_tree;
use buffer_diff::{BufferDiff, DiffConfig, DiffLineType};
use git::{Commit, Repository};

/// Diff data for a single file in a commit
#[derive(Clone)]
struct FileDiff {
    path: String,
    old_content: String,
    new_content: String,
    buffer_diff: BufferDiff,
}

/// Which panel is currently shown in the left sidebar
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum ActivePanel {
    History,
    #[default]
    FileTree,
}

pub struct ChangeologyApp {
    /// The git repository (if opened)
    repository: Option<Repository>,

    /// Current working directory path
    cwd: Option<String>,

    /// Whether the sidebar is collapsed
    #[allow(dead_code)]
    sidebar_collapsed: bool,

    /// Which panel is active
    active_panel: ActivePanel,

    /// File tree state
    file_tree_state: Entity<TreeState>,

    /// Selected file path
    #[allow(dead_code)]
    selected_file: Option<String>,

    /// Commit history
    commits: Vec<Commit>,

    /// Selected commit index
    selected_commit: Option<usize>,

    /// Diffs for the selected commit
    commit_diffs: Vec<FileDiff>,
}

impl ChangeologyApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        // Try to open repository at current directory
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string());
        let repository = cwd.as_ref().and_then(|path| Repository::open(path).ok());

        // Create tree state
        let file_tree_state = cx.new(|cx| TreeState::new(cx));

        let mut app = Self {
            repository,
            cwd,
            sidebar_collapsed: false,
            active_panel: ActivePanel::FileTree,
            file_tree_state,
            selected_file: None,
            commits: Vec::new(),
            selected_commit: None,
            commit_diffs: Vec::new(),
        };

        // Load initial data
        app.refresh(window, cx);

        app
    }

    pub fn refresh(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(repo) = &self.repository {
            // Load file status
            if let Ok(status) = repo.status() {
                let items = file_tree::build_nested_tree(&status);
                self.file_tree_state.update(cx, |state, cx| {
                    state.set_items(items, cx);
                });
            }

            // Load commit history (limit to 100 most recent commits)
            if let Ok(commits) = repo.log(Some(100)) {
                self.commits = commits;
            }
        }
        cx.notify();
    }

    fn load_commit_diffs(&mut self, commit_index: usize, _cx: &mut Context<Self>) {
        self.commit_diffs.clear();

        if let Some(repo) = &self.repository {
            if let Some(commit) = self.commits.get(commit_index) {
                // Get list of files changed in this commit
                if let Ok(files) = repo.get_commit_files(&commit.id) {
                    for file_path in files {
                        // Get the old content (parent commit) and new content (this commit)
                        let old_content = if !commit.parent_ids.is_empty() {
                            repo.get_content_at_revision(&commit.parent_ids[0], &file_path)
                                .ok()
                                .flatten()
                                .unwrap_or_default()
                        } else {
                            String::new() // First commit, no parent
                        };

                        let new_content = repo
                            .get_content_at_revision(&commit.id, &file_path)
                            .ok()
                            .flatten()
                            .unwrap_or_default();

                        // Compute the BufferDiff
                        let config = DiffConfig::default();
                        if let Ok(buffer_diff) = config.diff(&old_content, &new_content) {
                            self.commit_diffs.push(FileDiff {
                                path: file_path,
                                old_content,
                                new_content,
                                buffer_diff,
                            });
                        }
                    }
                }
            }
        }
    }

    fn render_title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .child(
                h_flex()
                    .gap_1()
                    .child(
                        Button::new("file-menu")
                            .ghost()
                            .label("File")
                            .dropdown_menu(
                                |menu: PopupMenu, _: &mut Window, _: &mut Context<PopupMenu>| {
                                    menu.menu("Open Repository...", Box::new(OpenRepository))
                                        .menu("Close Repository", Box::new(CloseRepository))
                                        .separator()
                                        .menu("Refresh", Box::new(Refresh))
                                        .separator()
                                        .menu("Quit", Box::new(Quit))
                                },
                            ),
                    )
                    .child(
                        Button::new("view-menu")
                            .ghost()
                            .label("View")
                            .dropdown_menu(
                                |menu: PopupMenu, _: &mut Window, _: &mut Context<PopupMenu>| {
                                    menu.menu("Toggle Sidebar", Box::new(ToggleSidebar))
                                        .separator()
                                        .menu("History", Box::new(ShowHistory))
                                        .menu("File Tree", Box::new(ShowFileTree))
                                },
                            ),
                    ),
            )
            .child(
                // Spacer + repo name centered
                h_flex()
                    .flex_1()
                    .justify_center()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(
                        self.cwd
                            .as_ref()
                            .and_then(|p| std::path::Path::new(p).file_name())
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "No Repository".to_string()),
                    ),
            )
    }

    fn render_panel_tabs(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected = match self.active_panel {
            ActivePanel::History => 0,
            ActivePanel::FileTree => 1,
        };

        h_flex()
            .w_full()
            .px_2()
            .py_1()
            .gap_1()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                TabBar::new("panel-tabs")
                    .segmented()
                    .small()
                    .selected_index(selected)
                    .child(Tab::new().icon(IconName::Inbox)) // History - using Inbox icon
                    .child(Tab::new().icon(IconName::Folder)) // Files
                    .on_click(cx.listener(|this, index: &usize, _, cx| {
                        this.active_panel = match index {
                            0 => ActivePanel::History,
                            _ => ActivePanel::FileTree,
                        };
                        cx.notify();
                    })),
            )
    }

    fn render_file_tree(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let tree_state = self.file_tree_state.clone();

        div()
            .size_full()
            .p_2()
            .child(tree(&tree_state, |ix, entry, selected, _window, cx| {
                let item = entry.item();
                let icon = if entry.is_folder() {
                    if entry.is_expanded() {
                        IconName::FolderOpen
                    } else {
                        IconName::Folder
                    }
                } else {
                    IconName::File
                };

                ListItem::new(ix)
                    .selected(selected)
                    .py(px(2.))
                    .pl(px(16.) * entry.depth() as f32 + px(12.))
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                Icon::new(icon)
                                    .small()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(item.label.clone()),
                    )
            }))
    }

    fn render_history_panel(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if self.commits.is_empty() {
            return v_flex()
                .size_full()
                .p_4()
                .items_center()
                .justify_center()
                .text_color(cx.theme().muted_foreground)
                .child(
                    Icon::new(IconName::Inbox)
                        .size(px(32.))
                        .text_color(cx.theme().muted_foreground),
                )
                .child("No commits found")
                .child(
                    div()
                        .text_xs()
                        .mt_2()
                        .text_color(cx.theme().muted_foreground)
                        .child("Initialize a repository or make some commits"),
                )
                .into_any_element();
        }

        v_flex()
            .size_full()
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .children(self.commits.iter().enumerate().map(|(i, commit)| {
                                let is_selected = self.selected_commit == Some(i);

                                ListItem::new(i)
                                    .selected(is_selected)
                                    .on_click(cx.listener(
                                        move |this, _: &gpui::ClickEvent, _window, cx| {
                                            this.selected_commit = Some(i);
                                            this.load_commit_diffs(i, cx);
                                            cx.notify();
                                        },
                                    ))
                                    .child(
                                        v_flex()
                                            .w_full()
                                            .gap_1()
                                            .child(
                                                h_flex()
                                                    .w_full()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .text_sm()
                                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                                            .child(commit.message.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(commit.short_id.clone()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!(
                                                        "{} • {}",
                                                        commit.author_name,
                                                        format_timestamp(commit.time)
                                                    )),
                                            ),
                                    )
                            })),
                    ),
            )
            .into_any_element()
    }

    fn render_sidebar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .child(self.render_panel_tabs(window, cx))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .child(match self.active_panel {
                        ActivePanel::History => {
                            self.render_history_panel(window, cx).into_any_element()
                        }
                        ActivePanel::FileTree => {
                            self.render_file_tree(window, cx).into_any_element()
                        }
                    }),
            )
    }

    fn render_content_area(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        if self.commit_diffs.is_empty() {
            return div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(cx.theme().background)
                .text_color(cx.theme().muted_foreground)
                .child(
                    v_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Icon::new(IconName::File)
                                .size(px(48.))
                                .text_color(cx.theme().muted_foreground),
                        )
                        .child("Select a commit to view diffs")
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Click on a commit in the history panel"),
                        ),
                )
                .into_any_element();
        }

        // Render all diffs vertically in a scrollable container
        div()
            .size_full()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
            .child(
                v_flex().w_full().p_4().gap_4().children(
                    self.commit_diffs
                        .iter()
                        .map(|file_diff| self.render_file_diff(file_diff, cx)),
                ),
            )
            .into_any_element()
    }

    fn render_file_diff(&self, file_diff: &FileDiff, cx: &mut Context<Self>) -> impl IntoElement {
        let file_path = file_diff.path.clone();
        let commit_hash = self
            .selected_commit
            .and_then(|idx| self.commits.get(idx))
            .map(|commit| commit.id.clone())
            .unwrap_or_default();

        let copy_value = format!("{} {}", commit_hash, file_path);
        let clipboard_id = SharedString::from(format!("copy-file-{}", file_path));

        v_flex()
            .w_full()
            .border_1()
            .border_color(cx.theme().border)
            .rounded(px(8.))
            .overflow_hidden()
            .child(
                // File header
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .bg(cx.theme().muted)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .justify_between()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(Icon::new(IconName::File).small())
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::SEMIBOLD)
                                            .child(file_diff.path.clone()),
                                    ),
                            )
                            .child(Clipboard::new(clipboard_id).value(copy_value)),
                    ),
            )
            .child(
                // Diff content
                div()
                    .w_full()
                    .bg(cx.theme().background)
                    .child(self.render_hunks(file_diff, cx)),
            )
    }

    fn render_hunks(&self, file_diff: &FileDiff, cx: &mut Context<Self>) -> impl IntoElement {
        let hunks = file_diff.buffer_diff.hunks();
        let old_lines: Vec<&str> = file_diff.old_content.lines().collect();
        let new_lines: Vec<&str> = file_diff.new_content.lines().collect();

        v_flex().w_full().children(hunks.iter().flat_map(|hunk| {
            let mut lines = Vec::new();

            // Use line_types for all hunks to properly handle context lines
            // Track separate offsets for old and new lines
            let mut old_offset = 0;
            let mut new_offset = 0;

            for &line_type in hunk.line_types.iter() {
                match line_type {
                    DiffLineType::OldOnly => {
                        // Line was removed
                        let old_line_idx = hunk.old_range.start + old_offset;
                        if let Some(line_content) = old_lines.get(old_line_idx) {
                            lines.push(self.render_diff_line(
                                Some(old_line_idx + 1),
                                None,
                                line_content,
                                DiffLineKind::Removed,
                                cx,
                            ));
                        }
                        old_offset += 1;
                    }
                    DiffLineType::NewOnly => {
                        // Line was added
                        let new_line_idx = hunk.new_range.start + new_offset;
                        if let Some(line_content) = new_lines.get(new_line_idx) {
                            lines.push(self.render_diff_line(
                                None,
                                Some(new_line_idx + 1),
                                line_content,
                                DiffLineKind::Added,
                                cx,
                            ));
                        }
                        new_offset += 1;
                    }
                    DiffLineType::Both => {
                        // Line exists in both (context or unchanged)
                        let old_line_idx = hunk.old_range.start + old_offset;
                        let new_line_idx = hunk.new_range.start + new_offset;
                        if let Some(line_content) = old_lines.get(old_line_idx) {
                            lines.push(self.render_diff_line(
                                Some(old_line_idx + 1),
                                Some(new_line_idx + 1),
                                line_content,
                                DiffLineKind::Context,
                                cx,
                            ));
                        }
                        old_offset += 1;
                        new_offset += 1;
                    }
                }
            }

            lines
        }))
    }

    fn render_diff_line(
        &self,
        old_line_num: Option<usize>,
        new_line_num: Option<usize>,
        content: &str,
        kind: DiffLineKind,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (bg_color, sign, text_color) = match kind {
            DiffLineKind::Added => (cx.theme().green.opacity(0.1), "+", cx.theme().green),
            DiffLineKind::Removed => (cx.theme().red.opacity(0.1), "-", cx.theme().red),
            DiffLineKind::Context => (cx.theme().background, " ", cx.theme().foreground),
        };

        h_flex()
            .w_full()
            .bg(bg_color)
            .px_2()
            .py_0p5()
            .child(
                div()
                    .w(px(40.))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{:>4}",
                        old_line_num
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| " ".to_string())
                    )),
            )
            .child(
                div()
                    .w(px(40.))
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!(
                        "{:>4}",
                        new_line_num
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| " ".to_string())
                    )),
            )
            .child(
                div()
                    .flex_1()
                    .text_xs()
                    .font_family("monospace")
                    .text_color(text_color)
                    .child(format!("{} {}", sign, content)),
            )
    }
}

#[derive(Clone, Copy)]
enum DiffLineKind {
    Added,
    Removed,
    Context,
}

impl Render for ChangeologyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(self.render_title_bar(window, cx))
            .child(
                h_resizable("main-layout")
                    .child(
                        resizable_panel()
                            .size(px(260.))
                            .size_range(px(180.)..px(450.))
                            .child(self.render_sidebar(window, cx)),
                    )
                    .child(resizable_panel().child(self.render_content_area(window, cx))),
            )
            // Required: Render overlay layers for dialogs/notifications
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}

/// Format a Unix timestamp as a human-readable string
fn format_timestamp(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let diff = now - timestamp;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{} minutes ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else if diff < 604800 {
        format!("{} days ago", diff / 86400)
    } else if diff < 2592000 {
        format!("{} weeks ago", diff / 604800)
    } else if diff < 31536000 {
        format!("{} months ago", diff / 2592000)
    } else {
        format!("{} years ago", diff / 31536000)
    }
}
