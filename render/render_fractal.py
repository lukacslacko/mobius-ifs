"""
Blender render script for quaternion IFS fractal meshes.

Usage:
    blender --background --python render_fractal.py -- fractal.ply [output.png]

Options after '--':
    arg 1: PLY file path (required)
    arg 2: output image path (default: fractal_render.png)
"""
import bpy
import sys
import math
import os

argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
ply_path = os.path.abspath(argv[0]) if argv else os.path.abspath("fractal.ply")
output_path = os.path.abspath(argv[1]) if len(argv) > 1 else os.path.abspath("fractal_render.png")

# Clear default scene
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete()
for c in bpy.data.collections:
    for o in c.objects:
        bpy.data.objects.remove(o)

# Import PLY
bpy.ops.wm.ply_import(filepath=ply_path)
obj = bpy.context.selected_objects[0]
bpy.ops.object.origin_set(type='ORIGIN_GEOMETRY', center='BOUNDS')
obj.location = (0, 0, 0)

# Auto-scale to fit in view
dims = obj.dimensions
max_dim = max(dims)
if max_dim > 0:
    obj.scale = tuple(2.0 / max_dim for _ in range(3))
    bpy.ops.object.transform_apply(scale=True)

# Smooth shading
bpy.ops.object.shade_smooth()

# Material: Principled BSDF with vertex colors
mat = bpy.data.materials.new("FractalMat")
nodes = mat.node_tree.nodes
links = mat.node_tree.links
nodes.clear()

output_node = nodes.new('ShaderNodeOutputMaterial')
output_node.location = (400, 0)

principled = nodes.new('ShaderNodeBsdfPrincipled')
principled.location = (0, 0)
principled.inputs['Roughness'].default_value = 0.15
principled.inputs['Metallic'].default_value = 0.3
principled.inputs['Specular IOR Level'].default_value = 0.8

vertex_color = nodes.new('ShaderNodeVertexColor')
vertex_color.location = (-300, 0)
# PLY vertex colors are typically named "Col"
vertex_color.layer_name = "Col"

links.new(vertex_color.outputs['Color'], principled.inputs['Base Color'])
links.new(principled.outputs['BSDF'], output_node.inputs['Surface'])

obj.data.materials.append(mat)

# Dark background
world = bpy.data.worlds['World']
wnodes = world.node_tree.nodes
wlinks = world.node_tree.links
wnodes.clear()

bg = wnodes.new('ShaderNodeBackground')
bg.inputs['Color'].default_value = (0.015, 0.015, 0.025, 1)
bg.inputs['Strength'].default_value = 0.3
wo = wnodes.new('ShaderNodeOutputWorld')
wlinks.new(bg.outputs['Background'], wo.inputs['Surface'])

# Three-point lighting
def add_area_light(name, location, energy, size, track_to=None):
    bpy.ops.object.light_add(type='AREA', location=location)
    light = bpy.context.object
    light.name = name
    light.data.energy = energy
    light.data.size = size
    # Point at origin
    direction = mathutils_look_at(location, (0, 0, 0))
    light.rotation_euler = direction
    return light

def mathutils_look_at(from_pos, to_pos):
    """Compute rotation to look from from_pos at to_pos."""
    dx = to_pos[0] - from_pos[0]
    dy = to_pos[1] - from_pos[1]
    dz = to_pos[2] - from_pos[2]
    dist_xy = math.sqrt(dx*dx + dy*dy)
    rot_x = math.atan2(dist_xy, -dz) - math.pi  # tilt
    rot_z = math.atan2(dy, dx) - math.pi/2       # pan
    return (rot_x + math.pi, 0, rot_z)

add_area_light("Key",  (3, -2, 4),  500, 2)
add_area_light("Fill", (-3, -1, 2), 200, 3)
add_area_light("Rim",  (0, 3, 2),   300, 1.5)

# Camera
bpy.ops.object.camera_add(location=(2.8, -2.8, 2.0))
cam = bpy.context.object
cam.data.lens = 50
cam.data.clip_end = 100

# Point camera at origin
direction = mathutils_look_at((2.8, -2.8, 2.0), (0, 0, 0))
cam.rotation_euler = direction

bpy.context.scene.camera = cam

# Render settings — Cycles path tracer
scene = bpy.context.scene
scene.render.engine = 'CYCLES'

# Prefer GPU if available
prefs = bpy.context.preferences.addons.get('cycles')
if prefs:
    prefs.preferences.compute_device_type = 'METAL'  # macOS M2
    bpy.context.scene.cycles.device = 'GPU'
    # Refresh devices
    prefs.preferences.get_devices()

scene.cycles.samples = 256
scene.cycles.use_denoising = True
scene.render.resolution_x = 3840
scene.render.resolution_y = 2160
scene.render.filepath = output_path
scene.render.image_settings.file_format = 'PNG'
scene.render.image_settings.color_depth = '16'

print(f"Rendering {ply_path} -> {output_path} at {scene.render.resolution_x}x{scene.render.resolution_y}...")
bpy.ops.render.render(write_still=True)
print("Render complete!")
