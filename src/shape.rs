use crate::common::{Node, NodeType};
use crate::BabelfontError;

#[derive(Debug)]
pub enum PathDirection {
    Clockwise = 1,
    Anticlockwise = 0,
}
#[derive(Debug)]
pub struct Component {
    pub reference: String,
    pub transform: kurbo::Affine,
}

#[derive(Debug)]
pub struct Path {
    pub nodes: Vec<Node>,
    pub closed: bool,
    pub direction: PathDirection,
}

impl Path {
    /// Converts the `Path` to a [`kurbo::BezPath`].
    // Stolen completely from norad
    pub fn to_kurbo(&self) -> Result<kurbo::BezPath, BabelfontError> {
        let mut path = kurbo::BezPath::new();
        let mut offs = std::collections::VecDeque::new();
        let mut nodes = if self.closed {
            // Add end-of-contour offcurves to queue
            let rotate = self
                .nodes
                .iter()
                .rev()
                .position(|pt| pt.nodetype != NodeType::OffCurve)
                .map(|idx| self.nodes.len() - 1 - idx);
            self.nodes
                .iter()
                .cycle()
                .skip(rotate.unwrap_or(0))
                .take(self.nodes.len() + 1)
        } else {
            self.nodes.iter().cycle().skip(0).take(self.nodes.len())
        };
        if let Some(start) = nodes.next() {
            path.move_to(start.to_kurbo());
        }
        for pt in nodes {
            let kurbo_point = pt.to_kurbo();
            match pt.nodetype {
                NodeType::Move => path.move_to(kurbo_point),
                NodeType::Line => path.line_to(kurbo_point),
                NodeType::OffCurve => offs.push_back(kurbo_point),
                NodeType::Curve => {
                    match offs.make_contiguous() {
                        [] => return Err(BabelfontError::BadPath),
                        [p1] => path.quad_to(*p1, kurbo_point),
                        [p1, p2] => path.curve_to(*p1, *p2, kurbo_point),
                        _ => return Err(BabelfontError::BadPath),
                    };
                    offs.clear();
                }
            }
        }
        Ok(path)
    }
}

#[derive(Debug)]
pub enum Shape {
    ComponentShape(Component),
    PathShape(Path),
}
