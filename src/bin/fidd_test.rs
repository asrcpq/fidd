use fidd::Fidd;

fn main() {
	let mut lines1 = Vec::new();
	for i in 0..100u8 {
		for j in i..200u8 {
			lines1.push(vec![j]);
		}
	}
	let mut lines2 = Vec::new();
	for i in (0..100u8).rev() {
		for j in i..200u8 {
			lines2.push(vec![j]);
		}
	}
	let fidd = Fidd::new(&lines1, &lines2);
	assert_eq!(fidd.dst_len(), lines2.len());
	eprintln!("{}", fidd.len());
	fidd.save("/tmp/fidd_test.fidd").unwrap();
	let fidd = Fidd::load("/tmp/fidd_test.fidd").unwrap();
	let lines3 = fidd.apply(&lines1);
	assert_eq!(lines2.len(), lines3.len());
}