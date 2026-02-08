// Copyright (c) EVA-OS. All rights reserved.
// Licensed under the MIT License.

//! High-level Rust API for Rodox NPU + ONNX Runtime
//!
//! ```rust
//! use rodox_npu::Model;
//!
//! let model = Model::load("yolo.onnx")?;
//! let output = model.run(&input)?;
//! ```

use std::path::Path;
use anyhow::{Result, Context};

pub struct Model {
    // Internal ONNX Runtime session
    session: *mut std::ffi::c_void,
}

pub struct Tensor {
    data: Vec<f32>,
    shape: Vec<usize>,
}

impl Model {
    /// Load ONNX model and compile for NPU
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref()
            .to_str()
            .context("Invalid path")?;

        unsafe {
            // Call C API: npu_load()
            let model_ptr = npu_load(path_str.as_ptr() as *const i8);

            if model_ptr.is_null() {
                let err = std::ffi::CStr::from_ptr(npu_get_error())
                    .to_str()
                    .unwrap_or("Unknown error");
                anyhow::bail!("Failed to load model: {}", err);
            }

            Ok(Self { session: model_ptr })
        }
    }

    /// Run inference on NPU
    pub fn run(&self, input: &Tensor) -> Result<Tensor> {
        unsafe {
            // Create C tensor
            let c_tensor = npu_tensor_create(
                input.data.as_ptr() as *const std::ffi::c_void,
                input.shape.as_ptr() as *const i64,
                input.shape.len(),
                0, // NPU_FLOAT32
            );

            if c_tensor.is_null() {
                anyhow::bail!("Failed to create input tensor");
            }

            // Run inference
            let output_ptr = npu_run(self.session, c_tensor);

            if output_ptr.is_null() {
                npu_tensor_free(c_tensor);
                anyhow::bail!("Inference failed");
            }

            // Convert output to Rust
            let data_ptr = npu_tensor_data(output_ptr) as *const f32;
            let len = 1000; // TODO: get actual size

            let data = std::slice::from_raw_parts(data_ptr, len).to_vec();

            npu_tensor_free(c_tensor);
            npu_tensor_free(output_ptr);

            Ok(Tensor {
                data,
                shape: vec![1, len],
            })
        }
    }

    /// Get NPU device info
    pub fn device_info() -> DeviceInfo {
        unsafe {
            let info = npu_get_device_info();
            DeviceInfo {
                name: std::ffi::CStr::from_ptr(info.name)
                    .to_string_lossy()
                    .into_owned(),
                total_memory: info.total_memory,
                available_memory: info.available_memory,
            }
        }
    }
}

impl Drop for Model {
    fn drop(&mut self) {
        unsafe {
            npu_free(self.session);
        }
    }
}

impl Tensor {
    pub fn new(data: Vec<f32>, shape: Vec<usize>) -> Self {
        Self { data, shape }
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }

    pub fn shape(&self) -> &[usize] {
        &self.shape
    }
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub total_memory: u64,
    pub available_memory: u64,
}

// FFI to C API
extern "C" {
    fn npu_load(path: *const i8) -> *mut std::ffi::c_void;
    fn npu_run(model: *mut std::ffi::c_void, input: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn npu_free(model: *mut std::ffi::c_void);
    fn npu_tensor_create(data: *const std::ffi::c_void, shape: *const i64, ndim: usize, dtype: i32) -> *mut std::ffi::c_void;
    fn npu_tensor_free(tensor: *mut std::ffi::c_void);
    fn npu_tensor_data(tensor: *const std::ffi::c_void) -> *const std::ffi::c_void;
    fn npu_get_error() -> *const i8;
    fn npu_get_device_info() -> CDeviceInfo;
}

#[repr(C)]
struct CDeviceInfo {
    name: *const i8,
    total_memory: u64,
    available_memory: u64,
}
