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

pub const TEMPERATURE_MAX: f32 = -55_f32;
pub const TEMPERATURE_MIN: f32 = 125_f32;
pub const LEN: usize = 8; // array column/row size
pub const POW: usize = LEN * LEN;
pub const PAYLOAD_LEN: usize = 4 * POW;

/*
#[allow(unused)]
pub const STATIC_ARRAY: [u8; POW] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63];
*/

#[allow(unused)]
pub const STATIC_ARRAY_FLIPPED_HORIZONTAL: [u8; POW] = [56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46, 47, 32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21, 22, 23, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7];

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
                        .map(|t| format!(" {t:02.02}"))
                        .collect::<String>(),
                )
            })
        })
    }
}

pub struct Payload<const N: usize> ([u8; N]);

// struct RawVec (Vec<u8>);

/*
impl<const N: usize> From<RawVec> for Payload<N> {
    fn from(raw_vec: RawVec) -> Self {
        let mut array: [u8; N] = [0 as u8; N];

        raw_vec
            .0
            .into_iter()
            .enumerate()
            .for_each(|(index, item)| array[index] = item);
            
        Payload(array)
    }
}
*/

impl<const N: usize> From<Vec<u8>> for Payload<N> {
    fn from(raw_vec: Vec<u8>) -> Self {
        let mut array: [u8; N] = [0 as u8; N];

        raw_vec
            .into_iter()
            .enumerate()
            .for_each(|(index, item)| array[index] = item);
            
        Payload(array)
    }
}

/*
//impl<T, const N: usize> From<[T; N]> for Vec<T> {
//impl<T, const N: usize> From<[T; N]> for RawVec<T> {
//impl<T, const N: usize> From<Payload<T, N>> for RawVec<T> {
impl<T, const N: usize> From<[T; N]> for RawVec<T> {
    fn from(array: [T; N]) -> Self {
        RawVec(
            array
                .into_iter()
                .map(|a| a)
                .collect::<Vec<T>>()
        )
    }
}
*/



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
#[allow(unused)]
pub fn measure<I2C, D, E, const L: usize>(grideye: &mut GridEye<I2C, D>) -> ([Temperature; L], Temperature, Temperature)
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
    E: Debug,
{

    let mut grid_raw = [TEMPERATURE_ERROR_VALUE; L];
    let mut max_temperature = TEMPERATURE_MAX;
    let mut min_temperature = TEMPERATURE_MIN;

    //(0..L as u8)                   // dynamic 00--07 .. 56--63
    //STATIC_ARRAY                   // static  00--07 .. 56--63
    STATIC_ARRAY_FLIPPED_HORIZONTAL  // static  56--63 .. 00--07
        .into_iter()
        .enumerate()
        .for_each(|(array_index, pixel_index)| {
            if let Ok(pixel_temp) = grideye.get_pixel_temperature_celsius(pixel_index) {
                if let Some(pixel) = grid_raw.get_mut(array_index) {
                    *pixel = pixel_temp;
                }
                
                if pixel_temp > max_temperature { max_temperature = pixel_temp }
                if pixel_temp < min_temperature { min_temperature = pixel_temp }
            }
        });

    (grid_raw, min_temperature ,max_temperature)
}


//
// single measurement return as be_bytes [u8; 4]
// we save one iteration, but can be harder to debug
#[allow(unused)]
pub fn measure_as_be_bytes_flat<I2C, D, E>(grideye: &mut GridEye<I2C, D>) -> (Vec<u8>, Temperature, Temperature)
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
    E: Debug,
{

    let mut max_temperature = TEMPERATURE_MAX;
    let mut min_temperature = TEMPERATURE_MIN;

    //(0..L as u8)                                  // dynamic 00--07 .. 56--63
    //STATIC_ARRAY                                  // static  00--07 .. 56--63
    let grid_raw = STATIC_ARRAY_FLIPPED_HORIZONTAL  // static  56--63 .. 00--07
        .into_iter()
        .flat_map(|pixel_index| {
            match grideye.get_pixel_temperature_celsius(pixel_index) {
                Ok(pixel_temp) => {
                    if pixel_temp > max_temperature { max_temperature = pixel_temp }
                    if pixel_temp < min_temperature { min_temperature = pixel_temp }
                    
                    pixel_temp.to_be_bytes()
                },
                Err(_e) => TEMPERATURE_ERROR_VALUE.to_be_bytes(),
            }
        })
        .collect::<Vec<u8>>();

    (grid_raw, min_temperature ,max_temperature)
}

//
// L is output array len
#[allow(unused)]
//pub fn measure_as_array_bytes<I2C, D, E, const L: usize>(grideye: &mut GridEye<I2C, D>) -> (Result<[u8; L], Vec<u8>>, Temperature, Temperature)
pub fn measure_as_array_bytes<I2C, D, E, const L: usize>(grideye: &mut GridEye<I2C, D>) -> ([u8; L], Temperature, Temperature)
//pub fn measure_as_array_bytes<I2C, D, E, const L: usize>(grideye: &mut GridEye<I2C, D>) -> (Payload<L>, Temperature, Temperature)
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u8>,
    E: Debug,
{

    let mut max_temperature = TEMPERATURE_MAX;
    let mut min_temperature = TEMPERATURE_MIN;

    //(0..L as u8)                   // dynamic 00--07 .. 56--63
    //STATIC_ARRAY                   // static  00--07 .. 56--63
    //let grid_raw: Result<[u8; L], Vec<u8>> = STATIC_ARRAY_FLIPPED_HORIZONTAL  // static  56--63 .. 00--07
    //let grid_raw: [u8; L] = STATIC_ARRAY_FLIPPED_HORIZONTAL  // static  56--63 .. 00--07
    let grid_raw = STATIC_ARRAY_FLIPPED_HORIZONTAL  // static  56--63 .. 00--07
        .into_iter()
        .enumerate()
        .flat_map(|(_array_index, pixel_index)| {
            match grideye.get_pixel_temperature_celsius(pixel_index) {
                Ok(pixel_temp) => {
                    if pixel_temp > max_temperature { max_temperature = pixel_temp }
                    if pixel_temp < min_temperature { min_temperature = pixel_temp }
                    
                    pixel_temp.to_be_bytes()
                },
                Err(_e) => TEMPERATURE_ERROR_VALUE.to_be_bytes(),
            }
        })
        .collect::<Vec<u8>>();
        //.collect::<RawVec<Vec<u8>>>()
        //.try_into();
        //.into();

    //let array: Payload<L> = RawVec(grid_raw).into();
    let array: Payload<L> = grid_raw.into();
    //let array = Into::<Payload<L>>::into(grid_raw).0;
    
    (array.0, min_temperature ,max_temperature)
    //(array, min_temperature ,max_temperature)
}
