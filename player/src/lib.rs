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

pub mod logger;

use pluto_engine::runtime::platform::winit::wgpu::WinitWgpuDisplay;
use pluto_engine::runtime::pluto_runtime::PlutoRuntime;
use pluto_engine::runtime::{ApplicationBootstrapper, Runtime};
use std::fs;

use pluto_engine::pluto_engine_display::pluto_engine_render::device::{
    CommandBuffer, CommandBufferBuilder, Device, PhysicalDevice, Queue,
};
use pluto_engine::pluto_engine_display::pluto_engine_render::instance::ContextInstance;
use pluto_engine::pluto_engine_display::pluto_engine_render::mesh::{AttributeFormat, Vertex};
use pluto_engine::pluto_engine_display::pluto_engine_render::pipeline::{
    Pipeline, PipelineCreateInfo,
};
use pluto_engine::pluto_engine_display::pluto_engine_render::shader::ShaderCode;
use pluto_engine::pluto_engine_display::pluto_engine_render::surface::{Surface, SurfaceTexture};
use pluto_engine::pluto_engine_display::pluto_engine_render::texture::TextureView;
use pluto_engine::pluto_engine_display::{
    ApplicationDisplay, ApplicationState, PlutoDevice, PlutoPipeline, PlutoQueue,
    PlutoSurfaceTexture,
};
use pluto_engine_core_platform_wgpu::instance::WgpuInstance;
use pluto_engine_core_platform_wgpu::raw_window_handle::HasRawWindowHandle;
use pluto_engine_core_platform_wgpu::surface::WgpuSurface;
use pluto_engine_core_platform_wgpu::wgpu;
use pluto_engine_core_platform_winit::event_loop::WinitEventLoop;
use pluto_engine_core_platform_winit::pluto_engine_window::window::Window;
use wgpu::util::DeviceExt;

use crate::AttributeFormat::Float32x3;

use pluto_engine::application::layer::pluto::PlutoLayerManager;
use pluto_engine::application::Application;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn main() {
    logger::init_logger();

    PlutoRuntime::run(ApplicationBootstrapper::<WinitEventLoop>::new(Box::new(
        |window| {
            let instance = WgpuInstance::new(&window);
            let (physical_device, mut surface) = instance.create_device_and_surface();
            let (device, queue) = physical_device.create_device_and_queue();
            surface.configure(&device);
            let display = WinitWgpuDisplay::new(&mut surface, &window, &device);
            let mut state = State::new(display, &device, &queue);
            let mut layer_manager = PlutoLayerManager::new();
            pluto_engine_test::ApplicationTest::run(&mut layer_manager);
            ApplicationBootstrapper::<WinitEventLoop>::default_loop(&mut state);
        },
    )));
}

struct State<'a, AD: ApplicationDisplay<'a>> {
    display: AD,
    device: &'a PlutoDevice<'a, AD>,
    queue: &'a PlutoQueue<'a, AD>,
    render_pipeline: PlutoPipeline<'a, AD>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TestVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex for TestVertex {
    const ATTRIBS: &'static [AttributeFormat] = &[Float32x3, Float32x3];
}

const VERTICES: &[TestVertex] = &[
    TestVertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    TestVertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    TestVertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

impl<
        'a,
        W: Window<SizeType = <WgpuSurface<'a> as Surface<'a>>::SizeType> + HasRawWindowHandle,
        AD: ApplicationDisplay<'a, WindowType = W, ContextType = WgpuInstance<'a, W>>,
    > ApplicationState<'a, AD> for State<'a, AD>
{
    fn new(display: AD, device: &'a PlutoDevice<'a, AD>, queue: &'a PlutoQueue<'a, AD>) -> Self {
        let shader_code = fs::read_to_string("assets/plutoengine.base/shader.wgsl").unwrap();

        let shader = device.create_shader(&ShaderCode::Wgsl {
            code: &shader_code,
            vertex_entry: "vs_main",
            fragment_entry: "fs_main",
        });

        let pipeline_layout = device.create_pipeline_layout(&shader);

        let render_pipeline = device.create_pipeline(&PipelineCreateInfo {
            shader: &shader,
            pipeline_layout: &pipeline_layout,
            buffer_layout: &[TestVertex::layout()],
            texture_format: display.get_surface().get_texture_format(),
        });

        Self {
            display,
            device,
            queue,
            render_pipeline,
        }
    }

    fn render(&mut self, surface_texture: &PlutoSurfaceTexture<'a, AD>) {
        let view = surface_texture.get_texture_view();

        let mut command_buf = self.device.begin_command_buffer();

        let encoder = command_buf.get_backing_command_buffer_builder();

        let b_device = self.device.get_backing_device();

        let vertex_buffer = b_device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let num_vertices = VERTICES.len() as u32;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: view.get_backing_texture_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.6,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(self.render_pipeline.get_backing_pipeline());
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..num_vertices, 0..1);
        }

        self.queue.get_backing_queue().submit(std::iter::once(
            command_buf.build().get_backing_command_buffer(),
        ));
    }

    fn display(&mut self) -> &mut AD {
        &mut self.display
    }
}
