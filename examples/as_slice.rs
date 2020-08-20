use const_experiments::vec::Vec;

fn main() {
    let mut v: Vec<u8, 4> = Vec::new();
    v.push(1);
    let s = v.as_mut_slice();
    println!("{:?}, s.len() {}", s, s.len());
    s[0] = 7;
    println!("{:?}, s.len() {}", s, s.len());
}