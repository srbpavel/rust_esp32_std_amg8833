use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::blocking::delay::DelayMs;

use esp_idf_hal::delay::FreeRtos;
use std::sync::mpsc::Sender;
use embedded_graphics::geometry::Point;

use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
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
pub enum DisplayIliClear {
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct Render {
    pub msg: String,
    pub point: Point,
    pub clear: DisplayIliClear,
    pub delay: Option<u32>,
}

impl Default for Render {
    fn default() -> Self {
        Self {
            msg: String::from("Render Ili Default"),
            point: Point::zero(),
            clear: DisplayIliClear::False,
            delay: None,
        }
    }
}

impl Render {
    //
    pub fn draw(self,
                sender: &Sender<Render>,
    ) {
        let msg = self.msg.clone();
        let delay = self.delay.clone();
        
        if let Err(e) = sender.send(self)
        {
            error!("error display_ili msg: {e:?}\n{:?}",
                   msg,
            )
        }

        if let Some(timeout) = delay {
            FreeRtos::delay_ms(timeout)
        }
    }
}

//
pub fn init<DI, DELAY, RST>(di: DI,
                            delay: &mut DELAY,
                            rst: RST,
                            display_receiver: std::sync::mpsc::Receiver<Render>,
) -> Result<(), crate::WrapError<esp_idf_sys::EspError>>
where
    DI: display_interface::WriteOnlyDataCommand + std::marker::Send + 'static,
    DELAY: DelayUs<u32> + DelayMs<u32> + std::marker::Send,
    RST: esp_idf_hal::gpio::OutputPin + esp_idf_hal::gpio::InputPin
{
    let mut rst = esp_idf_hal::gpio::PinDriver::output(rst)?;

    warn!("$$$ SPI display PinDriver RST set_high()");
    rst.set_high()?;

    /*
    //delay.delay_us(1_000 as u32);
    //delay.delay_ms(1 as u32);
    delay.delay_us(10 as u32);
    info!("### DISPLAY SPI RESET is set high: {:?}",
          rst.is_set_high(),
    );
    
    rst.set_low()?;
    //delay.delay_us(10_000 as u32);
    delay.delay_ms(10 as u32);
    info!("### DISPLAY SPI RESET is set high: {:?}",
          rst.is_set_high(),
    );
    
    rst.set_high()?;
    */
    
    info!("### DISPLAY SPI RESET is set high: {:?}",
          rst.is_set_high(),
    );
    
    warn!("### DISPLAY SPI display init");
    // DI: WriteOnlyDataCommand
    // with Some(rst)
    let mut display_spi = mipidsi::Builder::ili9341_rgb666(di)
    // with None    
    //let mut display_spi:
    //mipidsi::Display<DI, mipidsi::models::ILI9341Rgb666, RST>
    //    = mipidsi::Builder::ili9341_rgb666(di)
        // default
        //.with_display_size(240 as u16, 320 as u16)
        // default
        //.with_framebuffer_size(240 as u16, 320 as u16)
        //.with_orientation(mipidsi::Orientation::Landscape(false))
        //.with_color_order(mipidsi::ColorOrder::Rgb)
        .init(
            // DELAY: DelayUs<u32>
            delay,
            // RST: OutputPin,
            Some(rst),
            //None,
        )?;
        //.map_err(|e| anyhow::anyhow!("Display SPI Builder Init error: {:?}", e)).unwrap();

    /*
    error!("$$$ SPI display orientation");
        display_spi
            .set_orientation(mipidsi::options::Orientation::Landscape(false))
            .map_err(|e| anyhow::anyhow!("Display SPI Orientation error: {:?}", e)).unwrap();//?;
    */

    std::thread::spawn(move || {
        while let Ok(channel_data) = display_receiver.recv() {
            // DEBUG
            info!("$$$ SPI display_spi_receiver msg: {channel_data:?}");

            // CLEAR
            if let DisplayIliClear::True = channel_data.clear {
                warn!("$$$ SPI display clear");
                if let Err(e) = display_spi
                    .clear(
                        embedded_graphics::pixelcolor::RgbColor::RED
                    ) {
                        error!("display_ili .clear() error: {e:?}")
                    }
            }

            // DRAW
            warn!("$$$ SPI set text.draw()");
            if let Err(e) = Text::new(
                &channel_data.msg,
                channel_data.point,
                MonoTextStyleBuilder::new()
                    .font(&FONT_6X10)
                    .text_color(embedded_graphics::pixelcolor::RgbColor::YELLOW)
                    .background_color(embedded_graphics::pixelcolor::RgbColor::GREEN)
                    .build(),
            ).draw(&mut display_spi) {
                error!("display_ili .draw() error: {e:?}")
            }
            
            /*
            display_spi
                .clear(
                    embedded_graphics::pixelcolor::RgbColor::RED
                ).unwrap();//?;
            */

            /*
            error!("$$$ SPI set pixel");
            display_spi.set_pixel(
                10,
                10,
                embedded_graphics::pixelcolor::RgbColor::GREEN,
            ).unwrap();//?;
            */
            
            /*
            let colors_vec: Vec<embedded_graphics_core::pixelcolor::Rgb666> =
                vec!(
                    embedded_graphics_core::pixelcolor::RgbColor::RED
                    //embedded_graphics_core::pixelcolor::Rgb666::new(0, 0, 0)
                );
            */
                
            /*
            let colors_array: [embedded_graphics_core::pixelcolor::Rgb666; 1] =
                [embedded_graphics_core::pixelcolor::RgbColor::RED];
            //[embedded_graphics_core::pixelcolor::Rgb666::new(0, 0, 0)];
            */

            /*
            error!("$$$ SPI set pixels");
            display_spi.set_pixels(
                0, // sx
                0, // sy
                319, // ex
                239, // ey
                //colors_vec,
                //colors_array,
                // [embedded_graphics::pixelcolor::RgbColor::RED; 1], // array
                [embedded_graphics::pixelcolor::RgbColor::RED], // slice
            )?;
            */
        }
    });

    Ok(())
}
