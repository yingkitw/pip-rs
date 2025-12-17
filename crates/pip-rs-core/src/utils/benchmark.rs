/// Performance benchmarking utilities
use std::time::{Instant, Duration};
use std::collections::HashMap;

/// Benchmark result
#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub iterations: u32,
}

impl BenchmarkResult {
    pub fn average_ms(&self) -> f64 {
        self.duration.as_secs_f64() * 1000.0 / self.iterations as f64
    }

    pub fn total_ms(&self) -> f64 {
        self.duration.as_secs_f64() * 1000.0
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    results: HashMap<String, BenchmarkResult>,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
        }
    }

    /// Run a benchmark
    pub fn benchmark<F>(&mut self, name: &str, iterations: u32, mut f: F) -> BenchmarkResult
    where
        F: FnMut(),
    {
        let start = Instant::now();
        for _ in 0..iterations {
            f();
        }
        let duration = start.elapsed();

        let result = BenchmarkResult {
            name: name.to_string(),
            duration,
            iterations,
        };

        self.results.insert(name.to_string(), result.clone());
        result
    }

    /// Get all results
    pub fn results(&self) -> Vec<BenchmarkResult> {
        self.results.values().cloned().collect()
    }

    /// Print summary
    pub fn print_summary(&self) {
        if self.results.is_empty() {
            println!("No benchmarks run");
            return;
        }

        println!("\n=== Benchmark Results ===");
        println!("{:<30} {:>15} {:>15} {:>15}", "Name", "Total (ms)", "Avg (ms)", "Iterations");
        println!("{}", "-".repeat(75));

        for result in self.results() {
            println!(
                "{:<30} {:>15.3} {:>15.3} {:>15}",
                result.name,
                result.total_ms(),
                result.average_ms(),
                result.iterations
            );
        }
    }
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timing macro
#[macro_export]
macro_rules! time_it {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed();
        eprintln!("{}: {:.3}ms", $name, elapsed.as_secs_f64() * 1000.0);
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            duration: Duration::from_millis(100),
            iterations: 10,
        };

        assert_eq!(result.average_ms(), 10.0);
        assert_eq!(result.total_ms(), 100.0);
    }

    #[test]
    fn test_benchmark_runner() {
        let mut runner = BenchmarkRunner::new();

        let result = runner.benchmark("test", 5, || {
            std::thread::sleep(Duration::from_millis(1));
        });

        assert_eq!(result.iterations, 5);
        assert!(result.duration.as_millis() >= 5);
    }

    #[test]
    fn test_benchmark_runner_multiple() {
        let mut runner = BenchmarkRunner::new();

        runner.benchmark("test1", 3, || {
            std::thread::sleep(Duration::from_millis(1));
        });

        runner.benchmark("test2", 2, || {
            std::thread::sleep(Duration::from_millis(1));
        });

        assert_eq!(runner.results().len(), 2);
    }
}
