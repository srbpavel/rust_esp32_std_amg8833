use crate::errors;

use errors::WrapError;

use esp_idf_sys::EspError;

use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::FromValueType;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

const I2C_TICK_TYPE: u32 = 100; // study more what should be correct value !!!

//const INVALID: [u8; 5] = [0x3C, 0x3D, 0x68, 0x69, 0x70];
const INVALID: [u8; 4] = [0x3C, 0x68, 0x69, 0x70];
const FILTER_INVALID: bool = true; //false;

// todo!()
// define, display, filter, whoami, single register read, ...
#[allow(unused)]
enum KnownDevice {
    Amg8833Default = 0x69,
    //Amg8833Alternate = 0x68,

    Ssd1306Default = 0x3C,
    Ssd1306Alternate = 0x3D,

    Shtc3 = 0x70,

    ImuAD0 = 0x68,
    //ImuAD1 = 0x69,
}

//
pub fn config<SDA, SCL>(
    i2c: I2C0,
    pin_sda: SDA,
    pin_scl: SCL,
) -> Result<I2cDriver<'static>, EspError>
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

                log::info!("{address:#X} {buffer:?}");
            }
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

//
#[allow(unused)]
pub fn scan_generic<I2C, E>(i2c: &mut I2C) -> Option<Vec<u8>>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
{
    let start = 0x01;
    let end = 0x7F;
    let mut address_list = vec![];

    // without .filter() it will freeze
    //let invalid = [0x3C, 0x3D, 0x68, 0x69, 0x70];
    //let invalid = [];

    log::error!("INVALID_LIST: {:?}",
                INVALID.iter().map(|a| format!("{a:#X} ")).collect::<Vec<String>>().concat());
    
    (start..end)
        .into_iter()
        .filter(|f| if FILTER_INVALID.eq(&true) { // we are filtering
            !INVALID.contains(f)                  // so we filter invalid
        } else { true })                          // not filtering so we pass all
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

                log::info!("{address:#X} {buffer:?}");
            }
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

//
#[allow(unused)]
pub fn scan_shared<I2C, E>(i2c: &mut I2C) -> Option<Vec<u8>>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
{
    let start = 0x01;
    let end = 0x7F;

    // without .filter() it will freeze
    //let invalid = [0x3C, 0x3D, 0x68, 0x69, 0x70];
    //let invalid = [];
    
    let mut address_list = vec![];

    log::error!("INVALID_LIST: {}",
                INVALID.iter().map(|a| format!("{a:#X} ")).collect::<Vec<String>>().concat());
    
    (start..end)
        .into_iter()
        .filter(|f| if FILTER_INVALID.eq(&true) { // we are filtering
            !INVALID.contains(f)                  // so we filter invalid
        } else { true })                          // not filtering so we pass all
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

                log::info!("{address:#X} {buffer:?}");
            }
        });

    if address_list.is_empty() {
        None
    } else {
        Some(address_list)
    }
}

/* // *scratch*
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

/*
// ALPHA
use embedded_hal::i2c::I2c;

#[allow(unused)]
pub fn scan_generic_alpha<I2C>(i2c: &mut I2C) -> Option<Vec<u8>>
where
    I2C: I2c,
{
    let start = 0x01;
    let end = 0x7F;
    let mut address_list = vec![];

    (start..end)
        .into_iter()
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
