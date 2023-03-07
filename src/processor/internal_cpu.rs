use crate::processor::status_register::StatusRegister;

#[derive(Clone)]
pub struct InternalCpu {
    pub acc: u8,   // Accumulator
    pub x_reg: u8, // X register
    pub y_reg: u8, // Y register
    pub sp: u8,    // Stack Pointer
    pub pc: u16,   // Program Counter
    pub sr: StatusRegister,
}

impl Default for InternalCpu {
    fn default() -> Self {
        Self {
            acc: 0,
            x_reg: 0,
            y_reg: 0,
            sp: 0xFF,
            pc: 0,
            sr: StatusRegister::default(),
        }
    }
}
