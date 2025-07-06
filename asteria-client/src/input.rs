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
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::network::NetworkClient;

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
}

impl InputCapture {
    pub fn new() -> Result<Self> {
        let mut libinput = Libinput::new_with_udev(Interface);

        if let Err(e) = libinput.udev_assign_seat("seat0") {
            error!("Failed to assign seat: {:?}", e);
            return Err(anyhow::anyhow!("Failed to assign seat: {:?}", e));
        }

        info!("Successfully initialized libinput and assigned seat");
        Ok(Self { libinput })
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

        loop {
            // Dispatch libinput events
            if let Err(e) = self.libinput.dispatch() {
                error!("libinput dispatch error: {:?}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                continue;
            }

            // Process all available events
            while let Some(event) = self.libinput.next() {
                if let Some(packet) = self.convert_event_to_packet(event) {
                    if let Err(e) = packet_sender.send(packet).await {
                        error!("Failed to send packet: {}", e);
                        return Err(anyhow::anyhow!("Packet sender channel closed"));
                    }
                }
            }

            // Yield control to allow other tasks to run
            tokio::task::yield_now().await;
        }
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
}

impl Default for InputCapture {
    fn default() -> Self {
        Self::new().expect("Failed to create input capture")
    }
}
