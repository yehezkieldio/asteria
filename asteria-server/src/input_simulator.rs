use anyhow::Result;
use asteria_core::protocol::{InputEvent, InputEventType};
use enigo::{Axis, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use tracing::debug;

/// Input simulator that translates protocol events into system input
pub struct InputSimulator {
    enigo: Enigo,
}

impl InputSimulator {
    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default())?;
        Ok(Self { enigo })
    }

    /// Simulate input based on the received event
    pub fn simulate_input(&mut self, event: &InputEvent) -> Result<()> {
        debug!("Simulating input event: {:?}", event);

        // Convert Linux input event codes to actions
        match event.event_type.as_str() {
            "EV_KEY" => self.handle_key_event(event.code, event.value),
            "EV_REL" => self.handle_relative_event(event.code, event.value),
            "EV_ABS" => self.handle_absolute_event(event.code, event.value),
            _ => {
                debug!("Unsupported event type: {}", event.event_type);
                Ok(())
            }
        }
    }

    /// Simulate input based on typed event
    pub fn simulate_typed_input(&mut self, event: &InputEventType) -> Result<()> {
        debug!("Simulating typed input event: {:?}", event);

        match event {
            InputEventType::KeyPress { key_code } => {
                if let Some(key) = self.linux_key_to_enigo(*key_code) {
                    self.enigo.key(key, Direction::Press)?;
                }
            }
            InputEventType::KeyRelease { key_code } => {
                if let Some(key) = self.linux_key_to_enigo(*key_code) {
                    self.enigo.key(key, Direction::Release)?;
                }
            }
            InputEventType::MouseMove { x, y } => {
                self.enigo.move_mouse(*x, *y, enigo::Coordinate::Rel)?;
            }
            InputEventType::MouseButton { button, pressed } => {
                let mouse_button = match button {
                    0 => enigo::Button::Left,
                    1 => enigo::Button::Right,
                    2 => enigo::Button::Middle,
                    _ => return Ok(()),
                };

                let direction = if *pressed {
                    Direction::Press
                } else {
                    Direction::Release
                };

                self.enigo.button(mouse_button, direction)?;
            }
            InputEventType::MouseScroll { dx, dy } => {
                if *dx != 0 {
                    self.enigo.scroll(*dx, Axis::Horizontal)?;
                }
                if *dy != 0 {
                    self.enigo.scroll(*dy, Axis::Vertical)?;
                }
            }
        }

        Ok(())
    }

    /// Handle Linux key events (EV_KEY)
    fn handle_key_event(&mut self, code: u16, value: i32) -> Result<()> {
        let direction = match value {
            0 => Direction::Release,
            1 => Direction::Press,
            2 => return Ok(()), // Key repeat - ignore for now
            _ => return Ok(()),
        };

        if let Some(key) = self.linux_key_to_enigo(code) {
            self.enigo.key(key, direction)?;
        } else {
            debug!("Unknown key code: {}", code);
        }

        Ok(())
    }

    /// Handle Linux relative events (EV_REL) - mouse movement and scroll
    fn handle_relative_event(&mut self, code: u16, value: i32) -> Result<()> {
        match code {
            0 => {
                // REL_X - mouse X movement
                self.enigo.move_mouse(value, 0, enigo::Coordinate::Rel)?;
            }
            1 => {
                // REL_Y - mouse Y movement
                self.enigo.move_mouse(0, value, enigo::Coordinate::Rel)?;
            }
            8 => {
                // REL_WHEEL - scroll wheel
                self.enigo.scroll(value, Axis::Vertical)?;
            }
            6 => {
                // REL_HWHEEL - horizontal scroll
                self.enigo.scroll(value, Axis::Horizontal)?;
            }
            _ => {
                debug!("Unsupported relative event code: {}", code);
            }
        }

        Ok(())
    }

    /// Handle Linux absolute events (EV_ABS) - touchpad/touch input
    fn handle_absolute_event(&mut self, code: u16, value: i32) -> Result<()> {
        match code {
            0 => {
                // ABS_X - absolute X position
                // For now, treat as relative movement
                // In a real implementation, you'd need to track the previous position
                debug!("Absolute X position: {}", value);
            }
            1 => {
                // ABS_Y - absolute Y position
                debug!("Absolute Y position: {}", value);
            }
            _ => {
                debug!("Unsupported absolute event code: {}", code);
            }
        }

        Ok(())
    }

    /// Convert Linux key codes to Enigo Key enum
    fn linux_key_to_enigo(&self, code: u16) -> Option<Key> {
        match code {
            // Letters
            30 => Some(Key::Unicode('a')),
            48 => Some(Key::Unicode('b')),
            46 => Some(Key::Unicode('c')),
            32 => Some(Key::Unicode('d')),
            18 => Some(Key::Unicode('e')),
            33 => Some(Key::Unicode('f')),
            34 => Some(Key::Unicode('g')),
            35 => Some(Key::Unicode('h')),
            23 => Some(Key::Unicode('i')),
            36 => Some(Key::Unicode('j')),
            37 => Some(Key::Unicode('k')),
            38 => Some(Key::Unicode('l')),
            50 => Some(Key::Unicode('m')),
            49 => Some(Key::Unicode('n')),
            24 => Some(Key::Unicode('o')),
            25 => Some(Key::Unicode('p')),
            16 => Some(Key::Unicode('q')),
            19 => Some(Key::Unicode('r')),
            31 => Some(Key::Unicode('s')),
            20 => Some(Key::Unicode('t')),
            22 => Some(Key::Unicode('u')),
            47 => Some(Key::Unicode('v')),
            17 => Some(Key::Unicode('w')),
            45 => Some(Key::Unicode('x')),
            21 => Some(Key::Unicode('y')),
            44 => Some(Key::Unicode('z')),

            // Numbers
            2 => Some(Key::Unicode('1')),
            3 => Some(Key::Unicode('2')),
            4 => Some(Key::Unicode('3')),
            5 => Some(Key::Unicode('4')),
            6 => Some(Key::Unicode('5')),
            7 => Some(Key::Unicode('6')),
            8 => Some(Key::Unicode('7')),
            9 => Some(Key::Unicode('8')),
            10 => Some(Key::Unicode('9')),
            11 => Some(Key::Unicode('0')),

            // Special keys
            57 => Some(Key::Space),
            28 => Some(Key::Return),
            1 => Some(Key::Escape),
            14 => Some(Key::Backspace),
            15 => Some(Key::Tab),
            42 => Some(Key::Shift),
            54 => Some(Key::Shift), // Right shift
            29 => Some(Key::Control),
            97 => Some(Key::Control), // Right control
            56 => Some(Key::Alt),
            100 => Some(Key::Alt), // Right alt

            // Arrow keys
            103 => Some(Key::UpArrow),
            108 => Some(Key::DownArrow),
            105 => Some(Key::LeftArrow),
            106 => Some(Key::RightArrow),

            // Function keys
            59 => Some(Key::F1),
            60 => Some(Key::F2),
            61 => Some(Key::F3),
            62 => Some(Key::F4),
            63 => Some(Key::F5),
            64 => Some(Key::F6),
            65 => Some(Key::F7),
            66 => Some(Key::F8),
            67 => Some(Key::F9),
            68 => Some(Key::F10),
            87 => Some(Key::F11),
            88 => Some(Key::F12),

            // Mouse buttons (handled as buttons, but included for completeness)
            272 => None, // BTN_LEFT
            273 => None, // BTN_RIGHT
            274 => None, // BTN_MIDDLE

            _ => {
                debug!("Unmapped key code: {}", code);
                None
            }
        }
    }
}

impl Default for InputSimulator {
    fn default() -> Self {
        Self::new().expect("Failed to create input simulator")
    }
}
