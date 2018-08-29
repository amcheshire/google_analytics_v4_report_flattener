use joinery::Joinable;
use types::*;

pub fn response_to_delimited_reports(response: &ReportResponse, delimiter: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::with_capacity(response.reports.len());
    for report in &response.reports {
        result.push(report_to_flat(&report, delimiter));
    }

    result
}

fn report_to_flat(report: &Report, delimiter: &str) -> String {
    let empty_vec = Vec::with_capacity(0);
    let dimension_header_iter = report.column_header.dimensions
        .as_ref()
        .unwrap_or(&empty_vec) // TODO: this is pretty ugly
        .iter()
        .map(|entry| format!("\"{}\"", entry));

    let metric_header_iter = report
        .get_metric_header_iterator()
        .map(|entry: &MetricHeaderEntry| format!("\"{}\"", &entry.name));

    let mut result = format!(
        "{}\n",
        dimension_header_iter
            .chain(metric_header_iter)
            .join_with(delimiter)
            .to_string()
    );

    for report_row in report.data.rows.as_ref().unwrap().iter() {
        if let Some(ref dimensions) = report_row.dimensions {
            result.push_str(
                dimensions
                    .iter()
                    .map(|entry| format!("\"{}\"", entry))
                    .join_with(delimiter)
                    .to_string()
                    .as_str(),
            );
            result.push_str(delimiter);
        };
        let metric_data = report_row
            .metrics
            .iter()
            .flat_map(|date_range_value| date_range_value.values.iter())
            .join_with(delimiter)
            .to_string();

        result.push_str(format!("{}\n", metric_data).as_str());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs;
    use std::path::PathBuf;
    use types::ReportResponse;

    #[test]
    fn no_dimensions() {
        let data: String = fs::read_to_string(PathBuf::from(format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/test_reports/no_dimensions.json"
        ))).unwrap();

        let deserialized_response: ReportResponse = serde_json::from_str(data.as_str()).unwrap();

        assert_eq!(
            report_to_flat(&deserialized_response.reports[0], ","),
            "\"ga:sessions\"\n44\n".to_string()
        )
    }

    #[test]
    fn single_dimension_and_metric() {
        let data: String = fs::read_to_string(PathBuf::from(format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/test_reports/single_dimension_and_metric.json"
        ))).unwrap();

        let deserialized_response: ReportResponse = serde_json::from_str(data.as_str()).unwrap();

        assert_eq!(
            report_to_flat(&deserialized_response.reports[0], "|delimiter|"),
            indoc!(
                r#""ga:deviceCategory"|delimiter|"ga:sessions"
                "desktop"|delimiter|43
                "mobile"|delimiter|1
                "#
            ).to_string()
        )
    }

    #[test]
    fn multiple_dimensions_and_metrics() {
        let data: String = fs::read_to_string(PathBuf::from(format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/test_reports/multiple_dimensions_and_metrics.json"
        ))).unwrap();

        let deserialized_response: ReportResponse = serde_json::from_str(data.as_str()).unwrap();

        assert_eq!(
            report_to_flat(&deserialized_response.reports[0], ","),
            indoc!(
                r#""ga:deviceCategory","ga:country","ga:sessions","ga:bounces"
                "desktop","Australia",1,1
                "desktop","France",39,21
                "desktop","United States",3,1
                "mobile","Brazil",1,0
                "#
            ).to_string()
        )
    }

    #[test]
    fn large_report() {
        let data: String = fs::read_to_string(PathBuf::from(format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/test_reports/large_report.json"
        ))).unwrap();

        let deserialized_response: ReportResponse = serde_json::from_str(data.as_str()).unwrap();

        assert!(report_to_flat(&deserialized_response.reports[0], ",").len() > 0)
    }

    #[test]
    fn multiple_reports() {
        let data: String = fs::read_to_string(PathBuf::from(format!(
            "{}{}",
            env!("CARGO_MANIFEST_DIR"),
            "/test_reports/multiple_reports.json"
        ))).unwrap();

        let deserialized_response: ReportResponse = serde_json::from_str(data.as_str()).unwrap();

        assert_eq!(
            response_to_delimited_reports(&deserialized_response, ","),
            vec![
                indoc!(
                    r#""ga:deviceCategory","ga:sessions","ga:bounces"
                "desktop",25,17
                "mobile",2,2
                "#
                ).to_string(),
                indoc!(
                    r#""ga:country","ga:sessions","ga:bounces"
                "Azerbaijan",1,0
                "France",18,11
                "Japan",4,4
                "Switzerland",1,1
                "United States",3,3
                "#
                ).to_string(),
            ]
        )
    }
}