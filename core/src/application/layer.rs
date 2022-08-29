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
use std::collections::BTreeMap;
use std::process::id;
use std::slice::Iter;

pub trait LayerDependencyManager {
    fn declare_dependency<T: Layer>(&mut self, supplier: impl FnOnce() -> T)
    where
        Self: Sized;

    fn declare_dependency_required<T: Layer>(&mut self) -> Option<()>
    where
        Self: Sized;
}

// Don't downcast this to the manager unless you want to be added to the naughty list. >:(
pub trait LayerSystemProvider {
    fn query<T: System>(&self) -> Iter<&T>
    where
        Self: Sized;

    fn query_mut<T: System>(&mut self) -> Iter<&mut T>
    where
        Self: Sized;
}

pub trait LayerSystemManager: LayerSystemProvider {
    fn provide_system<T: System>(&mut self, system: &mut T)
    where
        Self: Sized;
}

pub trait Layer {
    fn should_detach(&self) -> Option<LayerSwapType>;

    fn on_attach(&mut self, _dependencies: &mut dyn LayerDependencyManager) {}

    fn on_detach(&mut self) {}

    fn poll_attach(&mut self) -> bool {
        true
    }

    fn poll_detach(&mut self) -> bool {
        true
    }

    fn on_enter(&mut self, _systems: &mut dyn LayerSystemManager) {}

    fn on_leave(&mut self, _systems: &mut dyn LayerSystemProvider) {}
}

struct ZombieLayer;

impl Layer for ZombieLayer {
    fn should_detach(&self) -> Option<LayerSwapType> {
        None
    }
}

type LayerId = u64;

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

struct LayerManagerProxy {
    new_layers: Vec<(LayerId, LayerSwapType, Box<dyn Layer>)>,
}

impl LayerDependencyManager for LayerManagerProxy {
    fn declare_dependency<T: Layer>(&mut self, supplier: impl FnOnce() -> T)
    where
        Self: Sized,
    {
        todo!()
    }

    fn declare_dependency_required<T: Layer>(&mut self) -> Option<()>
    where
        Self: Sized,
    {
        todo!()
    }
}

struct LayerSystemProxy;

impl LayerSystemProvider for LayerSystemProxy {
    fn query<T: System>(&self) -> Iter<&T>
    where
        Self: Sized,
    {
        todo!()
    }

    fn query_mut<T: System>(&mut self) -> Iter<&mut T>
    where
        Self: Sized,
    {
        todo!()
    }
}

impl LayerSystemManager for LayerSystemProxy {
    fn provide_system<T: System>(&mut self, system: &mut T)
    where
        Self: Sized,
    {
        todo!()
    }
}

pub struct LayerManager {
    layers: BTreeMap<LayerId, Box<dyn Layer>>,
    proxy: LayerManagerProxy,
    system_proxy: LayerSystemProxy,
}

impl LayerManager {
    const LAYER_ID_SPACING: LayerId = 100;

    pub fn new() -> Self {
        Self {
            layers: BTreeMap::new(),
            proxy: LayerManagerProxy {
                new_layers: Vec::new(),
            },
            system_proxy: LayerSystemProxy,
        }
    }

    pub fn add_layer(&mut self, mut layer: Box<dyn Layer>) {
        let key_id = self
            .layers
            .keys()
            .next_back()
            .map_or(0, |l_id| l_id + Self::LAYER_ID_SPACING);

        layer.on_attach(&mut self.proxy);
        // Manually added layers are always polled to completion (synchronously).
        LayerSwapType::Synchronous.poll_attach(&mut layer);
        self.layers.insert(key_id, layer);
    }

    pub fn run(&mut self) {
        let mut layers_to_detach: Vec<(LayerId, LayerSwapType)> = Vec::new();
        let mut detaching_layers: Vec<(LayerSwapType, Box<dyn Layer>)> = Vec::new();

        loop {
            for layer in self.layers.values_mut() {
                layer.on_enter(&mut self.system_proxy);
            }

            for layer in self.layers.values_mut() {
                layer.on_leave(&mut self.system_proxy);
            }

            // Remove layers that are detaching
            for (id, layer) in self.layers.iter() {
                if let Some(swap_type) = layer.should_detach() {
                    layers_to_detach.push((*id, swap_type));
                }
            }

            for (id, swap_type) in layers_to_detach.drain(..) {
                let layer = self.layers.remove(&id).unwrap();
                detaching_layers.push((swap_type, layer));
            }

            // Poll detaching layers
            let mut i = 0;
            while i < detaching_layers.len() {
                let (swap_type, layer) = &mut detaching_layers[i];
                if swap_type.poll_detach(layer) {
                    detaching_layers.remove(i);
                } else {
                    i += 1;
                }
            }

            // Poll attaching layers
            let mut i = 0;
            while i < self.proxy.new_layers.len() {
                let (id, swap_type, ..) = self.proxy.new_layers[i];
                let (.., layer) = &mut self.proxy.new_layers[i];

                if swap_type.poll_attach(layer) {
                    let (.., layer_owned) = self.proxy.new_layers.remove(i);
                    self.layers.insert(id, layer_owned);
                } else {
                    i += 1;
                }
            }

            // Quit if no layers are left
            if self.layers.is_empty() {
                break;
            }
        }
    }
}
