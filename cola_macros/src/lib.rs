#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

extern crate cola_lib;
extern crate proc_macro2;
// pub mod cola_a;
// pub mod cola_messages;

// use proc_macro::TokenTree;
// // use proc_macro::Literal;
// use proc_macro2::{Literal, TokenStream};

use darling::usage::IdentSet;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseBuffer},
    punctuated::Punctuated,
    token::Comma,
    DeriveInput, Expr, Field, GenericParam, Generics, Ident,
};
use syn::{Data, Index};

const COLA_M: &str = "cola_m";

#[proc_macro_attribute]
pub fn cola_m(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let cln = input.clone();
    let mut data: DeriveInput = parse_macro_input!(cln as DeriveInput);
    let name = &data.ident;
    let mut inner: TokenStream = TokenStream::new();
    let mut outer: TokenStream = TokenStream::new();
    match data.data {
        Data::Enum(ref mut s) => {
            for v in s.variants.iter_mut() {
                let cmd = v
                    .attrs
                    .iter()
                    .filter(|a| a.path().is_ident(COLA_M))
                    .map(|f| f.parse_args::<proc_macro2::TokenStream>())
                    .last()
                    .unwrap()
                    .unwrap();
                let mut vars = quote! {};
                let id = &v.ident;
                let pred = quote! {let mut __internal = cola_lib::cola_a::CoLaUtil::vec_from_command(#cmd);};
                let eval: proc_macro2::TokenStream = match &v.fields {
                    syn::Fields::Named(f) => {
                        let intermediate: proc_macro2::TokenStream = f
                            .named
                            .iter()
                            .map(|f| &f.ident)
                            .map(|f| quote! {#f ,})
                            .collect();
                        vars = quote! {{#intermediate}};
                        field_expander(f.named.clone()).into()
                    }
                    syn::Fields::Unnamed(f) => {
                        //TODO:
                        field_expander(f.unnamed.clone()).into()
                    }
                    syn::Fields::Unit => quote! {},
                };
                inner.extend::<proc_macro::TokenStream>(
                    quote! {
                        #name::#id #vars => {
                            #pred
                            #eval
                            __internal
                        },
                    }
                    .into(),
                );
                outer.extend::<proc_macro::TokenStream>(quote! {}.into());
                v.attrs.retain(|a| !a.path().is_ident(COLA_M));
            }
        }
        _ => panic!("Can only be applied to enum types!"),
    };
    let inner: proc_macro2::TokenStream = inner.into();
    let _outer: proc_macro2::TokenStream = outer.into();
    let full = quote! {
        #data
        impl #name {
            pub fn to_raw_message(&self) -> Option<cola_lib::cola_a::ColaMessageRaw> {
                Some(match *self {
                    #inner
                })
            }
        }
    };
    full.into()
}

const COLA_INCOMING: &str = "cola_incoming";
#[proc_macro_attribute]
pub fn cola_incoming(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let cln = input.clone();
    let mut data: DeriveInput = parse_macro_input!(cln as DeriveInput);
    let name = &data.ident;
    let mut inner: TokenStream = TokenStream::new();
    // let mut outer: TokenStream = TokenStream::new();
    match data.data {
        Data::Enum(ref mut s) => {
            for v in s.variants.iter_mut() {
                let cmd = v
                    .attrs
                    .iter()
                    .filter(|a| a.path().is_ident(COLA_INCOMING))
                    .map(|f| f.parse_args::<proc_macro2::TokenStream>())
                    .last()
                    .unwrap()
                    .unwrap();
                let mut vars = quote! {};
                let id = &v.ident;
                let pred = quote! {/* let mut __internal = cola_lib::cola_a::CoLaUtil::vec_from_command(#cmd); */};
                let eval: proc_macro2::TokenStream = match &v.fields {
                    syn::Fields::Named(f) => {
                        let intermediate: proc_macro2::TokenStream = f
                            .named
                            .iter()
                            .map(|f| &f.ident)
                            .map(|f| quote! {#f ,})
                            .collect();
                        vars = quote! {{#intermediate}};
                        field_expander_incoming(f.named.clone()).into()
                    }
                    syn::Fields::Unnamed(f) => {
                        todo!("Not implemented for unnamed data types!")
                        //TODO:
                        //field_expander(f.unnamed.clone()).into()
                    }
                    syn::Fields::Unit => quote! {},
                };
                inner.extend::<proc_macro::TokenStream>(
                    quote! {
                        #cmd => {
                            let out = #name::#id{#eval};
                            // dbg!(&out);
                            return Ok(out);
                            // #pred
                            // #eval
                            // __internal
                        },
                    }
                    .into(),
                );
                v.attrs.retain(|a| !a.path().is_ident(COLA_INCOMING));
            }
        }
        _ => panic!("Can only be applied to enum types!"),
    };
    let inner: proc_macro2::TokenStream = inner.into();
    // let _outer: proc_macro2::TokenStream = outer.into();
    let full = quote! {
        #data
        impl #name {
            pub fn from_raw_message(msg: &mut cola_lib::cola_a::ColaMessageRaw) -> std::result::Result<#name, std::boxed::Box<(dyn std::error::Error + 'static)>> {
                let cmd_type: String = cola_lib::cola_a::CoLaDataType::get_from_data(msg)?;
                let cmd: String = cola_lib::cola_a::CoLaDataType::get_from_data(msg)?;
                // dbg!(&cmd_type);
                // dbg!(&cmd);
                // dbg!(&msg);
                match cmd.as_str() {
                    #inner
                    _ => {dbg!("No match!");
                        return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to get message!",
            )))}
                }
            }
        }
    };
    full.into()
}

#[proc_macro_derive(CoLaDataType)]
pub fn derive_data_type(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let cln = input.clone();
    let mut data: DeriveInput = parse_macro_input!(cln as DeriveInput);
    let name = &data.ident;
    let (impl_gen, ty_gen, wh_gen) = data.generics.split_for_impl();
    // let mut binding = gen.clone();
    // let wh = binding.;
    let mut inner_a: TokenStream = TokenStream::new();
    let mut inner_b: TokenStream = TokenStream::new();
    match data.data {
        Data::Struct(ref mut s) => s.fields.iter().for_each(|f| {
            let id = &f.ident;
            inner_a
                .extend::<proc_macro::TokenStream>(quote! {self.#id.write_to_data(data);}.into());
            inner_b.extend::<proc_macro::TokenStream>(
                quote! {#id: cola_lib::cola_a::CoLaDataType::get_from_data(input)?,}.into(),
            );
        }),
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
    let temp = name.to_string();

    let inner_a: proc_macro2::TokenStream = inner_a.into();
    let inner_b: proc_macro2::TokenStream = inner_b.into();
    quote! {
        impl #impl_gen cola_lib::cola_a::CoLaDataType for #name #ty_gen #wh_gen {

            fn write_to_data(&self, data: &mut Vec<u8>) {
                #inner_a
            }

            fn get_from_data(input: &mut Vec<u8>) -> std::result::Result<Self, std::boxed::Box<(dyn std::error::Error + 'static)>>  where Self:Sized {
                print!("Doing: ");
                 println!(#temp);
                Ok(Self{#inner_b})
            }
        }
    }
    .into()
}

fn field_expander(input: Punctuated<Field, Comma>) -> TokenStream {
    input
        .iter()
        .map(|d| &d.ident)
        .flat_map(|d| -> TokenStream {
            quote! {cola_lib::cola_a::CoLaDataType::write_to_data(&#d, &mut __internal);}.into()
        })
        .collect::<TokenStream>()
}

fn field_expander_incoming(input: Punctuated<Field, Comma>) -> TokenStream {
    input
        .iter()
        .map(|d| &d.ident)
        .flat_map(|d| -> TokenStream {
            quote! {#d: cola_lib::cola_a::CoLaDataType::get_from_data(msg)?,}.into()
        })
        .collect::<TokenStream>()
}
