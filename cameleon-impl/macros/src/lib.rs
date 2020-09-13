mod memory;
mod register;

#[proc_macro_attribute]
pub fn register(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match register::expand(args, input) {
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
