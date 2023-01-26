use crate::errors;

use errors::WrapError;

//use esp_idf_hal::i2c::I2cError;
use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::FromValueType;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

// /*
use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;
// */
// alpha
//use embedded_hal::i2c::I2c;
    
//use shared_bus::I2cProxy;
//use std::sync::Mutex;

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

//
pub fn scan(i2c: &mut I2cDriver<'_>) -> Option<Vec<u8>> {
//pub fn scan(i2c: &mut I2cProxy<'_, Mutex<I2cDriver<'static>>>) -> Option<Vec<u8>> {
/*
pub fn scan<I2C, E>(i2c: &mut I2C) -> Option<Vec<u8>>
//pub fn scan<I2C>(i2c: &mut I2C) -> Option<Vec<u8>>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    // alpha
    //I2C: I2c,
{
*/
    //let address = 0x3C; // ssd1306 default
    //let address = 0x3D; // ssd1306 alternate
    //let address = 0x69; // grideye standard
    //let address = 0x68; // grideye alternate
    //let address = 0x70; // grideye ???

    let start = 0x01; // 0x01; // 0x08
    let end = 0x7F; // 0x7F; // 0x77
    let mut address_list = vec![];

    (start..end)
        .into_iter()
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
                .map_err(WrapError::I2c);
                
            if read_result.is_ok() {
                address_list.push(address);

                // /*
                log::warn!("{address:#X} {buffer:?}");
                // */
            }
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

//
pub fn scan_shared<I2C, E>(i2c: &mut I2C) -> Option<Vec<u8>>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
{
    let start = 0x01;
    let end = 0x7F;

    // without .filter() it will freeze
    let invalid = [0x3C, 0x3D, 0x68, 0x69, 0x70];
    //let invalid = [0x70];
    //let invalid = [];
    
    let mut address_list = vec![];

    (start..end)
        .into_iter()
        .filter(|f| {
            !invalid.contains(f)
        })
        .for_each(|address| {
            /*
            log::warn!("going to i2c_write: {address:#X}");
            let write_result = i2c
                .write(address as u8, &[])
                .map_err(WrapError::I2c);

            if write_result.is_ok() {
                address_list.push(address);

                log::warn!("device at {address:#X}");
            }
            */

            // /*
            let mut buffer = [0, 0];
            
            let read_result = i2c
                .read(
                    address,
                    &mut buffer,
                )
                .map_err(WrapError::I2c);
                
            if read_result.is_ok() {
                address_list.push(address);

                log::warn!("{address:#X} {buffer:?}");
            }
            // */
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

/*
//
pub fn scan_i2c_shared<I2C, E>(i2c: &mut I2C) -> Option<Vec<u8>>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
{
    let start = 0x01;
    let end = 0x7F;

    let invalid = [0x3C, 0x3D, 0x68, 0x69, 0x70];
    //let invalid = [];
    
    let mut address_list = vec![];

    (start..end)
        .into_iter()
        .filter(|f| {
            !invalid.contains(f)
        })
        .for_each(|address| {
            let mut buffer = [0, 0];
            
            let read_result = i2c
                .read(
                    address,
                    &mut buffer,
                )
                .map_err(WrapError::I2c);
            
            if read_result.is_ok() {
                address_list.push(address);
                
                log::warn!("{address:#X} {buffer:?}");
            }
        });
    
    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}
*/
