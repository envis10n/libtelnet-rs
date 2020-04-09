/// Concatenate 2 `&[u8]` slices into a `Vec<u8>`
///
/// TODO: Optimize or replace with a better method for this!
pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
  [&a[..], &b[..]].concat()
}
