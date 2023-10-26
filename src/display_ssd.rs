use std::fmt::Debug;

use esp_idf_hal::delay::FreeRtos;
use embedded_graphics::geometry::Point;

use embedded_hal::blocking::i2c::Read;
use embedded_hal::blocking::i2c::Write;
use embedded_hal::blocking::i2c::WriteRead;

use ssd1306::I2CDisplayInterface;
use ssd1306::Ssd1306;
use ssd1306::mode::DisplayConfig;
use ssd1306::size::DisplaySize128x64;
use ssd1306::rotation::DisplayRotation;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::text::Baseline;
use embedded_graphics::text::Text;
use embedded_graphics::Drawable;
use embedded_graphics::draw_target::DrawTarget;

#[allow(unused_imports)]
use log::error;
#[allow(unused_imports)]
use log::info;
#[allow(unused_imports)]
use log::warn;

#[derive(Debug, Clone)]
pub enum DisplaySsdClear {
    True,
    False,
}

#[derive(Debug, Clone)]
pub enum DisplaySsdFlush {
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct Render {
    pub msg: String,
    pub point: Point,
    pub clear: DisplaySsdClear,
    pub flush: DisplaySsdFlush,
    pub delay: Option<u32>,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            msg: String::from("Render Ssd Default"),
            point: Point::zero(),
            clear: DisplaySsdClear::False,
            flush: DisplaySsdFlush::False,
            delay: None,
        }
    }
}

impl Render {
    //
    pub fn draw(self,
                sender: &std::sync::mpsc::Sender<Render>,
    ) {
        if let Err(e) = sender.send(self)
        {
            error!("error display_ssd msg: {e:?}")
        }
    }
}

//
pub fn init<I2C, E, T>(i2c: I2C,
                       display_receiver: std::sync::mpsc::Receiver<Render>,
)
where
    I2C: Read<Error = E>
    + Write<Error = E>
    + WriteRead<Error = E>
    + std::marker::Send
    + 'static,
    E: Debug,
{
    // for thread::spawn -> we need to use i2c not already initialized Ssd1306 !!!
    std::thread::spawn(move || {
        let mut display = Ssd1306::new(I2CDisplayInterface::new(i2c),
                                       DisplaySize128x64,
                                       DisplayRotation::Rotate0,
        )
            .into_buffered_graphics_mode();
        
        if let Err(e) = display.init() {
            error!("display_ssd init error: {e:?}")
        }
        
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_6X10)
            .text_color(BinaryColor::On)
            .build();
        
        while let Ok(channel_data) = display_receiver.recv() {
            // DEBUG
            //info!("display_ssd_receiver msg: {channel_data:?}");
            
            // CLEAR
            if let DisplaySsdClear::True = channel_data.clear {
                if let Err(e) = display.clear(embedded_graphics::pixelcolor::BinaryColor::Off) {
                    error!("display_ssd .clear() error: {e:?}")
                }
            }
            
            // DRAW TEXT
            if let Err(e) = Text::with_baseline(
                &channel_data.msg,
                channel_data.point,
                text_style,
                Baseline::Top,
            )
                .draw(&mut display) {
                    error!("display_ssd .draw() error: {e:?}")
                }
            
            // FLUSH
            if let DisplaySsdFlush::True = channel_data.flush {
                if let Err(e) = display.flush() {
                    error!("display_ssd .flush() error: {e:?}")
                }
            }

            // DELAY
            if let Some(timeout) = channel_data.delay {
                FreeRtos::delay_ms(timeout)
            }
        }
    });
}
