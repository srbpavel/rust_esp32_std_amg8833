mod config;
mod errors;
mod i2c;
mod sensor_agm;
mod eventloop;
mod wifi;

use crate::sensor_agm::Temperature;
use crate::sensor_agm::HeatMap;
use crate::sensor_agm::Payload;
use crate::sensor_agm::LEN;
//use crate::sensor_agm::POW;
use crate::sensor_agm::PAYLOAD_LEN;

use eventloop::EventLoopMessage;

use errors::WrapError;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::eventloop::EspSystemEventLoop;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cError;
use esp_idf_hal::i2c::I2cDriver;

use grideye::Address;
use grideye::GridEye;
use grideye::Power;

use ssd1306::prelude::*;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::text::Baseline;
use embedded_graphics::text::Text;

use esp_idf_svc::mqtt::client::MqttClientConfiguration;
use esp_idf_svc::mqtt::client::EspMqttClient;
use embedded_svc::mqtt::client::QoS;

use embedded_svc::mqtt::client::{
    Details::Complete,
    Event::{
        Received,
        BeforeConnect,
        Connected,
        Disconnected,
        Subscribed,
        Unsubscribed,
        Published,
        Deleted,
    },
    Message,
    Connection,
};

use std::sync::mpsc::channel;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

//
// todo!(all error)
fn main() -> Result<(), WrapError<I2cError>> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();
    info!("### amg8833: array {LEN}x{LEN}");

    let app_config = config::CONFIG;
    if app_config.flag_debug.eq(&true) {
        info!("### CONFIG >>> {:#?}", app_config);
    }
    
    let machine_boot = EspSystemTime {}.now();
    warn!("machine_uptime: {machine_boot:?}");
    let mut cycle_counter = 0;

    let mut sleep = FreeRtos {};
    let delay = Ets {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    // EVENT LOOP -> FUTURE_USE
    let sysloop = EspSystemEventLoop::take()?;
    error!("@@@ about to subscribe to the background event loop");
    let _subscription = sysloop.subscribe(move |message: &EventLoopMessage| {
        error!("@@@ got message from the event loop: {:?} <- uptime",
               message);
    })?;
    
    // WIFI
    let nvs_partition = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;
    let _wifi = wifi::wifi(peripherals.modem,
                           sysloop,
                           app_config.wifi_ssid,
                           app_config.wifi_pass,
                           nvs_partition,
    )?;

    // MQTT
    let mqtt_client_id_uniq = &format!("{}_{}_{}_{}", 
                                       app_config.machine_type,
                                       app_config.machine_number,
                                       app_config.machine_name,
                                       app_config.uuid,
    );
    
    let mqtt_config: MqttClientConfiguration = MqttClientConfiguration {
        client_id: Some(mqtt_client_id_uniq),
        ..Default::default()
    };
    
    let (mut mqtt_client, mut mqtt_connection) = EspMqttClient::new_with_conn(
        app_config.mqtt_broker_url,
        &mqtt_config,
    )?;

    // CHANNEL's
    let (mqtt_sender, _mqtt_receiver) = channel();

    // MQTT LISTEN via CONNECTION + also PUBLISH
    info!("### MQTT Listening for messages");
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
                                        mqtt_sender
                                            .send(recieved_bytes)
                                            .unwrap();
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
                        // SILENT
                        //info!("MQTT Message : Published({})", message_id)
                    },
                    Deleted(message_id) => info!("MQTT Message : Deleted({})", message_id),
                },
                Err(e) => info!("### MQTT Message ERROR: {:?}", e), // todo!()
            }
        }
        
        info!("MQTT connection loop exit");
    });

    // VALID I2C pins
    let pin_scl = peripherals.pins.gpio8;
    let pin_sda = peripherals.pins.gpio10;

    // I2C type just to see type for error imlp
    let i2c: Result<esp_idf_hal::i2c::I2cDriver<'_>, esp_idf_sys::EspError> =
        i2c::config(peripherals.i2c0, pin_sda, pin_scl);
    let i2c = i2c?;

    // I2C SHARED
    let i2c_shared: &'static _ = shared_bus::new_std!(I2cDriver = i2c).ok_or(WrapError::I2cError)?;
    
    let i2c_proxy_1 = i2c_shared.acquire_i2c(); // agm
    let i2c_proxy_2 = i2c_shared.acquire_i2c(); // ssd1306
    let i2c_proxy_5 = i2c_shared.acquire_i2c(); // i2c scan share loop
    
    // GRIDEYE
    let mut grideye = GridEye::new(i2c_proxy_1, delay, Address::Standard);

    // DISPLAY    
    // Option<Ssd1306<DI, SIZE, MODE>, MonoTextStyle<'_, BinaryColor>)>
    let mut display_data = if app_config.flag_display.eq(&true) {
        let display = Ssd1306::new(I2CDisplayInterface::new(i2c_proxy_2),
                                   DisplaySize128x64,
                                   DisplayRotation::Rotate0,
        )
            .into_buffered_graphics_mode();

        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        
        Some((display, text_style))
    } else { None };

    // INIT
    if let Some((ref mut display, text_style)) = display_data {
        match display.init() {
            Ok(_) => {
                Text::with_baseline(
                    "foookume is QuEeN!",
                    Point::new(0, 32),
                    text_style,
                    Baseline::Top,
                )
                    .draw(display)?;
                
                display.flush()?;
                
                // boot display
                sleep.delay_ms(app_config.delay_sleep_after_boot);
            },
            Err(e) => {
                display_data = None;

                error!("Error display init: {e:?}")
            },
        }
    }

    // TOTAL
    let mut max_temperature_total = sensor_agm::TEMPERATURE_MAX;
    let mut min_temperature_total = sensor_agm::TEMPERATURE_MIN; 

    if grideye.power(Power::Wakeup).is_ok() {
        loop {
            cycle_counter += 1;

            // I2C LOOP scan for debug
            if app_config.flag_scan_i2c.eq(&true) {
                let mut i2c_clone = i2c_proxy_5.clone();
                std::thread::spawn(move || {
                    warn!("i2c_scan_shared_loop + thread");
                    let active_address = i2c::scan_shared(&mut i2c_clone);
                    
                    info!(
                        "I2C LOOP active address: {:?}",
                        match active_address {
                            Some(active) => {
                                active
                                    .iter()
                                    .map(|a| format!("{a:#X} "))
                                    .collect::<Vec<String>>()
                                    .concat()
                            }
                            None => {
                                String::from("")
                            }
                        }
                    );
                });
            }
            
            // GRIDEYE
            let (payload, min_temperature, max_temperature): (Payload<PAYLOAD_LEN>, Temperature, Temperature) = sensor_agm::measure_as_array_bytes(&mut grideye);

            if app_config.flag_show_payload {
                warn!("payload[{}]: {:?}",
                      payload.0.len(),
                      payload.0,
                );
            }
            
            // TOTAL
            if max_temperature > max_temperature_total {
                max_temperature_total = max_temperature;
            }

            if min_temperature < min_temperature_total {
                min_temperature_total = min_temperature;
            }
            
            // MQTT_PUB
            match mqtt_client.publish(app_config.mqtt_topic,
                                      QoS::AtLeastOnce,
                                      false,
                                      &payload.0,
            ) {
                Ok(_status) => {
                    // SILENT
                    //info!("### MQTT >>> Status: {status} / Published to: '{topic}'");
                },
                Err(e) => {
                    error!("### MQTT >>> Publish Error: '{e}'");
                },
            };

            // SHOW ARRAY_INDEX
            // index is u8 but method takes Temperature, try harder ...
            // generic -> can be done, but what to do with array init
            // honestly, we can just display ARRAY just to see, no need 2d array
            if app_config.flag_show_array_index {
                sensor_agm::STATIC_ARRAY_FLIPPED_HORIZONTAL
                    .chunks(LEN)
                    .for_each(|chunk| {
                        info!("{:?}", chunk)
                    });
                    
                    
                /*
                let array_index: [Temperature; POW] = sensor_agm::STATIC_ARRAY_FLIPPED_HORIZONTAL
                    .iter()
                    .map(|i|*i as Temperature)
                    .collect::<Vec<Temperature>>()
                    .try_into()
                    .unwrap();
                
                let array_map: Result<HeatMap<Temperature, LEN>, &'static str> = HeatMap::try_from(array_index);

                match array_map {
                    Ok(m) => info!("array_map_display:\n\n{m}"),
                    Err(e) => error!("array to heat_map failed >>> {e}"),
                }
                */
            }

            // DISPLAY HEATMAP
            if app_config.flag_show_heatmap {
                match sensor_agm::payload_to_values(payload.clone()) {
                    Ok(grid_raw) => {
                        let heat_map: Result<HeatMap<Temperature, LEN>, &'static str> = HeatMap::try_from(grid_raw);
                        
                        match heat_map {
                            Ok(m) => {
                                info!("heat_map_display:\n\n{m}");
                            },
                            Err(e) => {
                                error!("array to heat_map failed >>> {e}");
                            },
                        }
                    },
                    Err(e) => {
                        error!("payload to array failed >>> {e:?}");
                    },
                }
            }
            
            // DISPLAY
            if let Some((ref mut display, text_style)) = display_data {
                display.clear();

                Text::with_baseline(
                    &format!("{cycle_counter}"),
                    Point::new(0, 0),
                    text_style,
                    Baseline::Top,
                )
                    .draw(display)?;
                
                Text::with_baseline(
                    &format!("max:  {max_temperature:0.02}"),
                    Point::new(48, 0),
                    text_style,
                    Baseline::Top,
                )
                    .draw(display)?;
            
                Text::with_baseline(
                    &format!("min:  {min_temperature:0.02}"),
                    Point::new(48, 16),
                    text_style,
                    Baseline::Top,
                )
                    .draw(display)?;
                
                // TOTAL
                Text::with_baseline(
                    &format!("{min_temperature_total:0.02} / {max_temperature_total:0.02}"),
                    Point::new(0, 48),
                    text_style,
                    Baseline::Top,
                )
                    .draw(display)?;
                
                Text::with_baseline(
                    &format!("diff: {:0.02}", max_temperature - min_temperature,),
                    Point::new(48, 32),
                    text_style,
                    Baseline::Top,
                )
                    .draw(display)?;
                
                display.flush()?;
            }
                
            sleep.delay_ms(app_config.delay_sleep_duration_ms);
        }
    }

    error!("ok");
    Ok(())
}
