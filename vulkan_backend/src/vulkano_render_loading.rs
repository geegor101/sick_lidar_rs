use std::sync::Arc;

use vulkano::{
    device::Device,
    format::Format,
    image::{view::ImageView, Image, ImageUsage},
    memory::allocator::{AllocationCreateInfo, StandardMemoryAllocator},
    ordered_passes_renderpass,
    render_pass::{AttachmentLoadOp, Framebuffer, FramebufferCreateInfo, RenderPass},
    single_pass_renderpass,
    swapchain::Swapchain,
};

//Done per frame
pub fn load_framebuffers(
    // images: Vec<Arc<Image>>,
    img: Arc<Image>,
    render_pass: Arc<RenderPass>,
    allocator: Arc<StandardMemoryAllocator>,
) -> Arc<Framebuffer> {
    // println!("")
    // dbg!(images.len());
    // images
    // .iter()
    // .map(
    // |img| {
    let depth = ImageView::new_default(
        Image::new(
            allocator.clone(),
            vulkano::image::ImageCreateInfo {
                image_type: vulkano::image::ImageType::Dim2d,
                format: Format::D16_UNORM,
                extent: img.extent(),
                usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
                ..Default::default()
            },
            AllocationCreateInfo::default(),
        )
        .expect("Failed to create image for depth buffer"),
    )
    .expect("Failed to create depthbuffer");
    let attachments = vec![
        ImageView::new_default(img.clone()).expect("Failed to make image view"),
        depth,
    ];
    Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments,
            ..Default::default()
        },
    )
    .expect("Failed to create framebuffer")
    //     }, // let attachments = vec![images.];
    // )
    // .collect()
}

pub fn load_framebuffers2(
    // images: Vec<Arc<Image>>,
    img: Arc<Image>,
    render_pass: Arc<RenderPass>,
    allocator: Arc<StandardMemoryAllocator>,
) -> Arc<Framebuffer> {
    // println!("")
    // dbg!(images.len());
    // images
    // .iter()
    // .map(
    // |img| {
    // let depth = ImageView::new_default(
    //     Image::new(
    //         allocator.clone(),
    //         vulkano::image::ImageCreateInfo {
    //             image_type: vulkano::image::ImageType::Dim2d,
    //             format: Format::D16_UNORM,
    //             extent: img.extent(),
    //             usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::TRANSIENT_ATTACHMENT,
    //             ..Default::default()
    //         },
    //         AllocationCreateInfo::default(),
    //     )
    //     .expect("Failed to create image for depth buffer"),
    // )
    // .expect("Failed to create depthbuffer");
    let attachments = vec![
        ImageView::new_default(img.clone()).expect("Failed to make image view"),
        // depth,
    ];
    Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments,
            ..Default::default()
        },
    )
    .expect("Failed to create framebuffer")
    //     }, // let attachments = vec![images.];
    // )
    // .collect()
}

pub fn load_render_pass(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    ordered_passes_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
            depth: {
                format: Format::D16_UNORM,
                samples: 1,
                load_op: Clear,
                store_op: DontCare,
            }
        },
        passes: [
            {
                color: [color],
                depth_stencil: {depth},
                input: []
            },
        ],
    )
    .expect("Failed to create render pass")
    //Format::R8G8B8A8_UNORM
}

pub fn load_render_pass2(device: Arc<Device>, swapchain: Arc<Swapchain>) -> Arc<RenderPass> {
    single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Load,
                store_op: Store,
            },

        },
        pass: {
            color: [color],
            depth_stencil: {}
        },
    )
    .expect("Failed to create render pass")
}
