use actix_web::Error;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;

pub async fn logger_middleware<B>(
    request: ServiceRequest,
    next: Next<B>,
) -> Result<ServiceResponse<B>, Error> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let response = next.call(request).await?;
    let status = response.status();
    tracing::info!("{} - {} {}", status, method, uri);
    Ok(response)
}
