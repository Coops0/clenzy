use crate::browser::installation::Installation;

pub mod profile;
pub mod installation;

pub trait Browser {
    fn name() -> &'static str;
    fn installations() -> Vec<Installation>;
    fn fetch_resources() -> Option<fn() -> color_eyre::Result<&'static str>> {
        None
    }
    fn debloat(installation: &Installation) -> color_eyre::Result<()>;
}
