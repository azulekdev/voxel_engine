use glam::{Mat4, Vec3};
use std::ffi::CString;
use std::fs;
use std::ptr;

pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Self {
        let vertex_code = fs::read_to_string(vertex_path)
            .expect(&format!("Failed to read vertex shader: {}", vertex_path));
        let fragment_code = fs::read_to_string(fragment_path).expect(&format!(
            "Failed to read fragment shader: {}",
            fragment_path
        ));

        unsafe {
            let vertex = Self::compile_shader(&vertex_code, gl::VERTEX_SHADER);
            let fragment = Self::compile_shader(&fragment_code, gl::FRAGMENT_SHADER);

            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            Self::check_link_errors(id);

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            Self { id }
        }
    }

    unsafe fn compile_shader(source: &str, shader_type: u32) -> u32 {
        unsafe {
            let shader = gl::CreateShader(shader_type);
            let c_str = CString::new(source.as_bytes()).unwrap();
            gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            Self::check_compile_errors(shader, shader_type);
            shader
        }
    }

    unsafe fn check_compile_errors(shader: u32, shader_type: u32) {
        unsafe {
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
                let type_name = if shader_type == gl::VERTEX_SHADER {
                    "VERTEX"
                } else {
                    "FRAGMENT"
                };
                panic!(
                    "{} shader compilation failed:\n{}",
                    type_name,
                    String::from_utf8_lossy(&buffer)
                );
            }
        }
    }

    unsafe fn check_link_errors(program: u32) {
        unsafe {
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut len = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
                let mut buffer = vec![0u8; len as usize];
                gl::GetProgramInfoLog(
                    program,
                    len,
                    ptr::null_mut(),
                    buffer.as_mut_ptr() as *mut i8,
                );
                panic!(
                    "Shader program linking failed:\n{}",
                    String::from_utf8_lossy(&buffer)
                );
            }
        }
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_mat4(&self, name: &str, mat: &Mat4) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::UniformMatrix4fv(location, 1, gl::FALSE, mat.as_ref().as_ptr());
        }
    }

    pub fn set_vec3(&self, name: &str, vec: &Vec3) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform3f(location, vec.x, vec.y, vec.z);
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform1i(location, value as i32);
        }
    }

    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(self.id, c_name.as_ptr());
            gl::Uniform1i(location, value);
        }
    }
}
