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
