use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input, punctuated::Punctuated, Data, DeriveInput,
    Field, Ident, Meta, Token,
};

/// Generates a trait implementation for `DataLen`:
///
/// ```rust
/// impl DataLen for MyStruct {
///     pub const LEN: usize = core::mem::size_of::<MyStruct>();
/// }
/// ```
#[proc_macro_derive(DataLen)]
pub fn derive_data_len(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        impl pinocchio_util::DataLen for #name {
            const LEN: usize = core::mem::size_of::<#name>();
        }
    };

    TokenStream::from(expanded)
}

/// Generates an update enum and trait implementation for `AccountUpdates`:
///
/// ```rust
/// pub enum MyStructUpdate {
///     SetField1(u32),
///     SetField2(u32),
/// }
///
/// impl AccountUpdates for MyStruct {
///     type Update = MyStructUpdate;
///     fn updates(&mut self, updates: Self::Update) {
///         match updates {
///             MyStructUpdate::SetField1(value) => self.field1 = value,
///             MyStructUpdate::SetField2(value) => self.field2 = value,
///         }
///     }
/// }
/// ```
#[proc_macro_derive(Updates)]
pub fn derive_updates(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let update_enum_name = Ident::new(&format!("{}Update", name), name.span());

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => panic!("Updates derive macro only supports structs"),
    };

    let field_variants: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(_i, field)| {
            let field_name = field.ident.as_ref().unwrap();
            let _field_type = &field.ty;
            let variant_name = Ident::new(
                &format!(
                    "Set{}",
                    field_name
                        .to_string()
                        .chars()
                        .next()
                        .unwrap()
                        .to_uppercase()
                        .chain(field_name.to_string().chars().skip(1))
                        .collect::<String>()
                ),
                field_name.span(),
            );

            quote! {
                #variant_name(#_field_type)
            }
        })
        .collect();

    let match_arms: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(_i, field)| {
            let field_name = field.ident.as_ref().unwrap();
            let _field_type = &field.ty;
            let variant_name = Ident::new(
                &format!(
                    "Set{}",
                    field_name
                        .to_string()
                        .chars()
                        .next()
                        .unwrap()
                        .to_uppercase()
                        .chain(field_name.to_string().chars().skip(1))
                        .collect::<String>()
                ),
                field_name.span(),
            );

            quote! {
                #update_enum_name::#variant_name(value) => self.#field_name = value,
            }
        })
        .collect();

    let expanded = quote! {
        pub enum #update_enum_name {
            #(#field_variants),*
        }

        impl pinocchio_util::AccountUpdates for #name {
            type Update = #update_enum_name;

            fn updates(&mut self, updates: Self::Update) -> Result<(), pinocchio::program_error::ProgramError> {
                match updates {
                    #(#match_arms)*
                }
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

struct ValidationAttr {
    non_empty: bool,
    is_signer: bool,
    is_executable: bool,
    len: Option<usize>,
    id: Option<syn::Expr>,
}

impl Parse for ValidationAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut non_empty = false;
        let mut len = None;
        let mut id = None;
        let mut is_signer = false;
        let mut is_executable = false;

        let args = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;

        for arg in args {
            match arg {
                Meta::Path(path) => {
                    if path.is_ident("non_empty") {
                        non_empty = true;
                    }
                }
                Meta::NameValue(name_value) => {
                    if name_value.path.is_ident("len") {
                        if let syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(lit_int),
                            ..
                        }) = &name_value.value
                        {
                            len = Some(lit_int.base10_parse()?);
                        }
                    } else if name_value.path.is_ident("id") {
                        id = Some(name_value.value);
                    }
                }
                _ => {}
            }
        }

        Ok(ValidationAttr {
            non_empty,
            len,
            id,
            is_signer,
            is_executable,
        })
    }
}

/// Generates an implementation for `Validate`:
///
/// ```rust
/// pub trait Validate {
///     fn validate(&self) -> Result<(), ProgramError>;
/// }
///
/// impl Validate for MyStruct {
///     fn validate(&self) -> Result<(), ProgramError> {
///         // Validations here
///         Ok(())
///     }
/// }
/// ```
///
/// Example usage:
///
/// ```rust
/// #[derive(Validate)]
/// struct MyStruct {
///     // Data length is non-zero, `field_1.key()` is the SYSTEM_PROGRAM_ID (Pubkey)
///     #[validate(non_empty, id = SYSTEM_PROGRAM_ID)]
///     field_1: &'a AccountInfo,
///
///     // Data length is 64, `field_2.key()` is the SOME_ID (Pubkey)
///     #[validate(len = 64, id = SOME_ID)]
///     field_2: &'a AccountInfo,
/// }
/// ```
#[proc_macro_derive(Validate, attributes(validate))]
pub fn derive_validate(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields = match input.data {
        Data::Struct(data) => data.fields,
        _ => panic!("This macro only supports structs"),
    };

    let validation_checks: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(_i, field)| {
            let field_name = field.ident.as_ref().unwrap();

            let mut validation_attr = None;
            for attr in &field.attrs {
                if attr.path().is_ident("validate") {
                    validation_attr = Some(attr.parse_args::<ValidationAttr>().unwrap());
                    break;
                }
            }

            if let Some(attr) = validation_attr {
                let mut checks = Vec::new();

                if attr.non_empty {
                    checks.push(quote! {
                        if self.#field_name.data_len() == 0 {
                            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                        }
                    });
                }

                if attr.is_signer {
                    checks.push(quote! {
                        if !self.#field_name.is_signer() {
                            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                        }
                    });
                }

                if attr.is_executable {
                    checks.push(quote! {
                        if !self.#field_name.is_executable() {
                            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                        }
                    });
                }

                if let Some(len) = attr.len {
                    checks.push(quote! {
                        if self.#field_name.data_len() != #len {
                            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                        }
                    });
                }

                if let Some(id) = attr.id {
                    checks.push(quote! {
                        if self.#field_name.key() != &#id {
                            return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                        }
                    });
                }

                quote! {
                    #(#checks)*
                }
            } else {
                quote! {}
            }
        })
        .collect();

    let expanded = quote! {
        impl<'info> pinocchio_util::Validate<'info> for #name<'info> {
            fn validate(&self) -> Result<(), pinocchio::program_error::ProgramError> {
                #(#validation_checks)*
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Generates an implementation for `Context`:
///
/// ```rust
/// pub trait Context<'info> {
///     const ACCOUNTS_LEN: usize;
///     fn build(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError>;
/// }
///
/// impl<'info> Context<'info> for MyStruct<'info> {
///     // # of fields in the struct
///     const ACCOUNTS_LEN: usize = 1;
///
///     fn build(accounts: &'info [AccountInfo]) -> Result<Self, ProgramError> {
///         let ctx = unsafe {
///             Self {
///                 field_1: &accounts.get_unchecked(0),
///                 field_2: &accounts.get_unchecked(1),
///             }
///         }
///
///         Ok(ctx)
///     }
/// }
/// ```
#[proc_macro_derive(Context)]
pub fn derive_context(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let lifetime_params: Vec<_> = input.generics.lifetimes().collect();

    if lifetime_params.len() != 1 {
        panic!("Context derive requires exactly one lifetime parameter");
    }

    let lifetime_param = &lifetime_params[0];
    let lifetime = &lifetime_param.lifetime;

    if lifetime.ident != "info" {
        panic!("Context derive requires the lifetime parameter to be named 'info");
    }

    let fields = match input.data {
        Data::Struct(ref data) => &data.fields,
        _ => panic!("Context derive only works on structs"),
    };

    let accounts_len = fields.len();
    let field_assignments: Vec<_> = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let field_name = field.ident.as_ref().unwrap();
            quote! { #field_name: &accounts.get_unchecked(#i), }
        })
        .collect();

    let expanded = quote! {
        impl<'info> pinocchio_util::Context<'info> for #name<'info> {
            const ACCOUNTS_LEN: usize = #accounts_len;

            fn build(accounts: &'info [pinocchio::account_info::AccountInfo])
                -> Result<Self, pinocchio::program_error::ProgramError>
            {
                if accounts.len() != Self::ACCOUNTS_LEN {
                    return Err(pinocchio::program_error::ProgramError::InvalidAccountData);
                }

                Ok(unsafe {
                    Self {
                        #(#field_assignments)*
                    }
                })
            }
        }
    };

    TokenStream::from(expanded)
}
