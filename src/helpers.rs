pub use self::helpers::{
  TappableOption
};

pub mod helpers {
  pub trait TappableOption<T> {
    fn tap<U, F: FnOnce(&T) -> U>(self, F) -> Self;
  }

  impl<T> TappableOption<T> for Option<T> {
    fn tap<U, F: FnOnce(&T) -> U>(self, op: F) -> Option<T> {
      self.map(|val| {
        op(&val);
        val
      })
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
