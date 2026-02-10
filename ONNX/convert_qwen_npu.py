#!/usr/bin/env python3
import sys, os

if sys.platform == 'win32':
    import codecs
    sys.stdout = codecs.getwriter('utf-8')(sys.stdout.buffer, 'strict')

# Force HuggingFace cache to D:/MODELOS
os.environ['HF_HOME'] = 'D:/MODELOS'
os.environ['HF_HUB_CACHE'] = 'D:/MODELOS'
os.environ['HF_HUB_DISABLE_SYMLINKS_WARNING'] = '1'
os.environ['HF_TOKEN'] = os.getenv('HF_TOKEN', 'YOUR_TOKEN_HERE')

print("Converting Qwen2.5-32B for NPU via OpenVINO (INT8 for 32GB RAM)...")

from optimum.intel.openvino import OVModelForCausalLM, OVWeightQuantizationConfig
from transformers import AutoTokenizer
import gc

model_id = "Qwen/Qwen2.5-32B-Instruct"
output_dir = "D:/MODELOS/qwen2.5-32b-openvino"

print(f"Model: {model_id}")
print(f"Output: {output_dir}")
print("Loading with INT8 weight compression (fits 32GB RAM)...")

quantization_config = OVWeightQuantizationConfig(
    bits=8,
    sym=True,
    group_size=128,
)

model = OVModelForCausalLM.from_pretrained(
    model_id,
    export=True,
    compile=False,
    cache_dir="D:/MODELOS",
    quantization_config=quantization_config,
    low_cpu_mem_usage=True,
)

print(f"Saving to {output_dir}...")
model.save_pretrained(output_dir)
del model
gc.collect()

tokenizer = AutoTokenizer.from_pretrained(model_id, cache_dir="D:/MODELOS")
tokenizer.save_pretrained(output_dir)

print(f"DONE! Saved to {output_dir}")

print("\nTesting inference on NPU...")
model = OVModelForCausalLM.from_pretrained(output_dir, device="NPU")
inputs = tokenizer("Hello, how are you?", return_tensors="pt")
outputs = model.generate(**inputs, max_new_tokens=50)
result = tokenizer.decode(outputs[0], skip_special_tokens=True)

print(f"\nResult: {result}")
print("\nSUCCESS! NPU working!")
