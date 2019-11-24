
import bpy

from bpy_extras.io_utils import ExportHelper
from bpy.props import StringProperty, BoolProperty
from .serializer import Serializer

# import mathutils
import bmesh
# from bpy import context

# bpy.ops.export.toy_scene(filepath="/home/patrick/Development/wasm-toys/src/bin/fish/main.toy")
# bpy.ops.export.toy_scene(filepath="/home/patrick/Development/wasm-toys/untitled.toy", debug_run=True)


def swap_coords(co):
	assert len(co) == 3 or len(co) == 4

	if len(co) == 3:
		return [co.x, co.z, -co.y]
	else:
		return [co.x, co.z, -co.y, co.w]


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
		bpy.context.evaluated_depsgraph_get()
		debug_run = self.debug_run

		fname = self.filepath
		if fname == "":
			self.report({'ERROR'}, "Empty filepath!")
			return {'CANCELLED'}

		if not fname.lower().endswith(".toy"):
			fname += ".toy"
			self.filepath += ".toy"

		self.entity_ids = {}
		self.mesh_ids = {}
		self.mesh_count = 0

		for scene in bpy.data.scenes:
			scene.view_layers[0].update() # to make sure they have a depsgraph

		with open(fname, 'wb') as out:
			ser = Serializer(out, debug_run)
			ser.write_magic_number(2)

			for s in bpy.data.scenes:
				for m in self.collect_meshes(s):
					self.write_mesh(ser, m)

				entities = []
				for e in self.collect_entities(s, entities):
					self.write_entity(ser, e)

				ser.start_section("SCNE")
				ser.write_string(s.name)

				ser.write_u32(len(entities))
				for e in entities:
					ser.write_u32(e)
				ser.end_section()


		return {'FINISHED'}


	def write_mesh(self, ser, mesh):
		num_vertices = len(mesh['vertices'])
		num_triangles = len(mesh['triangles']) // 3

		# WebGL 1 only supports 16b element arrays
		assert num_vertices < 65536
		assert len(mesh['triangles']) % 3 == 0

		ser.start_section("MESH")

		ser.write_u16(num_vertices)
		for v in mesh['vertices']:
			ser.write_v3(*v)
		
		ser.write_u16(num_triangles)
		if num_vertices < 256:
			for t in mesh['triangles']:
				ser.write_u8(t)
		else:
			for t in mesh['triangles']:
				ser.write_u16(t)

		ser.write_u8(len(mesh['extra_data']))
		for name, data in mesh['extra_data']:
			ser.write_tag("MDTA")
			ser.write_string(name)
			ser.write_u16(len(data))
			for el in data:
				ser.write_v4(*el)

		ser.end_section()


	def write_entity(self, ser, entity):
		ser.start_section("ENTY")
		ser.write_string(entity['name'])

		ser.write_v3(*entity['position'])
		ser.write_v4(*entity['rotation'])
		ser.write_v3(*entity['scale'])
		ser.write_u16(entity['mesh_id'])
		ser.end_section()


	def collect_meshes(self, scene):
		# depsgraph = scene.view_layers[0].depsgraph
		depsgraph = bpy.context.evaluated_depsgraph_get()

		for obj in scene.objects:
			if obj.type == 'MESH':
				odata = obj.data
				if odata in self.mesh_ids:
					continue

				bm = bmesh.new()
				bm.from_object(obj, depsgraph, deform=True)
				bmesh.ops.triangulate(bm, faces=bm.faces)

				bm.verts.ensure_lookup_table()
				bm.faces.ensure_lookup_table()

				for f in bm.faces:
					assert len(f.loops) == 3


				layers = bm.loops.layers.color.items()
				verts = []
				tris = []

				# TODO: find a faster way
				def vert_index(vpos, vlayers):
					for i, vb in enumerate(verts):
						if vpos != vb['pos']:
							continue

						for la, lb in zip(vlayers, vb['layers']):
							if la != lb:
								break
						else:
							return i

					verts.append({ 'pos': vpos, 'layers': vlayers })
					return len(verts)-1


				for face in bm.faces:
					for loop in face.loops:
						vlayers = [loop[layer_id] for _, layer_id in layers]
						tris.append(vert_index(loop.vert.co, vlayers))


				vert_positions = [swap_coords(v['pos']) for v in verts]
				layer_data = [
					(name, [v['layers'][i].copy() for v in verts])
					for i, (name, _) in enumerate(layers)
				]

				# TODO: normal data
				# TODO: uv data

				bm.free()

				self.mesh_count += 1
				self.mesh_ids[odata] = self.mesh_count # ids start at 1

				yield {
					'vertices': vert_positions,
					'triangles': tris,
					'extra_data': layer_data,
				}



	def collect_entities(self, scene, ent_list):
		for obj in scene.objects:
			entity_id = self.entity_ids.get(obj, None)
			if entity_id is not None:
				ent_list.append(entity_id)
				continue

			# entity ids are 1-indexed
			entity_id = len(self.entity_ids) + 1
			self.entity_ids[obj] = entity_id

			ent_list.append(entity_id)

			mesh_id = 0
			if obj.data:
				mesh_id = self.mesh_ids.get(obj.data, 0)

			# TODO: object type
			# TODO: collections
			# TODO: tags

			scale = obj.scale
			scale = [scale.x, scale.z, scale.y]

			yield {
				'name': obj.name,
				'entity_id': entity_id,
				'mesh_id': mesh_id,

				'position': swap_coords(obj.location.xyz),
				'rotation': swap_coords(obj.rotation_euler.to_quaternion()), # This okay so long as handedness stays the same
				'scale': scale,
			}




def menu_func(self, context):
	self.layout.operator_context = 'INVOKE_DEFAULT'
	self.layout.operator(ExportToyScene.bl_idname, text="Toy Scene (.toy)")