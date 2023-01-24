mod errors;
mod i2c;
mod sensor_agm;

use errors::WrapError;

use sensor_agm::FramerateWrap;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cError;

use grideye::Address;
use grideye::GridEye;
use grideye::Power;

// /*
use ssd1306::prelude::*;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

// RAW img
//use embedded_graphics::image::Image;
//use embedded_graphics::image::ImageRaw;
//use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;

use embedded_graphics::text::Baseline;
use embedded_graphics::text::Text;

use embedded_graphics::pixelcolor::BinaryColor;
//use embedded_graphics::pixelcolor::Rgb565;

use embedded_graphics::primitives::Rectangle;
use embedded_graphics::primitives::PrimitiveStyleBuilder;

// */

use log::error;
use log::info;
use log::warn;

const SLEEP_DURATION: u16 = 30 * 1000;
const TEMPERATURE_ERROR_VALUE: f32 = 85.0;
const LEN: usize = 8; // array column/row size

//
fn main() -> Result<(), WrapError<I2cError>> {
    esp_idf_sys::link_patches();
    EspLogger::initialize_default();

    info!("### amg8833: array {LEN}x{LEN}");

    let machine_boot = EspSystemTime {}.now();
    warn!("machine_uptime: {machine_boot:?}");
    let mut cycle_counter = 0;

    let mut sleep = FreeRtos {};
    let delay = Ets {};

    let peripherals = esp_idf_hal::peripherals::Peripherals::take().unwrap();

    // VALID
    let pin_scl = peripherals.pins.gpio8;
    let pin_sda = peripherals.pins.gpio10;

    /* // INVALID -> pin's swapped
    let pin_scl = peripherals.pins.gpio10; // 8
    let pin_sda = peripherals.pins.gpio8; // 10
    */

    /*
    // I2C just to see type for error imlp
    let i2c: Result<esp_idf_hal::i2c::I2cDriver<'_>, esp_idf_sys::EspError> =
        i2c::config(peripherals.i2c0, pin_sda, pin_scl);
    */

    // I2C SHARED
    let i2c_shared = i2c::config_shared(peripherals.i2c0, pin_sda, pin_scl)?;
    let i2c_proxy_1 =i2c_shared.acquire_i2c();
    let i2c_proxy_2 =i2c_shared.acquire_i2c();
    
    // GRIDEYE
    //let mut grideye = GridEye::new(i2c?, delay, Address::Standard);
    let mut grideye = GridEye::new(i2c_proxy_1, delay, Address::Standard);

    // /*
    // DISPLAY
    //let interface = I2CDisplayInterface::new(i2c?);
    let interface = I2CDisplayInterface::new(i2c_proxy_2);
    // enum BinaryColor
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    ).into_buffered_graphics_mode();

    display.init()?;

    /* // LOGO
    let raw: ImageRaw<BinaryColor> =
        ImageRaw::new(include_bytes!("./rust_logo.raw"), 64);

    let im = Image::new(&raw, Point::new(32, 0));
    im.draw(&mut display)?;//.unwrap();
    */

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline("foookume is KiNg!",
                        //Point::zero(),
                        Point::new(0, 32),
                        text_style,
                        Baseline::Top)
        .draw(&mut display)?;
    
    Text::with_baseline("Hello Rust!",
                        Point::new(0, 16),
                        text_style,
                        Baseline::Top)
        .draw(&mut display)?;

    /* 128*64
    // Top side 
    display.set_pixel(0, 0, true);
    display.set_pixel(1, 0, true);
    display.set_pixel(2, 0, true);
    display.set_pixel(3, 0, true);

    // Right side
    display.set_pixel(3, 0, true);
    display.set_pixel(3, 1, true);
    display.set_pixel(3, 2, true);
    display.set_pixel(3, 3, true);

    // Bottom side
    display.set_pixel(0, 3, true);
    display.set_pixel(1, 3, true);
    display.set_pixel(2, 3, true);
    display.set_pixel(3, 3, true);

    // Left side
    display.set_pixel(0, 0, true);
    display.set_pixel(0, 1, true);
    display.set_pixel(0, 2, true);
    display.set_pixel(0, 3, true);
    */
    
    display.flush()?;

    sleep.delay_ms(SLEEP_DURATION / 10);
    //_
    // */

    // /*
    if grideye.power(Power::Wakeup).is_ok() {
        loop {
            cycle_counter += 1;

            // just to see type for error imlp
            let raw_temp: Result<u16, grideye::Error<_>> = grideye.get_device_temperature_raw();

            let device_temperature = grideye.get_device_temperature_celsius()?;
            
            warn!(
                "[{}] device >>> raw: {:0>16} / temperature: {}°C / status: {}",
                cycle_counter, // since last flash or soft-boot
                format!("{:b}", raw_temp?),
                device_temperature,
                grideye.pixel_temperature_out_ok()?,
            );

            if let Ok(framerate) = grideye.get_framerate() {
                warn!("framerate: Fps{}", FramerateWrap(framerate));
            }

            // 64 63 62 61 60 59 58 57
            // 56 55 54 53 52 51 50 49
            // 48 47 46 45 44 43 42 41
            // 40 39 38 37 36 35 34 33
            // 32 31 30 29 28 27 26 25
            // 24 23 22 21 20 19 18 17
            // 16 15 14 13 12 11 10  9
            //  8  7  6  5  4  3  2  1

            // /*
            let mut heat_map = sensor_agm::HeatMap([[0.0; LEN]; LEN]);

            info!("array occupies {} bytes", std::mem::size_of_val(&heat_map));

            // base 1d array
            let mut grid_raw = [0.0; LEN * LEN];
            let mut max_temperature = -55.;
            let mut min_temperature = 125.;
            
            let mut pixel_index = 0;
            
            (0..LEN as u8).into_iter().for_each(|x| {
                (0..LEN as u8).into_iter().for_each(|y| {
                    let pixel = (x * LEN as u8) + y;

                    if x.eq(&0) && y.eq(&0) {
                        warn!("status: {:?}", grideye.pixel_temperature_out_ok());
                    }
                    
                    // we don't want to fall only beause a single pixel error
                    let temp = match grideye.get_pixel_temperature_celsius(pixel) {
                        Ok(pixel_temp) => pixel_temp,
                        Err(e) => {
                            error!("Error reading pixel x: {x} y: {y} temperature: {e:?}");

                            TEMPERATURE_ERROR_VALUE
                        }
                    };

                    heat_map.0[x as usize][y as usize] = temp;

                    grid_raw[pixel_index as usize] = temp;
                    pixel_index += 1;

                    if temp > max_temperature {
                        max_temperature = temp;
                    }

                    if temp < min_temperature {
                        min_temperature = temp;
                    }
                })
            });
            // */

            // DEBUG
            //info!("debug: {heat_map:?}");

            // DISPLAY
            info!("heat_map_display:\n\n{heat_map}");

            /*
            (0..(LEN * LEN) as u8)
                .into_iter()
                .for_each(|index| {
                    let temp =
                        match grideye.get_pixel_temperature_celsius(index) {
                            Ok(pixel_temp) => pixel_temp,
                            Err(e) => {
                                error!("Error reading pixel index: {index} temperature: {e:?}");
                                
                                TEMPERATURE_ERROR_VALUE
                            }
                        };
                    
                    grid_raw[index as usize] = temp;
                    
                });
            */

            let grid_base = grid_raw.chunks(LEN);
            println!("grid_base: {:?}\n", grid_base);

            let grid_vec: Vec<_> = grid_base.collect();
            println!("grid_vec: {:?\n}", grid_vec);

            let grid_slice: &[&[f32]] = grid_vec.as_slice();
            println!("grid_slice: {:?}", grid_slice);

            display.clear();

            Text::with_baseline(&format!("cycle: {cycle_counter}"),
                                Point::new(0, 0),
                                text_style,
                                Baseline::Top)
                .draw(&mut display)?;
            
            Text::with_baseline(&format!("{device_temperature}"),
                                Point::new(0, 16),
                                text_style,
                                Baseline::Top)
                .draw(&mut display)?;

            Text::with_baseline(&format!("max: {max_temperature}"),
                                Point::new(64, 0),
                                text_style,
                                Baseline::Top)
                .draw(&mut display)?;
            
            Text::with_baseline(&format!("min: {min_temperature}"),
                                Point::new(64, 16),
                                text_style,
                                Baseline::Top)
                .draw(&mut display)?;

            let r_style = PrimitiveStyleBuilder::new()
                //.stroke_color(Rgb565::RED)
                .stroke_color(BinaryColor::On)
                .stroke_width(1)
                //.fill_color(Rgb565::GREEN)
                .fill_color(BinaryColor::Off)
                .build();
            
            Rectangle::new(Point::new(64, 32), Size::new(32, 32))
                .into_styled(r_style)
                .draw(&mut display)?;
            
            display.flush()?;
            
            info!("chrrr...\n");
            sleep.delay_ms(SLEEP_DURATION);
        }
    }
    // */

    Ok(())
}
