use gl::types::*;
use glam::{Mat4, Vec3};
use std::ffi::CString;
use std::fs;
use std::ptr;
use std::str;

pub struct Shader {
    pub id: u32,
}

impl Shader {
    pub fn new(vertex_path: &str, fragment_path: &str) -> Self {
        let mut shader = Shader { id: 0 };

        let vertex_code = fs::read_to_string(vertex_path)
            .unwrap_or_else(|_| panic!("Failed to read vertex shader: {}", vertex_path));
        let fragment_code = fs::read_to_string(fragment_path)
            .unwrap_or_else(|_| panic!("Failed to read fragment shader: {}", fragment_path));

        let v_shader_code = CString::new(vertex_code.as_bytes()).unwrap();
        let f_shader_code = CString::new(fragment_code.as_bytes()).unwrap();

        unsafe {
            let vertex = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex, 1, &v_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(vertex);
            shader.check_compile_errors(vertex, "VERTEX");

            let fragment = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment, 1, &f_shader_code.as_ptr(), ptr::null());
            gl::CompileShader(fragment);
            shader.check_compile_errors(fragment, "FRAGMENT");

            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex);
            gl::AttachShader(id, fragment);
            gl::LinkProgram(id);
            shader.check_compile_errors(id, "PROGRAM");

            gl::DeleteShader(vertex);
            gl::DeleteShader(fragment);

            shader.id = id;
        }

        shader
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(self.id, c_name.as_ptr()),
                value as i32,
            );
        }
    }

    #[allow(dead_code)]
    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            gl::Uniform1i(gl::GetUniformLocation(self.id, c_name.as_ptr()), value);
        }
    }

    #[allow(dead_code)]
    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            gl::Uniform1f(gl::GetUniformLocation(self.id, c_name.as_ptr()), value);
        }
    }

    pub fn set_mat4(&self, name: &str, value: &Mat4) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.id, c_name.as_ptr()),
                1,
                gl::FALSE,
                &value.to_cols_array()[0],
            );
        }
    }

    pub fn set_vec3(&self, name: &str, value: &Vec3) {
        unsafe {
            let c_name = CString::new(name).unwrap();
            gl::Uniform3fv(
                gl::GetUniformLocation(self.id, c_name.as_ptr()),
                1,
                &value.to_array()[0],
            );
        }
    }

    unsafe fn check_compile_errors(&self, shader: u32, type_: &str) {
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(1024);
        unsafe {
            info_log.set_len(1024 - 1); // subtract 1 to skip the trailing null character
        }

        if type_ != "PROGRAM" {
            unsafe {
                gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetShaderInfoLog(
                        shader,
                        1024,
                        ptr::null_mut(),
                        info_log.as_mut_ptr() as *mut GLchar,
                    );
                    println!(
                        "ERROR::SHADER_COMPILATION_ERROR of type: {}\n{}\n -- --------------------------------------------------- -- ",
                        type_,
                        str::from_utf8(&info_log).unwrap()
                    );
                }
            }
        } else {
            unsafe {
                gl::GetProgramiv(shader, gl::LINK_STATUS, &mut success);
                if success != gl::TRUE as GLint {
                    gl::GetProgramInfoLog(
                        shader,
                        1024,
                        ptr::null_mut(),
                        info_log.as_mut_ptr() as *mut GLchar,
                    );
                    println!(
                        "ERROR::PROGRAM_LINKING_ERROR of type: {}\n{}\n -- --------------------------------------------------- -- ",
                        type_,
                        str::from_utf8(&info_log).unwrap()
                    );
                }
            }
        }
    }
}
