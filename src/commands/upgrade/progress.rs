/// Progress indication with animation for package updates
use std::io::Write;

pub struct ProgressIndicator {
    total: usize,
    current: usize,
    spinner_frames: Vec<&'static str>,
    frame_index: usize,
}

impl ProgressIndicator {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            current: 0,
            spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            frame_index: 0,
        }
    }

    pub fn update(&mut self, package_name: &str, is_upgrading: bool) {
        self.current += 1;
        let percent = (self.current as f64 / self.total as f64 * 100.0) as u32;
        let bar = self.format_bar(percent);
        let spinner = self.spinner_frames[self.frame_index % self.spinner_frames.len()];
        self.frame_index += 1;

        let operation = if is_upgrading { "⬆" } else { "✓" };
        
        eprint!(
            "\r{} [{}] {:3}% | {}/{} | {} {}",
            spinner, bar, percent, self.current, self.total, operation, package_name
        );
        let _ = std::io::stderr().flush();
    }

    pub fn finish(&self) {
        let bar = self.format_bar(100);
        eprintln!("\r✓ [{}] 100% | {}/{} | Complete!", bar, self.total, self.total);
    }

    fn format_bar(&self, percent: u32) -> String {
        let width: usize = 30;
        let filled = (width as f64 * percent as f64 / 100.0) as usize;
        let empty = width.saturating_sub(filled);
        
        let filled_char = "█";
        let empty_char = "░";
        
        format!("{}{}",
            filled_char.repeat(filled),
            empty_char.repeat(empty)
        )
    }
}
