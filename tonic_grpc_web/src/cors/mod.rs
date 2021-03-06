pub mod builder; 
pub mod config;
pub mod service;

pub use self::builder::CorsBuilder;
pub use self::config::AllowedOrigins;
pub use self::service::CorsService;
pub use self::config::Config;

use self::config::{ CorsResource };