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

use pluto_engine_render::texture::{Texture, TextureFormat, TextureView};
use std::marker::PhantomData;
use wgpu::TextureViewDescriptor;

pub struct WgpuTextureFormat(pub(crate) wgpu::TextureFormat);

impl TextureFormat for WgpuTextureFormat {
    type BackingType = wgpu::TextureFormat;

    fn get_backing_format(&self) -> Self::BackingType {
        self.0
    }
}

pub struct WgpuTexture<'a> {
    pub(crate) texture: wgpu::Texture,
    pub(crate) parent: PhantomData<&'a ()>,
}

impl<'a> Texture<'_> for WgpuTexture<'a> {
    type BackingType = wgpu::Texture;
    type ViewType = WgpuTextureView<'a>;

    fn get_backing_texture(&self) -> &Self::BackingType {
        &self.texture
    }

    fn create_view(&self) -> Self::ViewType {
        WgpuTextureView {
            view: self.texture.create_view(&TextureViewDescriptor::default()),
            parent: PhantomData,
        }
    }
}

pub struct WgpuTextureView<'a> {
    pub(crate) view: wgpu::TextureView,
    pub(crate) parent: PhantomData<&'a ()>,
}

impl<'a> TextureView<'_> for WgpuTextureView<'a> {
    type BackingType = wgpu::TextureView;

    fn get_backing_texture_view(&self) -> &Self::BackingType {
        &self.view
    }
}
