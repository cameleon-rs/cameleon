/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

macro_rules! gentl_api {
    (
        pub fn $name:ident($($arg:ident: $ty:ty),*$(,)?) -> GenTlResult<()> $body:tt
    )
    => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $ty),*) -> GC_ERROR {
            #[inline(always)]
            fn inner($($arg: $ty),*) -> GenTlResult<()> {
                crate::ffi::assert_lib_initialized()?;
                $body
            }

            let res = inner($($arg),*);
            let code = (&res).into();
            crate::ffi::save_last_error(res);
            code
        }
    };

    (
        no_assert pub fn $name:ident($($arg:ident: $ty:ty),*$(,)?) -> GenTlResult<()> $body:tt
    )
    => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $ty),*) -> GC_ERROR {
            #[inline(always)]
            fn inner($($arg: $ty),*) -> GenTlResult<()> {
                $body
            }

            let res = inner($($arg),*);
            let code = (&res).into();
            crate::ffi::save_last_error(res);
            code
        }
    };
}

/// See https://github.com/rust-lang/rust/issues/36927 to know why this macro is needed.
/// Fortunately, GenTL specification specifies that enumeration values are signed 32 bit integers
/// (6.4 Enumerations).
macro_rules! newtype_enum {
    (
    pub enum $name:ident {
        $(
            $(#[$meta:meta])*
            $variant:ident = $value:literal,
        )*
    }
    ) => {

        #[derive(PartialEq, Eq, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name(i32);

        impl $name {
            $(
                $(#[$meta])*
                pub const $variant: $name = $name($value);
            )*
        }
    };
}
