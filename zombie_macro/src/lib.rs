use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Serialize, attributes(id))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    quote! {
        #[automatically_derived]
        impl zombie::Serialize for #name {
            fn serialize(&self) { println!("serialized"); }
        }
    }
    .into()
}
