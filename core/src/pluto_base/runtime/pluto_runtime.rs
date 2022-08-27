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

use crate::pluto_base::runtime::{ApplicationBootstrapper, Runtime};

use pluto_engine_display::pluto_engine_window::event_loop::{EventLoop, EventLoopWindowFactory};
use std::convert::Infallible;
use std::thread;

pub struct PlutoRuntime;

impl<E: EventLoop> Runtime<E> for PlutoRuntime {
    fn run(bootstrapper: ApplicationBootstrapper<E>) -> Infallible {
        E::run(move |evt_loop| {
            let runtime = Self;
            runtime.create_application(evt_loop, bootstrapper);
        })
    }

    fn spawn_application_worker<F: FnOnce() + Send + 'static>(&self, worker: F) {
        thread::spawn(worker);
    }

    fn create_application<ELW: EventLoopWindowFactory<E> + ?Sized>(
        &self,
        event_loop: &mut ELW,
        bootstrapper: ApplicationBootstrapper<E>,
    ) {
        let window = event_loop.create_window();
        <PlutoRuntime as Runtime<E>>::spawn_application_worker(self, move || {
            bootstrapper.bootstrap(window);
        });
    }
}
