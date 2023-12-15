use std::cell::RefCell;

use bitflags::bitflags;
use crossbeam_channel::{Receiver, TryRecvError};

use crate::interfaces::Memory;
use crate::utils;

pub struct Controller {
    enabled: bool,
    buttons: ControllerButtons,
    keyboard_channel: Receiver<char>,
    controller_snapshot: RefCell<InnerController>,
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

impl Controller {
    pub fn new(keyboard: Receiver<char>, enabled: bool) -> Self {
        Self {
            enabled,
            buttons: ControllerButtons::default(),
            keyboard_channel: keyboard,
            controller_snapshot: RefCell::new(InnerController::empty()),
        }
    }

    pub fn connect(&mut self, buttons: ControllerButtons) {
        self.enabled = true;
        self.buttons = ControllerButtons {
            left: buttons.left.to_uppercase().next().unwrap(),
            down: buttons.down.to_uppercase().next().unwrap(),
            right: buttons.right.to_uppercase().next().unwrap(),
            up: buttons.up.to_uppercase().next().unwrap(),
            select: buttons.select.to_uppercase().next().unwrap(),
            start: buttons.start.to_uppercase().next().unwrap(),
            a: buttons.a.to_uppercase().next().unwrap(),
            b: buttons.b.to_uppercase().next().unwrap(),
        }
    }

    pub fn disconnect(&mut self) {
        self.enabled = false;
    }
}

impl Memory for Controller {
    fn read(&self, _address: u16) -> u8 {
        if !self.enabled {
            return 0;
        }

        let data = utils::bv(self.controller_snapshot.borrow().bits(), 7);
        let updated =
            InnerController::from_bits(self.controller_snapshot.borrow().bits() << 1).unwrap();
        *self.controller_snapshot.borrow_mut() = updated;
        // println!("[controller] read: {data:0>8b} updated: {updated:0>8b}");
        data
    }

    fn write(&mut self, _address: u16, _data: u8) {
        // TODO: if write = 1, signal the controller to poll it's input. write =
        // 0 to end polling. Read bit by bit

        // Read PISO (Parallel-In Serial-Out)
        let mut buffer = String::new();

        loop {
            match self.keyboard_channel.try_recv() {
                Ok(c) => buffer.push(c),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("Keyboard channel disconnected!"),
            }
        }

        if !self.enabled {
            return;
        }

        if buffer.is_empty() {
            return;
        }

        let mut input = InnerController::empty();
        for c in buffer.chars() {
            let c = c.to_uppercase().next().unwrap();
            if c == self.buttons.left {
                input.insert(InnerController::LEFT);
            } else if c == self.buttons.down {
                input.insert(InnerController::DOWN);
            } else if c == self.buttons.right {
                input.insert(InnerController::RIGHT);
            } else if c == self.buttons.up {
                input.insert(InnerController::UP);
            } else if c == self.buttons.select {
                input.insert(InnerController::SELECT);
            } else if c == self.buttons.start {
                input.insert(InnerController::START);
            } else if c == self.buttons.a {
                input.insert(InnerController::A);
            } else if c == self.buttons.b {
                input.insert(InnerController::B);
            } else {
                // ignore
            }
        }
        println!();

        *self.controller_snapshot.borrow_mut() = input;
        // println!("[controller] New controller: {:0>8b}", input.bits());
    }

    fn size(&self) -> usize {
        1
    }
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
