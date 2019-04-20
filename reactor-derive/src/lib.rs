#![recursion_limit="128"]

use quote::quote;

extern crate proc_macro;
extern crate proc_macro2;

use syn::parse::{
    Parse,
    ParseStream
};

#[derive(Debug)]
struct SubState {
    ident: syn::Ident,
    sub_states: Vec<SubState>,
}

impl Parse for SubState {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident;
        if input.peek(syn::Token![enum]) {
            input.parse::<syn::Token![enum]>()?;
            ident = input.parse()?;

            let content;
            syn::braced!(content in input);

            Ok(SubState {
                ident,
                sub_states: content.parse_terminated::<SubState, syn::Token![,]>(SubState::parse)?.into_iter().collect(),
            })
        } else if input.peek(syn::Ident) {
            ident = input.parse()?;
            
            Ok(SubState {
                ident,
                sub_states: vec![],
            })
        } else {
            panic!("Expected a state identifier or a nested state enum");
        }
    }
}

fn generate_enum_variants(state: &SubState) -> proc_macro2::TokenStream {
    let definitions = state.sub_states.iter().map(|sub_state| {
        let variant = sub_state.ident.clone();

        if sub_state.sub_states.is_empty() {
            quote! {
                #variant(#variant)
            }
        } else {
            quote! {
                #variant(#variant)
            }
        }
    })
    .collect::<Vec<_>>();

    quote! {
        #(#definitions),*
    }
}

fn assemble_from_sub_state(root: &SubState, sub_state: &SubState) -> proc_macro2::TokenStream {
    let sub_state_name = sub_state.ident.clone();

    let sub_state_definitions = sub_state.sub_states.iter().map(|sub_state| assemble_from_sub_state(root, sub_state)).collect::<Vec<_>>();
    let sub_state_variants = generate_enum_variants(sub_state);

    let super_trait_impl = impl_super(root, sub_state);

    if sub_state_variants.is_empty() {
        quote! {
            struct #sub_state_name {}

            #super_trait_impl

            #(#sub_state_definitions)*
        }
    } else {
        quote! {
            enum #sub_state_name {
                #(#sub_state_variants)*
            }

            #super_trait_impl

            #(#sub_state_definitions)*
        }
    }
}

#[proc_macro]
pub fn state_machine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    
    let input = proc_macro2::TokenStream::from(input);

    // println!("{:#?}", input);

    let root: SubState = syn::parse2(input.clone()).unwrap();

    println!("{:#?}", root);

    let enum_definitions = assemble_from_sub_state(&root, &root);

    let res = quote! {
        #enum_definitions
    };

    println!("{}", res.clone().to_string());

    res.into()
}

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

fn impl_super(root: &SubState, sub_state: &SubState) -> proc_macro2::TokenStream {
    let enter_match_branches = sub_state.sub_states.iter().map(|v| {
        let ident = sub_state.ident.clone();
        let variant_ident = v.ident.clone();

        quote! {
            #ident::#variant_ident(b) => {
                b.super_enter()
            }
        }
    }).collect::<Vec<_>>();

    let handle_match_branches = sub_state.sub_states.iter().map(|v| {
        let ident = sub_state.ident.clone();
        let variant_ident = v.ident.clone();
        quote!{
            #ident::#variant_ident(b) => {
                match b.super_handle(event.clone()) {
                    EventResult::Handled => EventResult::Handled,
                    EventResult::Transition(t) => EventResult::Transition(t),
                    EventResult::NotHandled => self.handle(event),
                }
            }
        }
    }).collect::<Vec<_>>();

    let exit_match_branches = sub_state.sub_states.iter().map(|v| {
        let ident = sub_state.ident.clone();
        let variant_ident = v.ident.clone();
        quote! {
            #ident::#variant_ident(b) => {
                b.super_exit()
            }
        }
    }).collect::<Vec<_>>();

    let name = sub_state.ident.clone();

    let (
        use_statement,
        match_enter_statement,
        match_handle_statement,
        match_exit_statement,
        initial_state_definition
    ) = if sub_state.sub_states.is_empty() {
        (
            quote! {},
            quote! {},
            quote! {
                self.handle(event)
            },
            quote! {},
            quote! {
                const INITIAL_STATE: Self = #name {};
            }
        )
    } else {
        let initial_variant = sub_state.sub_states.last().unwrap().ident.clone();
        (
            quote! {
                use #name::*;
            },
            quote! {
                match self {
                    #(#enter_match_branches,)*
                }
            },
            quote! {
                match self {
                    #(#handle_match_branches,)*
                }
            },
            quote! {
                match self {
                    #(#exit_match_branches,)*
                }
            },
            quote! {
                const INITIAL_STATE: Self = #name::#initial_variant(#initial_variant {});
            }
        )
    };

    let name = sub_state.ident.clone();
    let root_name = root.ident.clone();

    let res = quote! {
        impl State<Event> for #name where #name: reactor::base::Actor<Event> {
            type State = #root_name;
            #initial_state_definition

            fn super_enter(&self) {
                self.enter();
                
                #use_statement
                #match_enter_statement
            }

            fn super_handle(&self, event: Event) -> EventResult<<Self as State<Event>>::State> {
                #use_statement
                #match_handle_statement
            }

            fn super_exit(&self) {
                #use_statement
                #match_exit_statement

                self.exit()
            }
        }
    };

    // Uncomment to debug
    // println!("{}", res.to_string());
    
    res
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