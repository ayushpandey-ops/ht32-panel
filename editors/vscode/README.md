# HT32 Panel Git Workspace for VS Code

This extension follows the Git repository containing the active editor and
updates the HT32 Panel Git face whenever you switch repositories. It supports
single-folder and multi-root workspaces.

## Requirements

- `ht32paneld` running and reachable on D-Bus
- `ht32panelctl` available on `PATH`, or configured with `ht32Panel.cliPath`
- Git available on `PATH`

For a source checkout, install the CLI where the extension looks for it by default:

```bash
cargo install --path crates/ht32-panel-cli --root "$HOME/.local"
```

The extension checks `~/.local/bin/ht32panelctl` automatically, then falls back
to `ht32panelctl` on `PATH`. Alternatively, set `ht32Panel.cliPath` to the
absolute path of the executable in your editor settings.

## Install

```bash
code --install-extension editors/vscode/ht32-panel-git-workspace-0.1.8.vsix
```

Restart or reload the editor. Opening a Git workspace selects the Git face and
shows its current branch and recent commits. Switching the active editor to a
file in another repository updates the panel automatically.
When an integrated terminal is active and VS Code shell integration provides
its current directory, the extension uses that directory first. This lets the
panel follow `cd` commands in the terminal; otherwise it falls back to the
active file's workspace.
After each completed command in the active integrated terminal, it refreshes the
Git snapshot even when the repository path has not changed. This keeps the
displayed branch current after checkout, switch, and branch operations.
Failed commands show a roast notification. Choose **Show pixel skull** to open
the pixel art and roast in an editor panel. The roast is also sent to the daemon
and shown on the HT32 LCD with pixel skull art for eight seconds, then the
active face resumes. Failure notifications require VS Code 1.93 or newer and a
terminal with shell integration enabled. The Output channel logs each reported
command and exit code.

To inspect detection and CLI errors, open **View → Output** and select
**HT32 Panel Git Workspace**. You can also run **HT32 Panel: Refresh Git
Repository** from the Command Palette to retry manually.
