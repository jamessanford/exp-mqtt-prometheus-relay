use anyhow::Context;
use axum::{Router, routing::get};
use clap::Parser;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::task::JoinSet;

mod metrics;
mod mqtt;
mod types;

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value = "localhost")]
    mqtt_host: String,
    #[clap(long, default_value = "2883")]
    mqtt_port: u16,
    #[clap(long, default_value = "0.0.0.0:9930")]
    listen: String,
}

async fn http_run(listen_addr: String, sensors: types::SensorMap) -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "mqtt-prometheus-relay" }))
        .route("/metrics", get(metrics::handler).with_state(sensors));

    let listener = TcpListener::bind(&listen_addr)
        .await
        .with_context(|| format!("failed to bind HTTP server on {}", listen_addr))?;

    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let sensors: types::SensorMap = Arc::new(DashMap::new());

    let mut tasks = JoinSet::new();

    tasks.spawn({
        let sensors = sensors.clone();
        async move { http_run(args.listen.clone(), sensors).await }
    });

    tasks.spawn({
        let sensors = sensors.clone();
        let (client, eventloop) = mqtt::new(args.mqtt_host, args.mqtt_port);
        async move { mqtt::run(client, eventloop, sensors).await }
    });

    while let Some(res) = tasks.join_next().await {
        res??;
    }

    Ok(())
}
