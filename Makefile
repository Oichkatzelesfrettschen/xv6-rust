###############################################################################
#  Meson build manifest for xv6-pdx-rust                                      #
#  –– targets real 80386 SX-through-Core-SSSE3 CPUs, with CPUID hotpaths       #
###############################################################################

project('xv6-pdx', ['c', 'asm', 'rust'],
        version: '0.1.0',
        default_options: [
          'c_std=c99',          # xv6 is C89 but GCC treats C99 as superset.
          'buildtype=debugoptimized',
          'b_lto=true',         # link-time optimisation but still debuggable
          'b_asneeded=true',    # dead-strip libs
          'b_staticpic=false',  # emit flat .text for a bare-metal binary
        ])

###############################################################################
#  ── 1. Build-time options mirroring the old Makefile conditionals ───────── #
###############################################################################
option('cs333_project',   type: 'integer',  min: 0, max: 5,
       description: 'Select CS333 assignment stage (0-5)',
       value: 0)

option('print_syscalls',  type: 'boolean',
       description: 'Enable syscall trace printk',
       value: false)

option('cpu_tier', type: 'combo',
       choices: ['386','486','p5','p5-mmx','p6-sse','p6-sse2','core-ssse3'],
       description: 'Pick ISA baseline used for the C objects',
       value: '386')

###############################################################################
#  ── 2. Compiler + linker arguments derived from tier + project flags ────── #
###############################################################################
cc   = meson.get_compiler('c')
as  = meson.get_compiler('asm')
rust = meson.get_compiler('rust')   # Meson ≥0.66 understands rustc :contentReference[oaicite:0]{index=0}

base_cflags = [
  '-fno-pic', '-static', '-fno-builtin', '-fno-strict-aliasing',
  '-pipe', '-ffunction-sections', '-fdata-sections',
  '-Wall', '-MD', '-ggdb', '-m32', '-Werror', '-Wno-array-bounds',
  '-fno-omit-frame-pointer',
]

# stack-protector probe (copied from old Makefile)
if cc.has_argument('-fno-stack-protector')
  base_cflags += ['-fno-stack-protector']
endif

# ISA-specific tuning
cpu_map = {
  '386'        : ['-march=i386',  '-mtune=i386'],
  '486'        : ['-march=i486',  '-mtune=i486'],
  'p5'         : ['-march=pentium','-mtune=pentium'],
  'p5-mmx'     : ['-march=pentium-mmx','-mtune=pentium-mmx','-mmmx'],
  'p6-sse'     : ['-march=pentium3','-msse','-mmmx'],
  'p6-sse2'    : ['-march=pentium4','-msse','-msse2','-mmmx'],
  'core-ssse3' : ['-march=core2','-msse','-msse2','-mssse3','-mmmx'],
}
base_cflags += cpu_map[get_option('cpu_tier')]       # vector-safe tuning :contentReference[oaicite:1]{index=1}

# CS333 feature macros
cs333_flags = ['-DPDX_XV6']                          # always on
pj = get_option('cs333_project')
if pj >= 1         ; cs333_flags += ['-DCS333_P1']          ; endif
if pj >= 2         ; cs333_flags += ['-DUSE_BUILTINS','-DCS333_P2']; endif
if pj >= 3         ; cs333_flags += ['-DCS333_P3P4']        ; endif
if pj >= 5         ; cs333_flags += ['-DCS333_P5']          ; endif
if get_option('print_syscalls')
                   cs333_flags += ['-DPRINT_SYSCALLS']      ; endif

add_project_arguments(cc.get_supported_arguments(base_cflags), language: 'c')
add_project_arguments(cs333_flags, language: 'c')           # Meson merges safely :contentReference[oaicite:2]{index=2}

###############################################################################
#  ── 3. Source lists (straight from Makefile) ─────────────────────────────── #
###############################################################################
srcs = files(
 'bio.c','console.c','exec.c','file.c','fs.c','ide.c','ioapic.c',
 'kalloc.c','lapic.c','log.c','main.c','mp.c','picirq.c','pipe.c',
 'proc.c','sleeplock.c','spinlock.c','swtch.S','syscall.c','sysfile.c',
 'trapasm.S','trap.c','vectors.S','vm.c',
)

###############################################################################
#  ── 4. Rust kernel library via Cargo + unstable build-std ────────────────── #
###############################################################################
cargo   = find_program('cargo', required: true)
rustlib = custom_target('libxv6.a',
  output: 'libxv6.a',
  build_by_default: true,
  command: [
    cargo, 'build',
      '-Z', 'build-std=core,alloc,compiler_builtins',
      '-Z', 'build-std-features=compiler-builtins-mem',
      '--manifest-path', meson.project_source_root() / 'rust' / 'Cargo.toml',
      '--target', meson.project_source_root() / 'i386.json',
      '--release',
  ],
  install: false,
)
# Meson knows how to pass a staticlib into later link steps :contentReference[oaicite:3]{index=3}

###############################################################################
#  ── 5. Link the monolithic kernel ELF ────────────────────────────────────── #
###############################################################################
ldscript = files('kernel.ld')
entry    = files('entry.S')

kernel = executable('kernel', [entry, srcs, rustlib],
  link_args: [
    '-nostdlib', '-T', ldscript,
    '-Wl,--gc-sections',
    '-Wl,-N',          # no page alignment, XV6 loads via bootloader
  ],
  link_depends: ldscript,
  install: false,
)

###############################################################################
#  ── 6. Bootloader, initcode & image assembly (dd, objcopy) ───────────────── #
###############################################################################
bootblock = custom_target('bootblock',
  input  : ['bootasm.S','bootmain.c'],
  output : 'bootblock',
  command: [
    cc, '-Os', '-fno-pic','-nostdinc','-I.', '-c', '@INPUT1@',
    '&&',
    cc, '-fno-pic','-nostdinc','-I.', '-c', '@INPUT0@',
    # LD-step
    '&&',
    'ld', '-N', '-e', 'start', '-Ttext', '0x7C00',
          '-o', 'bootblock.o', 'bootasm.o', 'bootmain.o',
    '&&', 'objcopy', '-O', 'binary', '-j', '.text', 'bootblock.o', '@OUTPUT@',
    '&&', meson.project_source_root() / 'sign.pl', '@OUTPUT@',
  ],
)

###############################################################################
#  ── 7. Disk image stitching — uses dd, controlled via Meson script() ─────── #
###############################################################################
dd = find_program('dd')
img = custom_target('xv6.img',
  input  : [bootblock, kernel],
  output : 'xv6.img',
  command: [
    dd, 'if=/dev/zero', 'of=@OUTPUT@', 'count=10000',
    '&&', dd, 'if=@INPUT0@', 'of=@OUTPUT@', 'conv=notrunc',
    '&&', dd, 'if=@INPUT1@', 'of=@OUTPUT@', 'seek=1', 'conv=notrunc',
  ],
)

###############################################################################
#  ── 8. QEMU launch helper (meson test harness) ───────────────────────────── #
###############################################################################
qemu = find_program(['qemu-system-i386','qemu-system-x86_64','qemu'], required: true)  # auto-detect :contentReference[oaicite:4]{index=4}

test('run-image',
     qemu,
     args: ['-serial', 'mon:stdio',
            '-drive', 'file=' + img.full_path() + ',index=0,media=disk,format=raw',
            '-smp', '2', '-m', '512'],
     timeout: 0,           # interactive; Ctrl-C to quit
     suite: 'runtime')

###############################################################################
#  ── 9. Install stanza (optional artifacts) ───────────────────────────────── #
###############################################################################
install_data(img, install_dir: get_option('prefix'))

###############################################################################
#  End of meson.build                                                         #
###############################################################################
