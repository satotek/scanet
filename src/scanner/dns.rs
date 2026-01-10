use std::net::Ipv4Addr;
use tokio::net::lookup_host;

/// Reverse DNS lookup to get hostname
pub async fn reverse_lookup(ip: Ipv4Addr) -> Option<String> {
    // Use system DNS resolver via tokio
    let addr = format!("{}:0", ip);

    // Try to get hostname using DNS PTR record
    // This uses the system's configured DNS servers
    if let Ok(mut addrs) = lookup_host(&addr).await {
        if let Some(socket_addr) = addrs.next() {
            // Try to resolve hostname
            let host_result = tokio::task::spawn_blocking(move || {
                dns_lookup::lookup_addr(&socket_addr.ip())
            })
            .await;

            if let Ok(Ok(hostname)) = host_result {
                // Don't return if it's just the IP address
                if hostname != ip.to_string() {
                    return Some(hostname);
                }
            }
        }
    }

    None
}
