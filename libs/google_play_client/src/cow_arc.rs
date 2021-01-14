use std::{
    sync::{
        Arc
    },
    ops::{
        Deref
    }
};

#[derive(Debug)]
pub struct CowArc<T: Clone>{
    inner: Arc<T>
}
impl<T: Clone> CowArc<T> {
    pub fn new(val: T) -> CowArc<T>{
        CowArc{
            inner: Arc::new(val)
        }
    }
    pub fn set_val(&mut self, val: T){
        self.inner = Arc::new(val);
    }
    pub fn update_val<F: FnOnce(&mut T)>(&mut self, f: F) {
        let mut v: T = self.inner.deref().clone();
        f(&mut v);
        self.inner = Arc::new(v);
    }
}
impl<T: Clone> Deref for CowArc<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl<T: Clone> Clone for CowArc<T>{
    fn clone(&self) -> Self {
        CowArc{
            inner: self.inner.clone()
        }   
    }
}