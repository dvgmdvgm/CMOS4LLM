# ADR-014: llama.cpp (via llama-cpp-2 crate) as Sub-LM runtime

- **Status:** Accepted
- **Date:** 2026-05-26
- **Decision-makers:** owner, claude
- **Related:** ADR-003, ADR-011

## Context

The Two-LLM economy (ADR-003) requires a local inference runtime for Sub-LM tasks: classification, extraction, summarization, dedup, drift detection, and counterfactual replay. Requirements: in-process (no separate server), GGUF model format, CUDA acceleration on Windows, LoRA adapter support (for V3.B research track), and active maintenance.

Owner's hardware: Windows 11 Pro, GPU specs TBD (ROADBLOCKS Q1). Runtime must support both GPU-accelerated and CPU-only fallback.

## Decision

**CMOS uses llama.cpp for local Sub-LM inference, accessed via the `llama-cpp-2` Rust crate.** Models are loaded in GGUF format. CUDA is the primary acceleration backend on Windows. CPU fallback is available when no GPU is detected.

## Rationale

1. **Most mature local inference engine:** llama.cpp is the de facto standard for local LLM inference. Actively maintained, tracks all major model architectures within days of release.
2. **Excellent Rust bindings:** `llama-cpp-2` crate — 111K downloads/month, ranked #3 in AI category on crates.io, updated frequently to track upstream.
3. **GGUF native:** GGUF is llama.cpp's format — first-class support for all quantization levels (Q4_K_M through F16).
4. **CUDA on Windows:** Explicitly supported via `cuda` feature flag. Also supports Vulkan as fallback for non-NVIDIA GPUs.
5. **LoRA support:** llama.cpp supports LoRA adapter loading at runtime — enables V3.B (LoRA-as-memory) research without changing the inference stack.
6. **In-process:** Loads as a library, no separate server process. Direct memory sharing with the Rust daemon.
7. **Model flexibility:** Can run any GGUF model — Qwen 2.5 Coder 14B, Phi-4, Llama 3.x, etc. Model choice is orthogonal to runtime choice.

## Consequences

### Positive
- Single runtime covers all Sub-LM task types (classification through counterfactual replay).
- In-process means <1ms dispatch overhead (critical for the 30ms classification budget).
- Hot-swappable models — can load different models for different task types if needed.
- Huge community, extensive documentation, rapid bug fixes.
- Supports batched inference for background tasks (extraction, drift scan).

### Negative
- C++ dependency — requires clang/LLVM for building on Windows. Adds build complexity.
- Bindings are "not safe" per maintainers — potential UB through misuse. Must wrap carefully.
- Tightly coupled to upstream C++ — breaking changes in llama.cpp API propagate to the crate.
- CUDA toolkit required for GPU acceleration (user must install separately or we bundle).

### Neutral / unknowns
- Exact model choice for MVP (Q1 in ROADBLOCKS — depends on owner's GPU/RAM).
- Whether to use one model for all tasks or specialize (small model for classification, larger for extraction).
- KV-cache management strategy for concurrent requests.
- Quantization level tradeoff (Q4_K_M for speed vs Q8 for quality) — needs benchmarking on Django code.

## Alternatives considered

- **Candle (HuggingFace Rust):** Native Rust ML framework, 664K downloads/month. Rejected: it's a framework, not a ready-to-use inference engine. No built-in GGUF loading (focuses on safetensors), no LoRA hot-swap, no batched inference out of the box. Would require building the entire inference pipeline from scratch.
- **Ollama:** Go binary wrapping llama.cpp with REST API. Rejected: separate process (not in-process), REST adds latency, no fine-grained control over KV-cache or batching, LoRA support is limited (baked into Modelfile, not hot-swappable). Wrong architecture for a tightly-integrated Sub-LM.
- **vLLM:** Python, Linux-focused. Rejected: no Windows support, requires separate process, Python dependency, designed for server-scale throughput not desktop single-user.
- **MLX:** Apple Silicon only. Rejected: owner is on Windows.

## Implementation notes

- Crate: `llama-cpp-2` with `cuda` feature flag.
- Model storage: `~/.cmos/models/` directory. Models downloaded on first use or bundled.
- Wrapper crate: `crates/sub-lm/` — safe Rust API over unsafe bindings. Handles model loading, context management, batching, and error recovery.
- Task dispatch: priority queue (live classification > background extraction > drift scan > counterfactual).
- Concurrency: single model loaded at a time in MVP. V1 may support multiple model slots if VRAM allows.
- Fallback: if no CUDA device detected, run on CPU (slower but functional). If model too large for available RAM/VRAM, fall back to Haiku API calls (ADR-003 fallback profile).

## Revisit conditions

- If `llama-cpp-2` crate is abandoned — evaluate direct FFI to llama.cpp C API or switch to candle if it matures.
- If a Rust-native inference engine with GGUF support and comparable performance emerges — evaluate for reduced build complexity.
- If owner's GPU cannot run 14B models at acceptable speed — downgrade to 7B/3B models or increase reliance on cloud fallback. This doesn't change the runtime choice, only the model choice.
- If Sub-LM extraction quality on Django code is <40% — the problem is likely model choice, not runtime. Try larger models or fine-tuned variants before changing runtime.
