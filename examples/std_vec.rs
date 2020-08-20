use std::vec::Vec;

#[derive(Debug)]
struct D {}

impl Drop for D {
    fn drop(&mut self) {
        println!("D:drop {:?}", self);
    }
}

fn main() {
    let mut v: Vec<D> = Vec::new();
    v.push(D {});
    v.push(D {});
    drop(v);
    println!("here");
}
