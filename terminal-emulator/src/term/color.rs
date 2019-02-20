use std::ops::{Index, IndexMut, Mul};
use std::fmt;

use crate::ansi;
use crate::config;

pub const COUNT: usize = 270;

pub const RED: Rgb = Rgb { r: 0xff, g: 0x0, b: 0x0 };
pub const YELLOW: Rgb = Rgb { r: 0xff, g: 0xff, b: 0x0 };

#[derive(Debug, Eq, PartialEq, Copy, Clone, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// a multiply function for Rgb, as the default dim is just *2/3
impl Mul<f32> for Rgb {
    type Output = Rgb;

    fn mul(self, rhs: f32) -> Rgb {
        let result = Rgb {
            r: (f32::from(self.r) * rhs).max(0.0).min(255.0) as u8,
            g: (f32::from(self.g) * rhs).max(0.0).min(255.0) as u8,
            b: (f32::from(self.b) * rhs).max(0.0).min(255.0) as u8
        };

        trace!("Scaling RGB by {} from {:?} to {:?}", rhs, self, result);

        result
    }
}

/// List of indexed colors
///
/// The first 16 entries are the standard ansi named colors. Items 16..232 are
/// the color cube.  Items 233..256 are the grayscale ramp. Item 256 is
/// the configured foreground color, item 257 is the configured background
/// color, item 258 is the cursor foreground color, item 259 is the cursor
/// background color. Following that are 8 positions for dim colors.
/// Item 268 is the bright foreground color, 269 the dim foreground.
#[derive(Copy, Clone)]
pub struct List([Rgb; COUNT]);

impl<'a> From<&'a config::Colors> for List {
    fn from(colors: &config::Colors) -> List {
        // Type inference fails without this annotation
        let mut list: List = unsafe { ::std::mem::uninitialized() };

        list.fill_named(colors);
        list.fill_cube(colors);
        list.fill_gray_ramp(colors);

        list
    }
}

impl List {
    pub fn fill_named(&mut self, colors: &config::Colors) {
        // Normals
        self[ansi::NamedColor::Black]   = colors.normal.black;
        self[ansi::NamedColor::Red]     = colors.normal.red;
        self[ansi::NamedColor::Green]   = colors.normal.green;
        self[ansi::NamedColor::Yellow]  = colors.normal.yellow;
        self[ansi::NamedColor::Blue]    = colors.normal.blue;
        self[ansi::NamedColor::Magenta] = colors.normal.magenta;
        self[ansi::NamedColor::Cyan]    = colors.normal.cyan;
        self[ansi::NamedColor::White]   = colors.normal.white;

        // Brights
        self[ansi::NamedColor::BrightBlack]   = colors.bright.black;
        self[ansi::NamedColor::BrightRed]     = colors.bright.red;
        self[ansi::NamedColor::BrightGreen]   = colors.bright.green;
        self[ansi::NamedColor::BrightYellow]  = colors.bright.yellow;
        self[ansi::NamedColor::BrightBlue]    = colors.bright.blue;
        self[ansi::NamedColor::BrightMagenta] = colors.bright.magenta;
        self[ansi::NamedColor::BrightCyan]    = colors.bright.cyan;
        self[ansi::NamedColor::BrightWhite]   = colors.bright.white;
        self[ansi::NamedColor::BrightForeground] = colors
            .primary
            .bright_foreground
            .unwrap_or(colors.primary.foreground);

        // Foreground and background
        self[ansi::NamedColor::Foreground] = colors.primary.foreground;
        self[ansi::NamedColor::Background] = colors.primary.background;

        // Foreground and background for custom cursor colors
        self[ansi::NamedColor::CursorText] = colors.cursor.text.unwrap_or_else(Rgb::default);
        self[ansi::NamedColor::Cursor]     = colors.cursor.cursor.unwrap_or_else(Rgb::default);

        // Dims
        self[ansi::NamedColor::DimForeground] = colors
            .primary
            .dim_foreground
            .unwrap_or(colors.primary.foreground * 0.66);
        match colors.dim {
            Some(ref dim) => {
                trace!("Using config-provided dim colors");
                self[ansi::NamedColor::DimBlack]   = dim.black;
                self[ansi::NamedColor::DimRed]     = dim.red;
                self[ansi::NamedColor::DimGreen]   = dim.green;
                self[ansi::NamedColor::DimYellow]  = dim.yellow;
                self[ansi::NamedColor::DimBlue]    = dim.blue;
                self[ansi::NamedColor::DimMagenta] = dim.magenta;
                self[ansi::NamedColor::DimCyan]    = dim.cyan;
                self[ansi::NamedColor::DimWhite]   = dim.white;
            }
            None => {
                trace!("Deriving dim colors from normal colors");
                self[ansi::NamedColor::DimBlack]   = colors.normal.black   * 0.66;
                self[ansi::NamedColor::DimRed]     = colors.normal.red     * 0.66;
                self[ansi::NamedColor::DimGreen]   = colors.normal.green   * 0.66;
                self[ansi::NamedColor::DimYellow]  = colors.normal.yellow  * 0.66;
                self[ansi::NamedColor::DimBlue]    = colors.normal.blue    * 0.66;
                self[ansi::NamedColor::DimMagenta] = colors.normal.magenta * 0.66;
                self[ansi::NamedColor::DimCyan]    = colors.normal.cyan    * 0.66;
                self[ansi::NamedColor::DimWhite]   = colors.normal.white   * 0.66;
            }
        }
    }

    pub fn fill_cube(&mut self, colors: &config::Colors) {
        let mut index: usize = 16;
        // Build colors
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    // Override colors 16..232 with the config (if present)
                    if let Some(indexed_color) = colors
                        .indexed_colors
                        .iter()
                        .find(|ic| ic.index == index as u8)
                    {
                        self[index] = indexed_color.color;
                    } else {
                        self[index] = Rgb { r: if r == 0 { 0 } else { r * 40 + 55 },
                            b: if b == 0 { 0 } else { b * 40 + 55 },
                            g: if g == 0 { 0 } else { g * 40 + 55 },
                        };
                    }
                    index += 1;
                }
            }
        }

        debug_assert!(index == 232);
    }

    pub fn fill_gray_ramp(&mut self, colors: &config::Colors) {
        let mut index: usize = 232;

        for i in 0..24 {
            // Index of the color is number of named colors + number of cube colors + i
            let color_index = 16 + 216 + i;

            // Override colors 232..256 with the config (if present)
            if let Some(indexed_color) = colors
                .indexed_colors
                .iter()
                .find(|ic| ic.index == color_index)
            {
                self[index] = indexed_color.color;
                index += 1;
                continue;
            }

            let value = i * 10 + 8;
            self[index] = Rgb {
                r: value,
                g: value,
                b: value
            };
            index += 1;
        }

        debug_assert!(index == 256);
    }
}

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("List[..]")
    }
}

impl Index<ansi::NamedColor> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: ansi::NamedColor) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<ansi::NamedColor> for List {
    #[inline]
    fn index_mut(&mut self, idx: ansi::NamedColor) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

impl Index<usize> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

impl IndexMut<usize> for List {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

impl Index<u8> for List {
    type Output = Rgb;

    #[inline]
    fn index(&self, idx: u8) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<u8> for List {
    #[inline]
    fn index_mut(&mut self, idx: u8) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}
