use std::collections::VecDeque;

pub const INTERVAL_WINDOW: usize = 256;
pub const MAX_INTERVAL_US: u32 = 250_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confidence {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Default)]
pub struct Metrics {
    events: u64,
    reports: u64,
    dropped: u64,
    intervals: VecDeque<u32>,
    last_report_us: Option<u64>,
}

impl Metrics {
    pub fn record_event(&mut self) {
        self.events = self.events.saturating_add(1);
    }

    pub fn record_report(&mut self, now_us: u64) {
        self.reports = self.reports.saturating_add(1);
        if let Some(previous) = self.last_report_us {
            let interval = now_us.saturating_sub(previous);
            if interval > 0 && interval <= u64::from(MAX_INTERVAL_US) {
                while self.intervals.len() >= INTERVAL_WINDOW {
                    self.intervals.pop_front();
                }
                self.intervals.push_back(interval as u32);
            }
        }
        self.last_report_us = Some(now_us);
    }

    pub fn record_dropped(&mut self, count: u64) {
        self.dropped = self.dropped.saturating_add(count);
    }

    pub fn hz_estimate(&self) -> Option<f32> {
        let median = self.median_interval_us()?;
        if median == 0 {
            None
        } else {
            Some(1_000_000.0 / median as f32)
        }
    }

    pub fn jitter_us(&self) -> Option<u32> {
        let sorted = self.sorted_intervals();
        if sorted.len() < 2 {
            return None;
        }
        Some(percentile(&sorted, 95).saturating_sub(percentile(&sorted, 50)))
    }

    pub fn median_interval_us(&self) -> Option<u32> {
        let sorted = self.sorted_intervals();
        if sorted.is_empty() {
            None
        } else {
            Some(percentile(&sorted, 50))
        }
    }

    pub fn confidence(&self) -> Confidence {
        match self.intervals.len() {
            0..=19 => Confidence::Low,
            20..=99 => Confidence::Medium,
            _ => Confidence::High,
        }
    }

    pub fn sample_count(&self) -> usize {
        self.intervals.len()
    }

    pub fn histogram(&self, bins: usize, max_us: u32) -> Vec<u32> {
        if bins == 0 {
            return Vec::new();
        }
        let width = max_us.max(1).div_ceil(bins as u32).max(1);
        let mut histogram = vec![0u32; bins];
        for interval in &self.intervals {
            let index = (*interval / width).min(bins as u32 - 1) as usize;
            histogram[index] = histogram[index].saturating_add(1);
        }
        histogram
    }

    pub fn events(&self) -> u64 {
        self.events
    }

    pub fn reports(&self) -> u64 {
        self.reports
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn sorted_intervals(&self) -> Vec<u32> {
        let mut sorted: Vec<u32> = self.intervals.iter().copied().collect();
        sorted.sort_unstable();
        sorted
    }
}

fn percentile(sorted: &[u32], percent: u32) -> u32 {
    let index = ((sorted.len() - 1) as f32 * (percent as f32 / 100.0)).round() as usize;
    sorted[index]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(metrics: &mut Metrics, step_us: u64, count: u64) {
        let mut now = metrics
            .last_report_us
            .map(|last| last + step_us)
            .unwrap_or(0);
        for _ in 0..count {
            metrics.record_report(now);
            now += step_us;
        }
    }

    #[test]
    fn estimates_1000_hz() {
        let mut metrics = Metrics::default();
        feed(&mut metrics, 1_000, 300);
        let hz = metrics.hz_estimate().unwrap();
        assert!((hz - 1_000.0).abs() < 0.01);
        assert_eq!(metrics.confidence(), Confidence::High);
    }

    #[test]
    fn estimates_125_hz() {
        let mut metrics = Metrics::default();
        feed(&mut metrics, 8_000, 300);
        let hz = metrics.hz_estimate().unwrap();
        assert!((hz - 125.0).abs() < 0.01);
    }

    #[test]
    fn ignores_invalid_intervals() {
        let mut metrics = Metrics::default();
        metrics.record_report(10_000);
        metrics.record_report(10_000);
        metrics.record_report(500_000);
        assert_eq!(metrics.sample_count(), 0);
    }

    #[test]
    fn computes_jitter() {
        let mut metrics = Metrics::default();
        metrics.record_report(0);
        for interval in [1_000, 1_000, 1_000, 2_000] {
            metrics.record_report(metrics.last_report_us.unwrap_or(0) + interval);
        }
        assert_eq!(metrics.jitter_us(), Some(1_000));
        assert_eq!(metrics.median_interval_us(), Some(1_000));
    }

    #[test]
    fn confidence_tracks_samples() {
        let mut metrics = Metrics::default();
        assert_eq!(metrics.confidence(), Confidence::Low);
        feed(&mut metrics, 1_000, 21);
        assert_eq!(metrics.confidence(), Confidence::Medium);
        feed(&mut metrics, 1_000, 100);
        assert_eq!(metrics.confidence(), Confidence::High);
    }

    #[test]
    fn builds_histogram() {
        let mut metrics = Metrics::default();
        feed(&mut metrics, 1_000, 300);
        let histogram = metrics.histogram(12, 24_000);
        assert_eq!(histogram.iter().sum::<u32>(), metrics.sample_count() as u32);
        assert!(histogram[0] > 0);
    }

    #[test]
    fn counts_events_reports_and_drops() {
        let mut metrics = Metrics::default();
        metrics.record_event();
        metrics.record_event();
        metrics.record_report(1_000);
        metrics.record_dropped(5);
        assert_eq!(metrics.events(), 2);
        assert_eq!(metrics.reports(), 1);
        assert_eq!(metrics.dropped(), 5);
    }

    #[test]
    fn reset_clears_all() {
        let mut metrics = Metrics::default();
        feed(&mut metrics, 1_000, 50);
        metrics.record_dropped(1);
        metrics.reset();
        assert!(metrics.hz_estimate().is_none());
        assert_eq!(metrics.events(), 0);
        assert_eq!(metrics.dropped(), 0);
    }
}
