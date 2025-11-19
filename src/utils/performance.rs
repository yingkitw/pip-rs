/// Performance monitoring and optimization utilities
use std::time::{Instant, Duration};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Performance metrics tracker
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Operation name
    pub name: String,
    /// Duration of operation
    pub duration: Duration,
    /// Memory used (bytes)
    pub memory_used: u64,
    /// Timestamp
    pub timestamp: Instant,
}

/// Global performance tracker
pub struct PerformanceTracker {
    metrics: Arc<Mutex<Vec<PerformanceMetrics>>>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Record a metric
    pub fn record(&self, name: String, duration: Duration, memory_used: u64) {
        let metric = PerformanceMetrics {
            name,
            duration,
            memory_used,
            timestamp: Instant::now(),
        };

        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.push(metric);
        }
    }

    /// Get all metrics
    pub fn get_metrics(&self) -> Vec<PerformanceMetrics> {
        self.metrics
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    /// Get metrics summary
    pub fn get_summary(&self) -> HashMap<String, MetricsSummary> {
        let metrics = self.get_metrics();
        let mut summary: HashMap<String, MetricsSummary> = HashMap::new();

        for metric in metrics {
            let entry = summary
                .entry(metric.name.clone())
                .or_insert_with(|| MetricsSummary {
                    name: metric.name.clone(),
                    count: 0,
                    total_duration: Duration::ZERO,
                    avg_duration: Duration::ZERO,
                    min_duration: Duration::MAX,
                    max_duration: Duration::ZERO,
                    total_memory: 0,
                    avg_memory: 0,
                });

            entry.count += 1;
            entry.total_duration += metric.duration;
            entry.total_memory += metric.memory_used;

            if metric.duration < entry.min_duration {
                entry.min_duration = metric.duration;
            }
            if metric.duration > entry.max_duration {
                entry.max_duration = metric.duration;
            }
        }

        // Calculate averages
        for summary in summary.values_mut() {
            if summary.count > 0 {
                summary.avg_duration = Duration::from_millis(
                    summary.total_duration.as_millis() as u64 / summary.count as u64,
                );
                summary.avg_memory = summary.total_memory / summary.count as u64;
            }
        }

        summary
    }

    /// Print performance report
    pub fn print_report(&self) {
        let summary = self.get_summary();

        if summary.is_empty() {
            println!("No performance metrics recorded");
            return;
        }

        println!("\n=== Performance Report ===");
        println!("{:<30} {:>10} {:>12} {:>12} {:>12} {:>12}",
            "Operation", "Count", "Avg (ms)", "Min (ms)", "Max (ms)", "Avg Mem (KB)");
        println!("{}", "-".repeat(100));

        for (_, metrics) in summary.iter() {
            println!("{:<30} {:>10} {:>12.2} {:>12.2} {:>12.2} {:>12}",
                metrics.name,
                metrics.count,
                metrics.avg_duration.as_secs_f64() * 1000.0,
                metrics.min_duration.as_secs_f64() * 1000.0,
                metrics.max_duration.as_secs_f64() * 1000.0,
                metrics.avg_memory / 1024,
            );
        }
    }

    /// Clear all metrics
    pub fn clear(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.clear();
        }
    }
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of metrics for an operation
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub name: String,
    pub count: u64,
    pub total_duration: Duration,
    pub avg_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub total_memory: u64,
    pub avg_memory: u64,
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    name: String,
}

impl Timer {
    /// Create a new timer
    pub fn new(name: &str) -> Self {
        Self {
            start: Instant::now(),
            name: name.to_string(),
        }
    }

    /// Get elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get elapsed milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }

    /// Print elapsed time
    pub fn print_elapsed(&self) {
        eprintln!("{}: {:.2}ms", self.name, self.elapsed_ms());
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        // Timer is dropped, operation complete
    }
}

/// Estimate memory usage
pub fn estimate_memory_usage() -> u64 {
    // This is a rough estimate
    // In production, use proper memory profiling
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(size_str) = line.split_whitespace().nth(1) {
                        if let Ok(size) = size_str.parse::<u64>() {
                            return size * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    // Fallback: return 0 if we can't determine
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_tracker_creation() {
        let tracker = PerformanceTracker::new();
        assert_eq!(tracker.get_metrics().len(), 0);
    }

    #[test]
    fn test_record_metric() {
        let tracker = PerformanceTracker::new();
        tracker.record(
            "test_op".to_string(),
            Duration::from_millis(100),
            1024,
        );
        assert_eq!(tracker.get_metrics().len(), 1);
    }

    #[test]
    fn test_metrics_summary() {
        let tracker = PerformanceTracker::new();
        tracker.record(
            "op1".to_string(),
            Duration::from_millis(100),
            1024,
        );
        tracker.record(
            "op1".to_string(),
            Duration::from_millis(200),
            2048,
        );

        let summary = tracker.get_summary();
        assert_eq!(summary.len(), 1);
        assert_eq!(summary["op1"].count, 2);
    }

    #[test]
    fn test_timer_creation() {
        let timer = Timer::new("test");
        let elapsed = timer.elapsed();
        assert!(elapsed.as_millis() >= 0);
    }

    #[test]
    fn test_timer_elapsed_ms() {
        let timer = Timer::new("test");
        std::thread::sleep(Duration::from_millis(10));
        let elapsed_ms = timer.elapsed_ms();
        assert!(elapsed_ms >= 10.0);
    }
}
