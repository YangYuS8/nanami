# Live2D Build Notes

This document describes the optional Cubism SDK for Native build skeleton introduced in Nanami 0.10g.

## Scope

- Nanami does not ship the Cubism SDK for Native.
- Nanami does not ship Live2D model resources.
- Nanami 0.10g does not render models.
- Nanami 0.10g only provides an optional CMake build boundary and placeholder adapter path.

## CMake Options

Use these options when configuring `nanami-ui`:

```bash
cmake -S . -B build -G Ninja -DNANAMI_ENABLE_LIVE2D=ON -DNANAMI_CUBISM_SDK_ROOT=/path/to/CubismSdkForNative
```

Available options:

- `NANAMI_ENABLE_LIVE2D`
  - Default: `OFF`
  - Enables the optional Live2D/Cubism build skeleton.
- `NANAMI_CUBISM_SDK_ROOT`
  - Optional path while `NANAMI_ENABLE_LIVE2D=OFF`
  - Required when `NANAMI_ENABLE_LIVE2D=ON`

## Current Behavior in 0.10g

- When `NANAMI_ENABLE_LIVE2D=OFF`, the project builds normally and falls back to the placeholder renderer.
- When `NANAMI_ENABLE_LIVE2D=ON` and the SDK root path is invalid, CMake fails early with a clear configuration error.
- When `NANAMI_ENABLE_LIVE2D=ON` and the SDK root path is valid, the build enables placeholder `src/live2d/` adapter compilation hooks only.

## Not Included Yet

Nanami 0.10g does not do any of the following:

- Include Cubism SDK headers or libraries.
- Parse `model3.json`.
- Load textures.
- Initialize the Cubism runtime.
- Create a real rendering surface.
- Render a Live2D model.

## Follow-Up Work

Before a real Live2D backend is claimed as supported, the project still needs separate work for:

- License review.
- Packaging strategy.
- Platform support validation.
- Real renderer integration.
- Model asset handling.
