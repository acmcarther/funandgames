pub use self::helpers::{
  Tappable,
  TappableIter
};

pub mod helpers {

  pub trait Tappable<T> {
    fn tap<U, F: FnOnce(&T) -> U>(self, F) -> Self;
  }

  pub trait TappableIter<T> {
    fn tap<U, F: Fn(&T) -> U>(self, F) -> Tap<Self, F>;
  }

  impl<T> Tappable<T> for Option<T> {
    fn tap<U, F: FnOnce(&T) -> U>(self, op: F) -> Option<T> {
      self.map(|val| {
        op(&val);
        val
      })
    }
  }

  impl<T, E> Tappable<T> for Result<T, E> {
    fn tap<U, F: FnOnce(&T) -> U>(self, op: F) -> Result<T, E> {
      self.map(|val| {
        op(&val);
        val
      })
    }
  }

  impl <T, I> TappableIter<T> for I
    where I: Iterator {
    fn tap<U, F: Fn(&T) -> U>(self, op: F) -> Tap<Self, F> {
      Tap {iter: self, f: op}
    }
  }

  #[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
  #[derive(Clone)]
  pub struct Tap<I, F> {
    iter: I,
    f: F
  }

  impl <I, F, U> Iterator for Tap<I, F>
    where I: Iterator, F: Fn(&<I as Iterator>::Item) -> U {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<I::Item> {
      self.iter.next().tap(|a| (self.f)(a))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
      self.iter.size_hint()
    }
  }
}

#[test]
fn it_taps_on_some() {
  let mut x = 1;
  let some = Some(2);
  let res = some.tap(|val| x = val + x);
  assert!(res.unwrap() == 2);
  assert!(x == 3);
}

#[test]
fn it_doesnt_tap_on_none() {
  let mut x = 1;
  let none = None;
  let res = none.tap(|val| x = val + x);
  assert!(x == 1);
}
