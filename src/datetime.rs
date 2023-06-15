/// Reference: https://stackoverflow.com/questions/67803619/using-serdeserialize-with-optionchronodatetime
pub mod datetime_serde {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: [&str; 3] = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M", "%Y-%m-%d"];

    pub fn serialize<S>(date: &Option<DateTime<Local>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(date) = date {
            let s = format!("{}", date.format(FORMAT[0]));
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
            for date_format in FORMAT {
                if let Ok(date) = Local.datetime_from_str(&string, date_format) {
                    return Ok(Some(date));
                }
            }
            return Err(serde::de::Error::custom(format!(
                "invalid date: {}, please make date with any of formats: {}.",
                string,
                FORMAT.join(", ")
            )));
        }

        Ok(None)
    }
}
