//! Estado vivo do daemon: sessões conhecidas e silenciamento.
//!
//! Deliberadamente pequeno. Fila de fala, log com retenção e métricas para a
//! GUI ainda não existem — ver `docs/pt-BR/architecture/03-component.md`.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use cvb_core::Evento;

pub struct Estado {
    sessoes: HashSet<String>,
    silenciado_ate: Option<Instant>,
    silenciado_indefinidamente: bool,
}

impl Estado {
    pub fn novo() -> Self {
        Estado {
            sessoes: HashSet::new(),
            silenciado_ate: None,
            silenciado_indefinidamente: false,
        }
    }

    pub fn registrar(&mut self, evento: &Evento) {
        if !evento.sessao_id.is_empty() {
            self.sessoes.insert(evento.sessao_id.clone());
        }
    }

    pub fn sessoes(&self) -> usize {
        self.sessoes.len()
    }

    /// `None` cala até `retomar`.
    pub fn silenciar(&mut self, segundos: Option<u64>) {
        match segundos {
            Some(s) => {
                self.silenciado_indefinidamente = false;
                self.silenciado_ate = Some(Instant::now() + Duration::from_secs(s));
            }
            None => {
                self.silenciado_indefinidamente = true;
                self.silenciado_ate = None;
            }
        }
    }

    pub fn retomar(&mut self) {
        self.silenciado_indefinidamente = false;
        self.silenciado_ate = None;
    }

    pub fn silenciado(&self) -> bool {
        if self.silenciado_indefinidamente {
            return true;
        }
        self.silenciado_ate.is_some_and(|ate| Instant::now() < ate)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn silenciar_por_tempo_expira_sozinho() {
        let mut e = Estado::novo();
        assert!(!e.silenciado());
        e.silenciar(Some(0));
        // Já passou: zero segundo expira na hora.
        assert!(!e.silenciado());
        e.silenciar(Some(3600));
        assert!(e.silenciado());
        e.retomar();
        assert!(!e.silenciado());
    }

    #[test]
    fn silenciar_sem_prazo_so_sai_com_retomar() {
        let mut e = Estado::novo();
        e.silenciar(None);
        assert!(e.silenciado());
        e.retomar();
        assert!(!e.silenciado());
    }
}
