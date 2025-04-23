use std::iter::Enumerate;
use std::marker::PhantomData;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexedVector<K, V>
where
    K: Into<usize> + From<usize>,
{
    phantom: PhantomData<K>,
    data: Vec<V>,
}

impl<K, V> IndexedVector<K, V>
where
    K: Into<usize> + From<usize>,
{
    pub fn get(&self, index: &K) -> Option<&V> {
        let idx = unsafe { std::ptr::read(index) }.into();
        self.data.get(idx)
    }

    pub fn get_mut(&mut self, index: &K) -> Option<&mut V> {
        let idx = unsafe { std::ptr::read(index) }.into();
        self.data.get_mut(idx)
    }

    pub fn insert(&mut self, index: K, value: V) -> Option<V> {
        let idx = index.into();
        if idx > self.data.len() {
            panic!("Index out of bounds");
        } else if idx == self.data.len() {
            self.data.push(value);
            None
        } else {
            Some(std::mem::replace(&mut self.data[idx], value))
        }
    }

    pub fn iter(&self) -> IndexedVectorIter<K, V> {
        IndexedVectorIter {
            inner: self.data.iter().enumerate(),
            phantom: PhantomData,
        }
    }

    pub fn remove(&mut self, index: K) -> Option<V> {
        let idx = index.into();
        if idx < self.data.len() {
            Some(self.data.remove(idx))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn new() -> Self {
        Self {
            phantom: PhantomData,
            data: Vec::new(),
        }
    }
}

pub struct IndexedVectorIter<'a, K, V>
where
    K: Into<usize> + From<usize>,
{
    inner: Enumerate<std::slice::Iter<'a, V>>,
    phantom: PhantomData<K>,
}

impl<'a, K, V> Iterator for IndexedVectorIter<'a, K, V>
where
    K: Into<usize> + From<usize>,
{
    type Item = (K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(idx, val)| (K::from(idx), val))
    }
}

impl<'a, K, V> IntoIterator for &'a IndexedVector<K, V>
where
    K: Into<usize> + From<usize>,
{
    type Item = (K, &'a V);
    type IntoIter = IndexedVectorIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedVectorIter {
            inner: self.data.iter().enumerate(),
            phantom: PhantomData,
        }
    }
}

pub struct IndexedVectorIterMut<'a, K, V>
where
    K: Into<usize> + From<usize>,
{
    inner: Enumerate<std::slice::IterMut<'a, V>>,
    phantom: PhantomData<K>,
}

impl<'a, K, V> Iterator for IndexedVectorIterMut<'a, K, V>
where
    K: Into<usize> + From<usize>,
{
    type Item = (K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(idx, val)| (K::from(idx), val))
    }
}

impl<'a, K, V> IntoIterator for &'a mut IndexedVector<K, V>
where
    K: Into<usize> + From<usize>,
{
    type Item = (K, &'a mut V);
    type IntoIter = IndexedVectorIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IndexedVectorIterMut {
            inner: self.data.iter_mut().enumerate(),
            phantom: PhantomData,
        }
    }
}
