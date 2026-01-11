use gpui::*;

use gpui_component::{
    button::{Button, ButtonVariants},
    h_flex,
    list::ListItem,
    menu::{DropdownMenu, PopupMenu},
    resizable::{h_resizable, resizable_panel},
    scroll::Scrollbar,
    tree::{tree, TreeState},
    v_flex, ActiveTheme, Icon, IconName, Root, Sizable, TitleBar,
};

use crate::diff_canvas::{DiffCanvasView, FileDiff};
use crate::menu::*;
use crate::panels::file_tree;
use crate::sidebar;
use buffer_diff::DiffConfig;
use git::{Commit, Repository};

pub struct ChangeologyApp {
    /// The git repository (if opened)
    repository: Option<Repository>,

    /// Current working directory path
    cwd: Option<String>,

    /// Whether the sidebar is collapsed
    #[allow(dead_code)]
    sidebar_collapsed: bool,

    /// Dirty files (unstaged changes)
    dirty_files: Vec<git::StatusEntry>,

    /// Staged files (ready to commit)
    staged_files: Vec<git::StatusEntry>,

    /// Selected dirty file index
    selected_dirty_file: Option<usize>,

    /// Selected staged file index
    selected_staged_file: Option<usize>,

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

    /// The diff canvas view for displaying diffs
    diff_canvas: Entity<DiffCanvasView>,

    /// Scroll handle for history list
    history_scroll_handle: ScrollHandle,
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

        // Create the diff canvas view
        let diff_canvas = cx.new(|cx| DiffCanvasView::new(window, cx));

        let mut app = Self {
            repository,
            cwd,
            sidebar_collapsed: false,
            dirty_files: Vec::new(),
            staged_files: Vec::new(),
            selected_dirty_file: None,
            selected_staged_file: None,
            file_tree_state,
            selected_file: None,
            commits: Vec::new(),
            selected_commit: None,
            commit_diffs: Vec::new(),
            diff_canvas,
            history_scroll_handle: ScrollHandle::new(),
        };

        // Load initial data
        app.refresh(window, cx);

        app
    }

    pub fn refresh(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(repo) = &self.repository {
            // Load dirty (unstaged) files
            if let Ok(dirty) = repo.unstaged_changes() {
                self.dirty_files = dirty;
            }

            // Load staged files
            if let Ok(staged) = repo.staged_changes() {
                self.staged_files = staged;
            }

            // Load file status for file tree
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

    fn load_commit_diffs(&mut self, commit_index: usize, cx: &mut Context<Self>) {
        self.commit_diffs.clear();

        let mut commit_info: Option<(String, String)> = None;

        if let Some(repo) = &self.repository {
            if let Some(commit) = self.commits.get(commit_index) {
                commit_info = Some((commit.short_id.clone(), commit.message.clone()));

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

        // Update the canvas view with the new diffs
        let diffs = self.commit_diffs.clone();
        self.diff_canvas.update(cx, |canvas, cx| {
            canvas.set_diffs(diffs, commit_info, cx);
        });
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

    fn render_dirty_files(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(sidebar::render_section_header(
                "CHANGES",
                self.dirty_files.len(),
                cx,
            ))
            .child(
                // Content
                div()
                    .flex_auto()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .children(self.dirty_files.iter().enumerate().map(|(i, entry)| {
                                let is_selected = self.selected_dirty_file == Some(i);
                                sidebar::render_file_entry(
                                    format!("dirty-{}", i),
                                    entry,
                                    is_selected,
                                    cx,
                                )
                                .on_click(cx.listener(
                                    move |this, _: &gpui::ClickEvent, _window, cx| {
                                        this.selected_dirty_file = Some(i);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element()
                            })),
                    ),
            )
    }

    fn render_staging_area(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(sidebar::render_section_header(
                "STAGED",
                self.staged_files.len(),
                cx,
            ))
            .child(
                // Content
                div()
                    .flex_auto()
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w_full()
                            .children(self.staged_files.iter().enumerate().map(|(i, entry)| {
                                let is_selected = self.selected_staged_file == Some(i);
                                sidebar::render_file_entry(
                                    format!("staged-{}", i),
                                    entry,
                                    is_selected,
                                    cx,
                                )
                                .on_click(cx.listener(
                                    move |this, _: &gpui::ClickEvent, _window, cx| {
                                        this.selected_staged_file = Some(i);
                                        cx.notify();
                                    },
                                ))
                                .into_any_element()
                            })),
                    ),
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
        v_flex()
            .size_full()
            .child(sidebar::render_section_header(
                "HISTORY",
                self.commits.len(),
                cx,
            ))
            .child(
                // Content - scrollable area
                div()
                    .id("history-scroll-area")
                    .flex_1()
                    .overflow_y_scroll()
                    .track_scroll(&self.history_scroll_handle)
                    .child(if self.commits.is_empty() {
                        sidebar::render_empty_state("No commits", cx).into_any_element()
                    } else {
                        v_flex()
                            .w_full()
                            .children(self.commits.iter().enumerate().map(|(i, commit)| {
                                let is_selected = self.selected_commit == Some(i);
                                sidebar::render_commit_entry(i, commit, is_selected, cx)
                                    .on_click(cx.listener(
                                        move |this, _: &gpui::ClickEvent, _window, cx| {
                                            this.selected_commit = Some(i);
                                            this.load_commit_diffs(i, cx);
                                            cx.notify();
                                        },
                                    ))
                                    .into_any_element()
                            }))
                            .into_any_element()
                    })
                    .child(Scrollbar::vertical(&self.history_scroll_handle)),
            )
    }

    fn render_sidebar(&self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().sidebar)
            .text_color(cx.theme().sidebar_foreground)
            .child(
                // Dirty files section - top 1/3
                div()
                    .flex_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(self.render_dirty_files(window, cx)),
            )
            .child(
                // Staging section - middle 1/3
                div()
                    .flex_1()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(self.render_staging_area(window, cx)),
            )
            .child(
                // History section - bottom 1/3
                div().flex_1().child(self.render_history_panel(window, cx)),
            )
    }

    fn render_content_area(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // Use the diff canvas view for displaying diffs
        // Wrap in a size_full div to ensure proper sizing
        div().size_full().child(self.diff_canvas.clone())
    }
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
