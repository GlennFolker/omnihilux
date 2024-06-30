use crate::{draw::vertex::DrawVertex, shape::vertex::Request};

pub mod basic;
pub mod line;
pub mod vertex;

pub struct Drawer<'a> {
    requests: &'a mut Vec<Request<DrawVertex>>,
}

impl<'a> Drawer<'a> {
    #[inline]
    pub fn new(requests: &'a mut Vec<Request<DrawVertex>>) -> Self {
        Self { requests }
    }
}
