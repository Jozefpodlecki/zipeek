pub mod browse;
pub mod error;
pub mod home;
pub mod not_found;
pub mod practice;
pub mod practice_setup;
pub mod random;

pub use browse::Browse;
pub use error::ErrorPage;
pub use home::Home;
pub use not_found::NotFound;
pub use practice::{Practice, Variant};
pub use practice_setup::PracticeSetup;
pub use random::Random;
