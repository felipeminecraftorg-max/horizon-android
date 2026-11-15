# Horizon Jade Lab 2.0 — shared console scene

Independent, offline Android visual laboratory (`dev.horizon.jadelab`).
It previews Old School navigation and materials; game installation, accounts,
worlds, mods and actual Minecraft launching stay in the main Horizon app.

## Implemented

- Horizontal categories, vertical contextual options, remembered option per
  category and hierarchical Back (dialog → options → category → Home).
  Pointer, direction-locked touch swipes, keyboard and standard gamepad input
  share the same model. Native Android Back delegates to that model.
- One perspective camera and one point light for all preview panels and the
  six-volume original block character. The WebGL2 fragment shader intersects
  oriented planes and boxes, evaluates GGX highlights, casts shadows between
  objects and computes a limited single-bounce reflection. This is a small
  analytic scene, not a general-purpose ray tracer or global illumination.
- Sharp HTML text and actual button hitboxes use the exact same plane-to-screen
  homography as the GPU. CSS no longer paints separate panel materials.
- Moving procedural ribbons, atmospheric gradients, complementary accent
  pairs, restrained graphite metals and independent material controls.
- The backdrop is cached in a lower-resolution framebuffer. Projected bounds
  and a broad-phase character volume reject unnecessary intersection tests.
  Render scale leaves text sharp. Pause freezes the shared scene clock;
  paused/reduced-motion scenes redraw on input instead of continuously.
- Measured frame rate and CPU submission time. GPU duration is shown only
  when `EXT_disjoint_timer_query_webgl2` supplies a valid measurement; adaptive
  scale responds to render cost, not the intentionally selected frame cap.
- Collapsible/resizable inspector, autosaved controls, individual tab JSON,
  full preset JSON, stored ZIP import/export and the actual GLSL sources.
  ZIPs also include the current plane/character geometry and uniform mapping.
  Legacy presets remain importable. Legacy `glassChrome` / `glassCTA` fields
  are retained in preset compatibility but are not editor controls in 2.0.
- Local downloads use Android MediaStore in `Downloads/JadeLab/`. The app
  requests no internet permission and does not use Horizon's data or keys.

## Validate and build

```sh
node --test experiments/jade-lab/tests/jade-lab.test.cjs
# Playwright and Chrome installed on the test host:
JADE_PLAYWRIGHT_MODULE=/path/to/playwright node experiments/jade-lab/tests/browser-check.cjs
# Android SDK 35, JDK 17+, Gradle 8.14 and the disposable local keystore:
gradle --no-daemon -p experiments/jade-lab/android assembleRelease
```

The isolated `jade-lab-apk.yml` workflow runs Node behavior tests, compiles
both GLSL stages, tests the actual application in Chromium at four viewport
sizes and verifies its APK signature/package/assets. Screenshots and the
behavior report are uploaded separately from the APK.

CI uses software WebGL to verify correctness. Its FPS is **not phone FPS**.
Adreno/Mali appearance, battery use, Android WebView and Bluetooth-controller
behavior still need handset testing. The GPU capture/export contains the
rendered scene without HTML labels or inspector chrome.

## Source

- `android/src/main/assets/scene.js`: navigation, camera and shared poses.
- `renderer.js`: bounded analytic geometry, lighting, shadows and GLSL.
- `app.js`, `palettes.js`: live editor, persistence and compatible exports.
- `scene.css`, `console-refine.css`: responsive text planes and inspector.
- `JadeLabActivity.java`: isolated WebView, Back bridge and local downloads.

The launcher uses Rust/wgpu/WGSL. Exported GLSL and geometry are an engineering
reference for native integration, not an automatically compatible Horizon mod.
Test APKs use a temporary signing identity; an earlier test version signed
with another key may require exporting its presets and uninstalling Jade Lab
before installing this build.
