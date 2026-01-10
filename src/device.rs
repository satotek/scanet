use std::net::Ipv4Addr;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Device {
    pub ip: Ipv4Addr,
    pub mac: Option<String>,
    pub hostname: Option<String>,
    pub vendor: Option<String>,
    pub response_time: Duration,
    pub last_seen: Instant,
    pub is_new: bool,
}

impl Device {
    pub fn new(ip: Ipv4Addr, response_time: Duration) -> Self {
        Self {
            ip,
            mac: None,
            hostname: None,
            vendor: None,
            response_time,
            last_seen: Instant::now(),
            is_new: true,
        }
    }

    pub fn response_time_ms(&self) -> f64 {
        self.response_time.as_secs_f64() * 1000.0
    }
}
