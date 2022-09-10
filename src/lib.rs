use std::collections::HashMap;

#[derive(Default)]
pub struct Fidd {
	items: Vec<Item>,
}

enum Item {
	New(Vec<u8>),
	FromSrc(usize, usize), // src line idx, len
}

impl Fidd {
	pub fn len(&self) -> usize {
		self.items.len()
	}

	pub fn is_empty(&self) -> bool {
		self.items.is_empty()
	}

	// TODO: fast mode(src_idx step window)
	pub fn new(src: &[Vec<u8>], dst: &[Vec<u8>]) -> Self {
		eprintln!("{} {}", src.len(), dst.len());
		let mut pairmap = HashMap::new();
		if dst.is_empty() {
			return Self::default()
		}
		for dst_idx in 0..dst.len() - 1 {
			let e = pairmap.entry([&dst[dst_idx], &dst[dst_idx + 1]]).or_insert(Vec::new());
			e.push(dst_idx);
		}
		let mut segments = Vec::new();
		for src_idx in 0..src.len() - 1 {
			eprintln!("{}", src_idx);
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
		segments.sort_unstable();
		let mut segment_cursor = 0;
		let mut items = Vec::new();
		let mut dst_cursor = 0;
		loop {
			// update segments cursor
			let mut exit_inner_loop = false;
			let finish_flag = 'inner: loop {
				if segment_cursor >= segments.len() {
					for dst_idx in dst_cursor..dst.len() {
						items.push(Item::New(dst[dst_idx].clone()));
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
				items.push(Item::New(dst[dst_cursor].clone()));
				dst_cursor += 1
			} else {
				let segment = &segments[farthest_idx];
				items.push(Item::FromSrc(segment[2], segment[3]));
				dst_cursor = segment[1];
			}
		}
		Self {items}
	}

	pub fn apply(&self, src: &[Vec<u8>]) -> Vec<Vec<u8>> {
		let mut result = Vec::new();
		for item in self.items.iter() {
			match item {
				Item::New(line) => result.push(line.clone()),
				Item::FromSrc(idx, len) => {
					for offset in 0..*len {
						result.push(src[idx + offset].clone());
					}
				}
			}
		}
		result
	}
}
