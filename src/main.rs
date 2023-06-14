use discover::PostWalker;
use site::get as site;

mod datetime;
mod discover;
mod env;
mod post;
mod site;

fn main() {
    let walker =
        <PostWalker<'_> as discover::WalkDirectory>::new("/home/sunnysab/Blog/source/_posts/");
    let _env = env::Environment::new().walk(walker).unwrap();
    // println!("{:#?}", env);
}
