pub mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: "
            #version 460
            layout(location = 0) in vec3 position; 
            layout(location = 1) in vec4 color; 
            
            // layout(location = 1) in float time;
            layout(location = 0) out vec4 color_to_frag;
            
            layout(set = 0, binding = 0) uniform WorldMats {
                mat4 world;
                mat4 view;
                mat4 proj;
            } uniforms;

            void main() {
                mat4 worldview = uniforms.view * uniforms.world;
                vec4 pos = uniforms.proj * worldview * vec4(position, 1.0);
                gl_Position = pos; 
                gl_PointSize = 2.0;// + 0.05 * pos.w;
                color_to_frag = color;
                // f_time = time;
            }
            "
    }
}
pub mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460
            layout(location = 0) out vec4 f_color;
            // layout(location = 0) in float f_time;
            layout(location = 0) in vec4 color_to_frag;
            
            void main() {
                // float temp = mod(f_time, 11100000.0) / 11100000.0;
                // f_color = vec4(0.0, 0.85, 0.00 * temp, 1.0);
                f_color = color_to_frag;
            }
            "
    }
}
