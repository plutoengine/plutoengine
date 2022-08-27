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

pub enum MeshLayout {
    Planar,
    Interleaved,
}

pub enum AttributeFormat {
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
}

impl AttributeFormat {
    pub const fn size(&self) -> usize {
        match self {
            AttributeFormat::Float32 => std::mem::size_of::<f32>(),
            AttributeFormat::Float32x2 => std::mem::size_of::<f32>() * 2,
            AttributeFormat::Float32x3 => std::mem::size_of::<f32>() * 3,
            AttributeFormat::Float32x4 => std::mem::size_of::<f32>() * 4,
        }
    }
}

pub trait Vertex: Sized {
    const ATTRIBS: &'static [AttributeFormat];

    fn layout<'a>() -> VertexLayout<'a> {
        VertexLayout {
            stride: std::mem::size_of::<Self>(),
            layout: MeshLayout::Interleaved,
            attributes: Self::ATTRIBS,
        }
    }
}

pub struct VertexLayout<'a> {
    pub stride: usize,
    pub layout: MeshLayout,
    pub attributes: &'a [AttributeFormat],
}

pub trait VertexBuffer {}

pub trait Mesh {}
