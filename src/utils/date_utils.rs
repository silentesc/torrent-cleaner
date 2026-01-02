use chrono::{Local, NaiveDate, ParseError};

static DATE_FORMAT: &str = "%Y-%m-%d";

pub struct DateUtils;

impl DateUtils {
    pub fn get_current_local_naive_date() -> NaiveDate {
        Local::now().date_naive()
    }

    pub fn convert_naive_date_to_string(naive_date: NaiveDate) -> String {
        naive_date.format(DATE_FORMAT).to_string()
    }

    pub fn parse_naive_date_from_str(str: &str) -> Result<NaiveDate, ParseError> {
        NaiveDate::parse_from_str(str, DATE_FORMAT)
    }
}
