use actix_web::http::header::HeaderValue;

pub trait TokenWithId: TryFrom<&'static HeaderValue> {
    fn id() -> String;
}
