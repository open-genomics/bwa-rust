use std::cmp::Ordering;

const MAX_SAMPLES: usize = 100_000;
const UPDATE_INTERVAL: usize = 1_000;

#[derive(Debug, Clone)]
pub struct InsertSizeStats {
    pub samples: Vec<i32>,
    pub median: f64,
    pub mad: f64,
    pub max_insert: i32,
    pub sample_count: usize,
}

impl Default for InsertSizeStats {
    fn default() -> Self {
        Self::new(500)
    }
}

impl InsertSizeStats {
    pub fn new(initial_max: i32) -> Self {
        Self {
            samples: Vec::with_capacity(MAX_SAMPLES),
            median: initial_max as f64 / 2.0,
            mad: initial_max as f64 / 6.0,
            max_insert: initial_max,
            sample_count: 0,
        }
    }

    pub fn add_sample(&mut self, insert_size: i32) {
        self.sample_count += 1;

        if self.samples.len() < MAX_SAMPLES {
            self.samples.push(insert_size);
        }

        if self.sample_count % UPDATE_INTERVAL == 0 && self.sample_count > 0 {
            self.update_stats();
        }
    }

    fn update_stats(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        let mut sorted = self.samples.clone();
        sorted.sort_unstable();

        let mid = sorted.len() / 2;
        self.median = if sorted.len() % 2 == 0 {
            (sorted[mid - 1] as f64 + sorted[mid] as f64) / 2.0
        } else {
            sorted[mid] as f64
        };

        let deviations: Vec<f64> = sorted.iter().map(|&x| (x as f64 - self.median).abs()).collect();

        let mut sorted_devs = deviations;
        sorted_devs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        self.mad = if sorted_devs.len() % 2 == 0 {
            (sorted_devs[mid - 1] + sorted_devs[mid]) / 2.0
        } else {
            sorted_devs[mid]
        };

        self.max_insert = ((self.median + 3.0 * self.mad) as i32).max(100);
    }

    pub fn is_valid_insert(&self, insert_size: i32) -> bool {
        if self.sample_count < 100 {
            return insert_size > 0 && insert_size <= self.max_insert;
        }

        let diff = (insert_size as f64 - self.median).abs();
        diff <= 3.0 * self.mad
    }

    pub fn insert_size_deviation_penalty(&self, insert_size: i32) -> i32 {
        if self.sample_count < 100 || self.mad < 1.0 {
            return 0;
        }

        let diff = (insert_size as f64 - self.median).abs();
        let tolerance = 3.0 * self.mad;

        if diff <= tolerance {
            0
        } else {
            ((diff - tolerance) / (2.0 * self.mad)) as i32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_size_stats_new() {
        let stats = InsertSizeStats::new(500);
        assert_eq!(stats.max_insert, 500);
        assert_eq!(stats.sample_count, 0);
        assert!(stats.samples.is_empty());
    }

    #[test]
    fn insert_size_stats_add_samples() {
        let mut stats = InsertSizeStats::new(500);

        for i in 100..200 {
            stats.add_sample(i);
        }

        assert_eq!(stats.sample_count, 100);
        assert_eq!(stats.samples.len(), 100);
    }

    #[test]
    fn insert_size_stats_update_median() {
        let mut stats = InsertSizeStats::new(500);

        for &val in &[100, 150, 200, 250, 300] {
            stats.add_sample(val);
        }
        stats.update_stats();

        assert_eq!(stats.median, 200.0);
    }

    #[test]
    fn insert_size_stats_update_mad() {
        let mut stats = InsertSizeStats::new(500);

        for &val in &[100, 150, 200, 250, 300] {
            stats.add_sample(val);
        }
        stats.update_stats();

        assert_eq!(stats.mad, 50.0);
    }

    #[test]
    fn insert_size_stats_valid_insert() {
        let mut stats = InsertSizeStats::new(500);

        for i in 100..200 {
            stats.add_sample(i);
        }

        assert!(stats.is_valid_insert(150));
        assert!(stats.is_valid_insert(50));
        assert!(!stats.is_valid_insert(1000));
    }

    #[test]
    fn insert_size_stats_deviation_penalty() {
        let mut stats = InsertSizeStats::new(500);

        for &val in &[100, 150, 200, 250, 300] {
            stats.add_sample(val);
        }
        stats.update_stats();

        stats.sample_count = 200;

        // median = 200, mad = 50, tolerance = 150
        // For insert_size = 400: diff = 200, penalty = (200 - 150) / 100 = 0.5 -> 0
        assert_eq!(stats.insert_size_deviation_penalty(200), 0);
        // For insert_size = 500: diff = 300, penalty = (300 - 150) / 100 = 1.5 -> 1
        assert!(stats.insert_size_deviation_penalty(500) > 0);
        assert_eq!(stats.insert_size_deviation_penalty(500), 1);
        // For insert_size = 600: diff = 400, penalty = (400 - 150) / 100 = 2.5 -> 2
        assert_eq!(stats.insert_size_deviation_penalty(600), 2);
    }

    #[test]
    fn insert_size_stats_max_samples_limit() {
        let mut stats = InsertSizeStats::new(500);

        for i in 0..200_000 {
            stats.add_sample(i);
        }

        assert_eq!(stats.samples.len(), MAX_SAMPLES);
        assert_eq!(stats.sample_count, 200_000);
    }

    #[test]
    fn insert_size_stats_default() {
        let stats = InsertSizeStats::default();
        assert_eq!(stats.max_insert, 500);
    }
}
