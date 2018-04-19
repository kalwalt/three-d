use gl;
use std;
use glm;

use dust::utility;
use dust::shader;

use std::ffi::{CString};

#[derive(Debug)]
pub enum Error {
    Shader(shader::Error),
    FailedToLinkProgram {message: String},
    FailedToCreateCString(std::ffi::NulError)
}

impl From<shader::Error> for Error {
    fn from(other: shader::Error) -> Self {
        Error::Shader(other)
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(other: std::ffi::NulError) -> Self {
        Error::FailedToCreateCString(other)
    }
}

#[derive(Clone)]
pub struct Program {
    gl: gl::Gl,
    id: gl::types::GLuint,
}

impl Program
{
    pub fn from_resource(gl: &gl::Gl, name: &str) -> Result<Program, Error>
    {
        const POSSIBLE_EXT: [&str; 2] = [
            ".vert",
            ".frag",
        ];

        let shaders = POSSIBLE_EXT.iter()
            .map(|file_extension| {
                shader::Shader::from_resource(gl, &format!("{}{}", name, file_extension))
            })
            .collect::<Result<Vec<shader::Shader>, shader::Error>>()?;

        Program::from_shaders(gl, &shaders[..])
    }

    pub fn from_source(gl: &gl::Gl, vertex_shader_source: &str, fragment_shader_source: &str) -> Result<Program, Error>
    {
        let vert_shader = shader::Shader::from_vert_source(gl, vertex_shader_source)?;
        let frag_shader = shader::Shader::from_frag_source(gl, fragment_shader_source)?;
        return Program::from_shaders( gl, &[vert_shader, frag_shader] );
    }

    pub fn from_shaders(gl: &gl::Gl, shaders: &[shader::Shader]) -> Result<Program, Error>
    {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe { gl.AttachShader(program_id, shader.id()); }
        }

        unsafe { gl.LinkProgram(program_id); }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = utility::create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar
                );
            }

            return Err(Error::FailedToLinkProgram {message: error.to_string_lossy().into_owned() });;
        }

        for shader in shaders {
            unsafe { gl.DetachShader(program_id, shader.id()); }
        }

        Ok(Program { gl: gl.clone(), id: program_id })
    }

    pub fn add_uniform_int(&self, name: &str, data: &i32) -> Result<(), Error>
    {
        let location= self.get_uniform_location(name)?;
        unsafe {
            self.gl.Uniform1iv(location, 1, data);
        }
        Ok(())
    }

    pub fn add_uniform_float(&self, name: &str, data: &f32) -> Result<(), Error>
    {
        let location= self.get_uniform_location(name)?;
        unsafe {
            self.gl.Uniform1fv(location, 1, data);
        }
        Ok(())
    }

    pub fn add_uniform_vec2(&self, name: &str, data: &glm::Vec2) -> Result<(), Error>
    {
        let location= self.get_uniform_location(name)?;
        unsafe {
            self.gl.Uniform2fv(location, 1, &data[0]);
        }
        Ok(())
    }

    pub fn add_uniform_vec3(&self, name: &str, data: &glm::Vec3) -> Result<(), Error>
    {
        let location= self.get_uniform_location(name)?;
        unsafe {
            self.gl.Uniform3fv(location, 1, &data[0]);
        }
        Ok(())
    }


    pub fn add_uniform_vec4(&self, name: &str, data: &glm::Vec4) -> Result<(), Error>
    {
        let location= self.get_uniform_location(name)?;
        unsafe {
            self.gl.Uniform4fv(location, 1, &data[0]);
        }
        Ok(())
    }

    pub fn add_uniform_mat4(&self, name: &str, data: &glm::Matrix4<f32>) -> Result<(), Error>
    {
        let location= self.get_uniform_location(name)?;
        unsafe {
            self.gl.UniformMatrix4fv(location, 1, gl::FALSE, &data[0][0]);
        }
        Ok(())
    }

    fn get_uniform_location(&self, name: &str) -> Result<i32, Error>
    {
        self.set_used();
        let location: i32;
        let c_str = CString::new(name)?;
        unsafe {
            location = self.gl.GetUniformLocation(self.id, c_str.as_ptr());
        }
        Ok(location)
    }

    pub fn setup_attributes(&self, data: &Vec<f32>) -> Result<(), Error>
    {
        let mut vbo: gl::types::GLuint = 0;
        unsafe {
            self.gl.GenBuffers(1, &mut vbo);
            self.gl.BindBuffer(gl::ARRAY_BUFFER, vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER, // target
                (data.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr, // size of data in bytes
                data.as_ptr() as *const gl::types::GLvoid, // pointer to data
                gl::STATIC_DRAW, // usage
            );
        }

        let location = self.get_attribute_location("Position")? as gl::types::GLuint;
        unsafe {
            self.gl.EnableVertexAttribArray(location);
            self.gl.VertexAttribPointer(
                location, // index of the generic vertex attribute
                3, // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE, // normalized (int-to-float conversion)
                (3 * std::mem::size_of::<f32>()) as gl::types::GLint, // stride (byte offset between consecutive attributes)
                std::ptr::null() // offset of the first component
            );
            self.gl.BindBuffer(gl::ARRAY_BUFFER, 0);
        }
        Ok(())
    }

    fn get_attribute_location(&self, name: &str) -> Result<i32, Error>
    {
        self.set_used();
        let location: i32;
        let c_str = CString::new(name)?;
        unsafe {
            location = self.gl.GetAttribLocation(self.id, c_str.as_ptr());
        }
        Ok(location)
    }

    pub fn set_used(&self) {
        unsafe {
            static mut CURRENTLY_USED: gl::types::GLuint = 1000000 as u32;
            if self.id != CURRENTLY_USED
            {
                self.gl.UseProgram(self.id);
                CURRENTLY_USED = self.id;
            }
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}
