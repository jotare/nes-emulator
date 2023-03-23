use std::cell::RefCell;

use bitflags::bitflags;
use crossbeam_channel::{Receiver, TryRecvError};

use crate::interfaces::Memory;
use crate::utils;

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

pub struct Controller {
    enabled: bool,
    keyboard_channel: Receiver<char>,
    controller_snapshot: RefCell<InnerController>,
}

impl Controller {
    pub fn new(keyboard: Receiver<char>, enabled: bool) -> Self {
        Self {
            enabled,
            keyboard_channel: keyboard,
            controller_snapshot: RefCell::new(InnerController::empty()),
        }
    }

    pub fn connect(&mut self) {
        self.enabled = true;
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
            match c {
                's' | 'S' => input.insert(InnerController::LEFT),
                'd' | 'D' => input.insert(InnerController::DOWN),
                'f' | 'F' => input.insert(InnerController::RIGHT),
                'e' | 'E' => input.insert(InnerController::UP),
                'g' | 'G' => input.insert(InnerController::SELECT),
                'h' | 'H' => input.insert(InnerController::START),
                'j' | 'J' => input.insert(InnerController::A),
                'k' | 'K' => input.insert(InnerController::B),
                _ => {} // ignore
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
