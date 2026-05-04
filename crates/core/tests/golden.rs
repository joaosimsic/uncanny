use std::time::{Duration, Instant};

use uncanny_core::{
    behavior::{classify, BehaviorMode},
    fusion::FusionEngine,
    types::PerceptionPacket,
};

fn load_fixture(name: &str) -> Vec<PerceptionPacket> {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"));
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad JSON in {path}: {e}")))
        .collect()
}

#[test]
fn sarcasm_trace_yields_analytical_stare() {
    let packets = load_fixture("sarcasm.jsonl");
    assert!(!packets.is_empty(), "fixture must not be empty");

    let mut engine = FusionEngine::new();
    let epoch = Instant::now();
    let mut last_mode = BehaviorMode::Idle;

    for (i, packet) in packets.iter().enumerate() {
        let now = epoch + Duration::from_millis(50 * i as u64);
        let ctx = engine.tick(packet, now);
        last_mode = classify(&ctx);
    }

    assert_eq!(
        last_mode,
        BehaviorMode::AnalyticalStare,
        "expected AnalyticalStare after sarcasm trace, got {last_mode:?}"
    );
}

#[test]
fn rapid_flip_does_not_commit() {
    let mut engine = FusionEngine::new();
    let epoch = Instant::now();

    let active_packet = PerceptionPacket {
        acoustic_valence: 0.5,
        acoustic_arousal: 0.4,
        visual_valence: 0.3,
        user_engagement: 0.8,
        semantic_valence: None,
        semantic_age_ms: u32::MAX,
        doa_bearing: None,
        user_id: None,
        timestamp_secs: 0.0,
    };
    let inactive_packet = PerceptionPacket {
        user_engagement: 0.0,
        acoustic_valence: 0.0,
        acoustic_arousal: 0.0,
        visual_valence: 0.0,
        ..active_packet
    };

    for i in 0u64..8 {
        engine.tick(&active_packet, epoch + Duration::from_millis(50 * i));
    }
    let baseline_ctx = engine.tick(&active_packet, epoch + Duration::from_millis(400));
    assert_ne!(classify(&baseline_ctx), BehaviorMode::Idle);

    let flip_start = epoch + Duration::from_millis(450);
    for i in 0u64..4 {
        let p = if i % 2 == 0 { &inactive_packet } else { &active_packet };
        let ctx = engine.tick(p, flip_start + Duration::from_millis(50 * i));
        assert_ne!(classify(&ctx), BehaviorMode::Idle, "hysteresis should hold at flip step {i}");
    }
}

#[test]
fn inactive_decays_toward_neutral() {
    let mut engine = FusionEngine::new();
    let epoch = Instant::now();

    let active_packet = PerceptionPacket {
        acoustic_valence: 0.8,
        acoustic_arousal: 0.9,
        visual_valence: 0.6,
        user_engagement: 0.9,
        semantic_valence: None,
        semantic_age_ms: u32::MAX,
        doa_bearing: None,
        user_id: None,
        timestamp_secs: 0.0,
    };
    let inactive_packet = PerceptionPacket {
        user_engagement: 0.0,
        acoustic_valence: 0.0,
        acoustic_arousal: 0.0,
        visual_valence: 0.0,
        ..active_packet
    };

    for i in 0u64..8 {
        engine.tick(&active_packet, epoch + Duration::from_millis(50 * i));
    }

    let base = epoch + Duration::from_millis(400);
    let mut ctx = engine.tick(&inactive_packet, base);
    for i in 1u64..=200 {
        ctx = engine.tick(&inactive_packet, base + Duration::from_millis(50 * i));
    }

    assert!(
        ctx.fused_arousal.abs() < 0.1,
        "arousal should decay toward 0, got {}",
        ctx.fused_arousal
    );
}
