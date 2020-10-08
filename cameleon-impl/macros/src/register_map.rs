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
            reg.verify(args.endianness)?;
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
                use std::{convert::TryInto, ops::{Index, IndexMut}};

                use cameleon_impl::{memory::*, byteorder::{LE, BE, WriteBytesExt, ReadBytesExt}};

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
        let mut max_size = 0;
        for reg in &self.regs {
            let size = reg.offset + reg.reg_attr.len();
            max_size = std::cmp::max(max_size, size);
        }
        max_size
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

        let reg_offset = match reg_attr.specified_offset() {
            Some(specified_offset) => {
                *offset = specified_offset + reg_attr.len();
                specified_offset
            }
            None => {
                let reg_offset = *offset;
                *offset += reg_attr.len();
                reg_offset
            }
        };

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

    fn verify(&self, endianness: Endianness) -> Result<()> {
        match &self.reg_attr.ty {
            RegisterType::BitField(ref bf) => bf.verify(endianness),
            _ => Ok(()),
        }
    }

    fn init_reg(&self, memory_ident: &syn::Ident) -> TokenStream {
        if self.init.is_none() {
            return quote! {};
        }

        let init = self.init.as_ref().unwrap();
        let ident = &self.ident;
        match init {
            InitValue::Expr(_) => {
                quote! {
                    #ident::write(#init, #memory_ident).unwrap();
                }
            }
            _ => {
                quote! {
                    #ident::write(#init.try_into().unwrap(), #memory_ident).unwrap();
                }
            }
        }
    }

    fn impl_register(&self, base: &Base, endianness: Endianness) -> TokenStream {
        let ty = &self.reg_attr.ty;
        let len = self.reg_attr.len();

        let parse = self.impl_parse(endianness);
        let serialize = self.impl_serialize(endianness);
        let write = self.impl_write(endianness);

        let offset = self.offset;
        let raw = quote! {
            fn raw() -> RawRegister {
                RawRegister::new(#base as usize + #offset, #len)
            }
        };

        let helper_methods = self.impl_helper_methods(endianness);

        let ident = &self.ident;
        quote! {
            impl #ident {
                #helper_methods
            }

            impl Register for #ident {
                type Ty = #ty;

                #parse
                #serialize
                #raw
                #write
            }
        }
    }

    fn impl_helper_methods(&self, endianness: Endianness) -> TokenStream {
        match &self.reg_attr.ty {
            RegisterType::BitField(bf) => bf.impl_helper_methods(endianness),
            _ => quote! {},
        }
    }

    fn impl_parse(&self, endianness: Endianness) -> TokenStream {
        let ty = &self.reg_attr.ty;
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

            RegisterType::BitField(bf) => {
                let read_integral = format_ident!("read_{}", bf.ty.associated_ty());
                let value = if bf.ty.integral_bits() == 8 {
                    quote! {
                        data.#read_integral().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))?
                    }
                } else {
                    quote! {
                        data.#read_integral::<#endianness>().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))?
                    }
                };
                let lsb = bf.lsb(endianness);
                let msb = bf.msb(endianness);
                let bits_len = msb - lsb;

                if bf.ty.is_signed() {
                    quote! {
                        let mut value = #value;
                        value &= Self::mask();
                        value >>= #lsb;
                        if ((1 << #bits_len) & value) != 0 {
                            // Sext.
                            let ext = -1 ^ (Self::mask() >> #lsb);
                            value |= ext;
                        }
                        Ok(value)
                    }
                } else {
                    quote! {
                        let mut value = #value;
                        value &= Self::mask();
                        value >>= #lsb;
                        Ok(value)
                    }
                }
            }

            _ => {
                let read_integral = format_ident!("read_{}", ty.associated_ty());
                if ty.numerical_bits() == 8 {
                    quote! {
                        data.#read_integral().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))
                    }
                } else {
                    quote! {
                        data.#read_integral::<#endianness>().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))
                    }
                }
            }
        };
        quote! {
            fn parse(mut data: &[u8]) -> MemoryResult<Self::Ty> {
                #main
            }
        }
    }

    fn impl_serialize(&self, endianness: Endianness) -> TokenStream {
        let ty = &self.reg_attr.ty;
        let len = self.reg_attr.len();
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
                    return Err(MemoryError::InvalidRegisterData("data length is larger than the reg length".into()));
                }
            },

            RegisterType::BitField(ref bf) => {
                let write_integral = format_ident!("write_{}", ty.associated_ty());
                let serialize_to_bytes = if bf.ty.integral_bits() == 8 {
                    quote! {
                        let mut result = std::vec::Vec::with_capacity(#len);
                        result.#write_integral(data).unwrap();
                    }
                } else {
                    quote! {
                        let mut result = std::vec::Vec::with_capacity(#len);
                        result.#write_integral::<#endianness>(data).unwrap();
                    }
                };

                quote! {
                   let data = Self::masked_int(data)?;
                   #serialize_to_bytes
                }
            }

            _ => {
                let write_integral = format_ident!("write_{}", ty.associated_ty());
                if ty.numerical_bits() == 8 {
                    quote! {
                        let mut result = std::vec::Vec::with_capacity(#len);
                        result.#write_integral(data).unwrap();
                    }
                } else {
                    quote! {
                        let mut result = std::vec::Vec::with_capacity(#len);
                        result.#write_integral::<#endianness>(data).unwrap();
                    }
                }
            }
        };

        quote! {
            fn serialize(data: Self::Ty) -> MemoryResult<Vec<u8>>
            {
                #main

                Ok(result)
            }
        }
    }

    fn impl_write(&self, endianness: Endianness) -> TokenStream {
        match &self.reg_attr.ty {
            RegisterType::BitField(ref bf) => {
                let read_integral = format_ident!("read_{}", bf.ty.associated_ty());
                let write_integral = format_ident!("write_{}", bf.ty.associated_ty());
                if bf.ty.integral_bits() == 8 {
                    quote! {
                        fn write(data: Self::Ty, memory: &mut[u8]) -> MemoryResult<()> {
                            let range = Self::raw().range();
                            let data = Self::masked_int(data)?;
                            let original_data = memory.index(range.clone()).#read_integral().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))?;
                            let new_data = (original_data & !Self::mask()) | data;
                            memory.index_mut(range).#write_integral(new_data).unwrap();
                            Ok(())
                        }
                    }
                } else {
                    quote! {
                        fn write(data: Self::Ty, memory: &mut[u8]) -> MemoryResult<()> {
                            let range = Self::raw().range();
                            let data = Self::masked_int(data)?;
                            let original_data = memory.index(range.clone()).#read_integral::<#endianness>().map_err(|e| MemoryError::InvalidRegisterData(format! {"{}", e}.into()))?;
                            let new_data = (original_data & !Self::mask()) | data;
                            memory.index_mut(range).#write_integral::<#endianness>(new_data).unwrap();
                            Ok(())
                        }
                    }
                }
            }

            _ => quote! {},
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
    offset: Option<syn::LitInt>,
}

impl RegisterAttr {
    fn len(&self) -> usize {
        self.len.base10_parse().unwrap()
    }

    fn specified_offset(&self) -> Option<usize> {
        match &self.offset {
            Some(offset) => Some(offset.base10_parse().unwrap()),
            _ => None,
        }
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
        let ty = ts.parse::<RegisterType>()?;

        let base_ty = ty.base_ty();
        if base_ty.is_numerical() && base_ty.numerical_bits() / 8 != len.base10_parse().unwrap() {
            return Err(Error::new_spanned(
                len,
                "specified len doesn't fit with specified ty",
            ));
        }

        let offset = if ts.parse::<syn::token::Comma>().is_ok() {
            match ts.parse::<syn::Ident>()? {
                offset if offset == "offset" => {}
                other => return Err(Error::new_spanned(other, "expected offset")),
            }
            ts.parse::<syn::Token![=]>()?;
            let offset = ts.parse::<syn::LitInt>()?;
            Some(offset)
        } else {
            None
        };

        Ok(Self {
            len,
            access,
            ty,
            offset,
        })
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
    LitFloat(syn::LitFloat),
    Array(syn::ExprArray),
    Var(syn::Path),
    Expr(Box<syn::Expr>),
}

impl InitValue {
    fn from_expr(expr: syn::Expr) -> Result<Self> {
        let error_msg = "only string literal, integer literal, or variable is allowed";
        match expr {
            syn::Expr::Lit(lit) => match lit.lit {
                syn::Lit::Str(lit_str) => Ok(InitValue::LitStr(lit_str)),
                syn::Lit::Int(lit_int) => Ok(InitValue::LitInt(lit_int)),
                syn::Lit::Float(lit_float) => Ok(InitValue::LitFloat(lit_float)),
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

            other => Ok(InitValue::Expr(other.into())),
        }
    }
}

impl quote::ToTokens for InitValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            InitValue::LitStr(string) => string.to_tokens(tokens),
            InitValue::LitInt(int) => int.to_tokens(tokens),
            InitValue::LitFloat(float) => float.to_tokens(tokens),
            InitValue::Array(arr) => arr.to_tokens(tokens),
            InitValue::Expr(expr) => expr.to_tokens(tokens),
            InitValue::Var(path) => {
                let path = prepend_super_if_needed(path);
                path.to_tokens(tokens)
            }
        }
    }
}

#[derive(Clone)]
enum RegisterType {
    Str,
    Bytes,
    BitField(BitField),
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl RegisterType {
    fn is_integral(&self) -> bool {
        use RegisterType::*;
        match self {
            Str | Bytes | BitField(..) | F32 | F64 => false,
            _ => true,
        }
    }

    fn is_numerical(&self) -> bool {
        use RegisterType::*;
        match self {
            Str | Bytes | BitField(..) => false,
            _ => true,
        }
    }

    fn is_signed(&self) -> bool {
        use RegisterType::*;
        match self {
            I8 | I16 | I32 | I64 => true,
            U8 | U16 | U32 | U64 => false,
            _ => panic!(),
        }
    }

    fn integral_bits(&self) -> usize {
        use RegisterType::*;
        match self {
            U8 | I8 => 8,
            U16 | I16 => 16,
            U32 | I32 => 32,
            U64 | I64 => 64,
            _ => panic!(),
        }
    }

    fn numerical_bits(&self) -> usize {
        use RegisterType::*;
        match self {
            U8 | I8 => 8,
            U16 | I16 => 16,
            U32 | I32 | F32 => 32,
            U64 | I64 | F64 => 64,
            _ => panic!(),
        }
    }

    fn associated_ty(&self) -> &str {
        use RegisterType::*;
        match self {
            Str => "std::string::String",
            Bytes => "Vec<u8>",
            BitField(bf) => bf.ty.associated_ty(),
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            I8 => "i8",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            F32 => "f32",
            F64 => "f64",
        }
    }

    fn base_ty(&self) -> Self {
        match self {
            RegisterType::BitField(bf) => *bf.ty.clone(),
            _ => self.clone(),
        }
    }
}

impl syn::parse::Parse for RegisterType {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        use RegisterType::*;

        let ident = input.parse::<syn::Ident>()?;
        let err_msg =
            "expected String, Bytes, BitField<ty, LSB = .., MSB = ..>, or primitive numerical types";

        match ident {
            _ if ident == "String" => Ok(Str),
            _ if ident == "Bytes" => Ok(Bytes),
            _ if ident == "u8" => Ok(U8),
            _ if ident == "u16" => Ok(U16),
            _ if ident == "u32" => Ok(U32),
            _ if ident == "u64" => Ok(U64),
            _ if ident == "i8" => Ok(I8),
            _ if ident == "i16" => Ok(I16),
            _ if ident == "i32" => Ok(I32),
            _ if ident == "i64" => Ok(I64),
            _ if ident == "f32" => Ok(F32),
            _ if ident == "f64" => Ok(F64),
            _ if ident == "BitField" => Ok(BitField(input.parse()?)),
            _ => Err(Error::new_spanned(ident, err_msg)),
        }
    }
}

#[derive(Clone)]
struct BitField {
    ty: Box<RegisterType>,
    lsb: syn::LitInt,
    msb: syn::LitInt,
}

impl BitField {
    fn lsb(&self, endianness: Endianness) -> usize {
        let len = self.ty.integral_bits();
        match endianness {
            Endianness::LE => self.lsb.base10_parse().unwrap(),
            Endianness::BE => (len - self.lsb.base10_parse::<usize>().unwrap() - 1),
        }
    }

    fn msb(&self, endianness: Endianness) -> usize {
        let len = self.ty.integral_bits();
        match endianness {
            Endianness::LE => self.msb.base10_parse().unwrap(),
            Endianness::BE => (len - self.msb.base10_parse::<usize>().unwrap() - 1),
        }
    }

    fn min(&self, endianness: Endianness) -> i64 {
        if self.ty.is_signed() {
            let lsb = self.lsb(endianness);
            let msb = self.msb(endianness);
            let value = 1 << (msb - lsb) as i64;
            -value
        } else {
            0
        }
    }

    fn max(&self, endianness: Endianness) -> i64 {
        let lsb = self.lsb(endianness);
        let msb = self.msb(endianness);
        if self.ty.is_signed() {
            (1 << (msb - lsb)) - 1
        } else {
            (1 << (msb - lsb + 1)) - 1
        }
    }

    fn impl_helper_methods(&self, endianness: Endianness) -> TokenStream {
        let lsb = self.lsb(endianness);
        let msb = self.msb(endianness);
        let ty = &self.ty;
        let min = self.min(endianness);
        let max = self.max(endianness);
        let ty_bits = ty.integral_bits();

        let mask = if ty.is_signed() {
            quote! {
                fn mask() -> #ty {
                    let mask1 = if #ty_bits - 1 == #msb {
                        -1
                    } else if #ty_bits - 2 == #msb {
                        #ty::MAX
                    } else {
                        (1 << #msb + 1) - 1
                    };

                    let mask2 = if #ty_bits -1 == #lsb {
                        #ty::MAX
                    } else {
                        !((1 << #lsb ) - 1)
                    };

                    mask1 & mask2
                }
            }
        } else {
            quote! {
                const fn mask() -> #ty {
                    let mask1 = if #ty_bits - 1 == #msb {
                        #ty::MAX
                    } else {
                        (1 << #msb + 1) - 1
                    };
                    let mask2 = !((1 << #lsb) - 1);
                    mask1 & mask2
                }
            }
        };

        quote! {
            #mask

            const fn min() -> #ty {
                #min as #ty
            }

            const fn max() -> #ty {
                #max as #ty
            }

            fn masked_int(data: #ty) -> MemoryResult<#ty> {
                let min = Self::min();
                let max = Self::max();
                if data < min  || data > max {
                    let err_msg = format!("data doesn't fit within ({}..={})", min, max);
                    return Err(MemoryError::InvalidRegisterData(err_msg.into()));
                }

                let mut data = data << #lsb;
                data &= Self::mask();
                Ok(data)
            }
        }
    }

    fn verify(&self, endianness: Endianness) -> Result<()> {
        if self.lsb(endianness) > self.msb(endianness) {
            return Err(syn::Error::new_spanned(
                &self.lsb,
                "expectd LSB < MSB if endianness = LE, else MSB > LSB if endianness = BE",
            ));
        }

        let len = self.ty.integral_bits();
        if self.msb(endianness) >= len {
            return Err(syn::Error::new_spanned(
                &self.msb,
                "msb exceeds register length",
            ));
        }

        Ok(())
    }
}

impl syn::parse::Parse for BitField {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        input.parse::<syn::token::Lt>()?;

        let ty_cursor = input.cursor();
        let ty: RegisterType = input.parse()?;
        if !ty.is_integral() {
            return Err(syn::Error::new(
                ty_cursor.span(),
                "expected integral primitive",
            ));
        }

        input.parse::<syn::token::Comma>()?;
        let ident = input.parse::<syn::Ident>()?;
        if ident != "LSB" {
            return Err(syn::Error::new_spanned(ident, "expected LSB"));
        }
        input.parse::<syn::Token![=]>()?;
        let lsb = input.parse::<syn::LitInt>()?;

        input.parse::<syn::token::Comma>()?;
        let ident = input.parse::<syn::Ident>()?;
        if ident != "MSB" {
            return Err(syn::Error::new_spanned(ident, "expected MSB"));
        }
        input.parse::<syn::Token![=]>()?;
        let msb = input.parse::<syn::LitInt>()?;

        input.parse::<syn::token::Gt>()?;

        Ok(BitField {
            ty: ty.into(),
            lsb,
            msb,
        })
    }
}

impl quote::ToTokens for RegisterType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        syn::parse_str::<syn::Path>(self.associated_ty())
            .unwrap()
            .to_tokens(tokens);
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
