use libc::consts::os::posix88::{SIGTERM,SIGQUIT,SIGINT};
use libc;
use errors;

extern {
	fn signal(signal: libc::c_int, callback: extern fn(i32)) -> libc::c_int;
}

#[deriving(Eq, PartialEq)]
pub enum SignalType {
	SignalTerminate,
	SignalQuit,
	SignalInterrupt,
	None
}

static mut current_signal : SignalType = None;

extern "C" fn sigint_callback(signal: i32) {
	debug!("Signal callback");
	match signal {
		SIGTERM => {
			unsafe {current_signal = SignalTerminate};
		},
		SIGQUIT => {
			unsafe {current_signal = SignalQuit};
		},
		SIGINT => {
			unsafe {current_signal = SignalInterrupt};
		}
		_ => {

		}
	}
}

pub fn last_signal() -> SignalType {
	unsafe {
		let signal = current_signal;
		current_signal = None;
		return signal;
	}
}

pub fn init() {
	unsafe {
		if signal(SIGTERM, sigint_callback) == -1 {
			error!("Unable to register signal handler");
			errors::print_error("Error registering signal handler");
		}

		if signal(SIGQUIT, sigint_callback) == -1 {
			error!("Unable to register signal handler");
			errors::print_error("Error registering signal handler");
		}

		if signal(SIGINT, sigint_callback) == -1 {
			error!("Unable to register signal handler");
			errors::print_error("Error registering signal handler");
		}
	}
}