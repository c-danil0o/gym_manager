import { superValidate } from 'sveltekit-superforms';
import { formSchema } from '../../schemas/login_schema';
import { zod } from 'sveltekit-superforms/adapters';

export const load = async () => {
	return {
		form: await superValidate(zod(formSchema))
	};
};
