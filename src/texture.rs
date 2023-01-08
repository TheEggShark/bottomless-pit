use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::Vertex;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl Texture {
    pub fn from_bytes(device: &wgpu::Device, queue: &wgpu::Queue, label: Option<&str>, bytes: &[u8]) -> Result<Self, image::ImageError>{
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, img, label))
    }

    pub fn from_image(device: &wgpu::Device, queue: &wgpu::Queue, img: image::DynamicImage, label: Option<&str>) -> Self {
        let diffuse_rgba = img.to_rgba8();
        let (width, height) = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor{
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
            // COPY_DST means that we want to copy data to this texture 
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label,
        });

        queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba, 
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            texture_size
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            // what to do when given cordinates outside the textures height/width
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            // what do when give less or more than 1 pixel to sample
            // linear interprelates between all of them nearest gives the closet colour
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float {filterable: true},
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ],
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                }
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            texture, 
            view, 
            sampler, 
            bind_group, 
            bind_group_layout
        }
    }
}

const RECT_INDICIES: &[u16] = &[
    0, 1, 2,
    3, 0, 2,
];

pub struct TexturedRect {
    texture: Texture,
    points: [Vertex; 4],
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl TexturedRect {
    pub fn new(texture: Texture, pos: [f32; 2], size: [f32; 2], device: &wgpu::Device) -> Self {
        let points = [
            Vertex::from_2d([0.0, 0.0], [0.0, 0.0]), 
            Vertex::from_2d([0.3, 0.0], [1.0, 0.0]),
            Vertex::from_2d([0.3, -0.3], [1.0, 1.0]),
            Vertex::from_2d([0.0, -0.3], [0.0, 1.0]),
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&points),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICIES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            texture,
            points,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.texture.bind_group
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.texture.bind_group_layout
    }
}

pub trait DrawTexturedRect<'a> {
    fn draw_textured_rect(&mut self, rect: &'a TexturedRect, camera_bind_group: &'a wgpu::BindGroup);
}

impl<'a, 'b> DrawTexturedRect<'b> for wgpu::RenderPass<'a> where 'b: 'a, {
    fn draw_textured_rect(&mut self, rect: &'b TexturedRect, camera_bind_group: &'b wgpu::BindGroup) {
        self.set_vertex_buffer(0, rect.vertex_buffer.slice(..));
        self.set_index_buffer(rect.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.set_bind_group(0, &rect.texture.bind_group, &[]);
        self.set_bind_group(1, camera_bind_group, &[]);
        self.draw_indexed(0..6, 0, 0..1);
    }
}