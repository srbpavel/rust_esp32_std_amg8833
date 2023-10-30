mod config;
mod errors;
mod i2c;
mod mqtt;
mod sensor_agm;
mod eventloop;
mod wifi;
mod display_ssd;
//mod display_ili;

use crate::errors::WrapError;

use crate::mqtt::MqttPub;

use crate::sensor_agm::Temperature;
use crate::sensor_agm::HeatMap;
use crate::sensor_agm::Payload;
use crate::sensor_agm::LEN;
use crate::sensor_agm::PAYLOAD_LEN;

use crate::display_ssd::Render as RenderSsd;
use crate::display_ssd::DisplaySsdClear;
use crate::display_ssd::DisplaySsdFlush;

//use crate::display_ili::Render as RenderIli;
//use crate::display_ili::DisplayIliClear;

use eventloop::EventLoopMessage;

use esp_idf_sys as _;

use esp_idf_svc::systime::EspSystemTime;
use esp_idf_svc::eventloop::EspSystemEventLoop;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cError;
use esp_idf_hal::i2c::I2cDriver;

// /*
use esp_idf_hal::spi::SpiDeviceDriver;
use esp_idf_hal::prelude::FromValueType;
use esp_idf_hal::gpio::AnyOutputPin;
use esp_idf_hal::gpio::AnyIOPin;
use esp_idf_hal::gpio::PinDriver;
// */

// /*
//use embedded_hal::blocking::delay::DelayUs;
//use embedded_hal::blocking::delay::DelayMs;

//use esp_idf_hal::delay::FreeRtos;
//use std::sync::mpsc::Sender;
//use embedded_graphics::geometry::Point;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;
// */

use grideye::Address;
use grideye::GridEye;
use grideye::Power;

use embedded_graphics::geometry::Point;

use std::sync::mpsc::channel;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

const BEEP_COUNTER: usize = 100; // 1000;

//
fn main() -> Result<(), WrapError<esp_idf_sys::EspError>> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    info!("### amg8833: array {LEN}x{LEN}");

    let machine_uuid = format!("{}", uuid::Uuid::new_v4().simple());
    let app_config = config::CONFIG;
    if app_config.flag_debug.eq(&true) {
        info!("### CONFIG >>> {:#?}\n machine_uuid: {}",
              app_config,
              machine_uuid,
        );
    }
    
    let machine_boot = EspSystemTime {}.now();
    warn!("machine_uptime: {machine_boot:?}");
    let mut cycle_counter = 0;

    let mut sleep = FreeRtos {};
    let delay = Ets {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    // EVENT LOOP -> FUTURE_USE
    let sysloop = EspSystemEventLoop::take()?;
    warn!("@@@ about to subscribe to the background event loop");
    let _subscription = sysloop.subscribe(move |message: &EventLoopMessage| {
        warn!("@@@ got message from the event loop: {:?} <- uptime",
              message);
    })?;

    // CHANNEL's
    let (display_i2c_sender, display_i2c_receiver) = channel::<RenderSsd>();
    //let (display_spi_sender, display_spi_receiver) = channel::<RenderIli>();
    // MQTT: we can have it directly here without channel, but just to verify
    //  it can be done like that.
    //  downside:
    //    - we clone topic
    //    - we change payload &[u8] -> Vec[u8]
    let (mqtt_client_sender, mqtt_client_receiver) = channel::<MqttPub>();
    // FUTURE USE -> for parsing incomming msg/command
    //let (mqtt_sender, mqtt_receiver) = channel();

    // /*
    // todo! -> something here is slowing down measurement !!!
    // DISPLAY_SPI
    if app_config.flag_display_spi.eq(&true) {
        let mut delay_spi = Ets {};

        warn!("### SPI peripherals");
        let spi = peripherals.spi2; // spi1 doest not impl SpiAnyPins
        
        warn!("### SPI pins");
        // DO NOT USE gpio18 and gpio19 -> will BLOCK USB
        let pin_sclk = peripherals.pins.gpio0; // SCK gpio5/6
        
        let pin_sdo = peripherals.pins.gpio1; // MISO gpio6/7
        let pin_sdi = peripherals.pins.gpio2; // MOSI gpio2/5
        
        let pin_cs = peripherals.pins.gpio9; // DC gpi020
        
        let pin_dc = peripherals.pins.gpio3; // DC gpio3
        let dc = PinDriver::output(pin_dc)?;
        
        let pin_rst = peripherals.pins.gpio4; // RST gpio4
    
        warn!("### SPI backlight");
        let pin_led = peripherals.pins.gpio5; // gpio1 LED pin ???
        let mut backlight = PinDriver::output(pin_led)?;
        /*
        backlight
            .set_drive_strength(esp_idf_hal::gpio::DriveStrength::I10mA)?;
         */
        backlight
            .set_high()?;
        
        let spi_driver_config = esp_idf_hal::spi::config::DriverConfig::new()
            .dma(esp_idf_hal::spi::Dma::Disabled);
        
        let spi_config = esp_idf_hal::spi::config::Config::new()
            .baudrate(
                34u32.MHz().into()
                //80_000_000u32.Hz()
            );
            /*
            .data_mode(
                // embedded_hal
                //embedded_hal::spi::MODE_0
                // embedded-hal-alpha
                embedded_hal_alpha::spi::MODE_0
            );
            */
            //.write_only(true);

        if let Ok(spi_device_driver) = SpiDeviceDriver::new_single(
            //spi: impl Peripheral<P = SPI> + 'd,
            spi,
            
            //sclk: impl Peripheral<P = impl OutputPin> + 'd,
            pin_sclk,
            
            //sdo: impl Peripheral<P = impl OutputPin> + 'd,
            pin_sdo,
            
            //sdi: Option<impl Peripheral<P = impl InputPin + OutputPin> + 'd>,
            Option::<AnyIOPin>::Some(pin_sdi.into()),
            //Option::<AnyIOPin>::None,
            
            //cs: Option<impl Peripheral<P = impl OutputPin> + 'd>,
            Option::<AnyOutputPin>::Some(pin_cs.into()),
            //Option::<AnyOutputPin>::None,
            
            //bus_config: &DriverConfig,
            &spi_driver_config,
            
            //config: &Config
            &spi_config,
        ) {
            warn!("### SPI SpiDeviceDriver.new_single() OK");
            
            let di = display_interface_spi::SPIInterfaceNoCS::new(
                // SPI: Write<u8> 
                spi_device_driver,
                // DC: OutputPin
                dc,
            );

            // MIPIDSI
            let mut rst = esp_idf_hal::gpio::PinDriver::output(pin_rst)?;
            warn!("$$$ SPI display PinDriver RST set_high()");
            rst.set_high()?;

            warn!("### DISPLAY SPI display init");
            let mut display_spi = mipidsi::Builder::ili9341_rgb666(di)
                .init(
                    // DELAY: DelayUs<u32>
                    &mut delay_spi,
                    // RST: OutputPin,
                    Some(rst),
                    //None,
                )?;

            // CLEAR
            warn!("$$$ SPI display clear");
            if let Err(e) = display_spi
                .clear(embedded_graphics::pixelcolor::RgbColor::RED) {
                    error!("display_ili .clear() error: {e:?}")
                }

            // DRAW
            warn!("$$$ SPI set text.draw()");
            if let Err(e) = Text::new(
                //&channel_data.msg,
                &format!("machine: {}", app_config.machine_name),
                //channel_data.point,
                Point::new(100, 100),
                MonoTextStyleBuilder::new()
                    .font(&FONT_6X10)
                    .text_color(embedded_graphics::pixelcolor::RgbColor::YELLOW)
                    .background_color(embedded_graphics::pixelcolor::RgbColor::GREEN)
                    .build(),
            ).draw(&mut display_spi) {
                error!("display_ili .draw() error: {e:?}")
            }
            
            /*
            warn!("### DISPLAY_ILI init");
            if let Err(e) = display_ili::init(di,
                                              &mut delay_spi,
                                              pin_rst,
                                              display_spi_receiver,
            ) {
                error!("%%% Display SPI init error: {e:?}");
            }
            */
            
            /*
            // DISPLAY MSG INIT
            RenderIli {
                msg: format!("machine: {}", app_config.machine_name),
                point: Point::new(100, 100),
                clear: DisplayIliClear::True,
                //delay: Some(2000u32)
                ..Default::default()
            }.draw(&display_spi_sender);
            */
        }
    }
    // */
    
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
    let mut grideye = GridEye::new(i2c_proxy_1,
                                   delay,
                                   Address::Standard,
    );

    // DISPLAY_I2C
    warn!("### DISPLAY_SSD receiver");
    if app_config.flag_display_i2c.eq(&true) {
        display_ssd::init::<shared_bus::I2cProxy<std::sync::Mutex<I2cDriver<'static>>>, I2cError, RenderSsd>(
            i2c_proxy_2,
            display_i2c_receiver,
        );
    }
    
    // DISPLAY MSG INIT
    RenderSsd {
        msg: format!("machine: {}", app_config.machine_name),
        point: Point::new(0, 32),
        clear: DisplaySsdClear::True,
        flush: DisplaySsdFlush::True,
        delay: Some(2000u32)
    }.draw(&display_i2c_sender);
    
    // WIFI
    let nvs_partition = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;
    let _wifi = wifi::wifi(peripherals.modem,
                           sysloop,
                           app_config.wifi_ssid,
                           app_config.wifi_pass,
                           nvs_partition,
    )?;

    // MQTT
    mqtt::init(&app_config,
               &machine_uuid,
               mqtt_client_receiver,
    )?;

    // MQTT PUB: COMMON_LOG - > boot
    if let Err(e) = mqtt_client_sender.send(
        MqttPub::new(
            mqtt::TopicKind::CommonLog,
            // todo!
            format!("{:?} : boot",
                    &[app_config.machine_name,
                      &machine_uuid,
                    ],
            )
                .as_bytes()
        )
    ) {
        error!("error mqtt_client_sender msg_boot: {e:?}")
    }
    
    // TEMPERATURE BOUNDARY
    let mut temperature_max_boundary = sensor_agm::TEMPERATURE_MAX;
    let mut temperature_min_boundary = sensor_agm::TEMPERATURE_MIN; 
    
    // LOOP
    if grideye.power(Power::Wakeup).is_ok() {
        loop {
            cycle_counter += 1;

            // FUTURE USE
            // MQTT PUB: COMMON_LOG - > beep_counter
            if (cycle_counter % BEEP_COUNTER).eq(&0) {
                if let Err(e) = mqtt_client_sender.send(
                    MqttPub::new(
                        mqtt::TopicKind::CommonLog,
                        // todo!
                        format!("{:?} : beep counter / {}",
                                &[app_config.machine_name,
                                  &machine_uuid,
                                ],
                                cycle_counter,
                        )
                            .as_bytes(),
                    )
                ) {
                    error!("error mqtt_client_sender msg_beep_counter: {e:?}")
                }  
            }

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
            let (payload, temperature_min, temperature_max): (Payload<PAYLOAD_LEN>, Temperature, Temperature) = sensor_agm::measure_as_array_bytes(&mut grideye);

            if app_config.flag_show_payload {
                warn!("payload[{}]: {:?}",
                      payload.0.len(),
                      payload.0,
                );
            }
            
            // TEMPERATUE BOUNDARY
            if temperature_max > temperature_max_boundary {
                temperature_max_boundary = temperature_max;
            }

            if temperature_min < temperature_min_boundary {
                temperature_min_boundary = temperature_min;
            }

            // MQTT PUB: payload
            if let Err(e) = mqtt_client_sender.send(
                MqttPub::new(
                    mqtt::TopicKind::Payload,
                    &payload.0
                )
            ) {
                error!("error mqtt_client_sender payload: {e:?}")
            }
            
            // SHOW ARRAY_INDEX
            if app_config.flag_show_array_index {
                sensor_agm::STATIC_ARRAY_FLIPPED_HORIZONTAL
                    .chunks(LEN)
                    .for_each(|chunk| {
                        info!("{:?}", chunk)
                    });
            }

            // DISPLAY HEATMAP
            if app_config.flag_show_heatmap {
                match sensor_agm::payload_to_values(payload.clone()) {
                    Ok(grid_raw) => {
                        let heat_map: Result<HeatMap<Temperature, LEN>, &'static str> = HeatMap::try_from(grid_raw);
                        
                        match heat_map {
                            Ok(m) => {
                                info!("heat_map_display[cycle: {}]:\n\n{m}",
                                      cycle_counter,
                                );
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

            // MSG DISPLAY LOOP

            //
            RenderSsd {
                msg: format!("{cycle_counter}"),
                clear: DisplaySsdClear::True,
                ..Default::default()
            }.draw(&display_i2c_sender);

            /*
            RenderIli {
                msg: format!("{cycle_counter}"),
                point: Point::new(10, 10),
                clear: DisplayIliClear::True,
                ..Default::default()
            }.draw(&display_spi_sender);
            */
            
            //
            RenderSsd {
                msg: format!("max:  {temperature_max:0.02}"),
                point: Point::new(48, 0),
                ..Default::default()
            }.draw(&display_i2c_sender);

            /*
            RenderIli {
                msg: format!("max:  {temperature_max:0.02}"),
                point: Point::new(48, 0),
                ..Default::default()
            }.draw(&display_spi_sender);
            */
            
            //
            RenderSsd {
                msg: format!("min:  {temperature_max:0.02}"),
                point: Point::new(48, 16),
                ..Default::default()
            }.draw(&display_i2c_sender);

            RenderSsd {
                msg: format!("{temperature_min_boundary:0.02} / {temperature_max_boundary:0.02}"),
                point: Point::new(0, 48),
                ..Default::default()
            }.draw(&display_i2c_sender);

            RenderSsd {
                msg: format!("diff: {:0.02}",
                             temperature_max - temperature_min,
                ),
                point: Point::new(48, 32),
                flush: DisplaySsdFlush::True,
                ..Default::default()
            }.draw(&display_i2c_sender);
            
            sleep.delay_ms(app_config.delay_sleep_duration_ms);
        }
    }

    error!("main() Ok -> probably i2c failed, check wires!!!");
    Ok(())
}

