use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

use crate::types;

type ParsedMessage = (String, types::SensorValue);

fn parse_message(topic: &str, payload: &[u8]) -> anyhow::Result<Option<ParsedMessage>> {
    if topic.ends_with("/data") || topic.ends_with("/status") {
        let sensor: types::SensorData = serde_json::from_slice(payload)?;
        Ok(Some((
            sensor.address.clone(),
            types::SensorValue::Data(sensor),
        )))
    } else if topic.ends_with("/device") {
        let report: types::SensorReport = serde_json::from_slice(payload)?;
        Ok(Some((
            report.device.clone(),
            types::SensorValue::Report(report),
        )))
    } else {
        Ok(None)
    }
}

fn with_parsed<F>(topic: &str, payload: &[u8], f: F)
where
    F: FnOnce(ParsedMessage),
{
    match parse_message(topic, payload) {
        Ok(Some(p)) => f(p),
        Ok(None) => tracing::info!("ignoring {}", topic),
        Err(e) => tracing::warn!("failed to parse {} {}", topic, e),
    }
}

pub fn new(host: String, port: u16) -> (AsyncClient, EventLoop) {
    let client_id = format!("prometheus-relay-{}", &Uuid::new_v4().to_string()[..8]);
    let mut mqttopts = MqttOptions::new(client_id, host, port);

    mqttopts.set_keep_alive(std::time::Duration::from_secs(30));

    rumqttc::AsyncClient::new(mqttopts, 10)
}

pub async fn run(
    client: AsyncClient,
    mut eventloop: EventLoop,
    sensors: types::SensorMap,
) -> anyhow::Result<()> {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Packet::ConnAck(c))) => {
                tracing::info!("ConnAck {:?}", c);

                // Despite examples, only do topic subscriptions after ConnAck,
                // as subscriptions do not appear to be retained across reconnects.
                //
                // If these error, it means the filter string itself is invalid.
                client
                    .subscribe("home/sensor/+/status", QoS::AtMostOnce)
                    .await?;
                client
                    .subscribe("home/sensor/+/+/data", QoS::AtMostOnce)
                    .await?;
                client
                    .subscribe("home/sensor/+/device", QoS::AtMostOnce)
                    .await?;
            }

            Ok(Event::Incoming(Packet::Publish(p))) => {
                with_parsed(&p.topic, &p.payload, |(key, value)| {
                    tracing::info!("observing {:?}", value);
                    sensors.insert(key, (value, SystemTime::now()));
                });
            }

            Ok(event) => {
                tracing::info!("event {:?}", event);
            }

            Err(e) => {
                tracing::error!("MQTT connection error: {}", e);
                // rumqttc will reconnect; but delay before letting loop continue
                // TODO: consider backoff
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
