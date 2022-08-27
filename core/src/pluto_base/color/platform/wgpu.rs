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

use crate::pluto_base::color;
use pluto_engine_core_platform_wgpu::wgpu;

impl From<color::RGBA> for wgpu::Color {
    fn from(rgba: color::RGBA) -> Self {
        Self {
            r: rgba.r as f64,
            g: rgba.g as f64,
            b: rgba.b as f64,
            a: rgba.a as f64,
        }
    }
}

impl From<wgpu::Color> for color::RGBA {
    fn from(wc: wgpu::Color) -> Self {
        Self {
            r: wc.r as f32,
            g: wc.g as f32,
            b: wc.b as f32,
            a: wc.a as f32,
        }
    }
}
