#!/bin/bash

function process() {
	local blend_path=$1
	local output_path=${blend_path%.*}
	local display_name=${output_path#./src/bin/*}

	echo "Reprocessing $display_name ($blend_path)..."

	local expr="import bpy; bpy.ops.export.toy_scene(filepath='$output_path', debug_run=False)"
	blender --background "$blend_path" --python-expr "$expr"
}


for blend in $(find . -iname "*.blend"); do
	process $blend
done
