use syn::{LitStr, parse_macro_input};

use crate::extractor::gen_extractor;

mod extractor;

///
/// Provides a simple grammar for expressing hlists of patterns to be used in matching
/// against [Paths](prelude/struct.Path.html)
///
/// Example `extractor!("a/{u32}/1:u32")`
///
/// terms from the example:
///   - `a`: literal, default &str
///   - `{u32}`: variable, u32
///   - `1`: literal, u32
///
/// ## Grammar Definition
/// ```
#[doc = include_str!("./extractor/extractor.pest")]
/// ```

#[proc_macro]
pub fn extractor(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let args = parse_macro_input!(ts as LitStr);
    match gen_extractor(args.value()) {
        Ok(ts) => ts.into(),
        Err(ts) => ts.into(),
    }
}
