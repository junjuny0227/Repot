use repot_core::env::{Env, FakeEnv};
use repot_core::model::*;
use repot_core::os::Os;
use repot_core::scan::{Provider, ProviderError, ProviderStatus, mark_duplicates, scan};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Stub {
    id: &'static str,
    detect: bool,
    result: fn() -> Result<Vec<Item>, ProviderError>,
    delay_ms: u64,
}
impl Provider for Stub {
    fn id(&self) -> &'static str {
        self.id
    }
    fn category(&self) -> Category {
        Category::PackageManager
    }
    fn detect(&self, _: &dyn Env) -> bool {
        self.detect
    }
    fn scan(&self, _: &dyn Env) -> Result<Vec<Item>, ProviderError> {
        std::thread::sleep(Duration::from_millis(self.delay_ms));
        (self.result)()
    }
}
fn stub(
    id: &'static str,
    detect: bool,
    result: fn() -> Result<Vec<Item>, ProviderError>,
    delay_ms: u64,
) -> Arc<dyn Provider> {
    Arc::new(Stub {
        id,
        detect,
        result,
        delay_ms,
    })
}
fn pkg(provider: &str, name: &str) -> Item {
    Item {
        id: format!("{provider}:{name}"),
        provider: provider.into(),
        category: Category::PackageManager,
        name: name.into(),
        version: None,
        description: None,
        catalog_id: None,
        origin: Origin::Command("x".into()),
        portability: Portability::SameOs,
        details: ItemDetails::Package {
            manager: PackageManager::Brew,
            kind: PackageKind::Formula,
            url: None,
        },
        findings: vec![],
    }
}
fn env() -> Arc<dyn Env> {
    Arc::new(FakeEnv::new(Os::MacOs, "/h"))
}

#[test]
fn reports_each_status_and_isolates_failures() {
    let providers = vec![
        stub("ok", true, || Ok(vec![pkg("ok", "b"), pkg("ok", "a")]), 0),
        stub("absent", false, || panic!("must not scan"), 0),
        stub(
            "broken",
            true,
            || {
                Err(ProviderError::CommandFailed {
                    command: "x".into(),
                    code: Some(1),
                    stderr: "boom".into(),
                })
            },
            0,
        ),
        stub("slow", true, || Ok(vec![pkg("slow", "z")]), 2000),
    ];
    let seen = Mutex::new(vec![]);
    let start = Instant::now();
    let r = scan(env(), providers, Duration::from_millis(300), &|p| {
        seen.lock().unwrap().push(p.clone())
    });
    assert!(start.elapsed() < Duration::from_secs(1));
    let st: Vec<_> = r
        .providers
        .iter()
        .map(|p| (p.id.as_str(), p.status.clone()))
        .collect();
    assert_eq!(
        st,
        vec![
            ("ok", ProviderStatus::Ok { count: 2 }),
            ("absent", ProviderStatus::NotDetected),
            (
                "broken",
                ProviderStatus::Failed {
                    message: "`x` exited with code Some(1)".into(),
                    detail: "boom".into()
                }
            ),
            ("slow", ProviderStatus::TimedOut),
        ]
    );
    assert_eq!(
        r.items.iter().map(|i| i.name.as_str()).collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 8);
    for id in ["ok", "absent", "broken", "slow"] {
        let events: Vec<_> = seen.iter().filter(|p| p.id == id).collect();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].status, ProviderStatus::Running);
        assert_ne!(events[1].status, ProviderStatus::Running);
    }
}

#[test]
fn parallel_workers_share_one_deadline() {
    let providers = vec![
        stub("a", true, || Ok(vec![pkg("a", "Git")]), 200),
        stub("b", true, || Ok(vec![pkg("b", "git")]), 200),
        stub("c", true, || Ok(vec![]), 2000),
        stub("d", true, || Ok(vec![]), 2000),
    ];
    let start = Instant::now();
    let r = scan(env(), providers, Duration::from_millis(400), &|_| {});
    assert!(start.elapsed() < Duration::from_millis(750));
    assert_eq!(r.items.len(), 2);
    assert!(r.items.iter().all(|item| item.findings.len() == 1));
    assert_eq!(r.providers[2].status, ProviderStatus::TimedOut);
    assert_eq!(r.providers[3].status, ProviderStatus::TimedOut);
}

#[test]
fn sorting_error_mapping_empty_scan_and_serialization() {
    let providers = vec![
        stub(
            "z",
            true,
            || {
                let mut last = pkg("a", "a");
                last.category = Category::Shell;
                Ok(vec![pkg("z", "a"), last, pkg("a", "z")])
            },
            50,
        ),
        stub(
            "parse",
            true,
            || {
                Err(ProviderError::Parse {
                    command: "x".into(),
                    message: "invalid".into(),
                })
            },
            0,
        ),
        stub(
            "env",
            true,
            || Err(repot_core::env::EnvError::PermissionDenied.into()),
            0,
        ),
    ];
    let r = scan(env(), providers, Duration::from_secs(2), &|_| {});
    assert_eq!(
        r.items
            .iter()
            .map(|i| (i.provider.as_str(), i.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("a", "z"), ("z", "a"), ("a", "a")]
    );
    assert_eq!(r.providers[0].id, "z");
    assert_eq!(
        r.providers[1].status,
        ProviderStatus::Failed {
            message: "could not parse output of `x`: invalid".into(),
            detail: String::new()
        }
    );
    assert_eq!(
        r.providers[2].status,
        ProviderStatus::Failed {
            message: "environment error: permission denied".into(),
            detail: String::new()
        }
    );
    let json = serde_json::to_string(&r).unwrap();
    assert_eq!(
        serde_json::from_str::<repot_core::scan::ScanResult>(&json).unwrap(),
        r
    );
    assert_eq!(
        serde_json::to_value(ProviderStatus::NotDetected).unwrap(),
        "notDetected"
    );
    let empty = scan(env(), vec![], Duration::ZERO, &|_| panic!("no events"));
    assert!(empty.items.is_empty() && empty.providers.is_empty());
}

#[test]
fn duplicates_only_link_packages_from_other_providers_once() {
    let mut global = pkg("npm", "git");
    global.details = ItemDetails::GlobalPackage {
        manager: GlobalManager::Npm,
    };
    let mut items = vec![pkg("a", "Git"), pkg("a", "git"), global];
    mark_duplicates(&mut items);
    assert!(items.iter().all(|item| item.findings.is_empty()));
    items.push(pkg("b", "GIT"));
    mark_duplicates(&mut items);
    let expected = items.clone();
    mark_duplicates(&mut items);
    assert_eq!(items, expected);
    assert_eq!(items[3].findings.len(), 1);
}

#[test]
fn marks_duplicates_case_insensitively() {
    let mut items = vec![
        pkg("winget", "Git"),
        pkg("choco", "git"),
        pkg("choco", "7zip"),
    ];
    mark_duplicates(&mut items);
    assert_eq!(
        items[0].findings,
        vec![Finding {
            line: None,
            kind: FindingKind::Duplicate {
                other_provider: "choco".into()
            }
        }]
    );
    assert_eq!(
        items[1].findings,
        vec![Finding {
            line: None,
            kind: FindingKind::Duplicate {
                other_provider: "winget".into()
            }
        }]
    );
    assert!(items[2].findings.is_empty());
}
