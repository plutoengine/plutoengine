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

use crate::mesh::Mesh;
use crate::pipeline::{Pipeline, PipelineCreateInfo, PipelineLayout};
use crate::shader::{Shader, ShaderCode};
use crate::texture::{Texture, TextureFormat};

pub trait Queue<'a> {
    type BackingType;

    fn get_backing_queue(&self) -> &Self::BackingType;
}

pub trait PhysicalDevice<'a> {
    type BackingType;

    type DeviceType: Device<'a>;
    type QueueType: Queue<'a>;

    fn new(adapter: Self::BackingType) -> Self;

    fn get_backing_physical_device(&self) -> &Self::BackingType;

    fn create_device_and_queue(&self) -> (Self::DeviceType, Self::QueueType);
}

pub trait Device<'a> {
    type BackingType;

    type ShaderType: Shader<'a>;
    type PipelineLayoutType: PipelineLayout<'a>;
    type PipelineType: Pipeline<'a, LayoutType = Self::PipelineLayoutType>;
    type CommandBufferBuilderType: CommandBufferBuilder<'a, Self::CommandBufferType>;
    type CommandBufferType: CommandBuffer<'a>;
    type ImageFormatType: TextureFormat;
    type TextureType: Texture<'a>;

    fn get_backing_device(&self) -> &Self::BackingType;

    fn begin_command_buffer(&self) -> Self::CommandBufferBuilderType;

    fn create_pipeline_layout(&self, shader: &Self::ShaderType) -> Self::PipelineLayoutType;

    fn create_pipeline(
        &self,
        info: &PipelineCreateInfo<
            'a,
            Self::PipelineLayoutType,
            Self::ShaderType,
            Self::ImageFormatType,
        >,
    ) -> Self::PipelineType;

    fn create_shader(&self, code: &ShaderCode<'_>) -> Self::ShaderType;
}

pub trait CommandBufferBuilder<'a, C: CommandBuffer<'a>> {
    type BackingType;

    fn build(self) -> C;

    fn get_backing_command_buffer_builder(&mut self) -> &mut Self::BackingType;
}

pub trait CommandBuffer<'a> {
    type BackingType;

    fn get_backing_command_buffer(self) -> Self::BackingType;
}

pub trait DeviceMeshFactory<'a, M: Mesh>: Device<'a> {
    fn create_mesh(&self) -> M;
}
