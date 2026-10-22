use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
  CGEvent, CGEventFlags, CGEventTap, CGEventTapLocation,
  CGEventTapOptions, CGEventTapPlacement, CGEventType,
  CallbackResult,
};
use core_graphics::geometry::CGPoint;
use std::time::Duration;

const TOP_LIMIT: f64 = 8.0;
const ENABLE_SHIFT_BYPASS: bool = true;

// Every event that carries a cursor position. Clamping only the movement
// events leaves clicks reporting the raw position, so they land at whatever
// is 8px above the position the app was told the cursor is at.
static EVENTS_OF_INTEREST: &[CGEventType] = &[
  CGEventType::MouseMoved,
  CGEventType::LeftMouseDown,
  CGEventType::LeftMouseUp,
  CGEventType::LeftMouseDragged,
  CGEventType::RightMouseDown,
  CGEventType::RightMouseUp,
  CGEventType::RightMouseDragged,
  CGEventType::OtherMouseDown,
  CGEventType::OtherMouseUp,
  CGEventType::OtherMouseDragged,
];

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
     event: &CGEvent| {
      // A tap disabled by the system cannot be re-enabled from inside this
      // callback, so just let the event through.
      if matches!(
        etype,
        CGEventType::TapDisabledByTimeout
          | CGEventType::TapDisabledByUserInput
      ) {
        return CallbackResult::Keep;
      }

      // Location first: most events are nowhere near the limit, and this
      // skips the flags fetch on the hot path.
      let location = event.location();
      if location.y >= TOP_LIMIT {
        return CallbackResult::Keep;
      }

      // Shift bypass
      if ENABLE_SHIFT_BYPASS
        && event.get_flags().contains(CGEventFlags::CGEventFlagShift)
      {
        return CallbackResult::Keep;
      }

      // Clamp movement and button events alike
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
        Duration::from_millis(100),
        false,
      );
    },
  );

  // If we can create the tap, we have permissions
  result.is_ok()
}
