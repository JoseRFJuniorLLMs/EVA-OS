# Googolplex-Books x NPU - Status dos Modelos e Integracao
**Data:** 2026-02-08
**Fonte:** REDOX-OS, EVA-OS, Googolplex-Books, D:/MODELOS

---

## 1. INVENTARIO DE MODELOS EM D:/MODELOS

| Modelo | Tamanho | Formato | Status | Uso |
|--------|---------|---------|--------|-----|
| Qwen2.5-32B-Instruct | 58 GB | PyTorch (HuggingFace) | DOWNLOAD INCOMPLETO (8 blobs .incomplete) | Traducao principal |
| Qwen2.5-7B-Instruct | 12 GB | PyTorch (HuggingFace) | DOWNLOAD em andamento (lock ativo) | Fallback leve |
| Qwen2.5-Coder-32B-Instruct | 55 GB | PyTorch (HuggingFace) | Completo (sem lock) | Nao usado no Googolplex |
| SDXL Base 1.0 | 14 GB | Diffusion | Completo | Nao usado no Googolplex |
| qwen2.5-32b-instruct-onnx | 0 bytes | ONNX | VAZIO - NAO CONVERTIDO | Target NPU |
| qwen2.5-coder-32b-instruct-onnx | 0 bytes | ONNX | VAZIO - NAO CONVERTIDO | - |

**Total em disco:** ~139 GB (modelos raw) + 0 GB (ONNX convertido)

---

## 2. MODELOS ONNX EM d:/DEV/models_onnx

| Modelo | Tamanho | Quantizacao | Status |
|--------|---------|-------------|--------|
| Phi-3-Mini-4K-DirectML | 2.1 GB | INT4 (RTN block-32) | COMPLETO e FUNCIONAL |
| qwen2.5-32b-instruct-onnx | 0 bytes | - | VAZIO |
| qwen2.5-7b-instruct-onnx | 0 bytes | - | VAZIO |

---

## 3. CONVERSAO PARA NPU - O QUE FALTA

### Problema Central
Os modelos Qwen foram baixados do HuggingFace em formato PyTorch (safetensors), mas **NENHUM foi convertido para ONNX/OpenVINO IR** que a NPU precisa.

### Script de conversao existe mas NUNCA foi executado:
- **Arquivo:** `d:\DEV\EVA-OS\ONNX\convert_qwen_npu.py`
- **Converte:** Qwen2.5-32B-Instruct → OpenVINO IR para NPU
- **Output esperado:** `D:/MODELOS/qwen2.5-32b-openvino`
- **Dependencias:** `optimum[openvino]`, `transformers`

### Bloqueios para conversao:
1. **Download do Qwen2.5-32B esta INCOMPLETO** (8 blobs .incomplete)
2. O script precisa do modelo completo para converter
3. Conversao de 32B vai exigir ~64GB RAM (modelo + overhead OpenVINO)
4. Com 32GB RAM, pode precisar de quantizacao durante conversao

---

## 4. PIPELINE GOOGOLPLEX-BOOKS - COMO USA OS MODELOS

### Arquitetura de 3 camadas:

```
CAMADA 1 - Traducao (NPU)
  translator_npu.py → ONNX Runtime + DirectML
  Modelo: d:/modelos/qwen2.5-32b-instruct-onnx (VAZIO!)
  Resultado: txt → translated/

CAMADA 2 - Correcao OCR + Notas (CPU/Ollama)
  processor.py → Ollama HTTP API
  Modelo: qwen2.5:32b (Ollama - REMOVIDO)
  Resultado: translated/ → docx/pipeline1/

CAMADA 3 - Bilingue + Semantic Priming (CPU/Ollama)
  processor_bilingual.py → Ollama HTTP API
  Modelo: qwen2.5:32b (Ollama - REMOVIDO)
  Resultado: docx/pipeline1/ → docx/pipeline2/
```

### Configuracao atual (.env):
```
MODEL_BACKEND=ollama          ← Ollama foi REMOVIDO
OLLAMA_MODEL=qwen2.5:32b     ← Modelo nao existe mais
OLLAMA_OPENVINO=1             ← Flag inutil sem Ollama
PARALLEL_CHUNKS=8             ← Otimizado para 32GB RAM
MAX_CHUNK_TOKENS=2000
TEMPERATURE=0.2
```

### NPU Translator (translator_npu.py):
```python
model_path = "d:/modelos/qwen2.5-32b-instruct-onnx"  # PASTA VAZIA!
provider = "DmlExecutionProvider"  # DirectML → NPU
```

---

## 5. INFRAESTRUTURA NPU PRONTA (REDOX-OS)

### OpenVINO NPU Plugin Standalone
- **Local:** `d:\DEV\REDOX-OS\openvino-npu-standalone\`
- **Plugin:** openvino_intel_npu_plugin.dll (4.4 MB)
- **Core:** openvino.dll (15.1 MB)
- **DLL compilada:** openvino_npu_standalone.dll (24 KB)
- **Status:** COMPILADO e FUNCIONAL

### API C disponivel:
```c
npu_plugin_create()     // Inicializar
npu_plugin_compile()    // Compilar ONNX → NPU
npu_plugin_execute()    // Inferencia
npu_plugin_destroy()    // Cleanup
```

### Properties NPU disponiveis:
- Dynamic quantization (INT8 automatico)
- QDQ optimization (ONNX quantizado)
- Turbo mode (max frequencia)
- Defer weights load (carregamento lazy)
- Multi-tile support

---

## 6. DIAGNOSTICO: POR QUE NPU NAO ESTA SENDO USADA

| Etapa | Status | Problema |
|-------|--------|----------|
| Hardware NPU | OK | Intel AI Boost 48 TOPS detectado |
| Driver OpenVINO | OK | 2025.4.1 instalado, NPU reconhecida |
| Plugin standalone | OK | Compilado (24KB DLL) |
| Download Qwen2.5-32B | INCOMPLETO | 8 blobs .incomplete |
| Conversao ONNX | NAO FEITA | Script existe mas nunca rodou |
| Pasta ONNX target | VAZIA | d:/modelos/qwen2.5-32b-instruct-onnx = 0 bytes |
| translator_npu.py | CONFIGURADO | Aponta para pasta vazia |
| Ollama (fallback) | REMOVIDO | Sem backend alternativo |

### Resultado: NENHUM modelo esta pronto para NPU

---

## 7. PLANO DE ACAO PARA ATIVAR NPU NO GOOGOLPLEX

### Passo 1: Completar download do Qwen2.5-32B
```bash
# Verificar e retomar download
python -c "from huggingface_hub import snapshot_download; snapshot_download('Qwen/Qwen2.5-32B-Instruct', cache_dir='D:/MODELOS')"
```
**ETA:** Depende da internet (58GB modelo)

### Passo 2: Converter para ONNX/OpenVINO
```bash
# Opcao A: OpenVINO IR (recomendado para NPU)
python d:\DEV\EVA-OS\ONNX\convert_qwen_npu.py

# Opcao B: ONNX direto (para DirectML)
optimum-cli export onnx --model Qwen/Qwen2.5-32B-Instruct d:/modelos/qwen2.5-32b-instruct-onnx
```
**Problema:** 32B precisa ~64GB RAM para conversao. Com 32GB RAM, opcoes:
- Usar `--weight-format int8` para reduzir memoria
- Converter Qwen2.5-7B primeiro (teste rapido)
- Usar `device_map="auto"` com offloading

### Passo 3: Atualizar Googolplex-Books config
- Apontar `translator_npu.py` para modelo convertido
- Remover referencias ao Ollama
- Testar com um livro pequeno primeiro

### Passo 4: Validar inferencia NPU
```python
from optimum.onnxruntime import ORTModelForCausalLM
model = ORTModelForCausalLM.from_pretrained("d:/modelos/qwen2.5-32b-instruct-onnx", provider="DmlExecutionProvider")
```

---

## 8. ALTERNATIVA RAPIDA: Phi-3-Mini (JA FUNCIONAL)

Enquanto Qwen2.5-32B nao esta convertido, existe um modelo ONNX pronto:

| Modelo | Tamanho | Formato | Local |
|--------|---------|---------|-------|
| Phi-3-Mini-4K | 2.1 GB | ONNX INT4 | d:/DEV/models_onnx/phi-3-mini-4k-directml/ |

**Limitacoes:** Modelo pequeno (3.8B params vs 32B), qualidade inferior para traducao.
**Uso:** Teste rapido de pipeline NPU, nao para producao.

---

## 9. RESUMO FINAL

```
MODELOS PRONTOS PARA NPU:     1 (Phi-3-Mini - teste apenas)
MODELOS EM CONVERSAO:          0
MODELOS COM DOWNLOAD COMPLETO: 2 (Coder-32B, SDXL - nao usados no Googolplex)
MODELOS COM DOWNLOAD PARCIAL:  2 (Qwen-32B, Qwen-7B)
PASTAS ONNX VAZIAS:            4 (todas as targets Qwen)

NPU FUNCIONAL:                 SIM (hardware + driver + plugin OK)
MODELO PARA NPU:               NAO (nenhum Qwen convertido)
GOOGOLPLEX TRADUZINDO:         NAO (sem modelo disponivel)
```

**Conclusao:** A infraestrutura NPU esta 100% pronta (hardware, driver, plugin, codigo). O gargalo e a conversao dos modelos: o download do Qwen2.5-32B precisa ser completado e depois convertido para formato ONNX/OpenVINO IR.
