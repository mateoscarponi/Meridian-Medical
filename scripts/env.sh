# Shared build environment for Windows (GNU toolchain).
#
# The user folder "C:\Users\Mateo G" contains a space, which breaks the MinGW tools
# (gcc's linker specs and dlltool both split paths on spaces). So:
#  - rustup/cargo are addressed through their 8.3 short paths (no spaces),
#  - Rust's bundled MinGW tools (gcc driver, ld, dlltool) come first on PATH,
#  - any external MinGW (e.g. WinLibs) is removed from PATH,
#  - build output goes to a target dir whose path has no spaces.
short() { cygpath -d "$1"; }
export RUSTUP_HOME="$(short "$HOME/.rustup")"
export CARGO_HOME="$(short "$HOME/.cargo")"
PROJECT_SHORT="$(short "$(pwd)")"
export CARGO_TARGET_DIR="$PROJECT_SHORT\target"
# Only dlltool is exposed on PATH (copied to a space-free folder). The gcc driver is NOT put on
# PATH, so rustc keeps using its self-contained linking mode with its own CRT objects.
SC="$(cygpath -u "$RUSTUP_HOME")/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/bin/self-contained"
TOOLS="$(cygpath -u "$PROJECT_SHORT")/target/tools"
mkdir -p "$TOOLS"
[ -f "$TOOLS/dlltool.exe" ] || cp "$SC/dlltool.exe" "$SC/libwinpthread-1.dll" "$TOOLS/"
# dlltool (used by `raw-dylib` crates such as windows-sys) shells out to the GNU assembler,
# which Rust does not bundle. Borrow `as` from an installed MinGW (WinLibs via winget).
if [ ! -f "$TOOLS/as.exe" ]; then
  W="$(ls -d "$LOCALAPPDATA"/Microsoft/WinGet/Packages/BrechtSanders.WinLibs*/mingw64/bin 2>/dev/null | head -1)"
  W="$(cygpath -u "$W")"
  if [ -n "$W" ] && [ -f "$W/as.exe" ]; then
    cp "$W/as.exe" "$W/libintl-8.dll" "$W/libiconv-2.dll" "$W/zlib1.dll" "$W/libzstd.dll" "$TOOLS/" 2>/dev/null || true
  else
    echo "warning: GNU assembler (as.exe) not found; install WinLibs or MSYS2 binutils" >&2
  fi
fi
export PATH="$(cygpath -u "$CARGO_HOME")/bin:$TOOLS:$(echo "$PATH" | tr ':' '\n' | grep -v -i winlibs | grep -v ucrt64 | paste -sd:)"

# C compiler for crates with C sources (e.g. `ring`, used by rustls/TLS): the installed MinGW gcc,
# called through its short path so its own install prefix has no spaces.
WINLIBS="$(ls -d "$LOCALAPPDATA"/Microsoft/WinGet/Packages/BrechtSanders.WinLibs*/mingw64/bin 2>/dev/null | head -1)"
if [ -n "$WINLIBS" ]; then
  WINLIBS_SHORT="$(cygpath -d "$WINLIBS")"
  export CC_x86_64_pc_windows_gnu="$WINLIBS_SHORT\gcc.exe"
  export AR_x86_64_pc_windows_gnu="$WINLIBS_SHORT\ar.exe"
fi

# Leptos view types nest deeply; give rustc a bigger stack (debug/test builds overflowed the default).
export RUST_MIN_STACK=268435456
