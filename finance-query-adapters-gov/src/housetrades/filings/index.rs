//! Yearly filing index parsing (`{year}FD.txt`).

use chrono::NaiveDate;

use super::super::models::PtrIndexEntry;

const FILING_TYPE_PTR: &str = "P";

/// `M/D/YYYY` (no fixed padding) to `YYYY-MM-DD`.
pub(super) fn parse_us_date(raw: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(raw.trim(), "%m/%d/%Y").ok()
}

/// Parse the tab-delimited index into PTR entries only, skipping every other
/// filing type and any row that doesn't have all nine columns.
pub(super) fn parse_index(text: &str, year: i32) -> Vec<PtrIndexEntry> {
    text.lines()
        .skip(1)
        .filter_map(|line| {
            let mut cols = line.split('\t');
            let _prefix = cols.next()?;
            let last = cols.next()?;
            let first = cols.next()?;
            let _suffix = cols.next()?;
            let filing_type = cols.next()?;
            let state_dst = cols.next()?;
            let _year = cols.next()?;
            let filing_date = cols.next()?;
            let doc_id = cols.next()?;
            (filing_type == FILING_TYPE_PTR).then(|| PtrIndexEntry {
                last: last.to_string(),
                first: first.to_string(),
                state_dst: state_dst.to_string(),
                year,
                filing_date: filing_date.to_string(),
                doc_id: doc_id.to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_text() -> &'static str {
        "Prefix\tLast\tFirst\tSuffix\tFilingType\tStateDst\tYear\tFilingDate\tDocID\n\
         \tAaron\tRichard\t\tD\tMI04\t2025\t3/24/2025\t40003749\n\
         Hon.\tAderholt\tRobert B.\t\tP\tAL04\t2025\t9/10/2025\t20032062\n\
         \tAbrevaya\tDavid\t\tW\tIL09\t2025\t5/19/2025\t8005\n"
    }

    #[test]
    fn only_ptr_rows_survive() {
        let entries = parse_index(index_text(), 2025);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].last, "Aderholt");
        assert_eq!(entries[0].state_dst, "AL04");
        assert_eq!(entries[0].doc_id, "20032062");
    }

    #[test]
    fn non_padded_us_dates_parse() {
        assert_eq!(
            parse_us_date("9/10/2025"),
            NaiveDate::from_ymd_opt(2025, 9, 10)
        );
        assert_eq!(
            parse_us_date("08/18/2026"),
            NaiveDate::from_ymd_opt(2026, 8, 18)
        );
    }
}
