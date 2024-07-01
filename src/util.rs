use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Transform score for easier score comprehension and usage
/// Scaled 4.0 - 4.75 range to 0.0 - 5.0
pub fn transform_score(score: f32) -> f32 {
    const SCALE_VALUE: f32 = 5.0 / (4.75 - 4.0);

    if score < 4.1 {
        1.0f32
    } else {
        (score - 4.1) * SCALE_VALUE
    }
}
pub fn weighted_search(dt: &[(u32, f32)], target_quality: f32) {
    let mut probes = dt.to_owned();
    probes.sort_by(|a, b| {
        (a.1 - target_quality)
            .abs()
            .partial_cmp(&(b.1 - target_quality).abs())
            .unwrap()
    });
    let v1 = probes[0];
    let v2 = probes[1];
    debug!("{:#?} {:#?}", v1, v2)
}
pub fn get_audio_time(input: &Path) -> Duration {
    const MILLIS_PER_MINUTE: u64 = 60_000;
    const MILLIS_PER_HOUR: u64 = MILLIS_PER_MINUTE * 60;

    // FIXME: Don't allow to segment be less that 5 sec
    let mut cmd = Command::new("ffprobe");
    cmd.arg("-i");
    cmd.arg(input);

    cmd.stderr(Stdio::piped());

    let output = match cmd.output() {
        Ok(output) => match String::from_utf8(output.stderr) {
            Ok(output) => output,
            Err(e) => {
                eprintln!("Failed to convert output to UTF-8: {:?}", e);
                return Duration::from_secs(0);
            }
        },
        Err(e) => {
            eprintln!("Failed to execute ffprobe command: {:?}", e);
            return Duration::from_secs(0);
        }
    };

    const START: &str = "Duration: ";
    const END: &str = ", start";

    let duration_str = match (output.find(START), output.find(END)) {
        (Some(start_index), Some(end_index)) => {
            &output[start_index + START.len()..end_index]
        }
        _ => {
            eprintln!("Failed to find duration in ffprobe output");
            return Duration::from_secs(0);
        }
    };

    let mut iter = duration_str.split(':');
    let hours: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minutes: u64 = iter.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let millis: u64 = (1000.0 * iter.next().and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0)) as u64;

    Duration::from_millis(millis + minutes * MILLIS_PER_MINUTE + hours * MILLIS_PER_HOUR)
}
