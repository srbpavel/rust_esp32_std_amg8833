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

const TEMPERATURE_ERROR_VALUE: f32 = 85.0;
pub const LEN: usize = 8; // array column/row size

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

#[derive(Debug)]
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
        let cell = LEN * 6;
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

/* // From and TryFrom cannot be impl at same time !!!
//  
impl<const N: usize, const L: usize> From<[f32; N]> for HeatMap<f32, L> {
    fn from(array: [f32; N]) -> Self {
        let mut heat_map = HeatMap::<f32, L>([[0f32; L]; L]);
        let mut index = 0;

        let len = (array.len() as f32).sqrt() as usize;
        
        (0..len as u8).into_iter().for_each(|x| {
            (0..len as u8).into_iter().for_each(|y| {
                heat_map.0[x as usize][y as usize] = array[index];
                
                index += 1;
            })
        });
        
        heat_map
    }
}
*/

//
//impl<const N: usize, const L: usize> TryFrom<[f32; N]> for HeatMap<f32, L> {
impl<const N: usize, const L: usize> TryFrom<[f32; N]> for HeatMap<f32, L> {
    //type Error = &'static String;
    type Error = &'static str;
    //type Error = std::convert::Infallible;
    
    fn try_from(array: [f32; N]) -> Result<Self, Self::Error> {
        let mut heat_map = HeatMap::<f32, L>([[TEMPERATURE_ERROR_VALUE; L]; L]);
        let mut index = 0;

        //let len = (array.len() as f32).sqrt() as usize;
        let sqrt = (array.len() as f32).sqrt();
        let len = if !sqrt.eq(&sqrt.floor()) {
            return Err("error try_from 1d array not square")
        } else {
            sqrt as usize
        };
        
        (0..len as u8).into_iter().for_each(|x| {
            (0..len as u8).into_iter().for_each(|y| {
                if let Some(i) = array.get(index) {
                    if let Some(x_row) = heat_map.0.get(x as usize) {
                        if x_row.get(y as usize).is_some() {
                            heat_map.0[x as usize][y as usize] = *i;
                        }
                    }
                }
                
                index += 1;
            })
        });

        Ok(heat_map)
    }
}
/*
impl<const T: usize,U> TryFrom<U> for HeatMap<f32, T>
where
    U: Into<HeatMap<f32, T>>,
{
    type Error = &'static str;

    fn try_from(array: U) -> Result<Self, Self::Error> {
        Err("try_from_err")
    }
}
*/

//
pub fn measure<I2C, D, E>(grideye: &mut GridEye<I2C, D>) -> ([f32; LEN * LEN], f32, f32)
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
    E: Debug,
{
    let mut grid_raw = [0_f32; LEN * LEN];
    let mut max_temperature = -55.;
    let mut min_temperature = 125.;
    
    (0..(LEN*LEN) as u8).into_iter().for_each(|pixel_index| {
        let temp = match grideye.get_pixel_temperature_celsius(pixel_index) {
            Ok(pixel_temp) => pixel_temp,
            Err(e) => {
                error!("Error reading pixel: {pixel_index} temperature: {e:?}");
                
                TEMPERATURE_ERROR_VALUE
            }
        };
        
        grid_raw[pixel_index as usize] = temp;
        
        if temp > max_temperature { max_temperature = temp }
        if temp < min_temperature { min_temperature = temp }
    });

    (grid_raw, min_temperature ,max_temperature)
}

#[allow(unused)]
pub fn array_to_map<const N: usize, const L: usize>(array: [f32; N]) -> HeatMap<f32, L> {
    let len = (array.len() as f32).sqrt() as usize;
    
    let mut heat_map = HeatMap::<f32, L>([[0_f32; L]; L]);
    let mut index = 0;
    
    (0..len as u8).into_iter().for_each(|x| {
        (0..len as u8).into_iter().for_each(|y| {
            heat_map.0[x as usize][y as usize] = array[index];
            index += 1;
        })
    });
    
    heat_map
}
