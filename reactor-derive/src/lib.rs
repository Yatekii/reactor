#![recursion_limit="128"]

use quote::quote;

extern crate proc_macro;
extern crate proc_macro2;

use syn::parse::{
    Parse,
    ParseStream
};

#[derive(Debug, PartialEq)]
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

fn assemble_from_sub_state(root: &SubState, sub_state: &SubState) -> (proc_macro2::TokenStream, usize) {
    let sub_state_name = sub_state.ident.clone();

    let t = sub_state.sub_states.iter().map(|sub_state| assemble_from_sub_state(root, sub_state)).collect::<(Vec<_>)>();
    let num_levels = t.iter().map(|v| v.1).fold(0, usize::max) + 1;
    let sub_state_definitions = t.into_iter().map(|v| v.0).collect::<Vec<_>>();

    let sub_state_variants = generate_enum_variants(sub_state);

    let super_trait_impl = impl_state(root, sub_state);

    (
        if sub_state_variants.is_empty() {
            quote! {
                #[derive(Copy, Clone, Debug)]
                struct #sub_state_name {}

                #super_trait_impl

                #(#sub_state_definitions)*
            }
        } else {
            quote! {
                #[derive(Copy, Clone, Debug)]
                enum #sub_state_name {
                    #(#sub_state_variants)*
                }

                #super_trait_impl

                #(#sub_state_definitions)*
            }
        },
        num_levels
    )
}

#[proc_macro]
pub fn state_machine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // Parse the string representation
    
    let input = proc_macro2::TokenStream::from(input);

    // println!("{:#?}", input);

    let root: SubState = syn::parse2(input.clone()).unwrap();

    println!("{:#?}", root);

    let (enum_definitions, num_levels) = assemble_from_sub_state(&root, &root);

    let ident = root.ident;

    let res = quote! {
        #enum_definitions

        #[derive(Debug)]
        pub struct Reactor {
            state: #ident,
        }

        const REACTOR_MAX_LEVELS: usize = #num_levels;

        impl React<Event> for Reactor {
            fn new() -> Self {
                let reactor = Self {
                    state: #ident::INITIAL_STATE,
                };
                reactor.state.super_enter(0);
                reactor
            }

            fn react(&mut self, event: Event) {
                match self.state.super_handle(event) {
                    EventResult::Transition(new_state) => {
                        // The initial value with the TypeId of the bool was chosen arbitrarily as there is no `::new()` or `::default()`.
                        // bool is no valid type in the enum tree, so no issue there.
                        let levels_new = &mut [core::any::TypeId::of::<bool>(); REACTOR_MAX_LEVELS];
                        let levels_old = &mut [core::any::TypeId::of::<bool>(); REACTOR_MAX_LEVELS];
                        new_state.get_levels(levels_new, 0);
                        self.state.get_levels(levels_old, 0);

                        let mut i = 0;
                        while i < REACTOR_MAX_LEVELS {
                            if levels_new[i] != levels_old[i] {
                                break;
                            }
                            i += 1;
                        }

                        println!("Moving {:?} -> {:?}", self, new_state);
                        
                        self.state.super_exit(i as i32);
                        self.state = new_state;
                        self.state.super_enter(i as i32);
                    }
                    _ => {},
                }
            }
        }
    };

    println!("{}", res.clone().to_string());

    res.into()
}

fn impl_state(root: &SubState, sub_state: &SubState) -> proc_macro2::TokenStream {
    let enter_match_branches = &sub_state.sub_states.iter().map(|v| {
        let ident = sub_state.ident.clone();
        let variant_ident = v.ident.clone();

        quote! {
            #ident::#variant_ident(b) => {
                b.super_enter(level - 1)
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

    let exit_match_branches = &sub_state.sub_states.iter().map(|v| {
        let ident = sub_state.ident.clone();
        let variant_ident = v.ident.clone();
        quote! {
            #ident::#variant_ident(b) => {
                b.super_exit(level - 1)
            }
        }
    }).collect::<Vec<_>>();

    let level_match_branches = sub_state.sub_states.iter().map(|v| {
        let ident = sub_state.ident.clone();
        let variant_ident = v.ident.clone();
        quote! {
            #ident::#variant_ident(b) => {
                levels[ptr] = core::any::TypeId::of::<#ident>();
                b.get_levels(levels, ptr + 1);
            }
        }
    }).collect::<Vec<_>>();

    let name = sub_state.ident.clone();

    let (
        match_level_statement,
        match_enter_statement,
        match_handle_statement,
        match_exit_statement,
        initial_state_definition
    ) = if sub_state.sub_states.is_empty() {
        let ident = sub_state.ident.clone();
        (
            quote! {
                levels[ptr] = core::any::TypeId::of::<#ident>();
            },
            quote! {
                self.enter()
            },
            quote! {
                self.handle(event)
            },
            quote! {
                self.exit()
            },
            quote! {
                const INITIAL_STATE: Self = #name {};
            }
        )
    } else {
        let initial_variant = sub_state.sub_states.last().unwrap().ident.clone();
        (
            quote! {
                match self {
                    #(#level_match_branches,)*
                }
            },
            quote! {
                if level > 0 {
                    match self {
                        #(#enter_match_branches,)*
                    }
                } else {
                    self.enter();
                    match self {
                        #(#enter_match_branches,)*
                    }
                }
            },
            quote! {
                match self {
                    #(#handle_match_branches,)*
                }
            },
            quote! {
                if level > 0 {
                    match self {
                        #(#exit_match_branches,)*
                    }
                } else {
                    match self {
                        #(#exit_match_branches,)*
                    }
                    self.exit();
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

            fn get_levels(&self, levels: &mut [core::any::TypeId], ptr: usize) {
                #match_level_statement
            }

            fn super_enter(&self, level: i32) {
                #match_enter_statement
            }

            fn super_handle(&self, event: Event) -> EventResult<<Self as State<Event>>::State> {
                #match_handle_statement
            }

            fn super_exit(&self, level: i32) {
                #match_exit_statement
            }
        }
    };

    // Uncomment to debug
    // println!("{}", res.to_string());
    
    res
}