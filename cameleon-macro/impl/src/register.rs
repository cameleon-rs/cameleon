use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let register_enum = RegisterEnum::parse(args, input)?;

    let expanded_enum = register_enum.define_enum();
    let impl_enum = register_enum.impl_enum();
    let impl_memory_fragment = register_enum.impl_memory_fragment();

    Ok(proc_macro::TokenStream::from(quote! {
            #expanded_enum
            #impl_enum
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
            attrs: input_enum.attrs,
        })
    }

    fn define_enum(&self) -> TokenStream {
        let variants = self.entries.iter().map(|entry| {
            let ident = &entry.ident;
            quote! {#ident}
        });
        let enum_name = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        quote! {
            #(#attrs)*
            #vis enum #enum_name {
                #(#variants),*
            }
        }
    }

    fn impl_enum(&self) -> TokenStream {
        let raw_entry_local = self.impl_into_raw_entry_local();
        let ident = &self.ident;

        quote! {
            impl #ident {
                #raw_entry_local
            }
        }
    }

    fn impl_into_raw_entry_local(&self) -> TokenStream {
        let enum_ident = &self.ident;
        let arms = self.entries.iter().map(|entry| {
            let ident = &entry.ident;
            let offset = entry.offset;
            let len = entry.entry_attr.len;
            quote! {
                 #enum_ident::#ident => cameleon_macro::RawEntry::new(#offset, #len)
            }
        });

        quote! {
            #[doc(hidden)]
            pub fn into_raw_entry_local(self) -> cameleon_macro::RawEntry {
                match self {
                    #(#arms,)*
                }
            }
        }
    }

    fn impl_memory_fragment(&self) -> TokenStream {
        let memory_protection = self.impl_memory_protection();
        //let fragment = self.impl_frament();
        let ident = &self.ident;
        let size = self.size();

        //quote! {
        //    impl cameleon_macro::MemoryFragment for #ident {
        //      const SIZE: usize = #size;
        //      #memory_protection
        //      //#fragment
        //    }
        //}
        quote! {
            impl cameleon_macro::MemoryFragment for #ident {
              const SIZE: usize = #size;
              #memory_protection
              //#fragment
            }
        }
    }

    fn impl_memory_protection(&self) -> TokenStream {
        let set_access_right = self.entries.iter().map(|entry| {
            let start = entry.offset;
            let end = start + entry.entry_attr.len;
            let access_right = entry.entry_attr.access.as_ident();
            quote! {
                memory_protection.set_access_right_with_range(#start..#end, cameleon_macro::AccessRight::#access_right)
            }});

        let size = self.size();
        quote! {
            fn memory_protection() -> MemoryProtection {
                let mut memory_protection = cameleon_macro::MemoryProtection::new(#size);
                #(#set_access_right;)*
                memory_protection
            }
        }
    }

    fn impl_frament(&self) -> TokenStream {
        todo!();
    }

    fn size(&self) -> usize {
        let last_field = self.entries.last().unwrap();
        last_field.offset + last_field.entry_attr.len
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
            attrs: variant.attrs,
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

    fn as_ident(&self) -> syn::Ident {
        use AccessRight::*;
        match self {
            NA => quote::format_ident!("NA"),
            RO => quote::format_ident!("RO"),
            WO => quote::format_ident!("WO"),
            RW => quote::format_ident!("RW"),
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
                endianess,
                "only BE or LE is allowed for endianess specifier",
            ))
        }
    }
}
