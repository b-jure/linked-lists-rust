use std::ptr::{self, NonNull};
use std::mem;
use std::marker::PhantomData;

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T
}

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _marker: PhantomData<T>
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList { 
            front: None, 
            back: None, 
            len: 0 ,
            _marker: PhantomData
        }
    }

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None,
                back: None,
                elem
            })));

            if let Some(old) = self.back {
                (*old.as_ptr()).back = Some(new);
                (*new.as_ptr()).front = Some(old);
            }else {
                self.front = Some(new);
            }

            self.back = Some(new);
        }

        self.len += 1;
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None,
                back: None,
                elem
            })));

            if let Some(old) = self.front {
                (*old.as_ptr()).front = Some(new);
                (*new.as_ptr()).back = Some(old);
            }else {
                self.back = Some(new);
            }

            self.front = Some(new);
        }

        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.back.map(|node| { 
                let old_back = Box::from_raw(node.as_ptr());

                self.back = old_back.front;

                if let Some(new) = self.back {
                    (*new.as_ptr()).back = None;
                }else {
                    self.front = None;
                }
                self.len -= 1;
                old_back.elem
            })
        }
    }
    
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|node| { 
                let old_front = Box::from_raw(node.as_ptr());

                self.front = old_front.back;

                if let Some(new) = self.front {
                    (*new.as_ptr()).front = None;
                }else {
                    self.back = None;
                }
                self.len -= 1;
                old_front.elem
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { 
            Some(&(*self.front?.as_ptr()).elem)
        }
    }
    
    pub fn back(&self) -> Option<&T> {
        unsafe { 
            Some(&(*self.back?.as_ptr()).elem)
        }
    }
    
    pub fn front_mut(&self) -> Option<&mut T> {
        unsafe {
            self.front.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn back_mut(&self) -> Option<&mut T> {
        unsafe {
            self.back.map(|node| &mut (*node.as_ptr()).elem)
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _marker: PhantomData<&'a T>
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.back = (*node.as_ptr()).front;
                self.len -= 1;
                &(*node.as_ptr()).elem
            })
        }else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|node| unsafe {
                self.front = (*node.as_ptr()).back;
                self.len -= 1;
                &(*node.as_ptr()).elem
            })
        }else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IntoIterator<T> {
    list: LinkedList<T>
}

impl<T> Iterator for IntoIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIterator<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

impl<T> ExactSizeIterator for IntoIterator<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _marker: PhantomData<&'a mut T>
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.back = (*node.as_ptr()).front;
                self.len -= 1;
                &mut (*node.as_ptr()).elem
            })
        }else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.back = (*node.as_ptr()).front;
                self.len -= 1;
                &mut (*node.as_ptr()).elem
            })
        }else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}


#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front_production_queue_unsafe() {
        let mut list = LinkedList::new();

        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}