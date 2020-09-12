use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let register_enum = RegisterEnum::parse(args, input)?;

    let expanded_enum = register_enum.expand_enum();

    Ok(proc_macro::TokenStream::from(quote! {
            #expanded_enum
    }))
}

struct RegisterEnum {
    ident: syn::Ident,
    vis: syn::Visibility,
    endianess: Endianess,
    entries: Vec<RegisterEntry>,
    //TODO:docs: Option<Vec<syn::Attribute>>,
}

impl RegisterEnum {
    fn parse(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> Result<Self> {
        let input_enum: syn::ItemEnum = syn::parse(input)?;

        let ident = input_enum.ident;
        let vis = input_enum.vis;

        let endianess: Endianess = syn::parse(args)?;

        let mut offset = 0;
        let mut entries = vec![];
        for variant in input_enum.variants.into_iter() {
            let ent = RegisterEntry::parse(variant, &mut offset)?;
            entries.push(ent);
        }

        Ok(Self {
            ident,
            vis,
            endianess,
            entries,
        })
    }

    fn expand_enum(&self) -> TokenStream {
        let variants = self.entries.iter().map(|entry| {
            let variant = &entry.ident;
            quote! {#variant}
        });
        let enum_name = &self.ident;
        let vis = &self.vis;

        quote! {
            #vis enum #enum_name {
                #(#variants),*
            }
        }
    }
}

struct RegisterEntry {
    ident: syn::Ident,
    offset: usize,
    entry_attr: EntryAttr,
    init: Option<InitValue>,
    //TODO:docs: Option<Vec<syn::Attribute>>,
}

impl RegisterEntry {
    fn parse(variant: syn::Variant, offset: &mut usize) -> Result<Self> {
        let entry_attr = Self::parse_entry_attr(&variant)?;
        let ident = variant.ident;
        let entry_offset = *offset;

        *offset += entry_attr.len;

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
        })
    }

    fn parse_entry_attr(variant: &syn::Variant) -> Result<EntryAttr> {
        let mut entry_attr = None;

        for attr in variant.attrs.clone() {
            match attr.path.get_ident() {
                Some(ident) if ident == "entry" => {
                    if entry_attr.is_none() {
                        let attr: EntryAttr = syn::parse(attr.tokens.into())?;
                        entry_attr = Some(attr);
                    } else {
                        return Err(Error::new_spanned(attr, "duplicated entry attribute"));
                    }
                }
                _ => continue,
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
    len: usize,
    access: AccessRight,
    ty: Option<EntryType>,
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
        let len = ts.parse::<syn::LitInt>()?.base10_parse()?;
        ts.parse::<syn::token::Comma>()?;

        match ts.parse::<syn::Ident>()? {
            access_right if access_right == "access_right" => {}
            other => return Err(Error::new_spanned(other, "expected access_right")),
        };
        ts.parse::<syn::Token![=]>()?;
        let access = AccessRight::from_ident(ts.parse::<syn::Ident>()?)?;

        let ty = if let Ok(_) = ts.parse::<syn::token::Comma>() {
            match ts.parse::<syn::Ident>()? {
                ty if ty == "type" => {}
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

enum EntryType {
    Str,
    U8,
    U16,
    U32,
    U64,
}

impl EntryType {
    fn from_ident(ident: syn::Ident) -> Result<Self> {
        if ident == "String" || ident == "&str" {
            Ok(EntryType::Str)
        } else if ident == "u8" {
            Ok(EntryType::U8)
        } else if ident == "u16" {
            Ok(EntryType::U16)
        } else if ident == "u32" {
            Ok(EntryType::U32)
        } else if ident == "u64" {
            Ok(EntryType::U64)
        } else {
            Err(Error::new_spanned(
                ident,
                "expected String, &str, u8, u16, u32 or u64",
            ))
        }
    }
}

enum Endianess {
    BE,
    LE,
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
                ident,
                "only BE or LE is allowed for endianess specifier",
            ))
        }
    }
}
