use crate::hw;

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

    /// Set the specified button to true
    pub fn set_button(&mut self, button: hw::Buttons) {
        match button {
            hw::Buttons::Up => self.up = true,
            hw::Buttons::Down => self.down = true,
            hw::Buttons::Left => self.left = true,
            hw::Buttons::Right => self.right = true,
            hw::Buttons::A => self.a = true,
            hw::Buttons::B => self.b = true,
            hw::Buttons::Start => self.start = true,
            hw::Buttons::Select => self.select = true,
        }
    }

    /// Unset all buttons
    pub fn unset_buttons(&mut self) {
        self.up = false;
        self.down = false;
        self.left = false;
        self.right = false;
        self.a = false;
        self.b = false;
        self.start = false;
        self.select = false;
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
