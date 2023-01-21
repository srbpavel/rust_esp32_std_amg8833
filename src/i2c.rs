//I2C
#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::*; //https://esp-rs.github.io/esp-idf-hal/esp_idf_hal/units/index.html

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

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
    let i2c_config = I2cConfig::new().baudrate(
        400.kHz() // 100 in newer code somewhere or doc ???
            .into(),
    );

    I2cDriver::new(i2c, pin_sda, pin_scl, &i2c_config)
}
