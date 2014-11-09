#![feature(phase)]

extern crate libc;
extern crate core;
extern crate native;

#[phase(plugin, link)]
extern crate log;

mod errors;
mod epoll;
mod sockets;
mod signals;

fn main() {
	let x = 5i;
	signals::init();

//	sockets::bind(8080);
	let addr = sockets::ServerSocket::new(& sockets::localhost(8080));

	if addr.is_ok() {
		let socket = addr.ok().unwrap();
		socket.listen();

		{
		let mut epoll = epoll::EPoll::new(0, 1);
		let mut event = epoll::EPollEvent::new(socket.handle as u64, [epoll::EPollEventType::EPollIn, epoll::EPollEventType::EPollEt]);
		let result = epoll.add(socket.handle, &mut event);

		let mut clients = Vec::new();

		loop {
				let last_signal = signals::last_signal();
				if last_signal == signals::SignalInterrupt || last_signal == signals::SignalQuit {
					return;
				}

				let events = epoll.poll(1000);

				match events {
						Ok(events) => {
						if (events.len() > 0) {
							debug!("Received {} events", events.len())

							for event in events.iter() {
								if (event.data as i32 == socket.handle) {
									//Listening port

									loop {
										let client = socket.accept();

										if (client.is_err()) {
											break;
										} else {
											let newClient = client.ok().unwrap();
											debug!("Accepted client {}", newClient);

											clients.push(newClient);
										}
									}
								}
							}
						}
					}
						_ => {
						error!("Encountered error polling epoll instance")
					}
				}
			}
		}
	}
}