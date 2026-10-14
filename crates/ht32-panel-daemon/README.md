# ht32-panel-daemon

Daemon for controlling HT32-based LCD displays and RGB LED strips. Provides a D-Bus interface and optional web UI.

## Installation

```bash
cargo install ht32-panel-daemon
```

## Usage

```bash
ht32paneld config.toml
```

## Configuration

```toml
[lcd]
orientation = "landscape"
face = "ascii"
theme = "nord"

[led]
device = "/dev/ttyUSB0"
effect = "rainbow"
intensity = 3
speed = 3

[dbus]
bus = "session"  # or "system"

[web]
enable = true
address = "127.0.0.1:8080"
```

## Features

- Multiple display faces: ASCII, Arcs, Clocks, Digits, Professional
- Color themes: Ember, Hacker, Nord, Solarized Dark/Light, Tokyo Night
- System metrics: CPU, memory, disk, network, temperature
- D-Bus API for integration
- Web UI for browser-based control

## License

AGPL-3.0-or-later
