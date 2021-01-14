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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_cow_arc(){
        {
            let test_str = "Test string";
            let new_val_str = "New value";

            let source = CowArc::new(test_str.to_owned());
            let cloned = source.clone();
            let mut changed = cloned.clone();
            changed.set_val(new_val_str.to_owned());
            let changed_cloned = changed.clone();
    
            let source_ptr: &String = source.deref();
            let cloned_ptr: &String = cloned.deref();
            let changed_ptr: &String = changed.deref();
            let changed_cloned_ptr: &String = changed_cloned.deref();
    
            assert!(std::ptr::eq(source_ptr, cloned_ptr));
            assert!(std::ptr::eq(source_ptr, changed_ptr) == false);
            assert!(std::ptr::eq(changed_ptr, changed_cloned_ptr));
            assert!(cloned_ptr.eq(test_str));
            assert!(changed.eq(new_val_str));
        }

        {
            let source = CowArc::new(vec![1, 2, 3]);
            let cloned = source.clone();
            let mut changed = cloned.clone();
            changed.set_val(vec![1, 2, 3, 4]);
            let changed_cloned = changed.clone();
            let mut updated = changed_cloned.clone();
            updated.update_val(|val|{
                val.push(5);
            });

            let source_ptr: &Vec<i32> = source.deref();
            let cloned_ptr: &Vec<i32> = cloned.deref();
            let changed_ptr: &Vec<i32> = changed.deref();
            let changed_cloned_ptr: &Vec<i32> = changed_cloned.deref();
            let updated_ptr: &Vec<i32> = updated.deref();

            assert!(std::ptr::eq(source_ptr, cloned_ptr));
            assert!(std::ptr::eq(source_ptr, changed_ptr) == false);
            assert!(std::ptr::eq(changed_ptr, changed_cloned_ptr));
            assert!(std::ptr::eq(changed_ptr, updated_ptr) == false);
            assert!(changed.eq(&vec![1, 2, 3, 4]));
            assert!(updated.eq(&vec![1, 2, 3, 4, 5]));
        }
    }
}