mod i2c;
mod sensor_agm;
mod errors;

use errors::WrapError;

use sensor_agm::FramerateWrap;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;

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

#[allow(unused_imports)]
use esp_idf_hal::i2c::I2cError;
#[allow(unused_imports)]
use esp_idf_sys::EspError;

//fn main() -> anyhow::Result<()> {
//fn main() -> anyhow::Result<(), WrapError> {
//fn main() -> Result<(), WrapError> {
fn main() -> Result<(), WrapError<esp_idf_hal::i2c::I2cError>> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    info!("### amg8833");

    let machine_boot = EspSystemTime {}.now();
    warn!("machine_uptime: {machine_boot:?}");
    let mut cycle_counter = 0;
    
    let mut sleep = FreeRtos {};
    let delay = Ets {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();
    let pin_scl = peripherals.pins.gpio8;
    let pin_sda = peripherals.pins.gpio10;

    let i2c: Result<esp_idf_hal::i2c::I2cDriver<'_>, EspError> =
        i2c::config(peripherals.i2c0, pin_sda, pin_scl);//?;

    let mut grideye = GridEye::new(i2c?, delay, Address::Standard);

    if grideye.power(Power::Wakeup).is_ok() {
        loop {
            cycle_counter += 1;

            // grideye::Error
            //
            // #[derive(Debug)]
            // pub enum Error<E> {
            //    /// I2C bus error
            //    I2c(E),
            // }
            let raw_temp: Result<u16, grideye::Error<_>> =
                grideye.get_device_temperature_raw();

            warn!(
                "[{}] device >>> raw: {:0>16} / temperature: {:?}",
                cycle_counter,

                //format!("{:b}", raw_temp.unwrap()),
                format!("{:b}", raw_temp?),

                grideye.get_device_temperature_celsius(),
            );

            if let Ok(framerate) = grideye.get_framerate() {
                warn!("framerate: Fps{}", FramerateWrap(framerate));
            }

            let mut heat_map = sensor_agm::HeatMap([[0.0; 8]; 8]);

            info!("array occupies {} bytes", std::mem::size_of_val(&heat_map));

            (0..8_u8).into_iter().for_each(|x| {
                (0..8_u8).into_iter().for_each(|y| {
                    let pixel = (x * 8) + y;

                    let temp = grideye.get_pixel_temperature_celsius(pixel).unwrap();

                    heat_map.0[x as usize][y as usize] = temp;
                })
            });

            info!("heat_map_display:\n\n{heat_map}");

            //info!("{heat_map:?}");

            info!("chrrr...\n");
            sleep.delay_ms(SLEEP_DURATION);
        }
    }

    Ok(())
}
