use std::{
    fmt::Debug,
    iter::{Cloned, Enumerate},
    ops::Deref,
    slice::Iter,
};

use nom::Input;

/// Purely a newtype wrapper over &[T]
#[derive(Clone, Debug, PartialEq)]
pub struct Slice<T>(pub T);

impl<'a, T> From<&'a [T]> for Slice<&'a [T]> {
    fn from(value: &'a [T]) -> Self {
        Self(value)
    }
}

impl<T> Deref for Slice<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Clone> Input for Slice<&'a [T]> {
    type Item = T;

    type Iter = Cloned<Iter<'a, T>>;

    type IterIndices = Enumerate<Self::Iter>;

    fn input_len(&self) -> usize {
        self.0.len()
    }

    fn take(&self, index: usize) -> Self {
        self.0[..index].into()
    }

    fn take_from(&self, index: usize) -> Self {
        self.0[index..].into()
    }

    fn take_split(&self, index: usize) -> (Self, Self) {
        let (prefix, suffix) = self.0.split_at(index);

        (prefix.into(), suffix.into())
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.iter().position(|b| predicate(b.clone()))
    }

    fn iter_elements(&self) -> Self::Iter {
        self.0.iter().cloned()
    }

    fn iter_indices(&self) -> Self::IterIndices {
        self.iter_elements().enumerate()
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        if count < self.input_len() {
            Ok(count)
        } else {
            Err(nom::Needed::new(count - self.input_len()))
        }
    }
}

#[cfg(test)]
mod tests {
    use nom::{Parser, bytes::complete::take, multi::count};

    use super::Slice;

    #[test]
    fn test_my_slice() {
        let input: Slice<&[_]> = ["a", "b", "c"].as_slice().into();

        let mut parser = take::<_, _, nom::error::Error<_>>(1_usize);

        let (rest, x) = parser.parse(input).unwrap();
        assert_eq!(x.0, &["a"]);
        assert_eq!(rest.0, &["b", "c"]);
    }

    #[test]
    fn test_combinator() {
        let input: Slice<&[_]> = ["a", "b", "c"].as_slice().into();

        let parser = take::<_, _, nom::error::Error<_>>(1_usize);
        let mut parser = count(parser, 2);

        let (rest, x) = parser.parse(input).unwrap();
        assert_eq!(&x, &[["a"].as_slice().into(), ["b"].as_slice().into()]);
        assert_eq!(rest.0, &["c"]);
    }
}
