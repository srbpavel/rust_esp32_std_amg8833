use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;

use std::ops;

use grideye::Framerate;
use grideye::GridEye;

use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

use embedded_hal::blocking::delay::DelayMs;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

const TEMPERATURE_ERROR_VALUE: f32 = 85_f32;
const TEMPERATURE_MAX: f32 = -55_f32;
const TEMPERATURE_MIN: f32 = 125_f32;
pub const LEN: usize = 8; // array column/row size
pub const POW: usize = LEN * LEN;

pub type Temperature = f32;

pub struct WrapFramerate(pub Framerate);

impl Display for WrapFramerate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}",
            match self.0 {
                Framerate::Fps10 => 10,
                Framerate::Fps1 => 1,
            }
        )
    }
}

type HeatArray<T, const N: usize> = [[T; N]; N];

#[derive(Debug, PartialEq)]
pub struct HeatMap<T, const N: usize>(pub HeatArray<T, N>);

impl<T, const N: usize> ops::Deref for HeatMap<T, N> {
    type Target = HeatArray<T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> Display for HeatMap<T, N>
where
    T: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut line = 0;
        let cell = N * 6;
        let blank_line = format!("\n*{}*", " ".repeat((cell) + 3));

        self.iter().fold(Ok(()), |result, row| {
            result.and_then(|_| {
                line += 1;

                let first = if line.eq(&1) {
                    format!("{}*{blank_line}\n", "* ".repeat(((cell) + 5) / 2))
                } else {
                    String::from("")
                };

                let last = if line.eq(&self.len()) {
                    format!("\n{}*", "* ".repeat(((cell) + 5) / 2))
                } else {
                    String::from("")
                };

                writeln!(
                    f,
                    "{first}* {}  *{blank_line}{last}",
                    row.iter()
                        .map(|t| { format!(" {t:0.02}") })
                        .collect::<String>(),
                )
            })
        })
    }
}

//
// N is array len, N = LEN * LEN
// L is LEN aka ROW/COLUMN size
impl<const N: usize, const L: usize> TryFrom<[Temperature; N]> for HeatMap<Temperature, L> {
    type Error = &'static str;
    
    fn try_from(array: [Temperature; N]) -> Result<Self, Self::Error> {
        let mut heat_map = HeatMap::<Temperature, L>([[TEMPERATURE_ERROR_VALUE; L]; L]);
        let mut index = 0;

        let sqrt = (N as f32).sqrt();
        let len = if sqrt.eq(&sqrt.floor()) {
            sqrt as usize
        } else {
            return Err("error try_from 1d array to square")
        };

        (0..len as u8).into_iter().for_each(|x| {
            (0..len as u8).into_iter().for_each(|y| {
                if let Some(pixel) = array.get(index) {
                    if let Some(x_row) = heat_map.0.get(x as usize) {
                        if x_row.get(y as usize).is_some() {
                            heat_map.0[x as usize][y as usize] = *pixel;
                        }
                    }
                }
                
                index += 1;
            })
        });

        Ok(heat_map)
    }
}

//
// L is output array len
pub fn measure<I2C, D, E, const L: usize>(grideye: &mut GridEye<I2C, D>) -> ([Temperature; L], Temperature, Temperature)
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
    E: Debug,
{

    let mut grid_raw = [TEMPERATURE_ERROR_VALUE; L];
    let mut max_temperature = TEMPERATURE_MAX;
    let mut min_temperature = TEMPERATURE_MIN;

    (0..L as u8).into_iter().for_each(|pixel_index| {
        if let Ok(pixel_temp) = grideye.get_pixel_temperature_celsius(pixel_index) {
            if let Some(pixel) = grid_raw.get_mut(pixel_index as usize) {
                *pixel = pixel_temp;
            }

            if pixel_temp > max_temperature { max_temperature = pixel_temp }
            if pixel_temp < min_temperature { min_temperature = pixel_temp }
        }
    });

    (grid_raw, min_temperature ,max_temperature)
}
