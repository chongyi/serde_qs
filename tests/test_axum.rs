#![cfg(feature = "axum")]

fn from_str<'de, D, S>(deserializer: D) -> Result<S, D::Error>
where
    D: serde::Deserializer<'de>,
    S: std::str::FromStr,
{
    let s = <&str as serde::Deserialize>::deserialize(deserializer)?;
    S::from_str(&s).map_err(|_| D::Error::custom("could not parse string"))
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct Query {
    foo: u64,
    bars: Vec<u64>,
    #[serde(flatten)]
    common: CommonParams,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
struct CommonParams {
    #[serde(deserialize_with = "from_str")]
    limit: u64,
    #[serde(deserialize_with = "from_str")]
    offset: u64,
    #[serde(deserialize_with = "from_str")]
    remaining: bool,
}

