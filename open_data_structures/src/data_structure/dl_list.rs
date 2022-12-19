use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::interface::clone_list::CloneList;

// type StrongLink<T> = Rc<RefCell<Node<T>>>;
// type WeakLink<T> = Weak<RefCell<Node<T>>>;

#[derive(Debug)]
pub struct Node<T> {
    x: T,
    next: Link<T>,
    prev: WeakLink<T>
}

impl<T: Default> Node<T> {
    fn new() -> Self {
        Self {
            x: T::default(),
            prev: WeakLink::empty(),
            next: Link::empty(),
        }
    }

    fn get_link(self) -> Link<T> {
        Link::new(self)
    }

}

#[derive(Debug)]
pub struct Link<T>(Option<Rc<RefCell<Node<T>>>>);

impl <T> Link<T> {

    fn empty() -> Self {
        Self(None)
    }

    fn new(node: Node<T>) -> Self {
        Link(Some(Rc::new(RefCell::new(node))))
    }

    fn next(&self) -> Option<Self> {
        self.0.map(|rc| rc.as_ref().borrow().next)
    }
}


#[derive(Debug)]
pub struct WeakLink<T>(Option<Weak<RefCell<Node<T>>>>); 

impl <T> WeakLink<T> {

    fn empty() -> Self {
        Self(None)
    }

    fn new(link: &Link<T>) -> Self {
        Self(link.0.map(|rc| Rc::downgrade(&rc)))
    }
}


/*/
pub trait Link<L, T> {
    fn new_link(node: Node<T>) -> L;
    fn set_next(&mut self, next: Option<StrongLink<T>>);
    fn set_prev(&mut self, prev: Option<WeakLink<T>>);
    fn get_prev(&self) -> Option<WeakLink<T>>;
    fn get_next(&self) -> Option<StrongLink<T>>;
}

impl<T: Default> Link<StrongLink<T>, T> for StrongLink<T> {
    fn new_link(node: Node<T>) -> StrongLink<T> {
        Rc::new(RefCell::new(node))
    }

    fn set_next(&mut self, next: Option<StrongLink<T>>) {
        self.borrow_mut().next = next
    }

    fn set_prev(&mut self, prev: Option<WeakLink<T>>) {
        self.borrow_mut().prev = prev
    }

    fn get_prev(&self) -> Option<WeakLink<T>> {
        self.borrow().prev.clone()
    }

    fn get_next(&self) -> Option<StrongLink<T>> {
        self.borrow().next.clone()
    }
}

impl<T: Default> Link<WeakLink<T>, T> for WeakLink<T> {
    
    fn new_link(node: Node<T>) -> WeakLink<T> {
        Rc::downgrade(&Rc::new(RefCell::new(node)))
    }

    fn set_next(&mut self, next: Option<StrongLink<T>>) {
        if let Some(p) = self.upgrade().as_mut() {
            p.set_next(next)
        }
    }

    fn set_prev(&mut self, prev: Option<WeakLink<T>>) {
        if let Some(p) = self.upgrade().as_mut() {
            p.set_prev(prev)
        }
    }

    fn get_next(&self) -> Option<StrongLink<T>> {
        self.upgrade().and_then(|p| p.get_next())
    }

    fn get_prev(&self) -> Option<WeakLink<T>> {
        self.upgrade().and_then(|p| p.get_prev())
    }
}*/
/// 双方向連結リスト
#[derive(Debug)]
pub struct DLList<T> {
    dummy: Link<T>,
    n: usize,
}

impl<T: Default + Clone> DLList<T> {
    pub fn new() -> Self {
        let dummy_node = Node::new();
        let link = dummy_node.get_link();
        dummy_node.prev = WeakLink::new(&link);
        dummy_node.next = link;
        Self { dummy: link, n: 0 }
    }

    pub fn get_link(&self, i: usize) -> Link<T> {
        let mut p: Link<T>;
        if i < self.n / 2 {
            p = self.dummy;
            for _ in 0..i {
                p = p.next();
                if let Some(n) = p.0 {
                    p = n.get_next();
                } else {
                    break;
                }
            }
        } else {
            p = self.dummy.0.clone();
            for _ in (i..self.n).rev() {
                if let Some(n) = p {
                    p = n.get_prev().and_then(|w| w.upgrade());
                } else {
                    break;
                }
            }
        }
        p
    }

    pub fn add_before(&mut self, mut target: Option<StrongLink<T>>, x: T) {
        let mut new = Node::new();
        new.x = x;
        
        let mut new_node = StrongLink::new_link(new);
        new_node.set_prev(target.as_ref().and_then(|p| p.get_prev()));
        if let Some(link) = target.as_mut() {
            link.set_prev(Some(Rc::downgrade(&new_node)))
        };
        new_node.set_next(target);
        if let Some(p) = new_node.get_prev().as_mut() {
            p.set_next(Some(Rc::clone(&new_node)))
        };
        self.n += 1;
    }

    pub fn remove_node(&mut self, w: Option<StrongLink<T>>) {
        let mut prev = w.as_ref().and_then(|p| p.get_prev());
        let mut next = w.and_then(|p| p.get_next());

        if let Some(weak) = prev.as_mut() {
            weak.set_next(next.clone())
        };
        if let Some(p) = next.as_mut() {
            p.set_prev(prev);
        };
        self.n -= 1;
    }
}

impl<T: Default + Clone> CloneList<T> for DLList<T> {
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<T> {
        self.get_link(i).map(|rc| rc.as_ref().borrow().x.clone())
    }

    fn set(&mut self, i: usize, x: T) -> T {
        let u = self.get_link(i);
        let y = u.map(|rc| std::mem::replace(&mut rc.as_ref().borrow_mut().x, x));
        y.unwrap()
    }

    fn add(&mut self, i: usize, x: T) {
        self.add_before(self.get_link(i), x);
    }

    fn remove(&mut self, i: usize) -> T {
        let node = self.get_link(i);
        let x = node.as_ref().map(|rc| rc.as_ref().borrow().x.clone());
        self.remove_node(node);
        x.unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_clone_list() {
        let mut list = DLList::new();
        list.add(0, 'a');
        list.add(1, 'b');
        list.add(2, 'c');
        list.add(3, 'd');
        list.add(4, 'e');
        assert_eq!(list.size(), 5);
        assert_eq!(list.get(0).unwrap(), 'a');
        assert_eq!(list.get(1).unwrap(), 'b');
        assert_eq!(list.get(2).unwrap(), 'c');
        assert_eq!(list.get(3).unwrap(), 'd');
        assert_eq!(list.get(4).unwrap(), 'e');

        list.remove(3);
        assert_eq!(list.size(), 4);
        assert_eq!(list.get(0).unwrap(), 'a');
        assert_eq!(list.get(1).unwrap(), 'b');
        assert_eq!(list.get(2).unwrap(), 'c');
        assert_eq!(list.get(3).unwrap(), 'e');
    }
}
