#![allow(dead_code)]
pub mod config;
pub mod error;
pub mod message;
pub mod transport;

mod template_registry;

// #[cfg(test)]
// mod tests {
//     use crate::email::error::EmailError;
//     use crate::email::config::EmailConfig;
//     use crate::email::transport::EmailTransport;
//     use crate::email::message::{EmailMessage, GaRelease};

//     #[test]
//     fn smtp_connection() -> Result<(), EmailError> {
//         let config = EmailConfig::from_env()?;
//         let transport = EmailTransport::new(config.smtp_connection())?;
//         GaRelease.send(&transport, config.from(), "alex@banyan.computer", true)?;
//         panic!("test");
//         Ok(())
//     }
// }
