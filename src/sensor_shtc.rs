use std::sync::Arc;
use std::sync::Mutex;

use std::fmt::Debug;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::delay::DelayUs;

use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

use shtcx::PowerMode;
use shtcx::ShtC3;
use shtcx::Measurement;

use log::info;
use log::error;

//
pub fn measure<I2C, E, D>(sensor: Option<Arc<Mutex<ShtC3<I2C>>>>,
               delay: &mut D,
               delay_duration: u16,
) -> Option<Measurement>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayUs<u16> + DelayMs<u16>,
    E: Debug,
{

    match sensor {
        Some(sensor) => {
            read_shtc(sensor,
                      delay,
                      delay_duration,
            )
        },
        None => {
            error!("### nothing to_publish as Peripheral i2c Error");

            None
        },
    }
}

//
fn read_shtc<I2C, E, D>(sensor: Arc<Mutex<ShtC3<I2C>>>,
                        delay: &mut D,
                        delay_duration: u16,
) -> Option<Measurement>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayUs<u16> + DelayMs<u16>,
    E: Debug,
{
    
    match sensor
        .lock() {
            Ok(mut sensor) => {
                match sensor.device_identifier() {
                    Ok(device_id) => {
                        info!("### SHTC3 -> Device ID: {:?}", device_id);

                        match sensor
                            .start_measurement(PowerMode::NormalMode)
                        {
                            Ok(()) => {

                                delay.delay_ms(delay_duration);
                                
                                match sensor
                                    .get_measurement_result() {
                                        Ok(measurement) => {
                                            Some(measurement)
                                        },
                                        Err(e) => {
                                            error!("### SHTC3 -> Get Measurement Result Error: '{:?}'", e);
                                            None
                                        },
                                    }
                            },
                            Err(e) => {
                                error!("### SHTC3 -> Start Measurement Error: '{:?}'", e);
                                None
                            },
                        }
                    },
                    Err(err) => {
                        error!("### SHTC3 -> Device ID Error: '{:?}'", err);
                        
                        None
                    },
                }
            },
            Err(_) => {
                error!("### SHTC3 -> unlock Arc error");

                None
            },
        }
}
