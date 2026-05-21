# keyboard-rs

Rust app that listens to:

- serial input (from keyboard/microcontroller)
- global keyboard hotkeys
- system tray mode switching

It executes shell commands mapped from keys in a JSON config file.

## Features

- Auto-detect serial port (USB preferred), open at `9600` baud
- Parse serial lines in format `EVENT:KEY` (example: `PRESS:K1`)
- Two tray modes:
  - `Normal`: uses `press_macros`
  - `Anki`: uses `anki_macros`
- `HOLD` events run `hold_macros` in `Normal` mode
- Global hotkey `Ctrl+Alt+I` launches "vim anywhere" flow:
  - opens `kitty -e nvim <tempfile>`
  - copies text with `wl-copy`
  - pastes via `ydotool`

## Requirements

- Rust toolchain
- Linux desktop with tray support (for `ksni`)
- Serial device
- Commands used by optional hotkey flow:
  - `kitty`
  - `nvim`
  - `wl-copy`
  - `ydotool`

## Configuration

Config path:

`~/.config/keyboard-rs/config.json`

Example:

```json
{
  "press_macros": [
    { "key": "K1", "command": "xdotool key ctrl+c" }
  ],
  "anki_macros": [
    { "key": "K1", "command": "xdotool key 1" }
  ],
  "hold_macros": [
    { "key": "K2", "command": "playerctl play-pause" }
  ]
}
```

Notes:

- Missing/invalid config loads as empty macro sets.
- Commands are executed with shell (`sh -c` on Linux).

## Serial Protocol

Input line format:

`EVENT:KEY`

Supported events:

- `PRESS`
- `HOLD`

Routing:

- `PRESS` + `Normal` mode -> `press_macros`
- `PRESS` + `Anki` mode -> `anki_macros`
- `HOLD` + `Normal` mode -> `hold_macros`

## Run

```bash
cargo run --release
```

On startup, app:

1. Detects and opens serial port
2. Starts global keyboard listener thread
3. Starts tray app for mode switch / exit

## Test

```bash
cargo test
```

Current integration test reads live config from `~/.config/keyboard-rs/config.json`.

## Package (RPM metadata present)

Project includes RPM metadata in `Cargo.toml` under `[package.metadata.rpm]`.
