use serde::{ser::SerializeSeq, Deserialize, Serialize};

/// A wrapper around a sequence of items
///
/// When serialized or deserialized, it behaves as if each item was wrapped in a
/// struct with a single attribute `"item"`.
///
/// ```rust
/// # fn main() -> serde_arrow::_impl::PanicOnError<()> {
/// # use serde_arrow::utils::Items;
/// #
/// assert_eq!(
///     serde_json::to_string(&Items([13, 21]))?,
///     r#"[{"item":13},{"item":21}]"#,
/// );
///
/// let Items(items): Items<Vec<u32>> = serde_json::from_str(r#"[
///     {"item": 21},
///     {"item": 42}
/// ]"#)?;
/// assert_eq!(items, &[21, 42]);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq)]
pub struct Items<T>(
    /// The wrapped object
    pub T,
);

/// A wrapper around a single item
///
/// When serialized or deserialized, it behaves as if the Item was wrapped in a
/// struct with a single attribute `"item"`.
///
/// ```rust
/// # fn main() -> serde_arrow::_impl::PanicOnError<()> {
/// # use serde_arrow::utils::Item;
/// #
/// assert_eq!(serde_json::to_string(&Item(42))?, r#"{"item":42}"#);
///
/// let Item(item): Item<u32> = serde_json::from_str(r#"{"item":21}"#)?;
/// assert_eq!(item, 21);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, PartialEq)]
pub struct Item<T>(
    /// The wrapped object
    pub T,
);

impl<T: Serialize> Serialize for Item<T> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        #[derive(Debug, Serialize)]
        struct Item<'a, T> {
            item: &'a T,
        }
        Item { item: &self.0 }.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Item<T> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(Debug, Deserialize)]
        struct Item<T> {
            item: T,
        }
        let item = Item::<T>::deserialize(deserializer)?;
        Ok(Item(item.item))
    }
}

// TODO: implement for all types?
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Items<Vec<T>> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        let items = Vec::<Item<T>>::deserialize(deserializer)?
            .into_iter()
            .map(|item| item.0)
            .collect();
        Ok(Items(items))
    }
}

impl<T: Serialize> Serialize for Items<Vec<T>> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        Items(self.0.as_slice()).serialize(serializer)
    }
}

impl<'a, T: Serialize> Serialize for Items<&'a Vec<T>> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        Items(self.0.as_slice()).serialize(serializer)
    }
}

impl<const N: usize, T: Serialize> Serialize for Items<[T; N]> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        Items(self.0.as_slice()).serialize(serializer)
    }
}

impl<'a, const N: usize, T: Serialize> Serialize for Items<&'a [T; N]> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        Items(self.0.as_slice()).serialize(serializer)
    }
}

impl<'a, T: Serialize> Serialize for Items<&'a [T]> {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            seq.serialize_element(&Item(item))?;
        }
        seq.end()
    }
}
