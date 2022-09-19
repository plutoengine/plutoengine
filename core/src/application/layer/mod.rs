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

use crate::application::system::System;
use std::any::{Any, TypeId};

pub mod pluto;

pub trait LayerDependencyManager {
    fn declare_or_create_dependency<T: Layer>(&mut self, supplier: impl FnOnce() -> T)
    where
        Self: Sized;

    fn declare_dependency<T: Layer>(&mut self) -> Option<()>
    where
        Self: Sized;
}

// Don't downcast this to the manager unless you want to be added to the naughty list. >:(
pub trait LayerSystemProvider {
    fn query<T: System>(&self) -> Option<&T>
    where
        Self: Sized;

    fn query_mut<T: System>(&mut self) -> Option<&mut T>
    where
        Self: Sized;
}

pub trait LayerSystemManager<'a>: LayerSystemProvider {
    fn provide_system<T: System>(&mut self, system: &'a mut Box<T>)
    where
        Self: Sized;

    fn as_provider(&self) -> &dyn LayerSystemProvider
    where
        Self: Sized,
    {
        self
    }

    fn as_provider_mut(&mut self) -> &mut dyn LayerSystemProvider
    where
        Self: Sized,
    {
        self
    }
}

type LayerId = u64;

pub trait Layer: 'static {
    fn should_detach(&self) -> Option<LayerSwapType>;

    fn on_attach(&mut self, _dependencies: &mut dyn LayerDependencyManager) {}

    fn on_detach(&mut self) {}

    fn poll_attach(&mut self) -> bool {
        true
    }

    fn poll_detach(&mut self) -> bool {
        true
    }

    fn on_enter(&mut self, systems: &mut dyn LayerSystemManager<'_>, next: &mut dyn LayerWalker) {
        next.next(systems);
    }

    fn on_leave(&mut self, _systems: &mut dyn LayerSystemProvider) {}

    fn as_any(&self) -> &dyn Any
    where
        Self: Sized,
    {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any
    where
        Self: Sized,
    {
        self
    }
}

pub trait LayerWalker {
    fn next(&mut self, _systems: &mut dyn LayerSystemManager<'_>);
}

#[derive(Copy, Clone, Debug)]
pub enum LayerSwapType {
    Synchronous,
    Deferred,
}

impl LayerSwapType {
    fn poll_attach(&self, layer: &mut Box<dyn Layer>) -> bool {
        match self {
            LayerSwapType::Synchronous => {
                loop {
                    if layer.poll_attach() {
                        break;
                    }
                }
                true
            }
            LayerSwapType::Deferred => layer.poll_attach(),
        }
    }

    fn poll_detach(&self, layer: &mut Box<dyn Layer>) -> bool {
        match self {
            LayerSwapType::Synchronous => {
                loop {
                    if layer.poll_detach() {
                        break;
                    }
                }
                true
            }
            LayerSwapType::Deferred => layer.poll_detach(),
        }
    }
}

type SystemId = TypeId;

pub trait LayerManager {
    fn add_layer(&mut self, layer: Box<dyn Layer>);

    fn run(&mut self) -> bool;
}
