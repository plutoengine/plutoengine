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

use pluto_engine_render::mesh::AttributeFormat;
use wgpu::{BufferAddress, VertexAttribute, VertexFormat};

pub(crate) trait WgpuAttribute: Sized {
    fn pluto_to_wgpu(&self, offset: &mut usize, position: usize) -> VertexAttribute;
}

impl WgpuAttribute for AttributeFormat {
    fn pluto_to_wgpu(&self, offset: &mut usize, position: usize) -> VertexAttribute {
        let attrib = VertexAttribute {
            offset: *offset as BufferAddress,
            format: match self {
                AttributeFormat::Float32 => VertexFormat::Float32,
                AttributeFormat::Float32x2 => VertexFormat::Float32x2,
                AttributeFormat::Float32x3 => VertexFormat::Float32x3,
                AttributeFormat::Float32x4 => VertexFormat::Float32x4,
            },
            shader_location: position as u32,
        };

        *offset += self.size();

        attrib
    }
}
