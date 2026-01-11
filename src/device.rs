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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_new() {
        let ip = Ipv4Addr::new(192, 168, 1, 10);
        let response_time = Duration::from_millis(50);
        let device = Device::new(ip, response_time);

        assert_eq!(device.ip, ip);
        assert_eq!(device.response_time, response_time);
        assert!(device.is_new);
        assert!(device.mac.is_none());
        assert!(device.hostname.is_none());
        assert!(device.vendor.is_none());
    }

    #[test]
    fn test_response_time_ms_milliseconds() {
        let device = Device::new(Ipv4Addr::new(192, 168, 1, 1), Duration::from_millis(500));
        assert!((device.response_time_ms() - 500.0).abs() < 0.001);
    }

    #[test]
    fn test_response_time_ms_seconds() {
        let device = Device::new(Ipv4Addr::new(192, 168, 1, 1), Duration::from_secs(1));
        assert!((device.response_time_ms() - 1000.0).abs() < 0.001);
    }

    #[test]
    fn test_response_time_ms_microseconds() {
        let device = Device::new(Ipv4Addr::new(192, 168, 1, 1), Duration::from_micros(1500));
        assert!((device.response_time_ms() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_response_time_ms_fractional() {
        let device = Device::new(Ipv4Addr::new(192, 168, 1, 1), Duration::from_secs_f64(0.25));
        assert!((device.response_time_ms() - 250.0).abs() < 0.001);
    }
}
