export interface MembershipType {
	id: number;
	name: string;
	duration_days: number | null;
	visit_limit: number | null;
	price: number;
	description: string | null;
	is_deleted: boolean; // Assuming you have this for soft deletes
	// Add created_at, updated_at if you want to display them
}
