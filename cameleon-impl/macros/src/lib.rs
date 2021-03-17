#![allow(
    clippy::module_name_repetitions,
    clippy::clippy::similar_names,
    clippy::clippy::missing_errors_doc
)]

mod memory;
mod register_map;
mod util;

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
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    match memory::expand(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
