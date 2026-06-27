# Action Island integration

`ActionIsland` is the shared overlay for AI, business and analytics modules. The
page mounts it once; feature branches only register actions.

```ts
import { defineAction, registerAction } from '$lib/actions';

const registration = registerAction(
	defineAction({
		id: 'analytics.inventory-overview',
		label: 'მარაგების მიმოხილვა',
		description: 'მთავარი მაჩვენებლების ანალიზი',
		icon: '⌁',
		group: 'ანალიტიკა',
		tone: 'accent',
		order: 200,
		run: async ({ signal, close }) => {
			// Call a typed service; respect `signal` for cancellable work.
			close();
		}
	})
);

// Call when the owning module is destroyed or disabled.
registration.unregister();
```

Rules for parallel branches:

- Namespace IDs by module: `ai.*`, `business.*`, `analytics.*`, `files.*`.
- Do not edit `ActionIsland.svelte` to add a button; register an action instead.
- Duplicate IDs fail fast. Use `{ replace: true }` only for an intentional HMR
  adapter or an agreed replacement.
- Put business and provider logic behind typed services, not in `run`.
- Use `visible` for contextual actions and `enabled` for temporarily unavailable
  actions.
- The overlay already owns `Alt+C`, `Escape`, focus management, mouse-shake,
  scrolling, reduced-motion behavior and async error presentation.

The built-in file action uses a native picker and opaque session IDs. A feature
must never accept an arbitrary filesystem path from the UI.
