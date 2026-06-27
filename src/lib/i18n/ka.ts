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
	},
	analytics: {
		title: 'ბიზნესის მდგომარეობა',
		subtitle: 'მოკლე ანალიზი თქვენი მარაგისა და გაყიდვების შესახებ',
		loading: 'ვაანალიზებ…',
		empty: 'ჯერ არ არის საკმარისი მონაცემი',
		bucketLabel: 'პერიოდი',
		buckets: {
			day: 'დღე',
			week: 'კვირა',
			month: 'თვე'
		},
		kpi: {
			items: 'პროდუქცია',
			stockValue: 'მარაგის ღირებულება',
			revenue: 'მთლიანი შემოსავალი',
			soldUnits: 'გაყიდული ერთეული',
			lowStock: 'დაბალი მარაგი',
			aiCoverage: 'AI დაფარვა'
		},
		topSellers: 'ტოპ გაყიდვები',
		deadStock: 'უმოძრაო მარაგი',
		deadStockHint: 'მარაგი გროვდება — განიხილეთ ფასის შემცირება ან აქცია',
		stockOut: 'მალე ამოიწურება',
		stockOutHint: 'პროგნოზი გაყიდვების ტემპით — დროულად შეუკვეთეთ',
		stockOutNoHistory: 'საკმარისი ისტორია არ არის პროგნოზისთვის',
		stockOutDaysLeft: '{days} დღე',
		stockOutVelocity: '{v} ერთ./დღეში',
		heatmap: {
			title: 'როდის ყიდიან',
			hint: 'კვირის დღე × საათი — მაქსიმალური აქტივობა',
			weekdays: ['მზე', 'ორშ', 'სამ', 'ოთხ', 'ხათ', 'პარ', 'შაბ'],
			hours: ['00', '02', '04', '06', '08', '10', '12', '14', '16', '18', '20', '22'],
			empty: 'ჯერ არ არის საკმარისი გაყიდვა',
			periods: {
				'7': '7 დღე',
				'30': '30 დღე',
				'90': '90 დღე',
				'all': 'ყველა'
			},
			peakLabel: 'პიკი',
			peakHint: '{weekday} {hour}:00 — შემოსავლის {pct}% ამ დროს მოდის',
			cellHint: '{weekday} {hour}:00 — {units} ერთ., {revenue}₾',
			scale: 'ნაკლები → მეტი',
			noRevenue: '0₾'
		},
		lowStock: 'დაბალი მარაგის სია',
		categories: 'კატეგორიები',
		categoriesHint: 'შემოსავალი და მარაგი კატეგორიების მიხედვით',
		activity: 'ბოლო მოვლენები',
		noActivity: 'მოვლენები ჯერ არ ჩანს',
		noLowStock: 'ყველა პროდუქტი ნორმალურ მარაგშია',
		noDeadStock: 'უმოძრაო მარაგი არ გაქვთ',
		noTopSellers: 'გაყიდვები ჯერ არ დაფიქსირდა',
		noCategories: 'კატეგორიები ჯერ არ არის',
		revenueLabel: 'შემოსავალი',
		stockLabel: 'მარაგი',
		soldLabel: 'გაყიდული'
	}
} as const;

export type Dict = typeof ka;
