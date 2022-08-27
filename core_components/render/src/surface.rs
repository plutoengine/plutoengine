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

use crate::device::Device;
use crate::texture::{TextureFormat, TextureView};
use pluto_engine_window::window::PhysicalSize;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Clone)]
pub enum SurfaceError<T: Clone + Error> {
    DeviceLost,
    OutOfMemory,
    Other(T),
}

impl<T: Clone + Error> Display for SurfaceError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceError::Other(cause) => write!(f, "{}", cause),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl<T: Clone + Error> Error for SurfaceError<T> {
    fn cause(&self) -> Option<&dyn Error> {
        match self {
            SurfaceError::Other(cause) => Some(cause),
            _ => None,
        }
    }
}

impl<T: Clone + Error> From<T> for SurfaceError<T> {
    fn from(err: T) -> Self {
        Self::Other(err)
    }
}

pub trait SurfaceTexture<'a> {
    type BackingType;
    type TextureViewType: TextureView<'a>;

    fn get_backing_texture(&self) -> &Self::BackingType;

    fn get_texture_view(&self) -> Self::TextureViewType;

    fn present(self);
}

pub trait SurfaceFormat {
    type BackingType: Copy + Clone;

    fn get_backing_format(&self) -> Self::BackingType;
}

pub trait Surface<'a> {
    type BackingType;

    type SizeType: Sized;
    type DeviceType: Device<'a>;
    type FormatType: SurfaceFormat;
    type TextureFormatType: TextureFormat;
    type TextureType: SurfaceTexture<'a>;
    type ErrorType: Clone + Error;

    fn configure(&mut self, device: &Self::DeviceType);

    fn resize(&mut self, device: &Self::DeviceType, size: PhysicalSize<Self::SizeType>);

    fn get_format(&self) -> Self::FormatType;

    fn get_texture_format(&self) -> Self::TextureFormatType;

    fn get_backing_surface(&self) -> &Self::BackingType;

    fn acquire_next_texture(&self) -> Result<Self::TextureType, SurfaceError<Self::ErrorType>>;
}
