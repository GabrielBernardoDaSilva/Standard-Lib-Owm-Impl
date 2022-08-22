#![allow(dead_code)]

use standard::linked_list_std::LinkedList;

mod standard;

fn main() {
    let mut list = LinkedList::<i32>::new();
    list.push(5);
   
    list.push(6);
    list.push(7);
    list.push(8);
    list.insert(0, 9);
    list.remove_front();
    

    for i in list.into_iter() {
        println!("it: {}", i);
    }
}

