#![allow(unused)]

use pest::{Parser, iterators::*};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{self:?}")]
pub enum ExtractorParserError {
    NotImplemented,
    RuleError(#[from] pest::error::Error<Rule>),
    UnexpectedSyntaxError { expected: Rule, actual: Rule },
    SyntaxError(String),
}

#[derive(Parser)]
#[grammar = "extractor/extractor.pest"]
pub struct ExtractorParser;

pub mod ast {
    use derive_more::Deref;

    #[derive(Debug, strum::EnumString)]
    #[strum(ascii_case_insensitive)]
    pub enum AsType {
        Str,
        U32,
    }

    #[derive(Debug)]
    pub enum Pattern<'a> {
        Literal {
            lit: &'a str,
            as_type: Option<AsType>,
        },

        Variable {
            as_type: AsType,
        },
    }

    #[derive(Debug, Deref)]
    pub struct Patterns<'a>(pub(super) Vec<Pattern<'a>>);
}

pub fn parse<'i>(input: &'i str) -> Result<ast::Patterns<'i>, ExtractorParserError> {
    let rule = Rule::patterns;
    let mut pairs = ExtractorParser::parse(rule, input)?;

    if let Some(pair) = pairs.next() {
        let patterns = pair.try_into()?;
        Ok(patterns)
    } else {
        Err(ExtractorParserError::NotImplemented)
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for ast::AsType {
    type Error = ExtractorParserError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let rule = pair.as_rule();
        match rule {
            Rule::as_type => {
                let as_type_str = pair.as_str();
                let as_type = ast::AsType::try_from(as_type_str).map_err(|e| {
                    ExtractorParserError::SyntaxError(format!(
                        "valid type values are [str, u32], got {}: {}",
                        as_type_str, e
                    ))
                })?;
                Ok(as_type)
            }
            _ => Err(ExtractorParserError::UnexpectedSyntaxError {
                expected: Rule::as_type,
                actual: rule,
            }),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for ast::Pattern<'a> {
    type Error = ExtractorParserError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let rule = pair.as_rule();
        match rule {
            Rule::literal => {
                let as_type = pair.clone().into_inner().find_map(|p| {
                    if p.as_rule() == Rule::as_type {
                        ast::AsType::try_from(p).ok()
                    } else {
                        None
                    }
                });

                let lit = pair
                    .clone()
                    .into_inner()
                    .find_map(|p| {
                        if p.as_rule() == Rule::ident {
                            Some(p.as_str())
                        } else {
                            None
                        }
                    })
                    .ok_or(ExtractorParserError::SyntaxError(format!(
                        "literal without ident contents: {:?}",
                        pair
                    )))?;

                Ok(ast::Pattern::Literal { lit, as_type })
            }

            Rule::variable => {
                let as_type = pair
                    .clone()
                    .into_inner()
                    .find_map(|p| {
                        if p.as_rule() == Rule::as_type {
                            ast::AsType::try_from(p).ok()
                        } else {
                            None
                        }
                    })
                    .ok_or(ExtractorParserError::SyntaxError(format!(
                        "variable without type: {:?}",
                        pair
                    )))?;
                Ok(ast::Pattern::Variable { as_type })
            }

            _ => Err(ExtractorParserError::UnexpectedSyntaxError {
                expected: Rule::pattern,
                actual: rule,
            }),
        }
    }
}

impl<'a> TryFrom<Pair<'a, Rule>> for ast::Patterns<'a> {
    type Error = ExtractorParserError;

    fn try_from(pair: Pair<'a, Rule>) -> Result<Self, Self::Error> {
        let rule = pair.as_rule();
        match rule {
            Rule::patterns => {
                let mut patterns = Vec::new();
                for inner in pair.into_inner() {
                    let pattern = inner.try_into()?;
                    patterns.push(pattern);
                }
                Ok(ast::Patterns(patterns))
            }

            _ => Err(ExtractorParserError::UnexpectedSyntaxError {
                expected: Rule::patterns,
                actual: rule,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser;

    use crate::extractor::parser::ast::AsType;

    use super::*;

    #[test]
    fn basic_parsing() {
        let result = parse("a/{u32}/c/1:u32");
        println!("{result:?}");

        if !matches!(
            result.unwrap().as_array(),
            Some([
                ast::Pattern::Literal {
                    lit: "a",
                    as_type: None,
                },
                ast::Pattern::Variable {
                    as_type: AsType::U32,
                },
                ast::Pattern::Literal {
                    lit: "c",
                    as_type: None,
                },
                ast::Pattern::Literal {
                    lit: "1",
                    as_type: Some(AsType::U32),
                },
            ])
        ) {
            panic!("unexpected structure")
        }
    }
}
