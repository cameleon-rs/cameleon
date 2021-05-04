use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Result};

pub(super) fn expand(input: proc_macro::TokenStream) -> Result<proc_macro::TokenStream> {
    let memory_struct = MemoryStruct::parse(input)?;

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
    fn parse(input: proc_macro::TokenStream) -> Result<Self> {
        let input_struct: syn::ItemStruct = syn::parse(input)?;
        let span = input_struct.span();

        let ident = input_struct.ident;
        let attrs = input_struct.attrs;
        let vis = input_struct.vis;

        let mut fragments = vec![];
        match input_struct.fields {
            syn::Fields::Named(fields) => {
                for field in fields.named {
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
        let ident = &self.ident;
        let vis = &self.vis;
        let attrs = &self.attrs;

        quote! {
            #(#attrs)*
            #vis struct #ident {
                raw: Vec<u8>,
                protection: cameleon_impl::memory::MemoryProtection,
                observers: std::vec::Vec<(std::ops::Range<usize>, std::boxed::Box<dyn cameleon_impl::memory::MemoryObserver>)>,
            }
        }
    }

    fn impl_methods(&self) -> TokenStream {
        let ident = &self.ident;
        let new = self.impl_new();
        let fragments_len = self.fragments.len();

        quote! {
            impl #ident {
                #new

                #[doc(hidden)]
                fn notify_all(&self, written_range: std::ops::Range<usize>) {

                    for (reg_range, observer) in &self.observers {

                        if written_range.start >= reg_range.end || written_range.end <= reg_range.start {
                            continue;
                        }
                        observer.update();
                    }
                }

                #[doc(hidden)]
                const fn calculate_memory_size(end_addresses: &[usize; #fragments_len]) -> usize {
                    let mut max = end_addresses[0];
                    let mut i = 0;
                    while i < #fragments_len {
                        if max < end_addresses[i] {
                            max = end_addresses[i];
                        }
                        i += 1;
                    }
                    max

                }
            }
        }
    }

    fn impl_memory_trait(&self) -> TokenStream {
        let ident = &self.ident;

        quote! {
            impl cameleon_impl::memory::prelude::MemoryRead for #ident {
                fn read_raw(&self, range: std::ops::Range<usize>) -> cameleon_impl::memory::MemoryResult<&[u8]> {
                    self.protection.verify_address_with_range(range.clone())?;
                    let access_right = self.protection.access_right_with_range(range.clone());
                    if !access_right.is_readable() {
                        return Err(cameleon_impl::memory::MemoryError::AddressNotReadable);
                    }

                    Ok(&self.raw[range])
                }

                fn read<T: cameleon_impl::memory::Register>(&self) -> cameleon_impl::memory::MemoryResult<T::Ty> {
                    T::read(&self.raw)
                }

                fn access_right<T: cameleon_impl::memory::Register>(&self) -> cameleon_impl::memory::AccessRight {
                    self.protection.access_right_with_range(T::range())
                }
            }

            impl cameleon_impl::memory::prelude::MemoryWrite for #ident {
                fn write_raw(&mut self, addr: usize, buf: &[u8]) -> cameleon_impl::memory::MemoryResult<()> {
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

                fn set_access_right<T: cameleon_impl::memory::Register>(&mut self, access_right: cameleon_impl::memory::AccessRight) {
                    self.protection.set_access_right_with_range(T::range(), access_right);
                }


                fn write<T: cameleon_impl::memory::Register>(&mut self, data: T::Ty) -> cameleon_impl::memory::MemoryResult<()>{
                    T::write(data, &mut self.raw)?;
                    self.notify_all(T::range());

                    Ok(())
                }

                fn register_observer<T, U>(
                    &mut self,
                    observer: U
                )
                    where T: cameleon_impl::memory::Register,
                          U: cameleon_impl::memory::MemoryObserver + 'static
                {
                    let reg_range = T::range();

                    self.observers.push((reg_range, Box::new(observer)));
                }

            }
        }
    }

    fn impl_new(&self) -> TokenStream {
        let vis = &self.vis;
        let memory_size = self.memory_size();

        let init_memory = self.fragments.iter().map(|f| {
            let ty = &f.ty;
            quote! {
                #ty::init_memory_protection(&mut protection);
                #ty::init_raw_memory(&mut raw);
            }
        });

        quote! {
            #vis fn new() -> Self {
                let mut raw = vec![0; #memory_size];
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

    fn memory_size(&self) -> TokenStream {
        let end_addresses = self.fragments.iter().map(|f| {
            let size = f.size();
            let base = f.base();
            quote! {#size + #base}
        });

        quote! {
            Self::calculate_memory_size(&[#(#end_addresses),*])
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
        quote!(#ty::base())
    }

    fn size(&self) -> TokenStream {
        let ty = &self.ty;
        quote!(#ty::size())
    }
}
