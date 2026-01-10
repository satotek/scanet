use crate::device::Device;
use std::net::Ipv4Addr;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanState {
    Idle,
    Scanning,
    Waiting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Main,
    Help,
    DeviceDetail,
    RangeInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeInputField {
    Start,
    End,
}

pub struct App {
    pub devices: Vec<Device>,
    pub selected_index: usize,
    pub scan_state: ScanState,
    pub last_scan: Option<Instant>,
    pub scan_range: (Ipv4Addr, Ipv4Addr),
    pub status_message: String,
    pub should_quit: bool,
    // Scan progress tracking
    pub scanned_count: usize,
    pub total_count: usize,
    pub spinner_index: usize,
    pub current_scanning_ip: Option<Ipv4Addr>,
    pub continuous_mode: bool,
    pub scan_interval_secs: u64,
    pub wait_started: Option<Instant>,
    // View state
    pub current_view: View,
    // Range input state
    pub range_input_start: String,
    pub range_input_end: String,
    pub range_input_field: RangeInputField,
    pub range_input_error: Option<String>,
    pub range_input_cursor: usize,
}

impl App {
    pub fn new(local_ip: Ipv4Addr) -> Self {
        let (start, end) = Self::calculate_scan_range(local_ip);
        Self {
            devices: Vec::new(),
            selected_index: 0,
            scan_state: ScanState::Idle,
            last_scan: None,
            scan_range: (start, end),
            status_message: String::from("Press 's' to start scanning, 'c' for continuous mode"),
            should_quit: false,
            scanned_count: 0,
            total_count: 0,
            spinner_index: 0,
            current_scanning_ip: None,
            continuous_mode: false,
            scan_interval_secs: 10,
            wait_started: None,
            current_view: View::Main,
            range_input_start: start.to_string(),
            range_input_end: end.to_string(),
            range_input_field: RangeInputField::Start,
            range_input_error: None,
            range_input_cursor: 0,
        }
    }

    fn calculate_scan_range(ip: Ipv4Addr) -> (Ipv4Addr, Ipv4Addr) {
        let octets = ip.octets();
        let start = Ipv4Addr::new(octets[0], octets[1], octets[2], 1);
        let end = Ipv4Addr::new(octets[0], octets[1], octets[2], 254);
        (start, end)
    }

    pub fn select_next(&mut self) {
        if !self.devices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.devices.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.devices.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.devices.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn add_device(&mut self, device: Device) {
        if let Some(existing) = self.devices.iter_mut().find(|d| d.ip == device.ip) {
            existing.last_seen = device.last_seen;
            existing.response_time = device.response_time;
            existing.is_new = false;
        } else {
            self.devices.push(device);
            self.devices.sort_by(|a, b| a.ip.cmp(&b.ip));
        }
    }

    pub fn update_device_hostname(&mut self, ip: Ipv4Addr, hostname: Option<String>) {
        if let Some(device) = self.devices.iter_mut().find(|d| d.ip == ip) {
            device.hostname = hostname;
        }
    }

    pub fn update_device_vendor(&mut self, ip: Ipv4Addr, vendor: Option<String>) {
        if let Some(device) = self.devices.iter_mut().find(|d| d.ip == ip) {
            device.vendor = vendor;
        }
    }

    pub fn clear_devices(&mut self) {
        self.devices.clear();
        self.selected_index = 0;
    }

    pub fn time_since_last_scan(&self) -> Option<u64> {
        self.last_scan.map(|t| t.elapsed().as_secs())
    }

    pub fn scan_range_cidr(&self) -> String {
        let octets = self.scan_range.0.octets();
        format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
    }

    pub fn update_progress(&mut self, scanned: usize, total: usize, current_ip: Option<Ipv4Addr>) {
        self.scanned_count = scanned;
        self.total_count = total;
        self.current_scanning_ip = current_ip;
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_index = (self.spinner_index + 1) % 10;
    }

    pub fn spinner_char(&self) -> char {
        const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        SPINNER_FRAMES[self.spinner_index]
    }

    pub fn progress_percent(&self) -> usize {
        if self.total_count == 0 {
            0
        } else {
            (self.scanned_count * 100) / self.total_count
        }
    }

    pub fn wait_remaining_secs(&self) -> u64 {
        if let Some(started) = self.wait_started {
            let elapsed = started.elapsed().as_secs();
            self.scan_interval_secs.saturating_sub(elapsed)
        } else {
            0
        }
    }

    pub fn selected_device(&self) -> Option<&Device> {
        self.devices.get(self.selected_index)
    }

    pub fn open_range_input(&mut self) {
        self.range_input_start = self.scan_range.0.to_string();
        self.range_input_end = self.scan_range.1.to_string();
        self.range_input_field = RangeInputField::Start;
        self.range_input_cursor = self.range_input_start.len();
        self.range_input_error = None;
        self.current_view = View::RangeInput;
    }

    pub fn toggle_range_input_field(&mut self) {
        self.range_input_field = match self.range_input_field {
            RangeInputField::Start => {
                self.range_input_cursor = self.range_input_end.len();
                RangeInputField::End
            }
            RangeInputField::End => {
                self.range_input_cursor = self.range_input_start.len();
                RangeInputField::Start
            }
        };
    }

    pub fn apply_range_input(&mut self) -> bool {
        // Check if start field contains CIDR notation
        if self.range_input_start.contains('/') {
            match Self::parse_cidr(&self.range_input_start) {
                Ok((start, end)) => {
                    self.scan_range = (start, end);
                    self.range_input_error = None;
                    self.current_view = View::Main;
                    self.status_message = format!("Range set: {} - {}", start, end);
                    return true;
                }
                Err(e) => {
                    self.range_input_error = Some(e);
                    return false;
                }
            }
        }

        let start: Result<Ipv4Addr, _> = self.range_input_start.parse();
        let end: Result<Ipv4Addr, _> = self.range_input_end.parse();

        match (start, end) {
            (Ok(s), Ok(e)) => {
                if s > e {
                    self.range_input_error = Some("Start must be <= End".to_string());
                    false
                } else {
                    self.scan_range = (s, e);
                    self.range_input_error = None;
                    self.current_view = View::Main;
                    self.status_message = format!("Range set: {} - {}", s, e);
                    true
                }
            }
            (Err(_), _) => {
                self.range_input_error = Some("Invalid start IP".to_string());
                false
            }
            (_, Err(_)) => {
                self.range_input_error = Some("Invalid end IP".to_string());
                false
            }
        }
    }

    fn parse_cidr(cidr: &str) -> Result<(Ipv4Addr, Ipv4Addr), String> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err("Invalid CIDR format".to_string());
        }

        let ip: Ipv4Addr = parts[0].parse().map_err(|_| "Invalid IP address")?;
        let prefix: u8 = parts[1].parse().map_err(|_| "Invalid prefix length")?;

        if prefix > 32 {
            return Err("Prefix must be 0-32".to_string());
        }

        let ip_u32 = u32::from(ip);
        let mask = if prefix == 0 {
            0
        } else {
            !((1u32 << (32 - prefix)) - 1)
        };

        let network = ip_u32 & mask;
        let broadcast = network | !mask;

        // For host addresses, use network+1 to broadcast-1
        let start = if prefix >= 31 { network } else { network + 1 };
        let end = if prefix >= 31 {
            broadcast
        } else {
            broadcast - 1
        };

        Ok((Ipv4Addr::from(start), Ipv4Addr::from(end)))
    }

    pub fn range_input_push(&mut self, c: char) {
        let input = match self.range_input_field {
            RangeInputField::Start => &mut self.range_input_start,
            RangeInputField::End => &mut self.range_input_end,
        };
        if self.range_input_cursor >= input.len() {
            input.push(c);
        } else {
            input.insert(self.range_input_cursor, c);
        }
        self.range_input_cursor += 1;
        self.range_input_error = None;
    }

    pub fn range_input_pop(&mut self) {
        if self.range_input_cursor == 0 {
            return;
        }
        let input = match self.range_input_field {
            RangeInputField::Start => &mut self.range_input_start,
            RangeInputField::End => &mut self.range_input_end,
        };
        if self.range_input_cursor >= input.len() {
            input.pop();
        } else {
            input.remove(self.range_input_cursor - 1);
        }
        self.range_input_cursor -= 1;
        self.range_input_error = None;
    }

    pub fn range_input_cursor_left(&mut self) {
        if self.range_input_cursor > 0 {
            self.range_input_cursor -= 1;
        }
    }

    pub fn range_input_cursor_right(&mut self) {
        let len = match self.range_input_field {
            RangeInputField::Start => self.range_input_start.len(),
            RangeInputField::End => self.range_input_end.len(),
        };
        if self.range_input_cursor < len {
            self.range_input_cursor += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    // ==================== parse_cidr tests ====================

    #[test]
    fn test_parse_cidr_valid_24() {
        let result = App::parse_cidr("192.168.1.0/24");
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert_eq!(start, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(end, Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn test_parse_cidr_valid_25() {
        let result = App::parse_cidr("10.0.0.0/25");
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert_eq!(start, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(end, Ipv4Addr::new(10, 0, 0, 126));
    }

    #[test]
    fn test_parse_cidr_valid_16() {
        let result = App::parse_cidr("172.16.0.0/16");
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert_eq!(start, Ipv4Addr::new(172, 16, 0, 1));
        assert_eq!(end, Ipv4Addr::new(172, 16, 255, 254));
    }

    #[test]
    fn test_parse_cidr_invalid_prefix_too_large() {
        let result = App::parse_cidr("192.168.1.0/33");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Prefix must be 0-32");
    }

    #[test]
    fn test_parse_cidr_invalid_ip() {
        let result = App::parse_cidr("invalid.ip/24");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid IP address");
    }

    #[test]
    fn test_parse_cidr_invalid_prefix_format() {
        let result = App::parse_cidr("192.168.1.0/abc");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid prefix length");
    }

    #[test]
    fn test_parse_cidr_missing_prefix() {
        let result = App::parse_cidr("192.168.1.0");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Invalid CIDR format");
    }

    // ==================== calculate_scan_range tests ====================

    #[test]
    fn test_calculate_scan_range_standard() {
        let (start, end) = App::calculate_scan_range(Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(start, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(end, Ipv4Addr::new(192, 168, 1, 254));
    }

    #[test]
    fn test_calculate_scan_range_class_a() {
        let (start, end) = App::calculate_scan_range(Ipv4Addr::new(10, 0, 5, 50));
        assert_eq!(start, Ipv4Addr::new(10, 0, 5, 1));
        assert_eq!(end, Ipv4Addr::new(10, 0, 5, 254));
    }

    #[test]
    fn test_calculate_scan_range_edge_ip() {
        let (start, end) = App::calculate_scan_range(Ipv4Addr::new(172, 16, 0, 1));
        assert_eq!(start, Ipv4Addr::new(172, 16, 0, 1));
        assert_eq!(end, Ipv4Addr::new(172, 16, 0, 254));
    }

    // ==================== progress_percent tests ====================

    #[test]
    fn test_progress_percent_zero_total() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.total_count = 0;
        app.scanned_count = 0;
        assert_eq!(app.progress_percent(), 0);
    }

    #[test]
    fn test_progress_percent_half() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.total_count = 100;
        app.scanned_count = 50;
        assert_eq!(app.progress_percent(), 50);
    }

    #[test]
    fn test_progress_percent_complete() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.total_count = 100;
        app.scanned_count = 100;
        assert_eq!(app.progress_percent(), 100);
    }

    #[test]
    fn test_progress_percent_integer_division() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.total_count = 3;
        app.scanned_count = 1;
        assert_eq!(app.progress_percent(), 33);
    }

    // ==================== spinner_char tests ====================

    #[test]
    fn test_spinner_char_index_0() {
        let app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(app.spinner_char(), '⠋');
    }

    #[test]
    fn test_spinner_char_cycles() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.spinner_index = 5;
        assert_eq!(app.spinner_char(), '⠴');
    }

    // ==================== scan_range_cidr tests ====================

    #[test]
    fn test_scan_range_cidr_format() {
        let app = App::new(Ipv4Addr::new(192, 168, 1, 100));
        assert_eq!(app.scan_range_cidr(), "192.168.1.0/24");
    }

    #[test]
    fn test_scan_range_cidr_different_network() {
        let app = App::new(Ipv4Addr::new(10, 0, 5, 50));
        assert_eq!(app.scan_range_cidr(), "10.0.5.0/24");
    }

    // ==================== select_next / select_previous tests ====================

    #[test]
    fn test_select_next_empty_list() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.select_next();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_select_next_wraps() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.devices.push(Device::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Duration::from_millis(1),
        ));
        app.devices.push(Device::new(
            Ipv4Addr::new(192, 168, 1, 2),
            Duration::from_millis(1),
        ));
        app.devices.push(Device::new(
            Ipv4Addr::new(192, 168, 1, 3),
            Duration::from_millis(1),
        ));
        app.selected_index = 2;
        app.select_next();
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_select_previous_wraps() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.devices.push(Device::new(
            Ipv4Addr::new(192, 168, 1, 1),
            Duration::from_millis(1),
        ));
        app.devices.push(Device::new(
            Ipv4Addr::new(192, 168, 1, 2),
            Duration::from_millis(1),
        ));
        app.devices.push(Device::new(
            Ipv4Addr::new(192, 168, 1, 3),
            Duration::from_millis(1),
        ));
        app.selected_index = 0;
        app.select_previous();
        assert_eq!(app.selected_index, 2);
    }

    // ==================== add_device tests ====================

    #[test]
    fn test_add_device_new() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        let device = Device::new(Ipv4Addr::new(192, 168, 1, 10), Duration::from_millis(5));
        app.add_device(device);
        assert_eq!(app.devices.len(), 1);
        assert!(app.devices[0].is_new);
    }

    #[test]
    fn test_add_device_sorted() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.add_device(Device::new(
            Ipv4Addr::new(192, 168, 1, 30),
            Duration::from_millis(1),
        ));
        app.add_device(Device::new(
            Ipv4Addr::new(192, 168, 1, 10),
            Duration::from_millis(1),
        ));
        app.add_device(Device::new(
            Ipv4Addr::new(192, 168, 1, 20),
            Duration::from_millis(1),
        ));
        assert_eq!(app.devices[0].ip, Ipv4Addr::new(192, 168, 1, 10));
        assert_eq!(app.devices[1].ip, Ipv4Addr::new(192, 168, 1, 20));
        assert_eq!(app.devices[2].ip, Ipv4Addr::new(192, 168, 1, 30));
    }

    #[test]
    fn test_add_device_update_existing() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.add_device(Device::new(
            Ipv4Addr::new(192, 168, 1, 10),
            Duration::from_millis(5),
        ));
        assert!(app.devices[0].is_new);
        app.add_device(Device::new(
            Ipv4Addr::new(192, 168, 1, 10),
            Duration::from_millis(3),
        ));
        assert_eq!(app.devices.len(), 1);
        assert!(!app.devices[0].is_new);
    }

    // ==================== apply_range_input tests ====================

    #[test]
    fn test_apply_range_input_valid_cidr() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.range_input_start = "10.0.0.0/24".to_string();
        assert!(app.apply_range_input());
        assert_eq!(app.scan_range.0, Ipv4Addr::new(10, 0, 0, 1));
        assert_eq!(app.scan_range.1, Ipv4Addr::new(10, 0, 0, 254));
    }

    #[test]
    fn test_apply_range_input_valid_ip_range() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.range_input_start = "192.168.1.1".to_string();
        app.range_input_end = "192.168.1.100".to_string();
        assert!(app.apply_range_input());
        assert_eq!(app.scan_range.0, Ipv4Addr::new(192, 168, 1, 1));
        assert_eq!(app.scan_range.1, Ipv4Addr::new(192, 168, 1, 100));
    }

    #[test]
    fn test_apply_range_input_invalid_range() {
        let mut app = App::new(Ipv4Addr::new(192, 168, 1, 1));
        app.range_input_start = "192.168.1.100".to_string();
        app.range_input_end = "192.168.1.50".to_string();
        assert!(!app.apply_range_input());
        assert!(app.range_input_error.is_some());
    }
}
