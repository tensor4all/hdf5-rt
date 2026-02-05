#!/usr/bin/env julia

# Interoperability test between Julia HDF5.jl and Rust hdf5-rust
#
# This script tests that HDF5 files created by Julia can be read by Rust,
# and that HDF5 files created by Rust can be read by Julia.
#
# Usage:
#   julia --project=tests/julia tests/julia/test_interop.jl
#
# Prerequisites:
#   - Julia with HDF5.jl installed
#   - Cargo (Rust build tool)

using Test
using HDF5
import HDF5_jll

# h5_get_libversion location varies by HDF5.jl version
const h5_get_libversion = try
    # HDF5.jl 0.17+
    HDF5.API.h5_get_libversion
catch
    # Older versions
    HDF5.h5_get_libversion
end

# Get the HDF5 library path from HDF5_jll
function get_hdf5_lib_path()
    return HDF5_jll.libhdf5_path
end

# Get all JLL dependency library paths needed for dlopen
function get_jll_lib_paths()
    # HDF5_jll.LIBPATH_list contains all directories needed for dependencies
    # (Zlib_jll, libaec_jll, etc.)
    if isdefined(HDF5_jll, :LIBPATH_list)
        return HDF5_jll.LIBPATH_list
    else
        return [dirname(get_hdf5_lib_path())]
    end
end

# Get project root directory (two levels up from this script)
function get_project_root()
    return dirname(dirname(dirname(@__FILE__)))
end

# Build the Rust interop test binary
function build_rust_binary()
    project_root = get_project_root()
    hdf5_dir = joinpath(project_root, "hdf5")

    println("Building Rust interop test binary...")

    # Build with default features (runtime-loading is now the only mode)
    cmd = Cmd(`cargo build --example interop_test`; dir=hdf5_dir)

    result = run(cmd; wait=true)
    if result.exitcode != 0
        error("Failed to build Rust binary")
    end

    # Return path to the built binary
    binary_path = joinpath(project_root, "target", "debug", "examples", "interop_test")

    if !isfile(binary_path)
        error("Built binary not found at: $binary_path")
    end

    return binary_path
end

# Run the Rust binary with specified arguments
function run_rust_binary(binary_path::String, hdf5_lib::String, mode::String, file_path::String)
    cmd = `$binary_path --hdf5-lib $hdf5_lib --mode $mode --file $file_path`

    println("Running: $cmd")

    # Set LD_LIBRARY_PATH so the Rust binary can dlopen HDF5 and its dependencies
    jll_paths = get_jll_lib_paths()
    env = copy(ENV)
    ld_path = get(env, "LD_LIBRARY_PATH", "")
    all_paths = join(jll_paths, ":")
    env["LD_LIBRARY_PATH"] = isempty(ld_path) ? all_paths : "$all_paths:$ld_path"
    println("  LD_LIBRARY_PATH: $(env["LD_LIBRARY_PATH"])")
    cmd = setenv(cmd, env)

    # Run and capture output
    output = read(cmd, String)
    println("stdout: $output")

    return true, output, ""
end

# Create a test HDF5 file using Julia
function create_julia_test_file(filepath::String)
    println("Creating HDF5 file with Julia: $filepath")

    h5open(filepath, "w") do file
        # Write scalar attribute to root group
        attrs(file)["test_attr"] = "hello from julia/python"

        # Write 1D integer dataset
        file["integers"] = Int64[1, 2, 3, 4, 5]

        # Write 2D float dataset (Julia is column-major, HDF5 stores row-major)
        # Rust expects [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]] which is 2x3 in row-major
        # In Julia, we need to transpose: 3x2 matrix that becomes 2x3 when read row-major
        matrix = Float64[1.0 4.0; 2.0 5.0; 3.0 6.0]  # 3x2 in Julia (column-major)
        file["matrix"] = matrix

        # Write string dataset (variable-length strings)
        file["strings"] = ["foo", "bar", "baz"]
    end

    println("  Created successfully")
end

# Read and verify a test HDF5 file created by Rust
function verify_rust_test_file(filepath::String)
    println("Verifying HDF5 file created by Rust: $filepath")

    h5open(filepath, "r") do file
        # Read and verify attribute
        attr_value = read_attribute(file, "test_attr")
        @test attr_value == "hello from rust"
        println("  Attribute 'test_attr': $attr_value")

        # Read and verify integer dataset
        integers = read(file, "integers")
        @test integers == Int64[10, 20, 30, 40, 50]
        println("  Dataset 'integers': $integers")

        # Read and verify matrix dataset
        # Rust writes [[1.5, 2.5, 3.5], [4.5, 5.5, 6.5]] (2x3 row-major)
        # Julia reads as column-major, so we get transposed view
        matrix = read(file, "matrix")
        expected_matrix = Float64[1.5 4.5; 2.5 5.5; 3.5 6.5]  # 3x2 in Julia
        @test matrix == expected_matrix
        println("  Dataset 'matrix': $matrix")

        # Read and verify string dataset
        strings = read(file, "strings")
        @test strings == ["rust", "test", "data"]
        println("  Dataset 'strings': $strings")
    end

    println("  Verification successful")
end

# Main test function
function run_tests()
    println("=" ^ 60)
    println("HDF5 Julia <-> Rust Interoperability Tests")
    println("=" ^ 60)

    # Get HDF5 library path
    hdf5_lib = get_hdf5_lib_path()
    println("HDF5 library: $hdf5_lib")

    # Print HDF5 version
    hdf5_version = h5_get_libversion()
    println("HDF5 version: $hdf5_version")
    println()

    # Build Rust binary
    rust_binary = build_rust_binary()
    println("Rust binary: $rust_binary")
    println()

    # Create temp directory for test files
    test_dir = mktempdir()
    println("Test directory: $test_dir")
    println()

    @testset "HDF5 Interoperability Tests" begin
        @testset "Julia -> Rust" begin
            println("-" ^ 40)
            println("Test: Julia writes, Rust reads")
            println("-" ^ 40)

            julia_file = joinpath(test_dir, "julia_created.h5")

            # Julia creates the file
            create_julia_test_file(julia_file)

            # Rust reads and verifies
            success, stdout, stderr = run_rust_binary(rust_binary, hdf5_lib, "read", julia_file)
            @test success
            @test contains(stdout, "SUCCESS")

            println("Julia -> Rust: PASSED")
            println()
        end

        @testset "Rust -> Julia" begin
            println("-" ^ 40)
            println("Test: Rust writes, Julia reads")
            println("-" ^ 40)

            rust_file = joinpath(test_dir, "rust_created.h5")

            # Rust creates the file
            success, stdout, stderr = run_rust_binary(rust_binary, hdf5_lib, "write", rust_file)
            @test success
            @test contains(stdout, "SUCCESS")

            # Julia reads and verifies
            verify_rust_test_file(rust_file)

            println("Rust -> Julia: PASSED")
            println()
        end
    end

    println("=" ^ 60)
    println("All tests completed!")
    println("=" ^ 60)
end

# Run tests if this is the main script
if abspath(PROGRAM_FILE) == @__FILE__
    run_tests()
end
