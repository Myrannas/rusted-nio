use libc::funcs::c95::stdio::perror;


pub fn print_error(message : &str) {
	let cstr = message.to_c_str();
	unsafe {perror(cstr.as_ptr())};
}
