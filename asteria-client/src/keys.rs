/// Common Linux input key codes
/// These correspond to the constants defined in linux/input-event-codes.h

#[allow(dead_code)]
pub mod key_codes {
    pub const KEY_ESC: u32 = 1;
    pub const KEY_1: u32 = 2;
    pub const KEY_2: u32 = 3;
    pub const KEY_3: u32 = 4;
    pub const KEY_4: u32 = 5;
    pub const KEY_5: u32 = 6;
    pub const KEY_6: u32 = 7;
    pub const KEY_7: u32 = 8;
    pub const KEY_8: u32 = 9;
    pub const KEY_9: u32 = 10;
    pub const KEY_0: u32 = 11;
    pub const KEY_MINUS: u32 = 12;
    pub const KEY_EQUAL: u32 = 13;
    pub const KEY_BACKSPACE: u32 = 14;
    pub const KEY_TAB: u32 = 15;
    pub const KEY_Q: u32 = 16;
    pub const KEY_W: u32 = 17;
    pub const KEY_E: u32 = 18;
    pub const KEY_R: u32 = 19;
    pub const KEY_T: u32 = 20;
    pub const KEY_Y: u32 = 21;
    pub const KEY_U: u32 = 22;
    pub const KEY_I: u32 = 23;
    pub const KEY_O: u32 = 24;
    pub const KEY_P: u32 = 25;
    pub const KEY_LEFTBRACE: u32 = 26;
    pub const KEY_RIGHTBRACE: u32 = 27;
    pub const KEY_ENTER: u32 = 28;
    pub const KEY_LEFTCTRL: u32 = 29;
    pub const KEY_A: u32 = 30;
    pub const KEY_S: u32 = 31;
    pub const KEY_D: u32 = 32;
    pub const KEY_F: u32 = 33;
    pub const KEY_G: u32 = 34;
    pub const KEY_H: u32 = 35;
    pub const KEY_J: u32 = 36;
    pub const KEY_K: u32 = 37;
    pub const KEY_L: u32 = 38;
    pub const KEY_SEMICOLON: u32 = 39;
    pub const KEY_APOSTROPHE: u32 = 40;
    pub const KEY_GRAVE: u32 = 41;
    pub const KEY_LEFTSHIFT: u32 = 42;
    pub const KEY_BACKSLASH: u32 = 43;
    pub const KEY_Z: u32 = 44;
    pub const KEY_X: u32 = 45;
    pub const KEY_C: u32 = 46;
    pub const KEY_V: u32 = 47;
    pub const KEY_B: u32 = 48;
    pub const KEY_N: u32 = 49;
    pub const KEY_M: u32 = 50;
    pub const KEY_COMMA: u32 = 51;
    pub const KEY_DOT: u32 = 52;
    pub const KEY_SLASH: u32 = 53;
    pub const KEY_RIGHTSHIFT: u32 = 54;
    pub const KEY_KPASTERISK: u32 = 55;
    pub const KEY_LEFTALT: u32 = 56;
    pub const KEY_SPACE: u32 = 57;
    pub const KEY_CAPSLOCK: u32 = 58;
    pub const KEY_F1: u32 = 59;
    pub const KEY_F2: u32 = 60;
    pub const KEY_F3: u32 = 61;
    pub const KEY_F4: u32 = 62;
    pub const KEY_F5: u32 = 63;
    pub const KEY_F6: u32 = 64;
    pub const KEY_F7: u32 = 65;
    pub const KEY_F8: u32 = 66;
    pub const KEY_F9: u32 = 67;
    pub const KEY_F10: u32 = 68;
    pub const KEY_NUMLOCK: u32 = 69;
    pub const KEY_SCROLLLOCK: u32 = 70;
    pub const KEY_F11: u32 = 87;
    pub const KEY_F12: u32 = 88;
    pub const KEY_RIGHTCTRL: u32 = 97;
    pub const KEY_RIGHTALT: u32 = 100;
    pub const KEY_HOME: u32 = 102;
    pub const KEY_UP: u32 = 103;
    pub const KEY_PAGEUP: u32 = 104;
    pub const KEY_LEFT: u32 = 105;
    pub const KEY_RIGHT: u32 = 106;
    pub const KEY_END: u32 = 107;
    pub const KEY_DOWN: u32 = 108;
    pub const KEY_PAGEDOWN: u32 = 109;
    pub const KEY_INSERT: u32 = 110;
    pub const KEY_DELETE: u32 = 111;
    pub const KEY_LEFTMETA: u32 = 125;
    pub const KEY_RIGHTMETA: u32 = 126;
}

/// Get a human-readable name for a key code
pub fn key_name(key_code: u32) -> &'static str {
    match key_code {
        key_codes::KEY_ESC => "Escape",
        key_codes::KEY_LEFTCTRL => "Left Ctrl",
        key_codes::KEY_RIGHTCTRL => "Right Ctrl",
        key_codes::KEY_LEFTALT => "Left Alt",
        key_codes::KEY_RIGHTALT => "Right Alt",
        key_codes::KEY_LEFTSHIFT => "Left Shift",
        key_codes::KEY_RIGHTSHIFT => "Right Shift",
        key_codes::KEY_LEFTMETA => "Left Meta/Super",
        key_codes::KEY_RIGHTMETA => "Right Meta/Super",
        key_codes::KEY_SPACE => "Space",
        key_codes::KEY_ENTER => "Enter",
        key_codes::KEY_TAB => "Tab",
        key_codes::KEY_BACKSPACE => "Backspace",
        key_codes::KEY_DELETE => "Delete",
        key_codes::KEY_F1 => "F1",
        key_codes::KEY_F2 => "F2",
        key_codes::KEY_F3 => "F3",
        key_codes::KEY_F4 => "F4",
        key_codes::KEY_F5 => "F5",
        key_codes::KEY_F6 => "F6",
        key_codes::KEY_F7 => "F7",
        key_codes::KEY_F8 => "F8",
        key_codes::KEY_F9 => "F9",
        key_codes::KEY_F10 => "F10",
        key_codes::KEY_F11 => "F11",
        key_codes::KEY_F12 => "F12",
        _ => "Unknown",
    }
}
