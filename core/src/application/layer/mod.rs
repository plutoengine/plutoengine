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

/// An object used to declare dependencies between layers.
pub struct LayerDependencyDeclaration<'a>(&'a mut dyn LayerDependencyManager);

impl LayerDependencyDeclaration<'_> {
    /// Declares a dependency on a layer, returning a reference to it.
    ///
    /// ***Panics** if such layer does not exist.*
    pub fn required<T: Layer>(&self) -> &T {
        &self
            .0
            .find_by_type(TypeId::of::<T>())
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap()
    }

    /// Declares a dependency on a layer, returning a reference to it if it exists.
    ///
    /// *Returns `None` if such layer does not exist.*
    pub fn optional<T: Layer>(&self) -> Option<&T> {
        self.0
            .find_by_type(TypeId::of::<T>())
            .and_then(|layer| layer.as_any().downcast_ref())
    }

    /// Declares a dependency on a layer, returning a reference to it.
    ///
    /// *Always returns a valid valid layer, creating one if there does not exist
    /// a layer of the given type.* **This applies recursively. If a dependency
    /// further down the dependency tree cannot be satisfied, it may cause a panic.**
    /// It is therefore recommended to for each layer to bring its own dependencies.
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

    /// Declares a dependency on a layer, returning a mutable reference to it.
    ///
    /// ***Panics** if such layer does not exist.*
    pub fn required_mut<T: Layer>(&mut self) -> &mut T {
        self.0
            .find_by_type_mut(TypeId::of::<T>())
            .unwrap()
            .as_any_mut()
            .downcast_mut()
            .unwrap()
    }

    /// Declares a dependency on a layer, returning a mutable reference to it.
    ///
    /// *Returns `None` if such layer does not exist.*
    pub fn optional_mut<T: Layer>(&mut self) -> Option<&mut T> {
        self.0
            .find_by_type_mut(TypeId::of::<T>())
            .and_then(|layer| layer.as_any_mut().downcast_mut())
    }

    /// Declares a dependency on a layer, returning a mutable reference to it.
    ///
    /// *Always returns a valid layer, creating one if there does not exist
    /// a layer of the given type.* **This applies recursively. If this dependency's
    /// dependencies further down the tree cannot be satisfied, it may cause a panic.**
    /// It is therefore recommended to for each layer to bring its own dependencies.
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

/// A proxy trait for [`Layer`] that allows layers to declare dependencies on other layers.
pub trait LayerDependencyManager {
    /// Returns a reference to the layer with the given type, if it exists.
    fn find_by_type(&self, layer_type: TypeId) -> Option<&dyn Layer>;

    /// Returns a mutable reference to the layer with the given type, if it exists.
    fn find_by_type_mut(&mut self, layer_type: TypeId) -> Option<&mut dyn Layer>;

    /// Adds a layer to the list of layers to be attached.
    fn add_layer(&mut self, layer: Box<dyn Layer>) -> &mut dyn Layer;
}

/// A trait for querying the layer manager for available systems provided by other layers.
///
/// *Don't downcast this to the manager unless you want to be added to the naughty list. >:(*
pub trait LayerSystemProvider {
    /// Returns a reference to the system of the given type, if it exists.
    fn query<T: System>(&self) -> Option<&T>
    where
        Self: Sized;

    /// Returns a mutable reference to the system of the given type, if it exists.
    fn query_mut<T: System>(&mut self) -> Option<&mut T>
    where
        Self: Sized;
}

/// A trait for layers to provide the layers above this one with additional systems.
///
/// This method is only available when traversing the stack upwards, any systems provided
/// are automatically popped when the layer is traversed downwards.
pub trait LayerSystemManager<'a>: LayerSystemProvider + AsProvider {
    fn provide_system<T: System>(&mut self, system: &'a mut Box<T>)
    where
        Self: Sized;
}

/// A utility trait for downcasting of the layer manager proxy to the layer provider proxy.
pub trait AsProvider {
    /// Downcasts the layer manager proxy to a reference to the layer provider proxy.
    fn as_provider(&self) -> &dyn LayerSystemProvider;

    /// Downcasts the layer manager proxy to a mutable reference the layer provider proxy.
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

/// A utility trait for dynamic typing of layers.
pub trait LayerObj: 'static {
    /// Converts a layer reference to an `Any` reference.
    fn as_any(&self) -> &dyn Any;

    /// Converts a layer mutable reference to an `Any` mutable reference.
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait Layer: LayerObj {
    /// Returns a layer swap strategy if the layer should be detached.
    ///
    /// *Returns `None` if the layer should not be detached.*
    fn should_detach(&self) -> Option<LayerSwapType>;

    /// An event that is called **before** the layer is attached to the layer stack.
    fn on_attach(&mut self, _dependencies: &mut LayerDependencyDeclaration) {}

    /// An event that is called **before** the layer is detached from the layer stack.
    fn on_detach(&mut self) {}

    /// Polls the layer until it is ready to be attached to the layer stack.
    ///
    /// *Returns `true` if the layer is ready to be attached.*
    ///
    /// Returning `false` will cause the layer to be polled again at some point in the future.
    /// This may be used to distribute heavy initialization work across multiple loops.
    fn poll_attach(&mut self) -> bool {
        true
    }

    /// Polls the layer until it is ready to be detached from the layer stack.
    ///
    /// *Returns `true` if the layer is ready to be detached.*
    ///
    /// Returning `false` will cause the layer to be polled again at some point in the future.
    /// This may be used to distribute heavy cleanup work across multiple loops.
    fn poll_detach(&mut self) -> bool {
        true
    }

    /// An event that is called when the layer is traversed **upwards**.
    ///
    /// The `systems` parameter provides all available systems provided by layers below this one.
    /// New systems may be provided to layers above this one by calling the
    /// [`LayerSystemManager::provide_system`] method.
    /// *These systems will be automatically popped when this layer is traversed downwards.*
    ///
    /// The `next` function MUST be called to continue the traversal.
    fn on_enter(&mut self, systems: &mut dyn LayerSystemManager<'_>, next: &mut dyn LayerWalker) {
        next.next(systems);
    }

    /// An event that is called when the layer is traversed **downwards**.
    ///
    /// The `systems` parameter provides all available systems provided by layers below this one.
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

/// A layer walker is used to traverse the layer stack, visiting each layer.
///
/// Layers may provide systems for other layers "above" it to use.
///
/// To visit the next layer, call `next()`.
pub trait LayerWalker {
    fn next(&mut self, systems: &mut dyn LayerSystemManager<'_>);
}

/// A strategy for swapping layers.
///
/// *This is used to determine how a layer should be attached and detached.*
#[derive(Copy, Clone, Debug, Default)]
pub enum LayerSwapType {
    /// The layer swap will be polled to completion synchronously in one iteration.
    /// This behavior is guaranteed to complete in a single loop.
    ///
    /// *This is the default strategy.*
    #[default]
    Synchronous,
    /// The layer swap will be polled once per iteration,
    /// it is up to the layer to decide when it is done swapping.
    Deferred,
}

impl LayerSwapType {
    /// Runs a layer swap strategy to attach a layer.
    ///
    /// Returns `true` if the poll completed.
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

    /// Runs a layer swap strategy to detach a layer.
    ///
    /// Returns `true` if the poll completed.
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

/// Systems are identified by their type.
type SystemId = TypeId;

/// A base trait for layer managers.
///
/// Layer managers are structures providing layer management and traversal functionality.
///
/// First, a layer manager is initialized with a stack of layers.
///
/// Then, the layer manager is traversed, visiting each layer in the stack using the `run` method.
/// The `run` method should be called repeatedly until it returns `true`.
pub trait LayerManager {
    /// Adds a layer to the top of the layer "stack".
    ///
    /// This method should only be called before the layer manager is first run.
    fn add_layer(&mut self, layer: Box<dyn Layer>);

    /// Runs a single iteration of the layer manager.
    ///
    /// *Layers are traversed first bottom to top, then top to bottom.*
    ///
    /// Returns `true` if the layer manager has finished running, that is whether no
    /// layers are attached and no layers are polled to be attached.
    fn run(&mut self) -> bool;
}
