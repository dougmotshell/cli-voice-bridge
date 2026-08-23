//! A fila de fala: prioridade, colapso, expiração e corte.
//!
//! Uma fila só e um reprodutor só. As regras estão em
//! `docs/pt-BR/specs/speech-output.md`; o que segue é o porquê de cada uma
//! existir, porque nenhuma delas é enfeite:
//!
//! - **Prioridade.** Um pedido de permissão chegando enquanto se anuncia uma
//!   tarefa concluída não pode esperar a vez: o agente está parado.
//! - **Corte.** Crítico interrompe o que estiver tocando e joga fora o que for
//!   menos urgente. Anunciar o que já não importa é ruído.
//! - **Colapso.** Três ferramentas falhando em sequência viram uma frase. Sem
//!   isso, a mesma frase sai três vezes e a pessoa desliga o projeto.
//! - **Expiração.** "Terminei" dito quarenta segundos depois é pior que
//!   silêncio: a pessoa já olhou, e agora só confunde.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use cvb_core::momento::agora_ms;
use cvb_core::{Momento, Origem, Urgencia};

use super::Voz;

/// Uma fala esperando a vez.
#[derive(Debug, Clone)]
pub struct Item {
    pub texto: String,
    pub urgencia: Urgencia,
    /// Itens com a mesma chave colapsam num só. `None` nunca colapsa.
    pub chave: Option<(Origem, Momento)>,
    pub criado_ms: u64,
    /// Quantas vezes o mesmo acontecimento se repetiu antes de falar.
    pub repeticoes: u32,
    /// Desempate por ordem de chegada dentro da mesma urgência.
    seq: u64,
}

impl Item {
    pub fn novo(texto: String, urgencia: Urgencia, chave: Option<(Origem, Momento)>) -> Item {
        Item {
            texto,
            urgencia,
            chave,
            criado_ms: agora_ms(),
            repeticoes: 0,
            seq: 0,
        }
    }

    /// O que de fato se fala, já contando as repetições colapsadas.
    pub fn falado(&self) -> String {
        match self.repeticoes {
            0 => self.texto.clone(),
            1 => format!("{}, e mais uma vez.", self.texto.trim_end_matches('.')),
            n => format!("{}, e mais {n} vezes.", self.texto.trim_end_matches('.')),
        }
    }
}

struct Interno {
    itens: VecDeque<Item>,
    encerrar: bool,
}

pub struct Fila {
    interno: Arc<(Mutex<Interno>, Condvar)>,
    voz: Arc<Voz>,
    seq: AtomicU64,
    /// Momento mais velho que isto é descartado sem falar. Crítico não expira.
    janela_ms: u64,
}

impl Fila {
    pub fn nova(voz: Arc<Voz>, janela_relevancia_s: u64) -> Arc<Fila> {
        let fila = Arc::new(Fila {
            interno: Arc::new((
                Mutex::new(Interno {
                    itens: VecDeque::new(),
                    encerrar: false,
                }),
                Condvar::new(),
            )),
            voz,
            seq: AtomicU64::new(0),
            janela_ms: janela_relevancia_s * 1000,
        });

        let trabalhadora = Arc::clone(&fila);
        std::thread::spawn(move || trabalhadora.trabalhar());
        fila
    }

    /// Põe na fila. Crítico corta o que estiver falando e limpa o resto.
    pub fn enfileirar(&self, mut item: Item) {
        item.seq = self.seq.fetch_add(1, Ordering::Relaxed);
        let critico = item.urgencia == Urgencia::Critica;

        {
            let (trava, aviso) = &*self.interno;
            let mut interno = trava.lock().unwrap_or_else(|e| e.into_inner());

            if critico {
                // Some com tudo que não é crítico: depois de um pedido de
                // permissão, ninguém quer ouvir que uma tarefa terminou.
                interno.itens.retain(|i| i.urgencia == Urgencia::Critica);
            } else if let Some(existente) = item
                .chave
                .and_then(|c| interno.itens.iter_mut().find(|i| i.chave == Some(c)))
            {
                existente.repeticoes += 1;
                aviso.notify_all();
                return;
            }

            interno.itens.push_back(item);
            aviso.notify_all();
        }

        if critico {
            self.voz.cortar();
        }
    }

    /// A pessoa voltou a digitar: cala tudo, na hora.
    pub fn cortar_tudo(&self) {
        {
            let (trava, aviso) = &*self.interno;
            let mut interno = trava.lock().unwrap_or_else(|e| e.into_inner());
            interno.itens.clear();
            aviso.notify_all();
        }
        self.voz.cortar();
    }

    pub fn tamanho(&self) -> usize {
        let (trava, _) = &*self.interno;
        trava.lock().unwrap_or_else(|e| e.into_inner()).itens.len()
    }

    /// Para a trabalhadora.
    ///
    /// Hoje só os testes usam: o daemon roda num laço infinito e morre por
    /// sinal, sem encerramento ordenado. O que falta — matar o reprodutor,
    /// descartar a fila, remover o socket — está em
    /// `docs/pt-BR/specs/daemon-lifecycle.md`.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn encerrar(&self) {
        let (trava, aviso) = &*self.interno;
        let mut interno = trava.lock().unwrap_or_else(|e| e.into_inner());
        interno.encerrar = true;
        aviso.notify_all();
    }

    /// Tira o de maior urgência; empate resolve por ordem de chegada.
    fn proximo(&self) -> Option<Item> {
        let (trava, aviso) = &*self.interno;
        let mut interno = trava.lock().unwrap_or_else(|e| e.into_inner());

        loop {
            if interno.encerrar {
                return None;
            }
            if let Some(pos) = melhor(&interno.itens) {
                return interno.itens.remove(pos);
            }
            interno = aviso
                .wait_timeout(interno, Duration::from_millis(500))
                .map(|(g, _)| g)
                .unwrap_or_else(|e| e.into_inner().0);
        }
    }

    fn trabalhar(&self) {
        while let Some(item) = self.proximo() {
            if self.envelheceu(&item) {
                eprintln!(
                    "cvb-hookd: descartei \"{}\" — passou da janela de relevância",
                    item.texto
                );
                continue;
            }
            let saida = self.voz.falar(&item.falado());
            if !saida.falou() {
                eprintln!(
                    "cvb-hookd: não falei \"{}\" — {}",
                    item.texto,
                    saida.descricao()
                );
            }
        }
    }

    fn envelheceu(&self, item: &Item) -> bool {
        // Crítico nunca expira: o agente continua parado esperando, por mais
        // tempo que tenha passado.
        item.urgencia != Urgencia::Critica
            && agora_ms().saturating_sub(item.criado_ms) > self.janela_ms
    }
}

/// Índice do item que deve falar primeiro.
fn melhor(itens: &VecDeque<Item>) -> Option<usize> {
    itens
        .iter()
        .enumerate()
        .max_by_key(|(_, i)| (i.urgencia, std::cmp::Reverse(i.seq)))
        .map(|(pos, _)| pos)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn item(texto: &str, u: Urgencia) -> Item {
        Item::novo(texto.into(), u, None)
    }

    fn com_chave(texto: &str, u: Urgencia, m: Momento) -> Item {
        Item::novo(texto.into(), u, Some((Origem::Claude, m)))
    }

    fn fila_de(itens: Vec<Item>) -> VecDeque<Item> {
        itens
            .into_iter()
            .enumerate()
            .map(|(i, mut it)| {
                it.seq = i as u64;
                it
            })
            .collect()
    }

    #[test]
    fn o_mais_urgente_fala_primeiro() {
        let f = fila_de(vec![
            item("baixa", Urgencia::Baixa),
            item("critica", Urgencia::Critica),
            item("media", Urgencia::Media),
        ]);
        assert_eq!(f[melhor(&f).unwrap()].texto, "critica");
    }

    #[test]
    fn empate_de_urgencia_respeita_a_ordem_de_chegada() {
        let f = fila_de(vec![
            item("primeiro", Urgencia::Alta),
            item("segundo", Urgencia::Alta),
        ]);
        assert_eq!(f[melhor(&f).unwrap()].texto, "primeiro");
    }

    #[test]
    fn fila_vazia_nao_tem_proximo() {
        assert!(melhor(&VecDeque::new()).is_none());
    }

    #[test]
    fn colapso_vira_uma_frase_so() {
        let mut i = com_chave(
            "uma ferramenta falhou.",
            Urgencia::Media,
            Momento::FerramentaFalhou,
        );
        assert_eq!(i.falado(), "uma ferramenta falhou.");
        i.repeticoes = 1;
        assert_eq!(i.falado(), "uma ferramenta falhou, e mais uma vez.");
        i.repeticoes = 3;
        assert_eq!(i.falado(), "uma ferramenta falhou, e mais 3 vezes.");
    }

    #[test]
    fn critico_nunca_expira() {
        let voz = Arc::new(Voz::nova(&cvb_core::Config::default()));
        let fila = Fila::nova(voz, 30);

        let mut velho_critico = item("permissao", Urgencia::Critica);
        velho_critico.criado_ms = agora_ms().saturating_sub(600_000);
        assert!(!fila.envelheceu(&velho_critico));

        let mut velho_alto = item("terminei", Urgencia::Alta);
        velho_alto.criado_ms = agora_ms().saturating_sub(600_000);
        assert!(fila.envelheceu(&velho_alto));

        fila.encerrar();
    }

    #[test]
    fn critico_limpa_o_que_e_menos_urgente() {
        let voz = Arc::new(Voz::nova(&cvb_core::Config::default()));
        let fila = Fila::nova(voz, 30);
        fila.encerrar(); // a trabalhadora não consome; olhamos a fila parada

        fila.enfileirar(item("baixa", Urgencia::Baixa));
        fila.enfileirar(item("media", Urgencia::Media));
        assert_eq!(fila.tamanho(), 2);

        fila.enfileirar(item("permissao", Urgencia::Critica));
        assert_eq!(fila.tamanho(), 1, "o crítico devia ter limpado o resto");
    }

    #[test]
    fn repetido_colapsa_em_vez_de_empilhar() {
        let voz = Arc::new(Voz::nova(&cvb_core::Config::default()));
        let fila = Fila::nova(voz, 30);
        fila.encerrar();

        for _ in 0..4 {
            fila.enfileirar(com_chave(
                "uma ferramenta falhou",
                Urgencia::Media,
                Momento::FerramentaFalhou,
            ));
        }
        assert_eq!(fila.tamanho(), 1, "quatro iguais deviam virar um");
    }

    #[test]
    fn cortar_tudo_esvazia() {
        let voz = Arc::new(Voz::nova(&cvb_core::Config::default()));
        let fila = Fila::nova(voz, 30);
        fila.encerrar();

        fila.enfileirar(item("a", Urgencia::Alta));
        fila.enfileirar(item("b", Urgencia::Media));
        fila.cortar_tudo();
        assert_eq!(fila.tamanho(), 0);
    }
}
