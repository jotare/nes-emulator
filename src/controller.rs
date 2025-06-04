use std::cell::RefCell;

use bitflags::bitflags;

use crate::events::{KeyEvent, KeyboardListener};
use crate::interfaces::Memory;
use crate::utils;

/// Standard NES controllers
pub struct Controllers {
    one: Controller,
    two: Controller,
    keyboard_listener: KeyboardListener,
}

struct Controller {
    enabled: bool,
    buttons: ControllerButtons,

    /// Controller port latch. When the controller is polled, this value gets
    /// populated and then values are read and removed one by one
    port_latch: RefCell<InnerController>,

    /// Last controller state we've seen. This value gets updated when the
    /// controller is polled.
    snapshot: InnerController,
}

pub struct ControllerButtons {
    pub left: char,
    pub down: char,
    pub right: char,
    pub up: char,
    pub select: char,
    pub start: char,
    pub a: char,
    pub b: char,
}

bitflags! {
    struct InnerController: u8 {
        const A = 0b1000_0000;
        const B = 0b0100_0000;
        const SELECT = 0b0010_0000;
        const START = 0b0001_0000;
        const UP = 0b0000_1000;
        const DOWN = 0b0000_0100;
        const LEFT = 0b0000_0010;
        const RIGHT = 0b0000_0001;
    }
}

struct PressedButtons(InnerController);
struct ReleasedButtons(InnerController);

impl Controllers {
    pub fn new(keyboard: KeyboardListener) -> Self {
        Self {
            one: Controller {
                enabled: false,
                // this is a placeholder
                buttons: ControllerButtons::default(),
                port_latch: RefCell::new(InnerController::empty()),
                snapshot: InnerController::empty(),
            },
            two: Controller {
                enabled: false,
                // this is a placeholder
                buttons: ControllerButtons::default(),
                port_latch: RefCell::new(InnerController::empty()),
                snapshot: InnerController::empty(),
            },
            keyboard_listener: keyboard,
        }
    }

    pub fn connect_controller_one(&mut self, buttons: ControllerButtons) {
        self.one.enabled = true;
        self.one.buttons = buttons.to_ascii_uppercase();
    }

    pub fn disconnect_controller_one(&mut self) {
        self.one.enabled = false;
    }

    pub fn connect_controller_two(&mut self, buttons: ControllerButtons) {
        self.two.enabled = true;
        self.two.buttons = buttons.to_ascii_uppercase();
    }

    pub fn disconnect_controller_two(&mut self) {
        self.two.enabled = false;
    }
}

impl Memory for Controllers {
    fn read(&self, address: u16) -> u8 {
        let address = 0x4016 + address;
        let controller = match address {
            0x4016 => &self.one,
            0x4017 => &self.two,
            _ => unreachable!("NES hardware setup error"),
        };

        if !controller.enabled {
            return 0;
        }

        // read and shift a single bit from the controller port latch
        let latch = controller.port_latch.borrow().bits();
        let data = utils::bv(latch, 7);
        *controller.port_latch.borrow_mut() = InnerController::from_bits(latch << 1).unwrap();
        data
    }

    fn write(&mut self, address: u16, data: u8) {
        let address = 0x4016 + address;

        // this is indeed writing to an APU register, not a controller xD
        if address == 0x4017 {
            // println!("Controller register $4017 is not writable. Why writing {data}?");
            return;
        }
        assert_eq!(address, 0x4016, "NES hardware setup error");

        if data & 0x01 == 1 {
            // poll controller input. As we have it buffered in our keyboard
            // listener, we don't need to do anything
        } else {
            // End of polling, update controller snapshots
            //
            // Read PISO (Parallel-In Serial-Out)

            let input = self.keyboard_listener.read();

            if !input.is_empty() {
                if self.one.enabled {
                    let (pressed, released) = self.one.buttons.parse_input(&input);
                    let state = self.one.snapshot.difference(released.0).union(pressed.0);
                    self.one.snapshot = state;
                }

                if self.two.enabled {
                    let (pressed, released) = self.two.buttons.parse_input(&input);
                    let state = self.two.snapshot.difference(released.0).union(pressed.0);
                    self.two.snapshot = state;
                }
            }

            // fill the latch with the last state we've seen (updated or not)
            if self.one.enabled {
                *self.one.port_latch.borrow_mut() = self.one.snapshot;
            }
            if self.two.enabled {
                *self.two.port_latch.borrow_mut() = self.two.snapshot;
            }
        }
    }

    fn size(&self) -> usize {
        2
    }
}

impl ControllerButtons {
    /// Makes a copy of the controller buttons with all key characters converted
    /// to its ASCII upper case equivalent. Non ASCII characters are unchanged
    fn to_ascii_uppercase(&self) -> Self {
        Self {
            left: self.left.to_ascii_uppercase(),
            down: self.down.to_ascii_uppercase(),
            right: self.right.to_ascii_uppercase(),
            up: self.up.to_ascii_uppercase(),
            select: self.select.to_ascii_uppercase(),
            start: self.start.to_ascii_uppercase(),
            a: self.a.to_ascii_uppercase(),
            b: self.b.to_ascii_uppercase(),
        }
    }

    /// Parse contorller input and return the pressed and released buttons for
    /// this controller
    fn parse_input(&self, input: &[KeyEvent]) -> (PressedButtons, ReleasedButtons) {
        let mut pressed = InnerController::empty();
        let mut released = InnerController::empty();

        for ev in input {
            let c = ev.get_char().to_uppercase().next().unwrap();
            let button;
            if c == self.left {
                button = InnerController::LEFT;
            } else if c == self.down {
                button = InnerController::DOWN;
            } else if c == self.right {
                button = InnerController::RIGHT;
            } else if c == self.up {
                button = InnerController::UP;
            } else if c == self.select {
                button = InnerController::SELECT;
            } else if c == self.start {
                button = InnerController::START;
            } else if c == self.a {
                button = InnerController::A;
            } else if c == self.b {
                button = InnerController::B;
            } else {
                // ignore
                continue;
            }

            match ev {
                KeyEvent::Pressed(_) => {
                    pressed.insert(button);
                    released.remove(button);
                }
                KeyEvent::Released(_) => {
                    released.insert(button);
                    pressed.remove(button);
                }
            }
        }

        (PressedButtons(pressed), ReleasedButtons(released))
    }
}

impl Default for ControllerButtons {
    fn default() -> Self {
        Self {
            left: 'S',
            down: 'D',
            right: 'F',
            up: 'E',
            select: 'G',
            start: 'H',
            a: 'J',
            b: 'K',
        }
    }
}
