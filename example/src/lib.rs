#![allow(unused, clippy::all, clippy::as_ptr_cast_mut)]

mod context;
mod error;
mod instructions;
mod processor;

#[cfg(not(feature = "bpf-entrypoint"))]
pub mod entrypoint {
    use pinocchio::{
        account_info::AccountInfo, entrypoint, msg, program_error::ProgramError, pubkey::Pubkey,
        ProgramResult,
    };
    use pinocchio_log::log;

    use crate::{error::ProcessError, instructions::ProgramInstruction, processor::Basic};

    entrypoint!(process_instruction);

    pub fn process_instruction(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Hello from Pinocchio!");
        let (discriminator, data) = instruction_data
            .split_first()
            .ok_or(ProgramError::InvalidArgument)?;

        match ProgramInstruction::try_from(*discriminator) {
            Ok(ProgramInstruction::Basic) => {
                log!("Instruction: Basic");
                let params = Basic::load(accounts, data)?;
                Basic::handle(params)?;
            }
            _ => {
                log!("Invalid instruction");
                return Err(ProcessError::InvalidInstruction.into());
            }
        };

        Ok(())
    }
}
