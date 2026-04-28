use crate::types::{BenchmarkConfig, CliArgs, FileConfig, SamplerKind, WorkloadProfile};
use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

fn load_file_config(path: &Path) -> Result<FileConfig> {
    if !path.exists() {
        return Ok(FileConfig::default());
    }
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config file: {}", path.display()))?;
    let parsed: FileConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse TOML config: {}", path.display()))?;
    Ok(parsed)
}

fn discover_default_config_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("llm-benchmark.toml"),
        PathBuf::from("tools/llm-benchmark/llm-benchmark.toml"),
        PathBuf::from("fluidity-bench.toml"),
        PathBuf::from("tools/llm-benchmark/fluidity-bench.toml"),
        PathBuf::from("../llm-benchmark.toml"),
        PathBuf::from("../tools/llm-benchmark/llm-benchmark.toml"),
        PathBuf::from("../fluidity-bench.toml"),
        PathBuf::from("../tools/llm-benchmark/fluidity-bench.toml"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

fn discover_model_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("models"),
        PathBuf::from("../models"),
        PathBuf::from("../../models"),
    ];

    for dir in candidates {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut ggufs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .map(|ext| ext.to_string_lossy().to_ascii_lowercase() == "gguf")
                        .unwrap_or(false)
            })
            .collect();
        ggufs.sort();
        if let Some(path) = ggufs.into_iter().next() {
            return Some(path);
        }
    }
    None
}

fn resolve_path(base_dir: &Path, value: PathBuf) -> PathBuf {
    if value.is_absolute() {
        value
    } else {
        base_dir.join(value)
    }
}

fn prompts_from_inputs(
    cli_prompts: Vec<String>,
    file_cfg: &FileConfig,
) -> Result<Vec<WorkloadProfile>> {
    let prompts = if !cli_prompts.is_empty() {
        cli_prompts
    } else if let Some(prompts) = &file_cfg.prompts {
        prompts.clone()
    } else if let Some(prompt) = &file_cfg.prompt {
        vec![prompt.clone()]
    } else {
        Vec::new()
    };
    if prompts.is_empty() {
        anyhow::bail!("prompt is required (flag --prompt, prompts in config, or prompt in config)");
    }
    Ok(prompts
        .into_iter()
        .enumerate()
        .map(|(i, prompt)| WorkloadProfile {
            name: format!("profile_{}", i + 1),
            prompt,
        })
        .collect())
}

pub fn build_effective_config(args: CliArgs) -> Result<BenchmarkConfig> {
    let cfg_path = args
        .config
        .or_else(discover_default_config_path)
        .unwrap_or_else(|| PathBuf::from("llm-benchmark.toml"));
    let file_cfg = load_file_config(&cfg_path)?;
    let cfg_base = cfg_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));

    let model_path = args
        .model_path
        .map(|p| resolve_path(&PathBuf::from("."), p))
        .or_else(|| {
            file_cfg
                .model_path
                .clone()
                .map(|p| resolve_path(&cfg_base, p))
        })
        .or_else(discover_model_path)
        .context(
            "model_path is required (flag --model-path, config file, or a .gguf in models/)",
        )?;
    let profiles = prompts_from_inputs(args.prompt, &file_cfg)?;
    let iterations = args.iterations.or(file_cfg.iterations).unwrap_or(3);
    let warmup_iterations = args
        .warmup_iterations
        .or(file_cfg.warmup_iterations)
        .unwrap_or(1);
    let max_tokens = args.max_tokens.or(file_cfg.max_tokens).unwrap_or(128);
    let cooldown_secs = args.cooldown_secs.or(file_cfg.cooldown_secs).unwrap_or(2.0);
    let n_threads = args.n_threads.or(file_cfg.n_threads);
    let gpu_layers = args.gpu_layers.or(file_cfg.gpu_layers).unwrap_or(0);
    let output = args
        .output
        .map(|p| resolve_path(&PathBuf::from("."), p))
        .or_else(|| file_cfg.output.map(|p| resolve_path(&cfg_base, p)));
    let jsonl_output = args
        .jsonl_output
        .map(|p| resolve_path(&PathBuf::from("."), p))
        .or_else(|| {
            file_cfg
                .jsonl_output
                .clone()
                .map(|p| resolve_path(&cfg_base, p))
        });
    let seed = args.seed.or(file_cfg.seed).unwrap_or(1234);
    let sampler = if args.sampler == SamplerKind::Greedy {
        file_cfg.sampler.clone().unwrap_or(args.sampler)
    } else {
        args.sampler
    };
    let temperature = args.temperature.or(file_cfg.temperature).unwrap_or(0.8);
    let top_k = args.top_k.or(file_cfg.top_k).unwrap_or(40);
    let top_p = args.top_p.or(file_cfg.top_p).unwrap_or(0.95);
    let timeout_secs = args.timeout_secs.or(file_cfg.timeout_secs);

    Ok(BenchmarkConfig {
        model_path,
        profiles,
        iterations,
        warmup_iterations,
        n_threads,
        gpu_layers,
        max_tokens,
        cooldown_secs,
        output,
        jsonl_output,
        seed,
        sampler,
        temperature,
        top_k,
        top_p,
        timeout_secs,
    })
}
