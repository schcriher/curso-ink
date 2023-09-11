#![cfg_attr(not(feature = "std"), no_std, no_main)]

pub use self::psp34::Psp34Ref;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#[openbrush::implementation(PSP34)] // ERC721 analogue
#[openbrush::contract]
pub mod psp34 {
    use openbrush::traits::Storage;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Psp34 {
        #[storage_field]
        psp34: psp34::Data,

        #[storage_field]
        next_id: u8,
    }

    impl Psp34 {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[ink(message)]
        pub fn mint_to(&mut self, to: AccountId) -> Result<(), PSP34Error> {
            psp34::Internal::_mint_to(self, to, Id::U8(self.next_id))?;
            self.next_id += 1;
            Ok(())
        }
    }
}
