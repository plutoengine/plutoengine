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

use crate::window::{WinitWindow, WinitWindowEvent};
use log::{info, warn};
use pluto_engine_window::event_loop::{
    DisplayCommand, DisplayEvent, EventLoop, EventLoopWindowFactory,
};
use pluto_engine_window::window::Window;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::mpsc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoopProxy};

pub struct WinitEventLoop {
    windows: HashMap<<WinitWindow as Window>::IdType, mpsc::SyncSender<DisplayEvent>>,
    proxy: EventLoopProxy<DisplayCommand>,
}

impl EventLoop for WinitEventLoop {
    type WindowType = WinitWindow;
    type LoopType = winit::event_loop::EventLoopWindowTarget<DisplayCommand>;

    fn run<F: FnOnce(&mut dyn EventLoopWindowFactory<Self, LoopType = Self::LoopType>) + 'static>(
        initializer: F,
    ) -> Infallible
    where
        Self: Sized,
    {
        let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
        let mut event_loop_data = Self {
            windows: HashMap::new(),
            proxy: event_loop.create_proxy(),
        };
        initializer(&mut WinitEventLoopWindowFactory {
            windows: &mut event_loop_data.windows,
            event_loop: &*event_loop,
            proxy: event_loop_data.proxy.clone(),
        });

        event_loop.run(move |event, _, control_flow| match event {
            Event::RedrawRequested(window_id) => {
                event_loop_data.send_event(window_id, DisplayEvent::Repaint);
            }

            Event::MainEventsCleared => {
                let window: Vec<_> = event_loop_data.windows.keys().copied().collect();
                window.into_iter().for_each(|id| {
                    event_loop_data.send_event(id, DisplayEvent::NextFrame);
                });
            }

            Event::WindowEvent {
                event: WindowEvent::Destroyed,
                window_id,
            } => {
                event_loop_data.windows.remove(&window_id);

                if event_loop_data.windows.is_empty() {
                    *control_flow = ControlFlow::Exit;
                }
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } => {
                event_loop_data.send_event(
                    window_id,
                    DisplayEvent::WindowEvent(WinitWindowEvent(event).into()),
                );
            }
            _ => {}
        })
    }

    fn send_event(
        &mut self,
        id: <<Self as EventLoop>::WindowType as Window>::IdType,
        event: DisplayEvent,
    ) {
        match self.windows.get_mut(&id) {
            Some(sender) => match sender.send(event) {
                Ok(_) => {}
                Err(_) => {
                    info!("The window ID {:?} receiver was disconnected.", id);
                    self.windows.remove(&id);
                }
            },
            None => {
                warn!(
                    "Received an event for an unregistered window with ID {:?}.",
                    id
                );
            }
        }
    }
}

pub struct WinitEventLoopWindowFactory<'a> {
    event_loop: &'a winit::event_loop::EventLoopWindowTarget<DisplayCommand>,
    proxy: EventLoopProxy<DisplayCommand>,
    windows: &'a mut HashMap<<WinitWindow as Window>::IdType, mpsc::SyncSender<DisplayEvent>>,
}

impl<'a> EventLoopWindowFactory<WinitEventLoop> for WinitEventLoopWindowFactory<'a> {
    type LoopType = winit::event_loop::EventLoopWindowTarget<DisplayCommand>;

    fn create_window(&mut self) -> WinitWindow {
        let (sender, receiver) = mpsc::sync_channel(16);
        let proxy = self.proxy.clone();
        let proxy_arc = Box::new(move |cmd| {
            proxy.send_event(cmd).ok();
        });
        let window = WinitWindow::new(self, receiver, proxy_arc);
        let id = window.get_id();
        self.windows.insert(id, sender);
        window
    }

    fn get_backing_loop(&self) -> &Self::LoopType {
        self.event_loop
    }
}
