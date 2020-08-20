use const_experiments::vec::Vec;

static mut V: Vec<u8, 8> = Vec::new();

fn main() {
    unsafe {
        assert!(V.len() == 0);
        let _ = V.push(1);
        assert!(V.len() == 1)
    }
}
