// Copyright (c) EVA-OS. All rights reserved.
// Licensed under the MIT License.

/**
 * @file yolo_npu.c
 * @brief Example: Run YOLO object detection on Rodox NPU
 */

#include "onnxruntime/core/providers/rodox_npu/npu.h"
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <yolo.onnx>\n", argv[0]);
        return 1;
    }

    const char* model_path = argv[1];

    // Get NPU device info
    npu_device_info info = npu_get_device_info();
    printf("NPU Device: %s\n", info.name);
    printf("Total Memory: %.2f GB\n", info.total_memory / (1024.0 * 1024.0 * 1024.0));
    printf("Available Memory: %.2f GB\n", info.available_memory / (1024.0 * 1024.0 * 1024.0));
    printf("\n");

    // Load YOLO model
    printf("Loading model: %s\n", model_path);
    npu_model* model = npu_load(model_path);

    if (!model) {
        fprintf(stderr, "Failed to load model: %s\n", npu_get_error());
        return 1;
    }

    printf("Model loaded successfully!\n\n");

    // Create input tensor (example: 1x3x640x640 for YOLOv8)
    int64_t input_shape[] = {1, 3, 640, 640};
    size_t input_size = 1 * 3 * 640 * 640;

    float* input_data = (float*)malloc(input_size * sizeof(float));

    // Fill with dummy data (in real app, this would be preprocessed image)
    for (size_t i = 0; i < input_size; i++) {
        input_data[i] = (float)i / input_size;
    }

    npu_tensor* input = npu_tensor_create(input_data, input_shape, 4, NPU_FLOAT32);
    free(input_data);

    if (!input) {
        fprintf(stderr, "Failed to create input tensor: %s\n", npu_get_error());
        npu_free(model);
        return 1;
    }

    printf("Running inference on NPU...\n");

    // Run inference
    npu_tensor* output = npu_run(model, input);

    if (!output) {
        fprintf(stderr, "Inference failed: %s\n", npu_get_error());
        npu_tensor_free(input);
        npu_free(model);
        return 1;
    }

    printf("Inference completed successfully!\n");

    // Get output data
    const float* output_data = (const float*)npu_tensor_data(output);

    printf("Output tensor (first 10 values):\n");
    for (int i = 0; i < 10; i++) {
        printf("  [%d] = %.6f\n", i, output_data[i]);
    }

    // Cleanup
    npu_tensor_free(input);
    npu_tensor_free(output);
    npu_free(model);

    printf("\nDone!\n");
    return 0;
}
