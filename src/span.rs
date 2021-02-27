use nom::{
    Compare, FindSubstring, InputIter, InputLength, InputTake, Offset, Slice, UnspecializedInput,
};
use nom_locate::LocatedSpan;

use std::fmt;
use std::ops::{self, Range, RangeFrom, RangeTo};
use std::str::{CharIndices, Chars};

#[derive(Clone, Copy)]
pub struct Span<'a>(pub LocatedSpan<&'a str>);

impl<'a> Span<'a> {
    pub fn new(i: &'a str) -> Self {
        Self(LocatedSpan::new(i))
    }
}

impl<'a> From<&'a str> for Span<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl Eq for Span<'_> {}
impl PartialEq for Span<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.fragment().eq(other.0.fragment())
    }
}

impl fmt::Debug for Span<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0.fragment())
    }
}

impl<'a> ops::Deref for Span<'a> {
    type Target = LocatedSpan<&'a str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Span<'_> {
    fn as_ref(&self) -> &str {
        self.0.fragment()
    }
}

impl InputTake for Span<'_> {
    #[inline(always)]
    fn take(&self, count: usize) -> Self {
        Self(self.0.take(count))
    }
    #[inline(always)]
    fn take_split(&self, count: usize) -> (Self, Self) {
        let (l, r) = self.0.take_split(count);
        (Self(l), Self(r))
    }
}

impl Compare<&str> for Span<'_> {
    #[inline(always)]
    fn compare(&self, t: &str) -> nom::CompareResult {
        self.0.compare(t)
    }
    #[inline(always)]
    fn compare_no_case(&self, t: &str) -> nom::CompareResult {
        self.0.compare_no_case(t)
    }
}

impl FindSubstring<&str> for Span<'_> {
    #[inline(always)]
    fn find_substring(&self, substr: &str) -> Option<usize> {
        self.0.find_substring(substr)
    }
}

impl Slice<RangeTo<usize>> for Span<'_> {
    #[inline(always)]
    fn slice(&self, range: RangeTo<usize>) -> Self {
        Self(self.0.slice(range))
    }
}

impl Slice<RangeFrom<usize>> for Span<'_> {
    #[inline(always)]
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        Self(self.0.slice(range))
    }
}

impl<'a> Slice<Range<usize>> for Span<'a> {
    #[inline(always)]
    fn slice(&self, range: Range<usize>) -> Self {
        Self(self.0.slice(range))
    }
}

impl<'a> InputLength for Span<'a> {
    #[inline(always)]
    fn input_len(&self) -> usize {
        self.0.input_len()
    }
}

impl<'a> InputIter for Span<'a> {
    type Item = char;
    type Iter = CharIndices<'a>;
    type IterElem = Chars<'a>;

    #[inline(always)]
    fn iter_indices(&self) -> Self::Iter {
        self.0.iter_indices()
    }
    #[inline(always)]
    fn iter_elements(&self) -> Self::IterElem {
        self.0.iter_elements()
    }
    #[inline(always)]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.position(predicate)
    }
    #[inline(always)]
    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.0.slice_index(count)
    }
}

impl<'a> UnspecializedInput for Span<'a> {}

impl<'a> Offset for Span<'a> {
    fn offset(&self, second: &Self) -> usize {
        self.0.offset(&second.0)
    }
}
