# CHECKPOINT - EVA-OS
**Data:** 2026-02-19

---

## O QUE E O PROJETO
Sistema operacional AI-native visando Redox OS (microkernel Rust). Dois eixos:
1. **NPU Driver**: driver userspace Intel NPU (Meteor Lake 0x7D1D) para Redox OS
2. **EVA Daemon**: assistente voz IA com wake word, streaming audio, Time Machine AI

**Tech Stack:** Rust 100%, Tokio async, cpal audio, Gemini API, AES-256-GCM, ONNX Runtime, SQLite

---

## O QUE FUNCIONA
### NPU Driver (drive/)
- PCI discovery Intel Meteor Lake NPU
- MMIO read/write volatile
- DMA contiguous memory
- Firmware loading com validacao magic number
- Boot 6 fases com hexspeak protocol
- Command queue ring buffer 256 slots
- Redox npu: scheme
- Mock mode (Windows/Linux dev)
- 5 testes passando

### EVA Daemon (eva-daemon/)
- Audio capture/playback real-time (cpal 16kHz PCM16)
- Wake word "Hey EVA" (Energy/MFCC/ONNX)
- VAD (RMS + zero-crossing)
- EVA-Mind WebSocket (wss://eva-ia.org:8090)
- Gemini API WebSocket (BidiGenerateContent)
- Sessions encriptadas AES-256-GCM
- Command parser + executor sandboxed
- Time Machine AI (screenshot+OCR+embeddings+busca+encriptacao)
- Terminal UI ANSI
- Emotion detection (8 tipos)

### C FFI (driver-c-api/) + ONNX API + OpenVINO
- C FFI: 9 funcoes exportadas com cbindgen
- DLLs OpenVINO pre-built presentes

---

## O QUE FALTA
1. **Offline STT (Vosk)** - codado mas nao integrado no main loop
2. **Local TTS (piper-rs)** - nao iniciado
3. **Gemini client** - codado mas main.rs usa EvaMindClient em vez
4. **Command pipeline** - parser/executor nao conectados (sem STT -> sem texto)
5. **TimeMachine delete_today()** - stub (retorna 0)
6. **SemanticIndex** - HashMap simples (README diz FAISS mas nao existe)
7. **OCR ONNX** - retorna "[ONNX OCR Output]" hardcoded
8. **Embedding ONNX** - retorna vec![0.0; 384]
9. **Wake word ONNX** - sempre retorna true sem parsear output
10. **OpenVINO build** - CMake incompleto, fases 2-5 nao iniciadas

---

## BUGS
1. **status_indicator.rs teste** - assert Idle mas initial e Initializing (FALHA)
2. **sysinfo 0.29 API deprecated** - usa traits removidos em 0.30
3. **eva_mind.rs** - send_audio falha se session_created demora
4. **WebSocket receive()** - println! debug em toda mensagem (flood)
5. **Salt fixo Argon2** - criptografia enfraquecida
6. **CPF hardcoded** - "64525430249" no EvaMindConfig
7. **update_memory() a cada ~100ms** - System::new_all() caro chamado excessivamente
8. **tls.rs, logging.rs, stt.rs** - importados mas nunca usados

---

## DEAD CODE
- main_phase1.rs (main antigo)
- Cargo_phase1.toml
- test_inference.rs (solto na raiz)
- gemini.rs (importado mas nunca usado)
- tls.rs, logging.rs, stt.rs (modulos mortos)
- search.rs (struct vazia)

---

## .md PARA DELETAR
- ONNX/projeto.md (0 bytes vazio)
- STATUS_COMPILACAO.md (possivelmente obsoleto)
- MD/Googolplex-Books.md (pertence a outro projeto)
