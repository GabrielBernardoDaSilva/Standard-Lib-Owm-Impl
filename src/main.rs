
mod standard;

use standard::vec_std::Vec;


fn main() {
    let mut v = Vec::<i32>::new();
    v.push(5);
    v.push(6);
    v.push(7);

    for value in v {
        
        println!("{}", value)
    }

}
