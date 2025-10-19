const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    // Create zaz module
    const zaz_mod = b.createModule(.{
        .root_source_file = b.path("zaz.zig"),
        .link_libc = true,
    });
    zaz_mod.addIncludePath(b.path("."));

    // Basic example
    const basic_exe = b.addExecutable(.{
        .name = "basic",
        .root_source_file = b.path("examples/basic.zig"),
        .target = target,
        .optimize = optimize,
    });
    basic_exe.root_module.addImport("zaz", zaz_mod);
    basic_exe.addIncludePath(.{ .cwd_relative = "." });
    basic_exe.addLibraryPath(.{ .cwd_relative = "../../target/release" });
    basic_exe.linkSystemLibrary("zaz");
    basic_exe.linkLibC();
    b.installArtifact(basic_exe);

    // Mosaic example
    const mosaic_exe = b.addExecutable(.{
        .name = "mosaic",
        .root_source_file = b.path("examples/mosaic.zig"),
        .target = target,
        .optimize = optimize,
    });
    mosaic_exe.root_module.addImport("zaz", zaz_mod);
    mosaic_exe.addIncludePath(.{ .cwd_relative = "." });
    mosaic_exe.addLibraryPath(.{ .cwd_relative = "../../target/release" });
    mosaic_exe.linkSystemLibrary("zaz");
    mosaic_exe.addCSourceFile(.{
        .file = b.path("image_loader.c"),
        .flags = &[_][]const u8{"-std=c99"},
    });
    mosaic_exe.linkLibC();
    b.installArtifact(mosaic_exe);

    // Run commands
    const run_basic = b.addRunArtifact(basic_exe);
    const run_mosaic = b.addRunArtifact(mosaic_exe);

    const run_basic_step = b.step("run-basic", "Run basic example");
    run_basic_step.dependOn(&run_basic.step);

    const run_mosaic_step = b.step("run-mosaic", "Run mosaic example");
    run_mosaic_step.dependOn(&run_mosaic.step);
}
