#![cfg_attr(not(feature = "std"), no_std, no_main)]

// https://openbrush.brushfam.io/
// https://learn.brushfam.io/docs/openbrush

pub use self::psp34::Psp34Ref;

#[cfg_attr(feature = "cargo-clippy", allow(clippy::new_without_default))]
#[openbrush::implementation(PSP34, PSP34Metadata)] // ERC721 analogue
#[openbrush::contract]
pub mod psp34 {
    use openbrush::traits::Storage;

    type NftResult = core::result::Result<(), PSP34Error>;

    #[ink(storage)]
    #[derive(Default, Storage)]
    pub struct Psp34 {
        #[storage_field]
        psp34: psp34::Data,

        #[storage_field]
        metadata: metadata::Data,

        #[storage_field]
        next_id: u8, // first id is 1
    }

    impl Psp34 {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[ink(message)]
        pub fn mint_to(&mut self, category: String, account_id: AccountId) -> NftResult {
            self.next_id += 1;

            metadata::Internal::_set_attribute(
                self,
                Id::U8(self.next_id),
                String::from("category"),
                category,
            );

            psp34::Internal::_mint_to(self, account_id, Id::U8(self.next_id))?;

            Ok(())
        }
    }
}
