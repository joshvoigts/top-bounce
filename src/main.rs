use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
  CGEventTap, CGEventTapLocation, CGEventTapOptions,
  CGEventTapPlacement, CGEventType, CallbackResult,
};
use core_graphics::geometry::CGPoint;

const TOP_LIMIT: f64 = 8.0;
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
        CGEventType::TapDisabledByTimeout
          | CGEventType::TapDisabledByUserInput
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
        && event.get_flags().contains(
          core_graphics::event::CGEventFlags::CGEventFlagShift,
        )
      {
        return CallbackResult::Keep;
      }

      // Early exit: if mouse is already above the limit, do nothing
      let location = event.location();
      if location.y >= TOP_LIMIT {
        return CallbackResult::Keep;
      }

      // Clamp cursor to limit
      event.set_location(CGPoint::new(location.x, TOP_LIMIT));

      CallbackResult::Keep
    },
    || {
      println!(
        "✅ Menu bar blocker active (event modification, no warp)"
      );
      println!("   Clamps Y to minimum {:.0}px", TOP_LIMIT);
      println!("   Hold Shift to bypass");
      CFRunLoop::run_current();
    },
  );

  if result.is_err() {
    println!("❌ Failed to create event tap.");
    std::process::exit(1);
  }
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
