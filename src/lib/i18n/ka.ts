// ქართული ლოკალიზაცია (Georgian strings).
// Ключи сгруппированы по смысловым неймспейсам — удобно расширять и переводить.
// Тексты — смысловые, не дословный перевод; естественная бизнес-лексика.

export const ka = {
	app: {
		title: 'Exsul',
		tagline: 'თქვენი ბიზნესის AI-სივრცე'
	},
	input: {
		placeholder: 'დაწერეთ — მოძებნე, გააანალიზე, აჩვენე…',
		hint: 'Enter — გასაგზავნად'
	},
	state: {
		waking: 'სივრცე იღვიძებს…',
		searching: 'ვეძებ…',
		thinking: 'ვაანალიზებ…',
		error: 'შეფერხება მოხდა'
	},
	results: {
		found: '{count} შედეგი',
		empty: 'ვერაფერი მოიძებნა',
		emptyHint: 'სცადეთ სხვა სიტყვა ან ატვირთეთ ფოტო',
		summary: 'რეზიუმე'
	},
	card: {
		price: 'ფასი',
		stock: 'მარაგი',
		sold: 'გაყიდული',
		category: 'კატეგორია',
		analyze: 'ფოტოს ანალიზი',
		noImage: 'ფოტო არ არის',
		confidence: 'სიზუსტე',
		matchedBy: 'ემთხვევა'
	},
	assistant: {
		label: 'ასისტენტი',
		errorFallback: 'AI დროებით მიუწვდომელია — შედეგები ნაჩვენებია ლოკალური ძიებიდან'
	},
	provider: {
		label: 'AI პროვაიდერი',
		openai: 'OpenAI',
		gemini: 'Gemini',
		claude: 'Claude',
		mock: 'სადემო (გასაღების გარეშე)',
		notConfigured: 'გასაღები არ არის'
	},
	units: {
		currency: '₾'
	},
	common: {
		close: 'დახურვა',
		retry: 'ხელახლა',
		demo: 'სადემო მონაცემების ჩატვირთვა'
	}
} as const;

export type Dict = typeof ka;
