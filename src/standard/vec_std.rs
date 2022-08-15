
use std::marker::PhantomData;
use std::ptr::{self};
use std::ops::{Deref, DerefMut };
use std::mem;
use super::raw_vec::{RawVec, RawValIter};
use super::drain_vec::Drain;


pub struct Vec<T> {
    buf: RawVec<T>,
    len: usize,
}

pub struct IntoIter<T>{
    _buf: RawVec<T>,
    iter: RawValIter<T>
}


impl<T> Vec<T> {

    fn ptr(&self) -> *mut T {
        self.buf.ptr.as_ptr()
    }

    fn cap(&self) -> usize {
        self.buf.cap
    }

    pub fn new() -> Self{
        Vec {
            buf: RawVec::new(),
            len: 0
        }
    }

   
    pub fn push(&mut self, elem: T) {
        if self.len == self.cap() {
            self.buf.grow();
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) {
        assert!(index <= self.len, "index out of bounds");
        if self.cap() == self.len {
            self.buf.grow();
        }

        unsafe{
            ptr::copy(self.ptr().add(index),
                      self.ptr().add(index + 1), 
                      self.len - index);
            ptr::write(self.ptr().add(index), elem);
            self.len += 1;
        }
    }

    pub fn remove(&mut self, index: usize) -> T{
        assert!(index < self.len, "index out of bounds");
        unsafe {
            self.len -= 1;
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(self.ptr().add(index + 1),
                      self.ptr().add(index), 
                      self.len - index);
            result
        }
    }

    pub fn drain(&mut self) -> Drain<T>{
        unsafe{
            let iter = RawValIter::new(&self);
            self.len = 0;

            Drain{
                iter,
                vec: PhantomData,
            }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        if self.cap() != 0 {
            while let Some(_) = self.pop() {}
        }
    }
}


impl<T> Deref for Vec<T>{
    type Target = [T];
    fn deref(&self) -> &Self::Target {
          unsafe{
              std::slice::from_raw_parts(self.ptr(), self.len)
          }
    } 
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe{
            std::slice::from_raw_parts_mut(self.ptr(), self.len)
        }
    }
}


impl<T> IntoIterator for Vec<T>{
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            let iter = RawValIter::new(&self);

            let buf = ptr::read(&self.buf);

            mem::forget(self);
            
            IntoIter {
                iter,
                _buf: buf,
            }
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


impl<T> DoubleEndedIterator for IntoIter<T>{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back()  
    }
}


impl<T> Drop for IntoIter<T>{
    fn drop(&mut self) {
           for _ in &mut *self {}
    }
}
