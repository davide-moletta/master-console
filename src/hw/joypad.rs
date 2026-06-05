pub enum Buttons {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

/// Emulates the physical buttons of the console
/// `selected` -> bits 4 & 5 are used to select the button to listen to
#[derive(Debug)]
pub struct Joypad {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    a: bool,
    b: bool,
    start: bool,
    select: bool,
    selected: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            up: false,
            down: false,
            left: false,
            right: false,
            a: false,
            b: false,
            start: false,
            select: false,
            selected: 0x30,
        }
    }

    /// Helper to interact with joypad
    pub fn set_button(&mut self, button: Buttons, val: bool) {
        match button {
            Buttons::Up => self.up = val,
            Buttons::Down => self.down = val,
            Buttons::Left => self.left = val,
            Buttons::Right => self.right = val,
            Buttons::A => self.a = val,
            Buttons::B => self.b = val,
            Buttons::Start => self.start = val,
            Buttons::Select => self.select = val,
        }
    }

    pub fn read(&self) -> u8 {
        let mut res = 0xCF | self.selected;

        if (self.selected & 0x10) == 0 {
            // Row 0: Directions
            if self.right {
                res &= !0x01;
            }
            if self.left {
                res &= !0x02;
            }
            if self.up {
                res &= !0x04;
            }
            if self.down {
                res &= !0x08;
            }
        }
        if (self.selected & 0x20) == 0 {
            // Row 1: Actions
            if self.a {
                res &= !0x01;
            }
            if self.b {
                res &= !0x02;
            }
            if self.select {
                res &= !0x04;
            }
            if self.start {
                res &= !0x08;
            }
        }
        res
    }

    pub fn write(&mut self, val: u8) {
        self.selected = val & 0x30;
    }
}
