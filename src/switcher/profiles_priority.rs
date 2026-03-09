use super::*;

#[derive(Clone, Debug)]
pub(super) struct PriorityUsage {
    pub(super) seven_day_left: i64,
    pub(super) seven_day_reset: Option<String>,
    pub(super) five_hour_left: i64,
    pub(super) five_hour_reset: Option<String>,
    pub(super) tier: u8,
    pub(super) score: i64,
}

#[derive(Clone, Debug)]
pub(super) enum PriorityState {
    Ready(PriorityUsage),
    Unavailable(String),
}

#[derive(Clone, Debug)]
pub(super) struct PriorityRow {
    pub(super) id: String,
    pub(super) profile_name: String,
    pub(super) label: Option<String>,
    pub(super) is_current: bool,
    pub(super) candidate: bool,
    pub(super) state: PriorityState,
}

#[derive(Clone, Copy, Default)]
struct UsageSortKey {
    five_hour_left: Option<i64>,
    secondary_left: Option<i64>,
    reset_at: Option<i64>,
    usable: bool,
}

pub(super) fn ordered_profiles_by_usage(
    snapshot: &Snapshot,
    ctx: &ListCtx,
    current_saved_id: Option<&str>,
) -> Vec<(String, u64)> {
    let mut ordered = snapshot
        .usage_map
        .iter()
        .map(|(id, ts)| (id.clone(), *ts))
        .collect::<Vec<_>>();
    let usage_scores = usage_sort_scores(snapshot, ctx, current_saved_id);
    ordered.sort_by(|(left_id, left_ts), (right_id, right_ts)| {
        let left_score = usage_scores.get(left_id).copied().unwrap_or_default();
        let right_score = usage_scores.get(right_id).copied().unwrap_or_default();
        let left_has_primary = left_score.five_hour_left.is_some();
        let right_has_primary = right_score.five_hour_left.is_some();
        let mut ordering = right_has_primary.cmp(&left_has_primary);
        if ordering != Ordering::Equal {
            return ordering;
        }
        ordering = right_score.usable.cmp(&left_score.usable);
        if ordering != Ordering::Equal {
            return ordering;
        }
        if left_score.usable && right_score.usable {
            ordering = right_score
                .five_hour_left
                .unwrap_or(-1)
                .cmp(&left_score.five_hour_left.unwrap_or(-1));
            if ordering != Ordering::Equal {
                return ordering;
            }
            ordering = right_score
                .secondary_left
                .unwrap_or(-1)
                .cmp(&left_score.secondary_left.unwrap_or(-1));
            if ordering != Ordering::Equal {
                return ordering;
            }
        } else if !left_score.usable && !right_score.usable {
            let left_reset = left_score.reset_at.unwrap_or(i64::MAX);
            let right_reset = right_score.reset_at.unwrap_or(i64::MAX);
            ordering = left_reset.cmp(&right_reset);
            if ordering != Ordering::Equal {
                return ordering;
            }
        }
        ordering = right_ts.cmp(left_ts);
        if ordering != Ordering::Equal {
            return ordering;
        }
        left_id.cmp(right_id)
    });
    ordered
}

fn usage_sort_scores(
    snapshot: &Snapshot,
    ctx: &ListCtx,
    current_saved_id: Option<&str>,
) -> HashMap<String, UsageSortKey> {
    let Some(base_url) = ctx.base_url.as_deref() else {
        return HashMap::new();
    };
    let now = ctx.now;
    let ids: Vec<String> = snapshot.usage_map.keys().cloned().collect();
    let build = |id: &String| {
        if current_saved_id == Some(id.as_str()) {
            return (id.clone(), UsageSortKey::default());
        }
        let key = usage_sort_key_for_profile(id, snapshot, base_url, now).unwrap_or_default();
        (id.clone(), key)
    };
    let mut scores = HashMap::with_capacity(ids.len());
    if ids.len() > MAX_USAGE_CONCURRENCY {
        for chunk in ids.chunks(MAX_USAGE_CONCURRENCY) {
            let chunk_scores: Vec<(String, UsageSortKey)> = chunk.par_iter().map(build).collect();
            for (id, key) in chunk_scores {
                scores.insert(id, key);
            }
        }
        return scores;
    }
    let entries: Vec<(String, UsageSortKey)> = ids.par_iter().map(build).collect();
    for (id, key) in entries {
        scores.insert(id, key);
    }
    scores
}

fn usage_sort_key_for_profile(
    id: &str,
    snapshot: &Snapshot,
    base_url: &str,
    now: DateTime<Local>,
) -> Option<UsageSortKey> {
    if profile_is_api_key(id, snapshot) || profile_is_free(id, snapshot) {
        return None;
    }
    let tokens = snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())?;
    let access_token = tokens.access_token.as_deref()?;
    let account_id = token_account_id(tokens)?;
    let limits = fetch_usage_limits(base_url, access_token, account_id, now).ok()?;
    let five_hour_left = usage_left_percent(limits.five_hour.as_ref())?;
    let secondary_left = usage_left_percent(limits.weekly.as_ref());
    let primary_left = five_hour_left;
    let secondary_left_value = secondary_left.unwrap_or(0);
    let primary_reset = usage_reset_at(limits.five_hour.as_ref());
    let secondary_reset = usage_reset_at(limits.weekly.as_ref());
    let reset_at = if primary_left <= 0 && secondary_left_value <= 0 {
        match (primary_reset, secondary_reset) {
            (Some(primary), Some(secondary)) => Some(primary.max(secondary)),
            (Some(primary), None) => Some(primary),
            (None, Some(secondary)) => Some(secondary),
            (None, None) => None,
        }
    } else if primary_left <= 0 {
        primary_reset
    } else if secondary_left_value <= 0 {
        secondary_reset
    } else {
        None
    };
    let usable = primary_left > 0 && secondary_left_value > 0;
    Some(UsageSortKey {
        five_hour_left: Some(five_hour_left),
        secondary_left,
        reset_at,
        usable,
    })
}

fn usage_left_percent(window: Option<&UsageWindow>) -> Option<i64> {
    window.map(|value| value.left_percent.round() as i64)
}

fn usage_reset_at(window: Option<&UsageWindow>) -> Option<i64> {
    window.map(|value| value.reset_at)
}

fn usage_reset_remaining(window: Option<&UsageWindow>) -> Option<String> {
    window
        .and_then(|value| value.reset_at_relative.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn profile_is_api_key(id: &str, snapshot: &Snapshot) -> bool {
    snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
        .map(is_api_key_profile)
        .or_else(|| {
            snapshot
                .index
                .profiles
                .get(id)
                .map(|entry| entry.is_api_key)
        })
        .unwrap_or(false)
}

fn profile_is_free(id: &str, snapshot: &Snapshot) -> bool {
    let plan = profile_plan_for_sort(id, snapshot);
    is_free_plan(plan.as_deref())
}

fn profile_plan_for_sort(id: &str, snapshot: &Snapshot) -> Option<String> {
    if let Some(tokens) = snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
    {
        let (_, plan) = extract_email_and_plan(tokens);
        if plan.is_some() {
            return plan;
        }
    }
    snapshot
        .index
        .profiles
        .get(id)
        .and_then(|entry| entry.plan.clone())
}

fn priority_state_for_profile(
    paths: &Paths,
    id: &str,
    snapshot: &Snapshot,
    base_url: &str,
    now: DateTime<Local>,
) -> PriorityState {
    let Some(tokens) = snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
        .cloned()
    else {
        return PriorityState::Unavailable("Profile tokens are unreadable".to_string());
    };
    let profile_path = profile_path_for_id(&paths.profiles, id);
    priority_state_for_tokens(tokens, Some(profile_path.as_path()), base_url, now)
}

fn has_refresh_token(tokens: &Tokens) -> bool {
    tokens
        .refresh_token
        .as_deref()
        .map(|value| !value.is_empty())
        .unwrap_or(false)
}

fn should_refresh_before_usage(tokens: &Tokens) -> bool {
    !is_api_key_profile(tokens)
        && has_refresh_token(tokens)
        && (tokens.access_token.is_none() || token_account_id(tokens).is_none())
}

fn fetch_priority_limits(
    tokens: &mut Tokens,
    profile_path: Option<&Path>,
    base_url: &str,
    now: DateTime<Local>,
) -> Result<crate::switcher::usage::UsageLimits, String> {
    if should_refresh_before_usage(tokens)
        && let Some(profile_path) = profile_path
    {
        refresh_profile_tokens(profile_path, tokens)?;
    }
    let access_token = tokens
        .access_token
        .as_deref()
        .ok_or_else(|| "Missing access token".to_string())?;
    let account_id = token_account_id(tokens).ok_or_else(|| "Missing account id".to_string())?;
    match fetch_usage_limits(base_url, access_token, account_id, now) {
        Ok(limits) => Ok(limits),
        Err(err)
            if err.status_code() == Some(401)
                && has_refresh_token(tokens)
                && profile_path.is_some() =>
        {
            let profile_path = profile_path.expect("path checked above");
            refresh_profile_tokens(profile_path, tokens)?;
            let access_token = tokens
                .access_token
                .as_deref()
                .ok_or_else(|| "Missing access token".to_string())?;
            let account_id =
                token_account_id(tokens).ok_or_else(|| "Missing account id".to_string())?;
            fetch_usage_limits(base_url, access_token, account_id, now)
                .map_err(|retry_err| normalize_error(&retry_err.message()))
        }
        Err(err) => Err(normalize_error(&err.message())),
    }
}

fn priority_state_for_tokens(
    mut tokens: Tokens,
    profile_path: Option<&Path>,
    base_url: &str,
    now: DateTime<Local>,
) -> PriorityState {
    if is_api_key_profile(&tokens) {
        return PriorityState::Unavailable("Usage unavailable for API key login".to_string());
    }
    let (_, plan) = extract_email_and_plan(&tokens);
    if is_free_plan(plan.as_deref()) {
        return PriorityState::Unavailable("Usage unavailable for free plan".to_string());
    }
    let limits = match fetch_priority_limits(&mut tokens, profile_path, base_url, now) {
        Ok(limits) => limits,
        Err(err) => return PriorityState::Unavailable(err),
    };
    let Some(five_hour_left) = usage_left_percent(limits.five_hour.as_ref()) else {
        return PriorityState::Unavailable("Missing 5h usage window".to_string());
    };
    let Some(seven_day_left) = usage_left_percent(limits.weekly.as_ref()) else {
        return PriorityState::Unavailable("Missing 7d usage window".to_string());
    };
    let five_hour_reset = usage_reset_remaining(limits.five_hour.as_ref());
    let seven_day_reset = usage_reset_remaining(limits.weekly.as_ref());
    let tier = if seven_day_left <= 0 {
        2
    } else if five_hour_left <= 0 {
        1
    } else {
        0
    };
    let score = if tier == 0 {
        seven_day_left * SCORE_7D_WEIGHT + five_hour_left * SCORE_5H_WEIGHT
    } else {
        0
    };
    PriorityState::Ready(PriorityUsage {
        seven_day_left,
        seven_day_reset,
        five_hour_left,
        five_hour_reset,
        tier,
        score,
    })
}

fn priority_profile_email(id: &str, snapshot: &Snapshot) -> String {
    snapshot
        .tokens
        .get(id)
        .and_then(|result| result.as_ref().ok())
        .and_then(|tokens| extract_email_and_plan(tokens).0)
        .or_else(|| {
            snapshot
                .index
                .profiles
                .get(id)
                .and_then(|entry| entry.email.clone())
        })
        .unwrap_or_else(|| id.to_string())
}

fn profile_name_for_priority_row(id: &str, snapshot: &Snapshot, label: Option<&str>) -> String {
    let email = priority_profile_email(id, snapshot);
    let display = match label {
        Some(label) => format!("{email} [{label}]"),
        None => email,
    };
    with_reserved_marker(display, profile_is_reserved(id, snapshot))
}

fn priority_identity_key(row: &PriorityRow, snapshot: &Snapshot) -> String {
    if row.id == "__current__" {
        return "id:__current__".to_string();
    }
    if let Some(tokens) = snapshot
        .tokens
        .get(&row.id)
        .and_then(|result| result.as_ref().ok())
    {
        if let Some(account_id) = token_account_id(tokens) {
            return format!("account:{account_id}");
        }
        if let Some(email) = extract_email_and_plan(tokens).0 {
            return format!("email:{}", email.to_ascii_lowercase());
        }
    }
    if let Some(entry) = snapshot.index.profiles.get(&row.id) {
        if let Some(account_id) = entry.account_id.as_deref() {
            return format!("account:{account_id}");
        }
        if let Some(email) = entry.email.as_deref() {
            return format!("email:{}", email.to_ascii_lowercase());
        }
    }
    format!("id:{}", row.id)
}

fn priority_state_cmp(left: &PriorityState, right: &PriorityState) -> Ordering {
    match (left, right) {
        (PriorityState::Ready(left_usage), PriorityState::Ready(right_usage)) => left_usage
            .tier
            .cmp(&right_usage.tier)
            .then_with(|| right_usage.score.cmp(&left_usage.score)),
        (PriorityState::Ready(_), PriorityState::Unavailable(_)) => Ordering::Less,
        (PriorityState::Unavailable(_), PriorityState::Ready(_)) => Ordering::Greater,
        (PriorityState::Unavailable(_), PriorityState::Unavailable(_)) => Ordering::Equal,
    }
}

fn priority_row_has_readable_tokens(row: &PriorityRow, snapshot: &Snapshot) -> bool {
    if row.id == "__current__" {
        return true;
    }
    snapshot
        .tokens
        .get(&row.id)
        .is_some_and(|result| result.is_ok())
}

fn priority_group_target_cmp(
    left: &PriorityRow,
    right: &PriorityRow,
    snapshot: &Snapshot,
) -> Ordering {
    let ordering = right.is_current.cmp(&left.is_current);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let ordering = right.label.is_some().cmp(&left.label.is_some());
    if ordering != Ordering::Equal {
        return ordering;
    }
    let ordering = right.candidate.cmp(&left.candidate);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let left_readable = priority_row_has_readable_tokens(left, snapshot);
    let right_readable = priority_row_has_readable_tokens(right, snapshot);
    let ordering = right_readable.cmp(&left_readable);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let ordering = priority_state_cmp(&left.state, &right.state);
    if ordering != Ordering::Equal {
        return ordering;
    }
    let left_last_used = snapshot
        .usage_map
        .get(&left.id)
        .copied()
        .unwrap_or_default();
    let right_last_used = snapshot
        .usage_map
        .get(&right.id)
        .copied()
        .unwrap_or_default();
    right_last_used
        .cmp(&left_last_used)
        .then_with(|| left.id.cmp(&right.id))
}

fn priority_display_label(rows: &[PriorityRow], target: &PriorityRow) -> Option<String> {
    target
        .label
        .clone()
        .or_else(|| {
            rows.iter()
                .find(|row| row.is_current)
                .and_then(|row| row.label.clone())
        })
        .or_else(|| {
            let mut labels = rows
                .iter()
                .filter_map(|row| row.label.clone())
                .collect::<Vec<_>>();
            labels.sort_by_key(|label| label.to_ascii_lowercase());
            labels.into_iter().next()
        })
}

fn merge_priority_rows(rows: Vec<PriorityRow>, snapshot: &Snapshot) -> PriorityRow {
    let target = rows
        .iter()
        .min_by(|left, right| priority_group_target_cmp(left, right, snapshot))
        .expect("priority group is not empty");
    let state = rows
        .iter()
        .min_by(|left, right| {
            priority_state_cmp(&left.state, &right.state)
                .then_with(|| priority_group_target_cmp(left, right, snapshot))
        })
        .map(|row| row.state.clone())
        .expect("priority group is not empty");
    let label = priority_display_label(&rows, target);
    let profile_name = if target.id == "__current__" {
        target.profile_name.clone()
    } else {
        profile_name_for_priority_row(&target.id, snapshot, label.as_deref())
    };
    PriorityRow {
        id: target.id.clone(),
        profile_name,
        label,
        is_current: rows.iter().any(|row| row.is_current),
        candidate: rows.iter().any(|row| row.candidate),
        state,
    }
}

fn dedupe_priority_rows(rows: Vec<PriorityRow>, snapshot: &Snapshot) -> Vec<PriorityRow> {
    let mut grouped: HashMap<String, Vec<PriorityRow>> = HashMap::new();
    for row in rows {
        grouped
            .entry(priority_identity_key(&row, snapshot))
            .or_default()
            .push(row);
    }
    let mut merged = grouped
        .into_values()
        .map(|group| merge_priority_rows(group, snapshot))
        .collect::<Vec<_>>();
    merged.sort_by(priority_row_cmp);
    merged
}

pub(super) fn priority_rows(
    paths: &Paths,
    snapshot: &Snapshot,
    current_saved_id: Option<&str>,
    include_unsaved_current: bool,
) -> Vec<PriorityRow> {
    let base_url = read_base_url(paths);
    let now = Local::now();
    let ids: Vec<String> = snapshot.usage_map.keys().cloned().collect();
    let build = |id: &String| {
        let label = label_for_id(&snapshot.labels, id);
        let profile_name = profile_name_for_priority_row(id, snapshot, label.as_deref());
        PriorityRow {
            id: id.clone(),
            profile_name,
            label,
            is_current: current_saved_id == Some(id.as_str()),
            candidate: !profile_is_reserved(id, snapshot),
            state: priority_state_for_profile(paths, id, snapshot, &base_url, now),
        }
    };
    let mut rows = if ids.len() > MAX_USAGE_CONCURRENCY {
        let mut out = Vec::with_capacity(ids.len());
        for chunk in ids.chunks(MAX_USAGE_CONCURRENCY) {
            let mut chunk_rows: Vec<PriorityRow> = chunk.par_iter().map(build).collect();
            out.append(&mut chunk_rows);
        }
        out
    } else {
        ids.par_iter().map(build).collect::<Vec<_>>()
    };
    if include_unsaved_current
        && current_saved_id.is_none()
        && let Ok(tokens) = read_tokens(&paths.auth)
    {
        let (email, _) = extract_email_and_plan(&tokens);
        let profile_name = email.unwrap_or_else(|| "Current profile".to_string());
        rows.push(PriorityRow {
            id: "__current__".to_string(),
            profile_name,
            label: None,
            is_current: true,
            candidate: false,
            state: priority_state_for_tokens(tokens, Some(paths.auth.as_path()), &base_url, now),
        });
    }
    dedupe_priority_rows(rows, snapshot)
}

fn priority_sort_label(row: &PriorityRow) -> String {
    row.label.as_deref().unwrap_or(&row.id).to_ascii_lowercase()
}

pub(super) fn priority_row_cmp(left: &PriorityRow, right: &PriorityRow) -> Ordering {
    match (&left.state, &right.state) {
        (PriorityState::Ready(left_usage), PriorityState::Ready(right_usage)) => left_usage
            .tier
            .cmp(&right_usage.tier)
            .then_with(|| right_usage.score.cmp(&left_usage.score))
            .then_with(|| priority_sort_label(left).cmp(&priority_sort_label(right)))
            .then_with(|| left.id.cmp(&right.id)),
        (PriorityState::Ready(_), PriorityState::Unavailable(_)) => Ordering::Less,
        (PriorityState::Unavailable(_), PriorityState::Ready(_)) => Ordering::Greater,
        (PriorityState::Unavailable(_), PriorityState::Unavailable(_)) => priority_sort_label(left)
            .cmp(&priority_sort_label(right))
            .then_with(|| left.id.cmp(&right.id)),
    }
}

pub(super) fn best_ready_row(rows: &[PriorityRow]) -> Option<&PriorityRow> {
    rows.iter()
        .find(|row| row.candidate && matches!(row.state, PriorityState::Ready(_)))
}

pub(super) fn render_priority_table(rows: &[PriorityRow], use_color: bool) -> String {
    let headers = [
        "#", "CUR", "PROFILE", "7D", "7D RESET", "5H", "5H RESET", "TIER", "SCORE", "STATE",
    ];
    let mut data = Vec::with_capacity(rows.len());
    let mut rank = 0usize;
    for row in rows {
        let mut rank_text = "-".to_string();
        let current = if row.is_current { "*" } else { "" }.to_string();
        let mut seven = "--".to_string();
        let mut seven_reset = "--".to_string();
        let mut five = "--".to_string();
        let mut five_reset = "--".to_string();
        let mut tier = "--".to_string();
        let mut score = "--".to_string();
        let state = match &row.state {
            PriorityState::Ready(usage) => {
                if row.candidate {
                    rank += 1;
                    rank_text = rank.to_string();
                }
                seven = format!("{}%", usage.seven_day_left);
                seven_reset = usage.seven_day_reset.as_deref().unwrap_or("--").to_string();
                five = format!("{}%", usage.five_hour_left);
                five_reset = usage.five_hour_reset.as_deref().unwrap_or("--").to_string();
                tier = format!("T{}", usage.tier);
                score = if usage.tier == 0 {
                    format!("{:.1}", usage.score as f64 / 100.0)
                } else {
                    "MIN".to_string()
                };
                if row.is_current && !row.candidate {
                    "CURRENT".to_string()
                } else {
                    match usage.tier {
                        0 => "READY".to_string(),
                        1 => "5H=0".to_string(),
                        _ => "7D=0".to_string(),
                    }
                }
            }
            PriorityState::Unavailable(_) => "UNAVAILABLE".to_string(),
        };
        data.push(vec![
            rank_text,
            current,
            row.profile_name.clone(),
            seven,
            seven_reset,
            five,
            five_reset,
            tier,
            score,
            state,
        ]);
    }

    let mut widths: Vec<usize> = headers.iter().map(|value| value.len()).collect();
    for row in &data {
        for (idx, value) in row.iter().enumerate() {
            widths[idx] = widths[idx].max(value.len());
        }
    }

    let mut lines = Vec::new();
    lines.push("Priority ranking (best first)".to_string());
    lines.push(format_row(&headers, &widths, None, use_color));
    let separator = widths.iter().map(|w| "-".repeat(*w)).collect::<Vec<_>>();
    lines.push(format_row_ref(&separator, &widths, None, use_color));

    for row in data {
        let state_style = row.last().cloned().unwrap_or_default();
        lines.push(format_row_ref(&row, &widths, Some(&state_style), use_color));
    }

    let unavailable_rows: Vec<String> = rows
        .iter()
        .filter_map(|row| match &row.state {
            PriorityState::Unavailable(reason) => Some(format!("{}: {}", row.profile_name, reason)),
            PriorityState::Ready(_) => None,
        })
        .collect();
    if !unavailable_rows.is_empty() {
        lines.push(String::new());
        lines.push("Unavailable profiles".to_string());
        for line in unavailable_rows {
            lines.push(format!("- {line}"));
        }
    }

    lines.join("\n")
}

fn format_row(row: &[&str], widths: &[usize], state: Option<&str>, use_color: bool) -> String {
    let parts = row
        .iter()
        .enumerate()
        .map(|(idx, value)| format!("{value:<width$}", width = widths[idx]))
        .collect::<Vec<_>>();
    format_row_ref(&parts, widths, state, use_color)
}

fn format_row_ref(
    row: &[String],
    widths: &[usize],
    state: Option<&str>,
    use_color: bool,
) -> String {
    let mut parts = Vec::with_capacity(row.len());
    for (idx, value) in row.iter().enumerate() {
        let padded = format!("{value:<width$}", width = widths[idx]);
        let styled = if idx + 1 == row.len() {
            style_priority_state_cell(&padded, state, use_color)
        } else if idx == 1 && value == "*" {
            style_text(&padded, use_color, |text| text.cyan().bold())
        } else {
            padded
        };
        parts.push(styled);
    }
    parts.join("  ")
}

fn style_priority_state_cell(cell: &str, state: Option<&str>, use_color: bool) -> String {
    let Some(state) = state else {
        return cell.to_string();
    };
    match state {
        "READY" => style_text(cell, use_color, |text| text.green().bold()),
        "CURRENT" => style_text(cell, use_color, |text| text.cyan().bold()),
        "5H=0" => style_text(cell, use_color, |text| text.yellow().bold()),
        "7D=0" => style_text(cell, use_color, |text| text.red().bold()),
        "UNAVAILABLE" => style_text(cell, use_color, |text| text.dimmed()),
        _ => cell.to_string(),
    }
}
