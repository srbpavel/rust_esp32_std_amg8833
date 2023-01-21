use rust_esp32_std_amg8833::i2c;
use rust_esp32_std_amg8833::sensor_agm;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::reset::WakeupReason;

use grideye::Address;
use grideye::GridEye;
use grideye::Power;

//use std::time::Instant;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

const SLEEP_DURATION: u16 = 30 * 1000;

//
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    let wakeup_reason = WakeupReason::get();

    info!("### amg8833");

    let machine_boot = EspSystemTime {}.now();
    warn!("machine_uptime: {machine_boot:?} wakeup: {wakeup_reason:?}");

    let mut sleep = FreeRtos {};
    let delay = Ets {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();
    let pin_scl = peripherals.pins.gpio8;
    let pin_sda = peripherals.pins.gpio10;
    let i2c = i2c::config(peripherals.i2c0, pin_sda, pin_scl)?;

    let mut grideye = GridEye::new(i2c, delay, Address::Standard);

    if grideye.power(Power::Wakeup).is_ok() {
        // LOOP
        loop {
            let raw_temp: Result<u16, grideye::Error<_>> = grideye.get_device_temperature_raw();

            warn!(
                "device >>> raw: {:0>16} / temperature: {:?}",
                format!("{:b}", raw_temp.unwrap()),
                grideye.get_device_temperature_celsius(),
            );

            let mut heat_map = sensor_agm::HeatMap([[0.0; 8]; 8]);

            info!("array occupies {} bytes", std::mem::size_of_val(&heat_map));

            //let start = Instant::now();
            (0..8_u8).into_iter().for_each(|x| {
                (0..8_u8).into_iter().for_each(|y| {
                    let pixel = (x * 8) + y;

                    let temp = grideye.get_pixel_temperature_celsius(pixel).unwrap();

                    heat_map.0[x as usize][y as usize] = temp;
                })
            });
            //let end = Instant::now();
            //warn!("measure duration -> {:?}", end.duration_since(start));

            /*
            // by index via .get()
            for x in 0..heat_map.len() + 1 { // OOPS, one element too far
                match heat_map.get(x) {
                    Some(row) => {
                        info!("[{x}]: {row:?}");

                        for y in 0..row.len() + 1 { // OOPS again
                            match row.get(y) {
                                Some(pixel) => {
                                    info!("[{x}, {y}] {pixel:?}");
                                },
                                None => error!("Error {} is too far!", y),
                            }
                        };
                    },
                    None => error!("Error {} is too far!", x),
                }
            }
            */

            info!("heat_map_display:\n\n{heat_map}");

            info!("chrrr...");
            sleep.delay_ms(SLEEP_DURATION);
        }
    }

    Ok(())
}
