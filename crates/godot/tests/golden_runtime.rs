use std::time::{Duration, Instant};

use uncanny_core::{
    behavior::BehaviorMode,
    fusion::FusionEngine,
    perception::aggregator::PerceptionAggregator,
    runtime::Runtime,
    types::PerceptionPacket,
};
use uncanny_godot::{
    recording::{EyeCommand, RecordingEye, RecordingVoice},
    stub::sources::FixtureSource,
};

fn load_fixture(name: &str) -> Vec<PerceptionPacket> {
    let path = format!("{}/../core/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {path}: {e}"));
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("bad JSON in {path}: {e}")))
        .collect()
}

fn run_fixture(
    packets: &[PerceptionPacket],
) -> (Vec<Vec<EyeCommand>>, Option<BehaviorMode>) {
    assert!(!packets.is_empty(), "fixture must not be empty");

    let epoch = Instant::now();
    let source = FixtureSource::new(packets[0]);
    let eye = RecordingEye::new();
    let eye_log = eye.log();
    let aggregator = PerceptionAggregator::new(
        source.clone(),
        source.clone(),
        source.clone(),
        source.clone(),
        epoch,
    );
    let mut runtime = Runtime::new(aggregator, FusionEngine::new(), eye, RecordingVoice::new());

    let mut batches: Vec<Vec<EyeCommand>> = Vec::new();
    for (i, packet) in packets.iter().enumerate() {
        let now = epoch + Duration::from_millis(50 * i as u64);
        source.set(*packet, now);
        runtime.tick(now);
        batches.push(eye_log.lock().unwrap().drain(..).collect());
    }

    (batches, runtime.last_mode())
}

#[test]
fn joy_trace_yields_mimicry_positive() {
    let packets = load_fixture("joy.jsonl");
    let (batches, last_mode) = run_fixture(&packets);

    assert_eq!(last_mode, Some(BehaviorMode::MimicryPositive));

    let last_batch = batches.last().unwrap();
    assert_eq!(
        last_batch,
        &[EyeCommand::LookAt(0.0, 0.0), EyeCommand::Blink],
        "expected mimicry-positive commands on final tick"
    );
}

#[test]
fn sarcasm_trace_yields_analytical_stare() {
    let packets = load_fixture("sarcasm.jsonl");
    let (batches, last_mode) = run_fixture(&packets);

    assert_eq!(last_mode, Some(BehaviorMode::AnalyticalStare));

    let last_batch = batches.last().unwrap();
    assert_eq!(
        last_batch,
        &[EyeCommand::LookAt(0.0, 0.0)],
        "expected analytical-stare command on final tick"
    );
}

#[test]
fn search_trace_yields_saccade() {
    let packets = load_fixture("search.jsonl");
    let (batches, last_mode) = run_fixture(&packets);

    assert_eq!(last_mode, Some(BehaviorMode::Search));

    let last_batch = batches.last().unwrap();
    assert_eq!(
        last_batch,
        &[EyeCommand::Saccade],
        "expected search saccade on final tick"
    );
}
