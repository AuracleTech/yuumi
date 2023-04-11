@echo off
glslc.exe assets/shaders/shader.vert -o data/vert.spv && glslc.exe assets/shaders/shader.frag -o data/frag.spv && cls