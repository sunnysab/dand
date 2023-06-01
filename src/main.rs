use std::path::PathBuf;
use crate::discover::{PostWalker};
use crate::post::PostBuilder;

mod discover;
mod post;

fn main() {
    let path = PathBuf::from(String::from("/home/sunnysab/Blog/source/_posts/ZIM-file-format.md"));
    let file = <PostWalker<'_> as discover::ReadSingleFile>::open(path).unwrap();
    println!("{:#?}", file);

    let post = PostBuilder::new(file).read().unwrap().parse().unwrap().build();
    println!("{:#?}", post);
}