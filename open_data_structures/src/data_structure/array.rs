use std::vec;

use crate::interface::list::List;

struct ArrayStack<T> {
    a: Box<[T]>, // 通常はVecで良いが、Vecは自動的に配列の長さが変わるため、resizeを実装するためにあえてBoxで持っている
    n: usize,    // 要素に入っているリストの要素数
}

impl<T: Default + Clone> ArrayStack<T> {
    fn new(size: usize) -> Self {
        Self {
            a: vec![T::default(); size].into_boxed_slice(),
            n: 0,
        }
    }

    // 配列の長さを変更する
    fn resize(&mut self) {
        let mut b = vec![T::default(); std::cmp::max(2 * self.n, 1)].into_boxed_slice();
        for i in 0..self.n {
            b[i] = self.a[i].clone();
        }
        self.a = b;
    }
}

impl<T> List<T> for ArrayStack<T>
where
    T: Default + Clone,
{
    fn size(&self) -> usize {
        self.n
    }

    fn get(&self, i: usize) -> Option<&T> {
        self.a.get(i)
    }

    fn set(&mut self, i: usize, x: T) -> T {
        std::mem::replace(&mut self.a[i], x)
    }

    fn add(&mut self, i: usize, x: T) {
        // 要素を一つ追加する分のキャパシティがなければresizeする
        if self.n >= self.a.len() {
            self.resize();
        }

        for j in (i + 1..=self.n).rev().step_by(1) {
            self.a[j] = self.a[j - 1].clone();
        }
        self.a[i] = x;
        self.n += 1;
    }

    fn remove(&mut self, i: usize) -> T {
        let x = self.a[i].clone();
        for j in i..(self.n - 1) {
            self.a[j] = self.a[j + 1].clone();
        }
        self.n -= 1;
        // 配列の長さに対して要素が少なすぎない場合はresizeする
        if self.a.len() >= 3 * self.n {
            self.resize();
        }
        x
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_resize() {
        let mut array: ArrayStack<i32> = ArrayStack::new(1);
        array.resize();
        assert_eq!(array.a.len(), 1);
        assert_eq!(array.n, 0);

        array.add(0, 0);
        assert_eq!(array.a.len(), 1);
        assert_eq!(array.n, 1);
    }

    #[test]
    fn test_array_stack() {
        let mut array = ArrayStack::new(6);
        array.add(0, "b");
        array.add(1, "r");
        array.add(2, "e");
        array.add(3, "d");
        assert_eq!(array.a, vec!["b", "r", "e", "d", "", ""].into_boxed_slice());
        assert_eq!(array.n, 4);
        array.add(2, "e");
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "d", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 5);
        array.add(5, "r");
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "d", "r"].into_boxed_slice()
        );
        assert_eq!(array.n, 6);
        array.add(5, "e");
        assert_eq!(
            array.a,
            vec!["b", "r", "e", "e", "d", "e", "r", "", "", "", "", ""].into_boxed_slice()
        );
        assert_eq!(array.n, 7);
    }
}
