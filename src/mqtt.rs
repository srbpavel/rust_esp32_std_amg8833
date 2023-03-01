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
use embedded_svc::mqtt::client::Message;
use embedded_svc::mqtt::client::MessageImpl;
use embedded_svc::mqtt::client::QoS;
use embedded_svc::utils::mqtt::client::ConnState;

use esp_idf_sys::EspError;

// todo!()
pub fn init() -> Result<(
    &'static str,
    impl Client + Publish,
    impl embedded_svc::mqtt::client::Connection,
), WrapError<EspError>> {
    let mqtt_broker_url = "broker";
    let mqtt_client_id = "client_id";

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
