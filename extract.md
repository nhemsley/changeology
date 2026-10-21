# Extract Plan

## Current State

### changeology repo (`git@github.com:nhemsley/changeology.git`)

- **main branch**: clean, pushed (`d23e4fc`)
- **main working tree**: 7 modified files (diff-ui refactor, app.rs cleanup) — **NOT committed**
- **infinite-canvas-textured-rendering**: 1 unpushed commit (`2ee7d62 fix warnings`)
- **render-to-texture-research**: local only, not pushed
- **Stashes**: 3 (on infinite-canvas-textured-rendering, tree-viewer, main)

### vendor/zed (`git@github.com:nhemsley/zed.git`, branch `textured-view`)

- **textured-view branch**: pushed, HEAD matches origin (`28a9b95d3f`)
- **17 commits** on top of upstream zed, adding `TexturedView` + related gpui features
- **3 untracked** research .md files in `crates/gpui/research/`
- Key addition: `crates/gpui/src/textured_view.rs` (811 lines) + vulkan lock fix + resize fix

### crates/infinite-canvas

- 6 source files: `lib.rs`, `camera.rs`, `canvas.rs`, `options.rs`, `provider.rs`, `textured_provider.rs`
- 1 example: `textured.rs`
- Deps: `gpui` (workspace = vendor/zed), `serde`, `log`
- `textured_provider.rs` uses `gpui::TexturedView` (linux/freebsd only) — this is the custom zed fork feature

## What to Extract

### 1. `gpui-infinite-canvas` → new standalone repo

```bash
# From changeology dir:

# First, push everything that's unpushed
git push origin infinite-canvas-textured-rendering
# Decide: commit or stash the working tree changes on main

# Then extract:
# Option A: git filter-repo (clean history)
# Option B: just copy the files into a new repo (simpler, lose history)
```

**New repo structure:**
```
gpui-infinite-canvas/
├── src/
│   ├── lib.rs
│   ├── camera.rs
│   ├── canvas.rs
│   ├── options.rs
│   ├── provider.rs
│   └── textured_provider.rs
├── examples/
│   └── textured.rs
├── Cargo.toml          # gpui as git dep pointing to nhemsley/zed textured-view
└── README.md
```

**Cargo.toml dep line:**
```toml
[dependencies]
gpui = { git = "https://github.com/nhemsley/zed.git", branch = "textured-view" }
serde = { version = "1.0", features = ["derive"] }
log = { version = "0.4", features = ["kv_unstable_serde", "serde"] }
```

### 2. co2 consumes it

```toml
# co2/Cargo.toml
[workspace.dependencies]
gpui-infinite-canvas = { git = "https://github.com/nhemsley/gpui-infinite-canvas.git" }
gpui = { git = "https://github.com/nhemsley/zed.git", branch = "textured-view" }
```

## Before Extracting — Push Checklist

| What | Status | Action |
|------|--------|--------|
| changeology main dirty working tree | 7 modified files | Commit or stash |
| infinite-canvas-textured-rendering | 1 unpushed commit | `git push origin infinite-canvas-textured-rendering` |
| render-to-texture-research branch | local only | Push or delete |
| 3 stashes | exist | Review, apply or drop |
| vendor/zed untracked research docs | 3 files | Commit + push to textured-view, or .gitignore |

## Open Research Questions

1. **vendor/zed vs git dep**: Co2 currently vendors jj. Do we also vendor zed, or use a git dep? Git dep is cleaner but slower builds on first clone. Vendoring is huge (~780 changed files in the fork).
2. **textured-view branch freshness**: Fork point is `392c78ea` — upstream has moved. Rebase needed eventually, but not blocking.
3. **TexturedView platform scope**: Only linux/freebsd. Co2 targets linux so this is fine, but the fallback path in `textured_provider.rs` is a placeholder div.
4. **Do we even need TexturedView for co2?** The infinite-canvas pan/zoom/camera stuff is useful standalone. The textured rendering is specifically for rendering GPUI elements as zoomable textures. Co2's initial use case (scrollable log viewer) may only need the camera + provider trait, not the texture pipeline. Could split: `gpui-infinite-canvas` (core) vs textured feature flag.