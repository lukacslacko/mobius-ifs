# Mobius IFS

Interactive browser-based visualizations of iterated function systems (IFS) using Mobius transformations, including 2D, 3D, and quaternion variants.

## Visualizations

Open any HTML file directly in a browser — no build step or server required.

- **index.html** — Main 2D Mobius IFS explorer
- **general.html** — General Mobius transformations
- **quadratic.html** — Quadratic Mobius maps
- **affine.html** — Affine IFS
- **forward.html** — Forward iteration view
- **color.html** — Color-mapped IFS
- **entities.html** — Entity visualization
- **explanation.html** — Visual explanation of the math
- **blowup.html** — Blowup visualization
- **tiled_blowup.html** — Tiled blowup patterns
- **ifs3d.html** — 3D IFS with WebGL
- **blowup3d.html** — 3D blowup visualization
- **ifs_raised.html** — Raised IFS surface
- **quaternion.html** — Quaternion Mobius IFS with volumetric and surface rendering

## Quaternion IFS

`quaternion.html` renders fractals defined by quaternion Mobius transformations on a 3D voxel grid using WebGL2 compute-style shaders.

### Controls

- **Functions**: Number of transforms (1-6)
- **Num/Den degree**: Polynomial degree for numerator/denominator (0-2)
- **Normalize coefficients**: Unit-normalize quaternion coefficients
- **Render mode**: Volume (with density/gamma sliders) or Surface (with threshold slider)
- **Color weight**: Controls per-transform color channel separation
- **Speed**: Animation speed of coefficient oscillation
- **Voxel resolution**: 32 to 512
- **Randomize**: Generate new random coefficients
- **Export params**: (Surface mode) Download current fractal parameters as JSON for offline rendering

### Parameter Export

In surface rendering mode, click **Export params** to download a `fractal_params.json` file containing all parameters needed to reproduce the current fractal:

```json
{
  "version": 1,
  "nT": 3,
  "numDeg": 2,
  "denDeg": 1,
  "normalize": true,
  "colorWeight": 2.0,
  "thresholdPct": 50,
  "ang": [...]
}
```

## High-Resolution Offline Renderer

The `render/` directory contains a Rust program that computes high-resolution quaternion IFS fractals offline and outputs a 3D mesh for rendering in Blender.

### Building

```bash
cd render
cargo build --release
```

### Usage

```bash
./target/release/mobius-ifs-render fractal_params.json [options]
```

Options:

| Flag | Default | Description |
|------|---------|-------------|
| `-n, --resolution` | 512 | Voxel grid resolution (N x N x N) |
| `-i, --iterations` | 80 | Number of IFS iterations |
| `-o, --output` | fractal.ply | Output PLY mesh path |

Memory usage is approximately `2 * N^3 * 12 bytes` for the two ping-pong grids:

| Resolution | Memory |
|-----------|--------|
| 256 | ~0.4 GB |
| 512 | ~3.2 GB |
| 768 | ~10.8 GB |

### Example

```bash
# Export params from the browser, then:
./target/release/mobius-ifs-render fractal_params.json -n 512 -i 80 -o fractal.ply
```

### Rendering with Blender

A Python script is provided to render the PLY mesh with Blender's Cycles path tracer:

```bash
blender --background --python render/render_fractal.py -- fractal.ply output.png
```

The script sets up:
- Principled BSDF material using vertex colors (low roughness, slight metallic)
- Three-point area lighting (key, fill, rim) against a dark background
- Cycles GPU rendering (Metal on Apple Silicon), 256 samples with denoising
- 4K output (3840 x 2160)

You can also open the PLY directly in Blender to adjust materials, lighting, and camera before rendering interactively.

## License

MIT
