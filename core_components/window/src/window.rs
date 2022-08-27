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

use crate::event_loop::{DisplayCommand, DisplayEvent, EventLoop, EventLoopWindowFactory};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::mpsc::Receiver;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct PhysicalSize<S> {
    pub width: S,
    pub height: S,
}

#[derive(Copy, Clone, Debug)]
pub enum WindowEvent {
    CloseRequested,
    Resized(PhysicalSize<u32>),
    Unknown,
}

pub trait WindowEventReceiver<T: Into<WindowEvent>>: Window {
    type EventType: Into<WindowEvent>;
}

pub trait Window {
    type IdType: Copy + Clone + Eq + Hash + Debug;
    type BackingType;
    type SizeType;
    type LoopType;

    fn new<
        EL: EventLoop<WindowType = Self> + 'static,
        ELW: EventLoopWindowFactory<EL, LoopType = Self::LoopType>,
    >(
        event_loop: &ELW,
        event_receiver: Receiver<DisplayEvent>,
        command_proxy: Box<dyn Fn(DisplayCommand) + Send>,
    ) -> Self;

    fn receive_event(&self) -> DisplayEvent;

    fn request_repaint(&self);

    fn get_id(&self) -> Self::IdType;

    fn get_size(&self) -> PhysicalSize<Self::SizeType>;

    fn get_backing_window(&self) -> &Self::BackingType;
}
