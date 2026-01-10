use crate::device::Device;
use anyhow::Result;
use std::net::Ipv4Addr;
use std::process::Stdio;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::process::Command;
use tokio::sync::Mutex;

pub struct PingScanner {
    timeout_ms: u64,
    concurrency: usize,
}

impl PingScanner {
    pub fn new() -> Self {
        Self {
            timeout_ms: 500,
            concurrency: 50,
        }
    }

    pub async fn scan_with_callbacks<F, G>(
        &self,
        start: Ipv4Addr,
        end: Ipv4Addr,
        on_progress: F,
        on_device_found: G,
    ) -> Result<Vec<Device>>
    where
        F: Fn(usize, usize, Ipv4Addr) + Send + Sync + 'static,
        G: Fn(Device) + Send + Sync + 'static,
    {
        let devices = Arc::new(Mutex::new(Vec::new()));
        let on_progress = Arc::new(on_progress);
        let on_device_found = Arc::new(on_device_found);

        let ips = Self::generate_ip_range(start, end);
        let total = ips.len();
        let scanned = Arc::new(AtomicUsize::new(0));

        let chunks: Vec<Vec<Ipv4Addr>> = ips.chunks(self.concurrency).map(|c| c.to_vec()).collect();

        for chunk in chunks {
            let mut handles = Vec::new();

            for ip in chunk {
                let devices = Arc::clone(&devices);
                let scanned = Arc::clone(&scanned);
                let on_progress = Arc::clone(&on_progress);
                let on_device_found = Arc::clone(&on_device_found);
                let timeout_ms = self.timeout_ms;

                let handle = tokio::spawn(async move {
                    if let Some(device) = Self::ping_host(ip, timeout_ms).await {
                        on_device_found(device.clone());
                        let mut devices = devices.lock().await;
                        devices.push(device);
                    }
                    let current = scanned.fetch_add(1, Ordering::SeqCst) + 1;
                    on_progress(current, total, ip);
                });

                handles.push(handle);
            }

            for handle in handles {
                let _ = handle.await;
            }
        }

        let devices = Arc::try_unwrap(devices)
            .map_err(|_| anyhow::anyhow!("Failed to unwrap devices"))?
            .into_inner();

        Ok(devices)
    }

    async fn ping_host(ip: Ipv4Addr, timeout_ms: u64) -> Option<Device> {
        let start = Instant::now();

        // Use system ping command
        // macOS: ping -c 1 -W timeout_ms
        // Linux: ping -c 1 -W timeout_sec
        let timeout_sec = (timeout_ms / 1000).max(1);

        let result = Command::new("ping")
            .args(["-c", "1", "-W", &timeout_sec.to_string(), &ip.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        match result {
            Ok(status) if status.success() => {
                let duration = start.elapsed();
                Some(Device::new(ip, duration))
            }
            _ => None,
        }
    }

    fn generate_ip_range(start: Ipv4Addr, end: Ipv4Addr) -> Vec<Ipv4Addr> {
        let start_u32 = u32::from(start);
        let end_u32 = u32::from(end);
        (start_u32..=end_u32).map(Ipv4Addr::from).collect()
    }
}

impl Default for PingScanner {
    fn default() -> Self {
        Self::new()
    }
}
