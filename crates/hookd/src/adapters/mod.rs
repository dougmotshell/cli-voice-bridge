//! Um adaptador por CLI. Cada um conhece o dialeto do seu, e só o dele.
//!
//! **A seta é sempre `adapters → core`.** O núcleo não sabe que estes existem —
//! é o que permite acrescentar um quarto CLI sem tocar nele (ADR-0007).

pub mod claude;
pub mod codex;
pub mod copilot;
