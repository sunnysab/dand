/// Reference: https://stackoverflow.com/questions/67803619/using-serdeserialize-with-optionchronodatetime
pub mod datetime_serde {
    use chrono::{DateTime, Local, LocalResult, NaiveDateTime, TimeZone};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: [&str; 2] = ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"];

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
            // Pure date, in format "%Y-%m-%d"
            if string.len() <= 4 + 1 + 2 + 1 + 2 {
                let date = string.as_str();
                let datetime = format!("{date} 00:00:00");
                if let Some(result) = parse_local_datetime(&datetime, FORMAT[0]) {
                    return Ok(Some(result));
                }
            } else {
                // Date and time
                let datetime = string.as_str();
                for dt_format in FORMAT {
                    if let Some(dt) = parse_local_datetime(datetime, dt_format) {
                        return Ok(Some(dt));
                    }
                }
            }

            return Err(serde::de::Error::custom(format!(
                "invalid date: {}, please make datetime in any of following formats: {} or put pure date %Y-%m-%d.",
                string,
                FORMAT.join(", ")
            )));
        }

        Ok(None)
    }

    fn parse_local_datetime(datetime: &str, format: &str) -> Option<DateTime<Local>> {
        let naive = NaiveDateTime::parse_from_str(datetime, format).ok()?;
        match Local.from_local_datetime(&naive) {
            LocalResult::Single(dt) => Some(dt),
            LocalResult::Ambiguous(earliest, _) => Some(earliest),
            LocalResult::None => None,
        }
    }
}
