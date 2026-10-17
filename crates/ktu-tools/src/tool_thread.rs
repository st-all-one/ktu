//! T2 — thread dedicada de execução de tools (§2.2 do `plan.md`).
//!
//! O kernel (T1) fala por **mensagens**: `tokio::sync::mpsc` para enviar a chamada
//! e `tokio::sync::oneshot` para receber o resultado. Nenhuma tool corre na task do
//! kernel (I8). Uma única T2 serve o agente primário e as sessões-filhas.
//!
//! O construtor do executor corre **dentro** da thread: assim pode emprestar estado
//! local à thread (portas `Fs`/`Clock`/`Memory`) sem exigir `'static` no executor.

use std::sync::mpsc as std_mpsc;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread::JoinHandle;

use async_trait::async_trait;
use ktu_core::domain::{ToolCall, ToolSpec};
use ktu_core::tools::{ToolExecutor, ToolOutcome};
use tokio::runtime::Builder;
use tokio::sync::{mpsc, oneshot};

/// Pedido de execução: a chamada e o canal de resposta.
struct Request {
    call: ToolCall,
    reply: oneshot::Sender<ToolOutcome>,
}

/// Contexto entregue ao construtor da thread. Serve pedidos até o kernel largar
/// o handle.
pub struct WorkerCtx {
    requests: mpsc::UnboundedReceiver<Request>,
    specs: std_mpsc::Sender<Vec<ToolSpec>>,
}

impl WorkerCtx {
    /// Publica as specs e serve pedidos até o canal fechar (handle largado).
    pub fn serve(mut self, tools: &dyn ToolExecutor) {
        if self.specs.send(tools.specs()).is_err() {
            return;
        }
        let runtime = Builder::new_current_thread().enable_all().build();
        while let Some(request) = self.requests.blocking_recv() {
            let outcome = match &runtime {
                Ok(runtime) => runtime.block_on(tools.execute(&request.call)),
                Err(_) => ToolOutcome::error("runtime de tools indisponivel"),
            };
            // Se o kernel abandonou o pedido, continua a servir os restantes.
            let _ignored = request.reply.send(outcome);
        }
    }
}

/// Handle do kernel (T1) para a thread de tools (T2).
pub struct ToolThread {
    requests: Option<mpsc::UnboundedSender<Request>>,
    specs: Vec<ToolSpec>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl ToolThread {
    /// Spawna a T2. `run` constrói o executor **dentro da thread** e chama
    /// [`WorkerCtx::serve`]. O handshake das specs acontece antes de retornar.
    ///
    /// # Errors
    /// Falha ao criar a thread do sistema operativo ou se a thread morrer antes de
    /// publicar as specs.
    pub fn spawn(run: impl FnOnce(WorkerCtx) + Send + 'static) -> std::io::Result<Self> {
        let (requests_tx, requests_rx) = mpsc::unbounded_channel();
        let (specs_tx, specs_rx) = std_mpsc::channel();
        let join = std::thread::Builder::new()
            .name("ktu-tools-t2".to_owned())
            .spawn(move || {
                run(WorkerCtx {
                    requests: requests_rx,
                    specs: specs_tx,
                });
            })?;
        let specs = specs_rx
            .recv()
            .map_err(|_| std::io::Error::other("thread de tools terminou durante o arranque"))?;
        Ok(Self {
            requests: Some(requests_tx),
            specs,
            join: Mutex::new(Some(join)),
        })
    }

    fn joins(&self) -> MutexGuard<'_, Option<JoinHandle<()>>> {
        self.join.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl ToolExecutor for ToolThread {
    fn specs(&self) -> Vec<ToolSpec> {
        self.specs.clone()
    }

    async fn execute(&self, call: &ToolCall) -> ToolOutcome {
        let Some(sender) = self.requests.as_ref() else {
            return ToolOutcome::error("thread de tools indisponivel");
        };
        let (reply, receiver) = oneshot::channel();
        let request = Request {
            call: call.clone(),
            reply,
        };
        if sender.send(request).is_err() {
            return ToolOutcome::error("thread de tools indisponivel");
        }
        match receiver.await {
            Ok(outcome) => outcome,
            Err(_) => ToolOutcome::error("thread de tools terminou sem resposta"),
        }
    }
}

impl Drop for ToolThread {
    fn drop(&mut self) {
        // Largar o emissor **antes** do join (temporário sem binding, largado já
        // nesta linha): sem isto a T2 nunca sai do `blocking_recv`.
        drop(self.requests.take());
        let join = self.joins().take();
        if let Some(join) = join {
            let _ignored = join.join();
        }
    }
}
