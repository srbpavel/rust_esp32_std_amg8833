use std::fmt;
use std::fmt::Debug;

use std::ops;

use grideye::Framerate;

pub struct FramerateWrap(pub Framerate);

impl fmt::Display for FramerateWrap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f, "{}",
            match self.0 {
                Framerate::Fps10 => 10,
                Framerate::Fps1 => 1,
            }
            //"{:#x}", self.0 as u8,
        )
    }
}
   

#[derive(Debug)]
pub struct HeatMap(pub [[f32; 8]; 8]);

impl ops::Deref for HeatMap {
    type Target = [[f32; 8]; 8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for HeatMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut line = 0;
        let cell = 8 * 6;
        let blank_line = format!("\n*{}*", " ".repeat((cell) + 3));

        self.iter().fold(Ok(()), |result, row| {
            result.and_then(|_| {
                line += 1;

                let first = if line.eq(&1) {
                    format!("{}*{blank_line}\n", "* ".repeat(((cell) + 5) / 2))
                } else {
                    "".to_string()
                };

                let last = if line.eq(&self.len()) {
                    format!("\n{}*", "* ".repeat(((cell) + 5) / 2))
                } else {
                    "".to_string()
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
