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
