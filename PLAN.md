# noti — Full Application Plan

**noti** is a minimal, keyboard-first Markdown note editor for Linux/Hyprland. It opens directly into one plain editing surface, auto-saves safely, names new notes from the date plus their first three words, retains automatic revision snapshots, and exposes secondary actions only through keyboard shortcuts and temporary overlays.

The default appearance must remain the user’s existing GTK/OS theme. noti should not set colours, fonts, padding, backgrounds, titlebar styling, dark mode, or custom CSS unless the user explicitly enables an override in `config.toml`. Libadwaita remains included for optional user-controlled appearance behavior, including theme preference and modern GTK styling support. [github](https://github.com/GNOME/libadwaita/blob/main/doc/styles-and-appearance.md)

***

## Goals

- Launch and type immediately.
- One document visible at a time; no tabs, sidebars, toolbars, notebooks, accounts, or sync service.
- Normal Markdown files stored in a user-configurable folder.
- Near-instant, crash-safe autosave without a Save button.
- Date-and-title file naming for new notes.
- Automatic revision history.
- Keyboard-first actions:
  - `Ctrl+P`: command palette
  - `Ctrl+R`: recent notes
  - `Ctrl+O`: open Markdown file
  - `Ctrl+Shift+C`: copy the entire document
- Native GTK behavior by default, with optional configuration overrides.
- Optional transparency that Hyprland can blur behind the editor.

***

## Non-goals for v1

- Rendered Markdown preview.
- Tags, graph view, backlinks, folders/notebooks managed by the app.
- Cloud sync, encryption, accounts, collaboration, or mobile support.
- Multiple open tabs or split views.
- A database; files and TOML state are enough.
- Built-in hard-coded dark/light themes.

***

## Technology stack

| Area | Choice | Purpose |
|---|---|---|
| Language | Rust stable | Safe file handling and a single native binary |
| Application/UI | GTK4 via `gtk-rs` | Native Linux/Wayland window, dialogs, shortcuts, clipboard |
| Optional styling support | Libadwaita via `libadwaita-rs` | Optional user-controlled colour-scheme behavior and GTK/Adwaita styling integration  [gtk-rs](https://gtk-rs.org/gtk4-rs/git/book/libadwaita.html) |
| Editor | GtkSourceView 5 via `sourceview5` | Markdown highlighting, undo/redo, text editor features |
| Configuration | TOML + Serde | Readable user configuration and app state |
| Data paths | XDG directories | Correct Linux configuration and data locations |
| Dates | `chrono` | Timestamped filenames and snapshots |
| Hashing | `sha2` | Skip duplicate saves and snapshots |
| Packaging | Cargo, then PKGBUILD or Flatpak | Personal development first, distribution later |

GtkSourceView’s language manager loads language definition files and can infer a language from a filename and MIME/content type, so noti can enable Markdown highlighting for `.md` files without implementing Markdown parsing itself. [gnome.pages.gitlab.gnome](https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/class.LanguageManager.html)

***

## Dependencies

Use matching major/minor GTK and Libadwaita crate versions.

```toml
[package]
name = "noti"
version = "0.1.0"
edition = "2024"

[dependencies]
gtk = { package = "gtk4", version = "0.10" }
adw = { package = "libadwaita", version = "0.8", features = ["v1_8"] }
sourceview5 = "0.10"
glib = "0.21"
gio = "0.21"

serde = { version = "1", features = ["derive"] }
toml = "0.8"
xdg = "3"

chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
anyhow = "1"
thiserror = "2"
```

Optional later dependency:

```toml
fuzzy-matcher = "0.3"
```

Libadwaita extends GTK4 with additional widgets, stylesheet integration, runtime recolouring support, and cross-desktop dark-style preference APIs; keep it available but do not force its styling choices by default. [gtk-rs](https://gtk-rs.org/gtk4-rs/git/book/libadwaita.html)

***

## Application identity

```text
App name:          noti
Application ID:    io.github.yourname.noti
Binary:            noti
Config directory:  ~/.config/noti/
State directory:   ~/.local/share/noti/
```

Use a stable application ID from the beginning. It makes desktop integration and an optional Hyprland window rule easy to target later.

***

## File locations

### Configuration and state

```text
~/.config/noti/
  config.toml

~/.local/share/noti/
  state.toml
  history/
    2026/
      2026-07-21/
        2026-07-21_20-18-34-project-ideas.md
```

### User notes

The default folder is `~/Documents/Notes`, but users can change it in `config.toml`.

```text
~/Documents/Notes/
  2026-07-21_20-18-34-project-ideas.md
  2026-07-20_09-05-11-meeting-with-alex.md
  2026-07-18_18-00-22-shopping-list.md
```

The notes are ordinary UTF-8 Markdown files. They must work cleanly in a terminal, file manager, Git repository, Syncthing folder, or another text editor.

***

## Default UI

The normal state should show only the editor surface:

```text
┌──────────────────────────────────────────────────┐
│                                                  │
│  Start typing…                                   │
│                                                  │
│                                                  │
│                                                  │
└──────────────────────────────────────────────────┘
```

There is no header bar, toolbar, side panel, persistent status bar, or visible save indicator.

### Editor defaults

- Full-window `GtkSourceView`.
- Markdown highlighting enabled for Markdown documents.
- Line wrapping enabled at word boundaries.
- Line numbers disabled.
- Standard GTK font, foreground colour, background colour, cursor, selection, margins, and focus styling.
- Standard GTK/Libadwaita widgets for command and recent overlays.
- The app uses the current GTK theme unless a configuration override is explicitly uncommented.

Do not set global CSS at launch. Loading no application CSS is the safest way to preserve the user’s GTK theme exactly. Libadwaita recommends using theme variables and semantic style classes rather than hard-coded colours when custom styling is necessary. [github](https://github.com/GNOME/libadwaita/blob/main/doc/styles-and-appearance.md)

***

## `config.toml`

On first launch, create `~/.config/noti/config.toml` with the active functional defaults and all optional appearance values **commented out**. Commented entries act as discoverable documentation but do not alter the GTK/OS theme.

```toml
# noti configuration
#
# Changes take effect after restarting noti, or immediately after
# running "Reload configuration" from Ctrl+P.

[notes]
# Folder for newly created notes. Existing files can still be opened anywhere.
directory = "~/Documents/Notes"

# New notes are saved with this extension.
extension = "md"

[autosave]
# Wait this long after typing stops before saving.
delay_ms = 500

# Create a revision copy at most once per changed note during this interval.
snapshot_interval_minutes = 5

# Remove automatic revision snapshots older than this number of days.
keep_snapshots_days = 90

[behavior]
# Reopen the previously active note on launch.
reopen_last_note = true

# If no previous note exists, create a fresh empty note.
create_note_on_launch_if_none = true

# Maximum number of entries stored for Ctrl+R.
recent_limit = 30

[editor]
# Optional text colour. Leave commented out to use the GTK theme's normal
# foreground colour exactly.
# text_color = "#e6edf3"

# Optional font description, for example "Inter 16" or "JetBrains Mono 15".
# Leave commented out to use the font selected by the user's GTK theme.
# font = "Inter 16"

# Optional editor content padding in pixels.
# Leave commented out to keep the GTK/theme default spacing.
# padding = 24

# Optional line-number display. Default is false.
# line_numbers = false

# Optional Markdown highlighting. Default is true.
# markdown_highlighting = true

# Optional word wrapping. Default is true.
# wrap_text = true

[window]
# Optional opacity from 0.10 to 1.00.
# Leave commented out for a fully opaque normal GTK window.
# On Hyprland, values below 1.0 can allow compositor blur behind the window.
# window_opacity = 0.82

# Optional colour-scheme preference. Leave commented out to follow the OS.
# Valid values: "system", "light", "dark"
# color_scheme = "system"

[appearance]
# Optional application CSS file. Leave commented out to use no app CSS.
# This file is loaded after the GTK/Libadwaita theme and can override it.
# css_file = "~/.config/noti/noti.css"
```

### Configuration rules

- **Active, uncommented defaults** control functionality only: note location, autosave, history retention, recents, and startup behavior.
- **Appearance options remain commented out** so noti does not override the user’s GTK configuration by default.
- If `text_color`, `font`, `padding`, `window_opacity`, `color_scheme`, or `css_file` is absent, noti does not apply that override.
- `Ctrl+P` -> **Reload configuration** re-parses the file and applies safe live changes. The current document remains open and unchanged.
- Invalid configuration values should be ignored, retain the previous valid/default behavior, and display a short non-blocking warning when the command palette is opened.

***

## Optional user styling

noti should support customization without shipping its own theme.

### Text colour

If the user uncomments:

```toml
[editor]
text_color = "#e6edf3"
```

noti loads only this narrow CSS rule:

```css
sourceview text {
  color: #e6edf3;
}
```

If no value exists, do not load the rule. The theme therefore supplies the normal foreground colour.

### Font and padding

If configured:

```toml
[editor]
font = "JetBrains Mono 15"
padding = 24
```

apply only those editor-specific properties. Do not alter global GTK typography or spacing.

### Window opacity

If configured:

```toml
[window]
window_opacity = 0.82
```

validate that it is between `0.10` and `1.00`, then apply it to the window surface. Do not use a global CSS `opacity` rule because that would also fade text, selections, and the cursor.

- `1.00`: ordinary opaque GTK window.
- `0.90`: subtle transparency.
- `0.82`: practical glass effect on Hyprland.
- `0.70`: visibly transparent; potentially distracting with busy wallpapers.

Hyprland performs the blur behind a transparent window; noti only requests the window opacity. Whether blur appears also depends on the user’s Hyprland compositor configuration.

### Theme preference

If configured:

```toml
[window]
color_scheme = "dark"
```

use Libadwaita’s style manager to request dark mode. Supported values:

- `system`: follow OS setting.
- `light`: prefer light.
- `dark`: prefer dark.

Leave it commented by default so the user’s cross-desktop system preference remains authoritative. Libadwaita’s style manager is specifically intended for application-wide style management and can select system, preferred, or forced colour-scheme behavior. [world.pages.gitlab.gnome](https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/stable/latest/docs/libadwaita/struct.StyleManager.html)

### Custom CSS

For advanced customization:

```toml
[appearance]
css_file = "~/.config/noti/noti.css"
```

noti loads the provided CSS after its own minimal optional rules. This lets users customize the window, editor, command palette, borders, corner radius, colours, or spacing without adding application settings for every possible preference.

Example `~/.config/noti/noti.css`:

```css
window {
  border-radius: 14px;
}

sourceview {
  background-color: rgba(22, 22, 28, 0.82);
}

sourceview text {
  color: #edf0f5;
}
```

The application should report CSS parsing errors without crashing.

***

## Markdown editor

Use:

```text
sourceview5::Buffer
sourceview5::View
sourceview5::LanguageManager
```

### Behaviour

- For `.md` and `.markdown` files, ask `LanguageManager` to resolve Markdown by filename/content type.
- Enable syntax highlighting when `markdown_highlighting` is true or absent.
- Preserve normal editing behavior: selection, undo, redo, clipboard actions, keyboard navigation, and input methods.
- Enable word wrapping by default.
- Do not add an HTML preview in v1.
- Do not auto-format, auto-indent aggressively, or modify user text.

GtkSourceView language definitions exist to supply syntax highlighting, and its language manager can find and choose an appropriate language from the document’s filename and content type. [gnome.pages.gitlab.gnome](https://gnome.pages.gitlab.gnome.org/gtksourceview/gtksourceview5/lang-intro.html)

***

## New-note naming

A new note begins untitled in memory. At the first successful autosave, noti generates a filename in the configured note directory.

### Format

```text
YYYY-MM-DD_HH-MM-SS-first-three-words.md
```

### Examples

| Initial content | Filename |
|---|---|
| `Buy oat milk tomorrow` | `2026-07-21_20-18-34-buy-oat-milk.md` |
| `Ideas for the editor` | `2026-07-21_20-18-34-ideas-for-the.md` |
| `# Weekly planning` | `2026-07-21_20-18-34-weekly-planning.md` |
| Empty document | `2026-07-21_20-18-34-untitled.md` |

### Algorithm

1. On the first autosave of an untitled document, inspect its content.
2. Find the first non-empty meaningful line.
3. Strip common Markdown punctuation, such as headings, emphasis markers, inline code markers, link punctuation, and list markers.
4. Extract the first three non-empty words.
5. Convert them to a safe lowercase filename slug joined by hyphens.
6. Prefix the slug with local date and time.
7. Use `untitled` if there are no words.
8. Append the configured extension, defaulting to `.md`.
9. If the path already exists, append `-2`, `-3`, and so forth.
10. Never rename the note automatically after it has been created.

This ensures stable paths even if the opening sentence later changes.

***

## Autosave

Autosave should be invisible and reliable.

### Save sequence

1. A buffer change marks the document dirty.
2. Restart a debounce timer.
3. After `delay_ms` without another change, collect the complete buffer text.
4. For an untitled note, generate its path first.
5. Write to a temporary sibling file, such as `.note-name.md.tmp`.
6. Flush/sync the temporary file.
7. Atomically rename it over the real note file.
8. Mark the document clean only after the atomic rename succeeds.
9. Force a final save on quit, document switch, and window close.

### Failure behavior

- Keep document text in memory.
- Keep the dirty flag set.
- Retry on the next edit or explicit action.
- Do not show a permanent status bar.
- Display a small transient warning only if the user invokes a palette/recent overlay or tries to close while saving has failed.

### External changes

At minimum for v1:

- Detect if the current file’s modification timestamp changed before writing.
- Before overwriting an externally modified file, create a revision snapshot of the loaded state.
- Then save current in-memory content normally.
- A later version can offer a merge/conflict dialog; do not block v1 on it.

***

## Revision history

Revision history protects against accidental deletion or unwanted edits without exposing complexity during normal typing.

### Snapshot policy

- Create a snapshot only after content changes.
- Create at most one snapshot for the active document every `snapshot_interval_minutes`, default 5.
- Create one extra snapshot before switching files if there are unsnapshotted changes.
- Skip snapshots whose content hash matches the previous snapshot.
- Prune snapshots older than `keep_snapshots_days`, default 90, during startup and after saving.

### Snapshot location

```text
~/.local/share/noti/history/
  2026/
    2026-07-21/
      2026-07-21_20-25-00-buy-oat-milk.md
```

### Restore behavior

`Ctrl+P` -> **Show revision history** opens an overlay listing snapshots for the current note.

- Show timestamp and a short first-line excerpt.
- Selecting a snapshot shows it in a temporary read-only preview or directly offers **Restore**.
- Restoring replaces the current editor contents only after confirmation.
- The restored content is autosaved as a new current version, preserving a recoverable history trail.

***

## Keyboard shortcuts

| Shortcut | Action | Behaviour |
|---|---|---|
| `Ctrl+P` | Command palette | Searchable list of commands |
| `Ctrl+R` | Recent notes | Searchable list of recently opened notes |
| `Ctrl+O` | Open Markdown file | Native file picker, filtered to Markdown/text |
| `Ctrl+N` | New note | Opens a new untitled in-memory document |
| `Ctrl+Shift+C` | Copy whole document | Copies all text without changing selection |
| `Ctrl+C` | Standard copy | Copies only selected text |
| `Ctrl+W` | Close note | Saves first, then returns to a new note or exits as configured |
| `Ctrl+Q` | Quit | Force-saves, then exits |
| `Escape` | Dismiss overlay | Returns focus to the editor |

GTK’s `ShortcutController` handles keyboard shortcuts and action activation. [gtk-rs](https://gtk-rs.org/gtk4-rs/git/docs/gtk4/struct.ShortcutController.html)

### Copy full document

`Ctrl+Shift+C` must:

1. Read the entire `SourceBuffer`.
2. Put all text into the system clipboard.
3. Leave selection, cursor, and scroll position unchanged.
4. Briefly show `Copied document` using a transient GTK overlay/toast.

GTK’s GDK clipboard API supports copying plain text through `Clipboard.set_text`. [docs.gtk](https://docs.gtk.org/gdk4/class.Clipboard.html)

***

## Command palette

`Ctrl+P` opens a small GTK-themed centered overlay. It has a single search entry and a filtered result list; focus enters the search field immediately.

### Initial commands

```text
New note
Open Markdown file…
Recent notes
Copy whole document
Show revision history
Restore revision…
Reveal current file in file manager
Open configuration file
Reload configuration
Toggle Markdown highlighting
Toggle line wrapping
Quit
```

### Interaction

- Type to filter commands with a case-insensitive word-prefix matcher initially.
- Up/Down selects an entry.
- Enter executes it.
- Escape closes the palette.
- Use GTK/Libadwaita default styling unless the user applies an optional CSS file.
- Do not show palette controls outside the overlay.

***

## Recent notes

`Ctrl+R` opens the same style of overlay, populated with recent files.

### Rules

- Add or update an entry when a note is opened, created, or successfully saved.
- Canonicalize/deduplicate paths.
- Keep a maximum of `recent_limit`, default 30.
- Sort by most recently opened.
- Verify that each file still exists before displaying it.
- Remove missing or unreadable files from state automatically.
- Display filename as the primary row text and the parent directory as secondary text.
- Save the current document before opening another one.

### `state.toml`

```toml
last_opened = "/home/user/Documents/Notes/2026-07-21_20-18-34-buy-oat-milk.md"

[cursor]
line = 4
column = 12

[[recent]]
path = "/home/user/Documents/Notes/2026-07-21_20-18-34-buy-oat-milk.md"
opened_at = "2026-07-21T20:18:34+02:00"

[[recent]]
path = "/home/user/Documents/Notes/project-ideas.md"
opened_at = "2026-07-20T18:42:11+02:00"
```

Also store optional window size and maximized state. Do not treat `state.toml` as user-editable configuration; tolerate corruption by resetting it safely.

***

## Open-file flow

`Ctrl+O` opens GTK’s asynchronous native file dialog.

### Requirements

- Start in `notes.directory` if it exists.
- Filter `.md`, `.markdown`, and `.txt`.
- Allow opening any readable text file if the user deliberately chooses it.
- Load file content as UTF-8.
- If invalid UTF-8, display a concise error and leave the current document unchanged.
- Save the current dirty document before replacing it.
- Add the opened path to recents.
- Apply Markdown highlighting for Markdown extensions.

***

## Window and Hyprland behavior

### Default

With `window_opacity` commented out, noti creates a normal opaque GTK window and relies entirely on the user’s GTK/Libadwaita theme.

### Optional transparency

If a user uncomments:

```toml
[window]
window_opacity = 0.82
```

noti applies the opacity value to the window surface.

- noti does not implement blur.
- Hyprland may blur what is behind the window when its compositor blur is enabled.
- Do not apply CSS opacity to editor text; that makes content, cursor, and selection difficult to read.
- If opacity is invalid, use normal opacity and report a configuration warning.

Users can target the stable app ID/class in their Hyprland configuration if they want blur/opacity rules scoped specifically to noti.

***

## Rust project layout

```text
noti/
  Cargo.toml
  README.md
  resources/
    io.github.yourname.noti.desktop
    io.github.yourname.noti.metainfo.xml
  src/
    main.rs
    app.rs
    config.rs
    document.rs
    editor.rs
    storage.rs
    autosave.rs
    snapshots.rs
    recents.rs
    palette.rs
    recent_dialog.rs
    open_dialog.rs
    clipboard.rs
    style.rs
    state.rs
    errors.rs
```

### Module responsibilities

| Module | Responsibility |
|---|---|
| `main.rs` | Initialize `adw::Application`, register startup/activate callbacks |
| `app.rs` | Shared application state, actions, document changes |
| `config.rs` | TOML defaults, parsing, validation, path expansion |
| `document.rs` | Current note metadata, dirty state, cursor/scroll restoration |
| `editor.rs` | `GtkSourceView` creation, Markdown language selection, editor options |
| `storage.rs` | Atomic writes, filename generation, path safety |
| `autosave.rs` | Debounce timer and save scheduling |
| `snapshots.rs` | Snapshot creation, pruning, listing, restoration |
| `recents.rs` | Recent note maintenance |
| `palette.rs` | `Ctrl+P` overlay and command dispatch |
| `recent_dialog.rs` | `Ctrl+R` overlay and note selection |
| `open_dialog.rs` | `Ctrl+O` file dialog logic |
| `clipboard.rs` | Whole-document copy command |
| `style.rs` | Apply only explicitly enabled font/colour/padding/opacity/theme/CSS overrides |
| `state.rs` | Read/write `state.toml` safely |
| `errors.rs` | Logging and short user-facing error notifications |

Use `Rc<RefCell<AppState>>` for the shared GUI state on GTK’s main thread. Start with synchronous small-file writes after the debounce period; Markdown notes are normally small, and this keeps the first implementation easier to reason about.

***

## Core data structures

```rust
struct Config {
    notes: NotesConfig,
    autosave: AutosaveConfig,
    behavior: BehaviorConfig,
    editor: EditorConfig,
    window: WindowConfig,
    appearance: AppearanceConfig,
}

struct Document {
    path: Option<PathBuf>,
    dirty: bool,
    last_saved_hash: Option<[u8; 32]>,
    last_snapshot_at: Option<DateTime<Local>>,
}

struct RecentEntry {
    path: PathBuf,
    opened_at: DateTime<FixedOffset>,
}

struct PersistedState {
    last_opened: Option<PathBuf>,
    cursor_line: u32,
    cursor_column: u32,
    recents: Vec<RecentEntry>,
    window_width: i32,
    window_height: i32,
    maximized: bool,
}

struct AppState {
    config: Config,
    document: Document,
    state: PersistedState,
    save_timer: Option<glib::SourceId>,
}
```

Represent paths as `PathBuf` internally. Expand `~` only while parsing configuration. Never construct paths by string concatenation.

***

## Error handling

The app should never discard text silently.

| Situation | Behaviour |
|---|---|
| Config TOML invalid | Keep safe defaults; notify briefly |
| Optional CSS invalid | Ignore CSS; retain GTK theme; notify briefly |
| Notes directory unavailable | Keep document in memory; explain save failure |
| Atomic write fails | Keep dirty state and retry later |
| Opened file is not UTF-8 | Do not replace current document; show error |
| Current note was deleted externally | Keep in-memory document; save as a newly named note on next save |
| Recent file missing | Remove from recent list |
| `state.toml` invalid | Reset state; do not affect user notes |

Log technical details using Rust logging; show concise, non-blocking messages in-app. Do not introduce permanent visual chrome solely for errors.

***

## Implementation order

1. Create the `adw::Application`, single empty GTK window, and full-window GtkSourceView.
2. Add Markdown language detection and source highlighting.
3. Implement config parsing and generate the first-run commented `config.toml`.
4. Create untitled documents and date-plus-three-word filename generation.
5. Implement atomic autosave with debounce and force-save on exit.
6. Add `state.toml`, last-note reopening, cursor restoration, and recent tracking.
7. Implement `Ctrl+O`, `Ctrl+N`, and `Ctrl+Shift+C`.
8. Implement `Ctrl+P` and `Ctrl+R` temporary overlays.
9. Add snapshot history, cleanup, listing, and restore.
10. Add optional style overrides: text colour, font, padding, opacity, Libadwaita scheme preference, custom CSS.
11. Test failure cases, external file changes, long notes, and Hyprland transparency.
12. Add desktop entry, icon, README, and an Arch package recipe.

***

## Completion checklist

noti v1 is complete when:

- It opens directly to a single Markdown-capable editor.
- New notes save automatically in the configured folder with date-plus-first-three-word filenames.
- Existing Markdown files open through `Ctrl+O`.
- Autosave is debounced, atomic, and runs on close/switch.
- Revision snapshots are created, pruned, listed, and recoverable.
- `Ctrl+P`, `Ctrl+R`, `Ctrl+O`, and `Ctrl+Shift+C` work.
- `Ctrl+Shift+C` copies the entire document while ordinary `Ctrl+C` preserves normal selection copy behavior.
- The last note, cursor position, and recent notes restore after relaunch.
- The default configuration does not override the GTK theme.
- Optional appearance settings are documented but commented out in `config.toml`.
- A user can opt into a custom text colour, font, padding, opacity, colour scheme, or custom CSS file.
- An opacity below 1.0 can produce a glass-like effect on a correctly configured Hyprland compositor, while text remains readable.
