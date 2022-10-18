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
    Layer, LayerDependencyDeclaration, LayerDependencyManager, LayerManager, LayerSwapType,
    LayerSystemManager, LayerSystemProvider, LayerWalker, SystemId,
};
use crate::application::system::System;
use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Formatter};
use std::slice::IterMut;

type LayerId = u64;

struct PlutoLayerProxy {
    new_layers: VecDeque<(LayerSwapType, Box<dyn Layer>)>,
}

struct PlutoLayerDependencyManager<'a> {
    manager: &'a mut PlutoLayerManager,
}

impl LayerDependencyManager for PlutoLayerDependencyManager<'_> {
    fn find_by_type(&self, layer_type: TypeId) -> Option<&dyn Layer> {
        Some(
            self.manager
                .layers
                .values()
                .find(|l| l.layer.type_id() == layer_type)?
                .layer
                .as_ref(),
        )
    }

    fn find_by_type_mut(&mut self, layer_type: TypeId) -> Option<&mut dyn Layer> {
        Some(
            self.manager
                .layers
                .values_mut()
                .find(|l| l.layer.type_id() == layer_type)?
                .layer
                .as_mut(),
        )
    }

    fn add_layer(&mut self, mut layer: Box<dyn Layer>) -> &mut dyn Layer {
        self.manager
            .proxy
            .new_layers
            .push_back((LayerSwapType::Synchronous, layer));

        self.manager.proxy.new_layers.back_mut().unwrap().1.as_mut()
    }
}

struct PlutoLayerSystemProxy<'a> {
    systems: HashMap<SystemId, &'a mut dyn System>,
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
    layers: IterMut<'a, *mut LayerInfo>,
}

impl LayerWalker for PlutoLayerWalker<'_> {
    fn next(&mut self, system_proxy: &mut dyn LayerSystemManager) {
        if let Some(&mut layer_info) = self.layers.next() {
            let layer_info = unsafe { &mut *layer_info };
            layer_info.layer.on_enter(system_proxy, self);
            layer_info.layer.on_leave(system_proxy.as_provider_mut());
        }
    }
}

struct LayerInfo {
    id: LayerId,
    layer: Box<dyn Layer>,
}

impl Debug for LayerInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LayerInfo {{ id: {}, layer: {:?} }}",
            self.id,
            <dyn Layer>::as_any(&*self.layer).type_id()
        )
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
enum TraversalChainNode {
    Start,
    End,
    Link(LayerId),
}

#[derive(Clone)]
struct TraversalChainWalker<'a>(TraversalChainNode, &'a TraversalChain);

impl Iterator for TraversalChainWalker<'_> {
    type Item = LayerId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 = self.1.get_next(&self.0);

        match self.0 {
            TraversalChainNode::Start => unreachable!(),
            TraversalChainNode::End => None,
            TraversalChainNode::Link(layer_id) => Some(layer_id),
        }
    }
}

struct TraversalChain {
    fwd_chain: HashMap<TraversalChainNode, TraversalChainNode>,
    bwd_chain: HashMap<TraversalChainNode, TraversalChainNode>,
}

impl TraversalChain {
    fn new() -> Self {
        let mut fwd_chain = HashMap::new();
        fwd_chain.insert(TraversalChainNode::Start, TraversalChainNode::End);

        let mut bwd_chain = HashMap::new();
        bwd_chain.insert(TraversalChainNode::End, TraversalChainNode::Start);

        Self {
            fwd_chain,
            bwd_chain,
        }
    }

    fn remove(&mut self, id: LayerId) {
        let link = &TraversalChainNode::Link(id);

        let next = self.fwd_chain[link];
        let prev = self.bwd_chain[link];

        self.fwd_chain.insert(prev, next);
        self.bwd_chain.insert(next, prev);

        self.fwd_chain.remove(link);
        self.bwd_chain.remove(link);
    }

    fn insert_after(&mut self, id: LayerId, after: LayerId) {
        let link = TraversalChainNode::Link(id);
        let after_link = TraversalChainNode::Link(after);

        let next = self.fwd_chain[&after_link];
        let prev = after_link;

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    fn insert_before(&mut self, id: LayerId, before: LayerId) {
        let link = TraversalChainNode::Link(id);
        let before_link = TraversalChainNode::Link(before);

        let next = before_link;
        let prev = self.bwd_chain[&before_link];

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    fn insert_first(&mut self, id: LayerId) {
        let link = TraversalChainNode::Link(id);
        let next = self.fwd_chain[&TraversalChainNode::Start];
        let prev = TraversalChainNode::Start;

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    fn insert_last(&mut self, id: LayerId) {
        let link = TraversalChainNode::Link(id);
        let next = TraversalChainNode::End;
        let prev = self.bwd_chain[&TraversalChainNode::End];

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    fn get_next(&self, node: &TraversalChainNode) -> TraversalChainNode {
        self.fwd_chain[node]
    }

    fn iter(&self) -> TraversalChainWalker {
        TraversalChainWalker(TraversalChainNode::Start, self)
    }
}

pub struct PlutoLayerManager {
    traversal_chain: TraversalChain,
    layers: HashMap<LayerId, LayerInfo>,
    detaching_layers: Vec<(LayerSwapType, Box<dyn Layer>)>,
    proxy: PlutoLayerProxy,
    id_counter: LayerId,
}

impl PlutoLayerManager {
    pub fn new() -> Self {
        Self {
            traversal_chain: TraversalChain::new(),
            layers: HashMap::new(),
            detaching_layers: Vec::new(),
            proxy: PlutoLayerProxy {
                new_layers: VecDeque::new(),
            },
            id_counter: 0,
        }
    }

    fn create_id(&mut self) -> LayerId {
        let id = self.id_counter;
        self.id_counter += 1;
        id
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
            let (swap_type, ..) = self.proxy.new_layers[i];
            let (.., layer) = &mut self.proxy.new_layers[i];

            if swap_type.poll_attach(layer) {
                let (.., layer_owned) = self.proxy.new_layers.remove(i).unwrap();
                let id = self.create_id();
                self.layers.insert(
                    id,
                    LayerInfo {
                        id,
                        layer: layer_owned,
                    },
                );
                self.traversal_chain.insert_last(id);
            } else {
                i += 1;
            }
        }
    }
}

impl LayerManager for PlutoLayerManager {
    fn add_layer(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach(&mut LayerDependencyDeclaration(
            &mut PlutoLayerDependencyManager { manager: self },
        ));

        while let Some((.., layer)) = self.proxy.new_layers.pop_front() {
            self.add_layer(layer);
        }

        // Manually added layers are always polled to completion (synchronously).
        LayerSwapType::Synchronous.poll_attach(&mut layer);

        let id = self.create_id();
        let info = LayerInfo { id, layer };
        self.layers.insert(id, info);
        self.traversal_chain.insert_last(id);
    }

    fn run(&mut self) -> bool {
        let mut system_proxy = PlutoLayerSystemProxy {
            systems: HashMap::new(),
        };

        let layers_iter = self.traversal_chain.iter();
        let mut layers = layers_iter
            .map(|id| self.layers.get_mut(&id).unwrap() as *mut LayerInfo)
            .collect::<Vec<_>>();

        let mut walker = PlutoLayerWalker {
            layers: layers.iter_mut(),
        };

        walker.next(&mut system_proxy);

        // Collect all layers that are detaching
        let layers_to_detach: Vec<(LayerId, LayerSwapType)> = self
            .layers
            .iter()
            .filter_map(|(id, layer_info)| {
                if let Some(swap_type) = layer_info.layer.should_detach() {
                    return Some((*id, swap_type));
                }

                None
            })
            .collect();

        // Remove layers that are detaching
        for (id, swap_type) in layers_to_detach.into_iter() {
            let layer_info = self.layers.remove(&id).unwrap();
            self.traversal_chain.remove(id);
            self.detaching_layers.push((swap_type, layer_info.layer));
        }

        self.detach_poll();

        self.attach_poll();

        self.layers.is_empty()
    }
}

#[cfg(test)]
mod test {
    use crate::application::layer::pluto::{PlutoLayerManager, TraversalChainNode};
    use crate::application::layer::{
        Layer, LayerDependencyDeclaration, LayerManager, LayerSwapType, LayerSystemManager,
        LayerWalker,
    };
    use log::debug;
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
                _ => unreachable!(
                    "Should not be called more than 3 times ({})!",
                    self.enter_count
                ),
            }
        }

        fn on_attach(&mut self, dependencies: &mut LayerDependencyDeclaration) {
            dependencies.or_create(|| Box::new(DummyLayer2 { enter_count: 0 }));
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

    /// A single layer with one dependency is added to the layer manager.
    /// Two layers should be present.
    #[test]
    fn test_dependencies() {
        let mut manager = PlutoLayerManager::new();
        manager.add_layer(Box::new(DummyLayer { enter_count: 0 }));

        println!("{:?}", manager.layers);

        assert_eq!(manager.layers.len(), 2);

        assert_eq!(
            <dyn Layer>::as_any(&*manager.layers.get(&0).unwrap().layer).type_id(),
            TypeId::of::<DummyLayer2>()
        );

        assert_eq!(
            <dyn Layer>::as_any(&*manager.layers.get(&1).unwrap().layer).type_id(),
            TypeId::of::<DummyLayer>()
        );
    }

    #[test]
    fn test_traversal_chain() {
        let mut layer_manager = PlutoLayerManager::new();
        layer_manager.add_layer(Box::new(DummyLayer { enter_count: 0 }));

        println!("{:?}", layer_manager.traversal_chain.fwd_chain);

        assert_eq!(layer_manager.traversal_chain.fwd_chain.len(), 3);
        assert_eq!(layer_manager.traversal_chain.bwd_chain.len(), 3);

        let mut node = TraversalChainNode::Start;

        for i in 0..layer_manager.traversal_chain.fwd_chain.len() {
            let next_node = layer_manager.traversal_chain.fwd_chain.get(&node).unwrap();
            assert_ne!(next_node, &node);
            node = next_node.clone();
        }

        assert_eq!(node, TraversalChainNode::End);
    }

    #[test]
    fn test_unmount() {
        let mut layer_manager = PlutoLayerManager::new();
        layer_manager.add_layer(Box::new(DummyLayer { enter_count: 0 }));

        println!("{:?}", layer_manager.traversal_chain.fwd_chain);

        loop {
            if layer_manager.run() {
                break;
            }

            assert!(!layer_manager.layers.is_empty());

            let layer = layer_manager.layers.get(&1);

            assert!(layer.is_some());

            let layer = layer.unwrap().layer.as_any().downcast_ref::<DummyLayer>();

            assert!(layer.is_some());

            let layer = layer.unwrap();

            assert!(layer.enter_count <= 3);
        }

        assert_eq!(layer_manager.layers.len(), 0);
    }

    #[test]
    fn test_traversal_chain_deconstruct() {
        let mut layer_manager = PlutoLayerManager::new();
        layer_manager.add_layer(Box::new(DummyLayer { enter_count: 0 }));

        println!("{:?}", layer_manager.traversal_chain.fwd_chain);

        assert_eq!(layer_manager.traversal_chain.fwd_chain.len(), 3);
        assert_eq!(layer_manager.traversal_chain.bwd_chain.len(), 3);

        loop {
            if layer_manager.run() {
                break;
            }
        }

        assert_eq!(layer_manager.traversal_chain.fwd_chain.len(), 1);
        assert_eq!(layer_manager.traversal_chain.bwd_chain.len(), 1);
    }
}
