//! Custom screener filter-condition building, shared by the REST
//! `/v2/screeners/custom` handler and the GraphQL `customScreener` resolver
//! so both transports validate/build filters identically.

use finance_query::{
    EquityField, EquityScreenerQuery, FinanceError, FundField, FundScreenerQuery, QueryCondition,
    ScreenerField, ScreenerFieldExt, finance,
};

pub enum ScreenerError {
    InvalidField(String),
    InvalidOperator(String),
    Finance(FinanceError),
}

impl From<FinanceError> for ScreenerError {
    fn from(e: FinanceError) -> Self {
        ScreenerError::Finance(e)
    }
}

fn numeric_value(v: &serde_json::Value) -> Result<f64, ScreenerError> {
    v.as_f64()
        .ok_or_else(|| ScreenerError::InvalidField(format!("Expected a numeric value, got: {}", v)))
}

fn between_values(v: &serde_json::Value) -> Result<(f64, f64), ScreenerError> {
    match v {
        serde_json::Value::Array(arr) if arr.len() == 2 => {
            let min = arr[0].as_f64().ok_or_else(|| {
                ScreenerError::InvalidField("BTWN first value must be numeric".to_string())
            })?;
            let max = arr[1].as_f64().ok_or_else(|| {
                ScreenerError::InvalidField("BTWN second value must be numeric".to_string())
            })?;
            Ok((min, max))
        }
        _ => Err(ScreenerError::InvalidField(
            "BTWN operator requires an array of exactly 2 numeric values: [min, max]".to_string(),
        )),
    }
}

fn build_condition_from_filter<F: ScreenerField + ScreenerFieldExt>(
    field: F,
    operator: &str,
    value: &serde_json::Value,
) -> Result<QueryCondition<F>, ScreenerError> {
    let op = operator.to_lowercase();
    match op.as_str() {
        "eq" | "=" | "==" => match value {
            serde_json::Value::String(s) => Ok(field.eq_str(s.clone())),
            serde_json::Value::Number(n) => Ok(field.eq_num(n.as_f64().unwrap_or(0.0))),
            _ => Ok(field.eq_str(value.to_string())),
        },
        "gt" | ">" => Ok(field.gt(numeric_value(value)?)),
        "gte" | ">=" => Ok(field.gte(numeric_value(value)?)),
        "lt" | "<" => Ok(field.lt(numeric_value(value)?)),
        "lte" | "<=" => Ok(field.lte(numeric_value(value)?)),
        "btwn" | "between" => {
            let (min, max) = between_values(value)?;
            Ok(field.between(min, max))
        }
        _ => Err(ScreenerError::InvalidOperator(format!(
            "Invalid operator: '{}'. Valid: eq, gt, gte, lt, lte, btwn",
            operator
        ))),
    }
}

pub fn build_equity_condition(
    field: &str,
    operator: &str,
    value: &serde_json::Value,
) -> Result<QueryCondition<EquityField>, ScreenerError> {
    let f = field.parse::<EquityField>().map_err(|_| {
        ScreenerError::InvalidField(format!(
            "Unknown equity field: '{}'. See EquityField for valid values.",
            field
        ))
    })?;
    build_condition_from_filter(f, operator, value)
}

pub fn build_fund_condition(
    field: &str,
    operator: &str,
    value: &serde_json::Value,
) -> Result<QueryCondition<FundField>, ScreenerError> {
    let f = field.parse::<FundField>().map_err(|_| {
        ScreenerError::InvalidField(format!(
            "Unknown fund field: '{}'. See FundField for valid values.",
            field
        ))
    })?;
    build_condition_from_filter(f, operator, value)
}

/// A single filter condition in transport-agnostic form.
pub struct FilterInput {
    pub field: String,
    pub operator: String,
    pub value: serde_json::Value,
}

pub async fn run_custom_equity_screener(
    size: u32,
    offset: u32,
    sort_field: Option<&str>,
    sort_ascending: bool,
    filters: &[FilterInput],
) -> Result<finance_query::ScreenerResults, ScreenerError> {
    let mut query = EquityScreenerQuery::new().size(size).offset(offset);

    if let Some(sort_field_str) = sort_field {
        let field = sort_field_str.parse::<EquityField>().map_err(|_| {
            ScreenerError::InvalidField(format!(
                "Unknown equity sort field: '{}'. Use EquityField enum values.",
                sort_field_str
            ))
        })?;
        query = query.sort_by(field, sort_ascending);
    }

    for filter in filters {
        let condition = build_equity_condition(&filter.field, &filter.operator, &filter.value)?;
        query = query.add_condition(condition);
    }

    Ok(finance::custom_screener(query).await?)
}

pub async fn run_custom_fund_screener(
    size: u32,
    offset: u32,
    sort_field: Option<&str>,
    sort_ascending: bool,
    filters: &[FilterInput],
) -> Result<finance_query::ScreenerResults, ScreenerError> {
    let mut query = FundScreenerQuery::new().size(size).offset(offset);

    if let Some(sort_field_str) = sort_field {
        let field = sort_field_str.parse::<FundField>().map_err(|_| {
            ScreenerError::InvalidField(format!(
                "Unknown fund sort field: '{}'. Use FundField enum values.",
                sort_field_str
            ))
        })?;
        query = query.sort_by(field, sort_ascending);
    }

    for filter in filters {
        let condition = build_fund_condition(&filter.field, &filter.operator, &filter.value)?;
        query = query.add_condition(condition);
    }

    Ok(finance::custom_screener(query).await?)
}
