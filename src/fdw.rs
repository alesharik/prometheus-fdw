use crate::error::{Error, Result};
use crate::query::PromQuery;
use crate::utils::{from_chrono, to_chrono};
use chrono::{NaiveDateTime, TimeZone, Utc};
use prometheus_http_query::response::InstantVector;
use prometheus_http_query::Client;
use std::collections::HashMap;
use std::str::FromStr;
use lazy_static::lazy_static;
use supabase_wrappers::prelude::*;

lazy_static!(
    static ref RT: Runtime = Runtime::new().unwrap();
);

#[wrappers_fdw(
    version = "0.1.0",
    author = "alesharik",
    website = "https://github.com/alesharik/prometheus_fdw",
    error_type = "Error"
)]
pub(crate) struct PrometheusFdw {
    client: Client,
    tgt_columns: Vec<Column>,
    result: Vec<InstantVector>,
    idx: usize,
}

impl ForeignDataWrapper<Error> for PrometheusFdw {
    fn new(options: &HashMap<String, String>) -> Result<Self> {
        Ok(Self {
            client: Client::from_str(options.get("address").ok_or(Error::AddressOptionRequired)?)?,
            tgt_columns: vec![],
            idx: 0,
            result: vec![],
        })
    }

    fn begin_scan(
        &mut self,
        quals: &[Qual],
        columns: &[Column],
        _sorts: &[Sort],
        _limit: &Option<Limit>,
        options: &HashMap<String, String>,
    ) -> Result<()> {
        let query = options.get("query").ok_or(Error::QueryOptionRequired)?;
        let query = PromQuery::parse(query).compile(quals)?;
        let time = quals.iter().find(|q| &q.field == "time").map(|q| {
            if q.operator != "=" {
                return Err(Error::TimeRequiresEquals);
            }
            match &q.value {
                Value::Cell(value) => match value {
                    Cell::Timestamp(ts) => Ok(to_chrono(ts.clone())),
                    _ => Err(Error::TimeRequiresTimestamp),
                },
                Value::Array(_) => Err(Error::TimeRequiresTimestamp),
            }
        });

        let mut req = self.client.query(query);
        if let Some(time) = time {
            req = req.at(time??.timestamp());
        }
        let result = RT.block_on(async move { req.get().await })?;
        self.result = result
            .data()
            .as_vector()
            .map(|v| v.to_vec())
            .unwrap_or(vec![]);
        self.tgt_columns = columns.to_vec();
        self.idx = 0;
        Ok(())
    }

    fn iter_scan(&mut self, row: &mut Row) -> Result<Option<()>> {
        if self.idx >= self.result.len() {
            return Ok(None)
        }

        let data: &InstantVector = &self.result[self.idx];
        for col in &self.tgt_columns {
            if data.metric().contains_key(&col.name) {
                row.push(
                    &col.name,
                    Some(Cell::String(
                        data.metric().get(&col.name).unwrap().to_string(),
                    )),
                );
            } else if col.name == "value" {
                row.push(&col.name, Some(Cell::F64(data.sample().value())));
            } else if col.name == "timestamp" {
                let ts = data.sample().timestamp();
                let ts = Utc.from_utc_datetime(
                    &NaiveDateTime::from_timestamp_millis((ts * 1000.0) as i64).unwrap(),
                );
                row.push(&col.name, Some(Cell::Timestamp(from_chrono(ts))));
            } else {
                row.push(&col.name, None);
            }
        }
        self.idx += 1;
        Ok(Some(()))
    }

    fn end_scan(&mut self) -> Result<()> {
        self.result.clear();
        Ok(())
    }
}
