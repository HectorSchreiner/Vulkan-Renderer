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
        .with_decorations(true)
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

unsafe fn create_instance(window: &Window, entry: &Entry) -> Result<Instance> {
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

    let extensions = vk_window::get_required_instance_extensions(window)
        .iter()
        .map(|e| e.as_ptr())
        .collect::<Vec<_>>();
    // what the fuck er det her!! future proof my ass
    let flags = vk::InstanceCreateFlags::empty();
    let info = vk::InstanceCreateInfo::builder()
        .application_info(&application_info)
        .enabled_extension_names(&extensions)
        .enabled_layer_names(&layers)
        .flags(flags);
    
    Ok(entry.create_instance(&info, None)?)
}

/// Our Vulkan app.
#[derive(Clone, Debug)]
struct App {
    entry: Entry,
    instance: Instance,
}

impl App {
    unsafe fn create(window: &Window) -> Result<Self> {
        let loader= LibloadingLoader::new(LIBRARY)?;
        // mapped to anyhow for prettier logging, might use my own libary later...
        let entry = Entry::new(loader).map_err(|e| anyhow!("{}", e))?; 
        let instance = create_instance(window, &entry)?;
        Ok(Self {
            entry,
            instance
        })
    }

    /// Renders a frame for our Vulkan app.
    unsafe fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    unsafe fn destroy(&mut self) {
        self.instance.destroy_instance(None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
struct AppData {}
