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

use crate::device::PhysicalDevice;
use crate::surface::Surface;
use pluto_engine_window::window::Window;

pub trait ContextInstance<'a> {
    type BackingType;

    type PhysicalDeviceType: PhysicalDevice<'a>;
    type SurfaceType: Surface<
        'a,
        DeviceType = <<Self as ContextInstance<'a>>::PhysicalDeviceType as PhysicalDevice<'a>>::DeviceType,
    >;
    type WindowType: Window;

    fn new(window: &'a Self::WindowType) -> Self;

    fn create_device_and_surface(&self) -> (Self::PhysicalDeviceType, Self::SurfaceType);

    fn get_backing_instance(&self) -> &Self::BackingType;
}
