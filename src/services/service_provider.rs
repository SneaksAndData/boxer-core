pub trait ServiceProvider<T>: Send + Sync
where
    T: Send + Sync + 'static,
{
    fn get(&self) -> T;
}
