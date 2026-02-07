mod parser;

use proc_macro2::TokenStream;
use quote::quote;
use std::u32;

use crate::extractor::parser::ast;

pub fn gen_extractor(expr: String) -> Result<TokenStream, TokenStream> {
    let patterns_ast = parser::parse(&expr)?;

    let hlist_t = quote! { ::http_path_core::hlist };
    let literal_t = quote! { ::http_path_core::matcher::patterns::literal };
    let variable_t = quote! { ::http_path_core::matcher::patterns::variable };
    let patterns = patterns_ast.iter().map(|p| match p {
        ast::Pattern::Literal {
            lit,
            as_type: None | Some(ast::AsType::Str),
        } => {
            quote! { #literal_t(#lit) }
        }
        ast::Pattern::Literal {
            lit,
            as_type: Some(ast::AsType::U32),
        } => {
            let lit: u32 = lit
                .parse()
                .expect(&format!("{lit} must be parsable as a u32"));
            quote! { #literal_t(#lit)}
        }
        ast::Pattern::Variable {
            as_type: ast::AsType::Str,
        } => {
            quote! { #variable_t::<&str>() }
        }
        ast::Pattern::Variable {
            as_type: ast::AsType::U32,
        } => {
            quote! { #variable_t::<u32>() }
        }
    });

    Ok(quote::quote! {
         #hlist_t! [
             #(#patterns), *
         ]
    }
    .into())
}

impl From<parser::ExtractorParserError> for TokenStream {
    fn from(error: parser::ExtractorParserError) -> Self {
        let message = error.to_string();
        quote! {
            compile_error!(#message)
        }
        .into()
    }
}

#[cfg(test)]
mod tests {}
