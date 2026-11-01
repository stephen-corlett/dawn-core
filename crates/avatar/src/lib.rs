//! A player's visible body, a supporting game module: what they look like, how their
//! body moves, and how the camera frames them. It has library-style boundaries, with
//! models, clips, and tuning supplied by the caller.
//!
//! To this module, a character is something to look at. It never decides where anyone
//! goes. It watches where `movement` puts them, and works out how to show that, which
//! is why remote players animate with nothing extra sent over the network.

#[cfg(feature = "client")]
pub mod client;
