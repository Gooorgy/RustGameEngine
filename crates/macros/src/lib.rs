use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FieldsNamed, ItemStruct};

#[proc_macro_attribute]
pub fn primitive_game_object(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let original_fields = match &input.fields {
        syn::Fields::Named(FieldsNamed { named, .. }) => named,
        _ => panic!("Expected a struct with named fields"),
    };

    let field_idents: Vec<_> = original_fields
        .iter()
        .filter_map(|f| f.ident.as_ref()) // skip unnamed fields (tuple structs)
        .collect();

    let expanded_struct = quote! {
        pub struct #struct_name {
            transform: rendering_backend::transform::Transform,
            #original_fields
         }
    };

    let expanded_impl = quote! {
        impl game_object::traits::GameObjectDefaults for #struct_name {
            fn get_transform(&self) -> rendering_backend::transform::Transform {
                self.transform
            }

            fn with_transform(mut self, transform: rendering_backend::transform::Transform) -> Self {
                self.transform = transform;
                self
            }

            fn with_location(mut self, location: nalgebra_glm::Vec3) -> Self {
                self.transform.location = location;
                self
            }

            fn with_rotation(mut self, rotation: nalgebra_glm::Vec3) -> Self {
                self.transform.rotation = rotation;
                self
            }

            fn with_scale(mut self, scale: nalgebra_glm::Vec3) -> Self {
                self.transform.scale = scale;
                self
            }
        }
    };

    let clone_impl = quote! {
        impl Clone for #struct_name {
            fn clone(&self) -> Self {
                Self {
                    #(
                        #field_idents: self.#field_idents.clone(),
                    )*
                    transform: self.transform.clone(),
                }
            }
        }
    };

    TokenStream::from(quote! {
        #expanded_struct
        #expanded_impl
        #clone_impl
    })
}
