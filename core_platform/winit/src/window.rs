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

use log::info;
use pluto_engine_window::event_loop::{
    DisplayCommand, DisplayEvent, EventLoop, EventLoopWindowFactory,
};
use pluto_engine_window::window;
use pluto_engine_window::window::{Window, WindowEventReceiver};
use raw_window_handle::RawWindowHandle;
use std::sync::mpsc::Receiver;
use winit::event::WindowEvent;
use winit::window::WindowBuilder;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use winit::dpi::PhysicalSize;

pub struct WinitWindow(
    winit::window::Window,
    Box<dyn Fn(DisplayCommand) + Send>,
    Receiver<DisplayEvent>,
);

pub struct WinitWindowEvent<'a, 'b>(pub(crate) &'a WindowEvent<'b>);

pub struct WinitPhysicalSize(PhysicalSize<<WinitWindow as Window>::SizeType>);

impl<'a, 'b> WindowEventReceiver<WinitWindowEvent<'a, 'b>> for WinitWindow {
    type EventType = WinitWindowEvent<'a, 'b>;
}

impl Window for WinitWindow {
    type IdType = winit::window::WindowId;
    type BackingType = winit::window::Window;
    type SizeType = u32;
    type LoopType = winit::event_loop::EventLoopWindowTarget<DisplayCommand>;

    fn new<
        EL: EventLoop<WindowType = Self> + 'static,
        ELW: EventLoopWindowFactory<EL, LoopType = Self::LoopType>,
    >(
        event_loop: &ELW,
        event_receiver: Receiver<DisplayEvent>,
        command_proxy: Box<dyn Fn(DisplayCommand) + Send>,
    ) -> Self {
        let backing_loop = event_loop.get_backing_loop();
        let window = WindowBuilder::new().build(backing_loop).unwrap();

        #[cfg(target_arch = "wasm32")]
        {
            window.set_inner_size(winit::dpi::PhysicalSize::new(640, 480));

            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(web_sys::Window::document)
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("pluto-viewport")?;
                    let canvas = web_sys::Element::from(window.canvas());
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Pluto window container HTML element not found!");
        }

        Self(window, command_proxy, event_receiver)
    }

    fn receive_event(&self) -> DisplayEvent {
        self.2.recv().unwrap_or_else(|e| {
            info!(
                "The window ID {:?} channel was disconnected: {e}",
                self.get_id()
            );

            DisplayEvent::Disconnected
        })
    }

    fn request_repaint(&self) {
        self.0.request_redraw()
    }

    fn get_id(&self) -> Self::IdType {
        self.0.id()
    }

    fn get_size(&self) -> window::PhysicalSize<u32> {
        window::PhysicalSize::from(WinitPhysicalSize(self.0.inner_size()))
    }

    fn get_backing_window(&self) -> &Self::BackingType {
        &self.0
    }
}

impl From<WinitWindowEvent<'_, '_>> for window::WindowEvent {
    fn from(e: WinitWindowEvent) -> Self {
        match e.0 {
            WindowEvent::Resized(ps) => {
                window::WindowEvent::Resized(window::PhysicalSize::from(WinitPhysicalSize(*ps)))
            }
            WindowEvent::Moved(_) => window::WindowEvent::Unknown,
            WindowEvent::CloseRequested => window::WindowEvent::CloseRequested,
            WindowEvent::Destroyed => window::WindowEvent::Unknown,
            WindowEvent::DroppedFile(_) => window::WindowEvent::Unknown,
            WindowEvent::HoveredFile(_) => window::WindowEvent::Unknown,
            WindowEvent::HoveredFileCancelled => window::WindowEvent::Unknown,
            WindowEvent::ReceivedCharacter(_) => window::WindowEvent::Unknown,
            WindowEvent::Focused(_) => window::WindowEvent::Unknown,
            WindowEvent::KeyboardInput { .. } => window::WindowEvent::Unknown,
            WindowEvent::ModifiersChanged(_) => window::WindowEvent::Unknown,
            WindowEvent::CursorMoved { .. } => window::WindowEvent::Unknown,
            WindowEvent::CursorEntered { .. } => window::WindowEvent::Unknown,
            WindowEvent::CursorLeft { .. } => window::WindowEvent::Unknown,
            WindowEvent::MouseWheel { .. } => window::WindowEvent::Unknown,
            WindowEvent::MouseInput { .. } => window::WindowEvent::Unknown,
            WindowEvent::TouchpadPressure { .. } => window::WindowEvent::Unknown,
            WindowEvent::AxisMotion { .. } => window::WindowEvent::Unknown,
            WindowEvent::Touch(_) => window::WindowEvent::Unknown,
            WindowEvent::ScaleFactorChanged { .. } => window::WindowEvent::Unknown,
            WindowEvent::ThemeChanged(_) => window::WindowEvent::Unknown,
            WindowEvent::Ime(_) => window::WindowEvent::Unknown,
            WindowEvent::Occluded(_) => window::WindowEvent::Unknown,
        }
    }
}

unsafe impl raw_window_handle::HasRawWindowHandle for WinitWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.get_backing_window().raw_window_handle()
    }
}

impl From<WinitPhysicalSize> for window::PhysicalSize<u32> {
    fn from(size: WinitPhysicalSize) -> Self {
        Self {
            width: size.0.width,
            height: size.0.height,
        }
    }
}
