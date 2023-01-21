//mod i2c;
use rust_esp32_std_amg8833::i2c;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;

use grideye::GridEye;
use grideye::Address;
use grideye::Power;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

const SLEEP_DURATION: u16 = 15 * 1000;
//const WTD_FEEDER_DURATION: u16 = 100;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    info!("### amg8833");

    let machine_boot = EspSystemTime {}.now();
    warn!("duration since machine_boot: {machine_boot:?}");

    let mut sleep = FreeRtos {};
    let delay = Ets {}; // mut FreeRtos {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();
    // PINS
    let pin_scl = peripherals.pins.gpio8;
    let pin_sda = peripherals.pins.gpio10;
    // PERIPHERAL
    let i2c = peripherals.i2c0;

    if let Some(i2c) = i2c::config(i2c, pin_sda, pin_scl) {

        let mut grideye = GridEye::new(
            i2c,
            delay,
            Address::Standard,
        );
        
        grideye.power(Power::Wakeup).unwrap();
        /*
        grideye.power(Power::Sleep).unwrap();
        grideye.power(Power::Standby60Seconds).unwrap();
        grideye.power(Power::Standby10Seconds).unwrap();
        */
        
        // get the device temperature

        //
        // pub fn get_device_temperature_raw(&mut self) -> Result<u16, Error<E>> {
        //     self.get_register_as_u16(Register::ThermistorLsb as u8)
        // }
        //
        // fn get_register_as_u16(&mut self,
        //                        register: u8,
        // ) -> Result<u16, Error<E>> {
        // let cmd = [register];
        //     self.i2c
        //         .write(self.address as u8, &cmd)
        //         .map_err(Error::I2c)?;
        //     let mut buffer = [0, 0];
        //     self.i2c
        //         .read(self.address as u8, &mut buffer)
        //         .map_err(Error::I2c)?;
        //     Ok(((buffer[1] as u16) << 8) + (buffer[0] as u16))
        // }
        let raw_temp: Result<u16, grideye::Error<_>> = grideye.get_device_temperature_raw();
        
        info!(
            "device raw: {:?} / temperature: {:?}",
            raw_temp,
            grideye.get_device_temperature_celsius(),
        );

        /*
        // framerate
        info!(
            "device frame_rateraw: {:?}",
            grideye.get_framerate(),
        );
        */
        
        // LOOP
        loop {
            info!("{}", std::iter::repeat("*").take(32).collect::<String>());

            let mut heat_map: [[f32; 8]; 8] = [[0.0; 8]; 8];

            info!("array occupies {} bytes", std::mem::size_of_val(&heat_map));
            
            (0..8_u8)
                .into_iter()
                .for_each(|x| {
                    (0..8_u8)
                        .into_iter()
                        .for_each(|y| {
                            let pixel = (x * 8) + y;
                            
                            //
                            // pub fn get_pixel_temperature_raw(&mut self,
                            //                                  pixel: u8,
                            // ) -> Result<u16, Error<E>> {
                            //     let pixel_low =
                            //         Register::TemperatureStart as u8 + (2 * pixel);
                            //     self.get_register_as_u16(pixel_low)
                            // }
                            // fn get_register_as_u16(&mut self,
                            //                        register: u8,
                            // ) -> Result<u16, Error<E>> {
                            //     let cmd = [register];
                            //     self.i2c
                            //         .write(self.address as u8, &cmd)
                            //         .map_err(Error::I2c)?;
                            //     let mut buffer = [0, 0];
                            //     self.i2c
                            //         .read(self.address as u8, &mut buffer)
                            //         .map_err(Error::I2c)?;
                            //     Ok(((buffer[1] as u16) << 8) + (buffer[0] as u16))
                            // }
                            let temp = grideye
                                .get_pixel_temperature_celsius(pixel)
                                .unwrap();

                            heat_map[x as usize][y as usize] = temp;
                        })
                });

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
            
            info!("heat_map: {heat_map:#?}");
            
            /*
            for x in 0..8 {
                for y in 0..8 {
                    let pixel = (x * 8) + y;

                    let _raw_temp: Result<u16, grideye::Error<_>> = grideye
                        .get_pixel_temperature_raw(pixel);
                    
                    // temperature f32
                    let temp = grideye
                        .get_pixel_temperature_celsius(pixel)
                        .unwrap();
                    
                    info!("{}", temp);
                    if y < 7 {
                        info!(";");
                    }
                }
                info!("_");
            }
            */
            
            sleep.delay_ms(SLEEP_DURATION);
        }
    }
    
    Ok(())
}
