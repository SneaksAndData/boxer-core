use actix_web::http::header::HeaderValue;

pub trait TokenWithId: TryFrom<HeaderValue, Error = anyhow::Error> {
    fn id(&self) -> String;
}
