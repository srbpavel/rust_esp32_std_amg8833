use esp_idf_hal::i2c::I2cError;
use esp_idf_sys::EspError;
use grideye::Error as GridEyeError;
use display_interface::DisplayError;

#[derive(Debug)]
pub enum WrapError<E> {
    WrapEspError(EspError),
    WrapI2cError(I2cError),
    WrapGridEyeError(GridEyeError<E>),
    // will return BusWriteError if display not found
    WrapDisplayError(DisplayError),
}

impl<E> From<DisplayError> for WrapError<E> {
    fn from(error: DisplayError) -> Self {
        Self::WrapDisplayError(error)
    }
}
    
impl<E> From<EspError> for WrapError<E> {
    fn from(error: EspError) -> Self {
        Self::WrapEspError(error)
    }
}

impl<E> From<I2cError> for WrapError<E> {
    fn from(error: I2cError) -> Self {
        Self::WrapI2cError(error)
    }
}

impl<E> From<GridEyeError<E>> for WrapError<E> {
    fn from(error: GridEyeError<E>) -> Self {
        Self::WrapGridEyeError(error)
    }
}
