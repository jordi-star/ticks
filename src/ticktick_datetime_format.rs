use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

const TICKTICK_DATETIME_FORMAT_STR: &str = "%Y-%m-%dT%T%z"; // "yyyy-MM-dd'T'HH:mm:ssZ"

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(TICKTICK_DATETIME_FORMAT_STR));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let dt = NaiveDateTime::parse_from_str(&s, TICKTICK_DATETIME_FORMAT_STR)
        .map_err(serde::de::Error::custom)?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}

pub mod optional_datetime {
    use chrono::{DateTime, Utc};
    use serde::{Serializer};

    pub fn serialize<S>(date_opt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date_opt {
            Some(date) => {
                let s = format!("{}", date.format(super::TICKTICK_DATETIME_FORMAT_STR));
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }
}
