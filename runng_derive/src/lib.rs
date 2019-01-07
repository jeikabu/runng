#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Lit, Meta, MetaNameValue};

#[proc_macro_derive(NngGetOpts, attributes(prefix, nng_member))]
pub fn derive_nng_get_opts(tokens: TokenStream) -> TokenStream {
    derive_nng_opts(tokens, gen_get_impl)
}

#[proc_macro_derive(NngSetOpts, attributes(prefix, nng_member))]
pub fn derive_nng_set_opts(tokens: TokenStream) -> TokenStream {
    derive_nng_opts(tokens, gen_set_impl)
}

fn get_nng_member(ast: &syn::DeriveInput) -> Option<syn::Ident> {
    match ast.data {
        syn::Data::Struct(ref data_struct) => {
            match data_struct.fields {
                // Structure with named fields (as opposed to tuple-like struct or unit struct)
                // E.g. struct Point { x: f64, y: f64 }
                syn::Fields::Named(ref fields_named) => {
                    // Iterate over the fields: `x`, `y`, ..
                    for field in fields_named.named.iter() {
                        // Attributes `#[..]` on each field
                        for attr in field.attrs.iter() {
                            // Parse the attribute
                            let meta = attr.parse_meta().unwrap();
                            match meta {
                                // Matches attribute with single word like `#[nng_member]` (as opposed to `#[derive(NngGetOps)]` or `#[nng_member = "socket"]`)
                                Meta::Word(ref ident) if ident == "nng_member" =>
                                // Return name of field with `#[nng_member]` attribute
                                {
                                    return field.ident.clone()
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            }
        }
        _ => panic!("Must be a struct"),
    }

    None
}

fn derive_nng_opts<F>(tokens: TokenStream, gen_impl: F) -> TokenStream
where
    F: Fn(&syn::Ident, &str, &syn::Ident) -> TokenStream,
{
    let ast: syn::DeriveInput = syn::parse(tokens).unwrap();
    // .filter(|attr| attr.path.is_ident("options"))
    let member: Option<syn::Ident> = get_nng_member(&ast);

    let mut prefix: Option<String> = None;
    for option in ast.attrs.into_iter() {
        let option = option.parse_meta().unwrap();
        match option {
            // Match `#[ident = lit]` attributes.  Match guard makes it `#[prefix = lit]`
            Meta::NameValue(MetaNameValue {
                ref ident, ref lit, ..
            }) if ident == "prefix" => {
                if let Lit::Str(lit) = lit {
                    prefix = Some(lit.value());
                } else {
                    panic!("Oops");
                }
            }
            _ => {} // Documentation comments `///` have meta.name()
        }
    }
    //TokenStream::from_iter(impls.into_iter())
    gen_impl(&ast.ident, &prefix.unwrap(), &member.unwrap())
}

fn gen_get_impl(name: &syn::Ident, prefix: &str, member: &syn::Ident) -> TokenStream {
    let getopt_bool = prefix.to_string() + "getopt_bool";
    let getopt_bool = syn::Ident::new(&getopt_bool, syn::export::Span::call_site());
    let getopt_int = prefix.to_string() + "getopt_int";
    let getopt_int = syn::Ident::new(&getopt_int, syn::export::Span::call_site());
    let getopt_size = prefix.to_string() + "getopt_size";
    let getopt_size = syn::Ident::new(&getopt_size, syn::export::Span::call_site());
    let getopt_uint64 = prefix.to_string() + "getopt_uint64";
    let getopt_uint64 = syn::Ident::new(&getopt_uint64, syn::export::Span::call_site());
    let getopt_string = prefix.to_string() + "getopt_string";
    let getopt_string = syn::Ident::new(&getopt_string, syn::export::Span::call_site());

    let gen = quote! {
        impl GetOpts for #name {
            fn getopt_bool(&self, option: NngOption) -> NngResult<bool> {
                unsafe {
                    let mut value: bool = Default::default();
                    NngFail::succeed( #getopt_bool (self.#member, option.as_cptr(), &mut value), value)
                }
            }
            fn getopt_int(&self, option: NngOption) -> NngResult<i32> {
                unsafe {
                    let mut value: i32 = Default::default();
                    NngFail::succeed( #getopt_int (self.#member, option.as_cptr(), &mut value), value)
                }
            }
            fn getopt_size(&self, option: NngOption) -> NngResult<usize>
            {
                unsafe {
                    let mut value: usize = Default::default();
                    NngFail::succeed( #getopt_size (self.#member, option.as_cptr(), &mut value), value)
                }
            }
            fn getopt_uint64(&self, option: NngOption) -> NngResult<u64> {
                unsafe {
                    let mut value: u64 = Default::default();
                    NngFail::succeed( #getopt_uint64 (self.#member, option.as_cptr(), &mut value), value)
                }
            }
            fn getopt_string(&self, option: NngOption) -> NngResult<NngString> {
                unsafe {
                    let mut value: *mut ::std::os::raw::c_char = std::ptr::null_mut();
                    let res = #getopt_string (self.#member, option.as_cptr(), &mut value);
                    NngFail::from_i32(res)?;
                    Ok(NngString::new(value))
                }
            }
        }
    };
    gen.into()
}

fn gen_set_impl(name: &syn::Ident, prefix: &str, member: &syn::Ident) -> TokenStream {
    let setopt_bool = prefix.to_string() + "setopt_bool";
    let setopt_bool = syn::Ident::new(&setopt_bool, syn::export::Span::call_site());
    let setopt_int = prefix.to_string() + "setopt_int";
    let setopt_int = syn::Ident::new(&setopt_int, syn::export::Span::call_site());
    let setopt_size = prefix.to_string() + "setopt_size";
    let setopt_size = syn::Ident::new(&setopt_size, syn::export::Span::call_site());
    let setopt_uint64 = prefix.to_string() + "setopt_uint64";
    let setopt_uint64 = syn::Ident::new(&setopt_uint64, syn::export::Span::call_site());
    let setopt_string = prefix.to_string() + "setopt_string";
    let setopt_string = syn::Ident::new(&setopt_string, syn::export::Span::call_site());

    let gen = quote! {
        impl SetOpts for #name {
            fn setopt_bool(&mut self, option: NngOption, value: bool) -> NngReturn {
                unsafe {
                    NngFail::from_i32(#setopt_bool(self.#member, option.as_cptr(), value))
                }
            }
            fn setopt_int(&mut self, option: NngOption, value: i32) -> NngReturn {
                unsafe {
                    NngFail::from_i32(#setopt_int(self.#member, option.as_cptr(), value))
                }
            }
            fn setopt_size(&mut self, option: NngOption, value: usize) -> NngReturn {
                unsafe {
                    NngFail::from_i32(#setopt_size(self.#member, option.as_cptr(), value))
                }
            }
            fn setopt_uint64(&mut self, option: NngOption, value: u64) -> NngReturn {
                unsafe {
                    NngFail::from_i32(#setopt_uint64(self.#member, option.as_cptr(), value))
                }
            }
            fn setopt_string(&mut self, option: NngOption, value: &str) -> NngReturn {
                unsafe {
                    let (_, value) = to_cstr(value)?;
                    NngFail::from_i32(#setopt_string(self.#member, option.as_cptr(), value))
                }
            }
        }
    };
    gen.into()
}
