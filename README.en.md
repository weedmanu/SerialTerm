# SerialTerm

[Français](README.md) | [English]

SerialTerm is a desktop app for communicating with devices over a serial port. You can configure the connection, read incoming data, and save the exchange to a text file. The interface uses GTK 4 and Libadwaita.

Version 1.0.0 supports serial connections.

## Features

- Configure the port, baud rate, data bits, parity, stop bits, and flow control.
- Display common ANSI colors and styles, with search and limited scrollback.
- Enable automatic reconnection after a disconnection.
- Choose a line ending for sent commands: none, LF, CR, or CRLF.
- Export received and sent text, with or without timestamps.
- Use the calculator, decimal, hexadecimal and binary converter, and three display themes.
- Use the interface in French or English.

SerialTerm displays supported ANSI sequences in a text view. It does not provide full grid-based terminal emulation.

## Installation

### Debian package

```bash
sudo dpkg -i dist/debian/serial-term_*.deb
sudo apt -f install
```

To access serial ports without administrator privileges, add your account to the `dialout` group, then sign in again:

```bash
sudo usermod -aG dialout "$USER"
```

### Build from source

Install Rust 1.85 or newer, GTK 4.14, Libadwaita 1.5, and their development libraries. Check your Rust version before building.

```bash
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev libudev-dev pkgconf
rustc --version
cargo build --release
./target/release/serial-term
```

The [install-deps.sh](scripts/install-deps.sh) script also prepares Debian, Ubuntu, Fedora, and Arch Linux.

## Getting started

1. Connect your serial device.
2. Select its port. On Linux, the app prefers stable paths in `/dev/serial/by-id/` when available.
3. Set the baud rate and other connection options.
4. Click **Connect**, then enter a command at the bottom of the window.

Each connection starts with an empty display. Data left in the port buffers is cleared. Save the displayed session before reconnecting if you want to keep it.

Preferences are stored in `~/.config/serial-term/settings.json`. If this file contains invalid JSON, the app warns you and keeps a copy named `settings.json.bak` before the next save.

See the [development guide](DEVELOPMENT.md) to contribute. The [improvement plan](PLAN_CORRECTIONS_AMELIORATIONS.md) lists outstanding work.

## License

SerialTerm is distributed under the [GPL-3.0-or-later](LICENSE) license.
