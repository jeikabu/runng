#![recursion_limit = "128"]

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, Lit, Meta, MetaNameValue};

#[proc_macro_derive(NngGetOpts, attributes(prefix, nng_member))]
pub fn derive_nng_socket_get_opts(tokens: TokenStream) -> TokenStream {
    derive_nng_opts(tokens, gen_get_impl)
}

#[proc_macro_derive(NngSetOpts, attributes(prefix, nng_member))]
pub fn derive_nng_socket_set_opts(tokens: TokenStream) -> TokenStream {
    derive_nng_opts(tokens, gen_set_impl)
}

/// Adds `impl NngMsg` containing all nng_msg_*() variants like `nng_msg_append_u32()`
#[proc_macro_derive(NngMsgOpts)]
pub fn derive_nng_msg(_tokens: TokenStream) -> TokenStream {
    _derive_nng_msg()
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
                                    return field.ident.clone();
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
    let get_bool = prefix.to_string() + "get_bool";
    let get_bool = syn::Ident::new(&get_bool, syn::export::Span::call_site());
    let get_int = prefix.to_string() + "get_int";
    let get_int = syn::Ident::new(&get_int, syn::export::Span::call_site());
    let get_ms = prefix.to_string() + "get_ms";
    let get_ms = syn::Ident::new(&get_ms, syn::export::Span::call_site());
    let get_size = prefix.to_string() + "get_size";
    let get_size = syn::Ident::new(&get_size, syn::export::Span::call_site());
    let get_uint64 = prefix.to_string() + "get_uint64";
    let get_uint64 = syn::Ident::new(&get_uint64, syn::export::Span::call_site());
    let get_string = prefix.to_string() + "get_string";
    let get_string = syn::Ident::new(&get_string, syn::export::Span::call_site());

    let gen = quote! {
        impl GetOpts for #name {
            /// Get `bool` option.
            /// See #get_bool
            fn get_bool(&self, option: NngOption) -> Result<bool> {
                unsafe {
                    let mut value: bool = Default::default();
                    Error::zero_map( #get_bool (self.#member, option.as_cptr(), &mut value), || value)
                }
            }
            /// Get `i32` option.
            /// See #get_int
            fn get_int(&self, option: NngOption) -> Result<i32> {
                unsafe {
                    let mut value: i32 = Default::default();
                    Error::zero_map( #get_int (self.#member, option.as_cptr(), &mut value), || value)
                }
            }
            /// Get `nng_duration` option.
            /// See #get_ms
            fn get_ms(&self, option: NngOption) -> Result<i32> {
                unsafe {
                    let mut value: i32 = Default::default();
                    Error::zero_map( #get_ms (self.#member, option.as_cptr(), &mut value), || value)
                }
            }
            /// Get `usize` option.
            /// See #get_size
            fn get_size(&self, option: NngOption) -> Result<usize>
            {
                unsafe {
                    let mut value: usize = Default::default();
                    Error::zero_map( #get_size (self.#member, option.as_cptr(), &mut value), || value)
                }
            }
            /// Get `u64` option.
            /// See #get_uint64
            fn get_uint64(&self, option: NngOption) -> Result<u64> {
                unsafe {
                    let mut value: u64 = Default::default();
                    Error::zero_map( #get_uint64 (self.#member, option.as_cptr(), &mut value), || value)
                }
            }
            /// Get `NngString` option.
            /// See #get_string
            fn get_string(&self, option: NngOption) -> Result<NngString> {
                unsafe {
                    let mut value: *mut ::std::os::raw::c_char = std::ptr::null_mut();
                    let res = #get_string (self.#member, option.as_cptr(), &mut value);
                    nng_int_to_result(res)?;
                    Ok(NngString::from_raw(value))
                }
            }
        }
    };
    gen.into()
}

fn gen_set_impl(name: &syn::Ident, prefix: &str, member: &syn::Ident) -> TokenStream {
    let set_bool = prefix.to_string() + "set_bool";
    let set_bool = syn::Ident::new(&set_bool, syn::export::Span::call_site());
    let set_int = prefix.to_string() + "set_int";
    let set_int = syn::Ident::new(&set_int, syn::export::Span::call_site());
    let set_ms = prefix.to_string() + "set_ms";
    let set_ms = syn::Ident::new(&set_ms, syn::export::Span::call_site());
    let set_size = prefix.to_string() + "set_size";
    let set_size = syn::Ident::new(&set_size, syn::export::Span::call_site());
    let set_uint64 = prefix.to_string() + "set_uint64";
    let set_uint64 = syn::Ident::new(&set_uint64, syn::export::Span::call_site());
    let set_string = prefix.to_string() + "set_string";
    let set_string = syn::Ident::new(&set_string, syn::export::Span::call_site());

    let gen = quote! {
        impl SetOpts for #name {
            /// Set `bool` [NngOption](./struct.NngOption.html).
            /// See #set_bool
            fn set_bool(&mut self, option: NngOption, value: bool) -> Result<()> {
                unsafe {
                    nng_int_to_result(#set_bool(self.#member, option.as_cptr(), value))
                }
            }
            /// See #set_int
            fn set_int(&mut self, option: NngOption, value: i32) -> Result<()> {
                unsafe {
                    nng_int_to_result(#set_int(self.#member, option.as_cptr(), value))
                }
            }
            /// See #set_ms
            fn set_ms(&mut self, option: NngOption, value: i32) -> Result<()> {
                unsafe {
                    nng_int_to_result(#set_ms(self.#member, option.as_cptr(), value))
                }
            }
            /// See #set_size
            fn set_size(&mut self, option: NngOption, value: usize) -> Result<()> {
                unsafe {
                    nng_int_to_result(#set_size(self.#member, option.as_cptr(), value))
                }
            }
            /// See #set_uint64
            fn set_uint64(&mut self, option: NngOption, value: u64) -> Result<()> {
                unsafe {
                    nng_int_to_result(#set_uint64(self.#member, option.as_cptr(), value))
                }
            }
            /// See #set_string
            fn set_string(&mut self, option: NngOption, value: &str) -> Result<()> {
                unsafe {
                    let (_, value) = to_cstr(value)?;
                    nng_int_to_result(#set_string(self.#member, option.as_cptr(), value))
                }
            }
        }
    };
    gen.into()
}

fn _derive_nng_msg() -> TokenStream {
    let methods = gen_method_symbols(&["append", "insert"]);
    let add_methods = methods.map(|(member, method, utype)| {
        quote! {
            pub fn #member(&mut self, data: #utype) -> Result<()> {
                unsafe { nng_int_to_result(#method(self.msg(), data)) }
            }
        }
    });

    let methods = gen_method_symbols(&["chop", "trim"]);
    let remove_methods = methods.map(|(member, method, utype)| {
        quote! {
            pub fn #member(&mut self) -> Result<#utype> {
                let mut val: #utype = 0;
                unsafe { Error::zero_map(#method(self.msg(), &mut val), || val) }
            }
        }
    });

    let gen = quote! {
        impl NngMsg {
            #(#add_methods)*
            #(#remove_methods)*
        }
    };
    gen.into()
}

// Takes list of methods and generates all the nng_msg_*() variants.
// For "append": nng_msg_append_u16 nng_msg_append_u32 .., nng_msg_header_append_u16 ..
fn gen_method_symbols(
    method_names: &'static [&'static str],
) -> impl Iterator<Item = (Ident, Ident, Ident)> {
    // To generate all the variants (nng_msg_append_u16, nng_msg_append_u32, ...) could use a
    // triple-nested `for` loop, or instead triple-nested `map`.  Need `flatten()` to turn:
    // [[[a b][c d]][[e f]]].flatten() -> [[a b][c d][e f]].flatten() -> [a b c...]
    ["", "header"]
        .iter()
        .map(move |prefix| {
            method_names
                .iter()
                .map(move |method| {
                    ["u16", "u32", "u64"].iter().map(move |utype| {
                        let (member, method) = if prefix.is_empty() {
                            let member = format!("{}_{}", method, utype);
                            let method = format!("nng_msg_{}_{}", method, utype);
                            (member, method)
                        } else {
                            let member = format!("{}_{}_{}", prefix, method, utype);
                            let method = format!("nng_msg_{}_{}_{}", prefix, method, utype);
                            (member, method)
                        };
                        let member = syn::Ident::new(&member, syn::export::Span::call_site());
                        let method = syn::Ident::new(&method, syn::export::Span::call_site());
                        let utype = syn::Ident::new(&utype, syn::export::Span::call_site());
                        (member, method, utype)
                    })
                })
                .flatten()
        })
        .flatten()
}
