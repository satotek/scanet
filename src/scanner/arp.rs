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
pub fn parse_arp_line(line: &str) -> Option<(Ipv4Addr, String)> {
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
    if mac == "(incomplete)" || mac == "<incomplete>" || mac.to_lowercase() == "incomplete" {
        return None;
    }

    // Normalize MAC format (uppercase, colon-separated)
    let mac = mac.replace('-', ":").to_uppercase();

    Some((ip, mac))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_arp_line_macos_format() {
        let line = "? (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope [ethernet]";
        let result = parse_arp_line(line);
        assert!(result.is_some());
        let (ip, mac) = result.unwrap();
        assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(mac, "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn test_parse_arp_line_linux_format() {
        let line = "? (10.0.0.5) at 11:22:33:44:55:66 [ether] on eth0";
        let result = parse_arp_line(line);
        assert!(result.is_some());
        let (ip, mac) = result.unwrap();
        assert_eq!(ip, Ipv4Addr::new(10, 0, 0, 5));
        assert_eq!(mac, "11:22:33:44:55:66");
    }

    #[test]
    fn test_parse_arp_line_hyphen_mac() {
        let line = "? (192.168.1.100) at aa-bb-cc-dd-ee-ff on en0";
        let result = parse_arp_line(line);
        assert!(result.is_some());
        let (ip, mac) = result.unwrap();
        assert_eq!(ip, Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(mac, "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn test_parse_arp_line_incomplete() {
        let line = "? (192.168.1.1) at (incomplete) on en0";
        let result = parse_arp_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_arp_line_incomplete_angle() {
        let line = "? (192.168.1.1) at <incomplete> on en0";
        let result = parse_arp_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_arp_line_incomplete_bare() {
        let line = "? (192.168.1.1) at incomplete on en0";
        let result = parse_arp_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_arp_line_no_parentheses() {
        let line = "malformed line without parentheses";
        let result = parse_arp_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_arp_line_no_at() {
        let line = "? (192.168.1.1) missing the keyword";
        let result = parse_arp_line(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_arp_line_invalid_ip() {
        let line = "? (invalid.ip) at aa:bb:cc:dd:ee:ff on en0";
        let result = parse_arp_line(line);
        assert!(result.is_none());
    }
}
