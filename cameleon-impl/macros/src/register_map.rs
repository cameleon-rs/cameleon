use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let register_enum = RegisterMap::parse(args, input)?;

    let expanded_module = register_enum.define_module();

    Ok(proc_macro::TokenStream::from(quote! {
            #expanded_module
    }))
}

struct RegisterMap {
    ident: syn::Ident,
    vis: syn::Visibility,
    args: Args,
    regs: Vec<Register>,
    attrs: Vec<syn::Attribute>,
}

impl RegisterMap {
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
        let mut regs = vec![];
        for variant in input_enum.variants.into_iter() {
            let reg = Register::parse(variant, &mut offset)?;
            regs.push(reg);
        }

        if regs.is_empty() {
            return Err(Error::new(span, "at least one variant is required"));
        }

        Ok(Self {
            ident,
            vis,
            args,
            regs,
            attrs: input_enum.attrs,
        })
    }

    fn define_module(&self) -> TokenStream {
        let mod_name = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        let vis_inside_mod = self.modify_visibility();

        let structs = self.regs.iter().map(|reg| {
            let ident = &reg.ident;
            let attrs = reg.attrs.iter();
            quote! {
                #(#attrs)*
                #vis_inside_mod struct #ident {}
            }
        });

        let init_raw_memory = self.impl_init_raw_memory();
        let init_memory_protection = self.impl_init_memory_protection();
        let base = self.const_base();
        let size = self.const_size();
        let impl_register = self.impl_register();

        quote! {
            #(#attrs)*
            #[allow(non_snake_case)]
            #[allow(clippy::string_lit_as_bytes)]
            #vis mod #mod_name {
                use std::convert::TryInto;

                use cameleon_impl::memory::*;

                use super::*;


                #base
                #size
                #init_raw_memory
                #init_memory_protection
                #impl_register
                #(#structs)*
            }
        }
    }

    fn impl_register(&self) -> TokenStream {
        let impls = self
            .regs
            .iter()
            .map(|reg| reg.impl_register(&self.args.base, self.args.endianness));

        quote! {
            #(#impls)*
        }
    }

    fn impl_init_memory_protection(&self) -> TokenStream {
        let set_access_right = self.regs.iter().map(|reg| {
            let ident = &reg.ident;
            let access_right = &reg.reg_attr.access;
            quote! {
                let range = #ident::raw().range();
                memory_protection.set_access_right_with_range(range, AccessRight::#access_right);
            }
        });

        let vis = self.modify_visibility();
        quote! {
            #vis fn init_memory_protection(memory_protection: &mut MemoryProtection) {
                #(#set_access_right)*
            }
        }
    }

    fn impl_init_raw_memory(&self) -> TokenStream {
        let memory_ident = format_ident!("memory");
        let mut writes = vec![];
        for reg in &self.regs {
            writes.push(reg.init_reg(&memory_ident));
        }

        let vis = self.modify_visibility();
        quote! {
            #vis fn init_raw_memory(#memory_ident: &mut [u8]) {
                #(#writes)*
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
        let last_field = self.regs.last().unwrap();
        last_field.offset + last_field.reg_attr.len()
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

struct Register {
    ident: syn::Ident,
    offset: usize,
    reg_attr: RegisterAttr,
    init: Option<InitValue>,
    attrs: Vec<syn::Attribute>,
}

impl Register {
    fn parse(mut variant: syn::Variant, offset: &mut usize) -> Result<Self> {
        let reg_attr = Self::parse_reg_attr(&mut variant)?;
        let ident = variant.ident;
        let reg_offset = *offset;

        *offset += reg_attr.len();

        let init = if let Some((_, expr)) = variant.discriminant {
            Some(InitValue::from_expr(expr)?)
        } else {
            None
        };

        Ok(Self {
            ident,
            offset: reg_offset,
            reg_attr,
            init,
            attrs: variant.attrs,
        })
    }

    fn init_reg(&self, memory_ident: &syn::Ident) -> TokenStream {
        if self.init.is_none() {
            return quote! {};
        }

        let init = self.init.as_ref().unwrap();
        let ident = &self.ident;
        quote! {
            #ident::write(#init.try_into().unwrap(), #memory_ident).unwrap();
        }
    }

    fn impl_register(&self, base: &Base, endianness: Endianness) -> TokenStream {
        let ty = &self.reg_attr.ty;
        let len = self.reg_attr.len();

        let parse = {
            let main = match ty {
                RegisterType::Str => quote! {
                    let str_end = data.iter().position(|c| *c == 0)
                        .ok_or_else(|| MemoryError::InvalidRegisterData("string reg must be null terminated".into()))?;
                    let result = std::str::from_utf8(&data[..str_end]).map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))?;
                    if !result.is_ascii() {
                        return Err(MemoryError::InvalidRegisterData("string reg must be ASCII".into()));
                    }

                    Ok(result.to_string())
                },

                RegisterType::Bytes => quote! {
                    Ok(data.into())
                },

                RegisterType::U8 => quote! {
                    data.read_u8().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))
                },

                RegisterType::U16 => quote! {
                    data.read_u16::<#endianness>().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))
                },

                RegisterType::U32 => quote! {
                    data.read_u32::<#endianness>().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))
                },

                RegisterType::U64 => quote! {
                    data.read_u64::<#endianness>().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))
                },
            };
            quote! {
                fn parse(mut data: &[u8]) -> MemoryResult<Self::Ty> {
                    use cameleon_impl::byteorder::{#endianness, ReadBytesExt};
                    #main
                }
            }
        };

        let serialize = {
            let main = match ty {
                RegisterType::Str => quote! {
                    if !data.is_ascii() {
                        return Err(MemoryError::InvalidRegisterData("string must be ASCII string".into()))
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
                        return Err(MemoryError::InvalidRegisterData("data length is larger than the reg length".into()))
                    }
                },

                RegisterType::Bytes => quote! {
                    let result = data;
                    if result.len() != #len {
                        return Err(MemoryError::InvalidRegisterData("data length is larget than the reg length".into()));
                    }
                },

                RegisterType::U8 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u8(data).unwrap();
                },

                RegisterType::U16 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u16::<#endianness>(data).unwrap();
                },

                RegisterType::U32 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u32::<#endianness>(data).unwrap();
                },

                RegisterType::U64 => quote! {
                    let mut result = std::vec::Vec::with_capacity(#len);
                    result.write_u64::<#endianness>(data).unwrap();
                },
            };

            quote! {
                fn serialize(data: Self::Ty) -> MemoryResult<Vec<u8>>
                {
                    use cameleon_impl::byteorder::{#endianness, WriteBytesExt};

                    #main

                    Ok(result)
                }
            }
        };

        let offset = self.offset;
        let raw = quote! {
            fn raw() -> RawRegister {
                RawRegister::new(#base as usize + #offset, #len)
            }
        };

        let ident = &self.ident;
        quote! {
            impl Register for #ident {
                type Ty = #ty;

                #parse
                #serialize
                #raw
            }
        }
    }

    fn parse_reg_attr(variant: &mut syn::Variant) -> Result<RegisterAttr> {
        let mut reg_attr = None;
        let mut i = 0;

        while i < variant.attrs.len() {
            match variant.attrs[i].path.get_ident() {
                Some(ident) if ident == "register" => {
                    let attr = variant.attrs.remove(i);
                    if reg_attr.is_none() {
                        let attr: RegisterAttr = syn::parse(attr.tokens.into())?;
                        reg_attr = Some(attr);
                    } else {
                        return Err(Error::new_spanned(attr, "duplicated register attribute"));
                    }
                }

                _ => i += 1,
            }
        }

        if let Some(reg_attr) = reg_attr {
            Ok(reg_attr)
        } else {
            Err(Error::new_spanned(
                variant,
                "register attributes must exist",
            ))
        }
    }
}

struct RegisterAttr {
    len: syn::LitInt,
    access: AccessRight,
    ty: RegisterType,
}

impl RegisterAttr {
    fn len(&self) -> usize {
        self.len.base10_parse().unwrap()
    }
}

impl syn::parse::Parse for RegisterAttr {
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
        let ty = RegisterType::from_ident(ts.parse::<syn::Ident>()?)?;

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
    Array(syn::ExprArray),
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

            syn::Expr::Reference(ref_expr) => {
                if let syn::Expr::Array(arr) = *ref_expr.expr {
                    Ok(InitValue::Array(arr))
                } else {
                    Err(Error::new_spanned(
                        ref_expr.expr,
                        "only &[.., .., ..] is accepted",
                    ))
                }
            }

            other => Err(Error::new_spanned(other, error_msg)),
        }
    }
}

impl quote::ToTokens for InitValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            InitValue::LitStr(string) => string.to_tokens(tokens),
            InitValue::LitInt(int) => int.to_tokens(tokens),
            InitValue::Array(arr) => arr.to_tokens(tokens),
            InitValue::Var(path) => {
                let path = prepend_super_if_needed(path);
                path.to_tokens(tokens)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RegisterType {
    Str,
    Bytes,
    U8,
    U16,
    U32,
    U64,
}

impl RegisterType {
    fn from_ident(ident: syn::Ident) -> Result<Self> {
        use RegisterType::*;
        if ident == "String" {
            Ok(Str)
        } else if ident == "Bytes" {
            Ok(Bytes)
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
                "expected String, u8, u16, u32, u64, or Bytes",
            ))
        }
    }

    fn is_integral(&self) -> bool {
        use RegisterType::*;
        match self {
            U8 | U16 | U32 | U64 => true,
            Str | Bytes => false,
        }
    }

    fn integral_bits(&self) -> usize {
        use RegisterType::*;
        match self {
            U8 => 8,
            U16 => 16,
            U32 => 32,
            U64 => 64,
            Str | Bytes => panic!(),
        }
    }
}

impl quote::ToTokens for RegisterType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use RegisterType::*;
        match self {
            Str => syn::parse_str::<syn::Path>("std::string::String")
                .unwrap()
                .to_tokens(tokens),
            U8 => format_ident!("u8").to_tokens(tokens),
            U16 => format_ident!("u16").to_tokens(tokens),
            U32 => format_ident!("u32").to_tokens(tokens),
            U64 => format_ident!("u64").to_tokens(tokens),
            Bytes => syn::parse_str::<syn::Path>("Vec<u8>")
                .unwrap()
                .to_tokens(tokens),
        }
    }
}

struct Args {
    base: Base,
    endianness: Endianness,
}

#[derive(Debug, Clone, Copy)]
enum Endianness {
    BE,
    LE,
}

impl quote::ToTokens for Endianness {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Endianness::BE => format_ident!("BE").to_tokens(tokens),
            Endianness::LE => format_ident!("LE").to_tokens(tokens),
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
                "expected `#[register_map(base = .., endianness = ..)]`",
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
        if ident != "endianness" {
            return Err(Error::new_spanned(
                ident,
                "expected `#[register_map(base = .., endianness = ..)]`",
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let endianness = input.parse::<syn::Ident>()?;
        let endianness = if endianness == "BE" {
            Endianness::BE
        } else if endianness == "LE" {
            Endianness::LE
        } else {
            return Err(Error::new_spanned(
                endianness,
                "only BE or LE is allowed for endianness specifier",
            ));
        };

        Ok(Self { base, endianness })
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
