use std::{
    ops::{AddAssign, Deref, Mul},
    sync::Arc,
};

use glam::{Mat4, Vec2, Vec3};
use tokio::runtime::{Builder, Runtime};
use vulkano::{
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        Buffer, BufferContents, BufferCreateInfo, BufferUsage,
    },
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferExecFuture, PrimaryAutoCommandBuffer,
        PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        SubpassEndInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    format::ClearValue,
    image::Image,
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::ViewportState,
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, RenderPass, RenderPassCreateInfo},
    swapchain::SwapchainAcquireFuture,
    sync::GpuFuture,
};
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{self, ActiveEventLoop, EventLoop},
    keyboard::KeyCode,
};

use crate::{
    vulkano_render_loading::{load_framebuffers, load_render_pass, load_render_pass2},
    vulkano_window_setup::{RenderPassContext, VulkanWindow, VulkanWindowSettings, WindowData},
};

mod pointcloud_renderer;
mod shaders;
mod vulkano_device_setup;
mod vulkano_render_loading;
pub mod vulkano_window_setup;

pub type TestVertexHolder = Arc<std::sync::Mutex<Vec<TestVertex>>>;

pub async fn load_window<const NUM_PASSES: usize>(
    settings: VulkanWindowSettings,
    renderer: Arc<std::sync::Mutex<dyn VulkanRenderer>>,
) {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let runtime = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .build()
        .expect("Failed to create tokio runtime for rendering");
    let mut window: VulkanWindow<2> = VulkanWindow::new(settings, Arc::new(runtime), renderer);
    let _ = event_loop.run_app(&mut window);
}

#[derive(BufferContents, Vertex, Clone, Copy)]
#[repr(C)]
pub struct TestVertex {
    #[format(R32G32B32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32B32A32_SFLOAT)]
    color: [f32; 4],
    // #[format(R64_SFLOAT)]
    // time: f64,
}

/*

impl Ord for TestVertex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.time < other.time {
            return std::cmp::Ordering::Less;
        }
        std::cmp::Ordering::Greater
        // Ord::cmp(&self.time, &other.time)
    }
}

impl PartialOrd for TestVertex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TestVertex {}

impl PartialEq for TestVertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.time == other.time
    }
}

// impl Eq for TestVertex {}

*/
impl TestVertex {
    pub fn new(x: f32, y: f32, z: f32, time: f64) -> TestVertex {
        TestVertex {
            position: [x, y, z],
            color: [1.0 * time as f32, 0.8, 0.0, 1.0],
        }
    }

    pub fn from_tuple(positions: (f64, f64, f64, f64)) -> TestVertex {
        TestVertex {
            position: [positions.0 as f32, positions.1 as f32, positions.2 as f32],
            color: [1.0 * positions.3 as f32, 0.8, 0.0, 1.0],
        }
    }
}

pub trait VulkanRenderer: Send + Sync {
    fn build_renderer(
        &mut self,
        context: SwapchainAcquireFuture,
        windowdata: &WindowData,
        image_idx: u32,
    ) -> Option<CommandBufferExecFuture<SwapchainAcquireFuture>>;
    fn assemble_self(&mut self, windowdata: &WindowData);
    fn handle_window_event(&mut self, _event: WindowEvent, _event_loop: &ActiveEventLoop) {}
    fn handle_device_event(&mut self, _event: DeviceEvent, _event_loop: &ActiveEventLoop) {}
    fn return_window_data(&self) -> WindowData;
    // fn return_render_pass(&self) -> [Arc<RenderPass>; NUM_PASSES];
    // fn return_pipeline(&self) -> [Arc<GraphicsPipeline>; NUM_PASSES];
    // fn load_framebuffers(
    //     &self,
    //     img: Arc<Image>,
    //     allocator: Arc<StandardMemoryAllocator>,
    // ) -> [Arc<Framebuffer>; NUM_PASSES];
}

#[derive(Default, Clone)]
pub struct TestRenderer<const NUM_PASSES: usize> {
    pub pipeline: Option<[Arc<GraphicsPipeline>; NUM_PASSES]>,
    pub layout: Option<[Arc<PipelineLayout>; NUM_PASSES]>,
    pub render_pass: Option<Arc<RenderPass>>,
    pub window_data: Option<WindowData>,
    pub pointcloud: TestVertexHolder,
    pub camera_pos: Vec3,
    pub look_dir: Vec2,
    pub proj: Mat4,
    pub scale: Mat4,
    pub mouse_enabled: bool,
    pub mouse_down: bool,
}

pub const CURRENT_PASSES: usize = 2;

impl TestRenderer<CURRENT_PASSES> {
    const DOWN_VEC: Vec3 = Vec3::new(0.0, -1.0, 0.0);
    fn handle_mouse_press(&mut self, button: MouseButton, state: ElementState) {
        if let MouseButton::Left = button {
            self.mouse_down = state.is_pressed();
        }
    }
    const MOVE_SPEED: f32 = 0.1;
    fn handle_keypress(&mut self, event: KeyEvent) {
        // let look = self.mouse_move_vec();
        match event.physical_key {
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyW) => self
                .camera_pos
                .add_assign(self.mouse_move_vec().mul(Vec3::splat(Self::MOVE_SPEED))),
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyS) => self
                .camera_pos
                .add_assign(self.mouse_move_vec().mul(Vec3::splat(-Self::MOVE_SPEED))),
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyA) => self.camera_pos.add_assign(
                self.mouse_move_vec()
                    .cross(Self::DOWN_VEC)
                    .mul(Vec3::splat(-Self::MOVE_SPEED)),
            ),
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyD) => self.camera_pos.add_assign(
                self.mouse_move_vec()
                    .cross(Self::DOWN_VEC)
                    .mul(Vec3::splat(Self::MOVE_SPEED)),
            ),
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyQ) => self
                .camera_pos
                .add_assign(Vec3::new(0.0, Self::MOVE_SPEED, 0.0)),
            winit::keyboard::PhysicalKey::Code(KeyCode::KeyE) => self
                .camera_pos
                .add_assign(Vec3::new(0.0, -Self::MOVE_SPEED, 0.0)),
            _ => {} // winit::keyboard::PhysicalKey::Unidentified(_) => todo!(),
        }
    }

    const SENSITIVITY: f32 = 0.005;
    fn mouse_move_vec(&self) -> Vec3 {
        let angles = self.look_dir;
        let s_theta = angles.x.sin();
        Vec3::new(
            angles.x.cos(),
            // 0.0,
            s_theta * angles.y.cos(),
            s_theta * angles.y.sin(),
            // 0.0, // angles.x.cos(),
        )
        .normalize()
    }

    fn handle_mouse_move(&mut self, delta: (f64, f64)) {
        if self.mouse_enabled && self.mouse_down {
            self.look_dir.add_assign(Vec2::new(
                delta.0 as f32 * Self::SENSITIVITY,
                delta.1 as f32 * -Self::SENSITIVITY,
            ));
            // dbg!(self.look_dir);
        }
    }
}

impl VulkanRenderer for TestRenderer<CURRENT_PASSES> {
    fn build_renderer(
        &mut self,
        context: SwapchainAcquireFuture,
        win: &WindowData,
        image_idx: u32,
    ) -> Option<CommandBufferExecFuture<SwapchainAcquireFuture>> {
        if self.pipeline.is_none()
            | self.layout.is_none()
            | self.render_pass.is_none()
            | self.window_data.is_none()
        {
            return None;
        }
        let mut data: Vec<TestVertex> = Vec::new();
        {
            data.clone_from(&self.pointcloud.lock().unwrap().clone());
        }
        let data_len: u32 = data.len() as u32;
        let vertex_buffer = Buffer::from_iter(
            win.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data,
        )
        .expect("Failed to create vertex buffer");
        //DUPE +++
        let mut data2: Vec<TestVertex> = Vec::new();
        data2.push(TestVertex {
            position: [0.0, 1.0, -39.0],
            color: [1.0, 0.0, 0.0, 1.0],
        });
        data2.push(TestVertex {
            position: [39.0, 1.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        });
        data2.push(TestVertex {
            position: [3.0, 39.0, 1.0],
            color: [1.0, 0.0, 0.0, 1.0],
        });
        // data2.push(TestVertex {
        //     position: [0.0, 1.0, -39.0],
        //     color: [1.0, 0.0, 0.0, 1.0],
        // });
        let data2_len: u32 = data2.len() as u32;
        let vertex_buffer2 = Buffer::from_iter(
            win.allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            data2,
        )
        .expect("Failed to create vertex buffer");
        // +++

        let uni_buffer = SubbufferAllocator::new(
            win.allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );
        let uni_sub_buffer = uni_buffer
            .allocate_sized()
            .expect("Failed to allocate sub buffer");

        let desc_layouts = self.layout.clone().unwrap()[0].set_layouts()[0].clone();
        //Uniform buffers
        let proj = self.proj.to_cols_array_2d();
        let scale = self.scale;
        let world = Mat4::IDENTITY.to_cols_array_2d();

        let view = (Mat4::look_to_rh(self.camera_pos, self.mouse_move_vec(), Self::DOWN_VEC)
            * scale)
            .to_cols_array_2d();
        let mats_data = crate::shaders::vs::WorldMats { world, view, proj };
        *uni_sub_buffer.write().expect("Failed to write") = mats_data;
        //
        let subset = PersistentDescriptorSet::new(
            &StandardDescriptorSetAllocator::new(win.device.clone(), Default::default()),
            desc_layouts.clone(),
            [WriteDescriptorSet::buffer(0, uni_sub_buffer)],
            [],
        )
        .expect("Failed to create descriptor set for 3d mats");
        let mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
            AutoCommandBufferBuilder::primary(
                win.command_buffer_allocator.deref(),
                win.queue_id,
                vulkano::command_buffer::CommandBufferUsage::MultipleSubmit,
            )
            .expect("Failed to build builder!!1");
        let framebuffer = load_framebuffers(
            win.images[image_idx as usize].clone(),
            self.render_pass.clone().unwrap(),
            win.allocator.clone(),
        );
        // context
        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![
                        Some([0.1, 0.1, 0.1, 1.0].into()),
                        Some(ClearValue::Depth(1.0)),
                    ],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: SubpassContents::Inline,
                    ..Default::default()
                },
            )
            .expect("Failed to begin render pass")
            .bind_pipeline_graphics(self.pipeline.clone().unwrap()[0].clone())
            .expect("Failed to apply pipeline")
            .bind_vertex_buffers(0, vertex_buffer.clone())
            .expect("Failed to bind vertex buffer")
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.layout.clone().unwrap()[0].clone(),
                0,
                subset.clone(),
            )
            .expect("Failed to bind perspective set")
            .draw(data_len, 1, 0, 0)
            .unwrap()
            // +
            .bind_pipeline_graphics(self.pipeline.clone().unwrap()[1].clone())
            .expect("Failed to apply pipeline")
            .bind_vertex_buffers(0, vertex_buffer2.clone())
            .expect("Failed to bind vertex buffer")
            // .bind_descriptor_sets(
            //     PipelineBindPoint::Graphics,
            //     self.layout.clone().unwrap()[0].clone(),
            //     0,
            //     subset.clone(),
            // )
            // .expect("Failed to bind perspective set")
            .draw(data2_len, 1, 0, 0)
            // +
            .expect("Failed to draw")
            .end_render_pass(SubpassEndInfo::default())
            .expect("Failed to end render pass");
        // builder
        //     .begin_render_pass(
        //         RenderPassBeginInfo {
        //             clear_values: vec![
        //                 Some([0.1, 0.1, 0.1, 1.0].into()),
        //                 Some(ClearValue::Depth(1.0)),
        //             ],
        //             ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
        //         },
        //         SubpassBeginInfo {
        //             contents: SubpassContents::Inline,
        //             ..Default::default()
        //         },
        //     )
        //     .expect("Failed to begin render pass")
        //     .bind_pipeline_graphics(self.pipeline.clone().unwrap()[1].clone())
        //     .expect("Failed to apply pipeline")
        //     .bind_vertex_buffers(0, vertex_buffer2.clone())
        //     .expect("Failed to bind vertex buffer")
        //     .bind_descriptor_sets(
        //         PipelineBindPoint::Graphics,
        //         self.layout.clone().unwrap()[0].clone(),
        //         0,
        //         subset.clone(),
        //     )
        //     .expect("Failed to bind perspective set")
        //     .draw(data2_len, 1, 0, 0)
        //     .expect("Failed to draw")
        //     .end_render_pass(SubpassEndInfo::default())
        //     .expect("Failed to end render pass");
        let comp1 = builder.build().expect("failed to build 1");
        // comp1.execute_after(context, win.queue.clone());
        Some(
            context
                .then_execute(win.queue.clone(), comp1)
                .expect("Failed to execute"),
        )

        // context
        //     .builder
        //     .begin_render_pass(
        //         RenderPassBeginInfo {
        //             clear_values: vec![
        //                 None,
        //                 // Some([0.1, 0.1, 0.4, 0.0].into()),
        //                 // Some(ClearValue::Depth(1.0)),
        //             ],
        //             ..RenderPassBeginInfo::framebuffer(context.framebuffer[1].clone())
        //         },
        //         SubpassBeginInfo {
        //             contents: SubpassContents::Inline,
        //             ..Default::default()
        //         },
        //     )
        //     .expect("Failed to begin render pass")
        //     .bind_pipeline_graphics(self.pipeline.clone().unwrap()[1].clone())
        //     .expect("Failed to apply pipeline")
        //     .bind_vertex_buffers(0, vertex_buffer2.clone())
        //     .expect("Failed to bind vertex buffer")
        //     .bind_descriptor_sets(
        //         PipelineBindPoint::Graphics,
        //         self.layout.clone().unwrap()[1].clone(),
        //         0,
        //         subset.clone(),
        //     )
        //     .expect("Failed to bind perspective set")
        //     .draw(data2_len, 1, 0, 0)
        //     .expect("Failed to draw")
        //     .end_render_pass(SubpassEndInfo::default())
        //     .expect("Failed to end render pass");
    }

    fn handle_window_event(&mut self, event: WindowEvent, event_loop: &ActiveEventLoop) {
        match event {
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                self.handle_keypress(event);
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
            } => self.handle_mouse_press(button, state),
            WindowEvent::CursorEntered { device_id: _ } => self.mouse_enabled = true,
            WindowEvent::CursorLeft { device_id: _ } => {
                self.mouse_enabled = false;
                self.mouse_down = false
            }
            _ => {}
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent, event_loop: &ActiveEventLoop) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            // dbg!(&delta);
            self.handle_mouse_move(delta);
        }
    }

    fn assemble_self(&mut self, windowdata: &WindowData) {
        // RenderPass::new(self.window_data.unwrap().device, RenderPassCreateInfo {});
        //Render Pass
        let render_pass = load_render_pass(windowdata.device.clone(), windowdata.swapchain.clone());
        //FIXME:
        self.render_pass = Some(render_pass.clone());

        //PipelineLayout
        let vs = crate::shaders::vs::load(windowdata.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = crate::shaders::fs::load(windowdata.device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let vertex_input_state = TestVertex::per_vertex()
            .definition(&vs.info().input_interface)
            .expect("Failed to create vertex state");
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout = PipelineLayout::new(
            windowdata.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(windowdata.device.clone())
                .expect("Failed to create pipeline layout info"),
        )
        .expect("Failed to create pipeline layout");
        //FIXME:
        self.layout = Some([layout.clone(), layout.clone()]);

        //Pipeline
        let input_assembly_state = Some(InputAssemblyState {
            topology: PrimitiveTopology::PointList,
            ..Default::default()
        });
        let subpass = render_pass.clone().first_subpass();
        let pipeline_info = GraphicsPipelineCreateInfo {
            stages: stages.clone().into_iter().collect(),
            vertex_input_state: Some(vertex_input_state.clone()),
            input_assembly_state,
            viewport_state: Some(ViewportState {
                viewports: [windowdata.viewport.clone()].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout.clone())
        };

        let pipeline = GraphicsPipeline::new(windowdata.device.clone(), None, pipeline_info)
            .expect("Failed to create pipeline");
        //FIXME: THIS
        // self.pipeline = Some(pipeline);
        self.window_data = Some(windowdata.clone());

        //Pipeline 2
        // let render_pass =
        //     load_render_pass2(windowdata.device.clone(), windowdata.swapchain.clone());
        let input_assembly_state = Some(InputAssemblyState {
            topology: PrimitiveTopology::TriangleStrip,
            ..Default::default()
        });
        let subpass = render_pass.first_subpass();
        let pipeline_info = GraphicsPipelineCreateInfo {
            stages: stages.clone().into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state,
            viewport_state: Some(ViewportState {
                viewports: [windowdata.viewport.clone()].into_iter().collect(),
                ..Default::default()
            }),
            rasterization_state: Some(RasterizationState::default()),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState::simple()),
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState::default(),
            )),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        };

        let pipeline2 = GraphicsPipeline::new(windowdata.device.clone(), None, pipeline_info)
            .expect("Failed to create pipeline");
        self.pipeline = Some([pipeline, pipeline2]);
        // self.pipeline = Some(pipeline);
    }

    fn return_window_data(&self) -> WindowData {
        self.window_data.clone().unwrap()
    }
}
