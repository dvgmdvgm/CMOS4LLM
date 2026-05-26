use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing;

use crate::error::SubLmError;
use crate::service::{CompletionRequest, InferenceService};

static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub enum TaskKind {
    Summarize { text: String, max_length: u32 },
    ExtractJson { text: String, schema_hint: String },
    Classify { text: String, categories: Vec<String> },
    Complete(CompletionRequest),
}

#[derive(Debug, Clone)]
pub struct TaskRequest {
    pub id: u64,
    pub kind: TaskKind,
    pub priority: TaskPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed(TaskResult),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub task_id: u64,
    pub output: String,
}

struct InternalTask {
    request: TaskRequest,
    response_tx: oneshot::Sender<Result<TaskResult, SubLmError>>,
}

pub struct TaskQueue {
    sender: mpsc::Sender<InternalTask>,
    pending_count: Arc<AtomicU64>,
    completed_count: Arc<AtomicU64>,
}

impl TaskQueue {
    pub fn new<S: InferenceService + 'static>(service: Arc<S>, worker_count: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<InternalTask>(256);
        let receiver = Arc::new(Mutex::new(receiver));
        let pending_count = Arc::new(AtomicU64::new(0));
        let completed_count = Arc::new(AtomicU64::new(0));

        for worker_id in 0..worker_count {
            let rx = Arc::clone(&receiver);
            let svc = Arc::clone(&service);
            let pending = Arc::clone(&pending_count);
            let completed = Arc::clone(&completed_count);

            tokio::spawn(async move {
                loop {
                    let task = {
                        let mut guard = rx.lock().await;
                        guard.recv().await
                    };

                    match task {
                        Some(internal) => {
                            pending.fetch_sub(1, Ordering::Relaxed);
                            let result =
                                execute_task(&*svc, &internal.request).await;
                            completed.fetch_add(1, Ordering::Relaxed);

                            if internal.response_tx.send(result).is_err() {
                                tracing::debug!(
                                    worker_id,
                                    task_id = internal.request.id,
                                    "task result receiver dropped"
                                );
                            }
                        }
                        None => break,
                    }
                }
                tracing::debug!(worker_id, "worker shutting down");
            });
        }

        Self {
            sender,
            pending_count,
            completed_count,
        }
    }

    pub async fn submit(
        &self,
        kind: TaskKind,
        priority: TaskPriority,
    ) -> Result<oneshot::Receiver<Result<TaskResult, SubLmError>>, SubLmError> {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        let request = TaskRequest { id, kind, priority };
        let (response_tx, response_rx) = oneshot::channel();

        let internal = InternalTask {
            request,
            response_tx,
        };

        self.sender
            .send(internal)
            .await
            .map_err(|_| SubLmError::QueueFull)?;

        self.pending_count.fetch_add(1, Ordering::Relaxed);
        Ok(response_rx)
    }

    pub async fn submit_and_wait(
        &self,
        kind: TaskKind,
        priority: TaskPriority,
    ) -> Result<TaskResult, SubLmError> {
        let rx = self.submit(kind, priority).await?;
        rx.await.map_err(|_| SubLmError::Cancelled)?
    }

    pub fn pending_count(&self) -> u64 {
        self.pending_count.load(Ordering::Relaxed)
    }

    pub fn completed_count(&self) -> u64 {
        self.completed_count.load(Ordering::Relaxed)
    }
}

async fn execute_task<S: InferenceService>(
    service: &S,
    request: &TaskRequest,
) -> Result<TaskResult, SubLmError> {
    let output = match &request.kind {
        TaskKind::Summarize { text, max_length } => service.summarize(text, *max_length).await?,
        TaskKind::ExtractJson { text, schema_hint } => {
            service.extract_json(text, schema_hint).await?
        }
        TaskKind::Classify { text, categories } => {
            let cat_refs: Vec<&str> = categories.iter().map(|s| s.as_str()).collect();
            let result = service.classify(text, &cat_refs).await?;
            serde_json::to_string(&serde_json::json!({
                "category": result.category,
                "confidence": result.confidence,
                "reasoning": result.reasoning,
            }))
            .unwrap_or_default()
        }
        TaskKind::Complete(req) => service.complete(req.clone()).await?,
    };

    Ok(TaskResult {
        task_id: request.id,
        output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{BackendStatus, ClassifyResult};
    use async_trait::async_trait;

    struct MockService;

    #[async_trait]
    impl InferenceService for MockService {
        async fn complete(&self, request: CompletionRequest) -> Result<String, SubLmError> {
            Ok(format!("completed: {}", request.user_prompt))
        }
        async fn classify(
            &self,
            _text: &str,
            categories: &[&str],
        ) -> Result<ClassifyResult, SubLmError> {
            Ok(ClassifyResult {
                category: categories[0].to_string(),
                confidence: 0.9,
                reasoning: None,
            })
        }
        async fn health_check(&self) -> Result<BackendStatus, SubLmError> {
            Ok(BackendStatus::Available {
                model: "mock".into(),
            })
        }
        async fn summarize(&self, text: &str, _max_length: u32) -> Result<String, SubLmError> {
            Ok(format!("summary of: {}", &text[..text.len().min(50)]))
        }
        async fn extract_json(
            &self,
            _text: &str,
            _schema_hint: &str,
        ) -> Result<String, SubLmError> {
            Ok(r#"{"extracted": true}"#.to_string())
        }
    }

    #[tokio::test]
    async fn test_submit_and_wait_complete() {
        let service = Arc::new(MockService);
        let queue = TaskQueue::new(service, 2);

        let result = queue
            .submit_and_wait(
                TaskKind::Complete(CompletionRequest {
                    system_prompt: "sys".into(),
                    user_prompt: "hello".into(),
                    max_tokens: 100,
                    temperature: 0.5,
                }),
                TaskPriority::Normal,
            )
            .await
            .unwrap();

        assert_eq!(result.output, "completed: hello");
    }

    #[tokio::test]
    async fn test_submit_and_wait_summarize() {
        let service = Arc::new(MockService);
        let queue = TaskQueue::new(service, 2);

        let result = queue
            .submit_and_wait(
                TaskKind::Summarize {
                    text: "some long text here".into(),
                    max_length: 50,
                },
                TaskPriority::Normal,
            )
            .await
            .unwrap();

        assert!(result.output.starts_with("summary of:"));
    }

    #[tokio::test]
    async fn test_multiple_tasks_parallel() {
        let service = Arc::new(MockService);
        let queue = TaskQueue::new(service, 4);

        let mut receivers = Vec::new();
        for i in 0..10 {
            let rx = queue
                .submit(
                    TaskKind::Complete(CompletionRequest {
                        system_prompt: "sys".into(),
                        user_prompt: format!("task {}", i),
                        max_tokens: 100,
                        temperature: 0.5,
                    }),
                    TaskPriority::Normal,
                )
                .await
                .unwrap();
            receivers.push(rx);
        }

        for rx in receivers {
            let result = rx.await.unwrap().unwrap();
            assert!(result.output.starts_with("completed: task"));
        }

        assert_eq!(queue.completed_count(), 10);
    }

    #[tokio::test]
    async fn test_counters() {
        let service = Arc::new(MockService);
        let queue = TaskQueue::new(service, 1);

        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.completed_count(), 0);

        queue
            .submit_and_wait(
                TaskKind::ExtractJson {
                    text: "test".into(),
                    schema_hint: "{}".into(),
                },
                TaskPriority::High,
            )
            .await
            .unwrap();

        assert_eq!(queue.completed_count(), 1);
    }
}
