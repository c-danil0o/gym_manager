export interface MembershipType {
	id: number;
	name: string;
	duration_days: number | null;
	visit_limit: number | null;
	enter_by: number | null;
	price: number;
	description: string | null;
	is_deleted: boolean;
	is_active: boolean;
	created_at?: Date;
	updated_at?: Date;
}
