# ht32-panel-cli

Command-line tool for controlling the HT32 Panel daemon.

## Installation

```bash
cargo install ht32-panel-cli
```

## Usage

```bash
# LCD control
ht32panelctl lcd orientation landscape
ht32panelctl lcd face ascii
ht32panelctl lcd theme nord

# LED control
ht32panelctl led set rainbow --intensity 3 --speed 3
ht32panelctl led off

# Status
ht32panelctl status
```

## Requirements

Requires `ht32paneld` to be running.

## License

AGPL-3.0-or-later
