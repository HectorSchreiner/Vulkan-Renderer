#![allow(
    dead_code,
    unused_variables,
    clippy::too_many_arguments,
    clippy::unnecessary_wraps
)]

use anyhow::{anyhow, Result};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::{Theme, Window, WindowBuilder};
use std::collections::HashSet;
use std::ffi::CStr;
use std::os::raw::c_void;
use log::*;

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::window as vk_window;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::Version;
use vulkanalia::vk::{ExtDebugUtilsExtension, InstanceCreateFlags};


// prob not gonna make it portable to macos, might remove idunno yet :,)
const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 216);

const VALIDATION_ENABLED: bool =
    cfg!(debug_assertions);

const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

fn main() -> Result<()> {
    pretty_env_logger::init();

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Epic Swaggy Blazingly Fast Renderer")
        .with_theme(Some(Theme::Dark))
        .with_decorations(false)
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(1000, 600))
        .build(&event_loop)?;

    let mut app = unsafe { App::create(&window)? };
    event_loop.run(move |event, elwt| {
        match event {
            // Request a redraw when all events were processed.
            Event::AboutToWait => window.request_redraw(),
            Event::WindowEvent { event, .. } => match event {
                // Render a frame if our Vulkan app is not being destroyed.
                WindowEvent::RedrawRequested if !elwt.exiting() => unsafe { app.render(&window) }.unwrap(),
                // Destroy our Vulkan app.
                WindowEvent::CloseRequested => {
                    elwt.exit();
                    unsafe { app.destroy(); }
                }
                _ => {}
            }
            _ => {}
        }
    })?;

    Ok(())
}

unsafe fn create_instance(window: &Window, entry: &Entry, data: &mut AppData) -> Result<Instance> {
    let available_layers = entry
    // mapped to anyhow for prettier logging, might use my own libary later...
    .enumerate_instance_layer_properties().map_err(|e| anyhow!("{}", e))?
    .iter()
    .map(|l| l.layer_name)
    .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation layer requested but not supported."));
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let application_info = vk::ApplicationInfo::builder()
        .application_name(b"Cool Renderer")
        .engine_name(b"No Engine")
        .application_version(vk::make_version(1, 0, 0))
        .api_version(vk::make_version(1, 0, 0));

    let mut extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();

    if VALIDATION_ENABLED {
        // lets not check if it exists.
        extensions.push(vk::EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    // what the fuck er det her!! future proof my ass
    let flags = vk::InstanceCreateFlags::empty();
    let info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_extension_names(&extensions)
        .enabled_layer_names(&layers)
        .flags(flags);
    
    let instance =  entry.create_instance(&info, None)?;
    if VALIDATION_ENABLED {
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .user_callback(Some(debug_callback));

        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
    
}

// follow system calling convention, otherwise vulkan gets mad
extern "system" fn debug_callback(
    severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    type_: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    // pointer to a struct that contains the debug information
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    match type_ {
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => warn!("Woah there cowboy, slow down... Or should i say speed up? ({:?}) {}", type_, message),
        _ => (),
    }

    // Here i should maybe rplace with the mogging logging libary
    // Just diplays the debugging information, and gives it some cool colors.
    match severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("({:?}) {}", type_, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("({:?}) {}", type_, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!("({:?}) {}", type_, message),
        _ => trace!("({:?}) {}", type_, message)
    }
    // if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::ERROR {
    //     error!("({:?}) {}", type_, message);
    // } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::WARNING {
    //     warn!("({:?}) {}", type_, message);
    // } else if severity >= vk::DebugUtilsMessageSeverityFlagsEXT::INFO {
    //     debug!("({:?}) {}", type_, message);
    // } else {
    //     trace!("({:?}) {}", type_, message);
    // }

    vk::FALSE
}

/// Our Vulkan app.
#[derive(Clone, Debug)]
struct App {
    entry: Entry,
    instance: Instance,
    data: AppData
}

impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        let loader= LibloadingLoader::new(LIBRARY)?;
        // mapped to anyhow for prettier logging, might use my own libary later...
        let entry = Entry::new(loader).map_err(|e| anyhow!("{}", e))?; 
        let mut data = AppData::default();

        let instance = create_instance(window, &entry, &mut data)?;
        Ok(Self {
            entry,
            instance,
            data
        })
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {
        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }
        self.instance.destroy_instance(None);
    }
}

/// The Vulkan handles and associated properties used by the vulkan app.
#[derive(Clone, Debug, Default)]
struct AppData {
    messenger: vk::DebugUtilsMessengerEXT,
}
