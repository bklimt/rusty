use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Serialize, attributes(id, pbtype))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    proc_macro::TokenStream::from(zombie_core::serialize::derive_serialize(input).unwrap())
}

#[proc_macro_derive(Deserialize, attributes(id, pbtype))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    proc_macro::TokenStream::from(zombie_core::deserialize::derive_deserialize(input).unwrap())
}
