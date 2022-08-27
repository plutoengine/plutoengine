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

use crate::device::WgpuPhysicalDevice;
use crate::surface::WgpuSurface;
use pluto_engine_render::device::PhysicalDevice;
use pluto_engine_render::instance::ContextInstance;
use pluto_engine_render::pluto_engine_window::window::Window;
use pluto_engine_render::surface::Surface;
use raw_window_handle::HasRawWindowHandle;

pub struct WgpuInstance<
    'a,
    W: Window<SizeType = <WgpuSurface<'a> as Surface<'a>>::SizeType> + HasRawWindowHandle + 'a,
>(wgpu::Instance, &'a W);

impl<
        'a,
        W: Window<SizeType = <WgpuSurface<'a> as Surface<'a>>::SizeType> + HasRawWindowHandle + 'a,
    > ContextInstance<'a> for WgpuInstance<'a, W>
{
    type BackingType = wgpu::Instance;

    type PhysicalDeviceType = WgpuPhysicalDevice<'a>;
    type SurfaceType = WgpuSurface<'a>;
    type WindowType = W;

    fn new(window: &'a Self::WindowType) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        Self(instance, window)
    }

    fn create_device_and_surface(&self) -> (Self::PhysicalDeviceType, Self::SurfaceType) {
        let surface = unsafe { self.0.create_surface(self.1) };
        let adapter = pollster::block_on(self.0.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let physical_device = WgpuPhysicalDevice::new(adapter);
        let sfc = WgpuSurface::from_window(self.1, &physical_device, surface);

        (physical_device, sfc)
    }

    fn get_backing_instance(&self) -> &wgpu::Instance {
        &self.0
    }
}
