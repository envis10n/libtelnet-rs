/// Alias for Vec<&[u8]>.concat because the type system makes it very verbose.
pub fn concat(a: Vec<&[u8]>) -> Vec<u8> {
  a.concat()
}
