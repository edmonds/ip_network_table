use ip_network::IpNetwork;
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};
use std::marker::PhantomData;

use crate::IpNetworkTable;

impl<T> Serialize for IpNetworkTable<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (len_ipv4, len_ipv6) = self.len();
        let mut map_serializer = serializer.serialize_map(Some(len_ipv4 + len_ipv6))?;
        for (k, v) in self.iter() {
            map_serializer.serialize_entry(&k, v)?;
        }
        map_serializer.end()
    }
}

struct IpNetworkTableVisitor<T> {
    marker: PhantomData<IpNetworkTable<T>>,
}

impl<T> IpNetworkTableVisitor<T> {
    pub fn new() -> Self {
        IpNetworkTableVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for IpNetworkTableVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = IpNetworkTable<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("IpNetworkTable")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(IpNetworkTable::new())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut table = IpNetworkTable::new();
        while let Some((key, value)) = map.next_entry::<IpNetwork, T>()? {
            table.insert(key, value);
        }
        Ok(table)
    }
}

impl<'de, T> Deserialize<'de> for IpNetworkTable<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(IpNetworkTableVisitor::<T>::new())
    }
}

#[cfg(test)]
mod tests {
    use ip_network::{IpNetwork, Ipv4Network, Ipv6Network};
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    use crate::IpNetworkTable;

    #[test]
    fn test_serde() {
        let mut table = IpNetworkTable::<i32>::new();

        table.insert(IpNetwork::from_str("127.0.0.1/32").unwrap(), 1);
        table.insert(Ipv6Network::from_str("2001:db8::/32").unwrap(), 2);
        table.insert(Ipv4Network::from_str("127.0.0.2/32").unwrap(), 3);
        table.insert(IpNetwork::from_str("127.0.0.0/8").unwrap(), 4);

        let serializer = serde_assert::Serializer::builder().build();

        let mut deserializer =
            serde_assert::Deserializer::builder(table.serialize(&serializer).unwrap()).build();

        assert_eq!(
            IpNetworkTable::<i32>::deserialize(&mut deserializer).unwrap(),
            table
        );
    }
}
