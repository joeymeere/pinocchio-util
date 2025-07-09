use pinocchio::{account_info::AccountInfo, program_error::ProgramError};

/// Get the length of an account's data.
pub trait DataLen {
    const LEN: usize;
}

/// Generate an enum and associated function for updating fields
/// on an account struct.
pub trait AccountUpdates {
    type Update;
    fn updates(&mut self, updates: Self::Update) -> Result<(), ProgramError>;
}

/// Validate surface level account attributes like keys, data length, and more.
pub trait Validate<'info> {
    fn validate(&self) -> Result<(), ProgramError>;
}

/// Build an instruction context with both accounts and instruction data
pub trait Context<'info>: Sized {
    const ACCOUNTS_LEN: usize;
    fn build(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError>;
}

/// Load an immutable reference to an account's data as an arbitrary type. This requires
/// that the provided type implements the `DataLen` trait so there's assurance that
/// no out of bounds access will occur.
///
/// # Example
///
/// ```rust
/// let account = AccountInfo::new(
///     &account,
///     false,
///     false,
///     false,
///     &mut accounts,
///     &mut ctx,
/// );
///
/// let account_data = load::<UserData>(&account)?;
/// ```
#[inline]
pub fn load<T: DataLen>(account: &AccountInfo) -> Result<&T, ProgramError> {
    if account.data_len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(unsafe {
        &*core::mem::transmute::<*const u8, *const T>(account.borrow_data_unchecked().as_ptr())
    })
}

/// Load a mutable reference to an account's data as an arbitrary type. This requires
/// that the provided type implements the `DataLen` trait so there's assurance that
/// no out of bounds access will occur.
///
/// # Example
///
/// ```rust
/// let mut account = AccountInfo::new(
///     &account,
///     false,
///     false,
///     false,
///     &mut accounts,
///     &mut ctx,
/// );
///
/// let mut account_data = load_mut::<UserData>(&account)?;
/// ```
#[inline]
pub fn load_mut<T: DataLen>(account: &AccountInfo) -> Result<&mut T, ProgramError> {
    if account.data_len() != T::LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(unsafe {
        &mut *core::mem::transmute::<*mut u8, *mut T>(
            account.borrow_mut_data_unchecked().as_mut_ptr(),
        )
    })
}

/// Extract an account's discriminator. This is useful if working with Anchor programs,
/// and you need to validate that a provided account is of a specific type.
///
/// You can optionally provide a custom length for the discriminator, and if not provided
/// the length will be defaulted to 8 bytes.
///
/// # Example
///
/// ```rust
/// let discriminator = load_discriminator(&account, None).unwrap();
/// assert_eq!(discriminator, &[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
///
/// let discriminator = load_discriminator(&account, Some(4)).unwrap();
/// assert_eq!(discriminator, &[0x00, 0x00, 0x00, 0x00]);
/// ```
///
#[inline]
pub fn load_discriminator(
    account: &AccountInfo,
    len: Option<usize>,
) -> Result<&[u8; 8], ProgramError> {
    let discriminator_len = len.unwrap_or(8);
    unsafe {
        account.borrow_data_unchecked()[0..discriminator_len]
            .try_into()
            .map_err(|_| ProgramError::InvalidAccountData)
    }
}
