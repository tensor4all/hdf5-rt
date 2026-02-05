#!/usr/bin/env python3
"""
Python-Rust HDF5 interoperability tests.

This script tests that:
1. Python can read HDF5 files created by Rust
2. Rust can read HDF5 files created by Python

Requirements:
    pip install h5py numpy

Usage:
    python test_interop.py
"""

import ctypes
import os
import platform
import subprocess
import sys
import tempfile
from pathlib import Path

import h5py
import numpy as np


def get_hdf5_library_path() -> str:
    """Get the path to the HDF5 shared library used by h5py."""
    # Try to get it from h5py's internal info
    try:
        # h5py exposes the library path in newer versions
        if hasattr(h5py, "get_config"):
            config = h5py.get_config()
            if hasattr(config, "hdf5_dir") and config.hdf5_dir:
                hdf5_dir = Path(config.hdf5_dir)
                # Try common library locations
                for lib_dir in ["lib", "lib64"]:
                    for lib_name in _get_library_names():
                        lib_path = hdf5_dir / lib_dir / lib_name
                        if lib_path.exists():
                            return str(lib_path)
    except Exception:
        pass

    # Try to find from h5py's loaded library
    try:
        # Get the HDF5 library handle from h5py
        import h5py._hl.base

        # h5py loads HDF5, so we can find it via ctypes
        if hasattr(h5py, "_hdf5"):
            lib = h5py._hdf5
            if hasattr(lib, "_name"):
                return lib._name
    except Exception:
        pass

    # Platform-specific search
    system = platform.system()

    if system == "Darwin":
        # macOS: Check common locations
        search_paths = [
            "/opt/homebrew/lib/libhdf5.dylib",  # Apple Silicon Homebrew
            "/usr/local/lib/libhdf5.dylib",  # Intel Homebrew
            "/opt/local/lib/libhdf5.dylib",  # MacPorts
        ]
        # Also check conda environment
        if "CONDA_PREFIX" in os.environ:
            search_paths.insert(
                0, os.path.join(os.environ["CONDA_PREFIX"], "lib", "libhdf5.dylib")
            )

    elif system == "Linux":
        search_paths = [
            "/usr/lib/x86_64-linux-gnu/libhdf5.so",
            "/usr/lib/libhdf5.so",
            "/usr/lib64/libhdf5.so",
        ]
        if "CONDA_PREFIX" in os.environ:
            search_paths.insert(
                0, os.path.join(os.environ["CONDA_PREFIX"], "lib", "libhdf5.so")
            )

    elif system == "Windows":
        search_paths = []
        if "CONDA_PREFIX" in os.environ:
            search_paths.append(
                os.path.join(os.environ["CONDA_PREFIX"], "Library", "bin", "hdf5.dll")
            )

    else:
        search_paths = []

    for path in search_paths:
        if os.path.exists(path):
            return path

    # Last resort: try to find via ldd/otool on h5py's binary
    try:
        h5py_path = h5py.__file__
        h5py_dir = os.path.dirname(h5py_path)

        # Find the compiled extension
        for fname in os.listdir(h5py_dir):
            if fname.endswith((".so", ".pyd", ".dylib")):
                ext_path = os.path.join(h5py_dir, fname)
                lib_path = _find_hdf5_from_binary(ext_path)
                if lib_path:
                    return lib_path
    except Exception:
        pass

    raise RuntimeError(
        "Could not find HDF5 library path. "
        "Please set HDF5_LIB environment variable."
    )


def _get_library_names() -> list:
    """Get platform-specific library names."""
    system = platform.system()
    if system == "Darwin":
        return ["libhdf5.dylib", "libhdf5.*.dylib"]
    elif system == "Windows":
        return ["hdf5.dll", "libhdf5.dll"]
    else:
        return ["libhdf5.so", "libhdf5.so.*"]


def _find_hdf5_from_binary(binary_path: str) -> str | None:
    """Find HDF5 library path from a binary's dependencies."""
    system = platform.system()

    try:
        if system == "Darwin":
            result = subprocess.run(
                ["otool", "-L", binary_path], capture_output=True, text=True
            )
            for line in result.stdout.split("\n"):
                if "libhdf5" in line.lower() or "hdf5" in line.lower():
                    # Extract path from otool output
                    path = line.strip().split()[0]
                    if os.path.exists(path):
                        return path

        elif system == "Linux":
            result = subprocess.run(
                ["ldd", binary_path], capture_output=True, text=True
            )
            for line in result.stdout.split("\n"):
                if "libhdf5" in line.lower():
                    parts = line.split("=>")
                    if len(parts) > 1:
                        path = parts[1].strip().split()[0]
                        if os.path.exists(path):
                            return path
    except Exception:
        pass

    return None


def get_project_root() -> Path:
    """Get the project root directory."""
    # This script is in tests/python/
    return Path(__file__).parent.parent.parent


def build_rust_binary() -> Path:
    """Build the Rust interop test binary."""
    project_root = get_project_root()

    print("Building Rust interop_test binary...")
    result = subprocess.run(
        [
            "cargo",
            "build",
            "--example",
            "interop_test",
        ],
        cwd=project_root / "hdf5",
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print("STDOUT:", result.stdout)
        print("STDERR:", result.stderr)
        raise RuntimeError(f"Failed to build Rust binary: {result.stderr}")

    # Find the built binary
    binary_name = "interop_test"
    if platform.system() == "Windows":
        binary_name += ".exe"

    binary_path = project_root / "target" / "debug" / "examples" / binary_name
    if not binary_path.exists():
        raise RuntimeError(f"Built binary not found at {binary_path}")

    print(f"Built: {binary_path}")
    return binary_path


def run_rust_binary(
    binary_path: Path, hdf5_lib: str, mode: str, file_path: Path
) -> subprocess.CompletedProcess:
    """Run the Rust interop test binary."""
    cmd = [
        str(binary_path),
        "--hdf5-lib",
        hdf5_lib,
        "--mode",
        mode,
        "--file",
        str(file_path),
    ]
    print(f"Running: {' '.join(cmd)}")
    return subprocess.run(cmd, capture_output=True, text=True)


def create_python_test_file(path: Path) -> None:
    """Create a test HDF5 file using Python/h5py."""
    print(f"Creating test file with Python: {path}")

    with h5py.File(path, "w") as f:
        # Create root attribute
        f.attrs["test_attr"] = "hello from julia/python"

        # Create 1D integer dataset
        f.create_dataset("integers", data=np.array([1, 2, 3, 4, 5], dtype=np.int64))

        # Create 2D float dataset (row-major, same as Rust expects)
        f.create_dataset(
            "matrix", data=np.array([[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]], dtype=np.float64)
        )

        # Create string dataset with variable-length strings
        dt = h5py.special_dtype(vlen=str)
        str_data = np.array(["foo", "bar", "baz"], dtype=object)
        f.create_dataset("strings", data=str_data, dtype=dt)

    print("  Created datasets: integers, matrix, strings")
    print("  Created attribute: test_attr")


def verify_rust_created_file(path: Path) -> None:
    """Verify a test HDF5 file created by Rust."""
    print(f"Verifying Rust-created file with Python: {path}")

    with h5py.File(path, "r") as f:
        # Verify root attribute
        attr_value = f.attrs["test_attr"]
        if isinstance(attr_value, bytes):
            attr_value = attr_value.decode("utf-8")
        assert attr_value == "hello from rust", f"Attribute mismatch: {attr_value}"
        print(f"  Attribute 'test_attr': {attr_value}")

        # Verify integer dataset
        int_data = f["integers"][:]
        expected_ints = np.array([10, 20, 30, 40, 50], dtype=np.int64)
        np.testing.assert_array_equal(int_data, expected_ints)
        print(f"  Dataset 'integers': {int_data}")

        # Verify float matrix
        float_data = f["matrix"][:]
        expected_floats = np.array(
            [[1.5, 2.5, 3.5], [4.5, 5.5, 6.5]], dtype=np.float64
        )
        np.testing.assert_array_almost_equal(float_data, expected_floats)
        print(f"  Dataset 'matrix': {float_data}")

        # Verify string dataset
        str_data = f["strings"][:]
        str_list = [s.decode("utf-8") if isinstance(s, bytes) else s for s in str_data]
        assert str_list == ["rust", "test", "data"], f"String mismatch: {str_list}"
        print(f"  Dataset 'strings': {str_list}")


def test_python_to_rust(binary_path: Path, hdf5_lib: str) -> bool:
    """Test that Rust can read files created by Python."""
    print("\n" + "=" * 60)
    print("TEST: Python -> Rust (Python writes, Rust reads)")
    print("=" * 60)

    with tempfile.NamedTemporaryFile(suffix=".h5", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        # Python creates the file
        create_python_test_file(tmp_path)

        # Rust reads and verifies
        result = run_rust_binary(binary_path, hdf5_lib, "read", tmp_path)
        print("Rust output:")
        print(result.stdout)
        if result.stderr:
            print("Rust stderr:")
            print(result.stderr)

        if result.returncode != 0:
            print("FAILED: Rust could not read Python-created file")
            return False

        if "SUCCESS" in result.stdout:
            print("PASSED: Rust successfully read Python-created file")
            return True
        else:
            print("FAILED: Rust did not report success")
            return False

    finally:
        if tmp_path.exists():
            tmp_path.unlink()


def test_rust_to_python(binary_path: Path, hdf5_lib: str) -> bool:
    """Test that Python can read files created by Rust."""
    print("\n" + "=" * 60)
    print("TEST: Rust -> Python (Rust writes, Python reads)")
    print("=" * 60)

    with tempfile.NamedTemporaryFile(suffix=".h5", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        # Rust creates the file
        result = run_rust_binary(binary_path, hdf5_lib, "write", tmp_path)
        print("Rust output:")
        print(result.stdout)
        if result.stderr:
            print("Rust stderr:")
            print(result.stderr)

        if result.returncode != 0:
            print("FAILED: Rust could not create file")
            return False

        # Python reads and verifies
        try:
            verify_rust_created_file(tmp_path)
            print("PASSED: Python successfully read Rust-created file")
            return True
        except AssertionError as e:
            print(f"FAILED: Verification error: {e}")
            return False
        except Exception as e:
            print(f"FAILED: Could not read file: {e}")
            return False

    finally:
        if tmp_path.exists():
            tmp_path.unlink()


def main():
    """Run all interoperability tests."""
    print("HDF5 Python-Rust Interoperability Tests")
    print("=" * 60)

    # Get HDF5 library path
    hdf5_lib = os.environ.get("HDF5_LIB")
    if not hdf5_lib:
        try:
            hdf5_lib = get_hdf5_library_path()
        except RuntimeError as e:
            print(f"ERROR: {e}")
            sys.exit(1)

    print(f"HDF5 library: {hdf5_lib}")

    # Build Rust binary
    try:
        binary_path = build_rust_binary()
    except RuntimeError as e:
        print(f"ERROR: {e}")
        sys.exit(1)

    # Run tests
    results = []

    results.append(("Python -> Rust", test_python_to_rust(binary_path, hdf5_lib)))
    results.append(("Rust -> Python", test_rust_to_python(binary_path, hdf5_lib)))

    # Summary
    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)

    all_passed = True
    for name, passed in results:
        status = "PASSED" if passed else "FAILED"
        print(f"  {name}: {status}")
        if not passed:
            all_passed = False

    if all_passed:
        print("\nAll tests passed!")
        sys.exit(0)
    else:
        print("\nSome tests failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()
