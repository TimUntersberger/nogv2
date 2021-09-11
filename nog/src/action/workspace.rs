use crate::{direction::Direction, workspace::WorkspaceId};

#[derive(Debug, Clone)]
pub enum WorkspaceAction {
    Focus(Option<WorkspaceId>, Direction),
    Swap(Option<WorkspaceId>, Direction),
}
