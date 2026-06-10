# SerialTerm

[FR](README.md) | [EN]

SerialTerm is a GTK4/Libadwaita desktop terminal application focused on serial communication. It lets you finely tune a serial port (baud rate, bits, parity, flow control), display full ANSI streams in real time, and save sessions.

> **Version v1.0.0 — Serial only.** This version focuses exclusively on serial use cases.

## Features

- Configurable serial connection (baud, bits, parity, stop, flow, timeout);
- Full ANSI terminal emulation (colors, SGR, PTY resize);
- Real-time display with scrollback, search, copy/paste;
- USB plug/unplug detection and automatic reconnection;
- Built-in tools: calculator and DEC/HEX/BIN converter;
- Save logs to text files (with or without timestamps);
- Themes (Light, Dark, Hacker);
- JSON persistent configuration;
- Bilingual UI (FR/EN).

## Installation

### Debian package (.deb)

```bash
sudo dpkg -i dist/debian/serial-term_*.deb
sudo apt -f install   # if dependencies are missing
```

To access serial ports without `sudo`:

```bash
sudo usermod -a -G dialout $USER
# then log out / log back in
```

### Build from source

Minimum requirements: Rust 1.75+, GTK 4.14+, Libadwaita 1.5+.

```bash
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev libudev-dev pkgconf cargo
```

```bash
cargo build --release
./target/release/serial-term
```

See [scripts/install-deps.sh](scripts/install-deps.sh) for Debian/Ubuntu, Fedora, and Arch.

## Quick start

1. Plug in your serial device (Arduino, ESP32, STM32, USB-TTL…).
2. Pick the port from the dropdown (stable `/dev/serial/by-id/...` aliases are preferred).
3. Choose baud, bits, parity, stop, flow, and timeout.
4. Click **Connect**.
5. Type commands in the input area at the bottom (line ending choice: none, LF, CR, CRLF).

## Persistent configuration

Settings are stored at:

```
~/.config/serial-term/settings.json
```

The file is created on first launch and stores UI preferences, last used port, serial parameters, and log options.

## Architecture

Strict hexagonal architecture:

- `src/core/`: business logic with no GTK dependency (settings, serial, connection);
- `src/application/`: use cases (validation, transformation);
- `src/ui/`: GTK4/Libadwaita presentation layer.

See [DEVELOPMENT.md](DEVELOPMENT.md) for conventions, tooling, and the validation gate.

## Main dependencies

| Crate          | Role                                          |
|----------------|-----------------------------------------------|
| `gtk4`         | GTK4 bindings                                 |
| `libadwaita`   | Adwaita components                            |
| `tokio`        | Async runtime                                 |
| `tokio-serial` | Serial port access                            |
| `serde_json`   | Settings persistence                          |
| `vte`          | ANSI parser (escape sequences)                |
| `anyhow`       | Error handling                                |

## License

GPL-3.0+ — see [LICENSE](LICENSE).
