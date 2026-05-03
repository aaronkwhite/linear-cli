use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Write-only PostHog project token. Safe to embed — cannot read data, only send events.
/// Alternative: use compile-time env var instead:
///   const POSTHOG_TOKEN: &str = env!("LIN_POSTHOG_TOKEN");
/// This requires LIN_POSTHOG_TOKEN set at build time (add to CI secrets).
const POSTHOG_TOKEN: &str = "phc_3DIgL4ES4ukoFmH4hgg3jR0e6O52PiQIfzfsVEjJu9u";

const POSTHOG_BATCH_URL: &str = "https://app.posthog.com/batch/";

/// Bumped whenever the event property shape changes.
const SCHEMA_VERSION: u32 = 1;

/// Cap the queue at 1000 events. On overflow, the oldest are dropped.
const MAX_QUEUE_EVENTS: usize = 1000;

/// Only attempt to flush at most once every 10 minutes.
const FLUSH_DEBOUNCE: Duration = Duration::from_secs(600);

const QUEUE_FILE: &str = "analytics_queue.jsonl";
const INSTALL_ID_FILE: &str = "analytics_id";
const FLUSH_SENTINEL_FILE: &str = "analytics_last_flush";

/// Check if analytics are enabled. Checks in order:
/// 1. DO_NOT_TRACK set to any non-empty value → disabled (consoledonottrack.com convention)
/// 2. Config `analytics_enabled == Some(false)` → disabled
/// 3. Otherwise → enabled
pub fn is_enabled() -> bool {
    is_enabled_for(do_not_track_set(), crate::config::is_analytics_enabled())
}

fn do_not_track_set() -> bool {
    std::env::var("DO_NOT_TRACK")
        .ok()
        .is_some_and(|v| !v.is_empty())
}

/// Pure function — testable without env or filesystem.
fn is_enabled_for(do_not_track: bool, config_enabled: bool) -> bool {
    if do_not_track {
        return false;
    }
    config_enabled
}

pub struct Event {
    pub command: String,
    pub flags: Vec<String>,
    pub success: bool,
    pub duration_ms: u64,
}

/// Track a command execution event. Appends to the local queue file unless this is
/// the first run (in which case the install ID is minted and the user sees the notice,
/// but no event is queued — the user should see the notice before any data is shipped).
///
/// In a non-interactive session (no TTY on stderr) where no install ID exists yet,
/// nothing is collected — this protects AI agents and CI pipelines from being
/// implicitly opted in.
pub fn track(event: &Event) {
    if !is_enabled() {
        return;
    }
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    track_to_dir(&dir, event, crate::output::interactive::is_interactive());
}

fn track_to_dir(dir: &Path, event: &Event, interactive: bool) -> bool {
    let install_id_path = dir.join(INSTALL_ID_FILE);

    // First-run path: no install ID exists yet.
    if !install_id_path.exists() {
        if !interactive {
            // Non-interactive (agent / CI / piped): default off until user opts in.
            return false;
        }
        // Interactive first run: print notice, mint UUID, do NOT queue this event.
        // The notice must be visible BEFORE any data leaves the machine.
        if fs::create_dir_all(dir).is_err() {
            return false;
        }
        let id = uuid::Uuid::new_v4().to_string();
        if fs::write(&install_id_path, &id).is_err() {
            return false;
        }
        eprintln!(
            "lin: anonymous usage stats enabled. \
             Disable: `lin config analytics off`, or set DO_NOT_TRACK=1."
        );
        return true;
    }

    // Subsequent runs: read the install ID and queue the event.
    let install_id = match fs::read_to_string(&install_id_path) {
        Ok(s) => {
            let trimmed = s.trim().to_string();
            if trimmed.is_empty() {
                return false;
            }
            trimmed
        }
        Err(_) => return false,
    };

    let payload = serde_json::json!({
        "event": "command_executed",
        "distinct_id": install_id,
        "properties": {
            "command": event.command,
            "flags": event.flags,
            "success": event.success,
            "duration_ms": event.duration_ms,
            "version": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "schema_version": SCHEMA_VERSION,
        }
    });

    let queue_path = dir.join(QUEUE_FILE);
    let Ok(line) = serde_json::to_string(&payload) else {
        return false;
    };

    let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&queue_path)
    else {
        return false;
    };
    let _ = writeln!(file, "{line}");
    drop(file);

    cap_queue(&queue_path, MAX_QUEUE_EVENTS);

    false
}

/// Trim the queue file to keep only the most recent `max` events.
/// FIFO: oldest events drop first.
fn cap_queue(path: &Path, max: usize) {
    let Ok(contents) = fs::read_to_string(path) else {
        return;
    };
    let lines: Vec<&str> = contents.lines().filter(|l| !l.is_empty()).collect();
    if lines.len() <= max {
        return;
    }
    let kept: String = lines[lines.len() - max..]
        .iter()
        .map(|l| format!("{l}\n"))
        .collect();
    let _ = fs::write(path, kept);
}

/// Returns true if a flush attempt should be made. Used by main.rs to skip the
/// flush spawn entirely on most invocations (which would otherwise add up to 3s
/// of latency to every command).
pub fn flush_due() -> bool {
    let Some(dir) = crate::config::config_dir() else {
        return false;
    };
    flush_due_in(&dir, FLUSH_DEBOUNCE)
}

fn flush_due_in(dir: &Path, debounce: Duration) -> bool {
    let queue_path = dir.join(QUEUE_FILE);
    match fs::metadata(&queue_path) {
        Ok(m) if m.len() > 0 => {}
        _ => return false,
    }

    let sentinel = dir.join(FLUSH_SENTINEL_FILE);
    match fs::metadata(&sentinel).and_then(|m| m.modified()) {
        Ok(t) => SystemTime::now()
            .duration_since(t)
            .map(|d| d >= debounce)
            .unwrap_or(true),
        Err(_) => true,
    }
}

/// Flush pending analytics events to PostHog.
pub async fn flush() {
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    flush_dir(&dir, POSTHOG_BATCH_URL).await;
}

/// Flush events from a specific directory to a specific URL. Testable.
async fn flush_dir(dir: &Path, url: &str) {
    let queue_path = dir.join(QUEUE_FILE);
    if !queue_path.exists() {
        return;
    }

    // Touch the sentinel before flushing so that concurrent / subsequent
    // invocations skip flushing for the debounce window.
    let sentinel = dir.join(FLUSH_SENTINEL_FILE);
    let _ = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&sentinel);

    // Atomically take ownership of the queue. If the rename fails, another
    // process beat us to it (or the file disappeared) — nothing to do.
    let flushing_path = match flushing_path(dir) {
        Some(p) => p,
        None => return,
    };
    if fs::rename(&queue_path, &flushing_path).is_err() {
        return;
    }

    let contents = fs::read_to_string(&flushing_path).unwrap_or_default();
    let events: Vec<serde_json::Value> = contents
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    if events.is_empty() {
        let _ = fs::remove_file(&flushing_path);
        return;
    }

    let batch_payload = serde_json::json!({
        "api_key": POSTHOG_TOKEN,
        "batch": events,
    });

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => {
            restore_queue(&flushing_path, &queue_path);
            return;
        }
    };

    match client.post(url).json(&batch_payload).send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() || status.is_client_error() {
                // 2xx: delivered. 4xx: poison pill — drop so we don't retry forever.
                let _ = fs::remove_file(&flushing_path);
            } else {
                // 5xx, redirect, anything else transient — retain for next attempt.
                restore_queue(&flushing_path, &queue_path);
            }
        }
        Err(_) => {
            restore_queue(&flushing_path, &queue_path);
        }
    }
}

fn flushing_path(dir: &Path) -> Option<PathBuf> {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_nanos();
    Some(dir.join(format!("analytics_queue.flushing.{pid}.{nanos}.jsonl")))
}

/// Append flushing-file contents back to the live queue, then delete the flushing file.
/// Called when delivery fails for a transient reason (5xx, network error).
fn restore_queue(flushing_path: &Path, queue_path: &Path) {
    if let Ok(contents) = fs::read_to_string(flushing_path)
        && let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(queue_path)
    {
        let _ = file.write_all(contents.as_bytes());
        cap_queue(queue_path, MAX_QUEUE_EVENTS);
    }
    let _ = fs::remove_file(flushing_path);
}

/// Delete all analytics artifacts (queue, install ID, sentinel) from the config directory.
/// Called on `lin config analytics off` so opting out is a clean break — no residual data.
pub fn purge_artifacts() {
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    purge_artifacts_in(&dir);
}

fn purge_artifacts_in(dir: &Path) {
    let _ = fs::remove_file(dir.join(QUEUE_FILE));
    let _ = fs::remove_file(dir.join(INSTALL_ID_FILE));
    let _ = fs::remove_file(dir.join(FLUSH_SENTINEL_FILE));
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Some(s) = name.to_str()
                && s.starts_with("analytics_queue.flushing.")
            {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

/// Drop the install ID so a fresh UUID is minted on the next run. Called on
/// `lin config analytics on` so re-enabling does not re-correlate to the prior identity.
pub fn reset_install_id() {
    let Some(dir) = crate::config::config_dir() else {
        return;
    };
    let _ = fs::remove_file(dir.join(INSTALL_ID_FILE));
}

/// Build the dotted command name (e.g. `"issues list"`, `"config analytics on"`)
/// from the parsed CLI. Subcommand granularity matters for product analytics —
/// `issues list` and `issues create` are very different signals.
pub fn command_name(cmd: &crate::cli::Commands) -> String {
    use crate::cli::Commands;
    match cmd {
        Commands::Api(_) => "api".into(),
        Commands::Auth(args) => format!("auth {}", auth_sub(&args.command)),
        Commands::Issues(args) => format!("issues {}", issues_sub(&args.command)),
        Commands::Projects(args) => format!("projects {}", projects_sub(&args.command)),
        Commands::Cycles(args) => format!("cycles {}", cycles_sub(&args.command)),
        Commands::Initiatives(args) => format!("initiatives {}", initiatives_sub(&args.command)),
        Commands::Roadmap(args) => format!("roadmap {}", roadmap_sub(&args.command)),
        Commands::Labels(args) => format!("labels {}", labels_sub(&args.command)),
        Commands::Teams(args) => format!("teams {}", teams_sub(&args.command)),
        Commands::Relations(args) => format!("relations {}", relations_sub(&args.command)),
        Commands::Customers(args) => format!("customers {}", customers_sub(&args.command)),
        Commands::Views(args) => format!("views {}", views_sub(&args.command)),
        Commands::Docs(args) => format!("docs {}", docs_sub(&args.command)),
        Commands::Notifications(args) => {
            format!("notifications {}", notifications_sub(&args.command))
        }
        Commands::Me(_) => "me".into(),
        Commands::Attachments(args) => format!("attachments {}", attachments_sub(&args.command)),
        Commands::Search(_) => "search".into(),
        Commands::Config(args) => format!("config {}", config_sub(&args.command)),
        Commands::Completions { .. } => "completions".into(),
    }
}

fn auth_sub(c: &crate::commands::auth::AuthCommand) -> &'static str {
    use crate::commands::auth::AuthCommand;
    match c {
        AuthCommand::Login { .. } => "login",
        AuthCommand::List => "list",
        AuthCommand::Default { .. } => "default",
        AuthCommand::Whoami => "whoami",
    }
}

fn issues_sub(c: &crate::commands::issues::IssuesCommand) -> &'static str {
    use crate::commands::issues::IssuesCommand;
    match c {
        IssuesCommand::Get { .. } => "get",
        IssuesCommand::List { .. } => "list",
        IssuesCommand::Search { .. } => "search",
        IssuesCommand::Create { .. } => "create",
        IssuesCommand::Update { .. } => "update",
        IssuesCommand::Comment { .. } => "comment",
        IssuesCommand::Archive { .. } => "archive",
        IssuesCommand::Branch { .. } => "branch",
        IssuesCommand::Delete { .. } => "delete",
        IssuesCommand::Unarchive { .. } => "unarchive",
        IssuesCommand::Subscribe { .. } => "subscribe",
        IssuesCommand::Unsubscribe { .. } => "unsubscribe",
        IssuesCommand::Start { .. } => "start",
        IssuesCommand::Pr { .. } => "pr",
    }
}

fn projects_sub(c: &crate::commands::projects::ProjectsCommand) -> &'static str {
    use crate::commands::projects::ProjectsCommand;
    match c {
        ProjectsCommand::Get { .. } => "get",
        ProjectsCommand::List { .. } => "list",
        ProjectsCommand::Issues { .. } => "issues",
        ProjectsCommand::Create { .. } => "create",
        ProjectsCommand::Update { .. } => "update",
        ProjectsCommand::Search { .. } => "search",
        ProjectsCommand::Archive { .. } => "archive",
        ProjectsCommand::Unarchive { .. } => "unarchive",
        ProjectsCommand::Delete { .. } => "delete",
    }
}

fn cycles_sub(c: &crate::commands::cycles::CyclesCommand) -> &'static str {
    use crate::commands::cycles::CyclesCommand;
    match c {
        CyclesCommand::List { .. } => "list",
        CyclesCommand::Get { .. } => "get",
        CyclesCommand::Issues { .. } => "issues",
        CyclesCommand::Create { .. } => "create",
        CyclesCommand::Update { .. } => "update",
        CyclesCommand::Add { .. } => "add",
        CyclesCommand::Remove { .. } => "remove",
        CyclesCommand::Archive { .. } => "archive",
    }
}

fn initiatives_sub(c: &crate::commands::initiatives::InitiativesCommand) -> &'static str {
    use crate::commands::initiatives::InitiativesCommand;
    match c {
        InitiativesCommand::List { .. } => "list",
        InitiativesCommand::Get { .. } => "get",
        InitiativesCommand::Create { .. } => "create",
        InitiativesCommand::Update { .. } => "update",
        InitiativesCommand::Archive { .. } => "archive",
        InitiativesCommand::Delete { .. } => "delete",
        InitiativesCommand::Projects { .. } => "projects",
        InitiativesCommand::Updates { .. } => "updates",
        InitiativesCommand::PostUpdate { .. } => "post-update",
        InitiativesCommand::AddProject { .. } => "add-project",
        InitiativesCommand::RemoveProject { .. } => "remove-project",
    }
}

fn roadmap_sub(c: &crate::commands::roadmap::RoadmapCommand) -> &'static str {
    use crate::commands::roadmap::RoadmapCommand;
    match c {
        RoadmapCommand::Updates { .. } => "updates",
        RoadmapCommand::Post { .. } => "post",
        RoadmapCommand::Milestones { .. } => "milestones",
        RoadmapCommand::CreateMilestone { .. } => "create-milestone",
        RoadmapCommand::UpdateMilestone { .. } => "update-milestone",
        RoadmapCommand::DeleteMilestone { .. } => "delete-milestone",
    }
}

fn labels_sub(c: &crate::commands::labels::LabelsCommand) -> &'static str {
    use crate::commands::labels::LabelsCommand;
    match c {
        LabelsCommand::List { .. } => "list",
        LabelsCommand::Create { .. } => "create",
        LabelsCommand::Update { .. } => "update",
        LabelsCommand::Delete { .. } => "delete",
        LabelsCommand::Apply { .. } => "apply",
        LabelsCommand::Remove { .. } => "remove",
        LabelsCommand::Usage { .. } => "usage",
    }
}

fn teams_sub(c: &crate::commands::teams::TeamsCommand) -> &'static str {
    use crate::commands::teams::TeamsCommand;
    match c {
        TeamsCommand::List => "list",
        TeamsCommand::Get { .. } => "get",
        TeamsCommand::Members { .. } => "members",
        TeamsCommand::States { .. } => "states",
        TeamsCommand::Workload { .. } => "workload",
    }
}

fn relations_sub(c: &crate::commands::relations::RelationsCommand) -> &'static str {
    use crate::commands::relations::RelationsCommand;
    match c {
        RelationsCommand::List { .. } => "list",
        RelationsCommand::Blocks { .. } => "blocks",
        RelationsCommand::BlockedBy { .. } => "blocked-by",
        RelationsCommand::Relates { .. } => "relates",
        RelationsCommand::Duplicate { .. } => "duplicate",
        RelationsCommand::Remove { .. } => "remove",
    }
}

fn customers_sub(c: &crate::commands::customers::CustomersCommand) -> &'static str {
    use crate::commands::customers::CustomersCommand;
    match c {
        CustomersCommand::List { .. } => "list",
        CustomersCommand::Create { .. } => "create",
        CustomersCommand::Update { .. } => "update",
        CustomersCommand::Delete { .. } => "delete",
        CustomersCommand::Link { .. } => "link",
        CustomersCommand::Needs { .. } => "needs",
        CustomersCommand::Tiers => "tiers",
        CustomersCommand::CreateTier { .. } => "create-tier",
    }
}

fn views_sub(c: &crate::commands::views::ViewsCommand) -> &'static str {
    use crate::commands::views::ViewsCommand;
    match c {
        ViewsCommand::List { .. } => "list",
        ViewsCommand::Get { .. } => "get",
        ViewsCommand::Create { .. } => "create",
        ViewsCommand::Update { .. } => "update",
        ViewsCommand::Delete { .. } => "delete",
        ViewsCommand::Issues { .. } => "issues",
    }
}

fn docs_sub(c: &crate::commands::docs::DocsCommand) -> &'static str {
    use crate::commands::docs::DocsCommand;
    match c {
        DocsCommand::List { .. } => "list",
        DocsCommand::Get { .. } => "get",
        DocsCommand::Search { .. } => "search",
        DocsCommand::Create { .. } => "create",
        DocsCommand::Update { .. } => "update",
        DocsCommand::Delete { .. } => "delete",
    }
}

fn notifications_sub(c: &crate::commands::notifications::NotificationsCommand) -> &'static str {
    use crate::commands::notifications::NotificationsCommand;
    match c {
        NotificationsCommand::List { .. } => "list",
        NotificationsCommand::Read { .. } => "read",
        NotificationsCommand::Archive { .. } => "archive",
        NotificationsCommand::Snooze { .. } => "snooze",
        NotificationsCommand::Unsnooze { .. } => "unsnooze",
    }
}

fn attachments_sub(c: &crate::commands::attachments::AttachmentsCommand) -> &'static str {
    use crate::commands::attachments::AttachmentsCommand;
    match c {
        AttachmentsCommand::List { .. } => "list",
        AttachmentsCommand::Create { .. } => "create",
        AttachmentsCommand::LinkUrl { .. } => "link-url",
        AttachmentsCommand::Delete { .. } => "delete",
    }
}

fn config_sub(c: &crate::commands::config::ConfigCommand) -> String {
    use crate::commands::config::{AnalyticsCommand, ConfigCommand};
    match c {
        ConfigCommand::SetToken => "set-token".into(),
        ConfigCommand::GetToken => "get-token".into(),
        ConfigCommand::Path => "path".into(),
        ConfigCommand::Analytics(args) => {
            let sub = match args.command {
                AnalyticsCommand::On => "on",
                AnalyticsCommand::Off => "off",
                AnalyticsCommand::Status => "status",
            };
            format!("analytics {sub}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("lin-test-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn make_event() -> Event {
        Event {
            command: "issues list".to_string(),
            flags: vec!["--json".to_string()],
            success: true,
            duration_ms: 100,
        }
    }

    // ---- is_enabled (pure) ----

    #[test]
    fn is_enabled_default_when_config_enabled() {
        assert!(is_enabled_for(false, true));
    }

    #[test]
    fn is_enabled_disabled_via_config() {
        assert!(!is_enabled_for(false, false));
    }

    #[test]
    fn do_not_track_overrides_config_enabled() {
        assert!(!is_enabled_for(true, true));
    }

    #[test]
    fn do_not_track_disables_even_when_config_disabled() {
        assert!(!is_enabled_for(true, false));
    }

    // ---- DO_NOT_TRACK env parsing (serialized to avoid env-var races) ----

    #[test]
    #[serial_test::serial]
    fn do_not_track_unset_returns_false() {
        temp_env::with_var_unset("DO_NOT_TRACK", || {
            assert!(!do_not_track_set());
        });
    }

    #[test]
    #[serial_test::serial]
    fn do_not_track_one_disables() {
        temp_env::with_var("DO_NOT_TRACK", Some("1"), || {
            assert!(do_not_track_set());
        });
    }

    #[test]
    #[serial_test::serial]
    fn do_not_track_any_value_disables() {
        // Per consoledonottrack.com, any non-empty value should disable.
        for v in ["true", "yes", "0", "anything"] {
            temp_env::with_var("DO_NOT_TRACK", Some(v), || {
                assert!(
                    do_not_track_set(),
                    "DO_NOT_TRACK={v} should disable analytics"
                );
            });
        }
    }

    #[test]
    #[serial_test::serial]
    fn do_not_track_empty_value_does_not_disable() {
        temp_env::with_var("DO_NOT_TRACK", Some(""), || {
            assert!(!do_not_track_set());
        });
    }

    // ---- First-run behavior ----

    #[test]
    fn first_run_interactive_prints_notice_and_does_not_queue() {
        let dir = tempdir();

        let first = track_to_dir(&dir, &make_event(), true);
        assert!(first, "first interactive run should report first_run=true");

        // Install ID minted...
        assert!(dir.join(INSTALL_ID_FILE).exists());
        // ...but no queued event.
        assert!(!dir.join(QUEUE_FILE).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn first_run_non_interactive_collects_nothing() {
        let dir = tempdir();

        let first = track_to_dir(&dir, &make_event(), false);
        assert!(!first);

        // No install ID minted, no queue file written.
        assert!(!dir.join(INSTALL_ID_FILE).exists());
        assert!(!dir.join(QUEUE_FILE).exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn second_run_queues_event() {
        let dir = tempdir();

        // First run: notice, no queue.
        track_to_dir(&dir, &make_event(), true);
        // Second run: should queue.
        let first = track_to_dir(&dir, &make_event(), true);
        assert!(!first);

        let queue = fs::read_to_string(dir.join(QUEUE_FILE)).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(queue.trim()).unwrap();
        assert_eq!(parsed["event"], "command_executed");
        assert_eq!(parsed["properties"]["command"], "issues list");
        assert_eq!(parsed["properties"]["schema_version"], 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn non_interactive_after_install_id_exists_still_queues() {
        // Once a user has opted in interactively, agent invocations from the same
        // machine should keep tracking — they're authorized by the existing UUID.
        let dir = tempdir();
        fs::write(dir.join(INSTALL_ID_FILE), "existing-uuid").unwrap();

        track_to_dir(&dir, &make_event(), false);

        let queue = fs::read_to_string(dir.join(QUEUE_FILE)).unwrap();
        assert!(queue.contains("\"distinct_id\":\"existing-uuid\""));

        let _ = fs::remove_dir_all(&dir);
    }

    // ---- Queue cap ----

    #[test]
    fn cap_queue_keeps_most_recent() {
        let dir = tempdir();
        let path = dir.join(QUEUE_FILE);
        let lines: String = (0..10).map(|i| format!("event-{i}\n")).collect();
        fs::write(&path, lines).unwrap();

        cap_queue(&path, 3);

        let after = fs::read_to_string(&path).unwrap();
        let kept: Vec<&str> = after.lines().collect();
        assert_eq!(kept, vec!["event-7", "event-8", "event-9"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cap_queue_noop_when_under_limit() {
        let dir = tempdir();
        let path = dir.join(QUEUE_FILE);
        fs::write(&path, "a\nb\nc\n").unwrap();

        cap_queue(&path, 10);

        let after = fs::read_to_string(&path).unwrap();
        assert_eq!(after, "a\nb\nc\n");

        let _ = fs::remove_dir_all(&dir);
    }

    // ---- Flush debounce ----

    #[test]
    fn flush_due_false_when_queue_missing() {
        let dir = tempdir();
        assert!(!flush_due_in(&dir, Duration::from_secs(60)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn flush_due_false_when_queue_empty() {
        let dir = tempdir();
        fs::write(dir.join(QUEUE_FILE), "").unwrap();
        assert!(!flush_due_in(&dir, Duration::from_secs(60)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn flush_due_true_when_no_sentinel_and_queue_has_data() {
        let dir = tempdir();
        fs::write(dir.join(QUEUE_FILE), "{}\n").unwrap();
        assert!(flush_due_in(&dir, Duration::from_secs(60)));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn flush_due_false_when_recently_flushed() {
        let dir = tempdir();
        fs::write(dir.join(QUEUE_FILE), "{}\n").unwrap();
        // Touch sentinel just now → debounce window not elapsed.
        fs::write(dir.join(FLUSH_SENTINEL_FILE), "").unwrap();
        assert!(!flush_due_in(&dir, Duration::from_secs(60)));
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- Flush + 4xx / 5xx handling ----

    #[tokio::test]
    async fn flush_2xx_drops_events() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::method("POST"))
            .and(matchers::path("/batch/"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let dir = tempdir();
        fs::write(dir.join(INSTALL_ID_FILE), "test-id").unwrap();
        track_to_dir(&dir, &make_event(), false);
        track_to_dir(&dir, &make_event(), false);

        flush_dir(&dir, &format!("{}/batch/", server.uri())).await;

        assert!(
            !dir.join(QUEUE_FILE).exists()
                || fs::metadata(dir.join(QUEUE_FILE)).unwrap().len() == 0
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn flush_5xx_retains_events() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::method("POST"))
            .and(matchers::path("/batch/"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let dir = tempdir();
        fs::write(dir.join(INSTALL_ID_FILE), "test-id").unwrap();
        track_to_dir(&dir, &make_event(), false);

        flush_dir(&dir, &format!("{}/batch/", server.uri())).await;

        // Events restored to the queue file.
        let queue = fs::read_to_string(dir.join(QUEUE_FILE)).unwrap();
        assert!(queue.contains("command_executed"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn flush_4xx_drops_events() {
        use wiremock::{Mock, MockServer, ResponseTemplate, matchers};

        let server = MockServer::start().await;
        Mock::given(matchers::method("POST"))
            .and(matchers::path("/batch/"))
            .respond_with(ResponseTemplate::new(400))
            .expect(1)
            .mount(&server)
            .await;

        let dir = tempdir();
        fs::write(dir.join(INSTALL_ID_FILE), "test-id").unwrap();
        track_to_dir(&dir, &make_event(), false);

        flush_dir(&dir, &format!("{}/batch/", server.uri())).await;

        // 4xx is a poison pill — events should be dropped, not retried forever.
        let queue_size = fs::metadata(dir.join(QUEUE_FILE))
            .map(|m| m.len())
            .unwrap_or(0);
        assert_eq!(queue_size, 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn flush_noop_on_missing_queue() {
        let dir = tempdir();
        flush_dir(&dir, "http://localhost:1/batch/").await;
        assert!(!dir.join(QUEUE_FILE).exists());
        let _ = fs::remove_dir_all(&dir);
    }

    // ---- Artifact cleanup ----

    #[test]
    fn purge_artifacts_removes_all_analytics_files() {
        let dir = tempdir();
        fs::write(dir.join(INSTALL_ID_FILE), "id").unwrap();
        fs::write(dir.join(QUEUE_FILE), "{}").unwrap();
        fs::write(dir.join(FLUSH_SENTINEL_FILE), "").unwrap();
        fs::write(dir.join("analytics_queue.flushing.123.456.jsonl"), "{}").unwrap();
        // Unrelated file should survive.
        fs::write(dir.join("config.toml"), "key = 1").unwrap();

        purge_artifacts_in(&dir);

        assert!(!dir.join(INSTALL_ID_FILE).exists());
        assert!(!dir.join(QUEUE_FILE).exists());
        assert!(!dir.join(FLUSH_SENTINEL_FILE).exists());
        assert!(!dir.join("analytics_queue.flushing.123.456.jsonl").exists());
        assert!(dir.join("config.toml").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    // ---- command_name ----

    #[test]
    fn command_name_includes_subcommand() {
        use crate::cli::Commands;
        use crate::commands::issues::{IssuesArgs, IssuesCommand};

        let cmd = Commands::Issues(IssuesArgs {
            command: IssuesCommand::List {
                team: None,
                all_teams: false,
                state: None,
                assignee: None,
                priority: None,
                label: vec![],
                created_after: None,
                updated_after: None,
                limit: 50,
            },
        });
        assert_eq!(command_name(&cmd), "issues list");
    }

    #[test]
    fn command_name_handles_nested_analytics() {
        use crate::cli::Commands;
        use crate::commands::config::{AnalyticsArgs, AnalyticsCommand, ConfigArgs, ConfigCommand};

        let cmd = Commands::Config(ConfigArgs {
            command: ConfigCommand::Analytics(AnalyticsArgs {
                command: AnalyticsCommand::Off,
            }),
        });
        assert_eq!(command_name(&cmd), "config analytics off");
    }

    #[test]
    fn command_name_top_level_only_for_unparameterized() {
        use crate::cli::Commands;
        use crate::commands::me::MeArgs;

        let cmd = Commands::Me(MeArgs {});
        assert_eq!(command_name(&cmd), "me");
    }
}
