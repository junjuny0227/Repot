use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportMeta {
    pub repot_version: String,
    pub date: String,
    pub generated_at: String,
}
impl ExportMeta {
    pub fn now() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (year, month, day) = civil_from_days((seconds / 86400) as i64);
        let date = format!("{year:04}-{month:02}-{day:02}");
        Self {
            repot_version: env!("CARGO_PKG_VERSION").into(),
            generated_at: format!(
                "{date} {:02}:{:02} UTC",
                seconds / 3600 % 24,
                seconds / 60 % 60
            ),
            date,
        }
    }
}
/// Gregorian date from Unix epoch days, using Howard Hinnant's civil-from-days algorithm.
pub fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    (
        (year + i64::from(month <= 2)) as i32,
        month as u32,
        day as u32,
    )
}
