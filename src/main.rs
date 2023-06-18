use std::path::PathBuf;

use discover::PostWalker;
use site::get as site;

use crate::discover::WalkDirectory;

mod datetime;
mod discover;
mod env;
mod post;
mod site;

fn main() -> anyhow::Result<()> {
    // let walker =
    //     <PostWalker<'_> as discover::WalkDirectory>::new("/home/sunnysab/Blog/source/_posts/");
    let mut env = env::Environment::new();
    //
    // for post_ref in walker.walk()?.into_iter().take(3) {
    //     println!("current post is {:?}", &post_ref.relative_name);
    //
    //     let post = post::load(post_ref)?.build(&mut env)?;
    //     println!("{:#?}", post.0.borrow().metadata.tags);
    // }

    let path = "/home/sunnysab/Blog/source/_posts/Some-Security-Issue.md";
    let post_ref = <PostWalker as discover::ReadSingleFile>::open(PathBuf::from(path))?;

    let post = post::load(post_ref)?.build(&mut env)?;
    println!("{:?}", post.0.borrow().metadata.tags);

    let renderer = post.renderer();
    let html = renderer.html();
    println!("{html}");
    Ok(())
}
