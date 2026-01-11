use std::path::PathBuf;
use std::time::Duration;

use log::{debug, info, warn};

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
use crate::watcher::{DataSourceKind, RepoWatcher};
use buffer_diff::DiffConfig;
use git::{Commit, Repository};

pub struct ChangeologyApp {
    /// The git repository (if opened)
    repository: Option<Repository>,

    /// Current working directory path
    cwd: Option<PathBuf>,

    /// File system watcher for repository changes
    watcher: Option<RepoWatcher>,

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
        info!("ChangeologyApp::new - initializing application");

        // Try to open repository at current directory
        let cwd = std::env::current_dir().ok();
        info!("Working directory: {:?}", cwd);

        let repository = cwd.as_ref().and_then(|path| Repository::open(path).ok());
        info!("Repository opened: {}", repository.is_some());

        // Create file watcher for the repository
        let watcher = cwd.as_ref().and_then(|path| RepoWatcher::new(path).ok());
        info!("File watcher created: {}", watcher.is_some());

        // Create tree state
        let file_tree_state = cx.new(|cx| TreeState::new(cx));

        // Create the diff canvas view
        let diff_canvas = cx.new(|cx| DiffCanvasView::new(window, cx));

        let mut app = Self {
            repository,
            cwd,
            watcher,
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
        info!("Loading initial data...");
        app.refresh_source(DataSourceKind::All, cx);

        // Start polling for file system changes
        info!("Starting file system polling loop");
        cx.spawn(
            async move |this: WeakEntity<Self>, cx: &mut AsyncApp| loop {
                cx.background_executor()
                    .timer(Duration::from_millis(500))
                    .await;

                let should_refresh = this
                    .update(cx, |this: &mut Self, _cx| {
                        this.watcher
                            .as_ref()
                            .and_then(|w: &RepoWatcher| w.poll_changes())
                    })
                    .ok()
                    .flatten();

                if let Some(kind) = should_refresh {
                    info!("File system change detected, refreshing: {:?}", kind);
                    let _ = this.update(cx, |this: &mut Self, cx: &mut Context<Self>| {
                        this.refresh_source(kind, cx);
                    });
                }
            },
        )
        .detach();

        app
    }

    /// Refresh a specific data source
    pub fn refresh_source(&mut self, kind: DataSourceKind, cx: &mut Context<Self>) {
        debug!("refresh_source called with kind: {:?}", kind);

        if self.repository.is_none() {
            debug!("No repository, skipping refresh");
            return;
        }

        match kind {
            DataSourceKind::DirtyFiles => {
                self.refresh_dirty_files(cx);
            }
            DataSourceKind::StagedFiles => {
                self.refresh_staged_files();
            }
            DataSourceKind::History => {
                self.refresh_history();
            }
            DataSourceKind::All => {
                self.refresh_dirty_files(cx);
                self.refresh_staged_files();
                self.refresh_history();
            }
        }

        cx.notify();
    }

    fn refresh_dirty_files(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = &self.repository else { return };

        if let Ok(dirty) = repo.unstaged_changes() {
            debug!("Refreshed dirty files: {} files", dirty.len());
            self.dirty_files = dirty;
        }

        // Also update file tree since it shows all status
        if let Ok(status) = repo.status() {
            let items = file_tree::build_nested_tree(&status);
            self.file_tree_state.update(cx, |state, cx| {
                state.set_items(items, cx);
            });
        }

        // Load all dirty file diffs onto the canvas
        self.load_all_dirty_diffs(cx);
    }

    fn refresh_staged_files(&mut self) {
        let Some(repo) = &self.repository else { return };

        if let Ok(staged) = repo.staged_changes() {
            debug!("Refreshed staged files: {} files", staged.len());
            self.staged_files = staged;
        }
    }

    fn refresh_history(&mut self) {
        let Some(repo) = &self.repository else { return };

        if let Ok(commits) = repo.log(Some(100)) {
            debug!("Refreshed history: {} commits", commits.len());
            self.commits = commits;
        }
    }

    /// Load diffs for all dirty (unstaged) files and display on canvas
    fn load_all_dirty_diffs(&mut self, cx: &mut Context<Self>) {
        let Some(repo) = &self.repository else {
            warn!("No repository available");
            return;
        };

        if self.dirty_files.is_empty() {
            info!("No dirty files to load");
            self.diff_canvas.update(cx, |canvas, cx| {
                canvas.set_diffs(vec![], None, cx);
            });
            return;
        }

        info!("Loading diffs for {} dirty files", self.dirty_files.len());

        let mut diffs = Vec::new();
        let config = DiffConfig::default();

        for entry in &self.dirty_files {
            let file_path = &entry.path;

            // Get HEAD version (empty string for new/untracked files)
            let old_content = repo
                .get_content_at_revision("HEAD", file_path)
                .ok()
                .flatten()
                .unwrap_or_default();

            // Get working directory version (empty string for deleted files)
            let new_content = repo
                .get_working_content(file_path)
                .ok()
                .flatten()
                .unwrap_or_default();

            // Compute diff
            match config.diff(&old_content, &new_content) {
                Ok(buffer_diff) => {
                    diffs.push(FileDiff {
                        path: file_path.clone(),
                        old_content,
                        new_content,
                        buffer_diff,
                    });
                }
                Err(e) => {
                    warn!("Failed to compute diff for {}: {}", file_path, e);
                }
            }
        }

        info!("Loaded {} diffs for dirty files", diffs.len());
        self.diff_canvas.update(cx, |canvas, cx| {
            canvas.set_diffs(diffs, None, cx);
        });
    }

    /// Load diff for a dirty (unstaged) file and display on canvas
    #[allow(dead_code)]
    fn load_dirty_file_diff(&mut self, file_index: usize, cx: &mut Context<Self>) {
        let Some(entry) = self.dirty_files.get(file_index) else {
            warn!("No dirty file at index {}", file_index);
            return;
        };
        let Some(repo) = &self.repository else {
            warn!("No repository available");
            return;
        };

        let file_path = &entry.path;
        info!("Loading diff for dirty file: {}", file_path);

        // Get HEAD version (empty string for new/untracked files)
        let old_content = repo
            .get_content_at_revision("HEAD", file_path)
            .ok()
            .flatten()
            .unwrap_or_default();

        // Get working directory version (empty string for deleted files)
        let new_content = repo
            .get_working_content(file_path)
            .ok()
            .flatten()
            .unwrap_or_default();

        debug!(
            "Diff content: old={} bytes, new={} bytes",
            old_content.len(),
            new_content.len()
        );

        // Compute diff
        let config = DiffConfig::default();
        match config.diff(&old_content, &new_content) {
            Ok(buffer_diff) => {
                let diffs = vec![FileDiff {
                    path: file_path.clone(),
                    old_content,
                    new_content,
                    buffer_diff,
                }];

                self.diff_canvas.update(cx, |canvas, cx| {
                    canvas.set_diffs(diffs, None, cx); // None = no commit info for dirty files
                });
                info!("Loaded diff for dirty file: {}", file_path);
            }
            Err(e) => {
                warn!("Failed to compute diff for {}: {}", file_path, e);
            }
        }
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
                            .and_then(|p| p.file_name())
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
                                        // TODO: Focus on this file's diff in the canvas
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

    #[allow(dead_code)]
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
