use axum::extract::State;
use std::{
    fmt::Display,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::types::{SensorMap, SensorValue};

const STALE_THRESHOLD: Duration = Duration::from_secs(90);

fn push_metric<T: Display>(buf: &mut String, name: &str, address: &str, value: T) {
    buf.push_str(&format!("{name}{{address=\"{address}\"}} {value}\n"));
}

fn epoch_secs(ts: &SystemTime) -> u64 {
    ts.duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

pub async fn handler(State(sensors): State<SensorMap>) -> String {
    let now = SystemTime::now();
    let mut result = String::new();
    sensors.retain(|_address, (_sensor, ts)| {
        now.duration_since(*ts).unwrap_or(Duration::MAX) < STALE_THRESHOLD
    });
    for item in sensors.iter() {
        let (address, (sensor_value, ts)) = item.pair();

        match sensor_value {
            SensorValue::Data(data) => {
                push_metric(&mut result, "xiaomi_temperature_c", address, data.temp_c);
                push_metric(&mut result, "xiaomi_temperature_f", address, data.temp_f);
                push_metric(
                    &mut result,
                    "xiaomi_humidity_pct",
                    address,
                    data.humidity_pct,
                );
                push_metric(&mut result, "xiaomi_battery_volts", address, data.voltage);
                push_metric(&mut result, "xiaomi_rssi_dbm", address, data.rssi);
                push_metric(&mut result, "xiaomi_update_time", address, epoch_secs(ts));
            }
            SensorValue::Report(report) => {
                push_metric(
                    &mut result,
                    "xiaomi_internal_temp_c",
                    address,
                    report.internal_temp_c,
                );
                push_metric(
                    &mut result,
                    "xiaomi_internal_uptime",
                    address,
                    report.current_time,
                );
            }
        }
    }
    tracing::info!("serving /metrics");
    result
}
