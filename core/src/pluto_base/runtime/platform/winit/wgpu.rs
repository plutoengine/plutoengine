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

use log::{error, warn};
use pluto_engine_core_platform_wgpu::device::WgpuDevice;
use pluto_engine_core_platform_wgpu::instance::WgpuInstance;
use pluto_engine_core_platform_winit::window::WinitWindow;
use pluto_engine_display::pluto_engine_render::surface::{Surface, SurfaceError, SurfaceTexture};
use pluto_engine_display::pluto_engine_window::event_loop::DisplayEvent;
use pluto_engine_display::pluto_engine_window::window::{PhysicalSize, Window, WindowEvent};
use pluto_engine_display::{
    ApplicationDisplay, ApplicationState, PlutoDevice, PlutoSurface, PlutoSurfaceSize,
    WindowDisplay,
};

pub struct WinitWgpuDisplay<'p> {
    surface: &'p mut PlutoSurface<'p, WinitWgpuDisplay<'p>>,
    window: &'p WinitWindow,
    device: &'p WgpuDevice<'p>,
    surface_size: PhysicalSize<<PlutoSurface<'p, WinitWgpuDisplay<'p>> as Surface<'p>>::SizeType>,
    close_requested: bool,
}

impl<'p> WindowDisplay for WinitWgpuDisplay<'p> {
    type WindowType = WinitWindow;

    fn close_requested(&self) -> bool {
        self.close_requested
    }

    fn on_event(&mut self, window_event: &WindowEvent) {
        match window_event {
            WindowEvent::CloseRequested => self.close_requested = true,
            WindowEvent::Resized(size) => self.resize_surface(*size),
            _ => {}
        };
    }

    fn get_window(&self) -> &Self::WindowType {
        self.window
    }
}

impl<'p> ApplicationDisplay<'p> for WinitWgpuDisplay<'p> {
    type ContextType = WgpuInstance<'p, Self::WindowType>;

    fn new(
        surface: &'p mut PlutoSurface<'p, Self>,
        window: &'p Self::WindowType,
        device: &'p PlutoDevice<'p, Self>,
    ) -> Self {
        Self {
            surface,
            window,
            device,
            surface_size: Default::default(),
            close_requested: false,
        }
    }

    fn on_event<AS: ApplicationState<'p, Self>>(
        &mut self,
        display_event: DisplayEvent,
    ) -> Box<dyn FnOnce(&mut AS)>
    where
        Self: Sized + ApplicationDisplay<'p>,
    {
        match &display_event {
            DisplayEvent::NextFrame => self.window.request_repaint(),
            DisplayEvent::Repaint => {
                return Box::new(|s| {
                    let surface = s.display().get_surface();
                    match surface.acquire_next_texture() {
                        Ok(texture) => {
                            s.render(&texture);
                            texture.present();
                        }
                        Err(SurfaceError::OutOfMemory) => {}
                        Err(SurfaceError::DeviceLost) => {
                            let display = s.display();
                            warn!("Device for window ID {:?} lost!", display.window.get_id());
                            display.refresh_surface();
                        }
                        Err(SurfaceError::Other(ref e)) => {
                            let display = s.display();
                            error!(
                                "Unknown surface error for window ID {:?}: {}",
                                display.window.get_id(),
                                e
                            );
                        }
                    }
                })
            }
            DisplayEvent::WindowEvent(ref window_event) => match window_event {
                _ => WindowDisplay::on_event(self, window_event),
            },
            DisplayEvent::Disconnected => {}
        };

        Box::new(|_| {})
    }

    fn refresh_surface(&mut self) {
        self.surface.resize(self.device, self.surface_size);
    }

    fn resize_surface(&mut self, size: PlutoSurfaceSize<'p, Self>) {
        self.surface_size = size;
        self.surface.resize(self.device, size);
    }

    fn get_surface(&self) -> &PlutoSurface<'p, Self> {
        self.surface
    }
}
