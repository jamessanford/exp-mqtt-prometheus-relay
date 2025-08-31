Experiment in Rust: Subscribe to MQTT data, offer as Prometheus metrics.

Exposes the MQTT data from my "BLEview2" on esp32-s3 sensors around my home.

---

This is an experiment to try out Rust.  I particularly like how it uses very little RAM,
and with `#[tokio::main(flavor = "current_thread")]` it turns into a single thread `epoll`
based server that is extremely efficient.

---

MQTT input:

```
home/sensor/esp32-84ffb62f8b8/a4:c1:38:47:01:6c/data = {
  "address": "a4:c1:38:47:01:6c",
  "temp_c": 24.6,
  "temp_f": 76.3,
  "humidity_pct": 56,
  "voltage": 2.944,
  "rssi": -66
}
```

becomes Prometheus /metrics:

```
xiaomi_temperature_c{address="a4:c1:38:47:01:6c"} 24.6
xiaomi_temperature_f{address="a4:c1:38:47:01:6c"} 76.3
xiaomi_humidity_pct{address="a4:c1:38:47:01:6c"} 56
xiaomi_battery_volts{address="a4:c1:38:47:01:6c"} 2.944
xiaomi_rssi_dbm{address="a4:c1:38:47:01:6c"} -66
xiaomi_update_time{address="a4:c1:38:47:01:6c"} 1756584485
```
