use crate::middleware::middleware::TreeMiddleware;

pub  struct  OrderState<M:TreeMiddleware>{
    tree:M,
    

}