use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::Telemetry;

#[derive(Debug, Clone)]
pub struct TelemetryHistory {
    buffer: VecDeque<TelemetrySnapshot>,
    max_duration: Duration,
    max_samples: usize,
}

#[derive(Debug, Clone)]
pub struct TelemetrySnapshot {
    pub timestamp: Instant,
    pub telemetry: Telemetry,
}

impl TelemetryHistory {
    pub fn new(max_duration: Duration, max_samples: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_samples),
            max_duration,
            max_samples,
        }
    }

    pub fn push(&mut self, telemetry: Telemetry) {
        let now = Instant::now();

        let cutoff_time = now - self.max_duration;
        self.buffer
            .retain(|snapshot| snapshot.timestamp > cutoff_time);

        if self.buffer.len() >= self.max_samples {
            self.buffer.pop_front();
        }

        self.buffer.push_back(TelemetrySnapshot {
            timestamp: now,
            telemetry,
        });
    }

    pub fn get_samples(&self) -> &VecDeque<TelemetrySnapshot> {
        &self.buffer
    }

    pub fn get_series<T, F>(&self, getter: F) -> Vec<T>
    where
        F: Fn(&Telemetry) -> T,
    {
        self.buffer.iter().map(|s| getter(&s.telemetry)).collect()
    }

    pub fn get_voltage_series(&self) -> Vec<f32> {
        self.get_series(|t| t.voltage)
    }

    pub fn get_current_series(&self) -> Vec<f32> {
        self.get_series(|t| t.current)
    }

    pub fn get_power_series(&self) -> Vec<f32> {
        self.get_series(|t| t.power)
    }

    pub fn get_temperature_series(&self) -> Vec<f32> {
        self.get_series(|t| t.temperature)
    }

    pub fn get_aiclk_series(&self) -> Vec<u32> {
        self.get_series(|t| t.aiclk)
    }

    pub fn get_stats(&self) -> Option<TelemetryStats> {
        if self.buffer.is_empty() {
            return None;
        }

        let count = self.buffer.len() as f32;

        let calc_stats = |values: &[f32]| -> (f32, f32, f32) {
            let min = values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let avg = values.iter().sum::<f32>() / count;
            (min, max, avg)
        };

        let voltages = self.get_voltage_series();
        let currents = self.get_current_series();
        let powers = self.get_power_series();
        let temperatures = self.get_temperature_series();
        let aiclks = self.get_aiclk_series();

        let (min_voltage, max_voltage, avg_voltage) = calc_stats(&voltages);
        let (min_current, max_current, avg_current) = calc_stats(&currents);
        let (min_power, max_power, avg_power) = calc_stats(&powers);
        let (min_temperature, max_temperature, avg_temperature) = calc_stats(&temperatures);

        let min_aiclk = *aiclks.iter().min().unwrap();
        let max_aiclk = *aiclks.iter().max().unwrap();
        let avg_aiclk = (aiclks.iter().map(|&x| x as u64).sum::<u64>() as f32 / count) as u32;

        Some(TelemetryStats {
            min_voltage,
            max_voltage,
            avg_voltage,
            min_current,
            max_current,
            avg_current,
            min_power,
            max_power,
            avg_power,
            min_temperature,
            max_temperature,
            avg_temperature,
            min_aiclk,
            max_aiclk,
            avg_aiclk,
        })
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryStats {
    pub min_voltage: f32,
    pub max_voltage: f32,
    pub avg_voltage: f32,
    pub min_current: f32,
    pub max_current: f32,
    pub avg_current: f32,
    pub min_power: f32,
    pub max_power: f32,
    pub avg_power: f32,
    pub min_temperature: f32,
    pub max_temperature: f32,
    pub avg_temperature: f32,
    pub min_aiclk: u32,
    pub max_aiclk: u32,
    pub avg_aiclk: u32,
}
