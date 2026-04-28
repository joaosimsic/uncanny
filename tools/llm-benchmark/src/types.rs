use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(name = "llm-benchmark")]
#[command(about = "Benchmark conversational fluidity for local llama.cpp inference")]
pub struct CliArgs {
    #[arg(long)]
    pub config: Option<PathBuf>,
    #[arg(long)]
    pub model_path: Option<PathBuf>,
    #[arg(long)]
    pub prompt: Vec<String>,
    #[arg(long)]
    pub iterations: Option<usize>,
    #[arg(long)]
    pub warmup_iterations: Option<usize>,
    #[arg(long)]
    pub n_threads: Option<usize>,
    #[arg(long)]
    pub gpu_layers: Option<u32>,
    #[arg(long)]
    pub max_tokens: Option<usize>,
    #[arg(long)]
    pub cooldown_secs: Option<f64>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub jsonl_output: Option<PathBuf>,
    #[arg(long)]
    pub seed: Option<u32>,
    #[arg(long, default_value = "greedy")]
    pub sampler: SamplerKind,
    #[arg(long)]
    pub temperature: Option<f32>,
    #[arg(long)]
    pub top_k: Option<i32>,
    #[arg(long)]
    pub top_p: Option<f32>,
    #[arg(long)]
    pub timeout_secs: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub model_path: PathBuf,
    pub profiles: Vec<WorkloadProfile>,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub n_threads: Option<usize>,
    pub gpu_layers: u32,
    pub max_tokens: usize,
    pub cooldown_secs: f64,
    pub output: Option<PathBuf>,
    pub jsonl_output: Option<PathBuf>,
    pub seed: u32,
    pub sampler: SamplerKind,
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub timeout_secs: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FileConfig {
    pub model_path: Option<PathBuf>,
    pub prompt: Option<String>,
    pub prompts: Option<Vec<String>>,
    pub iterations: Option<usize>,
    pub warmup_iterations: Option<usize>,
    pub n_threads: Option<usize>,
    pub gpu_layers: Option<u32>,
    pub max_tokens: Option<usize>,
    pub cooldown_secs: Option<f64>,
    pub output: Option<PathBuf>,
    pub jsonl_output: Option<PathBuf>,
    pub seed: Option<u32>,
    pub sampler: Option<SamplerKind>,
    pub temperature: Option<f32>,
    pub top_k: Option<i32>,
    pub top_p: Option<f32>,
    pub timeout_secs: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SamplerKind {
    Greedy,
    Stochastic,
}

impl Default for SamplerKind {
    fn default() -> Self {
        Self::Greedy
    }
}

impl std::str::FromStr for SamplerKind {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "greedy" => Ok(Self::Greedy),
            "stochastic" | "dist" => Ok(Self::Stochastic),
            _ => Err("sampler must be one of: greedy, stochastic".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkloadProfile {
    pub name: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ThermalSample {
    pub timestamp_ms: u128,
    pub celsius: Option<f32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct IterationReport {
    pub profile: String,
    pub iteration: usize,
    pub run_status: String,
    pub error: Option<String>,
    pub generated_tokens: usize,
    pub ttft_us: u128,
    pub prompt_decode_us: u128,
    pub first_token_decode_us: u128,
    pub tps_mean: f64,
    pub tps_median: f64,
    pub itl_variance_us2: f64,
    pub itl_p99_us: f64,
    pub itl_samples_us: Vec<u128>,
    pub memory_before_kib: u64,
    pub memory_peak_kib: u64,
    pub memory_after_kib: u64,
    pub avg_cpu_usage_percent: f32,
    pub peak_cpu_usage_percent: f32,
    pub thermal_samples: Vec<ThermalSample>,
    pub token_flow: Vec<TokenFlowPoint>,
    pub timed_out: bool,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkReport {
    pub generated_at_unix_secs: u64,
    pub model_path: String,
    pub profile_count: usize,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub n_threads: usize,
    pub gpu_layers: u32,
    pub max_tokens: usize,
    pub cooldown_secs: f64,
    pub cpu_physical_cores: usize,
    pub cpu_logical_cores: usize,
    pub core_pinned: Option<usize>,
    pub mlock_requested: bool,
    pub mlock_supported: bool,
    pub sampler: SamplerKind,
    pub seed: u32,
    pub temperature: f32,
    pub top_k: i32,
    pub top_p: f32,
    pub timeout_secs: Option<f64>,
    pub runtime_fingerprint: RuntimeFingerprint,
    pub summary: BenchmarkSummary,
    pub profiles: Vec<ProfileReport>,
    pub runs: Vec<IterationReport>,
    pub jsonl_output: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MetricSummary {
    pub p50: f64,
    pub p95: f64,
    pub max: f64,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkSummary {
    pub measured_runs: usize,
    pub failed_runs: usize,
    pub timed_out_runs: usize,
    pub generated_tokens_total: usize,
    pub ttft_us: MetricSummary,
    pub tps: MetricSummary,
    pub itl_us: MetricSummary,
    pub itl_percentiles_us: ItlPercentiles,
    pub drift: DriftSummary,
}

#[derive(Debug, Serialize, Clone)]
pub struct TokenFlowPoint {
    pub token_index: usize,
    pub decode_us: u128,
    pub since_generation_start_us: u128,
    pub cumulative_tps: f64,
}

#[derive(Debug, Serialize)]
pub struct ItlPercentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

#[derive(Debug, Serialize)]
pub struct DriftSummary {
    pub tps_early_late_delta: f64,
    pub itl_early_late_delta_us: f64,
}

#[derive(Debug, Serialize)]
pub struct ProfileReport {
    pub name: String,
    pub prompt_chars: usize,
    pub summary: BenchmarkSummary,
}

#[derive(Debug, Serialize)]
pub struct RuntimeFingerprint {
    pub model_sha256: String,
    pub crate_version: String,
}
