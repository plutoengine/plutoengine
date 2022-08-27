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

use pluto_engine_render::device::{Device, PhysicalDevice};
use pluto_engine_render::instance::ContextInstance;
use pluto_engine_render::surface::{Surface, SurfaceError};
use pluto_engine_window::event_loop::DisplayEvent;
use pluto_engine_window::window;
use pluto_engine_window::window::{PhysicalSize, WindowEvent};

pub use pluto_engine_render;
pub use pluto_engine_window;

pub type PlutoInstance<'a, AD> = <AD as ApplicationDisplay<'a>>::ContextType;

pub type PlutoPhysicalDevice<'a, AD> =
    <PlutoInstance<'a, AD> as ContextInstance<'a>>::PhysicalDeviceType;

pub type PlutoSurface<'a, AD> = <PlutoInstance<'a, AD> as ContextInstance<'a>>::SurfaceType;

pub type PlutoSurfaceTexture<'a, AD> = <PlutoSurface<'a, AD> as Surface<'a>>::TextureType;

pub type PlutoSurfaceError<'a, AD> = SurfaceError<<PlutoSurface<'a, AD> as Surface<'a>>::ErrorType>;

pub type PlutoSurfaceSizeType<'a, AD> = <PlutoSurface<'a, AD> as Surface<'a>>::SizeType;

pub type PlutoSurfaceSize<'a, AD> = PhysicalSize<PlutoSurfaceSizeType<'a, AD>>;

pub type PlutoDevice<'a, AD> = <PlutoPhysicalDevice<'a, AD> as PhysicalDevice<'a>>::DeviceType;

pub type PlutoQueue<'a, AD> = <PlutoPhysicalDevice<'a, AD> as PhysicalDevice<'a>>::QueueType;

pub type PlutoShader<'a, AD> = <PlutoDevice<'a, AD> as Device<'a>>::ShaderType;

pub type PlutoPipelineLayout<'a, AD> = <PlutoDevice<'a, AD> as Device<'a>>::PipelineLayoutType;

pub type PlutoPipeline<'a, AD> = <PlutoDevice<'a, AD> as Device<'a>>::PipelineType;

pub trait WindowDisplay {
    type WindowType: window::Window;

    fn close_requested(&self) -> bool;

    fn on_event(&mut self, window_event: &WindowEvent);

    fn get_window(&self) -> &Self::WindowType;
}

pub trait ApplicationDisplay<'a>: WindowDisplay {
    type ContextType: ContextInstance<'a, WindowType = Self::WindowType>;

    fn new(
        surface: &'a mut PlutoSurface<'a, Self>,
        window: &'a Self::WindowType,
        device: &'a PlutoDevice<'a, Self>,
    ) -> Self;

    fn on_event<AS: ApplicationState<'a, Self>>(
        &mut self,
        display_event: DisplayEvent,
    ) -> Box<dyn FnOnce(&mut AS)>
    where
        Self: Sized + ApplicationDisplay<'a>;

    fn refresh_surface(&mut self);
    fn resize_surface(&mut self, size: PlutoSurfaceSize<'a, Self>);

    fn get_surface(&self) -> &PlutoSurface<'a, Self>;
}

pub trait ApplicationState<'a, AD: ApplicationDisplay<'a>> {
    fn new(display: AD, device: &'a PlutoDevice<'a, AD>, queue: &'a PlutoQueue<'a, AD>) -> Self;

    fn render(&mut self, surface_texture: &PlutoSurfaceTexture<'a, AD>);

    fn display(&mut self) -> &mut AD;
}
