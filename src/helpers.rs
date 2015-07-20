pub use self::helpers::{
  TappableOption
};

mod helpers {
  trait TappableOption<T> {
    fn tap(&self) -> Self;
  }

  impl TappableOption<T> for Option<T> {
    fn tap<F: FnOnce(&T) -> U>(self, op: f) -> Option<T> {
      self.map(|val| {
        op(&val);
        val
      });
   }
  }
}
