# Music Visualiser

This repository hosts the Rust implementation of the feature-rich music
visualiser described in the project specification. The current focus is setting
up a maintainable foundation so that the subsystems (audio capture/analysis,
scene management, rendering, and recording) can be implemented incrementally.

## Crate Layou---

## 2) Target Platform & Tech Stack

* **Language**: **Rust** for performance + safety.
* **Graphics**: **wgpu** (Vulkan/Metal/DX12/WebGPU backend) + **WGSL shaders**; support HDR pipeline where available.
* **Windowing/UI**: **winit** + **egui** (inspector, graphs, node editors, preset browser).
* **Audio Capture/IO**: **cpal** (loopback/host capture) for Live; **symphonia** for file decode (MP3/WAV/FLAC) in Precomputed.
* **DSP/Analysis**: **rustfft/realfft** for STFT; custom onset/beat/tempo/section detectors; mel‑spectrogram, chroma; spectral centroid/roll‑off/flatness; RMS/energy.
* **3D Assets**: **stl_io** (or custom) for STL; optional **gltf** later. Mesh processing utilities (normals, edge extraction, point sampling).
* **Scheduling/Graph**: custom scene graph + modulation graph (audio features, math ops, LFOs, envelopes) with hot‑reloadable WGSL.
* **Recording**: offscreen render → **ffmpeg** pipe (spawned process) for MP4/ProRes; deterministic timeline in Precomputed.
* **Config/Presets**: human‑readable **TOML/JSON** with JSON Schema for validation.

> Alternate stack (if required): C++20 + GLFW + Vulkan/OpenGL + RtAudio/libsndfile/Essentia; same architecture.

---

## 3) Core Architecture

### High-Level Modules

1. **Audio Engine**

   * Live: host loopback capture → ring buffer → analysis.
   * Precomputed: decode full track → analysis cache (JSON+binary features) → deterministic playback clock.
2. **Analysis Engine**

   * STFT pipeline with hop sizes tuned for beat/onset vs. timbre features.
   * Feature extractors: RMS, mel bands, spectral centroid/roll‑off/flatness, MFCC/chroma (optional), tempo & beat grid (multi‑tempo hypothesis + Viterbi), onset strength, section segmentation (novelty + self‑similarity matrix).
   * Outputs: time‑indexed feature buffers (float32), beat grid with confidence, section markers, “anticipated drop” markers.
3. **Visual Engine**

   * Render graph (wgpu passes) → geometry stage → material stage → post‑FX.
   * Scene system: declarative scene descriptors (what to draw) + controller scripts (how to modulate/transition).
   * Assets: STL loader → Mesh, Wireframe (edge list), PointCloud (Poisson/stratified sampling), Normals/Curvature.
   * Procedurals: SDF raymarcher, particle systems (compute), kaleidoscope/mirror/tunnel passes, noise fields (Perlin/Simplex/Worley), reaction‑diffusion (optional compute).
4. **Modulation & Mapping**

   * Parameter Mapping Matrix: sources (features/LFOs/envelopes/expressions/MIDI) → targets (shader uniforms, material params, transform channels).
   * Curves (Bezier), smoothing (EMA), dynamic range (min/max), attack/release, side‑chain bands.
5. **Scheduler/Timeline**

   * Live: reactive switching with hysteresis; rate limiter for transitions; beat‑sync phase.
   * Precomputed: follows section/beat timeline; anticipatory cues (pre‑roll before drops); crossfades and morphs.
6. **UI/UX**

   * Performance HUD (FPS, latency, audio meters).
   * Preset browser, Scene playlist, Mapping matrix editor, Node‑graph view for modulation, STL controls.
   * “Quick Start” wizard and one‑click Showcase.
7. **Persistence**

   * Project file (bundle): audio file ref (hash), analysis cache, scenes, mappings, camera paths, render settings.

---

## 4) Data Flow

* **Live**: CPAL stream → circular buffer → frame STFT → features → mapping → shader uniforms → render → (optional) live record.
* **Precomputed**: Decode entire file → multi‑pass analysis → write `trackname.analysis.json` + `trackname.features.bin` → deterministic playback clock → timeline scheduler → render.

---

## 5) Audio Analysis Details (Deterministic in Precomputed)

* **STFT**: 2048/4096 window (Hann), hop 256–512; dual‑resolution: small hop for transients, larger for timbre/tempo.
* **Features** (per frame):

  * RMS/energy; per‑band energy (e.g., 8–32 mel bands).
  * Spectral centroid, roll‑off (e.g., 85/95%), flatness, flux.
  * Onset strength (superflux), peak picking → onset times.
  * Tempo estimation: autocorrelation + tempogram + multi‑hypothesis smoothing → beat grid + phase.
  * Section boundaries: novelty function from self‑similarity matrix; pick peaks → label sections A/B/C; compute energy slope, brightness.
  * "Drop predictor": detect concurrent low‑mid energy dip + rising tension + big onset cluster → emit anticipatory cue `t_drop-∆`.

**Outputs**

```json
{
  "sr": 48000,
  "hop": 512,
  "tempo_bpm": 128,
  "beats": [{"t": 1.245, "conf": 0.92}, ...],
  "onsets": [0.412, 0.789, ...],
  "sections": [{"start":0.0,"end":31.5,"label":"A"},...],
  "features": {
    "rms": [ ... ],
    "centroid": [ ... ],
    "mel": [[...], ...]
  },
  "drop_cues": [29.8, 92.1]
}
```

---

## 6) Scene System

### Scene Types (initial set)

* **STL Scene**: displays loaded STL; modes = Mesh (PBR-ish), Wireframe, PointCloud, Normals, Curvature heatmap.
* **Kaleidoscope/Mirror**: screen‑space N‑fold symmetry; rotational + mirror planes; time‑varying.
* **Tunnel**: radial/spiral UV warp; depth‑mapped textures or SDF tunnel.
* **Particles**: GPU compute; emitters modulated by bands; turbulence noise; attract to STL surface.
* **SDF Playground**: raymarched primitives (torus/sphere/fractal); audio drives deformers.
* **Pattern Generator**: tiling noise fields, Voronoi/Worley, L‑systems (optional), cellular automata.

### Transitions

* Cross‑fade, additive blend, glitch cut, bloom ramp, morph (when same geometry), camera whip. Beat‑locked; section‑aware in Precomputed.

### Camera

* Procedural camera (noise paths), spline paths, beat‑snap; FOV & dolly/zoom mapped to energy.

---

## 7) Parameter Mapping & Modulation

* **Sources**: per‑band energies, beat phase (0–1), onset impulses, section IDs, drop countdown, RMS/centroid; LFOs (sine/saw/s+h), envelopes (AR/ADSR), randoms, expressions.
* **Targets**: any uniform/UBO: colour hue/sat/value, light intensity, bloom, distortion strength, kaleidoscope order, tunnel speed, STL transform/scale/wobble, particle rate, camera FOV, etc.
* **Operators**: scale/offset, clamp, curve (Bezier), smoothing (EMA), attack/release, side‑chain, remap by BPM grid.
* **Editor**: matrix view + node graph; saveable **Mappings**.

**Mapping Snippet (JSON)**

```json
{
  "mappings": [
    {"source":"mel[0..3].avg","target":"stl.scale","range":[0.9,1.2],"smoothing":0.2},
    {"source":"beat.phase","target":"kaleidoscope.rotation","scale": 6.283},
    {"source":"drop.countdown","target":"camera.zoom","curve":"easeIn"}
  ]
}
```

---

## 8) STL Ingestion & Rendering

* **Load**: parse ASCII/binary STL → de‑dupe vertices, build normals; validate watertight (optional check) and report.
* **Wireframe**: edge extraction (unique edges), GPU line/mesh shader.
* **Point Cloud**: sample N points (area‑weighted triangle sampling); jitter for density modulation.
* **Shading**: PBR‑lite (albedo/roughness/emissive), matcap option; normal/curvature visualisation; rim light driven by highs.
* **Transforms**: position/rotation/scale; time‑varying (audio‑mapped); deformers (noise wobble, pulse swell).

---

## 9) Visual Effects (Post & Space Warp)

* **Post**: exposure, tone‑map, bloom, vignette, chromatic aberration, film grain, CRT, glitch (scanline skip, color split), barrel/pincushion warp.
* **Space Warps**: kaleidoscope, mirrors (planar/cylindrical), tunnel (radial zoom), swirl.
* **Colour System**: global palette, hue‑rotate on beat, palette morph by section; color‑grade LUTs.

---

## 10) UI/UX

* **Home**: choose Live vs Precomputed; drag‑and‑drop audio or STL.
* **Inspector**: scene params, mapping per target with learn‑mode (wiggle a control while music plays to auto‑map from likely sources).
* **Analysis View (Precomputed)**: waveform, beats, sections, drop markers; timeline editor to schedule scenes; preview scrub.
* **Performance HUD**: FPS, frame time, GPU VRAM, audio latency; “Safe Mode” toggle lowers quality.
* **Presets**: curated show presets; user save/load; import/export ZIP.

---

## 11) Configuration & Presets (Schema)

* **Project File** (`.aviz` JSON/TOML bundle)

```json
{
  "version": 1,
  "audio": {"mode": "precomputed", "file": "song.mp3"},
  "analysis": "song.analysis.json",
  "scenes": [
    {"id": "stl_showcase", "type": "stl", "asset": "model.stl", "mode": "mesh"},
    {"id": "kaleido", "type": "kaleidoscope", "order": 8}
  ],
  "playlist": [
    {"scene": "stl_showcase", "enter": 0.0, "exit": 30.0, "transition": "bloom_cross"},
    {"scene": "kaleido", "enter": "section:B", "exit": "section:C", "transition": "glitch_cut"}
  ],
  "mappings": { /* see §7 */ },
  "render": {"width":1920, "height":1080, "fps":60, "hdr":false}
}
```

* **JSON Schema**: deliver alongside implementation for validation in UI and CI.

---

## 12) CLI & Headless

* `aviz live --preset Showcase`
* `aviz precompute song.mp3 --out song.analysis.json`
* `aviz render project.aviz --video out.mp4 --fps 60 --bitrate 25M`
* Options: `--gpu`, `--safe-mode`, `--duration`, `--from`, `--to`.

---

## 13) File/Module Structure (Rust)

```
crates/
  app/                 # binary; UI, main event loop
  core/
    audio/            # cpal, ring buffer, resampler
    analysis/         # stft, features, beat/tempo, sections
    timeline/         # scheduler, transitions
    mapping/          # modulation graph, sources/targets
    scene/            # scene descriptors, loaders
    render/           # wgpu setup, passes, pipelines, postfx
    assets/           # STL loader, textures, cache
    record/           # offscreen + ffmpeg pipe
    config/           # serde + schema
  shaders/            # WGSL files (modular)
  presets/            # builtin .aviz and assets
```

---

## 14) Shader/Pipeline Overview (WGSL)

* **Geometry Pass**: STL/particles/SDF → G‑buffer (pos, normal, albedo, emissive).
* **Lighting**: single directional + rim + environment (IBL optional); HSV phase rotation.
* **Post**: bloom (dual‑Kawase), tone‑map (ACES), aberration, film grain, vignette, CRT, glitch.
* **Warp**: kaleidoscope/mirror/tunnel as separate screen passes with controllable uniforms.

**Uniform Block Example**

```wgsl
struct AudioUniforms {
  rms: f32,
  centroid: f32,
  mel: array<f32, 32>,
  beat_phase: f32,
  section_id: u32,
}
```

---

## 15) Performance Targets & Tuning

* **Frame time budgets**: 1080p60 on GTX 1660‑class GPU; aim < 10 ms render, < 1 ms mapping, < 3 ms analysis (live).
* **Async**: analysis on dedicated thread; GPU pipelines pre‑compiled; descriptor pools reused; bindless where possible.
* **Quality Levels**: dynamic resolution, particle count, blur iterations.

---

## 16) Testing & Validation

* **Unit tests**: STFT, onset peak‑picking, tempo grid alignment, JSON schema validation.
* **Golden tests** (precomputed): deterministic renders for short snippets.
* **Latency tests** (live): audio → visual oscilloscope vs. screen capture.
* **STL tests**: large meshes (≥ 1M tris), degenerate triangles, non‑watertight handling.
* **Property tests**: mapping graph acyclicity, NaN/Inf rejection, bounds.

---

## 17) Presets to Ship (Curated)

1. **Showcase**: auto‑loads kaleidoscope + STL morph + tunnel; tasteful bloom; colour phases on beat.
2. **Wire Pulse**: STL wireframe + bass‑driven thickness + highs sparkle.
3. **Point Surge**: point cloud pulses + radial tunnel on drops.
4. **SDF Nebula**: raymarched forms breathing with mids; chroma on onsets.
5. **Mirror Hall**: mirrors + camera whip on sections; cool→warm palette morph.

---

## 18) Extensibility & Plugins

* **Shader Plugins**: load WGSL at runtime; declared uniforms auto‑exposed to mapping matrix.
* **Scene Plugins**: dynamic library API (C‑ABI) for custom scene nodes.
* **MIDI/OSC** (optional): extra modulation sources.

---

## 19) Privacy & Permissions

* Live mode uses system loopback only; no network data. File analysis happens locally. Crash logs redact file paths by default.

---

## 20) Deliverables & Milestones

**M1 — Core Skeleton (2–3 weeks)**

* Window, wgpu setup, egui HUD, CPAL live capture, basic STFT, one procedural scene, minimal mapping.

**M2 — Analysis & Precompute (2 weeks)**

* File decode, full feature set, beat/tempo/sections, analysis cache, deterministic playback.

**M3 — STL & Scenes (2 weeks)**

* STL loader + mesh/wire/points; kaleidoscope/tunnel; transitions; colour system.

**M4 — UI & Presets (2 weeks)**

* Mapping editor, playlist timeline, presets; recording pipeline.

**M5 — Hardening (1–2 weeks)**

* Performance tuning, tests, crash handling, schema docs.

---

## 21) Acceptance Criteria (Definition of Done)

* Runs on Windows/Linux/macOS; 1080p60 on mid‑range GPU using Showcase preset.
* Live latency < 50 ms; Precomputed renders deterministic across runs.
* Ships with 5+ curated presets; STL modes functional; parameter mapping robust; transitions beat‑aware.
* CLI headless render works and matches UI render.

---

## 22) Example Pseudo‑Code (Key Paths)

**Live Loop (simplified)**

```rust
loop {
  audio.fill_ring();
  if analyzer.ready_frame() { features = analyzer.compute_frame(); }
  mappings.update(features, clock.now());
  scheduler.tick(features);
  renderer.draw(scenes, mappings.snapshot());
}
```

**Precompute → Render**

```rust
let analysis = analyze_file("song.mp3");
let timeline = plan_scenes(&analysis.sections, &analysis.drop_cues);
play_deterministic(|t| {
  let feats = analysis.sample_at(t);
  scheduler.tick_with_timeline(t, &analysis);
  renderer.draw(&timeline.scenes, mappings.with(feats));
});
```

---

## 23) Risk Log & Mitigations

* **OS loopback capture variability** → fallbacks (WASAPI loopback on Windows; CoreAudio aggregate on macOS; PulseAudio/PIPEWIRE on Linux) + doc.
* **Shader hot‑reload crashes** → sandbox compile then swap.
* **Large STL performance** → LOD, decimation, point budget cap.
* **Tempo mis‑detection** → multi‑hypothesis with confidence; manual BPM override.

---

## 24) Licensing

* Core under MIT/Apache‑2.0; include attribution for third‑party crates; ship shaders and presets under same.

---

## 25) Handoff Notes for LLM Implementation

* Treat this spec as source of truth.
* Prefer deterministic, testable components; expose tunables but ship opinionated defaults.
* Start with Showcase preset end‑to‑end; iterate toward full scene set and analysis fidelity.t

The workspace contains two crates:

- `music-visualiser-core`: reusable library that defines the major subsystems
  (audio, analysis, mapping, scheduling, assets, rendering, recording, and
  configuration). Each module exposes lightweight placeholder types that will be
  expanded with real functionality in subsequent milestones.
- `music-visualiser-app`: command-line entry point with a `live` and a
  `precompute` subcommand mirroring the two main operating modes of the
  application. The binary wires together the core scaffolding and initialises
  tracing/logging so runtime instrumentation is ready once the heavy lifting is
  implemented.

## Getting Started

Ensure you have a stable Rust toolchain installed (Rust 1.74+ recommended) and
run:

```bash
cargo check
```

This validates that the workspace builds successfully. As the project evolves
additional commands (e.g., `cargo run -- live`) will become available.
