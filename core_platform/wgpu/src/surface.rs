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

use crate::device::WgpuDevice;
use crate::texture::{WgpuTextureFormat, WgpuTextureView};
use pluto_engine_render::device::{Device, PhysicalDevice};
use pluto_engine_render::pluto_engine_window::window::{PhysicalSize, Window};
use pluto_engine_render::surface::{Surface, SurfaceError, SurfaceFormat, SurfaceTexture};
use raw_window_handle::HasRawWindowHandle;
use std::marker::PhantomData;
use wgpu::TextureViewDescriptor;

pub struct WgpuSurfaceFormat(wgpu::TextureFormat);

impl SurfaceFormat for WgpuSurfaceFormat {
    type BackingType = wgpu::TextureFormat;

    fn get_backing_format(&self) -> Self::BackingType {
        self.0
    }
}

pub struct WgpuSurface<'a> {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    parent: PhantomData<&'a ()>,
}

impl<'a> WgpuSurface<'_> {
    pub(crate) fn from_window<
        W: Window<SizeType = <WgpuSurface<'a> as Surface<'a>>::SizeType> + HasRawWindowHandle,
        D: PhysicalDevice<'a, BackingType = wgpu::Adapter>,
    >(
        window: &W,
        physical_device: &D,
        surface: wgpu::Surface,
    ) -> Self {
        let size = window.get_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_preferred_format(physical_device.get_backing_physical_device())
                .unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        Self {
            surface,
            config,
            parent: PhantomData,
        }
    }
}

pub struct WgpuSurfaceTexture<'a> {
    texture: wgpu::SurfaceTexture,
    parent: PhantomData<&'a ()>,
}

impl<'a> SurfaceTexture<'_> for WgpuSurfaceTexture<'a> {
    type BackingType = wgpu::SurfaceTexture;
    type TextureViewType = WgpuTextureView<'a>;

    fn get_backing_texture(&self) -> &Self::BackingType {
        &self.texture
    }

    fn get_texture_view(&self) -> Self::TextureViewType {
        WgpuTextureView {
            view: self
                .texture
                .texture
                .create_view(&TextureViewDescriptor::default()),
            parent: PhantomData,
        }
    }

    fn present(self) {
        self.texture.present();
    }
}

impl<'a> Surface<'_> for WgpuSurface<'a> {
    type BackingType = wgpu::Surface;

    type SizeType = u32;
    type DeviceType = WgpuDevice<'a>;
    type FormatType = WgpuSurfaceFormat;
    type TextureFormatType = WgpuTextureFormat;
    type TextureType = WgpuSurfaceTexture<'a>;

    type ErrorType = wgpu::SurfaceError;

    fn configure(&mut self, device: &WgpuDevice<'a>) {
        self.surface
            .configure(device.get_backing_device(), &self.config);
    }

    fn resize(&mut self, device: &WgpuDevice<'a>, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.config.width = size.width;
        self.config.height = size.height;
        self.configure(device);
    }

    fn get_format(&self) -> WgpuSurfaceFormat {
        WgpuSurfaceFormat(self.config.format)
    }

    fn get_texture_format(&self) -> Self::TextureFormatType {
        WgpuTextureFormat(self.config.format)
    }

    fn get_backing_surface(&self) -> &Self::BackingType {
        &self.surface
    }

    fn acquire_next_texture(&self) -> Result<Self::TextureType, SurfaceError<Self::ErrorType>> {
        Ok(WgpuSurfaceTexture {
            texture: self.surface.get_current_texture()?,
            parent: PhantomData,
        })
    }
}
