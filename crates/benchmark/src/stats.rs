use std::time::Duration;

pub struct BenchmarkResult {
    pub name: String,
    pub samples: Vec<Duration>,
}

impl BenchmarkResult {
    pub fn print(&self) {
        let min = self.samples.iter().min().unwrap();
        let max = self.samples.iter().max().unwrap();

        let avg = self.samples
            .iter()
            .map(|d| d.as_secs_f64())
            .sum::<f64>()
            / self.samples.len() as f64;

        println!();
        println!("========== {} ==========", self.name);
        println!("Iterations : {}", self.samples.len());
        println!("Average    : {:.3} ms", avg * 1000.0);
        println!("Min        : {:.3} ms", min.as_secs_f64() * 1000.0);
        println!("Max        : {:.3} ms", max.as_secs_f64() * 1000.0);
    }
}