@echo off
glslc.exe assets/shaders/shader.vert -o lib/vert.spv && glslc.exe assets/shaders/shader.frag -o lib/frag.spv && cls