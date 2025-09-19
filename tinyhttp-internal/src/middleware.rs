use super::response::Response;

#[derive(Debug, Clone)]
pub enum MiddlewareResponse {
    Next,
    Redirect(Response),
}
