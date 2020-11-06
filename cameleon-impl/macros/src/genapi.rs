use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Error, Result};

use cameleon_impl_genapi_parser as genapi_parser;
use genapi_parser::register_node_elem;

use super::util::modify_visibility;

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
        let vis = &self.xml.vis;
        let ident = &self.xml.ident;
        let register = self.expand_register()?;
        let genapi_util = self.expand_genapi_util()?;
        let non_registers = self.xml.epxand_non_register()?;

        Ok(quote! {
            #[allow(non_snake_case)]
            #[allow(clippy::string_lit_as_bytes)]
            #vis mod #ident {
                #register
                #genapi_util
                #(#non_registers)*
            }
        })
    }

    fn expand_register(&self) -> Result<TokenStream> {
        let ident = &self.xml.ident;
        let vis = &self.xml.vis;
        let endianness = &self.args.endianness;
        let regs = self.xml.expand_registers()?;

        Ok(quote! {
            #[cameleon_impl::memory::register_map(base = 0, endianness = #endianness, genapi)]
            #vis enum #ident {
                #(#regs,)*
            }
        })
    }

    fn expand_genapi_util(&self) -> Result<TokenStream> {
        let vis_inside_mod = modify_visibility(&self.xml.vis)?;
        let xml_length = self.xml.xml_str.value().as_bytes().len();
        let xml_address = self.xml.xml_base_address()?;

        let schema_major = self.xml.reg_desc.schema_major_version();
        let schema_minor = self.xml.reg_desc.schema_minor_version();
        let schema_subminor = self.xml.reg_desc.schema_subminor_version();

        let genapi_major = self.xml.reg_desc.major_version();
        let genapi_minor = self.xml.reg_desc.minor_version();
        let genapi_subminor = self.xml.reg_desc.subminor_version();

        let vendor_name = self.xml.reg_desc.vendor_name();

        Ok(quote! {
            #vis_inside_mod const fn xml_address() -> u64 {
                #xml_address as u64
            }

            #vis_inside_mod const fn xml_length() -> usize {
                #xml_length as usize
            }

            #vis_inside_mod const fn vendor_name() -> &'static str {
                &#vendor_name
            }

            #vis_inside_mod fn schema_version() -> cameleon_impl::semver::Version {
                cameleon_impl::semver::Version::new(#schema_major, #schema_minor, #schema_subminor)
            }

            #vis_inside_mod fn genapi_version() -> cameleon_impl::semver::Version {
                cameleon_impl::semver::Version::new(#genapi_major, #genapi_minor, #genapi_subminor)
            }
        })
    }
}

struct XML {
    ident: syn::Ident,
    vis: syn::Visibility,
    xml_tag: syn::Ident,
    xml_str: syn::LitStr,
    reg_desc: genapi_parser::RegisterDescription,
}

impl XML {
    fn expand_registers(&self) -> Result<Vec<TokenStream>> {
        let mut registers = self.expand_reg_desc()?;
        registers.push(self.expand_xml()?);

        Ok(registers)
    }

    fn expand_reg_desc(&self) -> Result<Vec<TokenStream>> {
        use genapi_parser::NodeKind::*;

        let mut regs = vec![];

        for node in self.reg_desc.nodes() {
            match node {
                IntReg(node) => regs.push(self.expand_int_reg(node)?),
                MaskedIntReg(node) => regs.push(self.expand_masked_int_reg(node)?),
                FloatReg(node) => regs.push(self.expand_float_reg(node)?),
                StringReg(node) => regs.push(self.expand_string_reg(node)?),
                Register(node) => regs.push(self.expand_reg(node)?),
                StructReg(node) => {
                    let masked_int_regs: Vec<_> = node.to_masked_int_regs();
                    for node in &masked_int_regs {
                        regs.push(self.expand_masked_int_reg(node)?);
                    }
                }
                _ => {}
            };
        }

        Ok(regs)
    }

    fn epxand_non_register(&self) -> Result<Vec<TokenStream>> {
        use genapi_parser::{numeric_node_elem::ValueKind, ImmOrPNode, NodeKind::*};
        let mut non_registers = vec![];
        let vis_inside_mod = modify_visibility(&self.vis)?;

        macro_rules! expand_imm {
            ($node: ident, $ty:ty, value_kind) => {{
                expand_imm!(
                    $node,
                    $ty,
                    match $node.value_kind() {
                        ValueKind::Value(v) => v,
                        _ => continue,
                    }
                );
            }};

            ($node: ident, $ty:ty) => {{
                expand_imm!(
                    $node,
                    $ty,
                    match $node.value() {
                        ImmOrPNode::Imm(v) => v,
                        _ => continue,
                    }
                );
            }};

            ($node: ident, $ty:ty, $value:expr) => {{
                let name = format_ident!("{}", $node.node_base().name());
                let value = $value;
                non_registers.push(quote! {
                    #vis_inside_mod const #name: $ty = #value;
                });
            }};
        }

        for node in self.reg_desc.nodes() {
            match node {
                Integer(node) => expand_imm!(node, i64, value_kind),
                Float(node) => expand_imm!(node, f64, value_kind),
                Boolean(node) => expand_imm!(node, bool),
                String(node) => expand_imm!(node, &'static str),
                Port(node) => expand_imm!(node, &'static str, node.node_base().name()),
                Enumeration(node) => non_registers.push(self.expand_enumeration_node(node)?),
                _ => {}
            }
        }

        Ok(non_registers)
    }

    fn expand_enumeration_node(
        &self,
        node: &Box<genapi_parser::EnumerationNode>,
    ) -> Result<TokenStream> {
        let name = format_ident!("{}", node.node_base().name());
        let variants = node.entries().iter().map(|ent| {
            let var = format_ident!("{}", ent.node_base().name());
            let value = ent.value() as isize;
            quote!(#var = #value)
        });

        let vis_inside_mod = modify_visibility(&self.vis)?;

        Ok(quote! {
            #vis_inside_mod enum #name {
                #(#variants,)*
            }
        })
    }

    fn expand_xml(&self) -> Result<TokenStream> {
        let xml_tag = &self.xml_tag;
        let xml_str = &self.xml_str;
        let xml_str_len = self.xml_str.value().as_bytes().len();
        let xml_base = self.xml_base_address()?;

        Ok(quote! {
            #[register(len = #xml_str_len, access = RO, ty = Bytes, offset = #xml_base)]
             #xml_tag = #xml_str.as_bytes().into()
        })
    }

    fn expand_int_reg(&self, node: &genapi_parser::IntRegNode) -> Result<TokenStream> {
        let name = node.node_base().name();
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

        let name = node.node_base().name();
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
        let name = node.node_base().name();
        let (addr, len, access) = self.register_attr(name, node.register_base())?;
        let ty = self.float_ty_from(name, len)?;

        let name = format_ident!("{}", name);
        Ok(quote! {
            #[register(len = #len, access = #access, ty = #ty, offset = #addr)]
            #name
        })
    }

    fn expand_string_reg(&self, node: &genapi_parser::StringRegNode) -> Result<TokenStream> {
        let name = node.node_base().name();
        let (addr, len, access) = self.register_attr(name, node.register_base())?;

        let name = format_ident!("{}", name);
        Ok(quote! {
            #[register(len = #len, access = #access, ty = String, offset = #addr)]
            #name
        })
    }

    fn expand_reg(&self, node: &genapi_parser::RegisterNode) -> Result<TokenStream> {
        let name = node.node_base().name();
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

    fn xml_base_address(&self) -> Result<i64> {
        use genapi_parser::NodeKind::*;

        let mut xml_base = 0;
        macro_rules! maximum_xml_base {
            ($xml_base: ident, $node: ident) => {{
                let node_base = $node.node_base();
                let name = node_base.name();
                let (addr, len, _) = self.register_attr(name, $node.register_base())?;
                if addr + len > $xml_base {
                    $xml_base = addr + len;
                }
            }};
        }

        for node in self.reg_desc.nodes() {
            match node {
                IntReg(node) => maximum_xml_base!(xml_base, node),
                MaskedIntReg(node) => maximum_xml_base!(xml_base, node),
                FloatReg(node) => maximum_xml_base!(xml_base, node),
                StringReg(node) => maximum_xml_base!(xml_base, node),
                Register(node) => maximum_xml_base!(xml_base, node),
                StructReg(node) => {
                    let masked_int_regs: Vec<_> = node.to_masked_int_regs();
                    for node in &masked_int_regs {
                        maximum_xml_base!(xml_base, node)
                    }
                }
                _ => {}
            };
        }

        Ok(xml_base)
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
    endianness: syn::Ident,
}

impl syn::parse::Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let err_msg = "expected `#[register_map(endianness = ..)]`";

        let ident = input.parse::<syn::Ident>()?;
        if ident != "endianness" {
            return Err(Error::new_spanned(ident, err_msg));
        }
        input.parse::<syn::Token![=]>()?;
        let endianness = input.parse()?;

        Ok(Self { endianness })
    }
}
