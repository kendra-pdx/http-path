#![cfg_attr(not(test), no_std)]

use http_path_core::*;

pub use http_path_core::hlist;
pub use http_path_core::hlist_pat;
pub use http_path_macros::extractor;

pub mod prelude {
    pub use crate::matcher::patterns;
    pub use crate::matcher::*;
    pub use crate::path::*;
    pub use crate::*;
}
