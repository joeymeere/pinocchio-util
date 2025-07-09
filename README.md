# pinocchio-util

## Overview

pinocchio-util is crate designed to make your life just a little bit easier when building [`pinocchio`](https://crates.io/crates/pinocchio) programs.

While only being around for a relatively short time, Pinocchio has quickly become a go-to library for building efficient Solana programs. It strikes an effective balance between low-level control and developer ergonomics, which is one of the reasons I love it.

That being said, I've noticed a variety of recurring patterns between my own programs, and programs built by others. This crate encompasses some of the tools I've used over time to make implementing those patterns a bit easier.

## Installation

Install this crate with the following command:

```bash
cargo add pinocchio-util
```

or add this to your `Cargo.toml`:

```toml
pinocchio-util = "0.0.1"
```

## Usage

Load an immutable reference to an account's data as an arbitrary type

```rust
let account_data = load::<UserData>(&account)?;
```

Load a mutable reference to an account's data as an arbitrary type:

```rust
let mut account_data = load_mut::<UserData>(&account)?;
```

Extract an account's discriminator:

```rust
/// Defaults to first 8 bytes like classic Anchor does
let discriminator = load_discriminator(&account, None).unwrap();

/// You can also define an arbitrary length, which some programs
/// >= Anchor 0.30.0 do to save bytes
let discriminator = load_discriminator(&account, Some(4)).unwrap();
```

This crate also exposes various traits like `DataLen`, `AccountUpdates`, `Validate`, and `Context`. These are useful in their own right, but are best used with the `derive` feature outlined below.

## Derive Macros

Enable the `derive` feature to leverage various procedural macros:

```toml
pinocchio-util = { version = "0.0.1", features = ["derive"] }
```

Now that you've enabled them, you can add them to the various structs in your program:

```rust
#[derive(Context, Validate)]
pub struct BasicContext<'info> {
    // This is the signer
    #[validate(is_signer)]
    pub from: &'info AccountInfo,

    // This does match my random id
    #[validate(id = RANDOM_ID)]
    pub to: &'info AccountInfo,

    // This is a program, and this is the system program
    #[validate(is_executable, id = pinocchio_system::ID)]
    pub system_program: &'info AccountInfo,
}

// ...

let ctx = BasicContext::build(accounts)
    .map_err(|_| ProgramError::InvalidArgument)?;

```

