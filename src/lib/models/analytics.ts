export interface MembershipTypeDistribution {
	membership_type_name: string;
	active_member_count: number;
}

export interface WeeklyHourlyDistribution {
	day_of_week: number;
	hour_of_day: number;
	visit_count: number;
}
export interface ActiveMembershipOverTime {
	year_month: string;
	active_member_count: number;
}
export interface MembershipRevenueMap {
	membership_type_name: string;
	total_revenue: number;
	count: number;
}
