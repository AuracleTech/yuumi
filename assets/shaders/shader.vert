#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform PushConstants {
    mat4 model;
} pcs;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;
layout(location = 2) in vec2 inTexCoord;
layout(location = 3) in vec4 in1InstanceModel;
layout(location = 4) in vec4 in2InstanceModel;
layout(location = 5) in vec4 in3InstanceModel;
layout(location = 6) in vec4 in4InstanceModel;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    mat4 inInstanceModel = mat4(in1InstanceModel, in2InstanceModel, in3InstanceModel, in4InstanceModel);
    gl_Position = ubo.proj * ubo.view * inInstanceModel * pcs.model  * vec4(inPosition, 1.0);
    fragColor = inColor;
    fragTexCoord = inTexCoord;
}