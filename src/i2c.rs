use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::FromValueType;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

/*
use shared_bus;
use shared_bus::BusManager;
use shared_bus::NullMutex;
*/

//use embedded_hal::prelude::_embedded_hal_blocking_i2c_Write;
//use embedded_hal::prelude::_embedded_hal_blocking_i2c_Read;

use log;

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
        .baudrate(400.kHz() // 100 in newer code somewhere or doc ???
                  .into(),
        );

    I2cDriver::new(i2c, pin_sda, pin_scl, &i2c_config)
}

/*
//
pub fn config_shared<SDA, SCL>(
    i2c: I2C0,
    pin_sda: SDA,
    pin_scl: SCL,
) ->  Result<BusManager<NullMutex<I2cDriver<'static>>>, esp_idf_sys::EspError> 
where
    SDA: OutputPin + InputPin,
    SCL: OutputPin + InputPin,
{
    Ok(
        shared_bus::BusManagerSimple::new(
            config(i2c,
                   pin_sda,
                   pin_scl,
            )?
        )
    )
}
*/

//
pub fn scan(i2c: &mut esp_idf_hal::i2c::I2cDriver<'_>) {
    //let address = 0x3C; // ssd1306 default
    //let address = 0x3D; // ssd1306 alternate
    //let address = 0x69; // grid standard
    //let address = 0x68; // grid alternate
    //let address = 0x70; // grid ???

    let register = 0x00;
    let cmd = [register];
    let start = 0x01; // 0x01; // 0x08
    let end = 0x7F;   // 0x7F; // 0x77
    
    let invalid = [];
    //let invalid = [0x3C, 0x3D, 0x68, 0x69, 0x70];
    
    let mut index = 0;                                 
                                                   
    (start..end)
        .into_iter()                                   
        .filter(|f| {
            !invalid.contains(f)
        })
        .for_each(|address| {                          
            let mut buffer = [0, 0];

            /*
            //warn!("going to i2c_write");
            let write_result = i2c_proxy_3
                .write(address as u8, &cmd)
                .map_err(grideye::Error::I2c);
            warn!("i2c_write_result: {write_result:?}");
            */

            //warn!("going to i2c_read");
            let read_result = i2c
                .read(address as u8,
                      &mut buffer,
                      I2C_TICK_TYPE,
                )
                .map_err(grideye::Error::I2c);
            //warn!("i2c_read_result: {read_result:?}");
            
            if let Ok(_) = read_result {
                log::warn!("[{index}]: {address:X} {:0>16}", 
                      format!("{:b}", address),         
                );                                         

                log::error!("{register:b} {cmd:?} {address:x} {buffer:?}");
            }

            index += 1;
        });
}
