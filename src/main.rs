fn main() {
  let hash = blake3::hash(b"foobarbaz");
  let hash = hash.to_string();
  println!("{}", hash.len());
}
