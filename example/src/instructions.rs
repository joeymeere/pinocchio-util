use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Debug, Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum ProgramInstruction {
    Basic = 0,
}

#[derive(Clone)]
pub struct BasicInstruction {
    pub amount: u64,
}
