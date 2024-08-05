use rumqttd::{Broker, Config};

fn main() {
    dotenv::dotenv().ok();

    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .with_module_level("rumqttd::router::routing", log::LevelFilter::Off)
        .with_module_level("rumqttd::server::broker", log::LevelFilter::Off)
        .init()
        .unwrap();

    let conf = config(Settings::from_env());

    // let (auth_tx, auth_rx) = std::sync::mpsc::channel::<AuthMsg>();

    let mut broker = Broker::new(conf, None);

    println!("======> STARTING <=======");
    broker.start().unwrap()
}
pub struct Settings {
    pub mqtt_port: u16,
    pub ws_port: Option<u16>,
    pub console_port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            mqtt_port: 1883,
            ws_port: None,
            console_port: 3030,
        }
    }
}

impl Settings {
    fn from_env() -> Self {
        let mut sets: Settings = Default::default();
        if let Ok(mqtt_port) = std::env::var("BROKER_MQTT_PORT") {
            if let Ok(mp) = mqtt_port.parse::<u16>() {
                sets.mqtt_port = mp;
            }
        }
        if let Ok(ws_port) = std::env::var("BROKER_WS_PORT") {
            if let Ok(wsp) = ws_port.parse::<u16>() {
                sets.ws_port = Some(wsp);
            }
        }
        if let Ok(console_port) = std::env::var("BROKER_CONSOLE_PORT") {
            if let Ok(cp) = console_port.parse::<u16>() {
                sets.console_port = cp;
            }
        }
        sets
    }
}

fn config(settings: Settings) -> Config {
    use rumqttd::{ConnectionSettings, ConsoleSettings, ServerSettings};
    use std::collections::HashMap;
    use std::net::{Ipv4Addr, SocketAddrV4};
    let router = rumqttd::RouterConfig {
        instant_ack: true,
        max_segment_size: 104857600,
        max_segment_count: 10,
        max_connections: 10010,
        max_read_len: 10240,
        ..Default::default()
    };
    let conns = ConnectionSettings {
        connection_timeout_ms: 5000,
        throttle_delay_ms: 0,
        max_payload_size: 262144,
        max_inflight_count: 256,
        max_inflight_size: 1024,
        auth: None,
        dynamic_filters: true,
    };
    let mut v4_servers = HashMap::new();
    v4_servers.insert(
        "v4".to_string(),
        ServerSettings {
            name: "v4".to_string(),
            listen: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), settings.mqtt_port).into(),
            next_connection_delay_ms: 1,
            connections: conns.clone(),
            tls: None,
        },
    );
    let ws_servers = if let Some(wsp) = settings.ws_port {
        let mut ws = HashMap::new();
        ws.insert(
            "ws".to_string(),
            ServerSettings {
                name: "ws".to_string(),
                listen: SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), wsp).into(),
                next_connection_delay_ms: 1,
                connections: conns,
                tls: None,
            },
        );
        Some(ws)
    } else {
        None
    };
    Config {
        id: 0,
        v4: v4_servers,
        ws: ws_servers,
        router,
        console: ConsoleSettings::new(&format!("0.0.0.0:{}", settings.console_port)),
        cluster: None,
        ..Default::default()
    }
}
