# Lazydraft — Code Improvement Plan

## Overview

Thorough audit of the entire codebase. Three categories of work: **critical bugs** (dashboard TUI), **structural improvements** (module decomposition), **hardening** (error handling, dead code).

---

## Phase 1: Critical Dashboard Bugs

### 1.1 Fix `ListState` cloning

**Files:** `src/dashboard.rs`

**Problem:** Lines 775 and 857 call `dashboard.list_state.clone()` and `dashboard.writings_list_state.clone()`, passing a **new** `ListState` every frame into `render_stateful_widget`. This resets the internal scroll offset tracking on every draw, making the selection highlight visually unreliable in long lists.

**Fix:** Change the rendering functions to accept `&mut ListState` directly instead of cloning. The caller (`ui`) must pass a mutable reference. This requires changing the signatures of `draw_projects_list` and `draw_writings_list` (or having them borrow mutably from the dashboard).

### 1.2 Fix auto-close popup

**Files:** `src/dashboard.rs`

**Problem:** Lines 450–452:
```rust
thread::spawn(move || {
    thread::sleep(Duration::from_secs(2));
});
```
This spawns a thread that sleeps for 2 seconds and does **nothing**. The popup is never auto-closed.

**Fix:** Track a `popup_timestamp: Option<Instant>` on the dashboard. In the event loop, if a popup has been showing for ≥2 seconds, auto-close it by setting `popup_type = None`. Remove the useless thread.

### 1.3 Fix help popup indicators

**Files:** `src/dashboard.rs`

**Problem:** Help text (line 722) says `🚀` = "Staged writing (auto-staging enabled)", but the rendering code (lines 692–697) shows `🚀⚡` when auto-staging is enabled and `🚀` when disabled.

**Fix:** Update the help text to match:
- `🚀⚡` = Staged writing (auto-staging enabled)
- `🚀` = Staged writing (auto-staging disabled)

---

## Phase 2: Structural Improvements

### 2.1 Extract `ProjectManager` from `config.rs` into `project.rs`

**Files involved:** `src/config.rs` → split into `src/config.rs` + `src/project.rs`

**Problem:** `config.rs` (300+ lines) holds both:
- `Config` struct with path expansion helpers and field validation
- `ProjectManager` with full CRUD (list, create, load, save, delete, active project), legacy migration, and directory management

These are two distinct responsibilities.

**Plan:**
- Move `ProjectConfig`, `ActiveProject`, `ProjectManager` into `src/project.rs`
- Keep `Config`, `Image`, `HeroImage`, `ConfigResult`, `expand_tilde` in `config.rs`
- Re-export `ProjectConfig`, `ProjectManager`, `ActiveProject` from `config.rs` (or update all imports)
- Update `main.rs`, `dashboard.rs`, `writing.rs` imports

### 2.2 Extract command execution from `main.rs` into `commands.rs`

**Files involved:** `src/main.rs` → extract into `src/commands.rs`

**Problem:** `main.rs` is ~280 lines but the `main()` function is a thin dispatcher. All command implementations live here:
- `execute_status_command`
- `execute_stage_command` / `execute_continuous_stage`
- `execute_project_command`
- `execute_config_command` / `open_config_in_editor` / `display_config_info`
- `execute_info_command`
- `check_config_for_empty_fields`
- `exit_with_message`

**Plan:**
- Move all `execute_*` functions and helpers into `pub mod commands { ... }` in `src/commands.rs`
- `main.rs` becomes: parse command, validate config, dispatch to commands module
- Each command function takes config/args and returns `Result<(), String>` or `io::Result<()>`

### 2.3 Split `writing.rs` → `writing.rs` + `frontmatter.rs`

**Files involved:** `src/writing.rs` → split into `src/writing.rs` + `src/frontmatter.rs`

**Problem:** `writing.rs` (~330 lines) handles:
- `Writing` struct + list creation (file scanning) ← keep in `writing.rs`
- Markdown file reading (`read_markdown_file`) ← keep in `writing.rs`
- Content writing pipeline (`update_writing_content_and_transfer`) ← keep in `writing.rs`
- Frontmatter manipulation:
  - `strip_tags`
  - `remove_empty_values`
  - `add_cover_image` / `add_hero_image`
  - `change_image_formats`
  - `strip_wikilinks`
  - `create_writing_name`

**Plan:**
- Move frontmatter manipulation functions into `src/frontmatter.rs`
- Re-export from `writing.rs` or update imports in `main.rs`, `dashboard.rs`, `asset.rs`

### 2.4 Split `dashboard.rs` → `dashboard.rs` + `views.rs`

**Files involved:** `src/dashboard.rs` → split into `src/dashboard.rs` + `src/views.rs`

**Problem:** `dashboard.rs` is ~700 lines. It mixes:
- Dashboard struct + business logic (state management, staging, file watching)
- Event loop (`run_app`)
- All rendering / UI functions (`ui`, `draw_*`, popups, layout helpers)

**Plan:**
- Move all `draw_*` functions, `ui`, `centered_rect`, and `format_relative_time` into `src/views.rs`
- Keep `Dashboard`, `ProjectStats`, `ViewMode`, `PopupType`, `run_dashboard`, `run_app` in `dashboard.rs`
- `views.rs` imports `Dashboard` (and `Theme`) and renders from immutable state
- This creates a clean **model/view separation** — rendering never mutates dashboard state

---

## Phase 3: Harden Core Functionality

### 3.1 Replace silent error swallows with proper propagation

**Files:** `src/dashboard.rs` (primarily), `src/main.rs`

**Problem:** ~15 places use the pattern:
```rust
if let Err(_) = some_operation() {
    // silently ignore
}
```
This hides real failures (disk full, permission denied, config corruption) from the user.

**Fix (tiered approach):**
- **Event loop** (`run_app`): Log errors to stderr instead of swallowing. Don't crash — just show a notification.
- **Dashboard methods** (`stage_selected_writing`, `refresh_data`, etc.): Return `Result` and let the caller decide. The event loop can show an error popup.
- **File watcher errors** (`process_file_events`): Log to stderr. File watching failures are non-fatal.

### 3.2 Replace critical `.expect()` calls with proper error handling

**Files:** `src/main.rs`, `src/writing.rs`

**Problem:** Several `.expect()` calls will crash the program on bad input:
- `main.rs:282` — `create_writing_list(config).expect("Writing list could not be created")`
- `main.rs:284` — `select_draft_writing_from_list(&writing_list).expect("Writing is not selected")`
- `main.rs:286` — `get_asset_list_of_writing(...).expect("Asset List could not be created")`
- `writing.rs:201,224` — Regex creation `expect("Failed to create Wikilink Regex")`
- `writing.rs:240,242` — Path parsing `expect("Could not parse writing name")`
- `writing.rs:268,305` — Config field access `expect("target asset prefix should be set")`

**Fix:**
- Replace `.expect("Writing list could not be created")` with `?` propagation — the caller already returns `io::Result`.
- Replace regex `.expect()` with lazy static or compile-once with `?`.
- Replace config field `.expect()` with `if let` or `ok_or_else` propagation.
- For path parsing `.expect()`, use `unwrap_or_default()` or `?` with a descriptive error.

### 3.3 Add target directory validation before file write

**Files:** `src/writing.rs`

**Problem:** `update_writing_content_and_transfer` creates parent directories (line 169), but doesn't validate that the target directory is writable or that the parent dir creation succeeded.

**Fix:** After `fs::create_dir_all(parent_dir)`, verify the directory exists and is writable. If not, return a clear `io::Error` with a descriptive message.

### 3.4 Remove dead code

**Files:** `src/command.rs`, `src/tui.rs`, `src/dashboard.rs`

| Location | What | Action |
|----------|------|--------|
| `command.rs:26` | `pub project: Option<String>` in `StageOptions` | Remove field (unused) |
| `tui.rs:6` | `pub accent_soft: Color` in `Theme` | Remove field (unused) |
| `dashboard.rs:323` | `if let Err(e) = ...` — variable `e` unused | Change to `if let Err(_) = ...` or use `e` |

---

## Phase 4: Cleanup & Verification

### 4.1 Verify build with zero warnings

```bash
cargo build 2>&1 | grep -E "^warning" | wc -l
# Expected: 0
```

### 4.2 Review diff size

The structural splits (Phase 2) will produce a larger diff. If minimizing diff is preferred, Phase 2 can be deferred. Phase 1 and Phase 3 are the highest-impact changes.

---

## Phase 1b: Dashboard TUI Improvements

### 1.4 Richer header with live status

**Files:** `src/dashboard.rs` (and `src/views.rs` after split)

**Problem:** The header only shows the view mode name. No active project, no auto-stage status, no last-refresh time.

**Fix:** Replace the plain header with an info bar showing:
- View mode and active project name
- Auto-stage ON/OFF indicator (with color)
- Last refresh time ("30s ago")
- Staged count when in writings view

### 1.5 Writings list — show publish date & cleaner indicators

**Files:** `src/dashboard.rs`

**Problem:** The writings list uses emoji icons (📝 ✅ 🚀 ⚡) which:
- Don't render consistently across terminals
- Take up horizontal space without adding clarity
- The staged/auto-stage distinction is confusing (🚀 vs 🚀⚡)

**Fix:**
- Replace emoji with styled text indicators: `[DRAFT]` / `[PUB]` / `[STAGED]` / `[AUTO]`
- Add publish date column (right-aligned) for better context
- Use color consistently: yellow=draft, green=published, cyan=staged

### 1.6 Writing details — richer info panel

**Files:** `src/dashboard.rs`

**Problem:** The writing details panel only shows title, status (Draft/Published), and file path. No publish date, no staging status, no word/file size.

**Fix:** Add:
- Publish date (from `writing.publish_date`)
- Staging status with active/inactive visual
- File size (quick `metadata().len()`)
- Staged/not staged indicator with color

### 1.7 Project details — path validation & clearer layout

**Files:** `src/dashboard.rs`

**Problem:** Project details show source/target paths but don't indicate whether those paths actually exist on disk. The "Configuration Required" warning is plain text.

**Fix:**
- Check if source/target paths exist on disk and show ✅/❌ indicators
- Show path existence as a separate field ("Source exists: ✅")
- Use styled block titles consistently

### 1.8 Footer — show last operation result

**Files:** `src/dashboard.rs`

**Problem:** The footer shows keybindings but no feedback about what happened. After staging, you have to see the popup (and the broken auto-close means it stays forever).

**Fix:**
- Add a `last_message: Option<(String, bool, Instant)>` to Dashboard — stores (message, is_success, timestamp)
- Footer shows the last operation message briefly (fades after 5s)
- Messages show in green (success) or red (error)

### 1.9 Quit confirmation

**Files:** `src/dashboard.rs`

**Problem:** Pressing `q` immediately exits with no confirmation. Easy to accidentally quit.

**Fix:**
- On first `q` press, show a confirmation popup "Press q again to quit"
- Only exit on second `q` within 1 second, or add a confirm popup

### 1.10 Unified color scheme & visual polish

**Files:** `src/tui.rs`, `src/dashboard.rs`

**Problem:** The theme has an unused `accent_soft` color, and some places use hardcoded styles instead of theme methods.

**Fix:**
- Remove unused `accent_soft` from Theme
- Ensure all drawing functions use Theme methods (`theme.success_style()`, `theme.warning_style()`, etc.)
- Add consistent section styling — block titles in accent color, borders in border color

---

## Summary Table

| Phase | Item | Impact | Effort | Priority |
|-------|------|--------|--------|----------|
| 1.1 | Fix ListState cloning | **Critical** — broken selection UX | Small | P0 |
| 1.2 | Fix auto-close popup | **Medium** — broken UX | Trivial | P1 |
| 1.3 | Fix help indicators | **Low** — cosmetic | Trivial | P1 |
| 2.1 | Extract ProjectManager | **Medium** — structure | Medium | P2 |
| 2.2 | Extract commands module | **Medium** — structure | Medium | P2 |
| 2.3 | Split writing/frontmatter | **Medium** — structure | Medium | P2 |
| 2.4 | Split dashboard/views | **Medium** — structure | Medium | P2 |
| 3.1 | Fix error swallowing | **High** — hiding real failures | Large | P1 |
| 3.2 | Replace .expect() calls | **High** — crash on bad input | Medium | P1 |
| 3.3 | Target dir validation | **Medium** — silent failures | Small | P2 |
| 3.4 | Remove dead code | **Low** — hygiene | Trivial | P1 |
| 1.4 | Richer header | **Medium** — info density | Small | P2 |
| 1.5 | Better writings list | **Medium** — readability | Small | P2 |
| 1.6 | Richer writing details | **Medium** — info density | Small | P2 |
| 1.7 | Path validation in details | **Low** — helpful UX | Small | P3 |
| 1.8 | Footer status messages | **Medium** — feedback | Small | P2 |
| 1.9 | Quit confirmation | **Low** — safety | Trivial | P3 |
| 1.10 | Visual polish | **Low** — cosmetics | Small | P3 |
| 4.1 | Zero warnings build | **Low** — hygiene | Trivial | P3 |

---

## Visual Preview (Improved Dashboard)

```
┌─ LazyDraft Dashboard ─────────────────────── Auto-stage: ON ── 12s ago ─┐
│                                                                          │
│  ┌─ Projects (3) ─────────────────┐  ┌─ Project Details ───────────────┐ │
│  │ ● my-blog (2 drafts)          │  │ Name:    my-blog                │ │
│  │   notes                        │  │ Status:  ● Active               │ │
│  │   work (⚠ unconfigured)        │  │ Drafts:  2 ⚠                    │ │
│  │                                │  │ Source:  ~/blog/src ✅          │ │
│  │                                │  │ Target:  ~/blog/dist ✅         │ │
│  │                                │  │ Last:    2h ago                 │ │
│  │                                │  ├─ Overall Statistics ───────────┐ │ │
│  │                                │  │ Projects: 3  Active: 1         │ │
│  │                                │  │ Drafts:   5  Files:  24        │ │
│  │                                │  └────────────────────────────────┘ │ │
│  └────────────────────────────────┘  └─────────────────────────────────┘ │
│  q:Quit h:Help r:Refresh ↑↓:Navigate →:View Writings Enter:Switch       │
└──────────────────────────────────────────────────────────────────────────┘
```

```
┌─ LazyDraft Dashboard — my-blog Writings ─── Auto-stage: ON ── 5s ago ──┐
│                                                                          │
│  ┌─ Writings (12 total, 3 staged) ──────────┐  ┌─ Writing Details ────┐ │
│  │ [DRAFT]  My New Post       2026-05-28  🚀 │  │ Title:  My New Post  │ │
│  │ [DRAFT]  Another Draft     2026-05-25  🚀 │  │ Status: Draft ⚠      │ │
│  │ [PUB]    Published Post    2026-05-20     │  │ Date:   2026-05-28    │ │
│  │ [DRAFT]  Work in Progress  2026-05-18     │  │ Size:   2.4 KB        │ │
│  │ [PUB]    Old Post          2026-05-01     │  │ Staged: ✅ Active     │ │
│  │                                          │  │ Path:   ~/blog/...    │ │
│  │                                          │  └──────────────────────┘ │
│  └──────────────────────────────────────────┘                           │
│  q:Quit h:Help r:Refresh ↑↓:Navigate ←:Back s:Stage u:Revert a:Auto    │
│  ✅ Staged "My New Post" (2 assets)                                     │
└──────────────────────────────────────────────────────────────────────────┘
```

### Key visual changes:
1. **Header** shows view mode + auto-stage status + refresh timer
2. **Projects list** shows draft counts inline, unconfigured warning
3. **Project details** shows path existence checkmarks
4. **Writings list** shows `[DRAFT]`/`[PUB]` instead of emoji, plus publish date column
5. **Writing details** shows file size, staging status
6. **Footer** has last operation message (bottom line, auto-fades)
