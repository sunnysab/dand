use discover::PostWalker;
use site::get as site;

use crate::discover::WalkDirectory;

mod datetime;
mod discover;
mod env;
mod post;
mod site;

fn main() -> anyhow::Result<()> {
    let walker =
        <PostWalker<'_> as discover::WalkDirectory>::new("/home/sunnysab/Blog/source/_posts/");
    let mut env = env::Environment::new();

    for post_ref in walker.walk()? {
        let post = post::load(post_ref)?.build(&mut env)?;
        println!("{:#?}", post);
    }
    Ok(())
}
