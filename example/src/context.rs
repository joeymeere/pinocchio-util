#![allow(unused)]

use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use pinocchio_derive::{Context, DataLen, Updates, Validate};
use pinocchio_log::log;
use pinocchio_pubkey::pubkey;

use crate::error::ProcessError;

const RANDOM_ID: Pubkey = pubkey!("ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt");

#[derive(DataLen, Updates)]
pub struct IWantToBeARealBoy {
    pub discriminator: [u8; 8],
    pub data: [u8; 32],
    pub bump: u8,
}

impl IWantToBeARealBoy {
    pub fn maybe_update(&mut self, update: IWantToBeARealBoyUpdate) -> Result<(), ProgramError> {
        match update {
            IWantToBeARealBoyUpdate::SetDiscriminator(discriminator) => {
                self.discriminator = discriminator;
            }
            IWantToBeARealBoyUpdate::SetData(data) => {
                self.data = data;
            }
            IWantToBeARealBoyUpdate::SetBump(bump) => {
                self.bump = bump;
            }
            _ => {
                return Err(ProgramError::InvalidArgument);
            }
        }

        Ok(())
    }
}

#[derive(Context, Validate)]
pub struct BasicContext<'info> {
    #[validate(is_signer)]
    pub from: &'info AccountInfo,

    #[validate(id = RANDOM_ID)]
    pub to: &'info AccountInfo,

    #[validate(is_executable, id = pinocchio_system::ID)]
    pub system_program: &'info AccountInfo,
}
