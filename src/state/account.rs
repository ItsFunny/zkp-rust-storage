use crate::middleware::middleware::TreeMiddleware;

pub struct AccountState<M: TreeMiddleware> {
    tree: M,
}


