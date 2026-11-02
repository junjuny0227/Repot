use crate::env::{Env, EnvError};
use crate::model::{Category, Finding, FindingKind, Item, ItemDetails};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

pub trait Provider: Send + Sync {
    fn id(&self) -> &'static str;
    fn category(&self) -> Category;
    fn detect(&self, env: &dyn Env) -> bool;
    fn scan(&self, env: &dyn Env) -> Result<Vec<Item>, ProviderError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("`{command}` exited with code {code:?}")]
    CommandFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },
    #[error("could not parse output of `{command}`: {message}")]
    Parse { command: String, message: String },
    #[error("environment error: {0}")]
    Env(#[from] EnvError),
}

pub const PROVIDER_TIMEOUT: Duration = Duration::from_secs(30);
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub items: Vec<Item>,
    pub providers: Vec<ProviderReport>,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderReport {
    pub id: String,
    pub category: Category,
    pub status: ProviderStatus,
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ProviderStatus {
    Running,
    Ok { count: usize },
    NotDetected,
    Failed { message: String, detail: String },
    TimedOut,
}

/// Runs all providers against one deadline. Timed-out workers are detached:
/// Rust threads cannot be forcibly cancelled; late results are discarded.
pub fn scan(
    env: Arc<dyn Env>,
    providers: Vec<Arc<dyn Provider>>,
    timeout: Duration,
    on_progress: &(dyn Fn(&ProviderReport) + Sync),
) -> ScanResult {
    let start = Instant::now();
    let (tx, rx) = mpsc::channel();
    let mut reports: Vec<_> = providers
        .iter()
        .map(|provider| ProviderReport {
            id: provider.id().into(),
            category: provider.category(),
            status: ProviderStatus::Running,
        })
        .collect();
    for (index, provider) in providers.into_iter().enumerate() {
        on_progress(&reports[index]);
        let env = Arc::clone(&env);
        let tx = tx.clone();
        std::thread::spawn(move || {
            let (status, items) = if !provider.detect(env.as_ref()) {
                (ProviderStatus::NotDetected, vec![])
            } else {
                match provider.scan(env.as_ref()) {
                    Ok(items) => (ProviderStatus::Ok { count: items.len() }, items),
                    Err(err) => {
                        let detail = match &err {
                            ProviderError::CommandFailed { stderr, .. } => stderr.clone(),
                            _ => String::new(),
                        };
                        (
                            ProviderStatus::Failed {
                                message: err.to_string(),
                                detail,
                            },
                            vec![],
                        )
                    }
                }
            };
            let _ = tx.send((index, start.elapsed(), status, items));
        });
    }
    drop(tx);
    let mut items = vec![];
    let mut pending = reports.len();
    while pending > 0 {
        match rx.recv_timeout(timeout.saturating_sub(start.elapsed())) {
            Ok((index, elapsed, status, found)) if elapsed <= timeout => {
                reports[index].status = status;
                items.extend(found);
                on_progress(&reports[index]);
                pending -= 1;
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    for report in &mut reports {
        if report.status == ProviderStatus::Running {
            report.status = ProviderStatus::TimedOut;
            on_progress(report);
        }
    }
    items.sort_by(|a, b| {
        (&a.category, &a.provider, &a.name).cmp(&(&b.category, &b.provider, &b.name))
    });
    mark_duplicates(&mut items);
    ScanResult {
        items,
        providers: reports,
    }
}

pub fn mark_duplicates(items: &mut [Item]) {
    let mut groups: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for item in items
        .iter()
        .filter(|item| matches!(item.details, ItemDetails::Package { .. }))
    {
        groups
            .entry(item.name.to_lowercase())
            .or_default()
            .insert(item.provider.clone());
    }
    for item in items
        .iter_mut()
        .filter(|item| matches!(item.details, ItemDetails::Package { .. }))
    {
        for provider in &groups[&item.name.to_lowercase()] {
            if *provider != item.provider {
                let finding = Finding {
                    line: None,
                    kind: FindingKind::Duplicate {
                        other_provider: provider.clone(),
                    },
                };
                if !item.findings.contains(&finding) {
                    item.findings.push(finding);
                }
            }
        }
    }
}
