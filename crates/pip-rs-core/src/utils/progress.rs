/// Progress bar utilities for better UX
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;

/// Create a spinner for indeterminate progress
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Create a progress bar for known-length operations
pub fn progress_bar(len: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("█▓▒░")
    );
    pb.set_message(message.to_string());
    pb
}

/// Create a download progress bar with bytes
pub fn download_bar(total_bytes: u64) -> ProgressBar {
    let pb = ProgressBar::new(total_bytes);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.green/white}] {bytes}/{total_bytes} ({bytes_per_sec})")
            .unwrap()
            .progress_chars("━━╸")
    );
    pb
}

/// Create a multi-progress container for parallel operations
pub fn multi_progress() -> MultiProgress {
    MultiProgress::new()
}

/// Finish progress bar with success message
pub fn finish_success(pb: &ProgressBar, message: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("✓ {msg}")
            .unwrap()
    );
    pb.finish_with_message(message.to_string());
}

/// Finish progress bar with error message
pub fn finish_error(pb: &ProgressBar, message: &str) {
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("✗ {msg}")
            .unwrap()
    );
    pb.finish_with_message(message.to_string());
}

/// Check if progress bars should be hidden (quiet mode or non-TTY)
pub fn is_quiet() -> bool {
    std::env::var("PIP_QUIET").is_ok() || !atty_check()
}

/// Check if stdout is a TTY
fn atty_check() -> bool {
    // Simple check - in production would use atty crate
    std::env::var("TERM").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let pb = spinner("Testing...");
        pb.finish();
    }

    #[test]
    fn test_progress_bar_creation() {
        let pb = progress_bar(100, "Processing");
        pb.inc(50);
        pb.finish();
    }
}
