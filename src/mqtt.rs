#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

use crate::config::Config;

use embedded_svc::mqtt::client::Details::Complete;
use embedded_svc::mqtt::client::Event::Received;
use embedded_svc::mqtt::client::Event::BeforeConnect;
use embedded_svc::mqtt::client::Event::Connected;
use embedded_svc::mqtt::client::Event::Disconnected;
use embedded_svc::mqtt::client::Event::Subscribed;
use embedded_svc::mqtt::client::Event::Unsubscribed;
use embedded_svc::mqtt::client::Event::Published;
use embedded_svc::mqtt::client::Event::Deleted;
use embedded_svc::mqtt::client::Message;
use embedded_svc::mqtt::client::Connection;

use esp_idf_svc::mqtt::client::MqttClientConfiguration;
use esp_idf_svc::mqtt::client::EspMqttClient;

use esp_idf_sys::EspError;

pub enum TopicKind {
    Payload,
    CommonLog,
}

pub struct MqttPub {
    topic: TopicKind,
    // &[u8] is not safe to share between threads
    // Arc<u8> can also be used
    msg: Vec<u8>,
}

impl MqttPub {
    //
    pub fn new(topic: TopicKind,
               msg: &[u8],
    ) -> Self {
        Self {
            topic,
            msg: msg.to_vec(),
        }
    }
}

//
// esp32_c3_rev3_rust_board_1_king_cc0e0cf470704e159f7462eb4ba6e3d8
// esp32_c3_rev3_rust_board_2_queen_a0ed93add67447a4abdb9648f77d7196
// esp32_c3_rev3_rust_board_3_witch_df2f2dba0bd14030ac80dcf885941f36
//
pub fn client_id(config: &Config,
                 uuid: &str) -> String {

    [config.machine_type,
     &format!("{}", config.machine_number),
     config.machine_name,
     uuid,
    ].join("_")
}

// base + machine + uuid
// /grid_eye/king/1769ddd62b944786bbc8ea9fa50a5867"
//
// base + common_log
// /grid_eye/common_log
//
pub fn create_topic(base: &str,
                    parts: &[&str],
) -> String {
    let mut path = std::path::PathBuf::new();
    path.push(base);
    
    let topic = parts
        .iter()
        .fold(path, |topic, part|
              topic.join(part)
        );

    String::from(
        topic.to_str().unwrap_or(
            // todo!
            "/grid_eye/common_log"
        )
    )
}

//
pub fn init(app_config: &Config,
            machine_uuid: &str,
            mqtt_client_receiver: std::sync::mpsc::Receiver<MqttPub>,
) -> Result<(), crate::WrapError<EspError>> {
    let mqtt_client_id_uniq = client_id(app_config,
                                        machine_uuid,
    );
    
    let mqtt_config = MqttClientConfiguration {
        client_id: Some(&mqtt_client_id_uniq),
        ..Default::default()
    };

    let (mut mqtt_client, mut mqtt_connection) = EspMqttClient::new_with_conn(
        app_config.mqtt_broker_url,
        &mqtt_config,
    )?;

    // MQTT CONNECTION: publish + send + ...
    warn!("### MQTT connection");
    std::thread::spawn(move || {
        while let Some(msg) = mqtt_connection.next() {
            match msg {
                Ok(message) => match message {
                    Received(recieved_bytes) => {
                        match recieved_bytes.details() {
                            Complete => {
                                match std::str::from_utf8(recieved_bytes.data()) {
                                    Err(e) => info!("### MQTT Message: Received Error: unreadable message! ({}) >>> data: {:?}", e, recieved_bytes.data()),
                                    Ok(_data) => {
                                        /* // FUTURE_USE
                                        info!("MQTT Message : Received({})",
                                              data,
                                        );

                                        mqtt_sender
                                            .send(recieved_bytes)
                                            .unwrap();
                                        */
                                    },
                                }
                            }
                            _ => error!(" ### MQTT received_bytes details not COMPLETE status"),
                        }
                    },
                    BeforeConnect => info!("MQTT Message : Before connect"),
                    Connected(tf) => info!("MQTT Message : Connected({})", tf),
                    Disconnected => info!("MQTT Message : Disconnected"),
                    Subscribed(message_id) => {
                        info!("MQTT Message : Subscribed({})", message_id)
                    }
                    Unsubscribed(message_id) => {
                        info!("MQTT Message : Unsubscribed({})", message_id)
                    }
                    Published(_message_id) => {
                        // DEBUG -> SILENT
                        //info!("MQTT Message : Published({})", message_id)
                    },
                    Deleted(message_id) => info!("MQTT Message : Deleted({})", message_id),
                },
                Err(e) => info!("### MQTT Message ERROR: {:?}", e), // todo!()
            }
        }
        
        warn!("MQTT connection loop exit");
    });
    //_

    let mqtt_topic_common_log = create_topic(
        app_config.mqtt_topic_base,
        &[app_config.mqtt_topic_common],
    );

    let mqtt_topic_payload = create_topic(
        app_config.mqtt_topic_base,
        &[app_config.machine_name,
          machine_uuid,
        ],
    );
    
    // MQTT CLIENT 
    warn!("### MQTT PUB");
    std::thread::spawn(move || {
        while let Ok(channel_data) = mqtt_client_receiver.recv() {
            // DEBUG
            //info!("$$$ MQTT PUB receiver msg: {channel_data:?}");

            let topic = match channel_data.topic {
                TopicKind::CommonLog => {
                    warn!("mqtt_topic_common_log: {:?}", mqtt_topic_common_log);
                    
                    &mqtt_topic_common_log
                    //mqtt_topic_common_log.clone()
                },
                TopicKind::Payload => {
                    // FUTURE USE -> if uuid per measurement will be needed    
                    /*
                    let topic = create_topic(
                        &mqtt_topic_payload,
                        &[&uuid::Uuid::new_v4().simple().to_string()],
                    );
                    */
                    
                    warn!("mqtt topic_payload: {:?}", mqtt_topic_payload);
                    //warn!("mqtt topic_payload: {:?}", topic);
                    
                    &mqtt_topic_payload
                    //topic
                },
            };
            
            match mqtt_client.publish(
                // &str
                topic,
                //&topic,
                // qos
                embedded_svc::mqtt::client::QoS::AtLeastOnce,
                // retain: bool
                false,
                // Vec<u8> -> &[u8]
                &channel_data.msg,
            ) {
                Ok(_status) => {
                    // todo!() -> config flag
                    // SILENT
                    //info!("### MQTT >>> Status: {status} / Published to: '{topic}'");
                },
                Err(e) => {
                    error!("### MQTT >>> Publish Error: '{e:?}'");
                },
            };
        }
    });
    
    Ok(())
}
