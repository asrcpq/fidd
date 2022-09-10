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
	pub fn new(src: &[Vec<u8>], dst: &[Vec<u8>]) -> Self {
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
			let mut indices = if let Some(indices)
				= pairmap.get(&[&src[src_idx], &src[src_idx + 1]])
			{
				indices.clone()
			} else {
				continue
			};
			let mut window = 2;
			loop {
				for dst_idx in std::mem::take(&mut indices).into_iter() {
					if src_idx + window >= src.len() {
						break
					}
					if dst_idx + window >= dst.len() {
						continue
					}
					if src[src_idx + window] == dst[dst_idx + window] {
						indices.push(dst_idx);
					}
				}
				if indices.is_empty() {
					break
				}
				window += 1;
			}
			let window = window - 1;
			if window < 2 { continue }
			for dst_idx in indices.into_iter() {
				segments.push([dst_idx, window, src_idx]);
			}
		}
		segments.sort_unstable();
		let mut segments_idx = 0;
		let mut items = Vec::new();
		for dst_idx in 0..dst.len() {
			if segments_idx >= segments.len() ||
				segments[segments_idx][0] > dst_idx
			{
				items.push(Item::New(dst[dst_idx].clone()));
				continue
			}
			assert!(segments[segments_idx][0] == dst_idx);
			while segments_idx < segments.len() {
				if segments[segments_idx][0] == dst_idx {
					segments_idx += 1;
				} else {
					break
				}
			}
			let seg = &segments[segments_idx - 1];
			let item = Item::FromSrc(seg[2], seg[1]);
			items.push(item);
		}
		Self {items}
	}
}
