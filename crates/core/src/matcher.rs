//!
//! Apply a path `&[&str]` to an HList of patterns, returning Some HList of matched
//! values if the path matches the patterns, or None if it doesn't.
//!
//! &["a", "1", "c"] • Literal("a") :: Variable(u32) :: Literal("c") :: HNil
//!     -> "a" :: 1 :: "c" :: HNil
//!

use core::marker::PhantomData;

use frunk::{HCons, HNil, hlist::HList};

pub trait Extract<'p>: Sized {
    fn from_str(s: &'p str) -> Option<Self>;
}

pub trait Extractor<'p> {
    type Output: HList;
    fn extract(&self, path: &[&'p str]) -> Option<Self::Output>;
}

#[derive(Debug)]
pub enum Pattern<'p, T: Extract<'p>> {
    Literal(T, PhantomData<&'p ()>),
    Variable(PhantomData<(T, &'p ())>),
}

pub fn extract<'p, P: Extractor<'p>>(path: &[&'p str], patterns: P) -> Option<P::Output> {
    patterns.extract(path)
}

impl<'p> Extract<'p> for &'p str {
    fn from_str(s: &'p str) -> Option<Self> {
        Some(s)
    }
}

impl<'p> Extract<'p> for u32 {
    fn from_str(s: &'p str) -> Option<Self> {
        s.parse().ok()
    }
}

impl<'p> Extractor<'p> for HNil {
    type Output = HNil;
    fn extract(&self, path: &[&'p str]) -> Option<Self::Output> {
        if path.is_empty() {
            Some(HNil)
        } else {
            // list lengths don't match
            None
        }
    }
}

impl<'p, T, Tail> Extractor<'p> for HCons<Pattern<'p, T>, Tail>
where
    T: PartialEq + Extract<'p>,
    Tail: HList + Extractor<'p>,
{
    type Output = HCons<T, Tail::Output>;

    fn extract(&self, path: &[&'p str]) -> Option<Self::Output> {
        if path.is_empty() {
            // list lengths don't match
            return None;
        }

        let (first, rest) = path.split_first()?;

        let head_value = match &self.head {
            Pattern::Literal(lit, _) => T::from_str(first).filter(|t| t == lit)?,
            Pattern::Variable(_) => T::from_str(first)?,
        };

        let tail_result = self.tail.extract(rest)?;

        Some(HCons {
            head: head_value,
            tail: tail_result,
        })
    }
}

pub mod patterns {
    use super::*;

    pub fn literal<'p, T: Extract<'p>>(t: T) -> Pattern<'p, T> {
        Pattern::Literal(t, PhantomData)
    }

    pub fn variable<'p, T: Extract<'p>>() -> Pattern<'p, T> {
        Pattern::<T>::Variable(PhantomData)
    }
}

#[cfg(test)]
mod tests {
    use frunk::*;

    use super::patterns as p;
    use super::*;

    #[test]
    fn usage() {
        let path = &["a", "b", "1"];
        let pattern = hlist![p::literal("a"), p::variable::<&str>(), p::literal(1)];
        let result = extract(path, pattern);

        assert!(result.is_some());
    }

    #[test]
    fn test_doc_example() {
        // From module docs:
        // &["a", "1", "c"] • Literal("a") :: Variable("second", u32) :: Literal("c") :: HNil
        //     -> Some("a" :: 1 :: "c" :: HNil)
        let path = &["a", "1", "c"];
        let pattern = hlist![p::literal("a"), p::variable::<u32>(), p::literal("c")];

        let result = extract(path, pattern);
        assert!(result.is_some());

        if let Some(hlist_pat![a, one, c]) = result {
            assert_eq!(a, "a");
            assert_eq!(one, 1u32);
            assert_eq!(c, "c");
        }
    }

    #[test]
    fn test_mismatch_literal() {
        let path = &["a", "b"];
        let pattern = hlist![p::literal("x"), p::variable::<&str>()];

        let result = extract(path, pattern);
        assert!(result.is_none());
    }

    #[test]
    fn test_length_mismatch() {
        let path = &["a"];
        let pattern = hlist![p::literal("a"), p::variable::<&str>()];

        let result = extract(path, pattern);
        assert!(result.is_none());
    }
}
