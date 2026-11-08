//! Present timing through `VK_GOOGLE_display_timing`: frame generation tells the
//! compositor when each frame should reach the screen, and reads back when the
//! frames really did. On Android the platform's own swapchain offers the extension
//! on every Vulkan driver (it is what the Android Frame Pacing library uses), and
//! SurfaceFlinger holds a queued frame until its time. Elsewhere it is usually absent.

/// A finished present, as the compositor reports it (nanoseconds on `CLOCK_MONOTONIC`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Past {
    pub id: u32,
    /// The time the present asked for (0: none).
    pub desired_ns: u64,
    /// When the frame reached the screen.
    pub actual_ns: u64,
}

/// Whether the device was opened with present timing (`wgpu::Features::VULKAN_GOOGLE_DISPLAY_TIMING`).
pub fn supported(device: &wgpu::Device) -> bool {
    device
        .features()
        .contains(wgpu::Features::VULKAN_GOOGLE_DISPLAY_TIMING)
}

/// Gives the surface's next present the id `id` and an earliest time `desired_ns`
/// (`CLOCK_MONOTONIC` nanoseconds; 0 shows it as soon as possible). False, and
/// nothing set, when the device or the surface can't.
pub fn set_next_present(
    device: &wgpu::Device,
    surface: &wgpu::Surface<'_>,
    id: u32,
    desired_ns: u64,
) -> bool {
    if !supported(device) {
        return false;
    }
    #[cfg(not(target_os = "macos"))]
    {
        use wgpu::hal::api::Vulkan;
        // Safety: only the surface's own swapchain is touched, by the thread that presents it.
        let Some(hal_surface) = (unsafe { surface.as_hal::<Vulkan>() }) else {
            return false;
        };
        if hal_surface.raw_native_swapchain().is_none() {
            return false;
        }
        hal_surface.set_next_present_time(ash::vk::PresentTimeGOOGLE {
            present_id: id,
            desired_present_time: desired_ns,
        });
        true
    }
    #[cfg(target_os = "macos")]
    {
        let _ = (surface, id, desired_ns);
        false
    }
}

/// The display's refresh period in nanoseconds, as the swapchain reports it.
pub fn refresh_cycle_ns(device: &wgpu::Device, surface: &wgpu::Surface<'_>) -> Option<u64> {
    if !supported(device) {
        return None;
    }
    #[cfg(not(target_os = "macos"))]
    {
        use wgpu::hal::api::Vulkan;
        // Safety: read-only queries of this device's live swapchain, from the thread that presents it.
        let hal_device = unsafe { device.as_hal::<Vulkan>() }?;
        let hal_surface = unsafe { surface.as_hal::<Vulkan>() }?;
        let swapchain = hal_surface.raw_native_swapchain()?;
        let loader = ash::google::display_timing::Device::new(
            hal_device.shared_instance().raw_instance(),
            hal_device.raw_device(),
        );
        let cycle = unsafe { loader.get_refresh_cycle_duration(swapchain) }.ok()?;
        (cycle.refresh_duration > 0).then_some(cycle.refresh_duration)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = surface;
        None
    }
}

/// Appends the presents that reached the screen since the last call to `out`.
pub fn past_presents(device: &wgpu::Device, surface: &wgpu::Surface<'_>, out: &mut Vec<Past>) {
    if !supported(device) {
        return;
    }
    #[cfg(not(target_os = "macos"))]
    {
        use wgpu::hal::api::Vulkan;
        // Safety: as `refresh_cycle_ns`.
        let (Some(hal_device), Some(hal_surface)) =
            (unsafe { device.as_hal::<Vulkan>() }, unsafe {
                surface.as_hal::<Vulkan>()
            })
        else {
            return;
        };
        let Some(swapchain) = hal_surface.raw_native_swapchain() else {
            return;
        };
        let loader = ash::google::display_timing::Device::new(
            hal_device.shared_instance().raw_instance(),
            hal_device.raw_device(),
        );
        if let Ok(past) = unsafe { loader.get_past_presentation_timing(swapchain) } {
            out.extend(past.iter().map(|p| Past {
                id: p.present_id,
                desired_ns: p.desired_present_time,
                actual_ns: p.actual_present_time,
            }));
        }
    }
    #[cfg(target_os = "macos")]
    let _ = (surface, out);
}
