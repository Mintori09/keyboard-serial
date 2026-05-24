# keyboard-rs

Rust app that listens to:

- serial input (from keyboard/microcontroller)
- global keyboard hotkeys
- system tray profile switching

It executes shell commands mapped from keys in a JSON config file.

## Features

- Auto-detect serial port (USB preferred), open at `9600` baud
- Parse serial lines in format `EVENT:KEY` (example: `PRESS:1`)
- Profile-based macros:
  - each profile has fixed keys: `0-9`, `*`, `#`
  - each profile has both `press_macros` and `hold_macros`
  - keys are fixed; only command strings are edited
- Tray menu can open GTK configurator (`Configure...`)
- Live apply: configurator saves config and sends `SIGHUP` to daemon PID
- Global hotkey `Ctrl+Alt+I` launches "vim anywhere" flow

## Requirements

- Rust toolchain
- Linux desktop with tray support (for `ksni`)
- GTK4 runtime/dev packages (for `keyboard-rs-config`)
- Serial device

## Configuration

Config path:

`~/.config/keyboard-rs/config.json`

Current schema:

```json
{
  "profiles": [
    {
      "name": "Normal",
      "press_macros": [
        { "key": "0", "command": "" },
        { "key": "1", "command": "" },
        { "key": "2", "command": "" },
        { "key": "3", "command": "" },
        { "key": "4", "command": "" },
        { "key": "5", "command": "" },
        { "key": "6", "command": "" },
        { "key": "7", "command": "" },
        { "key": "8", "command": "" },
        { "key": "9", "command": "" },
        { "key": "*", "command": "" },
        { "key": "#", "command": "" }
      ],
      "hold_macros": [
        { "key": "0", "command": "" },
        { "key": "1", "command": "" },
        { "key": "2", "command": "" },
        { "key": "3", "command": "" },
        { "key": "4", "command": "" },
        { "key": "5", "command": "" },
        { "key": "6", "command": "" },
        { "key": "7", "command": "" },
        { "key": "8", "command": "" },
        { "key": "9", "command": "" },
        { "key": "*", "command": "" },
        { "key": "#", "command": "" }
      ]
    }
  ]
}
```

Notes:

- Legacy schema is auto-migrated when loaded.
- `_default` fallback is removed.
- Keys are normalized to fixed set `0-9`, `*`, `#`.

## Serial Protocol

Input line format:

`EVENT:KEY`

Supported events:

- `PRESS`
- `HOLD`

Routing:

- `PRESS` -> selected profile `press_macros`
- `HOLD` -> selected profile `hold_macros`

## Run

Run daemon:

```bash
cargo run --release
```

Run GTK configurator:

```bash
cargo run --release --bin keyboard-rs-config
```

## Configurator Apply Behavior

`Apply` in GTK app:

1. Validates profile names
2. Normalizes fixed key layout (`0-9`, `*`, `#`)
3. Saves `config.json`
4. Sends `SIGHUP` to PID from `/tmp/keyboard-rs.pid`

If step 4 fails, config is still saved and a warning is shown.

## Test

```bash
cargo test
```

## Desktop Launcher

Template file in repo:

`keyboard-rs-config.desktop`

Install for current user:

```bash
mkdir -p ~/.local/share/applications
cp keyboard-rs-config.desktop ~/.local/share/applications/
update-desktop-database ~/.local/share/applications 2>/dev/null || true
```

## systemd

A user service unit is provided at `systemd/user/keyboard-rs.service`.
