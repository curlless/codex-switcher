use super::*;

pub(super) fn make_candidates(
    paths: &Paths,
    snapshot: &Snapshot,
    ordered: &[(String, u64)],
) -> Vec<Candidate> {
    let current_saved = current_saved_id(paths, &snapshot.usage_map, &snapshot.tokens);
    build_candidates(ordered, snapshot, current_saved.as_deref())
}

pub(super) fn pick_one(
    action: &str,
    label: Option<&str>,
    snapshot: &Snapshot,
    candidates: &[Candidate],
) -> Result<Candidate, String> {
    if let Some(label) = label {
        select_by_label(label, &snapshot.labels, candidates)
    } else {
        require_tty(action)?;
        select_single_profile("", candidates)
    }
}

pub(super) fn pick_many(
    action: &str,
    label: Option<&str>,
    snapshot: &Snapshot,
    candidates: &[Candidate],
) -> Result<Vec<Candidate>, String> {
    if let Some(label) = label {
        Ok(vec![select_by_label(label, &snapshot.labels, candidates)?])
    } else {
        require_tty(action)?;
        select_multiple_profiles("", candidates)
    }
}

pub(crate) struct ProfileInfo {
    pub(crate) display: String,
    pub(crate) email: Option<String>,
    pub(crate) plan: Option<String>,
    pub(crate) is_free: bool,
}

pub(crate) fn profile_info(
    tokens: Option<&Tokens>,
    label: Option<String>,
    is_current: bool,
    use_color: bool,
) -> ProfileInfo {
    profile_info_with_fallback(tokens, None, label, is_current, use_color)
}

fn profile_info_with_fallback(
    tokens: Option<&Tokens>,
    fallback: Option<&ProfileIndexEntry>,
    label: Option<String>,
    is_current: bool,
    use_color: bool,
) -> ProfileInfo {
    let (email, plan) = if let Some(tokens) = tokens {
        extract_email_and_plan(tokens)
    } else if let Some(entry) = fallback {
        (entry.email.clone(), entry.plan.clone())
    } else {
        (None, None)
    };
    let is_free = is_free_plan(plan.as_deref());
    let display = crate::switcher::format_profile_display(
        email.clone(),
        plan.clone(),
        label,
        is_current,
        use_color,
    );
    let display = with_reserved_marker(display, fallback.is_some_and(|entry| entry.reserved));
    ProfileInfo {
        display,
        email,
        plan,
        is_free,
    }
}

#[derive(Debug)]
pub(super) enum LoadChoice {
    SaveAndContinue,
    ContinueWithoutSaving,
    Cancel,
}

pub(crate) fn prompt_unsaved_load(paths: &Paths, reason: &str) -> Result<LoadChoice, String> {
    let is_tty = io::stdin().is_terminal();
    if !is_tty {
        let hint = format_save_before_load(paths, use_color_stderr());
        return Err(format!("Error: current profile is not saved. {hint}"));
    }
    let selection = Select::new(
        "",
        vec![
            "Save current profile and continue",
            "Continue without saving",
            "Cancel",
        ],
    )
    .with_render_config(inquire_select_render_config())
    .prompt();
    prompt_unsaved_load_with(paths, reason, is_tty, selection)
}

pub(super) fn prompt_unsaved_load_with(
    paths: &Paths,
    reason: &str,
    is_tty: bool,
    selection: Result<&str, inquire::error::InquireError>,
) -> Result<LoadChoice, String> {
    if !is_tty {
        let hint = format_save_before_load(paths, use_color_stderr());
        return Err(format!("Error: current profile is not saved. {hint}"));
    }
    let warning = format_warning(
        &format!("Current profile is not saved ({reason})."),
        use_color_stderr(),
    );
    eprintln!("{warning}");
    match selection {
        Ok("Save current profile and continue") => Ok(LoadChoice::SaveAndContinue),
        Ok("Continue without saving") => Ok(LoadChoice::ContinueWithoutSaving),
        Ok(_) => Ok(LoadChoice::Cancel),
        Err(err) if is_inquire_cancel(&err) => Ok(LoadChoice::Cancel),
        Err(err) => Err(format!("Error: failed to prompt for load: {err}")),
    }
}

pub(crate) fn build_candidates(
    ordered: &[(String, u64)],
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
) -> Vec<Candidate> {
    let mut candidates = Vec::with_capacity(ordered.len());
    let use_color = use_color_stderr();
    for (id, ts) in ordered {
        let label = label_for_id(&snapshot.labels, id);
        let tokens = snapshot
            .tokens
            .get(id)
            .and_then(|result| result.as_ref().ok());
        let index_entry = snapshot.index.profiles.get(id);
        let is_current = current_saved_id == Some(id.as_str());
        let info = profile_info_with_fallback(tokens, index_entry, label, is_current, use_color);
        let last_used = if is_current {
            String::new()
        } else {
            format_last_used(*ts)
        };
        candidates.push(Candidate {
            id: id.clone(),
            display: info.display,
            last_used,
            is_current,
        });
    }
    candidates
}

pub(crate) fn require_tty(action: &str) -> Result<(), String> {
    require_tty_with(io::stdin().is_terminal(), action)
}

pub(super) fn require_tty_with(is_tty: bool, action: &str) -> Result<(), String> {
    if is_tty {
        Ok(())
    } else {
        Err(format!(
            "Error: {action} selection requires a TTY. Run `{} {action}` interactively.",
            command_name()
        ))
    }
}

pub(crate) fn select_single_profile(
    title: &str,
    candidates: &[Candidate],
) -> Result<Candidate, String> {
    let options = candidates.to_vec();
    let render_config = inquire_select_render_config();
    let prompt = Select::new(title, options)
        .with_help_message(LOAD_HELP)
        .with_render_config(render_config)
        .prompt();
    handle_inquire_result(prompt, "selection")
}

pub(crate) fn select_multiple_profiles(
    title: &str,
    candidates: &[Candidate],
) -> Result<Vec<Candidate>, String> {
    let options = candidates.to_vec();
    let render_config = inquire_select_render_config();
    let prompt = MultiSelect::new(title, options)
        .with_help_message(DELETE_HELP)
        .with_render_config(render_config)
        .prompt();
    let selections = handle_inquire_result(prompt, "selection")?;
    if selections.is_empty() {
        return Err(CANCELLED_MESSAGE.to_string());
    }
    Ok(selections)
}

pub(crate) fn select_by_label(
    label: &str,
    labels: &Labels,
    candidates: &[Candidate],
) -> Result<Candidate, String> {
    let id = resolve_label_id(labels, label)?;
    let Some(candidate) = candidates.iter().find(|candidate| candidate.id == id) else {
        return Err(format!(
            "Error: label '{label}' does not match a saved profile. {}",
            format_list_hint(use_color_stderr())
        ));
    };
    Ok(candidate.clone())
}

pub(crate) fn confirm_delete_profiles(displays: &[String]) -> Result<bool, String> {
    let is_tty = io::stdin().is_terminal();
    if !is_tty {
        return Err(
            "Error: deletion requires confirmation. Re-run with `--yes` to skip the prompt."
                .to_string(),
        );
    }
    let prompt = if displays.len() == 1 {
        format!("Delete profile {}? This cannot be undone.", displays[0])
    } else {
        let count = displays.len();
        eprintln!("Delete {count} profiles? This cannot be undone.");
        for display in displays {
            eprintln!(" - {display}");
        }
        "Delete selected profiles? This cannot be undone.".to_string()
    };
    let selection = Confirm::new(&prompt)
        .with_default(false)
        .with_render_config(inquire_select_render_config())
        .prompt();
    confirm_delete_profiles_with(is_tty, selection)
}

pub(super) fn confirm_delete_profiles_with(
    is_tty: bool,
    selection: Result<bool, inquire::error::InquireError>,
) -> Result<bool, String> {
    if !is_tty {
        return Err(
            "Error: deletion requires confirmation. Re-run with `--yes` to skip the prompt."
                .to_string(),
        );
    }
    match selection {
        Ok(value) => Ok(value),
        Err(err) if is_inquire_cancel(&err) => Err(CANCELLED_MESSAGE.to_string()),
        Err(err) => Err(format!("Error: failed to prompt for delete: {err}")),
    }
}

#[derive(Clone)]
pub(crate) struct Candidate {
    pub(crate) id: String,
    pub(crate) display: String,
    pub(crate) last_used: String,
    pub(crate) is_current: bool,
}

impl fmt::Display for Candidate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header = format_entry_header(
            &self.display,
            &self.last_used,
            self.is_current,
            use_color_stderr(),
        );
        write!(f, "{header}")
    }
}

pub(super) fn render_entries(
    entries: &[Entry],
    show_last_used: bool,
    ctx: &ListCtx,
    separator: Option<&str>,
    allow_plain_spacing: bool,
) -> Vec<String> {
    let mut lines = Vec::with_capacity((entries.len().max(1)) * 4);
    for (idx, entry) in entries.iter().enumerate() {
        let header = format_entry_header(
            &entry.display,
            if show_last_used { &entry.last_used } else { "" },
            entry.is_current,
            ctx.use_color,
        );
        let show_detail_lines = ctx.show_usage || entry.always_show_details;
        if !show_detail_lines {
            if let Some(err) = entry.error_summary.as_deref() {
                let mut header = header;
                header.push_str(&format!("  {err}"));
                lines.push(header);
            } else {
                lines.push(header);
            }
        } else {
            lines.push(header);
            lines.extend(entry.details.iter().cloned());
        }
        if idx + 1 < entries.len() {
            push_separator(&mut lines, separator, allow_plain_spacing);
        }
    }
    lines
}

pub(super) fn push_separator(
    lines: &mut Vec<String>,
    separator: Option<&str>,
    allow_plain_spacing: bool,
) {
    match separator {
        Some(value) => lines.push(value.to_string()),
        None => {
            if !is_plain() || allow_plain_spacing {
                lines.push(String::new());
            }
        }
    }
}

pub(super) fn separator_line(trim: usize) -> Option<String> {
    if is_plain() {
        return None;
    }
    let width = terminal_width()?;
    let len = width.saturating_sub(trim);
    if len == 0 {
        return None;
    }
    let line = "-".repeat(len);
    Some(style_text(&line, use_color_stdout(), |text| text.dimmed()))
}

fn make_error(
    label: Option<String>,
    index_entry: Option<&ProfileIndexEntry>,
    use_color: bool,
    last_used: String,
    message: &str,
    summary_label: &str,
    is_current: bool,
) -> Entry {
    let display =
        profile_info_with_fallback(None, index_entry, label, is_current, use_color).display;
    Entry {
        display,
        last_used,
        details: vec![format_error(message)],
        error_summary: Some(error_summary(summary_label, message)),
        always_show_details: false,
        is_current,
    }
}

fn unavailable_lines(message: &str, use_color: bool) -> Vec<String> {
    vec![format_usage_unavailable(message, use_color)]
}

fn detail_lines(
    tokens: &mut Tokens,
    email: Option<&str>,
    plan: Option<&str>,
    show_spinner: bool,
    profile_path: &Path,
    ctx: &ListCtx,
    allow_401_refresh: bool,
) -> (Vec<String>, Option<String>) {
    let plan_is_free = is_free_plan(plan);
    let use_color = ctx.use_color;
    let account_id = token_account_id(tokens).map(str::to_string);
    let access_token = tokens.access_token.clone();
    if is_api_key_profile(tokens) {
        if ctx.show_usage {
            return (
                unavailable_lines("Usage unavailable for API key login", use_color),
                None,
            );
        }
        return (Vec::new(), None);
    }
    let unavailable_text = usage_unavailable(plan_is_free);
    if let Some(message) = profile_error(tokens, email, plan) {
        let missing_access = access_token.is_none() || account_id.is_none();
        if ctx.show_usage && missing_access && email.is_some() && plan.is_some() {
            return (unavailable_lines(unavailable_text, use_color), None);
        }
        let details = vec![format_error(message)];
        let summary = Some(error_summary("Error", message));
        return (details, summary);
    }
    if ctx.show_usage {
        let Some(base_url) = ctx.base_url.as_deref() else {
            return (Vec::new(), None);
        };
        let Some(access_token) = access_token.as_deref() else {
            return (Vec::new(), None);
        };
        let Some(account_id) = account_id.as_deref() else {
            return (Vec::new(), None);
        };
        match fetch_usage_details(
            base_url,
            access_token,
            account_id,
            unavailable_text,
            ctx.now,
            show_spinner,
        ) {
            Ok(details) => (details, None),
            Err(err) if allow_401_refresh && err.status_code() == Some(401) => {
                match refresh_profile_tokens(profile_path, tokens) {
                    Ok(()) => {
                        let Some(access_token) = tokens.access_token.as_deref() else {
                            let message = "Error: refreshed access_token is missing.";
                            return (
                                vec![format_error(message)],
                                Some(error_summary("Auth error", message)),
                            );
                        };
                        match fetch_usage_details(
                            base_url,
                            access_token,
                            account_id,
                            unavailable_text,
                            ctx.now,
                            show_spinner,
                        ) {
                            Ok(details) => (details, None),
                            Err(err) => (
                                vec![format_error(&err.message())],
                                Some(error_summary("Usage error", &err.message())),
                            ),
                        }
                    }
                    Err(err) => (
                        vec![format_error(&err)],
                        Some(error_summary("Auth error", &err)),
                    ),
                }
            }
            Err(err) => (
                vec![format_error(&err.message())],
                Some(error_summary("Usage error", &err.message())),
            ),
        }
    } else if plan_is_free {
        (unavailable_lines(unavailable_text, use_color), None)
    } else {
        (Vec::new(), None)
    }
}

enum RefreshAttempt {
    Skipped,
    Succeeded,
    Failed(String),
}

fn refresh_for_status(tokens: &mut Tokens, profile_path: &Path, ctx: &ListCtx) -> RefreshAttempt {
    if !ctx.show_usage {
        return RefreshAttempt::Skipped;
    }
    if is_api_key_profile(tokens) {
        return RefreshAttempt::Skipped;
    }
    let has_refresh = tokens
        .refresh_token
        .as_deref()
        .map(|value| !value.is_empty())
        .unwrap_or(false);
    if !has_refresh {
        return RefreshAttempt::Failed(
            "Error: profile is missing refresh_token; run `codex login` and save it again."
                .to_string(),
        );
    }
    match refresh_profile_tokens(profile_path, tokens) {
        Ok(()) => RefreshAttempt::Succeeded,
        Err(err) => RefreshAttempt::Failed(err),
    }
}

pub(super) fn make_entry(
    last_used: String,
    label: Option<String>,
    tokens_result: Option<&Result<Tokens, String>>,
    index_entry: Option<&ProfileIndexEntry>,
    profile_path: &Path,
    ctx: &ListCtx,
    is_current: bool,
) -> Entry {
    let use_color = ctx.use_color;
    let label_for_error = label.clone().or_else(|| profile_id_from_path(profile_path));
    let mut tokens = match tokens_result {
        Some(Ok(tokens)) => tokens.clone(),
        Some(Err(err)) => {
            return make_error(
                label_for_error,
                index_entry,
                use_color,
                last_used,
                err,
                "Error",
                is_current,
            );
        }
        None => {
            return make_error(
                label_for_error,
                index_entry,
                use_color,
                last_used,
                "profile file missing",
                "Error",
                is_current,
            );
        }
    };
    let refresh_attempt = refresh_for_status(&mut tokens, profile_path, ctx);
    let info = profile_info(Some(&tokens), label, is_current, use_color);
    let allow_401_refresh = matches!(refresh_attempt, RefreshAttempt::Skipped);
    let (mut details, mut summary) = detail_lines(
        &mut tokens,
        info.email.as_deref(),
        info.plan.as_deref(),
        false,
        profile_path,
        ctx,
        allow_401_refresh,
    );
    if let RefreshAttempt::Failed(err) = refresh_attempt {
        let warning = format_warning(&normalize_error(&err), use_color);
        details.insert(0, warning);
        if summary.is_none() {
            summary = Some(error_summary("Auth refresh", &err));
        }
    }
    Entry {
        display: info.display,
        last_used,
        details,
        error_summary: summary,
        always_show_details: info.is_free,
        is_current,
    }
}

fn make_saved(
    id: &str,
    ts: u64,
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
    ctx: &ListCtx,
) -> Entry {
    let profile_path = ctx.profiles_dir.join(format!("{id}.json"));
    let label = label_for_id(&snapshot.labels, id);
    let is_current = current_saved_id == Some(id);
    let last_used = if is_current {
        String::new()
    } else {
        format_last_used(ts)
    };
    make_entry(
        last_used,
        label,
        snapshot.tokens.get(id),
        snapshot.index.profiles.get(id),
        &profile_path,
        ctx,
        is_current,
    )
}

pub(super) fn make_entries(
    ordered: &[(String, u64)],
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
    ctx: &ListCtx,
) -> Vec<Entry> {
    let build = |(id, ts): &(String, u64)| make_saved(id, *ts, snapshot, current_saved_id, ctx);
    if ctx.show_usage && ordered.len() >= 3 {
        if ordered.len() > MAX_USAGE_CONCURRENCY {
            let mut entries = Vec::with_capacity(ordered.len());
            for chunk in ordered.chunks(MAX_USAGE_CONCURRENCY) {
                let mut chunk_entries: Vec<Entry> = chunk.par_iter().map(build).collect();
                entries.append(&mut chunk_entries);
            }
            return entries;
        }
        return ordered.par_iter().map(build).collect();
    }

    ordered.iter().map(build).collect()
}

pub(super) fn make_current(
    paths: &Paths,
    current_saved_id: Option<&str>,
    labels: &Labels,
    tokens_map: &BTreeMap<String, Result<Tokens, String>>,
    usage_map: &BTreeMap<String, u64>,
    ctx: &ListCtx,
) -> Option<Entry> {
    if !paths.auth.is_file() {
        return None;
    }
    let mut tokens = match read_tokens(&paths.auth) {
        Ok(tokens) => tokens,
        Err(err) => {
            return Some(make_error(
                None,
                None,
                ctx.use_color,
                String::new(),
                &err,
                "Error",
                true,
            ));
        }
    };
    let refresh_attempt = refresh_for_status(&mut tokens, &ctx.auth_path, ctx);
    let (email, _) = extract_email_and_plan(&tokens);
    let refreshed_saved_id =
        if matches!(refresh_attempt, RefreshAttempt::Succeeded) || current_saved_id.is_none() {
            match (token_account_id(&tokens), email.as_deref()) {
                (Some(account_id), Some(email)) => {
                    let candidates = cached_profile_ids(tokens_map, account_id, Some(email));
                    pick_primary(&candidates, usage_map)
                }
                _ => None,
            }
        } else {
            None
        };
    let effective_saved_id = refreshed_saved_id.as_deref().or(current_saved_id);
    if matches!(refresh_attempt, RefreshAttempt::Succeeded)
        && let Some(id) = effective_saved_id
    {
        let profile_path = ctx.profiles_dir.join(format!("{id}.json"));
        if profile_path.is_file()
            && let Err(err) = copy_atomic(&ctx.auth_path, &profile_path)
        {
            let warning = format_warning(&normalize_error(&err), use_color_stderr());
            eprintln!("{warning}");
        }
    }
    let label = effective_saved_id.and_then(|id| label_for_id(labels, id));
    let use_color = ctx.use_color;
    let info = profile_info(Some(&tokens), label, true, use_color);
    let plan_is_free = info.is_free;
    let can_save = is_profile_ready(&tokens);
    let is_unsaved = effective_saved_id.is_none() && can_save;
    let allow_401_refresh = matches!(refresh_attempt, RefreshAttempt::Skipped);
    let (mut details, mut summary) = detail_lines(
        &mut tokens,
        info.email.as_deref(),
        info.plan.as_deref(),
        ctx.show_spinner,
        &ctx.auth_path,
        ctx,
        allow_401_refresh,
    );
    if let RefreshAttempt::Failed(err) = refresh_attempt {
        let warning = format_warning(&normalize_error(&err), use_color);
        details.insert(0, warning);
        if summary.is_none() {
            summary = Some(error_summary("Auth refresh", &err));
        }
    }

    if is_unsaved && !plan_is_free {
        details.extend(format_unsaved_warning(use_color));
    }

    Some(Entry {
        display: info.display,
        last_used: String::new(),
        details,
        error_summary: summary,
        always_show_details: is_unsaved || (plan_is_free && !ctx.show_usage),
        is_current: true,
    })
}

fn error_summary(label: &str, message: &str) -> String {
    format!("{label}: {}", normalize_error(message))
}

pub(super) struct ListCtx {
    pub(super) base_url: Option<String>,
    pub(super) now: DateTime<Local>,
    pub(super) show_usage: bool,
    pub(super) show_spinner: bool,
    pub(super) use_color: bool,
    pub(super) profiles_dir: PathBuf,
    pub(super) auth_path: PathBuf,
}

impl ListCtx {
    pub(super) fn new(paths: &Paths, show_usage: bool) -> Self {
        Self {
            base_url: show_usage.then(|| read_base_url(paths)),
            now: Local::now(),
            show_usage,
            show_spinner: show_usage,
            use_color: use_color_stdout(),
            profiles_dir: paths.profiles.clone(),
            auth_path: paths.auth.clone(),
        }
    }
}

pub(super) struct Entry {
    pub(super) display: String,
    pub(super) last_used: String,
    pub(super) details: Vec<String>,
    pub(super) error_summary: Option<String>,
    pub(super) always_show_details: bool,
    pub(super) is_current: bool,
}

const LOAD_HELP: &str = "Type to search • Use ↑/↓ to select • ENTER to load";
const DELETE_HELP: &str = "Type to search • Use ↑/↓ to select • SPACE to select • ENTER to delete";

pub(super) fn handle_inquire_result<T>(
    result: Result<T, inquire::error::InquireError>,
    context: &str,
) -> Result<T, String> {
    match result {
        Ok(value) => Ok(value),
        Err(err) if is_inquire_cancel(&err) => Err(CANCELLED_MESSAGE.to_string()),
        Err(err) => Err(format!("Error: failed to prompt for {context}: {err}")),
    }
}

