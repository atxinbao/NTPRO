// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! FFI types and functions for time event handling.
//!
//! Rust-only builds keep the legacy C layout stable while representing Rust
//! callbacks with a null opaque callback pointer.

use std::ffi::c_char;

use nautilus_core::{
    UUID4,
    ffi::string::{cstr_to_ustr, str_to_cstr},
};
use ustr::ustr;

use crate::timer::{TimeEvent, TimeEventCallback, TimeEventHandler};

#[repr(C)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
/// FFI time event handler for legacy FFI interoperability.
///
/// Associates a `TimeEvent` with a callback function that is triggered
/// when the event's timestamp is reached.
pub struct TimeEventHandler_API {
    /// The time event.
    pub event: TimeEvent,
    /// The callable raw pointer.
    pub callback_ptr: *mut c_char,
}

impl Clone for TimeEventHandler_API {
    fn clone(&self) -> Self {
        Self {
            event: self.event.clone(),
            callback_ptr: self.callback_ptr,
        }
    }
}

impl Drop for TimeEventHandler_API {
    fn drop(&mut self) {}
}

impl TimeEventHandler_API {
    /// Creates a null (sentinel) `TimeEventHandler_API`.
    ///
    /// Used to indicate "no event" when returning from pop operations.
    #[must_use]
    pub fn null() -> Self {
        Self {
            event: TimeEvent::new(ustr(""), UUID4::default(), 0.into(), 0.into()),
            callback_ptr: std::ptr::null_mut(),
        }
    }
}

/// Drops a `TimeEventHandler_API`.
///
/// The handler must be valid and not previously dropped.
#[unsafe(no_mangle)]
pub extern "C" fn time_event_handler_drop(handler: TimeEventHandler_API) {
    drop(handler);
}

impl From<TimeEventHandler> for TimeEventHandler_API {
    fn from(value: TimeEventHandler) -> Self {
        match value.callback {
            TimeEventCallback::Rust(_) | TimeEventCallback::RustLocal(_) => Self {
                event: value.event,
                callback_ptr: std::ptr::null_mut(),
            },
        }
    }
}

/// # Safety
///
/// Assumes `name_ptr` is borrowed from a valid UTF-8 C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn time_event_new(
    name_ptr: *const c_char,
    event_id: UUID4,
    ts_event: u64,
    ts_init: u64,
) -> TimeEvent {
    // SAFETY: `name_ptr` is guaranteed to be a valid C string by the FFI caller contract.
    TimeEvent::new(
        unsafe { cstr_to_ustr(name_ptr) },
        event_id,
        ts_event.into(),
        ts_init.into(),
    )
}

/// Returns a [`TimeEvent`] as a C string pointer.
#[unsafe(no_mangle)]
pub extern "C" fn time_event_to_cstr(event: &TimeEvent) -> *const c_char {
    str_to_cstr(&event.to_string())
}

// This function only exists so that `TimeEventHandler_API` is included in the definitions
#[unsafe(no_mangle)]
pub const extern "C" fn dummy(v: TimeEventHandler_API) -> TimeEventHandler_API {
    v
}
