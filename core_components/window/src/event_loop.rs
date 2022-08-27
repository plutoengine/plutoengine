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

use crate::window::{Window, WindowEvent};
use std::convert::Infallible;

#[derive(Copy, Clone, Debug)]
pub enum DisplayEvent {
    Disconnected,
    Repaint,
    NextFrame,
    WindowEvent(WindowEvent),
}

#[derive(Copy, Clone, Debug)]
pub enum DisplayCommand {}

pub trait EventLoop: 'static {
    type WindowType: Window + Send;
    type LoopType: 'static;

    fn run<F: FnOnce(&mut dyn EventLoopWindowFactory<Self, LoopType = Self::LoopType>) + 'static>(
        initializer: F,
    ) -> Infallible
    where
        Self: Sized;

    fn send_event(
        &mut self,
        id: <<Self as EventLoop>::WindowType as Window>::IdType,
        event: DisplayEvent,
    );
}

pub trait EventLoopWindowFactory<E: EventLoop> {
    type LoopType: 'static;

    fn create_window(&mut self) -> E::WindowType;

    fn get_backing_loop(&self) -> &Self::LoopType;
}
