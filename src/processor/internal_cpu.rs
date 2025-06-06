use crate::processor::status_register::StatusRegister;

#[derive(Clone)]
pub struct InternalCpu {
    pub acc: u8,   // Accumulator
    pub x_reg: u8, // X register
    pub y_reg: u8, // Y register
    pub sp: u8,    // Stack Pointer
    pub pc: u16,   // Program Counter
    pub sr: StatusRegister,

    // indicates whether an instruction affecting the PC crossed a page. This
    // have an effect on the execution time, usually adding a clock
    pub page_boundary_crossed: bool,

    // when a branch operation is executed, a boolean is set. This can add 1 or
    // 2 extra clocks to the instruction execution depending whether the page
    // boundary was crossed or not.
    pub branch_taken: bool,
}

impl Default for InternalCpu {
    fn default() -> Self {
        Self {
            acc: 0,
            x_reg: 0,
            y_reg: 0,
            sp: 0,
            pc: 0,
            sr: StatusRegister::default(),
            page_boundary_crossed: false,
            branch_taken: false,
        }
    }
}
