#version 330 core
out vec4 FragColor;

in vec2 TexCoord;
in vec3 Normal;
in vec3 FragPos;

uniform sampler2D ourTexture;
uniform vec3 lightDir;
uniform vec3 viewPos;
uniform bool isWater;

void main()
{
    // Vibrant colors settings
    float gamma = 2.2;
    // Simple texture fetch
    vec4 texColor = texture(ourTexture, TexCoord);
    
    // Ambient
    float ambientStrength = 0.5;
    vec3 ambient = ambientStrength * vec3(1.0, 1.0, 1.0);
  
    // Diffuse
    vec3 norm = normalize(Normal);
    vec3 lightDirNormalized = normalize(-lightDir); 
    float diff = max(dot(norm, lightDirNormalized), 0.0);
    vec3 diffuse = diff * vec3(1.0, 0.95, 0.9); // Slight warm sunlight

    // Specular (Simple Blinn-Phong)
    float specularStrength = 0.1;
    vec3 viewDir = normalize(viewPos - FragPos);
    vec3 halfwayDir = normalize(lightDirNormalized + viewDir);  
    float spec = pow(max(dot(norm, halfwayDir), 0.0), 32.0);
    vec3 specular = specularStrength * spec * vec3(1.0);  

    vec3 result = (ambient + diffuse + specular) * texColor.rgb;

    // Gamma correction
    result = pow(result, vec3(1.0/gamma));
    
    // Enhanced coloring system - use texture if available, otherwise use procedural colors
    vec3 baseColor;
    if(texColor.r < 0.01 && texColor.g < 0.01 && texColor.b < 0.01) {
        // Procedural coloring based on normal and position
        if(abs(Normal.y) > 0.9) {
            // Top/bottom faces - grass green
            baseColor = vec3(0.35, 0.75, 0.25);
        } else {
            // Side faces - dirt brown with variation
            baseColor = vec3(0.55, 0.38, 0.22);
            // Add slight variation based on position for texture-like appearance
            float variation = sin(FragPos.x * 2.0) * 0.05 + cos(FragPos.z * 2.0) * 0.05;
            baseColor += vec3(variation);
        }
        
        // Apply lighting to base color
        vec3 ambient = ambientStrength * vec3(1.0, 1.0, 1.0);
        vec3 norm = normalize(Normal);
        vec3 lightDirNormalized = normalize(-lightDir);
        float diff = max(dot(norm, lightDirNormalized), 0.0);
        vec3 diffuse = diff * vec3(1.0, 0.95, 0.9);
        
        result = (ambient + diffuse) * baseColor;
    }
    
    // Gamma correction
    result = pow(result, vec3(1.0/gamma));
    
    // Saturation boost for "gammy" look
    vec3 gray = vec3(dot(result, vec3(0.2126, 0.7152, 0.0722)));
    result = mix(gray, result, 1.3); // Slightly less saturation for natural look

    float alpha = texColor.a;
    if(isWater) {
        alpha = 0.7; // Transparency for water
    }
    // Ensure minimum alpha for opaque blocks
    if(alpha < 0.1 && !isWater) {
        alpha = 1.0;
    }

    FragColor = vec4(result, alpha);
}
