//! UI module
//! Consists of abstractions from ways of getting input from a user
//! and returning output
//!
//! As of now, there is only CLI (Command Line Interface), for a
//! standard console communication
//!
//! Specifically for DES, there are utility functions for sanitizing
//! encryption keys, gathered from user input 

pub mod cli;
pub mod key_parsing;
