use std::io::{self, BufRead, Write};

use crate::context::PipelineContext;
use crate::graph_store::Node;

use super::{Phase, PhaseError, PhaseId, PhaseOutput};

pub struct PolicyElicitationPhase;

impl Phase for PolicyElicitationPhase {
    fn id(&self) -> PhaseId {
        PhaseId::PolicyElicitation
    }

    fn run(&self, ctx: &mut PipelineContext) -> Result<PhaseOutput, PhaseError> {
        if !ctx.config.bootstrap.interactive {
            return Err(PhaseError::Skipped("interactive mode disabled (--no-interactive)".into()));
        }

        let mut nodes_created = 0;
        let warnings = Vec::new();

        let questions = self.generate_questions(ctx)?;
        let total = questions.len();

        ctx.progress.phase_detail(&format!("{} questions prepared", total));
        println!();
        println!("  Policy Elicitation: answer questions about your project conventions.");
        println!("  Press Enter to skip a question, Ctrl+C to stop (progress saved).");
        println!();

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for (i, question) in questions.iter().enumerate() {
            print!("  [{}/{}] {} ", i + 1, total, question.text);
            stdout.flush().ok();

            let mut answer = String::new();
            match stdin.lock().read_line(&mut answer) {
                Ok(_) => {}
                Err(_) => break,
            }

            let answer = answer.trim();
            if answer.is_empty() {
                continue;
            }

            ctx.graph.insert_node(&Node {
                id: None,
                project_id: ctx.config.project.name.clone(),
                kind: "policy_rule".to_string(),
                label: question.id.clone(),
                file_path: None,
                line_start: None,
                line_end: None,
                properties_json: serde_json::json!({
                    "question": question.text,
                    "answer": answer,
                    "category": question.category,
                    "source": "owner",
                    "confidence": 1.0,
                }).to_string(),
                phase_id: PhaseId::PolicyElicitation.as_u8(),
            })?;
            nodes_created += 1;
        }

        Ok(PhaseOutput {
            nodes_created,
            edges_created: 0,
            warnings,
        })
    }
}

impl PolicyElicitationPhase {
    fn generate_questions(&self, ctx: &PipelineContext) -> Result<Vec<PolicyQuestion>, PhaseError> {
        let mut questions = Vec::new();

        let models = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "django_model")?;
        let _conventions = ctx.graph.query_nodes_by_kind(&ctx.config.project.name, "convention")?;

        if !models.is_empty() {
            questions.push(PolicyQuestion {
                id: "decimal_money".into(),
                text: "Do you use Decimal for monetary values as a strict policy? (yes/no/explain)".into(),
                category: "data_types".into(),
            });
        }

        questions.push(PolicyQuestion {
            id: "view_style".into(),
            text: "FBV vs CBV: is function-based views a deliberate choice or legacy? (deliberate/legacy/mixed)".into(),
            category: "architecture".into(),
        });

        questions.push(PolicyQuestion {
            id: "test_strategy".into(),
            text: "What's your testing strategy? (unit/integration/e2e/minimal)".into(),
            category: "testing".into(),
        });

        questions.push(PolicyQuestion {
            id: "deployment".into(),
            text: "How do you deploy? (docker/heroku/vps/serverless/other)".into(),
            category: "infrastructure".into(),
        });

        questions.push(PolicyQuestion {
            id: "code_review".into(),
            text: "Do you have code review requirements? (required/optional/solo)".into(),
            category: "process".into(),
        });

        questions.push(PolicyQuestion {
            id: "error_handling".into(),
            text: "Error handling preference: fail-fast or graceful degradation?".into(),
            category: "architecture".into(),
        });

        questions.push(PolicyQuestion {
            id: "async_strategy".into(),
            text: "For background tasks: signals, Celery, management commands, or other?".into(),
            category: "architecture".into(),
        });

        questions.push(PolicyQuestion {
            id: "api_versioning".into(),
            text: "API versioning strategy? (url-prefix/header/none)".into(),
            category: "api".into(),
        });

        questions.push(PolicyQuestion {
            id: "db_migrations".into(),
            text: "Database migration policy: auto-generate or hand-written?".into(),
            category: "database".into(),
        });

        questions.push(PolicyQuestion {
            id: "naming_convention".into(),
            text: "Any strict naming conventions for models/views/urls? (describe or 'none')".into(),
            category: "style".into(),
        });

        Ok(questions)
    }
}

struct PolicyQuestion {
    id: String,
    text: String,
    category: String,
}
