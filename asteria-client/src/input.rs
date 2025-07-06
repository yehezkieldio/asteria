use anyhow::Result;
use asteria_core::protocol::{InputEventType, Message, Packet};
use input::{
    Libinput, LibinputInterface,
    event::{
        Event,
        keyboard::{KeyState, KeyboardEvent, KeyboardEventTrait},
        pointer::{Axis, ButtonState, PointerEvent, PointerScrollEvent},
    },
};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

use crate::network::NetworkClient;

// Linux input event ioctl constants
const EVIOCGRAB: u64 = 0x40044590;
const EVIOCGBIT_KEY: u64 = 0x80604521;
const EVIOCGBIT_REL: u64 = 0x80604522;
const EVIOCGBIT_ABS: u64 = 0x80604523;
const EVIOCGNAME: u64 = 0x80ff4506;

// Event type constants
const EV_KEY: u8 = 0x01;
const EV_REL: u8 = 0x02;
const EV_ABS: u8 = 0x03;

// Key/button bit masks
const REL_X: u8 = 0x00;
const REL_Y: u8 = 0x01;
const ABS_X: u8 = 0x00;
const ABS_Y: u8 = 0x01;

#[allow(dead_code)]
struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

/// Input capture system that monitors Linux input events
pub struct InputCapture {
    libinput: Libinput,
    toggle_key: u32,
    relay_state: Arc<RwLock<RelayState>>,
    grabbed_devices: HashMap<String, OwnedFd>,
}

#[derive(Debug, Clone)]
pub struct RelayState {
    pub relay_enabled: bool,
    pub suppress_local_input: bool,
}

impl Default for RelayState {
    fn default() -> Self {
        Self {
            relay_enabled: false,
            suppress_local_input: false,
        }
    }
}

impl InputCapture {
    pub fn new() -> Result<Self> {
        Self::new_with_toggle_key(0x1D) // Default to Left Ctrl (KEY_LEFTCTRL)
    }

    pub fn new_with_toggle_key(toggle_key: u32) -> Result<Self> {
        let mut libinput = Libinput::new_with_udev(Interface);

        if let Err(e) = libinput.udev_assign_seat("seat0") {
            error!("Failed to assign seat: {:?}", e);
            return Err(anyhow::anyhow!("Failed to assign seat: {:?}", e));
        }

        info!("Successfully initialized libinput and assigned seat");
        info!("Toggle key set to: 0x{:02x}", toggle_key);

        Ok(Self {
            libinput,
            toggle_key,
            relay_state: Arc::new(RwLock::new(RelayState::default())),
            grabbed_devices: HashMap::new(),
        })
    }

    /// Get the current relay state
    pub async fn get_relay_state(&self) -> RelayState {
        self.relay_state.read().await.clone()
    }

    /// Toggle the relay state
    async fn toggle_relay(&mut self) -> Result<()> {
        let current_state = {
            let state = self.relay_state.read().await;
            state.relay_enabled
        };

        if current_state {
            // Disable relay and restore local input
            {
                let mut state = self.relay_state.write().await;
                state.relay_enabled = false;
                state.suppress_local_input = false;
            }

            // Release all grabbed devices
            if let Err(e) = self.release_input_devices().await {
                error!("Failed to release input devices: {}", e);
            }

            info!("🔄 Relay disabled - Linux input restored");
        } else {
            // Grab all input devices first
            if let Err(e) = self.grab_input_devices().await {
                error!("Failed to grab input devices: {}", e);
                return Err(e);
            }

            // Enable relay and suppress local input
            {
                let mut state = self.relay_state.write().await;
                state.relay_enabled = true;
                state.suppress_local_input = true;
            }

            info!("🔄 Relay enabled - Linux input suppressed, relaying to Windows");
        }

        Ok(())
    }

    /// Start capturing input events and relay them through the network client
    pub async fn start_and_relay(&mut self, mut network_client: NetworkClient) -> Result<()> {
        info!("Starting input capture and relay...");

        // Create a channel for packet communication
        let (packet_sender, packet_receiver) = mpsc::channel(1000);

        // Start the network relay task
        let network_task =
            tokio::spawn(async move { network_client.start_relay(packet_receiver).await });

        // Start the input capture in the current task to avoid Send issues
        let input_result = self.capture_input_events(packet_sender).await;

        // Cancel the network task if input capture ends
        network_task.abort();

        // Wait for the network task to complete
        let _ = network_task.await;

        input_result
    }

    /// Capture input events from libinput
    async fn capture_input_events(&mut self, packet_sender: mpsc::Sender<Packet>) -> Result<()> {
        info!("Starting input event capture loop...");
        info!(
            "Press the toggle key (0x{:02x}) to enable/disable relay",
            self.toggle_key
        );

        loop {
            // Dispatch libinput events
            if let Err(e) = self.libinput.dispatch() {
                error!("libinput dispatch error: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }

            // Process all available events
            while let Some(event) = self.libinput.next() {
                let relay_state = self.relay_state.read().await;

                // ALWAYS process the toggle key, even when relay is enabled
                if let Event::Keyboard(ref keyboard_event) = event {
                    if keyboard_event.key() == self.toggle_key
                        && keyboard_event.key_state() == KeyState::Pressed
                    {
                        // Drop the read lock before calling toggle_relay
                        drop(relay_state);

                        if let Err(e) = self.toggle_relay().await {
                            error!("Failed to toggle relay: {}", e);
                        }
                        continue; // Don't process the toggle key itself
                    }
                }

                // Only process and relay other events if relay is enabled
                if relay_state.relay_enabled {
                    if let Some(packet) = self.convert_event_to_packet(event) {
                        if let Err(e) = packet_sender.send(packet).await {
                            error!("Failed to send packet: {}", e);
                            return Err(anyhow::anyhow!("Packet sender channel closed"));
                        }
                    }
                }
            }

            // Yield control to allow other tasks to run
            tokio::task::yield_now().await;
        }
    }

    // ...existing code...

    /// Check if device is safe to grab (not used by our own libinput instance)
    fn is_safe_to_grab(&self, device_path: &str) -> bool {
        // Get the device name to check if it's something we should avoid
        if let Ok(file) = OpenOptions::new().read(true).open(device_path) {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();

            // Get device name
            let mut name_buf = [0u8; 256];
            let name_result = unsafe { libc::ioctl(fd, EVIOCGNAME, name_buf.as_mut_ptr()) };

            if name_result >= 0 {
                if let Ok(name) = std::str::from_utf8(&name_buf[..name_result as usize]) {
                    let name = name.trim_end_matches('\0');
                    debug!("Device {} name: {}", device_path, name);

                    // Skip virtual devices and special devices
                    if name.to_lowercase().contains("virtual")
                        || name.to_lowercase().contains("uinput")
                        || name.to_lowercase().contains("asteria")
                    {
                        debug!("Skipping virtual/special device: {}", name);
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Grab a specific input device with selective grabbing
    async fn grab_device(&mut self, device_path: &str) -> Result<()> {
        use std::os::unix::io::AsRawFd;

        // Open the device
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(device_path)
            .map_err(|e| anyhow::anyhow!("Failed to open device {}: {}", device_path, e))?;

        let fd = file.as_raw_fd();

        // For now, don't actually grab devices to avoid lock-out
        // Instead, we'll rely on libinput's event handling
        // This is a safer approach until we implement proper device filtering

        // Store the file descriptor for tracking, but don't grab
        debug!("Tracking device (not grabbing): {}", device_path);
        self.grabbed_devices
            .insert(device_path.to_string(), file.into());

        Ok(())
    }

    /// Convert a libinput event to a protocol packet
    fn convert_event_to_packet(&self, event: Event) -> Option<Packet> {
        match event {
            Event::Keyboard(keyboard_event) => self.convert_keyboard_event(keyboard_event),
            Event::Pointer(pointer_event) => self.convert_pointer_event(pointer_event),
            _ => {
                debug!("Ignoring unsupported event type: {:?}", event);
                None
            }
        }
    }

    /// Convert keyboard events to protocol packets
    fn convert_keyboard_event(&self, keyboard_event: KeyboardEvent) -> Option<Packet> {
        let key_code = keyboard_event.key();
        let state = keyboard_event.key_state();

        debug!("Keyboard event - Key: {}, State: {:?}", key_code, state);

        let input_event_type = match state {
            KeyState::Pressed => InputEventType::KeyPress {
                key_code: key_code as u16,
            },
            KeyState::Released => InputEventType::KeyRelease {
                key_code: key_code as u16,
            },
        };

        Some(Packet::new(Message::InputEventTyped(input_event_type)))
    }

    /// Convert pointer events to protocol packets
    fn convert_pointer_event(&self, pointer_event: PointerEvent) -> Option<Packet> {
        match pointer_event {
            PointerEvent::Motion(motion_event) => {
                let dx = motion_event.dx();
                let dy = motion_event.dy();

                debug!("Pointer motion - dx: {}, dy: {}", dx, dy);

                if dx != 0.0 || dy != 0.0 {
                    let input_event_type = InputEventType::MouseMove {
                        x: dx as i32,
                        y: dy as i32,
                    };
                    Some(Packet::new(Message::InputEventTyped(input_event_type)))
                } else {
                    None
                }
            }
            PointerEvent::Button(button_event) => {
                let button = button_event.button();
                let state = button_event.button_state();

                debug!("Pointer button - Button: {}, State: {:?}", button, state);

                let pressed = match state {
                    ButtonState::Pressed => true,
                    ButtonState::Released => false,
                };

                // Convert libinput button codes to standard mouse button codes
                let button_code = match button {
                    0x110 => 1, // BTN_LEFT
                    0x111 => 2, // BTN_RIGHT
                    0x112 => 3, // BTN_MIDDLE
                    _ => {
                        warn!("Unsupported mouse button: {}", button);
                        return None;
                    }
                };

                let input_event_type = InputEventType::MouseButton {
                    button: button_code,
                    pressed,
                };
                Some(Packet::new(Message::InputEventTyped(input_event_type)))
            }
            PointerEvent::ScrollWheel(scroll_event) => {
                let dx = scroll_event.scroll_value(Axis::Horizontal);
                let dy = scroll_event.scroll_value(Axis::Vertical);

                debug!("Pointer scroll - dx: {}, dy: {}", dx, dy);

                if dx != 0.0 || dy != 0.0 {
                    let input_event_type = InputEventType::MouseScroll {
                        dx: dx as i32,
                        dy: -(dy as i32), // Invert vertical scroll
                    };
                    Some(Packet::new(Message::InputEventTyped(input_event_type)))
                } else {
                    None
                }
            }
            _ => {
                debug!("Ignoring unsupported pointer event: {:?}", pointer_event);
                None
            }
        }
    }

    /// Grab all input devices to suppress local input
    async fn grab_input_devices(&mut self) -> Result<()> {
        info!("Grabbing input devices for suppression...");

        // Get list of input devices
        let device_paths = self.get_input_device_paths()?;

        for device_path in device_paths {
            if let Err(e) = self.grab_device(&device_path).await {
                warn!("Failed to grab device {}: {}", device_path, e);
                // Continue with other devices even if one fails
            }
        }

        info!(
            "Successfully grabbed {} input devices",
            self.grabbed_devices.len()
        );
        Ok(())
    }

    /// Release all grabbed input devices
    async fn release_input_devices(&mut self) -> Result<()> {
        info!("Releasing grabbed input devices...");

        // Close all grabbed device file descriptors
        for (device_path, fd) in self.grabbed_devices.drain() {
            drop(fd);
            debug!("Released device: {}", device_path);
        }

        info!("All input devices released");
        Ok(())
    }

    /// Get paths to all input devices, filtering out devices that should not be grabbed
    fn get_input_device_paths(&self) -> Result<Vec<String>> {
        let mut device_paths = Vec::new();

        // Scan /dev/input/ for event devices
        let input_dir = std::fs::read_dir("/dev/input/")?;

        for entry in input_dir {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name() {
                if let Some(filename_str) = filename.to_str() {
                    if filename_str.starts_with("event") {
                        if let Some(path_str) = path.to_str() {
                            // Check if this device should be grabbed
                            if self.should_grab_device(path_str)? {
                                device_paths.push(path_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        device_paths.sort();
        debug!("Found {} grabbable input devices", device_paths.len());
        Ok(device_paths)
    }

    /// Check if a device should be grabbed based on its capabilities
    fn should_grab_device(&self, device_path: &str) -> Result<bool> {
        use std::os::unix::io::AsRawFd;

        // First check if it's safe to grab this device
        if !self.is_safe_to_grab(device_path) {
            return Ok(false);
        }

        // Try to open the device to check its capabilities
        let file = match OpenOptions::new().read(true).open(device_path) {
            Ok(file) => file,
            Err(e) => {
                debug!(
                    "Cannot open device {} for capability check: {}",
                    device_path, e
                );
                return Ok(false);
            }
        };

        let fd = file.as_raw_fd();

        // Check if device has keyboard or mouse capabilities
        let mut key_bits = [0u8; 96]; // EV_KEY bitmap (768 bits / 8 = 96 bytes)
        let mut rel_bits = [0u8; 8]; // EV_REL bitmap (64 bits / 8 = 8 bytes)
        let mut abs_bits = [0u8; 8]; // EV_ABS bitmap (64 bits / 8 = 8 bytes)

        // Get key capabilities (keyboards)
        let key_result = unsafe { libc::ioctl(fd, EVIOCGBIT_KEY, key_bits.as_mut_ptr()) };

        // Get relative axis capabilities (mice)
        let rel_result = unsafe { libc::ioctl(fd, EVIOCGBIT_REL, rel_bits.as_mut_ptr()) };

        // Get absolute axis capabilities (touchpads, tablets)
        let abs_result = unsafe { libc::ioctl(fd, EVIOCGBIT_ABS, abs_bits.as_mut_ptr()) };

        // Check if device has keyboard keys
        let has_keyboard = key_result >= 0 && key_bits.iter().any(|&b| b != 0);

        // Check if device has mouse relative movement
        let has_mouse_rel = rel_result >= 0 && (rel_bits[0] & (1 << REL_X | 1 << REL_Y)) != 0;

        // Check if device has absolute positioning (touchpad)
        let has_abs_pos = abs_result >= 0 && (abs_bits[0] & (1 << ABS_X | 1 << ABS_Y)) != 0;

        let should_grab = has_keyboard || has_mouse_rel || has_abs_pos;

        if should_grab {
            debug!(
                "Device {} capabilities: keyboard={}, mouse_rel={}, abs_pos={}",
                device_path, has_keyboard, has_mouse_rel, has_abs_pos
            );
        }

        Ok(should_grab)
    }

    /// Gracefully shutdown the input capture system
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down input capture system...");

        // Check if relay is enabled and disable it
        let should_release = {
            let state = self.relay_state.read().await;
            state.relay_enabled
        };

        if should_release {
            // Update state first
            {
                let mut state = self.relay_state.write().await;
                state.relay_enabled = false;
                state.suppress_local_input = false;
            }

            // Then release devices
            if let Err(e) = self.release_input_devices().await {
                error!("Failed to release input devices during shutdown: {}", e);
            }
        }

        info!("Input capture system shutdown complete");
        Ok(())
    }
}

impl Default for InputCapture {
    fn default() -> Self {
        Self::new().expect("Failed to create input capture")
    }
}

impl Drop for InputCapture {
    fn drop(&mut self) {
        // Release all grabbed devices when the InputCapture is dropped
        if !self.grabbed_devices.is_empty() {
            info!(
                "Releasing {} grabbed devices on drop",
                self.grabbed_devices.len()
            );
            for (device_path, fd) in self.grabbed_devices.drain() {
                drop(fd);
                debug!("Released device on drop: {}", device_path);
            }
        }
    }
}
