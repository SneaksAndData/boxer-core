use actix_web::body::MessageBody;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::Next;
use actix_web::Error;
use log::warn;

pub async fn custom_error_logging(
    req: ServiceRequest,
    srv: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let res = srv.call(req).await?;
    if let Some(error) = res.response().error() {
        warn!("Response error: {:?}", error);
    }
    Ok(res)
}
