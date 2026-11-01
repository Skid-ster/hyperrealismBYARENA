#ifndef PROCESSING_GLSL
#define PROCESSING_GLSL

// https://www.shadertoy.com/view/XdcXzn

mat4 contrastMatrix(float contrast) {
  float t = (1.0 - contrast) / 2.0;

  return mat4(
    contrast,
    0,
    0,
    0,
    0,
    contrast,
    0,
    0,
    0,
    0,
    contrast,
    0,
    t,
    t,
    t,
    1
  );

}

mat4 saturationMatrix(float saturation) {
  vec3 luminance = vec3(0.3086, 0.6094, 0.082);

  float oneMinusSat = 1.0 - saturation;

  vec3 red = vec3(luminance.x * oneMinusSat);
  red += vec3(saturation, 0, 0);

  vec3 green = vec3(luminance.y * oneMinusSat);
  green += vec3(0, saturation, 0);

  vec3 blue = vec3(luminance.z * oneMinusSat);
  blue += vec3(0, 0, saturation);

  return mat4(red, 0, green, 0, blue, 0, 0, 0, 0, 1);
}

// A restrained teal-shadow / amber-highlight split tone gives the pack a
// deliberate illustrated-fantasy palette without bypassing the LabPBR lighting
// and reflection passes. Keep this in the final display transform only.
vec3 hyperrealismGrade(vec3 color) {
  float luminance = dot(color, vec3(0.2126, 0.7152, 0.0722));
  float strength = float(STYLE_STRENGTH) / 100.0;
  float shadows = 1.0 - smoothstep(0.06, 0.48, luminance);
  float highlights = smoothstep(0.48, 0.95, luminance);
  vec3 coolShadow = mix(vec3(1.0), vec3(0.82, 0.95, 1.12), shadows * strength);
  vec3 warmHighlight = mix(vec3(1.0), vec3(1.12, 1.02, 0.82), highlights * strength);
  return clamp(color * coolShadow * warmHighlight, 0.0, 1.0);
}

vec4 postProcess(vec4 color) {
  color = contrastMatrix(CONTRAST) * saturationMatrix(SATURATION) * color;
  color.rgb = hyperrealismGrade(color.rgb);
  return color;
}

#endif
