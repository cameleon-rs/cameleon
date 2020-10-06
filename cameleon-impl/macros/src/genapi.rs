use cameleon_impl_genapi_parser as genapi_parser;
use genapi_parser::register_node_elem;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let genapi_def = GenApiDefinition::parse(args, input)?;

    let ts = genapi_def.expand()?;
    Ok(ts.into())
}

struct GenApiDefinition {
    xml: XML,
    args: Args,
}

impl GenApiDefinition {
    fn parse(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> Result<Self> {
        let args: Args = syn::parse(args)?;

        let xml: XML = syn::parse(input)?;

        Ok(Self { args, xml })
    }

    fn expand(&self) -> Result<TokenStream> {
        let ident = &self.xml.ident;
        let vis = &self.xml.vis;
        let endianness = &self.args.endianness;

        let regs = self.xml.expand_registers(&self.args)?;

        Ok(quote! {
            #[cameleon_impl::memory::register_map(base = 0, endianness = #endianness)]
            #vis enum #ident {
                #(#regs,)*
            }
        })
    }
}

struct XML {
    ident: syn::Ident,
    vis: syn::Visibility,
    xml_tag: syn::Ident,
    xml_str: syn::LitStr,
    #[allow(unused)]
    reg_desc: genapi_parser::RegisterDescription,
}

impl XML {
    fn expand_registers(&self, args: &Args) -> Result<Vec<TokenStream>> {
        let mut registers = self.expand_reg_desc()?;
        registers.push(self.expand_xml(args));

        Ok(registers)
    }

    #[allow(unused)]
    fn expand_reg_desc(&self) -> Result<Vec<TokenStream>> {
        use genapi_parser::NodeKind::*;

        let mut regs = vec![];

        for node in self.reg_desc.nodes() {
            match node {
                IntReg(node) => regs.push(self.expand_int_reg(node)?),
                MaskedIntReg(node) => regs.push(self.expand_masked_int_reg(node)?),
                FloatReg(node) => regs.push(self.expand_float_reg(node)?),
                StringReg(node) => regs.push(self.expand_string_reg(node)?),
                _ => todo!(),
            };
        }

        Ok(regs)
    }

    fn expand_xml(&self, args: &Args) -> TokenStream {
        let xml_tag = &self.xml_tag;
        let xml_str = &self.xml_str;
        let xml_str_len = self.xml_str.value().as_bytes().len();
        let xml_base = &args.xml_base;

        quote! {
            #[register(len = #xml_str_len, access = RO, ty = Bytes, offset = #xml_base)]
            #xml_tag = #xml_str.as_bytes().into()
        }
    }

    fn expand_int_reg(&self, node: &genapi_parser::IntRegNode) -> Result<TokenStream> {
        let node_base = node.node_base();
        let name = node_base.name();
        let (addr, len, access) = self.register_attr(name, node.register_base())?;
        let sign = node.sign();
        let ty = self.int_ty_from(name, len, sign)?;

        let name = format_ident!("{}", name);
        Ok(quote! {
            #[register(len = #len, access = #access, ty = #ty, offset = #addr)]
            #name
        })
    }

    fn expand_masked_int_reg(&self, node: &genapi_parser::MaskedIntRegNode) -> Result<TokenStream> {
        use register_node_elem::BitMask;

        let node_base = node.node_base();
        let name = node_base.name();
        let (addr, len, access) = self.register_attr(name, node.register_base())?;
        let sign = node.sign();
        let ty = self.int_ty_from(name, len, sign)?;

        let (lsb, msb) = match node.bit_mask() {
            BitMask::SingleBit(bit) => (bit, bit),
            BitMask::Range { lsb, msb } => (lsb, msb),
        };

        let name = format_ident!("{}", name);
        Ok(quote! {
            #[register(len = #len, access = #access, ty = BitField<#ty, LSB=#lsb, MSB=#msb>, offset = #addr)]
            #name
        })
    }

    fn expand_float_reg(&self, node: &genapi_parser::FloatRegNode) -> Result<TokenStream> {
        let node_base = node.node_base();
        let name = node_base.name();
        let (addr, len, access) = self.register_attr(name, node.register_base())?;
        let ty = self.float_ty_from(name, len)?;

        let name = format_ident!("{}", name);
        Ok(quote! {
            #[register(len = #len, access = #access, ty = #ty, offset = #addr)]
            #name
        })
    }

    fn expand_string_reg(&self, node: &genapi_parser::StringRegNode) -> Result<TokenStream> {
        let node_base = node.node_base();
        let name = node_base.name();
        let (addr, len, access) = self.register_attr(name, node.register_base())?;

        let name = format_ident!("{}", name);
        Ok(quote! {
            #[register(len = #len, access = #access, ty = Bytes, offset = #addr)]
            #name
        })
    }

    fn register_attr(
        &self,
        name: &str,
        reg_base: &genapi_parser::RegisterBase,
    ) -> Result<(i64, i64, syn::Ident)> {
        use genapi_parser::AccessMode;
        use register_node_elem::AddressKind;

        let len = *reg_base
            .length()
            .imm()
            .ok_or_else(|| self.xml_err(name, "length must be immediate"))?;

        if len < 0 {
            return Err(self.xml_err(name, "length must be positive"));
        }

        let address_kinds = reg_base.address_kinds();
        if address_kinds.len() != 1 {
            return Err(self.xml_err(name, "address must be specified just once"));
        }

        let addr = match &address_kinds[0] {
            AddressKind::Address(addr) => *addr
                .imm()
                .ok_or_else(|| self.xml_err(name, "address must be immediate"))?,
            _ => {
                return Err(self.xml_err(name, "address must be immediate"));
            }
        };

        if addr < 0 {
            return Err(self.xml_err(name, "address must be positive"));
        }

        let access = match reg_base.access_mode() {
            AccessMode::RO => format_ident!("RO"),
            AccessMode::WO => format_ident!("WO"),
            AccessMode::RW => format_ident!("RW"),
        };

        Ok((addr, len, access))
    }

    fn int_ty_from(
        &self,
        name: &str,
        len: i64,
        sign: register_node_elem::Sign,
    ) -> Result<syn::Ident> {
        use register_node_elem::Sign::*;
        match (len, sign) {
            (1, Unsigned) => Ok(format_ident!("u8")),
            (2, Unsigned) => Ok(format_ident!("u16")),
            (4, Unsigned) => Ok(format_ident!("u32")),
            (8, Unsigned) => Ok(format_ident!("u64")),
            (1, Signed) => Ok(format_ident!("i8")),
            (2, Signed) => Ok(format_ident!("i16")),
            (4, Signed) => Ok(format_ident!("i32")),
            (8, Signed) => Ok(format_ident!("i64")),
            _ => Err(self.xml_err(
                name,
                "invalid integer type register length, expected 1, 2, 4, or 8",
            )),
        }
    }

    fn float_ty_from(&self, name: &str, len: i64) -> Result<syn::Ident> {
        match len {
            4 => Ok(format_ident!("f32")),
            8 => Ok(format_ident!("f64")),
            _ => Err(self.xml_err(name, "invalid float type register length, expected 4 or 8")),
        }
    }

    fn xml_err(&self, name: &str, msg: &str) -> Error {
        Error::new_spanned(&self.xml_str, format! {"{}: {}", name, msg})
    }
}

impl syn::parse::Parse for XML {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let vis = input.parse()?;
        input.parse::<syn::Token![enum]>()?;
        let ident = input.parse()?;

        let body;
        syn::braced!(body in input);

        let xml_tag = body.parse()?;
        body.parse::<syn::Token![=]>()?;
        let xml_str: syn::LitStr = body.parse()?;
        body.parse::<syn::Token![,]>()?;

        let xml_str_val = xml_str.value();
        let parser = genapi_parser::Parser::from_bytes(&xml_str_val)
            .map_err(|e| Error::new_spanned(&xml_str, e))?;
        let reg_desc = parser
            .parse()
            .map_err(|e| Error::new_spanned(&xml_str, e))?;

        Ok(Self {
            ident,
            vis,
            xml_tag,
            xml_str,
            reg_desc,
        })
    }
}

struct Args {
    xml_base: syn::LitInt,
    endianness: syn::Ident,
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let err_msg = "expected `#[register_map(xml_base = .., endianness = ..)]`";
        let ident = input.parse::<syn::Ident>()?;
        if ident != "xml_base" {
            return Err(Error::new_spanned(ident, err_msg));
        }
        input.parse::<syn::Token![=]>()?;
        let xml_base = input.parse()?;

        input.parse::<syn::Token![,]>()?;
        let ident = input.parse::<syn::Ident>()?;
        if ident != "endianness" {
            return Err(Error::new_spanned(ident, err_msg));
        }
        input.parse::<syn::Token![=]>()?;
        let endianness = input.parse()?;

        Ok(Self {
            xml_base,
            endianness,
        })
    }
}
