use gl;
use dust::*;
use dust::core::program;
use dust::traits;
use dust::core::surface;
use dust::camera;

pub struct Monkey {
    program: program::Program,
    model: surface::TriangleSurface
}

impl traits::Reflecting for Monkey
{
    fn reflect(&self, transformation: &Mat4, camera: &camera::Camera) -> Result<(), traits::Error>
    {
        self.program.add_uniform_vec3("color", &vec3(1.0, 1.0, 1.0))?;
        self.program.add_uniform_mat4("modelMatrix", &transformation)?;
        self.program.add_uniform_mat4("viewMatrix", &camera.get_view())?;
        self.program.add_uniform_mat4("projectionMatrix", &camera.get_projection())?;
        self.program.add_uniform_mat4("normalMatrix", &transformation.try_inverse().unwrap().transpose())?;
        self.model.render()?;
        Ok(())
    }
}

impl Monkey
{
    pub fn create(gl: &gl::Gl) -> Result<Monkey, traits::Error>
    {
        let mesh = mesh_loader::load_obj_as_static_mesh("/examples/assets/models/suzanne.obj").unwrap();
        let program = program::Program::from_resource(&gl, "examples/assets/shaders/standard")?;
        let mut model = surface::TriangleSurface::create(gl, &mesh)?;
        model.add_attributes(&mesh, &program,&vec!["position", "normal"])?;

        Ok(Monkey { program, model })
    }
}
