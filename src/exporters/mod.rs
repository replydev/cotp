pub mod andotp;

pub trait Exporter<T> {
    fn export(self: Self) -> T;
}
