//! Núcleo do `cli-voice-bridge`.
//!
//! Aqui mora o que todo mundo compartilha: o vocabulário de momentos, o
//! protocolo de IPC e os caminhos de cada plataforma.
//!
//! **Regra de dependência.** Este crate não conhece nenhum CLI de IA. A seta é
//! sempre `adapters → core`, nunca o contrário — é o que permite acrescentar um
//! quarto CLI sem tocar aqui. Ver ADR-0007.

pub mod audio;
pub mod caminhos;
pub mod config;
pub mod ipc;
pub mod momento;
pub mod protocolo;
pub mod sidecar;

pub use config::Config;
pub use momento::{Evento, Momento, Origem, Transporte, Urgencia};
pub use protocolo::{Requisicao, Resposta, VERSAO_PROTOCOLO};
