use std::alloc::{self, Layout};
use std::fmt::Display;
use std::marker::PhantomData;
use std::mem;
use std::ptr::{self, NonNull};

type Link<T> = Option<NonNull<Node<T>>>;

pub struct Node<T> {
    next: Link<T>,
    elem: T,
}

pub struct LinkedList<T> {
    start: Link<T>,
    end: Link<T>,
    len: usize,
    _phatom: PhantomData<T>,
}

struct LinkedListIter<T> {
    actual_node: NonNull<Node<T>>,
}

unsafe impl<T> Send for LinkedList<T> where T: Display {}
unsafe impl<T> Sync for LinkedList<T> where T: Display {}

pub struct IntoIter<T> {
    _list: LinkedList<T>,
    iter: LinkedListIter<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            start: Some(NonNull::dangling()),
            end: Some(NonNull::dangling()),
            len: 0,
            _phatom: PhantomData,
        }
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            let layout = Layout::new::<Node<T>>();
            let n = Node {
                elem,
                next: Some(NonNull::dangling()),
            };

            let ptr_node = alloc::alloc(layout);

            ptr::write(ptr_node as *mut Node<T>, n);

            if self.len == 0 {
                let non_null_node_ptr = NonNull::new(ptr_node as *mut Node<T>);
                self.start = non_null_node_ptr;
                self.end = non_null_node_ptr;
            } else {
                let non_null_node_ptr = NonNull::new(ptr_node as *mut Node<T>);
                if let Some(old_back) = self.end {
                    (*(old_back.as_ptr())).next = non_null_node_ptr;
                    self.end = non_null_node_ptr;
                }
            }

            self.len += 1;
        };
    }

    pub fn pop(&mut self) -> Option<T> {
        if let Some(last) = self.end {
            unsafe {
                let layout = Layout::new::<Node<T>>();

                let result = ptr::read(last.as_ptr()).elem;
                self.len -= 1;
                if self.len == 0 {
                    self.start = Some(NonNull::dangling());
                    self.end = Some(NonNull::dangling());
                } else {
                    let new_last = self.check().unwrap();
                    (*(new_last.as_ptr())).next = Some(NonNull::dangling());
                    self.end = Some(new_last);
                }
                alloc::dealloc(last.as_ptr() as *mut u8, layout);

                return Some(result);
            }
        }
        None
    }

    pub fn insert(&mut self, mut pos: usize, elem: T) {
        assert!(pos <= self.len, "Out of bound");
        if self.len > 0 && pos <= self.len {
            unsafe {
                let layout = Layout::new::<Node<T>>();

                if pos == 0 {
                    //here we have to change the start of the linked-list-unsafe
                    let old_start = self.start;
                    let mut new_ptr = alloc::alloc(layout) as *mut Node<T>;
                    (*(new_ptr)).elem = elem;
                    (*(new_ptr)).next = old_start;
                    self.start = NonNull::new(new_ptr);
                } else {
                    let mut dest = self.start.unwrap();
                    let is_the_last_item = if pos == self.len { true } else { false };

                    while pos > 1 {
                        dest = (*(dest.as_ptr())).next.unwrap();
                        pos -= 1;
                    }
                    let next_ptr = (*(dest.as_ptr())).next;

                    let mut new_ptr = alloc::alloc(layout) as *mut Node<T>;
                    let non_null_new_ptr = NonNull::new(new_ptr);
                    (*(new_ptr)).elem = elem;
                    (*(new_ptr)).next = if is_the_last_item {
                        self.end = non_null_new_ptr;
                        Some(NonNull::dangling())
                    } else {
                        next_ptr
                    };
                    (*(dest.as_ptr())).next = non_null_new_ptr
                }
            }
        }
    }

    pub fn remove_front(&mut self) {
        if self.len > 0 {
            unsafe {
                let layout = Layout::new::<Node<T>>();

                //here we have to change the start of the linked-list-unsafe
                let old_start = self.start;
                self.start = (*(old_start.unwrap().as_ptr())).next;

                alloc::dealloc(old_start.unwrap().as_ptr() as *mut u8, layout);
            }
        }
    }

    fn check(&self) -> Option<NonNull<Node<T>>> {
        let mut start = self.start.unwrap();
        let end = self.end.unwrap();
        unsafe {
            while start != NonNull::dangling() && end != (*(start.as_ptr())).next.unwrap() {
                start = (*(start.as_ptr())).next.unwrap();
            }
        }
        if start == NonNull::dangling() {
            return None;
        }
        Some(start)
    }

    pub fn last(&self) -> Option<&T> {
        if let Some(v) = self.end {
            if v != NonNull::dangling() {
                unsafe { return Some(&(*(v.as_ptr())).elem) }
            }
            return None;
        }
        None
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.len > 0 {
            self.pop();
        }
    }
}

impl<T> LinkedListIter<T> {
    pub unsafe fn new(list: &LinkedList<T>) -> Self {
        Self {
            actual_node: list.start.unwrap(),
        }
    }
}

impl<T> Iterator for LinkedListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.actual_node == NonNull::dangling() {
                return None;
            }
            let old_ptr = self.actual_node;
            self.actual_node = (*(self.actual_node.as_ptr())).next.unwrap();
            Some(ptr::read(&(*(old_ptr.as_ptr())).elem))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let elem_size = mem::size_of::<T>();
        (elem_size, Some(elem_size))
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let iter = LinkedListIter::new(&self);
            let _list = ptr::read(&self);

            mem::forget(self);
            IntoIter { iter, _list }
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}


impl<T> Drop for IntoIter<T>{
    fn drop(&mut self) {
           for _ in &mut *self {}
    }
}
