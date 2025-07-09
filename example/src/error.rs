use num_enum::{IntoPrimitive, TryFromPrimitive};
use pinocchio::program_error::ProgramError;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
pub enum ProcessError {
    InvalidInstruction = 6001,
    NotEnoughAccounts = 6002,
}

impl From<ProcessError> for ProgramError {
    fn from(e: ProcessError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
