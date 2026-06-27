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
		matchedBy: 'ემთხვევა',
		upload: {
			hint: 'ჩასმა · ჩაგდება · არჩევა',
			hintSub: 'Ctrl+V ან ფაილის გადმოთრევა',
			replace: 'შეცვლა',
			uploading: 'იტვირთება…',
			notImage: 'მხოლოდ სურათის ფაილი',
			tooBig: 'ფაილი ძალიან დიდია (≤ 5 მბ)',
			aria: 'ფოტოს დამატება: {title}'
		}
	},
	assistant: {
		label: 'ასისტენტი',
		errorFallback: 'AI დროებით მიუწვდომელია — შედეგები ნაჩვენებია ლოკალური ძიებიდან'
	},
	actions: {
		title: 'ქმედებების სივრცე',
		hint: 'მიმდინარე კონტექსტის ინსტრუმენტები',
		empty: 'მოდულებს ქმედებები ჯერ არ დაუმატებიათ',
		open: 'ქმედებები',
		focus: 'წერის დაწყება',
		focusDescription: 'დაუბრუნდით ცენტრალურ ველს და განაგრძეთ აზრი',
		newContext: 'ახალი კონტექსტი',
		newContextDescription: 'მიმდინარე სცენის ისტორიის გასუფთავება',
		attachFiles: 'ფაილების დამატება',
		attachFilesDescription: 'აირჩიეთ დოკუმენტები შემდეგი AI-მოთხოვნისთვის',
		clearFiles: 'ფაილების მოცილება',
		clearFilesDescription: 'არჩეული დოკუმენტების მოცილება AI-კონტექსტიდან',
		refreshSearch: 'ძიების განახლება',
		refreshSearchDescription: 'პროდუქტებისა და AI-მეტამონაცემების ლოკალური ინდექსის თავიდან აგება',
		createBackup: 'სარეზერვო ასლის შექმნა',
		createBackupDescription: 'ლოკალური ბაზისა და სურათების ერთ არქივში შენახვა',
		contextGroup: 'კონტექსტი',
		aiContextGroup: 'AI-კონტექსტი',
		dataGroup: 'მონაცემები'
	},
	files: {
		attached: 'ფაილები კონტექსტში · {count}',
		attachedRejected: 'დამატებულია {count} · უარყოფილია {rejected}',
		inContext: '{count} კონტექსტში',
		remove: 'ფაილის მოცილება: {name}',
		label: 'დამატებული ფაილები'
	},
	system: {
		indexUpdated: 'ინდექსი განახლდა · {count}',
		backupReady: 'სარეზერვო ასლი მზადაა'
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
