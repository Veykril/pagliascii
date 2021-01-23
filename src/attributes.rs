use std::collections::HashMap;
use std::iter::FromIterator;

#[derive(Clone, Debug, Default)]
pub struct AttributeMap {
    inner: HashMap<String, String>,
}

impl AttributeMap {
    #[inline]
    pub fn contains<K>(&self, k: &K) -> bool
    where
        String: std::borrow::Borrow<K>,
        K: std::hash::Hash + Eq + ?Sized,
    {
        self.inner.contains_key(k)
    }
}

impl<T, U> FromIterator<(T, U)> for AttributeMap
where
    T: Into<String>,
    U: Into<String>,
{
    fn from_iter<II: IntoIterator<Item = (T, U)>>(iter: II) -> Self {
        AttributeMap {
            inner: FromIterator::from_iter(iter.into_iter().map(|(t, u)| (t.into(), u.into()))),
        }
    }
}
