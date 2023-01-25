use esp_idf_hal::i2c::I2cConfig; // TypeDefinition
use esp_idf_hal::i2c::I2cDriver; // Struct
use esp_idf_hal::i2c::I2C0; //
use esp_idf_hal::units::FromValueType;

use esp_idf_hal::gpio::InputPin;
use esp_idf_hal::gpio::OutputPin;

use shared_bus;
use shared_bus::BusManager;
use shared_bus::NullMutex;

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

    let i2c_driver = I2cDriver::new(i2c, pin_sda, pin_scl, &i2c_config);

    //todo!()
    // list all addresses

    i2c_driver
        
}

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
