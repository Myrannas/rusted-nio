use libc::types::os::common::bsd44::{ifaddrs, sockaddr, sockaddr_in, in_addr};
use libc::funcs::bsd43::{getifaddrs,freeifaddrs,socket,bind,listen,accept};
use std::ptr;
use libc;

//OS
use libc::consts::os::bsd44::{AF_INET,AF_INET6,SOCK_STREAM};
use libc::consts::os::extra::O_NONBLOCK;

//POSIX88
use libc::funcs::posix88::unistd::close;
use libc::funcs::posix88::fcntl::{fcntl};
use libc::consts::os::posix88::{EWOULDBLOCK,EAGAIN};

use libc::consts::os::posix01::{F_GETFL,F_SETFL};

use std::num::FromPrimitive;
use std::mem::{size_of,transmute};
use std::os::errno;

use errors;

extern {
	fn htonl(host : u32 ) -> u32;
	fn htons(host : u16 ) -> u16;
}

#[deriving(Eq, PartialEq, Show)]
pub enum Address {
	IPv4(u8, u8, u8, u8, u16)
}


pub fn localhost(port:u16) -> Address { IPv4(127,0,0,1, port) }

impl Address {
	pub fn to_addr(&self) -> sockaddr {
        match *self {
			IPv4(a1, a2, a3, a4, port) => {
		        let socket_address = sockaddr_in {
			        sin_family : AF_INET as u16,
			        sin_addr : in_addr {
			            s_addr : unsafe{htonl(a1 as u32 << 24 | a2 as u32 << 16 | a3 as u32 << 8 | a4 as u32)}
			        },
			        sin_port : unsafe{htons(port)},
			        sin_zero : [0, 0, 0, 0, 0, 0, 0, 0, ]
		        };

		        return unsafe{transmute(socket_address)};
	        }
		}
	}

	pub fn to_family(&self) -> i32 {
		match *self {
			IPv4(_,_,_,_,_) => AF_INET
		}
	}
}

//pub struct Address {
//	addr : *const sockaddr,
//}
//
//impl Address{
//	pub fn protocol(&self) -> Option<Protocol> {
//		unsafe {
//			return FromPrimitive::from_u16((*self.addr).sa_family);
//		}
//	}
//}
//
//pub struct AddressInformation {
//	addresses: Vec<Address>,
//	ptr: *mut ifaddrs
//}
//
//impl Drop for AddressInformation {
//	fn drop(&mut self) {
//		unsafe {
//			freeifaddrs(self.ptr);
//		}
//	}
//}

//impl AddressInformation {
//	pub fn new() -> Result<AddressInformation, ()> {
//		unsafe {
//			let mut addr : *mut ifaddrs = ptr::null_mut();
//
//			let result = getifaddrs(&mut addr);
//
//			if result == -1 {
//				return Err(());
//			}
//
//			let mut currentAddress = addr;
//			let mut addresses = Vec::new();
//
//			while currentAddress != ptr::null_mut() {
//				addresses.push(Address {addr: & (*(*currentAddress).ifa_addr)});
//
//				currentAddress = (*currentAddress).ifa_next;
//			}
//
//
//
//			Ok(AddressInformation { addresses: addresses, ptr : addr })
//		}
//	}
//
////	pub fn with_protocol(&mut self, protocol: Protocol) {
////		self.addresses = self.addresses
////						.into_iter()
////						.filter(|a| { a.protocol().unwrap_or(Invalid) == protocol })
////						.collect::<Vec<Address>>();
////	}
//}

#[deriving(Show)]
pub struct RemoteSocket {
	pub handle: libc::c_int,
//	pub addr: Address
}

impl RemoteSocket {
	pub fn make_non_blocking(& self) -> Result<(), ()> {

		let mut flags = unsafe {fcntl (self.handle, F_GETFL, 0u32)};

		if flags == -1 {
			error!("Error encountered while modifying socket mode to be non-blocking");
			return Err(());
		}

		flags |= O_NONBLOCK;

		if unsafe {fcntl (self.handle, F_SETFL, flags)} == -1 {
			error!("Error encountered while modifying socket mode to be non-blocking");
			return Err(());
		}

		return Ok(())
	}
}

impl Drop for RemoteSocket {
fn drop(&mut self) {
	unsafe {
		debug!("Closing remote socket ({})", self);
		close(self.handle);
	}
}
}

#[deriving(Show)]
pub struct ServerSocket {
	pub handle: libc::c_int
}

impl ServerSocket {
	pub fn new(address: &Address) -> Result<ServerSocket, ()> {

			debug!("Attempting to create socket descriptor");

			let fd = unsafe {socket(address.to_family(), SOCK_STREAM | O_NONBLOCK, 0)};

			let socket = ServerSocket{handle: fd};

			if fd != -1 {
				debug!("Binding to socket ({}) on with address ({})", socket, address);

				let addr = address.to_addr();
				let result = unsafe {bind(fd, &addr, size_of::<sockaddr>() as u32)};

				return Ok(socket)
			} else {
				error!("Error creating socket descriptor - errno {}", fd);

				return Err(())
			}
	}

	pub fn listen(&self) -> Result<(),()> {
		unsafe {
			if listen(self.handle, 128) != -1 {
				debug!("Listening to socket ({})", self);
				Ok(())
			} else {
				Err(())
			}
		}
	}

	pub fn accept(&self) -> Result<RemoteSocket,()> {
		let mut addr = IPv4(0,0,0,0,0).to_addr();
		let mut len = size_of::<sockaddr>() as u32;

		let result = unsafe {accept(self.handle, &mut addr, &mut len)};

		if result == -1 {
			if errno() == EAGAIN as int || errno() == EWOULDBLOCK as int {
				Err(())
			}  else {
				debug!("Error accepting socket ({})", self);
				errors::print_error("Error accepting socket");
				Err(())
			}
		} else {
			let remote = RemoteSocket{ handle: result };
			remote.make_non_blocking();

			Ok( remote )
		}
	}
}

impl Drop for ServerSocket {
	fn drop(&mut self) {
		unsafe {
			debug!("Closing socket ({})", self);
			close(self.handle);
		}
	}
}
