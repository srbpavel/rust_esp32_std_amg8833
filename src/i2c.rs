use crate::errors;

use errors::WrapError;

use esp_idf_hal::i2c::I2cError;
use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::FromValueType;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

//use log;

const I2C_TICK_TYPE: u32 = 100; // study more what should be correct value !!!

//
pub fn config<SDA, SCL>(
    i2c: I2C0,
    pin_sda: SDA,
    pin_scl: SCL,
) -> Result<I2cDriver<'static>, esp_idf_sys::EspError>
where
    SDA: OutputPin + InputPin,
    SCL: OutputPin + InputPin,
{
    let i2c_config = I2cConfig::new()
        // baudrate is in Hertz
        .baudrate(
            400.kHz() // 100 in newer code somewhere or doc ???
                .into(),
        );

    I2cDriver::new(i2c, pin_sda, pin_scl, &i2c_config)
}

pub fn scan(i2c: &mut esp_idf_hal::i2c::I2cDriver<'_>) -> Option<Vec<u8>> {
    //let address = 0x3C; // ssd1306 default
    //let address = 0x3D; // ssd1306 alternate
    //let address = 0x69; // grideye standard
    //let address = 0x68; // grideye alternate
    //let address = 0x70; // grideye ???

    let start = 0x01; // 0x01; // 0x08
    let end = 0x7F; // 0x7F; // 0x77
    //let invalid = [];
    //let mut index = 0;
    let mut address_list = vec![];

    (start..end)
        .into_iter()
        /*
        .filter(|f| {
            !invalid.contains(f)
        })
        */
        .for_each(|address| {
            let mut buffer = [0, 0];

            /*
            let register = 0x00;
            let cmd = [register];
            
            log::warn!("going to i2c_write: {address:#X}");
            let write_result = i2c
                .write(address,
                       &cmd,
                       I2C_TICK_TYPE,
                )
                .map_err(grideye::Error::I2c);
            log::warn!("i2c_write_result: {write_result:?}");
            */

            let read_result = i2c
                .read(
                    address,
                    &mut buffer,
                    I2C_TICK_TYPE,
                )
                .map_err(WrapError::<I2cError>::WrapEspError);
                
            if read_result.is_ok() {
                address_list.push(address);

                // /*
                log::warn!("{address:#X} {buffer:?}");
                // */
            }

            //index += 1;
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

/*
use shared_bus::NullMutex;
use shared_bus::I2cProxy;
use embedded_hal::prelude::_embedded_hal_blocking_i2c_Read;
*/

use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

//
pub fn scan_shared<I2C, E>(i2c: &mut I2C,
//pub fn scan_shared<I2C, E>(i2c: I2C,
) -> Option<Vec<u8>>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E> + Clone,
{
    let start = 0x01;
    let end = 0x7F;
    let mut address_list = vec![];

    let mut i2c = i2c.clone();
    
    (start..end)
        .into_iter()
        .for_each(|address| {
            let mut buffer = [0, 0];

            // /*
            log::warn!("going to i2c_read: {address:#X}");
            let read_result = i2c
                .read(
                    address,
                    &mut buffer,
                    //I2C_TICK_TYPE,
                )
                .map_err(WrapError::I2c);
                
            if read_result.is_ok() {
                address_list.push(address);

                // /*
                log::warn!("{address:#X} {buffer:?}");
                // */
            }
            // */
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

