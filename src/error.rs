use pgrx::pg_sys::panic::ErrorReport;
use pgrx::PgSqlErrorCode;

#[derive(Debug)]
pub enum Error {
    AddressOptionRequired,
    QueryOptionRequired,
    VariableNotFound(String),
    TimeRequiresEquals,
    TimeRequiresTimestamp,
    TimestampInvalid,
    PrometheusError(prometheus_http_query::Error),
    NoResult,
    IoError(std::io::Error),
}

impl From<Error> for ErrorReport {
    fn from(value: Error) -> Self {
        match value {
            Error::VariableNotFound(name) => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_DYNAMIC_PARAMETER_VALUE_NEEDED,
                format!("Variable {} required, but not found in where clause", name),
                "",
            ),
            Error::QueryOptionRequired => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_ERROR,
                "Query option required",
                "",
            ),
            Error::AddressOptionRequired => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_ERROR,
                "Address option required",
                "",
            ),
            Error::TimeRequiresEquals => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_ERROR,
                "Time where clause requires '=' operator",
                "",
            ),
            Error::TimeRequiresTimestamp => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_ERROR,
                "Time where clause requires timestamp value",
                "",
            ),
            Error::TimestampInvalid => {
                ErrorReport::new(PgSqlErrorCode::ERRCODE_FDW_ERROR, "Invalid timestamp", "")
            }
            Error::NoResult => ErrorReport::new(PgSqlErrorCode::ERRCODE_FDW_ERROR, "No result", ""),
            Error::PrometheusError(e) => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_ERROR,
                format!("Prometheus error: {}", e),
                "",
            ),
            Error::IoError(e) => ErrorReport::new(
                PgSqlErrorCode::ERRCODE_FDW_ERROR,
                format!("IO error: {}", e),
                "",
            ),
        }
    }
}

impl From<prometheus_http_query::Error> for Error {
    fn from(value: prometheus_http_query::Error) -> Self {
        Error::PrometheusError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
