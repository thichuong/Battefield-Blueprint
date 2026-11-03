use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct QuadVertex {
    pub quad_pos: [f32; 2],
    pub quad_uv: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct SpriteInstance {
    pub pos: [f32; 2],
    pub size: [f32; 2],
    pub uv_min: [f32; 2],
    pub uv_max: [f32; 2],
    pub color: [f32; 4],
    pub rotation: f32,
    pub _pad: f32,
}

impl SpriteInstance {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SpriteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct SpriteBatch {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub instances: Vec<SpriteInstance>,
    pub max_instances: usize,
}

impl SpriteBatch {
    pub fn new(device: &wgpu::Device) -> Self {
        // Quad 1x1 xoay quanh tâm (-0.5 .. 0.5)
        let vertices = [
            QuadVertex {
                quad_pos: [-0.5, -0.5],
                quad_uv: [0.0, 0.0],
            },
            QuadVertex {
                quad_pos: [0.5, -0.5],
                quad_uv: [1.0, 0.0],
            },
            QuadVertex {
                quad_pos: [0.5, 0.5],
                quad_uv: [1.0, 1.0],
            },
            QuadVertex {
                quad_pos: [-0.5, 0.5],
                quad_uv: [0.0, 1.0],
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SpriteBatch Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("SpriteBatch Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let initial_capacity = 8192;
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SpriteBatch Instance Buffer"),
            size: (initial_capacity * std::mem::size_of::<SpriteInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
            instances: Vec::with_capacity(initial_capacity),
            max_instances: initial_capacity,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.instances.clear();
    }

    #[inline]
    pub fn push(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        uv_min: [f32; 2],
        uv_max: [f32; 2],
        color: [f32; 4],
        rotation: f32,
    ) {
        self.instances.push(SpriteInstance {
            pos,
            size,
            uv_min,
            uv_max,
            color,
            rotation,
            _pad: 0.0,
        });
    }

    /// Vẽ một hình chữ nhật đặc (dùng pixel trắng 1x1 trên atlas)
    #[inline]
    pub fn push_rect(
        &mut self,
        pos: [f32; 2],
        size: [f32; 2],
        color: [f32; 4],
        white_uv: [f32; 2],
    ) {
        self.push(pos, size, white_uv, white_uv, color, 0.0);
    }

    /// Vẽ một đoạn thẳng dày (dùng quad xoay quanh tâm)
    pub fn push_line(
        &mut self,
        p1: [f32; 2],
        p2: [f32; 2],
        thickness: f32,
        color: [f32; 4],
        white_uv: [f32; 2],
    ) {
        let dx = p2[0] - p1[0];
        let dy = p2[1] - p1[1];
        let length = (dx * dx + dy * dy).sqrt();
        if length <= 0.0001 {
            return;
        }
        let mid_x = (p1[0] + p2[0]) * 0.5;
        let mid_y = (p1[1] + p2[1]) * 0.5;
        let angle = dy.atan2(dx);

        self.push(
            [mid_x, mid_y],
            [length, thickness],
            white_uv,
            white_uv,
            color,
            angle,
        );
    }

    /// Vẽ khung chữ nhật rỗng (stroke rect)
    pub fn push_stroke_rect(
        &mut self,
        center: [f32; 2],
        size: [f32; 2],
        thickness: f32,
        color: [f32; 4],
        white_uv: [f32; 2],
    ) {
        let hw = size[0] * 0.5;
        let hh = size[1] * 0.5;
        let cx = center[0];
        let cy = center[1];

        // Top line
        self.push_rect([cx, cy - hh], [size[0], thickness], color, white_uv);
        // Bottom line
        self.push_rect([cx, cy + hh], [size[0], thickness], color, white_uv);
        // Left line
        self.push_rect([cx - hw, cy], [thickness, size[1]], color, white_uv);
        // Right line
        self.push_rect([cx + hw, cy], [thickness, size[1]], color, white_uv);
    }

    /// Vẽ vòng tròn dạng đoạn thẳng xấp xỉ (Selection ring / Attack move circle)
    pub fn push_circle_stroke(
        &mut self,
        center: [f32; 2],
        radius_x: f32,
        radius_y: f32,
        thickness: f32,
        color: [f32; 4],
        segments: usize,
        white_uv: [f32; 2],
    ) {
        let segs = segments.max(12);
        let step = std::f32::consts::TAU / (segs as f32);
        for i in 0..segs {
            let a1 = (i as f32) * step;
            let a2 = ((i + 1) as f32) * step;
            let p1 = [center[0] + a1.cos() * radius_x, center[1] + a1.sin() * radius_y];
            let p2 = [center[0] + a2.cos() * radius_x, center[1] + a2.sin() * radius_y];
            self.push_line(p1, p2, thickness, color, white_uv);
        }
    }

    /// Chuẩn bị và upload dữ liệu instances lên GPU
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let count = self.instances.len();
        if count == 0 {
            return;
        }

        if count > self.max_instances {
            self.max_instances = (count * 2).max(self.max_instances * 2);
            self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("SpriteBatch Expanded Instance Buffer"),
                size: (self.max_instances * std::mem::size_of::<SpriteInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        let slice = bytemuck::cast_slice(&self.instances);
        queue.write_buffer(&self.instance_buffer, 0, slice);
    }

    /// Thực hiện Draw Call trên RenderPass
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let count = self.instances.len() as u32;
        if count == 0 {
            return;
        }
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..count);
    }
}

