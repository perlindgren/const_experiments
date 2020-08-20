use const_experiments::vec::Vec;

#[derive(Debug)]
struct D {}

impl Drop for D {
    fn drop(&mut self) {
        println!("D:drop {:?}", self);
    }
}
fn main() {
    {
        let mut v: Vec<D, 4> = Vec::new();
        v.push(D {});
        v.push(D {});
        // v will be dropped here
    }
    println!("here");
}
