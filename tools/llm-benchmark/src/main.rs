use anyhow::{Context, Result};
use clap::Parser;
use llama_cpp_4::{
    llama_backend::LlamaBackend,
    model::{LlamaModel, params::LlamaModelParams},
};
use std::{collections::HashMap, fs, thread, time::Duration};
use sysinfo::Pid;

mod config;
mod metrics;
mod reporting;
mod runner;
mod system_monitor;
mod types;

use config::build_effective_config;
use metrics::compute_summary;
use reporting::{model_sha256, write_jsonl_flow};
use runner::run_iteration;
use types::{BenchmarkReport, CliArgs, IterationReport, ProfileReport, RuntimeFingerprint};

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let config = build_effective_config(args)?;
    if !config.model_path.exists() {
        anyhow::bail!("model path does not exist: {}", config.model_path.display());
    }

    let physical_cores = num_cpus::get_physical().max(1);
    let logical_cores = num_cpus::get();
    let n_threads = config.n_threads.unwrap_or(physical_cores);

    let pinned_core = None;

    let backend = LlamaBackend::init().context("failed to initialize llama backend")?;
    let mlock_supported = llama_cpp_4::mlock_supported();
    let model_params = LlamaModelParams::default()
        .with_use_mlock(true)
        .with_n_gpu_layers(config.gpu_layers);
    let model = LlamaModel::load_from_file(
        &backend,
        config.model_path.to_str().unwrap_or_default(),
        &model_params,
    )
    .context("failed to load model file")?;

    let pid = Pid::from_u32(std::process::id());
    let mut runs = Vec::with_capacity(config.iterations * config.profiles.len());
    let mut profile_runs: HashMap<String, Vec<IterationReport>> = HashMap::new();
    let total_runs = (config.warmup_iterations + config.iterations) * config.profiles.len();
    let mut run_ix = 0usize;

    for profile in &config.profiles {
        let profile_key = profile.name.clone();
        for i in 0..(config.warmup_iterations + config.iterations) {
            let report = match run_iteration(profile, i, &config, &model, &backend, n_threads, pid)
            {
                Ok(r) => r,
                Err(err) => IterationReport {
                    profile: profile_key.clone(),
                    iteration: i + 1,
                    run_status: "failed".to_string(),
                    error: Some(err.to_string()),
                    generated_tokens: 0,
                    ttft_us: 0,
                    prompt_decode_us: 0,
                    first_token_decode_us: 0,
                    tps_mean: 0.0,
                    tps_median: 0.0,
                    itl_variance_us2: 0.0,
                    itl_p99_us: 0.0,
                    itl_samples_us: Vec::new(),
                    memory_before_kib: 0,
                    memory_peak_kib: 0,
                    memory_after_kib: 0,
                    avg_cpu_usage_percent: 0.0,
                    peak_cpu_usage_percent: 0.0,
                    thermal_samples: Vec::new(),
                    token_flow: Vec::new(),
                    timed_out: false,
                },
            };
            if i >= config.warmup_iterations {
                runs.push(report.clone());
                profile_runs
                    .entry(profile_key.clone())
                    .or_default()
                    .push(report);
            }
            run_ix += 1;
            if run_ix < total_runs {
                thread::sleep(Duration::from_secs_f64(config.cooldown_secs));
            }
        }
    }

    let generated_at_unix_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let summary = compute_summary(&runs);
    let profiles: Vec<ProfileReport> = config
        .profiles
        .iter()
        .map(|profile| {
            let reports = profile_runs.get(&profile.name).cloned().unwrap_or_default();
            ProfileReport {
                name: profile.name.clone(),
                prompt_chars: profile.prompt.chars().count(),
                summary: compute_summary(&reports),
            }
        })
        .collect();
    let runtime_fingerprint = RuntimeFingerprint {
        model_sha256: model_sha256(&config.model_path)?,
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let final_report = BenchmarkReport {
        generated_at_unix_secs,
        model_path: config.model_path.display().to_string(),
        profile_count: config.profiles.len(),
        iterations: config.iterations,
        warmup_iterations: config.warmup_iterations,
        n_threads,
        gpu_layers: config.gpu_layers,
        max_tokens: config.max_tokens,
        cooldown_secs: config.cooldown_secs,
        cpu_physical_cores: physical_cores,
        cpu_logical_cores: logical_cores,
        core_pinned: pinned_core,
        mlock_requested: true,
        mlock_supported,
        sampler: config.sampler.clone(),
        seed: config.seed,
        temperature: config.temperature,
        top_k: config.top_k,
        top_p: config.top_p,
        timeout_secs: config.timeout_secs,
        runtime_fingerprint,
        summary,
        profiles,
        runs,
        jsonl_output: config
            .jsonl_output
            .as_ref()
            .map(|p| p.display().to_string()),
    };

    let json = serde_json::to_string_pretty(&final_report)?;
    if let Some(path) = &config.output {
        fs::write(path, &json)
            .with_context(|| format!("failed to write JSON report to {}", path.display()))?;
    }
    if let Some(path) = &config.jsonl_output {
        write_jsonl_flow(&final_report, path)?;
    }
    println!("{json}");

    Ok(())
}
