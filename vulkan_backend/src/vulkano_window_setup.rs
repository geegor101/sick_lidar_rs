use std::{ops::Deref, sync::Arc, time::Duration};

use tokio::{runtime::Runtime, sync::Mutex, time::Interval};
use vulkano::{
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder,
        CommandBufferInheritanceInfo, CommandBufferUsage, PrimaryAutoCommandBuffer,
    },
    device::{physical::PhysicalDevice, Device, Queue},
    image::Image,
    instance::Instance,
    memory::allocator::StandardMemoryAllocator,
    pipeline::{graphics::viewport::Viewport, GraphicsPipeline},
    render_pass::{Framebuffer, RenderPass},
    swapchain::{self, Surface, Swapchain, SwapchainPresentInfo},
    sync::GpuFuture,
    Validated,
};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::Window,
};

use crate::{
    vulkano_device_setup::{
        load_vulkan_instance, make_device, make_phys_device, make_surface, make_swapchain,
        make_window,
    },
    vulkano_render_loading::{load_framebuffers, load_framebuffers2},
    VulkanRenderer,
};

pub struct VulkanWindow<const NUM_PASSES: usize> {
    window_data: Option<WindowData>,
    frame_time: Arc<Mutex<Interval>>,
    settings: VulkanWindowSettings,
    runtime: Arc<Runtime>,
    renderer: Arc<std::sync::Mutex<dyn VulkanRenderer + Send>>,
}

#[derive(Clone)]
pub struct VulkanWindowSettings {
    pub window_size: [u32; 2],
    pub fps: u64,
    // pub proj: Mat4,
    // pub scale: Mat4,
    // pub vertex_data: TestVertexHolder,
}

impl Default for VulkanWindowSettings {
    fn default() -> Self {
        Self {
            window_size: [1024, 1024],
            fps: 30,
            // proj: Mat4::perspective_rh_gl(90.0_f32.to_radians(), 1.0, 0.01, 40.0), //Mat4::orthographic_rh_gl(left, right, bottom, top, near, far)
            // scale: Mat4::from_scale(Vec3::splat(0.001)),
            // vertex_data: Arc::new(std::sync::Mutex::new(vec![TestVertex::new(
            //     0.0, 0.0, 0.0, 0.0,
            // )])),
        }
    }
}

#[derive(Clone)]
pub struct WindowData {
    pub instance: Arc<Instance>,
    pub window: Arc<Window>,
    pub surface: Arc<Surface>,
    pub phys_device: Arc<PhysicalDevice>,
    pub device: Arc<Device>,
    pub swapchain: Arc<Swapchain>,
    pub images: Vec<Arc<Image>>,
    pub allocator: Arc<
        vulkano::memory::allocator::GenericMemoryAllocator<
            vulkano::memory::allocator::FreeListAllocator,
        >,
    >,
    pub command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    pub queue_id: u32,
    // fence: Vec<Option<Arc<FenceSignalFuture<_>>>>,
    pub queue: Arc<Queue>,
    pub viewport: Viewport,
}

impl<const NUM_PASSES: usize> VulkanWindow<NUM_PASSES> {
    pub fn new(
        settings: VulkanWindowSettings,
        runtime: Arc<Runtime>,
        renderer: Arc<std::sync::Mutex<dyn VulkanRenderer + Send>>,
    ) -> VulkanWindow<NUM_PASSES> {
        let frame_time = tokio::time::interval(Duration::from_micros(1000000 / settings.fps));
        let frame_time = Arc::new(Mutex::new(frame_time));

        VulkanWindow {
            window_data: None,
            frame_time,
            settings,
            runtime,
            renderer, //: Arc::new(std::sync::Mutex::new(renderer)),
        }
    }

    fn load_swapchain(&mut self, event_loop: &ActiveEventLoop) {
        let instance = load_vulkan_instance(event_loop);
        let window = make_window(event_loop, self.settings.window_size);
        let surface = make_surface(instance.clone(), window.clone());
        let (phys_device, queue_id) = make_phys_device(instance.clone(), surface.clone());
        let (device, queue) = make_device(phys_device.clone());
        let (swapchain, images) = make_swapchain(
            phys_device.clone(),
            surface.clone(),
            window.clone(),
            device.clone(),
        );
        let allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [
                self.settings.window_size[0] as f32,
                self.settings.window_size[1] as f32,
            ],
            depth_range: 0.0..=1.0,
        };

        self.window_data = Some(WindowData {
            instance,
            window,
            surface,
            phys_device,
            queue,
            queue_id,
            swapchain,
            device,
            images,
            allocator,
            command_buffer_allocator,
            viewport,
        });
        self.renderer
            .lock()
            .unwrap()
            .assemble_self(&self.window_data.clone().unwrap());
    }

    async fn handle_redraw_request(
        info: WindowRedrawContext,
        renderer: Arc<std::sync::Mutex<(dyn VulkanRenderer + Send)>>,
    ) {
        let win: WindowData;
        // let render_pass: [Arc<RenderPass>; NUM_PASSES];
        // let pipeline: [Arc<GraphicsPipeline>; NUM_PASSES];
        // let framebuffer: [Arc<Framebuffer>; NUM_PASSES];
        {
            let lock = renderer.lock().unwrap();
            win = lock.return_window_data();
            // render_pass = lock.return_render_pass();
            // pipeline = lock.return_pipeline();
        }

        let (image_idx, _, swapchain_future): (u32, bool, swapchain::SwapchainAcquireFuture) =
            match swapchain::acquire_next_image(win.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(n) => n,
                Err(e) => {
                    Self::send_redraw_req(info, &win).await;
                    println!("{e}");
                    return;
                } // Err(vulkano::VulkanError::OutOfDate) => return,
                  // Err(e) => panic!("Could not find next image {e}"),
            };
        // println!("started!");
        // {
        //     let lock = renderer.lock().unwrap();
        //     framebuffer = lock.load_framebuffers(
        //         win.images[image_idx as usize].clone(),
        //         win.allocator.clone(),
        //     );
        // }

        // let framebuffer: [Arc<Framebuffer>; NUM_PASSES] = load_framebuffers(
        //     win.images[image_idx as usize].clone(),
        //     render_pass.clone(),
        //     win.allocator.clone(),
        // );

        // let comp: Arc<PrimaryAutoCommandBuffer>;
        {
            // vulkano::command_buffer::;
            // let mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer> =
            //     AutoCommandBufferBuilder::primary(
            //         win.command_buffer_allocator.deref(),
            //         win.queue_id,
            //         vulkano::command_buffer::CommandBufferUsage::MultipleSubmit,
            //     )
            //     .expect("Failed to build builder!!1");
            // AutoCommandBufferBuilder::secondary(
            //     &win.command_buffer_allocator.clone(),
            //     win.queue_id,
            //     CommandBufferUsage::MultipleSubmit,
            //     CommandBufferInheritanceInfo {
            //         ..Default::default()
            //     },
            // );
            // let mut context = RenderPassContext {
            //     swapchain_future, // builder: &mut builder,
            //                       // framebuffer,
            //                       // pipeline: pipeline.clone(),
            // };

            // comp = builder.build().expect("Failed to build command buffer");
        }
        let swapchain_future =
            renderer
                .lock()
                .unwrap()
                .build_renderer(swapchain_future, &win, image_idx);
        if swapchain_future.is_some() {
            let _ = swapchain_future
                .unwrap()
                // .then_execute(win.queue.clone(), comp)
                // .expect("Failed to execute")
                .then_swapchain_present(
                    win.queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(win.swapchain.clone(), image_idx),
                )
                .then_signal_fence_and_flush()
                .expect("Failed to signal")
                .await;

            // println!("finished!");
        } else {
            // println!("err!")
        }
        Self::send_redraw_req(info, &win).await;
        // println!("finished!");
        // vulkano::command_buffer::Rec
    }

    async fn send_redraw_req(info: WindowRedrawContext, win: &WindowData) {
        let _ = info.frame_time.lock().await.tick().await;
        win.window.request_redraw();
    }
}

struct WindowRedrawContext {
    frame_time: Arc<Mutex<Interval>>,
}

pub struct RenderPassContext {
    // pub builder: &'a mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    // pub framebuffer: [Arc<Framebuffer>; NUM_PASSES],
    pub swapchain_future: swapchain::SwapchainAcquireFuture,
    // pub pipeline: [Arc<GraphicsPipeline>; NUM_PASSES],
}

impl<const NUM_PASSES: usize> ApplicationHandler for VulkanWindow<NUM_PASSES> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.load_swapchain(event_loop);
    }
    fn device_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.renderer
            .lock()
            .expect("Failed to handle device")
            .handle_device_event(event, event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.runtime
                    .clone()
                    .spawn(VulkanWindow::<NUM_PASSES>::handle_redraw_request(
                        WindowRedrawContext {
                            frame_time: self.frame_time.clone(),
                        },
                        // self.pipeline_info.as_ref().unwrap().clone(),
                        self.renderer.clone(),
                    ));
            }
            event => self
                .renderer
                .lock()
                .unwrap()
                .handle_window_event(event, event_loop),
        }
    }
}
