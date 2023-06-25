use anyhow::Result;

pub trait Render {
    fn html(&self) -> Result<String>;
}
