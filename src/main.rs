use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::display::CGDisplay;
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType,
    CallbackResult,
};
use core_graphics::geometry::CGPoint;

const DEFAULT_TOP_LIMIT: f64 = 8.0;
const ENABLE_SHIFT_BYPASS: bool = true;

// Use Vec since that's what the API requires
static EVENTS_OF_INTEREST: &[CGEventType] =
    &[CGEventType::MouseMoved, CGEventType::LeftMouseDragged];

fn main() {
    // Check accessibility permissions
    if !is_process_trusted() {
        println!("⚠️  Needs Accessibility permissions.");
        std::process::exit(1);
    }

    // Detect notch and adjust top limit accordingly
    let top_limit = get_top_limit();

    // Use CGEventTap::with_enabled which takes a closure to run the event loop
    let result = CGEventTap::with_enabled(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        EVENTS_OF_INTEREST.to_vec(),
        |_proxy: *const std::ffi::c_void,
         etype: CGEventType,
         event: &core_graphics::event::CGEvent| {
            // Re-enable tap if it was disabled
            if matches!(
                etype,
                CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
            ) {
                return CallbackResult::Keep;
            }

            // Only process mouse moved and left mouse dragged
            if !matches!(
                etype,
                CGEventType::MouseMoved | CGEventType::LeftMouseDragged
            ) {
                return CallbackResult::Keep;
            }

            // Shift bypass
            if ENABLE_SHIFT_BYPASS
                && event
                    .get_flags()
                    .contains(core_graphics::event::CGEventFlags::CGEventFlagShift)
            {
                return CallbackResult::Keep;
            }

            // Early exit: if mouse is already above the limit, do nothing
            let location = event.location();
            if location.y >= top_limit {
                return CallbackResult::Keep;
            }

            // Clamp cursor to limit
            event.set_location(CGPoint::new(location.x, top_limit));

            CallbackResult::Keep
        },
        || {
            println!("✅ Menu bar blocker active (event modification, no warp)");
            println!("   Clamps Y to minimum {:.0}px", top_limit);
            println!("   Hold Shift to bypass");
            CFRunLoop::run_current();
        },
    );

    if result.is_err() {
        println!("❌ Failed to create event tap.");
        std::process::exit(1);
    }
}

// Detect notch by comparing display bounds vs visible area
fn get_top_limit() -> f64 {
    let main_display = CGDisplay::main();
    let bounds = main_display.bounds();

    // Use raw Core Graphics API to get visible area
    // CGDisplayGetVisibleBounds is not exposed in core-graphics 0.25.0
    let visible_rect = unsafe {
        let mut rect = core_graphics::geometry::CGRect::default();
        let status = CGDisplayGetVisibleBounds(main_display.id, &mut rect);
        if status == core_graphics::base::kCGErrorSuccess {
            rect
        } else {
            // Fall back to bounds if visible area fails
            bounds
        }
    };

    // On notched displays, the visible area's height is less than the bounds
    // The difference indicates the notch area
    let notch_height = bounds.size.height - visible_rect.size.height;

    if notch_height > 0.0 {
        println!("📱 Notch detected (notch height: {:.0}px)", notch_height);
        // Add notch height to the default limit to account for the notch
        DEFAULT_TOP_LIMIT + notch_height
    } else {
        println!("✅ No notch detected");
        DEFAULT_TOP_LIMIT
    }
}

// Raw Core Graphics function declaration
extern "C" {
    fn CGDisplayGetVisibleBounds(
        display: core_graphics::display::CGDirectDisplayID,
        visibleBoundsOut: *mut core_graphics::geometry::CGRect,
    ) -> core_graphics::base::CGError;
}

// AXIsProcessTrusted equivalent
// We check by attempting to create an event tap with a short timeout
fn is_process_trusted() -> bool {
    // Try to create an event tap - this will fail if accessibility is not enabled
    let result = CGEventTap::with_enabled(
        CGEventTapLocation::HID,
        CGEventTapPlacement::HeadInsertEventTap,
        CGEventTapOptions::Default,
        EVENTS_OF_INTEREST.to_vec(),
        |_proxy, _etype, _event| CallbackResult::Keep,
        || {
            // Run the run loop for a short time to allow the tap to be created
            // Then exit, which will cause the tap to be dropped
            let _ = CFRunLoop::run_in_mode(
                unsafe { kCFRunLoopDefaultMode },
                std::time::Duration::from_millis(100),
                false,
            );
        },
    );

    // If we can create the tap, we have permissions
    result.is_ok()
}
