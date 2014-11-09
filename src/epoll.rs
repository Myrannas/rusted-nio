use libc::funcs::posix88::unistd::close;
use libc;
use errors;

#[allow(dead_code)]
enum EPollOperation {
CtlAdd = 1,
CtlDel = 2,
CtlMod = 3
}

#[allow(dead_code)]
pub enum EPollEventType
{
EPollIn = 0x001,
EPollPri = 0x002,
EPollOut = 0x004,
EPollRdNorm = 0x040,
EPollRdBand = 0x080,
EPollWrNorm = 0x100,
EPollWrBand = 0x200,
EPollMsg = 0x400,
EPollErr = 0x008,
EPollHup = 0x010,
EPollOneShot = (1 << 30),
EPollEt = (1 << 31)
}

#[repr(C)]
#[deriving(Show)]
pub struct EPollEvent {
	events: u32,
	pub data: u64
}

extern {
	fn epoll_create1(flags: i32) -> libc::c_int;
	fn epoll_ctl(epoll: i32, operation: i32, fd: i32, event: *mut EPollEvent) -> libc::c_int;
	fn epoll_wait(epoll: i32, events: *mut EPollEvent, maxEvents: libc::c_int, timeout: libc::c_int) -> libc::c_int;
}

#[deriving(Show)]
pub struct EPoll {
	handle: i32,
	event_buffer: Vec<EPollEvent>
}

impl EPoll {
	pub fn new(flags : u32, buffer_size: uint) -> EPoll {
		debug!("Creating new epoll instance");

		let event_buffer = Vec::from_fn(buffer_size,|_| {EPollEvent{events:0 , data:0}});

		return EPoll {
			handle : unsafe {epoll_create1(flags as i32)},
			event_buffer : event_buffer
		}
	}

	pub fn add(& self, fd: i32, event: &mut EPollEvent) -> Result<(),()> {
		let result = unsafe {epoll_ctl(self.handle, CtlAdd as i32, fd, event)};

		if result == -1 {
			error!("Unable to add event ({}) to epoll instance ({})", event, self);
			return Err(());
		} else {
			debug!("Added new event ({}) to epoll instance ({})", event, self);
			return Ok(());
		}
	}

	pub fn poll(& mut self, timeout: i32) -> Result<&[EPollEvent],()> {
		let result = unsafe {epoll_wait(self.handle, self.event_buffer.as_mut_ptr(), self.event_buffer.len() as libc::c_int, timeout as libc::c_int)};

		if result == -1 {
			error!("Error polling ({})", self);
			errors::print_error("Error polling");
			return Err(());
		} else {
			return Ok(self.event_buffer.as_slice().slice(0, result as uint))
		}
	}
}

impl EPollEvent {
pub fn new(data: u64, events: &[EPollEventType]) -> EPollEvent {
	let mut event_mask : u32 = 0;

	for event in events.iter() {
		event_mask = event_mask | (*event as u32);
	}

	return EPollEvent {
	events : event_mask,
	data : data
	}
}
}

impl Drop for EPoll {
fn drop(&mut self) {
	unsafe {
			close(self.handle);
	}
}
}
