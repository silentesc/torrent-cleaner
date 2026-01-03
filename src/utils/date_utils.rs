use chrono::{Local, NaiveDate, NaiveDateTime, ParseError};

static DATE_FORMAT: &str = "%Y-%m-%d";
static DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub struct DateUtils;

impl DateUtils {
    pub fn get_current_local_naive_date() -> NaiveDate {
        Local::now().date_naive()
    }

    pub fn get_current_local_naive_datetime() -> NaiveDateTime {
        Local::now().naive_local()
    }

    pub fn convert_naive_date_to_string(naive_date: NaiveDate) -> String {
        naive_date.format(DATE_FORMAT).to_string()
    }

    pub fn convert_naive_datetime_to_string(naive_datetime: NaiveDateTime) -> String {
        naive_datetime.format(DATETIME_FORMAT).to_string()
    }

    pub fn parse_naive_date_from_str(str: &str) -> Result<NaiveDate, ParseError> {
        NaiveDate::parse_from_str(str, DATE_FORMAT)
    }

    pub fn parse_naive_datetime_from_str(str: &str) -> Result<NaiveDateTime, ParseError> {
        NaiveDateTime::parse_from_str(str, DATETIME_FORMAT)
    }
}
