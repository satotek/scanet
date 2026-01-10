use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::process::Stdio;
use tokio::process::Command;

/// Get MAC addresses from ARP table
pub async fn get_arp_table() -> HashMap<Ipv4Addr, String> {
    let mut result = HashMap::new();

    // Run arp -a command
    let output = Command::new("arp")
        .arg("-a")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .await;

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some((ip, mac)) = parse_arp_line(line) {
                result.insert(ip, mac);
            }
        }
    }

    result
}

/// Get MAC address for a specific IP (fresh lookup with retry)
pub async fn get_mac_for_ip(ip: Ipv4Addr) -> Option<String> {
    // Retry a few times with increasing delay
    for delay_ms in [50, 100, 200] {
        tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;

        // Run arp -a and search for the IP
        let output = Command::new("arp")
            .arg("-a")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .await;

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some((parsed_ip, mac)) = parse_arp_line(line) {
                    if parsed_ip == ip {
                        return Some(mac);
                    }
                }
            }
        }
    }

    None
}

/// Parse a single line from arp -a output
/// macOS format: ? (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope [ethernet]
/// Linux format: ? (192.168.1.1) at aa:bb:cc:dd:ee:ff [ether] on eth0
fn parse_arp_line(line: &str) -> Option<(Ipv4Addr, String)> {
    // Find IP in parentheses
    let start = line.find('(')? + 1;
    let end = line.find(')')?;
    let ip_str = &line[start..end];
    let ip: Ipv4Addr = ip_str.parse().ok()?;

    // Find MAC after "at "
    let at_pos = line.find(" at ")?;
    let after_at = &line[at_pos + 4..];
    let mac_end = after_at.find(' ').unwrap_or(after_at.len());
    let mac = &after_at[..mac_end];

    // Skip incomplete entries
    if mac == "(incomplete)" || mac == "<incomplete>" {
        return None;
    }

    // Normalize MAC format (uppercase, colon-separated)
    let mac = mac
        .replace('-', ":")
        .to_uppercase();

    Some((ip, mac))
}
