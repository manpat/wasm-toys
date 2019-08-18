bl_info = {
	"name": "Toy Scene Exporter",
	"author": "Patrick Monaghan",
	"description": "Exports scenes in a format that wasm-toys can eat",
	"category": "Export",
	"version": (0, 0, 1),
	"blender": (2, 80, 0),
}

import bpy

from bpy_extras.io_utils import ExportHelper
from bpy.props import StringProperty, BoolProperty

# import mathutils
import struct
import bmesh
# from bpy import context

# https://docs.python.org/3/library/struct.html

def swap_coords(co):
	assert len(co) == 3 or len(co) == 4

	if len(co) == 3:
		return [co.x, co.z, -co.y]
	else:
		return [co.x, co.z, -co.y, co.w]


def write_binary_string(out, s):
	assert len(s) < 256
	out.write(struct.pack('=B', len(s)))
	out.write(bytes(s, 'utf-8'))


class ExportToyScene(bpy.types.Operator, ExportHelper):
	"""Toy scene exporter"""
	bl_idname = "export.toy_scene"
	bl_label = "Export Toy Scene"

	filename_ext = ".toy"
	filter_glob: StringProperty(
		default="*.toy",
		options={'HIDDEN'},
		maxlen=255,  # Max internal buffer length, longer would be clamped.
	)

	debug_run: BoolProperty(
		name="Debug Run"
	)

	def execute(self, context):
		debug_run = self.debug_run

		fname = self.filepath
		if fname == "":
			self.report({'ERROR'}, "Empty filepath!")
			return {'CANCELLED'}

		if not fname.lower().endswith(".toy"):
			fname += ".toy"
			self.filepath += ".toy"

		self.scenes = []
		self.meshes = []
		self.entities = []
		self.mesh_ids = {}

		self.collect_scenes()

		for scene in self.scenes:
			self.collect_meshes(scene["raw"])
			self.collect_entities(scene)

		if debug_run:
			with open(fname, 'w') as out:
				out.write("TOY")
				out.write("\n\nScenes\n")
				out.write(str(self.scenes))
				out.write("\n\nMeshes\n")
				out.write(str(self.meshes))
				out.write("\n\nMeshIDs\n")
				out.write(str(self.mesh_ids))
				out.write("\n\nEntities\n")
				out.write(str(self.entities))

			return {'FINISHED'}


		with open(fname, 'wb') as out:
			out.write(b"TOY") # Magic
			out.write(struct.pack('=B', 1)) # Version

			out.write(struct.pack('=H', len(self.meshes)))
			for m in self.meshes:
				num_vertices = len(m['vertices'])
				num_triangles = len(m['triangles']) / 3

				# WebGL 1 only supports 16b element arrays
				assert num_vertices < 65536

				out.write(b"MESH")
				out.write(struct.pack('=I', num_vertices))
				for v in m['vertices']:
					out.write(struct.pack('=fff', v))

				if num_vertices < 256:
					pack_str = '=B'
				else:
					pack_str = '=H'
				
				out.write(struct.pack('=I', num_triangles))
				for t in m['triangles']:
					out.write(struct.pack(pack_str, t))

			out.write(struct.pack('=H', len(self.entities)))
			for e in self.entities:
				out.write(b"ENTY")
				write_binary_string(out, e['name'])

				out.write(struct.pack('=fff', *e['position']))
				out.write(struct.pack('=ffff', *e['rotation']))
				out.write(struct.pack('=fff', *e['scale']))
				out.write(struct.pack('=H', e['mesh_id']))

		return {'FINISHED'}


	def collect_scenes(self):
		for scene in bpy.data.scenes:
			self.scenes.append({
				"name": scene.name,
				"raw": scene,
				"objects": scene.objects[:],
				"entities": []
			})


	def collect_meshes(self, scene):
		depsgraph = bpy.context.evaluated_depsgraph_get()

		for obj in scene.objects:
			if obj.type == 'MESH':
				odata = obj.data
				if odata.name in self.mesh_ids: continue

				bm = bmesh.new()
				bm.from_object(obj, depsgraph, deform=True)
				bmesh.ops.triangulate(bm, faces=bm.faces)

				bm.verts.ensure_lookup_table()
				bm.faces.ensure_lookup_table()

				for f in bm.faces:
					assert len(f.loops) == 3

				verts = []
				layers = bm.loops.layers.color.items()

				for face in bm.faces:
					for loop in face.loops:
						vert = { 'pos': loop.vert.co, 'layers': [] }

						for _, layer_id in layers:
							vert['layers'].append(loop[layer_id])

						verts.append(vert)


				deduped_verts = []

				def vert_index(va):
					for i, vb in enumerate(deduped_verts):
						if va['pos'] != vb['pos']:
							continue

						for la, lb in zip(va['layers'], vb['layers']):
							if la != lb:
								break
						else:
							return i

					deduped_verts.append(va)
					return len(deduped_verts)-1

				ts = [vert_index(v) for v in verts]

				vert_positions = [swap_coords(v['pos']) for v in deduped_verts]
				layer_data = [
					(name, [v['layers'][i].copy() for v in deduped_verts])
					for i, (name, _) in enumerate(layers)
				]

				# TODO: normal data
				# TODO: uv data

				mesh = {
					'vertices': vert_positions,
					'triangles': ts,
					'extra_data': layer_data,
				}

				bm.free()

				self.meshes.append(mesh)
				self.mesh_ids[odata.name] = len(self.meshes) # ids start at 1


	def collect_entities(self, scene):
		for obj in scene['raw'].objects:
			mesh_id = 0
			if obj.data:
				mesh_id = self.mesh_ids.get(obj.data.name, 0)

			# TODO: object type
			# TODO: collections
			# TODO: tags

			scale = obj.scale
			scale = [scale.x, scale.z, scale.y]

			data = {
				'name': obj.name,
				'mesh_id': mesh_id,

				'position': swap_coords(obj.location.xyz),
				'rotation': swap_coords(obj.rotation_euler.to_quaternion()), # This okay so long as handedness stays the same
				'scale': scale,
			}

			self.entities.append(data)
			scene['entities'].append(len(self.entities)) # entity IDs start at 1


def menu_func(self, context):
	self.layout.operator_context = 'INVOKE_DEFAULT'
	self.layout.operator(ExportToyScene.bl_idname, text="Toy Scene (.toy)")

# Register and add to the file selector
def register():
	bpy.utils.register_class(ExportToyScene)
	bpy.types.TOPBAR_MT_file_export.append(menu_func)

def unregister():
	bpy.utils.unregister_class(ExportToyScene)
	bpy.types.TOPBAR_MT_file_export.remove(menu_func)

if __name__ == '__main__':
	register()
	bpy.ops.export.toy_scene('INVOKE_DEFAULT')