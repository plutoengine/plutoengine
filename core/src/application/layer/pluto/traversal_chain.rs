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

use crate::application::layer::pluto::LayerId;
use std::collections::HashMap;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub(super) enum TraversalChainNode {
    Start,
    End,
    Link(LayerId),
}

#[derive(Clone)]
pub(super) struct TraversalChainWalker<'a>(
    pub(super) TraversalChainNode,
    pub(super) &'a TraversalChain,
);

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

pub(super) struct TraversalChain {
    pub(super) fwd_chain: HashMap<TraversalChainNode, TraversalChainNode>,
    pub(super) bwd_chain: HashMap<TraversalChainNode, TraversalChainNode>,
}

impl TraversalChain {
    pub(super) fn new() -> Self {
        let mut fwd_chain = HashMap::new();
        fwd_chain.insert(TraversalChainNode::Start, TraversalChainNode::End);

        let mut bwd_chain = HashMap::new();
        bwd_chain.insert(TraversalChainNode::End, TraversalChainNode::Start);

        Self {
            fwd_chain,
            bwd_chain,
        }
    }

    pub(super) fn remove(&mut self, id: LayerId) {
        let link = &TraversalChainNode::Link(id);

        let next = self.fwd_chain[link];
        let prev = self.bwd_chain[link];

        self.fwd_chain.insert(prev, next);
        self.bwd_chain.insert(next, prev);

        self.fwd_chain.remove(link);
        self.bwd_chain.remove(link);
    }

    pub(super) fn insert_after(&mut self, id: LayerId, after: LayerId) {
        let link = TraversalChainNode::Link(id);
        let after_link = TraversalChainNode::Link(after);

        let next = self.fwd_chain[&after_link];
        let prev = after_link;

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    pub(super) fn insert_before(&mut self, id: LayerId, before: LayerId) {
        let link = TraversalChainNode::Link(id);
        let before_link = TraversalChainNode::Link(before);

        let next = before_link;
        let prev = self.bwd_chain[&before_link];

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    pub(super) fn insert_first(&mut self, id: LayerId) {
        let link = TraversalChainNode::Link(id);
        let next = self.fwd_chain[&TraversalChainNode::Start];
        let prev = TraversalChainNode::Start;

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    pub(super) fn insert_last(&mut self, id: LayerId) {
        let link = TraversalChainNode::Link(id);
        let next = TraversalChainNode::End;
        let prev = self.bwd_chain[&TraversalChainNode::End];

        self.fwd_chain.insert(prev, link);
        self.fwd_chain.insert(link, next);

        self.bwd_chain.insert(next, link);
        self.bwd_chain.insert(link, prev);
    }

    pub(super) fn get_next(&self, node: &TraversalChainNode) -> TraversalChainNode {
        self.fwd_chain[node]
    }

    pub(super) fn iter(&self) -> TraversalChainWalker {
        TraversalChainWalker(TraversalChainNode::Start, self)
    }
}
