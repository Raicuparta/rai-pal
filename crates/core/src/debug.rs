use std::time::Instant;

pub trait LoggableInstant {
	fn log_next(&mut self, message: &str);
}

impl LoggableInstant for Instant {
	fn log_next(&mut self, message: &str) {
		let millis = self.elapsed().as_millis();
		println!("{millis:10}ms # {message}");
		*self = Self::now();
	}
}
