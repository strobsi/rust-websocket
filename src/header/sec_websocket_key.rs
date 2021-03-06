use std::mem;
use std::str::FromStr;

use base64;
use http::header::HeaderValue;
//use hyper::header::parsing::from_one_raw_str;
use std::fmt::{self, Debug};
use rand;
use result::{WebSocketResult, WebSocketError};

/// Represents a Sec-WebSocket-Key header.
#[derive(PartialEq, Clone, Copy, Default)]
pub struct WebSocketKey(pub [u8; 16]);

impl Debug for WebSocketKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "WebSocketKey({})", self)
	}
}

impl FromStr for WebSocketKey {
	type Err = WebSocketError;

	fn from_str(key: &str) -> WebSocketResult<WebSocketKey> {
		match base64::decode(key) {
			Ok(vec) => {
				if vec.len() != 16 {
					return Err(WebSocketError::ProtocolError(
						"Sec-WebSocket-Key must be 16 bytes",
					));
				}
				let mut array = [0u8; 16];
				let mut iter = vec.into_iter();
				for i in &mut array {
					*i = iter.next().unwrap();
				}

				Ok(WebSocketKey(array))
			}
			Err(_) => Err(WebSocketError::ProtocolError(
				"Invalid Sec-WebSocket-Accept",
			)),
		}
	}
}

impl WebSocketKey {
	pub const HEADER_NAME: &'static str = "Sec-WebSocket-Key";
	/// Generate a new, random WebSocketKey
	pub fn new() -> WebSocketKey {
		let key: [u8; 16] = unsafe {
			// Much faster than calling random()q several times
			mem::transmute(rand::random::<(u64, u64)>())
		};
		WebSocketKey(key)
	}
}

impl From<WebSocketKey> for HeaderValue {
	fn from(key: WebSocketKey) -> Self {
		HeaderValue::from_str(&format!("{}", key)).unwrap()
	}
}

// impl TryFrom<HeaderValue> for WebSocketKey {
// 	type Error = ();
// 	fn try_from(value: HeaderValue) -> Result<WebSocketKey, ()> {
// 		Ok(WebSocketKey([0u8; 16]))
// 	}
// }

impl fmt::Display for WebSocketKey {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let WebSocketKey(accept) = *self;
		write!(f, "{}", base64::encode(&accept))
	}
}

#[cfg(all(feature = "nightly", test))]
mod tests {
	use super::*;
	use http::header::HeaderMap;
	use test;
	#[test]
	fn test_header_key() {
		use header::Headers;

		let extensions = WebSocketKey([65; 16]);
		let mut headers = HeaderMap::new();
		headers.insert(WebSocketKey::HEADER_NAME, HeaderValue::from(extensions));

		assert_eq!(
			&headers.to_string()[..],
			"Sec-WebSocket-Key: QUFBQUFBQUFBQUFBQUFBQQ==\r\n"
		);
	}
	#[bench]
	fn bench_header_key_new(b: &mut test::Bencher) {
		b.iter(|| {
			let mut key = WebSocketKey::new();
			test::black_box(&mut key);
		});
	}
	#[bench]
	fn bench_header_key_parse(b: &mut test::Bencher) {
		let value = vec![b"QUFBQUFBQUFBQUFBQUFBQQ==".to_vec()];
		b.iter(|| {
			let mut key: WebSocketKey = Header::parse_header(&value[..]).unwrap();
			test::black_box(&mut key);
		});
	}
	#[bench]
	fn bench_header_key_format(b: &mut test::Bencher) {
		let value = vec![b"QUFBQUFBQUFBQUFBQUFBQQ==".to_vec()];
		let val = Header::parse_header(&value[..]).unwrap();
		b.iter(|| {
			format!("{}", val.serialize());
		});
	}
}
