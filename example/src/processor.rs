use pinocchio::account_info::AccountInfo;
use pinocchio::program_error::ProgramError;
use pinocchio::ProgramResult;
use pinocchio_log::log;
use pinocchio_system::instructions::Transfer;
use pinocchio_util::Context;

use crate::context::*;
use crate::instructions::*;

pub struct Basic<'info> {
    pub accounts: BasicContext<'info>,
    pub data: BasicInstruction,
}

impl<'info> Basic<'info> {
    pub fn load(accounts: &'info [AccountInfo], data: &[u8]) -> Result<Self, ProgramError> {
        let ctx = BasicContext::build(accounts).map_err(|_| ProgramError::InvalidArgument)?;
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        Ok(Self {
            accounts: ctx,
            data: BasicInstruction { amount },
        })
    }

    pub fn handle(params: Self) -> ProgramResult {
        log!("I want to be a real boy!");

        Transfer {
            from: params.accounts.from,
            to: params.accounts.to,
            lamports: params.data.amount,
        }
        .invoke()?;

        Ok(())
    }
}
