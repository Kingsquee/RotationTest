#![allow(unused_mut)]
#[macro_use]
extern crate logl2;
use logl2::*;

extern crate nalgebra as na;
use na::*;

mod framework;
use framework::DemoFramework;

logl2_pass! {
	floor_pass::FloorPass {
		element_index_type: u16,
		program: {
			vertex_shader: "
				void main() {
					gl_Position = u_proj * u_view * vec4(a_pos, 1.0);
				}
			",
			fragment_shader: "
				void main() {
					gl_FragColor = vec4(vec3(gl_FragCoord.z), 1.0);
				}
			"
		}
		attributes: {
			non_interleaved: {
				a_pos: [f32; 3] {
					normalize: false,
					index: 0,
					divisor: 0
				}
			}
			interleaved: {}
		}
		uniforms: {
			u_view: [[f32; 4]; 4],
			u_proj: [[f32; 4]; 4]
		}
		bitmaps: {}
		cubemaps: {}
	}
}

logl2_pass! {
	perspective_pass::PerspectivePass {
		element_index_type: u16,
		program: {
			vertex_shader: "
				varying vec3 v_col;
				void main() {
					v_col = a_col;
					gl_Position = u_proj * u_view * u_model * vec4(a_pos, 1.0);
				}
			",
			fragment_shader: "
				varying vec3 v_col;
				void main() {
					gl_FragColor = vec4(v_col, 1.0);
				}
			"
		}
		attributes: {
			non_interleaved: {
				a_pos: [f32; 3] {
					normalize: false,
					index: 0,
					divisor: 0
				}
				a_col: [f32; 3] {
					normalize: false,
					index: 1,
					divisor: 0
				}
			}
			interleaved: {}
		}
		//TODO: define varyings in macro rather than in shaders
		/*
		varyings: {
			varying vec3 v_col;
		}
		*/
		uniforms: {
			u_model: [[f32; 4]; 4],
			u_view: [[f32; 4]; 4],
			u_proj: [[f32; 4]; 4]
		}
		bitmaps: {}
		cubemaps: {}
	}
}

fn main() {
	let mut width: u32 = 800;
	let mut height: u32 = 600;

	let mut demo = DemoFramework::new("logl2 non interleaved example", width, height);
	let mut logl2 = LOGL2::load_with(|s| demo.get_proc_address(s) as *const _).unwrap();

	let positions_buffer: NonInterleavedVertexBufferHandle<[f32; 3]> = logl2.create_non_interleaved_vertex_buffer(&[
		// floor
		[-0.5 ,  0.5, 0.0], // 0
		[ 0.5 ,  0.5, 0.0], // 1
		[-0.75, -0.5, 0.0], // 2
		[ 0.75, -0.5, 0.0], // 3

		// fore wall
		[-0.5, -1.0, 0.0],  // 4
		[ 0.5, -1.0, 0.0],  // 5

		// back wall
		[-0.55, 1.0, 0.0],   // 6
		[ 0.55, 1.0, 0.0]    // 7
	], DrawHint::Static);

	let colors_buffer: NonInterleavedVertexBufferHandle<[f32; 3]> = logl2.create_non_interleaved_vertex_buffer(&[
		// floor
		[1.0 , 1.00, 1.00], // 0
		[1.0 , 1.00, 1.00], // 1
		[0.75, 0.75, 1.00], // 2
		[0.75, 0.75, 1.00], // 3

		// fore wall
		[0.25, 0.25, 0.50], // 4
		[0.25, 0.25, 0.50], // 5

		// back wall
		[0.75, 0.75, 1.0], // 6
		[0.75, 0.75, 1.0]  // 7
	], DrawHint::Static);

	let element_buffer: ElementBufferHandle<u16> = logl2.create_element_buffer(&[
		// floor
		0, 1, 2,
		2, 1, 3,

		// fore wall
		2, 3, 4,
		4, 3, 5,

		// back wall
		1, 0, 6,
		1, 6, 7u16,
	], DrawHint::Static);

	let floor_quad_positions_buffer: NonInterleavedVertexBufferHandle<[f32; 3]> = logl2.create_non_interleaved_vertex_buffer(&[
		[-100.0,  0.0, 100.0], [ 100.0, 0.0, 100.0],
		[-100.0, 0.0, -100.0], [ 100.0, 0.0, -100.0]
	], DrawHint::Static);

	let floor_quad_element_buffer: ElementBufferHandle<u16> = logl2.create_element_buffer(&[
		0, 1, 2,
		2, 1, 3
	], DrawHint::Static);

	let mut camera_translation = Vector3::new(0.0, 2.0, 3.0);
	let mut camera_rotation = Rotation3::from_euler_angles(0.0, 0.0, 0.0);

	let mut model_translation = Vector3::new(0.0, 2.0, 0.0);
	let mut model_rotation = Rotation3::from_euler_angles(0.0, 0.0, 0.0);

	let mut floor_pass = FloorPass::new(&mut logl2);
	let mut perspective_pass = PerspectivePass::new(&mut logl2);

	demo.main_loop(
		|resized: Option<(u32, u32)>| {
			match resized {
				Some(dimensions) => {
					let (w, h) = dimensions;
					width = w;
					height = h;
				}
				None => ()
			}

			// Rotate the camera
			camera_rotation *= Rotation3::from_euler_angles(0.0, 0.01, 0.0);

			let u_model = {
				let o = 1.0f32;
				let z = 0.0f32;

				let t = model_translation;
				let r = model_rotation;

				// Object's transform matrix, (model space -> world space)
				&[
					/*X*/ [r[(0, 0)], r[(0, 1)], r[(0, 2)], z],
					/*Y*/ [r[(1, 0)], r[(1, 1)], r[(1, 2)], z],
					/*Z*/ [r[(2, 0)], r[(2, 1)], r[(2, 2)], z],
					/*T*/ [t[0], t[1], t[2], o]
				]
			};

			let u_view = {
				let o = 1.0f32;
				let z = 0.0f32;

				let t = camera_translation;
				let r = camera_rotation;

				// Camera's inverse transform matrix (world space -> view space)
				&[
					/*X*/ [r[(0, 0)], 	r[(1, 0)], 	r[(2, 0)], 	z],
					/*Y*/ [r[(0, 1)], 	r[(1, 1)], 	r[(2, 1)], 	z],
					/*Z*/ [r[(0, 2)], 	r[(1, 2)], 	r[(2, 2)], 	z],
					/*T*/ [-t[0], 	-t[1], 	-t[2], 	o]
				]
			};

			let u_perspective_proj = {
				let x_aspect = width as f32 / height as f32;
				let r = 0.5 * x_aspect;
				let l = -r;
				let t = 0.5;
				let b = -t;
				let n = 0.99f32;
				let f = 1000f32;

				// perspective
				// (view space -> projection space)
				&[
					/*X*/ [(2.0*n)/(r-l), 		0f32, 				0f32, 				0f32],
					/*Y*/ [0f32, 				(2.0*n)/(t-b),		0f32, 				0f32],
					/*Z*/ [(r+l)/(r-l), 		(t+b)/(t-b), 		-(f+n)/(f-n), 		-1f32],
					/*T*/ [0f32, 				0f32, 				(-2.0*f*n)/(f-n), 	0f32]
				]
			};

			logl2.render(
				&mut floor_pass,
				floor_pass::Parameters {
					elements: &floor_quad_element_buffer,
					attributes: floor_pass::Attributes {
						a_pos: &floor_quad_positions_buffer,
					},
					uniforms: floor_pass::Uniforms {
						u_view: &u_view,
						u_proj: &u_perspective_proj
					}
				},
				State {
					canvas: Canvas {
						viewport_area: [[0, 0],[width as i32, height as i32]],
						scissor_area: Some([[0, 0],[width as i32, height as i32]])
					},
					color_buffer: ColorBuffer {
						clear: Some([0.2, 0.3, 0.3, 1.0]),
						write_mask: ColorWriteMask {
							red: true,
							green: true,
							blue: true,
							alpha: true
						}
					},
					depth_buffer: Some(
						DepthBuffer {
							clear: Some(1.0),
							range: [0.0, 1.0],
							operator: DepthComparisonOperator::LessThan,
							write_mask: true,
							polygon_offset: None
						}
					),
					stencil_buffer: None,
					blending: None,
					face_culling: FaceCulling {
						front_face_winding_order: WindingOrder::Clockwise,
						cull_face: None
					},
					multisampling: None,
					primitive_type: PrimitiveType::Triangles
				}
			);

			logl2.render(
				&mut perspective_pass,
				perspective_pass::Parameters {
					elements: &element_buffer,
					attributes: perspective_pass::Attributes {
						a_pos: &positions_buffer,
						a_col: &colors_buffer
					},
					uniforms: perspective_pass::Uniforms {
						u_model: &u_model,
						u_view: &u_view,
						u_proj: &u_perspective_proj
					}
				},
				State {
					canvas: Canvas {
						viewport_area: [[0, 0],[width as i32, height as i32]],
						scissor_area: Some([[0, 0],[width as i32, height as i32]])
					},
					color_buffer: ColorBuffer {
						clear: None,
						write_mask: ColorWriteMask {
							red: true,
							green: true,
							blue: true,
							alpha: true
						}
					},
					depth_buffer: Some(
						DepthBuffer {
							clear: None,
							range: [0.0, 1.0],
							operator: DepthComparisonOperator::LessThan,
							write_mask: true,
							polygon_offset: None
						}
					),
					stencil_buffer: None,
					blending: None,
					face_culling: FaceCulling {
						front_face_winding_order: WindingOrder::Clockwise,
						cull_face: None
					},
					multisampling: None,
					primitive_type: PrimitiveType::Triangles
				}
			);
		}
	)
}
