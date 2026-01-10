# scanet

A TUI tool for scanning and displaying devices on your local network.

## Features

- **Fast Scanning**: Parallel ping scanning completes /24 networks in seconds
- **Real-time Display**: Discovered devices appear instantly
- **Device Information**: Shows IP address, MAC address, hostname, and vendor name
- **Continuous Mode**: Automatically rescans at configurable intervals
- **Flexible Range**: Specify scan range using CIDR notation or Start-End IPs

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/scanet.git
cd scanet

# Build
cargo build --release

# Run
./target/release/scanet
```

## Usage

```bash
# Auto-detect current network and scan
scanet
```

After launching, press `s` to start scanning.

## Key Bindings

| Key | Action |
|-----|--------|
| `s` | Start scan |
| `c` | Toggle continuous mode |
| `r` | Set scan range |
| `↑` / `k` | Select previous device |
| `↓` / `j` | Select next device |
| `Enter` | Show device details |
| `+` / `-` | Increase/decrease scan interval (continuous mode) |
| `?` | Show help |
| `q` | Quit |

## Scan Range Configuration

Press `r` to open the range configuration dialog.

- **CIDR notation**: `192.168.1.0/24`
- **Range specification**: Enter Start IP and End IP separately

## Screenshot

```
┌─ scanet ─────────────────────────────────────────────────────────────┐
│ Scanning 192.168.1.0/24                                               │
├───────────────────────────────────────────────────────────────────────┤
│ IP Address      MAC Address        Hostname         Vendor            │
│ 192.168.1.1     AA:BB:CC:DD:EE:FF  router.local     Cisco Systems     │
│ 192.168.1.10    11:22:33:44:55:66  macbook.local    Apple, Inc.       │
│ 192.168.1.25    77:88:99:AA:BB:CC  iphone.local     Apple, Inc.       │
├───────────────────────────────────────────────────────────────────────┤
│ Found 3 devices │ [s]can [c]ontinuous [r]ange [?]help [q]uit          │
└───────────────────────────────────────────────────────────────────────┘
```

## Supported Platforms

- macOS
- Linux

## Dependencies

- [ratatui](https://github.com/ratatui/ratatui) - TUI framework
- [tokio](https://tokio.rs/) - Async runtime
- [mac_oui](https://crates.io/crates/mac_oui) - MAC address vendor lookup

## Limitations

- MAC address retrieval is limited to devices on the same subnet (ARP limitation)
- Devices with ICMP blocked by firewall may not be detected

## License

MIT
