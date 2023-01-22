mod errors;
mod i2c;
mod sensor_agm;

use errors::WrapError;

use sensor_agm::FramerateWrap;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cError;

use grideye::Address;
use grideye::GridEye;
use grideye::Power;

#[allow(unused_imports)]
use std::fmt;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

const SLEEP_DURATION: u16 = 30 * 1000;
const TEMPERATURE_ERROR_VALUE: f32 = 85.0;

//
#[allow(unused_doc_comments)]
fn main() -> Result<(), WrapError<I2cError>> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    info!("### amg8833");

    let machine_boot = EspSystemTime {}.now();
    warn!("machine_uptime: {machine_boot:?}");
    let mut cycle_counter = 0;

    let mut sleep = FreeRtos {};
    let delay = Ets {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    // VALID
    let pin_scl = peripherals.pins.gpio8;
    let pin_sda = peripherals.pins.gpio10;

    /* // INVALID -> pin's swapped
    let pin_scl = peripherals.pins.gpio10; // 8
    let pin_sda = peripherals.pins.gpio8; // 10
    */

    /// just to see type for error imlp
    let i2c: Result<esp_idf_hal::i2c::I2cDriver<'_>, esp_idf_sys::EspError> =
        i2c::config(peripherals.i2c0, pin_sda, pin_scl);

    let mut grideye = GridEye::new(i2c?, delay, Address::Standard);

    if grideye.power(Power::Wakeup).is_ok() {
        loop {
            cycle_counter += 1;

            /// just to see type for error imlp
            let raw_temp: Result<u16, grideye::Error<_>> = grideye.get_device_temperature_raw();

            warn!(
                "[{}] device >>> raw: {:0>16} / temperature: {}°C / status: {}",
                cycle_counter, // since last flash or soft-boot
                format!("{:b}", raw_temp?),
                grideye.get_device_temperature_celsius()?,
                grideye.pixel_temperature_out_ok()?,
            );

            if let Ok(framerate) = grideye.get_framerate() {
                warn!("framerate: Fps{}", FramerateWrap(framerate));
            }

            /// 64 63 62 61 60 59 58 57
            /// 56 55 54 53 52 51 50 49
            /// 48 47 46 45 44 43 42 41
            /// 40 39 38 37 36 35 34 33
            /// 32 31 30 29 28 27 26 25
            /// 24 23 22 21 20 19 18 17
            /// 16 15 14 13 12 11 10  9
            ///  8  7  6  5  4  3  2  1
            let mut heat_map = sensor_agm::HeatMap([[0.0; 8]; 8]);

            info!("array occupies {} bytes", std::mem::size_of_val(&heat_map));

            (0..8_u8).into_iter().for_each(|x| {
                (0..8_u8).into_iter().for_each(|y| {
                    let pixel = (x * 8) + y;

                    ///status
                    if x.eq(&0) && y.eq(&0) {
                        warn!("status: {:?}", grideye.pixel_temperature_out_ok());
                    }
                    
                    /// we don't want to fall only beause a single pixel error
                    let temp = match grideye.get_pixel_temperature_celsius(pixel) {
                        Ok(pixel_temp) => pixel_temp,
                        Err(e) => {
                            /// just to see
                            error!("Error reading pixel x: {x} y: {y} temperature: {e:?}");

                            TEMPERATURE_ERROR_VALUE
                        }
                    };

                    heat_map.0[x as usize][y as usize] = temp;
                })
            });

            info!("heat_map_display:\n\n{heat_map}");

            /// impl csv/... format for heat map picture
            //info!("{heat_map:?}");

            info!("chrrr...\n");
            sleep.delay_ms(SLEEP_DURATION);
        }
    }

    Ok(())
}
