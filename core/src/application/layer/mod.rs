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

use crate::application::system::{System, SystemDyn};
use std::any::{Any, TypeId};

pub mod pluto;

pub struct LayerDependencyDeclaration<'a>(&'a mut dyn LayerDependencyManager);

impl LayerDependencyDeclaration<'_> {
    pub fn required<T: Layer>(&self) -> &T {
        &self
            .0
            .find_by_type(TypeId::of::<T>())
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap()
    }

    pub fn optional<T: Layer>(&self) -> Option<&T> {
        self.0
            .find_by_type(TypeId::of::<T>())
            .and_then(|layer| layer.as_any().downcast_ref())
    }

    pub fn or_create<T: Layer>(&mut self, supplier: impl FnOnce() -> Box<T>) -> &T {
        if self.optional::<T>().is_some() {
            return self
                .optional::<T>()
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
        }

        let layer = supplier();
        self.0.add_layer(layer).as_any().downcast_ref().unwrap()
    }

    pub fn required_mut<T: Layer>(&mut self) -> &mut T {
        self.0
            .find_by_type_mut(TypeId::of::<T>())
            .unwrap()
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    pub fn optional_mut<T: Layer>(&mut self) -> Option<&mut T> {
        self.0
            .find_by_type_mut(TypeId::of::<T>())
            .and_then(|layer| layer.as_any_mut().downcast_mut())
    }

    pub fn or_create_mut<T: Layer>(&mut self, supplier: impl FnOnce() -> Box<T>) -> &mut T {
        if self.optional_mut::<T>().is_some() {
            return self
                .optional_mut::<T>()
                .unwrap()
                .as_any_mut()
                .downcast_mut()
                .unwrap();
        }

        let layer = supplier();
        self.0.add_layer(layer).as_any_mut().downcast_mut().unwrap()
    }
}

pub trait LayerDependencyManager {
    fn find_by_type(&self, layer_type: TypeId) -> Option<&dyn Layer>;

    fn find_by_type_mut(&mut self, layer_type: TypeId) -> Option<&mut dyn Layer>;

    fn add_layer(&mut self, layer: Box<dyn Layer>) -> &mut dyn Layer;
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

pub trait LayerSystemManager<'a>: LayerSystemProvider + AsProvider {
    fn provide_system<T: System>(&mut self, system: &'a mut Box<T>)
    where
        Self: Sized;
}

pub trait AsProvider {
    fn as_provider(&self) -> &dyn LayerSystemProvider;

    fn as_provider_mut(&mut self) -> &mut dyn LayerSystemProvider;
}

impl<'a, T> AsProvider for T
where
    T: LayerSystemManager<'a> + LayerSystemProvider,
{
    fn as_provider(&self) -> &dyn LayerSystemProvider {
        self
    }

    fn as_provider_mut(&mut self) -> &mut dyn LayerSystemProvider {
        self
    }
}

pub trait LayerObj: 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Layer: LayerObj {
    fn should_detach(&self) -> Option<LayerSwapType>;

    fn on_attach(&mut self, _dependencies: &mut LayerDependencyDeclaration) {}

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
}

impl<T: Layer> LayerObj for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
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
