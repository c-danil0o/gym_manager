use crate::{error::Result as AppResult, state::AppState};
use chrono::NaiveDate;
use tauri::State;

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct MembershipTypeDistributionItem {
    membership_type_name: String,
    active_member_count: i64,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct ActiveMembershipsOverTimeItem {
    year_month: String, // e.g., "2023-01"
    active_member_count: i64,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct HourlyVisitCount {
    hour_of_day: i64, // Or u32
    visit_count: i64,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct DailyHourlyVisitCount {
    day_of_week: i64, // 0-6
    hour_of_day: i64, // 0-23
    visit_count: i64,
}

#[derive(sqlx::FromRow, serde::Serialize)]
pub struct RevenueByMembershipTypeItem {
    membership_type_name: String,
    total_revenue: f64,
    count: i64, // Number of memberships of this type
}

const MEMBERSHIP_TYPE_DISTRIBUTION_QUERY: &str = r#"
SELECT
    mt.name AS membership_type_name,
    COUNT(DISTINCT m.id) AS active_member_count
FROM
    membership_types mt
JOIN
    memberships ms ON mt.id = ms.membership_type_id
JOIN
    members m ON ms.member_id = m.id
WHERE
    ms.status = 'active'
    AND (ms.is_deleted IS NULL OR ms.is_deleted = FALSE)
    AND (mt.is_deleted IS NULL OR mt.is_deleted = FALSE)
    AND (m.is_deleted IS NULL OR m.is_deleted = FALSE)
GROUP BY
    mt.name
ORDER BY
    active_member_count DESC;
  "#;

const DAILY_HOURLY_VISIT_COUNT_QUERY: &str = r#"
SELECT
    CAST(strftime('%w', datetime(entry_time, 'localtime')) AS INTEGER) AS day_of_week,
    CAST(strftime('%H', datetime(entry_time, 'localtime')) AS INTEGER) AS hour_of_day,
    COUNT(*) AS visit_count
FROM
    entry_logs
WHERE
    status = 'allowed'
    AND entry_time >= ?1
    AND entry_time <= ?2
GROUP BY
    day_of_week,
    hour_of_day
ORDER BY
    day_of_week ASC,
    hour_of_day ASC;
"#;

const REVENUE_BY_MEMBERSHIP_TYPE_QUERY: &str = r#"
SELECT
    mt.name AS membership_type_name,
    SUM(mt.price) AS total_revenue, -- Summing the price of the type for each membership instance created
    COUNT(ms.id) AS count
FROM
    membership_types mt
JOIN
    memberships ms ON mt.id = ms.membership_type_id
WHERE
    DATE(ms.purchase_date, 'localtime') >= DATE(?1)
    AND DATE(ms.purchase_date, 'localtime') <= DATE(?2)
    AND (ms.is_deleted IS NULL OR ms.is_deleted = FALSE)
    AND (mt.is_deleted IS NULL OR mt.is_deleted = FALSE)
GROUP BY
    mt.name
ORDER BY
    total_revenue DESC;
"#;

const ACTIVE_MEMBERSHIPS_OVER_TIME_QUERY: &str = r#"
WITH RECURSIVE MonthSeries(month_start, month_end) AS (
    SELECT
        DATE(?1, 'start of month') as month_start,
        DATE(?1, 'start of month', '+1 month', '-1 day') as month_end
    UNION ALL
    SELECT
        DATE(month_start, '+1 month'),
        DATE(month_start, '+2 months', '-1 day')
    FROM MonthSeries
    WHERE DATE(month_start, '+1 month') <= DATE(?2, 'start of month')
)
SELECT
    strftime('%Y-%m', ms.month_start) AS year_month,
    COUNT(DISTINCT mship.member_id) AS active_member_count
FROM
    MonthSeries ms
LEFT JOIN
    memberships mship ON
        (mship.is_deleted IS NULL OR mship.is_deleted = 0 OR mship.is_deleted = FALSE)
        AND mship.start_date <= ms.month_end
        AND (mship.end_date IS NULL OR mship.end_date >= ms.month_start)
        AND mship.status IN ('active', 'pending', 'expired')
GROUP BY
    ms.month_start
ORDER BY
    ms.month_start ASC
"#;
#[tauri::command]
pub async fn get_membership_type_distribution(
    state: State<'_, AppState>,
) -> AppResult<Vec<MembershipTypeDistributionItem>> {
    let rows =
        sqlx::query_as::<_, MembershipTypeDistributionItem>(MEMBERSHIP_TYPE_DISTRIBUTION_QUERY)
            .fetch_all(&state.db_pool)
            .await?;

    Ok(rows)
}

#[tauri::command]
pub async fn get_daily_hourly_visit_count(
    state: State<'_, AppState>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> AppResult<Vec<DailyHourlyVisitCount>> {
    let rows = sqlx::query_as::<_, DailyHourlyVisitCount>(DAILY_HOURLY_VISIT_COUNT_QUERY)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&state.db_pool)
        .await?;

    Ok(rows)
}

#[tauri::command]
pub async fn get_revenue_by_membership_type(
    state: State<'_, AppState>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> AppResult<Vec<RevenueByMembershipTypeItem>> {
    let rows = sqlx::query_as::<_, RevenueByMembershipTypeItem>(REVENUE_BY_MEMBERSHIP_TYPE_QUERY)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&state.db_pool)
        .await?;

    Ok(rows)
}

#[tauri::command]
pub async fn get_active_memberships_over_time(
    state: State<'_, AppState>,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> AppResult<Vec<ActiveMembershipsOverTimeItem>> {
    tracing::info!(
        "Fetching active memberships over time from {} to {}",
        start_date,
        end_date
    );
    let rows =
        sqlx::query_as::<_, ActiveMembershipsOverTimeItem>(ACTIVE_MEMBERSHIPS_OVER_TIME_QUERY)
            .bind(start_date)
            .bind(end_date)
            .fetch_all(&state.db_pool)
            .await?;

    Ok(rows)
}
