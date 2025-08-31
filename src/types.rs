use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::SystemTime};

#[derive(Deserialize, Serialize, Debug)]
pub struct SensorData {
    pub address: String,
    pub temp_c: f64,
    pub temp_f: f64,
    pub humidity_pct: i8,
    pub voltage: f64,
    pub rssi: i8,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SensorReport {
    pub device: String,
    pub internal_temp_c: f64,
    pub current_time: u64,
}

#[derive(Debug)]
pub enum SensorValue {
    Data(SensorData),
    Report(SensorReport),
}

pub type SensorMap = Arc<DashMap<String, (SensorValue, SystemTime)>>;
