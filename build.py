#!/usr/bin/env python3

import datetime as dt
import itertools
import os
import platform
import re
import shutil
import subprocess
import sys
import sysconfig
from pathlib import Path



# Platform constants
IS_LINUX = platform.system() == "Linux"
IS_MACOS = platform.system() == "Darwin"
IS_WINDOWS = platform.system() == "Windows"
IS_ARM64 = platform.machine() in ("arm64", "aarch64")


# The Rust toolchain to use for builds
RUSTUP_TOOLCHAIN = os.getenv("RUSTUP_TOOLCHAIN", "stable")
# The Cargo build mode
BUILD_MODE = os.getenv("BUILD_MODE", "release")
# If COPY_TO_SOURCE is enabled, copy built *.so files back into the source tree
COPY_TO_SOURCE = os.getenv("COPY_TO_SOURCE", "true").lower() == "true"
# Force stripping of debug symbols even in non-release builds
FORCE_STRIP = os.getenv("FORCE_STRIP", "false").lower() == "true"
# If dry run only print the commands that would be executed
DRY_RUN = bool(os.getenv("DRY_RUN", ""))

# Precision mode configuration
# https://nautilustrader.io/docs/nightly/getting_started/installation#precision-mode
HIGH_PRECISION = os.getenv("HIGH_PRECISION", "true").lower() == "true"
if IS_WINDOWS and HIGH_PRECISION:
    print(
        "Warning: high-precision mode not supported on Windows (128-bit integers unavailable)\nForcing standard-precision (64-bit) mode",
    )
    HIGH_PRECISION = False

################################################################################
#  RUST BUILD
################################################################################

USE_SCCACHE = "sccache" in os.environ.get("CC", "") or "sccache" in os.environ.get("CXX", "")
if USE_SCCACHE:
    os.environ["RUSTC_WRAPPER"] = "sccache"
    os.environ["CARGO_INCREMENTAL"] = "0"

if IS_LINUX:
    # Use clang as the default compiler
    os.environ["CC"] = "sccache clang" if USE_SCCACHE else "clang"
    os.environ["CXX"] = "sccache clang++" if USE_SCCACHE else "clang++"
    os.environ["LDSHARED"] = "clang -shared"

if IS_MACOS and IS_ARM64:
    os.environ["ARCHFLAGS"] = "-arch arm64"
    os.environ["CFLAGS"] = f"{os.environ.get('CFLAGS', '')} -arch arm64"
    os.environ["LDFLAGS"] = f"{os.environ.get('LDFLAGS', '')} -arch arm64 -w"

if IS_LINUX and IS_ARM64:
    os.environ["CFLAGS"] = f"{os.environ.get('CFLAGS', '')} -fPIC"
    os.environ["LDFLAGS"] = f"{os.environ.get('LDFLAGS', '')} -fPIC"

    python_lib_dir = os.environ.get("PYTHON_LIB_DIR")
    python_version = ".".join(platform.python_version_tuple()[:2])  # e.g. "3.12"

    if python_lib_dir:
        print(f"Setting RUSTFLAGS to link with Python {python_version} in {python_lib_dir}")
        rustflags = f"{os.environ.get('RUSTFLAGS', '')} -C link-arg=-L{python_lib_dir} -C link-arg=-lpython{python_version}"
        os.environ["RUSTFLAGS"] = rustflags

if IS_WINDOWS:
    # Linker error 1181
    # https://docs.microsoft.com/en-US/cpp/error-messages/tool-errors/linker-tools-error-lnk1181?view=msvc-170&viewFallbackFrom=vs-2019
    RUST_LIB_PFX = ""
    RUST_STATIC_LIB_EXT = "lib"
    RUST_DYLIB_EXT = "dll"
    # Rust target is typically x86_64-pc-windows-msvc; C deps (ring, zstd-sys, aws-Lc-sys) need MSVC's cl.exe, not cc/g++/clang.
    # Unset CC/CXX compilers so the build uses the default MSVC toolchain.
    os.environ.pop("CC", None)
    os.environ.pop("CXX", None)
elif IS_MACOS:
    RUST_LIB_PFX = "lib"
    RUST_STATIC_LIB_EXT = "a"
    RUST_DYLIB_EXT = "dylib"
else:  # Linux
    RUST_LIB_PFX = "lib"
    RUST_STATIC_LIB_EXT = "a"
    RUST_DYLIB_EXT = "so"

CARGO_TARGET_DIR = os.environ.get("CARGO_TARGET_DIR", Path.cwd() / "target")
CARGO_BUILD_TARGET = os.environ.get("CARGO_BUILD_TARGET", "")

# Determine the profile directory name
if BUILD_MODE == "release":
    profile_dir = "release"
elif BUILD_MODE == "ci-pr":
    profile_dir = "ci-pr-wheel"
elif BUILD_MODE == "debug-pyo3":
    profile_dir = "debug-pyo3"
else:
    profile_dir = "debug"

CARGO_TARGET_DIR = Path(CARGO_TARGET_DIR) / CARGO_BUILD_TARGET / profile_dir

RUST_LIB_PATHS: list[Path] = [
    CARGO_TARGET_DIR / f"{RUST_LIB_PFX}nautilus_backtest.{RUST_STATIC_LIB_EXT}",
    CARGO_TARGET_DIR / f"{RUST_LIB_PFX}nautilus_common.{RUST_STATIC_LIB_EXT}",
    CARGO_TARGET_DIR / f"{RUST_LIB_PFX}nautilus_core.{RUST_STATIC_LIB_EXT}",
    CARGO_TARGET_DIR / f"{RUST_LIB_PFX}nautilus_model.{RUST_STATIC_LIB_EXT}",
    CARGO_TARGET_DIR / f"{RUST_LIB_PFX}nautilus_persistence.{RUST_STATIC_LIB_EXT}",
]
RUST_LIBS: list[str] = [str(path) for path in RUST_LIB_PATHS]


def _set_feature_flags() -> list[str]:
    feature_list = [
        "arrow",
        "extension-module",
        "ffi",
        "postgres",
        "python",
        "tracing-bridge",
    ]

    if HIGH_PRECISION:
        feature_list.append("high-precision")

    feature_list.sort()

    flags = ["--no-default-features", "--features", ",".join(feature_list)]

    return flags


def _build_rust_libs() -> None:
    print("Compiling Rust libraries...")

    try:
        # Build the Rust libraries using Cargo
        if RUSTUP_TOOLCHAIN not in ("stable", "nightly"):
            raise ValueError(f"Invalid `RUSTUP_TOOLCHAIN` '{RUSTUP_TOOLCHAIN}'")

        needed_crates = [
            "nautilus-backtest",
            "nautilus-common",
            "nautilus-core",
            "nautilus-model",
            "nautilus-persistence",
            "nautilus-pyo3",
        ]

        if BUILD_MODE == "release":
            build_options = ["--release"]
            # Only pass '-s' at link time on Linux. On macOS this flag is obsolete
            # and may cause failures with recent toolchains. Cargo already performs
            # symbol stripping per profile, and we post-strip where applicable.
            if IS_LINUX:
                existing_rustflags = os.environ.get("RUSTFLAGS", "")
                os.environ["RUSTFLAGS"] = f"{existing_rustflags} -C link-arg=-s"
        elif BUILD_MODE == "ci-pr":
            build_options = ["--profile", "ci-pr-wheel"]
        elif BUILD_MODE == "debug-pyo3":
            build_options = ["--profile", "debug-pyo3"]
        else:
            build_options = []

        features = _set_feature_flags()

        cmd_args = [
            "cargo",
            "build",
            "--lib",
            *itertools.chain.from_iterable(("-p", p) for p in needed_crates),
            *build_options,
            *features,
        ]

        if RUSTUP_TOOLCHAIN == "nightly":
            cmd_args.insert(1, "+nightly")

        print(" ".join(cmd_args))

        subprocess.run(
            cmd_args,
            check=True,
        )
    except subprocess.CalledProcessError as e:
        raise RuntimeError(
            f"Error running cargo: {e}",
        ) from e


def _copy_rust_dylibs_to_project() -> None:
    # https://pyo3.rs/latest/building-and-distribution#manual-builds
    ext_suffix = sysconfig.get_config_var("EXT_SUFFIX")
    src = Path(CARGO_TARGET_DIR) / f"{RUST_LIB_PFX}nautilus_pyo3.{RUST_DYLIB_EXT}"
    dst = Path("nautilus_trader/core") / f"nautilus_pyo3{ext_suffix}"
    shutil.copyfile(src=src, dst=dst)

    print(f"Copied {src} to {dst}")


def _get_nautilus_version() -> str:
    with open("pyproject.toml", encoding="utf-8") as f:
        pyproject_content = f.read().strip()
    if not pyproject_content:
        raise ValueError("pyproject.toml is empty or not properly formatted")

    version_match = re.search(r'version\s*=\s*"(.*?)"', pyproject_content)
    if not version_match:
        raise ValueError("Version not found in pyproject.toml")

    return version_match.group(1)


def _get_clang_version() -> str:
    try:
        result = subprocess.run(
            ["clang", "--version"],
            check=True,
            capture_output=True,
        )
        output = (
            result.stdout.decode()
            .splitlines()[0]
            .removeprefix("Apple ")
            .removeprefix("Ubuntu ")
            .removeprefix("clang version ")
        )
        return output
    except (subprocess.CalledProcessError, FileNotFoundError) as e:
        err_msg = str(e) if isinstance(e, FileNotFoundError) else e.stderr.decode()
        raise RuntimeError(
            f"You are installing from source which requires the Clang compiler to be installed.\nError running clang: {err_msg}",
        ) from e


def _get_rustc_version() -> str:
    try:
        result = subprocess.run(
            ["rustc", "--version"],
            check=True,
            capture_output=True,
        )
        output = result.stdout.decode().lstrip("rustc ").strip()
        return output
    except (subprocess.CalledProcessError, FileNotFoundError) as e:
        err_msg = str(e) if isinstance(e, FileNotFoundError) else e.stderr.decode()
        raise RuntimeError(
            "You are installing from source which requires the Rust compiler to be installed.\n"
            "Find more information at https://www.rust-lang.org/tools/install\n"
            f"Error running rustc: {err_msg}",
        ) from e


def _ensure_windows_python_import_lib() -> None:
    """
    Ensure that the *t* suffixed Python import library exists on Windows.

    On some official CPython Windows builds the import library is named
    ``pythonXY.lib`` (for example ``python313.lib``). However, when building
    C-extensions ``distutils``/``setuptools`` may ask the MSVC linker for the
    file ``pythonXYt.lib`` - note the additional *t* suffix. The *t* variant
    historically referred to a *thread-safe* build but is no longer shipped.

    When the file is missing the linker exits with
    ``LINK : fatal error LNK1104: cannot open file 'pythonXYt.lib'`` which
    breaks the CI build on Windows. To work around this we simply create a
    copy of the existing import library with the expected name **before** the
    extension build starts.

    """
    if not IS_WINDOWS:
        return

    try:
        # The virtual environment as well as the base installation may both
        # participate in the link search path.  Attempt the fix in both
        # locations to maximise the chance of success.
        candidate_roots = {Path(sys.base_prefix), Path(sys.prefix)}

        # Example: for Python 3.13 -> '313'
        major, minor, *_ = platform.python_version_tuple()
        version_compact = f"{major}{minor}"

        for root in candidate_roots:
            libs_dir = root / "libs"
            if not libs_dir.exists():
                continue

            src = libs_dir / f"python{version_compact}.lib"
            dst = libs_dir / f"python{version_compact}t.lib"

            if src.exists() and not dst.exists():
                print(
                    f"Creating missing Windows import lib {dst} (copying from {src})",
                )
                shutil.copyfile(src, dst)
    except Exception as e:  # pragma: no cover - defensive
        # Never fail the build because of this helper, just show the warning
        print(f"Warning: failed to create *t* suffixed Python import library: {e}")


def _strip_unneeded_symbols() -> None:
    try:
        print("Stripping unneeded symbols from binaries...")
        total_before = 0
        total_after = 0

        for so in itertools.chain(Path("nautilus_trader").rglob("*.so")):
            size_before = so.stat().st_size
            total_before += size_before

            if IS_LINUX:
                strip_cmd = ["strip", "--strip-all", "-R", ".comment", "-R", ".note", so]
            elif IS_MACOS:
                strip_cmd = ["strip", "-x", so]
            else:
                raise RuntimeError(f"Cannot strip symbols for platform {platform.system()}")
            subprocess.run(
                strip_cmd,  # type: ignore [arg-type]
                check=True,
                capture_output=True,
            )

            size_after = so.stat().st_size
            total_after += size_after

        if total_before > 0:
            reduction = (1 - total_after / total_before) * 100
            print(
                f"Stripped binaries: {total_before / 1024 / 1024:.1f}MB -> {total_after / 1024 / 1024:.1f}MB ({reduction:.1f}% reduction)",
            )
    except subprocess.CalledProcessError as e:
        raise RuntimeError(f"Error when stripping symbols.\n{e}") from e


def show_rustanalyzer_settings() -> None:
    """
    Show appropriate vscode settings for the build.
    """
    import json

    # Set environment variables
    settings: dict[str, object] = {}

    for key in [
        "rust-analyzer.check.extraEnv",
        "rust-analyzer.runnables.extraEnv",
        "rust-analyzer.cargo.features",
    ]:
        settings[key] = {
            "CC": os.environ["CC"],
            "CXX": os.environ["CXX"],
            "VIRTUAL_ENV": os.environ["VIRTUAL_ENV"],
        }

    # Set features
    features = _set_feature_flags()
    if features[0] == "--all-features":
        settings["rust-analyzer.cargo.features"] = "all"
        settings["rust-analyzer.check.features"] = "all"
    else:
        settings["rust-analyzer.cargo.features"] = features[1].split(",")
        settings["rust-analyzer.check.features"] = features[1].split(",")

    print("Set these rust analyzer settings in .vscode/settings.json")
    print(json.dumps(settings, indent=2))


def _ensure_local_editable_pth() -> None:
    # Make the v1 source tree (with its built `.so` files) importable from any cwd
    # after `make build`, without requiring a follow-up `uv sync`. This closes the
    # gap where a bare `make build` leaves the venv unable to resolve `nautilus_trader`
    # from a tempdir (e.g. the docs tutorial subprocess tests).
    if not COPY_TO_SOURCE:
        return
    if sys.prefix == sys.base_prefix:
        return  # Not running inside a venv (e.g. PEP 517 build host)

    site_packages = Path(sysconfig.get_paths()["purelib"])
    try:
        site_packages.relative_to(Path(sys.prefix))
    except ValueError:
        return  # `purelib` is outside the active venv prefix

    if not site_packages.is_dir():
        return

    repo_root = Path(__file__).resolve().parent
    pth_file = site_packages / "nautilus-trader-local.pth"
    contents = f"{repo_root}\n"

    if pth_file.is_file() and pth_file.read_text() == contents:
        return

    pth_file.write_text(contents)
    print(f"Wrote local editable .pth: {pth_file}")


def build() -> None:
    """
    Construct the extensions and distribution.
    """
    _ensure_windows_python_import_lib()
    _build_rust_libs()
    # Allow skipping Rust dylib copy in constrained environments
    if not os.getenv("SKIP_RUST_DYLIB_COPY"):
        _copy_rust_dylibs_to_project()

    if (BUILD_MODE == "release" or FORCE_STRIP) and (IS_LINUX or IS_MACOS):
        # Strip symbols for release builds or when forced
        _strip_unneeded_symbols()

    _ensure_local_editable_pth()


def print_env_var_if_exists(key: str) -> None:
    value = os.environ.get(key)
    if value is not None:
        print(f"{key}={value}")


if __name__ == "__main__":
    print("\033[36m")
    print("=====================================================================")
    print(f"Nautilus Builder {_get_nautilus_version()}")
    print("=====================================================================\033[0m")
    print(f"System: {platform.system()} {platform.machine()}")
    print(f"Clang:  {_get_clang_version()}")
    print(f"Rust:   {_get_rustc_version()}")
    print(f"Python: {platform.python_version()} ({sys.executable})")

    print(f"\nRUSTUP_TOOLCHAIN={RUSTUP_TOOLCHAIN}")
    print(f"BUILD_MODE={BUILD_MODE}")
    print(f"HIGH_PRECISION={HIGH_PRECISION}")
    print(f"COPY_TO_SOURCE={COPY_TO_SOURCE}")
    print(f"FORCE_STRIP={FORCE_STRIP}")
    print_env_var_if_exists("CC")
    print_env_var_if_exists("CXX")
    print_env_var_if_exists("LDSHARED")
    print_env_var_if_exists("CFLAGS")
    print_env_var_if_exists("LDFLAGS")
    print_env_var_if_exists("LD_LIBRARY_PATH")
    print_env_var_if_exists("PYO3_PYTHON")
    print_env_var_if_exists("PYTHONHOME")
    print_env_var_if_exists("RUSTFLAGS")
    print_env_var_if_exists("DRY_RUN")
    print_env_var_if_exists("RUSTC_WRAPPER")
    print_env_var_if_exists("CARGO_INCREMENTAL")

    if DRY_RUN:
        show_rustanalyzer_settings()
    else:
        print("\nStarting build...")
        ts_start = dt.datetime.now(dt.UTC)
        build()
        print(f"Build time: {dt.datetime.now(dt.UTC) - ts_start}")
        print("\033[32m" + "Build completed" + "\033[0m")
