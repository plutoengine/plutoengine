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
use pluto_engine_display::pluto_engine_window::event_loop::{EventLoop, EventLoopWindowFactory};
use pluto_engine_display::pluto_engine_window::window::Window;
use pluto_engine_display::{ApplicationDisplay, ApplicationState};
use std::convert::Infallible;

pub mod pluto_runtime;

pub mod platform {
    cfg_if::cfg_if! {
        if #[cfg(feature = "pe_window_winit")] {
            pub mod winit;
        }
    }
}

pub struct ApplicationBootstrapper<E>(Box<dyn FnOnce(E::WindowType) + Send + 'static>)
where
    E: EventLoop;

impl<E> ApplicationBootstrapper<E>
where
    E: EventLoop,
{
    pub fn default_loop<'a, AD: ApplicationDisplay<'a>>(state: &mut impl ApplicationState<'a, AD>) {
        loop {
            let display = state.display();
            if display.close_requested() {
                break;
            }

            let event = display.get_window().receive_event();
            ApplicationDisplay::on_event(display, event)(state);
        }

        info!(
            "Window ID {:?} close requested.",
            state.display().get_window().get_id()
        );
    }

    pub fn new(main_loop: Box<dyn FnOnce(E::WindowType) + Send + 'static>) -> Self {
        Self(main_loop)
    }

    pub fn bootstrap(self, window: E::WindowType) {
        self.0(window)
    }
}

pub trait Runtime<E: EventLoop>: 'static {
    fn run(bootstrapper: ApplicationBootstrapper<E>) -> Infallible;

    fn spawn_application_worker<F: FnOnce() + Send + 'static>(&self, worker: F);

    fn create_application<ELW: EventLoopWindowFactory<E> + ?Sized>(
        &self,
        event_loop: &mut ELW,
        bootstrapper: ApplicationBootstrapper<E>,
    );
}
