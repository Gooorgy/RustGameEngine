use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FieldsNamed, ItemStruct};

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let original_fields = match &input.fields {
        syn::Fields::Named(FieldsNamed { named, .. }) => named,
        _ => panic!("Expected a struct with named fields"),
    };

    let expanded_struct = quote! {
        pub struct #struct_name {
            transform: Transform,
            #original_fields
         }
    };

    let expanded_impl = quote! {
        impl Component for #struct_name {
            fn get_transform(&self) -> &Transform {
                &self.transform
            }
        }
    };

    TokenStream::from(quote! {
        #expanded_struct
        #expanded_impl
    })
}


