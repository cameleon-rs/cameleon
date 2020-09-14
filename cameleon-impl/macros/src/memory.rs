use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Result};

pub(super) fn expand(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream> {
    let memory_struct = MemoryStruct::parse(args, input)?;

    let expanded_struct = memory_struct.define_struct();
    let methods = memory_struct.impl_methods();
    let memory_trait = memory_struct.impl_memory_trait();
    let into_raw_entry_for_fragments = memory_struct.impl_into_raw_entry_for_fragments();

    Ok(proc_macro::TokenStream::from(quote! {
        #expanded_struct
        #memory_trait
        #methods
        #into_raw_entry_for_fragments
    }))
}

struct MemoryStruct {
    vis: syn::Visibility,
    ident: syn::Ident,
    attrs: Vec<syn::Attribute>,
    fragments: Vec<MemoryFragment>,
}

impl MemoryStruct {
    fn parse(_args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> Result<Self> {
        let input_struct: syn::ItemStruct = syn::parse(input)?;
        let span = input_struct.span();

        let ident = input_struct.ident;
        let attrs = input_struct.attrs;
        let vis = input_struct.vis;

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

        if fragments.is_empty() {
            return Err(Error::new(span, "at least one field required"));
        }

        Ok(Self {
            vis,
            ident,
            attrs,
            fragments,
        })
    }

    fn define_struct(&self) -> TokenStream {
        let last_fragment = self.fragments.last().unwrap();
        let last_fragment_size = last_fragment.size();
        let last_fragment_offset = last_fragment.offset();
        let memory_size = quote! {#last_fragment_size + #last_fragment_offset};

        let ident = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        quote! {
            #(#attrs)*
            #vis struct #ident {
                raw: [u8; #memory_size],
                protection: cameleon_impl::MemoryProtection,
                observers: std::vec::Vec<(cameleon_impl::RawEntry, std::boxed::Box<dyn cameleon_impl::MemoryObserver>)>,
            }
        }
    }

    fn impl_methods(&self) -> TokenStream {
        let ident = &self.ident;
        let new = self.impl_new();

        quote! {
            impl #ident {
                #new
                fn notify_all(&self, written_range: std::ops::Range<usize>) {

                    for (raw_entry, observer) in &self.observers {

                        let entry_range = raw_entry.range();
                        if written_range.start >= entry_range.end || written_range.end <= entry_range.end {
                            continue;
                        }
                        observer.update(&self.raw[entry_range]);
                    }
                }
            }
        }
    }

    fn impl_memory_trait(&self) -> TokenStream {
        let ident = &self.ident;

        quote! {
            impl cameleon_impl::prelude::MemoryRead for #ident {
                fn read(&self, range: std::ops::Range<usize>) -> cameleon_impl::MemoryResult<&[u8]> {
                    self.protection.verify_address_with_range(range.clone())?;
                    let access_right = self.protection.access_right_with_range(range.clone());
                    if !access_right.is_readable() {
                        return Err(cameleon_impl::MemoryError::AddressNotReadable);
                    }

                    Ok(&self.raw[range])
                }

                fn read_entry(&self, entry: impl std::convert::Into<cameleon_impl::RawEntry>) -> cameleon_impl::MemoryResult<&[u8]> {
                    let entry: cameleon_impl::RawEntry = entry.into();
                    self.read(entry.range())
                }

                fn access_right(&self, entry: impl std::convert::Into<cameleon_impl::RawEntry>) -> cameleon_impl::AccessRight {
                    let entry: cameleon_impl::RawEntry = entry.into();
                    self.protection.access_right_with_range(entry.range())
                }
            }

            impl cameleon_impl::prelude::MemoryWrite for #ident {
                fn write(&mut self, addr: usize, buf: &[u8]) -> cameleon_impl::MemoryResult<()> {
                    let (start, end) = (addr, addr + buf.len());
                    let range = start..end;
                    self.protection.verify_address_with_range(range.clone())?;
                    let access_right = self.protection.access_right_with_range(range.clone());
                    if !access_right.is_writable() {
                        return Err(cameleon_impl::MemoryError::AddressNotWritable);
                    }

                    self.raw[range].copy_from_slice(buf);

                    self.notify_all(start..end);

                    Ok(())
                }

                fn set_access_right(&mut self, entry: impl std::convert::Into<cameleon_impl::RawEntry>, access_right: cameleon_impl::AccessRight) {
                    let entry: cameleon_impl::RawEntry = entry.into();
                    self.protection.set_access_right_with_range(entry.range(), access_right);
                }


                fn write_entry(&mut self, entry: impl std::convert::Into<cameleon_impl::RawEntry>, buf: &[u8]) -> cameleon_impl::MemoryResult<()> {
                    let entry: cameleon_impl::RawEntry = entry.into();
                    if entry.len < buf.len() {
                        return Err(cameleon_impl::MemoryError::EntryOverrun);
                    }

                    self.write(entry.offset, buf)
                }

                fn register_observer(&mut self, observer: impl cameleon_impl::MemoryObserver + 'static + std::clone::Clone, target: impl Into<cameleon_impl::RawEntry>)
                {
                    let target: cameleon_impl::RawEntry = target.into();

                    self.observers.push((target, Box::new(observer)));
                }

            }
        }
    }

    fn impl_new(&self) -> TokenStream {
        let vis = &self.vis;
        let last_fragment = self.fragments.last().unwrap();
        let last_fragment_size = last_fragment.size();
        let last_fragment_offset = last_fragment.offset();
        let memory_size = quote! {#last_fragment_size + #last_fragment_offset};

        let init_memory = self.fragments.iter().map(|f| {
            let offset = f.offset();
            let size = f.size();
            let ty = &f.ty;
            quote! {
                let fragment = <#ty as cameleon_impl::MemoryFragment>::fragment();
                let fragment_protection = <#ty as cameleon_impl::MemoryFragment>::memory_protection();
                raw[#offset..#offset+#size].copy_from_slice(&fragment);
                protection.copy_from(&fragment_protection, #offset);
            }
        });

        quote! {
            #vis fn new() -> Self {
                let mut raw = [0; #memory_size];
                let mut protection = cameleon_impl::MemoryProtection::new(#memory_size);
                #(#init_memory)*
                Self {
                    raw,
                    protection,
                    observers: std::vec::Vec::new(),
                }
            }
        }
    }

    fn impl_into_raw_entry_for_fragments(&self) -> TokenStream {
        let impls = self.fragments.iter().map(|f| f.impl_into_raw_entry());
        quote! {
            #(#impls)*
        }
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

    fn offset(&self) -> TokenStream {
        let offset = &self.offset;
        quote! {#offset as usize}
    }

    fn size(&self) -> TokenStream {
        let ty = &self.ty;
        quote! {
            <#ty as cameleon_impl::MemoryFragment>::SIZE
        }
    }

    fn impl_into_raw_entry(&self) -> TokenStream {
        let ty = &self.ty;
        let offset = self.offset();
        quote! {
            impl std::convert::Into<cameleon_impl::RawEntry> for #ty {
                fn into(self) -> cameleon_impl::RawEntry {
                    let local_raw_entry = cameleon_impl::MemoryFragment::local_raw_entry(&self);
                    cameleon_impl::RawEntry::new(local_raw_entry.offset + #offset, local_raw_entry.len)
                }
            }
        }
    }
}

enum FragmentOffset {
    Lit(syn::LitInt),
    Var(syn::Path),
}

impl quote::ToTokens for FragmentOffset {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            FragmentOffset::Lit(ref lit) => lit.to_tokens(tokens),
            FragmentOffset::Var(ref var) => var.to_tokens(tokens),
        }
    }
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
