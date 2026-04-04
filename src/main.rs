use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop};
use core_graphics::event::{
  CGEventFlags, CGEventTap, CGEventTapLocation, CGEventTapOptions,
  CGEventTapPlacement, CGEventType, CallbackResult,
};
use core_graphics::geometry::CGPoint;

const TOP_LIMIT: f64 = 8.0;
const ENABLE_SHIFT_BYPASS: bool = true;

// Helper to get raw value from CGEventType for comparison
fn event_type_to_u32(e: CGEventType) -> u32 {
  // CGEventType is a u32 enum, so we can cast directly
  e as u32
}

fn main() {
  // Check accessibility permissions
  if !is_process_trusted() {
    println!("⚠️  Needs Accessibility permissions.");
    std::process::exit(1);
  }

  let events_of_interest =
    vec![CGEventType::MouseMoved, CGEventType::LeftMouseDragged];

  // Use CGEventTap::with_enabled which takes a closure to run the event loop
  let result = CGEventTap::with_enabled(
    CGEventTapLocation::HID,
    CGEventTapPlacement::HeadInsertEventTap,
    CGEventTapOptions::Default,
    events_of_interest,
    |_proxy: *const std::ffi::c_void,
     etype: CGEventType,
     event: &core_graphics::event::CGEvent| {
      let etype_val = event_type_to_u32(etype);

      // Re-enable tap if it was disabled
      // Note: This would require accessing the tap from a static context
      // For simplicity, we just keep the event and let the system handle it
      if etype_val == CGEventType::TapDisabledByTimeout as u32
        || etype_val == CGEventType::TapDisabledByUserInput as u32
      {
        return CallbackResult::Keep;
      }

      // Only process mouse moved and left mouse dragged
      if etype_val != CGEventType::MouseMoved as u32
        && etype_val != CGEventType::LeftMouseDragged as u32
      {
        return CallbackResult::Keep;
      }

      // Shift bypass
      if ENABLE_SHIFT_BYPASS {
        let flags = event.get_flags();
        if flags.contains(CGEventFlags::CGEventFlagShift) {
          return CallbackResult::Keep;
        }
      }

      let location = event.location();

      // If cursor would go above limit, modify the event to clamp it
      if location.y < TOP_LIMIT {
        let new_location = CGPoint::new(location.x, TOP_LIMIT);
        event.set_location(new_location);
      }

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
  let events_of_interest =
    vec![CGEventType::MouseMoved, CGEventType::LeftMouseDragged];

  // Use CGEventTap::with_enabled to create the tap and run briefly
  let result = CGEventTap::with_enabled(
    CGEventTapLocation::HID,
    CGEventTapPlacement::HeadInsertEventTap,
    CGEventTapOptions::Default,
    events_of_interest,
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
