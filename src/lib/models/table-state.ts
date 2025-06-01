export interface FilterField {
	field: string;
	value: string;
}

export interface QueryRequest {
	page?: number;
	per_page?: number;
	order_by?: string;
	order_direction?: 'asc' | 'desc';
	search_string?: string;
	filter_fields?: FilterField[];
}

export interface QueryResponse<T> {
		data: T[];
		total: number;
		page: number;
		per_page: number;
		total_pages: number;
	}
