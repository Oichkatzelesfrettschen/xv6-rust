# Build Rust library using cargo xbuild
cargo = find_program('cargo')
rust_lib = custom_target(
  'libxv6',
  output: 'libxv6.a',
  command: [cargo, 'xbuild', '--release', '--target', '../i386.json'],
  build_by_default: true
)
