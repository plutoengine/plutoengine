/*
 * MIT License
 *
 * Copyright (c) 2022 AMNatty
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use cgmath::Vector4;

pub mod platform;
pub use self::platform::*;

type RGBAu8 = (u8, u8, u8, u8);

pub trait Color: From<RGBAu8> {
    fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::from((r, g, b, a))
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba(r, g, b, 255)
    }

    fn lerp(self, other: Self, ratio: f32) -> Self;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<Vector4<f32>> for RGBA {
    fn from(vec: Vector4<f32>) -> Self {
        unsafe { std::mem::transmute(vec) }
    }
}

impl From<RGBA> for Vector4<f32> {
    fn from(rgba: RGBA) -> Self {
        unsafe { std::mem::transmute(rgba) }
    }
}

impl From<RGBAu8> for RGBA {
    fn from((r, g, b, a): RGBAu8) -> Self {
        Self {
            r: r as f32 / u8::MAX as f32,
            g: g as f32 / u8::MAX as f32,
            b: b as f32 / u8::MAX as f32,
            a: a as f32 / u8::MAX as f32,
        }
    }
}

impl Color for RGBA {
    fn lerp(self, other: Self, ratio: f32) -> Self {
        RGBA {
            r: self.r * ratio + other.r * (1.0 - ratio),
            g: self.g * ratio + other.g * (1.0 - ratio),
            b: self.b * ratio + other.b * (1.0 - ratio),
            a: self.a * ratio + other.a * (1.0 - ratio),
        }
    }
}

#[derive(Copy, Clone)]
struct HSBA {
    pub h: f32,
    pub s: f32,
    pub b: f32,
    pub a: f32,
}

impl From<HSBA> for RGBA {
    fn from(hsba: HSBA) -> Self {
        let h6 = hsba.h / 60.0;

        let hue_side = h6 as i32;

        // The color component furthest on the hue wheel
        let p = hsba.b * (1.0 - hsba.s);

        let hue_fract_ccw = h6 - hue_side as f32;
        // The second-nearest color component on the hue wheel - counter-clockwise
        let q = hsba.b * (1.0 - hue_fract_ccw * hsba.s);

        let hue_fract_cw = 1.0 - hue_fract_ccw;
        // The second-nearest color component on the hue wheel - clockwise
        let t = hsba.b * (1.0 - hue_fract_cw * hsba.s);

        match hue_side % 6 {
            // Hues 60°-119° -- Green is the brightest color, no blue is present at max saturation
            1 => Self {
                r: q,
                g: hsba.b,
                b: p,
                a: hsba.a,
            },
            // Hues 120°-179° -- Green is the brightest color, no red is present at max saturation
            2 => Self {
                r: p,
                g: hsba.b,
                b: t,
                a: hsba.a,
            },
            // Hues 180°-239° -- Blue is the brightest color, no red is present at max saturation
            3 => Self {
                r: p,
                g: q,
                b: hsba.b,
                a: hsba.a,
            },
            // Hues 240°-299° -- Blue is the brightest color, no green is present at max saturation
            4 => Self {
                r: t,
                g: p,
                b: hsba.b,
                a: hsba.a,
            },
            // Hues 300°-359° -- Red is the brightest color, no green is present at max saturation
            5 => Self {
                r: hsba.b,
                g: p,
                b: q,
                a: hsba.a,
            },
            // Hues 0°-59° -- Red is the brightest color, no blue is present at max saturation
            _ => Self {
                r: hsba.b,
                g: t,
                b: p,
                a: hsba.a,
            },
        }
    }
}

impl From<RGBA> for HSBA {
    fn from(rgba: RGBA) -> Self {
        let brightness = rgba.r.max(rgba.g).max(rgba.b);
        let min = rgba.r.min(rgba.g).min(rgba.b);
        let chroma = brightness - min;
        let saturation = chroma / brightness;
        let hue = if brightness == rgba.r {
            if rgba.g < rgba.b {
                (rgba.g - rgba.b) / chroma + 6.0
            } else {
                (rgba.g - rgba.b) / chroma
            }
        } else if brightness == rgba.g {
            (rgba.b - rgba.r) / chroma + 2.0
        } else {
            (rgba.r - rgba.g) / chroma + 4.0
        };

        Self {
            h: hue * 60.0,
            s: saturation,
            b: brightness,
            a: rgba.a,
        }
    }
}

impl From<RGBAu8> for HSBA {
    fn from(rgba: RGBAu8) -> Self {
        RGBA::from(rgba).into()
    }
}

impl Color for HSBA {
    fn lerp(self, other: Self, ratio: f32) -> Self {
        HSBA {
            h: self.h * ratio + other.h * (1.0 - ratio),
            s: self.s * ratio + other.s * (1.0 - ratio),
            b: self.b * ratio + other.b * (1.0 - ratio),
            a: self.a * ratio + other.a * (1.0 - ratio),
        }
    }
}

pub const WHITE: RGBA = RGBA {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

pub const BLACK: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const RED: RGBA = RGBA {
    r: 1.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const GREEN: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

pub const BLUE: RGBA = RGBA {
    r: 0.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

pub const YELLOW: RGBA = RGBA {
    r: 1.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
