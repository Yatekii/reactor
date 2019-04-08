#![recursion_limit="128"]

use quote::quote;

extern crate proc_macro;
extern crate proc_macro2;


#[proc_macro_derive(StateMachine, attributes(state_transitions))]
pub fn hsm(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    // Build the impl
    let expanded = impl_hsm(&ast);

    expanded.into()
}

fn impl_hsm(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    if let syn::Data::Enum(syn::DataEnum { ref variants,.. }) = ast.data {
        return impl_reactor(&ast.ident, variants.iter().collect::<Vec<_>>().as_slice(), &ast.generics);
    } else {
        panic!("State Machine must be derived on a enum.");
    }
}

fn impl_reactor(name: &syn::Ident, variants: &[&syn::Variant], _generics: &syn::Generics) -> proc_macro2::TokenStream {
    
    // let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

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
        impl<E: Clone> State<E> for #name where #name: reactor::base::Actor<E> {
            #initial_state_definition

            fn super_enter(&self) {
                self.enter();
                
                use #name::*;
                match self {
                    #(#enter_match_branches,)*
                }
            }

            fn super_handle<O>(&self, event: E) -> EventResult<O> {
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
    println!("{}", res.to_string());
    
    res
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic() {
        
    }
}