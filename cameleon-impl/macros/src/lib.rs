mod genapi;
mod memory;
mod register_map;

#[proc_macro_attribute]
pub fn register_map(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match register_map::expand(args, input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn memory(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match memory::expand(args, input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn genapi(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match genapi::expand(args, input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
