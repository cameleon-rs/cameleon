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

    Ok(proc_macro::TokenStream::from(quote! {
        #expanded_struct
        #memory_trait
        #methods
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
        let last_fragment_base = last_fragment.base();
        let memory_size = quote! {#last_fragment_size + #last_fragment_base};

        let ident = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        quote! {
            #(#attrs)*
            #vis struct #ident {
                raw: [u8; #memory_size],
                protection: cameleon_impl::memory::MemoryProtection,
                observers: std::vec::Vec<(cameleon_impl::memory::RawEntry, std::boxed::Box<dyn cameleon_impl::memory::MemoryObserver>)>,
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
            impl cameleon_impl::memory::prelude::MemoryRead for #ident {
                fn read(&self, range: std::ops::Range<usize>) -> cameleon_impl::memory::MemoryResult<&[u8]> {
                    self.protection.verify_address_with_range(range.clone())?;
                    let access_right = self.protection.access_right_with_range(range.clone());
                    if !access_right.is_readable() {
                        return Err(cameleon_impl::memory::MemoryError::AddressNotReadable);
                    }

                    Ok(&self.raw[range])
                }

                fn read_entry<T: cameleon_impl::memory::RegisterEntry>(&self) -> cameleon_impl::memory::MemoryResult<T::Ty> {
                    let range = T::raw_entry().range();
                    debug_assert!(self.protection.verify_address_with_range(range.clone()).is_ok());

                    T::parse(&self.raw[range])
                }

                fn access_right<T: cameleon_impl::memory::RegisterEntry>(&self) -> cameleon_impl::memory::AccessRight {
                    self.protection.access_right_with_range(T::raw_entry().range())
                }
            }

            impl cameleon_impl::memory::prelude::MemoryWrite for #ident {
                fn write(&mut self, addr: usize, buf: &[u8]) -> cameleon_impl::memory::MemoryResult<()> {
                    let (start, end) = (addr, addr + buf.len());
                    let range = start..end;
                    self.protection.verify_address_with_range(range.clone())?;
                    let access_right = self.protection.access_right_with_range(range.clone());
                    if !access_right.is_writable() {
                        return Err(cameleon_impl::memory::MemoryError::AddressNotWritable);
                    }

                    self.raw[range].copy_from_slice(buf);

                    self.notify_all(start..end);

                    Ok(())
                }

                fn set_access_right<T: cameleon_impl::memory::RegisterEntry>(&mut self, access_right: cameleon_impl::memory::AccessRight) {
                    self.protection.set_access_right_with_range(T::raw_entry().range(), access_right);
                }


                fn write_entry<T: cameleon_impl::memory::RegisterEntry>(&mut self, data: T::Ty) -> cameleon_impl::memory::MemoryResult<()>{
                    let entry = T::raw_entry();
                    let data = T::serialize(data)?;
                    let range = entry.range();

                    debug_assert!(entry.len == data.len());
                    debug_assert!(self.protection.verify_address_with_range(range.clone()).is_ok());

                    self.raw[range.clone()].copy_from_slice(data.as_slice());

                    self.notify_all(range);

                    Ok(())
                }

                fn register_observer<T, U>(
                    &mut self,
                    observer: U
                )
                    where T: cameleon_impl::memory::RegisterEntry,
                          U: cameleon_impl::memory::MemoryObserver + 'static
                {
                    let entry = T::raw_entry();

                    self.observers.push((entry, Box::new(observer)));
                }

            }
        }
    }

    fn impl_new(&self) -> TokenStream {
        let vis = &self.vis;
        let last_fragment = self.fragments.last().unwrap();
        let last_fragment_size = last_fragment.size();
        let last_fragment_offset = last_fragment.base();
        let memory_size = quote! {#last_fragment_size + #last_fragment_offset};

        let init_memory = self.fragments.iter().map(|f| {
            let base = f.base();
            let size = f.size();
            let ty = &f.ty;
            quote! {
                let fragment = #ty::raw();
                let fragment_protection = #ty::memory_protection();
                raw[#base..#base+#size].copy_from_slice(&fragment);
                protection.copy_from(&fragment_protection, #base);
            }
        });

        quote! {
            #vis fn new() -> Self {
                let mut raw = [0; #memory_size];
                let mut protection = cameleon_impl::memory::MemoryProtection::new(#memory_size);
                #(#init_memory)*
                Self {
                    raw,
                    protection,
                    observers: std::vec::Vec::new(),
                }
            }
        }
    }
}

struct MemoryFragment {
    ty: syn::Path,
}

impl MemoryFragment {
    fn parse(field: syn::Field) -> Result<Self> {
        let ty = match field.ty {
            syn::Type::Path(p) => p.path,
            other => return Err(Error::new_spanned(other, "expected type path")),
        };

        Ok(Self { ty })
    }

    fn base(&self) -> TokenStream {
        let ty = &self.ty;
        quote!(#ty::BASE)
    }

    fn size(&self) -> TokenStream {
        let ty = &self.ty;
        quote!(#ty::SIZE)
    }
}
