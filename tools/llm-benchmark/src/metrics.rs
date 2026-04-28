use crate::types::{
    BenchmarkSummary, DriftSummary, IterationReport, ItlPercentiles, MetricSummary,
};

pub fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

pub fn variance(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    values
        .iter()
        .map(|v| {
            let d = v - mean;
            d * d
        })
        .sum::<f64>()
        / values.len() as f64
}

pub fn summarize_metric(values: &[f64]) -> MetricSummary {
    if values.is_empty() {
        return MetricSummary {
            p50: 0.0,
            p95: 0.0,
            max: 0.0,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    let max = *sorted.last().unwrap_or(&0.0);
    MetricSummary {
        p50: percentile(&sorted, 0.5),
        p95: percentile(&sorted, 0.95),
        max,
    }
}

pub fn summarize_itl_percentiles(values: &[f64]) -> ItlPercentiles {
    if values.is_empty() {
        return ItlPercentiles {
            p50: 0.0,
            p90: 0.0,
            p95: 0.0,
            p99: 0.0,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));
    ItlPercentiles {
        p50: percentile(&sorted, 0.5),
        p90: percentile(&sorted, 0.9),
        p95: percentile(&sorted, 0.95),
        p99: percentile(&sorted, 0.99),
    }
}

pub fn compute_drift(reports: &[IterationReport]) -> DriftSummary {
    let mut early_tps = Vec::new();
    let mut late_tps = Vec::new();
    let mut early_itl = Vec::new();
    let mut late_itl = Vec::new();
    for report in reports {
        if report.run_status != "ok" || report.token_flow.len() < 4 {
            continue;
        }
        let half = report.token_flow.len() / 2;
        let early = &report.token_flow[..half];
        let late = &report.token_flow[half..];
        if !early.is_empty() && !late.is_empty() {
            early_tps
                .push(early.iter().map(|p| p.cumulative_tps).sum::<f64>() / early.len() as f64);
            late_tps.push(late.iter().map(|p| p.cumulative_tps).sum::<f64>() / late.len() as f64);
            early_itl
                .push(early.iter().map(|p| p.decode_us as f64).sum::<f64>() / early.len() as f64);
            late_itl.push(late.iter().map(|p| p.decode_us as f64).sum::<f64>() / late.len() as f64);
        }
    }
    let mean = |v: &[f64]| -> f64 {
        if v.is_empty() {
            0.0
        } else {
            v.iter().sum::<f64>() / v.len() as f64
        }
    };
    DriftSummary {
        tps_early_late_delta: mean(&late_tps) - mean(&early_tps),
        itl_early_late_delta_us: mean(&late_itl) - mean(&early_itl),
    }
}

pub fn compute_summary(runs: &[IterationReport]) -> BenchmarkSummary {
    let successful: Vec<&IterationReport> = runs.iter().filter(|r| r.run_status == "ok").collect();
    let ttft_values: Vec<f64> = successful.iter().map(|r| r.ttft_us as f64).collect();
    let tps_values: Vec<f64> = successful.iter().map(|r| r.tps_mean).collect();
    let itl_p99_values: Vec<f64> = successful.iter().map(|r| r.itl_p99_us).collect();
    let all_itl_values: Vec<f64> = successful
        .iter()
        .flat_map(|r| r.itl_samples_us.iter().map(|v| *v as f64))
        .collect();
    let generated_tokens_total = successful.iter().map(|r| r.generated_tokens).sum();
    BenchmarkSummary {
        measured_runs: runs.len(),
        failed_runs: runs.iter().filter(|r| r.run_status != "ok").count(),
        timed_out_runs: runs.iter().filter(|r| r.timed_out).count(),
        generated_tokens_total,
        ttft_us: summarize_metric(&ttft_values),
        tps: summarize_metric(&tps_values),
        itl_us: summarize_metric(&itl_p99_values),
        itl_percentiles_us: summarize_itl_percentiles(&all_itl_values),
        drift: compute_drift(runs),
    }
}
