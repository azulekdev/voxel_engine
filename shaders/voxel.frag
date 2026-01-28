#version 330 core

out vec4 FragColor;

in vec2 TexCoord;
in vec3 Normal;
in vec3 FragPos;
in float AO;

uniform sampler2D blockTexture;
uniform vec3 viewPos;
uniform vec3 lightDir;
uniform bool isWater;

const float ambientStrength = 0.4;
const vec3 skyColor = vec3(0.53, 0.81, 0.92);
const float fogStart = 80.0;
const float fogEnd = 200.0;

void main() {
    // Sample texture
    vec4 texColor = texture(blockTexture, TexCoord);
    
    // Lighting calculation
    vec3 norm = normalize(Normal);
    vec3 lightDirNorm = normalize(-lightDir);
    
    // Diffuse lighting
    float diff = max(dot(norm, lightDirNorm), 0.0);
    vec3 diffuse = diff * vec3(1.0, 0.95, 0.85); // Warm sunlight
    
    // Ambient lighting with sky color tint
    vec3 ambient = ambientStrength * vec3(0.6, 0.7, 0.9);
    
    // Apply ambient occlusion
    float aoFactor = 0.3 + (AO * 0.7); // AO ranges from 0.3 to 1.0
    
    // Combine lighting
    vec3 lighting = (ambient + diffuse) * aoFactor;
    vec3 result = texColor.rgb * lighting;
    
    // Vibrant color boost
    result = pow(result, vec3(0.9)); // Slight gamma adjustment
    result *= 1.2; // Brightness boost
    result = clamp(result, 0.0, 1.0);
    
    // Atmospheric fog
    float distance = length(viewPos - FragPos);
    float fogFactor = clamp((fogEnd - distance) / (fogEnd - fogStart), 0.0, 1.0);
    result = mix(skyColor, result, fogFactor);
    
    // Alpha
    float alpha = texColor.a;
    if (isWater) {
        alpha = 0.7;
    }
    
    FragColor = vec4(result, alpha);
}
