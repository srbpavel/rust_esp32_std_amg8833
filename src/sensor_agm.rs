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

impl fmt::Display for WrapFramerate {
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

// L
impl<const N: usize, const L: usize> From<[f32; N]> for HeatMap<f32, L> {
    fn from(array: [f32; N]) -> Self {
        let len = (array.len() as f32).sqrt() as usize;

        //let mut heat_map = HeatMap::<f32, L>([[0f32; L]; L]);
        let mut heat_map = HeatMap([[0f32; L]; L]);
        let mut index = 0;
        
        (0..len as u8).into_iter().for_each(|x| {
            (0..len as u8).into_iter().for_each(|y| {
                heat_map.0[x as usize][y as usize] = array[index];
                
                index += 1;
            })
        });
        
        heat_map
    }
}

/*
// LEN
impl<const N: usize> From<[f32; N]> for HeatMap<f32, LEN> {
    fn from(array: [f32; N]) -> Self {
        let mut heat_map = HeatMap([[0f32; LEN]; LEN]);
        let mut index = 0;
        
        (0..LEN as u8).into_iter().for_each(|x| {
            (0..LEN as u8).into_iter().for_each(|y| {
                heat_map.0[x as usize][y as usize] = array[index];
                
                index += 1;
            })
        });
        
        heat_map
    }
}
*/

/*
// len
//impl<const N: usize> FromN<[f32; N]> for HeatMap<f32, N> {
impl<const N: usize> HeatMap<f32, N> {
    pub fn from_n(array: [f32; N]) -> Self {
        let len = array.len();
        let mut heat_map = HeatMap([[0f32; N]; N]);
        let mut index = 0;
        
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

//
pub fn measure<I2C, D, E>(grideye: &mut GridEye<I2C, D>) -> ([f32; LEN * LEN], f32, f32)
//pub fn measure<I2C, D, E, const SQRT: usize>(grideye: &mut GridEye<I2C, D>) -> ([f32; SQRT], f32, f32)
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
    E: Debug,
{
    let mut grid_raw = [0.; LEN * LEN];
    //let mut grid_raw: [f32; SQRT ] = [0f32; SQRT];
    let mut max_temperature = -55.;
    let mut min_temperature = 125.;
    
    (0..(LEN*LEN as usize) as u8).into_iter().for_each(|pixel_index| {
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
                                            
    /*
    let mut pixel_index = 0;    
    
    (0..LEN as u8).into_iter().for_each(|x| {
        (0..LEN as u8).into_iter().for_each(|y| {
            let pixel = (x * LEN as u8) + y;
            
            // we don't want to fall only beause a single pixel error
            let temp = match grideye.get_pixel_temperature_celsius(pixel) {
                Ok(pixel_temp) => pixel_temp,
                Err(e) => {
                    error!("Error reading pixel x: {x} y: {y} temperature: {e:?}");
                    
                    TEMPERATURE_ERROR_VALUE
                }
            };
            
            grid_raw[pixel_index as usize] = temp;
            pixel_index += 1;

            if temp > max_temperature { max_temperature = temp }
            if temp < min_temperature { min_temperature = temp }
        })
    });
    */

    (grid_raw, min_temperature ,max_temperature)
}

/*
//
//pub fn array_to_map<const N: usize>(array: [f32; N]) -> HeatMap<f32, N> {
#[allow(unused)]
pub fn array_to_map<const N: usize>(array: [f32; N]) -> HeatMap<f32, 8> {
    //let len = 8; //array.len(); as 64 not 8
    let len = (array.len() as f32).sqrt() as usize;

    //let mut heat_map = HeatMap::<f32, N>([[0f32; N]; N]);
    let mut heat_map = HeatMap([[0f32; 8]; 8]);
    let mut index = 0;
    
    (0..len as u8).into_iter().for_each(|x| {
        (0..len as u8).into_iter().for_each(|y| {
            //heat_map.0[x as usize][y as usize] = array[index];
            heat_map.0[x as usize][y as usize] = array[index];

            /*
            heat_map.0[x as usize][y as usize] = match array.get(index) {
                Some(pixel) => {*pixel},
                None => {85f32},
            };
            */

            index += 1;
        })
    });
    
    heat_map
}
*/

#[allow(unused)]
pub fn array_to_map_with_len<const N: usize, const L: usize>(array: [f32; N],
                                                             //len: usize,
) -> HeatMap<f32, L> {
    let len = (array.len() as f32).sqrt() as usize;
    
    let mut heat_map = HeatMap::<f32, L>([[0f32; L]; L]);
    //let mut heat_map = HeatMap([[0f32; L]; L]);
    let mut index = 0;
    
    (0..len as u8).into_iter().for_each(|x| {
        (0..len as u8).into_iter().for_each(|y| {
            heat_map.0[x as usize][y as usize] = array[index];
            index += 1;
        })
    });
    
    heat_map
}

/*
//
#[allow(unused)]
pub fn array_to_map_result<const N: usize, const L: usize>(array: [f32; N],
) -> Result<HeatMap<f32, L>, String> {
    let len = (array.len() as f32).sqrt() as usize;
    
    let mut heat_map = HeatMap::<f32, L>([[0f32; L]; L]);
    let mut index = 0;
    
    (0..len as u8).into_iter().for_each(|x| {
        (0..len as u8).into_iter().for_each(|y| {
            heat_map.0[x as usize][y as usize] = array[index];
            index += 1;
        })
    });
    
    //Ok::<HeatMap<f32, L>, String>(heat_map)
    Ok(heat_map)
}
*/

