# keyboard-serial

Matrix keypad firmware and serial listener. Press keys on a 4x3 keypad, execute shell commands on your PC.

## Parts

- **platformio/** — Arduino firmware (PlatformIO). Scans a 4x3 matrix keypad, sends `PRESS:K1` / `HOLD:K1` events over serial.
- **rust/** — Desktop listener (Rust). Reads serial events, looks up the key in a config file, runs the mapped shell command.

## Prerequisites

- Arduino Uno or compatible board with a 4x3 matrix keypad
- Linux desktop with system tray support
- Rust toolchain
- PlatformIO CLI (optional, for firmware development)

## Quick start

Flash the firmware:

```bash
cd platformio
pio run -t upload
```

Create `~/.config/keyboard-rs/config.json`:

```json
{
  "press_macros": [
    { "key": "1", "command": "xdotool key ctrl+c" }
  ],
  "hold_macros": [
    { "key": "2", "command": "playerctl play-pause" }
  ]
}
```

Run the listener:

```bash
cd rust
cargo run --release
```

## Features

- 12-key matrix scanning with debounce and hold detection
- USB serial port auto-detection
- Two operating modes (Normal / Anki) switchable via system tray
- Configurable macro mappings per mode
- Global hotkey `Ctrl+Alt+I` opens a temporary nvim buffer, copies content, pastes anywhere

## Configuration

See [rust/README.md](rust/README.md) for config reference, serial protocol, and systemd integration.

## Build

```bash
./build.sh
```

## License

MIT
