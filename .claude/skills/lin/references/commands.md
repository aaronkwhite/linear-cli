# lin Command Reference

All commands accept `--json` for compact JSON output and `--debug` to print raw GraphQL to stderr.

---

## Issues

```
lin issues get <IDENTIFIER>
  --json

lin issues list
  --team <TEAM>        Team key or name
  --all-teams            Query across all teams (conflicts with --team)
  --state <STATE>      Workflow state name (alias: --status)
  --assignee <NAME>    Filter by assignee name
  --priority <1-4>     1=Urgent 2=High 3=Medium 4=Low
  --label <NAME>       Label name (repeatable for multiple)
  --created-after <DATE> Only issues created on or after this date (YYYY-MM-DD)
  --updated-after <DATE> Only issues updated on or after this date (YYYY-MM-DD)
  --limit <N>          Max results [default: 50]
  --json

lin issues search <QUERY>
  --limit <N>          [default: 50]
  --json

lin issues create
  --team <TEAM>        REQUIRED. Team key or name
  --title <TITLE>      REQUIRED. Issue title
  --description <TEXT> Body text
  --description-file <PATH>  Read description from a file (conflicts with --description)
  --assignee <NAME>    Assignee display name
  --priority <1-4>     1=Urgent 2=High 3=Medium 4=Low
  --estimate <N>       Story points
  --due-date <DATE>    YYYY-MM-DD
  --label <NAME>       Label name
  --parent <ID>        Parent issue identifier (makes sub-issue)
  --project <NAME>     Project name
  --status <STATUS>    Initial state name
  --json

lin issues update <IDENTIFIER>
  --status <STATUS>    New state name
  --assignee <NAME>    New assignee
  --priority <1-4>
  --estimate <N>
  --due-date <DATE>
  --parent <ID>        Set parent (makes sub-issue)
  --project <NAME>     Move to project
  --label <NAME>       Set label
  --milestone <NAME>   Assign to milestone
  --team <TEAM>        Move issue to a different team (key or name)
  --description-file <PATH>  Read description from a file
  --json

lin issues comment <IDENTIFIER> [BODY]
  --body-file <PATH>   Read body from a file
  --json

lin issues archive <IDENTIFIER>
lin issues unarchive <IDENTIFIER>
lin issues delete <IDENTIFIER>
lin issues subscribe <IDENTIFIER>
lin issues unsubscribe <IDENTIFIER>
lin issues branch <BRANCH_NAME>    # find issue by git branch name

lin issues start <IDENTIFIER>
  --status <STATUS>      Update issue status after branching
  --print-only           Just print the branch name
  --json

lin issues pr <IDENTIFIER>
  --draft                Create as draft PR
  --base <BRANCH>        Base branch
```

---

## Projects

```
lin projects list
  --status <STATUS>    Filter by status
  --team <KEY>         Filter by team key
  --limit <N>          [default: 50]
  --json

lin projects get <NAME>
  --json

lin projects issues <NAME>
  --json

lin projects create
  --name <NAME>        REQUIRED. Project name
  --team <TEAM>        REQUIRED. Team key or name
  --description <TEXT>
  --lead <NAME>        Lead user name
  --start-date <DATE>  YYYY-MM-DD
  --target-date <DATE> YYYY-MM-DD
  --json

lin projects update <NAME>
  --name <NEW_NAME>
  --description <TEXT>
  --lead <NAME>
  --status <STATUS>
  --start-date <DATE>
  --target-date <DATE>
  --add-team <TEAM>    Add a team to this project (key or name)
  --json

lin projects archive <NAME>
lin projects unarchive <NAME>
lin projects delete <NAME>
lin projects search <QUERY> --json
```

---

## Cycles (Sprints)

```
lin cycles list
  --team <TEAM>        REQUIRED. Team key or name
  --json

lin cycles get <CYCLE_ID>
  --json

lin cycles issues
  --team <TEAM>        REQUIRED. Active cycle issues
  --json

lin cycles create
  --team <TEAM>        REQUIRED
  --name <NAME>
  --start-date <DATE>  YYYY-MM-DD
  --end-date <DATE>    YYYY-MM-DD
  --json

lin cycles add <ISSUE_ID>
  --team <TEAM>        Team key or name (uses active cycle)
  --cycle <ID>         Specific cycle ID (overrides --team active cycle lookup)
  --json

lin cycles remove <ISSUE_ID>
  --json

lin cycles archive <CYCLE_ID>
```

---

## Teams

```
lin teams list --json
lin teams get <KEY> --json
lin teams members <KEY> --json
lin teams states <KEY> --json        # list workflow state names
lin teams workload <KEY> --json      # distribution by assignee
```

---

## Labels

```
lin labels list --json
lin labels create --name <NAME> --color <HEX> --json
lin labels update <NAME> --name <NEW> --color <HEX> --json
lin labels delete <NAME>
lin labels apply <ISSUE_ID> <LABEL_NAMES>    # comma-separated names
lin labels remove <ISSUE_ID> <LABEL_NAMES>
lin labels usage <NAME> --json               # issues with this label
```

---

## Relations

```
lin relations list <ISSUE_ID> --json
lin relations blocks <A> <B>        # A blocks B
lin relations blocked-by <A> <B>    # A is blocked by B
lin relations relates <A> <B>       # related
lin relations duplicate <A> <B>     # A duplicates B
lin relations remove <A> <B>
```

---

## Search (cross-entity)

```
lin search <QUERY>
  --limit <N>          Max per type [default: 10]
  --json
```

Searches issues, projects, and docs simultaneously.

---

## Notifications

```
lin notifications list
  --unread             Unread only
  --limit <N>          [default: 25]
  --json

lin notifications read <ID>     # single or "all"
lin notifications archive <ID>  # single or "all"
lin notifications snooze
lin notifications unsnooze
```

---

## Customers

```
lin customers list --limit <N> --json
lin customers create --name <NAME> --json
lin customers update <NAME> --json
lin customers delete <NAME>
lin customers link <ISSUE_ID> --customer <NAME> --json
lin customers needs <NAME_OR_ISSUE_ID> --json   # auto-detects issue vs customer
lin customers tiers --json
lin customers create-tier --name <NAME> --json
```

---

## Initiatives

```
lin initiatives list
  --status <Planned|Active|Completed>
  --limit <N>          [default: 20]
  --json

lin initiatives get <NAME> --json
lin initiatives create --name <NAME> --json
lin initiatives update <NAME> --json
lin initiatives archive <NAME>
lin initiatives delete <NAME>
lin initiatives projects <NAME> --json
lin initiatives updates <NAME> --json
lin initiatives post-update <NAME> --body <TEXT> --json
lin initiatives add-project <INITIATIVE> <PROJECT>
lin initiatives remove-project <INITIATIVE> <PROJECT>
```

---

## Roadmap

```
lin roadmap updates <PROJECT> --limit <N> --json
lin roadmap post <PROJECT> --body <TEXT> --status <STATUS> --json
lin roadmap milestones <PROJECT> --json
lin roadmap create-milestone <PROJECT> --name <NAME> --target-date <DATE> --json
lin roadmap update-milestone <PROJECT> <MILESTONE> --json
lin roadmap delete-milestone <PROJECT> <MILESTONE>
```

---

## Views

```
lin views list --json
lin views get <VIEW_ID> --json
lin views create --name <NAME> --json
lin views update <VIEW_ID> --json
lin views delete <VIEW_ID>
lin views issues <VIEW_ID> --limit <N> --json
```

---

## Docs

```
lin docs list
  --project <NAME>     Filter by project
  --limit <N>          [default: 50]
  --json

lin docs get <ID> --json       # includes full content
lin docs search <QUERY> --json
lin docs create --title <TITLE> --project <NAME> --json
lin docs update <ID> --json
lin docs delete <ID>
```

---

## Attachments

```
lin attachments list <ISSUE_ID> --limit <N> --json
lin attachments create <ISSUE_ID> --title <TITLE> --url <URL> --json
lin attachments link-url <ISSUE_ID> --url <URL> --json
lin attachments delete <ID>
```

---

## Auth (Workspaces)

```
lin auth login                       # interactive: add workspace name + API key
  --name <NAME>                      # workspace name (for scripting)
  --key <KEY>                        # API key (for scripting)
lin auth list              # list configured workspaces (* = default)
lin auth default <NAME>    # set default workspace
lin auth whoami            # show current user and workspace
```

---

## Other

```
lin me --json                        # authenticated user info
lin search <QUERY> --limit <N> --json
lin config set-key                   # set API key interactively
lin config show                      # show current config (no secrets)
lin completions <SHELL>              # generate shell completions
```

---

## API (Raw GraphQL)

```
lin api '<QUERY>'
  --variables <JSON>   JSON variables object (e.g. '{"teamId": "abc"}')
  (always outputs JSON)

Examples:
  lin api '{ viewer { id displayName } }'
  lin api '{ project(id: "uuid") { name teams { nodes { name } } } }'
  lin api 'mutation($id: String!) { issueArchive(id: $id) { success } }' --variables '{"id": "uuid"}'
```
