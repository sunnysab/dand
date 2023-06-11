use std::path::PathBuf;

use discover::PostWalker;
use post::PostParser;

mod datetime;
mod discover;
mod post;

fn main() {
    let path = PathBuf::from(String::from("/home/sunnysab/blog-test.md"));
    let file = <PostWalker<'_> as discover::ReadSingleFile>::open(path).unwrap();
    println!("{:#?}", file);

    let post = PostParser::open(file).unwrap().parse().unwrap();
    println!("{:#?}", post);
}
