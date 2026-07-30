# noti

`noti` is a minimal, keyboard-first Markdown note editor for Linux/Hyprland. It opens directly into one plain editing surface, auto-saves safely, names new notes from the date plus their first three words, retains automatic revision snapshots, and exposes secondary actions only through keyboard shortcuts and temporary overlays.

## Features

- **Instant launch**: Launch and type immediately.
- **Minimal interface**: No header bar, tabs, sidebars, or toolbars.
- **Autosave**: Crash-safe autosave with atomic writes and debounce.
- **Auto-naming**: Notes are named `YYYY-MM-DD_HH-MM-SS-first-three-words.md`.
- **Revision History**: Snapshots are taken periodically and are easy to restore.
- **Open with…**: Open Markdown or text files from your file manager (or the command line) and edit them in place.
- **Keyboard-first**: Command palette and recent note popovers via hotkeys.
- **Theme support**: Uses your GTK theme exactly, with optional TOML config overrides for transparency, fonts, colors, and padding.

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+P` | Open command palette |
| `Ctrl+R` | Open recent notes list |
| `Ctrl+O` | Open note via file chooser |
| `Ctrl+N` | Create a new note |
| `Ctrl+S` | Save immediately (autosave still runs on its own) |
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
- **Save now**: Write the current file to disk immediately.
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

## Opening existing files ("Open with…")

`noti` registers itself for `text/markdown` and `text/plain`, so it shows up in
the "Open with" menu of your file manager. You can also pass a path directly:

```bash
noti ~/projects/README.md
noti notes/todo.md          # the file is created on the first edit
```

Files opened this way are **edited in place**: every autosave writes straight
back to the original path, so `noti` behaves like a plain text editor for them.
Because those files are not part of your notes folder, the note-management
features are switched off for them:

- no revision snapshots and no revision history (`Show revision history` says so)
- no entry in the recent list, and the next plain `noti` launch reopens your last
  *note*, not the external file
- `Delete current note` (`Ctrl+D`) is disabled

Everything else works as usual — autosave, `Ctrl+S`, highlighting, and the
command palette. The window title shows the file name while such a file is open.
Opening a note (`Ctrl+O`, `Ctrl+R`, `Ctrl+N`, `Alt+Left/Right`) leaves this mode.

`noti` uses a single window: opening a file while `noti` is already running
loads it into the existing window instead of starting a second instance.

## Installation

> **Note:** `noti` depends on **libadwaita ≥ 1.8**, which currently ships only on
> rolling-release distributions (Arch, Fedora Rawhide, openSUSE Tumbleweed, …).
> On older/stable distributions you will need to update these libraries yourself.
>
> The AUR name `noti` was already taken by an unrelated project, so this app is
> published as **`noti-notes`**. The installed command is still `noti`.

### Option A: Arch Linux via the AUR (recommended)

Install with your favourite AUR helper:

```bash
paru -S noti-notes      # latest tagged release
# or
yay -S noti-notes
```

Prefer to track the latest commit? Use the `-git` package instead:

```bash
paru -S noti-notes-git
```

> Both packages install `/usr/bin/noti` and therefore conflict with the
> unrelated `noti` AUR package (a process monitor). You can only have one installed.

Uninstall with `sudo pacman -R noti-notes`.

### Option B: Prebuilt binary from GitHub Releases

Each release on the [Releases page](https://github.com/L3-N0X/noti/releases)
ships a `noti-<version>-x86_64-linux.tar.gz` archive (built on a rolling-release
distro, so it needs an up-to-date GTK 4 / libadwaita stack):

```bash
# Download and verify (replace <version>)
curl -LO https://github.com/L3-N0X/noti/releases/download/v<version>/noti-<version>-x86_64-linux.tar.gz
curl -LO https://github.com/L3-N0X/noti/releases/download/v<version>/noti-<version>-x86_64-linux.tar.gz.sha256
sha256sum -c noti-<version>-x86_64-linux.tar.gz.sha256

# Extract and install (uses the bundled prebuilt binary — no rebuild needed)
tar -xzf noti-<version>-x86_64-linux.tar.gz
cd noti-<version>-x86_64-linux
sudo make install                 # or: make PREFIX=~/.local install
```

### Dependencies (for building from source)

To build `noti` yourself, make sure you have the required libraries and toolchain installed:
*   GTK 4 (`gtk4`)
*   libadwaita ≥ 1.8 (`libadwaita`)
*   gtksourceview5 (`gtksourceview5`)
*   Rust / Cargo (`cargo`)
*   Make (`make`, if installing via Makefile)

### Option C: Build from source (Makefile)

Works on most Linux distributions that provide the dependencies above.

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

### Option D: Arch Linux (local PKGBUILD)

If you have the repository checked out, you can build a native package directly
from the tagged sources using the bundled `PKGBUILD`:

```bash
makepkg -si
sudo pacman -R noti-notes   # to uninstall
```

## Configuration

Settings are stored in `~/.config/noti/config.toml`. A default commented configuration is generated on first launch.

