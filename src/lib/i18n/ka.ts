export const ka = {
	app: {
		title: 'Exsul',
		tagline: 'ერთიანი სივრცე ბიზნესის ძიებისთვის, ანალიზისთვის და მოქმედებისთვის'
	},
	input: {
		placeholder: 'წერე ნებისმიერ ადგილას',
		hint: '/ მოქმედებები  @ ფაილები  # პროდუქტები  ! აზროვნება'
	},
	state: {
		waking: 'სივრცე იღვიძებს...',
		searching: 'ვეძებ...',
		thinking: 'ვ ა ა ნ ა ლ ი ზ ე ბ...',
		error: 'შეფერხება მოხდა'
	},
	results: {
		found: '{count} შედეგი',
		empty: 'შედეგი ვერ მოიძებნა',
		emptyHint: 'სცადეთ სხვა აღწერა ან დაამატეთ ფოტო',
		summary: 'შეჯამება'
	},
	card: {
		price: 'ფასი',
		stock: 'მარაგი',
		sold: 'გაყიდულია',
		category: 'კატეგორია',
		analyze: 'ფოტოს ანალიზი',
		noImage: 'ფოტო არ არის',
		confidence: 'სიზუსტე',
		matchedBy: 'დამთხვევა',
		upload: {
			hint: 'ჩასმა · ჩაგდება · არჩევა',
			hintSub: 'Ctrl+V ან ფაილის გადმოთრევა',
			replace: 'შეცვლა',
			uploading: 'იტვირთება...',
			notImage: 'აირჩიეთ მხოლოდ სურათის ფაილი',
			tooBig: 'ფაილი ძალიან დიდია (<= 5 მბ)',
			aria: 'ფოტოს დამატება: {title}'
		}
	},
	assistant: {
		label: 'ცოცხალი კონტექსტი',
		errorFallback:
			'AI დროებით მიუწვდომელია. შედეგები ნაჩვენებია ადგილობრივი ძიებიდან.'
	},
	actions: {
		title: 'მოქმედებების სივრცე',
		hint: 'მიმდინარე კონტექსტის ინსტრუმენტები',
		empty: 'მოდულებს მოქმედებები ჯერ არ დაუმატებიათ',
		open: 'მოქმედებები',
		focus: 'წერის დაწყება',
		focusDescription: 'დაუბრუნდით მთავარ კონტექსტს და გააგრძელეთ აზრი',
		newContext: 'ახალი კონტექსტი',
		newContextDescription: 'გაასუფთავეთ მიმდინარე სცენის ისტორია',
		attachFiles: 'ფაილების დამატება',
		attachFilesDescription: 'აირჩიეთ დოკუმენტები შემდეგი AI მოთხოვნისთვის',
		clearFiles: 'ფაილების მოცილება',
		clearFilesDescription: 'მოაცილეთ არჩეული დოკუმენტები AI კონტექსტიდან',
		refreshSearch: 'ძიების განახლება',
		refreshSearchDescription: 'თავიდან ააგეთ პროდუქტებისა და AI მეტამონაცემების ინდექსი',
		createBackup: 'სარეზერვო ასლი',
		createBackupDescription: 'შეინახეთ ადგილობრივი ბაზა და სურათები ერთ არქივში',
		moduleIntro: '{name} მოდული ჩაერთო კონტექსტში',
		moduleBusiness: 'ბიზნესის ანალიზი',
		moduleBusinessDescription: 'მარჟა, მარაგი, ფასები და შემდეგი მოქმედება',
		moduleGovernment: 'სახელმწიფო სერვისები',
		moduleGovernmentDescription: 'ქართული საჯარო რეესტრები და პროცედურები',
		moduleSocial: 'სოციალური არხები',
		moduleSocialDescription: 'Instagram/Facebook კონტენტი ბიზნესის კონტექსტით',
		moduleCompetitors: 'კონკურენტები',
		moduleCompetitorsDescription: 'ფასების, შეთავაზებების და პოზიციონირების შედარება',
		moduleStats: 'სტატისტიკა',
		moduleStatsDescription: 'სექტორული მონაცემები და ბაზრის სიგნალები',
		contextGroup: 'კონტექსტი',
		aiContextGroup: 'AI კონტექსტი',
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
		mock: 'სადემო რეჟიმი',
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
