use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

use crate::types::BenchmarkReport;

pub fn model_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)
        .with_context(|| format!("failed to read model for hash: {}", path.display()))?;
    let digest = Sha256::digest(bytes);
    Ok(format!("{digest:x}"))
}

pub fn write_jsonl_flow(report: &BenchmarkReport, output_path: &Path) -> Result<()> {
    let mut lines = Vec::new();
    for run in &report.runs {
        for point in &run.token_flow {
            lines.push(
                serde_json::json!({
                    "profile": run.profile,
                    "iteration": run.iteration,
                    "token_index": point.token_index,
                    "decode_us": point.decode_us,
                    "since_generation_start_us": point.since_generation_start_us,
                    "cumulative_tps": point.cumulative_tps,
                })
                .to_string(),
            );
        }
    }
    fs::write(output_path, lines.join("\n"))
        .with_context(|| format!("failed to write JSONL report to {}", output_path.display()))?;
    Ok(())
}
