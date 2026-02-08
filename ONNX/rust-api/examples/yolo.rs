// Copyright (c) EVA-OS. All rights reserved.
// Licensed under the MIT License.

use rodox_npu::{Model, Tensor};
use anyhow::Result;

fn main() -> Result<()> {
    // Get device info
    let info = Model::device_info();
    println!("NPU Device: {}", info.name);
    println!("Total Memory: {:.2} GB", info.total_memory as f64 / 1e9);
    println!("Available: {:.2} GB\n", info.available_memory as f64 / 1e9);

    // Load YOLO model
    println!("Loading YOLOv8...");
    let model = Model::load("yolov8n.onnx")?;
    println!("Model loaded!\n");

    // Create input (1x3x640x640)
    let input_size = 1 * 3 * 640 * 640;
    let data: Vec<f32> = (0..input_size)
        .map(|i| i as f32 / input_size as f32)
        .collect();

    let input = Tensor::new(data, vec![1, 3, 640, 640]);

    // Run inference on NPU
    println!("Running inference on NPU...");
    let output = model.run(&input)?;
    println!("Inference complete!\n");

    // Print results
    println!("Output shape: {:?}", output.shape());
    println!("First 10 values:");
    for (i, val) in output.data().iter().take(10).enumerate() {
        println!("  [{i}] = {val:.6}");
    }

    Ok(())
}
