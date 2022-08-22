pub trait Processor {
    /// Reset the processor
    fn reset(&mut self);

    /// Fetch the instruction pointed by the program counter from
    /// memory and execute it atomically
    fn execute(&mut self);
}

pub trait Bus {
    fn read(&self, address: u16) -> u8;
    fn write(&self, address: u16, data: u8);
}

pub trait Memory {
    /// Read a byte from the specified `address`
    fn read(&self, address: u16) -> u8;

    /// Write a byte of `data` to the specified `address`
    fn write(&mut self, address: u16, data: u8);
}
