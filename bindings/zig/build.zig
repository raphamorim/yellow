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

    // Colors RGB example
    const colors_exe = b.addExecutable(.{
        .name = "colors-rgb",
        .root_source_file = b.path("examples/colors-rgb.zig"),
        .target = target,
        .optimize = optimize,
    });
    colors_exe.root_module.addImport("zaz", zaz_mod);
    colors_exe.addIncludePath(.{ .cwd_relative = "." });
    colors_exe.addLibraryPath(.{ .cwd_relative = "../../target/release" });
    colors_exe.linkSystemLibrary("zaz");
    colors_exe.linkLibC();
    b.installArtifact(colors_exe);

    // Run commands
    const run_basic = b.addRunArtifact(basic_exe);
    const run_colors = b.addRunArtifact(colors_exe);

    const run_basic_step = b.step("run-basic", "Run basic example");
    run_basic_step.dependOn(&run_basic.step);

    const run_colors_step = b.step("run-colors", "Run colors-rgb example");
    run_colors_step.dependOn(&run_colors.step);
}
