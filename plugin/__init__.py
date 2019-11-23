bl_info = {
	"name": "Toy Scene Utils",
	"author": "Patrick Monaghan",
	"description": "Exports scenes in a format that wasm-toys can eat",
	"category": "Development",
	"version": (0, 0, 1),
	"blender": (2, 80, 0),
}

import bpy
from . import exporter

# Register and add to the file selector
def register():
	bpy.utils.register_class(exporter.ExportToyScene)
	bpy.types.TOPBAR_MT_file_export.append(exporter.menu_func)

def unregister():
	bpy.utils.unregister_class(exporter.ExportToyScene)
	bpy.types.TOPBAR_MT_file_export.remove(exporter.menu_func)

if __name__ == '__main__':
	register()