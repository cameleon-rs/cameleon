use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let register_enum = RegisterEnum::parse(args, input)?;

    let expanded_enum = register_enum.define_enum();
    let impl_memory_fragment = register_enum.impl_memory_fragment()?;

    Ok(proc_macro::TokenStream::from(quote! {
            #expanded_enum
            #impl_memory_fragment
    }))
}

struct RegisterEnum {
    ident: syn::Ident,
    vis: syn::Visibility,
    endianess: Endianess,
    entries: Vec<RegisterEntry>,
    attrs: Vec<syn::Attribute>,
}

impl RegisterEnum {
    fn parse(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> Result<Self> {
        let input_enum: syn::ItemEnum = syn::parse(input)?;
        let span = input_enum.span();

        let ident = input_enum.ident;
        let vis = input_enum.vis;

        let endianess: Endianess = syn::parse(args)?;

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
            endianess,
            entries,
            attrs: input_enum.attrs,
        })
    }

    fn define_enum(&self) -> TokenStream {
        let enum_name = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        let variants = self.entries.iter().map(|entry| {
            let ident = &entry.ident;
            let attrs = entry.attrs.iter();
            quote! {
                #(#attrs)*
                #ident
            }
        });

        quote! {
            #(#attrs)*
            #vis enum #enum_name {
                #(#variants),*
            }
        }
    }

    fn impl_local_raw_entry(&self) -> TokenStream {
        let enum_ident = &self.ident;
        let arms = self.entries.iter().map(|entry| {
            let ident = &entry.ident;
            let offset = entry.offset;
            let len = entry.entry_attr.len();
            quote! {
                 #enum_ident::#ident => cameleon_impl::memory::RawEntry::new(#offset, #len)
            }
        });

        quote! {
            #[doc(hidden)]
            fn local_raw_entry(&self) -> cameleon_impl::memory::RawEntry {
                match self {
                    #(#arms,)*
                }
            }
        }
    }

    fn impl_memory_fragment(&self) -> Result<TokenStream> {
        let memory_protection = self.impl_memory_protection();
        let fragment = self.impl_frament()?;
        let ident = &self.ident;
        let size = self.size();
        let raw_entry_local = self.impl_local_raw_entry();

        Ok(quote! {
            impl cameleon_impl::memory::MemoryFragment for #ident {
              const SIZE: usize = #size;
              #memory_protection
              #fragment
              #raw_entry_local
            }
        })
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
        quote! {
            fn memory_protection() -> cameleon_impl::memory::MemoryProtection {
                let mut memory_protection = cameleon_impl::memory::MemoryProtection::new(#size);
                #(#set_access_right)*
                memory_protection
            }
        }
    }

    fn impl_frament(&self) -> Result<TokenStream> {
        let fragment = format_ident!("fragment");
        let mut writes = vec![];
        for entry in &self.entries {
            writes.push(entry.write_to_fragment(fragment.clone(), self.endianess)?);
        }

        let endianess = self.endianess;
        let size = self.size();
        Ok(quote! {
            fn fragment() -> Vec<u8> {
                use cameleon_impl::byteorder::{#endianess, WriteBytesExt};
                use std::io::Write;
                let mut fragment_base = vec![0; #size];
                let mut #fragment = std::io::Cursor::new(fragment_base.as_mut_slice());
                #(#writes)*
                fragment_base
            }
        })
    }

    fn size(&self) -> usize {
        let last_field = self.entries.last().unwrap();
        last_field.offset + last_field.entry_attr.len()
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

    fn write_to_fragment(&self, fragment: syn::Ident, endianess: Endianess) -> Result<TokenStream> {
        if self.init.is_none() {
            return Ok(quote! {});
        }

        let init = self.init.as_ref().unwrap();
        let start = self.offset as u64;
        let endianess = endianess;

        let set_position_expand = quote! {#fragment.set_position(#start);};

        let write_expand = match self.infer_init_ty()?.unwrap() {
            EntryType::Str => {
                let len = self.entry_attr.len();
                quote! {
                    if #len < #init.as_bytes().len() {
                        panic!("String length overruns entry length");
                    }
                    #fragment.write(#init.as_bytes()).unwrap();
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

        Ok(quote! {
            #set_position_expand
            #write_expand
        })
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

    fn infer_init_ty(&self) -> Result<Option<EntryType>> {
        if self.init.is_none() {
            return Ok(None);
        }

        match self.init.as_ref().unwrap() {
            InitValue::LitStr(string) => match self.entry_attr.ty {
                Some(_) => Err(Error::new_spanned(
                    string,
                    "ty attribute can't be accepted when the initial value is specified as literal",
                )),
                None => Ok(Some(EntryType::Str)),
            },

            InitValue::LitInt(int) => match self.entry_attr.ty {
                Some(_) => Err(Error::new_spanned(
                    int,
                    "ty attribute can't be accepted when the initial value is specified as literal",
                )),
                None => Ok(Some(EntryType::integral_from_size(
                    self.entry_attr.len() * 8,
                ))),
            },

            InitValue::Var(var) => match self.entry_attr.ty {
                Some(ty) => {
                    if ty.is_integral() && self.entry_attr.len() != ty.integral_bits() / 8 {
                        Err(Error::new_spanned(
                            var,
                            "specified len doesn't fit with specified ty",
                        ))
                    } else {
                        Ok(Some(ty))
                    }
                }
                None => Err(Error::new_spanned(
                    var,
                    "ty attribute is required when initial value is specified by ident",
                )),
            },
        }
    }
}

struct EntryAttr {
    len: syn::LitInt,
    access: AccessRight,
    ty: Option<EntryType>,
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
        len.base10_parse::<usize>()?;
        ts.parse::<syn::token::Comma>()?;

        match ts.parse::<syn::Ident>()? {
            access_right if access_right == "access" => {}
            other => return Err(Error::new_spanned(other, "expected access")),
        };
        ts.parse::<syn::Token![=]>()?;
        let access = AccessRight::from_ident(ts.parse::<syn::Ident>()?)?;

        let ty = if let Ok(_) = ts.parse::<syn::token::Comma>() {
            match ts.parse::<syn::Ident>()? {
                ty if ty == "ty" => {}
                other => return Err(Error::new_spanned(other, "expected type")),
            }
            ts.parse::<syn::Token![=]>()?;
            Some(EntryType::from_ident(ts.parse::<syn::Ident>()?)?)
        } else {
            None
        };

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
            InitValue::Var(var) => var.to_tokens(tokens),
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

    fn integral_from_size(size: usize) -> Self {
        use EntryType::*;
        match size {
            8 => U8,
            16 => U16,
            32 => U32,
            64 => U64,
            _ => panic!(),
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

impl syn::parse::Parse for Endianess {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        if ident != "endianess" {
            return Err(Error::new_spanned(
                ident,
                "args must be `endianess = LE` or `endianess = BE`",
            ));
        }

        input.parse::<syn::Token![=]>()?;

        let endianess = input.parse::<syn::Ident>()?;
        if endianess == "BE" {
            Ok(Endianess::BE)
        } else if endianess == "LE" {
            Ok(Endianess::LE)
        } else {
            Err(Error::new_spanned(
                endianess,
                "only BE or LE is allowed for endianess specifier",
            ))
        }
    }
}
