use std::collections::HashMap;
use std::io::{BufReader, BufRead, BufWriter, Write};

#[derive(Default)]
pub struct Fidd {
	items: Vec<Item>,
}

enum Item {
	New(Vec<Vec<u8>>),
	FromSrc(usize, usize), // src line idx, len
}

impl Fidd {
	pub fn len(&self) -> usize {
		self.items.len()
	}

	pub fn dst_len(&self) -> usize {
		let mut result = 0;
		for item in self.items.iter() {
			match item {
				Item::New(x) => result += x.len(),
				Item::FromSrc(_, len) => result += len,
			}
		}
		return result
	}

	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	// TODO: fast mode(src_idx step window)
	pub fn new(src: &[Vec<u8>], dst: &[Vec<u8>]) -> Self {
		let mut pairmap = HashMap::new();
		if dst.is_empty() {
			return Self::default()
		}

		// build pairmap
		for dst_idx in 0..dst.len() - 1 {
			let e = pairmap.entry([&dst[dst_idx], &dst[dst_idx + 1]]).or_insert(Vec::new());
			e.push(dst_idx);
		}

		// build segments mapping
		let mut segments = Vec::new();
		for src_idx in 0..src.len() - 1 {
			eprint!("[2K{}/{}\r", src_idx, src.len());
			let mut last_indices = if let Some(indices)
				= pairmap.get(&[&src[src_idx], &src[src_idx + 1]])
			{
				indices.clone()
			} else {
				continue
			};
			let mut window = 2;
			loop {
				let mut new_indices = Vec::new();
				if src_idx + window >= src.len() {
					break
				}
				for dst_idx in last_indices.clone().into_iter() {
					if dst_idx + window >= dst.len() {
						continue
					}
					if src[src_idx + window] == dst[dst_idx + window] {
						new_indices.push(dst_idx);
					}
				}
				if new_indices.is_empty() {
					break
				}
				last_indices = new_indices;
				window += 1;
			}
			let window = window - 1;
			if window < 2 { continue }
			for dst_idx in last_indices.into_iter() {
				let dst_end = dst_idx + window;
				segments.push([dst_idx, dst_end, src_idx, window]);
			}
		}
		eprintln!();
		segments.sort_unstable();

		// incremental matching
		let mut segment_cursor = 0;
		let mut items = Vec::new();
		let mut dst_cursor = 0;
		loop {
			// update segments cursor
			let mut exit_inner_loop = false;
			let finish_flag = 'inner: loop {
				if segment_cursor >= segments.len() {
					let mut new = Vec::new();
					for dst_idx in dst_cursor..dst.len() {
						new.push(dst[dst_idx].clone());
					}
					if !new.is_empty() {
						items.push(Item::New(new));
					}
					break 'inner true
				}
				if exit_inner_loop { break 'inner false }
				let seg = &segments[segment_cursor];
				if seg[1] < dst_cursor {
					segment_cursor += 1;
				} else {
					exit_inner_loop = true;
				}
			};
			if finish_flag { break }

			// find farthest cover segment
			let mut farthest = 0;
			let mut farthest_idx = 0;
			for segment_idx in segment_cursor..segments.len() {
				let segment = &segments[segment_idx];
				if segment[0] > dst_cursor {
					break
				}
				if segment[1] > farthest {
					farthest = segment[1];
					farthest_idx = segment_idx;
				}
			}
			if farthest <= dst_cursor + 1 {
				match items.last_mut() {
					Some(Item::New(x)) => x.push(dst[dst_cursor].clone()),
					_ => {
						let new = vec![dst[dst_cursor].clone()];
						items.push(Item::New(new));
					},
				}
				dst_cursor += 1
			} else {
				let segment = &segments[farthest_idx];
				let offset = dst_cursor - segment[0];
				items.push(Item::FromSrc(segment[2] + offset, segment[3] - offset));
				dst_cursor = segment[1];
			}
		}
		Self {items}
	}

	pub fn apply(&self, src: &[Vec<u8>]) -> Vec<Vec<u8>> {
		let mut result = Vec::new();
		for item in self.items.iter() {
			match item {
				Item::New(lines) => for line in lines.iter() {
					result.push(line.clone());
				}
				Item::FromSrc(idx, len) => {
					for offset in 0..*len {
						result.push(src[idx + offset].clone());
					}
				}
			}
		}
		result
	}

	pub fn save(&self, file: &str) -> Result<(), std::io::Error> {
		let f = std::fs::File::create(file)?;
		let mut f = BufWriter::new(f);
		for item in self.items.iter() {
			match item {
				Item::New(lines) => {
					writeln!(f, "new {}", lines.len())?;
					for line in lines.iter() {
						f.write(line)?;
						writeln!(f)?;
					}
				},
				Item::FromSrc(idx, len) => {
					writeln!(f, "src {} {}", idx, len)?;
				}
			}
		}
		Ok(())
	}

	pub fn load(file: &str) -> Result<Self, std::io::Error> {
		let f = std::fs::File::open(file)?;
		let mut f = BufReader::new(f);
		let mut linebuf = Vec::new();
		let mut items = Vec::new();
		loop {
			let buflen = f.read_until(b'\n', &mut linebuf)?;
			if buflen == 0 { break }
			assert_eq!(linebuf[buflen - 1], b'\n');
			linebuf.pop();
			let string = String::from_utf8(std::mem::take(&mut linebuf)).unwrap();
			let mut iter = string.split_whitespace();
			match iter.next() {
				Some("src") => {
					let idx = iter.next().unwrap().parse::<usize>().unwrap();
					let len = iter.next().unwrap().parse::<usize>().unwrap();
					let item = Item::FromSrc(idx, len);
					items.push(item);
				}
				Some("new") => {
					let line_count = iter.next().unwrap().parse::<usize>().unwrap();
					let mut new = Vec::new();
					for _ in 0..line_count {
						let buflen = f.read_until(b'\n', &mut linebuf)?;
						assert_eq!(linebuf[buflen - 1], b'\n');
						linebuf.pop();
						new.push(std::mem::take(&mut linebuf));
					}
					let item = Item::New(new);
					items.push(item);
				}
				_ => panic!("Unknown description line: {}", string)
			}
		}
		Ok(Self {items})
	}
}

// TODO test: case 12121212
