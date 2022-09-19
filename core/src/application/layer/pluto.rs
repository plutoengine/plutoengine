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

use crate::application::layer::{
    Layer, LayerDependencyManager, LayerId, LayerManager, LayerSwapType, LayerSystemManager,
    LayerSystemProvider, LayerWalker, SystemId,
};
use crate::application::system::System;
use std::any::TypeId;
use std::collections::btree_map::ValuesMut;
use std::collections::BTreeMap;

struct PlutoLayerDependencyManager {
    new_layers: Vec<(LayerId, LayerSwapType, Box<dyn Layer>)>,
}

impl LayerDependencyManager for PlutoLayerDependencyManager {
    fn declare_or_create_dependency<T: Layer>(&mut self, supplier: impl FnOnce() -> T)
    where
        Self: Sized,
    {
        todo!();
    }

    fn declare_dependency<T: Layer>(&mut self) -> Option<()>
    where
        Self: Sized,
    {
        todo!()
    }
}

struct PlutoLayerSystemProxy<'a> {
    systems: BTreeMap<SystemId, &'a mut dyn System>,
}

impl LayerSystemProvider for PlutoLayerSystemProxy<'_> {
    fn query<T: System>(&self) -> Option<&T>
    where
        Self: Sized,
    {
        self.systems
            .get(&TypeId::of::<T>())
            .map(|system| system.as_any())
            .and_then(|system| system.downcast_ref::<T>())
    }

    fn query_mut<T: System>(&mut self) -> Option<&mut T>
    where
        Self: Sized,
    {
        self.systems
            .get_mut(&TypeId::of::<T>())
            .map(|system| system.as_any_mut())
            .and_then(|system| system.downcast_mut::<T>())
    }
}

impl<'a> LayerSystemManager<'a> for PlutoLayerSystemProxy<'a> {
    fn provide_system<T: System>(&mut self, system: &'a mut Box<T>)
    where
        Self: Sized,
    {
        self.systems
            .insert(TypeId::of::<T>(), system.as_system_mut());
    }
}

struct PlutoLayerWalker<'a> {
    layers_iter: ValuesMut<'a, LayerId, LayerInfo>,
}

impl LayerWalker for PlutoLayerWalker<'_> {
    fn next(&mut self, system_proxy: &mut dyn LayerSystemManager) {
        if let Some(layer_info) = self.layers_iter.next() {
            layer_info.layer.on_enter(system_proxy, self);
            layer_info.layer.on_leave(system_proxy);
        }
    }
}

struct LayerInfo {
    id: LayerId,
    layer: Box<dyn Layer>,
}

pub struct PlutoLayerManager {
    layers: BTreeMap<LayerId, LayerInfo>,
    detaching_layers: Vec<(LayerSwapType, Box<dyn Layer>)>,
    proxy: PlutoLayerDependencyManager,
}

impl PlutoLayerManager {
    const LAYER_ID_SPACING: LayerId = 100;

    pub fn new() -> Self {
        Self {
            layers: BTreeMap::new(),
            detaching_layers: Vec::new(),
            proxy: PlutoLayerDependencyManager {
                new_layers: Vec::new(),
            },
        }
    }

    fn detach_poll(&mut self) {
        // Poll detaching layers
        let mut i = 0;
        while i < self.detaching_layers.len() {
            let (swap_type, layer) = &mut self.detaching_layers[i];

            if swap_type.poll_detach(layer) {
                self.detaching_layers.remove(i);
            } else {
                i += 1;
            }
        }
    }

    fn attach_poll(&mut self) {
        // Poll attaching layers
        let mut i = 0;
        while i < self.proxy.new_layers.len() {
            let (id, swap_type, ..) = self.proxy.new_layers[i];
            let (.., layer) = &mut self.proxy.new_layers[i];

            if swap_type.poll_attach(layer) {
                let (.., layer_owned) = self.proxy.new_layers.remove(i);
                self.layers.insert(
                    id,
                    LayerInfo {
                        id,
                        layer: layer_owned,
                    },
                );
            } else {
                i += 1;
            }
        }
    }
}

impl LayerManager for PlutoLayerManager {
    fn add_layer(&mut self, mut layer: Box<dyn Layer>) {
        let id = self
            .layers
            .keys()
            .next_back()
            .map_or(0, |l_id| l_id + Self::LAYER_ID_SPACING);

        layer.on_attach(&mut self.proxy);
        // Manually added layers are always polled to completion (synchronously).
        LayerSwapType::Synchronous.poll_attach(&mut layer);
        self.layers.insert(id, LayerInfo { id, layer });
    }

    fn run(&mut self) -> bool {
        let mut system_proxy = PlutoLayerSystemProxy {
            systems: BTreeMap::new(),
        };

        let mut walker = PlutoLayerWalker {
            layers_iter: self.layers.values_mut(),
        };

        walker.next(&mut system_proxy);

        // Collect all layers that are detaching
        let mut layers_to_detach: Vec<(LayerId, LayerSwapType)> = self
            .layers
            .iter()
            .filter_map(|(id, layer_info)| {
                if let Some(swap_type) = layer_info.layer.should_detach() {
                    Some((*id, swap_type));
                }

                None
            })
            .collect();

        // Remove layers that are detaching
        for (id, swap_type) in layers_to_detach.into_iter() {
            let layer_info = self.layers.remove(&id).unwrap();
            self.detaching_layers.push((swap_type, layer_info.layer));
        }

        self.detach_poll();

        self.attach_poll();

        self.layers.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::application::layer::pluto::PlutoLayerManager;
    use crate::application::layer::{
        Layer, LayerDependencyManager, LayerManager, LayerSwapType, LayerSystemManager, LayerWalker,
    };
    use std::any::{Any, TypeId};

    struct DummyLayer2 {
        enter_count: u32,
    }

    impl Layer for DummyLayer2 {
        fn should_detach(&self) -> Option<LayerSwapType> {
            Some(LayerSwapType::Synchronous)
        }

        fn on_detach(&mut self) {
            assert_eq!(self.enter_count, 1);
        }

        fn on_enter(
            &mut self,
            system_provider: &mut dyn LayerSystemManager,
            layer_walker: &mut dyn LayerWalker,
        ) {
            self.enter_count += 1;

            layer_walker.next(system_provider);
        }
    }

    struct DummyLayer {
        enter_count: u32,
    }

    impl Layer for DummyLayer {
        fn should_detach(&self) -> Option<LayerSwapType> {
            match self.enter_count {
                0..=2 => None,
                3 => Some(LayerSwapType::Synchronous),
                _ => unreachable!("Should not be called more than 3 times!"),
            }
        }

        fn on_attach(&mut self, dependencies: &mut dyn LayerDependencyManager) {
            dependencies.declare_or_create_dependency(|| DummyLayer2 { enter_count: 0 });
        }

        fn on_detach(&mut self) {
            assert_eq!(self.enter_count, 3);
        }

        fn on_enter(
            &mut self,
            system_provider: &mut dyn LayerSystemManager,
            layer_walker: &mut dyn LayerWalker,
        ) {
            self.enter_count += 1;

            layer_walker.next(system_provider);
        }
    }

    #[test]
    fn test_layer_manager() {
        let mut layer_manager = PlutoLayerManager::new();
        layer_manager.add_layer(Box::new(DummyLayer { enter_count: 0 }));

        assert_eq!(
            layer_manager.layers.get(&0).unwrap().type_id(),
            TypeId::of::<DummyLayer2>()
        );

        assert_eq!(
            layer_manager
                .layers
                .get(&PlutoLayerManager::LAYER_ID_SPACING)
                .unwrap()
                .type_id(),
            TypeId::of::<DummyLayer2>()
        );

        loop {
            if layer_manager.run() {
                break;
            }

            assert!(!layer_manager.layers.is_empty());

            let layer = layer_manager
                .layers
                .get(&PlutoLayerManager::LAYER_ID_SPACING);

            assert!(layer.is_some());

            let layer = layer.unwrap().layer.as_any().downcast_ref::<DummyLayer>();

            assert!(layer.is_some());

            let layer = layer.unwrap();

            assert!(layer.enter_count <= 3);
        }
    }
}
