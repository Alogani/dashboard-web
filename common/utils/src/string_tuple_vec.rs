use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::marker::PhantomData;

pub fn deserialize<'de, D, V>(deserializer: D) -> Result<Vec<(String, V)>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    struct StringTupleVecVisitor<V>(PhantomData<V>);

    impl<'de, V> Visitor<'de> for StringTupleVecVisitor<V>
    where
        V: Deserialize<'de>,
    {
        type Value = Vec<(String, V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map with string keys")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::new();

            while let Some((key, value)) = access.next_entry()? {
                vec.push((key, value));
            }

            Ok(vec)
        }
    }

    deserializer.deserialize_map(StringTupleVecVisitor(PhantomData))
}
