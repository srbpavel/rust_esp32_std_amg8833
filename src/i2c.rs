//I2C
#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::*; //https://esp-rs.github.io/esp-idf-hal/esp_idf_hal/units/index.html

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

//
pub fn config<SDA, SCL>(i2c: I2C0,
                        pin_sda: SDA,
                        pin_scl: SCL,
                        
) -> Option<I2cDriver<'static>>
where                          
                               SDA: OutputPin + InputPin,
                               SCL: OutputPin + InputPin, 
{
    
    warn!("### I2C pins via CONF >>> sda: {:?} scl: {:?}",
          pin_sda.pin(),
          pin_scl.pin(),
    );
            
    let i2c_config = I2cConfig::new()
        .baudrate(400.kHz() // 100 in newer code somewhere or doc ???
                  .into()
        );
    
    match I2cDriver::new(i2c,
                         pin_sda,
                         pin_scl,
                         &i2c_config,
            ) {
        Ok(i2c) => Some(i2c),
        Err(e) => {
            error!("### i2c Driver Error: {:?}", e);
            
            None
        },
    }
}
