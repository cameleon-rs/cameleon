macro_rules! try_gentl {
    ($expr:expr) => {{
        let res = $expr;
        match &res {
            Ok(data) => *data,
            Err(_) => {
                let code = (&res).into();
                crate::ffi::save_last_error(res);
                return code;
            }
        }
    }};
}

macro_rules! gentl_api {
    (
        pub fn $name:ident($($arg:ident: $ty:ty,)*) -> GenTlResult<()> $body: tt
    )
    => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $ty),*) -> GC_ERROR {
            try_gentl!(crate::ffi::assert_lib_initialized());
            let res: GenTlResult<()> = $body;
            let code = (&res).into();
            crate::ffi::save_last_error(res);
            code
        }
    };

    (
        no_assert pub fn $name:ident($($arg:ident: $ty:ty,)*) -> GenTlResult<()> $body: tt
    )
    => {
        #[no_mangle]
        pub extern "C" fn $name($($arg: $ty),*) -> GC_ERROR {
            let res: GenTlResult<()> = $body;
            let code = (&res).into();
            crate::ffi::save_last_error(res);
            code
        }
    };
}
