# ht32-panel-applet

System tray applet for controlling the HT32 Panel daemon. Works with GNOME, KDE, and other desktop environments supporting the StatusNotifierItem protocol.

## Installation

```bash
cargo install ht32-panel-applet
```

## Usage

```bash
ht32-panel-applet
```

The applet provides quick access to:
- Open web UI
- Toggle LED effects
- View daemon status

## Requirements

- `ht32paneld` running
- Desktop environment with system tray support
- GTK3

## License

AGPL-3.0-or-later
