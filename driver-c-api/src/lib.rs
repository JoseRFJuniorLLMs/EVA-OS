// Copyright (c) EVA-OS. All rights reserved.
// Licensed under the MIT License.

//! C API wrapper for EVA-OS Intel NPU Driver
//!
//! This provides a C-compatible interface for ONNX Runtime to use the NPU.

use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;
use std::sync::Mutex;

// Static NPU state
static mut NPU_DEVICE: Option<Mutex<NpuState>> = None;

struct NpuState {
    initialized: bool,
    allocations: Vec<*mut u8>,
}

impl NpuState {
    fn new() -> Self {
        Self {
            initialized: false,
            allocations: Vec::new(),
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_init() -> i32 {
    unsafe {
        if NPU_DEVICE.is_some() {
            return 0; // Already initialized
        }

        let mut state = NpuState::new();

        // TODO: Call actual NPU initialization from intel-npu crate
        #[cfg(target_os = "redox")]
        {
            // Real Redox initialization
            match intel_npu::init_npu() {
                Ok(_) => state.initialized = true,
                Err(_) => return -1,
            }
        }

        #[cfg(not(target_os = "redox"))]
        {
            // Mock for development
            state.initialized = true;
        }

        NPU_DEVICE = Some(Mutex::new(state));
        0
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_shutdown() {
    unsafe {
        if let Some(device) = NPU_DEVICE.take() {
            let mut state = device.lock().unwrap();

            // Free all allocations
            for ptr in &state.allocations {
                libc::free(*ptr as *mut libc::c_void);
            }
            state.allocations.clear();
            state.initialized = false;
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_alloc(size: usize) -> *mut libc::c_void {
    unsafe {
        if let Some(ref device) = NPU_DEVICE {
            let mut state = device.lock().unwrap();

            #[cfg(target_os = "redox")]
            {
                // Real Redox allocation
                let ptr = intel_npu::alloc_npu_memory(size);
                if !ptr.is_null() {
                    state.allocations.push(ptr as *mut u8);
                }
                ptr
            }

            #[cfg(not(target_os = "redox"))]
            {
                // Mock: allocate from system heap
                let ptr = libc::malloc(size);
                if !ptr.is_null() {
                    state.allocations.push(ptr as *mut u8);
                }
                ptr
            }
        } else {
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_free(ptr: *mut libc::c_void) {
    if ptr.is_null() {
        return;
    }

    unsafe {
        if let Some(ref device) = NPU_DEVICE {
            let mut state = device.lock().unwrap();

            // Remove from tracking
            state.allocations.retain(|&p| p != ptr as *mut u8);

            #[cfg(target_os = "redox")]
            {
                intel_npu::free_npu_memory(ptr);
            }

            #[cfg(not(target_os = "redox"))]
            {
                libc::free(ptr);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_memcpy_to_device(
    dst: *mut libc::c_void,
    src: *const libc::c_void,
    size: usize,
) -> i32 {
    if dst.is_null() || src.is_null() {
        return -1;
    }

    unsafe {
        #[cfg(target_os = "redox")]
        {
            match intel_npu::copy_to_device(dst, src, size) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }

        #[cfg(not(target_os = "redox"))]
        {
            ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_memcpy_from_device(
    dst: *mut libc::c_void,
    src: *const libc::c_void,
    size: usize,
) -> i32 {
    if dst.is_null() || src.is_null() {
        return -1;
    }

    unsafe {
        #[cfg(target_os = "redox")]
        {
            match intel_npu::copy_from_device(dst, src, size) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }

        #[cfg(not(target_os = "redox"))]
        {
            ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, size);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_execute(
    blob: *const libc::c_void,
    blob_size: usize,
    inputs: *const *const libc::c_void,
    outputs: *mut *mut libc::c_void,
    num_inputs: usize,
    num_outputs: usize,
) -> i32 {
    if blob.is_null() {
        return -1;
    }

    unsafe {
        #[cfg(target_os = "redox")]
        {
            let blob_slice = std::slice::from_raw_parts(blob as *const u8, blob_size);
            let input_slice = std::slice::from_raw_parts(inputs, num_inputs);
            let output_slice = std::slice::from_raw_parts_mut(outputs, num_outputs);

            match intel_npu::execute_model(blob_slice, input_slice, output_slice) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }

        #[cfg(not(target_os = "redox"))]
        {
            // Mock: just simulate execution delay
            std::thread::sleep(std::time::Duration::from_millis(10));
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_get_total_memory() -> u64 {
    #[cfg(target_os = "redox")]
    {
        intel_npu::get_total_memory().unwrap_or(0)
    }

    #[cfg(not(target_os = "redox"))]
    {
        4 * 1024 * 1024 * 1024 // 4GB mock
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_get_available_memory() -> u64 {
    #[cfg(target_os = "redox")]
    {
        intel_npu::get_available_memory().unwrap_or(0)
    }

    #[cfg(not(target_os = "redox"))]
    {
        4 * 1024 * 1024 * 1024 // 4GB mock
    }
}

#[no_mangle]
pub extern "C" fn eva_npu_get_device_name() -> *const c_char {
    static DEVICE_NAME: &[u8] = b"Intel Meteor Lake NPU (VPU 4.0) via EVA-OS\0";
    DEVICE_NAME.as_ptr() as *const c_char
}
