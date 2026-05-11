use actix_web::{web, HttpResponse, Responder};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::oneshot::Receiver;

/// Tracks readiness of a backend component via a oneshot channel.
///
/// The first time the endpoint is queried after the sender fires, the state
/// is cached so subsequent checks are lock-free.
pub struct ReadinessProbe {
    rx: Mutex<Option<Receiver<()>>>,
    ready: AtomicBool,
}

impl ReadinessProbe {
    pub fn new(rx: Receiver<()>) -> Arc<Self> {
        Arc::new(Self {
            rx: Mutex::new(Some(rx)),
            ready: AtomicBool::new(false),
        })
    }

    pub async fn is_ready(&self) -> bool {
        if self.ready.load(Ordering::Relaxed) {
            return true;
        }
        let mut guard = self.rx.lock().await;
        if let Some(ref mut rx) = *guard {
            match rx.try_recv() {
                Ok(_) => {
                    self.ready.store(true, Ordering::Relaxed);
                    *guard = None;
                    true
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }
}

pub async fn readiness_handler(probe: web::Data<Arc<ReadinessProbe>>) -> impl Responder {
    HttpResponse::Ok().json(probe.is_ready().await)
}
