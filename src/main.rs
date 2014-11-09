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
	signals::init();

	let addr = sockets::ServerSocket::new(& sockets::localhost(8080));

	if addr.is_ok() {
		let socket = addr.ok().unwrap();
		if socket.listen().is_err() {
			error!("Error listening for socket {}", socket);
			return;
		}

		{
		let mut epoll = epoll::EPoll::new(0, 1);
		let mut event = epoll::EPollEvent::new(socket.handle as u64, [epoll::EPollEventType::EPollIn, epoll::EPollEventType::EPollEt]);
		let result = epoll.add(socket.handle, &mut event);

		if result.is_err() {
			error!("Error binding epoll {} to listen socket {}", epoll, socket);
			return;
		}

		let mut clients = Vec::new();

		loop {
				let last_signal = signals::last_signal();
				if last_signal == signals::SignalInterrupt || last_signal == signals::SignalQuit {
					return;
				}

				let events = epoll.poll(1000);

				match events {
						Ok(events) => {
						if events.len() > 0 {
							debug!("Received {} events", events.len())

							for event in events.iter() {
								if event.data as i32 == socket.handle {
									//Listening port

									loop {
										let client = socket.accept();

										if client.is_err() {
											break;
										} else {
											let new_client = client.ok().unwrap();
											debug!("Accepted client {}", new_client);

											clients.push(new_client);
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