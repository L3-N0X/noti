# noti

`noti` is a minimal, keyboard-first Markdown note editor for Linux/Hyprland. It opens directly into one plain editing surface, auto-saves safely, names new notes from the date plus their first three words, retains automatic revision snapshots, and exposes secondary actions only through keyboard shortcuts and temporary overlays.

## Features

- **Instant launch**: Launch and type immediately.
- **Minimal interface**: No header bar, tabs, sidebars, or toolbars.
- **Autosave**: Crash-safe autosave with atomic writes and debounce.
- **Auto-naming**: Notes are named `YYYY-MM-DD_HH-MM-SS-first-three-words.md`.
- **Revision History**: Snapshots are taken periodically and are easy to restore.
- **Keyboard-first**: Command palette and recent note popovers via hotkeys.
- **Theme support**: Uses your GTK theme exactly, with optional TOML config overrides for transparency, fonts, colors, and padding.

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+P` | Open command palette |
| `Ctrl+R` | Open recent notes list |
| `Ctrl+O` | Open note via file chooser |
| `Ctrl+N` | Create a new note |
| `Ctrl+W` | Close the current note |
| `Ctrl+Shift+C` | Copy the entire note to clipboard |
| `Ctrl+C` | Copy active text selection |
| `Ctrl+Q` | Save and quit |
| `Alt+Left` | Cycle to previous note |
| `Alt+Right` | Cycle to next note |
| `Escape` | Dismiss any open overlay / dialog |

## Command Palette Actions

By pressing `Ctrl+P`, you can access additional actions and editor settings:

- **New note**: Create a new blank note.
- **Close current note**: Close the active note.
- **Open Markdown file…**: Open any markdown file via file browser.
- **Recent notes**: Show the recent notes list overlay.
- **Copy whole document**: Copy all text to the clipboard.
- **Show revision history**: Browse and restore automatic revision snapshots.
- **Reveal current file in file manager**: Open parent directory in default file manager.
- **Open configuration file**: Open `config.toml` in your default text editor.
- **Reload configuration**: Hot-reload appearance settings.
- **Toggle Markdown highlighting**: Turn syntax highlighting on or off.
- **Toggle line wrapping**: Turn line wrapping on or off.
- **Quit**: Save and exit the application.

## Installation

### Dependencies
Before building `noti`, make sure you have the required libraries and toolchain installed:
*   GTK 4 (`gtk4`)
*   libadwaita (`libadwaita`)
*   gtksourceview5 (`gtksourceview5`)
*   Rust / Cargo (`cargo`)
*   Make (`make`, if installing via Makefile)

### Option A: Universal Installation (Makefile)
This is the recommended installation method for most Linux distributions.

1. **Build the application:**
   ```bash
   make
   ```
2. **Install (System-wide):**
   ```bash
   sudo make install
   ```
   *Installs the binary to `/usr/local/bin`, the desktop launcher to `/usr/local/share/applications`, the icon to `/usr/local/share/icons/hicolor`, and registers them.*

3. **Install (User-local / No `sudo`):**
   ```bash
   make PREFIX=~/.local install
   ```
   *Installs inside your user's home folder (`~/.local/bin`, `~/.local/share/applications`, etc.).*

4. **Uninstall:**
   ```bash
   sudo make uninstall  # Or: make PREFIX=~/.local uninstall
   ```

### Option B: Arch Linux (PKGBUILD)
If you are running Arch Linux, Manjaro, or EndeavourOS, you can build a native package from your local files:

1. **Build and install:**
   ```bash
   makepkg -si --noprepare
   ```
   *The `--noprepare` flag ensures the package builder uses your local workspace files rather than fetching the remote repository.*

2. **Uninstall:**
   ```bash
   sudo pacman -R noti
   ```

## Configuration

Settings are stored in `~/.config/noti/config.toml`. A default commented configuration is generated on first launch.

