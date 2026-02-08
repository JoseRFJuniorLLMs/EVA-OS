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

print("Converting Qwen2.5-32B for NPU via OpenVINO...")

from optimum.intel.openvino import OVModelForCausalLM
from transformers import AutoTokenizer

model_id = "Qwen/Qwen2.5-32B-Instruct"
output_dir = "D:/MODELOS/qwen2.5-32b-openvino"

print(f"Model: {model_id}")
print(f"Output: {output_dir}")
print("Loading and converting (this takes a while)...")

model = OVModelForCausalLM.from_pretrained(
    model_id,
    export=True,
    device="NPU",
    compile=False,
    cache_dir="D:/MODELOS"
)

print(f"Saving to {output_dir}...")
model.save_pretrained(output_dir)

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
