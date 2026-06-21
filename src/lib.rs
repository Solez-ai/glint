// Glint - A next-generation native Windows photo viewer and lightweight editor
// Copyright (c) 2025 Samin Yeasar. All rights reserved.
// Licensed under the MIT License.

#![doc = include_str!("../README.md")]

pub mod app;
pub mod browser;
pub mod editor;
pub mod image;
pub mod metadata;
pub mod platform;
pub mod renderer;
pub mod thumbnail;
pub mod ui;

pub use app::{AppMessage, GlintApp};
