# Meson build file for xv6-rust
project('xv6-rust', ['c', 'rust', 'cpp'], default_options : ['buildtype=debug'])

# Compiler flags for 32-bit kernel code
c_args = [
  '-m32',
  '-fno-pic',
  '-fno-builtin',
  '-fno-strict-aliasing',
  '-fno-omit-frame-pointer',
  '-Wall',
  '-Werror'
]

asm_args = ['-m32']
link_args = ['-m32']

# Source files translated from Makefile OBJS list
c_sources = [
  'bio.c',
  'console.c',
  'exec.c',
  'file.c',
  'fs.c',
  'ide.c',
  'ioapic.c',
  'kalloc.c',
  'lapic.c',
  'log.c',
  'main.c',
  'mp.c',
  'picirq.c',
  'pipe.c',
  'proc.c',
  'sleeplock.c',
  'spinlock.c',
  'swtch.S',
  'syscall.c',
  'sysfile.c',
  'trapasm.S',
  'trap.c',
  'vectors.S',
  'vm.c'
]

# Build the Rust static library using cargo
subdir('src')

kernel = executable(
  'kernel',
  c_sources,
  include_directories : include_directories('.'),
  c_args : c_args,
  link_args : link_args,
  dependencies : [rust_lib],
  link_depends : 'kernel.ld'
)
