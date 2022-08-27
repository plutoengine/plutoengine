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

use crate::mesh::WgpuAttribute;
use crate::pipeline::{WgpuPipeline, WgpuPipelineLayout};
use crate::shader::WgpuShader;
use crate::texture::{WgpuTexture, WgpuTextureFormat};
use pluto_engine_render::device::{
    CommandBuffer, CommandBufferBuilder, Device, PhysicalDevice, Queue,
};
use pluto_engine_render::mesh::MeshLayout;
use pluto_engine_render::pipeline::{PipelineCreateInfo, PipelineLayout};
use pluto_engine_render::shader::{Shader, ShaderCode};
use pluto_engine_render::texture::TextureFormat;
use smallvec::SmallVec;
use std::borrow::Cow;
use std::marker::PhantomData;
use wgpu::{BufferAddress, VertexBufferLayout, VertexStepMode};

pub struct WgpuQueue<'a>(wgpu::Queue, PhantomData<&'a ()>);

impl<'a> Queue<'_> for WgpuQueue<'a> {
    type BackingType = wgpu::Queue;

    fn get_backing_queue(&self) -> &Self::BackingType {
        &self.0
    }
}

pub struct WgpuPhysicalDevice<'a>(wgpu::Adapter, PhantomData<&'a ()>);

impl<'a> PhysicalDevice<'_> for WgpuPhysicalDevice<'a> {
    type BackingType = wgpu::Adapter;

    type DeviceType = WgpuDevice<'a>;
    type QueueType = WgpuQueue<'a>;

    fn new(adapter: Self::BackingType) -> Self {
        Self(adapter, PhantomData)
    }

    fn get_backing_physical_device(&self) -> &Self::BackingType {
        &self.0
    }

    fn create_device_and_queue(&self) -> (Self::DeviceType, Self::QueueType) {
        let (device, queue) = pollster::block_on(self.0.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ))
        .unwrap();

        (
            WgpuDevice(device, PhantomData),
            WgpuQueue(queue, PhantomData),
        )
    }
}

pub struct WgpuDevice<'a>(wgpu::Device, PhantomData<&'a ()>);

impl<'a> Device<'_> for WgpuDevice<'a> {
    type BackingType = wgpu::Device;
    type ShaderType = WgpuShader<'a>;
    type PipelineLayoutType = WgpuPipelineLayout<'a>;
    type PipelineType = WgpuPipeline<'a>;
    type CommandBufferBuilderType = WgpuCommandBufferBuilder<'a>;
    type CommandBufferType = WgpuCommandBuffer<'a>;
    type ImageFormatType = WgpuTextureFormat;
    type TextureType = WgpuTexture<'a>;

    fn get_backing_device(&self) -> &Self::BackingType {
        &self.0
    }

    fn begin_command_buffer(&self) -> Self::CommandBufferBuilderType {
        WgpuCommandBufferBuilder(
            self.0
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                }),
            PhantomData,
        )
    }

    fn create_pipeline_layout(&self, shader: &Self::ShaderType) -> Self::PipelineLayoutType {
        WgpuPipelineLayout {
            layout: self
                .0
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                }),
            parent: PhantomData,
        }
    }

    fn create_pipeline(
        &self,
        info: &PipelineCreateInfo<
            '_,
            Self::PipelineLayoutType,
            Self::ShaderType,
            Self::ImageFormatType,
        >,
    ) -> Self::PipelineType {
        // TODO: Very ugly and I really don't like the SmallVec here.
        // Either I use a Vec or somehow convert these compile-time.

        let buffer_layouts: SmallVec<[_; 8]> = info
            .buffer_layout
            .iter()
            .map(|vertex_layout| {
                let mut offset: usize = 0;
                let attribs = vertex_layout
                    .attributes
                    .iter()
                    .enumerate()
                    .map(|(i, attr)| attr.pluto_to_wgpu(&mut offset, i))
                    .collect::<SmallVec<[_; 16]>>();

                (attribs, &vertex_layout.layout, &vertex_layout.stride)
            })
            .collect();

        let buffer_layout_slice: SmallVec<[_; 8]> = buffer_layouts
            .iter()
            .map(|layout| VertexBufferLayout {
                array_stride: *layout.2 as BufferAddress,
                step_mode: match layout.1 {
                    MeshLayout::Planar => todo!(),
                    MeshLayout::Interleaved => VertexStepMode::Vertex,
                },
                attributes: layout.0.as_slice(),
            })
            .collect();

        let pipeline = self
            .0
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(info.pipeline_layout.get_backing_pipeline_layout()),
                vertex: wgpu::VertexState {
                    module: info.shader.get_backing_module(),
                    entry_point: info.shader.vertex_entry_point(),
                    buffers: buffer_layout_slice.as_slice(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: info.shader.get_backing_module(),
                    entry_point: info.shader.fragment_entry_point(),
                    targets: &[wgpu::ColorTargetState {
                        format: info.texture_format.get_backing_format(),
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
            });

        Self::PipelineType {
            pipeline,
            parent: PhantomData,
        }
    }

    fn create_shader(&self, shader_code: &ShaderCode<'_>) -> Self::ShaderType {
        match *shader_code {
            ShaderCode::Wgsl {
                code,
                fragment_entry,
                vertex_entry,
            } => {
                let module = self.0.create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: None,
                    source: wgpu::ShaderSource::Wgsl(Cow::from(code)),
                });

                WgpuShader {
                    module,
                    vertex_entry: vertex_entry.to_string(),
                    fragment_entry: fragment_entry.to_string(),
                    parent: PhantomData,
                }
            }
        }
    }
}

pub struct WgpuCommandBufferBuilder<'a>(wgpu::CommandEncoder, PhantomData<&'a ()>);

impl<'a> CommandBufferBuilder<'_, WgpuCommandBuffer<'a>> for WgpuCommandBufferBuilder<'a> {
    type BackingType = wgpu::CommandEncoder;

    fn build(self) -> WgpuCommandBuffer<'a> {
        WgpuCommandBuffer(self.0.finish(), self.1)
    }

    fn get_backing_command_buffer_builder(&mut self) -> &mut Self::BackingType {
        &mut self.0
    }
}

pub struct WgpuCommandBuffer<'a>(wgpu::CommandBuffer, PhantomData<&'a ()>);

impl<'a> CommandBuffer<'_> for WgpuCommandBuffer<'a> {
    type BackingType = wgpu::CommandBuffer;

    fn get_backing_command_buffer(self) -> Self::BackingType {
        self.0
    }
}
