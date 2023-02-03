mod errors;
mod i2c;
mod sensor_agm;

use errors::WrapError;

use sensor_agm::LEN;

use esp_idf_sys as _;

use esp_idf_svc::log::EspLogger;
use esp_idf_svc::systime::EspSystemTime;

use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::Ets;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cError;
use esp_idf_hal::i2c::I2cDriver;

use grideye::Address;
use grideye::GridEye;
use grideye::Power;

use ssd1306::prelude::*;
use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::Baseline;
use embedded_graphics::text::Text;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

const SLEEP_DURATION: u16 = 30 * 1000;

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

    // I2C type just to see type for error imlp
    let i2c: Result<esp_idf_hal::i2c::I2cDriver<'_>, esp_idf_sys::EspError> =
        i2c::config(peripherals.i2c0, pin_sda, pin_scl);
    let i2c = i2c?;

    // I2C SHARED
    let i2c_shared: &'static _ = shared_bus::new_std!(I2cDriver = i2c).ok_or(WrapError::I2cError)?;
    
    let i2c_proxy_1 = i2c_shared.acquire_i2c(); // agm
    let i2c_proxy_2 = i2c_shared.acquire_i2c(); // ssd1306
    //let mut i2c_proxy_3 = i2c_shared.acquire_i2c(); // i2c scan share
    //let i2c_proxy_4 = i2c_shared.acquire_i2c(); // shtc3
    let i2c_proxy_5 = i2c_shared.acquire_i2c(); // i2c scan share loop
    
    // GRIDEYE
    let mut grideye = GridEye::new(i2c_proxy_1, delay, Address::Standard);
    
    // DISPLAY
    let interface = I2CDisplayInterface::new(i2c_proxy_2);
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init()?;

    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    Text::with_baseline(
        "foookume is KiNg!",
        Point::new(0, 32),
        text_style,
        Baseline::Top,
    )
    .draw(&mut display)?;

    display.flush()?;

    sleep.delay_ms(SLEEP_DURATION / 10);

    if grideye.power(Power::Wakeup).is_ok() {
        loop {
            // I2C LOOP scan 
            let mut i2c_clone = i2c_proxy_5.clone();
            std::thread::spawn(move || {
                warn!("i2c_scan_shared_loop + thread");
                let active_address = i2c::scan_shared(&mut i2c_clone);

                info!(
                    "I2C LOOP active address: {:?}",
                    match active_address {
                        Some(active) => {
                            active
                                .iter()
                                .map(|a| format!("{a:#X} "))
                                .collect::<Vec<String>>()
                                .concat()
                        }
                        None => {
                            String::from("")
                        }
                    }
                );
            });

            // GRIDEYE
            cycle_counter += 1;

            let (grid_raw, min_temperature, max_temperature): ([f32; LEN * LEN], f32, f32) = sensor_agm::measure(&mut grideye);

            // via trait
            let heat_map: sensor_agm::HeatMap<f32, LEN> = sensor_agm::HeatMap::from(grid_raw);
            // via fn
            //let heat_map: sensor_agm::HeatMap<f32, LEN> = sensor_agm::array_to_map(grid_raw);

            // DEBUG
            //info!("debug: {heat_map:?}");

            // DISPLAY
            info!("heat_map_display:\n\n{heat_map}");

            let mut grid_indexed = grid_raw
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    format!("{index:02}:{item:02.02}")
                })
                .collect::<Vec<_>>();

            grid_indexed.reverse();

            // DEBUG RAW <-- reversed
            info!("grid_indexed: {grid_indexed:?}");
            
            // SSD1306
            display.clear();

            Text::with_baseline(
                &format!("{cycle_counter}"),
                Point::new(0, 0),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)?;

            Text::with_baseline(
                &format!("max:  {max_temperature}"),
                Point::new(48, 0),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)?;

            Text::with_baseline(
                &format!("min:  {min_temperature}"),
                Point::new(48, 16),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)?;

            Text::with_baseline(
                &format!("diff: {}", max_temperature - min_temperature,),
                Point::new(48, 32),
                text_style,
                Baseline::Top,
            )
            .draw(&mut display)?;

            let style = PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::On)
                .stroke_width(1)
                .fill_color(BinaryColor::Off)
                .build();

            Rectangle::new(Point::new(48, 48), Size::new(16, 16))
                .into_styled(style)
                .draw(&mut display)?;

            display.flush()?;

            info!("chrrr...\n");
            sleep.delay_ms(SLEEP_DURATION);
        }
    }

    Ok(())
}
