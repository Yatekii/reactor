#![recursion_limit="128"]

use quote::quote;

extern crate proc_macro;
extern crate proc_macro2;


#[proc_macro_derive(StateMachine, attributes(event, state))]
pub fn hsm(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    // Build the impl
    let expanded = impl_hsm(&ast);

    expanded.into()
}

fn impl_hsm(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    if let syn::Data::Enum(syn::DataEnum { ref variants,.. }) = ast.data {
        return impl_reactor(&ast.ident, &ast.attrs, variants.iter().collect::<Vec<_>>().as_slice(), &ast.generics);
    } else {
        panic!("State Machine must be derived on a enum.");
    }
}

fn impl_reactor(name: &syn::Ident, attrs: &Vec<syn::Attribute>, variants: &[&syn::Variant], _generics: &syn::Generics) -> proc_macro2::TokenStream {
    
    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let event_attr = attrs.iter().filter(|attr| attr.path.segments.first().unwrap().value().ident.to_string() == "event").next().unwrap();
    let event_enum_type: Box<syn::Type> = syn::parse2(event_attr.tts.clone()).expect("Expected an enum type.");

    let state_attr = attrs.iter().filter(|attr| attr.path.segments.first().unwrap().value().ident.to_string() == "state").next().unwrap();
    let state_enum_type: Box<syn::Type> = syn::parse2(state_attr.tts.clone()).expect("Expected an enum type.");

    let enter_match_branches = variants.iter().map(|v| {
        let variant_ident = v.ident.clone();
        match &v.fields {
            syn::Fields::Unit => quote! {
                #variant_ident => {}
            },
            _ => quote! {
                #variant_ident(b) => if let Some(b) = b {
                    b.super_enter()
                }
            },
        }
    }).collect::<Vec<_>>();

    let handle_match_branches = variants.iter().map(|v| {
        let variant_ident = v.ident.clone();
        match &v.fields {
            syn::Fields::Unit => quote! {
                #variant_ident => self.handle(event)
            },
            _ => quote! {
                #variant_ident(b) => if let Some(b) = b {
                    match b.handle(event.clone()) {
                        EventResult::Handled => EventResult::Handled,
                        EventResult::Transition(t) => EventResult::Transition(t),
                        EventResult::NotHandled => self.handle(event),
                    }
                } else {
                    self.handle(event)
                }
            },
        }
    }).collect::<Vec<_>>();

    let exit_match_branches = variants.iter().map(|v| {
        let variant_ident = v.ident.clone();
        match &v.fields {
            syn::Fields::Unit => quote! {
                #variant_ident => {}
            },
            _ => quote! {
                #variant_ident(b) => if let Some(b) = b {
                    b.super_exit()
                }
            },
        }
    }).collect::<Vec<_>>();

    let initial_state_definition = variants.first().map(|v| {
        let variant_ident = v.ident.clone();
        match &v.fields {
            syn::Fields::Unit => quote! {
                const INITIAL_STATE: Self = #name::#variant_ident;
            },
            syn::Fields::Unnamed(_) => {
                quote! {
                    const INITIAL_STATE: Outer = #name::#variant_ident(None);
                }
            }
            _ => quote! {}
        }
    }).unwrap();

    let res = quote! {
        impl State<#event_enum_type> for #name where #name: reactor::base::Actor<#event_enum_type> {
            type State = #state_enum_type;
            #initial_state_definition

            fn super_enter(&self) {
                self.enter();
                
                use #name::*;
                match self {
                    #(#enter_match_branches,)*
                }
            }

            fn super_handle(&self, event: #event_enum_type) -> EventResult<<Self as State<#event_enum_type>>::State> {
                use #name::*;
                match self {
                    #(#handle_match_branches,)*
                }
            }

            fn super_exit(&self) {
                use #name::*;
                match self {
                    #(#exit_match_branches,)*
                }

                self.exit()
            }
        }
    };

    // Uncomment to debug
    // println!("{}", res.to_string());
    
    res
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        
    }
}