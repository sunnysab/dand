/// Reference: https://stackoverflow.com/questions/67803619/using-serdeserialize-with-optionchronodatetime
pub mod datetime_serde {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M";

    pub fn serialize<S>(date: &Option<DateTime<Local>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(date) = date {
            let s = format!("{}", date.format(FORMAT));
            serializer.serialize_str(&s)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(string) = Option::<String>::deserialize(deserializer)? {
            let date = Local.datetime_from_str(&string, FORMAT).map_err(|_| {
                serde::de::Error::custom(format!(
                    "invalid date: {}, please make date with format: {}",
                    string, FORMAT
                ))
            })?;
            return Ok(Some(date));
        }

        Ok(None)
    }
}
