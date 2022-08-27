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

use pluto_engine_render::pipeline::{Pipeline, PipelineLayout};
use std::marker::PhantomData;

pub struct WgpuPipelineLayout<'a> {
    pub(crate) layout: wgpu::PipelineLayout,
    pub(crate) parent: PhantomData<&'a ()>,
}

impl<'a> PipelineLayout<'_> for WgpuPipelineLayout<'a> {
    type BackingType = wgpu::PipelineLayout;

    fn get_backing_pipeline_layout(&self) -> &Self::BackingType {
        &self.layout
    }
}

pub struct WgpuPipeline<'a> {
    pub(crate) pipeline: wgpu::RenderPipeline,
    pub(crate) parent: PhantomData<&'a ()>,
}

impl<'a> Pipeline<'_> for WgpuPipeline<'a> {
    type BackingType = wgpu::RenderPipeline;
    type LayoutType = WgpuPipelineLayout<'a>;

    fn get_backing_pipeline(&self) -> &Self::BackingType {
        &self.pipeline
    }
}
