use quote::quote;
use syn::{spanned::Spanned, Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let memory_struct = MemoryStruct::parse(args, input)?;
    Ok(proc_macro::TokenStream::from(quote! {}))
}

struct MemoryStruct {
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    fragments: Vec<MemoryFragment>,
}

impl MemoryStruct {
    fn parse(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> Result<Self> {
        let input_struct: syn::ItemStruct = syn::parse(input)?;
        let ident = input_struct.ident;
        let attrs = input_struct.attrs;
        let mut fragments = vec![];

        match input_struct.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named.into_iter() {
                    fragments.push(MemoryFragment::parse(field)?);
                }
            }
            other => {
                return Err(Error::new_spanned(other, "expected named field"));
            }
        }

        Ok(Self {
            ident,
            attrs,
            fragments,
        })
    }
}

struct MemoryFragment {
    ty: syn::Path,
    offset: FragmentOffset,
}

impl MemoryFragment {
    fn parse(field: syn::Field) -> Result<Self> {
        let span = field.span();
        let ty = match field.ty {
            syn::Type::Path(p) => p.path,
            other => return Err(Error::new_spanned(other, "expected type path")),
        };

        let offset = FragmentOffset::parse(field.attrs, span)?;
        Ok(Self { ty, offset })
    }
}

enum FragmentOffset {
    Lit(syn::LitInt),
    Var(syn::Path),
}

impl FragmentOffset {
    fn parse(attrs: Vec<syn::Attribute>, field_span: proc_macro2::Span) -> Result<Self> {
        let mut offset = None;
        for attr in attrs.into_iter() {
            if let Some(ident) = attr.path.get_ident() {
                if ident != "offset" {
                    continue;
                }

                if offset.is_some() {
                    return Err(Error::new_spanned(attr, "duplicated offset attribute"));
                }

                let expr: syn::Expr = attr.parse_args()?;
                offset = match expr {
                    syn::Expr::Lit(expr_lit) => {
                        if let syn::Lit::Int(litint) = expr_lit.lit {
                            Some(FragmentOffset::Lit(litint))
                        } else {
                            return Err(Error::new_spanned(
                                expr_lit,
                                "argument of offset attribute must be path or litint",
                            ));
                        }
                    }
                    syn::Expr::Path(p) => Some(FragmentOffset::Var(p.path)),
                    other => {
                        return Err(Error::new_spanned(
                            other,
                            "argument of offset attribute must be path or litint",
                        ));
                    }
                };
            }
        }

        offset.ok_or_else(|| Error::new(field_span, "`#[offset(..)]` is required"))
    }
}
