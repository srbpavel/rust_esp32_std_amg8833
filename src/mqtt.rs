//MQTT
use crate::WrapError;

#[warn(unused_imports)]
use log::error;
#[warn(unused_imports)]
use log::info;
#[warn(unused_imports)]
use log::warn;

use esp_idf_svc::mqtt::client::EspMqttClient;
use esp_idf_svc::mqtt::client::MqttClientConfiguration;

use embedded_svc::mqtt::client::Client;
use embedded_svc::mqtt::client::Publish;
//use embedded_svc::mqtt::client::Connection;
use embedded_svc::mqtt::client::Message;
use embedded_svc::mqtt::client::MessageImpl;
use embedded_svc::mqtt::client::QoS;
use embedded_svc::utils::mqtt::client::ConnState;

use esp_idf_sys::EspError;

//
//pub fn init<CP, C>() -> Result<(
pub fn init() -> Result<(
    &'static str,
    //impl CP,
    //EspMqttClient,
    impl Client + Publish,
    //impl C,
    impl embedded_svc::mqtt::client::Connection,
    //embedded_svc::utils::mqtt::client::Connection,
), WrapError<EspError>>
//where
//    CP: Client + Publish,
//    C: embedded_svc::mqtt::client::Connection,
//    //C: embedded_svc::utils::mqtt::client::Connection,
{
    // MQTT
    //let mqtt_broker_url = mqtt::mqtt_broker_url(&app_config);
    let mqtt_broker_url = "broke";
    //let mqtt_client_id = mqtt::mqtt_client_id(&app_config);
    let mqtt_client_id = "client_id";

    //let mqtt_config = mqtt::mqtt_config(&mqtt_client_id);
    let mqtt_config: MqttClientConfiguration = MqttClientConfiguration {
        client_id: Some(mqtt_client_id),
        ..Default::default()
    };
    
    let (mut mqtt_client, mut mqtt_connection) = EspMqttClient::new_with_conn(
        mqtt_broker_url,
        &mqtt_config,
    )?;
    
    Ok((mqtt_client_id, mqtt_client, mqtt_connection))
}


/*
//use embedded_svc::utils::mutex::RawCondvar;
//use esp_idf_svc::mdns::Type;

//use serde::Deserialize;

// single machine + color via name
//  { machine_id: 101, color: Some("black"), rgb: None }
//
// broadcast to all machines + color via rgb
//  { machine_id: 255, color: None, rgb: Some([0, 0, 0]) }
//
//#[derive(Deserialize, Debug)]
pub struct RgbLedCmd {
    pub machine_id: u8,
    pub color: Option<String>,
    pub rgb: Option<[u8; 3]>,
}

//#[derive(Deserialize, Debug)]
pub struct Measure {
    pub machine_id: u8,
    pub sensors: Vec<String>,
}


/*
pub struct MqttClient<'a> {
    pub config: &'a Config,
}

impl MqttClient<'_> {
    pub fn new(&self) -> Result<(
        EspMqttClient<ConnState<MessageImpl, EspError>>,
        Connection<esp_idf_svc::private::mutex::RawCondvar, MessageImpl, EspError>),
                                EspError> {

        let mqtt_broker_url = mqtt_broker_url(&self.config);
        let mqtt_client_id = mqtt_client_id(&self.config);
        let mqtt_config = mqtt_config(&mqtt_client_id);

        if self.config.debug.eq(&true) {
            info!("### MQTT_CONFIG: {:?}", mqtt_config);
        }
        
        EspMqttClient::new_with_conn(
            mqtt_broker_url,
            &mqtt_config,
        )
    }
}
*/


//
// MqttClientConfiguration::default() -> set client_id: None -> multiple machines collision
//
pub fn mqtt_config(client_id: &str) -> MqttClientConfiguration {
    MqttClientConfiguration {
        client_id: Some(client_id),
        ..Default::default()
    
    }
}


//
/*
pub fn mqtt_client_id(config: &Config) -> String {
    format!("{}_{}",
            config.machine_type,
            config.machine_number,
    )
}
*/

/*
//
pub fn mqtt_broker_url(config: &Config) -> String {
    if !config.mqtt_user.is_empty() {
        format!(
            "mqtt://{}:{}@{}",
            config.mqtt_user,
            config.mqtt_pass,
            config.mqtt_host
        )
    } else {
        format!("mqtt://{}", config.mqtt_host)
    }
}
*/

/*
//
pub fn pub_to_topic(client: &mut EspMqttClient<ConnState<MessageImpl, EspError>>,
                    topic: &str,
                    payload: &[u8],
                    app_config: &Config) {
    
    match client.publish(topic,
                         QoS::AtLeastOnce,
                         false,
                         payload,
    ) {
        Ok(status) => {
            if app_config.debug_ping_pong.eq(&true) {
                info!("### MQTT >>> Status: {status} / Published to: '{topic}'");
            }
        },
        Err(e) => {
            error!("### MQTT >>> Publish Error: '{e}'");
        },
    };
}
*/

//
pub fn sub_to_topics(client: &mut EspMqttClient<ConnState<MessageImpl, EspError>>,
                     topics: &[&str]) {
    
    topics
        .iter()
        .for_each(|topic| match client.subscribe(topic, QoS::AtLeastOnce) {
            Ok(status) => {
                info!("### MQTT >>> Status: {status} / Subscribed to: '{topic}'");
            },
            Err(e) => {
                error!("### MQTT >>> Subscribed Error: '{e}'");
            },
        });
}

/*
//
pub fn json_msg_to_data(message_data: &[u8],
                        app_config: &Config) -> String {
    
    let data = message_data
        .iter()
        .map(|c| *c as char)
        .map(|ch| ch.to_string())
        .collect::<Vec<_>>()
        .concat();

    if app_config.debug_json.eq(&true) {
        info!("data: {data:?}");
    }
    
    data
}
*/

/*
// mqtt message data json to struct
pub fn data_via_json_to_struct<'a, T>(json: &'a str,
                                      app_config: &Config) -> Result<T, serde_json::Error>
where
    T: std::fmt::Debug + Deserialize<'a>
{
    let data = serde_json::from_str(json);
    
    if app_config.debug_struct.eq(&true) {
        info!("struct: {data:?}");
    }

    data
}
*/

//TOPICS
#[allow(dead_code)]
pub fn data_battery_topic(uuid: &str) -> String {
    format!("{uuid}/data/battery")
}

#[allow(dead_code)]
pub fn data_shtc_topic(uuid: &str) -> String {
    format!("{uuid}/data/shtc")
}

#[allow(dead_code)]
pub fn data_ds_topic(uuid: &str) -> String {
    format!("{uuid}/data/ds")
}

pub fn data_harvester_topic(uuid: &str) -> String {
    format!("{uuid}/data/harvester")
}

pub fn data_config_topic(uuid: &str) -> String {
    format!("{uuid}/data/config")
}

// REQUESTS
pub fn measure_shtc_sensor_topic(uuid: &str) -> String {
    format!("{uuid}/measure/shtc_sensor")
}

pub fn measure_ds_sensor_topic(uuid: &str) -> String {
    format!("{uuid}/measure/ds_sensor")
}

pub fn measure_battery_topic(uuid: &str) -> String {
    format!("{uuid}/measure/battery")
}

pub fn measure_topic(uuid: &str) -> String {
    format!("{uuid}/measure")
}

// CMD
pub fn rgb_led_topic(uuid: &str) -> String {
    format!("{uuid}/rgb_led")
}

// HEARTBEAT
pub fn heart_beat_ping_topic(uuid: &str) -> String {
    format!("{uuid}/heart_beat/ping")
}

pub fn heart_beat_pong_topic(uuid: &str) -> String {
    format!("{uuid}/heart_beat/pong")
}
*/
