;(() => {
	var implementors = Object.fromEntries([
		[
			'tower_governor',
			[
				[
					'impl&lt;K, S, ReqBody&gt; Service&lt;Request&lt;ReqBody&gt;&gt; for <a class="struct" href="tower_governor/governor/struct.Governor.html" title="struct tower_governor::governor::Governor">Governor</a>&lt;K, NoOpMiddleware, S&gt;<div class="where">where\n    K: <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>,\n    S: Service&lt;Request&lt;ReqBody&gt;, Response = Response&lt;Body&gt;&gt;,</div>',
				],
				[
					'impl&lt;K, S, ReqBody&gt; Service&lt;Request&lt;ReqBody&gt;&gt; for <a class="struct" href="tower_governor/governor/struct.Governor.html" title="struct tower_governor::governor::Governor">Governor</a>&lt;K, StateInformationMiddleware, S&gt;<div class="where">where\n    K: <a class="trait" href="tower_governor/key_extractor/trait.KeyExtractor.html" title="trait tower_governor::key_extractor::KeyExtractor">KeyExtractor</a>,\n    S: Service&lt;Request&lt;ReqBody&gt;, Response = Response&lt;Body&gt;&gt;,</div>',
				],
			],
		],
	])
	if (window.register_implementors) {
		window.register_implementors(implementors)
	} else {
		window.pending_implementors = implementors
	}
})()
//{"start":57,"fragment_lengths":[1068]}
