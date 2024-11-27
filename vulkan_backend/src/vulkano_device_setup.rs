use std::sync::Arc;

use vulkano::{
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, Features, Queue, QueueCreateInfo, QueueFlags,
    },
    image::{Image, ImageUsage},
    instance::{Instance, InstanceCreateInfo, InstanceExtensions},
    swapchain::{Surface, Swapchain, SwapchainCreateInfo},
    VulkanLibrary,
};
use winit::{
    dpi::PhysicalSize, event_loop::ActiveEventLoop, platform::x11::WindowAttributesExtX11,
    window::Window,
};

pub fn load_vulkan_instance(event_loop: &ActiveEventLoop) -> Arc<Instance> {
    let lib = VulkanLibrary::new().expect("No vulkan loaded!");
    // let enabled_extensions = InstanceExtensions {
    //     khr_xlib_surface: true,
    //     // khr_swapchain: true,
    //     ..InstanceExtensions::default()
    // };
    let creation_info: InstanceCreateInfo = InstanceCreateInfo {
        enabled_extensions: Surface::required_extensions(event_loop),
        ..Default::default()
    };
    Instance::new(lib, creation_info).expect("Failed to initialize vulkan")
}

pub fn make_device(phys_device: Arc<PhysicalDevice>) -> (Arc<Device>, Arc<Queue>) {
    let (idx, _dev) = phys_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .find(|(_n, d)| d.queue_flags.contains(QueueFlags::GRAPHICS))
        .expect("No graphics device");
    let enabled_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::default()
    };
    let enabled_features = Features {
        geometry_shader: true,
        ..Default::default()
    };
    let (device, mut queues) = Device::new(
        phys_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index: idx as u32,
                ..Default::default()
            }],
            enabled_extensions,
            enabled_features,
            ..Default::default()
        },
    )
    .expect("Could not create device");
    (device, queues.next().unwrap())
}

pub fn make_phys_device(
    instance: Arc<Instance>,
    surface: Arc<Surface>,
) -> (Arc<PhysicalDevice>, u32) {
    let req_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };
    let phys_device = instance
        .enumerate_physical_devices()
        .expect("Failed to load devices")
        .filter(|d| d.supported_extensions().contains(&req_ext))
        .filter_map(|f| {
            f.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, queue)| {
                    queue.queue_flags.contains(QueueFlags::GRAPHICS)
                        && f.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|queue| (f, queue as u32))
        })
        .min_by_key(|(d, _)| match d.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 100,
        })
        // .next()
        .expect("No devices");
    phys_device
    // todo!()
}

pub fn make_window(event_loop: &ActiveEventLoop, size: [u32; 2]) -> Arc<Window> {
    let size = PhysicalSize::new(size[0], size[1]);
    let attr = Window::default_attributes()
        .with_base_size(size)
        .with_min_inner_size(size)
        .with_max_inner_size(size)
        .with_title("SLAM")
        .with_active(false);

    Arc::new(
        event_loop
            .create_window(attr)
            .expect("Failed to create window for event loop"),
    )
}

pub fn make_surface(instance: Arc<Instance>, window: Arc<Window>) -> Arc<Surface> {
    Surface::from_window(instance.clone(), window.clone())
        .expect("Failed to make surface for window")
}
pub fn make_swapchain(
    phys_device: Arc<PhysicalDevice>,
    surface: Arc<Surface>,
    window: Arc<Window>,
    device: Arc<Device>,
) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let caps = phys_device
        .surface_capabilities(&surface, Default::default())
        .expect("Failed to extract capabilities from device");
    let dim = window.inner_size();
    let composite_alpha = caps
        .supported_composite_alpha
        .into_iter()
        .next()
        .expect("Failed to extract transparency data");
    let image_format = phys_device
        .surface_formats(&surface, Default::default())
        .expect("Failed to retrieve image format")[0]
        .0;
    let swapchain_info = SwapchainCreateInfo {
        min_image_count: caps.min_image_count + 1,
        image_format,
        image_extent: dim.into(),
        image_usage: ImageUsage::COLOR_ATTACHMENT,
        composite_alpha,
        ..Default::default()
    };
    Swapchain::new(device.clone(), surface.clone(), swapchain_info)
        .expect("Failed to create swapchain")
}
