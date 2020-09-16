use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let register_enum = RegisterEnum::parse(args, input)?;

    let expanded_module = register_enum.define_module();

    Ok(proc_macro::TokenStream::from(quote! {
            #expanded_module
    }))
}

struct RegisterEnum {
    ident: syn::Ident,
    vis: syn::Visibility,
    args: Args,
    entries: Vec<RegisterEntry>,
    attrs: Vec<syn::Attribute>,
}

impl RegisterEnum {
    fn parse(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> Result<Self> {
        let input_enum: syn::ItemEnum = syn::parse(input)?;
        let span = input_enum.span();

        let ident = input_enum.ident;
        let vis = input_enum.vis;
        if let syn::Visibility::Restricted(restricted) = &vis {
            if restricted.in_token.is_some() {
                return Err(Error::new_spanned(vis, "pub(in ...) can't be used"));
            }
        };

        let args: Args = syn::parse(args)?;

        let mut offset = 0;
        let mut entries = vec![];
        for variant in input_enum.variants.into_iter() {
            let ent = RegisterEntry::parse(variant, &mut offset)?;
            entries.push(ent);
        }

        if entries.is_empty() {
            return Err(Error::new(span, "at least one variant is required"));
        }

        Ok(Self {
            ident,
            vis,
            args,
            entries,
            attrs: input_enum.attrs,
        })
    }

    fn define_module(&self) -> TokenStream {
        let mod_name = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        let vis_inside_mod = self.modify_visibility();

        let structs = self.entries.iter().map(|entry| {
            let ident = &entry.ident;
            let attrs = entry.attrs.iter();
            quote! {
                #(#attrs)*
                #vis_inside_mod struct #ident {}
            }
        });

        let raw = self.impl_raw();
        let memory_protection = self.impl_memory_protection();
        let base = self.const_base();
        let size = self.const_size();
        let impl_register_entry = self.impl_register_entry();

        quote! {
            #(#attrs)*
            #[allow(non_snake_case)]
            #[allow(clippy::string_lit_as_bytes)]
            #vis mod #mod_name {
                use super::*;
                #base
                #size
                #raw
                #memory_protection
                #impl_register_entry
                #(#structs)*
            }
        }
    }

    fn impl_register_entry(&self) -> TokenStream {
        let impls = self
            .entries
            .iter()
            .map(|entry| entry.impl_register_entry(&self.args.base, self.args.endianess));

        quote! {
            #(#impls)*
        }
    }

    fn impl_memory_protection(&self) -> TokenStream {
        let set_access_right = self.entries.iter().map(|entry| {
            let start = entry.offset;
            let end = start + entry.entry_attr.len();
            let access_right = &entry.entry_attr.access;
            quote! {
                memory_protection.set_access_right_with_range(#start..#end, cameleon_impl::memory::AccessRight::#access_right);
            }});

        let size = self.size();
        let vis = self.modify_visibility();
        quote! {
            #vis fn memory_protection() -> cameleon_impl::memory::MemoryProtection {
                let mut memory_protection = cameleon_impl::memory::MemoryProtection::new(#size);
                #(#set_access_right)*
                memory_protection
            }
        }
    }

    fn impl_raw(&self) -> TokenStream {
        let fragment = format_ident!("fragment");
        let mut writes = vec![];
        for entry in &self.entries {
            writes.push(entry.init_entry(fragment.clone(), self.args.endianess));
        }

        let endianess = self.args.endianess;
        let size = self.size();
        let vis = self.modify_visibility();
        quote! {
            #vis fn raw() -> Vec<u8> {
                use cameleon_impl::byteorder::{#endianess, WriteBytesExt};
                use std::io::Write;
                let mut fragment_vec = vec![0; #size];
                let mut #fragment = std::io::Cursor::new(fragment_vec.as_mut_slice());
                #(#writes)*
                fragment_vec
            }
        }
    }

    fn const_base(&self) -> TokenStream {
        let base = &self.args.base;
        let vis = self.modify_visibility();
        quote! {
            #vis const BASE: usize = #base as usize;
        }
    }

    fn const_size(&self) -> TokenStream {
        let size = self.size();
        let vis = self.modify_visibility();
        quote! {
            #vis const SIZE: usize = #size;
        }
    }

    fn size(&self) -> usize {
        let last_field = self.entries.last().unwrap();
        last_field.offset + last_field.entry_attr.len()
    }

    fn modify_visibility(&self) -> syn::Visibility {
        use syn::Visibility::*;
        match &self.vis {
            Public(_) | Crate(_) => self.vis.clone(),
            Inherited => syn::parse_str("pub(super)").unwrap(),
            Restricted(restricted) => {
                let original = restricted.path.get_ident().unwrap();
                if original == "crate" {
                    syn::parse_str("pub(crate)").unwrap()
                } else if original == "super" {
                    syn::parse_str("pub(in super::super)").unwrap()
                } else if original == "self" {
                    syn::parse_str("pub(super)").unwrap()
                } else {
                    unreachable!();
                }
            }
        }
    }
}

struct RegisterEntry {
    ident: syn::Ident,
    offset: usize,
    entry_attr: EntryAttr,
    init: Option<InitValue>,
    attrs: Vec<syn::Attribute>,
}

impl RegisterEntry {
    fn parse(mut variant: syn::Variant, offset: &mut usize) -> Result<Self> {
        let entry_attr = Self::parse_entry_attr(&mut variant)?;
        let ident = variant.ident;
        let entry_offset = *offset;

        *offset += entry_attr.len();

        let init = if let Some((_, expr)) = variant.discriminant {
            Some(InitValue::from_expr(expr)?)
        } else {
            None
        };

        Ok(Self {
            ident,
            offset: entry_offset,
            entry_attr,
            init,
            attrs: variant.attrs,
        })
    }

    fn init_entry(&self, fragment: syn::Ident, endianess: Endianess) -> TokenStream {
        if self.init.is_none() {
            return quote! {};
        }

        let init = self.init.as_ref().unwrap();
        let start = self.offset as u64;
        let endianess = endianess;

        let set_position_expand = quote! {#fragment.set_position(#start);};

        let write_expand = match self.entry_attr.ty {
            EntryType::Str => {
                let len = self.entry_attr.len();
                quote! {
                    if #len < #init.as_bytes().len() {
                        panic!("String length overruns entry length");
                    }
                    #fragment.write_all(#init.as_bytes()).unwrap();
                }
            }
            EntryType::U8 => {
                quote! {
                    #fragment.write_u8(#init).unwrap();
                }
            }
            EntryType::U16 => {
                quote! {
                    #fragment.write_u16::<#endianess>(#init).unwrap();
                }
            }
            EntryType::U32 => {
                quote! {
                    #fragment.write_u32::<#endianess>(#init).unwrap();
                }
            }
            EntryType::U64 => {
                quote! {
                    #fragment.write_u64::<#endianess>(#init).unwrap();
                }
            }
        };

        quote! {
            #set_position_expand
            #write_expand
        }
    }

    fn impl_register_entry(&self, base: &Base, endianess: Endianess) -> TokenStream {
        let ty = &self.entry_attr.ty;
        let len = self.entry_attr.len();

        let parse = {
            let main = match ty {
                EntryType::Str => quote! {
                    let str_end = data.iter().position(|c| *c == 0)
                        .ok_or_else(|| cameleon_impl::memory::MemoryError::InvalidEntryData("string entry must be null terminated".into()))?;
                    let result = std::str::from_utf8(&data[..str_end]).map_err(|e| cameleon_impl::memory::MemoryError::InvalidEntryData(format! {"{}", e}.into()))?;
                    if !result.is_ascii() {
                        return Err(cameleon_impl::memory::MemoryError::InvalidEntryData("string entry must be ASCII".into()));
                    }

                    Ok(result.to_string())
                },

                EntryType::U8 => quote! {
                    data.read_u8().map_err(|e| cameleon_impl::memory::MemoryError::InvalidEntryData(format! {"{}", e}.into()))
                },

                EntryType::U16 => quote! {
                    data.read_u16::<#endianess>().map_err(|e| cameleon_impl::memory::MemoryError::InvalidEntryData(format! {"{}", e}.into()))
                },

                EntryType::U32 => quote! {
                    data.read_u32::<#endianess>().map_err(|e| cameleon_impl::memory::MemoryError::InvalidEntryData(format! {"{}", e}.into()))
                },

                EntryType::U64 => quote! {
                    data.read_u64::<#endianess>().map_err(|e| cameleon_impl::memory::MemoryError::InvalidEntryData(format! {"{}", e}.into()))
                },
            };
            quote! {
                fn parse(mut data: &[u8]) -> cameleon_impl::memory::MemoryResult<Self::Ty> {
                    use cameleon_impl::byteorder::{#endianess, ReadBytesExt};
                    #main
                }
            }
        };

        let serialize = {
            let main = match ty {
                EntryType::Str => quote! {
                    if !data.is_ascii() {
                        return Err(cameleon_impl::memory::MemoryError::InvalidEntryData("string must be ASCII string".into()))
                    }

                    let mut result = data.into_bytes();
                    // Zero teminate.
                    match result.last() {
                        Some(0) => {}
                        _ => {result.push(0)}
                    }

                    if result.len() < #len {
                        result.resize(#len, 0);
                    } else if result.len() > #len {
                        return Err(cameleon_impl::memory::MemoryError::InvalidEntryData("data length is larger than the entry length".into()))
                    }
                },

                EntryType::U8 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u8(data).unwrap();
                },

                EntryType::U16 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u16::<#endianess>(data).unwrap();
                },

                EntryType::U32 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u32::<#endianess>(data).unwrap();
                },

                EntryType::U64 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u64::<#endianess>(data).unwrap();
                },
            };
            quote! {
                fn serialize(data: Self::Ty) -> cameleon_impl::memory::MemoryResult<Vec<u8>>
                {
                    use cameleon_impl::byteorder::{#endianess, WriteBytesExt};

                    #main

                    Ok(result)
                }
            }
        };

        let offset = self.offset;
        let raw_entry = quote! {
            fn raw_entry() -> cameleon_impl::memory::RawEntry {
                cameleon_impl::memory::RawEntry::new(#base as usize + #offset, #len)
            }
        };

        let ident = &self.ident;
        quote! {
            impl cameleon_impl::memory::RegisterEntry for #ident {
                type Ty = #ty;

                #parse
                #serialize
                #raw_entry
            }
        }
    }

    fn parse_entry_attr(variant: &mut syn::Variant) -> Result<EntryAttr> {
        let mut entry_attr = None;
        let mut i = 0;

        while i < variant.attrs.len() {
            match variant.attrs[i].path.get_ident() {
                Some(ident) if ident == "entry" => {
                    let attr = variant.attrs.remove(i);
                    if entry_attr.is_none() {
                        let attr: EntryAttr = syn::parse(attr.tokens.into())?;
                        entry_attr = Some(attr);
                    } else {
                        return Err(Error::new_spanned(attr, "duplicated entry attribute"));
                    }
                }

                _ => i += 1,
            }
        }

        if let Some(entry_attr) = entry_attr {
            Ok(entry_attr)
        } else {
            Err(Error::new_spanned(variant, "entry attributes must exist"))
        }
    }
}

struct EntryAttr {
    len: syn::LitInt,
    access: AccessRight,
    ty: EntryType,
}

impl EntryAttr {
    fn len(&self) -> usize {
        self.len.base10_parse().unwrap()
    }
}

impl syn::parse::Parse for EntryAttr {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let ts;
        syn::parenthesized!(ts in input);

        match ts.parse::<syn::Ident>()? {
            len if len == "len" => {}
            other => return Err(Error::new_spanned(other, "expected len")),
        };
        ts.parse::<syn::Token![=]>()?;
        let len = ts.parse::<syn::LitInt>()?;
        // Verify litint.
        len.base10_parse::<usize>()?;

        ts.parse::<syn::token::Comma>()?;
        match ts.parse::<syn::Ident>()? {
            access_right if access_right == "access" => {}
            other => return Err(Error::new_spanned(other, "expected access")),
        };
        ts.parse::<syn::Token![=]>()?;
        let access = AccessRight::from_ident(ts.parse::<syn::Ident>()?)?;

        ts.parse::<syn::token::Comma>()?;
        match ts.parse::<syn::Ident>()? {
            ty if ty == "ty" => {}
            other => return Err(Error::new_spanned(other, "expected ty")),
        };
        ts.parse::<syn::Token![=]>()?;
        let ty = EntryType::from_ident(ts.parse::<syn::Ident>()?)?;

        if ty.is_integral() && ty.integral_bits() / 8 != len.base10_parse().unwrap() {
            return Err(Error::new_spanned(
                len,
                "specified len doesn't fit with specified ty",
            ));
        }

        Ok(Self { len, access, ty })
    }
}

enum AccessRight {
    NA,
    RO,
    WO,
    RW,
}

impl AccessRight {
    fn from_ident(ident: syn::Ident) -> Result<Self> {
        if ident == "NA" {
            Ok(AccessRight::NA)
        } else if ident == "RO" {
            Ok(AccessRight::RO)
        } else if ident == "WO" {
            Ok(AccessRight::WO)
        } else if ident == "RW" {
            Ok(AccessRight::RW)
        } else {
            Err(Error::new_spanned(ident, "expected NA, RO, WO, or RW"))
        }
    }
}

impl quote::ToTokens for AccessRight {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use AccessRight::*;
        match self {
            NA => format_ident!("NA").to_tokens(tokens),
            RO => format_ident!("RO").to_tokens(tokens),
            WO => format_ident!("WO").to_tokens(tokens),
            RW => format_ident!("RW").to_tokens(tokens),
        }
    }
}

enum InitValue {
    LitStr(syn::LitStr),
    LitInt(syn::LitInt),
    Var(syn::Path),
}

impl InitValue {
    fn from_expr(expr: syn::Expr) -> Result<Self> {
        let error_msg = "only string literal, integer literal, or variable is allowed";
        match expr {
            syn::Expr::Lit(lit) => match lit.lit {
                syn::Lit::Str(lit_str) => Ok(InitValue::LitStr(lit_str)),
                syn::Lit::Int(lit_int) => Ok(InitValue::LitInt(lit_int)),
                other => Err(Error::new_spanned(other, error_msg)),
            },
            syn::Expr::Path(path) => Ok(InitValue::Var(path.path)),
            other => Err(Error::new_spanned(other, error_msg)),
        }
    }
}

impl quote::ToTokens for InitValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            InitValue::LitStr(string) => string.to_tokens(tokens),
            InitValue::LitInt(int) => int.to_tokens(tokens),
            InitValue::Var(path) => {
                let path = prepend_super_if_needed(path);
                path.to_tokens(tokens)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EntryType {
    Str,
    U8,
    U16,
    U32,
    U64,
}

impl EntryType {
    fn from_ident(ident: syn::Ident) -> Result<Self> {
        use EntryType::*;
        if ident == "String" || ident == "&str" {
            Ok(Str)
        } else if ident == "u8" {
            Ok(U8)
        } else if ident == "u16" {
            Ok(U16)
        } else if ident == "u32" {
            Ok(U32)
        } else if ident == "u64" {
            Ok(U64)
        } else {
            Err(Error::new_spanned(
                ident,
                "expected String, &str, u8, u16, u32 or u64",
            ))
        }
    }

    fn is_integral(&self) -> bool {
        use EntryType::*;
        match self {
            U8 | U16 | U32 | U64 => true,
            Str => false,
        }
    }

    fn integral_bits(&self) -> usize {
        use EntryType::*;
        match self {
            U8 => 8,
            U16 => 16,
            U32 => 32,
            U64 => 64,
            Str => panic!(),
        }
    }
}

impl quote::ToTokens for EntryType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use EntryType::*;
        match self {
            Str => syn::parse_str::<syn::Path>("std::string::String")
                .unwrap()
                .to_tokens(tokens),
            U8 => format_ident!("u8").to_tokens(tokens),
            U16 => format_ident!("u16").to_tokens(tokens),
            U32 => format_ident!("u32").to_tokens(tokens),
            U64 => format_ident!("u64").to_tokens(tokens),
        }
    }
}

struct Args {
    base: Base,
    endianess: Endianess,
}

#[derive(Debug, Clone, Copy)]
enum Endianess {
    BE,
    LE,
}

impl quote::ToTokens for Endianess {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Endianess::BE => format_ident!("BE").to_tokens(tokens),
            Endianess::LE => format_ident!("LE").to_tokens(tokens),
        }
    }
}

#[derive(Clone)]
enum Base {
    Lit(syn::LitInt),
    Var(syn::Path),
}

impl quote::ToTokens for Base {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Base::Lit(lit) => lit.to_tokens(tokens),
            Base::Var(path) => {
                let path = prepend_super_if_needed(path);
                path.to_tokens(tokens)
            }
        }
    }
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> Result<Args> {
        let ident = input.parse::<syn::Ident>()?;
        if ident != "base" {
            return Err(Error::new_spanned(
                ident,
                "expected `#[register(base = .., endianess = ..)]`",
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let base = input.parse::<syn::Expr>()?;
        let base = match base {
            syn::Expr::Lit(expr_lit) => {
                if let syn::Lit::Int(litint) = expr_lit.lit {
                    Base::Lit(litint)
                } else {
                    return Err(Error::new_spanned(
                        expr_lit,
                        "argument of offset attribute must be path or litint",
                    ));
                }
            }
            syn::Expr::Path(p) => Base::Var(p.path),
            other => {
                return Err(Error::new_spanned(
                    other,
                    "argument of offset attribute must be path or litint",
                ));
            }
        };

        input.parse::<syn::Token![,]>()?;
        let ident = input.parse::<syn::Ident>()?;
        if ident != "endianess" {
            return Err(Error::new_spanned(
                ident,
                "expected `#[register(base = .., endianess = ..)]`",
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let endianess = input.parse::<syn::Ident>()?;
        let endianess = if endianess == "BE" {
            Endianess::BE
        } else if endianess == "LE" {
            Endianess::LE
        } else {
            return Err(Error::new_spanned(
                endianess,
                "only BE or LE is allowed for endianess specifier",
            ));
        };

        Ok(Self { base, endianess })
    }
}

fn prepend_super_if_needed(path: &syn::Path) -> syn::Path {
    let ident = &path.segments[0];
    if ident.ident != "super" {
        return path.clone();
    }

    let trailing_super = format_ident!("super");
    syn::parse(quote! { #trailing_super::#path }.into()).unwrap()
}
