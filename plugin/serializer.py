
import struct
# https://docs.python.org/3/library/struct.html


class Serializer:
	def __init__(self, out, debug):
		self.out_stack = [out]
		self.out = out
		self.debug = debug

	def debug_write(self, s):
		indent = "  " * (len(self.out_stack)-1)
		self.out.write(bytes(f"{indent}{s}\n", 'utf-8'))

	def write_raw(self, fmt, *args):
		if self.debug:
			if len(args) > 1:
				s = ', '.join([str(a) for a in args])
				self.debug_write(f"[{s}]")
			else:
				self.debug_write(str(*args))
		else:
			self.out.write(struct.pack(fmt, *args))

	def write_string(self, s):
		if self.debug:
			self.debug_write(f"'{s}'")
		else:
			assert len(s) < 256
			self.write_u8(len(s))
			self.out.write(bytes(s, 'utf-8'))

	def write_magic_number(self, version):
		self.out.write(b"TOY")
		self.write_u8(version)

	def write_tag(self, tag):
		assert len(tag) == 4
		if self.debug:
			self.debug_write(f"<{tag}>")
		else:
			self.out.write(bytes(tag, 'utf-8'))

	def write_v3(self, v1, v2, v3):
		self.write_raw('=fff', v1, v2, v3)

	def write_v4(self, v1, v2, v3, v4):
		self.write_raw('=ffff', v1, v2, v3, v4)

	def write_u8(self, v):
		self.write_raw('=B', v)

	def write_u16(self, v):
		self.write_raw('=H', v)

	def write_u32(self, v):
		self.write_raw('=I', v)


	def start_section(self, tag):
		section_buff = Buffer(tag)
		self.out_stack.append(section_buff)
		self.out = section_buff

	def end_section(self):
		section_buff = self.out_stack.pop()
		assert isinstance(section_buff, Buffer)
		assert len(self.out_stack) > 0

		self.out = self.out_stack[-1]

		self.write_tag(section_buff.tag)
		self.write_u32(len(section_buff.data))
		self.out.write(bytes(section_buff.data))


class Buffer:
	def __init__(self, tag):
		self.data = []
		self.tag = tag

	def write(self, bs):
		self.data.extend(bs)