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
