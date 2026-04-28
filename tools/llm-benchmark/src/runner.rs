use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use llama_cpp_4::{
    context::params::LlamaContextParams,
    llama_backend::LlamaBackend,
    llama_batch::LlamaBatch,
    model::{AddBos, LlamaModel, Special},
    sampling::LlamaSampler,
};
use std::{
    num::NonZeroU32,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use sysinfo::Pid;

use crate::{
    metrics::{percentile, variance},
    system_monitor::{monitor_hardware, process_memory_kib},
    types::{BenchmarkConfig, IterationReport, SamplerKind, TokenFlowPoint, WorkloadProfile},
};

mod timer {
    use std::time::Instant;

    pub struct MicroTimer {
        start: Instant,
    }

    impl MicroTimer {
        pub fn start_new() -> Self {
            Self {
                start: Instant::now(),
            }
        }

        pub fn elapsed_us(&self) -> u128 {
            self.start.elapsed().as_micros()
        }

        pub fn restart(&mut self) {
            self.start = Instant::now();
        }
    }
}

fn sampler_from_config(config: &BenchmarkConfig) -> LlamaSampler {
    match config.sampler {
        SamplerKind::Greedy => LlamaSampler::chain_simple([LlamaSampler::greedy()]),
        SamplerKind::Stochastic => LlamaSampler::chain_simple([
            LlamaSampler::top_k(config.top_k),
            LlamaSampler::top_p(config.top_p, 1),
            LlamaSampler::temp(config.temperature),
            LlamaSampler::dist(config.seed),
        ]),
    }
}

pub fn run_iteration(
    profile: &WorkloadProfile,
    iteration: usize,
    config: &BenchmarkConfig,
    model: &LlamaModel,
    backend: &LlamaBackend,
    n_threads: usize,
    pid: Pid,
) -> Result<IterationReport> {
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(2048))
        .with_n_batch(512)
        .with_n_threads(n_threads as i32);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .context("failed to create llama context")?;

    let prompt_tokens = model
        .str_to_token(&profile.prompt, AddBos::Always)
        .context("failed to tokenize prompt")?;
    let prompt_len = prompt_tokens.len();
    if prompt_len == 0 {
        anyhow::bail!("prompt tokenization produced no tokens");
    }

    let pb = ProgressBar::new(config.max_tokens as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{msg} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {per_sec}",
        )?
        .progress_chars("#>-"),
    );
    pb.set_message(format!("iter {}", iteration + 1));

    let memory_before_kib = process_memory_kib(pid);
    let started_at = Instant::now();
    let run_flag = Arc::new(AtomicBool::new(true));
    let monitor_handle = monitor_hardware(
        Arc::clone(&run_flag),
        pid,
        Duration::from_millis(200),
        started_at,
    );

    let mut batch = LlamaBatch::new(2048, 1);
    for (i, &tok) in prompt_tokens.iter().enumerate() {
        batch.add(tok, i as i32, &[0], i == prompt_len - 1)?;
    }

    let mut timer = timer::MicroTimer::start_new();
    ctx.decode(&mut batch)
        .context("failed to decode prompt batch")?;
    let prompt_decode_us = timer.elapsed_us();

    let sampler = sampler_from_config(config);
    let mut itl_samples_us = Vec::with_capacity(config.max_tokens);
    let mut token_flow = Vec::with_capacity(config.max_tokens);
    let mut generated_tokens = 0usize;
    let generation_start = Instant::now();
    let mut pos = prompt_len as i32;
    let mut first_token_decode_us = 0u128;
    let mut timed_out = false;

    for _ in 0..config.max_tokens {
        if let Some(timeout_secs) = config.timeout_secs
            && generation_start.elapsed().as_secs_f64() > timeout_secs
        {
            timed_out = true;
            break;
        }
        let token = sampler.sample(&ctx, -1);
        if model.is_eog_token(token) {
            break;
        }

        let _ = model.token_to_bytes(token, Special::Plaintext)?;

        batch.clear();
        batch.add(token, pos, &[0], true)?;
        timer.restart();
        ctx.decode(&mut batch)
            .context("failed to decode generated token")?;
        let decode_us = timer.elapsed_us();
        if first_token_decode_us == 0 {
            first_token_decode_us = decode_us;
        }
        itl_samples_us.push(decode_us);
        pos += 1;
        generated_tokens += 1;

        let elapsed_s = generation_start.elapsed().as_secs_f64().max(1e-9);
        let live_tps = generated_tokens as f64 / elapsed_s;
        token_flow.push(TokenFlowPoint {
            token_index: generated_tokens,
            decode_us,
            since_generation_start_us: generation_start.elapsed().as_micros(),
            cumulative_tps: live_tps,
        });
        pb.set_message(format!(
            "iter {} | TPS {:.2} | ITL {}us",
            iteration + 1,
            live_tps,
            decode_us
        ));
        pb.inc(1);
    }
    pb.finish_and_clear();

    run_flag.store(false, Ordering::Relaxed);
    let hw_stats = monitor_handle
        .join()
        .map_err(|_| anyhow::anyhow!("hardware monitor thread panicked"))?;

    let memory_after_kib = process_memory_kib(pid);
    let memory_peak_kib = hw_stats.memory_peak_kib.max(memory_after_kib);

    let total_generation_secs = generation_start.elapsed().as_secs_f64().max(1e-9);
    let tps_mean = generated_tokens as f64 / total_generation_secs;

    let mut tps_samples: Vec<f64> = itl_samples_us
        .iter()
        .copied()
        .filter(|us| *us > 0)
        .map(|us| 1_000_000.0 / us as f64)
        .collect();
    tps_samples.sort_by(|a, b| a.total_cmp(b));
    let tps_median = percentile(&tps_samples, 0.5);

    let mut itl_f64: Vec<f64> = itl_samples_us.iter().map(|v| *v as f64).collect();
    itl_f64.sort_by(|a, b| a.total_cmp(b));
    let itl_p99_us = percentile(&itl_f64, 0.99);
    let itl_variance_us2 = variance(&itl_f64);

    let avg_cpu_usage_percent = if hw_stats.cpu_samples.is_empty() {
        0.0
    } else {
        hw_stats.cpu_samples.iter().sum::<f32>() / hw_stats.cpu_samples.len() as f32
    };
    let peak_cpu_usage_percent = hw_stats.cpu_samples.iter().copied().fold(0.0f32, f32::max);

    Ok(IterationReport {
        profile: profile.name.clone(),
        iteration: iteration + 1,
        run_status: "ok".to_string(),
        error: None,
        generated_tokens,
        ttft_us: prompt_decode_us + first_token_decode_us,
        prompt_decode_us,
        first_token_decode_us,
        tps_mean,
        tps_median,
        itl_variance_us2,
        itl_p99_us,
        itl_samples_us,
        memory_before_kib,
        memory_peak_kib,
        memory_after_kib,
        avg_cpu_usage_percent,
        peak_cpu_usage_percent,
        thermal_samples: hw_stats.thermal_samples,
        token_flow,
        timed_out,
    })
}
