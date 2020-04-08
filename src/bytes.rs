pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
  [&a[..], &b[..]].concat()
}
