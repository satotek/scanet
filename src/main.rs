mod app;
mod device;
mod scanner;
mod ui;

use anyhow::Result;
use app::{App, ScanState, View};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use device::Device;
use local_ip_address::local_ip;
use mac_oui::Oui;
use ratatui::{backend::CrosstermBackend, Terminal};
use scanner::{get_arp_table, get_mac_for_ip, lookup_vendor_online, reverse_lookup, PingScanner};
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;

enum ScanMessage {
    Initializing,
    ScanStarted,
    Device(Device),
    UpdateDevice { ip: Ipv4Addr, hostname: Option<String> },
    UpdateVendor { ip: Ipv4Addr, vendor: String },
    Progress { scanned: usize, total: usize, current_ip: Ipv4Addr },
    ScanComplete,
}

fn lookup_vendor(oui_db: &Oui, mac: &str) -> Option<String> {
    oui_db
        .lookup_by_mac(mac)
        .ok()
        .flatten()
        .map(|entry| entry.company_name.clone())
}

fn start_scan(
    tx: mpsc::Sender<ScanMessage>,
    start: Ipv4Addr,
    end: Ipv4Addr,
    oui_db: Arc<Option<Oui>>,
) {
    tokio::spawn(async move {
        // Notify initializing
        let _ = tx.send(ScanMessage::Initializing).await;

        // Get ARP table
        let arp_table = Arc::new(get_arp_table().await);

        // Notify scan started
        let _ = tx.send(ScanMessage::ScanStarted).await;

        let scanner = PingScanner::new();
        let progress_tx = tx.clone();
        let device_tx = tx.clone();

        let on_progress = move |scanned: usize, total: usize, current_ip: Ipv4Addr| {
            let tx = progress_tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(ScanMessage::Progress { scanned, total, current_ip }).await;
            });
        };

        let arp_table_clone = arp_table.clone();
        let oui_db_clone = oui_db;
        let on_device_found = move |device: Device| {
            let tx = device_tx.clone();
            let arp_table = arp_table_clone.clone();
            let oui_db = oui_db_clone.clone();
            let ip = device.ip;

            // Get MAC from cached ARP table
            let cached_mac = arp_table.get(&ip).cloned();

            tokio::spawn(async move {
                // If not in cache, do a fresh lookup
                let mac = if cached_mac.is_some() {
                    cached_mac
                } else {
                    get_mac_for_ip(ip).await
                };

                // Get vendor from MAC address
                let vendor = mac.as_ref().and_then(|m| {
                    oui_db.as_ref().as_ref().and_then(|db| lookup_vendor(db, m))
                });

                // Send device with MAC and vendor
                let mut device = device;
                device.mac = mac;
                device.vendor = vendor;

                let _ = tx.send(ScanMessage::Device(device)).await;

                // Resolve hostname in background
                let hostname = reverse_lookup(ip).await;

                // Send update if hostname found
                if hostname.is_some() {
                    let _ = tx.send(ScanMessage::UpdateDevice { ip, hostname }).await;
                }
            });
        };

        let _ = scanner.scan_with_callbacks(start, end, on_progress, on_device_found).await;
        let _ = tx.send(ScanMessage::ScanComplete).await;
    });
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get local IP address
    let local_ip = match local_ip()? {
        std::net::IpAddr::V4(ip) => ip,
        std::net::IpAddr::V6(_) => {
            eprintln!("IPv6 not supported, using default");
            Ipv4Addr::new(192, 168, 1, 1)
        }
    };

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(local_ip);

    // Initialize OUI database once at startup
    let oui_db = Arc::new(Oui::default().ok());

    // Channel for scan results
    let (tx, mut rx) = mpsc::channel::<ScanMessage>(256);

    // Main loop
    loop {
        // Draw UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Check for scan results
        while let Ok(msg) = rx.try_recv() {
            match msg {
                ScanMessage::Initializing => {
                    app.scan_state = ScanState::Scanning;
                    app.status_message = String::from("Loading ARP table...");
                }
                ScanMessage::ScanStarted => {
                    // Ignore if continuous mode was turned off
                    if app.scan_state == ScanState::Idle && !app.continuous_mode {
                        continue;
                    }
                    app.scan_state = ScanState::Scanning;
                    app.wait_started = None;
                }
                ScanMessage::Device(device) => {
                    app.add_device(device);
                }
                ScanMessage::UpdateDevice { ip, hostname } => {
                    app.update_device_hostname(ip, hostname);
                }
                ScanMessage::UpdateVendor { ip, vendor } => {
                    app.update_device_vendor(ip, Some(vendor));
                }
                ScanMessage::Progress { scanned, total, current_ip } => {
                    app.update_progress(scanned, total, Some(current_ip));
                }
                ScanMessage::ScanComplete => {
                    app.last_scan = Some(Instant::now());
                    if app.continuous_mode {
                        // Auto restart scan in continuous mode with interval
                        let interval = app.scan_interval_secs;
                        let (start, end) = app.scan_range;
                        let tx_clone = tx.clone();
                        let oui_db_clone = oui_db.clone();
                        if interval > 0 {
                            app.scan_state = ScanState::Waiting;
                            app.wait_started = Some(Instant::now());
                        }
                        tokio::spawn(async move {
                            if interval > 0 {
                                tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
                            }
                            start_scan(tx_clone, start, end, oui_db_clone);
                        });
                    } else {
                        app.scan_state = ScanState::Idle;
                        app.status_message = format!("Found {} devices", app.devices.len());
                    }
                }
            }
        }

        // Tick spinner when scanning or waiting
        if matches!(app.scan_state, ScanState::Scanning | ScanState::Waiting) {
            app.tick_spinner();
        }

        // Handle events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle keys based on current view
                    match app.current_view {
                        View::Help => {
                            // Any key closes help
                            app.current_view = View::Main;
                        }
                        View::DeviceDetail => {
                            // Escape or Enter closes detail view
                            match key.code {
                                KeyCode::Esc | KeyCode::Enter => {
                                    app.current_view = View::Main;
                                }
                                _ => {}
                            }
                        }
                        View::Main => {
                            match key.code {
                                KeyCode::Char('q') => {
                                    app.should_quit = true;
                                }
                                KeyCode::Char('?') => {
                                    app.current_view = View::Help;
                                }
                                KeyCode::Enter => {
                                    if !app.devices.is_empty() {
                                        app.current_view = View::DeviceDetail;
                                        // Trigger vendor lookup if vendor is unknown but MAC is available
                                        if let Some(device) = app.selected_device() {
                                            if device.vendor.is_none() {
                                                if let Some(mac) = device.mac.clone() {
                                                    let ip = device.ip;
                                                    let tx_clone = tx.clone();
                                                    tokio::spawn(async move {
                                                        if let Some(vendor) = lookup_vendor_online(&mac).await {
                                                            let _ = tx_clone.send(ScanMessage::UpdateVendor { ip, vendor }).await;
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('c') => {
                                    app.continuous_mode = !app.continuous_mode;
                                    if app.continuous_mode {
                                        app.status_message = String::from("Continuous mode ON");
                                        if app.scan_state == ScanState::Idle {
                                            app.scan_state = ScanState::Scanning;
                                            let (start, end) = app.scan_range;
                                            start_scan(tx.clone(), start, end, oui_db.clone());
                                        }
                                    } else {
                                        app.status_message = String::from("Continuous mode OFF");
                                        if app.scan_state == ScanState::Waiting {
                                            app.scan_state = ScanState::Idle;
                                            app.wait_started = None;
                                        }
                                    }
                                }
                                KeyCode::Char('s') => {
                                    if app.scan_state == ScanState::Idle {
                                        app.clear_devices();

                                        let (start, end) = app.scan_range;
                                        start_scan(tx.clone(), start, end, oui_db.clone());
                                    }
                                }
                                KeyCode::Down | KeyCode::Char('j') => {
                                    app.select_next();
                                }
                                KeyCode::Up | KeyCode::Char('k') => {
                                    app.select_previous();
                                }
                                KeyCode::Char('+') | KeyCode::Char('=') => {
                                    app.scan_interval_secs = app.scan_interval_secs.saturating_add(1);
                                    app.status_message = format!("Interval: {}s", app.scan_interval_secs);
                                }
                                KeyCode::Char('-') | KeyCode::Char('_') => {
                                    app.scan_interval_secs = app.scan_interval_secs.saturating_sub(1);
                                    app.status_message = format!("Interval: {}s", app.scan_interval_secs);
                                }
                                KeyCode::Char('r') => {
                                    if app.scan_state == ScanState::Idle {
                                        app.open_range_input();
                                    }
                                }
                                _ => {}
                            }
                        }
                        View::RangeInput => {
                            match key.code {
                                KeyCode::Esc => {
                                    app.current_view = View::Main;
                                }
                                KeyCode::Enter => {
                                    app.apply_range_input();
                                }
                                KeyCode::Tab | KeyCode::Down | KeyCode::Up => {
                                    app.toggle_range_input_field();
                                }
                                KeyCode::Left => {
                                    app.range_input_cursor_left();
                                }
                                KeyCode::Right => {
                                    app.range_input_cursor_right();
                                }
                                KeyCode::Backspace => {
                                    app.range_input_pop();
                                }
                                KeyCode::Char(c) => {
                                    if c.is_ascii_digit() || c == '.' || c == '/' {
                                        app.range_input_push(c);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
