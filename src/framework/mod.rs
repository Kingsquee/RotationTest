#![allow(dead_code)]

extern crate sdl2;
use self::sdl2::{Sdl, EventPump, VideoSubsystem};
use self::sdl2::video::*;
use self::sdl2::event::{Event, WindowEvent};
use self::sdl2::keyboard::Keycode;

pub struct DemoFramework {
	sdl2_context: Sdl,
	gl_context: GLContext,
	video_context: VideoSubsystem,
	window: Window,
	events: EventPump
}

impl DemoFramework {
	#[inline(always)]
	pub fn get_proc_address(&self, proc_name: &str) -> *const () {
		self.video_context.gl_get_proc_address(proc_name)
	}

	// initialize windowing and events
	pub fn new(name: &str, width: u32, height: u32) -> DemoFramework {
		let sdl2_context = sdl2::init().unwrap();
		let video_context = sdl2_context.video().unwrap();

		#[cfg(any(target_arch="x86", target_arch="x86_64"))] {
			let gl_attributes = video_context.gl_attr();
			gl_attributes.set_context_profile(sdl2::video::GLProfile::Compatibility);
			gl_attributes.set_context_version(2, 0);
		}

		let window = video_context
			.window(name, width, height)
			.position_centered()
			.opengl()
			.build()
		.unwrap();

		let gl_context = window.gl_create_context().unwrap();
		window.gl_make_current(&gl_context).unwrap();

		let events = sdl2_context.event_pump().unwrap();

		DemoFramework {
			sdl2_context,
			gl_context,
			video_context,
			window,
			events
		}
	}

	// desktop
	#[cfg(not(any(target_arch = "asmjs", target_arch = "wasm32")))]
	pub fn main_loop<F>(&mut self, mut f: F) where F: FnMut(Option<(u32, u32)>) {
		loop {
			let resized = self.one_pass();
			f(resized);
			self.window.gl_swap_window();
		}
	}


	// emscripten
	// from https://github.com/badboy/rust-triangle-js/blob/master/src/main.rs
	// so this function is under the MIT license
	#[cfg(any(target_arch = "asmjs", target_arch = "wasm32"))]
	pub fn main_loop<F>(&mut self, mut f: F) where F: FnMut(Option<(u32, u32)>) {

		use std::cell::RefCell;
		use std::ptr::null_mut;
		use std::os::raw::{c_int, c_void};

		#[allow(non_camel_case_types)]
		type em_callback_func = unsafe extern fn();
		extern {
			fn emscripten_set_main_loop(func : em_callback_func, fps : c_int, simulate_infinite_loop : c_int);
		}

		thread_local!(static MAIN_LOOP_CALLBACK: RefCell<*mut c_void> = RefCell::new(null_mut()));

		pub fn set_main_loop_callback<F>(callback : F) where F : FnMut() {
			MAIN_LOOP_CALLBACK.with(|log| {
					*log.borrow_mut() = &callback as *const _ as *mut c_void;
					});

			unsafe { emscripten_set_main_loop(wrapper::<F>, 0, 1); }

			unsafe extern "C" fn wrapper<F>() where F : FnMut() {
				MAIN_LOOP_CALLBACK.with(|z| {
					let closure = *z.borrow_mut() as *mut F;
					(*closure)();
				});
			}
		}

		set_main_loop_callback(
			|| {
				self.one_pass();
				f(None)
			}
		);

	}

	fn one_pass(&mut self) -> Option<(u32, u32)> {
		let mut resized = None;
		for event in self.events.poll_iter() {
			match event {
				Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), ..} => {
					use std::process;
					process::exit(0);
				}
				Event::Window { win_event, .. } => {
					match win_event {
						WindowEvent::Resized(width, height) => {
							resized = Some((width as u32, height as u32));
						}
						_ => ()
					}
				}
				_ => ()
			}
		}
		resized
	}
}
